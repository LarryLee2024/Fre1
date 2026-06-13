# ADR: 关卡配置与胜负条件系统

## 状态

Proposed

---

## 背景

项目已完成完整战斗闭环，但关卡配置和胜负判定存在架构缺陷：

1. **数据缺失**：`assets/maps/tutorial.ron` 缺少 `victory_condition`、`turn_limit`、`rewards` 字段，`LevelConfigDef` 和 `LevelConfig` 均无对应类型定义
2. **硬编码胜负逻辑**：`src/ui/view_models.rs` 的 `update_game_over_state()` 硬编码了"全灭敌人=胜利，全灭玩家=失败"，每帧轮询执行
3. **违反 Logic/Presentation 分离**：胜负判定是业务逻辑，但 `update_game_over_state()` 位于 UI ViewModel 层
4. **违反 Rule/Content 分离**：胜负条件应从 RON 配置读取，不应写死在代码中
5. **潜在 Bug**：当前胜负检查使用 `Query<&Unit>` 未排除 `Dead` 组件单位，死亡单位仍被计入存活判断

domain-designer 已产出两个领域规则文档，本 ADR 基于这两个文档设计实现架构。

---

## 引用的领域规则

- **docs/domain/level_rules_v1.md** — 关卡配置领域规则
  - Level = Map + UnitDeployDef[] + VictoryCondition + TurnLimit? + Reward[]?
  - LevelConfig 是 Definition，运行时不可变
  - 关卡必须有胜利条件（不变量 3.9）
  - LevelRegistry 空即为空（不变量 3.10）
  - 关卡生命周期：Loading → Validated → Spawned → Active → Completed

- **docs/domain/victory_condition_rules_v1.md** — 胜负条件领域规则
  - VictoryCondition = WinCondition[] + LoseCondition[]
  - 条件类型：KillAll / SurviveTurns(n) / DefeatBoss(boss_id) / AllDead / TurnLimitExceeded
  - 全灭玩家即失败是绝对不变量（3.1），不可被关卡配置移除
  - 失败优先原则（3.2）：胜负同时成立时判定 Defeat
  - 终态不可逆（3.3）：Victory/Defeat 不可变回 Playing
  - 检查时机一致性（3.4）：所有条件在 TurnEnd 阶段统一检查
  - 条件检查不修改游戏状态（3.5）：只读操作
  - 胜利条件之间 OR 关系，失败条件之间 OR 关系

---

## 决策

### 决策 1：扩展 LevelConfigDef / LevelConfig 类型

在 `src/map/data.rs` 的现有 `LevelConfigDef` 和 `LevelConfig` 上新增字段：

- `victory_condition: VictoryConditionDef` — 胜利条件配置（必要字段）
- `turn_limit: Option<u32>` — 可选回合上限

新增相关类型定义（`VictoryConditionDef`、`WinConditionDef`、`LoseConditionDef`、`ConditionParamsDef`）均在 `src/map/data.rs` 中，遵循项目双类型模式（Def 用于 RON 反序列化）。

`LevelConfig`（运行时）存储从 Def 转换后的不可变数据。由于 `VictoryConditionDef` 不包含字符串标签引用（无需 GameplayTag 转换），运行时直接复用 `VictoryConditionDef` 类型。

### 决策 2：GameOverState 从 UI 层迁移到 Turn 模块

**核心架构变更**：将 `GameOverState` 的定义从 `src/ui/view_models.rs`（UI 层）迁移到 `src/turn/state.rs`（业务层）。

理由：
- `GameOverState` 是战斗流程的终态标识（Playing/Victory/Defeat），本质是游戏流程状态，不是 UI 展示数据
- `AppState`（MainMenu/InGame/GameOver）已在 `src/turn/state.rs`，`GameOverState` 是其细化状态
- 胜负条件检查系统需要**写入** GameOverState，业务逻辑不应写入 UI 层资源
- 迁移后 UI 层**只读取** GameOverState，符合 Logic/Presentation 分离

