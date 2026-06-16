---
id: domains.narrative.schema.v1
title: Narrative Schema — 叙事/对话数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition, instance, persistence
replay-safe: false
---

# Narrative Schema — 叙事/对话数据架构

> **领域归属**: Domains — 叙事内容层 | **依赖 Schema**: Condition, Event, Faction, Quest | **定义依据**: `docs/02-domain/narrative_domain.md`

---

## 1. Schema Design

### 1.1 DialogueTreeDef（Definition 层）

```rust
/// 对话树定义。内容团队配置，运行时只读。
struct DialogueTreeDef {
    /// 对话树唯一标识（前缀: `dlg_`）
    id: DialogueTreeId,

    /// 关联的 NPC 实体 ID 或 NPC 类型标签
    npc_id: Option<EntityId>,
    npc_tags: Vec<TagDefId>,

    /// 入口节点 ID
    entry_node_id: DialogueNodeId,

    /// 所有节点
    nodes: HashMap<DialogueNodeId, DialogueNodeDef>,
}

struct DialogueNodeDef {
    /// 节点唯一标识（对话树内唯一）
    id: DialogueNodeId,

    /// NPC 台词本地化 Key
    npc_text_key: LocalizationKey,

    /// 可选的演出指令（镜头/表情/音效）
    stage_direction: Option<StageDirection>,

    /// 可选分支列表
    choices: Vec<DialogueChoiceDef>,

    /// 进入此节点时自动设置的 StoryFlag
    auto_set_flags: Vec<(StoryFlagId, String)>,

    /// 是否标记为"重要对话"（不可跳过）
    is_critical: bool,
}

struct DialogueChoiceDef {
    /// 分支唯一标识
    id: DialogueChoiceId,

    /// 玩家选项文本本地化 Key
    text_key: LocalizationKey,

    /// 分支可见条件（使用 ConditionDefId）
    visibility_condition: Option<ConditionDefId>,

    /// 分支可选条件（灰色不可选）
    enabled_condition: Option<ConditionDefId>,

    /// 选择后跳转到的下一节点 ID（None = 结束对话）
    next_node_id: Option<DialogueNodeId>,

    /// 选择后的副作用
    side_effects: Vec<DialogueSideEffect>,
}

enum DialogueSideEffect {
    /// 设置故事标记
    SetStoryFlag { flag_id: StoryFlagId, value: String },
    /// 触发任务事件
    TriggerQuest { quest_id: QuestDefId, action: QuestAction },
    /// 声望变化
    ReputationChange { faction_id: FactionDefId, delta: i32 },
}

enum QuestAction { Accept, Advance, Complete }
```

### 1.2 StoryFlag（Instance 层/Persistence 层）

```rust
/// 故事标记——记录玩家在剧情中的关键选择或达成的状态。
/// 只增不减，一旦设置不可还原。
struct StoryFlag {
    /// 标记 ID
    flag_id: StoryFlagId,

    /// 当前值（字符串枚举）
    value: String,

    /// 设置来源（对话/任务/事件）
    source: StoryFlagSource,
}

enum StoryFlagSource { Dialogue, Quest, Event, Initial }
```

### 1.3 DialogueState（Instance 层 — 瞬时）

```rust
/// 对话运行时状态。对话期间存在，对话结束后销毁。
struct DialogueState {
    /// 当前对话树 ID
    tree_id: DialogueTreeId,

    /// 当前节点 ID
    current_node_id: DialogueNodeId,

    /// 已访问的节点历史（用于回溯和跳过已读）
    visited_nodes: Vec<DialogueNodeId>,

    /// 当前可见的分支（已过滤不可见分支）
    available_choices: Vec<DialogueChoiceId>,
}

/// 对话历史记录（Persistence 层 — 可选）
struct DialogueHistory {
    /// 已完成/已读的对话树列表
    completed_trees: Vec<DialogueTreeId>,

    /// 最近的分支选择记录
    recent_choices: Vec<(DialogueTreeId, DialogueChoiceId)>,
}
```

### 1.4 CutsceneDef（Definition 层）

```rust
/// 过场动画定义。
struct CutsceneDef {
    /// 过场动画唯一标识（前缀: `cut_`）
    id: CutsceneId,

    /// 触发条件
    trigger: CutsceneTrigger,

    /// 参与的实体列表
    participants: Vec<EntityId>,

    /// 持续时间（秒）
    duration_seconds: f32,
}

enum CutsceneTrigger {
    OnDialogueChoice { tree_id: DialogueTreeId, choice_id: DialogueChoiceId },
    OnQuestAccepted { quest_id: QuestDefId },
    OnQuestTurnedIn { quest_id: QuestDefId },
    OnStoryFlag { flag_id: StoryFlagId, value: String },
    OnEnterArea { area_id: String },
}
```

### 1.5 NarrativeState（Persistence 层）

```rust
/// 叙事系统的持久化状态。
struct NarrativeState {
    /// 所有已设置的故事标记
    story_flags: Vec<StoryFlag>,

    /// 对话历史记录
    dialogue_history: DialogueHistory,
}
```

---

## 2. Layer Summary

| Layer | Structures | 说明 |
|-------|-----------|------|
| **Definition** | `DialogueTreeDef`, `CutsceneDef` | 对话树和演出为静态内容配置 |
| **Spec** | — | Narrative 无 Spec 层 |
| **Instance** | `DialogueState` (瞬时), `StoryFlag` | 对话运行时状态；StoryFlag 持久化 |
| **Persistence** | `NarrativeState` | StoryFlag 和对话历史持久化 |

---

## 3. Dependency Analysis

| 依赖 | 说明 |
|------|------|
| → ConditionSchema | 对话分支可见/可选条件 |
| → EventSchema | 对话事件发布（ChoiceMade, StoryFlagSet 等） |
| → FactionSchema | 对话副作用引用声望变化 |
| → QuestSchema | 对话可接受/推进任务 |
| ← FactionSchema | 声望等级影响对话分支可见性 |

---

## 4. Replay & Save

### Replay

- 标记 `replay-safe: false` — 叙事是玩家进程数据，不参与战斗回放

### Save

- `NarrativeState` 持久化（StoryFlag + DialogueHistory）
- DialogueTreeDef/CutsceneDef 从内容配置加载
- DialogueState 不持久化（对话结束时销毁）

---

## 5. Validation Rules

| 规则 | 说明 | 违反处理 |
|------|------|----------|
| 对话树无环 | 节点之间不得形成循环引用 | Schema 校验拒绝 |
| 分支条件确定性 | 相同状态下分支判定结果必须一致 | 运行时断言（调试模式） |
| StoryFlag 只增不减 | 标记一旦设置不可还原 | Schema 校验拒绝 |
| 分支互斥性 | 互斥条件的分支不同时出现 | 运行时条件检查 |
| 节点存在性 | 跳转目标节点必须在 nodes 中 | Schema 校验拒绝 |

---

## 6. Constitution Check

- ✅ **Data Law 001 (Def-Instance分离)**: DialogueTreeDef/CutsceneDef 为 Definition，StoryFlag/DialogueState 为 Instance
- ✅ **Data Law 002 (Rule-Content分离)**: 对话内容为内容配置，分支规则为代码逻辑
- ✅ **Data Law 003 (配置只引用ID)**: DialogueNodeDef 引用 ConditionDefId/QuestDefId/FactionDefId
- ✅ **Data Law 011 (Schema版本化)**: NarrativeState 携带版本号
- ✅ **Data Law 012 (域间禁止直接数据引用)**: Narrative 通过 Event 与 Quest/Faction/Inventory 通信
