---
id: 03-content.definitions.buff-def
title: BuffDef — Buff Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# BuffDef — Buff Content Def 定义

> **Content Layer**: L1 Capability | **领域规则**: `docs/02-domain/capabilities/effect_domain.md` + `docs/02-domain/capabilities/stacking_domain.md` | **数据 Schema**: `docs/04-data/capabilities/effect_schema.md` (EffectDef 部分) | **插件代码**: `src/content/plugins/buff_plugin.rs`

---

## 1. Overview

BuffDef 是**持久状态的 Effect 容器**——它将一个 EffectDef 包装为可以在目标实体上持续管理的独立状态。BuffDef 与 EffectDef 的核心区别：

- **EffectDef** 定义效果本身（做什么、持续多久、如何计算），它是"原子效果"
- **BuffDef** 定义持久化效果的状态管理方式（分类、UI 显示、免疫规则、可驱散性），它是"持久状态的 Effect 容器"

### 哲学依据

将 BuffDef 与 EffectDef 分离的设计动机：

1. **复用 EffectDef**：同一个 `eff:burn` EffectDef 可以被 `buff:burning`（怪物燃烧）、`buff:fire_weapon`（武器附魔）、`ability:fireball`（火球术直接引用）共享
2. **明确状态边界**：BuffDef 管理的是"持续存在于实体上的状态"，而 EffectDef 管理的是"效果逻辑本身"
3. **UI 与分类**：Buff/Debuff/Control 的分类、是否在 UI 显示、排序优先级等展示层信息归属 BuffDef，不污染 EffectDef

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `effect_domain.md` | Buff 的生命周期管理、与 Stacking 的交互 |
| `stacking_domain.md` | Buff 的堆叠规则（同 Buff 多层叠加行为） |
| `effect_schema.md` | EffectDef 的数据结构（BuffDef 包装的对象） |
| `effect-def.md` | 本 Def 的 `effect_def_id` 引用的 EffectDef |
| `condition-def.md` | 本 Def 的 `condition` 字段 |
| `stacking-def.md` | 本 Def 的 `stacking_def` 引用 |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// Buff Def 定义——持久状态的 Effect 容器。
///
/// BuffDef 包装一个 EffectDef，添加 Buff/Debuff 分类、UI 显示规则、免疫规则、
/// 可驱散性、显示优先级等持久状态管理所需的元数据。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct BuffDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID
    pub id: BuffId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号（用于未来迁移兼容）
    pub schema_version: u32,

    // ── 核心引用 ──
    /// 被包装的 EffectDef（Buff 的所有效果逻辑由此 EffectDef 定义）
    pub effect_def_id: EffectDefId,

    // ── Buff 专属元数据 ──
    /// Buff 分类（Buff/Debuff/Control/Other）
    pub category: StatusCategoryId,

    /// Buff 类型——用于过滤和分类（可选）
    pub buff_type: Option<BuffType>,

    /// 是否为常驻状态（不可驱散、不可移除）
    pub permanent: bool,

    /// 是否在 UI 中可见
    pub visible: bool,

    /// 是否可被驱散
    pub dispellable: bool,

    /// 显示优先级（UI 排序用，越大越靠前）
    pub display_priority: u8,

    /// Buff 图标 Key
    pub icon_key: Option<String>,

    // ── 持续时间与周期 ──
    /// 持续回合数（覆盖 EffectDef 中的 duration，可选）
    pub duration_turns: Option<u32>,

    /// 周期 Tick 配置（覆盖 EffectDef 中的 period，可选）
    pub period: Option<EffectPeriod>,

    // ── 堆叠 ──
    /// 堆叠配置（覆盖 EffectDef 中的 stacking，可选）
    pub stacking: Option<StackingConfig>,

    /// 可选的 StackingDef 引用
    pub stacking_def: Option<StackingDefId>,

    // ── 条件与标签 ──
    /// 激活条件（可选，条件满足时 Buff 生效，不满足时暂停）
    pub condition: Option<ConditionDefId>,

    /// 持有此 Buff 期间授予实体的标签
    pub granted_tags: Vec<TagId>,

    /// 持有此 Buff 期间要求的标签（不满足时自动暂停）
    pub required_tags: Option<Vec<TagId>>,

    // ── 免疫与冲突 ──
    /// 免疫此 Buff 的标签条件（目标拥有其中任一标签则不可应用）
    pub immunity_tags: Vec<TagId>,

    /// 与此 Buff 互斥的其他 BuffId（应用此 Buff 时自动移除）
    pub exclusive_with: Vec<BuffId>,

    // ── 表现 ──
    /// Buff 图标在 UI 中的表现绑定
    pub cues: Vec<CueBinding>,
}

