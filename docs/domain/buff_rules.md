# Buff 领域

Version: 1.0

Buff 领域管理所有临时状态效果（增益、减益、DoT、HoT、晕眩），遵循 Definition / Instance 分离，通过统一修饰符管线影响属性。

核心原则：
- Definition / Instance 分离
- Buff = 临时 Trait
- 统一修饰符管线
- 标签三层架构

---

# 术语定义

## BuffData

Buff 的静态定义，描述 Buff"是什么"。

不是 BuffInstance。BuffData 不可变，BuffInstance 是运行时实例。

关键属性：
- id / name：标识和展示
- default_duration：默认持续回合
- modifiers：属性修饰符列表
- tags：标签列表
- dot_damage / hot_heal / is_stun / is_cleanse / is_buff：特殊类型标记

---

## BuffInstance

Buff 的运行时实例，挂载在单位上。

不是 BuffData。Instance 有剩余回合和来源，Data 是配置。

关键属性：
- instance_id：唯一实例 ID
- buff_id：对应 BuffData ID
- remaining_turns：剩余回合
- source_entity：来源实体

---

## ActiveBuffs

单位上所有活跃 Buff 实例的容器。

不是 BuffRegistry。ActiveBuffs 是实例集合，BuffRegistry 是定义注册表。

关键属性：
- instances：BuffInstance 列表
- next_id：实例 ID 生成器

---

## DoT

持续伤害效果，每回合结算。

不是即时伤害。DoT 在回合开始时结算，即时伤害在 Effect Pipeline 中结算。

关键属性：
- dot_damage：每回合伤害值

---

## HoT

持续治疗效果，每回合结算。

不是即时治疗。HoT 在回合开始时结算，即时治疗在 Effect Pipeline 中结算。

关键属性：
- hot_heal：每回合治疗值

---

## Cleanse

净化效果，立即驱散所有 Debuff。

不是 Buff 实例。Cleanse 不创建实例，立即执行驱散。

关键属性：
- is_cleanse = true

---

# 领域边界

## 本领域负责

- BuffData 定义和注册表（BuffRegistry）
- BuffInstance 的施加、移除、同源刷新
- ActiveBuffs 容器管理
- DoT / HoT / 晕眩的回合结算
- Buff 修饰符的添加和清理
- 标签重建（rebuild_tags）

## 本领域不负责

- 属性计算和修饰符管线（由 stat_system 领域负责）
- 效果管线中 ApplyBuff 的生成（由 battle_rules 领域负责）
- 死亡判定（由 battle_rules 领域负责，DoT 致死时调用 Dead Tag）
- UI 展示（由 ui_rules 领域负责）
- Trait 效果分发（由 trait_rules 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 晕眩结算 | StunApplied Message | ui / battle_record |
| DoT 伤害 | DotApplied Message | ui / battle_record |
| HoT 治疗 | HotApplied Message | ui / battle_record |
| DoT 致死 | Dead Tag + CharacterDied Message | battle / ui |
| 修饰符变化 | 直接修改 Attributes | stat_system |

---

# 生命周期

## Buff 实例生命周期

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Defined | BuffData 在注册表中 | — |
| Active | BuffInstance 在 ActiveBuffs 中 | Ticking, Expired |
| Ticking | 回合结算中（DoT/HoT/Stun） | Active, Expired |
| Expired | remaining_turns = 0 | Removed |
| Removed | 实例和修饰符已清理 | — |

## 状态转换图

Defined → Active → Ticking → Active（递减后）
                  ↘ Expired → Removed

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Defined | Active | apply_buff() 调用 |
| Active | Ticking | 回合开始 resolve_status_effects |
| Ticking | Active | remaining_turns > 0，递减后 |
| Ticking | Expired | remaining_turns = 0 |
| Active | Expired | tick() 后 remaining_turns = 0 |
| Expired | Removed | 下次 tick() 清理 |

---

# 不变量

## 不变量1：Buff 必须有来源

apply_buff 完成后：

每个 BuffInstance 的 source_entity 必须有值（或明确为 None 表示系统施加）。

违反表现：

无法追踪 Buff 来源，无法实现同源刷新逻辑。

---

## 不变量2：Buff 必须有过期条件

任意时刻：

