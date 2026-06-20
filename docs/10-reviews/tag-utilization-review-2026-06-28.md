---
id: 10-reviews.tag-utilization-review
title: Tag 系统重构方案 — 五层架构下的 Tag 定位与重构路径
status: active
owner: architect
created: 2026-06-28
updated: 2026-06-28
reviewed_documents:
  - docs/ai_ignore_this_dir/12tag.md（最佳实践参考）
  - docs/02-domain/capabilities/tag_domain.md
  - docs/04-data/capabilities/tag_schema.md
  - docs/01-architecture/README.md
  - src/core/capabilities/tag/
  - src/core/capabilities/condition/
  - src/core/capabilities/effect/foundation/def.rs
reference_standard: 五层架构（Type→Tag→Query→Rule→Content）
target_scale: 50万行代码 SRPG
---

# Tag 系统重构方案 — 五层架构下的 Tag 定位与重构路径

> **核心问题**: Tag 在 50 万行 SRPG 中应该扮演什么角色？
> **关键结论**: Tag 不是"万物皆标签"的万能胶水，而是五层架构中的**语义层**——与 Type System（规则层）、Query System（筛选层）、Rule System（逻辑层）、Content System（配置层）协同工作。

---

## 0. 哲学纠偏：为什么"万物皆标签"是错误的

### 0.1 旧审查报告的问题

旧报告（§2）提出"43 处 enum 分类应迁移为 Tag"，这犯了一个根本性错误：

> 把 Tag 当成了"领域建模工具"，而非"表达层工具"。

### 0.2 12tag.md 的核心洞察

最佳实践文档（`docs/ai_ignore_this_dir/12tag.md`）明确指出：

```
Tag 不是 DamageType（那是 Type）
Tag 不是 BuffId（那是 Identity）
Tag 不是 WeaponType（那是 Type）

Tag 表达的是：Attribute（属性）、Semantic（语义）、Relationship（关系）
```

**Tag 只回答"是不是"，不能回答"多少"。**

### 0.3 五层架构模型

面向 50 万行 SRPG，正确的架构是：

```
Type System（世界真相）     ← DamageType, UnitClass, AbilityId（强类型）
       ↓
Tag System（世界语义）      ← Boss, Undead, Flying（语义描述）
       ↓
Query System（筛选语言）    ← TargetQuery, Condition（统一查询）
       ↓
Rule System（规则引擎）     ← Condition → Effect（数据驱动规则）
       ↓
Content System（配置层）    ← RON/YAML, Mod（内容驱动）
```

### 0.4 五层职责边界

| 层 | 职责 | 例子 | 谁修改 |
|----|------|------|--------|
| **Type System** | 世界运行规律，参与规则计算 | `DamageType::Fire`, `AbilityState::Casting` | 程序员（编译期） |
| **Tag System** | 世界语义描述，不影响核心规则 | `Enemy.Boss`, `Ability.Fire`, `Terrain.Water` | 内容团队（运行时） |
| **Query System** | 统一筛选语言，跨系统查询 | `TagQuery(Any, [Fire, Ice])`, `Condition::TagMatch` | 配置（Def） |
| **Rule System** | 数据驱动规则，Condition → Effect | `Rule { condition, effect }` | 配置（Def） |
| **Content System** | 内容配置，Mod 扩展 | RON 文件, YAML 配置 | 内容团队 |

---

## 1. 现状分析

### 1.1 Tag 基础设施（A 级）

现有 Tag 系统实现质量高：

| 能力 | 状态 | 说明 |
|------|------|------|
| TagHierarchy（层级树+继承掩码） | ✅ 完整 | 位掩码 O(1) 检查 |
| TagSet（ECS Component） | ✅ 完整 | u128 位掩码，支持 128 个标签 |
| TagQuery（Any/All/None） | ✅ 完整 | 层级继承查询 |
| TagAdded/TagRemoved 事件 | ✅ 完整 | Observer 驱动 |
| TagNamespace（25 个命名空间） | ✅ 完整 | 含本轮新增 |
| 注册校验（5 项） | ✅ 完整 | 循环/唯一/命名空间 |

