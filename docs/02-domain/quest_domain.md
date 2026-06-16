# Quest（任务）领域规则 v1.0

Version: 1.0
Status: Draft
Applies To: Domains — 叙事内容层

---

## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Quest | 任务，有目标/进度的可完成游戏内容单元 | 负责：任务的生命周期管理；不负责：任务的触发条件 |
| QuestState | 任务状态，定义任务所处的生命周期阶段 | 负责：状态的流转规则；不负责：状态变化后的行为 |
| Objective | 任务目标，任务的子目标/里程碑，需完成才能推进任务 | 负责：目标的条件定义与进度追踪；不负责：目标完成后的奖励 |
| ObjectiveProgress | 目标进度，记录目标当前的完成状态（如"击杀 3/5 哥布林"） | 负责：进度的存储与更新；不负责：进度的检查逻辑 |
| QuestReward | 任务奖励，任务完成时发放的经验/物品/声望/解锁 | 负责：奖励的定义与发放；不负责：奖励的具体数值计算 |
| QuestPrerequisite | 任务前置条件，接受任务前必须满足的条件 | 负责：前置条件的定义与检查；不负责：不满足条件时的替代处理 |

### 任务状态机

```
Unavailable（不可用——前置条件未满足）
   │  [前置条件满足]
   ▼
Available（可用——可接受）
   │  [接受任务]
   ▼
Active（进行中）
   │  [追踪所有目标进度]
   │      │
   │      ├──→ [所有目标完成] → 任务变为可交付
   │      │
   │      └──→ [任务失败条件满足] → 变为 Failed
   │
   ├──→ [可交付状态 → 交付任务]
   │
   ▼
Completed（已完成——玩家已获得奖励）
   │
   └──→ [后续任务解锁]

Failed（失败——条件不可挽回）
   │  [可选择重新开始/放弃]
   ▼
Available / Removed（回到可用或移除）
```

### 目标类型

```
ObjectiveType
 ├── Kill（击杀）：击杀特定数量/类型的敌人
 │    示例：击杀 5 只哥布林（进度：3/5）
 ├── Collect（收集）：收集特定数量的物品
 │    示例：收集 3 颗龙鳞（进度：1/3）
 ├── Talk（对话）：与特定 NPC 对话
 │    示例：与村长交谈（进度：0/1—布尔型）
 ├── Reach（到达）：到达特定位置
 │    示例：到达山顶（进度：未到达/已到达）
 ├── Escort（护送）：护送目标安全到达地点
 │    示例：护送商队到城镇（进度：进行中/完成/失败）
 ├── Use（使用）：在特定位置/目标上使用物品
 │    示例：对封印石门使用魔法钥匙（进度：未完成/已完成）
 └── Custom（自定义）：由 Domain 注册的自定义目标类型
       示例：在战斗中让两个特定 NPC 对话（自定义条件）
```

### 已对齐项目术语

- **Combat**：Quest 监听 Combat 事件的击杀数据以更新目标进度
- **Narrative**：对话选项可能接受任务或推进任务
- **Inventory**：任务给予/消耗物品，收集类目标监听物品获得事件
- **Progression**：任务奖励包含经验值
- **Faction**：任务奖励包含声望值；部分任务需要特定声望才能接取
- **Condition**：任务前置条件和目标条件使用 Condition 领域

---

## 2. 任务状态转换规则

| 转换 | 触发条件 | 动作 |
|------|---------|------|
| Unavailable → Available | QuestPrerequisite 全部满足（自动检测） | 更新任务列表，通知玩家有新任务可用 |
| Available → Active | 玩家接受任务 | 初始化 ObjectiveProgress，开始追踪目标 |
| Active → 可交付 | 所有 Objective 完成 | 标记任务为"可交付"，通知玩家可前往交付 |
| 可交付 → Completed | 玩家交付任务/自动完成 | 发放 QuestReward，触发后续任务 |
| Active → Failed | 失败条件满足（如时间超限/关键 NPC 死亡） | 标记失败，清理任务追踪器 |
| Failed → Available | 重置条件满足（重新接取） | 重置任务状态到 Available |
| 禁止 | 跳过 Active 直接 Completed | 所有任务必须经过 Active 阶段，不能直接完成 |

---

## 3. 不变量（Invariants）

### 3.1 任务前置链完整性
- **条件**：任何任务被接受前
- **不变量**：任务的所有 QuestPrerequisite（前置任务/等级/阵营声望）必须全部满足
- **违反后果**：玩家接到本不应可接的任务

### 3.2 目标进度不可倒退
- **条件**：任何 ObjectiveProgress 更新时
- **不变量**：目标进度只增不减（击杀数不会减少，收集数不会减少）
- **违反后果**：进度倒退导致任务完成窗口被关闭

### 3.3 奖励不可重复发放
- **条件**：任务奖励发放时
- **不变量**：每个任务完成后只发放一次奖励，禁止重复领取
- **违反后果**：玩家重复完成任务获得多次奖励

### 3.4 任务互斥性
- **条件**：互斥任务同时激活时
- **不变量**：互斥的两个任务（如"加入 A 阵营"和"加入 B 阵营"）不能同时处于 Active 状态
- **违反后果**：玩家同时接受互斥任务导致冲突

### 3.5 关键任务保护
- **条件**：主线/关键任务相关时
- **不变量**：标记为"关键"的任务不可被放弃或失败（必须完成才能推进主线）
- **违反后果**：主线任务被放弃导致剧情无法推进

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：任务直接修改玩家属性/物品/等级 — 理由：奖励通过 QuestReward 事件发放，所属领域自行处理
- 🟥 禁止：已完成的任务被重新激活 — 理由：Completed 是终态，不可逆
- 🟥 禁止：任务接受后不追踪目标进度 — 理由：Active 状态必须对所有目标启动进度追踪
- 🟥 禁止：目标进度不由事件驱动（如轮询检查） — 理由：Quest 通过监听领域事件更新进度，不轮询

