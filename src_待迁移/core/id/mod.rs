/// 强类型 ID 模块（ADR-002）
///
/// 核心领域实体的强类型标识，替代裸 Entity 作为业务标识跨模块传递。
/// 每个业务实体使用独立的 ID 类型，编译期防止传参错误。
///
/// 设计原则：
/// - 每个业务领域定义独立 ID 类型
/// - 内部使用 String 存储可读标识
/// - 日志、错误信息输出字符串 ID 而非 Entity 编号
mod buff_id;
mod item_id;
mod skill_id;
mod unit_id;

pub use buff_id::BuffId;
pub use item_id::ItemId;
pub use skill_id::SkillId;
pub use unit_id::UnitId;
