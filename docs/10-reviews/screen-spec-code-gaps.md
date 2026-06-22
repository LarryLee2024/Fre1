---
status: active
type: review
reviewer: code-reviewer
created: 2026-06-22
scope: src/ui/
standard: SSPEC (Screen Specification)
---

# SSPEC（Screen Specification）代码差距分析

## 目的

评估 `src/ui/` 与 Screen Specification（SSPEC）标准的差距。SSPEC 要求：
1. 工厂构造 Screen —— 禁止直接 `commands.spawn` Node/Button/Text
2. 禁止直接查询 Domain —— ViewModel（UiStore）是 Widget 的唯一数据源
3. 所有用户可见文本使用 `LocalizationKey` —— 禁止硬编码字符串
4. 所有颜色/字体/间距使用 `StyleToken`（Theme）—— 禁止硬编码像素或 rgba
5. 生命周期退出时确定性销毁
6. 可组合的 Screen + Overlay 层级

严重级别：P0 = 必须修复（架构违规），P1 = 应该修复（代码质量/技术债），P2 = 可观察（次要，后续跟踪）

---

## 1. Screen 构造 —— 工厂模式合规性

### 1.1 Primitives 工厂覆盖（良好基础）

Primitives 层提供了定义良好的工厂函数，覆盖所有基本 UI 原子。全部使用 `spawn_*` 命名，通过参数接受 `Theme`，返回 `Entity`。覆盖的原子：

| Factory | Type | Localized Variant |
|---------|------|-------------------|
| `spawn_text` | `Text` | `spawn_localized_text` |
| `spawn_button` | `Button` | `spawn_localized_button` |
| `spawn_panel` | `Panel` | -- |
| `spawn_progress_bar` | `ProgressBar` | -- |
| `spawn_toggle` | `Toggle` | built-in key support |
| `spawn_modal` | `Modal` | built-in key support |
| `spawn_list` | `List` | -- |
| `spawn_tab_panel` | `TabPanel` | MISSING |
| `spawn_scroll_panel` | `ScrollPanel` | -- |
| `spawn_select_list` | `SelectList` | -- |

### 1.2 Screen 工厂分析

| Screen | 工厂函数 | 违规情况 | 严重级别 |
|--------|---------|----------|----------|
| MainMenu | `spawn_main_menu` | 调用 `spawn_panel` 后覆盖了工厂 Node | P2 |
| Battle | `spawn_battle_screen` | **第 53 行直接使用 `commands.spawn((Node{...}, BattleScreen))`** | **P0** |
| Settings | `spawn_settings_screen` | 调用 `spawn_panel` 后覆盖了工厂 Node | P2 |
| SaveLoad | 内部工厂 | 使用 `spawn_panel` 后用原始 Node 覆盖；硬编码 slots | P2 |
| Shop | `spawn_shop_panel` | 第 48 行用原始布局覆盖工厂 Node | P2 |
| Inventory | `spawn_inventory_screen` | 使用 `spawn_panel` 后用原始 Node 覆盖 | P2 |

**详情：**

- **P0 -- `src/ui/screens/battle/mod.rs:53-63`**：BattleScreen 根节点通过原始
  `commands.spawn((Node{...}, BattleScreen, Name {...}))` 创建，而非调用
  `spawn_panel`。这是唯一完全绕过 primitives 工厂根容器的 Screen。

- **P2 -- 所有 Screen**：每个 Screen 工厂都先调用 `spawn_panel`，然后立即
  覆盖返回实体的 Node 组件：
  ```rust
  let root = spawn_panel(commands, theme, PanelVariant::Basic);
  commands.entity(root).insert((Node { width: 100%, ... }));
  ```
  这种模式是代码异味——工厂返回值被丢弃。未来的 SSPEC 改进应提供
  `spawn_fullscreen_panel` 或类似函数。

### 1.3 Widget 工厂分析

