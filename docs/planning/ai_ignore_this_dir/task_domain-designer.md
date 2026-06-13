# Task: @domain-designer — Campaign / Scenario 领域模型

## 触发来源
第三阶段评审报告（`docs/reviews/phase3_review_25第三阶段.md`）识别出最大缺口：项目缺少 `campaign/`、`content/`、`scenario/`、`progression/` 等游戏层模块。

## 背景
当前关卡配置（`assets/maps/tutorial.ron`）将地图、单位部署、胜利条件杂糅在一个文件中。评审建议分离为：
- `content/maps/` — 纯地图数据（尺寸 + 地形网格）
- `content/encounters/` — 敌人编队模板（可复用）
- `content/battles/` — 关卡配置（组合地图 + 编队 + 胜负条件）

此外，项目没有"战役/关卡序列"的概念，游戏直接启动 InGame，无法选择关卡。

## 现有基础
- `docs/domain/level_rules_v1.md` — 已有关卡配置领域规则（Level = Map + UnitDeployDef[] + VictoryCondition + TurnLimit）
- `docs/domain/victory_condition_rules_v1.md` — 已有胜负条件领域规则
- 已有 23 个领域文档（battle/buff/skill/turn/character 等），术语已对齐
- 已有 ADR `docs/adr/level-and-victory-system.md` — 关卡/胜负条件实现架构

## 任务目标
设计 **Campaign / Content Pipeline** 领域模型，回答以下核心问题：

1. 一个"战役"由什么组成？(Chapter / Stage / Battle 的层级关系)
2. Battle 配置与 Map 配置如何分离？各自的核心字段是什么？
3. Encounter（敌人编队）是否需要独立为可复用的领域概念？
4. 关卡选择/进度/解锁规则是什么？（玩家必须按顺序通关？还是自由选择？）
5. 当前 LevelConfigDef 的字段（map + units + conditions）如何映射到新的分离结构中？

## 预期成果
输出 `docs/domain/campaign_rules_v1.md`，包含：
- 1. 统一术语（Campaign / Chapter / Stage / Battle / Encounter / MapAsset）
- 2. 实体关系图（文本形式），如 Campaign → Chapter[] → Stage[] → Battle
- 3. 不变量（Invariants），如：Battle 必须引用一个已注册的 MapAsset
- 4. 流程定义：如何从 Campaign 配置选择并加载 Battle
- 5. 禁止事项（Forbidden），如：禁止 Battle 配置中内嵌地图数据
- 6. 术语与已有 level_rules_v1 的关系映射

## 时间节点
1 次迭代完成。完成后建议调用 @architect 进行架构设计。

## 参考文档（必须读取）
- `docs/domain/level_rules_v1.md` — 已有关卡规则
- `docs/domain/victory_condition_rules_v1.md` — 胜负条件规则
- `docs/reviews/phase3_review_25第三阶段.md` — 评审发现
- `docs/adr/level-and-victory-system.md` — 现有实现架构
- `assets/maps/tutorial.ron` — 当前"地图+关卡"单一文件示例

## 禁止事项
- 🟥 禁止讨论代码实现（ECS、Plugin、struct 等）
- 🟥 禁止讨论数据库/序列化
- 🟥 禁止重复定义已有术语（如 Level、Map、VictoryCondition 已在 level_rules_v1.md 中定义）
- 🟥 禁止设计 Quest/Save/Dialogue 系统（不在本次范围内）
- 🟥 禁止发明与已有术语冲突的新含义

## 交接
完成后 → 输出 `docs/domain/campaign_rules_v1.md` → 建议调用 @architect
