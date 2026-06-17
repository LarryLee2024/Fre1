---
id: 03-technical.command-bus-rules
title: Command Bus Rules
status: draft
owner: feature-developer
created: 2026-06-14
updated: 2026-06-14
tags:
  - technical
---

# 命令总线领域

Version: 1.0
Status: Proposed

命令总线领域管理所有玩家和 AI 操作的统一抽象通道，将操作封装为 Command 对象，经过校验后执行。

核心原则（对应宪法第十一部分 11.5 命令层）：
- 🟩 11.5.1 所有操作入口为命令（玩家/AI/回放/网络统一转换为 Command）
- 🟩 11.5.2 命令无差别执行（执行系统不区分命令来源）
- 🟥 11.5.2 绝对禁止 AI 和玩家使用不同的伤害计算路径
- 所有操作皆命令，禁止直接修改游戏状态
- 校验只读、执行只写，两阶段严格分离
- Player 和 AI 使用完全相同的 Command 类型和执行路径

---

# 术语定义

## 命令总线（Command Bus）

所有玩家和 AI 操作的统一抽象通道，负责将 UiCommand / AiCommand 转换为领域 Command 对象并调度执行。

不是输入处理。不是 Effect Pipeline。

关键属性：
- 接收 UI 层的 UiCommand 和 AI 层的 AiCommand
- 通过 CommandHandler 转换为领域 Command 对象
- 调度 CommandQueue 执行
- 与 Effect Pipeline 集成（命令执行后触发效果管线）

---

## 命令（Command）

> 🟩 对应宪法 11.5.1：所有操作入口为命令

封装操作意图的对象，包含校验逻辑和执行逻辑，是游戏操作的统一接口。

不是直接状态修改。不是 Message。

关键属性：
- trait 接口：validate() → Result, execute() → CommandResult
- 校验阶段只读，执行阶段修改状态
- 支持撤销（undo）和回放（replay）导出
- 必须使用 Strong ID（参见 shared_layer_rules.md#Strong ID）

---

## 命令校验（Command Validation）

执行前的只读检查，验证操作合法性（权限、消耗、目标合法性）。

不是执行。不是状态修改。

关键属性：
- 只读，不修改任何游戏状态
- 返回 Ok(()) 或 Err(ValidationError)
- 检查内容：单位存在性、存活状态、是否已行动、MP 消耗、技能冷却、目标范围
- 校验失败时返回具体错误类型

---

## 命令执行（Command Execution）

验证通过后的状态变更，信任校验层结果，不重复校验。

不是校验。

关键属性：
- 修改游戏状态（扣 MP、设冷却、移动位置等）
- 不重复验证，信任 validate 结果
- 触发 Effect Pipeline（CombatIntent）
- 发送领域事件（Message）

---

## 命令队列（Command Queue）

待执行和已执行命令的有序列表，支持撤销、回放导出和批量原子执行。

不是 Message 队列。不是指令列表。

关键属性：
- 待执行缓冲（pending）：等待校验和执行的命令
- 已执行历史（executed）：记录执行结果和 tick 编号
- 支持 undo_last（撤销最后一个可撤销命令）
- 支持 export_for_replay（导出命令序列用于回放）

---

## 批量原子执行（Batch Atomic Execution）

全部命令预校验通过后全部执行，任一校验失败则整批拒绝。

不是逐个执行。

关键属性：
- 预校验所有命令（只读）
- 任何一条校验失败，整批返回 ValidationFailed
- 全部通过后逐个执行
- 确保"全有或全无"语义

---

## 命令回退（Command Undo）

> **优化来源**: docs/01-architecture/command_bus_design.md — Memento 模式替代手写 undo()

撤销已执行命令的效果，恢复到命令执行前的状态。

不是 Ops 日志。不是回放。

关键属性：
- 仅支持 is_undoable() 返回 true 的命令
- 从已执行历史中找到最后一个可撤销命令
- 调用 undo() 恢复状态
- 不支持撤销的命令返回 ExecutionError

---

## GameCommand Enum

> **优化来源**: docs/01-architecture/command_bus_design.md — 将 Box\<dyn Command\> 重构为可序列化枚举

替代 Box\<dyn Command\> 的具体枚举，支持 RON 序列化。

