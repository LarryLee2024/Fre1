---
id: 01-architecture.skill-buff-abstraction
title: SRPG Lite-GAS 统一数据驱动抽象模型
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-15
tags:
  - architecture
  - gas
  - skill-buff-effect
---

# SRPG Lite-GAS 统一数据驱动抽象模型

Version: 3.0
Status: Proposed
Source: `docs/其他/27技能buf抽象.md` + `docs/其他/76.md`（SRPG-GAS 架构裁切）+ `docs/其他/77.md`（SRPG Lite-GAS 冻结版）

本文档定义 SRPG 项目的统一数据驱动抽象模型，覆盖完整的 GAS（Gameplay Ability System）链路。
核心目标：500 技能 + 1000 Buff 收敛为 20~30 个 Execution Executor，新增内容只改配置不改代码。

> v3.0 变更：对齐 SRPG Lite-GAS 冻结版架构（`docs/其他/77.md`），新增 Execution 层和 Cue 层，
> Buff 不再是独立顶层模块（Effect + Duration 替代），属性系统升级为 Primary/Derived 一级领域。

> 本文档是 `architecture.md`、`content-pipeline.md`、`attribute_modifier_rules.md`、`skill_rules.md` 的胶水层，
> 将完整的 GAS 链路（Ability → Targeting → Effect → Stacking → Execution → Modifier → Attribute → Tag → Cue → Replay）
> 统一到一个概念模型中。

---

# 1. 反模式：一个技能 = 一个函数

很多战棋项目会这样设计：

```rust
fn fireball()
fn heal()
fn poison()
fn rage()
fn shield()
fn stun()
fn freeze()
fn lightning()
```

做到 100 个技能：

```
100 个函数
100 个逻辑
100 个 Bug 来源
```

做到 500 个技能：

```
彻底爆炸
```

根本问题：

- 每个技能是独立的硬编码逻辑，无法复用
- 修复一个 Bug 可能影响另一个技能
- 新增技能必须修改 Rust 代码，违反 Rule/Content 分离
- 无法实现数据驱动配置

《铃兰之剑》《火纹》《梦幻模拟战》《FFTA》这类 SRPG 根本不是这样实现的。

---

# 1.1 GAS 概念映射：UE 验证过的 AAA 标准模式

> **优化来源**：`docs/其他/74借鉴.md` §1 — UE Gameplay Ability System 借鉴

本项目的 Skill/Buff/Effect 抽象并非凭空设计，而是与 UE 的 **Gameplay Ability System（GAS）** 完全对应。GAS 是 UE 在 AAA 级项目中经过大量验证的战斗子系统架构，我们的设计已覆盖其核心概念。

### GAS ↔ 本项目 概念映射表（10 模块 + 3 基建）

> v3.0 更新：对齐 `docs/其他/77.md` SRPG Lite-GAS 冻结版，从 8 概念扩展为 10 模块 + 3 基建。

| UE GAS 概念 | 本项目概念 | 说明 |
|-------------|-----------|------|
| **GameplayAbility** | **Ability**（技能） | 一次能力释放的完整定义：选择目标 → 消耗资源 → 执行效果 |
| **GameplayEffect** | **Effect**（效果） | 通用效果总层（v3.0 合并原 Buff），含 Duration 变体区分瞬时/回合/永久 |
| — | **Stacking**（堆叠）🆕 | 效果堆叠策略中心：Replace / RefreshDuration / StackAdd / StackMax |
| — | **Execution**（执行算式）🆕 | trait-based 公式执行层：DamageExecution / HealExecution / ShieldExecution |
| **GameplayTag** | **Tag**（标签） | 分类与匹配系统：元素类型、伤害类型、状态标记 |
| **Attribute** | **Attribute**（属性） | Primary（基础）/ Derived（派生）双层属性体系 |
| **AttributeSet** | **Attribute**（属性） | 集中管理的属性容器：由 Modifier 统一计算 Derived 属性 |
| **Modifier** | **Modifier**（修饰器） | 数值修饰链：暴击倍率、元素克制、地形加成 |
| **GameplayCue** | **Cue**（表现信号）🆕 | 统一表现事件总线：业务逻辑只发 Cue，表现层独立订阅响应 |
| **AbilitySystemComponent** | **EffectHandlerRegistry**（Resource） | 全局效果处理器注册表 |
| — | **Registry**（基建）🆕 | 资产统一注册中心：Ability/Effect/Execution/Tag 全局注册 |
| — | **Pipeline**（基建）🆕 | 回合战斗执行管线：Turn System 驱动 |
| — | **Replay**（基建）🆕 | 确定性战斗回放基建：指令 + 种子快照 |

### 核心设计原则：Skill 只负责"我要做什么"，Effect 负责"真正执行"

这是 GAS 最重要的设计洞察，也是本项目的铁律：

```
Ability = 意图声明（我要做什么）
    ↓
Effect = 时长定义（瞬时/回合持续/永久常驻）
    ↓
Execution = 公式执行（怎么算数值）
    ↓
Modifier = 属性修改（怎么改属性）
```

示例：

| 技能 | Ability 声明（意图） | Effect[] 真正执行 |
|------|---------------------|------------------|
| 火球术 | `Ability { selector: EnemySingle, effects: [Damage(120)] }` | DamageEffect → DamageExecution 计算 → Modifier 修饰 → 扣 HP → Cue 飘字 |
| 治疗术 | `Ability { selector: AllySingle, effects: [Heal(100)] }` | HealEffect → HealExecution 计算 → Modifier 修饰 → 加 HP → Cue 飘字 |
| 中毒箭 | `Ability { selector: EnemySingle, effects: [Damage(80), ApplyModifier(Poison)] }` | DamageEffect + PoisonEffect(Duration::TurnLimited(3)) → 两步执行 |

**禁止**在 Skill 里写执行逻辑：

```rust
// 🟥 禁止：cast_fireball() 里面 1000 行
fn cast_fireball() {
    let damage = calculate_damage(...);  // 不应该在这里
    target.hp -= damage;                 // 不应该在这里
    spawn_vfx(...);                     // 不应该在这里
}
```

### GAS 参考说明

本项目的 **Effect Pipeline** 与 UE GAS 的核心流程完全对应，但裁剪了网络相关复杂度：

| UE GAS 流程 | 本项目流程 | 说明 |
|-------------|-----------|------|
| `ApplyGameplayEffect` | **Generate** | 生成初始效果数据（基础值计算） |
| `ExecutionCalculation` | **Execution** 🆕 | trait-based 公式执行（伤害/治疗/护盾公式） |
| `Modifier` | **Modify** | 修饰器链调整数值（暴击/克制/地形） |
| `Execute` | **Execute** | 最终执行效果（修改 World 状态） |
| `GameplayCue` | **Cue** 🆕 | 表现层事件总线（飘字/动画/音效） |

> 本项目将 UE GAS 的单体架构拆分为 10 个正交子系统 + 3 个基础设施，更适合 Bevy ECS 的组件化思维。
> 这是对 UE GAS 的**演进而非降级**——保留核心思想，裁剪网络复杂度，适配单机回合制 SRPG。

> **v3.0 架构冻结**：10 模块（Attribute / Tag / Modifier / Stacking / Execution / Effect / Targeting / Ability / Trigger / Cue）+ 3 基建（Registry / Pipeline / Replay），详见 `docs/其他/77.md`。

---

# 1.2 完整 GAS 链路（v3.0 冻结版）

> **来源**：`docs/其他/77.md` §十 — SRPG Lite-GAS 100 分方案

本项目的完整执行链路是**单向闭环、无反向依赖、无循环调用**的：

```
Ability（技能定义 + 施法校验）
    ↓
Targeting（纯函数目标筛选、无副作用）
    ↓
Effect（时效定义：瞬时 / 回合持续 / 永久常驻）
    ↓
Stacking（堆叠策略匹配：覆写 / 刷新 / 叠加 / 上限）
    ↓
Execution（公式执行：伤害 / 治疗 / 百分比 / 自定义算式）
    ↓
Modifier（属性修改单元批量挂载）
    ↓
Attribute（基础 → 派生属性批量刷新计算）
    ↓
Tag（标签增减、状态判定、互斥拦截）
    ↓
Cue（下发表现事件、逻辑与表现彻底解耦）
    ↓
Replay（指令 + 种子快照持久化）
```

### 链路设计规约

| 规约 | 说明 |
|------|------|
| **单向依赖** | 上层可调用下层，禁止反向依赖 |
| **无副作用** | Targeting 是纯函数，Execution 是纯计算 |
| **确定性** | 同输入 + 同种子 = 同结果，Replay 原生适配 |
| **可测试** | 每层可独立单元测试，无需模拟整条链路 |

### Buff 吸收说明（v3.0 核心变更）

> **v3.0 核心决策**：删除独立的顶层 Buff 模块。Buff 本质上是 **Effect + Duration** 的组合。

| 传统模型 | v3.0 模型 | 说明 |
|---------|----------|------|
| Buff 作为独立顶层模块 | Effect + Duration::TurnLimited | 两者归一，减少跨模块依赖 |
| Equipment 效果 | Effect + Duration::Permanent | 装备 = 永久 Effect |
| 瞬间伤害 | Effect + Duration::Instant | 瞬时效果 = 即时 Effect |
| 持续治疗（HoT） | Effect + Duration::TurnLimited | HoT = 回合持续 Effect |

```
// 火球术 = 瞬时 Effect
Effect { kind: Damage(120), duration: Duration::Instant }

// 中毒 = 回合持续 Effect（原 Buff）
Effect { kind: ApplyModifier(Poison), duration: Duration::TurnLimited(3) }

// 装备属性加成 = 永久 Effect（原 Equipment 效果）
Effect { kind: ApplyModifier(AttackBoost), duration: Duration::Permanent }
```

---

# 2. 抽象模型

## 2.1 第一层：技能 ≠ 逻辑，技能 = 配置

```
技能
≠
逻辑

技能
=
配置
```

火球术、治疗术、猛击、中毒箭——在程序眼里根本不存在。
程序只认识 `Effect`。

## 2.2 第二层：Skill = Selector + Effect[]

```
Skill {
    selector,       // 目标选择器（对谁放）
    requirements[], // 释放前提（能不能放）
    costs[],        // 消耗（消耗什么）
    effects[],      // 效果列表（放什么）
    tags,           // 标签（分类）
}
```

示例：

| 技能 | Selector | Effect[] |
|------|----------|----------|
| 火球术 | EnemySingle | [Damage(120)] |
| 治疗术 | AllySingle | [Heal(100)] |
| 中毒箭 | EnemySingle | [Damage(80), ApplyBuff(Poison)] |
| 火焰风暴 | EnemyAOE十字 | [Damage(80)] |
| 处决 | EnemySingle | [ConditionalEffect(HpBelow(30%), Execute)] |
| 背刺 | EnemySingle | [ConditionalEffect(BehindTarget, Damage(+50%))] |

程序只执行：

```rust
for effect in skill.effects {
    execute(effect);
}
```

## 2.3 第三层：Effect 才是核心

真正需要分类的是 `Effect`，而不是 Skill。

### 完整 Effect 分类（20 种原子能力）

