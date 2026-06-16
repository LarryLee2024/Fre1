# ADR-032: Effect Pipeline 全链路重构

## 状态
Accepted（2026-06-15）

## 背景

当前 Effect Pipeline 是项目中最严重的技术债务集中地：

### 核心问题

1. **双路径执行并行** — 旧 `calculate_damage_from_effect()`（`effect/types.rs:218`，标记 deprecated 但仍在用）与新 `Execution` trait（`execution/damage.rs`）同时存在，运行时路径不确定
2. **Buff 模块（3030 行/9 文件）已废弃未删除** — 标记 deprecated 但仍被其他模块引用，`resolve_status_effects()` 系统处理 DoT/HoT/Stun 的逻辑没有迁移到新 Pipeline
3. **EffectHandler 巨文件** — `effect/handler.rs` 942 行，使用 `Box<dyn EffectHandler>` 动态分发，不符合"类型安全优先"原则
4. **Pipeline 阶段不完整** — 当前只有 Generate→Modify→Execute 三段，缺少独立的 Stacking 和 Cue 阶段
5. **旧 EffectQueue 设计** — `EffectQueue` Resource + `PendingEffect` 的队列模式已被新 Pipeline 的 `ExecutionRegistry` trait 对象模式取代，但队列代码未删除

### Linglan 目标管线

```
Effect（生成效果意图）
  → Stacking（堆叠策略匹配）
  → Execution（公式执行：伤害/治疗/护盾）
  → Modifier（属性数值修正）
  → Attribute（属性刷新）
  → Tag（标签变更）
  → Cue（表现事件）
```

### 引用文档

- `docs/04-data/ll/04_Effect_ll.md` — 8 种 Effect 类型 + Buff 生命周期 4 阶段
- `docs/04-data/ll/08_Execution_ll.md` — 4 段伤害公式 + 治疗/护盾公式 + 数值边界
- `docs/04-data/ll/09_Stacking_ll.md` — 8 种堆叠策略
- `docs/04-data/ll/10_Pipeline_Replay_ll.md` — 回合管线 + 技能管线 7 步
- `docs/04-data/ll/data_relationship_overview.md` — 数据流方向
- `docs/01-architecture/README.md` §Effect Pipeline — Generate→Modify→Execute 三段式

## 决策

### 1. 新 Effect Pipeline 架构

效果执行走完整 7 阶段管线：

```rust
/// 新 Effect Pipeline 调度器
pub fn execute_effect_pipeline(
    world: &mut World,
    effect: EffectInstance,
    context: ExecutionContext,
) -> PipelineResult {
    // Phase 1: Stacking 判定
    let stacking_result = resolve_stacking(&effect, &context);
    if stacking_result == StackingResult::Ignored { return PipelineResult::Blocked; }

    // Phase 2: Execution 公式结算
    let execution_result = execute_formula(&effect, &context);

    // Phase 3: Modifier 修正
    let modified_result = apply_modifiers(execution_result, &context);

    // Phase 4: Attribute 写入
    apply_attribute_changes(&modified_result, &context);

    // Phase 5: Tag 变更
    apply_tag_changes(&effect, &context);

    // Phase 6: Cue 事件下发
    emit_cue_events(&effect, &modified_result, &context);

    PipelineResult::Completed(modified_result)
}
```

### 2. Effect 类型（8 种，对应 Linglan 模型）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectType {
    Damage(DamageEffect),           // 伤害
    Heal(HealEffect),               // 治疗
    ApplyBuff(BuffEffect),          // 施加状态效果（含 Duration）
    Dispel(DispelEffect),           // 驱散
    Displacement(DisplacementEffect), // 位移
    ApplyShield(ShieldEffect),      // 护盾
    Summon(SummonEffect),           // 召唤
    Kill(KillEffect),               // 死亡
}
```

每个 EffectType 的详细字段见 `docs/04-data/ll/04_Effect_ll.md` §四。

### 3. EffectDef RON Schema

```ron
// content/effects/poison.ron
(
    id: "effect.e_001",
    version: 1,
    name_key: Some("buff.b_001.name"),     // 作为 ApplyBuff 时有名称
    desc_key: Some("buff.b_001.desc"),
    effect_type: ApplyBuff,
    duration: Some((
        duration_type: Turns,
        value: 3,
        tick_timing: ActionEnd,
    )),
    stacking: Some("stack_independent"),
    max_stack: 9,
    tick_effect: Some("effect.e_002"),     // Tick 时执行的效果
    cue: Some("cue.c_001"),
)

