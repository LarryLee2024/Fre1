# Battle 领域

Version: 1.1

Battle 领域管理从攻击意图到伤害结算的完整战斗流程，采用三步效果管线（Generate → Modify → Execute），通过 Message 实现逻辑与表现分离。

核心原则：
- 🟥 Logic / Presentation 分离（宪法 1.1.4）
- 🟥 ECS 是数据流，不是调用链（宪法 2.3.1）
- 🟥 Message 负责跨 Feature 广播（宪法 5.0）
- 🟥 Rule / Content 分离（宪法 1.1.3）
- 🟩 组合优于继承（宪法 1.1.6）

---

# 术语定义

## CombatIntent

攻击意图，表示谁用什么技能攻击谁。

不是 DamageEvent。CombatIntent 是意图，不代表已经发生攻击。

关键属性：
- source_entity：攻击者
- target_coord：目标坐标
- skill_id：技能 ID

---

## EffectQueue

战斗唯一效果缓冲区，保存待执行效果。

不是直接执行通道。所有效果必须进入队列，禁止直接执行。

关键属性：
- pending：PendingEffect 列表

---

## PendingEffect

待处理效果，尚未经过 Modify 和 Execute。

不是最终伤害。PendingEffect 是中间状态，必须经过 Modify → Execute。

关键属性：
- source / target：攻击者/目标实体
- data：效果数据（Damage / Heal / ApplyBuff / Cleanse）
- source_tags：技能标签
- terrain_id：地形 ID

---

## DamageBreakdown

伤害分解，记录从原始值到最终值的完整修饰过程。

不是最终伤害值。Breakdown 是过程记录，actual_damage 是结果。

关键属性：
- base_amount：Generate 阶段原始值
- modified_amount：Modify 阶段修饰后值
- modifiers：每步修饰记录
- actual_damage：Execute 阶段实际扣血

---

## BattleRecord

战斗记录，结构化记录所有战斗事件。

不是 CombatLog。BattleRecord 是结构化数据，CombatLog 是 UI 展示文本。

关键属性：
- entries：BattleEntry 列表
- turn_number：当前回合

---

## EffectHandler

效果处理器 Trait，定义效果的生成和执行逻辑。

不是 enum+match 分发。EffectHandler 通过注册表分发，新增效果类型只需实现 Trait 并注册。

关键属性：
- generate()：生成效果
- execute()：执行效果

---

# 领域边界

## 本领域负责

- 效果管线三步流程（Generate / Modify / Execute）
- CombatIntent 的设置和消费
- 伤害/治疗/Buff/Cleanse 的执行
- 死亡判定（HP ≤ 0 → Dead Tag）
- Trait 触发器（OnAttack / OnHit / OnKill）
- DamageBreakdown 伤害分解
- BattleRecord 战斗记录
- 行动路由（route_after_action）

## 本领域不负责

- 属性计算和修饰符管线（由 stat_system 领域负责）
- ModifierRule 的定义和匹配（由 modifier_rules 领域负责）
- Buff 的生命周期管理（由 buff_rules 领域负责）
- 技能定义和冷却（由 skill_rules 领域负责）
- 回合和阶段管理（由 turn_rules 领域负责）
- UI 展示和战斗日志文本（由 ui_rules 领域负责）
- AI 决策（由 ai_rules 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 伤害发生 | DamageApplied Message | ui / battle_record |
| 治疗发生 | HealApplied Message | ui / battle_record |
| 角色死亡 | CharacterDied Message | ui / battle_record / turn |
| 晕眩施加 | StunApplied Message | ui / battle_record |
| DoT/HoT | DotApplied / HotApplied Message | ui / battle_record |

---

# 生命周期

## 战斗行动生命周期

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Idle | 无战斗行动 | IntentGenerated |
| IntentGenerated | CombatIntent 已设置 | EffectsGenerated |
| EffectsGenerated | Generate 完成 | EffectsModified |
| EffectsModified | Modify 完成 | EffectsExecuted |
| EffectsExecuted | Execute 完成 | MessagesSent |
| MessagesSent | 消息已发送 | Completed |
| Completed | 行动结束 | Idle |

