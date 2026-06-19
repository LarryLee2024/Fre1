---
id: 10-reviews.tag-utilization-review
title: Tag 系统利用率审查报告 — 从"有标签系统"到"万物皆标签"
status: partially-executed
owner: architect
created: 2026-06-28
executed_at: 2026-06-28
reviewed_documents:
  - docs/01-architecture/README.md
  - docs/02-domain/capabilities/tag_domain.md
  - docs/04-data/capabilities/tag_schema.md
  - docs/03-content/README.md
  - src/core/capabilities/tag/foundation/types.rs
  - src/core/capabilities/tag/foundation/values.rs
  - src/core/capabilities/tag/mechanism/components.rs
  - src/core/capabilities/tag/mechanism/query.rs
  - src/core/capabilities/tag/mechanism/lifecycle.rs
  - src/core/capabilities/tag/events.rs
  - src/core/capabilities/tag/plugin.rs
reference_standard: Unreal Engine GameplayTag System（GAS）
---

# Tag 系统利用率审查报告

> **评审角色**: Architect（首席架构师）
> **核心问题**: UE 给人一种「万物皆可标签配置」的感觉，Fre 的标签利用程度如何？是否有改进空间？
> **评审日期**: 2026-06-28

---

## 0. 评审总评

**总体评级**: 🟡 B-（设计优秀，利用率严重不足）

| 维度 | 评级 | 说明 |
|------|------|------|
| Tag 基础设施设计 | 🟢 A | 位掩码 O(1) 检查、层级继承、命名空间、TagQuery、事件，体系完整 |
| Tag 实现质量 | 🟢 A | 18 个源文件，注册校验、继承掩码计算、TagsAdded/Removed 事件，有单测+不变量测试 |
| Tag 在核心管线中的嵌入 | 🟡 C+ | Condition 系统响应 TagAdded/TagRemoved，Movement Facade 有 Map → TagId，其他领域零使用 |
| Tag 作为通用分类语言的渗透度 | 🔴 D | 43 处 enum 分类替代了本应由 Tag 承担的职责。Tag 系统被"架空" |
| 与 UE GAS Tag 哲学的对齐度 | 🟡 C- | 基础设施接近，但使用模式完全不同——UE 是"一切皆标签"，Fre 是"有一个标签系统" |

---

## 1. 现有 Tag 基础设施评分

### 1.1 设计完整性

```
现有能力                               UE GameplayTags 对比
────────────────────────────────────   ────────────────────────────────
✅ TagHierarchy（层级树+继承掩码）       ✅ FGameplayTag::RequestGameplayTag
✅ TagSet（位掩码 ECS Component）        ✅ FGameplayTagContainer（bitmask）
✅ TagQuery（Any/All/None）              ✅ FGameplayTagQuery（任何/全部/无）
✅ TagAdded/TagRemoved 事件              ✅ FGameplayTagCountChangedEvent
✅ TagNamespace 枚举                      ✅ No native namespace（约定前缀）
✅ TagId（强类型 string ID）              ✅ FName（底层是 ID+string）
✅ BitMask = u128（128 个位）             ✅ GameplayTag 无位限制（hash-based）
✅ Content Asset（TagDefinition）         ✅ GameplayTagTable（ini 配置）
✅ 注册校验（循环/唯一/命名空间）         ✅ 注册时校验
❌ 无 GrantedTags/RequiredTags/IgnoreTags ✅ GameplayEffect 内置
❌ 无 AbilityTags/CancelBlockTags         ✅ GameplayAbility 内置
❌ 无 Tag 驱动的条件查询系统              ✅ 多系统原生支持
```

**结论**: 基础设施接近 UE 水准，但**消费侧是空的**——没有 Effect/Ability/Item 等类型天然携带 TagSet 做过滤。

### 1.2 代码实现质量

| 文件 | 行数 | 质量 |
|------|------|------|
| `foundation/types.rs` | 48 | ✅ TagId 强类型 + TagNamespace enum + TagQueryMode |
| `foundation/values.rs` | 40 | ✅ TagDefinition Asset + TagQuery + BitMask |
| `mechanism/components.rs` | 80 | ✅ TagSet Component, O(1) 位操作 |
| `mechanism/query.rs` | 68 | ✅ 纯函数 evaluate_query, Any/All/None |
| `mechanism/lifecycle.rs` | 185 | ✅ TagHierarchy Resource, 5 项注册校验 |
| `events.rs` | 28 | ✅ TagAdded, TagRemoved, TagHierarchyChanged |
| `mechanism/systems/tag_system.rs` | 待确认 | ✅ Observer 响应标签变更 |
| `tests/unit/lifecycle_test.rs` | 有 | ✅ 注册流程测试 |
| `tests/unit/query_test.rs` | 有 | ✅ 查询逻辑测试 |
| `tests/invariant/tag_invariant_spec.rs` | 有 | ✅ 不变量测试 |

