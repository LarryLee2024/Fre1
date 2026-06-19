# AGENTS.md — Bevy SRPG Project

## 项目概述
基于 Bevy 0.19 的回合制战棋项目，严格遵循 ECS 架构与领域分离原则。所有 Agent 输出必须以 `docs/00-governance/ai-constitution-complete.md` 为最高宪法准则。
只要遇到 `ai_ignore_this_dir` 这样的目录，除非我主动提到你们的文件，否则均视为不存在，严禁阅读。

## 通用行为红线（所有角色必须遵守）
1. 严禁绕过 Effect/Modifier 管线直接修改战斗数值与属性
2. 严禁突破模块边界、违反 ECS 架构模式
3. 严禁修改定义态（Definition）配置数据
4. 严禁超出自身角色职责范围跨环节作业，`@feature-developer` 严禁写单元测试代码
5. 严禁写过时、不符合最新 Bevy 0.19 版本的代码
6. 数据架构变更必须经过 @data-architect 审查，确保 Replay/Save 兼容性
7. 内容架构变更（Def Schema / Registry / Validation / Localization Key）必须经过 @content-architect 审查
8. UI 架构变更（Projection / ViewModel / Screen / Widget Contract）必须经过 @presentation-architect 审查

## Agent 分级体系（Tier S / A / B）

为管理 9 个 Agent 的协作复杂度，引入三级分治体系：

```
Tier S: 架构委员会（Architecture Board）      ── 战略层：定义规则与边界
Tier A: 工程委员会（Engineering Board）        ── 治理层：确保合规与质量
Tier B: 执行层（Implementation）              ── 战术层：按规则交付
```

### Tier S — 架构委员会（战略层）

5 个角色，共同构成 Architecture Board。其中 @data-architect 在 Tier A 也有治理角色（双角色），因为在设计阶段需要 Schema 作为下游输入：

| Agent | 职责 | 输入 | 输出 | 关键文档 |
|-------|------|------|------|---------|
| **@domain-designer** | 定义"规则是什么"—业务术语、不变量、状态机、流程 | 需求 | `docs/02-domain/` | 领域规则文档 |
| **@data-architect** | 定义"规则如何表达"—Schema、数据层划分、Replay/Save兼容 | Domain | `docs/04-data/` | Schema + Data Laws |
| **@content-architect** | 定义"Def 如何落地"—Def Schema、Registry、Validation、Asset、Localization | Domain + Data | `docs/03-content/` | Def 定义 + Registry |
| **@presentation-architect** | 定义"UI 如何表现"—Projection、ViewModel、Screen、Widget、Navigation | Domain | `docs/06-ui/` | UI 架构文档 |

**协作顺序（固定不串位）**：

```
需求
 │
 ├─→ @domain-designer     ← 第一步：定义领域规则
 │
 ├─→ @data-architect      ← 第二步：设计数据 Schema
 │
 ├─→ @content-architect   ← 第三步：设计 Def 落地（依赖 Schema）
 │
 └─→ @presentation-architect ← 第四步：设计 UI 表现（依赖领域规则）
         │
         ▼
  @architect（首席架构）    ← 第五步：系统集成与模块边界
```

### Tier A — 工程委员会（治理层）

3 个治理角色，负责质量把关：

| Agent | 职责 | 审查时机 |
|-------|------|---------|
| **@data-architect**（Tier S → Tier A 双角色） | 数据合规审查 | Schema 变更 / 迁移 |
| **@code-reviewer** | 代码合规审查 | PR 提交 |
| **@test-guardian** | 测试合规审查 | 功能完成 |

### Tier B — 执行层

2 个执行角色，只交付不决策：

| Agent | 职责 | 输出 |
|-------|------|------|
| **@feature-developer** | 按架构与领域模型编码 | `src/` 代码 |
| **@refactor-guardian** | 技术债扫描与清理 | `docs/11-refactor/` 债务清单 |

## 角色总览（9 个专用 Agent）

各角色严格守界。Agent 定义以双目录维护，格式不同但内容同步：

| 目录 | 目标工具 | 格式特点 |
|------|---------|---------|
| `.qoder/agents/*.md` | **OpenCode**（Sisyphus） | 原始 frontmatter（`name` + `tools: Read, Grep, Write`） |
| `.mimocode/agents/*.md` | **MiMo Code** | MiMo 兼容 frontmatter（`mode: subagent` + YAML `tools:` 字典） |

调用方式因工具而异（见下方"调用方式"章节），但 agent 行为完全一致。

