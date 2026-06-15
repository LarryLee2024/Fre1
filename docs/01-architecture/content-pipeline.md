---
id: 01-architecture.content-pipeline
title: Content Pipeline
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
---

# Content Pipeline — 数据驱动与内容架构

Version: 1.0
Status: Proposed

本文档定义 SRPG 项目的数据驱动架构和内容管线设计。
核心目标：新增 1000 技能、500 Buff、100 地图、50 章节，只改 RON 文件，不改 Rust 代码。

> **统一抽象模型**：Skill/Buff/Effect 的数据驱动抽象层级参见 `docs/01-architecture/skill-buff-abstraction.md`。本文件关注 Content 层如何将 RON 数据桥接到 Core 层的规则引擎。

交叉引用：
- `docs/AI开发宪法完整版.md` — AI 开发宪法（最高约束力），本文档对应条款：1.1.2（定义与实例分离）、1.1.3（规则与内容分离）、1.1.5（数据驱动优先）、12.1.1-12.1.6（数据驱动核心）、12.2.1（Schema）、12.3.1（唯一事实源）

---

## 核心原则

### Rule / Content 分离（宪法 §1.1.3）

```
Rule  = Rust 代码  →  游戏规则引擎（怎么算伤害、怎么判定Buff、怎么执行效果）
Content = RON 文件  →  游戏内容数据（火球术技能、剧毒Buff、骑士职业）
```

🟥 **绝对禁止**：新增内容时修改 Rust 代码。

### Definition / Instance 分离（宪法 §1.1.2）

```
Definition = 不可变配置  →  SkillDef, BuffDef, EquipmentDef, ItemDef, UnitTemplate
Instance   = 运行时状态  →  SkillSlots, ActiveBuffs, EquipmentSlots, ItemInstance, Unit
```

🟥 **绝对禁止**：运行时修改 Definition 中的任何字段。

> **优化来源**: `docs/其他/74借鉴.md` §6, §11

### Definition 即数据资产（宪法 §1.1.2 — Godot Resource / Unity ScriptableObject）

Definition 不是 ECS Component，而是**可序列化的数据资产**。这与游戏引擎中的成熟模式完全对应：

| 引擎概念 | 等价物 | 本质 |
|----------|--------|------|
| Godot `Resource`（`.tres`/`.res`） | `SkillDef`/`BuffDef`/`CharacterDef` | 可序列化的数据资产，不参与 ECS 查询 |
| Unity `ScriptableObject` | 同上 | 资产容器，策划可直接编辑 |
| Bevy RON 配置 + `AssetLoader` | `SkillAssetLoader` + `content/skills/*.ron` | 数据资产的 Bevy 落地形态 |

**核心规则**：

🟥 **不要什么都 `spawn(Entity)` 加 Component — 应该先定义 Definition，然后从 Definition 生成 Instance。**

```
策划编写 RON → XxxDef（数据资产） → Registry（全局注册表） → 从 Definition 生成 Instance（运行时 Entity）
```

定义态（Definition）在内容管线中创建和加载，实例态（Instance）在运行时由 System 从 Registry 中查询并生成。两者之间永远隔着 Registry，不直接引用。

### Scene 模式（宪法 §3.0.2 — Godot Scene 思想）

> **优化来源**: `docs/其他/74借鉴.md` §7

大型 SRPG 项目应借鉴 Godot 的 Scene 组织方式。不要什么都 `spawn(Entity)` 然后散落全局，而是将相关 Entity 组织为独立的场景/Plugin：

```
BattleScene  → 独立 Plugin，管理战斗相关的所有 Entity 生命周期
TownScene    → 独立 Plugin，管理城镇界面的 Entity 生命周期
WorldMapScene → 独立 Plugin，管理大地图的 Entity 生命周期
```

**规则**：
- 🟩 每个 Scene 是独立的 Bevy Plugin，拥有自己的 Entity 生命周期
- 🟩 Scene 切换时清理该 Scene 的所有 Entity 和 Strong Handle
- 🟩 新内容通过 Plugin 合并注册，不散落在全局
- 🟥 禁止跨 Scene 直接引用 Entity（必须通过 ID 查询）

> **ScriptableObject 模式的 Bevy 实现**（§11 补充）

Definition = Unity ScriptableObject = Godot Resource = Bevy RON config。在 Bevy 中的落地形态为：

