---
id: 01-architecture.ids-design
title: Ids Design
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
  - design
---

# Strong ID 系统设计 — 编译期类型安全基石

Version: 1.0
Status: Proposed

Source: `docs/其他/31遗漏.md`（高优先级第 1 项）

本文档定义 SRPG 项目中强类型 ID 系统的架构设计，覆盖 ID 结构、命名空间、分配策略、生命周期和使用规范。

交叉引用：
- `docs/02-domain/shared_layer_rules.md` — 共享层 Strong ID 领域规则
- `docs/02-domain/ecs_communication_rules.md` — Message 中 ID 的传递方式
- `content-pipeline.md` — 配置文件中的 ID 引用与解析

---

## 概述

🟥 强类型 ID（Strong ID）是 Rust 类型安全体系的基石（宪法 1.2.1）。通过 newtype 模式为每个业务实体创建独立的 ID 类型，在编译期彻底杜绝不同实体类型之间的 ID 混用。

**核心价值**：将 `CharacterId` 和 `SkillId` 传参混淆从运行时 Bug 降级为编译错误，消灭一整类潜在 Bug。

> **优化来源**：`docs/其他/34.md` — S 级第 1 项「全局强类型 ID 体系」深度点评

### 🔒 Strong ID 不可妥协（Non-negotiable）

🟥 **Phase 2 之后，所有 Message、Event、Save 结构体中的实体标识必须使用 Strong ID，裸 `Entity` 将产生编译错误。**

理由（来自 34.md 深度剖析）：
- Bevy 的 `Entity` 是**瞬态的**——实体被销毁后 ID 可能在下一局被复用，绝不能作为存档、回放、网络同步的标识
- 裸 `String` 没有类型安全，拼写错误要运行时才发现，千条内容后排查成本爆炸
- **最小可行性架构（MVA）节奏**：第一周只定义 SkillId 等 Newtype，此时就能跑通"两个方块互相平A掉血"的 Demo

禁止模式（编译期强制）：
```rust
// 🟥 Phase 2 后禁止：Message 使用裸 Entity
pub struct DamageApplied { source: Entity, target: Entity }

// ✅ 必须使用 Strong ID
pub struct DamageApplied { source: UnitId, target: UnitId }
```

### ID 宏：`define_id!` 一键生成

🟥 **推荐使用 `define_id!` 宏自动生成 newtype + 所有必需 trait，避免手写样板代码。**

> **优化来源**：`docs/其他/34.md` — 避坑建议「Rust 最佳实践：使用 Newtype 模式」

```rust
/// 一键定义 Strong ID 类型，自动派生所有必须 trait。
macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = IdParseError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(s.to_string()))
            }
        }
    };
}

// 使用示例：
define_id!(SkillId);
define_id!(BuffId);
define_id!(ItemId);
```

收益：新增 ID 类型只需一行 `define_id!(XxxId)`，零手写、零遗漏、全 trait 覆盖。

### ID 在存档文件中的稳定性

🟥 **存档文件必须使用 Strong ID 的字符串表示（如 `"fireball"`），禁止使用 u64 等数值型内部表示。**

> **优化来源**：`docs/其他/34.md` — B 级第 12 项「内容迁移」、D 级第 15 项「Save Version」

理由：
- u64 在不同编译、不同平台下可能变化（内存布局、分配顺序），导致存档不可读
- 字符串表示（如 `base:fireball`）天然稳定，跨编译、跨平台、跨 MOD 均可识别
- 命名空间前缀（如 `base:`、`mod_xxx:`）从第一天起就内置在 ID 中，零成本解决 MOD ID 冲突

---

## 设计原则

### 原则 1：类型即语义 🟥

🟥 每个实体类型拥有独立的 ID 类型，ID 的类型名即业务含义（宪法 1.2.1）。`UnitId` 只能代表单位，`SkillId` 只能代表技能——编译器强制执行。

### 原则 2：内部表示对使用者透明 🟩

🟩 ID 的内部存储（String / u64）是实现细节，外部代码不得依赖内部表示。所有访问通过 Display、PartialEq、Hash 等 trait 完成。

### 原则 3：ID 不可变 🟥

🟥 ID 在创建后不可修改。删除实体不等于释放 ID——被删除实体的 ID 永不复用。

### 原则 4：零业务逻辑 🟥

