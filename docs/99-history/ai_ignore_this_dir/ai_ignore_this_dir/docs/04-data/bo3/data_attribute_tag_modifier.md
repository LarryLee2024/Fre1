# Data Architecture Proposal V2 — BG3 基础数据层提取

> 来源：`docs/其他/79博德3.md` 第1-2节（属性体系、标签体系）+ 第4节（Modifier部分）
> 提取角色：Data Architect
> 提取日期：2026-06-15
> V2变更：内嵌国际化架构（ADR-017），所有文本字段替换为本地化Key
> 国际化依据：`docs/08-decisions/ADR-017-国际化架构决策.md`

---

## Domain Ownership

| 领域 | 数据类别 | 来源章节 |
|------|----------|----------|
| **Attribute** | 三层嵌套派生属性模型 | 第1节 |
| **Tag** | 六大标签分类体系 + 层级继承 | 第2节 |
| **Modifier** | 数值加值型 + 优势劣势型 | 第4节 |

---

## Problem

BG3 的属性体系是「三层嵌套派生 + 骰子检定」模型，标签体系是「六维分类 + 层级继承」的通用底层语言，Modifier 包含 DND 特有的「优势/劣势」概率级修正。需提取可复用的数据结构，同时过滤掉 DND 专属复杂度（骰子检定、六维主属性），并确保所有文本字段符合 ADR-017 国际化规范。

---

## 国际化架构约束（ADR-017）

以下约束贯穿全文，不再逐项重复标注：

| 约束 | 规范 | 违反后果 |
|------|------|---------|
| Content数据只存Key | `name_key`/`desc_key` 替代 `name`/`description` | 无法多语言，违反宪法§17.2.2 |
| Key格式 | `namespace.id.suffix`（如 `attribute.attr_0001.name`） | — |
| 禁止语义化Key | ❌ `attribute.strength.name` | 改名后批量替换灾难 |
| 禁止无意义编号 | ❌ `text_001` | 维护时无法识别内容 |
| 禁止硬编码文本 | ❌ `name: "力量"` | 语言切换无法刷新 |
| Core层不调用LocalizationService | Core只存Key，解析在Content/UI层 | 架构越界 |

### Key命名空间与ID格式

| 领域 | 命名空间 | ID前缀 | 示例Key |
|------|---------|--------|---------|
| Attribute | `attribute` | `attr_` | `attribute.attr_0001.name` |
| Tag | `tag` | `tag_` | `tag.tag_0001.name` |
| Modifier | `modifier` | `mod_` | `modifier.mod_0001.name` |

### Key后缀规范

| 后缀 | 用途 | 必选 |
|------|------|------|
| `.name` | 名称 | ✅ |
| `.desc` | 描述 | ❌ |
| `.short_desc` | 简短描述 | ❌ |
| `.tooltip` | 提示文本 | ❌ |

---

## Schema Design

### 1. Attribute Domain — 三层派生属性

#### 1.1 提取的数据元素

| 数据元素 | 数据层 | 类型 | 说明 | BG3来源 |
|----------|--------|------|------|---------|
| `primary_attribute` | Definition | `Enum` | 基础属性枚举 | 六维主属性（力/敏/体/智/感/魅） |
| `primary_value` | Instance | `u32` | 基础属性原始值 | 属性值（如16点力量） |
| `modifier_value` | Instance | `i32` | 属性调整值 | ⌊(属性值 - 10) / 2⌋ |
| `proficiency_bonus` | Definition | `u32` | 熟练加值 | 随等级提升（12级+4） |
| `derived_attribute` | Definition | `Enum` | 派生属性枚举 | AC/攻击加值/法术DC/技能修正 |
| `derived_formula` | Definition | `FormulaId` | 派生计算公式引用 | 主属性调整值 + 熟练加值 + 其他修正 |
| `temporary_modifier` | Runtime | `ModifierInstance` | 临时修正 | Buff/法术/地形/站位 |

#### 1.2 BG3三层结构 → Lite-GAS双层映射

```
BG3 三层:                    Lite-GAS 双层:
┌─────────────────┐         ┌─────────────────┐
│ Core Base       │         │ Primary         │
│ (六维主属性)     │    →    │ (基础属性,只读)   │
├─────────────────┤         ├─────────────────┤
│ Derived Layer   │         │ Derived         │
│ (熟练+二级属性)  │    →    │ (派生属性,       │
├─────────────────┤         │  Modifier统一算) │
│ Temporary       │         │                 │
│ (临时修正)       │    →    │ Modifier管线     │
└─────────────────┘         └─────────────────┘
```

