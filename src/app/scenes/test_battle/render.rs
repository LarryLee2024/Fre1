//! 占位视觉渲染系统 — 零外部美术资产的战斗场景渲染
//!
//! 本模块提供测试战斗场景的占位视觉效果：
//! - 单位用彩色方块表示（蓝色=玩家，红色=敌人）
//! - 棋盘格网格背景
//! - 2D 摄像机
//!
//! ⚠️ 视觉系统与逻辑彻底分离：
//!   - `spawn.rs` 处理纯逻辑组件（HitPoints, GridPos 等）
//!   - `render.rs` 只添加视觉组件（Sprite, Transform 等）
//!   - 替换为正式渲染管线时，只需替换本模块

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::*;

use crate::core::domains::combat::components::{
    CombatParticipant, HitPoints, TeamId, UnitIdComponent,
};
use crate::core::domains::tactical::components::GridPos;
use crate::infra::picking::selection::Selection;

use super::spawn::TestBattleScenario;

// ─── 颜色常量（易于替换为 Theme 资源引用） ─────────────────────────

/// 玩家单位颜色
const PLAYER_COLOR: Color = Color::srgb(0.2, 0.5, 0.9);
/// 敌方单位颜色
const ENEMY_COLOR: Color = Color::srgb(0.9, 0.2, 0.2);
/// 其他单位颜色
const NEUTRAL_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);

/// 悬停高亮颜色（金色）
const HIGHLIGHT_COLOR: Color = Color::srgb(1.0, 0.9, 0.4);

/// 选中高亮颜色（青色）
const SELECTED_COLOR: Color = Color::srgb(0.2, 1.0, 1.0);

/// 网格浅色（棋盘格亮格）
const GRID_LIGHT: Color = Color::srgb(0.2, 0.2, 0.25);
/// 网格深色（棋盘格暗格）
const GRID_DARK: Color = Color::srgb(0.15, 0.15, 0.2);

/// 单位精灵像素大小
const UNIT_SIZE: f32 = 60.0;

// ─── 纹理创建（共享的 1x1 白图） ──────────────────────────────────

/// 创建一张 1x1 纯白纹理，供 Sprite 着色使用。
///
/// 所有单位共享同一张纹理，通过 Sprite.color 实现不同颜色。
fn create_white_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let image = Image::new(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![255u8, 255u8, 255u8, 255u8],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    images.add(image)
}

// ─── 系统：单位点击处理器 ──────────────────────────────────────

/// 单位点击事件处理器
///
/// 左键选择单位，右键取消选择。
/// 每个单位实体通过 `.observe(on_unit_click)` 注册此处理器。
fn on_unit_click(
    ev: On<Pointer<Click>>,
    unit_ids: Query<&UnitIdComponent>,
    mut selection: ResMut<Selection>,
) {
    println!(
        "[DEBUG] on_unit_click fired! button={:?} entity={:?}",
        ev.event().button,
        ev.event_target()
    );

    // 右键点击 → 取消选择
    if ev.event().button == PointerButton::Secondary {
        selection.selected_unit = None;
        println!("[DEBUG] Right-click deselected");
        return;
    }

    let Ok(uid) = unit_ids.get(ev.event_target()) else {
        println!("[DEBUG] Clicked entity has no UnitIdComponent");
        return;
    };
    selection.selected_unit = Some(ev.event_target());
    println!(
        "[Picking] Unit selected: {} (entity={:?})",
        uid.id,
        ev.event_target()
    );
}

/// 单位悬停高亮处理器（Phase 3）
///
/// 鼠标移入单位 Sprite 时切换为高亮颜色（金色）。
fn on_unit_hover(ev: On<Pointer<Over>>, mut sprites: Query<&mut Sprite>) {
    let Ok(mut sprite) = sprites.get_mut(ev.event_target()) else {
        return;
    };
    sprite.color = HIGHLIGHT_COLOR;
}

