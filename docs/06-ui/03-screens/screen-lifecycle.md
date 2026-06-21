---
id: 06-ui.screen-lifecycle
title: Screen and Widget Lifecycle — 页面生命周期与组件架构
status: code-aligned
owner: presentation-architect
created: 2026-06-20
tags:
  - ui
  - screen
  - widget
  - lifecycle
  - state-machine
---

# Screen and Widget Architecture — 页面与组件架构

> **职责**: @presentation-architect | **上游**: domain rules §1-§2, §8, §INV-UI-005, §INV-UI-009 | ADR-055 §5.4-§5.5, §8, §9, §12

---

## 1. 设计目的

Screen 和 Widget 是 UI 层的两个核心构造。Screen 是页面级容器，组合 Widget；Widget 是独立可复用的 UI 组件。50 万行代码下，如果没有清晰的 Screen/Widget 分层和生命周期管理，UI 代码将不可避免退化为 Node 堆砌。

本文档定义 Screen 和 Widget 的完整架构：生命周期状态机、组合规则、Contract 模式、目录层级约定。

---

## 2. Screen 架构

### 2.1 Screen 定义

Screen 是**页面级 Widget 容器**，与 `GameState`（ADR-050）对应。一个 Screen 对应一个完整的"页面"，负责 Widget 的组合与布局协调，不负责 Widget 内部的渲染逻辑。

| 职责 | 不负责 |
|------|--------|
| 页面级布局与 Widget 组合 | Widget 内部渲染逻辑 |
| Screen 生命周期管理 | 业务数据获取 |
| 页面级交互协调 | 单个 UI 元素的具体交互 |
| FocusGroup 初始化 | 焦点导航策略细节 |

（引用：domain rules §1 — 统一术语，Screen 定义）

### 2.2 Screen 生命周期状态机

```
Defined（已定义——在 Theme/Content 中配置）
   │  [ScreenStack::push]
   ▼
Loading（加载中——初始化 ViewModel、加载资源）
   │  [资源就绪 + ViewModel 初始化完成]
   ▼
Active（活跃——可见可交互）
   │  [ScreenStack::push(新 Screen)]
   ▼
Background（后台——被新 Screen 遮挡，不可交互）
   │  [上层 Screen pop]
   ▼
Active（恢复活跃）
   │  [ScreenStack::pop 或 replace]
   ▼
Unloading（卸载中——清理资源、注销 Observer）
   │  [清理完毕]
   ▼
Destroyed（已销毁）
```

### 2.3 状态转换规则

| 转换 | 触发条件 | 动作 |
|------|---------|------|
| Defined → Loading | ScreenStack::push | 初始化 ViewModel，注册 Observer，加载资源 |
| Loading → Active | 资源就绪 + ViewModel 初始化完成 | 显示 Screen，激活 FocusGroup |
| Active → Background | 新 Screen push 到栈顶 | 暂停交互，保留 ViewModel |
| Background → Active | 上层 Screen pop | 恢复交互，刷新 ViewModel（如 Dirty） |
| Active → Unloading | ScreenStack::pop/replace | 注销 Observer，清理定时器 |
| Background → Unloading | Screen 被 replace 或强制移除 | 注销 Observer，清理定时器 |
| Unloading → Destroyed | 清理完毕 | 移除所有 Entity |
| 🟥 禁止 | Loading → Background | 未完成加载的 Screen 不可被遮挡 |
| 🟥 禁止 | Destroyed → 任何状态 | Screen 销毁后不可复活，需重新 push |

（引用：domain rules §2.1 — Screen 生命周期状态机）

### 2.4 Screen 组合 Widget 的规则

**规则 R-SCR-01：Screen 只做组合，不直接拼 Node**

```pseudocode
// ✅ 正确：Screen 通过 Factory 函数组合 Widget
fn spawn_battle_screen(commands, vm_store):
    screen_root = commands.spawn((
        BattleScreen,
        Node { fullscreen },
    ))
    // 通过 WidgetFactory 创建子 Widget 并挂到 Screen Root 下
    commands.entity(screen_root).add_children(&[
        TopBar::create(commands, &vm_store.top_bar),
        TurnBar::create(commands, &vm_store.turn_order),
        CharacterPanel::create(commands, &vm_store.characters),
        SkillPanel::create(commands, &vm_store.skills),
        ActionMenu::create(commands, &vm_store.actions),
    ])

// ❌ 禁止：Screen 中直接写 Node/BackgroundColor/Interaction
fn battle_screen(mut commands: Commands) {
    commands.spawn((
        Node { .. },
        BackgroundColor(Color::BLACK),
    ))
}
```

**规则 R-SCR-02：Screen 使用 WidgetFactory 组合 Widget**