迁移影响：
- `src/ui/view_models.rs`：删除 `GameOverState` 定义
- `src/ui/panels/turn_indicator.rs`：改为 `use crate::turn::GameOverState`
- `src/battle/record.rs`（如引用）：改为 `use crate::turn::GameOverState`
- TurnPlugin 负责 `init_resource::<GameOverState>()` 和 Reflect 注册

### 决策 3：胜负检查系统归属 Turn 模块

新增 `src/turn/victory_check.rs`，包含胜负条件检查逻辑。

理由：
- 领域规则明确指出"在 TurnEnd 阶段统一检查"
- 系统读取 `TurnState`（回合号）和 `TurnPhase`（阶段）
- 系统注册在 `OnEnter(TurnEnd)` 调度点，与 `turn_end_on_enter` 同一调度点
- Turn 模块已管理 `AppState`、`TurnPhase`、`TurnState`，胜负检查是流程的自然延伸

系统排序约束：
- `check_victory_conditions` 在 `turn_end_on_enter` **之前**运行（`.before()`）
- 原因：胜负检查读取当前回合号（turn_number = 本回合），检查完成后再由 `turn_end_on_enter` 递增回合号

### 决策 4：胜负检查分两层执行

| 层级 | 执行时机 | 检查内容 | 理由 |
|------|----------|----------|------|
| 响应式 | OnEnter(TurnEnd) | 所有配置条件（KillAll、SurviveTurns、DefeatBoss、TurnLimitExceeded）+ 默认全灭检查 | 统一检查时机，数据驱动 |
| 兜底 | Update（每帧，有 early return） | 仅检查"全灭玩家即失败"绝对不变量 | 防止极端情况（如 OnEnter 被跳过） |

兜底检查是防御性编程，确保"全灭玩家即失败"这一绝对不变量在任何情况下都不会被遗漏。正常流程中，OnEnter(TurnEnd) 的响应式检查会先捕获所有条件。

### 决策 5：删除 UI 层的硬编码胜负逻辑

- 删除 `update_game_over_state()` 函数（`src/ui/view_models.rs`）
- 删除该系统在 `src/ui/mod.rs` 的注册
- 删除该系统在 UI 层的所有胜负判定逻辑

替代方案：胜负判定完全由 `src/turn/victory_check.rs` 的 `check_victory_conditions` 系统负责。

---

## Module Design

### 修改的模块

```
src/map/data.rs                ← 新增 VictoryConditionDef 等类型定义，扩展 LevelConfigDef/LevelConfig
src/turn/state.rs              ← 新增 GameOverState（从 UI 层迁入）
src/turn/victory_check.rs      ← 新增胜负条件检查系统
src/turn/mod.rs                ← 注册新系统和 GameOverState 资源
src/turn/order.rs              ← turn_end_on_enter 添加 .after() 排序约束
src/ui/view_models.rs          ← 删除 GameOverState 定义，删除 update_game_over_state()
src/ui/mod.rs                  ← 删除 update_game_over_state 系统注册
src/ui/panels/turn_indicator.rs ← 修改 GameOverState 导入路径
assets/maps/tutorial.ron       ← 新增 victory_condition 字段
```

### 新增类型

| 类型 | 文件 | 性质 | 职责 |
|------|------|------|------|
| `VictoryConditionDef` | `src/map/data.rs` | Definition | 关卡的完整胜负条件配置 |
| `WinConditionDef` | `src/map/data.rs` | Definition | 单条胜利条件（type + params） |
| `LoseConditionDef` | `src/map/data.rs` | Definition | 单条失败条件（type + params） |
| `ConditionParamsDef` | `src/map/data.rs` | Definition | 条件参数（n, boss_id, max_turns） |
| `GameOverState` | `src/turn/state.rs` | Resource | 胜负状态枚举（从 UI 层迁入） |

### 新增 System

| System | 文件 | 调度 | 职责 |
|--------|------|------|------|
| `check_victory_conditions` | `src/turn/victory_check.rs` | `OnEnter(TurnEnd).before(turn_end_on_enter)` | 读取 VictoryConditionDef + 战场状态，写入 GameOverState |
| `check_all_dead_safety` | `src/turn/victory_check.rs` | `Update`（run_if: InGame && Playing） | 兜底：全灭玩家即失败 |

