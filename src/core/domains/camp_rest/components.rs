//! ECS Components — 营地/休息领域组件与类型
//!
//! 定义休息状态、生命骰池、营地 NPC 等组件。
//! 详见 docs/02-domain/domains/camp_rest_domain.md
//! 详见 docs/04-data/domains/camp_rest_schema.md

use bevy::prelude::*;

// ─── ID 类型 ──────────────────────────────────────────────────────

/// 营地事件模板标识符（前缀: `cmp_`）。
///
/// 统一使用 shared::ids::CampEventId。
pub use crate::shared::ids::CampEventId;

// ─── 值类型 ────────────────────────────────────────────────────────

/// 休息类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum RestType {
    ShortRest,
    LongRest,
}

/// 休息阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum RestPhase {
    /// 未在休息中。
    Idle,
    /// 休息进行中（短休 / 长休睡眠阶段）。
    Resting,
    /// 长休轻活动阶段。
    LightActivity,
    /// 已完成。
    Complete,
    /// 长休失败（中断超 1h）。
    Failed,
}

impl RestPhase {
    pub fn is_resting(&self) -> bool {
        matches!(self, RestPhase::Resting | RestPhase::LightActivity)
    }
}

/// 生命骰类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum DiceType {
    D6,
    D8,
    D10,
    D12,
}

impl DiceType {
    /// 获取该骰子的最大面值。
    pub fn max_value(&self) -> u32 {
        match self {
            DiceType::D6 => 6,
            DiceType::D8 => 8,
            DiceType::D10 => 10,
            DiceType::D12 => 12,
        }
    }
}

/// 营地事件类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum CampEventType {
    /// 剧情推进。
    Story,
    /// 角色个人事件。
    Character,
    /// 随机遭遇。
    Random,
    /// 纯休息（无事件）。
    Rest,
}

// ─── ECS Components ───────────────────────────────────────────────

/// 休息状态组件（Instance 层）。
///
/// 标记队伍当前的休息阶段和进度。
/// 不变量 3.1~3.5：休息频率、安全要求、中断规则。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct RestState {
    /// 休息类型（None = 不在休息）。
    pub rest_type: Option<RestType>,
    /// 当前阶段。
    pub phase: RestPhase,
    /// 长休中断累计时间（分钟）。
    pub interrupt_duration: u32,
    /// 上次长休完成的 GameTime 帧计数（用于 24 小时限制检查）。
    pub last_long_rest_frame: Option<u64>,
}

impl RestState {
    /// 创建默认休息状态（Idle）。
    pub fn new() -> Self {
        Self {
            rest_type: None,
            phase: RestPhase::Idle,
            interrupt_duration: 0,
            last_long_rest_frame: None,
        }
    }

    /// 开始短休。
    pub fn start_short_rest(&mut self) {
        self.rest_type = Some(RestType::ShortRest);
        self.phase = RestPhase::Resting;
        self.interrupt_duration = 0;
    }

    /// 开始长休。
    pub fn start_long_rest(&mut self) {
        self.rest_type = Some(RestType::LongRest);
        self.phase = RestPhase::Resting;
        self.interrupt_duration = 0;
    }

    /// 进入长休轻活动阶段。
    pub fn enter_light_activity(&mut self) {
        self.phase = RestPhase::LightActivity;
    }

    /// 完成休息。
    pub fn complete(&mut self) {
        self.phase = RestPhase::Complete;
    }

    /// 标记休息失败。
    pub fn fail(&mut self) {
        self.phase = RestPhase::Failed;
    }

    /// 重置为 Idle。
    pub fn reset(&mut self) {
        self.rest_type = None;
        self.phase = RestPhase::Idle;
        self.interrupt_duration = 0;
    }
}

impl Default for RestState {
    fn default() -> Self {
        Self::new()
    }
}

/// 生命骰池组件（Instance 层/Persistence 层）。
///
/// 短休时可消耗来恢复 HP。
/// 不变量 3.4：生命骰恢复上限为角色等级的一半。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct HitDicePool {
    /// 当前可用的生命骰数量。
    pub current: u32,
    /// 最大生命骰数（等于角色等级）。
    pub max: u32,
    /// 生命骰类型（按职业）。
    pub dice_type: DiceType,
}

impl HitDicePool {
    /// 创建初始生命骰池。
    pub fn new(max: u32, dice_type: DiceType) -> Self {
        Self {
            current: max,
            max,
            dice_type,
        }
    }

    /// 消耗生命骰。
    ///
    /// 返回是否成功消耗。
    pub fn spend(&mut self, count: u32) -> bool {
        if count <= self.current {
            self.current -= count;
            true
        } else {
            false
        }
    }

    /// 长休恢复生命骰。
    ///
    /// 恢复量为 max(当前, ceil(等级/2))，但不超过 max。
    /// 不变量 3.4。
    pub fn recover_for_long_rest(&mut self) {
        let max_after_rest = (self.max + 1) / 2; // ceil(level/2)，max 等于角色等级
        let recover_to = max_after_rest.min(self.max);
        if self.current < recover_to {
            self.current = recover_to;
        }
    }

    /// 设置新的最大值（升级时调用）。
    pub fn set_max(&mut self, new_max: u32) {
        self.max = new_max;
        if self.current > self.max {
            self.current = self.max;
        }
    }
}

/// 营地 NPC 组件（Instance 层）。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct CampNPC {
    /// 当前是否在营地中。
    pub is_at_camp: bool,
    /// 可用对话选项列表（由 Narrative 领域提供）。
    pub available_dialogues: Vec<String>,
}

impl CampNPC {
    pub fn new() -> Self {
        Self {
            is_at_camp: false,
            available_dialogues: Vec::new(),
        }
    }
}

impl Default for CampNPC {
    fn default() -> Self {
        Self::new()
    }
}

/// 营地事件模板定义（Definition 层）。
#[derive(Debug, Clone, Reflect)]
pub struct CampEventDef {
    /// 营地事件唯一标识。
    pub id: CampEventId,
    /// 事件标题本地化 Key。
    pub title_key: String,
    /// 事件描述本地化 Key。
    pub desc_key: String,
    /// 触发条件（ConditionDefId 占位）。
    pub trigger_conditions: Vec<String>,
    /// 事件类型。
    pub event_type: CampEventType,
    /// 事件优先级（数值越小优先级越高）。
    pub priority: u32,
}

/// 营地/休息系统标记组件。
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct CampRestMarker;