不是 trait object。不是动态分发。

关键属性：
- 所有命令类型集中在一个 enum 中（MoveCommand、CastSkillCommand、UseItemCommand 等）
- 支持 RON 序列化/反序列化（存储和网络传输）
- 编译时确定所有变体，无动态分发开销
- Player 和 AI 共用同一 Enum（仅 CommandSource 字段区分来源）

---

## Memento 撤销

> **优化来源**: docs/01-architecture/command_bus_design.md — 放弃反向操作 undo，改用状态快照

状态快照方式替代手写 undo()。

不是反向操作链。不是 Ops 日志。

关键属性：
- 命令执行前保存受影响实体的状态快照（StateSnapshot）
- 撤销时恢复快照，覆盖当前状态
- 无需为每个命令编写反向操作
- Effect Pipeline 连锁反应由快照自动覆盖
- CommandHistory 维护快照栈，最大历史长度防止内存溢出

> **优化来源**: docs/01-architecture/command_bus_design.md — Memento 模式：代码量减少 80%，绝对可靠

---

## Exclusive System

\&mut World 独占访问系统，Command 执行必须在此类 System 中。

不是普通 System。不是 Parallel System。

关键属性：
- 获取 &mut World 独占访问，允许修改任意 ECS 状态
- Command 的 validate() 和 execute() 必须在 Exclusive System 中执行
- 不与其他 System 并行运行（保证执行期间无竞争）
- Bevy 中通过 ExclusiveSystemSet 或 schedule 独占阶段配置

> **优化来源**: docs/01-architecture/command_bus_design.md — Command Bus 核心执行必须声明为 Exclusive System

---

## Cursor-based 执行（游标执行）

> **优化来源**: docs/01-architecture/command_bus_design.md — 游标指针追踪执行位置，支持断点恢复和分段执行

命令队列的游标指针追踪当前执行位置，支持暂停、恢复和重放。

不是一次性全量执行。不是随机访问。

关键属性：
- cursor: usize 游标指针追踪当前执行位置
- execute_from_cursor()：从当前游标位置开始执行，校验失败时停在失败位置
- pause()：保存当前 cursor 位置，支持断点恢复
- resume_from(position)：从指定位置恢复执行
- reset_cursor()：重置游标到队列开头（用于重放）
- 校验失败时 cursor 不前进，支持从失败位置继续

> **优化来源**: docs/01-architecture/command_bus_design.md — 游标执行模型：断点恢复、分段执行、重放支持

---

## ActionQueue（效果执行队列）

> **优化来源**: docs/01-architecture/command_bus_design.md §18 — ActionQueue 顺序执行队列

技能释放后效果的顺序执行容器，确保伤害、Buff、死亡、反击等效果按确定性顺序链式执行。

不是 CommandQueue。不是并行处理。

关键属性：
- 记录内容：效果执行（伤害、Buff、死亡判定、反击），非用户操作
- 生产者：Effect Pipeline，非 Player/AI
- 执行时机：效果结算阶段，非行动阶段
- 数据粒度：细粒度（一个 Action = 一个效果步骤），非粗粒度
- 核心价值：效果执行顺序确定性，每步结果影响下一步

ActionQueue vs CommandQueue 区别：

| | CommandQueue | ActionQueue |
|---|-------------|-------------|
| 记录内容 | 用户操作（移动、释放技能） | 效果执行（伤害、Buff、死亡判定） |
| 生产者 | Player Input / AI Decision | Effect Pipeline |
| 执行时机 | 玩家/AI 行动阶段 | 效果结算阶段 |
| 数据粒度 | 粗粒度（一个 Command = 一个操作） | 细粒度（一个 Action = 一个效果步骤） |
| 核心价值 | 操作可回滚/可回放 | 效果执行顺序确定性 |

> **优化来源**: docs/01-architecture/command_bus_design.md §18 — ActionQueue 链式执行：DamageAction → BuffAction → DeathCheckAction → CounterAttackAction

---

## 三大基本命令类型

> **优化来源**: docs/01-architecture/command_bus_design.md §13 — SRPG 核心操作覆盖

SRPG 的三大基本命令类型覆盖核心操作，Player 和 AI 共用：

