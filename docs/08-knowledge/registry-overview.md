---
id: 08-knowledge.registry
title: Registry 注册中心系统深度解析
status: draft
owner: architect
created: 2026-06-21
updated: 2026-06-21
tags:
  - knowledge
  - registry
  - definition
---

# Registry 注册中心系统深度解析

> 从宪法到代码：注册中心如何设计、项目中有哪些注册中心、数据如何流动。

---

## 0. 先讲个故事：没有注册中心的世界

想象你是一个游戏策划，要给游戏加一个新技能"火球术"。你需要：

1. 在配置文件夹里写一个 `abl_000042.ron` 文件，描述火球术的伤害、范围、冷却
2. 告诉程序"火球术"这个技能存在
3. 确保没有其他技能用了 `abl_000042` 这个 ID
4. 让战斗系统能查到这个技能的数据

如果没有注册中心，你会遇到这些问题：

- **找不着**：战斗系统不知道技能文件在哪，得手动告诉它
- **冲突**：两个人各写了一个技能，都用了 ID 000042，后加载的把前面的覆盖了
- **改不动**：改技能数值要重启游戏，策划想实时看效果
- **没校验**：技能引用了"灼烧"效果，但"灼烧"效果根本没定义，运行时崩了

注册中心就是解决这些问题的统一枢纽。

---

## 1. 什么是 Registry？

**Registry（注册中心）** 是游戏所有"定义数据"的中央数据库。类比现实：

| 现实场景 | 注册中心类比 |
|---------|-------------|
| 图书馆的图书目录 | 按类型（技能/效果/标签）分桶，按 ID 索引 |
| 饭店的点菜系统 | 菜名（ID）→ 做法（Def 数据） |
| 电话簿 | 姓名（ID）→ 号码（数据），改了重印（热重载） |

在 Fre 项目里，所有"静态的、配置化的、运行时只读的"游戏数据——技能定义、效果定义、标签体系、属性定义、物品模板——都经过注册中心管理和访问。

---

## 2. 项目里到底有几个"注册中心"？

这是个容易混淆的地方：Fre 项目里有**三套不同层级的注册相关系统**，外加多个**领域专属注册表**。它们职责不同，但协作紧密。

### 2.1 基础设施级：DefinitionRegistry（infra/registry）

```
src/infra/registry/
├── registry.rs              # DefinitionRegistry Resource + RegistryBucket<T>
├── resolver.rs              # ID 分配器 (IdAllocator) + 校验器 (ValidationRunner)
└── plugin.rs                # RegistryPlugin（初始化注册中心）
```

**核心类型：`DefinitionRegistry`**

这是一个 Bevy Resource，管理 16 个命名桶（Bucket）：

```
abilities       ── 技能定义桶       triggers   ── 触发器定义桶
effects         ── 效果定义桶       cues       ── 表现信号桶
modifiers       ── 修改器桶         items      ── 物品定义桶
tags            ── 标签体系桶       spells     ── 法术定义桶
attributes      ── 属性定义桶       buffs      ── 增益/减益桶
factions        ── 阵营定义桶       terrains   ── 地形定义桶
recipes         ── 配方桶           loot_tables ── 战利品表桶
quests          ── 任务定义桶       custom     ── 自定义扩展桶
```

每个桶都是一个 `RegistryBucket<RegistryEntry>`，本质是 `HashMap<DefinitionId, RegistryEntry>`
+ 分类索引 + 版本号。提供完整的增删改查和变更追踪：

```rust
pub struct RegistryBucket<T> {
    items: HashMap<DefinitionId, T>,    // 核心映射
    indices: HashMap<IndexKey, Vec<DefinitionId>>,  // 分类索引
    version: u64,                        // 变更版本号
}
```

`IndexKey` 是分类查询的关键——比如按 `category`、`movement_type`、`damage_type` 索引：

```rust
// 给地形条目加索引
bucket.add_index(IndexKey::new("category", "water"), ter_id);

// 按索引查所有水系地形
let water_terrains = bucket.query_index(&IndexKey::new("category", "water"));
```

**当前状态：v1 存储模式**。目前每个条目以 `serde_json::Value` 形式直接存值。v2 目标（见 registry_schema.md）是改为 Handle 间接存储，对接 Bevy 原生 Asset 系统的热重载能力。

### 2.2 运行时能力级：DefRegistry（core/capabilities/runtime/registry）

