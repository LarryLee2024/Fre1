---
id: domains.terrain.schema.v1
title: Terrain Schema — 地形数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition, instance, persistence
replay-safe: true
---

# Terrain Schema — 地形数据架构

> **领域归属**: Domains — 战术空间层 | **依赖 Schema**: Tag, Effect, Event | **定义依据**: `docs/02-domain/terrain_domain.md`

---

## 1. Schema Design

### 1.1 TileProperties（Definition 层 + Instance 层混合）

```rust
/// 格子的地形属性。Definition 层定义静态配置，运行时表面可被修改。
struct TileProperties {
    /// 格子坐标（x, y, layer）
    position: GridPosition,

    /// 地形类型（Definition — 静态）
    terrain_type: TerrainType,

    /// 基础通行性（Definition — 静态）
    base_passability: Passability,

    /// 基础遮蔽度（Definition — 静态）
    base_concealment: Concealment,

    /// 当前表面类型（Instance — 运行时可变）
    surface: SurfaceType,

    /// 原始表面类型（Instance — 用于恢复）
    original_surface: SurfaceType,
}

enum TerrainType {
    Normal,        // 平地，无特殊效果
    Highground,    // 高地，提供视野/命中优势
    Obstacle,      // 障碍，不可通行，提供掩体
    Water,         // 水域，通行消耗翻倍
    Bush,          // 丛林，提供隐蔽
    Ice,           // 冰面，移动消耗翻倍
    Poison,        // 毒池，进入施加中毒
    Burning,       // 灼烧，每回合燃烧伤害
    Oil,           // 油面，可被点燃
    Lava,          // 岩浆，极高伤害
}

enum Passability {
    Walkable,      // 可行走
    Blocked,       // 阻挡（障碍/边界）
    Flyable,       // 飞行单位可越过
    Impassable,    // 所有单位不可通行
}

enum Concealment {
    None,          // 完全可见
    Half,          // 半遮蔽，隐蔽 -2 命中
    Full,          // 不可见，无法作为目标
}
```

### 1.2 SurfaceOverride（Instance 层）

```rust
/// 表面类型的变化记录。每个表面变化必须可恢复。
struct SurfaceOverride {
    /// 当前表面类型
    current: SurfaceType,

    /// 原始表面类型（用于恢复）
    original: SurfaceType,

    /// 该覆盖的持续回合数
    remaining_duration: Option<u32>,

    /// 恢复方式
    recovery: SurfaceRecovery,
}

enum SurfaceType { Normal, Ice, Oil, Water, Poison, Burning, Lava }

enum SurfaceRecovery {
    /// 到期自动恢复
    Timed { total_duration: u32 },
    /// 被驱散恢复
    Dispel,
    /// 显式声明为永久变化（需 [Data Exemption]）
    Permanent,
}
```

### 1.3 TerrainAttachEffect（Instance 层/Persistence 层）

```rust
/// 绑定到格子的地形效果。Terrain 领域只记录「哪个格子挂了哪些效果」，
/// 效果的实际生命周期由 Effect 领域管理。
struct TerrainAttachEffect {
    /// 绑定的格子位置
    tile: GridPosition,

    /// 引用的 EffectDefId
    effect_id: EffectDefId,

    /// 剩余持续时间（回合数），None = 永久
    remaining_duration: Option<u32>,
}
```

### 1.4 HazardZoneDef（Definition 层）

```rust
/// 危险区域/陷阱定义。静态配置，运行时只读。
struct HazardZoneDef {
    /// 陷阱唯一标识（前缀: `haz_`）
    id: HazardZoneId,

    /// 区域形状与范围
    area: AreaDefinition,

    /// 触发条件
    trigger_condition: HazardTriggerCondition,

    /// 触发后执行的效果 EffectDefId 列表
    effects: Vec<EffectDefId>,

    /// 该陷阱是否为消耗型（一次触发后失效）
    is_consumable: bool,

    /// 可见性控制（隐藏陷阱/可见区域）
    visibility: HazardVisibility,
}

struct AreaDefinition {
    shape: AreaShape,
    radius: u32,           // 半径（格数）
    offset: GridPosition,  // 相对偏移
}

enum AreaShape { Single, Cross, Square, Circle, Line, Cone }

enum HazardTriggerCondition {
    /// 单位进入时触发
    OnEnter {
        exclude_factions: Vec<FactionDefId>,
        exclude_types: Vec<UnitType>,
    },
    /// 单位在区域内停留时每回合触发
    OnStay {
        exclude_factions: Vec<FactionDefId>,
    },
    /// 回合结束时触发
    OnRoundEnd,
    /// 被技能/效果主动激活
    OnActivate,
}

enum HazardVisibility {
    Visible,       // 开战时可见
    Hidden,        // 隐藏，需要侦查发现
    RevealedOn,    // 特定条件触发后可见
}
```

### 1.5 TerrainInteractionDef（Definition 层）

