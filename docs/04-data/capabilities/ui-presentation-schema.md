---
id: capabilities.ui-presentation.schema.v1
title: UI Presentation Schema — UI 表现层数据架构
status: draft
owner: data-architect
created: 2026-06-19
updated: 2026-06-20
layer: runtime, persistence
replay-safe: true
---

# UI Presentation Schema — UI 表现层数据架构

> **领域归属**: Capabilities — UI 表现层 (L3) | **依赖 Schema**: Cue, Event, Localization, Input, Attribute, Tag, Ability, Effect, Inventory, Quest, Economy | **参考 ADR**: ADR-055 | **参考领域规则**: `docs/02-domain/capabilities/ui-presentation.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `StyleToken` (UiColors/UiSpacing/UiTypography) | Definition | 静态主题配置，运行时不可变 |
| `Theme` | Definition | 主题定义，聚合所有 StyleToken |
| `ViewModel` (各 Vm 结构) | Runtime | Domain Event 投影，瞬态不持久化 |
| `UiStore` | Runtime | ViewModel 统一容器 (Resource) |
| `UiState` | Runtime | 交互瞬态（焦点、悬停、拖拽） |
| `Dirty<T>` | Runtime | 脏标记机制，驱动 Widget 按需刷新 |
| `UiSettings` | Persistence | 用户偏好，跨会话持久化 (SettingsGroup) |
| `ScreenStack` | Persistence | 导航栈，当前 Screen 进入存档 |
| `UiCommand` | Runtime | UI → Domain 的命令通道 |
| `Focusable` / `FocusGroup` | Runtime | 焦点导航，瞬态 |
| `UiBinding` | Runtime | UI 绑定标识，反 Marker 模式，替代独立 Marker 结构体 |
| `WidgetFactory` trait | Runtime | Widget 创建/刷新/销毁入口 |

---

## 2. Problem

UI 作为独立的 Presentation Layer (L3)，需要完整的数据架构设计以解决以下问题：

| # | 问题 | 影响 |
|---|------|------|
| P1 | ViewModel 无统一容器，各 UI 面板直接读取 Domain 数据 | 违反 Presentation/Logic 分离，UI 与 Domain 耦合 |
| P2 | 无 StyleToken 体系，视觉属性散落在 Widget 代码中 | 主题切换困难，视觉一致性无法保证 |
| P3 | 无 Dirty Flag 机制，每帧全量刷新所有 Widget | 性能浪费，帧率不稳定 |
| P4 | UiSettings 无持久化方案，用户偏好丢失 | 用户体验差，每次启动需重新配置 |
| P5 | 导航状态无管理，Screen 堆栈混乱 | 返回逻辑不可预测，存档恢复困难 |
| P6 | UI 数据与 Replay/Save 边界不清 | UI 状态误入 Replay 导致回放断裂 |
| P7 | 无 Projection 层定义，Domain → ViewModel 转换逻辑散落 | 投影逻辑不可测试、不可追踪 |
| P8 | 焦点导航无数据结构支持 | 手柄/键盘导航无法实现 |

---

## 3. 数据层划分

### 3.1 UI 数据分层

```
L3-UI-Data-1: StyleToken（静态，Theme 配置）    → Definition 层
L3-UI-Data-2: ViewModel（动态，Domain 投影）     → Runtime 层
L3-UI-Data-3: UiState（交互状态，瞬态）          → Runtime 层
L3-UI-Data-4: UiSettings（持久，跨会话）         → Persistence 层
```

### 3.2 数据流向

```
Domain Event
    ↓
Projection（纯函数，无副作用）
    ↓
ViewModel（UiStore 中）
    ↓ Dirty<T>
Widget（ECS Component）
    ↓
UiAction
    ↓
UiCommand
    ↓
Domain
```

**关键约束**：
- ViewModel 是 UI 的唯一数据源，Widget 不直接读取 Domain
- Projection 是纯函数，输入 Domain Event，输出 ViewModel 差异
- UiCommand 是 UI → Domain 的唯一通道，禁止 Widget 直接修改 Domain

### 3.3 与其他层的数据关系

| 数据类型 | 来源 | 存储位置 | 进入 Replay | 进入 Save |
|---------|------|---------|------------|----------|
| ViewModel | Domain Event 投影 | UiStore (Resource) | 否 | 否 |
| UiState | 用户交互 | Component | 否 | 否 |
| UiSettings | 用户偏好 | SettingsGroup | 否 | 是 |
| StyleToken | Theme 配置 | Resource | 否 | 否 |
| ScreenStack | 导航操作 | Resource | 否 | 是（当前 Screen） |
| UiCommand | 用户操作 | Event | 是（仅 Command） | 否 |

---

## 4. Schema Design

### 4.1 UiStore（统一 ViewModel 容器）

```rust
/// UI 状态统一容器。
/// 类似 Redux Store，所有 ViewModel 集中管理。
/// Projection 系统更新此容器，Widget 从此容器读取。
#[derive(Resource, Reflect, Default)]
#[reflect(Resource, Default)]
pub struct UiStore {
    /// 战斗 HUD
    pub battle_hud: BattleHudVm,
    /// 角色面板
    pub character_panel: CharacterPanelVm,
    /// 技能面板
    pub skill_panel: SkillPanelVm,
    /// 背包
    pub inventory: InventoryVm,
    /// 商店
    pub shop: ShopVm,
    /// 任务日志
    pub quest_log: QuestLogVm,
    /// 通知队列（先进先出）
    pub notification_queue: Vec<NotificationVm>,
    /// 模态弹窗栈（后进先出）
    pub modal_stack: Vec<ModalVm>,
}
```

**设计决策**：
- UiStore 是 Resource 而非 Component，全局唯一
- 各 ViewModel 字段平铺而非 HashMap，利用 Rust 类型系统保证访问安全
- notification_queue 和 modal_stack 使用 Vec 而非专用队列类型，保持 Reflect 兼容性

### 4.2 BattleHudVm

```rust
/// 战斗 HUD 视图模型
#[derive(Clone, Reflect, Default)]
pub struct BattleHudVm {
    /// 当前 HP
    pub hp: u32,
    /// 最大 HP
    pub max_hp: u32,
    /// 当前 MP
    pub mp: u32,
    /// 最大 MP
    pub max_mp: u32,
    /// 当前回合数
    pub current_turn: u32,
    /// 当前行动角色
    pub active_character: Option<CharacterId>,
    /// 战斗阶段
    pub phase: BattlePhaseVm,
    /// 技能冷却映射（SkillId → 剩余冷却）
    pub cooldowns: HashMap<SkillId, f32>,
    /// 行动点
    pub action_points: u32,
    /// 最大行动点
    pub max_action_points: u32,
}

