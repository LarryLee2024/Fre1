//! DamagePolicy — 伤害计算策略
//!
//! 封装伤害公式选择、暴击判定、减免计算等规则。
//! 遵循 Policy 模式：单一 `evaluate()` 入口，返回结构化决策结果。
//!
//! # Policy 模式
//!
//! 参考 economy 域的 `RestockPolicy` 枚举模式，但此处使用 struct + 静态方法，
//! 因为伤害计算是纯算法，不需要变体配置。
//!
//! 详见 docs/02-domain/domains/combat_domain.md

use std::collections::HashMap;

// ─── 伤害类型 ──────────────────────────────────────────────────────

/// 伤害类型分类。
///
/// 用于减免计算、技能交互、标签匹配。
/// 此处定义为领域级枚举，避免直接从 capabilities 暴露位标签复杂性。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageType {
    /// 物理伤害（劈砍、钝击、穿刺等物理来源）
    Physical,
    /// 火焰伤害
    Fire,
    /// 冰霜伤害
    Ice,
    /// 闪电伤害
    Lightning,
    /// 暗影/负能量伤害
    Shadow,
    /// 光耀/正能量伤害
    Radiant,
    /// 毒素伤害
    Poison,
    /// 心灵伤害
    Psychic,
    /// 真实伤害（无视减免与抗性）
    True,
}

impl DamageType {
    /// 返回该伤害类型对应的修饰标签前缀（用于查找 modifier rule）。
    pub fn modifier_tag(&self) -> &'static str {
        match self {
            Self::Physical => "dmg_physical",
            Self::Fire => "dmg_fire",
            Self::Ice => "dmg_ice",
            Self::Lightning => "dmg_lightning",
            Self::Shadow => "dmg_shadow",
            Self::Radiant => "dmg_radiant",
            Self::Poison => "dmg_poison",
            Self::Psychic => "dmg_psychic",
            Self::True => "dmg_true",
        }
    }
}

// ─── 伤害上下文 ────────────────────────────────────────────────────

/// DamagePolicy::evaluate() 的输入上下文。
///
/// 纯值对象，不含 ECS 引用。调用方负责从 ECS 提取必要字段后传入。
#[derive(Debug, Clone)]
pub struct DamageContext {
    /// 攻击方实体 ID（用于追踪/日志，不参与计算）
    pub attacker: u64,
    /// 目标方实体 ID（用于追踪/日志，不参与计算）
    pub target: u64,
    /// 基础伤害（技能/武器基础值）
    pub base_damage: u32,
    /// 暴击率（0.0 ~ 1.0）
    pub critical_chance: f32,
    /// 暴击判定值（0.0 ~ 1.0，由调用方通过确定性 RNG 预生成）。
    ///
    /// 当 `critical_roll <= critical_chance` 时判定为暴击。
    /// 这样 Policy 保持纯函数，随机性由调用方控制（replay-safe）。
    pub critical_roll: f32,
    /// 暴击倍率（如 2.0 表示双倍）
    pub critical_multiplier: f32,
    /// 伤害类型
    pub damage_type: DamageType,
    /// 攻击方属性快照（如 "strength": 18.0, "phys_atk": 120.0）
    pub attacker_stats: HashMap<String, f32>,
    /// 目标方属性快照（如 "phys_def": 80.0, "resistance_fire": 0.5）
    pub target_stats: HashMap<String, f32>,
}

impl DamageContext {
    /// 创建最小化上下文（用于快速调用，默认无属性加成）。
    pub fn simple(attacker: u64, target: u64, base_damage: u32, damage_type: DamageType) -> Self {
        Self {
            attacker,
            target,
            base_damage,
            critical_chance: 0.0,
            critical_roll: 1.0,
            critical_multiplier: 1.0,
            damage_type,
            attacker_stats: HashMap::new(),
            target_stats: HashMap::new(),
        }
    }
}

// ─── 伤害决策结果 ───────────────────────────────────────────────────

/// DamagePolicy::evaluate() 的输出决策。
///
/// 包含完整的伤害分解链，用于日志、回放和 UI 显示。
#[derive(Debug, Clone)]
pub struct DamageDecision {
    /// 基础伤害（来自技能/武器）
    pub base_damage: u32,
    /// 是否暴击
    pub is_critical: bool,
    /// 暴击倍率
    pub critical_multiplier: f32,
    /// 减免量（目标防御/抗性抵消的伤害）
    pub mitigated_amount: u32,
    /// 最终伤害（实际扣除 HP 的量）
    pub final_damage: u32,
    /// 分步计算说明（用于日志/调试/UI 详情）
    pub breakdown: Vec<String>,
}

// ─── 伤害策略 ──────────────────────────────────────────────────────

/// 伤害计算策略。
///
/// 纯函数集合，封装伤害公式、暴击判定、减免计算等规则。
/// 零 ECS 依赖，输入输出均为值类型。
pub struct DamagePolicy;

