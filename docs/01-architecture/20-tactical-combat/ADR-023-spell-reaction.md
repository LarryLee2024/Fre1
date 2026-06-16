---
id: 01-architecture.ADR-023
title: ADR-023 — Spell & Reaction System Design
status: proposed
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-023: 法术与反应机制设计

## 状态

**Proposed** — 依赖 ADR-010（Ability Pipeline）、ADR-020（Combat Pipeline）和 ADR-021（Turn State Machine）。

## 背景

法术是 Ability 的一种特殊形式，有独立的资源系统（MP/SP）和施法流程。反应机制（Reaction）包括护盾、反击、吸血、拦截等"在特定条件下自动触发的效果"。两者都属于 Layer 3（Combat Execution），依赖 Layer 2 的能力系统。

## 引用的领域规则与数据架构

- `docs/02-domain/spell_domain.md` — Spell 领域规则
- `docs/02-domain/reaction_domain.md` — Reaction 领域规则
- `docs/04-data/domains/spell_schema.md` — Spell Schema
- `docs/04-data/domains/reaction_schema.md` — Reaction Schema
- `.trae/rules/SRPG专项规则.md` §一（战斗事件链）、§六（Buff 生命周期）
- `docs/04-data/README.md` — Data Law 004、005

## 决策

### 1. Spell 架构

#### 1.1 Spell 与 Ability 的关系

Spell 是 Ability 的子类型 + 扩展：

```
Spell = Ability (Cost + Cooldown + Targeting + Effects)
      + ManaCost (新增资源消耗)
      + CastTime (施法时间，可能跨回合)
      + SpellSchool (法术学派，用于抗性/专精)
      + SpellTags (标签：飞行/投射/瞬发/引导)
```

**实现策略**：Spell 不是一个独立的执行管线，它复用了 Ability Pipeline（ADR-010），只是在其基础上增加了 Spell 特有的前置检查：

```
SpellCastIntent (Event)
       │
       ▼
SpellValidationSystem
  ├── Mana/SP 是否充足？
  ├── CastTime 条件满足？
  ├── Silenced/Stunned 状态？
  └── SpellSchool 是否可用？
       │
       ▼
进入 Ability Pipeline (ADR-010)
  Phase 1: Validate (含 Spell 特有校验)
  Phase 2: PreCost (含 Mana 扣除)
  Phase 3-5: Target → Execute → Resolve (同 Ability)
  Phase 6: PostCost (含 Spell 冷却)
```

#### 1.2 Spell 数据结构

```rust
/// SpellDef — 配置文件加载
#[derive(Asset, TypePath)]
pub struct SpellDef {
    pub id: SpellDefId,
    pub base_ability: AbilityDefId,     // 复用的 Ability（包含 effects/targeting）
    pub mana_cost: ManaCost,
    pub cast_time: CastTime,             // 0 = 瞬发, >0 = 需要准备
    pub school: SpellSchool,
    pub tags: SpellTags,
    pub charge_limit: Option<u32>,       // 战斗中可使用次数
}

/// ManaCost — 支持多种资源
pub struct ManaCost {
    pub mp_cost: u32,
    pub sp_cost: u32,                    // 特殊资源
    pub hp_cost: Option<u32>,            // 血魔法
    pub other_costs: Vec<ResourceCost>,  // 物品、献祭等
}

/// SpellInstance — 运行时状态
#[derive(Component)]
pub struct SpellInstance {
    pub spell_def_id: SpellDefId,
    pub caster: Entity,
    pub state: SpellState,
    pub cast_progress: f32,              // CastTime 进度
    pub charges_remaining: u32,
}

pub enum SpellState {
    Ready,
    Casting { remaining: f32 },
    Cooldown { remaining: f32 },
}
```

#### 1.3 法术施放流程

```
玩家/AI 选择法术 → 选择目标
       │
       ▼
SpellCastRequest (Event)
       │
       ▼
SpellValidation → 失败？ → 反馈错误，不消耗资源
       │ 成功
       ▼
Mana扣除（如果瞬发）/ 或进入引导状态
       │
       ├── 瞬发 → 立即进入 Ability Pipeline
       │
       └── 引导 → 创建 SpellCasting Component
              │
              ▼
        每帧更新 CastProgress
              │
        完成？→ 进入 Ability Pipeline
```

### 2. Reaction 架构

#### 2.1 Reaction 的定义