| 命令类型 | 对应操作 | 模式说明 |
|----------|---------|---------|
| MoveCommand | 单位移动 | 移动是 SRPG 最高频操作 |
| CastSkillCommand | 释放技能 | 技能是战斗核心 |
| UseItemCommand | 使用物品 | 道具是策略维度 |

关键属性：
- 三种类型均通过 GameCommand Enum 集中定义
- 均支持 RON 序列化/反序列化
- 均经过校验层 → 执行层两阶段处理
- Command Pattern 直接受益于 Replay / AI / 联机

---

# 领域边界

## 本领域负责

- Command trait 定义和 CommandResult 类型
- 命令队列（CommandQueue）的管理和调度
- 校验层和执行层的职责分离
- 批量原子执行（预校验 + 全部执行）
- 命令回退（undo）和回放导出
- Player 和 AI 的命令生成统一
- 与 Effect Pipeline 的集成点

## 本领域不负责

- 具体命令类型的业务规则（由各功能领域负责）
- UI 交互和输入处理（由 Input / UI 领域负责）
- AI 策略选择和行为定义（由 AI 领域负责）
- Effect Pipeline 的内部执行（由 Attribute Modifier 领域负责）
- 战斗回合状态机（由 Turn 领域负责）
- 消息通信机制（由 ECS Communication 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| UiCommand | Message | 命令总线（CommandHandler） |
| AiCommand | 函数调用 | 命令总线（CommandHandler） |
| CommandResult | 函数调用返回 | UI / AI 领域 |
| CombatIntent | Resource | Attribute Modifier 领域（Effect Pipeline） |
| 命令执行结果 | Message | UI / Debug / Audit 领域 |

---

# 生命周期

本领域无状态机，为纯函数式计算。

命令的生命周期为：生成 → 校验 → 执行（或拒绝）→ 记录历史。

CommandQueue 的生命周期为：接收命令 → 预校验 → 执行 → 记录到历史。

---

# 不变量

## 不变量1：每个操作必须封装为 Command

任意时刻：

玩家的每个操作（移动、攻击、使用物品、结束回合）和 AI 的每个决策都必须封装为 Command 对象。禁止直接修改游戏状态。

违反表现：

UI 直接修改 ECS 组件、AI 直接执行攻击逻辑、绕过命令总线修改状态。

---

## 不变量2：校验阶段只读

任意时刻：

Command 的 validate() 方法必须只读，禁止任何状态修改。

违反表现：

validate() 中扣减 MP、设置 acted 标记、修改属性值。

---

## 不变量3：执行阶段不重复校验

任意时刻：

Command 的 execute() 方法必须信任 validate() 的结果，不重复验证。

违反表现：

execute() 中再次检查单位是否存在、是否已行动、MP 是否足够。

---

## 不变量4：Player 和 AI 共用 Command 类型

> 🟩 对应宪法 11.5.2：命令无差别执行

任意时刻：

玩家和 AI 使用完全相同的 Command 类型和执行路径，区别仅在于命令的生产者不同。

违反后果：

AI 和玩家使用不同的伤害计算路径，测试无法覆盖 AI 行为，回放系统不一致。

---

## 不变量5：Command 使用 Strong ID

任意时刻：

Command 对象中引用的单位、物品等实体必须使用 Strong ID，禁止使用裸 Entity。

违反表现：

使用 Entity 而非 UnitId / SkillId / ItemId，导致命令无法序列化。

---

## 不变量6：Command 执行必须在 Exclusive System 中

> **优化来源**: docs/01-architecture/command_bus_design.md — CommandQueue::execute 接收 &mut World，必须在 Exclusive System 中

任意时刻：

Command 的 validate() 和 execute() 必须在 Exclusive System（\&mut World 独占访问）中执行。

违反表现：

在普通 System 或 Parallel System 中执行 Command，导致并发竞争和状态不一致。

---

## 不变量7：Player 和 AI 共用 GameCommand Enum 类型

> **优化来源**: docs/01-architecture/command_bus_design.md — Command Pattern → Replay / AI / 联机直接受益

任意时刻：

Player 和 AI 的命令必须使用相同的 GameCommand Enum 类型，仅通过 CommandSource 字段区分来源。

