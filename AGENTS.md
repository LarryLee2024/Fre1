# AGENTS.md — Bevy SRPG Project

## 项目概述
基于 Bevy 0.18.1 的回合制战棋项目，严格遵循 ECS 架构与领域分离原则。所有 Agent 输出必须以 `docs/01-architecture/README.md` 为最高架构准则。

## 角色总览
共 6 个专用 Agent，各角色严格守界，详细 Prompt 见 `.qoder/agents/*.md`。
- **@architect**：架构设计，输出 ADR；只设计不写代码，所有方案不得违反架构规范
- **@domain-designer**：领域建模，输出领域文档；不讨论代码实现，术语与现有体系对齐
- **@feature-developer**：功能实现，按架构与领域模型编码；发现架构问题立即上报，不私自修改
- **@code-reviewer**：代码审查，按优先级校验合规性；只提意见不直接改代码
- **@test-guardian**：测试守护，以领域规则优先；Bug 必须转化为可复现的回放测试
- **@refactor-guardian**：技术债扫描，定期输出债务清单；优先删代码而非加封装

## 协作流程
需求 → @domain-designer（领域模型） → 输出：`docs/02-domain/`
     → @architect（架构设计） → 输出：架构设计输出：`docs/01-architecture/` + ADR输出：`docs/08-decisions/`
     → @feature-developer（代码实现） → 输出：`src/`（代码），若有执行计划输出到 `docs/09-planning/` 
     → @test-guardian（测试审查） → 输出：`docs/05-testing/`（计划）+ `src/` 和 `tests/`（代码）
     → @code-reviewer（代码审查） → 输出：`docs/10-reviews/`
     → @refactor-guardian（技术债扫描） → 输出：`docs/11-refactor/`
     
## 必须做的行为
- 所有 Agent 写的日志，必须按 `.trae/rules/日志规则.md` 写日志，关键地方必须写日志。

## 通用行为红线（所有角色必须遵守）
1. 严禁绕过 Effect/Modifier 管线直接修改战斗数值与属性
2. 严禁突破模块边界、违反 ECS 架构模式
3. 严禁修改定义态（Definition）配置数据
4. 严禁超出自身角色职责范围跨环节作业
5. 严禁写过时、不符合最新 Bevy 0.18.1 版本的代码

## 参考文档

### 顶层核心文档（最高优先级）

| 文件 | 说明 | 优先级 |
|------|------|--------|
| `docs/01-architecture/README.md` | 七层架构总纲（v4.0），Feature 边界、ECS 规则、Effect/Modifier 管线 | 🟥 **最高** |
| `docs/00-governance/ai-constitution-complete.md` | AI 开发宪法 v1.6 完整版（20 部分），覆盖架构/ECS/代码/测试/日志/工程质量 | 🟥 **最高** |
| `docs/00-governance/coding-rules.md` | 编码执行规范 v1.0，AI 编码自检清单，Effect/Modifier 管线保护 | 🟩 必须遵守 |
| `docs/02-domain/README.md` | 领域规则汇总索引，39 个领域文件的速查入口 | 🟩 必须遵守 |
| `docs/05-testing/test-spec.md` | 测试宪法 v3.1，测试分层/回放测试/覆盖率策略 | 🟩 必须遵守 |

### `.trae/rules/` — 项目规则集（14 文件）

AI 编码时直接引用，覆盖宪法分册与专项规则：

| 文件 | 内容定位 | 适用场景 |
|------|----------|----------|
| `架构规则.md` | 宪法 v1.6 · 架构篇 · 顶层骨架与模块边界 | 新建模块、架构决策 |
| `ECS规则.md` | 宪法 v1.6 · ECS 篇 · Bevy ECS 最佳实践 | 编写 System/Component/通信 |
| `AI协作规则.md` | 宪法 v1.6 · AI 协作篇 · 24 条反模式黑名单 + 自检清单 | AI 编码/改 Bug |
| `SRPG专项规则.md` | 宪法 v1.6 · SRPG 专项篇 · 角色/技能/Buff/战斗 | 玩法系统开发 |
| `AI开发宪法.md` | 宪法 v1.1 紧凑执行版 · 最高优先级 10 条 + 禁令速查 | AI 快速对照 |
| `AI架构准则.md` | 英文简短版 · 架构原则/ECS/Rust/项目纪律 | 快速回顾 |
| `编码规则.md` | 编码执行规范 · Feature First/ECS/Bevy 原生/通信机制 | 日常编码 |
| `Bug修复规则.md` | Bug 分级（P0-P3）+ 修复流程 + 质量门禁 | Bug 修复 |
| `代码风格.md` | 命名/文件/函数/模块/Rust 风格规范 | 代码审查 |
| `注释规则.md` | 注释宪法 v1.0 · Why 优先/强制注释场景/注释禁令 | 写注释时 |
| `错误规则.md` | 分领域错误枚举/失败分类/禁止全局 AppError | 错误处理 |
| `日志规则.md` | tracing 结构化日志/领域事件驱动日志/分级规范 | 日志输出 |
| `审查规则.md` | 代码审查 Checklist（架构/领域/测试/命名/错误处理） | PR 审查 |
| `测试规范.md` | 测试宪法精简版 · 测试分类与优先级 | 写测试时 |

### `docs/` — 文档目录结构
详细内容见 `README.md`，以下是概览：
| 目录 | 文件数 | 说明 |
|------|--------|------|
| `00-governance/` | 5 | 治理规则（AI 开发宪法、编码规范、Bevy 参考） |
| `01-architecture/` | 38 | 七层架构设计文档（App 装配、战斗 FSM、组件设计、数据管线等） |
| `02-domain/` | 24 子目录 | 领域规则（Ability/AI/Attribute/Battle/Buff/Character/Condition/Cost/Cue/Duration/Effect/Execution/Formula/Input/Map/Requirement/Selector/Skill/StackPolicy/Tag/Targeting/Trigger/Turn） |
| `03-technical/` | 15 | 技术实现规则（ECS 通信、错误处理、日志、持久化、UI 架构等） |
| `04-data/` | 8 | 数据与配置规则（内容系统、配置系统、资源生命周期、Feature Flag 等） |
| `05-testing/` | 4 | 测试规范（测试宪法、测试规则） |
| `06-ai/` | 1 | AI 协作流程说明 |
| `07-operations/` | 1 | 运维文档 |
| `08-decisions/` | 27 | 架构决策记录 |
| `09-planning/` | 2 | 执行计划 |
| `10-reviews/` | 1 | 代码审查记录 |
| `11-refactor/` | 1 | 技术债扫描记录 |
| `98-roadmap/` | 1 | 项目路线图 |
| `99-history/` | 2 | 历史归档（含 7 个归档子目录） |
| `其他/` | 1 | 临时目录（ai_ignore_this_dir） |

**核心文档入口：**
- 最高优先级：`01-architecture/README.md`（七层架构总纲）
- 领域规则：`02-domain/README.md`（39 个领域文件速查）
- 架构决策：`08-decisions/README.md`（ADR 索引）

详细文档说明请参阅 `README.md`。