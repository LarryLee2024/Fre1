# Task: @feature-developer — Battle_001 内容完善 + 框架 UI 实现

## 触发来源
第三阶段评审（`docs/reviews/phase3_review_25第三阶段.md`）识别出两个需要开发者实现的类别：

### A. 内容修缮（P1，优先级高）
多个内容资产存在问题，需要在开始新功能前修复。

### B. 框架 UI 实现（P0，优先级最高）
游戏进出流程不完整：没有主菜单、无关卡选择、无结果画面。

## 现有基础
- `assets/maps/tutorial.ron` — 当前唯一关卡（10×8，已修复 victory_condition 和 turn_limit）
- `assets/units/` — 5 个单位模板
- `assets/skills/` — 6 个技能，其中 3 个（pierce/heal/cleanse_skill）无人使用
- `src/ui/` — 完整战斗 UI 基础设施（panels/action_menu/highlight/events/command_handler）
- `src/turn/state.rs` — AppState(MainMenu/InGame/GameOver) + GameOverState(Victory/Defeat)
- `src/ui/panels/turn_indicator.rs` — 读取 GameOverState 但仅用于状态栏，无结果画面

## 任务目标

### A. 内容修缮

#### A1. 修复 player_archer.ron 缺失技能引用
- 当前：`skill_ids: ["basic_attack"]`
- 目标：`skill_ids: ["basic_attack", "pierce"]`
- pierce.ron 已存在于 `assets/skills/pierce.ron`，只需修改模板引用
- 文件：`assets/units/player_archer.ron`

#### A2. 新增 enemy_goblin_leader.ron
- 在 `assets/units/enemy_goblin_leader.ron` 创建哥布林队长模板
- 属性略高于普通哥布林（Might: 5, Vitality: 5, Agility: 5）
- 技能：`basic_attack` + `charge`
- 行为：`cautious`（谨慎型 AI）
- 参考 `enemy_goblin.ron` 和 `enemy_dark_knight.ron` 的格式

#### A3. 补充 3 个已有技能的"用户"
- heal.ron 已定义但无人引用 → 给 `player_mage` 或新增某个玩家职业追加
- cleanse_skill.ron 已定义但无人引用 → 可选追加（可以是 priest 类职业的战技，当前无对应职业可不做）
- 注意：如果增加 heal/cleanse 给已有职业，必须确保 `docs/domain/character_rules_v1.md` 的职业定义不冲突

### B. 框架 UI 实现（基于 ADR `docs/adr/framework-ui.md`）

**先决条件：** 等待 @architect 产出 `docs/adr/framework-ui.md` 后才能开始此部分。

#### B1. 主菜单（AppState::MainMenu）
- 实现简单主菜单 UI（"开始游戏" + "退出"按钮）
- 需要在 `src/ui/` 下新增 `menu/` 子模块
- 使用 Bevy UI 或现有的 widget 库
- 不依赖 egui 调试面板

#### B2. 关卡选择
- 实现关卡列表 UI（当前只有 "教学关"）
- 从 LevelRegistry 读取已注册关卡并展示
- 点击关卡→进入 AppState::InGame 并加载对应地图

#### B3. 胜利/失败结果画面
- 监听 GameOverState 变化（Playing → Victory/Defeat）
- 显示结果文字（"胜利！" / "失败！"）
- 提供"重新开始"和"返回菜单"按钮
- 游戏结束后不能停留在战斗画面

#### B4. 状态转换完整链路
- MainMenu → LevelSelect → InGame → GameOver → MainMenu
- 确保每次重新开始能重置所有状态（TurnState/TurnOrder/CombatLog 等）

## 预期成果
1. 内容修缮部分：修改 `assets/units/player_archer.ron` + 新增 `assets/units/enemy_goblin_leader.ron`
2. 框架 UI 部分：`src/ui/menu/` 子模块 + 修改 `src/ui/mod.rs` + 修改 `src/main.rs`（如需要）
3. 关卡选择数据源：如果 ADR 设计了 CampaignRegistry，基于其实现

## 时间节点
建议分两阶段：
- 第一阶段：内容修缮（简单，可立即开始）
- 第二阶段：框架 UI（需等待 @architect 的 ADR）

## 参考文档（必须读取）
- `docs/architecture.md` — 最高架构规范
- `docs/adr/framework-ui.md` — 框架 UI 架构设计（等待产出后读取）
- `docs/adr/campaign-pipeline.md` — 内容管线架构（等待产出后读取）
- `docs/domain/ui_rules_v2.md` — UI 领域规则
- `docs/reviews/phase3_review_25第三阶段.md`
- `src/ui/mod.rs` — 现有 UI 插件组合
- `src/ui/events.rs` — UI 事件定义
- `src/ui/command_handler.rs` — 命令处理模式
- `src/turn/state.rs` — AppState/GameOverState
- `src/map/data.rs` — LevelRegistry
- 现有单位模板：`assets/units/enemy_goblin.ron`, `assets/units/enemy_dark_knight.ron`

## 开发顺序（严格执行）
1. **Definition 优先**：先完成内容修缮（RON 文件修改），再实现 UI
2. **纯函数优先**：UI 渲染逻辑中不包含业务规则
3. **ECS 接入**：UI 组件通过 System 读取 GameOverState/LevelRegistry，不直接操作业务 Resource

## 禁止事项
- 🟥 禁止修改 ADR 定义的架构边界（等 ADR 出来后再实现 UI 部分）
- 🟥 禁止绕过 Effect Pipeline 直接扣血/加 Buff
- 🟥 禁止在内容修缮中修改技能定义（pierce.ron/heal.ron 等不需要改）
- 🟥 禁止创建 components.rs/systems.rs/utils.rs 巨文件
- 🟥 禁止在框架 UI 中混入战斗 UI 逻辑
- 🟥 禁止使用 `as any` / `@ts-ignore`（Rust 无此语法，类比：禁止 unsafe 块用于绕过类型）
- 🟥 **内容修缮不需要等待 ADR**，可以立即开始

## 自检清单
完成后输出：
```
Content Fixes: PASS/FAIL
Framework UI: PASS/FAIL
AppState Flow Complete: PASS/FAIL
Architecture Violation: NONE/XXX
Tests Added: YES/NO
cargo build: PASS/FAIL
```

## 交接
完成后 → 建议调用 @test-guardian → 建议调用 @code-reviewer
