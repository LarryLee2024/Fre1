---
id: 09-planning.integration-facade
title: Tactical integration.rs Anti-Corruption Layer 实施计划
status: proposed
owner: feature-developer
created: 2026-06-17
updated: 2026-06-17
tags:
  - planning
  - tactical
  - integration
  - facade
  - anti-corruption
---

# Tactical integration.rs → Full Anti-Corruption Layer (Facade) 实施计划

> **问题**：当前 `integration.rs` 太薄（121 行），`movement_system.rs` 仍然直接 import Capabilities 类型到 Query 签名中。如果 Capabilities 内部结构变化（如 `AttributeContainer` → `AttributeGraph`），所有系统都会编译失败。
>
> **目标**：将 `integration.rs` 扩展为完整的 Anti-Corruption Layer（Facade 模式），隐藏所有 Capabilities 内部结构，系统只通过语义化函数访问 Capabilities 逻辑。

---

## 1. 核心设计原则

### 1.1 Anti-Corruption Layer 三层分工

```
┌─────────────────────────────────────────────────────────┐
│  Systems（movement_system.rs 等）                        │
│  - Query 只包含 Tactical 专属组件（GridPos, MovementPoints）  │
│  - 调用 integration 语义函数                              │
│  - 永远不 import Capabilities 类型                        │
├─────────────────────────────────────────────────────────┤
│  integration.rs（Facade / ACL）                          │
│  - 导入 Capabilities 类型（仅此处）                         │
│  - 封装所有 Capabilities 交互为语义函数                    │
│  - 提供 Entity 级别的查询/修改 API                        │
├─────────────────────────────────────────────────────────┤
│  Capabilities（tag/attribute/modifier）                   │
│  - 内部结构可自由变化                                     │
│  - 只要 integration 适配层更新，系统不受影响               │
└─────────────────────────────────────────────────────────┘
```

### 1.2 隐藏粒度

| 隐藏目标 | 当前暴露 | 目标 |
|----------|----------|------|
| `TagSet` | Query 参数 | ✅ 隐藏，通过 `has_tag(entity)` 查询 |
| `AttributeContainer` | Query 参数 | ✅ 隐藏，通过 `read_attr(entity, key)` 查询 |
| `ModifierContainer` | Query 参数 | ✅ 隐藏，通过 `collect_modifiers(entity, key)` 查询 |
| `TagHierarchy` | Res 参数 | ✅ 隐藏，通过内部缓存 |
| `ModifierData` | 返回值 | ✅ 隐藏，返回 Tactical 语义类型 |
| `TagDefinition` | 返回值 | ✅ 隐藏，返回 bool/枚举 |
| `AttributeId` | 内部常量 | ✅ 隐藏，内部使用字符串常量 |
| `TagId` | 内部使用 | ✅ 隐藏，内部使用字符串常量 |

### 1.3 系统 Query 改造前后对比

**Before（当前 — 违规）**：
```rust
// movement_system.rs
cap_query: Query<(&TagSet, &AttributeContainer, &ModifierContainer)>,
tag_hierarchy: Res<TagHierarchy>,
```

**After（目标 — 合规）**：
```rust
// movement_system.rs
// 零 Capabilities imports
// 通过 integration 查询所有 Capabilities 逻辑
```

---

## 2. integration.rs 扩展设计

### 2.1 模块结构

```
integration.rs
├── 内部常量区（Attribute/Modifier Key 定义）
├── Tag 查询函数组（实体标签查询）
├── Attribute 查询函数组（属性读写）
├── Modifier 查询函数组（修改器查询/计算）
├── 复合查询函数组（跨管线组合查询）
├── 写入操作函数组（添加/移除标签、修改器）
└── 预留扩展区（Ability/Buff/AI/Pathfinding/UI Preview）
```

### 2.2 完整函数签名设计

