// 属性类型定义：8维核心属性 + 生命资源 + 衍生属性
// 设计理念：核心属性由种族/职业/等级决定，衍生属性实时计算，生命资源存储当前值

use serde::Deserialize;

/// 属性类型（统一枚举，涵盖核心/资源/衍生三大类）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum AttributeKind {
    // ── 8 维核心属性（玩家可见、可成长、有 base 值）──
    /// 力量：物理攻击力
    Might,
    /// 技巧：命中、暴击、远程
    Dexterity,
    /// 敏捷：行动顺序、闪避、移动
    Agility,
    /// 体质：生命、物防
    Vitality,
    /// 智力：法术攻击、法力
    Intelligence,
    /// 意志：魔防、治疗、异常抵抗
    Willpower,
    /// 魅力：光环、召唤、指挥
    Presence,
    /// 幸运：暴击、掉落、随机事件
    Luck,

    // ── 生命资源（存储当前值，战斗中变化）──
    /// 当前生命值
    Hp,
    /// 当前法力值
    Mp,
    /// 当前耐力值
    Stamina,

    // ── 衍生属性（实时计算，不存储 base 值）──
    /// 最大生命值
    MaxHp,
    /// 最大法力值
    MaxMp,
    /// 最大耐力值
    MaxStamina,
    /// 物理攻击力
    Attack,
    /// 物理防御力
    Defense,
    /// 魔法攻击力
    MagicAttack,
    /// 魔法防御力
    MagicDefense,
    /// 命中率
    Accuracy,
    /// 闪避率
    Evasion,
    /// 暴击率
    CritRate,
    /// 移动力
    MoveRange,
    /// 行动速度
    Initiative,
    /// 攻击范围
    AttackRange,
}

impl AttributeKind {
    /// 是否为核心属性
    pub fn is_core(&self) -> bool {
        matches!(
            self,
            Self::Might
                | Self::Dexterity
                | Self::Agility
                | Self::Vitality
                | Self::Intelligence
                | Self::Willpower
                | Self::Presence
                | Self::Luck
        )
    }

    /// 是否为生命资源（存储当前值）
    pub fn is_vital(&self) -> bool {
        matches!(self, Self::Hp | Self::Mp | Self::Stamina)
    }

    /// 是否为衍生属性（实时计算）
    pub fn is_derived(&self) -> bool {
        matches!(
            self,
            Self::MaxHp
                | Self::MaxMp
                | Self::MaxStamina
                | Self::Attack
                | Self::Defense
                | Self::MagicAttack
                | Self::MagicDefense
                | Self::Accuracy
                | Self::Evasion
                | Self::CritRate
                | Self::MoveRange
                | Self::Initiative
                | Self::AttackRange
        )
    }

    /// 属性中文名
    pub fn label(&self) -> &'static str {
        match self {
            Self::Might => "力量",
            Self::Dexterity => "技巧",
            Self::Agility => "敏捷",
            Self::Vitality => "体质",
            Self::Intelligence => "智力",
            Self::Willpower => "意志",
            Self::Presence => "魅力",
            Self::Luck => "幸运",
            Self::Hp => "HP",
            Self::Mp => "MP",
            Self::Stamina => "耐力",
            Self::MaxHp => "MaxHP",
            Self::MaxMp => "MaxMP",
            Self::MaxStamina => "MaxSTA",
            Self::Attack => "物攻",
            Self::Defense => "物防",
            Self::MagicAttack => "魔攻",
            Self::MagicDefense => "魔防",
            Self::Accuracy => "命中",
            Self::Evasion => "闪避",
            Self::CritRate => "暴击",
            Self::MoveRange => "移动",
            Self::Initiative => "速度",
            Self::AttackRange => "射程",
        }
    }

    /// 核心属性缩写（用于 RON 和 UI）
    pub fn short_label(&self) -> &'static str {
        match self {
            Self::Might => "MIG",
            Self::Dexterity => "DEX",
            Self::Agility => "AGI",
            Self::Vitality => "VIT",
            Self::Intelligence => "INT",
            Self::Willpower => "WIL",
            Self::Presence => "PRE",
            Self::Luck => "LCK",
            _ => self.label(),
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
