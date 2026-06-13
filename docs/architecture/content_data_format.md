# 内容数据格式规范 — RON 配置文件契约

Version: 1.0
Status: Proposed

Source: `docs/其他/31遗漏.md`（高优先级第 3 项）

本文档定义 SRPG 项目中 RON 配置文件的格式规范、引用机制、校验规则和版本兼容策略。这是数据驱动架构的底层契约，支撑"新增 1000 技能、500 Buff、100 地图只改 RON 文件"的核心目标。

交叉引用：
- `content-pipeline.md` — 内容管线整体架构、数据流、热重载
- `shared_layer_rules.md` — Strong ID 在配置中的引用方式
- `layer-contracts.md` — Content 层职责边界

---

## 概述

RON（Rusty Object Notation）是本项目的配置文件格式。选择 RON 的原因：

- Rust 原生 serde 支持，与 Rust 类型系统天然对齐
- 人类可读，策划可直接编辑
- 支持注释（`//` 和 `/* */`），便于配置说明
- 支持嵌套结构，表达复杂数据
- 社区成熟，Bevy 生态广泛使用

**核心约束**：配置文件是纯数据声明，不包含任何逻辑代码。

---

## 设计原则

### 原则 1：数据即配置

每个 RON 文件只声明数据结构和数值，不包含条件判断、循环或函数调用。逻辑由 Rust 代码解释配置。

### 原则 2：引用靠 ID

配置文件之间通过字符串 ID 互相引用，在 Content 层加载时解析为 Strong ID。禁止直接引用文件路径。

### 原则 3：向后兼容优先

新增字段必须有默认值，删除字段必须有迁移脚本。配置格式的稳定性优先于优雅性。

### 原则 4：校验前置

配置数据在加载时进行 Schema 校验和引用完整性校验，错误配置在启动阶段就被拦截。

---

## 通用配置结构

### 基础字段（所有配置文件必须包含）

```ron
(
    // ── 标识字段（必须）──
    id: "fireball",              // 唯一标识符（Strong ID 的字符串形式）

    // ── 描述字段（推荐）──
    name: "火球术",              // 显示名称
    description: "对目标区域造成火焰伤害",  // 人类可读描述

    // ── 版本控制（必须）──
    version: 1,                  // 配置格式版本号（递增）

    // ── 标签（按需）──
    tags: ["magic", "fire", "aoe"],  // 分类标签（用于过滤、查询）

    // ── 业务数据（按领域定义）──
    // ... 具体领域的字段
)
```

### 字段说明

| 字段 | 类型 | 必须 | 说明 |
|------|------|------|------|
| `id` | `String` | 🟥 必须 | 唯一标识符，全局唯一 |
| `name` | `String` | 🟩 推荐 | 人类可读名称，用于 UI 显示 |
| `description` | `String` | 🟨 优先 | 人类可读描述，用于提示和文档 |
| `version` | `u32` | 🟥 必须 | 配置格式版本号，每次修改递增 |
| `tags` | `Vec<String>` | 🟦 按需 | 分类标签，用于运行时过滤 |

---

## 领域配置结构

### 技能配置（Skill Config）

```ron
// content/skills/fireball.ron
(
    id: "fireball",
    name: "火球术",
    description: "对目标区域造成火焰伤害，附带灼烧效果",
    version: 1,
    tags: ["magic", "fire", "aoe", "offensive"],

    // ── 技能基础属性 ──
    skill_type: "Active",           // Active / Passive / Toggle
    element: "Fire",                // 元素属性
    mana_cost: 15,                  // MP 消耗
    cooldown: 2,                    // 冷却回合数
    cast_range: 3,                  // 施法范围（格数）
    target_type: "SingleEnemy",     // 单体/范围/自身/友方
    area_of_effect: (               // AOE 参数
        shape: "Circle",
        radius: 1,
    ),

    // ── 技能效果 ──
    base_damage: 50,                // 基础伤害
    damage_scaling: 1.5,            // 伤害缩放系数（与 Intelligence 关联）
    damage_type: "MagicFire",       // 伤害类型

    // ── 引用其他配置 ──
    buff_effects: ["burning"],      // 引用 BuffId 列表
    required_traits: ["magic_proficiency"],  // 引用 TraitId 列表

    // ── UI / 表现 ──
    icon: "skills/fireball.png",    // 图标资源路径
    vfx_effect: "vfx/fire_explosion",  // 特效资源路径
    sfx_sound: "sfx/fire_cast",     // 音效资源路径
)
```

