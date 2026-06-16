---
id: 02-domain.attribute-modifier.attribute-modifier-rules
title: Attribute Modifier Rules
status: draft
owner: domain-designer
created: 2026-06-14
updated: 2026-06-14
tags:
  - domain
  - attribute-modifier
---

# 属性修饰管线领域

Version: 1.1
Status: Proposed
Changelog: v1.1 — 新增"与 Effect 领域的边界"章节，明确 Modify 阶段内部逻辑的所有权边界；新增交叉引用

属性修饰管线领域管理战斗中属性的计算、修饰器的堆叠和效果管线的执行。

核心原则：
- 🟩 8.0.3 所有属性修改必须通过修饰器栈，禁止直接修改（HP/MP资源型数值除外）
- 🟩 7.0.4 Buff、装备、特质统一通过修饰器机制修改属性（Modifier管线统一）
- 🟩 11.2.1 效果管线 Generate → Modify → Execute 三步严格顺序执行
- 🟩 7.0.3 修饰器链（Modifier Chain）是Effect执行链路中的修饰环节：暴击→属性克制→地形加成→Buff加成→最终结果

---

# 术语定义

## 属性（Attribute）

🟩 8.0.1 角色的数值化能力指标，分为核心属性、生命资源和衍生属性三类。

不是修饰器。不是修饰后的计算结果。

关键属性：
- 核心属性（Core Stat）8 维：力量、技巧、敏捷、体质、智力、意志、魅力、幸运
- 生命资源（Vital Resource）3 个：当前 HP、当前 MP、当前耐力
- 衍生属性（Derived Stat）13 个：最大 HP、攻击力、防御力、命中率、移动力等
- 每个属性有唯一的 AttributeKind 枚举标识

---

## 核心属性（Core Stat）

🟩 8.0.1 由种族、职业、等级决定的基础数值，是衍生属性计算的输入。

不是衍生属性。不是生命资源。不是修饰后的值。

关键属性：
- 存储在 Attributes.base（HashMap）
- set_base() 仅对 Core Stat 有效
- 共 8 维：Might / Dexterity / Agility / Vitality / Intelligence / Willpower / Presence / Luck
- 运行时不直接被伤害/治疗管线修改

---

## 基础属性值（Base Attribute Value）

核心属性在未叠加任何修饰器前的原始值。

不是当前值。不是修饰后的最终值。

关键属性：
- 来源为 UnitTemplate（种族/职业/等级配置）
- 通过 set_base() 设置
- 运行时不会被伤害/治疗管线改变

---

## 生命资源（Vital Resource）

存储当前值的消耗型属性，战斗中直接变化。

不是核心属性。不是衍生属性。不是可缓存的值。

关键属性：
- HP（当前生命值）、MP（当前魔法值）、Stamina（当前耐力值）
- 通过 set_vital() 设置，语义为"修改当前值"
- 不存储 base 值（base 由 MaxHp 等衍生属性决定）
- fill_vital_resources() 将当前值设为最大值

---

## 衍生属性（Derived Stat）

🟩 8.0.1 从核心属性实时计算的属性，不存储 base 值。

不是核心属性。不是生命资源。不是可缓存的值。

关键属性：
- 🟩 1.4.1 实时计算，每次 get() 调用重新求值（纯函数）
- 🟩 8.0.5 公式固定（如 Attack = Might × 2，Defense = Vitality）
- 🟩 8.0.3 可被修饰器叠加（Add / Multiply）
- 🟥 8.0.3 不可通过 set_base() 设置（set_base 对 Derived Stat 无效）

---

## 修饰器（Modifier）

🟩 7.0.3 对某个属性的增量或倍率修改，携带来源标识和操作类型。🟩 7.0.4 所有影响基础/派生属性的数值来源必须进入统一计算管线。

不是属性本身。不是效果。不是 Effect。

关键属性：
- 操作类型：Add（加法）或 Multiply（乘法）
- 来源标识：ModifierSource（Trait / Equipment / Buff / Consumable）
- 每个修饰器关联一个 AttributeKind
- 运行时实例为 AttributeModifierInstance

---

## 修饰器来源（ModifierSource）

🟩 7.0.2 标识修饰器由谁提供。根据宪法定义：种族=Trait+Modifier，职业=成长率+技能池+Trait，天赋=特殊Trait+专属Modifier，装备=Modifier+Trait，Buff/Debuff=临时Modifier+临时Trait+生命周期。

不是 Buff 实例 ID。不是 Entity ID。

关键属性：
- Trait 区间：u64::MAX ~ u64::MAX - 999
- Equipment 区间：u64::MAX - 1000 ~ u64::MAX - 1999
- Consumable 区间：u64::MAX - 2001 ~ u64::MAX - 2999
- Buff 区间：1 ~ u64::MAX - 2000
- 每种来源类型有独立的构造方法（trait_source / equipment_source / buff_source / consumable_source）

---

## 修饰器栈（Modifier Stack）

某个属性上叠加的所有修饰器集合，按 Add 先求和、Multiply 再求积的顺序计算。

不是列表。不是无序集合。不是修饰规则。

关键属性：
- 存储在 Attributes.modifiers（Vec）
- 同一来源可提供多个修饰器（如一个 Buff 同时修改攻击和防御）
- 计算公式：(base + sum(Add)) × product(Multiply)
- 通过 add_modifier / remove_modifiers_from 管理

---

## 效果（Effect）

🟩 11.2.1 一个动作（技能施放、攻击等）产生的待处理结果，如伤害、治疗、施加 Buff。

不是修饰器。不是属性变化。不是属性修饰后的值。

关键属性：
- 定义态为 EffectDef（Damage / Heal / ApplyBuff / Cleanse）
- 运行态为 PendingEffect / PendingEffectData
- 携带 source_entity、target_entity、source_tags、terrain_id
- 必须经过 Generate → Modify → Execute 三步管线

---

## 效果管线（Effect Pipeline）

