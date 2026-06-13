# 胜负条件 领域规则 v1.0

Version: 1.0
Status: Draft
Applies To: 战斗胜负条件的定义、检查时机、判定逻辑、结果处理

---

## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| VictoryCondition | 关卡的胜负条件配置（Definition），描述玩家达成胜利或失败的条件 | 负责：条件描述和数据定义；不负责：条件检查的执行逻辑 |
| WinCondition | VictoryCondition 中的胜利子条件，描述玩家获胜的具体目标 | 负责：胜利目标描述；不负责：失败判定 |
| LoseCondition | VictoryCondition 中的失败子条件，描述玩家失败的具体触发 | 负责：失败触发描述；不负责：胜利判定 |
| ConditionType | 胜负条件的类型枚举，定义具体的检查方式 | 负责：类型标识和参数；不负责：检查执行 |
| VictoryCheckResult | 胜负条件检查的结果，三态枚举 | 负责：结果表达；不负责：结果处理 |
| GameOverState | 战斗的最终状态（ViewModel），展示给玩家的胜负结果 | 负责：UI 展示状态；不负责：条件检查逻辑 |
| GameOver | 游戏结束的整体概念，包含胜负结果和后续流程 | 负责：结束流程编排；不负责：单条条件的检查 |

### 术语关系

```
VictoryCondition（配置）
├── WinCondition[]（胜利条件列表）
│   └── ConditionType（具体类型 + 参数）
├── LoseCondition[]（失败条件列表）
│   └── ConditionType（具体类型 + 参数）
└── 检查时机：TurnPhase 的指定阶段

VictoryCheckResult（检查结果）
├── Victory → GameOverState.Victory
├── Defeat  → GameOverState.Defeat
└── Continue → GameOverState.Playing（继续战斗）
```

### 与已有术语的关系

| 已有术语 | 所属领域 | 与 VictoryCondition 的关系 |
|----------|----------|---------------------------|
| Unit | character_rules | 胜负条件通过检查 Unit 的 Faction 和存活状态判定 |
| Dead | character_rules | 单位死亡状态是 KillAll 等条件的检查依据 |
| Faction | character_rules | 阵营区分（Player/Enemy）是条件判定的基础 |
| TurnPhase | turn_rules | 胜负条件在指定的 TurnPhase 阶段检查 |
| TurnState | turn_rules | SurviveTurns 条件通过 turn_number 判定 |
| GameOverState | turn（业务层 Resource） | 胜负条件检查结果驱动 GameOverState 变化 |
| AppState | turn_rules | GameOverState 非 Playing 时驱动 AppState → GameOver |
| BattleRecord | battle_rules | 胜负判定结果记录到 BattleRecord |
| LevelConfig | level_rules | VictoryCondition 作为 LevelConfig 的配置字段 |

---

## 2. 状态机

### 胜负判定状态机

```
Checking
  │ [检查所有 WinCondition]
  ├─── 任一 WinCondition 满足 ──► Victory
  │
  │ [检查所有 LoseCondition]
  ├─── 任一 LoseCondition 满足 ──► Defeat
  │
  └─── 所有条件均未满足 ──► Continue

GameOverState 状态机：

Playing
  ├─── VictoryCondition 判定 Victory ──► Victory
  │
  └─── LoseCondition 判定 Defeat ──► Defeat

终态：
Victory（不可逆）
Defeat（不可逆）
```

状态列表：
- Playing：战斗进行中，胜负未定
- Checking：胜负条件正在检查（瞬时状态，每次检查时短暂进入）
- Victory：玩家胜利（终态）
- Defeat：玩家失败（终态）