```
src/core/capabilities/runtime/registry/
├── foundation/
│   ├── types.rs             # IdType / AllocatorState / IdAllocator / RegistryEntry
│   ├── values.rs            # DefRegistry（String → RegistryEntry 映射）
│   └── error.rs             # RegistryError 枚举
├── mechanism/
│   └── validator.rs         # ID 校验 + 跨 Def 引用检查
└── events.rs                # DefRegistered / DefDeprecated / RegistryValidated 事件
```

**核心类型：`DefRegistry`**

这是一个更轻量的 `HashMap<String, RegistryEntry>`，不区分桶，所有 Def 类型混在一个大表里：

```rust
pub struct DefRegistry {
    entries: HashMap<String, RegistryEntry>,     // def_id → entry
    type_index: HashMap<String, Vec<String>>,    // def_type → [def_id]
}

impl DefRegistry {
    pub fn register(&mut self, entry: RegistryEntry) -> Result<(), RegistryError>;
    pub fn get(&self, def_id: &str) -> Option<&RegistryEntry>;
    pub fn get_by_type(&self, def_type: &str) -> Vec<&RegistryEntry>;
    pub fn mark_deprecated(&mut self, def_id: &str, superseded_by: Option<String>);
    pub fn all_entries(&self) -> Vec<&RegistryEntry>;
}
```

它的事件体系比较完整——注册、废弃、校验完成后都会发出 Bevy Event：

```rust
#[derive(Event)]
pub struct DefRegistered { pub def_id: String, pub def_type: String }

#[derive(Event)]
pub struct DefDeprecated { pub def_id: String, pub superseded_by: Option<String> }

#[derive(Event)]
pub struct RegistryValidated { pub total_defs: u32, pub broken_refs: u32, pub passed: bool }
```

### 2.3 内容加载层：Loaded*Defs（content/）

```
src/content/
├── content_plugin.rs        # ContentPlugin + 各个 Loaded*Defs Resource（~1000行）
├── loading/
│   ├── mod.rs               # 导出加载管线
│   ├── definition_type.rs   # DefinitionType trait（密封 trait）
│   ├── discovery.rs         # 文件发现（目录扫描）
│   ├── errors.rs            # ConfigError / ValidationError
│   └── ron_loader.rs        # Bevy AssetLoader 实现
├── def_impls.rs             # DefinitionType trait 的具体实现（14 个类型）
└── hot_reload.rs            # 基于 mtime 的热重载系统（~750行）
```

**核心类型：`LoadedAbilitiesDefs`、`LoadedEffectDefs`、`LoadedSpellDefs` 等**

这里不是"注册中心"，而是**加载暂存区**。ContentPlugin 在启动时扫描 `assets/config/` 目录，把 RON 文件反序列化后存到各自的 `Loaded*Defs` Resource 里。每一个 Def 类型有一个独立的 `Vec<ConcreteDef> + Vec<(PathBuf, String)>` 的错误列表。

```rust
#[derive(Resource)]
pub struct LoadedSpellDefs {
    pub defs: Vec<SpellDef>,                     // 成功的加载结果
    pub errors: Vec<(PathBuf, String)>,          // 失败记录
}
```

### 2.4 三套系统关系总结

| 系统 | 位置 | 存储方式 | 用途 | 当前状态 |
|------|------|---------|------|---------|
| DefinitionRegistry | infra/registry | 16 个 typed bucket | 统一的 Def 查询入口 | 🟡 已初始化，未填充 |
| DefRegistry | core/.../runtime/registry | 单表 String→Entry | 早期原型，快速跑通 | 🟡 遗留代码 |
| Loaded*Defs | content/ | 各类型独立 Vec | 内容加载暂存 | ✅ 实际使用中 |

---

## 3. DefinitionType Trait：连接内容和注册的桥梁

内容层定义了一个 **密封（Sealed）Trait**，连接"RON 文件"和"加载后的 Def 数据"：

```rust
// content/loading/definition_type.rs
pub trait DefinitionType: sealed::Sealed + Asset + TypePath {
    const BUCKET_NAME: &'static str;     // 桶名，如 "spells"
    const EXTENSION: &'static str;       // 扩展名，如 "ron"
    
    fn from_deserialized(data: Self) -> Result<Self, ConfigError> { Ok(data) }
    fn validate(&self) -> Result<(), ValidationError>;
    fn config_dir() -> &'static str { /* ... */ }
}
```

使用 Sealed 模式保证只有 crate 内部的类型能实现它。所有实现放在 `content/def_impls.rs`（而不是各个 Def 所在的 core/ 模块），以维护依赖方向：内容层 → 核心层。