**关键决策**：BG3的「中间派生层」与「临时修正层」合并为 Lite-GAS 的 Derived + Modifier 管线。原因：
- SRPG不需要DND的「熟练加值」概念
- 所有派生计算统一走Modifier管线，符合Law 006

#### 1.3 Schema草案

```rust
// === Definition Layer ===

/// 属性定义（配置，运行时不可变）
///
/// 不变量：
/// - base_value >= 1
/// - 每个角色每个primary_attribute恰好一条记录
/// - name_key 必须符合 ADR-017 Key命名规范
struct AttributeDefinition {
    id: AttributeId,              // 如 "attr_0001"

    // 国际化字段（ADR-017: Content数据只存Key）
    name_key: String,             // "attribute.attr_0001.name"
    desc_key: String,             // "attribute.attr_0001.desc"

    category: AttributeCategory,  // Primary | Derived
    min_value: u32,               // 最小值（通常1）
    max_value: u32,               // 最大值（通常99）
    default_value: u32,           // 默认值
}

/// 派生属性定义
///
/// 不变量：
/// - formula_id 必须指向已注册的公式
/// - source_attributes 必须全部为 Primary 类型
struct DerivedAttributeDefinition {
    id: AttributeId,              // 如 "attr_0101"

    // 国际化字段
    name_key: String,             // "attribute.attr_0101.name"
    desc_key: String,             // "attribute.attr_0101.desc"

    formula_id: FormulaId,        // 引用公式，禁止内联公式（Law 002）
    source_attributes: Vec<AttributeId>, // 依赖的Primary属性
}

// === Instance Layer ===

/// 属性实例（每个角色一份）
///
/// 不变量：
/// - current_value 在 [min_value, max_value] 范围内
/// - Primary属性只通过Modifier管线修改
struct AttributeInstance {
    attribute_id: AttributeId,
    base_value: u32,              // 原始基础值
    // current_value 由 Modifier 管线实时计算，不存储
}

// === Runtime Layer ===

/// 属性计算结果缓存
///
/// 缓存。非事实源。可随时删除并重新计算。
///
/// 临时方案：当前直接实时计算。
/// 当Profile确认属性计算成为瓶颈时，可引入缓存层。
struct AttributeCache {
    attribute_id: AttributeId,
    computed_value: u32,          // Modifier管线计算后的最终值
    version: u64,                 // 缓存版本号，用于脏检测
}
```

#### 1.4 BG3骰子检定 → Lite-GAS确定性计算

| BG3机制 | Lite-GAS替代 | 原因 |
|---------|-------------|------|
| d20骰子检定 | 确定性公式计算 | Law 010: Replay优先 |
| 属性调整值 = ⌊(val-10)/2⌋ | 配置化公式引用 | Law 002: 规则属代码，内容属配置 |
| 优势/劣势(投2次取极值) | 概率修正映射为数值修正 | SRPG不需要d20概率模型 |
| 熟练加值 | 等级成长曲线配置 | 简化为配置驱动 |

---

### 2. Tag Domain — 六大分类 + 层级继承

#### 2.1 提取的数据元素

| 数据元素 | 数据层 | 类型 | 说明 | BG3来源 |
|----------|--------|------|------|---------|
| `tag_id` | Definition | `String` | 标签唯一标识 | 各类标签名称 |
| `tag_category` | Definition | `Enum` | 标签分类 | 六大分类 |
| `parent_tag` | Definition | `Option<TagId>` | 父标签（继承关系） | 如「火焰伤害」→「元素伤害」 |
| `tag_mutual_exclusion` | Definition | `Vec<(TagId, TagId)>` | 互斥标签对 | 同类型状态不叠加 |
| `tag_priority` | Definition | `u32` | 标签优先级 | 强控覆盖弱控 |
| `entity_tags` | Instance | `BitMask` | 实体拥有的标签集合 | 运行时标签状态 |

#### 2.2 六大标签分类 → Lite-GAS标签体系映射

