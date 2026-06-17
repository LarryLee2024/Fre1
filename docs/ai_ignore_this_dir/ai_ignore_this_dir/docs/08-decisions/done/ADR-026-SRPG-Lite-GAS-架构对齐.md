# ADR-026: SRPG Lite-GAS 架构对齐

## 状态

**Complete** ✅

> 2026-06-15: Feature Developer 已完成全部 13 模块实现，750 测试通过。

## 背景

项目已完成七领域模块化架构（ADR-025），将战斗能力系统拆分为 Tag、Modifier、Buff、Effect、Targeting、Ability、Trigger 七个独立领域。经深度评审 SRPG Lite-GAS 冻结架构方案（`docs/其他/77.md`），发现当前七领域架构存在以下结构性缺陷：

| 问题 | 当前状态（ADR-025） | 后果 |
|------|---------------------|------|
| Buff 作为独立顶层模块 | Buff 与 Effect 平行，职责重叠 | 跨模块依赖冗余、生命周期管理重复、AI 生成代码需维护两套规则 |
| 缺少 Execution 层 | Effect 内部直接写公式逻辑 | 新增伤害类型需修改 Effect 代码、巨型 match 分支膨胀 |
| 缺少 Cue 表现层 | 逻辑与表现未严格解耦 | 业务代码直接调用 UI/特效，违反宪法 Logic/Presentation 分离 |
| Attribute 非一级模块 | 属性散落在 modifier_rule 和 character 中 | `attack += 10` 硬编码散落，无统一基础/派生属性管线 |
| Stacking 策略不完整 | ADR-021 定义 3 种 StackPolicy（NoStack/Replace/Add） | 缺少 RefreshDuration 和 StackMax(u32)，无法覆盖中毒叠层、护盾叠加、Boss 光环等场景 |
| 缺少 Registry 注册中心 | 各模块独立注册，无统一入口 | 新增技能/效果需改代码，无法纯配置扩展 |
| 缺少 Replay 回放基建 | 无确定性回放机制 | 复杂 Buff/Trait/Modifier 出 Bug 时无法回溯调试 |

### 当前架构（七领域）

```
Tag → Modifier → Effect → Buff → Ability
                  ↓        ↑
              Targeting  Trigger
```

### 目标架构（SRPG Lite-GAS 冻结版 · 13 模块）

```
Attribute → Tag → Modifier → Effect → Ability → Trigger → Targeting
                                          ↓
                              Stacking → Execution → Cue → Replay
```

**核心变更**：
1. Buff 从顶层删除，统一为带 Duration 的 Effect
2. 新增 Execution（trait-based 公式执行层）
3. 新增 Cue（统一表现事件总线）
4. Attribute 升级为一级领域（Primary/Derived 双分层）
5. Stacking 升级为 4-enum 模型
6. 新增 Registry（资产统一注册中心）
7. 新增 Replay（确定性战斗回放基建）

## 引用的领域规则

- `docs/02-domain/battle/battle-rules.md` — 战斗状态机、Effect Pipeline、伤害计算
- `docs/02-domain/character/character-rules.md` — 角色属性、Faction、UnitSnapshot
- `docs/02-domain/skill/skill-rules.md` — 技能定义、冷却、五阶段释放管线
- `docs/02-domain/attribute-modifier/attribute-modifier-rules.md` — Modifier 管线、属性修饰、叠加规则
- `docs/02-domain/turn/turn-rules.md` — TurnPhase、回合阶段、行动队列
- `docs/02-domain/trigger/trigger-rules.md` — 触发器、事件链
- `docs/01-architecture/01-battle-gas/skill-buff-abstraction.md` — v2.0 Effect 一级领域化
- `docs/01-architecture/README.md` — 七层架构总纲
- `docs/00-governance/ai-constitution-complete.md` — AI 开发宪法 v1.6

## 决策

### 一、采用 10+3 模块架构

将战斗能力系统从七领域扩展为 **10 业务领域 + 3 基础设施**，共 13 个核心模块：

#### 业务领域层（Gameplay · 无引擎依赖 · DDD 边界）