### 文件组织

```
src/turn/
├── mod.rs              // 回合管理模块：状态机、行动队列、胜负检查、SystemSet 编排
├── state.rs            // AppState, TurnPhase, GameOverState, GameSet
├── order.rs            // TurnOrder 行动队列、TurnState、TurnStarted/TurnEnded Message
└── victory_check.rs    // 胜负条件检查系统（数据驱动，读取 LevelConfig.VictoryConditionDef）
```

---

## Communication Design

### Message: 跨功能通信

| Message | 发送方 | 接收方 | 触发时机 |
|---------|--------|--------|----------|
| `LevelCompleted`（新增） | `turn/victory_check` | `battle/record`、`ui` | GameOverState 变为 Victory 或 Defeat 时 |

`LevelCompleted` Message 定义：
- 携带数据：`level_id: String`, `result: GameOverState`, `turn_number: u32`
- 发送方：`check_victory_conditions` 系统
- 接收方：`battle/record`（记录战斗结果）、`ui`（显示结算界面）

注册位置：`TurnPlugin::build()` 中 `app.add_message::<LevelCompleted>()`

### Observer: 同功能状态变化响应

不涉及。胜负检查是跨模块读取的集中式检查，不属于单一 Feature 内的局部响应。

### Hook: 组件添加/删除的副作用

不涉及。胜负条件检查不依赖组件 Hook 触发。

### 函数调用: 模块内直接调用

- `check_victory_conditions` 系统内部：
  - `check_kill_all()` — 检查 KillAll 条件
  - `check_survive_turns()` — 检查 SurviveTurns 条件
  - `check_defeat_boss()` — 检查 DefeatBoss 条件
  - `check_all_dead()` — 检查默认全灭失败
  - `check_turn_limit_exceeded()` — 检查超时失败

均为 `victory_check.rs` 内部私有函数，不暴露到模块外。

---

## 边界定义

### 允许

- `turn/victory_check` 读取 `map::LevelRegistry`（获取 VictoryConditionDef）
- `turn/victory_check` 读取 `character::Unit`、`Faction`、`Dead`（检查单位存活状态）
- `turn/victory_check` 读取 `turn::TurnState`（获取 turn_number）
- `turn/victory_check` 写入 `turn::GameOverState`（更新胜负结果）
- `turn/victory_check` 发送 `LevelCompleted` Message
- `ui` 读取 `turn::GameOverState`（展示胜负界面）
- `ui` 读取 `LevelCompleted` Message（更新结算 UI）
- `battle/record` 读取 `LevelCompleted` Message（记录结果）

### 禁止

- `turn/victory_check` 修改任何单位属性（HP、Buff、位置等） — 检查是只读操作
- `turn/victory_check` 直接操作 UI 组件 — 通过 Message 通知
- `ui` 写入 `GameOverState` — UI 只读
- `ui` 包含任何胜负判定逻辑 — 胜负判定是业务逻辑
- 任何其他模块绕过 `GameOverState` 直接切换 `AppState::GameOver`

---

## Forbidden（禁止事项）

- **FORBIDDEN-1** — 🟥 禁止：胜负条件硬编码在代码中 — 理由：必须从 RON 配置的 `victory_condition` 字段读取（Rule/Content 分离）
- **FORBIDDEN-2** — 🟥 禁止：在 UI 层包含胜负判定逻辑 — 理由：胜负判定是业务逻辑，属于 Turn 模块（Logic/Presentation 分离）
- **FORBIDDEN-3** — 🟥 禁止：胜负检查过程中修改游戏状态（单位 HP、Buff、地形等） — 理由：检查是只读操作（不变量 3.5）
- **FORBIDDEN-4** — 🟥 禁止：在 TurnEnd 以外的阶段执行胜负条件检查（兜底的全灭检查除外） — 理由：统一检查时机（不变量 3.4）
- **FORBIDDEN-5** — 🟥 禁止：终态可逆（Victory/Defeat 变回 Playing 或互相转换） — 理由：终态不可逆（不变量 3.3）
- **FORBIDDEN-6** — 🟥 禁止：胜负同时成立时判定为 Victory — 理由：失败优先原则（不变量 3.2）
- **FORBIDDEN-7** — 🟥 禁止：绕过 GameOverState 直接切换 AppState 到 GameOver — 理由：GameOverState 是唯一真相源
- **FORBIDDEN-8** — 🟥 禁止：关卡配置中移除默认失败条件（全灭玩家即失败） — 理由：绝对不变量（不变量 3.1/3.8）
- **FORBIDDEN-9** — 🟥 禁止：新增胜负条件类型时修改检查调度流程 — 理由：新增类型只需在 match 分支中添加检查函数，不修改调度逻辑
- **FORBIDDEN-10** — 🟥 禁止：运行时修改 LevelConfig 中的 VictoryConditionDef — 理由：Definition 不可变
- **FORBIDDEN-11** — 🟥 禁止：胜负检查使用 `Query<&Unit>` 不排除 `Dead` 组件 — 理由：死亡单位不应计入存活判断

