---
id: 01-architecture.ADR-040
title: ADR-040 — Data Flow & Ownership Strategy
status: approved
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-040: 数据流与所属权策略

## 状态

**Approved** — 依赖 ADR-000（Feature Module Map）和 `docs/04-data/README.md` 全部内容，本架构决策正式生效。

## 背景

35 个 Feature 模块之间需要清晰的数据流转规则：谁拥有什么数据？谁可以读/写？谁负责序列化？没有清晰的所属权策略，就会出现数据被多处修改、序列化遗漏、读写在错误边界执行等问题。

## 引用的领域规则与数据架构

- `docs/04-data/README.md` — Data Law 012（域间禁止直接数据引用）
- `docs/04-data/README.md` — 四层数据架构
- `.trae/rules/架构规则.md` §三 — 跨模块交互规范
- `.trae/rules/编码规则.md` — 领域不变量、Effect Pipeline 保护

## 决策

### 1. 数据所属权原则

每个数据有且仅有一个**拥有者（Owner）Feature**：

| 数据 | 拥有者 | 读取者 | 修改者 |
|------|--------|--------|--------|
| `Health` | combat | combat, ui(只读) | combat 专用 System |
| `ManaPool` | spell | spell, ui | spell System |
| `ModifierSet` | modifier | 所有 Feature | modifier System（通过 Trigger） |
| `AttributeSet` | attribute | 所有 Feature | attribute Resolver（间接） |
| `Inventory` | inventory | inventory, ui | inventory System |
| `Equipment` | inventory | inventory, modifier(只读) | inventory System |
| `Experience` | progression | progression, ui | progression System |
| `GridMap` | grid_map | 所有 Feature | terrain 系统（有限） |
| `Wallet` | economy | economy, ui | economy System |
| `Party` | party | party, camp, ui | party System |
| `StoryState` | narrative | narrative, quest | narrative System |
| `QuestInstance` | quest | quest, ui | quest System |
| `TurnQueue` | turn_phase | turn_phase, 所有 Feature(只读) | turn_phase System |

### 2. 数据访问规则

```
                    ┌─────────────────────┐
                    │   Owner Feature     │
                    │   (Full Access)     │
                    └──────────┬──────────┘
                               │
              ┌────────────────┼────────────────┐
              │                │                │
              ▼                ▼                ▼
      ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
      │  Same Layer  │ │ Upper Layer  │ │  UI (Read)   │
      │  Read Only   │ │ Read Access  │ │ Changed Only │
      └──────────────┘ └──────────────┘ └──────────────┘
```

**具体规则：**

| 访问类型 | 条件 | 机制 |
|---------|------|------|
| **读** | 任何 Feature | 公开 API 函数 / `pub` Component Query |
| **写（内部）** | Owner Feature 内 | 直接 System 修改 |
| **写（外部触发）** | 非 Owner | 必须通过 Event/Trigger 请求 Owner 执行 |
| **序列化** | Owner Feature | 每个 Feature 负责自己的 Persistence Schema |

### 3. 跨 Feature 数据修改的合法途径

```rust
// ✅ 合法路径 1: Owner 内部的直接修改
// inventory Feature 内的 System
fn pickup_item(mut query: Query<&mut Inventory>, /* ... */) {
    query.get_mut(entity).unwrap().items.push(new_item);
}

// ✅ 合法路径 2: 通过 Event 请求修改
// combat Feature 发送事件，并不直接修改 Inventory
fn drop_loot(mut loot_writer: EventWriter<LootDropEvent>) {
    loot_writer.send(LootDropEvent { /* ... */ });
}
// inventory Feature 监听事件并执行实际修改
fn on_loot_drop(mut reader: EventReader<LootDropEvent>, mut query: Query<&mut Inventory>) {
    for ev in reader.read() {
        query.get_mut(ev.entity).unwrap().items.push(ev.loot);
    }
}

// ✅ 合法路径 3: 通过 Trigger 请求 Modifier 修改
// inventory Feature 不能直接修改 ModifierSet
fn on_equip(trigger: Trigger<EquipEvent>, mut commands: Commands) {
    // 请求 modifier Feature 添加 Modifier
    commands.trigger(ApplyModifier { /* ... */ });
}

// ❌ 非法: 跨 Feature 直接修改
fn illegal_combat_modify_inventory(
    mut inv_query: Query<&mut Inventory>,  // combat 不应该直接修改 inventory
) {
    inv_query.get_mut(entity).unwrap().items.clear(); // 禁止!
}
```

