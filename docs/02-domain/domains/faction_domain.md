---
id: 02-domain.faction
title: Faction（阵营关系）领域规则 v1.0
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-19
tags:
  - domain
  - faction
  - business-domain
---


## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Faction | 阵营/势力，定义一个具有共同利益或立场的群体 | 负责：阵营的定义与归属管理，Faction 的 LocalizationKey（name_key/desc_key）；不负责：阵营间的动态关系 |
| FactionMembership | 角色所属的阵营列表，一个角色可以属于多个阵营 | 负责：角色与阵营的关联管理；不负责：阵营间的关系判定 |
| Reputation | 角色在某个阵营中的声望值，反映该阵营对该角色的好感度 | 负责：声望数值的维护；不负责：声望阈值的业务影响 |
| ReputationLevel | 声望等级分段，将数值声望映射为业务层面的关系等级 | 负责：数值→等级的转换规则；不负责：等级的具体效果 |
| FactionRelation | 两个阵营之间的基础关系（盟友/中立/敌对/战争） | 负责：阵营间固有关系的定义；不负责：因声望产生的个体关系变化 |
| RelationshipState | 角色与阵营关系的综合状态，由 FactionRelation 和 Reputation 共同决定 | 负责：个体对阵营的实际关系判定；不负责：关系的业务影响 |

### 声望等级体系

```
ReputationLevel
 ├── 仇恨（Hated）：    -100 ~ -51    → 主动攻击，不再交易，对话不可用
 ├── 敌对（Hostile）：  -50  ~ -11    → 可攻击，交易价格 ×2，对话受限
 ├── 中立（Neutral）：  -10  ~ +9     → 不主动攻击，标准交易，标准对话
 ├── 友好（Friendly）： +10  ~ +49    → 不攻击，交易 -10%，额外对话选项
 ├── 尊敬（Honored）：  +50  ~ +89    → 不攻击，交易 -20%，特殊任务解锁
 └── 崇敬（Revered）：  +90  ~ +100   → 不攻击，交易 -30%，专属装备/任务
```

### 关系判定规则

```
RelationshipState 由两个维度共同决定：

维度1：FactionRelation（阵营固有关系）
  ├── 盟友（Ally）：两个阵营默认友好
  ├── 中立（Neutral）：无特殊关系
  ├── 敌对（Hostile）：两个阵营默认敌对
  └── 战争（War）：全面战争，无差别攻击

维度2：Reputation（个体声望修正）
  ├── Reputation >= +50：即使 FactionRelation=Hostile，个体对该角色不主动攻击
  └── Reputation <= -50：即使 FactionRelation=Ally，个体对该角色关闭交易/对话

最终判定：
  RelationshipState = max(FactionRelation 基础, ReputationLevel 修正)
```

### 已对齐项目术语

- **Narrative**：对话分支依赖 Faction 的声望数据（不同声望触发不同对话选项）
- **Economy**：商店价格受声望折扣影响（友好 -10%，崇敬 -30%）
- **Quest**：任务条件/奖励可能依赖声望值
- **Combat**：阵营关系决定战斗中的敌对/友方判定

---

## 2. 关系状态机

### 个体与阵营的关系变化

```
Neutral（中立——初始状态）
   │  [声望变化/阵营事件]
   ▼
RelationshipChanged（关系变化中）
   │  [重新计算 RelationshipState]
   │
   ├──→ 声望上升：
   │      Neutral → Friendly → Honored → Revered
   │
   └──→ 声望下降：
          Neutral → Hostile → Hated
```

### 阵营间关系的状态转换

```
FactionRelation
 ├── Allied（盟友）      ↔ 可经由重大外交事件变为 Neutral 或 Hostile
 ├── Neutral（中立）     ↔ 可经由结盟事件变为 Allied
 │                       ↔ 可经由冲突事件变为 Hostile
 ├── Hostile（敌对）     ↔ 可经由和谈/任务变为 Neutral
 └── War（战争）         ↔ 可经由停战协议变为 Hostile 或 Neutral
```

---

## 3. 不变量（Invariants）

### 3.1 声望值范围
- **条件**：任何声望变更时
- **不变量**：声望值必须在 [-100, +100] 范围内
- **违反后果类型**：🔴 规则失败
- **违反后果**：超出范围的值被 clamp 到边界

### 3.2 声望变化有因
- **条件**：任何声望增减时
- **不变量**：每次声望变化必须有明确原因（击杀敌对成员/完成任务/对话选择/偷窃行为）
- **违反后果类型**：🔴 程序错误
- **违反后果**：无缘由的声望变化导致玩家无法理解关系变化原因

### 3.3 关系对称性
- **条件**：阵营间 FactionRelation 定义时
- **不变量**：FactionRelation 是双向对称的（A 与 B 为盟友 = B 与 A 也为盟友）
- **违反后果类型**：🔴 程序错误
- **违反后果**：关系不对称导致不同视角看到的状态不一致

### 3.4 声望阈值不可跳过
- **条件**：声望等级跨越时
- **不变量**：声望等级必须逐级跨越，不可跳过中间等级（如从中立直接跳到崇敬）
- **违反后果类型**：🔴 规则失败
- **违反后果**：跳过中间等级导致对应的解锁内容（对话/折扣/任务）被错过

