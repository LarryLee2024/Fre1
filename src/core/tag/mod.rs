// 标签系统：位掩码实现，O(1) 查询，替代硬编码枚举匹配

pub mod def;

pub use def::*;

use bevy::prelude::*;
use serde::Deserialize;

/// 游戏标签（位掩码）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
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

    // ── 移动类型 ──
    pub const FLYING: Self = Self(1 << 48);
    pub const MOUNTED: Self = Self(1 << 49);
    pub const SWIMMING: Self = Self(1 << 50);

    // ── 物品类型 ──
    pub const CONSUMABLE: Self = Self(1 << 51);
    pub const AMMO: Self = Self(1 << 52);
    pub const MATERIAL: Self = Self(1 << 53);
    pub const CURRENCY: Self = Self(1 << 54);
    pub const QUEST_ITEM: Self = Self(1 << 55);

    // ── 消耗品子类 ──
    pub const HEALING: Self = Self(1 << 56);
    pub const POTION: Self = Self(1 << 57);
    pub const SCROLL: Self = Self(1 << 58);
    pub const FOOD: Self = Self(1 << 59);

    // ── 技能类型 ──
    pub const SKILL_ACTIVE: Self = Self(1 << 32);
    pub const SKILL_PASSIVE: Self = Self(1 << 33);

    // ── Buff 类型 ──
    pub const BUFF: Self = Self(1 << 40);
    pub const DEBUFF: Self = Self(1 << 41);

    // ── 装备属性 ──
    pub const HEAVY_ARMOR: Self = Self(1 << 42);
    pub const LIGHT_ARMOR: Self = Self(1 << 43);
    pub const SHIELD: Self = Self(1 << 44);
    pub const TWO_HANDED: Self = Self(1 << 45);
    pub const MARTIAL: Self = Self(1 << 46);
    pub const SIMPLE: Self = Self(1 << 47);

    // ── 武器类型 ──
    pub const SWORD: Self = Self(1 << 20);
    pub const AXE: Self = Self(1 << 21);
    pub const BOW: Self = Self(1 << 22);
    pub const STAFF: Self = Self(1 << 23);

    /// 返回位掩码中已使用的 bit 数量
    pub fn used_bits() -> u32 {
        TagName::ALL
            .iter()
            .map(|name| name.to_tag().0.count_ones())
            .sum()
    }
}

/// 实体上的标签集合组件
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct GameplayTags(pub u64);

/// 持久化标签（不被 rebuild 丢失，支持 Trait + Equipment 两层）
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct PersistentTags {
    /// Trait 授予的标签（种族/职业/天赋，最持久）
    pub from_traits: GameplayTags,
    /// 装备授予的标签（穿脱变化）
    pub from_equipment: GameplayTags,
}

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

    /// 返回所有已激活的标签列表
    pub fn active_tags(&self) -> Vec<GameplayTag> {
        const ALL_TAGS: &[GameplayTag] = &[
            GameplayTag::FIRE,
            GameplayTag::ICE,
            GameplayTag::POISON,
            GameplayTag::STUN,
            GameplayTag::BURN,
            GameplayTag::REGEN,
            GameplayTag::MELEE,
            GameplayTag::RANGED,
            GameplayTag::SWORD,
            GameplayTag::AXE,
            GameplayTag::BOW,
            GameplayTag::STAFF,
            GameplayTag::WARRIOR,
            GameplayTag::ARCHER,
            GameplayTag::MAGE,
            GameplayTag::FLYING,
            GameplayTag::MOUNTED,
            GameplayTag::SWIMMING,
            GameplayTag::CONSUMABLE,
            GameplayTag::AMMO,
            GameplayTag::MATERIAL,
            GameplayTag::CURRENCY,
            GameplayTag::QUEST_ITEM,
            GameplayTag::HEALING,
            GameplayTag::POTION,
            GameplayTag::SCROLL,
            GameplayTag::FOOD,
            GameplayTag::SKILL_ACTIVE,
            GameplayTag::SKILL_PASSIVE,
            GameplayTag::BUFF,
            GameplayTag::DEBUFF,
            GameplayTag::HEAVY_ARMOR,
            GameplayTag::LIGHT_ARMOR,
            GameplayTag::SHIELD,
            GameplayTag::TWO_HANDED,
            GameplayTag::MARTIAL,
            GameplayTag::SIMPLE,
        ];
        ALL_TAGS.iter().copied().filter(|t| self.has(*t)).collect()
    }
}

/// 标签名称枚举（用于数据定义中的序列化/反序列化）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, Deserialize)]
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
    Sword,
    Axe,
    Bow,
    Staff,
    Warrior,
    Archer,
    Mage,
    Flying,
    Mounted,
    Swimming,
    Consumable,
    Ammo,
    Material,
    Currency,
    QuestItem,
    Healing,
    Potion,
    Scroll,
    Food,
    SkillActive,
    SkillPassive,
    Buff,
    Debuff,
    HeavyArmor,
    LightArmor,
    Shield,
    TwoHanded,
    Martial,
    Simple,
}

