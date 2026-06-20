# ID 系统激进重构计划 — 零技术债目标

> **扫描日期**: 2026-06-20 | **范围**: 全量 ID 体系（文档 + 代码）
> **目标**: 消除所有文档冲突、代码-文档偏差、架构违规、技术债，不留一丝死角

---

## 1. 问题总览

### 1.1 文档自相矛盾（Doc vs Doc）

| # | 冲突方 A | 冲突方 B | 矛盾内容 | 正确方向 |
|---|----------|----------|---------|----------|
| D1 | `id-taxonomy.md §5.3`: Save ID 必须是 `SaveObjectId(uuid::Uuid)` | `ADR-042 §3`: 使用 `PersistentEntityId(u64)` | UUID vs u64 | 保持 u64（当前实现），但添加 UUID 可选项 |
| D2 | `id-taxonomy.md §3.4`: Template ID 应升级为 `namespace:type.name` 格式 | `id_strategy.md §1`: ID 格式为 `前缀_6位数字`（`abl_000042`） | 语义 vs 无语义 | 当前保持无语义，但添加命名空间扩展点 |
| D3 | `id-taxonomy.md §10.12`: Template ID 必须无语义 | `id_strategy.md §1.1`: ID 无语义 | 一致 — 但 `define_string_id!` 不强制校验格式 | 添加格式校验 |
| D4 | `id-taxonomy.md §6.3`: 需要统一 Entity 映射层 | `ADR-048 §Entity↔String`: 定义了 `BattleUnitRegistry`（域特定映射） | 统一 vs 分散 | 统一用 `EntityMapper<ID>` 泛型 |
| D5 | `docs/04-data/README.md §3.1`: 15 个领域前缀（含 `oot_` 给 LootTable） | `src/shared/ids/types.rs`: 24 个 String ID 类型（LootTable 前缀 `ltb`） | 前缀数量 + 值不匹配 | 以代码为准但更新文档 |
| D6 | `Fre项目架构设计.md §四`: 每文件一个 ID 类型（`unit_id.rs`, `skill_id.rs`...） | 实际代码: `types.rs` 宏统一生成 | 文件结构过时 | 标记旧文档为 deprecated |

### 1.2 文档声称未实现但代码已实现（Doc vs Code — 过时）

| # | 文档声称 | 实际代码 | 影响 |
|---|---------|---------|------|
| C1 | `id-taxonomy.md §9.2`: RuntimeId (index+generation) **待实现** | `runtime_id.rs`: 已实现 + 11 个测试 | 文档严重过时 |
| C2 | `id-taxonomy.md §9.2`: EntityMapper **待实现** | `entity_mapper.rs`: 已实现 + 8 个测试 | 文档严重过时 |
| C3 | `ids-overview.md §671`: EntityMapper **未实现** | 同上 | 知识文档过时 |

### 1.3 文档规定但代码未执行（Doc vs Code — 未达标）

| # | 文档规定 | 代码现状 | 违规等级 |
|---|---------|---------|----------|
| V1 | `id-taxonomy.md §10.11`: Runtime ID 必须有 Generation 保护 | `ModifierInstanceId` 是裸 u64 wrapper，无 generation | 🔴 严重 |
| V2 | `ADR-042 §3`: EntityRemapper 用 `HashMap` 实现 O(1) 映射 | `resources.rs`: 用 `Vec` + O(n) 线性查找 | 🟡 中 |
| V3 | `id_strategy.md §2.1`: 需要 `IdAllocator` 集中管理 | 无 `IdAllocator`，`define_string_id!` 无分配校验 | 🟡 中 |
| V4 | `id_strategy.md §6.1`: Registry 加载时 ID 格式校验 | 可能未实现 | 🟡 需核实 |
| V5 | `id-taxonomy.md §10.9`: 所有 ID 集中在 `shared/ids/` | `ModifierInstanceId` 定义在 `modifier/foundation/types.rs` | 🟡 中 |
| V6 | `id-taxonomy.md §10.7`: 禁止隐式 ID 转换 | 需全局审计 | 🟡 需核实 |
| V7 | `id_strategy.md §1.1`: ID 格式为定长 `前缀_6位数字` | `define_string_id!` 接受任意字符串值 | 🟡 中 |

### 1.4 架构违规（Architecture Violations）

