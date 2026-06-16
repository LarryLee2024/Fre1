---
id: domains.party.schema.v1
title: Party Schema — 队伍数据架构
status: draft
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: instance, persistence
replay-safe: false
---

# Party Schema — 队伍数据架构

> **领域归属**: Domains — 成长养成层 | **依赖 Schema**: Modifier, Event, Combat | **定义依据**: `docs/02-domain/party_domain.md`

---

## 1. Schema Design

### 1.1 PartyRoster（Instance 层/Persistence 层）

```rust
/// 队伍成员名册。管理所有可加入队伍的角色。
struct PartyRoster {
    /// 当前活跃（上场战斗）成员
    active_members: Vec<EntityId>,

    /// 预备队员（不在战斗中）
    reserve_members: Vec<EntityId>,

    /// 活跃成员上限（默认 4）
    max_active: u32,

    /// 队伍总人数上限（活跃 + 预备，默认 12）
    max_total: u32,
}
```

### 1.2 BondDef（Definition 层）

```rust
/// 羁绊模板定义。描述特定角色组合的羁绊条件和效果。
struct BondDef {
    /// 羁绊唯一标识（前缀: `bnd_`）
    id: BondDefId,

    /// 羁绊名称本地化 Key
    name_key: LocalizationKey,

    /// 羁绊描述本地化 Key
    desc_key: LocalizationKey,

    /// 激活条件：需要哪些角色/标签同时在活跃队伍中
    required_members: Vec<BondRequirement>,

    /// 各等级的效果（ModifierDefId 列表）
    level_effects: HashMap<u32, Vec<ModifierDefId>>,

    /// 最大羁绊等级
    max_level: u32,
}

struct BondRequirement {
    /// 具体角色 EntityId（特定 NPC 绑定），或
    specific_entity: Option<EntityId>,
    /// 标签条件（如 `tag_race_elf`）
    required_tags: Vec<TagDefId>,
    /// 同时满足（And）/ 任一满足（Or）
    match_mode: BondMatchMode,
}

enum BondMatchMode { All, Any }
```

### 1.3 BondState（Instance 层/Persistence 层）

```rust
/// 运行时羁绊状态。
struct BondState {
    /// 当前激活的羁绊
    active_bonds: Vec<ActiveBond>,
}

struct ActiveBond {
    /// 羁绊模板 ID
    bond_id: BondDefId,

    /// 当前等级
    level: u32,

    /// 参与的角色
    participants: Vec<EntityId>,

    /// 积累的战斗次数（用于升级）
    accumulated_battles: u32,
}
```

### 1.4 FormationDef（Definition 层）

```rust
/// 阵型模板定义。
struct FormationDef {
    /// 阵型唯一标识
    id: FormationDefId,

    /// 阵型名称本地化 Key
    name_key: LocalizationKey,

    /// 各位置的站位偏移
    positions: Vec<GridPosition>,

    /// 阵型提供的 Modifier（如特定位置 +AC）
    modifiers: Vec<FormationModifier>,
}

struct FormationModifier {
    slot_index: u32,
    modifier_id: ModifierDefId,
}
```

### 1.5 PartyState（Persistence 层）

```rust
/// 队伍系统的持久化状态。
struct PartyState {
    /// 队伍成员名册
    roster: PartyRoster,

    /// 羁绊状态
    bonds: BondState,

    /// 当前使用的阵型
    active_formation: Option<FormationDefId>,
}
```

---

## 2. Layer Summary

| Layer | Structures | 说明 |
|-------|-----------|------|
| **Definition** | `BondDef`, `FormationDef` | 羁绊和阵型的静态模板 |
| **Spec** | — | Party 无 Spec 层 |
| **Instance** | `PartyRoster`, `BondState` | 队伍运行时状态 |
| **Persistence** | `PartyState` | 队伍配置、羁绊状态持久化 |

---

## 3. Dependency Analysis

| 依赖 | 说明 |
|------|------|
| → ModifierSchema | 羁绊和阵型效果引用 ModifierDefId |
| → EventSchema | 队伍事件发布（MemberJoined, BondActivated 等） |
| → CombatSchema | 换人操作更新战斗参与者；阵型影响站位 |
| ← CampRestSchema | 长休时可在营地调整队伍配置 |

---

## 4. Replay & Save

### Replay

- 标记 `replay-safe: false` — 队伍配置是进程数据，不参与战斗回放

### Save

- `PartyState` 完整持久化（成员名册、羁绊状态、阵型配置）
- BondDef/FormationDef 从 Definition 层加载
- 换人/羁绊变更记录为存档增量

---

## 5. Validation Rules

| 规则 | 说明 | 违反处理 |
|------|------|----------|
| 战斗人数上限 | active_members.len() <= max_active (4) | 战斗开始前检查 |
| 总人数上限 | roster 总人数 <= max_total (12) | 新成员加入时拒绝 |
| 羁绊条件实时评估 | 队伍变化时重新检查所有羁绊 | 运行时强制重评估 |
| 羁绊唯一性 | 同一角色不可同时参与多个同类型羁绊 | 激活时检查 |
| 角色状态互斥 | 同一角色不可同时在 active 和 reserve | 数据一致性断言 |

---

## 6. Constitution Check

- ✅ **Data Law 001 (Def-Instance分离)**: BondDef/FormationDef 为 Definition，PartyRoster/BondState 为 Instance
- ✅ **Data Law 003 (配置只引用ID)**: BondDef 引用 TagDefId/ModifierDefId，BondState 引用 BondDefId
- ✅ **Data Law 006 (Modifier不拥有业务逻辑)**: 羁绊/阵型的加成效果通过 Modifier 表达，不含逻辑
- ✅ **Data Law 011 (Schema版本化)**: PartyState 携带版本号
- ✅ **Data Law 012 (域间禁止直接数据引用)**: Party 通过 Event 与 CampRest/Combat 通信