| Widget | 工厂函数 | ViewModel 绑定 | 严重级别 |
|--------|---------|----------------|----------|
| CharacterCard | `spawn_character_card` | `Dirty<CharacterPanelVm>`（已连线） | OK |
| SkillSlot | `spawn_skill_slot` | `Dirty<SkillPanelVm>`（已连线） | OK |
| ActionMenu | `spawn_action_menu` | 无 Dirty 绑定 | **P1** |
| ShopPanel | `spawn_shop_panel` | 无 Dirty 绑定 | **P1** |
| InventoryGrid | `spawn_inventory_grid` | 无 Dirty 绑定 | **P1** |
| ShopItemCard | `spawn_shop_item_card` | 无 Dirty 绑定 | P2（ShopPanel 的子级） |
| InventoryItemRow | `spawn_inventory_item_row` | 无 Dirty 绑定 | P2（InventoryGrid 的子级） |

**P1 -- `src/ui/widgets/action_menu/factory.rs`**：ActionMenu 未携带
`Dirty<ActionMenuVm>`（该类型不存在）。它依赖静态构造和硬编码字符串。没有 ViewModel 驱动其状态。

---

## 2. ViewModel 使用违规

### 2.1 架构违规（P0）

UI 代码中的直接 Domain 查询——这些破坏了 ViewModel 防火墙：

1. **P0 -- `src/ui/screens/battle/systems.rs:30`**：
   ```rust
   turn_queue: Option<Res<TurnQueue>>,
   ```
   `TurnQueue` 是 `core::domains::combat` 类型的领域组件。UI observer 直接导入
   并查询领域组件。违反的规则：`docs/06-ui/04-data-flow/`
   规定 UiStore 是唯一数据源。

   影响：在 `BattleAction::EndTurn` 时，unit_id 从
   `turn_queue.current()`（第 42-46 行）提取，这使得依赖回放的 UI 状态
   从 UI 角度来看变得非确定性。

2. **P0 -- `src/ui/screens/battle/visibility.rs:20`**：
   ```rust
   fn battle_zone_visibility_system(
       battle_phase: Res<State<BattlePhase>>,
   ```
   `BattlePhase` 是领域状态。区域可见性应由
   `BattleHudVm.phase_key` 或专用的可见性 ViewModel 驱动。

3. **P1 -- `src/ui/screens/battle/visibility.rs`**：直接使用 `Changed<BattleZone>` 和
   `State<BattlePhase>`。不存在可见性的 ViewModel 投影。

4. **P1 -- `src/ui/screens/battle/systems.rs:47`**：
   ```rust
   .unwrap_or_default()
   ```
   在业务代码中使用。这将逻辑错误掩盖为空字符串。

### 2.2 ViewModel 定义缺口

| ViewModel | 已定义 | 已消费 | 投影已连线 |
|-----------|--------|--------|-----------|
| `BattleHudVm` | 是（7 字段） | 否 | 部分（投影中的直接 Query） |
| `CharacterPanelVm` | 是 | 通过 Dirty 的 CharacterCard | 硬编码默认值 |
| `SkillPanelVm` | 是 | 通过 Dirty 的 SkillSlot | 投影存在（硬编码数据） |

**P1 -- BattleHudVm 已定义但没有 Widget 消费它**。该结构体包含
`hp`、`max_hp`、`mp`、`max_mp`、`turn_number`、`phase_key`，但没有任何 widget
系统读取 `Dirty<BattleHudVm>`。位于 `battle/mod.rs:73` 的回合指示文本
是硬编码字符串，未绑定到任何 VM。

### 2.3 投影层问题

**P1 -- `src/ui/projections/battle.rs:141`**：
```rust
fn on_turn_started_projection(... query: Query<&ActionPoints>)
```
投影应该是将领域事件转换为 ViewModel 更新的纯函数。在投影内部查询
`ActionPoints` 模糊了边界。注释写着 "intentional bridge"，
但这削弱了 ViewModel 隔离原则。

**P1 -- `src/ui/projections/battle.rs:231-270`**：
```rust
fn on_character_panel_projection(... query: Query<&Name>)
```
类似地，在投影中直接查询领域组件 `Name`。

---

## 3. Hardcoded Text