```rust
//! integration — Tactical 域与 Capabilities 的 Anti-Corruption Layer
//!
//! **Facade 模式**：隐藏所有 Capabilities 内部结构（TagSet, AttributeContainer,
//! ModifierContainer, TagHierarchy），对外只暴露语义化函数。
//!
//! Systems 禁止直接 import Capabilities 类型，必须通过本模块的函数访问。
//! 当 Capabilities 内部结构变化时（如 AttributeContainer → AttributeGraph），
//! 只需修改本文件，所有 System 自动适配。
//!
//! 详见 ADR-022 §2.2 + ADR-045

use bevy::prelude::*;

// ─── 内部 Capabilities 导入（仅此处允许） ───────────────────────
use crate::core::capabilities::attribute::foundation::AttributeId;
use crate::core::capabilities::attribute::mechanism::AttributeContainer;
use crate::core::capabilities::modifier::foundation::{ModifierData, ModifierOp};
use crate::core::capabilities::modifier::mechanism::ModifierContainer;
use crate::core::capabilities::tag::foundation::{TagDefinition, TagId, TagNamespace};
use crate::core::capabilities::tag::mechanism::{TagHierarchy, TagSet};

// ─── Tactical 内部类型导入 ──────────────────────────────────────
use super::components::MovementType;

// ════════════════════════════════════════════════════════════════════
// 常量定义 — Attribute / Modifier Key 映射
// ════════════════════════════════════════════════════════════════════

/// 移动点数在 Capabilities Attribute 系统中的 Key。
pub(crate) const MOVEMENT_POINTS_ATTR: &str = "attr_movement_points";

/// 移动成本修饰器在 Capabilities Modifier 系统中的 Key。
pub(crate) const MOVEMENT_COST_MOD: &str = "movement_cost";

// ════════════════════════════════════════════════════════════════════
// §1 Tag 查询函数组
// ════════════════════════════════════════════════════════════════════

/// 检查实体是否拥有指定 MovementType 的 Tag。
pub(crate) fn has_movement_tag(
    tag_set: &TagSet,
    hierarchy: &TagHierarchy,
    movement_type: MovementType,
) -> bool {
    let tag_id_str = movement_type_to_tag_id(movement_type);
    let tag_id = TagId::new(tag_id_str);
    hierarchy.tags.get(&tag_id).is_some_and(|def| tag_set.has_tag(def))
}

/// 检查实体是否拥有指定的原始 Tag ID 字符串。
pub(crate) fn has_raw_tag(
    tag_set: &TagSet,
    hierarchy: &TagHierarchy,
    tag_id_str: &str,
) -> bool {
    let tag_id = TagId::new(tag_id_str);
    hierarchy.tags.get(&tag_id).is_some_and(|def| tag_set.has_tag(def))
}

/// 检查实体是否拥有任意一个指定的 Tag（OR 逻辑）。
pub(crate) fn has_any_movement_tag(
    tag_set: &TagSet,
    hierarchy: &TagHierarchy,
    types: &[MovementType],
) -> bool {
    types.iter().any(|&mt| has_movement_tag(tag_set, hierarchy, mt))
}

// ════════════════════════════════════════════════════════════════════
// §2 Attribute 查询函数组
// ════════════════════════════════════════════════════════════════════

/// 从 AttributeContainer 中读取当前值。
pub(crate) fn read_attr_current(container: &AttributeContainer, key: &str) -> f32 {
    let attr_id = AttributeId::new(key);
    container.attributes.get(&attr_id)
        .map(|v| v.current_value)
        .unwrap_or(0.0)
}

/// 从 AttributeContainer 中读取基础值。
pub(crate) fn read_attr_base(container: &AttributeContainer, key: &str) -> f32 {
    let attr_id = AttributeId::new(key);
    container.attributes.get(&attr_id)
        .map(|v| v.base_value)
        .unwrap_or(0.0)
}

/// 读取移动点数当前值（语义封装）。
pub(crate) fn read_movement_points(container: &AttributeContainer) -> f32 {
    read_attr_current(container, MOVEMENT_POINTS_ATTR)
}

/// 读取移动点数基础值（语义封装）。
pub(crate) fn read_movement_points_base(container: &AttributeContainer) -> f32 {
    read_attr_base(container, MOVEMENT_POINTS_ATTR)
}

// ════════════════════════════════════════════════════════════════════
// §3 Modifier 查询函数组
// ════════════════════════════════════════════════════════════════════

/// 收集指定 Key 的所有 Modifier（原始数据，供内部使用）。
pub(crate) fn collect_modifiers_raw(
    container: &ModifierContainer,
    key: &str,
) -> Vec<&ModifierData> {
    container.modifiers.get(key)
        .map(|mods| mods.iter().collect())
        .unwrap_or_default()
}

/// 计算指定 Key 的所有 Modifier 总影响值。
///
/// 将所有 Add 类型 modifier 的 magnitude 求和。
pub(crate) fn total_modifier_effect(container: &ModifierContainer, key: &str) -> f32 {
    container.modifiers.get(key)
        .map(|mods| {
            mods.iter()
                .filter(|m| m.op == ModifierOp::Add)
                .map(|m| m.magnitude)
                .sum()
        })
        .unwrap_or(0.0)
}

/// 计算移动成本的总 Modifier 影响（语义封装）。
pub(crate) fn total_movement_modifier_effect(container: &ModifierContainer) -> f32 {
    total_modifier_effect(container, MOVEMENT_COST_MOD)
}

/// 收集移动成本 Modifier（语义封装）。
pub(crate) fn collect_movement_modifiers(container: &ModifierContainer) -> Vec<&ModifierData> {
    collect_modifiers_raw(container, MOVEMENT_COST_MOD)
}

// ════════════════════════════════════════════════════════════════════
// §4 复合查询函数组（跨管线组合）
// ════════════════════════════════════════════════════════════════════

/// 完整的移动能力评估：Tag 验证 + Attribute 读取 + Modifier 影响。
///
/// 返回一个语义化结构体，系统只需处理结果，不接触 Capabilities 类型。
pub(crate) fn evaluate_movement_capability(
    tag_set: &TagSet,
    attrs: &AttributeContainer,
    mods: &ModifierContainer,
    hierarchy: &TagHierarchy,
    movement_type: MovementType,
) -> MovementCapability {
    MovementCapability {
        has_valid_tag: has_movement_tag(tag_set, hierarchy, movement_type),
        current_mp: read_movement_points(attrs),
        base_mp: read_movement_points_base(attrs),
        movement_modifier_total: total_movement_modifier_effect(mods),
        active_modifiers_count: collect_movement_modifiers(mods).len(),
    }
}

/// 移动能力评估结果 — 语义化输出，不暴露 Capabilities 类型。
#[derive(Debug, Clone)]
pub(crate) struct MovementCapability {
    pub has_valid_tag: bool,
    pub current_mp: f32,
    pub base_mp: f32,
    pub movement_modifier_total: f32,
    pub active_modifiers_count: usize,
}

// ════════════════════════════════════════════════════════════════════
// §5 写入操作函数组（添加/移除标签、修改器）
// ════════════════════════════════════════════════════════════════════

/// 为实体添加 MovementType Tag（通过 Entity Commands）。
///
/// 当前为预留接口，实际使用时需要通过 Commands 修改 TagSet。
pub(crate) fn add_movement_tag(
    tag_set: &mut TagSet,
    hierarchy: &TagHierarchy,
    movement_type: MovementType,
) -> bool {
    let tag_id_str = movement_type_to_tag_id(movement_type);
    let tag_id = TagId::new(tag_id_str);
    if let Some(def) = hierarchy.tags.get(&tag_id) {
        tag_set.add_tag(def);
        true
    } else {
        false
    }
}

/// 为实体移除 MovementType Tag。
pub(crate) fn remove_movement_tag(
    tag_set: &mut TagSet,
    hierarchy: &TagHierarchy,
    movement_type: MovementType,
) -> bool {
    let tag_id_str = movement_type_to_tag_id(movement_type);
    let tag_id = TagId::new(tag_id_str);
    if let Some(def) = hierarchy.tags.get(&tag_id) {
        tag_set.remove_tag(def);
        true
    } else {
        false
    }
}

// ════════════════════════════════════════════════════════════════════
// §6 预留扩展区 — 未来 System 需要的接口
// ════════════════════════════════════════════════════════════════════

// --- Ability System 预留 ---
// TODO: pub(crate) fn has_ability_tag(...) -> bool
// TODO: pub(crate) fn read_ability_cooldown(...) -> f32
// TODO: pub(crate) fn read_ability_cost(...) -> f32

// --- Buff/Effect System 预留 ---
// TODO: pub(crate) fn active_buff_count(container: &ModifierContainer) -> usize
// TODO: pub(crate) fn has_debuff(container: &ModifierContainer) -> bool

// --- AI System 预留 ---
// TODO: pub(crate) fn evaluate_ai_threat(...) -> f32
// TODO: pub(crate) fn query_enemy_tags(...) -> bool

// --- Pathfinding System 预留 ---
// TODO: pub(crate) fn movement_range_for_type(...) -> u32
// TODO: pub(crate) fn terrain_cost_multiplier(...) -> f32

// --- UI Preview 预留 ---
// TODO: pub(crate) fn preview_move_cost(...) -> MovementPreview

// ════════════════════════════════════════════════════════════════════
// 内部辅助函数
// ════════════════════════════════════════════════════════════════════

/// MovementType → TagId 字符串映射。
fn movement_type_to_tag_id(movement_type: MovementType) -> &'static str {
    match movement_type {
        MovementType::Walk => "tag_000010",
        MovementType::Fly => "tag_000011",
        MovementType::Swim => "tag_000012",
        MovementType::Climb => "tag_000013",
        MovementType::Teleport => "tag_000014",
    }
}
```

