---
id: foundation.id-taxonomy
title: ID 分类体系 — 五类 ID 的职责边界与生命周期
status: accepted
owner: data-architect
created: 2026-06-28
layer: definition
replay-safe: true
---

# ID 分类体系 — 五类 ID 的职责边界与生命周期

> **前置文档**: `docs/04-data/foundation/id_strategy.md`（ID 格式与分配机制）
> **本文档是 ID 体系的分类学展开**，定义项目中所有 ID 的类型层次、职责边界和生命周期约束。

---

## 1. 为什么需要分类？

大型 SRPG 项目中，不同场景对 ID 的要求截然不同：

| 场景 | 需求 | 错误代价 |
|------|------|---------|
| 配置表引用 | 永久稳定、可序列化、Mod 友好 | 配表引用失效 → 全量返工 |
| 运行时实例 | 每局唯一、高性能、可回收 | ID 复用 → 引用悬空 |
| 存档兼容 | 跨版本、跨存档格式、可迁移 | 存档崩溃 → 玩家数据丢失 |
| ECS 引用 | 零成本、引擎原生 | 业务层泄露 Entity → 重建失效 |
| 网络同步 | 确定性、低带宽 | 同步错乱 → 多人游戏不可玩 |

**核心原则：不同生命周期的 ID 必须使用不同类型，禁止混用。**

---

## 2. 五类 ID 总览

```
┌─────────────────────────────────────────────────────────────┐
│                    ID 分类体系                               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─ Template ID（模板标识）────────────────────────────┐   │
│  │  来源：配置文件（YAML/RON/JSON）                     │   │
│  │  存储：String 或 SmolStr                            │   │
│  │  特点：永久稳定、不可重用、Mod 友好                   │   │
│  │  示例：CharacterTemplateId("char:knight")          │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─ Runtime ID（运行时实例标识）────────────────────────┐   │
│  │  来源：IdAllocator 动态分配                          │   │
│  │  存储：index + generation（防止 ID 复用）            │   │
│  │  特点：每局唯一、可回收、ECS 不可见                   │   │
│  │  示例：UnitId { index: 3, generation: 2 }           │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─ Save ID（存档标识）────────────────────────────────┐   │
│  │  来源：首次实例化时分配                               │   │
│  │  存储：Uuid                                         │   │
│  │  特点：跨存档、跨版本、跨加载周期                     │   │
│  │  示例：SaveObjectId(uuid!("..."))                   │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─ Entity（ECS 引擎标识）─────────────────────────────┐   │
│  │  来源：Bevy ECS 自动分配                             │   │
│  │  存储：bevy::ecs::entity::Entity                    │   │
│  │  特点：零成本、仅基础设施层可见                       │   │
│  │  示例：Entity(42u64)                                │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─ Network ID（网络同步标识）─────────────────────────┐   │
│  │  来源：网络会话协商                                   │   │
│  │  存储：确定性哈希或协商 ID                            │   │
│  │  特点：跨客户端确定性、低带宽                         │   │
│  │  示例：NetId(u32)                                   │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Template ID — 配置表之锚

### 3.1 定义

配置文件中的静态内容标识。内容团队在 YAML/RON/JSON 中定义的 ID。

### 3.2 特性

| 特性 | 说明 |
|------|------|
| **永久稳定** | 一旦发布，ID 永不改变 |
| **可序列化** | 配置文件、存档、网络传输中以字符串形式存在 |
| **Mod 友好** | 支持命名空间前缀（如 `core:unit.knight`） |
| **无运行时状态** | 纯数据标识，不关联任何 ECS 组件 |

### 3.3 命名空间规范

```
<namespace>:<type>.<name>
     │          │       │
     │          │       └── 内容名称（无语义，用编号更佳）
     │          └── 内容类型（unit/ability/effect/...）
     └── 命名空间（core/dlc1/mod_abc）
```

**示例**：
```
core:unit.knight          # 核心内容
core:ability.fireball     # 核心能力
dlc1:unit.knight_fire     # DLC 内容
mod_abc:unit.knight_dark  # Mod 内容
```

**禁止**：
- ❌ `id: 1001` — 数字 ID 在 DLC 合并时必然冲突
- ❌ `id: fireball` — 无命名空间，Mod 间必然冲突

### 3.4 当前实现

使用 `define_string_id!` 宏定义的 ID 类型：

```rust
define_string_id!(pub CharacterId, prefix: "char");
define_string_id!(pub AbilityId, prefix: "abl");
define_string_id!(pub EffectId, prefix: "eff");
// ... 共 22 个
```

> **待升级**: 当前的 `prefix:value` 格式应升级为 `namespace:type.name` 格式以支持 Mod。

### 3.5 生命周期

```
定义阶段 → 发布阶段 → 永久有效
   │          │            │
   配置编写   v1.0 发布    ID 永不改变
              ↓
         Deprecated（标记废弃，但 ID 保留）
              ↓
         Archived（超 N 个大版本后可清理引用）