### Buff 配置（Buff Config）

```ron
// content/buffs/burning.ron
(
    id: "burning",
    name: "灼烧",
    description: "每回合受到火焰伤害",
    version: 1,
    tags: ["debuff", "fire", "dot"],

    // ── Buff 基础属性 ──
    buff_type: "DoT",               // DoT / HoT / Stat / Control / Shield
    max_stacks: 3,                  // 最大叠加层数
    duration: 3,                    // 持续回合数（-1 = 永久）
    tick_type: "TurnStart",         // TurnStart / TurnEnd / OnAction

    // ── Buff 效果 ──
    damage_per_stack: 10,           // 每层每回合伤害
    stat_modifier: None,            // 属性修饰（可选）
    // stat_modifier: (
    //     attribute: "MagicDefense",
    //     type: "Flat",
    //     value: -5,
    // ),

    // ── 触发条件 ──
    stack_refresh: true,            // 新层是否刷新持续时间
    immune_tags: ["fire_immunity"], // 免疫标签

    // ── 引用 ──
    source_skills: ["fireball", "flame_wave"],  // 可施加此 Buff 的技能

    // ── UI / 表现 ──
    icon: "buffs/burning.png",
    vfx_effect: "vfx/burning_aura",
)
```

### 角色模板配置（Character Template Config）

```ron
// content/characters/warrior.ron
(
    id: "warrior",
    name: "战士",
    description: "近战物理输出，高生命值和防御力",
    version: 1,
    tags: ["melee", "physical", "tank"],

    // ── 种族与职业 ──
    race: "human",                  // 引用 RaceId
    job: "warrior",                 // 引用 JobId

    // ── 基础属性（8 维 Primary Stat）──
    primary_stats: (
        might: 14,
        agility: 10,
        vitality: 13,
        intelligence: 6,
        luck: 8,
        resistance: 9,
        dexterity: 11,
        willpower: 10,
    ),

    // ── 技能池 ──
    skill_pool: [
        "slash",                    // 引用 SkillId
        "shield_bash",
        "battle_cry",
        "heavy_strike",
    ],

    // ── 初始装备 ──
    initial_equipment: [
        (slot: "Weapon", item: "iron_sword"),    // 引用 EquipmentSlot + ItemId
        (slot: "Armor", item: "leather_armor"),
    ],

    // ── Trait 集合 ──
    traits: ["warrior_training", "heavy_armor_proficiency"],

    // ── AI 行为（可选）──
    ai_behavior: "aggressive_melee",  // 引用 AiBehaviorId
)
```

### 关卡配置（Stage Config）

```ron
// content/stages/stage_01.ron
(
    id: "stage_01",
    name: "第一章：觉醒",
    description: "主角初次踏上战场",
    version: 1,
    tags: ["chapter_1", "tutorial"],

    // ── 地图配置 ──
    map: "maps/plains_01",         // 引用地图资源
    map_size: (width: 16, height: 12),

    // ── 友方单位 ──
    player_units: [
        (template: "warrior", position: (x: 2, y: 6)),
        (template: "mage", position: (x: 3, y: 5)),
        (template: "healer", position: (x: 3, y: 7)),
    ],

    // ── 敌方单位 ──
    enemy_units: [
        (template: "goblin", position: (x: 10, y: 5), count: 3),
        (template: "goblin_archer", position: (x: 12, y: 6), count: 2),
        (template: "goblin_chief", position: (x: 14, y: 6), count: 1),
    ],

    // ── 胜负条件 ──
    victory_condition: (
        type: "DefeatAll",
    ),
    defeat_condition: (
        type: "AllPlayerDefeated",
    ),

    // ── 可选配置 ──
    turn_limit: 20,                 // 回合上限（可选）
    difficulty: "Normal",           // 难度（可选）
    background_music: "bgm/battle_01",  // 背景音乐（可选）
)
```

