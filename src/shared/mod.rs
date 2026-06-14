pub mod audit;
pub mod collections;
pub mod constants;
pub mod error;
pub mod events;
/// Shared 层：基础能力（通用工具）
///
/// Layer 3 职责：零外部依赖，不含业务逻辑。
/// 将在 Phase 1 迁移中逐步填充。
pub mod ids;
pub mod math;
pub mod random;
pub mod resettable;
pub mod testing;
pub mod time;
pub mod traits;
pub mod validation;
pub mod versioning;