## 状态转换图

Idle → IntentGenerated → EffectsGenerated → EffectsModified → EffectsExecuted → MessagesSent → Completed → Idle

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Idle | IntentGenerated | 玩家/AI 设置 CombatIntent |
| IntentGenerated | EffectsGenerated | generate_combat_effects 完成 |
| EffectsGenerated | EffectsModified | modify_effects 完成 |
| EffectsModified | EffectsExecuted | execute_effects 完成 |
| EffectsExecuted | MessagesSent | 所有 Message 发送完成 |
| MessagesSent | Completed | route_after_action 完成 |

---

# 不变量

## 不变量1：EffectQueue 执行后清空 🟥

Execute 阶段结束后：

EffectQueue.pending 必须为空。

违反表现：

残留效果在下一轮被重复执行。

---

## 不变量2：伤害下限 🟥

任意时刻：

所有伤害值 ≥ 1。

违反表现：

伤害为 0 或负数，角色无法造成伤害。

---

## 不变量3：治疗上限 🟩

Execute 阶段完成后：

治疗后的 HP ≤ MaxHp。

违反表现：

HP 超过 MaxHp，属性系统不一致。

---

## 不变量4：死亡判定一致性 🟥

Execute 阶段完成后：

HP ≤ 0 的单位必须拥有 Dead 组件。

违反表现：

死亡单位仍可行动、被选中、被攻击。

架构违规检测：

发现 HP ≤ 0 的单位缺少 Dead Tag 时，必须停止。必须输出：

```
ARCHITECTURE VIOLATION: HP ≤ 0 的单位缺少 Dead Tag，违反死亡判定一致性不变量。
```

---

## 不变量5：管线严格顺序 🟥

任意时刻：

效果必须按 Generate → Modify → Execute 顺序处理，不可跳步或乱序。

违反表现：

Generate 直接扣血、Modify 发送死亡消息、Execute 创建新 CombatIntent。

架构违规检测：

发现效果处理跳步时，必须停止。必须输出：

```
ARCHITECTURE VIOLATION: 效果管线跳步 [从 XXX 直接到 XXX]，违反 Generate → Modify → Execute 严格顺序。
```

---

## 不变量6：CombatIntent 消费后清除 🟩

Execute 阶段完成后：

CombatIntent 的 source_entity / target_coord / skill_id 必须为 None。

违反表现：

下一次行动误读上一次的意图。

---

## 不变量7：属性修改必须通过 Modifier 🟥

宪法依据：2.2.1（禁止直接修改最终属性值）

任意时刻：

所有属性修改必须通过 Modifier 管线，禁止直接修改 HP 等最终属性值（Execute 阶段扣血除外，因为 Execute 是管线的终点执行）。

违反表现：

在 Generate 或 Modify 阶段直接修改 HP。

---

# 业务规则

## 规则1：效果管线 🟥

宪法依据：1.1.4（Logic/Presentation 分离）、2.3.1（ECS 是数据流）

禁止：
- 🟥 跳过管线直接执行效果
- 🟥 跳过 Modify 阶段
- 🟥 在 Generate 阶段修改 HP
- 🟥 在 Modify 阶段发送 BattleMessage
- 🟥 在 Execute 阶段创建新的 CombatIntent

必须：
- 所有效果进入 EffectQueue
- Generate 推入，Modify 修改，Execute 消费
- Execute 使用 drain(..) 清空队列

允许：
- 🟩 ApplyBuff / Cleanse 不参与 Modify 修饰

---

## 规则2：CombatIntent 🟥

宪法依据：7.0.5（CombatIntent 是唯一攻击意图通道）

禁止：
- 🟥 绕过 CombatIntent 直接发起攻击
- 🟥 AI 和玩家使用不同的攻击路径

