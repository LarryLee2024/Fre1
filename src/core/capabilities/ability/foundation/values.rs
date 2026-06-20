//! Ability 值对象
//!
//! 技能运行时实例、激活上下文、消耗追踪、冷却状态。
//!
//! 详见 docs/04-data/capabilities/ability_schema.md §3.6、§3.7。

use crate::core::capabilities::ability::foundation::types::{
    AbilityInstanceId, AbilityState, ActivationType,
};

/// 技能激活时的上下文信息。
#[derive(Debug, Clone)]
pub struct ActivationContext {
    /// 施法者实体（字符串标识，跨领域兼容）
    pub caster: String,
    /// 可选的目标实体（预选目标）
    pub target: Option<String>,
    /// 可选的目标位置（网格坐标字符串，如 "5,3"）
    pub target_position: Option<String>,
    /// 当前游戏帧号
    pub frame: u64,
    /// 激活时使用的等级覆盖（None 表示使用 Spec 当前等级）
    pub level_override: Option<u8>,
}

impl ActivationContext {
    /// 初始化施法上下文。后续通过 with_target / with_position / with_level_override 链式填充。
    /// frame 在生命周期内不可变——Replay 依赖此值进行帧精确还原。
    pub fn new(caster: impl Into<String>, frame: u64) -> Self {
        Self {
            caster: caster.into(),
            target: None,
            target_position: None,
            frame,
            level_override: None,
        }
    }

    /// 预选目标实体（可选——部分技能在激活时不一定确定目标，由后续 Targeting 系统选择）。
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// 目标位置（网格坐标字符串，如 "5,3"）。地面目标技能需要此值进行 AoE 判定。
    pub fn with_position(mut self, pos: impl Into<String>) -> Self {
        self.target_position = Some(pos.into());
        self
    }

    /// 升环施法时覆盖技能等级。None 表示使用 Spec 当前等级。
    /// 等级覆盖影响 Execution 阶段的计算（如伤害骰数量）。
    pub fn with_level_override(mut self, level: u8) -> Self {
        self.level_override = Some(level);
        self
    }
}

/// 资源消耗追踪条目。
#[derive(Debug, Clone)]
pub struct CostEntry {
    /// 消耗的资源属性 ID
    pub resource: String,
    /// 消耗量
    pub amount: f32,
    /// 是否已消耗
    pub consumed: bool,
}

impl CostEntry {
    /// 追踪技能激活所需的资源消耗。consumed=false 表示尚未从实体扣除。
    /// 不变量 §3.4：所有消耗条目必须在技能执行完毕前标记为 consumed。
    pub fn new(resource: impl Into<String>, amount: f32) -> Self {
        Self {
            resource: resource.into(),
            amount,
            consumed: false,
        }
    }
}

/// 冷却状态条目。
#[derive(Debug, Clone)]
pub struct CooldownEntry {
    /// 关联的 Spec ID
    pub spec_id: String,
    /// 总冷却回合数
    pub total_turns: u32,
    /// 剩余回合数
    pub remaining_turns: u32,
    /// 共享冷却组名（可选）
    pub shared_group: Option<String>,
    /// 冷却是否从激活时开始计时
    pub starts_on_activate: bool,
}

impl CooldownEntry {
    /// total_turns = remaining_turns 初始化。冷却结束后由 CooldownSystem 移除。
    pub fn new(spec_id: impl Into<String>, total_turns: u32) -> Self {
        Self {
            spec_id: spec_id.into(),
            total_turns,
            remaining_turns: total_turns,
            shared_group: None,
            starts_on_activate: false,
        }
    }

    /// 同组技能共享冷却时间（如"所有传送技能共享 3 回合冷却"）。
    pub fn with_shared_group(mut self, group: impl Into<String>) -> Self {
        self.shared_group = Some(group.into());
        self
    }

    /// 默认冷却从技能进入 Cooldown 状态开始计时。
    /// 设为 true 后从 Active 状态就开始计时，适合"持续效果结束后冷却已过半"的设计。
    pub fn with_starts_on_activate(mut self, value: bool) -> Self {
        self.starts_on_activate = value;
        self
    }

    /// 冷却结束后由 CooldownSystem 在回合开始时自动移除。
    pub fn is_expired(&self) -> bool {
        self.remaining_turns == 0
    }

