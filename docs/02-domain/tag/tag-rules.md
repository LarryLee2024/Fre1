---
id: 02-domain.tag.tag-rules
title: Tag Rules
status: draft
owner: domain-designer
created: 2026-06-14
updated: 2026-06-14
tags:
  - domain
  - tag
---

# 标签系统领域

Version: 1.0
Status: Proposed
Source: `docs/01-architecture/skill-buff-abstraction.md` §4.9、`docs/08-decisions/ADR-023-标签系统架构重整.md`、`docs/08-decisions/ADR-015-技能标签与分类体系.md`

标签系统（GameplayTag）是 SRPG 中跨所有业务领域的核心分类机制，被技能、Buff、装备、物品、角色、地图、修饰规则、AI 决策等多个子系统依赖。标签的本质是**用位掩码实现的 O(1) 分类查询**，消灭 if 地狱。

**核心原则**：
- 🟩 GameplayTag 位掩码是运行时查询的唯一方式，O(1) 复杂度
- 🟥 TagName 枚举 + RON 元数据双类型模式：枚举保证类型安全，RON 保证数据驱动
- 🟥 三重数据源必须收敛为单一事实源（RON 是元数据唯一来源）
- 🟥 三层标签管理：Trait（Layer 1）→ Equipment（Layer 2）→ Buff（Layer 3）
- 🟥 新增标签元数据只改 RON，不改 Rust 代码（元数据部分）

**领域定位**：

```
Skill ── 意图（Intent）：我要做什么
  ↓ 标签驱动
Effect ── 结果（Result）：产生什么效果
  ↓ 标签匹配
Modifier ── 计算规则（Rule）：如何调整数值
  ↓ 标签查询
Tag ── 分类（Classification）：怎么分类和匹配 ← 本领域
```

---

# 统一术语

## GameplayTag（标签位掩码）

用 u64 位掩码表示的单个分类标签，支持 O(1) 位运算查询。

不是字符串。不是枚举变体。不是标签集合。

关键属性：
- 内部表示为 u64 的单个 bit（如 `1 << 0` = FIRE）
- 当前已使用 37/64 bits
- 位运算查询：`has()` / `has_any()` / `has_all()` 均为 O(1)
- 通过 TagName 枚举转换（`to_tag()`），零开销

---

## TagName（标签名称枚举）

用于 RON 配置文件反序列化的枚举类型，提供编译时类型安全。

不是 GameplayTag 位掩码。不是运行时查询方式。不是标签集合。

关键属性：
- 枚举变体使用 PascalCase（如 `Fire`、`Ice`、`SkillActive`）
- RON 中使用 SCREAMING_SNAKE_CASE（如 `FIRE`、`ICE`、`SKILL_ACTIVE`）
- 每个变体映射到唯一的 GameplayTag 位掩码（`to_tag()`）
- `TagName::ALL` 静态数组包含所有变体，用于遍历校验
- 新增标签必须在此枚举中添加变体

---

## GameplayTags（标签集合组件）

Entity 上的标签集合，存储为 u64 位掩码，支持多标签组合查询。

不是单个 GameplayTag。不是 PersistentTags。不是 TagRegistry。

关键属性：
- Bevy Component，附加在单位实体上
- 通过 `add()` / `remove()` 增删标签
- `has(tag)` — 单标签查询（O(1)）
- `has_any(tags)` — 任一匹配（O(1)）
- `has_all(tags)` — 全部匹配（O(1)）
- `from_tags(&[GameplayTag])` — 从标签列表构建
- `active_tags()` — 返回已激活的标签列表（遍历 ALL_TAGS）

---

## PersistentTags（持久化标签来源）

管理标签的多层来源，支持 Trait + Equipment 两层叠加。

不是 GameplayTags。不是 TagRegistry。不是 TagDefinition。

关键属性：
- `from_traits`：Trait 授予的标签（种族/职业/天赋，最持久）
- `from_equipment`：装备授予的标签（穿脱变化）
- 变化时通过 `rebuild_tags()` 重建 GameplayTags
- 三层标签管理中的 Layer 1（Trait）和 Layer 2（Equipment）
- Layer 3（Buff）由 ActiveBuffs 在运行时管理

---

## TagDefinition（标签定义）

标签的元数据定义，从 RON 文件加载，包含显示名、描述、分类。

不是 GameplayTag。不是 TagName。不是 GameplayTags。

