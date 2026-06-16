---
id: foundation.id-strategy.v1
title: ID Strategy Deep Dive — 标识符策略详述
status: draft
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition
replay-safe: true
---

# ID Strategy — 标识符策略详述

> **总纲引用**: `docs/04-data/README.md` §3 — ID 策略与命名规范
> **本文档是 ID 策略的深度展开**，覆盖分配机制、生命周期、校验规则和 Registry 集成。

---

## 1. ID 体系概述

Fre 项目采用**类型前缀 + 定长数字编号**的全局 ID 体系：

```
<类型前缀>_<6 位十进制编号, 0-padded>
       ↓              ↓
  领域标识      递增计数器
```

| 示例 ID | 含义 | 领域 |
|---------|------|------|
| `abl_000042` | 第 42 个 AbilityDef | Ability |
| `eff_000001` | 第 1 个 EffectDef | Effect |
| `tag_000015` | 第 15 个 TagDef | Tag |
| `fct_000003` | 第 3 个 FactionDef | Faction |

### 1.1 设计原则

| 原则 | 理由 |
|------|------|
| **无语义 ID** | `abl_000042` 而非 `ability.fireball`。语义变化需要改 ID，破坏所有引用 |
| **定长填充** | 排序统一、格式化对齐、前缀+编号总长度恒定（10 字符） |
| **永不重用** | ID 一旦分配永久有效，删除时标记 deprecated，不重新分配 |
| **全局唯一** | 前缀隔离不同领域，编号保证领域内唯一 |

### 1.2 为什么是 6 位编号？

6 位十进制 = 0–999,999，每领域 100 万 ID 空间。

- 大型 SRPG 项目的 AbilityDef 通常不超过 5000，EffectDef 不超过 10000
- 6 位有 99 万冗余，覆盖 DLC、Mod、长期运营
- 定长格式化方便文件命名和调试工具解析

---

## 2. ID 分配机制

### 2.1 分配器架构

ID 分配由 `Registry` 领域的 `IdAllocator` 集中管理：

```rust
/// ID 分配器。按类型前缀维护计数器，保证线程安全和持久化。
struct IdAllocator {
    /// 各类型前缀的当前最大编号（下一个可用编号 = max + 1）
    counters: HashMap<IdPrefix, u32>,

    /// 已 deprecated 的 ID 列表（永不重用）
    deprecated: HashSet<String>,

    /// 预留的 ID 范围（用于特定内容规划）
    reserved: Vec<IdRange>,
}

struct IdRange {
    prefix: IdPrefix,
    start: u32,
    end: u32,
    purpose: String,   // 预留用途描述
}
```

### 2.2 分配流程

```
请求新 ID
    │
    ▼
1. IdAllocator 检查该前缀的当前计数器值
    │
    ▼
2. 新编号 = 当前计数器 + 1
    │
    ▼
3. 检查是否在 reserved 范围内 → 如是则跳过（保留给预留用途）
    │
    ▼
4. 检查组编号是否已 deprecated → 如是则跳过（永不重用）
    │
    ▼
5. 分配 ID = 前缀 + 格式化(编号, 6位)
    │
    ▼
6. 更新计数器 = 编号
    │
    ▼
7. 返回 ID
```

### 2.3 批量预留

内容团队在项目规划阶段可为大模块预留 ID 范围：

```
// 预留示例
IdRange { prefix: "eff_", start: 1000, end: 1999, purpose: "核心战斗效果" }
IdRange { prefix: "eff_", start: 2000, end: 2999, purpose: "法术效果" }
IdRange { prefix: "abl_", start: 1000, end: 1999, purpose: "职业能力" }
```

预留范围内的 ID 不被常规分配流程占用，直到内容团队主动填充。

---

## 3. ID 生命周期

```
Allocated（已分配）
    │  [配置文件中首次出现]
    ▼
Active（生效中）
    │  [标记为废弃，但保留引用]
    ▼
Deprecated（已废弃）
    │  [不再使用，但 ID 保留]
    ▼
Archived（已归档——仅存在于历史存档/Replay 中）
```

| 阶段 | 含义 | 能否被引用 | 能否重新分配 |
|------|------|-----------|-------------|
| **Allocated** | ID 已从分配器取出，但尚未在任何配置文件中使用 | 否 | 可回收（未使用） |
| **Active** | ID 在配置文件中定义为活跃内容 | 是 | 否 |
| **Deprecated** | 内容已废弃，但保留 ID 防止存档/Replay 引用断裂 | 仅旧存档/Replay | 否 |
| **Archived** | Deprecated 超过 N 个大版本，且无存档引用 | 否 | 否（永久保留） |

### 3.1 Deprecated 规则

- Deprecated 的 ID 在 Registry 中标记为 `status: deprecated`
- 加载旧存档时，引用了 deprecated ID 的实体使用默认回退行为
- 新配置禁止引用 deprecated ID
- 每个大版本发布时，可归档一批 deprecated 超过 2 个大版本的 ID

---

## 4. ID 引用规范

### 4.1 跨域引用规则

所有 Definition 之间的引用必须通过 ID 字符串，禁止直接嵌入结构：

```rust
// ✅ 正确：引用 ID
struct EffectDef {
    modifier_ids: Vec<ModifierDefId>,  // Vec<String> 或 newtype
}

// ❌ 错误：嵌入结构
struct EffectDef {
    modifiers: Vec<ModifierDef>,  // 直接包含 ModifierDef，违反 Data Law 003
}
```

