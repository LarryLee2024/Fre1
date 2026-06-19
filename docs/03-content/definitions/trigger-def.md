---
id: 03-content.definitions.trigger-def
title: TriggerDef — Trigger Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# TriggerDef — Trigger Content Def 定义

> **Content Layer**: L1 Capability | **领域规则**: `docs/02-domain/capabilities/trigger_domain.md` | **数据 Schema**: `docs/04-data/capabilities/trigger_schema.md` | **插件代码**: `src/content/plugins/trigger_plugin.rs`

---

## 1. Overview

TriggerDef 是**事件到技能的激活桥梁**——定义"当某个事件发生时，检查条件是否满足，满足则激活目标技能"。Trigger 是游戏中反应/自动行为的核心机制：

- 被动技能："每回合开始时恢复 5% 生命值"
- 反击："受到近战攻击时，进行一次普通攻击"
- 自动效果："受到火焰伤害时，获得一个护盾"
- 死亡触发："死亡时，对周围所有敌人造成伤害"
- 条件触发："生命低于 30% 时，进入狂暴状态"

### 设计原则

- **Trigger 不拥有行为**：Trigger 只描述"何时触发什么技能"，不包含行为逻辑本身
- **组合优于继承**：TriggerType + Condition + target_ability 的三段式组合覆盖所有触发场景
- **频率控制**：每回合触发次数上限、是否消耗反应动作等控制字段防止滥用

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `trigger_domain.md` | 触发优先级、频率限制、触发上下文传递、循环触发防护 |
| `trigger_schema.md` | TriggerDef 完整字段、TriggerType 枚举、TriggerPriority、TriggerContext |
| `condition-def.md` | 本 Def 的 `condition` 字段 |
| `ability-def.md` | 本 Def 的 `target_ability` 字段引用的 AbilityDef |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// Trigger Def 定义——事件到技能的激活桥梁。
///
/// 当指定事件发生时，检查条件（如果设置），条件满足则激活目标技能。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct TriggerDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID
    pub id: TriggerId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 触发条件 ──
    /// 触发类型（什么事件触发）
    pub trigger_type: TriggerType,

    /// 附加过滤条件（可选，在触发类型基础上进一步过滤）
    pub condition: Option<ConditionDefId>,

    // ── 目标技能 ──
    /// 触发器满足时激活哪个技能
    pub target_ability: AbilityDefId,

    // ── 触发控制 ──
    /// 触发优先级
    pub priority: TriggerPriority,

    /// 每回合最大触发次数（0 = 不限制）
    pub max_triggers_per_turn: u32,

    /// 是否允许在技能执行过程中再次触发
    pub allow_concurrent: bool,

    /// 触发后是否消耗资源（如消耗反应动作）
    pub consumes_reaction: bool,

    /// 生命周期（可选，触发器持续多久后自动移除）
    pub lifetime: Option<TriggerLifetime>,

    // ── 附加参数 ──
    /// 不同类型触发器的自定义参数
    pub params: TriggerParams,

    // ── 元数据 ──
    /// 分类标签
    pub tags: Vec<TagId>,
}

/// 触发优先级
#[derive(Deserialize, Clone, Debug)]
pub enum TriggerPriority {
    Critical = 0,
    High = 25,
    Medium = 50,
    Low = 75,
    Last = 100,
}

