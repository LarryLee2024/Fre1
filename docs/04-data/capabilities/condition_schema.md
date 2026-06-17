---
id: capabilities.condition.schema.v1
title: Condition Schema — 条件/限制数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition
replay-safe: true
---

# Condition Schema — 条件/限制数据架构

> **领域归属**: Capabilities — 逻辑骨架层 | **依赖 Schema**: Tag, Attribute | **定义依据**: `docs/02-domain/capabilities/condition_domain.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `Condition` | Definition | 统一条件定义（TagRequirement / AttributeCheck / ResourceCheck / Custom） |
| `ConditionGroup` | Definition | 条件组合（AND/OR/NOT），支持任意嵌套 |
| `ConditionResult` | Runtime | 条件评估结果（Passed / Failed） |

---

## 2. Problem

Condition 是贯穿全系统的「统一条件检查语言」——技能激活前检查、效果应用前检查、装备穿戴前检查、免疫检查、对话分支过滤……所有"判断是否允许"的逻辑都使用 Condition。Schema 必须解决：
- 条件组合的任意嵌套（AND/OR/NOT）
- 统一的 Pass/Fail 结果格式（含失败原因）
- TagRequirement、AttributeCheck、ResourceCheck 三种内置类型的数据表达
- 免疫条件的最高优先级处理
- CustomCondition 的扩展点

---

## 3. Schema Design

### 3.1 Condition（Definition 层）

```rust
enum Condition {
    /// 标签需求检查——实体是否拥有/不拥有特定标签
    TagRequirement(TagRequirement),

    /// 属性阈值检查——属性值 >= 阈值
    AttributeCheck(AttributeCheck),

    /// 资源充足检查——资源量 >= 消耗量
    ResourceCheck(ResourceCheck),

    /// 条件组合（AND / OR / NOT）
    Group(ConditionGroup),

    /// 自定义条件（由 Domain 注册）
    Custom(CustomConditionDef),
}
```

### 3.2 TagRequirement（Definition 层）

```rust
struct TagRequirement {
    /// 匹配模式
    mode: TagRequirementMode,

    /// 目标标签
    target_tags: Vec<TagId>,

    /// 是否考虑层级继承
    respect_hierarchy: bool,
}

enum TagRequirementMode {
    /// 实体必须拥有所有指定标签
    HasAll,
    /// 实体必须拥有至少一个指定标签
    HasAny,
    /// 实体不得拥有任何指定标签（免疫/排除）
    HasNone,
}
```

### 3.3 AttributeCheck（Definition 层）

```rust
struct AttributeCheck {
    /// 目标属性
    attribute_id: AttributeId,

    /// 比较运算符
    operator: ComparisonOp,

    /// 比较值（固定值）
    threshold: f32,

    /// 比较值来源（可选，来自另一个属性的当前值）
    /// 如果指定，threshold 被忽略，使用 source_attribute 的 current_value
    source_attribute: Option<AttributeId>,
}

enum ComparisonOp {
    GreaterThan,
    GreaterOrEqual,
    LessThan,
    LessOrEqual,
    Equal,
    NotEqual,
}
```

### 3.4 ResourceCheck（Definition 层）

```rust
struct ResourceCheck {
    /// 要检查的资源属性
    resource_attribute: AttributeId,

    /// 需要消耗的量
    required_amount: f32,

    /// 消费后是否允许 resource == 0（true = 允许耗尽，false = 必须剩余 > 0）
    allow_deplete: bool,
}
```

### 3.5 ConditionGroup（Definition 层）

```rust
struct ConditionGroup {
    /// 组合逻辑
    logic: GroupLogic,

    /// 子条件列表（递归结构，支持任意嵌套）
    conditions: Vec<Condition>,
}

enum GroupLogic {
    /// 所有子条件必须通过（AND）
    All,
    /// 至少一个子条件通过（OR）
    Any,
    /// 子条件不能通过（NOT——只能包含一个子条件）
    Not,
}
```

### 3.6 CustomConditionDef（Definition 层）

```rust
struct CustomConditionDef {
    /// 自定义条件类型标识（如 "combat.is_adjacent", "spell.has_slot"）
    condition_type: String,

    /// 自定义参数（JSON-like 结构）
    parameters: HashMap<String, ConditionParam>,
}

enum ConditionParam {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    EntityId(EntityId),
    TagId(TagId),
    AttributeId(AttributeId),
}
```

### 3.7 ConditionResult（Runtime 层）

```rust
struct ConditionResult {
    /// 是否通过
    passed: bool,

    /// 失败原因（仅 passed == false 时有效）
    failure_reason: Option<ConditionFailure>,

    /// 评估路径（调试用）
    eval_path: Option<Vec<EvalStep>>,
}

struct ConditionFailure {
    /// 失败的条件描述
    failed_condition: String,

    /// 失败类型
    failure_type: FailureType,

    /// 失败详情
    detail: String,
}

enum FailureType {
    TagMissing,
    TagPresent,
    AttributeTooLow,
    AttributeTooHigh,
    ResourceInsufficient,
    Custom(String),
}

