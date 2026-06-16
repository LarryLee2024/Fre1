---
id: capabilities.spec.schema.v1
title: Spec Schema — 规格/配置数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: spec
replay-safe: true
---

# Spec Schema — 规格/配置数据架构

> **领域归属**: Capabilities — 逻辑骨架层 | **依赖 Schema**: Tag, Attribute | **定义依据**: `docs/02-domain/spec_domain.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `AbilitySpec` | Spec | 角色身上的技能配置（等级、冷却缩减、强化等） |
| `EffectSpec` | Spec | 效果应用后的实例配置（来源、持续时间修正、堆叠计数等） |
| `DefReference` | Definition | Def→Spec 的引用关系模板 |
| `SpecRegistryConfig` | Definition | Def→Spec 工厂转换的配置规则 |

---

## 2. Problem

Spec 是「三层分离」的中间桥梁——连接 Definition（内容配置）和 Instance（运行时状态）。Schema 必须解决：
- Def→Spec→Instance 三层的数据边界和职责划分
- Spec 在 Def 基础上叠加的定制数据（等级、冷却缩减、强化）的数据结构
- EffectSpec 的快照机制（应用时对属性值拍快照，后续变化不影响计算）
- SpecRegistry 的工厂转换的数据驱动配置

---

## 3. Schema Design

### 3.1 AbilitySpec（Spec 层 — ECS Component）

```rust
struct AbilitySpec {
    /// Spec 唯一标识
    spec_id: SpecId,

    /// 引用的 AbilityDef ID
    def_id: AbilityDefId,

    /// 技能等级（1..MaxLevel，MaxLevel 由 AbilityDef 定义）
    level: u8,

    /// 冷却缩减（回合数，正值表示减多少回合冷却）
    cooldown_reduction: i32,

    /// 冷却覆盖（如果不为 None，替代 Def 中的冷却定义）
    cooldown_override: Option<u32>,

    /// 消耗覆盖（如果不为 None，替代 Def 中的消耗定义）
    cost_override: Option<CostOverride>,

    /// 输入绑定（快捷栏位置）
    input_binding: Option<InputBinding>,

    /// 强化/专长标记列表（如「强化火焰」「法术穿透」）
    enhancements: Vec<EnhancementId>,

    /// 是否隐藏（被动技能在 UI 中不可见）
    hidden: bool,

    /// 上次使用帧号（用于冷却计算）
    last_used_frame: u64,

    /// 强制解除冷却（用于特殊效果如"技能冷却立即结束"）
    forced_cooldown_reset: bool,
}

struct CostOverride {
    /// 替代的消耗类型（如消耗改为 HP 而非 MP）
    resource_attribute: AttributeId,
    /// 替代的消耗量
    amount: f32,
}
```

### 3.2 EffectSpec（Spec 层 — ECS Component）

```rust
struct EffectSpec {
    /// Spec 唯一标识
    spec_id: SpecId,

    /// 引用的 EffectDef ID
    def_id: EffectDefId,

    /// 来源上下文（谁施加了这个效果）
    source_context: EffectSource,

    /// 持续时间修正（帧数，正值表示增加持续时间）
    duration_modifier: i64,

    /// 堆叠层数（当前层数，受 Stacking 领域管理）
    stack_count: u32,

    /// 属性快照——应用时对施法者和目标的相关属性拍快照
    /// 后续属性变化不影响此 Effect 的计算
    snapshot: EffectSnapshot,

    /// 是否为周期性效果
    is_periodic: bool,

    /// 周期间隔（帧数，仅 is_periodic = true 时有效）
    period_interval: Option<u64>,

    /// 是否已被 Condition 系统验证通过
    condition_passed: bool,
}

struct EffectSource {
    /// 来源实体
    source_entity: EntityId,
    /// 来源能力（可选）
    source_ability: Option<AbilityDefId>,
    /// 来源物品（可选）
    source_item: Option<ItemId>,
}

struct EffectSnapshot {
    /// 快照时的施法者属性值
    caster_attributes: HashMap<AttributeId, f32>,
    /// 快照时的目标属性值
    target_attributes: HashMap<AttributeId, f32>,
    /// 快照时的上下文标记
    context_tags: Vec<TagId>,
    /// 快照帧号
    snapshot_frame: u64,
}
```

### 3.3 DefReference（Definition 层）

```rust
/// Def 元数据，供 SpecRegistry 进行 Def→Spec 转换时使用。
/// 嵌入在 AbilityDef / EffectDef 等配置结构中。
struct DefReference {
    /// Def ID
    def_id: String,

    /// Def 类型
    def_type: DefType,

    /// 创建 Spec 时的默认参数
    default_spec_params: DefaultSpecParams,
}

enum DefType {
    Ability,
    Effect,
    Buff,
    Item,
}

struct DefaultSpecParams {
    /// 初始等级
    initial_level: u8,
    /// 最大等级
    max_level: u8,
    /// 是否允许等级覆盖
    allow_level_override: bool,
}
```

### 3.4 SpecRegistryConfig（Definition 层 — 配置格式）

```yaml
# RON 配置示例 — Spec 注册中心配置
SpecRegistryConfig:
  # AbilitySpec 默认参数
  ability_defaults:
    initial_level: 1
    max_level: 5
    allow_level_override: true

  # EffectSpec 默认参数
  effect_defaults:
    max_stack: 1
    enable_snapshot: true

  # 特定 AbilityDef 的 Spec 覆盖
  ability_overrides:
    "abl_000001":      # 火球术
      max_level: 3
      allow_level_override: false

    "abl_000100":      # 石肤术
      initial_level: 2