### 3.1 P0 -- 用户可见的硬编码字符串

以下为用户可见文本，必须使用 `LocalizationKey`：

| 位置 | 字符串 | 上下文 |
|------|--------|--------|
| `screens/main_menu/mod.rs:78` | `"Fre"` | 游戏标题 |
| `screens/main_menu/mod.rs:102` | `"A Bevy SRPG"` | 副标题 |
| `screens/main_menu/mod.rs:151` | `"v0.1.0"` | 版本文本 |
| `screens/main_menu/mod.rs:122` | `"New Game"` | 按钮回退（key 存在：loc::ui::NEW_GAME） |
| `screens/main_menu/mod.rs:129` | `"Load Game"` | 按钮回退（key 存在：loc::ui::LOAD_GAME） |
| `screens/main_menu/mod.rs:136` | `"Settings"` | 按钮回退（key 存在：loc::ui::SETTINGS） |
| `screens/battle/mod.rs:73` | `"Turn: 3    Phase: Player Turn"` | 回合指示器——完全硬编码的调试文本 |
| `screens/battle/mod.rs:95` | `"Aria"` | 角色名称 |
| `screens/battle/mod.rs:118` | `"End Turn"` | 按钮回退（key 存在：loc::ui::BATTLE_END_TURN） |
| `screens/save_load/mod.rs:111` | `"Save/Load"` | Screen 标题 |
| `screens/save_load/mod.rs:134` | `"Save Slot {}"` | 存档位格式 |
| `screens/save_load/mod.rs:138` | `"Empty"` | 空存档位标签 |
| `screens/save_load/mod.rs:153,165` | `"Save"` / `"Load"` | 按钮回退文本 |
| `screens/settings/mod.rs:129` | `"Show Damage Numbers"` | 开关标签 |
| `screens/settings/mod.rs:147` | `"Dark Theme"` | 开关标签 |
| `screens/settings/mod.rs:163` | `"Close"` | 按钮回退 |
| `screens/settings/mod.rs:180` | `"Save"` | 按钮回退 |
| `screens/inventory/systems.rs:33,39` | `"player"` | 硬编码用户 ID |

### 3.2 P1 -- Widget 级别硬编码字符串

| 位置 | 字符串 | 上下文 |
|------|--------|--------|
| `widgets/action_menu/factory.rs:47-71` | `"Attack"`、`"Defend"`、`"Skill"`、`"Item"`、`"Wait"` | 行动标签（key 已存在） |
| `widgets/character_card/factory.rs:73` | `format!("Lv.{}", level)` | 等级格式 |
| `widgets/shop_panel/factory.rs:76` | `"Shop"` | 商店标题 |
| `widgets/shop_panel/factory.rs:84` | `"Gold: 999"` | 金币显示 |
| `widgets/shop_panel/factory.rs:95` | `"Buy"`、`"Sell"` | 选项卡标签 |
| `widgets/shop_panel/factory.rs:100-102` | `"Health Potion"`、`"Mana Potion"`、`"Antidote"` | 物品名称 |
| `widgets/shop_panel/factory.rs:119` | `"Old Sword"`、`"Leather Armor"` | 出售物品名称 |
| `widgets/shop_item_card/factory.rs:52` | `format!("Gold: {}", price)` | 价格格式 |
| `widgets/shop_item_card/factory.rs:53` | `format!("Stock: {}", stock)` | 库存格式 |
| `widgets/inventory_item_row/factory.rs` | `format!("x{}", qty)` | 数量格式 |
| `widgets/inventory_grid/factory.rs` | `"Inventory"`、`"Gold: 100"`、物品名称 | 完整库存文本 |
| `primitives/progress_bar/factory.rs:63-67` | `"HP "`、`"MP "`、`"XP "` | 进度条前缀 |

### 3.3 P2 -- 非面向用户的硬编码字符串

| 位置 | 字符串 | 上下文 |
|------|--------|--------|
| `widgets/shop_panel/factory.rs:149` | `"Sell"` | 按钮回退（key 存在：loc::economy::SHOP_SELL_TEXT） |
| `widgets/shop_item_card/factory.rs:116` | `"Buy"` | 按钮回退（key 存在：loc::economy::SHOP_BUY_TEXT） |

