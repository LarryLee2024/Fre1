//! TestBattle — 数据驱动的测试战斗场景
//!
//! 使用 RON 配置文件 `assets/configs/scenarios/test_battle.ron` 定义战斗单位与网格参数。
//! 纯逻辑 Spawn + 占位视觉渲染，零外部美术资产依赖。
//!
//! 视觉系统独立于逻辑 Spawn，便于替换为正式渲染管线。
//!
//! 详见 ADR-050, ADR-052

pub mod def;
pub mod render;
pub mod spawn;

use bevy::prelude::*;

use crate::infra::camera::foundation::request::CameraRequest;
use crate::infra::camera::foundation::target::CameraTarget;
use crate::shared::game_state::GameState;

use self::render::{attach_unit_visuals, spawn_grid_background};
use self::spawn::{load_test_battle_scenario, spawn_test_battle};

/// 系统：移动镜头到战斗场景初始位置
///
/// 镜头实体由 CameraPlugin 在 Startup 时自动生成，此处不再 spawn 新摄像头，
/// 而是通过 CameraRequest::MoveTo 调整已有镜头位置。
fn spawn_camera_for_battle(mut commands: Commands) {
    commands.trigger(CameraRequest::MoveTo {
        target: CameraTarget::WorldPos(Vec2::new(240.0, 240.0)),
        duration: 0.0,
    });
}

/// 测试战斗场景 Plugin
///
/// 注册系统：
/// - `Startup`: 加载 RON 配置
/// - `OnEnter(GameState::Combat)`: 生成单位实体 + 摄像机 + 网格 + 视觉效果
///
/// 系统顺序：spawn → camera/grid → visuals
pub struct TestBattlePlugin;

impl Plugin for TestBattlePlugin {
    fn build(&self, app: &mut App) {
        // 启动时加载配置文件
        app.add_systems(Startup, load_test_battle_scenario);

        // 进入战斗场景时：先生成实体，再附加视觉效果（.chain() 确保 flush）
        app.add_systems(
            OnEnter(GameState::Combat),
            (
                (
                    spawn_test_battle,
                    spawn_camera_for_battle,
                    spawn_grid_background,
                ),
                attach_unit_visuals,
            )
                .chain(),
        );
    }
}