必须：
- 玩家通过 UI 设置 CombatIntent
- AI 通过决策系统设置 CombatIntent
- Generate 阶段读取 CombatIntent
- Execute 完成后清除 CombatIntent

架构违规检测：

发现绕过 CombatIntent 直接发起攻击时，必须停止。必须输出：

```
ARCHITECTURE VIOLATION: 绕过 CombatIntent 直接发起攻击，违反"CombatIntent 是唯一攻击意图通道"原则。
```

---

## 规则3：死亡处理 🟥

宪法依据：2.1.4（禁止用 bool 代替 Tag Component）

禁止：
- 🟥 直接删除 Entity
- 🟥 跳过 Dead Tag 直接处理死亡
- 🟥 在 HP 变化时内联死亡处理
- 🟥 用 is_dead: bool 代替 Dead Tag

必须：
- HP ≤ 0 时插入 Dead Tag
- Dead Hook 自动标记 acted + 移除 Selected
- 发送 CharacterDied Message
- route_after_action 通过 HP 判断存活（不依赖 Dead 组件，因为 Dead 是 deferred command）

---

## 规则4：Trait 触发 🟩

宪法依据：6.0.2（Trait 用于扩展点）

禁止：
- OnAttack/OnHit/OnKill 处理 GrantTag / ModifyAttribute

必须：
- 仅处理 ApplyBuff 效果
- OnAttack：Generate 阶段末尾，攻击者触发
- OnHit：Execute 阶段，目标触发
- OnKill：Execute 阶段，攻击者击杀时触发

允许：
- 多个同类型 Trait 全部触发

---

## 规则5：行动路由 🟩

禁止：
- route_after_action 中依赖 Dead 组件判断存活

必须：
- 通过 HP 判断存活
- 队列耗尽时进入 TurnEnd
- 跳过已死亡单位

---

## 规则6：战斗记录 🟩

禁止：
- 修改 BattleRecord 时不记录来源

必须：
- 所有修饰记录写入 ModifierEntry
- 记录伤害来源和目标
- 记录技能 ID 和效果类型

允许：
- entries_for / recent / stats_for 查询

---

## 规则7：EffectHandler trait 分发 🟩

宪法依据：6.0.2（Trait 用于扩展点）

禁止：
- 🟥 match 分发效果类型
- 🟥 新增效果类型时修改管线流程

必须：
- 通过 EffectHandlerRegistry 查找 trait 对象
- 新增效果类型只需实现 EffectHandler trait 并注册

---

# 流程管线

## 效果管线

CombatIntent → Generate → Modify → Execute → Messages

### Step1：Generate

输入：CombatIntent + SkillData + 属性
处理：从技能定义生成原始效果，地形防御加成在此传入
输出：EffectQueue.pending 填充
🟥 禁止：修改 HP、发送 Message、跳过前置检查（晕眩/冷却）

### Step2：Modify

输入：EffectQueue + ModifierRuleRegistry
处理：对 Damage/Heal 应用 ModifierRule 修饰，记录 base_amount
输出：修饰后的 EffectQueue
🟥 禁止：修改 HP、发送 Message、创建新效果

### Step3：Execute

输入：修饰后的 EffectQueue
处理：扣血/回血/施加Buff/净化 + 死亡判定 + 发送 Message
输出：属性变化 + Messages
🟥 禁止：创建新 CombatIntent、跳过死亡判定

### Step4：Messages

输入：Execute 产生的事件
处理：发送 DamageApplied / HealApplied / CharacterDied 等 Message
输出：Message 广播
🟥 禁止：在 Message 处理中修改战斗状态

---

# 数据结构

## CombatIntent（Resource）

职责：攻击意图，唯一攻击入口

结构：
- source_entity：攻击者实体
- target_coord：目标坐标
- skill_id：技能 ID

要求：
- 🟥 玩家和 AI 共用（宪法 7.0.5）
- 🟥 Execute 后必须清除
- 🟩 不可当全局变量仓库（宪法 2.1.5）

---

## EffectQueue（Resource）

职责：战斗唯一效果缓冲区

