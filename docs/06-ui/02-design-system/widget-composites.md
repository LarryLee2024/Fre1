---
id: 06-ui.widget-composites
title: Widget Composites — 复合组件（Molecules / Organisms）
status: draft
owner: presentation-architect
created: 2026-06-20
tags:
  - ui
  - widget
  - composite
  - molecule
  - organism
  - composition
---

# Widget Composites — 复合组件（Molecules / Organisms）

> **职责**: @presentation-architect | **上游**: 02-design-system/widget-atoms.md（原子组件契约），03-screens/screens.md（Screen 组合需求），03-screens/screen-lifecycle.md §3（Widget 生命周期），ADR-055 §5.4（Widget Contract 模式），ADR-055 §8（Widget 分类）
>
> **架构隔离规则**：Composites（`widgets/`）只能引用 Primitives 层（`primitives/`）的组件和工厂函数。
> 禁止直接 import Bevy UI 类型（Node、Button、Interaction、BackgroundColor 等）。
> 违反此规则会导致 UI 重构时波及范围失控。

---

## 1. 设计目的

原子组件（Atoms）是 UI 的最小构建块（Button、ProgressBar、Text、Panel 等），但在 SRPG 的实际页面中，UI 元素从来不是单个原子组件出现的——它们以可识别的模式组合出现：角色头像区域、技能槽位、物品行、回合指示器。

本文档定义了介于**原子组件（Atoms）**和**页面（Screens）**之间的复合组件层：

| 层级 | 定义 | 复用范围 | 内部组成 |
|------|------|---------|---------|
| **Atom** | 不可再分的最小 UI 元素 | 全局 | 独立渲染逻辑 |
| **Molecule** | 3-5 个 Atom 的逻辑组合 | 多个 Screen 间复用 | 仅由 Atom 组成 |
| **Organism** | 多个 Molecule + Atom 的有机组合 | 特定 Screen 内复用 | 由 Molecule + Atom 组成 |
| **Screen** | 完整页面，与 GameState 对应 | 唯一实例 | 由 Organism + Molecule + Atom 组成 |

**分层目标**：

1. 消除 Screen → Atom 的"跳跃式引用"——BattleScreen 现在引用 BattleHud（Organism），而非直接引用 15 个分散的 Atom
2. 提供可复用的"UI 中间件"——SkillSlot（Molecule）在 BattleScreen 和 PartySetupScreen 中共同使用
3. 明确组合边界——每个复合组件拥有自己的 ViewModel、内部状态和自己负责的交互逻辑

### 1.1 复合组件 Contract 模板

每个复合组件遵循与原子组件相同的 Contract 模式，但加上"组成"字段：

```
[Name]
  Composes:   [引用的子组件，来源：widget-atoms.md §X]
  Props:      [输入数据字段]
  Layout:     [布局策略]
  Events:     [UiAction 向上传递]
  Local State:[内部状态]
  ViewModel:  [消费的 ViewModel]
  Used in:    [使用这个复合组件的 Screen]
```

### 1.2 复合组件与 ViewModel 的对应

复合组件不直接消费领域 ViewModel，而是消费**复合组件级 ViewModel**。每个复合组件对应的 ViewModel 在 `04-data-flow/projection-viewmodel.md` 中定义，由对应的 Projection 负责从 Domain Event 投影。

```
Domain Event → Projection → CompositeVm → Composite Widget

示例：
CombatTurnStarted → TurnProjection → TurnOrderBarVm → TurnOrderBar
ItemAcquired       → InventoryProjection → InventoryItemRowVm → InventoryItemRow
```

---

## 2. Molecules（分子级复合组件）

Molecule 是 3-5 个原子组件（Atom）的**逻辑组合**，体现一个独立的数据展示单元：
- 可以在多个 Screen 间复用
- 有自己的 Props 输入和 Events 输出
- 内部布局固定，对外不暴露内部原子组件细节
- 不直接引用 Domain 数据，仅消费 ViewModel

### 2.1 SkillSlot

**用途**: 角色技能快捷栏中的一个技能槽位，展示技能图标、名称、冷却状态和消耗。

**Composes**:
| 原子组件 | 来源 | 用途 |
|---------|------|------|
| IconButton | widget-atoms.md §2.4 | 技能图标，点击触发技能选择 |
| CaptionText (LocalizedText 变体) | widget-atoms.md §6.1 | 技能名称标签 |
| ProgressBar | widget-atoms.md §3.1 | 冷却覆盖层（当前回合剩余/总冷却回合） |
| CaptionText (LocalizedText 变体) | widget-atoms.md §6.1 | 消耗数值显示（AP/MP/Special） |

**Props**:

| 字段 | 类型 | 说明 |
|------|------|------|
| skill_id | SkillId | 技能 ID，非 Entity |
| name_key | UiTextKey | 技能名称本地化 Key |
| icon_key | AssetKey | 技能图标资源 Key |
| cooldown_remaining | u32 | 剩余冷却回合数（0 = 可用） |
| cooldown_total | u32 | 总冷却回合数 |
| ap_cost | u32 | AP 消耗 |
| mp_cost | u32 | MP 消耗 |
| enabled | bool | 是否可交互（有足够资源、非冷却、在射程内） |
| is_selected | bool | 是否被玩家选中 |

**Layout**: 垂直排列。上层为图标按钮（居中），图标右下角叠加冷却覆盖层（ProgressBar，从满到空表示冷却进度）。下方第一行为技能名称（CaptionText，截断）。下方第二行为消耗文本（CaptionText，格式 "AP:3 MP:5"）。

**Events**:

| 事件 | 负载 | 触发条件 |
|------|------|---------|
| UiAction::SelectSkill | SkillId | 点击技能图标且 enabled=true |
| UiAction::ShowSkillTooltip | SkillId | 悬停技能图标超过 0.3s |

**Local State**:

| 状态 | 类型 | 初始值 | 说明 |
|------|------|--------|------|
| hovered | bool | false | 悬停状态 |
| cooldown_anim_progress | f32 | 0.0 | 冷却动画插值（0→1 每帧更新） |

**ViewModel**: `SkillSlotVm`（定义于 projection-viewmodel.md）

**Used in**: BattleScreen（技能面板），PartySetupScreen（查看技能），CharacterDetailScreen（技能列表）

---

### 2.2 CharacterPortrait

**用途**: 角色头像区域，展示头像图片、名称、HP 状态和状态效果图标。

**Composes**:
| 原子组件 | 来源 | 用途 |
|---------|------|------|
| Panel (CardPanel 变体) | widget-atoms.md §4.1 | 头像区域背景，带轻微阴影 |
| Image | — (Bevy Image) | 角色头像图片 |
| BodyText (LocalizedText 变体) | widget-atoms.md §6.1 | 角色名称 |
| HpBar (ProgressBar 变体) | widget-atoms.md §3.1 | HP 条，红色填充 |
| Panel (TransparentPanel 变体) | widget-atoms.md §4.1 | 状态图标容器，水平排列 |

**Props**:

