// 单位信息面板：显示选中单位的属性、资源条、技能、Buff

use crate::assets::CnFont;
use crate::character::{HpBarFg, Unit};
use crate::core::attribute::{AttributeKind, Attributes};
use crate::turn::AppState;
use crate::ui::theme::UiTheme;
use crate::ui::view_models::SelectedUnitView;
use crate::ui::widgets::layout::*;
use crate::ui::widgets::resource_bar::spawn_resource_bar;
use bevy::prelude::*;

/// 面板文本标签（统一枚举，避免多个 &mut Text Query 冲突）
#[derive(Component)]
pub enum PanelLabel {
    UnitName,
    UnitInfo,
    Hp,
    Mp,
    Stamina,
    CoreAttrs,
    CombatAttrs,
    SupportAttrs,
    Skills,
    Traits,
}

/// 单位信息面板根节点
#[derive(Component)]
pub struct UnitInfoPanel;

/// HP 进度条填充
#[derive(Component)]
pub struct HpBarFill;

/// MP 进度条填充
#[derive(Component)]
pub struct MpBarFill;

/// 耐力进度条填充
#[derive(Component)]
pub struct StaminaBarFill;

/// Buff 容器节点
#[derive(Component)]
pub struct BuffsContainer;

/// 生成单位信息面板
pub fn spawn_unit_info_panel(mut commands: Commands, theme: Res<UiTheme>) {
    commands
        .spawn((
            panel_bottom(
                &theme,
                theme.gap_large,
                theme.gap_large,
                theme.unit_panel_width,
            ),
            BackgroundColor(theme.panel_bg),
            UnitInfoPanel,
        ))
        .with_children(|parent| {
            // 单位名称
            parent.spawn((
                Text::new("选择一个单位"),
                TextFont {
                    font_size: theme.font_medium,
                    ..default()
                },
                TextColor(Color::WHITE),
                PanelLabel::UnitName,
            ));
            // 种族/职业信息
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: theme.font_small,
                    ..default()
                },
                TextColor(theme.text_secondary),
                PanelLabel::UnitInfo,
            ));
            // HP/MP/STA 资源条
            spawn_resource_bar(
                parent,
                &theme,
                "HP",
                theme.hp_bar_color,
                HpBarFill,
                PanelLabel::Hp,
            );
            spawn_resource_bar(
                parent,
                &theme,
                "MP",
                theme.mp_bar_color,
                MpBarFill,
                PanelLabel::Mp,
            );
            spawn_resource_bar(
                parent,
                &theme,
                "STA",
                theme.stamina_bar_color,
                StaminaBarFill,
                PanelLabel::Stamina,
            );
            // 属性文本
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: theme.font_small,
                    ..default()
                },
                TextColor(theme.text_secondary),
                PanelLabel::CoreAttrs,
            ));
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: theme.font_small,
                    ..default()
                },
                TextColor(theme.text_primary),
                PanelLabel::CombatAttrs,
            ));
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: theme.font_small,
                    ..default()
                },
                TextColor(theme.text_secondary),
                PanelLabel::SupportAttrs,
            ));
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: theme.font_small,
                    ..default()
                },
                TextColor(theme.text_skill),
                PanelLabel::Skills,
            ));
            // Trait 文本
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: theme.font_small,
                    ..default()
                },
                TextColor(theme.text_secondary),
                PanelLabel::Traits,
            ));
            // Buff 容器
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(theme.gap_small),
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                BuffsContainer,
            ));
        });
}

/// 设置中文字体
pub fn setup_ui_font(cn_font: Res<CnFont>, mut query: Query<&mut TextFont, With<Node>>) {
    for mut text_font in &mut query {
        text_font.font = cn_font.handle.clone();
    }
}

