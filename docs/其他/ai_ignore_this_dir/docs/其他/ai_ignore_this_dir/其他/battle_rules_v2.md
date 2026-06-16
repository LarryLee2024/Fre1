# Battle 领域

Version: 2.0

## Purpose

Battle 领域管理从攻击意图到伤害结算的完整战斗流程。采用三步效果管线（Generate → Modify → Execute），通过 Message 实现逻辑与表现分离。CombatIntent 是唯一攻击入口，AI 和玩家共用同一条管线。

---

## Glossary

| 术语 | 定义 | 易混淆项 |
|------|------|----------|
| CombatIntent | 攻击意图，表示谁用什么技能攻击谁 | ≠ DamageEvent：Intent 是意图，不代表已发生攻击 |
| EffectQueue | 战斗唯一效果缓冲区，保存待执行效果 | ≠ 直接执行通道：所有效果必须进入队列 |
| PendingEffect | 待处理效果，尚未经过 Modify 和 Execute | ≠ 最终伤害：是中间状态，必须经过完整管线 |
| DamageBreakdown | 伤害分解，记录从原始值到最终值的完整修饰过程 | ≠ 最终伤害值：Breakdown 是过程记录 |
| BattleRecord | 战斗记录，结构化记录所有战斗事件 | ≠ CombatLog：Record 是结构化数据，Log 是 UI 展示文本 |
| EffectHandler | 效果处理器 Trait，定义效果的生成和执行逻辑 | ≠ enum+match：通过注册表分发 |

---

## Responsibilities

### Owns

- 效果管线三步流程（Generate / Modify / Execute）
- CombatIntent 的设置和消费
- 伤害/治疗/Buff/Cleanse 的执行
- 死亡判定（HP ≤ 0 → Dead Tag）
- Trait 触发器（OnAttack / OnHit / OnKill）
- 伤害分解记录
- 战斗记录
- 行动路由

### Does Not Own

- 属性计算和修饰符管线 → stat_system
- ModifierRule 的定义和匹配 → modifier_rules
- Buff 的生命周期管理 → buff_rules
- 技能定义和冷却 → skill_rules
- 回合和阶段管理 → turn_rules
- UI 展示和战斗日志文本 → ui_rules
- AI 决策 → ai_rules

---

## Invariants

### INV-BTL-01：EffectQueue 执行后清空 🟥

Execute 阶段结束后，EffectQueue 必须为空。

违反：残留效果在下一轮被重复执行。

### INV-BTL-02：伤害下限 🟥

所有伤害值 ≥ 1。

违反：伤害为 0 或负数，角色无法造成伤害。

### INV-BTL-03：治疗上限 🟩

治疗后的 HP ≤ MaxHp。

违反：HP 超过 MaxHp。

### INV-BTL-04：死亡判定一致性 🟥

宪法：2.1.4

HP ≤ 0 的单位必须拥有 Dead Tag。禁止直接删除 Entity，禁止用 bool 代替 Tag。

违反：死亡单位仍可行动、被选中、被攻击。

### INV-BTL-05：管线严格顺序 🟥

宪法：2.3.1

效果必须按 Generate → Modify → Execute 顺序处理，不可跳步或乱序。

违反：Generate 直接扣血、Modify 发送消息、Execute 创建新攻击意图。

### INV-BTL-06：CombatIntent 消费后清除 🟩

Execute 阶段完成后，CombatIntent 必须清除。

违反：下一次行动误读上一次的意图。

### INV-BTL-07：属性修改必须通过 Modifier 🟥

宪法：2.2.1

所有属性修改必须通过 Modifier 管线，禁止直接修改最终属性值（Execute 阶段扣血除外，因为 Execute 是管线终点执行）。

违反：在 Generate 或 Modify 阶段直接修改 HP。

### INV-BTL-08：CombatIntent 是唯一攻击入口 🟥

宪法：7.0.5

AI 和玩家共用 CombatIntent，禁止绕过它直接发起攻击。

违反：AI 和玩家走不同攻击路径。

### INV-BTL-09：EffectHandler Trait 分发 🟩

宪法：6.0.2

效果通过注册表分发，禁止 match 分发效果类型。新增效果类型只需实现 Trait 并注册。

违反：新增效果类型需要修改分发代码。

---

## State Machine

### 战斗行动状态

| 状态 | 含义 | 转换到 |
|------|------|--------|
| Idle | 无战斗行动 | IntentGenerated |
| IntentGenerated | 攻击意图已设置 | EffectsGenerated |
| EffectsGenerated | Generate 完成 | EffectsModified |
| EffectsModified | Modify 完成 | EffectsExecuted |
| EffectsExecuted | Execute 完成 | MessagesSent |
| MessagesSent | 消息已发送 | Completed |
| Completed | 行动结束 | Idle |

```
Idle → IntentGenerated → EffectsGenerated → EffectsModified → EffectsExecuted → MessagesSent → Completed → Idle
```

---

## Business Rules

### BR-BTL-01：效果管线

- 所有效果进入 EffectQueue
- Generate 推入，Modify 修改，Execute 消费并清空
- ApplyBuff / Cleanse 不参与 Modify 修饰

### BR-BTL-02：死亡处理

