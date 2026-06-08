// 步骤 1：生成战斗效果（从技能定义 + 属性计算）
// 同时支持玩家（Selected）和 AI（CombatIntent.source_entity）
// 使用 EffectHandlerRegistry trait 分发，新增效果类型无需修改此文件

use crate::character::{GridPosition, Selected, Unit, UnitName};
use crate::gameplay::attribute::Attributes;
use crate::gameplay::effect::{EffectHandlerRegistry, EffectQueue, GenerateContext, PendingEffect};
use crate::gameplay::tag::GameplayTags;
use crate::map::Tile;
use crate::skill::{BASIC_ATTACK_ID, SkillCooldowns, SkillRegistry};
use bevy::prelude::*;

use super::intent::CombatIntent;

/// 生成战斗效果：从攻击者的技能定义 + 目标属性计算，推入 EffectQueue
///
/// 支持两种来源：
/// - 玩家：通过 Selected 组件查找攻击者
/// - AI：通过 CombatIntent.source_entity 查找攻击者
pub fn generate_combat_effects(
    mut queue: ResMut<EffectQueue>,
    handler_registry: Res<EffectHandlerRegistry>,
    // 玩家攻击者（Selected）
    selected_units: Query<
        (
            Entity,
            &Unit,
            &GridPosition,
            &UnitName,
            &Attributes,
            &GameplayTags,
            &SkillCooldowns,
        ),
        With<Selected>,
    >,
    // 所有单位（用于查找 AI 攻击者和目标）
    all_units: Query<(
        Entity,
        &Unit,
        &GridPosition,
        &UnitName,
        &Attributes,
        &GameplayTags,
        &SkillCooldowns,
    )>,
    // 目标单位（含 Transform，仅用于兼容）
    targets: Query<(
        Entity,
        &Unit,
        &GridPosition,
        &UnitName,
        &Attributes,
        &GameplayTags,
        &Transform,
    )>,
    tiles: Query<&Tile>,
    combat_intent: Res<CombatIntent>,
    skill_registry: Res<SkillRegistry>,
) {
    // 确定攻击者来源
    let source_info = if let Ok(info) = selected_units.single() {
        // 玩家：通过 Selected 查找
        Some(info)
    } else if let Some(source_entity) = combat_intent.source_entity {
        // AI：通过 CombatIntent.source_entity 查找
        all_units.get(source_entity).ok()
    } else {
        None
    };

    let Some((
        source_entity,
        source_unit,
        _source_gp,
        _source_name,
        source_attrs,
        source_tags,
        source_cooldowns,
    )) = source_info
    else {
        return;
    };

    // 晕眩检查
    if source_tags.has(crate::gameplay::tag::GameplayTag::STUN) {
        return;
    }

    let Some(target_coord) = combat_intent.target_coord else {
        return;
    };

    let skill_id = combat_intent.skill_id.as_deref().unwrap_or(BASIC_ATTACK_ID);
    let Some(skill_data) = skill_registry.get(skill_id) else {
        return;
    };

    // 冷却检查（玩家需要，AI 已在决策时检查）
    if source_cooldowns.get(skill_id) > 0 {
        return;
    }

    // 查找目标
    for (
        target_entity,
        target_unit,
        target_gp,
        _target_name,
        target_attrs,
        _target_tags,
        _target_transform,
    ) in &targets
    {
        if target_gp.coord != target_coord || target_unit.faction == source_unit.faction {
            continue;
        }

        let Some(tile) = tiles.iter().find(|t| t.coord == target_gp.coord) else {
            continue;
        };
        let terrain = tile.terrain;
        let defense_bonus = tile.defense_bonus;

        for effect_def in &skill_data.effects {
            // 通过 EffectHandlerRegistry trait 分发，新增效果类型无需修改此处
            if let Some(handler) = handler_registry.find(effect_def.type_name()) {
                let ctx = GenerateContext {
                    source_entity,
                    target_entity,
                    source_attrs: source_attrs.clone(),
                    target_attrs: target_attrs.clone(),
                    defense_bonus,
                    skill_id: skill_id.to_string(),
                    source_tags: skill_data.tags.clone(),
                    terrain,
                };

                if let Some(data) = handler.generate(effect_def, &ctx) {
                    queue.push(PendingEffect {
                        source: source_entity,
                        target: target_entity,
                        data,
                        source_tags: skill_data.tags.clone(),
                        terrain,
                    });
                }
            } else {
                bevy::log::warn!(
                    "未注册的效果处理器: {}，跳过效果生成",
                    effect_def.type_name()
                );
            }
        }
        break;
    }
}
