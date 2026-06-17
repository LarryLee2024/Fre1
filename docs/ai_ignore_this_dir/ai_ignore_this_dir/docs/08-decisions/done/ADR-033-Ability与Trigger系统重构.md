# ADR-033: Ability/Trigger/Targeting 系统重构

## 状态
Accepted（2026-06-15）

## 背景

当前 Ability（Skill）、Trigger、Targeting 系统存在以下问题：

### Ability 系统现状（2032 行）

1. **命名矛盾** — 当前模块命名为 `ability/`（符合 ADR-026）但核心类型仍叫 `SkillData`/`SkillDef`/`SkillRegistry`，文件名仍为 `skill/` 残留
2. **5 阶段管线不完整** — `prepare_skill_execution()` 只实现了 Validate→Cost→Cast 三个阶段，Effect→Settlement 阶段在 `battle/pipeline/` 中，跨模块跳转
3. **技能类型混用** — `ReactionSkill`（反应技）归在 AbilityType 中，按 Data Law 004 应归属 Trigger 领域
4. **Condition 系统弱** — `SkillCondition` 只有 5 种基础条件（MpCost/RequireTag/TargetRequireTag/HpBelow/HpAbove），缺少 Linglan 的条件丰富度
5. **RON Schema 缺少 linglan 字段** — `SkillDef` 缺少 `cost.cp`、`special_rules`、`tags_required`/`tags_forbidden` 等 Linglan 标准字段

### Trigger 系统现状（997 行 + buff/trigger.rs）

1. **TriggerRegistry 重复** — `core/trigger/registry.rs` 和 `core/buff/trigger.rs` 各有一个 TriggerRegistry，后者随 Buff 废弃应删除
2. **事件分类弱** — `Trigger` 枚举 15 个事件的平坦列表，缺少 Linglan 5 大类层次结构
3. **无触发链深度控制** — `ExecutionStack` 有 `MAX_STACK_DEPTH` 但 Trigger 自身无链深度限制
4. **无反应技配额管理** — 缺少每回合/每场战斗触发次数上限的运行时追踪
5. **无 Condition 系统对接** — Trigger 的条件判断逻辑散落在 handler 中

### Targeting 系统现状（310 行）

**`core/targeting/resolver.rs` 只有 3 行注释占位符。** 目标选择解析实际上在 `ability/pipeline.rs` 中硬编码实现。没有独立的 TargetingDefinition 概念。

### 引用文档

- `docs/04-data/ll/05_Ability_ll.md` — 5 种技能类型 + 资源配置三层体系
- `docs/04-data/ll/06_Trigger_ll.md` — 5 大类事件 + 反应技规则 4 条 + 优先级+链深度
- `docs/04-data/ll/07_Targeting_ll.md` — 7 种目标类型 + 3 维筛选条件
- `docs/04-data/ll/data_relationship_overview.md` — Ability→Effect→Targeting→Trigger 引用关系
- `docs/01-architecture/README.md` §Skill — 技能定义与冷却
- `docs/08-decisions/ADR-014-技能释放管线设计.md` — 当前 5 阶段管线

## 决策

### 1. Ability 系统完全重写

#### 1.1 统一命名为 Ability

将 `SkillData`/`SkillDef`/`SkillRegistry`/`SkillSlots`/`SkillCooldowns` 全部重命名为 `Ability*`：

| 旧名 | 新名 | 理由 |
|------|------|------|
| `SkillData` | `AbilityData` | ADR-026 已定义 Ability 概念 |
| `SkillDef` | `AbilityDef` | 对应 Linglan AbilityDefinition |
| `SkillRegistry` | `AbilityRegistry` | ADR-030 统一 Registry 命名 |
| `SkillSlots` | `AbilitySlots` | 角色技能槽位 |
| `SkillCooldowns` | `AbilityCooldowns` | 冷却管理 |
| `SkillCondition` | `AbilityCondition` | 前置条件 |
| `SkillError` | `AbilityError` | 领域错误 |

