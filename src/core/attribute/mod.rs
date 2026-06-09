// 属性系统：8维核心属性 + 衍生属性实时计算 + 修饰符栈
// 核心属性由种族/职业/等级决定，衍生属性从核心属性公式计算，生命资源存储当前值

mod types;

pub use types::*;

use bevy::prelude::*;
use std::collections::HashMap;

/// 属性组件：核心属性基础值 + 生命资源当前值 + 修饰符栈
#[derive(Component, Default, Debug, Clone)]
pub struct Attributes {
    /// 核心属性基础值（8维：Might/Dexterity/Agility/Vitality/Intelligence/Willpower/Presence/Luck）
    pub base: HashMap<AttributeKind, f32>,
    /// 当前 HP（战斗中变化）
    pub current_hp: f32,
    /// 当前 MP（战斗中变化）
    pub current_mp: f32,
    /// 当前 Stamina（战斗中变化）
    pub current_stamina: f32,
    /// 基础攻击范围（由职业/装备决定，不随属性变化）
    pub base_attack_range: u32,
    /// 修饰符栈
    pub modifiers: Vec<AttributeModifierInstance>,
}

impl Attributes {
    // ── 核心属性访问 ──

    /// 获取核心属性原始基础值（不含修饰符）
    pub fn core_base(&self, kind: AttributeKind) -> f32 {
        self.base.get(&kind).copied().unwrap_or(0.0)
    }

    /// 获取核心属性最终值（base + 修饰符）
    pub fn core(&self, kind: AttributeKind) -> f32 {
        let base = self.core_base(kind);
        let add_sum: f32 = self
            .modifiers
            .iter()
            .filter(|m| m.kind == kind && m.op == ModifierOp::Add)
            .map(|m| m.value)
            .sum();
        let mul_product: f32 = self
            .modifiers
            .iter()
            .filter(|m| m.kind == kind && m.op == ModifierOp::Multiply)
            .map(|m| m.value)
            .product::<f32>();
        let mul = if mul_product == 0.0 { 1.0 } else { mul_product };
        (base + add_sum) * mul
    }

    // ── 衍生属性计算（从核心属性实时推导）──

    /// MaxHp = 5 + Vitality * 5
    fn calc_max_hp(&self) -> f32 {
        let vit = self.core(AttributeKind::Vitality);
        let base = 5.0 + vit * 5.0;
        self.apply_modifiers(AttributeKind::MaxHp, base)
    }

    /// MaxMp = Intelligence * 5
    fn calc_max_mp(&self) -> f32 {
        let int = self.core(AttributeKind::Intelligence);
        let base = int * 5.0;
        self.apply_modifiers(AttributeKind::MaxMp, base)
    }

    /// MaxStamina = 10 + (Vitality + Might) * 2
    fn calc_max_stamina(&self) -> f32 {
        let vit = self.core(AttributeKind::Vitality);
        let mig = self.core(AttributeKind::Might);
        let base = 10.0 + (vit + mig) * 2.0;
        self.apply_modifiers(AttributeKind::MaxStamina, base)
    }

    /// Attack = Might * 2
    fn calc_attack(&self) -> f32 {
        let mig = self.core(AttributeKind::Might);
        let base = mig * 2.0;
        self.apply_modifiers(AttributeKind::Attack, base)
    }

    /// Defense = Vitality
    fn calc_defense(&self) -> f32 {
        let vit = self.core(AttributeKind::Vitality);
        let base = vit;
        self.apply_modifiers(AttributeKind::Defense, base)
    }

    /// MagicAttack = Intelligence * 2
    fn calc_magic_attack(&self) -> f32 {
        let int = self.core(AttributeKind::Intelligence);
        let base = int * 2.0;
        self.apply_modifiers(AttributeKind::MagicAttack, base)
    }

    /// MagicDefense = Willpower
    fn calc_magic_defense(&self) -> f32 {
        let wil = self.core(AttributeKind::Willpower);
        let base = wil;
        self.apply_modifiers(AttributeKind::MagicDefense, base)
    }