| 序号 | 模块 | 职责 | 变更说明 |
|------|------|------|----------|
| 1 | **Attribute** | 基础/派生属性全域体系 | 新增为一级领域（原散落在 modifier/character） |
| 2 | **Tag** | 游戏标签体系（GAS 核心） | 保留 |
| 3 | **Modifier** | 属性修改器单元 | 保留 |
| 4 | **Effect** | 通用效果总层（含原 Buff 全部能力） | 扩展：吸收 Buff 的 Duration 语义 |
| 5 | **Ability** | 战斗技能领域 | 保留 |
| 6 | **Trigger** | 被动触发事件体系 | 保留 |
| 7 | **Targeting** | 目标选取领域 | 保留 |
| 8 | **Execution** | 效果执行算式层 | **新增**：trait-based 公式执行 |
| 9 | **Stacking** | 效果堆叠规则中心 | 升级：4-enum 模型 |
| 10 | **Cue** | 表现层信号总线 | **新增**：统一表现事件 |

#### 底层基建层（Infrastructure · 全局公共 · 业务无感）

| 序号 | 模块 | 职责 | 变更说明 |
|------|------|------|----------|
| 11 | **Registry** | 资产统一注册中心 | **新增**：技能/效果/算式/标签全局注册 |
| 12 | **Pipeline** | 回合战斗执行管线 | 保留（从 Effect Pipeline 升级为全局管线） |
| 13 | **Replay** | 确定性战斗回放基建 | **新增**：指令+种子快照持久化 |

### 二、删除 Buff 顶层模块

Buff 不再作为独立顶层领域，统一为带 Duration 的 Effect：

| 原 Buff 类型 | 新 Effect 表达 | Duration |
|-------------|----------------|----------|
| 瞬时伤害/治疗 | `Effect::ApplyModifier(...)` | `Duration::Instant` |
| 回合 Buff（灼烧/眩晕/攻速增减） | `Effect::ApplyModifier(...)` | `Duration::TurnLimited(u32)` |
| 永久常驻（装备属性/职业天赋/被动光环） | `Effect::ApplyModifier(...)` | `Duration::Permanent` |

**设计理由**：
- Buff 本质是「带时长的 Effect」，无需独立生命周期管理
- 减少跨模块依赖、减少 Plugin 数量、减少边界冲突
- 堆叠、时长、结算全部复用 Effect 原生管线
- AI 生成代码规则统一，无需维护两套生命周期

### 三、新增 Execution 域（trait-based 公式执行层）

所有伤害/治疗/地形/百分比数值计算，全部抽离为独立 Execution Trait：

```rust
pub trait Execution {
    fn execute(&self, context: &ExecutionContext) -> ExecutionResult;
}

pub struct DamageExecution;      // 普通伤害: Attack - Defense
pub struct TrueDamageExecution;  // 真实伤害: Attack
pub struct CritExecution;        // 暴击: Attack * CritMultiplier
pub struct TerrainExecution;     // 地形伤害: MaxHP * 10%
pub struct HealExecution;        // 治疗: HealPower * multiplier
pub struct ShieldExecution;      // 护盾: ShieldValue
```

**设计理由**：
- 消灭巨型 match 分支，新增伤害类型 = 新增 Execution 实现
- 数值策划可独立配公式，不侵入业务代码
- Execution 无副作用，天然适配单元测试和回放

### 四、新增 Cue 域（统一表现事件总线）

Cue 作为业务层与表现层的唯一桥梁，下发纯数据业务事件：

| Cue 事件 | 业务触发点 | 表现层响应 |
|----------|-----------|-----------|
| `CueDamage` | Execution 执行后 | 播放飘字动画 |
| `CueDeath` | Attribute HP ≤ 0 | 播放死亡动画 |
| `CueHeal` | HealExecution 执行后 | 播放绿色飘字 |
| `CueBuffApply` | Effect ApplyModifier 后 | 播放 Buff 特效 |
| `CueShield` | ShieldExecution 执行后 | 播放护盾特效 |

**设计理由**：
- 严格落地宪法「逻辑与表现强制分离」
- 业务零耦合动画/特效/UI 资源
- 表现层仅订阅 Cue，反向零依赖业务战斗逻辑