| # | Effect | 说明 | 子类型/参数 |
|---|--------|------|------------|
| 1 | **Damage** | 造成伤害 | 物理、魔法、真实、百分比、反伤 |
| 2 | **Heal** | 恢复生命 | 固定、百分比、持续恢复 |
| 3 | **Shield** | 护盾 | 吸收量、持续时间 |
| 4 | **ModifyResource** | 资源变化 | MP、TP、怒气、行动点 |
| 5 | **ModifyAttribute** | 属性修改 | 攻击、防御、速度、暴击、命中、闪避 |
| 6 | **ApplyBuff** | 施加 Buff | buff_id、持续时间 |
| 7 | **ApplyDebuff** | 施加 Debuff | 与 ApplyBuff 统一（Buff 的 tags 区分增/减） |
| 8 | **Dispel** | 驱散 | 驱散增益、驱散减益、驱散全部 |
| 9 | **Purify** | 净化 | 移除控制效果 |
| 10 | **Revive** | 复活 | 恢复 HP 比例、站起位置 |
| 11 | **Summon** | 召唤 | 召唤物模板、持续时间 |
| 12 | **Teleport** | 位移 | 目标坐标、限制条件 |
| 13 | **Push** | 击退 | 方向、格数、碰撞伤害 |
| 14 | **Pull** | 拉拽 | 方向、格数 |
| 15 | **SwapPosition** | 交换位置 | 两个目标 |
| 16 | **Transform** | 变身 | 变身模板、持续时间 |
| 17 | **SpawnEntity** | 生成实体 | 陷阱、地雷、图腾、召唤物 |
| 18 | **RemoveEntity** | 删除实体 | 移除条件 |
| 19 | **TriggerSkill** | 触发技能 | 连击、追击、反击、协同攻击 |
| 20 | **Execute** | 斩杀 | HP 阈值、即死 |

> 当前代码库实现了 4 种（Damage、Heal、ApplyBuff、Cleanse 作为 EffectDef 变体）。
> 其余 16 种是设计目标，按需逐步实现。

> ⚠️ **§1.1.7 过度设计警告**：20 种 Effect 类型为设计目标，当前仅实现 4 种。**新增 Effect 类型必须基于当前明确需求**，禁止为未来可能出现的玩法提前实现完整 Effect 类型集。每新增一个 Effect 类型，必须证明当前有对应的技能/Buff 需求。

> **v3.0 变更**：Effect 分类表中的 `ApplyBuff` / `ApplyDebuff` 在 v3.0 中统一为 `ApplyModifier` + Duration 变体。原独立的 Buff 领域已被吸收，见 §2.4。

## 2.4 第四层：Buff 吸收为 Effect + Duration（v3.0）

> **v3.0 核心变更**：Buff 不再是独立顶层模块，而是 **Effect + Duration::TurnLimited** 的组合。详见 `docs/其他/77.md` §二。

传统模型中 Buff 是独立实体，v3.0 中 Buff 的能力被完全吸收到 Effect 体系：

```
// v2.0（旧模型）：Buff 是独立顶层模块
Buff {
    triggers[],      // 触发时机
    duration,        // 持续策略
    stack_policy,    // 叠层规则
    conditions[],    // 生效条件
    effects[],       // 效果列表
    tags,            // 标签
}

// v3.0（新模型）：Buff = Effect + Duration::TurnLimited
Effect {
    kind: ApplyModifier(modifier_id),  // 效果内容
    duration: Duration::TurnLimited(3), // 持续 3 回合 = 原 Buff
    stacking: StackingRule::StackMax(5), // 叠层规则
    triggers: [TurnStart],             // 触发时机
    conditions: [],                     // 生效条件
    tags: [Status.Poison],             // 标签
}
```

### Duration 变体决定效果性质

| Duration 变体 | 对应概念 | 说明 | 示例 |
|---------------|---------|------|------|
| `Duration::Instant` | 瞬时效果 | 执行后立即消失 | 伤害、击退、即时治疗 |
| `Duration::TurnLimited(u32)` | 回合 Buff | 持续 N 回合后过期 | 中毒、眩晕、攻击增减 |
| `Duration::UntilDeath` | 标记效果 | 直到死亡才消失 | 诅咒、标记 |
| `Duration::Permanent` | 永久效果 | 被动光环、装备属性 | 职业天赋、装备加成 |
| `Duration::BattleEnd` | 战斗效果 | 战斗结束消失 | 增益、场地效果 |

### 经典 Buff 示例（v3.0 表达）

| Buff 概念 | Effect kind | Duration | Trigger | Stacking |
|-----------|------------|----------|---------|----------|
| 中毒 | ApplyModifier(Poison) | TurnLimited(3) | TurnStart | StackMax(5) |
| 再生 | ApplyModifier(Regen) | TurnLimited(3) | TurnStart | RefreshDuration |
| 狂怒 | ApplyModifier(Rage) | TurnLimited(2) | AfterDamaged | StackAdd |
| 荆棘 | ApplyModifier(Thorns) | Permanent | AfterDamaged | Replace |
| 吸血 | ApplyModifier(Lifesteal) | Permanent | AfterAttack | RefreshDuration |
| Boss 光环 | ApplyModifier(BossAura) | Permanent | — | Replace |

> 本质完全一样，只是 Duration 和 Stacking 参数不同。v3.0 中不存在独立的"Buff 概念"，所有 Buff 效果统一表达为 Effect + Duration + Stacking。

### Trigger 枚举（11 种触发时机）

```rust
enum Trigger {
    TurnStart,       // 回合开始
    TurnEnd,         // 回合结束
    BeforeAttack,    // 攻击前
    AfterAttack,     // 攻击后
    BeforeDamaged,   // 受伤前
    AfterDamaged,    // 受伤后
    BeforeMove,      // 移动前
    AfterMove,       // 移动后
    KillTarget,      // 击杀目标时
    Death,           // 死亡时
    BattleStart,     // 战斗开始
    BattleEnd,       // 战斗结束
}
```

---

# 3. 执行管线

## 3.1 完整执行链路

> **优化来源**: `docs/其他/74借鉴.md` §18 — ActionQueue 作为 Effect Pipeline 的执行容器

```
Skill / Buff
    ↓
Effect[] 生成
    ↓
ActionQueue 入队（顺序链式执行）
    ↓
Effect Pipeline: Generate → Modify → Execute
    ↓
EffectHandler trait 分发
    ↓
游戏状态变更
```

### ActionQueue：效果执行的顺序容器

ActionQueue 确保效果按顺序链式执行：**技能释放 → 伤害 → Buff → 死亡 → 反击**。不是 Parallel Events，是 Sequential Action Queue。

> ActionQueue 的详细设计见 `docs/01-architecture/command_bus_design.md` §ActionQueue。

## 3.2 Effect Pipeline 三步管线

> 详见 `docs/02-domain/attribute_modifier_rules.md` 效果管线章节。

### Step 1：Generate（生成效果）

输入：EffectDef + GenerateContext（攻击者/目标属性、地形、标签）
处理：通过 EffectHandlerRegistry 查找处理器，调用 handler.generate() 计算初始值
输出：PendingEffectData（含 amount、source_tags、terrain_id）
禁止：在 Generate 阶段修改目标属性

### Step 2：Modify（修饰效果）

输入：PendingEffect + ModifierRuleRegistry（修饰规则）
处理：遍历规则，标签匹配后通过 Calculator 计算修饰，记录 ModifierEntry
输出：修改后的 PendingEffectData（amount 已更新，modifiers 已填充）
禁止：跳过标签匹配、不记录 ModifierEntry

### Step 3：Execute（执行效果）

输入：PendingEffect + ExecuteContext（World 访问）
处理：通过 EffectHandlerRegistry 查找处理器，调用 handler.execute() 执行
输出：EffectResult（target_died 状态）+ PendingMessage（DamageApplied / HealApplied）
禁止：在 Execute 阶段重新应用修饰规则

## 3.3 EffectHandler trait 分发

> 详见 `docs/02-domain/attribute_modifier_rules.md` 效果处理器章节。

> **优化来源**：`docs/其他/67.md` — Effect Pipeline 三步与 Bevy Message 映射表 + Bevy ECS 映射

每种 Effect 类型实现一个 EffectHandler：

```
DamageHandler   → generate_damage / execute_damage
HealHandler     → generate_heal / execute_heal
BuffHandler     → generate_apply_buff / execute_apply_buff
CleanseHandler  → generate_cleanse / execute_cleanse
```

新增效果类型只需：
1. 实现 EffectHandler trait（type_name / generate / preview / execute）
2. 注册到 EffectHandlerRegistry
3. 添加对应的 EffectDef 变体

禁止修改管线调度代码。

### Effect Pipeline 三步与 Bevy Message 的映射

| Pipeline 步骤 | Bevy 机制 | 数据流 | 说明 |
|--------------|-----------|--------|------|
| **Generate** | 纯函数计算 | 输入：EffectDef + GenerateContext → 输出：PendingEffectData | 不涉及 ECS 状态变更，可并行 |
| **Modify** | 纯函数计算 | 输入：PendingEffect + ModifierRuleRegistry → 输出：修改后的 PendingEffectData | 不涉及 ECS 状态变更，可并行 |
| **Execute** | Message/Commands | 输入：PendingEffect + World → 输出：EffectResult + PendingMessage | 涉及 ECS 状态变更，通过 Message 通知其他系统 |

Execute 阶段产出的 Message 类型：

| Message 类型 | 消费方 | 用途 |
|-------------|--------|------|
| `DamageApplied` | BattleRecord、UI 飘字、回放录制 | 伤害已应用 |
| `HealApplied` | BattleRecord、UI 飘字 | 治疗已应用 |
| `BuffApplied` | BattleRecord、UI 图标 | Buff 已施加 |
| `EntityDied` | TurnSet、VictoryCheck | 实体死亡 |
| `EffectCompleted` | EffectPipeline（下一步） | 效果执行完成 |

### Bevy ECS 映射：抽象模型 → Bevy 原语

| 抽象概念 | Bevy ECS 原语 | 说明 |
|---------|---------------|------|
| **SkillDef** | `Asset` | 技能定义作为资产加载，只读，全局共享 |
| **SkillInstance** | `Component` | 技能运行时实例，附加在实体上，包含冷却、弹药等状态 |
| **BuffDef** | `Asset` | Buff 定义作为资产加载，只读 |
| **BuffInstance** | `Component` | Buff 运行时实例，附加在实体上，包含层数、剩余回合 |
| **EffectHandler** | `trait`（不是 Component） | 效果处理器是纯逻辑 trait，通过 Registry 注册 |
| **EffectHandlerRegistry** | `Resource` | 效果处理器注册表，全局单例 |
| **ModifierRuleRegistry** | `Resource` | 修饰规则注册表，全局单例 |
| **FormulaRegistry** | `Resource` | 公式注册表，全局单例 |
| **Trigger** | `Message` / `Observer` | 触发时机通过 Message/Observer 机制实现 |
| **Selector** | `trait`（封装 Query） | 目标选择器封装 Bevy Query 进行空间查询 |

