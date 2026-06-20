---
id: 01-architecture.40-cross-cutting.ADR-055
title: "ADR-055: UI 表现层架构"
status: proposed
owner: architect
created: 2026-06-19
tags:
  - architecture
  - ui
  - presentation
  - cross-cutting
  - bevy-0.19
---

# ADR-055: UI 表现层架构

## 状态

提议中

## 背景

当前项目 15 个 Capabilities + 15 个业务域 + 全部基础设施已实现，架构总纲（`docs/01-architecture/README.md`）定义了 DDD 纵向三层 + 横切四层的模块组织，宪法三层（Domain / Application / Presentation）也明确了运行时职责分离。然而：

1. **缺少 Presentation Layer 的具体规范**：宪法 §1.5(2) 定义了三层运行时分离，但 UI 层没有对应的模块定义和边界约束
2. **UI 与 Domain 耦合风险高**：ADR-040 明确"UI 只读"，但没有规定 UI 如何安全地读取 Domain 数据——直接 Query Domain Component 会导致紧耦合
3. **缺少 UI 通信规范**：ADR-043 定义了 GameCommand，但 UI 层的输入意图（点击技能按钮、选择目标）如何转换为 GameCommand 没有规范
4. **缺少 UI 与 Cue 的对接规范**：ADR-012 定义了 Cue 作为逻辑→表现的桥梁，但 UI 如何消费 Cue（伤害数字、状态提示）没有规范
5. **缺少 UI 与 GameState 的对接规范**：ADR-050 定义了 GameState/OverlayState，但 UI Screen 如何与 GameState 对应没有规范
6. **50 万行代码规模下**：没有架构约束的 UI 代码会迅速腐化，成为最大的技术债来源

## 引用的领域规则与架构

- `docs/01-architecture/README.md` §1.1 — 三条基石原则（Feature First / 三层架构 / Effect Pipeline）
- `docs/01-architecture/README.md` §2.1 — DDD 纵向三层
- `docs/01-architecture/README.md` §2.4 — 运行时三层（Domain / Application / Presentation）
- `docs/01-architecture/README.md` §4.2 — 四级通信机制
- `docs/01-architecture/README.md` §6 — Plugin 组合与注册顺序
- `ADR-040` — 数据流与所属权策略（UI 只读 Domain 数据）
- `ADR-043` — 命令层与输入抽象（GameCommand 统一入口）
- `ADR-050` — 游戏状态机与场景架构（GameState / OverlayState / SceneRoot）
- `ADR-053` — Localization 基础设施架构（LocalizedText Component / LocalizationKey）
- `ADR-054` — Bevy 0.19 迁移决策（BSN 范围 / Observer run_if / Delayed Commands）
- `ADR-012` — Stacking / Trigger / Cue 分离（Cue 是逻辑→表现的桥梁）
- `docs/02-domain/capabilities/cue_domain.md` — Cue 领域规则（CueType::Popup 对接 UI）
- `docs/00-governance/ai-constitution-complete.md` §1.5(7) — P0 Localization First

## 决策

### 1. UI 作为独立顶层模块

`src/ui/` 作为与 `core/`、`infra/`、`app/` 平级的顶层模块，对应 DDD 的 Presentation Layer。

纵向四层：

```
L0 Shared
L1 Core（Capabilities + Domains）
L2 Infra
L3 UI（Presentation Layer）
```

依赖方向：

```
UI → Core → Shared（允许）
UI → Infra/Input（通过接口，允许）
UI → Infra/Localization（通过 LocalizedText Component，允许）
Core → UI（禁止，铁律）
Infra → UI（禁止）
```

**架构变更说明**：本决策在现有 DDD 三层 + 横切四层模型中新增 L3 UI 层。这不改变现有纵向三层的依赖方向（Shared → Core → Infra），而是在 Infra 之上新增一个只读消费层。UI 层是纯消费方，不产生任何被 Core/Infra 依赖的类型。

**与横切层的关系**：
- UI 不属于横切四层（App/Content/Tools/Modding），而是纵向 L3 层
- App 层负责注册 UiPlugin（与注册其他 Plugin 一致）
- Content 层不涉及 UI（UI 没有 Definition 需要加载）
- Tools 层的 DebugOverlay 通过 UI 的 overlay/ 子模块实现
- Modding 层未来可扩展 UI Mod，但当前不在范围内

### 2. src/ui/ 目录结构

