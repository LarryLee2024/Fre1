---
id: infrastructure.registry.schema.v1
title: Registry Schema — 注册中心数据架构
status: draft
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition
replay-safe: true
---

# Registry Schema — 注册中心数据架构

> **领域归属**: Infrastructure | **依赖 Schema**: 全部 Schema | **定义依据**: `docs/00-governance/Fre项目架构设计.md` §6.5 C3 Runtime

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `DefRegistry` | Definition | 所有 Definition 的全局注册中心 |
| `IdAllocator` | Definition | ID 分配器（类型前缀 + 数字编号） |
| `RegistryValidation` | Definition | 注册时的一致性校验 |

---

## 2. Schema Design

### 2.1 DefRegistry（Definition 层）

```rust
/// 全局 Definition 注册中心。
/// 所有 Def 在内容加载时通过 Registry 注册，运行时只读。
struct DefRegistry {
    /// 所有 AbilityDef
    abilities: HashMap<AbilityDefId, AbilityDef>,
    /// 所有 EffectDef
    effects: HashMap<EffectDefId, EffectDef>,
    /// 所有 TriggerDef
    triggers: HashMap<TriggerDefId, TriggerDef>,
    /// 所有 TagDefinition
    tags: HashMap<TagId, TagDefinition>,
    /// 所有 AttributeDefinition
    attributes: HashMap<AttributeId, AttributeDefinition>,
    /// 所有 CueDef
    cues: HashMap<CueDefId, CueDef>,
    /// 所有 ItemDef
    items: HashMap<ItemId, ItemDef>,
    /// 所有 SpellDef
    spells: HashMap<SpellDefId, SpellDef>,
    /// 所有 QuestDef
    quests: HashMap<QuestDefId, QuestDef>,
    /// 所有 FactionDef
    factions: HashMap<FactionId, FactionDef>,
    /// 所有 TerrainDef
    terrains: HashMap<TerrainDefId, TerrainDef>,
    /// 所有 RecipeDef
    recipes: HashMap<RecipeId, RecipeDef>,

    /// 自定义 Def 扩展（Domain 可注册自定义 Def 类型）
    custom: HashMap<String, Box<dyn Any>>,
}
```

### 2.2 RegistryEntry（Definition 层）

```rust
/// 每个注册项的元数据。
struct RegistryEntry {
    /// Def ID
    def_id: String,
    /// Def 类型
    def_type: String,
    /// 注册帧号/版本
    registered_at_version: String,
    /// 是否为 Deprecated
    deprecated: bool,
    /// 替换者 ID（如果被取代）
    superseded_by: Option<String>,
}
```

### 2.3 IdAllocator（Definition 层）

```rust
/// ID 分配器：管理各类型前缀的数字编号分配。
struct IdAllocator {
    allocators: HashMap<IdType, AllocatorState>,
}

struct AllocatorState {
    /// 类型前缀
    prefix: &'static str,
    /// 当前最大已分配编号
    next_id: u64,
    /// 已释放/回收的 ID（可选）
    recycled: Vec<u64>,
    /// ID 格式（总位数，0-padded）
    digit_count: u8,
}

enum IdType {
    Ability,
    Effect,
    Trigger,
    Tag,
    Attribute,
    Cue,
    Item,
    Spell,
    Quest,
    Faction,
    Terrain,
    Recipe,
    Buff,
    LootTable,
    Custom(String),
}
```

### 2.4 RegistryValidation（Definition 层）

```rust
/// 注册时的一致性校验结果。
struct RegistryValidation {
    /// 是否有错误
    has_errors: bool,
    /// 错误列表
    errors: Vec<ValidationError>,
    /// 警告列表
    warnings: Vec<ValidationWarning>,
    /// 跨 Def 引用检查（所有引用的 ID 都存在）
    cross_references: CrossReferenceReport,
}

struct CrossReferenceReport {
    total_defs: u32,
    total_references: u32,
    broken_references: Vec<BrokenReference>,
}

struct BrokenReference {
    /// 来源 Def
    source_def: String,
    /// 引用字段
    field: String,
    /// 引用的 ID
    referenced_id: String,
    /// 期望的类型
    expected_type: String,
}
```

### 2.5 RegistryConfig（Definition 层 — 示例）

```yaml
# RON 配置 — Registry 配置
RegistryConfig:
  id_allocators:
    ability:
      prefix: "abl_"
      next_id: 1
      digit_count: 6
    effect:
      prefix: "eff_"
      next_id: 1
      digit_count: 6
    tag:
      prefix: "tag_"
      next_id: 1
      digit_count: 6
    attribute:
      prefix: "attr_"
      next_id: 1
      digit_count: 6
    cue:
      prefix: "cue_"
      next_id: 1
      digit_count: 6
    item:
      prefix: "itm_"
      next_id: 1
      digit_count: 6

  validation:
    check_cross_references: true
    strict_mode: false  # true = 引用断裂阻止加载
```

---

## 3. Layer Analysis

Registry 是纯 Definition 层基础设施——所有注册数据在内容加载时构建，运行时只读。不参与 Instance/Persistence 层。

---

## 4. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → 全部 Schema | 持有所有 Def 类型的引用 |
| 被依赖 | ← 全部 Schema | 所有系统在加载/运行时查询 Registry |

---

## 5. Validation Rules

| # | 规则 | 触发时机 |
|---|------|----------|
| V1 | ID 格式正确（前缀 + 6位数字） | Def 注册 |
| V2 | ID 全局唯一（跨类型也唯一） | Def 注册 |
| V3 | 所有跨 Def 引用有效 | 加载完成时全量校验 |
| V4 | Deprecated 的 Def 不再被引用 | 加载完成时全量校验 |
| V5 | 循环依赖检测 | 加载完成时全量校验 |

---

## 6. Replay / Save Compatibility

Registry 是内容加载阶段的基础设施，不参与运行时回放。存档不包含 Registry 数据（存档只存 Instance 和 Persistence 层数据）。

---

## 7. Constitution Check

| 条款 | 合规 | 说明 |
|------|------|------|
| Data Driven First | ✅ | Registry 由配置驱动，代码不硬编码 Def |
| Single Source of Truth | ✅ | 所有 Def 通过 Registry 查询，禁止重复定义 |