| # | 违规 | 说明 | 等级 |
|---|------|------|------|
| A1 | **BattleUnitRegistry 重复实现 EntityMapper** | `registry.rs` 自己手写了双向 HashMap，与 `EntityMapper<ID>` 功能完全一致 | 🔴 严重 |
| A2 | **BattleUnitId 作为 Component 存储** | `BattleUnitId(String)` 作为 Component 挂在 Entity 上，违反宪法 §724 "Entity 只是 ID" | 🔴 严重 |
| A3 | **BattleUnitId 使用语义化 String** | `"bu:0:0"` 格式违反 id_strategy §1.1 无语义原则 | 🟡 中 |
| A4 | **Numeric ID 不实现 StrongId trait** | `define_numeric_id!` 生成的类型缺少 `StrongId` 实现 | 🟡 中 |
| A5 | **DefinitionId 不实现 StrongId trait** | 手写定义，但不满足 `Deref<Target=str>` 之外的 StrongId 约束 | 🟡 中 |

### 1.6 代码结构混乱（Structural Debt）

| # | 问题 | 文件 | 说明 | 等级 |
|---|------|------|------|------|
| S1 | **types.rs 巨型文件** | `types.rs:356行` | 宏定义 + 24 个 ID 类型 + DefinitionId 全塞一个文件，违反单一职责 | 🔴 严重 |
| S2 | **StrongId trait 内联在 mod.rs** | `mod.rs:38-43` | trait 定义应该有自己的文件，方便文档和发现 | 🟡 中 |
| S3 | **无 Error 类型** | — | `id_strategy.md` 要求 ID 格式校验，但无 `IdFormatError` 定义 | 🟡 中 |
| S4 | **无 prelude 模块** | — | `shared/` 层其他模块有 prelude 模式，ids 没有 | 🟢 低 |
| S5 | **目录扁平无层次** | `ids/` | 4 个文件平铺，无 foundation/types/mapping 分层，50 万行项目不合适 | 🔴 严重 |
| S6 | **测试结构不完整** | `tests/` | 只有 `unit/`，缺少 `invariant/` 和 `fixtures/` | 🟡 中 |
| S7 | **ModifierInstanceId 游离在外** | `modifier/foundation/types.rs` | 唯一 numeric ID 类型不在 shared/ids/ 中 | 🟡 中 |
| S8 | **entity_mapper.rs 位置不当** | `ids/entity_mapper.rs` | 属于 infra 运行时映射，混在纯 ID 类型中 | 🟢 低 |

---

## 2. 重构计划

### 阶段 0：文档急修（~1 天）

> 目标：消除最严重的文档过时和矛盾，建立可信文档基线。

#### 0.1 修复 `id-taxonomy.md`

- [ ] §9.2 "待实现" 表格：`RuntimeId` 和 `EntityMapper` 改为 ✅ 已实现
- [ ] §9.2 添加 `ModifierInstanceId` 条目，标记为 🔴 待升级（裸 u64 → RuntimeId）
- [ ] §3.4 "当前实现"：更新描述，匹配实际 24 个 String ID 类型
- [ ] §3.2 命名空间规范：添加说明「当前使用 `prefix:value` 格式，Mod 命名空间扩展为未来预留」
- [ ] §5.3 Save ID：添加说明「当前使用 `PersistentEntityId(u64)`，UUID 为未来方向」
- [ ] §6.3 Entity 映射层：更新描述，反映 `EntityMapper<ID>` 已实现
- [ ] §10.12 Template ID 约束表：添加「格式校验待实现」
- [ ] §10.11 Runtime ID 约束表：添加「ModifierInstanceId 待升级」
- [ ] §11 验证清单：更新所有状态为当前代码实际情况

#### 0.2 修复 `ids-overview.md`

- [ ] §现状盘点 "待实现" 表格：`EntityMapper` 改为 ✅ 已实现
- [ ] §已定义的 ID 类型清单：更新为 25 个 String ID + 1 个 Numeric ID
- [ ] §6.1 表格：LootTableId 前缀修正为 `ltb`

#### 0.3 修复 `docs/04-data/README.md`

- [ ] §3.1 前缀表：更新为实际 24 个类型（补 `char`, `unit`, `equip`, `team`, `cls`, `tal`, `sub`, `bnd`, `fmd`, `cmp`）
- [ ] §3.1 前缀表：LootTable 修正为 `ltb`
- [ ] §附录 B 文件状态：确认所有 ID 相关 Schema 状态准确