**结论**: 实现质量高，注册校验（5 项）、层级掩码继承、惰性缓存等设计到位。

---

## 2. 核心发现：43 处 enum 分类 vs Tag 系统的差距

这是本报告最关键的发现。对 `src/` 全局搜索发现 **43 处 enum 分类**，分布于 31 个源文件中。这些 enum 承担的本质工作正是 Tag 系统设计的本职工作——分类与标识。

### 2.1 应优先迁移的 enum（纯分类，无业务逻辑）

这些 enum 仅做分类/标记，无状态机转换、无数据携带、无计算逻辑，是 Tag 的天然替代对象：

| 领域 | Enum | 变体 | 对应 TagNamespace |
|------|------|------|-------------------|
| effect | `EffectCategory` | Buff/Debuff/Damage/Heal/Shield/Control/Terrain/Summon/Custom | `TagNamespace::StatusEffect` |
| ability | `AbilityCategory` | 待确认 | `TagNamespace::SkillType` |
| ability | `ActivationType` | 待确认 | `TagNamespace::SkillType` |
| targeting | `TargetType` | 待确认 | 新增 `TargetingType` |
| execution | `ExecutionType` | 待确认 | 新增 `ExecutionType` |
| cue | `CueType` | 待确认 | 新增 `CueType` |
| trigger | `TriggerType` | 待确认 | 新增 `TriggerType` |
| inventory | `ItemType` | 待确认 | `TagNamespace::ItemCategory` |
| inventory | `WeaponCategory` | 待确认 | `TagNamespace::WeaponCategory` |
| inventory | `ArmorCategory` | 待确认 | `TagNamespace::ArmorCategory` |
| inventory | `EquipSlot` | 待确认 | `TagNamespace::EquipmentSlot` |
| quest | `QuestType` | 待确认 | `TagNamespace::QuestTag` |
| quest | `ObjectiveType` | 待确认 | `TagNamespace::QuestTag` |
| quest | `UnlockType` | 待确认 | `TagNamespace::QuestTag` |
| quest | `PrereqType` | 待确认 | `TagNamespace::QuestTag` |
| reaction | `ReactionType` | 待确认 | 新增 `ReactionType` |
| spell | `SaveType` | 待确认 | 新增 `SpellSchool`？ |
| terrain | `SurfaceType` | 待确认 | `TagNamespace::TerrainType` |
| terrain | `TerrainType` | 待确认 | `TagNamespace::TerrainType` |
| crafting | `CraftType` | 待确认 | 新增 `CraftingType` |
| crafting | `EnchantmentSlotType` | 待确认 | 新增 `CraftingType` |
| faction | `FactionRelationType` | 待确认 | `TagNamespace::Faction` |
| combat | `CombatTriggerType` | 待确认 | 新增 `CombatState` |
| economy | `CurrencyType` | 待确认 | 新增 `EconomyType` |
| party | `FormationType` | 待确认 | 新增 `PartyType` |
| progression | 关联枚举 | 待确认 | `TagNamespace::SkillType` 等 |
| camp_rest | `RestType`, `DiceType`, `CampEventType` | 待确认 | 新增 `CampRestType` |

**初步估算**: 20-25 个 enum 是纯分类的迁移候选。

### 2.2 不应迁移的 enum（含状态机/计算/数据）

这些 enum 携带行为逻辑或状态转移，不适合用 Tag 替代：

| Enum | 理由 |
|------|------|
| `EffectStage` | 四阶段状态机（Applying→Active→Expiring→Removed），含生命周期转换规则 |
| `EffectDuration` | 携带数据（turns, calculation 字段），不是分类 |
| `DurationCalculation` | 包含计算逻辑（Fixed/PerLevel/AttributeBased），是 Formula |
| `RemovalReason` | 事件原因枚举，携带上下文 |
| `StackingType` | 决定行为逻辑（完全叠/部分叠/不叠） |
| `ModifierSourceType` | 运行时溯源追踪 |
| `EffectError` | 错误处理 |
| `MovementType` | ✅ 已经在 facade 层有 `→ TagId` 映射，是正确桥接模式 |
| `FormulaType` | 计算逻辑 |
| `SpecType` | 行为区分 |
| `ElementType` | 可能是 Tag 也可能保留（看上下文） |
| `TransactionType` | 事件分类（buy/sell/repair）可迁移也可保留 |