| BG3分类 | BG3标签示例 | Lite-GAS映射 | 吸收策略 |
|---------|-----------|-------------|---------|
| 伤害/元素标签 | 火焰、寒冰、电击、毒素、挥砍、穿刺 | `damage_element` 分类 | ✅ 直接吸收，层级继承设计 |
| 生物类型标签 | 类人、亡灵、龙类、构造体、野兽 | `creature_type` 分类 | ✅ 直接吸收，目标筛选用 |
| 状态标签 | 眩晕、魅惑、沉默、隐形、无敌、专注中 | `status` 分类 | ✅ 直接吸收，能力前置校验用 |
| 环境/地形标签 | 水域、油面、火焰地表、高处、遮蔽 | `environment` 分类 | ⚠️ 按需吸收，SRPG环境交互范围待定 |
| 叙事/身份标签 | 博德之门人、夺心魔寄生者 | — | ❌ 不吸收，纯战斗SRPG不需要 |
| 机制元标签 | 可驱散、不可驱散、可反弹、触发反应 | `mechanism` 分类 | ✅ 直接吸收，底层规则判定用 |

#### 2.3 标签层级继承关系（可吸收部分）

```
damage_element
├── fire
├── ice
├── lightning
├── poison
├── slashing
└── piercing

creature_type
├── humanoid
├── undead
├── dragon
├── construct
└── beast

status
├── crowd_control
│   ├── stun
│   ├── charm
│   └── silence
├── visibility
│   ├── invisible
│   └── revealed
├── immunity
│   └── invulnerable
└── concentration

environment
├── surface
│   ├── water
│   ├── oil
│   └── fire_surface
├── elevation
│   └── high_ground
└── cover
    └── sheltered

mechanism
├── dispellable
├── undispellable
├── reflectable
└── reaction_trigger
```

#### 2.4 Schema草案

```rust
// === Definition Layer ===

/// 标签定义（配置，运行时不可变）
///
/// 不变量：
/// - id 全局唯一
/// - parent_id 必须指向已存在的标签或为None
/// - 不允许循环继承
/// - name_key 必须符合 ADR-017 Key命名规范
struct TagDefinition {
    id: TagId,                      // 如 "tag_0001"

    // 国际化字段（ADR-017）
    name_key: String,               // "tag.tag_0001.name"

    category: TagCategory,          // 六大分类
    parent_id: Option<TagId>,       // 层级继承
    priority: u32,                  // 优先级（强控 > 弱控）
    mutual_exclusions: Vec<TagId>,  // 互斥标签
}

/// 标签分类枚举
enum TagCategory {
    DamageElement,    // 伤害/元素
    CreatureType,     // 生物类型
    Status,           // 状态
    Environment,      // 环境/地形
    Mechanism,        // 机制元标签
}

// === Instance Layer ===

/// 实体标签集合（位掩码，O(1)查询）
///
/// 不变量：
/// - 互斥标签不能同时存在
/// - 标签来源可追溯（Trait/Equipment/Buff）
struct EntityTags {
    bitmask: u64,                    // 位掩码，运行时查询唯一方式
    sources: HashMap<TagId, TagSource>, // 标签来源追踪
}

/// 标签来源（三层管理）
enum TagSource {
    Intrinsic,    // 角色固有
    Equipment,    // 装备赋予
    Buff,         // Buff赋予
}

// === Runtime Layer ===

/// 标签查询结果
struct TagQueryResult {
    has_tag: bool,
    has_parent_tag: bool,  // 通过继承链查询
    source: Option<TagSource>,
}
```

---

### 3. Modifier Domain — 数值加值 + 优势劣势

#### 3.1 提取的数据元素

| 数据元素 | 数据层 | 类型 | 说明 | BG3来源 |
|----------|--------|------|------|---------|
| `modifier_type` | Definition | `Enum` | 加值类型分类 | 附魔/洞察/环境/士气/幸运 |
| `modifier_op` | Definition | `Enum` | 修正操作 | Add/Subtract/Multiply/Override |
| `modifier_value` | Instance | `f32` | 修正数值 | 具体加值量 |
| `advantage_state` | Instance | `Enum` | 优势/劣势状态 | 优势/劣势/普通/抵消 |
| `same_type_policy` | Definition | `Enum` | 同类型加值策略 | 取最高/不叠加 |

#### 3.2 BG3加值分类 → Lite-GAS Modifier类型映射