🟥 ID 是纯标识符，不包含任何游戏规则逻辑。不在 ID 上实现 `calculate_damage()` 等业务方法（宪法 2.1.2 数据与行为分离）。

---

## 架构

### ID 结构定义

每个 Strong ID 使用 newtype 模式，内部存储使用 `String`（便于日志、调试和人类阅读）：

```rust
// src/shared/ids/unit_id.rs

/// 战场上每个战斗单位的唯一标识。
/// 内部存储为 String，Display 格式为 `Unit(inner_value)`。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnitId(String);

impl UnitId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for UnitId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unit({})", self.0)
    }
}

impl FromStr for UnitId {
    type Err = IdParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 解析 "Unit(xxx)" 格式
        // 或直接接受裸字符串（向后兼容）
        Ok(Self(s.to_string()))
    }
}
```

### IdParseError 详细定义

`IdParseError` 必须明确以下错误变体，避免各模块各写各的错误类型：

```rust
// src/shared/ids/error.rs

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IdParseError {
    /// 输入字符串为空
    #[error("ID 字符串为空")]
    EmptyInput,

    /// 格式错误：缺少类型前缀括号（如 "warrior_001" 应为 "Unit(warrior_001)"）
    #[error("ID 格式错误：缺少类型前缀括号，输入 '{input}'")]
    InvalidFormat { input: String },

    /// 前缀不匹配：类型前缀与预期不符（如解析 UnitId 时输入 "Skill(fireball)"）
    #[error("ID 前缀不匹配：期望 '{expected}'，实际 '{actual}'")]
    PrefixMismatch { expected: String, actual: String },
}
```

**错误处理建议**：
- 必填引用缺失 → `IdParseError` 作为致命错误，阻止加载
- 可选引用缺失 → `warn!` 日志 + 使用默认值，不阻断流程
- 日志格式统一：`error!(id = %raw, "ID 解析失败: {err}")`

> **优化来源**：`docs/其他/53.md` — 错误处理细节「IdParseError 具体错误变体定义」

### 必须实现的 Trait

每个 Strong ID 类型必须实现以下 trait：

| Trait | 用途 | 强制等级 |
|-------|------|----------|
| `Debug` | 调试输出 | 🟥 必须 |
| `Clone` | 值传递 | 🟥 必须 |
| `Copy` | 仅当内部为 Copy 类型时 | 🟨 优先（String 不适用） |
| `PartialEq` | 相等比较 | 🟥 必须 |
| `Eq` | 哈希键 | 🟥 必须 |
| `Hash` | HashMap / HashSet 使用 | 🟥 必须 |
| `Display` | 人类可读输出，含类型前缀 | 🟥 必须 |
| `Serialize` | 序列化（审计、回放、存档） | 🟥 必须 |
| `Deserialize` | 反序列化 | 🟥 必须 |
| `FromStr` | 字符串解析 | 🟩 推荐 |
| `Default` | 仅当有明确空值语义时 | 🟦 按需 |

### Display 格式规范 🟥

🟥 所有 ID 的 Display 格式必须包含类型前缀（宪法 1.2.2 对外输出必须使用可读标识）。

```
UnitId("warrior_001")  →  "Unit(warrior_001)"
SkillId("fireball")    →  "Skill(fireball)"
BuffId("poison")       →  "Buff(poison)"
ItemId("iron_sword")   →  "Item(iron_sword)"
```

目的：日志和调试输出中立即识别 ID 类型，避免混淆。

---

## ID 命名空间

### 核心 ID 类型 🟥

🟥 所有业务实体必须定义专属强类型标识，禁止直接使用裸 `Entity` 作为业务标识跨模块传递（宪法 1.2.1）。
🟥 必须覆盖核心实体：`UnitId`、`SkillId`、`BuffId`、`ItemId`、`QuestId`（宪法 1.2.1）。

| ID 类型 | 用途 | 存放位置 |
|---------|------|----------|
| `UnitId` | 战场上的战斗单位 | `shared/ids/unit_id.rs` |
| `SkillId` | 技能定义 | `shared/ids/skill_id.rs` |
| `BuffId` | Buff/Debuff 效果 | `shared/ids/buff_id.rs` |
| `ItemId` | 游戏物品 | `shared/ids/item_id.rs` |
| `EquipmentId` | 装备定义 | `shared/ids/equipment_id.rs` |
| `QuestId` | 任务定义 | `shared/ids/quest_id.rs` |
| `StageId` | 关卡/地图 | `shared/ids/stage_id.rs` |
| `FactionId` | 阵营 | `shared/ids/faction_id.rs` |