| 字段 | 类型 | 说明 |
|------|------|------|
| character_id | CharacterId | 角色 ID，非 Entity |
| name_key | UiTextKey | 角色名称 Key |
| portrait_key | AssetKey | 头像资源 Key |
| hp_current | u32 | 当前 HP |
| hp_max | u32 | 最大 HP |
| status_icons | Vec\<BuffVm\> | 激活的 Buff/Debuff 图标数据 |
| is_active_turn | bool | 是否当前行动角色 |
| is_selected | bool | 是否被选为目标 |

**Layout**: 水平排列。左侧为头像图片（Panel 内嵌，圆形裁剪或圆角方形），头像右下角叠加选中高亮边框（is_selected 时显示）。右侧垂直排列：第一行角色名称（BodyText），第二行 HP 条（HpBar），第三行状态图标行（最多 5 个 StatusIcon，超出隐藏）。

**Events**:

| 事件 | 负载 | 触发条件 |
|------|------|---------|
| UiAction::SelectCharacter | CharacterId | 点击头像区域（战斗中选择目标） |
| UiAction::ShowCharacterDetail | CharacterId | 双击头像区域 |

**Local State**: 无（纯展示，交互委托给上层 Organism）

**ViewModel**: `CharacterPortraitVm`（定义于 projection-viewmodel.md）

**Used in**: BattleScreen（角色面板），PartySetupScreen（队伍编成），InventoryScreen（选中角色装备）

---

### 2.3 InventoryItemRow

**用途**: 背包/商店中一行的物品条目，展示图标、名称、数量和操作按钮。

**Composes**:
| 原子组件 | 来源 | 用途 |
|---------|------|------|
| IconButton | widget-atoms.md §2.4 | 物品图标 |
| BodyText (LocalizedText 变体) | widget-atoms.md §6.1 | 物品名称 |
| CaptionText (LocalizedText 变体) | widget-atoms.md §6.1 | 物品数量文本（如 "×3"） |
| Panel (TransparentPanel 变体) | widget-atoms.md §4.1 | 稀有度边框或背景着色 |
| IconButton | widget-atoms.md §2.4 | 操作按钮（使用/装备/出售，由上下文决定） |

**Props**:

| 字段 | 类型 | 说明 |
|------|------|------|
| item_id | ItemId | 物品 ID，非 Entity |
| name_key | UiTextKey | 物品名称 Key |
| icon_key | AssetKey | 物品图标 Key |
| quantity | u32 | 持有数量 |
| rarity | RarityVm | 稀有度枚举（Common/Uncommon/Rare/Epic/Legendary） |
| action_type | ItemActionVm | 行操作类型（Use/Equip/Sell/None） |
| is_selected | bool | 是否被选中 |
| price | Option\<u32\> | 商店中显示的价格（非商店上下文为 None） |

**Layout**: 水平排列。最左侧为物品图标（IconButton），图标左侧边框按稀有度着色。中间垂直排列：第一行物品名称（BodyText），第二行数量文本（CaptionText，灰色）。最右侧为操作按钮（IconButton，上下文相关），商店上下文中价格显示在名称和数量右侧。

**Events**:

| 事件 | 负载 | 触发条件 |
|------|------|---------|
| UiAction::SelectItem | ItemId | 点击行主体 |
| UiAction::UseItem | ItemId | 点击"使用"动作按钮 |
| UiAction::EquipItem | ItemId | 点击"装备"动作按钮 |
| UiAction::ShowContextMenu | (ItemId, Vec2) | 右键点击行主体 |

**Local State**:

| 状态 | 类型 | 初始值 | 说明 |
|------|------|--------|------|
| hovered | bool | false | 悬停背景高亮 |
| pressed | bool | false | 按下状态 |

**ViewModel**: `InventoryItemRowVm`（定义于 projection-viewmodel.md）

**Used in**: InventoryScreen（背包列表），ShopScreen（出售列表），LootScreen（战利品拾取）

---

### 2.4 QuestEntry

**用途**: 任务日志中的一项任务条目，展示标题、进度条、奖励和状态。

**Composes**:
| 原子组件 | 来源 | 用途 |
|---------|------|------|
| Panel (CardPanel 变体) | widget-atoms.md §4.1 | 条目背景面板 |
| BodyText (LocalizedText 变体) | widget-atoms.md §6.1 | 任务标题 |
| ProgressBar | widget-atoms.md §3.1 | 任务进度条（如 "3/5 杀敌"） |
| CaptionText (LocalizedText 变体) | widget-atoms.md §6.1 | 奖励列表文本（短格式） |
| Panel (TransparentPanel 变体) | widget-atoms.md §4.1 | 状态标签（进行中/已完成/已放弃） |

**Props**:

| 字段 | 类型 | 说明 |
|------|------|------|
| quest_id | QuestId | 任务 ID |
| title_key | UiTextKey | 任务标题 Key |
| description_key | UiTextKey | 任务描述 Key（折叠时隐藏） |
| progress_current | u32 | 当前进度 |
| progress_total | u32 | 总进度 |
| rewards_key | UiTextKey | 奖励文本 Key（如 "奖励: 金币×500, EXP×1000"） |
| status | QuestStatusVm | 任务状态（Active/Completed/Abandoned） |
| is_expanded | bool | 是否展开显示详情 |

**Layout**: 垂直排列。顶部为标题(BodyText) + 状态标签(Panel + CaptionText，右上角)，颜色由状态决定：Active=蓝色，Completed=绿色，Abandoned=灰色。中间为进度条(ProgressBar)，下方为奖励文本(CaptionText，灰色)。展开时额外显示描述文本(BodyText)。

**Events**:

| 事件 | 负载 | 触发条件 |
|------|------|---------|
| UiAction::SelectQuest | QuestId | 点击条目主体 |
| UiAction::ToggleQuestExpand | QuestId | 点击展开/折叠图标 |
| UiAction::TrackQuest | QuestId | 点击"追踪"按钮（设为当前追踪任务） |

**Local State**:

| 状态 | 类型 | 初始值 | 说明 |
|------|------|--------|------|
| expanded | bool | false | 展开/折叠状态 |

**ViewModel**: `QuestEntryVm`（定义于 projection-viewmodel.md）

**Used in**: QuestLogScreen（任务日志），HudOverlay（追踪任务显示）

---

### 2.5 DialogueChoice

**用途**: 对话系统中的单个选项按钮，展示选项文本和选中状态。

**Composes**:
| 原子组件 | 来源 | 用途 |
|---------|------|------|
| Panel (CardPanel 变体) | widget-atoms.md §4.1 | 选项背景面板 |
| BodyText (LocalizedText 变体) | widget-atoms.md §6.1 | 选项文本 |
| IconButton | widget-atoms.md §2.4 | 选中指示图标（圆圈/勾选） |

**Props**:

| 字段 | 类型 | 说明 |
|------|------|------|
| choice_id | ChoiceId | 选项 ID |
| text_key | UiTextKey | 选项文本 Key |
| text_params | HashMap\<String, String\> | 文本参数插值 |
| is_selected | bool | 当前是否被选中（键盘/手柄导航时） |
| is_available | bool | 是否可选（由条件决定，false 时灰色显示） |
| requirement_hint | Option\<UiTextKey\> | 不可选时的原因提示 Key |

