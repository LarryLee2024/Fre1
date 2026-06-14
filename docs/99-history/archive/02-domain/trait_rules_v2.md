---
id: history.archive.trait_rules_v2
title: trait_rules_v2
status: archived
owner: domain-designer
created: 2026-06-14
updated: 2026-06-14
superseded_by: ../../02-domain/character/character-rules.md
---

# Trait 领域

Version: 2.0

## Purpose

Trait 领域管理角色能力的统一扩展机制。种族、职业、天赋、装备、Buff 均通过 Trait + Modifier 管线影响角色。Trait 表示能力，不表示分类；组合优于继承。

---

## Glossary

| 术语 | 定义 | 易混淆项 |
|------|------|----------|
| TraitData | Trait 的运行时数据，包含触发时机和效果列表 | ≠ TraitCollection：Data 是定义，Collection 是实例集合 |
| TraitTrigger | Trait 的触发时机，决定"什么时候生效" | ≠ TraitSource：Trigger 标记"何时触发"，Source 标记"从哪来" |
| TraitEffect | Trait 的效果类型，决定"做什么" | ≠ TraitData：Effect 是组成部分，Data 包含多个 Effect |
| TraitEffectHandler | 效果处理器，执行具体效果逻辑 | ≠ TraitEffect：Handler 是执行者，Effect 是配置 |
| TraitSource | Trait 的来源标记，区分内在来源和装备来源 | ≠ TraitTrigger：Source 标记"从哪来"，Trigger 标记"何时触发" |
| TraitCollection | 单位拥有的 Trait 条目集合 | ≠ TraitRegistry：Collection 是实例，Registry 是定义注册表 |

---

## Responsibilities

### Owns

- TraitData 定义和注册表
- TraitEffectHandler 注册表
- TraitTrigger 和 TraitEffect 的对应关系
- TraitCollection 的增删管理
- apply_passive_traits 被动效果应用
- Trait 重建（rebuild_trait_effects）

### Does Not Own

- 属性计算和修饰符管线 → stat_system
- 装备穿脱触发 → equipment_rules
- Buff 的生命周期 → buff_rules
- 战斗管线中的 Trait 触发调用 → battle_rules
- UI 展示 → ui_rules

---

## Invariants

### INV-TRT-01：Passive 效果仅 GrantTag 和 ModifyAttribute 🟥

Passive 触发的 Trait 只产生 GrantTag 和 ModifyAttribute 效果。

违反：Passive Trait 产生 ApplyBuff 效果，无触发时机，永远不会执行。

### INV-TRT-02：触发型效果仅 ApplyBuff 🟥

OnAttack/OnHit/OnKill/OnTurnStart/OnTurnEnd 触发的 Trait 只产生 ApplyBuff 效果。

违反：触发型 Trait 产生 GrantTag/ModifyAttribute，标签和修饰符在触发时临时添加后无法正确移除。

### INV-TRT-03：Handler 覆盖所有效果类型 🟥

TraitEffect 的每个变体都有对应的 TraitEffectHandler 注册。

违反：新增效果类型但未注册 Handler，apply_passive_traits 跳过该效果。

### INV-TRT-04：修饰符 Source 区间隔离 🟥

Trait 修饰符的 ModifierSource 在 Trait 区间内（u64::MAX ~ u64::MAX - 999）。

违反：Trait 修饰符与 Buff/Equipment 修饰符冲突，无法按来源精确移除。

### INV-TRT-05：修饰符必须通过 Modifier 管线 🟥

宪法：2.2.1

Trait 的 ModifyAttribute 效果必须通过 ModifierSource::trait_source 添加到 Attributes，禁止直接修改属性值。

违反：Trait 直接修改 HP、ATK 等最终属性值。

### INV-TRT-06：Handler 分发 🟥

宪法：6.0.2

效果通过 type_name() 查找 Handler 分发，禁止 match 分发效果类型。

违反：新增效果类型需要修改分发代码。

### INV-TRT-07：统一扩展机制 🟥

宪法：1.1.6

所有能力来源（种族/职业/装备/Buff）走同一 Trait 管线，禁止为每种来源写独立逻辑。

违反：种族/职业/装备各有独立逻辑，维护成本指数增长。

---

## State Machine

本领域无状态机，为纯函数式计算。

Trait 生命周期由外部驱动：
- 生成时：apply_passive_traits 应用被动效果
- 穿脱时：rebuild_trait_effects 重建
- 战斗时：trigger_traits 触发效果

---

## Business Rules

### BR-TRT-01：效果与触发器对应

- Passive → GrantTag + ModifyAttribute
- 触发型（OnAttack/OnHit/OnKill/OnTurnStart/OnTurnEnd）→ ApplyBuff

