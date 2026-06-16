---
id: 05-testing.test-spec
title: Test Spec
status: draft
owner: test-guardian
created: 2026-06-14
updated: 2026-06-14
tags:
  - testing
---

# Bevy SRPG Testing Constitution v4.0
**与《SRPG 项目总宪法 v5.0》第十二编配套执行**

Version: 4.0
Status: Active
Applies To:
* All Rust code
* All Bevy plugins
* All ECS systems
* All gameplay features
* All AI-generated code

---

# 1. Purpose
本文件定义项目测试规则。
目标：
1. 验证业务规则正确性
2. 防止历史Bug回归
3. 保证重构安全
4. 保证AI修改代码时不会破坏既有行为
5. 提供唯一测试标准

## 1.0 核心原则（宪法 12.2 🟥）

> **测试跟领域走（Feature First），但不写在源码文件内部。**

- 🟥 禁止 `#[cfg(test)] mod tests` 内联测试（对 AI 上下文污染严重）
- 🟥 禁止将所有测试平铺到根 `tests/unit/`（后期变成大杂烩）
- 🟩 测试与被测领域同目录放置，形成 Feature Folder 结构
- 🟩 根 `tests/` 仅保留跨领域测试（战斗流程、存档、回归、E2E）

## 1.1 Non-goals（绝对不测）
本规范不包含以下测试，AI绝对禁止生成：
- UI视觉与动画测试
- 性能压测与基准测试
- Bevy引擎本身功能验证
- 第三方库功能验证
- 美术资源与音效测试
- 网络同步测试

本文件不定义业务规则。业务规则定义于：
* architecture.md
* domain_rules.md

---

# 2. Source Of Truth
规则优先级（绝对不可违反）：
**Level 1：domain_rules.md**
定义：
* 战斗规则
* 属性规则
* Buff规则
* 地图规则
* 回合规则

**Level 2：architecture.md**
定义：
* 模块边界
* ECS规则
* 数据流规则

**Level 3：test_spec.md**
定义：
* 测试规范
* 测试格式
* 测试执行要求

当发生冲突时：
`domain_rules.md > architecture.md > test_spec.md`

---

# 2.5 测试命名规范

- 测试函数名用**中文**描述预期行为，技术术语如 UI/AI/HP/MP/Buff 等保留英文
- 文件名保持英文 snake_case

合法示例：
```rust
#[test]
fn 物理伤害正确应用护甲减免() { ... }

#[test]
fn Buff到期自动移除() { ... }

#[test]
fn HP不会低于零() { ... }

#[test]
fn 技能冷却期间无法重复释放() { ... }
```

非法示例：
```rust
#[test]
fn test_damage() { ... }              // 无业务语义

#[test]
fn physical_damage_respects_armor() { ... }  // 应使用中文

#[test]
fn a() { ... }                        // 无意义命名
```

---

# 3. Testing Philosophy
测试验证：**Behavior**。而不是：**Implementation**
正确：`assert_eq!(target.hp, 75)`
错误：`assert_eq!(effect_queue.len(), 1)`

正确：`assert!(unit.has::<Dead>())`
错误：`assert!(death_system_called)`

测试关注：**What**
不关注：**How**

---

# 4. Test Pyramid
推荐比例（领域内聚四层 + 跨域测试）：
- **unit** (单元测试) — 验证单个函数/纯规则的正确性
- **integration** (集成测试) — 验证领域内多组件协作
- **invariant** (不变量测试) — 验证领域不变量（**最高价值**）
- **fixtures** (测试数据) — Builder 模式构造的测试数据

跨域测试（根 tests/）：
- battle_flow / save_load / regression / replay / golden / simulation / performance / e2e

禁止：
为了测试而启动完整游戏。
优先测试：
- 纯规则
- 纯逻辑
- 纯函数

---

# 5. Test Categories
## Unit Test（单元测试）
目标：验证单一规则。
特点：
* 不启动App
* 不加载Plugin
* 不依赖资源
* 位于 `<domain>/tests/unit/`

示例：
* 伤害公式
* 属性计算
* Buff持续时间

---

## Integration Test（集成测试）
目标：验证多个模块协作。
特点：
* 最小Bevy App
* 最少Plugin
* 位于 `<domain>/tests/integration/`

示例：
* 角色穿戴装备→Modifier→Attribute 联动
* 技能释放→Effect→Buff 联动

---

## Invariant Test（不变量测试）— 最高价值

验证领域不变量，是架构稳定性的最后防线。

位于 `<domain>/tests/invariant/`。

| 不变量 | 说明 | 测试文件 |
|--------|------|----------|
| Tag bit 唯一 | 同一 Tag 不能在位掩码中重复设置 | `tag_invariant_spec.rs` |
| Buff 不重复叠加 | 同源同类型 Buff 不会无限堆叠 | `buff_invariant_spec.rs` |
| Effect 不修改不存在属性 | Effect 引用的 AttributeId 必须已注册 | `effect_invariant_spec.rs` |
| HP 永远 >= 0 | HP 计算结果不能为负 | `hp_invariant_spec.rs` |
| Modifier 不改变基础值 | Modifier 只影响聚合后的当前值 | `modifier_invariant_spec.rs` |
| 回合先攻排序稳定 | 同先攻值的单位顺序确定 | `turn_invariant_spec.rs` |
| 技能消耗原子性 | 消耗失败时不产生部分效果 | `ability_invariant_spec.rs` |

