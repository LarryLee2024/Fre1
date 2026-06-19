# AGENTS.md — Bevy SRPG Project

## 项目概述
基于 Bevy 0.18.1 的回合制战棋项目，严格遵循 ECS 架构与领域分离原则。所有 Agent 输出必须以 `docs/00-governance/ai-constitution-complete.md` 为最高宪法准则。
只要遇到 `ai_ignore_this_dir` 这样的目录，均视为不存在，严禁阅读。

## 通用行为红线（所有角色必须遵守）
1. 严禁绕过 Effect/Modifier 管线直接修改战斗数值与属性
2. 严禁突破模块边界、违反 ECS 架构模式
3. 严禁修改定义态（Definition）配置数据
4. 严禁超出自身角色职责范围跨环节作业，`@feature-developer` 严禁写单元测试代码
5. 严禁写过时、不符合最新 Bevy 0.18.1 版本的代码
6. 数据架构变更必须经过 @data-architect 审查，确保 Replay/Save 兼容性

## 角色总览
共 7 个专用 Agent，各角色严格守界，详细 Prompt 见 `.qoder/agents/*.md`，请牢记这些 Agent 位置。
- **@architect**：架构设计，输出 ADR；只设计不写代码，所有方案不得违反架构规范
- **@domain-designer**：领域建模，输出领域文档；不讨论代码实现，术语与现有体系对齐
- **@data-architect**：数据架构设计，设计 Config/Save/Replay Schema、Registry 结构、ID 策略和数据迁移规则；确保数据结构统一、Schema 可演化、Replay/Save 兼容
- **@feature-developer**：功能实现，按架构与领域模型编码；发现架构问题立即上报，不私自修改；严禁写单元测试代码
- **@code-reviewer**：代码审查，按优先级校验合规性；只提意见不直接改代码
- **@test-guardian**：测试守护，以领域规则优先；Bug 必须转化为可复现的回放测试
- **@refactor-guardian**：技术债扫描（六大维度：架构漂移/抽象泄漏/AI可维护性/测试债务/内容债务/生命周期管理），定期输出债务清单；优先删代码而非加封装

## 协作流程

### 标准开发流程
```
需求
  │
  ├─→ @domain-designer（领域建模） → 输出：`docs/02-domain/`（领域规则、业务术语、不变量）
  │
  ├─→ @data-architect（数据架构）  → 输出：`docs/04-data/`（Schema 设计、数据层划分、Replay/Save 兼容）
  │
  └─→ @architect（架构设计）       → 输出：`docs/01-architecture/`（架构总纲 + ADR 决策记录）
          │
          ↓
  @feature-developer（代码实现）    → 输出：`src/`（代码）+ `docs/09-planning/`（执行计划）
          │
          ├─→ @test-guardian（测试审查）    → 输出：`docs/05-testing/`（测试计划）+ `tests/`（测试代码）
          │
          └─→ @code-reviewer（代码审查）    → 输出：`docs/10-reviews/`（审查报告）
                      │
                      ↓
          @refactor-guardian（技术债扫描）  → 输出：`docs/11-refactor/`（技术债清单）
```

### 触发时机与协作模式
1. **新项目启动**：@domain-designer → @data-architect → @architect → @feature-developer
2. **新增功能**：@domain-designer（如需要）→ @data-architect（如需要）→ @feature-developer → @test-guardian → @code-reviewer
3. **Bug 修复**：@test-guardian（写失败测试）→ @feature-developer → @test-guardian（验证）→ @code-reviewer
4. **重构优化**：@refactor-guardian（发现债务）→ @architect（评估影响）→ @feature-developer → @code-reviewer
5. **数据变更**：@data-architect（Schema 设计）→ @architect（架构适配）→ @feature-developer（迁移实现）

## 必须做的行为
- 所有 Agent 写的日志，必须按 `.trae/rules/日志规则.md` 写日志，关键地方必须写日志。
- 跨角色协作时，必须引用上游输出作为输入依据，确保设计决策的可追溯性。
- `@feature-developer` 在代码中写 `// TODO: `，要严格遵守 TODO规范
- 必须用 `cargo nextest run` 替代 `cargo test` 做测试

