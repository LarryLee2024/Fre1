---
id: 01-architecture.command-bus-design
title: Command Bus Design
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
  - design
---

# 命令总线设计 — 输入→命令→校验→执行抽象

Version: 1.0
Status: Proposed

Source: `docs/其他/31遗漏.md`（高优先级第 4 项）

> **优化来源**: `docs/其他/41.md`

本文档定义 SRPG 项目的命令总线架构，覆盖命令模式、校验层、执行层、命令队列和与 Effect Pipeline 的集成。命令总线是实现"操作可回滚、可回放、可同步"的核心抽象。

交叉引用：
- `docs/02-domain/ecs_communication_rules.md` — Message 通信机制、UiCommand 定义
- `docs/02-domain/shared_layer_rules.md` — Strong ID 在命令中的使用
- `docs/02-domain/replay_rules.md` — 命令队列如何支持回放
- `content-pipeline.md` — 配置驱动的命令参数

---

## 概述

命令总线将玩家和 AI 的所有操作统一抽象为 Command 对象。每个命令经过校验后执行，确保游戏状态的合法性。

**核心价值**：
- 输入与领域逻辑解耦：UI/AI 只生成命令，不直接操作状态
- 操作可回滚：命令队列支持撤销（pop last command）
- 操作可回放：命令序列可序列化、重放
- 校验与执行分离：校验阶段只读、执行阶段只写

> **优化来源**: `docs/其他/74借鉴.md` §13 — Command Pattern 是 Unity 大型 SRPG 的核心抽象

**跨引擎类比**：Unity 大型 SRPG 项目广泛使用 Command Pattern。SRPG 特别适合此模式，因为回合制的"选择→确认→执行"天然对应"校验→执行"流程。三大基本命令类型覆盖了 SRPG 的核心操作：

| 命令类型 | 对应操作 | 模式说明 |
|----------|---------|---------|
| `MoveCommand` | 单位移动 | 移动是 SRPG 最高频操作 |
| `CastSkillCommand` | 释放技能 | 技能是战斗核心 |
| `UseItemCommand` | 使用物品 | 道具是策略维度 |

**Command Pattern → Replay / AI / 联机直接受益**：

```
Command 是纯数据 → 序列化存储 → Replay 回放
Command 统一接口   → AI 只需生成 Command → 与 Player 共用执行路径
Command 序列传输   → 网络同步 → 联机对战
```

---

## 设计原则

### 原则 1：所有操作皆命令

玩家的每个操作（移动、攻击、使用物品、结束回合）和 AI 的每个决策都封装为 Command 对象。禁止直接修改游戏状态。

### 原则 2：先校验后执行

命令在执行前必须经过校验层检查（权限、消耗、目标合法性）。校验阶段只读、不修改任何状态。

### 原则 3：执行不重复校验

命令执行层仅做状态变更，不重复校验。信任校验层的结果。

### 原则 4：Player 与 AI 共用

玩家和 AI 使用相同的 Command 类型和执行路径，区别仅在于命令的生产者不同。

---

## Bevy ECS 落地关键约束

> **优化来源**: `docs/其他/41.md`

### Exclusive System 要求

🟥 **Command Bus 的核心执行 System 必须声明为 Exclusive System（如 `fn run_command_bus(world: &mut World)`），禁止在普通并行 System 中直接调用 execute。**

原因：
- `CommandQueue::execute` 接收 `&mut World`
- 在 Bevy 中，直接传递 `&mut World` 意味着代码必须运行在 Exclusive System 中
- 这会阻塞所有其他并行 System 的执行
- 对于回合制 SRPG，在"玩家行动阶段"或"结算阶段"使用 Exclusive System 是完全合理且推荐的，因为此时本来就需要串行处理核心逻辑以保证确定性

```rust
// ✅ 正确：Exclusive System
fn run_command_bus(world: &mut World) {
    // 可以安全地获取 &mut World
    let mut queue = world.resource_mut::<CommandQueue>();
    // ...
}

// 🟥 错误：并行 System 中调用
fn bad_system(queue: ResMut<CommandQueue>, world: &World) {
    // 编译器会报错：cannot borrow `*world` as mutable because it is also borrowed as immutable
}
```

### Effect Pipeline 集成规则

- Command 只负责生成 CombatIntent，绝不直接修改血量
- 扣血、加 Buff 必须全部交给 Effect Pipeline 处理
- 保持 Command 的纯粹性，保证伤害结算逻辑的全局唯一性

---

## 架构

### GameCommand 枚举（数据驱动命令）

> **优化来源**: `docs/其他/41.md` — 将 `Box<dyn Command>` 重构为可序列化的枚举，解决 Trait Object 无法序列化的问题。

🟥 **禁止使用 `Box<dyn Command>` 存储命令**。改为使用可序列化的 `GameCommand` 枚举：

