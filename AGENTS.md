# AGENTS.md — Bevy SRPG Project

## 项目概述
基于 Bevy 0.18.1 的回合制战棋项目，严格遵循 ECS 架构与领域分离原则。所有 Agent 输出必须以 `docs/00-governance/ai-constitution-complete.md` 为最高宪法准则。
只要遇到 `ai_ignore_this_dir` 这样的目录，均视为不存在，严禁阅读。

## 角色总览
共 7 个专用 Agent，各角色严格守界，详细 Prompt 见 `.qoder/agents/*.md`。
- **@architect**：架构设计，输出 ADR；只设计不写代码，所有方案不得违反架构规范
- **@domain-designer**：领域建模，输出领域文档；不讨论代码实现，术语与现有体系对齐
- **@data-architect**：数据架构设计，设计 Config/Save/Replay Schema、Registry 结构、ID 策略和数据迁移规则；确保数据结构统一、Schema 可演化、Replay/Save 兼容
- **@feature-developer**：功能实现，按架构与领域模型编码；发现架构问题立即上报，不私自修改；严禁写单元测试代码
- **@code-reviewer**：代码审查，按优先级校验合规性；只提意见不直接改代码
- **@test-guardian**：测试守护，以领域规则优先；Bug 必须转化为可复现的回放测试
- **@refactor-guardian**：技术债扫描，定期输出债务清单；优先删代码而非加封装

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

### 角色协同关系

| 角色 | 职责定位 | 输入来源 | 输出交付 | 下游依赖 |
|------|----------|----------|----------|----------|
| **@domain-designer** | 定义"规则是什么" | 业务需求 | 领域规则、术语、不变量 | @architect、@data-architect |
| **@data-architect** | 定义"规则如何表达" | 领域规则 | Schema 设计、数据层划分 | @architect、@feature-developer |
| **@architect** | 定义"系统如何组织" | 领域规则、数据架构 | ADR、模块边界、通信设计 | @feature-developer |
| **@feature-developer** | 实现"如何做" | ADR、领域模型、Schema | 代码实现 | @test-guardian、@code-reviewer |
| **@test-guardian** | 验证"是否正确" | 领域规则、实现代码 | 测试用例、回归测试 | @code-reviewer |
| **@code-reviewer** | 保证"质量合规" | 代码、测试、架构文档 | 审查报告 | @refactor-guardian |
| **@refactor-guardian** | 监控"技术健康" | 代码库、审查报告 | 技术债清单 | @architect（重大重构） |

### 触发时机与协作模式
1. **新项目启动**：@domain-designer → @data-architect → @architect → @feature-developer
2. **新增功能**：@domain-designer（如需要）→ @data-architect（如需要）→ @feature-developer → @test-guardian → @code-reviewer
3. **Bug 修复**：@test-guardian（写失败测试）→ @feature-developer → @test-guardian（验证）→ @code-reviewer
4. **重构优化**：@refactor-guardian（发现债务）→ @architect（评估影响）→ @feature-developer → @code-reviewer
5. **数据变更**：@data-architect（Schema 设计）→ @architect（架构适配）→ @feature-developer（迁移实现）

## 必须做的行为
- 所有 Agent 写的日志，必须按 `.trae/rules/日志规则.md` 写日志，关键地方必须写日志。
- 跨角色协作时，必须引用上游输出作为输入依据，确保设计决策的可追溯性。

## 通用行为红线（所有角色必须遵守）
1. 严禁绕过 Effect/Modifier 管线直接修改战斗数值与属性
2. 严禁突破模块边界、违反 ECS 架构模式
3. 严禁修改定义态（Definition）配置数据
4. 严禁超出自身角色职责范围跨环节作业，`@feature-developer` 严禁写单元测试代码
5. 严禁写过时、不符合最新 Bevy 0.18.1 版本的代码
6. 数据架构变更必须经过 @data-architect 审查，确保 Replay/Save 兼容性

## 参考文档

### 顶层核心文档（最高优先级）