**当前实现了此 trait 的 14 个类型及校验规则：**

| 类型 | BUCKET_NAME | ID 前缀 | 额外校验 |
|------|-------------|---------|---------|
| SpellDef | `spells` | `spl_` | name_key 和 desc_key 非空 |
| CueDef | `cues` | `cue_` | ID 非空 |
| EffectDef | `effects` | `eff_` | name_key 和 desc_key 非空 |
| AbilityDef | `abilities` | `abl_` | name_key 和 desc_key 非空 |
| RuleDef | `rules` | `rule_` | name_key 和 desc_key 非空 |
| QuestDef | `quests` | `qst_` | name_key、desc_key、objectives 非空 |
| RecipeDef | `recipes` | `rcp_` | name_key、materials、output.item_id 非空 |
| ShopDef | `shops` | `shp_` | name_key、faction_id、inventory 非空 |
| TargetingDef | `targeting` | 无固定前缀 | max_targets > 0 |
| TagDefinition | `tags` | 无固定前缀 | path 非空，bit_index < 128 |
| AttributeDefinition | `attributes` | 无固定前缀 | min_value < max_value |
| SummonTemplateDef | `summon_templates` | 无固定前缀 | name_key、base_attributes 非空 |
| CampEventDef | `camp_events` | 无固定前缀 | title_key、desc_key 非空 |
| BondDef | `bonds` | 无固定前缀 | name_key、desc_key、required_members、max_level 非空 |
| EnchantmentDef | `enchantments` | 无固定前缀 | name_key、modifier_id 非空 |

---

## 4. 文件发现机制：配置文件目录结构

ContentPlugin 的发现逻辑在 `content/loading/discovery.rs`：

```
assets/config/
├── spells/              → bucket_name = "spells"
│   ├── fireball.ron
│   └── magic_missile.ron
├── effects/
│   ├── eff_000001.ron
│   └── burn.ron
├── tags/
│   ├── elemental.ron
│   └── damage_types.ron
├── cues/                → 10 个 cue 定义
├── targeting/           → 11 个目标选择定义
├── shops/               → 5 个商店定义
├── recipes/             → 6 个配方定义
├── quests/              → 6 个任务定义
├── camp_events/         → 6 个营地事件
├── bonds/               → 6 个羁绊定义
├── enchantments/        → 6 个附魔定义
├── summon_templates/    → 4 个召唤模板
├── attributes/          → 2 个属性配置
├── progression/         → 1 个成长表
└── spell_config/        → 1 个法术配置
```

**核心规则**：目录名决定 bucket_name。一个 RON 文件可以包含单条记录（大多数）或数组（tags、attributes 支持 `[item1, item2, ...]` 格式）。

**真实 RON 文件示例：**

```ron
// assets/config/spells/fireball.ron
(
    id: "spl_000001",
    name_key: "spell.fireball.name",
    desc_key: "spell.fireball.desc",
    level: L3,
    casting_time: Action,
    range: Ranged(base: 150, max: None),
    duration: Instant,
    requires_concentration: false,
    saving_throw: Some(Dexterity),
    can_upcast: true,
    effects: [],
)

// assets/config/effects/eff_000001.ron  — Damage -25
(
    id: "eff_000001",
    name_key: "effect.eff_000001.name",
    duration: Instant,
    modifiers: [
        (op: Add, target_attribute: "attr_000030",
         value: Fixed(-25.0), priority: 50),
    ],
    stacking: (stacking_type: None, max_stacks: 1, ...),
    effect_category: Damage,
    cues: [],
)

// assets/config/tags/elemental.ron
(
    id: "tag:elemental",
    path: "DamageType.Elemental",
    parent_id: None,
    bit_index: 2,
    is_abstract: true,
    namespace: Damage,
)
```

---

## 5. 加载全流程：从 RON 文件到可查询

```
Startup System: ContentPlugin.load_all_content()
═══════════════════════════════════════════════

Step 1: discover_ron_files("assets/config")
        ↓
        递归扫描所有 .ron 文件
        从目录名推断 bucket_name
        返回 Vec<ContentFile> → 存入 ContentState

Step 2: for file in discovered_files:
            match file.bucket_name:
                "spells"  → ron::from_str::<SpellDef>()
                "effects" → ron::from_str::<EffectDef>()
                "cues"    → ron::from_str::<CueDef>()
                "quests"  → ron::from_str::<QuestDef>()
                "recipes" → ron::from_str::<RecipeDef>()
                ...
                other     → info!("未知 bucket，跳过")
        ↓
        每个文件独立 try/catch：
            ✓ 成功 → push 到对应 Loaded*Defs.defs
            ✗ 失败 → push 到对应 Loaded*Defs.errors

Step 3: def.validate() 校验
        ID 格式、必填字段、数值范围
        失败 → 记录到 errors，该 Def 不进入加载结果

Step 4: 统计输出
        ContentLoadSummary 记录总数/成功/失败
```