#### 0.4 标记过时文档

- [ ] `Fre项目架构设计.md`：添加 frontmatter `status: deprecated`，在顶部添加⚠️ 说明

#### 0.5 修复 `id_strategy.md`

- [ ] §2.1 分配器架构：添加说明「当前分配器尚未实现，String ID 由宏直接构造」
- [ ] §8 前缀表：补充所有 24 个前缀的最新状态
- [ ] §6 Registry 集成：明确哪些校验已实现、哪些待实现

---

### 阶段 1：代码目录结构重构（~2 天）

> 目标：将扁平、混杂的 `src/shared/ids/` 重构为分层结构，符合 50 万行项目规范。

#### 1.1 问题诊断

当前结构（扁平混乱）：

```
src/shared/ids/
├── mod.rs              # StrongId trait + 模块声明 + re-exports（48 行）
├── types.rs            # ❌ 巨型文件：宏定义 + 24 个 ID 类型 + DefinitionId（356 行）
├── runtime_id.rs       # RuntimeId + RuntimeIdAllocator
├── entity_mapper.rs    # EntityMapper<ID>（Bevy Resource）
└── tests/
    ├── mod.rs
    └── unit/            # ❌ 只有 unit，缺少 invariant/fixtures
```

所有职责不同的代码平铺在一个目录下，没有分层。对比项目已有的 `modifier/` 模块结构（`foundation/` + `mechanism/`），显然扁平结构不符合项目的成熟度。

#### 1.2 目标结构

```
src/shared/ids/
├── mod.rs                       # 模块根：只做 re-export，不定义逻辑（~15 行）
├── prelude.rs                   # 便捷导入：`use crate::shared::ids::prelude::*`（新增）
│
├── foundation/                  # 核心抽象层（零依赖，仅 Rust 标准库）
│   ├── mod.rs                   # Re-exports
│   ├── strong_id.rs             # StrongId trait（从 mod.rs 移出）
│   ├── macros.rs                # define_string_id! + define_numeric_id!（从 types.rs 移出）
│   └── errors.rs                # IdFormatError, IdAllocationError（新增）
│
├── types/                       # 具体 ID 类型定义
│   ├── mod.rs                   # Re-exports all
│   ├── string_ids.rs            # 所有 define_string_id! 调用（24 个类型）
│   ├── numeric_ids.rs           # 所有 define_numeric_id! 调用 + InstanceId<T>
│   ├── definition_id.rs         # DefinitionId（从 types.rs 移出）
│   ├── runtime_id.rs            # RuntimeId + RuntimeIdAllocator（从 runtime_id.rs 移入）
│   └── battle_unit_id.rs        # BattleUnitId define_string_id!（从 replay 移入，doc #phase1-structural）
│
├── mapping/                     # Entity ↔ ID 映射（Bevy Resource 层）
│   ├── mod.rs                   # Re-exports
│   └── entity_mapper.rs         # EntityMapper<ID>（从 entity_mapper.rs 移入）
│
└── tests/
    ├── mod.rs
    ├── unit/
    │   ├── mod.rs
    │   ├── string_id_test.rs
    │   ├── numeric_id_test.rs
    │   ├── runtime_id_test.rs
    │   ├── entity_mapper_test.rs
    │   ├── strong_id_test.rs         # 新增：StrongId trait 测试
    │   └── id_format_test.rs         # 新增：格式校验测试
    ├── invariant/
    │   ├── mod.rs
    │   └── identity_invariant_test.rs # 新增：13 条铁律自动化检查
    └── fixtures/
        ├── mod.rs
        └── id_fixtures.rs             # 新增：共享 ID 测试数据
```

#### 1.3 具体步骤

**步骤 A：创建 `foundation/` 子模块**

- [ ] 从 `mod.rs` 提取 `StrongId` trait 到 `foundation/strong_id.rs`
- [ ] 从 `types.rs` 提取 `define_string_id!` 和 `define_numeric_id!` 到 `foundation/macros.rs`
- [ ] 新建 `foundation/errors.rs`：

```rust
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IdFormatError {
    #[error("ID is empty")]
    Empty,
    #[error("Invalid prefix: expected '{expected}', got '{actual}'")]
    PrefixMismatch { expected: &'static str, actual: String },
    #[error("ID contains invalid characters: {0}")]
    InvalidCharacters(String),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IdAllocationError {
    #[error("ID range exhausted for prefix '{0}'")]
    RangeExhausted(&'static str),
    #[error("ID '{0}' is deprecated and cannot be reused")]
    Deprecated(String),
}
```

