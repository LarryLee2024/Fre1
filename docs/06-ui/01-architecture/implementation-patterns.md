---
id: 06-ui.implementation-patterns
title: Implementation Patterns — Widget/Screen/ViewModel/Overlay 的 Bevy ECS 骨架
status: draft
owner: presentation-architect
created: 2026-06-20
tags:
  - ui
  - implementation
  - patterns
  - bevy
  - ecs
  - widget
  - screen
  - viewmodel
  - overlay
  - plugin
  - bsn
---

# Implementation Patterns — Widget/Screen/ViewModel/Overlay 的 Bevy ECS 骨架

> **职责**: @presentation-architect | **上游**: widget-atoms.md（21 个原子组件契约）, widget-composites.md（16 个复合组件）, screens.md（6 个 Screen 组合树）, focus-binding.md（Dirty<T> + UiBinding）, application-layer.md（Intent/UiAction/UiCommand）, architecture.md §2.5（Bevy 0.19 BSN 约束）, ADR-055（整体架构）, ADR-054 DR-003（BSN 决策）
>
> **前置阅读**: 读本文前请先阅读以上上游文档——本文不重复"建什么"，只说明"怎么做"。

> **SSPEC参考**: docs/06-ui/07-specs/ — AI-Consumable Screen Specification 标准。新增 Screen 必须先写 SSPEC，见 ADR-066。

---

## 0. 本文定位

现有 `docs/06-ui/` 各文件回答了"**建什么**"（What）：
- widget-atoms.md：定义 21 个原子组件的输入/输出/状态/变体
- widget-composites.md：定义 16 个复合组件的组成树和组合规则
- screens.md：定义 6 个 Screen 的 Widget 组合树和 ViewModel 消费
- focus-binding.md：定义 Dirty<T> 和 UiBinding 机制

本文回答"**怎么做**"（How）——每个 Widget/Screen/ViewModel/Overlay 在 Bevy ECS 中对应哪些组件、System、Resource、事件，以及它们如何组合成完整的 UI 子系统。

**目标读者**：feature-developer 在看到 widget-atoms.md 中 "ProgressBar 有 current/max Props" 后，知道需要创建哪些 ECS 注册项、System、以及数据流链路。

---

## 1. Widget Plugin 模式

### 1.1 模式概述

每个 Widget（= 1 个 Contract 定义）在 Bevy ECS 中对应 1 个 Plugin。这个 Plugin 注册所有与该 Widget 相关的 ECS 类型：Props 组件、State 组件、UiAction 事件、spawn/update/cleanup System。

```
┌─────────────────────────────────────────────────┐
│             Widget Plugin (1:1 with Contract)   │
│                                                 │
│  ▸ register_props_component()                   │
│  ▸ register_state_component()                   │
│  ▸ register_ui_action_event()                   │
│  ▸ add_systems(spawn_on_command)                │
│  ▸ add_systems(on_dirty)                        │
│  ▸ add_systems(on_interaction)                  │
│  ▸ add_systems(cleanup_on_despawn)              │
└─────────────────────────────────────────────────┘
```

### 1.2 Plugin 骨架（伪代码结构）

```
plugin ProgressBarPlugin:

    fn build(app):
        // ── 1. 注册 Props Component ──
        // 对应 widget-atoms.md §3.1 ProgressBar 的 Props 定义
        // Props 是数据输入，Screen 在 spawn Widget 时设置
        app.register_component<ProgressBarProps>()

        // ── 2. 注册 State Component ──
        // 对应 Widget 的本地交互状态
        // Widget Plugin 注册 State 但外部不可见
        app.register_component<ProgressBarState>()

        // ── 3. 注册 UiAction Event ──
        // 对应 widget-atoms.md 中 Widget 的 Events 输出
        // ProgressBar 是纯展示组件，无需 Events
        // 但如果是有交互的 Widget（如 Button），则需要注册

        // ── 4. 注册 spawn_on_command System ──
        // Command 触发：UiCommand::OpenScreen → Screen Plugin → WidgetFactory::create
        // 或者更直接：WidgetFactory 在 Screen 的 OnEnter 事件中调用
        app.add_systems(OnEnter(GameState::Combat), spawn_battle_hud_progress_bars)

        // ── 5. 注册 on_dirty System ──
        // 数据驱动更新：ViewModel 变更 → Dirty<T> → System 检测 → UI Node 刷新
        // 每帧运行，但只在 Dirty<T> 被标记时才消费
        app.add_systems(Update, refresh_progress_bar.run_if(dirty_detected::<BattleHudVm>))

        // ── 6. 注册 on_interaction System ──
        // 用户交互 → UiAction → 冒泡到 Screen
        // 仅交互性 Widget（Button/SelectList/Toggle 等）需要
        // ProgressBar 是纯展示，跳过此步

        // ── 7. 注册 cleanup_on_despawn System ──
        // 当 Widget 实体被 despawn 时清理自身状态
        // 通常使用 OnRemove 钩子或 Observer
        app.add_observer(cleanup_progress_bar)
```

### 1.3 Props Component（示例：ProgressBar）

```
// ProgressBarProps — 对应 widget-atoms.md §3.1 ProgressBar 的 Props 字段
// Screen 在创建 Widget 时设置此组件值
component ProgressBarProps:
    current: f32          // 当前值
    max: f32              // 最大值（约束：max > 0.0）
    show_label: bool      // 是否显示 "current/max" 文本
    height: Val           // 进度条高度（默认 UiSpacing::md）
    animate: bool         // 是否播放过渡动画
```

Props 组件的关键约束：
- 字段类型不引用任何 Domain 类型（禁止 Health、Ability 等）
- 文本字段使用 UiTextKey 而非 String
- 颜色字段使用 UiColors 语义枚举而非 Color::srgb()

### 1.4 State Component（示例：ProgressBar）

```
// ProgressBarState — Widget 内部的交互状态
// 只对 Widget 自身可见，Screen 不读不写
component ProgressBarState:
    // 只有有交互的 Widget 才需要 State
    // ProgressBar 是纯展示组件，State 为空
    // 作为对比，Button 的 State 包含：
    //   hovered: bool    — 鼠标悬停
    //   pressed: bool    — 鼠标按下
```

State 组件的设计规则：
- 只存放 Widget 交互相关的瞬态数据
- 不存放从 Props 派生可计算的数据（避免冗余）
- 不持久化（被 TabPanel 切换或 navigate 导航不应保持深层的 hover 状态 —— 除 Focus 外）

### 1.5 System 伪签名