```rust
// src/core/command/game_command.rs

use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use crate::shared::ids::*;

/// 数据驱动的命令枚举 — 可序列化、可反射、可用于存档/Replay/网络同步
#[derive(Serialize, Deserialize, Clone, Reflect, Debug)]
pub enum GameCommand {
    /// 释放技能
    CastSkill {
        caster: UnitId,
        skill_id: SkillId,
        targets: Vec<UnitId>,
    },
    /// 移动
    Move {
        unit: UnitId,
        target: IVec2,
    },
    /// 使用物品
    UseItem {
        user: UnitId,
        item_id: ItemId,
        targets: Vec<UnitId>,
    },
    /// 结束回合
    EndTurn {
        unit: UnitId,
    },
}

impl GameCommand {
    /// 命令描述（用于日志和调试）
    pub fn description(&self) -> String {
        match self {
            GameCommand::CastSkill { caster, skill_id, targets } => {
                format!("CastSkill({}, caster={}, targets={:?})", skill_id, caster, targets)
            }
            GameCommand::Move { unit, target } => {
                format!("Move({}, target={})", unit, target)
            }
            GameCommand::UseItem { user, item_id, targets } => {
                format!("UseItem({}, user={}, targets={:?})", item_id, user, targets)
            }
            GameCommand::EndTurn { unit } => {
                format!("EndTurn(unit={})", unit)
            }
        }
    }
}
```

**优势**：
- `GameCommand` 是纯数据，可以轻松序列化为 RON 用于存档/Replay/网络同步
- 完美契合"数据驱动"和"MOD 注入"理念（MOD 可以通过反射扩展 Enum 的变体或拦截执行器）
- 编译时类型安全，无需动态分发

### CommandExecutor（命令执行器）

```rust
// src/core/command/command_executor.rs

/// 统一的命令执行器 — 通过 match 分发执行逻辑
pub fn execute_command(
    cmd: &GameCommand,
    world: &mut World,
    ctx: &CommandContext,
) -> CommandResult {
    match cmd {
        GameCommand::CastSkill { caster, skill_id, targets } => {
            execute_cast_skill(caster, skill_id, targets, world, ctx)
        }
        GameCommand::Move { unit, target } => {
            execute_move(unit, target, world, ctx)
        }
        GameCommand::UseItem { user, item_id, targets } => {
            execute_use_item(user, item_id, targets, world, ctx)
        }
        GameCommand::EndTurn { unit } => {
            execute_end_turn(unit, world, ctx)
        }
    }
}
```

### Command Trait（过渡方案 — ⚠️ 待废弃）

> ⚠️ **宪法合规警告（§1.1.7 过度设计）**：以下 Command trait 设计与 `GameCommand` 枚举方案存在重复。根据宪法 §1.1.7 "只解决当前复杂度"原则，应优先使用 `GameCommand` 枚举方案，Command trait 仅作为过渡期向后兼容保留。**新代码禁止使用 `Box<dyn Command>`，统一使用 `GameCommand` 枚举。**

保留 Command trait 用于校验逻辑，但执行层使用 GameCommand 枚举：

```rust
// src/core/command/command_trait.rs

/// 命令校验 trait — 仅用于校验逻辑（过渡方案，最终废弃）
pub trait CommandValidator: Send + Sync + 'static {
    /// 校验命令是否可执行（只读，不修改状态）
    fn validate(&self, world: &World, context: &CommandContext) -> Result<(), ValidationError>;
}
```

### Memento 模式（状态快照撤销）

> **优化来源**: `docs/其他/41.md` — 放弃反向操作 undo，改用状态快照。在复杂 SRPG 中，一个 CastSkillCommand 可能触发 Effect Pipeline 导致连锁反应，手写 undo 逻辑是地狱难度。

🟥 **禁止在 Command trait 中定义 undo 方法**。改用 Memento 模式：

```rust
// src/core/command/memento.rs

/// 状态快照 — 保存受影响 Entity 的相关 Component
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    /// 受影响的 Entity
    pub entity: Entity,
    /// 快照时的 Component 数据
    pub attributes: Option<Attributes>,
    pub grid_position: Option<GridPosition>,
    pub buffs: Option<Buffs>,
    // ... 其他需要快照的 Component
}

/// 命令执行历史 — 存储状态快照用于撤销
#[derive(Resource)]
pub struct CommandHistory {
    /// 状态快照栈（后进先出）
    snapshots: Vec<StateSnapshot>,
    /// 最大历史长度（防止内存溢出）
    max_history: usize,
}

impl CommandHistory {
    /// 执行命令前，自动提取受影响 Entity 的状态快照
    pub fn take_snapshot(&mut self, entity: Entity, world: &World) {
        let snapshot = StateSnapshot {
            entity,
            attributes: world.entity(entity).get::<Attributes>().cloned(),
            grid_position: world.entity(entity).get::<GridPosition>().cloned(),
            buffs: world.entity(entity).get::<Buffs>().cloned(),
        };
        self.snapshots.push(snapshot);

        // 防止内存溢出
        if self.snapshots.len() > self.max_history {
            self.snapshots.remove(0);
        }
    }

    /// 撤销：从历史栈弹出 StateSnapshot，覆盖当前 Entity 的状态
    pub fn undo_last(&mut self, world: &mut World) -> Result<(), ExecutionError> {
        let snapshot = self.snapshots.pop()
            .ok_or(ExecutionError::StateChangeFailed {
                reason: "No history to undo".to_string(),
            })?;

        // 恢复 Component 状态
        if let Some(attrs) = snapshot.attributes {
            world.entity_mut(snapshot.entity).insert(attrs);
        }
        if let Some(pos) = snapshot.grid_position {
            world.entity_mut(snapshot.entity).insert(pos);
        }
        if let Some(buffs) = snapshot.buffs {
            world.entity_mut(snapshot.entity).insert(buffs);
        }

        Ok(())
    }
}
```

