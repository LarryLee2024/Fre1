# App 层与启动引导架构

> Version: 1.1
> Status: Proposed
> 来源：`docs/其他/30.md` 第983-1008行（App层）、`docs/architecture.md`（七层架构）、`docs/architecture/layer-contracts.md`（App层合约）
> 宪法依据：`docs/AI开发宪法完整版.md` v1.6（第1.3节架构分层、第2.3.6/2.3.8节状态管理与Schedule、第3.0节模块边界）

---

## 1. App 层哲学

App 层是游戏的**装配器（Assembler）**，不是执行器。

```
App = 全局唯一组装者
  - 知道所有模块的存在
  - 不包含任何业务逻辑
  - 唯一职责：组装整个游戏
```

类比：App 层就像一台汽车的总装线——它知道发动机、变速箱、轮胎各在哪里，但自己从不运转。所有实际功能（发动机燃烧、轮胎转动）都由各 Plugin 自己完成。

### 判断标准

> 这是把游戏"启动起来"的代码吗？
> → 是 → App 层。
> → 否 → 其他层。

---

## 2. 目录结构

```
src/app/
├── app_plugin.rs       # 主 Plugin，注册所有子 Plugin
├── game_state.rs       # AppState 定义 (MainMenu→LevelSelect→InGame→GameOver)
├── schedules.rs        # Schedule 定义
├── sets.rs             # SystemSet 定义
├── startup.rs          # 启动逻辑
├── shutdown.rs         # 关闭逻辑
└── plugins.rs          # Plugin 汇集注册
```

| 文件 | 职责 | 禁止 |
|------|------|------|
| `app_plugin.rs` | 主 Plugin 入口，串联所有子 Plugin | 禁止包含业务逻辑 |
| `game_state.rs` | AppState 状态机定义 | 禁止在状态回调中执行业务逻辑 |
| `schedules.rs` | Schedule 定义与配置 | 禁止定义 System |
| `sets.rs` | SystemSet 定义与排序约束 | 禁止在 Set 中注册 System |
| `startup.rs` | 启动时的轻量初始化 | 禁止创建业务 Entity |
| `shutdown.rs` | 关闭时的清理逻辑 | 禁止修改业务状态 |
| `plugins.rs` | 所有 Plugin 的汇集与注册 | 禁止跳过任何 Plugin |

---

## 3. AppState 状态机

App 顶层状态机，控制游戏的宏观流程：

```
AppState
├── MainMenu        # 主菜单：进入游戏的入口
├── LevelSelect     # 关卡选择：选关、组队
├── InGame          # 游戏中：核心战斗循环
│   └── TurnPhase (SubState)   # 回合阶段（仅 InGame 激活）
│       ├── SelectUnit         # 选择行动单位
│       ├── MoveUnit           # 移动单位
│       ├── ActionMenu         # 动作菜单（攻击/技能/道具/待机）
│       ├── SelectTarget       # 选择目标
│       ├── ExecuteAction      # 执行动作
│       ├── WaitAction         # 等待动画/结算
│       └── TurnEnd            # 回合结束结算
└── GameOver        # 游戏结束：胜利/失败画面
```

### 各状态职责

| 状态 | 职责 | 入场系统（OnEnter） | 退场系统（OnExit） |
|------|------|---------------------|---------------------|
| `MainMenu` | 插件注册、资产预加载、用户设置 | 初始化 UI、加载菜单资源 | 清理菜单 UI |
| `LevelSelect` | 关卡选择、队伍编成 | 显示关卡列表、队伍编辑器 | 清理选择 UI |
| `InGame` | 核心战斗循环 | 加载地图、生成单位、初始化回合 | 保存战斗结果、清理战场 |
| `GameOver` | 胜利/失败、存档、返回菜单 | 显示结算画面、触发存档 | 清理结算 UI |

### TurnPhase SubState 规则

> **优化来源**：`docs/其他/35.md` — SubState 设计亮点与避坑建议
> **宪法依据**：🟩 2.3.6 状态管理规范（States/SubStates/ComputedStates）

- 🟥 **仅在 `AppState::InGame` 时激活** — 〔宪法 2.3.6〕
- 🟥 **状态转换必须通过 `NextState<TurnPhase>` 驱动** — 〔宪法 2.3.6〕
- 🟥 **禁止在 OnEnter 中执行跨阶段跳转** — 〔宪法 16.0.3 副作用透明〕
- 🟩 **每个阶段系统必须轻量，重型逻辑拆分到独立系统** — 〔宪法 16.0.1 状态切换轻量〕
- 🟥 **Bevy 0.15+ 要求显式声明 `#[derive(SubStates)]`**，TurnPhase 必须在 `InGame` 状态下注册

