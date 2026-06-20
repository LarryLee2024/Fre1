# GAS-Lite 能力系统深度解析 — 游戏规则的"魔法圈"

> 如果你用过 Unreal Engine 的 Gameplay Ability System（GAS），然后发现 Fre 也有类似的东西——没错，Fre 的 Capabilities 系统就是一个"GAS-Lite"：保留了 Ability → Effect → Modifier → Attribute 的核心理念，但为回合制 SRPG 做了大幅裁剪。没有网络同步，没有预测/回滚，没有 GameplayTask——换来的是 15 个自包含的领域模块、严格的 C1→C2→C3 分层、以及一套纯函数驱动的四阶段聚合管线。

---

## 1. 什么是 GAS-Lite？

### 1.1 从 UE GAS 说起

Unreal Engine 的 Gameplay Ability System 是一套为动作游戏设计的技能框架。它的核心概念链非常清晰：

```
Ability（技能）→ Effect（效果）→ Modifier（数值修改）→ Attribute（属性）
```

这个链路的优点在于**关注点分离极度严格**——Ability 只管"发起"，Effect 只管"执行"，Modifier 只管"改数值"，Attribute 只管"存属性"。谁都不越界。

但 UE GAS 是为《堡垒之夜》这种游戏设计的——它需要处理网络同步、客户端预测、服务器回滚、延迟补偿。这些对单机回合制战棋来说完全不必要，反而带来了巨大的复杂度。

### 1.2 Fre 的裁剪策略

Fre 保留了 UE GAS 的概念链路，但做了三方面裁剪：

| 保留 | 去掉 | 改进了 |
|------|------|--------|
| Ability → Effect 管线 | 网络同步 | 纯函数驱动，去掉了 GameplayTask |
| Tag 标签系统 | 客户端预测/回滚 | 明确 Def→Spec→Instance 分离 |
| Modifier 聚合管线 | GameplayEffect | 引入 Cue 作为表现信号层 |
| Cue 表现信号 | GameplayCue 管理器 | 事件驱动而非轮询 |

结果是一个**完全单机、纯函数式、确定性的能力系统**——每个能力激活、每段伤害计算、每次属性修改都可以被测试框架独立验证。

### 1.3 "魔法圈"的含义

代码注释和架构文档里反复出现一个词："**魔法圈**"（Magic Circle）。意思是从 Tag（最底层的标签分类）一路贯穿到 Cue（最终的表现信号），这 15 个能力模块形成了一个封闭的、自我完备的链路：

```
任何游戏行为，最终都能被追踪为：
  Tag 分类 → Condition 检查 → Ability 发起 → Execution 计算
  → Effect 生效 → Modifier 修改 → Aggregator 聚合 → Attribute 变更 → Cue 表现
```

没有任何游戏行为能够绕过这个链路——这是宪法的硬性要求。

---

## 2. 15 个能力模块全景

### 2.1 三层分治

这 15 个能力（Capabilities）不是平铺的，而是按依赖关系分成了三个逻辑层：

```
第一层：Foundation（基础数据层）—— 缺了谁也玩不转
  Tag → Modifier → Attribute → Aggregator → GameplayContext

第二层：Logic Skeleton（配置与条件层）—— 架子搭起来
  Spec → Condition → Trigger → Event

第三层：Behavior（行为与表现层）—— 真正的游戏逻辑
  Ability → Targeting → Execution → Effect → Stacking → Cue
```

再加上一个 **Runtime**（运行时编排层），它不提供新概念，它协调上面三层的执行。

还有一个 **Rule**（规则引擎），它是纯数据驱动的，没有自己的 Plugin，直接嵌入到执行链路里。

### 2.2 逐能力简析

#### Foundation 层（5 个）

**Tag（标签）**——最底层的基础设施。一个 `TagId`（强类型字符串，前缀 `tag_`）就是一个语义标签，比如 `tag_element_fire`、`tag_status_stun`。标签是层级化的（继承体系），内部用 128 位 `BitMask` 做 O(1) 包含检查。几乎所有其他能力都引用 Tag。

核心文件：`tag/foundation/types.rs`（TagId, BitMask, TagNamespace, TagQuery）
关键代码模式：
```rust
// 标签检查是 O(1) 的——不涉及字符串比较或树遍历
fn has_tag(entity_tags: &TagSet, target_tag: TagId) -> bool {
    entity_tags.bitmask.contains(target_tag.bit_index())
}
```

**Modifier（数值修改器）**——属性的"原子操作"。一个 Modifier 只做三件事：Add（加法）、Multiply（乘法）、Override（替换）。它不关心业务含义，只关心"怎么改数值"。`ModifierData` 包含 [op, target_attribute, magnitude, priority, source]，全是纯数据。

**Attribute（属性）**——角色的数值骨架。分为四类：Primary（力量/敏捷等基础属性）、Secondary（由 Primary 衍生的比如攻击力）、Derived（由多个属性计算得出的比如负重上限）、Resource（HP/MP 等可消耗资源）。`DerivedFormula` 支持 Constant / Sum / Max / Min / WeightedSum / Custom 六种派生公式。

**Aggregator（聚合器）**——把一堆 Modifier 变成最终属性值。执行严格四阶段管线：

```
Add 阶段： base_value + sum(所有 Add 修饰)
  → Multiply 阶段： current × product(所有 Multiply 修饰)【连乘！不是累加】
    → Override 阶段： 取最高优先级的 Override 值直接替换
      → Clamp 阶段： current.clamp(min, max)
```

这四步是一个纯函数——输入 `[ModifierEntry]` + `base_value`，输出 `AggregationResult`。没有副作用，没有外部状态。

关键代码（`aggregator/mechanism/pipeline.rs`）：
```rust
pub fn execute_aggregation(modifiers: &[ModifierEntry], base: f32, pipeline: &CalcPipeline) -> AggregationResult {
    // Add 阶段: 把所有 Add modifier 的 magnitude 加起来
    let add_sum = modifiers.iter()
        .filter(|m| m.op == ModifierOp::Add)
        .map(|m| m.magnitude)
        .sum::<f32>();

    // Multiply 阶段: 所有 Multiply 连乘 (1.2 × 1.3 = 1.56, 不是 1 + 0.2 + 0.3)
    let mul_product = modifiers.iter()
        .filter(|m| m.op == ModifierOp::Multiply)
        .map(|m| m.magnitude)
        .product::<f32>();

    // Override 阶段: 最高优先级
    let override_val = modifiers.iter()
        .filter(|m| m.op == ModifierOp::Override)
        .max_by_key(|m| m.priority);

    // Clamp 阶段: 边界保障
    let final_val = result.clamp(pipeline.clamp_min, pipeline.clamp_max);
    // ...
}
```

