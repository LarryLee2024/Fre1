---
id: 02-domain.effect.effect-rules
title: Effect Rules
status: draft
owner: domain-designer
created: 2026-06-14
updated: 2026-06-14
tags:
  - domain
  - effect
---

# 效果系统领域

Version: 1.1
Status: Proposed
Source: `docs/其他/76.md` §三（领域职责）— Effect 作为独立一级领域
Changelog: v1.1 — 对齐 attribute-modifier-rules.md（EffectDef 使用富定义 Damage{multiplier,ignore_def_percent}），明确 Pipeline 所有权边界，补充 ExecutionStack 协作说明

效果系统领域管理"能力执行后产生什么结果"——Damage、Heal、ApplyBuff、Cleanse 等原子操作。Effect 是 Skill 和 Buff 的"执行子"，承上启下，是 SRPG-GAS 架构中最关键的一层。

**核心原则**：
- 🟩 Effect = 一次性行为，不管理持续状态（那是 Buff 的职责）
- 🟩 所有 Effect 通过 Generate → Modify → Execute 三步管线执行
- 🟩 Effect 是独立的一级领域，不属于 Skill 也不属于 Buff
- 🟥 禁止跳过 Effect Pipeline 直接修改游戏状态
- 🟥 禁止在 Generate/Modify 阶段产生副作用

**领域定位**：

```
Skill ── 意图（Intent）：我要做什么
  ↓
Effect ── 结果（Result）：产生什么效果 ← 本领域
  ↓
Buff ── 状态（State）：持续性状态
  ↓
Modifier ── 计算规则（Rule）：如何调整数值
```

---

# 术语定义

## 效果（Effect）

能力执行后产生的原子性结果。每个 Effect 是瞬时行为，执行完毕后即结束。

不是 Skill。不是 Buff。不是 Modifier。

关键属性：
- 由 EffectDef 定义（RON 反序列化用），运行时转换为 EffectData
- 每种效果类型对应一个 EffectHandler 实现
- 效果执行通过 Effect Pipeline（Generate → Modify → Execute）
- EffectHandlerRegistry 全局管理所有效果处理器

---

## 效果定义（EffectDef）

效果类型的 RON 配置表示，引用 Effect 类型和参数。与 `docs/02-domain/attribute-modifier/attribute-modifier-rules.md` §EffectDef 保持一致。

不是运行时数据。不是 EffectHandler。不是 PendingEffect。

```ron
// 在 SkillDef / BuffDef 中嵌入使用
effects: [
    Damage { multiplier: 1.2, ignore_def_percent: 0 },
    Heal { amount: 100 },
    ApplyBuff { buff_id: "burn", duration: 3 },
    Cleanse,
]
```

当前支持的 EffectDef 变体（对齐 attribute-modifier-rules.md）：

| 变体 | 说明 | 参数 |
|------|------|------|
| Damage | 造成伤害 | multiplier: f32（伤害倍率）, ignore_def_percent: f32（忽略防御百分比） |
| Heal | 恢复生命 | amount: i32 |
| ApplyBuff | 施加 Buff | buff_id: String, duration: u32 |
| Cleanse | 驱散 | 无参数（驱散所有 Debuff） |

> **Design Note**：Damage 使用 multiplier 而非 flat amount，是因为最终伤害值由 Formula 在 Generate 阶段计算，EffectDef 只声明意图（倍率/忽略防御比例）。这与 UE GAS 的 GameplayEffect Modifier 语义一致。

---

## 效果处理器（EffectHandler）

每种 Effect 类型的执行逻辑封装。通过 EffectHandler trait 定义接口。Modify 阶段的内部逻辑由 AttributeModifier 领域负责（ModifierRuleRegistry + ModifierCalculator），EffectHandler 不参与 Modify 阶段。

不是 Effect 本身。不是 EffectDef。不是 EffectRegistry。

关键属性：
- 实现 EffectHandler trait（type_name / generate / preview / execute）
- 注册到 EffectHandlerRegistry，运行时通过 type_name 查找分发
- 新增效果类型只需实现 trait 并注册，不修改管线代码
- 500 技能 ≈ 20-30 种原子 Effect 组合，不需要为每个技能写独立逻辑

