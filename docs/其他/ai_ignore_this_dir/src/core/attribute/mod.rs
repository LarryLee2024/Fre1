//! 属性系统（ADR-031 §1）
//!
//! 基于 Linglan 5+6 属性模型：
//! - 核心五维：PhysAtk / MagicAtk / PhysDef / MagicDef / MaxHp
//! - 次级六维：CritRate / CritDmg / MoveRange / AtkRange / HitRate / DodgeRate
//!
//! 所有数值使用 i32（百分比 = 万分比，如 50% = 5000）。
//! 禁止 f32、禁止硬编码派生公式。

pub mod conversion;
pub mod def;
pub mod ops;

pub use def::*;

use bevy::prelude::*;
use enum_map::{Enum, EnumMap};
use serde::Deserialize;
use std::collections::HashMap;

// ============================================================================
// CoreAttribute — 核心五维
// ============================================================================

/// 核心属性枚举（5 维，Definition → Instance 固定）
///
/// 对应 Linglan 核心五维：物理攻击、魔法攻击、物理防御、魔法防御、最大生命值。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Enum)]
pub enum CoreAttribute {
    PhysAtk,
    MagicAtk,
    PhysDef,
    MagicDef,
    MaxHp,
}

impl CoreAttribute {
    /// 返回核心属性对应的 RON 配置 ID 字符串
    pub fn config_id(&self) -> &'static str {
        match self {
            CoreAttribute::PhysAtk => "phys_atk",
            CoreAttribute::MagicAtk => "magic_atk",
            CoreAttribute::PhysDef => "phys_def",
            CoreAttribute::MagicDef => "magic_def",
            CoreAttribute::MaxHp => "max_hp",
        }
    }

    /// 返回所有核心属性
    pub const fn all() -> [CoreAttribute; 5] {
        [
            CoreAttribute::PhysAtk,
            CoreAttribute::MagicAtk,
            CoreAttribute::PhysDef,
            CoreAttribute::MagicDef,
            CoreAttribute::MaxHp,
        ]
    }
}

// ============================================================================
// SecondaryAttribute — 次级六维
// ============================================================================

/// 次级属性枚举（6 维，运行时由 Modifier 管线计算）
///
/// 对应 Linglan 次级六维：暴击率、暴击伤害、移动范围、攻击范围、命中率、闪避率。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Enum)]
pub enum SecondaryAttribute {
    CritRate,
    CritDmg,
    MoveRange,
    AtkRange,
    HitRate,
    DodgeRate,
}

impl SecondaryAttribute {
    /// 返回次级属性对应的 RON 配置 ID 字符串
    pub fn config_id(&self) -> &'static str {
        match self {
            SecondaryAttribute::CritRate => "crit_rate",
            SecondaryAttribute::CritDmg => "crit_dmg",
            SecondaryAttribute::MoveRange => "move_range",
            SecondaryAttribute::AtkRange => "atk_range",
            SecondaryAttribute::HitRate => "hit_rate",
            SecondaryAttribute::DodgeRate => "dodge_rate",
        }
    }

    /// 返回所有次级属性
    pub const fn all() -> [SecondaryAttribute; 6] {
        [
            SecondaryAttribute::CritRate,
            SecondaryAttribute::CritDmg,
            SecondaryAttribute::MoveRange,
            SecondaryAttribute::AtkRange,
            SecondaryAttribute::HitRate,
            SecondaryAttribute::DodgeRate,
        ]
    }
}

// ============================================================================
// Attributes Component
// ============================================================================

/// 内部修饰符条目
#[derive(Clone, Debug)]
pub(crate) struct StoredModifier {
    pub op: ModifierOp,
    pub value: i32,
    pub source: ModifierSource,
}

