---
id: history.archive.adr.framework-ui
title: framework-ui
status: archived
owner: architect
created: 2026-06-14
updated: 2026-06-14
---

# ADR: 框架 UI 架构（Framework UI）

## 状态
Proposed

---

## 背景

项目拥有完整的战斗 UI（单位面板、行动菜单、战斗日志、VFX），但缺少框架层面（out-of-battle）的 UI 系统：

- 游戏启动后直接进入战斗（`Startup → InGame`），无主菜单
- 只有一个关卡，无关卡选择界面
- `GameOver` 状态通过 `turn_indicator` 系统切换，但无对应的 GameOver 交互界面（结果展示、重玩、返回）
- 玩家退出到主菜单后无法重新进入游戏

@domain-designer 已产出 `docs/domain/campaign_rules_v1.md`，定义了完整的战役流程状态机（MainMenu → LevelSelect → InGame → GameOver → LevelSelect）。@architect 已完成 `docs/adr/campaign-pipeline.md` 定义了 Campaign 数据层架构。本 ADR 基于两者设计对应的框架 UI 架构。

### 关键问题

1. 框架 UI 屏幕（MainMenu、LevelSelect、GameOver）如何组织？作为独立模块还是 `ui/` 子模块？
2. 屏幕之间如何切换？OnEnter/OnExit 模式还是 System 驱动？
3. 屏幕 UI 如何遵守现有的 ViewModel 隔离原则？
4. 现有 `AppState` 只有 MainMenu/InGame/GameOver，需要新增 LevelSelect 状态吗？
5. `UiCommand` 需要新增哪些变体以支持菜单交互？
6. `UiTheme` 需要新增哪些菜单专用样式？

---

## 引用的领域规则

- `docs/domain/campaign_rules_v1.md` — 战役流程状态机、关卡选择流程、GameOver 处理
- `docs/domain/ui_rules_v2.md` — UI 三层架构（UiCommand → ViewModel → Panel）、不变量
- `docs/domain/level_rules_v1.md` — LevelRegistry 通关引用
- `docs/adr/campaign-pipeline.md` — CampaignPlugin / CampaignRegistry / CampaignProgress 设计
- `docs/architecture.md` — 插件注册顺序、Logic/Presentation 分离、AppState 层次

---

## 决策

### 决策 1：新增 `ui/screens/` 模块组织框架 UI

新增 `src/ui/screens/` 目录，包含以下文件：

```
src/ui/screens/
├── mod.rs             // ScreensPlugin 导出
├── main_menu.rs       // 主菜单屏幕
├── level_select.rs    // 关卡选择屏幕
└── game_over.rs       // 游戏结果屏幕
```

**理由**：
- 与现有 `ui/panels/`（战斗内面板）并排，表达"屏幕 vs 面板"的职责差异
- 屏幕（Screen）= 全屏、独占的 UI 视图，面板（Panel）= 战斗中的局部 UI 元素
- 不创建独立的 `menu/` 模块（菜单是表现层，应属于 `ui/` 领域）
- 符合 Feature First 原则：`screens/` 表达"这是一个屏幕"的业务含义

#### 允许
- `screens/` 模块引用 `campaign::CampaignRegistry` 和 `campaign::CampaignProgress`
- `screens/` 使用 `ui::view_models` 中定义的 ViewModel
- `screens/` 读取 `AppState` 驱动自身可见性

#### 禁止
- 🟥 禁止：`screens/` 中的屏幕直接操作 ECS 组件
- 🟥 禁止：`screens/` 绕过 ViewModel 直接 Query 游戏数据
- 🟥 禁止：`screens/` 在 InGame 状态下运行（应该在 InGame 时 despawn）

### 决策 2：AppState 扩展为四态

当前 `AppState`：

```rust
pub enum AppState {
    MainMenu,
    InGame,
    GameOver,
}
```

扩展为：

```rust
pub enum AppState {
    MainMenu,
    LevelSelect,
    InGame,
    GameOver,
}
```

**理由**：
- 领域模型明确要求 MainMenu → LevelSelect → InGame → GameOver 流程
- 禁止跳过 LevelSelect（领域规则 3.4 强制）
- 四态都有独立的可见性逻辑（哪些 UI 显示/隐藏）

**变更**：
- `AppState` 的 `default` 保持 `MainMenu`（配合 `campaign-pipeline.md` 的 Startup 变更）
- 现有 `run_if(in_state(AppState::InGame))` 条件保持不变
- `LevelSelect` 新增对应的 `run_if` 守卫