```rust
pub trait EffectHandler: Send + Sync {
    fn type_name(&self) -> &'static str;
    fn generate(&self, ctx: &GenerateContext) -> Option<PendingEffectData>;
    fn preview(&self, ctx: &PreviewContext) -> Option<EffectPreview>;
    fn execute(&self, ctx: &mut ExecuteContext) -> Option<EffectResult>;
}
```

---

## 待处理效果（PendingEffectData）

Effect Pipeline 中流转的中间数据，在 Generate 阶段创建，Modify 阶段修饰，Execute 阶段消费。详见 `docs/02-domain/attribute-modifier/attribute-modifier-rules.md` PendingEffect/PendingEffectData 章节。

不是最终结果。不是 EffectDef。不是 EffectResult。

结构：
```rust
// 包装结构
pub struct PendingEffect {
    pub source: Entity,
    pub target: Entity,
    pub data: PendingEffectData,
    pub source_tags: GameplayTags,
    pub terrain_id: Option<u32>,
}

// 效果数据枚举
pub enum PendingEffectData {
    Damage { amount: i32, is_skill: bool, base_amount: i32, modifiers: Vec<ModifierEntry> },
    Heal { amount: i32, base_amount: i32, modifiers: Vec<ModifierEntry> },
    ApplyBuff { buff_id: String, duration: u32 },
    Cleanse,
}
```

要求：
- base_amount 在 Modify 阶段首次设置（记录修饰前原始值）
- modifiers 在 Modify 阶段填充（每步 ModifierEntry）
- 不直接存储 Entity 引用（存储 Entity ID）
- 纯数据传递，不包含 ECS 引用
- 与 attribute-modifier-rules.md 中的定义保持一致

---

## 效果结果（EffectResult）

Effect 执行完成后的统一返回结果，包含目标状态和产生的消息。详见 `docs/02-domain/attribute-modifier/attribute-modifier-rules.md` EffectResult 章节。

不是 PendingEffectData。不是领域事件。不是 EffectHandler。

关键属性：
- target_died：bool — 目标是否死亡
- damage_dealt：i32 — 实际造成的伤害值
- healing_done：i32 — 实际完成的治疗值
- buff_applied：bool — 是否成功施加 Buff
- PendingMessage 列表：DamageApplied / HealApplied / BuffApplied / EntityDied / EffectCompleted
- 所有 Effect Handler 执行后统一返回 EffectResult
- 用于触发后续检查（死亡处理、击杀触发等）

---

## 效果管线（Effect Pipeline）

三步执行的标准化效果处理管线。详见 `docs/01-architecture/skill-buff-abstraction.md` §3 和 `docs/02-domain/attribute-modifier/attribute-modifier-rules.md` 效果管线章节。

Generate → Modify → Execute

**所有权声明**：Effect 领域拥有 Pipeline 的编排调度（三步顺序、Handler 分发）；Modify 阶段的内部逻辑（ModifierRuleRegistry 匹配、Calculator 分发）由 AttributeModifier 领域负责。

---

## 效果注册表（EffectHandlerRegistry）

全局唯一的效果处理器注册表 Resource。

不是 SkillRegistry。不是 BuffRegistry。不是 ModifierRuleRegistry。

关键属性：
- HashMap<String, Box<dyn EffectHandler>> 存储
- 通过 type_name 字符串查找
- 游戏初始化时注册所有效果处理器
- 新增效果类型只需注册，不修改管线代码

---

# 领域边界

## 本领域负责

- EffectDef 类型定义（Damage / Heal / ApplyBuff / Cleanse 等）
- EffectHandler trait 定义
- EffectHandlerRegistry 管理
- Effect Pipeline 三步编排调度（Generate → Modify → Execute 的顺序控制）
- PendingEffectData 中间数据定义
- EffectPreview 预览数据定义
- EffectResult 结果定义

## 本领域不负责

