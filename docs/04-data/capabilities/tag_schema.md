---
id: capabilities.tag.schema.v1
title: Tag Schema — 标签数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition
replay-safe: true
---

# Tag Schema — 标签数据架构

> **领域归属**: Capabilities — 核心基石层 | **依赖 Schema**: 无（最底层） | **定义依据**: `docs/02-domain/tag_domain.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `TagDefinition` | Definition | 标签的静态定义（ID、父标签、路径名） |
| `TagHierarchy` | Definition | 层级树结构（全局唯一） |
| `TagSet` | Instance | 实体当前持有的标签集（位掩码） |
| `TagQuery` | Definition | 查询条件定义（Any/All/None 模式） |

---

## 2. Problem

Tag 系统是整个数据宇宙的「通用底层语言」——所有领域（Condition、Trigger、Effect、Combat 等）都依赖标签做分类、查询和条件匹配。Schema 必须解决：
- 标签 ID 的跨文件唯一引用
- 层级树的高效查询（父→子包含关系）
- 位掩码 ID 的全局分配与冲突检测
- Definition 层与 Instance 层的清晰分离

---

## 3. Schema Design

### 3.1 TagDefinition（Definition 层）

```rust
struct TagDefinition {
    /// 标签唯一标识，格式: `tag_<6位数字>`
    id: TagId,

    /// 层级路径名，用于人类可读的引用（如 "DamageType.Elemental.Fire"）
    /// 不可用于程序逻辑，仅用于内容编辑和调试
    path: String,

    /// 父标签 ID。None 表示根标签。
    parent_id: Option<TagId>,

    /// 标签描述本地化 Key（格式: `tag.<id>.desc`）
    desc_key: LocalizationKey,

    /// 分配到的位掩码索引（由 Registry 自动分配）
    /// 范围: 0..MAX_BITS（如 u64 则最多 64 个独立标签位）
    bit_index: u32,

    /// 该标签是否为抽象标签（不可直接授予实体，仅用于层级分组）
    is_abstract: bool,

    /// 元数据：所属分类命名空间
    namespace: TagNamespace,
}
```

### 3.2 TagNamespace（Definition 层）

```rust
/// 标签命名空间枚举，用于强制命名空间一致性（Data Law: 禁止跨域引用）
enum TagNamespace {
    DamageType,
    StatusEffect,
    SkillType,
    EquipmentSlot,
    EquipmentCategory,
    WeaponCategory,
    ArmorCategory,
    ItemCategory,
    Faction,
    CombatState,
    MovementType,
    TerrainType,
    BuffCategory,
    Immune,
    Cooldown,
    SpellSchool,
    QuestTag,
    DialogueTag,
    Custom(String),    // 允许扩展，但必须注册
}
```

### 3.3 TagHierarchy（Definition 层，全局单例）

```rust
struct TagHierarchy {
    /// 所有已注册标签的完整映射
    tags: HashMap<TagId, TagDefinition>,

    /// 子标签索引: parent_id → Vec<child_id>
    children: HashMap<TagId, Vec<TagId>>,

    /// 位掩码继承映射: tag_id → 包含自身及所有子标签的位掩码
    /// 例如 Tag(DamageType.Elemental) 的 bitmask 包含其所有子标签的位
    inherited_masks: HashMap<TagId, BitMask>,

    /// 校验用：所有抽象标签列表
    abstract_tags: HashSet<TagId>,
}
```

### 3.4 TagSet（Instance 层 — ECS Component）

```rust
struct TagSet {
    /// 实体当前持有的所有标签的位掩码
    /// 位操作: O(1) 包含检查
    bits: BitMask,

    /// 缓存：该实体的所有标签 ID 列表（由 bits 推导，用于枚举场景）
    /// 惰性计算，bits 变化时失效
    cached_tags: Vec<TagId>,
}

/// 位掩码类型（平台相关，当前使用 u128 支持最多 128 个独立标签位）
/// 可根据项目扩展需求升级为自定义大整数类型
type BitMask = u128;
```

### 3.5 TagQuery（Definition 层）

```rust
struct TagQuery {
    /// 匹配模式
    mode: TagQueryMode,

    /// 目标标签列表（引用 TagId）
    target_tags: Vec<TagId>,

    /// 是否考虑层级继承（父标签匹配子标签）
    /// true: 查询 DamageType.Elemental 同时匹配 Fire/Cold/Lightning/Acid
    /// false: 仅精确匹配
    respect_hierarchy: bool,
}

