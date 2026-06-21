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

use crate::shared::game_state::GameState;

use self::render::{attach_unit_visuals, spawn_camera, spawn_grid_background};
use self::spawn::{load_test_battle_scenario, spawn_test_battle};

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

        // 进入战斗场景时生成实体
        app.add_systems(
            OnEnter(GameState::Combat),
            (spawn_test_battle, spawn_camera, spawn_grid_background),
        );

        // 视觉效果在实体生成后执行（延迟一帧确保 entity 存在）
        app.add_systems(
            OnEnter(GameState::Combat),
            (attach_unit_visuals,).after(spawn_test_battle),
        );
    }
}
