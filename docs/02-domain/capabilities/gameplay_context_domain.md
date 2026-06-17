---
id: 02-domain.gameplay_context
title: GameplayContext（游戏上下文/载荷）领域规则 v1.0
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-16
tags:
  - domain
  - gameplay-context
  - capabilities
---


## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| GameplayContext | 跨系统传递的统一数据载体，封装一次游戏行为的所有相关数据 | 负责：承载行为来源、目标、能力、环境等全维度数据；不负责：数据的业务含义解释 |
| GameplayContextData | 上下文数据的结构化内容，包含 source/target/ability/weapon/element 等字段 | 负责：结构化存储行为上下文的所有字段；不负责：字段的校验规则 |
| SourceInfo | 行为的发起者信息（实体 Id、阵营、位置） | 负责：标识"谁发起了这个行为"；不负责：发起者的属性状态 |
| TargetInfo | 行为的目标者信息（实体 Id、阵营、位置、是否有效） | 负责：标识"这个行为施加给谁"；不负责：目标选择逻辑 |
| ContextChain | 上下文溯源链，记录行为链的完整路径（如 A 攻击 B → B 反击 → A 受伤） | 负责：行为链的串联追踪，防止无限循环；不负责：链中单个节点的业务处理 |
| ContextBuilder | 上下文的构建器，通过链式调用逐步组装上下文 | 负责：分步骤构建完整上下文；不负责：上下文的校验 |
| ContextOrigin | 上下文的原始触发类型（Direct/Chain Reaction/Triggered/Periodic） | 负责：标记上下文是如何产生的；不负责：上下文的业务处理 |

### ContextChain 溯源链结构

```
ContextChain 是单向链表结构，每个节点记录一次行为的关键信息：

ChainNode {
    origin: ContextOrigin,     // 触发类型
    source: SourceInfo,        // 当前节点的发起者
    target: TargetInfo,        // 当前节点的目标
    ability_id: Option<Id>,    // 当前行为使用的能力
    timestamp: GameTime,       // 当前行为发生的时间点
    prev: Option<Box<ChainNode>> // 上一节点（溯源链接）
}

示例——火球术→燃烧→溅射链：
  Node 3 (Splash):   source=法师, target=目标B, ability=溅射, time=T3
      ↑ prev
  Node 2 (BurnTick): source=法师, target=目标A, ability=燃烧, time=T2
      ↑ prev
  Node 1 (Fireball): source=法师, target=目标A, ability=火球术, time=T1
      ↑ prev
  Root (None)

用途：
1. 防止无限循环：检查链中是否已存在相同 source+target+ability 组合
2. 伤害溯源：最终伤害结算时需要知道最初是谁发起的攻击
3. 反击逻辑：反击是否触发取决于攻击链的原始发起者
```

### 已对齐项目术语

- **Event**：GameplayContext 是 Event 的载荷载体，通过 EventPayload 传递 ContextData
- **Execution**：执行计算接收 GameplayContext 作为计算输入
- **Ability**：技能激活时创建 GameplayContext，向下游 Effect/Execution 传递
- **Cue**：表现信号通过 GameplayContext 携带视觉/音效参数

---

## 2. 上下文状态机

### 上下文的生命周期

```
Building（构建中）
   │  [ContextBuilder 链式调用中]
   ▼
Validated（已校验）
   │  [一次性校验通过]
   ▼
Active（活跃/传递中）
   │  [被 Event/Ability/Execution 依次传递]
   ▼
Consumed（已消费）
   │  [行为链终点]
   ▼
Archived（已归档）
```

### 状态转换规则

| 转换 | 触发条件 | 动作 |
|------|---------|------|
| Building → Validated | ContextBuilder 调用 build() 完成 | 执行一次性校验（所有必填字段已填充） |
| Validated → Active | 上下文开始被系统传递 | 标记为不可变，开始溯源链记录 |
| Active → Consumed | 行为链的最后一个消费者处理完毕 | 标记结束时间戳 |
| Consumed → Archived | 上下文从内存清理 | 写入日志归档（仅 dev-tools） |
| 禁止 | Active 状态下修改 ContextData | 上下文一旦开始传递即不可变 |

---

## 3. 不变量（Invariants）

### 3.1 上下文不可变性（传递中）
- **条件**：GameplayContext 进入 Active 状态后
- **不变量**：上下文的所有字段在传递过程中不可修改
- **违反后果**：下游处理系统读取到的数据与上游不一致

### 3.2 溯源链无环
- **条件**：ContextChain 添加新节点时
- **不变量**：新节点不得与链中已有节点形成 source+target+ability 完全相同的情况（防止同一次行为的无限循环引用）
- **违反后果**：检测到环路时禁止添加新节点，触发循环保护机制