```
// ─── spawn_on_command System ───
// 触发时刻：Screen Plugin 的 OnEnter 事件中调用 WidgetFactory
// 职责：创建 Widget 的 ECS 实体树

fn spawn_progress_bar(commands: &mut Commands, props: &ProgressBarProps):
    // 1. 创建根 Node
    // 2. 添加 ProgressBarProps 组件（保存原始数据供后续刷新）
    // 3. 添加 UiBinding::Hp 或 UiBinding::Mp（由调用方指定）
    // 4. 添加背景 Node（进度条槽）
    // 5. 添加填充 Node（进度条填充，宽度 = current/max %）
    // 6. 添加 LocalizedText Node（如果 show_label）
    // 返回根 Entity


// ─── on_dirty System ───
// 触发时刻：每帧 Update 调度，但只处理被标记的 Dirty<T>
// 职责：读取 ViewModel → 更新 UI Node 属性

fn refresh_progress_bar(
    dirty_query: Query<&Dirty<BattleHudVm>>,          // 脏标记检测
    vm_store: Res<UiStore>,                           // ViewModel 数据源
    binding_query: Query<(&UiBinding, &mut Style)>,    // 目标 UI Node
):
    for dirty in dirty_query:
        if dirty.consume():                           // 只在 dirty = true 时执行
            for (binding, mut style) in binding_query:
                match binding:
                    UiBinding::Hp:
                        ratio = vm_store.battle_hud.hp / vm_store.battle_hud.max_hp
                        style.width = Val::Percent(clamp(ratio) * 100)
                    UiBinding::Mp:
                        ratio = vm_store.battle_hud.mp / vm_store.battle_hud.max_mp
                        style.width = Val::Percent(clamp(ratio) * 100)


// ─── on_interaction System（Button 示例）───
// 触发时刻：每帧 Update 调度
// 职责：检测交互事件 → 更新本地 State → 发射 UiAction

fn handle_button_interaction(
    interaction_query: Query<(&Interaction, &mut ButtonState), (Changed<Interaction>, With<PrimaryButtonProps>)>,
    mut action_writer: EventWriter<UiAction>,
):
    for (interaction, mut state) in interaction_query:
        match interaction:
            Interaction::Pressed:
                state.pressed = true
            Interaction::Hovered:
                state.hovered = true
            Interaction::None:
                if state.pressed:                      // 释放时触发 Click
                    action_writer.send(UiAction::Click)
                state.pressed = false
                state.hovered = false


// ─── cleanup_on_despawn System ───
// 触发时刻：Widget 实体被 despawn 时
// 职责：清理资源（动画 Timer、注册的 Event 等）

fn cleanup_progress_bar(trigger: Trigger<OnRemove, ProgressBarProps>):
    // ProgressBar 无特殊资源需要清理
    // 但实际 Widget 可能需清理：Timer、Asset Handle、注册的回调
    // （通常 Bevy 的 entity despawn 会自动回收 Component，无需手动清理）
```

### 1.6 事件冒泡路径

```
Widget 交互 → UiAction → Screen Handler → UiCommand → GameCommand

完整路径（以 Button 为例）：

1. 用户点击 UI 中的 PrimaryButton
2. Button 的 on_interaction System 检测到 Interaction::Pressed → None
3. System 发射 UiAction::Click 到 EventWriter<UiAction>
4. UiAction 事件通过 Parent 关系冒泡到 Screen Entity
5. Screen Plugin 注册的 on_ui_action System 接收 UiAction
6. System 根据当前 Screen 上下文，将 UiAction 转换为 UiCommand
   ── BattleScreen 上下文中：UiAction::Click → UiCommand::CastSkill(skill_id)
   ── InventoryScreen 上下文中：UiAction::Click → UiCommand::UseItem(item_id)
7. UiCommand 通过 application-layer.rs 的 command.rs 转换为 GameCommand
8. GameCommand 进入 ADR-043 CommandQueue
```

### 1.7 数据驱动路径

```
Projection → ViewModel → Dirty<T> → on_dirty System → UI Node Update

完整路径（以 HP 条为例）：

1. Domain Event: DamageApplied { target: CharId(7), value: 15 } 触发
2. Observer（UI 注册的 Observer）捕获事件
3. Observer 调用 BattleProjection::project_damage(store, triggered_event)
4. Projection 纯函数：store.battle_hud.hp = previous_hp - 15
5. Projection 设置：store.battle_hud.mark_dirty()
6. 下一帧 Update：
   a. refresh_progress_bar System 检测到 Dirty<BattleHudVm> 被标记
   b. consume() 返回 true，is_dirty 重置为 false
   c. System 读取 UiStore.battle_hud.hp / max_hp 计算比例
   d. System 通过 UiBinding::Hp 找到对应的 Style 组件
   e. System 更新 style.width = Val::Percent(ratio * 100)
   f. Bevy UI 自动重绘
7. 如果没有更多 Dirty 标记，System 下次 consume() 返回 false → 休眠
```

### 1.8 Widget Plugin 清单

以下列出每个原子 Widget 是否需要注册 Props/State/Events：

| Widget | Props | State | UiAction | 纯展示 | 参考 Contract |
|--------|-------|-------|----------|--------|--------------|
| PrimaryButton | 是 | hovered, pressed | Click | 否 | widget-atoms §2.1 |
| SecondaryButton | 是 | hovered, pressed | Click | 否 | widget-atoms §2.2 |
| DangerButton | 是 | hovered, pressed | Click | 否 | widget-atoms §2.3 |
| IconButton | 是 | hovered, pressed | Click | 否 | widget-atoms §2.4 |
| ProgressBar | 是 | — | — | 是 | widget-atoms §3.1 |
| MultiSegmentBar | 是 | — | — | 是 | widget-atoms §3.2 |
| Panel | 是 | — | — | 是 | widget-atoms §4.1 |
| ScrollPanel | 是 | scroll_offset, content_height, visible_height | — | 是（有内部交互无业务输出） | widget-atoms §4.2 |
| TabPanel | 是 | active_tab | ChangeTab | 否 | widget-atoms §4.3 |
| VirtualList | 是 | scroll_offset, first_visible, last_visible | — | 是 | widget-atoms §5.1 |
| SelectList | 是 | selected, hovered | SelectItem, ToggleItem, ConfirmSelection | 否 | widget-atoms §5.2 |
| ItemGrid | 是 | hovered_slot, drag_source | SelectItem, UseItem, ShowContextMenu | 否 | widget-atoms §5.3 |
| LocalizedText | 是 | — | — | 是 | widget-atoms §6.1 |
| FormattedText | 是 | — | — | 是 | widget-atoms §6.2 |
| Tooltip | 是 | — | — | 是（由 Service 管理生命周期） | widget-atoms §7.1 |
| Modal | 是 | — | Confirm, Cancel | 否 | widget-atoms §8.1 |
| Notification | 是 | elapsed, dismissed | Dismiss | 否 | widget-atoms §9.1 |
| TextInput | 是 | value, cursor_position, is_focused | TextChanged, TextConfirmed | 否 | widget-atoms §10.1 |
| Toggle | 是 | hovered | Toggle | 否 | widget-atoms §10.2 |
| TurnBar | 是 | — | — | 是 | widget-atoms §11.1 |
| StatusIcon | 是 | hovered | — | 是 | widget-atoms §11.2 |

