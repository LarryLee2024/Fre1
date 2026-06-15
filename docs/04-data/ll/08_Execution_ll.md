# Execution 领域 — 铃兰之剑数据提取

> 领域：Execution | 来源：78铃兰.md §二、补充6、补充11 | 数据层：Runtime

---

## 一、数据实体清单

### 1.1 ExecutionContext（Runtime层）

伤害/治疗/护盾计算的运行时上下文，不持久化。

| 字段名 | 类型 | 说明 | 来源 |
|--------|------|------|------|
| `attacker` | Entity | 攻击者 | §二.2 |
| `defender` | Entity | 防御者 | §二.2 |
| `skill_multiplier` | f32 | 技能倍率 | §二.2 |
| `damage_type` | DamageTypeTag | 伤害类型 | §二.1 |
| `is_crit` | bool | 是否暴击 | §二.2 |
| `is_backstab` | bool | 是否背击 | §二.2 |
| `height_advantage` | Option<HeightMod> | 高低地修正 | §二.2 |

---

## 二、伤害类型分类

### 2.1 DamageTypeTag 枚举

| 类型 | 防御减免 | 护盾交互 | 说明 | 来源 |
|------|----------|----------|------|------|
| `Physical` | 受物防减免 | 物理护盾可挡 | 物理攻击 | §二.1 |
| `Magical` | 受魔防减免 | 魔法护盾可挡 | 魔法攻击 | §二.1 |
| `Pierce` | 无视防御 | 部分护盾可挡 | 穿透攻击 | §二.1 |
| `True` | 无视防御无视护盾 | 不可挡 | 真实伤害 | §二.1 |

---

## 三、四段伤害公式（核心算式）

### 3.1 第一段：攻击值结算

```
最终攻击值 = (基础攻击 × (1 + Σ攻击百分比加算) + Σ固定攻击加成 + 属性转换攻击) × 属性克制倍率 × 高低地修正
```

| 输入 | 类型 | 叠加方式 | 来源 |
|------|------|----------|------|
| 基础攻击 | f32 | - | §二.2 |
| 攻击百分比 | AddPercent | 加算 | §二.2 |
| 固定攻击加成 | Add | 加算 | §二.2 |
| 属性转换攻击 | Add | 加算 | §一.3 |
| 属性克制倍率 | MulPercent | 乘算 | §二.2 |
| 高低地修正 | MulPercent | 乘算 | §二.2 |

高低地规则：
- 高差≥2格：+15%攻击
- 低地：-10%攻击

### 3.2 第二段：防御值结算

```
有效防御 = 目标防御 × Π(1 - 降防效果) × Π(1 - 无视防御效果)
```

| 输入 | 类型 | 叠加方式 | 来源 |
|------|------|----------|------|
| 目标防御 | f32 | - | §二.2 |
| 降防效果 | MulPercent | 乘算 | §二.2 |
| 无视防御 | MulPercent | 乘算 | §二.2 |

> 降防和无视防御都是乘算，效果递减但不会出现负防御。

### 3.3 第三段：基础伤害计算

```
基础伤害 = (最终攻击值 - 有效防御) × 技能倍率
```

| 输入 | 类型 | 说明 | 来源 |
|------|------|------|------|
| 最终攻击值 | f32 | 第一段输出 | §二.2 |
| 有效防御 | f32 | 第二段输出 | §二.2 |
| 技能倍率 | f32 | 技能配置值 | §二.2 |

### 3.4 第四段：最终伤害修正

```
最终伤害 = 基础伤害 × Π(1 + 增伤效果) × Π(1 + 易伤效果) × Π(1 - 减伤效果) × 暴击倍率
```

| 输入 | 类型 | 叠加方式 | 来源 |
|------|------|----------|------|
| 基础伤害 | f32 | 第三段输出 | §二.2 |
| 增伤效果 | AddPercent | 加算 | §二.2 |
| 易伤效果 | MulPercent | 乘算 | §二.2 |
| 减伤效果 | MulPercent | 乘算 | §二.2 |
| 暴击倍率 | f32 | 暴击时=crit_dmg，否则=1.0 | §二.2 |

---

## 四、治疗计算公式

### 4.1 基础治疗量

```
基础治疗量 = 攻击 × 技能倍率 / 治疗强度
```

