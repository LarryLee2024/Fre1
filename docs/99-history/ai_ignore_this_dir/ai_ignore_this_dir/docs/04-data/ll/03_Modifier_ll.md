# Modifier 领域 — 铃兰之剑数据提取

> 领域：Modifier | 来源：78铃兰.md §一、§二、§五、§六 | 数据层：Definition + Instance

---

## 一、数据实体清单

### 1.1 ModifierDefinition（Definition层）

| 字段名 | 类型 | 约束 | 说明 | 来源 |
|--------|------|------|------|------|
| `id` | ModifierId | PK | 修正器唯一标识 | §五 |
| `target_attr` | AttributeId | FK | 目标属性 | §五 |
| `operation` | ModifierOp | - | 修正操作类型 | §五 |
| `value` | f32 | - | 修正值 | §五 |
| `stacking_rule` | StackingId | FK | 堆叠策略引用 | §五 |
| `source_type` | Enum | buff/equipment/trait/terrain | 来源分类 | §六 |

### 1.2 ModifierInstance（Instance层）

| 字段名 | 类型 | 说明 | 来源 |
|--------|------|------|------|
| `entity` | Entity | 挂载目标实体 | - |
| `modifier_id` | ModifierId | 引用ModifierDefinition | - |
| `source` | ModifierSource | 来源（Buff实例/装备/天赋） | - |
| `remaining_duration` | Option<u32> | 剩余持续回合（None=永久） | §四.2 |
| `current_stack` | u32 | 当前层数 | §四.3 |

---

## 二、修正操作类型

### 2.1 ModifierOp 枚举

| 操作 | 公式 | 叠加方式 | 适用属性 | 来源 |
|------|------|----------|----------|------|
| `Add` | base + value | - | 固定值加成 | §二.2 |
| `AddPercent` | base × (1 + Σvalue) | 加算 | 攻击%、暴击率、增伤% | §五.2 |
| `MulPercent` | base × Π(1 - value) | 乘算 | 降防、无视防御、减伤、易伤 | §五.2 |

### 2.2 各区间的叠加方式

| 区间 | ModifierOp | 叠加方式 | 公式 | 来源 |
|------|-----------|----------|------|------|
| 攻击百分比 | AddPercent | 加算 | Σ(all_pct) | §二.2第一段 |
| 固定攻击加成 | Add | 加算 | Σ(all_flat) | §二.2第一段 |
| 降防效果 | MulPercent | 乘算 | Π(1 - each) | §二.2第二段 |
| 无视防御 | MulPercent | 乘算 | Π(1 - each) | §二.2第二段 |
| 增伤效果 | AddPercent | 加算 | Σ(all_pct) | §二.2第四段 |
| 易伤效果 | MulPercent | 乘算 | Π(1 + each) | §二.2第四段 |
| 减伤效果 | MulPercent | 乘算 | Π(1 - each) | §二.2第四段 |
| 暴击率 | AddPercent | 加算 | Σ(all_pct) | §五.2 |
| 暴击伤害 | AddPercent | 加算 | Σ(all_pct) | §五.2 |

---

## 三、Modifier来源分类

| 来源类型 | 说明 | 生命周期 | 来源 |
|----------|------|----------|------|
| Buff | Buff附加的属性修正 | 随Buff存在 | §四 |
| Equipment | 装备词条附加的属性修正 | 装备穿戴期间 | §六 |
| Trait | 天赋/被动附加的属性修正 | 永久 | §三.1 |
| Terrain | 地形附加的属性修正 | 站在对应地形时 | §一.2 |

---

## 四、装备词条Modifier映射

| 装备层级 | ModifierOp | 说明 | 来源 |
|----------|-----------|------|------|
| 第一层：基础白值 | Add | 计入基础面板 | §六.1 |
| 第二层：常驻百分比 | AddPercent | 基于基础面板，和其他百分比加算 | §六.1 |
| 第三层：条件百分比 | AddPercent（条件型） | 满足条件时生效，和常驻百分比加算 | §六.1 |
| 第四层：装备特效 | 不产生Modifier | 走事件触发，不直接加属性 | §六.1 |

---

## 五、Schema草案

```yaml
# modifier_config.ron
(
  modifiers: [
    # 攻击提升20%
    (id: "atk_up_20", target_attr: "phys_atk", operation: AddPercent, value: 0.2,
     stacking_rule: "additive_same_name_max"),
    # 降防40%
    (id: "def_down_40", target_attr: "phys_def", operation: MulPercent, value: 0.4,
     stacking_rule: "multiplicative"),
    # 无视防御40%
    (id: "armor_pen_40", target_attr: "phys_def", operation: MulPercent, value: 0.4,
     stacking_rule: "multiplicative"),
    # 增伤15%
    (id: "dmg_up_15", target_attr: "dmg_multiplier", operation: AddPercent, value: 0.15,
     stacking_rule: "additive"),
  ],
)
```

---

## 六、依赖关系

| 依赖领域 | 依赖方向 | 说明 |
|----------|----------|------|
| Attribute | Modifier → Attribute | Modifier修改Attribute值 |
| Stacking | Modifier ← Stacking | Stacking决定Modifier叠加方式 |
| Effect | Modifier ← Effect | Effect产生Modifier |
| Execution | Modifier → Execution | Execution读取Modifier计算最终值 |

---

## 七、Data Laws合规

| Law | 状态 | 说明 |
|-----|------|------|
| 005 | ✅ | Modifier由Effect产生，不被Ability/Trigger直接调用 |
| 006 | ✅ | Modifier只改变数值，无事件处理逻辑 |
| 008 | ✅ | Modifier叠加行为由Stacking策略决定，不自行处理 |

---

## 八、代码实现映射

| 概念 | Rust 类型 | 源码路径 | 层级 |
|------|-----------|----------|------|
| ModifierRuleDef | `ModifierRuleDef { version, name, source_tag: String, target_tag: String, effect: ModifierEffectDef }` | `src/core/modifier/types.rs` | Definition (RON) |
| ModifierRule | `ModifierRule { name, source_tag: GameplayTag, target_tag: GameplayTag, effect: ModifierEffect }` | `src/core/modifier/types.rs` | Definition (Runtime) |
| ModifierRuleRegistry | `ModifierRuleRegistry { rules: Vec<ModifierRule>, calculators }` | `src/core/modifier/types.rs` | Definition (Resource) |
| ModifierEffect | `enum ModifierEffect { DamageMultiplier(f32), DamageBonus(i32), HealMultiplier(f32), HealBonus(i32) }` | `src/core/modifier/types.rs` | Definition |
| ModifierEntry | `ModifierEntry { before: i32, after: i32, rule_name: String }` — 审计追踪 | `src/core/modifier/types.rs` | Runtime |

**核心方法**（`src/core/modifier/types.rs`）：
- `apply_damage_modifiers()` — 伤害修饰管线
- `apply_heal_modifiers()` — 治疗修饰管线
- `with_breakdown` 变体 — 带修饰记录版本

**RON 配置**：`content/modifiers/element_interactions.ron`（数组格式，元素交互规则）