```pseudocode
// ✅ 正确：所有 Widget 通过 WidgetFactory trait 组合
// WidgetFactory::create 接受 Commands + ViewModel，返回 Entity
fn spawn_battle_screen(commands, vm_store):
    screen_root = commands.spawn(BattleScreen)
    turn_order_bar  = TurnOrderBar::create(commands, &vm_store.turn_order)
    battle_hud      = BattleHud::create(commands, &vm_store.battle_hud)
    character_panel = CharacterPanel::create(commands, &vm_store.characters)
    skill_panel     = SkillPanel::create(commands, &vm_store.skills)
    action_menu     = ActionMenu::create(commands, &vm_store.actions)
    commands.entity(screen_root).add_children(&[
        turn_order_bar, battle_hud, character_panel,
        skill_panel, action_menu,
    ])
```

**规则 R-SCR-03：Screen 是 Navigation 的最小单元**

- `ScreenStack::push` / `ScreenStack::pop` / `ScreenStack::replace` 以 Screen 为单位
- 不能只导航 Screen 内的某个 Widget

（引用：ADR-055 §5.5 — Screen 组合 Widget；ADR-055 §12 — WidgetFactory trait）

---

## 3. Widget 架构

### 3.1 Widget 定义

Widget 是**可复用 UI 组件**，每个 Widget 是独立的 Plugin，有自己的渲染逻辑和 Contract。

| 职责 | 不负责 |
|------|--------|
| 单一 UI 元素的渲染与交互 | 业务数据获取 |
| 通过 ViewModel 消费 UI 数据 | 直接 Query Domain 组件 |
| 声明输入（Props）、输出（Events）、本地状态 | 业务逻辑计算 |

（引用：domain rules §1 — 统一术语，Widget 定义）

### 3.2 Widget 生命周期

```
Spawned（已创建——Entity 已 spawn）
   │  [WidgetContract 输入数据就绪]
   ▼
Mounted（已挂载——首次渲染完成）
   │  [Dirty<T> 标记为 true]
   ▼
Updating（更新中——重新渲染）
   │  [渲染完成，Dirty<T> 清除]
   ▼
Mounted（回到已挂载）
   │  [Visibility::Hidden 或 Persistent 模式]
   ▼
Hidden（隐藏——不可见但 Entity 存在）
   │  [Visibility::Visible]
   ▼
Mounted（恢复可见）
   │  [despawn 或 Ephemeral 模式销毁]
   ▼
Despawned（已销毁）
```

### 3.3 Widget 生命周期模式

| 模式 | 隐藏方式 | 销毁时机 | 适用场景 |
|------|---------|---------|---------|
| **Persistent** | Visibility 切换 | Screen 销毁时 | 频繁开关的面板（技能栏、状态栏、战斗 HUD） |
| **Ephemeral** | 直接 despawn | 关闭时立即 despawn | 一次性弹窗、确认框 |

**Persistent 模式原则**：频繁开关的 Widget 不销毁实体，通过 `Visibility` 切换显示状态。适用于战斗 HUD（HP/MP 条、回合指示器）、技能面板、小地图。不适用于全屏页面（MainMenu、Inventory），这些使用 spawn/despawn。

（引用：ADR-055 §8 — Persistent Widget 模式；domain rules §2.2 — Widget 生命周期规则）

### 3.4 Widget Contract 模式

每个 Widget 必须有明确的契约声明：输入数据（Props）、输出动作（Events）、禁止行为（Prohibited）。

#### 3.4.1 Contract 定义模板

```
/// Widget 名称
/// Input:   XxxVm（ViewModel 类型）
/// Output:  UiAction::Xxx, UiAction::Yyy
/// State:   （本地状态，如选中项、悬停状态）
/// Prohibited: Query<&...>, EventReader, 硬编码颜色
```

#### 3.4.2 Widget Contract 清单

| Widget | Input | Output | Prohibited |
|--------|-------|--------|------------|
| PrimaryButton | UiTextKey | UiAction::Click | Query, EventReader |
| ProgressBar | f32 (ratio) | — | Query |
| SkillPanel | SkillPanelVm | UiAction::SelectSkill, UiAction::CastSkill | Query<&Ability> |
| CharacterPanel | CharacterPanelVm | UiAction::SelectCharacter | Query<&Health> |
| InventoryGrid | InventoryVm | UiAction::SelectItem, UiAction::UseItem | Query<&Item> |
| TurnBar | BattleHudVm | — | Query<&TurnState> |
| Tooltip | TooltipVm | — | Query |
| Modal | ModalVm | UiAction::Confirm, UiAction::Cancel | Query |
| Notification | NotificationVm | — | Query |
| LocalizedText | UiTextKey | — | 硬编码字符串 |

（引用：domain rules §8 — Widget Contract 清单）

#### 3.4.3 UI Schema 治理

Widget Contract 的机器可读版本存放在 `docs/ui_schema/`：

```
docs/ui_schema/
├── screens/               # Screen Schema
├── widgets/               # Widget Schema
├── view_models/           # ViewModel Schema
└── contracts/             # Contract 模板
```

每个新增 Widget 必须先写 Schema，再写代码。AI 生成 UI 代码时必须遵守 Schema。