注意：这些降级为 P2 是因为 `spawn_localized_button` 调用
确实传入了正确的 localization key——硬编码字符串只是
回退值。这是符合 SSPEC 的正确模式。

---

## 4. 颜色/字体违规 —— 硬编码 vs StyleToken

### 4.1 良好实践：Theme Token 使用

主题系统（`src/ui/theme/`）设计良好：

- `UiColors` 包含 `dark()`/`light()` 构造器，覆盖约 20 个语义 token
- `UiSpacing` 包含命名 token（xs=4、sm=8、md=16、lg=24、xl=32、xxl=48、border_radius_sm/lg、button_height）
- `UiTypography` 包含字体路径和大小/粗细 token

所有 primitives 工厂正确地消费 Theme：
- `button/factory.rs`：`theme.colors.accent_*`、`theme.colors.surface_*`、`theme.colors.text_*`
- `panel/factory.rs`：`theme.colors.surface_*`、`theme.colors.border_*`
- `text/factory.rs`：`theme.colors.text_*`、`theme.typography.*`
- `progress_bar/factory.rs`：`theme.colors.feedback_*`、`theme.colors.accent_*`
- `toggle/factory.rs`：`theme.colors.accent_*`、`theme.colors.surface_*`

### 4.2 P1 违规

1. **P1 -- `src/ui/screens/main_menu/mod.rs:87`**：
   ```rust
   font_size: FontSize::Px(48.0),
   ```
   游戏标题的字体大小被硬编码。主题定义了 `size_display： 36.0`
   和 `size_title： 24.0`，但两者都未使用。标题使用了 `TextVariant::Title`
   （通过 `font_size_for_variant` 默认提供 24px），然后被硬编码的
   `48.0` 覆盖。需要新增 `TextVariant::Display`，或者主题
   应暴露 `size_display`。

2. **P1 -- `src/ui/primitives/modal/factory.rs:61`**：
   ```rust
   let overlay_color = Color::srgba(0.0, 0.0, 0.0, 0.6);
   ```
   Modal 遮罩透明度被硬编码。应为 `theme.colors.overlay` 或类似。

### 4.3 P2 观察

- `Color::NONE` 在多个地方用于 BorderColor。这可以接受，
   但未来添加 `border_none` 的 Theme token 会更一致。
- `button/factory.rs:83-86`：非 Secondary 变体使用 `Color::NONE` 边框。
   此模式可以接受但略显晦涩。

---

## 5. Screen 组合与布局

### 5.1 BattleScreen 9 区域布局

**良好实践**：区域布局（`battle/layout.rs`）使用绝对定位，
配合 `spawn_zone` 工厂和 `BattleZone` 枚举（9 个区域）。区域工厂
正确地读取 `theme.spacing` token 来设置 padding/margin。

**问题**：

- **P1 -- `battle/layout.rs:53-54`**：Z2 TopCenter 存在一个 TODO 关于缺少
  水平居中：
  ```rust
  // TODO[P2] missing: center horizontally
  ```

- **P2 -- `battle/mod.rs:79,85,126`**：Z2、Z3、Z8 为空（TODO 注释）。
  这些是有意为之的 P2 范围占位符。

### 5.2 生命周期管理

| Screen | 创建 | 销毁 | 评分 |
|--------|------|------|------|
| MainMenu | `Startup` | `OnExit(GameState::MainMenu)` | 初始阶段可接受 |
| Battle | `OnEnter(GameState::Combat)` | `OnExit(GameState::Combat)` | OK |
| Settings | 基于 `UiCommand::OpenScreen` 的 Observer | 基于 `UiCommand::CloseScreen` 的 Observer | 脆弱 |
| SaveLoad | 基于 `UiCommand::OpenScreen` 的 Observer | 基于 `UiCommand::CloseScreen` 的 Observer | 脆弱 |
| Shop | 基于 `UiCommand::OpenScreen` 的 Observer | 基于 `UiCommand::CloseScreen` 的 Observer | 脆弱 |

