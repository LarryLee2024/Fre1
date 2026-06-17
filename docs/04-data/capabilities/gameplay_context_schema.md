---
id: capabilities.gameplay-context.schema.v1
title: GameplayContext Schema — 游戏上下文数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: instance, runtime
replay-safe: true
---

# GameplayContext Schema — 游戏上下文数据架构

> **领域归属**: Capabilities — 聚合层 | **依赖 Schema**: Tag, Attribute | **定义依据**: `docs/02-domain/capabilities/gameplay_context_domain.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `GameplayContextData` | Runtime | 行为上下文的完整数据载体，系统间传递 |
| `SourceInfo` | Runtime | 行为发起者信息 |
| `TargetInfo` | Runtime | 行为目标者信息 |
| `ContextChain` | Runtime | 溯源链，追踪行为链的完整路径 |
| `ContextOrigin` | Definition | 触发类型枚举 |

---

## 2. Problem

GameplayContext 是贯穿能力系统全链路的「统一数据总线」——从 Ability 激活到 Target 选定到 Execution 计算再到 Effect 应用和 Cue 触发，所有系统读写同一份行为上下文。Schema 必须解决：
- 统一的 source/target/ability 数据结构（避免架构文档所述的「字段膨胀」问题）
- 溯源链的防循环（反击/连锁/伤害转移场景）
- 跨系统传递的只读保证
- 上下文链的上限保护

---

## 3. Schema Design

### 3.1 GameplayContextData（Runtime 层）

```rust
/// 跨系统传递的统一数据载体。
/// 通过 ContextBuilder 构建，构建完成后不可变。
struct GameplayContextData {
    /// 上下文唯一标识
    context_id: ContextId,

    /// 触发类型
    origin: ContextOrigin,

    /// 行为发起者信息
    source: SourceInfo,

    /// 行为目标者信息
    target: TargetInfo,

    /// 使用的能力（可选，如环境伤害、陷阱等无能力来源的行为）
    ability_id: Option<AbilityDefId>,

    /// 使用的武器/装备（可选）
    equipment_id: Option<EquipmentId>,

    /// 元素类型（可选，如火焰/冰冻/闪电）
    element_type: Option<ElementType>,

    /// 是否为暴击
    is_critical: bool,

    /// 溯源链（追踪行为链）
    chain: ContextChain,

    /// 自定义扩展数据（供 Domain 注入特定数据）
    extensions: HashMap<String, Box<dyn ContextExtension>>,

    /// 构建时间（帧号）
    created_at_frame: u64,

    /// 元数据
    metadata: ContextMetadata,
}

struct ContextMetadata {
    /// 上下文版本
    schema_version: u32,
    /// 构建状态
    status: ContextStatus,
    /// 校验哈希
    checksum: u64,
}
```

### 3.2 SourceInfo / TargetInfo（Runtime 层）

```rust
struct SourceInfo {
    /// 发起者实体 ID
    entity_id: EntityId,

    /// 发起者阵营
    faction: FactionId,

    /// 发起者位置（可选，用于距离/掩体判定）
    position: Option<GridPosition>,

    /// 发起者的部分快照属性（可选，Execution 阶段可能使用）
    snapshot_attributes: Option<HashMap<AttributeId, f32>>,
}

struct TargetInfo {
    /// 目标实体 ID
    entity_id: EntityId,

    /// 目标阵营
    faction: FactionId,

    /// 目标位置（可选）
    position: Option<GridPosition>,

    /// 目标是否有效
    is_valid: bool,
}
```

### 3.3 ContextOrigin（Definition 层）

```rust
enum ContextOrigin {
    /// 直接行为（如主动施放技能）
    Direct,
    /// 链式反应（如反击、连锁闪电）
    ChainReaction,
    /// 触发器触发（如 OnDamaged 触发的被动技能）
    Triggered,
    /// 周期性触发（如 DoT/HoT 的每跳）
    Periodic,
    /// 环境原因（如陷阱、毒池）
    Environmental,
}
```

### 3.4 ContextChain（Runtime 层）

```rust
/// 溯源链——单向链表，记录行为链的完整路径。
/// 用于防止无限循环（反击→触发反击→触发反击...）。
struct ContextChain {
    /// 当前节点
    current: ChainNode,

    /// 上一节点（Option<Box> 而非 Vec，强制线性链）
    prev: Option<Box<ContextChain>>,

    /// 链长度上限
    max_length: u8,
}

struct ChainNode {
    /// 节点触发类型
    origin: ContextOrigin,

    /// 该节点的行为发起者
    source: SourceInfo,

    /// 该节点的行为目标
    target: TargetInfo,

    /// 该节点使用的能力
    ability_id: Option<AbilityDefId>,

    /// 节点时间（帧号）
    frame: u64,

    /// 节点唯一 ID（用于循环检测）
    node_id: u64,
}