### Agents By Tier

**Tier S — 架构委员会（Architecture Board）**

- **@domain-designer**：领域建模，输出领域规则文档；不讨论代码实现，术语与现有体系对齐
- **@data-architect**：数据架构设计，设计 Config/Save/Replay Schema、Registry 结构、ID 策略和数据迁移规则；确保数据结构统一、Schema 可演化、Replay/Save 兼容
- **@content-architect**：内容架构设计，设计 Def Schema、Registry、Validation、Dependency Graph、Localization Key、Asset 目录结构；将领域规则和数据 Schema 落地为可加载的配置定义系统。输入：02-domain（领域规则）+ 04-data（数据 Schema）；输出：03-content/
- **@presentation-architect**：UI/表现层架构设计，设计 Projection/ViewModel/Screen/Widget 分层方案、导航系统、UI 状态管理、Widget Contract。输入：02-domain（领域规则）；输出：docs/06-ui/
- **@architect**：首席架构师（系统集成），负责模块边界、依赖方向、Plugin 结构、ADR 决策记录、跨系统集成方案。**不再直接设计 Schema、UI、Content**，而是协调四个专业架构师的输出进行集成

**Tier A — 工程委员会（Engineering Board）**

- **@code-reviewer**：代码审查，按优先级校验合规性；只提意见不直接改代码
- **@test-guardian**：测试守护，以领域规则优先；Bug 必须转化为可复现的回放测试

**Tier B — 执行层**

- **@feature-developer**：功能实现，按架构与领域模型编码；发现架构问题立即上报，不私自修改；严禁写单元测试代码
- **@refactor-guardian**：技术债扫描（六大维度：架构漂移/抽象泄漏/AI可维护性/测试债务/内容债务/生命周期管理），定期输出债务清单；优先删代码而非加封装

## 调用方式

各 AI 工具通过不同机制调用这些 agent，以下分别说明。

### MiMo Code — `actor` 工具

主 agent 通过 `actor` 工具调用子 agent，`subagent_type` 取 `.mimocode/agents/*.md` 文件名（去 `.md`）。

**可用 subagent_type**：architect, domain-designer, data-architect, content-architect, presentation-architect, feature-developer, code-reviewer, test-guardian, refactor-guardian, explore, general

**调用模式**：
- `spawn`：后台执行，立即返回 actor_id，结果通过 notification 投递
- `run`：阻塞执行，结果内联返回
- `wait`：等待 spawn 的子 agent 完成

**并行调度**：单条消息内发多个 `actor({ action: "spawn", ... })` 调用，子 agent 并发执行。

**prompt 构造**：读取对应 `.mimocode/agents/*.md` 的 `---` 之后内容作为系统提示词前缀，拼接具体任务描述。

### OpenCode / Sisyphus — `task()` 映射

OpenCode（Sisyphus）则是通过 `task()` 函数调用，category 映射规则如下：

| 角色 | task() 调用方式 | 说明 |
|------|---------------|------|
| @architect | `task(category="unspecified-high", prompt="<agent prompt + 具体任务>")` | 复杂架构推理，完整推理能力 |
| @domain-designer | `task(category="unspecified-high", prompt="<agent prompt + 具体任务>")` | 领域建模分析 |
| @data-architect | `task(category="unspecified-high", prompt="<agent prompt + 具体任务>")` | Schema 设计 |
| @content-architect | `task(category="unspecified-high", prompt="<agent prompt + 具体任务>")` | 内容架构设计 |
| @presentation-architect | `task(category="unspecified-high", prompt="<agent prompt + 具体任务>")` | UI 架构设计 |
| @feature-developer | `task(category="deep", prompt="<agent prompt + 具体任务>")` | 需要 bash/write/edit 的编码工作 |
| @code-reviewer | `task(category="quick", prompt="<agent prompt + 具体任务>")` | 只读分析，无需写权限 |
| @test-guardian | `task(category="unspecified-high", prompt="<agent prompt + 具体任务>")` | 需要 write 权限的测试编写 |
| @refactor-guardian | `task(category="unspecified-high", prompt="<agent prompt + 具体任务>")` | 代码扫描分析 |

`<agent prompt>` 即 `.qoder/agents/*.md` 中 `---` 之后的全部内容（系统提示词）。Sisyphus 在执行任务前读取对应文件并将 prompt 注入 `task()`。

## 协作流程

### 标准开发流程（升级版）