| BG3加值类型 | 说明 | Lite-GAS映射 |
|-----------|------|-------------|
| 附魔加值 | 装备附魔 | `enchantment` ModifierCategory |
| 洞察加值 | 专长/技能 | `insight` ModifierCategory |
| 环境加值 | 地形/站位 | `environment` ModifierCategory |
| 士气加值 | Buff/士气 | `morale` ModifierCategory |
| 幸运加值 | 特殊能力 | `luck` ModifierCategory |

**核心规则**：同类型不叠加取最高，异类型可叠加 → 直接映射为 Stacking 策略。

#### 3.3 优势/劣势 → Lite-GAS概率修正映射

| BG3机制 | Lite-GAS替代 | 说明 |
|---------|-------------|------|
| 优势(投2次取大) | `hit_rate_bonus: +25%` | 映射为命中率加成 |
| 劣势(投2次取小) | `hit_rate_bonus: -25%` | 映射为命中率惩罚 |
| 优势+劣势抵消 | 互相抵消逻辑 | 保留抵消规则 |
| 多个优势不叠加 | 优势状态为布尔 | 保留不叠加规则 |

#### 3.4 Schema草案

```rust
// === Definition Layer ===

/// Modifier定义（配置，运行时不可变）
///
/// 不变量：
/// - Modifier不拥有业务逻辑（Law 006）
/// - 只描述「修正什么、怎么修正」
/// - name_key 必须符合 ADR-017 Key命名规范
struct ModifierDefinition {
    id: ModifierId,                  // 如 "mod_0001"

    // 国际化字段（ADR-017）
    name_key: String,                // "modifier.mod_0001.name"
    desc_key: String,                // "modifier.mod_0001.desc"

    target_attribute: AttributeId,   // 修正目标属性
    operation: ModifierOp,           // 操作类型
    category: ModifierCategory,      // 加值类型（决定叠加规则）
    value: f32,                      // 修正值
}

/// Modifier操作类型
enum ModifierOp {
    Add,        // 加法
    Subtract,   // 减法
    Multiply,   // 乘法
    Override,   // 覆盖
}

/// Modifier加值类型（决定同类型叠加规则）
///
/// 同类型不叠加取最高，异类型可叠加。
/// 来源：BG3加值分类堆叠规则。
enum ModifierCategory {
    Enchantment,   // 附魔
    Insight,       // 洞察
    Environment,   // 环境
    Morale,        // 士气
    Luck,          // 幸运
    Base,          // 基础（无叠加限制）
}

// === Instance Layer ===

/// Modifier实例（运行时状态）
///
/// 不变量：
/// - 必须有来源（Law: Buffs must have a source）
/// - 必须有过期条件（Law: Buffs must have an expiration condition）
struct ModifierInstance {
    definition_id: ModifierId,
    source: ModifierSource,          // 来源追踪
    remaining_turns: Option<u32>,    // None=永久
    applied_value: f32,              // 实际应用值（经Stacking计算后）
}

/// 优势/劣势状态（独立于数值Modifier）
///
/// 规则：
/// - 多个优势不叠加，有一个即生效
/// - 优势+劣势互相抵消
/// - 抵消后变回普通
enum AdvantageState {
    Normal,
    Advantage,    // 映射为命中率+25%
    Disadvantage, // 映射为命中率-25%
}
```

---

## 国际化数据元素

### 本领域新增国际化数据元素

| # | 数据元素 | 数据层 | 类型 | 必选 | 说明 |
|---|---------|--------|------|------|------|
| I18N-01 | LocalizedKey | Infrastructure | Struct | ✅ | 类型安全的Key包装 |
| I18N-02 | LocalizationError | Infrastructure | Enum | ✅ | 国际化错误类型 |
| I18N-03 | VALID_NAMESPACES | Infrastructure | Set<String> | ✅ | 有效命名空间白名单 |
| I18N-04 | VALID_SUFFIXES | Infrastructure | Set<String> | ✅ | 有效后缀白名单 |

### LocalizedKey 类型定义

```rust
/// 本地化Key（类型安全包装）
///
/// 不变量：
/// - 必须符合命名规范：`namespace.id.suffix`
/// - namespace必须是已注册领域
/// - id必须是永久唯一ID
/// - suffix必须是name/desc/short_desc/tooltip之一
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocalizedKey {
    namespace: String,   // 如 "attribute"
    id: String,          // 如 "attr_0001"
    suffix: String,      // 如 "name"
}

impl LocalizedKey {
    /// 从字符串解析Key
    pub fn parse(key: &str) -> Result<Self, LocalizationError> { /* ... */ }

    /// 生成完整Key字符串
    pub fn to_key_string(&self) -> String {
        format!("{}.{}.{}", self.namespace, self.id, self.suffix)
    }
}
```