---

## 2. Screen Plugin 模式

### 2.1 模式概述

每个 Screen（= 1 个 GameState/OverlayState 映射）在 Bevy ECS 中对应 1 个 Plugin。Screen Plugin 注册 Screen 生命周期钩子、Widget 创建/销毁系统、以及 UiAction → UiCommand 转换系统。

```
┌─────────────────────────────────────────────────┐
│             Screen Plugin (1:1 with Screen)      │
│                                                 │
│  ▸ register_screen_component()                  │
│  ▸ register_screen_resource()                   │
│  ▸ add_systems(OnEnter → spawn_screen)          │
│  ▸ add_systems(OnExit → despawn_screen)         │
│  ▸ add_systems(on_command_handling)             │
│  └───────────────────────────────────────────── │
│  ▸ (可选) register_overlay_dependencies()        │
└─────────────────────────────────────────────────┘
```

### 2.2 Plugin 骨架（伪代码结构）

```
plugin BattleScreenPlugin:

    fn build(app):
        // ── 1. 注册 Screen Component ──
        // 标记当前 Screen 实体，用于 Query 筛选
        app.register_component<BattleScreen>()

        // ── 2. 注册 Screen Resource ──
        // Screen 专属的会话级状态（Level 2 — 非持久化）
        // 存的不是 ViewModel（ViewModel 在 UiStore 中统一管理）
        // 而是 Screen 自己的导航/选择/临时状态
        app.init_resource<BattleScreenState>()

        // ── 3. 注册 OnEnter System ──
        // GameState → Combat 时触发
        // 职责：创建 Widget 实体树
        app.add_systems(OnEnter(GameState::Combat), spawn_battle_screen)

        // ── 4. 注册 OnExit System ──
        // GameState → Combat 退出时触发
        // 职责：回收全部 Widget 实体
        app.add_systems(OnExit(GameState::Combat), despawn_battle_screen)

        // ── 5. 注册 on_command System ──
        // UiAction → UiCommand 转换
        app.add_systems(Update, handle_battle_screen_commands)

        // ── 6. (可选) 注册 Overlay 依赖 ──
        // Screen 需要的 Overlay 类型声明（由 OverlayService 管理）
```

### 2.3 Screen Resource 示例

```
// BattleScreenState — 战斗屏幕的会话状态（Level 2）
// 用于存储屏幕上用户操作相关的临时状态
// 不属于 ViewModel（ViewModel 由 Domain Event 投影而来）
// ViewModel 和 ScreenState 的区别：
//   ViewModel: Domain 数据的 UI 投影（HP/MP/技能冷却）
//   ScreenState: UI 内部的会话状态（选中的技能、选中的目标、当前模式）

resource BattleScreenState:
    selected_skill: Option<SkillId>        // 当前选中的技能（UI 内部选择，非 Domain 数据）
    selected_target: Option<CharacterId>   // 当前选中的目标
    battle_mode: BattleMode                // Normal / Boss / Auto
    show_action_menu: bool                 // 是否显示操作菜单
```

### 2.4 OnEnter：spawn_screen System

> 🟩 **Bevy 0.19 BSN 策略**（宪法第九编 + architecture.md §2.5）：
> - `src/app/scenes/` ✅ 允许 BSN（Composition Root，一次性装配）
> - `src/ui/screens/` 🟥 禁止 BSN（Screen 必须通过 Factory 构建）
> - `src/ui/widgets/` 🟥 禁止 BSN（Widget 必须通过 Factory 构建）
> - Factory 是 UI 的唯一构建入口，输入仅限 Props/ViewModel/Theme
> - 详见 `architecture.md §2.5`

```
// 触发条件：OnEnter(GameState::Combat)
// 职责：创建 BattleScreen 的完整 Widget 实体树

fn spawn_battle_screen(commands: &mut Commands, vm_store: Res<UiStore>):

    // 创建 Screen 根实体
    screen_root = commands.spawn((
        BattleScreen,                      // Screen 标记组件
        Node { /* 全屏布局 */ },
        // ...
    ))

    // 通过 WidgetFactory 创建组合树
    // 对应 screens.md §2.3 定义的 BattleScreen 组合结构
    //
    // BattleScreen
    // ├── TurnOrderBar [Organism]
    // ├── BattleHud [Organism]
    │   │   ├── CharacterStatusPanel × N [Organism]
    │   │   ├── SkillPanel [Organism]
    │   │   └── TopBar (Panel + LocalizedText 组合)
    // ├── EnemyStatusBar (Panel)
    // ├── BattleFieldArea (Panel)
    // └── ActionMenu (Panel)

    turn_order_bar = TurnOrderBar::create(commands, &vm_store.turn_order)
    battle_hud = BattleHud::create(commands, &vm_store.battle_hud)
    enemy_bar = create_enemy_status_bar(commands, &vm_store.enemy_characters)
    field_area = create_battle_field(commands)
    action_menu = create_action_menu(commands, BattleScreenState::default.mode)

    // 所有 Organism 挂在 Screen Root 下（通过 Parent 关系）
    commands.entity(screen_root).add_children(&[
        turn_order_bar,
        battle_hud,
        enemy_bar,
        field_area,
        action_menu,
    ])
```

### 2.5 Widget 实体树结构