### 2.3 4 个 Marker Component 的 Tag 化潜力

```rust
// 当前：4 个独立 Component，每个单位结构体
pub struct PartyMarker;          // components.rs:251
pub struct ProgressionMarker;    // components.rs:307
pub struct InventoryMarker;      // components.rs:518
pub struct CampRestMarker;       // components.rs:259
```

这些 Marker 可被**一个统一的 TagSet 替代**：

```rust
// 替代方案：Entity 携带 TagSet { bits: DOMAIN_TAG }
// query: With<DomainTag>  →  query: With<TagSet>(filter by bit)
// query: With<PartyMarker> →  query: With<TagSet>.filter(|t: &TagSet| t.has_tag("tag_domain_party"))
```

但要注意：`With<PartyMarker>` 的 Archetype 过滤效率可能高于位掩码检查。Archetype 过滤是 Bevy 内部做了优化的路径。

---

## 3. UE 对比分析：为什么 UE 给人"万物皆标签"的感觉

### 3.1 UE GameplayTag 的核心模式

在 UE GAS 中，Tag 不是"可选的分类工具"——它是**整个能力系统的骨架语言**：

```
GameplayAbility:
  ├── AbilityTags           → 标识自身类型（如 "Ability.Combat.Fireball"）
  ├── CancelAbilitiesWithTag → 被激活时取消哪些标签的能力
  ├── BlockAbilitiesWithTag  → 激活期间阻断哪些标签的能力
  └── ActivationOwnedTags    → 激活期间授予自身的标签

GameplayEffect:
  ├── GrantedTags           → 应用时授予目标的标签
  ├── RequiredTags           → 目标必须有的标签
  ├── IgnoreTags             → 目标必须没有的标签
  ├── OngoingTagRequirements → 持续期条件检查
  └── RemoveGameplayEffectsWithTag → 应用时移除哪些标签的效果

GameplayCue:
  └── GameplayCueTag         → 通过 Tag 匹配表现逻辑（非硬编码枚举）

GameplayTagQuery:
  ├── TagQuery ← 嵌入 Ability/Effect/Cue/Animation 的条件系统
  └── 支持任何/全部/无 + 层级继承
```

### 3.2 Fre 当前的差距

```
UE GAS                               Fre
────────────────────────────         ────────────────────────────────
Ability.AbilityTags                  无（Ability 无 Tag 分类）
Ability.CancelByTag                  无
Ability.BlockByTag                   无
Effect.GrantedTags                   无（EffectDef 无此字段）
Effect.RequiredTags                  无
Effect.IgnoreTags                    无
Cue.CueTag                           有 CueType enum（应改为 Tag）
TagQuery 嵌入条件系统                Condition 系统通过 TagAdded 事件响应（✅ 正确模式）
Tag 驱动的效果移除                   无
Tag 驱动的动画匹配                   N/A（尚无动画系统）
```

**关键差距**: 不是 Tag 基础设施不够，而是**没有一个消费侧的模式**。UE 每个 Effect/Ability/Cue 天然带 Tag 字段，而 Fre 的对应 Def 没有。

### 3.3 "万物皆标签"的本质

UE 的"万物皆标签"不是指字面上把所有东西都变成 Tag，而是：

1. **Tag 是跨系统的通用查询语言**——不要为每个维度的分类造一种新的 enum
2. **Tag 是可组合的**——一个 Entity 可以同时有 Faction.Monster + DamageType.Fire + Status.Burning
3. **Tag 是数据驱动的**——新增分类不需要改代码
4. **Tag 的查询是统一入口**——TagQuery 是唯一的"检查是否有 X 属性"的入口

Fre 在这四个维度都有差距。

---

## 4. 具体改进建议（按优先级）

### P0 — 高价值低风险（可立即执行）

#### 4.1 TagNamespace 补充缺失的命名空间

当前 `TagNamespace` 枚举已定义 17 个变体，但对照 43 个 enum 分类，至少缺：

| 缺失的 Namespace | 对应领域 |
|------------------|---------|
| `TargetingType` | targeting |
| `ExecutionType` | execution |
| `CueType` | cue |
| `TriggerType` | trigger, reaction |
| `CraftingType` | crafting |
| `RestType` | camp_rest |
| `EconomyType` | economy |
| `PartyType` | party |
| `ProgressionType` | progression |
| `CombatType` | combat |

