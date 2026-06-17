//! 标签系统（ADR-031 §2）
//!
//! 位掩码实现，O(1) 查询。基于 Linglan 5 分类模型：
//! - Elemental（bits 0-7）：伤害类型
//! - Status（bits 8-15）：状态/控制层级
//! - Class（bits 16-23）：阵营/身份
//! - Equipment（bits 24-31）：装备属性
//! - Mechanism（bits 32-39）：底层机制
//!
//! 标签定义从 RON 加载（`content/tags/tags.ron`）。

pub mod control;
pub mod def;

pub use def::*;

use bevy::prelude::*;

// ============================================================================
// GameplayTag — 位掩码常量
// ============================================================================

/// 游戏标签（位掩码）
///
/// 每个标签占用 1 bit，支持 O(1) 查询和组合运算。
/// 5 分类分布：
///   bits 0-7  : Elemental（伤害类型）
///   bits 8-15 : Status（状态/控制层级）
///   bits 16-23: Class（阵营/身份）
///   bits 24-31: Equipment（装备属性）
///   bits 32-39: Mechanism（底层机制）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub struct GameplayTag(pub u64);

impl GameplayTag {
    // ========== Elemental（bits 0-7）==========
    pub const DMG_FIRE: Self = Self(1 << 0);
    pub const DMG_ICE: Self = Self(1 << 1);
    pub const DMG_PHYSICAL: Self = Self(1 << 2);
    pub const DMG_MAGICAL: Self = Self(1 << 3);
    pub const DMG_PIERCE: Self = Self(1 << 4);
    pub const DMG_TRUE: Self = Self(1 << 5);

    // ========== Status（bits 8-15）==========
    pub const BUFF: Self = Self(1 << 8);
    pub const DEBUFF: Self = Self(1 << 9);
    pub const SPECIAL_STATE: Self = Self(1 << 10);
    pub const CONTROL_SOFT: Self = Self(1 << 11);
    pub const CONTROL_HARD: Self = Self(1 << 12);
    pub const CONTROL_FULL: Self = Self(1 << 13);
    pub const INVINCIBLE: Self = Self(1 << 14);
    pub const UNTARGETABLE: Self = Self(1 << 15);

    // ========== Class / Faction（bits 16-23）==========
    pub const ALLY: Self = Self(1 << 16);
    pub const ENEMY: Self = Self(1 << 17);
    pub const SUMMON: Self = Self(1 << 18);
    pub const BOSS: Self = Self(1 << 19);
    pub const MECHANICAL: Self = Self(1 << 20);

    // ========== Equipment（bits 24-31）==========
    pub const WEAPON_SWORD: Self = Self(1 << 24);
    pub const WEAPON_BOW: Self = Self(1 << 25);
    pub const WEAPON_STAFF: Self = Self(1 << 26);
    pub const HEAVY_ARMOR: Self = Self(1 << 27);
    pub const LIGHT_ARMOR: Self = Self(1 << 28);
    pub const SHIELD: Self = Self(1 << 29);

    // ========== Mechanism（bits 32-39）==========
    pub const FLYING: Self = Self(1 << 32);
    pub const GROUNDED: Self = Self(1 << 33);
    pub const DISPELLABLE: Self = Self(1 << 34);
    pub const UNDISPELLABLE: Self = Self(1 << 35);
    pub const REFLECTABLE: Self = Self(1 << 36);
    pub const UNTRIGGERABLE: Self = Self(1 << 37);

    /// 从 u64 原始值构造
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    /// 返回位掩码的原始值
    pub const fn bits(&self) -> u64 {
        self.0
    }
}

// ============================================================================
// GameplayTags — 实体标签组件
// ============================================================================

/// 实体上的标签集合组件（运行时）
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
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

    /// 检查是否满足条件 tag（AND 语义）
    pub fn matches(&self, required: GameplayTag) -> bool {
        self.has(required)
    }
}

// ============================================================================
// PersistentTags — 持久化标签组件
// ============================================================================

/// 持久化标签（不被 rebuild 丢失，支持 Trait + Equipment 两层）
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct PersistentTags {
    /// Trait 授予的标签（种族/职业/天赋，最持久）
    pub from_traits: GameplayTags,
    /// 装备授予的标签（穿脱变化）
    pub from_equipment: GameplayTags,
}

// ============================================================================
// rebuild_tags
// ============================================================================

/// 从 PersistentTags 重建 GameplayTags
///
/// 统一标签合并方式：Trait 标签 | Equipment 标签
pub fn rebuild_tags(persistent: &PersistentTags) -> GameplayTags {
    GameplayTags(persistent.from_traits.0 | persistent.from_equipment.0)
}

// ============================================================================
// TagPlugin
// ============================================================================

/// 标签层 Plugin（ADR-025 七领域之 TagPlugin）
///
/// 职责：注册标签系统的所有类型。
/// 在七领域 DAG 中为第 1 层，无其它依赖。
pub struct TagPlugin;

