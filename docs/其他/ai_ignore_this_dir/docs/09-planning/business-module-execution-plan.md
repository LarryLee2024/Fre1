---
id: 09-planning.business-module-execution-plan
title: 业务模块执行计划
status: Draft
owner: architect, domain-designer
created: 2026-06-15
updated: 2026-06-15
tags:
  - planning
  - execution
  - architecture
---

# 业务模块执行计划 v2.0

> **版本说明**：v2.0 基于源码实际扫描结果重写，修正了 v1.0 中多处与真实代码不符的假设。

| 版本 | 日期 | 作者 | 变更 |
|------|------|------|------|
| 1.0 | 2026-06-15 | Architect + Domain-Designer | 初版（含多处错误假设） |
| 2.0 | 2026-06-15 | Architect + Domain-Designer | 源码验证后重写：ADR-025 七领域基本落地，聚焦真实剩余差距 |

---

## 目录

1. [执行摘要](#1-执行摘要)
2. [源码验证的方法与发现](#2-源码验证的方法与发现)
3. [核心纠正：v1.0 中的错误假设](#3-核心纠正v10-中的错误假设)
4. [修正后的 ADR 状态总览](#4-修正后的-adr-状态总览)
5. [架构真实状态评估](#5-架构真实状态评估)
6. [实际剩余差距分析](#6-实际剩余差距分析)
7. [分阶段执行计划](#7-分阶段执行计划)
8. [依赖图与关键路径](#8-依赖图与关键路径)
9. [风险评估](#9-风险评估)

---

## 1. 执行摘要

通过 4 组源码探索任务对 `src/core/` 下 **16 个目录模块**、`src/app/plugin.rs` 的 **28 个 Plugin 注册**、`src/shared/event/` 的 **10 个事件文件**、`content/` 的 **37 个 RON 文件** 进行了全面验证。

### 修正后的核心结论

**ADR-025（七领域模块化架构）已基本落地**，之前分析的很多「缺失」模块实际上已经存在：

| 领域 | 模块状态 | Plugin | 文件数 |
|------|---------|--------|--------|
| Tag | ✅ 存在 | TagPlugin + TagDefPlugin | 2 文件 |
| Modifier | ✅ 存在 | ModifierRulePlugin | 4 文件 |
| Effect | ✅ 存在（EffectDef 已迁入） | EffectPlugin | 3 文件 |
| Buff | ✅ 存在（含 Duration/Stack 策略） | BuffPlugin | 8 文件 |
| Targeting | ✅ 存在 | TargetingPlugin | 4 文件 |
| Trigger | ✅ 存在（ExecutionStack + TriggerRegistry） | TriggerPlugin | 4 文件 |
| Ability | ✅ 存在（取代 skill） | AbilityPlugin | 7 文件 |

**Plugin 注册顺序已严格遵循 DAG**（见 §5.2）。

**内容管线已迁移** — 37 个 RON 文件已在 `content/` 目录，14 个 `load_from_dir` 调用均使用 `"content/..."` 路径。

### 实际剩余工作（远少于 v1.0 估算）

```
P0 —— 架构债务（~3 天）
P1 —— 能力增强（~5 天）
P2 —— 基建补齐（~3 天）
P3 —— 内容整理（~2 天）
P4 —— 质量测试（~5 天）
```

---

## 2. 源码验证的方法与发现

### 2.1 验证范围

| 探索任务 | 覆盖内容 | 发现的关键事实 |
|---------|---------|---------------|
| **模块结构** | 17 个 core 模块的 mod.rs | `skill/` 不存在，已被 `ability/` 取代 |
| **EffectDef 归属** | 全局搜索 EffectDef 定义 + 事件结构体 | EffectDef 已迁入 effect/，事件已在 shared/event/ |
| **Plugin 注册** | app/plugin.rs 全部 28 个 Plugin | 注册顺序完全符合 DAG |
| **内容管线** | content/ 全部目录 + RON 加载代码 | 37 RON 文件在 content/，14 个加载点 |

### 2.2 方法

对每个模块读取 `mod.rs` 确认：
- 目录是否存在
- 子文件有哪些
- 有哪些 Pub 类型
- Plugin 是否存在
- domain/error.rs 是否存在
- def.rs 是否存在

---

## 3. 核心纠正：v1.0 中的错误假设

| 错误假设 | v1.0 断言 | 实际状态 | 影响 |
|---------|-----------|---------|------|
| `modifier/` 模块缺失 | ❌ 不存在 | ✅ `src/core/modifier/` 已存在（4 文件） | 估算从 3-5 天降至 0 |
| `targeting/` 模块缺失 | ❌ 不存在 | ✅ `src/core/targeting/` 已存在（4 文件） | 估算从 2-3 天降至 0 |
| `ability/` 模块缺失 | ❌ 不存在 | ✅ `src/core/ability/` 已存在（完整实现） | 估算从 3-5 天降至 0 |
| `skill/` 仍存在 | ❌ 过度耦合 | ✅ `skill/` 已完全被 `ability/` 取代 | 无需 migration |
| EffectDef 在 skill/ | ❌ 需迁移 | ✅ 已在 `effect/types.rs` | 无需迁移 |
| 事件在 infrastructure/logging/ | ❌ 需迁移 | ✅ 已在 `shared/event/`（10 文件） | 无需迁移 |
| Plugin 顺序未对齐 DAG | ❌ 可能错 | ✅ 严格按 DAG 注册 | 无问题 |
| RON 内容在 assets/ | ❌ 需迁移到 content/ | ✅ 37 个 .ron 已在 content/ | 无问题 |
| Condition 模块缺失 | ✅ 正确 | ❌ `condition/` 确实不存在 | 真正的 gap（但小） |

**v1.0 的 Phase 2（七领域落地）已基本完成**，不需要 2-3 周。

---

## 4. 修正后的 ADR 状态总览

### 4.1 ADR 状态修正

| # | 标题 | 原状态（v1.0） | 修正后状态 | 修正理由 |
|---|------|---------------|-----------|---------|
| 001 | 迁移总计划 | ✅ Accepted | ✅ Accepted | 已执行 |
| 002 | 技术债修复方案 | ✅ Accepted | ✅ Accepted | 已部分执行 |
| 003 | 分层契约与依赖迁移 | ✅ Accepted | ✅ Accepted | 七层已落地 |
| 004 | 内容与数据迁移方案 | ✅ Accepted | ✅ **Complete** | RON 已全在 content/ |
| 005 | 插件与通信迁移方案 | ✅ Accepted | ✅ **Complete** | DAG 已对齐 |
| 006 | 验证与测试迁移方案 | ✅ Accepted | ⚠️ Partial | 有测试但覆盖率待补 |
| 007 | 目录结构迁移映射 | ✅ Accepted | ✅ **Complete** | 目录结构已对齐 |
| 008 | 核心机制与工程质量 | ✅ Accepted | ✅ **Complete** | 核心机制已落地 |
| 009 | 迁移合规修正 | ✅ Accepted | ✅ **Complete** | 已修正 |
| 010 | 测试迁移与重整 | 💡 Proposed | ⚠️ Proposed | 等待资源 |
| 011 | 错误模块实施 | ✅ Accepted | ⚠️ Partial | shared/error 骨架存在，ErrorContext/GameErrorEvent 待实现 |
| 012 | 日志模块与统一事件 | ✅ Accepted | ⚠️ Partial | 事件在 shared/event/ ✅，但 core 层有重复事件定义 |
| 013 | 技能数据模型与配置 | 💡 Proposed | ✅ **Complete** | ability/ 完整实现，RON 在 content/skills/ |
| 014 | 技能释放管线设计 | 💡 Proposed | ✅ **Complete** | ability/pipeline.rs 五阶段管线 |
| 015 | 技能标签与分类体系 | 💡 Proposed | ⚠️ Partial | tag/ 存在但 label() 中文待清理 |
| 016 | 技能系统扩展点设计 | 💡 Proposed | ⚠️ Partial | EffectHandlerRegistry + TargetingResolver 存在，Condition 缺 |
| 017 | 国际化架构决策 | 💡 Proposed | ❌ **Not Implemented** | LocalizationPlugin 存在但未注册到 app |
| 018 | 国际化迁移方案 | 💡 Proposed | ❌ **Not Implemented** | 30+ 处中文硬编码未迁移 |
| 020 | Buff 数据模型与配置 | 💡 Proposed | ✅ **Complete** | BuffDef/BuffData/DurationPolicy/StackPolicy 在 buff/domain/ |
| 021 | Buff 生命周期与持续策略 | 💡 Proposed | ⚠️ Partial | DurationPolicy 部分实现，7 策略验证待做 |
| 022 | Buff 触发系统与事件架构 | 💡 Proposed | ⚠️ Partial | buff/trigger.rs 存在，与 Effect Pipeline 衔接待验证 |
| 023 | 标签系统架构重整 | 💡 Proposed | ⚠️ Partial | tag/ 模块存在，label() 废弃和 RON 扩展待执行 |
| 024 | 标签系统迁移方案 | 💡 Proposed | ⚠️ Partial | RON 已扩展但 label() 全域替换未做 |
| 025 | 七领域模块化架构设计 | 💡 Proposed | ✅ **Complete** | 全部 7 领域模块已实现，Plugin DAG 已对齐 |

### 4.2 修正后的状态分布

```
✅ Complete:   10（+8 从 Proposed 升级）
⚠️ Partial:    10
❌ Not Impl:   2（ADR-017/018 国际化）
💡 Future:     2（ADR-010 测试迁移、ADR-016 扩展点局部）
```

---

## 5. 架构真实状态评估

### 5.1 Core 模块完全清单

| 模块 | 存在 | 文件数 | Plugin | domain/error | def.rs | 关键类型 |
|------|------|--------|--------|-------------|--------|---------|
| **tag/** | ✅ | 2 | TagPlugin + TagDefPlugin | — | ✅ | GameplayTag, TagRegistry, TagCategory |
| **effect/** | ✅ | 3 | EffectPlugin | — | — | EffectDef, EffectHandler, EffectQueue |
| **modifier/** | ✅ | 4 | ModifierRulePlugin | — | — | ModifierRule, ModifierCalculator |
| **attribute/** | ✅ | 3 | AttributeDefPlugin (def.rs) | — | ✅ | Attributes, AttributeKind, ModifierOp |
| **ability/** | ✅ | 7 | AbilityPlugin | ✅ | — | SkillDef, SkillData, SkillRegistry, pipeline |
| **targeting/** | ✅ | 4 | TargetingPlugin | — | — | SkillTargeting, resolver, validator |
| **trigger/** | ✅ | 4 | TriggerPlugin | — | — | Trigger, ExecutionStack, TriggerRegistry |
| **buff/** | ✅ | 8 | BuffPlugin | ✅ | — | BuffDef, DurationPolicy, StackPolicy, ActiveBuffs |
| **battle/** | ✅ | 10+ | BattlePlugin | ✅ | — | CombatIntent, pipeline, record |
| **character/** | ✅ | 8 | CharacterPlugin | — | template.rs | Unit, Faction, TraitCollection, animation |
| **equipment/** | ✅ | 6 | EquipmentPlugin | — | ✅ | EquipmentDef, EquipmentSlots |
| **inventory/** | ✅ | 8 | InventoryPlugin | ✅ (domain) | ✅ | ItemDef, Container, BattleBag |
| **map/** | ✅ | 8 | MapPlugin | — | — | TerrainGrid, OccupancyGrid, GameMap |
| **turn/** | ✅ | 4 | TurnPlugin | — | — | TurnPhase, TurnOrder, AppState |
| **ai/** | ✅ | 7 | AiPlugin | — | — | AiBehavior, strategy selectors |
| **movement/** | ✅ | 2 | **无 Plugin** | — | — | MovementIntent 事件 |
| **campaign/** | ✅ | 6 | CampaignPlugin | — | ✅ | CampaignProgress, CampaignRegistry |
| **condition/** | ❌ | — | — | — | — | 内嵌在 ability 和 buff 中 |

### 5.2 Plugin 注册顺序（实际）

验证自 `src/app/plugin.rs`，注释已按 DAG 分层标注：

```
 0. DefaultPlugins + EguiPlugin
 1. ContentPlugin (stub — 暂无实际加载)
── Layer 1: 无依赖基础设施 ──────────
 2. TagPlugin, TagDefPlugin, AttributeDefPlugin
── Layer 2: 依赖 tag ────────────────
 3. ModifierRulePlugin
── Layer 3: 依赖 tag + modifier ─────
 4. EffectPlugin
── Layer 4: 3 平行，依赖 effect ─────
 5. BuffPlugin, TargetingPlugin, TriggerPlugin
── Layer 5: 依赖所有下层 ────────────
 6. AbilityPlugin
── Layer 6: 其他数据 Plugin ─────────
 7. AiBehaviorPlugin, EquipmentPlugin, InventoryPlugin
── Infrastructure ────────────────────
 8. LogPlugin, AuditPlugin
── Game Logic ────────────────────────
 9. AssetsPlugin, TurnPlugin, MapPlugin, CharacterPlugin, BattlePlugin, AiPlugin
10. CampaignPlugin
── Presentation ──────────────────────
11. UiPlugin（含 11 个子 Plugin）, InputPlugin
12. DebugPlugin
```

**结论**：完全符合 ADR-025 DAG。无注册顺序问题。

### 5.3 内容管线状态（已验证）

**RON 文件分布**：

| 目录 | 文件数 | 加载方式 | 加载方 |
|------|--------|---------|--------|
| `content/definitions/` | 2 | load_from_file | tag/def.rs, ai/ (attribute def) |
| `content/skills/` | 6 | load_from_dir | ability/mod.rs |
| `content/buffs/` | 8 | load_from_dir | buff/mod.rs |
| `content/characters/` | 6 | load_from_dir | character/template.rs |
| `content/classes/` | 5 | load_from_dir | character/traits/ |
| `content/ai_behaviors/` | 3 | load_from_dir | ai/behavior.rs |
| `content/equipments/` | **0** (目录不存在) | load_from_dir | equipment/mod.rs (回退 default) |
| `content/items/` | **0** (目录不存在) | load_from_dir | inventory/mod.rs (回退 default) |
| `content/modifiers/` | 1 | load_from_dir_vec | modifier/mod.rs |
| `content/terrains/` | 4 | load_from_dir | map/data.rs |
| `content/stages/` | 1 | load_from_dir | map/data.rs |
| `content/campaigns/` | 1 | (直接加载) | campaign/loader.rs |

**问题**：`content/equipments/` 和 `content/items/` 目录在磁盘上不存在，代码调用 `load_from_dir` 后回退到 `register_defaults()`。这是数据缺失，不是代码问题。

---

## 6. 实际剩余差距分析

### 6.1 P0 — 架构债务（3 天）

| # | 问题 | 严重度 | 描述 | 修复方案 | 工时 |
|---|------|--------|------|---------|------|
| 1 | **重复事件定义** | 🔴 P0 | core 层 Entity 版事件（`core/battle/events.rs` 等）与 `shared/event/` 版 Strong ID 事件重复 | 统一到 `shared/event/`，core 层改用 shared 版本 | 1-2 天 |
| 2 | **LocalizationPlugin 未注册** | 🔴 P0 | 完整实现的国际化模块被 "orphan"，游戏无本地化 | 在 app/plugin.rs 注册 LocalizationPlugin | 0.5 天 |
| 3 | **SharedPlugin 未注册** | 🔴 P0 | SharedPlugin 存在但未注册 | 在 app/plugin.rs 注册 SharedPlugin | 0.5 天 |
| 4 | **ContentPlugin stub** | 🟡 P1 | 14 个 `load_from_dir` 散落在各 core Plugin，未集中 | 逐步将加载逻辑收拢到 ContentPlugin | 1-2 天 |

### 6.2 P1 — 能力增强（5 天）

| # | 问题 | 严重度 | 描述 | 修复方案 | 工时 |
|---|------|--------|------|---------|------|
| 5 | **`label()` 中文硬编码** | 🟡 P1 | `GameplayTag::label()` 返回硬编码中文，违反国际化 | 改为从 RON 获取 display_name，移除硬编码 | 1 天 |
| 6 | **TagCategory 扩展** | 🟡 P1 | 仅 6 个变体，缺少 Movement/ItemType/WeaponType | 按 ADR-023 扩展 | 1 天 |
| 7 | **Condition 模块缺失** | 🟡 P1 | 无独立 condition/ 模块，条件逻辑散落 | 创建 `core/condition/` 模块 | 1-2 天 |
| 8 | **Buff DurationPolicy 验证** | 🟡 P1 | DurationPolicy 存在但需逐个验证 7 种策略 | 代码审查 + 单元测试 | 1 天 |
| 9 | **Buff Trigger→Effect Pipeline 衔接** | 🟡 P1 | buff/trigger.rs 存在，但需验证衔接完整性 | 代码审查 + 集成测试 | 1 天 |

### 6.3 P2 — 基建补齐（3 天）

| # | 问题 | 严重度 | 描述 | 修复方案 | 工时 |
|---|------|--------|------|---------|------|
| 10 | **ErrorContext trait** | 🟡 P2 | ADR-011 要求的 ErrorContext 未实现 | 在 shared/error/ 实现 | 1 天 |
| 11 | **GameErrorEvent 事件通道** | 🟡 P2 | ADR-011 要求的统一错误事件通道 | 在 shared/event/ 添加 infra 事件 | 1 天 |
| 12 | **LogIfError trait** | 🟡 P2 | 辅助 trait，自动记录 error | 在 shared/error/ 实现 | 0.5 天 |
| 13 | **bevy::log → tracing 验证** | 🟡 P2 | 确认 infrastructure/logging/ 全部使用 tracing | grep 验证 | 0.5 天 |

### 6.4 P3 — 内容整理（2 天）

| # | 问题 | 严重度 | 描述 | 修复方案 | 工时 |
|---|------|--------|------|---------|------|
| 14 | **`content/equipments/` 缺失** | 🟡 P3 | 目录不存在，装备使用硬编码 default | 从 assets/ 迁移或创建 RON | 1 天 |
| 15 | **`content/items/` 缺失** | 🟡 P3 | 同上 | 从 assets/ 迁移或创建 RON | 1 天 |

### 6.5 P4 — 质量测试（5 天）

| # | 问题 | 严重度 | 描述 | 修复方案 | 工时 |
|---|------|--------|------|---------|------|
| 16 | **核心规则测试覆盖率** | 🟡 P4 | 单元测试覆盖不足 | 补充 Effect Pipeline、Modifier 计算、Buff 生命周期测试 | 3 天 |
| 17 | **集成测试** | 🟡 P4 | 跨模块交互测试 | 添加 Battle Scenario 测试 | 2 天 |

---

## 7. 分阶段执行计划

### Phase 1：架构债务清理 🔴（3 天）

| # | 任务 | 交付物 | 工时 |
|---|------|--------|------|
| P1.1 | **注册 SharedPlugin** | app/plugin.rs 加一行 `.add_plugins(SharedPlugin)` | 0.5 天 |
| P1.2 | **注册 LocalizationPlugin** | app/plugin.rs 加一行 `.add_plugins(LocalizationPlugin)` | 0.5 天 |
| P1.3 | **统一重复事件**：core 层 Entity 事件 → shared/event/ Strong ID 事件 | 删除 core/battle/events.rs 中的重复定义，统一使用 shared 版本 | 1-2 天 |
| P1.4 | **ContentPlugin 初步收拢** | 将 1-2 个模块的加载（如 tag/def, modifier/）搬入 ContentPlugin | 1 天 |

**验证**：`cargo check` + 运行时功能无退化。

### Phase 2：能力增强 🟡（5 天）

| # | 任务 | 交付物 | 工时 |
|---|------|--------|------|
| P2.1 | **label() 中文硬编码清理** | 所有中文字符串从 RON display_name 获取 | 1 天 |
| P2.2 | **TagCategory 扩展** | 补充 Movement/ItemType/WeaponType 等 + RON 元数据同步 | 1 天 |
| P2.3 | **创建 `condition/` 模块** | 从 ability 和 buff 中提取 Condition 逻辑到独立模块 | 1-2 天 |
| P2.4 | **Buff DurationPolicy 验证 + 测试** | 7 种策略枚举验证、补充单元测试 | 1 天 |
| P2.5 | **Buff Trigger→Effect Pipeline 验证** | 审查衔接路径，补充集成测试 | 1 天 |

**验证**：`cargo test` 全部通过。

### Phase 3：基建补齐 🟡（3 天）

| # | 任务 | 交付物 | 工时 |
|---|------|--------|------|
| P3.1 | **ErrorContext trait 实现** | 在 `shared/error/` 添加 ErrorContext | 1 天 |
| P3.2 | **GameErrorEvent 事件通道** | 在 `shared/event/infra.rs` 添加事件定义 + 注册 | 1 天 |
| P3.3 | **LogIfError trait + tracing 审查** | 实现 LogIfError；检查所有日志是否为 tracing | 1 天 |

**验证**：`cargo check`。

### Phase 4：内容整理 🟢（2 天）

| # | 任务 | 交付物 | 工时 |
|---|------|--------|------|
| P4.1 | **创建 `content/equipments/` RON 文件** | 从 `assets/equipment/` 迁移或创建默认值 | 1 天 |
| P4.2 | **创建 `content/items/` RON 文件** | 从 `assets/items/` 迁移或创建默认值 | 1 天 |

**验证**：运行时加载日志无回退 fallback 警告。

### Phase 5：质量测试 🟢（5 天）

| # | 任务 | 交付物 | 工时 |
|---|------|--------|------|
| P5.1 | **Effect Pipeline 单元测试** | Generate→Modify→Execute 全链路测试 | 1 天 |
| P5.2 | **Modifier 计算测试** | Flat/Percent/Override 三种操作测试 | 1 天 |
| P5.3 | **Buff 生命周期测试** | 施加→Tick→过期测试 + Stack/Duration 策略测试 | 1 天 |
| P5.4 | **Battle Scenario 集成测试** | 完整战斗回合测试（移动→攻击→技能→Buff→死亡） | 2 天 |

**验证**：`cargo test` 全部通过 + 手动运行游戏无异常。

---

## 8. 依赖图与关键路径

```
Phase 1: 架构债务（3 天）— 关键路径 🔴
  ├── P1.1 SharedPlugin 注册    (0.5d)
  ├── P1.2 LocalizationPlugin   (0.5d)
  ├── P1.3 重复事件统一          (1-2d)  ← 最长子任务
  └── P1.4 ContentPlugin 初步    (1d)
       │
       ▼
Phase 2: 能力增强（5 天）
  ├── P2.1 label() 清理         (1d)
  ├── P2.2 TagCategory 扩展     (1d)
  ├── P2.3 condition/ 模块      (1-2d)
  ├── P2.4 Buff Duration 验证   (1d)
  └── P2.5 Buff Trigger 验证    (1d)
       │
       ▼
Phase 3: 基建补齐（3 天）  ← 可并行
  ├── P3.1 ErrorContext         (1d)
  ├── P3.2 GameErrorEvent       (1d)
  └── P3.3 LogIfError + tracing (1d)

Phase 4: 内容整理（2 天）  ← 可并行
  ├── P4.1 equipments/ RON     (1d)
  └── P4.2 items/ RON          (1d)

Phase 5: 质量测试（5 天）  ← 依赖 Phase 1-2
  ├── P5.1 Effect Pipeline 测试 (1d)
  ├── P5.2 Modifier 测试        (1d)
  ├── P5.3 Buff 生命周期测试    (1d)
  └── P5.4 Battle Scenario 测试 (2d)
```

**关键路径**：Phase 1 → Phase 2 → Phase 5，约 13 天。

Phase 3（基建）和 Phase 4（内容）可独立于关键路径并行执行，不增加总工期。

---

## 9. 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| 重复事件统一导致编译中断 | 中 | 高 | 逐文件替换，保留旧定义直至替换完成 |
| LocalizationPlugin 启用后 UI 异常 | 低 | 中 | 先用最小配置启用，观察 UI 行为 |
| ContentPlugin 收拢加载破坏加载顺序 | 中 | 高 | 渐进式迁移，每次只移一个类型 |
| condition/ 模块拆分破坏引用 | 中 | 中 | 先创建新模块 + 重导出，稳定后删除旧路径 |
| 测试发现隐藏的逻辑错误 | 中 | 高 | 测试先行策略——先发现就早修复 |

---

## 附录 A：源码验证证据

所有验证结果基于 2026-06-15 对 `src/` 和 `content/` 的实际文件扫描。

### A.1 Core 模块 mod.rs 声明

`src/core/mod.rs` 声明（精确顺序）：
```rust
pub mod tag;
pub mod attribute;
pub mod effect;
pub mod modifier;
pub mod ability;
pub mod targeting;
pub mod trigger;
pub mod buff;
pub mod map;
pub mod equipment;
pub mod inventory;
pub mod movement;
pub mod character;
pub mod battle;
pub mod ai;
pub mod turn;
pub mod campaign;
```

### A.2 app/plugin.rs Plugin 注册顺序

精确行号引用：`src/app/plugin.rs` 的 `AppPlugin::build()` 方法。

### A.3 shared/event/ 事件文件

| 文件 | 事件类型 |
|------|---------|
| `battle.rs` | DamageDealt, CharacterDied, HealApplied, StunApplied, DotApplied, HotApplied |
| `buff.rs` | BuffApplied, BuffRemoved |
| `skill.rs` | SkillActivated |
| `turn.rs` | TurnStarted, TurnEnded |
| `equipment.rs` | EquipmentEquipped, EquipmentUnequipped |
| `campaign.rs` | LevelCompleted |
| `character.rs` | UnitMoved |
| `inventory.rs` | ItemUsed, ItemTransferred |
| `infra.rs` | ConfigLoaded, SnapshotCreated |
| `mod.rs` | pub mod 声明 |

### A.4 RON 文件分布

`content/` 下 37 个 .ron 文件，`assets/` 下仅 1 个（settings.ron）。