```rust
// ✅ Bevy 0.15+ SubStates 声明
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SubStates)]
#[source(AppState = AppState::InGame)]
pub enum TurnPhase {
    SelectUnit,
    MoveUnit,
    ActionMenu,
    SelectTarget,
    ExecuteAction,
    WaitAction,
    TurnEnd,
}
```

### WaitAction 阶段：逻辑与动画分离

> **优化来源**：`docs/其他/35.md` — SRPG 核心痛点「逻辑与动画的时间轴撕裂」

SRPG 最怕"逻辑已经算完了（单位已死），但死亡动画还在播，玩家狂点鼠标触发了下一个回合"。

🟥 **WaitAction 阶段只等待动画事件，不执行逻辑。逻辑在 ExecuteAction 阶段全部完成。**

```
ExecuteAction（逻辑层）
  ├─ combat_intent_system         # 生成战斗意图
  ├─ effect_generate              # 效果管线 - 生成
  ├─ effect_modify                # 效果管线 - 修饰
  ├─ effect_execute               # 效果管线 - 执行
  ├─ buff_resolve_system          # Buff 结算
  └─ 产出 GameEvent 到 ActionTimeline

WaitAction（等待层）
  └─ wait_for_animation           # 仅消费 ActionTimeline，等待动画播放完毕
      └─ 当 ActionTimeline 为空 → 推进到 TurnEnd
```

引入 `ActionTimeline`（行动时间轴/命令队列）Resource：
- 逻辑层只产出 `GameEvent`（如 UnitMoved, UnitDamaged）到 ActionTimeline
- UI/动画层从 ActionTimeline 消费事件并播放动画
- 只有当 ActionTimeline 为空时，TurnPhase 才从 WaitAction 推进到 TurnEnd

---

## 4. 启动序列

```
main.rs
  │
  ├─ App::new()
  │   │
  │   ├─ add_plugins(DefaultPlugins)
  │   │   ├─ WindowPlugin  (窗口配置)
  │   │   ├─ AssetPlugin    (资产路径配置)
  │   │   └─ ...            (其他 Bevy 默认插件)
  │   │
  │   ├─ add_plugins(EguiPlugin)          // 调试基础设施
  │   │
  │   └─ add_plugins(AppPlugin)           // app_plugin.rs —— 主入口
  │       │
  │       ├─ init_resource(AppSettings)   // 全局配置
  │       │
  │       ├─ ─── Shared 插件（零依赖） ───
  │       ├─ SharedPlugin                 // shared/
  │       │
  │       ├─ ─── Infrastructure 插件 ───
  │       ├─ InfrastructurePlugin         // infrastructure/
  │       │   ├─ LogPlugin               //   日志系统
  │       │   └─ AuditPlugin             //   审计系统
  │       │
  │       ├─ ─── Core 插件（数据层 + 逻辑层） ───
  │       ├─ CorePlugins                  // core/
  │       │   ├─ EffectPlugin            //   效果管线
  │       │   ├─ ModifierRulePlugin      //   修饰规则
  │       │   ├─ AttributeDefPlugin      //   属性定义
  │       │   └─ TagDefPlugin            //   标签定义
  │       │
  │       ├─ ─── Content 插件 ───
  │       ├─ ContentPlugin                // content/
  │       │   ├─ SkillPlugin             //   技能注册
  │       │   ├─ BuffPlugin              //   Buff 注册
  │       │   ├─ AiBehaviorPlugin        //   AI 行为注册
  │       │   ├─ EquipmentPlugin         //   装备注册
  │       │   ├─ InventoryPlugin         //   物品注册
  │       │   └─ AssetsPlugin            //   资产加载器
  │       │
  │       ├─ ─── 逻辑层插件 ───
  │       ├─ LogicPlugins                 // core/ (业务逻辑)
  │       │   ├─ TurnPlugin             //   回合状态机
  │       │   ├─ MapPlugin              //   地图系统
  │       │   ├─ CharacterPlugin        //   角色系统
  │       │   ├─ BattlePlugin           //   战斗系统
  │       │   ├─ AiPlugin               //   AI 决策
  │       │   └─ CampaignPlugin         //   战役系统
  │       │
  │       ├─ ─── UI 插件 ───
  │       ├─ UiPlugin                     // ui/
  │       │   └─ InputPlugin             //   输入处理
  │       │
  │       ├─ ─── Debug 插件（仅开发模式） ───
  │       └─ #[cfg(feature = "dev")]
  │           └─ DebugPlugin             // debug/
  │
  └─ add_systems(OnEnter(AppState::MainMenu), startup_setup)
  └─ run()
```

