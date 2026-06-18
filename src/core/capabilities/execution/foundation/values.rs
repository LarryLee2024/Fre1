//! Execution 值对象
//!
//! 执行计算参数、上下文、结果等值对象定义。
//!
//! 详见 docs/04-data/capabilities/execution_schema.md §3。

use serde::{Deserialize, Serialize};

use crate::core::capabilities::execution::foundation::types::{ExecutionType, ScalableValue};

/// 伤害计算参数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageParams {
    /// 伤害公式 ID（指向 Domains/rules/ 中的具体公式）
    pub formula_id: String,
    /// 伤害类型标签列表
    pub damage_type: Vec<String>,
    /// 基础伤害骰（如 1d8 = DiceDef { count: 1, sides: 8 }）
    pub damage_dice: Option<DiceDef>,
    /// 固定伤害加值
    pub flat_bonus: Option<ScalableValue>,
    /// 属性修正（如力量修正、智力修正）
    pub attribute_modifier: Option<AttributeModifierDef>,
    /// 是否可暴击
    pub can_critical: bool,
    /// 暴击倍率（V3: ≥ 1.0）
    pub critical_multiplier: f32,
}

impl DamageParams {
    /// 创建新的伤害计算参数。
    ///
    /// # Errors
    /// - V3: critical_multiplier ≥ 1.0
    pub fn new(formula_id: impl Into<String>) -> Self {
        Self {
            formula_id: formula_id.into(),
            damage_type: Vec::new(),
            damage_dice: None,
            flat_bonus: None,
            attribute_modifier: None,
            can_critical: false,
            critical_multiplier: 1.0,
        }
    }

    /// 设置伤害类型标签。
    pub fn with_damage_type(mut self, tags: Vec<String>) -> Self {
        self.damage_type = tags;
        self
    }

    /// 设置伤害骰。
    pub fn with_dice(mut self, dice: DiceDef) -> Self {
        self.damage_dice = Some(dice);
        self
    }

    /// 设置固定伤害加值。
    pub fn with_flat_bonus(mut self, bonus: ScalableValue) -> Self {
        self.flat_bonus = Some(bonus);
        self
    }

    /// 设置属性修正。
    pub fn with_attribute_modifier(mut self, modifier: AttributeModifierDef) -> Self {
        self.attribute_modifier = Some(modifier);
        self
    }

    /// 设置暴击参数。
    pub fn with_critical(mut self, multiplier: f32) -> Self {
        self.can_critical = true;
        self.critical_multiplier = if multiplier >= 1.0 { multiplier } else { 1.0 };
        self
    }

    /// 校验参数合法性。
    ///
    /// - V3: critical_multiplier ≥ 1.0
    pub fn validate(&self) -> Result<(), super::types::ExecutionError> {
        if self.can_critical && self.critical_multiplier < 1.0 {
            return Err(super::types::ExecutionError::InvalidResult(format!(
                "critical_multiplier {} must be >= 1.0",
                self.critical_multiplier
            )));
        }
        Ok(())
    }
}

/// 治疗计算参数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealParams {
    /// 治疗公式 ID
    pub formula_id: String,
    /// 基础治疗量
    pub base_heal: ScalableValue,
    /// 属性修正
    pub attribute_modifier: Option<AttributeModifierDef>,
    /// 是否为临时生命值
    pub is_temporary_hp: bool,
}

impl HealParams {
    /// 创建新的治疗计算参数。
    pub fn new(formula_id: impl Into<String>, base_heal: ScalableValue) -> Self {
        Self {
            formula_id: formula_id.into(),
            base_heal,
            attribute_modifier: None,
            is_temporary_hp: false,
        }
    }

    /// 设置属性修正。
    pub fn with_attribute_modifier(mut self, modifier: AttributeModifierDef) -> Self {
        self.attribute_modifier = Some(modifier);
        self
    }

    /// 设置为临时生命值。
    pub fn with_temporary_hp(mut self, value: bool) -> Self {
        self.is_temporary_hp = value;
        self
    }
}

/// 骰子定义。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DiceDef {
    /// 骰子个数
    pub count: u8,
    /// 骰子面数
    pub sides: u8,
}

