---
id: capabilities.execution.schema.v1
title: Execution Schema — 执行计算数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: runtime
replay-safe: true
---

# Execution Schema — 执行计算数据架构

> **领域归属**: Capabilities — 行为表现层 | **依赖 Schema**: GameplayContext, Attribute, Modifier | **定义依据**: `docs/02-domain/execution_domain.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `ExecutionType` | Definition | 执行计算类型枚举（Damage/Heal/Custom） |
| `ExecutionContext` | Runtime | 执行计算的完整输入上下文 |
| `ExecutionResult` | Runtime | 执行计算的结果（数值 + 效果） |
| `CustomExecutionRegistry` | Definition | 自定义执行计算的注册表 |

---

## 2. Problem

Execution 是能力系统的「计算引擎」——接收 GameplayContext 和 Targeting 结果，调用 Domain 公式计算伤害/治疗/其他效果，输出 Effect。Schema 必须解决：
- 执行计算的标准化输入（从哪里读取属性值、技能参数、环境因子）
- 计算结果的标准化输出（数值 + 产生的 Effect）
- 自定义执行计算的扩展接口
- Execution 不包含业务公式（公式在 Domains/rules/）

---

## 3. Schema Design

### 3.1 ExecutionType（Definition 层）

```rust
enum ExecutionType {
    /// 伤害计算
    Damage(DamageParams),
    /// 治疗计算
    Heal(HealParams),
    /// 自定义计算
    Custom(CustomExecutionRef),
    /// 直接修改属性（如设置某属性为固定值）
    DirectAttributeMod {
        attribute_id: AttributeId,
        operation: DirectOp,
        value: ScalableValue,
    },
    /// 空执行（什么都不做，用于占位）
    None,
}

enum DirectOp {
    Set,
    Add,
    Subtract,
    Multiply,
}
```

### 3.2 DamageParams（Definition 层）

```rust
struct DamageParams {
    /// 伤害公式 ID（指向 Domains/rules/damage_formula.rs 中的具体公式）
    formula_id: String,

    /// 伤害类型标签
    damage_type: Vec<TagId>,

    /// 基础伤害骰（如 "8d6"）
    damage_dice: Option<DiceDef>,

    /// 固定伤害加值
    flat_bonus: Option<ScalableValue>,

    /// 属性修正（如力量修正、智力修正）
    attribute_modifier: Option<AttributeModifierDef>,

    /// 是否可暴击
    can_critical: bool,

    /// 暴击倍率
    critical_multiplier: f32,
}

struct DiceDef {
    /// 骰子个数
    count: u8,
    /// 骰子面数
    sides: u8,
    /// 每级加骰（如 Lv3 时 8d6 → 10d6）
    per_level_count: Option<u8>,
}

struct AttributeModifierDef {
    /// 用于修正的属性
    source_attribute: AttributeId,
    /// 修正系数（如 1.0 = 全值，0.5 = 半值）
    multiplier: f32,
    /// 是否使用基础值而非当前值
    use_base: bool,
}
```

### 3.3 HealParams（Definition 层）

```rust
struct HealParams {
    /// 治疗公式 ID
    formula_id: String,

    /// 基础治疗量
    base_heal: ScalableValue,

    /// 属性修正
    attribute_modifier: Option<AttributeModifierDef>,

    /// 是否为临时生命值
    is_temporary_hp: bool,
}
```

### 3.4 CustomExecutionRef（Definition 层）

```rust
struct CustomExecutionRef {
    /// 自定义执行 ID（对应 CustomExecutionRegistry 中的注册项）
    execution_id: String,

    /// 自定义参数（传递给自定义计算的领域特定数据）
    params: HashMap<String, ConditionParam>,
}
```

### 3.5 ExecutionContext（Runtime 层）

```rust
struct ExecutionContext {
    /// 执行类型
    execution_type: ExecutionType,

    /// 来源实体（施法者/攻击者）
    source_entity: EntityId,

