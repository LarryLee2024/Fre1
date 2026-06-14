# ADR-022: Buff 触发系统与事件架构

## 状态

Proposed

## 背景

当前 Buff 系统的功能触发通过硬编码实现：DoT/HoT 伤害在 `resolve_status_effects` 中直接执行扣血/回血逻辑，stun 检查在 `intent.rs` 和 `resolve.rs` 两处分散判断，缺乏统一的 Trigger 注册和分发机制。`docs/01-architecture/skill-buff-abstraction.md` §4.8 定义了 11 种 Trigger 时机和 TriggerRegistry，但当前代码库未实现。

本 ADR 定义 Buff 触发系统的统一架构，包括 TriggerRegistry、TriggerHandler trait、TriggerContext、与 Effect Pipeline 的衔接。

## 引用的领域规则

- `docs/01-architecture/skill-buff-abstraction.md` — §4.8 Trigger 系统、§4.8.1 Stack 执行栈、§4.8.2 TriggerRegistry
- `docs/02-domain/trigger/trigger-rules.md` — 触发器、事件链（伤害→护盾→吸血→反击）
- `docs/01-architecture/events_audit_design.md` — 事件审计、独立 Struct 事件
- `docs/01-architecture/command_bus_design.md` — ActionQueue 效果执行队列

## 决策

### 1. TriggerRegistry（触发器注册表）

所有 Buff 触发的时机通过 `TriggerRegistry` 统一注册，禁止分散在 System 中硬编码判断：

```rust
/// 触发器注册表 — 所有 Trigger Handler 统一注册
#[derive(Resource, Default)]
pub struct TriggerRegistry {
    handlers: HashMap<Trigger, Vec<TriggerHandlerEntry>>,
}

struct TriggerHandlerEntry {
    handler: Box<dyn TriggerHandler>,
    priority: i32,
}
```

### 2. Trigger 枚举

与 `docs/01-architecture/skill-buff-abstraction.md` §4.8 定义的 11 种触发时机对齐：

```rust
/// 触发时机枚举
pub enum Trigger {
    TurnStart,       // 回合开始时
    TurnEnd,         // 回合结束时
    BeforeAttack,    // 攻击前（判断是否可攻击）
    AfterAttack,     // 攻击后（如吸血、连击）
    BeforeDamaged,   // 受伤前（如护盾吸收）
    AfterDamaged,    // 受伤后（如荆棘反伤）
    BeforeMove,      // 移动前
    AfterMove,       // 移动后
    KillTarget,      // 击杀目标时
    Death,           // 死亡时
    BattleStart,     // 战斗开始
    BattleEnd,       // 战斗结束
}
```

### 3. TriggerHandler trait

```rust
/// Trigger Handler trait — 每种触发器的处理逻辑
pub trait TriggerHandler: Send + Sync + 'static {
    /// 触发器类型
    fn trigger_type(&self) -> Trigger;

    /// 处理触发事件，返回要执行的 Effect 列表
    fn handle(&self, ctx: &TriggerContext) -> Vec<EffectDef>;

    /// 触发优先级（决定同 Tick 内的执行顺序）
    fn priority(&self) -> i32 { 0 }
}
```

### 4. TriggerContext（触发上下文）

```rust
/// 触发上下文 — 封装一次触发所需的全部数据
pub struct TriggerContext {
    /// 触发时机
    pub trigger: Trigger,
    /// 来源实体（谁造成的触发）
    pub source: Entity,
    /// 目标实体（谁被触发影响）
    pub target: Entity,
    /// 技能 ID（如有）
    pub skill_id: Option<String>,
    /// 造成的伤害量（AfterAttack/AfterDamaged 需要）
    pub damage_dealt: Option<i32>,
    /// 是否暴击
    pub is_critical: bool,
    /// 地形 ID
    pub terrain_id: String,
}
```

### 5. 触发分发流程

```
游戏事件（如单位受到伤害）
    ↓
TriggerDispatcher System
    ├── 读取 TriggerRegistry 获取所有 AfterDamaged Handler
    ├── 按 priority 排序
    ├── 逐个调用 handler.handle(ctx)
    ├── 收集 EffectDef[] 
    └── 压入 EffectQueue → Effect Pipeline（Generate → Modify → Execute）
```

### 6. 与 Effect Pipeline 的衔接

Trigger 产生的 Effect 必须通过 Effect Pipeline 执行，禁止直接修改状态：

```
Trigger 触发
    ↓
handler.handle(ctx) → Vec<EffectDef>
    ↓
EffectQueue.push()（推入待处理队列）
    ↓
Effect Pipeline: Generate → Modify → Execute
    ↓
EffectResult
```

### 7. 当前代码的迁移路径

当前分散在各处的触发逻辑：

| 当前实现 | 目标 | 迁移步骤 |
|---------|------|----------|
| `resolve.rs` DoT 直接扣血 | OnTurnStart Trigger → Effect Pipeline | 1. 定义 Trigger::TurnStart 2. 注册 TriggerHandler 3. `resolve_status_effects` 改为触发 TurnStart |
| `resolve.rs` HoT 直接回血 | OnTurnStart Trigger → Effect Pipeline | 同上 |
| `intent.rs` stun 检查（`tags.has(STUN)`） | OnBeforeAttack Trigger → Guard 检查 | 1. 定义 Trigger::BeforeAttack 2. `can_use()` 增加 Stun 检查 |
| `execute.rs` OnHit/OnKill Trait 触发 | TriggerRegistry → 统一分发 | OnHit/OnKill 已部分实现（trait_trigger.rs），逐步迁移到 TriggerRegistry |

### 8. 分阶段实施计划

