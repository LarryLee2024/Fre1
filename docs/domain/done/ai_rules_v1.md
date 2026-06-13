# AI 领域

Version: 1.1

AI 领域管理敌方单位的自动决策。采用数据驱动的行为配置 + 策略 trait 扩展点，替代硬编码的 AI 逻辑。

核心原则：
- 🟩 Trait 描述规则，不描述内容（宪法 6.0.1）
- 🟩 数据驱动（宪法 1.1.5）
- 🟩 注册表分发
- 🟥 Rule / Content 分离（宪法 1.1.3）
- 🟩 组合优于继承（宪法 1.1.6）

---

# 术语定义

## AiBehavior

AI 行为配置，定义目标/移动/技能策略的组合。

不是 AiStrategy。Behavior 是配置，Strategy 是策略实现。

关键属性：
- target_strategy / move_strategy / skill_strategy：策略名称
- skill_priority：技能优先级列表

---

## TargetSelector

目标选择策略，决定 AI 攻击谁。

不是 MoveSelector。Target 选择攻击目标，Move 选择移动位置。

关键属性：
- strategy_name()：策略名称
- select()：从候选单位中选择目标

---

## MoveSelector

移动选择策略，决定 AI 移动到哪里。

不是 TargetSelector。Move 选择位置，Target 选择目标。

关键属性：
- strategy_name()：策略名称
- select()：从可达范围中选择位置

---

## SkillSelector

技能选择策略，决定 AI 使用什么技能。

不是 TargetSelector。Skill 选择技能，Target 选择目标。

关键属性：
- strategy_name()：策略名称
- select()：从可用技能中选择

---

## UnitSnapshot

单位快照，AI 决策时的纯数据视图。

不是 ECS Query。Snapshot 是快照，避免借用冲突。

关键属性：
- entity / faction / coord / atk / hp / max_hp / mov / attack_range
- skill_ids / cooldowns / ai_behavior_id / tags

---

## CombatIntent

攻击意图组件，AI 和玩家共用，由 Effect Pipeline 统一处理。

不是 AI 专属组件。CombatIntent 是唯一攻击意图通道（宪法 7.0.5，架构 AI 模块边界）。

---

# 领域边界

## 本领域负责

- AiBehavior 定义和注册表（AiBehaviorRegistry）
- 策略 Trait 定义（TargetSelector / MoveSelector / SkillSelector）
- 策略注册表（AiStrategyRegistry）
- UnitSnapshot 收集
- enemy_ai_system 决策流程

## 本领域不负责

- 效果管线执行（由 effect_pipeline 领域负责）
- 寻路计算（由 map_rules 领域负责）
- 回合推进（由 turn_rules 领域负责）
- UI 展示（由 ui_rules 领域负责）
- 伤害计算（由 effect_pipeline 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 攻击意图 | CombatIntent 组件 | battle |
| 移动意图 | MovingUnit 组件 | battle |
| 寻路请求 | find_reachable_tiles 调用 | map |
| 地形成本 | resolve_from_tags 调用 | map |

---

# 生命周期

## AI 决策生命周期

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Waiting | 等待计时器 | Deciding |
| Deciding | 收集快照 + 策略选择 | Acting |
| Acting | 设置 CombatIntent / MovingUnit | Done |
| Done | 标记 acted | — |

## 状态转换图

Waiting → Deciding → Acting → Done

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Waiting | Deciding | AI 计时器到期 + 当前单位是敌方 |
| Deciding | Acting | 策略选择完成 |
| Acting | Done | CombatIntent / MovingUnit 已设置 |

---

# 不变量

## 不变量1：AI 和玩家共用 Effect Pipeline 🟥

任意时刻：

AI 设置的 CombatIntent 与玩家设置的走同一条效果管线。

违反表现：

AI 攻击绕过效果管线，直接扣血。

架构违规检测：

发现 AI 模块包含独立伤害计算时，必须停止。必须输出：

```
ARCHITECTURE VIOLATION: AI 模块包含独立伤害计算，违反"AI 与玩家共享 Effect Pipeline"原则。
```

---

## 不变量2：CombatIntent 是唯一攻击意图通道 🟥

任意时刻：

AI 的攻击意图只能通过 CombatIntent 组件表达，禁止使用其他方式触发攻击。

违反表现：

AI 直接调用伤害函数或创建独立攻击逻辑。

---

## 不变量3：UnitSnapshot 避免借用冲突 🟩

enemy_ai_system 执行时：

所有 ECS 数据通过 UnitSnapshot 快照访问，不持有 ECS 引用。

违反表现：

AI 系统直接 Query 可变组件，与其他系统借用冲突。

---

## 不变量4：策略名称与 RON 对应 🟩

AiBehavior 加载完成后：

strategy_name 与 RON 配置中的 enum variant 名一致。

违反表现：

策略查找失败，回退到默认策略。

---

## 不变量5：技能冷却检查 🟩

SkillSelector.select 完成后：

选择的技能不在冷却中。

违反表现：

AI 使用冷却中的技能。

---

# 业务规则

## 规则1：策略 trait 替代 enum+match 🟩