/// 循环检测规则：
/// 如果链中已存在相同 (source.entity_id, target.entity_id, ability_id) 的组合 → 新节点被拒绝
impl ContextChain {
    fn would_create_cycle(&self, new_source: &EntityId, new_target: &EntityId, new_ability: &Option<AbilityDefId>) -> bool {
        // 遍历整个链，检查是否有完全相同的 (source, target, ability) 组合
        let mut current = Some(self);
        while let Some(chain) = current {
            let node = &chain.current;
            if &node.source.entity_id == new_source
                && &node.target.entity_id == new_target
                && node.ability_id == *new_ability
            {
                return true;
            }
            current = chain.prev.as_deref();
        }
        false
    }

    fn is_at_max_length(&self) -> bool {
        let mut length: u8 = 0;
        let mut current = Some(self);
        while let Some(chain) = current {
            length += 1;
            if length > chain.max_length {
                return true;
            }
            current = chain.prev.as_deref();
        }
        false
    }
}
```

### 3.5 ContextExtension（Runtime 层 — Trait）

```rust
/// 域自定义扩展数据 Trait。
/// Domain 通过实现此 Trait 向 GameplayContext 注入领域特定数据。
trait ContextExtension: Send + Sync {
    /// 扩展数据类型标识
    fn extension_type(&self) -> &'static str;
    /// 克隆（用于上下文传递时的复制）
    fn clone_extension(&self) -> Box<dyn ContextExtension>;
}
```

### 3.6 GameplayContextSnapshot（Persistence 层）

```rust
struct GameplayContextSnapshot {
    /// 存档版本
    schema_version: u32,

    /// 上下文 ID
    context_id: ContextId,

    /// 触发类型
    origin: ContextOrigin,

    /// 发起方和目标方（只存 ID，不存快照）
    source_entity: EntityId,
    target_entity: EntityId,

    /// 能力 ID
    ability_id: Option<AbilityDefId>,

    /// 链长度
    chain_length: u8,

    /// 是否为暴击
    was_critical: bool,
}
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 可热重载 | 备注 |
|----------|-------|--------|----------|------|
| `ContextOrigin` | Definition | 代码枚举 | 否 | 编译时确定 |
| `GameplayContextData` | Runtime | 否（仅快照） | 否 | 瞬时传递数据 |
| `SourceInfo` / `TargetInfo` | Runtime | 否 | 否 | GameplayContextData 内嵌 |
| `ContextChain` | Runtime | 否（仅快照） | 否 | 行为追踪 |
| `ContextExtension` | Runtime | 否 | 否 | Domain 扩展接口 |
| `GameplayContextSnapshot` | Persistence | 是（仅关键节点） | 否 | 回放校验用 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → TagSchema | FactionId 引用 Tag |
| 依赖 | → AttributeSchema | 快照属性值 |
| 被依赖 | ← AbilitySchema | 技能激活时创建上下文 |
| 被依赖 | ← ExecutionSchema | 执行计算读取上下文 |
| 被依赖 | ← EffectSchema | 效果应用引用上下文数据 |
| 被依赖 | ← CueSchema | 表现信号通过上下文携带参数 |
| 被依赖 | ← EventSchema | 事件以上下文为载荷 |

---

## 6. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | source 和 target 非空 | build() 完成 | `source.entity_id` 和 `target.entity_id` 必须有效 |
| V2 | 无环路 | 溯源链扩展 | would_create_cycle() 返回 true 时拒绝 |
| V3 | 链不超上限 | 溯源链扩展 | is_at_max_length() 返回 true 时拒绝 |
| V4 | 不可变性 | 传递中 | Active 状态后的修改断言失败 |
| V5 | ContextId 唯一 | 创建时 | 同一帧内不产生重复 ID |

---

## 7. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| 上下文创建 | 🟩 完全确定 | 由 Command/Ability 确定触发 |
| 溯源链扩展 | 🟩 完全确定 | 循环检测和链长上限都是确定性的 |
| 上下文传递 | 🟩 确定 | 构建后不可变，保证下游读取一致 |

---

## 8. Save Compatibility

GameplayContext 是纯运行时数据，不需要常规存档持久化。仅在回放系统的关键校验点记录轻量级快照。

---

## 9. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2（未来） | 扩展字段结构化 | 增加 extensions 类型注册表，向下兼容 |

---

## 10. Future Extension

- **ContextExtension 注册表**：将 `extensions: HashMap<String, Box<dyn ContextExtension>>` 改为注册表模式，保证扩展类型的安全反序列化
- **上下文压缩**：在长行为链中压缩已归档的上游节点数据（保留摘要而非完整节点）
- **上下文路由**：将 ContextOrigin 扩展为路由标签，控制上下文只传递给特定系统

---

## 11. Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| ContextExtension 滥用 | 各 Domain 向上下文注入过多数据导致膨胀 | 限制 extensions 大小，仅在必要时使用 |
| 内存泄漏 | ContextArchived 后未清理的上下文持续占用 | 引入 ContextId 的 TTL 机制 |
| 链长度误判 | 合法长链（如 5 次连锁闪电）被误判为循环 | 区分「循环」和「长链」——长链只告警不阻止 |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| Replay First | ✅ | 上下文创建和传递确定 |
| Logic/Presentation Separation | ✅ | 纯数据载体，不涉及表现 |
| 宪法 §16.5 GameplayContext | ✅ | 统一载荷规范 |
