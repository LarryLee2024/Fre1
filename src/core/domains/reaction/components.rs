//! ECS Components — 反应领域组件与类型
//!
//! 定义反应相关的 ECS 组件、值类型和瞬时结构。
//! 详见 docs/02-domain/domains/reaction_domain.md
//! 详见 docs/04-data/domains/reaction_schema.md

use bevy::prelude::*;

// ─── 值类型 ────────────────────────────────────────────────────────

/// 反应类型枚举。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub enum ReactionType {
    /// 机会攻击（借机攻击）。
    OpportunityAttack,
    /// 法术反制。
    Counterspell,
    /// 护盾术。
    Shield,
    /// 援护格挡。
    Guardian,
    /// 特殊反应（由 Domain 注册的自定义反应）。
    Special { custom_id: String },
}

impl ReactionType {
    /// 获取反应类型的中文描述。
    pub fn display_name(&self) -> &str {
        match self {
            ReactionType::OpportunityAttack => "机会攻击",
            ReactionType::Counterspell => "法术反制",
            ReactionType::Shield => "护盾术",
            ReactionType::Guardian => "援护格挡",
            ReactionType::Special { .. } => "特殊反应",
        }
    }
}

/// 反应条目在队列中的状态。
#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub enum ReactionEntryStatus {
    /// 等待玩家/AI 决策。
    Pending,
    /// 已接受（选择使用反应）。
    Accepted,
    /// 已拒绝（放弃使用反应）。
    Declined,
    /// 已执行完毕。
    Executed,
    /// 因前置反应的影响而取消。
    Cancelled,
}

/// 反应触发事件上下文。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum ReactionTrigger {
    /// 单位离开威胁区。
    LeaveThreatRange {
        /// 移动的单位。
        mover: Entity,
        /// 目标位置（网格坐标 x, y）。
        to_x: i32,
        to_y: i32,
    },
    /// 敌方施法。
    EnemySpellCast {
        /// 施法者。
        caster: Entity,
        /// 法术 ID。
        spell_id: String,
    },
    /// 被攻击命中前。
    BeforeHit {
        /// 攻击者。
        attacker: Entity,
        /// 目标。
        target: Entity,
        /// 攻击检定结果。
        attack_roll: i32,
    },
    /// 相邻友方被攻击。
    AdjacentAllyHit {
        /// 被攻击的友方。
        ally: Entity,
        /// 攻击者。
        attacker: Entity,
    },
}

// ─── 反应队列条目 ────────────────────────────────────────────────

/// 反应队列中的单一条目。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct ReactionEntry {
    /// 触发者（将执行反应的单位）。
    pub reactor: Entity,
    /// 反应类型。
    pub reaction_type: ReactionType,
    /// 触发事件上下文。
    pub trigger: ReactionTrigger,
    /// 优先级（数值越大越优先，防御型 > 进攻型）。
    pub priority: u32,
    /// 条目当前状态。
    pub status: ReactionEntryStatus,
}

// ─── Instance 层组件 ─────────────────────────────────────────────

/// 反应槽位状态组件。
///
/// 标记单位在当前回合是否已使用反应，以及是否拥有额外反应次数。
/// 每回合开始时重置。
/// 详见 docs/04-data/domains/reaction_schema.md §1.1
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct ReactionState {
    /// 当前回合是否已使用基本反应。
    pub used: bool,
    /// 额外反应次数（特殊能力/专长提供，默认 0）。
    pub extra_reactions: u32,
    /// 本回合已使用的额外反应次数。
    pub extra_used: u32,
}

impl ReactionState {
    /// 创建初始反应状态（可用，无额外反应）。
    pub fn new() -> Self {
        Self {
            used: false,
            extra_reactions: 0,
            extra_used: 0,
        }
    }

    /// 当前是否可以使用反应。
    pub fn can_react(&self) -> bool {
        !self.used || self.extra_used < self.extra_reactions
    }

    /// 消耗一次反应机会。
    ///
    /// 如果基本反应未使用则消耗基本；否则消耗额外。
    /// 返回是否消耗成功。
    pub fn consume(&mut self) -> bool {
        if !self.used {
            self.used = true;
            true
        } else if self.extra_used < self.extra_reactions {
            self.extra_used += 1;
            true
        } else {
            false
        }
    }

    /// 重置反应状态（新回合开始时调用）。
    pub fn reset(&mut self) {
        self.used = false;
        self.extra_used = 0;
    }
}

impl Default for ReactionState {
    fn default() -> Self {
        Self::new()
    }
}

// ─── 瞬时结构（非 Component，一帧内创建销毁） ─────────────────

/// 反应队列。
///
/// 当多个反应同时触发时，按优先级排队执行。
/// 瞬时结构，在一帧内创建、消费、销毁。
#[derive(Debug, Clone, Reflect)]
pub struct ReactionQueue {
    /// 排队中的反应条目。
    pub entries: Vec<ReactionEntry>,
    /// 当前正在处理的条目标引。
    pub current_index: usize,
}