```
src/ui/
├── application/          # UI 应用层（Command Bus + Event Bus）
│   ├── command.rs        # UiCommand 枚举 + 分发
│   ├── event.rs          # UiEvent 枚举 + 广播
│   ├── intent.rs         # UiIntent（输入意图抽象）
│   └── mod.rs
├── projections/          # 投影层（Domain → ViewModel 的防火墙）
│   ├── battle.rs         # BattleProjection
│   ├── inventory.rs      # InventoryProjection
│   ├── character.rs      # CharacterProjection
│   ├── quest.rs          # QuestProjection
│   ├── economy.rs        # EconomyProjection
│   └── mod.rs
├── view_models/          # ViewModel 定义
│   ├── battle_hud.rs     # BattleHudVm
│   ├── character_panel.rs # CharacterPanelVm
│   ├── skill_panel.rs    # SkillPanelVm
│   ├── inventory.rs      # InventoryVm
│   ├── shop.rs           # ShopVm
│   ├── quest_log.rs      # QuestLogVm
│   ├── notification.rs   # NotificationVm
│   └── mod.rs
├── primitives/           # L3-P: UI 原语层（唯一允许操作 Bevy UI 底层类型的模块）
│   ├── button/           # PrimaryButton, SecondaryButton, DangerButton
│   ├── progress_bar/     # ProgressBar
│   ├── panel/            # Panel, CardPanel
│   ├── text/             # LocalizedText（统一文本包装，对接 ADR-053）
│   ├── list/             # VirtualList
│   └── modal/            # ModalService
├── widgets/              # L3-W: 游戏业务控件（组合原语，骨架阶段）
│   ├── tooltip/          # TooltipService（不属于任何原语）
│   ├── notification/     # NotificationService
│   └── mod.rs
├── screens/              # L3-S: 页面（组合 Widget，与 GameState 对应）
│   ├── battle/           # BattleScreen（对应 GameState::Combat）
│   ├── menu/             # MainMenuScreen（对应 GameState::MainMenu）
│   ├── inventory/        # InventoryScreen
│   ├── shop/             # ShopScreen（对应 OverlayState::Shop）
│   ├── settings/         # SettingsScreen
│   ├── save_load/        # SaveLoadScreen
│   └── mod.rs
├── overlay/              # 独立叠加层（不挂在 Screen 下，跨 Screen 共享）
│   ├── tooltip.rs        # TooltipOverlay
│   ├── damage_text.rs    # DamageTextOverlay（消费 CueType::Popup）
│   ├── notification.rs   # NotificationOverlay
│   ├── loading.rs        # LoadingOverlay
│   ├── debug.rs          # DebugOverlay（仅 dev feature）
│   └── mod.rs
├── navigation/           # 页面导航栈
│   ├── screen_stack.rs   # ScreenStack（push/pop/replace）
│   ├── screen_state.rs   # UiScreenState
│   └── mod.rs
├── theme/                # 主题/设计令牌
│   ├── colors.rs         # UiColors（Primary/Danger/PanelBg 等）
│   ├── spacing.rs        # UiSpacing
│   ├── typography.rs     # UiTypography（对接 FontSource 语义类别）
│   ├── theme.rs          # Theme 系统（Light/Dark/Pixel/HD2D）
│   └── mod.rs
├── localization/         # UI 文本国际化（对接 ADR-053）
│   ├── text_keys.rs      # UiTextKey 枚举
│   └── mod.rs
├── focus/                # 焦点系统（键盘/手柄导航）
│   ├── focusable.rs      # Focusable Component
│   ├── focus_group.rs    # FocusGroup
│   └── mod.rs
├── binding/              # 数据绑定（ViewModel → Widget）
│   ├── dirty_flag.rs     # Dirty<T> 机制
│   ├── vm_binding.rs     # VmBinding trait
│   └── mod.rs
├── tests/                # UI 测试
│   ├── unit/             # Widget 测试
│   ├── snapshot/         # UI 树结构快照测试
│   └── integration/      # Screen 集成测试
├── plugin.rs             # UiPlugin
└── mod.rs
```

**与现有目录结构的关系**：
- `src/ui/` 是新增顶层目录，与 `src/core/`、`src/infra/` 平级
- 不修改现有 `src/` 下任何目录
- `src/app/scenes/`（ADR-050 定义）负责 GameState 生命周期管理，UI Screen 由 `src/ui/screens/` 定义，两者通过 `OnEnter(GameState::X)` 关联

### 3. 单向数据流

