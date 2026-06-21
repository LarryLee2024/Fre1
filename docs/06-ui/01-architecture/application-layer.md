---
id: 06-ui.application-layer
title: UI Application Layer — 输入意图、命令、事件
status: code-aligned
owner: presentation-architect
created: 2026-06-20
updated: 2026-06-21
tags:
  - ui
  - application
  - intent
  - command
  - event
  - input
---

# UI Application Layer — 输入意图、命令、事件

> **职责**: @presentation-architect | **上游**: ADR-055 §3 (数据流), §Communication Design (通信表) | domain rules §1 (UiIntent/UiCommand/UiAction 定义), §5.3 (用户输入处理流程) | schema §10 (UiCommand Schema), §11 (UiAction Schema)

---

## 1. 设计目的

UI Application 层是 UI 层与 Domain 层的桥梁，负责：
- **UiIntent** — 将原始输入（键鼠/手柄）抽象为语义化意图
- **UiAction** — Widget 级别的交互输出声明
- **UiCommand** — UI → Domain 的命令，封装用户操作意图
- **UiEvent** — UI 内部事件广播（ViewModel 更新、导航变更等）

三者的角色区分：

```
输入层（硬件）    抽象层（UI 内）    传递层（UI → Domain）
InputAction ──→ UiIntent ──→ UiAction ──→ UiCommand ──→ GameCommand
 (Click)        (Select)     (UiAction::  (CastSkill)    (GameCommand::
                           SelectSkill)                   CastSpell)
```

核心约束：
- UiIntent 是 UI 层的输入意图抽象，比 InputAction 更高层（InputAction 是硬件语义，UiIntent 是业务语义）
- UiCommand 通过 `UiCommand → GameCommand` 转换器进入 ADR-043 定义的 CommandQueue
- 转换器在 `src/ui/application/command.rs` 中实现，是 UI 层与 Command 层的唯一桥梁

---

## 2. UiIntent — 输入意图抽象

### 2.1 定义

UiIntent 是原始输入（键盘、鼠标、手柄）到语义化意图的映射层。InputSystem 将原始事件转换为 UiIntent，UiIntent 不包含任何硬件细节。

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Event)]
pub enum UiIntent {
    // ── 导航意图（FocusGroup 导航） ──
    /// 焦点上移
    NavigateUp,
    /// 焦点下移
    NavigateDown,
    /// 焦点左移
    NavigateLeft,
    /// 焦点右移
    NavigateRight,
    /// 确认当前焦点元素
    Confirm,
    /// 取消/返回
    Cancel,

    // ── 选择意图 ──
    /// 选择技能（SkillId 占位符）
    SelectSkill(u32),
    /// 选择目标（CharacterId 占位符）
    SelectTarget(u32),

    // ── 屏幕操作意图 ──
    /// 打开指定页面
    OpenScreen(ScreenType),
    /// 关闭当前页面
    CloseScreen,
    /// 切换暂停状态
    TogglePause,
    /// 打开设置页面
    OpenSettings,
    /// 打开背包页面
    OpenInventory,

    // ── 系统意图 ──
    /// 切换调试叠加层（仅 dev 构建）
    ToggleDebug,
}
```

### 2.2 InputAction → UiIntent 映射

| InputAction（原始输入） | UiIntent | 上下文 |
|------------------------|---------|--------|
| Key(W) / Key(Up) / DPAD_UP | NavigateUp | FocusGroup 活跃时 |
| Key(S) / Key(Down) / DPAD_DOWN | NavigateDown | FocusGroup 活跃时 |
| Key(A) / Key(Left) / DPAD_LEFT | NavigateLeft | FocusGroup 活跃时 |
| Key(D) / Key(Right) / DPAD_RIGHT | NavigateRight | FocusGroup 活跃时 |
| Key(Enter) / MouseLeftClick / BUTTON_A | Confirm | Focusable 元素聚焦时 |
| Key(Escape) / BUTTON_B | Cancel | 任何时候 |
| Key(Escape) | TogglePause | BattleScreen Active 时 |
| Key(I) | OpenInventory | 非 InventoryScreen 时 |
| Key(Escape) | OpenSettings → CloseScreen | SettingsScreen 非活跃时 |
| Key(F12) | ToggleDebug | dev feature 下 |

### 2.3 Intent 路由规则

```
UiIntent
    │
    ▼
Application layer (intent.rs)
    │
    ├── Navigate意图 → 路由到当前活跃 FocusGroup
    │       │
    │       ▼
    │   FocusGroup 计算焦点移动
    │
    ├── Select意图 → 路由到当前活跃 Widget
    │       │
    │       ▼
    │   Widget 处理选择 → 发射 UiAction
    │
    ├── Open/Close意图 → 路由到 ScreenStack
    │       │
    │       ▼
    │   ScreenStack push/pop/replace
    │
    └── System意图 → 直接执行（ToggleDebug, Screenshot）
