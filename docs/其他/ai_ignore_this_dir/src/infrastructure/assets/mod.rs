/// 资产加载错误枚举
pub mod asset_error;
/// 资源加载模块（Phase 4.1 迁移）
///
/// 从 `src/assets.rs` 迁移至此，属于基础设施层。
/// 负责管理应用程序级别的共享资源和文件加载。

/// 全局共享资源（字体等）
pub mod game_assets;
/// RON 文件加载工具
pub mod loaders;
/// AssetsPlugin 插件注册
pub mod plugin;

pub use asset_error::{AssetError, AssetResult};
/// 公开 re-exports
pub use game_assets::CnFont;
pub use plugin::AssetsPlugin;