    /// 目标实体
    target_entity: EntityId,

    /// 来源属性快照
    source_attributes: HashMap<AttributeId, f32>,

    /// 目标属性快照
    target_attributes: HashMap<AttributeId, f32>,

    /// 来源 Modifier 状态
    source_modifiers: Option<Vec<ModifierData>>,

    /// 目标 Modifier 状态
    target_modifiers: Option<Vec<ModifierData>>,

    /// 来源标签
    source_tags: Vec<TagId>,

    /// 目标标签
    target_tags: Vec<TagId>,

    /// 技能参数（来自 AbilityDef/EffectDef）
    ability_params: AbilityExecutionParams,

    /// 环境参数
    environment: EnvironmentParams,

    /// 原始 GameplayContext
    gameplay_context: Option<GameplayContextData>,
}

struct AbilityExecutionParams {
    ability_def_id: Option<AbilityDefId>,
    ability_level: u8,
    targeting_result: Option<TargetData>,
    effect_override: Option<EffectOverride>,
}

struct EnvironmentParams {
    /// 是否在高地
    is_high_ground: bool,
    /// 是否在掩体后
    has_cover: bool,
    /// 是否被夹击
    is_flanked: bool,
    /// 当前地形标签
    terrain_tags: Vec<TagId>,
    /// 当前回合数
    current_turn: u32,
}
```

### 3.6 ExecutionResult（Runtime 层）

```rust
struct ExecutionResult {
    /// 执行是否成功
    success: bool,

    /// 计算出的数值（伤害量/治疗量等）
    value: f32,

    /// 是否为暴击
    was_critical: bool,

    /// 是否为未命中
    was_miss: bool,

    /// 产生的效果列表（Execution 可以产生多个 Effect，如伤害 + 击退）
    produced_effects: Vec<ProducedEffect>,

    /// 计算过程追踪（调试用）
    calc_trace: Option<CalcTrace>,
}

struct ProducedEffect {
    /// 要应用的 EffectDef ID
    effect_def_id: EffectDefId,

    /// 参数覆盖
    override_params: Option<EffectOverride>,

    /// 目标（None = 默认 Execution 的目标）
    target: Option<EntityId>,

    /// 应用延迟
    delay_frames: Option<u64>,
}

struct CalcTrace {
    formula_id: String,
    inputs: HashMap<String, f32>,
    intermediate_values: Vec<(String, f32)>,
    output: f32,
}
```

### 3.7 CustomExecutionRegistry（Definition 层）

```rust
/// 自定义执行计算的注册表。
/// Domain 在初始化时注册自定义执行逻辑。
struct CustomExecutionRegistration {
    /// 唯一标识
    execution_id: String,

    /// 所属 Domain
    owning_domain: String,

    /// 描述
    description_key: LocalizationKey,

    /// 参数 Schema（定义域特定参数的结构）
    param_schema: HashMap<String, ParamFieldDef>,
}

struct ParamFieldDef {
    field_type: ParamFieldType,
    required: bool,
    description_key: LocalizationKey,
}

