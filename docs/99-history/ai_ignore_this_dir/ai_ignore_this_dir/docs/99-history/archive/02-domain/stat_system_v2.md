---
id: history.archive.stat_system_v2
title: stat_system_v2
status: archived
owner: domain-designer
created: 2026-06-14
updated: 2026-06-14
superseded_by: ../../02-domain/character/character-rules.md
---

# Stat System 领域

Version: 2.0

## Purpose

Stat System 领域管理角色的所有数值计算，采用三层架构（核心属性 → 衍生属性 → 生命资源），通过统一修饰符管线实现所有数值变化。本领域为纯函数式计算，无状态机。

---

## Glossary

| 术语 | 定义 | 易混淆项 |
|------|------|----------|
| Core Stat | 8 维核心属性，有基础值，可通过修饰符修改 | ≠ Derived Stat：Core 有 base 值，Derived 没有 |
| Derived Stat | 13 维衍生属性，从 Core Stat 实时推导，不存储基础值 | ≠ Core Stat：Derived 无 base 值，set_base() 对其无效 |
| Vital Resource | 3 维生命资源，存储当前值 | ≠ Derived Stat：Vital 有 current 值而非 base 值 |
| Modifier | 属性修饰符，通过 Add 或 Multiply 修改属性值 | ≠ 直接属性修改：Modifier 走管线，直接修改绕过管线 |
| ModifierSource | 修饰符来源标识，u64 类型，按区间隔离 | ≠ GameplayTag：Source 标识"谁加的"，Tag 标识"有什么标签" |

---

## Responsibilities

### Owns

- 三层属性架构（Core / Derived / Vital）
- 衍生属性公式计算
- 修饰符管线的添加、移除、计算
- ModifierSource 区间管理
- 伤害/治疗的修饰计算和下限保护

### Does Not Own

- 属性变化后的 UI 刷新 → ui_rules
- Buff 的生命周期管理 → buff_rules
- 装备的穿脱触发 → equipment_rules
- 效果管线的生成和执行 → effect_pipeline
- Trait 效果分发 → trait_rules

---

## Invariants

### INV-STAT-01：衍生属性无基础值 🟥

Derived Stat 不存储 base 值，set_base() 对其无效。

违反：衍生属性与核心属性脱钩，公式一致性被破坏。

### INV-STAT-02：先加后乘 🟥

修饰符计算顺序固定：先 Add 后 Multiply。

最终值 = (base + Σ Add) × Π Multiply

违反：修饰符顺序影响结果。

### INV-STAT-03：乘法零值保护 🟥

乘法修饰符乘积为 0 时视为 1.0。

违反：单个 Multiply ×0 修饰符将属性归零。

### INV-STAT-04：伤害/治疗下限 🟥

伤害 ≥ 1，治疗 ≥ 0。

违反：伤害为 0 或负数，治疗为负数。

### INV-STAT-05：Source 区间隔离 🟥

Trait / Equipment / Buff 修饰符的 Source 值在各自区间内，互不重叠。

- Trait 区间：u64::MAX ~ u64::MAX - 999
- Equipment 区间：u64::MAX - 1000 ~ u64::MAX - 1999
- Buff 区间：1 ~ 999999

违反：按来源移除修饰符时误删其他来源的修饰符。

### INV-STAT-06：三类属性互斥 🟥

每个 AttributeKind 恰好属于 Core / Derived / Vital 三类之一。

违反：is_core() / is_derived() / is_vital() 判定矛盾。

### INV-STAT-07：禁止直接修改最终属性值 🟥

宪法：2.2.1

所有属性修改必须通过 Modifier 管线，禁止直接修改 get() 返回的最终值。

违反：属性变化无法追踪、无法回滚。

---

## State Machine

本领域无状态机，为纯函数式计算。

属性值 = f(base, modifiers)，任意时刻可重新计算。

---

## Business Rules

### BR-STAT-01：属性访问

- 通过 get(kind) 统一访问
- Core Stat 通过 set_base() 设置基础值
- Vital Resource 通过 set_vital() 设置当前值
- Derived Stat 通过公式实时计算
- Modifier 作用于 Core Stat 和 Derived Stat

### BR-STAT-02：修饰符管理

