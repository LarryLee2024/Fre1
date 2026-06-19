---
id: 03-content.definitions.cue-def
title: CueDef — Cue Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# CueDef — Cue Content Def 定义

> **Content Layer**: L1 Capability | **领域规则**: `docs/02-domain/capabilities/cue_domain.md` | **数据 Schema**: `docs/04-data/capabilities/cue_schema.md` | **插件代码**: `src/content/plugins/cue_plugin.rs`

---

## 1. Overview

CueDef 定义**表现信号**——VFX、SFX、动画、屏幕震动、UI 浮动文字等所有视觉效果和音频反馈。Cue 是逻辑层与表现层的单向桥梁：

- VFX：火焰爆炸、冰冻光效、治疗波纹
- SFX：技能音效、命中声音、环境音
- Animation：施法动画、受击动画、死亡动画
- Shake：屏幕震动（伤害反馈、地震效果）
- Popup：伤害数字、治疗数字、状态文字（"免疫！"、"暴击！"）

### 设计原则

- **Cue 是纯表现**：Cue 触发是 fire-and-forget 的——逻辑层发出 Cue 信号后不等待表现层完成
- **Cue 不修改逻辑数据**：表现层不得通过 Cue 反向修改游戏状态
- **Cue 可附带条件**：Cue 可以在特定条件下触发（如暴击时显示暴击 Popup）
- **关键/非关键区分**：关键 Cue（critical=true）必须播放，非关键 Cue 可在性能压力下被丢弃

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `cue_domain.md` | Cue 的触发生命周期、与 Effect 的绑定规则、无限循环防护 |
| `cue_schema.md` | CueDef 完整字段、CueType(VFX/SFX/Animation/Shake/Popup) 参数、CueTag |
| `effect-def.md` | 本 Def 被 EffectDef.cues[].cue_def_id 引用 |
| `buff-def.md` | 本 Def 被 BuffDef.cues[].cue_def_id 引用 |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// Cue Def 定义——表现信号。
///
/// 定义 VFX/SFX/动画/震动/UI 文字的信号参数，被 EffectDef 和 BuffDef 引用。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct CueDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID
    pub id: CueId,
    /// 显示名称（本地化 Key，主要用于调试和编辑器）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 信号核心 ──
    /// 信号类型
    pub cue_type: CueType,

    /// 触发时机标签
    pub cue_tag: CueTag,

    /// 延迟触发（帧数，用于错开多 Cue 播放时间）
    pub delay_frames: Option<u64>,

    // ── 条件与控制 ──
    /// 该 Cue 是否可被打断
    pub interruptible: bool,

    /// 触发条件（可选，条件满足时才触发此 Cue）
    pub condition: Option<ConditionDefId>,

    /// 是否必须播放（不可丢弃）
    pub critical: bool,

    // ── 元数据 ──
    /// 分类标签
    pub tags: Vec<TagId>,
}

/// 信号类型及参数
#[derive(Deserialize, Clone, Debug)]
pub enum CueType {
    /// 视觉特效
    VFX(VFXParams),
    /// 音效
    SFX(SFXParams),
    /// 动画
    Animation(AnimationParams),
    /// 屏幕震动
    Shake(ShakeParams),
    /// UI 浮动文字
    Popup(PopupParams),
}

/// VFX 参数
#[derive(Deserialize, Clone, Debug)]
pub struct VFXParams {
    /// 特效资源路径 Key
    pub effect_key: String,
    /// 附着点（如 weapon_tip, chest, ground）
    pub attach_point: Option<String>,
    /// 是否跟随目标移动
    pub follow_target: bool,
    /// 持续帧数
    pub duration_frames: Option<u64>,
    /// 缩放
    pub scale: Option<f32>,
    /// 颜色覆盖
    pub color_override: Option<String>,
}

/// SFX 参数
#[derive(Deserialize, Clone, Debug)]
pub struct SFXParams {
    /// 音效资源路径 Key
    pub sound_key: String,
    /// 音量 0.0-1.0
    pub volume: f32,
    /// 是否 3D 空间音效
    pub is_3d: bool,
    /// 音调偏移
    pub pitch_shift: Option<f32>,
}

/// 动画参数
#[derive(Deserialize, Clone, Debug)]
pub struct AnimationParams {
    /// 动画名称
    pub animation_name: String,
    /// 播放速率
    pub speed: f32,
    /// 是否循环播放
    pub r#loop: bool,
    /// 淡入淡出帧数
    pub crossfade_frames: Option<u64>,
}

/// 屏幕震动参数
#[derive(Deserialize, Clone, Debug)]
pub struct ShakeParams {
    /// 振幅
    pub intensity: f32,
    /// 持续帧数
    pub duration_frames: u64,
    /// 衰减曲线
    pub falloff: ShakeFalloff,
}

/// UI 浮动文字参数
#[derive(Deserialize, Clone, Debug)]
pub struct PopupParams {
    /// 文字内容 Key（本地化）
    pub text_key: LocalizationKey,
    /// 文字颜色（十六进制）
    pub color: String,
    /// 字体大小
    pub font_size: u8,
    /// 浮动方向
    pub float_direction: PopupDirection,
    /// 持续帧数
    pub duration_frames: u64,
}

/// Cue 触发时机
#[derive(Deserialize, Clone, Debug)]
pub enum CueTag {
    /// 效果/技能被应用时触发
    OnApply,
    /// 持续效果的每次 Tick 触发
    OnTick,
    /// 效果被移除时触发
    OnRemove,
    /// 技能/效果被打断时触发
    OnInterrupt,
    /// 自定义触发时机
    Custom(String),
}