### 按需扩展的 ID 类型

| ID 类型 | 用途 | 触发条件 |
|---------|------|----------|
| `AiBehaviorId` | AI 行为配置 | 新增 AI 策略模板时 |
| `TerrainId` | 地形类型 | 地形系统独立化时 |
| `DialogueId` | 对话配置 | 对话系统实现时 |
| `ModifierRuleId` | 修饰规则 | 规则引擎扩展时 |
| `TraitId` | Trait 定义 | Trait 配置化时 |

### ID 与 Bevy Entity 的关系

```
Strong ID（UnitId / SkillId / ...）
    │
    │  业务层标识符，全局唯一，可序列化
    │  跨模块传递、配置引用、存档存储
    │
    └─ 不等于 Bevy Entity（Entity 是 ECS 内部标识）

Bevy Entity
    │
    │  ECS 运行时标识符，进程内唯一
    │  仅在 System 执行期间有效
    │
    └─ 不等于 Strong ID
```

### 性能优化：高频 ID 预计算哈希缓存

对于 ECS 中频繁查询的 ID 类型（如 `UnitId`），内部使用 `String` 的哈希/比较性能弱于 `u64`。推荐采用**双存储**策略——对外暴露 `String`，内部缓存预计算的 `u64` 哈希值：

```rust
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnitId {
    inner: String,
    /// 预计算的哈希缓存，避免 HashMap 中重复计算 String 哈希
    cached_hash: OnceLock<u64>,
}

impl UnitId {
    pub fn new(id: impl Into<String>) -> Self {
        let inner = id.into();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&inner, &mut hasher);
        Self {
            inner,
            cached_hash: OnceLock::from(hasher.finish()),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl std::hash::Hash for UnitId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // 使用预计算的缓存哈希，O(1) 替代 O(n)
        self.cached_hash.get_or_init(|| {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&self.inner, &mut h);
            h.finish()
        }).hash(state);
    }
}
```

**适用场景**：`HashMap<UnitId, T>`、`HashSet<UnitId>` 等高频查找场景。

**不适用场景**：低频 ID（如 `StageId`）可保持简单 newtype，无需缓存。

> **优化来源**：`docs/其他/53.md` — 性能优化空间「预计算 u64 哈希缓存 + 双存储策略」

**映射关系**：通过 `UnitEntity` 组件（包含 `UnitId`）建立 Strong ID 与 Entity 的关联。

```rust
#[derive(Component)]
pub struct UnitEntity {
    pub id: UnitId,
}

// 查询方式
fn find_unit_by_id(
    id: &UnitId,
    query: Query<(Entity, &UnitEntity)>,
) -> Option<Entity> {
    query.iter()
        .find(|(_, ue)| &ue.id == id)
        .map(|(e, _)| e)
}
```

---

## ID 分配策略

### 分配者职责 🟥

| 分配者 | 负责 ID | 策略 |
|--------|---------|------|
| 角色生成系统 | `UnitId` | 单调递增 + 语义前缀（如 `warrior_001`） |
| 内容加载器（Content 层） | `SkillId`, `BuffId`, `EquipmentId`, `ItemId` 等 | 从 RON 文件的 `id` 字段读取 |
| MOD 加载器（Modding 层） | MOD 提供的所有 ID | 与基础内容相同，加 MOD 前缀隔离 |
| 关卡加载器 | `StageId` | 从 RON 文件的 `id` 字段读取 |

### 分配推荐策略

**推荐方案：语义化字符串 ID**

```ron
// content/skills/fireball.ron
(
    id: "fireball",          // 人类可读的语义 ID
    name: "火球术",
    // ...
)
```

理由：
- 人类可读：日志、调试、配置文件中一眼识别
- 不依赖全局分配器：每个 RON 文件自包含 ID
- MOD 友好：MOD 作者可预测 ID（`mod_prefix + name`）
- 序列化友好：存档、回放文件中 ID 可读

**备选方案：单调递增 u64**

适用于运行时生成的 ID（如 UnitId），需要全局分配器：

