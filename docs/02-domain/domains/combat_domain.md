---
id: 02-domain.combat
title: Combat（战斗）领域规则 v1.1
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-19
tags:
  - domain
  - combat
  - business-domain
---


## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| CombatState | 战斗整体状态机，定义战斗从开始到结束的全流程阶段 | 负责：战斗宏观阶段的流转，Combat 的 LocalizationKey（name_key/desc_key）；不负责：单个单位的回合管理 |
| TurnOrder | 先攻排序队列，决定单位在战斗中的行动顺序 | 负责：先攻值的计算与排序；不负责：先攻检定的具体公式 |
| CombatParticipant | 战斗参与者标记，标识哪些单位正在参与当前战斗 | 负责：参与者的注册与移除；不负责：参与者的战斗行为 |
| Dead | 阵亡标记 Tag Component，实体获得此 Tag 表示已在战斗中死亡 | 负责：标识死亡状态；不负责：死亡处理逻辑 |
| InitiativeValue | 先攻值，决定单位在回合顺序中的位置 | 负责：先攻值的表示；不负责：先攻值的变化 |
| DamageResult | 伤害结算结果，包含最终伤害数值和命中/暴击/免疫等标志 | 负责：伤害结算的输出封装；不负责：伤害计算的过程 |
| VictoryCondition | 胜负判定条件，定义战斗结束的条件 | 负责：胜负条件的定义与检查；不负责：战斗结束后的处理 |

### 战斗宏观状态机

```
           ┌──────────────────────────────────────────────────────┐
           │                     CombatState                       │
           │                                                       │
           │  Preparation（战前准备）                              │
           │    │  [编队确认 / 先攻检定]                            │
           │    ▼                                                  │
           │  InProgress（战斗中）                                  │
           │    │  ┌──────────────────────────────────┐            │
           │    │  │ Turn（单回合循环）               │            │
           │    │  │  RoundStart → 单位回合循环 → RoundEnd  │     │
           │    │  └──────────────────────────────────┘            │
           │    │  [VictoryCondition 满足]                         │
           │    ▼                                                  │
           │  Resolution（战斗结算）                                │
           │    │  [经验发放 / 战利品结算 / 胜负处理]               │
           │    ▼                                                  │
           │  Ended（战斗结束）                                     │
           └──────────────────────────────────────────────────────┘
```

### 回合微观状态机

```
RoundStart（回合开始）
   │  [所有单位的回合开始触发]
   ▼
UnitTurn - Player（玩家单位行动阶段）
   │  [行动点分配 → 移动 → 行动（攻击/技能/物品）→ 结束]
   ▼
UnitTurn - Enemy（敌方单位行动阶段）
   │  [AI 决策 → 移动 → 行动 → 结束]
   ▼
RoundEnd（回合结束）
   │  [持续效果 Tick → 冷却推进 → 状态更新]
   ▼
RoundStart（下一回合开始）
```

### 已对齐项目术语

- **Tactical**：Combat 消费 Tactical 的夹击/背刺/掩体/高地判定结果
- **Terrain**：Combat 在回合过程中检查地形效果（格子效果/陷阱）
- **Spell**：Combat 管理施法时机（法术在单位行动阶段施放）
- **Reaction**：Reaction 在 Combat 的回合流程中插入执行（回合外触发）
- **Ability**：Combat 调用 Ability 领域执行单位行动时的技能
- **Effect**：Combat 管理持续效果的 Tick 推进
- **Execution**：Combat 调用 Execution 领域结算伤害/治疗
- **Condition**：Combat 检查战斗相关条件的满足与否
- **Event**：Combat 发布战斗事件供其他领域订阅

---

## 2. 战斗核心规则

### 2.1 先攻规则

```
先攻流程：
1. 战斗开始时，所有参与者进行先攻检定
2. 先攻值 = d20 + 敏捷调整值 + 其他加值
3. 按先攻值从高到低排序
4. 先攻值相同时，敏捷属性高者优先
5. 先攻值完全相同时，掷骰决定
6. 先攻排序在战斗过程中不重新计算（除非有特殊能力/效果）
```

### 2.2 回合行动规则

