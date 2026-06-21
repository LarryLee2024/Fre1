# Task: @architect — 内容管线架构 + 框架 UI 架构

## 触发来源
第三阶段评审（`docs/reviews/phase3_review_25第三阶段.md`）识别出两个需要架构设计的领域：

### A. 内容管线架构（Campaign / Content Pipeline）
评审最大缺口指出：项目缺少 `campaign/`、`content/` 游戏层模块。@domain-designer 将产出 `docs/domain/campaign_rules_v1.md`，本 ADR 基于该领域模型进行架构设计。

### B. 框架 UI 架构（Main Menu → Level Select → Game Over）
评审指出 P0 问题：游戏没有主菜单、无关卡选择、无胜利/失败结果画面。AppState 已有 MainMenu/InGame/GameOver 但无对应 UI。

## 现有基础
- `docs/architecture.md`（1855 行）— 完整 ECS 架构规范
- `docs/adr/level-and-victory-system.md` — 关卡/胜负条件架构决策
- `src/turn/state.rs` — AppState( MainMenu / InGame / GameOver ) + TurnPhase + GameOverState
- `src/map/data.rs` — LevelConfigDef / LevelConfig / LevelRegistry（当前"一切合一"模式）
- `src/ui/` — 已有完整战斗 UI 基础设施（面板/菜单/高亮/camera/vfx）
- `assets/maps/tutorial.ron` — 当前单个关卡配置示例

## 任务目标

### ADR A：内容管线架构
设计 Content Pipeline 的模块结构，包括：

1. **模块边界**：`src/campaign/` 或 `src/content/` 模块的设计
   - 与现有 `src/map/` 模块的职责划分（MapPlugin 应只负责地图运行时，不负责关卡组合）
   - 与现有 `src/turn/` 的边界（关卡加载→进入 InGame 的流程）
2. **数据结构**：
   - BattleDef（组合 MapRef + EnemyEncounter + VictoryCondition）
   - MapAsset（纯地图数据：尺寸 + 地形网格）
   - EncounterDef（敌人编队模板）
   - CampaignDef（Battle 序列 + 进度/解锁规则）
3. **加载流程**：从 assets 加载 RON → 构建 Registry → 游戏启动时初始化
4. **Plugin 注册**：ContentPlugin 或 CampaignPlugin 的注册顺序（数据层→逻辑层）

### ADR B：框架 UI 架构
设计 AppState 各状态的 UI 架构：

1. **MainMenu** 状态：主菜单 Plugin 设计，至少"开始游戏"+"退出"按钮
2. **LevelSelect** 状态（或合并到 MainMenu 的子阶段）：展示可选关卡列表
3. **GameOver** 状态：结果显示 UI（Victory/Defeat 画面 + 重玩/返回菜单按钮）
4. **状态转换流**：MainMenu → LevelSelect → InGame → GameOver → MainMenu（循环完整）
5. **与现有 UI 系统的集成**：当前 UiPlugin 在 InGame 状态下运行，不干扰框架 UI

## 预期成果
输出两个 ADR 到 `docs/adr/` 目录：

### `docs/adr/campaign-pipeline.md`
- Module Design：`campaign/` 模块的目录结构和文件组织
- 数据结构设计：BattleDef / MapAsset / EncounterDef / CampaignDef
- Communication Design：关卡加载流程的 Message/System 设计
- 插件注册顺序：相对于 MapPlugin / TurnPlugin 的位置
- Forbidden 清单

### `docs/adr/framework-ui.md`
- Module Design：框架 UI 的划分（menu/level_select/game_over 子模块）
- AppState 各状态的 UI System 注册方式
- 状态转换的 Communication Design
- 与现有 UI 系统的集成策略
- Forbidden 清单

## 时间节点
1 次迭代完成两个 ADR。完成后建议调用 @feature-developer 实现。

## 参考文档（必须读取）
- `docs/architecture.md` — 最高架构规范
- `docs/domain/campaign_rules_v1.md` — @domain-designer 产出的领域模型（先确认其存在）
- `docs/domain/level_rules_v1.md`
- `docs/domain/ui_rules_v2.md` — 已有 UI 领域规则
- `docs/reviews/phase3_review_25第三阶段.md`
- `docs/adr/level-and-victory-system.md`
- `src/turn/state.rs` — AppState/TurnPhase/GameOverState 定义
- `src/main.rs` — 插件注册顺序
- `src/ui/mod.rs` — 现有 UI 基础设施
- `src/map/data.rs` — LevelConfigDef / LevelRegistry

## 禁止事项
- 🟥 禁止写具体业务代码（不写 System/Component/UI 代码）
- 🟥 禁止修改已有架构边界（如不将 core/ 拆散）
- 🟥 禁止设计 Quest/Save/Dialogue 系统
- 🟥 禁止设计内容编辑工具
- 🟥 禁止在 content pipeline ADR 中包含 UI 细节
- 🟥 禁止在 framework UI ADR 中包含内容管线细节

## 交接
完成后 → 输出两个 ADR → 建议调用 @feature-developer
