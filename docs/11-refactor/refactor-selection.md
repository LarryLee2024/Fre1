# 重构计划：Selection 状态管理重构（Entity → PickTarget，分离 Focus/Selection）

> 关联 ADR: ADR-068 §Forbidden (F2, F6, F10) | docs/06-ui/01-architecture/architecture.md §7 UI 状态分级

## 目标状态 (Target)

### 状态分层

| 状态 | 类型 | 存储 | 生命周期 | 数据 | 职责 |
|------|------|------|---------|------|------|
| **Hovered** | L3 瞬态 | `HoverState` Resource | 帧级 | `Option<PickTarget>` | 悬停预览、Tooltip 触发 |
| **Focused** | L3 瞬态→会话 | `FocusState` Resource | 跨交互 | `Option<PickTarget>` | 键盘/手柄导航焦点 |
| **Selected** | L2 会话 | `SelectionState` Resource | 行动级 | `Option<BattleUnitId>`, `SelectionPhase` | 当前操作单位 |
| **Targeting** | L2 会话 | `SelectionState.phase` | 子行动 | `PickContext`, `Vec<GridPos>` | 技能/攻击目标范围 |
| **Activated** | 瞬态 | Event | 帧级 | `CombatAction` 事件 | 行动执行中标记 |

### SelectionState 完整定义

```rust
/// SelectionState — 选择状态机。
/// 
/// 生命周期：从玩家选中单位到执行完一个行动。
/// 由 UnitClicked 领域事件驱动。
#[derive(Resource, Debug, Clone, Reflect)]
pub struct SelectionState {
    /// 当前选中的单位（使用 BattleUnitId，禁止使用 Entity）
    pub selected_unit: Option<BattleUnitId>,
    /// 当前选择阶段
    pub phase: SelectionPhase,
    /// 最近一次选中的时间戳（用于 debounce）
    pub last_selected_at: Option<f64>,
}

/// SelectionPhase — 选择阶段。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum SelectionPhase {
    /// 未选中任何单位
    Idle,
    /// 已选中单位，等待选择行动
    UnitSelected,
    /// 已选择行动（移动/攻击/技能），等待选择目标或确认
    ActionSelected { action_type: ActionType },
    /// 目标选择中（显示范围指示器）
    Targeting { 
        context: PickContext,
        valid_targets: Vec<GridPos>,
    },
}

/// ActionType — 可行的行动类型。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum ActionType {
    Move,
    Attack,
    CastSkill(SkillId),
    UseItem(ItemId),
    Wait,
    EndTurn,
}

/// HoverState — 悬停状态（帧级瞬态）。
#[derive(Resource, Default, Debug, Clone, Reflect)]
pub struct HoverState {
    /// 当前悬停的目标
    pub hovered_target: Option<PickTarget>,
    /// 上一个悬停的目标（用于 Out 事件判断）
    pub prev_hovered: Option<PickTarget>,
}

/// FocusState — 焦点状态（会话级）。
#[derive(Resource, Default, Debug, Clone, Reflect)]
pub struct FocusState {
    pub focused_target: Option<PickTarget>,
    pub focus_group: Option<String>,
}
```

### 状态转移图

```
                    ┌──────────────┐
                    │    Idle      │
                    └──────┬───────┘
                           │
                    UnitClicked(unit_id)
                    context=Normal
                           │
                           ▼
                    ┌──────────────┐
                    │ UnitSelected  │
                    └──────┬───────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
     ActionMenu::    ActionMenu::   ActionMenu::
     MoveSelected   AttackSelected  SkillSelected
              │            │            │
              ▼            ▼            ▼
     ┌──────────┐  ┌──────────┐  ┌──────────┐
     │ Targeting │  │ Targeting │  │ Targeting │
     │(Move)     │  │(Attack)   │  │(Skill)    │
     └─────┬────┘  └─────┬────┘  └─────┬────┘
           │             │             │
     TileClicked    UnitClicked    UnitClicked
     (dest)         (target)      (target)
           │             │             │
           ▼             ▼             ▼
     ┌──────────┐
     │   Idle   │  ← 行动执行完毕或取消
     └──────────┘
```

