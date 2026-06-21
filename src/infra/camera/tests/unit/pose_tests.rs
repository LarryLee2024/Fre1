use crate::infra::camera::foundation::pose::CameraPose;
use bevy::math::Vec2;

/// 测试 CameraPose::lerp 在 t=0 时返回原始姿态
#[test]
fn lerp_at_zero_returns_self() {
    let current = CameraPose {
        position: Vec2::new(100.0, 200.0),
        zoom: 1.0,
        rotation: 0.0,
    };
    let target = CameraPose {
        position: Vec2::new(300.0, 400.0),
        zoom: 2.0,
        rotation: 0.5,
    };
    let result = current.lerp(&target, 0.0);
    assert_eq!(result, current);
}

/// 测试 CameraPose::lerp 在 t=1 时返回目标姿态
#[test]
fn lerp_at_one_returns_target() {
    let current = CameraPose {
        position: Vec2::new(100.0, 200.0),
        zoom: 1.0,
        rotation: 0.0,
    };
    let target = CameraPose {
        position: Vec2::new(300.0, 400.0),
        zoom: 2.0,
        rotation: 0.5,
    };
    let result = current.lerp(&target, 1.0);
    assert_eq!(result, target);
}

/// 测试 CameraPose::lerp 在 t=0.5 时返回中间值
#[test]
fn lerp_at_half_returns_midpoint() {
    let current = CameraPose {
        position: Vec2::new(0.0, 0.0),
        zoom: 1.0,
        rotation: 0.0,
    };
    let target = CameraPose {
        position: Vec2::new(200.0, 100.0),
        zoom: 3.0,
        rotation: 1.0,
    };
    let result = current.lerp(&target, 0.5);
    assert_eq!(
        result,
        CameraPose {
            position: Vec2::new(100.0, 50.0),
            zoom: 2.0,
            rotation: 0.5,
        }
    );
}

/// 测试 CameraPose::lerp 是确定性的（同输入、同参数产生同输出）
#[test]
fn lerp_is_deterministic() {
    let current = CameraPose {
        position: Vec2::new(10.0, 20.0),
        zoom: 1.0,
        rotation: 0.0,
    };
    let target = CameraPose {
        position: Vec2::new(50.0, 60.0),
        zoom: 2.0,
        rotation: 0.3,
    };

    let result1 = current.lerp(&target, 0.33);
    let result2 = current.lerp(&target, 0.33);
    assert_eq!(result1, result2);
}

/// 测试 CameraPose::Default 返回预期的默认值
#[test]
fn default_pose() {
    let pose = CameraPose::default();
    assert_eq!(pose.position, Vec2::ZERO);
    assert_eq!(pose.zoom, 1.0);
    assert_eq!(pose.rotation, 0.0);
}
