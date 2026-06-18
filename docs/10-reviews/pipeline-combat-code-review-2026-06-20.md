---
id: 10-reviews.pipeline-combat-code-review
title: 代码审查 — Pipeline → Combat 接入（回合管线替代 TurnSubState）
status: completed
owner: code-reviewer
created: 2026-06-20
tags:
  - code-review
  - pipeline
  - combat
  - infrastructure-integration
---

# 代码审查报告 — Pipeline → Combat 接入

## 审查范围

Phase F 完成后对 `pipeline → combat 流程` 的代码审查，涉及将 `TurnSubState` 状态机替换为 `CombatPipelineDriver` 驱动的回合管线。

### 审查文件

**新建文件** (4):
- `src/core/domains/combat/pipeline/mod.rs` — 模块入口
- `src/core/domains/combat/pipeline/definition.rs` — `build_turn_pipeline()` 管线定义
- `src/core/domains/combat/pipeline/steps.rs` — 5 个步骤执行函数
- `src/core/domains/combat/pipeline/driver.rs` — `CombatPipelineDriver` Resource + Update System + Observer

**修改文件** (5):
- `src/core/domains/combat/plugin.rs` — 注册模式重写
- `src/core/domains/combat/components.rs` — 移除 TurnSubState
- `src/core/domains/combat/systems/turn_systems.rs` — 缩减为仅生命周期+Observer
- `src/core/domains/combat/mod.rs` — 添加 pipeline 子模块
- `src/core/domains/combat/tests/unit/turn_test.rs` — 移除 TurnSubState 测试

## ✅ 通过的检查项

### 架构合规性
- [x] Feature First：pipeline/ 正确作为 combat 域的子模块，未引入全局目录
- [x] 领域逻辑在域内：所有回合逻辑仍保留在 combat 领域内部
- [x] 未绕过 Effect/Modifier Pipeline — 步骤函数通过领域事件（OnTurnStart/OnTurnEnd）与效果管线交互
- [x] 定义与实例分离：PipelineDefinition 在 plugin.rs 中构建并注册，运行时通过 Registry 获取
- [x] 无 AppError/全局错误：所有错误情况使用 warn! 日志，不 panic
- [x] 无跨域直接引用：integration/ 层保留为唯一对外接口

### ECS 模式合规性
- [x] 系统为无状态函数（CombatPipelineDriver 作为 Resource 持有运行时状态）
- [x] Component 为纯数据（ActionPoints impl 块仅含 getter/setter）
- [x] Observer 使用正确：`On<'_, '_, UnitActionComplete>` 用于恢复驾驶员
- [x] 未发现 OOP 模式
- [x] 未使用 bool 字段替代 Tag 组件
- [x] Trigger (commands.trigger) 用于领域事件，Observer 用于事件消费

### Rust 代码质量
- [x] 无 unwrap() / expect() 在业务代码中
- [x] 可见性恰当：所有新类型和函数均为 `pub(crate)`
- [x] 无过度抽象或 Trait 滥用
- [x] 迭代器模式适当使用（`position()`, `iter().fold()`, `contains_key()`）

### Bevy 0.18 最佳实践
- [x] 使用 `commands.trigger()` + Observer 而非旧版 EventReader/EventWriter
- [x] Component 使用 `register_type()` + Reflect 注册
- [x] 跨阶段通信通过 Trigger (OnTurnStart, OnTurnEnd, OnBattleEnd)
- [x] Schedule 使用合理（Update 管线驱动，Observer 事件响应）

### 代码规范
- [x] 命名约定正确（PascalCase 类型，snake_case 函数）
- [x] 模块级文档注释完整（含 ASCII 流程图）
- [x] 函数有 doc comment 说明职责
- [x] 无死代码或注释掉的代码块
- [x] 无 TODO 遗留

## ❌ 发现的问题

### [High] H1. Core → Infra 反向依赖 — combat 域直接引用 infra::pipeline

- **位置**: `src/core/domains/combat/pipeline/driver.rs:17`
- **规则**: `docs/01-architecture/README.md` §2.3 — 依赖方向严格单向：`Shared ← Core ← Infra`
- **说明**: `combat/pipeline/driver.rs` 导入 `crate::infra::pipeline::PipelineRegistry`。架构总纲明确禁止 Core 层（包括 domains/）依赖 Infra 层。`PipelineRegistry` 放置在 `infra/pipeline/` 但 Core 层的 combat domain 直接引用它。当前 `PipelineDefinition/PipelineState/PipelineContext` 等 foundation 类型在 `core/capabilities/runtime/pipeline/foundation/` 中，而 Registry 在 Infra，导致 Domain 被迫跨层依赖。
- **建议**:
  - 方案 A（推荐）: 将 `PipelineRegistry` 移至 `core/capabilities/runtime/pipeline/registry.rs`（与 foundation 类型同层），Infra 保留 wrapper/re-export
  - 方案 B: 在 Core 层定义 `PipelineRegistryAccess` trait，combat 只依赖 trait，Infra 实现
  - 方案 C: 接受为已知架构例外并文档化（风险：后续更多 Core 模块可能复制此模式）

### [Medium] M1. 新增管线代码无测试覆盖

