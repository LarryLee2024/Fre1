/// 物品强类型 ID（ADR-002）
/// 用于在业务逻辑中安全地标识物品，避免裸 String 混用
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemId(pub String);
