# Task: @code-reviewer — 第三阶段实现代码审查

## 触发来源
第三阶段评审后，以下 Agent 将按顺序产出：
1. @domain-designer → `docs/domain/campaign_rules_v1.md`
2. @architect → `docs/adr/campaign-pipeline.md` + `docs/adr/framework-ui.md`
3. @feature-developer → 内容修缮 + 框架 UI 代码
4. @test-guardian → 内容加载测试 + UI 流程测试

需要审查的代码分布在两轮迭代中。

## 审查范围

### 第一轮：内容修缮
- `assets/units/player_archer.ron` — 技能引用修改
- `assets/units/enemy_goblin_leader.ron` — 新建单位模板
- 对应测试代码

### 第二轮：框架 UI 实现
- `src/ui/menu/` 子模块（主菜单 + 关卡选择 + GameOver 结果画面）
- `src/ui/mod.rs` — UI 插件注册变更
- `src/main.rs` — 如涉及插件注册顺序变更
- 对应测试代码

### 文档审查（非代码，但需要确认完整性）
- `docs/domain/campaign_rules_v1.md` — 领域模型（仅确认存在，不审查内容质量）
- `docs/adr/campaign-pipeline.md` 和 `docs/adr/framework-ui.md` — 架构决策（仅确认存在）

## 审查优先级

```
第一轮：内容修缮（简单，高置信度）
  1. 架构合规性 — RON 文件是否遵循 Definition/Instance 分离
  2. Rust 代码质量 — 如有 Rust 代码变更
  3. 测试规范 — 内容加载测试是否正确

第二轮：框架 UI（复杂，需要更严格审查）
  1. 架构合规性 — 是否违反 architecture.md 的模块边界和通信规则
  2. ECS 模式正确性 — UI 组件/Syste/Resource 使用是否合理
  3. AppState 状态机 — MainMenu→LevelSelect→InGame→GameOver 转换是否完整
  4. Logic/Presentation 分离 — 框架 UI 是否混入业务逻辑
  5. Rust 代码质量
  6. 测试规范
```

## 具体审查要点

### 架构合规性检查
- 🟥 **禁止**：框架 UI 代码中混入战斗逻辑（伤害计算、效果管线等）
- 🟥 **禁止**：UI 层直接操作业务 Resource（应通过 Message/Command）
- 🟥 **禁止**：在 MainMenu/GameOver 状态下运行 InGame 的 System
- 🟩 **必须**：AppState 状态分离正确（各状态只运行自己的 System）
- 🟩 **必须**：新增 UI 子模块使用正确的 Plugin 注册方式

### ECS 模式检查
- UI Component 是否正确使用了 Tag Component 而非 bool 字段
- System 是否无状态（不存储中间状态）
- 是否符合 `src/ui/` 现有的模式（参考 events.rs + command_handler.rs）

### 内容修缮检查
- `enemy_goblin_leader.ron` 字段完整性（id/name/faction/base_attributes/skill_ids/trait_ids/ai_behavior）
- `player_archer.ron` 修改后是否保持兼容（老格式兼容性）

### 测试检查（参考 test-guardian 产出）
- 测试是否验证业务规则而非实现细节
- 测试是否使用标准测试数据
- 测试名称是否描述业务场景
- GameOver 测试是否验证了状态转换的完整性（包括状态重置）

## 预期成果
输出两份审查报告到 `docs/reviews/` 目录：

### `docs/reviews/code_review_content_fixes.md`
- 审查内容修缮部分（archer fix + goblin leader + 测试）

### `docs/reviews/code_review_framework_ui.md`
- 审查框架 UI 实现（menu/level_select/game_over + 测试）

## 时间节点
- 第一轮审查：等待 @feature-developer 完成内容修缮后执行
- 第二轮审查：等待 @feature-developer 完成框架 UI 后执行

## 参考文档（必须读取）
- `docs/architecture.md` — 最高架构规范（重点是：Feature 划分、Module 边界、通信规则）
- `docs/domain/campaign_rules_v1.md` — 领域模型
- `docs/adr/campaign-pipeline.md` — 内容管线架构
- `docs/adr/framework-ui.md` — 框架 UI 架构
- `docs/reviews/phase3_review_25第三阶段.md`
- `src/ui/mod.rs` — 现有 UI 架构
- `src/turn/state.rs` — AppState/GameOverState
- `src/main.rs` — 插件注册

## 禁止事项
- 🟥 **绝对禁止直接修改代码**（只提意见）
- 🟥 **绝对禁止生成修复后的代码**
- 🟥 禁止审查实现细节（如变量命名风格等低优先级问题当 Critical 提）
- 🟥 禁止在内容修缮审查中讨论框架 UI 问题
- 🟥 禁止在框架 UI 审查中讨论内容管线设计

## 审查报告格式
每份报告必须包含：

```
## Code Review Report

### ✅ 通过的检查
- [架构合规性] ...
- [ECS 模式] ...

### ❌ 发现的问题
#### [Critical/High/Medium/Low] 问题标题
- 位置：file.rs:line
- 规则：违反的规范条款
- 说明：为什么是问题
- 建议：如何修复

### 📋 总结
- Critical: X
- High: Y
- Medium: Z
- Low: W

### 🎯 结论
PASS / FAIL
```

## 交接
- Critical 问题修复后 → 建议再次调用 @code-reviewer 复审
- 发现系统性技术债（如框架 UI 架构有根本性问题） → 建议调用 @architect
- 发现测试质量问题 → 建议调用 @test-guardian
