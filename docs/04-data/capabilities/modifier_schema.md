---
id: capabilities.modifier.schema.v1
title: Modifier Schema — 修改器数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition, instance
replay-safe: true
---

# Modifier Schema — 修改器数据架构

> **领域归属**: Capabilities — 核心基石层 | **依赖 Schema**: Tag, Attribute | **定义依据**: `docs/02-domain/capabilities/modifier_domain.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `ModifierData` | Instance | 单个修改器的运行时实例（运算类型、目标属性、幅度、优先级） |
| `ScalableValue` | Definition | 可缩放数值定义（固定值/曲线/属性缩放） |
| `ModifierContainer` | Instance | 实体上的活跃修改器容器（ECS Component） |
| `ModifierSource` | Definition | 修改器来源标记（用于追溯和堆叠判定） |

---

## 2. Problem

Modifier 是属性变更的「最小原子单元」——所有影响属性值的效果（Buff、装备、地形、技能）最终都表达为一组 Modifier。Schema 必须解决：
- 运算类型（Add/Multiply/Override）的精确数据表达
- 幅度值的灵活性（固定值、曲线值、属性缩放三种模式）
- 修改器的来源追溯（用于堆叠规则判定和审计）
- 优先级排序的数据结构
- 多个 Override 类型修改器的冲突处理

---

## 3. Schema Design

### 3.1 ModifierData（Instance 层）

```rust
struct ModifierData {
    /// 修改器唯一标识（运行时分配，用于追踪和移除）
    id: ModifierInstanceId,

    /// 运算类型
    op: ModifierOp,

    /// 目标属性 ID
    target_attribute: AttributeId,

    /// 幅度值（已解析为具体 f32 的"快照值"，在效果应用时确定）
    magnitude: f32,

    /// 执行优先级（越小越优先执行）
    /// 范围: 0–100，默认 50
    priority: u8,

    /// 修改器来源（用于追溯和堆叠判定）
    source: ModifierSource,

    /// 持续时间帧数（None = 无限期）
    duration_frames: Option<u64>,

    /// 已存活帧数（运行时维护，不持久化）
    elapsed_frames: u64,
}

enum ModifierOp {
    /// 加法: Final = Base + Sum(Add modifiers)
    Add,
    /// 乘法: Final = (Base + Add) * Product(Multiply modifiers)
    Multiply,
    /// 覆盖: Final = Override value（忽略其他运算）
    Override,
}

struct ModifierSource {
    /// 来源类型（Buff/Equipment/Ability/Passive/Environmental/Item）
    source_type: ModifierSourceType,

    /// 来源的具体 ID（如 BuffId, EquipmentId, AbilityId）
    source_id: String,

    /// 效果实例 ID（EffectInstanceId，如果有）
    effect_instance_id: Option<EffectInstanceId>,
}

enum ModifierSourceType {
    Buff,
    Equipment,
    Ability,
    Passive,
    Environmental,
    Item,
    Progression,
    Custom(String),
}
```

### 3.2 ScalableValue（Definition 层 — 配置时使用）

```rust
/// 可缩放数值定义。
/// 在 EffectDef/AbilityDef 配置中使用，运行时由 ModifierFactory 解析为具体的 f32。
enum ScalableValue {
    /// 固定值
    Fixed(f32),

    /// 曲线值: 从曲线表查询 level 对应的值
    Curve {
        /// 曲线表 ID
        curve_id: String,
        /// 查询等级（通常为技能等级/角色等级）
        level_source: LevelSource,
    },

    /// 属性缩放: 基于另一个属性的当前值 × 比率
    AttributeScaling {
        /// 源属性 ID
        source_attribute: AttributeId,
        /// 缩放比率（如 0.5 表示取属性的 50%）
        ratio: f32,
    },
}

enum LevelSource {
    /// 技能等级
    SkillLevel,
    /// 角色总等级
    CharacterLevel,
    /// 角色职业等级
    ClassLevel(ClassType),
    /// 固定值（不缩放）
    Fixed(u32),
}
```

### 3.3 ModifierContainer（Instance 层 — ECS Component）

```rust
struct ModifierContainer {
    /// 所有活跃修改器
    /// 按目标属性 ID 分组: attribute_id → Vec<ModifierData>
    modifiers: HashMap<AttributeId, Vec<ModifierData>>,