**关键设计：加载失败不 panic**

每个 Loaded*Defs Resource 的 `errors` 向量记录所有失败。游戏继续启动，只是出错的 Def 不会被使用。启动日志会报告每个 bucket 的加载统计。

**RonAssetLoader 的作用**

除了同步加载，每个 Def 类型还注册了 Bevy AssetLoader，使它们可以在 Bevy 的 Asset 系统中原生使用：

```rust
app.init_asset::<SpellDef>()
   .init_asset_loader::<RonAssetLoader<SpellDef>>();
```

RonAssetLoader 实现了 `AssetLoader` trait，异步读取字节 → ron::de::from_bytes → 返回 Asset。这使得未来接入 Bevy 原生热重载（file_watcher）不需要改动加载逻辑。

---

## 6. 热重载：改配置不重启

### 为什么需要热重载？

策划调数值的传统方式是：改文件 → 保存 → 编译 → 重启 → 看效果。一次 30 秒，一天改 50 次就是 25 分钟浪费在等待上。

热重载让这个过程变成：改文件 → 保存 → 马上看效果。

### 当前实现：mtime 轮询

ContentPlugin 使用基于文件修改时间的轮询机制：

```
每帧：
    hot_reload_content_system
        ↓
    每 2 秒：
        1. 重新扫描 assets/config/
        2. 比较当前 mtime 与记录的 mtime
        3. 找出变更的文件
        4. 对每个变更文件：重新解析 + validate()
        5. 更新 Loaded*Defs（retain 旧记录 + push 新记录）
        6. 更新记录的 mtime
```

关键模式是 `retain + push`——热重载时不清空整个 Vec，而是只删除同 ID 的旧记录再插入新记录：

```rust
// hot_reload.rs 模式
fn reload_single_spell(spells: &mut ResMut<LoadedSpellDefs>, file: &ContentFile) -> bool {
    // ...读取、反序列化、校验...
    spells.defs.retain(|d| d.id != def.id);  // 移除旧版本
    spells.defs.push(def);                    // 插入新版本
    true
}
```

### ADR-013 规划的未来两层热重载架构

```
配置文件修改 (file_watcher 检测)
       │
       ▼
Layer 1: Asset Server 重新加载 .ron
       │
       ▼
Assets<T> 中的 Asset 被替换
       │
       ▼
on_asset_changed::<T> (Observer)
       │
       │── Registry 更新索引和版本号
       └── 触发 OnDefinitionReloaded 事件
              │
              ▼
       下游 Observer 响应重载
       (如 UI 刷新、技能重新选择)
```

### 运行时安全：快照机制

热重载有个关键问题：如果战斗中改了"火球术"的伤害值，已经在飞行途中的火球术怎么办？

**答案：Spec 层的快照（Snapshot）**

```
施法时创建 Snapshot：
  EffectSnapshot {
      caster_attributes: { "damage": 42, "spell_power": 15 },
      target_attributes: { "armor": 10 },
      snapshot_frame: current_frame,
  }

后续 Effect 使用 Snapshot 中的值执行 → 不受热重载影响
新施放的技能                          → 用新值
```

---

## 7. ID 系统：每件东西都有一个编号

所有的游戏定义（Definition）使用统一 ID 格式：

```
{类型前缀}_{6 位数字}
abl_000042     ← 技能 "火球术"
eff_000015     ← 效果 "灼烧"
```

### 完整前缀表（infra/resolver.rs）

| 前缀 | 类型 | 代码枚举 |
|------|------|---------|
| `abl_` | AbilityDef | IdType::Ability |
| `eff_` | EffectDef | IdType::Effect |
| `trg_` | TriggerDef | IdType::Trigger |
| `tag_` | Tag | IdType::Tag |
| `attr_` | Attribute | IdType::Attribute |
| `cue_` | CueDef | IdType::Cue |
| `itm_` | ItemDef | IdType::Item |
| `spl_` | SpellDef | IdType::Spell |
| `qst_` | QuestDef | IdType::Quest |
| `fct_` | FactionDef | IdType::Faction |
| `ter_` | TerrainDef | IdType::Terrain |
| `rcp_` | RecipeDef | IdType::Recipe |
| `buf_` | BuffDef | IdType::Buff |
| `ltb_`/`oot_` | LootTable | IdType::LootTable |
| `cst_` | Custom | IdType::Custom |

