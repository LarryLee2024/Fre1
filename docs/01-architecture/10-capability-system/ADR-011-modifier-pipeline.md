---
id: 01-architecture.ADR-011
title: ADR-011 — Modifier → Attribute Pipeline Architecture
status: proposed
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-011: Modifier → Attribute 管线架构

## 状态

**Proposed** — 依赖 ADR-010（Ability Pipeline）和 `docs/04-data/capabilities/modifier_schema.md`。

## 背景

属性系统是 SRPG 角色能力的数值基础。所有属性修改必须通过 Modifier Pipeline（SRPG §1.3、编码规则 §领域不变量），禁止直接修改最终属性值。Modifier 不拥有业务逻辑（Data Law 006），它只负责改变数值。需要一个统一的管线来收集、聚合、解析 Modifier。

## 引用的领域规则与数据架构

- `docs/02-domain/modifier_domain.md` — Modifier 领域规则
- `docs/02-domain/attribute_domain.md` — Attribute 领域规则
- `docs/02-domain/aggregator_domain.md` — Aggregator 领域规则
- `docs/04-data/capabilities/modifier_schema.md` — Modifier Schema
- `docs/04-data/capabilities/attribute_schema.md` — Attribute Schema
- `docs/04-data/capabilities/aggregator_schema.md` — Aggregator Schema
- `.trae/rules/SRPG专项规则.md` §二 — 属性系统规范
- `docs/04-data/README.md` — Data Law 006（Modifier 不拥有业务逻辑）

## 决策

### 1. Modifier Pipeline 四阶段

```
Phase 1: Collect    ── 收集 Entity 上所有活跃 Modifier
       │
       ▼
Phase 2: Aggregate  ── 按 Aggregator 策略叠加计算
       │
       ▼
Phase 3: Resolve   ── 输入 AttributeResolver 计算最终属性值
       │
       ▼
Phase 4: Publish   ── 属性变化事件发布（Changed Filter / Observer）
```

### 2. 计算策略

**默认策略：实时计算（On-Read）**

```rust
/// 属性实时计算入口 — 每次读取时计算
/// 非热点路径使用此策略
fn resolve_attribute(
    entity: Entity,
    attribute_id: AttributeId,
    query: Query<&ModifierSet>,
    modifier_resolver: Res<ModifierResolver>,
) -> f32 {
    let modifiers = query.get(entity).unwrap();
    modifier_resolver.resolve(attribute_id, modifiers)
}
```

**缓存策略：On-Change 失效**

```rust
/// 当 Profile 证明确为热点路径时：
/// 1. 属性值缓存于 `AttributeCache` Resource
/// 2. Modifier 变化时立即失效
/// 3. 下次读取时重建
fn on_modifier_changed(
    trigger: Trigger<ModifierChanged>,
    mut cache: ResMut<AttributeCache>,
) {
    cache.invalidate(trigger.entity());
}
```

> 🟩 默认实时计算保证一致性。缓存仅用于 Profile 证实的热点路径（SRPG §2.2）。

### 3. Modifier 数据结构

```rust
/// Modifier 源 — 用于追溯"这个属性增益从哪来"
#[derive(Component)]
pub struct ModifierSource {
    pub source_type: SourceType,  // Equipment | Buff | Passive | Talent | Terrain
    pub source_id: DefinitionId,  // 来源的 ID
}

/// Modifier 集合 — 挂在 Entity 上
#[derive(Component)]
pub struct ModifierSet {
    pub modifiers: Vec<ModifierEntry>,
}

pub struct ModifierEntry {
    pub modifier_def_id: ModifierDefId,
    pub stat: AttributeId,         // 目标属性
    pub value: ModifierValue,      // 数值
    pub modifier_type: ModifierType, // Add | Multiply | Override
    pub source: ModifierSource,    // 来源追溯
    pub group: StackGroup,         // 堆叠分组
}
```

### 4. Aggregator 策略

Aggregator 决定同一属性的多个 Modifier 如何合并：

| 策略 | 计算方式 | 适用场景 |
|------|---------|---------|
| **Additive** | `base + Σ(add) * (1 + Σ(mul))` | 攻击力、防御力 |
| **Multiplicative** | `base * Π(1 + add) * Π(1 + mul)` | 暴击率、闪避率 |
| **Min** | `min(all_values)` | 移动力限制 |
| **Max** | `max(all_values)` | 视野范围 |
| **Override** | `last_write_wins` | 特殊状态覆盖 |
| **Formula** | `f(base, modifiers)` | 复杂公式（如伤害减免曲线） |

```rust
/// Aggregator 策略枚举
pub enum AggregationStrategy {
    Additive { base: f32 },
    Multiplicative { base: f32 },
    Min,
    Max,
    Override,
    Formula(fn(f32, &[ModifierEntry]) -> f32),
}
```

### 5. Modifier 生命周期

```
Effect Executed
    │
    ▼
commands.trigger(ApplyModifier)
    │
    ▼
on_apply_modifier (Observer)
    ├── 创建 ModifierEntry → 加入 Entity.ModifierSet
    ├── 如果同时有 Duration → 创建 ExpireTimer Component
    └── 触发 ModifierChanged → Cache 失效
    │
    ▼
... 游戏进行 ...
    │
    ▼
ExpireTimer 到期 / RemoveModifier 触发
    │
    ▼
on_remove_modifier (Observer)
    ├── 移除 ModifierEntry
    ├── 触发 ModifierChanged → Cache 失效
    └── 清理相关 Tag/Component
```

### 6. 属性解析器 (AttributeResolver)

