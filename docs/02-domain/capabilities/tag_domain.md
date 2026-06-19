---
id: 02-domain.tag
title: Tag（标签）领域规则 v2.0
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-28
tags:
  - domain
  - tag
  - capabilities
---

# Tag 领域规则

> **哲学定位**: Tag 是五层架构中的**语义层**（Type→Tag→Query→Rule→Content）。
> Tag 回答"是不是"（语义描述），不回答"多少"（数值计算）。
> 参与规则计算的东西用强类型（Type System），用于筛选/分类/内容驱动的东西用 Tag。

---

## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Tag | 具有层级关系的标签标识，通过位掩码实现 O(1) 包含检查 | 负责：语义描述与分类筛选；**不负责**：规则计算、状态管理、数据携带 |
| TagId | 标签的唯一标识符，强类型 | 负责：标签身份的唯一定义；不负责：描述标签含义 |
| TagSet | 标签的集合表示，支持位掩码运算 | 负责：标签集合的高效存储与查询；不负责：标签的层级推导 |
| TagHierarchy | 标签之间的父子层级关系（父标签自动包含子标签） | 负责：层级树的构建与维护；不负责：业务逻辑中的标签使用 |
| TagQuery | 标签条件查询，支持 Any/All/None 三种匹配模式 | 负责：基于标签的条件筛选；不负责：具体的条件业务含义 |

### 术语映射关系

```
TagId          ─── 身份的"身份证号"
TagHierarchy   ─── 家族的"族谱"
TagSet         ─── 集合的"清单"
TagQuery       ─── 筛选的"过滤器"
```

### 已对齐项目术语

- **Unit**：战场上的可操作单位（玩家或 AI 控制），通过 GameTagContainer 获得标签
- **Skill**：主动技能，通过标签标记技能类型（如 Skill.Fire, Skill.Heal）
- **Buff**：持续性增益/减益效果，通过标签标记 Buff 类型（如 Buff.Poison, Buff.Burn）
- **Equipment**：装备物品，通过标签标记装备类型（如 Equipment.Weapon.Sword）
- **Faction**：阵营关系，通过标签表达阵营归属（如 Faction.Goblin, Faction.Player）

---

## 2. 标签层级状态机

### 层级树示例

```
Tag.Root
 ├── Tag.Damage
 │    ├── Tag.Damage.Physical
 │    │    ├── Tag.Damage.Physical.Slashing
 │    │    ├── Tag.Damage.Physical.Piercing
 │    │    └── Tag.Damage.Physical.Bludgeoning
 │    └── Tag.Damage.Elemental
 │         ├── Tag.Damage.Elemental.Fire
 │         ├── Tag.Damage.Elemental.Cold
 │         ├── Tag.Damage.Elemental.Lightning
 │         └── Tag.Damage.Elemental.Acid
 ├── Tag.Status
 │    ├── Tag.Status.Buff
 │    ├── Tag.Status.Debuff
 │    ├── Tag.Status.Immune
 │    │    ├── Tag.Status.Immune.Fire
 │    │    └── Tag.Status.Immune.Poison
 │    └── Tag.Status.State
 │         ├── Tag.Status.State.Bless
 │         ├── Tag.Status.State.Poisoned
 │         └── Tag.Status.State.Burned
 ├── Tag.Ability
 │    ├── Tag.Ability.School
 │    │    ├── Tag.Ability.School.Fire
 │    │    └── Tag.Ability.School.Healing
 │    ├── Tag.Ability.Type
 │    │    ├── Tag.Ability.Type.Active
 │    │    ├── Tag.Ability.Type.Passive
 │    │    └── Tag.Ability.Type.Reaction
 │    └── Tag.Ability.Targeting
 │         ├── Tag.Ability.Targeting.Single
 │         └── Tag.Ability.Targeting.AoE
 ├── Tag.Equipment
 │    ├── Tag.Equipment.Slot
 │    │    ├── Tag.Equipment.Slot.MainHand
 │    │    ├── Tag.Equipment.Slot.OffHand
 │    │    ├── Tag.Equipment.Slot.Helmet
 │    │    └── Tag.Equipment.Slot.Armor
 │    └── Tag.Equipment.Category
 │         ├── Tag.Equipment.Category.Weapon
 │         └── Tag.Equipment.Category.Armor
 ├── Tag.Faction
 │    ├── Tag.Faction.Player
 │    ├── Tag.Faction.Enemy
 │    └── Tag.Faction.Neutral
 └── Tag.Combat
      ├── Tag.Combat.InCombat
      └── Tag.Combat.OutOfCombat
```