```
Entity: BattleScreen Root
  Component: BattleScreen (标记)
  Component: Node { width: 100%, height: 100% }
  │
  ├── Entity: TurnOrderBar [Organism]
  │   ├── Component: TurnOrderBar
  │   ├── Component: UiBinding::TurnBar
  │   │
  │   ├── Entity: TurnIndicator #0 [Molecule]
  │   │   ├── Component: TurnIndicator
  │   │   ├── Component: UiBinding::TurnSlot(0)
  │   │   ├── Child: IconButton (Avatar)
  │   │   ├── Child: Panel (Highlight)
  │   │   └── Child: CaptionText (AP)
  │   │
  │   ├── Entity: TurnIndicator #1 [Molecule]
  │   │   └── ...
  │   │
  │   └── Entity: TurnIndicator #N [Molecule]
  │
  ├── Entity: BattleHud [Organism]
  │   ├── Component: BattleHud
  │   │
  │   ├── Entity: CharacterStatusPanel #0 [Organism]
  │   │   ├── Component: CharacterStatusPanel
  │   │   ├── Component: Dirty<CharacterStatusPanelVm>
  │   │   │
  │   │   ├── Entity: CharacterPortrait [Molecule]
  │   │   │   ├── Component: CharacterPortrait
  │   │   │   ├── Child: Panel (bg)
  │   │   │   ├── Child: Image (avatar)
  │   │   │   ├── Child: BodyText (name)
  │   │   │   ├── Child: HpBar [ProgressBar]  ── UiBinding::Hp
  │   │   │   └── Child: Panel (status icons)
  │   │   │
  │   │   ├── Entity: MpBar [ProgressBar]      ── UiBinding::Mp
  │   │   ├── Entity: ApBar [ProgressBar]      ── UiBinding::Ap
  │   │   ├── Entity: BuffIcon #0 [Molecule]   ── UiBinding::BuffSlot(0)
  │   │   └── Entity: StatusText [BodyText]    ── UiBinding::Status
  │   │
  │   ├── Entity: SkillPanel [Organism]
  │   │   ├── Component: SkillPanel
  │   │   ├── Component: Dirty<SkillPanelVm>
  │   │   │
  │   │   ├── Entity: TabPanel [Atom]
  │   │   ├── Entity: ScrollPanel [Atom]
  │   │   └── Entity: SkillSlot #0 [Molecule]  ── UiBinding::SkillSlot(0)
  │   │       └── ...
  │   │
  │   └── Entity: TopBar
  │       ├── Entity: TurnCounter [LocalizedText]  ── UiBinding::Turn
  │       ├── Entity: PhaseIndicator [LocalizedText]
  │       └── Entity: EndTurnButton [SecondaryButton]
  │
  ├── Entity: EnemyStatusBar
  │   └── Entity: EnemyPortrait #0 [CharacterPortrait]
  │
  ├── Entity: BattleFieldArea
  │   └── Entity: GridOverlay
  │
  └── Entity: ActionMenu
      ├── Entity: AttackButton [PrimaryButton]
      ├── Entity: SkillButton [SecondaryButton]
      ├── Entity: WaitButton [SecondaryButton]
      └── Entity: CancelButton [DangerButton]
```

### 2.6 OnExit：despawn_screen System

```
// 触发条件：OnExit(GameState::Combat)
// 职责：回收所有 BattleScreen 相关的 Widget 实体

fn despawn_battle_screen(commands: &mut Commands, screen_query: Query<Entity, With<BattleScreen>>):
    for entity in screen_query:
        // despawn_recursive 会清理整个实体树
        // 所有子 Organism/Molecule/Atom 自动回收
        commands.entity(entity).despawn_recursive()

    // OnExit 不需要手动清理 UiStore 中的 ViewModel
    // ViewModel 在下一个 GameState 的 OnEnter 中重新投影
    // 或者保持原有值（下次进入 Combat 时重新填充）
```

OnExit 系统设计规则：
- 使用 `despawn_recursive()` 而非逐个清除子 Widget
- OnExit 不清理 UiStore（UiStore 由 Projection 管理）
- OnExit 不注销 Observer（Observer 在下次 Screen 进入时根据 run_if 条件自动过滤）

### 2.7 UiAction → UiCommand 转换

```
fn handle_battle_screen_commands(
    mut action_reader: EventReader<UiAction>,
    mut command_writer: EventWriter<UiCommand>,
    screen_state: Res<BattleScreenState>,
    vm_store: Res<UiStore>,
):
    for action in action_reader:
        match action:
            // ── 技能选择 → 技能施放 ──
            UiAction::SelectSkill(skill_id):
                // 屏幕级别的状态更新
                screen_state.selected_skill = Some(skill_id)
                // 不发射 UiCommand，等待目标选择完成

            UiAction::SelectCharacter(character_id):
                if screen_state.selected_skill.is_some():
                    // 有选中技能 → 发射施放命令
                    command_writer.send(
                        UiCommand::CastSkill(
                            screen_state.selected_skill.unwrap(),
                            character_id
                        )
                    )
                    screen_state.selected_skill = None
                else:
                    // 无选中技能 → 只是选择目标
                    screen_state.selected_target = Some(character_id)

            // ── 按钮操作 ──
            UiAction::Click:
                // Button 的 Click 需要通过 UiBinding 或实体 ID 判断来源
                // 这里简化为示例——实际情况需要匹配实体来源
                // （可以使用 Trigger<UiAction> + target entity 实现）
                pass

            UiAction::EndTurn:
                command_writer.send(UiCommand::EndTurn)
```

### 2.8 Overlay 依赖注册

```
// Screen Plugin 的 build() 中声明覆盖层依赖
// 这不是 Screen Plugin 的必需部分，但有助于维护 Overlay 依赖清单

fn register_overlay_dependencies(battle_screen: &mut ScreenOverlayRegistry):
    // BattleScreen 需要的 Overlay 类型
    battle_screen
        .requires::<DamageTextOverlay>()      // 伤害数字浮层（CueType::Popup 消费）
        .requires::<TooltipOverlay>()          // 技能/Buff 说明浮层
        .requires::<NotificationOverlay>()     // 战斗消息提示（升级、获得物品）
        .requires::<ModalOverlay>()            // 暂停菜单、退出确认
```

### 2.9 Screen Plugin 汇总

| Screen | OnEnter | OnExit | 使用的 Organism | 关键 UiCommand 转换 |
|--------|---------|--------|-----------------|---------------------|
| BattleScreen | GameState::Combat | Combat退出 | BattleHud, SkillPanel, TurnOrderBar, CharacterStatusPanel | CastSkill, SelectTarget, EndTurn |
| MainMenuScreen | GameState::MainMenu | MainMenu退出 | 无（仅 Panel + Atom 按钮） | OpenScreen, NewGame |
| InventoryScreen | GameState::Inventory | Inventory退出 | InventoryGrid | UseItem, EquipItem, DropItem |
| ShopScreen | OverlayState::Shop | Shop关闭 | ShopPanel, InventoryGrid | BuyItem, SellItem |
| SettingsScreen | GameState::Settings | Settings退出 | 无（仅 TabPanel + Toggle + Slider） | ChangeSettings |
| SaveLoadScreen | GameState::SaveLoad | SaveLoad退出 | 无（仅 SelectList + Panel + Button） | SaveGame, LoadGame |