### 地形配置（Terrain Config）

```ron
// content/terrains/plains.ron
(
    id: "plains",
    name: "平原",
    description: "开阔的平原地形，无特殊效果",
    version: 1,
    tags: ["ground", "open"],

    // ── 地形属性 ──
    movement_cost: 1,               // 移动消耗
    defense_bonus: 0,               // 防御加成（%）
    evasion_bonus: 0,               // 闪避加成（%）
    height: 0,                      // 高度等级

    // ── 特殊效果 ──
    burn_damage_bonus: 0,           // 火焰伤害加成
    ice_damage_bonus: 0,            // 冰霜伤害加成
    water_effect: None,             // 水域效果

    // ── 视觉表现 ──
    tile_sprite: "tiles/plains.png",
    color_tint: [0.8, 0.9, 0.6, 1.0],
)
```

### 装备配置（Equipment Config）

```ron
// content/equipments/iron_sword.ron
(
    id: "iron_sword",
    name: "铁剑",
    description: "坚固的铁制长剑",
    version: 1,
    tags: ["weapon", "sword", "physical"],

    // ── 装备属性 ──
    equipment_type: "Weapon",
    slot: "Weapon",

    // ── 属性修饰 ──
    stat_modifiers: [
        (attribute: "Attack", type: "Flat", value: 12),
        (attribute: "CritRate", type: "Percent", value: 5),
    ],

    // ── 装备需求 ──
    requirements: (
        min_level: 1,
        required_job: ["warrior", "swordsman"],
        required_tags: ["melee_proficiency"],
    ),

    // ── 特殊效果 ──
    passive_traits: [],             // 穿戴后获得的 Trait
    on_attack_effects: [],          // 攻击时触发的效果

    // ── 经济属性 ──
    buy_price: 100,
    sell_price: 50,
    weight: 3.0,

    // ── UI 表现 ──
    icon: "equipments/iron_sword.png",
)
```

---

## 引用机制

### 引用规则

配置文件之间的引用遵循以下规则：

1. **通过 ID 引用**：使用字符串 ID，不使用文件路径
2. **引用类型明确**：引用时明确标注引用的实体类型（如 `buff_effects` 引用 BuffId）
3. **加载时解析**：引用在 Content 层加载时解析为 Strong ID
4. **引用完整性校验**：加载后校验所有引用的 ID 是否存在

### 引用格式

```ron
// 引用单个 ID
buff_id: "burning",

// 引用 ID 列表
buff_effects: ["burning", "slow", "stun"],

// 引用结构化配置
initial_equipment: [
    (slot: "Weapon", item: "iron_sword"),
    (slot: "Armor", item: "leather_armor"),
],
```

### 引用解析流程

```
RON 文件中的字符串 ID
    ↓  Content 层加载
XxxDef（包含字符串引用）
    ↓  校验引用完整性
    ↓  解析为 Strong ID
XxxData（包含 Strong ID 引用）
    ↓  Registry.insert()
XxxRegistry（全局注册表）
```

### 引用校验规则

| 校验类型 | 级别 | 失败处理 |
|----------|------|----------|
| 必填引用缺失 | 🟥 错误 | 加载失败，报告错误 |
| 可选引用缺失 | ⚠️ 警告 | 跳过该引用，使用默认值 |
| 循环引用 | 🟥 错误 | 加载失败，报告错误 |
| 引用类型不匹配 | 🟥 错误 | 加载失败，报告错误 |

---

## Schema 校验

### 三级校验体系

```
Level 1: Schema 校验（加载时）
    - RON 语法正确性
    - 必填字段存在性
    - 类型正确性
    - 字段值范围

Level 2: 引用校验（加载后）
    - ID 引用存在性
    - 标签引用存在性
    - 循环依赖检测

Level 3: 规则校验（运行时 / 开发工具）
    - 游戏规则一致性
    - 数值平衡合理性
    - 逻辑矛盾检测
```