每个 BuffInstance 的 remaining_turns 必须 > 0（Active 状态）。

违反表现：

永不过期的 Buff 累积，属性无限叠加。

---

## 不变量3：过期 Buff 修饰符必须清理

tick 完成后：

过期 Buff（remaining_turns = 0）的修饰符必须从 Attributes 中移除。

违反表现：

过期 Buff 的属性修饰仍然生效，角色属性计算错误。

---

## 不变量4：标签安全移除

remove_buff 完成后：

只有没有其他活跃 Buff 提供的标签才从 GameplayTags 中移除。

违反表现：

移除一个 Buff 的标签时误删其他 Buff 提供的相同标签。

---

## 不变量5：Cleanse 不创建实例

apply_buff(Cleanse) 完成后：

ActiveBuffs 中不存在 Cleanse 类型的实例。

违反表现：

Cleanse 作为 Buff 实例存在，占用槽位且无法正常过期。

---

# 业务规则

## 规则1：施加 Buff

禁止：
- Buff 无来源
- Buff 永不过期
- Cleanse 创建实例

必须：
- 同源同 buff_id 刷新持续时间（不新增实例）
- 不同来源的同 ID Buff 可共存
- 修饰符通过 ModifierSource::buff_source 添加到 Attributes
- 标签添加到 GameplayTags

允许：
- 净化立即驱散所有 Debuff

---

## 规则2：移除 Buff

禁止：
- 移除时不清理修饰符
- 移除时误删共享标签

必须：
- 从 ActiveBuffs 移除实例
- 从 Attributes 移除修饰符
- 检查共享标签后再从 GameplayTags 移除

允许：
- remove_all_debuffs() 批量移除

---

## 规则3：回合结算

禁止：
- 每回合结算多次
- 跳过 DoT 致死判定
- HoT 超过 MaxHp

必须：
- 通过 NeedsResolve 标记保证每回合一次
- 结算顺序：晕眩 → DoT → HoT → tick
- DoT 致死时插入 Dead Tag + 发送 CharacterDied
- HoT 回血上限为 MaxHp

允许：
- 晕眩消耗后移除

---

## 规则4：tick 递减

禁止：
- 先递减再移除（导致 remaining=0 的 Buff 被递减为 -1）

必须：
- 先移除 remaining=0 的实例
- 再递减所有剩余实例
- 收集即将过期的修饰符 Source
- 清理过期修饰符
- 重建 GameplayTags

---

## 规则5：标签重建

禁止：
- 增量更新标签（遗漏过期 Buff 的标签）

必须：
- 每次 tick 后完全重建 GameplayTags
- 三层叠加：Trait 授予 → 装备授予 → Buff 授予
- 过期 Buff（remaining_turns=0）的标签不参与重建

---

# 流程管线

## 施加管线

查找定义 → Cleanse 检查 → 生成实例 → 添加修饰符 → 添加标签 → 加入 ActiveBuffs

### Step1：查找定义

输入：buff_id + BuffRegistry
处理：查找 BuffData
输出：BuffData
禁止：buff_id 不存在时静默跳过

### Step2：Cleanse 检查

输入：BuffData.is_cleanse
处理：如果是 Cleanse，立即驱散所有 Debuff 并返回
输出：驱散结果
禁止：Cleanse 创建实例

### Step3：生成实例

输入：BuffData + source_entity + duration
处理：生成 instance_id，构建 BuffInstance
输出：BuffInstance
禁止：同源同 buff_id 时新增实例（应刷新）

### Step4：添加修饰符

输入：BuffData.modifiers + ModifierSource::buff_source
处理：添加到 Attributes
输出：属性变化
禁止：绕过 Modifier 管线

### Step5：添加标签

输入：BuffData.tags
处理：添加到 GameplayTags
输出：标签变化
禁止：跳过标签添加

---

## 回合结算管线

晕眩结算 → DoT 结算 → HoT 结算 → tick 递减 → 标签重建

### Step1：晕眩结算

输入：ActiveBuffs.is_stunned()
处理：标记 acted = true，移除 STUN Buff，发送 StunApplied Message
输出：晕眩状态
禁止：跳过晕眩检查

### Step2：DoT 结算

