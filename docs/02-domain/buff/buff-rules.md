---
id: 02-domain.buff.buff-rules
title: Buff Rules
status: draft
owner: domain-designer
created: 2026-06-14
updated: 2026-06-14
tags:
  - domain
  - buff
---

# Buff 系统领域

Version: 1.1
Status: Proposed
Changelog: v1.1 — 新增"与 Effect 领域的边界"章节、Buff→Effect→Modifier→Tag 数据流、ExecutionStack 协作说明、SRPG-GAS 对齐声明

Buff 系统领域管理持续性效果（Buff/Debuff）的定义、施加、持续、触发、叠层和移除。Buff 是 SRPG 战斗系统的核心组成之一，与 Skill、Effect Pipeline、Modifier 管线紧密协作。

核心原则：
- 🟩 Buff = 临时 Trait + 临时 Modifier（宪法 3.3）
- 🟩 Buff 通过 Effect Pipeline 执行其效果列表，Buff 自身不直接修改游戏状态
- 🟩 每个 Buff 必须有明确的持续时间，禁止永久存在（除非 DurationPolicy::Permanent 显式声明）
- 🟩 每个 Buff 必须有明确来源，禁止无来源 Buff
- 🟩 Buff 的施加和移除必须触发对应的修饰符清理和标签重建
- 🟩 新增 Buff = 新增 RON 文件，不修改 Rust 代码
- 🟥 禁止绕过 Effect Pipeline 直接在 apply_buff 中扣血/回血
- 🟥 禁止 Buff 直接执行效果——Buff 只声明 EffectDef[]，由 Effect Pipeline 执行

本领域的高层模型：Buff = Trigger[]（什么时候触发）+ Duration（持续多久）+ StackPolicy（如何叠层）+ Condition[]（触发条件）+ Effect[]（触发什么效果）+ Tags（分类标签）。

参见：`trigger-rules.md`, `duration-rules.md`, `stack-policy-rules.md`, `condition-rules.md` 提供各个子系统的详细规则。

---

# 术语定义

## Buff（持续性效果）

🟩 单位身上的临时性增益/减益效果，有持续时间限制。Buff = 临时 Trait（授予/移除标签和能力）+ 临时 Modifier（修改属性值）+ 临时 Effect[]（触发时执行的效果）。

不是 Effect。不是 Skill。不是 Trait 本身——Buff 是 Trait 的临时载体。

关键属性：
- 定义态为 BuffDef（RON 反序列化用），运行态为 BuffData
- 包含：id、name、effects、duration、stack、conditions、tags
- 效果列表中的每个 EffectDef 描述一种触发时执行的效果（通过 Effect Pipeline 执行，Buff 不直接执行）
- 通过 BuffRegistry 按 ID 查询
- `is_buff` 字段区分 Buff（增益）和 Debuff（减益）

## Buff 定义（BuffDef / BuffData）

RON 文件中的 Buff 配置。双类型模式：BuffDef 使用 TagName（RON 友好），BuffData 使用 GameplayTag（运行时位掩码）。

不是运行时实例。不是 ActiveBuffs 组件。不是 Effect Pipeline。

关键属性：
- `effects: Vec<EffectDef>` — 触发时执行的效果列表（替代过时的扁平字段 dot_damage/hot_heal）
- `duration: DurationDef` — 持续策略（替代过时的 default_duration: u32）
- `stack: StackDef` — 叠层策略
- `conditions: Vec<BuffConditionDef>` — 触发条件（可选）

## Buff 实例（BuffInstance）

单位身上的运行时 Buff 状态。附加在 ActiveBuffs 组件中。

不是 BuffDef。不是 BuffData。不是 BuffRegistry。

关键属性：
- instance_id — 全局唯一实例 ID（递增分配）
- buff_id — 关联的 BuffData ID
- remaining_turns — 剩余回合数（仅对 DurationPolicy::Turns 有效）
- source_entity — 来源实体（谁施加的）
- 包含 snapshot 的 tags、is_buff、dot_damage、hot_heal

## 活跃 Buff 列表（ActiveBuffs）

单位身上的所有运行时 Buff 实例的容器，附加在 Unit 实体上。

不是 BuffRegistry。不是 BuffData。不是单独的 BuffInstance。