### 五、Attribute 升级为一级领域

属性系统从散落状态升级为独立模块，采用 Primary/Derived 双分层：

| 分层 | 属性 | 规则 |
|------|------|------|
| **Primary**（基础属性） | 力量、敏捷、智力、体质 | 原始只读，无运行时修改（由职业/装备配置） |
| **Derived**（派生属性） | 攻击力、防御、暴击、移速、治疗加成 | 由 Modifier 统一计算，禁止业务代码直接修改 |

**设计理由**：
- 杜绝 `attack += 10` 硬编码散落
- 职业、装备、Buff、天赋、地形、天气全部通过 Attribute 层修改
- Primary → Derived 计算链路清晰，可审计、可回放

### 六、Stacking 升级为 4-enum 模型

从 ADR-021 的 3 种 StackPolicy 升级为 4-enum 冻结模型：

```rust
pub enum StackingRule {
    Replace,           // 覆盖原有同 ID 效果
    RefreshDuration,   // 刷新剩余时长、保留层数
    StackAdd,          // 层数叠加、无上限
    StackMax(u32),     // 层数叠加、硬上限约束
}
```

| 场景 | StackingRule | 说明 |
|------|-------------|------|
| 中毒 | `StackAdd` + `StackMax(5)` | 最多叠 5 层 |
| 护盾 | `StackAdd` | 无限叠加 |
| 狂暴 | `RefreshDuration` | 刷新持续时间 |
| Boss 光环 | `Replace` | 新光环覆盖旧光环 |

### 七、GAS 执行链（冻结时序）

单向闭环业务链路，无反向依赖、无循环调用、无跨层耦合：