```
每个单位在自己的回合中拥有：
- 1 个标准动作（攻击/施法/使用物品/疾走等）
- 1 个附赠动作（特定技能/物品/天赋允许的快捷动作）
- 1 个反应（可在回合外使用，用于借机攻击/护盾术等）
- 行动力（MovementPoints，用于移动）
- 1 个自由动作（说话/开门/捡起物品等非常简单的操作）

使用规则：
- 标准动作可降级为附赠动作
- 反应在回合外使用，每个回合最多 1 次
- 未使用的反应不累积到下一回合
```

### 2.3 伤害结算流程

```
1. 命中判定：攻击骰 = d20 + 熟练加值 + 属性调整 + 其他加值
   - 自然 20 = 暴击（命中 + 伤害骰翻倍）
   - 自然 1 = 必定未命中
   - >= 目标 AC = 命中
2. 伤害计算：伤害骰 + 属性调整值 + 其他加值
   - 暴击时伤害骰翻倍
   - 应用目标减伤/抗性
   - 应用地形/战术修正（掩体 AC 加成、高地命中优势等）
3. 伤害应用：
   - 目标 HP -= 最终伤害
   - HP <= 0 → 死亡/濒死处理
```

---

## 3. 不变量（Invariants）

### 3.1 CombatIntent 是唯一攻击入口
- **条件**：任何攻击/伤害行为发生时
- **不变量**：所有伤害必须通过 CombatIntent（战斗意图）进入结算流程，禁止绕过 Combat 直接伤害目标
- **违反后果类型**：🔴 程序错误
- **违反后果**：绕过 CombatIntent 的伤害无法被反应/援护/减伤等机制拦截，属系统 Bug

### 3.2 回合严格交替
- **条件**：战斗中
- **不变量**：单位必须按先攻顺序依次行动，禁止跳过或重新排序
- **违反后果类型**：🔴 程序错误
- **违反后果**：回合顺序错乱导致反应/冷却/效果时机失控，属系统 Bug

### 3.3 先攻排序不变性
- **条件**：战斗开始后
- **不变量**：先攻排序在战斗过程中不变（除非有特殊能力声明"重roll先攻"）
- **违反后果类型**：🔴 程序错误
- **违反后果**：先攻变化导致回合顺序不确定，属系统 Bug

### 3.4 一回合只能有一个行动态单位
- **条件**：任何时刻
- **不变量**：同一时刻最多只有一个单位处于"行动中"状态
- **违反后果类型**：🔴 程序错误
- **违反后果**：多个单位同时行动导致战斗状态冲突，属系统 Bug

### 3.5 战斗结束不可逆
- **条件**：CombatState 进入 Ended 后
- **不变量**：已结束的战斗不可重新进入（数据归档，仅可回放查看）
- **违反后果类型**：🔴 程序错误
- **违反后果**：已结束的战斗被重新激活导致状态不一致，属系统 Bug

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：单位在非自己回合时消耗行动资源（反应除外） — 理由：标准动作/附赠动作/行动力只在己方回合消耗
- 🟥 禁止：跳过先攻排序让单位"立即行动" — 理由：所有战斗行为必须按先攻顺序进行
- 🟥 禁止：战斗中对同一伤害进行多次结算（如同时走 Combat 伤害流程和直接扣血） — 理由：同一伤害只能结算一次
- 🟥 禁止：战斗结束后仍保留战斗状态组件 — 理由：战斗结束应清理所有 CombatParticipant/TurnOrder 等战斗组件
- 🟥 禁止：CombatDef 中直接存储用户可见文本的自然语言 — 理由：必须使用 name_key/desc_key: LocalizationKey 引用。违反宪法 §22 Localization First。

---

## 5. 流程定义

### 5.1 战斗开始

- **输入**：参与战斗的单位列表、初始阵营关系、地图
- **处理**：
  1. 注册所有参与者为 CombatParticipant
  2. 所有参与者进行先攻检定（InitiativeValue = d20 + 敏捷调整值）
  3. 按先攻值从高到低排列 TurnOrder
  4. 设置 CombatState = InProgress
  5. 从先攻最高者开始第一回合
  6. 发布 CombatStarted 事件
- **输出**：CombatStarted 事件（参与单位、先攻排序、地图信息）
- **失败处理**：参与单位少于 2 方时战斗无法开始 → 这是**规则失败**（预期业务分支，战斗编队不满足最低要求）

### 5.2 回合流转