### 当前实现对照

| 设计目标 | 当前 main.rs 状态 | 差距 |
|----------|-------------------|------|
| AppPlugin 统一入口 | ❌ main.rs 直接注册所有 Plugin | 需重构为 AppPlugin 模式 |
| 分层分组注册 | ⚠️ 已有分组但不够清晰 | 需更明确的分层分组 |
| AppState 状态机 | ⚠️ 未体现 | 需补充 game_state.rs |
| Schedule/SystemSet | ⚠️ 未体现 | 需补充 schedules.rs、sets.rs |
| Debug 条件编译 | ❌ DebugPlugin 无条件注册 | 需加 `#[cfg(feature = "dev")]` |

---

## 5. Schedule 设计

### 标准 Bevy Schedule

| Schedule | 用途 | 说明 |
|----------|------|------|
| `PreUpdate` | 输入处理 | 读取原始输入事件，转换为游戏输入 |
| `Update` | 游戏逻辑 | 核心业务系统运行（TurnPhase 控制） |
| `PostUpdate` | UI 更新与清理 | ViewModel 刷新、临时资源清理 |
| `FixedUpdate` | 游戏逻辑帧 | 固定步长游戏逻辑（物理/动画帧） |
| `First` | 最先执行 | 全局状态初始化 |
| `Last` | 最后执行 | 全局清理 |

### TurnPhase 内的系统顺序

在 `Update` Schedule 中，系统按 TurnPhase 驱动：

```
TurnPhase::SelectUnit
  ├─ highlight_reachable_tiles    # 高亮可达格子
  ├─ highlight_enemies_in_range   # 高亮范围内敌人
  └─ select_unit_on_click         # 点击选择单位

TurnPhase::MoveUnit
  ├─ show_movement_range          # 显示移动范围
  ├─ move_unit_to_target          # 移动单位到目标格
  └─ path_find_system             # 寻路计算

TurnPhase::ActionMenu
  └─ show_action_menu             # 显示动作菜单

TurnPhase::SelectTarget
  ├─ highlight_targets            # 高亮可选目标
  └─ select_target_on_click       # 点击选择目标

TurnPhase::ExecuteAction
  ├─ combat_intent_system         # 生成战斗意图
  ├─ effect_generate              # 效果管线 - 生成
  ├─ effect_modify                # 效果管线 - 修饰
  ├─ effect_execute               # 效果管线 - 执行
  └─ buff_resolve_system          # Buff 结算

TurnPhase::WaitAction
  └─ wait_for_animation           # 等待动画/结算完成

TurnPhase::TurnEnd
  ├─ turn_end_cleanup             # 回合结束清理
  ├─ victory_defeat_check         # 胜负判定
  └─ next_turn_or_phase           # 下一回合/阶段
```

### Schedule 规则

- 🟥 **禁止在 PreUpdate 中执行游戏逻辑**（PreUpdate 专用于输入处理） — 〔宪法 2.3.8〕
- 🟥 **禁止在 PostUpdate 中修改游戏状态**（PostUpdate 专用于 UI 更新） — 〔宪法 2.3.8〕
- 🟩 **FixedUpdate 用于需要固定步长的逻辑**（物理、动画帧） — 〔宪法 2.3.8〕

> **优化来源**：`docs/其他/35.md` — `.chain()` 致命性能陷阱与 EffectPipelineSchedule 方案

#### ⚠️ `.chain()` 性能陷阱（Bevy 并行杀手）

🟥 **`.chain()` 强制系统串行执行，彻底破坏 Bevy 多线程并行优势。除非是单系统管道，否则禁止使用 `.chain()`。**

```
// 🟥 灾难：全部串行，20 个单位同时结算时帧率暴跌
app.add_systems(Update, (
    combat_intent_system,
    effect_generate,
    effect_modify,
    effect_execute,
).chain());  // ← 破坏并行！

// ✅ 正确：用 after()/before() 约束，Bevy 自动并行化无冲突系统
app.add_systems(Update, (
    effect_generate.after(combat_intent_system),
    effect_modify.after(effect_generate),
    effect_execute.after(effect_modify),
));
```