/// 运行时属性组件（Instance 层）
///
/// 存储实体的核心属性基础值和次级属性当前值。
/// 次级属性由 Modifier 管线（Step 4）实时计算。
///
/// 临时修饰符存储（`base_values` + `modifiers`）提供向前兼容的
/// get/set 接口，供 Phase 2 下游消费者编译使用。
/// 将在 Step 4（Modifier 重构）中替换为正式的 Modifier 管线。
#[derive(Component, Default, Debug, Clone)]
pub struct Attributes {
    /// 核心属性：基础值（战斗外固定，由 UnitTemplate 设定）
    pub core: EnumMap<CoreAttribute, i32>,
    /// 次级属性：当前值（由 Modifier 管线计算）
    pub secondary: EnumMap<SecondaryAttribute, i32>,
    /// 当前生命值（战斗中变化，不可超过 MaxHp 或低于 0）
    pub current_hp: i32,
    /// 基础属性存储（按 config_id 字符串键，用于非枚举属性访问）
    pub base_values: HashMap<String, i32>,
    /// 活动修饰符存储（config_id → 修饰符列表），临时方案
    pub(crate) modifiers: HashMap<String, Vec<StoredModifier>>,
}

impl Attributes {
    /// 创建新的 Attributes 实例，初值为全 0
    pub fn new() -> Self {
        Self {
            core: EnumMap::default(),
            secondary: EnumMap::default(),
            current_hp: 0,
            base_values: HashMap::new(),
            modifiers: HashMap::new(),
        }
    }

    /// 获取核心属性值
    pub fn core_value(&self, attr: CoreAttribute) -> i32 {
        self.core[attr]
    }

    /// 设置核心属性基础值（超出边界时 clamp）
    pub fn set_core(&mut self, attr: CoreAttribute, value: i32) {
        self.core[attr] = value.max(0); // 核心属性最小值 0
    }

    /// 获取次级属性值
    pub fn secondary_value(&self, attr: SecondaryAttribute) -> i32 {
        self.secondary[attr]
    }

    /// 设置次级属性值（超出边界时 clamp）
    /// 通常由 Modifier 管线调用，不直接使用
    pub fn set_secondary(&mut self, attr: SecondaryAttribute, value: i32) {
        self.secondary[attr] = value;
    }

    /// 获取最大生命值（核心属性 MaxHp）
    pub fn max_hp(&self) -> i32 {
        self.core[CoreAttribute::MaxHp]
    }

    /// 设置最大生命值
    pub fn set_max_hp(&mut self, value: i32) {
        self.core[CoreAttribute::MaxHp] = value.max(1);
        // 确保 current_hp 不超过新的最大值
        self.current_hp = self.current_hp.min(self.max_hp());
    }

    /// 初始化当前生命值为最大值
    pub fn fill_hp(&mut self) {
        self.current_hp = self.max_hp();
    }

    /// 受到伤害（减去护盾 / 减伤后调用）
    /// 返回实际造成的伤害量
    pub fn take_damage(&mut self, amount: i32) -> i32 {
        let actual = amount.min(self.current_hp);
        self.current_hp -= actual;
        actual
    }

    /// 恢复生命值（不超过最大值）
    /// 返回实际恢复量
    pub fn heal(&mut self, amount: i32) -> i32 {
        let missing = self.max_hp().saturating_sub(self.current_hp);
        let actual = amount.min(missing);
        self.current_hp += actual;
        actual
    }

