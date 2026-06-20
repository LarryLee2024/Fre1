//! Targeting Selector — 目标选择与校验纯函数
//!
//! 提供目标筛选、合法性校验、优先级排序和最终选择的核心逻辑。
//! 遵循 docs/02-domain/capabilities/targeting_domain.md §5 的流程定义。
//!
//! 核心函数：
//! - select_targets() — 主入口：筛选 → 校验 → 排序 → 截断
//! - filter_by_type() — 按 TargetType 初筛
//! - validate_candidate() — 候选目标合法性校验（射程/阵营/视野）
//! - apply_priority() — 按优先级规则排序
//! - truncate_by_limit() — 按数量上限截断

use bevy::prelude::{Commands, Entity};

use crate::core::capabilities::targeting::events::{NoValidTarget, TargetSelected};
use crate::core::capabilities::targeting::foundation::error::TargetingError;
use crate::core::capabilities::targeting::foundation::{
    EntityTarget, PriorityRule, TargetContext, TargetData, TargetType, TargetingDef,
    ValidationResult,
};

/// 候选目标描述——选择前的原始目标数据。
#[derive(Debug, Clone)]
pub struct CandidateTarget {
    /// 实体标识
    pub entity_id: String,
    /// 实体位置（网格坐标字符串，如 "5,3"）
    pub position: String,
    /// 与施法者的距离
    pub distance: f32,
    /// 阵营标识
    pub faction: String,
    /// 是否存活
    pub alive: bool,
    /// 生命值（用于血量优先级排序）
    pub health: Option<f32>,
    /// 最大生命值
    pub max_health: Option<f32>,
    /// 是否为施法者自身
    pub is_caster: bool,
}

impl CandidateTarget {
    /// 创建候选目标。
    pub fn new(entity_id: impl Into<String>) -> Self {
        Self {
            entity_id: entity_id.into(),
            position: String::new(),
            distance: 0.0,
            faction: String::new(),
            alive: true,
            health: None,
            max_health: None,
            is_caster: false,
        }
    }

    /// 设置位置。
    pub fn with_position(mut self, pos: impl Into<String>) -> Self {
        self.position = pos.into();
        self
    }

    /// 设置距离。
    pub fn with_distance(mut self, distance: f32) -> Self {
        self.distance = distance;
        self
    }

    /// 设置阵营。
    pub fn with_faction(mut self, faction: impl Into<String>) -> Self {
        self.faction = faction.into();
        self
    }

    /// 设置存活状态。
    pub fn with_alive(mut self, alive: bool) -> Self {
        self.alive = alive;
        self
    }

    /// 设置生命值。
    pub fn with_health(mut self, health: f32, max_health: f32) -> Self {
        self.health = Some(health);
        self.max_health = Some(max_health);
        self
    }

    /// 标记为施法者。
    pub fn with_is_caster(mut self, value: bool) -> Self {
        self.is_caster = value;
        self
    }
}

// ============================================================================
// 主入口：select_targets
// ============================================================================