```rust
// Content 层：作为 Asset 加载，只读，全局共享
#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct SkillDef {
    pub id: String,
    pub selector: SelectorDef,
    pub effects: Vec<EffectDef>,
    pub costs: Vec<CostDef>,
    // ...
}

// Rule/State 层：作为 Component 附加在实体上，包含运行时状态
#[derive(Component)]
pub struct SkillInstance {
    pub def_handle: Handle<SkillDef>, // 引用 Asset
    pub cooldown_remaining: u32,      // 运行时状态
    pub ammo: u32,                    // 运行时状态
}

// Registry 作为 Resource
#[derive(Resource)]
pub struct EffectHandlerRegistry {
    handlers: HashMap<String, Box<dyn EffectHandler>>,
}
```

---

# 4. 十大补充系统

> 以下系统来自 `docs/其他/27技能buf抽象.md` 的第六层到第十五层抽象。
> 很多架构失败，不是因为 Effect 不够，而是因为只抽象了 Skill/Buff/Effect，却没有继续向上抽象。

> **优化来源**：`docs/其他/67.md` — 十个正交子系统解耦边界精确说明

### 十大子系统正交性说明

十大子系统彼此正交，每个子系统解决一个独立维度的问题。正交性意味着：修改任何一个子系统的实现，不会影响其他子系统。以下是关键边界的精确说明：

| 子系统 | 职责 | 不负责 | 与其他子系统的关系 |
|--------|------|--------|-------------------|
| **Condition** | 效果是否生效（执行时判断） | 技能不能放（那是 Requirement） | 与 Requirement 语义分离：Condition 在 Effect 执行时判断，Requirement 在技能释放前判断 |
| **Requirement** | 技能不能放（释放前判断） | 效果是否生效（那是 Condition） | 与 Condition 语义分离：Requirement 失败 → UI 提示"技能不可用"；Condition 失败 → 静默跳过效果 |
| **Selector** | 对谁放（目标选择） | 放什么效果（那是 Effect） | Selector × Effect 的组合决定技能种类 |
| **Modifier** | 效果数值修饰（暴击/克制/地形） | 效果本身（那是 Effect） | Modifier 在 Effect Pipeline 的 Modify 阶段介入 |
| **Cost** | 消耗什么（MP/HP/怒气） | 技能能不能放（那是 Requirement） | Cost 检查通过后才扣除，Requirement 检查在 Cost 之前 |
| **Duration** | Buff 持续多久（N回合/直到死亡/永久） | Buff 如何叠层（那是 StackPolicy） | Duration 控制"何时过期"，StackPolicy 控制"如何叠加"——两者独立 |
| **StackPolicy** | Buff 如何叠层（可叠N层/不可叠/刷新） | Buff 持续多久（那是 Duration） | StackPolicy 控制"叠几层"，Duration 控制"持续多久"——两者独立 |
| **Trigger** | Buff 什么时候触发（TurnStart/AfterDamaged） | 触发后做什么（那是 Effect） | Trigger 是 Buff 的"开关"，决定何时将 Effect 推入 Pipeline |
| **Tag** | 分类标签（Fire/Physical/Melee） | 具体效果逻辑 | Tag 驱动 Modifier 匹配和 Requirement 检查 |
| **Formula** | 数值计算公式（物理伤害/魔法伤害） | 效果执行逻辑 | Formula 被 Effect 在 Generate 阶段调用 |

关键区分：
- **Condition vs Requirement**：Condition 是"效果生效条件"（如"目标 HP<30% 时触发 Execute"），Requirement 是"技能释放前提"（如"需要弓、未沉默"）。Condition 失败时效果静默跳过，Requirement 失败时 UI 提示技能不可用。
- **Duration vs StackPolicy**：Duration 管"何时过期"（3 回合、直到死亡、永久），StackPolicy 管"如何叠加"（不可叠、可叠 5 层、叠满刷新）。两者完全独立，可自由组合。

## 4.1 Condition（条件系统）

> **优化来源**: `docs/其他/74借鉴.md` §16 — Condition 系统统一抽象

条件系统将效果从"无条件执行"升级为"条件触发执行"。

### ConditionDef 配置数据

典型条件场景：**回合开始**、**血量低于50%**、**被攻击后** — 统一为 `ConditionDef` 配置数据：

```
ConditionalEffect {
    condition,    // 触发条件
    effect,       // 效果
}
```

| 条件 | 说明 | 示例 |
|------|------|------|
| HpBelow(percent) | 目标 HP 低于百分比 | 处决：目标 HP < 30% 时触发 Execute |
| TurnStart | 回合开始时触发 | 中毒：每回合开始扣血 |
| AfterDamaged | 被攻击后触发 | 荆棘：被攻击后反弹伤害 |
| BehindTarget | 背击 | 背刺：目标背后时 Damage +50% |
| HasBuff(tag) | 有指定 Buff | 燃烧爆破：目标有燃烧时触发爆炸 |
| NoBuff(tag) | 无指定 Buff | 治愈：目标无 debuff 时治疗翻倍 |
| IsCritical | 暴击时 | 暴击特效 |
| IsKill | 击杀时 | 击杀回复 |
| TerrainIs(tag) | 地形为指定类型 | 草地加成：地形为草地时伤害 +20% |
| AdjacentAlly | 相邻有友军 | 协同防御 |

条件与要求的区别：
- **Condition**：效果是否生效（Effect 生效时的判断）
- **Requirement**：技能能不能放（技能释放前的判断）

> Condition 已在 `docs/02-domain/` 中有详细领域规则定义，此处仅做上层抽象说明。实现细节参见领域文档。

---

## 4.2 Selector（目标选择器）

技能种类 ≈ Selector × Effect。

```
技能种类 ≠ Effect 种类
技能种类 ≈ Selector × Effect
```

| Selector | 说明 | 示例 |
|----------|------|------|
| EnemySingle | 敌方单体 | 火球术 |
| EnemyAOE十字 | 敌方十字范围 | 火焰风暴 |
| EnemyAOE圆形 | 敌方圆形范围 | 流星雨 |
| EnemyAll | 敌方全体 | 毁灭射线 |
| AllySingle | 友方单体 | 治疗术 |
| AllyAOE | 友方范围 | 群体治疗 |
| SelfOnly | 自身 | 狂暴 |
| EmptyTile | 空地 | 传送、放置陷阱 |
| Summon | 召唤位置 | 召唤仆从 |

火球术和火焰风暴逻辑完全一样，只是 Selector 不同。

---

## 4.3 Modifier（修饰器链）

> **优化来源**：`docs/其他/67.md` — Modifier 链剥离原理深化

暴击、弱点、地形加成、职业克制、Buff 加成、天气加成——实际上都不是 Effect，它们是 Modifier。

```
Damage
  ↓
Modifier 链（暴击 → 属性克制 → 地形加成 → 天气加成）
  ↓
Final Damage
```

如果没有 Modifier 层，后面会出现：

```rust
execute_damage()
execute_damage_with_crit()
execute_damage_with_element()
execute_damage_with_backstab()
```

越来越多。

> 已在 `docs/02-domain/attribute_modifier_rules.md` 中详细定义 ModifierRule、ModifierCalculator trait、
> 标签匹配机制。此处明确 Modifier 在上层抽象中的位置：Modifier 是 Effect 执行链路中的修饰环节，
> 不是 Effect 本身。

### 为什么暴击/克制/地形加成是 Modifier 而非 Effect

**核心区别**：Effect 是"做什么"（造成伤害、恢复生命、施加 Buff），Modifier 是"怎么调整数值"（伤害暴击倍率、元素克制系数、地形加成百分比）。

| 维度 | Effect | Modifier |
|------|--------|----------|
| **职责** | 执行具体效果（Damage/Heal/Buff） | 修饰效果数值（乘以系数/加减固定值） |
| **执行时机** | Execute 阶段 | Modify 阶段（在 Execute 之前） |
| **是否修改 World** | 是（修改 HP/添加 Component） | 否（只修改 PendingEffectData 的 amount） |
| **是否可独立触发** | 是（技能直接调用 Effect） | 否（必须附着在某个 Effect 上） |
| **数据来源** | EffectDef 配置 | ModifierRule 配置 + 标签匹配 |

**举例**：

```
火球术造成 120 点火焰伤害：
  Effect = Damage(120, Fire)
  Modifier 链 = [
    CriticalModifier(+50%),      // 暴击：120 × 1.5 = 180
    ElementWeakModifier(+20%),   // 火焰弱点：180 × 1.2 = 216
    TerrainModifier(+10%),       // 草地加成：216 × 1.1 = 237.6
  ]
  Final Damage = 237（取整）
```

如果暴击是 Effect 而非 Modifier，那么每个伤害技能都需要单独实现"暴击版"和"非暴击版"，组合爆炸无法控制。Modifier 的设计让"暴击"成为一个独立的、可复用的修饰规则，自动应用到所有伤害类型上。

### 三种基本修饰类型

> **优化来源**: `docs/其他/74借鉴.md` §17 — Modifier 系统三种基本修饰类型

| 修饰类型 | 语法 | 说明 | 示例 |
|----------|------|------|------|
| **加算** | `+10%` | 在当前值基础上加百分比 | 地形加成 +10% |
| **乘算** | `×1.5` | 在当前值基础上乘倍率 | 暴击倍率 ×1.5 |
| **覆盖** | `override` | 直接替换为固定值 | 某些 Buff 覆盖属性为固定值 |

Modifier 链按类型顺序执行：**加算 → 乘算 → 覆盖**，保证数值计算的确定性。

> ⚠️ **铁律**：不要直接改属性 — 所有属性变动必须通过 Modifier Pipeline。直接修改属性会破坏修饰链的可观测性和回放一致性。

> Modifier 的详细实现见 `docs/02-domain/attribute_modifier_rules.md`。

---

## 4.3.1 Execution（执行算式层）🆕

> **来源**：`docs/其他/77.md` §五 — UE GameplayEffectExecutionCalculation 精简适配

### 为什么需要 Execution 层

Effect 定义"做什么"（造成伤害、恢复生命），Modifier 定义"怎么调整数值"（暴击倍率、元素克制），但**具体怎么算**——是 `Attack - Defense` 还是 `MaxHP * 10%` 还是 `Attack * CritMultiplier`——这个职责属于 Execution。

如果没有 Execution 层：

```rust
// 🟥 禁止：Effect 内部直接写公式，match 分支膨胀
match damage_type {
    Physical => atk - def,
    Magical => matk - mdef,
    TrueDamage => atk,
    CritDamage => atk * crit_multiplier,
    TerrainDamage => max_hp * 0.1,
    // 每新增一种公式，这里就要加一个分支 → 爆炸
}
```

### Execution trait 模式

所有数值计算公式通过 trait 注册，Effect 只负责调用：

```rust
/// Execution trait — 所有效果执行算式的统一接口
pub trait Execution: Send + Sync {
    /// 算式 ID
    fn execution_id(&self) -> ExecutionId;

    /// 执行计算：输入上下文 → 输出最终数值
    fn calculate(&self, ctx: &ExecutionContext) -> i32;
}
```

### 具体 Execution 实现