```

---

## 4. Runtime ID — 运行时实例之魂

### 4.1 定义

运行时动态分配的实例唯一标识。每次游戏运行独立生成。

### 4.2 Generation 机制

**核心问题**：ID 被回收后如果立即复用，持有旧 ID 的引用会指向错误对象。

**解决方案**：index + generation 双字段设计（类似 Bevy Entity）。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeId {
    pub(crate) index: u32,      // 数组索引（快速查找）
    pub(crate) generation: u32, // 代际计数器（防止复用）
}
```

**工作原理**：

```
第一次分配: UnitId { index: 0, generation: 0 }
    ↓ 使用中...
回收:       generation 变为 1，index 0 进入空闲池
    ↓
第二次分配: UnitId { index: 0, generation: 1 }  ← 同 index，不同 generation
    ↓
旧引用 UnitId { index: 0, generation: 0 } 
    → generation 不匹配 → 检测到已失效 → 安全拒绝
```

### 4.3 当前实现

当前项目使用 `ModifierInstanceId` 作为唯一的 Runtime ID：

```rust
define_numeric_id!(ModifierInstanceId);
// 仅存储 u64，无 generation 机制
```

> **待升级**: 所有 Runtime ID 应迁移到 `RuntimeId` 格式（index + generation）。

### 4.4 生命周期

```
请求分配 → 活跃使用 → 回收（进入空闲池）→ 可重新分配
    │          │            │
    Allocator  游戏运行     despawn 事件
```

---

## 5. Save ID — 存档之桥

### 5.1 定义

跨存档、跨版本的持久化标识。用于将运行时状态序列化到存档文件。

### 5.2 为什么需要单独的 Save ID？

| 问题 | 原因 |
|------|------|
| Runtime ID 不可靠 | 每次加载后 Entity 重建，Runtime ID 变化 |
| Template ID 不够 | 多个同模板实例需要不同标识（两个骑士） |
| Entity 不可序列化 | Entity 是 ECS 内部 ID，存档后无意义 |

### 5.3 设计

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SaveObjectId(uuid::Uuid);
```

**特点**：
- 基于 UUID，全局唯一，无需中央分配器
- 存档文件中以字符串形式存储
- 加载时通过 `SaveObjectId → Entity` 映射恢复

### 5.4 当前实现

> **未实现**。存档系统当前使用 `ErrorContext<String>` 包装错误，尚无独立的 Save ID 体系。

### 5.5 生命周期

```
首次实例化 → 存入存档 → 跨版本加载 → 永久有效
    │            │            │
  Uuid生成    序列化写入   反序列化恢复
```

---

## 6. Entity — ECS 引擎之骨

### 6.1 定义

Bevy ECS 内部的实体标识符。引擎自动分配和管理。

### 6.2 硬规则：Domain 层禁止裸 Entity

> **这是本项目最重要的 ID 规则之一。**

```
┌─────────────────────────────────────────────────┐
│  Infrastructure Layer（基础设施层）              │
│  ✅ Entity 自由使用                             │
│  ✅ Entity ↔ 业务 ID 映射                       │
├─────────────────────────────────────────────────┤
│  Domain Layer（领域层）                          │
│  🟥 禁止裸 Entity 作为业务对象标识               │
│  🟥 禁止 Entity 出现在领域规则、Def、事件中      │
│  ✅ 只允许显式命名的强类型 ID                    │
├─────────────────────────────────────────────────┤
│  Application Layer（应用层）                     │
│  🟡 通过 ACL 间接使用 Entity                     │
└─────────────────────────────────────────────────┘
```

**理由**：
1. Entity 在 Entity 重建后失效（存档加载、场景重载）
2. Entity 在不同系统间传递时丧失类型安全
3. Entity 无法序列化到配置文件
4. Entity 无法跨网络确定性同步

**已有宪法依据**：
- `docs/00-governance/ai-constitution-complete.md` §724: "Entity 只是 ID"
- `docs/00-governance/coding-rules.md` §129: "Entity 仅用于引用实体"
- `docs/02-domain/capabilities/ui-presentation.md` INV-UI-002: "Widget 组件禁止包含 Entity 字段"

### 6.3 Entity 映射层

基础设施层维护 `业务 ID ↔ Entity` 的双向映射：

```rust
pub struct EntityMapper {
    id_to_entity: HashMap<UnitId, Entity>,
    entity_to_id: HashMap<Entity, UnitId>,
}
```

### 6.4 当前实现

当前项目通过 `integration/` 模块的 ACL 隔离 Entity 访问，但缺少统一的映射层。

---

## 7. Network ID — 网络同步之锚

### 7.1 定义

多人游戏网络同步中使用的确定性标识。

### 7.2 特性

| 特性 | 说明 |
|------|------|
| **确定性** | 所有客户端对同一对象生成相同的 NetId |
| **低带宽** | u32 或更小，减少网络包大小 |
| **会话级** | 每次网络会话重新分配 |

### 7.3 当前实现

> **未实现**。项目当前为单机 SRPG，暂无网络同步需求。预留扩展点。

---

## 8. 五类 ID 的映射关系

```
配置文件                    游戏运行时                   存档文件
─────────                 ──────────                  ─────────
CharacterId("knight")     ┌─→ Entity(42)              SaveObjectId(uuid)
      │                   │     ↕
      │ Template→Runtime  │  UnitId(3, gen=2)  ←──→  SaveObjectId(uuid)
      ↓                   │     ↕
  TemplateRegistry        └─→ NetId(1001) [网络]