转换规则：
- Playing → Checking：到达检查时机（TurnEnd 阶段）
- Checking → Victory：任一 WinCondition 满足
- Checking → Defeat：任一 LoseCondition 满足
- Checking → Playing：所有条件均未满足，继续战斗
- Playing → Victory：直接判定胜利（如 WinCondition 即时满足）
- Playing → Defeat：直接判定失败（如 LoseCondition 即时满足）
- 禁止：Victory → Playing（胜利不可逆）
- 禁止：Defeat → Playing（失败不可逆）
- 禁止：Victory → Defeat（胜负状态不可互相转换）
- 禁止：Defeat → Victory（胜负状态不可互相转换）

---

## 3. 不变量（Invariants）

### 3.1 全灭玩家即失败 🟥

- 条件：每次胜负条件检查时
- 不变量：战场上不存在存活的 Player 阵营单位时，必须判定为 Defeat
- 违反后果：玩家全灭但游戏未结束，无法继续操作也无法退出
- 备注：此为绝对不变量，无论 VictoryCondition 如何配置，全灭即失败

### 3.2 胜负互斥 🟥

- 条件：每次胜负条件检查时
- 不变量：Victory 和 Defeat 不能同时成立。若同时满足胜利和失败条件，优先判定 Defeat
- 违反后果：胜负状态矛盾，UI 显示异常
- 备注：失败优先是因为"玩家全灭"这一绝对条件必须优先于任何胜利条件

### 3.3 终态不可逆 🟥

- 条件：GameOverState 变为 Victory 或 Defeat 后
- 不变量：GameOverState 不可再变回 Playing 或切换为另一种终态
- 违反后果：胜负结果被翻转，玩家体验严重受损

### 3.4 检查时机一致性 🟥

- 条件：胜负条件检查
- 不变量：所有胜负条件必须在 TurnEnd 阶段统一检查，禁止在其他阶段零散检查
- 违反后果：胜负判定遗漏或重复，某些条件可能被跳过
- 宪法依据：状态机只负责流程控制，不包含业务细节

### 3.5 条件检查不修改游戏状态 🟥

- 条件：胜负条件检查过程中
- 不变量：检查逻辑只能读取游戏状态，不能修改任何单位属性、Buff、地形等
- 违反后果：胜负检查产生副作用，影响战斗逻辑

### 3.6 胜负条件必须有配置 🟥

- 条件：关卡加载验证时
- 不变量：每个关卡必须配置至少一个 WinCondition 和一个 LoseCondition
- 违反后果：战斗无法结束，玩家永远处于 Playing 状态

### 3.7 GameOverState 与 AppState 同步 🟩

- 条件：GameOverState 变为 Victory 或 Defeat 后
- 不变量：AppState 必须切换到 GameOver
- 违反后果：游戏逻辑仍在运行，但胜负已定

### 3.8 默认失败条件不可移除 🟥

- 条件：任何关卡配置
- 不变量："全灭玩家 = Defeat"作为默认失败条件，不可被关卡配置覆盖或移除
- 违反后果：玩家全灭后游戏不结束

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：胜负条件硬编码在代码中 — 理由：胜负条件属于关卡配置内容，必须数据驱动（宪法 1.1.3 Rule/Content 分离）
- 🟥 禁止：跳过全灭检查 — 理由：全灭玩家即失败是绝对不变量
- 🟥 禁止：胜负检查修改游戏状态 — 理由：检查是只读操作，修改状态破坏副作用透明性
- 🟥 禁止：在非 TurnEnd 阶段检查胜负 — 理由：统一检查时机，避免遗漏和重复
- 🟥 禁止：终态可逆 — 理由：Victory/Defeat 是不可逆的终态
- 🟥 禁止：胜负同时成立时判定为 Victory — 理由：失败优先原则
- 🟥 禁止：绕过 GameOverState 直接切换 AppState 到 GameOver — 理由：GameOverState 是 ViewModel 层的唯一真相源
- 🟥 禁止：关卡配置中移除默认失败条件 — 理由：全灭失败是不可移除的保底机制
- 🟥 禁止：新增胜负条件类型时修改检查流程代码 — 理由：应通过 ConditionType 扩展，新类型自动参与检查