#### 1.2 5 种 AbilityType（完全对齐 Linglan）

```rust
pub enum AbilityType {
    NormalAttack,   // 普攻 — 消耗 1 AP，无冷却
    ActiveSkill,    // 主动技能 — 消耗 AP + CP，有冷却
    ReactionSkill,  // 反应技 — 不消耗行动，事件触发（Data Law 004 注意）
    SupportSkill,   // 支援技能 — 瞬发，保留行动，消耗 CP
    PassiveSkill,   // 被动 — 无消耗，常驻/条件触发
}
```

> **Data Law 004 实现**：`ReactionSkill` 在 Ability 层面保留为标识，但实际触发行为由 Trigger 系统接管。即：`ReactionSkill` 的 `AbilityDefinition` 在 TriggerDefinition 中被引用，Ability 本身不包含"何时触发"的逻辑。

#### 1.3 Ability 完整 5 字段构成

```rust
pub struct AbilityDefinition {
    pub id: AbilityId,                    // 唯一标识
    pub name_key: LocalizedKey,           // ADR-017
    pub desc_key: LocalizedKey,           // ADR-017
    pub ability_type: AbilityType,        // 5 类型
    pub cost: CostDef,                    // AP + CP + HP%
    pub cooldown: Option<u32>,            // 冷却回合数
    pub range: u32,                       // 射程
    pub targeting_id: TargetingId,        // → TargetingDefinition
    pub effects: Vec<EffectId>,           // → EffectDefinition 列表
    pub tags_required: Vec<TagId>,        // 使用者必须具备的 Tag
    pub tags_forbidden: Vec<TagId>,       // 使用者禁止具备的 Tag
    pub special_rules: Vec<SpecialRule>,  // 特殊规则（瞬发/背击/多段）
}
```

#### 1.4 5 阶段 Ability 释放管线（完整实现）

```rust
pub fn execute_ability(
    world: &mut World,
    caster: Entity,
    ability_id: AbilityId,
    target: TargetingResult,
) -> AbilityResult {
    // Phase 1: Requirement 检查
    validate_requirements(caster, ability_id)?;     // Tag 检查 + 沉默/控制检查

    // Phase 2: Cost 扣除
    deduct_costs(caster, ability_id)?;               // AP + CP + HP%

    // Phase 3: Targeting 解析（委托 core/targeting/）
    let targets = resolve_targeting(caster, ability_id, target);

    // Phase 4: Effect 执行（委托 Effect Pipeline，ADR-032）
    for effect_id in &ability_def.effects {
        execute_effect_pipeline(world, caster, target_entity, effect_id);
    }

    // Phase 5: Cooldown/Cleanup
    set_cooldown(caster, ability_id);                // 设置冷却
    emit_cue(CueSkillCast { caster, ability_id });   // 表现信号

    Ok(())
}
```

#### 1.5 资源管控三层体系

```rust
// 行动点 (AP)
pub struct ActionPoints {
    pub current: u32,        // 当前 AP
    pub max_per_turn: u32,   // 每回合上限（默认 1）
    pub bonus: u32,          // 额外 AP（「再行动」效果）
}

// 能量 (CP)
pub struct Energy {
    pub current: i32,        // 当前能量
    pub max: i32,            // 能量上限
    pub regen_per_turn: i32, // 每回合固定回复
    pub bonus_on_kill: i32,  // 击杀额外回复
    pub bonus_on_hit: i32,   // 受击额外回复
}
```

### 2. Trigger 系统完全重写

#### 2.1 5 大类事件体系

```rust
pub enum TriggerEvent {
    // 回合事件
    TurnStart,
    TurnEnd,
    ActionStart,
    ActionEnd,

    // 战斗事件
    DealDamage,
    TakeDamage,
    KillUnit,
    UnitDied,

    // 技能事件
    BeforeSkillCast,
    AfterSkillCast,
    SkillHit,

    // 状态事件
    BuffApplied,       // 状态施加
    BuffRemoved,       // 状态移除
    ControlApplied,    // 控制生效
    ControlRemoved,    // 控制解除

    // 移动事件
    MoveStart,
    MoveEnd,
    EnterTile,
    LeaveTile,

    // 死亡链路事件（独立子阶段）
    NearDeath,         // 濒死窗口
    AboutToDie,        // 即将死亡
    UnitDeath,         // 单位死亡
}
```