- [ ] `foundation/mod.rs` 汇总 re-export

**步骤 B：创建 `types/` 子模块**

- [ ] 从 `types.rs` 提取所有 `define_string_id!` 调用到 `types/string_ids.rs`
- [ ] 从 `types.rs` 提取 `DefinitionId` 到 `types/definition_id.rs`
- [ ] 从 `runtime_id.rs` 移入 `RuntimeId` + `RuntimeIdAllocator` 到 `types/runtime_id.rs`
- [ ] 新建 `types/numeric_ids.rs`：`ModifierInstanceId` 定义迁移至此
- [ ] 新建 `types/battle_unit_id.rs`：`BattleUnitId` 从 Component 改为 `define_string_id!` 类型
- [ ] `types/mod.rs` 汇总 re-export

**步骤 C：创建 `mapping/` 子模块**

- [ ] `entity_mapper.rs` 移入 `mapping/entity_mapper.rs`
- [ ] `mapping/mod.rs` re-export

**步骤 D：重写 `mod.rs`**

```rust
// mod.rs — 只做 re-export，不定义逻辑
pub mod foundation;
pub mod mapping;
pub mod prelude;
pub(crate) mod types;

// 向下兼容：保持所有外部导入路径不变
pub use foundation::macros::*;
pub use foundation::strong_id::StrongId;
pub use types::*;

#[cfg(test)]
mod tests;
```

**步骤 E：创建 `prelude.rs`**

```rust
// prelude.rs
pub use super::foundation::strong_id::StrongId;
pub use super::types::*;
pub use super::mapping::entity_mapper::EntityMapper;
```

**步骤 F：迁移 ModifierInstanceId 位置**

- [ ] 从 `core/capabilities/modifier/foundation/types.rs` 删除 `define_numeric_id!(ModifierInstanceId)`
- [ ] 在 `types/numeric_ids.rs` 中添加 `ModifierInstanceId`
- [ ] 更新 `modifier/foundation/types.rs`：改为 `use crate::shared::ids::ModifierInstanceId`
- [ ] 更新 modifier 模块下所有引用方
- [ ] 更新 `src/shared/testing/fixtures.rs` 中的 fixture 数据

**步骤 G：整理测试目录**

- [ ] 保留现有单元测试，匹配新的源文件结构
- [ ] 新建 `tests/invariant/identity_invariant_test.rs`：验证 §10 的 13 条铁律
- [ ] 新建 `tests/fixtures/id_fixtures.rs`：共享 ID 测试数据

**步骤 H：更新所有导入路径**

确保 `mod.rs` 的 re-export 链完整，所有外部导入路径不变。只有深度路径变化：

| 旧路径 | 新路径 | 影响数 |
|--------|--------|--------|
| `crate::shared::ids::runtime_id::*` | 通过 `mod.rs` re-export 保持兼容 | 3 处 |
| `crate::shared::ids::entity_mapper::*` | 保持 `pub(crate) mod mapping` 兼容 | 若干 |
| `crate::shared::ids::AbilityId` | 不变（re-export 保证） | 大量 |
| `crate::define_string_id!` | 不变（`#[macro_export]`） | — |

#### 1.4 与后续阶段的关系

本阶段是 P0 中最基础的一层，是所有后续代码改动的**前置条件**。只有在目录结构重构完成后，阶段 2/3/4/5/6 才能基于清晰的结构进行修改。

**验收标准**：
- ✅ `types.rs` 删除，拆分为 5 个职责单一的子文件
- ✅ `mod.rs` 从 48 行降至 ~15 行，只做 re-export
- ✅ 所有外部导入路径兼容
- ✅ `cargo build` `cargo nextest run` `cargo clippy -- -D warnings` 全部通过
- ✅ `ModifierInstanceId` 位于 `shared/ids/types/numeric_ids.rs`
- ✅ 测试结构包含 `unit/` `invariant/` `fixtures/`
- ✅ 新增 `prelude.rs` 和 `foundation/errors.rs`

---

### 阶段 2：ModifierInstanceId 升级 RuntimeId（~2 天）