**GameplayContext（玩法上下文）**——整个能力链路的"数据书包"。从 Ability 激活到 Cue 播放，这中间经过的所有阶段都可以往 ContextChain 里放数据。包含循环检测（同 source+target+ability 的链节点达到上限时中断）。

#### Logic Skeleton 层（4 个）

**Spec（桥梁层）**——解决"配置定义的技能 vs 运行时具体的技能"的鸿沟。`AbilityDef`（从 RON 文件加载的静态配置）→ `AbilitySpec`（运行时桥接，携带等级、冷却缩减、强化选项）→ `AbilityInstance`（正在执行的实例，携带状态机进度）。每层职责明确。

**Condition（条件系统）**——递归条件树。叶子节点是基础检查（TagRequirement / AttributeCheck / ResourceCheck），组合节点是 And/Or/Not。整个树是一个递归枚举：

```rust
pub enum Condition {
    TagRequirement { mode: TagRequirementMode, tag_id: TagId },
    AttributeCheck { attribute_id: AttributeId, comparison: ComparisonOp, value: f32 },
    ResourceCheck { resource_id: AttributeId, amount: f32 },
    And(Vec<Condition>),
    Or(Vec<Condition>),
    Not(Box<Condition>),
    Custom(CustomCondition),
}
```

评估器（`evaluator.rs`）递归遍历这棵树，短路求值。用在三个地方：Effect 是否可以施加？Ability 能否激活？Trigger 能否触发？

**Trigger（触发器）**——"当 X 发生时响应 Y"。比如"当受到火属性伤害时触发自燃效果"。12 种触发类型（OnDamaged / OnHealed / OnTurnStart / OnDeath / OnConditionMet / OnCustom 等）。Trigger 本身不做事，它监听 Event，经过 Condition 过滤，然后激活 Ability。

**Event（事件系统）**——领域间的通信基础设施。13 种 EventTag 类型，带循环保护（`EVENT_CYCLE_LIMIT = 5`）。Event 不是简单的 Pub/Sub——它是在 Trigger 上层提供"什么发生了"的语义信号。`DamageDealt`、`Healed`、`UnitDied`、`BuffApplied`……每类事件都有一个明确的 Payload 结构。

#### Behavior 层（6 个）

**Ability（技能）**——游戏里所有"动作"的模板。状态机：**Ready → Casting → Active → Cooldown → Ready**（再加 Blocked 和 Removed 两个终端状态）。不变量：同一个 Spec 不能有两个活跃实例。

Ability 本身**没有任何业务逻辑**（这是 Data Law 004 的硬性要求）。它只定义：Cost（消耗什么）、Cooldown（冷却多久）、Targeting（怎么选目标）、Effects（产生哪些效果）。至于"伤害怎么算"、"Buff 持续多久"——那是 Execution 和 Effect 的事。

**Targeting（目标选择）**——7 种目标类型（Self / Ally / Enemy / Dead / Any / Summon / Party）× 7 种范围形状（Single / Area / Line / Cone / Chain / Burst / Wall）× 5 种优先级规则（Nearest / Farthest / LowestHealth / HighestHealth / Random）。

**Execution（执行计算）**——数值结算中心。分发到 Damage / Heal / DirectAttributeMod / Custom 四种执行类型。Damage 路径包含：读取属性修正 → 计算骰面值 → 固定加值 → 暴击判断 → 非负保证。计算结果是 `ExecutionResult`，包含 `CalcTrace`（每一阶段的输入输出都可追踪——不变量 3.2 的硬性要求）。

**Effect（效果）**——所有"结果"的载体。四阶段生命周期：

```
Applying（刚生成，还没进入状态）→ Active（生效中，每回合 Tick）
  → Expiring（已到期，等系统清理）→ Removed（已移除）
```

Effect 有 Instant（瞬发，如伤害）、HasDuration（持续 N 回合，如 Buff）、Infinite（永久，如被动技能）三种持续时间。每种 Effect 包含：Modifier 配置列表（要改哪些属性、怎么改）、Cue 绑定（播放什么表现）、堆叠配置（能不能叠加）、条件（满足什么条件才施加）。

**Stacking（堆叠）**——控制"同一个 Effect 能叠多少份"。四种策略：
- **None**：不允许叠加，第二次直接拒绝
- **Aggregate**：叠层数（受 `max_stacks` 限制），满了之后的策略由 OverflowBehavior 决定
- **RefreshDuration**：重置持续时间到最大值，不叠层
- **Replace**：新实例替换旧实例

**Cue（表现信号）**——Effect 完成后，"表现层该知道什么"的信号通道。Cue 定义了一个语义信号（OnApply / OnTick / OnRemove / OnInterrupt），但不包含 VFX/SFX 的具体实现。VFX/SFX 系统订阅 Cue 事件来播放特效。Data Law 009 明确规定：**Effect 不得直接播放特效，必须经过 Cue**。

### 2.3 Runtime 编排层

Runtime 不是一个"能力"，而是一个编排器。它有 5 个子模块：

```
pipeline/     —— 通用管线引擎（Stage → Step → Hook，失败策略 Abort/Skip/Retry）
scheduler/    —— 游戏时间、阶段推进、帧管理
registry/     —— 全局 Def 注册中心（ID 分配、冲突检测）
command/      —— 业务命令枚举 + 命令队列（玩家/AI/Replay 统一入口）
replay/       —— 确定性回放（Recorder + Player + RNG + Validator）
```

其中 `pipeline/` 是最核心的——它提供了一个通用编排引擎，可以被 Ability Pipeline、Modifier Pipeline、Combat Pipeline 复用。`PipelineDefinition` 定义了 stages → steps 的拓扑结构，`PipelineHook` trait 提供观察者式生命周期回调。

---

## 3. Def → Spec → Instance 三层分离

这是 Fre GAS-Lite 最容易被误解的概念。很多人以为 Def 就是配置数据，Instance 就是运行时实体——但中间还有一层 Spec。