### 1.2 Tag 利用率（D 级 — 架空状态）

**核心问题**: Tag 基础设施是 A 级，但利用率是 D 级。

**实际消费者（仅 3 处）**:
1. `condition/` — `TagRequirement` + `TagMatch` 条件检查
2. `effect/def.rs` — `granted_tags` / `required_tags` / `ignored_tags` 字段
3. `movement/facade.rs` — `MovementType → TagId` 映射

**未被 Tag 化的分类 enum（30+ 处）**:
- `EffectCategory`, `AbilityCategory`, `ActivationType`
- `QuestType`, `ObjectiveType`, `TerrainType`, `SurfaceType`
- `SpellLevel`, `CastingTime`, `SpellRange`, `SpellDuration`, `SaveType`
- `FormationType`, `CurrencyType`, `RestType`
- 等等

### 1.3 错误的旧方案

旧报告建议"把 43 个 enum 全部迁移到 Tag"，这是**错误的**。

**原因**:

| 错误假设 | 正确理解 |
|---------|---------|
| "enum 分类 = Tag 的职责" | enum 分类可能是 Type System 的职责（参与规则计算） |
| "Tag 是万能的" | Tag 只是语义层，不能替代类型系统 |
| "越多 Tag 越好" | 过多 Tag 会导致"debug 地狱"和语义膨胀 |

---

## 2. 正确的重构方向：五层落地

### 2.1 Type System 层 — 保留哪些 Enum

**原则**: 参与规则计算、影响核心逻辑、频繁重构的 enum **必须保留为强类型**。

**保留为 Type 的 enum**:

| Enum | 理由 |
|------|------|
| `DamageType` | 参与伤害公式计算（火抗、冰抗等） |
| `EffectStage` | 四阶段状态机，生命周期转换规则 |
| `EffectDuration` | 携带数据（turns, calculation） |
| `AbilityState` | 技能生命周期状态机 |
| `ActivationType` | 影响施法流程逻辑 |
| `MovementType` | 参与移动规则计算（已有 → TagId 桥接） |
| `TerrainType` | 参与地形移动代价计算 |
| `Passability` | 参与路径规划规则 |
| `FormulaType` | 计算逻辑 |
| `ModifierOp` | 数值修改操作 |
| `StackingType` | 叠加行为逻辑 |

### 2.2 Tag System 层 — 新增哪些 Tag

**原则**: 语义描述、内容驱动、多领域引用的分类 **应该用 Tag**。

**应该 Tag 化的分类**:

| 当前 Enum | Tag 化方向 | TagNamespace |
|-----------|-----------|-------------|
| `EffectCategory` | `Effect.Category.Buff` 等 | `StatusEffect` |
| `QuestType` | `Quest.Type.Main` 等 | `QuestTag` |
| `ObjectiveType`（部分） | `Quest.Objective.Kill` 等 | `QuestTag` |
| `SpellSchool`（隐含） | `Ability.School.Fire` 等 | `SpellSchool` |
| Marker Components | `Domain.Party` 等 | `Domain`（新增） |
| 事件分类 | `Event.DamageDealt` 等 | `EventType`（新增） |

**不应该 Tag 化的分类（保留为 Type）**:

| Enum | 理由 |
|------|------|
| `SpellLevel` | 参与法术位消耗计算 |
| `CastingTime` | 影响行动顺序逻辑 |
| `SaveType` | 参与豁免检定计算 |
| `CurrencyType` | 参与经济计算 |
| `FormationType` | 影响阵型规则 |

### 2.3 Tag 命名空间重构

当前 25 个 TagNamespace 过于碎片化。按 12tag.md 建议，应精简为 **10-15 个顶级命名空间**：