impl DiceDef {
    /// 创建骰子定义。
    ///
    /// 校验：count ≥ 1, sides ≥ 2
    pub fn new(count: u8, sides: u8) -> Result<Self, super::types::ExecutionError> {
        if count < 1 {
            return Err(super::types::ExecutionError::InvalidResult(format!(
                "dice count must be ≥ 1, got {}",
                count
            )));
        }
        if sides < 2 {
            return Err(super::types::ExecutionError::InvalidResult(format!(
                "dice sides must be ≥ 2, got {}",
                sides
            )));
        }
        Ok(Self { count, sides })
    }

    /// 计算最大可能骰面值（用于上限判断）。
    pub fn max_roll(&self) -> u32 {
        self.count as u32 * self.sides as u32
    }

    /// 计算最小可能骰面值。
    pub fn min_roll(&self) -> u32 {
        self.count as u32
    }
}

/// 属性修正定义——指定从哪个属性读取修正值及系数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeModifierDef {
    /// 用于修正的属性标识
    pub source_attribute: String,
    /// 修正系数（1.0 = 全值，0.5 = 半值）
    pub multiplier: f32,
    /// 是否使用基础值而非当前值
    pub use_base: bool,
}

impl AttributeModifierDef {
    /// 创建属性修正定义。
    pub fn new(source_attribute: impl Into<String>) -> Self {
        Self {
            source_attribute: source_attribute.into(),
            multiplier: 1.0,
            use_base: false,
        }
    }

    /// 设置系数。
    pub fn with_multiplier(mut self, multiplier: f32) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// 设置使用基础值。
    pub fn with_use_base(mut self, value: bool) -> Self {
        self.use_base = value;
        self
    }

    /// 计算修正值。
    ///
    /// 从 attributes 中读取指定属性的值，乘以系数后返回。
    /// 如果属性不存在，默认返回 0.0。
    pub fn calculate(&self, attributes: &std::collections::HashMap<String, f32>) -> f32 {
        let value = attributes
            .get(&self.source_attribute)
            .copied()
            .unwrap_or(0.0);
        value * self.multiplier
    }
}

/// 执行计算上下文——从 GameplayContext 派生，携带计算所需的全部输入。
///
/// 包含来源/目标属性快照、技能参数、环境因素等。
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionContext {
    /// 执行类型
    pub execution_type: ExecutionType,
    /// 来源实体（施法者/攻击者）标识
    pub source_entity: String,
    /// 目标实体标识
    pub target_entity: String,
    /// 来源属性快照
    pub source_attributes: std::collections::HashMap<String, f32>,
    /// 目标属性快照
    pub target_attributes: std::collections::HashMap<String, f32>,
    /// 来源标签
    pub source_tags: Vec<String>,
    /// 目标标签
    pub target_tags: Vec<String>,
    /// 技能执行参数
    pub ability_params: AbilityExecutionParams,
    /// 环境参数
    pub environment: EnvironmentParams,
}

impl ExecutionContext {
    /// 创建新的执行上下文。
    pub fn new(
        execution_type: ExecutionType,
        source_entity: impl Into<String>,
        target_entity: impl Into<String>,
    ) -> Self {
        Self {
            execution_type,
            source_entity: source_entity.into(),
            target_entity: target_entity.into(),
            source_attributes: std::collections::HashMap::new(),
            target_attributes: std::collections::HashMap::new(),
            source_tags: Vec::new(),
            target_tags: Vec::new(),
            ability_params: AbilityExecutionParams::default(),
            environment: EnvironmentParams::default(),
        }
    }

    /// 设置来源属性快照。
    pub fn with_source_attributes(mut self, attrs: std::collections::HashMap<String, f32>) -> Self {
        self.source_attributes = attrs;
        self
    }

    /// 设置目标属性快照。
    pub fn with_target_attributes(mut self, attrs: std::collections::HashMap<String, f32>) -> Self {
        self.target_attributes = attrs;
        self
    }