关键属性：
- `instances: Vec<BuffInstance>` — 按施加顺序排列
- 支持按 instance_id 的 O(n) 查找（n 通常 ≤ 10）
- 提供 tick()、is_stunned()、dot_damage()、hot_heal() 聚合查询

## 施加（Apply）

给单位添加一个 Buff 实例的过程。

不是技能效果执行。不是 Modifier 添加。不是标签添加。

流程自动包含：
1. 查找 BuffData 定义
2. 执行 StackPolicy 检查（叠层/刷新/拒绝）
3. 创建 BuffInstance
4. 添加 Modifier 到 Attributes
5. 添加 Tag 到 GameplayTags
6. 注册到 ActiveBuffs

## 移除（Remove）

从单位移除一个 Buff 实例的过程。

不是过期检查。不是 Attribute 恢复。不是标签清理。

流程自动包含：
1. 从 ActiveBuffs 移除实例
2. 从 Attributes 移除对应 Modifier
3. 从 GameplayTags 移除对应的 Tag（仅当无其他 Buff 提供相同 Tag）

## 来源（Source）

每个 Buff 实例必须记录来源实体（source_entity: Option\<Entity\>）。

来源为空（None）时的处理规则：
- 允许的空来源场景：系统生成 Buff、地形 Buff、被动 Buff
- 禁止的空来源场景：技能施加的 Buff、装备施加的 Buff

---

# 状态机

## Buff 实例生命周期

```
定义（BuffDef / RON）
  │ [加载]
  ▼
注册（BuffRegistry）
  │ [技能效果/装备/事件触发]
  ▼
施加（ActiveBuffs ← BuffInstance）
  │ [每秒/每回合]
  ▼
活跃（持续中）
  │ [持续时间到达 / 条件触发 / 手动移除]
  ▼
移除（清理 Modifier + Tag）
  │
  └──→ 生命周期结束
```

## 施加管线

```
can_apply 前置检查
  │ [检查：目标已死亡/免疫/冲突/叠层上限]
  ▼
StackPolicy 检查
  ├── NoStack → 检查同 id 实例 → 刷新 duration 或拒绝
  ├── Stackable(n) → 检查层数 → 新增实例 或 刷新最旧层
  └── StackableNoRefresh(n) → 检查层数 → 新增实例 或 静默跳过
  │
  ▼
生成 BuffInstance
  │
  ▼
添加 Attribute Modifier
  │
  ▼
添加 GameplayTag
  │
  ▼
注册到 ActiveBuffs
```

## 生命周期阶段

| 阶段 | 触发时机 | 执行动作 |
|------|----------|----------|
| **Apply** | 效果执行时 | 创建实例、添加 Modifier、添加 Tag |
| **Tick** | TurnPhase::SelectUnit | 递减 remaining_turns、DoT/HoT 结算 |
| **Expire Check** | Tick 后 | remaining_turns == 0 → 加入过期列表 |
| **Remove** | 过期/驱散/手动 | 清理 Modifier、清理 Tag、触发 BuffRemoved |
| **Rebuild Tags** | Remove 后 | 三层标签重建（Trait → Equipment → Buff） |

---

# 不变量（Invariants）

### INV-BUFF-001：Buff 必须有来源
- 条件：技能/装备施加 Buff 时
- 不变量：source_entity 必须为 Some
- 违反后果：系统生成 Buff 允许 None，其他情况禁止
- 豁免机制：系统 Buff（地形、被动）可豁免

### INV-BUFF-002：Buff 必须有持续时间上限
- 条件：任何 Buff 实例创建时
- 不变量：DurationPolicy != Permanent 时，remaining_turns \<= 99
- 违反后果：禁止创建，返回错误

### INV-BUFF-003：Modifier 与 Buff 一一对应
- 条件：Buff 施加/移除时
- 不变量：施加时添加的 Modifier 数量 == 移除时清理的 Modifier 数量
- 违反后果：Attribute 数据不一致

### INV-BUFF-004：标签无残留
- 条件：Buff 移除后
- 不变量：移除的 Buff 提供的 Tag 不再出现在 GameplayTags 中（除非被其他 Buff/Trait/Equipment 提供）
- 违反后果：残留标签导致不正确的 Modifier 匹配

