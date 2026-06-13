/// 分领域错误体系（ADR-004）
///
/// 采用「分领域错误枚举」架构，每个业务领域定义独立错误枚举。
/// 严格区分四类失败场景：RuleFailure / DomainError / InfrastructureError / Bug。
///
/// 设计原则：
/// - 每个领域定义独立错误枚举，避免全局 AppError 大枚举
/// - 使用错误码格式：领域前缀+序号（如 B001、S001）
/// - 所有错误携带完整上下文信息
/// - 核心业务层禁止 unwrap/expect/panic
mod battle_error;
mod buff_error;
mod game_result;
mod inventory_error;
mod skill_error;

pub use battle_error::BattleError;
pub use buff_error::BuffError;
pub use game_result::GameResult;
pub use inventory_error::InventoryError;
pub use skill_error::SkillError;