```rust
/// 属性解析器 — 读取 Definition（base）+ ModifierSet（加成）→ 最终值
pub struct AttributeResolver {
    /// 注册的属性解析函数（由 attribute Feature 提供）
    resolvers: HashMap<AttributeId, ResolverFn>,
}

impl AttributeResolver {
    pub fn resolve(
        &self,
        entity: Entity,
        attribute_id: AttributeId,
        base_value: f32,           // 从 AttributeDef.base 读取
        modifiers: &ModifierSet,
    ) -> f32 {
        let strategy = self.get_strategy(attribute_id);
        let effective = strategy.aggregate(base_value, modifiers);
        // 最终值 clamp
        effective.clamp(self.get_min(attribute_id), self.get_max(attribute_id))
    }
}
```

### 7. HP/MP 例外路径

SRPG 规则明确允许 HP/MP 等资源型数值直接修改（不走 Modifier Pipeline）：

```rust
/// HP 修改专用 System — 不走 Modifier Pipeline
/// 例外依据：SRPG §2.2 §修改规范
fn apply_hp_damage(
    mut query: Query<&mut Health>,
    mut damage_events: EventReader<DamageEvent>,
) {
    for ev in damage_events.read() {
        if let Ok(mut health) = query.get_mut(ev.target) {
            health.current = (health.current - ev.amount).max(0.0);
        }
    }
}
```

> ⚠️ 注意：伤害数值本身（`ev.amount`）必须经过 Combat Pipeline 计算，其中可能使用了 Modifier（攻击力、防御力等），但最终的 HP 扣除操作是直接修改。这是**唯一**的例外路径。

## Module Design

```
src/core/capabilities/modifier/
  ├── components.rs      — ModifierSet, ModifierEntry, ModifierSource
  ├── systems.rs         — on_apply_modifier, on_remove_modifier
  ├── resources.rs       — ModifierResolver
  └── events.rs          — ModifierChanged, ApplyModifier, RemoveModifier

src/core/capabilities/attribute/
  ├── components.rs      — AttributeSet (if runtime overrides needed)
  └── resources.rs       — AttributeResolver

src/core/capabilities/aggregator/
  ├── components.rs      — (策略注册主要走 Res)
  └── resources.rs       — AggregationStrategies
```

## Communication Design

| 通信 | 机制 | 方向 |
|------|------|------|
| Effect → Modifier 应用 | `commands.trigger(ApplyModifier)` | effect → modifier |
| Modifier 变更通知 | Observer (`ModifierChanged`) | modifier → 系统内部 |
| Cache 失效 | Observer (`ModifierChanged`) | modifier → cache |
| 属性值读取 | 直接调用 `AttributeResolver::resolve()` | 任何 Feature → attribute |
| HP 修改例外 | `DamageEvent` → `apply_hp_damage` system | combat → health |

## 边界定义

### 允许
- Any Feature 通过 `AttributeResolver` 读取最终属性值
- Effect 通过 Trigger 添加/移除 Modifier
- HP/MP 直接修改（仅限资源型属性，数值型必须走 Modifier）
- 缓存失效在 Modifier 变化时触发

### 🟥 禁止
- 直接修改 `base_stat` 或 `final_stat`（必须通过 Modifier）
- Modifier 中包含业务逻辑（if/else 条件分支）
- Aggregator 中引用 ECS Query（Aggregator 应为纯函数）
- `Health.current` 之外的任何属性直接修改
- 使用 Modifier Pipeline 处理 Cost 扣除（SP/MP 消耗走 PreCost 阶段）

## Forbidden

| 禁止行为 | 理由 |
|---------|------|
| 直接修改 final_stat 字段 | 绕过 Modifier Pipeline，数据不一致 |
| Modifier 中包含 `on_turn_start` 等逻辑 | Data Law 006 禁止 |
| Aggregator 中调用外部 System | Aggregator 应为无副作用的纯函数 |
| 凭直觉提前引入 Cache | 必须 Profile 确认为热点后再优化（SRPG §2.2） |
| 属性解析依赖特定 Entity archetype | Resolver 必须与 Entity 类型无关 |

## Definition / Instance Design

- **Definition**: `AttributeDef` (Asset), `ModifierDef` (Asset), `AggregationStrategy` (config)
- **Instance**: `ModifierSet` (Component), `ModifierEntry` (struct), `AttributeCache` (Resource, if hot path)
- **Spec**: 无独立的 Spec 层，Modifier 直接来自 Effect 实例化
- **Persistence**: 存档 `ModifierSet` + source 信息（用于读档后重新关联）

## 后果

### 正面
- Modifier → Aggregator → Resolver 三阶段清晰分离
- 默认实时计算保证一致性，缓存优化可控
- HP/MP 例外路径合理（不强制所有属性走同一流程）
- 堆叠策略完全下沉到 Aggregator（Data Law 008）

### 负面
- 实时计算在 Entity 千级别时可能成为热点（需要 Profile 验证）
- HP 例外路径需要团队共识（可能有人错误地推广到其他属性）

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 全缓存策略 | 出现不一致事件的概率 > 性能提升的价值 |
| Effect 直接修改属性 | 违反 Data Law 005/006 |
| Modifier 作为回调函数 | 违反 Data Law 006（包含逻辑） |
| 每个属性独立 Component | Archetype 爆炸，查询不灵活 |

## 评审要点

- [ ] HP/MP 例外路径的边界是否清晰？哪些属性属于"资源型"？
- [ ] Override 策略的优先级冲突如何解决（多个 Override 叠加）？
- [ ] ModifierSource 是否足够追溯"这个 buff 是谁给的"？
- [ ] 缓存策略的失效粒度：全 Entity 失效还是单属性失效？
