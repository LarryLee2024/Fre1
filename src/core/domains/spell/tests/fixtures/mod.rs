//! Spell Domain — 测试辅助
//!
//! 提供 Builder 模式和标准测试数据。

use crate::core::domains::spell::components::{
    CastingTime, SpellComponents, SpellDef, SpellDefId, SpellDuration, SpellLevel, SpellRange,
    SpellSlotEntry, SpellSlotPool,
};

/// 创建一个标准的测试法术。
pub fn base_spell(id: &str, level: SpellLevel) -> SpellDef {
    SpellDef {
        id: SpellDefId::new(id),
        name_key: "spell.test.name".into(),
        desc_key: "spell.test.desc".into(),
        level,
        casting_time: CastingTime::Action,
        components: SpellComponents {
            verbal: true,
            somatic: true,
            material: None,
        },
        range: SpellRange::Self_,
        duration: SpellDuration::Instant,
        requires_concentration: false,
        saving_throw: None,
        can_upcast: true,
        effects: vec![],
    }
}

/// 创建一个法术位池。
pub fn slot_pool(counts: &[(u32, u32)]) -> SpellSlotPool {
    let mut slots = Vec::with_capacity(9);
    for i in 0..9 {
        let (total, used) = counts.get(i).copied().unwrap_or((0, 0));
        slots.push(SpellSlotEntry { total, used });
    }
    SpellSlotPool {
        slots_by_level: slots,
    }
}

/// 一个可升环的 3 环法术。
pub fn fireball() -> SpellDef {
    SpellDef {
        id: SpellDefId::new("spl_fireball"),
        name_key: "spell.fireball.name".into(),
        desc_key: "spell.fireball.desc".into(),
        level: SpellLevel::L3,
        casting_time: CastingTime::Action,
        components: SpellComponents {
            verbal: true,
            somatic: true,
            material: None,
        },
        range: SpellRange::Ranged {
            base: 150,
            max: None,
        },
        duration: SpellDuration::Instant,
        requires_concentration: false,
        saving_throw: None,
        can_upcast: true,
        effects: vec![],
    }
}
