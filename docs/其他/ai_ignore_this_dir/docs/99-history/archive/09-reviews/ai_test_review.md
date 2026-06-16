# AI 模块测试评审报告

Version: 1.0
Date: 2026-06-11
Reviewer: Test Guardian
Scope: `src/ai/` 全部代码文件
Standard: `docs/test_spec.md` (Bevy SRPG Testing Constitution v3.1)
Domain Reference: `docs/domain/ai_rules_v2.md`

---

# 1. 评审范围

| 文件 | 行数 | 内联测试数 | 测试覆盖状态 |
|------|------|-----------|-------------|
| `mod.rs` | 15 | 0 | N/A（模块声明） |
| `behavior.rs` | 241 | 4 | 部分覆盖 |
| `decision.rs` | 234 | 0 | **未覆盖** |
| `movement.rs` | 89 | 5 | 良好 |
| `skill_select.rs` | 94 | 7 | 良好 |
| `strategy.rs` | 440 | 3 | 部分覆盖 |
| `targeting.rs` | 205 | 6 | 良好 |
| `plugin.rs` | 15 | 0 | N/A（插件注册） |

**总计：25 个内联测试，0 个集成测试，0 个场景测试**

外部测试目录（`tests/`）中无 AI 专属测试文件。

---

# 2. 评审标准

依据 `test_spec.md` 以下条款逐项评审：

| 条款 | 内容 | 评审重点 |
|------|------|----------|
| §3 Testing Philosophy | 测试验证 Behavior，不验证 Implementation | 断言是否关注 What 而非 How |
| §4 Test Pyramid | 70% Unit / 20% Integration / 8% Replay / 2% E2E | 各层级比例是否合理 |
| §5 Test Categories | Unit/Integration/Replay/Regression/E2E 定义 | 是否有缺失类别 |
| §6 Determinism Rules | 禁止随机、固定 Seed | 测试是否确定性 |
| §7 Test Case Schema | Test ID / Title / Given / When / Then / Assertions | 测试结构是否规范 |
| §7.1 Standard Test Data | Unit_001 / Unit_002 / Unit_003 | 是否使用标准数据 |
| §9 Coverage Strategy | 100% 核心领域规则覆盖 | 领域不变量是否全部测试 |
| §10 Error Testing | Invalid Input / Boundary Values | 边界和错误场景是否覆盖 |
| §13 AI Constraints | 禁止测试私有实现 | 是否越界测试内部细节 |
| §13.1 AI Self-Check | 6 项自检标注 | 是否有自检标注 |

---

# 3. 领域不变量覆盖评审

依据 `ai_rules_v2.md` 中定义的 7 个不变量：

| 不变量 | 描述 | 测试覆盖 | 评审结论 |
|--------|------|----------|----------|
| INV-AI-01 | AI 和玩家共用 Effect Pipeline | **无** | **严重缺失** — 核心宪法级不变量无任何测试保护 |
| INV-AI-02 | CombatIntent 是唯一攻击意图通道 | **无** | **严重缺失** — 无测试验证 AI 不绕过 CombatIntent |
| INV-AI-03 | 策略名称与配置对应 | 部分覆盖 | `策略注册表_按名称查找` 验证了查找，但未验证 RON 配置中策略名与注册表的一致性 |
| INV-AI-04 | 技能冷却检查 | 覆盖 | `技能策略_优先特殊_跳过冷却` 等测试验证 |
| INV-AI-05 | Rule / Content 分离 | 部分覆盖 | RON 反序列化测试验证了配置加载，但未验证"新增行为不改代码"这一规则 |
| INV-AI-06 | 策略 Trait 替代 enum+match | 覆盖 | 注册表分发测试验证 |
| INV-AI-07 | UnitSnapshot 避免借用冲突 | **无** | 架构级不变量，难以直接测试，但应在集成测试中隐式验证 |

**覆盖率：3/7 完全覆盖，2/7 部分覆盖，2/7 无覆盖**