### 层级包含规则

- **父标签自动包含子标签**：查询 Tag.DamageType.Elemental 时，所有元素伤害子标签（Fire/Cold/Lightning/Acid）均匹配
- **子标签不反向包含父标签**：查询 Tag.DamageType.Elemental.Fire 时，Tag.DamageType.Elemental 不自动匹配
- **多继承禁止**：一个标签只能有唯一父标签，禁止多父标签（防止层级推导的歧义性）

---

## 3. 不变量（Invariants）

### 3.1 标签唯一性
- **条件**：任何标签定义时
- **不变量**：TagId 在整个标签树中必须全局唯一
- **违反后果**：注册冲突，后注册的标签被拒绝并产生错误日志

### 3.2 层级闭环禁止
- **条件**：任何层级关系建立时
- **不变量**：父子关系不得形成循环引用（A 是 B 的父，B 不能是 A 的父或祖先）
- **违反后果**：层级注册失败，返回 TagHierarchyError::CircularDependency

### 3.3 父标签必有实际含义
- **条件**：定义新标签时
- **不变量**：所有叶节点标签必须有实际业务含义，禁止创建"空壳"标签
- **违反后果**：标签被注册但被标记为"未使用"，由内容校验器报告告警

### 3.4 标签存在性
- **条件**：任何引用标签的操作（查询/匹配/条件检查）执行前
- **不变量**：被引用的 TagId 必须在标签树中已注册
- **违反后果**：引用不存在的标签导致查询失败，返回 TagLookupError

### 3.5 标签命名空间一致性
- **条件**：标签层级路径中
- **不变量**：同一层级的标签组属于同一命名空间分类（如 DamageType 下不允许混入 EquipmentSlot 标签）
- **违反后果**：违反命名空间规则被视为配置错误，内容校验不通过

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：标签多继承（一个标签有多个父标签） — 理由：层级推导会出现歧义，父标签不可确定
- 🟥 禁止：运行时动态创建新标签类型 — 理由：标签树应在加载阶段确定，运行时只增删实体上的标签实例
- 🟥 禁止：标签跨域引用（如 DamageType 下放 Faction 标签） — 理由：破坏命名空间一致性，导致信息查询混乱
- 🟥 禁止：用标签承载数据（如标签名中编码数值信息） — 理由：标签只做标识分类，数据应放在 Attribute 中
- 🟥 禁止：直接移除有子标签的父标签 — 理由：移除父标签必须连带移除所有子标签，或先迁移子标签到新父标签
- 🟥 禁止：TagDef 中直接存储用户可见文本的自然语言 — 理由：必须使用 name_key/desc_key: LocalizationKey 引用。违反宪法 §22 Localization First。
- 🟥 禁止：用 Tag 替代参与规则计算的 Type — 理由：`if target.has_tag("boss")` 决定伤害公式是错误的，应该用强类型（DamageType::Fire）或 Trait 系统。Tag 只做语义描述，不参与规则计算。
- 🟥 禁止：用 Tag 表达动态状态 — 理由：`Character.Dead`、`Character.Stunned` 应该是 ECS Component，不是 Tag。Tag 描述长期不变的语义（Enemy.Boss, Character.Human），不描述瞬时状态。
- 🟥 禁止：Tag 命名空间随意新增 — 理由：顶级命名空间控制在 12 个以内，新增需架构评审。否则几年后 `Skill.Fire` 和 `Ability.Fire` 同时存在，直接灾难。

---

## 5. 流程定义

### 5.1 标签注册