- Skill 的释放管线（由 Skill 领域负责）
- Buff 的生命周期管理（由 Buff 领域负责）
- **Modify 阶段内部逻辑**：ModifierRuleRegistry 管理、ModifierCalculator trait 分发、标签匹配修饰逻辑（由 AttributeModifier 领域负责）。详见 `docs/02-domain/attribute-modifier/attribute-modifier-rules.md`
- **属性计算**：修饰器栈的 Add/Multiply 计算、Derived Stat 公式（由 AttributeModifier 领域负责）
- 公式的定义和注册（由 Formula 领域负责）
- 目标选择逻辑（由 Selector 领域负责）
- 消耗检查和扣除（由 Cost 领域负责）
- 伤害/治疗的最终数值计算（由 Formula 领域负责）
- 触发链调度（由 Trigger 领域负责）
- 战斗记录和审计（由 EventAudit 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 效果执行请求 | 函数调用（Pipeline::execute） | Battle/Turn 领域（效果执行系统） |
| 效果预览请求 | 函数调用（EffectHandler::preview） | UI 领域（伤害预览） |
| 伤害应用 | Message（DamageApplied） | BattleRecord、UI 飘字 |
| 治疗应用 | Message（HealApplied） | BattleRecord、UI 飘字 |
| Buff 施加 | Message（BuffApplied） | Buff 领域 |
| 效果完成 | 函数返回（EffectResult） | Trigger 领域（触发链） |

---

# 生命周期

## Effect 执行状态

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Defined | Effect 已定义（Def 状态） | GenerateStarted |
| GenerateStarted | Generate 阶段执行中 | GenerateCompleted |
| GenerateCompleted | Generate 完成，数据已生成 | ModifyStarted |
| ModifyStarted | Modify 阶段执行中 | ModifyCompleted |
| ModifyCompleted | Modify 完成，数据已修饰 | ExecuteStarted |
| ExecuteStarted | Execute 阶段执行中 | ExecuteCompleted |
| ExecuteCompleted | 效果执行完成 | 终态 |

## 状态转换图

```
Defined → GenerateStarted → GenerateCompleted → ModifyStarted
→ ModifyCompleted → ExecuteStarted → ExecuteCompleted
```

**转换条件**：每个阶段完成后自动进入下一阶段。禁止跳步。

---

# 不变量

## 不变量1：Effect 必须走三步管线

任意时刻：

Effect 执行必须经过 Generate → Modify → Execute 三步。禁止跳步，禁止在 Generate 阶段直接扣血。

违反表现：伤害值未经过 Modifier 修饰、BattleRecord 缺少修饰记录。

> Modify 阶段的内部逻辑（ModifierRuleRegistry 匹配、Calculator 分发）由 AttributeModifier 领域保证。详见 `docs/02-domain/attribute-modifier/attribute-modifier-rules.md` 不变量3。

---

## 不变量2：Generate 和 Modify 为纯函数

任意时刻：

Generate 和 Modify 阶段不产生副作用（不修改游戏状态）。仅 Execute 阶段可修改 World。

违反表现：预览时修改了 HP、Modifier 阶段触发了领域事件。

---

## 不变量3：EffectHandler 通过 Registry 查找

任意时刻：

Effect 执行时通过 EffectHandlerRegistry 查找处理器，禁止在管线代码中硬编码 match 分发。

违反表现：新增效果类型需要修改管线调度代码。

---

## 不变量4：Effect 不感知触发来源

任意时刻：

Effect 不应该知道它是被 Skill 还是 Buff 触发的。Effect 只管执行"放什么效果"，不问"谁让我放的"。

违反表现：Effect 内部判断 is_from_skill / is_from_buff 等来源标志。

---

## 不变量5：每种 Effect 类型有且只有一个 Handler

任意时刻：

同一个 EffectDef 变体在所有 Skill/Buff 中使用相同的 EffectHandler 实现。禁止为不同 Skill 中的同一效果类型实现不同逻辑。

违反表现：fireball 的 Damage 和 basic_attack 的 Damage 使用不同的伤害计算逻辑。

---

# 流程定义

## 执行效果管线

- 输入：EffectDef + GenerateContext（攻击者/目标属性、地形、标签）
- 处理：
  1. 通过 EffectHandlerRegistry 查找 type_name 对应的处理器
  2. 调用 handler.generate(ctx) → PendingEffectData（纯函数）
  3. **委托 AttributeModifier 领域**：遍历 ModifierRuleRegistry 匹配规则，通过 Calculator 计算修饰（纯函数）。详见 `docs/02-domain/attribute-modifier/attribute-modifier-rules.md` 效果管线 Step2
  4. 调用 handler.execute(ctx) → EffectResult（副作用）
  5. 发送 DamageApplied / HealApplied / BuffApplied Message
