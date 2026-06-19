---
id: 06-ui.application-layer
title: UI Application Layer — 输入意图、命令、事件
status: draft
owner: presentation-architect
created: 2026-06-20
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
#[derive(Clone, Reflect)]
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
    /// 选择技能（手柄方向键+技能槽位映射）
    SelectSkill(SkillId),
    /// 选择目标（方向键选择场上角色）
    SelectTarget(CharacterId),
    /// 选择网格位置
    SelectGridPos(GridPos),
    /// 选择物品槽位
    SelectItem(ItemId),
    /// 选择存档槽位
    SelectSaveSlot(SaveSlot),

    // ── 操作意图 ──
    /// 打开页面
    OpenScreen(ScreenType),
    /// 关闭页面
    CloseScreen,
    /// 切换暂停
    TogglePause,
    /// 打开设置
    OpenSettings,
    /// 打开背包
    OpenInventory,

    // ── 系统意图 ──
    /// 切换 Debug Overlay（仅 dev）
    ToggleDebug,
    /// 截图
    Screenshot,
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
#[derive(Clone, Reflect)]
pub enum UiAction {
    // ── 通用 ──
    /// 确认
    Confirm,
    /// 取消
    Cancel,
    /// 关闭/消除
    Dismiss,

    // ── 选择 ──
    /// 点击（Button Widget 主要输出）
    Click,
    /// 选择技能
    SelectSkill(SkillId),
    /// 选择物品
    SelectItem(ItemId),
    /// 选择角色
    SelectCharacter(CharacterId),
    /// 选择网格位置
    SelectGridPos(GridPos),

    // ── 切换/筛选 ──
    /// 切换选中状态
    Toggle(bool),
    /// 切换标签页
    ChangeTab(usize),
    /// 切换筛选条件
    ChangeFilter(InventoryFilterVm),
    /// 切换排序方式
    ChangeSort(InventorySortOrder),

    // ── 输入 ──
    /// 文本变更
    TextChanged(String),
    /// 文本确认
    TextConfirmed(String),

    // ── 上下文 ──
    /// 显示上下文菜单
    ShowContextMenu(ItemId, Vec2),

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
#[derive(Clone, Reflect)]
pub enum UiCommand {
    // ── Combat（战斗） ──
    /// 施放技能（SkillId, target: CharacterId）
    CastSkill(SkillId, CharacterId),
    /// 选择目标（CharacterId）
    SelectTarget(CharacterId),
    /// 结束回合
    EndTurn,
    /// 移动到网格位置
    MoveToPosition(GridPos),

    // ── Inventory（背包） ──
    /// 使用物品
    UseItem(ItemId),
    /// 装备物品
    EquipItem(ItemId, EquipmentSlot),
    /// 丢弃物品
    DropItem(ItemId),

    // ── Quest（任务） ──
    /// 接受任务
    AcceptQuest(QuestId),
    /// 放弃任务
    AbandonQuest(QuestId),

    // ── Economy（经济） ──
    /// 购买物品（ItemId, quantity）
    BuyItem(ItemId, u32),
    /// 出售物品（ItemId, quantity）
    SellItem(ItemId, u32),

    // ── Save（存档） ──
    /// 保存游戏
    SaveGame(SaveSlot),
    /// 加载游戏
    LoadGame(SaveSlot),

