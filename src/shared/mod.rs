pub mod audit;
pub mod collections;
pub mod constants;
pub mod error;
/// 统一可观察事件定义（按领域分文件）
pub mod event;
/// 跨模块领域事件白名单（占位）
pub mod events;
/// Shared 层：基础能力（通用工具）
///
/// Layer 3 职责：零外部依赖，不含业务逻辑。
/// 将在 Phase 1 迁移中逐步填充。
pub mod ids;
pub mod math;
pub mod random;
/// Registry 基础 trait 族（ADR-030 §2）
///
/// - [`registry::Registry`] — 只读查询接口
/// - [`registry::LoadableRegistry`] — RON 加载支持
/// - [`registry::ValidatableRegistry`] — 自校验支持
/// - [`registry::RegistryInitStage`] — DAG 初始化阶段
pub mod registry;
pub mod resettable;
/// Shared 层统一 Plugin
pub mod shared_plugin;
pub mod testing;
pub mod time;
pub mod traits;
pub mod validation;
pub mod versioning;

pub use shared_plugin::SharedPlugin;