/// 执行完整的目标选择流程。
///
/// 流程（docs/02-domain/capabilities/targeting_domain.md §5.1）：
/// 1. 根据 TargetType 筛选候选目标池
/// 2. 根据 TargetShape 计算影响范围
/// 3. 对候选目标执行合法性校验
/// 4. 按优先级排序
/// 5. 限制最终目标数量不超过上限
/// 6. 封装结果到 TargetData
///
/// # Errors
/// - `TargetingError::NoValidTargets` 当无合法目标时。
pub fn select_targets(
    def: &TargetingDef,
    candidates: Vec<CandidateTarget>,
    context: TargetContext,
    entity: Entity,
    ability_id: &str,
    commands: &mut Commands,
) -> Result<TargetData, TargetingError> {
    // 1. 按 TargetType 初筛
    let filtered = filter_by_type(def, &candidates, &context.caster_faction);

    if filtered.is_empty() {
        let reason = format!("no candidates match target type {:?}", def.target_type);
        commands.trigger(NoValidTarget {
            entity,
            ability_id: ability_id.to_string(),
            fail_reason: reason.clone(),
        });
        return Err(TargetingError::NoValidTargets { reason });
    }

    // 2. 校验候选目标
    let mut valid: Vec<EntityTarget> = Vec::new();
    for candidate in &filtered {
        match validate_candidate(def, candidate) {
            ValidationResult::Pass => {
                valid.push(EntityTarget {
                    entity_id: candidate.entity_id.clone(),
                    position: candidate.position.clone(),
                    distance: candidate.distance,
                    selection_order: 0,
                });
            }
            ValidationResult::Fail(_) => {
                // 跳过不合法目标
            }
        }
    }

    if valid.is_empty() {
        let reason: String = "all candidates failed validation checks".into();
        commands.trigger(NoValidTarget {
            entity,
            ability_id: ability_id.to_string(),
            fail_reason: reason.clone(),
        });
        return Err(TargetingError::NoValidTargets { reason });
    }

    // 3. 按优先级排序
    if let Some(rule) = &def.priority_rule {
        apply_priority(rule, &mut valid, &filtered);
    }

    // 4. 按数量上限截断
    truncate_by_limit(&def.max_targets, &mut valid);

    // 5. 更新选择顺序
    for (i, target) in valid.iter_mut().enumerate() {
        target.selection_order = i as u32 + 1;
    }

    // 6. 提取位置列表
    let positions: Vec<String> = valid.iter().map(|t| t.position.clone()).collect();

    let target_count = valid.len() as u32;
    let first_target = valid.first().map(|t| t.entity_id.clone());

    commands.trigger(TargetSelected {
        entity,
        ability_id: ability_id.to_string(),
        target_count,
        first_target,
    });

    Ok(TargetData::with_targets(valid, positions, context))
}

// ============================================================================
// 按 TargetType 筛选
// ============================================================================

/// 按 TargetType 从候选池中筛选合法目标。
///
/// 处理规则：
/// - Self_：仅施法者自身
/// - Ally：同阵营且非自身
/// - Enemy：不同阵营
/// - Dead：已死亡的实体
/// - Any：所有实体
/// - Party：小队全体（同阵营所有，含自身）
///
/// 同时处理 include_self（是否包含施法者自身）、allow_dead_targets（是否包含死亡实体）。
pub fn filter_by_type<'a>(
    def: &TargetingDef,
    candidates: &'a [CandidateTarget],
    caster_faction: &str,
) -> Vec<&'a CandidateTarget> {
    candidates
        .iter()
        .filter(|c| {
            // 死亡实体过滤
            if !c.alive && !def.allow_dead_targets && !matches!(def.target_type, TargetType::Dead) {
                return false;
            }

            // 施法者自身过滤
            if c.is_caster && !def.include_self {
                return false;
            }

            // TargetType 匹配
            match def.target_type {
                TargetType::Self_ => c.is_caster,
                TargetType::Ally => !c.is_caster && c.faction == caster_faction,
                TargetType::Enemy => !c.is_caster && c.faction != caster_faction,
                TargetType::Dead => !c.alive,
                TargetType::Neutral => {
                    c.faction.is_empty() || (c.faction != caster_faction && !c.is_caster)
                }
                TargetType::Any => true,
                TargetType::Summon => true,
                TargetType::Party => true,
            }
        })
        .collect()
}

// ============================================================================
// 候选目标合法性校验
// ============================================================================

/// 校验单个候选目标是否合法。
///
/// 校验项（不变量 3.1-3.5）：
/// 1. 实体存在性（由调用方保证候选目标的有效性）
/// 2. 射程校验（不变量 3.2）
/// 3. 阵营一致性（不变量 3.3）
/// 4. 视野校验（不变量 3.5，由调用方提供）
///
/// 注意：由于网格/阵营系统尚未实现，部分校验使用占位逻辑。
pub fn validate_candidate(def: &TargetingDef, candidate: &CandidateTarget) -> ValidationResult {
    // 射程校验（不变量 3.2）
    if let Some(max_range) = def.range
        && candidate.distance > max_range
    {
        return ValidationResult::fail(format!(
            "distance {} exceeds max range {}",
            candidate.distance, max_range
        ));
    }

    if let Some(min_range) = def.min_range
        && candidate.distance < min_range
    {
        return ValidationResult::fail(format!(
            "distance {} is below min range {}",
            candidate.distance, min_range
        ));
    }

    ValidationResult::Pass
}