- 输出：EffectResult（含 target_died 状态）
- 失败处理：handler.generate 返回 None 时静默跳过（类型不匹配）；handler.execute 返回 None 时记录 WARN 日志

---

## 预览效果

- 输入：EffectDef + PreviewContext
- 处理：通过 EffectHandlerRegistry 查找处理器，调用 handler.preview(ctx)
- 输出：EffectPreview（含预览数值）
- 失败处理：handler.preview 返回 None 时跳过（部分效果无法预览）

---

# 数据结构

> 以下数据结构在术语定义章节中已定义，此处仅做索引参考。完整定义见上方术语定义。

| 数据结构 | 定义位置 | 说明 |
|----------|---------|------|
| EffectDef | 术语定义 · 效果定义 | 效果类型枚举（Damage/Heal/ApplyBuff/Cleanse） |
| PendingEffectData | 术语定义 · 待处理效果 | Pipeline 中间数据（含 amount、modifiers） |
| EffectResult | 术语定义 · 效果结果 | 执行结果（含 target_died、damage_dealt 等） |
| EffectPreview | 术语定义 · 效果预览 | UI 展示数据 |

---

# 禁止事项

- 🟥 禁止：跳过 Effect Pipeline 直接扣血/回血 — 理由：管线保证修饰规则和可观测性
- 🟥 禁止：在 Generate/Modify 阶段修改游戏状态 — 理由：纯函数原则
- 🟥 禁止：硬编码 match 分发效果类型 — 理由：必须通过 EffectHandlerRegistry
- 🟥 禁止：Effect 感知触发来源（Skill/Buff） — 理由：职责单一，只做效果执行
- 🟥 禁止：为不同 Skill 中的同一 Effect 类型实现不同 Handler — 理由：统一实现
- 🟥 禁止：新增 Effect 类型时修改管线调度代码 — 理由：扩展性
- 🟥 禁止：Effect 直接调用 Formula 之外的数值计算 — 理由：公式统一管理
- 🟥 禁止：Effect 直接操作 ModifierRuleRegistry 或 ModifierCalculator — 理由：Modify 阶段内部逻辑由 AttributeModifier 领域负责。详见 `docs/02-domain/attribute-modifier/attribute-modifier-rules.md`

---

# 与 AttributeModifier 领域的精确边界

Effect 和 AttributeModifier 是 Pipeline 中紧密协作但职责清晰分离的两个领域。

## 边界原则

| 维度 | Effect 领域负责 | AttributeModifier 领域负责 |
|------|----------------|---------------------------|
| **Pipeline 编排** | 三步顺序控制（Generate→Modify→Execute） | — |
| **Generate 阶段** | EffectHandler.generate() 生成初始值 | — |
| **Modify 阶段** | 调用时机（何时触发 Modify） | 内部逻辑（ModifierRuleRegistry 匹配、Calculator 分发） |
| **Execute 阶段** | EffectHandler.execute() 执行效果 | — |
| **数据定义** | EffectDef、PendingEffectData、EffectResult | ModifierRule、ModifierEntry、ModifierCalculator |
| **注册表** | EffectHandlerRegistry | ModifierRuleRegistry |
| **可观测性** | EffectResult 输出 | ModifierEntry 记录（Modify 阶段每步） |

## 手续交接点

Effect Pipeline 在 Modify 阶段将 PendingEffectData 交给 AttributeModifier 领域处理：

```
Effect 领域                          AttributeModifier 领域
─────────────                       ─────────────────────
Generate 完成
  ↓
PendingEffectData 产出
  ↓
调用 Modify ──────────────────────→ ModifierRuleRegistry 遍历
                                     ↓
                                   标签匹配（source_tag + target_tag）
                                     ↓
                                   Calculator 计算修饰
                                     ↓
                                   记录 ModifierEntry
                                     ↓
修改后的 PendingEffectData ←──────── 返回
  ↓
Execute 阶段
```

## 共享数据结构

以下数据结构由 Effect 领域定义，但在 AttributeModifier 领域中被深度使用：

- **PendingEffectData**：Modify 阶段的输入/输出载体
- **ModifierEntry**：Modify 阶段的每步修饰记录（由 AttributeModifier 填充）
- **PendingEffectData.modifiers**：修饰记录列表（Modify 阶段填充，Execute 阶段消费）