```
Domain Event（DamageApplied, HealthChanged, TurnStarted...）
    ↓ Observer（UI 注册的 Observer，带 run_if 条件）
Projection（BattleProjection::project_damage）
    ↓ 纯函数映射
ViewModel（BattleHudVm.hp = 80）
    ↓ Dirty Flag
Widget（HpBarWidget::refresh）
```

反向：

```
User Input（Click/Key/Touch）
    ↓
UiIntent（SelectSkill, ConfirmAction）
    ↓
UiCommand（UiCommand::CastSkill(skill_id)）
    ↓
Domain（通过 Observer/Trigger 转换为 GameCommand）
```

**与 ADR-043 的对接**：
- UiIntent 是 UI 层的输入意图抽象，比 InputAction 更高层（InputAction 是硬件语义，UiIntent 是业务语义）
- UiCommand 通过 `UiCommand → GameCommand` 转换器进入 ADR-043 定义的 CommandQueue
- 转换器在 `src/ui/application/command.rs` 中实现，是 UI 层与 Command 层的唯一桥梁

**与 ADR-012/Cue 的对接**：
- `CueType::Popup`（伤害数字、治疗数字、状态提示）由 `src/ui/overlay/damage_text.rs` 消费
- UI Observer 监听 `CueTriggered` 事件，根据 `CueType` 路由到对应 Overlay
- Cue 是逻辑→UI 的单向通道，UI 不回写 Cue

### 4. Bevy 0.19 新特性采用

#### 4.1 BSN 全面使用

所有 UI 实体使用 `bsn!` 宏生成（与 ADR-054 DR-003 对齐，BSN 仅限 UI 层）：

```rust
fn primary_button(label: UiTextKey) -> impl Scene {
    bsn! {
        PrimaryButton
        Children [
            LocalizedText(label)
        ]
    }
}
```

#### 4.2 SceneComponent 预制体化

关键 Widget 使用 SceneComponent（与 ADR-054 "有条件采用"对齐，UI 层是 SceneComponent 的主要使用场景）：

```rust
#[derive(Component, SceneComponent)]
#[scene(battle_screen())]
struct BattleScreen;
```

#### 4.3 FontSize 枚举

```rust
FontSize::Px(14.0)   // 固定像素
FontSize::Rem(1.2)   // 响应式
```

#### 4.4 FontSource 语义类别

```rust
FontSource::Family("heading")  // 标题字体
FontSource::Family("body")     // 正文字体
FontSource::Family("mono")     // 等宽字体
```

语义类别在 `src/ui/theme/typography.rs` 中定义，与 `src/ui/widgets/text/` 的 LocalizedText 对接。

#### 4.5 Observer + run_if

UI Observer 使用 run_if 条件（与 ADR-054 DR-001 对齐）：

```rust
app.add_observer(on_damage_applied.run_if(screen_is_active::<BattleScreen>));
```

#### 4.6 Delayed Commands

UI 动画延迟使用 Delayed Commands（与 ADR-054 DR-002 对齐）：

```rust
commands.delayed().secs(0.3).entity(toast).despawn();
```

#### 4.7 User Settings

UI 偏好使用 bevy_settings：

```rust
#[derive(Resource, SettingsGroup, Reflect, Default)]
struct UiSettings {
    show_damage_numbers: bool,
    show_minimap: bool,
    battle_speed: f32,
}
```

### 5. UI 宪法级规则

#### 5.1 UI Query 禁止规则

禁止在 UI 模块中直接 Query Domain 组件：

```rust
// 禁止
fn update_hp_ui(query: Query<&Health>) { ... }

// 允许
fn update_hp_ui(vm: Res<BattleHudVm>) { ... }
```

**理由**：直接 Query Domain Component 导致 UI 与 Domain 紧耦合。50 万行代码下，Domain 重构会波及所有直接 Query 的 UI 代码。Projection 层是解耦的防火墙。

**唯一例外**：Projection 自身可以 Query Domain Component（这是 Projection 的职责），但 Projection 输出的 ViewModel 不包含任何 Domain 类型。

#### 5.2 Widget 不持有 Entity

```rust
// 禁止
struct SkillButton { skill_entity: Entity }

// 允许
struct SkillButton { skill_id: SkillId }
```

**理由**：Entity 是 ECS 运行时概念，Widget 持有 Entity 会导致 Widget 生命周期与 Entity 生命周期耦合。SkillId 是 Definition 层的强类型 ID，不受 Entity 回收影响。

