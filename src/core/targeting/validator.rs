// 目标验证器：校验候选目标是否符合技能的目标选择规则
//
// 参考：docs/02-domain/targeting/targeting-rules.md
// 参考：docs/02-domain/selector/selector-rules.md

use crate::core::character::{Dead, Faction, Unit};
use crate::core::targeting::{SkillTargeting, TargetingContext};
use bevy::prelude::*;

/// 验证结果错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetValidationError {
    /// 目标实体不存在
    EntityNotFound,
    /// 目标已死亡
    TargetDead,
    /// 目标阵营不匹配（需要敌对但选了友方，反之亦然）
    FactionMismatch,
    /// 不是自身（SelfOnly 技能选了其他目标）
    NotSelf,
    /// 尝试对空地上锁
    NotAnEntity,
}

/// 目标验证器 trait：校验候选目标是否合法
///
/// 每次目标选择时调用，确保技能不会对非法目标生效。
/// 验证器为纯函数，不修改 World 状态。
pub trait TargetValidator: Send + Sync + 'static {
    /// 校验目标是否合法
    fn validate(
        &self,
        ctx: &TargetingContext,
        target: Entity,
        caster_faction: Faction,
        target_faction: Faction,
        is_dead: bool,
    ) -> Result<(), TargetValidationError>;
}

/// 默认目标验证器：按 SkillTargeting 类型进行基础校验
#[derive(Default)]
pub struct DefaultTargetValidator;

impl TargetValidator for DefaultTargetValidator {
    fn validate(
        &self,
        ctx: &TargetingContext,
        target: Entity,
        caster_faction: Faction,
        target_faction: Faction,
        is_dead: bool,
    ) -> Result<(), TargetValidationError> {
        // 1. 阵营匹配检查
        match ctx.targeting_type {
            SkillTargeting::SingleEnemy | SkillTargeting::AoeEnemies => {
                // 敌对目标：阵营必须不同
                if caster_faction == target_faction {
                    return Err(TargetValidationError::FactionMismatch);
                }
            }
            SkillTargeting::SingleAlly | SkillTargeting::AoeAllies => {
                // 友方目标：阵营必须相同
                if caster_faction != target_faction {
                    return Err(TargetValidationError::FactionMismatch);
                }
            }
            SkillTargeting::SelfOnly => {
                // 自身：必须与施法者相同
                if target != ctx.caster {
                    return Err(TargetValidationError::NotSelf);
                }
            }
            SkillTargeting::NoTarget => {
                // 无目标：不需要目标选择
                return Err(TargetValidationError::NotAnEntity);
            }
        }

        // 2. 存活检查：不能对已死亡的目标释放技能（除非目标就是施法者自己）
        if is_dead && target != ctx.caster {
            return Err(TargetValidationError::TargetDead);
        }

        Ok(())
    }
}

/// 辅助函数：通过 ECS Query 执行一次完整的目标验证
///
/// 在 System 中快速调用，自动从 World 读取 Unit（含Faction）/Dead 组件。
pub fn validate_target_with_query(
    query: &Query<(&Unit, Has<Dead>)>,
    ctx: &TargetingContext,
    target: Entity,
    validator: &dyn TargetValidator,
) -> Result<(), TargetValidationError> {
    let Ok((target_unit, is_dead)) = query.get(target) else {
        return Err(TargetValidationError::EntityNotFound);
    };
    let caster_faction = query
        .get(ctx.caster)
        .map(|(unit, _)| unit.faction)
        .map_err(|_| TargetValidationError::EntityNotFound)?;

    validator.validate(ctx, target, caster_faction, target_unit.faction, is_dead)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context(targeting_type: SkillTargeting) -> TargetingContext {
        TargetingContext {
            caster: Entity::from_bits(1),
            ability_id: "test_ability".into(),
            targeting_type,
        }
    }

    #[test]
    fn 默认验证器_敌对阵营_同阵营报错() {
        let validator = DefaultTargetValidator;
        let ctx = make_context(SkillTargeting::SingleEnemy);
        let result = validator.validate(
            &ctx,
            Entity::from_bits(2),
            Faction::Player,
            Faction::Player,
            false,
        );
        assert_eq!(result, Err(TargetValidationError::FactionMismatch));
    }

    #[test]
    fn 默认验证器_敌对阵营_不同阵营通过() {
        let validator = DefaultTargetValidator;
        let ctx = make_context(SkillTargeting::SingleEnemy);
        let result = validator.validate(
            &ctx,
            Entity::from_bits(2),
            Faction::Player,
            Faction::Enemy,
            false,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn 默认验证器_自身_他人报错() {
        let validator = DefaultTargetValidator;
        let ctx = make_context(SkillTargeting::SelfOnly);
        let result = validator.validate(
            &ctx,
            Entity::from_bits(2), // 不是施法者
            Faction::Player,
            Faction::Player,
            false,
        );
        assert_eq!(result, Err(TargetValidationError::NotSelf));
    }

    #[test]
    fn 默认验证器_自身_施法者通过() {
        let validator = DefaultTargetValidator;
        let ctx = make_context(SkillTargeting::SelfOnly);
        let result = validator.validate(
            &ctx,
            Entity::from_bits(1), // 施法者
            Faction::Player,
            Faction::Player,
            false,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn 默认验证器_已死亡目标_非施法者_报错() {
        let validator = DefaultTargetValidator;
        let ctx = make_context(SkillTargeting::SingleEnemy);
        let result = validator.validate(
            &ctx,
            Entity::from_bits(2),
            Faction::Player,
            Faction::Enemy,
            true, // 目标已死亡
        );
        assert_eq!(result, Err(TargetValidationError::TargetDead));
    }

    #[test]
    fn 默认验证器_已死亡目标_施法者自身_通过() {
        let validator = DefaultTargetValidator;
        let ctx = make_context(SkillTargeting::SelfOnly);
        // 即使已死亡，自身仍然可以作为目标（如复活技能）
        let result = validator.validate(
            &ctx,
            Entity::from_bits(1),
            Faction::Player,
            Faction::Player,
            true,
        );
        assert!(result.is_ok());
    }
}
