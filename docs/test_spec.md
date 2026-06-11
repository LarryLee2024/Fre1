# Bevy 0.18+ SRPG AI测试规范 v2.0（95+工业级）
> 基于你提供的工业级框架重构，100%符合"结构稳定+可执行+可验证+无歧义+机器可解析"标准
> 与《Bevy 0.18+ SRPG AI开发宪法v1.1》配套执行

---

## 0. 文档元信息（必须）
```md
# Test Specification

## Version
v2.0.0

## Owner
Bevy SRPG AI开发组

## Last Updated
2026-06-11

## Scope
本测试规范覆盖Bevy SRPG项目所有核心业务逻辑：
- 战斗系统
- 角色系统
- 属性系统
- 技能系统
- Buff/Debuff系统
- 装备系统
- 地图与寻路系统
- 回合管理系统

## Non-goals（绝对不测）
- UI视觉与动画测试
- 性能压测与基准测试
- Bevy引擎本身功能验证
- 第三方库功能验证
- 美术资源与音效测试
- 网络同步测试（本项目为单机）
```

---

## 1. 测试目标（Test Objectives）
```md
## Objectives

本测试的目标是验证：
1. 所有核心游戏规则的功能正确性
2. 所有状态转换的一致性和确定性
3. 所有边界输入和异常状态的稳定性
4. 所有错误输入的优雅处理能力
5. 所有历史Bug的永久防护
```

---

## 2. 系统范围拆解（System Under Test）
```md
## System Components

### Core Modules
- DamageResolver
- TurnManager
- CombatSystem
- SkillExecutor
- BuffSystem
- AttributeSystem
- EquipmentSystem
- PathfindingSystem
- MapSystem

### Data Structures
- Unit
- Skill
- Buff
- Modifier
- Equipment
- MapTile
- CombatState
```

---

## 3. 测试分类体系（Test Taxonomy ⭐核心）
```md
## Test Categories

### 1. Unit Test（单元测试）
- 验证单一纯函数的行为
- 不启动Bevy App
- 不依赖任何外部资源

### 2. Integration Test（集成测试）
- 验证两个及以上模块间的交互
- 使用最小化Bevy App
- 只加载必要的Plugin

### 3. System Test（系统测试）
- 验证完整的业务流程
- 使用接近生产环境的App配置

### 4. Edge Case Test（边界测试）
- 验证极端输入和极端状态
- 包括数值边界、状态边界和逻辑边界

### 5. Regression Test（回归测试）
- 所有历史修复的Bug对应的测试用例
- 永久保留，每次构建必须执行

### 6. Battle Replay Test（SRPG专属最高优先级）
- 通过录制/回放验证复杂战斗逻辑
- 所有战斗相关Bug必须转化为此类测试
- 优先级高于所有其他测试类型
```

---

## 4. 状态模型（State Model ⭐非常重要）
```md
## State Model

### Combat State Machine

States（枚举值）:
- Idle
- SelectingUnit
- SelectingAction
- SelectingTarget
- ExecutingAction
- ResolvingEffects
- TurnEnd
- CombatVictory
- CombatDefeat

Transitions（方向固定）:
Idle → SelectingUnit
SelectingUnit → SelectingAction
SelectingAction → SelectingTarget
SelectingTarget → ExecutingAction
ExecutingAction → ResolvingEffects
ResolvingEffects → TurnEnd
TurnEnd → Idle
ResolvingEffects → CombatVictory
ResolvingEffects → CombatDefeat

### Unit State Machine

States（枚举值）:
- Alive
- Dead
- Stunned
- Frozen
- Poisoned
- Silenced
```

---

## 5. 测试用例标准格式（Test Case Schema ⭐核心）
```md
## Test Case Format

所有测试用例必须严格遵循以下格式，不得增减字段

### TestCase ID: TC-[模块缩写]-[三位数字]
例如：TC-DMG-001, TC-COMB-012

#### Title
一句话描述测试场景和预期结果

#### Preconditions
- 条件1：具体数值
- 条件2：具体状态
- 条件3：具体配置

#### Input
- 输入1：具体数值
- 输入2：具体操作
- 输入3：具体事件

#### Steps
1. 执行操作1
2. 执行操作2
3. 执行操作3

#### Expected Output
- 状态变化：[旧状态] → [新状态]
- 数值变化：[旧值] → [新值]
- 事件触发：事件名称(参数)

#### Assertions
- assert!(condition1, "错误信息")
- assert_eq!(actual, expected, "错误信息")
- assert_relative_eq!(actual, expected, epsilon=0.0001, "错误信息")

#### Postconditions
- 最终状态：具体状态
- 最终数值：具体数值
- 清理操作：需要清理的资源
```

