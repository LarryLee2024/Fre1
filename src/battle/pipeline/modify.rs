// 步骤 2：修饰效果（从 ModifierRuleRegistry 加载规则）

use crate::core::effect::{EffectQueue, PendingEffectData};
use crate::core::modifier_rule::ModifierRuleRegistry;
use crate::core::tag::GameplayTags;
use bevy::prelude::*;

/// 修饰效果：对所有效果应用 ModifierRule 规则
pub fn modify_effects(
    mut queue: ResMut<EffectQueue>,
    tags_query: Query<&GameplayTags>,
    names_query: Query<&Name>,
    rules: Res<ModifierRuleRegistry>,
) {
    for effect in &mut queue.pending {
        if let Ok(target_tags) = tags_query.get(effect.target) {
            let target_name = names_query.get(effect.target).map(|n| n.as_str()).unwrap_or("?");
            match &mut effect.data {
                PendingEffectData::Damage {
                    amount,
                    base_amount,
                    modifiers,
                    ..
                } => {
                    let original = *amount;
                    // 首次 modify 时记录 base_amount
                    if base_amount.is_none() {
                        *base_amount = Some(original);
                    }
                    let (new_amount, entries) =
                        rules.apply_damage_modifiers_with_breakdown(*amount, &effect.source_tags, target_tags);
                    // 不变量3：伤害下限保护，Modify 完成后伤害值 ≥ 1
                    *amount = new_amount.max(1);
                    *modifiers = entries;
                    bevy::log::debug!(
                        target: "battle",
                        target_entity = ?effect.target,
                        target_name = %target_name,
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
                        target_name = %target_name,
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
