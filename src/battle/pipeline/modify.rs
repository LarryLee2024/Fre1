// 步骤 2：修饰效果（从 ModifierRuleRegistry 加载规则）

use crate::gameplay::effect::{EffectQueue, PendingEffectData};
use crate::gameplay::modifier_rule::ModifierRuleRegistry;
use crate::gameplay::tag::GameplayTags;
use bevy::prelude::*;

/// 修饰效果：对所有伤害效果应用 ModifierRule 规则
pub fn modify_effects(
    mut queue: ResMut<EffectQueue>,
    tags_query: Query<&GameplayTags>,
    rules: Res<ModifierRuleRegistry>,
) {
    for effect in &mut queue.pending {
        if let PendingEffectData::Damage { ref mut amount, .. } = effect.data {
            if let Ok(target_tags) = tags_query.get(effect.target) {
                *amount = rules.apply_damage_modifiers(*amount, &effect.source_tags, target_tags);
            }
        }
    }
}