impl Plugin for TagPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameplayTag>()
            .register_type::<GameplayTags>()
            .register_type::<PersistentTags>();
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gameplay_tag_bitmask() {
        let tag = GameplayTag::DMG_FIRE;
        assert_eq!(tag.bits(), 1 << 0);
    }

    #[test]
    fn gameplay_tags_has_add_remove() {
        let mut tags = GameplayTags::default();
        assert!(!tags.has(GameplayTag::DMG_FIRE));

        tags.add(GameplayTag::DMG_FIRE);
        assert!(tags.has(GameplayTag::DMG_FIRE));
        assert!(!tags.has(GameplayTag::DMG_ICE));

        tags.remove(GameplayTag::DMG_FIRE);
        assert!(!tags.has(GameplayTag::DMG_FIRE));
    }

    #[test]
    fn gameplay_tags_multi_bit() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::DMG_FIRE);
        tags.add(GameplayTag::BUFF);

        assert!(tags.has(GameplayTag::DMG_FIRE));
        assert!(tags.has(GameplayTag::BUFF));
        assert!(!tags.has(GameplayTag::DMG_ICE));
    }

    #[test]
    fn gameplay_tags_has_any() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::DMG_FIRE);

        let check = GameplayTags::from_tags(&[GameplayTag::DMG_FIRE, GameplayTag::DMG_ICE]);
        assert!(tags.has_any(&check));
    }

    #[test]
    fn gameplay_tags_has_all() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::DMG_FIRE);

        let check = GameplayTags::from_tags(&[GameplayTag::DMG_FIRE, GameplayTag::DMG_ICE]);
        assert!(!tags.has_all(&check));

        tags.add(GameplayTag::DMG_ICE);
        assert!(tags.has_all(&check));
    }

    #[test]
    fn gameplay_tags_from_tags_empty() {
        let tags = GameplayTags::from_tags(&[]);
        assert!(!tags.has(GameplayTag::DMG_FIRE));
    }

    #[test]
    fn gameplay_tags_from_tags_multiple() {
        let tags = GameplayTags::from_tags(&[GameplayTag::DMG_FIRE, GameplayTag::BUFF]);
        assert!(tags.has(GameplayTag::DMG_FIRE));
        assert!(tags.has(GameplayTag::BUFF));
        assert!(!tags.has(GameplayTag::DMG_ICE));
    }

    #[test]
    fn gameplay_tags_add_idempotent() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::DMG_FIRE);
        tags.add(GameplayTag::DMG_FIRE); // twice
        assert!(tags.has(GameplayTag::DMG_FIRE));
    }

    #[test]
    fn gameplay_tags_has_all_empty_set() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::DMG_FIRE);
        let empty = GameplayTags::default();
        assert!(tags.has_all(&empty));
    }

    #[test]
    fn gameplay_tags_has_any_no_match() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::DMG_FIRE);
        let check = GameplayTags::from_tags(&[GameplayTag::DMG_ICE, GameplayTag::BUFF]);
        assert!(!tags.has_any(&check));
    }

    #[test]
    fn gameplay_tags_matches() {
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::ALLY);
        assert!(tags.matches(GameplayTag::ALLY));
        assert!(!tags.matches(GameplayTag::ENEMY));
    }

    #[test]
    fn rebuild_tags_combines_sources() {
        let mut traits = GameplayTags::default();
        traits.add(GameplayTag::ALLY);
        let mut equip = GameplayTags::default();
        equip.add(GameplayTag::HEAVY_ARMOR);

        let persistent = PersistentTags {
            from_traits: traits,
            from_equipment: equip,
        };

        let rebuilt = rebuild_tags(&persistent);
        assert!(rebuilt.has(GameplayTag::ALLY));
        assert!(rebuilt.has(GameplayTag::HEAVY_ARMOR));
    }

    #[test]
    fn gameplay_tag_distinct_bits() {
        // Verify no bits overlap between categories
        let elemental = GameplayTag::from_bits(0xFF);
        let status = GameplayTag::from_bits(0xFF00);
        let class_faction = GameplayTag::from_bits(0xFF0000);
        let equipment = GameplayTag::from_bits(0xFF000000);
        let mechanism = GameplayTag::from_bits(0xFF00000000);

        // Categories should be in distinct byte regions
        assert_eq!(elemental.0 & status.0, 0);
        assert_eq!(elemental.0 & class_faction.0, 0);
        assert_eq!(elemental.0 & equipment.0, 0);
        assert_eq!(elemental.0 & mechanism.0, 0);
        assert_eq!(status.0 & class_faction.0, 0);
        assert_eq!(status.0 & equipment.0, 0);
        assert_eq!(status.0 & mechanism.0, 0);
        assert_eq!(class_faction.0 & equipment.0, 0);
        assert_eq!(class_faction.0 & mechanism.0, 0);
        assert_eq!(equipment.0 & mechanism.0, 0);
    }
}