### LocalizationError 类型定义

```rust
/// 国际化错误
#[derive(Debug, thiserror::Error)]
pub enum LocalizationError {
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
    #[error("Unknown namespace: {0}")]
    UnknownNamespace(String),
    #[error("Invalid ID format: {0}")]
    InvalidIdFormat(String),
    #[error("Invalid suffix: {0}")]
    InvalidSuffix(String),
    #[error("Translation not found: {0}")]
    TranslationNotFound(String),
    #[error("Missing required argument: {0}")]
    MissingArgument(String),
}
```

---

## Dependency Analysis

### 领域间依赖关系

```
Tag ──────→ Attribute (标签影响属性计算，如「沉默」影响施法)
  │
  └──────→ Modifier (标签匹配抗性/免疫/易伤，影响Modifier生效)

Attribute ─→ Modifier (属性值通过Modifier管线计算)

Modifier ─→ Stacking (同类型Modifier叠加规则由Stacking策略决定)
```

### 数据依赖规则

| 依赖 | 规则 | 违反后果 |
|------|------|---------|
| TagDefinition.parent_id | 必须指向已存在标签或为None | 加载时校验失败 |
| DerivedAttributeDefinition.source_attributes | 必须全部为Primary类型 | 派生计算循环依赖 |
| ModifierDefinition.target_attribute | 必须指向已注册属性 | Modifier应用失败 |
| ModifierCategory | 同类型不叠加取最高 | 数值膨胀 |
| Tag互斥 | 互斥标签不能同时存在 | 状态冲突 |
| name_key/desc_key | 必须符合ADR-017 Key格式 | 国际化解析失败 |

---

## Validation Rules

### Attribute校验

| 规则 | 校验时机 | 错误级别 |
|------|---------|---------|
| Primary属性base_value ∈ [1, 99] | 加载时 | ERROR |
| Derived属性formula_id必须已注册 | 加载时 | ERROR |
| 派生属性source_attributes无循环依赖 | 加载时 | ERROR |
| 每个角色每个Attribute恰好一条Instance | 运行时 | WARN |
| name_key格式必须符合`attribute.attr_XXXX.suffix` | 加载时 | ERROR |
| desc_key格式必须符合`attribute.attr_XXXX.suffix` | 加载时 | WARN |

### Tag校验

| 规则 | 校验时机 | 错误级别 |
|------|---------|---------|
| TagId全局唯一 | 加载时 | ERROR |
| parent_id无循环继承 | 加载时 | ERROR |
| 互斥标签不能同时存在于同一实体 | 运行时 | WARN |
| 机制元标签不能由Buff来源赋予 | 加载时 | ERROR |
| name_key格式必须符合`tag.tag_XXXX.suffix` | 加载时 | ERROR |

### Modifier校验

| 规则 | 校验时机 | 错误级别 |
|------|---------|---------|
| Modifier必须有source | 应用时 | ERROR |
| Modifier必须有过期条件 | 应用时 | ERROR |
| target_attribute必须已注册 | 加载时 | ERROR |
| 同Category的Modifier不叠加，取最高 | 运行时 | — (正常行为) |
| name_key格式必须符合`modifier.mod_XXXX.suffix` | 加载时 | ERROR |

### 国际化校验（ADR-017）

| 规则 | 校验时机 | 错误级别 |
|------|---------|---------|
| name_key必选，不能为空 | 加载时 | ERROR |
| Key格式必须为`namespace.id.suffix` | 加载时 | ERROR |
| namespace必须是已注册领域 | 加载时 | ERROR |
| id必须是永久唯一ID格式 | 加载时 | ERROR |
| suffix必须是name/desc/short_desc/tooltip之一 | 加载时 | ERROR |
| 禁止语义化Key（如`attribute.strength.name`） | 加载时 | ERROR |
| 禁止无意义编号Key（如`text_001`） | 加载时 | ERROR |
| Key不能包含中文/特殊字符 | 加载时 | ERROR |

---

## Replay Compatibility