```

**转换规则**：

| 源 → 目标 | 转换方式 | 场景 |
|-----------|---------|------|
| Template → Runtime | `spawn_unit(template_id)` | 创建实例 |
| Runtime → Entity | `entity_mapper.get_entity(unit_id)` | ECS 查询 |
| Entity → Runtime | `entity_mapper.get_unit_id(entity)` | 事件处理 |
| Runtime → Save | `save_registry.get_save_id(unit_id)` | 存档写入 |
| Save → Runtime | `save_registry.restore_unit_id(save_id)` | 存档加载 |
| Runtime → Network | `net_id_allocator.alloc(unit_id)` | 联机同步 |

---

## 9. 当前实现差距与迁移计划

### 9.1 已实现

| 组件 | 状态 | 说明 |
|------|------|------|
| Template ID 宏生成 | ✅ | `define_string_id!` 宏，22 个类型 |
| StrongId trait | ✅ | 统一接口 |
| 基本 Serde 支持 | ✅ | 序列化/反序列化 |

### 9.2 待实现

| 组件 | 优先级 | 说明 |
|------|--------|------|
| RuntimeId (index + generation) | 🔴 高 | 防止 ID 复用，存档安全 |
| EntityMapper (双向映射) | 🔴 高 | Domain 层隔离 Entity |
| SaveObjectId (Uuid) | 🟡 中 | 存档兼容性 |
| Mod 命名空间支持 | 🟡 中 | Template ID 前缀升级 |
| Network ID | 🟢 低 | 联机功能预留 |
| IdRegistry 统一管理 | 🟡 中 | 生成/回收/映射/校验 |

### 9.3 迁移原则

1. **新增优先于迁移**：新功能使用新体系，旧代码逐步迁移
2. **存档兼容**：任何 ID 格式变更必须有迁移路径
3. **编译期安全**：类型系统强制防止跨类 ID 混用

---

## 10. 验证清单

- [ ] Template ID 使用命名空间格式（`namespace:type.name`）
- [ ] Runtime ID 使用 generation 机制
- [ ] Save ID 基于 Uuid，独立于 Runtime ID
- [ ] Domain 层代码中无裸 `Entity` 出现
- [ ] Domain 层代码中无裸 `u64` 作为业务标识
- [ ] Entity ↔ 业务 ID 映射集中在基础设施层
- [ ] 所有 ID 类型实现 StrongId trait
- [ ] 存档加载能正确恢复 ID 映射

---

## 参考文档

| 文档 | 内容 |
|------|------|
| `docs/04-data/foundation/id_strategy.md` | ID 格式、分配机制、生命周期 |
| `docs/04-data/foundation/save_architecture.md` | 存档架构（Save ID 集成） |
| `docs/00-governance/ai-constitution-complete.md` §724 | Entity 只是 ID |
| `docs/00-governance/coding-rules.md` §129 | Entity 仅用于引用实体 |
| `docs/02-domain/capabilities/ui-presentation.md` INV-UI-002 | Widget 禁止 Entity 字段 |
| `src/shared/ids/mod.rs` | StrongId trait 定义 |
| `src/shared/ids/types.rs` | 当前 ID 类型定义 |