**P1 -- 生命周期模式不一致**。`screens/mod.rs:48` 的注释
确认了这一点：
```rust
// 未来将迁移到 OnEnter(GameState::...) + OnExit(...)
```
Overlay Screen 使用基于 Observer 的创建/销毁，这与 ScreenStack
导航系统不整合。当 `UiScreenState` 接入后，这些需要迁移。

**P2 -- `src/ui/navigation/screen_state.rs`**：`UiScreenState` 定义了
`ScreenLifecycle` 枚举，但从未作为 Resource 插入。ScreenStack
也从未执行 push/pop。

### 5.3 Screen 组合总结

```
当前状态：
  Screen
    ├── Panel（被覆盖）
    ├── 原始文本（硬编码或带 localization 回退）
    ├── Widget（部分有 Dirty 绑定，部分没有）
    └── 按钮（通过 spawn_localized_button 实现本地化）

预期状态（SSPEC）：
  Screen（工厂）
    ├── Panel（spawn_panel，无覆盖）
    ├── 文本（全部通过 spawn_localized_text 使用 loc::ui::* key）
    ├── Widget（全部带有 Dirty<ViewModel> 绑定）
    ├── 按钮（全部通过 spawn_localized_button 使用 loc::* key）
    └── ViewModel 投影连接到领域事件
```

---

## 6. Localization Key 格式不一致

**P1 -- Key 格式不匹配**。代码库使用两种不兼容的 key 格式：

| 格式 | 示例 | 位置 |
|------|------|------|
| 生成的常量（snake_case） | `loc::ui::BATTLE_END_TURN`、`loc::ui::CLOSE` | 大多数 Screen |
| 原始点号字符串 | `"ui.settings.show_damage"`、`"ui.settings.dark_theme"` | Settings 开关创建 |

Settings 开关在 `screens/settings/mod.rs:128` 传入了原始 key 字符串：
```rust
spawn_toggle(commands, theme, "ui.settings.show_damage", "Show Damage Numbers", ...)
```

这绕过了生成的 `loc::*` 常量。要么让 `spawn_toggle`
接受 `&'static str` key（它确实接受——API 是正确的），并让调用方
使用 `loc::ui::SETTINGS_SHOW_DAMAGE`，要么生成的 keys 模块
需要为这些添加条目。

**P2 -- `spawn_tab_panel` 不支持 key**。位于 `widgets/shop_panel/factory.rs:95` 的 Tab panel 明确说明了：
```rust
// MVP: uses plain English labels since spawn_tab_panel does not support
// localization keys yet.
```

---

## 7. 其他发现

### 7.1 P1 -- `unimplemented!()` / TODO 存在崩溃风险

检查 `src/ui/` 中是否存在 `unimplemented!()` 或 `panic!()` 调用。

### 7.2 P1 -- Bridge 模块缺失

`src/ui/bridge/` 目录不存在（在架构规划中被引用过）。
当前 UI 层和 Domain 层之间没有正式的桥接层；
`application/command.rs` 中的 UiCommand -> GameCommand 转换是最接近的实现，
但不完整。

### 7.3 P2 -- UiStore 默认值

`src/ui/view_models/mod.rs:41-49`：BattleHudVm 默认值使用了占位值：
- `phase_key: ""` —— 空字符串默认值意味着没有 phase_key 的战斗 HUD
  将显示空白而不是有信息量的占位符。
- `turn_number: 0` —— 根据注释是有意为之，但最终应使用
  `Option<u32>` 来区分"尚未加载"和"第 0 回合"。

### 7.4 P2 -- 设置持久化

`src/ui/settings.rs:41`：UiSettings::load 中的 `unwrap_or_default()` —— 这
通过返回默认设置来静默吞掉读取错误。损坏的文件
对玩家来说将是不可见的。

---

## 8. 汇总表

