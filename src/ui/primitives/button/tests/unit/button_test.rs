//! Button Widget 测试
//!
//! 测试验证 Button Widget 契约：
//! - 工厂创建正确的实体结构（属性、UI 组件）
//! - 变体颜色匹配主题令牌
//! - 禁用状态阻止交互并设置正确颜色
//! - ButtonInteraction 跟踪悬停/按下/点击状态
//! - ButtonClicked 事件在点击释放时触发
//!
//! 这些是行为测试 — 它们验证输入->输出，
//! 不测试实现细节（内部字段、中间状态）。

use bevy::ecs::observer::On;
use bevy::prelude::*;
use bevy::ui::Interaction;

use crate::ui::primitives::button::ButtonPlugin;
use crate::ui::primitives::button::components::{ButtonInteraction, ButtonState, ButtonVariant};
use crate::ui::primitives::button::events::ButtonClicked;
use crate::ui::primitives::button::factory::spawn_button;
use crate::ui::theme::{Theme, ThemePlugin};

// ── 辅助函数 ──

/// 构建注册了 Theme + Button 插件的最小 App。
fn button_app() -> App {
    let mut app = App::new();
    app.add_plugins(ThemePlugin);
    app.add_plugins(ButtonPlugin);
    app
}

/// 生成按钮并运行一帧以刷新延迟命令。
fn spawn_button_in_app(app: &mut App, label: &str, variant: ButtonVariant) -> Entity {
    let theme = app.world().resource::<Theme>().clone();
    let entity = {
        let mut commands = app.world_mut().commands();
        spawn_button(&mut commands, &theme, label, variant)
    };
    app.update();
    entity
}

// ═══════════════════════════════════════════════════════════════════
//  工厂测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn factory_spawns_button_with_correct_props() {
    let mut app = button_app();
    let entity = spawn_button_in_app(&mut app, "Click Me", ButtonVariant::Primary);

    let state = app.world().entity(entity).get::<ButtonState>().unwrap();
    assert_eq!(state.label, "Click Me", "label should match spawn input");
    assert_eq!(
        state.variant,
        ButtonVariant::Primary,
        "variant should match spawn input"
    );
    assert!(!state.disabled, "factory creates enabled button by default");
}

#[test]
fn factory_spawns_button_with_default_interaction() {
    let mut app = button_app();
    let entity = spawn_button_in_app(&mut app, "Test", ButtonVariant::Primary);

    let interaction = app
        .world()
        .entity(entity)
        .get::<ButtonInteraction>()
        .unwrap();
    assert!(!interaction.hovered, "initial hovered must be false");
    assert!(!interaction.pressed, "initial pressed must be false");
    assert!(
        !interaction.just_clicked,
        "initial just_clicked must be false"
    );
}

#[test]
fn factory_spawns_button_with_ui_components() {
    let mut app = button_app();
    let entity = spawn_button_in_app(&mut app, "Test", ButtonVariant::Primary);

    assert!(
        app.world().entity(entity).contains::<Node>(),
        "factory should spawn a Node component"
    );
    assert!(
        app.world().entity(entity).contains::<BackgroundColor>(),
        "factory should spawn a BackgroundColor component"
    );
}

// ═══════════════════════════════════════════════════════════════════
//  变体颜色测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn primary_variant_uses_accent_color() {
    let mut app = button_app();
    let theme = app.world().resource::<Theme>().clone();
    let entity = spawn_button_in_app(&mut app, "OK", ButtonVariant::Primary);

    let bg = app.world().entity(entity).get::<BackgroundColor>().unwrap();
    assert_eq!(
        bg.0, theme.colors.accent_primary,
        "Primary variant should use accent_primary color"
    );
}

#[test]
fn secondary_variant_uses_surface_color() {
    let mut app = button_app();
    let theme = app.world().resource::<Theme>().clone();
    let entity = spawn_button_in_app(&mut app, "OK", ButtonVariant::Secondary);

    let bg = app.world().entity(entity).get::<BackgroundColor>().unwrap();
    assert_eq!(
        bg.0, theme.colors.surface_secondary,
        "Secondary variant should use surface_secondary color"
    );
}

#[test]
fn danger_variant_uses_negative_feedback_color() {
    let mut app = button_app();
    let theme = app.world().resource::<Theme>().clone();
    let entity = spawn_button_in_app(&mut app, "Delete", ButtonVariant::Danger);

    let bg = app.world().entity(entity).get::<BackgroundColor>().unwrap();
    assert_eq!(
        bg.0, theme.colors.feedback_negative,
        "Danger variant should use feedback_negative color"
    );
}

#[test]
fn ghost_variant_uses_transparent_color() {
    let mut app = button_app();
    let entity = spawn_button_in_app(&mut app, "Cancel", ButtonVariant::Ghost);

    let bg = app.world().entity(entity).get::<BackgroundColor>().unwrap();
    assert_eq!(
        bg.0,
        Color::NONE,
        "Ghost variant should have transparent background"
    );
}

// ═══════════════════════════════════════════════════════════════════
//  禁用态测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn disabled_button_ignores_press_interaction() {
    let mut app = button_app();
    let entity = spawn_button_in_app(&mut app, "Test", ButtonVariant::Primary);

    // Mark disabled AND apply a press simultaneously
    {
        let world = app.world_mut();
        world.entity_mut(entity).insert(ButtonState {
            variant: ButtonVariant::Primary,
            disabled: true,
            label: "Test".into(),
        });
        world.entity_mut(entity).insert(Interaction::Pressed);
    }
    app.update();

    let interaction = app
        .world()
        .entity(entity)
        .get::<ButtonInteraction>()
        .unwrap();
    assert!(
        !interaction.hovered,
        "disabled button must not become hovered"
    );
    assert!(
        !interaction.pressed,
        "disabled button must not become pressed"
    );
    assert!(
        !interaction.just_clicked,
        "disabled button must not become just_clicked"
    );
}

