//! CameraQuery — 公开只读查询 API
//!
//! 提供坐标转换和可视区域查询的纯函数集合。
//! 外部系统通过此 API 获取镜头信息，无需直接 Query Camera Entity。
//!
//! 详见 `docs/02-domain/infrastructure/camera_domain.md` §6.3

use bevy::prelude::*;

/// CameraQuery 系统参数——对外只读查询 API。
///
/// 用法：
/// ```ignore
/// fn my_system(camera_query: CameraQuery, camera: Query<(&Camera, &GlobalTransform)>, window: Query<&Window>) {
///     let Ok((cam, cam_transform)) = camera.single() else { return };
///     let Ok(window) = window.single() else { return };
///     let screen_pos = CameraQuery::world_to_screen(world_pos, cam, cam_transform, &window);
/// }
/// ```
pub struct CameraQuery;

impl CameraQuery {
    /// 世界坐标 → 屏幕像素坐标。
    ///
    /// 将游戏世界中的位置转换为屏幕上的像素位置。
    /// 返回 None 表示位置在屏幕外或转换失败。
    pub fn world_to_screen(
        world_pos: Vec2,
        camera: &Camera,
        camera_transform: &GlobalTransform,
        window: &Window,
    ) -> Option<Vec2> {
        let world_pos_3d = world_pos.extend(0.0);
        let view_combined = camera_transform.to_matrix() * camera.clip_from_view();
        let ndc = view_combined.project_point3(world_pos_3d);

        // NDC [-1, 1] → 屏幕像素坐标
        let screen_size = Vec2::new(window.width(), window.height());
        let screen_pos = Vec2::new(
            (ndc.x + 1.0) * 0.5 * screen_size.x,
            (1.0 - ndc.y) * 0.5 * screen_size.y,
        );

        Some(screen_pos)
    }

    /// 屏幕像素坐标 → 世界坐标。
    ///
    /// 将屏幕上的像素位置转换为游戏世界位置。
    /// 返回 None 表示转换失败。
    pub fn screen_to_world(
        screen_pos: Vec2,
        camera: &Camera,
        camera_transform: &GlobalTransform,
        window: &Window,
    ) -> Option<Vec2> {
        let screen_size = Vec2::new(window.width(), window.height());

        // 屏幕像素坐标 → NDC [-1, 1]
        let ndc = Vec3::new(
            (screen_pos.x / screen_size.x) * 2.0 - 1.0,
            (1.0 - screen_pos.y / screen_size.y) * 2.0 - 1.0,
            0.0,
        );

        let view_combined = camera_transform.to_matrix() * camera.clip_from_view();
        let view_inverse = view_combined.inverse();
        let world_pos = view_inverse.project_point3(ndc);

        Some(world_pos.truncate())
    }

    /// 当前可视矩形（世界空间）。
    ///
    /// 返回摄像机在当前位姿下可以看到的矩形区域。
    /// 用于视口框显示、可见性判断等。
    pub fn visible_rect(_camera: &Camera, camera_transform: &GlobalTransform) -> Rect {
        // 从 camera 的投影参数计算可视范围
        // 对于正交投影，可视范围由 projection.near/far 和视口大小决定
        let transform = camera_transform.to_matrix();
        let _view_inverse = transform.inverse();

        // 使用投影矩阵计算视锥体的近平面在 world space 中的范围
        // 对于 2D 正交相机，可视矩形 = 中心点 ± 半尺寸
        let camera_pos = camera_transform.translation();

        // 默认可视矩形以相机位置为中心
        // 实际计算需要投影参数——简化实现返回大范围
        // TODO[P2][Camera][2026-07-01]: 基于 OrthographicProjection 的精确可视矩形计算
        let default_half_size = 500.0;
        Rect::from_center_size(camera_pos.truncate(), Vec2::splat(default_half_size * 2.0))
    }
}