### 2.3 设计要点说明

| 要点 | 说明 |
|------|------|
| **Entity 不是参数** | 当前设计中，函数接收 `&TagSet`、`&AttributeContainer` 等组件引用，而非 Entity。这是因为 ECS System 在 Query 中获取组件引用后传递给集成层函数，避免重复 Query。 |
| **返回语义类型** | `evaluate_movement_capability()` 返回 `MovementCapability` 结构体，不返回任何 Capabilities 类型。 |
| **预留扩展区** | Ability/Buff/AI/Pathfinding/UI Preview 的接口以 TODO 形式预留，遵循"三次才抽象"原则，当前只实现 movement 相关。 |
| **写入操作** | `add_movement_tag` / `remove_movement_tag` 预留了修改能力，但当前 movement_system 不需要写入。 |

---

## 3. movement_system.rs 改造

### 3.1 Before（当前）

```rust
use crate::core::capabilities::attribute::mechanism::AttributeContainer;
use crate::core::capabilities::modifier::mechanism::ModifierContainer;
use crate::core::capabilities::tag::mechanism::{TagHierarchy, TagSet};

cap_query: Query<(&TagSet, &AttributeContainer, &ModifierContainer)>,
tag_hierarchy: Res<TagHierarchy>,
```

### 3.2 After（目标）