**优势**：
- 代码量减少 80%，且绝对可靠
- 无需关心 Effect Pipeline 有多复杂（伤害、Buff、反击、吸血、被动技能等连锁反应）
- 撤销时直接恢复整个 Entity 状态，保证一致性

### Command Trait

所有命令的统一接口：

```rust
// src/core/command/command_trait.rs

use crate::shared::ids::*;

/// 命令执行结果
#[derive(Debug, Clone)]
pub enum CommandResult {
    /// 执行成功
    Success,
    /// 校验失败（命令未执行）
    ValidationFailed(ValidationError),
    /// 执行过程中出错
    ExecutionFailed(ExecutionError),
}

/// 校验错误类型
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// 单位不存在
    UnitNotFound { unit_id: UnitId },
    /// 单位已行动
    UnitAlreadyActed { unit_id: UnitId },
    /// 单位已死亡
    UnitDead { unit_id: UnitId },
    /// MP 不足
    InsufficientMana { required: i32, available: i32 },
    /// 技能冷却中
    CooldownNotExpired { skill_id: SkillId, remaining: u32 },
    /// 目标不在范围内
    TargetOutOfRange { from: IVec2, to: IVec2, max_range: u32 },
    /// 无效目标
    InvalidTarget { target: UnitId, reason: String },
    /// 背包已满
    InventoryFull,
    /// 装备需求不满足
    RequirementNotMet { item_id: ItemId, reason: String },
}

/// 执行错误类型
#[derive(Debug, Clone)]
pub enum ExecutionError {
    /// 效果执行失败
    EffectExecutionFailed { reason: String },
    /// 状态变更失败
    StateChangeFailed { reason: String },
}

/// 命令 trait — 所有游戏操作的统一接口。
pub trait Command: Send + Sync + 'static {
    /// 校验命令是否可执行（只读，不修改状态）
    fn validate(&self, world: &World, context: &CommandContext) -> Result<(), ValidationError>;

    /// 执行命令（修改游戏状态）
    fn execute(&self, world: &mut World, context: &CommandContext) -> CommandResult;

    /// 命令描述（用于日志和调试）
    fn description(&self) -> String;

    /// 命令是否支持撤销
    fn is_undoable(&self) -> bool {
        false
    }

    /// 撤销命令（如果支持）
    fn undo(&self, _world: &mut World, _context: &CommandContext) -> Result<(), ExecutionError> {
        Err(ExecutionError::StateChangeFailed {
            reason: "Command does not support undo".to_string(),
        })
    }
}
```

### CommandContext（命令上下文）

命令执行时的只读上下文信息：

```rust
// src/core/command/command_context.rs

/// 命令执行上下文 — 包含命令执行所需的环境信息。
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// 当前回合号
    pub turn_number: u32,
    /// 当前阶段
    pub phase: String,
    /// 命令来源（Player / AI）
    pub source: CommandSource,
    /// 随机种子（确定性保证）
    pub random_seed: u64,
}

/// 命令来源
#[derive(Debug, Clone, PartialEq)]
pub enum CommandSource {
    Player,
    Ai(AiBehaviorId),
}
```

### 具体命令类型

#### CastSkillCommand（释放技能）

```rust
// src/core/command/commands/cast_skill.rs

/// 释放技能命令。
/// Player 和 AI 共用此命令类型。
#[derive(Debug, Clone)]
pub struct CastSkillCommand {
    /// 释放者
    pub caster: UnitId,
    /// 技能 ID
    pub skill_id: SkillId,
    /// 目标列表
    pub targets: Vec<UnitId>,
}

impl Command for CastSkillCommand {
    fn validate(&self, world: &World, context: &CommandContext) -> Result<(), ValidationError> {
        // 1. 检查 caster 存在且存活
        // 2. 检查 caster 未行动
        // 3. 检查 skill 存在
        // 4. 检查 MP 消耗
        // 5. 检查冷却
        // 6. 检查目标合法性（存在、存活、在范围内）
        // 全程只读，不修改任何状态
        todo!("validate implementation")
    }

    fn execute(&self, world: &mut World, context: &CommandContext) -> CommandResult {
        // 1. 扣除 MP
        // 2. 设置冷却
        // 3. 设置 acted 标记
        // 4. 发送 SkillCasted Message
        // 5. 触发 Effect Pipeline
        // 不重复校验，信任 validate 结果
        todo!("execute implementation")
    }

    fn description(&self) -> String {
        format!(
            "CastSkill({}, caster={}, targets={:?})",
            self.skill_id, self.caster, self.targets
        )
    }

    fn is_undoable(&self) -> bool {
        true  // 技能释放可撤销（在执行前）
    }
}
```

#### MoveCommand（移动）

```rust
#[derive(Debug, Clone)]
pub struct MoveCommand {
    /// 移动者
    pub unit: UnitId,
    /// 目标位置
    pub target: IVec2,
}

impl Command for MoveCommand {
    fn validate(&self, world: &World, context: &CommandContext) -> Result<(), ValidationError> {
        // 1. 检查 unit 存在且存活
        // 2. 检查 unit 未行动
        // 3. 检查目标位置可达（寻路）
        // 4. 检查目标位置未被占用
        // 全程只读
        todo!("validate implementation")
    }

    fn execute(&self, world: &mut World, context: &CommandContext) -> CommandResult {
        // 1. 更新 GridPosition
        // 2. 更新 OccupancyGrid
        // 3. 设置 acted 标记
        // 4. 发送 UnitMoved Message
        // 不重复校验
        todo!("execute implementation")
    }

    fn description(&self) -> String {
        format!("Move({}, target={})", self.unit, self.target)
    }
}
```