#### 5.3 UI 动画不写业务逻辑

```rust
// 禁止：动画结束后发奖励
fn on_animation_end(trigger: Trigger<AnimationComplete>) {
    commands.trigger(GiveReward);
}

// 允许：奖励发放后播放动画
// Domain 先完成奖励，UI 再播放动画
```

**理由**：UI 是 Presentation Layer，不驱动业务流程。业务逻辑的执行时机由 Domain 决定，UI 只负责表现。违反此规则会导致"动画卡住则业务卡住"的耦合问题。

#### 5.4 Widget Contract

每个 Widget 必须声明输入输出：

```rust
/// SkillPanel Widget
/// Input: SkillPanelVm
/// Output: UiAction::SelectSkill, UiAction::CastSkill
/// 禁止: Query<...>, EventReader
```

**理由**：Widget Contract 使 AI 生成 UI 代码时有明确约束，也使 Widget 可独立测试。

#### 5.5 Screen 组合 Widget

Screen 只做组合，不直接拼 Node：

```rust
// 正确
fn battle_screen() -> impl Scene {
    bsn! {
        BattleScreen
        Children [
            TopBar,
            TurnBar,
            CharacterPanel,
            SkillPanel,
            ActionMenu,
        ]
    }
}
```

**理由**：Screen 是 Widget 的组合层，不应包含布局细节。布局细节属于 Widget 内部。

### 6. UI Root 分层

```
UiRoot
├── ScreenLayer     ← 当前 Screen（与 GameState 对应）
├── PopupLayer      ← Modal/Dialog（与 OverlayState 对应）
├── TooltipLayer    ← 独立 Tooltip
├── NotificationLayer ← Toast/Banner
└── DebugLayer      ← FPS/VM 状态（仅 dev feature）
```

每层独立，Screen 销毁不影响 Tooltip。

**与 ADR-050 的对接**：
- `ScreenLayer` 的内容由 `OnEnter(GameState::X)` 驱动 spawn
- `PopupLayer` 的内容由 `PushOverlay(OverlayState::X)` 驱动 spawn
- `OnExit(GameState::X)` 只清理 `ScreenLayer`，不影响其他层
- `PopOverlay` 只清理 `PopupLayer`

### 7. UI 状态分级

| 级别 | 类型 | 生命周期 | 存储位置 | 示例 |
|------|------|---------|---------|------|
| Level 1 | 持久状态 | 跨会话 | `UiSettings`（bevy_settings） | show_damage_numbers, battle_speed |
| Level 2 | 会话状态 | 单次游戏 | ECS Resource | InventoryFilter, SelectedTab |
| Level 3 | 瞬态状态 | 单次交互 | ECS Component | Hover, Drag, Tooltip |

分开管理，禁止一个巨型 `UiState` Resource。

**与 Save/Replay 的关系**：
- Level 1 状态通过 bevy_settings 持久化，不进入 Save 文件
- Level 2 状态如果影响业务（如 InventoryFilter 不影响），不进入 Save 文件
- Level 3 状态不持久化
- Replay 不录制 UI 状态（UI 从 Domain Event 重建）

### 8. Persistent Widget 模式

频繁开关的 Widget 不销毁，用 Visibility 切换：

```rust
// SkillPanel 永远存在，只是隐藏
fn toggle_skill_panel(mut query: Query<&mut Visibility, With<SkillPanel>>) {
    query.single_mut().toggle();
}
```

**适用场景**：
- 战斗 HUD（HP/MP 条、回合指示器）
- 技能面板
- 小地图

**不适用场景**：
- 全屏页面（MainMenu、Inventory）—— 使用 spawn/despawn
- 一次性弹窗（确认对话框）—— 使用 spawn + delayed despawn

### 9. 与 ADR-050 GameState 的 Screen 映射

| GameState | UI Screen | ScreenLayer 内容 |
|-----------|-----------|-----------------|
| MainMenu | MainMenuScreen | 标题画面、开始/继续/设置按钮 |
| PartySetup | PartySetupScreen | 队伍编成、角色选择 |
| TacticalMap | TacticalMapScreen | 地图、小地图、队伍信息 |
| Combat | BattleScreen | 战斗 HUD、技能面板、回合条 |
| Result | ResultScreen | 战斗结算、奖励展示 |
| CampRest | CampRestScreen | 营地界面、休息选项 |
| GameOver | GameOverScreen | 游戏结束画面 |