```
Definition（RON 文件定义，只读）
    │  例如：AbilityDef { id: "abl_fireball", cost: 20, cooldown: 3, effects: [...] }
    │
    ▼
Spec（运行时配置槽位，授予实体后生成）
    │  例如：AbilitySpec { def_id: "abl_fireball", level: 3, cooldown_reduction: 1,
    │                     enhancements: [...], last_used_frame: None }
    │
    ▼
Instance（执行中的实例，激活后才创建）
    │  例如：AbilityInstance { spec_id: "...", state: Casting, cast_progress: 0.5, ... }
```

为什么中间要插一层 Spec？因为同一个技能（Def）被不同角色使用时，配置可能不同：等级不同、强化选项不同、冷却缩减不同。Def 是只读的模板，不能在那里写运行时数据。Instance 是每次激活才创建的临时对象。Spec 就是持久的、每个角色独享的"配置槽位"。

Effect 也一样：`EffectDef` → `EffectSpec`（含快照、持续时间修改、堆叠计数）→ `EffectInstance`（运行时状态）。

---

## 4. 全链路端到端数据流

这是 GAS-Lite 最核心的东西：一次技能激活，数据是怎么一步步流过所有模块的。

### 4.1 六阶段 Ability Pipeline（ADR-010）

```
Phase 1: Validate      ── 目标校验、条件检查、冷却检查
Phase 2: PreCost       ── 扣消耗（SP/MP/物品）
Phase 3: Targeting     ── 确定最终目标集合
Phase 4: Execute       ── 生成 Effect 列表
Phase 5: Resolve       ── Effect 逐一执行
Phase 6: PostCost      ── 冷却开始、充能消耗
```

每个阶段是一个独立 Event，通过 `commands.trigger()` 发布。这六个阶段不是硬编码在一个函数里的——它们是一个"管线定义"，由 Runtime Pipeline 引擎编排。

### 4.2 Effect 执行的四次跳转

当 Effect 被施加（Phase 5 的 `Resolve` 阶段）时，会分叉三条路径：

```
Effect 被施加后，产生三个独立的输出：
  │
  ├── 1. Modifier（数值修改）→ Stacking 检查（能不能叠）
  │      → ModifierSet（写入目标实体的组件）
  │      → Aggregator 标记属性为 Dirty
  │      → 下次读取时重新执行四阶段聚合
  │
  ├── 2. Trigger（连锁触发）→ 检查领域事件是否匹配 Trigger 条件
  │      → 如果匹配，激活新的 Ability（形成链路）
  │      → GameplayContext 做循环检测（链长 ≤ 10）
  │
  └── 3. Cue（表现信号）→ 发送到 CueBus
         → VFX/SFX/UI 系统消费
```

这三条路径是并发的——Modifier 修改数值，Trigger 可能触发连锁反应，Cue 通知表现层。互不阻塞。

### 4.3 Modifier Pipeline 的四阶段聚合（ADR-011）

```
Phase 1: Collect   ── 从实体的 ModifierSet 收集所有活跃 Modifier
Phase 2: Aggregate ── Add → Multiply → Override → Clamp
Phase 3: Resolve   ── 输入 AttributeResolver 计算最终值
Phase 4: Publish   ── 发布 AttributeChanged 事件
```

### 4.4 完整事件链

```
AbilityActivated
  → TargetSelected / NoValidTarget
    → ExecutionCompleted / ExecutionFailed
      → EffectApplied / EffectImmunityTriggered
        → ModifierApplied / ModifierRemoved
          → AggregateDirty
            → AggregationComplete
              → [属性值更新]
```

每个箭头都是 `commands.trigger()` → observer system 消费。这意味着你可以挂载任意多个 observer 到任意环节——日志系统就在 AbilityActivated 上挂了一个 observer，每次技能激活都会自动记录到 tracing。

---

## 5. 宪法级别的约束（Data Laws #004-#009）

GAS-Lite 最严格的东西不是代码框架——是一组不可违抗的数据架构规则（Data Laws），写在 `docs/04-data/README.md` 里。这六条约束直接塑造了整个 Capabilities 系统的形状。

### #004：Ability 不含业务逻辑

Ability 只描述 Cost、Cooldown、Targeting、Effects 的集合。任何"伤害怎么算、Buff 怎么生效"的逻辑都不可以写在 Ability 模块里。

```
✅ 正确：AbilityDef.effects = ["eff_fireball_damage"]
❌ 错误：AbilityDef 里有 damage_formula 字段
```

实践影响：新增一个技能时，你不需要修改 Ability 的代码。只需要写 EffectDef（业务逻辑）+ CueDef（表现信号）+ 配置数值。

### #005：Effect 是唯一业务执行入口

没有人——不是 Ability，不是 Trigger，不是 Condition——可以直接修改数值。所有"对游戏世界产生影响"的操作必须经过 Effect。

这条约束的效果远超预期：它让整个游戏的所有"结果"都变成了可审计的。你想知道"这个秒杀效果是从哪来的"？追溯 Effect 的来源就行。Effect 是唯一的路口。

### #006：Modifier 不含业务逻辑

Modifier 只改变数值。它不决定"这个 Buff 能不能叠"（那是 Stacking 的事），不决定"这个值要不要触发暴击"（那是 Execution 的事）。Modifier 就是 Add / Multiply / Override 三个原子操作。

### #007：Duration 属于 Effect

Duration 不是独立系统。没有独立的"Buff 管理器"或"持续效果系统"。所有持续时间的配置，都落在 `EffectDef.duration` 字段里。`Instant` / `HasDuration { turns, calculation }` / `Infinite`——三种选择，没有第四种。

### #008：堆叠行为归属 Stacking

关于"能不能叠"，只在一个地方定义：`StackingConfig`。不在 AbilityDef、EffectDef、ModifierConfig 里散落堆叠逻辑。

### #009：表现必须经过 Cue

Effect 不能直接播放 VFX/SFX。Effect 执行完后产生 Cue 信号，表现层订阅 Cue 事件来展示效果。这让 Effect 可以保持纯逻辑（无渲染依赖），也让表现层可以在不同平台（PC / Mobile）上有不同的实现。

---

## 6. 17 个模块的 C1→C2→C3 自包含模式

每个能力模块（tag / modifier / effect / ability / ...）都遵循完全相同的内部结构：

```
capabilities/<能力名>/
├── plugin.rs              # Bevy Plugin（注册 resources、observers）
├── events.rs              # 领域事件（pub，模块的公开 API）
├── foundation/            # C1：纯数据定义（pub(crate)）
│   ├── types.rs           #   枚举、标记、约束
│   ├── values.rs          #   值对象、结构体
│   ├── error.rs           #   领域错误枚举
│   └── def.rs             #   Asset 定义（如有）
├── mechanism/             # C2：规则与逻辑（pub(crate)）
│   ├── components.rs      #   ECS Components
│   ├── lifecycle.rs       #   生命周期管理
│   └── systems/           #   Bevy Systems
└── tests/                 # 测试
```

