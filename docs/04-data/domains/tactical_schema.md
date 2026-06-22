---
id: domains.tactical.schema.v1
title: Tactical Schema — 战术空间数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: instance
replay-safe: true
---

# Tactical Schema — 战术空间数据架构

> **领域归属**: Domains — 战术空间层 | **依赖 Schema**: Tag, Terrain | **定义依据**: `docs/02-domain/domains/tactical_domain.md`

---

## 1. Schema Design

### GridPosition（Instance 层）

```rust
struct GridPosition {
    x: i32,
    y: i32,
    layer: i8,          // 层高（0=地面, 1=高地, -1=地下）
}
```

### MovementPoints（Instance 层）

```rust
struct MovementPoints {
    current: f32,
    max: f32,
    consumed: f32,
    movement_type: MovementType,
}

enum MovementType { Walk, Fly, Swim, Climb, Teleport }
```

### Facing（Instance 层）

```rust
struct Facing {
    direction: HexDirection,
}

enum HexDirection { N, NE, SE, S, SW, NW }
```

### FlankingState / CoverState / HighgroundState（Runtime 层 — 瞬时判定结果）

```rust
struct FlankingState { is_flanked: bool, flankers: Vec<EntityId> }
struct CoverState { cover_level: CoverLevel, cover_source: Option<EntityId> }
enum CoverLevel { None, Half, ThreeQuarters, Full }
struct HighgroundState { height_diff: i8, has_advantage: bool }
```

### PathData（Runtime 层）

```rust
struct PathData { waypoints: Vec<GridPosition>, total_cost: f32, is_valid: bool }
```

---

## 2. Layer Summary

Data is primarily Instance/Runtime — positions and movement are tracked per-entity. Grid data itself (map layout, tile data) is managed by Terrain.

---

## 3. Dependency Analysis

| 依赖 | 说明 |
|------|------|
| → TerrainSchema | Tile 通行性、高度数据 |
| → TagSchema | MovementType 可表达为 Tag |
| ← CombatSchema | 战斗中的夹击/高地判定 |
| ← TargetingSchema | 射程/视野计算 |

---

## 4. Replay & Save

Movement commands recorded in ReplayLog. Positions serialized in entity save data.

---

## 6. Selection — 选择交互状态 Schema

### SelectionState（Instance 层 — Resource）

```rust
/// 选择交互状态 — 五态分离的非持久化 Resource。
///
/// 五态设计原因：
/// - Hovered 帧级、Focused 会话级、Selected 行动级、Targeted 瞄准级、Activated 执行级
/// - 五种状态生命周期不同，合并则造成生命周期冲突
/// - 分离后各状态可独立监听、独立响应
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct SelectionState {
    /// 当前悬停目标 — 鼠标指针当前所在的交互对象。
    /// 生命周期：帧级瞬态。每帧 Pointer<Over> 设置，Pointer<Out> 清除
    pub hovered: Option<PickTarget>,

    /// 当前焦点目标 — 键盘/手柄导航的当前位置。
    /// 生命周期：会话级（Tab/方向键切换）
    pub focused: Option<PickTarget>,

    /// 当前选中目标 — 玩家主动选中的主要交互对象。
    /// 生命周期：行动级（选中后保持到取消或行动完成）
    pub selected: Option<PickTarget>,

    /// 当前瞄准目标 — 对选中单位执行行动时的目标。
    /// 生命周期：瞄准阶段（选择行动到确认目标之间）
    /// 约束：存在时 selected 必须也存在
    pub targeted: Option<PickTarget>,

    /// 当前激活目标 — 正在执行中的行动目标。
    /// 生命周期：执行阶段（目标确认后到执行完成前）
    /// 约束：此字段设置期间禁止输入
    pub activated: Option<PickTarget>,

    /// 当前选择上下文
    pub context: PickContext,
}
```

**字段约束表**：

| 字段 | 类型 | 默认值 | 生命周期 | 更新频率 |
|------|------|--------|---------|---------|
| `hovered` | `Option<PickTarget>` | `None` | 帧级瞬态 | 每帧 0-N 次 |
| `focused` | `Option<PickTarget>` | `None` | 会话级 | 低频 |
| `selected` | `Option<PickTarget>` | `None` | 行动级 | 每次选择/取消 |
| `targeted` | `Option<PickTarget>` | `None` | 瞄准阶段 | 每次瞄准更新 |
| `activated` | `Option<PickTarget>` | `None` | 执行阶段 | 每次行动确认 |
| `context` | `PickContext` | `Normal` | 模式级 | 每次模式切换 |

**合法组合表**：

| 组合 | hovered | focused | selected | targeted | activated | 合法性 |
|------|---------|---------|----------|----------|-----------|--------|
| IDLE | None | None | None | None | None | ✅ |
| HVR | Some | None | None | None | None | ✅ |
| SEL | None | Any | Some | None | None | ✅ |
| SEL+TGT | Any | Any | Some | Some | None | ✅ |
| ACT | None | Any | None | None | Some | ✅ |
| SEL+ACT | Any | Any | Some | None | Some | ❌ |
| TGT-ONLY | Any | Any | None | Some | None | ❌ |

### 状态转移矩阵

```
当前状态          事件                         新状态
─────────        ────                         ──────
IDLE             UnitClicked (Normal)          SEL
SEL              UnitClicked (Normal/不同)      SEL (切换)
SEL              TileClicked (Normal)           IDLE
SEL              PickTarget::Empty              IDLE
SEL              AttackTargeting context set     SEL+TGT
SEL+TGT          UnitClicked (合法目标)          SEL+TGT (更新)
SEL+TGT          TargetConfirmed                 ACT
ACT              ExecutionComplete               IDLE
Any              PickTarget::Empty               IDLE (紧急清除)
Any              新帧开始                       清除 hovered
```

### 与 PickContext 的关系

```
  Normal:            UnitClicked → UnitSelected (设置 selected)
                     TileClicked → 清除 selected / 显示 TileInfo
                     Click Empty → 清除 selected

  AttackTargeting:   UnitClicked(敌方) → TargetConfirmed
                     Click Empty → 取消瞄准 → Normal

  SkillTargeting:    UnitClicked(合法) → TargetConfirmed
                     TileClicked(合法) → TargetConfirmed
                     Click Empty → 取消瞄准 → Normal

  Inspect:           UnitClicked → 显示详情面板
                     Click Empty → 关闭面板 → Normal
```

### Layer Summary

| 数据结构 | 层 | 持久化 | Replay |
|---------|----|--------|--------|
| `SelectionState` | Instance / Resource | 不存档 | 不回放 |
| `SelectionState::hovered` | Instance (帧级瞬态) | 不存档 | 不回放 |

### Replay & Save

**Replay**: `SelectionState` 不进 Replay log。在 Replay 回放时通过消费 `UnitClicked` / `TileClicked` / `TargetConfirmed` 事件自动重建。`hovered` 在 Replay 中始终为 `None`（鼠标位置不确定）。

**Save**: `SelectionState` 不存入存档。读档后从 `new()` 初始化，玩家需要重新选中单位。
