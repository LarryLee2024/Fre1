# 修饰规则领域文档 (Modifier Rules)

## 1. 领域概述

修饰规则系统是数据驱动的效果修饰管线，替代 modify_effects 中的硬编码 if-else。通过标签匹配 + 计算器 trait 分发，实现伤害/治疗的多步修饰链。

### 核心原则

- **Rule / Content 分离**：代码负责修饰规则，配置负责修饰内容
- **Trait 替代 match**：ModifierCalculator trait 分发替代 enum+match
- **标签驱动匹配**：source_tag + target_tag 双标签匹配
- **修饰链可追踪**：ModifierEntry 记录每步 before/after

---

## 2. ModifierEffect — 修饰效果类型

```rust
pub enum ModifierEffect {
    DamageMultiplier(f32),  // 伤害倍率
    DamageBonus(i32),       // 伤害固定加成
    HealMultiplier(f32),    // 治疗倍率
    HealBonus(i32),         // 治疗固定加成
}
```

| 类型 | 说明 | 适用 |
|------|------|------|
| `DamageMultiplier(f32)` | 伤害 × 倍率 | 伤害 |
| `DamageBonus(i32)` | 伤害 + 固定值 | 伤害 |
| `HealMultiplier(f32)` | 治疗 × 倍率 | 治疗 |
| `HealBonus(i32)` | 治疗 + 固定值 | 治疗 |

---

## 3. ModifierCalculator — 计算器 Trait

```rust
pub trait ModifierCalculator: Send + Sync + 'static {
    fn type_name(&self) -> &'static str;
    fn applies_to_damage(&self) -> bool;
    fn applies_to_heal(&self) -> bool;
    fn calculate(&self, effect: &ModifierEffect, current: f32) -> f32;
}
```

### 3.1 内置计算器

| 计算器 | type_name | 伤害 | 治疗 | 公式 |
|--------|-----------|------|------|------|
| `DamageMultiplierCalculator` | "DamageMultiplier" | ✓ | ✗ | `current * mul` |
| `DamageBonusCalculator` | "DamageBonus" | ✓ | ✗ | `current + bonus` |
| `HealMultiplierCalculator` | "HealMultiplier" | ✗ | ✓ | `current * mul` |
| `HealBonusCalculator` | "HealBonus" | ✗ | ✓ | `current + bonus` |

---

## 4. ModifierRule — 修饰规则

### 4.1 运行时结构

```rust
pub struct ModifierRule {
    pub name: String,
    pub source_tag: GameplayTag,  // 攻击方技能需包含的标签
    pub target_tag: GameplayTag,  // 目标需包含的标签
    pub effect: ModifierEffect,
}
```

### 4.2 RON 定义结构

```rust
pub struct ModifierRuleDef {
    pub version: u32,
    pub name: String,
    pub source_tag: TagName,
    pub target_tag: TagName,
    pub effect: ModifierEffectDef,
}
```

### 4.3 匹配规则

规则匹配需同时满足：
1. **source_tag**：攻击方技能标签包含规则的 source_tag
2. **target_tag**：目标标签集合包含规则的 target_tag

两个条件都满足时，规则生效。

---

## 5. ModifierEntry — 修饰记录

```rust
pub struct ModifierEntry {
    pub before: i32,       // 修饰前值
    pub after: i32,        // 修饰后值
    pub rule_name: String, // 规则名称
}
```

用于战斗记录（BattleRecord）中追踪每步修饰详情。

---

## 6. ModifierCalculatorRegistry — 计算器注册表

```rust
#[derive(Resource)]
pub struct ModifierCalculatorRegistry {
    calculators: Vec<Box<dyn ModifierCalculator>>,
}
```

| 方法 | 说明 |
|------|------|
| `with_defaults()` | 创建包含所有内置计算器的注册表 |
| `register(calculator)` | 注册自定义计算器 |
| `find_damage_calculator(effect)` | 查找伤害计算器 |
| `find_heal_calculator(effect)` | 查找治疗计算器 |

**查找逻辑**：按 `type_name` 匹配 + `applies_to_damage/heal` 过滤。

---

## 7. ModifierRuleRegistry — 规则注册表

```rust
#[derive(Resource)]
pub struct ModifierRuleRegistry {
    pub rules: Vec<ModifierRule>,
    calculators: ModifierCalculatorRegistry,
}
```

### 7.1 核心方法

| 方法 | 说明 |
|------|------|
| `apply_damage_modifiers(amount, source_tags, target_tags)` | 应用伤害修饰 |
| `apply_heal_modifiers(amount, source_tags, target_tags)` | 应用治疗修饰 |
| `apply_damage_modifiers_with_breakdown(...)` | 伤害修饰 + 记录每步 |
| `apply_heal_modifiers_with_breakdown(...)` | 治疗修饰 + 记录每步 |
| `register_calculator(calculator)` | 注册自定义计算器 |

### 7.2 修饰管线流程

```
1. 遍历所有 ModifierRule
2. 检查 source_tag 匹配
3. 检查 target_tag 匹配
4. 通过 calculator registry 查找计算器
5. 计算修饰后的值
6. 继续下一条规则（链式修饰）
7. 最终值：伤害 ≥ 1，治疗 ≥ 0
```

### 7.3 多规则叠加示例

```
基础伤害: 10
规则1: DamageMultiplier(1.5) → 10 * 1.5 = 15
规则2: DamageBonus(3)       → 15 + 3 = 18
最终伤害: 18
```

---

## 8. RON 配置格式

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
    (
        name: "毒伤加深",
        source_tag: POISON,
        target_tag: POISON,
        effect: DamageBonus(5),
    ),
]
```

**数据加载**：`assets/rules/` 目录，通过 `RegistryLoader` 加载 RON 文件（Vec 格式）。

---

## 9. 与 Effect Pipeline 的集成

修饰规则在 Effect Pipeline 的 **Modify 阶段** 应用：

```
Generate → Modify（修饰规则在此） → Execute
```

`modify_effects` 系统调用 `ModifierRuleRegistry::apply_damage_modifiers_with_breakdown()` 或 `apply_heal_modifiers_with_breakdown()`，将修饰记录写入 BattleRecord。

---

## 10. 关键约束

1. **双标签匹配**：source_tag 和 target_tag 必须同时满足
2. **链式修饰**：规则按注册顺序依次应用，结果累积
3. **伤害下限为 1**：`result.max(1.0) as i32`
4. **治疗下限为 0**：`result.max(0.0) as i32`
5. **Calculator trait 替代 match**：新增效果类型只需实现 trait 并注册
6. **type_name 与 enum variant 对应**：查找计算器时按 type_name 匹配
7. **ModifierEntry 可追踪**：每步修饰记录 before/after/rule_name
8. **默认规则兜底**：无规则时注册 "火焰共鸣" 作为默认
9. **配置驱动**：新增修饰规则修改 RON，不修改代码
10. **version 字段预留**：用于未来存档兼容性检查
