//! Pipeline 模块 — 回合战斗执行管线
//!
//! ADR-026 §十二：回合内 System 调度，整合 GAS 链各环节
//! - Ability → Targeting → Effect → Stacking → Execution → Modifier → Attribute → Tag → Cue → Replay

use bevy::prelude::*;

/// Pipeline 模块插件
pub struct BattlePipelinePlugin;

impl Plugin for BattlePipelinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GasPipelineState>()
            .add_systems(Update, run_gas_pipeline);
    }
}

/// GAS 链执行阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum GasPhase {
    /// 技能定义 + 施法校验
    Ability,
    /// 目标选取（纯函数）
    Targeting,
    /// 效果意图（Damage/Heal/Shield + 参数）
    Effect,
    /// 堆叠策略（覆写/刷新/叠加/上限）
    Stacking,
    /// 公式执行：计算具体数值
    Execution,
    /// 属性修改器挂载
    Modifier,
    /// 基础→派生属性刷新
    Attribute,
    /// 标签增减、状态判定
    Tag,
    /// 表现事件下发
    Cue,
    /// 指令+种子快照持久化
    Replay,
}

impl GasPhase {
    /// 获取所有阶段的有序列表
    pub fn all() -> &'static [GasPhase] {
        &[
            GasPhase::Ability,
            GasPhase::Targeting,
            GasPhase::Effect,
            GasPhase::Stacking,
            GasPhase::Execution,
            GasPhase::Modifier,
            GasPhase::Attribute,
            GasPhase::Tag,
            GasPhase::Cue,
            GasPhase::Replay,
        ]
    }

    /// 获取阶段索引
    pub fn index(&self) -> usize {
        Self::all().iter().position(|p| p == self).unwrap_or(0)
    }
}

/// GAS 管线运行状态 Resource
#[derive(Resource, Default)]
pub struct GasPipelineState {
    /// 当前执行阶段
    pub current_phase: Option<GasPhase>,
    /// 是否正在执行
    pub is_executing: bool,
}

/// GAS 管线执行 System
///
/// 在 TurnPhase::ExecuteAction 期间运行，按顺序执行各阶段
pub fn run_gas_pipeline(
    turn_phase: Res<State<crate::core::turn::TurnPhase>>,
    mut pipeline_state: ResMut<GasPipelineState>,
) {
    if *turn_phase.get() != crate::core::turn::TurnPhase::ExecuteAction {
        return;
    }

    // 标记管线正在执行
    if !pipeline_state.is_executing {
        pipeline_state.is_executing = true;
        pipeline_state.current_phase = Some(GasPhase::Ability);
    }

    // 当前阶段执行完成，推进到下一阶段
    if let Some(current) = pipeline_state.current_phase {
        let next_index = current.index() + 1;
        if next_index < GasPhase::all().len() {
            pipeline_state.current_phase = Some(GasPhase::all()[next_index]);
        } else {
            // 所有阶段完成
            pipeline_state.current_phase = None;
            pipeline_state.is_executing = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gas_phase_all_has_10_phases() {
        assert_eq!(GasPhase::all().len(), 10);
    }

    #[test]
    fn gas_phase_order() {
        let phases = GasPhase::all();
        assert_eq!(phases[0], GasPhase::Ability);
        assert_eq!(phases[1], GasPhase::Targeting);
        assert_eq!(phases[2], GasPhase::Effect);
        assert_eq!(phases[3], GasPhase::Stacking);
        assert_eq!(phases[4], GasPhase::Execution);
        assert_eq!(phases[5], GasPhase::Modifier);
        assert_eq!(phases[6], GasPhase::Attribute);
        assert_eq!(phases[7], GasPhase::Tag);
        assert_eq!(phases[8], GasPhase::Cue);
        assert_eq!(phases[9], GasPhase::Replay);
    }

    #[test]
    fn gas_phase_index() {
        assert_eq!(GasPhase::Ability.index(), 0);
        assert_eq!(GasPhase::Replay.index(), 9);
    }
}
