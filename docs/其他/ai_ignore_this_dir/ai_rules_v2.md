# AI 领域

Version: 2.0

## Purpose

AI 领域管理敌方单位的自动决策。采用数据驱动的行为配置 + 策略 Trait 扩展点，替代硬编码 AI 逻辑。AI 与玩家共用 Effect Pipeline，CombatIntent 是唯一攻击意图通道。

---

## Glossary

| 术语 | 定义 | 易混淆项 |
|------|------|----------|
| AiBehavior | AI 行为配置，定义目标/移动/技能策略的组合 | ≠ AiStrategy：Behavior 是配置，Strategy 是策略实现 |
| TargetSelector | 目标选择策略，决定 AI 攻击谁 | ≠ MoveSelector：Target 选攻击目标，Move 选移动位置 |
| MoveSelector | 移动选择策略，决定 AI 移动到哪里 | ≠ TargetSelector：Move 选位置，Target 选目标 |
| SkillSelector | 技能选择策略，决定 AI 使用什么技能 | ≠ TargetSelector：Skill 选技能，Target 选目标 |
| UnitSnapshot | 单位快照，AI 决策时的纯数据视图 | ≠ ECS Query：Snapshot 是快照，避免借用冲突 |
| CombatIntent | 攻击意图组件，AI 和玩家共用 | ≠ AI 专属组件：是唯一攻击意图通道 |

---

## Responsibilities

### Owns

- AiBehavior 定义和注册表
- 策略 Trait 定义（TargetSelector / MoveSelector / SkillSelector）
- 策略注册表
- 单位快照收集
- AI 决策流程

### Does Not Own

- 效果管线执行 → effect_pipeline
- 寻路计算 → map_rules
- 回合推进 → turn_rules
- UI 展示 → ui_rules
- 伤害计算 → effect_pipeline

---

## Invariants

### INV-AI-01：AI 和玩家共用 Effect Pipeline 🟥

宪法：1.1.4

AI 设置的攻击意图与玩家设置的走同一条效果管线。AI 禁止包含独立伤害计算逻辑。

违反：AI 攻击绕过效果管线直接扣血。

### INV-AI-02：CombatIntent 是唯一攻击意图通道 🟥

宪法：7.0.5

AI 的攻击意图只能通过 CombatIntent 组件表达。

违反：AI 直接调用伤害函数或创建独立攻击逻辑。

### INV-AI-03：策略名称与配置对应 🟩

AiBehavior 中的策略名称必须与注册表中的实现对应。未知名称回退到默认策略。

违反：策略查找失败，AI 使用默认策略。

### INV-AI-04：技能冷却检查 🟩

选择的技能必须不在冷却中。

违反：AI 使用冷却中的技能。

### INV-AI-05：Rule / Content 分离 🟥

宪法：1.1.3

新增 AI 行为只修改 RON 配置，不修改核心规则代码。新增策略只需实现 Trait 并注册。

违反：新增 AI 行为时修改了决策流程代码。

### INV-AI-06：策略 Trait 替代 enum+match 🟩

宪法：6.0.2

策略通过注册表查找分发，禁止 match 分发策略类型。

违反：新增策略需要修改分发代码。

### INV-AI-07：UnitSnapshot 避免借用冲突 🟩

宪法：2.1.2

AI 决策时所有 ECS 数据通过快照访问，不持有 ECS 可变引用。

违反：AI 系统直接 Query 可变组件，运行时 panic。

---

## State Machine

### AI 决策状态

| 状态 | 含义 | 转换到 |
|------|------|--------|
| Waiting | 等待计时器 | Deciding |
| Deciding | 收集快照 + 策略选择 | Acting |
| Acting | 设置攻击意图和移动意图 | Done |
| Done | 标记已行动 | — |

```
Waiting → Deciding → Acting → Done
```

| 从 | 到 | 条件 |
|----|-----|------|
| Waiting | Deciding | 计时器到期 + 当前单位是敌方 |
| Deciding | Acting | 策略选择完成 |
| Acting | Done | 攻击意图和移动意图已设置 |

---

## Business Rules

### BR-AI-01：AI 决策流程

- 收集所有单位快照
- 获取 AI 行为配置
- 按顺序选择目标 → 移动 → 技能
- 设置攻击意图和移动意图

### BR-AI-02：行动结果

| 条件 | 行动 |
|------|------|
| 有攻击目标 + 需移动 | 移动 → 执行攻击 |
| 有攻击目标 + 不需移动 | 执行攻击 |
| 无攻击目标 + 需移动 | 移动 → 待机 |
| 无攻击目标 + 不需移动 | 待机 |