---

## 6. 测试数据规范（Test Data Spec）
```md
## Test Data

### Standard Test Units
所有测试必须使用以下标准测试单位，不得自定义

#### Unit_001 (基础战士)
- HP: 100
- MaxHP: 100
- ATK: 30
- DEF: 10
- SPD: 10
- Level: 1

#### Unit_002 (基础法师)
- HP: 80
- MaxHP: 80
- ATK: 40
- DEF: 5
- SPD: 12
- Level: 1

#### Unit_003 (基础坦克)
- HP: 150
- MaxHP: 150
- ATK: 20
- DEF: 20
- SPD: 5
- Level: 1

### Standard Test Skills
#### Skill_001 (普通攻击)
- Damage: 1.0 * ATK
- Range: 1
- TargetType: SingleEnemy

#### Skill_002 (火球术)
- Damage: 1.5 * ATK
- Range: 3
- TargetType: SingleEnemy
```

---

## 7. 行为规则（Rules / Oracle）
```md
## Game Rules

### Damage Formula
Damage = floor((ATK * SkillMultiplier - DEF) * DamageReduction)

### Damage Constraints
- Damage ≥ 1
- HP ≥ 0
- HP ≤ MaxHP

### Attribute Rules
- FinalStat = BaseStat + Sum(FlatModifiers) + BaseStat * Sum(PercentModifiers)
- Modifier优先级：Flat → Percent
- 临时Modifier优先级高于永久Modifier

### Turn Order Rules
- 单位按SPD降序排列
- SPD相同时按Unit ID升序排列
- 回合顺序在战斗开始时确定，战斗过程中不变

### Death Rules
- 当HP ≤ 0时，单位进入Dead状态
- Dead状态的单位不能执行任何操作
- Dead状态的单位会被自动移除出回合顺序
```

---

## 8. 错误与异常（Error Cases）
```md
## Error Handling

### Invalid Inputs
- 攻击不存在的Unit ID
- 对Dead状态的单位使用技能
- 超出技能范围的攻击
- 负数值的属性修改
- 空的技能ID或Buff ID

### Expected Behaviors
- 返回对应的错误码
- 不修改任何游戏状态
- 不崩溃
- 记录Error级别日志

### Error Codes
- E_INVALID_UNIT: 无效的单位ID
- E_INVALID_TARGET: 无效的目标
- E_OUT_OF_RANGE: 超出范围
- E_INVALID_VALUE: 无效的数值
- E_NOT_FOUND: 资源未找到
```

---

## 9. 覆盖率要求（Coverage Requirements）
```md
## Coverage

### Required Coverage
- 100% 战斗状态转换
- 100% 核心规则函数
- 100% 技能类型
- 100% Buff类型
- 95% 边界情况
- 90% 错误处理路径

### Exclusions
- 日志打印代码
- 调试工具代码
- 配置加载代码
```

---

## 10. 测试执行规则（Execution Rules）
```md
## Execution Rules

- 所有测试必须是确定性的
- 所有随机数必须使用固定种子(42)
- 测试之间必须完全独立，不能共享状态
- 测试执行顺序不能影响测试结果
- 所有测试必须能在10秒内完成
- 禁用所有日志输出，除非测试失败
```

---

## 11. 示例测试集（Sample Test Cases）
```md
## Sample Tests

### TC-DMG-001: 基础伤害计算

#### Preconditions
- 攻击者：Unit_001 (ATK=30)
- 防御者：Unit_002 (DEF=5)
- 技能：Skill_001 (Multiplier=1.0)
- 无任何Modifier

#### Input
- 攻击者使用Skill_001攻击防御者

#### Steps
1. 调用calculate_damage(30, 5, 1.0, 0.0)

#### Expected Output
- 伤害值：25

#### Assertions
- assert_eq!(calculate_damage(30, 5, 1.0, 0.0), 25)

#### Postconditions
- 无状态变化

---

### TC-COMB-001: 普通攻击杀死单位

#### Preconditions
- 攻击者：Unit_001 (ATK=30)
- 防御者：Unit_002 (HP=25, DEF=5)
- 战斗状态：ExecutingAction

#### Input
- 攻击者使用Skill_001攻击防御者

#### Steps
1. 发送AttackEvent(attacker=Unit_001, target=Unit_002, skill=Skill_001)
2. 运行CombatSystem
3. 运行DamageResolver
4. 运行DeathSystem

#### Expected Output
- 防御者HP：25 → 0
- 防御者状态：Alive → Dead
- 触发DamageEvent(25)
- 触发DeathEvent(Unit_002)

#### Assertions
- assert_eq!(defender.hp, 0)
- assert!(defender.has_component::<Dead>())
- assert!(events.read::<DeathEvent>().any(|e| e.unit == defender_id))

#### Postconditions
- 防御者处于Dead状态
- 防御者被移出回合顺序
```

