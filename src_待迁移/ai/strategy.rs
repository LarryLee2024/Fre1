// AI 策略 trait 扩展点
// Trait 描述规则，不描述内容；运行时通过注册表分发，替代 enum+match 模式
// 新增策略只需实现对应 trait 并注册，无需修改已有代码

use std::collections::HashMap;

use bevy::prelude::*;

use crate::battle::manhattan_distance;
use crate::skill::{BASIC_ATTACK_ID, SkillCooldowns};

use super::targeting::UnitSnapshot;

// ── 目标选择规则 ──────────────────────────────────────────────

/// 目标选择规则 trait：描述如何选择攻击目标
pub(crate) trait TargetSelector: Send + Sync + 'static {
    /// 策略标识（与 RON 中的策略名称字符串对应）
    fn strategy_name(&self) -> &'static str;
    /// 从候选单位中选择目标坐标
    fn select(&self, candidates: &[UnitSnapshot], my_coord: IVec2) -> Option<IVec2>;
}

/// 选择最近的敌人
pub(crate) struct NearestTarget;

impl TargetSelector for NearestTarget {
    fn strategy_name(&self) -> &'static str {
        "Nearest"
    }

    fn select(&self, candidates: &[UnitSnapshot], my_coord: IVec2) -> Option<IVec2> {
        candidates
            .iter()
            .min_by_key(|s| manhattan_distance(my_coord, s.coord))
            .map(|s| s.coord)
    }
}

/// 选择血量最低的敌人
pub(crate) struct WeakestTarget;

impl TargetSelector for WeakestTarget {
    fn strategy_name(&self) -> &'static str {
        "Weakest"
    }

    fn select(&self, candidates: &[UnitSnapshot], _my_coord: IVec2) -> Option<IVec2> {
        candidates
            .iter()
            .min_by_key(|s| s.hp as i32)
            .map(|s| s.coord)
    }
}

/// 选择攻击力最高的敌人
pub(crate) struct MostDangerousTarget;

impl TargetSelector for MostDangerousTarget {
    fn strategy_name(&self) -> &'static str {
        "MostDangerous"
    }

    fn select(&self, candidates: &[UnitSnapshot], _my_coord: IVec2) -> Option<IVec2> {
        candidates
            .iter()
            .max_by_key(|s| s.atk as i32)
            .map(|s| s.coord)
    }
}

/// 选择血量百分比最低的敌人
pub(crate) struct LowestHpPercentTarget;

impl TargetSelector for LowestHpPercentTarget {
    fn strategy_name(&self) -> &'static str {
        "LowestHpPercent"
    }

    fn select(&self, candidates: &[UnitSnapshot], _my_coord: IVec2) -> Option<IVec2> {
        candidates
            .iter()
            .min_by_key(|s| {
                if s.max_hp > 0.0 {
                    (s.hp / s.max_hp * 100.0) as i32
                } else {
                    0
                }
            })
            .map(|s| s.coord)
    }
}

// ── 移动选择规则 ──────────────────────────────────────────────

/// 移动选择规则 trait：描述如何选择移动目标
pub(crate) trait MoveSelector: Send + Sync + 'static {
    fn strategy_name(&self) -> &'static str;
    fn select(
        &self,
        reachable: &HashMap<IVec2, u32>,
        my_coord: IVec2,
        target_coord: IVec2,
        attack_range: u32,
    ) -> IVec2;
}

/// 贪心靠近目标
pub(crate) struct AggressiveMove;

impl MoveSelector for AggressiveMove {
    fn strategy_name(&self) -> &'static str {
        "Aggressive"
    }

    fn select(
        &self,
        reachable: &HashMap<IVec2, u32>,
        my_coord: IVec2,
        target_coord: IVec2,
        _attack_range: u32,
    ) -> IVec2 {
        reachable
            .keys()
            .min_by_key(|coord| manhattan_distance(**coord, target_coord))
            .copied()
            .unwrap_or(my_coord)
    }
}

/// 保持攻击距离，不靠近超过攻击范围
pub(crate) struct CautiousMove;

impl MoveSelector for CautiousMove {
    fn strategy_name(&self) -> &'static str {
        "Cautious"
    }

    fn select(
        &self,
        reachable: &HashMap<IVec2, u32>,
        my_coord: IVec2,
        target_coord: IVec2,
        attack_range: u32,
    ) -> IVec2 {
        // 筛选在攻击范围内的位置
        let at_range: Vec<_> = reachable
            .keys()
            .filter(|coord| manhattan_distance(**coord, target_coord) <= attack_range)
            .collect();

        if at_range.is_empty() {
            // 没有在攻击范围内的位置，靠近
            reachable
                .keys()
                .min_by_key(|coord| manhattan_distance(**coord, target_coord))
                .copied()
                .unwrap_or(my_coord)
        } else {
            // 选择最远的（保持距离）
            at_range
                .iter()
                .max_by_key(|coord| manhattan_distance(***coord, target_coord))
                .map(|c| **c)
                .unwrap_or(my_coord)
        }
    }
}