| Execution | 说明 | 计算逻辑 |
|-----------|------|---------|
| **DamageExecution** | 物理/魔法伤害 | `ATK * (1 + CritBonus) * ElementMultiplier - DEF * Resistance` |
| **HealExecution** | 治疗量计算 | `BaseHeal * (1 + HealBonus)` |
| **ShieldExecution** | 护盾吸收量 | `ShieldValue * (1 + ShieldBonus)` |
| **TrueDamageExecution** | 真实伤害 | `ATK`（无视防御） |
| **TerrainDamageExecution** | 地形伤害 | `MaxHP * Percentage` |
| **PoisonExecution** | 中毒伤害 | `SourceATK * Stacks * Multiplier` |

### Execution 在 GAS 链路中的位置

```
Effect（定义效果类型和参数）
    ↓
Execution（选择对应的 Execution trait 计算数值）
    ↓
Modifier（修饰计算结果：暴击/克制/地形加成）
    ↓
Attribute（更新最终属性值）
```

### ExecutionRegistry（算式注册表）

所有 Execution 统一注册到 ExecutionRegistry，新增算式只需注册不改代码：

```rust
#[derive(Resource)]
pub struct ExecutionRegistry {
    executions: HashMap<ExecutionId, Box<dyn Execution>>,
}
```

> **与 EffectHandler 的区别**：EffectHandler 管理"效果生命周期"（generate → modify → execute 三步），Execution 管理"数值计算公式"（纯函数，只算不改状态）。EffectHandler 在 Execute 阶段调用 Execution 获取最终数值，再通过 Modifier 修饰后应用到 World。

> Execution 的详细实现见 `docs/02-domain/effect/effect-rules.md`。

---

## 4.4 Cost（消耗系统）

技能天然包含 Cost，不要把 MP 消耗写进技能逻辑。

```rust
Skill {
    costs,        // 统一消耗列表
    selector,
    effects,
}
```

| Cost 类型 | 说明 | 示例 |
|-----------|------|------|
| MpCost | MP 消耗 | 大部分魔法技能 |
| HpCost | HP 消耗 | 自损技能 |
| RageCost | 怒气消耗 | 狂暴技能 |
| ActionPointCost | 行动点消耗 | AP 制游戏 |
| AmmoCost | 弹药消耗 | 射击技能 |
| DurabilityCost | 耐久消耗 | 武器耐久 |
| CurrencyCost | 金币消耗 | 商店技能 |
| SacrificeCost | 献祭消耗 | 牺牲队友 |

---

## 4.5 Requirement（释放前提）

> **优化来源**: `docs/其他/74借鉴.md` §15 — Requirement 系统统一抽象

典型前提场景：**需要MP**、**需要目标**、**需要武器** — 统一为 `RequirementDef` 配置数据。

Requirement 与 Condition 不同：

| | Requirement | Condition |
|---|-------------|-----------|
| **判断时机** | 技能释放前 | 效果执行时 |
| **判断对象** | 技能能不能放 | 效果是否生效 |
| **失败结果** | 技能不可用（UI 提示） | 效果不触发（静默跳过） |
| **示例** | 需要弓、未沉默、目标存在 | HP<30%、背击、有 Buff |

| Requirement | 说明 |
|-------------|------|
| HasWeapon(tag) | 需要指定武器类型 |
| NotSilenced | 未被沉默 |
| TargetExists | 目标存在 |
| MpAbove(threshold) | MP 大于阈值 |
| HasAmmo | 有弹药 |
| IsStanding | 需要站立（非倒地） |

> Requirement 已在 `docs/02-domain/skill_rules.md` 中有详细领域规则定义，此处仅做上层抽象说明。实现细节参见领域文档。

---

## 4.6 Duration（持续策略）

很多人只做"持续 3 回合"，实际上不够。

```rust
enum DurationPolicy {
    Turns(u32),           // 持续 N 回合
    UntilDeath,           // 直到死亡
    UntilMove,            // 直到移动
    UntilAttack,          // 直到攻击
    UntilDamaged,         // 直到受伤
    BattleEnd,            // 持续到战斗结束
    Permanent,            // 永久
}
```

| 策略 | 说明 | 典型用例 |
|------|------|----------|
| Turns(3) | 持续 3 回合 | 中毒、护盾 |
| UntilDeath | 直到死亡才消失 | 标记、诅咒 |
| UntilMove | 移动后消失 | 蓄力、坚守 |
| UntilAttack | 攻击后消失 | 蓄力、反击准备 |
| UntilDamaged | 受伤后消失 | 护盾、隐身 |
| BattleEnd | 战斗结束消失 | 增益、场地效果 |
| Permanent | 永久（直到手动移除） | 被动光环 |

---

## 4.7 StackPolicy（叠层系统）

叠层是后期 Buff 爆炸的来源，必须单独抽象。

```rust
enum StackPolicy {
    NoStack,              // 不可叠，重复施加刷新 duration
    Stackable(u32),       // 可叠 N 层，达到上限后刷新 duration
    StackableNoRefresh(u32), // 可叠 N 层，达到上限后无效
}
```

| 策略 | 说明 | 典型用例 |
|------|------|----------|
| NoStack | 不可叠，刷新 duration | 易伤、护盾 |
| Stackable(5) | 可叠 5 层 | 中毒、流血 |
| Stackable(10) | 可叠 10 层 | 连击标记 |
| StackableNoRefresh(3) | 可叠 3 层，满了不再叠 | 狂怒层数 |

叠层与 Buff 定义分离：

```
Buff {
    triggers,
    duration,
    stack_policy,  // ← 独立系统
    effects,
}
```

---

## 4.8 Trigger（触发器）

Trigger 是 Buff 的核心。详见 2.4 节 Trigger 枚举。

额外需要的是 **TriggerContext**（触发上下文）：

```rust
struct TriggerContext {
    trigger: Trigger,
    source_entity: Entity,      // 谁触发的
    target_entity: Entity,      // 触发目标
    damage_dealt: Option<i32>,  // 造成多少伤害（AfterAttack 需要）
    is_critical: Option<bool>,  // 是否暴击
    // ...其他上下文数据
}
```

例如吸血 Buff：触发时机为 AfterAttack，需要 `damage_dealt` 上下文才能计算回复量。
否则实现不了。

---

## 4.8.1 Stack（执行栈）— MTG 风格的嵌套/中断/取消机制

> **优化来源**：`docs/其他/74借鉴.md` §19 — 卡牌游戏 Stack 系统借鉴（Magic: The Gathering）

### Stack 与 Action Queue 的区别

Action Queue 是**有序队列**（FIFO），按顺序逐条执行，不支持中断。Stack 是**响应栈**（LIFO），支持嵌套、中断、取消，类似 MTG 的堆叠响应机制。

| 维度 | Action Queue | Stack（执行栈） |
|------|-------------|----------------|
| **数据结构** | FIFO 队列 | LIFO 栈 |
| **执行顺序** | 严格按入队顺序 | 后进先出，支持响应嵌套 |
| **中断能力** | 不支持 | 支持（新事件可打断当前事件） |
| **取消能力** | 不支持 | 支持（栈顶事件可被取消） |
| **典型场景** | 技能 → 伤害 → Buff → 死亡（线性流程） | 死亡触发 → Buff触发 → 反击触发（嵌套响应） |
| **复杂度** | 低 | 中等，需严格控制弹出顺序 |

### Stack 使用场景

```
场景：单位 A 攻击单位 B

1. OnAttack 触发 → 压入 Stack
2. 造成伤害 → 压入 Stack
3. B 死亡触发 OnDeath → 压入 Stack（不立即执行）
4. B 的荆棘 Buff 触发 AfterDamaged → 压入 Stack
5. A 的吸血 Buff 触发 AfterAttack → 压入 Stack

Stack 解析（LIFO）：
  ├─ A 的吸血触发（最先执行）
  ├─ B 的荆棘反击
  ├─ B 的死亡处理
  ├─ 伤害结算
  └─ 攻击完成（最后执行）
```

### Bevy 0.18+ Stack 实现模式

```rust
/// 执行栈 — 处理嵌套触发和中断的 LIFO 结构
#[derive(Resource)]
pub struct ExecutionStack {
    entries: Vec<StackEntry>,
}

#[derive(Debug, Clone)]
pub struct StackEntry {
    /// 触发源：哪个事件压入的
    pub trigger: Trigger,
    /// 触发上下文
    pub context: TriggerContext,
    /// 优先级（数值越高越先弹出）
    pub priority: i32,
    /// 是否可被取消
    pub cancellable: bool,
}

impl ExecutionStack {
    /// 压入新事件到栈顶
    pub fn push(&mut self, entry: StackEntry) {
        self.entries.push(entry);
    }
    
    /// 弹出栈顶事件并执行
    pub fn pop_and_resolve(&mut self) -> Option<StackEntry> {
        self.entries.pop()
    }
    
    /// 取消栈顶事件（响应方可取消触发）
    pub fn cancel_top(&mut self) -> bool {
        if let Some(top) = self.entries.last() {
            if top.cancellable {
                self.entries.pop();
                return true;
            }
        }
        false
    }
    
    /// 获取栈深度（防止无限递归触发）
    pub fn depth(&self) -> usize {
        self.entries.len()
    }
}

/// 栈深度上限 — 防止无限递归触发导致栈溢出
pub const MAX_STACK_DEPTH: usize = 32;
```

### Stack 与 Effect Pipeline 的协作

```
触发事件（如 OnDeath）
    ↓
压入 ExecutionStack
    ↓
Stack 弹出 → 检查 Condition
    ↓
├── Condition 通过 → 进入 Effect Pipeline（Generate → Modify → Execute）
├── Condition 失败 → 跳过，弹出下一个
└── 栈深度 > MAX_STACK_DEPTH → 强制弹出 + WARN 日志
```

> **关键约束**：Stack 是触发调度层，不是 Effect 执行层。Stack 负责决定"什么时候执行"，Effect Pipeline 负责"怎么执行"。Stack 弹出的事件最终仍进入 Effect Pipeline。

> 交叉引用：`docs/02-domain/trigger_rules.md`（触发规则）、`docs/02-domain/stack_policy_rules.md`（Stack 解析策略）

---

## 4.8.2 Trigger 系统 — 统一注册与分发

> **优化来源**：`docs/其他/74借鉴.md` §20 — 卡牌游戏 Trigger 系统（OnAttack/OnDamage/OnDeath/OnTurnStart 统一注册）

### TriggerRegistry（触发器注册表）

Trigger 不应散落在各个 System 中硬编码判断，而是统一注册到 TriggerRegistry，由引擎在事件发生时自动分发。

```rust
/// 触发器注册表 — 所有 Trigger Handler 统一注册
#[derive(Resource)]
pub struct TriggerRegistry {
    handlers: HashMap<Trigger, Vec<Box<dyn TriggerHandler>>>,
}

/// Trigger Handler trait — 每种触发器的处理逻辑
pub trait TriggerHandler: Send + Sync {
    /// 触发器类型
    fn trigger_type(&self) -> Trigger;
    
    /// 处理触发事件，返回要执行的 Effect 列表
    fn handle(&self, ctx: &TriggerContext) -> Vec<EffectDef>;
    
    /// 触发优先级（决定同 Tick 内的执行顺序）
    fn priority(&self) -> i32 { 0 }
}
```

