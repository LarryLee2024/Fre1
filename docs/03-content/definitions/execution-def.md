---
id: 03-content.definitions.execution-def
title: ExecutionDef — Execution Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# ExecutionDef — Execution Content Def 定义

> **Content Layer**: L1 Capability | **领域规则**: `docs/02-domain/capabilities/execution_domain.md` | **数据 Schema**: `docs/04-data/capabilities/execution_schema.md` | **插件代码**: `src/content/plugins/execution_plugin.rs`

---

## 1. Overview

ExecutionDef 是**可命名的执行计算定义**——定义一个伤害/治疗/属性修改的计算公式引用和参数。Execution 是能力系统的计算引擎：

- 伤害计算：引用公式 ID + 骰子参数 + 属性修正 + 暴击参数
- 治疗计算：引用公式 ID + 基础治疗量 + 属性修正
- 直接属性修改：设置属性为固定值、加法、减法、乘法
- 自定义执行：由 Domain 注册的自定义计算逻辑

### 设计原则

- **Execution 不包含业务公式**：公式实现位于 `core/domains/rules/`，ExecutionDef 只通过 `formula_id` 引用
- **Execution 是可复用的计算模板**：同一计算规则（如 `exec:melee_damage`）可被多个 EffectDef 引用
- **Execution 是纯 Def**：不含运行时状态，所有状态通过 ExecutionContext 传递

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `execution_domain.md` | 执行计算流程、公式注册表、ExecutionContext 和 ExecutionResult 的数据生命周期 |
| `execution_schema.md` | ExecutionType、DamageParams、HealParams、CustomExecutionRef 的完整数据结构 |
| `effect-def.md` | 本 Def 被 EffectDef.execution_def 引用 |
| `ability-def.md` | 本 Def 被 AbilityDef 的效果链间接引用 |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// Execution Def 定义——可命名的执行计算模板。
///
/// ExecutionDef 定义一个计算规则（伤害/治疗/属性修改/自定义），
/// 可在多个 EffectDef 之间共享。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct ExecutionDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID
    pub id: ExecutionId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 执行核心 ──
    /// 执行计算类型
    pub execution_type: ExecutionType,

    // ── 元数据 ──
    /// 所属领域（用于 CustomExecution 的归属追踪）
    pub domain: Option<String>,

    /// 分类标签
    pub tags: Vec<TagId>,
}

/// 执行计算类型
#[derive(Deserialize, Clone, Debug)]
pub enum ExecutionType {
    /// 伤害计算
    Damage(DamageParams),
    /// 治疗计算
    Heal(HealParams),
    /// 自定义计算
    Custom(CustomExecutionRef),
    /// 直接修改属性
    DirectAttributeMod {
        attribute_id: AttributeId,
        operation: DirectOp,
        value: ScalableValue,
    },
    /// 空执行（占位）
    None,
}

/// 伤害计算参数
#[derive(Deserialize, Clone, Debug)]
pub struct DamageParams {
    /// 伤害公式 ID（指向 Domains/rules/ 中的具体公式）
    pub formula_id: String,
    /// 伤害类型标签
    pub damage_type: Vec<TagId>,
    /// 基础伤害骰子
    pub damage_dice: Option<DiceDef>,
    /// 固定伤害加值
    pub flat_bonus: Option<ScalableValue>,
    /// 属性修正（如力量修正）
    pub attribute_modifier: Option<AttributeModifierDef>,
    /// 是否可暴击
    pub can_critical: bool,
    /// 暴击倍率
    pub critical_multiplier: f32,
}

/// 治疗计算参数
#[derive(Deserialize, Clone, Debug)]
pub struct HealParams {
    /// 治疗公式 ID
    pub formula_id: String,
    /// 基础治疗量
    pub base_heal: ScalableValue,
    /// 属性修正
    pub attribute_modifier: Option<AttributeModifierDef>,
    /// 是否为临时生命值
    pub is_temporary_hp: bool,
}

/// 自定义执行引用
#[derive(Deserialize, Clone, Debug)]
pub struct CustomExecutionRef {
    /// 自定义执行 ID（对应 CustomExecutionRegistry 中的注册项）
    pub execution_id: String,
    /// 自定义参数
    pub params: HashMap<String, ConditionParam>,
}

/// 骰子定义
#[derive(Deserialize, Clone, Debug)]
pub struct DiceDef {
    pub count: u8,
    pub sides: u8,
    pub per_level_count: Option<u8>,
}

/// 属性修正定义
#[derive(Deserialize, Clone, Debug)]
pub struct AttributeModifierDef {
    pub source_attribute: AttributeId,
    pub multiplier: f32,
    pub use_base: bool,
}

