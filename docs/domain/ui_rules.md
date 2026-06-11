# UI 领域规则 (UI Rules)

## 1. 领域概述

UI 系统负责所有面板、行动菜单、浮窗、视觉效果的表现层。遵循 **Logic / Presentation 分离**原则：UI 只展示状态，不保存业务真相；业务逻辑不直接操作 UI。

### 核心原则

- **UI 不操作 ECS，只发出意图**：UiCommand Message 是唯一交互通道
- **ViewModel 层隔离**：游戏逻辑 → ViewModel → UI，UI 只读 ViewModel
- **UI 监听状态变化刷新自己**：不主动轮询
- **主题系统统一样式**：换皮肤只改 UiTheme，UI 代码不动
- **焦点管理**：模态面板阻止游戏输入

---

## 2. UiCommand — UI 命令事件

```rust
#[derive(Message)]
pub enum UiCommand {
    SelectUnit { entity: Entity },
    MoveUnit { coord: IVec2 },
    Attack,
    Skill { skill_id: String },
    SelectTarget { coord: IVec2 },
    Wait,
    Cancel,
    EndTurn,
}
```

### 2.1 命令与阶段映射

| 命令 | 触发阶段 | 目标阶段 |
|------|----------|----------|
| `SelectUnit` | SelectUnit | MoveUnit |
| `MoveUnit` | MoveUnit | ActionMenu（移动后）/ MovingUnit |
| `Attack` | ActionMenu | SelectTarget |
| `Skill` | ActionMenu | SelectTarget |
| `SelectTarget` | SelectTarget | ExecuteAction / ActionMenu |
| `Wait` | ActionMenu | WaitAction |
| `Cancel` | 任意 | 上下文推断回退 |
| `EndTurn` | 任意 | TurnEnd |

### 2.2 Cancel 上下文推断

```
有 skill_id → SelectTarget 取消 → ActionMenu
有菜单实体 → ActionMenu 取消 → 回退位置 → SelectUnit
否则 → MoveUnit 取消 → SelectUnit
```

---

## 3. ViewModel 层

### 3.1 SelectedUnitView — 选中单位视图

```rust
#[derive(Resource)]
pub struct SelectedUnitView {
    pub name: String,
    pub race: String,
    pub class: String,
    pub hp: i32, pub max_hp: i32,
    pub mp: i32, pub max_mp: i32,
    pub stamina: i32, pub max_stamina: i32,
    pub core_attrs: Vec<CoreAttrEntry>,      // 8维核心属性
    pub combat_attrs: Vec<DerivedAttrEntry>,  // 战斗组衍生属性
    pub support_attrs: Vec<DerivedAttrEntry>, // 辅助组衍生属性
    pub skills: Vec<SkillEntry>,
    pub traits: Vec<TraitEntry>,
    pub buffs: Vec<BuffEntry>,
    pub equipment: Vec<EquipmentSlotEntry>,
    pub inventory: Vec<InventoryEntry>,
    pub is_selected: bool,
}
```

**刷新策略**：仅在 `HoveredEntity` 变化时刷新，避免每帧重建。

### 3.2 HoveredEntity — 悬停实体

```rust
#[derive(Resource)]
pub struct HoveredEntity {
    pub entity: Option<Entity>,
}
```

不限于 Selected，任何单位都可查看信息。

### 3.3 TurnInfoView — 回合信息视图

```rust
#[derive(Resource)]
pub struct TurnInfoView {
    pub turn_number: u32,
    pub is_player_turn: bool,
    pub turn_order: Vec<(String, bool)>,  // (name, is_player)
    pub current_index: usize,
}
```

**刷新策略**：TurnState 或 TurnOrder 变化时刷新。

### 3.4 CombatPreviewView — 战斗预览视图

```rust
#[derive(Resource)]
pub struct CombatPreviewView {
    pub is_visible: bool,
    pub estimated_damage: i32,
    pub hit_rate: i32,
    pub crit_rate: i32,
    pub is_lethal: bool,
}
```

仅在 SelectTarget 阶段显示。

### 3.5 GameOverState — 胜负状态

```rust
pub enum GameOverState {
    Playing,
    Victory,
    Defeat,
}
```

检测条件：无敌方 → Victory，无玩家 → Defeat。

---

## 4. UiTheme — 主题系统

### 4.1 颜色常量

| 类别 | 字段 | 默认值 |
|------|------|--------|
| 面板 | `panel_bg` | rgba(0.1, 0.1, 0.1, 0.9) |
| 按钮 | `button_bg` / `button_hover` | None / rgba(0.3, 0.3, 0.3, 0.5) |
| 文本 | `text_primary` / `text_secondary` | White / rgb(0.7, 0.7, 0.7) |
| 伤害 | `damage_color` / `crit_color` / `heal_color` | 黄 / 红 / 绿 |
| 范围 | `movable_range` / `attack_range` | 蓝(0.4α) / 红(0.35α) |
| 高亮 | `selection_highlight` | 黄(0.5α) |
| 进度条 | `hp_bar_color` / `mp_bar_color` / `stamina_bar_color` | 红 / 蓝 / 绿 |
| Buff | `buff_color` / `debuff_color` | 绿 / 红 |

### 4.2 字号常量

| 字段 | 默认值 | 用途 |
|------|--------|------|
| `font_large` | 24.0 | 大标题 |
| `font_medium` | 18.0 | 正文 |
| `font_small` | 14.0 | 小字 |
| `font_menu` | 16.0 | 菜单按钮 |
| `font_log` | 13.0 | 战斗日志 |
| `font_damage` | 16.0 | 伤害数字 |
| `font_crit` | 22.0 | 暴击数字 |