### 统一 Trigger 类型表

| Trigger | 触发时机 | 典型用途 | 优先级 |
|---------|---------|---------|--------|
| `OnAttack` | 攻击发起时 | 吸血、反击准备 | 0 |
| `OnDamage` | 伤害结算后 | 荆棘反伤、护盾消耗 | 10 |
| `OnDeath` | 单位死亡时 | 死亡爆炸、遗言效果 | 20（高优先级） |
| `OnKill` | 击杀目标时 | 击杀回复、连杀奖励 | 10 |
| `OnTurnStart` | 回合开始时 | 中毒/再生结算、蓄力 tick | 0 |
| `OnTurnEnd` | 回合结束时 | 持续效果过期检查 | 0 |
| `OnHeal` | 治疗结算后 | 溢出治疗转化 | 5 |
| `OnBuffApplied` | Buff 施加时 | Buff 叠层触发 | 5 |
| `OnBuffRemoved` | Buff 移除时 | Buff 清除效果 | 5 |
| `OnMove` | 移动完成后 | 地形效果、陷阱触发 | 5 |
| `OnRevive` | 复活时 | 复活效果、入场触发 | 15 |

### Trigger 分发流程

```
游戏事件（如单位受到伤害）
    ↓
EventBus 广播 DamageDealt 消息
    ↓
TriggerDispatcher 系统
    ├── 查询 TriggerRegistry 获取所有 OnDamage Handler
    ├── 按 priority 排序
    ├── 逐个调用 handler.handle(ctx)
    ├── 收集 EffectDef[]
    └── 压入 ExecutionStack（见 4.8.1）
```

### Trigger 与 Stack 的协作

```
事件发生
    ↓
TriggerRegistry 分发 → 生成 EffectDef[]
    ↓
压入 ExecutionStack
    ↓
Stack LIFO 解析 → 每个 Entry 进入 Effect Pipeline
    ↓
执行完毕 → 检查是否有新触发 → 可能压入新 Entry
    ↓
栈空 → 本轮触发链结束
```

> **关键约束**：TriggerHandler 只返回 `Vec<EffectDef>`，不直接修改 World 状态。所有状态变更必须通过 Effect Pipeline。这保证了触发链的可预测性和可审计性。

> 交叉引用：`docs/02-domain/trigger_rules.md`（Trigger 完整定义）、`docs/02-domain/stack_policy_rules.md`（Stack 解析策略）

---

## 4.8.3 Cue（表现信号总线）🆕

> **来源**：`docs/其他/77.md` §六 — UE GameplayCue 精简适配

### 为什么需要 Cue 层

业务逻辑（伤害计算、Buff 施加、死亡处理）不应该直接调用任何 UI/特效/音效代码。Cue 层是**统一的表现事件总线**，业务逻辑只发 Cue 事件，表现层独立订阅响应。

```
// 🟥 禁止：业务逻辑直接操作表现
fn execute_damage() {
    target.hp -= damage;
    spawn_floating_text(damage);    // 不应该在这里
    play_hit_sound();               // 不应该在这里
    shake_camera();                 // 不应该在这里
}

// 🟩 正确：业务逻辑只发 Cue
fn execute_damage() {
    target.hp -= damage;
    cue_sender.send(CueEvent::DamageApplied { target, amount: damage });
    // 表现层独立响应
}
```

### CueEvent 类型

| CueEvent | 说明 | 表现层响应 |
|----------|------|-----------|
| `DamageApplied` | 伤害已应用 | 飘字 + 受击特效 + 音效 |
| `HealApplied` | 治疗已应用 | 绿色飘字 + 治疗特效 |
| `BuffApplied` | 效果已施加 | Buff 图标 + 施加特效 |
| `BuffRemoved` | 效果已移除 | Buff 图标消失 |
| `Death` | 单位死亡 | 死亡动画 + 音效 |
| `Revive` | 单位复活 | 复活特效 |
| `CriticalHit` | 暴击触发 | 暴击特效 + 特殊音效 |
| `ElementWeak` | 元素弱点 | 元素特效 |
| `ShieldBreak` | 护盾破碎 | 破碎特效 |
| `TurnStart` | 回合开始 | 回合切换 UI |
| `LevelUp` | 升级 | 升级特效 + UI |

### Cue 与 Message 的关系

Cue 本质上是 Message 的一个子集，专门服务于表现层：

```
DomainEvent（通用领域事件）
    ├── DamageApplied → 既发 Message（跨模块通信）又发 Cue（表现响应）
    ├── HealApplied → 同上
    └── CharacterDied → 同上

CueEvent（表现专用事件）
    ├── DamageApplied（飘字/特效）
    ├── CriticalHit（暴击特效）
    └── ...（纯表现，不影响业务逻辑）
```

### Bevy ECS 映射

```rust
/// Cue 发送器 — 业务层通过此发送表现事件
#[derive(Resource)]
pub struct CueSender {
    sender: Sender<CueEvent>,
}

/// Cue 订阅器 — 表现层通过此监听表现事件
#[derive(Resource)]
pub struct CueReceiver {
    receiver: Receiver<CueEvent>,
}
```

### Cue 设计规约

| 规约 | 说明 |
|------|------|
| **单向数据** | Cue 只携带纯数据（target_id、amount、element），不携带资源路径 |
| **无副作用** | 发送 Cue 不修改任何业务状态 |
| **零依赖** | Cue 模块不依赖任何 UI/特效/音效模块 |
| **可丢弃** | 表现层未订阅时，Cue 事件静默丢弃，不影响业务逻辑 |

> **与 §3.3 PendingMessage 的关系**：PendingMessage 是 Effect Pipeline 内部的消息传递机制，Cue 是面向表现层的事件总线。Effect Execute 阶段产生的 PendingMessage 可以同时触发 CueEvent，但两者职责不同：PendingMessage 用于跨模块业务通信，CueEvent 用于表现层订阅。

> Cue 的详细实现见 `docs/02-domain/effect/cue_rules.md`（待建）。

---

## 4.9 Tag（标签系统）

标签是大型项目神器。

> 已在 `docs/02-domain/shared_layer_rules.md` 中定义 GameplayTag 位掩码实现。
> 此处说明标签在 Skill/Buff 抽象中的驱动作用。

| 标签 | 说明 |
|------|------|
| Fire / Ice / Lightning | 元素类型 |
| Physical / Magical / True | 伤害类型 |
| Melee / Ranged | 攻击距离 |
| Holy / Dark | 神圣/暗黑 |
| Poison / Bleed | 持续伤害类型 |

标签驱动的交互：

```
火伤 +20%          → ModifierRule(source_tag=Fire, target_tag=FireWeakness)
所有 Fire 技能 CD-1 → 标签查询 + 冷却修改
免疫物理伤害        → GameplayTag 查询 + 伤害类型判定
```

禁止硬编码：

```rust
if skill_id == FIREBALL  // 🟥 禁止
if tags.has(FIRE)        // 🟩 允许
```

### Tag 统一使用场景

> **优化来源**：`docs/其他/74借鉴.md` §2 — UE GameplayTag 神设计：没有 Tag 系统 = 后期 if 地狱

GameplayTag 是 UE 中最被低估却最有威力的设计。所有需要"分类判断"的场景都应通过 Tag 而非硬编码 if 匹配：

| 使用场景 | 说明 | 示例 |
|----------|------|------|
| **技能筛选** | 按 Tag 过滤技能列表 | 查询所有 `Damage.Fire` 技能进行冷却缩减 |
| **Buff 触发** | Buff 的触发/免疫条件基于 Tag | 目标有 `Status.Poison` 免疫 → 跳过中毒效果 |
| **AI 决策** | AI 按 Tag 评估目标状态 | 目标有 `Status.Stunned` → 选择物理攻击 |
| **装备限制** | 装备佩戴条件通过 Tag 校验 | 需要 `Unit.Human` + `Proficiency.Sword` |
| **元素交互** | 元素克制/免疫通过 Tag 组合 | `Damage.Fire` + 目标 `Weakness.Fire` → 增伤 |
| **伤害过滤** | 受击时按 Tag 决定伤害类型 | 免疫 `Damage.Physical` → 物理伤害无效 |

### Tag 层级命名规范

Tag 应使用点分层级命名，支持前缀匹配查询：

```
Damage.Fire              → 伤害类型.火焰
Damage.Ice               → 伤害类型.冰霜
Damage.Physical          → 伤害类型.物理
Damage.Magical           → 伤害类型.魔法
Damage.True              → 伤害类型.真实伤害
Unit.Human               → 单位.人类
Unit.Boss                → 单位.Boss
Unit.Flying              → 单位.飞行
Status.Poison            → 状态.中毒
Status.Stunned           → 状态.眩晕
Status.Burning           → 状态.灼烧
Weapon.Sword             → 武器.剑
Weapon.Bow               → 武器.弓
Buff.Defensive           → Buff.防御类
Buff.Offensive           → Buff.攻击类
```

> **实现细节**：Tag 位掩码实现见 `docs/02-domain/shared_layer_rules.md`#标签

**没有 Tag 系统的后果**：

```rust
// 🟥 后期 if 地狱 — 硬编码匹配导致代码指数膨胀
if skill.element == "fire" && target.weakness == "fire" { damage *= 1.5; }
if skill.element == "ice" && target.weakness == "ice" { damage *= 1.5; }
if skill.element == "lightning" && target.weakness == "lightning" { damage *= 1.5; }
// 每新增一种元素/类型，所有匹配点都要修改 → 爆炸

// 🟩 Tag 驱动 — 零 if 地狱
let multiplier = modifier_rules.match(source_tags, target_tags);
// 新增元素类型只需添加 Tag 定义和 ModifierRule 配置，不改代码
```

---

## 4.10 Formula（公式系统）

> **优化来源**: `docs/其他/74借鉴.md` §14 — Formula 系统统一注册

这是最终层。Effect 只负责调用公式，不负责怎么算。

### FormulaRegistry：所有公式统一注册

所有公式必须在 `FormulaRegistry` 中注册，Effect 通过 `FormulaId` 调用，禁止在代码中硬编码计算逻辑。

```rust
// ✅ 正确：通过 FormulaRegistry 调用
let formula = formula_registry.get(formula_id);
let result = formula.calculate(context);

// 🟥 禁止：硬编码计算公式
let damage = atk * 2 + 30;  // 散落在代码中 = 后期平衡修改的地狱
```

> ⚠️ **反模式警告**：`atk * 2 + 30` 这类公式散落在代码各处，是后期数值平衡调整的噩梦。每次修改公式都需要全文搜索、逐个替换，极易遗漏且无法追溯修改历史。所有公式必须收敛到 `FormulaRegistry`，通过配置数据管理。

```rust
enum FormulaId {
    PhysicalDamage,    // 物理伤害公式
    MagicDamage,       // 魔法伤害公式
    TrueDamage,        // 真实伤害公式
    HealFormula,       // 治疗公式
    PoisonFormula,     // 中毒公式（基于攻击者属性）
    BurnFormula,       // 燃烧公式（固定值 + 属性缩放）
    SummonFormula,     // 召唤物属性公式
    ShieldFormula,     // 护盾吸收公式
}
```