```rust
// 1. RON 配置文件（策划编辑）
// content/skills/fireball.ron

// 2. AssetLoader（运行时加载）
pub struct SkillAssetLoader;

// 3. Registry（全局注册表，查询入口）
pub struct SkillRegistry {
    skills: HashMap<String, SkillData>,
}
```

🟥 **不要用 `const FIREBALL` 硬编码技能数据**。所有游戏内容必须通过 Definition → Registry 管线。

---

## 数据流架构

### 完整数据流

```
content/*.ron
    ↓  [AssetServer 加载]
XxxDef (RON 反序列化类型，TagName 字符串)
    ↓  [impl From<XxxDef> for XxxData]
XxxData (运行时类型，GameplayTag 位掩码)
    ↓  [Registry.insert()]
XxxRegistry (全局注册表，不可变)
    ↓  [System 查询]
运行时业务逻辑
```

### 数据流分层

```
┌────────────────┐
│  content/      │  ← RON 配置文件（策划可编辑）
│  *.ron         │
└───────┬────────┘
        │ Bevy AssetServer 加载
        ↓
┌────────────────┐
│  XxxDef        │  ← RON 反序列化类型
│  (TagName)     │     使用字符串标签
└───────┬────────┘
        │ impl From<XxxDef> for XxxData
        ↓
┌────────────────┐
│  XxxData       │  ← 运行时类型
│  (GameplayTag) │     使用位掩码标签
└───────┬────────┘
        │ Registry.insert()
        ↓
┌────────────────┐
│  XxxRegistry   │  ← 全局注册表 (Resource)
│  （不可变）    │     加载后不再修改
└───────┬────────┘
        │ System 查询
        ↓
┌────────────────┐
│  运行时业务    │  ← ECS Component / System
│  (Instance)    │     可变运行时状态
└────────────────┘
```

---

## Content vs Core 判定标准

### 核心区分

> **Skill 是 Core，Fireball 是 Content。**

| 概念 | 层级 | 位置 | 说明 |
|------|------|------|------|
| 技能规则引擎 | Core | `src/core/skill/` | 怎么释放技能、冷却怎么算 |
| 火球术数据 | Content | `content/skills/fireball.ron` | 伤害数值、范围、标签 |
| Buff 规则引擎 | Core | `src/core/buff/` | 怎么施加Buff、回合结算 |
| 剧毒数据 | Content | `content/buffs/poison.ron` | 持续伤害、回合数 |
| 装备规则引擎 | Core | `src/core/equipment/` | 怎么穿脱装备、需求检查 |
| 铁剑数据 | Content | `content/equipments/iron_sword.ron` | 属性加成、装备需求 |
| 回合规则 | Core | `src/core/turn/` | 怎么管理回合、行动顺序 |
| 效果管线 | Core | `src/core/battle/pipeline/` | 怎么生成→修饰→执行 |
| 地图规则 | Core | `src/core/map/` | 怎么寻路、怎么占位 |
| 第三关配置 | Content | `content/stages/stage_03.ron` | 地图大小、敌人配置、胜负条件 |

### 一句话总结

> **Core 回答"怎么做"，Content 回答"是什么"。**

---

## RON 配置规范

### 通用配置结构

每个 RON 文件遵循以下结构：

```ron
(
    // 元数据
    id: "fireball",
    name: "火球术",
    description: "对目标区域造成火焰伤害",
    
    // 版本控制
    version: 1,
    
    // 标签
    tags: ["magic", "fire", "aoe"],
    
    // 业务数据
    // ...
)
```

### 配置引用完整性

🟩 所有配置引用必须自动校验：

```ron
// content/skills/fireball.ron
(
    id: "fireball",
    // 这里的 effect_id 必须指向一个真实存在的 effect
    effect_ids: ["direct_damage"],
    // 这里的 buff_id 必须指向一个真实存在的 buff
    buff_ids: ["burning"],
)
```

配置加载时必须校验：
- `effect_ids` 中的每个 ID 在 `EffectRegistry` 中存在
- `buff_ids` 中的每个 ID 在 `BuffRegistry` 中存在
- 引用失败 → 加载时 `warn!` 并跳过或使用默认值

### 配置兼容性

🟩 配置的向后兼容性优先于格式优雅性：

- 新增字段必须有默认值
- 删除字段必须有版本迁移脚本
- 配置版本号必须递增

---

## Content 目录与 Core 模块对应关系

