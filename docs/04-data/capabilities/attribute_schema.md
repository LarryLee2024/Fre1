---
id: capabilities.attribute.schema.v1
title: Attribute Schema — 属性数据架构
status: draft
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition, instance
replay-safe: true
---

# Attribute Schema — 属性数据架构

> **领域归属**: Capabilities — 核心基石层 | **依赖 Schema**: Tag | **定义依据**: `docs/02-domain/attribute_domain.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `AttributeDefinition` | Definition | 属性的静态定义（ID、分类、默认值、边界） |
| `AttributeValue` | Instance | 属性的运行时数值（base + current） |
| `AttributeContainer` | Instance | 实体上的属性容器（ECS Component） |
| `AttributeSnapshot` | Persistence | 属性的存档快照 |
| `DerivedFormula` | Definition | 派生属性的计算公式定义 |

---

## 2. Problem

Attribute 是角色的「数值骨架」——所有战斗计算（伤害、治疗、资源消耗）、成长系统（升级属性增长）、装备效果（属性加值）都以 Attribute 为基础。Schema 必须解决：
- 基础值与当前值的分离（BaseValue 运行时不可变，CurrentValue 可被 Modifier 修改）
- 属性分类（Primary/Secondary/Derived/Resource）的数据表达
- 派生属性计算公式的管理（Derived 属性的计算依赖哪些其他属性）
- 属性快照的存档兼容

---

## 3. Schema Design

### 3.1 AttributeDefinition（Definition 层）

```rust
struct AttributeDefinition {
    /// 属性唯一标识，格式: `attr_<6位数字>`
    id: AttributeId,

    /// 属性分类
    category: AttributeCategory,

    /// 显示名称本地化 Key（格式: `attribute.<id>.name`）
    name_key: LocalizationKey,

    /// 描述本地化 Key（格式: `attribute.<id>.desc`）
    desc_key: LocalizationKey,

    /// 基础默认值
    default_base_value: f32,

    /// 当前值最小值（含 clamp 下界）
    min_value: f32,

    /// 当前值最大值（含 clamp 上界）
    max_value: f32,

    /// 如果 category = Derived，此属性引用的其他属性列表
    /// 用于自动追踪依赖属性的变更并触发重算
    derived_dependencies: Vec<AttributeId>,

    /// 是否在 UI 中隐藏（如内部计算用属性）
    hidden: bool,
}

enum AttributeCategory {
    /// 主属性：力量、敏捷、体质、智力、感知、魅力
    Primary,
    /// 副属性：熟练加值、先攻调整值
    Secondary,
    /// 派生属性：生命值上限、防御等级、负重上限
    Derived,
    /// 资源属性：当前生命值、法力值、行动力
    Resource,
}
```

### 3.2 AttributeValue（Instance 层 — ECS Component 字段）

```rust
struct AttributeValue {
    /// 属性引用的 Definition ID
    def_id: AttributeId,

    /// 基础值（运行时只读，仅在升级/永久装备时修改）
    base_value: f32,

    /// 当前值（通过 Aggregator 管线从 base_value + 所有 Modifier 计算得出）
    current_value: f32,

    /// 当前值是否由 Aggregator 管线维护
    /// false = 手动管理（Resource 类属性的消费场景）
    aggregator_managed: bool,
}
```

### 3.3 AttributeContainer（Instance 层 — ECS Component）

```rust
struct AttributeContainer {
    /// 实体拥有全部属性的映射
    attributes: HashMap<AttributeId, AttributeValue>,

    /// 派生属性缓存：derived_attr_id → 当前计算值
    /// 用于减少重复计算
    derived_cache: HashMap<AttributeId, f32>,
}
```

### 3.4 DerivedFormula（Definition 层）

派生属性的计算公式，以纯数据形式描述（不包含可执行代码）。

```rust
struct DerivedFormula {
    /// 目标属性 ID
    target_attr_id: AttributeId,

    /// 公式类型
    formula_type: FormulaType,

    /// 公式参数（根据 formula_type 不同含义不同）
    parameters: FormulaParameters,
}

