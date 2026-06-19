//! Cue 基础类型与枚举
//!
//! 定义表现信号的类型、参数、触发时机以及领域错误。
//!
//! 详见 docs/02-domain/capabilities/cue_domain.md §1、§3。
//! 详见 docs/04-data/capabilities/cue_schema.md §3。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 表现信号类型枚举。
///
/// 决定信号在 Infra 表现层由哪个子系统处理。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub enum CueType {
    /// 视觉特效（粒子/光效/拖尾/爆炸）
    VFX(VFXParams),
    /// 音效（音效/语音/环境声）
    SFX(SFXParams),
    /// 动画（骨骼动画/状态切换）
    Animation(AnimationParams),
    /// 屏幕震动
    Shake(ShakeParams),
    /// UI 浮动文字（伤害数字/状态提示）
    Popup(PopupParams),
}

impl CueType {
    /// 返回信号类型名称。
    pub fn name(&self) -> &str {
        match self {
            Self::VFX(_) => "VFX",
            Self::SFX(_) => "SFX",
            Self::Animation(_) => "Animation",
            Self::Shake(_) => "Shake",
            Self::Popup(_) => "Popup",
        }
    }
}

/// VFX 视觉特效参数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct VFXParams {
    /// 特效资源 Key
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

impl VFXParams {
    /// 创建 VFX 参数。
    pub fn new(effect_key: impl Into<String>) -> Self {
        Self {
            effect_key: effect_key.into(),
            attach_point: None,
            follow_target: false,
            duration_frames: None,
            scale: None,
            color_override: None,
        }
    }

    /// 设置附着点。
    pub fn with_attach_point(mut self, point: impl Into<String>) -> Self {
        self.attach_point = Some(point.into());
        self
    }

    /// 设置跟随目标。
    pub fn with_follow(mut self, follow: bool) -> Self {
        self.follow_target = follow;
        self
    }

    /// 设置持续帧数。
    pub fn with_duration(mut self, frames: u64) -> Self {
        self.duration_frames = Some(frames);
        self
    }
}

/// SFX 音效参数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct SFXParams {
    /// 音效资源 Key
    pub sound_key: String,
    /// 音量 0.0–1.0
    pub volume: f32,
    /// 是否 3D 空间音效
    pub is_3d: bool,
    /// 音调偏移
    pub pitch_shift: Option<f32>,
}

impl SFXParams {
    /// 创建 SFX 参数。
    pub fn new(sound_key: impl Into<String>) -> Self {
        Self {
            sound_key: sound_key.into(),
            volume: 1.0,
            is_3d: true,
            pitch_shift: None,
        }
    }

    /// 设置音量。
    ///
    /// # Panics
    /// V2: 音量必须在 0.0–1.0 范围内。
    pub fn with_volume(mut self, volume: f32) -> Self {
        assert!(
            (0.0..=1.0).contains(&volume),
            "volume must be in 0.0..1.0 range"
        );
        self.volume = volume;
        self
    }
}

/// 动画参数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct AnimationParams {
    /// 动画名称
    pub animation_name: String,
    /// 播放速率
    pub speed: f32,
    /// 是否循环
    pub loop_anim: bool,
    /// 淡入淡出帧数
    pub crossfade_frames: Option<u64>,
}

impl AnimationParams {
    /// 创建动画参数。
    pub fn new(animation_name: impl Into<String>) -> Self {
        Self {
            animation_name: animation_name.into(),
            speed: 1.0,
            loop_anim: false,
            crossfade_frames: None,
        }
    }
}

/// 屏幕震动参数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct ShakeParams {
    /// 振幅
    pub intensity: f32,
    /// 持续帧数
    pub duration_frames: u64,
    /// 衰减曲线
    pub falloff: ShakeFalloff,
}

impl ShakeParams {
    /// 创建震动参数。
    pub fn new(intensity: f32, duration_frames: u64) -> Self {
        Self {
            intensity,
            duration_frames,
            falloff: ShakeFalloff::Linear,
        }
    }
}

/// 震动衰减曲线。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum ShakeFalloff {
    Linear,
    Exponential,
    None,
}

/// UI 浮动文字参数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct PopupParams {
    /// 本地化 Key
    pub text_key: String,
    /// 文字颜色（CSS 格式）
    pub color: String,
    /// 字体大小
    pub font_size: u8,
    /// 浮动方向
    pub float_direction: PopupDirection,
    /// 持续帧数
    pub duration_frames: u64,
}