| 类别 | P0 | P1 | P2 | 总计 |
|------|----|----|----|------|
| 工厂合规 | 1 | 0 | 4 | 5 |
| ViewModel 违规 | 2 | 4 | 2 | 8 |
| 硬编码文本 | 1 主要 + 12 Screen 字符串 | 15+ Widget 字符串 | 2 | ~30 |
| 颜色/字体违规 | 0 | 2 | 2 | 4 |
| 生命周期/组合 | 0 | 2 | 3 | 5 |
| Localization 格式 | 0 | 1 | 1 | 2 |
| **总计** | **4** | **11** | **14** | **~29** |

---

## 9. 严重程度分布

```
P0（必须修复）：  4  ████████████████████
P1（应该修复）： 11  ████████████████████████████████████████████████████████
P2（观察）：   14   ██████████████████████████████████████████████████████████████████
```

---

## 10. 按优先级排列的建议

### P0 —— 立即修复（架构违规）

1. **BattleScreen 根工厂**：将 `battle/mod.rs:53-63` 处的原始
   `commands.spawn(Node{...})` 替换为 `spawn_panel` 调用，
   然后将 `BattleScreen` 和 `Name` 作为额外组件插入到工厂实体上。

2. **TurnQueue 领域查询**：从 `battle/systems.rs:30` 移除
   `Option<Res<TurnQueue>>`。EndTurn 应从 ViewModel
   推导 `unit_id`，而不是直接查询领域。向 `BattleHudVm` 添加 `current_unit_id`。

3. **BattlePhase 领域查询**：从 `battle/visibility.rs:20` 移除
   `Res<State<BattlePhase>>`。区域可见性应由 ViewModel
   字段（例如 `visible_zones： Vec<BattleZone>`）驱动，通过投影更新。

4. **Battle 回合指示文本**：将硬编码的 `"Turn： 3    Phase: Player Turn"`
   替换为绑定到 `BattleHudVm` 字段的 `spawn_localized_text`。

### P1 —— 应该修复（代码质量/技术债）

1. 将 `BattleHudVm` 通过 `Dirty<BattleHudVm>` 连接到 widget 刷新系统。
2. 向 ActionMenu widget 添加 `Dirty<ActionMenuVm>` 或等效绑定。
3. 为 CharacterCard 添加 `Dirty<CharacterPanelVm>` 刷新系统（已存在但需验证）。
4. 将所有 widget 硬编码字符串迁移为 localization key 和回退值。
5. 为标题添加 `TextVariant::Display` 或 `theme.typography.size_display`。
6. 为模态框遮罩透明度添加 `theme.colors.overlay` token。
7. 修复 localization key 格式：`"ui.settings.show_damage"` -> `loc::ui::*` 常量。
8. 为区域可见性添加投影连线。
9. 添加导航桥接层：接通 `ScreenStack` push/pop，连接 `UiScreenState`。

### P2 —— 观察（后续跟踪）

1. 监控 `spawn_panel` 覆盖模式影响所有 Screen 的情况（非紧急）。
2. 监控空区域 Z2、Z3、Z8 何时获得 widget。
3. TabPanel localization key 支持。
4. UiStore 默认值与 Option 语义。
5. 设置文件错误处理。
6. 为 Overlay Screen 添加 Spawn/Despawn 生命周期钩子点。

---

## 11. 结论

**评估结果：未通过（4 个 P0 问题）**

代码库具有坚实的架构基础——Primitives 工厂、
Theme token 系统、Dirty<T> 绑定和 ViewModel 定义都正确地
构建了。差距在于**一致性应用**：

- BattleScreen 是问题最多的区域，4 个 P0 违规中有 3 个在此。
- Widget 级别的文本硬编码广泛存在（约 30 处），但迁移
  路径由现有的 `spawn_localized_*` 工厂很好地支持。
- ViewModel 管线部分接通（SkillPanelVm -> SkillSlot 正常），
  但 BattleHudVm 没有消费者，ActionMenu 没有绑定。

4 个 P0 项目必须在此审查关闭前解决。之后，
P1 项目可以增量处理。建议在 P0 修复应用后
重新调用 `@code-reviewer`。