关键属性：
- `tag`：TagName 枚举值（关联哪个标签）
- `display_name`：显示名称（旧字段，直接文本）
- `display_name_key`：本地化 Key（新字段，优先使用）
- `description`：描述文本
- `desc_key`：本地化描述 Key
- `category`：TagCategory 分类
- `version`：配置版本号（预留）

---

## TagRegistry（标签注册表）

全局唯一的标签元数据注册表 Resource，从 RON 加载。

不是 GameplayTag。不是 GameplayTags。不是 ModifierRuleRegistry。

关键属性：
- `definitions: HashMap<GameplayTag, TagDefinition>` 存储
- `get(tag)` — 获取标签定义
- `display_name(tag)` — 获取显示名称（找不到回退到默认值）
- `tags_by_category(category)` — 按分类查询标签
- `from_enum_defaults()` — RON 缺失时从枚举生成默认元数据
- `ron_missing_tags()` — 检查 RON 中缺失的标签定义
- `bit_mask_usage_warning()` — 位掩码使用率 ≥ 80% 告警

---

## TagCategory（标签分类）

标签的分类维度，用于按类别查询标签。

不是 GameplayTag。不是 TagName。

关键属性：
- 5 个分类维度（Elemental / Status / Class / Equipment / Mechanism）— 由 ADR-031 确认的折中方案，保留 Equipment 分类以支持装备/物品标签查询
- 废弃的历史分类：Weapon, WeaponType, Movement, SkillType, BuffType, ItemType, EquipmentAttribute（合并入 Equipment 或 Class）
- `default_for(name)` — 根据 TagName 返回默认分类
- 每个标签必须属于且仅属于一个分类
- 分类覆盖所有现有标签

---

# 状态机

## Tag 生命周期

标签本身是静态定义，没有复杂的状态机。标签在 Entity 上的生命周期如下：

```
Defined → Loaded → Applied → Removed
```

| 状态 | 含义 | 触发条件 |
|------|------|----------|
| Defined | TagName 枚举定义 + RON 元数据加载 | 游戏启动（TagDefPlugin） |
| Loaded | TagRegistry 就绪，所有 TagDefinition 注册完成 | TagDefPlugin::build() 完成 |
| Applied | GameplayTag 已添加到 Entity 的 GameplayTags 组件 | spawn / 装备穿戴 / Buff 施加 |
| Removed | GameplayTag 已从 Entity 的 GameplayTags 组件移除 | 装备脱下 / Buff 过期 / rebuild_tags() |

**状态转换图**：

```
Defined → Loaded → Applied → Removed
                           ↑        │
                           └────────┘  （Buff 刷新 / 装备切换）
```

## 标签重建状态

PersistentTags 的重建是标签管理的核心状态转换：

```
Trait 变化 → rebuild_tags() → GameplayTags 更新
装备穿脱 → rebuild_tags() → GameplayTags 更新
Buff 施加/移除 → 直接修改 GameplayTags（Layer 3）
```

**三层合并公式**：
```
GameplayTags = from_traits | from_equipment | from_buffs
```

---

# 不变量

## 不变量1：标签位掩码唯一性 🟥

任意时刻：

每个 GameplayTag 占据且仅占据 u64 中的一个 bit。不允许两个不同的 TagName 映射到同一个 bit 位。

违反表现：
两个不同标签的位掩码值相同，导致查询时误匹配。

---

## 不变量2：u64 位掩码容量上限 🟥

任意时刻：

当前已使用 37/64 bits。当使用率 ≥ 80%（51 bits）时，TagDefPlugin 启动时输出 WARN 日志，提示准备迁移到分层标签系统。

违反表现：
超过 64 个标签需要新 bit 位，无可用位。

---

## 不变量3：三层标签管理不变 🟥

任意时刻：

Entity 的 GameplayTags 必须且仅由三层来源合并而成：
- Layer 1：Trait（from_traits）— 种族/职业/天赋授予的永久标签
- Layer 2：Equipment（from_equipment）— 装备穿戴期间的标签
- Layer 3：Buff（由 ActiveBuffs 运行时管理）— Buff 持续期间的标签

违反表现：
标签来源不清晰，某些标签无法追溯到来源层。

---

## 不变量4：RON 是元数据唯一事实源 🟥

任意时刻：

标签元数据（display_name、description、category）的唯一事实源是 `content/definitions/tags.ron`。`TagRegistry::register_defaults()` 中的硬编码元数据已废弃，仅作为 RON 缺失时的回退方案。