---

## 5. 流程定义

### 5.1 任务接受

- **输入**：任务 ID、接受者（玩家/队伍）
- **处理**：
  1. 检查 QuestPrerequisite（前置任务/等级/声望/StoryFlag）——使用 Condition 领域
  2. 检查任务当前状态是否为 Available
  3. 如果不满足前置条件或不是 Available 状态，拒绝接受
  4. 将所有 Objective 初始化为 ObjectiveProgress（默认进度 = 0）
  5. 设置 QuestState = Active
  6. 注册目标的进度监听（订阅相关领域事件）
  7. 发布 QuestAccepted 事件
- **输出**：QuestAccepted 事件
- **失败处理**：前置条件不满足时拒绝接受

### 5.2 目标进度更新

- **输入**：领域事件（CombatEvents.UnitDied / InventoryEvents.ItemAcquired / NarrativeEvents.ChoiceMade 等）
- **处理**：
  1. 遍历所有 Active 状态的任务
  2. 对每个任务，遍历其 Objective 列表
  3. 检查事件是否匹配某个 Objective 的更新条件（如"击杀哥布林"匹配 UnitDied 且单位类型为哥布林）
  4. 如果匹配，ObjectiveProgress += 增长量
  5. 如果 ObjectiveProgress 达到目标值，标记该 Objective 为完成
  6. 如果所有 Objective 都完成，标记 Quest 为"可交付"
  7. 发布 ObjectiveCompleted 事件（可选，单个目标完成时）
- **输出**：ObjectiveProgress 更新通知（内部），ObjectiveCompleted 事件（单个目标完成时）
- **失败处理**：进度更新异常时记录警告，不影响后续更新

### 5.3 任务交付

- **输入**：交付请求（与交付 NPC 对话/自动完成）
- **处理**：
  1. 检查任务是否为"可交付"状态（所有 Objective 完成）
  2. 发放 QuestReward：
     a. 经验奖励 → 发布 ExperienceGained 事件
     b. 物品奖励 → 发布 ItemAcquired 事件（Inventory 领域）
     c. 声望奖励 → 发布 ReputationChanged 事件（Faction 领域）
     d. 解锁奖励 → 解锁新内容（任务/区域/能力）
  3. 设置 QuestState = Completed
  4. 如果有关联的后续任务，将其前置条件标记为满足
  5. 发布 QuestTurnedIn 事件
- **输出**：QuestTurnedIn 事件（奖励明细列表）
- **失败处理**：任务尚未可交付时交付失败

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| QuestAccepted | 任务被接受时 | entity_id, quest_id, objectives[ ]（初始进度） | Quest（开始追踪）、UI（添加到任务日志） |
| ObjectiveCompleted | 单个目标完成时 | entity_id, quest_id, objective_id, objective_type | Quest（检查是否所有目标完成）、UI（显示目标完成通知） |
| QuestTurnedIn | 任务交付完成时 | entity_id, quest_id, rewards[ ]（经验/物品/声望/解锁） | Progression（发放经验）、Inventory（添加物品）、Faction（更新声望）、UI（显示奖励） |
| QuestFailed | 任务失败时 | entity_id, quest_id, fail_reason | Quest（清理追踪）、UI（显示任务失败） |
| QuestProgressUpdated | 任务进度变化时 | entity_id, quest_id, objective_id, old_progress, new_progress, target | UI（更新任务日志进度显示） |

### 事件订阅关系图

```
QuestAccepted
    │
    ├──→ Quest：开始所有目标的进度监听
    ├──→ UI：添加到任务日志
    └──→ 日志：记录任务接受

ObjectiveCompleted
    │
    ├──→ Quest：检查是否所有目标完成 → 标记可交付
    ├──→ UI：显示目标完成通知
    └──→ Cue：目标完成音效

QuestTurnedIn
    │
    ├──→ Progression：发放经验奖励
    ├──→ Inventory：发放物品奖励
    ├──→ Faction：发放声望奖励
    ├──→ Quest：解锁后续任务
    ├──→ UI：显示任务完成/奖励界面
    ├──→ Cue：完成任务特效
    └──→ 日志：记录任务完成

QuestFailed
    │
    ├──→ Quest：清理任务追踪
    ├──→ UI：显示任务失败通知
    └──→ 日志：记录任务失败原因
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：Quest 域位于 `core/domains/quest/`，components.rs 定义 QuestLog/QuestState/ObjectiveProgress，systems/ 实现任务生命周期/目标追踪/奖励系统，rules/ 定义前置条件和奖励规则
- ✅ 目标进度由事件驱动：Quest 不轮询，通过订阅 Combat/Inventory/Narrative 等领域的事件来更新进度
- ✅ 奖励通过事件分发：不直接操作属性/背包/声望，通过对应领域的事件通知
- ✅ 条件检查复用 Condition 领域：前置条件和目标条件统一使用 Condition 评估

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖任务接受、目标追踪、任务交付、失败处理等全场景
- [x] 所有不变量和约束条件已识别（5 条不变量）
- [x] 禁止事项已明确列出（4 条禁止）
- [x] 任务状态机（Unavailable→Available→Active→Completed/Failed）定义清晰
- [x] 多种目标类型定义清晰（Kill/Collect/Talk/Reach/Escort/Use/Custom）
- [x] 每个操作有完整的流程定义（接受、进度更新、交付）
