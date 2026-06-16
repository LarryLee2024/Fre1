---
id: 01-architecture.ADR-020
title: ADR-020 — CombatIntent Pipeline (Damage/Heal)
status: proposed
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-020: CombatIntent Pipeline 设计

## 状态

**Proposed** — 依赖 ADR-010（Ability Pipeline）和 ADR-011（Modifier Pipeline）。

## 背景

战斗是 SRPG 的核心。伤害/治疗计算涉及多个领域的交叉：攻击方属性、防御方属性、地形效果、Buff/Debuff、暴击/命中判定、随机数。需要一个统一的 Combat Pipeline 来收敛所有战斗数值计算，确保：
- 所有伤害/治疗经过统一管线（编码规则 §领域不变量）
- 预览路径与执行路径分离（SRPG §8.1）
- 管线可录制、可回放、可测试

## 引用的领域规则与数据架构

- `docs/02-domain/combat_domain.md` — Combat 领域规则
- `docs/02-domain/spell_domain.md` — Spell 领域规则
- `docs/02-domain/reaction_domain.md` — Reaction 领域规则
- `docs/04-data/domains/combat_schema.md` — Combat Schema
- `docs/04-data/domains/spell_schema.md` — Spell Schema
- `docs/04-data/domains/reaction_schema.md` — Reaction Schema
- `.trae/rules/SRPG专项规则.md` §五（技能执行管线）、§八（CQRS Lite）

## 决策

### 1. Combat Pipeline 七阶段

```
Phase 1: Intent      ── 创建 CombatIntent（攻击/治疗/真伤）
       │
       ▼
Phase 2: Generate    ── 基础值计算（攻方属性 × 技能倍率 + 随机）  
       │
       ▼
Phase 3: Modify      ── Modifier 修正（Buff/Debuff/地形/士气）
       │
       ▼
Phase 4: Defend      ── 防御方结算（护甲/抗性/闪避/格挡）
       │
       ▼
Phase 5: Resolve     ── 最终数值应用（扣血/回血）
       │
       ▼
Phase 6: React       ── 反应链触发（护盾/吸血/反击）
       │
       ▼
Phase 7: Finalize    ── 后处理（死亡检查/日志/Cue）
```

### 2. 核心数据结构

```rust
/// CombatIntent — 管线的输入（轻量，只含必要引用）
#[derive(Event)]
pub struct CombatIntent {
    pub intent_type: IntentType,   // Attack | Heal | TrueDamage
    pub source: Entity,            // 攻击者
    pub target: Entity,            // 目标
    pub ability_spec: Entity,      // 触发此 Combat 的 AbilitySpec Entity
    pub position: Option<Vec2>,    // 攻击位置（用于地形判定）
    pub preview_only: bool,        // 预览模式（不产生副作用）
}

/// CombatContext — 管线中传递的完整上下文
pub struct CombatContext {
    pub intent: CombatIntent,

    // Phase 2: Generate 结果
    pub base_damage: f32,
    pub hit_chance: f32,
    pub crit_chance: f32,
    pub is_critical: bool,

    // Phase 3: Modify 结果
    pub damage_multiplier: f32,
    pub damage_flat_bonus: f32,
    pub penetration: f32,

    // Phase 4: Defend 结果
    pub mitigated_damage: f32,
    pub is_dodged: bool,
    pub is_blocked: bool,

    // Phase 5: Resolve 结果
    pub final_damage: f32,
    pub actual_hp_delta: f32,

    // 随机种子
    pub rng_seed: u64,
}

/// CombatResult — 管线输出（结算完成后发布）
#[derive(Event)]
pub struct CombatResult {
    pub context: CombatContext,
    pub reactions_triggered: Vec<ReactionEvent>,
    pub cues: Vec<CueSignal>,
    pub did_kill: bool,
}
```

### 3. 管线阶段详解

#### Phase 1: Intent — 创建意图

```rust
fn create_combat_intent(
    mut ability_events: EventReader<AbilityExecuteEvent>,
    mut intent_writer: EventWriter<CombatIntent>,
) {
    for ev in ability_events.read() {
        if let Some(combat_effect) = ev.get_combat_effect() {
            intent_writer.send(CombatIntent {
                intent_type: combat_effect.intent_type,
                source: ev.caster,
                target: ev.primary_target,
                ability_spec: ev.ability_entity,
                position: ev.target_position,
                preview_only: ev.preview_mode,
            });
        }
    }
}
```

