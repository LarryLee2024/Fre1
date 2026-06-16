// ADR-014: 五阶段技能释放管线
//
// 所有技能释放必须遵循固定管线：
//   Stage 1: Validate（验证）→ 调用 SkillData.can_use()
//   Stage 2: Cost（消耗）→ 扣 MP + 设置冷却
//   Stage 3: Cast（施法）→ 发送 SkillActivated Message
//   Stage 4: Effect（效果执行）→ Effect Pipeline（在 battle/pipeline/ 中实现）
//   Stage 5: Settlement（结算）→ 死亡检查 + Trait 触发

use crate::core::attribute::Attributes;
use crate::core::tag::GameplayTags;
use crate::core::turn::TurnOrder;
use bevy::prelude::*;

use super::domain::{SkillData, SkillUseError};
use super::preview::SkillExecutionContext;
use super::slots::SkillCooldowns;

/// 技能执行准备结果：Validate → Cost → Cast 三阶段的产物
/// 供 Effect Pipeline（Stage 4）消费
#[derive(Clone, Debug)]
pub struct PreparedSkillAction {
    /// 技能执行上下文（传递给 Effect Pipeline）
    pub context: SkillExecutionContext,
    /// 技能数据（含 effects 列表）
    pub skill_data: SkillData,
    /// 地形 ID
    pub terrain_id: String,
}

/// ADR-014 Stage 1-3: Validate → Cost → Cast
///
/// 纯函数式验证 + 代价计算（不修改 ECS 状态）。
/// 返回值：
///   - Ok(PreparedSkillAction) → Effect Pipeline 可以安全消费
///   - Err(SkillUseError) → 释放失败原因
///
/// 调用者负责：
///   - Stage 2 的副作用（MP 扣减、冷却设置）
///   - Stage 3 的 SkillActivated 发送
pub fn prepare_skill_execution(
    source: Entity,
    target: Entity,
    skill_data: &SkillData,
    source_attrs: &Attributes,
    source_tags: &GameplayTags,
    target_attrs: &Attributes,
    target_tags: &GameplayTags,
    cooldowns: &SkillCooldowns,
    terrain_defense_bonus: i32,
    terrain_id: &str,
) -> Result<PreparedSkillAction, SkillUseError> {
    let skill_id = &skill_data.id;
    let cooldown_remaining = cooldowns.get(skill_id);

    // Stage 1: Validate
    skill_data.can_use(
        source_attrs,
        source_tags,
        Some(target_tags),
        cooldown_remaining,
    )?;

    // Stage 2-3: 计算代价（调用者负责副作用）
    // 返回 PreparedSkillAction 供 Effect Pipeline 消费
    let context = SkillExecutionContext::from_query(
        source,
        target,
        skill_id,
        source_attrs,
        target_attrs,
        source_tags,
        target_tags,
        terrain_defense_bonus,
    );

    Ok(PreparedSkillAction {
        context,
        skill_data: skill_data.clone(),
        terrain_id: terrain_id.to_string(),
    })
}

/// Stage 2 side effects: 扣 MP + 设置冷却
/// 与 prepare_skill_execution 分离，因为需要 &mut Attributes 和 &mut SkillCooldowns
pub fn apply_skill_costs(
    skill_data: &SkillData,
    source_attrs: &mut Attributes,
    cooldowns: &mut SkillCooldowns,
) {
    skill_data.deduct_cost(source_attrs);
    if skill_data.cooldown > 0 {
        cooldowns.set(&skill_data.id, skill_data.cooldown);
    }
}

/// 判断单位是否存活（HP > 0）
pub fn is_unit_alive(attrs: &Attributes) -> bool {
    attrs.current_hp > 0
}

