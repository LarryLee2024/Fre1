---
id: infrastructure.picking.schema.v1
title: Picking Schema — 选择交互层数据架构
status: draft
owner: data-architect
created: 2026-06-23
updated: 2026-06-23
layer: instance, transient
replay-safe: true
---

# Picking Schema — 选择交互层数据架构

> **领域归属**: Infrastructure — Picking | **依赖 Schema**: Tactical (GridPos), Event | **架构依据**: `docs/01-architecture/40-cross-cutting/ADR-068-picking-architecture-constitution.md`, `docs/01-architecture/40-cross-cutting/🔒ADR-067-sprite-picking-architecture.md`
>
> **重要**: 本 Schema 定义 Picking 系统的数据类型。Picking 运行时归属 Presentation Layer (L3) 的输入适配子层 (`src/ui/picking/`)，但其基础类型（ID、枚举）作为跨层合约在此定义。

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 数据层 | 说明 |
|----------|--------|--------|------|
| `BattleUnitId` | Shared — ID 系统 | Shared | 战场单位强类型 ID，跨层使用 |
| `PickTarget` | UI — Picking | Transient | 选择目标枚举，嵌入事件与状态中 |
| `PickContext` | UI — Picking | Instance | 当前选择上下文，Runtime Resource |
| `UnitClicked` | Core — Domain Event | Transient | 单位被点击的原始领域事件 |
| `UnitSelected` | Core — Domain Event | Transient | 单位变为选中状态的派生事件 |
| `SelectionChanged` | Core — Domain Event | Transient | 选择状态变化的通知事件 |
| `TileClicked` | Core — Domain Event | Transient | 瓦片被点击的原始领域事件 |
| `TargetConfirmed` | Core — Domain Event | Transient | 瞄准阶段目标确认的派生事件 |

---

## 2. Problem

当前项目中 Picking 数据层存在以下问题：

1. **缺少 BattleUnitId 强类型定义** — 战场单位 ID 当前使用 raw Entity 或 String，缺乏类型安全、序列化契约和确定性分配保证
2. **PickTarget 无统一 Schema** — 选择目标的多种变体（单位/瓦片/技能/物品）散落在不同模块，缺乏枚举级类型安全
3. **PickContext 与 Selection 状态耦合** — 选择上下文（Normal/Attack/Skill 等）影响点击行为，但当前数据结构未显式建模这一关系
4. **领域事件类型未标准化** — UnitClicked、SelectionChanged 等事件缺少字段级契约和 Data Law 合规审查
5. **Replay 边界模糊** — SelectionState 不应进入 Replay log，但其依赖的输入事件必须确定，当前无此区分

---

## 3. Schema Design

### 3.1 BattleUnitId（Shared 层 — 强类型 ID）

```rust
/// 战场单位全局唯一标识。
///
/// 与 UnitIdComponent.id 的关系：
/// - BattleUnitId 是 UnitIdComponent.id 的领域域类型（Domain Type）
/// - UnitIdComponent 是 ECS Component，存储 BattleUnitId 的值
/// - 内容配置中角色的 BattleUnitId 固定（如 "unit_000001" 永远是同一个角色）
/// - 但 BattleUnitId 的实体绑定在每次战斗开始时确定（同一角色在不同战斗中可能对应不同 Entity）
///
/// 分配规则：单位进入战场时按确定性顺序分配（由 Replay 种子决定），
/// 保证同一场战斗在 Replay 中产生相同的 BattleUnitId→Entity 映射。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BattleUnitId(pub String);
```

**字段约束表**：

| 字段 | 类型 | 默认值 | 约束 |
|------|------|--------|------|
| 内部值 | `String` | — | 格式 `unit_` + 6 位数字（如 `unit_000001`），禁止语义化名称 |

**设计决策**：
- `String` 而非 `u64`：与现有 ID 体系保持一致（所有 Def ID 均为 `prefix_6digit` 字符串格式），便于调试、序列化和内容团队使用
- `Serialize`/`Deserialize`：跨网络、跨存档、跨 Replay 必须可序列化
- `Hash` + `Eq`：用于 HashMap/HashSet 查找和状态比较

