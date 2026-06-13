/// 技能标识（SkillId）
///
/// 技能的唯一标识。技能可以是主动技能（如火球术）或被动技能（如铁壁）。
use std::fmt;

/// 技能标识
///
/// ADR-002 §决策: 强类型 ID 包装器模式
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkillId(pub String);

impl SkillId {
    /// 创建一个新的 SkillId
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for SkillId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Skill({})", self.0)
    }
}

impl From<&str> for SkillId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for SkillId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_id_创建与相等性() {
        let id1 = SkillId::new("fireball");
        let id2 = SkillId::new("fireball");
        let id3 = SkillId::new("heal");
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn skill_id_display_格式() {
        let id = SkillId::new("fireball");
        assert_eq!(id.to_string(), "Skill(fireball)");
    }

    #[test]
    fn skill_id_与unit_id_类型安全() {
        // 编译时防止把 SkillId 传成 UnitId
        let skill = SkillId::new("fireball");
        let _unit_id = crate::core::id::UnitId::new("warrior");
        // 下面这行如果取消注释会编译错误，证明类型安全
        // assert_eq!(skill, _unit_id); // 编译错误!
        let _ = skill; // 使用变量避免警告
    }
}