### 3.5 最低声望保护
- **条件**：关键/剧情角色（如队友）的声望变动时
- **不变量**：关键角色的声望不能降到导致其永久敌对/离开队伍的阈值以下（除非剧情允许）
- **违反后果类型**：🔴 规则失败
- **违反后果**：关键角色因声望过低而永久离开，导致剧情无法推进

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：Faction 领域控制战斗中的 AI 行为 — 理由：AI 决策归独立的 AI/决策系统，Faction 只提供"是否敌对"的基础判定
- 🟥 禁止：声望值通过非行为方式变化（如时间推移自动变化） — 理由：所有声望变化必须有玩家/事件触发的明确原因
- 🟥 禁止：阵营间关系在运行时被频繁切换 — 理由：FactionRelation 是相对稳定的设定，频繁切换破坏世界观一致性
- 🟥 禁止：Faction 领域直接修改交易价格或对话选项 — 理由：价格归 Economy 领域，对话归 Narrative 领域，Faction 只提供声望数据
- 🟥 禁止：FactionDef 中直接存储用户可见文本的自然语言 — 理由：必须使用 name_key/desc_key: LocalizationKey 引用。违反宪法 §22 Localization First。

---

## 5. 流程定义

### 5.1 声望变更

- **输入**：角色实体、目标阵营、声望变化量、变化原因
- **处理**：
  1. 计算新声望 = clamp(当前声望 + 变化量, -100, +100)
  2. 检查声望等级是否跨越阈值
  3. 如果跨越，检查是否跳过中间等级（不变量 3.4）
  4. 如果角色为关键角色，检查是否触发最低声望保护（不变量 3.5）
  5. 更新声望值
  6. 如果声望等级变化，发布相关通知
- **输出**：声誉变更确认，ReputationChanged 事件
- **失败处理**：声望越界时 clamp，记录越界警告 → 这是**规则失败**（预期业务分支，声望值被 clamp 到合法范围）

### 5.2 关系判定

- **输入**：角色实体、目标阵营/目标角色
- **处理**：
  1. 获取目标阵营的 FactionRelation（如为目标角色则获取其所属阵营）
  2. 获取角色在目标阵营的 Reputation 值
  3. 根据 FactionRelation 基础 + Reputation 修正综合判定 RelationshipState
  4. 返回最终关系
- **输出**：RelationshipState（盟友/中立/敌对/战争）
- **失败处理**：角色或阵营不存在时返回"中立" → 这是**程序错误**（系统异常，数据缺失应记 Bug）

### 5.3 阵营关系变更（外交事件）

- **输入**：阵营 A、阵营 B、新关系（Allied/Neutral/Hostile/War）、触发条件/事件
- **处理**：
  1. 校验关系变更的触发条件是否满足（如需要完成特定任务）
  2. 检查是否所有受影响的角色都应有通知
  3. 更新 FactionRelation
  4. 发布 FactionRelationChanged 事件
- **输出**：关系变更确认，FactionRelationChanged 事件
- **失败处理**：触发条件不满足时变更被拒绝 → 这是**规则失败**（预期业务分支，外交变更需要满足条件）

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| ReputationChanged | 角色在某阵营的声望变化时 | entity_id, faction_id, old_value, new_value, new_level, reason | Narrative（更新对话选项）、Economy（更新价格折扣）、Quest（检查任务条件）、UI（显示声望变化通知）、日志（LogCode: FAC001） |
| FactionRelationChanged | 两个阵营间的关系变化时 | faction_a, faction_b, old_relation, new_relation, cause | Combat（更新战场敌对关系）、Narrative（更新剧情走向）、Faction（通知阵营成员）、日志（LogCode: FAC002） |
| ReputationLevelUp | 声望等级提升时 | entity_id, faction_id, old_level, new_level | Narrative（解锁新对话）、Quest（解锁新任务）、UI（显示达成的消息）、日志（LogCode: FAC003） |
| RelationshipEvaluated | 关系判定完成时（调试用） | entity_id, target_id, base_relation, reputation_modifier, final_state | 调试工具、UI（关系面板）、日志（LogCode: FAC004） |

### 事件订阅关系图

```
ReputationChanged
    │
    ├──→ Narrative：更新对话可用性/选项
    ├──→ Economy：更新商店价格修正
    ├──→ Quest：检查任务触发/完成条件
    ├──→ UI：显示声望变化（+X 声望 / -X 声望）
    └──→ Faction：更新阵营成员对该角色的态度

FactionRelationChanged
    │
    ├──→ Combat：更新战场敌对关系（停战→停止攻击）
    ├──→ Narrative：触发相关剧情事件
    ├──→ Faction：通知所有关联角色
    └──→ UI：显示阵营关系变化公告
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：Faction 域位于 `core/domains/faction/`，components.rs 定义 FactionMembership/Reputation/FactionRelation，systems/ 实现声望/关系/奖励系统，rules/ 定义声望阈值和关系判定
- ✅ 职责明确：Faction 只做"声望/关系管理"，不做"价格计算"（Economy）、"对话过滤"（Narrative）、"战斗 AI"（AI 系统）
- ✅ 轻依赖：Faction 是叙事层中的轻量领域，依赖链短（仅依赖 Event），适合提前设计
- ✅ 声望作为跨系统数据桥梁：Narrative（对话分支）、Economy（价格折扣）、Quest（任务条件）都消费声望数据
- ✅ LocalizationKey：本领域涉及的用户可见文本使用 LocalizationKey 而非硬编码文本（宪法 §22）

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖声望变更、关系判定、阵营关系变更等全场景
- [x] 所有不变量和约束条件已识别（5 条不变量）
- [x] 禁止事项已明确列出（4 条禁止）
- [x] 声望等级体系定义清晰（6 级：Hated→Hostile→Neutral→Friendly→Honored→Revered）
- [x] 每个操作有完整的流程定义（声望变更、关系判定、阵营关系变更）
