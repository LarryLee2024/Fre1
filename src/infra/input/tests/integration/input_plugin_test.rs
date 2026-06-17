use bevy::app::App;
use bevy::prelude::*;

use crate::infra::input::action::{InputAction, InputMap};
use crate::infra::input::plugin::InputPlugin;
use crate::infra::input::resources::InputState;

/// InputPlugin 注册所有必需的 Resource。
#[test]
fn input_plugin_registers_resources() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(InputPlugin);

    let world = app.world();
    assert!(
        world.contains_resource::<InputMap>(),
        "InputMap resource should be registered"
    );
    assert!(
        world.contains_resource::<InputState>(),
        "InputState resource should be registered"
    );
}

/// InputPlugin 注册 InputMap 的默认键盘绑定。
#[test]
fn input_plugin_default_input_map_has_move_bindings() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(InputPlugin);

    let input_map = app.world().resource::<InputMap>();
    assert_eq!(
        input_map.get_keyboard_action(&KeyCode::KeyW),
        Some(InputAction::MoveUp)
    );
    assert_eq!(
        input_map.get_keyboard_action(&KeyCode::KeyS),
        Some(InputAction::MoveDown)
    );
}

/// collect_input_actions system 在 PreUpdate 中运行并更新 InputState。
/// 手动注册 ButtonInput Resource 以模拟按键输入环境。
#[test]
fn input_plugin_systems_run_without_panicking() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // ButtonInput resources are required by collect_input_actions
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.add_plugins(InputPlugin);

    // 运行一帧 — PreUpdate 中的 collect_input_actions 不应 panic
    app.update();

    let input_state = app.world().resource::<InputState>();
    // 没有按键按下，所有动作列表应为空
    assert!(input_state.pressed_actions.is_empty());
    assert!(input_state.just_pressed_actions.is_empty());
    assert!(input_state.just_released_actions.is_empty());
}
