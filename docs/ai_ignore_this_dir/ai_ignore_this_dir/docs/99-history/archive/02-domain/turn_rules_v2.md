---
id: history.archive.turn_rules_v2
title: turn_rules_v2
status: archived
owner: domain-designer
created: 2026-06-14
updated: 2026-06-14
superseded_by: ../../02-domain/turn/turn-rules.md
---

# Turn 领域

Version: 2.0

## Purpose

Turn 领域管理 SRPG 的游戏流程状态机、行动队列和回合切换。敏捷驱动行动队列，所有单位按 Initiative 降序行动，队列耗尽时回合结束。状态机负责流程，不负责业务细节。

---

## Glossary

| 术语 | 定义 | 易混淆项 |
|------|------|----------|
| TurnPhase | 回合阶段状态机，定义当前游戏处于哪个交互阶段 | ≠ TurnOrder：Phase 是交互阶段，Order 是行动队列 |
| TurnOrder | 行动队列，按 Initiative 降序排列的单位列表 | ≠ TurnState：Order 是队列，State 是当前阵营和回合号 |
| TurnState | 回合状态，记录当前阵营和回合号 | ≠ TurnOrder：State 是全局状态，Order 是队列数据 |
| Initiative | 单位的敏捷值，决定行动顺序 | ≠ Speed：Initiative 是排序依据，Speed 是属性名 |

---

## Responsibilities

### Owns

- AppState 和 TurnPhase 状态机
- TurnOrder 行动队列构建和推进
- TurnState 回合状态管理
- 回合生命周期（开始/结束/重建）
- TurnStarted / TurnEnded / ForceEndTurn Message

### Does Not Own

- 单位行动的具体执行 → battle_rules
- Buff 回合结算 → buff_rules
- AI 决策 → ai_rules
- UI 交互 → ui_rules

---

## Invariants

### INV-TRN-01：行动顺序由 Initiative 决定 🟥

TurnOrder 构建完成后，queue 按 Initiative 降序排列，不区分阵营。

违反：低 Initiative 单位先行动，或同阵营连续行动。

### INV-TRN-02：队列耗尽自动进入 TurnEnd 🟥

advance() 返回 None 时，必须切换到 TurnPhase::TurnEnd。

违反：队列耗尽后停留在 SelectUnit，无法继续游戏。

### INV-TRN-03：回合结束重置所有单位 acted 🟥

TurnEnd 处理完成后，所有存活单位的 acted = false。

违反：新回合单位无法行动。

### INV-TRN-04：回合结束总是回到 SelectUnit 🟥

TurnEnd 处理完成后，必须切换到 TurnPhase::SelectUnit。

违反：跳过 SelectUnit 直接进入其他阶段。

### INV-TRN-05：状态机不处理业务细节 🟥

宪法：2.1.2, 1.1.4

TurnPhase 的 OnEnter / OnExit 只负责流程转换，不包含业务逻辑。

违反：OnEnter(ExecuteAction) 中包含伤害计算、Buff 应用等业务逻辑。

### INV-TRN-06：NeedsResolve 防重复 🟥

只在 TurnEnd 设置 needs_resolve = true，结算后设置 needs_resolve = false。禁止每次进入 SelectUnit 都结算 Buff。

违反：Buff 效果被重复应用。

---

## State Machine

### 回合阶段

| 状态 | 含义 | 转换到 |
|------|------|--------|
| SelectUnit | 选择行动单位 | MoveUnit, TurnEnd |
| MoveUnit | 移动阶段 | ActionMenu |
| ActionMenu | 行动菜单 | SelectTarget, WaitAction |
| SelectTarget | 选择攻击目标 | ExecuteAction |
| ExecuteAction | 执行攻击 | SelectUnit |
| WaitAction | 待机 | SelectUnit |
| TurnEnd | 回合结束 | SelectUnit（新回合） |

```
SelectUnit → MoveUnit → ActionMenu → SelectTarget → ExecuteAction → SelectUnit
                     ↘ WaitAction → SelectUnit
SelectUnit → TurnEnd → SelectUnit（新回合）
```

| 从 | 到 | 条件 |
|----|-----|------|
| SelectUnit | MoveUnit | 选择了一个可行动单位 |
| MoveUnit | ActionMenu | 单位移动完成或跳过 |
| ActionMenu | SelectTarget | 选择攻击 |
| ActionMenu | WaitAction | 选择待机 |
| SelectTarget | ExecuteAction | 选择目标 |
| ExecuteAction | SelectUnit | 攻击执行完成 |
| WaitAction | SelectUnit | 待机完成 |
| SelectUnit | TurnEnd | 队列耗尽 |
| TurnEnd | SelectUnit | 回合重建完成 |

