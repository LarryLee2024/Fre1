//! UiBinding — ViewModel 字段到 UI Node 的映射标记（反 Marker 模式）
//!
//! 替代大量独立 Marker 结构体（如 `struct HpText;`、`struct ManaText;`），
//! 避免 Archetype 爆炸，所有 UI 绑定通过单一 `Query<&UiBinding>` 可查。
//!
//! 使用方式：
//! - 静态 Widget（如 HP 条、回合指示器）使用无参变体
//! - 动态 Widget（如技能槽位）使用有参变体
//!
//! 参见 `docs/06-ui/02-design-system/focus-binding.md` §4

use bevy::prelude::*;

/// UI 绑定标识 — 替代大量独立 Marker 结构体
///
/// 标识 Widget 实体对应的 ViewModel 数据字段。
/// Dirty<T> 标记消费后，通过 UiBinding 找到对应的 Style/Text 组件进行刷新。
///
/// # 变体分类
/// - Battle HUD: Hp, Mp, Ap, Turn, Phase
/// - Character Panel: Level, Exp, Name
/// - Skill Panel: SkillSlot, Cooldown
/// - Inventory: ItemSlot, Gold
/// - Quest: QuestEntry
/// - General: Tooltip, Modal, Notification
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub enum UiBinding {
    // ── Battle HUD ──
    /// HP 值/条
    Hp,
    /// 最大 HP
    MaxHp,
    /// MP 值/条
    Mp,
    /// 最大 MP
    MaxMp,
    /// 行动点
    Ap,
    /// 最大行动点
    MaxAp,
    /// 回合数
    Turn,
    /// 战斗阶段
    Phase,

    // ── Character Panel ──
    /// 等级
    Level,
    /// 经验值
    Exp,
    /// 角色名称
    Name,
    /// 角色等级文本
    CharacterLevel,

    // ── Skill Panel ──
    /// 第 N 个技能槽（0~N-1）
    SkillSlot(u8),
    /// 冷却
    Cooldown,

    // ── Inventory ──
    /// 第 N 个物品槽（0~N-1）
    ItemSlot(u8),
    /// 金币
    Gold,

    // ── Quest ──
    /// 第 N 个任务条目（0~N-1）
    QuestEntry(u16),

    // ── General ──
    /// 工具提示区域
    Tooltip,
    /// 模态弹窗
    Modal,
    /// 通知
    Notification,
    /// 通用文本
    Text,
    /// 通用图标
    Icon,
}