| Content 目录 | Core 模块 | 说明 |
|--------------|-----------|------|
| `content/skills/` | `src/core/skill/` | 技能数据 ↔ 技能规则 |
| `content/buffs/` | `src/core/buff/` | ⚠️ 已废弃（吸收为 Effect + Duration），参见 ADR-026 |
| `content/effects/` | `src/core/effect/` | 效果数据 ↔ 效果规则 |
| `content/formulas/` | `src/core/formula/` | 公式数据 ↔ 公式引擎 |
| `content/classes/` | `src/core/character/` | 职业数据 ↔ 角色规则 |
| `content/characters/` | `src/core/character/` | 角色数据 ↔ 角色规则 |
| `content/enemies/` | `src/core/character/` | 敌人数据 ↔ 角色规则 |
| `content/executions/` | `src/core/execution/` | **新增** 执行算式数据 ↔ Execution 算式引擎 |
| `content/cues/` | `src/core/cue/` | **新增** 表现信号配置 ↔ Cue 事件路由 |
| `content/items/` | `src/core/inventory/` | 物品数据 ↔ 背包规则 |
| `content/equipments/` | `src/core/equipment/` | 装备数据 ↔ 装备规则 |
| `content/quests/` | `src/core/quest/` | 任务数据 ↔ 任务规则 |
| `content/dialogues/` | `src/core/dialogue/` | 对话数据 ↔ 对话规则 |
| `content/stages/` | `src/core/stage/` | 关卡数据 ↔ 关卡规则 |
| `content/terrains/` | `src/core/terrain/` | 地形数据 ↔ 地形规则 |
| `content/ai_behaviors/` | `src/core/ai/` | AI行为数据 ↔ AI规则 |
| `content/factions/` | `src/core/faction/` | 阵营数据 ↔ 阵营规则 |
| `content/loot_tables/` | `src/core/loot/` | 掉落数据 ↔ 掉落规则 |
| `content/shops/` | `src/core/economy/` | 商店数据 ↔ 经济规则 |
| `content/campaigns/` | `src/core/campaign/` | 战役数据 ↔ 战役规则 |
| `content/chapters/` | `src/core/chapter/` | 章芽数据 ↔ 章节规则 |
| `content/achievements/` | `src/core/achievement/` | 成就数据 ↔ 成就规则 |

> **优化来源**：`docs/其他/47.md` — 大规模内容的性能优化：HashMap vs BTreeMap、懒加载、内存预算

### 大规模内容性能优化

目标是支撑"1000 技能、500 Buff、100 地图"，需要考虑以下性能维度：

**Registry 数据结构选择**：

| 数据结构 | 适用场景 | 优势 | 劣势 |
|---------|---------|------|------|
| `HashMap<K, V>` | 频繁随机查找 | O(1) 查找 | 无序，内存开销大 |
| `BTreeMap<K, V>` | 需要有序遍历 | 有序，范围查询 | O(log n) 查找 |
| `Vec<V>` + 索引 | 固定 ID 集合 | 连续内存，缓存友好 | 不支持动态增删 |

**推荐**：Registry 使用 `HashMap`（随机查找为主），配置热重载时使用 `BTreeMap` 临时存储变更。

**懒加载（Lazy Loading）**：
- 关卡配置按需加载：只在进入关卡时加载 `StageConfig`
- AI 行为按需加载：只在战斗开始时加载 `AiBehaviorConfig`
- 图标/特效按需加载：UI 显示时才加载资源

**内存预算**：
- 1000 技能 × 平均 2KB/个 ≈ 2MB（配置数据）
- 500 Buff × 平均 1KB/个 ≈ 0.5MB
- 100 地图 × 平均 10KB/个 ≈ 1MB
- 总计：~3.5MB 配置数据（可接受）

### 策划友好性

> **优化来源**：`docs/其他/47.md` — RON 对非技术背景策划有学习成本，需要配套工具降低门槛

RON 格式虽然人类可读，但对非技术背景的策划仍有学习成本。配套工具建议：

**可视化配置工具**：
- 技能编辑器：拖拽式配置技能效果、范围、冷却
- Buff 编辑器：可视化配置 Buff 层数、触发条件、效果
- 关卡编辑器：集成 Tiled 地图编辑器，自动生成 RON 配置