    /// Override 类型修改器的快速索引
    /// attribute_id → 当前生效的 Override Modifier（优先级最高的）
    override_index: HashMap<AttributeId, ModifierInstanceId>,

    /// 容量上限（防止修改器数量爆炸）
    max_modifiers: u32,
}
```

### 3.4 ModifierFilter（Definition 层 — 配置时使用）

用于条件性应用修改器的过滤规则（可选字段）。

```rust
struct ModifierFilter {
    /// 仅在持有特定标签时生效
    required_tags: Option<TagQuery>,

    /// 仅在特定 GameplayContext 下生效
    required_context: Option<ContextCondition>,
}
```

### 3.5 ModifierConfig（Definition 层 — 配置格式）

```yaml
# RON 配置示例 — 嵌入在 EffectDef 内的 Modifier 配置
ModifierConfig:
  # 示例1: 固定值加法修改器（「巨力腰带」+2 力量）
  - op: Add
    target_attribute: "attr_000001"   # 力量
    value:
      Fixed: 2.0
    priority: 50
    source:
      source_type: Equipment
      source_id: "itm_000042"

  # 示例2: 属性缩放乘法修改器（「火球术」伤害 = 施法属性 × 1.5）
  - op: Multiply
    target_attribute: "attr_000030"   # 火焰伤害
    value:
      AttributeScaling:
        source_attribute: "attr_000005"  # 智力
        ratio: 1.5
    priority: 50
    source:
      source_type: Ability
      source_id: "abl_000042"

  # 示例3: 曲线值加法修改器（「生命值成长」= 等级 × 5 + 10）
  - op: Add
    target_attribute: "attr_000020"   # 生命值上限
    value:
      Curve:
        curve_id: "hp_growth_curve"
        level_source:
          CharacterLevel: ~
    priority: 10
    source:
      source_type: Progression
      source_id: "class_000001"

  # 示例4: Override 修改器（「石化术」敏捷 = 0）
  - op: Override
    target_attribute: "attr_000002"   # 敏捷
    value:
      Fixed: 0.0
    priority: 80
    source:
      source_type: Ability
      source_id: "abl_000100"
```

### 3.6 ModifierSnapshot（Persistence 层）

```rust
struct ModifierSnapshot {
    schema_version: u32,
    entity_id: EntityId,

    /// 所有活跃修改器的完整列表
    active_modifiers: Vec<ModifierData>,

    /// Override 索引重建所需信息
    override_index: HashMap<AttributeId, ModifierInstanceId>,
}
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 可热重载 | 备注 |
|----------|-------|--------|----------|------|
| `ScalableValue` | Definition | 是（EffectDef 内嵌） | 是 | 配置时定义 |
| `ModifierConfig` | Definition | 是（配置文件） | 是 | EffectDef 的一部分 |
| `ModifierFilter` | Definition | 是（EffectDef 内嵌） | 是 | 可选条件过滤 |
| `ModifierData` | Instance | 否（通过 Snapshot） | 否 | 运行时实例 |
| `ModifierContainer` | Instance | 否（通过 Snapshot） | 否 | ECS Component |
| `ModifierSnapshot` | Persistence | 是（存档） | 否 | 存档格式 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → AttributeSchema | target_attribute 引用 AttributeId |
| 依赖 | → TagSchema | ModifierFilter 的 required_tags 引用 TagId |
| 被依赖 | ← AggregatorSchema | 聚合器消费 ModifierData |
| 被依赖 | ← EffectSchema | Effect 携带 ModifierConfig 列表 |
| 被依赖 | ← StackingSchema | Stacking 判定决定重复 Modifier 的处理 |

---