违反表现：

AI 和 Player 使用不同的命令类型，导致测试无法覆盖 AI 行为，回放系统不一致。

---

# 业务规则

## 规则1：Command trait 接口

必须：
- 新增 Command 类型必须实现完整的 Command trait（validate / execute / description）
- validate() 返回 Ok(()) 或 Err(ValidationError)
- execute() 返回 CommandResult
- description() 返回清晰的命令描述（用于日志和回放）

禁止：
- 跳过 Command trait 直接实现命令
- validate() 中修改游戏状态
- execute() 中重复校验

允许：
- is_undoable() 默认返回 false
- undo() 默认返回 ExecutionError

---

## 规则2：命令来源统一

禁止：
- UI 直接修改 ECS 组件
- AI 独立执行攻击逻辑
- 在 OnEnter/OnExit 中执行命令

必须：
- UI → UiCommand → CommandHandler → Command 对象
- AI → AiCommand → CommandHandler → Command 对象
- 所有命令经过 CommandQueue 执行

允许：
- AI 通过函数调用直接生成 Command 对象

---

## 规则3：批量原子执行

必须：
- 预校验所有命令（只读）
- 任何一条校验失败，整批拒绝
- 全部校验通过后逐个执行
- 每个命令的执行结果记录到历史

禁止：
- 逐个校验逐个执行（非原子性）
- 校验失败后继续执行其他命令
- 批量执行中跳过个别命令

允许：
- 批量执行的结果中包含每个命令的独立结果

---

## 规则4：命令队列管理

必须：
- 已执行命令记录到 executed 历史（包含 description、result、tick）
- 支持 undo_last（撤销最后一个可撤销命令）
- 支持 export_for_replay（导出命令序列）

禁止：
- 丢失已执行命令的历史记录
- 撤销不支持 undo 的命令（返回错误）
- 在命令中包含业务规则逻辑

允许：
- 命令历史仅保留必要信息（不保留原始命令引用）
- 导出格式为字符串描述序列

---

## 规则5：GameCommand Enum 替代 Box\<dyn Command\>

> **优化来源**: docs/01-architecture/command_bus_design.md — 编译时类型安全，支持 Reflect + serde

允许：
- 使用 GameCommand Enum 集中定义所有命令类型
- 通过 RON 序列化/反序列化存储命令
- 编译时检查所有命令变体的完整性

禁止：
- 使用 Box\<dyn Command\> trait object（无法 RON 序列化）
- 为每个命令创建独立的 trait 实现（分散管理）

必须：
- 所有命令类型集中在一个 GameCommand Enum 中
- Enum 变体支持 RON 序列化（存储、网络传输）
- 新增命令类型必须添加到 GameCommand Enum 中

---

## 规则6：Memento 替代 undo()

> **优化来源**: docs/01-architecture/command_bus_design.md — 代码量减少 80%，绝对可靠

允许：
- 使用状态快照方式实现命令撤销
- 在 execute() 前保存快照，在 undo() 时恢复

禁止：
- 手写每个命令的 undo() 反向操作链
- 使用 Ops 日志方式回放反向操作

必须：
- Memento 覆盖所有受 Effect Pipeline 影响的实体状态
- 快照在 execute() 执行前保存
- 撤销时恢复整个快照，不依赖特定命令逻辑

---

## 规则7：校验层性能优化

允许：
- validate() 阶段缓存 ReachableGrid / VisionGrid（避免重复计算）
- 批量校验时复用网格缓存

禁止：
- validate() 每次重新计算完整网格
- 缓存跨帧使用（网格可能过期）

必须：
- 缓存仅在单次批量校验生命周期内有效
- 缓存失效时重新计算（地图状态变更）

---

## 规则8：执行失败回滚

允许：
- 扣 MP 后 Effect 失败时回滚 MP
- 事务式回滚保证状态一致性

禁止：
- Effect 失败后保留前置扣减（MP 已扣但效果未生效）
- 部分执行成功后跳过回滚

必须：
- 前置操作（扣 MP、设冷却）在 Effect Pipeline 失败时全部回滚
- 回滚操作在同一 Exclusive System 中执行（无竞争）

---

# 流程管线