#### UseItemCommand（使用物品）

```rust
#[derive(Debug, Clone)]
pub struct UseItemCommand {
    /// 使用者
    pub user: UnitId,
    /// 物品 ID
    pub item_id: ItemId,
    /// 目标列表
    pub targets: Vec<UnitId>,
}

impl Command for UseItemCommand {
    fn validate(&self, world: &World, context: &CommandContext) -> Result<(), ValidationError> {
        // 1. 检查 user 存在且存活
        // 2. 检查物品在背包中
        // 3. 检查目标合法性
        // 全程只读
        todo!("validate implementation")
    }

    fn execute(&self, world: &mut World, context: &CommandContext) -> CommandResult {
        // 1. 扣除物品
        // 2. 应用物品效果
        // 3. 发送 ItemUsed Message
        // 不重复校验
        todo!("execute implementation")
    }

    fn description(&self) -> String {
        format!("UseItem({}, user={}, targets={:?})", self.item_id, self.user, self.targets)
    }
}
```

#### EndTurnCommand（结束回合）

```rust
#[derive(Debug, Clone)]
pub struct EndTurnCommand {
    /// 结束回合的单位
    pub unit: UnitId,
}

impl Command for EndTurnCommand {
    fn validate(&self, world: &World, context: &CommandContext) -> Result<(), ValidationError> {
        // 1. 检查 unit 存在
        // 2. 检查当前阶段允许结束回合
        // 全程只读
        todo!("validate implementation")
    }

    fn execute(&self, world: &mut World, context: &CommandContext) -> CommandResult {
        // 1. 设置 acted 标记
        // 2. 发送 TurnEnded Message
        // 3. 触发回合阶段转换
        // 不重复校验
        todo!("execute implementation")
    }

    fn description(&self) -> String {
        format!("EndTurn(unit={})", self.unit)
    }
}
```

---

## 命令队列

### CommandQueue 结构

```rust
// src/core/command/command_queue.rs

/// 命令队列 — 支持撤销、回放和批量原子执行。
#[derive(Resource)]
pub struct CommandQueue {
    /// 已执行的命令历史
    executed: Vec<QueuedCommand>,
    /// 待执行的命令缓冲
    /// ⚠️ 过渡方案：新代码应使用 Vec<GameCommand> 替代 Vec<Box<dyn Command>>
    pending: Vec<Box<dyn Command>>,
}

/// 带元数据的已执行命令
#[derive(Debug, Clone)]
pub struct QueuedCommand {
    /// 命令描述（用于日志和回放）
    pub description: String,
    /// 执行结果
    pub result: CommandResult,
    /// 执行时的 tick 编号
    pub tick: u32,
}

impl CommandQueue {
    /// 执行单个命令（校验 + 执行）
    pub fn execute(
        &mut self,
        command: Box<dyn Command>,
        world: &mut World,
        context: &CommandContext,
    ) -> CommandResult {
        // 1. 校验
        if let Err(e) = command.validate(world, context) {
            return CommandResult::ValidationFailed(e);
        }

        // 2. 执行
        let result = command.execute(world, context);

        // 3. 记录到历史
        self.executed.push(QueuedCommand {
            description: command.description(),
            result: result.clone(),
            tick: context.turn_number,
        });

        result
    }

    /// 批量原子执行（所有命令都校验通过才执行）
    pub fn execute_batch(
        &mut self,
        commands: Vec<Box<dyn Command>>,
        world: &mut World,
        context: &CommandContext,
    ) -> Vec<CommandResult> {
        // 1. 预校验所有命令
        for cmd in &commands {
            if let Err(e) = cmd.validate(world, context) {
                // 任何一条校验失败，整批不执行
                return commands.iter()
                    .map(|_| CommandResult::ValidationFailed(e.clone()))
                    .collect();
            }
        }

        // 2. 全部校验通过，逐个执行
        commands.into_iter()
            .map(|cmd| self.execute(cmd, world, context))
            .collect()
    }

    /// 撤销最后一个可撤销的命令
    pub fn undo_last(
        &mut self,
        world: &mut World,
        context: &CommandContext,
    ) -> Result<(), ExecutionError> {
        // 找到最后一个 is_undoable 的命令
        let last_undoable = self.executed.iter().rposition(|q| {
            // 需要原始命令来判断是否可撤销
            // 实际实现中需要保存原始命令引用
            todo!("undo logic")
        });
        todo!("undo implementation")
    }

    /// 导出命令序列（用于回放）
    pub fn export_for_replay(&self) -> Vec<String> {
        self.executed.iter()
            .map(|q| q.description.clone())
            .collect()
    }

    /// 获取命令历史
    pub fn history(&self) -> &[QueuedCommand] {
        &self.executed
    }
}
```

### 批量原子执行

批量执行确保"全有或全无"：

```
输入：[Cmd1, Cmd2, Cmd3]
    ↓  预校验（全部通过？）
    ├─ 是 → 逐个执行
    └─ 否 → 整批拒绝，返回校验错误
```

### Cursor-based 执行（游标执行）

> **优化来源**: `docs/其他/41.md` — 游标指针追踪队列中的执行位置，支持断点恢复和分段执行。