    /// Accuracy = 80 + Dexterity * 2
    fn calc_accuracy(&self) -> f32 {
        let dex = self.core(AttributeKind::Dexterity);
        let base = 80.0 + dex * 2.0;
        self.apply_modifiers(AttributeKind::Accuracy, base)
    }

    /// Evasion = Agility * 3
    fn calc_evasion(&self) -> f32 {
        let agi = self.core(AttributeKind::Agility);
        let base = agi * 3.0;
        self.apply_modifiers(AttributeKind::Evasion, base)
    }

    /// CritRate = 5 + Luck
    fn calc_crit_rate(&self) -> f32 {
        let lck = self.core(AttributeKind::Luck);
        let base = 5.0 + lck;
        self.apply_modifiers(AttributeKind::CritRate, base)
    }

    /// MoveRange = floor(Agility * 0.5) + 2
    fn calc_move_range(&self) -> f32 {
        let agi = self.core(AttributeKind::Agility);
        let base = (agi * 0.5).floor() + 2.0;
        self.apply_modifiers(AttributeKind::MoveRange, base)
    }

    /// Initiative = Agility * 2 + Luck
    fn calc_initiative(&self) -> f32 {
        let agi = self.core(AttributeKind::Agility);
        let lck = self.core(AttributeKind::Luck);
        let base = agi * 2.0 + lck;
        self.apply_modifiers(AttributeKind::Initiative, base)
    }

    /// AttackRange = base_attack_range（由职业/装备决定）
    fn calc_attack_range(&self) -> f32 {
        let base = self.base_attack_range as f32;
        self.apply_modifiers(AttributeKind::AttackRange, base)
    }

    /// 对衍生属性基础值应用修饰符
    fn apply_modifiers(&self, kind: AttributeKind, base: f32) -> f32 {
        let add_sum: f32 = self
            .modifiers
            .iter()
            .filter(|m| m.kind == kind && m.op == ModifierOp::Add)
            .map(|m| m.value)
            .sum();
        let mul_product: f32 = self
            .modifiers
            .iter()
            .filter(|m| m.kind == kind && m.op == ModifierOp::Multiply)
            .map(|m| m.value)
            .product::<f32>();
        let mul = if mul_product == 0.0 { 1.0 } else { mul_product };
        (base + add_sum) * mul
    }

    // ── 统一属性访问接口 ──

    /// 获取任意属性值（核心/资源/衍生统一接口）
    pub fn get(&self, kind: AttributeKind) -> f32 {
        match kind {
            // 核心属性
            AttributeKind::Might
            | AttributeKind::Dexterity
            | AttributeKind::Agility
            | AttributeKind::Vitality
            | AttributeKind::Intelligence
            | AttributeKind::Willpower
            | AttributeKind::Presence
            | AttributeKind::Luck => self.core(kind),

            // 生命资源
            AttributeKind::Hp => self.current_hp,
            AttributeKind::Mp => self.current_mp,
            AttributeKind::Stamina => self.current_stamina,

            // 衍生属性
            AttributeKind::MaxHp => self.calc_max_hp(),
            AttributeKind::MaxMp => self.calc_max_mp(),
            AttributeKind::MaxStamina => self.calc_max_stamina(),
            AttributeKind::Attack => self.calc_attack(),
            AttributeKind::Defense => self.calc_defense(),
            AttributeKind::MagicAttack => self.calc_magic_attack(),
            AttributeKind::MagicDefense => self.calc_magic_defense(),
            AttributeKind::Accuracy => self.calc_accuracy(),
            AttributeKind::Evasion => self.calc_evasion(),
            AttributeKind::CritRate => self.calc_crit_rate(),
            AttributeKind::MoveRange => self.calc_move_range(),
            AttributeKind::Initiative => self.calc_initiative(),
            AttributeKind::AttackRange => self.calc_attack_range(),
        }
    }