```rust
pub struct IdAllocator {
    counter: AtomicU64,
}

impl IdAllocator {
    pub fn next_unit_id(&self) -> UnitId {
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        UnitId::new(format!("unit_{:06}", n))
    }
}
```

### 分配策略对比

| 方案 | 优点 | 缺点 | 适用场景 |
|------|------|------|----------|
| 语义字符串 | 可读、可预测、MOD 友好 | 需要全局唯一性校验 | 内容 ID（Skill/Buff/Item） |
| 单调递增 | 分配快、天然唯一 | 不可读、需分配器 | 运行时生成 ID（Unit） |
| UUID | 分布式友好 | 不可读、存储开销大 | 网络同步（本项目不推荐） |

---

## ID 在内容文件中的引用

### RON 文件中的 ID 引用

配置文件通过字符串引用其他实体的 ID，由 Content 层在加载时解析为 Strong ID：

```ron
// content/skills/fireball.ron
(
    id: "fireball",
    name: "火球术",

    // 引用 BuffId — 加载时解析
    buff_effects: ["burning", "slow"],

    // 引用 EffectId — 加载时解析
    effect_handlers: ["direct_damage", "aoe_damage"],

    // 引用 TraitId — 加载时解析
    required_traits: ["magic_proficiency"],
)
```

### 引用解析流程

```
RON 文件字符串 ID
    ↓  Content 层加载
XxxDef（字符串引用）
    ↓  impl From<XxxDef> for XxxData
XxxData（Strong ID 引用）
    ↓  引用校验
XxxRegistry（全局注册表）
```

### 引用校验规则

加载时必须校验所有 ID 引用的完整性：

- ✅ 引用的 ID 在对应 Registry 中存在 → 加载成功
- ⚠️ 引用的 ID 不存在 → `warn!` 日志 + 跳过该引用或使用默认值
- 🟥 必填引用的 ID 不存在 → 加载失败，报告错误

---

## ID 生命周期

### 创建 🟥

ID 在以下时机创建：

1. **内容加载时**：从 RON 文件的 `id` 字段读取 → 创建 Strong ID
2. **运行时生成时**：角色生成系统分配 → 创建 Strong ID
3. **MOD 加载时**：MOD 加载器从 MOD 配置读取 → 创建 Strong ID

### 使用 🟥

🟥 跨模块传递 ID 只使用 Strong ID 类型（宪法 1.2.1、1.2.2）。

ID 在以下场景被引用：

- ECS Component 中存储 ID（如 `UnitEntity { id: UnitId }`）
- Message 中携带 ID（如 `DamageApplied { source: UnitId, target: UnitId }`）
- Registry 中以 ID 为键（如 `SkillRegistry: HashMap<SkillId, SkillData>`）
- 配置文件中引用 ID（字符串形式）
- 存档文件中序列化 ID
- 回放文件中序列化 ID

### 不可变 🟥

🟥 ID 创建后不可修改（宪法 1.2.1）。

- 🟥 禁止 `unit_id.0 = "new_value".to_string()`（内部字段不可变）
- 🟥 禁止通过任何方式修改已创建的 ID 值

### 销毁（永不复用） 🟥

🟥 实体被删除时，其 ID 不会被释放或复用（宪法 1.2.1）。

- 🟥 禁止将已删除实体的 ID 分配给新实体
- 🟩 已删除的 ID 可以在日志/审计中继续引用（作为历史记录）
- 🟩 已删除的 ID 可以在存档中保留（标记为已删除状态）

---

## 版本迁移兼容

### 旧格式兼容解析

ID 格式可能随版本迭代（如早期 `Unit(001)` 升级为 `Unit(warrior_001)`），`FromStr` 实现必须提供向后兼容解析：

```rust
impl FromStr for UnitId {
    type Err = IdParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 1. 尝试新格式：Unit(warrior_001)
        if let Some(inner) = s.strip_prefix("Unit(").and_then(|s| s.strip_suffix(')')) {
            return Ok(Self(inner.to_string()));
        }

        // 2. 兼容旧格式：Unit(001) → 自动转换为 Unit(unit_001)
        if let Some(inner) = s.strip_prefix("Unit(").and_then(|s| s.strip_suffix(')')) {
            // 旧格式：纯数字 → 加前缀
            if inner.chars().all(|c| c.is_ascii_digit()) {
                return Ok(Self(format!("unit_{inner}")));
            }
        }

        // 3. 兼容裸字符串（向后兼容）
        if !s.is_empty() {
            return Ok(Self(s.to_string()));
        }

        Err(IdParseError::EmptyInput)
    }
}
```