| 数据元素 | Replay影响 | 处理策略 |
|---------|-----------|---------|
| 属性计算公式 | 必须确定性 | 公式引用ID，不内联（Law 002） |
| 标签状态 | 必须确定性 | 标签位掩码序列化到Replay |
| Modifier应用顺序 | 必须确定性 | 按ModifierCategory优先级排序 |
| 优势/劣势 | 必须确定性 | 映射为固定数值修正，不使用随机 |
| BG3骰子检定 | ❌ 不兼容 | 替换为确定性公式计算 |
| name_key/desc_key | 不影响Replay | Key是确定性字符串 |

**Law 010检查**：属性计算、标签查询、Modifier应用全部为确定性操作。BG3的d20骰子机制被替换为确定性公式，满足Replay要求。国际化Key不影响Replay。✅ 通过。

---

## Save Compatibility

| 数据元素 | Save版本策略 | 迁移考虑 |
|---------|-------------|---------|
| AttributeDefinition | 大版本+1才可删除/改类型 | 新属性必须有default_value |
| TagDefinition | 小版本可新增标签 | 新标签默认不存在于旧存档 |
| ModifierCategory | 只能新增不能删除 | 旧存档的Category映射到Base |
| AttributeInstance | 新字段需默认值 | base_value缺失时用default_value |
| name_key/desc_key | 新字段需默认值 | 旧存档硬编码文本需迁移为Key |

**存档迁移**：旧存档中的硬编码文本需通过 `LEGACY_TEXT_KEY_MAP` 映射表转换为Key。

---

## Migration Strategy

### 从BG3模型迁移到Lite-GAS

| 迁移项 | BG3模型 | Lite-GAS目标 | 迁移路径 |
|--------|---------|-------------|---------|
| 六维主属性 | STR/DEX/CON/INT/WIS/CHA | 项目自定义Primary属性 | 重新设计属性枚举 |
| 熟练加值 | 等级驱动 | 等级成长曲线配置 | 配置化公式 |
| d20检定 | 随机骰子 | 确定性公式 | 完全替换 |
| 优势/劣势 | 概率修正 | 数值修正映射 | 映射表 |
| 六大标签分类 | BG3完整分类 | 按需吸收5类（排除叙事类） | 过滤迁移 |
| 硬编码文本 | `name: "力量"` | `name_key: "attribute.attr_0001.name"` | Key映射表 |

---

## Future Extension

| 扩展点 | 当前设计 | 未来可能 |
|--------|---------|---------|
| 属性类型 | Primary/Derived双层 | 可新增Computed层（多级派生） |
| 标签分类 | 5类（排除叙事） | 可新增AI标签、关卡标签 |
| Modifier操作 | Add/Subtract/Multiply/Override | 可新增Clamp、Scale |
| 优势/劣势 | 映射为固定数值 | 可配置化映射比例 |
| 标签继承 | 单继承 | 可扩展为多继承（标签组合） |
| Key后缀 | 4种 | 可新增`.flavor`、`.lore` |

---

## Risks

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| BG3骰子检定习惯影响设计 | 可能引入非确定性 | 严格Law 010审查 |
| 标签层级继承过深 | 查询性能下降 | 限制继承深度≤3 |
| ModifierCategory膨胀 | 叠加规则复杂化 | 控制Category数量≤8 |
| 优势/劣势映射为固定值不够灵活 | 玩家感知差异 | 配置化映射比例 |
| Key映射表维护成本 | 旧存档迁移复杂 | 自动化迁移工具 |

---

## Constitution Check

| Data Law / 规范 | 检查结果 | 说明 |
|----------------|---------|------|
| Law 001 | ✅ 通过 | AttributeDefinition/AttributeInstance分离；TagDefinition/EntityTags分离；ModifierDefinition/ModifierInstance分离 |
| Law 002 | ✅ 通过 | 派生公式通过formula_id引用，不内联 |
| Law 003 | ✅ 通过 | Modifier引用AttributeId，不重复定义属性 |
| Law 006 | ✅ 通过 | Modifier只改变数值，无业务逻辑 |
| Law 008 | ✅ 通过 | 同类型Modifier叠加规则归属Stacking策略 |
| Law 010 | ✅ 通过 | 所有计算确定性，骰子机制已替换 |
| ADR-017 | ✅ 通过 | 所有文本字段使用name_key/desc_key，Key格式符合规范 |
| 宪法§17.2.2 | ✅ 通过 | 禁止硬编码玩家可见文本 |

**[Data Exemption]**：无。