    /// 设置基础值（仅核心属性和生命资源有效）
    pub fn set_base(&mut self, kind: AttributeKind, value: f32) {
        match kind {
            AttributeKind::Hp => self.current_hp = value,
            AttributeKind::Mp => self.current_mp = value,
            AttributeKind::Stamina => self.current_stamina = value,
            _ if kind.is_core() => {
                self.base.insert(kind, value);
            }
            _ => {
                bevy::log::warn!("不能设置衍生属性的基础值: {:?}", kind);
            }
        }
    }

    /// 设置基础攻击范围（由职业/装备决定）
    pub fn set_base_attack_range(&mut self, range: u32) {
        self.base_attack_range = range;
    }

    /// 添加修饰符
    pub fn add_modifier(&mut self, modifier: AttributeModifierInstance) {
        self.modifiers.push(modifier);
    }

    /// 移除来自指定 Buff 的所有修饰符
    pub fn remove_modifiers_from(&mut self, source: BuffInstanceId) {
        self.modifiers.retain(|m| m.source != source);
    }

    /// 移除所有减益修饰符（值为负的 Add 修饰符 + 值 < 1.0 的 Multiply 修饰符）
    pub fn remove_debuff_modifiers(&mut self) {
        self.modifiers.retain(|m| match m.op {
            ModifierOp::Add => m.value >= 0.0,
            ModifierOp::Multiply => m.value >= 1.0,
        });
    }

    /// 从定义列表和 BuffInstanceId 批量添加修饰符
    pub fn add_modifiers_from_def(
        &mut self,
        defs: &[AttributeModifierDef],
        source: BuffInstanceId,
    ) {
        for def in defs {
            self.add_modifier(AttributeModifierInstance {
                kind: def.kind,
                op: def.op,
                value: def.value,
                source,
            });
        }
    }

    /// 初始化生命资源为最大值（生成单位时调用）
    pub fn fill_vital_resources(&mut self) {
        self.current_hp = self.calc_max_hp();
        self.current_mp = self.calc_max_mp();
        self.current_stamina = self.calc_max_stamina();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_warrior_attrs() -> Attributes {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Might, 5.0);
        attrs.set_base(AttributeKind::Dexterity, 3.0);
        attrs.set_base(AttributeKind::Agility, 6.0);
        attrs.set_base(AttributeKind::Vitality, 5.0);
        attrs.set_base(AttributeKind::Intelligence, 2.0);
        attrs.set_base(AttributeKind::Willpower, 3.0);
        attrs.set_base(AttributeKind::Presence, 2.0);
        attrs.set_base(AttributeKind::Luck, 2.0);
        attrs.set_base_attack_range(1);
        attrs.fill_vital_resources();
        attrs
    }

    #[test]
    fn 核心属性_基础值() {
        let attrs = make_warrior_attrs();
        assert_eq!(attrs.core(AttributeKind::Might), 5.0);
        assert_eq!(attrs.core(AttributeKind::Vitality), 5.0);
    }

    #[test]
    fn 衍生属性_战士模板() {
        let attrs = make_warrior_attrs();
        // MaxHp = 5 + 5*5 = 30
        assert_eq!(attrs.get(AttributeKind::MaxHp), 30.0);
        // Attack = 5*2 = 10
        assert_eq!(attrs.get(AttributeKind::Attack), 10.0);
        // Defense = 5
        assert_eq!(attrs.get(AttributeKind::Defense), 5.0);
        // MoveRange = floor(6*0.5) + 2 = 3 + 2 = 5
        assert_eq!(attrs.get(AttributeKind::MoveRange), 5.0);
        // AttackRange = 1
        assert_eq!(attrs.get(AttributeKind::AttackRange), 1.0);
        // MagicAttack = 2*2 = 4
        assert_eq!(attrs.get(AttributeKind::MagicAttack), 4.0);
        // MagicDefense = 3
        assert_eq!(attrs.get(AttributeKind::MagicDefense), 3.0);
    }