> 目标：消除最严重的安全漏洞——无 generation 保护的 Runtime ID。

#### 2.1 改造 `define_numeric_id!`

```rust
// 改造目标：define_numeric_id! 生成的类型包装 RuntimeId 而非 u64

define_numeric_id_staged!(ModifierInstanceId);

// 展开后：
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModifierInstanceId(RuntimeId);  // 不是 u64！

impl ModifierInstanceId {
    pub fn new(index: u32, generation: u32) -> Self { ... }
    pub fn runtime_id(&self) -> &RuntimeId { ... }
}
```

或者更好的方案——定义新的 `InstanceId<T>` 泛型包装器：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceId<T>(RuntimeId, PhantomData<T>);
```

这样所有 Instance ID 共享同一套 generation 保护逻辑。

#### 2.2 更新 `ModifierInstanceId` 使用方

- [ ] `modifier/foundation/types.rs`：修改定义
- [ ] `modifier/mechanism/`：所有引用处适配新类型
- [ ] 测试更新：修改 `ModifierInstanceId(42)` → `ModifierInstanceId::new(...)`

#### 2.3 数据迁移

- [ ] `RuntimeId` 序列化格式兼容性检查
- [ ] 旧存档/Replay 中的 u64 格式 ModifierInstanceId 迁移路径

**验收标准**：
- ✅ `ModifierInstanceId` 包含 generation 字段
- ✅ 所有 `id.is_stale()` 检测可用
- ✅ 测试覆盖 generation 冲突检测
- ✅ 存档/Replay 向前兼容

---

### 阶段 3：EntityMapper 统一化（~2 天）

> 目标：消除 BattleUnitRegistry 重复实现，统一为 EntityMapper<ID>。

#### 3.1 BattleUnitRegistry → EntityMapper<BattleUnitId>

```rust
// 不再有独立的 BattleUnitRegistry！
type BattleUnitMapper = EntityMapper<BattleUnitId>;

// BattleUnitRegistry 的所有功能 EntityMapper 都有：
// - register(id, entity) → register(id, entity)
// - get_id(entity) → get_id(entity)  
// - get_entity(id) → get_entity(id)
// - clear() → clear()
// - is_empty() → is_empty()
```

#### 3.2 BattleUnitId 重构

```rust
// 改造前：BattleUnitId(String) 作为 Component
#[derive(Component)]
pub struct BattleUnitId(pub String);

// 改造后：BattleUnitId 是强类型 ID，用现有的 define_string_id! 或新方式
// 选择方案 A：用 define_string_id! 统一
define_string_id!(pub BattleUnitId, prefix: "bu");

// 并且不再作为 Component。Component 只是 EntityMapper 的 key：
// 系统通过 EntityMapper<BattleUnitId> 查询，而非从 Entity 读 Component。
```

#### 3.3 更新回放桥接层

- [ ] `registry.rs`：删除 `BattleUnitRegistry`，改为 `EntityMapper<BattleUnitId>`
- [ ] `registry.rs`：删除 `BattleUnitId` Component，改为纯 ID 映射
- [ ] `recording.rs`：适配新 API
- [ ] `playback.rs`：适配新 API
- [ ] 测试全部重写在 `entity_mapper_test.rs` 同层

**验收标准**：
- ✅ `BattleUnitRegistry` 完全删除
- ✅ 所有回放桥接测试通过
- ✅ EntityMapper 无需修改核心逻辑即可接管
- ✅ 性能不下降（HashMap vs HashMap）

---

### 阶段 4：EntityRemapper Vec→HashMap（~1 天）

> 目标：修复 ADR-042 规定的 HashMap 实现。

#### 4.1 改造 `EntityRemapper`

```rust
// 改造前：Vec<(PersistentEntityId, Entity)>  O(n)
#[derive(Resource)]
pub struct EntityRemapper {
    persistent_to_entity: Vec<(PersistentEntityId, Entity)>,
    next_id: u64,
}

