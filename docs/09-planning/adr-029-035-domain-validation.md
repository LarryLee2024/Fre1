---
id: 09-planning.adr-029-035-domain-validation
title: ADR-029~035 领域规则验证报告
status: draft
owner: domain-designer
created: 2026-06-15
updated: 2026-06-15
tags:
  - domain
  - validation
  - adr
---

# ADR-029~035 领域规则验证报告

验证人：@domain-designer
验证日期：2026-06-15
基线：`docs/02-domain/` 全部 39 份领域规则文件

## 总结裁决

| ADR | 裁决 | 关键冲突数 | 待处理项 |
|-----|------|-----------|---------|
| ADR-029（总纲） | ✅ **PASS** | 0 | 无 |
| ADR-030（ID/Registry） | ⚠️ **CONDITIONAL PASS** | 1 | `define_id!` 宏名称需确认 |
| ADR-031（Attribute/Tag） | ❌ **FAIL** | 2 | TagCategory 10→4 冲突, AttributeKind 未定义 |
| ADR-032（Effect Pipeline） | ✅ **PASS** | 0 | 无（领域层已废弃 Buff） |
| ADR-033（Ability/Trigger） | ✅ **PASS** | 0 | 无（领域层已迁移到 Ability） |
| ADR-034（Cue/Replay/I18n） | ⚠️ **CONDITIONAL PASS** | 1 | Replay 域与 ADR 的 Cue 定位需细微对齐 |
| ADR-035（清理计划） | ✅ **PASS** | 0 | 无 |

---

## 1. ADR-029 数据模型完全重构总纲

**裁决：✅ PASS**

**验证项** | **结果**
--- | ---
是否与现有领域规则冲突？ | 无冲突。总纲定义的 13 领域覆盖范围与 `docs/02-domain/README.md` 的 Core Domain 21 文件完全对应。
术语是否对齐？ | ✅ "Effect→Execution→Modifier→Attribute→Tag" 链与 domain README 中 execution-rules.md 的定位链完全一致。
是否有遗漏领域概念？ | 无遗漏。总纲引用的领域结构映射到了现有的领域文件集。

**备注**：总纲的 5 阶段策略与领域规则无冲突，每个 phase 都有对应的领域文件支撑。

---

## 2. ADR-030 ID系统与Registry基础设施重构

**裁决：⚠️ CONDITIONAL PASS**

**验证项** | **结果**
--- | ---
`define_id!` 宏是否存在命名冲突？ | ❌ 领域层无此命名，但需确认宏名称是否与 Bevy 或其他库的命名规范一致。当前领域文件使用 `UnitId`、`SkillId`（现为 `AbilityId`）等强类型 ID 概念，与 ADR 方向一致。
Registry 统一 trait | 领域层的 `UnitTemplateRegistry`、`TerrainRegistry`、`SelectorRegistry`（已过时）、`AbilityRegistry` 等概念均存在，统一化方向正确。
重复 ID 删除 | 领域层不引用重复 ID 问题（代码层问题）。领域层面无冲突。

**条件**：确认 `define_id!` 宏名称在 Rust edition 2024 命名规范下无保留字冲突（建议在实现前咨询 @data-architect）。

---

## 3. ADR-031 核心属性与标签系统重构

**裁决：❌ FAIL — 2 个关键冲突需要解决**

### 冲突 1：TagCategory 10→4 缩减

**领域现状**（`docs/02-domain/tag/tag-rules.md` §TagCategory）：
- 当前定义 **10 个分类维度**：Element / Status / Weapon / WeaponType / Class / Movement / SkillType / BuffType / ItemType / EquipmentAttribute
- 37/64 bits 已使用
- 每个标签属于且仅属于一个分类

**ADR-031 提案**：缩减为 **4 类**（Elemental / Status / Class / Meta）

**分析**：这是 ADR 中最严重的领域冲突。
- Weapon, WeaponType, ItemType, EquipmentAttribute 这 4 个分类在领域规则中用于装备/物品的标签分类。如果合并到 Class 或 Meta，装备类的标签查询将失去分类粒度。
- BuffType 在领域规则中已被 ADR-026 废弃（Buff → Effect + Duration），但仍存在于 `tag-rules.md` 中。
- Movement 分类用于寻路和 AI 决策，AI 领域规则依赖此分类。

**建议方案**：
1. 采纳 ADR-031 的 4 类缩减方向，但需要将 Weapon/WeaponType/ItemType/EquipmentAttribute 合并为 `TagCategory::Equipment`（而非 Class/Meta），保留装备标签查询接口
2. 将 Movement 合并到 Status
3. 将 BuffType 删除（与 ADR-026 一致）
4. 最终方案应为 **5 类**：Elemental / Status / Class / Equipment / Meta，而非 4 类

**建议修改 ADR-031 或更新 tag-rules.md。**

### 冲突 2：AttributeKind 24→11 但领域层尚无 AttributeKind 定义

**领域现状**：
- `docs/02-domain/attribute/` 目录在 README 中被引用为"新增"但 **文件不存在于磁盘上**
- 当前唯一有定义的属性分类在 `docs/02-domain/character/character-rules.md` 中提及 "8 维核心属性"（base_attributes）
- `attribute-modifier/attribute-modifier-rules.md` 未定义 AttributeKind 枚举

**ADR-031 提案**：AttributeKind 24→11 变体，5 基础 + 6 派生（Linglan 模型）

**分析**：领域层根本没有 AttributeKind 定义——ADR 不是在"修改"领域规则，而是在"创建"它。这不是冲突，而是空白需要填补。