| OverlayState | UI Overlay | PopupLayer 内容 |
|-------------|-----------|----------------|
| Dialogue | DialogueOverlay | 对话框、选项 |
| Shop | ShopScreen | 商店界面 |
| Cutscene | CutsceneOverlay | 过场演出 |
| Tutorial | TutorialOverlay | 新手指引 |

### 10. Plugin 注册位置

```rust
// Phase 11: UI Presentation Layer (L3)
// 在 Infra (Phase 8) 和 ScenePlugin (Phase 9) 之后
// 确保 Localization 已初始化（ADR-053）、GameState 已注册（ADR-050）
.add_plugins(ui::UiPlugin)
```

UiPlugin 内部注册顺序：

```rust
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // 1. 主题与设计令牌（最先注册，其他 Widget 依赖）
        app.add_plugins(theme::ThemePlugin);

        // 2. 数据绑定基础设施
        app.add_plugins(binding::BindingPlugin);

        // 3. 焦点系统
        app.add_plugins(focus::FocusPlugin);

        // 4. 导航栈
        app.add_plugins(navigation::NavigationPlugin);

        // 5. UI 应用层（Command/Event/Intent）
        app.add_plugins(application::UiApplicationPlugin);

        // 6. 投影层（Domain → ViewModel）
        app.add_plugins(projections::ProjectionPlugin);

        // 7. ViewModel 注册
        app.add_plugins(view_models::ViewModelPlugin);

        // 8. Widget 注册（每个 Widget 独立 Plugin）
        app.add_plugins(widgets::WidgetsPlugin);

        // 9. Overlay 注册
        app.add_plugins(overlay::OverlayPlugin);

        // 10. Screen 注册（最后，依赖 Widget 和 ViewModel）
        app.add_plugins(screens::ScreensPlugin);
    }
}
```

## Module Design

### 新增模块

```
src/ui/                   # UI 表现层（L3）
├── mod.rs                # pub mod 声明
├── plugin.rs             # UiPlugin
├── application/          # UI 应用层
├── projections/          # 投影层
├── view_models/          # ViewModel
├── widgets/              # Widget 集合
├── screens/              # Screen 集合
├── overlay/              # 叠加层
├── navigation/           # 导航栈
├── theme/                # 主题系统
├── localization/         # UI 国际化
├── focus/                # 焦点系统
├── binding/              # 数据绑定
└── tests/                # UI 测试
```

### 修改文件

| 文件 | 变更 |
|------|------|
| `src/app/app_plugin.rs` | 新增 `UiPlugin` 注册（Phase 11） |
| `docs/01-architecture/README.md` | 更新架构总图（新增 L3 UI 层）、更新 ADR 索引 |

**不修改的文件**：所有 Core 层、Infra 层、Content 层代码。UI 是纯消费方，不产生任何被其他层依赖的类型。

## Communication Design

| 通信 | 机制 | 方向 | 说明 |
|------|------|------|------|
| Domain Event → Projection | Observer + run_if | Core → UI | UI 监听 Domain 事件，更新 ViewModel |
| ViewModel → Widget | Dirty Flag + Changed Filter | UI 内部 | Widget 检测 ViewModel 变化后刷新 |
| User Input → UiIntent | InputAction 映射 | Infra/Input → UI | 硬件输入转为业务意图 |
| UiIntent → UiCommand | Intent 解析 | UI 内部 | 业务意图转为 UI 命令 |
| UiCommand → GameCommand | 转换器 | UI → Core | UI 命令转为业务命令（唯一出口） |
| CueTriggered → Overlay | Observer | Core → UI | Cue 信号驱动 UI 表现 |
| GameState → Screen | OnEnter/OnExit | App → UI | 场景生命周期驱动 Screen spawn/despawn |
| OverlayState → Popup | PushOverlay/PopOverlay | App → UI | 覆盖层生命周期驱动 Popup spawn/despawn |
| LocalizationKey → Text | LocalizedText Component | Infra → UI | UI 读取 LocalizedText 自动渲染 |

## 边界定义

### 允许

- UI 通过 Projection 读取 Domain Component（Projection 是唯一合法的 Domain 数据读取点）
- UI 通过 UiCommand → GameCommand 转换器向 Domain 发送命令
- UI 通过 Observer 监听 Domain Event 和 Cue 信号
- UI 使用 Infra/Localization 的 LocalizedText Component
- UI 使用 Infra/Input 的 InputAction（经过 UiIntent 抽象）
- Screen 与 GameState 通过 OnEnter/OnExit 关联