/// Stage 5: 路由到下一个单位
/// 从 TurnOrder 队列前进到下一个存活的单位
pub fn advance_to_next_unit(
    turn_order: &mut TurnOrder,
    unit_query: &Query<(&Attributes,), Without<crate::core::character::Selected>>,
    next_phase: &mut ResMut<NextState<crate::core::turn::TurnPhase>>,
) {
    loop {
        match turn_order.advance() {
            Some(next_entity) => {
                if let Ok((attrs,)) = unit_query.get(next_entity) {
                    if is_unit_alive(attrs) {
                        next_phase.set(crate::core::turn::TurnPhase::SelectUnit);
                        return;
                    }
                }
            }
            None => {
                next_phase.set(crate::core::turn::TurnPhase::TurnEnd);
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::domain::{SkillData, SkillTargeting};
    use super::*;
    use crate::core::attribute::Attributes;
    use crate::core::effect::EffectDef;
    use crate::core::tag::GameplayTags;

    fn make_test_skill(cost_mp: i32, cooldown: u32) -> SkillData {
        SkillData {
            id: "test_skill".into(),
            name: "测试技能".into(),
            description: String::new(),
            name_key: None,
            desc_key: None,
            cost_mp,
            range: 1,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![EffectDef::Damage {
                multiplier: 1.0,
                ignore_def_percent: 0.0,
            }],
            tags: vec![],
            conditions: vec![],
            cooldown,
            priority: 0,
        }
    }

    fn make_attrs(mp: i32, hp: i32) -> Attributes {
        let mut attrs = Attributes::default();
        attrs.set_max_hp(30);
        attrs.set_base("mp", 10);
        attrs.fill_hp();
        attrs.current_hp = hp;
        attrs.set_base("mp", mp);
        attrs
    }

    #[test]
    fn prepare_execution_验证通过() {
        let skill = make_test_skill(5, 2);
        let source_attrs = make_attrs(10, 30);
        let target_attrs = make_attrs(5, 20);
        let tags = GameplayTags::default();
        let cooldowns = SkillCooldowns::default();

        let result = prepare_skill_execution(
            Entity::from_bits(1),
            Entity::from_bits(2),
            &skill,
            &source_attrs,
            &tags,
            &target_attrs,
            &tags,
            &cooldowns,
            0,
            "plain",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn prepare_execution_冷却中失败() {
        let skill = make_test_skill(0, 2);
        let attrs = make_attrs(10, 30);
        let tags = GameplayTags::default();
        let mut cooldowns = SkillCooldowns::default();
        cooldowns.set("test_skill", 3);

        let result = prepare_skill_execution(
            Entity::from_bits(1),
            Entity::from_bits(2),
            &skill,
            &attrs,
            &tags,
            &attrs,
            &tags,
            &cooldowns,
            0,
            "plain",
        );
        assert!(matches!(
            result,
            Err(SkillUseError::OnCooldown { remaining: 3 })
        ));
    }

    #[test]
    fn prepare_execution_mp_不足失败() {
        let skill = make_test_skill(10, 0);
        let source_attrs = make_attrs(3, 30); // MP=3 < 10
        let target_attrs = make_attrs(5, 20);
        let tags = GameplayTags::default();
        let cooldowns = SkillCooldowns::default();

        let result = prepare_skill_execution(
            Entity::from_bits(1),
            Entity::from_bits(2),
            &skill,
            &source_attrs,
            &tags,
            &target_attrs,
            &tags,
            &cooldowns,
            0,
            "plain",
        );
        assert!(matches!(
            result,
            Err(SkillUseError::InsufficientMp {
                required: 10,
                current: 3
            })
        ));
    }

    #[test]
    fn apply_costs_扣mp和冷却() {
        let skill = make_test_skill(5, 3);
        let mut attrs = make_attrs(10, 30);
        let mut cooldowns = SkillCooldowns::default();

        apply_skill_costs(&skill, &mut attrs, &mut cooldowns);

        assert_eq!(attrs.get("mp"), 5); // 10 - 5 = 5
        assert_eq!(cooldowns.get("test_skill"), 3);
    }

    #[test]
    fn apply_costs_冷却为0不设置() {
        let skill = make_test_skill(5, 0);
        let mut attrs = make_attrs(10, 30);
        let mut cooldowns = SkillCooldowns::default();

        apply_skill_costs(&skill, &mut attrs, &mut cooldowns);

        assert_eq!(cooldowns.get("test_skill"), 0);
    }

    #[test]
    fn apply_costs_mp_不足不减到负数() {
        let skill = make_test_skill(10, 0);
        let mut attrs = make_attrs(3, 30); // MP=3 < 10

        apply_skill_costs(&skill, &mut attrs, &mut SkillCooldowns::default());

        assert_eq!(attrs.get("mp"), 0); // max(3-10, 0) = 0
    }

    #[test]
    fn is_alive_存活检测() {
        let mut attrs = Attributes::default();
        attrs.set_max_hp(30);
        attrs.fill_hp();
        attrs.current_hp = 10;
        assert!(is_unit_alive(&attrs));

        attrs.current_hp = 0;
        assert!(!is_unit_alive(&attrs));
    }
}
