use crate::infra::save::events::*;
use crate::infra::save::plugin::SavePlugin;
use crate::infra::save::resources::SaveManager;
use bevy::prelude::*;
use std::sync::{Arc, Mutex};

#[test]
fn save_request_updates_manager() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SavePlugin);
    app.world_mut().trigger(SaveRequest {
        path: Some("test.fresave".into()),
        label: Some("Test".into()),
    });
    let m = app.world().resource::<SaveManager>();
    assert_eq!(
        m.current_save_path.as_ref().unwrap().to_string_lossy(),
        "test.fresave"
    );
    assert_eq!(m.metadata.label, "Test");
    assert!(!m.is_dirty);
}

#[test]
fn save_request_without_path_reuses_current() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SavePlugin);
    app.world_mut()
        .resource_mut::<SaveManager>()
        .current_save_path = Some("existing.fresave".into());
    app.world_mut().trigger(SaveRequest::default());
    let m = app.world().resource::<SaveManager>();
    assert_eq!(
        m.current_save_path.as_ref().unwrap().to_string_lossy(),
        "existing.fresave"
    );
}

#[test]
fn save_request_triggers_completed_event() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SavePlugin);
    let captured = Arc::new(Mutex::new(None::<SaveCompleted>));
    let c2 = captured.clone();
    app.add_observer(move |t: On<SaveCompleted>| {
        *c2.lock().unwrap() = Some(t.event().clone());
    });
    app.world_mut().trigger(SaveRequest {
        path: Some("t.fresave".into()),
        label: None,
    });
    // on_save_request uses commands.trigger() (deferred), flush with update()
    app.update();
    let saved = captured.lock().unwrap().take();
    assert!(saved.is_some());
    assert_eq!(saved.unwrap().path, "t.fresave");
}

#[test]
fn load_missing_file_triggers_error() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SavePlugin);
    let errored = Arc::new(Mutex::new(false));
    let e2 = errored.clone();
    app.add_observer(move |_: On<SaveError>| {
        *e2.lock().unwrap() = true;
    });
    app.world_mut().trigger(LoadRequest {
        path: "nonexistent.fresave".into(),
    });
    // on_load_request uses commands.trigger() (deferred), flush with update()
    app.update();
    assert!(
        *errored.lock().unwrap(),
        "SaveError should trigger for missing file"
    );
}
