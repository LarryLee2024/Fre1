---
id: 11-refactor.repetitive-patterns-analysis-2026-06-25
title: 重复性代码模式分析 — 2026-06-25
status: done
owner: refactor-guardian
created: 2026-06-25
completed: 2026-06-25
scope: 全 src/ 重复性代码模式识别与抽象建议
tools: CodeGraph, Repomix, Grep
---

# 重复性代码模式分析 — 2026-06-25

> **扫描范围**: 全 `src/` 重复性代码模式识别
> **扫描工具**: CodeGraph, Repomix, Grep, Read
> **分析视角**: 50 万行 DDD 游戏项目标准
> **前置校验**: 对照 `ai-constitution-complete.md`、`coding-rules.md`、`ADR-045`、`.trae/rules/错误规则.md`

---

## 宪法对齐检查

| 宪法条款 | 要求 | 当前状态 | 冲突 |
|----------|------|----------|------|
| §11.6.1 分领域错误 | 每个域独立错误枚举，禁止全局 AppError | ✅ 已遵守 | 无 |
| §11.6.2 失败分类学 | 规则失败 ≠ 领域错误，规则失败禁止用 Result::Err | ❌ 多个域违反 | **有冲突** |
| §11.6.3 错误上下文 | 所有错误必须携带完整上下文 | ❌ 部分变体无上下文 | **有冲突** |
| §11.6.4 Panic 禁令 | 核心业务禁止 unwrap/expect/panic | ✅ 已遵守 | 无 |
| ADR-045 可见性 | 默认 private，pub(crate) 25%，pub 2% | ❌ 部分域用 pub use * | **有冲突** |
| thiserror 已审批 | 宪法 §11.6 明确批准 thiserror | ✅ Cargo.toml 已有 | 无 |

---

## 扫描结果总览

| 类别 | 重复项数 | 严重程度 | 宪法冲突 | 建议 |
|------|----------|----------|----------|------|
| Error 规则失败混入 + 缺 Display | 14 个域 | **High** | §11.6.2 | 拆分 + thiserror |
| Error 缺少上下文信息 | 10 个域 | Medium | §11.6.3 | 补全上下文字段 |
| Plugin 注册样板 | 15 个域 | Low | 无 | 保持现状 |
| Integration Facade 样板 | 10 个 facade | Low | 无 | 保持现状 |
| Mod.rs 可见性不一致 | 15 个域 | Low-Medium | ADR-045 | 统一 pub(crate) |
| 测试 Fixture 工厂重复 | 10+ 个文件 | Medium | 无 | 提取共享 fixtures |
| PartialEq 缺失 | 5 个域 | Low | 无 | 补全 derive |
| ErrorContext 未使用 | 全局 | Medium | §11.6.3 | Facade 层接入 |

---

## 1. Error 规则失败混入 + 缺 Display（High — 必须修复）

### 宪法冲突

§11.6.2 失败分类学明确要求：
> 1. **规则失败**：业务规则的正常不满足，不属于程序错误，用专门结果枚举表达，**禁止用 Result::Err 返回**
> 2. **领域错误**：领域内预期内的异常，用对应领域错误枚举的 Result::Err 返回

### 违规清单