```rust
// ✅ 零 Capabilities imports
// 只 import Tactical 组件 + integration

use crate::core::domains::tactical::components::{GridPos, MovementPoints, MovementType};
use crate::core::domains::tactical::error::TacticalError;
use crate::core::domains::tactical::events::ComputeMoveRequest;
use crate::core::domains::tactical::resources::GridMap;
use crate::core::domains::tactical::{integration as tac_integration, rules};

// Capabilities 查询参数仍保留（因为 ECS Query 需要它们）
// 但不再在函数体中直接使用 Capabilities 类型
cap_query: Query<(&TagSet, &AttributeContainer, &ModifierContainer)>,
tag_hierarchy: Res<TagHierarchy>,
```

**关键变化**：虽然 ECS Query 签名仍需要 Capabilities 组件（因为 Bevy 的 Query 宏需要 Component 类型），但函数体中所有对 Capabilities 的操作都通过 `tac_integration` 函数完成。

### 3.3 系统函数体改造

```rust
pub(crate) fn on_compute_move(
    trigger: On<ComputeMoveRequest>,
    mut commands: Commands,
    mut tac_query: Query<(&mut MovementPoints, &mut GridPos)>,
    cap_query: Query<(&TagSet, &AttributeContainer, &ModifierContainer)>,
    tag_hierarchy: Res<TagHierarchy>,
    grid_map: Res<GridMap>,
) {
    let entity = trigger.entity;
    let path = &trigger.event().path;
    let emit_event = trigger.event().emit_moved_event;

    // ... 路径校验（不变）...

    // 解析 Capabilities 组件（类型仍暴露给 ECS，但不直接使用内部结构）
    let Ok((tag_set, attrs, mods)) = cap_query.get(entity) else {
        // ... 错误处理 ...
    };

    // ── 通过 integration 访问所有 Capabilities 逻辑 ──
    let capability = tac_integration::evaluate_movement_capability(
        tag_set, attrs, mods, &tag_hierarchy, *mov_type,
    );

    if !capability.has_valid_tag {
        warn!("Entity {} has no movement tag for {:?}", entity, mov_type);
        commands.trigger(TacticalError::InvalidGridPosition);
        return;
    }

    info!(
        "[Capabilities Integration] ✅ Tag pipeline: movement tag resolved for {:?}",
        mov_type
    );
    info!(
        "[Capabilities Integration] ✅ Attribute pipeline: MP={}/{}",
        capability.current_mp, capability.base_mp
    );
    info!(
        "[Capabilities Integration] ✅ Modifier pipeline: {} active modifiers, total effect={}",
        capability.active_modifiers_count,
        capability.movement_modifier_total
    );

    // ── 后续移动计算使用 capability 结果 ──
    // total_cost += capability.movement_modifier_total;  // 替代原来的 _modifier_effect
    // ...
}
```