Schema 示例：
```yaml
# docs/ui_schema/widgets/skill_panel.yaml
widget: SkillPanel
input:
  type: SkillPanelVm
  fields:
    - skills: Vec<SkillSlotVm>
    - selected: Option<SkillId>
    - ap_remaining: u32
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

（引用：ADR-055 §9 — UI Schema 治理；domain rules §5.8 — UI Schema 治理流程；schema §24 — UI Schema 治理）

---

## 4. Screen ↔ GameState 映射表

### 4.1 GameState → Screen 映射

| GameState | UI Screen | ScreenLayer 内容 |
|-----------|-----------|-----------------|
| MainMenu | MainMenuScreen | 标题画面、开始/继续/设置按钮 |
| PartySetup | PartySetupScreen | 队伍编成、角色选择 |
| TacticalMap | TacticalMapScreen | 地图、小地图、队伍信息 |
| Combat | BattleScreen | 战斗 HUD、技能面板、回合条 |
| Result | ResultScreen | 战斗结算、奖励展示 |
| CampRest | CampRestScreen | 营地界面、休息选项 |
| GameOver | GameOverScreen | 游戏结束画面 |

### 4.2 OverlayState → Overlay 映射

| OverlayState | UI Overlay | PopupLayer 内容 |
|-------------|-----------|----------------|
| Dialogue | DialogueOverlay | 对话框、选项 |
| Shop | ShopScreen | 商店界面 |
| Cutscene | CutsceneOverlay | 过场演出 |
| Tutorial | TutorialOverlay | 新手指引 |

### 4.3 映射规则

- `OnEnter(GameState::X)` → spawn 对应 Screen
- `OnExit(GameState::X)` → despawn 对应 Screen（仅清理 ScreenLayer）
- `PushOverlay(OverlayState::X)` → spawn 对应 Overlay（PopupLayer）
- `PopOverlay` → despawn Overlay（仅清理 PopupLayer）
- 切换 GameState 时，先 OnExit 当前状态再 OnEnter 新状态

（引用：ADR-055 §9 — 与 ADR-050 GameState 的 Screen 映射；ADR-055 §6 — UI Root 分层）

---

## 5. Screen 组合 Widget 的目录层级约定

### 5.1 目录结构

```
screens/
├── battle/               # BattleScreen
│   ├── mod.rs            # BattleScreen Plugin + bsn! 组合
│   ├── tests/            # Screen 级测试
│   └── components.rs     # Screen 级私有组件
├── menu/
│   ├── mod.rs
│   └── tests/
├── inventory/
│   ├── mod.rs
│   └── tests/
└── ...
```

### 5.2 目录约定

| 约定 | 说明 |
|------|------|
| 每个 Screen 一个目录 | 避免 `screens.rs` 单文件膨胀 |
| Screen 不直接引用另一个 Screen | Screen 间通信通过 ScreenStack（push/pop/replace） |
| Screen 可以引用多个 Widget | Widget 是 Screen 的构建块 |
| Screen 的私有组件放在 `components.rs` | 不暴露到 `mod.rs` 外部 |

### 5.3 Widget 目录结构

```
widgets/
├── button/               # Button 系列 Widget
│   ├── mod.rs            # Widget Plugin 注册
│   ├── primary.rs        # PrimaryButton
│   ├── secondary.rs      # SecondaryButton
│   ├── danger.rs         # DangerButton
│   └── tests/
├── progress_bar/
│   ├── mod.rs
│   └── tests/
├── tooltip/
│   ├── mod.rs
│   ├── service.rs        # TooltipService（单例）
│   └── tests/
├── panel/
│   ├── mod.rs
│   ├── card.rs
│   └── tests/
├── list/
│   ├── mod.rs
│   ├── virtual_list.rs
│   └── tests/
├── text/
│   ├── mod.rs
│   ├── localized_text.rs # LocalizedText Widget
│   └── tests/
├── modal/
│   ├── mod.rs
│   ├── service.rs        # ModalService
│   └── tests/
└── notification/
    ├── mod.rs
    ├── service.rs        # NotificationService
    └── tests/
```

### 5.4 组合规则汇总

| 规则 | 描述 | 违反后果 |
|------|------|---------|
| Screen 组合 Widget | Screen 只做 Widget 组合，不拼 Node | Screen 代码膨胀，Widget 不可复用 |
| Widget 不组合 Screen | Widget 不感知 Screen 的存在 | 违反单向依赖 |
| Screen 不引用 Screen | Screen 间通过 Stack 通信 | 硬编码导航逻辑 |
| Widget 内部布局自由 | Widget 的 Node/布局对外不可见 | 封装被破坏，样式散落 |

---

## 6. Screen 不变量的交叉引用

| 不变量 | 关联文件 | 说明 |
|--------|---------|------|
| INV-UI-001 | Architecture §6.1 + 本文 §3.1 | Widget 不直接 Query Domain |
| INV-UI-002 | Architecture §6.2 + 本文 §3.1 | Widget 不持有 Entity |
| INV-UI-005 | 本文 §2.4 R-SCR-01 | Screen 不直接拼 Node |
| INV-UI-006 | navigation-overlay.md | Overlay 独立于 Screen |

---

*本文档由 @presentation-architect 维护。新增 Screen/Widget 必须先过 Contract 审查。*