#### 2.2 TriggerDefinition

```rust
pub struct TriggerDefinition {
    pub id: TriggerId,
    pub event_type: TriggerEvent,                  // 监听事件
    pub condition: Option<ConditionId>,             // 触发条件
    pub effect_id: EffectId,                       // 触发后执行的效果
    pub priority: u32,                             // 触发优先级
    pub max_trigger_per_turn: Option<u32>,          // 每回合上限
    pub max_trigger_per_battle: Option<u32>,        // 每场战斗上限
    pub chain_depth: u32,                          // 触发链深度（0 = 不触发链）
}
```

#### 2.3 触发优先级与链深度

```rust
pub struct TriggerManager {
    /// 对每个事件类型，按优先级排序的触发器列表
    handlers: HashMap<TriggerEvent, Vec<TriggerHandler>>,
    /// 当前触发的链深度
    current_chain_depth: u32,
    /// 运行时触发计数器
    trigger_counts: HashMap<(Entity, TriggerId), TriggerCounter>,
}

impl TriggerManager {
    /// 触发事件（递归安全）
    pub fn fire_event(&mut self, event: TriggerEvent, context: &TriggerContext) {
        if self.current_chain_depth >= MAX_CHAIN_DEPTH { return; } // 默认 = 0
        self.current_chain_depth += 1;

        for handler in self.handlers.get(&event) {
            if self.check_quota(handler) {
                handler.execute(context);
                self.increment_counter(handler);
            }
        }

        self.current_chain_depth -= 1;
    }

    /// 反应技触发规则：同类反应只触发最高优先级
    pub fn fire_reaction(&mut self, event: TriggerEvent, context: &TriggerContext) {
        // 收集所有匹配的 ReactionSkill
        // 按优先级排序
        // 只触发最高优先级的
    }
}
```

#### 2.4 合并 TriggerRegistry

删除 `core/buff/trigger.rs`（的 TriggerRegistry），合并到 `core/trigger/registry.rs`。

### 3. Targeting 系统完全实现

#### 3.1 填充 `core/targeting/resolver.rs`

```rust
/// 目标选择解析（纯函数）
pub fn resolve_targeting(
    caster: Entity,
    ability: &AbilityDefinition,
    player_input: PlayerTargetingInput,
    world: &World,
) -> TargetingResult {
    let targeting_def = TARGETING_REGISTRY.get(&ability.targeting_id)?;

    match targeting_def.target_type {
        TargetType::SingleEnemy => {
            // 验证目标阵营 = enemy
            // 验证距离 ≤ range
            // 验证目标不在 forbidden_tags 列表中
            // 返回单个目标 Entity
        }
        TargetType::AoEEnemy => {
            // 计算 AOE 形状（十字/圆形/直线）
            // 获取范围内所有敌人
            // 应用筛选条件
            // 返回目标列表
        }
        TargetType::Self => {
            // 返回施法者自身
        }
        // ... 其他 4 种类型
    }
}
```

#### 3.2 TargetingDefinition

```rust
pub struct TargetingDefinition {
    pub id: TargetingId,
    pub target_type: TargetType,          // 7 种
    pub range: u32,                       // 射程
    pub aoe_shape: Option<AoEShape>,      // 可选 AOE 形状
    pub filters: Vec<TargetFilter>,       // 筛选条件 [阵营/状态/距离]
    pub displacement: Option<DisplacementTargeting>, // 位移专用
}

pub enum TargetType {
    SingleEnemy,
    SingleAlly,
    Self,
    AoEEnemy,
    AoEAlly,
    AllEnemy,
    DirectionalLine,
}

pub enum AoEShape {
    Cross { radius: u32 },
    Circle { radius: u32 },
    Fan { angle: u32, radius: u32 },
    Line { length: u32, width: u32 },
    AllEnemies,
}
```

