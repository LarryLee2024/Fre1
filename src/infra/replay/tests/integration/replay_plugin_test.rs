use bevy::app::App;
use bevy::prelude::*;

use crate::infra::replay::plugin::ReplayPlugin;
use crate::infra::replay::resources::{
    DeterministicRng, FrameCounter, PlaybackSession, RecordingSession, ReplayModeGuard,
};

/// ReplayPlugin 注册所有必需的 Resource。
#[test]
fn replay_plugin_registers_all_resources() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ReplayPlugin);

    let world = app.world();
    assert!(
        world.contains_resource::<DeterministicRng>(),
        "DeterministicRng resource should be registered"
    );
    assert!(
        world.contains_resource::<ReplayModeGuard>(),
        "ReplayModeGuard resource should be registered"
    );
    assert!(
        world.contains_resource::<RecordingSession>(),
        "RecordingSession resource should be registered"
    );
    assert!(
        world.contains_resource::<PlaybackSession>(),
        "PlaybackSession resource should be registered"
    );
    assert!(
        world.contains_resource::<FrameCounter>(),
        "FrameCounter resource should be registered"
    );
}

/// ReplayPlugin 注册的 Resource 具有正确的默认值。
#[test]
fn replay_plugin_resources_have_correct_defaults() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ReplayPlugin);

    let world = app.world();
    let rng = world.resource::<DeterministicRng>();
    let seeds = rng.0.get_all_seeds();
    assert_eq!(seeds.combat_seed, 0, "default RNG seed should be 0");

    let guard = world.resource::<ReplayModeGuard>();
    assert!(!guard.0.is_replay, "default mode guard should be normal");

    let counter = world.resource::<FrameCounter>();
    assert_eq!(counter.0, 0, "frame counter should start at 0");
}

/// frame_counter_system 每帧递增计数器（PreUpdate）。
#[test]
fn frame_counter_increments_each_update() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ReplayPlugin);

    assert_eq!(app.world().resource::<FrameCounter>().0, 0);

    app.update();
    assert_eq!(app.world().resource::<FrameCounter>().0, 1);

    app.update();
    assert_eq!(app.world().resource::<FrameCounter>().0, 2);

    app.update();
    assert_eq!(app.world().resource::<FrameCounter>().0, 3);
}

/// frame_counter_system 不 panic，即使没有其他系统。
#[test]
fn frame_counter_system_runs_without_panicking() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ReplayPlugin);

    // 运行多帧确保无 panic
    for _ in 0..10 {
        app.update();
    }
    assert_eq!(app.world().resource::<FrameCounter>().0, 10);
}
