/// Shared 层：基础能力（通用工具）
///
/// Layer 3 职责：零外部依赖，不含业务逻辑。
/// 将在 Phase 1 迁移中逐步填充。
pub mod ids;
pub mod error;
pub mod events;
pub mod audit;
pub mod random;
pub mod math;
pub mod time;
pub mod collections;
pub mod validation;
pub mod constants;
pub mod traits;
pub mod testing;
pub mod versioning;