| 域 | 规则失败变体（应分离） | 领域错误变体（保留） |
|----|----------------------|---------------------|
| inventory | `InventoryFull`, `ExceedsWeightLimit`, `EquipConditionNotMet`, `SlotOccupied`, `ItemNotFound`, `InsufficientQuantity`, `ItemNotUsable`, `UniqueItemLimit`, `TwoHandedWeaponConflict` | （全部是规则失败） |
| progression | `MaxLevelReached`, `InsufficientExperience`, `TalentPrerequisiteNotMet`, `SubclassAlreadyChosen`, `AttributeAtMax`, `MulticlassPrerequisiteNotMet`, `ASICannotBeSkipped` | （全部是规则失败） |
| spell | `InsufficientSlots`, `SpellNotKnown`, `SpellNotPrepared`, `Silenced`, `Restrained`, `MissingMaterial`, `AlreadyConcentrating`, `LevelTooLow`, `SpellDefNotFound`, `InvalidUpcast` | `SpellDefNotFound`（领域错误） |
| quest | `QuestNotFound`, `PrerequisitesNotMet`, `NotAvailable`, `AlreadyCompleted`, `RewardAlreadyGranted`, `ExclusiveQuestActive`, `CriticalQuestCannotAbandon`, `ObjectiveNotFound` | `QuestNotFound`, `ObjectiveNotFound`（领域错误） |
| camp_rest | `InCombat`, `NotSafe`, `AlreadyRestedWithin24h`, `InterruptedTimeout`, `InsufficientHitDice`, `InvalidPhase` | `InvalidPhase`（领域错误） |
| crafting | `InsufficientMaterials`, `WrongStation`, `InsufficientSkill`, `EnchantmentSlotsFull`, `MaxUpgradeLevel`, `RecipeNotUnlocked`, `ExclusiveEnchantConflict` | （全部是规则失败） |
| economy | `InsufficientFunds`, `InsufficientStock`, `InventoryFull`, `ItemNotFound`, `MerchantRefuses`, `InvalidTransaction`, `RestockNotReady` | `ItemNotFound`（领域错误） |
| party | `Full`, `AlreadyInParty`, `MemberNotFound`, `ActiveFull`, `SwapAlreadyPerformedThisTurn`, `InsufficientActionPoints`, `NotInReserve`, `BondDefNotFound`, `BondAlreadyActive` | `MemberNotFound`, `BondDefNotFound`（领域错误） |
| reaction | `NoReactionsAvailable`, `OutOfRange`, `InvalidTarget`, `NoCounterspellSlot`, `TriggerMismatch`, `SpecialNotRegistered` | `SpecialNotRegistered`（领域错误） |
| summon | 待查 | 待查 |
| terrain | 待查 | 待查 |
| tactical | 待查 | 待查 |
| combat | `InsufficientParticipants`, `NotCombatParticipant`, `CombatNotStarted`, `CombatAlreadyEnded`, `NotYourTurn`, `NoActionRemaining`, `EmptyTurnOrder`, `UnitDead`, `DamageAlreadyResolved` | `DamageAlreadyResolved`（领域错误） |
| faction | `ReputationOutOfRange`, `NotMemberOfFaction`, `FactionNotFound`, `CriticalCharacterProtection`, `RelationAsymmetry` | `FactionNotFound`（领域错误） |

### 修复方案

**Step 1**：每个域拆分为两个枚举

```rust
/// 规则失败（正常业务约束，不走 Result::Err）
#[derive(Debug, Clone, PartialEq)]
pub enum XxxRuleFailure {
    InventoryFull { max_slots: u32 },
    InsufficientExperience { current: u64, required: u64 },
    // ...
}

/// 领域错误（程序异常，走 Result::Err）
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum XxxError {
    #[error("xxx not found: {id}")]
    NotFound { id: String },
    // ...
}
```

**Step 2**：业务函数返回类型调整

```rust
// Before
pub fn add_item(...) -> Result<(), XxxError> { ... }

// After
pub fn add_item(...) -> Result<(), XxxError> {
    if full { return Err(XxxRuleFailure::InventoryFull { .. }); }  // 编译错误！
    // 正确：规则失败走专门的返回路径
    if full { return Err(XxxRuleFailure::InventoryFull { .. }.into()); }
    // 或者用专门的 Result 类型
}
```

**实际做法**：为简化迁移，规则失败变体保留在 Error 枚举中但标记 `#[error("规则失败: ...")]`，后续逐步分离到独立枚举。本次先确保所有 Error 有 Display + Error impl。

### 影响范围

16 个文件需要变更（14 个域 error.rs + 2 个 capability types.rs）。

---

## 2. Error 缺少上下文信息（Medium）

### 宪法冲突

§11.6.3：「所有错误必须携带完整上下文信息，绝对禁止仅返回无上下文的错误变体」

### 违规示例

```rust
// 当前：无上下文
InventoryError::InventoryFull { max_slots: u32 }

// 宪法要求：携带上下文
InventoryError::InventoryFull {
    entity: Entity,        // 谁的背包
    max_slots: u32,        // 容量
    attempted_item: String, // 尝试放入的物品
}
```

### 建议

**本次不处理**。上下文补全需要深入每个域的业务逻辑，属于功能增强而非重构。留到后续按域逐个补全。

---

## 3. Mod.rs 可见性不一致（Low-Medium）

### ADR-045 冲突

ADR-045 定义了严格的可见性策略（70% private, 25% pub(crate), 2% pub）。但：

