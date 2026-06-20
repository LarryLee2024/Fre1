//! Effect 值对象
//!
//! 周期 Tick 状态、效果运行时实例、活跃效果容器。
//!
//! 详见 docs/04-data/capabilities/effect_schema.md §3。

use bevy::prelude::*;

use crate::core::capabilities::effect::foundation::types::{
    EffectDuration, EffectPeriod, EffectStage,
};

/// 周期 Tick 状态——追踪持续性效果的周期性触发进度。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct TickState {
    /// 已触发的 Tick 次数
    pub tick_count: u32,
    /// 距下次 Tick 的剩余回合数
    pub remaining_turns: u32,
    /// 每次 Tick 的间隔回合数
    pub interval_turns: u32,
    /// 总 Tick 上限
    pub max_ticks: Option<u32>,
}

impl TickState {
    /// 从 EffectPeriod 初始化，remaining_turns 初始等于 interval_turns。
    pub fn new(period: &EffectPeriod) -> Self {
        Self {
            tick_count: 0,
            remaining_turns: period.interval_turns,
            interval_turns: period.interval_turns,
            max_ticks: period.max_ticks,
        }
    }

    /// max_ticks=None 时永不停止；max_ticks=Some(n) 时 tick_count < n 才能继续。
    pub fn has_more(&self) -> bool {
        match self.max_ticks {
            Some(max) => self.tick_count < max,
            None => true,
        }
    }

    /// 推进多回合，返回是否触发 Tick。
    ///
    /// 当 remaining_turns 减到 0 时触发 Tick 并重置计时。
    pub fn advance(&mut self, turns: u32) -> bool {
        if !self.has_more() {
            return false;
        }

        if turns >= self.remaining_turns {
            let overflow = turns - self.remaining_turns;
            self.tick_count += 1;
            self.remaining_turns = self.interval_turns.saturating_sub(overflow);
            true
        } else {
            self.remaining_turns -= turns;
            false
        }
    }
}

/// 效果运行时实例——单个 Effect 在实体上的运行时快照。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct EffectInstance {
    /// 实例唯一标识
    pub instance_id: String,
    /// EffectDef ID
    pub def_id: String,
    /// 效果分类标签（从 EffectDef.effect_tags 复制）
    pub tags: Vec<String>,
    /// 当前阶段
    pub stage: EffectStage,
    /// 来源实体标识（不变量 3.1）
    pub source_entity: String,
    /// 目标实体标识
    pub target_entity: String,
    /// 持续时间类型
    pub duration: EffectDuration,
    /// 剩余持续回合数
    pub remaining_turns: i64,
    /// 周期 Tick 状态（仅 Duration 类效果）
    pub tick_state: Option<TickState>,
    /// 当前堆叠层数
    pub stack_count: u32,
    /// 是否暂停
    pub paused: bool,
    /// 实例创建时的帧号
    pub created_at_turn: u64,
    /// 关联的 Modifier 数量（用于移除时回退追踪，不变量 3.4）
    pub modifier_count: u32,
    /// 是否可驱散
    pub dispellable: bool,
}

impl EffectInstance {
    /// 初始阶段为 Applying，由 EffectSystem 在施加校验通过后推进到 Active。
    pub fn new(
        instance_id: impl Into<String>,
        def_id: impl Into<String>,
        tags: Vec<String>,
        source_entity: impl Into<String>,
        target_entity: impl Into<String>,
        duration: EffectDuration,
        created_at_turn: u64,
    ) -> Self {
        let remaining_turns = duration.initial_remaining_turns();
        Self {
            instance_id: instance_id.into(),
            def_id: def_id.into(),
            tags,
            stage: EffectStage::Applying,
            source_entity: source_entity.into(),
            target_entity: target_entity.into(),
            duration,
            remaining_turns,
            tick_state: None,
            stack_count: 1,
            paused: false,
            created_at_turn,
            modifier_count: 0,
            dispellable: true,
        }
    }

    /// 仅 Duration 类效果需要 Tick 状态。Instant 效果设置此值无意义。
    pub fn with_period(mut self, period: EffectPeriod) -> Self {
        self.tick_state = Some(TickState::new(&period));
        self
    }

    /// 不可驱散的效果只能通过手动移除或来源死亡解除。默认可驱散（dispellable=true）。
    pub fn with_undispellable(mut self) -> Self {
        self.dispellable = false;
        self
    }

    /// 移除效果时需要回退对应数量的 Modifier。不变量 3.4：modifier_count 必须与 ModifierContainer 中的数量一致。
    pub fn with_modifiers(mut self, count: u32) -> Self {
        self.modifier_count = count;
        self
    }

