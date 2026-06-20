//! ECS Components — 叙事/对话领域组件
//!
//! 定义对话状态、故事标记、演出状态等 ECS 组件。
//! 详见 docs/02-domain/domains/narrative_domain.md

use std::collections::HashMap;

use bevy::prelude::*;

// ─── 核心枚举 ─────────────────────────────────────────────────────

/// 对话状态阶段状态机。
///
/// ```text
/// NotStarted → Speaking → End
/// ```
/// 玩家选择退出或对话树完毕时进入 End。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum DialoguePhase {
    /// 未开始
    #[default]
    NotStarted,
    /// 对话中
    Speaking,
    /// 对话结束
    End,
}

/// 演出状态阶段状态机。
///
/// ```text
/// Idle → Playing ⇄ Paused → Finished
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum CutscenePhase {
    /// 未播放
    #[default]
    Idle,
    /// 播放中
    Playing,
    /// 暂停
    Paused,
    /// 完成
    Finished,
}

/// 对话分支选项（运行时展示用，非数据定义）。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct ChoiceOption {
    /// 分支 ID
    pub choice_id: String,
    /// 玩家可见的选项文本
    pub text: String,
    /// 该分支是否可见（基于条件过滤）
    pub visible: bool,
}

// ─── 对话节点定义（运行时常量） ──────────────────────────────────

/// 对话分支定义。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct ChoiceDef {
    /// 分支 ID
    pub id: String,
    /// 选项文本
    pub text: String,
    /// 分支指向的下一节点 ID（None = 结束对话）
    pub next_node_id: Option<String>,
    /// 需要设置的故事标记 (flag_id → value)
    pub set_flags: Vec<(String, String)>,
    /// 是否需要特殊条件可见（预留，当前由 content 定义）
    pub condition_ref: Option<String>,
}

/// 对话节点定义。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct DialogueNodeDef {
    /// 节点 ID
    pub id: String,
    /// NPC 对话文本
    pub npc_text: String,
    /// 可选分支
    pub choices: Vec<ChoiceDef>,
    /// 是否为重要对话（不可跳过，不变量 3.4）
    pub is_important: bool,
    /// 入口前提条件（预留）
    pub condition_ref: Option<String>,
}

/// 对话树定义（全局 Resource — 由内容加载填充）。
#[derive(Resource, Debug, Clone, Default, Reflect)]
#[reflect(Resource)]
pub struct DialogueTreeRegistry {
    /// dialogue_tree_id → entry_node_id
    pub trees: HashMap<String, String>,
    /// node_id → DialogueNodeDef
    pub nodes: HashMap<String, DialogueNodeDef>,
}

impl DialogueTreeRegistry {
    /// 创建一个空的 DialogueTreeRegistry。trees 和 nodes 均为空。
    pub fn new() -> Self {
        Self {
            trees: HashMap::new(),
            nodes: HashMap::new(),
        }
    }

    /// 注册对话树。
    pub fn register_tree(&mut self, tree_id: impl Into<String>, entry_node_id: impl Into<String>) {
        self.trees.insert(tree_id.into(), entry_node_id.into());
    }

    /// 注册对话节点。
    pub fn register_node(&mut self, node: DialogueNodeDef) {
        let id = node.id.clone();
        self.nodes.insert(id, node);
    }

    /// 获取对话树的入口节点。
    pub fn entry_node(&self, tree_id: &str) -> Option<&DialogueNodeDef> {
        self.trees
            .get(tree_id)
            .and_then(|node_id| self.nodes.get(node_id))
    }

    /// 获取指定节点。
    pub fn get_node(&self, node_id: &str) -> Option<&DialogueNodeDef> {
        self.nodes.get(node_id)
    }
}

// ─── ECS Components ───────────────────────────────────────────────

/// 当前对话状态。
///
/// 当玩家与 NPC 对话时附加到玩家实体上，对话结束后移除。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct DialogueState {
    /// 当前对话阶段
    pub phase: DialoguePhase,
    /// 对话树 ID
    pub tree_id: String,
    /// 当前节点 ID
    pub current_node_id: String,
    /// 当前可选分支
    pub available_choices: Vec<ChoiceOption>,
    /// 对话起始时的帧/回合
    pub started_at: f64,
}

impl DialogueState {
    /// 创建新的对话状态。
    pub fn new(
        tree_id: impl Into<String>,
        entry_node_id: impl Into<String>,
        choices: Vec<ChoiceOption>,
        time: f64,
    ) -> Self {
        Self {
            phase: DialoguePhase::Speaking,
            tree_id: tree_id.into(),
            current_node_id: entry_node_id.into(),
            available_choices: choices,
            started_at: time,
        }
    }

    /// 跳转到新节点。
    pub fn advance(&mut self, node_id: impl Into<String>, choices: Vec<ChoiceOption>) {
        self.current_node_id = node_id.into();
        self.available_choices = choices;
    }