### 必填字段校验

每个配置文件的以下字段是必填的：

| 字段 | 校验规则 |
|------|----------|
| `id` | 非空字符串，全局唯一 |
| `version` | 正整数，每次修改递增 |
| `name` | 非空字符串 |

### 类型约束

| 字段类型 | 约束 |
|----------|------|
| 整数 | 游戏数值使用 `i32` 或 `u32`，禁止浮点数（精度问题） |
| 浮点数 | 仅用于缩放系数等非精确计算，需明确精度 |
| 字符串 | 非空，不包含特殊字符 |
| 枚举 | 值必须在预定义的合法值范围内 |
| 列表 | 元素类型一致，无重复（除非语义允许叠加） |

### 默认值规则

新增字段时，必须提供默认值以保证向后兼容：

| 字段类型 | 默认值规则 |
|----------|-----------|
| `Option<T>` | `None` |
| `Vec<T>` | `[]`（空列表） |
| `bool` | `false` |
| `u32` / `i32` | `0` |
| `String` | `""`（空字符串） |
| 枚举 | 第一个变体或 `None` |

---

## 版本兼容策略

### 版本号规则

- `version` 字段每次修改递增（从 1 开始）
- 语义化版本：`major.minor`（但 RON 文件中只用 `version: N` 整数）
- 向后兼容变更：`version += 1`
- 破坏性变更：`version += 1` + 提供迁移脚本

### 向后兼容变更

以下变更不破坏兼容性：

- 新增可选字段（`Option<T>` 类型，有默认值）
- 新增 `tags` 标签
- 新增 `description` 等描述性字段

### 破坏性变更

以下变更破坏兼容性：

- 删除已有字段
- 修改字段类型
- 修改字段语义
- 修改枚举变体

### 迁移脚本

破坏性变更必须提供迁移脚本：

```rust
// content/migrations/migrate_v1_to_v2.rs

/// 将 v1 格式的技能配置迁移到 v2 格式。
pub fn migrate_skill_config_v1_to_v2(old: &SkillDefV1) -> SkillDefV2 {
    SkillDefV2 {
        id: old.id.clone(),
        name: old.name.clone(),
        description: old.description.clone(),
        version: 2,
        // 新字段使用默认值
        new_field: old.calculate_new_field(),
        // ...
    }
}
```

---

## MOD 内容格式

### MOD 配置规范

MOD 内容遵循与基础内容相同的 RON 格式，加载时使用不同的优先级：

```
基础内容（content/）     ← 最先加载
    ↓
MOD 内容（mods/xxx/content/）  ← 后加载覆盖先加载
```

### MOD ID 隔离

MOD 内容的 ID 必须加 MOD 前缀，避免与基础内容冲突：

```ron
// mods/fire_mod/content/skills/inferno.ron
(
    id: "fire_mod.inferno",    // MOD 前缀 + 名称
    name: "炼狱之火",
    // ... 其他字段与标准格式相同
)
```

### MOD 覆盖规则

MOD 可以覆盖基础内容的配置，通过相同的 `id` 实现：

```ron
// mods/balance_mod/content/skills/fireball.ron
(
    id: "fireball",              // 与基础内容相同的 id
    name: "火球术",
    version: 1,
    // 覆盖基础数值
    base_damage: 60,             // 原值 50
    mana_cost: 12,               // 原值 15
)
```

---

## 目录结构