结构：
- pending：PendingEffect 列表

要求：
- 🟥 Generate 推入，Modify 修改，Execute drain 清空
- 🟥 Execute 后 pending 必须为空

---

## PendingEffect（Instance）

职责：待处理效果

结构：
- source / target：攻击者/目标实体
- data：效果数据（Damage / Heal / ApplyBuff / Cleanse）
- source_tags：技能标签
- terrain_id：地形 ID

要求：
- 🟥 必须经过 Modify → Execute
- 🟩 Damage 包含 amount / base_amount / modifiers

---

## DamageBreakdown（值对象）

职责：伤害分解记录

结构：
- base_amount：Generate 原始值
- modified_amount：Modify 修饰后值
- modifiers：ModifierEntry 列表
- actual_damage：Execute 实际扣血

要求：
- 🟩 三步对应 Generate → Modify → Execute
- 🟩 modifiers 记录每步修饰详情

---

## BattleRecord（Resource）

职责：战斗记录，结构化存储

结构：
- entries：BattleEntry 列表
- turn_number：当前回合

要求：
- 🟩 8 个录制系统通过 MessageReader 监听
- 🟩 支持 entries_for / recent / stats_for 查询
- 🟩 不可当全局变量仓库（宪法 2.1.5）

---

## EntityBattleStats（值对象）

职责：实体战斗统计

结构：
- damage_dealt / damage_taken / heal_received / kills

要求：
- 🟩 击杀数从 CharacterDied 前最后一条 DamageApplied 的 attacker 计算

---

# 禁止事项

🟥 禁止：跳过管线直接执行效果

原因：管线保证 Generate → Modify → Execute 严格顺序，跳步破坏修饰和记录（宪法 2.3.1）

违反后果：伤害未经修饰、未经记录、死亡判定被跳过

架构违规检测：

```
ARCHITECTURE VIOLATION: 效果处理跳步 [从 XXX 直接到 XXX]，违反 Generate → Modify → Execute 严格顺序。
```

---

🟥 禁止：Generate 阶段修改 HP

原因：Generate 只生成效果，不执行效果（宪法 2.2.1）

违反后果：伤害未经修饰直接生效，ModifierRule 失效

---

🟥 禁止：Modify 阶段发送 BattleMessage

原因：Modify 只修饰效果数值，不产生副作用（宪法 1.1.4）

违反后果：UI 在修饰未完成时收到通知，显示错误数值

---

🟥 禁止：Execute 阶段创建新的 CombatIntent

原因：Execute 是管线终点，不应发起新行动

违反后果：无限循环攻击

---

🟥 禁止：直接删除死亡 Entity

原因：死亡是状态标记（Dead Tag），不是即时删除（宪法 2.1.4）

违反后果：其他系统无法响应死亡事件，TurnOrder 索引错乱

---

🟥 禁止：route_after_action 依赖 Dead 组件判断存活

原因：Dead 是 deferred command，可能尚未生效

违反后果：已死亡单位仍被选为下一个行动者

---

🟥 禁止：用 is_dead: bool 代替 Dead Tag

原因：宪法 2.1.4 明确禁止用 bool 代替 Tag Component

违反后果：无法使用 Added/Changed/Removed 检测状态变更

---

🟥 禁止：match 分发效果类型

原因：应通过 EffectHandler trait 分发（宪法 6.0.2）

违反后果：新增效果类型需要修改分发代码

---

🟥 禁止：绕过 CombatIntent 直接发起攻击

原因：CombatIntent 是唯一攻击意图通道（宪法 7.0.5）

违反后果：AI 和玩家走不同攻击路径

架构违规检测：

```
ARCHITECTURE VIOLATION: 绕过 CombatIntent 直接发起攻击，违反"CombatIntent 是唯一攻击意图通道"原则。
```

---

# AI 修改规则

## 如果新增效果类型

允许：
- 新增 EffectHandler 实现并注册
- 新增 PendingEffectData 变体