```rust
// src/core/command/command_queue.rs

/// 命令队列 — 支持游标执行、撤销、回放和批量原子执行
#[derive(Resource)]
pub struct CommandQueue {
    /// 待执行的命令列表
    commands: Vec<GameCommand>,
    /// 已执行的命令历史（用于撤销）
    history: CommandHistory,
    /// 游标指针 — 追踪当前执行位置
    cursor: usize,
    /// 批次元数据
    batch_id: Option<BatchId>,
}

impl CommandQueue {
    /// 游标执行：从当前 cursor 位置开始执行
    pub fn execute_from_cursor(
        &mut self,
        world: &mut World,
        ctx: &CommandContext,
    ) -> Vec<CommandResult> {
        let mut results = Vec::new();

        while self.cursor < self.commands.len() {
            let cmd = &self.commands[self.cursor];

            // 1. 快照（用于撤销）
            self.take_snapshot_for_command(cmd, world);

            // 2. 校验
            if let Err(e) = validate_command(cmd, world, ctx) {
                results.push(CommandResult::ValidationFailed(e));
                break; // 停在失败位置，cursor 不前进
            }

            // 3. 执行
            let result = execute_command(cmd, world, ctx);
            results.push(result);

            // 4. 推进游标
            self.cursor += 1;
        }

        results
    }

    /// 暂停执行（保存当前 cursor 位置）
    pub fn pause(&self) -> usize {
        self.cursor
    }

    /// 从指定位置恢复执行
    pub fn resume_from(&mut self, position: usize, world: &mut World, ctx: &CommandContext) -> Vec<CommandResult> {
        self.cursor = position;
        self.execute_from_cursor(world, ctx)
    }

    /// 重置游标到队列开头（用于重放）
    pub fn reset_cursor(&mut self) {
        self.cursor = 0;
    }
}
```

**优势**：
- 支持断点恢复：执行到一半可以暂停，稍后从断点继续
- 支持分段执行：可以将长队列分成多个批次执行
- 支持重放：重置游标后可以重新执行整个队列

---

## 校验层 vs 执行层

### 职责分离

```
┌─────────────────────────────────────┐
│           校验层（Validate）          │
│                                     │
│  ✅ 只读，不修改任何状态             │
│  ✅ 检查权限、消耗、目标合法性        │
│  ✅ 返回 Ok(()) 或 Err(ValidationError) │
│  ✅ 不产生副作用                    │
└──────────────────┬──────────────────┘
                   │ 校验通过
                   ↓
┌─────────────────────────────────────┐
│           执行层（Execute）           │
│                                     │
│  ✅ 修改游戏状态                     │
│  ✅ 不重复校验                      │
│  ✅ 发送领域事件                     │
│  ✅ 触发 Effect Pipeline            │
│  ❌ 不检查权限                      │
│  ❌ 不检查消耗                      │
└─────────────────────────────────────┘
```

### 校验层示例

```rust
fn validate_cast_skill(
    command: &CastSkillCommand,
    world: &World,
    context: &CommandContext,
) -> Result<(), ValidationError> {
    // 检查 caster 存在
    let caster_entity = find_unit_by_id(&command.caster, world)
        .ok_or(ValidationError::UnitNotFound { unit_id: command.caster.clone() })?;

    // 检查 caster 存活
    if world.entity(caster_entity).contains::<Dead>() {
        return Err(ValidationError::UnitDead { unit_id: command.caster.clone() });
    }

    // 检查 caster 未行动
    if world.entity(caster_entity).contains::<Acted>() {
        return Err(ValidationError::UnitAlreadyActed { unit_id: command.caster.clone() });
    }

    // 检查 MP 消耗
    let attributes = world.entity(caster_entity).get::<Attributes>()
        .ok_or(ValidationError::UnitNotFound { unit_id: command.caster.clone() })?;
    let skill_data = get_skill_data(&command.skill_id, world)
        .ok_or(ValidationError::InvalidTarget { target: command.caster.clone(), reason: "Skill not found".into() })?;

    if attributes.current_mp < skill_data.mana_cost {
        return Err(ValidationError::InsufficientMana {
            required: skill_data.mana_cost,
            available: attributes.current_mp,
        });
    }

    // ... 其他校验

    Ok(())  // 全部通过
}
```

### 执行层示例

```rust
fn execute_cast_skill(
    command: &CastSkillCommand,
    world: &mut World,
    context: &CommandContext,
) -> CommandResult {
    // 扣除 MP
    if let Some(mut attributes) = world.entity_mut(caster_entity).get_mut::<Attributes>() {
        attributes.current_mp -= skill_data.mana_cost;
    }

    // 设置冷却
    world.entity_mut(caster_entity).insert(SkillCooldown {
        skill_id: command.skill_id.clone(),
        remaining: skill_data.cooldown,
    });

    // 设置 acted 标记
    world.entity_mut(caster_entity).insert(Acted);

    // 发送事件（审计系统会自动监听）
    // SkillCasted 事件通过 Message 系统广播

    // 触发 Effect Pipeline
    // generate → modify → execute

    CommandResult::Success
}
```

---

## 命令来源：Player vs AI

### 统一命令类型

Player 和 AI 使用相同的 Command 类型，区别仅在于命令的生产者：

