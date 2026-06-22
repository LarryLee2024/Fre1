---
id: 01-architecture.ADR-033
title: ADR-033 — Narrative & Quest Architecture
status: approved
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-033: 叙事 / 任务 / 召唤架构

## 状态

**Approved** — 依赖 ADR-030（Progression/Inventory）和 ADR-020（Combat Pipeline），本架构决策正式生效。

## 背景

叙事系统（故事/对话）和任务系统（目标/追踪/奖励）构成游戏的非战斗内容层。它们监听战斗和探索事件来推进故事和任务进度。召唤是战斗中的特殊机制——创建临时单位。

## 引用的领域规则与数据架构

- `docs/02-domain/domains/narrative_domain.md` — Narrative 领域规则
- `docs/02-domain/domains/quest_domain.md` — Quest 领域规则
- `docs/04-data/domains/narrative_schema.md` — Narrative Schema
- `docs/04-data/domains/quest_schema.md` — Quest Schema

## 决策

### 1. Narrative 架构

#### 1.1 故事状态管理

```rust
/// StoryState — 全局故事进度
#[derive(Resource)]
pub struct StoryState {
    /// 当前激活的故事章节
    pub current_chapter: Option<ChapterId>,
    /// 已触发的故事标记
    pub flags: HashSet<StoryFlag>,
    /// 已完成章节
    pub completed_chapters: Vec<ChapterId>,
    /// 对话分支选择历史
    pub choices: Vec<ChoiceRecord>,
}

pub struct ChoiceRecord {
    pub dialogue_id: DialogueId,
    pub choice_index: u32,
    pub timestamp: GameTime,
}
```

#### 1.2 故事触发机制

Narrative 系统不主动运行业务逻辑，它通过监听领域事件来推进故事：

```rust
/// 故事触发器 — 配置化
pub struct StoryTrigger {
    pub trigger_id: StoryTriggerId,
    pub condition: StoryCondition,        // 触发条件
    pub action: StoryAction,              // 触发后执行的动作
    pub one_shot: bool,                   // 是否只触发一次
    pub priority: i32,
}

pub enum StoryCondition {
    ChapterCompleted(ChapterId),
    StoryFlagSet(StoryFlag),
    QuestCompleted(QuestId),
    UnitJoined(UnitTemplateId),
    BattleWon(BattleId),
    ChoiceMade(DialogueId, u32),
    Custom(ConditionDefId),     // 委托给 Condition 系统
}

pub enum StoryAction {
    SetFlag(StoryFlag),
    StartDialogue(DialogueId),
    StartChapter(ChapterId),
    UnlockQuest(QuestId),
    ModifyWorld(WorldStateMod),
    GiveItem(ItemDefId, u32),
    Reward(QuestReward),
}
```

#### 1.3 对话系统

```rust
/// DialogueDef — 配置加载
#[derive(Asset, TypePath)]
pub struct DialogueDef {
    pub id: DialogueDefId,
    pub nodes: Vec<DialogueNode>,
    pub conditions: Vec<ConditionDef>,      // 可触发此对话的条件
    pub on_complete: Vec<StoryAction>,       // 对话完成后执行
}

pub struct DialogueNode {
    pub speaker: SpeakerRef,
    pub text: LocalizationKey,
    pub choices: Vec<DialogueChoice>,
    pub on_enter: Vec<StoryAction>,
    pub on_exit: Vec<StoryAction>,
}

pub enum SpeakerRef {
    ByEntity(Entity),       // 运行时绑定
    ByTemplate(UnitTemplateId), // 通过模板查找
    Narration,              // 旁白
}
```

#### 1.4 对话触发流程

```
StoryTrigger 激活 / 玩家与 NPC 交互
       │
       ▼
DialogueSystem
  ├── 加载 DialogueDef
  ├── 检查 conditions
  ├── 创建 DialogueSession Resource
  └── 切换到 Dialogue 状态（UI 接管）
       │
       ▼
玩家选择 → DialogueNode.on_exit → 下一个 Node
       │
       ▼
对话结束 → DialogueDef.on_complete
  ├── StoryAction 执行（SetFlag / GiveReward）
  └── 销毁 DialogueSession → 恢复之前的状态
```