/// 战斗阶段视图
#[derive(Clone, Reflect, Default, PartialEq)]
pub enum BattlePhaseVm {
    #[default]
    None,
    PlayerTurn,
    EnemyTurn,
    Animation,
    Victory,
    Defeat,
}
```

### 4.3 CharacterPanelVm

```rust
/// 角色面板视图模型
#[derive(Clone, Reflect, Default)]
pub struct CharacterPanelVm {
    /// 当前查看的角色 ID
    pub character_id: Option<CharacterId>,
    /// 角色名称本地化 Key
    pub name_key: UiTextKey,
    /// 等级
    pub level: u32,
    /// HP / MaxHP / MP / MaxMP
    pub hp: u32,
    pub max_hp: u32,
    pub mp: u32,
    pub max_mp: u32,
    /// 经验值 / 升级所需经验
    pub exp: u32,
    pub exp_to_next: u32,
    /// Buff 列表
    pub buffs: Vec<BuffVm>,
    /// 属性面板
    pub stats: StatsVm,
}

/// Buff 视图模型
#[derive(Clone, Reflect, Default)]
pub struct BuffVm {
    /// Buff Definition ID
    pub buff_id: BuffId,
    /// 名称本地化 Key
    pub name_key: UiTextKey,
    /// 图标资源 Key
    pub icon_key: AssetKey,
    /// 剩余回合数（None = 永久）
    pub remaining_turns: Option<u32>,
    /// 当前堆叠数
    pub stacks: u32,
    /// 是否为 Debuff
    pub is_debuff: bool,
}

/// 属性面板视图模型
#[derive(Clone, Reflect, Default)]
pub struct StatsVm {
    pub attack: i32,
    pub defense: i32,
    pub speed: i32,
    pub crit_rate: f32,
    pub crit_damage: f32,
    pub hit_rate: f32,
    pub evasion: f32,
}
```

### 4.4 SkillPanelVm

```rust
/// 技能面板视图模型
#[derive(Clone, Reflect, Default)]
pub struct SkillPanelVm {
    /// 技能槽位列表
    pub skills: Vec<SkillSlotVm>,
    /// 当前选中技能
    pub selected: Option<SkillId>,
    /// 剩余行动点
    pub ap_remaining: u32,
    /// 最大行动点
    pub max_ap: u32,
}

/// 技能槽位视图模型
#[derive(Clone, Reflect)]
pub struct SkillSlotVm {
    /// 技能 Definition ID
    pub skill_id: SkillId,
    /// 名称本地化 Key
    pub name_key: UiTextKey,
    /// 描述本地化 Key
    pub desc_key: UiTextKey,
    /// 图标资源 Key
    pub icon_key: AssetKey,
    /// 冷却剩余（0 = 可用）
    pub cooldown_remaining: f32,
    /// 是否可用（综合判断：AP、MP、冷却、沉默等）
    pub is_usable: bool,
    /// MP 消耗
    pub mp_cost: u32,
    /// AP 消耗
    pub ap_cost: u32,
}
```

### 4.5 InventoryVm

```rust
/// 背包视图模型
#[derive(Clone, Reflect, Default)]
pub struct InventoryVm {
    /// 物品槽位列表
    pub items: Vec<InventorySlotVm>,
    /// 金币
    pub gold: u32,
    /// 当前筛选
    pub filter: InventoryFilterVm,
    /// 当前选中物品
    pub selected: Option<ItemId>,
    /// 排序方式
    pub sort_order: InventorySortOrder,
}

/// 背包筛选类型
#[derive(Clone, Reflect, Default, PartialEq)]
pub enum InventoryFilterVm {
    #[default]
    All,
    Equipment,
    Consumable,
    Material,
    KeyItem,
}

/// 背包排序方式
#[derive(Clone, Reflect, Default, PartialEq)]
pub enum InventorySortOrder {
    #[default]
    ByType,
    ByName,
    ByRarity,
}

/// 背包槽位视图模型
#[derive(Clone, Reflect)]
pub struct InventorySlotVm {
    /// 物品 Definition ID
    pub item_id: ItemId,
    /// 名称本地化 Key
    pub name_key: UiTextKey,
    /// 图标资源 Key
    pub icon_key: AssetKey,
    /// 数量
    pub quantity: u32,
    /// 稀有度
    pub rarity: RarityVm,
    /// 是否已装备
    pub is_equipped: bool,
}

/// 稀有度视图
#[derive(Clone, Reflect, Default, PartialEq)]
pub enum RarityVm {
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}
```

### 4.6 ShopVm

```rust
/// 商店视图模型
#[derive(Clone, Reflect, Default)]
pub struct ShopVm {
    /// 商品列表
    pub items: Vec<ShopSlotVm>,
    /// 玩家当前金币
    pub player_gold: u32,
    /// 商店所属阵营 ID
    pub faction_id: Option<FactionId>,
}