```
Player Input → UiCommand → CommandHandler → Command 对象
                                                      ↓
AI Decision → AiCommand → CommandHandler → Command 对象
                                                      ↓
                                              Command Queue
                                                      ↓
                                            校验 → 执行
```

### Player 命令生成

```rust
// src/ui/command_handler.rs

/// UI 命令处理器。
/// 将 UiCommand 转换为领域 Command。
pub fn handle_player_command(
    mut ui_command_reader: MessageReader<UiCommand>,
    mut command_queue: ResMut<CommandQueue>,
    world: &World,
    context: CommandContext,
) {
    for ui_cmd in ui_command_reader.read() {
        let command: Box<dyn Command> = match ui_cmd {
            UiCommand::CastSkill { caster, skill_id, targets } => {
                Box::new(CastSkillCommand {
                    caster: caster.clone(),
                    skill_id: skill_id.clone(),
                    targets: targets.clone(),
                })
            }
            UiCommand::Move { unit, target } => {
                Box::new(MoveCommand {
                    unit: unit.clone(),
                    target: *target,
                })
            }
            UiCommand::UseItem { user, item_id, targets } => {
                Box::new(UseItemCommand {
                    user: user.clone(),
                    item_id: item_id.clone(),
                    targets: targets.clone(),
                })
            }
            UiCommand::EndTurn { unit } => {
                Box::new(EndTurnCommand {
                    unit: unit.clone(),
                })
            }
        };

        command_queue.execute(command, world, &context);
    }
}
```

### AI 命令生成

```rust
// src/ai/ai_command_generator.rs

/// AI 命令生成器。
/// 将 AI 决策转换为领域 Command（与 Player 相同的 Command 类型）。
pub fn generate_ai_commands(
    ai_unit: &UnitId,
    ai_behavior: &AiBehavior,
    world: &World,
) -> Vec<Box<dyn Command>> {
    let mut commands = Vec::new();

    // AI 决策：选择目标、技能、移动位置
    let target = ai_behavior.target_selector.select_target(ai_unit, world);
    let skill = ai_behavior.skill_selector.select_skill(ai_unit, &target, world);
    let move_pos = ai_behavior.move_selector.select_position(ai_unit, world);

    // 生成命令（与 Player 完全相同的命令类型）
    if let Some(pos) = move_pos {
        commands.push(Box::new(MoveCommand {
            unit: ai_unit.clone(),
            target: pos,
        }));
    }

    if let Some(skill_id) = skill {
        commands.push(Box::new(CastSkillCommand {
            caster: ai_unit.clone(),
            skill_id,
            targets: vec![target],
        }));
    }

    commands
}
```

---

## 与 Effect Pipeline 集成

命令是 Effect Pipeline 的前置步骤：

```
Command 校验通过
    ↓
Command 执行 → 生成 Effect（CombatIntent）
    ↓
Effect Pipeline: Generate → Modify → Execute
    ↓
状态变更 + 领域事件广播
```

### 集成点

```rust
// CastSkillCommand::execute() 中触发 Effect Pipeline
fn execute(&self, world: &mut World, context: &CommandContext) -> CommandResult {
    // 1. 前置状态变更（扣 MP、设冷却）
    // ...

    // 2. 生成 CombatIntent（Effect Pipeline 的输入）
    let intent = CombatIntent {
        source: self.caster.clone(),
        skill_id: self.skill_id.clone(),
        targets: self.targets.clone(),
    };

    // 3. 触发 Effect Pipeline
    // Pipeline 内部执行: generate → modify → execute
    // 效果执行后发送 DamageApplied / HealApplied / BuffApplied 等 Message

    CommandResult::Success
}
```

---

## ActionQueue（效果执行队列）

> **优化来源**: `docs/其他/74借鉴.md` §18 — ActionQueue 顺序执行队列

很多人遗漏了 ActionQueue 的设计。技能释放后，伤害、Buff、死亡、反击等效果必须按顺序链式执行，而非并行处理。

### ActionQueue vs CommandQueue 的区别

| | CommandQueue | ActionQueue |
|---|-------------|-------------|
| **记录内容** | 用户操作（移动、释放技能、使用物品） | 效果执行（伤害、Buff、死亡判定、反击） |
| **生产者** | Player Input / AI Decision | Effect Pipeline |
| **执行时机** | 玩家/AI 行动阶段 | 效果结算阶段 |
| **数据粒度** | 粗粒度（一个 Command = 一个操作） | 细粒度（一个 Action = 一个效果步骤） |
| **核心价值** | 操作可回滚/可回放 | 效果执行顺序确定性 |

### ActionQueue 链式执行

```
技能释放
    ↓
ActionQueue 入队：
    1. DamageAction → 计算伤害、扣 HP
    2. BuffAction → 施加 Buff/Debuff
    3. DeathCheckAction → 判定死亡、触发 Death Event
    4. CounterAttackAction → 触发反击（如果目标存活且有反击 Buff）
    ↓
逐个执行，每步结果影响下一步
```

### 为什么需要 ActionQueue

没有 ActionQueue 会出现：

```rust
// 🟥 并行处理 — 顺序不确定
spawn_damage_effect(target, 100);
spawn_buff_effect(target, Poison);  // 可能在伤害之前执行
check_death(target);                 // 可能在 Buff 之前执行
trigger_counter(target, attacker);   // 可能在死亡检查之前执行
```

使用 ActionQueue：