> 不变量测试的价值远大于普通单元测试。

---

## Replay Test（回放测试）
**项目最高优先级测试**。
目标：验证完整战斗过程。
特点：
* 输入固定
* 输出固定
* 可重复执行

所有战斗Bug：**必须**转化为Replay Test。

---

## Regression Test（回归测试）
所有已修复Bug：**必须**对应回归测试。
永久保留。
禁止删除。

---

## End To End Test（端到端测试）
仅验证核心主流程。
数量保持最少。
位于根 `tests/e2e/`。

---

# 6. Determinism Rules
所有测试必须确定性。
禁止：
- ThreadRng
- 随机时间
- 随机资源
- 网络依赖
- 当前系统时间

必须：
- 固定Seed
- Seed = 42

相同输入：**必须**产生相同结果。

---

# 7. Test Case Schema
所有测试必须严格遵循以下结构，不得增减字段：
- Test ID
- Title
- Given
- When
- Then
- Assertions

Example：
- Test ID: DMG-001
- Title:基础伤害计算
- Given:
  - ATK = 30
  - DEF = 10
  - Multiplier = 1.0
- When:执行伤害计算
- Then:Damage = 20
- Assertions:assert_eq!(damage, 20)

## 7.1 Standard Test Data
所有测试**必须**使用以下标准测试单位，不得自定义：
- Unit_001 (基础战士): HP=100, MaxHP=100, ATK=30, DEF=10, SPD=10, Level=1
- Unit_002 (基础法师): HP=80, MaxHP=80, ATK=40, DEF=5, SPD=12, Level=1
- Unit_003 (基础坦克): HP=150, MaxHP=150, ATK=20, DEF=20, SPD=5, Level=1

---

# 8. Replay Test Schema
Replay文件格式：`battle_replays/*.yaml`
结构：
- Scenario
- Initial State
- Actions
- Expected State
- Expected Messages
- Expected Winner

示例：
- Scenario: basic_attack

- Initial State:
  - Knight HP 100
  - Mage HP 80

- Actions:
  - Turn1 Knight Attack Mage

- Expected State:
  - Mage HP 55

- Expected Winner:
  - None

---

# 9. Coverage Strategy
必须覆盖：**100%核心领域规则**
包括：
* Damage
* Heal
* Death
* Buff
* Turn
* Equipment
* Modifier

覆盖目标：**Rule Coverage**
而不是：**Code Coverage**
禁止：为了提高覆盖率编写无价值测试。

---

# 10. Error Testing
必须验证：
- Invalid Input
- Invalid State
- Missing Data
- Boundary Values

验证内容：
- 错误码
- 状态不变
- 日志记录
- 不会崩溃

---

# 11. Regression Rules
发现Bug：**必须先写测试**。

流程：
- Step 1：创建失败测试
- Step 2：确认失败
- Step 3：修复代码
- Step 4：确认通过
- Step 5：加入回归测试集
禁止：
- 先修复后补测试。

---

# 12. AI Decision Rules
当测试失败时：**严格执行以下流程**。
- Step 1：检查 domain_rules.md
- Step 2：检查 architecture.md
- Step 3：检查测试是否符合本规范
  - 判断：
    - 测试违反领域规则 → 修改测试
    - 代码违反领域规则 → 修改代码
    - 双方都符合 → 更新领域规则
- 禁止：为了让测试通过而修改业务逻辑。
- 禁止：为了让代码通过而修改测试。

---

# 13. AI Constraints
禁止：
- 测试私有实现
- 测试内部缓存
- 测试Query数量
- 测试System顺序
- 测试组件布局
- 测试内部数据结构

允许：
- 测试业务行为
- 测试状态变化
- 测试消息产生
- 测试最终结果

## 13.1 AI Self-Check（文档参考，不输出到代码）

> **说明**：此清单仅作为 AI 生成测试时的内部参考，不要求在生成的测试文件中输出自检结果。
> 真正有效的合规检查依赖 CI 门禁（cargo test / clippy），而非 AI 自检。

AI 生成测试前应内部对照：

| 检查项 | 说明 |
|--------|------|
| 测试行为，不是实现 | 验证 What，不验证 How |
| 符合领域规则 | 对照 domain_rules.md |
| 测试是确定性的 | 固定 Seed，无系统时间依赖 |
| 使用 Builder 模式构造数据 | 不硬编码业务数值 |
| 没有测试私有实现 | 只测公共行为和状态变化 |
| 测试跟领域走 | 放在 `<domain>/tests/` 而非根 `tests/unit/` |
| 含 invariant 层 | 关键领域必须有不变量测试 |
| 一个测试只验证一种行为 | 失败时可快速定位 |

---

# 14. CI Requirements
每次提交必须执行：
- Unit Test
- Integration Test
- Replay Test
- Regression Test

任何失败：**禁止合并**。

---

# 15. Exemptions
允许豁免：
- 实验功能
- 临时代码
- 原型验证

必须记录：
- Reason
- Owner
- Created Date
- Expire Date

禁止永久豁免。

---

# 16. Definition Of Done
一个功能完成必须满足：
- 功能实现完成
- 领域规则更新
- 测试通过
- 回归测试补齐
- Replay测试补齐
- 代码评审通过

否则：视为未完成。

---

# 17. Final Principle
测试的目标不是证明代码正确。
测试的目标是证明：
- 代码仍然符合领域规则。
- Domain Rules First.
- Tests Second.
- Implementation Third.

---