**Layout**: 水平排列。左侧为选中指示图标(IconButton，选中时填充圆，未选中时空心圆)。右侧为选项文本(BodyText)。不可选时整体半透明并显示原因提示文字(CaptionText)在文本右侧。

**Events**:

| 事件 | 负载 | 触发条件 |
|------|------|---------|
| UiAction::SelectChoice | ChoiceId | 点击选项且 is_available=true |
| UiAction::ConfirmChoice | ChoiceId | 键盘 Enter / 手柄 A 确认选中项 |

**Local State**:

| 状态 | 类型 | 初始值 | 说明 |
|------|------|--------|------|
| hovered | bool | false | 悬停背景高亮 |
| navigate_hint_alpha | f32 | 0.0 | 导航提示动画（选中时闪烁） |

**ViewModel**: `DialogueChoiceVm`（定义于 projection-viewmodel.md）

**Used in**: DialogueOverlay（对话系统）

---

### 2.6 ShopItemCard

**用途**: 商店中的一个商品卡片，展示物品图标、名称、价格和折扣信息。

**Composes**:
| 原子组件 | 来源 | 用途 |
|---------|------|------|
| Panel (CardPanel 变体) | widget-atoms.md §4.1 | 卡片背景 |
| IconButton | widget-atoms.md §2.4 | 物品图标 |
| CaptionText (LocalizedText 变体) | widget-atoms.md §6.1 | 物品名称 |
| StatText (LocalizedText 变体) | widget-atoms.md §6.1 | 价格文本 |
| PrimaryButton | widget-atoms.md §2.1 | 购买按钮 |
| Panel (TransparentPanel 变体) | widget-atoms.md §4.1 | 折扣徽章（可选） |

**Props**:

| 字段 | 类型 | 说明 |
|------|------|------|
| item_id | ItemId | 商品 ID |
| item_icon | AssetKey | 物品图标 Key |
| item_name_key | UiTextKey | 物品名称 Key |
| item_name_params | HashMap\<String, String\> | 名称参数插值 |
| price | u32 | 当前价格（折扣后） |
| original_price | Option\<u32\> | 原价（有折扣时 != price） |
| stock | u32 | 剩余库存 |
| stock_max | u32 | 最大库存 |
| player_can_afford | bool | 玩家是否买得起 |
| discount_pct | Option\<u32\> | 折扣百分比（如 20 表示 20% off） |

**Layout**: 垂直排列。顶部为物品图标(IconButton，居中)，图标右上角叠加折扣徽章(Panel + CaptionText，如 "-20%"，红色背景)。下方为物品名称(CaptionText，截断)。再下方为价格行：有折扣时显示原价(删除线) + 现价(StatText，红色)，无折扣时仅显示价格(StatText)。底部为购买按钮(PrimaryButton，禁用态由 player_can_afford 控制)。卡片右下角显示库存文本(CaptionText，灰色)。

**Events**:

| 事件 | 负载 | 触发条件 |
|------|------|---------|
| UiAction::SelectShopItem | ItemId | 点击卡片主体 |
| UiAction::BuyItem | ItemId | 点击购买按钮且 player_can_afford=true |
| UiAction::ShowItemTooltip | ItemId | 悬停图标超过 0.3s |

**Local State**:

| 状态 | 类型 | 初始值 | 说明 |
|------|------|--------|------|
| hovered | bool | false | 悬停缩放效果 |
| stock_anim | f32 | 0.0 | 库存变更动画 |

**ViewModel**: `ShopItemCardVm`（定义于 projection-viewmodel.md）

**Used in**: ShopScreen（商店面板）

---

### 2.7 BuffIcon

**用途**: Buff/Debuff 状态图标，显示状态效果和剩余回合数。

**Composes**:
| 原子组件 | 来源 | 用途 |
|---------|------|------|
| IconButton | widget-atoms.md §2.4 | 状态效果图标 |
| SuperCaptionText (LocalizedText 变体) | widget-atoms.md §6.1 | 剩余回合数（极小学号，图标右下角叠加） |
| TooltipTrigger | — | 悬停触发详细说明 |

**Props**:

| 字段 | 类型 | 说明 |
|------|------|------|
| buff_id | BuffId | Buff/Debuff ID |
| icon_key | AssetKey | 效果图标 Key |
| is_debuff | bool | 是否为减益效果（影响边框颜色） |
| remaining_turns | u32 | 剩余持续回合数 |
| max_turns | u32 | 总持续回合数 |
| name_key | UiTextKey | 效果名称 Key（用于 Tooltip） |
| description_key | UiTextKey | 效果描述 Key（用于 Tooltip） |
| stack_count | Option\<u32\> | 叠层数（可叠加的 Buff 显示层数） |

**Layout**: 方形图标(IconButton)。图标右下角叠加回合数文本(极小学号，白色，有底部阴影)。buff 边框为蓝色，debuff 边框为红色。有叠层时左下角显示层数徽章(圆形 CaptionText)。悬停时触发 TooltipOverlay 显示名称和描述。

**Events**:

| 事件 | 负载 | 触发条件 |
|------|------|---------|
| UiAction::ShowBuffTooltip | BuffId | 悬停图标超过 0.3s |

**Local State**:

| 状态 | 类型 | 初始值 | 说明 |
|------|------|--------|------|
| hovered | bool | false | 悬停状态 |
| pulse_anim | f32 | 0.0 | 剩余 1 回合时闪烁动画 |

**ViewModel**: `BuffVm`（定义于 projection-viewmodel.md）

**Used in**: BattleScreen（状态栏），CharacterStatusPanel，CharacterDetailScreen

---

### 2.8 TurnIndicator

**用途**: 回合顺序指示器中的一个角色条目，显示角色头像、活跃状态和 AP 指示。

**Composes**:
| 原子组件 | 来源 | 用途 |
|---------|------|------|
| IconButton | widget-atoms.md §2.4 | 角色头像图标 |
| Panel (TransparentPanel 变体) | widget-atoms.md §4.1 | 活跃高亮边框 |
| CaptionText (LocalizedText 变体) | widget-atoms.md §6.1 | AP 指示点（"● ● ●"或"○ ● ●"） |

**Props**:

| 字段 | 类型 | 说明 |
|------|------|------|
| character_id | CharacterId | 角色 ID |
| portrait_key | AssetKey | 头像图标 Key |
| name_key | UiTextKey | 角色名 Key（Tooltip 显示） |
| is_active | bool | 是否当前行动角色 |
| is_next | bool | 是否是下一个行动角色 |
| ap_remaining | u32 | 剩余 AP |
| ap_max | u32 | 最大 AP |
| faction | FactionVm | 阵营（Player/Enemy/Ally，影响边框颜色） |

**Layout**: 垂直排列。上方为头像图标(IconButton，圆形)，is_active 时外围高亮光环(Panel，金色边框+发光效果)，is_next 时浅色边框。头像下方为 AP 指示点(CaptionText，实心圆=剩余，空心圆=已消耗，水平排列)。最下方为角色名称在 Tooltip 中显示。阵营通过头像边框颜色区分：玩家=蓝色，敌方=红色，友方=绿色。