```rust
// ✅ 顺序执行 — 确定性保证
action_queue.push(DamageAction { target, amount: 100 });
action_queue.push(BuffAction { target, buff_id: "poison" });
action_queue.push(DeathCheckAction { target });
action_queue.push(CounterAttackAction { target, attacker });

// 逐个执行，每步结果影响下一步
while let Some(action) = action_queue.pop() {
    action.execute(world);
}
```

### Bevy ECS 映射

```rust
#[derive(Resource)]
pub struct ActionQueue {
    actions: VecDeque<PendingAction>,
}

/// 待执行的效果动作
#[derive(Debug, Clone)]
pub enum PendingAction {
    Damage { source: Entity, target: Entity, amount: i32 },
    Heal { source: Entity, target: Entity, amount: i32 },
    ApplyBuff { target: Entity, buff_id: BuffId },
    RemoveBuff { target: Entity, buff_id: BuffId },
    DeathCheck { entity: Entity },
    CounterAttack { target: Entity, attacker: Entity },
    TriggerSkill { source: Entity, skill_id: SkillId, targets: Vec<Entity> },
}
```

> ActionQueue 作为 Effect Pipeline 的执行容器，确保效果的顺序解析。详见 `docs/01-architecture/01-battle-gas/skill-buff-abstraction.md` §ActionQueue。

---

## 通信流程

### 命令通信全链路

```
┌──────────┐     ┌──────────┐     ┌──────────────┐     ┌──────────────┐
│ UI / AI  │────→│Command   │────→│  Command     │────→│  Effect      │
│          │     │Handler   │     │  Queue       │     │  Pipeline    │
│ UiCommand│     │          │     │              │     │              │
│ AiCommand│     │ 生成      │     │ 校验+执行    │     │ generate→    │
│          │     │ Command  │     │              │     │ modify→      │
│          │     │ 对象      │     │              │     │ execute      │
└──────────┘     └──────────┘     └──────────────┘     └──────────────┘
                                       │                       │
                                       │ 领域事件               │ 状态变更
                                       ↓                       ↓
                                 ┌──────────┐          ┌──────────────┐
                                 │ 审计系统  │          │ UI ViewModel │
                                 │ AuditTrail│          │  刷新         │
                                 └──────────┘          └──────────────┘
```

### 通信方式

| 通信路径 | 方式 | 说明 |
|----------|------|------|
| UI → CommandHandler | Message (UiCommand) | 跨层通信 |
| AI → CommandHandler | 函数调用 | AI 内部直接调用 |
| CommandHandler → CommandQueue | 函数调用 | 同模块内 |
| CommandQueue → Effect Pipeline | 函数调用 + Message | 执行命令触发效果 |
| Effect Pipeline → 审计系统 | Message | 领域事件广播 |
| Effect Pipeline → UI | Message (ViewModel) | 状态变更通知 |

---

## 目录结构

```
src/core/command/
├── mod.rs                      # 模块导出
├── command_trait.rs             # Command trait + Result 类型
├── command_context.rs           # CommandContext 结构
├── command_queue.rs             # CommandQueue 资源
├── validation_error.rs          # ValidationError 枚举
├── execution_error.rs           # ExecutionError 枚举
└── commands/
    ├── mod.rs
    ├── cast_skill.rs            # CastSkillCommand
    ├── move_command.rs          # MoveCommand
    ├── use_item.rs              # UseItemCommand
    └── end_turn.rs              # EndTurnCommand
```

---

## 允许的模式

### 模式 1：命令校验后执行

```rust
// ✅ 允许：标准的校验+执行流程
let result = command_queue.execute(
    Box::new(CastSkillCommand { ... }),
    &mut world,
    &context,
);
match result {
    CommandResult::Success => { /* ... */ }
    CommandResult::ValidationFailed(e) => { /* 通知 UI */ }
    CommandResult::ExecutionFailed(e) => { /* 错误处理 */ }
}
```

### 模式 2：批量原子执行

```rust
// ✅ 允许：所有命令都通过才执行
let results = command_queue.execute_batch(
    vec![
        Box::new(MoveCommand { ... }),
        Box::new(CastSkillCommand { ... }),
    ],
    &mut world,
    &context,
);
```

### 模式 3：命令序列导出

```rust
// ✅ 允许：导出命令序列用于回放
let commands = command_queue.export_for_replay();
// commands: ["Move(Unit(warrior_001), target=IVec2(5, 3))", ...]
```

---

## 禁止事项

### 🟥 绝对禁止

| 禁止行为 | 原因 | 违反后果 |
|----------|------|----------|
| 跳过校验直接执行命令 | 游戏状态可能非法 | 数值崩坏、逻辑错误 |
| 校验阶段修改游戏状态 | 校验必须只读 | 非确定性行为 |
| 执行阶段重复校验 | 冗余检查、性能损失 | 校验逻辑分散 |
| UI 直接修改 ECS 组件 | 绕过命令总线 | 操作不可回滚/回放 |
| AI 独立执行攻击逻辑 | 绕过 Effect Pipeline | 伤害计算不一致 |
| 使用裸 Entity 而非 Strong ID | 不可序列化 | 回放系统无法使用 |
| 命令中包含业务规则 | 命令是操作抽象 | 规则与命令耦合 |
| 在 OnEnter/OnExit 中执行命令 | 执行时机不确定 | 状态不一致 |

### 🟩 必须遵守