/// 优先靠近友军（暂用最近目标逻辑）
pub(crate) struct SupportMove;

impl MoveSelector for SupportMove {
    fn strategy_name(&self) -> &'static str {
        "Support"
    }

    fn select(
        &self,
        reachable: &HashMap<IVec2, u32>,
        my_coord: IVec2,
        target_coord: IVec2,
        _attack_range: u32,
    ) -> IVec2 {
        // 优先靠近友军（暂用最近目标逻辑）
        reachable
            .keys()
            .min_by_key(|coord| manhattan_distance(**coord, target_coord))
            .copied()
            .unwrap_or(my_coord)
    }
}

// ── 技能选择规则 ──────────────────────────────────────────────

/// 技能选择规则 trait：描述如何选择使用的技能
pub(crate) trait SkillSelector: Send + Sync + 'static {
    fn strategy_name(&self) -> &'static str;
    fn select<'a>(
        &self,
        skill_ids: &'a [String],
        cooldowns: &SkillCooldowns,
        priority: &'a [String],
    ) -> &'a str;
}

/// 优先特殊技能
pub(crate) struct PreferSpecialSkill;

impl SkillSelector for PreferSpecialSkill {
    fn strategy_name(&self) -> &'static str {
        "PreferSpecial"
    }

    fn select<'a>(
        &self,
        skill_ids: &'a [String],
        cooldowns: &SkillCooldowns,
        _priority: &'a [String],
    ) -> &'a str {
        // 优先特殊技能（跳过冷却中的），否则基础攻击
        skill_ids
            .iter()
            .find(|id| *id != BASIC_ATTACK_ID && cooldowns.get(id) == 0)
            .map(|s| s.as_str())
            .unwrap_or(BASIC_ATTACK_ID)
    }
}

/// 优先基础攻击
pub(crate) struct PreferBasicSkill;

impl SkillSelector for PreferBasicSkill {
    fn strategy_name(&self) -> &'static str {
        "PreferBasic"
    }

    fn select<'a>(
        &self,
        skill_ids: &'a [String],
        cooldowns: &SkillCooldowns,
        _priority: &'a [String],
    ) -> &'a str {
        // 优先基础攻击
        if cooldowns.get(BASIC_ATTACK_ID) == 0 {
            BASIC_ATTACK_ID
        } else {
            skill_ids
                .iter()
                .find(|id| cooldowns.get(id) == 0)
                .map(|s| s.as_str())
                .unwrap_or(BASIC_ATTACK_ID)
        }
    }
}

/// 按优先级排序选择技能
pub(crate) struct ByPrioritySkill;

impl SkillSelector for ByPrioritySkill {
    fn strategy_name(&self) -> &'static str {
        "ByPriority"
    }

    fn select<'a>(
        &self,
        skill_ids: &'a [String],
        cooldowns: &SkillCooldowns,
        priority: &'a [String],
    ) -> &'a str {
        // 按优先级列表选择
        if !priority.is_empty() {
            for preferred in priority {
                if skill_ids.iter().any(|id| id == preferred) && cooldowns.get(preferred) == 0 {
                    return preferred.as_str();
                }
            }
        }
        // 回退：优先特殊技能
        skill_ids
            .iter()
            .find(|id| *id != BASIC_ATTACK_ID && cooldowns.get(id) == 0)
            .map(|s| s.as_str())
            .unwrap_or(BASIC_ATTACK_ID)
    }
}

// ── 策略注册表 ────────────────────────────────────────────────

/// AI 策略注册表资源：管理所有策略 trait 对象
#[derive(Resource)]
pub(crate) struct AiStrategyRegistry {
    pub(crate) target_selectors: HashMap<String, Box<dyn TargetSelector>>,
    pub(crate) move_selectors: HashMap<String, Box<dyn MoveSelector>>,
    pub(crate) skill_selectors: HashMap<String, Box<dyn SkillSelector>>,
}

impl Default for AiStrategyRegistry {
    fn default() -> Self {
        let mut registry = Self {
            target_selectors: HashMap::new(),
            move_selectors: HashMap::new(),
            skill_selectors: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }
}

impl AiStrategyRegistry {
    /// 注册内置默认策略
    pub(crate) fn register_defaults(&mut self) {
        // 目标选择策略
        self.register_target_selector(Box::new(NearestTarget));
        self.register_target_selector(Box::new(WeakestTarget));
        self.register_target_selector(Box::new(MostDangerousTarget));
        self.register_target_selector(Box::new(LowestHpPercentTarget));

        // 移动策略
        self.register_move_selector(Box::new(AggressiveMove));
        self.register_move_selector(Box::new(CautiousMove));
        self.register_move_selector(Box::new(SupportMove));

        // 技能选择策略
        self.register_skill_selector(Box::new(PreferSpecialSkill));
        self.register_skill_selector(Box::new(PreferBasicSkill));
        self.register_skill_selector(Box::new(ByPrioritySkill));
    }