**Events**: 无（纯展示组件，交互由 TurnOrderBar 管理）

**Local State**: 无

**ViewModel**: `TurnIndicatorVm`（定义于 projection-viewmodel.md）

**Used in**: TurnOrderBar（Organism），BattleScreen

---

## 3. Organisms（有机体级复合组件）

Organism 是多个 Molecule + Atom 的**有机组合**，体现一个功能完整的 UI 区域：
- 通常在特定 Screen 内使用（或少数相关 Screen 间复用）
- 包含内部布局协调逻辑和子组件通信
- 可以管理子组件的可见性和排列
- 是 Screen 组合的基本单元——Screen 现在引用 Organism，而非直接管理 20+ 个 Atom

### 3.1 SkillPanel

**用途**: 技能面板，包含技能分类标签页、技能列表和滚动容器，是战斗/编成中技能选择的入口。

**Composes**:

| 子组件 | 类型 | 数量 | 来源 |
|-------|------|------|------|
| SkillSlot | Molecule | 多行 | §2.1 本文 |
| TabPanel | Atom | 1 | widget-atoms.md §4.3 |
| ScrollPanel | Atom | 1 | widget-atoms.md §4.2 |

**Props**:

| 字段 | 类型 | 说明 |
|------|------|------|
| skill_panel_id | SkillPanelId | 技能面板 ID |
| categories | Vec\<SkillCategoryDef\> | 技能分类定义（标签名 + 技能列表） |
| default_category | usize | 默认选中的分类索引 |
| ap_remaining | u32 | 当前剩余 AP（控制技能是否可施放） |
| selected_skill | Option\<SkillId\> | 当前选中的技能 |
| is_enabled | bool | 面板是否可交互（非玩家回合时禁用） |
| max_visible_slots | u32 | 最大可见技能槽位数（超出滚动） |

**SkillCategoryDef**:

| 字段 | 类型 | 说明 |
|------|------|------|
| label_key | UiTextKey | 分类标签 Key |
| skills | Vec\<SkillSlotVm\> | 该分类下的技能列表 |

**Layout**: 垂直排列。顶部为 TabPanel 的标签栏（水平排列的分类按钮：主动技能/被动技能/道具/特殊）。下方为 ScrollPanel 包裹的技能列表。技能列表内部为垂直排列的 SkillSlot（每个 SkillSlot 一行）。当前选中技能高亮边框。不可用的技能（AP 不足、冷却中、射程外）显示为灰色禁用态。

**Events**:

| 事件 | 负载 | 触发条件 |
|------|------|---------|
| UiAction::SelectSkill | SkillId | 点击 SkillSlot |
| UiAction::CastSkill | SkillId | 选择技能后点击确认（或双击技能图标） |
| UiAction::ChangeSkillCategory | usize | 切换分类标签页 |

**Local State**:

| 状态 | 类型 | 初始值 | 说明 |
|------|------|--------|------|
| active_category | usize | default_category | 当前激活的分类索引 |
| scroll_offset | f32 | 0.0 | 技能列表滚动偏移 |

**ViewModel**: `SkillPanelVm`（定义于 projection-viewmodel.md）

**Used in**: BattleScreen（战斗技能面板），PartySetupScreen（队伍编成技能配置），CharacterDetailScreen（角色技能浏览）

**错误处理**:
- 空分类：显示 CaptionText "该分类下无技能"（本地化 Key: ui.skill.no_skills_in_category）
- 全部技能冷却/不可用：禁用所有 SkillSlot，在面板顶部显示提示 CaptionText
- AP=0：禁用所有消耗 AP 的技能，SkillSlot 显示"AP不足"提示

---

### 3.2 CharacterStatusPanel

**用途**: 角色状态面板，集中显示角色名称、HP/MP/AP、Buff 列表和状态文本。

**Composes**:

| 子组件 | 类型 | 数量 | 来源 |
|-------|------|------|------|
| CharacterPortrait | Molecule | 1 | §2.2 本文 |
| MpBar (ProgressBar 变体) | Atom | 1 | widget-atoms.md §3.1 |
| ApBar (ProgressBar 变体) | Atom | 1 | widget-atoms.md §3.1 |
| BuffIcon | Molecule | 多行 | §2.7 本文 |
| BodyText (LocalizedText 变体) | Atom | 1 | widget-atoms.md §6.1 |
| Panel (DarkPanel 变体) | Atom | 1 | widget-atoms.md §4.1 |

**Props**:

| 字段 | 类型 | 说明 |
|------|------|------|
| character_id | CharacterId | 角色 ID |
| name_key | UiTextKey | 角色名 Key |
| portrait_key | AssetKey | 头像 Key |
| hp_current | u32 | 当前 HP |
| hp_max | u32 | 最大 HP |
| mp_current | u32 | 当前 MP |
| mp_max | u32 | 最大 MP |
| ap_current | u32 | 当前 AP |
| ap_max | u32 | 最大 AP |
| buffs | Vec\<BuffVm\> | 激活的 Buff/Debuff |
| status_text | Option\<UiTextKey\> | 额外状态文本（如"移动中""待机"）|
| is_enemy | bool | 是否为敌方角色（影响布局尺寸） |
| is_active | bool | 是否为当前行动角色 |

**Layout**: 垂直排列（玩家角色）或精简水平排列（敌方角色）。

**玩家角色（全尺寸）**: 顶部为 CharacterPortrait（含头像、名称、HP 条、状态图标）。中部为 MP 条(MpBar)和 AP 条(ApBar)垂直排列。下部为 BuffIcon 行(水平排列，自动换行，最多 8 个图标)。最底部为状态文本(BodyText，居中，如"待机中")。

**敌方角色（精简）**: 水平排列。左侧为 CharacterPortrait(仅头像+HP 条，无名称和状态图标)。右侧为 BuffIcon(最多 3 个)。无 MP/AP 显示。

**Events**:

| 事件 | 负载 | 触发条件 |
|------|------|---------|
| UiAction::SelectCharacter | CharacterId | 点击角色头像区域 |
| UiAction::ShowCharacterDetail | CharacterId | 双击角色名区域 |
| UiAction::ShowBuffTooltip | BuffId | 悬停 BuffIcon 超过 0.3s |

**Local State**: 无

**ViewModel**: `CharacterStatusPanelVm`（定义于 projection-viewmodel.md）

**Used in**: BattleScreen（主要角色状态区），PartySetupScreen（队伍成员状态）

**错误处理**:
- Buff 数量超过显示限制（>8）：显示 "+N" 气泡图标表示更多 Buff
- MP 条隐藏（无 MP 的角色）：MP 条不渲染，不对齐
- HP=0：整体半透明，显示"已阵亡"状态覆盖

---

### 3.3 BattleHud

**用途**: 战斗 HUD，整合战斗中玩家需要的全部实时信息：角色状态、技能、回合顺序。

**Composes**:

| 子组件 | 类型 | 数量 | 来源 |
|-------|------|------|------|
| CharacterStatusPanel | Organism | 1-N（取决于队伍大小） | §3.2 本文 |
| SkillPanel | Organism | 1 | §3.1 本文 |
| TurnOrderBar | Organism | 1 | §3.4 本文 |
| Panel (DarkPanel 变体) | Atom | 多组 | widget-atoms.md §4.1 |