---

# 4. 业务规则覆盖评审

| 规则 | 描述 | 测试覆盖 | 评审结论 |
|------|------|----------|----------|
| BR-AI-01 | AI 决策流程（快照→目标→移动→技能→意图） | **无** | `decision.rs` 的 `enemy_ai_system` 零测试 |
| BR-AI-02 | 行动结果路由（4 种情况） | **无** | 无测试验证攻击+移动/攻击/移动+待机/待机 4 种路由 |
| BR-AI-03 | CautiousMove 保持距离 | 覆盖 | `移动策略_谨慎_*` 系列测试验证 |

**覆盖率：1/3 完全覆盖，2/3 无覆盖**

---

# 5. 现有测试质量评审

## 5.1 逐文件评审

### behavior.rs（4 个测试）

| 测试 | 验证目标 | Behavior/Implementation | 合规性 |
|------|----------|------------------------|--------|
| `ron_反序列化_ai行为` | RON 格式正确解析 | 边界 — RON 格式是数据契约的一部分，可接受 | 缺少 Test ID、Given/When/Then 结构 |
| `ai_behavior_def_转换为_ai_behavior` | Def→Data 转换正确 | 边界 — 转换逻辑是公开契约，可接受 | 同上 |
| `ai_behavior_registry_默认行为` | 默认行为查找 | Behavior — 验证回退机制 | 同上 |
| `ron_反序列化_带技能优先级` | 技能优先级解析 | Behavior — 验证配置能力 | 同上 |

**问题：**
- 缺少 `default_behavior()` 在空注册表时的行为测试（应 panic 还是返回错误？）
- 缺少 `register_defaults()` 幂等性测试（重复调用是否安全？）

### targeting.rs（6 个测试）

| 测试 | 验证目标 | Behavior/Implementation | 合规性 |
|------|----------|------------------------|--------|
| `目标策略_最近` | Nearest 选择最近 | Behavior | 良好 |
| `目标策略_最弱` | Weakest 选择最低 HP | Behavior | 良好 |
| `目标策略_最危险` | MostDangerous 选择最高 ATK | Behavior | 良好 |
| `目标策略_最低血量百分比` | LowestHpPercent 选择最低 HP% | Behavior | 良好 |
| `目标策略_无玩家时返回自身位置` | 空候选回退 | Behavior — 边界测试 | 良好 |
| `目标策略_通过注册表分发` | 注册表查找正确 | Behavior | 良好 |

**问题：**
- 缺少平局测试：两个目标距离/HP 相同时的选择行为
- 缺少单目标测试：只有一个候选时的选择
- `Entity::PLACEHOLDER` 使用 — 不影响测试正确性，但不符合 Standard Test Data 规范

### movement.rs（5 个测试）

| 测试 | 验证目标 | Behavior/Implementation | 合规性 |
|------|----------|------------------------|--------|
| `移动策略_激进_贪心靠近目标` | Aggressive 贪心 | Behavior | 良好 |
| `移动策略_谨慎_保持攻击距离` | Cautious 保持距离 | Behavior | 良好 |
| `移动策略_谨慎_无范围内位置时靠近` | Cautious 回退 | Behavior — 边界测试 | 良好 |
| `移动策略_空可移动范围返回自身` | 空可达回退 | Behavior — 边界测试 | 良好 |
| `移动策略_通过注册表分发` | 注册表查找正确 | Behavior | 良好 |

**问题：**
- 缺少 SupportMove 独立行为测试（当前实现与 Aggressive 相同，但应测试其契约）
- 缺少单格可达测试
- 缺少目标在攻击范围内不需要移动的测试

### skill_select.rs（7 个测试）

