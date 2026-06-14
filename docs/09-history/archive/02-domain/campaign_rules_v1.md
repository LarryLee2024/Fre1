---
id: history.archive.campaign_rules_v1
title: campaign_rules_v1
status: archived
owner: domain-designer
created: 2026-06-14
updated: 2026-06-14
superseded_by: ../../02-domain/campaign/
---

# Campaign 领域规则 v1.0

Version: 1.0
Status: Draft
Applies To: 战役流程编排、关卡序列管理、关卡选择与解锁、内容资产组织

---

## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Campaign | 一场完整的战役，由一组按顺序排列的关卡（Stage）组成，代表从开始到通关的完整流程 | 负责：战役的整体编排和关卡顺序；不负责：单个关卡的内部逻辑、数据加载 |
| Stage | 战役流程中的单个关卡位置标识，记录关卡 ID 和其在战役中的次序 | 负责：定义关卡在战役中的位置；不负责：关卡的实际数据（由 Level 负责） |
| Battle | 一次完整的战斗过程，从关卡加载到胜负结算的运行时概念 | 负责：战斗流程和效果管线；不负责：关卡配置定义或战役编排（已由 battle_rules_v1 定义） |
| Level | 一个完整的战斗关卡配置，包含地图、单位部署、胜负条件的全部数据（与 level_rules_v1 定义一致） | 负责：关卡的完整数据描述；不负责：关卡在战役中的位置和顺序 |
| ContentAsset | 内容资产的总称，包含 Level、UnitTemplate、SkillDef、BuffDef 等所有从 RON 加载的配置数据 | 负责：资产的统一管理和加载；不负责：资产的运行时实例 |
| CampaignProgress | 玩家的战役进度记录，记录已通关的 Stage、解锁的 Stage、当前选中的 Stage | 负责：进度状态的运行时跟踪；不负责：进度的持久化存储 |
| LevelSelect | 玩家从战役的关卡列表中选择一个可玩关卡的行为 | 负责：可选关卡列表的呈现；不负责：进度的判定规则 |

### 术语关系

```
Campaign
  └── Stage[]（有序列表）
       └── Level（通过 level_id 引用）
            ├── Map（地形数据）
            ├── UnitDeployDef[]（单位部署）
            └── VictoryCondition（胜负条件）

CampaignProgress
  ├── completed_stages: StageId[]（已完成关卡）
  ├── unlocked_stages: StageId[]（已解锁关卡）
  └── current_stage: StageId?（当前选中的关卡）
```

### 与已有术语的关系

| 已有术语 | 所属领域 | 与 Campaign 的关系 |
|----------|----------|-------------------|
| Level | level_rules_v1 | Stage 通过 level_id 引用一个已注册的 LevelConfig（不内嵌，只引用） |
| LevelConfig | level_rules_v1 | Campaign 不涉及 LevelConfig 的内部结构，只通过 ID 引用 |
| LevelRegistry | level_rules_v1 | Campaign 从 LevelRegistry 查找关卡的完整数据 |
| Stage | level_rules_v1（已有定义） | 已有定义："关卡在战役流程中的位置标识" — 本文档扩展 Stage 为 Campaign 的子元素 |
| GameOverState | victory_condition_rules_v1 | 关卡完成后，GameOverState 驱动 CampaignProgress 更新 |
| AppState | turn_rules | Campaign 流程决定 AppState 的切换（LevelSelect → InGame → GameOver → LevelSelect） |
| UnitTemplate | character_rules | Campaign 不直接涉及，Level 内部通过 template ID 引用 |

### 已有定义确认

`level_rules_v1.md` 中已定义的 "Stage" 术语：

> Stage | 关卡在战役流程中的位置标识（如"第一章第三关"） | 负责：关卡之间的串联关系；不负责：单个关卡内部逻辑

本文档与已有定义保持完全一致。Stage 仍为"位置标识"，新增 Campaign 作为 Stage 的容器。

---

## 2. 状态机

### 战役流程状态机

```
MainMenu
  │ [玩家选择"开始游戏"]
  ▼
LevelSelect
  │ [玩家选择一个已解锁的 Stage]
  ├─── [Stage 对应 Level 从 LevelRegistry 中找到] ──► InGame
  │
  └─── [Level 不存在或加载失败] ──► Error

InGame
  │ [胜负条件达成：GameOverState ≠ Playing]
  ▼
GameOver
  │ [玩家选择"下一关" / "重玩" / "返回"]
  ├─── [下一关] ──► LevelSelect（选中下一个 Stage）
  ├─── [重玩]  ──► InGame（重新加载当前 Level）
  └─── [返回]  ──► LevelSelect

Error
  │ [显示错误信息]
  ▼
LevelSelect（返回关卡选择，跳过无效关卡）
```

