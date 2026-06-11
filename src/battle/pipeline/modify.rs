// 步骤 2：修饰效果（从 ModifierRuleRegistry 加载规则）

use crate::core::effect::{EffectQueue, PendingEffectData};
use crate::core::modifier_rule::ModifierRuleRegistry;
use crate::core::tag::GameplayTags;
use bevy::prelude::*;

/// 修饰效果：对所有效果应用 ModifierRule 规则
pub fn modify_effects(
    mut queue: ResMut<EffectQueue>,
    tags_query: Query<&GameplayTags>,
    rules: Res<ModifierRuleRegistry>,
) {
    for effect in &mut queue.pending {
        if let Ok(target_tags) = tags_query.get(effect.target) {
            match &mut effect.data {
                PendingEffectData::Damage {
                    amount,
                    base_amount,
                    ..
                } => {
                    let original = *amount;
                    // 首次 modify 时记录 base_amount
                    if base_amount.is_none() {
                        *base_amount = Some(original);
                    }
                    *amount =
                        rules.apply_damage_modifiers(*amount, &effect.source_tags, target_tags);
                    bevy::log::debug!(
                        target: "battle",
                        target_entity = ?effect.target,
                        original_damage = original,
                        modified_damage = *amount,
                        "伤害修饰"
                    );
                }
                PendingEffectData::Heal {
                    amount,
                    base_amount,
                } => {
                    let original = *amount;
                    if base_amount.is_none() {
                        *base_amount = Some(original);
                    }
                    *amount = rules.apply_heal_modifiers(*amount, &effect.source_tags, target_tags);
                    bevy::log::debug!(
                        target: "battle",
                        target_entity = ?effect.target,
                        original_heal = original,
                        modified_heal = *amount,
                        "治疗修饰"
                    );
                }
                PendingEffectData::ApplyBuff { .. } | PendingEffectData::Cleanse => {}
            }
        }
    }
}