---

## 5. 流程定义

### 5.1 胜负条件检查（统一流程）

- 输入：当前 TurnPhase = TurnEnd + VictoryCondition 配置 + 战场状态（所有 Unit、TurnState）
- 处理：
  1. 检查所有 LoseCondition（失败优先）
     - 遍历 LoseCondition 列表，逐一检查
     - 任一 LoseCondition 满足 → 立即返回 Defeat
  2. 检查默认失败条件（全灭玩家）
     - 查询所有存活的 Player 阵营单位
     - 存活数 = 0 → 返回 Defeat
  3. 检查所有 WinCondition
     - 遍历 WinCondition 列表，逐一检查
     - 任一 WinCondition 满足 → 返回 Victory
  4. 检查 turn_limit（如配置）
     - TurnState.turn_number > turn_limit → 根据配置判定（超时失败或超时胜利）
  5. 所有条件均未满足 → 返回 Continue
- 输出：VictoryCheckResult（Victory / Defeat / Continue）
- 失败处理：检查过程出错时返回 Continue，记录错误日志

### 5.2 KillAll 条件检查

- 输入：战场上所有 Unit 的 Faction 和存活状态
- 处理：
  1. 查询所有 Enemy 阵营单位
  2. 过滤掉已死亡的（HP <= 0 或拥有 Dead 组件）
  3. 存活 Enemy 数量 = 0 → 条件满足
- 输出：true（满足）/ false（未满足）
- 失败处理：查询出错时返回 false

### 5.3 SurviveTurns(n) 条件检查

- 输入：TurnState.turn_number + 目标回合数 n
- 处理：
  1. 读取当前 turn_number
  2. turn_number >= n → 条件满足
- 输出：true（满足）/ false（未满足）
- 失败处理：turn_number 读取失败时返回 false

### 5.4 DefeatBoss(boss_id) 条件检查

- 输入：boss_id + 战场上所有 Unit 的 UnitId 和存活状态
- 处理：
  1. 查询 UnitId = boss_id 的单位
  2. 该单位不存在 → 条件满足（Boss 已被消灭）
  3. 该单位存在但 HP <= 0 或拥有 Dead 组件 → 条件满足
  4. 该单位存在且存活 → 条件未满足
- 输出：true（满足）/ false（未满足）
- 失败处理：boss_id 对应的单位在战场上不存在时视为条件满足

### 5.5 胜负结果处理

- 输入：VictoryCheckResult
- 处理：
  1. 如果结果 = Continue：不做任何操作，等待下一次检查
  2. 如果结果 = Victory：
     a. 更新 GameOverState 为 Victory
     b. 发送 LevelCompleted Message（携带 level_id + Victory）
     c. 驱动 AppState 切换到 GameOver
  3. 如果结果 = Defeat：
     a. 更新 GameOverState 为 Defeat
     b. 发送 LevelCompleted Message（携带 level_id + Defeat）
     c. 驱动 AppState 切换到 GameOver
- 输出：GameOverState 更新 + AppState 切换 + LevelCompleted Message
- 失败处理：Message 发送失败时记录错误日志

---

## 6. 胜负条件类型定义

### 6.1 胜利条件类型

| 类型 | 参数 | 含义 | 检查逻辑 |
|------|------|------|----------|
| KillAll | 无 | 消灭所有敌方单位 | 战场上无存活的 Enemy 阵营单位 |
| SurviveTurns | n: u32 | 存活 N 回合 | 当前 turn_number >= n |
| DefeatBoss | boss_id: String | 击败指定 Boss | boss_id 对应的单位已死亡或不存在 |

### 6.2 失败条件类型

| 类型 | 参数 | 含义 | 检查逻辑 |
|------|------|------|----------|
| AllDead | 无 | 全灭（默认条件，不可移除） | 战场上无存活的 Player 阵营单位 |
| TurnLimitExceeded | max_turns: u32 | 超时失败 | 当前 turn_number > max_turns |