```rust
/// 地形交互定义。描述技能/效果如何改变地形。
struct TerrainInteractionDef {
    /// 目标表面类型
    target_surface: SurfaceType,

    /// 持续时间
    duration: InteractionDuration,

    /// 是否与现有表面冲突
    conflict_check: bool,
}

enum InteractionDuration {
    Instant,            // 立即生效，不可恢复
    Timed { turns: u32 }, // 持续 N 回合后恢复
    Permanent,           // 永久改变（需 [Data Exemption]）
}
```

### 1.6 TerrainState（Persistence 层）

```rust
/// 地形持久化状态。存档时保存所有被修改过的格子。
struct TerrainState {
    /// 所有被覆盖了表面的格子（只保存有变化的格子，节约空间）
    surface_overrides: Vec<(GridPosition, SurfaceOverride)>,

    /// 所有激活的地形效果（格子绑定）
    active_tile_effects: Vec<TerrainAttachEffect>,

    /// 陷阱消耗状态（消耗型陷阱是否已被触发）
    hazard_consumed: Vec<HazardZoneId>,
}
```

---

## 2. Layer Summary

| Layer | Structures | 说明 |
|-------|-----------|------|
| **Definition** | `TerrainType`, `Passability`, `Concealment` (enum), `HazardZoneDef`, `TerrainInteractionDef` | 地形类型、陷阱定义、交互规则为静态配置 |
| **Spec** | — | Terrain 无 Spec 层；地形效果通过 Effect 的 Spec 机制表达 |
| **Instance** | `SurfaceOverride`, `TerrainAttachEffect` | 表面变化记录、动态地形效果绑定 |
| **Persistence** | `TerrainState` | 存档只保存有变化的格子，不保存整个地图 |

---

## 3. Dependency Analysis

| 依赖 | 说明 |
|------|------|
| → TagSchema | SurfaceType 可表达为 Tag，通行性标记 |
| → EffectSchema | TerrainAttachEffect 引用 EffectDefId |
| → EventSchema | 地形变化发布 SurfaceChanged/HazardTriggered 事件 |
| → FactionSchema | HazardTriggerCondition 引用 FactionDefId 排除阵营 |
| ← TacticalSchema | 移动/掩体判定消费通行性和遮蔽度数据 |
| ← CombatSchema | 回合中检查地形效果 |
| ← TargetingSchema | 视野校验消费遮蔽度数据 |

---

## 4. Replay & Save

### Replay

- 地形表面变化由 Effect/Ability 触发 → 作为 Effect 的一部分被录制为 Command
- 陷阱触发由单位移动触发 → 移动 Command 中包含进入的格子信息
- 确定性：TerrainType/Passability/Concealment 为静态配置，SurfaceOverride 的恢复由计时器驱动（GameTime 帧计数）

### Save

- `TerrainState` 只保存被修改过的格子（surface_overrides），而非整个地图
- 未修改的格子从地图配置（assets/config/maps/）中重建
- 消耗型陷阱状态（hazard_consumed）持久化，防止读档后陷阱重生

---

## 5. Validation Rules

| 规则 | 说明 | 违反处理 |
|------|------|----------|
| Surface 可逆性 | 所有 SurfaceOverride 必须有 recovery 机制（除非显式 Permanent） | Schema 校验拒绝 |
| 互斥表面 | 禁止同一格同时存在互斥表面（如 Ice+Burning） | 运行时拒绝覆盖 |
| Height 连续性 | 相邻格高度差不得超过预设最大值 | 地图加载时校验 |
| Hazard 触发条件 | 每个 HazardZoneDef 必须有明确的 trigger_condition | Schema 校验拒绝 |
| Effect 引用有效 | TerrainAttachEffect 的 effect_id 必须在 Registry 中存在 | 运行时断言 |

---

## 6. Constitution Check

- ✅ **Data Law 001 (Def-Instance分离)**: TileProperties 定义与表面状态分离；HazardZoneDef 为纯 Definition
- ✅ **Data Law 002 (Rule-Content分离)**: 表面变化规则属于代码，地形类型枚举属于内容
- ✅ **Data Law 003 (配置只引用ID)**: TerrainAttachEffect 引用 EffectDefId，不嵌入效果定义
- ✅ **Data Law 005 (Effect是唯一业务执行入口)**: 地形效果通过 TerrainAttachEffect.effect_id → Effect 领域执行
- ✅ **Data Law 009 (表现必须经过Cue)**: 表面变化通过 Event → Cue 触发特效
- ✅ **Data Law 010 (Replay优先)**: 表面变化由 Effect 驱动，Effect 本身具有 replay 确定性
- ✅ **Data Law 011 (Schema版本化)**: 本 schema 携带版本号，Persistence 层 TerrainState 支持迁移
- ✅ **Data Law 012 (域间禁止直接数据引用)**: Terrain 通过 Event 向外通信，不直接引用其他领域的组件
