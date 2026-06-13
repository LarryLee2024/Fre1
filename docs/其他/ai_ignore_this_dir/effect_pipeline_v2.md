# Effect Pipeline 领域

Version: 2.0

## Purpose

Effect Pipeline 领域管理战斗效果的数据流，从技能定义生成效果、修饰效果、执行效果。采用 Generate → Modify → Execute 三步管线，ECS 是数据流而非调用链。

---

## Glossary

| 术语 | 定义 | 易混淆项 |
|------|------|----------|
| EffectDef | 技能定义中的效果配置，描述"产生什么效果" | ≠ PendingEffectData：Def 是配置，Data 是运行时数据 |
| PendingEffect | 待处理效果，尚未执行 | ≠ 最终伤害：是中间状态，必须经过完整管线 |
| PendingEffectData | 待处理效果的数据，包含计算后的数值 | ≠ EffectDef：Data 有计算后的数值，Def 只有配置参数 |
| EffectQueue | 战斗唯一效果缓冲区，管线三步共享 | ≠ 直接执行通道：所有效果必须进入队列 |
| EffectHandler | 效果处理器 Trait，负责 Generate 和 Preview | ≠ EffectDef：Handler 是执行者，Def 是配置 |
| GenerateContext | Generate 阶段的上下文，包含属性快照 | ≠ ECS Query：Context 是快照，避免借用冲突 |

---

## Responsibilities

### Owns

- EffectDef 定义
- EffectHandler 注册表
- EffectQueue 管理
- Generate / Modify / Execute 三步管线的流程定义
- PendingEffect 和 PendingEffectData 的数据结构
- EffectPreview 预览
- 伤害计算公式

### Does Not Own

- ModifierRule 的定义和匹配 → modifier_rules
- Buff 的施加和移除 → buff_rules
- CombatIntent 的设置 → battle_rules
- 死亡判定的 Hook/Observer/Message → battle_rules
- UI 展示 → ui_rules

---

## Invariants

### INV-EFP-01：管线严格顺序 🟥

宪法：2.3.1

效果必须按 Generate → Modify → Execute 顺序处理，不可跳步或乱序。

违反：Generate 直接扣血、Modify 发送消息、Execute 创建新攻击意图。

### INV-EFP-02：EffectQueue 执行后清空 🟥

Execute 阶段结束后，EffectQueue 必须为空。

违反：残留效果在下一轮被重复执行。

### INV-EFP-03：伤害下限 🟥

伤害值 ≥ 1。

违反：伤害为 0 或负数。

### INV-EFP-04：base_amount 首次记录 🟩

Modify 阶段首次记录 Generate 阶段的原始值，后续 Modify 不覆盖。

违反：多次 Modify 覆盖 base_amount，伤害分解显示错误。

### INV-EFP-05：Generate 不修改 ECS 🟥

宪法：2.3.1, 1.1.4

Generate 阶段完成后，所有 ECS 组件和 Resource 无变化（除 EffectQueue）。

违反：Generate 阶段扣减 HP 或发送 Message。

### INV-EFP-06：属性修改必须通过 Modifier 🟥

宪法：2.2.1

伤害/治疗修饰必须通过 ModifierRule 管线，禁止直接修改最终属性值。

违反：在 Generate 或 Modify 阶段直接修改 HP。

### INV-EFP-07：Handler Trait 分发 🟩

宪法：6.0.2

效果通过注册表分发，禁止 match 分发效果类型。新增效果类型只需实现 Trait 并注册。

违反：新增效果类型需要修改分发代码。

---

## State Machine

### 效果状态

| 状态 | 含义 | 转换到 |
|------|------|--------|
| Defined | EffectDef 在 SkillData 中 | — |
| Generated | PendingEffect 在 EffectQueue 中 | Modified |
| Modified | 修饰完成 | Executed |
| Executed | 已消费，属性已变化 | — |

```
Defined → Generated → Modified → Executed
```

---

## Business Rules

### BR-EFP-01：Generate

- 通过 EffectHandlerRegistry 分发
- 从 EffectDef 生成 PendingEffectData
- 组装 PendingEffect 推入 EffectQueue
- 触发 OnAttack Trait
- 类型不匹配时 Handler 返回 None

### BR-EFP-02：Modify

- Damage 走伤害修饰管线
- Heal 走治疗修饰管线
- ApplyBuff / Cleanse 不修饰
- 首次记录 base_amount