### 6.3 扩展性

新增条件类型时：
- 允许：新增 ConditionType 变体
- 允许：在 RON 配置中使用新类型
- 禁止：修改检查流程的调度逻辑
- 禁止：修改"失败优先"原则

### 6.4 多条件组合规则

**当前版本**：
- 胜利条件之间为 OR 关系（任一满足即胜利）
- 失败条件之间为 OR 关系（任一满足即失败）
- 失败检查优先于胜利检查

**未来扩展（预留）**：
- AND 组合：所有胜利条件都满足才判定胜利
- 条件链：条件 A 满足后才检查条件 B
- 备注：上述扩展仅在复杂度真正需要时引入，遵循"只解决当前复杂度"原则（宪法 1.1.9）

---

## 7. 与 TurnPhase 的衔接

### 7.1 检查时机

胜负条件检查在 **TurnEnd 阶段**执行。

理由：
- TurnEnd 是每回合的最终阶段，所有行动已完成
- 在此阶段检查可以获取到最完整的战场状态
- 避免在 ExecuteAction 等中间阶段检查导致的状态不一致

### 7.2 衔接流程

```
TurnPhase::TurnEnd
  │
  ▼
胜负条件检查系统执行
  │
  ├── Continue → TurnPhase 回到 SelectUnit（下一回合）
  │
  ├── Victory → GameOverState = Victory
  │              → AppState = GameOver
  │
  └── Defeat  → GameOverState = Defeat
                 → AppState = GameOver
```

### 7.3 与当前实现的关系

当前代码中 `update_game_over_state()` 硬编码了"全灭敌人=胜利，全灭玩家=失败"逻辑。

本规则要求：
- 该逻辑从硬编码改为读取关卡配置的 VictoryCondition
- GameOverState 的三态（Playing / Victory / Defeat）保持不变
- 检查时机从"每帧轮询"改为"TurnEnd 阶段统一检查"

**DOMAIN CONFLICT 标注**：
- 当前 `update_game_over_state()` 位于 `src/ui/view_models.rs`（UI ViewModel 层）
- 按架构规则，UI 层不应包含业务逻辑（宪法 1.1.4 Logic/Presentation 分离）
- 胜负条件检查属于业务逻辑，应从 UI 层移出
- 建议：胜负条件检查系统归属 turn 领域或独立的 victory 领域，检查结果写入 GameOverState（ViewModel），UI 层只读取 GameOverState

---

## 8. RON 配置示例

### 8.1 KillAll 类型（当前教学关等效）

```ron
victory_condition: (
    win_conditions: [
        (type: "KillAll"),
    ],
    lose_conditions: [
        (type: "AllDead"),
    ],
)
```

### 8.2 SurviveTurns 类型

```ron
victory_condition: (
    win_conditions: [
        (type: "SurviveTurns", params: (n: 10)),
    ],
    lose_conditions: [
        (type: "AllDead"),
    ],
)
```

### 8.3 DefeatBoss 类型

```ron
victory_condition: (
    win_conditions: [
        (type: "DefeatBoss", params: (boss_id: "dark_lord")),
    ],
    lose_conditions: [
        (type: "AllDead"),
        (type: "TurnLimitExceeded", params: (max_turns: 20)),
    ],
)
```

### 8.4 多条件组合

```ron
victory_condition: (
    win_conditions: [
        (type: "KillAll"),
        (type: "SurviveTurns", params: (n: 15)),
    ],
    lose_conditions: [
        (type: "AllDead"),
    ],
    turn_limit: 20,
)
```

含义：全灭敌人或存活 15 回合均可胜利，全灭或超时 20 回合均失败。

---