## 6. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | 目标属性已注册 | ModifierConfig 加载 | `target_attribute` 在 AttributeDefinition 中存在 |
| V2 | 优先级在合法范围 | ModifierConfig 加载 | `0 ≤ priority ≤ 100` |
| V3 | 来源可追溯 | ModifierData 创建 | `source.source_id` 非空 |
| V4 | Override 互斥 | 运行时注册 | 同一属性已有 Override 时，优先级高者生效，低者被抑制 |
| V5 | AttributeScaling 不引用自身 | ModifierConfig 加载 | `source_attribute ≠ target_attribute`（防止逻辑循环） |
| V6 | Curve 引用的曲线表已注册 | ModifierConfig 加载 | `curve_id` 在 CurveTableRegistry 中存在 |

---

## 7. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| Modifier 注册 | 🟩 完全确定 | 通过 Effect/Ability 的确定性流程触发 |
| ScalableValue 解析 | 🟩 确定 | Fixed/Curve/AttributeScaling 均在应用时确定快照值 |
| Modifier 移除 | 🟩 确定 | 持续时长由帧计数控制（非 wall-clock） |
| Override 冲突解决 | 🟩 完全确定 | 优先级排序 → 最高者生效，结果确定 |

**结论**: Modifier Schema 是 Replay-safe 的。ScalableValue 中的 AttributeScaling 在应用时对源属性值拍摄快照，不产生运行时耦合。

---

## 8. Save Compatibility

| 场景 | 兼容性 | 版本策略 |
|------|--------|----------|
| 基础存档 | 🟩 | Save v1: 存 ModifierData 完整列表（含快照后的 magnitude） |
| 新增 Op 类型 | 🟨 前向兼容 | 新增 variant 时旧存档的 ModifierOp 不受影响 |
| ScalableValue 格式变更 | 🟩 配置层隔离 | 存档只存解析后的 magnitude f32 值，不存原始 ScalableValue |
| Modifier 来源变更 | 🟩 无损 | source.source_id 为字符串，格式变化不影响反序列化 |

**关键设计**: Modifier 持久化时只保存 `ModifierData`（含已解析的 magnitude f32），不保存 `ScalableValue`。ScalableValue 的解析在效果应用时完成，存档只关心"被应用了什么值"而非"值如何被计算"。

---

## 9. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2（未来） | 增加 ModifierTag（对修改器加标签过滤） | 新增 optional 字段，旧存档缺省 |
| v3（未来） | 引入 ModifierGroup 批量管理 | 新增 optional group_id 字段 |

---

## 10. Future Extension

- **ModifierGroup**: 一组逻辑相关的修改器，可被整体移除/暂停/免疫
- **ModifierTag**: 为修改器附加标签，支持 Condition 中的过滤（如「禁用所有魔法来源的 Modifier」）
- **DiminishingReturns**: 对同一属性的多层同类型 Modifier 应用递减收益曲线
- **ModifierOverrideChain**: 支持多级 Override 的优先级回退（最高优先级失效后自动启用次高）

---

## 11. Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| ModifierContainer 膨胀 | 长时间战斗后修改器数量过多 | 设 max_modifiers 上限，定期清理已过期修改器 |
| AttributeScaling 快照不一致 | 属性值在快照瞬间变化导致引用值错误 | 快照在 Aggregator 管线计算完成后拍摄 |
| Override 优先级竞争 | 多来源 Override 同时注册时行为不确定 | 明确的优先级排序规则 + 冲突日志 |
| 来源 ID 格式不一致 | 不同系统使用不同 source_id 格式 | 强制 `source_id` 统一为 `<类型前缀>_<6位数字>` |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| 三层分离（Def→Spec→Instance） | ✅ | ScalableValue → ModifierConfig → ModifierData |
| Data Driven First | ✅ | 修改器数值通过 ScalableValue 配置驱动 |
| Effect 是唯一业务执行入口 | ✅ | Modifier 仅通过 Effect/Cue 应用，无独立入口 |
| Modifier 不拥有业务逻辑 | ✅ | Modifier 是纯数值描述，不含 on_hit 等逻辑 |
| Replay First | ✅ | 快照值在效果应用时确定，确定性移除 |