### INV-BUFF-005：同源同 id 刷新不重复添加 Modifier
- 条件：source 相同且 buff_id 相同时
- 不变量：不新增实例，只刷新 remaining_turns
- 违反后果：Modifier 叠加超出预期

### INV-BUFF-006：过期 Buff 必须清理
- 条件：remaining_turns 递减到 0 后
- 不变量：必须执行 Modifier 清理 + Tag 清理 + 触发 BuffRemoved 事件
- 违反后果：属性永久修改

### INV-BUFF-007：叠层上限检查
- 条件：任何 Buff 施加前
- 不变量：当前层数 + 1 <= stack_policy.max
- 违反后果：超出上限的施加被拒绝（StackableNoRefresh）或刷新最旧层（Stackable）

### INV-BUFF-008：三分法原则
- 条件：Buff 的标签不应覆盖、受限于 Trait 或 Equipment 的标签
- 不变量：Buff 标签只在第三层存在
- 违反后果：Trait 标签被 Buff 覆盖

---

# 禁止事项（Forbidden）

- 🟥 禁止：apply_buff() 直接扣血/回血 — 理由：必须通过 Effect Pipeline 执行效果
- 🟥 禁止：Buff 直接执行 EffectDef[] 中的效果 — 理由：Buff 只声明效果，由 Effect Pipeline 执行（详见 `docs/02-domain/effect/effect-rules.md`）
- 🟥 禁止：Buff 无来源 — 理由：不可审计
- 🟥 禁止：Buff 永不过期 — 理由：必须有 DurationPolicy 兜底
- 🟥 禁止：跳过 Modifier 清理 — 理由：属性残留
- 🟥 禁止：跳过 Tag 重建 — 理由：标签残留
- 🟥 禁止：多次施加同源同 id Buff 时重复添加 Modifier — 理由：刷新语义
- 🟥 禁止：StackableNoRefresh 超过上限时静默覆盖 — 理由：应当拒绝
- 🟥 禁止：为新增 Buff 修改 Rust 代码 — 理由：新增内容 = 新增 RON 文件
- 🟥 禁止：运行时修改 BuffDef — 理由：Definition/Instance 分离

---

# 流程定义

## 5.1 施加 Buff（Apply Buff）

- 输入：
  - active_buffs: ActiveBuffs（目标单位的活跃 Buff 列表）
  - attributes: Attributes（目标单位的属性，用于添加 Modifier）
  - tags: GameplayTags（目标单位的标签，用于添加 Buff 标签）
  - buff_data: BuffData（Buff 定义数据）
  - source_entity: Option\<Entity\>（来源实体）
  - duration: u32（本次施加的持续时间）
- 处理：
  1. 检查 buff_data 是否为空（数据完整性验证）
  2. Cleanse 特殊处理：如果是驱散，移除所有 Debuff 并返回
  3. StackPolicy 检查：同源同 id → 刷新 duration，不重复添加 Modifier
  4. 不同源或新 id → 检查 StackPolicy 上限
     - StackableNoRefresh 且已达上限 → 静默跳过
     - Stackable 且已达上限 → 依次移除最旧实例再新增
     - 未达上限 → 直接新增
  5. 生成 BuffInstance（分配 instance_id）
  6. 添加 Attribute Modifier
  7. 添加 GameplayTag
  8. 注册到 ActiveBuffs
- 输出：BuffInstanceId
- 失败处理：数据不存在时不施加（静默跳过）

## 5.2 移除 Buff（Remove Buff）

- 输入：instance_id、active_buffs、attributes、tags
- 处理：
  1. 按 instance_id 查找 BuffInstance
  2. 从 ActiveBuffs 移除
  3. 从 Attributes 移除对应 Modifier（通过 ModifierSource::BuffInstanceId）
  4. 从 GameplayTags 移除 Buff 提供的 Tag（仅当无其他 Buff 提供相同 Tag）
- 输出：被移除的 BuffInstance 或 None

## 5.3 持续效果结算（Tick）