    /// 由 StackingDecider 在判定后设置。默认值为 1（无堆叠）。
    pub fn with_stack(mut self, count: u32) -> Self {
        self.stack_count = count;
        self
    }

    /// 转换到下一阶段。
    ///
    /// 合法的阶段转换：
    /// - Applying → Active
    /// - Applying → Removed（Instant 效果执行完毕）
    /// - Active → Expiring
    /// - Active → Removed（被驱散/移除）
    /// - Expiring → Removed
    pub fn transition_to(&mut self, next: EffectStage) -> Result<(), super::types::EffectError> {
        if !matches!(
            (self.stage, next),
            (EffectStage::Applying, EffectStage::Active)
                | (EffectStage::Applying, EffectStage::Removed)
                | (EffectStage::Active, EffectStage::Expiring)
                | (EffectStage::Active, EffectStage::Removed)
                | (EffectStage::Expiring, EffectStage::Removed)
        ) {
            return Err(super::types::EffectError::InvalidStageTransition {
                from: self.stage,
                to: next,
                detail: format!("cannot transition from {:?} to {:?}", self.stage, next),
            });
        }

        self.stage = next;
        Ok(())
    }
}

/// 活跃效果容器——每个实体的效果管理组件。
///
/// 管理目标实体上所有当前生效的效果。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct ActiveEffectContainer {
    /// 所有活跃的效果实例
    pub effects: Vec<EffectInstance>,
    /// 效果槽位上限
    pub max_effects: u32,
}

impl ActiveEffectContainer {
    /// 默认效果槽位上限为 50（覆盖大多数角色同时生效的效果数量）。
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            max_effects: 50,
        }
    }

    /// 通过 Def 的 slot_limit 覆盖默认值。槽位满后新效果无法施加。
    pub fn with_max_effects(mut self, max: u32) -> Self {
        self.max_effects = max;
        self
    }

    /// 仅统计 Applying + Active 阶段的效果。Expiring/Removed 不计入。
    pub fn active_count(&self) -> usize {
        self.effects.iter().filter(|e| e.stage.is_active()).count()
    }

    /// 按 def_id 查找活跃效果实例。
    pub fn find_by_def(&self, def_id: &str) -> Vec<&EffectInstance> {
        self.effects
            .iter()
            .filter(|e| e.def_id == def_id && e.stage.is_active())
            .collect()
    }

    /// 精确查找（包含所有阶段，不仅限于活跃效果）。
    pub fn find_by_id(&self, instance_id: &str) -> Option<&EffectInstance> {
        self.effects.iter().find(|e| e.instance_id == instance_id)
    }

    /// 用户 EffectTickSystem 修改活跃效果的 TickState。
    pub fn find_by_id_mut(&mut self, instance_id: &str) -> Option<&mut EffectInstance> {
        self.effects
            .iter_mut()
            .find(|e| e.instance_id == instance_id)
    }

    /// 用于来源死亡级联移除。只返回 Applying + Active 阶段的效果。
    pub fn find_by_source(&self, source_entity: &str) -> Vec<&EffectInstance> {
        self.effects
            .iter()
            .filter(|e| e.source_entity == source_entity && e.stage.is_active())
            .collect()
    }

    /// 由 EffectTickSystem 每回合调用。暂停的效果不参与 Tick。
    pub fn get_tickable(&self) -> Vec<&EffectInstance> {
        self.effects
            .iter()
            .filter(|e| e.stage.can_tick() && e.tick_state.is_some() && !e.paused)
            .collect()
    }

    /// 检查是否已有同源效果（不变量 3.5）。
    pub fn has_duplicate(&self, def_id: &str, source_entity: &str) -> bool {
        self.effects
            .iter()
            .any(|e| e.def_id == def_id && e.source_entity == source_entity && e.stage.is_active())
    }

    /// 槽位满时 EffectSystem 返回 SlotLimitReached，由调用方决定是否移除旧效果腾出空间。
    pub fn is_full(&self) -> bool {
        self.active_count() as u32 >= self.max_effects
    }

    /// 等价于 active_count()，为符合常见命名习惯提供别名。
    pub fn count(&self) -> u32 {
        self.active_count() as u32
    }

    /// 没有任何 Applying 或 Active 阶段的效果时视为空。Expiring/Removed 不计入。
    pub fn is_empty(&self) -> bool {
        !self.effects.iter().any(|e| e.stage.is_active())
    }
}

impl Default for ActiveEffectContainer {
    fn default() -> Self {
        Self::new()
    }
}