输入：ActiveBuffs.dot_damage()
处理：扣血 max(0, hp - dot)，发送 DotApplied Message，致死判定
输出：HP 变化 + 可能的 Dead Tag
禁止：跳过致死判定

### Step3：HoT 结算

输入：ActiveBuffs.hot_heal()
处理：回血 min(max_hp, hp + hot)，发送 HotApplied Message
输出：HP 变化
禁止：回血超过 MaxHp

### Step4：tick 递减

输入：ActiveBuffs
处理：先移除 remaining=0，再递减所有，收集过期修饰符
输出：过期 Buff 列表
禁止：先递减再移除

### Step5：标签重建

输入：PersistentTags + ActiveBuffs
处理：三层叠加重建 GameplayTags
输出：更新后的 GameplayTags
禁止：增量更新

---

# 数据结构

## BuffData（Definition）

职责：Buff 的静态定义

结构：
- id / name：标识和展示
- default_duration：默认持续回合
- modifiers：属性修饰符列表
- tags：标签列表
- dot_damage / hot_heal：持续效果数值
- is_stun / is_cleanse / is_buff：类型标记

要求：
- 不可变，加载后不修改
- RON 配置路径：assets/buffs/

---

## BuffInstance（Instance）

职责：Buff 的运行时实例

结构：
- instance_id：唯一实例 ID
- buff_id：对应 BuffData ID
- name：显示名称
- remaining_turns：剩余回合
- source_entity：来源实体
- tags：标签副本
- is_buff：增益/减益
- dot_damage / hot_heal：持续效果数值

要求：
- instance_id 全局唯一
- remaining_turns > 0（Active 状态）

---

## ActiveBuffs（Instance Component）

职责：单位上所有活跃 Buff 容器

结构：
- instances：BuffInstance 列表
- next_id：实例 ID 生成器

要求：
- add 时同源刷新
- tick 先移除再递减
- 查询方法：is_stunned / dot_damage / hot_heal

---

# 禁止事项

禁止：Buff 永不过期

原因：Buff 是临时效果，必须有过期条件

违反后果：Buff 无限累积，属性无限叠加，游戏平衡崩溃

---

禁止：Buff 无来源

原因：来源是同源刷新和追踪的依据

违反后果：同源刷新失效，同一 Buff 重复叠加

---

禁止：移除 Buff 时不清理修饰符

原因：修饰符是属性计算的一部分，不清理导致属性错误

违反后果：过期 Buff 的属性修饰仍然生效

---

禁止：移除 Buff 时误删共享标签

原因：多个 Buff 可能提供相同标签

违反后果：其他 Buff 的标签被误删，标签系统不一致

---

禁止：tick 先递减再移除

原因：remaining=0 的 Buff 不应被递减为负数

违反后果：Buff 持续时间变为负数，逻辑异常

---

禁止：跳过 DoT 致死判定

原因：DoT 伤害可导致角色死亡

违反后果：DoT 将 HP 降到 0 以下但角色不死亡

---

# AI 修改规则

## 如果新增 Buff

允许：
- 新增 RON 配置文件
- 新增 BuffData 字段（需配套反序列化）

禁止：
- 修改 apply_buff 流程
- 修改 remove_buff 流程
- 修改 tick 递减逻辑

优先检查：
- BuffRegistry 注册
- 修饰符 Source 区间是否冲突
- 标签是否在 GameplayTag 枚举中

---

## 如果新增持续效果类型

允许：
- 在 BuffData 中新增字段
- 在 resolve_status_effects 中新增结算步骤

禁止：
- 修改现有结算顺序（晕眩 → DoT → HoT → tick）
- 修改 tick 递减逻辑

优先检查：
- 结算顺序是否影响其他效果
- Message 是否需要新增
- BattleRecord 是否需要适配

---

## 如果新增 Buff 交互规则

允许：
- 新增同源刷新变体

禁止：
- 修改"不同源可共存"规则
- 修改标签安全移除逻辑

优先检查：
- 同源刷新条件
- 标签共享检查
- 修饰符清理

---

## 如果测试失败

排查顺序：
1. 检查 Buff 是否正确施加（修饰符 + 标签）
2. 检查 tick 递减顺序（先移除再递减）
3. 检查过期 Buff 修饰符是否清理
4. 检查标签重建是否正确（三层叠加）
5. 检查同源刷新是否生效