## 9. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| VictoryConditionMet | 胜利条件满足 | level_id, 满足的条件类型, turn_number | ui（显示胜利界面）、battle_record（记录结果） |
| LoseConditionMet | 失败条件满足 | level_id, 满足的条件类型, turn_number | ui（显示失败界面）、battle_record（记录结果） |
| LevelCompleted | 关卡结束（Victory 或 Defeat） | level_id, 结果（Victory/Defeat） | ui（结算界面）、battle_record（最终记录） |
| TurnLimitWarning | 接近回合上限 | level_id, 剩余回合数 | ui（显示警告提示） |

---

## 10. 与已有领域规则的一致性检查

### 与 turn_rules 的关系

- 胜负条件检查在 TurnEnd 阶段执行，不影响 TurnPhase 状态机的转换逻辑
- TurnState.turn_number 是 SurviveTurns 和 TurnLimitExceeded 的检查依据
- 胜负结果驱动 AppState 从 InGame 切换到 GameOver

### 与 battle_rules_v1 的关系

- 胜负条件检查只读取 Unit 的 Faction 和存活状态，不参与 Effect Pipeline
- CharacterDied Message 可辅助判断单位死亡（但最终以 HP 和 Dead 组件为准）
- BattleRecord 记录胜负判定结果

### 与 level_rules_v1 的关系

- VictoryCondition 作为 LevelConfig 的配置字段
- 关卡验证时检查 VictoryCondition 的有效性
- LevelConfig 加载后，VictoryCondition 不可修改

### 与 map_rules_v1 的关系

- 胜负条件不依赖地形数据
- 无冲突

### DOMAIN CONFLICT 检查

- **潜在冲突**：当前 `update_game_over_state()` 在 UI 层（view_models.rs）硬编码了胜负逻辑
- **建议**：胜负条件检查逻辑应从 UI 层移出到业务层，GameOverState 保持为 ViewModel（只被读取，不被 UI 层写入）
- **影响范围**：需要重构 `update_game_over_state()` 的职责归属
- **不阻塞**：此冲突不影响本领域规则的完整性，实现时由架构设计解决

---

## 11. 宪法条款映射

| 宪法条款 | 本领域对应 |
|----------|-----------|
| 1.1.3 Rule/Content 分离 | 胜负检查是规则，条件配置是内容（RON） |
| 1.1.4 Logic/Presentation 分离 | 胜负检查是业务逻辑，GameOverState 是 ViewModel |
| 1.1.5 数据驱动 | 胜负条件从关卡 RON 配置加载 |
| 1.1.9 只解决当前复杂度 | 多条件组合标注为未来扩展，当前只实现 OR 关系 |
| 5.0 通信三原则 | LevelCompleted Message 跨模块广播 |
| 7.0 状态机职责 | 状态机只负责流程控制，胜负检查在 TurnEnd 阶段执行 |

---

## 12. 架构违规检测

| 违规行为 | 检测方式 | 输出 |
|----------|----------|------|
| 胜负条件硬编码 | 代码审查 | ARCHITECTURE VIOLATION: 胜负条件硬编码在 [文件] 中，违反 Rule/Content 分离原则。应通过关卡配置的 victory_condition 字段实现。 |
| 胜负检查在非 TurnEnd 阶段执行 | 代码审查 | ARCHITECTURE VIOLATION: 胜负检查在 [阶段名] 执行，违反统一检查时机原则。应在 TurnEnd 阶段统一检查。 |
| 胜负检查修改游戏状态 | 代码审查 | ARCHITECTURE VIOLATION: 胜负检查过程中修改了游戏状态，违反只读检查原则。 |
| 终态被逆转 | 代码审查 | ARCHITECTURE VIOLATION: GameOverState 从 [终态] 变为 [新状态]，违反终态不可逆原则。 |
| 绕过 GameOverState 直接切换 AppState | 代码审查 | ARCHITECTURE VIOLATION: 绕过 GameOverState 直接切换 AppState 到 GameOver，违反 ViewModel 唯一真相源原则。 |