### 4.3 阵营颜色映射

| 阵营 | 颜色 |
|------|------|
| Player | rgb(0.2, 0.5, 1.0) 蓝色系 |
| Enemy | rgb(1.0, 0.3, 0.2) 红色系 |

已行动单位：饱和度 ×0.2，亮度 ×0.5 + 0.25（变灰）。

---

## 5. UiFocusState — 焦点管理

```rust
#[derive(Component)]
pub struct BlocksGameInput;  // 标记组件

#[derive(Resource)]
pub struct UiFocusState {
    pub blocks_input: bool,
}
```

**规则**：
- 拥有 `BlocksGameInput` 组件的面板会阻止游戏输入
- `update_ui_focus_state` 系统自动检测并更新 `blocks_input`
- 游戏输入系统读取 `UiFocusState` 决定是否跳过

---

## 6. 高亮与标记

### 6.1 标记组件

| 组件 | 说明 |
|------|------|
| `Selected` | 选中单位 |
| `MovableRange` | 可移动范围标记 |
| `AttackRange` | 攻击范围标记 |
| `SelectionHighlight` | 选中高亮 |

### 6.2 范围显示

| 函数 | 说明 |
|------|------|
| `show_move_range()` | BFS 可达范围，蓝色半透明 |
| `show_attack_range()` | 曼哈顿距离范围，红色半透明 |
| `spawn_selection_highlight()` | 选中格子黄色高亮 |
| `clear_selection()` | 清除选中 + 范围标记 |
| `clear_markers()` | 仅清除范围标记 |

所有函数接收 `&UiTheme` 参数，响应运行时主题变更。

---

## 7. handle_ui_commands — 命令处理器

### 7.1 核心职责

将 UiCommand Message 转化为游戏状态变更：
- 修改 TurnPhase
- 设置 CombatIntent
- 生成 MovingUnit
- 显示/清除范围标记
- 发送 ForceEndTurn Message

### 7.2 执行条件

- `AppState::InGame`
- `player_turn()`：TurnState.current_faction == Player

### 7.3 关键流程

**MoveUnit 命令**：
```
1. 检查是否点击当前位置（原地不动 → ActionMenu）
2. 检查目标是否在可移动范围内
3. 计算路径（reconstruct_path）
4. 生成路径箭头（spawn_path_arrows）
5. 插入 MovingUnit 组件
6. 清除范围标记
```

**SelectTarget 命令**：
```
1. 检查点击位置是否有敌方单位
2. 检查是否在攻击范围内
3. 在范围内 → 设置 CombatIntent → ExecuteAction
4. 不在范围内 → 回到 ActionMenu
```

---

## 8. 模块结构

```
ui/
├── mod.rs              # 模块定义 + re-exports
├── plugin.rs           # UiPlugin（组合所有子插件）
├── events.rs           # UiCommand Message
├── command_handler.rs  # 命令处理器
├── view_models.rs      # ViewModel 定义 + 更新系统
├── theme.rs            # UiTheme 主题系统
├── focus.rs            # 焦点管理
├── highlight.rs        # 高亮与标记
├── camera.rs           # 相机控制
├── action_menu.rs      # 行动菜单
├── combat_preview.rs   # 战斗预览
├── combat_log_handler.rs   # 战斗日志表现层
├── combat_vfx_handler.rs   # 战斗飘字表现层
├── tile_info.rs        # 地形浮窗
├── vfx.rs              # 视觉效果
├── settings.rs         # 游戏设置
├── panels/             # 面板模块
│   ├── unit_info.rs    # 单位信息面板
│   ├── combat_log_panel.rs  # 战斗日志面板
│   ├── inventory_panel.rs   # 背包面板
│   ├── turn_indicator.rs    # 回合指示器
│   └── action_hint.rs       # 行动提示
└── widgets/            # 基础组件库
    ├── layout.rs       # 布局工具
    ├── resource_bar.rs # 资源条
    └── popup.rs        # 弹窗
```

---

## 9. 战斗日志表现层

combat_log_handler 监听 Message 写入 CombatLog：

| 系统 | 监听 Message |
|------|-------------|
| `on_damage_applied` | DamageApplied |
| `on_heal_applied` | HealApplied |
| `on_character_died_log` | CharacterDied |
| `on_stun_applied` | StunApplied |
| `on_dot_applied` | DotApplied |
| `on_hot_applied` | HotApplied |
| `on_item_equipped` | ItemEquipped |
| `on_item_unequipped` | ItemUnequipped |

---

## 10. 战斗飘字表现层

combat_vfx_handler 监听 Message 生成飘字：

| 系统 | 监听 Message |
|------|-------------|
| `on_damage_vfx` | DamageApplied |
| `on_dot_vfx` | DotApplied |

---

## 11. 关键约束

1. **UI 不操作 ECS，只发 UiCommand**：所有 UI→Logic 交互通过 Message
2. **ViewModel 隔离**：UI 只读 ViewModel，不直接 Query 游戏组件
3. **HoveredEntity 驱动刷新**：SelectedUnitView 仅在悬停变化时重建
4. **主题统一样式**：所有颜色/字号/间距从 UiTheme 读取
5. **BlocksGameInput 阻止输入**：模态面板标记此组件
6. **handle_ui_commands 仅玩家回合**：`player_turn()` 条件过滤
7. **Cancel 上下文推断**：根据 skill_id / 菜单实体推断回退阶段
8. **MoveUnit 原地不动**：点击当前位置直接进入 ActionMenu
9. **已行动单位变灰**：饱和度 ×0.2，亮度调整
10. **Reflect 注册统一管理**：所有 ViewModel 类型在 UiPlugin 中注册
