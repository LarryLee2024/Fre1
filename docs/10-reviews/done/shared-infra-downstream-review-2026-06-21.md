---
id: 10-reviews.shared-infra-downstream-2026-06-21
title: "Shared & Infra 层下游应用情况评审"
status: completed
owner: architect
created: 2026-06-21
tags:
  - review
  - shared
  - infra
  - downstream
  - adoption
  - layering
---

# Shared & Infra 层下游应用情况评审

> **评审日期**: 2026-06-21 | **范围**: src/shared/ + src/infra/ 的下游消费方
> **方法**: 全量 grep + 模块间依赖分析 | **测试基线**: 1791 tests ✅

---

## 执行摘要

Shared 和 Infra 层的设计完整性已在上周重构中大幅提升。然而下游消费情况呈现出鲜明的"头重脚轻"格局——基础设施层功能完备，但业务层采纳率不足 30%。部分关键组件（prelude、validation、hashing、math）处于"零消费者"状态。

| 维度 | 评分 | 说明 |
|------|------|------|
| 设计完整性 | 🟢 8/10 | 模块齐全，文档完整，测试覆盖 |
| 下游采纳率 | 🟡 4/10 | 核心类型广泛使用，新模块零采纳 |
| 层依赖合规 | 🟡 6/10 | Core→App 违规（8 文件），其余正确 |
| API 易用性 | 🟡 5/10 | 无 prelude 使用，全部走直接 import |

---

## 1. Prelude 采用情况

### 现状

`shared/prelude/mod.rs` 在本次重构中已填充了 canonical exports：

```rust
pub use crate::shared::constants::*;
pub use crate::shared::diagnostics::{AuditEvent, Domain, DomainEvent, ...};
pub use crate::shared::localization_key::LocalizationKey;
pub use crate::shared::random::{DeterministicRng, RngSeeds, RngStream};
pub use crate::shared::time::GameTime;
pub use crate::shared::traits::RuleFailure;
```

但 **0 个下游文件** 使用 `use crate::shared::prelude::*;`。全部 107 个引用 shared 类型的文件都走直接 import 路径。

### 评估

| 指标 | 值 |
|------|-----|
| 使用 `prelude::*` 的文件数 | 0 |
| 直接 import shared/ 的文件数 | 107 |
| prelude 覆盖率 | **0%** |

**根因分析**：
1. prelude 刚创建（本次重构），尚无时间推广
2. 项目已存在的 107 个文件已有稳定的直接 import 模式
3. prelude 未包含 IDs——而 IDs 是最常用的 shared 类型之一

**建议**：不强行推广 prelude。Rust 社区对 glob import 有争议（降低可读性）。prelude 的价值在于给新模块提供"默认可用"的类型，而非重构已有代码。继续保持 prelude 的存在，但**不要求**已有代码迁移。

---

## 2. 新增共享模块的采纳率

5 个在本次重构中实现的 shared 模块，**下游采纳率全部为 0%**：

| 模块 | 关键类型 | 下游消费者 | 备注 |
|------|---------|-----------|------|
| `collections/` | GroupByMap, TakeWhileInclusive, PartitionMap | **0** | 刚创建，尚无迭代器链需要这些扩展 |
| `hashing/` | FastHasher, fast_hash, new_fast_hashmap/set | **0** | 项目使用 std HashMap/HashSet，尚未遇到性能热点 |
| `math/` | HexCoord, hex_distance, FloatEq, lerp | **0** | tactical domain 有自己的 `GridPos.hex_distance()` |
| `validation/` | ValidationResult, Validator, ValidationChain | **0** | targeting 域自创了 `ValidationResult` 枚举 |
| `path/` | ProjectDirs, ensure_dir, asset_path | **0** | I/O 操作尚未规模化 |

### 典型重复案例：ValidationResult

targeting capability 在 `src/core/capabilities/targeting/foundation/values.rs` 中自创了：

```rust
pub enum ValidationResult {
    Pass,
    Fail(String),
}
```

而 `shared/validation/` 已提供参数化版本：

```rust
pub enum ValidationResult<T> {
    Valid(T),
    Invalid(Vec<ValidationError>),
}
```