### 3.4 Query 签名的折中说明

当前阶段，ECS Query 签名仍需暴露 Capabilities 类型：

```rust
cap_query: Query<(&TagSet, &AttributeContainer, &ModifierContainer)>,
```

**这是合理的折中**：
1. Bevy Query 宏必须知道 Component 类型才能编译
2. 系统文件顶部的 `use` 语句确实引入了 Capabilities 类型
3. 但函数体中**所有业务逻辑**都通过 integration 函数完成
4. 如果未来 Capabilities 组件重命名（如 `AttributeContainer` → `AttributeGraph`），只需修改：
   - `integration.rs` 的 import 和函数实现
   - `movement_system.rs` 的 Query 签名（仅 1 处）

**未来优化路径**：如果要完全消除 Query 中的 Capabilities 类型，需要引入 `CapabilitiesBundle` 或通过 `Option<In<Capabilities>>` 的方式包装，这属于更深层次的重构，不在本次范围内。

---

## 4. 文档更新计划

### 4.1 `docs/01-architecture/README.md` — §3.3 业务域表格

在 tactical 行的"核心职责"列扩展：

```markdown
| `tactical` | `tactical_domain.md` | `domains/tactical_schema.md` | 网格位置、移动、掩体、夹击；`integration.rs` 作为 Anti-Corruption Layer 隐藏 Capabilities 内部结构 |
```

在 §6.2 "Business Domains 结构"的 `integration.rs` 行扩展：

```markdown
└── integration.rs     # Anti-Corruption Layer：隐藏 Capabilities 内部，系统只调用语义函数
```

### 4.2 `docs/02-domain/domains/tactical_domain.md` — §1 术语 + 新增 §9

在 §1 术语表新增：

```markdown
| Integration Facade | Tactical 域与 Capabilities 的 Anti-Corruption Layer，隐藏 TagSet/AttributeContainer/ModifierContainer 等内部类型 |
```

新增 §9 "集成架构"：

```markdown
## 9. 集成架构 — Anti-Corruption Layer

### 9.1 设计模式
Tactical 域通过 `integration.rs` 实现 Facade + Anti-Corruption Layer 模式，
隐藏所有 Capabilities 内部结构（TagSet, AttributeContainer, ModifierContainer, TagHierarchy）。

### 9.2 隐藏边界
| 隐藏目标 | 暴露形式 | 说明 |
|----------|----------|------|
| TagSet | `has_movement_tag(entity)` | 只暴露查询结果（bool） |
| AttributeContainer | `read_movement_points(container)` | 只暴露语义化数值（f32） |
| ModifierContainer | `total_movement_modifier_effect(container)` | 只暴露聚合结果（f32） |
| TagHierarchy | 内部缓存 | 外部完全不可见 |

### 9.3 系统约束
- 🟥 禁止 System 直接使用 Capabilities 返回的原始类型
- 🟥 禁止 System 直接 import Capabilities 的 foundation/mechanism 类型
- 🟩 System 只能通过 integration 函数获取 Capabilities 逻辑的结果
- 🟩 Capabilities 内部结构变化时，只修改 integration.rs，系统自动适配
```