这种结构意味着：**你不用读整个模块来理解它在做什么**。打开 foundation/ 看数据类型，打开 events.rs 看它发出什么事件，打开 plugin.rs 看它注册了什么。99% 的理解需求都能在这三处解决。

---

## 7. 依赖关系图

虽然 15 个能力听起来很多，但它们的依赖关系其实很稀疏。只有 4 个模块依赖其他模块的 foundation 类型：

```
tag {}                       ← 被所有其他模块引用
attribute {}                 ← 被 modifier, execution, condition 引用
modifier {}                  ← 被 effect 的 def.rs 引用
event {}                     ← 被 trigger（监听事件）引用

condition { tag }            ← 依赖 tag
effect  { modifier,          ← 依赖四个低层模块
          execution,
          stacking,
          cue } 
stacking {}                  ← 被 effect 引用
cue {}                       ← 被 effect 引用
ability {}                   ← 被 execution, effect 引用
execution {}                 ← 被 effect 引用
targeting {}                 ← 被 ability 引用
spec {}                      ← 被 ability, effect 引用
gameplay_context {}          ← 被整个链路消费
aggregator {}                ← 隐式依赖 modifier
trigger { event, condition } ← 依赖 event + condition
rule { condition }           ← 依赖 condition
```

简单来说：**大多数模块不依赖别人**。15 个能力里只有 condition、effect、trigger、rule 四家有外部依赖，其余都是自包含的。这个设计不是偶然的——它是 C1 Foundation 层设计的核心目标：低层模块零上层依赖。

---

## 8. 常见陷阱

### 8.1 "我想加个新属性"

不要直接在 Attribute 系统里加。先问自己：这个"属性"是静态数值（比如力量 18）还是动态修改（比如力量+2）？如果是静态数值，加 `AttributeDefinition`。如果是动态修改，用 `Modifier`。如果是两者的组合——先写 `AttributeDefinition`，然后用 Effect + Modifier 来动态修改。

### 8.2 "这个效果不经过 Cue"

Effect 写完直接播放 VFX？Data Law 009 会生气。先定义一个 `CueDef`（比如 `cue_fireball_explosion`），然后在 `EffectDef.cues` 里引用它。VFX 代码订阅 `CueData` 事件来播放。这样 Effect 是纯逻辑（可测试），VFX 是纯表现（可替换）。

### 8.3 "这个堆叠放哪里方便"

放在 ModifierConfig 里？放在 AbilityDef 里？都不对。Data Law 008 说所有堆叠配置归 Stacking。要叠加 Effect，在 `EffectDef.stacking` 里配 `StackingConfig`。

### 8.4 "我要绕过 Aggregator"

直接写 `target_entity.hp -= 50`？这违反了 #005（Effect 是唯一入口）和 Modifier Pipeline 的约束。正确路径：通过 Effect 发出 Modifier，Modifier 进入 Aggregator，Aggregator 计算新属性值，属性写入系统响应 `AggregationComplete` 事件。

---

## 9. 相关 ADR

| ADR | 标题 | 说明 |
|-----|------|------|
| ADR-010 | Ability → Effect Pipeline Architecture | 六阶段管线定义 |
| ADR-011 | Modifier → Attribute Pipeline Architecture | 四阶段聚合管线 |
| ADR-012 | Stacking / Trigger / Cue Separation | 三个关注点的分离边界 |
| ADR-013 | Registry & Hot-reload | 注册表和热重载 |
| ADR-044 | Pipeline Engine | 统一管线引擎（Stage→Step→Hook） |

---

## 10. 代码入口

如果你想自己走一遍 GAS-Lite 的代码，这个顺序是最合理的：

1. **`src/core/capabilities/mod.rs`** — 看看 17 个模块都有谁
2. **`src/core/capabilities/tag/foundation/types.rs`** — 从最底层的 Tag 开始
3. **`src/core/capabilities/modifier/foundation/types.rs`** — ModifierOp 的三个原子操作
4. **`src/core/capabilities/aggregator/mechanism/pipeline.rs`** — `execute_aggregation` 四阶段聚合
5. **`src/core/capabilities/ability/foundation/def.rs`** — AbilityDef 的结构
6. **`src/core/capabilities/ability/mechanism/lifecycle.rs`** — `try_activate` 的激活流程
7. **`src/core/capabilities/effect/foundation/def.rs`** — EffectDef 的结构
8. **`src/core/capabilities/effect/mechanism/lifecycle.rs`** — `apply_effect` → `tick_durations` → `expire_effects`
9. **`src/core/capabilities/execution/mechanism/calculator.rs`** — Damage/Heal/Custom 分发
10. **`src/core/capabilities/runtime/pipeline/mechanism/executor.rs`** — `execute_pipeline` 通用编排

---

## 11. Capabilities / Domains 双轴架构——机翻制 vs 业务翻译

前面的内容全部在讲 Capabilities（能力机制），但 Fre 的架构还有一个同等重要的轴：**Domains（业务领域）**。这两者的关系是理解整个项目架构的关键。

### 11.1 两个轴各管什么

```
Capabilities（能力轴）             Domains（业务轴）
─────────────────────────          ─────────────────────────
管"机制"——怎么做                    管"业务"——做什么
Tag 标签系统                        Tactical（战术：移动、掩体、夹击）
Modifier 数值修改                   Combat（战斗：回合流程、胜负判定）
Effect 效果生命周期                 Spell（法术：专注、豁免、升环）
Execution 执行计算                  Progression（成长：经验、等级、天赋）
...                                ...
15 个通用能力模块                   15 个业务子系统
无游戏语义                         满载游戏语义
可以被任何 SRPG 复用                Fre 独有
```

用一句话区分：**Capabilities 提供了"扣血"的能力（Execution + Modifier），但"什么情况下扣血、扣多少、扣完触发什么"是 Combat Domain 的业务逻辑。**

### 11.2 Domains 的内部结构

每个 Domain 有标准 7 文件结构（以 `combat/` 为例）：

