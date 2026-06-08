// 属性系统：基础值 + 修饰符栈，替代硬编码的 attack_mod/defense_mod

use bevy::prelude::*;
use std::collections::HashMap;

/// 属性类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttributeKind {
    Hp,
    MaxHp,
    Mp,
    MaxMp,
    Atk,
    Def,
    Mov,
    AttackRange,
}

impl AttributeKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Hp => "HP",
            Self::MaxHp => "MaxHP",
            Self::Mp => "MP",
            Self::MaxMp => "MaxMP",
            Self::Atk => "ATK",
            Self::Def => "DEF",
            Self::Mov => "MOV",
            Self::AttackRange => "Range",
        }
    }
}

/// 修饰符操作类型
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ModifierOp {
    /// 加法：base + sum(Add modifiers)
    Add,
    /// 乘法：(base + add_sum) * product(Multiply modifiers)
    Multiply,
}

/// 属性修饰符定义（用于 BuffData 等数据定义）
#[derive(Clone, Debug)]
pub struct AttributeModifierDef {
    pub kind: AttributeKind,
    pub op: ModifierOp,
    pub value: f32,
}

/// Buff 实例的唯一标识
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BuffInstanceId(pub u64);

/// 属性修饰符实例（运行时，关联到具体 BuffInstance）
#[derive(Clone, Debug)]
pub struct AttributeModifierInstance {
    pub kind: AttributeKind,
    pub op: ModifierOp,
    pub value: f32,
    pub source: BuffInstanceId,
}

/// 属性组件：基础值 + 修饰符栈
#[derive(Component, Default, Debug, Clone)]
pub struct Attributes {
    pub base: HashMap<AttributeKind, f32>,
    pub modifiers: Vec<AttributeModifierInstance>,
}

impl Attributes {
    /// 计算最终属性值：base → Add 叠加 → Multiply 叠加
    pub fn get(&self, kind: AttributeKind) -> f32 {
        let base = self.base.get(&kind).copied().unwrap_or(0.0);
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

        let mul = if mul_product == 0.0 {
            1.0
        } else {
            mul_product
        };
        (base + add_sum) * mul
    }

    /// 设置基础属性
    pub fn set_base(&mut self, kind: AttributeKind, value: f32) {
        self.base.insert(kind, value);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 属性_基础值() {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Atk, 10.0);
        assert_eq!(attrs.get(AttributeKind::Atk), 10.0);
    }

    #[test]
    fn 属性_加法修饰符() {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Atk, 10.0);
        attrs.add_modifier(AttributeModifierInstance {
            kind: AttributeKind::Atk,
            op: ModifierOp::Add,
            value: 5.0,
            source: BuffInstanceId(1),
        });
        attrs.add_modifier(AttributeModifierInstance {
            kind: AttributeKind::Atk,
            op: ModifierOp::Add,
            value: -3.0,
            source: BuffInstanceId(2),
        });
        // (10 + 5 - 3) * 1.0 = 12
        assert_eq!(attrs.get(AttributeKind::Atk), 12.0);
    }

    #[test]
    fn 属性_乘法修饰符() {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Atk, 10.0);
        attrs.add_modifier(AttributeModifierInstance {
            kind: AttributeKind::Atk,
            op: ModifierOp::Multiply,
            value: 1.5,
            source: BuffInstanceId(1),
        });
        // (10 + 0) * 1.5 = 15
        assert_eq!(attrs.get(AttributeKind::Atk), 15.0);
    }

    #[test]
    fn 属性_加乘混合() {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Atk, 10.0);
        attrs.add_modifier(AttributeModifierInstance {
            kind: AttributeKind::Atk,
            op: ModifierOp::Add,
            value: 5.0,
            source: BuffInstanceId(1),
        });
        attrs.add_modifier(AttributeModifierInstance {
            kind: AttributeKind::Atk,
            op: ModifierOp::Multiply,
            value: 1.5,
            source: BuffInstanceId(2),
        });
        // (10 + 5) * 1.5 = 22.5
        assert_eq!(attrs.get(AttributeKind::Atk), 22.5);
    }

    #[test]
    fn 属性_移除指定源修饰符() {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Atk, 10.0);
        attrs.add_modifier(AttributeModifierInstance {
            kind: AttributeKind::Atk,
            op: ModifierOp::Add,
            value: 5.0,
            source: BuffInstanceId(1),
        });
        attrs.add_modifier(AttributeModifierInstance {
            kind: AttributeKind::Atk,
            op: ModifierOp::Add,
            value: 3.0,
            source: BuffInstanceId(2),
        });
        attrs.remove_modifiers_from(BuffInstanceId(1));
        // (10 + 3) * 1.0 = 13
        assert_eq!(attrs.get(AttributeKind::Atk), 13.0);
    }

    #[test]
    fn 属性_移除减益修饰符() {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Atk, 10.0);
        attrs.add_modifier(AttributeModifierInstance {
            kind: AttributeKind::Atk,
            op: ModifierOp::Add,
            value: 5.0,
            source: BuffInstanceId(1),
        });
        attrs.add_modifier(AttributeModifierInstance {
            kind: AttributeKind::Atk,
            op: ModifierOp::Add,
            value: -3.0,
            source: BuffInstanceId(2),
        });
        attrs.remove_debuff_modifiers();
        // (10 + 5) * 1.0 = 15
        assert_eq!(attrs.get(AttributeKind::Atk), 15.0);
    }
}
