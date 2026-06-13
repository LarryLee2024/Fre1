# 技能/Buff/Effect 统一数据驱动抽象模型

Version: 1.0
Status: Proposed
Source: `docs/其他/27技能buf抽象.md`

本文档定义 SRPG 项目的技能、Buff、Effect 统一数据驱动抽象模型。
核心目标：500 技能 + 1000 Buff 收敛为 20~30 个 Effect Executor，新增内容只改配置不改代码。

> 本文档是 `architecture.md`、`content-pipeline.md`、`attribute_modifier_rules.md`、`skill_rules.md` 的胶水层，
> 将分散的 Effect Pipeline、Skill Registry、Buff 模块、标签系统、修饰规则统一到一个概念模型中。

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

## 2.4 第四层：Buff = Trigger[] + Effect[]

Buff 才是真正的大头：

```
100 技能 ≈ 15 种 Effect 组合
100 Buff = 100 种规则
```

Buff 模型：

```
Buff {
    triggers[],      // 触发时机（什么时候触发）
    duration,        // 持续策略（持续多久）
    stack_policy,    // 叠层规则（如何叠层）
    conditions[],    // 生效条件（是否触发）
    effects[],       // 效果列表（触发什么）
    tags,            // 标签（分类）
}
```

### 经典 Buff 示例

| Buff | Trigger | Effect[] |
|------|---------|----------|
| 中毒 | TurnStart | [Damage(30)] |
| 再生 | TurnStart | [Heal(30)] |
| 狂怒 | AfterDamaged | [ModifyAttribute(Attack, +10%)] |
| 荆棘 | AfterDamaged | [DamageBack(20)] |
| 吸血 | AfterAttack | [Heal(造成伤害 × 20%)] |

本质完全一样，只是参数不同。

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

```
Skill / Buff
    ↓
Effect[] 生成
    ↓
Effect Pipeline: Generate → Modify → Execute
    ↓
EffectHandler trait 分发
    ↓
游戏状态变更
```

## 3.2 Effect Pipeline 三步管线

> 详见 `docs/domain/attribute_modifier_rules.md` 效果管线章节。

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

> 详见 `docs/domain/attribute_modifier_rules.md` 效果处理器章节。

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

---

# 4. 十大补充系统

> 以下系统来自 `docs/其他/27技能buf抽象.md` 的第六层到第十五层抽象。
> 很多架构失败，不是因为 Effect 不够，而是因为只抽象了 Skill/Buff/Effect，却没有继续向上抽象。

## 4.1 Condition（条件系统）

条件系统将效果从"无条件执行"升级为"条件触发执行"。

```
ConditionalEffect {
    condition,    // 触发条件
    effect,       // 效果
}
```

| 条件 | 说明 | 示例 |
|------|------|------|
| HpBelow(percent) | 目标 HP 低于百分比 | 处决：目标 HP < 30% 时触发 Execute |
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

> 已在 `docs/domain/attribute_modifier_rules.md` 中详细定义 ModifierRule、ModifierCalculator trait、
> 标签匹配机制。此处明确 Modifier 在上层抽象中的位置：Modifier 是 Effect 执行链路中的修饰环节，
> 不是 Effect 本身。

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

## 4.9 Tag（标签系统）

标签是大型项目神器。

> 已在 `docs/domain/shared_layer_rules.md` 中定义 GameplayTag 位掩码实现。
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

---

## 4.10 Formula（公式系统）

这是最终层。Effect 只负责调用公式，不负责怎么算。

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

---

# 5. 最终统一模型

## 5.1 数据驱动架构图

```
Skill
├─ Requirement    ← 能不能放（释放前提）
├─ Cost           ← 消耗什么（MP/HP/怒气/弹药...）
├─ Selector       ← 对谁放（敌方单体/十字/全体...）
├─ Effect[]       ← 放什么效果（Damage/Heal/Buff...）
└─ Tags           ← 分类标签（Fire/Physical/Melee...）

Buff
├─ Trigger[]      ← 什么时候触发（TurnStart/AfterDamaged...）
├─ Duration       ← 持续多久（N回合/直到死亡/永久...）
├─ StackPolicy    ← 如何叠层（可叠N层/不可叠/刷新...）
├─ Condition[]    ← 触发条件（HP<30%/背击/有Buff...）
├─ Effect[]       ← 触发什么效果（Damage/Heal/ModifyAttribute...）
└─ Tags           ← 分类标签（Poison/Debuff/Physical...）

Effect 执行
    ↓
Modifier 链       ← 暴击/属性克制/地形/天气
    ↓
Formula           ← 计算公式
    ↓
最终结果
```

## 5.2 概念关系图