/// 商店槽位视图模型
#[derive(Clone, Reflect)]
pub struct ShopSlotVm {
    /// 物品 Definition ID
    pub item_id: ItemId,
    /// 名称本地化 Key
    pub name_key: UiTextKey,
    /// 图标资源 Key
    pub icon_key: AssetKey,
    /// 售价
    pub price: u32,
    /// 是否可购买（库存/金币判断）
    pub can_buy: bool,
    /// 库存数量（None = 无限）
    pub stock: Option<u32>,
}
```

### 4.7 QuestLogVm

```rust
/// 任务日志视图模型
#[derive(Clone, Reflect, Default)]
pub struct QuestLogVm {
    /// 活跃任务列表
    pub active_quests: Vec<QuestSlotVm>,
    /// 已完成任务 ID 列表
    pub completed_quests: Vec<QuestId>,
    /// 当前筛选
    pub filter: QuestFilterVm,
}

/// 任务筛选
#[derive(Clone, Reflect, Default, PartialEq)]
pub enum QuestFilterVm {
    #[default]
    All,
    MainQuest,
    SideQuest,
    Completed,
}

/// 任务槽位视图模型
#[derive(Clone, Reflect)]
pub struct QuestSlotVm {
    /// 任务 Definition ID
    pub quest_id: QuestId,
    /// 名称本地化 Key
    pub name_key: UiTextKey,
    /// 描述本地化 Key
    pub desc_key: UiTextKey,
    /// 任务类型
    pub quest_type: QuestTypeVm,
    /// 进度（当前/目标）
    pub progress_current: u32,
    pub progress_target: u32,
}

/// 任务类型视图
#[derive(Clone, Reflect, Default, PartialEq)]
pub enum QuestTypeVm {
    #[default]
    Main,
    Side,
    Hidden,
}
```

### 4.8 NotificationVm

```rust
/// 通知视图模型
#[derive(Clone, Reflect)]
pub struct NotificationVm {
    /// 通知唯一标识
    pub id: NotificationId,
    /// 消息本地化 Key
    pub message_key: UiTextKey,
    /// 本地化参数（用于 Fluent 变量插值）
    pub params: HashMap<String, String>,
    /// 优先级
    pub priority: NotificationPriorityVm,
    /// 显示时长（秒）
    pub duration_secs: f32,
    /// 通知类型
    pub notification_type: NotificationTypeVm,
}

/// 通知优先级
#[derive(Clone, Reflect, PartialEq, PartialOrd)]
pub enum NotificationPriorityVm {
    Normal,
    Important,
    Critical,
}

/// 通知类型
#[derive(Clone, Reflect, PartialEq)]
pub enum NotificationTypeVm {
    Toast,
    Banner,
    Popup,
}
```

### 4.9 ModalVm

```rust
/// 模态弹窗视图模型
#[derive(Clone, Reflect)]
pub struct ModalVm {
    /// 弹窗唯一标识
    pub id: ModalId,
    /// 标题本地化 Key
    pub title_key: UiTextKey,
    /// 正文本地化 Key
    pub body_key: UiTextKey,
    /// 正文本地化参数
    pub body_params: HashMap<String, String>,
    /// 按钮列表
    pub buttons: Vec<ModalButtonVm>,
    /// 弹窗类型
    pub modal_type: ModalTypeVm,
}

/// 弹窗按钮视图模型
#[derive(Clone, Reflect)]
pub struct ModalButtonVm {
    /// 按钮文本本地化 Key
    pub label_key: UiTextKey,
    /// 点击后发送的 UiAction
    pub action: UiAction,
    /// 按钮样式
    pub style: ButtonStyleVm,
}

/// 弹窗类型
#[derive(Clone, Reflect, PartialEq)]
pub enum ModalTypeVm {
    Confirm,
    Alert,
    Custom,
}

/// 按钮样式
#[derive(Clone, Reflect, Default, PartialEq)]
pub enum ButtonStyleVm {
    #[default]
    Primary,
    Secondary,
    Danger,
}
```

---

## 5. StyleToken Schema

### 5.1 UiColors

```rust
/// UI 颜色体系 — 全局颜色定义
/// 由 Theme 加载，运行时只读
#[derive(Resource, Reflect, Clone)]
pub struct UiColors {
    // 语义色
    pub primary: Color,
    pub secondary: Color,
    pub danger: Color,
    pub success: Color,
    pub warning: Color,
    // 面板
    pub panel_bg: Color,
    pub panel_border: Color,
    // 文本
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    // 状态条
    pub hp_bar: Color,
    pub mp_bar: Color,
    pub exp_bar: Color,
    // Buff/Debuff
    pub buff_icon: Color,
    pub debuff_icon: Color,
}
```

### 5.2 UiSpacing

```rust
/// UI 间距体系 — 全局间距定义
#[derive(Resource, Reflect, Clone)]
pub struct UiSpacing {
    /// 4px
    pub xs: Val,
    /// 8px
    pub sm: Val,
    /// 16px
    pub md: Val,
    /// 24px
    pub lg: Val,
    /// 32px
    pub xl: Val,
    /// 48px
    pub xxl: Val,
    /// 面板内边距
    pub panel_padding: Val,
    /// 按钮内边距
    pub button_padding: Val,
    /// 图标尺寸
    pub icon_size: Val,
    /// 圆角半径
    pub border_radius: Val,
}
```

### 5.3 UiTypography

```rust
/// UI 排版体系 — 全局字体定义
#[derive(Resource, Reflect, Clone)]
pub struct UiTypography {
    /// 标题字体
    pub heading_font: FontSource,
    /// 正文字体
    pub body_font: FontSource,
    /// 等宽字体
    pub mono_font: FontSource,
    /// 标题字号
    pub heading_size: FontSize,
    /// 正文字号
    pub body_size: FontSize,
    /// 注释字号
    pub caption_size: FontSize,
    /// 按钮字号
    pub button_size: FontSize,
}
```

### 5.4 Theme

```rust
/// 主题定义 — 聚合所有 StyleToken
/// 由 Theme 配置文件加载，运行时只读
#[derive(Resource, Reflect, Clone, Default)]
pub struct Theme {
    pub colors: UiColors,
    pub spacing: UiSpacing,
    pub typography: UiTypography,
    pub name: ThemeName,
}

