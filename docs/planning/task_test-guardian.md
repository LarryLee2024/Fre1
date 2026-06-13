# Task: @test-guardian — 内容加载测试 + UI 流程测试

## 触发来源
第三阶段评审后，@feature-developer 将产出：
1. 内容修缮（archer 技能修复、goblin_leader 新增）
2. 框架 UI（主菜单、关卡选择、GameOver 结果画面）

需要确保这些新增功能有对应的测试保护。

## 现有基础
- 已有 498 个测试（全部通过）
- 测试框架成熟：`tests/common/` + `tests/feature/` + `tests/scenario/` + `tests/golden/` + `tests/system/` + `tests/rule/`
- 标准测试数据：`tests/common/` 中的 UnitBuilder（Unit_001/002/003）
- 测试规范：`docs/testing_spec.md`
- 领域规则：23 个领域文档

## 任务目标

### A. 内容加载测试

#### A1. 新单位模板加载测试
确保 `enemy_goblin_leader.ron` 能被正确反序列化和加载：
- 测试 RON 反序列化成功
- 测试 UnitTemplate 字段正确（属性、技能、AI 行为）
- 参考已有测试：`character::template::tests::ron_deserialize_unit_template`

#### A2. 关卡配置完整性测试
- 测试 tutorial.ron（含 victory_condition + turn_limit）可正确加载
- 测试 LevelConfig.from_def 转换正确
- 测试所有 assets/ 中的数据文件可加载（已有 RegistryLoader 测试机制）

#### A3. Archer 技能修复验证测试
- 测试 player_archer 模板加载后 skill_ids 包含 "pierce"
- 测试 player_archer 模板中 basic_attack 和 pierce 都可用

### B. UI 流程测试（新）

**先决条件：** 等待 @feature-developer 完成框架 UI 实现。

#### B1. 主菜单测试
- 测试 MainMenu 状态下 UI 元素存在
- 测试"开始游戏"按钮触发状态转换

#### B2. 关卡选择测试
- 测试 LevelRegistry 中有数据时关卡列表展示
- 测试选择关卡后进入 InGame 状态

#### B3. GameOver 结果画面测试
- 测试 GameOverState::Victory 时 UI 显示胜利信息
- 测试 GameOverState::Defeat 时 UI 显示失败信息
- 测试"重新开始"按钮重置游戏状态
- 测试"返回菜单"按钮回到 MainMenu

#### B4. 完整流程测试（E2E）
- MainMenu → 选择关卡 → InGame → GameOver → 返回 MainMenu
- 每次重新开始确保状态完全重置（GameOverState = Playing, TurnState = 默认, 单位重生）

## 预期成果
输出到 `docs/testing/` 目录（测试计划）和 `src/` / `tests/`（测试代码）：

1. `docs/testing/content_loading_test_plan.md` — 内容加载测试计划
2. `docs/testing/ui_flow_test_plan.md` — UI 流程测试计划
3. 新增测试代码：
   - `src/character/template.rs` 或 `tests/feature/` 中的内容加载测试
   - `tests/system/` 或 `tests/feature/` 中的 UI 流程测试

## 测试金字塔要求
| 层级 | 占比 | 内容 |
|------|------|------|
| Unit | 70% | 模板反序列化、关卡配置解析、状态转换纯函数 |
| Integration | 20% | UI 组件渲染、GameOver 状态读取 |
| E2E | 10% | 完整流程（MainMenu→GameOver→MainMenu） |

## 时间节点
先完成 A 部分（内容加载测试），等 @feature-developer 完成 UI 实现后再完成 B 部分。

## 参考文档（必须读取）
- `docs/testing_spec.md` — 测试体系规范
- `docs/domain/level_rules_v1.md` — 关卡领域规则
- `docs/domain/character_rules_v1.md` — 角色领域规则
- `docs/domain/ui_rules_v2.md` — UI 领域规则
- `docs/reviews/phase3_review_25第三阶段.md`
- `src/character/template.rs` — 现有模板测试（参考格式）
- `tests/common/` — 标准测试数据
- `src/turn/state.rs` — AppState/GameOverState

## 禁止事项
- 🟥 禁止测试实现细节（不测试内部字段值、私有函数）
- 🟥 禁止在 UI 测试中依赖像素级断言（只验证组件存在性和状态转换）
- 🟥 禁止修改领域规则来让测试通过
- 🟥 禁止删除已有的 498 个测试
- 🟥 禁止使用非确定性数据（随机数必须 Seed=42）
- 🟥 禁止在 Bug 修复后再补测试（必须：失败测试 → 修复 → 通过）

## 交接
完成后 → 输出测试报告 → 建议调用 @code-reviewer 审查测试质量