**迁移规则**：
- 存档加载时自动识别旧格式并转换，无需用户干预
- 转换后的 ID 与新格式完全兼容（`FromStr(Display(id).to_string()) == id`）
- 版本变更时在 `migration.rs` 中添加对应的转换函数

### MOD ID 前缀全局唯一性校验

MOD 提供的 ID 必须通过前缀隔离，避免与基础内容 ID 冲突：

```
基础内容 ID：  skill/fireball, buff/poison
MOD 内容 ID：  mod_<mod_name>/skill/fireball, mod_<mod_name>/buff/poison
```

**校验规则**：
- MOD ID 前缀格式：`mod_<mod_name>/`（小写 + 下划线，不含特殊字符）
- MOD 名称全局唯一性由 MOD 加载器在加载时校验
- 禁止 MOD 使用 `mod_` 前缀以外的命名空间

```rust
/// MOD 加载时校验 ID 前缀
fn validate_mod_id_prefix(mod_id: &str, content_id: &str) -> Result<(), IdParseError> {
    if !content_id.starts_with(&format!("mod_{mod_id}/")) {
        return Err(IdParseError::PrefixMismatch {
            expected: format!("mod_{mod_id}/"),
            actual: content_id.to_string(),
        });
    }
    Ok(())
}
```

> **优化来源**：`docs/其他/53.md` — MOD 隔离「前缀格式定义 + 全局唯一性校验规则」

---

## 测试规范

### 双向一致性测试

所有 Strong ID 类型必须通过以下测试，确保 `FromStr` 和 `Display` 双向一致性：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// FromStr(Display(id).to_string()) == id
    #[test]
    fn bidirectional_consistency() {
        let id = UnitId::new("warrior_001");
        let displayed = id.to_string();  // "Unit(warrior_001)"
        let parsed: UnitId = displayed.parse().unwrap();
        assert_eq!(id, parsed);
    }

    /// 空字符串解析应返回错误
    #[test]
    fn parse_empty_string() {
        assert!(UnitId::from_str("").is_err());
    }

    /// 旧格式兼容解析
    #[test]
    fn legacy_format_compat() {
        let id: UnitId = "Unit(001)".parse().unwrap();
        assert_eq!(id.as_str(), "unit_001");
    }

    /// 裸字符串向后兼容
    #[test]
    fn bare_string_compat() {
        let id: UnitId = "fireball".parse().unwrap();
        assert_eq!(id.as_str(), "fireball");
    }

    /// 序列化/反序列化往返测试（存档兼容性）
    #[test]
    fn serialization_roundtrip() {
        let id = SkillId::new("fireball");
        let json = serde_json::to_string(&id).unwrap();
        let restored: SkillId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, restored);
    }
}
```

**测试覆盖要求**：
- 每个 ID 类型至少包含：双向一致性、空字符串、前缀匹配、序列化往返
- 并发测试：多线程同时创建/解析 ID 不应 panic
- 版本迁移测试：旧格式字符串能正确转换为新格式

> **优化来源**：`docs/其他/53.md` — 测试规范「双向一致性测试 + 边界测试 + 版本兼容测试」

## 目录结构

```
src/shared/ids/
├── mod.rs              # 模块导出
├── unit_id.rs          # UnitId
├── skill_id.rs         # SkillId
├── buff_id.rs          # BuffId
├── item_id.rs          # ItemId
├── equipment_id.rs     # EquipmentId
├── quest_id.rs         # QuestId
├── stage_id.rs         # StageId
├── faction_id.rs       # FactionId
└── error.rs            # IdParseError 等解析错误
```

### mod.rs 导出规范

```rust
// src/shared/ids/mod.rs
mod unit_id;
mod skill_id;
mod buff_id;
mod item_id;
mod equipment_id;
mod quest_id;
mod stage_id;
mod faction_id;
mod error;

pub use unit_id::UnitId;
pub use skill_id::SkillId;
pub use buff_id::BuffId;
pub use item_id::ItemId;
pub use equipment_id::EquipmentId;
pub use quest_id::QuestId;
pub use stage_id::StageId;
pub use faction_id::FactionId;
pub use error::IdParseError;
```

---

## 允许的模式

### 模式 1：ID 作为函数参数

```rust
// ✅ 允许：类型安全的函数签名
fn calculate_damage(attacker: &UnitId, target: &UnitId) -> DamageResult { ... }

