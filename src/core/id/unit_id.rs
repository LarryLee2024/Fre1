/// 单位标识（UnitId）
///
/// 战场上每个战斗单位的唯一标识。
/// 使用 String 内部存储以实现日志可读性。
use std::fmt;

/// 单位标识
///
/// ADR-002 §决策: 强类型 ID 包装器模式
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnitId(pub String);

impl UnitId {
    /// 创建一个新的 UnitId
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for UnitId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unit({})", self.0)
    }
}

impl From<&str> for UnitId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for UnitId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_id_创建与相等性() {
        let id1 = UnitId::new("warrior_001");
        let id2 = UnitId::new("warrior_001");
        let id3 = UnitId::new("mage_001");
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn unit_id_display_格式() {
        let id = UnitId::new("warrior_001");
        assert_eq!(id.to_string(), "Unit(warrior_001)");
    }

    #[test]
    fn unit_id_from_str() {
        let id: UnitId = "warrior_001".into();
        assert_eq!(id.0, "warrior_001");
    }

    #[test]
    fn unit_id_hash_一致性() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(UnitId::new("a"));
        set.insert(UnitId::new("b"));
        set.insert(UnitId::new("a")); // duplicate
        assert_eq!(set.len(), 2);
    }
}
