use crate::infra::camera::foundation::target::CameraTarget;
use bevy::math::Vec2;

/// 测试 CameraTarget::WorldPos 解析
#[test]
fn resolve_world_pos() {
    let target = CameraTarget::WorldPos(Vec2::new(100.0, 200.0));
    let pos = target.resolve(&|_| None, 80.0);
    assert_eq!(pos, Vec2::new(100.0, 200.0));
}

/// 测试 CameraTarget::TilePos 解析（使用默认 tile_size=80.0）
#[test]
fn resolve_tile_pos() {
    let target = CameraTarget::TilePos(3, 4);
    let pos = target.resolve(&|_| None, 80.0);
    assert_eq!(pos, Vec2::new(240.0, 320.0));
}

/// 测试 CameraTarget::TilePos 解析（使用自定义 tile_size）
#[test]
fn resolve_tile_pos_custom_size() {
    let target = CameraTarget::TilePos(2, 5);
    let pos = target.resolve(&|_| None, 64.0);
    assert_eq!(pos, Vec2::new(128.0, 320.0));
}

/// 测试 CameraTarget::UnitId 解析（使用已注册的解析器）
#[test]
fn resolve_unit_id_with_resolver() {
    let target = CameraTarget::UnitId(42);
    let resolver = |id: u64| -> Option<Vec2> {
        match id {
            42 => Some(Vec2::new(500.0, 300.0)),
            _ => None,
        }
    };
    let pos = target.resolve(&resolver, 80.0);
    assert_eq!(pos, Vec2::new(500.0, 300.0));
}

/// 测试 CameraTarget::UnitId 在解析器返回 None 时的回退行为
#[test]
fn resolve_unit_id_fallback_on_missing() {
    let target = CameraTarget::UnitId(999);
    let resolver = |_id: u64| -> Option<Vec2> { None };
    let pos = target.resolve(&resolver, 80.0);
    assert_eq!(pos, Vec2::ZERO); // 回退到默认值
}