### 3.2 PickTarget（Transient 层 — 选择目标枚举）

```rust
/// 选择目标枚举 — Picking 系统的核心产出类型。
///
/// 表示玩家在当前交互中选中的"目标"是什么。
/// 每种变体对应不同类型的实体或交互对象。
/// PickTarget 本身是瞬态值类型，不作为持久化数据存储。
///
/// 与 Entity 的转换仅在 Presentation 层进行：
/// - PickTarget::Unit(id) → Entity：通过 UnitIdComponent 查询映射
/// - PickTarget::Tile(pos) → Entity：通过 TileComponent 查询映射（如需）
/// - Domain 层始终使用 PickTarget（含领域 ID），不直接操作 Entity
/// - 禁止在 Domain 层将 PickTarget 转换为 Entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PickTarget {
    /// 选中一个战场单位。
    Unit(BattleUnitId),

    /// 选中一个地图瓦片。
    Tile(GridPos),

    /// 选中一个技能/法术。
    Skill(SkillId),

    /// 选中一个物品。
    Item(ItemId),

    /// 点击空处（没有选中有意义的对象）。
    Empty,
}
```

**PickTarget 各变体使用场景矩阵**：

| 变体 | 来源 Input | 触发方式 | 主要消费者 | 跨帧持久？ |
|------|-----------|----------|-----------|-----------|
| `Unit(BattleUnitId)` | Sprite Picking | `Pointer<Click>` on Unit Sprite | Selection 状态机, Combat Domain | 是（选中保持） |
| `Tile(GridPos)` | Sprite/Grid Picking | `Pointer<Click>` on Tile | Tactical Domain (Pathfinding) | 否（瞬态） |
| `Skill(SkillId)` | UI Interaction | UI 技能面板点击 | Ability Domain | 否（瞬态） |
| `Item(ItemId)` | UI Interaction | UI 背包/快捷栏点击 | Inventory Domain | 否（瞬态） |
| `Empty` | Sprite Picking (miss) | `Pointer<Click>` on 空处 | Selection 状态机（取消选择） | 否（瞬态） |

**与 Entity 的转换规则**：

| 方向 | 允许？ | 位置 | 方法 |
|------|--------|------|------|
| `PickTarget::Unit(id)` → Entity | ✅ 仅在 Presentation 层 | `ui/selection/bridge.rs` | `UnitIdComponent` 查询表 |
| Entity → `PickTarget::Unit(id)` | ✅ 仅在 Presentation 层 | `ui/picking/intent/` | 读取 `UnitIdComponent::id` |
| `PickTarget::Tile(pos)` → Entity | ❌ 禁止 | — | Tile 无对应 Entity，直接使用 GridPos |
| PickTarget 跨层传递 | ✅ | 通过 Event 系统 | `Serialize`/`Deserialize` 保证 |

### 3.3 PickContext（Instance 层 — 选择上下文枚举）

```rust
/// 当前选择上下文 — 决定点击事件的语义解析方式。
///
/// PickContext 是全局性状态（通常作为 Resource），由当前游戏模式设置。
/// 当上下文变化时，所有未完成的 Selection 状态应当重置。
///
/// 上下文转换规则：
/// - Normal ↔ 任何其他上下文：直接切换，清空 active 状态（selected/targeted）
/// - AttackTargeting → Normal：攻击确认或取消后自动回退
/// - SkillTargeting → Normal：技能确认或取消后自动回退
/// - Inspect → Normal：关闭检视面板后自动回退
///
/// 状态机约束：同一时间只能有一个活跃上下文。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PickContext {
    /// 普通模式 — 默认交互上下文。
    Normal,

    /// 攻击瞄准模式 — 选择攻击目标。
    AttackTargeting,

    /// 技能瞄准模式 — 选择技能目标。
    SkillTargeting {
        /// 当前选择的目标技能 ID
        skill_id: SkillId,
    },

    /// 检视模式 — 查看单位/瓦片详情。
    Inspect,
}
```

**上下文转换矩阵**：

