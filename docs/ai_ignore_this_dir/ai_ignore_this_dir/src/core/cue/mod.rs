//! Cue 模块 — 表现层信号总线
//!
//! ADR-026 §四：Cue 作为业务层与表现层的唯一桥梁
//! - 业务逻辑只发射 CueEvent，不直接调用 UI/特效
//! - 表现层只订阅 CueEvent，零耦合回战斗逻辑
//! - CueEvent 只携带纯数据，不引用表现资源

pub mod emitter;
pub mod types;

pub use emitter::*;
pub use types::*;

use bevy::prelude::*;

/// Cue 模块插件
pub struct CuePlugin;

impl Plugin for CuePlugin {
    fn build(&self, app: &mut App) {
        // 注册所有 CueEvent 为 Bevy Message
        app.add_message::<CueDamage>()
            .add_message::<CueDeath>()
            .add_message::<CueHeal>()
            .add_message::<CueBuffApply>()
            .add_message::<CueBuffRemove>()
            .add_message::<CueShield>()
            .add_message::<CueSkillCast>()
            .add_message::<CueMovement>()
            .add_message::<CueStatusChange>()
            // 注册 CueEmitter Resource
            .init_resource::<CueEmitter>()
            // 注册 flush 系统
            .add_systems(Update, flush_cue_emitter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn Cue插件_注册事件() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(CuePlugin);

        // 验证插件注册成功（不会 panic）
    }

    #[test]
    fn 完整Cue场景() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(CuePlugin);

        // 模拟业务逻辑发送 Cue
        let mut emitter = app.world_mut().resource_mut::<CueEmitter>();
        emitter.emit_damage(Entity::from_bits(1), 50, true, Some(Entity::from_bits(2)));
        emitter.emit_death(Entity::from_bits(1), Some(Entity::from_bits(2)));

        assert!(emitter.has_pending());

        // 运行 flush 系统
        drop(emitter);
        app.update();

        // 验证事件已发送
        let emitter = app.world().resource::<CueEmitter>();
        assert!(!emitter.has_pending());
    }
}