#### 3.3 删除旧的 SkillTargeting

删除 `core/ability/domain/types.rs` 中的 `SkillTargeting` 枚举（如果与新 `TargetingDefinition` 冲突），所有引用替换为新的 `TargetingId` + `TargetingDefinition` 模式。

### 4. Condition 系统增强

增强为 Linglan 条件模型：

```rust
pub enum Condition {
    // 属性条件
    AttributeCompare { attr: AttributeId, op: CompareOp, value: i32 },
    // Tag 条件
    HasTag(TagId),
    NotHasTag(TagId),
    // 状态条件
    HealthPercent { op: CompareOp, pct: i32 },  // HP 百分比
    EnergyPercent { op: CompareOp, pct: i32 },   // CP 百分比
    // 位置条件
    HeightDifference { min: u32 },               // 高差
    TerrainType(TerrainId),                      // 地形类型
    // 战斗条件
    IsInRange { distance: u32 },                 // 距离检查
    HasEffect(EffectId),                         // 效果存在检查
    // 复合条件
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),
}
```

## Module Design

```
src/core/
├── ability/
│   ├── mod.rs              # AbilityPlugin + 公共类型
│   ├── domain/
│   │   ├── mod.rs          # AbilityRegistry (HashMap<AbilityId, AbilityData>)
│   │   ├── types.rs        # AbilityDefinition + AbilityData + AbilityType
│   │   ├── error.rs        # AbilityError (领域错误)
│   │   └── defaults.rs     # 默认技能（basic_attack）
│   ├── pipeline.rs         # execute_ability() 5 阶段管线
│   ├── preview.rs          # 技能效果预览
│   ├── slots.rs            # AbilitySlots + AbilityCooldowns Components
│   ├── cost.rs             # 资源管控（AP + CP + HP%）
│   └── condition.rs        # AbilityCondition 条件系统
├── trigger/
│   ├── mod.rs              # TriggerPlugin
│   ├── types.rs            # TriggerDefinition + TriggerEvent + TriggerInstance
│   ├── registry.rs         # TriggerManager（统一，吸收 buff/trigger.rs）
│   ├── stack.rs            # ExecutionStack（链深度控制）
│   └── quota.rs            # 触发配额管理（每回合/每战斗计数）
├── targeting/
│   ├── mod.rs              # TargetingPlugin
│   ├── types.rs            # TargetingDefinition + TargetType + AoEShape
│   └── resolver.rs         ★ resolve_targeting() 完整实现（不再是占位符）
├── condition/
│   └── ...                 # Condition 枚举 + ConditionRegistry（引用自多个领域）
└── buff/（❗ 已删除）
```

## Communication Design

```
玩家输入 / AI 决策
  │
  ↓
AbilitySystem::execute(caster, ability_id, target_input)
  │
  ├──→ [Phase 1] Requirement 检查
  │       ├──→ condition::evaluate(requirement_condition, caster)
  │       └──→ 失败 → 阻断流程 + 错误返回
  │
  ├──→ [Phase 2] Cost 扣除
  │       ├──→ action_points.current -= cost.ap
  │       ├──→ energy.current -= cost.cp
  │       └──→ 不足 → 阻断流程
  │
  ├──→ [Phase 3] Targeting 解析
  │       ├──→ targeting::resolver.resolve(ability.targeting_id, input)
  │       └──→ 返回 TargetingResult（Entity 列表）
  │
  ├──→ [Phase 4] Effect 执行（→ ADR-032 Pipeline）
  │       ├──→ for each effect_id: execute_effect_pipeline()
  │       ├──→ [消息] AfterSkillCast
  │       └──→ [消息] SkillHit（每个命中目标）
  │
  └──→ [Phase 5] 收尾
          ├──→ ability_cooldowns.set(ability_id, cooldown)
          ├──→ [Cue] CueSkillCast
          └──→ [消息] TurnOrder 更新（如需插队）
```