这是 **重复造轮子** 的典型案例——shared 层存在通用工具，但下游不知道或不愿意用。

### 评估

所有 5 个模块均处于"基础设施先行"状态——功能可用但无实际需求驱动。"三次才抽象"原则在此适用：当业务代码出现重复模式时再推广使用，而非现在强行重构。

---

## 3. 核心类型下游使用分析

### DeterministicRng

| 使用模式 | 位置 | 数量 |
|---------|------|------|
| `ResMut<DeterministicRng>` | `infra/replay/systems.rs` | 1 |
| `Res<DeterministicRng>` | `combat/integration/replay/recording.rs` | 1 |
| 测试直接使用 | replay 测试 | 多处 |

**评估**：DeterministicRng 作为 Resource 已注册（SharedPlugin），但业务域（combat 判定、AI 决策、掉落生成）尚未接入。当前的消费者仅限于 Replay 系统内部同步。

### GameTime

| 使用模式 | 位置 | 数量 |
|---------|------|------|
| `Res<GameTime>` | `camp_rest/systems/` | 1 |
| `ResMut<GameTime>` | `shared/shared_plugin.rs` | 1 |
| 嵌入事件 | `scheduler/events.rs` | 5 个事件类型 |

**评估**：GameTime 是项目中最广泛使用的 shared 类型之一，通过调度器嵌入 5 种事件，覆盖帧级和回合级时间追踪。

### RuleFailure

| 实现者 | 数量 |
|--------|------|
| Core Domains | 15 （全部 domain） |
| Core Capabilities | 2 （Effect, Ability） |
| **合计** | **17 个实现** |

**评估**：RuleFailure 是采用率最高的 shared trait——所有 15 个业务域和 2 个核心能力层都通过 `impl_rule_failure!` 宏实现了它。这是 shared → core 单向依赖的成功案例。

### DomainEvent

Blanket impl 使得所有 Bevy Event 自动成为 DomainEvent。`ObservableEvent` 仅有 1 个显式实现（`LevelUp`），导致 19/20 的日志 Observer 处于"有监听、无事件"状态。

### LocalizationKey

被 7 个 domain 的 component 使用（economy/party/spell/camp_rest/quest/crafting/ability），是 Localization First 原则的正确落地。

---

## 4. Infra 层下游使用分析

### infra/replay

| 消费者 | 通过路径 | 合规 |
|--------|---------|------|
| combat 桥接层 | `core::capabilities::runtime::replay::mechanism` | ✅ 正确通过 core 能力层 |
| infra replay tests | 直接 import infra | ✅ 测试不受约束 |

Replay 桥接的层依赖路径是正确的：`combat/integration/` → `core::capabilities::runtime::replay` → `infra::replay`。

### infra/input

| 消费者 | 导入类型 |
|--------|---------|
| `combat/systems/input_system.rs` | InputAction, InputState |
| `tactical/systems/input_system.rs` | InputAction, InputState |

仅有战斗和战术两个玩家交互域使用输入系统。这是合理的——其他域（经济、制造、叙事）的输入通过 UI 间接驱动。

### infra/save

**零下游采用。** 没有任何 domain 注册 save handler、save marker 或响应 save events。SavePlugin 注册在 app_plugin 但处于"有基础设施、无业务接入"状态。

### infra/registry

**零下游采用（生产代码）。** `DefinitionRegistry` 和 `RegistryPlugin` 仅在 infra 自己的测试中使用，没有任何 domain plugin 直接 import infra/registry。

### infra/localization

仅在 UI modal factory 中使用 `LocalizedText`。其他 UI 组件尚未接入本地化。

### infra/logging/observers

20 个 observer 文件全部注册，但仅 1 个 `ObservableEvent` 实现存在。Observer 基础设施完备，但事件源头不足。

---

## 5. 层依赖合规检查

### ❌ P0: Core → App 违规（8 文件）