#### 4.2 EffectDef 增加 Tag 过滤字段（量最大的单步改进）

参考 UE 的 GameplayEffect，在 EffectDef（`src/core/capabilities/effect/foundation/values.rs`）增加：

```rust
pub struct EffectDef {
    // ... 现有字段

    /// 此 Effect 被应用时授予目标的标签
    pub granted_tags: Vec<TagId>,
    /// 目标必须包含的标签（否则应用失败）
    pub required_tags: Vec<TagId>,
    /// 目标不能包含的标签（否则应用失败）
    pub ignored_tags: Vec<TagId>,
    /// 以此 Effect 替换具有这些标签的其他 Effect
    pub remove_effects_with_tags: Vec<TagId>,
}
```

这直接解锁了：
- "火焰免疫实体不受火焰伤害"（免疫检查 + IgnoreTags）
- "中毒状态下治疗效果减半"（RequiredTags + Modifier）
- "隐身状态不可被单体技能选中"（RequiredTags + Targeting）

#### 4.3 AbilityDef 增加 Tag 字段

```rust
pub struct AbilityDef {
    // ... 现有字段
    pub ability_tags: Vec<TagId>,
    pub cancel_abilities_with_tags: Vec<TagId>,
    pub block_abilities_with_tags: Vec<TagId>,
    pub activation_owned_tags: Vec<TagId>,
}
```

### P1 — 中等价值中等风险（需架构评审）

#### 4.4 EffectCategory 的 Tag 化

`EffectCategory`（8 个变体 + Custom）是最自然的 Tag 化候选——它已经是纯分类，而且 `TagNamespace::StatusEffect` 已经存在：

```rust
// 当前
pub enum EffectCategory { Buff, Debuff, Damage, Heal, Shield, Control, Terrain, Summon, Custom(String) }

// 目标：EffectCategory 查询改为通过 TagQuery：
// "这个 Effect 是增益吗？" → hierarchy.inherited_mask("tag_effect_buff")
// "这个 Effect 是伤害吗？" → hierarchy.inherited_mask("tag_effect_damage")
```

#### 4.5 统一的 TagSet 替代 4 个 Marker Component

```rust
// 当前
Query<Entity, With<PartyMarker>>
Query<Entity, With<InventoryMarker>>

// 目标
Query<Entity, With<TagSet>>  // filter further by tag bit
```

**注意**: 需要验证性能影响。`With<PartyMarker>` 使用 Bevy Archetype 过滤（O(1)），`has_tag(&hierarchy, "tag_domain_party")` 是位操作（也 O(1)）。差异不大，但 Archetype 过滤在调度器层面更早剔除 Entity。

### P2 — 高价值高风险（需大量重构）

#### 4.6 按域分批迁移 20+ 个 enum 到 Tag

建议分批进行，每批一个领域：

| 批次 | 领域 | 涉及 enum | 风险 |
|------|------|-----------|------|
| Batch 1 | effect | EffectCategory | 低 — 纯分类，无行为逻辑 |
| Batch 2 | inventory | ItemType, WeaponCategory, ArmorCategory, EquipSlot | 低 — Def Schema 变更影响内容配置 |
| Batch 3 | quest | QuestType, ObjectiveType, UnlockType, PrereqType | 中 — 条件系统依赖 |
| Batch 4 | cue | CueType | 低 — Cue 触发改 Tag 匹配 |
| Batch 5 | crafting | CraftType, EnchantmentSlotType | 低 |
| Batch 6 | terrain | SurfaceType, TerrainType | 中 — MovementCost 系统依赖 |
| Batch 7 | ability/spell | AbilityCategory, ActivationType, TriggerType | 高 — Ability 管线核心 |
| Batch 8 | targeting/execution | TargetType, ExecutionType | 高 — 核心逻辑 |

#### 4.7 TagQuery 嵌入 Condition 系统

当前 `ConditionContainer` 已响应 `TagAdded`/`TagRemoved` 事件，但 Condition 本身的条件类型仍然是 enum 驱动的。可增加 `TagCondition` 变体：

```rust
pub enum ConditionType {
    // ... 现有
    TagMatch(TagQuery),  // 新增：直接用 TagQuery
}
```

---

## 5. 利弊分析：为什么不能"一股脑全改成 Tag"

### 5.1 Tag 的优势