## 边界定义

| 规则 | 允许 | 禁止 |
|------|------|------|
| Ability → Effect | 通过 EffectId 引用执行 | 在 Ability 内硬编码效果逻辑 |
| Ability → Targeting | 通过 TargetingId 引用 | 在 Ability 内实现选择逻辑 |
| Trigger → Effect | 通过 EffectId 触发 | Trigger 直接修改游戏状态 |
| Trigger → Ability | ReactionSkill 在 Trigger 中引用 | Ability 管理自身触发条件 |
| Targeting → Tag | Filter 使用 TagId 引用 | Targeting 修改 Tag 状态 |

## Forbidden（禁止事项）

- 🟥 **禁止** 保留 `SkillData`/`SkillDef`/`SkillRegistry` 旧命名 — 全部重命名为 `Ability*`
- 🟥 **禁止** `core/targeting/resolver.rs` 为占位符 — 必须完整实现 7 种 TargetType
- 🟥 **禁止** 保留 `core/buff/trigger.rs` 中的 TriggerRegistry 副本 — 合并到 `core/trigger/`
- 🟥 **禁止** `ReactionSkill` 在 Ability 中包含触发逻辑 — 触发行为归 Trigger 系统
- 🟥 **禁止** Trigger 链深度突破 `MAX_CHAIN_DEPTH` — 必须强制执行
- 🟥 **禁止** Trigger 无触发次数上限 — 每个 Trigger 必须声明 `max_trigger_per_turn` 或 `max_trigger_per_battle`
- 🟥 **禁止** Ability 释放管线中跳过任何阶段 — 5 阶段必须完整执行
- 🟥 **禁止** 技能系统直接使用 `&mut World` 修改 Entity 状态 — 必须通过 `execute_effect_pipeline()`

## Definition / Instance Design

| 层 | Ability | Trigger | Targeting |
|----|---------|---------|-----------|
| Definition | `AbilityDefinition`（id, type, cost, cooldown, effects, targeting, conditions） | `TriggerDefinition`（id, event, condition, effect, priority, max_trigger, chain_depth） | `TargetingDefinition`（id, target_type, range, aoe_shape, filters） |
| Instance | `AbilityInstance`（entity, ability_id, cooldown_remaining, current_energy_cost） | `TriggerInstance`（entity, trigger_id, triggers_this_turn, triggers_this_battle） | —（纯函数，无实例） |
| Runtime | `ActionPoints` Component + `Energy` Component | `TriggerManager` Resource（全局触发管理器） | `TargetingResult`（目标列表） |

## 后果

### 正面
- 消除命名矛盾：完全采用 Ability 命名体系
- Targeting 系统从 3 行占位符变为完整实现
- Trigger 系统统一，消除重复 TriggerRegistry
- 5 阶段 Ability 释放管线完整实现在单一模块内
- 资源管控（AP/CP/冷却）对齐 Linglan 三层体系

### 负面
- `core/ability/` 模块 2032 行代码大部分需要重写
- 所有技能 RON 文件（6 个）需要迁移到新 Schema
- `battle/pipeline/intent.rs` 中的 `prepare_skill_execution()` 需要重构，将其职责移回 `ability/pipeline.rs`
- `Trigger` 枚举从 15 事件平坦列表迁移到 5 大类 20+ 事件的层次化结构，影响所有引用点

## 替代方案（已拒绝）

| 方案 | 拒绝原因 |
|------|----------|
| 保留 Skill 命名，仅在文档层使用 Ability | 延续命名矛盾，下游代码始终混乱 |
| 分步迁移 Target 解析：先重构 ability/pipeline.rs, 再填充 targeting/resolver.rs | 两个文件之间的循环引用会延长共存期 |
| Trigger 保持平坦枚举，加 event_category 字段 | 不如层次化枚举类型安全 |