**建议**：
1. 在 `docs/02-domain/attribute/attribute-rules.md` 中正式定义 AttributeKind 枚举（目前 README 已引用但文件不存在），可由 @domain-designer 或 ADR-031 作者补充定义
2. ADR-031 需要明确列出 11 个变体名称及映射关系

---

## 4. ADR-032 Effect管线全链路重构

**裁决：✅ PASS**

**验证项** | **结果**
--- | ---
Buff 模块删除 | ✅ 领域层 `buff/buff-rules.md` 已标注"已废弃"（位于 `_ai_ignore_this_dir/buff/buff-rules(已废弃❌).md`），与 ADR-032 完全一致
7 阶段 Pipeline（Effect→Stacking→Execution→Modifier→Attribute→Tag→Cue） | ✅ 领域层已定义 `effect/effect-rules.md`（一级 Effect 领域）、`execution/execution-rules.md`（Execution trait）、`attribute-modifier/attribute-modifier-rules.md`（Modifier）、`tag/tag-rules.md`、`cue/cue-rules.md`。Stacking 由 `stack-policy/stack-policy-rules.md` 覆盖
Execution trait | ✅ `execution/execution-rules.md` 已定义 `Execution` trait 和 `ExecutionRegistry`，与 ADR-032 方案一致
8 种 StackType | 领域层 `stack-policy/stack-policy-rules.md` 未具体列出 8 种。ADR-032 的 8 种需以此为参考定义具体的堆叠策略

**备注**：领域层在 Effect Pipeline 重构方向上是 ADR-032 的超集——领域层甚至更激进（Buff 已废弃、Cue 已独立）。此 ADR 应在 StackType 具体实现后补充领域定义。

---

## 5. ADR-033 Ability与Trigger系统重构

**裁决：✅ PASS**

**验证项** | **结果**
--- | ---
`Skill*`→`Ability*` 重命名 | ✅ `docs/02-domain/ability/ability-rules.md` 已存在。`skill/skill-rules.md` 已标注"已过时"并位于 `_ai_ignore_this_dir/skill/skill-rules(已废弃❌).md`
5 阶段 Ability 释放管线 | ✅ `ability/ability-rules.md` 定义 "Requirement → Cost → Targeting → Effect → Settlement"，加上 domain README 中的五阶段引用，一致
Trigger 5 大类 20+ 事件 | ✅ `trigger/trigger-rules.md`（894 行）完整覆盖
targeting/resolver.rs 填充 | 领域层 `targeting/targeting-rules.md` 已存在（取代 selector-rules.md）。代码层的 resolver.rs 占位符填充不受领域影响。

---

## 6. ADR-034 Cue与Replay与I18n系统实现

**裁决：⚠️ CONDITIONAL PASS**

**验证项** | **结果**
--- | ---
Cue（表现信号） | ✅ `cue/cue-rules.md` 已定义 GameplayCue 总线，与 ADR-034 的 12 种 Cue 方向一致
Replay（回放系统） | ⚠️ `docs/03-technical/replay-rules.md` 定义了 Command Stream (Track A) / Audit Trail (Track B) 双轨制，ADR-034 的 Replay 方案需要对齐此双轨设计中的 Replay 定位
FTL 国际化 | ✅ `docs/03-technical/localization-rules.md`（982 行）定义了 Fluent (.ftl) 技术选型，与 ADR-034 一致
name_key/desc_key | ✅ `localization-rules.md` 已定义 Key 永久 ID、LocalizedText 组件、语言回退链

**条件**：ADR-034 的 Replay 方案需确认与 `replay-rules.md` 的 Command Stream 模型一致——尤其是"所有战斗 Bug 必须通过 Battle Replay 重现"这一领域不变量。

---

## 7. ADR-035 模块清理与迁移执行计划

**裁决：✅ PASS**

**验证项** | **结果**
--- | ---
文件级删除清单 | 无领域冲突——此为代码层操作
6 阶段门禁 | 无领域层面冲突
RON 41 文件迁移 | 内容加载规范由 `docs/04-data/content-system-rules.md` 定义，ADR 的迁移策略与此一致

---

## 关键冲突汇总（需要解决后方可进入实现）

| 优先级 | 冲突 | 涉及 ADR | 涉及领域文件 | 建议解决方案 |
|--------|------|----------|-------------|-------------|
| ~~🔴 **已解决**~~ | ~~TagCategory 10→4 缩减~~ | ~~ADR-031~~ | ~~`tag/tag-rules.md`~~ | ✅ **已解决** — 改为 5 类（Elemental/Status/Class/Equipment/Mechanism），ADR-031 和 tag-rules.md 均已更新 |
| ~~🔴 **已解决**~~ | ~~AttributeKind 无领域定义~~ | ~~ADR-031~~ | ~~`attribute/` 目录不存在~~ | ✅ **已解决** — ADR-031 自身定义 11 变体（5 Core + 6 Secondary），实现时同步创建 attribute-rules.md |
| 🟡 **中** | Replay 双轨制对齐 | ADR-034 | `replay-rules.md` | ADR-034 确认 Command Stream 兼容性 |
| ⚪ **低** | define_id! 命名规范 | ADR-030 | — | 实现前咨询 data-architect |

---

## 推荐的领域后续更新

1. **`docs/02-domain/attribute/attribute-rules.md`** — 实现时同步创建此文件，内容由 ADR-031 的 11 变体定义驱动
2. **`docs/02-domain/stack-policy/stack-policy-rules.md`** — 补充 8 种 StackType 的具体定义（与 ADR-032 实现同步）
3. **`docs/02-domain/targeting/targeting-rules.md`** — 确认 Resolver 接口定义与 `targeting/resolver.rs` 实现一致（与 ADR-033 实现同步）

其余 ADR（029, 030, 032, 033, 035）在领域层面无阻塞性问题，可以进入下一阶段。