状态列表：
- MainMenu：游戏主菜单，等待玩家操作
- LevelSelect：关卡选择界面，展示当前战役中已解锁的关卡
- InGame：战斗进行中（已有定义，与 turn_rules 一致）
- GameOver：战斗结束，显示结果（已有定义，与 victory_condition_rules 一致）
- Error：关卡加载失败，显示错误信息

转换规则：
- MainMenu → LevelSelect：玩家选择"开始游戏"或"继续战役"
- LevelSelect → InGame：玩家选择已解锁且可用的关卡
- InGame → GameOver：胜负条件达成（GameOverState 写入 Victory/Defeat）
- GameOver → LevelSelect：玩家选择"返回"
- GameOver → InGame：玩家选择"重玩"（重新加载当前关卡）
- LevelSelect → LevelSelect（同状态循环）：玩家切换选中的关卡但不进入
- 禁止：InGame → LevelSelect（战斗中不可选择关卡）
- 禁止：跳过 LevelSelect 直接从 MainMenu 到 InGame（即使只有一个关卡）

### 关卡解锁状态机

```
Locked
  │ [前置关卡全部完成]
  ▼
Unlocked
  │ [玩家选中并进入战斗]
  ▼
InProgress
  │ [GameOverState 变为 Victory]
  ▼
Completed
  │ [下一关卡的前置条件]
  ▼
（下一个 Locked → Unlocked）
```

状态列表：
- Locked：关卡锁定，不可选择（前置关卡未完成）
- Unlocked：关卡解锁，可选择进入
- InProgress：关卡正在进行中（等价于 InGame 状态）
- Completed：关卡已通关（GameOverState = Victory）

转换规则：
- Locked → Unlocked：该关卡的所有前置关卡状态为 Completed
- Unlocked → InProgress：玩家选择该关卡并确认进入
- InProgress → Completed：关卡胜利（GameOverState = Victory）
- InProgress → Unlocked：关卡失败（GameOverState = Defeat），回到未完成状态
- 禁止：Completed → Locked（通关不可逆）
- 禁止：跳过 Locked 直接进入 InProgress

---

## 3. 不变量（Invariants）

### 3.1 战役至少包含一个关卡 🟩

- 条件：Campaign 定义时
- 不变量：Campaign 的 Stage 列表不能为空
- 违反后果：玩家进入战役后没有可玩的关卡，游戏流程中断

### 3.2 关卡 ID 在战役内唯一 🟩

- 条件：Campaign 定义时
- 不变量：同一 Campaign 内的 Stage 引用的 level_id 不能重复
- 违反后果：同一关卡在关卡列表中多次出现，玩家混淆
- 备注：不同 Campaign 可以引用同一个 Level（复用关卡数据），但同一 Campaign 内不应重复

### 3.3 关卡 ID 必须存在于 LevelRegistry 🟥

- 条件：Campaign 加载或玩家选择关卡时
- 不变量：Stage 引用的 level_id 必须在 LevelRegistry 中有对应的 LevelConfig
- 违反后果：玩家选择关卡后无法加载，游戏流程中断
- 备注：关卡加载失败时进入 Error 状态，不崩溃

### 3.4 关卡解锁必须依赖前置关卡 🟩

- 条件：玩家选择关卡时
- 不变量：第一个 Stage 始终解锁；后续 Stage 的前置 Stage 必须为 Completed
- 违反后果：玩家跳过未通关的关卡，破坏战役连贯性

### 3.5 战役进度运行时不可持久化 🟩

- 条件：CampaignProgress 运行时
- 不变量：进度数据只存在于内存中，退出游戏后丢失
- 违反后果：持久化机制尚未实现（"只解决当前复杂度"原则），提前设计会导致过度工程
- 备注：此不变量在未来实现存档系统后应被标记为 DEPRECATED

### 3.6 首次进入战役时第一个关卡默认解锁 🟩

- 条件：Campaign 首次加载时
- 不变量：CampaignProgress 为空时，第一个 Stage 自动设置为 Unlocked
- 违反后果：玩家进入战役后没有可选择的关卡