宪法依据：6.0.2（Trait 用于扩展点）

禁止：
- match 分发策略类型
- 为每种策略修改决策流程

必须：
- 通过 strategy_name() 查找 trait 对象
- 新增策略只需实现 trait 并注册
- 注册表回退机制（未知名称 → 默认策略）

---

## 规则2：AI 决策流程 🟩

禁止：
- 跳过计时器直接行动
- 跳过技能冷却检查
- 直接修改 ECS 状态

必须：
- 收集所有单位快照
- 获取 AI 行为配置
- 按顺序选择目标 → 移动 → 技能
- 设置 CombatIntent / MovingUnit

---

## 规则3：行动结果 🟩

禁止：
- 无目标时不移动

必须：
- 有攻击目标 + 需移动 → MovingUnit → ExecuteAction
- 有攻击目标 + 不需移动 → ExecuteAction
- 无攻击目标 + 需移动 → MovingUnit → WaitAction
- 无攻击目标 + 不需移动 → WaitAction

---

## 规则4：CautiousMove 保持距离 🟩

禁止：
- 贪心靠近目标

必须：
- 筛选攻击范围内的可达位置
- 有范围内位置 → 选择距目标最远的
- 无范围内位置 → 贪心靠近

---

## 规则5：Rule / Content 分离 🟥

宪法依据：1.1.3（规则与内容强制分离）

禁止：
- 🟥 新增 AI 行为时修改核心规则代码
- 🟥 硬编码行为逻辑

必须：
- 新增 AI 行为只修改 RON 配置
- 新增 AI 策略只需实现 trait 并注册

发现为了新增 AI 行为而修改 enemy_ai_system 核心流程时，必须停止。必须输出：

```
ARCHITECTURE VIOLATION: 新增内容修改了规则代码 [文件名]，违反 Rule/Content 分离原则。
应通过 RON 配置实现，而非修改代码。
```

---

# 流程管线

## AI 决策管线

计时器检查 → 快照收集 → 目标选择 → 寻路 → 移动选择 → 技能选择 → 设置意图

### Step1：计时器检查

输入：AiTimer + TurnPhase + TurnOrder
处理：检查是否是敌方回合 + 计时器是否到期
输出：是否执行 AI
🟥 禁止：跳过计时器

### Step2：快照收集

输入：所有单位的 ECS 数据
处理：构建 UnitSnapshot 列表
输出：Vec<UnitSnapshot>
🟥 禁止：持有 ECS 可变引用（宪法 2.1.2）

### Step3：策略选择

输入：AiBehavior + AiStrategyRegistry + UnitSnapshot
处理：目标选择 → 移动选择 → 技能选择
输出：目标坐标 + 移动坐标 + 技能 ID
🟩 禁止：跳过任何策略步骤

### Step4：设置意图

输入：策略选择结果
处理：设置 CombatIntent / MovingUnit
输出：ECS 状态变化
🟥 禁止：直接执行效果（必须走 Effect Pipeline）

---

# 数据结构

## AiBehavior（Definition）

职责：AI 行为配置

结构：
- id / name：标识和展示
- target_strategy / move_strategy / skill_strategy：策略名称
- skill_priority：技能优先级列表

要求：
- 🟥 不可变，加载后不修改（宪法 1.1.2）
- RON 配置路径：assets/ai/
- 策略名称与 trait 的 strategy_name() 对应

---

## TargetSelector（Trait）

职责：目标选择策略

结构：
- strategy_name()：策略名称
- select(candidates, my_coord)：选择目标坐标

要求：
- 🟩 用于定义需要扩展的接口（宪法 6.0.2）
- 内置四种实现
- 未知名称回退 "Nearest"

---

## MoveSelector（Trait）

职责：移动选择策略

结构：
- strategy_name()：策略名称
- select(reachable, my_coord, target_coord, attack_range)：选择移动位置

要求：
- 🟩 用于定义需要扩展的接口（宪法 6.0.2）
- 内置三种实现
- 未知名称回退 "Aggressive"

---

## SkillSelector（Trait）

职责：技能选择策略

结构：
- strategy_name()：策略名称
- select(skill_ids, cooldowns, priority)：选择技能 ID

要求：
- 🟩 用于定义需要扩展的接口（宪法 6.0.2）
- 内置三种实现
- 未知名称回退 "PreferSpecial"
- 跳过冷却中的技能

---

## UnitSnapshot（值对象）

职责：AI 决策时的纯数据视图

结构：
- entity / faction / coord / atk / hp / max_hp / mov / attack_range
- skill_ids / cooldowns / ai_behavior_id / tags

要求：
- 🟩 纯数据，不持有 ECS 引用（宪法 2.1.2）
- 🟩 Entity 仅作为 ID 传递（宪法 2.1.1）

---

# 禁止事项

🟥 禁止：AI 独立执行攻击逻辑

原因：AI 和玩家共用 Effect Pipeline（架构 AI 模块边界）

违反后果：AI 攻击绕过管线，效果未经修饰，伤害计算不一致

架构违规检测：