## 命令执行管线（Command Execution Pipeline）

```
UI/AI → CommandHandler → Command 对象 → CommandQueue → 校验 → 执行 → 记录历史
```

### Step1：CommandHandler 生成命令

输入：UiCommand（Message）或 AiCommand（函数调用）
处理：将 UiCommand / AiCommand 转换为领域 Command 对象
输出：GameCommand
禁止：在转换过程中修改游戏状态

### Step2：预校验（批量执行时）

输入：待执行的命令列表
处理：逐个调用 validate()，全程只读
输出：全部通过 或 某条失败
禁止：在校验阶段修改任何状态

### Step3：执行

输入：校验通过的 Command
处理：调用 execute() 修改游戏状态，触发 Effect Pipeline
输出：CommandResult（Success / ExecutionFailed）
禁止：在 execute() 中重复校验

### Step4：记录历史

输入：Command 执行结果
处理：记录到 CommandQueue.executed（description、result、tick）
输出：历史记录更新
禁止：丢失执行历史

---

## Effect Pipeline 集成管线

```
Command 校验通过 → Command 执行 → 生成 CombatIntent → Effect Pipeline → 状态变更 + 事件广播
```

### Step1：Command 执行前置状态变更

输入：Command 参数（caster、skill_id、targets 等）
处理：扣 MP、设冷却、设 acted 标记等前置操作
输出：前置状态变更完成
禁止：跳过前置操作直接触发 Effect Pipeline

### Step2：生成 CombatIntent

输入：Command 参数
处理：构建 CombatIntent Resource（source、skill_id、targets）
输出：CombatIntent 就绪
禁止：在 CombatIntent 中携带执行逻辑

### Step3：Effect Pipeline 执行

输入：CombatIntent
处理：Generate → Modify → Execute 三步管线
输出：状态变更 + 领域事件
禁止：绕过 Effect Pipeline 直接扣血

---

# 数据结构

## Command（命令 trait）

职责：所有游戏操作的统一接口

结构：
- validate()：只读校验，返回 Result<(), ValidationError>
- execute()：修改状态，返回 CommandResult
- description()：命令描述（用于日志和回放）
- is_undoable()：是否支持撤销

要求：
- 必须实现 validate 和 execute
- validate 必须只读
- execute 不重复校验
- 使用 Strong ID 引用实体
- 🟥 undo() 不在 Command trait 上定义，改用 Memento 模式（参见规则6）

---

## CommandResult（命令执行结果）

职责：标识命令执行的最终状态

结构：
- Success：执行成功
- ValidationFailed(ValidationError)：校验失败（命令未执行）
- ExecutionFailed(ExecutionError)：执行过程中出错

要求：
- 校验失败时命令未执行
- 执行失败时可能已部分修改状态
- 每个命令必须返回明确的结果

---

## ValidationError（校验错误类型）

职责：描述校验失败的具体原因

结构：
- UnitNotFound：单位不存在
- UnitAlreadyActed：单位已行动
- UnitDead：单位已死亡
- InsufficientMana：MP 不足
- CooldownNotExpired：技能冷却中
- TargetOutOfRange：目标不在范围内
- InvalidTarget：无效目标
- InventoryFull：背包已满
- RequirementNotMet：装备需求不满足

要求：
- 每个错误类型携带相关参数
- 错误信息足够定位问题

---

## CommandQueue（命令队列）

职责：管理待执行和已执行的命令，支持撤销和回放

结构：
- pending：待执行的命令缓冲（Vec<GameCommand>）
- executed：已执行的命令历史（Vec<QueuedCommand>）

要求：
- pending 在批量预校验时使用
- executed 记录每个命令的 description、result、tick
- 支持 undo_last 和 export_for_replay

---

## CommandContext（命令上下文）

职责：封装命令执行所需的环境信息

结构：
- turn_number：当前回合号
- phase：当前阶段
- source：命令来源（Player / AI）
- random_seed：随机种子（确定性保证）

要求：
- 纯数据传递，不存储持久状态
- 所有命令共享同一上下文
- 随机种子用于确定性执行

---

# 禁止事项

禁止：跳过校验直接执行命令

原因：校验是保证游戏状态合法性的唯一防线，跳过校验可能导致非法状态。