| 测试 | 验证目标 | Behavior/Implementation | 合规性 |
|------|----------|------------------------|--------|
| `技能策略_优先特殊_跳过冷却` | PreferSpecial 跳过冷却 | Behavior | 良好 |
| `技能策略_优先特殊_全冷却回退基础攻击` | PreferSpecial 回退 | Behavior — 边界测试 | 良好 |
| `技能策略_优先基础` | PreferBasic 选择基础 | Behavior | 良好 |
| `技能策略_按优先级` | ByPriority 按序选择 | Behavior | 良好 |
| `技能策略_按优先级_首选冷却时选次选` | ByPriority 回退 | Behavior | 良好 |
| `技能策略_按优先级_空回退特殊` | ByPriority 空优先级回退 | Behavior — 边界测试 | 良好 |
| `技能策略_通过注册表分发` | 注册表查找正确 | Behavior | 良好 |

**问题：**
- 缺少 PreferBasicSkill 基础攻击在冷却时的回退测试
- 缺少空 skill_ids 列表测试
- 缺少优先级列表中技能不在 skill_ids 中的测试
- 缺少所有技能都在冷却时 ByPriority 的行为测试

### strategy.rs（3 个测试）

| 测试 | 验证目标 | Behavior/Implementation | 合规性 |
|------|----------|------------------------|--------|
| `策略注册表_默认注册所有策略` | 默认注册完整性 | **Implementation** — 检查 contains_key | **不合规** — 应验证行为而非注册表内容 |
| `策略注册表_按名称查找` | 查找返回正确策略 | Behavior | 良好 |
| `策略注册表_未知名称回退默认` | 回退机制 | Behavior | 良好 |

**问题：**
- `策略注册表_默认注册所有策略` 使用 `contains_key` 检查注册表内部结构，违反 §13"禁止测试内部数据结构"。应改为验证每个策略的行为（如查找后调用返回正确结果）。

### decision.rs（0 个测试）

**严重缺失** — 这是 AI 模块的核心系统文件，包含 `enemy_ai_system`，负责完整的 AI 决策流程。零测试意味着以下关键行为完全无保护：

- AI 仅在 SelectUnit 阶段执行
- AI 仅对敌方单位执行
- AI 计时器到期后才执行
- AI 设置 CombatIntent 而非直接执行效果（INV-AI-01/02）
- AI 行动结果正确路由（BR-AI-02）
- AI 快照正确收集所有单位数据

### plugin.rs（0 个测试）

可接受 — 插件注册通常由集成测试覆盖。

---

# 6. test_spec.md 合规性评审

## 6.1 §3 Testing Philosophy — 测试行为而非实现

| 评级 | 说明 |
|------|------|
| **基本合规** | 大部分测试验证行为（选择结果），但 `strategy.rs` 的 `contains_key` 检查违反此原则 |

具体违规：
- `策略注册表_默认注册所有策略`：断言 `registry.target_selectors.contains_key("Nearest")` 是检查内部 HashMap 结构，应改为通过 `target_selector("Nearest")` 查找并验证返回策略的 `strategy_name()`。

## 6.2 §4 Test Pyramid — 测试金字塔

| 层级 | 要求 | 实际 | 差距 |
|------|------|------|------|
| Unit Test | 70% | 25 个（100%） | 比例过高，缺少其他层级 |
| Integration Test | 20% | 0 个（0%） | **严重缺失** |
| Replay Test | 8% | 0 个（0%） | 缺失 |
| E2E Test | 2% | 0 个（0%） | 可接受（AI 不需要 E2E） |

**结论：测试金字塔严重失衡，缺少集成测试和 Replay 测试。**

## 6.3 §6 Determinism Rules — 确定性

| 评级 | 说明 |
|------|------|
| **合规** | 所有现有测试均为纯函数测试，无随机性。但 `decision.rs` 依赖 `Time` 和 `AiTimer`，未来集成测试需注意固定时间 |

## 6.4 §7 Test Case Schema — 测试用例结构

| 评级 | 说明 |
|------|------|
| **不合规** | 所有测试缺少 Test ID、Given/When/Then 结构化注释 |

现有测试使用中文函数名描述意图，但未按 §7 要求组织。示例对比：