- **输入**：当前单位行动结束信号
- **处理**：
  1. 如果当前单位还有未使用的反应，释放反应槽位
  2. 如果当前单位的回合中有未完成的持续效果，推进一次 Tick
  3. 移动到 TurnOrder 中的下一个单位
  4. 如果所有单位都已行动完毕，进入 RoundEnd
  5. RoundEnd 处理：
     a. 所有效果的持续时间推进（Effect 领域）
     b. 冷却推进（Ability 领域）
     c. 地形效果 Tick（Terrain 领域）
     d. 发布 RoundEnd 事件
  6. 进入下一轮 RoundStart：
     a. 重置单位行动力
     b. 重置单位反应槽位
     c. 发布 RoundStart 事件
  7. 检查 VictoryCondition（不变量 3.5）
- **输出**：TurnEnd / RoundEnd / RoundStart / TurnBegin 事件序列
- **失败处理**：单位行动后状态异常（如行动中掉线），跳过该单位继续 → 这是**规则失败**（预期业务分支，异常单位不阻塞战斗流程）

### 5.3 伤害结算

- **输入**：攻击意图（CombatIntent：攻击者、目标、攻击方式、攻击骰、伤害骰、上下文）
- **处理**：
  1. 创建 CombatIntent（唯一攻击入口）
  2. 检查 Reaction 拦截（是否有援护/护盾/法术反制等反应触发）
  3. 如果被拦截，跳转到拦截处理
  4. 命中判定：d20 + 熟练 + 属性调整 >= 目标 AC？
  5. 如果命中，伤害计算：
     a. 伤害骰掷出结果
     b. 加属性调整值
     c. 如果是暴击（自然 20），伤害骰翻倍
     d. 应用目标减伤/抗性
     e. 应用地形/战术修正
  6. 如果未命中，发布 Miss 事件
  7. 命中时：
     a. 目标 HP 减少
     b. 如果 HP <= 0，进入死亡/濒死处理
     c. 发布 DamageDealt 事件
  8. 检查是否有反击/后续触发
- **输出**：DamageResult（最终伤害、命中/暴击/免疫标志、计算明细）
- **失败处理**：CombatIntent 被反应拦截时，伤害不结算 → 这是**规则失败**（预期业务分支，反应机制是战斗的正常组成部分）

### 5.4 胜负判定

- **输入**：每次单位死亡/战斗状态变化
- **处理**：
  1. 检查胜利条件（VictoryCondition）是否满足：
     - 敌方全灭 → 己方胜利
     - 己方全灭 → 敌方胜利
     - 特殊条件（护送目标死亡/据点被占领等）
  2. 如果胜利条件满足：
     a. 设置 CombatState = Resolution
     b. 发放经验/战利品
     c. 设置 CombatState = Ended
     d. 发布 CombatEnded 事件（含胜负结果、存活单位、统计数据）
- **输出**：CombatEnded 事件
- **失败处理**：胜负条件同时满足（如双方最后单位同归于尽）→ 平局 → 这是**规则失败**（预期业务分支，平局是战斗的合法结局之一）

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| CombatStarted | 战斗开始时 | participants[ ], turn_order, map_id, combat_id | 所有 Domain（初始化战斗状态）、UI（切换到战斗界面）、日志（LogCode: BAT001） |
| TurnBegin | 某单位的回合开始时 | entity_id, turn_number, action_points, movement_points | Ability（重置技能冷却状态）、Terrain（回合开始地形效果）、Trigger（回合开始触发器）、日志（LogCode: BAT005） |
| TurnEnd | 某单位的回合结束时 | entity_id, turn_number, action_summary | Effect（推进回合类效果）、Cooldown（推进冷却）、日志（LogCode: BAT006） |
| RoundStart | 新的一轮开始时 | round_number, turn_order_snapshot | 所有持续效果系统、日志（LogCode: BAT003） |
| RoundEnd | 一轮结束时 | round_number, participants_state | Effect（持续效果的 Tick）、日志（LogCode: BAT004） |
| DamageDealt | 伤害结算完成时 | attacker, target, damage_result, context | UI（显示伤害数字）、Trigger（受伤触发器）、Reaction（反击检测）、日志（LogCode: BAT007） |
| CombatEnded | 战斗结束时 | combat_id, winner, survivors[ ], stats（总回合数/总伤害/击杀数） | Progression（发放经验）、Quest（检查战斗任务）、Party（更新队伍状态）、日志（LogCode: BAT002） |
| UnitDied | 单位死亡时 | entity_id, killer_id, death_reason, position | Tactical（移除战场单位）、Trigger（死亡触发器）、Quest（检查任务进度）、日志（LogCode: BAT008） |