---

## Definition / Instance Design

### Definition（不可变配置）

| 类型 | 存储位置 | 加载来源 | 不可变性保证 |
|------|----------|----------|-------------|
| `VictoryConditionDef` | `LevelConfig.victory_condition` | `assets/maps/*.ron` | 作为 `LevelConfig` 的嵌套字段，随 `LevelConfig` 一起不可变 |
| `WinConditionDef` | `VictoryConditionDef.win_conditions` | 同上 | 同上 |
| `LoseConditionDef` | `VictoryConditionDef.lose_conditions` | 同上 | 同上 |
| `ConditionParamsDef` | 嵌套在各 ConditionDef 中 | 同上 | 同上 |

### Instance（运行时状态）

| 类型 | 存储方式 | 写入方 | 读取方 |
|------|----------|--------|--------|
| `GameOverState` | `Resource`（TurnPlugin 注册） | `check_victory_conditions` 系统 | UI 层（`turn_indicator`、结算界面） |

`GameOverState` 不需要额外的运行时中间状态。`check_victory_conditions` 系统直接读取 Definition + 战场 ECS 数据，计算后写入 `GameOverState`，无中间 Component 或 Resource。

---

## RON 格式设计

### 完整教程关配置（更新后 tutorial.ron）

```ron
(
    id: "tutorial",
    name: "教学关",
    width: 10,
    height: 8,
    terrain_grid: [
        "MMMMMMMMMM",
        "MPPFPPPWPM",
        "MPPPPPPPPM",
        "MPPWPPPFPM",
        "MPPPPPPPPM",
        "MPFPPPPWPM",
        "MPPPPPFPPM",
        "MMMMMMMMMM",
    ],
    player_units: [
        (template: "player_warrior", coord: (4, 3)),
        (template: "player_archer", coord: (3, 4)),
        (template: "player_mage", coord: (2, 5)),
    ],
    enemy_units: [
        (template: "enemy_goblin", coord: (7, 5)),
        (template: "enemy_goblin", coord: (8, 3)),
        (template: "enemy_dark_knight", coord: (6, 6)),
    ],
    victory_condition: (
        win_conditions: [
            (type: "KillAll"),
        ],
        lose_conditions: [
            (type: "AllDead"),
        ],
    ),
)
```

### 存活 N 回合关卡

```ron
(
    id: "survival_10",
    name: "坚守 10 回合",
    width: 8,
    height: 8,
    terrain_grid: [ /* ... */ ],
    player_units: [ /* ... */ ],
    enemy_units: [ /* ... */ ],
    turn_limit: 15,
    victory_condition: (
        win_conditions: [
            (type: "SurviveTurns", params: (n: 10)),
        ],
        lose_conditions: [
            (type: "AllDead"),
            (type: "TurnLimitExceeded", params: (max_turns: 15)),
        ],
    ),
)
```

### Boss 战关卡