| 必须行为 | 原因 |
|----------|------|
| 所有玩家/AI 操作封装为 Command | 统一接口、可回滚/回放 |
| 校验阶段只读、不修改状态 | 保证校验的无副作用性 |
| 执行阶段不重复校验 | 信任校验层结果 |
| 命令使用 Strong ID | 类型安全、可序列化 |
| 命令携带完整上下文 | 执行时无需反向查询 |
| 批量执行保证原子性 | 全有或全无 |
| Command Bus 使用 Exclusive System | 保证确定性，避免借用冲突 |

### 常见反模式警示

> **优化来源**: `docs/其他/41.md`

🟥 **反模式 1：在 validate 中调用 asset_server.load()**
- IO 操作破坏确定性且极慢
- 校验阶段必须只读，不产生任何副作用

🟥 **反模式 2：在 execute 中直接 world.spawn() 生成新单位**
- 应通过 Effect Pipeline 的 SpawnUnitEffect 处理
- 保持 Command 的纯粹性

🟥 **反模式 3：试图手写复杂的 undo 逻辑来逆转连锁反应**
- 使用 Memento 模式（状态快照）
- 代码量减少 80%，且绝对可靠

🟥 **反模式 4：使用 Box<dyn Command> 存储命令**
- Trait Object 无法直接序列化
- 使用 GameCommand 枚举，支持 Reflect + serde

---

## Command 序列化与 Replay 协议

> **优化来源**: `docs/其他/41.md`

### GameCommand 序列化格式

```ron
// replay/battle_001.ron
[
    (tick: 10, cmd: Move { unit: "warrior_001", target: (5, 3) }),
    (tick: 12, cmd: CastSkill { caster: "mage_001", skill_id: "base:fireball", targets: ["enemy_003"] }),
    (tick: 15, cmd: EndTurn { unit: "warrior_001" }),
]
```

### Replay 环境还原

重放时必须还原：
1. 随机种子（`rng_seed`）— 保证伪随机操作结果一致
2. 回合阶段（`phase`）— 保证命令在正确的阶段执行
3. 单位状态（`Attributes`、`GridPosition`）— 保证校验结果一致

### Replay 执行器

```rust
pub fn replay_commands(
    replay_data: &[ReplayEntry],
    world: &mut World,
) -> Result<(), ReplayError> {
    // 1. 还原初始状态
    // 2. 按 tick 顺序执行命令
    for entry in replay_data {
        // 还原随机种子
        let ctx = CommandContext {
            turn_number: entry.tick / 100, // 根据 tick 计算回合号
            phase: "PlayerAction".to_string(),
            source: CommandSource::Replay,
            random_seed: entry.rng_seed,
        };

        // 执行命令
        let result = execute_command(&entry.cmd, world, &ctx);
        if let CommandResult::ValidationFailed(e) = result {
            return Err(ReplayError::ValidationFailed {
                tick: entry.tick,
                error: e,
            });
        }
    }
    Ok(())
}
```

---

## AI 修改规则

### 如果新增命令类型

允许：
- 在 `src/core/command/commands/` 中创建新的命令文件
- 实现 `Command` trait 的所有方法
- 在 `command_handler.rs` 中添加对应的转换逻辑

禁止：
- 命令中包含业务规则逻辑（规则在 Core 层）
- 命令的 validate 方法修改游戏状态
- 跳过 Command trait 直接实现

优先检查：
- 命令是否使用 Strong ID（而非裸 Entity 或 String）
- validate 方法是否只读
- execute 方法是否不重复校验
- 命令描述是否清晰（用于日志和回放）

### 如果修改现有命令类型

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

## 宪法合规说明

| 条款 | 合规状态 | 说明 |
|------|---------|------|
| 🟩 §11.5.1 命令层 | ✅ 合规 | 所有操作封装为标准化 Command |
| 🟩 §11.7 读写分离 | ✅ 合规 | Validate 只读，Execute 写入 |
| 🟩 §1.2.1 强类型 ID | ✅ 合规 | 使用 UnitId、SkillId、ItemId |
| 🟩 §2.3.4 Resource 规范 | ✅ 合规 | CommandQueue 为全局唯一状态 |
| 🟥 §1.1.7 避免过度设计 | ⚠️ 需关注 | Box<dyn Command> 与 GameCommand 枚举重复，应统一 |
| 🟥 §20.6.1 禁止 todo!() | ⚠️ 需关注 | 代码示例中的 todo!() 仅为占位，实现时必须替换 |

⚠️ **§1.1.7 过渡期说明**：文档同时存在 `GameCommand` 枚举方案（推荐）和 `Command` trait + `Box<dyn Command>` 方案（过渡）。新代码必须使用 `GameCommand` 枚举，`Box<dyn Command>` 仅在迁移期间保留，应在下一个架构复盘时完全移除。

## 交叉引用

| 文档 | 关系 |
|------|------|
| `docs/01-architecture/01-battle-gas/skill-buff-abstraction.md` | Effect Pipeline 集成、ActionQueue 设计 |
| `docs/01-architecture/schedules_design.md` | Command Bus 在 Schedule 中的位置 |
| `docs/01-architecture/battle_fsm_design.md` | 命令执行与 FSM 状态转换的关系 |
| `docs/01-architecture/component_design_rules.md` | Strong ID 在命令中的使用 |
| `docs/AI开发宪法完整版.md` | §11.5.1 命令层、§11.7 读写分离、§1.1.7 避免过度设计 |