```
domains/combat/
├── plugin.rs              # Plugin（注册 Component、Resource、Observer、Pipeline）
├── components.rs          # ECS Components（ActionPoints, TurnQueue, BattlePhase）
├── systems/               # 业务 Systems
│   ├── input_system.rs    #   玩家输入响应
│   ├── turn_systems.rs   #   回合流程管理
│   └── effect_tick_system.rs  # 效果计时
├── events.rs              # 领域事件
├── error.rs               # 领域错误（CombatError）
├── failure.rs             # 规则失败（CombatFailure）
├── rules/                 # 纯业务规则（纯函数，零 ECS）
│   ├── formulas.rs        #   伤害公式
│   └── rules.rs           #   胜负判定、回合约束
└── integration/           # ★ 唯一调用 Capabilities 的入口
    ├── ability/           #   按能力域拆分的子模块
    ├── effect/
    ├── execution/
    └── ...
```

### 11.3 integration/——Anti-Corruption Layer

`integration/` 是 **Domain 调用 Capabilities 的唯一通道**。它的设计原则：

1. **禁止 Domain 的 System 直接 import Capabilities 组件类型进行字段访问**
2. `integration/` 里的 facade 函数是唯一访问 Capabilities 数据的地方
3. 当 Capabilities 内部结构变化时，只需要修改 `integration/` 里的代码

以 combat 为例，它的 `integration/` 按 10 个能力域拆分子模块（ability / aggregator / condition / effect / event / execution / gameplay_context / replay / targeting / trigger / turn），每个子模块里是 facade 函数 + View Types + SystemParam。

### 11.4 依赖方向

```
Capabilities（无业务语义） ←── Domains（满载业务语义）
         ↑                          ↑
         │                          │
         └────── 都依赖 Shared ──────┘
```

**Domains 引用 Capabilities，Capabilities 绝不引用 Domains。** 这是从底层防止业务逻辑泄漏到通用机制层。

### 11.5 Plugin 注册顺序

```
Foundation Capabilities  (5)  Tag → Attribute → ... → GameplayContext
Logic Skeleton Capabilities (4)  Spec → Condition → Trigger → Event
Behavior Capabilities     (6)  Ability → ... → Cue
Runtime Capabilities      (1)  RuntimePlugin
────────────────────────────────────────────────────────
Foundation Domains        (3)  Tactical → Terrain → Faction
Core Domains              (7)  Combat → Spell → ... → Party
Narrative & Economy       (5)  Narrative → Quest → ... → Summon
```

15 Capabilities 先注册（基础机制就绪），然后 15 Domains 再注册（业务逻辑上线）。层级清晰。

---

## 12. Error vs Failure 严格分离——Bug 和业务结果是两回事

### 12.1 根本区别

这是 Fre 最重视的原则之一（ADR-051），也是新来者最容易混淆的地方。

```
程序错误（Error）            规则失败（Failure）
──────────────              ──────────────
本质：程序 Bug              本质：业务规则不满足
含义："不该发生的异常"       含义："业务预期中的分支"
示例：配置不存在、ID 无效    示例：MP 不足、背包满了、冷却中
处理：? 传播、panic 上报     处理：UI 提示、播放拒绝动画
类型：thiserror Error        类型：枚举 + RuleFailure trait
```

### 12.2 代码里的体现——ActivationIssue

Ability 的激活函数 `try_activate` 返回的不是 `Result<T, E>`——而是一个分裂的两路结果：

```rust
// Ability 激活的返回类型
pub enum ActivationIssue {
    /// 程序错误（不应发生的异常）
    Error(AbilityError),
    /// 业务规则失败（正常预期结果）
    Failure(AbilityFailure),
}
```

`AbilityError` 是程序 Bug（7 个变体，如 `InvalidStateTransition`、`SpecNotFound`），而 `AbilityFailure` 是业务分支（3 个变体：`ConditionFailed` / `InsufficientCost` / `OnCooldown`）。

### 12.3 RuleFailure trait

所有 Failure 类型实现同一个 trait：

```rust
/// 规则失败标记 trait——统一结构，归各领域所有
pub trait RuleFailure: Debug + Send + Sync + 'static {
    /// 返回机器可读的错误码
    fn code(&self) -> &'static str;
}
```

Domain 层面，CombatFailure 有 4 个变体（NotYourTurn / NoActionRemaining 等），InventoryFailure 有 8 个，SpellFailure 有 9 个。每个 Failure 枚举都实现了 `RuleFailure`，返回唯一的错误码字符串。

### 12.4 目前在哪里落地

| 位置 | 类型 | 变体数 |
|------|------|--------|
| Capabilities: `ability/foundation/failure.rs` | AbilityFailure | 3 |
| Capabilities: `effect/foundation/failure.rs` | EffectFailure | 2 |
| Capabilities: `ability/foundation/error.rs` | AbilityError（程序错误） | 7 |
| Capabilities: `effect/foundation/error.rs` | EffectError（程序错误） | 7 |
| Domain: `combat/failure.rs` | CombatFailure | 4 |
| Domain: `inventory/failure.rs` | InventoryFailure | 8 |
| Domain: `spell/failure.rs` | SpellFailure | 9 |
| Domain: `reaction/failure.rs` | ReactionFailure | — |
| Domain: `progression/failure.rs` | ProgressionFailure | — |
| Domain: `quest/failure.rs` | QuestFailure | — |
| Domain: `faction/failure.rs` | FactionFailure | — |
| Domain: `party/failure.rs` | PartyFailure | — |
| Domain: `crafting/failure.rs` | CraftingFailure | — |

---

## 13. Command 命令系统——玩家/AI/回放统一入口

### 13.1 为什么需要一个统一的命令系统

游戏里的"动作"有很多来源：玩家点按钮、AI 做决策、回放录制回退。如果每个来源单独处理，会产生三条并行的执行路径，每次都写三份相似的处理逻辑。

Command 系统把这三个来源统一成一个入口。

### 13.2 GameCommand 枚举

所有业务命令的起点（`runtime/command/foundation/types.rs`）：

```rust
pub enum GameCommand {
    // ── Tactical ──
    MoveUnit { unit_id: String, path: Vec<String> },
    Wait { unit_id: String },

    // ── Combat ──
    Attack { attacker_id: String, target_id: String, ability_slot: Option<u32> },
    CastSpell { caster_id: String, spell_def_id: String, target_id: String },
    UseItem { user_id: String, item_instance_id: String, target_id: Option<String> },

    // ── Turn ──
    EndTurn { unit_id: String },

    // ── Meta ──
    OpenMenu, SaveGame, LoadGame,
}
```

### 13.3 命令的来源与流程