- HP ≤ 0 时插入 Dead Tag
- Dead Hook 自动标记已行动 + 移除选中
- 发送 CharacterDied Message
- 行动路由通过 HP 判断存活（不依赖 Dead 组件，因为 Dead 是延迟命令）

### BR-BTL-03：Trait 触发

- OnAttack：Generate 阶段末尾，攻击者触发
- OnHit：Execute 阶段，目标触发
- OnKill：Execute 阶段，攻击者击杀时触发
- 仅处理 ApplyBuff 效果，多个同类型 Trait 全部触发

### BR-BTL-04：行动路由

- 通过 HP 判断存活（不依赖 Dead 组件）
- 队列耗尽时进入回合结束
- 跳过已死亡单位

### BR-BTL-05：战斗记录

- 所有修饰记录写入 ModifierEntry
- 记录伤害来源和目标、技能 ID 和效果类型
- 支持 按实体查询 / 最近记录 / 统计查询

---

## Pipelines

### 效果管线

攻击意图 → 生成效果 → 修饰效果 → 执行效果 → 发送消息

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| Generate | 攻击意图 + 技能定义 + 属性 | 待处理效果队列 | 禁止修改 HP（INV-BTL-07）、禁止发送消息、禁止跳过前置检查 |
| Modify | 效果队列 + 修饰规则 | 修饰后效果队列 | 禁止修改 HP（INV-BTL-07）、禁止发送消息、禁止创建新效果 |
| Execute | 修饰后效果队列 | 属性变化 + 消息 | 禁止创建新攻击意图（INV-BTL-05）、禁止跳过死亡判定 |
| Messages | 执行产生的事件 | 消息广播 | 禁止在消息处理中修改战斗状态 |

---

## Data Model

### CombatIntent（Resource）

攻击意图，唯一攻击入口。

- 攻击者实体
- 目标坐标
- 技能 ID
- Execute 后必须清除

### EffectQueue（Resource）

战斗唯一效果缓冲区。

- 待处理效果列表
- Generate 推入，Modify 修改，Execute 清空

### PendingEffect（Instance）

待处理效果。

- 攻击者/目标实体
- 效果数据（Damage / Heal / ApplyBuff / Cleanse）
- 技能标签
- 地形 ID

### DamageBreakdown（值对象）

伤害分解记录，三步对应 Generate → Modify → Execute。

- 原始值 → 修饰后值 → 实际扣血
- 每步修饰记录

### BattleRecord（Resource）

战斗记录，结构化存储。

- 战斗条目列表
- 当前回合
- 支持按实体查询 / 最近记录 / 统计查询

---

## Cross Domain Contracts

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 伤害发生 | DamageApplied Message | ui / battle_record |
| 治疗发生 | HealApplied Message | ui / battle_record |
| 角色死亡 | CharacterDied Message | ui / battle_record / turn |
| 晕眩施加 | StunApplied Message | ui / battle_record |
| DoT/HoT | DotApplied / HotApplied Message | ui / battle_record |

---

## Change Rules

### 新增效果类型

- 允许：新增 EffectHandler 实现 + 注册 + 新增 PendingEffectData 变体
- 禁止：修改管线流程（INV-BTL-05）、match 分发（INV-BTL-09）
- 检查：注册表注册、Modify/Execute 阶段是否需要适配

### 新增 Trait 触发时机

- 允许：新增触发变体 + 在对应阶段添加触发逻辑
- 禁止：修改现有触发器处理逻辑、在触发器中处理非 ApplyBuff 效果
- 检查：触发位置、触发目标、EffectQueue 是否已清空

### 新增战斗消息

- 允许：新增 Message 类型 + 新增录制系统
- 禁止：修改现有 Message 字段、在 Execute 中直接调用 UI
- 检查：消息注册、消费者、BattleRecord 录制

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
| INV-BTL-01 | EffectQueue Execute 后非空 | 效果必须全部消费 | 检查 Execute 是否使用 drain 清空 |
| INV-BTL-04 | HP ≤ 0 缺少 Dead Tag | 死亡判定一致性 | 在 Execute 中添加死亡判定 |
| INV-BTL-05 | 效果管线跳步 | Generate → Modify → Execute 严格顺序 | 检查管线是否完整执行 |
| INV-BTL-07 | Generate/Modify 阶段修改 HP | 属性修改必须通过 Modifier 管线 | 将 HP 修改移到 Execute 阶段 |
| INV-BTL-08 | 绕过 CombatIntent 发起攻击 | CombatIntent 是唯一攻击入口 | 通过 CombatIntent 设置攻击意图 |
| INV-BTL-09 | match 分发效果类型 | 应通过 EffectHandler Trait 分发 | 改为注册表查找 |

---

## Test Requirements

宪法：13.0.1-13.0.3

- 单元测试：验证管线各阶段输入输出
- 集成测试：验证完整战斗流程
- Bug 修复必须先编写重现测试
- Battle Replay 优先于手工验证

排查顺序：
1. 管线是否跳步（Generate → Modify → Execute）
2. EffectQueue 是否在 Execute 后清空
3. CombatIntent 是否在 Execute 后清除
4. 死亡判定是否通过 HP 而非 Dead 组件
5. ModifierRule 是否正确匹配标签