---

## 3. ViewModel 更新周期

### 3.1 完整生命周期

数据变更从 Domain 事件到 UI 节点更新的完整链路（以 HP 条为例）：

```
1. Domain Event 触发
   ── DamageApplied { target: CharId(7), value: 85, source: SkillId(42) }

      │
      ▼

2. Observer 捕获 → 调用 Projection
   ── on_damage_applied.run_if(screen_is_active::<BattleScreen>)
   ── battle_projection::project_damage(&mut store, event)

      │
      ▼

3. Projection 纯函数写入 ViewModel
   ── store.battle_hud.characters[7].hp = 85
   ── store.battle_hud.characters[7].hp = max(85, 0)   // clamp 保护

      │
      ▼

4. Projection 设置 Dirty<T> = true
   ── store.battle_hud.mark_dirty()
   ── 等效于：Dirty<BattleHudVm> { is_dirty: true }

      │
      ▼

5. 下一帧 Update：on_dirty System 检测 Dirty<T>
   ── refresh_progress_bar System 运行
   ── dirty_query 查询所有 &Dirty<BattleHudVm>
   ── dirty.consume() 返回 true
   ── is_dirty = false（防止同帧重复消费）

      │
      ▼

6. System 从 ViewModel 读取新值
   ── hp = vm_store.battle_hud.hp           // 85
   ── max_hp = vm_store.battle_hud.max_hp   // 100
   ── ratio = 85.0 / 100.0 = 0.85

      │
      ▼

7. System 更新 UI Node
   ── UiBinding::Hp 对应的 Style.width = Val::Percent(85.0)
   ── UiBinding::MaxHp 对应的 Text = "85/100"

      │
      ▼

8. Widget 休眠直到下次标记
   ── 下一帧 consume() 返回 false
   ── 所有 System 跳过该 Widget
   ── 直到新的 Domain Event 触发又一轮周期
```

### 3.2 BattleHudVm + HpBar 具体例子

```
// ── 步骤 1：Domain Event ──
//
// Combat Domain 处理施放技能逻辑后，发射 Domain Event
Event: DamageApplied {
    target_id: CharacterId(7),
    value: 85,
    source: SkillId(42),
}


// ── 步骤 2：Observer ──
//
// UI 层注册的 Observer | 在 BattleScreen Plugin 初始化时注册
// 使用 run_if 条件：只在 BattleScreen 激活时触发

observer on_damage_applied:
    conditions: trigger(event = DamageApplied)
    run_if: screen_is_active::<BattleScreen>

    body:
        let store = ResMut<UiStore>
        let event = trigger.event()

        // 调用 Projection 纯函数
        BattleProjection::project_damage(&mut store, event)


// ── 步骤 3：Projection ──
//
// 纯函数：将 Domain Event 映射为 ViewModel 更新
// 不读取 UI 组件，不发射 UiEvent

fn BattleProjection::project_damage(store: &mut UiStore, event: &DamageApplied):
    let char_vm = store.battle_hud.characters.get_mut(&event.target_id)
    char_vm.hp = (char_vm.hp - event.value).max(0).min(char_vm.max_hp)

    // ！关键步骤：标记 Dirty
    store.battle_hud.mark_dirty()


// ── 步骤 4：Dirty<T> 标记 ──
//
// BattleHudVm 的 mark_dirty() 内部操作：
//    self.dirty_flag.store(true, Ordering::Release)
// 等效于 Dirty<BattleHudVm> { is_dirty: true }

struct Dirty<T> {
    inner: T,         // 实际数据
    is_dirty: bool,   // 标记位
}

impl<T> Dirty<T>:
    fn mark_dirty(&mut self):
        if !self.is_dirty:
            self.is_dirty = true

    fn consume(&mut self) -> bool:
        if self.is_dirty:
            self.is_dirty = false
            return true
        return false


// ── 步骤 5：on_dirty System ──
//
// 每帧运行，但只在 Dirty 为 true 时执行实际刷新逻辑

system refresh_battle_hud:
    query: &Dirty<BattleHudVm>
    vm_store: Res<UiStore>

    for dirty in query:
        if dirty.consume():
            refresh_hp_bars(&vm_store, ...)
            refresh_mp_bars(&vm_store, ...)
            refresh_ap_bars(&vm_store, ...)
            refresh_buff_icons(&vm_store, ...)
            refresh_turn_order(&vm_store, ...)


// ── 步骤 6-7：UI Node 更新 ──
//
// 在 refresh_hp_bars 内部：
//   逐行扫描 UiBinding，找到绑定的进度条后更新 Style.width

fn refresh_hp_bars(vm_store: &UiStore, binding_query: &mut Query<(&UiBinding, &mut Style)>):
    for (binding, mut style) in binding_query:
        if let UiBinding::Hp = binding:
            let char_vm = /* 找到对应的角色 */;
            let ratio = char_vm.hp as f32 / char_vm.max_hp as f32;
            style.width = Val::Percent(clamp(ratio, 0.0, 1.0) * 100.0);

        if let UiBinding::MaxHp = binding:
            // 更新文本显示 "85/100"


// ── 步骤 8：休眠 ──
//
// 下一帧：没有新的 Domain Event → Dirty 未被标记
//    consume() 返回 false → System 跳过所有 Widget
// 直到新的 DamageApplied 或 HealingApplied 等事件触发
```

### 3.3 性能关键点

| 阶段 | 性能行为 | 优化策略 |
|------|---------|---------|
| Observer 捕获 | 仅在事件触发时执行 | run_if 过滤不活跃的 Screen |
| Projection 写入 | O(1) 或 O(n)（n = 受影响实体数） | UiStore 使用 HashMap + 直接字段访问 |
| Dirty 标记 | O(1) | 原子操作，无锁 |
| on_dirty System | 每帧全量检查 Dirty flag，但只在 flag=true 时执行实际刷新 | consume() 后跳过 |
| UI Node 更新 | O(m)（m = 该 ViewModel 类型绑定的 UI 元素） | 只刷新标记了的绑定 |

### 3.4 多级 Dirty 传播

Dirty 标记是嵌套的——当 Organism 的 ViewModel 更新时，其内部所有子 Widget 应更新：