| Phase | 范围 | 说明 |
|-------|------|------|
| Phase 1 | TriggerRegistry + TriggerHandler trait | 基础设施，不影响现有逻辑 |
| Phase 2 | TurnStart/TurnEnd Handler | 替代 resolve.rs 中的 DoT/HoT 硬编码 |
| Phase 3 | AfterAttack/AfterDamaged Handler | 替代 trait_trigger.rs 中的部分逻辑 |
| Phase 4 | 所有 Trigger 覆盖 + 旧代码清理 | |

## Module Design

### 新增文件

```
src/core/buff/
├── trigger.rs         ← TriggerRegistry, TriggerHandler trait, Trigger 枚举
├── handlers/
│   ├── mod.rs         ← Handler 注册入口
│   ├── dot.rs         ← OnTurnStart DoT Handler
│   ├── hot.rs         ← OnTurnStart HoT Handler
│   └── stun.rs        ← OnBeforeAttack Stun Handler（可选）
```

### TriggerPlugin 注册

```rust
/// 触发系统插件（独立于 BuffPlugin，允许按需注册）
pub struct TriggerPlugin;

impl Plugin for TriggerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TriggerRegistry>()
            .add_systems(OnEnter(TurnPhase::SelectUnit), dispatch_turn_start_triggers)
            .add_systems(OnEnter(TurnPhase::TurnEnd), dispatch_turn_end_triggers);
    }
}
```

## Communication Design

### Message

| Message | 发送方 | 接收方 | 用途 |
|---------|--------|--------|------|
| `shared::event::buff::BuffApplied` | apply_buff 调用者 | UI、日志、回放 | Buff 已施加 |
| `shared::event::buff::BuffRemoved` | resolve.rs | UI、日志、回放 | Buff 已移除 |
| `shared::event::battle::DamageDealt` | Effect Pipeline Execute 阶段 | Trigger 分发 | 伤害事件触发 AfterDamaged |
| `shared::event::battle::HealApplied` | Effect Pipeline Execute 阶段 | Trigger 分发 | 治疗事件触发 AfterHealed |

### 函数调用

- `TriggerRegistry.dispatch(trigger, ctx)` — 统一分发入口
- `handler.handle(ctx)` — 具体处理逻辑
- `TriggerDispatcher` — ECS System

### Trigger 与 Exisiting Trait System 的关系

当前 `trait_trigger.rs` 中已实现 `trigger_on_attack_traits`、`trigger_on_hit_traits`、`trigger_on_kill_traits`。这些是 Trait 系统的触发机制，与 Buff 的 Trigger 系统**相互独立**：

```
Trait 触发 → TraitEffectHandlerRegistry → EffectQueue
Buff 触发  → TriggerRegistry → Vec<EffectDef> → EffectQueue
```

两套系统最终都汇聚到 EffectQueue，进入 Effect Pipeline。未来可考虑统一为一个 TriggerRegistry。

## 边界定义

- 允许：`core/buff/trigger.rs` 定义 Trigger 枚举和 TriggerRegistry
- 允许：`core/buff/trigger.rs` 依赖 `core/effect/`（EffectDef、EffectQueue）
- 允许：Trigger 分发系统在 `TurnPhase::SelectUnit` 和 `TurnPhase::TurnEnd` 阶段运行
- 禁止：TriggerHandler 直接修改 ECS World 状态（必须返回 EffectDef，走 Effect Pipeline）
- 禁止：Trigger 在 OnEnter 阶段同一帧内递归触发超过 MAX_STACK_DEPTH（32 层）
- 禁止：将 Trait 触发和 Buff 触发合并为同一个 Handler 类型（目前保持分离，未来再议）

## Forbidden（禁止事项）

- 🟥 禁止：TriggerHandler 直接修改 HP/MP 或任何 ECS 状态 — 理由：必须通过 Effect Pipeline
- 🟥 禁止：Trigger 在 Execute 阶段直接扣血（如荆棘反伤在 Execute 阶段插入新 Effect）— 理由：管线顺序
- 🟥 禁止：同帧内递归触发超过 32 层 — 理由：防止栈溢出
- 🟥 禁止：Trigger 硬编码在 System 中（如 `if tags.has(STUN) { unit.acted = true }`）— 理由：统一触发管理
- 🟥 禁止：TriggerHandler 的 handle() 中包含异步操作 — 理由：同步确定性执行
- 🟥 禁止：Trigger 分发跳过 Effect Pipeline 的 Modify 阶段 — 理由：修饰规则完整性
- 🟥 禁止：为每个新增 Buff 类型编写 TriggerHandler（应通过配置数据表达触发条件）— 理由：Rule/Content 分离

## Definition / Instance Design

- Definition：Trigger 枚举、TriggerHandler trait、TriggerRegistry（Resource）
- Instance：TriggerContext（每次触发生成）、EffectDef 列表（Handler 产出）

## 后果

### 正面
- 统一 Trigger 分发消除分散在 System 中的硬编码触发检查
- TriggerContext 提供标准化上下文，消除"吸血需要 damage_dealt"等问题
- 与 Effect Pipeline 的固定衔接保证修饰规则完整性
- 分阶段迁移路径平滑

### 负面
- TriggerRegistry 增加一个全局 Resource
- 初期 Handler 数量少时显得过度设计
- 当前 trait_trigger.rs 与 TriggerRegistry 的拆分增加短期维护成本

## 替代方案

| 方案 | 优点 | 缺点 | 为何放弃 |
|------|------|------|----------|
| 保持现状（硬编码触发） | 简单 | 每次新增触发都要修改核心 System | 违反开闭原则 |
| Observer 替代 TriggerRegistry | 低耦合 | 无法控制执行优先级、难以调试 | TriggerRegistry 更可控 |
| 完全移除 trait_trigger.rs | 整洁 | 破坏现有 Trait 系统 | 分阶段迁移更安全 |