### BR-EFP-03：Execute

- 消费所有效果并清空队列
- Damage：扣血 + 死亡判定 + Message
- Heal：回血 + Message
- ApplyBuff：施加 Buff
- Cleanse：驱散 Debuff

### BR-EFP-04：伤害计算

- 有效攻击力 → 减去防御 → 减去地形加成 → 乘以倍率 → 下限保护
- 无视防御百分比基于 base_def（非 effective_def）
- 下限保护：max(1, result)

---

## Pipelines

### 效果管线

效果定义 → 生成效果 → 修饰效果 → 执行效果

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| Generate | 攻击意图 + 技能定义 + 上下文 | 待处理效果队列 | 禁止修改 ECS 状态（INV-EFP-05） |
| Modify | 效果队列 + 修饰规则 | 修饰后效果队列 | 禁止修改 ECS 状态、禁止创建新效果 |
| Execute | 修饰后效果队列 | 属性变化 + 消息 | 禁止创建新攻击意图（INV-EFP-01） |

### 伤害计算管线

有效攻击力 → 减去防御 → 减去地形加成 → 乘以倍率 → 下限保护

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 计算防御 | 有效防御 + 基础防御 + 无视百分比 | 最终防御 | 无视防御基于 base_def |
| 基础伤害 | 有效攻击 + 最终防御 | 基础伤害 | — |
| 地形和倍率 | 基础伤害 + 地形加成 + 倍率 | 结果 | — |
| 下限保护 | 结果 | 最终伤害 | 必须 max(1, result) |

---

## Data Model

### EffectDef（Definition）

技能中的效果配置。

- Damage：倍率 + 无视防御百分比
- Heal：固定治疗量
- ApplyBuff：buff_id + duration
- Cleanse：无参数

### PendingEffectData（Instance）

待处理效果的运行时数据。

- Damage：amount / base_amount / modifiers
- Heal：amount / base_amount
- ApplyBuff：buff_id / duration
- Cleanse：无参数

### EffectQueue（Resource）

战斗唯一效果缓冲区。

- 待处理效果列表
- Generate 推入，Modify 修改，Execute 清空

### EffectHandler（Trait）

效果处理器接口。

- 输入：EffectDef + GenerateContext
- 输出：PendingEffectData / EffectPreview
- 内置四个 Handler

### GenerateContext（值对象）

Generate 阶段上下文快照，纯数据。

- 攻击者/目标属性快照
- 地形防御加成
- 技能 ID / 标签 / 地形 ID

---

## Cross Domain Contracts

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 效果生成 | 推入 EffectQueue | battle |
| 效果修饰 | 调用 ModifierRuleRegistry | modifier_rules |
| 效果执行 | 修改 Attributes + 发送 Message | battle / ui |

---

## Change Rules

### 新增效果类型

- 允许：新增 EffectDef 变体 + PendingEffectData 变体 + EffectHandler 实现
- 禁止：修改管线流程（INV-EFP-01）、match 分发（INV-EFP-07）
- 检查：注册表注册、Modify/Execute 是否需要适配、Preview 是否需要配套

### 修改伤害公式

- 允许：修改伤害计算逻辑
- 禁止：修改管线三步流程、修改下限保护规则
- 检查：伤害下限 ≥ 1、无视防御基于 base_def

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
| INV-EFP-01 | 效果管线跳步 | Generate → Modify → Execute 严格顺序 | 检查管线是否完整执行 |
| INV-EFP-05 | Generate 阶段修改 ECS | Generate 只生成效果，不执行 | 将 ECS 修改移到 Execute 阶段 |
| INV-EFP-06 | 直接修改最终属性值 | 属性修改必须通过 Modifier 管线 | 通过 ModifierRule 管线修饰 |
| INV-EFP-07 | match 分发效果类型 | 应通过 EffectHandler Trait 分发 | 改为注册表查找 |

---

## Test Requirements

宪法：13.0.1-13.0.3

- 单元测试：验证管线各阶段输入输出
- 集成测试：验证完整效果管线
- Bug 修复必须先编写重现测试
- Battle Replay 优先于手工验证

排查顺序：
1. 管线是否跳步
2. EffectQueue 是否在 Execute 后清空
3. Handler 是否正确注册
4. base_amount 是否首次记录
5. 伤害下限保护