- **位置**: `src/core/domains/combat/pipeline/` 全部 4 个文件
- **规则**: `docs/05-testing/test-spec.md` — 新增 Observer、System、Resource 必须有测试覆盖
- **说明**: 新建的 pipeline 模块（definition.rs, steps.rs, driver.rs, mod.rs）以及 `on_unit_action_complete` Observer 均无单元测试或集成测试。`turn_test.rs` 仅覆盖了保留的数据类型（ActionPoints, TurnQueue）。未验证的内容包括：
  - Pipeline 步骤函数的边界条件（空队列、无 ActionPoints）
  - CombatPipelineDriver 状态转移（暂停/恢复/完成）
  - Observer 对 UnitActionComplete 的响应
  - PhaseCheck 分支逻辑（HasActions → UnitAction / Idle → TurnSettlement）
- **建议**: 纯函数步骤（`step_phase_check`, `step_turn_start`, `step_turn_settlement`, `step_turn_end`）可添加单元测试。Driver 和 Observer 需要集成测试（App 级别构造）。调用 @test-guardian 角色。

### [Medium] M2. CombatPipelineDriver 字段 pub 暴露破坏封装

- **位置**: `src/core/domains/combat/pipeline/driver.rs:30-34`
- **规则**: `docs/00-governance/ai-constitution-complete.md` — 最小可见性原则；ADR-021 §Forbidden — 禁止外部操作回合状态
- **说明**: `CombatPipelineDriver` 的 `state` 和 `paused` 字段均为 `pub`。`PipelineState` 的字段也全部 `pub`。这允许外部任意代码直接 `driver.paused = false` 跳过暂停，或 `driver.state.current_stage_index = 3` 直接跳阶段，破坏了 ADR-021 禁止的"外部跳过 UnitAction 直接进入 TurnSettlement"。
- **建议**: 改为 `pub(crate)` 限制到 combat 模块内，或提供安全的 setter 方法（现有 `resume()`/`start_turn()` 不完整）。更彻底：只暴露查询方法（`is_driving()` 等），禁止外部直接写 state。

### [Medium] M3. Pipeline ID 字符串硬编码在两处位置

- **位置**: `src/core/domains/combat/pipeline/definition.rs:14` 和 `driver.rs:42`
- **规则**: `.trae/rules/编码规则.md` — 避免魔数字符串
- **说明**: `"combat.turn"` 同时在 `build_turn_pipeline()` 和 `CombatPipelineDriver::new()` 中硬编码。两处无编译期关联。若将来 ID 变更（例如改为 `"combat.turn_phase"`）只改了一处，会出现运行时 `registry.get()` 返回 None，此时 driver 静默标记 completed 并退出，回合循环无声中断。
- **建议**: 定义常量 `pub const COMBAT_TURN_PIPELINE_ID: &str = "combat.turn";`，两处引用同一常量。或让 `CombatPipelineDriver::new(id)` 参数化。

### [Medium] M4. TurnEnd 中内嵌团队胜负判定逻辑

- **位置**: `src/core/domains/combat/pipeline/steps.rs:188-220`
- **规则**: `docs/01-architecture/README.md` §6 — 业务规则应放入 `rules/`；函数职责单一
- **说明**: `step_turn_end` 末尾约 30 行的团队全灭检查（fold 统计各队伍存活数 → `alive_teams <= 1` 判定胜负）属于业务规则而非步骤编排。这使 step 函数混合了"步骤推进"和"业务判定"两种职责。
- **建议**: 提取为独立函数 `fn check_team_elimination(query: &Query<&CombatParticipant>) -> bool`，放入 `combat/rules/` 或 `turn_systems.rs` 中。step_turn_end 只保留编排逻辑（advance + 事件发射 + 调用判定函数）。

### [Low] L1. step_phase_check 签名使用 `&Query<&mut ActionPoints>` 但仅读取

- **位置**: `src/core/domains/combat/pipeline/steps.rs:65`
- **规则**: Rust 最小权限原则 — 不需要写权限时不应声明 &mut
- **说明**: `step_phase_check` 签名 `ap_query: &Query<&mut ActionPoints>` 但仅调用 `get()` 读取。`&mut` 查询会在调用方有其他 `&mut` 查询时产生冲突。当前 Driver 中确实有 `mut ap_query: Query<&mut ActionPoints>` 用于其他步骤，所以调用侧没出问题，但类型签名夸大了权限。
- **建议**: 改为 `&Query<&ActionPoints>`。如需保留 &mut 供其他步骤共享，可接受现状。低优先级。

## 📋 总结

| 严重程度 | 数量 | 关键问题 |
|---------|------|----------|
| Critical (严重) | 0 | — |
| High (高) | 1 | Core→Infra 反向依赖 |
| Medium (中) | 4 | 无测试覆盖 / 字段暴露 / ID 硬编码 / 业务逻辑嵌入 |
| Low (低) | 1 | Query 权限过宽 |

## 🎯 结论

**FAIL** — 存在 1 个 High 级别问题（架构依赖方向违规），需要处理后方可 PASS。

### 建议修复顺序

1. **[架构决策]** Core→Infra 依赖方向 — 需 @architect 确认方案（迁移 Registry / Trait 抽象 / 接受例外）
2. **[快速修复]** M2 字段可见性 + M3 ID 常量 + M4 提取胜负判定 — @feature-developer 可独立完成
3. **[测试]** M1 测试覆盖 — 需 @test-guardian 介入
4. **[可选]** L1 签名精简 — 低优先级

### 推荐下一步

启动 Pipeline → Combat 接入后的后续任务（ErrorContext → pipeline/save，见 `docs/09-planning/infrastructure-integration-plan.md` §1.4，预估 0.5 天），同时并行修复 M2/M3/M4 低风险问题。H1 的 Core→Infra 问题可先决策方案再集中处理。