注意：infra/resolver.rs 中的前缀和 core runtime registry 的版本有细微差别（`oot_` vs `ltb_`），这是两个独立实现的遗留问题。

### ID 的四大规则

1. **无语义化**：ID 不表达含义。`abl_fireball` 被禁止，因为如果技能改名叫"炎爆术"，ID 就过时了
2. **永不重用**：删除的 ID 标记为 deprecated，不重新分配
3. **全局唯一**：跨桶检测，V2 校验确保不重复
4. **自动分配**：`IdAllocator` 按类型自动递增

### ID 分配器

```rust
let mut allocator = IdAllocator::new_full();
let new_id = allocator.allocate(&IdType::Ability);
// → Some("abl_000043")  假设已有 42 个技能
```

---

## 8. 校验体系：注册中心的四道防线

### 基础设施级校验（infra/resolver.rs：ValidationRunner）

Infra 层的 `ValidationRunner` 提供全面校验：

| 规则 | 名称 | 检查内容 | 代码位置 |
|------|------|---------|---------|
| V1 | ID 格式 | 前缀合法、后缀数字 | `validate_id_formats()` |
| V2 | 全局唯一 | 跨所有桶无重复 ID | `check_global_uniqueness()` |
| V6 | 条目完整性 | RegistryEntry 是否有 data | `check_entry_integrity()` |

### 运行时级校验（core/.../runtime/registry/mechanism/validator.rs）

运行时 `DefRegistry` 的校验器更简单，主要是 ID 格式和跨引用检查：

```rust
// V1: ID 格式校验
pub fn validate_id_format(def_id: &str) -> Result<(), RegistryError>;

// V3: 跨 Def 引用检查
pub fn validate_cross_references(registry: &DefRegistry) -> CrossReferenceReport;
// 扫描每个 entry 的 data 字符串
// 匹配 "xxx_NNNNNN" 模式提取引用 ID
// 检查所有引用 ID 是否在 registry 中存在
```

### 内容层校验（content/def_impls.rs：具体 validate 方法）

每个 DefinitionType 实现自己的 validate 方法，校验内容因类型而异：

```
SpellDef    → ID 前缀 "spl_"，name_key/desc_key 非空
EffectDef   → ID 前缀 "eff_"，name_key/desc_key 非空
TagDef      → path 非空，bit_index < 128
AttributeDef → min_value < max_value（数值合理性）
RecipeDef   → materials 和 output.item_id 非空
BondDef     → required_members 和 max_level 非空
...
```

### 校验错误类型（content/loading/errors.rs）

```
ConfigError:
  ├── FileReadError      文件读取失败
  ├── DeserializeError    RON 反序列化失败
  ├── ConversionError    Def 转换失败
  └── ValidationFailed   校验失败（含多个 ValidationError）

ValidationError:
  ├── EmptyId            ID 为空
  ├── InvalidIdPrefix    前缀不匹配
  ├── InvalidIdFormat    格式非法
  ├── MissingField       必填字段缺失
  ├── OutOfRange         数值越界
  ├── BrokenReference    引用不存在的 Def
  └── Custom             自定义错误
```

---

## 9. 领域专属注册表

除了中央注册系统，各个业务域还有自己的"小注册表"——专门管理该域特有的数据。

### HazardZoneRegistry（terrain 域）

```rust
#[derive(Resource)]
pub struct HazardZoneRegistry {
    pub zones: Vec<HazardZoneDef>,
}
```

地形域的陷阱/危险区域注册表，存储所有 `HazardZoneDef`（触发条件、效果、可见性）。由 HazardSystem 在运行时查询。

### EnchantmentDefRegistry（crafting 域）

```rust
#[derive(Resource)]
pub struct EnchantmentDefRegistry {
    pub defs: HashMap<String, EnchantmentDef>,
}
```

制作域的附魔定义注册表，用 `HashMap<String, EnchantmentDef>` 存储，提供 `register()` 和 `get()` 方法。内容层加载后填充此 registry。

### DialogueTreeRegistry（narrative 域）

```rust
#[derive(Resource)]
pub struct DialogueTreeRegistry {
    pub trees: HashMap<String, String>,           // tree_id → entry_node_id
    pub nodes: HashMap<String, DialogueNodeDef>,   // node_id → node
}
```

