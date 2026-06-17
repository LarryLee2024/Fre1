// 单位信息面板：显示选中单位的属性、资源条、技能、Buff

use crate::core::turn::AppState;
use crate::infrastructure::assets::CnFont;
use crate::infrastructure::localization::{CurrentLocale, LocalizationService};
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
    Equipment,
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
pub fn spawn_unit_info_panel(
    mut commands: Commands,
    theme: Res<UiTheme>,
    localization: Res<LocalizationService>,
    locale: Res<CurrentLocale>,
) {
    let select_unit_text = localization.resolve("ui.unit_info.select_unit", &locale.0, None);
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
            Visibility::Hidden,
        ))
        .insert(Name::new("UnitInfoPanel"))
        .with_children(|parent| {
            // 单位名称
            parent.spawn((
                Text::new(select_unit_text),
                TextFont {
                    font_size: theme.font_medium,
                    ..default()
                },
                TextColor(theme.text_primary),
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
            // 装备文本
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: theme.font_small,
                    ..default()
                },
                TextColor(theme.text_primary),
                PanelLabel::Equipment,
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
        text_font.font = cn_font.font_handle();
    }
}

/// 更新单位信息面板（读取 SelectedUnitView）
pub fn update_unit_info(
    view: Res<SelectedUnitView>,
    mut queries: ParamSet<(
        Query<&mut Visibility, With<UnitInfoPanel>>,
        Query<(&PanelLabel, &mut Text)>,
        Query<&mut Node, With<HpBarFill>>,
        Query<&mut Node, (With<MpBarFill>, Without<HpBarFill>)>,
        Query<&mut Node, (With<StaminaBarFill>, Without<HpBarFill>, Without<MpBarFill>)>,
        Query<(Entity, &mut Visibility), With<BuffsContainer>>,
    )>,
    mut commands: Commands,
    theme: Res<UiTheme>,
    localization: Res<LocalizationService>,
    locale: Res<CurrentLocale>,
) {
    if !view.is_changed() {
        return;
    }

    // 控制面板整体可见性
    let should_show = view.is_selected;
    for mut vis in queries.p0().iter_mut() {
        if should_show {
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }

    if !should_show {
        return;
    }

    // HP 进度条
    let hp_ratio = if view.max_hp > 0 {
        view.hp as f32 / view.max_hp as f32
    } else {
        0.0
    };
    for mut node in queries.p2().iter_mut() {
        node.width = Val::Percent(hp_ratio * 100.0);
    }

    // MP 进度条
    let mp_ratio = if view.max_mp > 0 {
        view.mp as f32 / view.max_mp as f32
    } else {
        0.0
    };
    for mut node in queries.p3().iter_mut() {
        node.width = Val::Percent(mp_ratio * 100.0);
    }

    // STA 进度条
    let sta_ratio = if view.max_stamina > 0 {
        view.stamina as f32 / view.max_stamina as f32
    } else {
        0.0
    };
    for mut node in queries.p4().iter_mut() {
        node.width = Val::Percent(sta_ratio * 100.0);
    }

    let core_attrs_header = localization.resolve("ui.unit_info.core_attrs", &locale.0, None);
    let combat_attrs_header = localization.resolve("ui.unit_info.combat_attrs", &locale.0, None);
    let support_attrs_header = localization.resolve("ui.unit_info.support_attrs", &locale.0, None);
    let skills_header = localization.resolve("ui.unit_info.skills", &locale.0, None);
    let traits_header = localization.resolve("ui.unit_info.traits", &locale.0, None);
    let equipment_header = localization.resolve("ui.unit_info.equipment", &locale.0, None);
    let none_text = localization.resolve("ui.unit_info.none", &locale.0, None);

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
        "{}\n{}  \n{}",
        core_attrs_header,
        core_line1.join("  "),
        core_line2.join("  ")
    );

    // 战斗属性
    let combat_line: Vec<String> = view
        .combat_attrs
        .iter()
        .map(|a| format!("{}:{}", a.label, a.value))
        .collect();
    let combat_text = format!("{}\n{}", combat_attrs_header, combat_line.join("  "));

    // 辅助属性
    let support_line: Vec<String> = view
        .support_attrs
        .iter()
        .map(|a| format!("{}:{}", a.label, a.value))
        .collect();
    let support_text = format!("{}\n{}", support_attrs_header, support_line.join("  "));

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
        "{}\n{}",
        skills_header,
        if skill_lines.is_empty() {
            none_text.clone()
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
        "{}\n{}",
        traits_header,
        if trait_lines.is_empty() {
            none_text.clone()
        } else {
            trait_lines.join("\n")
        }
    );

    // 统一更新所有面板文本
    for (label, mut text) in queries.p1().iter_mut() {
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
            PanelLabel::Equipment => {
                let lines: Vec<String> = view
                    .equipment
                    .iter()
                    .map(|e| {
                        if let (Some(name), Some(rarity)) = (&e.item_name, &e.rarity) {
                            format!("  {}: {} [{}]", e.slot_label, name, rarity)
                        } else {
                            format!("  {}: ──", e.slot_label)
                        }
                    })
                    .collect();
                **text = format!("{}\n{}", equipment_header, lines.join("\n"));
            }
        }
    }

    // Buff 彩色标签
    for (container_entity, mut vis) in queries.p5().iter_mut() {
        if view.buffs.is_empty() {
            *vis = Visibility::Hidden;
            commands.entity(container_entity).despawn_children();
        } else {
            *vis = Visibility::Visible;
            // 只在 Buff 数量变化时重建子节点
            commands.entity(container_entity).despawn_children();
            let status_header = localization.resolve("ui.unit_info.status_header", &locale.0, None);
            commands.entity(container_entity).with_children(|parent| {
                parent.spawn((
                    Text::new(format!("{} ", status_header)),
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

/// 单位信息面板插件
pub struct UnitInfoPlugin;

impl Plugin for UnitInfoPlugin {
    fn build(&self, app: &mut App) {
        use crate::core::turn::GameSet;
        app.add_systems(
            OnEnter(AppState::InGame),
            spawn_unit_info_panel.in_set(GameSet::Ui),
        )
        .add_systems(
            Update,
            (setup_ui_font, update_unit_info).run_if(in_state(AppState::InGame)),
        );
    }
}