- 输入：所有单位 + ActiveBuffs + Attributes + GameplayTags
- 触发时机：OnEnter(TurnPhase::SelectUnit)
- 处理：
  1. Stun 结算：被晕眩的单位标记 acted=true，移除 Stun Buff
  2. DoT 结算：汇总 dot_damage → 生成 EffectDef::Damage → 进入 Effect Pipeline 执行（禁止直接扣 HP）
  3. HoT 结算：汇总 hot_heal → 生成 EffectDef::Heal → 进入 Effect Pipeline 执行（禁止直接加 HP）
  4. Duration 递减：对 DurationPolicy::Turns(n) 递减 remaining_turns
  5. 过期清理：remaining_turns == 0 的 Buff 加入过期列表
  6. 对过期 Buff 执行 Remove 流程
  7. 三层标签重建
- 输出：DotApplied、HotApplied、StunApplied、BuffRemoved 事件
- 关键约束：DoT/HoT 的 HP 变更必须通过 Effect Pipeline，保证修饰规则生效和可观测性

## 5.4 三层标签重建（Rebuild Tags）

- 输入：active_buffs、tags、persistent_tags（from_traits + from_equipment）
- 处理：
  1. 清零 GameplayTags
  2. 第一层：Trait 授予 Tag（from_traits，最持久）
  3. 第二层：装备授予 Tag（from_equipment，穿脱变化）
  4. 第三层：活跃 Buff 授予 Tag（临时）
  5. 合并写入 GameplayTags
- 输出：更新后的 GameplayTags

---

# 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| BuffApplied | Buff 施加成功后 | target, target_name, buff_id, source, remaining_turns | UI 图标更新、战斗日志、回放录制 |
| BuffRemoved | Buff 移除后 | target, target_name, buff_id, reason(Expired/Dispelled/Replaced/Manual) | UI 图标刷新、战斗日志 |
| DotApplied | DoT 伤害结算后 | target, target_name, amount | 战斗日志、飘字 |
| HotApplied | HoT 治疗结算后 | target, target_name, amount | 战斗日志、飘字 |
| StunApplied | 晕眩生效时 | target, target_name, duration | 战斗日志、UI 状态 |

---

# 与 Effect 领域的边界

Buff 和 Effect 是紧密协作但职责清晰分离的两个领域。Buff 管理"持续性状态"，Effect 管理"一次性行为"。

## 边界原则

| 维度 | Buff 领域负责 | Effect 领域负责 |
|------|-------------|----------------|
| **定义** | BuffDef/BuffData（触发条件、持续策略、叠层策略） | EffectDef（Damage/Heal/ApplyBuff/Cleanse） |
| **触发** | Trigger 时机点匹配（TurnStart/AfterDamaged 等） | — |
| **效果声明** | effects: Vec\<EffectDef\>（声明触发时执行什么效果） | — |
| **效果执行** | — | Effect Pipeline（Generate → Modify → Execute） |
| **Modifier 管理** | 施加/移除 AttributeModifier（通过 ModifierSource::Buff） | — |
| **Tag 管理** | 施加/移除 GameplayTag | — |
| **生命周期** | Apply → Tick → Expire → Remove | 单次执行，无生命周期 |

## 数据流：Buff → Effect → Modifier → Tag

```
Buff 触发（如 TurnStart）
    ↓
TriggerRegistry 匹配 → TriggerHandler.handle(ctx)
    ↓
返回 Vec<EffectDef>（Buff 的 effects 列表）
    ↓
压入 ExecutionStack（LIFO，见 trigger-rules.md §执行栈）
    ↓
Stack 弹出 → Condition 检查 → Effect Pipeline
    ↓
Generate → Modify → Execute
    ↓
EffectResult（damage_dealt / healing_done / buff_applied）
    ↓
领域事件（DamageApplied / HealApplied / BuffApplied）
    ↓
订阅者（BattleRecord / UI / Trigger 链）
```

**关键约束**：Buff 只声明 EffectDef[]，不直接执行效果。所有效果执行必须通过 Effect Pipeline。这保证了：
- Buff 不感知具体的伤害计算逻辑
- Effect 不感知触发来源（Skill 还是 Buff）
- Modifier 管线在 Modify 阶段统一处理修饰
- Tag 在 Execute 阶段后由订阅者响应

## 不变量交叉检查

| 不变量 | 归属 | 说明 |
|--------|------|------|
| Buff 必须有来源 | Buff | source_entity 必须为 Some |
| Buff 通过 Effect Pipeline 执行效果 | Buff → Effect | 禁止直接扣血/回血 |
| Effect 不感知触发来源 | Effect | Effect 不知道是 Skill 还是 Buff 触发的 |
| Modifier 在 Modify 阶段介入 | Effect → Modifier | Modifier 不修改 World 状态 |
| EffectResult 包含完整执行结果 | Effect | damage_dealt / healing_done / buff_applied |

