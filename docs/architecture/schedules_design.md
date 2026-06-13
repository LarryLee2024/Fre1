# Bevy Schedule 与 SystemSet 组织

> Version: 1.0
> Status: Proposed
> 来源：`docs/其他/31遗漏.md` Section 二（第77-93行）

---

## 1. 概述

Bevy 的 Schedule 和 SystemSet 是控制**系统执行顺序**的核心机制。90% 的 Bevy 奇怪 Bug 来自系统执行顺序问题。本设计定义：

- 自定义 Schedule 的组织
- SystemSet 层级结构与排序约束
- 关键顺序要求（Damage → Buff → Attribute）
- 状态门控调度
- 并行 vs 顺序执行策略

与 `app-bootstrap.md` 的关系：**本文档是 Schedule/SystemSet 的详细设计，`app-bootstrap.md` 是 App 层的启动装配概览**。本文档定义"系统怎么排"，`app-bootstrap.md` 定义"Plugin 怎么注册"。

---

## 2. 设计

### 2.1 Custom Schedules

在 Bevy 默认 Schedule 之外，项目定义以下自定义 Schedule：

```rust
/// 自定义 Schedule 定义
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputSchedule;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogicSchedule;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PresentationSchedule;
```

#### Schedule 职责

| Schedule | 职责 | Bevy 内置对应 | 说明 |
|----------|------|-------------|------|
| InputSchedule | 输入处理 | PreUpdate | 读取原始输入，转换为游戏命令 |
| Update（默认） | 游戏逻辑 | Update | 核心业务系统（TurnPhase 控制） |
| LogicSchedule | 复杂逻辑编排 | — | Effect Pipeline、战斗结算等需要严格顺序的逻辑 |
| PresentationSchedule | 表现层 | PostUpdate | UI 更新、动画、音效 |
| FixedUpdate | 固定步长逻辑 | FixedUpdate | 物理、动画帧（当前未使用） |

#### Schedule 注册

```rust
app.add_schedule(InputSchedule)
    .add_schedule(LogicSchedule)
    .add_schedule(PresentationSchedule);
```

#### Schedule 执行顺序

```
每帧执行顺序：
PreUpdate → InputSchedule → Update → LogicSchedule → PostUpdate → PresentationSchedule → Last
```

---

### 2.2 SystemSet 层级结构

```
InputSet
  ├── KeyboardInputSet      # 键盘输入
  ├── MouseInputSet         # 鼠标输入
  └── TouchInputSet         # 触摸输入

CommandSet
  └── CommandDispatchSet    # 命令分发

LogicSet
  ├── TurnSet               # 回合管理
  ├── MovementSet           # 移动与寻路
  ├── CombatSet             # 战斗逻辑
  ├── BuffSet               # Buff 管理
  └── AISet                 # AI 决策

EffectSet
  ├── EffectGenerateSet     # 效果生成
  ├── EffectModifySet       # 效果修饰
  └── EffectExecuteSet      # 效果执行

ViewModelSet
  ├── BattleViewModelSet    # 战斗视图模型
  ├── BuffViewModelSet      # Buff 视图模型
  └── TurnViewModelSet      # 回合视图模型

UISet
  ├── BattleUISet           # 战斗 UI 面板
  ├── BuffUISet             # Buff UI 面板
  └── DebugUISet            # 调试 UI 面板
```

#### Set 排序定义

```rust
app.configure_sets(Update, (
    InputSet,
    CommandSet.after(InputSet),
    LogicSet.after(CommandSet),
    EffectSet.after(LogicSet),
    ViewModelSet.after(EffectSet),
    UISet.after(ViewModelSet),
));

// LogicSet 内部排序
app.configure_sets(LogicSet, (
    TurnSet,
    MovementSet.after(TurnSet),
    CombatSet.after(MovementSet),
    BuffSet.after(CombatSet),
    AISet.after(BuffSet),
));

// EffectSet 内部排序（管线三步）
app.configure_sets(EffectSet, (
    EffectGenerateSet,
    EffectModifySet.after(EffectGenerateSet),
    EffectExecuteSet.after(EffectModifySet),
));
```

---

### 2.3 关键顺序约束

#### 必须严格顺序执行的系统