### 2. Quest 架构

#### 2.1 任务定义

```rust
/// QuestDef — 配置加载
#[derive(Asset, TypePath)]
pub struct QuestDef {
    pub id: QuestDefId,
    pub name: LocalizationKey,
    pub description: LocalizationKey,
    pub category: QuestCategory,     // Main | Side | Daily | Chain
    pub objectives: Vec<QuestObjective>,
    pub rewards: QuestReward,
    pub prerequisites: Vec<QuestPrerequisite>,
    pub time_limit: Option<u32>,     // 回合限制
    pub failure_conditions: Vec<ConditionDef>,
    pub next_quest: Option<QuestDefId>, // 连锁任务
}

pub enum QuestObjective {
    KillMonsters {
        target_tag: TagId,
        count: u32,
        current: u32,
    },
    CollectItems {
        item_def_id: ItemDefId,
        count: u32,
        current: u32,
    },
    ReachLocation {
        marker: WorldMarkerId,
    },
    TalkToNpc {
        npc_template: UnitTemplateId,
    },
    ProtectUnit {
        target: Entity,
        duration: u32,
    },
    Custom(ConditionDefId),   // 委托给 Condition 系统
}

pub struct QuestReward {
    pub xp: u64,
    pub gold: u64,
    pub items: Vec<(ItemDefId, u32)>,
    pub reputation: Vec<(FactionDefId, i32)>,
}
```

#### 2.2 任务生命周期

```
Quest 解锁 (prerequisites 满足)
       │
       ▼
QuestState::Available
       │
玩家接受
       │
       ▼
QuestState::Active
       │
       ▼
QuestTrackingSystem (监听领域事件)
  ├── 监听 CombatResult → 更新 KillMonsters 进度
  ├── 监听 ItemAcquired → 更新 CollectItems 进度
  ├── 监听 GridPos 变化 → 检查 ReachLocation
  ├── 监听 DialogueComplete → 检查 TalkToNpc
  └── 检查 failure_conditions
       │
       ▼
所有 objectives 完成？
       │
       ├── Yes → QuestState::Completed
       │          └── 发放奖励
       │
       └── No → 继续追踪
              │
              └── failure 条件触发？
                     │
                     ├── Yes → QuestState::Failed
                     └── No → 继续
```

#### 2.3 任务追踪

```rust
/// QuestInstance — 运行时
#[derive(Component)]
pub struct QuestInstance {
    pub quest_def_id: QuestDefId,
    pub state: QuestState,
    pub objective_progress: Vec<ObjectiveProgress>,
    pub accepted_at: GameTime,
}

pub enum QuestState {
    Available,
    Active,
    Completed,
    Failed,
}

pub struct ObjectiveProgress {
    pub objective_index: usize,
    pub current: u32,
    pub target: u32,
    pub is_complete: bool,
}
```

#### 2.4 任务进度监听

```rust
fn track_kill_objectives(
    mut combat_results: EventReader<CombatResult>,
    mut quest_query: Query<&mut QuestInstance>,
) {
    for result in combat_results.read() {
        if result.did_kill {
            for mut quest in quest_query.iter_mut() {
                for progress in &mut quest.objective_progress {
                    if let QuestObjective::KillMonsters { ref target_tag, .. } = /* ... */ {
                        // 检查被杀单位是否有 target_tag
                        // progress.current += 1
                    }
                }
            }
        }
    }
}
```

## Module Design

