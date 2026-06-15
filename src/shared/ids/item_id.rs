/// 物品标识（ItemId）
///
/// 游戏物品的唯一标识。物品包括武器、防具、消耗品、任务道具等。
use std::fmt;

/// 物品标识
///
/// ADR-002 §决策: 强类型 ID 包装器模式
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemId(pub String);

impl ItemId {
    /// 创建一个新的 ItemId
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Item({})", self.0)
    }
}

impl From<&str> for ItemId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for ItemId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_id_创建与相等性() {
        let id1 = ItemId::new("iron_sword");
        let id2 = ItemId::new("iron_sword");
        let id3 = ItemId::new("healing_potion");
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn item_id_display_格式() {
        let id = ItemId::new("iron_sword");
        assert_eq!(id.to_string(), "Item(iron_sword)");
    }

    #[test]
    fn item_id_from_string() {
        let id = ItemId::from("healing_potion".to_string());
        assert_eq!(id.0, "healing_potion");
    }
}
