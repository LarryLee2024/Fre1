//! UI 主题系统（ThemePlugin + 样式令牌）
//!
//! 注册全局 `Theme` 资源和所有样式令牌类型
//! 以支持反射。必须在 UI 插件链中首先添加，
//! 以便 Widget 和界面在构造期间可以访问主题值。
//!
//! 参见 `docs/06-ui/01-architecture/architecture.md` §8.2

pub mod colors;
pub mod resource;
pub mod sizing;
pub mod spacing;
pub mod switch;
pub mod typography;

pub use colors::UiColors;
pub use resource::Theme;
pub use sizing::UiSizing;
pub use spacing::UiSpacing;
pub use switch::ThemeVariant;
pub use typography::UiTypography;

use bevy::prelude::*;

/// 注册全局 Theme 资源和反射类型的插件。
pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        // 插入默认（暗色）主题作为全局资源
        app.init_resource::<Theme>();

        // 注册所有主题类型以支持反射
        app.register_type::<UiColors>();
        app.register_type::<UiSizing>();
        app.register_type::<UiSpacing>();
        app.register_type::<UiTypography>();
    }
}
