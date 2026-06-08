// 标签系统：位掩码实现，O(1) 查询，替代硬编码枚举匹配

use bevy::prelude::*;
use serde::Deserialize;

/// 游戏标签（位掩码）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GameplayTag(pub u64);

impl GameplayTag {
    // ── 元素 ──
    pub const FIRE: Self = Self(1 << 0);
    pub const ICE: Self = Self(1 << 1);
    pub const POISON: Self = Self(1 << 2);

    // ── 状态条件 ──
    pub const STUN: Self = Self(1 << 8);
    pub const BURN: Self = Self(1 << 9);
    pub const REGEN: Self = Self(1 << 10);

    // ── 武器/攻击类型 ──
    pub const MELEE: Self = Self(1 << 16);
    pub const RANGED: Self = Self(1 << 17);

    // ── 职业 ──
    pub const WARRIOR: Self = Self(1 << 24);
    pub const ARCHER: Self = Self(1 << 25);
    pub const MAGE: Self = Self(1 << 26);

    // ── 技能类型 ──
    pub const SKILL_ACTIVE: Self = Self(1 << 32);
    pub const SKILL_PASSIVE: Self = Self(1 << 33);

    // ── Buff 类型 ──
    pub const BUFF: Self = Self(1 << 40);
    pub const DEBUFF: Self = Self(1 << 41);

    /// 标签中文名
    pub fn label(&self) -> &'static str {
        match *self {
            Self::FIRE => "火焰",
            Self::ICE => "冰霜",
            Self::POISON => "毒素",
            Self::STUN => "晕眩",
            Self::BURN => "燃烧",
            Self::REGEN => "恢复",
            Self::MELEE => "近战",
            Self::RANGED => "远程",
            Self::WARRIOR => "战士",
            Self::ARCHER => "弓手",
            Self::MAGE => "法师",
            Self::SKILL_ACTIVE => "主动技能",
            Self::SKILL_PASSIVE => "被动技能",
            Self::BUFF => "增益",
            Self::DEBUFF => "减益",
            _ => "未知",
        }
    }
}

/// 实体上的标签集合组件
#[derive(Component, Default, Debug, Clone)]
pub struct GameplayTags(pub u64);

impl GameplayTags {
    pub fn has(&self, tag: GameplayTag) -> bool {
        self.0 & tag.0 != 0
    }

    pub fn add(&mut self, tag: GameplayTag) {
        self.0 |= tag.0;
    }

    pub fn remove(&mut self, tag: GameplayTag) {
        self.0 &= !tag.0;
    }

    pub fn has_any(&self, tags: &GameplayTags) -> bool {
        self.0 & tags.0 != 0
    }

    pub fn has_all(&self, tags: &GameplayTags) -> bool {
        self.0 & tags.0 == tags.0
    }

    /// 从标签列表构建
    pub fn from_tags(tags: &[GameplayTag]) -> Self {
        let mut result = Self::default();
        for tag in tags {
            result.add(*tag);
        }
        result
    }
}

/// 标签名称枚举（用于数据定义中的序列化/反序列化）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TagName {
    Fire,
    Ice,
    Poison,
    Stun,
    Burn,
    Regen,
    Melee,
    Ranged,
    Warrior,
    Archer,
    Mage,
    SkillActive,
    SkillPassive,
    Buff,
    Debuff,
}

impl TagName {
    pub fn to_tag(&self) -> GameplayTag {
        match self {
            Self::Fire => GameplayTag::FIRE,
            Self::Ice => GameplayTag::ICE,
            Self::Poison => GameplayTag::POISON,
            Self::Stun => GameplayTag::STUN,
            Self::Burn => GameplayTag::BURN,
            Self::Regen => GameplayTag::REGEN,
            Self::Melee => GameplayTag::MELEE,
            Self::Ranged => GameplayTag::RANGED,
            Self::Warrior => GameplayTag::WARRIOR,
            Self::Archer => GameplayTag::ARCHER,
            Self::Mage => GameplayTag::MAGE,
            Self::SkillActive => GameplayTag::SKILL_ACTIVE,
            Self::SkillPassive => GameplayTag::SKILL_PASSIVE,
            Self::Buff => GameplayTag::BUFF,
            Self::Debuff => GameplayTag::DEBUFF,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 标签_位掩码查询() {
        let mut tags = GameplayTags::default();
        assert!(!tags.has(GameplayTag::FIRE));

        tags.add(GameplayTag::FIRE);
        assert!(tags.has(GameplayTag::FIRE));
        assert!(!tags.has(GameplayTag::ICE));

        tags.remove(GameplayTag::FIRE);
        assert!(!tags.has(GameplayTag::FIRE));
    }

    #[test]
    fn 标签_多标签组合() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::FIRE);
        tags.add(GameplayTag::DEBUFF);

        assert!(tags.has(GameplayTag::FIRE));
        assert!(tags.has(GameplayTag::DEBUFF));
        assert!(!tags.has(GameplayTag::ICE));
    }

    #[test]
    fn 标签_has_any() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::FIRE);

        let check = GameplayTags::from_tags(&[GameplayTag::FIRE, GameplayTag::ICE]);
        assert!(tags.has_any(&check));
    }

    #[test]
    fn 标签_has_all() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::FIRE);

        let check = GameplayTags::from_tags(&[GameplayTag::FIRE, GameplayTag::ICE]);
        assert!(!tags.has_all(&check));

        tags.add(GameplayTag::ICE);
        assert!(tags.has_all(&check));
    }

    #[test]
    fn tag_name_转换() {
        assert_eq!(TagName::Fire.to_tag(), GameplayTag::FIRE);
        assert_eq!(TagName::Stun.to_tag(), GameplayTag::STUN);
    }
}
