这恰恰是很多战棋项目最后失控的原因。

新人通常会这样设计：

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

做到100个技能：

```text
100个函数
100个逻辑
100个Bug来源
```

做到500个技能：

```text
彻底爆炸
```

而《铃兰之剑》《火纹》《梦幻模拟战》《FFTA》这种游戏，实际上根本不是这样实现的。

---

# 第一层抽象：技能 ≠ 逻辑

你要先建立一个认知：

```text
技能
≠
逻辑

技能
=
配置
```

例如：

```text
火球术
治疗术
猛击
中毒箭
```

在程序眼里根本不存在。

程序只认识：

```text
Effect
```

---

# 第二层抽象：技能 = Target + Effects

我建议你的最终模型：

```rust
Skill {
    target_rule,
    effects,
}
```

例如：

```rust
Fireball {
    target_rule: EnemySingle,

    effects: [
        Damage(120),
    ]
}
```

---

治疗术：

```rust
Heal {
    target_rule: AllySingle,

    effects: [
        Heal(100),
    ]
}
```

---

中毒箭：

```rust
PoisonArrow {
    target_rule: EnemySingle,

    effects: [
        Damage(80),
        ApplyBuff(Poison),
    ]
}
```

---

程序只执行：

```rust
for effect in skill.effects {
    execute(effect);
}
```

---

# 第三层抽象：Effect 才是核心

真正需要分类的是：

```rust
Effect
```

而不是 Skill。

---

# 我推荐的完整 Effect 分类

## 1 Damage

造成伤害

```rust
Damage
```

包含：

```text
物理
魔法
真实
百分比
反伤
```

其实都是参数。

---

## 2 Heal

恢复生命

```rust
Heal
```

包含：

```text
固定
百分比
持续恢复
```

---

## 3 Shield

护盾

```rust
Shield
```

---

## 4 Resource

资源变化

```rust
ModifyResource
```

例如：

```text
MP
TP
怒气
行动点
```

---

## 5 Attribute

属性修改

```rust
ModifyAttribute
```

例如：

```text
攻击
防御
速度
暴击
命中
闪避
```

---

## 6 Buff

施加Buff

```rust
ApplyBuff
```

---

## 7 Debuff

施加Debuff

```rust
ApplyDebuff
```

其实和Buff可以统一。

---

## 8 Dispel

驱散

```rust
Dispel
```

例如：

```text
驱散增益
驱散减益
驱散全部
```

---

## 9 Purify

净化

```rust
Purify
```

---

## 10 Revive

复活

```rust
Revive
```

---

## 11 Summon

召唤

```rust
Summon
```

---

## 12 Teleport

位移

```rust
Teleport
```

---

## 13 Push

击退

```rust
Push
```

---

## 14 Pull

拉拽

```rust
Pull
```

---

## 15 Swap

交换位置

```rust
SwapPosition
```

---

## 16 Transform

变身

```rust
Transform
```

---

## 17 SpawnEntity

生成实体

```rust
SpawnEntity
```

例如：

```text
陷阱
地雷
图腾
召唤物
```

---

## 18 RemoveEntity

删除实体

---

## 19 TriggerSkill

触发技能

```rust
TriggerSkill
```

例如：

```text
连击
追击
反击
协同攻击
```

---

## 20 Execute

斩杀

```rust
Execute
```

---

# 第四层抽象：Buff才是真正的大头

很多人以为技能多。

其实：

```text
Buff > 技能
```

---

因为：

```text
100技能
≈

15种Effect组合
```

---

但：

```text
100Buff
=
100种规则
```

---

# Buff不要设计成技能

推荐：

```rust
Buff {
    triggers,
    effects,
}
```

---

例如中毒：

```rust
Poison
```

```rust
Trigger:
    TurnStart

Effect:
    Damage(30)
```

---

燃烧：

```rust
Burn
```

```rust
Trigger:
    TurnStart

Effect:
    Damage(50)
```

---

本质完全一样。

只是参数不同。

---

# Trigger 才是Buff核心

推荐统一：

```rust
enum Trigger {
    TurnStart,
    TurnEnd,

    BeforeAttack,
    AfterAttack,

    BeforeDamaged,
    AfterDamaged,

    BeforeMove,
    AfterMove,

    KillTarget,
    Death,

    BattleStart,
    BattleEnd,
}
```

---

# 举几个经典Buff

## 中毒

```rust
TurnStart

Damage(30)
```

---

## 再生

```rust
TurnStart

Heal(30)
```

---

## 狂怒

```rust
AfterDamaged

ModifyAttribute(
    Attack +10%
)
```

---

## 荆棘

```rust
AfterDamaged

DamageBack(20)
```

---

## 吸血

```rust
AfterAttack

Heal(造成伤害20%)
```

---

# 最终抽象

做到这里以后：

```text
Skill
=
TargetRule
+
Effect[]
```

---

```text
Buff
=
Trigger[]
+
Effect[]
```

---

```text
Effect
=
20~30种原子能力
```

---

然后：

```text
500个技能
1000个Buff
```

最终都会收敛成：

```text
20~30个Effect Executor
```

例如：

```rust
execute_damage()
execute_heal()
execute_buff()
execute_attribute()
execute_summon()
execute_teleport()
```

整个项目可能只需要二三十个真正的执行器。

这其实就是《铃兰之剑》《梦幻模拟战》《火纹》《FFTA》这类SRPG最常见的数据驱动架构：

```text
Skill
    ↓
TargetRule
    ↓
Effect[]
    ↓
EffectExecutor

Buff
    ↓
Trigger
    ↓
Effect[]
    ↓
EffectExecutor
```

而不是：

```text
一个技能
=
一个函数
```