> **注意**: BattleHud 引用 TurnOrderBar，而 TurnOrderBar 的定义在 §3.4。这是 Organism 引用 Organism 的唯一特例——因为回合条和角色面板在布局上共属战斗 HUD 整体。禁止 Organism 之间任意交叉引用。

**Props**:

| 字段 | 类型 | 说明 |
|------|------|------|
| battle_hud_id | BattleHudId | 战斗 HUD ID |
| player_characters | Vec\<CharacterStatusPanelVm\> | 玩家队伍状态列表 |
| enemy_characters | Vec\<CharacterStatusPanelVm\> | 敌方可见角色状态列表（精简模式） |
| turn_order | TurnOrderBarVm | 回合顺序数据 |
| skill_panel | SkillPanelVm | 技能面板数据 |
| phase | BattlePhaseVm | 当前战斗阶段（PlayerTurn/EnemyTurn/Animation） |

**Layout**: 屏幕边缘布局，分为四个区域：

1. **左下区域**（队伍状态区）：垂直排列的玩家 CharacterStatusPanel（每个角色一个，垂直堆叠，宽度固定），显示 HP 条、MP 条、Buff。
2. **右下区域**（技能面板区）：SkillPanel，宽度固定，高度自适应，显示当前选中角色的技能分类和技能槽位。
3. **顶部区域**（回合顺序条）：TurnOrderBar，水平条跨屏幕顶部宽度，显示全部参战角色的回合顺序。
4. **顶部回合条下方**（阶段指示）：CaptionText 居中显示当前阶段（如"玩家回合""敌方回合"）。

BattleHud 不遮挡中央的战斗网格区域。

**Events**:

| 事件 | 负载 | 触发条件 |
|------|------|---------|
| UiAction::SelectCharacter | CharacterId | 点击角色状态面板头像 |
| UiAction::SelectSkill | SkillId | 点击技能槽位 |
| UiAction::CastSkill | SkillId | 确认施放技能 |
| UiAction::EndTurn | — | 结束回合按钮 |

**Local State**: 无（由 Screen 统一管理）

**ViewModel**: `BattleHudVm`（定义于 projection-viewmodel.md）

**Used in**: BattleScreen（唯一使用场景）

**变体**:

| 变体 | 差异 | 切换条件 |
|------|------|---------|
| NormalBattle | 标准 HUD 布局 | 常规战斗 |
| BossBattle | 额外 Boss 大型 HP 条 + 阶段提示 | Boss 战（由 BattleScreen 检测） |
| AutoBattle | 隐藏技能面板，显示自动战斗状态 | 自动战斗模式开启 |

**错误处理**:
- 队伍为空：显示 "No characters in party" 占位文本
- 技能面板数据延迟：显示 loading spinner（微小 ProgressBar 循环动画）
- 角色阵亡：CharacterStatusPanel 显示阵亡状态，技能面板禁用

---

### 3.4 TurnOrderBar

**用途**: 回合顺序条，水平排列显示所有参战角色的行动顺序，指示当前行动角色。

**Composes**:

| 子组件 | 类型 | 数量 | 来源 |
|-------|------|------|------|
| TurnIndicator | Molecule | N（按参战角色数） | §2.8 本文 |
| IconButton | Atom | 1 | widget-atoms.md §2.4 |

**Props**:

| 字段 | 类型 | 说明 |
|------|------|------|
| turn_order | Vec\<TurnIndicatorVm\> | 按行动顺序排列的角色指示器列表 |
| is_player_turn | bool | 是否玩家回合（控制高亮颜色） |
| current_turn_index | u32 | 当前行动角色在列表中的索引 |

**Layout**: 水平排列（从左到右表示行动顺序从先到后）。每个 TurnIndicator 等宽排列，间距 UiSpacing::sm。当前行动角色的 TurnIndicator 下方显示金色箭头(IconButton，向下箭头图标)。所有指示器放入 ScrollPanel（当角色超过 8 个时允许水平滚动）。不同阵营的 TurnIndicator 通过边框颜色区分（玩家=蓝，敌方=红，友方=绿）。

**Events**: 无（纯展示组件）

**Local State**: 无

**ViewModel**: `TurnOrderBarVm`（定义于 projection-viewmodel.md）

**Used in**: BattleScreen（通过 BattleHud 引用），CampRestScreen（营地休息的回合式流程指示）

**错误处理**:
- 无可显示角色：显示 "No turn data" 占位文本
- 行动顺序变更动画：播放简单滑动动画，角色从旧位置移到新位置

---

### 3.5 InventoryGrid

**用途**: 背包物品网格，包含过滤、搜索、排序和物品行列表。

**Composes**:

| 子组件 | 类型 | 数量 | 来源 |
|-------|------|------|------|
| InventoryItemRow | Molecule | 多行 | §2.3 本文 |
| TabPanel | Atom | 1 | widget-atoms.md §4.3 |
| TextInput | Atom | 1 | widget-atoms.md §10.1 |
| SelectList | Atom | 1 | widget-atoms.md §5.2 |
| ScrollPanel | Atom | 1 | widget-atoms.md §4.2 |
| Panel (DarkPanel 变体) | Atom | 1 | widget-atoms.md §4.1 |

**Props**:

| 字段 | 类型 | 说明 |
|------|------|------|
| inventory_id | InventoryId | 背包 ID |
| items | Vec\<InventoryItemRowVm\> | 物品行数据列表 |
| categories | Vec\<CategoryFilterDef\> | 分类过滤标签定义 |
| search_placeholder_key | UiTextKey | 搜索框占位文本 Key |
| sort_options | Vec\<SortOptionDef\> | 排序选项 |
| default_filter | usize | 默认选中的过滤分类索引 |
| default_sort | usize | 默认排序选项索引 |
| selected_item | Option\<ItemId\> | 当前选中的物品 |

**Layout**: 垂直排列。顶部为 TabPanel（分类过滤标签：全部/装备/消耗品/材料/关键物品）。分类标签下方为搜索栏(TextInput + 搜索图标)。搜索栏下方左侧为排序下拉(SelectList，极简模式)，右侧显示物品总数(CaptionText，如"共 24 件")。下方为 ScrollPanel 包裹的 InventoryItemRow 列表（垂直排列）。选中某物品时，右侧或底部展开详情面板（由 Screen 管理，不在 InventoryGrid 内）。

**Events**:

| 事件 | 负载 | 触发条件 |
|------|------|---------|
| UiAction::SelectItem | ItemId | 点击物品行 |
| UiAction::UseItem | ItemId | 点击使用按钮 |
| UiAction::EquipItem | (ItemId, EquipmentSlot) | 点击装备按钮 |
| UiAction::DropItem | ItemId | 点击丢弃按钮 |
| UiAction::SearchItems | String | 搜索文本变更 |
| UiAction::SortItems | SortOptionVm | 切换排序选项 |
| UiAction::FilterCategory | usize | 切换分类过滤 |
| UiAction::ShowContextMenu | (ItemId, Vec2) | 右键物品行 |
| UiAction::ShowItemTooltip | ItemId | 悬停物品图标超过 0.3s |