    /// 结束对话。
    pub fn end(&mut self) {
        self.phase = DialoguePhase::End;
    }
}

/// 故事标记集合。
///
/// 记录玩家在剧情中做出的关键选择或达成的状态。
/// 标记一旦设置不可回退（不变量 3.3）。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct StoryFlags {
    /// flag_id → value（"救了村民" → "true", "选择的阵营" → "faction_a"）
    pub flags: HashMap<String, String>,
}

impl StoryFlags {
    /// 创建空的标记集合。
    pub fn new() -> Self {
        Self {
            flags: HashMap::new(),
        }
    }

    /// 设置故事标记。
    ///
    /// 不变量 3.3：标记一旦设置不可覆盖为不同值。
    /// 如果标记已存在且新值不同，返回 false 表示拒绝覆盖。
    pub fn set_flag(&mut self, flag_id: impl Into<String>, value: impl Into<String>) -> bool {
        let id = flag_id.into();
        let new_val = value.into();
        if let Some(existing) = self.flags.get(&id) {
            if *existing != new_val {
                tracing::warn!(target: "narrative",
                    event = "narrative.story_flag.override_rejected",
                    flag_id = %id,
                    existing = %existing,
                    rejected = %new_val,
                    "StoryFlag '{}' 已设为 '{}'，拒绝覆盖为 '{}'",
                    id, existing, new_val
                );
                return false;
            }
            return true; // 相同值，幂等
        }
        self.flags.insert(id, new_val);
        true
    }

    /// 检查故事标记是否等于指定值。
    pub fn check(&self, flag_id: &str, expected: &str) -> bool {
        self.flags.get(flag_id).map(|v| v.as_str()) == Some(expected)
    }

    /// 检查故事标记是否存在。
    pub fn has(&self, flag_id: &str) -> bool {
        self.flags.contains_key(flag_id)
    }
}

impl Default for StoryFlags {
    fn default() -> Self {
        Self::new()
    }
}

/// 演出状态。
///
/// 控制过场动画/镜头/音效的播放。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct CutsceneState {
    /// 当前演出阶段
    pub phase: CutscenePhase,
    /// 演出 ID
    pub cutscene_id: String,
    /// 持续时间（秒）
    pub duration: f32,
    /// 已播放时长（秒）
    pub elapsed: f32,
    /// 参与者列表
    pub participants: Vec<Entity>,
}

impl CutsceneState {
    /// 创建新的演出状态。
    pub fn new(cutscene_id: impl Into<String>, duration: f32, participants: Vec<Entity>) -> Self {
        Self {
            phase: CutscenePhase::Playing,
            cutscene_id: cutscene_id.into(),
            duration,
            elapsed: 0.0,
            participants,
        }
    }

    /// 暂停演出。
    pub fn pause(&mut self) {
        if self.phase == CutscenePhase::Playing {
            self.phase = CutscenePhase::Paused;
        }
    }

    /// 恢复演出。
    pub fn resume(&mut self) {
        if self.phase == CutscenePhase::Paused {
            self.phase = CutscenePhase::Playing;
        }
    }
}

/// 对话历史（全局 Resource）。
///
/// 记录玩家已看过的对话和选择，支持对话回溯/跳过。
#[derive(Resource, Debug, Clone, Default, Reflect)]
#[reflect(Resource)]
pub struct DialogueHistory {
    /// tree_id → 已完成节点的 ID 列表
    pub completed_trees: HashMap<String, Vec<String>>,
    /// 所有玩家的选择记录 (dialogue_id → choice_id)
    pub choices_made: Vec<(String, String)>,
}

impl DialogueHistory {
    /// 创建空的对话历史。
    pub fn new() -> Self {
        Self {
            completed_trees: HashMap::new(),
            choices_made: Vec::new(),
        }
    }

    /// 记录对话树中的一个节点被访问过。
    pub fn visit_node(&mut self, tree_id: impl Into<String>, node_id: impl Into<String>) {
        self.completed_trees
            .entry(tree_id.into())
            .or_default()
            .push(node_id.into());
    }

    /// 记录玩家选择。
    pub fn record_choice(&mut self, dialogue_id: impl Into<String>, choice_id: impl Into<String>) {
        self.choices_made
            .push((dialogue_id.into(), choice_id.into()));
    }

    /// 检查某节点是否已被访问过。
    pub fn has_visited(&self, tree_id: &str, node_id: &str) -> bool {
        self.completed_trees
            .get(tree_id)
            .map(|nodes| nodes.contains(&node_id.to_string()))
            .unwrap_or(false)
    }

    /// 跳过已读对话（如果对话树的所有节点都访问过，返回 true）。
    pub fn can_skip(&self, tree_id: &str) -> bool {
        // 简化实现：如果有过选择记录，视为可跳过
        self.choices_made.iter().any(|(tid, _)| tid == tree_id)
    }
}