🟩 11.2.1 战斗效果从生成到执行的三步严格管线。

不是单一计算。不是伤害公式。不是修饰规则。

关键属性：
- Generate 阶段：从 EffectDef + 上下文生成 PendingEffectData
- Modify 阶段：应用 ModifierRule 修饰，记录每步 ModifierEntry
- Execute 阶段：执行效果（扣血/加 Buff），通过 EffectHandler trait 分发
- 三步严格顺序，禁止跳步

---

## 效果上下文（GenerateContext / ExecuteContext）

封装效果生成和执行所需的全部输入数据。

不是全局状态。不是 ECS World。🟩 1.4.1 领域层纯函数化，不依赖引擎类型。

关键属性：
- 🟩 1.4.1 GenerateContext：纯数据结构，包含 source_attrs、target_attrs、defense_bonus、skill_id、source_tags、terrain_id（领域层不引用 ECS Query/Entity）
- 🟩 11.2.2 PreviewContext：source_attrs、target_attrs、terrain_defense_bonus、buff_registry（纯只读，无副作用）
- 🟩 1.4.2 ExecuteContext：封装效果执行所需的输入数据（apply_damage / apply_heal / apply_buff / apply_cleanse 的参数），副作用由应用层系统统一执行
- 纯数据传递，不存储持久状态
- 🟥 1.4.2 领域层 ExecuteContext 不得直接持有 `&mut World`，效果执行的副作用（扣血、加Buff）由应用层系统调用领域函数后写回 ECS

---

## 修饰规则（ModifierRule）

🟩 1.1.3 数据驱动的效果修饰规则，通过标签匹配决定是否应用修饰。

不是修饰器。不是属性修饰器。不是 Effect。