**配置模板**：
- 为每种配置类型提供 `.ron.template` 文件
- 策划只需填写数值，不需要了解 RON 语法
- 模板文件包含详细注释说明每个字段的含义和取值范围

**自动生成**：
- 从 Excel/CSV 表格自动生成 RON 配置
- CI 流程中集成配置校验，防止错误配置提交

---

## Content 层代码架构

### Content Plugin 加载流程

```rust
// src/content/content_plugin.rs
pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        app
            // Phase 1: 加载所有 Definition 数据
            .add_plugins(SkillContentPlugin)
            .add_plugins(BuffContentPlugin)
            .add_plugins(ClassContentPlugin)
            .add_plugins(CharacterContentPlugin)
            .add_plugins(EquipmentContentPlugin)
            .add_plugins(ItemContentPlugin)
            .add_plugins(StageContentPlugin)
            .add_plugins(AiBehaviorContentPlugin)
            .add_plugins(TerrainContentPlugin)
            // Phase 2: 校验所有引用完整性
            .add_systems(OnEnter(AppState::InGame), validate_all_references)
            // Phase 3: 注册完成通知
            .add_systems(OnEnter(AppState::InGame), content_loaded_notification);
    }
}
```

### 每个 Content 模块的结构

```rust
// src/content/skills/mod.rs
pub mod skill_content;
pub mod skill_content_plugin;

// src/content/skills/skill_content.rs
use bevy::prelude::*;
use crate::core::skill::skill_def::SkillDef;
use crate::core::skill::skill_data::SkillData;
use crate::core::skill::skill_registry::SkillRegistry;
use crate::infrastructure::assets::ron_loader::RonLoader;

pub fn load_skills(
    asset_server: Res<AssetServer>,
    mut registry: ResMut<SkillRegistry>,
) {
    let skill_paths = discover_ron_files("content/skills/");
    for path in skill_paths {
        let def: SkillDef = asset_server.load(&path);
        let data: SkillData = def.into();
        registry.insert(data.id.clone(), data);
    }
}
```

> **优化来源**：`docs/其他/47.md` — 致命修复：AssetServer::load() 返回 Handle<T> 而非 T，必须使用 AssetEvent 响应式管线

### ⚠️ 致命异步时序错误修正

🟥 **上述 `load_skills` 代码示例存在严重的异步时序错误，必须立即修正。**

`AssetServer::load()` 是非阻塞异步的，它立即返回 `Handle<T>`，此时文件还在后台 IO 线程中读取。你无法在调用 `load()` 的同一帧内拿到 `SkillDef` 的数据并执行 `.into()`。

**正确做法 — 基于 AssetLoader + AssetEvent 的响应式管线**：

```rust
// Step 1: 编写自定义 AssetLoader，在 Loader 内部完成 RON 解析
pub struct SkillAssetLoader;

impl AssetLoader for SkillAssetLoader {
    type Asset = SkillData;
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &str,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let raw: RawSkillDef = ron::de::from_bytes(&bytes)?;
        // 在 Loader 内部完成 Raw → Baked 转换
        let data = SkillData::from_raw(raw)?;
        Ok(data)
    }
}

// Step 2: 使用 load_folder 批量加载
fn init_skills(asset_server: Res<AssetServer>) {
    asset_server.load_folder("content/skills");
}

// Step 3: 监听 AssetEvent 填充 Registry
fn sync_skill_registry(
    mut events: EventReader<AssetEvent<SkillData>>,
    skills: Res<Assets<SkillData>>,
    mut registry: ResMut<SkillRegistry>,
) {
    for event in events.read() {
        if let AssetEvent::Added { id } | AssetEvent::Modified { id } = event {
            if let Some(data) = skills.get(*id) {
                registry.insert(data.id.clone(), data.clone());
            }
        }
    }
}
```

### 加载进度屏障（Loading Barrier）

> **优化来源**：`docs/其他/47.md` — validate_all_references 必须在所有 Asset 完全加载后执行，否则会因引用缺失报满屏 Error

🟥 **禁止在 `OnEnter(AppState::InGame)` 时立即执行 `validate_all_references`**。

当状态切换触发时，AssetServer 可能还在后台加载那 1000 个技能文件。此时执行校验会因大量引用找不到而报出满屏 Error，甚至导致游戏判定加载失败。

**正确做法 — 基于 LoadingProgress 的屏障机制**：