    /// 是否存活
    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }

    /// 生命值百分比（万分比，如 50% = 5000）
    pub fn hp_percent(&self) -> i32 {
        let max = self.max_hp();
        if max <= 0 {
            return 0;
        }
        (self.current_hp as i64 * 10000 / max as i64) as i32
    }

    /// 已损失生命值
    pub fn lost_hp(&self) -> i32 {
        self.max_hp().saturating_sub(self.current_hp)
    }

    // ========================================================================
    // 临时向后兼容接口（Phase 2 编译桥接，Step 4 重构时删除）
    // ========================================================================

    /// 通过 config_id 获取属性的当前值（基础值 + 修饰符求和）
    ///
    /// 优先匹配 CoreAttribute 和 SecondaryAttribute 枚举，然后查 base_values。
    pub fn get(&self, config_id: &str) -> i32 {
        // 先查枚举属性
        let base = match config_id {
            "phys_atk" => self.core_value(CoreAttribute::PhysAtk),
            "magic_atk" => self.core_value(CoreAttribute::MagicAtk),
            "phys_def" => self.core_value(CoreAttribute::PhysDef),
            "magic_def" => self.core_value(CoreAttribute::MagicDef),
            "max_hp" => self.core_value(CoreAttribute::MaxHp),
            "crit_rate" => self.secondary_value(SecondaryAttribute::CritRate),
            "crit_dmg" => self.secondary_value(SecondaryAttribute::CritDmg),
            "move_range" => self.secondary_value(SecondaryAttribute::MoveRange),
            "atk_range" => self.secondary_value(SecondaryAttribute::AtkRange),
            "hit_rate" => self.secondary_value(SecondaryAttribute::HitRate),
            "dodge_rate" => self.secondary_value(SecondaryAttribute::DodgeRate),
            _ => *self.base_values.get(config_id).unwrap_or(&0),
        };
        // 叠加修饰符
        if let Some(mods) = self.modifiers.get(config_id) {
            let mut total = base as i64;
            for m in mods {
                match m.op {
                    ModifierOp::Add => total += m.value as i64,
                    ModifierOp::Multiply => total = total * m.value as i64 / 10000,
                }
            }
            total as i32
        } else {
            base
        }
    }

    /// 通过 config_id 设置基础属性值
    pub fn set_base(&mut self, config_id: &str, value: i32) {
        match config_id {
            "phys_atk" => self.set_core(CoreAttribute::PhysAtk, value),
            "magic_atk" => self.set_core(CoreAttribute::MagicAtk, value),
            "phys_def" => self.set_core(CoreAttribute::PhysDef, value),
            "magic_def" => self.set_core(CoreAttribute::MagicDef, value),
            "max_hp" => self.set_max_hp(value),
            "crit_rate" => self.set_secondary(SecondaryAttribute::CritRate, value),
            "crit_dmg" => self.set_secondary(SecondaryAttribute::CritDmg, value),
            "move_range" => self.set_secondary(SecondaryAttribute::MoveRange, value),
            "atk_range" => self.set_secondary(SecondaryAttribute::AtkRange, value),
            "hit_rate" => self.set_secondary(SecondaryAttribute::HitRate, value),
            "dodge_rate" => self.set_secondary(SecondaryAttribute::DodgeRate, value),
            _ => {
                self.base_values.insert(config_id.to_string(), value);
            }
        }
    }

    /// 添加单个修饰符
    pub fn add_modifier(
        &mut self,
        config_id: String,
        op: ModifierOp,
        value: i32,
        source: ModifierSource,
    ) {
        self.modifiers
            .entry(config_id)
            .or_default()
            .push(StoredModifier { op, value, source });
    }

    /// 从定义列表添加修饰符
    pub fn add_modifiers_from_def(
        &mut self,
        defs: &[AttributeModifierDef],
        source: ModifierSource,
    ) {
        for def in defs {
            self.add_modifier(def.config_id.clone(), def.op, def.value, source);
        }
    }

    /// 移除指定来源的所有修饰符
    pub fn remove_modifiers_from(&mut self, source: ModifierSource) {
        for mods in self.modifiers.values_mut() {
            mods.retain(|m| m.source != source);
        }
    }

    /// 移除所有 Trait 来源的修饰符（Phase 2 桥接）
    pub fn remove_trait_modifiers(&mut self) {
        for mods in self.modifiers.values_mut() {
            mods.retain(|m| !m.source.is_trait());
        }
    }

    /// 设置基础攻击范围（临时桥接，将在 Step 4 删除）
    pub fn set_base_attack_range(&mut self, value: u32) {
        self.set_base("atk_range", value as i32);
    }

    /// 填充生命资源（临时桥接，等效于 fill_hp）
    pub fn fill_vital_resources(&mut self) {
        self.fill_hp();
    }
}