```
Player Input ──┐
AI Decision  ──┼──→ CommandQueue Resource ──→ ExecuteCommand System
Replay       ──┘
```

不管是玩家、AI 还是回放，产生的都是同一个 `GameCommand` 枚举。`CommandSource` 标记命令来源：

```rust
pub enum CommandSource { Player, AI, Replay, System }
```

执行系统**不区分命令来源**——攻击就是攻击，不管谁发起的。这让回放录制变得极其简单：录制时只记录 `GameCommand` + `frame_number`，回放时按帧重播命令流。

### 13.4 RecordedCommand——回放的原子单元

```rust
pub struct RecordedCommand {
    pub source: CommandSource,
    pub command: GameCommand,
    pub frame_number: u64,
}
```

回放文件就是 `Vec<RecordedCommand>` 按 `frame_number` 排序的时间线。

---

## 14. Content.rs——从 RON 配置文件到运行时的桥梁

### 14.1 问题

游戏数据写在 RON 文件里（`assets/config/`），但 ECS 世界需要运行时类型（Resource、Component）。谁负责把 RON 配置翻译成运行时数据？

### 14.2 Content.rs 模式

每个需要从配置加载数据的 Capability，有一个 `content.rs` 模块（注意是 `mod content;`——私有模块，只被 plugin.rs 调用）：

```
tag/mod.rs:     mod content;         // ← 私有，不被外部看到
tag/content.rs:                      // ← RON → TagHierarchy
```

`content.rs` 的职责：

```rust
// tag/content.rs 的核心逻辑
pub(crate) fn register_tags_from_content(
    mut hierarchy: ResMut<TagHierarchy>,      // 运行时标签层级
    mut loaded_tags: ResMut<LoadedTagDefs>,   // 从 RON 加载的标签定义
    mut commands: Commands,
) {
    let defs = std::mem::take(&mut loaded_tags.defs);
    for def in defs {
        hierarchy.register(def, &mut commands);
    }
}
```

流程：

```
RON 文件（assets/config/tags/*.ron）
    ↓ Content Plugin 加载
LoadedTagDefs Resource（Vec<TagDefinition>）
    ↓ register_tags_from_content system（在 content.rs 里）
TagHierarchy Resource（运行时数据结构，O(1) BitMask 查询）
```

### 14.3 哪些能力有 content.rs

查看各能力的 `mod.rs` 可以看到哪些有 `mod content;` 声明：tag、attribute、ability、effect——这四个能力有自己的 RON 配置文件需要加载。其他能力（如 condition、trigger、cue）的数据要么嵌入在 Def 里作为子结构，要么由其他系统动态生成。

---

## 15. 通用 Pipeline 引擎——复用一段编排器

### 15.1 问题

Ability 有六阶段管线、Modifier 有四阶段管线、Combat 也有自己的管线。每个管线都是 Stage → Step → Execute 的模式。如果每条管线各自实现，就会有三份几乎相同的编排代码。

### 15.2 Pipeline 引擎的定位（ADR-044）

Pipeline 引擎是**纯编排器，不含业务逻辑**。它只提供五个能力：

```
① Stage 注册与排序
② Step 执行调度
③ 前置/后置 Hook（PipelineHook trait）
④ 执行日志记录
⑤ 失败策略执行（Abort / SkipAndContinue / Retry）
```

### 15.3 核心类型

```rust
/// Pipeline 的整体定义
pub struct PipelineDefinition {
    pub stages: Vec<PipelineStage>,
    pub state: PipelineState,
}

/// 一个 Stage 包含多个 Step
pub struct PipelineStage {
    pub name: String,
    pub steps: Vec<PipelineStep>,
    pub failure_strategy: FailureStrategy,
}

/// 一个 Step 指向一个执行器
pub struct PipelineStep {
    pub name: String,
    pub executor: StepExecutor,      // 函数指针
}

/// 失败策略
pub enum FailureStrategy {
    Abort,                           // 终止整个管线
    SkipAndContinue,                 // 跳过失败的步骤
    Retry { max_retries: u32 },      // 最多重试 N 次
}
```

### 15.4 Hook 系统

```rust
/// Pipeline Hook——前置/后置回调
pub trait PipelineHook: Send + Sync {
    fn name(&self) -> &str;
    fn on_stage_start(&self, _stage: &str, _context: &PipelineContext) {}
    fn on_stage_end(&self, _stage: &str, _context: &PipelineContext, _result: &StepResult) {}
    fn on_step_start(&self, _stage: &str, _step: &str, _context: &PipelineContext) {}
    fn on_step_end(&self, _stage: &str, _step: &str, _context: &PipelineContext, _result: &StepResult) {}
}
```

Hook 不是拦截器——它**只观察不阻断**。用于日志、调试、指标收集。

### 15.5 三种管线如何复用

```
Pipeline Engine（通用编排引擎）
    ├── 实例化为 Ability Pipeline（6 stages: Validate → PreCost → ... → PostCost）
    ├── 实例化为 Modifier Pipeline（4 stages: Collect → Aggregate → Resolve → Publish）
    └── 实例化为 Combat Pipeline（TurnStart → PhaseCheck → UnitAction → ... → TurnEnd）
```

每种业务管线只需要：
1. 定义自己的 `PipelineDefinition`（stage → step 拓扑）
2. 实现各 step 的 `StepExecutor` 函数
3. 注册到 `PipelineRegistry`

同样的编排引擎、同样的失败策略、同样的日志层级，不需要重复写。

---

## 16. Tag 位掩码原理——为什么标签检查是 O(1) 的

### 16.1 位掩码结构

Tag 系统使用一个 128 位的 `BitMask` 来表示一个实体上的所有标签：

```rust
pub type BitMask = u128;
```

每个 `TagDefinition` 在注册时分配一个唯一的 `bit_index`（0-127）。因为位操作是 CPU 单指令的，所以标签包含检查永远是一个 `AND` 指令——不涉及字符串比较、不涉及树遍历。

### 16.2 层级继承

Tag 是有层级的：

```
tag_element        (bit 0)
├── tag_element_fire   (bit 1, parent: tag_element)
├── tag_element_ice    (bit 2, parent: tag_element)
└── tag_element_lightning (bit 3, parent: tag_element)
```

`is_descendant_of(tag_element_fire, tag_element)` 怎么做到 O(1)？每个 `TagDefinition` 存储了从根到自己的完整路径位掩码。继承检查就是一次按位操作：