```rust
#[derive(Resource, Default)]
pub struct LoadingProgress {
    pub total: usize,
    pub loaded: usize,
    pub validated: bool,
}

// 在 AppState::Loading 阶段触发所有 load_folder
// 使用 AssetServer::is_loaded_with_dependencies() 轮询进度
// 只有当 loaded >= total 时，才允许状态机切换到 AppState::InGame
fn check_loading_complete(
    progress: Res<LoadingProgress>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if progress.loaded >= progress.total && !progress.validated {
        // 所有 Asset 已加载，执行校验
        validate_all_references();
        next_state.set(AppState::InGame);
    }
}
```

---

## 热重载架构

### 热重载流程

```
开发者修改 content/skills/fireball.ron
    ↓  [文件监视器检测变更]
AssetServer 触发 AssetEvent::Modified
    ↓  [Content Hot Reload System]
重新加载 SkillDef → 转换为 SkillData → 更新 Registry
    ↓  [领域事件：SkillDataReloaded]
UI 刷新技能面板
```

### 热重载约束（宪法 §12.1.5 — 热重载优先）

- 🟩 所有配置必须优先支持热重载
- 🟩 高频修改的资源必须优先支持热重载
- 🟩 热重载不得影响运行中的战斗实例
- 🟥 热重载禁止修改 Instance 数据

### 热重载安全机制

```rust
// 热重载只更新 Definition（Registry），不更新 Instance
fn on_skill_def_modified(
    event: Res<Events<AssetEvent<SkillDef>>>,
    mut registry: ResMut<SkillRegistry>,
) {
    for event in event.iter() {
        match event {
            AssetEvent::Modified { handle } => {
                // 只更新 Registry（不可变配置）
                // 不更新已有的 SkillInstance（运行时状态）
                registry.update(handle);
            }
            _ => {}
        }
    }
}
```

---

## 配置校验管线

### 三级校验

```
Level 1: Schema 校验（加载时）
    - RON 语法正确性
    - 必填字段存在性
    - 类型正确性

Level 2: 引用校验（加载后）
    - ID 引用存在性
    - 标签引用存在性
    - 循环依赖检测

Level 3: 规则校验（运行时）
    - 游戏规则一致性
    - 数值平衡合理性
    - 逻辑矛盾检测
```

### 校验时机

- Level 1：Content Plugin 加载时
- Level 2：所有 Content 加载完成后（`validate_all_references` system）
- Level 3：开发工具运行时（`tools/data_validator/`）

---

## MOD 内容覆盖架构

详见 `docs/01-architecture/modding-design.md`。

### 统一注册中心（Unified Registry）

> **优化来源**：`docs/其他/74借鉴.md` §23 — 所有内容统一注册（SkillRegistry/BuffRegistry/QuestRegistry/CharacterRegistry 都是同一注册模式）

大型 SRPG 项目的 Registry 不应各自为政，而应遵循**统一注册模式**。所有内容类型的 Registry 共享相同的加载、查询、校验接口。

#### 统一 Registry 模式

| Registry | 内容类型 | Key 类型 | Value 类型 | 说明 |
|----------|---------|----------|-----------|------|
| `SkillRegistry` | 技能 | `SkillId` | `SkillData` | 所有技能定义 |
| `BuffRegistry` | Buff | `BuffId` | `BuffData` | 所有 Buff 定义 |
| `CharacterRegistry` | 角色 | `CharacterId` | `CharacterData` | 所有角色模板 |
| `QuestRegistry` | 任务 | `QuestId` | `QuestData` | 所有任务定义 |
| `FormulaRegistry` | 公式 | `FormulaId` | `FormulaData` | 所有数值公式 |
| `EquipmentRegistry` | 装备 | `EquipmentId` | `EquipmentData` | 所有装备定义 |
| `TerrainRegistry` | 地形 | `TerrainId` | `TerrainData` | 所有地形定义 |

#### 统一 Registry trait

```rust
/// 统一注册中心 trait — 所有 Registry 共享的接口
pub trait ContentRegistry: Send + Sync + 'static {
    type Id: Eq + Hash + Clone + std::fmt::Display;
    type Data: Clone;
    
    /// 注册内容
    fn insert(&mut self, id: Self::Id, data: Self::Data);
    
    /// 查询内容
    fn get(&self, id: &Self::Id) -> Option<&Self::Data>;
    
    /// 检查内容是否存在
    fn contains(&self, id: &Self::Id) -> bool;
    
    /// 获取所有已注册 ID（用于启动时校验）
    fn all_ids(&self) -> Vec<&Self::Id>;
    
    /// 获取已注册数量
    fn count(&self) -> usize;
}
```