impl TagName {
    /// 所有 TagName 变体列表（用于遍历校验）
    pub const ALL: &'static [TagName] = &[
        TagName::Fire,
        TagName::Ice,
        TagName::Poison,
        TagName::Stun,
        TagName::Burn,
        TagName::Regen,
        TagName::Melee,
        TagName::Ranged,
        TagName::Sword,
        TagName::Axe,
        TagName::Bow,
        TagName::Staff,
        TagName::Warrior,
        TagName::Archer,
        TagName::Mage,
        TagName::Flying,
        TagName::Mounted,
        TagName::Swimming,
        TagName::Consumable,
        TagName::Ammo,
        TagName::Material,
        TagName::Currency,
        TagName::QuestItem,
        TagName::Healing,
        TagName::Potion,
        TagName::Scroll,
        TagName::Food,
        TagName::SkillActive,
        TagName::SkillPassive,
        TagName::Buff,
        TagName::Debuff,
        TagName::HeavyArmor,
        TagName::LightArmor,
        TagName::Shield,
        TagName::TwoHanded,
        TagName::Martial,
        TagName::Simple,
    ];

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
            Self::Sword => GameplayTag::SWORD,
            Self::Axe => GameplayTag::AXE,
            Self::Bow => GameplayTag::BOW,
            Self::Staff => GameplayTag::STAFF,
            Self::Warrior => GameplayTag::WARRIOR,
            Self::Archer => GameplayTag::ARCHER,
            Self::Mage => GameplayTag::MAGE,
            Self::Flying => GameplayTag::FLYING,
            Self::Mounted => GameplayTag::MOUNTED,
            Self::Swimming => GameplayTag::SWIMMING,
            Self::Consumable => GameplayTag::CONSUMABLE,
            Self::Ammo => GameplayTag::AMMO,
            Self::Material => GameplayTag::MATERIAL,
            Self::Currency => GameplayTag::CURRENCY,
            Self::QuestItem => GameplayTag::QUEST_ITEM,
            Self::Healing => GameplayTag::HEALING,
            Self::Potion => GameplayTag::POTION,
            Self::Scroll => GameplayTag::SCROLL,
            Self::Food => GameplayTag::FOOD,
            Self::SkillActive => GameplayTag::SKILL_ACTIVE,
            Self::SkillPassive => GameplayTag::SKILL_PASSIVE,
            Self::Buff => GameplayTag::BUFF,
            Self::Debuff => GameplayTag::DEBUFF,
            Self::HeavyArmor => GameplayTag::HEAVY_ARMOR,
            Self::LightArmor => GameplayTag::LIGHT_ARMOR,
            Self::Shield => GameplayTag::SHIELD,
            Self::TwoHanded => GameplayTag::TWO_HANDED,
            Self::Martial => GameplayTag::MARTIAL,
            Self::Simple => GameplayTag::SIMPLE,
        }
    }
}

/// 从 PersistentTags 重建 GameplayTags
/// 统一标签合并方式，替代 spawn.rs 中的手动位运算
pub fn rebuild_tags(persistent: &PersistentTags) -> GameplayTags {
    GameplayTags(persistent.from_traits.0 | persistent.from_equipment.0)
}

/// 标签层 Plugin（ADR-025 七领域之 TagPlugin）
///
/// 职责：注册标签系统的所有类型，作为七领域 DAG 的第 1 层。
/// 无依赖，所有其它领域都依赖此层。
pub struct TagPlugin;

impl Plugin for TagPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameplayTag>()
            .register_type::<GameplayTags>()
            .register_type::<PersistentTags>()
            .register_type::<TagName>();
    }
}

#[cfg(test)]
mod tests {
    // ================================================
    // Bevy SRPG AI宪法 v1.1 自检结果（测试专用）
    // ================================================
    // ✅ 测行为不测实现：是 — 断言验证标签位运算结果，不验证内部 u64 表示
    // ✅ 符合领域规则：是 — 覆盖 INV-TAG-1~5 标签系统不变量
    // ✅ 确定性：是 — 硬编码标签值
    // ✅ 使用标准数据：是 — 使用标准 GameplayTag 枚举
    // ✅ 无越界测试：是 — 仅测试公共 API
    // ✅ 未测试私有实现：是 — 仅通过 pub 接口测试
    // ================================================
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

    #[test]
    fn 标签_from_tags空数组() {
        let tags = GameplayTags::from_tags(&[]);
        assert!(!tags.has(GameplayTag::FIRE));
    }

    #[test]
    fn 标签_from_tags多个标签() {
        let tags = GameplayTags::from_tags(&[GameplayTag::FIRE, GameplayTag::STUN]);
        assert!(tags.has(GameplayTag::FIRE));
        assert!(tags.has(GameplayTag::STUN));
        assert!(!tags.has(GameplayTag::ICE));
    }

    #[test]
    fn 标签_has_any都不匹配() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::FIRE);
        let check = GameplayTags::from_tags(&[GameplayTag::ICE, GameplayTag::STUN]);
        assert!(!tags.has_any(&check));
    }

    #[test]
    fn 标签_has_all空集() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::FIRE);
        let empty = GameplayTags::default();
        assert!(tags.has_all(&empty));
    }

    #[test]
    fn 标签_add重复幂等() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::FIRE);
        tags.add(GameplayTag::FIRE);
        assert!(tags.has(GameplayTag::FIRE));
    }
}
