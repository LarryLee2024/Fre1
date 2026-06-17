use crate::infra::save::plugin::SavePlugin;
use crate::infra::save::resources::*;
use bevy::prelude::*;

#[test]
fn save_plugin_registers_resources() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SavePlugin);
    let w = app.world();
    assert!(w.contains_resource::<SaveManager>());
    assert!(w.contains_resource::<AutoSaveConfig>());
    assert!(w.contains_resource::<EntityRemapper>());
}

#[test]
fn save_plugin_resources_have_correct_defaults() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SavePlugin);
    let manager = app.world().resource::<SaveManager>();
    assert!(manager.current_save_path.is_none());
    assert_eq!(manager.save_version, 1);
}

#[test]
fn save_plugin_runs_without_panicking() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SavePlugin);
    for _ in 0..5 {
        app.update();
    }
}