#[test]
fn disabled_button_uses_disabled_surface_color() {
    let mut app = button_app();
    let theme = app.world().resource::<Theme>().clone();
    let entity = spawn_button_in_app(&mut app, "Test", ButtonVariant::Primary);

    {
        let world = app.world_mut();
        world.entity_mut(entity).insert(ButtonState {
            variant: ButtonVariant::Primary,
            disabled: true,
            label: "Test".into(),
        });
    }
    app.update();

    let bg = app.world().entity(entity).get::<BackgroundColor>().unwrap();
    assert_eq!(
        bg.0, theme.colors.surface_disabled,
        "disabled button should use surface_disabled color"
    );
}

// ═══════════════════════════════════════════════════════════════════
//  交互状态测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn button_tracks_hover_state() {
    let mut app = button_app();
    let entity = spawn_button_in_app(&mut app, "Test", ButtonVariant::Primary);

    // -- Hover on --
    app.world_mut()
        .entity_mut(entity)
        .insert(Interaction::Hovered);
    app.update();
    let interaction = app
        .world()
        .entity(entity)
        .get::<ButtonInteraction>()
        .unwrap();
    assert!(
        interaction.hovered,
        "hovered should be true when Interaction is Hovered"
    );
    assert!(!interaction.pressed, "should not be pressed when hovering");

    // -- Hover off --
    app.world_mut().entity_mut(entity).insert(Interaction::None);
    app.update();
    let interaction = app
        .world()
        .entity(entity)
        .get::<ButtonInteraction>()
        .unwrap();
    assert!(
        !interaction.hovered,
        "hovered should clear when Interaction is None"
    );
}

#[test]
fn button_tracks_press_state() {
    let mut app = button_app();
    let entity = spawn_button_in_app(&mut app, "Test", ButtonVariant::Primary);

    app.world_mut()
        .entity_mut(entity)
        .insert(Interaction::Pressed);
    app.update();

    let interaction = app
        .world()
        .entity(entity)
        .get::<ButtonInteraction>()
        .unwrap();
    assert!(
        interaction.pressed,
        "pressed should be true when Interaction is Pressed"
    );
    assert!(!interaction.hovered, "should not be hovered when pressed");
}

#[test]
fn button_detects_click_on_press_release() {
    let mut app = button_app();
    let entity = spawn_button_in_app(&mut app, "Test", ButtonVariant::Primary);

    // Press
    app.world_mut()
        .entity_mut(entity)
        .insert(Interaction::Pressed);
    app.update();

    // Release
    app.world_mut().entity_mut(entity).insert(Interaction::None);
    app.update();

    let interaction = app
        .world()
        .entity(entity)
        .get::<ButtonInteraction>()
        .unwrap();
    assert!(
        interaction.just_clicked,
        "just_clicked should be true after press-then-release cycle"
    );
}

#[test]
fn just_clicked_resets_after_one_frame() {
    let mut app = button_app();
    let entity = spawn_button_in_app(&mut app, "Test", ButtonVariant::Primary);

    // Complete click cycle: press then release
    app.world_mut()
        .entity_mut(entity)
        .insert(Interaction::Pressed);
    app.update();
    app.world_mut().entity_mut(entity).insert(Interaction::None);
    app.update();

    // One more frame: just_clicked should reset to false
    app.update();
    let interaction = app
        .world()
        .entity(entity)
        .get::<ButtonInteraction>()
        .unwrap();
    assert!(
        !interaction.just_clicked,
        "just_clicked must reset to false after one frame"
    );
}

// ═══════════════════════════════════════════════════════════════════
//  事件测试
// ═══════════════════════════════════════════════════════════════════

/// 跟踪最后点击的实体用于事件断言。
#[derive(Resource, Default)]
struct ClickTracker(Option<Entity>);

#[test]
fn button_clicked_event_fires_on_click_release() {
    let mut app = button_app();
    app.init_resource::<ClickTracker>();
    let entity = spawn_button_in_app(&mut app, "Test", ButtonVariant::Primary);

    // Register observer matching the production API
    app.add_observer(|on: On<ButtonClicked>, mut tracker: ResMut<ClickTracker>| {
        tracker.0 = Some(on.event().entity);
    });

    // Press then release
    app.world_mut()
        .entity_mut(entity)
        .insert(Interaction::Pressed);
    app.update();
    app.world_mut().entity_mut(entity).insert(Interaction::None);
    app.update();

    let tracker = app.world().resource::<ClickTracker>();
    assert_eq!(
        tracker.0,
        Some(entity),
        "ButtonClicked event should fire with the correct entity on release"
    );
}

#[test]
fn button_clicked_event_does_not_fire_on_hover_alone() {
    let mut app = button_app();
    app.init_resource::<ClickTracker>();
    let entity = spawn_button_in_app(&mut app, "Test", ButtonVariant::Primary);

    app.add_observer(|on: On<ButtonClicked>, mut tracker: ResMut<ClickTracker>| {
        tracker.0 = Some(on.event().entity);
    });

    // Hover does NOT click
    app.world_mut()
        .entity_mut(entity)
        .insert(Interaction::Hovered);
    app.update();

    let tracker = app.world().resource::<ClickTracker>();
    assert_eq!(
        tracker.0, None,
        "hover alone must not trigger ButtonClicked"
    );
}