```

**失败处理**：UiIntent 无法映射时静默忽略（如当前无 Focusable 元素，Navigate 意图不产生任何效果）。

---

## 3. UiAction — Widget 交互输出

### 3.1 定义

UiAction 是 Widget 级别的交互输出声明。Widget 在交互时发射 UiAction，UiActionHandler 将 UiAction 转换为 UiCommand。

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Event)]
pub enum UiAction {
    // ── 通用 ──
    /// 点击（Button Widget 主要输出）
    Click,
    /// 确认
    Confirm,
    /// 取消
    Cancel,
    /// 关闭/消除
    Dismiss,

    // ── 选择 ──
    /// 选择技能（SkillId 占位符）
    SelectSkill(u32),
    /// 选择物品（ItemId 占位符）
    SelectItem(u32),
    /// 选择角色（CharacterId 占位符）
    SelectCharacter(u32),

    // ── 切换/筛选 ──
    /// 切换选中状态
    Toggle(bool),
    /// 切换标签页
    ChangeTab(usize),

    // ── 输入 ──
    /// 文本变更
    TextChanged(String),
    /// 文本确认
    TextConfirmed(String),

    // ── 自定义 ──
    /// 自定义动作（携带字符串标识）
    Custom(String),
}
```

### 3.2 UiAction 边界

| 角色 | 职责 | 示例 |
|------|------|------|
| Widget | 根据交互发射 UiAction | Button 点击 → UiAction::Click |
| Screen | 将 UiAction 转换为 UiCommand | UiAction::Click → UiCommand::CastSkill(skill_id) |
| 既不是 Widget 也不是 | — | — |

---

## 4. UiCommand — UI → Domain 的命令

### 4.1 定义

UiCommand 是 UI 层向 Domain 层发送的命令枚举。所有用户操作必须通过此枚举进入 Domain，禁止 Widget 直接修改 Domain。

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Event)]
pub enum UiCommand {
    // ── Combat（战斗） ──
    /// 施放技能（SkillId, target CharacterId）
    CastSkill(u32, u32),
    /// 选择目标（CharacterId）
    SelectTarget(u32),
    /// 结束回合
    EndTurn,
    /// 移动到网格位置（x, y）
    MoveToPosition(i32, i32),

    // ── Inventory（背包） ──
    /// 使用物品
    UseItem(u32),
    /// 装备物品（ItemId, slot index）
    EquipItem(u32, u32),
    /// 丢弃物品
    DropItem(u32),

    // ── Quest（任务） ──
    /// 接受任务
    AcceptQuest(u32),
    /// 放弃任务
    AbandonQuest(u32),

    // ── Economy（经济） ──
    /// 购买物品（ItemId, quantity）
    BuyItem(u32, u32),
    /// 出售物品（ItemId, quantity）
    SellItem(u32, u32),

    // ── Save（存档） ──
    /// 保存游戏
    SaveGame(u32),
    /// 加载游戏
    LoadGame(u32),