### 决策 3：屏幕使用 Spawn/Despawn OnEnter/OnExit 模式

每个屏幕在 `OnEnter(AppState::Xxx)` 时生成 UI Entity，在 `OnExit(AppState::Xxx)` 时清理。

**理由**：
- 框架屏幕（菜单/选关/结果）与战斗面板性质不同：它们是独占的、全屏的、非持久性的
- 使用 Entity 管理生命周期，而非 Visibility 切换，避免"隐藏但仍在运行"的内存浪费
- 符合 Bevy 官方推荐的屏幕管理模式
- OnExit 自动清理，防止状态残留

**模式**：

```rust
// 以 MainMenu 为例
fn spawn_main_menu(mut commands: Commands) {
    // 生成主菜单 UI Entity，并附带 DespawnOnExit<MainMenu> 标记
    commands.spawn((
        NodeBundle { /* 全屏布局 */ },
        MainMenuScreen,        // 标记组件
        DespawnOnExit,         // 标记：此状态退出时自动清理
    ));
}
```

**实现选择**：使用 `#[derive(Component)]` 标记 + `OnExit` 系统清理，而非 Bevy 内置的 `StateScoped`（Bevy 0.18 不支持默认 `StateScoped`）。

清理系统设计：

```rust
// 统一清理：在进入新状态时，清理上一状态的所有屏幕实体
fn despawn_screens(mut commands: Commands, screens: Query<Entity, Or<(
    With<MainMenuScreen>,
    With<LevelSelectScreen>,
    With<GameOverScreen>,
)>>) {
    for entity in &screens {
        commands.entity(entity).despawn_recursive();
    }
}
```

或者更精确的 OnExit 系统：

```rust
// OnExit 系统在各状态退出时注册
fn cleanup_main_menu(mut commands: Commands, screens: Query<Entity, With<MainMenuScreen>>) {
    for entity in &screens {
        commands.entity(entity).despawn_recursive();
    }
}
```

**选择更精确的 OnExit 模式**，因为：
- 各状态清理职责清晰
- 避免"跨状态误清理"（例如 GameOver Screen 和 MainMenu Screen 同时存在时不应清理对方）
- 与 Bevy 的状态机精神一致

### 决策 4：屏幕 ViewModel 扩展

新增框架 UI 专用的 ViewModel，遵循现有 ViewModel 隔离原则。

#### MenuState（新增 Resource）

```rust
#[derive(Resource, Reflect, Default, Debug)]
pub struct MenuState {
    pub active: bool,
}
```

#### LevelSelectState（新增 Resource）

```rust
#[derive(Resource, Reflect, Default, Debug)]
pub struct LevelSelectState {
    pub stages: Vec<StageEntry>,
    pub selected_stage: Option<String>,
    pub campaign_name: String,
}

#[derive(Clone, Debug, Reflect)]
pub struct StageEntry {
    pub stage_id: String,
    pub level_name: String,
    pub status: StageStatus,  // Locked / Unlocked / Completed
    pub level_description: String,
}
```

此 ViewModel 从 `CampaignRegistry` + `CampaignProgress` 构建。

#### GameResultView（新增 Resource）

```rust
#[derive(Resource, Reflect, Default, Debug)]
pub struct GameResultView {
    pub result: GameOutcome,   // Victory / Defeat
    pub turn_count: u32,
    pub stage_name: String,
    pub has_next_stage: bool,
}

#[derive(Clone, Debug, Reflect, PartialEq, Eq)]
pub enum GameOutcome {
    Victory,
    Defeat,
}
```

此 ViewModel 从 `GameOverState` + `CampaignProgress` 构建。

### 决策 5：新增 UiCommand 变体

扩展 `UiCommand` 枚举以支持菜单交互：

```rust
pub enum UiCommand {
    // ... 已有变体保持不变 ...

    // 新增：菜单命令
    StartGame,             // 主菜单 → 开始游戏
    SelectStage { stage_id: String },  // 关卡选择
    ConfirmStage,          // 确认进入选中关卡
    RetryStage,            // 重玩当前关卡
    NextStage,             // 下一关
    BackToLevelSelect,     // 返回关卡选择
    BackToMainMenu,        // 返回主菜单
    QuitGame,              // 退出游戏
}
```

**处理位置**：新增 `handle_menu_commands` 系统，与 `handle_ui_commands` 并列但由不同的 `AppState` 守卫触发：

