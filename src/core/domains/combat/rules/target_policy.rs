//! TargetPolicy — 目标选择与合法性判定策略
//!
//! 封装目标选择逻辑：是否可选定、如何筛选有效目标。
//! 遵循 Policy 模式：`can_target()` 返回结构化决策，`find_valid_targets()` 返回候选列表。
//!
//! # Policy 模式
//!
//! 与 DamagePolicy 一致，纯函数集合。参考 economy 域 `RestockPolicy` 模式。
//!
//! 详见 docs/02-domain/domains/combat_domain.md

use std::collections::HashSet;

// ─── 队伍关系 ──────────────────────────────────────────────────────

/// 目标与攻击方的队伍关系。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetRelation {
    /// 同队（盟友）
    Ally,
    /// 敌对
    Enemy,
    /// 中立
    Neutral,
    /// 自己
    Itself,
}

// ─── 目标选择上下文 ────────────────────────────────────────────────

/// TargetPolicy 的输入上下文。
///
/// 纯值对象，不含 ECS 引用。调用方负责从 ECS 提取必要字段后传入。
#[derive(Debug, Clone)]
pub struct TargetContext {
    /// 攻击方实体 ID
    pub source: u64,
    /// 目标实体 ID
    pub target: u64,
    /// 攻击方队伍 ID
    pub source_team: u64,
    /// 目标方队伍 ID
    pub target_team: u64,
    /// 攻击方与目标的关系过滤
    pub allowed_relations: Vec<TargetRelation>,
    /// 与目标的距离（网格单位）
    pub distance: u32,
    /// 技能/攻击的最大距离
    pub max_range: u32,
    /// 技能/攻击的最小距离
    pub min_range: u32,
    /// 是否有视线
    pub has_line_of_sight: bool,
    /// 目标是否存活（无 Dead 标签）
    pub target_alive: bool,
    /// 目标是否可被选定（无 Untargetable 标签）
    pub target_is_targetable: bool,
    /// 目标的额外标签（用于更精细的过滤）
    pub target_tags: HashSet<String>,
    /// 要求的标签（目标必须包含此标签才可被选定）
    pub required_tags: Vec<String>,
    /// 禁止的标签（目标包含此标签时不可选定）
    pub forbidden_tags: Vec<String>,
}

impl TargetContext {
    /// 创建简化上下文（用于快速检查，使用默认的通用过滤）。
    pub fn simple(
        source: u64,
        target: u64,
        source_team: u64,
        target_team: u64,
        distance: u32,
        max_range: u32,
        has_line_of_sight: bool,
    ) -> Self {
        let relation = if source_team == target_team {
            if source == target {
                TargetRelation::Itself
            } else {
                TargetRelation::Ally
            }
        } else {
            TargetRelation::Enemy
        };

        Self {
            source,
            target,
            source_team,
            target_team,
            allowed_relations: vec![relation],
            distance,
            max_range,
            min_range: 0,
            has_line_of_sight,
            target_alive: true,
            target_is_targetable: true,
            target_tags: HashSet::new(),
            required_tags: Vec::new(),
            forbidden_tags: Vec::new(),
        }
    }
}

// ─── 目标选择决策 ───────────────────────────────────────────────────

/// TargetPolicy::can_target() 的输出决策。
#[derive(Debug, Clone)]
pub struct TargetDecision {
    /// 是否可以选定目标
    pub allowed: bool,
    /// 拒绝原因（允许时为 None）
    pub reason: Option<TargetRejection>,
    /// 分步说明
    pub breakdown: Vec<String>,
}

/// 目标被拒绝的具体原因。
#[derive(Debug, Clone, PartialEq)]
pub enum TargetRejection {
    /// 超出范围
    OutOfRange,
    /// 视线被阻挡
    NoLineOfSight,
    /// 目标已死亡
    TargetDead,
    /// 目标不可选定
    TargetUntargetable,
    /// 目标被禁止（包含 forbidden_tags 中的标签）
    TargetForbidden,
    /// 目标缺少必需标签
    MissingRequiredTag,
    /// 关系不匹配（非友非敌）
    RelationMismatch,
    /// 自身目标不合法
    InvalidSelfTarget,
    /// 缺乏有效范围
    InvalidRange,
}

impl TargetRejection {
    /// 返回人类可读的描述。
    pub fn description(&self) -> &'static str {
        match self {
            Self::OutOfRange => "目标超出攻击范围",
            Self::NoLineOfSight => "视线被阻挡",
            Self::TargetDead => "目标已死亡",
            Self::TargetUntargetable => "目标不可选定",
            Self::TargetForbidden => "目标类型不被允许",
            Self::MissingRequiredTag => "目标缺少必需标记",
            Self::RelationMismatch => "目标关系不匹配",
            Self::InvalidSelfTarget => "不能以自身为目标",
            Self::InvalidRange => "目标在最小范围外",
        }
    }

    /// 返回机器可读的错误码。
    pub fn code(&self) -> &'static str {
        match self {
            Self::OutOfRange => "TARGET_OUT_OF_RANGE",
            Self::NoLineOfSight => "TARGET_NO_LOS",
            Self::TargetDead => "TARGET_DEAD",
            Self::TargetUntargetable => "TARGET_UNTARGETABLE",
            Self::TargetForbidden => "TARGET_FORBIDDEN",
            Self::MissingRequiredTag => "TARGET_MISSING_TAG",
            Self::RelationMismatch => "TARGET_RELATION_MISMATCH",
            Self::InvalidSelfTarget => "TARGET_SELF_INVALID",
            Self::InvalidRange => "TARGET_INVALID_RANGE",
        }
    }
}