<!-- ENGINEERING_PRINCIPLES_START -->
## Engineering Principles

### Simplicity First
Prefer the simplest solution that satisfies the requirements.
Avoid:
* speculative abstractions
* future-proofing without evidence
* unnecessary configuration
* single-use indirection
If a solution can be simpler without sacrificing correctness, prefer the simpler version.

### Surgical Changes
Modify only what is required.
Do not:
* refactor unrelated code
* reformat unrelated files
* rename unrelated symbols
* introduce architecture changes outside the task scope
Every change should be traceable to the requested outcome.
<!-- ENGINEERING_PRINCIPLES_END -->



<!-- TOOL_PRIORITY_START -->
## Tool Priority
Always prefer the highest-level source available.
1. `CodeGraph` → code relationships, symbols, call chains
2. `Repomix` → architecture, repository overview
3. `Context7` → external library documentation
4. `Git` → history, rationale, regressions
5. `Filesystem` → implementation details
Never start implementation before consulting the highest-priority relevant source.
<!-- TOOL_PRIORITY_END -->

<!-- CODEGRAPH_START -->
## CodeGraph
If a `.codegraph/` directory exists, use CodeGraph before grep, find, or reading files.
Use it for:
* Symbol lookup
* Call chains
* Dependency tracing
* Impact analysis
CodeGraph is the source of truth for code relationships.
<!-- CODEGRAPH_END -->

<!-- REPOMIX_START -->
## Repomix
If a `repomix-output.xml` exists, read it before exploring the repository.
任何人在使用前，都必须在根目录 运行 `repomix` 输出最新的 `repomix-output.xml`
Use it for:
* Architecture review
* Directory structureßß
* Module boundaries
* Plugin discovery
* Project onboarding
Do not read large numbers of files before reviewing Repomix.
Repomix is the source of truth for repository structure.
<!-- REPOMIX_END -->

<!-- CONTEXT7_START -->
## Context7
For external libraries and frameworks, consult Context7 before relying on memory.
Use it for:
* API references
* Version-specific behavior
* Best practices
* Migration guidance
Never assume APIs for the project's framework version.
Documentation is the source of truth for external dependencies.
<!-- CONTEXT7_END -->

<!-- GIT_START -->
## Git
Use Git before making assumptions about why code exists.
Use it for:
* Recent changes
* Regression analysis
* Historical context
* Refactor safety checks
Always review relevant diffs before modifying existing code.
Git history is the source of truth for project evolution.
<!-- GIT_END -->

<!-- FILESYSTEM_START -->
## Filesystem
Use filesystem exploration only after higher-level sources have been consulted.
Use it for:
* Reading implementations
* Finding reusable code
* Creating or editing files
Before creating new code:
1. Search existing implementations.
2. Reuse existing abstractions.
3. Follow established project patterns.
Prefer consistency over novelty.
<!-- FILESYSTEM_END -->

<!-- THINKING_START -->
## Sequential Thinking
For any task estimated above 30 minutes of engineering work, think before coding.
Required workflow:
1. Understand
2. Analyze
3. Design
4. Validate
5. Implement
Before implementation, explicitly identify:
* Goal
* Constraints
* Existing systems
* Risks
* Alternatives
Never:
* Code before understanding architecture.
* Refactor before impact analysis.
* Introduce new systems without evaluating reuse.
* Change public APIs without considering downstream effects.
Reasoning first.
Implementation second.
<!-- THINKING_END -->

## Core References
1. `docs/00-governance/ai-constitution-complete.md` — 项目总宪法 v5.0
2. `docs/01-architecture/README.md` — 架构总纲 + ADR 索引
3. `docs/02-domain/README.md` — 领域规则索引
4. `docs/04-data/README.md` — 数据架构规范
5. `docs/05-testing/test-spec.md` — 测试宪法 v4.0

## Rules
Primary Rule Source: `.trae/rules/`
Consult relevant rule files before implementation.