/// 单位悬停恢复处理器
///
/// 鼠标移出时：如果单位被选中则恢复为选中色，否则恢复为队伍色。
fn on_unit_unhover(
    ev: On<Pointer<Out>>,
    mut sprites: Query<&mut Sprite>,
    participants: Query<&CombatParticipant>,
    selection: Res<Selection>,
) {
    let target = ev.event_target();
    let Ok(mut sprite) = sprites.get_mut(target) else {
        return;
    };

    // 如果该单位是选中的单位，使用选中色而非队伍色
    if selection.selected_unit == Some(target) {
        sprite.color = SELECTED_COLOR;
        return;
    }

    let Ok(participant) = participants.get(target) else {
        return;
    };
    sprite.color = match participant.team_id.as_str() {
        "Player" => PLAYER_COLOR,
        "Enemy" => ENEMY_COLOR,
        _ => NEUTRAL_COLOR,
    };
}

// ─── 系统：附加单位视觉效果 ──────────────────────────────────────

/// 系统：为已生成的战斗单位添加占位精灵视觉效果。
///
/// 使用 `Without<Sprite>` 过滤器确保仅对尚无视觉组件的实体操作
/// （幂等性保证）。
pub fn attach_unit_visuals(
    mut commands: Commands,
    unit_query: Query<
        (
            Entity,
            &GridPos,
            &CombatParticipant,
            &HitPoints,
            &UnitIdComponent,
        ),
        Without<Sprite>,
    >,
    mut images: ResMut<Assets<Image>>,
) {
    let white = create_white_texture(&mut images);

    for (entity, pos, participant, _hp, uid) in unit_query.iter() {
        // 根据队伍选择颜色
        let color = match participant.team_id.as_str() {
            "Player" => PLAYER_COLOR,
            "Enemy" => ENEMY_COLOR,
            _ => NEUTRAL_COLOR,
        };

        // 网格坐标 → 屏幕像素坐标（原点在左下，向上/Y 轴生长）
        let x = pos.x as f32 * 80.0 + 40.0;
        let y = pos.y as f32 * 80.0 + 40.0;

        commands
            .entity(entity)
            .insert((
                Sprite {
                    image: white.clone(),
                    color,
                    custom_size: Some(Vec2::new(UNIT_SIZE, UNIT_SIZE)),
                    ..default()
                },
                Transform::from_xyz(x, y, 1.0),
                Visibility::default(),
                Pickable::default(),
            ))
            .observe(on_unit_click)
            .observe(on_unit_hover)
            .observe(on_unit_unhover);

        tracing::trace!(target: "app",
            "Attached visual for unit {} at ({}, {}) — entity={:?}",
            uid.id, pos.x, pos.y, entity,
        );
        println!("[DEBUG] Unit entity: {:?} id={}", entity, uid.id);
    }
}

// ─── 系统：网格背景 ────────────────────────────────────────────

/// 系统：生成棋盘格网格背景。
///
/// 每个格子用一个 Sprite 表示，颜色交替形成棋盘图案。
pub fn spawn_grid_background(
    mut commands: Commands,
    scenario: Option<Res<TestBattleScenario>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(scenario) = scenario else {
        return;
    };

    let grid = &scenario.def.grid;
    let cell_size = grid.cell_size;
    let white = create_white_texture(&mut images);

    for x in 0..grid.width {
        for y in 0..grid.height {
            let px = x as f32 * cell_size;
            let py = y as f32 * cell_size;

            // 棋盘格着色
            let is_dark = (x + y as u32) % 2 == 0;
            let bg_color = if is_dark { GRID_DARK } else { GRID_LIGHT };

            commands.spawn((
                Sprite {
                    image: white.clone(),
                    color: bg_color,
                    custom_size: Some(Vec2::new(cell_size - 1.0, cell_size - 1.0)),
                    ..default()
                },
                Transform::from_xyz(px + cell_size / 2.0, py + cell_size / 2.0, 0.0),
                Visibility::default(),
            ));
        }
    }

    tracing::debug!(target: "app",
        "Spawned grid background: {}x{}",
        grid.width, grid.height,
    );
}