```

### 3.5 SpecContainer（Instance 层 — ECS Component）

```rust
struct SpecContainer {
    /// 所有 AbilitySpec（keyed by spec_id）
    abilities: HashMap<SpecId, AbilitySpec>,

    /// 所有活跃的 EffectSpec（keyed by spec_id）
    effects: HashMap<SpecId, EffectSpec>,

    /// DefId → SpecId 的索引（快速查找）
    ability_by_def: HashMap<AbilityDefId, SpecId>,
    effect_by_def: HashMap<EffectDefId, Vec<SpecId>>,
}
```

### 3.6 SpecSnapshot（Persistence 层）

```rust
struct SpecSnapshot {
    schema_version: u32,
    entity_id: EntityId,

    /// 所有 AbilitySpec 的存档数据
    abilities: Vec<AbilitySpecSnapshot>,

    /// 所有活跃 EffectSpec 的存档数据
    effects: Vec<EffectSpecSnapshot>,
}

struct AbilitySpecSnapshot {
    def_id: AbilityDefId,
    level: u8,
    cooldown_reduction: i32,
    enhancements: Vec<EnhancementId>,
    last_used_frame: u64,
}

struct EffectSpecSnapshot {
    def_id: EffectDefId,
    source_entity: EntityId,
    stack_count: u32,
    duration_modifier: i64,
    snapshot: EffectSnapshot,
    period_interval: Option<u64>,
    remaining_frames: Option<u64>,
}
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 可热重载 | 备注 |
|----------|-------|--------|----------|------|
| `DefReference` | Definition | 是（配置内嵌） | 是 | 嵌入在 AbilityDef/EffectDef 中 |
| `SpecRegistryConfig` | Definition | 是（配置文件） | 是 | 工厂参数配置 |
| `AbilitySpec` | Spec | 是（通过 Snapshot） | 否 | ECS Component |
| `EffectSpec` | Spec | 是（通过 Snapshot） | 否 | ECS Component |
| `EffectSnapshot` | Spec | 是 | 否 | 属性值快照 |
| `SpecContainer` | Instance | 否（通过 Snapshot） | 否 | 组合容器 |
| `SpecSnapshot` | Persistence | 是（存档） | 否 | 存档格式 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → TagSchema | EffectSnapshot.context_tags 引用 Tag |
| 依赖 | → AttributeSchema | EffectSnapshot.attributes 快照属性值 |
| 被依赖 | ← AbilitySchema | AbilityInstance 引用 AbilitySpec |
| 被依赖 | ← EffectSchema | EffectInstance 引用 EffectSpec |
| 被依赖 | ← StackingSchema | Stacking 管理 EffectSpec 的 stack_count |

---

## 6. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | Def 已注册 | Spec 创建 | def_id 必须在 DefRegistry 中存在 |
| V2 | 等级在合法范围 | Spec 创建/等级变更 | `1 ≤ level ≤ max_level` |
| V3 | 无重复 Spec | Spec 授予 | 同一实体的同一 def_id 只能有一个 AbilitySpec |
| V4 | Snapshot 一致性 | EffectSpec 创建 | 快照属性值必须与当前 Aggregator 结果一致 |
| V5 | 级联终止 | Spec 移除 | 所有关联的 Instance 被同时终止 |

---

## 7. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| Spec 授予 | 🟩 完全确定 | 由确定性命令触发 |
| Spec 等级变更 | 🟩 完全确定 | 等级变化由 Event 驱动 |
| EffectSpec Snapshot | 🟩 完全确定 | 快照在确定时刻拍摄，属性值确定 |
| Spec 移除 | 🟩 完全确定 | 级联终止行为确定 |

---

## 8. Save Compatibility

| 场景 | 兼容性 | 版本策略 |
|------|--------|----------|
| 基础存档 | 🟩 | Save v1: SpecSnapshot 存能力等级、冷却、Effect 快照 |
| 新增 Enhancement 类型 | 🟩 前向兼容 | enum 新 variant，旧存档缺省为空列表 |
| AbilityDef 变更 | 🟩 运行时重算 | Spec 存 level 不存具体数值，读档后按当前 Def 重算 |
| 快照格式变更 | 🟨 需要迁移 | 旧格式快照→新格式自动转换 |

---

## 9. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2（未来） | AbilitySpec 增加专长树路径 | 新增 enhancements 字段，旧存档自动为空 |

---

## 10. Future Extension

- **AbilitySpec 专长系统**：enhancements 扩展为树形结构，支持专长选择/重置
- **动态 EffectSpec**：通过 Config 配置的 Effect 组合，运行时动态生成
- **Spec 版本化**：每个 Spec 记录生效的 Def 版本号，支持 Def 变更后的自动升级

---

## 11. Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| Def 与 Spec 版本不同步 | Def 更新后旧 Spec 数据不匹配 | Spec 记录 Def 版本号，加载时不匹配则自动重建 |
| Snapshot 数据膨胀 | 多 Effect 快照大量属性值 | 快照只存 Effect 实际引用的属性，非全量 |
| SpecContainer 容量 | 角色拥有大量技能/效果 | 设上限（默认 100 AbilitySpec + 200 EffectSpec） |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| 三层分离（Def→Spec→Instance） | ✅ | Spec 是桥梁，不混合 Def/Instance |
| Data Driven First | ✅ | SpecRegistryConfig 数据驱动 |
| Replay First | ✅ | Spec 创建/移除确定，Snapshot 确定 |