#### Phase 2: Generate — 基础值生成

```rust
fn generate_combat_values(
    mut pipeline: ResMut<CombatPipeline>,
    mut reader: EventReader<CombatIntent>,
    stats: Query<&AttributeSet>,
) {
    for intent in reader.read() {
        let atk_stat = stats.get(intent.source)
            .map(|s| s.get(AttributeId::ATTACK))
            .unwrap_or(1.0);
        let power = intent.get_ability_power();
        let base = atk_stat * power;

        let context = CombatContext {
            intent: intent.clone(),
            base_damage: base,
            // ... 初始化默认值
        };
        pipeline.push(context);
    }
}
```

#### Phase 3: Modify — 修正阶段

```rust
fn modify_combat_values(
    mut pipeline: ResMut<CombatPipeline>,
    modifier_query: Query<&ModifierSet>,
) {
    for ctx in pipeline.iter_mut(Phase::Modify) {
        // 攻击方的伤害加成
        if let Ok(modifiers) = modifier_query.get(ctx.intent.source) {
            ctx.damage_multiplier = calculate_bonus(modifiers, "damage_dealt");
            ctx.damage_flat_bonus = calculate_flat(modifiers, "flat_damage");
        }
        // 检查地形加成
        if let Some(terrain) = ctx.position.and_then(get_terrain) {
            ctx.damage_multiplier *= terrain.attack_bonus;
        }
    }
}
```

#### Phase 4: Defend — 防御结算

```rust
fn defend_combat_values(
    mut pipeline: ResMut<CombatPipeline>,
    stats: Query<&AttributeSet>,
    modifier_query: Query<&ModifierSet>,
) {
    for ctx in pipeline.iter_mut(Phase::Defend) {
        let def_stat = stats.get(ctx.intent.target)
            .map(|s| s.get(AttributeId::DEFENSE))
            .unwrap_or(0.0);
        let dodge = stats.get(ctx.intent.target)
            .map(|s| s.get(AttributeId::DODGE))
            .unwrap_or(0.0);

        ctx.is_dodged = roll_dodge(ctx.rng_seed, dodge);
        if ctx.is_dodged {
            ctx.final_damage = 0.0;
            ctx.mitigated_damage = ctx.base_damage;
            continue;
        }

        let effective_def = def_stat * (1.0 - ctx.penetration);
        ctx.mitigated_damage = (ctx.base_damage * ctx.damage_multiplier
            + ctx.damage_flat_bonus)
            .max(1.0)  // 至少 1 点伤害
            .min(effective_def);  // 防御抵消上限
        ctx.final_damage = ctx.base_damage - ctx.mitigated_damage;
    }
}
```

#### Phase 5: Resolve — 数值应用

```rust
fn resolve_combat(
    mut pipeline: ResMut<CombatPipeline>,
    mut health_query: Query<&mut Health>,
    mut result_writer: EventWriter<CombatResult>,
) {
    for ctx in pipeline.drain(Phase::Resolve) {
        if ctx.preview_only { continue; }

        if let Ok(mut health) = health_query.get_mut(ctx.intent.target) {
            let delta = match ctx.intent.intent_type {
                IntentType::Attack | IntentType::TrueDamage => -ctx.final_damage,
                IntentType::Heal => ctx.final_damage,
            };
            health.current = (health.current + delta)
                .clamp(0.0, health.max);

            result_writer.send(CombatResult {
                context: ctx,
                reactions_triggered: vec![],
                cues: vec![],
                did_kill: health.current <= 0.0,
            });
        }
    }
}
```

### 4. 预览与执行分离

```rust
/// 预览路径 — 只计算，不执行
pub fn preview_combat(
    intent: CombatIntent,
    pipeline: &CombatPipeline,
    world: &World,
) -> CombatPreview {
    // 1. 标记 preview_only = true
    let preview_intent = CombatIntent { preview_only: true, ..intent };
    // 2. 跑 Phase 1-4
    // 3. 返回计算结果（不触发 Resolve）
    // 4. 不产生任何副作用
    CombatPreview {
        estimated_damage: ctx.final_damage,
        hit_chance: ctx.hit_chance,
        crit_chance: ctx.crit_chance,
        is_dodged: ctx.is_dodged,
    }
}
```