```text
Ability（技能定义 + 施法校验）
    ↓
Targeting（纯函数目标筛选、无副作用）
    ↓
Effect（时效定义：瞬时/回合持续/永久常驻）
    ↓
Stacking（堆叠策略匹配：覆写/刷新/叠加/上限）
    ↓
Execution（公式执行：伤害/治疗/百分比/自定义算式）
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

**链路不变量**：此顺序为冻结时序，禁止调整。任何新增业务模块不得插入链路中间。

## Module Design

### 目标文件组织

```
src/
├── core/
│   ├── attribute/              ← 领域 1：属性领域
│   │   ├── mod.rs             → AttributePlugin
│   │   ├── types.rs           → PrimaryAttr, DerivedAttr, AttributeSet
│   │   ├── calculator.rs      → DerivedCalculator（Primary → Derived 计算）
│   │   └── components.rs      → AttributeComponent（Entity 上的属性数据）
│   │
│   ├── tag/                    ← 领域 2：标签系统（保留）
│   │   ├── mod.rs             → TagPlugin
│   │   ├── types.rs           → GameplayTag(u64), TagName, TagCategory
│   │   ├── components.rs      → PersistentTags Component
│   │   └── registry.rs        → TagRegistry Resource
│   │
│   ├── modifier/               ← 领域 3：属性修饰器（保留）
│   │   ├── mod.rs             → ModifierPlugin
│   │   ├── types.rs           → ModifierRule, ModifierEntry, ModifierOp
│   │   ├── calculator.rs      → ModifierCalculator
│   │   └── registry.rs        → ModifierRuleRegistry Resource
│   │
│   ├── effect/                 ← 领域 4：统一效果层（吸收原 Buff）
│   │   ├── mod.rs             → EffectPlugin
│   │   ├── types.rs           → EffectDef, DurationDef, DurationPolicy
│   │   ├── handler.rs         → EffectHandler trait + EffectHandlerRegistry
│   │   ├── pipeline.rs        → Generate → Modify → Execute 调度
│   │   └── data.rs            → PendingEffectData, EffectResult
│   │
│   ├── ability/                ← 领域 5：战斗技能（保留）
│   │   ├── mod.rs             → AbilityPlugin
│   │   ├── types.rs           → AbilityDef, AbilityData
│   │   ├── pipeline.rs        → Requirement → Cost → Targeting → Effect → Settlement
│   │   └── registry.rs        → AbilityRegistry Resource
│   │
│   ├── trigger/                ← 领域 6：触发器系统（保留）
│   │   ├── mod.rs             → TriggerPlugin
│   │   ├── types.rs           → Trigger, TriggerContext
│   │   ├── stack.rs           → ExecutionStack Resource
│   │   └── registry.rs        → TriggerRegistry Resource
│   │
│   ├── targeting/              ← 领域 7：目标选择（保留）
│   │   ├── mod.rs             → TargetingPlugin
│   │   ├── types.rs           → TargetingType, TargetingContext
│   │   └── resolver.rs        → calculate_targets() 纯函数
│   │
│   ├── execution/              ← 领域 8：执行算式层（新增）
│   │   ├── mod.rs             → ExecutionPlugin
│   │   ├── types.rs           → ExecutionContext, ExecutionResult
│   │   ├── damage.rs          → DamageExecution, TrueDamageExecution, CritExecution
│   │   ├── heal.rs            → HealExecution
│   │   ├── shield.rs          → ShieldExecution
│   │   └── registry.rs        → ExecutionRegistry Resource
│   │
│   ├── stacking/               ← 领域 9：堆叠规则（新增为独立模块）
│   │   ├── mod.rs             → StackingPlugin
│   │   ├── types.rs           → StackingRule 枚举（4-enum 冻结版）
│   │   └── resolver.rs        → resolve_stacking() 纯函数
│   │
│   └── cue/                    ← 领域 10：表现信号总线（新增）
│       ├── mod.rs             → CuePlugin
│       ├── types.rs           → CueEvent 枚举（CueDamage, CueDeath, ...）
│       └── emitter.rs         → CueEmitter Resource
│
├── infra/
│   ├── registry/               ← 基建 11：统一注册中心（新增）
│   │   ├── mod.rs             → RegistryPlugin
│   │   ├── ability_registry.rs → AbilityRegistry（从 ability/ 迁入）
│   │   ├── effect_registry.rs  → EffectHandlerRegistry（从 effect/ 迁入）
│   │   ├── execution_registry.rs → ExecutionRegistry（从 execution/ 迁入）
│   │   └── tag_registry.rs     → TagRegistry（从 tag/ 迁入）
│   │
│   ├── pipeline/               ← 基建 12：回合战斗管线（升级）
│   │   ├── mod.rs             → BattlePipelinePlugin
│   │   └── scheduler.rs       → 回合内 System 调度
│   │
│   └── replay/                 ← 基建 13：战斗回放（新增）
│       ├── mod.rs             → BattleReplayPlugin
│       ├── record.rs          → BattleRecord, CommandEntry
│       └── player.rs          → ReplayPlayer（确定性回放）
```

### Plugin 注册顺序（DAG，禁止颠倒）

```rust
pub struct SrpgGasPlugin;