## 不变量交叉检查

Effect 领域的不变量与 AttributeModifier 领域的不变量互补：

| 不变量 | 归属 | 说明 |
|--------|------|------|
| Effect 必须走三步管线 | Effect | 禁止跳步 |
| Generate 和 Modify 为纯函数 | Effect | 不产生副作用 |
| 属性修改必须通过修饰器栈 | AttributeModifier | 禁止直接修改 |
| 每步修饰必须记录 ModifierEntry | AttributeModifier | Modify 阶段填充 |
| 伤害下限 ≥ 1，治疗下限 ≥ 0 | AttributeModifier | Modify 阶段后保证 |

---

# 与相邻领域的关系

| 相邻领域 | 关系 | 边界 |
|----------|------|------|
| **Skill** | Skill 的 effects 列表引用 EffectDef | Skill 不直接调用 EffectHandler；Skill 不关心 Effect 执行细节 |
| **Buff** | Buff 的 effects 列表也引用 EffectDef，触发时进入同一 Effect Pipeline | Buff 不修改 Effect 逻辑；Effect 不感知 Buff 生命周期 |
| **AttributeModifier** | Modifier 在 Effect Pipeline 的 Modify 阶段介入 | Effect 拥有 Pipeline 编排；Modifier 拥有 Modify 阶段内部逻辑（ModifierRuleRegistry、Calculator、标签匹配）。详见 `docs/02-domain/attribute-modifier/attribute-modifier-rules.md` |
| **Formula** | Effect 的 Generate 阶段调用 FormulaRegistry 计算数值 | Effect 不硬编码公式；Formula 是 Effect 的计算来源 |
| **Trigger** | Effect 执行完成后产生领域事件 → Trigger 系统分发 | Effect 不管理触发链；Trigger 不修改 Effect 执行逻辑 |
| **ExecutionStack** | Effect 执行结果可能压入 Stack 产生嵌套触发 | Effect 领域不感知 Stack；Stack 由 Trigger 领域管理。详见 `docs/02-domain/trigger/trigger-rules.md` ExecutionStack 章节 |
| **BattleRecord** | Effect 执行结果记录到 BattleRecord | Effect 执行时发送 Message；BattleRecord 异步消费 |

---

## 附录：铃兰参考数据

> 领域：Effect | 来源：78铃兰.md §四、补充3、补充4、补充5、补充6、补充7、补充10 | 数据层：Definition + Instance

#### EffectDefinition（Definition层）

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| `id` | EffectId | PK | 效果唯一标识 |
| `name_key` | String | - | 效果名称本地化Key（ApplyBuff类型使用） |
| `desc_key` | String | - | 效果描述本地化Key（ApplyBuff类型使用） |
| `effect_type` | EffectType | - | 效果类型 |
| `duration` | Option<DurationDef> | - | 持续时间定义 |
| `stacking` | Option<StackingId> | FK | 堆叠策略引用 |
| `cue` | Option<CueId> | FK | 表现引用 |

#### EffectInstance（Instance层）

| 字段名 | 类型 | 说明 |
|--------|------|------|
| `entity` | Entity | 挂载目标实体 |
| `effect_id` | EffectId | 引用EffectDefinition |
| `source_entity` | Option<Entity> | 来源实体 |
| `remaining_duration` | Option<u32> | 剩余持续回合 |
| `current_stack` | u32 | 当前层数 |
| `lifecycle_phase` | LifecyclePhase | 当前生命周期阶段 |

#### EffectType 枚举

| 类型 | 说明 | 产出 |
|------|------|------|
| `Damage` | 伤害效果 | 扣减HP |
| `Heal` | 治疗效果 | 恢复HP |
| `ApplyBuff` | 施加Buff | 添加Tag + Modifier |
| `Dispel` | 驱散效果 | 移除Tag + Modifier |
| `Displacement` | 位移效果 | 改变位置 |
| `ApplyShield` | 施加护盾 | 添加护盾实例 |
| `Summon` | 召唤效果 | 创建召唤物实体 |
| `Kill` | 死亡效果 | 标记死亡 |

#### Buff生命周期