### 4. 数据的 Definition/Instance 映射规则

| 数据层 | 所属权 | 加载时机 | 序列化 |
|--------|--------|---------|--------|
| **Definition** | Registry (Layer 7) | 游戏启动/热重载 | 不序列化（从配置加载） |
| **Spec** | 创建 Spec 的 Feature（如 ability） | 运行时按需创建 | 随 Entity 序列化 |
| **Instance** | 各 Feature Owner | 运行时 | Owner 负责 |
| **Persistence** | Save System (Layer 7) + Owner 配合 | 存档/读档 | Save Schema 定义 |

### 5. 数据流转图

```
配置加载时:
  registry::DefinitionRegistry
       │
       ├──→ tag::TagHierarchy
       ├──→ attribute::AttributeDefs
       ├──→ modifier::ModifierDefs
       └──→ ability::AbilityDefs

运行时（以战斗伤害为例）:
  player_input → command::Command
       │
       ▼
  combat::CombatIntent
       │
       ▼
  combat::CombatPipeline (内部)
       │
       ├──→ modifier::ModifierSet (读取)
       ├──→ attribute::AttributeSet (读取)
       │
       ├──→ combat::Health (写入)
       │
       └──→ combat::CombatResult (Event)
              │
              ├──→ reaction::Reaction (监听)
              ├──→ progression::Experience (监听)
              ├──→ quest::QuestInstance (监听)
              └──→ cue::CueSignal (Event)
                     │
                     └──→ ui::HpBar (Changed Filter)
```

### 6. 调试与审计

```rust
/// 数据变更审计 — 可选开启（dev feature）
#[cfg(feature = "dev")]
pub fn audit_data_ownership(world: &World) {
    // 检查是否有非 Owner 直接修改了受保护数据
    // 仅在调试模式下启用
}
```

## Module Design

本 ADR 不产生具体的模块文件。数据所属权规则是架构级约定，各 Feature 在实现时遵守。

## Communication Design

| 场景 | 机制 | 依据 |
|------|------|------|
| 请求数据修改 | Event（请求）→ Owner System（执行） | ADP |
| 读取数据 | 公开 API 函数 | ADP |
| 数据变更通知 | Event（`DataChanged`）+ Changed Filter | ADP |
| 序列化/反序列化 | Owner 提供 Serde impl，Save System 调度 | ADP |

## 边界定义

### 允许
- 任何 Feature 读取任何公开 Component/Resource（通过 Query）
- Owner Feature 直接修改自己的 Component/Resource
- 通过 Event 请求其他 Feature 修改数据

### 🟥 禁止
- 非 Owner 直接 `&mut` 修改其他 Feature 的 Component
- 一个 Component 被两个 Feature 同时"拥有"
- Definition 数据在运行时被任何 Feature 修改
- 在 Event Handler 中修改发送者的数据

## Forbidden

| 禁止行为 | 理由 |
|---------|------|
| 跨 Feature 直接 `&mut` 修改 Component | 违反数据所属权 |
| 一个数据多个 Owner | 责任不清，序列化冲突 |
| 在预览路径中修改持久状态 | 违反 CQRS Lite |
| Event Handler 产生副作用 | Handler 应轻量，副作用由 System 处理 |
| 序列化非 Owner 的数据 | Owner 负责自己的序列化 |

## Definition / Instance Design

本 ADR 定义数据流转规则，不直接定义具体结构。

## 后果

### 正面
- 数据所属权清晰，"每个数据有且只有一个 Owner"
- 跨 Feature 数据修改经过 Event/Trigger，可追踪可审计
- 序列化职责明确，不会遗漏

### 负面
- 跨 Feature 修改需要经过 Event，增加少量样板代码
- 团队成员需要记住每个数据的 Owner

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 全局数据无 Owner 限制 | 无法控制修改边界，Bug 难以追踪 |
| 所有数据修改走中央 DataBus | 过度设计，简单需求复杂化 |
| 每个数据独立 Add/Remove Event | 事件爆炸 |

## 评审要点

- [ ] 是否所有重要的 Component 都分配了明确的 Owner？
- [ ] Resource 的 Owner 如何定义？`GridMap` 是 grid_map 还是 shared？
- [ ] Event 请求修改 vs 直接修改——阈值在哪里？