// content/effects/phys_damage.ron
(
    id: "effect.e_010",
    version: 1,
    effect_type: Damage,
    damage: (
        damage_type: "dmg_physical",
        skill_multiplier: 1.0,
        can_crit: true,
        is_multi_hit: false,
    ),
    execution: "execution.ex_001",  // 引用 ExecutionDefinition
    cue: Some("cue.c_010"),
)
```

### 4. Stacking 系统（8 类型）

扩展当前 4 枚举到 Linglan 8 类型：

```rust
pub enum StackType {
    RefreshDuration,      // 刷新持续时间，不叠加层数（改名：当前 Replace→RefreshDuration）
    StackIndependent,     // 独立叠加，每层效果独立（改名：当前 StackAdd→StackIndependent）
    StackDecay,           // 衰减叠加，层数递增效果递减
    StackUndispellable,   // 不可驱散叠加，层数影响驱散难度
    TakeStrongest,        // 取最强效果，弱效果被覆盖
    CounterStack,         // 反击类堆叠，只触发最高优先级
    ShieldMaxRefresh,     // 护盾取最大值+刷新时长
    ShieldIndependent,    // 不同类护盾独立共存
}
```

删除旧的 `Replace` 变体（被 `RefreshDuration` 替代）和旧的 `StackMax` 变体（语义不明确，拆分为 `TakeStrongest` + `StackIndependent`）。

### 5. Execution 系统（4 段伤害公式）

当前 3 个 Executor（Damage/Heal/Shield）保留，但 Damage 执行器需要重写为 Linglan 4 段公式：

```rust
// execution/damage.rs — 新 4 段公式
pub struct DamageExecution;