```ron
(
    id: "boss_fight",
    name: "暗黑领主之战",
    width: 12,
    height: 10,
    terrain_grid: [ /* ... */ ],
    player_units: [ /* ... */ ],
    enemy_units: [
        (template: "dark_lord", coord: (10, 5)),
        (template: "enemy_goblin", coord: (8, 3)),
        (template: "enemy_goblin", coord: (8, 7)),
    ],
    turn_limit: 20,
    victory_condition: (
        win_conditions: [
            (type: "DefeatBoss", params: (boss_id: "dark_lord")),
        ],
        lose_conditions: [
            (type: "AllDead"),
            (type: "TurnLimitExceeded", params: (max_turns: 20)),
        ],
    ),
)
```

### 多条件组合关卡

```ron
victory_condition: (
    win_conditions: [
        (type: "KillAll"),
        (type: "SurviveTurns", params: (n: 15)),
    ],
    lose_conditions: [
        (type: "AllDead"),
    ],
),
turn_limit: 20,
```

含义：全灭敌人**或**存活 15 回合均可胜利；全灭**或**超时 20 回合均失败。

---

## 迁移计划

### Phase 1：类型定义与配置扩展

1. 在 `src/map/data.rs` 新增 `VictoryConditionDef`、`WinConditionDef`、`LoseConditionDef`、`ConditionParamsDef` 类型
2. 在 `LevelConfigDef` 新增 `victory_condition: VictoryConditionDef` 和 `turn_limit: Option<u32>` 字段（`#[serde(default)]` 保证向后兼容）
3. 在 `LevelConfig` 新增对应字段
4. 更新 `LevelConfig::from_def()` 传递新字段
5. 更新 `assets/maps/tutorial.ron` 添加 `victory_condition` 字段
6. 更新现有测试中的 `LevelConfigDef` 构造

### Phase 2：GameOverState 迁移

1. 将 `GameOverState` 枚举从 `src/ui/view_models.rs` 迁移到 `src/turn/state.rs`
2. 在 `src/turn/mod.rs` 的 TurnPlugin 中注册 `init_resource::<GameOverState>()` 和 `register_type::<GameOverState>()`
3. 更新所有引用 `GameOverState` 的文件导入路径：
   - `src/ui/view_models.rs`：删除定义
   - `src/ui/panels/turn_indicator.rs`：`use crate::turn::GameOverState`
   - `src/ui/mod.rs`：删除 GameOverState 的 re-export（如有）
   - `src/battle/record.rs`：`use crate::turn::GameOverState`（如有引用）

### Phase 3：胜负检查系统实现

1. 新建 `src/turn/victory_check.rs`
2. 实现 `check_victory_conditions` 系统（OnEnter(TurnEnd) 调度）
3. 实现 `check_all_dead_safety` 兜底系统（Update 调度，仅检查全灭失败）
4. 实现内部检查函数：`check_kill_all`、`check_survive_turns`、`check_defeat_boss`、`check_all_dead`、`check_turn_limit_exceeded`
5. 定义 `LevelCompleted` Message 并在 TurnPlugin 注册
6. 在 `src/turn/mod.rs` 注册两个新系统，配置排序约束
7. 修改 `turn_end_on_enter` 添加 `.after(check_victory_conditions)` 排序约束

### Phase 4：清理旧逻辑

1. 删除 `update_game_over_state()` 函数（`src/ui/view_models.rs`）
2. 删除该系统在 `src/ui/mod.rs` 的注册
3. 更新 `src/ui/view_models.rs` 的测试（删除 `update_game_over_state` 相关测试）
4. 验证 `src/ui/panels/turn_indicator.rs` 的 `check_game_over()` 仍正常工作（读取路径变更）

### Phase 5：验证与回归

1. 运行全量测试确保无回归
2. 确认 `cargo build` 无编译错误
3. 确认教学关加载、战斗、胜负判定流程正常

---

## 后果

### 正面

- **数据驱动**：新关卡只需新增 RON 文件，无需修改代码
- **架构合规**：胜负判定从 UI 层迁移到业务层，符合 Logic/Presentation 分离
- **可扩展**：新增条件类型只需添加 ConditionType 变体和对应检查函数
- **Bug 修复**：新系统正确排除 Dead 单位，修复当前存活判断 Bug
- **性能优化**：从每帧轮询改为 TurnEnd 阶段集中检查（兜底系统有 early return）
- **可测试性**：胜负检查系统是纯逻辑，可独立进行 ECS 集成测试
- **安全性**：失败优先原则和终态不可逆通过代码逻辑强制保证