效果执行时：

```rust
// Effect 不关心怎么算，只负责调用公式
let formula = formula_registry.get(formula_id);
let result = formula.calculate(context);
```

### Bevy ECS 映射

`FormulaRegistry` 作为 `Resource` 全局单例，在游戏初始化时注册所有公式：

```rust
#[derive(Resource)]
pub struct FormulaRegistry {
    formulary: HashMap<FormulaId, Box<dyn Formula>>,
}

impl FormulaRegistry {
    pub fn get(&self, id: FormulaId) -> &dyn Formula {
        self.formulary.get(&id).expect("Formula not registered")
    }
}
```

> 公式的详细实现见 `docs/02-domain/formula_rules.md` §FormulaRegistry。

---

# 5. 最终统一模型

## 5.1 数据驱动架构图（v3.0）

```
Ability（技能定义 + 施法校验）
├─ Requirement    ← 能不能放（释放前提）
├─ Cost           ← 消耗什么（MP/HP/怒气/弹药...）
├─ Targeting      ← 对谁放（敌方单体/十字/全体...）
├─ Effect[]       ← 放什么效果（Damage/Heal/ApplyModifier...）
└─ Tags           ← 分类标签（Fire/Physical/Melee...）

Effect（含原 Buff 能力）
├─ kind           ← 效果内容（Damage/Heal/ApplyModifier...）
├─ duration       ← 持续策略（Instant/TurnLimited/Permanent...）
├─ stacking       ← 堆叠规则（Replace/Refresh/StackAdd/StackMax...）
├─ triggers[]     ← 触发时机（TurnStart/AfterDamaged...）
├─ conditions[]   ← 生效条件（HP<30%/背击/有Buff...）
└─ Tags           ← 分类标签（Poison/Debuff/Physical...）

执行链路
    ↓
Execution        ← 公式计算（DamageExecution/HealExecution...）
    ↓
Modifier 链      ← 暴击/属性克制/地形/天气
    ↓
Attribute        ← 基础→派生属性刷新
    ↓
Tag              ← 标签增减/状态判定
    ↓
Cue              ← 表现事件（飘字/动画/音效）
```

## 5.2 概念关系图（v3.0）

```
                    ┌─────────────────┐
                    │  Ability (配置)   │
                    │ ─────────────── │
                    │ Requirement[]    │
                    │ Cost[]           │
                    │ Targeting        │
                    │ Effect[]         │
                    │ Tags             │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │  Effect 层       │
                    │ kind + Duration  │
                    │ + Stacking       │
                    └────────┬────────┘
                             │
               ┌─────────────┼─────────────┐
               ▼             ▼             ▼
      ┌──────────────┐ ┌──────────┐ ┌──────────────┐
      │  Execution   │ │ Modifier │ │  Attribute   │
      │ (公式计算)   │ │ 链(修饰) │ │ (属性刷新)  │
      └──────┬───────┘ └──────────┘ └──────────────┘
             │
             ▼
      ┌──────────────┐
      │     Tag      │
      │ (标签判定)   │
      └──────┬───────┘
             │
             ▼
      ┌──────────────┐
      │     Cue      │
      │ (表现事件)   │
      └──────┬───────┘
             │
             ▼
      ┌──────────────┐
      │ 游戏状态变更  │
      └──────────────┘
```

---

# 6. Content/Rule 映射表

> **优化来源**：`docs/其他/67.md` — Rule/Content 映射表强化（每个 Rust trait → 对应 RON config → Registry 完整链路）

| 概念 | Rule（Rust 代码） | Content（RON 配置） | Registry（运行时注册） |
|------|-------------------|---------------------|----------------------|
| **Ability** | 技能规则引擎（Validation → Route → Effect Pipeline）| AbilityDef（selector, effects[], cost, requirement）| AbilityRegistry（Resource）|
| **Effect** | EffectHandler trait（generate / preview / execute）| EffectDef 变体（Damage / Heal / ApplyModifier / ...）| EffectHandlerRegistry（Resource）|
| **Execution** 🆕 | Execution trait（calculate）| ExecutionDef（DamageExecution / HealExecution / ...）| ExecutionRegistry（Resource）|
| **Stacking** 🆕 | StackingRule 枚举（Replace / RefreshDuration / StackAdd / StackMax）| 嵌入 EffectDef 的 stacking 字段 | 无独立 Registry，由 Effect 统一管理 |
| **Selector** | TargetSelector trait（resolve_targets）| SelectorDef（EnemySingle / EnemyAOE / ...）| 内嵌在 AbilityDef 中 |
| **Cost** | CostValidator trait（check / consume）| CostDef（MpCost / HpCost / ItemCost / ...）| 内嵌在 AbilityDef 中 |
| **Requirement** | RequirementChecker trait（can_cast）| RequirementDef（HasWeapon / NotSilenced / ...）| 内嵌在 AbilityDef 中 |
| **Condition** | ConditionEvaluator trait（evaluate）| ConditionDef（HpBelow / BehindTarget / ...）| 内嵌在 EffectDef 中 |
| **Duration** | DurationPolicy trait（tick / expired）| DurationDef（Instant / TurnLimited / Permanent / ...）| 内嵌在 EffectDef 中 |
| **Modifier** | ModifierRule trait + ModifierCalculator | ModifierRuleDef（Flat / Percent / Override）| ModifierRuleRegistry（Resource）|
| **Attribute** 🆕 | Primary/Derived 属性计算 | AttributeDef（Might / Agility / MaxHp / ...）| AttributeRegistry（Resource）|
| **Formula** | Formula trait（calculate）| FormulaDef（PhysicalDamage / MagicDamage / ...）| FormulaRegistry（Resource）|
| **Tag** | GameplayTag 位掩码（has / add / remove）| TagName 枚举变体（Fire / Ice / Physical / ...）| 内嵌在各定义中 |
| **Cue** 🆕 | CueEvent 枚举 + CueSender/Receiver | 无配置（纯代码枚举）| 无独立 Registry（Message 机制）|

### 完整链路示例：SkillDef → RON → Registry → 运行时

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. RON 配置文件（Content）                                       │
│    assets/skills/fireball.ron                                   │
│    SkillDef { id: "fireball", selector: EnemySingle, ... }     │
├─────────────────────────────────────────────────────────────────┤
│ 2. AssetServer 加载（Bevy Asset Pipeline）                       │
│    Handle<SkillDef>  →  缓存在 Assets<SkillDef> 中             │
├─────────────────────────────────────────────────────────────────┤
│ 3. SkillRegistry 注册（Resource）                                │
│    registry.register("fireball", handle);                       │
│    运行时通过 skill_id 查找 Handle<SkillDef>                     │
├─────────────────────────────────────────────────────────────────┤
│ 4. SkillInstance 组件（Component）                               │
│    entity.insert(SkillInstance { def_handle, cooldown, ammo }); │
│    附加在实体上，包含运行时状态                                   │
├─────────────────────────────────────────────────────────────────┤
│ 5. EffectHandler 分发（trait）                                   │
│    registry.get("Damage").generate(ctx);                        │
│    通过 EffectHandlerRegistry 查找处理器                        │
├─────────────────────────────────────────────────────────────────┤
│ 6. Effect Pipeline 执行                                          │
│    Generate → Modify → Execute                                  │
│    最终修改 World 状态                                           │
└─────────────────────────────────────────────────────────────────┘
```

> Rule/Content 分离原则：新增内容只改 RON，不改 Rust 代码。
> 详见 `docs/01-architecture/03-data-config-asset/content-pipeline.md`。

---

# 7. 收敛原理

## 7.1 组合爆炸 → 原子收敛

```
1000 技能 ≈ 20 Selector × 20 Effect × 10 Cost × 5 Requirement
500  Buff ≈ 12 Trigger × 20 Effect × 7 Duration × 5 StackPolicy
```

关键洞察：游戏内容的组合爆炸收敛为一小套可组合的原语。

## 7.2 为什么收敛

> **优化来源**：`docs/其他/67.md` — "组合爆炸→原子收敛"量化证明

- 每个 Skill 和 Buff 本质上都是 **Selector/Trigger × Effect[] × 条件 × 参数** 的组合
- Effect 执行器只有 20~30 种原子能力
- 新增 1000 个技能，大多数情况下只是新增 RON 配置，不需要新增 Rust 代码
- 具体游戏的差异体现在 **配置数据**（参数、组合、条件），不在 **执行逻辑**

**量化证明**：

```
┌─────────────────────────────────────────────────────────────────┐
│  传统方式（一个技能一个函数）：                                    │
│  1000 技能 × 500 Buff = 1500 个函数 = 1500 个 Bug 来源           │
│                                                                 │
│  原子收敛方式：                                                   │
│  20 Effect × 12 Trigger × 9 Selector = 2160 种组合               │
│  但实际需要实现的 EffectHandler 只有 20 个                        │
│  实际需要实现的 Trigger Handler 只有 12 个                        │
│  实际需要实现的 Selector 只有 9 个                                │
│  总计：20 + 12 + 9 = 41 个可复用的原子组件                       │
│                                                                 │
│  收敛比：1500 → 41 = 36.6 倍代码缩减                             │
│  每新增 1 个技能：只需 1 个 RON 文件（约 20 行配置）               │
│  每新增 1 个 Buff：只需 1 个 RON 文件（约 15 行配置）             │
└─────────────────────────────────────────────────────────────────┘
```

关键洞察：游戏内容的组合爆炸收敛为一小套可组合的原语。500 技能 + 1000 Buff 不需要 1500 个函数，只需要 41 个原子组件的任意组合。

## 7.3 收敛后的代码量

```
Skill 新增：1 个 RON 文件
Buff 新增：1 个 RON 文件
Effect 新增（偶尔）：1 个 EffectHandler + 1 个 EffectDef 变体 + 注册
```

整套系统可能只需要 20~30 个 EffectHandler 实现 + 对应的 EffectDef 变体。

---

# 8. 当前覆盖状态与建设优先级

## 8.1 已有覆盖

| 系统 | 状态 | 文档 |
|------|------|------|
| Tag | ✅ 已实现 | `docs/02-domain/shared_layer_rules.md`（GameplayTag 位掩码）|
| Modifier | ✅ 部分实现 | `attribute_modifier_rules.md`（ModifierRule + Calculator）|
| Effect Pipeline | ✅ 已实现 | `attribute_modifier_rules.md`（Generate → Modify → Execute）|
| EffectHandler | ✅ 已实现 4 种 | `attribute_modifier_rules.md`（Damage/Heal/ApplyBuff/Cleanse）|
| Skill 模型 | ✅ 已实现 | `skill_rules.md`（SkillDef/SkillData/SkillTargeting）|
| Skill 条件 | ✅ 已实现 | `skill_rules.md`（SkillCondition 枚举）|

## 8.2 待建设系统

| 系统 | 状态 | 优先级 | 说明 |
|------|------|--------|------|
| **Condition** | ❌ 未覆盖 | 高 | 条件效果（ConditionalEffect = condition + effect）|
| **Requirement** | ⚠️ SkillCondition 部分覆盖 | 高 | 需要独立为 Requirement 系统，支持武器/沉默/状态等 |
| **Duration** | ❌ 未覆盖 | 高 | DurationPolicy（N 回合/直到死亡/永久...）|
| **StackPolicy** | ❌ 未覆盖 | 中 | 叠层系统（可叠 N 层/不可叠/刷新...）|
| **Trigger Context** | ❌ 未覆盖 | 中 | 触发上下文（damage_dealt、is_critical 等）|
| **Formula** | ❌ 未覆盖 | 中 | FormulaId（物理/魔法/治疗/中毒公式）|
| **完整 Selector** | ⚠️ SkillTargeting 部分覆盖 | 低 | 需要扩展为空地/召唤/十字/圆形等 |
| **完整 Cost** | ⚠️ cost_mp 部分覆盖 | 低 | 需要统一为 Cost[] 列表 |

## 8.3 建设优先级建议

> 来源：`docs/其他/27技能buf抽象.md` 第 1214-1234 行 + `docs/其他/76.md` SRPG-GAS 架构裁切

对于当前项目，真正值得现在就设计好的，不是再增加第 21、第 22 种 Effect，而是补齐以下系统：

```
Condition       → 条件效果（ConditionalEffect）— ✅ 已覆盖
Modifier        → 修饰器链 — ✅ 已有基础，需要扩展到 Buff 触发
Requirement     → 释放前提（独立于 SkillCondition）— ✅ 已覆盖
Duration        → 持续策略（DurationPolicy）— ✅ 已覆盖
StackPolicy     → 叠层系统 — ✅ 已覆盖
Tag             → 标签交互 — ✅ 已有基础
ExecutionStack  → LIFO 响应栈 — ❌ 新增（§4.8.1 升级）
TriggerRegistry → 统一触发器注册与分发 — ❌ 新增（§4.8.2 升级）
```

这六个系统决定未来能否做到：

```
1000 技能
500 Buff
基本不新增 Rust 代码
只新增配置
```

# 9. 禁止事项

## 🟥 绝对禁止

**禁止：一个技能 = 一个函数**

原因：100 个技能 = 100 个 Bug 来源，无法复用，违反 Rule/Content 分离
违反后果：代码膨胀，修复一个技能 Bug 可能影响另一个技能

**禁止：为新增技能修改 Rust 代码**

原因：新增内容 = 新增 RON 文件，不改代码
违反后果：违反 Rule/Content 分离原则

**禁止：绕过 Effect Pipeline 直接执行效果**

原因：Generate → Modify → Execute 是保证战斗公平性和可观测性的核心管线
违反后果：修饰规则不生效、伤害值异常、BattleRecord 缺少记录

**禁止：绕过 Modifier 管线直接修改属性**

原因：所有属性修改必须通过修饰器栈
违反后果：属性计算的统一性和可观测性被破坏

**禁止：Buff 永不过期**

原因：Buff 必须有 Duration 策略
违反后果：Buff 堆积导致属性无限增长

**禁止：Buff 无来源**

原因：移除 Buff 时必须清理对应的 Modifier
违反后果：Modifier 残留导致属性值与实际状态不一致

**禁止：Skill/Buff 硬编码效果逻辑**

原因：效果逻辑应在 EffectHandler 中实现，不在 Skill/Buff 定义中
违反后果：同一个 Effect 类型在不同 Skill 中有不同实现，无法统一维护

## 🟩 必须遵守

**必须：新增 Effect 类型通过 EffectHandler trait 实现**

```rust
impl EffectHandler for NewEffectHandler {
    fn type_name(&self) -> &str { "NewEffect" }
    fn generate(&self, ctx: &GenerateContext) -> Option<PendingEffectData> { ... }
    fn preview(&self, ctx: &PreviewContext) -> Option<EffectPreview> { ... }
    fn execute(&self, ctx: &mut ExecuteContext) -> Option<EffectResult> { ... }
}
```

**必须：Buff 的 Effect[] 也走 Effect Pipeline**

```
Buff Trigger 触发
    ↓
