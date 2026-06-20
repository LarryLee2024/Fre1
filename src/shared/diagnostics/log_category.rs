//! 日志分类，按业务领域划分五大类。

use super::LogCode;

/// 日志分类，按业务领域划分五大类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogCategory {
    /// 战斗相关：战斗开始/结束、回合流转、伤害结算、击杀、反应触发
    Battle,

    /// 技能/效果相关：技能激活、法术施放、触发器
    Ability,

    /// 效果/标签/修改器相关：效果生命周期、标签变更、属性聚合
    Effect,

    /// 内容/数据相关：任务进度、经验获取、背包变化、交易、声望
    Content,

    /// 基础设施相关：存档、回放、内容加载、系统事件
    Infra,
}

impl std::fmt::Display for LogCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Battle => write!(f, "Battle"),
            Self::Ability => write!(f, "Ability"),
            Self::Effect => write!(f, "Effect"),
            Self::Content => write!(f, "Content"),
            Self::Infra => write!(f, "Infra"),
        }
    }
}

impl LogCode {
    /// 返回该 LogCode 所属的分类。
    pub fn category(&self) -> LogCategory {
        match self {
            // BAT / TAC / TER / RCT → Battle
            Self::BAT001
            | Self::BAT002
            | Self::BAT003
            | Self::BAT004
            | Self::BAT005
            | Self::BAT006
            | Self::BAT007
            | Self::BAT008
            | Self::BAT009
            | Self::BAT010
            | Self::TAC001
            | Self::TAC002
            | Self::TAC003
            | Self::TAC004
            | Self::TAC005
            | Self::TER001
            | Self::TER002
            | Self::TER003
            | Self::TER004
            | Self::RCT001
            | Self::RCT002
            | Self::RCT003
            | Self::RCT004
            | Self::RCT005
            | Self::RCT006
            | Self::RCT007 => LogCategory::Battle,

            // ABL / SPR / TRG → Ability
            Self::ABL001
            | Self::ABL002
            | Self::ABL003
            | Self::ABL004
            | Self::SPR001
            | Self::SPR002
            | Self::SPR003
            | Self::SPR004
            | Self::TRG001
            | Self::TRG002
            | Self::TRG003
            | Self::TRG004 => LogCategory::Ability,

            // EFF / TAG / MOD / AGG → Effect
            Self::EFF001
            | Self::EFF002
            | Self::EFF003
            | Self::EFF004
            | Self::EFF005
            | Self::EFF006
            | Self::EFF007
            | Self::EFF008
            | Self::TAG001
            | Self::TAG002
            | Self::TAG003
            | Self::TAG004
            | Self::MOD001
            | Self::MOD002
            | Self::MOD003
            | Self::MOD004
            | Self::AGG001
            | Self::AGG002
            | Self::AGG003
            | Self::AGG004 => LogCategory::Effect,

            // QST / PRG / INV / ECO / CRF / FAC / PRY / CNR / NAR / SUM → Content
            Self::QST001
            | Self::QST002
            | Self::QST003
            | Self::QST004
            | Self::QST005
            | Self::PRG001
            | Self::PRG002
            | Self::PRG003
            | Self::PRG004
            | Self::PRG005
            | Self::PRG006
            | Self::INV001
            | Self::INV002
            | Self::INV003
            | Self::INV004
            | Self::INV005
            | Self::ECO001
            | Self::ECO002
            | Self::ECO003
            | Self::CRF001
            | Self::CRF002
            | Self::CRF003
            | Self::CRF004
            | Self::CRF005
            | Self::CRF006
            | Self::FAC001
            | Self::FAC002
            | Self::FAC003
            | Self::FAC004
            | Self::PRY001
            | Self::PRY002
            | Self::PRY003
            | Self::PRY004
            | Self::PRY005
            | Self::CNR001
            | Self::CNR002
            | Self::CNR003
            | Self::CNR004
            | Self::CNR005
            | Self::NAR001
            | Self::NAR002
            | Self::NAR003
            | Self::NAR004
            | Self::NAR005
            | Self::SUM001
            | Self::SUM002
            | Self::SUM003
            | Self::SUM004 => LogCategory::Content,

            // CNT / SAV / RPL → Infra
            Self::CNT001
            | Self::CNT002
            | Self::CNT003
            | Self::CNT004
            | Self::CNT005
            | Self::CNT006
            | Self::CNT007
            | Self::CNT008
            | Self::CNT009
            | Self::CNT010
            | Self::CNT011
            | Self::CNT012
            | Self::CNT013
            | Self::CNT014
            | Self::CNT015
            | Self::CNT016
            | Self::CNT017
            | Self::CNT018
            | Self::CNT019
            | Self::CNT020
            | Self::CNT021
            | Self::CNT022
            | Self::CNT023
            | Self::CNT024
            | Self::CNT025
            | Self::CNT026
            | Self::CNT027
            | Self::CNT028
            | Self::SAV001
            | Self::SAV002
            | Self::SAV003
            | Self::RPL001
            | Self::RPL002
            | Self::RPL003 => LogCategory::Infra,
        }
    }
}
