# 属性系统领域规则 (Stat System Rules)

## 1. 领域概述

属性系统是 SRPG 的数值基础，负责角色的所有数值计算。采用**三层架构**：核心属性（基础值）、衍生属性（实时计算）、生命资源（当前值），通过统一的修饰符管线（Modifier Pipeline）实现所有数值变化。

### 核心原则

- **Primary Stat 与 Derived Stat 分离**：核心属性有基础值，衍生属性实时计算
- **Derived Stat 优先实时计算**：不存储衍生属性基础值
- **统一 Modifier 管线**：所有数值变化通过修饰符栈统一管理
- **属性公式集中管理**：所有衍生公式定义在 `Attributes` impl 中
- **配置型数据尽量保持不可变**：模板基础值不随战斗变化

---

## 2. 属性分类

### 2.1 三层架构

| 层级 | 类型 | 存储 | 示例 |
|------|------|------|------|
| 核心属性 | 8 维 | `base: HashMap<AttributeKind, f32>` | Might, Dexterity, Agility, Vitality, Intelligence, Willpower, Presence, Luck |
| 生命资源 | 3 维 | `current_hp/mp/stamina: f32` | HP, MP, Stamina |
| 衍生属性 | 13 维 | 实时计算，不存储 | MaxHp, Attack, Defense, MoveRange, Initiative 等 |

### 2.2 AttributeKind 完整枚举

```
核心属性（8维，有 base 值）：
  Might       力量 → 物理攻击力
  Dexterity   技巧 → 命中、暴击、远程
  Agility     敏捷 → 行动顺序、闪避、移动
  Vitality    体质 → 生命、物防
  Intelligence 智力 → 法术攻击、法力
  Willpower   意志 → 魔防、治疗、异常抵抗
  Presence    魅力 → 光环、召唤、指挥
  Luck        幸运 → 暴击、掉落、随机事件

生命资源（3维，存储当前值）：
  Hp          当前生命值
  Mp          当前法力值
  Stamina     当前耐力值

衍生属性（13维，实时计算）：
  MaxHp       最大生命值
  MaxMp       最大法力值
  MaxStamina  最大耐力值
  Attack      物理攻击力
  Defense     物理防御力
  MagicAttack 魔法攻击力
  MagicDefense 魔法防御力
  Accuracy    命中率
  Evasion     闪避率
  CritRate    暴击率
  MoveRange   移动力
  Initiative  行动速度
  AttackRange 攻击范围
```

### 2.3 分类判定

```rust
impl AttributeKind {
    pub fn is_core(&self) -> bool    // 8维核心属性
    pub fn is_vital(&self) -> bool   // 3维生命资源
    pub fn is_derived(&self) -> bool // 13维衍生属性
}
```

**规则**：三类互斥，每个 AttributeKind 恰好属于一个类别。

---

## 3. 衍生属性公式

所有衍生属性从核心属性实时推导，公式集中管理：

| 衍生属性 | 公式 | 依赖核心属性 |
|----------|------|-------------|
| MaxHp | `5 + Vitality * 5` | Vitality |
| MaxMp | `Intelligence * 5` | Intelligence |
| MaxStamina | `10 + (Vitality + Might) * 2` | Vitality, Might |
| Attack | `Might * 2` | Might |
| Defense | `Vitality` | Vitality |
| MagicAttack | `Intelligence * 2` | Intelligence |
| MagicDefense | `Willpower` | Willpower |
| Accuracy | `80 + Dexterity * 2` | Dexterity |
| Evasion | `Agility * 3` | Agility |
| CritRate | `5 + Luck` | Luck |
| MoveRange | `floor(Agility * 0.5) + 2` | Agility |
| Initiative | `Agility * 2 + Luck` | Agility, Luck |
| AttackRange | `base_attack_range` | 职业/装备决定 |

**规则**：
- 衍生属性不能通过 `set_base()` 设置基础值（会 warn 并忽略）
- 衍生属性可通过修饰符修改（如 Buff 增加 Attack +5）
- `AttackRange` 特殊：由 `base_attack_range` 字段决定（职业/装备），不依赖核心属性

---

## 4. 修饰符系统

### 4.1 修饰符操作

```rust
pub enum ModifierOp {
    Add,       // 加法：base + sum(Add modifiers)
    Multiply,  // 乘法：(base + add_sum) * product(Multiply modifiers)
}
```

**计算顺序**：先加后乘
```
最终值 = (base + Σ(Add modifiers)) × Π(Multiply modifiers)
```