/// 主题名称
#[derive(Clone, Reflect, Default, PartialEq)]
pub enum ThemeName {
    #[default]
    Dark,
    Light,
    Pixel,
    HD2D,
}
```

---

## 6. UiSettings Schema

```rust
/// UI 用户偏好 — 持久化到磁盘
/// 使用 Bevy 0.19 的 SettingsGroup 自动持久化
#[derive(Resource, SettingsGroup, Reflect, Clone, Default)]
#[reflect(Resource, SettingsGroup, Default)]
pub struct UiSettings {
    /// 显示伤害数字
    pub show_damage_numbers: bool,
    /// 显示小地图
    pub show_minimap: bool,
    /// 显示网格
    pub show_grid: bool,
    /// 战斗速度倍率
    pub battle_speed: f32,
    /// 自动战斗
    pub auto_battle: bool,
    /// 工具提示延迟（秒）
    pub tooltip_delay_secs: f32,
    /// 通知显示时长（秒）
    pub notification_duration_secs: f32,
    /// 当前主题
    pub theme: ThemeName,
    /// 语言
    pub language: LanguageVm,
}

/// 语言选项
#[derive(Clone, Reflect, Default, PartialEq)]
pub enum LanguageVm {
    #[default]
    ZhCn,
    En,
    Ja,
    Ko,
}
```

**SettingsGroup 约束**（Bevy 0.19 要求）：
- 必须同时 derive `Resource` + `SettingsGroup` + `Reflect` + `Default`
- `#[reflect(...)]` 必须包含 `Resource`, `SettingsGroup`, `Default`
- 字段类型必须实现 `Reflect` + `Serialize` + `DeserializeOwned`
- 必须在 UiPlugin 中调用 `register_type::<UiSettings>()`

---

## 7. Navigation Schema

### 7.1 ScreenStack

```rust
/// 页面导航栈 — 管理所有 Screen 的推入/弹出
#[derive(Resource, Reflect, Default)]
pub struct ScreenStack {
    /// 栈中的页面列表（底部 = 最早推入）
    pub screens: Vec<ScreenInfo>,
    /// 最大栈深度（防止无限嵌套）
    pub max_depth: u32, // 默认 10
}

/// 页面信息
#[derive(Clone, Reflect)]
pub struct ScreenInfo {
    /// 页面类型
    pub screen_type: ScreenType,
    /// 页面根实体
    pub entity: Entity,
    /// 过渡动画类型
    pub transition: TransitionType,
}

/// 页面类型
#[derive(Clone, Reflect, PartialEq)]
pub enum ScreenType {
    MainMenu,
    Battle,
    Inventory,
    Shop,
    Settings,
    SaveLoad,
    QuestLog,
    CharacterDetail,
}

/// 过渡动画类型
#[derive(Clone, Reflect, Default, PartialEq)]
pub enum TransitionType {
    #[default]
    Instant,
    Fade(f32),     // 参数：持续时间（秒）
    Slide(Direction),
}

/// 方向
#[derive(Clone, Reflect, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}
```

---

## 8. Focus Schema

```rust
/// 可聚焦组件 — 标记 UI 元素可被手柄/键盘选中
#[derive(Component, Reflect)]
pub struct Focusable {
    /// 焦点 ID
    pub focus_id: FocusId,
    /// 所属焦点组
    pub group: FocusGroupId,
    /// 焦点优先级（数值越大越优先）
    pub priority: u32,
}

/// 焦点组 — 定义一组 Focusable 的导航规则
#[derive(Component, Reflect)]
pub struct FocusGroup {
    /// 组 ID
    pub group_id: FocusGroupId,
    /// 导航模式
    pub navigation: FocusNavigation,
    /// 是否循环导航
    pub wrap: bool,
}

/// 焦点导航模式
#[derive(Clone, Reflect, Default, PartialEq)]
pub enum FocusNavigation {
    #[default]
    Grid { cols: u32 },
    Linear,
    Custom,
}
```

---

## 9. Dirty Flag Schema

```rust
/// 脏标记机制 — ViewModel 更新后设置 Dirty，Widget 只在 Dirty 时刷新
///
/// 设计原则：
/// - Projection 更新 ViewModel 后 mark_dirty()
/// - Widget 系统 consume() 检测脏标记，脏则刷新，否则跳过
/// - consume() 自动清除脏标记，保证每帧最多刷新一次
#[derive(Component, Reflect, Default)]
pub struct Dirty<T: Reflect + Default> {
    pub inner: T,
    pub is_dirty: bool,
}

impl<T: Reflect + Default> Dirty<T> {
    /// 标记为脏
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    /// 消费脏标记：如果脏则清除并返回 true，否则返回 false
    pub fn consume(&mut self) -> bool {
        if self.is_dirty {
            self.is_dirty = false;
            true
        } else {
            false
        }
    }
}
```

**注意**：`Dirty<T>` 是泛型 Component，在 Bevy 0.19 中需要为每个具体类型注册：
```rust
app.register_type::<Dirty<BattleHudVm>>();
app.register_type::<Dirty<CharacterPanelVm>>();
// ... 每个 ViewModel 类型
```

---

## 10. UiCommand Schema

```rust
/// UI → Domain 命令 — UI 层向 Domain 层发送的命令
/// 所有用户操作必须通过此枚举，禁止 Widget 直接修改 Domain
#[derive(Clone, Reflect)]
pub enum UiCommand {
    // ── Combat ──
    /// 施放技能
    CastSkill(SkillId),
    /// 选择目标
    SelectTarget(CharacterId),
    /// 结束回合
    EndTurn,
    /// 移动到网格位置
    MoveToPosition(GridPos),

    // ── Inventory ──
    /// 使用物品
    UseItem(ItemId),
    /// 装备物品
    EquipItem(ItemId, EquipmentSlot),
    /// 丢弃物品
    DropItem(ItemId),

    // ── Quest ──
    /// 接受任务
    AcceptQuest(QuestId),
    /// 放弃任务
    AbandonQuest(QuestId),

    // ── Economy ──
    /// 购买物品
    BuyItem(ItemId, u32),
    /// 出售物品
    SellItem(ItemId, u32),

    // ── System ──
    /// 保存游戏
    SaveGame(SaveSlot),
    /// 加载游戏
    LoadGame(SaveSlot),
    /// 修改设置
    ChangeSettings(UiSettings),
    /// 切换暂停
    TogglePause,
    /// 打开页面
    OpenScreen(ScreenType),
    /// 关闭页面
    CloseScreen,
}
```

