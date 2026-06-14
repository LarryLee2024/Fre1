//! LocalizationPlugin：本地化插件

use bevy::prelude::*;

use super::cache::LocalizedTextCache;
use super::locale::{CurrentLocale, Locale};
use super::service::LocalizationService;
use super::systems;

/// 本地化插件
pub struct LocalizationPlugin;

impl Plugin for LocalizationPlugin {
    fn build(&self, app: &mut App) {
        // 注册 Resource
        app.insert_resource(CurrentLocale::default())
            .insert_resource(LocalizationService::new(Locale::default()))
            .insert_resource(LocalizedTextCache::new());

        // 注册 Message
        app.add_message::<super::LanguageChangedMessage>();

        // 注册系统
        app.add_systems(Startup, systems::initialize_localization)
            .add_systems(
                Update,
                (
                    systems::on_language_changed,
                    systems::resolve_localized_texts,
                )
                    .chain(),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_registers_resources() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(LocalizationPlugin);

        assert!(app.world().get_resource::<CurrentLocale>().is_some());
        assert!(app.world().get_resource::<LocalizationService>().is_some());
    }
}