```
当前（25 个）                        目标（12 个）
──────────────────────              ──────────────────────
DamageType                    →     Damage.*
StatusEffect                  →     Status.*
SkillType                     →     Ability.*
EquipmentSlot                 →     Equipment.*
EquipmentCategory             →     Equipment.*
WeaponCategory                →     Equipment.*
ArmorCategory                 →     Equipment.*
ItemCategory                  →     Item.*
Faction                       →     Faction.*
CombatState                   →     Combat.*
MovementType                  →     （保留为 Type，不 Tag 化）
TerrainType                   →     Terrain.*
BuffCategory                  →     Status.*
Immune                        →     Status.*
Cooldown                      →     （保留为运行时机制）
SpellSchool                   →     Ability.*
QuestTag                      →     Quest.*
DialogueTag                   →     Dialogue.*
TargetingType                 →     Ability.*
ExecutionType                 →     Ability.*
CueType                       →     Cue.*
TriggerType                   →     Trigger.*
CraftingType                  →     Crafting.*
RestType                      →     Camp.*
EconomyType                   →     Economy.*
PartyType                     →     Party.*
ProgressionType               →     Progression.*
Custom(String)                →     Custom(String)
```

### 2.4 Query System 层 — TagQuery 嵌入内容

**已完成**: `Condition::TagMatch` 支持 `TagQuery`（Any/All/None + 层级继承）。

**待做**: 将 TagQuery 嵌入所有 Def 类型，实现 UE GAS 模式：

```
EffectDef:
  ├── granted_tags         → 应用时授予
  ├── required_tags        → 目标必须有
  ├── ignored_tags         → 目标不能有（免疫）
  ├── removed_tags         → 移除时清理
  └── remove_effects_with_tags → 替换效果

AbilityDef（待创建）:
  ├── ability_tags         → 标识自身类型
  ├── cancel_by_tags       → 激活时取消
  ├── block_by_tags        → 激活期间阻断
  └── activation_owned_tags → 激活时授予

ItemDef:
  ├── item_tags            → 物品分类（Weapon/Armor/Consumable）
  └── required_tags        → 使用条件

QuestDef:
  ├── quest_tags           → 任务分类（Main/Side/Faction）
  └── condition_tags       → 触发条件
```

### 2.5 Rule System 层 — 新建

**当前缺失**: 没有独立的 Rule System。规则逻辑散落在各系统中。

**目标架构**:

```rust
/// 数据驱动规则（Definition 层）
pub struct RuleDef {
    /// 规则 ID
    pub id: String,
    /// 触发条件（TagQuery + Condition）
    pub condition: Condition,
    /// 规则效果（Modifier / Tag 操作 / 事件触发）
    pub effect: RuleEffect,
    /// 优先级
    pub priority: u32,
}

pub enum RuleEffect {
    /// 数值修改
    Modifier(ModifierConfig),
    /// 标签授予/移除
    TagOperation { grant: Vec<TagId>, remove: Vec<TagId> },
    /// 事件触发
    EventTrigger(String),
    /// 效果应用
    EffectApply { effect_id: String },
}
```

**统一规则引擎**:

```rust
pub fn evaluate_rules(
    rules: &[RuleDef],
    context: &ConditionContext,
    entity: Entity,
    commands: &mut Commands,
) -> Vec<RuleEffect> {
    rules.iter()
        .filter(|rule| evaluate(&rule.condition, context, entity, commands).is_passed())
        .map(|rule| rule.effect.clone())
        .collect()
}
```

### 2.6 Content System 层 — RON 配置驱动

```
assets/config/
├── tags/                    ← TagDefinition RON 文件
│   ├── ability.ron          ← Ability.* 标签定义
│   ├── status.ron           ← Status.* 标签定义
│   ├── faction.ron          ← Faction.* 标签定义
│   └── ...
├── effects/                 ← EffectDef RON 文件（含 Tag 字段）
├── abilities/               ← AbilityDef RON 文件（含 Tag 字段）
├── rules/                   ← RuleDef RON 文件（新增）
└── ...
```