struct EvalStep {
    /// 条件描述
    condition_desc: String,
    /// 评估结果
    result: bool,
    /// 子步骤（用于嵌套条件组合）
    children: Vec<EvalStep>,
}
```

### 3.8 ConditionConfig（Definition 层 — 配置格式）

```yaml
# RON 配置示例 — Condition 表达式（嵌入在 AbilityDef/EffectDef 中使用）
Condition:
  Group:
    logic: All
    conditions:
      # 条件1: 目标不能有火焰免疫标签
      - TagRequirement:
          mode: HasNone
          target_tags: ["tag_000030"]   # Tag.Immune.Fire
          respect_hierarchy: true

      # 条件2: 施法者智力 >= 16
      - AttributeCheck:
          attribute_id: "attr_000005"   # 智力
          operator: GreaterOrEqual
          threshold: 16.0

      # 条件3: 施法者法力 >= 20
      - ResourceCheck:
          resource_attribute: "attr_000031"  # 法力值
          required_amount: 20.0
          allow_deplete: false

      # 条件4 (OR): 目标在近战范围 OR 施法者在施法范围
      - Group:
          logic: Any
          conditions:
            - Custom:
                condition_type: "tactical.is_adjacent"
                parameters:
                  source: EntityId("$CASTER")
                  target: EntityId("$TARGET")
                  range: Integer(1)
            - Custom:
                condition_type: "spell.in_range"
                parameters:
                  spell_id: String("abl_000001")
                  source: EntityId("$CASTER")
                  target: EntityId("$TARGET")
```

### 3.9 ConditionSnapshot（Persistence 层）

```rust
/// Condition 本身是纯 Definition 数据，不直接持久化。
/// 需要持久化的只是 Condition 评估的上下文依赖状态（如免疫标签）。
struct ConditionSnapshot {
    entity_id: EntityId,
    immunity_tags: Vec<TagId>,
    special_conditions: Vec<(String, bool)>, // (condition_type, is_active)
}
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 可热重载 | 备注 |
|----------|-------|--------|----------|------|
| `Condition` | Definition | 是（Def 内嵌） | 是 | 嵌入在 AbilityDef/EffectDef/ItemDef 中 |
| `TagRequirement` | Definition | 是（Def 内嵌） | 是 | 归属 Tag 领域 |
| `AttributeCheck` | Definition | 是（Def 内嵌） | 是 | 归属 Attribute 领域 |
| `ConditionGroup` | Definition | 是（Def 内嵌） | 是 | 组合逻辑 |
| `ConditionResult` | Runtime | 否 | 否 | 瞬时评估结果 |
| `CustomConditionDef` | Definition | 是（Def 内嵌） | 是 | 扩展点 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → TagSchema | TagRequirement.target_tags 引用 TagId |
| 依赖 | → AttributeSchema | AttributeCheck 引用 AttributeId |
| 被依赖 | ← AbilitySchema | 技能激活前检查激活条件 |
| 被依赖 | ← EffectSchema | 效果应用前检查应用条件 |
| 被依赖 | ← TriggerSchema | TriggerCondition 引用 Condition |
| 被依赖 | ← InventorySchema | 装备穿戴前检查属性需求 |

---

## 6. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | TagId 存在 | Condition 加载 | TagRequirement.target_tags 中的所有 TagId 已注册 |
| V2 | AttributeId 存在 | Condition 加载 | AttributeCheck.attribute_id 已注册 |
| V3 | NOT 只能有一个子条件 | Condition 加载 | GroupLogic::Not 的 conditions.len() == 1 |
| V4 | 评估无副作用 | 运行时 | 断言：评估过程不修改任何 ECS 数据 |
| V5 | 免疫最高优先级 | 运行时 | TagRequirement(HasNone, Tag.Immune.X) 在 All 组中即使其他条件通过也否决 |

---

## 7. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| Condition 评估 | 🟩 完全确定 | 纯函数：输入实体状态 → 输出 Pass/Fail |
| 组合条件 | 🟩 完全确定 | AND/OR/NOT 逻辑确定 |
| 免疫检查 | 🟩 完全确定 | 标签状态确定 → 免疫结果确定 |

**结论**: Condition Schema 是天然 Replay-safe 的。条件评估是确定性纯函数。

---

## 8. Save Compatibility

Condition 定义（Definition 层）嵌入在其他 Def 结构中，随 Def 一起版本管理。运行时 Condition 不单独持久化。仅实体上的免疫状态等信息作为 EntityState 的一部分存于存档。

---

## 9. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2（未来） | 新增内置 ConditionType | 新增 enum variant，旧配置不受影响 |

---

## 10. Future Extension

- **条件缓存**: 高频评估的 Condition（如每帧检查的被动条件）缓存结果，依赖状态变化时失效
- **条件复合键**: 为常用条件组合定义复合键（如「战斗施法许可」= 非沉默 + 有法术位 + 非禁锢）
- **条件钩子**: 允许 Domain 在 Condition 评估前后注入自定义逻辑

---

## 11. Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| 条件树过深 | 复杂嵌套导致递归深度超限 | 设条件树最大深度（默认 10 层） |
| 自定义条件注册混乱 | Domain 各自注册导致管理失控 | 所有 CustomConditionType 必须通过 ConditionRegistry 注册 |
| 条件评估性能 | 每帧评估大量复杂条件 | 条件缓存 + 惰性重算（依赖状态变化时标记失效） |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| Condition 无副作用 | ✅ | 评估不修改状态 |
| 免疫统一处理 | ✅ | 统一 TagRequirement(HasNone) 模式 |
| Replay First | ✅ | 确定性纯函数 |