/// 触发器生命周期
#[derive(Deserialize, Clone, Debug)]
pub enum TriggerLifetime {
    /// 固定次数（触发 N 次后自动移除）
    Count(u32),
    /// 固定回合（N 回合后自动移除）
    Turns(u32),
    /// 持续到战斗结束
    UntilCombatEnd,
    /// 无限期
    Permanent,
}
```

### TriggerType 补充定义

TriggerType 在 `trigger_schema.md` 中完整定义，以下是关键类型概览：

```rust
#[derive(Deserialize, Clone, Debug)]
pub enum TriggerType {
    OnTagAdded { watch_tags: Vec<TagId>, respect_hierarchy: bool },
    OnTagRemoved { watch_tags: Vec<TagId> },
    OnDamaged { damage_type_filter: Option<Vec<TagId>>, min_damage: Option<f32> },
    OnHealed,
    OnAttack { attack_type_filter: Option<Vec<TagId>> },
    OnTurnStart,
    OnTurnEnd,
    OnDeath { watcher: DeathWatcher },
    OnMove,
    OnAbilityUsed { ability_filter: Option<Vec<AbilityDefId>> },
    OnCustom(CustomTriggerType),
    OnConditionMet(Condition),
}
```

### 字段说明

- **`trigger_type`**: 定义什么事件触发此 Trigger。不同的 TriggerType 有不同的参数（如 OnDamaged 需要 damage_type_filter）
- **`condition`**: 附加过滤条件，在事件发生后、技能激活前检查。例如"OnDamaged + cond:is_burning" = "受到伤害时如果正在燃烧则触发"
- **`target_ability`**: 触发时激活的目标技能。注意这是 AbilityDefId，因此 Trigger 不能直接触发 Effect（必须通过 Ability 编排）
- **`max_triggers_per_turn`**: 防止滥用（如无限反击）。0 表示不限次数
- **`allow_concurrent`**: 是否允许在该技能的 Effect 链尚未执行完毕时再次触发
- **`lifetime`**: 控制触发器的持续时长。在某些场景后自动失效

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// TriggerDef 注册插件
pub struct TriggerDefPlugin;

impl Plugin for TriggerDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<TriggerDef>();
        app.init_asset_loader::<RonAssetLoader<TriggerDef>>();
        app.insert_resource(DefRegistry::<TriggerDef>::new());

        app.add_systems(
            PreUpdate,
            load_trigger_defs
                .run_if(resource_changed::<Assets<TriggerDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按 ID 查找 TriggerDef
pub fn get_trigger_def(id: &TriggerId, registry: &DefRegistry<TriggerDef>) -> Option<&TriggerDef> {
    registry.get(id)
}

/// 按触发类型过滤
pub fn get_triggers_by_type(
    trigger_type_filter: TriggerType,
    registry: &DefRegistry<TriggerDef>,
) -> Vec<&TriggerDef> {
    registry.iter()
        .filter(|def| std::mem::discriminant(&def.trigger_type) == std::mem::discriminant(&trigger_type_filter))
        .collect()
}
```

### 注册生命周期

```
Load (triggers.ron) → Deserialize → Validate → Register (DefRegistry<TriggerDef>) → Freeze
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | TriggerId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | `max_triggers_per_turn` 合法 | >= 0，建议上限不超过单回合最大行动数 |
| V4 | `trigger_type` 参数合法 | 如 OnDamaged.min_damage > 0（如果设置），OnDeath 的 watcher 有效 |
| V5 | TriggerType::OnConditionMet 不引用自身 | 防止 A 触发 B，B 再触发 A 的无限循环 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V6 | `condition` (如果设置) 已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V7 | `target_ability` 已注册 | 在 DefRegistry<AbilityDef> 中存在 |
| V8 | TriggerType::OnDamaged.damage_type_filter 中的 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V9 | TriggerType::OnTagAdded.watch_tags 中的 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V10 | TriggerType::OnAbilityUsed.ability_filter 中的 AbilityDefId 已注册 | 在 DefRegistry<AbilityDef> 中存在 |
| V11 | TriggerDef 不得引用任何 L2+ Def | L1 内容不可引用 Entity/Gameplay/World 层内容 |

### 4.3 循环触发防护

| # | 规则 | 说明 |
|---|------|------|
| V12 | target_ability 的 Effect 链中不得引用可能再次触发此 Trigger 的事件 | 防止无限触发链 |

---

## 5. RON 示例

```ron
// TriggerDef 示例：反击
//
// 当受到近战物理伤害时，进行一次普通攻击反击。
(
    id: "trig:counterattack",
    name_key: "trigger.counterattack.name",
    description_key: "trigger.counterattack.desc",
    schema_version: 1,

    trigger_type: OnDamaged((
        damage_type_filter: Some(["tag:damage_type_slashing", "tag:damage_type_piercing"]),
        min_damage: Some(1.0),
    )),

    condition: Some("cond:target_in_melee_range"),

    target_ability: "ability:basic_attack",

    priority: Medium,
    max_triggers_per_turn: 1,
    allow_concurrent: false,
    consumes_reaction: true,

    tags: ["tag:combat", "tag:reaction"],
)
```

```ron
// TriggerDef 示例：自动回血
//
// 每回合开始时自动恢复生命值。
(
    id: "trig:auto_regen",
    name_key: "trigger.auto_regen.name",
    description_key: "trigger.auto_regen.desc",
    schema_version: 1,

    trigger_type: OnTurnStart,
    condition: Some("cond:in_combat"),

    target_ability: "ability:auto_regen",

    priority: Low,
    max_triggers_per_turn: 0,     // 不限制（每回合最多触发 1 次，因为是 OnTurnStart）
    allow_concurrent: false,
    consumes_reaction: false,

    tags: ["tag:combat", "tag:passive"],
)
```