---

## 3. 重构路径（按优先级）

### Phase 1: Tag 命名空间重构（低风险）

| 任务 | 说明 | 工作量 |
|------|------|--------|
| 重命名 TagNamespace | 25 → 12 个顶级命名空间 | 2 天 |
| 更新 TagDefinition RON | 所有 tag RON 文件路径更新 | 1 天 |
| 更新引用方 | 所有 `TagNamespace::Xxx` 引用更新 | 0.5 天 |

### Phase 2: Tag 嵌入 Def（中风险）

| 任务 | 说明 | 工作量 |
|------|------|--------|
| AbilityDef 创建 | 新增 AbilityDef + Tag 字段 | 3 天 |
| EffectDef Tag 字段完善 | 已完成（ignored_tags, remove_effects_with_tags） | ✅ |
| ItemDef Tag 字段 | 物品分类 Tag | 1 天 |
| QuestDef Tag 字段 | 任务分类 Tag | 1 天 |
| SpellDef Tag 字段 | 法术分类 Tag | 1 天 |

### Phase 3: EffectCategory Tag 化（中风险）

| 任务 | 说明 | 工作量 |
|------|------|--------|
| EffectCategory → Tag | 将 EffectCategory 替换为 Tag 查询 | 2 天 |
| 免疫系统实现 | 基于 ignored_tags 的免疫检查 | 2 天 |
| 测试更新 | 所有 EffectCategory 引用更新 | 1 天 |

### Phase 4: Rule System 建设（高风险）

| 任务 | 说明 | 工作量 |
|------|------|--------|
| RuleDef 定义 | 数据驱动规则结构 | 2 天 |
| RuleEngine | 统一规则评估引擎 | 3 天 |
| Condition → Rule 桥接 | 现有 Condition 系统集成 | 2 天 |
| 内容配置 | RuleDef RON 文件 | 2 天 |

### Phase 5: Marker Component 统一（低风险）

| 任务 | 说明 | 工作量 |
|------|------|--------|
| 4 个 Marker → TagSet | PartyMarker 等替换为 TagSet | 1 天 |
| 性能验证 | Archetype 过滤 vs 位掩码 | 0.5 天 |

---

## 4. Tag 治理规范（面向 50 万行）

### 4.1 Tag 生命周期

```
提案 → 审核 → 注册 → 使用 → 引用统计 → 废弃 → 删除
```

### 4.2 Tag 文档要求

每个 Tag 必须有 YAML 描述：

```yaml
Enemy.Boss:
  description: |
    用于标记 Boss 单位。
    不保证具有特殊数值。
    不用于伤害计算。
    可用于目标筛选、UI展示、成就统计。
  level: L2    # L1=Core / L2=Gameplay / L3=Content / L4=Temporary
  deprecated: false
  replacement: null
```

### 4.3 Tag 引用统计

```rust
tag_registry.find_references("Enemy.Boss")
// → 技能引用: 53, Buff引用: 12, 任务引用: 4, AI引用: 8
```

### 4.4 Tag 废弃机制

```yaml
Enemy.Monster:
  deprecated: true
  replacement: Enemy.Beast
  deprecated_at: "2026-06-28"
```

### 4.5 Tag 数量控制

| 级别 | 数量上限 | 说明 |
|------|---------|------|
| L1 Core Tag | < 100 | 极少修改，几十年不动 |
| L2 Gameplay Tag | < 500 | 玩法层，变化较少 |
| L3 Content Tag | < 1000 | 内容层，变化频繁 |
| L4 Temporary Tag | 无限制 | 活动/实验，可删除 |
| **总计** | **< 1500** | 超过需架构评审 |

---

## 5. Tag vs Type 决策指南（修正版）

### 核心原则