### 3.7 Level 数据不可在运行时修改（继承自 level_rules_v1 3.2）🟥

- 条件：Campaign 引用 Level 时
- 不变量：Campaign 不得修改 LevelConfig 的任何字段
- 违反后果：被多个 Campaign 引用的 Level 数据不一致
- 宪法依据：1.1.2（Definition / Instance 分离）

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：Campaign 配置中内嵌 Level 数据（地形、单位部署等） — 理由：Campaign 只通过 level_id 引用 Level，内嵌导致数据冗余和一致性隐患（违反宪法 1.1.2 Definition/Instance 分离）
- 🟥 禁止：Campaign 修改 LevelConfig 的运行时数据 — 理由：LevelConfig 是 Definition，运行时不可变（继承 level_rules_v1 4.1）
- 🟥 禁止：为此领域设计 Quest/Save/Dialogue 系统 — 理由："只解决当前复杂度"原则，当前仅有 1 个关卡，存档等系统为时过早
- 🟥 禁止：在 Campaign 领域规则中规定 UI 表现细节 — 理由：逻辑与表现分离，Campaign 只定义数据和流程
- 🟥 禁止：Campaign 运行时依赖 Level 内部字段 — 理由：Campaign 只通过 level_id 引用 Level，不应访问 LevelConfig.terrain_grid 等内部数据
- 🟥 禁止：创建一个名为 campaign/ 的顶层模块但其中仅包含配置加载逻辑 — 理由：若当前只配置加载，应放入 AssetLoader 或 LevelRegistry 扩展；若未来需要战役运行时逻辑再创建独立模块
- 🟥 禁止：关卡锁定机制硬编码前置关卡数量 — 理由：解锁规则应通过配置表达（如 prerequisites: [stage_001]），而非代码中的 `if completed == 1`

---

## 5. 流程定义

### 5.1 Campaign 加载

- 输入：Campaign RON 文件路径 + LevelRegistry
- 处理：
  1. 读取 Campaign RON 文件，反序列化为 CampaignDef
  2. 验证所有 Stage 的 level_id 在 LevelRegistry 中存在（3.3）
  3. 验证 Stage 列表非空（3.1）
  4. 验证 level_id 在 Campaign 内唯一（3.2）
  5. 生成 CampaignProgress：第一个 Stage → Unlocked，其余 → Locked
- 输出：Campaign 运行时实例 + CampaignProgress
- 失败处理：
  - RON 解析失败：记录错误日志，游戏保持 MainMenu 状态
  - 验证失败（level_id 不存在）：记录错误日志，跳过无效 Stage
  - 所有 Stage 均无效：Campaign 为空，返回 MainMenu

### 5.2 关卡选择

- 输入：CampaignProgress + 玩家选择的 StageId
- 处理：
  1. 检查 StageId 在 CampaignProgress 中是否为 Unlocked（3.4）
  2. 检查 LevelRegistry 中存在该 Stage 引用的 LevelConfig（3.3）
  3. 设置 CampaignProgress.current_stage
  4. 触发 InGame 切换（由 UI/应用层负责）
- 输出：当前选中的 LevelConfig 引用（只读）
- 失败处理：
  - 关卡未解锁：提示玩家，不进入战斗
  - Level 不存在：显示错误信息，进入 Error 状态

### 5.3 关卡完成

- 输入：GameOverState（来自 victory_condition_rules）+ CampaignProgress
- 处理：
  1. 如果 GameOverState = Victory：
     a. 将当前 Stage 标记为 Completed
     b. 将下一个 Stage（如有）标记为 Unlocked
  2. 如果 GameOverState = Defeat：
     a. 当前 Stage 保持 Unlocked（可重玩）
     b. 不清除已累积的、影响后续关卡的状态（无"永久死亡"等机制）
- 输出：更新后的 CampaignProgress
- 失败处理：CampaignProgress 写入失败时记录错误日志

### 5.4 关卡重玩

- 输入：CampaignProgress（当前 Stage 为 Unlocked 或 Completed）+ LevelRegistry
- 处理：
  1. 重新加载当前 Stage 对应的 LevelConfig
  2. 重置所有运行时状态（TurnState、TurnOrder、CombatLog、单位位置等）
  3. 进入 InGame
- 输出：重置后的战场初始状态
- 失败处理：重置失败时保持 GameOver 状态

---

## 6. 内容资产组织（建议方案）

### 6.1 当前状态（已有）