    #[test]
    fn 衍生属性_弓手模板() {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Might, 4.0);
        attrs.set_base(AttributeKind::Dexterity, 6.0);
        attrs.set_base(AttributeKind::Agility, 6.0);
        attrs.set_base(AttributeKind::Vitality, 3.0);
        attrs.set_base(AttributeKind::Intelligence, 3.0);
        attrs.set_base(AttributeKind::Willpower, 2.0);
        attrs.set_base(AttributeKind::Presence, 2.0);
        attrs.set_base(AttributeKind::Luck, 3.0);
        attrs.set_base_attack_range(3);
        attrs.fill_vital_resources();

        // MaxHp = 5 + 3*5 = 20
        assert_eq!(attrs.get(AttributeKind::MaxHp), 20.0);
        // Attack = 4*2 = 8
        assert_eq!(attrs.get(AttributeKind::Attack), 8.0);
        // Defense = 3
        assert_eq!(attrs.get(AttributeKind::Defense), 3.0);
        // MoveRange = floor(6*0.5) + 2 = 5
        assert_eq!(attrs.get(AttributeKind::MoveRange), 5.0);
        // AttackRange = 3
        assert_eq!(attrs.get(AttributeKind::AttackRange), 3.0);
    }

    #[test]
    fn 生命资源_初始化为最大值() {
        let attrs = make_warrior_attrs();
        assert_eq!(attrs.get(AttributeKind::Hp), 30.0);
        assert_eq!(attrs.get(AttributeKind::MaxHp), 30.0);
    }

    #[test]
    fn 生命资源_战斗中变化() {
        let mut attrs = make_warrior_attrs();
        attrs.set_base(AttributeKind::Hp, 20.0);
        assert_eq!(attrs.get(AttributeKind::Hp), 20.0);
        assert_eq!(attrs.get(AttributeKind::MaxHp), 30.0); // MaxHp 不变
    }

    #[test]
    fn 加法修饰符_核心属性() {
        let mut attrs = make_warrior_attrs();
        attrs.add_modifier(AttributeModifierInstance {
            kind: AttributeKind::Might,
            op: ModifierOp::Add,
            value: 3.0,
            source: BuffInstanceId(1),
        });
        // Might: 5 + 3 = 8
        assert_eq!(attrs.core(AttributeKind::Might), 8.0);
        // Attack 衍生: 8*2 = 16
        assert_eq!(attrs.get(AttributeKind::Attack), 16.0);
    }

    #[test]
    fn 加法修饰符_衍生属性() {
        let mut attrs = make_warrior_attrs();
        attrs.add_modifier(AttributeModifierInstance {
            kind: AttributeKind::Attack,
            op: ModifierOp::Add,
            value: 5.0,
            source: BuffInstanceId(1),
        });
        // Attack: (5*2) + 5 = 15
        assert_eq!(attrs.get(AttributeKind::Attack), 15.0);
    }

    #[test]
    fn 乘法修饰符_衍生属性() {
        let mut attrs = make_warrior_attrs();
        attrs.add_modifier(AttributeModifierInstance {
            kind: AttributeKind::Attack,
            op: ModifierOp::Multiply,
            value: 1.5,
            source: BuffInstanceId(1),
        });
        // Attack: 10 * 1.5 = 15
        assert_eq!(attrs.get(AttributeKind::Attack), 15.0);
    }

    #[test]
    fn 移除指定源修饰符() {
        let mut attrs = make_warrior_attrs();
        attrs.add_modifier(AttributeModifierInstance {
            kind: AttributeKind::Attack,
            op: ModifierOp::Add,
            value: 5.0,
            source: BuffInstanceId(1),
        });
        attrs.add_modifier(AttributeModifierInstance {
            kind: AttributeKind::Attack,
            op: ModifierOp::Add,
            value: 3.0,
            source: BuffInstanceId(2),
        });
        attrs.remove_modifiers_from(BuffInstanceId(1));
        // Attack: 10 + 3 = 13
        assert_eq!(attrs.get(AttributeKind::Attack), 13.0);
    }

    #[test]
    fn 移除减益修饰符() {
        let mut attrs = make_warrior_attrs();
        attrs.add_modifier(AttributeModifierInstance {
            kind: AttributeKind::Attack,
            op: ModifierOp::Add,
            value: 5.0,
            source: BuffInstanceId(1),
        });
        attrs.add_modifier(AttributeModifierInstance {
            kind: AttributeKind::Attack,
            op: ModifierOp::Add,
            value: -3.0,
            source: BuffInstanceId(2),
        });
        attrs.remove_debuff_modifiers();
        // Attack: 10 + 5 = 15
        assert_eq!(attrs.get(AttributeKind::Attack), 15.0);
    }

    #[test]
    fn 不能设置衍生属性基础值() {
        let mut attrs = make_warrior_attrs();
        let before = attrs.get(AttributeKind::Attack);
        attrs.set_base(AttributeKind::Attack, 999.0);
        assert_eq!(attrs.get(AttributeKind::Attack), before);
    }

    #[test]
    fn add_modifiers_from_def_批量添加() {
        let mut attrs = make_warrior_attrs();
        let defs = vec![
            AttributeModifierDef {
                kind: AttributeKind::Attack,
                op: ModifierOp::Add,
                value: 5.0,
            },
            AttributeModifierDef {
                kind: AttributeKind::Defense,
                op: ModifierOp::Add,
                value: -2.0,
            },
        ];
        attrs.add_modifiers_from_def(&defs, BuffInstanceId(1));
        // Attack: 10 + 5 = 15
        assert_eq!(attrs.get(AttributeKind::Attack), 15.0);
        // Defense: 5 - 2 = 3
        assert_eq!(attrs.get(AttributeKind::Defense), 3.0);
    }

    #[test]
    fn 哥布林模板() {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Might, 4.0);
        attrs.set_base(AttributeKind::Dexterity, 2.0);
        attrs.set_base(AttributeKind::Agility, 4.0);
        attrs.set_base(AttributeKind::Vitality, 3.0);
        attrs.set_base(AttributeKind::Intelligence, 1.0);
        attrs.set_base(AttributeKind::Willpower, 2.0);
        attrs.set_base(AttributeKind::Presence, 1.0);
        attrs.set_base(AttributeKind::Luck, 2.0);
        attrs.set_base_attack_range(1);
        attrs.fill_vital_resources();

        // MaxHp = 5 + 3*5 = 20
        assert_eq!(attrs.get(AttributeKind::MaxHp), 20.0);
        // Attack = 4*2 = 8
        assert_eq!(attrs.get(AttributeKind::Attack), 8.0);
        // Defense = 3
        assert_eq!(attrs.get(AttributeKind::Defense), 3.0);
        // MoveRange = floor(4*0.5) + 2 = 2 + 2 = 4
        assert_eq!(attrs.get(AttributeKind::MoveRange), 4.0);
    }

    #[test]
    fn 法师模板() {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Might, 2.0);
        attrs.set_base(AttributeKind::Dexterity, 3.0);
        attrs.set_base(AttributeKind::Agility, 6.0);
        attrs.set_base(AttributeKind::Vitality, 3.0);
        attrs.set_base(AttributeKind::Intelligence, 5.0);
        attrs.set_base(AttributeKind::Willpower, 4.0);
        attrs.set_base(AttributeKind::Presence, 3.0);
        attrs.set_base(AttributeKind::Luck, 2.0);
        attrs.set_base_attack_range(2);
        attrs.fill_vital_resources();

        // MaxHp = 5 + 3*5 = 20
        assert_eq!(attrs.get(AttributeKind::MaxHp), 20.0);
        // MagicAttack = 5*2 = 10
        assert_eq!(attrs.get(AttributeKind::MagicAttack), 10.0);
        // MagicDefense = 4
        assert_eq!(attrs.get(AttributeKind::MagicDefense), 4.0);
        // MoveRange = floor(6*0.5) + 2 = 5
        assert_eq!(attrs.get(AttributeKind::MoveRange), 5.0);
        // MaxMp = 5*5 = 25
        assert_eq!(attrs.get(AttributeKind::MaxMp), 25.0);
    }
}
