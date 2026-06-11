---
alwaysApply: false
description: @测试卫士 agent、@test-guardian、写测试代码时，只要与测试相关就生效
---
> **最高约束力**：本规范优先级高于任何通用测试最佳实践，与《Bevy 0.18+ SRPG AI开发宪法v1.1》配套执行。任何违反本规范的测试代码均视为不合格输出。

## 0. 元信息与边界
- 版本：v2.0.0
- 覆盖范围：战斗、角色、属性、技能、Buff、装备、地图、回合管理系统
- **绝对不测**：UI视觉/动画、性能压测、引擎/第三方库、美术音效、网络同步

## 1. 测试目标
验证：核心规则正确性、状态转换一致性、边界行为稳定性、错误处理能力、历史Bug永久防护

## 2. 测试分类（优先级从高到低）
1. **Battle Replay测试**：所有战斗Bug必须转化为此类，优先级最高
2. **回归测试**：所有历史修复Bug对应的测试用例
3. **边界测试**：极端输入/状态
4. **单元测试**：单一纯函数，不启动Bevy App
5. **集成测试**：多模块交互，最小化Bevy App
6. **系统测试**：完整业务流程

## 3. 状态模型（必须使用枚举值）
- 战斗状态：Idle → SelectingUnit → SelectingAction → SelectingTarget → ExecutingAction → ResolvingEffects → TurnEnd → Idle；ResolvingEffects可跳转至CombatVictory/CombatDefeat
- 单位状态：Alive、Dead、Stunned、Frozen、Poisoned、Silenced

## 4. 测试用例强制格式（不得增减字段）
每个测试必须包含：
- TC-[模块缩写]-[三位数字]（唯一ID）
- Title（场景+预期）
- Preconditions（具体数值/状态）
- Input（具体操作/事件）
- Steps（编号步骤）
- Expected Output（状态变化+数值变化+事件）
- Assertions（明确断言，无模糊描述）
- Postconditions（最终状态）

## 5. 测试数据规范
必须使用标准测试单位，不得自定义：
- Unit_001(战士)：HP=100, ATK=30, DEF=10, SPD=10
- Unit_002(法师)：HP=80, ATK=40, DEF=5, SPD=12
- Unit_003(坦克)：HP=150, ATK=20, DEF=20, SPD=5
- Skill_001(普攻)：1.0*ATK，范围1，单体敌人
- Skill_002(火球)：1.5*ATK，范围3，单体敌人

## 6. 核心规则（必须严格遵守）
- 伤害公式：Damage = floor((ATK*SkillMultiplier - DEF)*DamageReduction)
- 伤害约束：Damage≥1，0≤HP≤MaxHP
- 属性公式：FinalStat = BaseStat + Sum(FlatModifiers) + BaseStat*Sum(PercentModifiers)
- 回合顺序：按SPD降序，相同按Unit ID升序，战斗开始后固定
- 死亡规则：HP≤0进入Dead状态，不能执行操作，移出回合顺序

## 7. 错误处理
- 错误码：E_INVALID_UNIT、E_INVALID_TARGET、E_OUT_OF_RANGE、E_INVALID_VALUE、E_NOT_FOUND
- 预期行为：返回对应错误码，不修改状态，不崩溃，记录Error日志

## 8. 覆盖率要求
- 100%战斗状态转换、核心规则函数、技能类型、Buff类型
- 95%边界情况，90%错误处理路径
- 排除：日志、调试工具、配置加载代码

## 9. 执行规则
- 所有测试必须确定性，随机数固定种子42
- 测试完全独立，无共享状态，执行顺序不影响结果
- 单个测试必须在10秒内完成，禁用非失败日志

## 10. AI执行约束
### 绝对禁止
- 使用"大概、可能、通常、应该"等模糊词
- 生成不在范围内的测试
- 测试实现细节而非行为
- 复制粘贴被测试代码
- 共享测试状态
- 标记#[ignore]而不修复

### 必须遵守
- 所有状态用枚举，所有数值具体
- 所有断言有明确预期值
- 所有测试用例有唯一ID
- 生成业务代码时必须同时生成对应测试
- 生成后必须完成自检并标注结果

## 11. Bug修复流程
1. 先写重现测试
2. 确认测试失败
3. 修复代码至测试通过
4. 运行所有相关测试
5. 添加至永久测试套件
6. 战斗Bug必须同时生成Battle Replay测试

## 12. 强制执行
生成任何测试代码后，必须在文件开头标注以下自检结果：
```rust
// Bevy SRPG AI测试规范v2.0自检结果
// ✅ 测试在范围内：是/否
// ✅ 遵循固定格式：是/否
// ✅ 使用标准数据：是/否
// ✅ 断言明确：是/否
// ✅ 测试确定性：是/否
// ✅ 未测试实现细节：是/否
// ❌ 违反条款：X.X.X
// [测试豁免] 理由：XXX | 有效期：YYYY-MM-DD
```
