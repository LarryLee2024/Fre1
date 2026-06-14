/// 资源加载模块（迁移自 src/assets.rs, Phase 4.1）
pub mod assets;
/// 审计轨迹：领域事件白名单与审计收集（ADR-006）
pub mod audit;
/// 运行时配置模块（空壳，Phase 4.3）
pub mod config;
/// 热重载模块（空壳，Phase 4.3）
pub mod hot_reload;
/// 本地化模块（空壳，Phase 4.3）
pub mod localization;
/// 基础设施层：日志、审计、调试等跨模块基础设施
pub mod logging;
/// 持久化模块（空壳，Phase 4.3）
pub mod persistence;
/// 回放模块（空壳，Phase 4.3）
pub mod replay;