enum FormulaType {
    /// 固定值: parameters = { constant: f32 }
    Constant,
    /// 属性求和: parameters = { source_ids: Vec<AttributeId>, multiplier: f32 }
    Sum,
    /// 属性取最大值: parameters = { source_ids: Vec<AttributeId> }
    Max,
    /// 属性取最小值: parameters = { source_ids: Vec<AttributeId> }
    Min,
    /// 加权和: parameters = { sources: Vec<(AttributeId, weight_f32)>, base: f32 }
    WeightedSum,
    /// 自定义（通过代码注册的公式计算器）
    Custom { formula_id: String },
}

/// 公式参数联合
struct FormulaParameters {
    constant: Option<f32>,
    source_ids: Option<Vec<AttributeId>>,
    multiplier: Option<f32>,
    weights: Option<Vec<(AttributeId, f32)>>,
    base: Option<f32>,
    formula_id: Option<String>,
}
```

### 3.5 AttributeInitializationConfig（Definition 层 — 配置格式）

```yaml
# RON 配置示例 — 属性定义
AttributeInitializationConfig:
  attributes:
    # Primary 属性
    - id: "attr_000001"
      category: Primary
      name_key: "attribute.attr_000001.name"
      desc_key: "attribute.attr_000001.desc"
      default_base_value: 10.0
      min_value: 1.0
      max_value: 30.0
      hidden: false

    - id: "attr_000007"
      category: Secondary
      name_key: "attribute.attr_000007.name"
      desc_key: "attribute.attr_000007.desc"
      default_base_value: 2.0
      min_value: 0.0
      max_value: 12.0
      derived_dependencies: ["attr_000001", "attr_000003"]
      hidden: false

    # Derived 属性
    - id: "attr_000020"
      category: Derived
      name_key: "attribute.attr_000020.name"
      desc_key: "attribute.attr_000020.desc"
      default_base_value: 10.0
      min_value: 0.0
      max_value: 999.0
      derived_dependencies: ["attr_000003"]
      hidden: false

    # Resource 属性
    - id: "attr_000030"
      category: Resource
      name_key: "attribute.attr_000030.name"
      desc_key: "attribute.attr_000030.desc"
      default_base_value: 100.0
      min_value: 0.0
      max_value: 100.0
      hidden: false

  derived_formulas:
    - target_attr_id: "attr_000007"
      formula_type: Sum
      parameters:
        source_ids: ["attr_000001", "attr_000003"]
        multiplier: 1.0

    - target_attr_id: "attr_000020"
      formula_type: WeightedSum
      parameters:
        base: 10.0
        sources:
          - ["attr_000003", 2.0]    # 体质 × 2
```

### 3.6 AttributeSnapshot（Persistence 层）

```rust
struct AttributeSnapshot {
    /// 存档版本
    schema_version: u32,

    /// 实体 ID
    entity_id: EntityId,

    /// 所有属性的快照 (attr_id → (base_value, current_value))
    attributes: HashMap<AttributeId, (f32, f32)>,

    /// 派生属性缓存（可选，用于加速读档）
    derived_cache: Option<HashMap<AttributeId, f32>>,
}
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 可热重载 | 备注 |
|----------|-------|--------|----------|------|
| `AttributeDefinition` | Definition | 是（配置文件） | 是 | 属性类型定义 |
| `DerivedFormula` | Definition | 是（配置文件） | 是 | 派生属性公式 |
| `AttributeValue` | Instance | 否（通过 Container 间接） | 否 | ECS Component 字段 |
| `AttributeContainer` | Instance | 否（通过 Snapshot 持久化） | 否 | ECS Component |
| `AttributeSnapshot` | Persistence | 是（存档） | 否 | 存档序列化格式 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → TagSchema | 属性可能关联 Tag（如 `Tag.Immune.Poison` 相关属性） |
| 被依赖 | ← ModifierSchema | 修改器引用 attribute_id 作为目标 |
| 被依赖 | ← AggregatorSchema | 聚合器以 BaseValue 为输入，产出 FinalValue |
| 被依赖 | ← ExecutionSchema | 伤害/治疗计算读取属性值 |
| 被依赖 | ← ProgressionSchema | 升级时修改 BaseValue |

---