```rust
// 检查 child 是否是 parent 的子标签
fn is_descendant_of(child: &TagDefinition, parent: &TagDefinition) -> bool {
    child.ancestor_mask & parent.bit_flag != 0
}
```

### 16.3 14 个 TagNamespace

| Namespace | 示例 |
|-----------|------|
| Character | tag_human, tag_elf, tag_undead |
| Ability | tag_ability_fireball, tag_ability_heal |
| Status | tag_status_stun, tag_status_poison |
| Equipment | tag_equip_sword, tag_equip_armor |
| Item | tag_item_potion, tag_item_scroll |
| Damage | tag_damage_fire, tag_damage_piercing |
| Terrain | tag_terrain_water, tag_terrain_lava |
| Faction | tag_faction_player, tag_faction_enemy |
| Quest | tag_quest_main, tag_quest_side |
| Combat | tag_combat_stealth, tag_combat_cover |
| Trigger | tag_trigger_on_hit, tag_trigger_on_death |
| Cue | tag_cue_explosion, tag_cue_heal |
| Domain | tag_domain_combat, tag_domain_narrative |
| Custom | 供 Mod 扩展使用 |

---

## 17. Cue 的五种表现类型

Cue 是 Effect 到表现层的信号桥梁。一个 CueDef 携带五种不同类型的参数，每种参数对应一种表现管道：

### 17.1 CueType 枚举

```rust
pub enum CueType {
    VFX(VFXParams),         // 视觉特效（粒子、光效、贴花）
    SFX(SFXParams),         // 音效
    Animation(AnimationParams),  // 骨骼动画
    Shake(ShakeParams),     // 屏幕震动
    Popup(PopupParams),     // 伤害数字/文字弹出
}
```

### 17.2 各参数详解

**VFXParams**——视觉特效：
```rust
pub struct VFXParams {
    pub effect_key: String,       // VFX 资源 key
    pub attach_point: String,     // 挂载点（如 "weapon", "head"）
    pub follow_target: bool,      // 是否跟随目标移动
    pub duration_frames: u32,     // 持续帧数
    pub scale: f32,               // 缩放
    pub color_override: Option<[f32; 4]>,  // 颜色覆盖
}
```

**SFXParams**——音效：
```rust
pub struct SFXParams {
    pub sound_key: String,       // 音频资源 key
    pub volume: f32,             // 0.0–1.0
    pub is_3d: bool,             // 3D 空间音频
    pub pitch_shift: Option<f32>, // 音高偏移
}
```

**AnimationParams**——骨骼动画：
```rust
pub struct AnimationParams {
    pub animation_name: String,  // 动画名
    pub speed: f32,              // 播放速度
    pub loop_anim: bool,         // 是否循环
    pub crossfade_frames: u32,   // 过渡帧数
}
```

**ShakeParams**——屏幕震动：
```rust
pub struct ShakeParams {
    pub intensity: f32,          // 震动强度
    pub duration_frames: u32,    // 持续帧数
    pub falloff: ShakeFalloff,   // 衰减曲线
}

pub enum ShakeFalloff { Linear, Exponential, None }
```

**PopupParams**——文字弹出：
```rust
pub struct PopupParams {
    pub text_key: String,         // Localization Key
    pub color: [f32; 4],         // 文字颜色
    pub font_size: f32,
    pub float_direction: FloatDirection,  // 浮动方向
    pub duration_frames: u32,
}

pub enum FloatDirection { Up, Down, Left, Right, Random }
```

### 17.3 CueTag——触发时机

CueDef 通过 `CueTag` 控制何时触发：

```rust
pub enum CueTag {
    OnApply,      // Effect 施加时
    OnTick,       // 周期性 Tick 时
    OnRemove,     // Effect 移除时
    OnInterrupt,  // 被打断时
    Custom(String), // 自定义时机
}
```

同一个 EffectDef 可以绑定多个 Cue，每个 Cue 有自己的 CueTag 和延迟帧数：

```rust
pub struct EffectCueBinding {
    pub cue_tag: CueTag,
    pub cue_def_id: String,
    pub delay_frames: u32,
}
```

---

## 18. Replay 兼容性设计——一切为了确定性

### 18.1 核心思路

回放系统（Replay）的核心挑战是**确定性**：同样的命令序列，必须产生完全相同的游戏状态。

Fre 的 Replay 设计围绕三条原则：

1. **所有玩家/AI 输入都走 `GameCommand`**——没有鼠标点击直接改变状态的路径
2. **RNG 使用独立种子流**——每个用途有自己的流，互不干扰
3. **Frame 作为时间单位**——回放按帧切分，帧内命令有序

### 18.2 Replay 的数据结构

```rust
/// 回放头信息（文件元数据）
pub struct ReplayHeader {
    pub schema_version: u32,       // Schema 版本（迁移检测）
    pub game_version: String,       // 游戏版本
    pub scene_id: String,           // 场景标识
    pub participants: Vec<String>,  // 参与实体列表
    pub initial_seed: u64,          // 初始种子
    pub total_frames: u64,
}

/// 回放帧——单帧的命令集合 + 种子信息
pub struct ReplayFrame {
    pub frame_number: u64,          // 帧序号（从 0 开始）
    pub commands: Vec<ReplayCommand>,// 本帧的所有命令
    pub rng_seed_offset: u64,       // 本帧的 RNG 种子偏移
    pub checksum: Option<u64>,      // 校验和（可选）
}
```

### 18.3 ReplayCommand——复制的命令

回放录制的是 `ReplayCommand`，而不是 `GameCommand`。区别在于 `ReplayCommand` 更细粒度——它记录了每一帧的每个原子操作：

```rust
pub enum ReplayCommand {
    UnitMove { unit: String, path: Vec<String> },
    UseAbility { caster: String, ability_def_id: String, target: AbilityTarget },
    UseItem { user: String, item_instance_id: String, target: Option<String> },
    SkipTurn { unit: String },
    DialogueChoice { speaker: String, choice_id: String },
    ReactionConfirm { reactor: String, trigger_def_id: String, accepted: bool },
    ConfirmTargets { caster: String, ability_def_id: String, selected_targets: Vec<String> },
    Custom { domain: String, command_type: String, params: Vec<(String, String)> },
}
```

### 18.4 RNG 独立种子流

随机数是确定性的最大敌人。Fre 的解决方案是**每个用途一个独立种子流**：

```rust
pub enum RngStream {
    Combat,    // 命中/暴击/伤害浮动
    Drop,      // 掉落/制造随机
    AI,        // AI 决策随机
    World,     // 世界事件随机
}
```