**Local State**:

| 状态 | 类型 | 初始值 | 说明 |
|------|------|--------|------|
| search_query | String | "" | 当前搜索关键词 |
| active_filter | usize | default_filter | 当前过滤分类索引 |
| active_sort | usize | default_sort | 当前排序选项索引 |
| scroll_offset | f32 | 0.0 | 列表滚动偏移 |

**ViewModel**: `InventoryGridVm`（定义于 projection-viewmodel.md）

**Used in**: InventoryScreen（核心内容），ShopScreen（出售标签页），LootScreen（战利品网格）

**错误处理**:
- 空背包（无物品）：显示空态图片 + CaptionText "背包空空如也"（本地化 Key: ui.inventory.empty）
- 搜索无结果：显示 CaptionText "未找到匹配物品" + 清除搜索按钮
- 分类过滤后无内容：清空列表，显示分类空态文本

---

### 3.6 QuestLog

**用途**: 任务日志面板，包含任务分类过滤、任务条目列表和任务详情面板。

**Composes**:

| 子组件 | 类型 | 数量 | 来源 |
|-------|------|------|------|
| QuestEntry | Molecule | 多行 | §2.4 本文 |
| TabPanel | Atom | 1 | widget-atoms.md §4.3 |
| ScrollPanel | Atom | 1 | widget-atoms.md §4.2 |
| Panel (DarkPanel 变体) | Atom | 1 | widget-atoms.md §4.1 |
| HeadingText (LocalizedText 变体) | Atom | 1 | widget-atoms.md §6.1 |
| BodyText (LocalizedText 变体) | Atom | 1 | widget-atoms.md §6.1 |

**Props**:

| 字段 | 类型 | 说明 |
|------|------|------|
| quest_log_id | QuestLogId | 任务日志 ID |
| quests | Vec\<QuestEntryVm\> | 任务条目数据列表 |
| categories | Vec\<QuestCategoryDef\> | 分类过滤定义（主线/支线/日常/活动） |
| selected_quest | Option\<QuestId\> | 当前选中的任务（用于显示详情） |
| tracked_quest | Option\<QuestId\> | 当前追踪的任务（HUD 中显示） |

**Layout**: 左右分栏布局。左侧为列表区（占 40% 宽度）：顶部为 TabPanel（任务分类标签：主线/支线/日常/活动/已完成）。下方为 ScrollPanel 包裹的 QuestEntry 列表。列表项点击后选中高亮。右侧为详情区（占 60% 宽度）：Panel 包含标题(HeadingText)、描述(BodyText)、奖励列表(多个 CaptionText)、追踪按钮和放弃按钮。

**Events**:

| 事件 | 负载 | 触发条件 |
|------|------|---------|
| UiAction::SelectQuest | QuestId | 点击任务条目 |
| UiAction::TrackQuest | QuestId | 点击追踪按钮 |
| UiAction::AbandonQuest | QuestId | 点击放弃按钮（弹出确认 Modal） |
| UiAction::FilterQuestCategory | usize | 切换分类过滤 |

**Local State**:

| 状态 | 类型 | 初始值 | 说明 |
|------|------|--------|------|
| active_category | usize | 0 | 当前显示的任务分类 |
| show_completed | bool | false | 是否显示已完成任务 |

**ViewModel**: `QuestLogVm`（定义于 projection-viewmodel.md）

**Used in**: QuestLogScreen，CampRestScreen（简易版，仅显示追踪任务进度）

**错误处理**:
- 当前分类下无任务：显示空态文本（如"当前无线程任务"）
- 选中任务数据缺失：详情面板显示"任务数据不可用"
- 追踪任务被放弃：清除 tracked_quest，刷新列表

---

### 3.7 DialoguePanel

**用途**: 对话系统面板，包含说话者头像、对话气泡、选项列表和跳过按钮。

**Composes**:

| 子组件 | 类型 | 数量 | 来源 |
|-------|------|------|------|
| CharacterPortrait | Molecule | 1（说话者） | §2.2 本文 |
| DialogueChoice | Molecule | N（选项数） | §2.5 本文 |
| Panel (DarkPanel 变体) | Atom | 1 | widget-atoms.md §4.1 |
| BodyText (LocalizedText 变体) | Atom | 1 | widget-atoms.md §6.1 |
| IconButton | Atom | 1 | widget-atoms.md §2.4 |
| PrimaryButton | Atom | 1 | widget-atoms.md §2.1 |

**Props**:

| 字段 | 类型 | 说明 |
|------|------|------|
| dialogue_id | DialogueId | 对话实例 ID |
| speaker | CharacterPortraitVm | 当前说话者 |
| dialogue_text_key | UiTextKey | 当前对话文本 Key |
| dialogue_text_params | HashMap\<String, String\> | 文本参数 |
| choices | Vec\<DialogueChoiceVm\> | 可用选项列表 |
| is_typing | bool | 是否正在逐字显示文本 |
| typing_progress | f32 | 打字动画进度（0.0~1.0） |
| is_skippable | bool | 是否可跳过 |
| auto_advance | bool | 是否自动推进（无选项时） |

**Layout**: 底部全宽面板布局。最底部为深色半透明背景 Panel。底部居左为说话者 CharacterPortrait(精简模式，仅头像+名称)。头像右侧为对话气泡(Panel)：顶部为说话者名称(CaptionText)，中部为对话文本(BodyText，逐字显示)，底部右侧为"继续"三角形图标(IconButton，闪烁)。有选项时：对话气泡下方垂直排列 DialogueChoice（最多 4 个，超出滚动）。右上角为跳过按钮(IconButton)和自动播放切换按钮(PrimaryButton 极小变体)。

**Events**:

| 事件 | 负载 | 触发条件 |
|------|------|---------|
| UiAction::SelectChoice | ChoiceId | 点击对话选项 |
| UiAction::AdvanceDialogue | — | 点击对话气泡区域（无选项时推进到下一句） |
| UiAction::SkipDialogue | — | 点击跳过按钮 |
| UiAction::ToggleAutoPlay | — | 点击自动播放按钮 |

**Local State**:

| 状态 | 类型 | 初始值 | 说明 |
|------|------|--------|------|
| typing_char_index | usize | 0 | 当前显示到的字符索引 |
| show_continue_hint | bool | false | 是否显示继续提示三角 |
| selected_choice | Option\<usize\> | None | 键盘导航选中的选项索引 |

**ViewModel**: `DialoguePanelVm`（定义于 projection-viewmodel.md）

**Used in**: DialogueOverlay（对话场景），CutsceneOverlay（过场中的对话）

**错误处理**:
- 说话者数据缺失：显示"?"占位头像 + "???"名称
- 选项列表为空且 auto_advance=false：显示"继续"按钮替代
- 文本参数缺失：显示原始 text_key，不替换参数
- 逐字显示被跳过：立即显示完整文本

---

### 3.8 ShopPanel

**用途**: 商店交易面板，包含商品列表、分类标签、玩家金币显示和购物车汇总。

**Composes**:

