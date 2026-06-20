//! Trigger 基础类型与枚举

use std::collections::HashMap;

use bevy::prelude::Reflect;

/// 触发类型枚举。
///
/// 每个变体对应一类事件源。OnCustom 供 Domain 注册自定义触发类型。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub enum TriggerType {
    /// 标签被授予时触发
    OnTagAdded,
    /// 标签被移除时触发
    OnTagRemoved,
    /// 受到伤害时触发
    OnDamaged,
    /// 受到治疗时触发
    OnHealed,
    /// 发动攻击时触发
    OnAttack,
    /// 回合开始时触发
    OnTurnStart,
    /// 回合结束时触发
    OnTurnEnd,
    /// 单位死亡时触发
    OnDeath,
    /// 移动时触发
    OnMove,
    /// 技能被使用时触发
    OnAbilityUsed,
    /// 特定 Condition 满足时触发
    OnConditionMet,
    /// 自定义触发事件（由 Domain 注册）
    OnCustom(String),
}

impl TriggerType {
    /// 返回的是 Rust 变体名的字面量（Custom 变体返回 "OnCustom" 固定值，不包含内部字符串）。
    pub fn name(&self) -> &str {
        match self {
            Self::OnTagAdded => "OnTagAdded",
            Self::OnTagRemoved => "OnTagRemoved",
            Self::OnDamaged => "OnDamaged",
            Self::OnHealed => "OnHealed",
            Self::OnAttack => "OnAttack",
            Self::OnTurnStart => "OnTurnStart",
            Self::OnTurnEnd => "OnTurnEnd",
            Self::OnDeath => "OnDeath",
            Self::OnMove => "OnMove",
            Self::OnAbilityUsed => "OnAbilityUsed",
            Self::OnConditionMet => "OnConditionMet",
            Self::OnCustom(_) => "OnCustom",
        }
    }
}

/// 触发频率状态。
///
/// 管理单回合内的触发次数限制（不变量 §3.3）。
#[derive(Debug, Clone, Reflect)]
pub struct TriggerFrequency {
    /// 每回合最大触发次数（0 = 无限制）
    pub max_per_turn: u32,
    /// 当前回合已触发次数
    pub current_turn_count: u32,
}

impl TriggerFrequency {
    /// max_per_turn=0，current_turn_count 初始为 0。由 TriggerSystem 在回合切换时重置。
    pub fn unlimited() -> Self {
        Self {
            max_per_turn: 0,
            current_turn_count: 0,
        }
    }

    /// 由 TriggerEntry 在定义时指定 max。can_trigger() 返回 false 后本回合不再触发。
    pub fn limited(max: u32) -> Self {
        Self {
            max_per_turn: max,
            current_turn_count: 0,
        }
    }

    /// 检查是否可在当前回合再次触发。
    pub fn can_trigger(&self) -> bool {
        self.max_per_turn == 0 || self.current_turn_count < self.max_per_turn
    }

    /// 用于避免溢出。saturating_add 确保从 u32::MAX 开始不会 panic。
    pub fn record_trigger(&mut self) {
        self.current_turn_count = self.current_turn_count.saturating_add(1);
    }

    /// 回合切换时由 TriggerSystem 调用。仅重置计数，不影响 max_per_turn。
    pub fn reset_turn(&mut self) {
        self.current_turn_count = 0;
    }
}

/// 触发器 ID 生成器（确定性）。
///
/// 使用 AtomicU64 生成 Replay-safe 的唯一触发 ID。
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TRIGGER_INSTANCE_ID: AtomicU64 = AtomicU64::new(1);

/// 生成一个唯一触发实例 ID（格式: "trig_{n}"）。
pub fn generate_trigger_id() -> String {
    let id = NEXT_TRIGGER_INSTANCE_ID.fetch_add(1, Ordering::Relaxed);
    format!("trig_{:010}", id)
}

/// 额外参数字典。
pub type TriggerParams = HashMap<String, String>;