## ExecutionStack 协作

Buff 触发的效果可能产生嵌套触发（如：中毒触发 → 伤害 → 触发荆棘 → 反伤），通过 ExecutionStack（LIFO 响应栈）管理：

```
Buff 触发 → EffectDef[] 压入 Stack
    ↓
Stack 弹出 → Effect Pipeline 执行
    ↓
EffectResult 产生新事件 → 可能触发新 Buff 的 Trigger
    ↓
新 EffectDef[] 压入 Stack（深度 +1）
    ↓
递归，直到栈空或深度 ≥ MAX_STACK_DEPTH（32）
```

> 详见 `docs/02-domain/trigger/trigger-rules.md` §执行栈（ExecutionStack）章节。
> Effect 领域不感知 Stack，Effect 只做好自己的三步（Generate → Modify → Execute）。Stack 是 Trigger 领域的事情。

---

# SRPG-GAS 对齐声明

本项目的 Buff 系统与 UE GAS（Gameplay Ability System）的概念映射：

| UE GAS 概念 | 本项目 Buff 领域对应 | 说明 |
|-------------|-------------------|------|
| GameplayEffect | BuffDef / BuffData | 持续性效果的定义 |
| DurationPolicy | DurationPolicy（Turns/UntilDeath/...） | 持续策略 |
| StackPolicy | StackPolicy（NoStack/Stackable/...） | 叠层策略 |
| GameplayEffectPeriod | Trigger[]（TurnStart/AfterDamaged 等） | 触发时机 |
| GameplayEffectModifier | AttributeModifierDef（Add/Multiply/Override） | 属性修饰 |
| GameplayTag | GameplayTag（位掩码标签） | 分类与匹配 |

本项目裁剪了 GAS 的网络同步、预测回滚等复杂度，保留了 Ability + Effect + Tag + Modifier 的核心抽象，并适配为回合制 SRPG 领域模型。

---

# 与相邻领域的关系

| 相邻领域 | 关系 | 边界 |
|----------|------|------|
| **Skill** | 技能效果列表可包含 ApplyBuff → 通过 EffectPipeline 最终调用 apply_buff | Skill 不直接调用 apply_buff；Skill 不关心 Buff 的内部逻辑 |
| **Effect Pipeline** | ApplyBuff 效果通过 EffectHandler 分发 → handler.execute() → apply_buff；Buff 的 effects 列表也引用 EffectDef，触发时进入同一 Effect Pipeline | Effect Pipeline 负责调度，Buff 模块负责施加。详见 `docs/02-domain/effect/effect-rules.md` |
| **Attribute** | apply_buff 添加 Modifier，remove_buff 清理 Modifier | 通过 ModifierSource::BuffInstanceId 关联 |
| **Tag** | Buff 携带分类标签（BUFF/DEBUFF/POISON/STUN/FIRE），参与战斗力修饰规则 | 标签是 Buff 的元数据，不决定 Buff 生命周期 |
| **Trigger** | Buff 触发行为（OnTurnStart DoT、AfterDamaged 荆棘）；Trigger 决定"什么时候触发"，Buff 决定"触发的效果" | Trigger 产生 EffectDef[] → 压入 ExecutionStack → Effect Pipeline 执行。详见 `docs/02-domain/trigger/trigger-rules.md` |
| **ExecutionStack** | Buff 触发的效果可能产生嵌套触发，通过 LIFO 栈管理递归 | Effect 领域不感知 Stack；Stack 由 Trigger 领域管理，MAX_STACK_DEPTH = 32 |
| **Duration** | 持续策略决定 Buff 何时过期 | DurationPolicy 是 Buff 的可组合策略组件 |
| **Stack** | 叠层策略决定多个 Buff 实例之间的关系 | StackPolicy 是 Buff 的可组合策略组件 |
| **Condition** | 触发条件决定 Buff 效果是否生效 | Condition 在触发时评估，失败则效果静默跳过 |
| **Turn** | Tick 生命周期与 TurnPhase::SelectUnit 同步 | Buff 不直接操作回合切换 |