- 添加修饰符时指定 ModifierSource
- 按来源移除时使用 remove_modifiers_from(source)
- 减益判定：Add < 0 或 Multiply < 1.0
- 叠加多个修饰符允许
- remove_debuff_modifiers() 批量清理减益

### BR-STAT-03：fill_vital_resources

- 仅在单位生成时调用一次
- 将 HP/MP/Stamina 设为对应 Max 值
- 战斗中通过 set_vital() 修改当前值

---

## Pipelines

### 修饰符计算管线

基础值 → Add 修饰符求和 → Multiply 修饰符求积 → 下限保护

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 基础值 | AttributeKind | 基础值（含 Core 修饰符） | 禁止对 Derived Stat 返回 base 值 |
| Add 求和 | 所有 ModifierOp::Add | 加法总和 | 禁止跳过任何 Add 修饰符 |
| Multiply 求积 | 所有 ModifierOp::Multiply | 乘法乘积 | 乘积为 0 时视为 1.0（INV-STAT-03） |
| 下限保护 | 计算结果 | 最终值 | 伤害 max(1)，治疗 max(0)（INV-STAT-04） |

---

## Data Model

### Attributes（Instance）

角色的所有数值容器。

- base：Core Stat 基础值映射（8 维）
- current_hp / current_mp / current_stamina：Vital Resource 当前值
- base_attack_range：基础攻击范围
- modifiers：修饰符实例列表

### AttributeModifierDef（Definition）

修饰符的数据定义，用于 RON 配置。

- attribute：目标属性种类
- op：Add 或 Multiply
- value：修饰值

### AttributeModifierInstance（Instance）

运行时修饰符实例。

- def：AttributeModifierDef
- source：ModifierSource

### ModifierSource（值对象）

修饰符来源标识，u64 类型。

- Trait 区间：u64::MAX ~ u64::MAX - 999
- Equipment 区间：u64::MAX - 1000 ~ u64::MAX - 1999
- Buff 区间：1 ~ 999999

---

## Cross Domain Contracts

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 属性修饰符变化 | 通过 ModifierSource 添加/移除 | character |
| 伤害/治疗修饰结果 | 返回值 | battle |
| HP 变化 | 通过 set_vital() 修改 | battle |

---

## Change Rules

### 新增属性种类

- 允许：在 AttributeKind 枚举中新增变体 + 在分类中归类 + 新增衍生公式
- 禁止：修改三类互斥规则、修改先加后乘计算顺序、跳过下限保护
- 检查：新属性属于哪一类、衍生公式是否依赖正确的 Core Stat、ModifierRule 是否需要适配

### 新增修饰效果类型

- 允许：新增 ModifierEffect 变体 + 新增 ModifierCalculator 实现并注册
- 禁止：修改现有 Calculator 的计算逻辑、修改 ModifierRule 应用管线流程
- 检查：CalculatorRegistry 注册、applies_to 过滤、下限保护是否覆盖新类型

### 新增 ModifierRule

- 允许：新增 RON 配置文件
- 禁止：修改 ModifierRule 应用管线代码、修改 Calculator 分发逻辑
- 检查：source_tag 和 target_tag 是否在 GameplayTag 枚举中、效果数值平衡

---

## Architecture Violations

发现架构违规时统一输出：

```
ARCHITECTURE VIOLATION:
Rule: <RuleID>
Reason: <Why>
Fix: <How>
```

| RuleID | 违规行为 | Reason | Fix |
|--------|----------|--------|-----|
| INV-STAT-07 | 直接修改最终属性值 | 属性修改必须通过 Modifier 管线 | 通过 ModifierSource 添加修饰符 |
| INV-STAT-01 | 对衍生属性调用 set_base() | Derived Stat 由公式推导 | 修改 Core Stat 或 Modifier |
| INV-STAT-05 | Source 区间冲突 | 区间隔离保证精确移除 | 检查 Source 值是否在正确区间 |

---

## Test Requirements

宪法：13.0.1-13.0.3

- 单元测试：验证属性计算公式正确性
- 集成测试：验证完整修饰符管线
- Bug 修复必须先编写重现测试

排查顺序：
1. 属性分类是否正确（Core / Derived / Vital）
2. 衍生公式是否依赖正确的 Core Stat
3. 修饰符 Source 区间是否冲突
4. 先加后乘计算顺序
5. 下限保护是否生效