// ❌ 禁止：裸类型签名
fn calculate_damage(attacker: &str, target: &str) -> DamageResult { ... }
```

### 模式 2：ID 在 Message 中携带

```rust
// ✅ 允许：Message 携带 Strong ID
#[derive(Message, Debug, Clone)]
pub struct DamageApplied {
    pub source: UnitId,      // Strong ID
    pub target: UnitId,      // Strong ID
    pub skill_id: Option<SkillId>,  // 可选 Strong ID
    pub amount: i32,
}

// ❌ 禁止：Message 使用裸 Entity 或 String
```

### 模式 3：ID 在 Registry 中作为键

```rust
// ✅ 允许：HashMap 以 Strong ID 为键
pub struct SkillRegistry {
    skills: HashMap<SkillId, SkillData>,
}

// ❌ 禁止：HashMap 以 String 为键
```

### 模式 4：ID 的 From 转换

```rust
// ✅ 允许：从字符串创建 ID
let id = SkillId::from("fireball");
let id: SkillId = "fireball".parse().unwrap();
let id = SkillId::new("fireball".to_string());
```

### 模式 5：ID 比较

```rust
// ✅ 允许：同类型比较
assert_eq!(unit_a_id, unit_b_id);

// ❌ 禁止：不同类型比较（编译错误）
// assert_eq!(unit_id, skill_id);  // UnitId ≠ SkillId
```

---

## 禁止事项

### 🟥 绝对禁止

| 禁止行为 | 原因 | 违反后果 |
|----------|------|----------|
| 使用裸 `String` 作为业务标识跨模块传递 | 编译期无法区分 ID 类型 | 运行时 ID 混淆导致数据错乱 |
| 使用裸 `Entity` 作为业务标识跨模块传递 | Entity 不可序列化，进程重启后失效 | 存档/回放系统无法使用 |
| 不同 ID 类型之间隐式转换 | 破坏类型安全 | `UnitId` 误当 `SkillId` 使用 |
| 在 ID 中存储业务逻辑 | ID 是纯标识符 | ID 职责膨胀，违反单一职责 |
| 复用已删除实体的 ID | 历史引用失效 | 存档/回放中的旧引用指向错误实体 |
| 在 Shared 层之外定义 Strong ID | ID 是共享层基础设施 | 依赖方向违反 |
| 在 `shared/` 之外重复定义同一 ID 类型 | 多个定义导致混乱 | 类型不兼容、强制类型转换 |

### 🟩 必须遵守

| 必须行为 | 原因 |
|----------|------|
| 每个 ID 实现所有必须的 trait | 保证 ID 可用于 HashMap、日志、序列化等场景 |
| Display 格式包含类型前缀 | 日志和调试中一眼识别 ID 类型 |
| ID 在 `shared/ids/` 目录定义 | 统一管理，避免分散 |
| 新增 ID 类型先在 `shared/ids/mod.rs` 导出 | 保证全局可用 |
| 跨模块传递 ID 只使用 Strong ID 类型 | 编译期类型安全 |

---

## AI 修改规则

### 如果新增 Strong ID 类型

允许：
- 在 `shared/ids/` 中创建新的 newtype 文件
- 为新类型实现所有必须的 trait
- 在 `shared/ids/mod.rs` 中添加 `pub use` 导出

禁止：
- 使用裸 String 或 Entity 替代 newtype
- 省略 Display、Hash、Eq 等 trait 实现
- 在 newtype 中添加业务逻辑方法

优先检查：
- 新 ID 类型是否真的需要在 Shared（而不是在业务模块内部）
- Display 格式是否包含类型前缀
- 是否与现有 ID 类型完全隔离

### 如果修改现有 ID 类型

允许：
- 新增方法（不破坏现有 API）
- 改进 Display 格式（需同步更新所有使用方）

禁止：
- 删除已有字段或方法
- 修改内部存储类型（如 String → u64）
- 修改 Display 格式的类型前缀

优先检查：
- 修改是否影响序列化/反序列化兼容性
- 所有使用该 ID 类型的模块是否同步更新
- 测试是否需要更新