enum TagQueryMode {
    /// 至少匹配一个目标标签
    Any,
    /// 匹配全部目标标签
    All,
    /// 不匹配任何目标标签（用于免疫/排除检查）
    None,
}
```

### 3.6 TagRegistrationRequest（Definition 层，配置格式）

```yaml
# RON 配置示例 — 标签注册表
TagRegistrationRequest:
  tags:
    - id: "tag_000001"
      path: "DamageType"
      parent_id: ~
      namespace: DamageType
      is_abstract: true

    - id: "tag_000002"
      path: "DamageType.Physical"
      parent_id: "tag_000001"
      namespace: DamageType
      is_abstract: true

    - id: "tag_000003"
      path: "DamageType.Physical.Slashing"
      parent_id: "tag_000002"
      namespace: DamageType
      is_abstract: false

    - id: "tag_000004"
      path: "DamageType.Elemental"
      parent_id: "tag_000001"
      namespace: DamageType
      is_abstract: true

    - id: "tag_000005"
      path: "DamageType.Elemental.Fire"
      parent_id: "tag_000004"
      namespace: DamageType
      is_abstract: false

    - id: "tag_000010"
      path: "StatusEffect"
      parent_id: ~
      namespace: StatusEffect
      is_abstract: true

    - id: "tag_000011"
      path: "StatusEffect.Poisoned"
      parent_id: "tag_000010"
      namespace: StatusEffect
      is_abstract: false
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 可热重载 | 备注 |
|----------|-------|--------|----------|------|
| `TagDefinition` | Definition | 是（配置文件） | 是 | 加载时构建 Hierarchy |
| `TagHierarchy` | Definition | 否（由 Definition 推导） | 是 | 加载时全量重建 |
| `TagSet` | Instance | 是（存档时持久化） | 否 | ECS Component |
| `TagQuery` | Definition | 是（引用 TagId） | 是 | 嵌入在 AbilityDef 等配置中 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 被依赖 | — | Tag 是最底层 Schema，无前置依赖 |
| 被依赖 | → Condition | TagQuery 被 Condition 系统消费 |
| 被依赖 | → Trigger | 标签变更（TagAdded/Removed）作为触发条件 |
| 被依赖 | → Effect | 效果携带 TagRequirements |
| 被依赖 | → Combat/Spell 等 | 所有 Domain 引用 Tag 做分类和过滤 |

---

## 6. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | TagId 全局唯一 | 配置加载 | 重复 TagId 拒绝注册 |
| V2 | 无循环层级 | 配置加载 | DFS 检测父→子链是否形成循环 |
| V3 | 父标签必须已注册 | 配置加载 | parent_id 必须在 tags 映射中 |
| V4 | 命名空间一致 | 配置加载 | 父子标签必须同 namespace |
| V5 | 叶节点不可抽象 | 配置加载 | 无子标签的标签 is_abstract 必须为 false |
| V6 | 位索引不重复 | Registry 分配 | 每个 TagId 分配到唯一的 bit_index |
| V7 | TagQuery 引用存在 | AbilityDef 加载 | query.target_tags 中所有 TagId 已注册 |

---

## 7. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| 标签授予/移除 | 🟩 完全确定 | 通过 Command 录制（GrantTag/RemoveTag），RNG 不参与 |
| 标签查询 | 🟩 完全确定 | TagQuery 是纯函数：输入 TagSet + Query → 布尔结果 |
| 位掩码计算 | 🟩 完全确定 | 位运算是确定性的 |
| 层级变化 | 🟩 加载期确定 | 层级树在加载时构建，运行时不变 |

**结论**: Tag Schema 是天然 Replay-safe 的。所有标签操作（授予/移除/查询）均不涉及随机性。

---

## 8. Save Compatibility

| 场景 | 兼容性 | 版本策略 |
|------|--------|----------|
| 实体 TagSet | 🟩 存档保存位掩码 | 1.0: u128 位掩码 |
| 新增标签类型 | 🟩 前向兼容 | 新标签分配新 bit_index，旧位掩码不受影响 |
| 标签重命名 | 🟩 无损 | 位掩码不变，只有 path/desc 变化 |
| 标签删除 | 🟨 软删除 | deprecated 标记保留 bit_index，不重新分配 |

**策略**: TagSet 只存位掩码，不存标签 ID 列表。位掩码的语义由运行时 TagHierarchy 解释，因此存档与标签定义是松散耦合的。

未来若 u128 位耗尽：
1. 升级为 `BitSet256` 自定义类型
2. Save Migration v1→v2 升级位掩码字段宽度

---

## 9. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2（未来） | 位掩码扩展至 256 位 | Save Migration: 零填充扩展，运行时检测位宽 |
| v3（未来） | 标签引入分类版本化 | 增加 TagDefinition.since_version 字段 |

---

## 10. Future Extension

- **标签元数据扩展**：为标签附加自定义元数据（如颜色、图标 Key、排序权重）而不改变核心位掩码逻辑
- **标签组（TagGroup）**：预定义的标签集合，作为配置快捷方式（如 "所有元素伤害" = DamageType.Elemental 及其子树）
- **动态标签**：在严格约束下允许实例级的动态标签（用于临时剧情标记），但不参与层级继承

---

## 11. Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| 位掩码位数耗尽 | 大项目可能超过 128 个独立标签 | 设计时就预留升级路径到 256 位 |
| 层级树加载顺序依赖 | 配置加载时标签注册顺序影响构建 | 采用两阶段加载：先注册所有节点，再构建层级关系 |
| 位掩码与存档版本耦合 | 标签定义变化导致旧存档语义不同 | Save Schema 只存位掩码，语义由当前 TagHierarchy 实时解释 |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| 三层分离（Def→Spec→Instance） | ✅ | TagDefinition (Def) → TagQuery (Spec) → TagSet (Instance) |
| Data Driven First | ✅ | 标签定义全部通过配置文件注册 |
| Replay First | ✅ | 所有标签操作确定性 |
| Logic/Presentation Separation | ✅ | Tag 不做表现，仅通过 Event 通知 |
| 宪法 §16.2 使用位掩码 | ✅ | 与宪法推荐的位掩码实现一致 |