| 先执行 | 后执行 | 强制等级 | 原因 |
|--------|--------|---------|------|
| `damage_calculation` | `buff_application` | 🟥 | 伤害必须在 Buff 应用前完成 |
| `buff_application` | `attribute_recalculate` | 🟥 | Buff 修改属性后需重算 |
| `effect_generate` | `effect_modify` | 🟥 | 管线三步顺序 |
| `effect_modify` | `effect_execute` | 🟥 | 管线三步顺序 |
| `combat_intent_system` | `effect_generate` | 🟥 | 意图必须在效果前 |
| `turn_end_cleanup` | `victory_check` | 🟥 | 清理后才能正确判定 |
| `victory_check` | `round_end` | 🟥 | 判定后才能结束回合 |
| `all logic systems` | `view_model_update` | 🟥 | 逻辑完成后刷新 UI |
| `view_model_update` | `ui_render` | 🟥 | 模型更新后渲染 |

#### 可以并行执行的系统

| 系统组 | 并行条件 | 说明 |
|--------|---------|------|
| KeyboardInput + MouseInput + TouchInput | 读取不同资源 | 输入处理互相独立 |
| BattleViewModel + BuffViewModel + TurnViewModel | 读取不同组件 | 视图模型互不依赖 |
| BattleUISet + BuffUISet | 读取不同 ViewModel | UI 面板互不依赖 |
| AI 决策（不同单位） | 读写不同 Entity | 可并行但需注意共享状态 |

#### 禁止并行的系统

| 系统组 | 原因 |
|--------|------|
| effect_generate ↔ effect_modify | 严格顺序依赖 |
| effect_modify ↔ effect_execute | 严格顺序依赖 |
| damage_calculation ↔ buff_application | 严格顺序依赖 |
| 任意写同一 Resource 的系统 | 数据竞争 |

---

### 2.4 状态门控调度

大多数系统只在特定 AppState 和 BattlePhase 下运行。

#### AppState 门控

```rust
// 所有战斗系统只在 InGame 时运行
app.configure_sets(Update, (
    LogicSet,
    EffectSet,
    ViewModelSet,
).run_if(in_state(AppState::InGame)));

// UI 系统在所有状态运行
app.configure_sets(Update, (
    UISet,
).run_if(in_state(AppState::MainMenu).or_else(
    in_state(AppState::InGame)).or_else(
    in_state(AppState::GameOver))));
```

#### BattlePhase 门控

```rust
// 回合管理只在 RoundStart/TurnEnd 时运行
app.configure_sets(LogicSet, (
    TurnSet.run_if(in_state(BattlePhase::RoundStart)
        .or_else(in_state(BattlePhase::TurnEnd))),
));

// 战斗逻辑只在 PlayerPhase/EnemyPhase 时运行
app.configure_sets(LogicSet, (
    CombatSet.run_if(in_state(BattlePhase::PlayerPhase)
        .or_else(in_state(BattlePhase::EnemyPhase))),
));

// 效果管线只在 PlayerPhase/EnemyPhase 时运行
app.configure_sets(EffectSet, (
    EffectGenerateSet.run_if(in_state(BattlePhase::PlayerPhase)
        .or_else(in_state(BattlePhase::EnemyPhase))),
));
```

#### TurnPhase 门控

```rust
// 移动系统只在 MoveUnit 阶段运行
app.configure_sets(MovementSet, (
    MovementSet.run_if(in_state(TurnPhase::MoveUnit)),
));

// 目标选择只在 SelectTarget 阶段运行
app.configure_sets(CombatSet, (
    TargetSelectionSet.run_if(in_state(TurnPhase::SelectTarget)),
));
```

---

### 2.5 FixedTimestep vs 帧依赖

#### 物理无关系统（使用 FixedUpdate）

| 系统 | 原因 |
|------|------|
| 物理模拟 | 需要固定步长保证确定性 |
| 动画帧 | 需要稳定的时间步进 |
| 状态哈希计算 | 需要在固定时机执行 |

#### 帧依赖系统（使用 Update）

| 系统 | 原因 |
|------|------|
| 输入处理 | 必须每帧响应 |
| UI 渲染 | 必须每帧更新 |
| ViewModel 更新 | 必须反映最新状态 |

#### 当前决策

