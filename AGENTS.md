# AGENTS.md — Bevy SRPG Project

## 项目概述
基于 Bevy 0.18.1 的回合制战棋项目，严格遵循 ECS 架构与领域分离原则。所有 Agent 输出必须以 `docs/architecture.md` 为最高架构准则。

## 角色总览
共 6 个专用 Agent，各角色严格守界，详细 Prompt 见 `.qoder/agents/*.md`。
- **@architect**：架构设计，输出 ADR；只设计不写代码，所有方案不得违反架构规范
- **@domain-designer**：领域建模，输出领域文档；不讨论代码实现，术语与现有体系对齐
- **@feature-developer**：功能实现，按架构与领域模型编码；发现架构问题立即上报，不私自修改
- **@code-reviewer**：代码审查，按优先级校验合规性；只提意见不直接改代码
- **@test-guardian**：测试守护，以领域规则优先；Bug 必须转化为可复现的回放测试
- **@refactor-guardian**：技术债扫描，定期输出债务清单；优先删代码而非加封装

## 协作流程
需求 → @domain-designer（领域模型） → 输出：`docs/domain/`
     → @architect（ADR 架构设计） → 输出：`docs/adr/`
     → @feature-developer（代码实现） → 输出：`src/`
     → @test-guardian（测试审查） → 输出：`docs/testing/`（计划）+ `src/` 和 `tests/`（代码）
     → @code-reviewer（代码审查） → 输出：`docs/reviews/`
     → @refactor-guardian（技术债扫描） → 输出：`docs/refactor/`

## 通用行为红线（所有角色必须遵守）
1. 严禁绕过 Effect/Modifier 管线直接修改战斗数值与属性
2. 严禁突破模块边界、违反 ECS 架构模式
3. 严禁修改定义态（Definition）配置数据
4. 严禁超出自身角色职责范围跨环节作业
5. 严禁写过时、不符合最新 Bevy 0.18.1 版本的代码

## 参考文档
- `docs/architecture.md` — 完整架构规范（最高优先级）
- `docs/coding_rules.md` — 编码风格与工程规范
- `docs/testing_spec.md` — 测试体系规范
- `docs/AI开发宪法.md` — AI 开发总原则
- `.lingma/rules/` — 项目规则目录