违反表现：
RON 文件与 TagRegistry 中的元数据不一致，显示名使用 code 硬编码而非 RON。

---

## 不变量5：标签查询必须使用位掩码 🟥

任意时刻：

运行时标签查询必须使用 GameplayTag 位掩码（`has()` / `has_any()` / `has_all()`），禁止使用字符串匹配。

违反表现：
`if tag_name == "fire"` 字符串比较出现。

---

## 不变量6：rebuild_tags() 统一合并 🟥

任意时刻：

Trait 或装备变化后，必须通过 `rebuild_tags()` 函数从 PersistentTags 重建 GameplayTags，禁止手动位运算合并。

违反表现：
`tags.0 = persistent.from_traits.0 | persistent.from_equipment.0` 直接位运算出现在 spawn.rs 或其他业务模块中（应调用 `rebuild_tags()`）。

---

## 不变量7：新增标签必须同步 RON 🟥

任意时刻：

新增标签时，必须同时更新以下三个位置：
1. `src/core/tag.rs`：TagName 枚举变体 + GameplayTag 常量 + label() + to_tag() + active_tags()
2. `content/definitions/tags.ron`：TagDefinition 条目

违反表现：
代码中新增了 GameplayTag 但 RON 中缺少对应定义，导致 `ron_missing_tags()` 返回缺失列表。

---

# 禁止事项

## 🟥 绝对禁止

**禁止：label() 用于 UI 显示**

原因：label() 返回硬编码中文字符串，违反国际化架构（ADR-017/ADR-018）的"Key 驱动内容"原则。应使用 `TagRegistry::display_name(tag)`。

违反后果：UI 显示中文硬编码，无法国际化。

---

**禁止：register_defaults() 模式**

原因：RON 内容在 Rust 代码中重复，违反 Rule/Content 分离原则。标签元数据必须从 RON 加载。

违反后果：两套元数据源，修改一处忘记另一处。

---

**禁止：新增标签不添加 RON 定义**

原因：导致元数据缺失，标签在 UI 中无显示名、无分类。

违反后果：标签查询失效，显示"未知"。

---

**禁止：运行时使用字符串查询标签**

原因：字符串匹配性能差（O(n)）、类型不安全、易拼写错误。必须使用 GameplayTag 位掩码（O(1)）。

违反后果：性能下降，运行时错误。

---

**禁止：绕过 TagRegistry 直接访问标签元数据**

原因：TagRegistry 是标签元数据的唯一访问入口，绕过它会导致数据不一致。

违反后果：UI 显示名不一致，分类查询失效。

---

**禁止：在 RON 中使用位掩码数值**

原因：可读性差，应使用 TagName 枚举（如 `FIRE` 而非 `1`）。

违反后果：配置文件不可读，维护困难。

---

**禁止：跳过 rebuild_tags() 手动位运算**

原因：标签来源管理混乱，无法追踪标签来自 Trait 还是 Equipment。

违反后果：标签残留，装备脱下后标签未清除。

---

**禁止：TagCategory 不匹配实际标签分类**

原因：导致分类查询失效，按类别查询返回空结果。

违反后果：UI 标签过滤器无法正确工作。

---

# 流程定义

## 标签加载管线

```
content/definitions/tags.ron
    ↓
TagDefPlugin::build()
    ↓
TagRegistry::load_from_file()
    ↓
校验：RON 覆盖所有 TagName 变体？
    ↓ 是                    ↓ 否
insert_resource          从 TagName 枚举生成默认元数据
    ↓                         ↓
校验：位掩码使用率 < 80%？  输出 WARN 日志
    ↓ 是                    ↓ 否
正常启动                   输出 WARN 日志（准备迁移）
    ↓
TagRegistry 就绪
```

## 标签重建管线

```
Trait 变化 / 装备穿脱
    ↓
PersistentTags 更新
    ↓
rebuild_tags(&PersistentTags)
    ↓
GameplayTags = from_traits | from_equipment
    ↓
GameplayTags 组件更新
    ↓
下游系统可查询新标签状态
```

## 标签匹配管线（ModifierRule）

```
Effect Pipeline Modify 阶段
    ↓
ModifierRule 遍历
    ↓
source_tags.contains(rule.source_tag)？  → 否 → 跳过
    ↓ 是
target_tags.has(rule.target_tag)？  → 否 → 跳过
    ↓ 是
ModifierCalculator::calculate() 记录 ModifierEntry
```