关键属性：
- 匹配条件：source_tag（攻击方技能标签）AND target_tag（目标标签集合）
- 修饰效果：DamageMultiplier / DamageBonus / HealMultiplier / HealBonus
- 通过 Calculator trait 分发计算（ModifierCalculator），禁止 match 分发
- 存储在 ModifierRuleRegistry，从 assets/rules/*.ron 加载

---

## 修饰器链（Modifier Chain）

效果执行过程中依序应用的所有修饰环节的总称。

不是单个ModifierRule。不是计算公式。不是Effect。

关键属性：
- 依序包含：暴击修饰 → 元素克制 → 地形加成 → 职业克制 → Buff加成 → 天气加成
- 每个环节通过ModifierRule的标签匹配机制生效
- 最终产出修饰后的效果值，再送入Formula公式计算
- 不是所有Effect都走完整修饰链（如治疗可能不走暴击）

> **优化来源**: docs/01-architecture/01-battle-gas/skill-buff-abstraction.md

---

## 修饰器组合规则（Modifier Composition Rules）

三种基本修饰类型的优先级和组合方式，保证数值计算的确定性。

不是修饰器本身。不是属性值。不是效果。

关键属性：
- 加算（Add）：在当前值基础上加百分比，如 +10% 地形加成
- 乘算（Multiply）：在当前值基础上乘倍率，如 ×1.5 暴击倍率
- 覆盖（Override）：直接替换为固定值，如某些 Buff 覆盖属性为固定值
- 修饰链按类型顺序执行：加算 → 乘算 → 覆盖
- 加算先求和，乘算再求积，覆盖最后执行

> **优化来源**: docs/01-architecture/01-battle-gas/skill-buff-abstraction.md

---

## 效果结果（EffectResult）

效果执行后的统一返回结果，包含目标状态和产生的消息。

不是 PendingEffect。不是 ModifierEntry。不是属性值。

关键属性：
- target_died：目标是否死亡
- damage_dealt：实际造成的伤害值
- healing_done：实际完成的治疗值
- buff_applied：是否成功施加 Buff
- PendingMessage 列表：DamageApplied / HealApplied / BuffApplied / EntityDied / EffectCompleted
- 所有 Effect Handler 执行后统一返回 EffectResult

> **优化来源**: docs/01-architecture/01-battle-gas/skill-buff-abstraction.md

---

## Effect 区分（Effect vs Modifier）

核心区别：Effect 是"做什么"（造成伤害、恢复生命、施加 Buff），Modifier 是"怎么调整数值"（伤害暴击倍率、元素克制系数、地形加成百分比）。

不是同一件事。不是替代关系。

关键属性：
- Effect 在 Execute 阶段执行，修改 World 状态
- Modifier 在 Modify 阶段执行，只修改 PendingEffectData 的 amount
- Effect 可独立触发（技能直接调用），Modifier 必须附着在 Effect 上
- 暴击、克制、地形加成是 Modifier 而非 Effect

> **优化来源**: docs/01-architecture/01-battle-gas/skill-buff-abstraction.md

---

# 领域边界

## 本领域负责

- 属性的三层分类（Core / Vital / Derived）和计算公式
- 修饰器栈的添加、移除、计算
- 修饰器来源的统一标识（ModifierSource）
- 效果管线 Generate → Modify → Execute 的执行逻辑
- 修饰规则的标签匹配和 Calculator 分发
- 标签系统的位掩码实现和匹配
- EffectHandler trait 分发机制
- 属性修改的可观测性（ModifierEntry 记录）
- 修饰器链的顺序编排和各环节通过标签匹配的修饰分发

## 本领域不负责

- Buff 的生命周期管理（由 Buff 领域负责：Apply/Remove/Tick）
- 装备的穿脱逻辑（由 Equipment 领域负责）
- 技能的冷却和槽位管理（由 Skill 领域负责）
- 回合状态机和行动顺序（由 Turn 领域负责）
- 战斗记录的 UI 展示（由 UI / Debug 领域负责）
- 角色生成和模板加载（由 Character 领域负责）
- 地形数据和寻路（由 Map 领域负责）
- 属性定义的 RON 加载（由 Content Pipeline 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| DamageApplied / HealApplied | Message | Battle/UI/Debug 领域 |
| CharacterDied | Message | Battle/Turn/UI 领域 |
| BuffApplied / BuffRemoved | Message | UI / Debug 领域 |
| EffectQueue 变更 | Resource 访问 | Battle 领域 |
| 属性值查询 | 函数调用 | 所有领域 |
| Trait 效果触发 | 函数调用 | Character 领域 |

---

# 与 Effect 领域的边界

> v1.1 新增 — Effect 成为一级领域后，明确 Modify 阶段内部逻辑的所有权边界。

Effect 和 AttributeModifier 是 Pipeline 中紧密协作但职责清晰分离的两个领域。详见 `docs/02-domain/effect/effect-rules.md` 与 AttributeModifier 领域的精确边界章节。

## 边界原则

| 维度 | Effect 领域负责 | AttributeModifier 领域负责 |
|------|----------------|---------------------------|
| **Pipeline 编排** | 三步顺序控制（Generate→Modify→Execute） | — |
| **Generate 阶段** | EffectHandler.generate() 生成初始值 | — |
| **Modify 阶段** | 调用时机（何时触发 Modify） | **内部逻辑**：ModifierRuleRegistry 遍历、标签匹配、Calculator 分发 |
| **Execute 阶段** | EffectHandler.execute() 执行效果 | — |
| **数据定义** | EffectDef、PendingEffectData、EffectResult | ModifierRule、ModifierEntry、ModifierCalculator、ModifierEffect |
| **注册表** | EffectHandlerRegistry | **ModifierRuleRegistry**、ModifierCalculatorRegistry |
| **可观测性** | EffectResult 输出 | **ModifierEntry 记录**（Modify 阶段每步填充） |

## 手续交接点

Effect Pipeline 在 Modify 阶段将 PendingEffectData 交给 AttributeModifier 领域处理：

```
Effect 领域                          AttributeModifier 领域
─────────────                       ─────────────────────
Generate 完成
  ↓
PendingEffectData 产出
  ↓
调用 Modify ──────────────────────→ ModifierRuleRegistry 遍历
                                     ↓
                                   标签匹配（source_tag + target_tag）
                                     ↓
                                   Calculator 计算修饰
                                     ↓
                                   记录 ModifierEntry
                                     ↓
修改后的 PendingEffectData ←──────── 返回
  ↓
Execute 阶段
```

## 共享数据结构

以下数据结构由 Effect 领域定义，但在 AttributeModifier 领域中被深度使用：

- **PendingEffectData**：Modify 阶段的输入/输出载体
- **ModifierEntry**：Modify 阶段的每步修饰记录（由 AttributeModifier 填充）
- **PendingEffectData.modifiers**：修饰记录列表（Modify 阶段填充，Execute 阶段消费）

## 不变量交叉检查

| 不变量 | 归属 | 说明 |
|--------|------|------|
| Effect 必须走三步管线 | Effect | 禁止跳步 |
| Generate 和 Modify 为纯函数 | Effect | 不产生副作用 |
| 属性修改必须通过修饰器栈 | **AttributeModifier** | 禁止直接修改 |
| 每步修饰必须记录 ModifierEntry | **AttributeModifier** | Modify 阶段填充 |
| 伤害下限 ≥ 1，治疗下限 ≥ 0 | **AttributeModifier** | Modify 阶段后保证 |
| ModifierRule 标签匹配 | **AttributeModifier** | source_tag + target_tag 双标签匹配 |
| Calculator trait 分发 | **AttributeModifier** | 禁止 match 分发修饰效果 |

---

# 生命周期

本领域无状态机，为纯函数式计算。

属性值每次 get() 调用实时计算，不存储最终值。修饰器栈的 Add/Remove 是函数式操作，不涉及状态转换。

唯一有状态的是 EffectQueue（Resource），其生命周期为：
- 系统清空（Generate 前清空）
- 推入效果（Generate 阶段）
- 修改效果（Modify 阶段）
- 排空执行（Execute 阶段）

---

# 不变量

## 不变量1：属性修改必须通过修饰器栈 🟥 8.0.3

任意时刻：

🟩 8.0.3 对属性的所有增量修改（Buff/装备/特质）必须通过 add_modifier / remove_modifiers_from 接口，禁止直接修改 base 值或 current_hp（生命资源的 set_vital 除外）。

违反表现：

`attributes.base[AttributeKind::Attack] = 999` 赋值语句出现。`attributes.current_hp -= damage` 不经管线直接扣血。

---

## 不变量2：衍生属性无 base 值 🟥 8.0.1

任意时刻：

🟩 8.0.1 set_base() 仅对 Core Stat 有效，对 Derived Stat 和 Vital Resource 调用 set_base 时应产生警告日志且不修改值。

违反表现：

调用 `set_base(AttributeKind::Attack, 999)` 后 Attack 值发生变化。

---

## 不变量3：效果管线三步严格顺序 🟥 11.2.1

任意时刻：

🟩 11.2.1 战斗效果必须经过 Generate → Modify → Execute 三步管线，禁止跳步执行。

违反表现：

在 Generate 阶段直接扣血。在 Execute 阶段重新应用修饰规则。绕过 EffectQueue 直接操作属性。

---

## 不变量4：伤害下限 ≥ 1，治疗下限 ≥ 0

任意时刻：

Modify 阶段完成后，伤害值必须 ≥ 1（确保每次攻击至少造成 1 点伤害），治疗值必须 ≥ 0（禁止负治疗）。

违反表现：

伤害值为 0 或负数。治疗值为负数。

---

## 不变量5：每步修饰必须记录

任意时刻：

Modify 阶段每应用一条 ModifierRule，必须生成 ModifierEntry（包含 before / after / rule_name），写入 PendingEffectData 的 modifiers 字段。

违反表现：

ModifierEntry 列表为空但伤害值已变化。DamageBreakdown 中缺少修饰步骤。

---

## 不变量6：Base Attribute 不被运行时修改

任意时刻：

Core Stat 的 base 值仅在角色生成（spawn from template）时设置，战斗运行时不通过 set_base 修改。生命资源的当前值通过 set_vital 修改，不修改 base。

违反表现：

战斗中调用 `set_base(AttributeKind::Might, 99)` 修改核心属性基础值。

---

## 不变量7：修饰器栈按操作类型分组计算 🟥 8.0.4

任意时刻：

🟩 8.0.4 属性最终值 = (base + sum(所有 Add 修饰器值)) × product(所有 Multiply 修饰器值)。Multiply 修饰器无 Add 时默认乘数为 1.0（不是 0）。

违反表现：

Multiply 修饰器乘数为 0 导致最终值为 0。Add 和 Multiply 混合计算顺序错误。

---

## 不变量8：修饰器链按类型顺序执行

任意时刻：

Modifier 链必须按加算 → 乘算 → 覆盖的顺序执行。加算先求和，乘算再求积，覆盖最后执行。保证数值计算的确定性。

违反表现：

覆盖修饰在加算之前执行，导致后续加算/乘算基于错误的基准值。乘算在加算之前执行，破坏 "(base + sum(Add)) × product(Multiply)" 公式。

> **优化来源**: docs/01-architecture/01-battle-gas/skill-buff-abstraction.md

---

## 不变量9：Modifier 是修饰环节而非 Effect

任意时刻：

暴击、元素克制、地形加成、职业克制、天气加成——这些都是 Modifier，不是 Effect。Modifier 在 Modify 阶段介入，不修改 World 状态。Effect 在 Execute 阶段执行，修改 World 状态。

违反表现：

将暴击实现为独立的 Effect（execute_damage_with_crit），而不是 Modifier 链中的一个修饰环节。

> **优化来源**: docs/01-architecture/01-battle-gas/skill-buff-abstraction.md

---

# 业务规则

## 规则1：统一修饰器机制 🟩 7.0.4

允许：
- Buff 通过 add_modifiers_from_def 添加修饰器
- 装备通过 add_modifiers_from_def 添加修饰器（ModifierSource::equipment_source）
- 特质通过 add_modifiers_from_def 添加修饰器（ModifierSource::trait_source）
- 消耗品通过 add_modifiers_from_def 添加修饰器

禁止：
- 为不同能力来源创建独立的属性修改机制
- 绕过修饰器栈直接修改属性值

必须：
- 所有来源统一使用 AttributeModifierDef → AttributeModifierInstance 转换
- 移除效果时通过 remove_modifiers_from(source) 精确清理对应来源的修饰器

---

## 规则2：效果管线不可跳步 🟩 11.2.1

允许：
- Generate 阶段通过 EffectHandler trait 分发生成
- Modify 阶段通过 ModifierRuleRegistry 标签匹配修饰
- Execute 阶段通过 EffectHandler trait 分发执行

禁止：
- 跳过 Modify 阶段直接 Execute
- 在 Generate 阶段执行扣血/加 Buff
- 在 Execute 阶段重新计算修饰

必须：
- Generate → Modify → Execute 使用 .chain() 保证严格顺序
- OnHit / OnKill 触发的效果也必须经过 Modify → Execute

---

## 规则3：标签匹配控制修饰规则 🟩 1.1.3

允许：
- ModifierRule 通过 source_tag + target_tag 双标签匹配
- 多条规则链式叠加（先匹配先应用）
- 自定义 Calculator 注册扩展修饰效果

禁止：
- 绕过标签匹配直接应用修饰规则
- 使用 match 硬编码修饰效果分发
- 运行时使用字符串查询标签（必须使用 GameplayTag 位掩码）

必须：
- 修饰规则从 assets/rules/*.ron 加载（Rule/Content 分离）
- Calculator 通过 type_name 查找分发，禁止 match

---

## 规则4：EffectHandler trait 分发

允许：
- 新增效果类型只需实现 EffectHandler trait
- 注册到 EffectHandlerRegistry（O(1) HashMap 查找）
- 通过 type_name 字符串匹配处理器

禁止：
- 使用 match 分发效果类型（违反 trait 扩展原则）
- 修改管线调度代码添加新效果类型
- 在 Execute 阶段使用 match 硬编码执行逻辑

必须：
- type_name 与 EffectDef::type_name 返回值一致
- 未注册的效果类型输出 warn 日志并跳过

---

## 规则5：修饰器来源精确清理

允许：
- 按来源精确移除：remove_modifiers_from(source)
- 按类型批量移除：remove_trait_modifiers / remove_equipment_modifiers
- 移除减益修饰符：remove_debuff_modifiers

禁止：
- 移除时不区分来源，清除所有修饰器
- Buff 移除后不清理对应的修饰器
- 装备脱下后不清理对应的修饰器

必须：
- Buff 移除时调用 remove_modifiers_from(buff_instance_id.to_modifier_source())
- 装备脱下时调用 remove_equipment_modifiers()
- 特质重建时先 remove_trait_modifiers() 再重新添加

---

## 规则6：Modifier 与 Effect 职责分离 🟩 11.2.1

允许：
- Effect 负责"做什么"（Damage / Heal / ApplyBuff / Cleanse）
- Modifier 负责"怎么调整数值"（暴击倍率、元素克制系数、地形加成百分比）
- Modifier 在 Modify 阶段介入，不修改 World 状态
- Effect 在 Execute 阶段执行，修改 World 状态

禁止：
- 将暴击、克制、地形加成实现为独立的 Effect（如 execute_damage_with_crit）
- Modifier 直接修改 World 状态（扣血、加 Buff）
- Effect 在 Execute 阶段重新应用修饰规则

必须：
- 每种修饰类型通过 ModifierRule 标签匹配机制统一处理
- 新增修饰环节只需添加新标签 + 新 ModifierRule 配置，不修改 Modifier 链逻辑

> **优化来源**: docs/01-architecture/01-battle-gas/skill-buff-abstraction.md

---

## 规则7：效果结果统一返回

允许：
- 所有 EffectHandler 执行后统一返回 EffectResult
- EffectResult 携带 target_died、damage_dealt、healing_done 等状态
- 通过 PendingMessage 列表广播效果结果（DamageApplied / HealApplied / BuffApplied / EntityDied）

禁止：
- EffectHandler 执行后不返回结果
- 直接修改属性而不通过 EffectResult 通知其他系统

必须：
- EffectResult 是效果执行的唯一返回类型
- PendingMessage 由 Effect Pipeline 统一广播

> **优化来源**: docs/01-architecture/01-battle-gas/skill-buff-abstraction.md

---

# 管线

## 效果管线（Effect Pipeline）

```
Generate → Modify → Execute
```

### Step1：Generate（生成效果）

输入：EffectDef（技能定义）+ GenerateContext（攻击者/目标属性、地形、标签）
处理：通过 EffectHandlerRegistry 查找处理器，调用 handler.generate() 计算初始值
输出：PendingEffectData（含 amount、source_tags、terrain_id）
禁止：在 Generate 阶段修改目标属性。跳过 EffectHandler 直接计算。

### Step2：Modify（修饰效果）

输入：PendingEffect（Generate 输出）+ ModifierRuleRegistry（修饰规则）
处理：遍历规则，标签匹配后通过 Calculator 计算修饰，记录 ModifierEntry
输出：修改后的 PendingEffectData（amount 已更新，modifiers 已填充）
禁止：在 Modify 阶段修改目标属性。跳过标签匹配。不记录 ModifierEntry。

### Step3：Execute（执行效果）

输入：PendingEffect（Modify 输出）+ ExecuteContext（World 访问）
处理：通过 EffectHandlerRegistry 查找处理器，调用 handler.execute() 执行
输出：EffectResult（target_died 状态）+ PendingMessage（DamageApplied / HealApplied）
禁止：在 Execute 阶段重新应用修饰规则。直接删除 Entity（必须通过 Dead Tag）。

---

## 属性计算管线（Attribute Calculation）

```
Base Value → + Add Modifiers → × Multiply Modifiers → Final Value
```

### Step1：获取 Base Value

输入：AttributeKind
处理：Core Stat 从 Attributes.base 读取；Derived Stat 通过公式计算（如 Attack = Might × 2）
输出：基础数值
禁止：Derived Stat 读取 base 值（Derived Stat 无 base）

### Step2：叠加 Add 修饰器

输入：base value + 该属性的所有 Add 修饰器
处理：sum(所有 ModifierOp::Add 的 value)
输出：base + add_sum
禁止：在 Add 阶段应用 Multiply 修饰器

### Step3：叠加 Multiply 修饰器

输入：(base + add_sum) + 该属性的所有 Multiply 修饰器
处理：product(所有 ModifierOp::Multiply 的 value)，无 Multiply 时默认 1.0
输出：最终属性值
禁止：Multiply 乘数为 0（会导致最终值归零）

---

## 修饰器应用管线（Modifier Application）

```
Source 创建 → 栈插入 → 值计算（随 get() 调用）
```

### Step1：Source 创建

输入：BuffData / EquipmentDef / TraitEffect
处理：构建 AttributeModifierInstance（kind / op / value / source）
输出：Modifier 实例
禁止：不携带 ModifierSource 创建修饰器

### Step2：栈插入

输入：Modifier 实例 + Attributes 组件
处理：调用 attributes.add_modifier() 推入 modifiers Vec
输出：修饰器栈更新
禁止：插入时修改 base 值

### Step3：值计算

输入：属性的 get() 调用
处理：实时读取 modifiers Vec，按 Add → Multiply 顺序计算
输出：最终属性值
禁止：缓存计算结果而不定义失效条件

---

# 数据结构

## Attributes（属性组件）

职责：存储角色的核心属性基础值、生命资源当前值和修饰器栈

结构：
- base：核心属性基础值映射（8 维）
- current_hp：当前生命值
- current_mp：当前魔法值
- current_stamina：当前耐力值
- base_attack_range：基础攻击范围
- modifiers：修饰器实例列表

要求：
- base 仅通过 set_base() 设置（仅 Core Stat 有效）
- current_hp/mp/stamina 通过 set_vital() 设置
- modifiers 通过 add_modifier / remove_modifiers_from 管理
- Derived Stat 通过 get() 实时计算，不存储

---

## ModifierSource（修饰器来源）

职责：唯一标识修饰器的提供者（Trait / Equipment / Buff / Consumable）

结构：
- 内部 u64 值，按区间区分来源类型

要求：
- Trait 区间、Equipment 区间、Buff 区间、Consumable 区间不重叠
- 每个来源有对应的判断方法（is_trait / is_equipment / is_buff）
- 同一来源的所有修饰器通过 remove_modifiers_from(source) 批量移除

---

## ModifierRule（修饰规则）

职责：数据驱动的效果修饰规则，通过标签匹配决定修饰方式

结构：
- name：规则名称（用于日志和 ModifierEntry 记录）
- source_tag：攻击方技能需要包含的标签
- target_tag：目标需要包含的标签
- effect：修饰效果（DamageMultiplier / DamageBonus / HealMultiplier / HealBonus）

要求：
- 从 assets/rules/*.ron 加载（Rule/Content 分离）
- 通过 ModifierCalculator trait 分发计算，禁止 match
- 匹配条件为 source_tag AND target_tag 双标签匹配

---

## ModifierEntry（修饰步骤记录）

职责：记录 Modify 阶段每一步修饰的前后值和规则名

结构：
- before：修饰前值
- after：修饰后值
- rule_name：修饰规则名称

要求：
- 每应用一条 ModifierRule 必须生成一个 ModifierEntry
- 存入 PendingEffectData 的 modifiers 字段
- 用于 BattleRecord 的 DamageBreakdown 显示

---

## EffectDef（效果定义）

职责：技能中声明的效果类型（RON 反序列化用）

结构：
- Damage { multiplier, ignore_def_percent }
- Heal { amount }
- ApplyBuff { buff_id, duration }
- Cleanse

要求：
- 每个变体实现 type_name() 返回效果类型名
- type_name 与 EffectHandler::type_name 一致
- 不包含运行时状态

---

## PendingEffect / PendingEffectData（待处理效果）

职责：效果管线中间态数据，从 Generate 流向 Execute

结构：
- PendingEffect：source / target / data / source_tags / terrain_id
- PendingEffectData::Damage { amount, is_skill, base_amount, modifiers }
- PendingEffectData::Heal { amount, base_amount, modifiers }
- PendingEffectData::ApplyBuff { buff_id, duration }
- PendingEffectData::Cleanse

要求：
- base_amount 在 Modify 阶段首次设置（记录修饰前原始值）
- modifiers 在 Modify 阶段填充（每步 ModifierEntry）
- 不直接存储 Entity 引用（存储 Entity ID）

---

## GameplayTag（标签位掩码）

职责：用 u64 位掩码表示的分类标签，支持 O(1) 查询

结构：
- GameplayTag(u64)：单个标签
- GameplayTags(u64)：Entity 上的标签集合组件

要求：
- 定义态 TagName 枚举用于 RON，运行时转换为 GameplayTag
- 位运算查询（has / has_any / has_all）
- 标签增删通过 GameplayTags 组件的 add / remove 方法

---

## EffectHandlerRegistry（效果处理器注册表）

职责：存储所有 EffectHandler 实现，通过 type_name 查找分发

结构：
- handlers：类型名到处理器的映射

要求：
- 注册 4 个内置处理器（Damage / Heal / ApplyBuff / Cleanse）
- 不重复注册（register 时检查 key 是否存在）
- O(1) HashMap 查找

---

# 禁止事项

禁止：直接修改属性绕过修饰器栈

原因：破坏属性计算的统一性和可观测性，导致 ModifierEntry 记录缺失，BattleRecord 数据不完整

违反后果：伤害计算结果与预期不一致，调试面板无法追踪属性变化来源

---

禁止：跳过效果管线直接执行

原因：Generate → Modify → Execute 是保证战斗公平性和可观测性的核心管线

违反后果：修饰规则不生效，伤害/治疗值异常，BattleRecord 缺少修饰步骤

---

禁止：在 Generate 阶段修改目标属性

原因：Generate 阶段只计算初始值，修改属性属于 Execute 阶段职责

违反后果：属性在管线中间被意外修改，后续 Modify/Execute 基于错误状态计算

---

禁止：缓存 Derived Stat 计算结果

原因：Derived Stat 依赖核心属性和修饰器栈，任一变化都应实时反映

违反后果：属性值与实际状态不一致，Buff/装备效果延迟生效

---

禁止：使用 match 分发修饰效果类型

原因：match 分发违反 Calculator trait 扩展原则，新增效果类型需修改分发代码

违反后果：每次新增修饰效果类型都要修改核心分发代码，违反开闭原则

---

禁止：使用 match 分发效果执行类型

原因：EffectHandler trait 分发是效果管线的扩展点，match 硬编码破坏扩展性

违反后果：每次新增效果类型都要修改 execute_effects 调度代码

---

禁止：将暴击/克制/地形加成实现为独立 Effect

原因：暴击、克制、地形加成是 Modifier 而非 Effect。Effect 是"做什么"（造成伤害），Modifier 是"怎么调整数值"（暴击倍率、克制系数）。如果暴击是 Effect 而非 Modifier，那么每个伤害技能都需要单独实现"暴击版"和"非暴击版"，组合爆炸无法控制。

违反后果：修饰环节增加时函数数量爆炸，无法复用修饰管线。

> **优化来源**: docs/01-architecture/01-battle-gas/skill-buff-abstraction.md

---

禁止：修饰器链不按类型顺序执行

原因：Modifier 链必须按加算 → 乘算 → 覆盖的顺序执行，保证数值计算的确定性。乱序执行会导致相同的 Modifier 组合产生不同的结果。

违反后果：数值计算结果不确定，同一技能在不同场景下伤害不一致。

> **优化来源**: docs/01-architecture/01-battle-gas/skill-buff-abstraction.md

---

禁止：Modifier 直接修改 World 状态

原因：Modifier 只修改 PendingEffectData 的 amount，不直接修改 HP、添加 Component 等 World 状态变更。World 状态变更属于 Effect 在 Execute 阶段的职责。

违反后果：Modifier 执行了不属于它的职责，破坏管线的阶段分离。

> **优化来源**: docs/01-architecture/01-battle-gas/skill-buff-abstraction.md

---

禁止：Buff/装备移除时不清理修饰器

原因：移除效果后修饰器残留会导致属性值与实际状态不一致

违反后果：脱下装备后攻击力未恢复，Buff 过期后属性未还原

---

禁止：Multiply 修饰器乘数为 0

原因：乘数为 0 会导致最终属性值归零，违反游戏平衡设计

违反后果：角色攻击力/防御力归零，战斗平衡崩溃

---

禁止：Derived Stat 通过 set_base 设置

原因：Derived Stat 无 base 值，其值由公式实时计算，set_base 无语义

违反后果：set_base 调用静默失败，开发者误以为属性已修改

---

禁止：EffectHandler type_name 与 EffectDef type_name 不一致

原因：EffectHandlerRegistry 通过 type_name 查找处理器，不一致会导致处理器匹配失败

违反后果：效果生成/执行时找不到处理器，输出 warn 日志并跳过效果

---

禁止：为每种修饰环节创建独立的execute函数（execute_damage_with_crit等）

原因：每种修饰类型应通过ModifierRule标签匹配机制统一处理，不应硬编码独立函数

违反后果：修饰环节增加时函数数量爆炸，无法复用修饰管线

---

# AI 修改规则

## 宪法合规检查清单

修改本领域代码前，必须逐项确认：
- 🟩 1.4.1 核心计算为纯函数，不依赖 ECS World
- 🟩 1.4.2 领域函数无副作用，不触发事件、不修改全局状态
- 🟩 8.0.3 属性修改通过修饰器栈，不直接修改最终值
- 🟩 8.0.4 所有数值来源进入统一计算管线
- 🟩 11.2.1 效果执行遵循 Generate → Modify → Execute 三步管线
- 🟩 1.1.3 新增修饰规则通过 RON 配置，不硬编码

## 领域事件清单

本领域产生的领域事件（🟩 2.2.6 领域事件是唯一业务事实源）：
- `DamageApplied` — 伤害施加完成，携带 source_unit、target_unit、damage、modifiers
- `HealApplied` — 治疗施加完成，携带 source_unit、target_unit、healing
- `BuffApplied` — Buff 施加完成，携带 target_unit、buff_id、duration
- `EntityDied` — 单位死亡，携带 unit_id、killer_id
- `EffectCompleted` — 效果管线执行完成，携带 effect_result

> 🟩 2.2.7 新增领域事件必须先更新白名单文档
> 🟩 13.10.1 所有核心业务事实通过领域事件表达，日志、回放、UI 均监听同一事件源

---

## 如果新增效果类型

允许：
- 实现 EffectHandler trait（type_name / generate / preview / execute）
- 注册到 EffectHandlerRegistry（在 register_defaults 中添加）
- 添加对应的 EffectDef 变体

禁止：
- 修改管线调度代码（generate.rs / modify.rs / execute.rs 的调度逻辑）
- 在 execute_effects 中添加 match 分支

优先检查：
- EffectDef::type_name 与 EffectHandler::type_name 是否一致
- generate 返回 None 是否正确处理（类型不匹配）
- execute 返回 None 是否正确处理（类型不匹配）

---

## 如果新增修饰规则

允许：
- 在 assets/rules/*.ron 中添加新的 RON 配置文件
- 确保 source_tag 和 target_tag 使用正确的 TagName

禁止：
- 在 ModifierRuleRegistry 中硬编码默认规则
- 修改 ModifierCalculator trait 的 calculate 方法签名

优先检查：
- source_tag 和 target_tag 是否在 GameplayTag 中有对应定义
- Calculator 是否能处理新的 ModifierEffect 类型
- 修饰结果是否满足伤害下限 ≥ 1 或治疗下限 ≥ 0

---

## 如果修改属性计算公式

允许：
- 修改 Derived Stat 的计算函数（如 calc_attack / calc_defense）
- 添加新的 Derived Stat（需在 AttributeKind 中添加枚举变体）

禁止：
- 修改 Core Stat 的 base 值含义
- 修改 Add → Multiply 的计算顺序
- 缓存 Derived Stat 结果

优先检查：
- 三类属性互斥性（Core / Vital / Derived 各属且仅属一类）
- 修饰器栈计算公式正确性（Add sum + Multiply product）
- 所有引用该属性的系统是否兼容新公式

---

## 如果新增 ModifierSource 来源类型

允许：
- 添加新的构造方法（如 consumable_source）
- 确保区间不与现有来源重叠

禁止：
- 使用已占用的 u64 区间
- 不定义 is_xxx() 判断方法

优先检查：
- 新区间是否与 Trait / Equipment / Buff / Consumable 区间重叠
- remove_modifiers_from 是否能正确清理新来源
- 装备穿脱 / Buff 移除时是否需要特殊处理新来源

---

## 如果新增修饰环节（如天气加成）

允许：
- 添加新标签+新ModifierRule配置

禁止：
- 修改Modifier链的顺序逻辑

优先检查：
- 新标签是否在GameplayTag中有对应定义
- 新ModifierRule的source_tag和target_tag是否正确
- 修饰结果是否满足伤害下限 ≥ 1 或治疗下限 ≥ 0

---

## 如果修改修饰器链顺序

允许：
- 在 Modifier 链中添加新的修饰环节（如天气加成）
- 调整单个修饰环节的计算逻辑

禁止：
- 改变加算 → 乘算 → 覆盖的总体执行顺序
- 将 Modifier 实现为独立的 Effect
- 在 Execute 阶段重新应用修饰规则

优先检查：
- 新修饰环节是否通过 ModifierRule 标签匹配机制生效
- 新修饰环节的执行顺序是否正确（加算在前、乘算在中、覆盖在后）
- 修饰结果是否满足伤害下限 ≥ 1 或治疗下限 ≥ 0
- ModifierEntry 记录是否包含新修饰环节

> **优化来源**: docs/01-architecture/01-battle-gas/skill-buff-abstraction.md

---

## 如果测试失败

排查顺序：
1. 检查 EffectDef::type_name 与 EffectHandler::type_name 是否匹配
2. 检查 ModifierRule 的 source_tag / target_tag 是否正确
3. 检查修饰器栈的 Add / Multiply 计算顺序是否正确
4. 检查 remove_modifiers_from 是否正确清理了来源
5. 检查 Derived Stat 公式是否被意外修改
6. 检查 Base Attribute 是否被运行时修改（set_base 对 Derived Stat 无效）
7. 检查修饰器链是否按加算 → 乘算 → 覆盖顺序执行
8. 检查暴击/克制/地形加成是否通过 Modifier 而非 Effect 实现

> **优化来源**: docs/01-architecture/01-battle-gas/skill-buff-abstraction.md

---

# 交叉引用

| 主题 | 详细文档 |
|------|----------|
| Effect 一级领域规则 | `docs/02-domain/effect/effect-rules.md` |
| Effect 与 AttributeModifier 边界 | `docs/02-domain/effect/effect-rules.md` 与 AttributeModifier 领域的精确边界 |
| GameplayTag 标签系统 | `docs/02-domain/tag/tag-rules.md` |
| 标签匹配驱动 ModifierRule | `docs/08-decisions/ADR-015-技能标签与分类体系.md` |
| 标签系统架构重整 | `docs/08-decisions/ADR-023-标签系统架构重整.md` |
| 技能/Buff/Effect 统一抽象 | `docs/01-architecture/01-battle-gas/skill-buff-abstraction.md` |
| Effect Pipeline 三步管线 | `docs/01-architecture/01-battle-gas/skill-buff-abstraction.md` §3.2 |

---

## 附录：铃兰参考数据

### Attribute 引用数据

> 领域：Attribute | 来源：78铃兰.md §一、§六、补充11 | 数据层：Definition + Instance + Runtime

#### 数据实体清单

##### AttributeDefinition（Definition层）

角色固有属性定义，战斗中不可变，是所有百分比加成的计算基数。

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| `id` | AttributeId | PK | 属性唯一标识 |
| `name_key` | String | - | 属性名称本地化Key |
| `category` | Enum | core/secondary | 属性分类 |
| `base_value` | f32 | ≥0 | 基础数值 |
| `min_value` | f32 | ≥0 | 属性下限 |
| `max_value` | f32 | >min_value | 属性上限 |

**核心五维（category = core）**

| AttributeId | 名称 | 下限 | 上限 |
|-------------|------|------|------|
| `phys_atk` | 物理攻击 | 0 | 无上限 |
| `magic_atk` | 魔法攻击 | 0 | 无上限 |
| `phys_def` | 物理防御 | 0 | 无上限 |
| `magic_def` | 魔法防御 | 0 | 无上限 |
| `max_hp` | 最大生命值 | 1 | 无上限 |

**次级属性（category = secondary）**

| AttributeId | 名称 | 下限 | 上限 |
|-------------|------|------|------|
| `crit_rate` | 暴击率 | 0 | 0.95 |
| `crit_dmg` | 暴击伤害 | 1.5 | 无上限 |
| `move_range` | 移动范围 | 1 | 无上限 |
| `atk_range` | 攻击范围 | 1 | 无上限 |
| `hit_rate` | 命中率 | 0 | 1.0 |
| `dodge_rate` | 闪避率 | 0 | 0.8 |

##### AttributeInstance（Instance层）

| 字段名 | 类型 | 说明 |
|--------|------|------|
| `entity` | Entity | 所属实体 |
| `attribute_id` | AttributeId | 属性引用 |
| `base_value` | f32 | 基础面板值（战斗外固定） |
| `modifiers` | Vec<ModifierRef> | 当前生效的修正列表 |

#### 属性转换机制

| 字段名 | 类型 | 说明 |
|--------|------|------|
| `source_attr` | AttributeId | 源属性（如防御、当前损失血量） |
| `target_attr` | AttributeId | 目标属性（如攻击） |
| `ratio` | f32 | 转换比例 |
| `condition` | Option<ConditionId> | 触发条件（可选） |

转换规则：
- 转换后的属性**可被后续百分比加成放大**
- 部分条件型加成**不参与转换**

#### 数值边界强制规则

| 规则 | 约束 |
|------|------|
| 取整规则 | 向下取整，最小为1 |
| 属性下限 | 防御最低0，移动最低1 |
| 暴击率上限 | 95% |
| 闪避率上限 | 80% |
| 命中率上限 | 100% |
| 减伤上限 | 90% |
| 结算顺序 | 所有百分比计算完成后统一取整 |

---

### Modifier 引用数据

> 领域：Modifier | 来源：78铃兰.md §一、§二、§五、§六 | 数据层：Definition + Instance

#### 数据实体清单

##### ModifierDefinition（Definition层）

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| `id` | ModifierId | PK | 修正器唯一标识 |
| `target_attr` | AttributeId | FK | 目标属性 |
| `operation` | ModifierOp | - | 修正操作类型 |
| `value` | f32 | - | 修正值 |
| `stacking_rule` | StackingId | FK | 堆叠策略引用 |
| `source_type` | Enum | buff/equipment/trait/terrain | 来源分类 |

##### ModifierOp 枚举

| 操作 | 公式 | 叠加方式 | 适用属性 |
|------|------|----------|----------|
| `Add` | base + value | - | 固定值加成 |
| `AddPercent` | base × (1 + Σvalue) | 加算 | 攻击%、暴击率、增伤% |
| `MulPercent` | base × Π(1 - value) | 乘算 | 降防、无视防御、减伤、易伤 |

#### 各区间的叠加方式

| 区间 | ModifierOp | 叠加方式 |
|------|-----------|----------|
| 攻击百分比 | AddPercent | 加算 |
| 固定攻击加成 | Add | 加算 |
| 降防效果 | MulPercent | 乘算 |
| 无视防御 | MulPercent | 乘算 |
| 增伤效果 | AddPercent | 加算 |
| 易伤效果 | MulPercent | 乘算 |
| 减伤效果 | MulPercent | 乘算 |
| 暴击率 | AddPercent | 加算 |
| 暴击伤害 | AddPercent | 加算 |

#### Modifier来源分类

| 来源类型 | 说明 | 生命周期 |
|----------|------|----------|
| Buff | Buff附加的属性修正 | 随Buff存在 |
| Equipment | 装备词条附加的属性修正 | 装备穿戴期间 |
| Trait | 天赋/被动附加的属性修正 | 永久 |
| Terrain | 地形附加的属性修正 | 站在对应地形时 |

#### Schema草案

```yaml
# modifier_config.ron
(
  modifiers: [
    (id: "atk_up_20", target_attr: "phys_atk", operation: AddPercent, value: 0.2,
     stacking_rule: "additive_same_name_max"),
    (id: "def_down_40", target_attr: "phys_def", operation: MulPercent, value: 0.4,
     stacking_rule: "multiplicative"),
    (id: "armor_pen_40", target_attr: "phys_def", operation: MulPercent, value: 0.4,
     stacking_rule: "multiplicative"),
    (id: "dmg_up_15", target_attr: "dmg_multiplier", operation: AddPercent, value: 0.15,
     stacking_rule: "additive"),
  ],
)
```