```rust
.add_systems(Update, handle_menu_commands
    .run_if(not(in_state(AppState::InGame))),  // 非战斗状态
)
```

### 决策 6：新增 UiTheme 菜单样式

扩展 `UiTheme` 以支持菜单专用样式：

```rust
pub struct UiTheme {
    // ... 已有字段 ...

    // ── 菜单样式 ──
    pub menu_title_color: Color,         // 标题颜色
    pub menu_button_bg: Color,           // 菜单按钮背景
    pub menu_button_hover: Color,        // 菜单按钮悬停
    pub menu_bg: Color,                  // 菜单背景色
    pub stage_locked_color: Color,       // 关卡锁定色
    pub stage_unlocked_color: Color,     // 关卡解锁色
    pub stage_completed_color: Color,    // 关卡已完成色
    pub stage_selected_border: Color,    // 关卡选中边框色
    pub victory_color: Color,            // 胜利结果色
    pub defeat_color: Color,             // 失败结果色

    // ── 菜单字号 ──
    pub font_title: f32,                 // 标题字号
    pub font_subtitle: f32,              // 副标题字号

    // ── 菜单布局 ──
    pub menu_button_width: f32,          // 菜单按钮宽度
    pub menu_button_height: f32,         // 菜单按钮高度
    pub stage_card_width: f32,           // 关卡卡片宽度
    pub stage_card_height: f32,          // 关卡卡片高度
}
```

**默认值**：采用与现有主题一致的低饱和度暗色系，菜单背景与面板背景协调。

---

## 屏幕设计

### MainMenu 主菜单

**布局**（垂直居中排列）：

```
┌──────────────────────────────────────┐
│                                      │
│        ⚔ 回合制战棋                  │  <- 标题（font_title）
│                                      │
│        ┌──────────────────┐           │
│        │   开始游戏        │           │  <- 按钮（font_menu）
│        └──────────────────┘           │
│        ┌──────────────────┐           │
│        │   继续战役        │           │  <- 按钮（灰显：CampaignProgress 为空时）
│        └──────────────────┘           │
│        ┌──────────────────┐           │
│        │   退出游戏        │           │
│        └──────────────────┘           │
│                                      │
│         版本号 v0.1.0                │  <- 底部小字
└──────────────────────────────────────┘
```

**交互**：
- "开始游戏" → 设置 CampaignProgress 初始状态，进入 LevelSelect
- "继续战役" → CampaignProgress 非空时可用，直接进入 LevelSelect
- "退出游戏" → 退出进程（`App::exit()`）

### LevelSelect 关卡选择

**布局**：

```
┌──────────────────────────────────────┐
│  边境之旅                    ← 返回  │  <- 顶部：战役名 + 返回按钮
├──────────────────────────────────────┤
│                                      │
│  ┌──────────┐   ┌──────────┐        │
│  │ 教学关    │   │ 森林路口  │        │  <- 关卡卡片
│  │ ◉ 已解锁  │   │ ◯ 已锁定  │        │     (选中时高亮边框)
│  │ 当前选中  │   │          │        │
│  └──────────┘   └──────────┘        │
│                                      │
│  ┌──────────┐   ┌──────────┐        │
│  │ 哥布林要塞│   │          │        │
│  │ ◯ 已锁定  │   │          │        │
│  │          │   │          │        │
│  └──────────┘   └──────────┘        │
│                                      │
├──────────────────────────────────────┤
│             [进入战斗]                │  <- 底部：确认按钮（解锁关卡才可点击）
└──────────────────────────────────────┘
```

**逻辑**：
- 读取 `CampaignProgress.stages`，显示各关卡状态
- Locked 关卡显示锁定样式，不可选中
- Unlocked/Completed 关卡可选择
- 选中后"进入战斗"按钮激活
- 点击"进入战斗"→ 设置 `CampaignProgress.current_stage` → 切换 AppState → InGame

**ViewModel 更新**：`update_level_select_view` 系统监听 `CampaignProgress` 变化

### GameOver 游戏结果

**布局**（胜利/失败两种状态）：