```
                    ┌──────────┐
                    │  Normal  │◄────────────────────┐
                    └────┬─────┘                     │
                         │                           │
              ┌──────────┼──────────┐                │
              ▼          ▼          ▼                │
        ┌──────────┐ ┌────────┐ ┌────────┐           │
        │  Attack  │ │  Skill │ │Inspect │           │
        │Targeting │ │Targetng│ │        │           │
        └────┬─────┘ └───┬────┘ └────────┘           │
             │           │                           │
             └───────────┼───────────────────────────┘
                         │ 确认/取消 → 回到 Normal
                         ▼
                    [执行行动]
```

| 源 \ 目标 | Normal | AttackTargeting | SkillTargeting | Inspect |
|-----------|--------|-----------------|----------------|---------|
| Normal | — | 进入攻击瞄准 | 选择技能后进入 | 进入检视 |
| AttackTargeting | 取消/完成 | — | 不允许直接转 | 不允许直接转 |
| SkillTargeting | 取消/完成 | 不允许直接转 | 切换技能 ID | 不允许直接转 |
| Inspect | 关闭面板 | 不允许直接转 | 不允许直接转 | — |

### 3.4 领域事件 Schema

所有领域事件继承相同的元属性：`#[derive(Event, Debug, Clone, PartialEq, Serialize, Deserialize)]`。

#### 3.4.1 UnitClicked（原始事件）

```rust
/// 玩家点击了一个战场单位。
///
/// 这是原始输入事件，在任何 PickContext 下都会触发。
/// 消费者根据 context 字段决定如何处理该点击。
/// 此事件在 Domain 层消费，不包含 Entity 引用。
pub struct UnitClicked {
    /// 被点击的单位 ID
    pub unit_id: BattleUnitId,
    /// 触发点击的鼠标按钮
    pub button: PointerButton,
    /// 点击时的 PickContext
    pub context: PickContext,
}
```

#### 3.4.2 UnitSelected（派生事件）

```rust
/// 单位被选中 — UnitClicked + Normal 上下文的处理结果。
pub struct UnitSelected {
    /// 新选中的单位 ID
    pub unit_id: BattleUnitId,
    /// 之前选中的单位 ID（如果没有则为 None）
    pub previous: Option<BattleUnitId>,
}
```

#### 3.4.3 TileClicked（原始事件）

```rust
/// 玩家点击了一个地图瓦片。
pub struct TileClicked {
    /// 被点击的瓦片位置
    pub tile: GridPos,
    /// 触发点击的鼠标按钮
    pub button: PointerButton,
    /// 点击时的 PickContext
    pub context: PickContext,
}
```

#### 3.4.4 TargetConfirmed（派生事件）

```rust
/// 瞄准目标已确认 — 在 AttackTargeting/SkillTargeting 上下文中点击合法目标的结果。
pub struct TargetConfirmed {
    /// 确认的瞄准目标（单位或瓦片位置）
    pub target: PickTarget,
    /// 确认时的瞄准上下文
    pub context: PickContext,
}
```

#### 3.4.5 SelectionChanged（通知事件）

```rust
/// 选择状态发生变化 — 任何 SelectionState 字段变更时触发。
pub struct SelectionChanged {
    /// 变化前的选择目标
    pub previous: Option<PickTarget>,
    /// 变化后的选择目标
    pub current: Option<PickTarget>,
    /// 变化的阶段/类型
    pub phase: SelectionPhase,
}

/// 选择变化阶段
pub enum SelectionPhase {
    Hovered,
    Focused,
    Selected,
    Targeted,
    Activated,
    Cleared,
}
```

---

## 4. Layer Analysis — 四层数据分配

| 类型 | Definition | Spec | Instance | Persistence | 说明 |
|------|-----------|------|----------|-------------|------|
| `BattleUnitId` | — | — | ✅ Shared | — | 领域 ID 类型，跨层值对象 |
| `PickTarget` | — | — | ✅ Transient | — | 事件/状态中嵌入的值类型 |
| `PickContext` | — | — | ✅ Resource | — | 运行时上下文状态 |
| `SelectionState` | — | — | ✅ Resource | — | 表现层状态 |
| 所有事件 | — | — | ✅ Event | — | 瞬态 |