叙事域的对话树注册表。不是从 RON 文件加载的典型 Def，而是由代码或脚本构建的运行时数据结构。提供 `register_tree()`、`register_node()`、`entry_node()` 等操作。

### SpecRegistry（spec Capability）

```rust
#[derive(Resource)]
pub struct SpecRegistry {
    pub registered_defs: HashMap<String, DefEntry>,  // def_id → 元信息
    pub config: SpecRegistryConfig,
    pub next_spec_id: u64,                           // 自增计数器
}
```

Spec 层独有的注册表，管理"哪些 Def 可以生成 Spec"。这是不同于中央注册中心的另一个维度——它不存 Def 数据本身，只存 Def 的元信息（类型、最大等级）。提供 Def→Spec 的工厂方法：

```rust
pub fn create_ability_spec(&mut self, def_id: String, level: u8) -> Result<AbilitySpec, SpecError>;
pub fn create_effect_spec(&mut self, def_id: String, source: EffectSource, frame: u64) -> Result<EffectSpec, SpecError>;
```

### 领域注册表汇总

| 注册表 | 所在域 | 存储内容 | 数据来源 |
|--------|-------|---------|---------|
| `HazardZoneRegistry` | terrain | 陷阱定义 | 内容加载 / 代码注册 |
| `EnchantmentDefRegistry` | crafting | 附魔定义 | 内容加载（RON） |
| `DialogueTreeRegistry` | narrative | 对话树节点 | 代码构建 / 脚本 |
| `SpecRegistry` | spec | Def 元信息 | 手动 register() 注册 |
| `TileEntityMap` | terrain | TilePos→Entity | 运行时维护 |
| `EntityMapper<BattleUnitId>` | combat/replay | 战场单位 ID 映射 | 战斗开始构建 |

---

## 10. Spec 层：Def → Instance 的桥梁

Registry 管的是"定义"（Definition）。但游戏运行时还需要一个中间层——**Spec（配置槽位）**。

### 三层数据模型

```
Definition（定义）              Spec（配置槽位）              Instance（实例）
─────────────────              ────────────────              ──────────────
EffectDef                      EffectSpec                   ActiveEffect
（RON 配置，只读）               （引用 Def ID + 快照）        （ECS Component，可变）
```

### AbilitySpec 结构

```rust
pub struct AbilitySpec {
    pub spec_id: SpecId,                    // "spec_0000000001" 自增
    pub def_id: String,                     // 引用的 AbilityDef ID
    pub level: u8,                          // 等级 1..5
    pub max_level: u8,
    pub enhancements: Vec<EnhancementId>,   // 强化列表
    pub snapshot: EffectSnapshot,           // 施法时快照
}
```

### EffectSnapshot 保护机制

```rust
pub struct EffectSnapshot {
    pub caster_attributes: HashMap<String, f32>,
    pub target_attributes: HashMap<String, f32>,
    pub snapshot_frame: u64,                // 快照帧号
}
```

在施法时对施法者和目标的关键属性做快照，后续 Effect 执行使用快照值而不是实时值。这样热重载改了配置，也不会影响已经在执行的技能效果。

### Spec 生命周期校验

SpecRegistry 在创建 Spec 时校验四条不变量：

```
V1: Def 必须已注册 → SpecRegistry.registered_defs 中存在
V2: 等级在合法范围 → [1, max_level]
V3: 同一实体无重复 AbilitySpec → validate_no_duplicate_ability()
V4: 快照值在施法时填充 → 由外部调用方负责
```

### Spec 操作

```
grant_ability_spec()     → 授予技能 → 触发 SpecGranted 事件
remove_ability_spec()    → 移除技能 → 触发 SpecRemoved 事件
change_ability_level()   → 改变等级 → 触发 SpecLevelChanged 事件
grant_effect_spec()      → 授予效果 → 触发 SpecGranted 事件
remove_effect_spec()     → 移除效果 → 触发 SpecRemoved 事件
```

---

## 11. Pipeline 也有自己的注册中心

除了数据注册中心，管线系统也有自己的注册中心——管的是"执行流程"：

```rust
#[derive(Resource)]
pub struct PipelineRegistry {
    pipelines: HashMap<String, PipelineDefinition>,
    hooks: Vec<Box<dyn PipelineHook>>,
}
```