// ============================================================================
// 优先级排序
// ============================================================================

/// 按优先级规则对候选目标排序。
///
/// 使用稳定的比较排序，同优先级保持原始顺序。
pub fn apply_priority(
    rule: &PriorityRule,
    targets: &mut [EntityTarget],
    candidates: &[&CandidateTarget],
) {
    // 建立 entity_id → CandidateTarget 的映射
    let candidate_map: std::collections::HashMap<&str, &CandidateTarget> = candidates
        .iter()
        .copied()
        .map(|c| (c.entity_id.as_str(), c))
        .collect();

    match rule {
        PriorityRule::Nearest => {
            targets.sort_by(|a, b| {
                a.distance
                    .partial_cmp(&b.distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        PriorityRule::Farthest => {
            targets.sort_by(|a, b| {
                b.distance
                    .partial_cmp(&a.distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        PriorityRule::LowestHealth => {
            targets.sort_by(|a, b| {
                let hp_a = candidate_map
                    .get(a.entity_id.as_str())
                    .and_then(|c| c.health)
                    .unwrap_or(f32::MAX);
                let hp_b = candidate_map
                    .get(b.entity_id.as_str())
                    .and_then(|c| c.health)
                    .unwrap_or(f32::MAX);
                hp_a.partial_cmp(&hp_b).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        PriorityRule::HighestHealth => {
            targets.sort_by(|a, b| {
                let hp_a = candidate_map
                    .get(a.entity_id.as_str())
                    .and_then(|c| c.health)
                    .unwrap_or(0.0);
                let hp_b = candidate_map
                    .get(b.entity_id.as_str())
                    .and_then(|c| c.health)
                    .unwrap_or(0.0);
                hp_b.partial_cmp(&hp_a).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        PriorityRule::Random => {
            // Random 使用确定性排序（按 entity_id 的哈希值）
            targets.sort_by(|a, b| a.entity_id.cmp(&b.entity_id));
        }
    }
}

// ============================================================================
// 数量限制
// ============================================================================

/// 按最大目标数限制截断列表。
///
/// 不变量 3.4: 选中的目标数量不得超过最大目标数上限。
/// 保留前 max_targets 个目标（已按优先级排序）。
pub fn truncate_by_limit(max_targets: &u32, targets: &mut Vec<EntityTarget>) {
    let limit = *max_targets as usize;
    if targets.len() > limit {
        targets.truncate(limit);
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 校验 TargetingDef 参数的全面合法性。
///
/// 校验项：
/// - V1: 形状参数合法
/// - V2: 最大目标数 ≥ 1
/// - V3: min_range ≤ range
/// - V4: Single 形状时 max_targets = 1
pub fn validate_targeting_def(def: &TargetingDef) -> Result<(), TargetingError> {
    // V1: 形状参数合法
    def.shape.validate()?;

    // V2: 最大目标数 ≥ 1
    if def.max_targets < 1 {
        return Err(TargetingError::InvalidMaxTargets(def.max_targets));
    }

    // V3: min_range ≤ range
    if let (Some(min), Some(max)) = (def.min_range, def.range)
        && min > max
    {
        return Err(TargetingError::InvalidRange {
            min: Some(min),
            max: Some(max),
            detail: format!("min_range {} > range {}", min, max),
        });
    }

    // V4: Single 形状时 max_targets = 1
    if def.shape.is_single() && def.max_targets != 1 {
        return Err(TargetingError::InvalidShapeParameter {
            shape: def.shape.name().into(),
            param: "max_targets",
            detail: format!(
                "Single shape requires max_targets = 1, got {}",
                def.max_targets
            ),
        });
    }

    Ok(())
}