enum ParamFieldType {
    Float,
    Integer,
    Boolean,
    AttributeId,
    TagId,
    EntityId,
}
```

### 3.8 ExecutionConfig（Definition 层 — 配置格式）

```yaml
# RON 配置示例 — Execution 配置（嵌入在 EffectDef 中使用）
ExecutionConfig:
  # 示例1: 物理伤害
  - execution_type:
      Damage:
        formula_id: "dnd_5e_damage"
        damage_type: ["tag_000003"]   # DamageType.Physical.Slashing
        damage_dice:
          count: 1
          sides: 8
        flat_bonus:
          Fixed: 3.0
        attribute_modifier:
          source_attribute: "attr_000001"   # 力量
          multiplier: 1.0
          use_base: false
        can_critical: true
        critical_multiplier: 2.0

  # 示例2: 治疗
  - execution_type:
      Heal:
        formula_id: "dnd_5e_heal"
        base_heal:
          Fixed: 10.0
        attribute_modifier:
          source_attribute: "attr_000006"   # 感知
          multiplier: 1.0
          use_base: false

  # 示例3: 自定义执行（击退）
  - execution_type:
      Custom:
        execution_id: "tactical.knockback"
        params:
          distance: Integer(2)
          direction: String("away_from_caster")
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 可热重载 | 备注 |
|----------|-------|--------|----------|------|
| `ExecutionType` | Definition | 是（Def 内嵌） | 是 | 执行类型枚举 |
| `DamageParams` / `HealParams` | Definition | 是（Def 内嵌） | 是 | 计算参数 |
| `CustomExecutionRef` | Definition | 是（Def 内嵌） | 是 | 自定义引用 |
| `ExecutionContext` | Runtime | 否 | 否 | 瞬时输入 |
| `ExecutionResult` | Runtime | 否 | 否 | 瞬时输出 |
| `CustomExecutionRegistry` | Definition | 代码注册 | 否 | 启动时注册 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → GameplayContextSchema | ExecutionContext 从 GameplayContext 派生 |
| 依赖 | → AttributeSchema | source/target_attributes |
| 依赖 | → ModifierSchema | source/target_modifiers |
| 依赖 | → TagSchema | damage_type, source/target_tags |
| 被依赖 | ← AbilitySchema | 技能激活后调用 Execution |
| 被依赖 | ← EffectSchema | Effect 的 Period Tick 调用 Execution |
| 被依赖 | ← CombatSchema | 战斗中的伤害/治疗结算 |

---

## 6. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | formula_id 已注册 | Def 加载 | formula_id 在 FormulaRegistry 中存在 |
| V2 | CustomExecution ID 唯一 | 启动注册 | 无重复 execution_id |
| V3 | 暴击倍率合法 | Def 加载 | critical_multiplier ≥ 1.0 |
| V4 | 结果值非负 | 运行时 | damage ≥ 0, heal ≥ 0 |
| V5 | 属性引用存在 | Def 加载 | attribute_modifier.source_attribute 已注册 |

---

## 7. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| 伤害计算 | 🟩 完全确定 | 公式由 formula_id 确定，输入确定 |
| Dice 投骰 | 🟩 确定 | 由确定性 RNG 驱动，种子来自 ReplayFrame |
| 暴击判定 | 🟩 确定 | 由 RNG + 暴击率公式确定 |
| 自定义执行 | 🟩 需要 Domain 保证 | CustomExecution 实现必须保证确定性 |

---

## 8. Save Compatibility

Execution 是纯运行时计算，计算参数（DamageParams 等）随 Def 保存。计算结果（ExecutionResult）不持久化——Effect 系统会根据计算结果产生 EffectInstance 来持久化效果。

---

## 9. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2（未来） | DamageParams 增加元素交互 | 新增 optional 字段 |

---

## 10. Future Extension

- **公式注册表增强**: formula_id 支持参数化公式（同一公式在不同上下文中使用不同系数）
- **计算管道**: 多个 Execution 串联执行（先计算伤害，再计算附加效果）
- **执行缓冲区**: 批量执行计算（处理 AOE 技能对多目标的一次完整结算）

---

## 11. Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| CustomExecution 非确定性 | 自定义计算可能引入外部状态 | 强制 CustomExecution 在测试中验证确定性 |
| Dice 与 RNG 耦合 | 骰子随机性依赖 RNG 种子管理 | DiceDef 直接使用 ContextChain 中的 RNG 种子 |
| 公式 ID 硬编码 | formula_id 字符串拼写错误 | Registry 启动时校验所有引用的 formula_id |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| Execution 不包含公式 | ✅ | formula_id 引用 Domains/rules/ |
| 计算结果可追踪 | ✅ | CalcTrace 记录完整计算过程 |
| Replay First | ✅ | 确定性的骰子 + 公式 |
