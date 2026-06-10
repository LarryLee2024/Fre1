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

/// 修饰符来源：统一标识 Trait / Equipment / Buff
/// 替代原 BuffInstanceId，解决"装备修饰符不是 Buff"的语义问题
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BuffInstanceId(pub u64);

impl BuffInstanceId {
    /// 转换为 ModifierSource（Buff 区间）
    pub fn to_modifier_source(self) -> ModifierSource {
        ModifierSource::buff_source(self.0)
    }
}

/// 属性修饰符实例（运行时，关联到具体来源）
#[derive(Clone, Debug)]
pub struct AttributeModifierInstance {
    pub kind: AttributeKind,
    pub op: ModifierOp,
    pub value: f32,
    pub source: ModifierSource,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_core ──

    #[test]
    fn 属性分类_核心属性返回true() {
        let core_attrs = [
            AttributeKind::Might,
            AttributeKind::Dexterity,
            AttributeKind::Agility,
            AttributeKind::Vitality,
            AttributeKind::Intelligence,
            AttributeKind::Willpower,
            AttributeKind::Presence,
            AttributeKind::Luck,
        ];
        for attr in &core_attrs {
            assert!(attr.is_core(), "{:?} should be core", attr);
        }
    }

    #[test]
    fn 属性分类_资源属性返回false() {
        assert!(!AttributeKind::Hp.is_core());
        assert!(!AttributeKind::Mp.is_core());
        assert!(!AttributeKind::Stamina.is_core());
    }

    #[test]
    fn 属性分类_衍生属性返回false() {
        let derived_attrs = [
            AttributeKind::MaxHp,
            AttributeKind::MaxMp,
            AttributeKind::MaxStamina,
            AttributeKind::Attack,
            AttributeKind::Defense,
            AttributeKind::MagicAttack,
            AttributeKind::MagicDefense,
            AttributeKind::Accuracy,
            AttributeKind::Evasion,
            AttributeKind::CritRate,
            AttributeKind::MoveRange,
            AttributeKind::Initiative,
            AttributeKind::AttackRange,
        ];
        for attr in &derived_attrs {
            assert!(!attr.is_core(), "{:?} should not be core", attr);
        }
    }

    // ── is_vital ──

    #[test]
    fn 属性分类_资源属性返回true() {
        let vital_attrs = [AttributeKind::Hp, AttributeKind::Mp, AttributeKind::Stamina];
        for attr in &vital_attrs {
            assert!(attr.is_vital(), "{:?} should be vital", attr);
        }
    }

    #[test]
    fn 属性分类_非资源属性返回false() {
        assert!(!AttributeKind::Might.is_vital());
        assert!(!AttributeKind::Attack.is_vital());
        assert!(!AttributeKind::MaxHp.is_vital());
    }

    // ── is_derived ──

    #[test]
    fn 属性分类_衍生属性返回true() {
        let derived_attrs = [
            AttributeKind::MaxHp,
            AttributeKind::MaxMp,
            AttributeKind::MaxStamina,
            AttributeKind::Attack,
            AttributeKind::Defense,
            AttributeKind::MagicAttack,
            AttributeKind::MagicDefense,
            AttributeKind::Accuracy,
            AttributeKind::Evasion,
            AttributeKind::CritRate,
            AttributeKind::MoveRange,
            AttributeKind::Initiative,
            AttributeKind::AttackRange,
        ];
        for attr in &derived_attrs {
            assert!(attr.is_derived(), "{:?} should be derived", attr);
        }
    }

    #[test]
    fn 属性分类_非衍生属性返回false() {
        assert!(!AttributeKind::Might.is_derived());
        assert!(!AttributeKind::Hp.is_derived());
    }

    // ── 互斥性 ──