// ─── 目标策略 ──────────────────────────────────────────────────────

/// 目标选择与合法性判定策略。
///
/// 纯函数集合，封装距离检查、视线判定、队伍关系过滤、标签过滤等规则。
/// 零 ECS 依赖，输入输出均为值类型。
pub struct TargetPolicy;

impl TargetPolicy {
    /// 判断特定目标是否可被选定。
    ///
    /// 检查顺序：
    /// 1. 存活状态
    /// 2. 目标可选定性
    /// 3. 队伍关系
    /// 4. 距离范围
    /// 5. 视线
    /// 6. 标签过滤
    pub fn can_target(ctx: &TargetContext) -> TargetDecision {
        let mut breakdown: Vec<String> = Vec::new();

        breakdown.push(format!(
            "源={}, 目标={}, 距离={}, 最大范围={}",
            ctx.source, ctx.target, ctx.distance, ctx.max_range
        ));

        // ── Step 1: 存活检查 ──
        if !ctx.target_alive {
            breakdown.push("失败: 目标已死亡".to_string());
            return TargetDecision {
                allowed: false,
                reason: Some(TargetRejection::TargetDead),
                breakdown,
            };
        }
        breakdown.push("通过: 目标存活".to_string());

        // ── Step 2: 可选定性检查 ──
        if !ctx.target_is_targetable {
            breakdown.push("失败: 目标不可选定".to_string());
            return TargetDecision {
                allowed: false,
                reason: Some(TargetRejection::TargetUntargetable),
                breakdown,
            };
        }
        breakdown.push("通过: 目标可选定".to_string());

        // ── Step 3: 队伍关系检查 ──
        let relation = Self::determine_relation(ctx);
        if !ctx.allowed_relations.contains(&relation) {
            breakdown.push(format!("失败: 关系 {:?} 不允许", relation));
            return TargetDecision {
                allowed: false,
                reason: Some(TargetRejection::RelationMismatch),
                breakdown,
            };
        }
        breakdown.push(format!("通过: 关系 {:?} 允许", relation));

        // ── Step 4: 距离检查 ──
        if ctx.distance > ctx.max_range {
            breakdown.push(format!(
                "失败: 距离 {} > 最大范围 {}",
                ctx.distance, ctx.max_range
            ));
            return TargetDecision {
                allowed: false,
                reason: Some(TargetRejection::OutOfRange),
                breakdown,
            };
        }
        if ctx.distance < ctx.min_range {
            breakdown.push(format!(
                "失败: 距离 {} < 最小范围 {}",
                ctx.distance, ctx.min_range
            ));
            return TargetDecision {
                allowed: false,
                reason: Some(TargetRejection::InvalidRange),
                breakdown,
            };
        }
        breakdown.push(format!("通过: 距离 {} 在有效范围内", ctx.distance));

        // ── Step 5: 视线检查 ──
        if !ctx.has_line_of_sight {
            breakdown.push("失败: 视线被阻挡".to_string());
            return TargetDecision {
                allowed: false,
                reason: Some(TargetRejection::NoLineOfSight),
                breakdown,
            };
        }
        breakdown.push("通过: 视线通畅".to_string());

        // ── Step 6: 标签过滤 ──
        // 必需标签检查
        for required in &ctx.required_tags {
            if !ctx.target_tags.contains(required) {
                breakdown.push(format!("失败: 缺少必需标签 '{}'", required));
                return TargetDecision {
                    allowed: false,
                    reason: Some(TargetRejection::MissingRequiredTag),
                    breakdown,
                };
            }
        }
        // 禁止标签检查
        for forbidden in &ctx.forbidden_tags {
            if ctx.target_tags.contains(forbidden) {
                breakdown.push(format!("失败: 包含禁止标签 '{}'", forbidden));
                return TargetDecision {
                    allowed: false,
                    reason: Some(TargetRejection::TargetForbidden),
                    breakdown,
                };
            }
        }
        breakdown.push("通过: 标签过滤".to_string());

        breakdown.push("结论: 目标可选定".to_string());

        TargetDecision {
            allowed: true,
            reason: None,
            breakdown,
        }
    }

    /// 查找所有有效目标（简化版：仅作演示，实际应由调用方提供候选列表）。
    ///
    /// 调用方负责遍历候选目标列表，对每个目标调用 `can_target()`。
    /// 此方法仅提供辅助过滤逻辑的封装示意。
    pub fn find_valid_targets(_ctx: &TargetContext, candidates: &[u64]) -> Vec<u64> {
        candidates
            .iter()
            .filter(|&&_candidate_id| {
                // 这里仅是演示，实际应为每个候选构建独立的 TargetContext
                // 调用方负责组装每个候选的上下文
                true
            })
            .copied()
            .collect()
    }

    /// 判断两个队伍之间的关系。
    fn determine_relation(ctx: &TargetContext) -> TargetRelation {
        if ctx.source == ctx.target {
            return TargetRelation::Itself;
        }
        if ctx.source_team == ctx.target_team {
            TargetRelation::Ally
        } else {
            TargetRelation::Enemy
        }
    }
}