Reaction 是"在某事件发生时，如果满足条件，自动触发"的机制。它与 Trigger 的区别：

| | Trigger | Reaction |
|--|---------|----------|
| 定义位置 | EffectDef 中的附属项 | 独立的 ReactionDef |
| 触发条件 | 由 Effect 本身决定 | 由 ReactionDef.conditions 独立定义 |
| 目标 | 通常为 Effect 同一目标 | 可以指定不同目标（如护盾给友方） |
| 优先级 | 全局配置 | 可以 per-reaction 配置 |
| 冷却 | 共享触发冷却 | 独立冷却 |
| 典型场景 | DOT 每回合触发伤害 | 护盾吸收伤害、反击攻击者 |

#### 2.2 Reaction 数据结构

```rust
/// ReactionDef — 配置文件加载
#[derive(Asset, TypePath)]
pub struct ReactionDef {
    pub id: ReactionDefId,
    pub trigger_event: ReactionTrigger,   // OnDamageTaken | OnDamageDealt | OnHeal | OnDeath | OnMove
    pub conditions: Vec<ConditionDef>,    // 触发条件
    pub effects: Vec<EffectDefId>,        // 触发后执行的效果
    pub priority: ReactionPriority,       // 执行优先级（护盾先于吸血）
    pub cooldown: Option<CooldownDef>,    // 反应冷却
    pub stacks: Option<u32>,              // 每回合触发次数限制
    pub target_override: TargetOverride,  // 目标覆盖（默认=触发者）
}

/// ReactionInstance — 运行时
#[derive(Component)]
pub struct ReactionInstance {
    pub reaction_def_id: ReactionDefId,
    pub owner: Entity,               // 拥有此反应的实体
    pub remaining_stacks: u32,
    pub cooldown_remaining: f32,
}
```

#### 2.3 Reactions 执行流程

```
DamageEvent (或其他触发事件)
       │
       ▼
ReactionScanner (System)
  ├── 扫描所有持有 ReactionInstance 的 Entity
  ├── 过滤匹配 trigger_event 的 Reaction
  ├── 按 priority 排序
  └── 检查冷却/堆叠/条件
       │
       ▼
ReactionActivated (Event)
       │
       ▼ （对每个激活的 Reaction）
ReactionExecutionSystem
  ├── 克隆 EffectDef → 创建 EffectInstance
  ├── 按 target_override 修正目标
  ├── 进入 Effect Pipeline
  └── 扣除堆叠/启动冷却
       │
       ▼
可能触发更多 Reaction（递归）
  └── 递归深度上限：10
```

#### 2.4 常见 Reaction 模式

| 反应类型 | trigger_event | 效果 | 优先级 |
|---------|--------------|------|--------|
| 护盾 | OnDamageTaken | 减少受到的伤害 | High (100) |
| 吸血 | OnDamageDealt | 治疗攻击者 | Medium (50) |
| 反击 | OnDamageTaken | 对攻击者造成伤害 | Medium (50) |
| 魔法护盾 | OnSpellHit | 按比例减少法术伤害 | High (90) |
| 自动治疗 | OnTurnStart | 恢复 HP | Low (10) |
| 死亡爆炸 | OnDeath | 对周围造成伤害 | Low (0) |

#### 2.5 递归防护

```rust
/// 反应链递归深度追踪 — 防止无限循环
#[derive(Resource, Default)]
pub struct ReactionDepthTracker {
    depth: u32,
    chain_history: Vec<ReactionChainEntry>,
}

impl ReactionDepthTracker {
    const MAX_DEPTH: u32 = 10;

    fn try_enter(&mut self, reaction: &ReactionDef) -> Result<(), ReactionError> {
        if self.depth >= Self::MAX_DEPTH {
            return Err(ReactionError::MaxDepthExceeded {
                depth: self.depth,
                reaction_id: reaction.id,
                history: self.chain_history.clone(),
            });
        }
        self.depth += 1;
        self.chain_history.push(ReactionChainEntry {
            reaction_id: reaction.id,
            frame: current_frame(),
        });
        Ok(())
    }

    fn exit(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }
}
```

### 3. Buff 生命周期集成

Buff 通过 Effect 创建，生命周期管理集成到回合状态机（ADR-021）中：

