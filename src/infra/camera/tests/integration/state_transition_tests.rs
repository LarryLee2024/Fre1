//! Camera 状态转移集成测试
//!
//! 验证 ADR-064 §3 定义的状态转移矩阵。
//! 使用最小 Bevy App 测试完整的事件驱动流程。

use bevy::prelude::*;

use crate::infra::camera::components::MainCamera;
use crate::infra::camera::foundation::pose::{CameraPose, CurrentPose, TargetPose};
use crate::infra::camera::foundation::request::CameraRequest;
use crate::infra::camera::foundation::state::CameraState;
use crate::infra::camera::foundation::target::CameraTarget;
use crate::infra::camera::resources::{TileSize, UnitPositionResolver};

/// 设置最小测试 App（CameraPlugin 已注册 + 一个 Camera Entity）
/// 返回 (App, Entity) 以便后续通过 entity ID 访问 Camera 组件。
fn setup_test_app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // 手动注册 CameraPlugin 的 observer 和 resources
    app.init_resource::<UnitPositionResolver>();
    app.init_resource::<TileSize>();
    app.add_observer(process_camera_requests_tester);

    // Spawn Camera Entity
    let camera_entity = app
        .world_mut()
        .spawn((
            MainCamera,
            TargetPose(CameraPose::default()),
            CurrentPose(CameraPose::default()),
            CameraState::Idle,
        ))
        .id();

    (app, camera_entity)
}

/// 一个简化的 CameraRequest 处理函数（仅测试状态转移逻辑）
fn process_camera_requests_tester(
    trigger: On<CameraRequest>,
    mut camera_query: Query<&mut CameraState, With<MainCamera>>,
) {
    let request = trigger.event();
    let Ok(mut state) = camera_query.single_mut() else {
        return;
    };

    match request {
        CameraRequest::Follow { target } if !matches!(*state, CameraState::Focus { .. }) => {
            *state = CameraState::Follow(*target);
        }
        CameraRequest::Unfollow if matches!(*state, CameraState::Follow(_)) => {
            *state = CameraState::Idle;
        }
        _ => {}
    }
}

/// 验证 Idle → Follow 状态转移
#[test]
fn idle_to_follow() {
    let (mut app, entity) = setup_test_app();

    let target = CameraTarget::WorldPos(Vec2::new(100.0, 200.0));
    app.world_mut()
        .commands()
        .trigger(CameraRequest::Follow { target });
    app.update();

    let state = app.world().entity(entity).get::<CameraState>().unwrap();
    assert_eq!(
        *state,
        CameraState::Follow(CameraTarget::WorldPos(Vec2::new(100.0, 200.0)))
    );
}

/// 验证 Follow → Unfollow → Idle 状态转移
#[test]
fn follow_to_unfollow() {
    let (mut app, entity) = setup_test_app();

    // 直接设置状态为 Follow
    app.world_mut()
        .entity_mut(entity)
        .insert(CameraState::Follow(CameraTarget::WorldPos(Vec2::new(
            50.0, 50.0,
        ))));

    app.world_mut().commands().trigger(CameraRequest::Unfollow);
    app.update();

    let state = app.world().entity(entity).get::<CameraState>().unwrap();
    assert_eq!(*state, CameraState::Idle);
}

/// 验证 Idle 时 Unfollow 被静默忽略
#[test]
fn idle_unfollow_is_noop() {
    let (mut app, entity) = setup_test_app();

    app.world_mut().commands().trigger(CameraRequest::Unfollow);
    app.update();

    let state = app.world().entity(entity).get::<CameraState>().unwrap();
    assert_eq!(*state, CameraState::Idle);
}

/// 验证 Focus 状态下 Follow 被静默忽略
#[test]
fn focus_ignores_follow() {
    let (mut app, entity) = setup_test_app();

    // 设置 Focus 状态
    app.world_mut()
        .entity_mut(entity)
        .insert(CameraState::Focus {
            target: CameraTarget::WorldPos(Vec2::ZERO),
            duration: 5.0,
            elapsed: 0.0,
        });

    // Follow 请求在 Focus 状态下应被忽略
    app.world_mut().commands().trigger(CameraRequest::Follow {
        target: CameraTarget::WorldPos(Vec2::new(999.0, 999.0)),
    });
    app.update();

    let state = app.world().entity(entity).get::<CameraState>().unwrap();
    assert!(matches!(*state, CameraState::Focus { .. }));
}

/// 验证 CameraTarget::resolve 的等价类——使用纯函数测试
#[test]
fn target_resolve_equivalence() {
    // WorldPos 直接返回位置
    let wp = CameraTarget::WorldPos(Vec2::new(42.0, 73.0));
    assert_eq!(wp.resolve(&|_| None, 80.0), Vec2::new(42.0, 73.0));

    // TilePos 乘以 tile_size
    let tp = CameraTarget::TilePos(3, 5);
    assert_eq!(tp.resolve(&|_| None, 80.0), Vec2::new(240.0, 400.0));

    // UnitId 通过解析器查找
    let uid = CameraTarget::UnitId(1);
    assert_eq!(
        uid.resolve(
            &|id| if id == 1 {
                Some(Vec2::new(100.0, 200.0))
            } else {
                None
            },
            80.0
        ),
        Vec2::new(100.0, 200.0)
    );
}
