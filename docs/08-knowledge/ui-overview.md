# UI 表现层深度解析

> 从架构铁律到像素渲染：Fre 项目的 UI 是如何设计、分层和实现的

---

## 目录

1. [UI 层在整个架构中的位置](#1-ui-层在整个架构中的位置)
2. [五条铁律——UI 架构的非协商条款](#2-五条铁律ui-架构的非协商条款)
3. [三层渲染栈：Theme → Primitives → Widgets → Screens](#3-三层渲染栈theme--primitives--widgets--screens)
4. [Theme 主题令牌系统——没有硬编码的颜色](#4-theme-主题令牌系统没有硬编码的颜色)
5. [Primitives 层——Bevy UI 的唯一桥梁](#5-primitives-层bevy-ui-的唯一桥梁)
6. [Widgets 层——组合原语成业务控件](#6-widgets-层组合原语成业务控件)
7. [Screens 层——全屏页面组装](#7-screens-层全屏页面组装)
8. [Plugin 注册链——谁在什么时候启动](#8-plugin-注册链谁在什么时候启动)
9. [Observer 事件路由——按钮点击是怎么传到业务逻辑的](#9-observer-事件路由按钮点击是怎么传到业务逻辑的)
10. [工厂模式——为什么 UI 元素只有一种创建方式](#10-工厂模式为什么-ui-元素只有一种创建方式)
11. [完整的数据流架构——设计中的四层管道](#11-完整的数据流架构设计中的四层管道)
12. [当前实现状态一览](#12-当前实现状态一览)
13. [全部相关文件索引](#13-全部相关文件索引)

---

## 1. UI 层在整个架构中的位置

Fre 项目的架构是纵向三层 + 横切四层的 DDD 结构。UI 层（在架构文档中称为 **L3 Presentation Layer**）位于最顶层：

```
┌──────────────────────────────────────────────────┐
│   L3: UI / Presentation（表现层）                 │
│   依赖 L1 Core 和 L2 Infra，但不被下层依赖         │
│   目录: src/ui/                                   │
├──────────────────────────────────────────────────┤
│   L2: Infrastructure（基础设施层）                 │
│   渲染、持久化、输入、寻路、本地化                  │
├──────────────────────────────────────────────────┤
│   L1: Core（领域核心层）                           │
│   15 Capabilities（能力机制）+ 15 Domains（业务域） │
├──────────────────────────────────────────────────┤
│   L0: Shared（共享层）                             │
│   强类型 ID、数学工具、确定性 RNG                   │
└──────────────────────────────────────────────────┘
```

UI 层是一个**纯消费者**——Domain/Core/Infra 层绝对不能依赖 UI 代码。这意味着任何 UI 相关的东西都不能被下层 import 或引用。这是 UI 架构的底层约束，一切设计都从这条规则展开。

---

## 2. 五条铁律——UI 架构的非协商条款

这五条规则出现在每一份 UI 架构文档中，是"宪法级的约束"：

| # | 规则 | 违反后果 |
|---|------|---------|
| 1 | **Domain 不能依赖 UI** — 领域层代码不能 import 任何 UI 类型 | 编译期模块边界检查 |
| 2 | **UI 不能直接查询 Domain** — 不能 `Query<&Health>`，只能通过 ViewModel 间接读取 | 架构审查不通过 |
| 3 | **Screen 组合 Widget，Screen 不造裸 Node** — 页面层必须用工厂函数创建控件，不能直接 `spawn(Node { ... })` | 架构审查不通过 |
| 4 | **颜色/字体/间距必须用 StyleToken** — 禁止 `Color::srgb(0.1, 0.1, 0.14)` 出现在 theme 文件之外 | 架构审查不通过 |
| 5 | **Primitives 是唯一的 Bevy UI 桥梁** — Widgets 和 Screens 不能 import `Node`、`Button`、`Interaction` 等 Bevy UI 类型 | 编译期模块导出控制 |

这些规则的共同目标：**防止"50 万行退化"**——随着代码规模增长，UI 代码会不可控地直接操作 Domain 数据、硬编码样式、架构边界模糊，最终导致整个项目不可维护。

---

## 3. 三层渲染栈：Theme → Primitives → Widgets → Screens

UI 代码的组织方式是一个**严格的四层金字塔**：

```
Screens（全屏页面）
    ↑ 组合
Widgets（业务复合控件）
    ↑ 组合
Primitives（原子控件）
    ↑ 读取
Theme（设计令牌）
```

```
       ┌─────────────────┐
       │    Screens      │  2 screens: Battle, MainMenu
       │   (全屏页面)     │
       └────────┬────────┘
                │ 组合
       ┌────────▼────────┐
       │    Widgets      │  5 widgets: ActionMenu, CharacterCard,
       │  (业务复合控件)   │  SkillSlot, BuffIcon, InventoryItemRow
       └────────┬────────┘
                │ 组合
       ┌────────▼────────┐
       │   Primitives    │  6 primitives: Button, Panel, Text,
       │   (原子控件)     │  ProgressBar, List, Modal
       └────────┬────────┘
                │ 读取
       ┌────────▼────────┐
       │     Theme       │  4 files: Colors, Spacing, Typography, Resource
       │  (设计令牌)      │
       └─────────────────┘
```

依赖方向严格向下。每个层级**只能依赖它的直接下层和 Theme**——Screens 不能直接 import Primitives 的 `Node` 布局配置（它应该只用 Widgets 的工厂函数）。

---

## 4. Theme 主题令牌系统——没有硬编码的颜色

Theme 系统是整个 UI 的"设计语言基础"——所有视觉属性从这里流出，没有任何地方硬编码颜色值。

### Theme Resource

```rust
// src/ui/theme/resource.rs
#[derive(Resource, Reflect)]
pub struct Theme {
    pub name: &'static str,
    pub colors: UiColors,
    pub spacing: UiSpacing,
    pub typography: UiTypography,
}
```

这是一个全局 **ECS Resource**，在 Phase 1 初始化。任何 UI 元素创建时都要引用它。

### 语义颜色：UiColors

```rust
// src/ui/theme/colors.rs — 约 30 个语义令牌
pub struct UiColors {
    // 面板表面（9 个）
    pub surface_primary: Color,   // 主背景：深色 0.11, 浅色 0.97
    pub surface_secondary: Color, // 次要背景
    pub surface_danger: Color,    // 危险操作背景
    pub surface_disabled: Color,  // 禁用态背景
    // ... 还有 hover/pressed 变体

    // 文字色（4 个）
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    pub text_accent: Color,

    // 交互色（3 个）
    pub accent_primary: Color,    // 主要按钮
    pub accent_hover: Color,
    pub accent_pressed: Color,

    // 反馈色（3 个）
    pub feedback_positive: Color, // 绿色 - 治疗/成功
    pub feedback_negative: Color, // 红色 - 伤害/错误
    pub feedback_warning: Color,  // 橙色 - 警告

    // 边框色（2 个）
    pub border_default: Color,
    pub border_focus: Color,
}
```

UiColors 提供 `dark()` 和 `light()` 两个工厂方法，分别返回完整的深色/浅色色板。

**关键约束**：任何非 theme 文件中出现 `Color::srgb(...)` 都是违规。所有颜色引用必须是 `theme.colors.text_primary` 这样的语义路径。

### 间距令牌：UiSpacing

```rust
// src/ui/theme/spacing.rs
pub struct UiSpacing {
    pub xs: f32,      // 4px
    pub sm: f32,      // 8px
    pub md: f32,      // 16px
    pub lg: f32,      // 24px
    pub xl: f32,      // 32px
    pub xxl: f32,     // 48px
    pub border_radius_sm: f32,
    pub border_radius_md: f32,
    pub border_radius_lg: f32,
    pub icon_size: f32,
    pub button_height: f32,
    pub min_touch_target: f32,  // 44px 无障碍最小触摸区域
}
```

### 字体令牌：UiTypography

```rust
// src/ui/theme/typography.rs
pub struct UiTypography {
    // 字体文件路径（assets/fonts/）
    pub font_body: String,      // FiraSans
    pub font_heading: String,   // FiraSans
    pub font_mono: String,      // FiraCode
    // 字号（8 个级别）
    pub size_caption: f32,      // 12px
    pub size_body: f32,         // 14px
    pub size_label: f32,        // 14px
    pub size_heading: f32,      // 18px
    pub size_title: f32,        // 24px
    pub size_display: f32,      // 36px
}
```

---

## 5. Primitives 层——Bevy UI 的唯一桥梁

Primitives 是项目中**唯一被允许 import Bevy UI 原生类型**（`Node`、`Button`、`Interaction`、`BackgroundColor`）的层。它提供 6 个原子控件，每个控件都遵守统一的内部结构：

```
primitives/{widget_name}/
├── mod.rs         → Plugin + 公开 re-export
├── components.rs  → Props 和 State 类型定义
├── factory.rs     → 唯一的创建入口（spawn_* 函数）
├── systems.rs     → 每帧更新逻辑（可选）
├── events.rs      → 交互事件定义（可选）
└── tests/         → 单元测试（可选）
```

### Button——最完整的 Primitive 示例

Button 是项目中被设计得最完整的 UI 原语，让我们拆开看：

**components.rs — 状态类型定义**

```rust
// 按钮样式变体
pub enum ButtonVariant { Primary, Secondary, Danger, Ghost }

// 按钮的本地状态（Widget Contract 的"Props"部分）
pub struct ButtonState {
    pub variant: ButtonVariant,
    pub disabled: bool,
    pub label: String,
}

// 按钮的交互状态（由 system 每帧更新）
pub struct ButtonInteraction {
    pub hovered: bool,
    pub pressed: bool,
    pub just_clicked: bool,  // 点击释放后持续一帧
}
```

**events.rs — 交互事件**

```rust
// 使用 Bevy 0.19 的 Event + Observer 模式
#[derive(Event)]
pub struct ButtonClicked {
    pub entity: Entity,
}
```

**factory.rs — 唯一的创建入口**

```rust
pub fn spawn_button(
    commands: &mut Commands,
    theme: &Theme,
    label: impl Into<String>,
    variant: ButtonVariant,
) -> Entity {
    // 从 Theme 令牌读取颜色（禁止硬编码）
    let bg_color = match (variant, interaction) {
        (Primary, pressed) => theme.colors.accent_pressed,
        (Primary, hovered) => theme.colors.accent_hover,
        (Primary, _)       => theme.colors.accent_primary,
        // ...
    };

    commands.spawn((
        Node { /* flex 居中, padding/margin 来自 theme.spacing */ },
        Button,
        BackgroundColor(bg_color),
        BorderColor(border),
        ButtonState { variant, disabled: false, label },
        ButtonInteraction::default(),
        Name::new(format!("Button({})", label)),
    )).with_children(|parent| {
        parent.spawn((Text::new(label), TextFont { /* theme.typography */ }));
    }).id()
}
```

**systems.rs — 每帧交互处理**

`button_interaction_system` 每帧查询所有 `(Entity, &Interaction, &ButtonState, &mut ButtonInteraction, &mut BackgroundColor)`：

1. 读取 Bevy 内置的 `Interaction` 组件（Hovered/Pressed/None）
2. 映射到自定义的 `ButtonInteraction`（hovered/pressed/just_clicked）
3. 检测到点击释放时，通过 `commands.trigger(ButtonClicked { entity })` 发射事件
4. 根据 variant + interaction state 更新背景色

### 其他 Primitives

| Primitive | 行数 | 特点 |
|-----------|------|------|
| **Panel** | 198 | 被动容器（不驱动任何 system），6 种变体（Basic/Card/Modal/Tooltip/List/Group），是绝大部分 UI 的根容器 |
| **Text** | 186 | 5 种变体（Body/Heading/Title/Caption/Label/Mono），支持 `Changed<TextWidget>` 驱动的自动更新 |
| **ProgressBar** | 279 | 4 种变体（Hp/Mp/Xp/Generic），颜色编码（绿/蓝/金/generic），`progress_bar_update_system` 每帧同步填充宽度和标签文本 |
| **List** | 135 | 3 种变体（Vertical/Horizontal/Virtual），纯容器不驱动 system |
| **Modal** | 467 | 3 种变体（Alert/Confirm/Custom），完整的浮层管理：半透明遮罩 + 卡片 + 按钮行 + Observer 路由 |

---

## 6. Widgets 层——组合原语成业务控件

Widgets 层在 Primitives 之上构建有业务含义的复合控件。每个 Widget 组合多个 Primitive 工厂调用，加上业务状态和事件路由。

### CharacterCard——Widget 的典型结构

```rust
pub fn spawn_character_card(
    commands: &mut Commands,
    asset_server: &AssetServer,
    theme: &Theme,
    name: impl Into<String>,
    level: u32,
    hp_current: f32, hp_max: f32,
    mp_current: f32, mp_max: f32,
) -> Entity {
    // 1. 容器 Panel
    let container = spawn_panel(commands, theme, PanelVariant::Card);
    commands.entity(container).insert(CharacterCardState { name, level, hp_current, hp_max, mp_current, mp_max });

    // 2-5. 子控件全部通过 Primitives 工厂创建
    spawn_text(commands, asset_server, theme, name, TextVariant::Caption);
    spawn_text(commands, asset_server, theme, level_str, TextVariant::Caption);
    spawn_progress_bar(commands, theme, ProgressBarVariant::Hp, hp_current, hp_max, true, ...);
    spawn_progress_bar(commands, theme, ProgressBarVariant::Mp, mp_current, mp_max, true, ...);

    // 6. 动作按钮（挂载 CharacterAction 标记组件用于 Observer 路由）
    let attack_btn = spawn_button(commands, theme, "Attack", ButtonVariant::Primary);
    commands.entity(attack_btn).insert(CharacterAction::Attack);

    container
}
```

产生的 UI 树：

```
Panel (Card)
  ├── Text ("Aria", Caption)
  ├── Text ("Lv.5", Caption)
  ├── ProgressBar (Hp, green, "80/100")
  ├── ProgressBar (Mp, blue, "40/50")
  ├── Button ("Attack", Primary)  — CharacterAction::Attack
  ├── Button ("Defend", Secondary) — CharacterAction::Defend
  └── Button ("Skill", Primary)   — CharacterAction::Skill
```

### 5 个 Widget

| Widget | 行数 | 组合的原语 | 关键特性 |
|--------|------|-----------|---------|
| ActionMenu | 214 | List + 5x Button | 战斗单位行动面板（攻击/防御/技能/道具/待机），每种 ActionType 挂载在按钮上 |
| CharacterCard | 269 | Panel + 2x Text + 2x ProgressBar + 3x Button | HP/MP 进度条 + 角色名 + 等级 + 动作按钮 |
| SkillSlot | 209 | Panel + Text + ProgressBar + Button | 技能冷却状态可视化，冷却中禁用按钮 |
| BuffIcon | 199 | Panel + Text + ProgressBar | 绿色/红色边框区分增益/减益，剩余回合数进度条 |
| InventoryItemRow | 216 | Panel + 2x Text + Button | 物品名 + 数量 + 使用按钮 |

每个 Widget 都有一个每帧状态的 system，当它的 State Component 变化时，同步更新子控件的状态（例如 CharacterCard 的 HP 变化 → 更新 child ProgressBar 的 current/maximum）。

---

## 7. Screens 层——全屏页面组装

Screens 是 UI 树的最顶层：组合 Widget 和 Primitives 形成完整页面。当前实现了 2 个屏幕。

### MainMenuScreen——~180 行

```
Panel (Basic, fullscreen centered)
  ├── Text ("Fre", Title, 48px)           ← 标题
  ├── Text ("A Bevy SRPG", Caption)        ← 副标题
  ├── List (Vertical, 200px)               ← 菜单列表
  │   ├── Button ("New Game", Primary)     ← MenuAction::NewGame
  │   ├── Button ("Load Game", Secondary)  ← MenuAction::LoadGame
  │   └── Button ("Settings", Secondary)   ← MenuAction::Settings
  └── Text ("v0.1.0", Caption)             ← 版本号
```

**关键实现细节**：
- 使用 `set_parent_in_place` 而非嵌套 `with_children` 来构建层次——工厂函数返回 Entity ID，然后用 `commands.entity(child).set_parent_in_place(parent)` 组织层级
- `MenuAction` 枚举作为 Component 挂载在按钮上，Observer 通过查询此标记组件来识别点击了哪个按钮
- `MainMenuScreen` 标记组件用于未来场景管理的 despawn 清理

### BattleScreen——~150 行

```
Panel (Basic, fullscreen column)
  ├── Text ("Turn: 3  Phase: Player Turn", Body)  ← 回合信息
  ├── Panel (Basic, 300px battle area)              ← 战斗区占位
  ├── CharacterCard (Aria, Lv.5, HP/MP bars)       ← 角色卡片
  ├── ActionMenu (Attack/Defend/Skill/Item/Wait)    ← 行动菜单
  └── Button ("End Turn", Danger)                   ← BattleAction::EndTurn
```

`BattleAction::EndTurn` 按钮使用 Danger 变体（红色），挂载 `BattleAction` 标记组件。

---

## 8. Plugin 注册链——谁在什么时候启动

UI 插件链在 Phase 11 注册（Infra Phase 8 和 ScenePlugin Phase 9 之后）：

```rust
// src/ui/plugin.rs
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ThemePlugin);       // 1. 设计令牌
        app.add_plugins(PrimitivesPlugin);  // 2. 原子控件
        app.add_plugins(WidgetsPlugin);     // 3. 业务控件
        app.add_plugins(ScreenPlugin);      // 4. 全屏页面
    }
}
```

每个子插件的注册内容：

| Plugin | 注册了什么 |
|--------|-----------|
| ThemePlugin | `Theme` Resource 初始化 + 3 个 Reflect 类型注册 |
| PrimitivesPlugin | 6 个原语的 Plugin（ButtonPlugin/PanelPlugin 等），注册 Component Reflect，添加每帧 System |
| WidgetsPlugin | 5 个 Widget 的 Plugin，注册状态 Component + 更新 System |
| ScreenPlugin | 2 个屏幕的 startup system + Observer 注册 |

---

## 9. Observer 事件路由——按钮点击是怎么传到业务逻辑的

整个 UI 层使用 Bevy 0.19 的 `Event + Observer` 模式来处理交互，而不是旧的 `EventWriter/EventReader`：

```rust
// 1. Primitives 层发射基础事件
button_interaction_system 检测到点击释放
  → commands.trigger(ButtonClicked { entity })   // 通用事件

// 2. Screen 层注册 Observer 来接收
pub fn on_battle_button_clicked(
    on: On<ButtonClicked>,
    query: Query<&BattleAction>,
) {
    let Ok(action) = query.get(on.event().entity) else { return; };
    match action {
        BattleAction::EndTurn => info!("结束回合"),
    }
}

// 3. 在 ScreenPlugin 中注册 Observer
app.observe(on_battle_button_clicked);
```

**标记组件路由模式**：按钮不直接关联到特定处理函数。相反，每个按钮挂载一个标记组件（如 `MenuAction::NewGame`、`BattleAction::EndTurn`、`CharacterAction::Attack`）。Observer 通过查询实体上的这些标记来决定如何处理点击。这种方式解耦了按钮的创建和按钮的逻辑处理。

这种模式也适用于 Modal：

```rust
// Modal 的确认按钮挂载 ModalButtonRole::Confirm
// Modal 的取消按钮挂载 ModalButtonRole::Cancel
// modal_interaction_observer 沿 ChildOf 链向上找到 ModalState
// 然后触发 ModalConfirmed / ModalCancelled 事件并 despawn overlay
```

---

## 10. 工厂模式——为什么 UI 元素只有一种创建方式

整个 UI 层**不允许直接通过 `commands.spawn(Node { ... })` 创建 UI**。每个 UI 元素只有一条创建路径：`spawn_*` 工厂函数。

```
创建路径（唯一）：
  spawn_button(&mut commands, &theme, "确认", ButtonVariant::Primary) → Entity

禁止路径（架构违规）：
  commands.spawn((Node { ... }, Button, BackgroundColor(...)))  // ❌
```

这样设计的原因：

1. **Theme 强制注入** — 工厂函数内部读取 Theme，确保颜色/间距/字体全部来自令牌，不会遗漏
2. **Props 校验** — 工厂函数签名决定了什么参数是必需的，不会遗漏字段
3. **Component 一致性** — 每个按钮自动获得相同的 Component 集合（ButtonState + ButtonInteraction + Name）
4. **变更可控** — 如果未来需要给所有按钮加一个新 Component，改一个 factory 就够了

---

## 11. 完整的数据流架构——设计中的四层管道

前面说的都是"UI 怎么渲染"。但数据从 Domain 到 UI 是怎么流的？这里涉及到一套更宏大的设计，目前还**在架构文档中但没有实现代码**。

### 正向流：Domain → UI

```
Domain Event (如 CombatTurnStarted)
    │
    ▼
Observer 触发 Projection (纯函数)
    │
    ├── 读取 Query<&Health, &MaxHealth>
    ├── 读取 Res<DefRegistry>
    ├── 计算 → 生成 ViewModel 值
    │
    ▼
ViewModel 写入 UiStore (ECS Resource)
    │
    ├── BattleHudVm.hp = 80
    ├── BattleHudVm.max_hp = 100
    │   Dirty<T> 标记为脏
    │
    ▼
Widget Update System 发现 Dirty<T> = true
    │
    ├── consume() 消费脏标记
    ├── 读取 ViewModel → 更新子控件状态
    │
    ▼
Primitives Update System 读取状态变化
    ├── progress_bar_update_system
    ├── text_update_system
    │
    ▼
屏幕像素更新
```

这条链条的关键设计：

- **Projection = 纯函数** — 没有副作用、没有 I/O、没有随机数、不修改 Domain 数据。纯粹把 Domain Component 转成 ViewModel 结构体
- **ViewModel 存储在 UiStore** — 一个类似 Redux Store 的 ECS Resource，扁平字段结构，避免 HashMap
- **Dirty<T>** — 每次 Projection 写入时标记脏，Widget System 在下一帧消费。避免每帧全量比对
- **Widget 不直接查询 Domain** — 这点非常重要。Widget 只知道 ViewModel，不知道 Domain Component 的存在

### 反向流：UI → Domain

```
用户点击 "Cast Skill" 按钮
    │
    ▼
ButtonClicked 事件
    │
    ▼
UiIntent (语义意图：SelectSkill { skill_id })
    │  ── Intent 层抽象用户意图，无关输入设备
    ▼
UiAction (控件输出：SkillUsed { slot_index })
    │  ── Action 是 Widget 说"我这边发生了什么"
    ▼
UiCommand (UI → Domain 命令：CastSkill { skill_id, target })
    │  ── Command 是转化后的 Domain 语言
    ▼
command.rs 转换器（UI → Domain 的唯一合法出口）
    │
    ▼
GameCommand (Domain 命令)
    │
    ▼
CommandQueue → Domain System 执行
```

这条链条的关键设计：

- 每个步骤是独立类型，职责清晰
- `command.rs` 是**唯一合法出口**——UI 只能通过这个转换器向 Domain 发送命令
- UiCommand 不能包含执行逻辑——它是"什么"而不是"怎么做"
- UiIntent 与设备无关——同一 Intent 可以来自鼠标、键盘或手柄

### 当前实现状态

以上整个正反向数据流（Projection / ViewModel / UiStore / UiIntent / UiAction / UiCommand）**在代码中还没有实现**。目前代码中的 UI 是"直接传递"模式：

```
主线流程（当前实现）：
  按钮点击 → Observer → info!("some log")     // 只打日志，不做任何 Domain 操作

架构设计（待实现）：
  按钮点击 → UiIntent → UiAction → UiCommand → GameCommand → CommandQueue → Domain
```

当前阶段（Phase B→C 过渡）的原语和控件层完成度很高，但中间的数据管道还是"待建设"状态。

---

## 12. 当前实现状态一览

理解了完整设计后，我们看看实际代码中有什么、没什么。

### ✅ 已在 src/ui/ 中实现

| 模块 | 文件数 | 行数 | 细节 |
|------|--------|------|------|
| Theme | 5 | ~380 | Colors (142行) + Spacing + Typography + Resource + Plugin |
| Primitives | 26 | ~2,100 | Button(711+384测试) + Panel(198) + Text(186) + ProgressBar(279) + List(135) + Modal(467) |
| Widgets | 18 | ~1,100 | ActionMenu(214) + CharacterCard(269) + SkillSlot(209) + BuffIcon(199) + InventoryItemRow(216) |
| Screens | 5 | ~330 | MainMenu(~180) + Battle(~150) |
| Plugin | 2 | ~80 | UiPlugin + 模块根 |
| **总计** | **56** | **~4,000** | |

### ❌ 在架构文档中但尚未在代码中实现

| 模块 | 设计文件 | 目标路径 | 缺失内容 |
|------|---------|---------|---------|
| Application 层 | `application-layer.md` | `src/ui/application/` | UiIntent/UiAction/UiCommand/UiEvent 类型 + 路由 + command.rs 转换器 |
| Projections | `projection-viewmodel.md` | `src/ui/projections/` | 5 个 Projection 纯函数（battle/inventory/character/quest/economy） |
| ViewModels | `projection-viewmodel.md` | `src/ui/view_models/` | 8 个 ViewModel 结构体 + UiStore Resource |
| Navigation | `navigation-overlay.md` | `src/ui/navigation/` | ScreenStack + ScreenState |
| Overlays | `overlays.md` | `src/ui/overlay/` | 5 个 Overlay 类型 + 4 个 Service |
| Focus | `focus-binding.md` | `src/ui/focus/` | Focusable/FocusGroup 组件 + 导航 system |
| Binding | `focus-binding.md` | `src/ui/binding/` | Dirty<T> + UiBinding 枚举 |
| 缺失 Screen | `screens.md` | `src/ui/screens/` | Inventory/Shop/Settings/SaveLoad（当前只有 Battle + MainMenu） |
| 缺失 Widget | `widget-composites.md` | `src/ui/widgets/` | 许多复合控件尚不存在 |
| 测试基础设施 | `testing.md` | `src/ui/tests/` | MockProjection + TestFixtures + Snapshot 测试 |

### 状态总结

```
目前实现程度：
  Theme      ██████████ 100%
  Primitives ██████████ 100%  (6/6 原子控件完成)
  Widgets    ████░░░░░░  40%  (5/13 复合控件完成)
  Screens    ██░░░░░░░░  20%  (2/6 屏幕完成)
  Application░░░░░░░░░░   0%  (未实现)
  Projection ░░░░░░░░░░   0%  (未实现)
  ViewModel  ░░░░░░░░░░   0%  (未实现)
  Navigation ░░░░░░░░░░   0%  (未实现)
  Overlay    ░░░░░░░░░░   0%  (未实现)
  Focus      ░░░░░░░░░░   0%  (未实现)
  Binding    ░░░░░░░░░░   0%  (未实现)
```

这是一个典型的前期建设阶段：底层（Theme + Primitives）完整，中间层（Widgets + Screens）部分完成，上层（Application + Projection + ViewModel + Navigation）待建设。

---

## 13. 全部相关文件索引

### 架构文档（14 个文件，8,529 行）

```
docs/06-ui/
├── README.md                                          231 行 — UI 架构总纲
├── 01-architecture/
│   ├── architecture.md                                651 行 — L3 架构总纲、5 铁律、数据流
│   ├── application-layer.md                           544 行 — UiIntent/Action/Command/Event 映射链
│   └── implementation-patterns.md                   1,182 行 — Widget/Screen/ViewModel 的 Bevy ECS 骨架
├── 02-design-system/
│   ├── widget-atoms.md                                650 行 — 21 个原子组件契约（Props/Events/State）
│   ├── widget-composites.md                         1,091 行 — 16 个复合组件详细设计
│   ├── theme-localization.md                          385 行 — StyleToken、Theme、UiTextKey
│   └── focus-binding.md                               500 行 — 焦点导航与 Dirty<T> 数据绑定
├── 03-screens/
│   ├── screen-lifecycle.md                            384 行 — Screen/Widget 生命周期状态机
│   ├── screens.md                                     504 行 — 6 个 Screen 详细设计
│   ├── navigation-overlay.md                          383 行 — ScreenStack + 5 层 Overlay
│   └── overlays.md                                    484 行 — 6 个 Overlay 详细设计
├── 04-data-flow/
│   └── projection-viewmodel.md                        779 行 — Projection + ViewModel + UiStore
└── 05-testing/
    └── testing.md                                     865 行 — UI 测试策略（三层 + Mock + Fixtures）
```

### ADR（主要）

```
docs/01-architecture/40-cross-cutting/ADR-055-ui-presentation-architecture.md  843 行 — UI 架构决策（核心）
docs/01-architecture/40-cross-cutting/ADR-053-localization-architecture.md    ~350 行 — LocalizedText/UiTextKey
docs/01-architecture/40-cross-cutting/ADR-043-command-input.md                316 行 — UiCommand 桥接
docs/01-architecture/00-foundation/ADR-050-game-state-machine.md              ~300 行 — Screen ↔ GameState
docs/01-architecture/00-foundation/ADR-054-bevy-0-19-migration.md              — BSN/Trigger/Observer
docs/01-architecture/40-cross-cutting/ADR-042-save-persistence.md             ~240 行 — UiSettings
```

### 领域规则 + 数据 Schema

```
docs/02-domain/capabilities/ui-presentation.md               595 行 — UI 领域规则（不变量/状态机/禁止项）
docs/04-data/capabilities/ui-presentation-schema.md        1,393 行 — ViewModel/StyleToken/Focus/UiCommand Schema
docs/04-data/infrastructure/localization_schema.md          ~350 行 — LocalizedText + UiTextKey
```

### 代码实现

```
src/ui/
├── mod.rs                                         46 行 — 模块根 + 公开 re-export
├── plugin.rs                                      34 行 — UiPlugin 注册链
├── theme/
│   ├── mod.rs                                     34 行 — ThemePlugin
│   ├── resource.rs                                61 行 — Theme Resource（dark/light 工厂）
│   ├── colors.rs                                 142 行 — UiColors（30 个语义令牌）
│   ├── spacing.rs                                 71 行 — UiSpacing（11 个间距令牌）
│   └── typography.rs                              68 行 — UiTypography（7 个字体令牌）
├── primitives/
│   ├── button/ (8 文件)                          711 行 — Button + 384 行测试
│   ├── progress_bar/ (4 文件)                    279 行 — ProgressBar
│   ├── panel/ (3 文件)                           198 行 — Panel
│   ├── text/ (4 文件)                            186 行 — Text
│   ├── list/ (3 文件)                            135 行 — List
│   └── modal/ (5 文件)                           467 行 — Modal
├── widgets/
│   ├── action_menu/ (4 文件)                     214 行 — 行动菜单
│   ├── character_card/ (4 文件)                  269 行 — 角色卡片
│   ├── skill_slot/ (4 文件)                      209 行 — 技能槽
│   ├── buff_icon/ (4 文件)                       199 行 — Buff 图标
│   └── inventory_item_row/ (4 文件)              216 行 — 物品行
└── screens/
    ├── battle/ (2 文件)                          147 行 — 战斗屏幕
    └── main_menu/ (2 文件)                       180 行 — 主菜单

src/infra/localization/ui/  (3 文件)                     — LocalizedText 组件 + 渲染
```

---

> **本篇为 Knowledge Base 文档，目标是"理解 UI 架构是如何组织、为什么这样设计"。** 
> 如需正式的规范定义，见 `docs/06-ui/` 和 `ADR-055`。
> 如需代码实现细节，见 `src/ui/` 各模块文件。

---

## 14. System Chaining——Widget 状态是怎么传到像素的

整个 UI 层的状态更新不是"一次到位"的——它是通过**多个 System 接力**完成的。以 CharacterCard 的 HP 减少为例：

```
战斗系统减少 HP
    │
    ▼
CharacterCard's update system 检测到 CharacterCardState 变化
    │  (Changed<CharacterCardState>)
    │
    ├── 找到子实体中的 ProgressBar (Hp variant)
    ├── 更新其 ProgressBarState { current: 80, maximum: 100 }
    │
    ▼
progress_bar_update_system 检测到 ProgressBarState 变化
    │  (没有 Changed 过滤——每帧无条件执行)
    │
    ├── 读取 current (80) / maximum (100)
    ├── 计算 ratio = clamp(80/100, 0, 1) = 0.8
    ├── 找到子实体中的 ProgressBarFill 节点
    │   └── 更新其 Node.width = Val::Percent(80%)
    └── 找到子实体中的 ProgressBarLabel 文本
        └── 更新其 Text.0 = "HP 80/100"
```

这是 `progress_bar_update_system` 的实际代码：

```rust
pub fn progress_bar_update_system(
    bar_query: Query<(&ProgressBarState, &Children)>,
    mut fill_query: Query<(&mut Node, &ProgressBarFill)>,
    mut label_query: Query<(&mut Text, &ProgressBarLabel)>,
) {
    for (state, children) in &bar_query {
        // 计算填充比例
        let ratio = if state.maximum > 0.0 {
            (state.current / state.maximum).clamp(0.0, 1.0)
        } else {
            0.0
        };

        for child in children.iter() {
            // 更新填充条宽度
            if let Ok((mut node, _)) = fill_query.get_mut(child) {
                node.width = Val::Percent(ratio * 100.0);
            }
            // 更新标签文本
            if let Ok((mut text, _)) = label_query.get_mut(child) {
                let prefix = match state.variant {
                    ProgressBarVariant::Hp => "HP ",
                    ProgressBarVariant::Mp => "MP ",
                    ProgressBarVariant::Xp => "XP ",
                    ProgressBarVariant::Generic => "",
                };
                text.0 = format!("{}{:.0}/{}", prefix, state.current, state.maximum as u32);
            }
        }
    }
}
```

注意这里**没有** `Changed<ProgressBarState>` 过滤器——这个 system 每帧无条件运行，因为 `Node.width` 和 `Text.0` 的变化本身就是罕见的，查阅所有 `Children` 的成本远低于额外维护一套脏标记。

---

## 15. Button Interaction System——点击检测的完整机制

Button 的交互处理是整个 UI 中最复杂的 system，因为它需要把 Bevy 内置的低级 `Interaction` 组件转成更高级的语义事件。

```rust
pub fn button_interaction_system(
    theme: Res<Theme>,
    mut button_query: Query<(
        Entity, &Interaction, &ButtonState,
        &mut ButtonInteraction, &mut BackgroundColor,
    )>,
    mut commands: Commands,
) {
    for (entity, interaction, state, mut btn_interaction, mut bg_color) in &mut button_query {
        // 禁用态：固定背景色，跳过所有交互
        if state.disabled {
            *bg_color = BackgroundColor(theme.colors.surface_disabled);
            btn_interaction.hovered = false;
            btn_interaction.pressed = false;
            btn_interaction.just_clicked = false;
            continue;
        }

        // 追踪悬停和按压
        btn_interaction.hovered = *interaction == Interaction::Hovered;
        let was_pressed = btn_interaction.pressed;
        btn_interaction.pressed = *interaction == Interaction::Pressed;

        // 关键逻辑：pressed 变为 false 触发 just_clicked（持续一帧）
        btn_interaction.just_clicked = was_pressed && !btn_interaction.pressed;

        // 发射事件
        if btn_interaction.just_clicked {
            commands.trigger(ButtonClicked { entity });
        }

        // 根据状态更新背景色
        *bg_color = BackgroundColor(match (state.variant, btn_interaction.hovered, btn_interaction.pressed) {
            (Primary, _, true)    => theme.colors.accent_pressed,
            (Primary, true, _)    => theme.colors.accent_hover,
            (Primary, _, _)       => theme.colors.accent_primary,
            // ... 其他变体
        });
    }
}
```

关键设计点：

- **just_clicked 检测**：`was_pressed && !btn_interaction.pressed` 意思是"上一帧按下、这一帧松开"。这比监听 `Interaction::None` 更可靠，因为 `just_clicked` 只持续一帧，不会重复触发
- **禁用态短路**：禁用时所有交互状态清零，背景色固定为 `surface_disabled`，不触发任何事件
- **状态驱动颜色**：背景色不是写死的——每一帧根据 variant + hover + press 实时计算

### 测试覆盖了什么

Button 的单元测试 `button_test.rs`（384 行）覆盖了：

| 测试类别 | 测试内容 |
|---------|---------|
| 构造 | `spawn_button` 返回的 Entity 有正确的 Component（ButtonState/ButtonInteraction/Node/Button/BackgroundColor） |
| 变体颜色 | 4 种 ButtonVariant 在 idle/hover/pressed 下都有正确的颜色值 |
| 禁用态 | disabled=true 时不触发 ButtonClicked，背景色固定为 surface_disabled |
| 交互追踪 | hovered/pressed/just_clicked 三字段在 Interaction 变化时的状态转换 |
| 事件发射 | 点击 Button → Observer 收到 ButtonClicked → entity 匹配 |

---

## 16. Modal 生命周期——最复杂的 Observer 路由

Modal 是项目中 Observer 模式最复杂的用例。它的完整生命周期涉及 4 个步骤：

### Step 1: Factory 创建 UI 树

```text
Overlay (全屏半透明遮罩, ModalState, absolute)
  └── Card (居中白色卡片)
        ├── Text (标题, heading 字号)
        ├── Text (消息, body 字号)
        └── ButtonRow (flex-end, 右对齐)
              ├── Button ("取消", Secondary) — ModalButtonRole::Cancel
              └── Button ("确定", Primary)   — ModalButtonRole::Confirm
```

Alert 变体只有"确定"按钮（用 Cancel 角色但显示 Confirm 文本），Confirm 变体有取消+确定两个按钮，Custom 变体没有默认按钮。

### Step 2: 按钮点击触发 ButtonClicked

和普通的按钮完全一样——`button_interaction_system` 检测到点击释放，`commands.trigger(ButtonClicked { entity })`。

### Step 3: Observer 沿 ChildOf 链向上查找 Modal

```rust
pub fn modal_interaction_observer(
    trigger: On<ButtonClicked>,
    mut commands: Commands,
    role_query: Query<&ModalButtonRole>,
    parent_query: Query<&ChildOf>,
    modal_query: Query<Entity, With<ModalState>>,
) {
    let button_entity = trigger.event().entity;

    // 只处理模态框内的按钮（有 ModalButtonRole 的）
    let Ok(role) = role_query.get(button_entity) else { return; };

    // 从按钮开始，沿 ChildOf 链向上爬，找 ModalState
    let mut current = button_entity;
    let overlay = loop {
        match parent_query.get(current) {
            Ok(parent) => {
                let parent_entity = parent.parent();
                if modal_query.contains(parent_entity) {
                    break Some(parent_entity);
                }
                current = parent_entity;
            }
            Err(_) => break None,
        }
    };

    // 触发 ModalConfirmed / ModalCancelled
    match role {
        ModalButtonRole::Confirm => {
            commands.trigger(ModalConfirmed { entity: overlay });
        }
        ModalButtonRole::Cancel => {
            commands.trigger(ModalCancelled { entity: overlay });
        }
    }

    // 清除整个模态框节点树
    commands.entity(overlay).despawn();
}
```

关键设计点：

- **ChildOf 链查找**：按钮可能嵌套在 ButtonRow → Card → Overlay 中，Observer 用 loop 逐层向上爬，直到找到 `With<ModalState>` 的实体
- **Role 区分**：通过 `ModalButtonRole` 标记组件（Confirm/Cancel）来区分两个按钮的行为，而不是用 `Entity` 硬编码
- **默认清除**：触发事件后自动 despawn 整个 overlay 子树

### Step 4: 事件传播

`ModalConfirmed` / `ModalCancelled` 是独立的 Event，供 Screen 层或其他系统监听做后续处理（如关闭面板后触发保存）。

---

## 17. ScreenPlugin——Observer 是怎么注册的

ScreenPlugin 是 UI 插件链的最后一环。它做的事情很简单：

```rust
pub struct ScreenPlugin;

impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册类型（用于 Reflect 序列化/调试）
            .register_type::<MenuAction>()
            .register_type::<MainMenuScreen>()
            .register_type::<BattleAction>()
            .register_type::<BattleScreen>()

            // Startup 阶段创建屏幕（当前方式，未来会改为 OnEnter/OnExit）
            .add_systems(Startup, spawn_main_menu)
            .add_systems(Startup, spawn_battle_screen)

            // 注册 Observer
            .add_observer(on_main_menu_button_clicked)
            .add_observer(on_battle_button_clicked);
    }
}
```

**当前的问题**（也是 Phase B→C 过渡的标志）：

- 使用 `Startup` system，意味着**两个屏幕同时存在**——它们不是互斥的。这是临时的
- 未来应该用 `OnEnter(GameState::MainMenu)` 和 `OnEnter(GameState::Combat)` 来切换屏幕
- `ScreenStack` 导航系统还未实现

---

## 18. LocalizedText——UI 文本是怎么变成各国语言的

架构文档描述了 Localization 的完整体系（详见 `docs/08-knowledge/localization-overview.md`），而 UI 层是它的主要消费者。

### LocalizedText 组件

```rust
#[derive(Component)]
pub struct LocalizedText {
    pub key: &'static str,           // 编译期常量 key
    pub params: Vec<(&'static str, String)>,  // Fluent 参数
}

impl LocalizedText {
    pub fn static_text(key: &'static str) -> Self { /* 无参数 */ }
    pub fn with_params(key: &'static str, params: ...) -> Self { /* 有参数 */ }
}
```

组件不存储翻译结果——只存储 Key 和参数。这种方式的好处是：切换语言时不需要更新任何 Component，只需要 `LocalizationDatabase` Res 被替换，下一次 `render_localized_text` system 运行时会自动重新解析。

### render_localized_text system

```rust
pub fn render_localized_text(
    db: Res<LocalizationDatabase>,
    mut cache: ResMut<LocalizedTextCache>,
    mut query: Query<(&LocalizedText, &mut Text), Changed<LocalizedText>>,
) {
    for (loc_text, mut text) in query.iter_mut() {
        match resolve_cached(&db, &mut cache, loc_text.key, &params) {
            Ok(resolved) => text.0 = resolved,
            Err(e) => text.0 = format!("[LOC_ERR: {}]", e),
        }
    }
}
```

**`Changed<LocalizedText>`** 过滤器确保只在 Key 或参数变化时才重新解析。使用 result 缓存避免频繁查找。

### 实际使用

目前唯一使用 `LocalizedText` 的代码是 Modal factory：

```rust
// Alert 对话框的确认按钮（使用生成的 loc::core::CONFIRM Key）
LocalizedText::static_text(loc::core::CONFIRM)

// Confirm 对话框的取消按钮
LocalizedText::static_text(loc::core::CANCEL)

// 确认按钮（同样是 loc::core::CONFIRM）
LocalizedText::static_text(loc::core::CONFIRM)
```

`loc::core::CONFIRM` 是 build.rs 从 `.ftl` 文件生成的 Rust 常量。将来所有用户可见文本都通过 `LocalizedText` + 生成的 loc 常量来引用。

---

## 19. Button 测试全景

Button 的测试文件（`src/ui/primitives/button/tests/unit/button_test.rs`）是 UI 层最完整的测试套件，384 行覆盖以下维度：

```rust
// 构造验证
#[test]
fn spawn_button_creates_button() {
    let mut app = App::new();
    app.add_plugins(TestThemePlugin);
    let entity = app.world_mut().run_system_cached(spawn_button, ...);
    assert!(entity.has::<ButtonState>());
    assert!(entity.has::<ButtonInteraction>());
    assert!(entity.has::<Node>());
    assert!(entity.has::<Button>());   // Bevy UI 原生类型
}

// 变体颜色测试——每个变体在 idle/hover/pressed 下都验证
#[test]
fn primary_button_has_correct_idle_color() { ... }
#[test]
fn primary_button_has_correct_hover_color() { ... }
#[test]
fn primary_button_has_correct_pressed_color() { ... }
#[test]
fn danger_button_has_correct_idle_color() { ... }
// 共 4 × 3 = 12 个颜色变体测试

// 禁用态测试
#[test]
fn disabled_button_does_not_emit_click() {
    // 设置 disabled = true → 点击后检查 no event
}

// 交互状态追踪
#[test]
fn just_clicked_only_on_release() {
    // Pressed → "was_pressed" → None → just_clicked = true
}

// Observer 接收事件
#[test]
fn button_click_observer_receives_event() {
    // 注册 Observer → trigger → 检查 receiver
}
```

测试模式：使用 `TestThemePlugin`（不加载配置文件，直接构造默认 Theme），用 `app.world_mut().run_system_cached()` 触发工厂函数，用 Query + assert 验证 Component 值。

---

## 20. 为什么是 set_parent_in_place 而不是嵌套 with_children

项目中所有 Screen 和 Widget 不使用 `spawn(...).with_children(|parent| { ... })` 的嵌套模式，而是用**"先创建、后关联"**模式：

```rust
// ✅ 实际做法：工厂返回 Entity，然后 set_parent_in_place
let root = spawn_panel(&mut commands, &theme, PanelVariant::Basic);
let title = spawn_text(&mut commands, &asset_server, &theme, "Fre", TextVariant::Title);

commands.entity(title).set_parent_in_place(root);

// ❌ 不做：在 spawn_* 内部用 with_children 硬编码层次
```

原因：

1. **工厂函数保持通用**：`spawn_button` 不知道按钮将来是放在主菜单、战斗屏幕还是 Modal 中。它只创建按钮本身，层次由调用方决定
2. **重新父级化容易**：如果需要将一个 Widget 从一个容器移到另一个容器，用 `set_parent_in_place` 重新挂载即可，无需重建 UI 树
3. **测试友好**：可以独立创建 UI 元素并验证 Component，不需要构造整个 UI 树
4. **清晰的关注点分离**：Factory 负责"创建"，Screen/Widget 负责"布局"