| 子组件 | 类型 | 数量 | 来源 |
|-------|------|------|------|
| ShopItemCard | Molecule | 多张 | §2.6 本文 |
| TabPanel | Atom | 1 | widget-atoms.md §4.3 |
| ScrollPanel | Atom | 1 | widget-atoms.md §4.2 |
| StatText (LocalizedText 变体) | Atom | 1 | widget-atoms.md §6.1 |
| Panel (DarkPanel 变体) | Atom | 1 | widget-atoms.md §4.1 |
| PrimaryButton | Atom | 1 | widget-atoms.md §2.1 |
| CaptionText (LocalizedText 变体) | Atom | 1 | widget-atoms.md §6.1 |

**Props**:

| 字段 | 类型 | 说明 |
|------|------|------|
| shop_id | ShopId | 商店实例 ID |
| shop_name_key | UiTextKey | 商店名称 Key |
| player_gold | u32 | 玩家持有金币 |
| items | Vec\<ShopItemCardVm\> | 商品数据列表 |
| categories | Vec\<ShopCategoryDef\> | 商品分类标签定义 |
| buy_mode | bool | true=购买模式，false=出售模式 |
| cart_items | Vec\<ItemId\> | 购物车中选中的商品 |
| cart_total | u32 | 购物车总价 |
| transaction_state | TransactionStateVm | 交易状态（Idle/Processing/Complete/Error） |

**Layout**: 垂直排列分层布局。

**顶部栏**（固定）：商店名称(HeadingText，左)，分类标签(TabPanel，中)，玩家金币(StatText，右，显示 "金币: {player_gold}")。

**中部**（ScrollPanel 区域，占大部分空间）：购买模式时 = 商品网格（3 列网格布局的 ShopItemCard，每张卡片等宽等高）；出售模式时 = InventoryItemRow 列表（垂直排列）。

**底部栏**（固定）：左为购物车摘要(CaptionText，"已选 {count} 件，合计 {total}")。右为购买按钮(PrimaryButton，disabled 当 cart_items 为空或 player_gold < cart_total)和取消按钮(SecondaryButton)。

**Events**:

| 事件 | 负载 | 触发条件 |
|------|------|---------|
| UiAction::SelectShopItem | ItemId | 点击商品卡片 |
| UiAction::AddToCart | ItemId | 点击购买模式的商品卡片上的"+"按钮 |
| UiAction::RemoveFromCart | ItemId | 点击购物车中的移除按钮 |
| UiAction::BuyItems | Vec\<ItemId\> | 点击购买按钮 |
| UiAction::SellItem | ItemId | 点击出售模式中的物品行出售按钮 |
| UiAction::SwitchShopTab | usize | 切换分类标签 |
| UiAction::ToggleBuySell | bool | 切换购买/出售模式 |

**Local State**:

| 状态 | 类型 | 初始值 | 说明 |
|------|------|--------|------|
| buy_mode | bool | true | 购买模式/出售模式 |
| active_category | usize | 0 | 当前商品分类 |
| cart | Vec\<CartEntry\> | Vec::new() | 购物车内容 |
| transaction_anim | f32 | 0.0 | 交易完成时的金币变化动画 |

**ViewModel**: `ShopPanelVm`（定义于 projection-viewmodel.md）

**Used in**: ShopScreen（商店界面）

**变体**:

| 变体 | 差异 | 切换条件 |
|------|------|---------|
| BuyMode | 显示 ShopItemCard 网格，"购买"按钮 | 默认模式 |
| SellMode | 显示 InventoryItemRow 列表，"出售"按钮 | 点击"出售"标签 |

**错误处理**:
- 商品列表为空：显示空态文本"当前分类无商品"
- 金币不足：购买按钮 disabled，商品卡片右下角显示 CaptionText "金币不足"
- 交易失败：底部栏显示错误文本(BodyText, 红色) + 重试按钮
- 库存为 0 的商品：卡片半透明，显示"售罄"徽章，不可点击

---

## 4. 复合组件一览表

### 4.1 Molecule 汇总

| Molecule | 原子组成数 | 主要 Atom | 使用 Screen | ViewModel |
|---------|----------|-----------|------------|-----------|
| SkillSlot | 4 | IconButton, CaptionText, ProgressBar | BattleScreen, PartySetupScreen, CharacterDetailScreen | SkillSlotVm |
| CharacterPortrait | 5 | Panel, Image, BodyText, HpBar, StatusIcon | BattleScreen, PartySetupScreen, InventoryScreen | CharacterPortraitVm |
| InventoryItemRow | 5 | IconButton, BodyText, CaptionText, Panel | InventoryScreen, ShopScreen, LootScreen | InventoryItemRowVm |
| QuestEntry | 5 | Panel, BodyText, ProgressBar, CaptionText | QuestLogScreen, HudOverlay | QuestEntryVm |
| DialogueChoice | 3 | Panel, BodyText, IconButton | DialogueOverlay | DialogueChoiceVm |
| ShopItemCard | 6 | Panel, IconButton, CaptionText, StatText, PrimaryButton | ShopScreen | ShopItemCardVm |
| BuffIcon | 3 | IconButton, SuperCaptionText, TooltipTrigger | BattleScreen, CharacterStatusPanel | BuffVm |
| TurnIndicator | 3 | IconButton, Panel, CaptionText | TurnOrderBar, BattleScreen | TurnIndicatorVm |

### 4.2 Organism 汇总

| Organism | 子组件组成 | 使用 Screen | ViewModel |
|---------|-----------|------------|-----------|
| SkillPanel | SkillSlot[] + TabPanel + ScrollPanel | BattleScreen, PartySetupScreen, CharacterDetailScreen | SkillPanelVm |
| CharacterStatusPanel | CharacterPortrait + MpBar + ApBar + BuffIcon[] + BodyText | BattleScreen, PartySetupScreen | CharacterStatusPanelVm |
| BattleHud | CharacterStatusPanel[] + SkillPanel + TurnOrderBar | BattleScreen | BattleHudVm |
| TurnOrderBar | TurnIndicator[] + IconButton | BattleScreen, CampRestScreen | TurnOrderBarVm |
| InventoryGrid | InventoryItemRow[] + TabPanel + TextInput + SelectList + ScrollPanel | InventoryScreen, ShopScreen, LootScreen | InventoryGridVm |
| QuestLog | QuestEntry[] + TabPanel + ScrollPanel + HeadingText + BodyText | QuestLogScreen, CampRestScreen | QuestLogVm |
| DialoguePanel | CharacterPortrait + DialogueChoice[] + Panel + BodyText + IconButton | DialogueOverlay, CutsceneOverlay | DialoguePanelVm |
| ShopPanel | ShopItemCard[] + TabPanel + ScrollPanel + StatText + Panel + PrimaryButton | ShopScreen | ShopPanelVm |

### 4.3 复合组件与 Screen 的使用关系