```
BattleHudVm.mark_dirty()
    │
    ├── BattleHud 的 on_dirty System 检测到 dirty
    ├── 它不直接刷新所有子 Widget
    │
    ├── 而是级联：CharacterStatusPanelVm、SkillPanelVm、TurnOrderBarVm
    │   的 Dirty 也被设置为 true（如果它们的子字段有变更）
    │
    ├── 每个 Organism/Molecule 的 on_dirty System 独立检测自己的 Dirty<T>
    │
    └── 性能收益：只有实际变更的字段触发对应的 Widget 刷新
        └── HP 变更 → 只刷新 HpBar，不刷新 SkillPanel 或 TurnOrderBar
```

级联 Dirty 可以通过以下方式之一实现：
- **方案 A（推荐）**：ViewModel 嵌套结构，父 mark_dirty() 时仅为直接子设置 dirty。各子 Widget 的 on_dirty System 独立查询子 Dirty ∈。如果子脏了就刷新。
- **方案 B**：Parent Projection 的 mark_dirty() 显式设置子 Dirty。更多代码但更确定。
- **方案 C（不推荐）**：父 Dirty 刷新时全量重新渲染所有子 Widget。违反 Dirty 的设计意图。

---

## 4. Overlay 触发模式

### 4.1 模式概述

Overlay（Tooltip/Notification/Modal/DamageText/Loading/Debug）是一类特殊的 UI 元素，它们：
- 不挂在 Screen 实体树下（属于独立层：TooltipLayer/NotificationLayer/PopupLayer/DebugLayer）
- 生命周期不受 Screen 影响（Screen 切换不销毁 Overlay）
- 触发源多样（用户交互、Domain Cue、系统事件、定时器）
- 通常自管理生命周期（定时消失、条件消失）

```
触发源                       Overlay Service              Bevy ECS
─────────                   ───────────────              ────────
Cue/CueBus ──────┐
Hover/交互  ──────┤
系统事件    ──────┼──→ Overlay Service ──→ ViewModel ──→ Spawn Widget → Timer → Despawn
定时器      ──────┤
领域事件    ──────┘
```

### 4.2 Overlay Service + Plugin 骨架

```
// Overlay Service Resource — 接收触发请求的入口
// 每个 Overlay 类型对应一个 Service（或一个统一的 Service 带类型参数）

resource NotificationService:
    queue: Vec<NotificationVm>      // 待显示的 Notification 队列
    active: Vec<Entity>             // 当前显示的 Notification 实体


// Overlay Plugin — 处理 Service 队列，管理 Widget 生命周期

plugin NotificationOverlayPlugin:

    fn build(app):
        // 1. 注册 Service
        app.init_resource<NotificationService>()

        // 2. 注册 Overlay ViewModel
        app.register_type::<Dirty<NotificationVm>>()

        // 3. 注册 Spawn System：Service 队列有数据 → 创建 Overlay Widget
        app.add_systems(Update, spawn_notification_from_queue)

        // 4. 注册 Timer System：检测超时 → 标记关闭
        app.add_systems(Update, tick_notification_timers)

        // 5. 注册 Cleanup System：关闭标记 → despawn
        app.add_systems(Update, despawn_expired_notifications)
```

### 4.3 DamageTextOverlay 完整触发链路

```
// ── 触发源：CueBus ──
// Combat Domain 处理伤害后，发射 Cue
// Cue 是逻辑→表现的单向通道（参见 ADR-012）

CueBus.receive(Cue {
    cue_type: CueType::Popup,
    payload: CuePayload::Damage {
        target_id: CharacterId(7),
        value: 85,
        damage_type: DamageType::Physical,
    },
})


// ── 投影步骤 ──
// UI 的 Observer 监听 CueTriggered 事件，投影为 DamageNumberVm

observer on_cue_triggered:
    conditions: trigger event CueTriggered where cue.cue_type == Popup

    body:
        let store = ResMut<UiStore>
        match event.cue.payload:
            CuePayload::Damage { target_id, value, damage_type } =>
                store.damage_numbers.push(DamageNumberVm {
                    text_key: UiTextKey("ui.combat.damage"),
                    params: { "value": value.to_string() },
                    position: get_screen_position(target_id),   // CharacterId → 屏幕位置
                    damage_type: damage_type,
                    animation: DamageAnim::FloatUp,
                    lifetime: 1.5,     // 1.5 秒后消失
                })
                store.damage_numbers.mark_dirty()


// ── Spawn ──
// System 检测到 DamageNumberVm 队列有数据 → 创建浮层 Widget

system spawn_damage_numbers:
    dirty_query: &Dirty<DamageNumberVm>
    service: Res<DamageTextOverlayService>

    for dirty in dirty_query:
        if dirty.consume():
            for vm in service.queue:
                entity = commands.spawn((
                    DamageNumber,           // Overlay 标记组件
                    Node { /* 绝对定位 */ },
                    Text(vm.text_key),      // 本地化文本（如 "-85"）
                    DamageAnimPlayer(vm.animation),
                    LifeTimer(vm.lifetime), // 生命周期 Timer
                ))
                commands.entity(UiRoot::NotificationLayer).add_child(entity)


// ── 动画 ──
// 每帧更新动画状态

system animate_damage_numbers:
    query: (&DamageAnimPlayer, &mut Transform, &LifeTimer)

    for (anim, mut transform, timer) in query:
        let progress = timer.elapsed / timer.duration

        match anim.current:
            DamageAnim::FloatUp =>
                transform.translate_y = lerp(0.0, -50.0, progress)
                transform.opacity = lerp(1.0, 0.0, clamp(progress * 2.0 - 1.0, 0.0, 1.0))
                // 前半段上移，后半段渐隐


// ── 生命周期结束 → Despawn ──
// Timer 归零时自动 despawn

system despawn_expired_damage_numbers:
    query: Entity & LifeTimer where timer.finished

    for entity in query:
        commands.entity(entity).despawn()
```

### 4.4 Tooltip Overlay 触发链路

Tooltip 的触发比较特殊——它依赖 Focus 系统或鼠标悬停：

```
1. 触发条件
   ── 用户将鼠标悬停在技能图标上超过 300ms
   ── 或键盘/手柄焦点移动到可聚焦元素上

2. Intent 路由
   ── 悬停 → FocusSystem 广播 UiEvent::FocusChanged(focus_id)
   ── 或 → 悬停定时器（300ms）到期

3. TooltipService 处理
   ── service.request_show(focus_id, cursor_position)
   ── service 查找 focus_id 对应的 TooltipVm
   ── service.active_tooltip = Some(tooltip_vm)

4. Spawn
   ── system 检测 service 变更 → spawn Tooltip Widget
   ── Widget 层级：UiRoot::TooltipLayer

5. Despawn
   ── 鼠标移出 → service.request_hide() → despawn
   ── 或焦点移走 → FocusSystem → UiEvent::FocusChanged → despawn
```

