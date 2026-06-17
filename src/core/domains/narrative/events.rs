//! 领域事件 — Narrative 域对外发布的事件
//!
//! 所有跨域通信必须通过 Event，禁止直接引用对方数据结构（Data Law 012）。
//!
//! 事件订阅关系详见 docs/02-domain/domains/narrative_domain.md §6

use bevy::prelude::*;

/// 对话开始时触发。
///
/// 订阅者：
/// - UI：打开对话界面/显示 NPC 头像和文本
/// - Quest：检查是否有对话相关的任务
#[derive(Event, Debug, Clone)]
pub struct DialogueStarted {
    /// 启动对话的实体
    pub entity: Entity,
    /// NPC 实体
    pub npc: Entity,
    /// 对话树 ID
    pub tree_id: String,
    /// 入口节点 ID
    pub entry_node_id: String,
    /// 可用分支选项
    pub available_choices: Vec<super::components::ChoiceOption>,
}

/// 玩家选择对话分支时触发。
///
/// 订阅者：
/// - StoryFlag：记录选择的标记
/// - Quest：接受新任务或推进已有任务
/// - Faction：更新声望
/// - UI：推进对话到下一节点
#[derive(Event, Debug, Clone)]
pub struct ChoiceMade {
    /// 做出选择的实体
    pub entity: Entity,
    /// 对话树 ID
    pub dialogue_id: String,
    /// 选择的分支 ID
    pub choice_id: String,
    /// 设置的 StoryFlag 列表
    pub story_flags_set: Vec<(String, String)>,
}

/// 故事标记被设置时触发。
///
/// 订阅者：
/// - Narrative：解锁新的对话分支/节点
/// - Quest：检查任务条件的满足状态
/// - UI：显示状态变化通知
#[derive(Event, Debug, Clone)]
pub struct StoryFlagSet {
    /// 触发设置的实体
    pub entity: Entity,
    /// 标记 ID
    pub flag_id: String,
    /// 标记值
    pub value: String,
    /// 来源（"dialogue"/"quest"/"event"）
    pub source: String,
}

/// 演出开始时触发。
///
/// 订阅者：
/// - UI：切换播放模式/隐藏 HUD
/// - Cue：过场音效
#[derive(Event, Debug, Clone)]
pub struct CutsceneStarted {
    /// 演出 ID
    pub cutscene_id: String,
    /// 持续时间（秒）
    pub duration: f32,
    /// 参与者列表
    pub participants: Vec<Entity>,
}

/// 演出结束时触发。
///
/// 订阅者：
/// - UI：恢复 HUD/交互模式
#[derive(Event, Debug, Clone)]
pub struct CutsceneEnded {
    /// 演出 ID
    pub cutscene_id: String,
}