### 禁止

- Core → UI 的任何依赖（铁律：Domain 不知道 UI 的存在）
- Infra → UI 的任何依赖
- UI 直接 Query Domain Component（必须经过 Projection）
- UI 直接修改 Domain Component（必须经过 UiCommand → GameCommand）
- Widget 持有 Entity 引用（使用强类型 ID）
- UI 动画驱动业务逻辑（业务先执行，UI 后表现）
- Screen 直接拼 Node（Screen 只组合 Widget）
- 巨型 UiState Resource（按生命周期分级管理）

## Forbidden

| 行为 | 理由 |
|------|------|
| UI 模块中直接 Query Domain Component | 违反 Projection 防火墙，导致 UI 与 Domain 紧耦合 |
| Core/Infra 依赖 UI 类型 | 违反依赖方向铁律（L3 → L2 → L1 → L0 单向） |
| Widget 持有 Entity 引用 | Entity 生命周期与 Widget 生命周期不同步，导致悬垂引用 |
| UI 动画回调触发业务逻辑 | 违反 Presentation Layer 纯消费原则，业务逻辑时机由 Domain 决定 |
| Screen 内直接拼 Node | Screen 是组合层，布局细节属于 Widget |
| 单一巨型 UiState Resource | 违反状态分级原则，导致状态管理混乱 |
| UI 直接调用 NextState | 违反 ADR-050 的 StateTransitionQueue 统一入口 |
| UI 绕过 GameCommand 直接调用 Domain 函数 | 违反 ADR-043 的命令统一入口，不可回放 |
| UI 存储翻译后的文本字符串 | 违反 ADR-053 的 LocalizationKey 原则，只存 Key + 参数 |
| Widget 内使用 EventReader/EventWriter | 违反 ADR-054 的 Observer 优先原则 |

## Definition / Instance Design

- **Definition（不可变配置）**：
  - `UiTheme`（Light/Dark/Pixel/HD2D 主题配置）
  - `UiColors`、`UiSpacing`、`UiTypography`（设计令牌）
  - `UiTextKey`（UI 文本 Key 枚举，编译期生成）

- **Instance（运行时状态）**：
  - `BattleHudVm`、`CharacterPanelVm` 等 ViewModel（ECS Resource）
  - `Dirty<T>` 绑定标记（ECS Component）
  - `Focusable`、`FocusGroup`（ECS Component）
  - `ScreenStack`（ECS Resource）
  - `UiSettings`（bevy_settings Resource）

### 9. UI Schema 治理

为 AI 编码合规建立机器可读的 Widget Schema，存放在 `docs/ui_schema/`：

```yaml
# docs/ui_schema/skill_panel.yaml
SkillPanel:
  input:
    type: SkillPanelVm
    fields: [skills, selected, ap_remaining]
  output:
    - UiAction::SelectSkill(SkillId)
    - UiAction::CastSkill(SkillId)
  children:
    - SkillButton
    - SkillTooltip
  prohibited:
    - Query<&Ability>
    - Query<&Health>
    - EventReader
```

AI 生成 UI 代码时必须遵守 Schema，否则 50 万行后 UI 会退化为不可维护的 Node 堆。

Schema 文件清单：
- `docs/ui_schema/screens/` — 每个 Screen 一个 Schema
- `docs/ui_schema/widgets/` — 每个 Widget 一个 Schema
- `docs/ui_schema/view_models/` — 每个 ViewModel 的字段定义
- `docs/ui_schema/contracts/` — Widget Contract 声明

### DR-XXX: Primitives 隔离层

**背景**：50 万行级别项目中，UI 框架替换成本极高。Bevy UI 底层 API（Node、Button、Interaction）可能随版本升级而变化。

**决策**：引入 Primitives 隔离层作为 UI 架构的最底层。

**Primitives 层确保**：
- 底层 UI 实现变更时只影响 `primitives/`
- 业务控件层（`widgets/`）不感知底层实现细节
- 新增 Widget 自动获得隔离保护

**约束**：
1. `primitives/` 是唯一允许直接操作 `Node`/`Button`/`Interaction`/`BackgroundColor` 等 Bevy UI 原语的模块
2. `widgets/` 和 `screens/` 只能通过 Primitives Factory 函数和组件使用底层 UI 能力
3. `primitives/` 依赖 `theme/`，不依赖任何业务模块
4. `.claude/rules/` 中增加规则禁止在 primitives 外使用 Bevy UI 原语类型