### 4.2 引用验证

Registry 在加载时执行全量引用检查：

```rust
/// 引用验证结果
struct ReferenceValidation {
    /// 未解析的引用（引用了不存在的 ID）
    dangling_refs: Vec<DanglingRef>,

    /// 引用了 deprecated ID 的引用
    deprecated_refs: Vec<DeprecatedRef>,

    /// 循环引用
    cycles: Vec<Vec<String>>,
}
```

加载时发现 dangling reference → 报告错误并拒绝加载（除非标记为宽松模式）。

### 4.3 序列化格式

ID 在序列化（RON/JSON/存档）中的格式统一为字符串：

```ron
// RON 配置
AbilityDef (
    id: "abl_000042",
    name_key: "ability.abl_000042.name",
    effects: ["eff_000001", "eff_000015"],
)
```

---

## 5. 本地化 Key 策略

### 5.1 Key 格式

```
<命名空间>.<ID>.<后缀>
    ↓         ↓       ↓
  领域      Def ID  字段类型
```

示例：
- `ability.abl_000042.name` — Ability 名称
- `ability.abl_000042.desc` — Ability 描述
- `effect.eff_000001.name` — Effect 名称
- `tag.tag_000015.name` — Tag 名称

### 5.2 后缀规范

| 后缀 | 用途 | 必选 | 最大长度（字符） |
|------|------|------|-----------------|
| `.name` | 显示名称 | 是 | 64 |
| `.desc` | 详细描述 | 是 | 512 |
| `.flavor` | 风味文本 | 否 | 256 |
| `.tooltip` | 工具提示 | 否 | 256 |

### 5.3 本地化文件组织

```
assets/
  localization/
    en/
      ability.ftl        # ability.* 命名空间的本地化
      effect.ftl
      tag.ftl
      item.ftl
      ...
    zh/
      ability.ftl
      effect.ftl
      ...
```

使用 Fluent (`ftl`) 格式，支持复数、变量插值：

```
# assets/localization/en/ability.ftl
ability_abl_000042_name = Fireball
ability_abl_000042_desc = A blazing sphere of fire erupts, dealing {$damage} fire damage.
```

---

## 6. Registry 集成

### 6.1 加载时校验

```
配置加载
    │
    ├── 1. Registry 扫描所有配置文件
    ├── 2. 提取所有 ID 字段
    ├── 3. 检查 ID 格式合法性（前缀 + 6 位数字）
    ├── 4. 检查 ID 是否重复
    ├── 5. 检查 ID 是否已 deprecated（新配置不允许引用 deprecated ID）
    ├── 6. 执行全量引用检查（dangling reference 检测）
    └── 7. 通过 → 加载到内存；失败 → 报告错误列表
```

### 6.2 热重载时的 ID 约束

热重载时，以下变更**不允许**：
- 修改已有 Definition 的 ID（等价于删除+新建）
- 删除正在被其他 Definition 引用的 ID
- 将 Active ID 直接改为 Deprecated（需先确保无引用）

允许的变更：
- 新增 Definition（分配新 ID）
- 修改非 ID 字段
- 修改本地化文本

---

## 7. ID 冲突与错误处理

| 场景 | 处理方式 | 严重级别 |
|------|---------|---------|
| 重复 ID（同一文件内） | 加载失败，报告具体行号 | Error |
| 重复 ID（跨文件） | 加载失败，报告两个文件路径 | Error |
| ID 格式非法 | 跳过该 Definition，报告错误 | Error |
| Dangling reference | 拒绝加载（宽松模式：跳过并警告） | Error / Warning |
| 引用 deprecated ID | 警告，自动使用回退逻辑 | Warning |
| ID 分配器计数器不一致 | 自动取 max(文件中的最大ID, 当前计数器) | Warning |

---

## 8. 与各领域 Schema 的关系

每个领域 Schema 在 ID 策略上的职责：

| 领域 | ID 前缀 | 监管要求 |
|------|---------|---------|
| `tag_schema.md` | `tag_` | Tag ID 在条件/触发器/Modifier 中被大量引用 |
| `attribute_schema.md` | `attr_` | 属性 ID 被 Modifier/Aggregator/Progression 引用 |
| `effect_schema.md` | `eff_` | Effect ID 被 Ability/Trigger/Terrain 引用 |
| `ability_schema.md` | `abl_` | Ability ID 被 Spec/Spell 引用 |
| `modifier_schema.md` | `mod_` | Modifier ID 被 Effect/Equipment/Bond 引用 |
| `item_schema.md` | `itm_` | Item ID 被 Inventory/Crafting/Economy/Quest 引用 |
| `quest_schema.md` | `qst_` | Quest ID 被 Narrative/CampEvent 引用 |
| `faction_schema.md` | `fct_` | Faction ID 被 Shop/Economy 引用 |

---

## 9. Future Extension

- **Mod 支持**: Mod ID 可作为前缀扩展，如 `mod_author.abl_000001`
- **UUID 可选项**: 对于 Mod/用户生成内容，可提供 UUID 模式作为备选（需标记）
- **自动补全工具**: 编辑器插件可根据分配器自动补全下一个可用 ID
- **ID 迁移工具**: 大规模重构时可将 ID 从 6 位升级到 8 位（需 ADR 批准）