// ============================================================================
// 跨模块类型（供 Buff / Equipment / Modifier 模块引用）
// 这些类型将在 Step 4（Modifier 重构）中迁移到 modifier 模块。
// ============================================================================

/// 修饰符操作类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect, Deserialize)]
pub enum ModifierOp {
    /// 加法：base + value
    Add,
    /// 乘法：base * value / 10000（value 为万分比）
    Multiply,
}

/// 属性修饰符定义（用于 BuffData 等数据定义，支持 RON 反序列化）
#[derive(Clone, Debug, Reflect, Deserialize)]
pub struct AttributeModifierDef {
    /// 配置 ID（如 "phys_atk"），运行时需查找 AttributeRegistry
    pub config_id: String,
    pub op: ModifierOp,
    /// 修饰值（万分比时如 5000 = 50%）
    pub value: i32,
}

/// 修饰符来源：统一标识 Trait / Equipment / Buff
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub struct ModifierSource(pub u64);

impl ModifierSource {
    // ── Trait 区间：u64::MAX ~ u64::MAX - 999 ──
    pub fn trait_source(index: u64) -> Self {
        Self(u64::MAX - index)
    }

    // ── Equipment 区间：u64::MAX - 1000 ~ u64::MAX - 1999 ──
    pub fn equipment_source(index: u64) -> Self {
        Self(u64::MAX - 1000 - index)
    }

    // ── Buff 区间：1 ~ 999999 ──
    pub fn buff_source(id: u64) -> Self {
        Self(id)
    }

    // ── Consumable 区间：u64::MAX - 2001 ~ u64::MAX - 2999 ──
    pub fn consumable_source(entity: Entity) -> Self {
        Self(u64::MAX - 2001 - entity.to_bits())
    }

    pub fn is_trait(&self) -> bool {
        self.0 > u64::MAX - 1000
    }

    pub fn is_equipment(&self) -> bool {
        self.0 > u64::MAX - 2000 && self.0 <= u64::MAX - 1000
    }

    pub fn is_buff(&self) -> bool {
        self.0 < u64::MAX - 2000
    }
}

/// Buff 实例的唯一标识（保留向后兼容，Buff 系统内部使用）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub struct BuffInstanceId(pub u64);

impl BuffInstanceId {
    /// 转换为 ModifierSource（Buff 区间）
    pub fn to_modifier_source(self) -> ModifierSource {
        ModifierSource::buff_source(self.0)
    }
}

/// 属性修饰符实例（运行时，关联到具体来源）
#[derive(Clone, Debug, Reflect)]
pub struct AttributeModifierInstance {
    pub config_id: String,
    pub op: ModifierOp,
    pub value: i32,
    pub source: ModifierSource,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── CoreAttribute ──

    #[test]
    fn core_attribute_config_id() {
        assert_eq!(CoreAttribute::PhysAtk.config_id(), "phys_atk");
        assert_eq!(CoreAttribute::MaxHp.config_id(), "max_hp");
    }

    #[test]
    fn core_attribute_all_count() {
        assert_eq!(CoreAttribute::all().len(), 5);
    }

    // ── SecondaryAttribute ──

    #[test]
    fn secondary_attribute_config_id() {
        assert_eq!(SecondaryAttribute::CritRate.config_id(), "crit_rate");
        assert_eq!(SecondaryAttribute::DodgeRate.config_id(), "dodge_rate");
    }

    #[test]
    fn secondary_attribute_all_count() {
        assert_eq!(SecondaryAttribute::all().len(), 6);
    }

    // ── Attributes Component ──

    fn make_test_attrs() -> Attributes {
        let mut attrs = Attributes::new();
        attrs.set_core(CoreAttribute::PhysAtk, 100);
        attrs.set_core(CoreAttribute::MagicAtk, 80);
        attrs.set_core(CoreAttribute::PhysDef, 60);
        attrs.set_core(CoreAttribute::MagicDef, 50);
        attrs.set_core(CoreAttribute::MaxHp, 500);
        attrs.fill_hp();
        attrs
    }

