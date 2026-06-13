# ADR-003: ECS 通信机制三原则

## 状态

Accepted

## 背景

当前代码中存在通信机制滥用的情况：
- 部分模块内逻辑使用 Message 通信（违反 §2.2.4）
- 部分跨 Feature 逻辑使用函数调用（违反 §2.2.3）
- 缺乏统一的 Hook/Observer/Message 选择标准

需要建立清晰的通信机制选择标准，防止：
- 事件系统膨胀（每个逻辑都事件化）
- 模块间紧耦合（直接访问内部状态）
- 调试困难（事件链路过长）

## 引用的领域规则

- `docs/AI开发宪法.md` §2.2.1 — Hook = 组件固有行为
- `docs/AI开发宪法.md` §2.2.2 — Observer = 局部响应
- `docs/AI开发宪法.md` §2.2.3 — Message = 跨 Feature 广播
- `docs/AI开发宪法.md` §2.2.4 — 模块内部优先函数调用
- `docs/AI开发宪法.md` §2.2.5 — 领域事件是唯一业务事实源

## 决策

采用「通信机制三原则」架构：

### 选择标准

| 机制 | 适用场景 | 特征 |
|------|----------|------|
| **Hook** | 组件添加/移除时的固有副作用 | 与组件定义绑定，不可分离 |
| **Observer** | 同一 Feature 内的状态变化响应 | 局部作用域，轻量级 |
| **Message** | 跨 Feature 的业务事件广播 | 全局作用域，松耦合 |
| **函数调用** | 模块内部的直接逻辑调用 | 最高效，无间接层 |

### 决策流程

```
需要通信？
├── 是组件固有行为？ → Hook
├── 是同一 Feature 内？ → Observer
├── 是跨 Feature？ → Message
└── 是模块内部？ → 函数调用
```

核心原则：
1. **Hook 绑定组件**：副作用与组件定义不可分离
2. **Observer 局部响应**：同 Feature 内的状态变化
3. **Message 跨 Feature 广播**：业务事件的唯一传播方式
4. **函数调用优先**：模块内部禁止事件化

## Module Design

不涉及新模块。通信机制是架构原则，通过代码审查和 lint 规则执行。

## Communication Design

### Message（跨 Feature 广播）
适用场景：
- 回合开始/结束 → TurnStarted/TurnEnded
- 战斗胜利/失败 → BattleVictory/BattleDefeat
- Buff 施加/移除 → BuffApplied/BuffRemoved
- 任务完成 → QuestCompleted

### Observer（局部响应）
适用场景：
- 角色死亡后刷新 UI → OnDeadObserver
- 属性变化后更新血条 → OnAttributeChanged
- Buff 过期后清理状态 → OnBuffExpired

### Hook（组件固有行为）
适用场景：
- Dead 组件添加时清除 Selected → on_add=remove_selected
- Stunned 组件添加时阻止行动 → on_add=disable_action

### 函数调用
适用场景：
- 模块内部的伤害计算
- 属性修饰符的添加/移除
- 路径查找算法

## 边界定义

### 允许
- 跨 Feature 通信使用 Message
- 同 Feature 响应使用 Observer
- 组件固有行为使用 Hook
- 模块内部使用函数调用

### 禁止
- 🟥 禁止将同一模块内的所有逻辑都事件化
- 🟥 禁止滥用事件系统模拟函数调用
- 🟥 禁止跨模块直接访问内部组件或状态
- 🟥 禁止在 Observer 中包含业务逻辑

## Forbidden（禁止事项）

- 🟥 禁止：将同一模块内的所有逻辑都事件化 — 理由：事件会增加代码复杂度和调试难度（§2.2.4）
- 🟥 禁止：滥用事件系统模拟函数调用 — 理由：ECS 通过系统处理组件数据，不是通过事件互相调用（§2.3.1）
- 🟥 禁止：跨模块直接访问内部组件或状态 — 理由：模块边界优先于目录结构（§3.0.2）
- 🟥 禁止：在 Observer 中包含业务逻辑 — 理由：Observer 只负责局部响应，不负责业务计算

## Definition / Instance Design

### Definition（不可变配置）
不涉及。通信机制是架构原则。

### Instance（运行时状态）
不涉及。

## 后果

### 正面
1. **清晰边界**：每个通信场景有明确的选择标准
2. **低耦合**：跨 Feature 使用 Message，模块内部使用函数
3. **可调试**：通信链路清晰，易于追踪
4. **高性能**：模块内部使用函数调用，无间接层开销

### 负面
1. **学习成本**：新成员需要理解三原则
2. **边界案例**：部分场景可能需要权衡（如同 Feature 但需要广播）

## 替代方案

### 方案1：全部使用 Message
优点：统一通信方式
缺点：事件膨胀，调试困难，性能开销
**结论：否决** — 违反 §2.2.4，模块内部禁止事件化

### 方案2：全部使用 Observer
优点：利用 Bevy 原生能力
缺点：Observer 用于局部响应，不适合跨 Feature 广播
**结论：否决** — 不适合全局事件场景

### 方案3：全部使用函数调用
优点：最高效
缺点：模块间紧耦合，无法实现松耦合
**结论：否决** — 违反模块边界原则

## 架构合规性检查

- [x] 符合 ECS 约束（Entity=ID, Component=数据, System=行为）
- [x] 符合 Feature First 原则（通信机制按 Feature 边界划分）
- [x] 符合模块边界优先原则（禁止跨模块直接访问）
- [x] 符合 Hook/Observer/Message 三原则
- [x] 所有禁止事项已明确列出
- [x] 已检查 docs/AI开发宪法.md §2.2