## 标签查询管线

```
查询方式         → 实现              → 复杂度
单标签查询       → has(tag)          → O(1) 位与
任一匹配         → has_any(tags)     → O(1) 位与
全部匹配         → has_all(tags)     → O(1) 位与
按分类查询       → TagRegistry.tags_by_category() → O(n) 遍历
显示名查询       → TagRegistry.display_name()     → O(1) HashMap
```

---

# 领域事件

## TagAdded — 标签已添加

当 GameplayTag 被添加到 Entity 的 GameplayTags 组件时触发。

数据：
- `entity: Entity` — 目标实体
- `tag: GameplayTag` — 被添加的标签
- `source: TagSource` — 来源（Trait / Equipment / Buff）

消费方：
- BattleRecord（记录标签变化）
- UI（标签过滤器刷新）
- Debug（标签状态查看）

---

## TagRemoved — 标签已移除

当 GameplayTag 被从 Entity 的 GameplayTags 组件移除时触发。

数据：
- `entity: Entity` — 目标实体
- `tag: GameplayTag` — 被移除的标签
- `source: TagSource` — 来源（Trait / Equipment / Buff）

消费方：
- BattleRecord（记录标签变化）
- UI（标签过滤器刷新）
- Debug（标签状态查看）

---

# 与相邻领域的关系

| 相邻领域 | 关系 | 边界 |
|----------|------|------|
| **AttributeModifier** | ModifierRule 通过标签匹配决定是否应用修饰 | Tag 提供分类查询；AttributeModifier 拥有 Modify 阶段内部逻辑 |
| **Effect** | Effect 的 source_tags 用于标签匹配 | Tag 提供分类元数据；Effect 拥有 Pipeline 编排 |
| **Skill** | SkillDef 的 tags 字段引用 GameplayTag | Skill 引用标签进行分类；Tag 不关心技能释放逻辑 |
| **Buff** | BuffDef 的 tags 字段引用 GameplayTag | Buff 引用标签进行分类；Tag 不关心 Buff 生命周期 |
| **Equipment** | EquipmentDef 的 tags 影响 PersistentTags.from_equipment | Equipment 提供标签来源；Tag 不关心装备穿脱逻辑 |
| **Character** | Unit 的 GameplayTags 和 PersistentTags 组件 | Character 管理实体上的标签组件；Tag 提供查询 API |
| **AI** | AI 通过 GameplayTags.has() 评估目标状态 | Tag 提供分类查询；AI 拥有决策逻辑 |
| **Formula** | Formula 通过标签区分伤害类型 | Tag 提供分类元数据；Formula 拥有计算逻辑 |
| **Map** | 地形通过标签区分类型（如草地/水域） | Tag 提供分类元数据；Map 拥有寻路逻辑 |

---

# 数据结构

## GameplayTag（标签位掩码）

```
GameplayTag(u64)
├── FIRE          = 1 << 0
├── ICE           = 1 << 1
├── POISON        = 1 << 2
├── STUN          = 1 << 8
├── ...（37 个标签已定义）
└── 共 64 bits 可用
```

## GameplayTags（标签集合组件）

```
GameplayTags(u64)  ← Bevy Component
├── has(tag)         → bool
├── has_any(tags)    → bool
├── has_all(tags)    → bool
├── add(tag)         → ()
├── remove(tag)      → ()
├── from_tags(list)  → Self
└── active_tags()    → Vec<GameplayTag>
```

## PersistentTags（持久化标签来源）

```
PersistentTags
├── from_traits: GameplayTags      ← Layer 1: Trait（种族/职业/天赋）
├── from_equipment: GameplayTags   ← Layer 2: Equipment（装备穿戴）
└── （Layer 3: Buff 由 ActiveBuffs 运行时管理）
```

## TagDefinition（标签定义）

```
TagDefinition
├── tag: TagName                    ← 关联的标签枚举
├── display_name: String            ← 显示名称（旧字段）
├── display_name_key: Option<String> ← 本地化 Key（新字段，优先）
├── description: String             ← 描述文本
├── desc_key: Option<String>        ← 本地化描述 Key
├── category: TagCategory           ← 分类
└── version: u32                    ← 配置版本号
```

## TagCategory（标签分类）