impl Plugin for SrpgGasPlugin {
    fn build(&self, app: &mut App) {
        app
        // 第 1 层：无依赖的基础设施
        .add_plugin(RegistryPlugin)
        .add_plugin(AttributePlugin)
        .add_plugin(TagPlugin)

        // 第 2 层：依赖 tag
        .add_plugin(ModifierPlugin)

        // 第 3 层：依赖 tag + modifier
        .add_plugin(EffectPlugin)

        // 第 4 层：依赖 effect
        .add_plugin(AbilityPlugin)
        .add_plugin(TriggerPlugin)
        .add_plugin(TargetingPlugin)
        .add_plugin(StackingPlugin)
        .add_plugin(ExecutionPlugin)

        // 第 5 层：依赖 execution + effect
        .add_plugin(CuePlugin)

        // 第 6 层：全局基建
        .add_plugin(BattlePipelinePlugin)
        .add_plugin(BattleReplayPlugin);
    }
}
```

### 模块依赖图（DAG，禁止循环依赖）

```
                    ┌──────────────┐
                    │  Registry    │ ← 全局注册（最底层）
                    └──────┬───────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
      ┌───────▼──────┐ ┌──▼────────┐ ┌─▼──────────┐
      │  Attribute   │ │   Tag     │ │  Replay    │
      │  (无依赖)    │ │ (无依赖)  │ │  (无依赖)  │
      └───────┬──────┘ └──┬────────┘ └────────────┘
              │           │
              └─────┬─────┘
                    │
            ┌───────▼───────┐
            │   Modifier    │ ← 依赖: attribute, tag
            └───────┬───────┘
                    │
            ┌───────▼───────┐
            │    Effect     │ ← 依赖: tag, modifier（吸收原 Buff）
            └───────┬───────┘
         ┌──────────┼──────────┬──────────┐
         │          │          │          │
   ┌─────▼─────┐ ┌──▼────┐ ┌──▼─────┐ ┌──▼────────┐
   │  Ability  │ │Trigger│ │Targeting│ │ Stacking  │
   │ 依赖:     │ │依赖:   │ │依赖: tag│ │依赖: effect│
   │effect,    │ │effect │ │         │ │           │
   │targeting, │ │tag    │ │         │ │           │
   │trigger,   │ │       │ │         │ │           │
   │stacking   │ │       │ │         │ │           │
   └───────────┘ └───────┘ └─────────┘ └───────────┘
                                  │
                          ┌───────▼───────┐
                          │  Execution    │ ← 依赖: attribute, modifier
                          └───────┬───────┘
                                  │
                          ┌───────▼───────┐
                          │     Cue       │ ← 依赖: execution, effect
                          └───────────────┘
```

**关键规则**：
- Registry、Attribute、Tag、Replay 是最底层无依赖模块
- Modifier 只依赖 Attribute + Tag
- Effect 依赖 Tag + Modifier
- Ability、Trigger、Targeting、Stacking 平行依赖 Effect
- Execution 依赖 Attribute + Modifier
- Cue 是最上层业务模块，依赖 Execution + Effect

## Communication Design

### 函数调用链（确定性执行）

```
Ability::execute()
  ├── cost::check_and_pay()              ← 函数调用
  ├── targeting::resolver::resolve()     ← 函数调用（纯函数）
  ├── effect::pipeline::execute()        ← 函数调用
  │     ├── stacking::resolve()          ← 函数调用（纯函数）
  │     ├── execution::execute()         ← 函数调用（纯函数）
  │     ├── modifier::calculator::apply() ← 函数调用（纯函数）
  │     └── attribute::calculate()       ← 函数调用（纯函数）
  ├── tag::update()                      ← 函数调用
  ├── cue::emit()                        ← 函数调用（下发表现事件）
  ├── replay::record()                   ← 函数调用（记录指令）
  └── cooldown::set()                    ← 函数调用
```

### Observer/Hook

| 组件 | Hook 事件 | 用途 |
|-----|----------|------|
| GameplayTags | on_add / on_remove | 标签变化时触发 ModifierRule 重匹配 |
| AttributeComponent | on_insert / on_modify | 属性变化时触发 Derived 属性重算 |

### Message

| Message | 发送者 | 订阅者 | 用途 |
|---------|-------|--------|------|
| `CueDamage` | Execution | UI 飘字、Trigger、Replay | 伤害通知 |
| `CueHeal` | Execution | UI 飘字、Replay | 治疗通知 |
| `CueDeath` | Attribute | UI 死亡动画、Trigger | 死亡通知 |
| `CueBuffApply` | Effect | UI Buff 图标、Trigger | 效果施加通知 |
| `AbilityCastStarted` | Ability | UI 技能面板、Replay | 技能开始 |
| `AbilityCastFinished` | Ability | UI 技能面板、Replay、Turn | 技能结束 |

### Message 设计原则

- 所有跨模块通知通过 Cue 事件总线中转
- 业务层只 emit CueEvent，不直接调用 UI/特效
- 表现层只 subscribe CueEvent，不反向依赖业务逻辑

## 边界定义

### 允许的跨模块依赖

| 目标模块 | 可以被哪些模块依赖 | 不允许被哪些模块依赖 |
|---------|------------------|-------------------|
| Registry | 所有 | 无（全局注册中心） |
| Attribute | Modifier, Execution | 无（最底层业务） |
| Tag | 所有 | 无（最底层业务） |
| Replay | 所有 | 无（最底层基建） |
| Modifier | Effect, Execution | Attribute, Tag（反向禁止） |
| Effect | Ability, Trigger, Stacking | Attribute, Tag, Modifier（反向禁止） |
| Ability | 无（它是顶层） | 所有模块 |
| Trigger | Ability | Attribute, Tag, Modifier, Effect, Targeting, Stacking |
| Targeting | Ability | Attribute, Tag, Modifier, Effect, Stacking |
| Stacking | Ability, Effect | Attribute, Tag, Modifier |
| Execution | Cue, Ability | Attribute, Tag（反向禁止） |
| Cue | 无（最上层业务） | 所有业务模块（反向禁止） |
| Pipeline | 所有业务模块（调度层） | 无 |
| Replay | 所有（记录层） | 无 |

### 禁止的循环依赖

- 🟥 Ability → Effect → Ability （禁止：循环依赖）
- 🟥 Effect → Modifier → Effect （禁止：循环依赖）
- 🟥 Trigger → Effect → Trigger （禁止：循环依赖）
- 🟥 Cue → Execution → Cue （禁止：循环依赖）

### 模块间数据流方向

```
只读数据（Definition, Registry）：
  Registry → Attribute → Tag → Modifier → Effect → Ability/Trigger/Targeting/Stacking → Execution → Cue → Replay