### 4.3 `docs/00-governance/ai-constitution-complete.md` — §3.6.1

扩展 §3.6.1 规范：

```markdown
#### 3.6.1 Domain 调用 Capabilities 的规范
- 必须通过 Capabilities 各模块的公开 API 调用，禁止直接访问内部实现
- 业务规则通过「配置注入 + 回调注册」的方式接入通用框架
- 每个 Domain 必须有且仅有一个集成层（`integration.rs`）作为与 Capabilities 的唯一交互入口
- 🟥 **integration.rs 必须是 Anti-Corruption Layer（Facade 模式）**：
  - 隐藏所有 Capabilities 内部组件类型（TagSet, AttributeContainer, ModifierContainer, TagHierarchy）
  - 对外只暴露语义化函数（如 `has_movement_tag()`, `read_movement_points()`）
  - Systems 禁止直接 import Capabilities 的 foundation/mechanism 类型到函数体
  - ECS Query 签名中暴露 Capabilities Component 类型是唯一允许的例外（Bevy Query 宏要求）
```

---

## 5. 实施步骤

### Phase 1：扩展 integration.rs（无破坏性变更）
1. 在 `integration.rs` 中新增 `evaluate_movement_capability()` 函数和 `MovementCapability` 结构体
2. 新增 `read_attr_current()`, `read_attr_base()` 通用属性查询函数
3. 新增 `total_modifier_effect()` 通用修改器计算函数
4. 新增 `has_raw_tag()`, `has_any_movement_tag()` Tag 查询函数
5. 新增 `add_movement_tag()`, `remove_movement_tag()` 写入预留
6. 保留原有 7 个函数不变（向后兼容）
7. **运行 `cargo test`** — 742 tests must pass

### Phase 2：改造 movement_system.rs（使用新 API）
1. 将 `on_compute_move` 中的 4 处直接 Capabilities 调用替换为 `evaluate_movement_capability()`
2. 移除 `use crate::core::capabilities::...` 的 import（如果 Query 签名不需要）
3. **运行 `cargo test`** — 确认无回归

### Phase 3：文档更新
1. 更新 `docs/01-architecture/README.md` §3.3 和 §6.2
2. 新增 `docs/02-domain/domains/tactical_domain.md` §9
3. 扩展 `docs/00-governance/ai-constitution-complete.md` §3.6.1

### Phase 4：验证
1. `cargo check` — 编译通过
2. `cargo test` — 742 tests pass
3. `cargo clippy` — 无新 warning

---

## 6. 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| ECS Query 签名仍暴露 Capabilities 类型 | 中 | 这是 Bevy 机制限制，当前阶段可接受；未来通过 Bundle 包装消除 |
| 原有 7 个函数被新函数替代 | 低 | 保留旧函数向后兼容，标记 `#[allow(dead_code)]` |
| 性能影响（多一层函数调用） | 极低 | 编译器 inline 消除；函数调用开销可忽略 |
| 测试覆盖不足 | 中 | Phase 1 不修改现有逻辑，Phase 2 仅替换调用方式 |

---

## 7. 后续演进路径

```
Phase 1-4（本次）    Phase 5（未来）           Phase 6（远期）
─────────────────   ──────────────────────   ──────────────────
integration.rs      integration.rs           CapabilitiesBundle
隐藏类型返回值       + Entity 级别 API        Query 零 Capabilities
+ 复合查询函数       + Cache 层               类型暴露
                    + 事件驱动更新            + 完全解耦
```

---

*本文档是 Tactical integration.rs 扩展为 Anti-Corruption Layer 的完整实施计划，由 @architect 审核后由 @feature-developer 执行。*