```
assets/maps/
  tutorial.ron    ← Level 定义（地图+单位+条件 合一）
```

### 6.2 建议的过渡方案

当前仍沿用"Level 一切合一"模式（与 level_rules_v1 一致），新增 Campaign 作为外部编排层：

```
assets/
  campaigns/
    campaign_001.ron  ← Campaign 定义（仅包含 Stage 序列和 level_id 引用）
  maps/
    tutorial.ron      ← Level 定义（不变，仍是 地图+单位+条件 合一）
```

`campaign_001.ron` 示例：

```ron
(
    id: "campaign_001",
    name: "边境之旅",
    stages: [
        (id: "stage_001", level_id: "tutorial"),
        // 后续关卡：
        // (id: "stage_002", level_id: "forest_crossing"),
        // (id: "stage_003", level_id: "goblin_fort"),
    ],
)
```

### 6.3 未来地图/编队分离（预留）

当关卡数量达到 5+ 并出现地图复用时：

```
assets/
  campaigns/
    campaign_001.ron  ← 战役定义
  content/
    battles/          ← Battle 配置（组合 MapRef + Encounter + VictoryCondition）
    maps/             ← 纯地图资产（仅尺寸 + 地形网格）
    encounters/       ← 敌人编队模板（可复用）
```

此方案仅在出现"同一张地图需要用于不同关卡"或"同一批敌人需要用于不同地图"的需求时实施。在此之前，Level 仍然保持"一切合一"。

### 6.4 组织原则

- Campaign 和 Level 使用不同的资产目录（区分"编排"和"配置"）
- Level 的数据结构保持现有模式（level_rules_v1 定义的 LevelConfigDef）
- Campaign 只引用 level_id，不内嵌 Level 数据
- 转换代价为零：Level 和 Campaign 的解耦意味着未来分离 Map 和 Battle 时，Campaign 不受影响

---

## 7. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| CampaignLoaded | Campaign RON 文件解析成功并初始化进度 | campaign_id, stage_count | LevelSelect UI（展示关卡列表） |
| CampaignLoadFailed | Campaign 加载或验证失败 | campaign_id, 错误原因 | UI（显示错误提示） |
| StageSelected | 玩家选中一个关卡 | stage_id, level_id | LevelSelect UI（高亮选中关卡） |
| StageStarted | 玩家确认进入关卡 | stage_id, level_id | battle（初始化战场）、ui（切换到战斗界面） |
| StageCompleted | 关卡完成（Victory） | stage_id, 结果 | CampaignProgress（标记完成）、ui（显示胜利） |
| StageFailed | 关卡失败（Defeat） | stage_id, 结果 | CampaignProgress（保持未完成）、ui（显示失败） |
| StageUnlocked | 新关卡解锁 | stage_id | LevelSelect UI（更新解锁状态） |
| CampaignCompleted | 战役全部通关 | campaign_id | ui（显示通关画面，预留） |

---

## 8. RON 配置示例

### 8.1 Campaign 定义

```ron
// assets/campaigns/campaign_001.ron
(
    id: "campaign_001",
    name: "边境之旅",
    stages: [
        (id: "stage_001", level_id: "tutorial"),
    ],
)
```

### 8.2 Level 定义（已有，无变化）

```ron
// assets/maps/tutorial.ron
(
    id: "tutorial",
    name: "教学关",
    width: 10,
    height: 8,
    terrain_grid: [
        "MMMMMMMMMM",
        "MPPFPPPWPM",
        // ...
    ],
    player_units: [
        (template: "player_warrior", coord: (4, 3)),
        // ...
    ],
    enemy_units: [
        (template: "enemy_goblin", coord: (7, 5)),
        // ...
    ],
    victory_condition: Some((
        win_conditions: [(type: KillAll)],
        lose_conditions: [(type: AllDead)],
    )),
    turn_limit: Some(20),
)
```

### 8.3 CampaignProgress 运行时结构（非 RON，仅运行时）

```text
CampaignProgress {
    campaign_id: "campaign_001",
    current_stage: Some("stage_001"),
    stages: {
        "stage_001": Unlocked,  // 玩家已开始
        "stage_002": Locked,    // 尚未解锁
        "stage_003": Locked,
    },
    completed: [],              // 尚无已通关关卡
}
```

---

## 9. 与已有领域规则的一致性检查

### 与 level_rules_v1 的关系