```
                    ┌─────────────────┐
                    │   Skill (配置)    │
                    │ ─────────────── │
                    │ Requirement[]    │
                    │ Cost[]           │
                    │ Selector         │
                    │ Effect[]         │
                    │ Tags             │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ Effect Pipeline  │
                    │ Generate→Modify  │
                    │ →Execute         │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
     ┌──────────────┐ ┌──────────┐ ┌──────────────┐
     │ EffectHandler │ │ Modifier │ │   Formula    │
     │ (trait 分发)  │ │ 链(修饰) │ │  (计算公式)  │
     └──────┬───────┘ └──────────┘ └──────────────┘
            │
            ▼
     ┌──────────────┐
     │ 游戏状态变更  │
     └──────────────┘

                    ┌─────────────────┐
                    │   Buff (配置)     │
                    │ ─────────────── │
                    │ Trigger[]        │
                    │ Duration         │
                    │ StackPolicy      │
                    │ Condition[]      │
                    │ Effect[]         │
                    │ Tags             │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │  Buff 规则引擎    │
                    │ Trigger→Validate │
                    │ →Effect Pipeline │
                    └─────────────────┘
```

---

# 6. Content/Rule 映射表

| 概念 | Rule（Rust 代码） | Content（RON 配置） |
|------|-------------------|---------------------|
| **Skill** | 技能规则引擎（Validation → Route → Effect Pipeline）| SkillDef（selector, effects[], cost, requirement）|
| **Buff** | Buff 规则引擎（Trigger → Validate → Effect Pipeline）| BuffDef（triggers[], duration, stack_policy, conditions[], effects[]）|
| **Effect** | EffectHandler trait（generate / preview / execute）| EffectDef 变体（Damage / Heal / Shield / ...）|
| **Selector** | TargetSelector trait（resolve_targets）| SelectorDef（EnemySingle / EnemyAOE / ...）|
| **Cost** | CostValidator trait（check / consume）| CostDef（MpCost / HpCost / ItemCost / ...）|
| **Requirement** | RequirementChecker trait（can_cast）| RequirementDef（HasWeapon / NotSilenced / ...）|
| **Condition** | ConditionEvaluator trait（evaluate）| ConditionDef（HpBelow / BehindTarget / ...）|
| **Duration** | DurationPolicy trait（tick / expired）| DurationDef（Turns / UntilDeath / ...）|
| **StackPolicy** | StackPolicy trait（can_stack / merge）| StackDef（Stackable / MaxStack / ...）|
| **Formula** | Formula trait（calculate）| FormulaDef（PhysicalDamage / MagicDamage / ...）|
| **Tag** | GameplayTag 位掩码（has / add / remove）| TagName 枚举变体（Fire / Ice / Physical / ...）|

> Rule/Content 分离原则：新增内容只改 RON，不改 Rust 代码。
> 详见 `docs/architecture/content-pipeline.md`。

---

# 7. 收敛原理

## 7.1 组合爆炸 → 原子收敛

```
1000 技能 ≈ 20 Selector × 20 Effect × 10 Cost × 5 Requirement
500  Buff ≈ 12 Trigger × 20 Effect × 7 Duration × 5 StackPolicy
```

关键洞察：游戏内容的组合爆炸收敛为一小套可组合的原语。

## 7.2 为什么收敛

- 每个 Skill 和 Buff 本质上都是 **Selector/Trigger × Effect[] × 条件 × 参数** 的组合
- Effect 执行器只有 20~30 种原子能力
- 新增 1000 个技能，大多数情况下只是新增 RON 配置，不需要新增 Rust 代码
- 具体游戏的差异体现在 **配置数据**（参数、组合、条件），不在 **执行逻辑**

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
| Tag | ✅ 已实现 | `shared_layer_rules.md`（GameplayTag 位掩码）|
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

> 来源：`docs/其他/27技能buf抽象.md` 第 1214-1234 行

对于当前项目，真正值得现在就设计好的，不是再增加第 21、第 22 种 Effect，而是补齐以下六个系统：

```
Condition       → 条件效果（ConditionalEffect）
Modifier        → 修饰器链（已有基础，需要扩展到 Buff 触发）
Requirement     → 释放前提（独立于 SkillCondition）
Duration        → 持续策略（DurationPolicy）
StackPolicy     → 叠层系统（独立于 Buff 定义）
Tag             → 标签交互（已有基础，需要更多交互规则）
```

这六个系统决定未来能否做到：

```
1000 技能
500 Buff
基本不新增 Rust 代码
只新增配置
```

---

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

# 附录：交叉引用

| 主题 | 详细文档 |
|------|----------|
| 效果管线 Generate→Modify→Execute | `docs/domain/attribute_modifier_rules.md` |
| Skill 定义与验证 | `docs/domain/skill_rules.md` |
| Rule/Content 分离、数据驱动架构 | `docs/architecture/content-pipeline.md` |
| GameplayTag 位掩码实现 | `docs/domain/shared_layer_rules.md` |
| 七层架构、模块边界 | `docs/architecture.md` |
| 原始来源 | `docs/其他/27技能buf抽象.md` |