## 当前状态 (Current)

- `src/infra/picking/selection.rs` — `Selection { selected_unit: Option<Entity> }`
- `src/ui/projections/selection.rs` — 混合选中/高亮/Camera 控制/技能面板填充
- `render.rs` — 视觉高亮逻辑（`on_unit_hover`, `on_unit_unhover`）
- 没有 `HoverState` — Hover 信息丢失在瞬态 Observer 中
- 没有 `FocusState` — Focus 和 Selection 混在一起
- `SelectionPhase` 不存在 — 选择/瞄准/行动没有阶段区分
- `Entity::to_bits() as u32` 作为 character_id（不稳定、不可靠）

## 差距分析

| # | 当前 | 目标 | 等级 |
|---|------|------|------|
| S1 | `Selection { selected_unit: Option<Entity> }` | `SelectionState { selected_unit: Option<BattleUnitId>, phase: SelectionPhase }` | 🟥 |
| S2 | 无 HoverState | `HoverState` Resource 独立管理 | 🟥 |
| S3 | 无 FocusState | `FocusState` Resource 独立管理 | 🟥 |
| S4 | 无 SelectionPhase | SelectionPhase 四状态（Idle/UnitSelected/ActionSelected/Targeting） | 🟥 |
| S5 | `entity.to_bits() as u32` | `BattleUnitId` （字符串或数值 ID） | 🟥 |
| S6 | `is_changed()` 轮询 | `SelectionChanged` 事件驱动 | 🟨 |
| S7 | 视觉高亮在 Projection 层 | `highlight_system.rs` 独立模块 | 🟨 |
| S8 | SkillPanel 硬编码数据填充 | 通过 Domain QueryParam 获取真实数据 | 🟥 |

## 迁移步骤

### 阶段 1: 定义新状态类型

#### 1.1 新建 `src/ui/picking/foundation/selection_state.rs`

```rust
use bevy::prelude::*;
use crate::core::domains::combat::components::BattleUnitId;
use crate::core::domains::tactical::components::GridPos;
use crate::ui::picking::foundation::pick_context::PickContext;

/// BattleUnitId — 单位领域 ID（临时定义，待 @data-architect 定义正式类型）
/// 当前使用字符串 ID，后续应升级为强类型 ID
pub type BattleUnitId = String;

#[derive(Resource, Debug, Clone, Reflect)]
pub struct SelectionState {
    pub selected_unit: Option<BattleUnitId>,
    pub phase: SelectionPhase,
    pub last_selected_at: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum SelectionPhase {
    Idle,
    UnitSelected,
    ActionSelected { action_type: ActionType },
    Targeting { context: PickContext, valid_targets: Vec<GridPos> },
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum ActionType {
    Move,
    Attack,
    CastSkill(SkillId),
    UseItem(ItemId),
    Wait,
    EndTurn,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            selected_unit: None,
            phase: SelectionPhase::Idle,
            last_selected_at: None,
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct SelectionChanged {
    pub previous: Option<BattleUnitId>,
    pub current: Option<BattleUnitId>,
    pub previous_phase: SelectionPhase,
    pub current_phase: SelectionPhase,
}
```

#### 1.2 新建 `src/ui/picking/foundation/hover_state.rs`

```rust
use bevy::prelude::*;
use crate::ui::picking::foundation::pick_target::PickTarget;

#[derive(Resource, Default, Debug, Clone, Reflect)]
pub struct HoverState {
    pub hovered_target: Option<PickTarget>,
    pub prev_hovered: Option<PickTarget>,
}
```

#### 1.3 新建 `src/ui/picking/foundation/focus_state.rs`