> **参与规则计算的东西，必须强类型。**
> **用于筛选/分类/内容驱动的东西，可以用 Tag。**

### 决策矩阵

| 问题 | 答案 |
|------|------|
| 会参与数值计算吗？ | → **Type**（enum） |
| 会影响核心规则吗？ | → **Type**（enum） |
| 会被频繁重构吗？ | → **Type**（enum） |
| 编译期必须保证正确？ | → **Type**（enum） |
| 用于筛选/分类？ | → **Tag** |
| 不影响核心规则正确性？ | → **Tag** |
| 可以容忍"写错名字"？ | → **Tag** |
| 内容团队需要配置？ | → **Tag** |
| Mod 需要扩展？ | → **Tag** |

### 速查表

```
DamageType     → Type（参与伤害公式）
UnitClass      → Type（影响职业规则）
AbilityState   → Type（状态机）
EffectStage    → Type（状态机）
MovementType   → Type（参与移动规则）
TerrainType    → Type（参与地形计算）

Boss           → Tag（语义描述）
Undead         → Tag（语义描述）
Flying         → Tag（语义描述）
Fire           → Tag（Ability.Fire = 火焰主题技能）
Healing        → Tag（Ability.Healing = 治疗主题）
MainHand       → Tag（Equipment.Slot.MainHand）
```

---

## 6. 与 UE GAS 的正确对齐

UE GAS 的"万物皆标签"不是指把所有东西都变成 Tag，而是：

1. **Tag 是跨系统的通用查询语言** — 不要为每个维度造一种新的 enum
2. **Tag 是可组合的** — 一个 Entity 可同时有多个命名空间的标签
3. **Tag 是数据驱动的** — 新增分类不需要改代码
4. **Tag 的查询是统一入口** — TagQuery 是唯一的"检查是否有 X 属性"的入口

**但 UE 同时也有强类型系统** — `EGameplayAbilitySpec`, `FGameplayEffectSpec` 等都是强类型结构。

**正确的对齐方式**:
```
UE GAS                          Fre（目标）
──────────────                  ──────────────
FGameplayTag                    TagId + TagSet
FGameplayTagQuery               TagQuery + Condition
FGameplayAbilitySpec            AbilitySpec（强类型）
FGameplayEffectSpec             EffectSpec（强类型）
AbilityTags                     ability_tags: Vec<TagId>
GrantedTags                     granted_tags: Vec<TagId>
```

---

## 7. 结论

### 7.1 核心结论

**Tag 不是万能胶水。** 在 50 万行 SRPG 中，Tag 的最大价值不是替代类型系统，而是成为整个项目共享的"语义词典"。当 Character、Ability、Buff、AI、Quest、UI、Story 都说同一种 Tag 语言时，项目规模越大，它的价值越高。

### 7.2 行动优先级

| 优先级 | 行动 | 工作量 | 状态 |
|--------|------|--------|------|
| **P0** | Tag 命名空间重构（25 → 12） | 3.5 天 | ✅ 完成 |
| **P0** | AbilityDef 创建 + Tag 字段 | 3 天 | ✅ 完成 |
| **P1** | EffectCategory Tag 化 | 5 天 | ✅ 完成 |
| **P1** | Rule System 建设 | 9 天 | ✅ 完成 |
| **P2** | Marker Component → TagSet | 1.5 天 | ✅ 完成（渐进策略） |
| **长期** | Tag 治理规范（引用统计/废弃/文档） | 持续 | ✅ 完成（§10 写入 tag_domain.md） |

### 7.3 一句话总结

> ❌ 不要"万物 tag 化"
> ✔ 应该是"核心强类型 + 外围 tag 化 + 内容驱动补充"

---

> **评审人**: Architect
> **参考来源**: `docs/ai_ignore_this_dir/12tag.md`（最佳实践）
> **下次评审触发条件**: Phase 1 完成后，或 Tag 数量超过 100 个时