- Campaign 不修改 LevelConfig 的任何字段
- Campaign 通过 level_id 引用 Level，不关心 Level 的内部结构
- Stage 术语与 level_rules_v1 中已定义的 Stage 保持完全一致
- LevelRegistry 仍然是 Level 的唯一注册表，Campaign 不创建新的注册表

### 与 victory_condition_rules_v1 的关系

- Campaign 不参与胜负条件判断
- Campaign 消费 victory_condition 的输出（GameOverState）
- StageCompleted/StageFailed 事件在 GameOverState 写入后触发

### 与 battle_rules_v1 的关系

- Campaign 不参与战斗流程（Effect Pipeline、CombatIntent 等）
- Battle 的运行时不感知 Campaign 的存在

### 与 turn_rules 的关系

- Campaign 不参与回合管理和状态转换
- Campaign 引用 AppState 但仅用于流程编排，不修改 TurnPhase

### 与 level_rules_v1 中内容组织的关系

- 当前建议：Level 保持"一切合一"模式，Campaign 作为外部编排层
- 未来可选：LevelConfig 内部可将 terrain_grid 进一步外部化为 MapAsset 引用
- 两者不冲突，Campaign 的 level_id 引用在任何模式下都有效

### DOMAIN CONFLICT 检查

- **无冲突**：本文档是对 level_rules_v1 的补充而非替代。Campaign 是新的顶层概念，Level 的所有已有规则保持不变。
- Stage 在 level_rules_v1 中已定义，本文档与其定义一致，不修改含义。

---

## 10. 宪法条款映射

| 宪法条款 | 本领域对应 |
|----------|-----------|
| 1.1.2 Definition/Instance 分离 | Campaign 是 Definition，CampaignProgress 是 Instance |
| 1.1.3 Rule/Content 分离 | 解锁规则是规则，Campaign 的 RON 配置是内容 |
| 1.1.4 Logic/Presentation 分离 | Campaign 只定义流程和数据，不定义 UI 表现 |
| 1.1.5 数据驱动 | Campaign 和 Level 的配置从 RON 加载 |
| 1.1.9 只解决当前复杂度 | 不设计 Chapter/Stage 层级结构，不做存档系统，Campaign 扁平 |

---

## 11. 架构违规检测

| 违规行为 | 检测方式 | 输出 |
|----------|----------|------|
| Campaign 中内嵌 Level 数据 | 代码审查 | ARCHITECTURE VIOLATION: Campaign 配置中内嵌了 Level 数据（地形/单位部署等），违反 Definition/Instance 分离原则。应通过 level_id 引用。 |
| Campaign 运行时修改 LevelConfig | 代码审查 | ARCHITECTURE VIOLATION: 运行时修改 LevelConfig，违反 Definition 不可变原则。 |
| 硬编码关卡解锁条件 | 代码审查 | ARCHITECTURE VIOLATION: 关卡解锁条件硬编码在 [文件] 中，违反 Rule/Content 分离原则。应通过 Campaign 配置表达。 |
| 跳过 LevelSelect 直接进入战斗 | 代码审查 | ARCHITECTURE VIOLATION: 从 [状态名] 直接进入 InGame，跳过 LevelSelect 状态，违反战役流程顺序。 |
| 战役进度写入持久化存储 | 代码审查 | ARCHITECTURE VIOLATION: CampaignProgress 被写入持久化存储（文件/数据库），违反"运行时不可持久化"原则。存档系统当前不应实现。 |

---

## 12. 预留的扩展点

以下内容在当前版本（v1.0）中明确不做，但预留扩展空间：

| 扩展点 | 触发条件 | 设计预留 |
|--------|----------|----------|
| Chapter / 章节层级 | 关卡数 > 10 且需要分组 | Stage 已有 id 字段，未来可在 id 上加前缀编码（如 "ch1_stage_03"） |
| Map / Battle 分离 | 同一张地图需要用于多个不同关卡 | Level 引用 map_id 替代内嵌 terrain_grid |
| 存档系统 | 玩家明确要求保存进度 | CampaignProgress 的序列化方案 |
| 支线/可选关卡 | 需要非线性战役 | 在 Stage 字段中添加 optional: bool |
| 剧情/对话系统 | 有关卡需要剧情 | 在 Stage 字段中添加 dialogue_id: Option<String> |
| 评价/评级系统 | 需要星级评价 | 在 Stage 中添加 score_conditions |

所有扩展点仅在复杂度真正达到时实施，遵循"只解决当前复杂度"原则（宪法 1.1.9）。