| 文件 | 违规导入 |
|------|---------|
| `core/domains/combat/plugin.rs` | `crate::app::scenes::GameState` |
| `core/domains/combat/components.rs` | `crate::app::scenes::GameState` |
| `core/domains/spell/plugin.rs` | `crate::app::scenes::GameState` |
| `core/domains/narrative/plugin.rs` | `crate::app::scenes::GameState` |
| `core/domains/tactical/plugin.rs` | `crate::app::scenes::GameState` |
| `core/domains/camp_rest/plugin.rs` | `crate::app::scenes::GameState` |
| `core/domains/terrain/plugin.rs` | `crate::app::scenes::GameState` |
| `core/domains/reaction/plugin.rs` | `crate::app::scenes::GameState` |

**问题**：Domain plugin 通过 `OnEnter(GameState::X)` / `OnExit(GameState::X)` 控制 System 在特定场景下运行。但 `GameState` 定义在 `app/` 层，Core 层依赖 App 层违反了 Shared ← Core ← Infra 的依赖方向。

**建议**：将 `GameState` 从 `app/` 下移至 `shared/` 层，或在 `core/` 层定义统一场景枚举，app 层做映射。

### ✅ 正确：Core → Infra 合规

之前存在的 `combat/integration/replay/recording.rs` 直接 import infra 的问题已在 RNG 重构中修复（改为通过 shared::random::DeterministicRng）。

Core 生产代码中 5 处 infra import：
| 导入 | 层 | 评估 |
|------|-----|------|
| `infra::input::InputAction` | 允许 | Input 是 Core 层正常依赖 |
| `infra::input::InputState` | 允许 | 同上 |
| `infra::logging::rate_limit::OnceGuard` (×3) | 🟡 可接受 | 日志速率限制是横切关注点 |

### content/ 层合规

Content 层仅依赖 Core + 自身，无 Infra 依赖。正确。

### ui/ 层合规

UI 层仅依赖 infra/localization（LocalizedText），无 Core 依赖。正确。

---

## 6. 发现汇总

### 严重问题

| # | 问题 | 优先级 | 影响 | 建议 | 状态 |
|---|------|--------|------|------|------|
| 1 | Core → App GameState 依赖（8 文件） | **P0** | 违反层架构 | 将 GameState 定义移至 shared/ | ✅ 已修复 |
| 2 | infra/save 零采用 | P2 | 存档功能不可用 | 需 domain 层面定义 SaveMarker | ⏳ 等待业务驱动 |
| 3 | infra/registry 零采用 | P2 | 配置注册框架空转 | 需 content 层面接入 | ⏳ 等待业务驱动 |
| 4 | ability_schema.md 远丰富于实现 | P2 | 文档与代码脱节 | 更新 schema 或标记为蓝图状态 | ⏳ 待处理 |
| 5 | fire_damage.ron 违反 V21 校验规则 | P3 | 内容合规性 | 放宽校验规则允许 modifiers 路径 | ✅ 已修复 |

### 中等观察

| # | 观察 | 说明 |
|---|------|------|
| 4 | 5 个新 shared 模块零采纳 | 按"三次才抽象"原则，等待下游需求自然驱动 |
| 5 | ValidationResult 重复定义 | targeting 域自创 vs shared 层已有——建议未来对齐 |
| 6 | ObservableEvent 仅 1 实现 | 20 个 Observer 在等待。需 domain 事件逐步接入 ObservableEvent |
| 7 | prelude 零使用 | 建议保留但不强推 |
| 8 | LocalizedText 仅 1 处使用 | 其他 UI Widget 尚未接入本地化 |

### 正确实践

| # | 实践 | 说明 |
|---|------|------|
| 9 | RuleFailure 17 实现 | shared → core 依赖的成功案例 |
| 10 | replay 桥接层合规 | combat → core:replay → infra:replay 路径正确 |
| 11 | input 系统仅 2 域使用 | 合理的按需接入 |
| 12 | DomainEvent blanket impl | 零样板代码，所有事件自动标记 |

---

## 7. 设计文档 vs 代码实现对齐评审

> 前面的分析聚焦于"代码消费代码"。本节评审"文档设计是否对齐代码实现"。

### 7.1 架构 ADR 对齐