```
Screen ─────────────────── 使用的 Organism ───────────── 使用的 Molecule（直接）
BattleScreen              BattleHud, SkillPanel,         TurnIndicator
                          CharacterStatusPanel,
                          TurnOrderBar

InventoryScreen           InventoryGrid                  InventoryItemRow

ShopScreen                ShopPanel, InventoryGrid       ShopItemCard, InventoryItemRow

QuestLogScreen            QuestLog                       QuestEntry

DialogueOverlay           DialoguePanel                  DialogueChoice

PartySetupScreen          SkillPanel,                    CharacterPortrait, SkillSlot
                          CharacterStatusPanel

CharacterDetailScreen     SkillPanel                     CharacterPortrait, SkillSlot,
                                                         BuffIcon

CampRestScreen            TurnOrderBar, QuestLog         QuestEntry

LootScreen                InventoryGrid                  InventoryItemRow

MainMenuScreen            （无 Organism，仅 Panel + Button 组合）
SettingsScreen            （无 Organism，仅 TabPanel + Toggle + ProgressBar 组合）
SaveLoadScreen            （无 Organism，仅 SelectList + Panel + Button 组合）
```

> **注意**: MainMenuScreen、SettingsScreen 和 SaveLoadScreen 不包含复合组件——它们的 UI 复杂度足够低，由 Panel + 原子按钮直接组合即可。这符合"不要为简单页面强行引入复合组件"的原则。

---

## 5. 复合组件设计规则

### 5.1 创建规则

| 规则 | 说明 |
|------|------|
| CMP-CR-01 | Molecule 必须包含 ≥3 个 Atom，且出现在 ≥2 个 Screen 中才应提取为 Molecule |
| CMP-CR-02 | Organism 必须包含 ≥1 个 Molecule，且对应一个功能完整的 UI 区域 |
| CMP-CR-03 | 不要为只在一个 Screen 中使用一次的"一次性组合"创建复合组件 |
| CMP-CR-04 | Molecule 只引用 Atom，不引用其他 Molecule 或 Organism |
| CMP-CR-05 | Organism 禁止交叉引用其他 Organism（BattleHud → TurnOrderBar 是唯一例外，需架构审批） |
| CMP-CR-06 | 复合组件不直接引用 Domain 数据，通过 ViewModel 消费 |
| CMP-CR-07 | 复合组件与屏幕之间必须经过 ViewModel：Screen 从 UiStore 读取 ViewModel → 传入复合组件 Props |

### 5.2 边界规则

| 规则 | 说明 |
|------|------|
| CMP-BD-01 | Molecule 的内部布局对外部不可见——Screen 不能访问 Molecule 内部的原子组件 |
| CMP-BD-02 | Organism 可以定义自己的 ViewModel（由对应 Projection 投影），不能直接暴露内部 Molecule/Atom 的 ViewModel |
| CMP-BD-03 | 复合组件通过 UiAction 向上传递事件，不直接调用 Screen 方法 |
| CMP-BD-04 | 复合组件的本地状态对外部不可见——Screen 不能访问复合组件内部的 hover/selected 状态 |

### 5.3 验证规则

| # | 规则 | 校验逻辑 |
|---|------|----------|
| CMP-VAL-01 | 复合组件 Props 中包含业务 ID 而非 Entity | 审查 Props 中无 Entity 字段 |
| CMP-VAL-02 | Molecule 不包含其他 Molecule | 审查 Molecule 组成的嵌套深度 |
| CMP-VAL-03 | Organism 最多嵌套一层 Molecule | 审查 Organism → Molecule 层级 |
| CMP-VAL-04 | 复合组件文本使用 UiTextKey | 审查 Props 中字符串字段类型 |
| CMP-VAL-05 | 复合组件颜色使用 StyleToken | 审查 Layout 描述中不引用裸颜色值 |

---

## 6. 与 widget-atoms.md 和 screens.md 的交叉引用

| 概念 | 本文 § | widget-atoms.md | screens.md |
|------|--------|-----------------|------------|
| SkillSlot | §2.1 | IconButton §2.4, ProgressBar §3.1, LocalizedText §6.1 | BattleScreen §2 |
| CharacterPortrait | §2.2 | Panel §4.1, ProgressBar §3.1, LocalizedText §6.1 | BattleScreen §2 |
| InventoryItemRow | §2.3 | IconButton §2.4, LocalizedText §6.1, Panel §4.1 | InventoryScreen §4 |
| QuestEntry | §2.4 | Panel §4.1, ProgressBar §3.1, LocalizedText §6.1 | QuestLogScreen |
| DialogueChoice | §2.5 | Panel §4.1, LocalizedText §6.1, IconButton §2.4 | DialogueOverlay (overlays.md) |
| ShopItemCard | §2.6 | Panel §4.1, IconButton §2.4, LocalizedText §6.1, PrimaryButton §2.1 | ShopScreen §5 |
| BuffIcon | §2.7 | IconButton §2.4, LocalizedText §6.1 | BattleScreen §2 |
| TurnIndicator | §2.8 | IconButton §2.4, Panel §4.1, LocalizedText §6.1 | BattleScreen §2 |
| SkillPanel | §3.1 | TabPanel §4.3, ScrollPanel §4.2 | BattleScreen §2, PartySetupScreen |
| CharacterStatusPanel | §3.2 | ProgressBar §3.1, LocalizedText §6.1, Panel §4.1 | BattleScreen §2 |
| BattleHud | §3.3 | Panel §4.1 | BattleScreen §2 |
| TurnOrderBar | §3.4 | IconButton §2.4 | BattleScreen §2, CampRestScreen |
| InventoryGrid | §3.5 | TabPanel §4.3, TextInput §10.1, SelectList §5.2, ScrollPanel §4.2 | InventoryScreen §4 |
| QuestLog | §3.6 | TabPanel §4.3, ScrollPanel §4.2, LocalizedText §6.1, Panel §4.1 | QuestLogScreen |
| DialoguePanel | §3.7 | Panel §4.1, LocalizedText §6.1, IconButton §2.4, PrimaryButton §2.1 | DialogueOverlay (overlays.md) |
| ShopPanel | §3.8 | TabPanel §4.3, ScrollPanel §4.2, LocalizedText §6.1, Panel §4.1, PrimaryButton §2.1 | ShopScreen §5 |

---

## 7. 复合组件目录结构（目标实现）

复合组件代码属于 `widgets/` 层，依赖 `primitives/` 层的原语控件：

```
src/ui/
├── primitives/               ← UI 原语层（对应 widget-atoms.md）
│   ├── button/
│   ├── progress_bar/
│   ├── panel/
│   ├── text/
│   ├── list/
│   └── modal/
└── widgets/                  ← 游戏业务控件层
    ├── tooltip/
    ├── notification/
    └── composites/           ← 复合组件（对应本文档）
        ├── molecules/
        │   ├── skill_slot/
        │   ├── character_portrait/
        │   ├── inventory_item_row/
        │   ├── quest_entry/
        │   ├── dialogue_choice/
        │   ├── shop_item_card/
        │   ├── buff_icon/
        │   └── turn_indicator/
        └── organisms/
            ├── skill_panel/
            ├── character_status_panel/
            ├── battle_hud/
            ├── turn_order_bar/
            ├── inventory_grid/
            ├── quest_log/
            ├── dialogue_panel/
            └── shop_panel/
```

---

*本文档由 @presentation-architect 维护。新增复合组件必须先在本文档定义，明确其组成、Props、Layout 和使用场景，再编写代码。*