### 负面

- **迁移成本**：需要修改约 6 个文件，涉及类型迁移和导入路径变更
- **向后兼容**：`LevelConfigDef` 新增字段需要 `#[serde(default)]`，旧 RON 文件缺少 `victory_condition` 时使用默认值（KillAll），可能导致关卡验证不够严格
- **排序依赖**：`check_victory_conditions` 必须在 `turn_end_on_enter` 之前运行，增加系统排序复杂度
- **兜底系统**：每帧运行兜底检查有轻微性能开销（虽然有 early return，但 `is_changed()` 检查仍有成本）

---

## 替代方案

### 替代方案 A：胜负检查放在 Battle 模块

将 `victory_check.rs` 放在 `src/battle/victory.rs`。

**放弃理由**：
- 胜负检查依赖 `TurnPhase`（TurnEnd）和 `TurnState`（turn_number），与 Turn 模块耦合更紧密
- 领域规则明确指出"在 TurnEnd 阶段执行"，归属 Turn 更自然
- Battle 模块已职责过大（Pipeline + Record + Log + Events），不宜继续膨胀

### 替代方案 B：GameOverState 保留在 UI 层

不迁移 `GameOverState`，让业务系统写入 UI 层的 Resource。

**放弃理由**：
- 业务逻辑写入 UI 层资源违反 Logic/Presentation 分离原则
- `GameOverState` 是流程状态（Playing/Victory/Defeat），不是展示数据
- `AppState` 已在 Turn 模块，`GameOverState` 作为其细化状态应同处

### 替代方案 C：纯事件驱动（无兜底系统）

仅依赖 `OnEnter(TurnEnd)` 检查，不设兜底。

**放弃理由**：
- "全灭玩家即失败"是绝对不变量，必须有防御性保障
- 如果 `OnEnter(TurnEnd)` 因 Bug 未被触发（如状态机跳转异常），全灭后游戏无法结束
- 兜底系统成本极低（early return），防御价值远大于性能开销

### 替代方案 D：VictoryConditionDef 使用 Rust 枚举序列化

使用 `#[serde(tag = "type")]` 实现 RON 枚举反序列化。

**放弃理由**：
- RON 的 serde tagged enum 支持有限，params 可选字段处理复杂
- 扁平结构（type + optional params）更简单，与项目其他 Def 类型风格一致
- 扁平结构对 RON 编写者更友好，参数缺失时默认处理更清晰

---

## 架构合规性自检

- [x] 符合 ECS 约束（Entity=ID, Component=数据, System=行为） — 胜负检查是 System，GameOverState 是 Resource（数据）
- [x] 符合 Plugin 注册顺序（Core → Data → Logic → Presentation） — GameOverState 在 TurnPlugin（Logic 层）注册，UI 层只读取
- [x] 没有创建禁止的模块（components.rs/systems.rs/utils.rs） — 新增 `victory_check.rs` 按业务命名
- [x] Effect/Modifier Pipeline 没有被绕过 — 胜负检查不涉及效果管线
- [x] Tag Components 优先于 bool 字段 — `Dead` 已是 Tag Component，检查使用 `Without<Dead>`
- [x] 符合"定义与实例分离"原则 — VictoryConditionDef 是 Definition（不可变），GameOverState 是 Instance（运行时可变）
- [x] 符合"规则与内容分离"原则 — 新关卡 = 新 RON 文件，检查逻辑代码不变
- [x] 符合"逻辑与表现分离" — 胜负判定在 Turn 模块（Logic），UI 只读 GameOverState
- [x] 所有禁止事项已明确列出（FORBIDDEN-1 到 FORBIDDEN-11）
- [x] 已检查 `docs/domain/` 相关文档（level_rules_v1、victory_condition_rules_v1）

---

## 与现有 Message 表的关系

新增 Message：

| Message | 发送方 | 接收方 |
|---------|--------|--------|
| `LevelCompleted` | turn/victory_check | battle/record, ui |

需更新 `docs/architecture.md` 的 Message 注册表。