> 🟥 预览路径绝对禁止修改 Health、触发 Event、产生副作用（SRPG §8.1）。

### 5. 管线编排

```rust
pub struct CombatPlugin;
impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CombatPipeline>()
            .add_event::<CombatIntent>()
            .add_event::<CombatResult>()
            .add_systems(PreUpdate, (
                create_combat_intent,
            ))
            .add_systems(Update, (
                generate_combat_values,
                modify_combat_values,
                defend_combat_values,
                resolve_combat,
            ).chain())
            .add_systems(PostUpdate, (
                finalize_combat,
                on_unit_damaged,     // Observer: 血量变化触发反应
                on_unit_killed,      // Observer: 死亡处理
            ));
    }
}
```

## Module Design

```
src/combat/
  ├── plugin.rs              — CombatPlugin
  ├── components.rs          — (Combat 专用 Component，如 Invulnerable Tag)
  ├── systems.rs             — 管线阶段 System
  ├── events.rs              — CombatIntent, CombatResult, DamageEvent, HealEvent
  ├── api.rs                 — preview_combat() 公开预览函数
  ├── resources.rs           — CombatPipeline (管线上下文传递)
  └── internal/
      ├── damage_formula.rs  — 纯函数：伤害公式
      ├── hit_chance.rs      — 纯函数：命中率计算
      └── crit_chance.rs     — 纯函数：暴击率计算
```

## Communication Design

| 通信 | 机制 | 方向 |
|------|------|------|
| Ability → CombatIntent | Event | ability → combat |
| 管线阶段间 | `CombatPipeline` Resource | combat 内部 |
| CombatResult → 外部 | Event | combat → 所有订阅者 |
| 伤害→反应链 | Observer (`on_damage_dealt`) | combat → reaction |
| 预览 | `api.rs` 公开纯函数 | 外部 → combat |

## 边界定义

### 允许
- Combat 读取 ModifierSet 和 AttributeSet（Layer 2）
- Combat 写入 Health.current（HP 例外路径）
- 预览模式跑完整计算路径（但不执行 Resolve 阶段）
- 外部通过 `CombatIntent` Event 发起战斗计算

### 🟥 禁止
- 跳过管线直接修改 Health（Effect 执行除外——Effect 唯一入口）
- 预览模式修改任何状态
- Combat Pipeline 直接播放 VFX/SFX（必须通过 Cue）
- 防御结算以外的领域在管线中修改防御逻辑
- 管线阶段复用同一 System 实例（每阶段独立 System）

## Forbidden

| 禁止行为 | 理由 |
|---------|------|
| 直接 `health.current -= damage` | 必须走 Combat Pipeline |
| 预览路径修改 Health | 违反 CQRS Lite |
| 在 Combat 中播放 VFX | 违反逻辑/表现分离 |
| 绕过 Ability 直接创建 CombatIntent | 需要经过 Ability Pipeline（ADR-010） |
| 管线阶段共享可变状态 | 每阶段应独立处理 Context |

## Definition / Instance Design

- **Definition**: `DamageFormula` (config), `DamageTypeDef` (config)
- **Instance**: `CombatContext` (transient, pipeline 内), `Health` (Component)
- **Persistence**: 存档 `Health`（含 current/max）

## 后果

### 正面
- 七阶段管线覆盖了从 Intent 到 Finalize 的完整流程
- 预览/执行分离满足 CQRS Lite 要求
- 管线阶段通过 `.chain()` 声明式编排
- 纯函数公式可独立测试、可仿真运行

### 负面
- 七阶段管线对简单攻击（普攻无特效）显得较重型
- `CombatPipeline` Resource 作为跨阶段上下文传递，需要确保不漏数据

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 单一大 System 实现全部战斗逻辑 | 不可测试，不易扩展 |
| 所有阶段 Event 化 | 阶段间强耦合用 Event 不合适 |
| 伤害预览直接调用扣血逻辑 | 违反 CQRS Lite |

## 评审要点

- [ ] 预览模式的边界是否足够安全？能否在编译期阻止副作用？
- [ ] 七阶段是否包含大后期可能需要的扩展点（如"伤害反射"）？
- [ ] `CombatContext` 的字段是否覆盖了所有修正场景？
- [ ] 公式纯函数的位置：`internal/` 还是独立 `combat_formula.rs` crate？