每种流从不同的种子初始化，互不干扰。`ReplayFrame.rng_seed_offset` 记录了这一帧每个流的种子偏移量，回放时从初始种子 + 偏移重建 RNG 状态。

### 18.5 Replay 系统的五个子模块

```
runtime/replay/
├── foundation/
│   ├── types.rs        # 核心类型（ReplayFrame, ReplayCommand, RngStream, RngSeeds）
│   ├── values.rs       # 值对象
│   └── error.rs        # ReplayError（5 个变体）
├── mechanism/
│   ├── recorder.rs     # 录制器——监听 CommandQueue，记录到帧
│   ├── player.rs       # 播放器——按帧重播命令
│   ├── deterministic_rng.rs  # 确定性 RNG 实现
│   └── validator.rs    # 校验器——运行时帧对比
└── events.rs           # ReplayStarted, ReplayPaused, ReplayCompleted 事件
```

---

## 19. 更多代码示例——从源码摘取的真实模式

### 19.1 try_activate—Ability 激活的完整流程

真实源码（`ability/mechanism/lifecycle.rs`）的精简映射：

```rust
pub fn try_activate(
    container: &mut ActiveAbilityContainer,
    generator: &AbilityInstanceIdGenerator,
    request: ActivationRequest,
) -> Result<AbilityInstance, ActivationIssue> {

    // 1. 冷却检查（不变量 3.3）
    if let Some(cooldown) = container.cooldowns.get(&request.spec_id) {
        if cooldown.remaining_turns > 0 {
            return Err(ActivationIssue::Failure(
                AbilityFailure::OnCooldown {
                    spec_id: request.spec_id.clone(),
                    remaining_turns: cooldown.remaining_turns,
                }
            ));
        }
    }

    // 2. 唯一实例检查（不变量 V5）
    if container.has_active_instance(&request.spec_id) {
        return Err(ActivationIssue::Error(
            AbilityError::DuplicateActivation { spec_id: request.spec_id }
        ));
    }

    // 3. 分配 Instance ID
    let instance_id = generator.next_id();

    // 4. 创建 Instance
    let instance = AbilityInstance::new(
        instance_id,
        request.spec_id,
        request.def_id,
        AbilityState::Ready,
        request.context,
    );

    // 5. 注册到容器
    container.insert_instance(instance.clone());

    // 6. 触发事件
    // （外部系统通过 observer 响应此事件）
    // commands.trigger(AbilityActivated { ... });

    Ok(instance)
}
```

注意冷却检查返回的是 `AbilityFailure`（业务规则失败），而重复激活检查返回的是 `AbilityError`（程序错误）。两种错误走不同的处理路径。

### 19.2 execute_aggregation—Modifier 聚合管线

纯函数式四阶段聚合（`aggregator/mechanism/pipeline.rs` 实际精简）：

```rust
pub fn execute_aggregation(
    modifiers: &[ModifierEntry],
    base_value: f32,
    pipeline: &CalcPipeline,
) -> AggregationResult {
    let mut stage_values: Vec<(CalcStage, f32)> = Vec::new();

    // Stage 1: Add
    let add_total = modifiers.iter()
        .filter(|m| m.op == ModifierOp::Add)
        .map(|m| m.magnitude)
        .sum::<f32>();
    let after_add = base_value + add_total;

    stage_values.push((CalcStage::Add, after_add));

    // Stage 2: Multiply（连乘！）
    let mul_total = modifiers.iter()
        .filter(|m| m.op == ModifierOp::Multiply)
        .map(|m| m.magnitude)
        .product::<f32>();
    let after_mul = after_add * mul_total;

    stage_values.push((CalcStage::Multiply, after_mul));

    // Stage 3: Override（最高优先级）
    let override_mod = modifiers.iter()
        .filter(|m| m.op == ModifierOp::Override)
        .max_by_key(|m| m.priority);
    let after_override = match override_mod {
        Some(m) => { stage_values.push((CalcStage::Override, m.magnitude)); m.magnitude }
        None => { stage_values.push((CalcStage::Override, after_mul)); after_mul }
    };

    // Stage 4: Clamp
    let final_val = after_override.clamp(pipeline.clamp_min, pipeline.clamp_max);
    stage_values.push((CalcStage::Clamp, final_val));

    AggregationResult {
        final_value: final_val,
        stage_values,
        participating_count: modifiers.len(),
        was_overridden: override_mod.is_some(),
        base_value,
    }
}
```

### 19.3 Condition 递归评估

条件树的递归求值（`condition/mechanism/evaluator.rs` 精简）：

```rust
pub fn evaluate(condition: &Condition, context: &ConditionContext) -> ConditionResult {
    match condition {
        // 叶子节点：直接检查
        Condition::TagRequirement { mode, tag_id } => {
            let has = context.tag_mask.contains(tag_id.bit_index());
            let passed = match mode {
                TagRequirementMode::Has => has,
                TagRequirementMode::Not => !has,
            };
            if passed { ConditionResult::Passed }
            else { ConditionResult::Failed { reason: format!("tag {} not found", tag_id) } }
        }

        Condition::AttributeCheck { attribute_id, comparison, value } => {
            let attr_val = context.attribute_values.get(attribute_id).copied().unwrap_or(0.0);
            let passed = match comparison {
                ComparisonOp::GreaterThan => attr_val > *value,
                ComparisonOp::LessOrEqual => attr_val <= *value,
                // ...
            };
            if passed { ConditionResult::Passed }
            else { ConditionResult::Failed { reason: format!("attr {} check failed", attribute_id) } }
        }

        // 组合节点：递归
        Condition::And(conditions) => {
            for c in conditions {
                if let ConditionResult::Failed { .. } = evaluate(c, context) {
                    return ConditionResult::Failed { reason: "And condition failed".into() };
                }
            }
            ConditionResult::Passed
        }

        Condition::Or(conditions) => {
            for c in conditions {
                if let ConditionResult::Passed = evaluate(c, context) {
                    return ConditionResult::Passed;
                }
            }
            ConditionResult::Failed { reason: "Or condition failed".into() }
        }

        Condition::Not(c) => match evaluate(c, context) {
            ConditionResult::Passed => ConditionResult::Failed { reason: "negated".into() },
            ConditionResult::Failed { .. } => ConditionResult::Passed,
        },
    }
}
```

短路求值——`And` 遇到第一个 Fail 就返回，`Or` 遇到第一个 Pass 就返回。