/// 更新单位信息面板（读取 SelectedUnitView）
pub fn update_unit_info(
    view: Res<SelectedUnitView>,
    mut panel_texts: Query<(&PanelLabel, &mut Text)>,
    mut hp_fill_query: Query<&mut Node, With<HpBarFill>>,
    mut mp_fill_query: Query<&mut Node, (With<MpBarFill>, Without<HpBarFill>)>,
    mut stamina_fill_query: Query<
        &mut Node,
        (With<StaminaBarFill>, Without<HpBarFill>, Without<MpBarFill>),
    >,
    mut buffs_container_query: Query<(Entity, &mut Visibility), With<BuffsContainer>>,
    mut commands: Commands,
    theme: Res<UiTheme>,
) {
    if !view.is_changed() {
        return;
    }

    if !view.is_selected {
        // 隐藏面板内容
        for (label, mut text) in &mut panel_texts {
            match label {
                PanelLabel::UnitName => **text = "选择一个单位".to_string(),
                PanelLabel::Hp | PanelLabel::Mp | PanelLabel::Stamina => **text = "0/0".to_string(),
                _ => **text = String::new(),
            }
        }
        for mut node in &mut hp_fill_query {
            node.width = Val::Percent(0.0);
        }
        for mut node in &mut mp_fill_query {
            node.width = Val::Percent(0.0);
        }
        for mut node in &mut stamina_fill_query {
            node.width = Val::Percent(0.0);
        }
        for (container_entity, mut vis) in &mut buffs_container_query {
            *vis = Visibility::Hidden;
            commands.entity(container_entity).despawn_children();
        }
        return;
    }

    // HP 进度条
    let hp_ratio = if view.max_hp > 0 {
        view.hp as f32 / view.max_hp as f32
    } else {
        0.0
    };
    for mut node in &mut hp_fill_query {
        node.width = Val::Percent(hp_ratio * 100.0);
    }

    // MP 进度条
    let mp_ratio = if view.max_mp > 0 {
        view.mp as f32 / view.max_mp as f32
    } else {
        0.0
    };
    for mut node in &mut mp_fill_query {
        node.width = Val::Percent(mp_ratio * 100.0);
    }

    // STA 进度条
    let sta_ratio = if view.max_stamina > 0 {
        view.stamina as f32 / view.max_stamina as f32
    } else {
        0.0
    };
    for mut node in &mut stamina_fill_query {
        node.width = Val::Percent(sta_ratio * 100.0);
    }

    // 核心属性（两行四列）
    let core_line1: Vec<String> = view
        .core_attrs
        .iter()
        .take(4)
        .map(|a| format!("{}:{}", a.label, a.value))
        .collect();
    let core_line2: Vec<String> = view
        .core_attrs
        .iter()
        .skip(4)
        .take(4)
        .map(|a| format!("{}:{}", a.label, a.value))
        .collect();
    let core_text = format!(
        "── 核心属性 ──\n{}  \n{}",
        core_line1.join("  "),
        core_line2.join("  ")
    );

    // 战斗属性
    let combat_line: Vec<String> = view
        .combat_attrs
        .iter()
        .map(|a| format!("{}:{}", a.label, a.value))
        .collect();
    let combat_text = format!("── 战斗属性 ──\n{}", combat_line.join("  "));

    // 辅助属性
    let support_line: Vec<String> = view
        .support_attrs
        .iter()
        .map(|a| format!("{}:{}", a.label, a.value))
        .collect();
    let support_text = format!("── 辅助属性 ──\n{}", support_line.join("  "));

    // 技能（带详细信息）
    let skill_lines: Vec<String> = view
        .skills
        .iter()
        .map(|s| {
            let cd = if s.cooldown > 0 {
                format!(" CD{}", s.cooldown)
            } else {
                String::new()
            };
            let mp = if s.cost_mp > 0 {
                format!(" MP{}", s.cost_mp)
            } else {
                String::new()
            };
            let range = if s.range > 0 {
                format!(" 射程{}", s.range)
            } else {
                String::new()
            };
            format!("  {}{}{}{} {}", s.name, mp, cd, range, s.description)
        })
        .collect();
    let skills_text = format!(
        "── 技能 ──\n{}",
        if skill_lines.is_empty() {
            "无".to_string()
        } else {
            skill_lines.join("\n")
        }
    );

    // Traits
    let trait_lines: Vec<String> = view
        .traits
        .iter()
        .map(|t| format!("  {} - {}", t.name, t.description))
        .collect();
    let traits_text = format!(
        "── 特性 ──\n{}",
        if trait_lines.is_empty() {
            "无".to_string()
        } else {
            trait_lines.join("\n")
        }
    );

    // 统一更新所有面板文本
    for (label, mut text) in &mut panel_texts {
        match label {
            PanelLabel::UnitName => **text = view.name.clone(),
            PanelLabel::UnitInfo => {
                let mut parts: Vec<String> = Vec::new();
                if !view.race.is_empty() {
                    parts.push(view.race.clone());
                }
                if !view.class.is_empty() {
                    parts.push(view.class.clone());
                }
                **text = if parts.is_empty() {
                    String::new()
                } else {
                    parts.join(" · ")
                };
            }
            PanelLabel::Hp => **text = format!("{}/{}", view.hp, view.max_hp),
            PanelLabel::Mp => **text = format!("{}/{}", view.mp, view.max_mp),
            PanelLabel::Stamina => **text = format!("{}/{}", view.stamina, view.max_stamina),
            PanelLabel::CoreAttrs => **text = core_text.clone(),
            PanelLabel::CombatAttrs => **text = combat_text.clone(),
            PanelLabel::SupportAttrs => **text = support_text.clone(),
            PanelLabel::Skills => **text = skills_text.clone(),
            PanelLabel::Traits => **text = traits_text.clone(),
        }
    }

    // Buff 彩色标签
    for (container_entity, mut vis) in &mut buffs_container_query {
        if view.buffs.is_empty() {
            *vis = Visibility::Hidden;
            commands.entity(container_entity).despawn_children();
        } else {
            *vis = Visibility::Visible;
            commands.entity(container_entity).despawn_children();
            commands.entity(container_entity).with_children(|parent| {
                parent.spawn((
                    Text::new("── 状态 ── "),
                    TextFont {
                        font_size: theme.font_small,
                        ..default()
                    },
                    TextColor(theme.text_secondary),
                ));
                for buff in &view.buffs {
                    let color = if buff.is_buff {
                        theme.buff_color
                    } else {
                        theme.debuff_color
                    };
                    parent.spawn((
                        Text::new(format!("[{}·{}t]", buff.name, buff.remaining_turns)),
                        TextFont {
                            font_size: theme.font_small,
                            ..default()
                        },
                        TextColor(color),
                    ));
                }
            });
        }
    }
}

/// 更新 HP 条宽度（地图上单位的 HP 条）
pub fn update_hp_bars(
    units: Query<(&Attributes, &Children), With<Unit>>,
    mut hp_fgs: Query<&mut Sprite, With<HpBarFg>>,
    map: Res<crate::map::GameMap>,
) {
    let bar_width = map.tile_size * 0.6;
    for (attrs, children) in &units {
        let hp = attrs.get(AttributeKind::Hp);
        let max_hp = attrs.get(AttributeKind::MaxHp);
        let ratio = if max_hp > 0.0 {
            (hp / max_hp).max(0.0)
        } else {
            0.0
        };
        for child in children.iter() {
            if let Ok(mut sprite) = hp_fgs.get_mut(child) {
                sprite.custom_size = Some(Vec2::new(bar_width * ratio, 4.0));
            }
        }
    }
}

/// 单位信息面板插件
pub struct UnitInfoPlugin;

impl Plugin for UnitInfoPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::GameSet;
        app.add_systems(
            OnEnter(AppState::InGame),
            spawn_unit_info_panel.in_set(GameSet::Ui),
        )
        .add_systems(
            Update,
            (setup_ui_font, update_unit_info, update_hp_bars).run_if(in_state(AppState::InGame)),
        );
    }
}
