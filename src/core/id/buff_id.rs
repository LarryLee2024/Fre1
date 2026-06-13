/// Buff 标识（BuffId）
///
/// Buff/Debuff 效果的唯一标识。
/// Buff 包括增益（如狂暴）和减益（如中毒、晕眩）。
use std::fmt;

/// Buff 标识
///
/// ADR-002 §决策: 强类型 ID 包装器模式
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BuffId(pub String);

impl BuffId {
    /// 创建一个新的 BuffId
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for BuffId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Buff({})", self.0)
    }
}

impl From<&str> for BuffId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for BuffId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buff_id_创建与相等性() {
        let id1 = BuffId::new("poison");
        let id2 = BuffId::new("poison");
        let id3 = BuffId::new("berserk");
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn buff_id_display_格式() {
        let id = BuffId::new("poison");
        assert_eq!(id.to_string(), "Buff(poison)");
    }

    #[test]
    fn buff_id_from_str() {
        let id: BuffId = "berserk".into();
        assert_eq!(id.0, "berserk");
    }
}