```
ARCHITECTURE VIOLATION: AI 模块包含独立伤害计算，违反"AI 与玩家共享 Effect Pipeline"原则。
```

---

🟥 禁止：AI 独立计算伤害

原因：伤害计算由 Effect Pipeline 统一处理（架构 AI 模块边界）

违反后果：AI 伤害与玩家伤害计算不一致

架构违规检测：

```
ARCHITECTURE VIOLATION: AI 模块包含独立伤害计算，违反"AI 与玩家共享 Effect Pipeline"原则。
```

---

🟥 禁止：match 分发策略类型

原因：策略通过 strategy_name() 查找 trait 对象（宪法 6.0.2）

违反后果：新增策略需要修改分发代码，违反 Rule/Content 分离

---

🟥 禁止：跳过技能冷却检查

原因：冷却中的技能不可使用

违反后果：AI 使用冷却中的技能

---

🟥 禁止：AI 直接执行效果

原因：AI 和玩家共用 Effect Pipeline（宪法 1.1.4）

违反后果：AI 攻击绕过管线，效果未经修饰

---

🟥 禁止：AI 系统持有 ECS 可变引用

原因：避免借用冲突（宪法 2.1.2）

违反后果：运行时 panic

---

🟩 禁止：跳过 AI 计时器

原因：0.8 秒延迟让玩家看到 AI 行动

违反后果：AI 瞬间完成所有行动

---

🟥 禁止：硬编码 AI 行为逻辑

原因：Rule / Content 分离（宪法 1.1.3）

违反后果：新增 AI 行为需要修改代码

---

# AI 修改规则

## 如果新增 AI 策略

允许：
- 新增 TargetSelector / MoveSelector / SkillSelector 实现
- 注册到 AiStrategyRegistry

禁止：
- 🟥 修改 enemy_ai_system 流程
- 🟥 修改 UnitSnapshot 结构（除非必要）

优先检查：
- AiStrategyRegistry 注册
- strategy_name() 是否与 RON 对应
- 回退策略是否正确
- 🟩 是否重复出现三次以上才抽象为 Trait（宪法 6.0.3）

---

## 如果新增 AI 行为

允许：
- 新增 AiBehavior RON 配置

禁止：
- 🟥 硬编码行为逻辑
- 🟥 修改核心规则代码

优先检查：
- AiBehaviorRegistry 注册
- 策略名称是否在 AiStrategyRegistry 中
- skill_priority 中的技能 ID 是否存在

---

## 如果修改 AI 决策流程

允许：
- 修改 enemy_ai_system 中的策略调用顺序

禁止：
- 🟥 跳过策略查找直接硬编码
- 🟥 直接执行效果
- 🟥 独立计算伤害

优先检查：
- CombatIntent 是否正确设置
- MovingUnit 是否正确设置
- 行动结果是否正确处理

---

## 如果测试失败

排查顺序：
1. 检查 AiBehavior 是否正确加载
2. 检查策略名称是否与注册表对应
3. 检查技能冷却是否正确检查
4. 检查 UnitSnapshot 数据是否正确
5. 检查 CombatIntent / MovingUnit 是否正确设置

测试要求（宪法 13.0.1-13.0.3）：
- 🟩 单元测试：验证策略选择逻辑
- 🟩 集成测试：验证完整 AI 决策流程
- 🟩 Bug 修复必须先编写重现测试（宪法 13.0.2）

---

# 宪法条款映射

| 宪法条款 | 本领域对应 |
|----------|-----------|
| 1.1.2 Definition/Instance 分离 | AiBehavior(Definition) vs AiStrategyRegistry(Instance) |
| 1.1.3 Rule/Content 分离 | 策略 trait 是规则，RON 配置是内容 |
| 1.1.4 Logic/Presentation 分离 | AI 决策不包含 UI/动画逻辑 |
| 1.1.5 数据驱动 | AiBehavior 从 RON 加载 |
| 1.1.6 组合优于继承 | 策略组合而非继承树 |
| 2.1.1 Entity 只是 ID | UnitSnapshot 中 Entity 仅作 ID |
| 2.1.2 数据与行为分离 | UnitSnapshot 纯数据 |
| 6.0.2 Trait 用于扩展点 | TargetSelector/MoveSelector/SkillSelector |
| 6.0.3 Trait 抽象原则 | 策略出现三次以上才抽象 |
| 7.0.5 战斗事件链 | CombatIntent 是唯一攻击意图通道 |

---

# 架构违规检测

| 违规行为 | 检测方式 | 输出 |
|----------|----------|------|
| AI 模块包含独立伤害计算 | 代码审查 | ARCHITECTURE VIOLATION: AI 模块包含独立伤害计算，违反"AI 与玩家共享 Effect Pipeline"原则。 |
| AI 直接修改 ECS 业务状态 | 代码审查 | ARCHITECTURE VIOLATION: AI 直接修改业务状态，违反 Logic/Presentation 分离原则。 |
| 新增 AI 行为修改规则代码 | 代码审查 | ARCHITECTURE VIOLATION: 新增内容修改了规则代码 [文件名]，违反 Rule/Content 分离原则。 |