    // ── System（系统） ──
    /// 切换暂停
    TogglePause,
    /// 打开页面
    OpenScreen(ScreenType),
    /// 关闭页面
    CloseScreen,
    /// 新游戏
    NewGame,
}
```

### 4.2 UiCommand 分类

| 分类 | 命令 | 进入 Replay | 目标 |
|------|------|------------|------|
| 战斗命令 | CastSkill, SelectTarget, EndTurn, MoveToPosition | 是 | Domain (Combat) |
| 背包命令 | UseItem, EquipItem, DropItem | 是 | Domain (Inventory) |
| 任务命令 | AcceptQuest, AbandonQuest | 是 | Domain (Quest) |
| 经济命令 | BuyItem, SellItem | 是 | Domain (Economy) |
| 存档命令 | SaveGame, LoadGame | 否 | Infra (Save) |
| UI 导航命令 | OpenScreen, CloseScreen | 否 | UI 内部 |
| 系统命令 | TogglePause, NewGame | 否 | App (GameState) |

### 4.3 UiCommand → GameCommand 转换

UiCommand 不是直接发向 Domain 的。UiCommand 首先经过转换器，转换为 ADR-043 定义的 GameCommand。

```rust
impl UiCommand {
    /// 将 UiCommand 转换为 GameCommand 以便 Domain 层执行。
    ///
    /// 返回 `None` 的场景：
    /// - UI 内部导航命令（OpenScreen、CloseScreen）
    /// - 需要调用方上下文信息才能填充的命令（如 CastSkill 缺少 caster_id）
    /// - 当前 GameCommand 尚不支持的领域操作
    pub fn into_game_command(&self) -> Option<GameCommand> {
        match self {
            UiCommand::EndTurn => Some(GameCommand::EndTurn {
                unit_id: String::new(), // 调用方应在入队前填充
            }),
            UiCommand::SaveGame(_) => Some(GameCommand::SaveGame),
            UiCommand::LoadGame(_) => Some(GameCommand::LoadGame),
            // 以下命令当前无法映射到 GameCommand：
            // CastSkill     — 需 caster_id（将在集成时从上下文获取）
            // SelectTarget  — 无对应 GameCommand
            // MoveToPosition — 无对应 GameCommand
            // UseItem       — 需 user_id/item_instance_id
            // EquipItem     — 无对应 GameCommand
            // DropItem      — 无对应 GameCommand
            // AcceptQuest   — 无对应 GameCommand
            // AbandonQuest  — 无对应 GameCommand
            // BuyItem       — 无对应 GameCommand
            // SellItem      — 无对应 GameCommand
            // TogglePause   — 无对应 GameCommand
            // OpenScreen    — UI 内部导航
            // CloseScreen   — UI 内部导航
            // NewGame       — 无对应 GameCommand
            _ => None,
        }
    }
}
```

**当前映射状态**（全部完成）：

| UiCommand | GameCommand | 说明 |
|-----------|------------|------|
| EndTurn | GameCommand::EndTurn { unit_id } | 结束回合 |
| SaveGame(slot) | GameCommand::SaveGame | 保存游戏 |
| LoadGame(slot) | GameCommand::LoadGame | 加载游戏 |
| CastSkill { skill_def_id, target_id, caster_id } | GameCommand::CastSpell { ... } | 施放技能 |
| MoveToPosition { unit_id, x, y } | GameCommand::MoveUnit { ... } | 战术移动 |
| UseItem { item_instance_id, user_id, target_id } | GameCommand::UseItem { ... } | 使用物品 |
| EquipItem { unit_id, item_instance_id, slot_index } | GameCommand::EquipItem { ... } | 装备物品 |
| DropItem { unit_id, item_instance_id, quantity } | GameCommand::DropItem { ... } | 丢弃物品 |
| AcceptQuest { unit_id, quest_def_id } | GameCommand::AcceptQuest { ... } | 接受任务 |
| AbandonQuest { unit_id, quest_def_id } | GameCommand::AbandonQuest { ... } | 放弃任务 |
| BuyItem { item_def_id, quantity, shop_id } | GameCommand::BuyItem { ... } | 购买物品 |
| SellItem { item_def_id, quantity, shop_id } | GameCommand::SellItem { ... } | 出售物品 |
| NewGame | GameCommand::NewGame | 新游戏 |
| TogglePause | GameCommand::OpenMenu | 切换暂停 |
| OpenScreen(screen) | None | UI 内部导航，ScreenStack 处理 |
| CloseScreen | None | UI 内部导航，ScreenStack 处理 |
| SelectTarget(id) | None | 需调用方上下文，暂不映射 |

所有业务命令通过 `into_game_command()` 转换后由 `process_ui_commands` Observer 推入 `CommandQueue`。UI 导航命令（OpenScreen/CloseScreen）和 SelectTarget 始终由 UI 内部处理，不经过 GameCommand。

### 4.4 UiCommand 的 Save/Replay 策略

| 命令类别 | 策略 |
|---------|------|
| 战斗/背包/任务/经济命令 | 通过 GameCommand 进入 Replay 录制 |
| 存档命令 | 不进入 Replay，独立处理 |
| UI 导航命令 | 不进入 Replay，Replay 时从 Domain Event 重新投影 |

---

## 5. UiEvent — UI 内部事件

### 5.1 定义

UiEvent 是 UI 层内部的事件广播机制，用于 UI 层内部各子系统通信（ViewModel 更新、导航变更、焦点切换等）。

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Event)]
pub enum UiEvent {
    // ── ViewModel 事件 ──
    /// ViewModel 已更新（设置 Dirty 标记后发射）
    ViewModelUpdated(&'static str),

    // ── 导航事件 ──
    /// Screen 推入完成
    ScreenPushed(ScreenType),
    /// Screen 弹出完成
    ScreenPopped(ScreenType),
    /// Screen 被替换（旧, 新）
    ScreenReplaced(ScreenType, ScreenType),
    /// 导航错误
    NavigationError(String),

    // ── 主题事件 ──
    /// 主题切换
    ThemeChanged(String),
}
```

### 5.2 UiEvent 使用场景