    // ── Settings（设置） ──
    /// 修改设置
    ChangeSettings(UiSettings),

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
| 设置命令 | ChangeSettings | 否 | Infra (Settings) |
| UI 导航命令 | OpenScreen, CloseScreen | 否 | UI 内部 |
| 系统命令 | TogglePause, NewGame | 否 | App (GameState) |

### 4.3 UiCommand → GameCommand 转换

UiCommand 不是直接发向 Domain 的。UiCommand 首先经过转换器，转换为 ADR-043 定义的 GameCommand。

```
UiCommand::CastSkill(skill_id, target_id)
    │
    ▼
Application layer (command.rs — 转换器)
    │   match command {
    │       UiCommand::CastSkill(skill_id, target_id) =>
    │           GameCommand::Ability(CastAbility { skill_id, caster_id, target_id }),
    │   }
    ▼
GameCommand（进入 ADR-043 CommandQueue）
    │
    ▼
Domain 执行
```

**完整映射表**：

| UiCommand | GameCommand | 说明 |
|-----------|------------|------|
| CastSkill(sid, cid) | GameCommand::Ability(CastAbility {..}) | 施放技能 |
| EndTurn | GameCommand::Combat(EndTurn) | 结束回合 |
| MoveToPosition(pos) | GameCommand::Tactical(MoveTo {..}) | 移动 |
| UseItem(iid) | GameCommand::Inventory(UseItem {..}) | 使用物品 |
| EquipItem(iid, slot) | GameCommand::Inventory(EquipItem {..}) | 装备 |
| DropItem(iid) | GameCommand::Inventory(DropItem {..}) | 丢弃 |
| BuyItem(iid, qty) | GameCommand::Economy(BuyItem {..}) | 购买 |
| SellItem(iid, qty) | GameCommand::Economy(SellItem {..}) | 出售 |
| AcceptQuest(qid) | GameCommand::Quest(AcceptQuest {..}) | 接受任务 |
| SaveGame(slot) | GameCommand::Save(SaveGame {..}) | 保存 |
| LoadGame(slot) | GameCommand::Save(LoadGame {..}) | 加载 |
| ChangeSettings(settings) | 直接修改 UiSettings Resource（不走 GameCommand） | 设置 |
| OpenScreen(type) | 直接调用 ScreenStack::push（UI 内部） | 导航 |
| CloseScreen | 直接调用 ScreenStack::pop（UI 内部） | 导航 |

### 4.4 UiCommand 的 Save/Replay 策略

| 命令类别 | 策略 |
|---------|------|
| 战斗/背包/任务/经济命令 | 通过 GameCommand 进入 Replay 录制 |
| 存档/设置命令 | 不进入 Replay，独立处理 |
| UI 导航命令 | 不进入 Replay，Replay 时从 Domain Event 重新投影 |

---

## 5. UiEvent — UI 内部事件

### 5.1 定义

UiEvent 是 UI 层内部的事件广播机制，用于 UI 层内部各子系统通信（ViewModel 更新、导航变更、焦点切换等）。

```rust
#[derive(Clone, Reflect)]
pub enum UiEvent {
    // ── ViewModel 事件 ──
    /// ViewModel 已更新（设置 Dirty 标记后发射）
    ViewModelUpdated(&'static str),  // ViewModel 名称
    /// UiStore 已更新
    StoreUpdated,

    // ── 导航事件 ──
    /// Screen 推入完成
    ScreenPushed(ScreenType),
    /// Screen 弹出完成
    ScreenPopped(ScreenType),
    /// Screen 被替换
    ScreenReplaced(ScreenType, ScreenType),  // (old, new)
    /// 导航错误
    NavigationError(NavigationError),

    // ── Overlay 事件 ──
    /// Overlay 显示
    OverlayShown(&'static str),
    /// Overlay 关闭
    OverlayHidden(&'static str),

    // ── 焦点事件 ──
    /// 焦点移动到新元素
    FocusChanged(FocusId),
    /// 焦点组切换
    FocusGroupChanged(FocusGroupId),

    // ── 主题事件 ──
    /// 主题切换
    ThemeChanged(ThemeName),

    // ── 本地化事件 ──
    /// 语言切换
    LanguageChanged(LanguageVm),

    // ── 动画事件 ──
    /// 过渡动画开始
    TransitionStarted(ScreenType),
    /// 过渡动画结束
    TransitionFinished(ScreenType),
}
```

### 5.2 UiEvent 使用场景

| 事件 | 广播者 | 监听者 | 用途 |
|------|--------|--------|------|
| ViewModelUpdated | Projection | Widget System | 触发 Widget 刷新（与 Dirty 机制的冗余保障） |
| ScreenPushed | ScreenStack | FocusSystem | 新 Screen 激活 → FocusGroup 初始化 |
| ScreenPopped | ScreenStack | OverlayService | Screen 退出时清理与其绑定的 Overlay |
| OverlayShown | OverlayService | InputSystem | 阻止下层 FocusGroup 交互 |
| FocusChanged | FocusSystem | TooltipService | 新焦点元素 → 更新 Tooltip |
| ThemeChanged | ThemeSystem | Widget System | 触发全局 Widget 刷新 |
| LanguageChanged | SettingsSystem | LocalizedText | 触发所有 LocalizedText 刷新 |

### 5.3 UiEvent 边界

- UiEvent 是 UI 内部事件，U 不跨出 UI 层
- Domain Event 通过 Observer 通知 UI，不经过 UiEvent
- UiEvent 的监听者限于 UI 层内部
- UiEvent 使用 Bevy Event 机制（`EventWriter`/`EventReader`）在 UI 层内部广播

---

## 6. Intent → Command → GameCommand 完整映射链

### 6.1 战斗场景示例：施放技能

```
用户点击技能图标
    │
    ▼
Widget (SkillSlot)
    │  Input: SkillSlotVm { skill_id: SkillId(42), is_usable: true, ... }
    │  Interaction: MouseButton::Left
    ▼
UiAction::SelectSkill(SkillId(42))
    │  Screen (BattleScreen) 的 UiActionHandler 处理
    ▼
UiCommand::CastSkill(SkillId(42), CharacterId(7))
    │  Application layer (command.rs) 转换
    ▼
GameCommand::Ability(CastAbility {
    skill_id: SkillId(42),
    caster_id: CharacterId(1),
    target_id: CharacterId(7),
})
    │  ADR-043 CommandQueue
    ▼
Ability Domain 执行
    │
    ▼
Domain Event: DamageApplied { target: CharacterId(7), value: 85 }
    │  Observer 监听
    ▼
BattleProjection::project_damage → BattleHudVm.hp = 85 → Dirty
```

### 6.2 背包场景示例：丢弃物品

```
用户点击"丢弃"按钮
    │
    ▼
Widget (DangerButton)
    │  Input: label_key: "ui.inventory.drop", enabled: true
    │  Interaction: Click
    ▼
UiAction::Click
    │
    ▼
UIAction → ModalService.push(ModalVm {
    title_key: "ui.inventory.drop_confirm",
    body_key: "ui.inventory.drop_confirm_body",
    buttons: [Cancel, Confirm],
    ...
})
    │  用户点击确认
    ▼
UiAction::Confirm
    │
    ▼
UiCommand::DropItem(ItemId(15))
    │  command.rs 转换
    ▼
GameCommand::Inventory(DropItem { item_id: ItemId(15) })
    │
    ▼
Inventory Domain 执行
```

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
    │  Intent 路由 → FocusGroup
    ▼
FocusGroup（BattleScreen.FocusGroup, FocusNavigation::Grid { cols: 4 }）
    │  计算焦点移动：当前焦点位置 (row=1, col=0) → (row=0, col=0)
    │  wrap = false，已达首行 → 无移动
    ▼
（无 UiAction/Command 发射，焦点可能已在边界）
```

---

## 7. 通信路径汇总

| 路径 | 机制 | 方向 | 涉及的枚举 |
|------|------|------|-----------|
| Input → Intent | InputSystem 映射 | Infra/Input → UI | InputAction → UiIntent |
| Intent → Action | FocusGroup 路由 | UI 内部 | UiIntent → UiAction |
| Action → Command | UiActionHandler 转换 | UI 内部 | UiAction → UiCommand |
| Command → GameCommand | command.rs 转换器 | UI → Core/Domain | UiCommand → GameCommand |
| Domain → ViewModel | Observer + Projection | Core → UI | Domain Event → ViewModel |
| ViewModel → Widget | Dirty<T> consume | UI 内部 | — |
| UI 内部广播 | UiEvent (EventWriter/Reader) | UI 内部 | UiEvent |

---

## 8. 验证规则

| # | 规则 | 校验逻辑 |
|---|------|----------|
| APP-VAL-01 | UiIntent 不包含硬件细节 | UiIntent 变体不引用 KeyCode/GamepadButton 等硬件类型 |
| APP-VAL-02 | UiCommand 不包含执行逻辑 | UiCommand 是纯数据枚举，匹配分支中无业务逻辑 |
| APP-VAL-03 | 转换器是唯一出口 | UI → Domain 的调用必须经过 UiCommand → GameCommand 转换器 |
| APP-VAL-04 | UiEvent 不跨 UI 层 | UiEvent 的事件订阅者仅限于 UI 层内部模块 |
| APP-VAL-05 | Intent 无法映射时静默忽略 | 无对应映射的 Intent 不产生崩溃 |

---

*本文档由 @presentation-architect 维护。新增 UiIntent/UiCommand/UiEvent 变体需经过架构审查。*
