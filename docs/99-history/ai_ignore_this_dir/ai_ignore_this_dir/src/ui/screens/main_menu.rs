/// 主菜单屏幕
use bevy::prelude::*;

use crate::infrastructure::assets::CnFont;
use crate::infrastructure::localization::{CurrentLocale, LocalizationService};
use crate::ui::events::UiCommand;
use crate::ui::theme::UiTheme;

/// 主菜单屏幕标记组件
#[derive(Component)]
pub struct MainMenuScreen;

#[derive(Component)]
pub struct StartGameButton;

#[derive(Component)]
pub struct ContinueButton;

#[derive(Component)]
pub struct QuitButton;

/// 生成主菜单
pub fn spawn_main_menu(
    mut commands: Commands,
    theme: Res<UiTheme>,
    cn_font: Res<CnFont>,
    localization: Res<LocalizationService>,
    locale: Res<CurrentLocale>,
) {
    let title_font = cn_font.text_font(theme.font_title);
    let button_font = cn_font.text_font(theme.font_menu);
    let small_font = cn_font.text_font(theme.font_small);

    let title_text = localization.resolve("ui.main_menu.title", &locale.0, None);
    let start_text = localization.resolve("ui.main_menu.start_game", &locale.0, None);
    let continue_text = localization.resolve("ui.main_menu.continue_game", &locale.0, None);
    let quit_text = localization.resolve("ui.main_menu.quit_game", &locale.0, None);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(theme.menu_bg),
            MainMenuScreen,
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new(title_text),
                title_font,
                TextColor(theme.menu_title_color),
                Node {
                    margin: UiRect::bottom(Val::Px(60.0)),
                    ..default()
                },
            ));

            // "开始游戏"按钮
            spawn_menu_button(parent, &start_text, &theme, &button_font, StartGameButton);

            // "继续战役"按钮
            spawn_menu_button(parent, &continue_text, &theme, &button_font, ContinueButton);

            // "退出游戏"按钮
            spawn_menu_button(parent, &quit_text, &theme, &button_font, QuitButton);

            // 版本号
            parent.spawn((
                Text::new("v0.1.0"),
                small_font,
                TextColor(theme.text_secondary),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(16.0),
                    ..default()
                },
            ));
        });
}

/// 生成一个菜单按钮
fn spawn_menu_button(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    label: &str,
    theme: &UiTheme,
    font: &TextFont,
    marker: impl Component,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(theme.menu_button_width),
                height: Val::Px(theme.menu_button_height),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(theme.menu_button_bg),
            marker,
        ))
        .with_child((
            Text::new(label),
            font.clone(),
            TextColor(theme.text_primary),
        ));
}

/// 处理主菜单按钮点击
pub fn handle_main_menu_buttons(
    mut cmd_events: MessageWriter<UiCommand>,
    buttons: Query<(&Interaction, Entity), (Changed<Interaction>, With<Button>)>,
    start_query: Query<Entity, With<StartGameButton>>,
    continue_query: Query<Entity, With<ContinueButton>>,
    quit_query: Query<Entity, With<QuitButton>>,
) {
    for (interaction, entity) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if start_query.contains(entity) {
            cmd_events.write(UiCommand::StartGame);
        } else if continue_query.contains(entity) {
            cmd_events.write(UiCommand::ContinueGame);
        } else if quit_query.contains(entity) {
            cmd_events.write(UiCommand::QuitGame);
        }
    }
}

/// 清理主菜单
pub fn cleanup_main_menu(mut commands: Commands, screens: Query<Entity, With<MainMenuScreen>>) {
    for entity in &screens {
        commands.entity(entity).despawn();
    }
}