/// 直接操作枚举
#[derive(Deserialize, Clone, Debug)]
pub enum DirectOp {
    Set, Add, Subtract, Multiply,
}
```

### 字段说明

- **`execution_type`**: 五种计算类型。Damage 和 Heal 使用 formula_id 引用领域公式，Custom 使用 execution_id 引用注册的自定义逻辑，DirectAttributeMod 直接操作属性值
- **`formula_id`**: 指向 `core/domains/rules/` 中的具体公式实现。公式 ID 在启动时通过 FormulaRegistry 注册，加载时校验存在性
- **`damage_dice`**: 骰子参数（如 8d6），由确定性 RNG 驱动以实现回放兼容
- **`can_critical / critical_multiplier`**: 暴击系统参数。暴击判定由领域规则 + 确定性 RNG 完成
- **`attribute_modifier`**: 属性修正（如"力量修正 × 1.0 加到伤害中"），`use_base` 控制使用基础值还是当前值

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// ExecutionDef 注册插件
pub struct ExecutionDefPlugin;

impl Plugin for ExecutionDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<ExecutionDef>();
        app.init_asset_loader::<RonAssetLoader<ExecutionDef>>();
        app.insert_resource(DefRegistry::<ExecutionDef>::new());

        app.add_systems(
            PreUpdate,
            load_execution_defs
                .run_if(resource_changed::<Assets<ExecutionDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按 ID 查找 ExecutionDef
pub fn get_execution_def(id: &ExecutionId, registry: &DefRegistry<ExecutionDef>) -> Option<&ExecutionDef> {
    registry.get(id)
}
```

### 注册生命周期

```
Load (executions.ron) → Deserialize → Validate → Register (DefRegistry<ExecutionDef>) → Freeze
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | ExecutionId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | ExecutionType::Damage 时 formula_id 非空 | 伤害计算必须指定公式 |
| V4 | ExecutionType::Heal 时 formula_id 非空 | 治疗计算必须指定公式 |
| V5 | `critical_multiplier >= 1.0` (can_critical = true 时) | 暴击倍率至少为 1 |
| V6 | DiceDef.count >= 1, DiceDef.sides >= 2 | 骰子参数合法 |
| V7 | ExecutionType::None 不能有附加参数 | None 类型的 params 应为空 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V8 | `formula_id` 在 FormulaRegistry 中已注册 | 公式实现必须预加载 |
| V9 | `damage_type` 中的 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V10 | `attribute_modifier.source_attribute` 已注册 | 在 DefRegistry<AttributeDef> 中存在 |
| V11 | `flat_bonus` 的 AttributeScaling.source_attribute (如果适用) 已注册 | 在 DefRegistry<AttributeDef> 中存在 |
| V12 | ExecutionType::DirectAttributeMod.attribute_id 已注册 | 在 DefRegistry<AttributeDef> 中存在 |
| V13 | ExecutionDef 不得引用任何 L2+ Def | L1 内容不可引用 Entity/Gameplay/World 层内容 |

---

## 5. RON 示例

```ron
// ExecutionDef 示例：近战物理伤害
//
// 标准的近战伤害计算：1d8 + 力量修正，可暴击。
(
    id: "exec:melee_damage",
    name_key: "execution.melee_damage.name",
    description_key: "execution.melee_damage.desc",
    schema_version: 1,

    execution_type: Damage((
        formula_id: "dnd_5e_weapon_damage",
        damage_type: ["tag:damage_type_physical"],
        damage_dice: Some((
            count: 1,
            sides: 8,
        )),
        flat_bonus: Some(AttributeScaling(
            source_attribute: "attr:strength",
            ratio: 1.0,
        )),
        attribute_modifier: Some((
            source_attribute: "attr:strength",
            multiplier: 1.0,
            use_base: false,
        )),
        can_critical: true,
        critical_multiplier: 2.0,
    )),

    domain: Some("combat"),
    tags: ["tag:combat", "tag:damage", "tag:physical"],
)
```

```ron
// ExecutionDef 示例：基础治疗
//
// 标准治疗：10 + 感知修正。
(
    id: "exec:basic_heal",
    name_key: "execution.basic_heal.name",
    description_key: "execution.basic_heal.desc",
    schema_version: 1,

    execution_type: Heal((
        formula_id: "dnd_5e_heal",
        base_heal: Fixed(10.0),
        attribute_modifier: Some((
            source_attribute: "attr:wisdom",
            multiplier: 1.0,
            use_base: false,
        )),
        is_temporary_hp: false,
    )),

    domain: Some("combat"),
    tags: ["tag:combat", "tag:heal"],
)
```