```
胜利时：
┌──────────────────────────────────────┐
│                                      │
│        🏆 胜利！                      │  <- 颜色：victory_color
│                                      │
│        教学关                        │  <- 关卡名
│        回合数：12                     │
│                                      │
│  ┌──────────┐   ┌──────────┐        │
│  │  下一关   │   │  重玩    │        │  <- 下一关灰显（无下一关时）
│  └──────────┘   └──────────┘        │
│  ┌──────────┐                        │
│  │  返回选关  │                      │
│  └──────────┘                        │
└──────────────────────────────────────┘

失败时：
┌──────────────────────────────────────┐
│                                      │
│        💀 失败...                     │  <- 颜色：defeat_color
│                                      │
│        教学关                        │
│        回合数：8                      │
│                                      │
│  ┌──────────┐   ┌──────────┐        │
│  │  重玩    │   │ 返回选关  │        │
│  └──────────┘   └──────────┘        │
└──────────────────────────────────────┘
```

**逻辑**：
- 胜利时：下一关可用（`CampaignProgress` 中下一个 stage 存在且当前为 Unlocked）
- 失败时：不显示"下一关"（必须重玩或返回）
- 重玩：重新进入 InGame（当前 level_id 不变）
- 下一关：进入 LevelSelect（自动选中下一关）
- 返回选关：进入 LevelSelect

**ViewModel 更新**：`update_game_result_view` 系统在进入 `AppState::GameOver` 时从 `GameOverState` 和 `CampaignProgress` 构建

---

## 插件注册顺序

### ScreensPlugin

新增 `ScreensPlugin`，在 `UiPlugin` 内部注册，与现有面板插件并列：

```rust
// src/ui/mod.rs
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // ... 现有注册 ...
            .add_plugins((
                // 现有面板插件
                camera::CameraPlugin,
                panels::TurnIndicatorPlugin,
                // ...
                // 新增：框架屏幕插件
                screens::ScreensPlugin,
            ))
            // 菜单命令处理（非 InGame 时运行）
            .add_systems(Update, handle_menu_commands
                .run_if(not(in_state(AppState::InGame))),
            );
    }
}
```

### ScreensPlugin 内部

```rust
pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册 ViewModel Resource
            .init_resource::<MenuState>()
            .init_resource::<LevelSelectState>()
            .init_resource::<GameResultView>()

            // MainMenu 屏幕
            .add_systems(OnEnter(AppState::MainMenu), spawn_main_menu)
            .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu)

            // LevelSelect 屏幕
            .add_systems(OnEnter(AppState::LevelSelect), spawn_level_select)
            .add_systems(OnExit(AppState::LevelSelect), cleanup_level_select)
            .add_systems(Update, (
                update_level_select_view,
                handle_level_select_interaction,
            ).run_if(in_state(AppState::LevelSelect)))

            // GameOver 屏幕
            .add_systems(OnEnter(AppState::GameOver), (
                // 注意顺序：先更新 ViewModel，再生成屏幕
                update_game_result_view,
                spawn_game_over_screen,
            ))
            .add_systems(OnExit(AppState::GameOver), cleanup_game_over_screen)
            .add_systems(Update, handle_game_over_interaction
                .run_if(in_state(AppState::GameOver)));
    }
}
```

---

## AppState 完整转换图

```
Startup
  │
  ▼
MainMenu ───► LevelSelect ───► InGame ───► GameOver
  ▲                ▲              │            │
  │                └──────────────┘            │
  │                  (返回选关/重玩)             │
  │                                            │
  └────────────────────────────────────────────┘
                    (返回主菜单)
```

| 转换 | 触发 | 处理 |
|------|------|------|
| Startup → MainMenu | 自动（default = MainMenu） | — |
| MainMenu → LevelSelect | 玩家点击"开始游戏"或"继续战役" | NextState(AppState::LevelSelect) |
| LevelSelect → InGame | 玩家选中关卡并点击"进入战斗" | StageStarted + NextState(InGame) |
| InGame → GameOver | GameOverState ≠ Playing | turn_indicator 现有逻辑 |
| GameOver → LevelSelect | 玩家点击"返回选关"或"下一关" | NextState(AppState::LevelSelect) |
| GameOver → InGame | 玩家点击"重玩" | NextState(AppState::InGame) |
| LevelSelect → MainMenu | 玩家点击"返回" | NextState(AppState::MainMenu) |
| GameOver → MainMenu | 留作扩展 | — |

---

## Communication Design

### Message（跨功能通信）

| Message | 发送方 | 接收方 | 说明 |
|---------|--------|--------|------|
| `UiCommand` | input / screens | `handle_menu_commands`, `handle_ui_commands` | 已有，扩展变体 |
| `StageStarted` | `screens/level_select` | `campaign/progress`, `battle/setup` | 新 Message，通知战斗初始化 |

### ViewModel（逻辑 → 表现桥接）