**依赖方向**：
```
primitives/ → theme/     （允许）
widgets/   → primitives/ （允许，禁止直接访问 Bevy UI 类型）
screens/   → widgets/ + primitives/ （允许，通过 Factory）
```

### 10. UI 三层测试体系

| 测试层 | 测试对象 | 验证内容 | 位置 |
|--------|---------|---------|------|
| Widget Test | 单个 Widget | 渲染正确性、交互响应、Contract 合规 | `src/ui/widgets/*/tests/` |
| Screen Test | Screen 组合 | Widget 组合正确、导航正常、ViewModel 绑定 | `src/ui/screens/*/tests/` |
| Snapshot Test | UI 树结构 | Entity 层级/Component 结构未意外变更 | `src/ui/tests/snapshot/` |

Widget Test 示例：
```rust
#[test]
fn skill_panel_displays_skills() {
    let vm = SkillPanelVm { skills: test_skills(), ..default() };
    let world = setup_ui_world(vm);
    // 验证：每个 SkillSlotVm 对应一个 SkillButton 实体
}

#[test]
fn skill_panel_contract_no_domain_query() {
    // 验证：SkillPanel 代码中无 Query<&Ability>
}
```

Screen Test 示例：
```rust
#[test]
fn battle_screen_combines_all_widgets() {
    let world = setup_battle_screen();
    // 验证：BattleScreen 包含 TopBar + TurnBar + CharacterPanel + SkillPanel
}
```

Snapshot Test 示例：
```rust
#[test]
fn battle_screen_tree_snapshot() {
    let tree = capture_ui_tree::<BattleScreen>();
    insta::assert_yaml_snapshot!("battle_screen_tree", tree);
}
```

### 11. UiBinding 反 Marker 模式

禁止为每个 UI 元素创建独立 Marker 结构体：
```rust
// ❌ 禁止：50 万行项目最终 400+ Marker
struct HpText;
struct ManaText;
struct ExpText;
struct GoldText;

// ✅ 允许：统一枚举
#[derive(Component, Reflect)]
pub enum UiBinding {
    Hp,
    Mana,
    Exp,
    Gold,
    Level,
    Turn,
    Ap,
}

// ✅ 或统一 ID
#[derive(Component, Reflect)]
pub struct UiElementId(pub u32);
```

原因：400+ Marker 结构体导致：
- 查询效率低（每个 Marker 一个 Archetype）
- AI 生成代码时不知道该用哪个 Marker
- 重构时需要逐个修改

### 12. WidgetFactory trait

所有 Widget 实现统一的工厂 trait，Screen 通过工厂组合 Widget：

```rust
/// Widget 工厂 trait
/// 每个 Widget 实现此 trait，Screen 通过它组合 Widget
pub trait WidgetFactory: Component {
    type Vm: Reflect + Default;

    /// 从 ViewModel 创建 Widget
    fn create(commands: &mut Commands, vm: &Self::Vm) -> Entity;

    /// 刷新 Widget（仅在 Dirty 时调用）
    fn refresh(entity: Entity, vm: &Self::Vm, query: &mut Query<&mut Self>);

    /// 销毁 Widget
    fn destroy(commands: &mut Commands, entity: Entity);
}
```

Screen 使用 WidgetFactory 组合：
```rust
fn battle_screen() -> impl Scene {
    bsn! {
        BattleScreen
        Children [
            TopBar::create_root(),
            TurnBar::create_root(),
            CharacterPanel::create_root(),
            SkillPanel::create_root(),
            ActionMenu::create_root(),
        ]
    }
}
```

### 13. UI 与 Content/Modding 数据流

UI 不直接加载 Content 数据，通过 Projection 防火墙：

```
Content (assets/config/*.ron)
    ↓ 加载
DefRegistry (Resource)
    ↓ Projection 查询
ViewModel (UiStore)
    ↓ Dirty Flag
Widget
```

禁止 UI 直接访问 DefRegistry 中的 Definition：
```rust
// ❌ 禁止：UI 直接读取 SpellDef
fn update_skill_icon(defs: Res<DefRegistry<SpellDef>>) { ... }

// ✅ 允许：Projection 读取 Def，投影到 ViewModel
fn project_skill_info(
    trigger: Trigger<AbilityUsed>,
    defs: Res<DefRegistry<SpellDef>>,
    mut store: ResMut<UiStore>,
) {
    if let Some(def) = defs.get(trigger.ability_id) {
        store.skill_panel.update_from_def(def);
    }
}
```