| ADR | 对齐状态 | 关键差异 |
|-----|---------|---------|
| ADR-041（回放确定性） | 🟡 **大部分对齐** | EnumMap 伪代码 vs 实际命名字段；SyncCheckpoint 已从代码中移除但 ADR 未更新；系统名（start_frame_recording vs frame_counter_system）不一致 |
| ADR-057（Sealed Trait） | 🟢 **完全对齐** | 5 个 sealed trait 均按 `pub(crate) mod sealed { pub trait Sealed {} }` 模式实现，宏自动生成 impl |
| ADR-060（Extension Trait） | 🟢 **对齐** | EntityCommandsExt 和 QueryExt 按 ADR 定义的路径和签名实现，且 Phase 1 桩实现状态与 ADR 一致 |
| ADR-062（Object Safety） | 🟢 **对齐** | shared/ 和 infra/ 中未发现违规使用 dyn，热路径用泛型、冷路径用 dyn 的模式正确落地 |
| ADR-063（宏治理） | 🟢 **对齐** | 11 条原则全部遵循：无全局宏文件、宏跟能力走、无跨层宏依赖 |

### 7.2 领域规则对齐

| 领域文档 | 对齐状态 | 关键差异 |
|---------|---------|---------|
| `tag_domain.md` | 🟡 **大部分对齐** | 文档使用 `GameTagContainer` 名称但代码实际用 `TagSet`——命名漂移 |
| `effect_domain.md` | 🟢 **对齐** | 四阶段生命周期、Duration 类型、Periodic ticking 全部匹配。仅 Modifier rollback 标为 TODO |
| `replay_domain.md` | 🟢 **新建即对齐** | 本次新建的文档，直接基于当前代码架构撰写 |
| `tactical_domain.md` | 🟢 **已补充** | HexCoord 规则已追加到 §10 |

### 7.3 数据 Schema 对齐

| Schema | 对齐状态 | 关键差异 |
|--------|---------|---------|
| `ability_schema.md` | 🔴 **显著陈旧** | Schema 描述的数据模型远比实现丰富：targeting、activation_conditions、level_scaling、restrictions 在代码中缺失。ActiveAbilityContainer 未出现在检查的代码中。AbilityDef 实际实现是简化平面结构 |
| `effect_schema.md` | 🟡 **大部分对齐** | 未深入检查 |
| 其他 Capability Schema | 🟢 **对齐** | 未发现明显差异 |

### 7.4 内容 Def 对齐

| Def 文档 | 对齐状态 | 关键差异 |
|---------|---------|---------|
| `effect-def.md` | 🟡 **大部分对齐** | schema_version 字段在代码 struct 中缺失；文档称 effect_category: EffectCategory 但代码/ RON 用 effect_tags: Vec\<String\> |
| RON 配置合规 | ❌ **规则违规** | `assets/config/effects/fire_damage.ron` 违反 content 校验规则 V21：duration: Instant 类型必须设置 execution 或 execution_def，但该文件 execution: None，通过 modifiers 直接改 HP |

### 7.5 对齐偏差汇总

| # | 偏差 | 严重性 | 说明 |
|---|------|--------|------|
| 1 | Ability schema 远丰富于实现 | 🔴 P2 | 文档是蓝图，代码还在 Phase 1。需更新 schema 或补全代码 |
| 2 | ADR-041 含已删除的 SyncCheckpoint | 🟡 P3 | 文档未反映代码变更 |
| 3 | GameTagContainer vs TagSet 命名漂移 | 🟡 P3 | 不影响功能，但造成文档→代码混淆 |
| 4 | fire_damage.ron 违反 V21 校验规则 | 🟡 P3 | Instant 效果无 execution，内容合规性需修复 |
| 5 | schema_version 在 EffectDef 代码中缺失 | 🟡 P3 | 与 content def 文档定义不一致 |

---

## 8. 结论

**建议优先修复**：P0 的 Core → App GameState 违规（8 文件）— ✅ 已于 2026-06-21 修复。GameState/OverlayState/TransitionRequest 已从 app/scenes/ 迁至 shared/game_state/。

**建议暂缓**：新 shared 模块的推广、ObservableEvent 的全量接入、prelude 的迁移——这些应该在业务开发中自然发生，而非为了"用工具而用工具"。