    #[test]
    fn attributes_new_is_zero() {
        let attrs = Attributes::new();
        assert_eq!(attrs.core_value(CoreAttribute::PhysAtk), 0);
        assert_eq!(attrs.current_hp, 0);
    }

    #[test]
    fn attributes_core_values() {
        let attrs = make_test_attrs();
        assert_eq!(attrs.core_value(CoreAttribute::PhysAtk), 100);
        assert_eq!(attrs.core_value(CoreAttribute::MaxHp), 500);
    }

    #[test]
    fn attributes_fill_hp() {
        let mut attrs = make_test_attrs();
        assert_eq!(attrs.current_hp, 500);
    }

    #[test]
    fn attributes_take_damage() {
        let mut attrs = make_test_attrs();
        let dealt = attrs.take_damage(30);
        assert_eq!(dealt, 30);
        assert_eq!(attrs.current_hp, 470);
    }

    #[test]
    fn attributes_take_damage_does_not_go_below_zero() {
        let mut attrs = make_test_attrs();
        let dealt = attrs.take_damage(9999);
        assert_eq!(dealt, 500);
        assert_eq!(attrs.current_hp, 0);
    }

    #[test]
    fn attributes_heal() {
        let mut attrs = make_test_attrs();
        attrs.take_damage(100);
        let healed = attrs.heal(50);
        assert_eq!(healed, 50);
        assert_eq!(attrs.current_hp, 450);
    }

    #[test]
    fn attributes_heal_does_not_exceed_max() {
        let mut attrs = make_test_attrs();
        attrs.take_damage(50);
        let healed = attrs.heal(200);
        assert_eq!(healed, 50); // only heals missing hp
        assert_eq!(attrs.current_hp, 500);
    }

    #[test]
    fn attributes_is_alive() {
        let mut attrs = make_test_attrs();
        assert!(attrs.is_alive());
        attrs.take_damage(500);
        assert!(!attrs.is_alive());
    }

    #[test]
    fn attributes_hp_percent() {
        let mut attrs = make_test_attrs();
        assert_eq!(attrs.hp_percent(), 10000); // 100%
        attrs.take_damage(250);
        assert_eq!(attrs.hp_percent(), 5000); // 50%
        attrs.take_damage(250);
        assert_eq!(attrs.hp_percent(), 0);
    }

    #[test]
    fn attributes_lost_hp() {
        let mut attrs = make_test_attrs();
        assert_eq!(attrs.lost_hp(), 0);
        attrs.take_damage(100);
        assert_eq!(attrs.lost_hp(), 100);
    }

    #[test]
    fn attributes_set_max_hp_clamps_current_hp() {
        let mut attrs = make_test_attrs();
        attrs.set_max_hp(300);
        assert_eq!(attrs.current_hp, 300); // current clamped
    }

    #[test]
    fn attributes_set_max_hp_minimum_one() {
        let mut attrs = make_test_attrs();
        attrs.set_max_hp(0);
        assert_eq!(attrs.core_value(CoreAttribute::MaxHp), 1);
    }

    #[test]
    fn attributes_core_value_min_zero() {
        let mut attrs = Attributes::new();
        attrs.set_core(CoreAttribute::PhysAtk, -100);
        assert_eq!(attrs.core_value(CoreAttribute::PhysAtk), 0);
    }

    // ── ModifierSource ──

    #[test]
    fn modifier_source_classification() {
        let buff = ModifierSource::buff_source(42);
        assert!(buff.is_buff());
        assert!(!buff.is_trait());
        assert!(!buff.is_equipment());

        let trait_src = ModifierSource::trait_source(0);
        assert!(trait_src.is_trait());

        let equip = ModifierSource::equipment_source(0);
        assert!(equip.is_equipment());
    }

    #[test]
    fn buff_instance_id_conversion() {
        let id = BuffInstanceId(42);
        let source = id.to_modifier_source();
        assert!(source.is_buff());
    }
}