```
TagCategory（10 个维度）
├── Element              ← FIRE, ICE, POISON
├── Status               ← STUN, BURN, REGEN
├── Weapon               ← MELEE, RANGED
├── WeaponType           ← SWORD, AXE, BOW, STAFF
├── Class                ← WARRIOR, ARCHER, MAGE
├── Movement             ← FLYING, MOUNTED, SWIMMING
├── SkillType            ← SKILL_ACTIVE, SKILL_PASSIVE
├── BuffType             ← BUFF, DEBUFF
├── ItemType             ← CONSUMABLE, AMMO, MATERIAL, CURRENCY, QUEST_ITEM
└── EquipmentAttribute   ← HEAVY_ARMOR, LIGHT_ARMOR, SHIELD, TWO_HANDED, MARTIAL, SIMPLE
```

---

# AI 修改规则

## 宪法合规检查清单

修改本领域代码前，必须逐项确认：
- 🟥 GameplayTag 位掩码唯一性（每个 bit 只对应一个标签）
- 🟥 u64 位掩码容量上限（使用率 ≥ 80% 时告警）
- 🟥 三层标签管理（Trait / Equipment / Buff）
- 🟥 RON 是元数据唯一事实源
- 🟥 标签查询使用位掩码（禁止字符串匹配）
- 🟥 rebuild_tags() 统一合并（禁止手动位运算）
- 🟥 新增标签必须同步 RON

## 如果新增标签

允许：
- 添加 TagName 枚举变体
- 添加 GameplayTag 常量（分配新 bit 位）
- 添加 to_tag() 分支
- 添加 active_tags() 静态列表条目
- 添加 content/definitions/tags.ron 条目

禁止：
- 分配已占用的 bit 位
- 不添加 RON 定义
- 不添加 TagCategory 分类

优先检查：
- bit 位是否与现有标签冲突
- TagCategory 是否正确分类
- RON 中 display_name 和 description 是否完整
- 位掩码使用率是否接近 80% 告警阈值

---

## 如果修改标签分类

允许：
- 调整 TagCategory 枚举变体
- 调整 TagCategory::default_for() 映射

禁止：
- 删除已有分类（破坏向后兼容）
- 不同步更新 RON 中的 category 字段

优先检查：
- 所有 TagName 变体是否有对应的默认分类
- 按分类查询结果是否正确
- UI 标签过滤器是否兼容新分类

---

## 如果测试失败

排查顺序：
1. GameplayTag 位掩码是否唯一（是否有两个标签使用同一个 bit）
2. TagName 枚举变体是否与 GameplayTag 常量对应（to_tag() 映射）
3. RON 文件是否覆盖所有 TagName 变体（ron_missing_tags()）
4. PersistentTags 的 from_traits 和 from_equipment 是否正确合并
5. rebuild_tags() 是否被调用（Trait/装备变化后）
6. GameplayTags.has() 查询的标签位是否正确

---

# 交叉引用

| 主题 | 详细文档 |
|------|----------|
| GameplayTag 位掩码实现 | `src/core/tag.rs` |
| TagRegistry 定义与加载 | `src/core/tag_def.rs` |
| 标签 RON 定义 | `content/definitions/tags.ron` |
| 标签在技能/Buff 中的使用 | `docs/01-architecture/skill-buff-abstraction.md` §4.9 |
| 标签系统架构重整 | `docs/08-decisions/ADR-023-标签系统架构重整.md` |
| 技能标签与分类体系 | `docs/08-decisions/ADR-015-技能标签与分类体系.md` |
| ModifierRule 标签匹配 | `docs/02-domain/attribute-modifier/attribute-modifier-rules.md` 规则3 |
| Effect Pipeline 标签流转 | `docs/02-domain/effect/effect-rules.md` 效果管线 |
| 七层架构与模块边界 | `docs/01-architecture/README.md` |

---

## 附录：铃兰参考数据

> 领域：Tag | 来源：78铃兰.md §补充1、补充2、§四 | 数据层：Definition + Instance

#### TagDefinition（Definition层）

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| `id` | TagId | PK | 标签唯一标识 |
| `name_key` | String | - | 标签名称本地化Key |
| `category` | Enum | damage_type/status/faction/mechanism | 标签分类 |
| `priority_weight` | u32 | ≥0 | 优先级权重（控制类使用） |
| `dispellable` | bool | - | 是否可驱散 |
| `reflectable` | bool | - | 是否可反弹 |
| `mutually_exclusive_with` | Vec<TagId> | - | 互斥标签列表 |