```
SRPG 是回合制游戏，战斗逻辑不需要固定步长。
所有战斗系统使用 Update Schedule，通过 SystemSet 控制顺序。
FixedUpdate 仅用于未来可能的物理/动画需求。
```

---

## 3. 不变量

### 不变量1：所有系统必须归属某个 Set

```
每个 System 必须通过 configure_sets 归属到某个 SystemSet。
禁止：系统无 Set 归属直接注册
```

### 不变量2：Set 间依赖必须显式声明

```
Set 之间的 after/before 依赖必须在 configure_sets 中声明。
禁止：依赖隐式执行顺序
```

### 不变量3：不能存在循环依赖

```
Set 依赖图必须是 DAG（有向无环图）。
禁止：A.after(B) 且 B.after(A)
```

### 不变量4：Effect Pipeline 三步必须严格顺序

```
EffectGenerate → EffectModify → EffectExecute 必须严格顺序执行。
禁止：并行执行管线三步
禁止：跳过任何一步
```

### 不变量5：LogicSet 必须在 EffectSet 之前

```
所有业务逻辑系统必须在效果管线系统之前执行。
禁止：EffectSet 在 LogicSet 之前
```

---

## 4. 禁止事项

| 禁止项 | 理由 | 违反后果 |
|--------|------|---------|
| 🟥 系统无 Set 归属直接注册 | 无法控制执行顺序 | 竞态条件 |
| 🟥 Set 间依赖不显式声明 | 顺序隐式依赖 | 不确定性 |
| 🟥 Set 循环依赖 | 调度器死锁 | 游戏卡死 |
| 🟥 Effect Pipeline 三步并行 | 严格顺序依赖 | 效果错误 |
| 🟥 在 PreUpdate 中执行游戏逻辑 | PreUpdate 专用于输入 | 职责混乱 |
| 🟥 在 PostUpdate 中修改游戏状态 | PostUpdate 专用于 UI | 状态不一致 |
| 🟥 系统依赖顺序但不声明 | Bevy 不保证隐式顺序 | 间歇性 Bug |
| 🟥 所有系统串行执行 | 浪费并行能力 | 性能低下 |

---

## 5. Schedule 注册规范

### 完整注册示例

```rust
impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // 1. 注册自定义 Schedule
        app.add_schedule(InputSchedule)
            .add_schedule(LogicSchedule)
            .add_schedule(PresentationSchedule);

        // 2. 配置顶层 Set 排序
        app.configure_sets(Update, (
            InputSet,
            CommandSet.after(InputSet),
            LogicSet.after(CommandSet),
            EffectSet.after(LogicSet),
            ViewModelSet.after(EffectSet),
            UISet.after(ViewModelSet),
        ).run_if(in_state(AppState::InGame)));

        // 3. 配置 LogicSet 内部排序
        app.configure_sets(LogicSet, (
            TurnSet,
            MovementSet.after(TurnSet),
            CombatSet.after(MovementSet),
            BuffSet.after(CombatSet),
            AISet.after(BuffSet),
        ));

        // 4. 配置 EffectSet 内部排序
        app.configure_sets(EffectSet, (
            EffectGenerateSet,
            EffectModifySet.after(EffectGenerateSet),
            EffectExecuteSet.after(EffectModifySet),
        ));

        // 5. 注册系统到对应 Set
        app.add_systems(LogicSet, (
            turn_system.in_set(TurnSet),
            movement_system.in_set(MovementSet),
            combat_system.in_set(CombatSet),
        ));

        app.add_systems(EffectSet, (
            effect_generate.in_set(EffectGenerateSet),
            effect_modify.in_set(EffectModifySet),
            effect_execute.in_set(EffectExecuteSet),
        ));
    }
}
```

---

## 6. 交叉引用

| 文档 | 关系 |
|------|------|
| `docs/architecture/app-bootstrap.md` | App 层启动装配、Plugin 注册顺序 |
| `docs/architecture/battle_fsm_design.md` | FSM 状态转换依赖 Schedule 执行 |
| `docs/architecture/determinism_rules.md` | 系统执行顺序是确定性的基础 |
| `docs/domain/turn_rules.md` | TurnPhase 内的系统顺序 |
| `docs/domain/battle_rules.md` | Effect Pipeline 的执行顺序 |
| `docs/architecture.md` | 效果管线三步顺序 |