可变数据（Instance, Component）：
  Ability → Effect → Stacking → Execution → Modifier → Attribute → Tag → Cue → Replay
  (单向从上至下，无反向直接修改)
```

## Forbidden（禁止事项）

- 🟥 **禁止独立 Buff 顶层模块** — Buff 统一为带 Duration 的 Effect，理由：消除跨模块冗余、统一生命周期管理
- 🟥 **禁止 Effect 内部写公式** — Execution trait 统一管理所有数值计算，理由：消灭巨型 match 分支、支持独立测试
- 🟥 **禁止大型 match 分发伤害类型** — 每个伤害类型是一个独立的 Execution，理由：开闭原则、新增类型不改已有代码
- 🟥 **禁止业务代码直接调用 UI/特效** — 必须通过 Cue 事件总线，理由：宪法 Logic/Presentation 强制分离
- 🟥 **禁止依赖 UE GAS 的 Prediction/Replication/GameplayTask** — 单机 SRPG 不需要网络预测、状态同步、任务调度
- 🟥 **禁止业务代码直接修改 Derived 属性数值** — 必须通过 Modifier → Attribute 管线，理由：保证属性计算可审计、可回放
- 🟥 **禁止跳过 Execution 直接在 Effect 中计算伤害** — 理由：Execution 是唯一数值计算入口
- 🟥 **禁止 Stacking 超过 4-enum 冻结模型** — 理由：StackingRule 枚举已冻结，不可新增变体
- 🟥 **禁止在 Cue 中包含业务逻辑** — Cue 只下发纯数据事件，理由：表现层零业务依赖
- 🟥 **禁止跨链路顺序调用** — GAS 执行链时序冻结，禁止逆序或跳跃调用
- 🟥 **禁止修改 Definition 配置数据** — Definition 不可变，理由：Definition/Instance 分离原则
- 🟥 **禁止新增模块插入执行链路中间** — 链路时序冻结，理由：保证确定性执行和回放兼容

## Definition / Instance Design

| 模块 | Definition（不可变配置） | Instance（运行时状态） |
|------|------------------------|----------------------|
| Attribute | AttributeDef（RON: Primary/Derived 初始值） | AttributeComponent（Entity 上的属性数据） |
| Tag | TagDefinition, TagName enum | GameplayTags Component, PersistentTags Component |
| Modifier | ModifierRule, ModifierEntry | ModifierEntry（记录在 PendingEffectData 中） |
| Effect | EffectDef, DurationDef | PendingEffectData（瞬态），EffectResult（瞬态） |
| Ability | AbilityDef, AbilityData | AbilitySlots Component, SkillCooldowns Component |
| Trigger | TriggerDef, TriggerHandler | ExecutionStack Resource, TriggerContext（瞬态） |
| Targeting | TargetingType enum | TargetingContext（瞬态） |
| Execution | ExecutionDef | ExecutionContext（瞬态），ExecutionResult（瞬态） |
| Stacking | StackingRule 枚举 | StackingState（记录在 ActiveEffects 中） |
| Cue | CueEventDef | CueEmitter Resource |
| Registry | 各类 Definition | 各类 Registry Resource |
| Pipeline | PipelineConfig | PipelineState Resource |
| Replay | — | BattleRecord, CommandEntry |

## 后果

### 正面

- 对齐 proven GAS 模式：10+3 模块覆盖 UE GAS 七大核心思想（Ability/Effect/Attribute/Tag/Execution/Stacking/Cue），同时裁剪网络/预测/Task 等单机不需要的部分
- 模块边界更清晰：每个模块职责单一，AI 生成代码规则统一
- 内容扩展无需改代码：Registry + Execution trait 使得新增技能/效果/伤害类型 = 新增 RON 文件 + 新增 Execution 实现
- 战斗可审计可回放：单向执行链 + Replay 基建，确定性 100% 保证
- 逻辑/表现彻底分离：Cue 事件总线作为唯一桥梁，业务零耦合 UI
- 属性修改可追踪：Attribute 双分层 + Modifier 管线，杜绝 `attack += 10` 硬编码

### 负面

- 迁移工作量大：需删除独立 Buff 模块、新增 Execution/Cue/Stacking/Attribute 独立模块
- 现有 Buff 代码需重构：BuffInstance/ActiveBuffs 需迁移到 Effect 模块的 Duration 系统
- ADR-020/021/022 需更新或标记 Superseded：Buff 相关 ADR 需适配新架构
- 文档需全面更新：`docs/02-domain/buff/` 领域文档需迁移到 `effect/`
- 团队需学习 13 模块的边界规则（比 7 模块多 6 个）
- Execution trait 设计需要仔细考虑泛型和性能（热路径）

## 替代方案

| 方案 | 优点 | 缺点 | 为何放弃 |
|------|------|------|----------|
| 保持七领域（ADR-025） | 无迁移成本 | 缺少 Execution/Cue/Stacking，属性散落 | 结构性缺陷无法通过补丁修复 |
| 照搬 UE GAS 全套 | 功能完整 | 网络/预测/Task 体系对单机 SRPG 无价值、维护爆炸 | 违反「只解决当前复杂度」原则 |
| 所有模块合并为 CombatSystem | 简单 | God 模块，违反 Feature First | 违反宪法 |
| 将 Execution 并入 Effect | 减少模块数 | Effect 职责膨胀、公式与效果耦合 | 消灭不了巨型 match |
| 将 Cue 并入 UI 层 | 看似合理 | 业务层需反向依赖 UI 层，违反分层 | 违反宪法 Logic/Presentation 分离 |
| Buff 保留为子模块（非顶层） | 折中 | 仍需独立生命周期管理，本质仍是 Effect | 增加复杂度无收益 |

## 架构合规自检

- [x] 符合 ECS 约束（Component=数据, System=行为）
- [x] 符合 Plugin 注册顺序（Registry→Attribute→Tag→Modifier→Effect→Ability/Trigger/Targeting/Stacking/Execution→Cue→Pipeline→Replay）
- [x] 没有创建禁止的模块（无 components.rs/systems.rs/utils.rs）
- [x] Effect/Modifier Pipeline 没有被绕过（能力执行必须走 Pipeline）
- [x] Execution 是唯一数值计算入口（Effect 不写公式）
- [x] Tag Components 优先于 bool 字段（GameplayTags 组件）
- [x] Cue 作为唯一表现桥梁（业务不直接调用 UI）
- [x] 符合"定义与实例分离"原则（所有模块有 Definition/Instance 表）
- [x] 符合"规则与内容分离"原则（配置 RON，逻辑 Rust，新内容=新 RON 文件）
- [x] 所有禁止事项已明确列出（12 条 Forbidden）
- [x] 已检查 `docs/02-domain/` 相关文档（13 个领域规则已就位或待补充）
- [x] GAS 执行链时序冻结，无反向依赖、无循环调用