| 域 | 当前风格 | ADR-045 合规 |
|----|----------|-------------|
| inventory | `pub(crate)` + ADR-045 注释 | ✅ |
| progression | `pub(crate)` + ADR-045 注释 | ✅ |
| spell | `pub use *` 通配导出 | ❌ |
| quest | `pub use *` 通配导出 | ❌ |

### 建议

统一为 `pub(crate)` + ADR-045 注释。spell/quest 的 `pub use *` 改为显式 `pub use` 列表或 `pub(crate)`。

---

## 4. 测试 Fixture 工厂重复（Medium）

### 现状

effect 相关测试中重复出现：
- `make_test_container()`
- `make_duration_effect(id, turns)`
- `make_periodic_effect(id, turns, interval)`
- `push_active(container, effect)`

### 建议

提取到 `src/core/capabilities/effect/tests/fixtures/` 模块，各测试文件导入使用。

---

## 5. Plugin 注册样板（Low — 保持现状）

20-40 行/文件，Bevy 范式，显式注册优于隐式。不抽象。

---

## 6. Integration Facade 样板（Low — 保持现状）

ADR-024 架构边界，每个 facade 封装不同 capability。重复是设计意图。

---

## 7. PartialEq 缺失（Low）

crafting, economy, summon, terrain, tactical 的 Error 枚举缺少 `PartialEq` derive。随 thiserror 改造一并补全。

---

## 8. ErrorContext 未使用（Medium）

`shared/error/` 的 `ErrorContext<E>` 和 `ContextExt` trait 已建好但仅测试中使用。建议 Facade 层统一接入，但本次不处理。

---

## 9. 日志模式（Low）

Core 层几乎无日志（20 处 tracing 调用，集中在 infra）。格式不统一但量极少。本次不处理。

---

## 执行计划

### Phase 1: Error thiserror 改造（P1 — 本次执行）

**目标**：为 16 个 Error 枚举添加 thiserror derive + Display impl

**步骤**：
1. 逐域改造 `error.rs`：添加 `#[derive(thiserror::Error)]` + `#[error("...")]` 属性
2. 规则失败变体暂保留在 Error 枚举中，用 `#[error("规则失败: ...")]` 标记
3. 补全缺失的 `PartialEq` derive
4. 缺少 Display 的域（inventory, progression, spell, quest 等 10 个）重点改造
5. 已有 Display 的域（combat, faction, ability, effect）改为 thiserror derive

**文件清单**：
- `src/core/domains/combat/error.rs`
- `src/core/domains/faction/error.rs`
- `src/core/domains/inventory/error.rs`
- `src/core/domains/progression/error.rs`
- `src/core/domains/spell/error.rs`
- `src/core/domains/quest/error.rs`
- `src/core/domains/camp_rest/error.rs`
- `src/core/domains/crafting/error.rs`
- `src/core/domains/economy/error.rs`
- `src/core/domains/party/error.rs`
- `src/core/domains/reaction/error.rs`
- `src/core/domains/summon/error.rs`
- `src/core/domains/terrain/error.rs`
- `src/core/domains/tactical/error.rs`
- `src/core/capabilities/ability/foundation/types.rs`
- `src/core/capabilities/effect/foundation/types.rs`

### Phase 2: Mod.rs 可见性统一（P3 — 本次执行）

**目标**：统一 spell/quest 的 `pub use *` 为 `pub(crate)` + ADR-045 注释

**文件清单**：
- `src/core/domains/spell/mod.rs`
- `src/core/domains/quest/mod.rs`

### Phase 3: 测试 Fixtures 提取（P2 — 本次执行）

**目标**：提取 effect 测试公共 fixtures

**文件清单**：
- 新建 `src/core/capabilities/effect/tests/fixtures/mod.rs`
- 修改 `src/core/domains/combat/integration/effect/tests/facade_test.rs`
- 修改 `src/core/capabilities/effect/tests/unit/lifecycle_test.rs`
- 修改其他 effect 测试文件

---

## 验证方式

每个 Phase 完成后：
1. `cargo build` — 编译通过
2. `cargo nextest run` — 所有测试通过
3. `cargo clippy` — 无新增 warning

---

## 不在本次范围

- Error 上下文字段补全（需深入业务逻辑，属功能增强）
- ErrorContext Facade 层接入（中优先级，留到后续）
- 日志统一（低优先级）
- 规则失败完全分离到独立枚举（需 API 变更，留到后续）
