//! 可观测域（Domain）——事件路由的分类标识。
//!
//! 每个 Domain 对应一个 LogCode 前缀组，决定 tracing target 的 `domain.xxx` 格式。
//! 这是 LogCode 的**路由关注点**，与 LogCode 本身的**编码关注点**分离。
//!
//! # 设计原则
//!
//! LogCode 只回答 "这是什么事件？"，Domain 只回答 "这个事件路由到哪里？"。
//! 一个事件类型（LogCode）属于且仅属于一个 Domain。
//!
//! ```rust
//! // ObservableEvent trait 通过 const DOMAIN 将二者连接：
//! impl ObservableEvent for LevelUp {
//!     const DOMAIN: Domain = Domain::Progression;
//!     const CODE: LogCode = LogCode::PRG002;
//! }
//! ```
//!
//! # 与 LogCategory 的区别
//!
//! - `Domain` → 路由（target = "domain.progression"），用于 tracing 过滤
//! - `LogCategory` → 聚合（5 大类），用于 metrics 汇总报告

use std::fmt;

/// 可观测域——决定 tracing target 的路由标识。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Domain {
    /// 战斗核心流程（BAT）
    Combat,
    /// 战术移动/网格（TAC）
    Tactical,
    /// 地形效果（TER）
    Terrain,
    /// 技能系统（ABL）
    Ability,
    /// 效果系统（EFF）
    Effect,
    /// 标签系统（TAG）
    Tag,
    /// 修改器系统（MOD）
    Modifier,
    /// 属性聚合（AGG）
    Aggregator,
    /// 触发器（TRG）
    Trigger,
    /// 法术系统（SPR）
    Spell,
    /// 反应/援护（RCT）
    Reaction,
    /// 任务系统（QST）
    Quest,
    /// 成长养成（PRG）
    Progression,
    /// 背包/物品（INV）
    Inventory,
    /// 经济/交易（ECO）
    Economy,
    /// 制作系统（CRF）
    Crafting,
    /// 阵营关系（FAC）
    Faction,
    /// 队伍管理（PRY）
    Party,
    /// 营地休息（CNR）
    CampRest,
    /// 叙事/对话（NAR）
    Narrative,
    /// 召唤系统（SUM）
    Summon,
    /// 内容加载/基础设施（CNT）
    Content,
    /// 存档（SAV）
    Save,
    /// 回放（RPL）
    Replay,
}

impl Domain {
    /// 返回该 Domain 对应的 tracing target 字符串。
    ///
    /// 格式：`domain.{name}`（领域层）或 `infra.{name}`（基础设施层）。
    ///
    /// 这是 tracing 过滤和路由的唯一依据，Observer 的 `#[instrument(target = ...)]`
    /// 应使用此返回值。
    pub const fn target(&self) -> &'static str {
        match self {
            Self::Combat => "domain.combat",
            Self::Tactical => "domain.tactical",
            Self::Terrain => "domain.terrain",
            Self::Ability => "domain.ability",
            Self::Effect => "domain.effect",
            Self::Tag => "domain.tag",
            Self::Modifier => "domain.modifier",
            Self::Aggregator => "domain.aggregator",
            Self::Trigger => "domain.trigger",
            Self::Spell => "domain.spell",
            Self::Reaction => "domain.reaction",
            Self::Quest => "domain.quest",
            Self::Progression => "domain.progression",
            Self::Inventory => "domain.inventory",
            Self::Economy => "domain.economy",
            Self::Crafting => "domain.crafting",
            Self::Faction => "domain.faction",
            Self::Party => "domain.party",
            Self::CampRest => "domain.camp_rest",
            Self::Narrative => "domain.narrative",
            Self::Summon => "domain.summon",
            Self::Content => "content",
            Self::Save => "infra.save",
            Self::Replay => "infra.replay",
        }
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.target())
    }
}