如果你按 Effect + Trigger 体系设计，后面新增 1000 个技能，大多数情况下都只是新增配置，不需要新增 Rust 代码。

没有穷尽。

但注意，不是继续往 `Effect` 里面加几十种类型。

很多架构最后失败，不是因为 Effect 不够，而是因为只抽象了：

```text
Skill
Buff
Effect
```

却没有继续向上抽象。

如果你准备做《铃兰之剑》《FFTA》《火纹》《梦幻模拟战》这种长期SRPG，那么除了前面那套 Effect 体系，还应该补充下面这些层。

---

# 第一类：Condition（条件系统）

这是很多人最晚才意识到的重要层。

前面只有：

```text
Effect
```

实际上应该变成：

```text
Condition
+
Effect
```

例如：

```text
生命低于50%
暴击时
背击时
击杀时
未移动时
有Buff时
无Buff时
相邻友军时
地形为草地时
```

---

例如：

```text
技能：
处决

Condition:
目标生命 < 30%

Effect:
Execute
```

---

例如：

```text
技能：
背刺

Condition:
目标背后

Effect:
Damage +50%
```

---

最终：

```rust
ConditionalEffect {
    condition,
    effect,
}
```

---

# 第二类：Selector（目标选择器）

很多人把目标规则写死在技能里。

实际上：

```text
敌方单体
敌方十字
敌方全体
友方单体
自身
空地
召唤物
```

都是：

```text
Selector
```

---

未来你会发现：

```text
技能种类 ≠ Effect种类

技能种类 ≈ Selector × Effect
```

---

例如：

```text
火球术
=
EnemySingle
+
Damage
```

---

```text
火焰风暴
=
EnemyAOE
+
Damage
```

---

逻辑完全一样。

---

# 第三类：Modifier（修饰器）

这是很多SRPG最容易漏掉的。

例如：

```text
暴击
弱点
地形加成
职业克制
Buff加成
天气加成
```

实际上都不是 Effect。

---

它们是：

```text
Modifier
```

---

例如：

```text
Damage
  ↓
Modifier
  ↓
Final Damage
```

---

如果没有 Modifier 层。

后面会出现：

```rust
execute_damage()
execute_damage_with_crit()
execute_damage_with_element()
execute_damage_with_backstab()
```

越来越多。

---

# 第四类：Cost（消耗系统）

很多项目后期才补。

实际上技能天然包含：

```text
Cost
```

---

例如：

```text
MP
HP
怒气
行动点
弹药
耐久
金币
祭品
```

---

推荐：

```rust
Skill {
    costs,
    selector,
    effects,
}
```

---

不要把 MP 消耗写进技能逻辑。

---

# 第五类：Requirement（释放前提）

和 Condition 不同。

Condition：

```text
效果是否生效
```

---

Requirement：

```text
技能能不能放
```

---

例如：

```text
需要弓
需要枪
需要站立
需要目标存在
需要MP>=30
需要有弹药
需要未沉默
```

---

# 第六类：Duration（持续规则）

很多人只做：

```text
持续3回合
```

其实不够。

---

常见：

```text
持续N回合

直到死亡

直到移动

直到攻击

直到受伤

直到战斗结束

永久
```

---

实际上应该抽象：

```rust
DurationPolicy
```

---

# 第七类：Stack（叠层系统）

后期Buff爆炸来源。

---

例如：

```text
中毒
可叠5层

流血
可叠10层

易伤
不可叠
```

---

推荐单独抽象：

```rust
StackPolicy
```

---

而不是塞进 Buff。

---

# 第八类：Trigger Context（上下文）

前面说了 Trigger。

但还缺：

```text
TriggerContext
```

---

例如：

```text
AfterAttack
```

到底是谁攻击？

攻击了谁？

造成多少伤害？

是否暴击？

---

Buff执行时需要这些数据。

---

例如吸血：

```text
AfterAttack
```

需要：

```text
damage_dealt
```

---

否则实现不了。

---

# 第九类：Tag（标签系统）

这是大型项目神器。

---

例如：

```text
Fire
Ice
Lightning

Physical
Magical

Melee
Ranged

Holy
Dark

Poison
Bleed
```

---

未来：

```text
火伤+20%

所有Fire技能CD-1
```

都是Tag驱动。

---

不要硬编码：

```rust
if skill_id == FIREBALL
```

---

# 第十类：Formula（公式系统）

这是最终层。

---

很多项目做到后期：

```rust
damage = atk * 1.2
```

到处都是。

---

推荐：

```rust
FormulaId
```

---

例如：

```text
PhysicalDamage

MagicDamage

HealFormula

PoisonFormula

SummonFormula
```

---

Effect只负责：

```text
调用公式
```

---

而不负责：

```text
怎么算
```

---

# 如果是工业级SRPG

最终抽象层级应该是：

```text
Skill
├─ Requirement
├─ Cost
├─ Selector
├─ Effect[]
└─ Tags

Buff
├─ Trigger
├─ Duration
├─ StackPolicy
├─ Condition[]
├─ Effect[]
└─ Tags

Effect
├─ Damage
├─ Heal
├─ Shield
├─ Attribute
├─ Buff
├─ Summon
└─ ...

Effect执行
↓
Modifier链
↓
Formula
↓
最终结果
```

---

对于你的项目，我认为真正值得现在就设计好的，不是再增加第21、第22种 Effect，而是补齐下面六个系统：

```text
Condition
Modifier
Requirement
Duration
StackPolicy
Tag
```

这六个东西决定你未来能否做到：

```text
1000技能
500Buff
基本不新增Rust代码
只新增配置
```

这才是《铃兰之剑》《梦幻模拟战》《FFTA》这类长期运营SRPG真正的数据驱动核心。