impl PopupParams {
    /// 创建 Popup 参数。
    pub fn new(text_key: impl Into<String>, color: impl Into<String>) -> Self {
        Self {
            text_key: text_key.into(),
            color: color.into(),
            font_size: 16,
            float_direction: PopupDirection::Up,
            duration_frames: 60,
        }
    }
}

/// Popup 浮动方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum PopupDirection {
    Up,
    Down,
    Left,
    Right,
    Random,
}

/// 触发时机标签。
///
/// 标记信号在效果生命周期的哪个阶段触发。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
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

impl CueTag {
    /// 返回触发时机名称。
    pub fn name(&self) -> &str {
        match self {
            Self::OnApply => "OnApply",
            Self::OnTick => "OnTick",
            Self::OnRemove => "OnRemove",
            Self::OnInterrupt => "OnInterrupt",
            Self::Custom(tag) => tag.as_str(),
        }
    }
}

/// Cue 信号定义（Definition 层）。
#[derive(Debug, Clone, PartialEq, Asset, Serialize, Deserialize, Reflect)]
pub struct CueDef {
    /// 信号唯一标识
    pub id: String,
    /// 信号类型及参数
    pub cue_type: CueType,
    /// 触发时机
    pub cue_tag: CueTag,
    /// 延迟触发（帧数）
    pub delay_frames: Option<u64>,
    /// 是否可被打断
    pub interruptible: bool,
    /// 关键信号（必须播放，不可丢弃）
    pub critical: bool,
}

impl CueDef {
    /// 创建 Cue 定义。
    pub fn new(id: impl Into<String>, cue_type: CueType, cue_tag: CueTag) -> Self {
        Self {
            id: id.into(),
            cue_type,
            cue_tag,
            delay_frames: None,
            interruptible: true,
            critical: false,
        }
    }

    /// 设置延迟触发。
    pub fn with_delay(mut self, frames: u64) -> Self {
        self.delay_frames = Some(frames);
        self
    }

    /// 标记为关键信号。
    pub fn with_critical(mut self) -> Self {
        self.critical = true;
        self.interruptible = false;
        self
    }
}

/// Cue 信号运行时数据载体（Runtime 层）。
///
/// 当信号被触发时创建的瞬时数据，包含表现层所需的所有上下文。
#[derive(Debug, Clone, PartialEq)]
pub struct CueData {
    /// CueDef ID
    pub cue_def_id: String,
    /// 信号类型及参数
    pub cue_type: CueType,
    /// 触发时机
    pub cue_tag: CueTag,
    /// 来源实体
    pub source_entity: Option<String>,
    /// 目标实体
    pub target_entity: Option<String>,
    /// 数值（用于 Popup 显示伤害/治疗数字）
    pub numeric_value: Option<f32>,
    /// 是否为关键信号
    pub critical: bool,
}

impl CueData {
    /// 创建 Cue 信号数据。
    pub fn new(cue_def_id: impl Into<String>, cue_type: CueType, cue_tag: CueTag) -> Self {
        Self {
            cue_def_id: cue_def_id.into(),
            cue_type,
            cue_tag,
            source_entity: None,
            target_entity: None,
            numeric_value: None,
            critical: false,
        }
    }

    /// 设置来源实体。
    pub fn with_source(mut self, entity: impl Into<String>) -> Self {
        self.source_entity = Some(entity.into());
        self
    }

    /// 设置目标实体。
    pub fn with_target(mut self, entity: impl Into<String>) -> Self {
        self.target_entity = Some(entity.into());
        self
    }

    /// 设置数值。
    pub fn with_value(mut self, value: f32) -> Self {
        self.numeric_value = Some(value);
        self
    }

    /// 标记为关键信号。
    pub fn with_critical(mut self) -> Self {
        self.critical = true;
        self
    }
}

/// Cue 领域错误。
#[derive(Debug, Clone, PartialEq)]
pub enum CueError {
    /// Cue 未找到
    CueNotFound(String),
    /// 无效的参数
    InvalidParams(String),
}

impl std::fmt::Display for CueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CueNotFound(id) => write!(f, "cue '{}' not found", id),
            Self::InvalidParams(msg) => write!(f, "invalid cue params: {}", msg),
        }
    }
}

impl std::error::Error for CueError {}