---

## 12. AI可执行性增强（非常关键 ⭐⭐⭐）
```md
## AI Execution Constraints

### 绝对禁止
- 使用任何模糊词汇："大概"、"可能"、"通常"、"应该"
- 生成不在测试范围内的测试用例
- 测试实现细节而非行为
- 在测试中复制粘贴被测试的代码
- 使用全局状态在测试之间共享数据
- 将测试标记为#[ignore]而不修复

### 必须遵守
- 所有状态必须使用枚举值，不能使用字符串
- 所有数值必须是具体的，不能使用"高"、"低"、"中等"
- 所有断言必须有明确的预期值
- 所有测试用例必须有唯一的ID
- 所有测试必须遵循固定的格式
- 生成任何业务代码时必须同时生成对应的测试代码

### 优先级
1. Battle Replay测试
2. 回归测试
3. 边界测试
4. 单元测试
5. 集成测试
6. 系统测试
```

---

## 13. Bug修复与回归测试流程
```md
## Bug Fix Process

1. 收到Bug报告后，首先创建对应的回归测试用例
2. 运行测试，确认测试失败
3. 修复代码，直到测试通过
4. 运行所有相关测试，确保没有引入回归
5. 将测试用例添加到永久测试套件中
6. 对于战斗相关Bug，必须同时生成Battle Replay测试
```

---

## 14. AI测试自检清单（强制执行）
```md
## AI Self-Check List

AI生成任何测试代码后，必须自动完成以下检查并在测试文件开头标注结果

// ================================================
// Bevy SRPG AI测试规范 v2.0 自检结果
// ================================================
// ✅ 测试在规范范围内：是/否
// ✅ 遵循固定测试用例格式：是/否
// ✅ 使用标准测试数据：是/否
// ✅ 所有断言有明确预期值：是/否
// ✅ 测试是确定性的：是/否
// ✅ 未测试实现细节：是/否
// ================================================
// ❌ 违反条款：X.X.X（条款编号）
// [测试豁免] 理由：XXX | 有效期：YYYY-MM-DD
// ================================================
```

---

## 15. 豁免机制
```md
## Exemption Rules

- 任何违反本规范的测试必须标注[测试豁免]
- 必须详细说明豁免理由、技术依据和有效期
- 所有豁免测试必须在每3个月的架构复盘时重新评估
- 禁止对🟥绝对禁止条款申请豁免
```

---

# 本规范达到95+分的核心依据
1. **结构100%固定**：15个章节的顺序和名称永久不变，AI可以形成稳定的生成模式
2. **完全schema化**：每个部分都可以被抽象为JSON结构，支持工具自动解析和生成
3. **零歧义**：所有规则都是可枚举、可断言的，没有任何需要人类主观判断的内容
4. **边界绝对清晰**：明确的Non-goals彻底消除了AI的生成歧义
5. **领域深度适配**：在通用框架基础上，深度融合了Bevy ECS和SRPG的特性
6. **可验证性优先**：所有测试结果只有"通过/失败"两种可能，没有中间状态
7. **AI友好度拉满**：专门的AI执行约束和自检清单，让AI知道"怎么做"和"怎么检查自己"

# 下一步升级方向（可选）
1. **JSON Schema版**：将整个规范转换为JSON Schema，支持工具自动校验AI生成的测试用例
2. **DSL编译器**：开发一个简单的DSL编译器，将测试用例直接编译为Rust测试代码
3. **测试生成器**：基于本规范，开发一个自动生成测试用例的AI代理
4. **CI集成**：将自检清单集成到CI流程中，自动检查AI生成的测试代码是否符合规范

需要我把这份规范转换成**纯提示词版**（去掉所有解释性内容，只保留AI需要执行的规则），或者**JSON Schema版**吗？