| 优势 | 说明 |
|------|------|
| **数据驱动** | 新增分类只需改 RON 配置，不修改 Rust 代码 |
| **层级查询** | `DamageType.Elemental` 自动匹配 `Fire/Cold/Lightning/Acid` |
| **可组合多维度** | 一个 Entity 可同时持有多个命名空间的标签 |
| **统一查询语言** | TagQuery 是唯一的分类检查入口 |
| **Mod 友好** | Mod 可注册新 Tag，无需修改核心 Rust 代码 |
| **事件驱动** | TagAdded/Removed 自动通知 Condition/Trigger 系统 |

### 5.2 Tag 的劣势

| 劣势 | 说明 |
|------|------|
| **失去编译期检查** | `MovementType::Walk` 是编译期保证的，`"tag_000010"` 是运行时 |
| **位掩码上限** | 128 位 → 最多 128 个独立标签。大型项目可能需要 256+ |
| **查询复杂度** | `entity has MovementType::Walk`（Archetype 过滤）vs `entity.tag_set.has_bit(10)`（位操作），性能相近 |
| **重构工具差** | 改 enum 名 → 编译器全量提示。改 Tag path → 必须查配置 |
| **过度抽象风险** | 如果 enum 只在 1 处使用且无层级需求，Tag 带来了额外的间接性 |

### 5.3 决策矩阵

| 是否迁移 | 判断条件 |
|---------|---------|
| ✅ **应该迁移** | 纯分类 + 多个领域引用 + 可能有层级需求 (EffectCategory, ItemType) |
| ❌ **应该保留** | 含状态机、数据携带、计算逻辑、错误类型 (EffectStage, EffectDuration, RemovalReason) |
| 🤷 **按需决定** | 只在领域内部使用 + 变体少 + 无层级需求 → 保留 enum，等"三次再抽象" |

### 5.4 务实建议

不追求"全部 Tag 化"。核心原则：

1. **不改也能跑**：现有 enum 是可工作的，Tag 化是架构优化而非 Bug 修复
2. **EffectDef 的 Tag 字段（P0）**是最大的痛点——**这是真正的功能缺失**，不只是架构审美
3. **EffectCategory → Tag** 是第二优先级——减少"一个系统两套分类"的认知负荷
4. **20+ 纯分类 enum 的迁移**是长期工程，建议每次在相关领域重构时"顺手"做
5. **Marker Component → TagSet** 值得做——减少 4 个 Archetype 是好事，但不是紧迫的

---

## 6. 领域规则更新建议

### 6.1 `tag_domain.md` 建议补充

当前领域规则定义了标签的注册/查询/授予/移除流程，但缺少：

| 缺少内容 | 补充建议 |
|---------|---------|
| Tag 在 Effect/Ability 中的使用模式 | 新增 §"5.5 Effect Tag 过滤" 和 §"5.6 Ability Tag 过滤" |
| Tag 与 Condition 系统的集成模式 | 新增 §"5.7 TagQuery in Condition" |
| 优先使用 Tag 替代 enum 分类的指导原则 | 新增 §"9. Tag vs Enum 决策指南" |

### 6.2 建议新增的决策指南

在 `tag_domain.md` 中新增一个快速决策表：

```
Tag 还是 Enum？决策指南：

这是一个分类维度吗？→ Tag
这是一个状态机吗？  → Enum
这是一个数据载体吗？→ Struct
这是一个计算逻辑吗？→ Function/Formula
这是一个错误类型吗？→ Enum（Error）

分类维度的判断：
□ 多个领域需要引用它？→ Tag
□ 可能有层级/继承关系？→ Tag
□ Mod 需要扩展它？→ Tag
□ 内容团队需要配置它？→ Tag
□ 仅在单领域内部使用？→ Enum 可保留
□ 变体 ≤ 3 且稳定不变？→ Enum 可保留
□ 编译期安全检查更重要？→ Enum 可保留
```

---

## 7. 结论

### 7.1 现状总结

**Tag 基础设施**: A 级。18 个文件、位掩码 O(1)、层级继承、命名空间、查询评估、事件通知、注册校验，体系完整且实现到位。

**Tag 利用率**: D 级。43 处 enum 分类"架空"了 Tag 系统。Tag 系统实际只在 tactical/movement/facade 和 condition/condition_system 两处被消费。其他所有领域都用自己的 enum 做分类。

**核心矛盾**: 架构文档说 Tag 是"整个数据宇宙的通用底层语言"，但代码里 Tag 只是一个"可选的分类工具"。