#### Registry 统一加载流程

```
content/*.ron
    ↓  AssetServer 异步加载
XxxDef（RON 反序列化）
    ↓  AssetLoader 内部转换
XxxData（运行时类型）
    ↓  统一注册
XxxRegistry（Resource）
    ↓  启动时校验（参见 validation_rules.md §10）
所有引用完整性通过
```

#### 为什么需要统一 Registry 模式

独立开发者后期最痛苦的问题之一：**不知道引用是否完整**。当技能数量超过 100 时，人工检查"这个技能引用的 Buff 是否存在"变得不可能。统一 Registry 模式的价值：

1. **启动时校验**：遍历所有 Registry 交叉校验引用（参见 `validation_rules.md` §10）
2. **热重载**：统一的热重载机制，修改任何 RON 文件自动更新对应 Registry
3. **调试工具**：bevy-inspector-egui 可以统一展示所有 Registry 的内容
4. **MOD 支持**：MOD 内容通过相同接口注册到 Registry，基础内容自动被覆盖

> 交叉引用：`docs/01-architecture/validation_rules.md` §10（启动时校验）、`docs/01-architecture/skill-buff-abstraction.md` §6（Content/Rule 映射表）

核心原则：
- MOD 内容通过 `mods/xxx/content/` 目录提供
- MOD 内容优先于基础内容（后加载覆盖先加载）
- MOD 内容必须通过同样的校验管线

> **优化来源**：`docs/其他/47.md` — MOD 内容必须通过与基础内容相同的三级校验（Schema → Reference → Rule）

### MOD 内容校验要求

🟥 **MOD 内容必须通过与基础内容完全相同的三级校验管线**，无任何豁免：

```
MOD RON 文件
    ↓  Level 1: Schema 校验（RON 语法、必填字段、类型）
    ↓  Level 2: 引用校验（ID 存在性、循环依赖）
    ↓  Level 3: 规则校验（数值平衡、逻辑一致性）
校验通过 → 内容可用
校验失败 → 报告错误，MOD 内容不加载
```

**MOD 特殊校验规则**：
- MOD 新增的配置 ID 必须带 MOD 前缀（如 `mod_xxx:skill:fireball`）
- MOD 覆盖基础内容时，只允许修改数值字段，不允许改变字段类型或语义
- MOD 引用基础内容的 ID 时，必须确保基础内容已加载

### Registry 并发安全

> **优化来源**：`docs/其他/47.md` — 热重载时 Registry 的并发读写需要原子替换机制

🟥 **热重载期间直接修改 Registry 会导致数据竞争**。

```rust
// 推荐方案：使用 Bevy 的 Schedule 错开读写
// 所有读取 Registry 的业务 System 在 Update 阶段运行
// Registry 更新在 PreUpdate 阶段执行
fn hot_reload_registry(
    mut registry: ResMut<SkillRegistry>,
    // ...
) {
    // 在 PreUpdate 阶段执行，确保 Update 阶段的 System 不会同时读取
}
```

---

## 新增内容流程规范

### 新增技能（示例）

1. 创建 `content/skills/new_skill.ron`
2. 在 RON 中定义技能属性
3. 确保所有引用的 `effect_ids`、`buff_ids` 存在
4. 运行游戏，AssetServer 自动加载
5. 技能立即可用

🟥 **禁止**：为了新增技能修改 `src/core/skill/` 中的任何代码。

### 新增职业（示例）

1. 创建 `content/classes/new_class.ron`
2. 在 RON 中定义职业属性、技能池、Trait 集合
3. 创建关联的 `content/characters/new_class_unit.ron`
4. 确保所有引用存在
5. 运行游戏，新职业立即可用

🟥 **禁止**：为了新增职业修改 Rust 代码。

### 新增章节（示例）

1. 创建 `content/campaigns/new_campaign.ron`
2. 创建 `content/chapters/new_chapter_01.ron`
3. 创建 `content/stages/new_stage_01.ron`
4. 确保地图资源存在于 `assets/art/maps/battle_maps/`
5. 确保敌人模板存在于 `content/enemies/`

🟥 **禁止**：为了新增章节修改 Rust 代码。