#### EffectPipelineSchedule：严格串行管线

🟥 **效果管线（Generate→Modify→Execute）是唯一需要严格串行的子系统。使用自定义 Schedule + `run_schedule()` 而非 `.chain()`。**

> **优化来源**：`docs/其他/35.md` — 自定义 Schedule 替代 `.chain()` 方案

```rust
/// 效果管线专属 Schedule — 在 Update 中通过 run_schedule 手动触发。
/// 保证 Generate→Modify→Execute 严格串行，但不占用 Update 的并行能力。
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EffectPipelineSchedule;

// 注册 EffectPipelineSchedule 内部顺序
app.add_schedule(EffectPipelineSchedule);
app.configure_sets(EffectPipelineSchedule, (
    EffectGenerateSet,
    EffectModifySet.after(EffectGenerateSet),
    EffectExecuteSet.after(EffectModifySet),
));

// 在 Update 中触发
fn trigger_effect_pipeline(world: &mut World) {
    world.run_schedule(EffectPipelineSchedule);
}
```

#### 确定性逻辑使用 FixedUpdate

🟥 **确定性战斗结算必须放在 FixedUpdate 中（固定 10Hz tick），Update 只处理输入和 UI 表现。避免帧率波动影响战斗数值。**

> **优化来源**：`docs/其他/35.md` — FixedUpdate Set 划分建议

---

## 6. SystemSet 设计

### 系统排序约束

```
InputSet → CommandSet → LogicSet → EffectSet → ViewModelSet → UISet
```

| SystemSet | 职责 | 包含的典型系统 |
|-----------|------|----------------|
| `InputSet` | 输入处理 | `keyboard_input`, `mouse_input`, `touch_input` |
| `CommandSet` | 命令分发 | `command_handler`, `ui_command_dispatch` |
| `LogicSet` | 业务逻辑 | `turn_system`, `movement_system`, `combat_system` |
| `EffectSet` | 效果管线 | `generate_effects`, `modify_effects`, `execute_effects` |
| `ViewModelSet` | 视图模型更新 | `update_battle_ui`, `update_buff_panel`, `update_turn_queue` |
| `UISet` | UI 渲染 | `render_ui`, `refresh_ui_panels` |

### 排序规则

```rust
app.configure_sets(Update, (
    InputSet,
    CommandSet.after(InputSet),
    LogicSet.after(CommandSet),
    EffectSet.after(LogicSet),
    ViewModelSet.after(EffectSet),
    UISet.after(ViewModelSet),
));
```

### Set 规则

- 🟥 **禁止绕过 Set 排序直接注册 System**（所有 System 必须归属某个 Set）
- 🟥 **禁止在 Set 内部执行跨 Set 的逻辑**（Set 之间只传递数据）
- 🟩 **Set 可以有子 Set**（如 LogicSet 包含 TurnSet、CombatSet、MapSet）

---

## 7. 关闭序列

```
游戏关闭
  │
  ├─ OnExit(AppState::InGame)
  │   ├─ save_game_state()           # 保存游戏状态（可选）
  │   └─ cleanup_battle_resources()  # 清理战斗资源
  │
  ├─ OnExit(AppState::GameOver)
  │   └─ cleanup_game_over_ui()      # 清理结算 UI
  │
  └─ App 关闭
      ├─ 各 Plugin::cleanup()        # 各 Plugin 自行清理
      └─ 释放全局资源
```

### 关闭规则

- 🟥 **禁止在关闭时修改业务状态**（关闭是清理，不是操作）
- 🟩 **关闭逻辑必须幂等**（多次调用不会出错）
- 🟩 **优先使用 Bevy 的 OnExit 自动清理**（而非手动清理）

> **优化来源**：`docs/其他/35.md` — OnExit 清理的"内存泄漏"刺客

#### BattleEntity 标记组件清理

🟥 **使用 `#[derive(Component)] struct BattleEntity` 标记组件 + `despawn_recursive()` 统一清理，禁止手动逐个 despawn。**

手动 despawn 的陷阱：极容易漏掉子 Entity（血条、特效挂载点），导致"幽灵单位"出现在下一局。

