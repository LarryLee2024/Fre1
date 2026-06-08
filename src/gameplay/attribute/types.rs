// 属性类型定义：枚举、修饰符、Buff 实例标识

use serde::Deserialize;

/// 属性类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "PascalCase")]
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
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ModifierOp {
    /// 加法：base + sum(Add modifiers)
    Add,
    /// 乘法：(base + add_sum) * product(Multiply modifiers)
    Multiply,
}

/// 属性修饰符定义（用于 BuffData 等数据定义，支持 RON 反序列化）
#[derive(Clone, Debug, Deserialize)]
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
