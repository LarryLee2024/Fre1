//! 领域 String ID 类型（由宏统一生成）
//!
//! 所有 Definition 级别的 String ID 集中在此文件。
//! 每个类型通过 `define_string_id!` 宏生成，确保行为一致。

use crate::define_string_id;
use bevy::prelude::Reflect;

// ============================================================================
// String ID 类型（领域 Definition 标识）
// ============================================================================

define_string_id! {
    pub AttributeId,
    prefix: "attr",
}

define_string_id! {
    pub TagId,
    prefix: "tag",
}

define_string_id! {
    pub ModifierId,
    prefix: "mod",
}

define_string_id! {
    pub EffectId,
    prefix: "eff",
}

define_string_id! {
    pub AbilityId,
    prefix: "abl",
}

define_string_id! {
    pub TriggerId,
    prefix: "trg",
}

define_string_id! {
    pub CueId,
    prefix: "cue",
}

define_string_id! {
    pub CharacterId,
    prefix: "char",
}

define_string_id! {
    pub UnitId,
    prefix: "unit",
}

define_string_id! {
    pub EquipmentId,
    prefix: "equip",
}

define_string_id! {
    pub ItemId,
    prefix: "itm",
}

define_string_id! {
    pub FactionId,
    prefix: "fct",
}

define_string_id! {
    pub QuestId,
    prefix: "qst",
}

define_string_id! {
    pub SpellId,
    prefix: "spl",
}

define_string_id! {
    pub BuffId,
    prefix: "buf",
}

define_string_id! {
    pub TerrainId,
    prefix: "ter",
}

define_string_id! {
    pub RecipeId,
    prefix: "rcp",
}

define_string_id! {
    pub LootTableId,
    prefix: "ltb",
}

define_string_id! {
    pub TeamId,
    prefix: "team",
}

define_string_id! {
    pub ClassId,
    prefix: "cls",
}

define_string_id! {
    pub TalentId,
    prefix: "tal",
}

define_string_id! {
    pub SubclassId,
    prefix: "sub",
}

define_string_id! {
    pub BondDefId,
    prefix: "bnd",
}

define_string_id! {
    pub FormationDefId,
    prefix: "fmd",
}

define_string_id! {
    pub CampEventId,
    prefix: "cmp",
}