impl DamagePolicy {
    /// 执行完整的伤害计算流程。
    ///
    /// 来源：D&D 5e SRD §9 + ADR-023 §2
    ///
    /// 管线步骤：
    /// 1. 暴击判定 → 基础伤害 × 暴击倍率
    /// 2. 属性加成（攻击方力量/法强等）
    /// 3. 减免计算（目标防御/抗性）
    /// 4. 最终伤害裁定
    pub fn evaluate(ctx: &DamageContext) -> DamageDecision {
        let mut breakdown: Vec<String> = Vec::new();

        breakdown.push(format!("基础伤害: {}", ctx.base_damage));

        // ── Step 1: 暴击判定 ──
        // 通过比较 critical_roll 与 critical_chance 确定是否暴击。
        // critical_roll 由调用方通过确定性 RNG 预生成，保证 replay 安全。
        let is_critical = ctx.critical_chance > 0.0 && ctx.critical_roll < ctx.critical_chance;
        let after_crit = if is_critical {
            let crit_value = (ctx.base_damage as f32 * ctx.critical_multiplier).ceil() as u32;
            breakdown.push(format!(
                "暴击! 倍率={}, 暴击后伤害={}",
                ctx.critical_multiplier, crit_value
            ));
            crit_value
        } else {
            if ctx.critical_chance > 0.0 {
                breakdown.push("未暴击".to_string());
            }
            ctx.base_damage
        };

        // ── Step 2: 属性加成 ──
        let attack_bonus = Self::calc_attack_bonus(&ctx.attacker_stats, ctx.damage_type);
        let after_bonus = after_crit.saturating_add(attack_bonus);
        if attack_bonus > 0 {
            breakdown.push(format!("属性加成: +{} (来源属性加成)", attack_bonus));
        }

        // ── Step 3: 减免计算 ──
        let mitigation_amount =
            Self::calc_mitigation_amount(after_bonus, &ctx.target_stats, ctx.damage_type);
        let final_damage = after_bonus.saturating_sub(mitigation_amount);
        if mitigation_amount > 0 {
            breakdown.push(format!(
                "减免: {} → {} (抵消 {})",
                after_bonus, final_damage, mitigation_amount,
            ));
        }

        breakdown.push(format!("最终伤害: {}", final_damage));

        DamageDecision {
            base_damage: ctx.base_damage,
            is_critical,
            critical_multiplier: ctx.critical_multiplier,
            mitigated_amount: mitigation_amount,
            final_damage,
            breakdown,
        }
    }

    /// 计算攻击方属性提供的伤害加成。
    ///
    /// - 物理伤害：查询 `phys_atk` 或 `strength`
    /// - 魔法伤害：查询 `magic_atk` 或 `intelligence`
    /// - 真实伤害：无属性加成
    fn calc_attack_bonus(stats: &HashMap<String, f32>, damage_type: DamageType) -> u32 {
        match damage_type {
            DamageType::True => 0,
            DamageType::Physical => stats
                .get("phys_atk")
                .or_else(|| stats.get("strength"))
                .map(|v| *v as u32)
                .unwrap_or(0),
            DamageType::Fire
            | DamageType::Ice
            | DamageType::Lightning
            | DamageType::Shadow
            | DamageType::Radiant
            | DamageType::Poison
            | DamageType::Psychic => stats
                .get("magic_atk")
                .or_else(|| stats.get("intelligence"))
                .map(|v| *v as u32)
                .unwrap_or(0),
        }
    }

    /// 计算目标防御的减免量（减免量 = base_damage - after_defense_damage）。
    ///
    /// 公式：减免量 = floor(def / (def + 100) * incoming_damage)
    /// 即防御越高减免比率越大，但永远无法达到 100%。
    fn calc_mitigation_amount(
        incoming_damage: u32,
        target_stats: &HashMap<String, f32>,
        damage_type: DamageType,
    ) -> u32 {
        match damage_type {
            DamageType::True => 0,
            DamageType::Physical => {
                let def = target_stats.get("phys_def").copied().unwrap_or(0.0);
                Self::def_to_mitigation(def)
            }
            DamageType::Fire
            | DamageType::Ice
            | DamageType::Lightning
            | DamageType::Shadow
            | DamageType::Radiant
            | DamageType::Poison
            | DamageType::Psychic => {
                let def = target_stats.get("magic_def").copied().unwrap_or(0.0);
                Self::def_to_mitigation(def)
            }
        }
    }

    /// 将防御值转换为减免量。
    ///
    /// 公式：减免量 = floor(def / (def + 100) * incoming_damage)
    /// 即防御越高减免比率越大，但永远无法达到 100%。
    fn def_to_mitigation(defense: f32) -> u32 {
        if defense <= 0.0 {
            return 0;
        }
        // 按比例减免：每 100 点防御约减免 50%，200 点约 66%
        (defense / (defense + 100.0) * 100.0).floor() as u32
    }
}