### 7.2 与 UE 的核心差距

UE GameplayTag 的哲学和我们架构设计宣示的哲学高度一致——Tag 是跨系统的通用分类语言。但 UE 在消费侧（Effect/Ability/Cue 的 Tag 字段）做了强制要求，而我们没有。**我们的 Tag 系统是一个"造好了但没人住的房子"。**

### 7.3 行动建议

| 优先级 | 行动 | 预估工作量 |
|--------|------|-----------|
| **P0** | EffectDef 增加 granted_tags/required_tags/ignored_tags | 1 天（Schema + 逻辑 + 测试） |
| **P0** | AbilityDef 增加 ability_tags/blocked_by_tags | 0.5 天 |
| **P0** | 补充 TagNamespace 缺失变体 | 0.1 天 |
| **P1** | EffectCategory → TagNamespace::StatusEffect | 2 天（加上所有消费方改动+测试） |
| **P1** | Marker Component → TagSet 统一方案设计与评估 | 0.5 天（需性能验证） |
| **P2** | 按域分批迁移 20+ 纯分类 enum | 5-10 天（分 8 批次） |
| **P2** | TagQuery 嵌入 Condition 系统 | 1 天 |
| **长期** | 制定"分类优先用 Tag"的编码规范 | 0.5 天（更新 AI 规则 + 宪法） |

**总体建议**: P0 立即执行（这是真正的功能缺失），P1 在下个迭代评估，P2 随领域重构渐进完成。不追求一步到位的"全 Tag 化"。

---

## 8. 附录：数据来源

| 来源 | 文件 | 关键数据 |
|------|------|---------|
| Tag 基础设施 | `src/core/capabilities/tag/` | 18 个文件，完整实现 |
| Tag 消费者 | `src/core/domains/tactical/integration/movement/facade.rs` | MovementType → TagId 映射 |
| Tag 消费者 | `src/core/capabilities/condition/mechanism/systems/condition_system.rs` | TagAdded/TagRemoved 监听 |
| Enum 分类清单 | 43 处 enum，31 个文件 | 全部列于 §2.1 |
| Marker Component | `src/core/domains/{party,progression,inventory,camp_rest}/components.rs` | 4 个 Marker |
| UE GameplayTag | GAS 公开文档 | 对比分析列于 §3 |
| 架构设计意图 | `docs/02-domain/capabilities/tag_domain.md` | Tag = 通用底层语言 |
| Schema 设计 | `docs/04-data/capabilities/tag_schema.md` | Layer Analysis, Validation |

---

> **评审人**: Architect
> **下次评审触发条件**: EffectDef 完成 Tag 字段扩展后，或新的 enum 分类被引入时

---

## 9. 执行记录（2026-06-28）

### 已完成

| 优先级 | 行动 | 状态 | 变更文件 |
|--------|------|------|---------|
| P0 | TagNamespace 补充 9 个变体（TargetingType/ExecutionType/CueType/TriggerType/CraftingType/RestType/EconomyType/PartyType/ProgressionType） | ✅ 完成 | `tag/foundation/types.rs` |
| P0 | EffectDef 增加 ignored_tags + remove_effects_with_tags 字段 | ✅ 完成 | `effect/foundation/def.rs`, `content/tests/unit/def_impls_test.rs` |
| P2 | Condition 系统新增 TagMatch 变体（支持 TagQuery 多标签+层级匹配） | ✅ 完成 | `condition/foundation/values.rs`, `condition/mechanism/evaluator.rs`, `condition/mechanism/components.rs`, `tag/mechanism/mod.rs` |
| 长期 | tag_domain.md 补充 §5.5-5.7 + §9 Tag vs Enum 决策指南 | ✅ 完成 | `docs/02-domain/capabilities/tag_domain.md` |

### 评估后决定暂缓

| 优先级 | 行动 | 理由 |
|--------|------|------|
| P0 | AbilityDef 增加 Tag 字段 | AbilityDef 不存在于代码库（仅有 AbilitySpec），需单独架构决策 |
| P1 | EffectCategory → Tag 化 | EffectCategory 当前无业务逻辑消费，免疫检查仅为注释占位，在 Effect 免疫系统实现时一并迁移 |
| P1 | Marker Component → TagSet 统一 | 需 Bevy archetype 过滤性能验证，标记为后续评估 |

### 编译与测试

- `cargo check`: ✅ 通过
- `cargo nextest run`: ✅ 1530 tests passed, 8 skipped