```
Apply（施加）→ Tick（周期触发）→ Expire（到期）→ Remove（移除）
```

| 阶段 | 触发时机 | 行为 |
|------|----------|------|
| Apply | 效果施加时 | 添加到目标，触发入场效果，刷新持续时间/层数 |
| Tick | 按回合/按行动 | 周期效果触发（如中毒每回合掉血） |
| Expire | 持续时间归零 | 触发到期效果 |
| Remove | 被驱散/覆盖 | 执行清理逻辑 |

#### 各类Effect详细数据

**Damage Effect**

| 字段名 | 类型 | 说明 |
|--------|------|------|
| `damage_type` | DamageTypeTag | 物理/魔法/穿透/真实 |
| `skill_multiplier` | f32 | 技能倍率 |
| `can_crit` | bool | 是否可暴击 |
| `is_multi_hit` | bool | 是否多段伤害 |
| `hit_count` | Option<u32> | 多段伤害段数 |

**Heal Effect**

| 字段名 | 类型 | 说明 |
|--------|------|------|
| `heal_base` | Enum | atk_based/fixed |
| `heal_multiplier` | f32 | 治疗倍率 |
| `can_crit` | bool | 是否可暴击 |
| `aoe_decay` | Option<f32> | AOE治疗递减率 |
| `overheal_to_shield` | Option<f32> | 过量治疗转护盾比例 |

**Shield Effect**

| 字段名 | 类型 | 说明 |
|--------|------|------|
| `shield_type` | Enum | physical/magical/universal |
| `shield_value` | f32 | 护盾吸收量 |
| `is_regen_shield` | bool | 是否每回合回复 |
| `regen_value` | Option<f32> | 每回合回复值 |
| `damage_type_filter` | Option<Vec<DamageTypeTag>> | 只吸收特定伤害类型 |

**Dispel Effect**

| 字段名 | 类型 | 说明 |
|--------|------|------|
| `dispel_type` | Enum | buff/debuff/all |
| `dispel_count` | u32 | 驱散数量 |
| `dispel_priority` | Enum | newest/oldest/strongest |

**Displacement Effect**

| 字段名 | 类型 | 说明 |
|--------|------|------|
| `displacement_type` | Enum | active/forced |
| `direction` | Option<Direction> | 强制位移方向 |
| `distance` | u32 | 位移格数 |
| `can_cross_obstacle` | bool | 是否可穿越障碍 |
| `wall_damage_pct` | Option<f32> | 撞墙伤害（最大生命值%） |
| `can_push_units` | bool | 是否推开路径上单位 |

**Summon Effect**

| 字段名 | 类型 | 说明 |
|--------|------|------|
| `summon_template` | CharacterId | 召唤物模板 |
| `inherit_ratio` | f32 | 属性继承比例 |
| `max_count` | u32 | 最大召唤数量 |
| `duration` | Option<u32> | 召唤持续回合 |

#### 死亡结算链路

```
伤害结算完成 → 血量≤0判定
  ↓
濒死窗口（回血/免死/假死）
  ↓ 血量仍≤0
「即将死亡」事件（最后一次触发被动）
  ↓
清除可驱散Buff/Debuff（保留机制Tag）
  ↓
「单位死亡」事件（击杀者触发击杀被动）
  ↓
标记死亡，移除实体控制权
```

#### 状态三大分类

| 分类 | 颜色标识 | 可驱散 |
|------|----------|--------|
| 增益 | 橙色 | 是 |
| 减益 | 红色/紫色 | 是 |
| 特殊 | 蓝色/灰色 | 否 |

#### Schema草案

```yaml
# effect_config.ron
(
  effects: [
    (id: "phys_damage", effect_type: Damage,
     damage: (damage_type: "dmg_physical", can_crit: true)),
    (id: "poison", name_key: "buff.b_001.name", desc_key: "buff.b_001.desc", effect_type: ApplyBuff,
     duration: (duration_type: Turns, value: 3, tick_timing: ActionEnd),
     stacking: "stack_independent",
     max_stack: 9,
     tick_effect: "poison_tick"),
    (id: "knockback_2", effect_type: Displacement,
     displacement: (displacement_type: Forced, distance: 2,
                    can_cross_obstacle: false, wall_damage_pct: 0.1)),
  ],
)
```
