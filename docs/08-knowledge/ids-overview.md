---
id: 08-knowledge.ids-overview
title: 强类型 ID 系统深度解析 — 类型安全的标识符策略
status: stable
owner: data-architect
created: 2026-06-20
updated: 2026-06-28
tags:
  - knowledge
  - ids
  - type-safety
  - macro
  - domain-driven
  - adr-000
  - adr-045
  - id-taxonomy
---

# 强类型 ID 系统深度解析

> 目标读者：新加入项目的开发者，或其他想理解 ID 系统全貌的人。
> 读完本文，你会知道 ID 是怎么定义的、为什么这么设计、代码放在哪、怎么用。

---

## 目录

1. [核心思想：为什么 ID 要这么设计？](#1-核心思想为什么-id-要这么设计)
2. [整体架构全景图](#2-整体架构全景图)
3. [两种 ID 类型：String ID vs Numeric ID](#3-两种-id-类型string-id-vs-numeric-id)
4. [宏详解：define_string_id! 与 define_numeric_id!](#4-宏详解define_string_id-与-define_numeric_id)
5. [StrongId trait — 统一接口](#5-strongid-trait--统一接口)
6. [已定义的 ID 类型清单](#6-已定义的-id-类型清单)
7. [数据流全景：一条 ID 的旅程](#7-数据流全景一条-id-的旅程)
8. [实战：如何添加新的 ID 类型](#8-实战如何添加新的-id-类型)
9. [与 Registry 系统的集成](#9-与-registry-系统的集成)
10. [规则速查：该做什么和不该做什么](#10-规则速查该做什么和不该做什么)

---

## 1. 核心思想：为什么 ID 要这么设计？

### 1.1 最大的原则：类型安全 + 无语义 ID

传统游戏项目的 ID 长这样：

```rust
// ❌ 传统做法：用 String 或 u64 作为 ID
fn get_ability(id: &str) -> AbilityDef { ... }
fn get_effect(id: u64) -> EffectDef { ... }

// 问题：id 和 id 可以混用，编译器无法区分
let ability_id = "abl_000042";
let effect_id = "abl_000042";  // 拼错了，但类型相同，编译通过
```

看起来没问题，但在 Fre 这种 50 万行级别的项目里会出现三个问题：

1. **类型混淆** — `AttributeId` 和 `TagId` 内部值相同，但语义完全不同，混用会导致难以追踪的 bug
2. **格式不统一** — 每个人写 ID 格式不一样，有的带前缀，有的不带，搜索困难
3. **序列化脆弱** — 手动实现 Serialize/Deserialize 容易出错，格式不一致

所以 Fre 的做法是反过来：**用宏生成强类型 ID，编译期防止混用。**

```rust
// ✅ Fre 的做法：强类型 ID
let attr_id = AttributeId::new("hp_max");
let tag_id = TagId::new("hp_max");

// 编译期类型不同，不能混用
// assert_eq!(attr_id, tag_id);  // 编译错误！

// Display 格式统一
assert_eq!(format!("{}", attr_id), "attr:hp_max");
assert_eq!(format!("{}", tag_id), "tag:hp_max");
```

这种设计叫做**强类型 ID 模式**，记录在 `docs/04-data/foundation/id_strategy.md` 中。

### 1.2 ID 的两个维度

Fre 的 ID 系统覆盖两个维度：

```
维度 1：定义标识（Definition ID）
  │  用于配置文件中的静态内容标识
  │  格式：prefix:value（如 "attr:hp_max"）
  │  类型：String ID（define_string_id! 宏生成）
  │
  └── 示例：AbilityId, EffectId, TagId, ItemId...

维度 2：实例标识（Instance ID）
  │  用于运行时动态分配的实例唯一标识
  │  格式：TypeName(value)（如 "ModifierInstanceId(42)"）
  │  类型：Numeric ID（define_numeric_id! 宏生成）
  │
  └── 示例：ModifierInstanceId
```

### 1.3 设计原则

| 原则 | 理由 |
|------|------|
| **无语义 ID** | `abl_000042` 而非 `ability.fireball`。语义变化需要改 ID，破坏所有引用 |
| **类型隔离** | 不同前缀的 ID 是不同类型，编译期防止混用 |
| **格式统一** | Display 输出 `<prefix>:<value>`，便于调试和日志 |
| **永不重用** | ID 一旦分配永久有效，删除时标记 deprecated，不重新分配 |
| **Serde 兼容** | 同时接受 `prefix:value` 和裸 `value` 格式 |

---

## 2. 整体架构全景图

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              src/shared/ids/                                     │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  foundation/                      核心抽象层（零依赖，仅 Rust std）               │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │  strong_id.rs    pub trait StrongId: Display + FromStr + Deref + Sized  │    │
│  │                  { fn prefix(), fn as_str() }                           │    │
│  │                                                                         │    │
│  │  macros.rs       define_string_id! 宏（String ID 生成器）               │    │
│  │                  define_numeric_id! 宏（Numeric ID 生成器）             │    │
│  │                                                                         │    │
│  │  errors.rs       IdFormatError / IdAllocationError（thiserror）         │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                 │
│  types/                           具体 ID 类型定义                              │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │  string_ids.rs     24 个 String ID（attr/tag/mod/eff/abl/...）          │    │
│  │  numeric_ids.rs    1+ 个 Numeric ID（ModifierInstanceId）               │    │
│  │  definition_id.rs  DefinitionId（无前缀通用 ID）                        │    │
│  │  runtime_id.rs     RuntimeId + RuntimeIdAllocator（generation 保护）    │    │
│  │  battle_unit_id.rs BattleUnitId（回放系统用）                           │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                 │
│  mapping/                         Entity ↔ ID 运行时映射                       │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │  entity_mapper.rs  EntityMapper<ID> 泛型双向映射器                      │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                 │
│  prelude.rs                       便捷导入                                     │
│  mod.rs                           模块根（只做 re-export）                     │
│                                                                                 │
│  tests/                           测试                                         │
│  ├── unit/                        单元测试（string/numeric/runtime/mapper）    │
│  ├── invariant/                   不变量测试（13 条 Identity Invariant）      │
│  └── fixtures/                    共享测试夹具                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                     使用方（Core / Infra / UI）                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  领域 Definition                                                        │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │  struct AbilityDef {                                            │    │
│  │      id: AbilityId,           // 强类型 ID，编译期类型安全      │    │
│  │      name_key: String,        // 本地化 key                    │    │
│  │      effect_ids: Vec<EffectId>, // 引用其他 ID                 │    │
│  │  }                                                               │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                         │
│  Registry 系统                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │  // 通过 StrongId trait 约束泛型                                │    │
│  │  trait Registry<T: StrongId> {                                  │    │
│  │      fn get(&self, id: &T) -> Option<&Def>;                    │    │
│  │  }                                                               │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                         │
│  配置文件（RON/JSON）                                                    │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │  AbilityDef (                                                   │    │
│  │      id: "abl_000042",        // 带前缀格式                    │    │
│  │      effects: ["eff_000001"], // 引用其他 ID                   │    │
│  │  )                                                               │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 3. 两种 ID 类型：String ID vs Numeric ID

### 3.1 String ID — 领域 Definition 标识

用于配置文件中的静态内容标识，格式：`<prefix>:<value>`

```rust
define_string_id! {
    pub AttributeId,
    prefix: "attr",
}

// 使用示例
let id = AttributeId::new("hp_max");
assert_eq!(id.as_str(), "hp_max");
assert_eq!(format!("{}", id), "attr:hp_max");
```

**特点**：
- 内部存储 `String`
- Display 输出带前缀格式
- 支持 Serde 序列化/反序列化
- 实现 `StrongId` trait

### 3.2 Numeric ID — 运行时实例标识

用于运行时动态分配的实例唯一标识，格式：`TypeName(value)`

```rust
define_numeric_id!(ModifierInstanceId);

// 使用示例
let id = ModifierInstanceId::new(42);
assert_eq!(id.value(), 42);
assert_eq!(format!("{}", id), "ModifierInstanceId(42)");
```

**特点**：
- 内部存储 `u64`
- 支持 `Copy` 语义
- Display 输出 `TypeName(value)` 格式
- 序列化为纯数字

### 3.3 对比表

| 维度 | String ID | Numeric ID |
|------|-----------|------------|
| **内部类型** | `String` | `u64` |
| **Display 格式** | `prefix:value` | `TypeName(value)` |
| **序列化格式** | `"prefix:value"` | `42` |
| **用途** | Definition 标识（静态） | Instance 标识（动态） |
| **Copy** | ❌ | ✅ |
| **StrongId** | ✅ | ❌ |
| **示例** | `AttributeId("hp_max")` | `ModifierInstanceId(42)` |

---

## 4. 宏详解：define_string_id! 与 define_numeric_id!

### 4.1 define_string_id! 宏

```rust
#[macro_export]
macro_rules! define_string_id {
    (
        $vis:vis $name:ident,
        prefix: $prefix:expr,
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
        #[reflect(Hash, PartialEq)]
        $vis struct $name(pub String);

        impl $name {
            $vis fn new(id: impl Into<String>) -> Self { ... }
            $vis fn as_str(&self) -> &str { ... }
            $vis fn is_empty(&self) -> bool { ... }
            $vis fn len(&self) -> usize { ... }
            $vis fn into_inner(self) -> String { ... }
        }

        impl std::fmt::Display for $name { ... }      // "prefix:value"
        impl std::str::FromStr for $name { ... }      // 解析 "prefix:value" 或裸 value
        impl std::ops::Deref for $name { ... }        // Deref<Target = str>
        impl From<&str> for $name { ... }
        impl From<String> for $name { ... }
        impl serde::Serialize for $name { ... }
        impl<'de> serde::Deserialize<'de> for $name { ... }
        impl StrongId for $name { ... }
    };
}
```

**关键行为**：
- **Display**: 输出 `"prefix:value"` 格式
- **FromStr**: 接受 `"prefix:value"` 或裸 `"value"`，拒绝错误前缀
- **Serialize**: 序列化为 `"prefix:value"` 字符串
- **Deserialize**: 反序列化时同时接受两种格式

### 4.2 define_numeric_id! 宏

```rust
#[macro_export]
macro_rules! define_numeric_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Reflect)]
        pub struct $name(pub u64);

        impl $name {
            pub fn new(id: u64) -> Self { ... }
            pub fn value(&self) -> u64 { ... }
        }

        impl std::fmt::Display for $name { ... }      // "TypeName(value)"
        impl From<u64> for $name { ... }
        impl std::ops::Deref for $name { ... }        // Deref<Target = u64>
        impl serde::Serialize for $name { ... }       // 序列化为纯数字
        impl<'de> serde::Deserialize<'de> for $name { ... }
    };
}
```

**关键行为**：
- **Display**: 输出 `"TypeName(value)"` 格式
- **Serialize**: 序列化为纯数字 `42`
- **Copy**: 支持复制语义，不消耗所有权

---

## 5. StrongId trait — 统一接口

```rust
/// 所有强类型 ID 必须实现的 trait。
///
/// 提供统一的接口以支持 Registry 约束和跨模块泛型操作。
pub trait StrongId:
    std::fmt::Display + std::str::FromStr + std::ops::Deref<Target = str> + Sized
{
    fn prefix() -> &'static str;  // 返回类型前缀（如 "attr"）
    fn as_str(&self) -> &str;     // 返回内部值（如 "hp_max"）
}
```

**用途**：
- Registry 泛型约束：`Registry<T: StrongId>`
- 跨模块 ID 操作
- 运行时前缀检查

**示例**：
```rust
assert_eq!(AttributeId::prefix(), "attr");
assert_eq!(TagId::prefix(), "tag");

let id = AttributeId::new("hp");
assert_eq!(StrongId::as_str(&id), "hp");
```

---

## 6. 已定义的 ID 类型清单

### 6.1 String ID 类型（22 个）

| 类型 | 前缀 | 用途 | 引用方 |
|------|------|------|--------|
| `AttributeId` | `attr` | 属性定义 | Modifier/Aggregator/Progression |
| `TagId` | `tag` | 标签定义 | Condition/Trigger/Modifier |
| `ModifierId` | `mod` | 修改器定义 | Effect/Equipment/Bond |
| `EffectId` | `eff` | 效果定义 | Ability/Trigger/Terrain |
| `AbilityId` | `abl` | 能力定义 | Spec/Spell |
| `TriggerId` | `trg` | 触发器定义 | Effect/Condition |
| `CueId` | `cue` | Cue 定义 | UI/叙事 |
| `CharacterId` | `char` | 角色定义 | Party/Inventory |
| `UnitId` | `unit` | 单位定义 | Combat/Tactical |
| `EquipmentId` | `equip` | 装备定义 | Inventory/Crafting |
| `ItemId` | `itm` | 物品定义 | Inventory/Crafting/Economy/Quest |
| `FactionId` | `fct` | 阵营定义 | Shop/Economy |
| `QuestId` | `qst` | 任务定义 | Narrative/CampEvent |
| `SpellId` | `spl` | 法术定义 | Spell/Ability |
| `BuffId` | `buf` | Buff 定义 | Effect/Condition |
| `TerrainId` | `ter` | 地形定义 | Tactical/Terrain |
| `RecipeId` | `rcp` | 配方定义 | Crafting |
| `LootTableId` | `ltb` | 战利品表定义 | Loot/Quest |
| `TeamId` | `team` | 队伍定义 | Combat/Party |
| `ClassId` | `cls` | 职业定义 | Progression/Party |
| `TalentId` | `tal` | 天赋定义 | Progression |
| `SubclassId` | `sub` | 子职业定义 | Progression |
| `BondDefId` | `bnd` | 羁绊定义 | Party |
| `FormationDefId` | `fmd` | 阵型定义 | Combat/Tactical |
| `CampEventId` | `cmp` | 营地事件定义 | CampRest |

### 6.2 Numeric ID 类型（1 个）

| 类型 | 用途 |
|------|------|
| `ModifierInstanceId` | 运行时修改器实例标识 |

### 6.3 特殊类型（1 个）

| 类型 | 说明 |
|------|------|
| `DefinitionId` | 无前缀通用 ID，用于 Registry 泛型查询 |

---

## 7. 数据流全景：一条 ID 的旅程

以「创建一个 AbilityDef」为例：

```
Step 1: 内容团队在配置文件中定义
─────────────────────────────────
  assets/definitions/ability.ron:
    AbilityDef (
        id: "abl_000042",              // ← 字符串格式
        name_key: "ability.abl_000042.name",
        effects: ["eff_000001"],       // ← 引用其他 ID
    )

Step 2: Registry 加载并解析
─────────────────────────────────
  registry_loader.rs:
    let def: AbilityDef = ron::from_str(&content)?;
    // id 字段自动解析为 AbilityId("abl_000042")

Step 3: 强类型 ID 保证类型安全
─────────────────────────────────
  ability_system.rs:
    fn get_ability(id: &AbilityId) -> Option<&AbilityDef> { ... }

    // ✅ 正确：类型匹配
    let id = AbilityId::new("abl_000042");
    let def = get_ability(&id);

    // ❌ 错误：类型不匹配，编译失败
    let wrong_id = EffectId::new("abl_000042");
    // get_ability(&wrong_id);  // 编译错误！

Step 4: Display 格式化输出
─────────────────────────────────
  // 日志/调试输出
  info!("Loaded ability: {}", ability_id);
  // 输出: "Loaded ability: abl:abl_000042"

Step 5: 序列化存储
─────────────────────────────────
  // 保存到存档
  let json = serde_json::to_string(&ability_id)?;
  // 输出: "\"abl:abl_000042\""

  // 反序列化恢复
  let restored: AbilityId = serde_json::from_str(&json)?;
  // 同时接受 "abl:abl_000042" 和 "abl_000042"
```

---

## 8. 实战：如何添加新的 ID 类型

假设你要为「商店系统」添加新的 ID 类型。

### 第一步：确定 ID 类型

- 如果是**静态 Definition 标识** → 使用 `define_string_id!`
- 如果是**运行时实例标识** → 使用 `define_numeric_id!`

### 第二步：选择前缀

查看 `docs/04-data/foundation/id_strategy.md` 中的前缀表，选择未使用的前缀。

```rust
// 前缀选择规则：
// 1. 3 个小写字母
// 2. 与领域名称相关
// 3. 不与其他前缀冲突

// 商店系统：前缀 "shp"
```

### 第三步：在 types.rs 中添加

```rust
// src/shared/ids/types.rs

// ============================================================================
// 补充领域 ID 类型（按 id_strategy.md table 新增）
// ============================================================================

define_string_id! {
    pub ShopId,
    prefix: "shp",
}
```

### 第四步：在 mod.rs 中确认导出

```rust
// src/shared/ids/mod.rs
pub use types::*;  // 已包含所有 ID 类型
```

### 第五步：编写测试

```rust
// src/shared/ids/tests/unit/string_id_test.rs

#[test]
fn shop_id_display_uses_shp_prefix() {
    let id = ShopId::new("general_store");
    assert_eq!(format!("{}", id), "shp:general_store");
}

#[test]
fn shop_id_from_str_parses_prefix_colon_value() {
    let id: Result<ShopId, _> = "shp:general_store".parse();
    assert!(id.is_ok());
    assert_eq!(id.unwrap().as_str(), "general_store");
}
```

### 检查清单

- [ ] 前缀在 `id_strategy.md` 中已定义
- [ ] 在 `types.rs` 中使用 `define_string_id!` 或 `define_numeric_id!`
- [ ] 测试覆盖 Display/FromStr/Serialize/Deserialize
- [ ] 更新 `id_strategy.md` 中的前缀表（如需要）

---

## 9. 与 Registry 系统的集成

### 9.1 StrongId 约束

Registry 系统使用 `StrongId` trait 约束泛型：

```rust
trait Registry<T: StrongId> {
    fn get(&self, id: &T) -> Option<&Def>;
    fn contains(&self, id: &T) -> bool;
}

// 使用示例
let ability_registry: Registry<AbilityId> = ...;
let effect_registry: Registry<EffectId> = ...;
```

### 9.2 引用验证

Registry 在加载时执行全量引用检查：

```
配置加载
    │
    ├── 1. Registry 扫描所有配置文件
    ├── 2. 提取所有 ID 字段
    ├── 3. 检查 ID 格式合法性（前缀 + 值）
    ├── 4. 检查 ID 是否重复
    ├── 5. 执行全量引用检查（dangling reference 检测）
    └── 6. 通过 → 加载到内存；失败 → 报告错误列表
```

### 9.3 序列化兼容

ID 在序列化（RON/JSON/存档）中的格式统一：

```ron
// RON 配置
AbilityDef (
    id: "abl_000042",
    effects: ["eff_000001", "eff_000015"],
)
```

```json
// JSON 存档
{
    "id": "abl:abl_000042",
    "effects": ["eff:eff_000001"]
}
```

---

## 10. Identity Invariant — ID 系统的 13 条铁律

> 详细定义见 `docs/04-data/foundation/id-taxonomy.md` §10

| # | 规则 | 一句话 | 反面教材 |
|---|------|--------|---------|
| 1 | ID 不参与业务逻辑 | ID 只负责 Identity，不承担分类/权限/行为/状态 | `if unit.id() == UnitId(1)` |
| 2 | ID 不隐含排序 | 排序用专用字段，不用 ID | `units.sort_by_key(\|u\| u.id())` |
| 3 | 不暴露 ID 生成方式 | 生成集中在 Allocator，业务代码不直接创建 | `UnitId(counter.fetch_add(1))` |
| 4 | ID 创建必须可审计 | Debug 模式记录创建来源、帧号、触发者 | 幽灵对象找不到来源 |
| 5 | 区分引用和拥有 | ID = 引用，`Vec<T>` = 拥有 | 循环引用、生命周期混乱 |
| 6 | Null ID 是反模式 | 用 `Option<T>` 或枚举，不用 Magic Number | `UnitId(0)` 表示无目标 |
| 7 | 跨层禁止隐式转换 | ID 转换经过显式服务 | `impl From<A> for B` |
| 8 | ID 不承担显示职责 | 显示用 `display_name()`，不用 `format!("{}", id)` | 日志全是 `Unit#18472` |
| 9 | Identity 是横切关注点 | 所有 ID 集中在 `shared/ids/` | 各领域自己定义 ID |
| 10 | 配置表引用编译为 Typed ID | 运行时零 String 查找 | 运行时 HashMap 查找 |
| 11 | Runtime ID 约束 | 唯一、不可变、可复制、可序列化、Generation 保护 | ID 复用导致引用悬空 |
| 12 | Template ID 约束 | 跨版本稳定、跨 DLC 稳定、跨 Mod 稳定、无语义 | DLC 合并 ID 冲突 |
| 13 | Entity 约束 | 不出 Infrastructure、通过映射访问、不可序列化 | Domain 层裸 Entity |

---

## 11. 规则速查：该做什么和不该做什么

### ✅ 允许的

| 场景 | 做法 |
|------|------|
| 定义新的 Definition ID | 使用 `define_string_id!` 宏 |
| 定义运行时实例 ID | 使用 `define_numeric_id!` 宏 |
| 获取 ID 前缀 | 调用 `T::prefix()` |
| 获取 ID 内部值 | 调用 `id.as_str()` 或 `id.value()` |
| 序列化/反序列化 | 使用 serde 自动实现 |

### ❌ 禁止的

| 场景 | 为什么禁止 |
|------|-----------|
| 使用 String/u64 作为 ID | 破坏类型安全，编译期无法检测混用 |
| 手动实现 Serialize/Deserialize | 格式不一致，容易出错 |
| 在 ID 中编码语义 | 语义变化需要改 ID，破坏所有引用 |
| 重用已废弃的 ID | 破坏存档/Replay 兼容性 |
| 混用不同前缀的 ID | 编译期类型不同，运行时值相同但语义不同 |
| `id() == SomeId(N)` 判断业务逻辑 | 后期 ID 重分配时逻辑崩溃 |
| `sort_by_key(\|x\| x.id())` 排序 | 存档恢复/网络同步后顺序变化 |
| 业务代码直接创建 ID | 破坏封装边界，导致 ID 冲突 |
| `UnitId(0)` 表示无目标 | Magic Number 传播，后期难查 |
| `impl From<A> for B` 隐式转换 | 数据流不可追踪 |
| `format!("{}", id)` 给玩家看 | 调试体验差，日志无意义 |

---

## 现状盘点：已经做了什么，还缺什么

### 已实现 — 代码层

| 组件 | 状态 | 说明 |
|------|------|------|
| StrongId trait | ✅ 完整 | 统一接口：prefix() + as_str() |
| define_string_id! 宏 | ✅ 完整 | 生成 String ID 类型（22 个） |
| define_numeric_id! 宏 | ✅ 完整 | 生成 Numeric ID 类型（1 个） |
| RuntimeId (index + generation) | ✅ 完整 | 带 Generation 保护的运行时 ID，11 个测试 |
| RuntimeIdAllocator | ✅ 完整 | 分配/回收/复用/空闲列表管理 |
| DefinitionId 通用 ID | ✅ 完整 | 无前缀通用 ID，用于 Registry 泛型查询 |
| String ID 类型 | ✅ 完整 | 22 个领域 ID 类型（attr/tag/eff/abl 等） |
| Numeric ID 类型 | ✅ 完整 | 1 个实例 ID 类型（ModifierInstanceId） |
| 单元测试 | ✅ 完整 | 345 行测试覆盖全部 API（含 RuntimeId 11 个测试） |
| Serde 支持 | ✅ 完整 | 序列化/反序列化兼容两种格式 |
| Reflect 支持 | ✅ 完整 | Bevy 反射系统集成 |
| EntityMapper (双向映射) | ✅ 完整 | Entity ↔ 业务 ID 双向映射，8 个测试 |

> **2026-06-20 更新**: EntityMapper 和 RuntimeId 均已实现并投入生产。
> 代码结构重构（扁平 → foundation/types/mapping 三层）计划见 `docs/11-refactor/id-system-refactoring-2026-06-20.md`。

### 已实现 — 文档/治理层

| 组件 | 状态 | 说明 |
|------|------|------|
| ID 分类体系文档 | ✅ 完整 | `docs/04-data/foundation/id-taxonomy.md`（607 行） |
| Identity Invariant 13 条铁律 | ✅ 完整 | 定义在 id-taxonomy.md §10 |
| 五类 ID 分类（Template/Runtime/Save/Entity/Network） | ✅ 完整 | id-taxonomy.md §2-§7 |
| 验证清单（18 项） | ✅ 完整 | id-taxonomy.md §11 |
| 宪法 Domain 层 Entity 限制 | ✅ 落地 | ai-constitution-complete.md §6.1 + §19.1 |
| 编码规则 Domain 层 Entity 限制 | ✅ 落地 | coding-rules.md §3 |
| ECS 规则 Domain 层 Entity 限制 | ✅ 落地 | .trae/rules/ECS规则.md §1.1 + 速查第 2 条 |
| 编码规则 .trae Domain 层 Entity 限制 | ✅ 落地 | .trae/rules/编码规则.md |
| AI 架构准则 Domain 层 Entity 限制 | ✅ 落地 | .trae/rules/AI架构准则.md |
| SRPG 专项规则 ID 章节 | ✅ 落地 | .trae/rules/SRPG专项规则.md §十 |
| 审查规则 ID 审查清单 | ✅ 落地 | .trae/rules/审查规则.md（6 项） |
| 测试规范 ID 测试不变量 | ✅ 落地 | .trae/rules/测试规范.md（5 项） |
| ADR-042 引用 id-taxonomy | ✅ 落地 | save-persistence.md |
| ADR-013 引用 id-taxonomy | ✅ 落地 | registry-hotreload.md |
| UI 架构引用 id-taxonomy | ✅ 落地 | theme-localization.md |
| 知识文档 Identity Invariant 速查 | ✅ 落地 | ids-overview.md §10（13 条一览表） |

### 待实现或待重构 — 代码层

| 组件 | 优先级 | 说明 |
|------|--------|------|
| 代码目录结构重构 | 🔴 高 | `shared/ids/` 扁平结构 → foundation/types/mapping 三层（见重构文档） |
| ModifierInstanceId → RuntimeId | 🔴 高 | 裸 u64 需升级为带 generation 保护的 RuntimeId |
| BattleUnitRegistry → EntityMapper | 🔴 高 | 重复实现需统一为泛型 EntityMapper\<ID\>（详见 ADR-048） |
| EntityRemapper Vec→HashMap | 🟡 中 | 存档量大的时候 O(n) 不可接受，需改为 HashMap O(1) |
| define_string_id! 格式校验 | 🟡 中 | 需添加 `checked_new()` 方法校验 ID 格式 |
| Numeric ID 实现 StrongId trait | 🟡 中 | 当前 define_numeric_id! 不实现 StrongId，类型体系不统一 |
| SaveObjectId (Uuid) | 🟡 中 | 存档兼容性，当前 PersistentEntityId(u64) 可用但非最佳 |
| Mod 命名空间 | 🟡 中 | Template ID 前缀需升级为 `namespace:type.name` |
| IdRegistry 统一管理 | 🟡 中 | 生成/回收/映射/校验集中管理 |
| ID 创建审计（Debug 模式） | 🟡 中 | IdCreationInfo 记录创建来源/帧号/触发者 |
| DefinitionId 实现 StrongId | 🟡 中 | 当前手写定义未统一实现 StrongId trait |
| Network ID | 🟢 低 | 联机功能预留 |
| Identity Invariant 代码级检查 | 🟢 低 | Clippy Lint 或 CI 脚本自动检测违规 |

### 设计决策说明

| 组件 | 决策 | 说明 |
|------|------|------|
| ID 前缀格式 | `<type>:<value>` | 如 `attr:hp_max`、`abl:abl_000042`，待升级为命名空间格式 |
| ID 生成方式 | 宏生成 | 确保所有 ID 类型行为一致 |
| ID 永不重用 | Registry 管理 | 删除时标记 deprecated，不重新分配 |
| 6 位编号 | 每领域 100 万空间 | 覆盖 DLC、Mod、长期运营 |
| Identity 定位 | Cross-cutting Concern | 类似 `shared/error/`、`shared/events/`，独立子域 |

### 大型项目硬规则

> **所有领域层（Domain/Application）禁止出现 `Entity`、`u64`、`usize` 作为业务对象标识；只能出现显式命名的强类型 ID。**

| 层 | Entity 使用 | 业务 ID |
|----|------------|---------|
| Infrastructure | ✅ 自由使用 | ✅ 维护映射 |
| Domain | 🟥 禁止裸 Entity | ✅ 只用强类型 ID |
| Application | 🟡 通过 ACL 间接使用 | ✅ 只用强类型 ID |

**理由**：Entity 在存档加载/场景重载后失效；u64 无法编译期防混用；裸类型无法跨 Mod/网络确定性同步。

### Governance 覆盖矩阵

| 治理层级 | 文件 | 引用 id-taxonomy |
|---------|------|-----------------|
| 宪法 | `ai-constitution-complete.md` §6.1 + §19.1 | ✅ |
| 编码规则 | `coding-rules.md` §3 | ✅ |
| ECS 规则 | `.trae/rules/ECS规则.md` §1.1 + 速查 | ✅ |
| 编码规则 .trae | `.trae/rules/编码规则.md` | ✅ |
| AI 架构准则 | `.trae/rules/AI架构准则.md` | ✅ |
| SRPG 专项规则 | `.trae/rules/SRPG专项规则.md` §十 | ✅ |
| 审查规则 | `.trae/rules/审查规则.md` | ✅ |
| 测试规范 | `.trae/rules/测试规范.md` | ✅ |
| 架构总纲 | `01-architecture/README.md` | ✅ |
| ADR-042 | `ADR-042-save-persistence.md` | ✅ |
| ADR-013 | `ADR-013-registry-hotreload.md` | ✅ |
| 数据架构总纲 | `04-data/README.md` | ✅ |
| UI 架构 | `06-ui/02-design-system/theme-localization.md` | ✅ |

### 与旧版对比

| 维度 | 旧版（ai_ignore_this_dir） | 新版（shared/ids） |
|------|---------------------------|-------------------|
| 文件位置 | `src/core/id/` | `src/shared/ids/` |
| ID 定义 | 每个 ID 一个文件 | 宏统一生成 |
| 前缀格式 | 不统一 | 统一 `<prefix>:<value>` |
| 测试覆盖 | 无 | 334 行测试 |
| Entity 隔离 | 无规则 | 宪法+编码规则+领域规则三重约束 |
| ID 分类 | 无 | 五类分离（Template/Runtime/Save/Entity/Network） |
| Identity Invariant | 无 | 13 条铁律 + 18 项验证清单 |
| Governance 覆盖 | 无 | 13 个文件引用 id-taxonomy |

---

## 参考文档

### 架构决策

| 文档 | 内容 |
|------|------|
| `docs/04-data/foundation/id_strategy.md` | ID 策略详述（分配机制、生命周期、校验规则） |
| `docs/04-data/foundation/id-taxonomy.md` | ID 分类体系（五类 ID：Template/Runtime/Save/Entity/Network） |
| `docs/04-data/README.md` §3 | ID 策略与命名规范总纲 |
| `docs/01-architecture/00-foundation/ADR-000-feature-module-map.md` | 模块地图（IDs 位置定义） |
| `docs/01-architecture/00-foundation/ADR-045-module-visibility-strategy.md` | 模块可见性（IDs 为 `pub(crate)`） |
| `docs/01-architecture/README.md` §IDs | 架构总纲中的 IDs 模块说明 |

### 宪法与编码规则

| 文档 | 内容 |
|------|------|
| `docs/00-governance/ai-constitution-complete.md` §724 | Entity 只是 ID |
| `docs/00-governance/ai-constitution-complete.md` §1425 | 强类型 ID 放在 `shared/ids/` |
| `docs/00-governance/coding-rules.md` §129 | Entity 仅用于引用实体 |
| `docs/02-domain/capabilities/ui-presentation.md` INV-UI-002 | Widget 组件禁止 Entity 字段 |

### 代码实现

| 文件 | 内容 |
|------|------|
| `src/shared/ids/` | 模块根目录（详见架构全景图 §2） |
| `src/shared/ids/mod.rs` | 模块根（只做 re-export） |
| `src/shared/ids/foundation/` | StrongId trait、宏定义、错误类型 |
| `src/shared/ids/types/` | 具体 ID 类型（string_ids / numeric_ids / runtime_id / definition_id） |
| `src/shared/ids/mapping/` | EntityMapper 通用双向映射器 |
| `src/shared/ids/tests/unit/string_id_test.rs` | String ID 单元测试（235 行） |
| `src/shared/ids/tests/unit/numeric_id_test.rs` | Numeric ID 单元测试（99 行） |
| `src/shared/ids/tests/unit/runtime_id_test.rs` | RuntimeId 测试（134 行） |
| `src/shared/ids/tests/unit/entity_mapper_test.rs` | EntityMapper 测试（104 行） |

### 重构计划

| 文档 | 内容 |
|------|------|
| `docs/11-refactor/id-system-refactoring-2026-06-20.md` | ID 系统激进重构计划（9 阶段，P0-P2 优先级） |