```rust
/// 战斗 Entity 标记组件。所有在 InGame 状态生成的 Entity 必须带上此组件。
#[derive(Component)]
pub struct BattleEntity;

// ✅ OnExit 统一清理 — 一个系统清理所有战斗 Entity
fn cleanup_battle_entities(
    mut commands: Commands,
    query: Query<Entity, With<BattleEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive(); // 包含所有子 Entity
    }
}

// ✅ OnEnter 重置资源 — 而非 OnExit 手动清理
fn reset_battle_resources(mut turn_state: ResMut<TurnState>) {
    *turn_state = TurnState::default();
}
```

规则：
- 🟥 禁止在 OnExit 里手动清理特定业务 Resource，让 Bevy 的 Res 自然覆盖或在 OnEnter 时重置
- 🟥 禁止在 OnExit 中写逐个 Entity 的 despawn 逻辑
- 🟩 OnExit 只做一件事：`commands.entity(e).despawn_recursive()` with `BattleEntity` marker

---

## 8. 禁止事项

> **宪法依据**：〔宪法 3.0.2 Plugin是唯一对外入口〕、〔宪法 1.1.4 逻辑与表现分离〕、〔宪法 1.1.2 定义与实例分离〕

| 禁止项 | 理由 | 替代方案 | 宪法条款 |
|--------|------|----------|----------|
| 🟥 App 层包含任何业务逻辑 | App 只是装配器 | 业务逻辑放在各 Plugin 内 | 3.0.2 |
| 🟥 App 层直接创建 Entity | Entity 是运行时概念 | 由各业务 Plugin 的 startup 创建 | 2.1.1 |
| 🟥 App 层硬编码游戏数值 | 违反数据驱动原则 | 数值放在 RON 配置文件中 | 1.1.2 |
| 🟥 跳过 Plugin 直接注册 System | 破坏模块边界 | 通过 Plugin 注册 System | 3.0.2 |
| 🟥 App 层修改任何业务状态 | App 只读不写 | 由业务 Plugin 通过 System 修改 | 1.1.4 |

---

## 9. 依赖规则

> **宪法依据**：〔宪法 1.3.2 依赖方向铁则〕领域层不得依赖应用层与表现层，应用层不得依赖表现层

```
App      → 任意层     ✅（唯一特权：全局组装者）〔宪法 1.3.1〕
Core     → Shared     ✅（纯领域逻辑只依赖工具）〔宪法 1.4.1 核心领域与引擎解耦〕
Shared   → 无         ✅（叶子节点，零外部依赖）
Infra    → Core, Shared     ✅
Content  → Core, Infra, Shared  ✅
UI       → ViewModel only   ✅〔宪法 1.3.2 表现层不依赖领域层〕
Debug    → Core（只读）      ✅
Modding  → Core, Shared, Infra, Content  ✅
```

### 关键约束

- 🟥 **没有任何层依赖 App**（App 是顶层，不可被依赖）
- 🟥 **App 不能被任何层 import**（App 只注册，不暴露接口）
- 🟩 **App 是唯一允许全局视野的层**（它知道所有模块的存在）

---

## 10. 交叉引用

| 文档 | 关系 |
|------|------|
| `docs/AI开发宪法完整版.md` | 宪法第1.3节架构分层、第2.3.6/2.3.8节状态管理与Schedule、第3.0节模块边界 |
| `docs/architecture.md` | 七层架构总览、依赖规则、插件注册顺序 |
| `docs/architecture/layer-contracts.md` | App 层完整合约（第36-79行） |
| `docs/architecture/project-structure.md` | App 目录结构（第70-78行） |
| `docs/architecture/plugin-design.md` | Plugin 组织与注册详细设计 |
| `docs/architecture/plugin_contract_rules.md` | Plugin 边界与依赖契约 |
| `docs/其他/30.md` | 原始架构来源（第983-1008行） |

---

## 附录 A：重构路线

当前 `main.rs` 直接注册所有 Plugin 的方式需要重构为 AppPlugin 模式：

1. 创建 `src/app/app_plugin.rs`，实现 `AppPlugin` 结构体
2. 将 `main.rs` 中的 Plugin 注册逻辑迁移到 `AppPlugin::build()`
3. `main.rs` 只保留 `App::new().add_plugins(AppPlugin).run()`
4. 补充 `game_state.rs`、`schedules.rs`、`sets.rs`

重构后的 `main.rs`：

```rust
use bevy::prelude::*;
use app::AppPlugin;

fn main() {
    App::new()
        .add_plugins(AppPlugin)
        .run();
}
```