- **输入**：标签定义（TagId、父标签 Id、可选描述）
- **处理**：
  1. 校验 TagId 全局唯一性（不变量 3.1）
  2. 校验父标签是否存在（不变量 3.4）
  3. 校验层级是否形成循环引用（不变量 3.2）
  4. 校验命名空间一致性（不变量 3.5）
  5. 注册标签到层级树
  6. 递归构建父子关系的位掩码映射
- **输出**：注册成功确认 或 TagHierarchyError
- **失败处理**：校验不通过时注册失败，输出具体错误原因，不破坏已有标签树

### 5.2 标签查询（TagQuery）

- **输入**：查询条件（Any/All/None 模式）、待查标签集
- **处理**：
  1. Any 模式：检查待查标签集中是否存在至少一个匹配目标标签
  2. All 模式：检查待查标签集中是否所有标签都匹配目标标签
  3. None 模式：检查待查标签集中是否没有任何标签匹配目标标签
  4. 所有模式均考虑层级包含关系（父标签匹配自动包含子标签）
- **输出**：布尔值（匹配/不匹配）
- **失败处理**：被查询的标签 ID 不存在时返回 false，并记录警告日志

### 5.3 标签授予与移除

- **输入**：
  - 授予：目标实体、授予的标签列表
  - 移除：目标实体、移除的标签列表
- **处理**：
  1. 检查待授予/移除的标签是否已注册（不变量 3.4）
  2. 授予：将标签加入实体的 GameTagContainer
  3. 移除：将标签从实体的 GameTagContainer 移除
  4. 如果授予/移除的是父标签，自动推导所有子标签的同步状态
- **输出**：GameTagContainer 变更确认
- **失败处理**：引用未注册标签时授予/移除失败，返回 TagLookupError

### 5.4 层级同步

- **输入**：父标签的授予或移除事件
- **处理**：
  1. 检测到父标签被授予实体
  2. 检查该实体现有 GameTagContainer 中是否有冲突标签（排除法）
  3. 标记该实体"拥有该父标签"的位
  4. 查询时，父标签的位掩码自动覆盖所有子标签
- **输出**：隐式层级状态更新
- **失败处理**：同步过程中数据不一致时触发完整重扫（Recalculation），保证最终一致性

### 5.5 Effect Tag 过滤

EffectDef 通过 Tag 字段实现条件过滤，参考 UE GameplayEffect 模式：

- **granted_tags**：Effect 应用时授予目标实体的标签（如 `tag_status_burning`）
- **required_tags**：目标必须拥有的标签，否则 Effect 应用失败（如治疗效果要求目标未死亡）
- **ignored_tags**：目标不能拥有的标签，否则 Effect 应用失败（用于免疫检查，如火焰免疫实体不受火焰伤害）
- **removed_tags**：Effect 移除时清理的标签
- **remove_effects_with_tags**：应用此 Effect 时，移除目标上具有这些标签的其他效果（如解毒剂移除所有 poison 标签的效果）

### 5.6 Ability Tag 过滤

Ability 通过 Tag 字段实现能力分类与交互控制（参考 UE GameplayAbility）：

- **ability_tags**：标识 Ability 自身类型（如 `tag_skill_type.fireball`）
- **cancel_abilities_with_tags**：激活时取消具有这些标签的其他 Ability
- **block_abilities_with_tags**：激活期间阻断具有这些标签的 Ability
- **activation_owned_tags**：激活期间授予自身的标签

### 5.7 TagQuery in Condition

Condition 系统通过 `TagMatch` 变体直接使用 `TagQuery` 进行多标签匹配：

- **TagRequirement**：单标签检查（Has/Not），基于字符串比较
- **TagMatch**：多标签 + 层级继承检查，基于位掩码 O(1) 评估

评估时需要 `ConditionContext` 提供 `tag_bits`（实体的标签位掩码）和 `tag_masks`（TagId → BitMask 映射）。

---

## 9. Tag vs Type 决策指南（五层架构版）

### 核心原则

> **参与规则计算的东西，必须强类型（Type System）。**
> **用于筛选/分类/内容驱动的东西，可以用 Tag（Tag System）。**

### 决策矩阵