impl Execution for DamageExecution {
    fn execute(&self, ctx: &ExecutionContext) -> ExecutionResult {
        let final_attack = self.resolve_attack(ctx);     // 第一段
        let effective_def = self.resolve_defense(ctx);    // 第二段
        let base_damage = self.base_damage(final_attack, effective_def, ctx); // 第三段
        let final_damage = self.final_modifiers(base_damage, ctx); // 第四段
        let clamped = clamp_damage(final_damage, ctx.damage_type);
        ExecutionResult::Damage(clamped)
    }
}
```

4 段公式输入/输出：

| 段 | 名称 | 输入 | 公式 | 输出 |
|----|------|------|------|------|
| 1 | 攻击结算 | 基础攻击, 攻击百分比加算, 固定加成, 属性转换, 元素克制, 高低地 | `(base × (1 + Σ%add) + Σflat + convert) × element × height` | final_attack |
| 2 | 防御结算 | 目标防御, 降防乘算, 无视防御乘算 | `target_def × Π(1 - def_break) × Π(1 - armor_pen)` | effective_def |
| 3 | 基础伤害 | final_attack, effective_def, 技能倍率 | `(final_attack - effective_def) × skill_mult` | base_damage |
| 4 | 最终修正 | base_damage, 增伤加算, 易伤乘算, 减伤乘算, 暴击倍率 | `base × (1 + Σdmg_up) × Πvuln × Πdmg_red × crit` | final_damage |

### 6. Buff 模块删除

**src/core/buff/ 整个目录删除**（9 文件，3030 行）。

迁移清单：

| 旧 Buff 功能 | 迁移目标 |
|-------------|---------|
| `BuffData` / `BuffDef` | → `EffectDefinition`（effect_type: ApplyBuff）+ `DurationDef` + `StackingDef` |
| `BuffInstance` / `ActiveBuffs` | → `EffectInstance` Component + `ActiveEffects` Component |
| `resolve_status_effects()`（DoT/HoT/Stun tick） | → Tick 阶段由 `EffectPipeline` 的 `Stacking→Execution→Modifier→Tag→Cue` 处理 |
| `apply.rs`（Buff 施加/叠加） | → `execute_effect_pipeline()` 中的 Stacking 阶段 |
| `buff/trigger.rs`（TriggerRegistry 副本） | → 合并到 `core/trigger/` 统一管理 |
| `buff/id.rs` | → **删除**（被 `shared/ids/` 替代） |
| `BuffRegistry` | → `EffectRegistry`（由 ADR-030 定义） |

### 7. EffectHandler 巨文件删除

`effect/handler.rs`（942 行）整体删除，替换为 `ExecutionRegistry` + 具体 Executor 文件：

| 旧 Handler | 新位置 |
|-----------|--------|
| `DamageHandler` | `execution/damage.rs`（已存在，需重写公式） |
| `HealHandler` | `execution/heal.rs`（已存在） |
| `ModifierHandler` | 合并到 `modifier/` 模块 |
| `CleanseHandler` | `effect/cleanse.rs`（新建） |
| `EffectPreview` | 合并到 `effect/preview.rs` |
| `GenerateContext` / `PreviewContext` / `ExecuteContext` | 由 `ExecutionContext` 统一替代 |

### 8. EffectQueue / PendingEffect 删除

删除以下废弃类型：
- `EffectQueue` Resource
- `PendingEffect` / `PendingEffectData`
- `EffectResult` / `EffectResultData`
- `EffectDef` 旧枚举变体（`ApplyBuff` 已废弃，`ApplyModifier` 推荐）
- `calculate_damage_from_effect()` 函数

## Module Design

重构后的模块结构：

```
src/core/
├── effect/
│   ├── mod.rs              # EffectPlugin + Pipeline 调度入口
│   ├── types.rs            # EffectType 枚举 + EffectDefinition + EffectInstance
│   ├── handler.rs          # EffectHandler trait（新，简洁版，非 942 行）
│   ├── pipeline.rs         # execute_effect_pipeline() 7 阶段调度
│   ├── cleanse.rs          # DispelEffect 处理
│   ├── displacement.rs     # DisplacementEffect 处理
│   ├── summon.rs           # SummonEffect 处理
│   └── kill.rs             # KillEffect 处理（死亡 4 步链路）
├── execution/
│   ├── mod.rs              # Execution trait + ExecutionRegistry
│   ├── damage.rs           # 4 段伤害公式
│   ├── heal.rs             # 治疗公式
│   └── shield.rs           # 护盾公式 + 吸收顺序
├── stacking/
│   ├── mod.rs              # StackingPlugin
│   ├── types.rs            # 8 种 StackType + StackingDefinition
│   └── resolver.rs         # resolve_stacking() 纯函数
├── modifier/
│   ├── mod.rs              # ModifierPlugin
│   ├── types.rs            # ModifierOperation（8 种）+ ModifierDefinition
│   └── calculator.rs       # apply_modifiers() / apply_damage_modifiers()
└── buff/（❗ 删除这个目录）
```

## Communication Design

```
Ability 释放 / Trigger 触发
  │
  ↓
EffectInstance 生成
  │
  ├──→ [Pipeline Phase 1] Stacking: resolve_stacking()
  │       ├──→ RefreshDuration: 刷新计时，不叠加
  │       ├──→ StackIndependent: 叠加层数
  │       ├──→ StackDecay: 衰减叠加
  │       └──→ Ignored: Pipeline 阻断，流程结束
  │
  ├──→ [Pipeline Phase 2] Execution: execute_formula()
  │       ├──→ DamageExecution: 4 段公式
  │       ├──→ HealExecution: 治疗公式
  │       └──→ ShieldExecution: 护盾公式
  │
  ├──→ [Pipeline Phase 3] Modifier: apply_modifiers()
  │       └──→ ModifierRuleRegistry 标签匹配
  │
  ├──→ [Pipeline Phase 4] Attribute: apply_attribute_changes()
  │       ├──→ HP 变更（Damage/Heal）
  │       ├──→ 护盾值变更
  │       └──→ [消息] AttributeModified
  │
  ├──→ [Pipeline Phase 5] Tag: apply_tag_changes()
  │       ├──→ 添加 Tag（ApplyBuff → control_stun）
  │       ├──→ 移除 Tag（Dispel → debuff）
  │       └──→ rebuild_tags() 触发
  │
  ├──→ [Pipeline Phase 6] Cue: emit_cue_events()
  │       ├──→ CueDamage / CueHeal / CueDeath / CueBuffApply
  │       └──→ CueEmitter 缓存 → 帧末批量下发
  │
  └──→ [Pipeline Phase 7] Record: record_to_battle_log()
          └──→ BattleRecord 追加