当前：
```rust
#[test]
fn 目标策略_最近() { ... }
```

应改为：
```rust
/// - Test ID: AI-TGT-001
/// - Title: Nearest 策略选择最近敌人
/// - Given: 两个玩家单位，距离分别为 3 和 10
/// - When: 使用 Nearest 策略选择目标
/// - Then: 返回距离为 3 的目标坐标
/// - Assertions: assert_eq!(result, IVec2::new(3, 0))
#[test]
fn test_ai_tgt_001_nearest_selects_closest() { ... }
```

## 6.5 §7.1 Standard Test Data — 标准测试数据

| 评级 | 说明 |
|------|------|
| **不合规** | AI 测试使用自定义 `make_snapshot()` 辅助函数，未使用 Unit_001/002/003 标准数据 |

AI 模块的 `UnitSnapshot` 结构与标准测试数据的属性模型不同（Snapshot 使用 atk/hp/max_hp，标准数据使用 HP/ATK/DEF/SPD），需要建立映射关系。

## 6.6 §9 Coverage Strategy — 领域规则覆盖

| 评级 | 说明 |
|------|------|
| **不合规** | 7 个不变量中仅 3 个完全覆盖，3 个业务规则中仅 1 个覆盖。核心不变量 INV-AI-01/02 无测试保护 |

## 6.7 §10 Error Testing — 错误测试

| 评级 | 说明 |
|------|------|
| **不合规** | 缺少以下错误/边界场景测试 |

缺失的错误测试：
- 空 skill_ids 列表
- 所有技能在冷却中
- 优先级列表中技能不在 skill_ids 中
- 平局目标选择（距离/HP 相同）
- 空注册表查找
- AiBehaviorRegistry 空时调用 default_behavior()
- register_defaults() 重复调用（幂等性）

## 6.8 §13 AI Constraints — 禁止测试私有实现

| 评级 | 说明 |
|------|------|
| **轻微违规** | `策略注册表_默认注册所有策略` 测试了 `contains_key`（内部数据结构） |

## 6.9 §13.1 AI Self-Check — 自检标注

| 评级 | 说明 |
|------|------|
| **不合规** | 无任何测试文件包含 AI Self-Check 标注 |

---

# 7. 问题分类统计

## 7.1 按严重程度

| 严重程度 | 数量 | 描述 |
|----------|------|------|
| **P0 Critical** | 2 | 核心不变量无测试保护（INV-AI-01、INV-AI-02） |
| **P1 High** | 4 | decision.rs 零测试、BR-AI-01/02 无覆盖、测试金字塔失衡、领域规则覆盖不足 |
| **P2 Medium** | 5 | Test Case Schema 不合规、Standard Test Data 不合规、Error Testing 缺失、边界测试不足、AI Self-Check 缺失 |
| **P3 Low** | 3 | contains_key 实现细节测试、SupportMove 缺少独立测试、RON 反序列化测试边界 |

## 7.2 按问题类型

| 类型 | 数量 | 具体问题 |
|------|------|----------|
| 覆盖缺失 | 8 | decision.rs、INV-AI-01/02、BR-AI-01/02、集成测试、Replay 测试、错误测试、边界测试 |
| 规范不合规 | 4 | Test Case Schema、Standard Test Data、AI Self-Check、测试金字塔比例 |
| 测试质量 | 3 | contains_key 实现、SupportMove 缺失、幂等性未测 |
| 架构风险 | 2 | AI 独立伤害计算无测试保护、CombatIntent 绕过无测试保护 |

---

# 8. 优先级建议

## P0 — 必须立即补充（阻塞合并）

### 8.1 集成测试：AI 使用 Effect Pipeline（INV-AI-01）

- **文件**：`tests/feature/ai.rs`（新建）
- **目标**：验证 AI 设置 CombatIntent 后，伤害由 Effect Pipeline 处理，AI 不直接扣血
- **方法**：构建含 AI 单位的 App，推进到 AI 回合，验证伤害通过 DamageApplied Message 产生，而非直接 HP 修改