```
Effect[AddBuff]
       │
       ▼
BuffEffectResolution
  ├── 创建 BuffInstance Component
  ├── 设置 duration（回合制）
  ├── 添加 Modifier（属性变化）
  └── 注册 OnTurnStart/OnTurnEnd 响应
       │
       ▼
OnTurnEnd (来自 TurnSubState::TurnSettlement)
       │
       ▼
BuffTickSystem
  ├── duration.remaining -= 1
  └── duration.remaining == 0？
         │
         ├── Yes → 触发 BuffExpired → 清理 Modifier
         └── No  → 继续
```

## Module Design

```
src/core/domains/spell/
  ├── plugin.rs              — SpellPlugin
  ├── components.rs          — SpellInstance, ManaPool (Component)
  ├── systems.rs             — spell_validation, mana_deduction, cast_progress
  ├── events.rs              — SpellCastRequest, SpellCastResult
  └── api.rs                 — SpellDef, SpellSchool

src/core/domains/reaction/
  ├── plugin.rs              — ReactionPlugin
  ├── components.rs          — ReactionInstance, ReactionCooldown
  ├── systems.rs             — reaction_scanner, reaction_executor
  ├── resources.rs           — ReactionDepthTracker
  └── events.rs              — ReactionActivated
```

## Communication Design

| 通信 | 机制 | 方向 |
|------|------|------|
| Spell 请求 → Validation | Event (`SpellCastRequest`) | input/ability → spell |
| Spell → Ability Pipeline | 函数调用复用 | spell → ability |
| 战斗事件 → Reaction | Event (`DamageEvent`) | combat → reaction |
| Reaction → Effect | Trigger (`ReactionActivated`) | reaction → effect |
| Buff → Turn 生命周期 | Observer (`OnTurnEnd`) | buff → turn_phase |

## 边界定义

### 允许
- Spell 复用 Ability Pipeline 的执行逻辑
- Reaction 通过 Event 监听战斗事件并自动触发
- Buff 生命周期绑定到回合阶段
- Reaction 链深度不超过 10

### 🟥 禁止
- Spell 绕过 Mana 检查直接执行效果
- Reaction 直接修改 Health/Mana（必须通过 Combat/Effect Pipeline）
- Reaction 触发自身导致无限循环（深度保护）
- Buff 使用独立的时间系统（必须绑定回合阶段）
- Reaction 阻塞主游戏流程（Reaction 应是同步的）

## Forbidden

| 禁止行为 | 理由 |
|---------|------|
| Spell 绕过 Mana 检查 | 资源系统不一致 |
| Reaction 中直接扣血/加血 | 必须经过 Effect Pipeline |
| 无限 Reaction 链 | 必须设深度上限 10 |
| Buff 使用 wall-clock 时间 | 违反回合制绑定 |
| Spell 独立执行管线 | 复用 Ability Pipeline |

## Definition / Instance Design

- **Definition**: `SpellDef` (Asset), `ReactionDef` (Asset), `BuffDef` (Asset)
- **Instance**: `SpellInstance` (Component), `ReactionInstance` (Component), `BuffInstance` (Component), `ManaPool` (Component)
- **Persistence**: `SpellInstance.cooldown`, `ReactionInstance.remaining_stacks`, `BuffInstance.duration`

## 后果

### 正面
- Spell 复用 Ability Pipeline，不重复实现执行逻辑
- Reaction 系统通过 Event 监听，松耦合
- Buff 生命周期绑定回合阶段，时序清晰
- 递归深度保护防止无限循环

### 负面
- ReactionScanner 需要每帧扫描所有 ReactionInstance，Entity 多时可能有性能压力
- Spell 和 Ability 的边界在实际使用中可能模糊（"一个技能既有 Spell 属性又没有"）

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| Spell 是独立 Feature，不依赖 Ability | 大量代码重复，违反组合原则 |
| Reaction 用 Trigger 驱动 | Trigger 绑定 Entity 不适合跨 Entity 的反应 |
| Buff 使用独立 Timer Component | 回合制游戏应绑定回合数，而非 real time |
| Spell 作为 Ability 的一个字段标记 | Ability 不应意識到 Spell 的存在 |

## 评审要点

- [ ] Spell 和 Ability 的区分是否足够清晰？什么情况应该用 Spell 而不是 Ability？
- [ ] Reaction 的优先级系统——是否应该支持打断（更高优先级 Reaction 可以阻止低优先级执行）？
- [ ] 反应链最大深度 10——极端情况下（10 个连锁护盾）是否合理？
- [ ] 引导型法术（CastTime > 0）被打断的流程是否完善？