| ViewModel | 构建时机 | 数据来源 | 屏幕 / 面板 |
|-----------|----------|----------|-------------|
| `MenuState` | OnEnter(MainMenu) | 静态 | MainMenu |
| `LevelSelectState` | OnEnter(LevelSelect) + CampaignProgress 变化 | CampaignRegistry, CampaignProgress | LevelSelect |
| `GameResultView` | OnEnter(GameOver) | GameOverState, CampaignProgress | GameOver |

---

## 与现有系统的关系

### 与 `turn_indicator.rs` 的关系

当前 `turn_indicator.rs` 包含 GameOver 状态切换逻辑：

```rust
// 现有逻辑（保持）
GameOverState::Victory => next_app_state.set(AppState::GameOver),
GameOverState::Defeat => next_app_state.set(AppState::GameOver),
```

此逻辑保持不变。新增的 `spawn_game_over_screen` 在 `OnEnter(GameOver)` 中运行，在 GameOver UI 就绪后接管交互。

### 与 `campaign/progression.rs` 的关系

`campaign/progression.rs` 监听 `LevelCompleted` Message 并更新 `CampaignProgress`。
GameOver 屏幕的 `ViewModel` 在 `OnEnter(GameOver)` 时从更新后的 `CampaignProgress` 读取数据。

顺序：
1. `turn/victory_check` → `GameOverState::Victory/Defeat`
2. `turn_indicator` → `AppState::GameOver`
3. `campaign/progression` → 更新 `CampaignProgress`（通过 `LevelCompleted` Message）
4. `OnEnter(GameOver)` → `update_game_result_view` 从 `CampaignProgress` 构建 ViewModel
5. `OnEnter(GameOver)` → `spawn_game_over_screen` 使用 ViewModel 渲染

### 与 `input.rs` 的关系

当前 input.rs 仅在 `AppState::InGame` 时处理输入。框架 UI 屏幕使用 bevy_ui 按钮交互（`Interaction` 组件），不依赖 input.rs。

---

## 边界定义

### 允许

- `screens/level_select` 读取 `campaign::CampaignRegistry`（展示关卡列表）
- `screens/level_select` 读取 `campaign::CampaignProgress`（展示解锁状态）
- `screens/game_over` 读取 `campaign::CampaignProgress`（判断下一关是否存在）
- `screens` 发送 `UiCommand` Message（通过 `handle_menu_commands` 处理）
- `command_handler` 处理新增的菜单相关 `UiCommand` 变体

### 禁止

- 🟥 禁止：屏幕直接修改 `CampaignProgress` — 必须通过 `UiCommand` + `handle_menu_commands`
- 🟥 禁止：屏幕直接操作 `NextState<AppState>` — 必须通过 `UiCommand`
- 🟥 禁止：屏幕在 `AppState::InGame` 时运行 — 由 `run_if` 守卫保证
- 🟥 禁止：`screens/` 模块引用战斗相关的内部数据（如 `CombatIntent`, `TurnPhase`）
- 🟥 禁止：屏幕 UI 缓存业务数据副本 — 每次从 ViewModel 读取

---

## Forbidden（禁止事项）

- 🟥 **FORBIDDEN-1** — 禁止：屏幕绕过 ViewModel 直接 Query 游戏组件 — 理由：违反 Logic/Presentation 分离（INV-UI-02）
- 🟥 **FORBIDDEN-2** — 禁止：屏幕直接修改 `CampaignProgress` — 理由：必须通过 `UiCommand` + `handle_menu_commands`
- 🟥 **FORBIDDEN-3** — 禁止：在 `AppState::InGame` 时保留菜单屏幕 Entity — 理由：OnExit 清理，避免隐藏 Entity 残留
- 🟥 **FORBIDDEN-4** — 禁止：屏幕之间共享可变状态 — 理由：每个屏幕的 ViewModel 独立，通过 AppState 生命周期管理
- 🟥 **FORBIDDEN-5** — 禁止：跳过 LevelSelect 直接从 MainMenu 进入 InGame — 理由：领域规则 3.4，即使只有一个关卡

---

## Definition / Instance Design

### Definition（不可变配置）

| 类型 | 存储位置 | 说明 |
|------|----------|------|
| `UiTheme`（扩展后） | Resource | 菜单样式常量 |
| 屏幕布局模板 | bevy_ui NodeBundle | 硬编码在 spawn 系统内 |

### Instance（运行时状态）