| Pipeline ID | 用途 | 阶段 |
|------------|------|------|
| `combat` | 战斗结算 | 命中判定→伤害计算→效果应用 |
| `ability` | 技能执行 | 目标选择→消耗→效果→冷却 |
| `modifier` | 属性修改 | 收集→聚合→应用 |

Pipeline 注册是**编译期确定的**，运行时不能改（破坏 Replay 确定性）。

---

## 12. 各业务域如何访问注册中心

### 当前模式：直接访问 Loaded*Defs Resource

```rust
fn my_system(spells: Res<LoadedSpellDefs>, effects: Res<LoadedEffectDefs>) {
    let fireball = spells.defs.iter().find(|d| d.id == "spl_000001");
    // ...
}
```

### 未来模式：通过 DefinitionRegistry 统一查询

```rust
fn my_system(registry: Res<DefinitionRegistry>) {
    let fireball: &RegistryEntry = registry.spells.get(&"spl_000001".into());
    // ...
}
```

业务域通过 `integration/` 层的 Facade 访问（ADR-046），不直接操作内部桶：

```rust
// crafting/integration/facade.rs
pub fn get_recipe(registry: &DefinitionRegistry, id: &DefinitionId) -> Option<&RegistryEntry> {
    registry.recipes.get(id)
}
```

---

## 13. 注册中心的数据流全景图

```
─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━
  Phase A: 启动加载（Startup）
─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━

assets/config/*.ron
       │
       ▼
ContentPlugin.load_all_content()
       │
       ├── 发现文件 → ContentState.discovered_files
       │
       ├── 按 bucket 分类加载（match 分支）
       │       │
       │       ├── spells   → ron::from_str::<SpellDef>()
       │       ├── effects  → ron::from_str::<EffectDef>()
       │       ├── tags     → ron::from_str::<TagDefinition>()
       │       ├── ...      → ...其他 14 个类型
       │       │
       │       └── 每个文件：读 → 反序列化 → validate()
       │               ✓ → push 到 Loaded*Defs.defs
       │               ✗ → push 到 Loaded*Defs.errors
       │
       └── 统计 → ContentLoadSummary

       └── (未来) 同时填充 DefinitionRegistry

─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━
  Phase B: 运行时
─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━

System 需要 Def 数据：
         │
         ├── Res<LoadedSpellDefs>       (当前方式)
         │        .defs.iter().find(..)
         │
         └── Res<DefinitionRegistry>    (未来方式)
                  .spells.get(&id)

System 创建 Spec（授予技能/施法）：
         │
         ├── SpecRegistry.create_ability_spec(def_id, level)
         ├── → 校验 V1（Def 已注册）、V2（等级合法）
         ├── → 创建 AbilitySpec（含快照）
         └── → 触发 SpecGranted 事件

Effect 执行：
         ├── 使用 Spec.snapshot 值
         └── 不受热重载影响

─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━
  Phase C: 热重载（Update 每帧）
─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━─━

hot_reload_content_system (每 2 秒)
       │
       ├── 扫描目录
       ├── 比较 mtime
       ├── 重新加载变更文件
       └── retain + push 到 Loaded*Defs
```

---

## 14. `register_domain_types!` 宏

每个业务域的 Plugin 使用一个通用宏来注册 Component 类型：

```rust
// narrative/plugin.rs
register_domain_types!(app, [DialogueState, StoryFlags, CutsceneState,]);
```

这个宏展开为：

```rust
app.register_type::<DialogueState>()
   .register_type::<StoryFlags>()
   .register_type::<CutsceneState>();
```

这是 Bevy Reflect 所需的类型注册，确保这些类型可以参与序列化、编辑器检查和运行时反射。虽然不是注册中心直接相关的，但它是 Def 类型和 ECS 组件注册的一个补充机制。

---

## 15. 常见问题

### Q: 为什么有三种注册体系？为什么不统一？

这是一个渐进式演进项目：
1. **早期**：用最简单的 `HashMap<String, RegistryEntry>`（DefRegistry）快速跑通能力系统
2. **中期**：设计了完整的 `DefinitionRegistry` + typed buckets + 索引 + 变更追踪
3. **当前**：内容加载层实现了独立的 Loaded*Defs Resource 体系
4. **未来**：内容层加载后直接填充 DefinitionRegistry，去掉中间状态

### Q: 新增一种 Def 类型需要改哪些文件？