```

## 边界定义

| 边界 | 允许 | 禁止 |
|------|------|------|
| Effect → Execution | Effect 引用 ExecutionId 执行公式 | Effect 内部硬编码公式逻辑 |
| Effect → Modifier | Modifier 管线修正计算结果 | Modifier 决定 Pipeline 流程 |
| Effect → Cue | 每 Effect 执行后至少发射一个 Cue | Cue 反向调用 Effect |
| Effect → Tag | Effect 添加/移除 Tag | Effect 直接读取 Tag 做条件判断（→ Trigger）|
| Stacking → Effect | Stacking 决定 Effect 是否生效 | Stacking 修改 Effect 内部数据 |

## Forbidden（禁止事项）

- 🟥 **禁止** 保留 `src/core/buff/` 目录中的任何文件 — Phase 3 开始时整个目录删除
- 🟥 **禁止** 保留 `effect/handler.rs` 的 942 行巨文件 — 必须拆分为独立处理器
- 🟥 **禁止** 使用 `Box<dyn EffectHandler>` 动态分发 — 改用 `ExecutionRegistry` + 具体类型
- 🟥 **禁止** 保留 `EffectQueue`、`PendingEffect`、`PendingEffectData` 等队列类型
- 🟥 **禁止** 保留 `calculate_damage_from_effect()` 函数 — 由 `DamageExecution` 替代
- 🟥 **禁止** 跳过 Stacking 阶段直接执行 Effect — 所有 Effect 必须经过 Stacking 判定
- 🟥 **禁止** Effect 内部跳过 Pipeline 直接修改 HP/Tag/Attribute
- 🟥 **禁止** Effect 不发射 Cue 事件 — Cue 是表现层的唯一入口
- 🟥 **禁止** 堆叠策略少于 8 种 — 必须完整实现 Linglan 8 类型

## Definition / Instance Design

| 层 | Effect | Execution | Stacking | Modifier |
|----|--------|-----------|----------|----------|
| Definition | `EffectDefinition`（id, effect_type, duration, stacking, execution, cue） | `ExecutionDefinition`（id, stages, boundaries） | `StackingDefinition`（id, stack_type, max_stack, on_max_action, duration_refresh） | `ModifierDefinition`（id, target_attr, operation, value, stacking_rule） |
| Instance | `EffectInstance`（entity, effect_id, source, remaining_duration, current_stack, lifecycle_phase） | `ExecutionContext`（attacker, defender, skill_mult, damage_type） | `StackingState`（current_stack, remaining_duration） | `ModifierInstance`（entity, modifier_id, source, remaining_duration, current_stack） |
| Runtime | `PipelineResult`（Completed/Blocked/Error） | `ExecutionResult`（Damage/Heal/Shield 数值） | `StackingResult`（NewlyApplied/Replaced/Refreshed/Stacked/Ignored） | `ModifierEntry`（操作记录） |

## 后果

### 正面
- 消除最大技术债务源：删除 3030 行废弃 Buff 模块
- 消除双路径执行的不确定性，统一走 Effect→Stacking→Execution→Modifier→Attribute→Tag→Cue 7 阶段管线
- EffectHandler 942 行巨文件拆分为独立的小型处理器
- 执行管线完整对齐 SRPG Lite-GAS 架构

### 负面
- 涉及 30+ 文件修改，是本次重构中影响面最大的 Phase
- 所有现有技能（6 个技能 RON）、Buff（8 个 Buff RON）需要按新 Schema 重写
- `resolve_status_effects()`（DoT/HoT/Stun）的 tick 逻辑需要在新 Pipeline 中重新实现
- `execute_combat_effects()`（intent.rs 中的完整战斗流程）需要与新的 Pipeline 调度对接

## 替代方案（已拒绝）

| 方案 | 拒绝原因 |
|------|----------|
| 保留旧 Pipeline 作为后备路径 | 违背"零兼容模式"原则，延续双路径问题 |
| 先提取新 Pipeline，再逐步迁移旧 EffectHandler | 渐进式 = 长期并存 = 技术债务不消除 |
| 在旧 Buff 模块上叠加新 Effect 层 | 多层间接增加复杂度，不改 3030 行遗留代码没有意义 |
