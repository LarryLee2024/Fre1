/// 目标选择模块：技能/能力的目标类型定义与解析
pub mod resolver;
pub mod types;
pub mod validator;

pub use types::*;
pub use validator::*;

use bevy::prelude::*;

/// 目标选择插件（注册 Reflect 类型）
pub struct TargetingPlugin;

impl Plugin for TargetingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SkillTargeting>();
    }
}