**规则**：
- 乘法修饰符乘积为 0 时视为 1.0（防止意外归零）
- 修饰符可作用于核心属性和衍生属性
- 修饰符作用于核心属性时，会级联影响所有依赖该属性的衍生属性

### 4.2 ModifierSource — 来源标识

```rust
pub struct ModifierSource(pub u64);
```

| 区间 | 范围 | 用途 |
|------|------|------|
| Trait | `u64::MAX ~ u64::MAX - 999` | 种族/职业/天赋修饰 |
| Equipment | `u64::MAX - 1000 ~ u64::MAX - 1999` | 装备修饰 |
| Buff | `1 ~ 999999` | Buff/Debuff 修饰 |

**判定方法**：
- `is_trait()` — 是否来自 Trait
- `is_equipment()` — 是否来自装备
- `is_buff()` — 是否来自 Buff

**规则**：
- 区间隔离保证不同来源的修饰符不会冲突
- 可按来源精确移除修饰符（如脱装备时移除 Equipment 区间修饰符）

### 4.3 AttributeModifierDef vs AttributeModifierInstance

| 类型 | 用途 | 包含 source |
|------|------|-------------|
| `AttributeModifierDef` | 数据定义（RON/BuffData） | 否 |
| `AttributeModifierInstance` | 运行时实例 | 是 |

### 4.4 修饰符管理操作

| 操作 | 方法 | 说明 |
|------|------|------|
| 添加 | `add_modifier(instance)` | 添加单个修饰符 |
| 批量添加 | `add_modifiers_from_def(defs, source)` | 从定义列表批量添加 |
| 按来源移除 | `remove_modifiers_from(source)` | 移除指定来源的所有修饰符 |
| 移除 Trait 修饰符 | `remove_trait_modifiers()` | 移除 Trait 区间所有修饰符 |
| 移除 Equipment 修饰符 | `remove_equipment_modifiers()` | 移除 Equipment 区间所有修饰符 |
| 移除减益修饰符 | `remove_debuff_modifiers()` | 移除 Add<0 或 Multiply<1.0 的修饰符 |

**减益判定规则**：
- `ModifierOp::Add` 且 `value < 0.0` → 减益
- `ModifierOp::Multiply` 且 `value < 1.0` → 减益

---

## 5. Attributes 组件

### 5.1 数据结构

```rust
#[derive(Component, Reflect, Default, Debug, Clone)]
pub struct Attributes {
    pub base: HashMap<AttributeKind, f32>,  // 核心属性基础值
    pub current_hp: f32,                    // 当前 HP
    pub current_mp: f32,                    // 当前 MP
    pub current_stamina: f32,               // 当前 Stamina
    pub base_attack_range: u32,             // 基础攻击范围
    pub modifiers: Vec<AttributeModifierInstance>,  // 修饰符栈
}
```

### 5.2 统一访问接口

```rust
pub fn get(&self, kind: AttributeKind) -> f32
```

- 核心属性 → `core(kind)` = base + 修饰符
- 生命资源 → 直接返回当前值
- 衍生属性 → 从核心属性实时计算 + 修饰符

### 5.3 设置接口

| 方法 | 适用范围 | 说明 |
|------|----------|------|
| `set_base(kind, value)` | 核心属性 + 生命资源 | 设置基础值/当前值 |
| `set_vital(kind, value)` | 仅 HP/MP/Stamina | 语义更清晰：设置当前值 |
| `set_base_attack_range(range)` | AttackRange | 由职业/装备决定 |
| `fill_vital_resources()` | HP/MP/Stamina | 初始化为最大值 |

**规则**：
- `set_base()` 对衍生属性无效（warn 并忽略）
- `set_vital()` 对非生命资源无效（warn 并忽略）
- `fill_vital_resources()` 在单位生成时调用一次

---

## 6. 修饰规则系统（ModifierRule）

### 6.1 设计理念

修饰规则是**数据驱动的效果修饰系统**，替代效果管线中的硬编码 if-else。规则通过标签匹配（source_tag + target_tag）决定是否触发，通过计算器（Calculator）执行具体计算。

### 6.2 ModifierEffect — 修饰效果

| 效果类型 | 说明 | Calculator |
|----------|------|------------|
| `DamageMultiplier(f32)` | 伤害倍率 | `DamageMultiplierCalculator` |
| `DamageBonus(i32)` | 伤害固定加成 | `DamageBonusCalculator` |
| `HealMultiplier(f32)` | 治疗倍率 | `HealMultiplierCalculator` |
| `HealBonus(i32)` | 治疗固定加成 | `HealBonusCalculator` |