---

## 数据清单汇总

### Attribute Domain

| # | 数据元素 | 数据层 | 类型 | 必选 | 来源 |
|---|---------|--------|------|------|------|
| A-01 | AttributeId | Definition | String | ✅ | BG3§1 |
| A-02 | AttributeCategory | Definition | Enum(Primary/Derived) | ✅ | BG3§1 |
| A-03 | AttributeDefinition.name_key | Definition | String(LocalizedKey) | ✅ | BG3§1 + ADR-017 |
| A-04 | AttributeDefinition.desc_key | Definition | String(LocalizedKey) | ❌ | BG3§1 + ADR-017 |
| A-05 | AttributeDefinition.min_value | Definition | u32 | ✅ | BG3§1 |
| A-06 | AttributeDefinition.max_value | Definition | u32 | ✅ | BG3§1 |
| A-07 | AttributeDefinition.default_value | Definition | u32 | ✅ | BG3§1 |
| A-08 | DerivedAttributeDefinition.formula_id | Definition | FormulaId | ✅ | BG3§1 |
| A-09 | DerivedAttributeDefinition.source_attributes | Definition | Vec<AttributeId> | ✅ | BG3§1 |
| A-10 | AttributeInstance.base_value | Instance | u32 | ✅ | BG3§1 |
| A-11 | AttributeCache.computed_value | Runtime | u32 | — | BG3§1 |

### Tag Domain

| # | 数据元素 | 数据层 | 类型 | 必选 | 来源 |
|---|---------|--------|------|------|------|
| T-01 | TagId | Definition | String | ✅ | BG3§2 |
| T-02 | TagCategory | Definition | Enum(5类) | ✅ | BG3§2 |
| T-03 | TagDefinition.name_key | Definition | String(LocalizedKey) | ✅ | BG3§2 + ADR-017 |
| T-04 | TagDefinition.parent_id | Definition | Option<TagId> | ❌ | BG3§2 |
| T-05 | TagDefinition.priority | Definition | u32 | ❌ | BG3§2 |
| T-06 | TagDefinition.mutual_exclusions | Definition | Vec<TagId> | ❌ | BG3§2 |
| T-07 | EntityTags.bitmask | Instance | u64 | ✅ | BG3§2 |
| T-08 | EntityTags.sources | Instance | HashMap<TagId, TagSource> | ✅ | BG3§2 |
| T-09 | TagSource | Instance | Enum(Intrinsic/Equipment/Buff) | ✅ | BG3§2 |

### Modifier Domain

| # | 数据元素 | 数据层 | 类型 | 必选 | 来源 |
|---|---------|--------|------|------|------|
| M-01 | ModifierId | Definition | String | ✅ | BG3§4 |
| M-02 | ModifierOp | Definition | Enum(4种) | ✅ | BG3§4 |
| M-03 | ModifierCategory | Definition | Enum(6种) | ✅ | BG3§4 |
| M-04 | ModifierDefinition.name_key | Definition | String(LocalizedKey) | ✅ | BG3§4 + ADR-017 |
| M-05 | ModifierDefinition.desc_key | Definition | String(LocalizedKey) | ❌ | BG3§4 + ADR-017 |
| M-06 | ModifierDefinition.target_attribute | Definition | AttributeId | ✅ | BG3§4 |
| M-07 | ModifierDefinition.value | Definition | f32 | ✅ | BG3§4 |
| M-08 | ModifierInstance.source | Instance | ModifierSource | ✅ | BG3§4 |
| M-09 | ModifierInstance.remaining_turns | Instance | Option<u32> | ✅ | BG3§4 |
| M-10 | ModifierInstance.applied_value | Instance | f32 | ✅ | BG3§4 |
| M-11 | AdvantageState | Instance | Enum(3种) | ❌ | BG3§4 |

### I18n Infrastructure

| # | 数据元素 | 数据层 | 类型 | 必选 | 来源 |
|---|---------|--------|------|------|------|
| I18N-01 | LocalizedKey | Infrastructure | Struct | ✅ | ADR-017 |
| I18N-02 | LocalizationError | Infrastructure | Enum | ✅ | ADR-017 |
| I18N-03 | VALID_NAMESPACES | Infrastructure | Set<String> | ✅ | ADR-017 |
| I18N-04 | VALID_SUFFIXES | Infrastructure | Set<String> | ✅ | ADR-017 |