**UiCommand 与 Replay 的关系**：
- UiCommand 中影响游戏逻辑的命令（如 `CastSkill`, `EndTurn`, `UseItem`）会被录制到 Replay
- 纯 UI 操作（如 `OpenScreen`, `CloseScreen`）不进入 Replay
- Replay 回放时，UI 从 Domain Event 重新投影，无需重放 UI 操作

---

## 11. UiAction Schema

```rust
/// UI 动作 — Widget 级别的交互动作
/// UiAction 由 Widget 发出，经 UiActionHandler 转换为 UiCommand
/// 与 UiCommand 的区别：UiAction 是 Widget 级别的，UiCommand 是 Domain 级别的
#[derive(Clone, Reflect)]
pub enum UiAction {
    /// 确认
    Confirm,
    /// 取消
    Cancel,
    /// 选择技能
    SelectSkill(SkillId),
    /// 选择物品
    SelectItem(ItemId),
    /// 选择角色
    SelectCharacter(CharacterId),
    /// 选择网格位置
    SelectGridPos(GridPos),
    /// 切换筛选
    ChangeFilter(InventoryFilterVm),
    /// 切换排序
    ChangeSort(InventorySortOrder),
    /// 自定义动作（携带字符串标识）
    Custom(String),
}
```

---

## 12. ID 策略

| ID 类型 | 格式 | 来源 | 唯一性范围 | 说明 |
|---------|------|------|-----------|------|
| `UiTextKey` | `ui.<scope>.<id>.<suffix>` | 定义态 | 全局唯一 | 本地化 Key，遵循 `docs/04-data/foundation/id_strategy.md` §5 |
| `AssetKey` | String | 定义态 | 全局唯一 | 资源路径 Key，如 `"icon_buff_poison"` |
| `NotificationId` | u64 | 运行时生成 | 会话唯一 | 递增计数器，不跨会话 |
| `ModalId` | u64 | 运行时生成 | 会话唯一 | 递增计数器，不跨会话 |
| `FocusId` | u32 | 运行时生成 | Widget 唯一 | Screen 内递增 |
| `FocusGroupId` | u32 | 运行时生成 | Screen 唯一 | Screen 内递增 |

**UiTextKey 命名示例**：

| Key | 含义 |
|-----|------|
| `ui.battle.end_turn` | 战斗：结束回合按钮 |
| `ui.battle.victory` | 战斗：胜利提示 |
| `ui.inventory.empty_slot` | 背包：空槽位提示 |
| `ui.shop.buy_confirm` | 商店：购买确认 |
| `ui.quest.abandon_confirm` | 任务：放弃确认 |
| `ui.settings.show_grid` | 设置：显示网格选项 |
| `ui.notification.item_acquired` | 通知：获得物品 |

---

## 13. Layer Analysis

| 数据结构 | Layer | 持久化 | 可热重载 | 进入 Save | 进入 Replay |
|----------|-------|--------|----------|----------|------------|
| `UiColors` | Definition | 配置文件 | 是 | 否 | 否 |
| `UiSpacing` | Definition | 配置文件 | 是 | 否 | 否 |
| `UiTypography` | Definition | 配置文件 | 是 | 否 | 否 |
| `Theme` | Definition | 配置文件 | 是 | 否 | 否 |
| `BattleHudVm` | Runtime | 否 | 否 | 否 | 否 |
| `CharacterPanelVm` | Runtime | 否 | 否 | 否 | 否 |
| `SkillPanelVm` | Runtime | 否 | 否 | 否 | 否 |
| `InventoryVm` | Runtime | 否 | 否 | 否 | 否 |
| `ShopVm` | Runtime | 否 | 否 | 否 | 否 |
| `QuestLogVm` | Runtime | 否 | 否 | 否 | 否 |
| `NotificationVm` | Runtime | 否 | 否 | 否 | 否 |
| `ModalVm` | Runtime | 否 | 否 | 否 | 否 |
| `UiStore` | Runtime | 否 | 否 | 否 | 否 |
| `Dirty<T>` | Runtime | 否 | 否 | 否 | 否 |
| `Focusable` | Runtime | 否 | 否 | 否 | 否 |
| `FocusGroup` | Runtime | 否 | 否 | 否 | 否 |
| `UiSettings` | Persistence | SettingsGroup | 否 | 是 | 否 |
| `ScreenStack` | Persistence | Resource | 否 | 是（当前 Screen） | 否 |
| `UiCommand` | Runtime | Event | 否 | 否 | 部分（逻辑命令） |
| `UiAction` | Runtime | Event | 否 | 否 | 否 |
| `UiBinding` | Runtime | Component | 否 | 否 | 否 |
| `WidgetFactory` | Runtime | Component | 否 | 否 | 否 |

---

## 14. Dependency Analysis

### 14.1 依赖的 Schema

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → LocalizationSchema | UiTextKey 类型定义、本地化查询 |
| 依赖 | → CueSchema | Popup 类型 Cue 的 UI 反馈 |
| 依赖 | → EventSchema | Domain Event → Projection 的输入 |
| 依赖 | → InputSchema | InputAction → UiAction 的映射 |
| 依赖 | → AttributeSchema | StatsVm 中的属性值来源 |
| 依赖 | → AbilitySchema | SkillPanelVm 中的技能信息来源 |
| 依赖 | → EffectSchema | BuffVm 中的 Buff 信息来源 |
| 依赖 | → InventorySchema | InventoryVm 中的物品信息来源 |
| 依赖 | → QuestSchema | QuestLogVm 中的任务信息来源 |
| 依赖 | → EconomySchema | ShopVm 中的商品信息来源 |
| 依赖 | → SaveSchema | UiSettings / ScreenStack 的持久化 |