**跨层污染检查**：
- [x] 没有 Definition 层数据
- [x] 没有 Spec 层数据
- [x] 没有 Persistence 层数据
- [x] BattleUnitId 作为值类型跨层传递

---

## 5. Dependency Analysis

| 依赖方向 | 依赖项 | 说明 |
|---------|--------|------|
| `PickTarget::Unit` | → `BattleUnitId` | 单位目标包含战场单位 ID |
| `PickTarget::Tile` | → `GridPos` (tactical) | 瓦片目标使用战术域网格坐标 |
| `PickTarget::Skill` | → `SkillId` | 技能目标使用技能 ID |
| `PickTarget::Item` | → `ItemId` | 物品目标使用物品 ID |
| `PickContext::SkillTargeting` | → `SkillId` | 技能瞄准上下文包含技能 ID |
| `UnitClicked` | → `BattleUnitId`, `PickContext` | 事件字段依赖 |
| `UnitSelected` | → `BattleUnitId` | 事件字段依赖 |
| `TileClicked` | → `GridPos`, `PickContext` | 事件字段依赖 |
| `TargetConfirmed` | → `PickTarget`, `PickContext` | 事件字段依赖 |
| `SelectionChanged` | → `PickTarget`, `SelectionPhase` | 事件字段依赖 |

---

## 6. Replay Compatibility

### 6.1 核心原则

**SelectionState 不进 Replay log**。Selection 是表现层状态，不影响游戏逻辑的确定性结果。

### 6.2 事件确定性分析

| 事件 | 需要进 Replay？ | 确定性保证 |
|------|----------------|------------|
| `UnitClicked` | ✅ 是 | 🟩 完全确定（BattleUnitId 确定性分配） |
| `TileClicked` | ✅ 是 | 🟩 完全确定（GridPos 固定坐标） |
| `UnitSelected` | ⛔ 否 | 🟩 可推导 |
| `TargetConfirmed` | ✅ 是 | 🟩 完全确定 |
| `SelectionChanged` | ⛔ 否 | 🟩 纯通知事件 |

### 6.3 BattleUnitId 的 Replay 确定性

```
战斗开始时（确定性流程）：
  1. Replay 种子 → RNG 实例（完全确定）
  2. 按确定顺序遍历参战单位列表（由 DefRegistry::list("unit_*") 提供）
  3. 每个单位：分配 BattleUnitId → 创建 Entity → 附加 UnitIdComponent
  4. 结果：同一 Replay 种子 + 同配置 = 完全一致的 BattleUnitId→Entity 映射

禁止：
  - UUID/v4 生成（非确定）
  - wall-clock 时间戳
  - Entity::from_bits() 作为稳定引用
  - 基于堆地址的 ID 分配
```

---

## 7. Save Compatibility

| 数据 | 持久化位置 | 说明 |
|------|-----------|------|
| `BattleUnitId` | Entity 存档中 | 作为 UnitIdComponent.id 持久化 |
| `PickTarget` | ❌ 不存档 | 瞬态值类型 |
| `PickContext` | ❌ 不存档 | 运行时状态 |
| `SelectionState` | ❌ 不存档 | 表现层状态 |
| 所有事件 | ❌ 不存档 | 瞬态 |

---

## 8. Data Laws 合规矩阵

| # | 规则 | 合规 | 说明 |
|---|------|------|------|
| DL001 | Def/Instance 分离 | ✅ | BattleUnitId 是 ID 值类型，PickTarget/PickContext 是 Transient |
| DL002 | Rule/Content 分离 | ✅ | 所有类型均为代码定义 |
| DL003 | Config IDs Only | ✅ | PickTarget 引用 BattleUnitId/SkillId/ItemId |
| DL009 | 表现通过 Cue | ✅ | SelectionChanged → Projection → ViewModel |
| DL010 | Replay 优先 | ✅ | SelectionState 不进 Replay；领域事件可重现 |
| DL011 | Schema 版本化 | ✅ | 瞬态类型无版本需求 |
| DL012 | 域间无直接引用 | ✅ | Picking 通过 Event 与 Domain 通信 |