```
src/core/domains/narrative/
  ├── plugin.rs              — NarrativePlugin
  ├── resources.rs           — StoryState, DialogueSession
  ├── systems.rs             — story_trigger_checker, dialogue_runner
  ├── events.rs              — StoryEvent, DialogueStarted, DialogueEnded
  └── integration/           — 跨域访问 ACL（ADR-046） DialogueDef, StoryFlag, ChapterId

src/core/domains/quest/
  ├── plugin.rs              — QuestPlugin
  ├── components.rs          — QuestInstance
  ├── resources.rs           — QuestLog, QuestTracker
  ├── systems.rs             — quest_acceptor, progress_tracker, reward_dispatcher
  ├── events.rs              — QuestAccepted, QuestProgressed, QuestCompleted, QuestFailed
  └── integration/           — 跨域访问 ACL（ADR-046） QuestDef, QuestObjective, QuestState
```

## Communication Design

| 通信 | 机制 | 方向 |
|------|------|------|
| 战斗 → 任务 | Event (`CombatResult`) | combat → quest |
| 任务 → 奖励 | Event (`QuestCompleted`) → `add_xp` / `add_item` | quest → progression/inventory |
| 故事 → 任务 | Event (`StoryEvent`) → `unlock_quest` | narrative → quest |
| 交互 → 对话 | `DialogueRequest` Event | interaction → narrative |
| 故事进展 | `StoryState.flags` 直接操作 | narrative 内部 |

## 边界定义

### 允许
- Quest 监听任何 Feature 的领域事件（只读）
- Narrative 在故事触发时修改 WorldState
- Quest 发放奖励时调用 Progression/Inventory 的公开 API
- Dialogue 通过 StoryAction 推进故事

### 🟥 禁止
- Quest 直接修改 Combat 或 Progression 的内部状态（奖励通过 API）
- Narrative 在对话中执行战斗逻辑
- Quest 任务目标在未激活时仍监听事件
- 故事触发器（StoryTrigger）包含无法序列化的条件
- Dialogue 阻塞游戏主循环（dialog 是 UI 层的责任）

## Forbidden

| 禁止行为 | 理由 |
|---------|------|
| Quest 系统直接操作 Inventory | 必须通过事件或 API |
| 故事标记硬编码在代码中 | 必须配置化 |
| 任务进度监听所有事件（不做过滤） | 性能浪费 |
| 对话中触发 Effect Pipeline | 对话不是战斗上下文 |
| 任务奖励直接 create Entity | 通过 Inventory API |

## Definition / Instance Design

- **Definition**: `ChapterDef` (config), `DialogueDef` (Asset), `QuestDef` (Asset), `StoryTrigger` (config)
- **Instance**: `StoryState` (Resource), `QuestInstance` (Component), `DialogueSession` (Resource)
- **Persistence**: `StoryState.flags`, `StoryState.completed_chapters`, `QuestInstance`（活跃任务列表及进度）

## 后果

### 正面
- 故事和任务系统被动监听领域事件，不与业务系统耦合
- 任务目标类型通过枚举扩展，新增目标类型不需要改架构
- 对话系统解耦为配置驱动，本地化通过 Key 引用
- 故事触发器配置化，叙事设计师可独立工作

### 负面
- 故事触发的条件可能变得非常复杂（嵌套 AND/OR/NOT）
- Quest 监听大量事件，需要确保性能（批次处理、每帧限制）
- 对话系统的 Speaker 绑定在运行时需要 Entity 查找逻辑

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 叙事逻辑硬编码在 Rust 代码中 | 违反 Rule/Content 分离 |
| 任务与故事合并为一个系统 | 关注点不同：故事是线性推进，任务是目标追踪 |
| 使用第三方 narrative engine（如 Yarn Spinner） | 需要 Rust 绑定，初期权重过高 |
| 对话使用 Entity Component 存储 | 对话是瞬态数据，不是 ECS 的适用场景 |

## 评审要点

- [ ] 任务目标类型是否覆盖了核心场景？是否需要 Escort/Interact 等？
- [ ] 故事状态机——是线性章节（Chapter）还是分支叙事（Branching）？
- [ ] 连锁任务（next_quest）的触发时机——自动接取还是手动？
- [ ] 任务奖励发放的时序——同时发放还是顺序发放？