| 类型 | 存储方式 | 写入方 | 读取方 |
|------|----------|--------|--------|
| `MenuState` | Resource | `spawn_main_menu` | MainMenu 渲染 |
| `LevelSelectState` | Resource | `update_level_select_view` | LevelSelect 渲染 |
| `GameResultView` | Resource | `update_game_result_view` | GameOver 渲染 |
| 屏幕 Entity | ECS Entity | `OnEnter` 系统 | `OnExit` 清理 |
| `UiCommand`（菜单变体） | Message | 屏幕按钮交互 | `handle_menu_commands` |

---

## 后果

### 正面

- **完整流程闭环**：玩家从启动到通关有完整的 UI 引导，不再"直接跳入战斗"
- **与领域模型一致**：AppState 四态映射战役流程状态机，无概念缺口
- **最小侵入**：新增 `screens/` 子模块，不修改现有面板代码；扩展 `UiCommand`、`UiTheme`、`view_models` 为增量修改
- **OnEnter/OnExit 确定性**：屏幕生命周期由 AppState 驱动，无隐藏状态
- **ViewModel 隔离延续**：菜单屏幕依然遵守同样的只读 ViewModel 原则

### 负面

- **新增代码量**：3 个屏幕文件 + 扩展 3 个现有文件（events.rs、theme.rs、view_models.rs），估算新增 400-600 行
- **AppState 四态 vs 三态**：新增 LevelSelect 状态，现有 `in_state(AppState::InGame)` 守卫不变，但新增状态需要确保不影响现有系统
- **`handle_menu_commands` vs `handle_ui_commands`**：两套命令处理器共享 `UiCommand` Message 但由不同守卫隔离，需要确保没有命令在错误状态下被处理

---

## 替代方案

### 替代方案 A：Visibility 切换而非 Spawn/Despawn

使用 `Visibility` 组件切换屏幕显隐，而非 OnEnter/OnExit 创建/销毁。

**放弃理由**：
- 隐藏的 Entity 仍然占用内存和（虽然极小）查询开销
- 需要额外的"清理旧屏幕"逻辑，不如 OnExit 自动清理干净
- 退出到主菜单再进入时，旧屏幕 Entity 可能残留
- Spawn/Despawn 模式在 Bevy 社区中更受推荐

### 替代方案 B：独立的 `src/screens/` 顶层模块

将屏幕代码放在 `src/screens/` 而非 `src/ui/screens/`。

**放弃理由**：
- 屏幕是表现层，应属于 `ui/` 领域（架构文档 4.0 UI 模块定义明确）
- 独立模块违反架构中的模块划分（已定义的模块列表不包含 screens/）
- 现有 `ui/panels/` 已是"面板子模块"的先例，screens 同理

### 替代方案 C：使用 Bevy 内置的 StateScoped

**放弃理由**：
- Bevy 0.18 不提供 `StateScoped` 组件（该功能在 0.19 引入）
- 手动 OnEnter/OnExit 清理更明确，不依赖框架版本

---

## 架构合规性自检

- [x] 符合 ECS 约束 — 屏幕使用 Entity + Component 模式，ViewModel 使用 Resource
- [x] 符合 Logic/Presentation 分离 — 屏幕通过 ViewModel 读取数据，通过 UiCommand 发出意图
- [x] 符合 Feature First 原则 — `screens/` 表达业务含义（屏幕）
- [x] 无 systems.rs/components.rs — 按屏幕名拆文件
- [x] ViewModel 隔离 — 菜单屏幕不直接 Query 游戏组件
- [x] 符合 OnEnter/OnExit 轻量原则 — OnEnter 只做 spawn + ViewModel 构建
- [x] AppState 扩展与现有系统兼容 — InGame 守卫不变
- [x] 没有创建禁止的模块 — screens/ 是 ui/ 的子模块
- [x] 没有绕过 Effect/Modifier Pipeline — 菜单屏幕不参与战斗
- [x] 符合"只解决当前复杂度" — 不设计存档/设置/剧情屏幕

## 与现有 Message 表的关系

| Message | 变更类型 | 说明 |
|---------|----------|------|
| `UiCommand` | 扩展变体 | 新增 StartGame / SelectStage / ConfirmStage / RetryStage / NextStage / BackToLevelSelect / BackToMainMenu / QuitGame |
| `StageStarted` | 新增 | LevelSelect → InGame 时发送，通知 campaign/progress 更新 current_stage |
| `LevelCompleted` | 已有 | GameOver 屏幕读取（间接通过 CampaignProgress） |