impl ReactionQueue {
    /// 创建空反应队列。
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            current_index: 0,
        }
    }

    /// 添加反应条目到队列（按优先级排序插入）。
    pub fn enqueue(&mut self, entry: ReactionEntry) {
        let pos = self
            .entries
            .binary_search_by(|e| e.priority.cmp(&entry.priority).reverse())
            .unwrap_or_else(|e| e);
        self.entries.insert(pos, entry);
    }

    /// 获取下一个待处理的反应条目（从 current_index 开始向后查找）。
    pub fn next_pending(&self) -> Option<&ReactionEntry> {
        self.entries
            .iter()
            .skip(self.current_index)
            .find(|e| e.status == ReactionEntryStatus::Pending)
    }

    /// 标记当前条目为已接受。
    pub fn accept_current(&mut self) -> bool {
        if let Some(entry) = self.entries.get_mut(self.current_index) {
            if entry.status == ReactionEntryStatus::Pending {
                entry.status = ReactionEntryStatus::Accepted;
                return true;
            }
        }
        false
    }

    /// 标记当前条目为已拒绝。
    pub fn decline_current(&mut self) -> bool {
        if let Some(entry) = self.entries.get_mut(self.current_index) {
            if entry.status == ReactionEntryStatus::Pending {
                entry.status = ReactionEntryStatus::Declined;
                self.current_index += 1;
                return true;
            }
        }
        false
    }

    /// 标记当前条目为已执行。
    pub fn mark_executed(&mut self) -> bool {
        if let Some(entry) = self.entries.get_mut(self.current_index) {
            if entry.status == ReactionEntryStatus::Accepted {
                entry.status = ReactionEntryStatus::Executed;
                self.current_index += 1;
                return true;
            }
        }
        false
    }

    /// 标记当前条目为已取消。
    pub fn cancel_current(&mut self) -> bool {
        if let Some(entry) = self.entries.get_mut(self.current_index) {
            entry.status = ReactionEntryStatus::Cancelled;
            self.current_index += 1;
            return true;
        }
        false
    }

    /// 是否所有条目都已处理完毕。
    pub fn is_finished(&self) -> bool {
        self.current_index >= self.entries.len()
    }
}

impl Default for ReactionQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// 机会攻击的触发与执行数据。
///
/// 瞬时结构，在一帧内创建、消费、销毁。
#[derive(Debug, Clone, Reflect)]
pub struct OpportunityAttackData {
    /// 攻击者（威胁单位）。
    pub attacker: Entity,
    /// 目标（离开威胁区的单位）。
    pub target: Entity,
    /// 触发位置的 x 坐标。
    pub from_x: i32,
    /// 触发位置的 y 坐标。
    pub from_y: i32,
    /// 攻击结果（由 Combat 领域填充）。
    pub result: Option<AttackResult>,
}

/// 攻击结果。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum AttackResult {
    /// 命中。
    Hit { damage: i32 },
    /// 未命中。
    Miss,
    /// 重击。
    CriticalHit { damage: i32 },
}

/// 法术反制的触发与判定数据。
///
/// 瞬时结构，在一帧内创建、消费、销毁。
#[derive(Debug, Clone, Reflect)]
pub struct CounterspellData {
    /// 反制者。
    pub counterer: Entity,
    /// 被反制的法术 ID。
    pub target_spell: String,
    /// 被反制法术的环级（数值 0-9）。
    pub target_level: u8,
    /// 反制使用的环级。
    pub counter_level: u8,
    /// 判定结果。
    pub verdict: CounterspellVerdict,
}

/// 法术反制判定结果。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum CounterspellVerdict {
    /// 自动成功（反制环级 >= 目标环级）。
    AutoSuccess,
    /// 需要施法属性检定。
    CheckRequired { dc: u32, roll: Option<i32> },
    /// 失败。
    Failed,
}

/// 援护格挡的触发与执行数据。
///
/// 瞬时结构，在一帧内创建、消费、销毁。
#[derive(Debug, Clone, Reflect)]
pub struct GuardianData {
    /// 援护者。
    pub guardian: Entity,
    /// 被援护的目标。
    pub target: Entity,
    /// 攻击者。
    pub attacker: Entity,
    /// 转移的伤害量。
    pub transferred_damage: i32,
    /// 援护者位置的 x 坐标。
    pub guardian_x: i32,
    /// 援护者位置的 y 坐标。
    pub guardian_y: i32,
}

// ─── 护盾术数据（瞬时） ───────────────────────────────────────────

/// 护盾术反应的触发与执行数据。
///
/// 瞬时结构，在一帧内创建、消费、销毁。
#[derive(Debug, Clone, Reflect)]
pub struct ShieldData {
    /// 施放护盾术的单位。
    pub caster: Entity,
    /// 攻击者。
    pub attacker: Entity,
    /// 原始 AC（护盾术增加 +5）。
    pub original_ac: i32,
    /// 增强后的 AC。
    pub boosted_ac: i32,
    /// 原攻击检定值。
    pub attack_roll: i32,
    /// 护盾术生效后攻击是否命中。
    pub still_hit: bool,
}