禁止：
- 🟥 修改 generate_combat_effects 流程
- 🟥 修改 modify_effects 流程
- 🟥 修改 execute_effects 流程
- 🟥 match 分发效果类型

优先检查：
- EffectHandlerRegistry 注册
- Modify 阶段是否需要适配新类型
- Execute 阶段是否需要适配新类型

---

## 如果新增 Trait 触发时机

允许：
- 新增 TraitTrigger 变体
- 在对应阶段添加触发逻辑

禁止：
- 🟥 修改 OnAttack/OnHit/OnKill 的处理逻辑
- 🟥 在触发器中处理非 ApplyBuff 效果

优先检查：
- 触发位置（Generate 末尾 / Execute 中）
- 触发目标（攻击者 / 目标）
- EffectQueue 是否已清空

---

## 如果新增战斗消息

允许：
- 新增 Message 类型
- 新增录制系统

禁止：
- 🟥 修改现有 Message 的字段
- 🟥 在 Execute 中直接调用 UI

优先检查：
- add_message::<T>() 注册
- MessageReader 消费
- BattleRecord 录制

---

## 如果测试失败

排查顺序：
1. 检查管线是否跳步（Generate → Modify → Execute）
2. 检查 EffectQueue 是否在 Execute 后清空
3. 检查 CombatIntent 是否在 Execute 后清除
4. 检查死亡判定是否通过 HP 而非 Dead 组件
5. 检查 ModifierRule 是否正确匹配标签

测试要求（宪法 13.0.1-13.0.3）：
- 🟩 单元测试：验证管线各阶段输入输出
- 🟩 集成测试：验证完整战斗流程
- 🟩 Bug 修复必须先编写重现测试（宪法 13.0.2）
- 🟩 Battle Replay 优先于手工验证（宪法 1.1.9）

---

# 宪法条款映射

| 宪法条款 | 本领域对应 |
|----------|-----------|
| 1.1.3 Rule/Content 分离 | EffectHandler trait 是规则，RON 配置是内容 |
| 1.1.4 Logic/Presentation 分离 | 战斗逻辑不包含 UI/动画逻辑 |
| 1.1.9 测试优先 | Battle Replay + 自动化测试优先 |
| 2.1.1 Entity 只是 ID | source_entity / target 仅作 ID |
| 2.1.4 禁止 bool 代替 Tag | Dead Tag 代替 is_dead: bool |
| 2.1.5 Resource 不是全局仓库 | CombatIntent / EffectQueue 有明确职责 |
| 2.1.6 禁止手写状态标记 | 使用 Added/Changed/Removed 检测 |
| 2.2.1 禁止直接修改最终属性 | HP 修改只在 Execute 阶段 |
| 2.3.1 ECS 是数据流 | 管线是数据流，不是调用链 |
| 5.0 通信三原则 | Message 负责跨 Feature 广播 |
| 6.0.2 Trait 用于扩展点 | EffectHandler trait 分发 |
| 7.0.5 战斗事件链 | CombatIntent 是唯一攻击意图通道 |

---

# 架构违规检测

| 违规行为 | 检测方式 | 输出 |
|----------|----------|------|
| 效果管线跳步 | 代码审查 | ARCHITECTURE VIOLATION: 效果管线跳步 [从 XXX 直接到 XXX]，违反 Generate → Modify → Execute 严格顺序。 |
| HP ≤ 0 缺少 Dead Tag | 代码审查 | ARCHITECTURE VIOLATION: HP ≤ 0 的单位缺少 Dead Tag，违反死亡判定一致性不变量。 |
| 绕过 CombatIntent 发起攻击 | 代码审查 | ARCHITECTURE VIOLATION: 绕过 CombatIntent 直接发起攻击，违反"CombatIntent 是唯一攻击意图通道"原则。 |
| Generate/Modify 阶段修改 HP | 代码审查 | ARCHITECTURE VIOLATION: 在 [Generate/Modify] 阶段直接修改 HP，违反"属性修改必须通过 Modifier 管线"原则。 |