### 事件订阅关系图

```
CombatStarted
    │
    ├──→ Spell：初始化法术位状态
    ├──→ Ability：重置技能冷却
    ├──→ Progression：记录战斗开始（用于战后经验计算）
    ├──→ Quest：标记战斗相关任务进行中
    ├──→ UI：加载战斗界面/HUD
    └──→ Cue：战斗开始的音效/特效

TurnBegin
    │
    ├──→ Ability：重置技能冷却状态
    ├──→ Condition：检查回合开始条件（如"每回合一闪"）
    ├──→ Effect：推进回合类效果倒计时
    ├──→ Terrain：回合开始的地形效果
    ├──→ Trigger：检查回合开始触发器
    └──→ UI：高亮当前行动单位

TurnEnd / RoundEnd
    │
    ├──→ Effect：推进持续性效果（倒计时减少）
    ├──→ Cooldown：推进冷却计时
    ├──→ Terrain：回合结束的地形效果
    └──→ UI：取消高亮

DamageDealt
    │
    ├──→ UI：显示伤害数字/飘字
    ├──→ Trigger：检查"受到伤害"/"造成伤害"触发器（反击/连击）
    ├──→ Reaction：检查反击/援护等反应触发
    ├──→ Quest：更新"造成 N 伤害"任务进度
    └──→ Cue：受击特效/音效

CombatEnded
    │
    ├──→ Progression：结算经验奖励
    ├──→ Quest：标记战斗相关任务为可完成
    ├──→ Inventory：添加战利品
    ├──→ Party：更新队伍状态（战斗中→探索）
    ├──→ CampRest：标记可进入休息
    ├──→ UI：加载战斗结算界面
    └──→ Cue：胜利/失败音效

UnitDied
    │
    ├──→ Tactical：从战场移除尸体/标记死亡位置
    ├──→ Trigger：检查死亡相关触发器（如尸爆）
    ├──→ Quest：更新"击杀 N 个敌人"任务进度
    ├──→ Progression：记录击杀经验
    └──→ UI：更新单位状态
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：Combat 域位于 `core/domains/combat/`，components.rs 定义 CombatState/TurnOrder/CombatParticipant/Initiative，systems/ 实现回合/先攻/伤害/死亡/胜负/反应触发系统，rules/ 定义伤害公式/暴击/掩体/优势规则
- ✅ CombatIntent 是唯一攻击入口，符合架构文档"CombatIntent 是唯一攻击入口"的架构决策
- ✅ 不造新系统：Combat 通过调用 Capabilities（Ability/Effect/Execution/Condition/Event）实现所有战斗机制
- ✅ 回合流程清晰：RoundStart → UnitTurn × N → RoundEnd 的循环结构，符合 SRPG 标准战斗模式
- ✅ 伤害结算经过 Reaction 拦截点，为 Reaction 领域提供触发时机
- ✅ LocalizationKey：本领域涉及的用户可见文本使用 LocalizationKey 而非硬编码文本（宪法 §22）

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖战斗开始、回合流转、伤害结算、胜负判定等全场景
- [x] 所有不变量和约束条件已识别（5 条不变量）
- [x] 禁止事项已明确列出（4 条禁止）
- [x] CombatState 宏观状态机 + 回合微观状态机定义清晰
- [x] 先攻规则/回合行动规则/伤害结算规则明确
- [x] 每个操作有完整的流程定义（战斗开始、回合流转、伤害结算、胜负判定）

---

## 9. 架构演进

### Policy 模式整合
DamagePolicy + TargetPolicy 已引入。伤害结算、目标选择走 Policy 模式。

### 伤害结算可解释性
DamagePolicy 输出的 DamageDecision 包含 breakdown 字段，支持 Explain 集成。

### 战斗事件历史
通过 Event History 系统记录战斗领域事件，与 Replay 互补。

### 编译期能力约束
- 🟩 关键战斗能力接口应通过 trait bound 在编译期约束
- 示例：`trait CanAttack {}` / `trait CanCast {}` / `trait CanMove {}`
- 系统签名：`fn execute<T: CanCast>(...)` 保证编译期类型安全
- 与运行时 Tag 查询互补：编译期约束用于系统骨架，运行时 Tag 用于动态场景