### 4.5 Notification Overlay 触发链路

```
1. 触发源
   ── 战斗事件：获得物品、升级、成就解锁
   ── 系统事件：存档完成、设置保存
   ── 交易事件：购买成功

2. Service 接收
   ── NotificationService.push(NotificationVm {
       message_key: UiTextKey("ui.notification.item_acquired"),
       params: { "item_name": item_name },
       priority: Normal,
       notification_type: Toast,
       duration: 3.0,
   })

3. Spawn
   ── system 检测 queue 非空 → spawn Notification Widget
   ── 支持队列（最多同时显示 3 个，超出排队）
   ── 优先级高的覆盖优先级低的

4. Timer
   ── 检测 timer.elapsed >= duration → mark dismissed

5. Despawn
   ── dismissed = true → despawn
   ── 或用户手动关闭（UiAction::Dismiss）→ despawn
```

### 4.6 Modal Overlay 触发链路

```
1. 触发源
   ── 用户点击危险操作（丢弃物品、放弃任务、删除存档）
   ── UiCommand 需要用户确认（如覆盖存档）
   ── Screen 在转换前需要 Modal 确认

2. Service 接收
   ── ModalService.push(ModalVm {
       title_key: UiTextKey("ui.modal.confirm_drop"),
       body_key: UiTextKey("ui.modal.confirm_drop_body"),
       body_params: { "item_name": item_name },
       buttons: [
           { label_key: "ui.button.cancel", action: UiAction::Cancel, style: Danger },
           { label_key: "ui.button.confirm", action: UiAction::Confirm, style: Primary },
       ],
   })

3. Spawn
   ── system 检测 queue 非空 → spawn Modal Widget
   ── Widget 层级：UiRoot::PopupLayer
   ── 阻塞下层 Screen 的 FocusGroup 交互（FocusGroup 切换）

4. 用户响应
   ── 点击确认 → UiAction::Confirm → ModalService 回调 → 执行原命令
   ── 点击取消 → UiAction::Cancel → ModalService 回调 → 关闭 Modal

5. Despawn
   ── 确认/取消后 → service.pop() → despawn Modal Widget
   ── 恢复下层 Screen 的 FocusGroup
```

### 4.7 Overlay 汇总

| Overlay | 层级 | 触发源 | 生命周期 | 目标 Widget |
|---------|------|--------|---------|------------|
| DamageText | PopupLayer | CueBus (CueType::Popup) | Timer 1.5s | DamageNumber |
| Tooltip | TooltipLayer | Focus/Hover | 悬停/焦点持续 | Tooltip |
| Notification | NotificationLayer | NotificationService | 3-5s Timer / 手动关闭 | Notification |
| Modal | PopupLayer | ModalService | 用户确认/取消 | Modal |
| Loading | PopupLayer | GameState 切换 | 资源加载完成 | LoadingSpinner |
| Debug | DebugLayer | F12 | 切换到关 | DebugPanel |

---

## 5. 生命周期概览图

完整的 UI 运行时生命周期，从用户操作开始到表现层结束：

```
用户操作开始
    │
    │  ┌─────────────────────────────────────────────────────────────────┐
    │  │                    输入层级 (Input)                              │
    │  │  InputAction (Key/Mouse/Gamepad)                                │
    │  │      ↓                                                          │
    │  │  InputSystem 映射（不含业务逻辑）                                  │
    │  │      ↓                                                          │
    │  │  UiIntent (NavigateUp/SelectSkill/Confirm)                       │
    │  └─────────────────────────────────────────────────────────────────┘
    │
    ▼
    │  ┌─────────────────────────────────────────────────────────────────┐
    │  │                   屏幕层级 (Screen)                              │
    │  │  OnEnter(GameState) → Screen Plugin                             │
    │  │      │                                                          │
    │  │      ├── Screen Resource 初始化                                   │
    │  │      ├── Observer 注册（Domain Event → Projection）               │
    │  │      └── WidgetFactory::create() → Widget 实体树 spawn            │
    │  │          │                                                      │
    │  │          ├── Organism Entity （BattleHud）                        │
    │  │          ├── Molecule Entity （SkillSlot）                        │
    │  │          └── Atom Entity × N（Button/ProgressBar/Text）           │
    │  └─────────────────────────────────────────────────────────────────┘
    │
    ▼
    │  ┌─────────────────────────────────────────────────────────────────┐
    │  │                    命令层级 (Command)                             │
    │  │  Widget 交互 → UiAction                                          │
    │  │      ↓                                                          │
    │  │  Screen Plugin UiActionHandler → UiCommand                       │
    │  │      ↓                                                          │
    │  │  application/command.rs → GameCommand                            │
    │  │      ↓                                                          │
    │  │  ADR-043 CommandQueue → Domain 执行                               │
    │  └─────────────────────────────────────────────────────────────────┘
    │
    ▼
    │  ┌─────────────────────────────────────────────────────────────────┐
    │  │                    投影层级 (Projection)                          │
    │  │  Domain Event 触发（如 DamageApplied）                             │
    │  │      ↓                                                          │
    │  │  Observer 捕获（带 run_if 条件）                                   │
    │  │      ↓                                                          │
    │  │  Projection 纯函数（BattleProjection::project_damage）             │
    │  │      │                                                          │
    │  │      ├── 读取 Domain Component（唯一合法的 Domain Query）           │
    │  │      ├── 写入 UiStore（ViewModel 字段更新）                        │
    │  │      └── store.battle_hud.mark_dirty()                          │
    │  └─────────────────────────────────────────────────────────────────┘
    │
    ▼
    │  ┌─────────────────────────────────────────────────────────────────┐
    │  │                    更新层级 (Update)                              │
    │  │  下一帧 Update 调度                                               │
    │  │      ↓                                                          │
    │  │  Dirty<T>.consume() → true（标记被消费）                          │
    │  │      ↓                                                          │
    │  │  on_dirty System 运行                                            │
    │  │      │                                                          │
    │  │      ├── 读取 ViewModel 最新值                                    │
    │  │      ├── 通过 UiBinding 定位目标 Node                             │
    │  │      └── 更新 Style/Text/Image 等组件                             │
    │  │          │                                                      │
    │  │          ├── Style.width = Val::Percent(85)   （进度条填充）       │
    │  │          ├── Text = "85/100"                   （文本标签）       │
    │  │          └── UiImage 替换                        （图标变更）      │
    │  │                                                               │
    │  │  Dirty<T>.is_dirty = false（休眠直到下次标记）                    │
    │  └─────────────────────────────────────────────────────────────────┘
    │
    ▼
    │  ┌─────────────────────────────────────────────────────────────────┐
    │  │                    表现层级 (Overlay)                             │
    │  │  Cue 触发（CueBus.receive）— 或 NotificationService.push         │
    │  │      ↓                                                          │
    │  │  Overlay Projection → OverlayVm                                  │
    │  │      ↓                                                          │
    │  │  Overlay Service 处理队列                                        │
    │  │      │                                                          │
    │  │      ├── 独立层 Spawn（TooltipLayer / NotificationLayer 等）      │
    │  │      ├── 动画播放（上移、淡出、放大）                              │
    │  │      ├── Timer 跟踪                                              │
    │  │      └── Timer 结束 → Despawn                                   │
    │  └─────────────────────────────────────────────────────────────────┘
    │
    ▼
    │  ┌─────────────────────────────────────────────────────────────────┐
    │  │                    退出层级 (Exit)                                │
    │  │  OnExit(GameState) → Screen Plugin                              │
    │  │      │                                                          │
    │  │      ├── despawn_recursive() → Widget 实体树回收                  │
    │  │      ├── Screen Resource 清理（如果非持久状态）                    │
    │  │      └── Observer run_if 自动过滤（不活跃的 Screen 不监听）        │
    │  └─────────────────────────────────────────────────────────────────┘
```