### 14.2 被依赖

| 被依赖方向 | 被依赖 Schema | 说明 |
|-----------|--------------|------|
| 被依赖 | ← 所有 Domain Schema | 各 Domain 通过 Event 驱动 ViewModel 更新 |

### 14.3 依赖约束

```
UI Presentation (L3)
    ↓ 只读
Capabilities (L1) — 通过 Event 读取
    ↓ 不依赖
Infrastructure (L2) — 通过 SettingsGroup / Resource 读取
```

**禁止**：UI Presentation 直接修改 Capabilities 或 Infrastructure 的数据。

---

## 15. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | UiTextKey 格式合法 | 编译期 / 加载期 | 匹配 `ui\.[a-z0-9_]+\.[a-z0-9_]+\.[a-z0-9_]+` |
| V2 | AssetKey 非空 | Def 加载 | 字符串长度 > 0 |
| V3 | ViewModel 数值范围 | Projection 更新 | hp <= max_hp, mp <= max_mp, exp <= exp_to_next |
| V4 | ScreenStack 深度限制 | Push 操作 | screens.len() < max_depth |
| V5 | Modal 按钮数量 | Modal 创建 | 1 <= buttons.len() <= 3 |
| V6 | Notification duration 正值 | Notification 创建 | duration_secs > 0.0 |
| V7 | BattleSpeed 范围 | Settings 修改 | 0.5 <= battle_speed <= 3.0 |
| V8 | TooltipDelay 非负 | Settings 修改 | tooltip_delay_secs >= 0.0 |
| V9 | FocusGroup 内 Focusable 非空 | Screen 构建 | group 内至少 1 个 Focusable |
| V10 | UiSettings 字段类型实现 Reflect | 编译期 | 所有字段类型 derive Reflect |
| V11 | ViewModel 不直接引用 Domain Component | 代码审查 | ViewModel 字段类型不含 Domain Component |
| V12 | UiCommand 不包含业务逻辑 | 代码审查 | UiCommand 是纯数据命令，不含执行逻辑 |

---

## 16. Replay Compatibility

### 16.1 原则

UI 数据不影响游戏逻辑回放。Replay 只录制 Domain 级别的 Command（如 `CastSkill`, `EndTurn`），UI 从 Domain Event 重新投影。

### 16.2 各数据类型的 Replay 兼容性

| 数据类型 | 进入 Replay | 说明 |
|---------|------------|------|
| ViewModel | 否 | 从 Domain Event 重新投影，无需录制 |
| UiState | 否 | 交互瞬态，不影响逻辑 |
| UiSettings | 否 | 用户偏好不影响回放结果 |
| StyleToken | 否 | 纯视觉，不影响逻辑 |
| ScreenStack | 否 | 导航状态不影响逻辑 |
| UiCommand（逻辑类） | 是 | `CastSkill`, `EndTurn` 等影响逻辑的命令 |
| UiCommand（UI 类） | 否 | `OpenScreen`, `CloseScreen` 不影响逻辑 |
| UiAction | 否 | Widget 级别动作，不进入 Replay |

### 16.3 确定性保证

| 要求 | 实现 |
|------|------|
| ViewModel 投影确定 | Projection 是纯函数，相同 Domain Event → 相同 ViewModel |
| 通知排序确定 | Notification 按优先级 + 时间戳排序，不依赖系统随机 |
| 模态弹窗确定 | Modal 由 Domain Event 触发，相同事件 → 相同弹窗 |
| 焦点导航确定 | FocusNavigation 是确定性算法，相同输入 → 相同焦点移动 |

---

## 17. Save Compatibility

### 17.1 进入 Save 的数据

| 数据 | Save 格式 | 说明 |
|------|----------|------|
| UiSettings | SettingsGroup 自动序列化 | Bevy 0.19 原生支持 |
| ScreenStack.screens（当前 Screen） | Reflect 序列化 | 只保存当前页面，不保存完整栈 |
| ThemeName | 随 UiSettings 保存 | 主题选择持久化 |

### 17.2 不进入 Save 的数据

| 数据 | 原因 |
|------|------|
| ViewModel | 从 Domain 重新投影，无需保存 |
| UiState | 交互瞬态，加载后重建 |
| Notification 队列 | 瞬时信息，加载后清空 |
| Modal 栈 | 瞬时交互，加载后清空 |
| FocusState | 加载后重新计算 |
| Dirty Flag | 加载后全部标记为脏，触发首次刷新 |

### 17.3 版本迁移

| 迁移场景 | 策略 |
|---------|------|
| UiSettings 新增字段 | SettingsGroup 自动使用 Default 值填充 |
| UiSettings 删除字段 | Reflect 反序列化自动忽略未知字段 |
| UiSettings 字段类型变更 | 需要显式迁移函数，在 UiPlugin 中注册 |
| ViewModel 结构变更 | 无需迁移（每次从 Domain 重新生成） |
| ScreenType 枚举新增 | 旧存档中的 ScreenType 反序列化失败时回退到 MainMenu |

### 17.4 Save 版本号

UiSettings 不独立版本化，随游戏主存档版本号迁移。ViewModel 不参与版本控制。

---

## 18. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2（未来） | 新增 MiniMapVm | UiStore 新增字段，Default 自动填充 |
| v3（未来） | 新增 DialogueVm | UiStore 新增字段，Default 自动填充 |
| v4（未来） | BattleSpeed 从 f32 改为枚举 | 显式迁移函数：f32 → 枚举映射 |
| v5（未来） | 新增 Theme 自定义颜色覆盖 | UiSettings 新增字段，Default 自动填充 |

**迁移原则**：
- 新增字段：依赖 `Default` trait 自动填充，无需显式迁移
- 删除字段：Reflect 反序列化自动忽略，无需显式迁移
- 类型变更：必须提供显式迁移函数