```rust
use bevy::prelude::*;
use crate::ui::picking::foundation::pick_target::PickTarget;

#[derive(Resource, Default, Debug, Clone, Reflect)]
pub struct FocusState {
    pub focused_target: Option<PickTarget>,
    pub focus_group: Option<String>,
}
```

### 阶段 2: 实现 Selection 状态机

#### 2.1 新建 `src/ui/picking/selection/selection_manager.rs`

实现状态机核心逻辑：

```rust
/// 处理 UnitClicked 领域事件 → 更新 SelectionState
pub fn handle_unit_clicked(
    ev: On<UnitClicked>,
    mut selection: ResMut<SelectionState>,
    mut sel_changed: EventWriter<SelectionChanged>,
    time: Res<Time>,
) {
    let previous = selection.selected_unit.clone();
    let prev_phase = selection.phase.clone();
    
    match ev.unit_id {
        None => {
            // 空点击 → 取消选择
            *selection = SelectionState::default();
        }
        Some(unit_id) => {
            // 选中或切换单位
            selection.selected_unit = Some(unit_id);
            selection.phase = SelectionPhase::UnitSelected;
            selection.last_selected_at = Some(time.elapsed_seconds_f64());
        }
    }
    
    // 通知下游
    sel_changed.send(SelectionChanged {
        previous,
        current: selection.selected_unit.clone(),
        previous_phase: prev_phase,
        current_phase: selection.phase.clone(),
    });
}

/// 处理 ActionMenu 行动选择 → 进入 Targeting 阶段
pub fn handle_action_selected(
    ev: On<ActionSelected>,
    mut selection: ResMut<SelectionState>,
    mut sel_changed: EventWriter<SelectionChanged>,
) {
    let prev_phase = selection.phase.clone();
    selection.phase = SelectionPhase::ActionSelected { action_type: ev.action_type };
    // 触发可视化更新
    sel_changed.send(SelectionChanged {
        previous: selection.selected_unit.clone(),
        current: selection.selected_unit.clone(),
        previous_phase: prev_phase,
        current_phase: selection.phase.clone(),
    });
}
```

#### 2.2 新建 `src/ui/picking/selection/highlight_system.rs`

独立视觉高亮系统，只修改 `Sprite.color`，不包含业务逻辑：

```rust
/// 统一的视觉高亮系统。
/// 读取 SelectionState + HoverState，不修改它们。
pub fn apply_visual_highlights(
    selection: Res<SelectionState>,
    hover: Res<HoverState>,
    mut sprites: Query<&mut Sprite>,
    unit_query: Query<&UnitIdComponent>,
) {
    // 1. 高亮选中的单位（青色）
    if let Some(ref unit_id) = selection.selected_unit {
        for (entity, uid) in unit_query.iter() {
            if &uid.id == unit_id {
                if let Ok(mut sprite) = sprites.get_mut(entity) {
                    sprite.color = SELECTED_COLOR;
                }
            }
        }
    }
    
    // 2. 高亮悬停的单位（金色），但不要覆盖选中色
    if let Some(PickTarget::Unit(ref hovered_id)) = hover.hovered_target {
        if Some(hovered_id) != selection.selected_unit.as_ref() {
            for (entity, uid) in unit_query.iter() {
                if &uid.id == hovered_id {
                    if let Ok(mut sprite) = sprites.get_mut(entity) {
                        sprite.color = HIGHLIGHT_COLOR;
                    }
                }
            }
        }
    }
    
    // 3. 恢复非选中/非悬停单位的队伍色
    for (entity, uid) in unit_query.iter() {
        // 跳过选中和悬停的单位
        // ...
        // 恢复为队伍色
    }
}
```

### 阶段 3: 重构 Projection（消除硬编码数据）

**关键变更** — 当前 `src/ui/projections/selection.rs` 中有大量硬编码数据：