    /// 注册目标选择策略
    pub(crate) fn register_target_selector(&mut self, selector: Box<dyn TargetSelector>) {
        let name = selector.strategy_name().to_string();
        self.target_selectors.insert(name, selector);
    }

    /// 注册移动策略
    pub(crate) fn register_move_selector(&mut self, selector: Box<dyn MoveSelector>) {
        let name = selector.strategy_name().to_string();
        self.move_selectors.insert(name, selector);
    }

    /// 注册技能选择策略
    pub(crate) fn register_skill_selector(&mut self, selector: Box<dyn SkillSelector>) {
        let name = selector.strategy_name().to_string();
        self.skill_selectors.insert(name, selector);
    }

    /// 按名称获取目标选择策略
    pub(crate) fn target_selector(&self, name: &str) -> &dyn TargetSelector {
        self.target_selectors
            .get(name)
            .unwrap_or_else(|| {
                self.target_selectors.get("Nearest").unwrap_or_else(|| {
                    bevy::log::error!(
                        target: "ai",
                        event = "ai_strategy_not_found",
                        strategy = "Nearest",
                        "Nearest 策略未注册"
                    );
                    panic!("Nearest 策略必须注册")
                })
            })
            .as_ref()
    }

    /// 按名称获取移动策略
    pub(crate) fn move_selector(&self, name: &str) -> &dyn MoveSelector {
        self.move_selectors
            .get(name)
            .unwrap_or_else(|| {
                self.move_selectors.get("Aggressive").unwrap_or_else(|| {
                    bevy::log::error!(
                        target: "ai",
                        event = "ai_strategy_not_found",
                        strategy = "Aggressive",
                        "Aggressive 策略未注册"
                    );
                    panic!("Aggressive 策略必须注册")
                })
            })
            .as_ref()
    }

    /// 按名称获取技能选择策略
    pub(crate) fn skill_selector(&self, name: &str) -> &dyn SkillSelector {
        self.skill_selectors
            .get(name)
            .unwrap_or_else(|| {
                self.skill_selectors
                    .get("PreferSpecial")
                    .unwrap_or_else(|| {
                        bevy::log::error!(
                            target: "ai",
                            event = "ai_strategy_not_found",
                            strategy = "PreferSpecial",
                            "PreferSpecial 策略未注册"
                        );
                        panic!("PreferSpecial 策略必须注册")
                    })
            })
            .as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 策略注册表_默认注册所有策略() {
        let registry = AiStrategyRegistry::default();

        // 目标选择策略
        assert!(registry.target_selectors.contains_key("Nearest"));
        assert!(registry.target_selectors.contains_key("Weakest"));
        assert!(registry.target_selectors.contains_key("MostDangerous"));
        assert!(registry.target_selectors.contains_key("LowestHpPercent"));

        // 移动策略
        assert!(registry.move_selectors.contains_key("Aggressive"));
        assert!(registry.move_selectors.contains_key("Cautious"));
        assert!(registry.move_selectors.contains_key("Support"));

        // 技能选择策略
        assert!(registry.skill_selectors.contains_key("PreferSpecial"));
        assert!(registry.skill_selectors.contains_key("PreferBasic"));
        assert!(registry.skill_selectors.contains_key("ByPriority"));
    }

    #[test]
    fn 策略注册表_按名称查找() {
        let registry = AiStrategyRegistry::default();

        let selector = registry.target_selector("Nearest");
        assert_eq!(selector.strategy_name(), "Nearest");

        let selector = registry.move_selector("Cautious");
        assert_eq!(selector.strategy_name(), "Cautious");

        let selector = registry.skill_selector("ByPriority");
        assert_eq!(selector.strategy_name(), "ByPriority");
    }

    #[test]
    fn 策略注册表_未知名称回退默认() {
        let registry = AiStrategyRegistry::default();

        // 未知目标策略回退到 Nearest
        let selector = registry.target_selector("UnknownStrategy");
        assert_eq!(selector.strategy_name(), "Nearest");

        // 未知移动策略回退到 Aggressive
        let selector = registry.move_selector("UnknownStrategy");
        assert_eq!(selector.strategy_name(), "Aggressive");

        // 未知技能策略回退到 PreferSpecial
        let selector = registry.skill_selector("UnknownStrategy");
        assert_eq!(selector.strategy_name(), "PreferSpecial");
    }
}