```
content/
├── skills/               # 技能配置
│   ├── fireball.ron
│   ├── ice_shard.ron
│   └── ...
├── buffs/                # Buff 配置
│   ├── burning.ron
│   ├── poison.ron
│   └── ...
├── characters/           # 角色模板
│   ├── warrior.ron
│   ├── mage.ron
│   └── ...
├── classes/              # 职业配置
│   ├── warrior_class.ron
│   └── ...
├── equipments/           # 装备配置
│   ├── iron_sword.ron
│   └── ...
├── items/                # 物品配置
│   ├── health_potion.ron
│   └── ...
├── stages/               # 关卡配置
│   ├── stage_01.ron
│   └── ...
├── terrains/             # 地形配置
│   ├── plains.ron
│   └── ...
├── ai_behaviors/         # AI 行为
│   ├── aggressive_melee.ron
│   └── ...
├── effects/              # 效果配置
│   ├── direct_damage.ron
│   └── ...
├── formulas/             # 公式配置
│   ├── damage_formula.ron
│   └── ...
├── factions/             # 阵营配置
│   └── ...
├── loot_tables/          # 掉落表
│   └── ...
└── migrations/           # 版本迁移脚本
    ├── migrate_v1_to_v2.rs
    └── ...
```

---

## 允许的模式

### 模式 1：引用其他配置

```ron
// ✅ 允许：通过 ID 引用
buff_effects: ["burning", "slow"],
required_traits: ["magic_proficiency"],

// ❌ 禁止：通过文件路径引用
// buff_effects: ["content/buffs/burning.ron"],
```

### 模式 2：嵌套结构

```ron
// ✅ 允许：嵌套表达复杂配置
area_of_effect: (
    shape: "Circle",
    radius: 1,
),
initial_equipment: [
    (slot: "Weapon", item: "iron_sword"),
],
```

### 模式 3：注释说明

```ron
// ✅ 允许：使用注释说明配置含义
// 火球术：基础火焰伤害技能
// 伤害公式：base_damage + intelligence * damage_scaling
(
    id: "fireball",
    base_damage: 50,
    damage_scaling: 1.5,
)
```

---

## 禁止事项

### 🟥 绝对禁止

| 禁止行为 | 原因 | 违反后果 |
|----------|------|----------|
| 在 RON 文件中包含逻辑代码 | 配置是纯数据声明 | 配置与代码耦合，无法独立修改 |
| 硬编码配置值在 Rust 代码中 | 违反数据驱动原则 | 新增内容必须修改代码 |
| 配置文件之间循环引用 | 加载顺序不确定 | 加载失败或无限循环 |
| 使用文件路径引用其他配置 | 路径脆弱，不可移植 | 移动文件后引用失效 |
| 删除已有字段不提供迁移 | 破坏旧版本配置兼容性 | 旧配置加载失败 |
| 在 RON 中使用位掩码值 | 不可读 | 应使用 TagName 字符串 |
| 新增必填字段不递增版本号 | 版本号失去意义 | 无法判断配置兼容性 |

### 🟩 必须遵守

| 必须行为 | 原因 |
|----------|------|
| 每个配置文件包含 `id` 和 `version` 字段 | 标识和版本管理 |
| 新增字段提供默认值 | 向后兼容 |
| 引用的 ID 在对应 Registry 中存在 | 引用完整性 |
| 破坏性变更提供迁移脚本 | 版本兼容 |
| MOD 配置加 MOD 前缀 | ID 隔离 |
| 配置文件使用 UTF-8 编码 | 跨平台兼容 |

---

## AI 修改规则

### 如果新增配置文件

允许：
- 在 `content/` 对应目录创建新的 `.ron` 文件
- 遵循通用配置结构（id、name、version、tags）
- 在 RON 中添加注释说明配置含义

禁止：
- 不包含 `id` 和 `version` 字段
- 在 RON 中包含逻辑代码
- 引用不存在的 ID

优先检查：
- 配置结构是否与 Core 层的 Def 类型一致
- 所有引用的 ID 是否在对应 Registry 中存在
- 版本号是否正确递增

### 如果修改现有配置格式

允许：
- 新增可选字段（提供默认值）
- 优化配置结构（保持向后兼容）

禁止：
- 删除已有字段（破坏兼容性）
- 修改字段类型（破坏兼容性）
- 修改字段语义（破坏兼容性）

优先检查：
- 所有使用该配置格式的模块是否同步更新
- 版本号是否递增
- 是否需要提供迁移脚本