    /// 设置标签。
    pub fn with_tags(mut self, source_tags: Vec<String>, target_tags: Vec<String>) -> Self {
        self.source_tags = source_tags;
        self.target_tags = target_tags;
        self
    }

    /// 设置技能参数。
    pub fn with_ability_params(mut self, params: AbilityExecutionParams) -> Self {
        self.ability_params = params;
        self
    }

    /// 设置环境参数。
    pub fn with_environment(mut self, env: EnvironmentParams) -> Self {
        self.environment = env;
        self
    }
}

/// 技能执行参数。
#[derive(Debug, Clone, PartialEq)]
pub struct AbilityExecutionParams {
    /// AbilityDef ID
    pub ability_def_id: Option<String>,
    /// 技能等级
    pub ability_level: u8,
    /// 是否使用了 EffectOverride
    pub has_effect_override: bool,
}

impl Default for AbilityExecutionParams {
    fn default() -> Self {
        Self {
            ability_def_id: None,
            ability_level: 1,
            has_effect_override: false,
        }
    }
}

/// 环境参数。
#[derive(Debug, Clone, PartialEq)]
pub struct EnvironmentParams {
    /// 是否在高地
    pub is_high_ground: bool,
    /// 是否在掩体后
    pub has_cover: bool,
    /// 是否被夹击
    pub is_flanked: bool,
    /// 当前回合数
    pub current_turn: u32,
}

impl Default for EnvironmentParams {
    fn default() -> Self {
        Self {
            is_high_ground: false,
            has_cover: false,
            is_flanked: false,
            current_turn: 0,
        }
    }
}

/// 执行计算的结果。
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionResult {
    /// 执行是否成功
    pub success: bool,
    /// 计算出的数值（伤害量/治疗量等）
    pub value: f32,
    /// 是否为暴击
    pub was_critical: bool,
    /// 是否为未命中
    pub was_miss: bool,
    /// 计算过程追踪（不变量 3.2）
    pub calc_trace: Option<CalcTrace>,
}

impl ExecutionResult {
    /// 创建成功结果。
    pub fn success(value: f32) -> Self {
        Self {
            success: true,
            value,
            was_critical: false,
            was_miss: false,
            calc_trace: None,
        }
    }

    /// 创建失败结果。
    pub fn failure() -> Self {
        Self {
            success: false,
            value: 0.0,
            was_critical: false,
            was_miss: false,
            calc_trace: None,
        }
    }

    /// 标记为暴击。
    pub fn with_critical(mut self, value: bool) -> Self {
        self.was_critical = value;
        self
    }

    /// 标记为未命中。
    pub fn with_miss(mut self, value: bool) -> Self {
        self.was_miss = value;
        self
    }

    /// 添加计算追踪。
    pub fn with_trace(mut self, trace: CalcTrace) -> Self {
        self.calc_trace = Some(trace);
        self
    }
}

/// 计算过程追踪（不变量 3.2: 计算结果可追踪）。
///
/// 记录完整的计算过程日志，包括输入参数、中间值、最终结果。
#[derive(Debug, Clone, PartialEq)]
pub struct CalcTrace {
    /// 使用的公式 ID
    pub formula_id: String,
    /// 输入参数快照
    pub inputs: std::collections::HashMap<String, f32>,
    /// 中间计算步骤
    pub intermediate_values: Vec<(String, f32)>,
    /// 最终输出值
    pub output: f32,
}

impl CalcTrace {
    /// 创建计算追踪。
    pub fn new(formula_id: impl Into<String>) -> Self {
        Self {
            formula_id: formula_id.into(),
            inputs: std::collections::HashMap::new(),
            intermediate_values: Vec::new(),
            output: 0.0,
        }
    }

    /// 记录一个输入参数。
    pub fn with_input(mut self, key: impl Into<String>, value: f32) -> Self {
        self.inputs.insert(key.into(), value);
        self
    }

    /// 记录一个中间值。
    pub fn with_intermediate(mut self, label: impl Into<String>, value: f32) -> Self {
        self.intermediate_values.push((label.into(), value));
        self
    }

    /// 设置最终输出值。
    pub fn with_output(mut self, value: f32) -> Self {
        self.output = value;
        self
    }
}