/// Buff 类型枚举
#[derive(Deserialize, Clone, Debug)]
pub enum BuffType {
    /// 增益（增强属性、给予能力）
    Buff,
    /// 减益（削弱属性、限制行动）
    Debuff,
    /// 控制（眩晕、魅惑、恐惧等）
    Control,
    /// 地形效果
    Terrain,
    /// 其他
    Other,
}
```

### 字段说明

- **`effect_def_id`**: BuffDef 的核心——所有效果逻辑由引用的 EffectDef 定义。BuffDef 本身不包含 Modifier、Execution、Stacking 等效果逻辑字段，全部委托给 EffectDef
- **`category`**: 使用 L0 的 `StatusCategoryDef` 进行分类，支持未来自定义类别扩展（而非固定枚举）
- **`permanent`**: 常驻 Buff（如种族特质）不可被驱散或移除，只可被替换或禁用
- **`duration_turns` / `period`**: 可选覆盖 EffectDef 中的默认值。例如 `eff:burn` 默认持续 3 回合，但 `buff:inferno_burn` 可覆盖为 5 回合
- **`immunity_tags`**: 目标拥有任意免疫标签时，此 Buff 不可被应用。这是免疫系统的第一道防线
- **`exclusive_with`**: Buff 互斥列表，用于处理"不能同时拥有两个护盾"等规则

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// BuffDef 注册插件
pub struct BuffDefPlugin;

impl Plugin for BuffDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<BuffDef>();
        app.init_asset_loader::<RonAssetLoader<BuffDef>>();
        app.insert_resource(DefRegistry::<BuffDef>::new());

        app.add_systems(
            PreUpdate,
            load_buff_defs
                .run_if(resource_changed::<Assets<BuffDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按 ID 查找 BuffDef
pub fn get_buff_def(buff_id: &BuffId, registry: &DefRegistry<BuffDef>) -> Option<&BuffDef> {
    registry.get(buff_id)
}

/// 按标签过滤 BuffDef
pub fn get_buffs_by_tag(tag_id: &TagId, registry: &DefRegistry<BuffDef>) -> Vec<&BuffDef> {
    registry.iter()
        .filter(|def| def.granted_tags.contains(tag_id))
        .collect()
}
```

### 注册生命周期

```
Load (buffs.ron) → Deserialize → Validate → Register (DefRegistry<BuffDef>) → Freeze
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | BuffId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | `display_priority` 范围 | 0-100，默认 50 |
| V4 | `duration_turns >= 1` (如果设置) | 持续回合数至少为 1 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V5 | `effect_def_id` 必须已注册 | 在 DefRegistry<EffectDef> 中存在 |
| V6 | `category` 必须已注册 | 在 DefRegistry<StatusCategoryDef> 中存在 |
| V7 | `condition` (如果设置) 必须已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V8 | `stacking_def` (如果设置) 必须已注册 | 在 DefRegistry<StackingDef> 中存在 |
| V9 | `granted_tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V10 | `required_tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V11 | `immunity_tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V12 | `exclusive_with` 中的每个 BuffId 已注册 | 在 DefRegistry<BuffDef> 中存在（允许引用的 BuffDef 尚未加载，由拓扑排序保证顺序） |
| V13 | `cues[].cue_def_id` 已注册 | 在 DefRegistry<CueDef> 中存在 |