Modding 数据流：
```
Mod
    ↓ 注册
Content
    ↓ 加载
DefRegistry
    ↓ Projection
ViewModel
    ↓
Widget
```

禁止 Mod 直接扩展 UI，必须通过 Content → DefRegistry → Projection 路径。

### 14. 四条铁律（精简版）

从 8 条不变量中提炼的 4 条最核心约束，必须写进 AI 宪法：

1. **Domain 不依赖 UI** — Core/Infra 层禁止 import ui/ 中的任何类型
2. **UI 不直接读 Domain** — 通过 ViewModel，禁止 Query Domain Component
3. **Screen 组合 Widget** — Screen 不直接拼 Node，只做 Widget 组合
4. **所有颜色字体间距统一 Token 化** — 禁止 Color::srgb() 直接写在 Widget 中

违反任何一条，50 万行后 UI 必然成为最大技术债。

## 后果

### 正面

1. **UI 与 Domain 完全解耦**：Projection 防火墙确保 Domain 重构不影响 Widget
2. **单向数据流**：状态变更可追踪，调试友好
3. **Widget 可复用**：独立 Plugin + 明确 Contract，Screen 可自由组合
4. **AI 生成 UI 代码成功率高**：Widget Contract 提供明确约束
5. **测试友好**：ViewModel 可独立测试（纯数据），Projection 可单元测试（纯函数）
6. **与现有架构无缝对接**：GameState/OverlayState、GameCommand、Cue、Localization 均有明确对接点

### 负面

1. **Projection 层增加代码量**：每个 Domain 数据的 UI 展示都需要一个 Projection 映射
2. **ViewModel 需要手动维护**：Domain 新增字段时需同步更新 ViewModel 和 Projection
3. **初期开发速度可能较慢**：严格的分层约束增加前期工作量
4. **学习成本**：团队需要理解 Projection/ViewModel/Widget 三层分离

### 缓解

1. Projection 可用 derive 宏减少样板代码（后续优化）
2. AI 可自动生成 Projection 和 ViewModel（有明确的 Contract 约束）
3. 长期收益远大于短期成本（50 万行代码下，耦合的 UI 是最大的技术债来源）
4. Widget Contract 文档化后，新成员可快速上手

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| UI 直接 Query Domain Component | 50 万行代码下，UI 与 Domain 紧耦合会导致重构灾难 |
| 传统 MVVM（双向绑定） | 双向绑定导致状态变更不可追踪，调试困难 |
| UI 作为 Infra 层的一部分 | UI 不是技术基础设施，是 Presentation Layer，语义不同 |
| UI 作为 App 层的一部分 | App 是 Composition Root，不含业务逻辑；UI 有自己的领域（布局/主题/焦点） |
| 全局 UiState Resource | 单一巨型状态违反关注点分离，状态管理混乱 |
| ECS UI 无分层（直接在 Domain Plugin 中写 UI） | 违反宪法三层分离，Domain 代码与 UI 代码纠缠不清 |

## 架构合规性自检

- [x] 符合 Feature First 原则（UI 是独立 Feature 模块，非全局技术目录）
- [x] 符合三层运行时分离（Domain / Application / Presentation）
- [x] 符合 DDD 三层+横切四层层间依赖方向（UI → Core → Shared 单向，禁止反向）
- [x] Effect Pipeline 没有被绕过（UI 不直接修改战斗数值）
- [x] Modifier Pipeline 没有被绕过（UI 不直接修改属性）
- [x] 定义了明确的 Forbidden 事项（10 条禁止项）
- [x] 引用了上游领域规则和数据 Schema（ADR-040/043/050/053/054/012）
- [x] Plugin 注册顺序符合层次要求（Phase 11，在 Infra 和 Scene 之后）
- [x] 通信机制选择符合四级通信规范（Observer + run_if 监听 Domain Event）
- [x] 符合 Bevy 0.19 规范：使用 trigger() + Observer（非 EventWriter/EventReader）
- [x] 符合 Bevy 0.19 规范：BSN 仅限 UI 层（与 ADR-054 DR-003 对齐）
- [x] 符合 Definition/Instance 分离原则（UiTheme/UiTextKey 为 Definition，ViewModel 为 Instance）
- [x] 符合 Rules/Content 分离原则（UI 主题配置为 Content，UI 行为规则为 Code）