#[derive(Deserialize, Clone, Debug)]
pub enum ShakeFalloff { Linear, Exponential, None, }

#[derive(Deserialize, Clone, Debug)]
pub enum PopupDirection { Up, Down, Left, Right, Random, }
```

### 字段说明

- **`cue_type`**: 五种表现类型。每个类型有自己的参数结构体，使用资源 Key 而非硬编码路径
- **`cue_tag`**: 触发时机，与 Effect 生命周期绑定。OnApply（应用时）、OnTick（每 Tick）、OnRemove（移除时）、OnInterrupt（中断时）
- **`interruptible`**: 可打断性。例如长动画可以被新 Cue 打断，短音效不可打断
- **`condition`**: 条件触发。例如"暴击时显示暴击文字"通过 ConditionDefId 引用一个条件
- **`critical`**: 关键 Cue 标记。为 true 时播放系统必须确保播放（如死亡动画），为 false 时可在负载过高时丢弃
- **资源 Key**: VFXParams.effect_key、SFXParams.sound_key、AnimationParams.animation_name 使用资源 Key（字符串 ID），而非直接文件路径。Key 到实际资源文件的映射由资产管理系统处理

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// CueDef 注册插件
pub struct CueDefPlugin;

impl Plugin for CueDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<CueDef>();
        app.init_asset_loader::<RonAssetLoader<CueDef>>();
        app.insert_resource(DefRegistry::<CueDef>::new());

        app.add_systems(
            PreUpdate,
            load_cue_defs
                .run_if(resource_changed::<Assets<CueDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按 ID 查找 CueDef
pub fn get_cue_def(id: &CueId, registry: &DefRegistry<CueDef>) -> Option<&CueDef> {
    registry.get(id)
}

/// 按 CueTag 过滤
pub fn get_cues_by_tag(cue_tag: &CueTag, registry: &DefRegistry<CueDef>) -> Vec<&CueDef> {
    registry.iter()
        .filter(|def| matches_cue_tag(&def.cue_tag, cue_tag))
        .collect()
}
```

### 注册生命周期

```
Load (cues.ron) → Deserialize → Validate → Register (DefRegistry<CueDef>) → Freeze
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | CueId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | VFXParams.effect_key 非空 | VFX 资源 Key 不能为空 |
| V4 | SFXParams.sound_key 非空 | 音效资源 Key 不能为空 |
| V5 | SFXParams.volume 范围 | 0.0 <= volume <= 1.0 |
| V6 | AnimationParams.animation_name 非空 | 动画名称不能为空 |
| V7 | ShakeParams.intensity > 0.0 | 震动强度必须为正 |
| V8 | ShakeParams.duration_frames > 0 | 震动持续帧数必须为正 |
| V9 | PopupParams.font_size 范围 | 8 <= font_size <= 72 |
| V10 | PopupParams.duration_frames > 0 | Popup 持续帧数必须为正 |
| V11 | PopupParams.color 有效十六进制格式 | 必须为 "#XXXXXX" 格式 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V12 | `condition` (如果设置) 已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V13 | 资源 Key 对应的资产文件存在 | effect_key/sound_key 对应资产路径校验（仅 warn，非硬错误） |
| V14 | CueDef 不得引用任何 L2+ Def | L1 内容不可引用 Entity/Gameplay/World 层内容 |

---

## 5. RON 示例

```ron
// CueDef 示例：火焰爆炸 VFX
//
// 技能命中时的火焰爆炸视觉效果。
(
    id: "cue:explosion_fire",
    name_key: "cue.explosion_fire.name",
    description_key: "cue.explosion_fire.desc",
    schema_version: 1,

    cue_type: VFX((
        effect_key: "vfx_fire_explosion",
        attach_point: Some("target_center"),
        follow_target: false,
        duration_frames: Some(30),
        scale: Some(1.5),
    )),

    cue_tag: OnApply,
    interruptible: false,
    critical: false,

    tags: ["tag:combat", "tag:vfx", "tag:fire"],
)
```

```ron
// CueDef 示例：暴击伤害数字 Popup
//
// 在暴击时显示金色的"暴击！"文字 + 伤害数字。
(
    id: "cue:critical_popup",
    name_key: "cue.critical_popup.name",
    description_key: "cue.critical_popup.desc",
    schema_version: 1,

    cue_type: Popup((
        text_key: "cue.critical_popup.text",
        color: "#FFD700",
        font_size: 24,
        float_direction: Up,
        duration_frames: 60,
    )),

    cue_tag: OnApply,
    delay_frames: Some(3),
    interruptible: true,
    critical: false,

    // 只有暴击时才显示
    condition: Some("cond:is_critical_hit"),

    tags: ["tag:combat", "tag:ui", "tag:critical"],
)
```

```ron
// CueDef 示例：治疗音效
//
// 友方治疗时的柔和的音效。
(
    id: "cue:heal_sound",
    name_key: "cue.heal_sound.name",
    description_key: "cue.heal_sound.desc",
    schema_version: 1,

    cue_type: SFX((
        sound_key: "sfx_heal_gentle",
        volume: 0.7,
        is_3d: true,
        pitch_shift: Some(1.0),
    )),

    cue_tag: OnApply,
    interruptible: true,
    critical: false,

    tags: ["tag:combat", "tag:sfx", "tag:heal"],
)
```
