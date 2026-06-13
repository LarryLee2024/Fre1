# Content Pipeline — 数据驱动与内容架构

Version: 1.0
Status: Proposed

本文档定义 SRPG 项目的数据驱动架构和内容管线设计。
核心目标：新增 1000 技能、500 Buff、100 地图、50 章节，只改 RON 文件，不改 Rust 代码。

> **统一抽象模型**：Skill/Buff/Effect 的数据驱动抽象层级参见 `docs/architecture/skill-buff-abstraction.md`。本文件关注 Content 层如何将 RON 数据桥接到 Core 层的规则引擎。

---

## 核心原则

### Rule / Content 分离

```
Rule  = Rust 代码  →  游戏规则引擎（怎么算伤害、怎么判定Buff、怎么执行效果）
Content = RON 文件  →  游戏内容数据（火球术技能、剧毒Buff、骑士职业）
```

🟥 **绝对禁止**：新增内容时修改 Rust 代码。

### Definition / Instance 分离

```
Definition = 不可变配置  →  SkillDef, BuffDef, EquipmentDef, ItemDef, UnitTemplate
Instance   = 运行时状态  →  SkillSlots, ActiveBuffs, EquipmentSlots, ItemInstance, Unit
```

🟥 **绝对禁止**：运行时修改 Definition 中的任何字段。

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
| `content/buffs/` | `src/core/buff/` | Buff数据 ↔ Buff规则 |
| `content/effects/` | `src/core/effect/` | 效果数据 ↔ 效果规则 |
| `content/formulas/` | `src/core/formula/` | 公式数据 ↔ 公式引擎 |
| `content/classes/` | `src/core/character/` | 职业数据 ↔ 角色规则 |
| `content/characters/` | `src/core/character/` | 角色数据 ↔ 角色规则 |
| `content/enemies/` | `src/core/character/` | 敌人数据 ↔ 角色规则 |
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

### 热重载约束

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

详见 `docs/architecture/modding-design.md`。

核心原则：
- MOD 内容通过 `mods/xxx/content/` 目录提供
- MOD 内容优先于基础内容（后加载覆盖先加载）
- MOD 内容必须通过同样的校验管线

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