| 问题 | 答案 | 层级 |
|------|------|------|
| 会参与数值计算吗？ | → **Type**（enum） | Type System |
| 会影响核心规则吗？ | → **Type**（enum） | Type System |
| 会被频繁重构吗？ | → **Type**（enum） | Type System |
| 编译期必须保证正确？ | → **Type**（enum） | Type System |
| 用于筛选/分类？ | → **Tag** | Tag System |
| 不影响核心规则正确性？ | → **Tag** | Tag System |
| 可以容忍"写错名字"？ | → **Tag** | Tag System |
| 内容团队需要配置？ | → **Tag** | Tag System |
| Mod 需要扩展？ | → **Tag** | Tag System |

### 速查表

```
# Type System — 强类型（参与规则计算）
DamageType          → Type（参与伤害公式）
UnitClass           → Type（影响职业规则）
AbilityState        → Type（状态机）
EffectStage         → Type（状态机）
MovementType        → Type（参与移动规则）
TerrainType         → Type（参与地形计算）
Passability         → Type（参与路径规划）
FormulaType         → Type（计算逻辑）
ModifierOp          → Type（数值修改操作）

# Tag System — 语义描述（不影响规则）
Boss                → Tag（语义描述）
Undead              → Tag（语义描述）
Flying              → Tag（语义描述）
Ability.Fire        → Tag（火焰主题技能）
Ability.Healing     → Tag（治疗主题）
Equipment.Slot.MainHand → Tag（装备槽位）
Effect.Category.Buff → Tag（增益分类）
Quest.Type.Main     → Tag（主线任务）
```

### 五层架构映射

```
Type System:  DamageType, AbilityState, EffectStage, MovementType, ...
Tag System:   Boss, Undead, Flying, Ability.Fire, Effect.Buff, ...
Query System: TagQuery(Any, [Fire, Ice]), Condition::TagMatch, ...
Rule System:  Rule { condition: TagMatch(...), effect: Modifier(...) }, ...
Content:      RON 配置文件, Mod 扩展
```

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| TagAdded | 标签被授予实体时 | entity_id, tag_id, source_context | Condition（条件检查）、Trigger（触发检测）、Event（事件路由）、日志（LogCode: TAG001） |
| TagRemoved | 标签从实体移除时 | entity_id, tag_id, source_context | Condition、Trigger、日志（LogCode: TAG002） |
| TagHierarchyChanged | 标签层级结构发生变更时（仅开发期/内容加载期） | parent_tag_id, affected_child_ids | Tag 同步系统、内容校验器、日志（LogCode: TAG003） |
| TagQueryEvaluated | TagQuery 条件评估完成时（调试用，仅 dev-tools） | query_type, target_tags, result | 日志分析器、调试工具、日志（LogCode: TAG004） |

### 事件订阅关系图

```
TagAdded/TagRemoved
    │
    ├──→ Condition 系统：检查免疫/限制条件（如 Tag.Immune.Fire 有变化）
    ├──→ Trigger 系统：检测技能触发条件（如 Tag.CombatState.InCombat 被授予）
    ├──→ Event 系统：路由相关事件（如 Tag.StatusEffect.Poisoned 被授予）
    └──→ Cue 系统：触发表现信号（如状态图标变化）
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：Tag 能力领域位于 `core/capabilities/tag/`，数据定义归 foundation/，规则组件归 mechanism/，符合 C1→C2 分层
- ✅ 术语一致：TagId、TagSet、TagHierarchy、TagQuery 与架构文档第六节完全一致
- ✅ 职责明确：Tag 只做标识与分类，不涉及业务逻辑，不越界到 Condition/Effect 领域
- ✅ ECS 边界：GameTagContainer 是 ECS Component，但其规则（标签生命周期/层级同步）属于机制层 C2，非 ECS 实现细节
- ✅ LocalizationKey：Tag 使用 LocalizationKey 而非硬编码文本（宪法 §22）

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖标签注册、层级管理、查询匹配、实体关联等场景
- [x] 所有不变量和约束条件已识别（5 条不变量）
- [x] 禁止事项已明确列出（5 条禁止）
- [x] 每个操作有完整的流程定义（注册、查询、授予/移除、层级同步）