### 4.3 逻辑冲突校验

| # | 规则 | 说明 |
|---|------|------|
| V14 | `effect_def_id` 不得引用自身 | 自引用无意义 |
| V15 | `exclusive_with` 不得包含自身 | 互斥列表包含自身表示配置错误 |
| V16 | `permanent` 与 `dispellable` 互斥 | 常驻 Buff 不可被驱散 |
| V17 | BuffDef 依赖图不得形成循环 | exclusive_with 的间接依赖不能成环 |
| V18 | BuffDef 不得引用任何 L2+ Def | L1 内容不可引用 Entity/Gameplay/World 层内容 |

### 4.4 EffectDef 一致性校验

| # | 规则 | 说明 |
|---|------|------|
| V19 | 引用的 EffectDef 的 `duration` 不能为 `Instant` | Buff 必须基于持续效果 |
| V20 | 引用的 EffectDef 的 `effect_category` 与 `buff_type` 语义一致 | 如 `buff_type: Debuff` 不应引用 `effect_category: Heal` |

---

## 5. RON 示例

```ron
// BuffDef 示例：燃烧状态
//
// 这是一个可持续管理的减益状态，包装 eff:burn EffectDef，
// 添加了 Buff 专有的 UI 展示规则、免疫标签和互斥关系。
(
    id: "buff:burning",
    name_key: "buff.burning.name",
    description_key: "buff.burning.desc",
    schema_version: 1,

    // 引用已注册的 EffectDef
    effect_def_id: "eff:burn",

    // 分类为 Debuff
    category: "status:debuff",
    buff_type: Some(Debuff),

    permanent: false,
    visible: true,
    dispellable: true,
    display_priority: 60,
    icon_key: Some("icon_burning"),

    // 覆盖持续时间为 5 回合（effect_def 默认是 3 回合）
    duration_turns: Some(5),

    // 免疫规则：拥有此标签的实体不会被施加燃烧
    immunity_tags: ["tag:immunity_fire", "tag:immunity_burn"],

    // 与其他减益互斥
    exclusive_with: ["buff:freezing", "buff:stun"],

    // 激活条件（可选）
    condition: Some("cond:target_not_dead"),

    // 持有此 Buff 期间获得的标签
    granted_tags: ["tag:status_burning"],

    // 表现绑定
    cues: [
        (cue_tag: OnApply, cue_def_id: "cue:fire_spark"),
        (cue_tag: OnTick, cue_def_id: "cue:burn_tick"),
        (cue_tag: OnRemove, cue_def_id: "cue:fire_extinguish"),
    ],
)
```

---

## 6. BuffDef vs EffectDef 完整对比

| 对比维度 | EffectDef | BuffDef |
|----------|-----------|---------|
| 本质 | 效果逻辑定义 | 持久状态的 Effect 容器 |
| 是否包含 Modifier | 是（modifiers 字段） | 否（委托给 EffectDef） |
| 是否包含 Execution | 是（execution 字段） | 否（委托给 EffectDef） |
| 是否包含 Duration | 是 | 可选覆盖 |
| 是否包含 Period | 是（可选） | 可选覆盖 |
| 是否包含 Stacking | 是（可选） | 可选覆盖 |
| 是否包含 Cue | 是 | 是（追加绑定） |
| 分类系统 | EffectCategory 枚举 | StatusCategoryId + BuffType |
| 免疫规则 | 无 | immunity_tags |
| 互斥规则 | 无 | exclusive_with |
| 资源引用 | 无直接资产依赖 | icon_key（UI 图标） |
| 典型加载文件 | `effects.ron` | `buffs.ron` |
| 创建方式 | Ability 效果链直接引用 | BuffPlugin 独立管理 |