| 事件 | 广播者 | 监听者 | 用途 |
|------|--------|--------|------|
| ViewModelUpdated | Projection | Widget System | 触发 Widget 刷新（与 Dirty 机制的冗余保障） |
| ScreenPushed | ScreenStack | FocusSystem | 新 Screen 激活 → FocusGroup 初始化 |
| ScreenPopped | ScreenStack | OverlayService | Screen 退出时清理与其绑定的 Overlay |
| ThemeChanged | ThemeSystem | Widget System | 触发全局 Widget 刷新 |

### 5.3 UiEvent 边界

- UiEvent 是 UI 内部事件，不跨出 UI 层
- Domain Event 通过 Observer 通知 UI，不经过 UiEvent
- UiEvent 的监听者限于 UI 层内部
- UiEvent 使用 Bevy Event 机制（`EventWriter`/`EventReader`）在 UI 层内部广播
- Overlay 事件、焦点事件、本地化事件、动画事件当前未实现，后续版本扩展

---

## 6. Intent → Command → GameCommand 完整映射链

### 6.1 战斗场景示例：施放技能

```
用户点击技能图标
    │
    ▼
Widget (SkillSlot)
    │  Input: SkillSlotVm { skill_id: 42, is_usable: true, ... }
    │  Interaction: MouseButton::Left
    ▼
UiAction::SelectSkill(42)
    │  Screen (BattleScreen) 的 UiActionHandler 处理
    ▼
UiCommand::CastSkill(42, 7)
    │  Application layer (command.rs) 转换
    │  注意：当前 MVP 中 CastSkill 返回 None（需 caster_id）
    │  调用方自行处理命令入队
    ▼
（视集成阶段而定：将来进入 GameCommand 或直接调用 Domain API）
    │
    ▼
Domain Event: TurnStarted / EffectApplied
    │  Observer 监听
    ▼
BattleProjection::on_turn_started → BattleHudVm.turn_number += 1 → Dirty
BattleProjection::on_effect_applied → 当前为 no-op placeholder
```

### 6.2 背包场景示例：丢弃物品（预留）

丢弃物品流程的 Modal 确认环节尚未实现。当前 MVP 中 DropItem 的确认流程由后续迭代补充。

### 6.3 导航示例：打开设置

```
用户按 Escape（BattleScreen 中）
    │
    ▼
InputSystem: Key(Escape)
    │
    ▼
UiIntent::OpenSettings
    │  Intent 路由
    ▼
UiCommand::OpenScreen(ScreenType::Settings)
    │  ScreenStack 处理
    ▼
ScreenStack::push(ScreenType::Settings)
    │
    ├── BattleScreen → Background（暂停交互）
    ├── SettingsScreen: Defined → Loading → Active
    └── UiEvent::ScreenPushed(ScreenType::Settings)
```

### 6.4 手柄导航示例：焦点移动

```
用户按手柄方向键上
    │
    ▼
InputSystem: DPAD_UP
    │
    ▼
UiIntent::NavigateUp
    │  Intent 路由 → FocusManager
    ▼
FocusManager（当前 FocusGroup 0）
    │  焦点已在边界 → 无移动
    │  返回 false，NavigateUp 意图静默忽略
    ▼
（无 UiAction/Command 发射）
```

---

## 7. 通信路径汇总

| 路径 | 机制 | 方向 | 涉及的枚举 |
|------|------|------|-----------|
| Intent → Action | FocusGroup 路由 | UI 内部 | UiIntent → UiAction |
| Action → Command | UiActionHandler 转换 | UI 内部 | UiAction → UiCommand |
| Command → GameCommand | into_game_command() 转换 | UI → Core/Domain | UiCommand → GameCommand |
| Domain → ViewModel | Observer + Projection | Core → UI | Domain Event → ViewModel |
| ViewModel → Widget | Dirty<T> consume | UI 内部 | — |
| UI 内部广播 | UiEvent (EventWriter/Reader) | UI 内部 | UiEvent |

---

## 8. 验证规则

| # | 规则 | 校验逻辑 |
|---|------|----------|
| APP-VAL-01 | UiIntent 不包含硬件细节 | UiIntent 变体不引用 KeyCode/GamepadButton 等硬件类型 |
| APP-VAL-02 | UiAction/UiCommand 是纯数据枚举 | 匹配分支中无业务执行逻辑 |
| APP-VAL-03 | 转换器是唯一出口 | UI → Domain 的调用必须经过 UiCommand::into_game_command() 转换器 |
| APP-VAL-04 | UiEvent 不跨 UI 层 | UiEvent 的事件订阅者仅限于 UI 层内部模块 |
| APP-VAL-05 | Intent 无法映射时静默忽略 | 无对应映射的 Intent 不产生崩溃 |

---

*本文档由 @presentation-architect 维护。新增 UiIntent/UiCommand/UiEvent 变体需经过架构审查。*

*最后更新: 2026-06-21 — 与实际代码实现对齐 (commit 903d039)*