---

## Business Rules

### BR-TRN-01：行动队列构建

- 按 Initiative 降序排列
- 相同 Initiative 保持原始顺序（稳定排序）
- 空队列返回空 Vec
- 不区分阵营排序

### BR-TRN-02：回合结束流程

- 发送 TurnEnded Message
- turn_number += 1
- 重置所有单位 acted = false
- 设置 needs_resolve = true
- 重建行动队列
- 更新 current_faction
- 重置 AI 计时器
- 发送 TurnStarted Message
- 切换到 SelectUnit

### BR-TRN-03：强制结束回合

- 发送 ForceEndTurn Message
- 队列自然耗尽后进入 TurnEnd
- 未行动单位跳过

---

## Pipelines

### 游戏开始管线

OnEnter(InGame) → Camera → Map → Unit → init_turn_order → TurnStarted

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 初始化 | GameSet 链式执行 | 地图和单位已生成 | 禁止跳过任何 GameSet |
| 构建队列 | 所有存活单位 + Initiative | TurnOrder | 禁止单位未生成时构建队列 |
| 发送消息 | turn_number = 1 | TurnStarted { turn: 1 } | 禁止跳过消息发送 |

### 回合结束管线

TurnEnded → acted 重置 → needs_resolve → 队列重建 → TurnStarted → SelectUnit

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 发送 TurnEnded | 当前 turn_number | 全局通知 | 禁止跳过消息发送 |
| 重置和结算 | 所有单位 + NeedsResolve | 重置后的状态 | 禁止跳过 acted 重置（INV-TRN-03） |
| 队列重建 | 所有存活单位 + Initiative | 新 TurnOrder | 禁止使用旧队列 |
| 新回合开始 | 新 turn_number | TurnStarted + SelectUnit | 禁止跳过 SelectUnit（INV-TRN-04） |

---

## Data Model

### TurnOrder（Resource）

行动队列。

- queue：Vec<Entity>（按 Initiative 降序）
- current_index：usize
- turn_number：u32

### TurnState（Resource）

回合全局状态。

- current_faction：Faction
- turn_number：u32

### TurnPhase（SubState）

回合阶段状态机。

- SelectUnit / MoveUnit / ActionMenu / SelectTarget / ExecuteAction / WaitAction / TurnEnd
- 仅在 AppState::InGame 时激活

### AiTimer（Resource）

AI 行动延迟。

- timer：Timer（0.8 秒，Once 模式）
- 每回合重置

---

## Cross Domain Contracts

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 回合开始 | TurnStarted Message | 全局 |
| 回合结束 | TurnEnded Message | 全局 |
| 强制结束 | ForceEndTurn Message | turn |
| 当前行动单位 | TurnOrder.current_unit() | battle / ai |

---

## Change Rules

### 新增回合阶段

- 允许：新增 TurnPhase 变体 + 新增阶段转换逻辑
- 禁止：修改现有阶段的转换条件、在 OnEnter/OnExit 中处理业务细节
- 检查：阶段转换是否完整、是否影响队列推进、UI 是否需要适配

### 修改行动顺序

- 允许：修改 Initiative 计算方式
- 禁止：修改 TurnOrder.build() 的排序逻辑、区分阵营排序
- 检查：稳定排序是否保持、队列耗尽是否正确触发、AI 行动是否受影响

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
| INV-TRN-05 | 状态机 OnEnter/OnExit 包含业务逻辑 | 状态机只负责流程 | 将业务逻辑移到对应领域 |
| INV-TRN-06 | 每次 SelectUnit 都结算 Buff | NeedsResolve 控制结算时机 | 只在 TurnEnd 设置 needs_resolve |

---

## Test Requirements

宪法：13.0.1-13.0.3

- 单元测试：验证行动队列排序和推进
- 集成测试：验证完整回合生命周期
- Bug 修复必须先编写重现测试

排查顺序：
1. TurnOrder 是否正确构建
2. advance() 是否正确推进
3. 队列耗尽是否触发 TurnEnd
4. acted 是否在回合结束时重置
5. NeedsResolve 是否防止重复结算
