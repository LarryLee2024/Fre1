# Attribute 领域 — 铃兰之剑数据提取

> 领域：Attribute | 来源：78铃兰.md §一、§六、补充11 | 数据层：Definition + Instance + Runtime

---

## 一、数据实体清单

### 1.1 AttributeDefinition（Definition层）

角色固有属性定义，战斗中不可变，是所有百分比加成的计算基数。

| 字段名 | 类型 | 约束 | 说明 | 来源 |
|--------|------|------|------|------|
| `id` | AttributeId | PK | 属性唯一标识 | §一.1 |
| `name_key` | String | - | 属性名称本地化Key | §一.1 + ADR-017 |
| `category` | Enum | core/secondary | 属性分类 | §一.1 |
| `base_value` | f32 | ≥0 | 基础数值 | §一.1 |
| `min_value` | f32 | ≥0 | 属性下限 | 补充11 |
| `max_value` | f32 | >min_value | 属性上限 | 补充11 |

#### 核心五维（category = core）

| AttributeId | 名称 | 下限 | 上限 | 说明 |
|-------------|------|------|------|------|
| `phys_atk` | 物理攻击 | 0 | 无上限 | 物理伤害基数 |
| `magic_atk` | 魔法攻击 | 0 | 无上限 | 魔法伤害基数 |
| `phys_def` | 物理防御 | 0 | 无上限 | 最低为0，不可为负 |
| `magic_def` | 魔法防御 | 0 | 无上限 | 最低为0，不可为负 |
| `max_hp` | 最大生命值 | 1 | 无上限 | 角色生命上限 |

#### 次级属性（category = secondary）

| AttributeId | 名称 | 下限 | 上限 | 说明 |
|-------------|------|------|------|------|
| `crit_rate` | 暴击率 | 0 | 0.95 | 上限95% |
| `crit_dmg` | 暴击伤害 | 1.5 | 无上限 | 默认150% |
| `move_range` | 移动范围 | 1 | 无上限 | 最低为1格 |
| `atk_range` | 攻击范围 | 1 | 无上限 | 最低为1格 |
| `hit_rate` | 命中率 | 0 | 1.0 | 上限100% |
| `dodge_rate` | 闪避率 | 0 | 0.8 | 上限80% |

### 1.2 AttributeInstance（Instance层）

战斗中每个实体的属性实例，包含基础值和运行时修正。

| 字段名 | 类型 | 说明 | 来源 |
|--------|------|------|------|
| `entity` | Entity | 所属实体 | - |
| `attribute_id` | AttributeId | 属性引用 | §一 |
| `base_value` | f32 | 基础面板值（战斗外固定） | §一.1 |
| `modifiers` | Vec<ModifierRef> | 当前生效的修正列表 | §一.2 |

### 1.3 AttributeModification（Runtime层）

战斗内动态属性修正，全部基于基础面板计算。

| 修正分类 | 类型 | 生效时机 | 叠加方式 | 来源 |
|----------|------|----------|----------|------|
| 无条件加成 | UnconditionalMod | 战斗开始即生效 | 加算 | §一.2 |
| 条件加成 | ConditionalMod | 满足条件时实时生效 | 加算 | §一.2 |

#### 无条件加成来源
- 光环效果
- 常驻Buff
- 装备常驻百分比词条

#### 条件加成来源
- 高地加成（高差≥2格：+15%攻击；低地：-10%攻击）
- 背击加成
- 濒死加成
- 血量阈值加成

---

## 二、属性转换机制

属性转换是中间层，在基础属性之后、最终属性之前结算。

| 字段名 | 类型 | 说明 | 来源 |
|--------|------|------|------|
| `source_attr` | AttributeId | 源属性（如防御、当前损失血量） | §一.3 |
| `target_attr` | AttributeId | 目标属性（如攻击） | §一.3 |
| `ratio` | f32 | 转换比例 | §一.3 |
| `condition` | Option<ConditionId> | 触发条件（可选） | §一.3 |

转换规则：
- 转换后的属性**可被后续百分比加成放大**
- 部分条件型加成**不参与转换**
- 归属：属性计算的中间层

典型配置：
```yaml
# 防御转攻击
attribute_conversion:
  source: phys_def
  target: phys_atk
  ratio: 0.5

# 损失血量转攻击
attribute_conversion:
  source: lost_hp
  target: phys_atk
  ratio: 0.3
```

---

## 三、装备词条对属性的影响

### 3.1 装备属性四层结构（§六）

| 层级 | 类型 | 生效时机 | 对基础面板的影响 | 来源 |
|------|------|----------|-----------------|------|
| 第一层：基础白值 | 固定数值 | 永久 | **计入基础面板**，是百分比加成基数 | §六.1 |
| 第二层：常驻百分比 | 百分比加成 | 战斗开始 | 不改基础面板，和其他百分比加算 | §六.1 |
| 第三层：条件百分比 | 条件百分比 | 条件满足时 | 不改基础面板，和常驻百分比加算 | §六.1 |
| 第四层：装备特效 | 事件触发 | 事件触发 | 不直接加属性，走事件触发 | §六.1 |

核心原则：
1. 所有属性加成最终汇入**统一属性管线**
2. 条件型词条**不改变基础面板**
3. 特效类**全部走事件触发**，不侵入属性计算核心逻辑

---

## 四、数值边界强制规则（补充11）