生成 Effect[]
    ↓
进入 Effect Pipeline（Generate → Modify → Execute）
    ↓
效果执行
```

**必须：StackPolicy 与 Buff 定义分离**

```
BuffDef {
    stack_policy: StackPolicy,  // 独立系统，不内嵌到 Buff 逻辑中
    // ...
}
```

**必须：Condition 与 Requirement 语义分离**

```
Requirement：技能能不能放（释放前判断）
Condition：效果是否生效（执行时判断）
```

**必须：Formula 由 Effect 调用，不在 Effect 中硬编码计算**

```
Effect 负责：调用哪个 Formula
Formula 负责：怎么算
```

---

# 10. Effect 作为一级领域（SRPG-GAS 核心裁切）

> **来源**：`docs/其他/76.md` §三（领域职责）— 借鉴 SRPG-GAS 的 Effect 独立领域化思想

## 10.1 为什么 Effect 必须是一级领域

很多 SRPG 项目的架构演进路径：

```
Phase 1: Character + Skill + Buff（起点）
Phase 2: + Effect（意识到 Effect 是独立概念）← 本项目当前处于此阶段
Phase 3: + Modifier + Tag（意识到修饰和分类也是独立概念）
Phase 4: + Trigger + Targeting + Cost + Requirement（正交子系统拆分）
Phase 5: + ExecutionStack + FormulaRegistry（执行调度和数值计算收敛）
```

当前项目已到 Phase 4，但 **Effect 仍不是真正的一级领域**：
- `EffectDef` 定义在 `skill/domain/types.rs` 中（不对）
- `EffectHandler` 在 `core/effect/handler.rs` 中（这种松散就位）
- 没有独立的 Effect Registry 生命周期
- 新增 Effect 类型需要改 2-3 个文件

### Effect 作为一级领域的要求

```
Effect 领域（独立于 Skill 和 Buff）
├── EffectDef          → 效果类型枚举
├── EffectData         → 运行时效果数据
├── EffectHandler trait → 效果处理器接口
├── EffectHandlerRegistry → 全局注册表 Resource
├── EffectPipeline     → Generate → Modify → Execute
└── EffectPreview      → 预览能力
```

### Effect 领域边界

| 维度 | Effect 负责 | Effect 不负责 |
|------|------------|--------------|
| **定义** | 效果类型、参数、标签 | 技能释放逻辑、Buff 生命周期 |
| **执行** | Generate → Modify → Execute 三步 | 目标选择、消耗扣除、冷却管理 |
| **扩展** | EffectHandler trait 实现 | Skill/Buff 配置格式 |
| **预览** | 效果数值计算（纯函数） | UI 布局、动画播放 |

## 10.2 Effect 数据流（新增视角）

76.md 提出的 `Character → Ability → Targeting → Effect → Buff → Modifier → Tag` 层级在项目中的映射：

```
Character (Unit Entity)
    ↓ 拥有
Skill (SkillDef + SkillSlots + SkillCooldowns)
    ↓ 执行管线：
    1. Requirement 检查（能不能放）
    2. Cost 扣除（消耗什么）
    3. Targeting 目标选择（对谁放）
    4. ⭐ Effect 执行（放什么效果） ← Effect 是核心
    5. Settlement 结算（冷却、状态更新）
            ↓
    Effect Pipeline (Generate → Modify → Execute)
            ↓
    EffectHandlerRegistry 分发
     ├── DamageHandler    → 伤害
     ├── HealHandler      → 治疗
     ├── ApplyBuffHandler → 施加 Buff → Buff 领域接管
     ├── CleanseHandler   → 驱散
     └── [新增]           → 扩展点
            ↓
    Buff (BuffData + BuffInstance + ActiveBuffs)
            ↓
    Modifier (StatModifier + AttributeModifierDef)
            ↓
    Tag (GameplayTag + GameplayTags)
```

## 10.3 Effect 生命周期

```
Effect 定义期：
RON 配置 → EffectDef → SkillDef/BuffDef 引用

Effect 执行期（完整链路）：
Skill/Buff 触发
    ↓
EffectDef 读取
    ↓
Step 1: Generate（纯函数，生成 PendineEffectData）
    ├── 从 EffectHandlerRegistry 查找 Handler
    ├── handler.generate(ctx) → PendingEffectData
    └── 含 amount、source_tags、terrain_id
    ↓
Step 2: Modify（纯函数，应用修饰规则）
    ├── ModifierRuleRegistry 遍历规则
    ├── 标签匹配 → Calculator 计算
    ├── 记录 ModifierEntry
    └── 输出修饰后的 PendingEffectData
    ↓
Step 3: Execute（副作用，修改游戏状态）
    ├── handler.execute(ctx) → EffectResult
    ├── 修改 HP/MP/状态
    ├── 发送 DamageApplied/HealApplied/BuffApplied Message
    └── 返回 target_died 等结果
    ↓
效果完成 → 触发 Trigger 链（→ ExecutionStack）
```

## 10.4 Effect 与 ExecutionStack 的协作

```
Effect Pipeline 执行完成
    ↓
产生领域事件（DamageApplied/HealApplied/CharacterDied）
    ↓
TriggerRegistry 分发 → 匹配的 Trigger Handler
    ↓
返回 EffectDef[] → 压入 ExecutionStack（LIFO）
    ↓
Stack 依次弹出 → 进入 Effect Pipeline
    ↓
递归，直到栈空或深度 ≥ MAX_STACK_DEPTH
```

**Effect 领域不关心 Stack**，Effect 只做好自己的三步。Stack 是 Trigger 领域的事情（§4.8.1）。这保证了 Effect 领域的纯粹性。

## 10.5 Effect 领域化对现有代码的影响

| 当前状态 | 迁移目标 | 影响范围 |
|---------|---------|---------|
| `EffectDef` 在 `skill/domain/types.rs` | 移至 `effect/domain/types.rs` | 所有 use 路径需要更新 |
| `EffectHandler` 在 `core/effect/handler.rs` | 保持不变（位置合理） | 无 |
| `EffectHandlerRegistry` 在 `core/effect/handler.rs` | 保持不变 | 无 |
| `PendingEffectData` 在 `battle/pipeline/` | 移至 `effect/domain/` | 少 |
| Skill/Buff 引用 Effect | 通过 EffectId 引用 | 无（当前已通过 EffectDef） |

---

# 11. 执行调度体系总览（SRPG-GAS 整体视图）

> **来源**：`docs/其他/76.md` §八（Battle Replay 兼容）、§九（Trigger 独立分层）

## 11.1 完整执行调度栈

```
┌──────────────────────────────────────────────────────┐
│                    Turn System                        │
│  TurnStarted → SelectUnit → ActionMenu → SelectTarget │
│  → ExecuteAction → TurnEnd → VictoryCheck            │
└────────────────────────┬─────────────────────────────┘
                         ↓
┌────────────────────────┴─────────────────────────────┐
│               Ability Execution Pipeline              │
│  1. CheckRequirement (SkillCondition)                 │
│  2. PayCost (cost_mp, SkillCondition::MpCost)         │
│  3. SelectTarget (SkillTargeting)                     │
│  4. ⭐ ExecuteEffect (→ Effect Pipeline)              │
│  5. SetCooldown (SkillCooldowns)                      │
└────────────────────────┬─────────────────────────────┘
                         ↓