| 输入 | 类型 | 说明 | 来源 |
|------|------|------|------|
| 攻击 | f32 | 攻击者攻击值 | 补充6 |
| 技能倍率 | f32 | 治疗技能倍率 | 补充6 |
| 治疗强度 | f32 | 治疗系数 | 补充6 |

### 4.2 治疗修正

```
最终治疗量 = 基础治疗量 × (1 + 受治疗提升) × 暴击倍率 × AOE衰减
```

| 修正项 | 类型 | 说明 | 来源 |
|--------|------|------|------|
| 受治疗提升 | AddPercent | 目标身上的效果 | 补充6 |
| 治疗暴击 | MulPercent | 可暴击治疗单独判定 | 补充6 |
| AOE衰减 | MulPercent | 多目标时按目标数递减 | 补充6 |
| 过量治疗转护盾 | Add | 转化率固定 | 补充6 |

---

## 五、护盾计算

### 5.1 护盾吸收顺序

```
伤害 → 先扣通用护盾 → 再扣物理/魔法专属护盾 → 最后扣HP
```

### 5.2 护盾叠加规则

| 护盾类型 | 叠加规则 | 来源 |
|----------|----------|------|
| 同类护盾 | 取最大值+刷新时长 | 补充6 |
| 不同类护盾 | 独立共存 | 补充6 |
| 伤害吸收护盾 | 只吸收特定伤害类型，其余穿透 | 补充6 |
| 持续回复护盾 | 每回合回复固定值，有上限 | 补充6 |

---

## 六、数值边界强制规则（补充11）

| 规则 | 约束 | 公式位置 |
|------|------|----------|
| 取整 | 向下取整，最小为1 | 最终结果 |
| 防御下限 | ≥0 | 第二段 |
| 移动下限 | ≥1 | - |
| 暴击率上限 | ≤95% | 第四段暴击判定 |
| 闪避率上限 | ≤80% | 第四段命中判定 |
| 减伤上限 | ≤90% | 第四段 |
| 结算顺序 | 所有百分比计算完成后统一取整 | 全流程 |

---

## 七、Schema草案

```yaml
# execution_config.ron
(
  damage_formula: (
    stages: [
      # 第一段：攻击结算
      (name: "attack_resolve",
       inputs: ["base_atk", "atk_pct_sum", "atk_flat_sum", "convert_atk"],
       formula: "(base_atk * (1 + atk_pct_sum) + atk_flat_sum + convert_atk) * element_mod * height_mod"),
      # 第二段：防御结算
      (name: "defense_resolve",
       inputs: ["target_def", "def_break_product", "armor_pen_product"],
       formula: "target_def * def_break_product * armor_pen_product"),
      # 第三段：基础伤害
      (name: "base_damage",
       inputs: ["final_atk", "effective_def", "skill_multiplier"],
       formula: "(final_atk - effective_def) * skill_multiplier"),
      # 第四段：最终修正
      (name: "final_damage",
       inputs: ["base_dmg", "dmg_up_sum", "vuln_product", "dmg_red_product", "crit_multiplier"],
       formula: "base_dmg * (1 + dmg_up_sum) * vuln_product * dmg_red_product * crit_multiplier"),
    ],
    boundaries: (
      min_damage: 1,
      min_true_damage: 0,
      max_damage_reduction: 0.9,
      max_crit_rate: 0.95,
      max_dodge_rate: 0.8,
      max_hit_rate: 1.0,
      min_defense: 0.0,
      min_move_range: 1,
      rounding: Floor,
    ),
  ),
)
```

---

## 八、依赖关系

| 依赖领域 | 依赖方向 | 说明 |
|----------|----------|------|
| Attribute | Execution ← Attribute | 属性值作为计算输入 |
| Modifier | Execution ← Modifier | 修正值影响计算结果 |
| Tag | Execution ← Tag | 伤害类型Tag决定减免规则 |
| Effect | Execution ← Effect | Effect调用Execution计算 |

---

## 九、Data Laws合规

| Law | 状态 | 说明 |
|-----|------|------|
| 002 | ✅ | 公式在代码中，数值在配置中 |
| 005 | ✅ | 所有业务结果通过Effect→Execution执行 |
| 010 | ✅ | 四段式固定管线+数值边界保证Replay确定性 |