### 5.1 各层级与上游文档的对应

| 层级 | 本文模式 | 上游参考 |
|------|---------|---------|
| 输入层级 | §1.6 事件冒泡路径 | application-layer.md §2 (UiIntent), §3 (UiAction) |
| 屏幕层级 | §2 Screen Plugin 模式 | screens.md §2-§7, widget-composites.md §3 |
| 命令层级 | §1.6 事件冒泡路径 | application-layer.md §4 (UiCommand), §6 (映射链) |
| 投影层级 | §3 ViewModel 更新周期 | focus-binding.md §3 (Dirty<T>), ADR-055 §3 (数据流) |
| 更新层级 | §3 ViewModel 更新周期 | focus-binding.md §3 Dirty 消费, §4 (UiBinding) |
| 表现层级 | §4 Overlay 触发模式 | ADR-012 (Cue), overlays.md |
| 退出层级 | §2.6 despawn_screen | screen-lifecycle.md |

---

## 6. Feature Developer Handoff Checklist

feature-developer 实现一个 UI 功能时，参考以下流程和对应文件：

### 6.1 实现顺序

```
Step 1: 确认 Contract 定义            → widget-atoms.md（原子组件）或 widget-composites.md（复合组件）
Step 2: 确认 ViewModel 定义           → projection-viewmodel.md（在 04-data-flow/ 下）
Step 3: 确认 Projection 映射          → projection-viewmodel.md（Projection 纯函数）
Step 4: 实现 Widget Plugin            → 本文 §1 Widget Plugin 模式 + widget-atoms.md §X
Step 5: 实现 Composite Widget Plugin  → 本文 §1 + widget-composites.md §X
Step 6: 实现 Screen Plugin            → 本文 §2 Screen Plugin 模式 + screens.md §X
Step 7: 注册 Overlay 依赖             → 本文 §4 Overlay 触发模式 + overlays.md
Step 8: 注册 UiCommand 转换           → application-layer.md §4.3 + 本文 §2.7
```

### 6.2 问题排查 Checklist

| 症状 | 检查位置 | 模式参考 |
|------|---------|---------|
| Widget 不显示 | ① Props 是否正确设置？② OnEnter 被触发？③ Widget 实体树下有 UiRoot？ | §2.4 spawn_screen |
| Widget 不更新 | ① Projection 是否调用了 mark_dirty()？② Dirty<T>.consume() 是否返回 true？③ UiBinding 是否正确匹配？ | §3.2 Dirty 消费 |
| 交互无响应 | ① 是否有 on_interaction System？② UiAction 是否发射？③ Screen 的 UiActionHandler 是否处理？ | §1.6 事件冒泡 |
| Overlay 不显示 | ① Service 队列是否有数据？② Spawn System 是否运行？③ Overlay Plugin 是否注册？ | §4.3 Overlay 触发 |
| Screen 切换异常 | ① OnEnter/OnExit 是否正确关联？② GameState 映射是否正确？③ despawn_recursive 是否执行？ | §2 生命周期 |

### 6.3 完整文件引用清单

```
docs/06-ui/01-architecture/
├── architecture.md                 ── L3 架构总纲（定位、数据流、通信、宪法规则）
├── application-layer.md            ── UiIntent/UiCommand/UiEvent + 映射链
└── implementation-patterns.md      ── [本文] Widget/Screen/ViewModel/Overlay 的 Bevy ECS 骨架

docs/06-ui/02-design-system/
├── widget-atoms.md                 ── 21 个原子组件的 Props/Events/State/变体 ← Step 1
├── widget-composites.md            ── 16 个复合组件（Molecule/Organism）           ← Step 1
├── theme-localization.md           ── StyleToken/Theme/UiTextKey
└── focus-binding.md                ── Focusable/FocusGroup/Dirty<T>/UiBinding     ← Step 4

docs/06-ui/03-screens/
├── screen-lifecycle.md             ── Screen 生命周期状态机
├── screens.md                      ── 6 个 Screen 的 Widget 组合树细节             ← Step 6
├── navigation-overlay.md           ── ScreenStack/Overlay 分层/Focus 系统
└── overlays.md                     ── Tooltip/DamageText/Notification/Modal 详细设计 ← Step 7

docs/06-ui/04-data-flow/
├── projection-viewmodel.md         ── Projection 纯函数/ViewModel 规范/Dirty<UiStore>  ← Step 2-3
└── camera-ui-interaction.md        ── Camera-UI 交互规则（CameraQuery/CameraRequest/CameraInputBlock）

docs/06-ui/05-testing/
└── testing.md                      ── Widget 单元/Screen 集成/快照/Mock Projection
```

---

*本文档由 @presentation-architect 维护。新增实现模式需通过架构审查。新增 Widget/Screen/Overlay 时，feature-developer 应严格按照 Handoff Checklist 的顺序确认上游文档、再参照本文模式编写 Plugin 代码。*