| 规则 | 约束 | 说明 |
|------|------|------|
| 取整规则 | 向下取整，最小为1 | 真实伤害可到0 |
| 属性下限 | 防御最低0，移动最低1 | 不会出现负数 |
| 暴击率上限 | 95% | 不存在100%必暴 |
| 闪避率上限 | 80% | 不存在100%必闪 |
| 命中率上限 | 100% | 不存在超100%命中 |
| 减伤上限 | 90% | 不会出现无敌减伤 |
| 易伤 | 无上限 | 但乘算递减 |
| 结算顺序 | 所有百分比计算完成后统一取整 | 避免分步取整偏差 |

---

## 五、Schema草案

```yaml
# attribute_config.ron
(
  attributes: [
    (id: "phys_atk", name_key: "attr.a_001.name", category: Core, min: 0.0, max: 99999.0),
    (id: "magic_atk", name_key: "attr.a_002.name", category: Core, min: 0.0, max: 99999.0),
    (id: "phys_def", name_key: "attr.a_003.name", category: Core, min: 0.0, max: 99999.0),
    (id: "magic_def", name_key: "attr.a_004.name", category: Core, min: 0.0, max: 99999.0),
    (id: "max_hp", name_key: "attr.a_005.name", category: Core, min: 1.0, max: 99999.0),
    (id: "crit_rate", name_key: "attr.a_006.name", category: Secondary, min: 0.0, max: 0.95),
    (id: "crit_dmg", name_key: "attr.a_007.name", category: Secondary, min: 1.5, max: 5.0),
    (id: "move_range", name_key: "attr.a_008.name", category: Secondary, min: 1.0, max: 99.0),
    (id: "atk_range", name_key: "attr.a_009.name", category: Secondary, min: 1.0, max: 99.0),
    (id: "hit_rate", name_key: "attr.a_010.name", category: Secondary, min: 0.0, max: 1.0),
    (id: "dodge_rate", name_key: "attr.a_011.name", category: Secondary, min: 0.0, max: 0.8),
  ],
)
```

### 对应 FTL 文件示例

```ftl
# zh-CN/attr.ftl
attr.a_001.name = 物理攻击
attr.a_002.name = 魔法攻击
attr.a_003.name = 物理防御
attr.a_004.name = 魔法防御
attr.a_005.name = 最大生命值
attr.a_006.name = 暴击率
attr.a_007.name = 暴击伤害
attr.a_008.name = 移动范围
attr.a_009.name = 攻击范围
attr.a_010.name = 命中率
attr.a_011.name = 闪避率
```

---

## 六、依赖关系

| 依赖领域 | 依赖方向 | 说明 |
|----------|----------|------|
| Modifier | Attribute ← Modifier | Modifier修改Attribute值 |
| Execution | Attribute → Execution | 属性值作为伤害/治疗计算输入 |
| Tag | Attribute ← Tag | Tag决定条件修正是否生效 |
| Stacking | Attribute ← Stacking | Stacking决定修正叠加方式 |

---

## 七、Data Laws合规

| Law | 状态 | 说明 |
|-----|------|------|
| 001 | ✅ | 基础属性(Definition)与战斗修正(Instance)严格分离 |
| 002 | ✅ | 属性转换规则在代码中，转换比例在配置中 |
| 007 | ✅ | Duration不属于Attribute，属于Effect |
| 010 | ✅ | 数值边界规则保证Replay确定性 |

---

## 八、代码实现映射

| 概念 | Rust 类型 | 源码路径 | 层级 |
|------|-----------|----------|------|
| AttributeDef | `AttributeDef { id, name_key, category, default, min, max }` | `src/core/attribute/def.rs` | Definition (RON) |
| AttributeDefinition | `AttributeDefinition { id: AttributeId, name_key, category, default, min, max }` | `src/core/attribute/def.rs` | Definition (Runtime) |
| AttributeRegistry | `AttributeRegistry { definitions: HashMap<AttributeId, AttributeDefinition> }` | `src/core/attribute/def.rs` | Definition (Resource) |
| CoreAttribute | `enum CoreAttribute { PhysAtk, MagicAtk, PhysDef, MagicDef, MaxHp }` | `src/core/attribute/mod.rs` | Definition (Enum) |
| SecondaryAttribute | `enum SecondaryAttribute { CritRate, CritDmg, MoveRange, AtkRange, HitRate, DodgeRate }` | `src/core/attribute/mod.rs` | Definition (Enum) |
| Attributes | `Attributes { core: EnumMap, secondary: EnumMap, current_hp, base_values, modifiers }` | `src/core/attribute/mod.rs` | Instance (Component) |
| ModifierOp | `enum ModifierOp { Add, Multiply }` (Multiply = value/10000) | `src/core/attribute/mod.rs` | Definition |
| ModifierSource | `ModifierSource(pub u64)` (bit-range: buff/trait/equip/consumable) | `src/core/attribute/mod.rs` | Runtime |
| AttributeConversion | `AttributeConversion { source, target, ratio, condition }` | `src/core/attribute/conversion.rs` | Definition |
| ConversionRegistry | `ConversionRegistry { conversions: HashMap<AttributeId, Vec<AttributeConversion>> }` | `src/core/attribute/conversion.rs` | Definition (Resource) |

**数值运算函数**（`src/core/attribute/ops.rs`）：
- `safe_add`, `safe_sub`, `safe_mul_percent`
- `apply_add_modifiers`, `apply_mul_modifiers`
- `clamp_attribute`, `clamp_all_attributes`

**RON 配置**：`content/attributes/attributes.ron`（单文件，11 个属性定义）