┌────────────────────────┴─────────────────────────────┐
│               Effect Pipeline                         │
│  Generate → Modify → Execute                         │
│  (EffectHandlerRegistry 分发)                         │
└────────────────────────┬─────────────────────────────┘
                         ↓
┌────────────────────────┴─────────────────────────────┐
│               Trigger System + ExecutionStack         │
│  EffectResult → DomainEvent → TriggerRegistry        │
│  → EffectDef[] → ExecutionStack (LIFO) → Effect      │
│  Pipeline (递归，≤ MAX_STACK_DEPTH)                   │
└──────────────────────────────────────────────────────┘
```

## 11.2 Replay 兼容性

Replay 只需记录：

```
AbilityCast + TargetSelection + RandomSeed
```

重新模拟时，Ability → Effect → Buff → Modifier 的完整链路自动重现。

所有触发链（Trigger → ExecutionStack → Effect Pipeline）完全确定：
- 输入相同 + 种子相同 = 结果相同
- 无时间依赖、无帧依赖
- 无外部 IO

---

# 附录：交叉引用

| 主题 | 详细文档 |
|------|----------|
| 效果管线 Generate→Modify→Execute | `docs/02-domain/attribute_modifier_rules.md` |
| Effect 一级领域规则 | `docs/02-domain/effect/effect-rules.md` |
| Execution 执行算式 | `docs/02-domain/effect/effect-rules.md`（§4.3.1 对应） |
| Cue 表现信号总线 | `docs/02-domain/effect/cue_rules.md`（待建） |
| Stacking 堆叠规则 | `docs/02-domain/stack-policy/stack-policy_rules.md` |
| Attribute 属性体系 | `docs/02-domain/character/character-rules.md` |
| Ability 定义与验证 | `docs/02-domain/skill/skill-rules.md` |
| Rule/Content 分离、数据驱动架构 | `docs/01-architecture/03-data-config-asset/content-pipeline.md` |
| GameplayTag 位掩码实现 | `docs/02-domain/shared_layer_rules.md` |
| 七层架构、模块边界 | `docs/01-architecture/README.md` |
| Trigger 规则定义（含 ExecutionStack） | `docs/02-domain/trigger/trigger-rules.md` |
| Stack 解析策略 | `docs/02-domain/stack-policy/stack-policy_rules.md` |
| 战斗 FSM 与调度时序 | `docs/01-architecture/01-battle-gas/battle-fsm-design.md` |
| SRPG Lite-GAS 冻结版架构 | `docs/其他/77.md` |
| SRPG-GAS 裁切方案 | `docs/其他/76.md` |
| 原始 Skill/Buff 抽象来源 | `docs/其他/27技能buf抽象.md` |
| 宪法条款 | `docs/00-governance/ai-constitution-complete.md` |

---

## 宪法合规说明

| 条款 | 合规状态 | 说明 |
|------|---------|------|
| 🟩 §1.1.3 Rule/Content 分离 | ✅ 合规 | 技能 = 配置（RON），效果 = 逻辑（EffectHandler） |
| 🟩 §1.1.6 组合优于继承 | ✅ 合规 | Skill = Selector + Effect[] + Cost[] + Requirement[] + Tags |
| 🟩 §7.0.1 分层扩展 | ✅ 合规 | Modifier 管线处理数值，Trait 处理行为，独立系统处理特殊机制 |
| 🟩 §8.0.3 属性修改规范 | ✅ 合规 | 所有属性变动通过 Modifier Pipeline |
| 🟩 §11.2.1 技能执行管线 | ✅ 合规 | Validate → Cost → Cast → Effect → Settlement |
| 🟩 §11.3.1 Buff 生命周期 | ✅ 合规 | Trigger → Duration → StackPolicy → Effect |
| 🟩 §2.2.2 Trigger 机制 | ✅ 合规 | Buff 通过 Trigger 构建事件链 |
| 🟩 ExecutionStack LIFO 响应栈 | ✅ 新增合规 | 嵌套触发（反击/吸血/死亡爆炸）通过 Stack 调度 |
| 🟩 Effect 一级领域化 | ✅ 新增合规 | Effect 独立于 Skill 和 Buff，拥有独立生命周期 |
| 🟩 TriggerRegistry 统一注册 | ✅ 新增合规 | 所有 Trigger Handler 统一注册，事件发生时自动分发 |
| 🟥 §1.1.7 避免过度设计 | ⚠️ 需关注 | 20 种 Effect 类型为设计目标，当前仅实现 4 种，新增必须基于明确需求 |
| 🟥 Stack 深度上限 | ⚠️ 需关注 | MAX_STACK_DEPTH=32，防止无限递归 |

---

# 附录 B：工业借鉴 Top10

> **优化来源**：`docs/其他/74借鉴.md` 最终排序（§850-881）— 对 SRPG 独立开发最值钱的 Top10 系统

### Top10 排序

| # | 系统 | 来源 | 本项目对应 | 价值 |
|---|------|------|-----------|------|
| 1 | **GAS（Ability + Effect）** | UE Gameplay Ability System | Skill + Effect Pipeline | 战斗子系统的骨架 |
| 2 | **GameplayTag** | UE 标签系统 | Tag（§4.9） | 统一分类判断，消灭 if 地狱 |
| 3 | **Curve** | UE CurveTable | FormulaRegistry + 配置曲线 | 数据驱动数值成长 |
| 4 | **AttributeSet** | UE 属性集 | AttributeSet（§1.1 映射表） | 集中管理可修改属性 |
| 5 | **Command Pattern** | Unity 大型项目 | MoveCommand / AttackCommand / CastSkillCommand | Replay/AI/联机的基石 |
| 6 | **Resource/Definition** | Godot Resource | SkillDef / BuffDef（Content 层） | Definition/Instance 分离 |
| 7 | **Formula System** | RPG 工业经验 | FormulaRegistry（§4.10） | 统一数值计算 |
| 8 | **Requirement System** | RPG 工业经验 | Requirement（§4.5） | 技能释放前提检查 |
| 9 | **Trigger System** | 卡牌游戏 | TriggerRegistry（§4.8.2） | 统一触发注册与分发 |
| 10 | **Action Queue** | RPG 工业经验 | Action Queue + ExecutionStack（§4.8.1） | 执行调度与嵌套响应 |

### SRPG-GAS 对齐声明（v3.0 更新）

> **来源**：`docs/其他/77.md` §一 — SRPG Lite-GAS 冻结版架构

本项目不照搬 Unreal GAS（Gameplay Ability System），而是做 **SRPG Lite-GAS（轻量版领域化 Ability Framework）**。

GAS 的 70% 复杂度来自网络同步、预测回滚、客户端服务端一致性，对本项目的单机回合制 SRPG 是纯负担。本项目裁剪了以下 GAS 复杂度：

- ❌ Prediction / Client Prediction / Server Reconciliation（不需要）
- ❌ ASC 网络同步 / 复制（不需要）
- ❌ GameplayTask 体系（SRPG 不需要单独抽象）
- ❌ Aggregator 黑盒计算（用 Execution trait 替代）
- ❌ 实时触发 / 帧同步（用回合驱动替代）

保留了以下 GAS 精华并适配为 SRPG 领域模型：

| GAS 精华 | SRPG-Lite-GAS 映射 | 项目状态 |
|----------|-------------------|---------|
| Ability | Ability（能力释放管线） | ✅ 已实现 |
| Effect | Effect（一级领域，含 Duration 变体） | ✅ 已实现（v3.0 吸收 Buff） |
| GameplayTag | Tag（位掩码标签） | ✅ 已实现 |
| Modifier | ModifierRule + ModifierCalculator | ✅ 已实现 |
| Attribute | Attribute（Primary/Derived 双层） | ✅ 已实现 |
| Execution | Execution trait（公式执行） | 🆕 v3.0 新增 |
| Stacking | StackingRule 枚举 | 🆕 v3.0 新增 |
| Cue | CueEvent（表现信号总线） | 🆕 v3.0 新增 |
| Cost | CostDef | ✅ 已实现 |
| Cooldown | SkillCooldowns | ✅ 已实现 |
| Requirement | Requirement | ✅ 已实现 |
| Trigger | TriggerRegistry 统一分发 | ✅ 已实现 |
| Execution Stack | ExecutionStack（LIFO 响应栈） | ✅ 已实现 |
| Registry | ExecutionRegistry / EffectHandlerRegistry | 🆕 v3.0 强化 |
| Replay | BattleReplay（确定性回放） | 🆕 v3.0 新增 |

### 融合架构宣言

本项目不是纯粹的 Bevy ECS 实践，而是多引擎工业经验的融合架构：

```
Bevy ECS         — 强类型组件化、Schedule 并行调度、Resource 全局状态
  +
UE GAS           — Skill/Effect 分离、GameplayTag、AttributeSet、Modifier 链
  +
Godot Resource   — Definition 即数据资产、Scene 即独立 Plugin
  +
Unity ScriptableObject — 配置驱动、Asset 管线
  +
SRPG 工业经验     — Command Pattern、Formula、Requirement、Trigger、Stack
```

**核心洞察**：Bevy 0.18 是优秀的 ECS 框架，但不等于成熟的游戏开发框架。很多工业级经验需要主动引入。本项目的架构本质上是 **Bevy ECS 作为运行时底座 + UE GAS 作为战斗抽象骨架 + Godot/Unity 的数据驱动思想 + SRPG 行业的 Trigger/Stack/Command 经验**。

> 这套融合架构是独立开发长期连载战棋 RPG 能够达到的最稳、最可扩展的路线。

---

# 版本修订说明

**v3.0**（当前版本）：
- 来源：`docs/其他/77.md`（SRPG Lite-GAS 冻结版架构）
- **核心变更**：对齐 SRPG Lite-GAS 10 模块 + 3 基建冻结版架构
- 新增 §1.2 完整 GAS 链路图（Ability → Targeting → Effect → Stacking → Execution → Modifier → Attribute → Tag → Cue → Replay）
- 新增 §4.3.1 Execution 层（trait-based 公式执行，替代 Effect 内部 match 分支）
- 新增 §4.8.3 Cue 层（统一表现事件总线，业务逻辑零耦合 UI）
- Buff 吸收为 Effect + Duration（删除独立顶层 Buff 模块）
- GAS 映射表从 8 概念扩展为 10 模块 + 3 基建
- Attribute 升级为 Primary/Derived 双层一级领域
- Stacking 升级为独立领域（Replace / RefreshDuration / StackAdd / StackMax）
- §5 数据驱动架构图和概念关系图全面更新
- §6 Content/Rule 映射表新增 Execution / Stacking / Attribute / Cue 行
- §附录B GAS 对齐声明更新（新增 Execution / Stacking / Cue / Registry / Replay）

**v2.0**：
- 新增 ExecutionStack（LIFO 响应栈）
- 新增 TriggerRegistry 统一注册与分发
- Effect 一级领域化
- Effect Pipeline 三步（Generate → Modify → Execute）

**v1.0**：
- 初始版本：Skill/Buff/Effect 统一数据驱动抽象模型
- 20 种 Effect 分类、10 大补充系统、收敛原理