### BR-AI-03：CautiousMove 保持距离

- 筛选攻击范围内的可达位置
- 有范围内位置 → 选择距目标最远的
- 无范围内位置 → 贪心靠近

---

## Pipelines

### AI 决策管线

计时器检查 → 快照收集 → 目标选择 → 寻路 → 移动选择 → 技能选择 → 设置意图

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 计时器检查 | 计时器 + 回合阶段 + 行动队列 | 是否执行 | 禁止跳过计时器 |
| 快照收集 | 所有单位数据 | 快照列表 | 禁止持有可变引用（INV-AI-07） |
| 策略选择 | 行为配置 + 注册表 + 快照 | 目标 + 位置 + 技能 | 禁止跳过任何策略步骤 |
| 设置意图 | 策略选择结果 | 意图组件 | 禁止直接执行效果（INV-AI-01） |

---

## Data Model

### AiBehavior（Definition）

AI 行为配置，不可变。

- 标识：id / name
- 策略名称：目标策略 / 移动策略 / 技能策略
- 技能优先级列表
- 配置来源：RON（assets/ai/）

### TargetSelector（Trait）

目标选择策略接口。

- 输入：候选目标 + 自身坐标
- 输出：目标坐标
- 内置四种实现，未知名称回退 "Nearest"

### MoveSelector（Trait）

移动选择策略接口。

- 输入：可达范围 + 自身坐标 + 目标坐标 + 攻击距离
- 输出：移动位置
- 内置三种实现，未知名称回退 "Aggressive"

### SkillSelector（Trait）

技能选择策略接口。

- 输入：可用技能 + 冷却状态 + 优先级
- 输出：技能 ID
- 内置三种实现，未知名称回退 "PreferSpecial"
- 跳过冷却中的技能

### UnitSnapshot（值对象）

AI 决策时的纯数据视图，不持有 ECS 引用。

- 身份：entity / faction / coord
- 战斗属性：atk / hp / max_hp / mov / attack_range
- 技能：skill_ids / cooldowns / ai_behavior_id / tags

---

## Cross Domain Contracts

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 攻击意图 | CombatIntent 组件 | battle |
| 移动意图 | MovingUnit 组件 | battle |
| 寻路请求 | 函数调用 | map |
| 地形成本 | 函数调用 | map |

---

## Change Rules

### 新增 AI 策略

- 允许：新增 Trait 实现 + 注册到注册表
- 禁止：修改决策流程（违反 INV-AI-05）、修改快照结构（除非必要）
- 检查：注册表注册、策略名称与配置对应、回退策略正确

### 新增 AI 行为

- 允许：新增 RON 配置
- 禁止：硬编码行为逻辑（违反 INV-AI-05）、修改核心规则代码
- 检查：注册表注册、策略名称在注册表中、技能 ID 存在

### 修改 AI 决策流程

- 允许：修改策略调用顺序
- 禁止：跳过策略查找直接硬编码、直接执行效果（违反 INV-AI-01）、独立计算伤害（违反 INV-AI-01）
- 检查：意图组件是否正确设置、行动结果是否正确处理

---

## Architecture Violations

发现架构违规时统一输出：

```
ARCHITECTURE VIOLATION:
Rule: <RuleID>
Reason: <Why>
Fix: <How>
```

| RuleID | 违规行为 | Reason | Fix |
|--------|----------|--------|-----|
| INV-AI-01 | AI 模块包含独立伤害计算 | AI 与玩家必须共享 Effect Pipeline | 删除独立伤害计算，通过 CombatIntent 走管线 |
| INV-AI-02 | 绕过 CombatIntent 发起攻击 | CombatIntent 是唯一攻击意图通道 | 通过 CombatIntent 设置攻击意图 |
| INV-AI-05 | 新增 AI 行为修改了规则代码 | Rule/Content 分离 | 改为 RON 配置实现 |
| INV-AI-06 | match 分发策略类型 | 策略通过注册表分发 | 改为注册表查找 |

---

## Test Requirements

宪法：13.0.1-13.0.3

- 单元测试：验证策略选择逻辑
- 集成测试：验证完整 AI 决策流程
- Bug 修复必须先编写重现测试

排查顺序：
1. AiBehavior 是否正确加载
2. 策略名称是否与注册表对应
3. 技能冷却是否正确检查
4. 快照数据是否正确
5. 意图组件是否正确设置