```
需求
 │
 ├─→ ① @domain-designer（领域建模）
 │     输出：docs/02-domain/（领域规则、业务术语、不变量）
 │
 ├─→ ② @data-architect（数据架构）
 │     输入：领域规则
 │     输出：docs/04-data/（Schema 设计、数据层划分、Replay/Save 兼容）
 │
 ├─→ ③ @content-architect（内容架构）
 │     输入：领域规则 + 数据 Schema
 │     输出：docs/03-content/（Def Schema、Registry、Validation、Localization）
 │
 ├─→ ④ @presentation-architect（UI 架构）
 │     输入：领域规则
 │     输出：docs/06-ui/（Projection、ViewModel、Screen、Widget、Navigation）
 │
 └─→ ⑤ @architect（首席架构 / 系统集成）
       输入：① Domain + ② Data + ③ Content + ④ Presentation
       输出：docs/01-architecture/（ADR、分层方案、Plugin 方案、集成方案）
          │
          ▼
  @feature-developer（代码实现）
    消费：01 Architecture + 02 Domain + 03 Content + 04 Data + 06 UI
    输出：src/（代码）+ docs/09-planning/（执行计划）
          │
   ┌──────┼──────────────┐
   ▼      ▼              ▼
 @test-guardian      @code-reviewer
 (测试验证)           (代码审查)
   │                   │
   │    @content-architect (内容审查)     ← 验证 Def 实现是否合规
   │    @presentation-architect (UI审查)  ← 验证 UI 实现是否合规
   │                   │
   └───────┼───────────┘
           ▼
  @refactor-guardian（技术债扫描）
    输出：docs/11-refactor/（技术债清单）
```

### 触发时机与协作模式

| 场景 | 流程 | 说明 |
|------|------|------|
| **新项目 / 新功能** | Domain → Data → Content → Presentation → Architect → Developer → Test → Review | 完整流程 |
| **纯数据变更** | Data → Architect → Developer | 不涉及领域规则变更 |
| **纯内容变更** | Content → Developer | 不涉及领域规则和数据 Schema 变更 |
| **纯 UI 变更** | Presentation → Developer | 不涉及领域规则变更 |
| **Bug 修复** | Test（写失败测试）→ Developer → Test（验证）→ Code Review | 快速通道 |
| **重构优化** | Refactor Guardian（发现债务）→ Architect（评估）→ Developer → Code Review | 需架构评估 |
| **数据迁移** | Data Architect（Schema）+ Content Architect（Def 适配）→ Architect（架构适配）→ Developer（迁移实现） | 数据+内容双审查 |

### 设计输入依赖图

```
Feature Developer 消费的全部输入：

┌─────────────────────────────────────────────────────────┐
│                     Feature Developer                     │
├─────────────────────────────────────────────────────────┤
│  01 Architecture   02 Domain   03 Content   04 Data   06 UI  │
│  (模块边界)       (业务规则)   (Def 定义)    (Schema)   (UI 架构)│
└─────────────────────────────────────────────────────────┘
```

### Review 流程升级

代码审查不再是单一维度，而是多维审查：

| 审查类型 | 负责 Agent | 审查内容 |
|---------|-----------|---------|
| 代码质量 | @code-reviewer | 编码规范、性能、安全 |
| 测试覆盖 | @test-guardian | 领域不变量、边界条件 |
| 内容合规 | @content-architect（副角色） | Def 设计是否合理、Localization 是否规范 |
| UI 合规 | @presentation-architect（副角色） | ViewModel 是否正确、Projection 是否存在 |

## 必须做的行为
- 所有 Agent 写的日志，必须按 `.trae/rules/日志规则.md` 写日志，关键地方必须写日志。
- 跨角色协作时，必须引用上游输出作为输入依据，确保设计决策的可追溯性。
- `@feature-developer` 在代码中写 `// TODO: `，要严格遵守 TODO规范
- 必须用 `cargo nextest run` 替代 `cargo test` 做测试
- 所有 Agent 做完任务，一定要更新之前依据的相关文件（如plan、review、refactor、test等）的完成状态，以及各涉及文件目录下的README.md文件

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
* Directory structure
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
4. `docs/03-content/README.md` — 内容架构索引（Def Schema / Registry / Validation）
5. `docs/04-data/README.md` — 数据架构规范
6. `docs/05-testing/test-spec.md` — 测试宪法 v4.0
7. `docs/06-ui/README.md` — UI/表现层架构索引（Projection / ViewModel / Widget）

## Rules
Primary Rule Source: `.trae/rules/`
Consult relevant rule files before implementation.