### 8.2 集成测试：CombatIntent 是唯一攻击通道（INV-AI-02）

- **文件**：同上
- **目标**：验证 AI 攻击后 CombatIntent 被正确设置（source_entity / target_coord / skill_id）
- **方法**：推进到 AI 回合，检查 CombatIntent 字段值

## P1 — 尽快补充（1 周内）

### 8.3 集成测试：AI 决策流程（BR-AI-01）

- **文件**：`tests/feature/ai.rs`
- **目标**：验证完整决策流程：快照收集 → 目标选择 → 移动选择 → 技能选择 → 意图设置
- **方法**：构建含敌方+玩家单位的 App，推进 AI 回合，验证最终 CombatIntent 和 MovingUnit 状态

### 8.4 集成测试：行动结果路由（BR-AI-02）

- **文件**：`tests/feature/ai.rs`
- **目标**：验证 4 种行动路由
  - 有攻击目标 + 需移动 → MovingUnit + ExecuteAction
  - 有攻击目标 + 不需移动 → ExecuteAction
  - 无攻击目标 + 需移动 → MovingUnit + WaitAction
  - 无攻击目标 + 不需移动 → WaitAction

### 8.5 单元测试：边界和错误场景

补充以下测试到各文件 `#[cfg(test)]` 模块：

**targeting.rs：**
- 平局目标选择（距离/HP 相同）
- 单候选目标
- max_hp = 0 时 LowestHpPercent 的行为

**skill_select.rs：**
- 空 skill_ids 列表
- 所有技能在冷却中
- 优先级列表中技能不在 skill_ids 中
- PreferBasicSkill 基础攻击在冷却时

**movement.rs：**
- SupportMove 独立行为测试
- 单格可达测试
- 目标在攻击范围内不需移动

**behavior.rs：**
- register_defaults() 幂等性
- default_behavior() 空注册表行为

## P2 — 计划补充（2 周内）

### 8.6 Test Case Schema 合规

为所有现有测试添加 Test ID 和 Given/When/Then 注释。Test ID 命名规则：

| 前缀 | 范围 |
|------|------|
| AI-BEH-xxx | behavior.rs 测试 |
| AI-TGT-xxx | targeting.rs 测试 |
| AI-MOV-xxx | movement.rs 测试 |
| AI-SKL-xxx | skill_select.rs 测试 |
| AI-STR-xxx | strategy.rs 测试 |
| AI-DEC-xxx | decision.rs 测试（新增） |

### 8.7 Standard Test Data 适配

创建 `UnitSnapshot` 从标准测试数据构建的辅助函数：

```rust
fn unit_001_snapshot(entity: Entity, coord: IVec2) -> UnitSnapshot {
    // HP=100, MaxHP=100, ATK=30, DEF=10, SPD=10
}
fn unit_002_snapshot(entity: Entity, coord: IVec2) -> UnitSnapshot {
    // HP=80, MaxHP=80, ATK=40, DEF=5, SPD=12
}
fn unit_003_snapshot(entity: Entity, coord: IVec2) -> UnitSnapshot {
    // HP=150, MaxHP=150, ATK=20, DEF=20, SPD=5
}
```

### 8.8 AI Self-Check 标注

为所有测试文件添加 §13.1 要求的自检标注。

## P3 — 低优先级

### 8.9 修复 contains_key 测试

将 `策略注册表_默认注册所有策略` 改为行为验证：

```rust
// Before (Implementation):
assert!(registry.target_selectors.contains_key("Nearest"));

// After (Behavior):
let selector = registry.target_selector("Nearest");
assert_eq!(selector.strategy_name(), "Nearest");
```

### 8.10 Replay 测试

为 AI 决策流程创建 Battle Replay 测试，验证完整战斗中 AI 行为的确定性。

---

# 9. 测试矩阵

## 9.1 领域不变量 × 测试覆盖