---

## 19. Future Extension

| 扩展点 | 说明 | 影响 |
|--------|------|------|
| MiniMapVm | 小地图视图模型 | UiStore 新增字段 |
| DialogueVm | 对话系统视图模型 | UiStore 新增字段 |
| CraftingVm | 制作系统视图模型 | UiStore 新增字段 |
| TutorialVm | 教程系统视图模型 | UiStore 新增字段 |
| AchievementVm | 成就系统视图模型 | UiStore 新增字段 |
| Theme 自定义 | 用户自定义颜色覆盖 | UiSettings 新增 custom_colors 字段 |
| 响应式布局 | 根据分辨率自动调整 Spacing | UiSpacing 新增 breakpoint 字段 |
| 动画主题 | Theme 支持过渡动画 | Theme 新增 transition_config 字段 |
| 无障碍 | 高对比度模式、字体缩放 | UiSettings 新增 accessibility 字段 |
| 多窗口 | 支持多 Screen 同时显示 | ScreenStack 改为多栈结构 |

---

## 20. Risks

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| UiStore 过大 | 所有 ViewModel 集中在一个 Resource 中，内存占用高 | 按需加载：非活跃 Screen 的 ViewModel 置为 Default |
| Projection 性能 | Domain Event 频繁触发时，Projection 可能成为瓶颈 | 增量投影：只更新变化的 ViewModel 字段 |
| Dirty 泛型注册 | 每个 ViewModel 类型都需要 register_type，容易遗漏 | 编译期宏自动生成注册代码 |
| SettingsGroup 字段类型限制 | 所有字段必须实现 Reflect + Serialize + DeserializeOwned | 自定义类型需补齐这三个 derive |
| ScreenStack Entity 失效 | 存档加载后 ScreenInfo.entity 指向已销毁的 Entity | 加载后重建 Screen Entity，不依赖持久化的 Entity |
| ViewModel 与 Domain 不同步 | Projection 延迟导致 UI 显示过时数据 | 使用 ChangeDetection 确保 Projection 在 Domain 更新后立即执行 |
| Notification 风暴 | 同一帧大量 Notification 导致 UI 卡顿 | 设置每帧最大 Notification 数量，超出则排队 |

---

## 21. Constitution Check

| 宪法条款 / Data Law | 合规 | 说明 |
|---------------------|------|------|
| Data Law 001: Def-Instance 强制分离 | 合规 | StyleToken/Theme 是 Definition，ViewModel 是 Runtime，UiSettings 是 Persistence，三层清晰分离 |
| Data Law 002: Rule-Content 强制分离 | 合规 | ViewModel 不含业务逻辑，Projection 是纯函数 |
| Data Law 003: 配置只引用 ID | 合规 | ViewModel 中使用 SkillId/ItemId/BuffId 等 ID 引用，不嵌入 Definition |
| Data Law 009: 表现必须经过 Cue | 合规 | UI 反馈（伤害数字、状态文字）通过 Cue Popup 触发 |
| Data Law 010: Replay 优先于便利 | 合规 | UI 数据不进入 Replay，逻辑命令进入 Replay |
| Data Law 011: Schema 必须版本化 | 合规 | UiSettings 随游戏版本迁移，ViewModel 无需版本化 |
| Data Law 012: 域间禁止直接数据引用 | 合规 | UI 通过 Event 接收 Domain 数据，通过 UiCommand 发送操作 |
| Data Law 013: 用户可见文本必须使用 LocalizationKey | 合规 | 所有 ViewModel 文本字段使用 UiTextKey |
| 宪法 §22: Localization First | 合规 | ViewModel 只存 UiTextKey，不存自然语言文本 |
| Logic/Presentation Separation | 合规 | ViewModel 是单向投影，UI 不直接修改 Domain |

---

## 22. Reflect 注册清单

所有 ViewModel、StyleToken、UiSettings 必须在 UiPlugin 中注册：

```rust
// UiPlugin::build 中
app.register_type::<UiStore>()
    .register_type::<BattleHudVm>()
    .register_type::<BattlePhaseVm>()
    .register_type::<CharacterPanelVm>()
    .register_type::<BuffVm>()
    .register_type::<StatsVm>()
    .register_type::<SkillPanelVm>()
    .register_type::<SkillSlotVm>()
    .register_type::<InventoryVm>()
    .register_type::<InventorySlotVm>()
    .register_type::<InventoryFilterVm>()
    .register_type::<InventorySortOrder>()
    .register_type::<RarityVm>()
    .register_type::<ShopVm>()
    .register_type::<ShopSlotVm>()
    .register_type::<QuestLogVm>()
    .register_type::<QuestSlotVm>()
    .register_type::<QuestFilterVm>()
    .register_type::<QuestTypeVm>()
    .register_type::<NotificationVm>()
    .register_type::<NotificationPriorityVm>()
    .register_type::<NotificationTypeVm>()
    .register_type::<ModalVm>()
    .register_type::<ModalButtonVm>()
    .register_type::<ModalTypeVm>()
    .register_type::<ButtonStyleVm>()
    .register_type::<UiColors>()
    .register_type::<UiSpacing>()
    .register_type::<UiTypography>()
    .register_type::<Theme>()
    .register_type::<ThemeName>()
    .register_type::<UiSettings>()
    .register_type::<LanguageVm>()
    .register_type::<ScreenStack>()
    .register_type::<ScreenInfo>()
    .register_type::<ScreenType>()
    .register_type::<TransitionType>()
    .register_type::<Direction>()
    .register_type::<Focusable>()
    .register_type::<FocusGroup>()
    .register_type::<FocusNavigation>()
    .register_type::<UiBinding>()
    .register_type::<UiAction>();
```

**derive 要求汇总**：