违反后果：数值崩坏、逻辑错误、游戏状态损坏。

---

禁止：校验阶段修改游戏状态

原因：校验必须只读，修改状态会导致非确定性行为和校验结果不一致。

违反后果：校验结果不可复现，回放系统失效，Bug 无法稳定复现。

---

禁止：UI 直接修改 ECS 组件

原因：绕过命令总线会导致操作不可回滚、不可回放，破坏架构一致性。

违反后果：操作无法撤销，回放系统无法记录，调试困难。

---

禁止：AI 独立执行攻击逻辑

原因：AI 必须与玩家使用相同的 Command 类型和 Effect Pipeline，否则伤害计算不一致。

违反后果：AI 伤害不走 Modifier 管线，测试无法覆盖 AI 行为，多人同步失败。

---

禁止：使用裸 Entity 而非 Strong ID

原因：Entity 不可序列化，导致命令无法存储到回放文件或网络同步。

违反后果：回放系统无法使用，多人同步失败，调试困难。

---

禁止：命令中包含业务规则

原因：命令是操作抽象，业务规则属于 Core 层职责，混合会导致规则与命令耦合。

违反后果：规则无法独立测试，命令类型膨胀，维护成本增加。

---

禁止：在 OnEnter/OnExit 中执行命令

原因：系统生命周期回调中的执行时机不确定，可能导致状态不一致。

违反后果：命令在错误的回合阶段执行，状态机转换异常。

---

禁止：执行阶段重复校验

原因：重复校验是冗余检查，浪费性能，且可能导致校验逻辑分散。

违反后果：校验逻辑分散在 validate 和 execute 中，维护困难。

---

禁止：在非 Exclusive System 中执行 Command

> **优化来源**: docs/01-architecture/command_bus_design.md — &mut World 独占访问要求

原因：普通 System 或 Parallel System 无法独占 &mut World，并发执行 Command 会导致竞争条件和状态不一致。

违反后果：多 System 同时修改 ECS 状态，数据竞争、游戏状态损坏。

---

禁止：手写 undo()

> **优化来源**: docs/01-architecture/command_bus_design.md — Effect Pipeline 连锁反应使手写反向操作不可行

原因：Effect Pipeline 连锁反应使手写反向操作不可行（扣 MP → 生成 Buff → 修改属性 → 触发被动），无法可靠逆序回滚。

违反后果：撤销逻辑不完整，状态残留，Bug 不可复现。使用 Memento 替代。

---

# AI 修改规则

## 如果新增命令类型

允许：
- 创建新的 Command trait 实现（validate / execute / description）
- 在 CommandHandler 中添加对应的转换逻辑
- 使用 Strong ID 引用实体

禁止：
- 命令中包含业务规则逻辑
- validate() 中修改游戏状态
- 跳过 Command trait 直接实现

优先检查：
- 命令是否使用 Strong ID（而非裸 Entity 或 String）
- validate 方法是否只读
- execute 方法是否不重复校验
- 命令描述是否清晰（用于日志和回放）

---

## 如果修改现有命令类型

允许：
- 新增可选字段（保持向后兼容）
- 改进校验逻辑
- 优化执行性能

禁止：
- 删除已有字段
- 修改 validate 的只读约束
- 修改 Command trait 接口

优先检查：
- 所有使用该命令的模块是否同步更新
- 校验逻辑是否仍然完整
- 是否影响命令队列的撤销/回放功能

---

## 如果修改批量执行逻辑

允许：
- 调整预校验策略
- 新增批量执行的约束条件

禁止：
- 改为逐个校验逐个执行（破坏原子性）
- 校验失败后继续执行其他命令

优先检查：
- 预校验是否覆盖所有命令
- 校验失败时是否整批拒绝
- 执行顺序是否保持不变

---

## 如果测试失败

排查顺序：
1. 检查 validate() 是否只读（是否修改了游戏状态）
2. 检查 execute() 是否重复校验
3. 检查命令是否使用 Strong ID（而非裸 Entity）
4. 检查 CommandHandler 转换逻辑是否正确
5. 检查批量执行的原子性是否被破坏
6. 检查 CommandContext 的随机种子是否一致