## 6. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | AttributeId 全局唯一 | 配置加载 | 重复 ID 拒绝 |
| V2 | 默认 base 值在 [min, max] 范围 | 配置加载 | `min ≤ default_base_value ≤ max` |
| V3 | Derived 公式引用的属性必须已注册 | 配置加载 | derived_dependencies 中所有 ID 存在 |
| V4 | Derived 公式无循环引用 | 配置加载 | DFS 检测 A→B→A 循环 |
| V5 | Resource 属性 min ≥ 0 | 配置加载 | Resource 类属性不可为负最小值 |
| V6 | Primary 属性 aggregator_managed = true | Entity 创建 | Primary 不可被手动修改 |
| V7 | 快照版本号匹配 | 读档 | schema_version 必须 ≤ 当前版本 |

---

## 7. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| 属性初始化 | 🟩 确定 | BaseValue 由 Definition 决定 + 角色模板覆盖 |
| 属性变更（Modifier） | 🟩 确定 | 通过 Aggregator 管线运算，纯确定性 |
| 资源消费 | 🟩 确定 | 消费量作为 Effect 参数录制 |
| 派生属性重算 | 🟩 确定 | Formula 是纯函数，输入相同则输出相同 |

**结论**: Attribute Schema 是 Replay-safe 的，前提是所有 AttributeValue.current_value 变更必须经过 Aggregator 管线（禁止绕过）。

---

## 8. Save Compatibility

| 场景 | 兼容性 | 版本策略 |
|------|--------|----------|
| 基础存档 | 🟩 | Save v1: 存 (attr_id, base, current) 三元组 |
| 新增属性类型 | 🟩 前向兼容 | 新属性存档时追加，旧存档加载时赋予默认值 |
| 属性重命名 | 🟩 无损 | ID 不变仅文本变化 |
| DerivedFormula 变更 | 🟨 读档时公式重算 | 存档只存 raw 值，Derived 属性读档后按当前公式重算 |
| 属性删除 | 🟨 软删除 | deprecated 标记保留 ID，旧存档中该属性丢弃 |

**关键设计**: 存档只序列化 `AttributeValue`（base/current 值），不序列化 `DerivedFormula`。Derived 属性值在读档后由当前公式重新计算，保证公式演化不影响存档兼容。

---

## 9. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2（未来） | 属性值类型从 f32 扩展为结构化类型 | 迁移: f32 → struct { raw, bonus, penalty } |
| v3（未来） | 引入属性标签分类（如 DiminishingReturns） | 增加 AttributeDefinition.metadata 字段 |

---

## 10. Future Extension

- **属性维度扩展**：`AttributeValue` 从简单标量扩展为 { raw, bonus, penalty, multiplier } 四维结构
- **属性标签**：为属性附加标签（如「魔法」「物理」分类），用于 Condition 系统的过滤
- **属性曲线表**：引入外部曲线表，使 DerivedFormula 支持 `Curve(level) → value` 查表
- **属性快照树**：支持多版本快照比对（战斗前后属性变化追踪）

---

## 11. Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| DerivedFormula 依赖循环 | 属性系统死锁 | 配置加载时 DFS 循环检测，禁止循环注册 |
| f32 精度漂移 | 大量叠乘后精度损失 | 使用定点数或指定精度截断策略 |
| 快照数据膨胀 | 大型战斗中频繁快照导致存档过大 | 快照只存变更属性，全量快照仅在战斗开始/结束时拍摄 |
| BaseValue 被意外修改 | 运行时 BaseValue 变化导致回放不一致 | BaseValue 写操作必须通过 Progression/Equipment 领域，强制执行审计 |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| 三层分离（Def→Spec→Instance） | ✅ | AttributeDefinition → AttributeContainer → AttributeValue |
| Data Driven First | ✅ | 所有属性定义通过配置文件，DerivedFormula 纯数据 |
| Replay First | ✅ | Aggregator 管线确定性，跳过直接修改禁止 |
| Composition Over Inheritance | ✅ | 属性通过 Modifier 组合，不通过继承 |
| 宪法 §16.1 属性系统 | ✅ | Base/Current 分离，Aggregator 管线合规 |