    #[test]
    fn 属性分类_三类互斥() {
        let all_attrs = [
            AttributeKind::Might,
            AttributeKind::Dexterity,
            AttributeKind::Agility,
            AttributeKind::Vitality,
            AttributeKind::Intelligence,
            AttributeKind::Willpower,
            AttributeKind::Presence,
            AttributeKind::Luck,
            AttributeKind::Hp,
            AttributeKind::Mp,
            AttributeKind::Stamina,
            AttributeKind::MaxHp,
            AttributeKind::MaxMp,
            AttributeKind::MaxStamina,
            AttributeKind::Attack,
            AttributeKind::Defense,
            AttributeKind::MagicAttack,
            AttributeKind::MagicDefense,
            AttributeKind::Accuracy,
            AttributeKind::Evasion,
            AttributeKind::CritRate,
            AttributeKind::MoveRange,
            AttributeKind::Initiative,
            AttributeKind::AttackRange,
        ];
        for attr in &all_attrs {
            let categories = attr.is_core() as u8 + attr.is_vital() as u8 + attr.is_derived() as u8;
            assert_eq!(
                categories, 1,
                "{:?} should belong to exactly one category",
                attr
            );
        }
    }

    // ── label ──

    #[test]
    fn 属性中文名_核心属性() {
        assert_eq!(AttributeKind::Might.label(), "力量");
        assert_eq!(AttributeKind::Dexterity.label(), "技巧");
        assert_eq!(AttributeKind::Agility.label(), "敏捷");
        assert_eq!(AttributeKind::Vitality.label(), "体质");
        assert_eq!(AttributeKind::Intelligence.label(), "智力");
        assert_eq!(AttributeKind::Willpower.label(), "意志");
        assert_eq!(AttributeKind::Presence.label(), "魅力");
        assert_eq!(AttributeKind::Luck.label(), "幸运");
    }

    #[test]
    fn 属性中文名_资源属性() {
        assert_eq!(AttributeKind::Hp.label(), "HP");
        assert_eq!(AttributeKind::Mp.label(), "MP");
        assert_eq!(AttributeKind::Stamina.label(), "耐力");
    }

    #[test]
    fn 属性中文名_衍生属性() {
        assert_eq!(AttributeKind::MaxHp.label(), "MaxHP");
        assert_eq!(AttributeKind::MaxMp.label(), "MaxMP");
        assert_eq!(AttributeKind::MaxStamina.label(), "MaxSTA");
        assert_eq!(AttributeKind::Attack.label(), "物攻");
        assert_eq!(AttributeKind::Defense.label(), "物防");
        assert_eq!(AttributeKind::MagicAttack.label(), "魔攻");
        assert_eq!(AttributeKind::MagicDefense.label(), "魔防");
        assert_eq!(AttributeKind::Accuracy.label(), "命中");
        assert_eq!(AttributeKind::Evasion.label(), "闪避");
        assert_eq!(AttributeKind::CritRate.label(), "暴击");
        assert_eq!(AttributeKind::MoveRange.label(), "移动");
        assert_eq!(AttributeKind::Initiative.label(), "速度");
        assert_eq!(AttributeKind::AttackRange.label(), "射程");
    }

    // ── short_label ──

    #[test]
    fn 属性缩写_核心属性返回三字母缩写() {
        assert_eq!(AttributeKind::Might.short_label(), "MIG");
        assert_eq!(AttributeKind::Dexterity.short_label(), "DEX");
        assert_eq!(AttributeKind::Agility.short_label(), "AGI");
        assert_eq!(AttributeKind::Vitality.short_label(), "VIT");
        assert_eq!(AttributeKind::Intelligence.short_label(), "INT");
        assert_eq!(AttributeKind::Willpower.short_label(), "WIL");
        assert_eq!(AttributeKind::Presence.short_label(), "PRE");
        assert_eq!(AttributeKind::Luck.short_label(), "LCK");
    }

    #[test]
    fn 属性缩写_非核心属性回退到label() {
        assert_eq!(AttributeKind::Hp.short_label(), "HP");
        assert_eq!(AttributeKind::Attack.short_label(), "物攻");
        assert_eq!(AttributeKind::MoveRange.short_label(), "移动");
    }
}