| 不变量 | Unit Test | Integration Test | Replay Test | 状态 |
|--------|:---------:|:----------------:|:-----------:|------|
| INV-AI-01 | — | **需补充** | **需补充** | 缺失 |
| INV-AI-02 | — | **需补充** | — | 缺失 |
| INV-AI-03 | 部分覆盖 | **需补充** | — | 不足 |
| INV-AI-04 | 覆盖 | — | — | 达标 |
| INV-AI-05 | 部分覆盖 | — | — | 不足 |
| INV-AI-06 | 覆盖 | — | — | 达标 |
| INV-AI-07 | — | **需补充** | — | 缺失 |

## 9.2 业务规则 × 测试覆盖

| 规则 | Unit Test | Integration Test | Scenario Test | 状态 |
|------|:---------:|:----------------:|:-------------:|------|
| BR-AI-01 | — | **需补充** | **需补充** | 缺失 |
| BR-AI-02 | — | **需补充** | — | 缺失 |
| BR-AI-03 | 覆盖 | — | — | 达标 |

## 9.3 策略实现 × 测试覆盖

| 策略 | 正常场景 | 边界场景 | 错误场景 | 注册表分发 | 状态 |
|------|:--------:|:--------:|:--------:|:----------:|------|
| NearestTarget | 覆盖 | 缺失 | 覆盖 | 覆盖 | 不足 |
| WeakestTarget | 覆盖 | 缺失 | — | — | 不足 |
| MostDangerousTarget | 覆盖 | 缺失 | — | — | 不足 |
| LowestHpPercentTarget | 覆盖 | 缺失 | — | — | 不足 |
| AggressiveMove | 覆盖 | 覆盖 | 覆盖 | 覆盖 | 达标 |
| CautiousMove | 覆盖 | 覆盖 | — | — | 良好 |
| SupportMove | **缺失** | — | — | — | 缺失 |
| PreferSpecialSkill | 覆盖 | 覆盖 | — | 覆盖 | 良好 |
| PreferBasicSkill | 覆盖 | 缺失 | — | — | 不足 |
| ByPrioritySkill | 覆盖 | 覆盖 | 缺失 | 覆盖 | 良好 |

---

# 10. 总体评估

| 维度 | 评级 | 说明 |
|------|------|------|
| 领域规则覆盖 | **D** | 7 个不变量仅 3 个完全覆盖，3 个业务规则仅 1 个覆盖 |
| 测试金字塔 | **D** | 100% Unit / 0% Integration / 0% Replay，严重失衡 |
| 测试质量 | **B** | 大部分测试验证行为而非实现，仅 1 处轻微违规 |
| 规范合规 | **D** | Test Case Schema、Standard Test Data、AI Self-Check 均不合规 |
| 错误/边界覆盖 | **C** | 有部分边界测试，但缺少错误场景和平局场景 |
| 确定性 | **A** | 所有测试均为纯函数，无随机性 |

**综合评级：D+**

核心问题：AI 模块最关键的系统（`enemy_ai_system`）和最关键的不变量（INV-AI-01/02）完全没有测试保护。这意味着 AI 可能绕过 Effect Pipeline 直接执行效果，或绕过 CombatIntent 发起攻击，而测试无法发现。

---

# 11. 行动计划摘要

| 优先级 | 行动项 | 预计新增测试数 |
|--------|--------|---------------|
| P0 | 集成测试：INV-AI-01 + INV-AI-02 | 2 |
| P1 | 集成测试：BR-AI-01 + BR-AI-02 | 4 |
| P1 | 边界/错误单元测试补充 | 10 |
| P2 | Test Case Schema 合规 | 0（重构现有） |
| P2 | Standard Test Data 适配 | 0（重构现有） |
| P2 | AI Self-Check 标注 | 0（添加注释） |
| P3 | contains_key 修复 | 0（修改 1 个测试） |
| P3 | Replay 测试 | 1 |

**预计新增测试：17 个，重构现有测试：25 个**