```
1. 定义 Rust struct（在 core/ 的对应模块）
2. 实现 DefinitionType trait（在 content/def_impls.rs）
3. 在 ContentPlugin（content_plugin.rs）：
   a. init_asset + init_asset_loader
   b. 添加 Loaded*Defs Resource
   c. 添加 load_xxx_def() 函数
   d. 在 load_all_content 添加 match 分支
5. 在 hot_reload.rs 添加 reload_single_xxx() 函数
6. 在 hot_reload_content_system 添加 match 分支
7. 在 DefinitionRegistry（未来）添加新桶
8. 在 IdType 枚举添加新变体
9. 在 resolver.rs 添加前缀映射
10. 写入 RON 配置文件

当前约需要改 6-8 个文件，未来统一后会减少。
```

### Q: 热重载和回放（Replay）兼容吗？

兼容。关键机制：
- **快照隔离**：Effect 执行时使用的快照值不受热重载影响
- **回放确定性**：回放使用录制的命令序列，不依赖 Registry 的当前状态
- **存档**：只存 Instance 和 Persistence 层数据，不存 Registry

### Q: infra 和 core 两个 registry 的前缀为什么有差异？

infra/resolver.rs 用 `ltb_` 前缀，而 core registry 的 IdType 用 `oot_`。这是两个独立实现的遗留数据不匹配。统一时需处理。

---

## 16. 代码阅读指引

| 你想了解什么 | 读哪个文件 |
|------------|-----------|
| 注册中心的数据结构 | `src/infra/registry/registry.rs`（DefinitionRegistry + RegistryBucket） |
| ID 分配器 | `src/infra/registry/resolver.rs`（IdAllocator + AllocatorState） |
| 注册校验规则 | `src/infra/registry/resolver.rs`（ValidationRunner） |
| 注册中心插件 | `src/infra/registry/plugin.rs`（RegistryPlugin） |
| 运行时轻量 DefRegistry | `src/core/capabilities/runtime/registry/foundation/values.rs` |
| ID 类型枚举 | `src/core/capabilities/runtime/registry/foundation/types.rs` |
| 运行时校验器 | `src/core/capabilities/runtime/registry/mechanism/validator.rs` |
| 注册事件 | `src/core/capabilities/runtime/registry/events.rs` |
| 内容加载入口 | `src/content/content_plugin.rs`（ContentPlugin） |
| DefinitionType trait | `src/content/loading/definition_type.rs` |
| DefinitionType 实现 | `src/content/def_impls.rs`（14 个类型的具体校验） |
| 文件发现机制 | `src/content/loading/discovery.rs` |
| RonAssetLoader | `src/content/loading/ron_loader.rs` |
| 热重载实现 | `src/content/hot_reload.rs` |
| 加载错误类型 | `src/content/loading/errors.rs` |
| Spec 注册中心 | `src/core/capabilities/spec/mechanism/lifecycle.rs` |
| Spec 值对象 | `src/core/capabilities/spec/foundation/values.rs` |
| Pipeline 注册中心 | `src/core/capabilities/runtime/pipeline/registry.rs` |
| 地形陷阱注册表 | `src/core/domains/terrain/resources.rs` |
| 附魔注册表 | `src/core/domains/crafting/resources.rs` |
| 对话树注册表 | `src/core/domains/narrative/components.rs` |
| 战场单位注册表 | `src/core/domains/combat/integration/replay/registry.rs` |
| 数据架构规范 | `docs/04-data/infrastructure/registry_schema.md` |
| ADR 热重载架构 | `docs/01-architecture/10-capability-system/ADR-013-registry-hotreload.md` |
| ADR 内容加载管线 | `docs/01-architecture/40-cross-cutting/ADR-047-content-loading-pipeline.md` |
| 启动装配顺序 | `src/app/app_plugin.rs` |

---

## 17. 宪法对照

| 宪法 / Data Law | 注册中心如何满足 |
|----------------|----------------|
| Data Law 001: Def-Instance 分离 | Registry 只存 Def，Instance 在 ECS Component 中 |
| Data Law 003: 配置只引用 ID | 所有 Def 之间引用使用 ID，不内嵌完整结构 |
| Data Law 005: Effect 唯一执行入口 | EffectDef 通过 Registry 管理，不绕过 |
| Data Law 010: Replay 优先 | 热重载通过 Snapshot 保护，不破坏回放确定性 |
| ADR-045: 模块可见性 | Registry 查询接口公开，内部通过 Facade 访问 |
| ADR-046: integration/ 统一访问 | 业务域通过 Facade 查询 Registry |
| 编码规则: 禁止绕过 Registry | 禁止直接读 RON 文件绕过注册中心 |