| 文件 | 说明 | 优先级 |
|------|------|--------|
| `docs/01-architecture/README.md` | 纵向三层+横切四层架构总纲，Capabilities/Domains 双轴结构、ECS 规则 | 🟥 **最高** |
| `docs/00-governance/ai-constitution-complete.md` | 项目总宪法 v5.0（21 编），覆盖架构/ECS/代码/测试/日志/工程质量/AI 执行 | 🟥 **最高** |
| `docs/00-governance/coding-rules.md` | 编码执行规范，Feature First/ECS/Bevy 原生/通信机制 | 🟩 必须遵守 |
| `docs/02-domain/README.md` | 领域规则汇总索引（`capabilities/` + `domains/` 双轴结构） | 🟩 必须遵守 |
| `docs/04-data/README.md` | 数据架构规范，Schema 设计指南、Save/Replay 兼容规则 | 🟩 必须遵守 |
| `docs/05-testing/test-spec.md` | 测试宪法 v4.0，领域内聚四层测试/不变量测试/回放测试 | 🟩 必须遵守 |

### `.trae/rules/` — 项目规则集（15 文件）

AI 编码时直接引用，覆盖宪法分册与专项规则：

| 文件 | 内容定位 | 适用场景 |
|------|----------|----------|
| `架构规则.md` | 宪法 v5.0 · 架构篇 · 纵向三层+横切四层/Capabilities-Domains 双轴 | 新建模块、架构决策 |
| `ECS规则.md` | 宪法 v5.0 · ECS 篇 · Bevy ECS 最佳实践 | 编写 System/Component/通信 |
| `AI协作规则.md` | 宪法 v5.0 · AI 协作篇 · 26 条反模式黑名单 + 自检清单 | AI 编码/改 Bug |
| `SRPG专项规则.md` | 宪法 v5.0 · SRPG 专项篇 · 角色/技能/Buff/战斗/双轴架构 | 玩法系统开发 |
| `AI开发宪法.md` | 宪法 v5.0 紧凑执行版 · 最高优先级 10 条 + 禁令速查 | AI 快速对照 |
| `AI架构准则.md` | 英文简短版 · 架构原则/ECS/Rust/项目纪律 | 快速回顾 |
| `编码规则.md` | 编码执行规范 · Feature First/ECS/Bevy 原生/四级通信 | 日常编码 |
| `Bug修复规则.md` | Bug 分级（P0-P3）+ 修复流程 + 质量门禁 | Bug 修复 |
| `代码风格.md` | 命名/文件/函数/模块/Rust 风格规范 | 代码审查 |
| `注释规则.md` | 注释宪法 v1.0 · Why 优先/强制注释场景/注释禁令 | 写注释时 |
| `错误规则.md` | 分领域错误枚举/失败分类/禁止全局 AppError | 错误处理 |
| `日志规则.md` | tracing 结构化日志/领域事件驱动日志/分级规范 | 日志输出 |
| `审查规则.md` | 代码审查 Checklist（架构/领域/测试/命名/错误处理） | PR 审查 |
| `测试规范.md` | 测试宪法 v4.0 · 领域内聚四层测试/不变量测试 | 写测试时 |
| `文档治理规则.md` | 文档目录结构/命名规范/版本管理 | 文档编写 |

### `docs/` — 文档目录结构
详细内容见 `README.md`，以下是概览：
| 目录 | 文件数 | 说明 |
|------|--------|------|
| `00-governance/` | 8 | 治理规则（AI 开发宪法、编码规范、Bevy 参考） |
| `01-architecture/` | 19 | 架构设计文档（总纲 + 18 个 ADR 决策记录） |
| `02-domain/` | 31 | 领域规则（`capabilities/` 15 能力机制 + `domains/` 15 业务域） |
| `03-technical/` | 0 | 技术实现规则（ECS 通信、错误处理、日志、持久化、UI 架构等） |
| `04-data/` | 1 | 数据与配置规则（内容系统、配置系统、资源生命周期、Feature Flag、Schema 设计等） |
| `05-testing/` | 4 | 测试规范（测试宪法、测试规则） |
| `06-ai/` | 1 | AI 协作流程说明 |
| `07-operations/` | 1 | 运维文档 |
| `09-planning/` | 1 | 执行计划 |
| `10-reviews/` | 7 | 代码审查记录 |
| `11-refactor/` | 0 | 技术债扫描记录 |
| `98-roadmap/` | 1 | 项目路线图 |
| `99-history/` | 0 | 历史归档 |
| `其他/` | 0 | 临时目录 |

**核心文档入口：**
- 最高优先级：`01-architecture/README.md`（架构总纲 + ADR 索引）
- 领域规则：`02-domain/README.md`（领域文件速查）
- 数据架构：`04-data/README.md`（Schema 设计与数据治理）

详细文档说明请参阅 `README.md`。