#### Tag四大分类

**伤害类型Tag（category = damage_type）**

| TagId | 名称 | 说明 |
|-------|------|------|
| `dmg_physical` | 物理伤害 | 受物防减免，物理护盾可挡 |
| `dmg_magical` | 魔法伤害 | 受魔防减免，魔法护盾可挡 |
| `dmg_pierce` | 穿透伤害 | 无视防御，部分护盾可挡 |
| `dmg_true` | 真实伤害 | 无视防御无视护盾 |
| `dmg_fire` | 火焰 | 元素子类型 |
| `dmg_ice` | 寒冰 | 元素子类型 |

**状态Tag（category = status）**

| TagId | 名称 | 控制层级 | 可驱散 | 说明 |
|-------|------|----------|--------|------|
| `buff` | 增益 | - | 是 | 橙色状态 |
| `debuff` | 减益 | - | 是 | 红色/紫色状态 |
| `special_state` | 特殊状态 | - | 否 | 蓝色/灰色，不可驱散 |
| `control_soft` | 软控 | 1 | 是 | 减速、命中降低、攻击降低 |
| `control_hard` | 硬控 | 2 | 是 | 定身、束缚、嘲讽 |
| `control_full` | 强控 | 3 | 是 | 眩晕、冰冻、石化 |
| `invincible` | 无敌 | - | 否 | 最高权限机制Tag |
| `untargetable` | 不可选中 | - | 否 | 屏蔽选中类效果 |

**阵营/身份Tag（category = faction）**

| TagId | 名称 | 说明 |
|-------|------|------|
| `ally` | 友方 | 己方单位 |
| `enemy` | 敌方 | 对方单位 |
| `summon` | 召唤物 | 召唤产生的单位 |
| `boss` | Boss | Boss级单位 |
| `mechanical` | 机械 | 机械类单位 |

**机制Tag（category = mechanism）**

| TagId | 名称 | 说明 |
|-------|------|------|
| `dispellable` | 可驱散 | 可被驱散效果移除 |
| `undispellable` | 不可驱散 | 只能靠时间衰减 |
| `reflectable` | 可反弹 | 可被反弹效果反射 |
| `untriggerable` | 不可触发 | 不触发被动 |
| `flying` | 飞行 | 不触发地面地形效果 |
| `grounded` | 地面 | 触发地面地形效果 |
| `no_cooldown_refresh` | 禁止刷新冷却 | 屏蔽冷却刷新效果 |

#### 控制层级与免疫规则

| 层级 | 名称 | 效果 | 覆盖规则 |
|------|------|------|----------|
| 1 | 软控（削弱层） | 不限制行动，仅削弱属性 | 被硬控/强控覆盖 |
| 2 | 硬控（行动限制层） | 禁止移动/改变目标，可释放技能/普攻 | 被强控覆盖 |
| 3 | 强控（完全失能层） | 完全禁止所有行动 | 最高级 |

**免疫分级**

| 免疫类型 | 免疫范围 |
|----------|----------|
| 免疫控制 | 硬控 + 强控 |
| 免疫行动限制 | 仅硬控 |
| 免疫眩晕 | 仅单种状态 |

#### Schema草案

```yaml
# tag_config.ron
(
  tags: [
    (id: "dmg_physical", category: DamageType, priority_weight: 0, dispellable: false, reflectable: true),
    (id: "dmg_magical", category: DamageType, priority_weight: 0, dispellable: false, reflectable: true),
    (id: "dmg_pierce", category: DamageType, priority_weight: 0, dispellable: false, reflectable: false),
    (id: "dmg_true", category: DamageType, priority_weight: 0, dispellable: false, reflectable: false),
    (id: "control_soft", category: Status, priority_weight: 1, dispellable: true, reflectable: false),
    (id: "control_hard", category: Status, priority_weight: 2, dispellable: true, reflectable: false),
    (id: "control_full", category: Status, priority_weight: 3, dispellable: true, reflectable: false),
    (id: "invincible", category: Mechanism, priority_weight: 99, dispellable: false, reflectable: false),
    (id: "untargetable", category: Mechanism, priority_weight: 98, dispellable: false, reflectable: false),
  ],
  mutual_exclusions: [
    (tag_a: "flying", tag_b: "grounded"),
    (tag_a: "control_full", tag_b: "control_hard"),
    (tag_a: "control_full", tag_b: "control_soft"),
  ],
)
```