### 3.3 必填字段完整性
- **条件**：ContextBuilder 调用 build() 完成校验时
- **不变量**：source 和 target 必须已填充（不允许"未知来源"或"空目标"的上下文进入传递）
- **违反后果**：build() 返回校验失败，上下文不能进入 Active 状态

### 3.4 链长上限
- **条件**：ContextChain 添加新节点时
- **不变量**：链长度不得超过预设上限（默认 10 跳），超过时视为异常行为链
- **违反后果**：达到上限时禁止继续扩展，当前行为链强制终止

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：多个系统同时修改同一个 GameplayContext — 理由：上下文一旦构建完成即不可变，不允许共享可变引用
- 🟥 禁止：在 Active 状态下回填构建阶段遗漏的字段 — 理由：构建阶段完成后所有字段锁定，遗漏应通过重建上下文解决
- 🟥 禁止：跳过 ContextBuilder 直接创建 GameplayContext — 理由：Builder 是唯一合法的构建入口，确保必填字段校验不被绕过
- 🟥 禁止：GameplayContext 携带系统的内部状态（如 ECS World 引用） — 理由：上下文是纯数据载体，不应引用任何系统对象
- 🟥 禁止：在溯源链中篡改上游节点数据 — 理由：每个节点在加入链时即固定，修改上游节点破坏溯源完整性

---

## 5. 流程定义

### 5.1 上下文构建

- **输入**：SourceInfo、TargetInfo、ContextOrigin、可选 ability/weapon/element 等附加字段
- **处理**：
  1. 创建 ContextBuilder 实例
  2. 链式设置各字段（source → target → ability → weapon → element → 其他自定义字段）
  3. 调用 build() 触发一次性校验（不变量 3.3）
  4. 校验通过后生成不可变 GameplayContext
- **输出**：GameplayContext（Validated 状态）
- **失败处理**：必填字段缺失时 build() 失败，返回错误列表指明缺失字段

### 5.2 溯源链扩展

- **输入**：当前 GameplayContext、新行为节点数据（SourceInfo、TargetInfo、ability_id）
- **处理**：
  1. 从当前上下文的 ContextChain 取出最新节点
  2. 检查新节点是否与链中已有节点形成环路（不变量 3.2）
  3. 检查链长度是否已达上限（不变量 3.4）
  4. 校验通过后创建新节点并链接到链尾
- **输出**：更新后的 ContextChain
- **失败处理**：环路检测或链长上限触发时扩展失败，行为链终止

### 5.3 上下文传递（系统间流转）

- **输入**：GameplayContext（Active）、消费者系统标识
- **处理**：
  1. 将上下文传递给当前消费者
  2. 消费者读取上下文数据（只读）
  3. 消费者根据上下文数据执行自身逻辑
  4. 如果消费者需要产生后续行为，通过溯源链扩展创建新上下文
- **输出**：消费者处理结果，可能产生新 GameplayContext
- **失败处理**：消费者处理失败时上下文标记为 Consumed 但仍保留数据用于错误分析

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| ContextCreated | 上下文构建完成时 | context_id, origin_type, source_id, target_id | 日志、调试工具 |
| ContextConsumed | 上下文生命周期结束时 | context_id, chain_length, total_time | 日志分析器、性能监控 |
| ContextCycleDetected | 溯源链检测到循环时 | context_id, cycle_node, chain_snapshot | 循环保护系统、告警 |
| ContextValidationFailed | 上下文构建校验失败时 | builder_state, missing_fields | 日志、开发调试 |

### 事件订阅关系图

```
ContextCreated
    │
    ├──→ 行为发起系统：确认上下文已就绪
    ├──→ 日志：记录行为链起点
    │
ContextConsumed
    │
    ├──→ 日志分析器：记录行为链完整路径
    └──→ 性能监控：统计行为链平均长度与耗时
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：GameplayContext 能力领域位于 `core/capabilities/gameplay_context/`，foundation/ 定义 context_data.rs，mechanism/ 定义 context_builder.rs 和 context_chain.rs，符合 C1→C2 分层
- ✅ 术语一致：GameplayContextData、ContextChain、ContextBuilder 与架构文档第六节完全一致
- ✅ 职责明确：GameplayContext 只做"数据载体"，不做"计算"（Executor）、"判断"（Condition）、"通知"（Event）
- ✅ 解决痛点：统一载体避免了架构文档所述"每个 Event 重复定义 source/target/ability 字段"的膨胀问题
- ✅ 循环防护：ContextChain 的环路检测 + 链长上限双重保护，防止反击/连锁/伤害转移的无限循环

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖上下文构建、溯源链追踪、系统间传递等全场景
- [x] 所有不变量和约束条件已识别（4 条不变量）
- [x] 禁止事项已明确列出（5 条禁止）
- [x] ContextChain 溯源链结构定义清晰（单向链表 + 环路检测）
- [x] 每个操作有完整的流程定义（构建、溯源链扩展、传递）