### BR-TRT-02：Handler 分发

- 通过 type_name() 查找 Handler
- 新增效果类型只需实现 Handler 并注册
- 默认注册三个内置 Handler

### BR-TRT-03：来源追踪

- Intrinsic 标记种族/职业/天赋
- Equipment { slot } 标记装备来源
- 脱卸装备时按 source 精确移除

### BR-TRT-04：Trait 重建

- 清除所有 Trait 来源修饰符
- 清除 Trait 授予的标签
- 重新应用所有 Passive Trait
- 重建 GameplayTags
- 禁止增量更新，必须完全重建

---

## Pipelines

### 被动 Trait 应用管线

遍历 entries → 跳过非 Passive → Handler 收集标签 → Handler 收集修饰符 → 分配 Source → 返回

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 遍历 entries | TraitCollection.entries | Passive 触发的 entries | 禁止处理非 Passive 触发 |
| Handler 收集 | TraitEffect + HandlerRegistry | 标签集合 + 修饰符列表 | 禁止跳过任何效果 |
| 分配 Source | 修饰符列表 + index | 带 Source 的修饰符实例 | Source 区间不与其他来源冲突（INV-TRT-04） |

### Trait 重建管线

清除修饰符 → 清除标签 → 重新应用 Passive → 重建 GameplayTags

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 清除 | Attributes + PersistentTags | 清除后的状态 | 禁止清除 from_equipment |
| 重新应用 | TraitCollection + Registry | 新标签 + 新修饰符 | 禁止跳过任何 Passive Trait |
| 重建标签 | PersistentTags | 更新后的 GameplayTags | 禁止包含 Buff 层标签 |

---

## Data Model

### TraitData（Definition）

Trait 的运行时数据。

- id / name / description：标识和展示
- trigger：触发时机
- effects：效果列表
- 配置来源：RON（assets/traits/）

### TraitEffect（值对象）

Trait 的效果类型。

- GrantTag(GameplayTag)：授予标签
- ModifyAttribute(AttributeModifierDef)：属性修饰
- ApplyBuff { buff_id, duration }：触发时施加 Buff

### TraitEffectHandler（Trait）

效果处理器接口。

- type_name()：分发键
- granted_tags()：提取授予的标签
- attribute_modifiers()：提取属性修饰

### TraitCollection（Instance Component）

单位 Trait 条目集合。

- entries：TraitEntry 列表
- add_entry 记录来源
- remove_by_source 精确清理

### TraitSource（值对象）

Trait 来源标记。

- Intrinsic：内在来源
- Equipment { slot }：装备来源

---

## Cross Domain Contracts

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 被动效果应用 | 返回值（标签+修饰符） | character |
| Trait 重建 | 直接函数调用 | equipment |
| 触发型效果 | 推入 EffectQueue | battle |

---

## Change Rules

### 新增 Trait 效果类型

- 允许：新增 TraitEffect 变体 + 新增 TraitEffectHandler 实现并注册
- 禁止：修改 TraitData 方法、修改 apply_passive_traits 流程、修改 rebuild_trait_effects 流程
- 检查：TraitEffectHandlerRegistry 注册、type_name() 是否与变体名一致、与 TraitTrigger 的对应关系

### 新增 Trait 触发时机

- 允许：新增 TraitTrigger 变体 + 在对应阶段添加触发调用
- 禁止：修改现有触发器的效果对应关系、修改 Handler 分发逻辑
- 检查：触发位置、EffectQueue 是否可用、触发目标

### 新增 Trait

- 允许：新增 RON 配置文件
- 禁止：修改 TraitData 结构、修改 TraitRegistry 加载流程
- 检查：TraitRegistry 注册、效果类型是否有对应 Handler、标签是否在 GameplayTag 枚举中

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
| INV-TRT-05 | Trait 直接修改属性值 | 属性修改必须通过 Modifier 管线 | 通过 ModifierSource::trait_source 添加 |
| INV-TRT-06 | match 分发效果类型 | 应通过 type_name() 分发 | 改为注册表查找 |
| INV-TRT-07 | 为某种来源写独立逻辑 | 统一扩展机制 | 走同一 Trait 管线 |

---

## Test Requirements

宪法：13.0.1-13.0.3

- 单元测试：验证 Handler 分发和效果收集
- 集成测试：验证 Trait 重建流程
- Bug 修复必须先编写重现测试

排查顺序：
1. TraitTrigger 与 TraitEffect 对应关系
2. Handler 是否正确注册
3. ModifierSource 区间是否冲突
4. Trait 重建是否完整
5. 来源追踪是否正确