| 类型 | 必须的 derive | 必须的 reflect 属性 |
|------|-------------|-------------------|
| Resource 类型 | `Resource, Reflect, Default` | `#[reflect(Resource, Default)]` |
| SettingsGroup 类型 | `Resource, SettingsGroup, Reflect, Default` | `#[reflect(Resource, SettingsGroup, Default)]` |
| Component 类型 | `Component, Reflect` | `#[reflect(Component)]` |
| 枚举 Component 类型 | `Component, Reflect, Clone, Copy, PartialEq` | `#[reflect(Component, PartialEq)]` |
| 普通 ViewModel | `Clone, Reflect, Default` | `#[reflect(Default)]` |
| 枚举类型 | `Clone, Reflect, Default, PartialEq` | `#[reflect(Default, PartialEq)]` |

---

## 23. UiBinding Schema（反 Marker 模式）

```rust
/// UI 绑定标识，替代大量独立 Marker 结构体
/// 50 万行项目如果每个 UI 元素一个 Marker，最终 400+ 结构体
/// 统一为枚举，一个 Archetype 搞定
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
pub enum UiBinding {
    // Battle HUD
    Hp,
    MaxHp,
    Mp,
    MaxMp,
    Ap,
    MaxAp,
    Turn,
    Phase,

    // Character Panel
    Level,
    Exp,
    Name,

    // Skill Panel
    SkillSlot(u8),  // 第 N 个技能槽
    Cooldown,

    // Inventory
    ItemSlot(u8),   // 第 N 个物品槽
    Gold,

    // Quest
    QuestEntry(u16), // 第 N 个任务条目

    // General
    Tooltip,
    Modal,
    Notification,
}
```

**设计决策**：
- 使用枚举替代独立 Marker 结构体，避免 Archetype 膨胀
- 带参数变体（如 `SkillSlot(u8)`）支持动态数量的 UI 元素
- 单一 Archetype 查询 `Query<&UiBinding>` 即可覆盖所有 UI 绑定

---

## 24. UI Schema 治理

机器可读的 Widget Schema，存放在 `docs/ui_schema/`：

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

Schema 文件结构：

```
docs/ui_schema/
├── screens/
│   ├── battle_screen.yaml
│   ├── inventory_screen.yaml
│   ├── shop_screen.yaml
│   └── settings_screen.yaml
├── widgets/
│   ├── primary_button.yaml
│   ├── progress_bar.yaml
│   ├── skill_panel.yaml
│   ├── character_panel.yaml
│   └── tooltip.yaml
├── view_models/
│   ├── battle_hud_vm.yaml
│   ├── character_panel_vm.yaml
│   └── skill_panel_vm.yaml
└── contracts/
    └── widget_contract_template.yaml
```

**治理规则**：
- 每个新增 Widget 必须先写 Schema，再写代码
- AI 生成 UI 代码时必须遵守 Schema
- Schema 中 `prohibited` 字段定义了 Widget 禁止使用的查询/访问模式

---

## 25. WidgetFactory Schema

```rust
/// Widget 工厂 trait
/// 所有 Widget 实现此 trait，Screen 通过它组合 Widget
pub trait WidgetFactory: Component {
    type Vm: Reflect + Default;

    /// 从 ViewModel 创建 Widget 实体
    fn create(commands: &mut Commands, vm: &Self::Vm) -> Entity;

    /// 刷新 Widget（仅在 Dirty 时调用）
    fn refresh(entity: Entity, vm: &Self::Vm, query: &mut Query<&mut Self>);

    /// 销毁 Widget 实体
    fn destroy(commands: &mut Commands, entity: Entity);
}
```

WidgetFactory 实现清单：

| Widget | Vm 类型 | create 输出 | refresh 触发 |
|--------|---------|------------|-------------|
| PrimaryButton | UiTextKey | Button + LocalizedText | label 变更 |
| ProgressBar | f32 (ratio) | Node + BackgroundColor | ratio 变更 |
| SkillPanel | SkillPanelVm | Container + N×SkillButton | skills/selected/ap 变更 |
| CharacterPanel | CharacterPanelVm | Container + Stats + BuffList | hp/mp/buffs 变更 |
| Tooltip | TooltipVm | FloatingPanel + Text | content 变更 |
| Modal | ModalVm | OverlayPanel + Buttons | title/body 变更 |
| Notification | NotificationVm | ToastPanel + Text | message 变更 |

**设计决策**：
- WidgetFactory 是 Widget 的唯一创建/刷新/销毁入口
- `refresh` 仅在 `Dirty<T>` 为 true 时调用，避免无谓刷新
- Screen 通过组合 WidgetFactory 构建 UI 树，不直接操作 Entity

---

## 26. UI 与 Content 数据流

UI 不直接访问 DefRegistry，通过 Projection 防火墙：

```
Content (assets/config/*.ron)
    ↓ AssetServer 加载
DefRegistry<SpellDef> (Resource)
    ↓ Projection 查询
SkillPanelVm (UiStore 中)
    ↓ Dirty Flag
SkillPanel Widget
```

Projection 查询 Def 的合法模式：

```rust
fn project_skill_info(
    trigger: Trigger<AbilityUsed>,
    defs: Res<DefRegistry<SpellDef>>,  // Projection 可以读 Def
    mut store: ResMut<UiStore>,         // 写入 ViewModel
) {
    if let Some(def) = defs.get(trigger.ability_id) {
        // Def → ViewModel 投影
        store.skill_panel.update_from_def(def);
        store.skill_panel.mark_dirty();
    }
}
```

禁止模式：

```rust
// ❌ Widget 直接读 Def
fn update_skill_icon(defs: Res<DefRegistry<SpellDef>>) { ... }

// ❌ Widget 直接读 Content
fn load_skill_icon(assets: Res<AssetServer>) { ... }
```

**Modding 数据流**：

```
Mod → Content → DefRegistry → Projection → ViewModel → Widget
```

禁止 Mod 直接扩展 UI Widget。

**数据流约束**：
- Projection 是 Def → ViewModel 的唯一合法通道
- Widget 只读 ViewModel，不读 Def、不读 Content、不读 AssetServer
- Mod 扩展 Content 后，Projection 自动投影到 ViewModel，无需修改 UI 代码