    /// 每回合开始时由 CooldownSystem 调用。到 0 后 is_expired() 返回 true。
    pub fn tick(&mut self) {
        self.remaining_turns = self.remaining_turns.saturating_sub(1);
    }
}

/// 技能激活后的运行时实例。
///
/// 携带激活时的完整上下文，追踪技能执行生命周期。
///
/// 详见 docs/04-data/capabilities/ability_schema.md §3.6。
#[derive(Debug, Clone)]
pub struct AbilityInstance {
    /// 实例唯一标识
    pub instance_id: AbilityInstanceId,
    /// 关联的 Spec ID
    pub spec_id: String,
    /// 引用的 AbilityDef ID
    pub def_id: String,
    /// 当前状态
    pub state: AbilityState,
    /// 激活类型
    pub activation: ActivationType,
    /// 激活时的上下文
    pub context: ActivationContext,
    /// 施法进度（Casting 状态下使用）
    pub cast_progress: u64,
    /// 总施法帧数
    pub cast_total: u64,
    /// 是否暂停（如被沉默/眩晕打断）
    pub paused: bool,
    /// 当前正在执行的效果链索引
    pub current_effect_index: usize,
    /// 实例创建帧号
    pub created_at_frame: u64,
    /// 消耗追踪列表
    pub costs: Vec<CostEntry>,
}

impl AbilityInstance {
    /// 创建新的 AbilityInstance（初始状态为 Casting 或 Ready）。
    ///
    /// - 瞬发技能（is_instant=true）初始状态为 Active
    /// - 非瞬发技能初始状态为 Casting
    pub fn new(
        instance_id: AbilityInstanceId,
        spec_id: impl Into<String>,
        def_id: impl Into<String>,
        activation: ActivationType,
        context: ActivationContext,
    ) -> Self {
        let frame = context.frame;
        let is_instant = activation.is_instant();

        Self {
            instance_id,
            spec_id: spec_id.into(),
            def_id: def_id.into(),
            state: if is_instant {
                AbilityState::Active
            } else {
                AbilityState::Casting
            },
            activation,
            context,
            cast_progress: 0,
            cast_total: if is_instant {
                0
            } else {
                activation.cast_frames()
            },
            paused: false,
            current_effect_index: 0,
            created_at_frame: frame,
            costs: Vec::new(),
        }
    }

    /// 在技能激活时由 ActivationSystem 调用，记录需消耗的资源。
    pub fn add_cost(&mut self, entry: CostEntry) {
        self.costs.push(entry);
    }

    /// 由 ExecutionSystem 在执行完成后检查。全部 consumed 才能标记为执行完毕。
    pub fn all_costs_consumed(&self) -> bool {
        !self.costs.is_empty() && self.costs.iter().all(|c| c.consumed)
    }

    /// 由 CostSystem 在扣除资源后调用。不变量 §3.4：一旦标记不可回退。
    pub fn mark_costs_consumed(&mut self) {
        for cost in &mut self.costs {
            cost.consumed = true;
        }
    }

    /// 推进施法进度。返回 true 表示施法完成。
    pub fn advance_cast(&mut self, delta: u64) -> bool {
        if self.state != AbilityState::Casting || self.paused {
            return false;
        }
        self.cast_progress = self.cast_progress.saturating_add(delta);
        if self.cast_progress >= self.cast_total {
            self.cast_progress = self.cast_total;
            true
        } else {
            false
        }
    }

    /// 实例是否处于活跃状态（Casting/Active，且未暂停）。
    pub fn is_active(&self) -> bool {
        matches!(self.state, AbilityState::Casting | AbilityState::Active) && !self.paused
    }

    /// 实例是否处于执行阶段（Active）。
    pub fn is_executing(&self) -> bool {
        self.state == AbilityState::Active && !self.paused
    }
}

/// 阻塞恢复后的状态还原信息。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockedRestoreState {
    /// 回到 Ready
    Ready,
    /// 回到 Casting（恢复施法进度）
    Casting,
    /// 回到 Active（恢复执行）
    Active,
    /// 回到 Cooldown
    Cooldown,
}

impl BlockedRestoreState {
    /// 阻塞解除后，技能恢复到对应状态。由 BlockSystem 在 remove_block 时调用。
    pub fn to_state(self) -> AbilityState {
        match self {
            Self::Ready => AbilityState::Ready,
            Self::Casting => AbilityState::Casting,
            Self::Active => AbilityState::Active,
            Self::Cooldown => AbilityState::Cooldown,
        }
    }
}