### 6.3 ModifierRule — 修饰规则

```rust
pub struct ModifierRule {
    pub name: String,           // 规则名称（如"火焰共鸣"）
    pub source_tag: GameplayTag, // 攻击方技能需要包含的标签
    pub target_tag: GameplayTag, // 目标需要包含的标签
    pub effect: ModifierEffect,  // 修饰效果
}
```

**匹配规则**：
- 攻击方标签包含 `source_tag` **且** 目标标签包含 `target_tag` 时触发
- 多条规则按顺序叠加
- 伤害最低为 1，治疗最低为 0

### 6.4 ModifierCalculator — 计算器 trait

```rust
pub trait ModifierCalculator: Send + Sync + 'static {
    fn type_name(&self) -> &'static str;
    fn applies_to_damage(&self) -> bool;
    fn applies_to_heal(&self) -> bool;
    fn calculate(&self, effect: &ModifierEffect, current: f32) -> f32;
}
```

**规则**：
- 新增效果类型只需实现 `ModifierCalculator` 并注册
- 通过 `type_name()` 分发，无需修改现有代码
- 内置 4 个计算器，支持自定义扩展

### 6.5 ModifierEntry — 修饰记录

```rust
pub struct ModifierEntry {
    pub before: i32,       // 修饰前值
    pub after: i32,        // 修饰后值
    pub rule_name: String, // 规则名称
}
```

用于 `apply_damage_modifiers_with_breakdown()` 和 `apply_heal_modifiers_with_breakdown()`，记录每步修饰详情，支持伤害明细展示。

### 6.6 应用流程

```
输入 amount + source_tags + target_tags
    ↓
遍历所有 ModifierRule
    ↓
匹配 source_tag ∈ source_tags && target_tag ∈ target_tags
    ↓
通过 CalculatorRegistry 查找对应计算器
    ↓
calculator.calculate(effect, current)
    ↓
记录 ModifierEntry（with_breakdown 模式）
    ↓
最终值 = max(1, result)（伤害）或 max(0, result)（治疗）
```

### 6.7 数据驱动配置

- RON 文件路径：`assets/rules/*.ron`（数组格式）
- 内置默认规则：`火焰共鸣`（FIRE × FIRE → DamageMultiplier(1.5)）

**RON 示例**：
```ron
[
    (
        name: "火焰共鸣",
        source_tag: FIRE,
        target_tag: FIRE,
        effect: DamageMultiplier(1.5),
    ),
    (
        name: "冰火相克",
        source_tag: FIRE,
        target_tag: ICE,
        effect: DamageMultiplier(0.5),
    ),
]
```

---

## 7. 数值示例

### 7.1 战士模板

| 核心属性 | 值 |
|----------|-----|
| Might | 5 |
| Dexterity | 3 |
| Agility | 6 |
| Vitality | 5 |
| Intelligence | 2 |
| Willpower | 3 |
| Presence | 2 |
| Luck | 2 |

| 衍生属性 | 计算 | 值 |
|----------|------|-----|
| MaxHp | 5 + 5×5 | 30 |
| MaxMp | 2×5 | 10 |
| Attack | 5×2 | 10 |
| Defense | 5 | 5 |
| MoveRange | floor(6×0.5)+2 | 5 |
| Initiative | 6×2+2 | 14 |
| AttackRange | base | 1 |

### 7.2 修饰符叠加示例

战士基础 Attack = 10，添加：
- Buff Add +5 → `(10 + 5) × 1.0 = 15`
- Buff Multiply ×1.5 → `(10 + 5) × 1.5 = 22.5`
- Debuff Add -3 → `(10 + 5 - 3) × 1.5 = 18.0`

---

## 8. 关键约束

1. **衍生属性不可设置基础值**：`set_base()` 对衍生属性无效，保证公式一致性
2. **先加后乘**：修饰符计算顺序固定，Add 先于 Multiply
3. **乘法零值保护**：乘法修饰符乘积为 0 时视为 1.0
4. **伤害最低为 1**：`apply_damage_modifiers` 保证 `result.max(1.0)`
5. **治疗最低为 0**：`apply_heal_modifiers` 保证 `result.max(0.0)`
6. **Source 区间隔离**：Trait/Equipment/Buff 各有独立区间，支持精确增减
7. **减益判定统一**：Add<0 或 Multiply<1.0 为减益，`remove_debuff_modifiers()` 统一清理
8. **fill_vital_resources 仅初始化时调用**：战斗中通过 `set_vital()` 修改当前值