```rust
// ❌ 当前 — 硬编码数据
store.character_panel.character_id = entity.to_bits() as u32;
store.character_panel.name_key = uid.id.clone();
store.character_panel.level = 1;           // ❌ 硬编码
store.character_panel.hp = hp.current as f32;
store.character_panel.max_hp = hp.maximum as f32;
store.character_panel.mp = 50.0;           // ❌ 硬编码
store.character_panel.max_mp = 50.0;       // ❌ 硬编码

// ✅ 目标 — 通过 Domain QueryParam 读取
pub fn on_selection_changed_projection(
    ev: On<SelectionChanged>,
    mut store: ResMut<UiStore>,
    query_param: TacticalQueryParam,       // ← Domain integration 层
) {
    let Some(ref unit_id) = ev.current else { return; };
    
    // 使用 Domain 的 QueryParam 读取真实数据
    if let Some(stats) = query_param.get_unit_stats(unit_id) {
        store.character_panel.hp = stats.hp.current as f32;
        store.character_panel.max_hp = stats.hp.maximum as f32;
        store.character_panel.mp = stats.mp.current as f32;
        store.character_panel.max_mp = stats.mp.maximum as f32;
        store.character_panel.level = stats.level;
        store.character_panel.name_key = stats.name_key;
    }
    
    // 技能数据也应该来自 Domain，而非硬编码 HashMap
    
    // 标记 Dirty
}
```

### 阶段 4: 移除旧代码

#### 4.1 删除 `src/infra/picking/selection.rs`

内容已迁移到 `SelectionState`。

#### 4.2 修改 `src/app/scenes/test_battle/render.rs`

删除 `on_unit_hover` 和 `on_unit_unhover`（已迁移到 highlight_system.rs）。

#### 4.3 修改 `src/ui/projections/selection.rs`

- 删除 `on_selection_visual` 函数
- 删除 `camera_follow_selection` 函数
- 删除硬编码的 SkillPanel 数据填充
- 精简 `on_selection_changed` 为纯 Projection 职责

### 阶段 5: 更新引用

搜索所有引用旧类型的地方：

| 旧类型 | 新类型 | 涉及文件 |
|--------|--------|---------|
| `Selection` | `SelectionState` | 所有 import |
| `selected_unit: Option<Entity>` | `selected_unit: Option<BattleUnitId>` | selection_manager, projection |
| `entity.to_bits() as u32` | `UnitIdComponent.id` (String) | projections/selection.rs |
| `selection.is_changed()` | `On<SelectionChanged>` 事件 | projections/selection.rs |

## 风险与缓解

| # | 风险 | 影响 | 缓解 |
|---|------|------|------|
| R1 | Entity→BattleUnitId 转换增加查询开销 | 性能 | 使用 HashMap 缓存 Entity→BattleUnitId 映射 |
| R2 | SelectionPhase 引入增加复杂度 | 状态管理 | 初始实现只支持 Idle/UnitSelected 两个阶段，后续扩展 |
| R3 | HoverState 帧级瞬态与多帧处理冲突 | 悬停闪烁 | 在 PostUpdate 末清除 HoverState（确保一帧内有足够时间消费） |
| R4 | BattleUnitId 还未被 data-architect 定义 | 类型不确定 | 临时使用 String，最终替换为强类型 ID |

## 预估工作量

| 阶段 | 文件操作 | 行数 | 时间 |
|------|---------|------|------|
| 1. 定义新状态类型 | 新建 3 个文件 | ~120 行 | 1h |
| 2. 状态机实现 | 新建 2 个文件 | ~150 行 | 1.5h |
| 3. 重构 Projection | 修改 1 个文件 | ~100 行 | 2h |
| 4. 移除旧代码 | 修改 3 个文件 | ~-100 行 | 0.5h |
| 5. 更新引用 | 搜索替换 | ~10 行 | 0.5h |
| **合计** | | **~280 行** | **~5.5h** |