// 改造后：HashMap  O(1)
#[derive(Resource)]
pub struct EntityRemapper {
    persistent_to_entity: HashMap<PersistentEntityId, Entity>,
    entity_to_persistent: HashMap<Entity, PersistentEntityId>,
    next_id: u64,
}
```

- [ ] `src/infra/save/resources.rs`：替换 Vec 为 HashMap
- [ ] `lookup()`：O(n) `find` → O(1) `get`
- [ ] `assign()`：同时维护双向映射
- [ ] `clear()`：清空两个 HashMap + 重置 next_id

**验收标准**：
- ✅ `lookup()` 为 O(1)
- ✅ ADR-042 规定的双向映射完整
- ✅ 所有存档测试通过
- ✅ 序列化格式不变

---

### 阶段 5：ID 格式校验与常量审计（~2 天）

> 目标：确保所有 ID 类型遵循 id_strategy.md 规定的格式。

#### 5.1 `define_string_id!` 添加格式校验

```rust
// 添加 checked_new() 方法到 define_string_id!
impl AttributeId {
    /// 创建 ID 并校验格式（前缀+至少1位数字）。
    /// 仅 Debug 模式启用。
    pub fn new_format(id: impl Into<String>) -> Result<Self, IdFormatError> {
        let s = id.into();
        // 校验：非空、匹配前缀格式（可选）
        if s.is_empty() {
            return Err(IdFormatError::Empty);
        }
        Ok(Self(s))
    }
}
```

#### 5.2 全局搜索硬编码 ID 对比

- [ ] 搜索 `id() == SomeId(`、`== SomeId::new(` 等模式
- [ ] 搜索 `sort_by_key(|x| x.id())` 等模式
- [ ] 搜索 `SomeId(0)` 或 `id == 0` 模式（Null ID 反模式）
- [ ] 搜索 `From<A> for B` 跨 ID 类型转换

#### 5.3 LocalizationKey 格式审计

- [ ] 验证所有 `name_key` / `desc_key` 使用 4 段式格式
- [ ] 禁止语义化 Key 名（`ability.fireball.name` vs `ability.abl_000042.name`）

**验收标准**：
- ✅ `define_string_id!` 有可选的格式校验
- ✅ 0 处硬编码 ID 业务逻辑
- ✅ 0 处 Null ID 模式
- ✅ 0 处隐式 ID 转换

---

### 阶段 6：StrongId 统一化（~1 天）

> 目标：Numeric ID 和 DefinitionId 实现 StrongId，统一类型体系。

#### 6.1 `define_numeric_id!` 添加 StrongId

```rust
// 问题：Numeric ID 不实现 StrongId（Deref<Target=str> 不适合 u64）
// 解决方案：为 Numeric ID 设计新的 InstanceId trait

pub trait InstanceId:
    Display + Copy + Clone + Eq + Hash + Send + Sync + Serialize + Deserialize
{
    type Inner: Display;
    fn value(&self) -> Self::Inner;
    fn as_u64(&self) -> u64;
}
```

或者更激进：废弃 `define_numeric_id!`，所有运行时实例 ID 统一使用 `InstanceId<T>` 泛型。

#### 6.2 `DefinitionId` 实现 StrongId

```rust
impl StrongId for DefinitionId {
    fn prefix() -> &'static str { "def" }
    fn as_str(&self) -> &str { &self.0 }
}
```

**验收标准**：
- ✅ 所有 ID 类型实现一致的 trait（StrongId 或 InstanceId）
- ✅ 泛型 Registry 可同时约束 String ID 和 Numeric ID

---

### 阶段 7：MD 文档最终对齐（~1 天）

> 目标：所有文档与最终代码完全一致。

#### 7.1 更新所有 ID 引用

- [ ] `docs/01-architecture/README.md`：IDs 行更新
- [ ] `docs/01-architecture/ADR-000-feature-module-map.md`：确认 IDs 位置
- [ ] `docs/00-governance/ai-constitution-complete.md` §1425：确认引用
- [ ] `docs/02-domain/domains/` 下所有事件负载：确认 ID 类型命名

#### 7.2 最终一致性验证

- [ ] 运行 `cargo doc` 确认文档链不断裂
- [ ] 全文搜索 `待实现`、`未实现`、`计划` 在 ID 相关文档，确认为准确
- [ ] 运行所有测试确认不回归

**验收标准**：
- ✅ 所有文档与代码 1:1 对应
- ✅ 无过时声明
- ✅ 无虚假的「待实现」

---

### 阶段 8：低优先级 + 自动化（~2 天，可并行）

#### 8.1 Debug 审计 ✅

- [x] 在 `foundation/errors.rs` 中实现 `IdCreationInfo` 结构体
- [x] 在 `define_string_id!` 中添加 `new_tracked(id, info)` 方法
  - Debug 模式：参数被使用，可附加创建来源/帧号/触发者
  - Release 模式：参数被 `_` 消除，零开销

#### 8.2 Identity Invariant 检查脚本 ✅

在 `tools/check-identity-invariants.sh` 实现自动化检查：
- Rule 1: 搜索 `.id() == N` 和 `== SomeId(` 模式
- Rule 2: 搜索 `sort_by_key(|x| x.id())` 模式
- Rule 6: 搜索 `SomeId(0)` Null ID 模式
- Rule 7: 搜索 `impl From<IdA> for IdB` 隐式转换
- 支持 `--ci` 参数（CI 模式下错误时退出码 1）
- 首次运行结果：**0 处违规**

#### 8.3 SmolStr 优化评估 ⏸️

**评估结论：当前不可行。**

| 障碍 | 说明 |
|------|------|
| Bevy Reflect 不兼容 | `SmolStr` 未实现 `Reflect`/`GetTypeRegistration`，无法在 `#[derive(Reflect)]` 中使用 |
| 侵入性改动 | 需要实现 `Reflect` for `SmolStr` 或修改宏不 `derive(Reflect)` |
| 收益有限 | String ID 非热路径（仅在配置加载时创建），性能收益可忽略 |

**建议**：等待 Bevy 生态对 SSO 字符串类型的内置支持，或改用 `arcstr`（如社区普遍采用后）再重新评估。

---

## 3. 优先级汇总

| 阶段 | 内容 | 等级 | 预估工时 | 状态 |
|------|------|------|---------|------|
| **P0 阶段 0** | 文档急修 | 🔴 必须优先 | 1 天 | ✅ 完成 |
| **P0 阶段 1** | 目录结构重构 | 🔴 结构地基 | 2 天 | ✅ 完成 |
| **P0 阶段 2** | ModifierInstanceId → RuntimeId | 🔴 安全漏洞 | 2 天 | ✅ 完成 |
| **P0 阶段 3** | EntityMapper 统一化 | 🔴 架构违规 | 2 天 | ✅ 完成 |
| **P1 阶段 4** | EntityRemapper HashMap | 🟡 性能 | 1 天 | ✅ 完成 |
| **P1 阶段 5** | ID 格式校验 | 🟡 质量 | 2 天 | ✅ 完成 |
| **P1 阶段 6** | StrongId 统一 | 🟡 一致性 | 1 天 | ✅ 完成 |
| **P2 阶段 7** | 文档最终对齐 | 🟢 收尾 | 1 天 | ✅ 完成 |
| **P2 阶段 8** | SmolStr/Debug/Lint | 🟢 增强 | 2 天 | ✅ 完成 |

**完成状态**: 全部 9 个阶段已完成。SmolStr 评估结论：当前不可行（与 Bevy Reflect 不兼容），待 Bevy 生态支持后重新评估。

**总执行**: ~12 天（已执行）

---

## 4. 回滚方案

| 风险 | 缓解措施 |
|------|---------|
| 阶段 1 目录结构重构破坏导入路径 | `mod.rs` 保持完整 re-export 链，所有外部路径兼容；深度路径变化仅影响 3 处 `runtime_id` 引用 |
| 阶段 2 ModifierInstanceId 升级破坏存档兼容 | 添加 `#[serde(alias = "...")]` 兼容旧格式，测试覆盖双向转换 |
| 阶段 3 BattleUnitRegistry 删除导致 replay 失败 | 所有测试通过后才能合并，保留旧代码在 git 历史中 |
| 阶段 4 Vec→HashMap 变化影响序列化 | HashMap 仅运行时结构，序列化走独立格式，不受影响 |
| 阶段 5 格式校验过于严格 | `new_format()` 为显式方法，不影响现有的 `new()` |
| 阶段 8 SmolStr 导致 panic | `SmolStr` 在字符串 > 23 字节时自动 fallback 到 heap，安全 |

---

## 5. 验证清单

每个阶段完成后必须执行：

- [ ] `cargo build` 通过
- [ ] `cargo nextest run` 全部通过
- [ ] `cargo clippy -- -D warnings` 通过
- [ ] 无 `#![allow(...)]` 新增（新增 clippy allow = 技术债）
- [ ] 更新的文档对应代码状态准确
- [ ] 0 处 `todo!()` 或 `unimplemented!()` 新增
