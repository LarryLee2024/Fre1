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
     → @architect（ADR 架构设计） → 输出：`docs/08-decisions/`
     → @feature-developer（代码实现） → 输出：`src/`
     → @test-guardian（测试审查） → 输出：`docs/testing/`（计划）+ `src/` 和 `tests/`（代码）
     → @code-reviewer（代码审查） → 输出：`docs/reviews/`
     → @refactor-guardian（技术债扫描） → 输出：`docs/refactor/`
     
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

### `docs/02-domain/` — 领域规则

按领域子目录分组，开发对应功能时直接查阅：

**Core Domain（14 文件）— 核心业务规则**

| 文件 | 关键词 |
|------|--------|
| `battle/battle-rules.md` | 战斗状态机、Effect Pipeline、伤害计算 |
| `character/character-rules.md` | 角色属性、Faction、UnitSnapshot |
| `skill/skill-rules.md` | 技能定义、冷却、五阶段释放管线 |
| `attribute-modifier/attribute-modifier-rules.md` | Modifier 管线、属性修饰、叠加规则 |
| `turn/turn-rules.md` | TurnPhase、回合阶段、行动队列 |
| `trigger/trigger-rules.md` | 触发器、事件链（伤害→护盾→吸血→反击） |
| `condition/condition-rules.md` | 条件系统、效果判断、运行时条件 |
| `formula/formula-rules.md` | 公式系统、数值计算、表达式求值 |
| `selector/selector-rules.md` | 目标选择、AOE、空地选择 |
| `duration/duration-rules.md` | 持续时间（回合/真实时间/永久） |
| `cost/cost-rules.md` | 消耗系统、资源扣除 |
| `stack-policy/stack-policy-rules.md` | 堆叠策略、Buff叠加/替换 |
| `requirement/requirement-rules.md` | 释放前提、技能可用性检查 |
| `input/input-rules.md` | 输入处理、UiCommand |

**Infrastructure（7 文件）— 基础设施规则（→ `docs/03-technical/`、`docs/05-testing/`）**

| 文件 | 关键词 |
|------|--------|
| `03-technical/error-system-rules.md` | 错误处理、Result 传播、分级 |
| `03-technical/logging-rules.md` | 日志分级、格式、调试日志 |
| `03-technical/persistence-rules.md` | 存档格式、版本迁移 |
| `03-technical/hot-reload-rules.md` | Definition 热更新、战斗中禁止 |
| `03-technical/determinism-rules.md` | 确定性、多 RNG 流独立 |
| `03-technical/replay-rules.md` | 战斗回放、Command Stream |
| `05-testing/testing-rules.md` | 测试金字塔、回放测试 |

**Content/Data（6 文件）— 数据与内容规则（→ `docs/04-data/`）**

| 文件 | 关键词 |
|------|--------|
| `04-data/content-system-rules.md` | RON 加载、Registry、Definition 不可变 |
| `04-data/config-system-rules.md` | 运行时配置、热重载 |
| `04-data/content-migration-rules.md` | 版本兼容、字段兼容 |
| `04-data/asset-lifecycle-rules.md` | 资源生命周期、Handle 类型、内存预算 |
| `04-data/asset-organization-rules.md` | 三树分离、命名空间 |
| `04-data/feature-flag-rules.md` | Feature Flag、灰度发布 |

**Cross-cutting（12 文件）— 横切关注点**

| 文件 | 关键词 |
|------|--------|
| `01-architecture/layer-architecture-rules.md` | 分层架构、层间依赖方向 |
| `03-technical/ecs-communication-rules.md` | Hook/Observer/Message/Trigger |
| `03-technical/command-bus-rules.md` | UiCommand、命令总线 |
| `03-technical/shared-layer-rules.md` | Shared 层、公共类型 |
| `03-technical/modding-system-rules.md` | MOD 加载、资源隔离 |
| `03-technical/ui-architecture-rules.md` | ViewModel、UiCommand、UI 渲染 |
| `03-technical/localization-rules.md` | 多语言、Fluent |
| `02-domain/map/map-terrain-rules.md` | 地图地形、寻路、视野 |
| `02-domain/ai/ai-rules.md` | AI 行为、策略模板、决策管线 |
| `03-technical/performance-budget-rules.md` | 帧率目标、内存限制 |
| `04-data/validation-rules.md` | 数据完整性、配置校验 |
| `03-technical/event-audit-rules.md` | 事件审计、双轨制日志 |

### `docs/01-architecture/` — 架构设计（37 文件）

七层架构各领域的详细设计文档：

| 分组 | 文件 | 核心内容 |
|------|------|----------|
| 🏗️ 架构总纲 | `app-bootstrap.md` | App 装配器、状态机、启动/关闭序列 |
| | `layer-contracts.md` | 七层边界定义、三问判断法 |
| | `layer-architecture-rules.md` | 分层架构、层间依赖方向 |
| | `project-structure.md` | 三棵树分离、完整源码/资产/内容树 |
| | `plugin-design.md` | Plugin 生命周期、声明式注册 |
| | `plugin-contract-rules.md` | 显式依赖、API 最小化、分层禁令 |
| | `schedules-design.md` | 自定义 Schedule、SystemSet 排序 |
| | `infrastructure-design.md` | 20 个 Infrastructure 模块 |
| | `migration-roadmap.md` | 7 Phase 迁移计划 |
| ⚔️ 战斗/技能 | `battle-fsm-design.md` | 战斗 FSM、Guard/Action/Effect 三段式 |
| | `skill-buff-abstraction.md` | Effect Executor 抽象、ActionQueue |
| | `command-bus-design.md` | GameCommand、Memento 撤销 |
| 🧙 角色/属性 | `component-design-rules.md` | 四位一体组件分类、Hook 安全 |
| | `system-design-rules.md` | Query 参数上限、读写分离 |
| | `determinism-rules.md` | ChaCha8Rng、整数精度、状态哈希 |
| 🗺️ 地图/寻路 | `pathfinding-design.md` | PathFinder trait、UnitBlocker |
| 🎨 UI | `ui-domain-boundary-rules.md` | 单向数据流、ViewModel、UiCommand |
| 📦 数据/配置 | `content-pipeline.md` | RON→Def→Data→Registry、LoadingProgress |
| | `content-data-format.md` | RON 契约、两阶段加载 |
| | `content-migration-design.md` | 内容格式迁移链 |
| | `config-system-design.md` | 四层配置、反上帝配置 |
| | `ids-design.md` | Strong ID newtype、define_id! 宏 |
| | `asset-organization.md` | 三树分离、Content Packs、外包工作流 |
| | `asset-lifecycle-rules.md` | Handle 选择、分阶段卸载、内存预算 |
| | `asset-namespace-design.md` | 命名空间前缀、MOD 隔离 |
| | `save-migration-rules.md` | 存档 SemVer、三步删除原则 |
| 📋 事件/日志/错误 | `events-audit-design.md` | 独立 Struct 事件、EventWhitelist |
| | `logging-design.md` | 领域事件驱动日志、LogObserver |
| | `error-architecture.md` | 三层错误、失败分类学 |
| 🔧 工具/调试 | `tools-architecture.md` | Tools 二进制、data_validator |
| | `testing-architecture.md` | 五层测试金字塔、Golden Test |
| | `validation-rules.md` | 校验检查点、全局不变量 |
| | `performance-budget.md` | 60fps 帧预算、模块级预算 |
| | `feature-flag-design.md` | 7 个 Feature Flag、PluginGroup |
| | `i18n-design.md` | Fluent 国际化、字体回退链 |
| | `modding-design.md` | MOD 生命周期、分级权限策略 |
| | `collaboration-model.md` | AI 6-Agent 协作、Handoff 协议 |

### `docs/08-decisions/` — 架构决策记录（9 ADR）

| 文件 | 主题 |
|------|------|
| `ADR-001-migration-plan.md` | 迁移总计划 |
| `ADR-002-技术债修复方案.md` | 技术债治理策略 |
| `ADR-003-分层契约与依赖迁移.md` | 七层架构落地 |
| `ADR-004-内容与数据迁移方案.md` | 配置数据迁移 |
| `ADR-005-插件与通信迁移方案.md` | 插件体系与通信 |
| `ADR-006-验证与测试迁移方案.md` | 测试体系迁移 |
| `ADR-007-目录结构迁移映射.md` | 源码/资产/内容目录 |
| `ADR-008-核心机制与工程质量迁移.md` | 核心机制与质量门禁 |

### `docs/reviews/` — 代码审查记录

| 文件 | 说明 |
|------|------|
| `adr_migration_review.md` | ADR 迁移审查报告，含 8 个待解决问题（路径引用/版本号/矛盾点） |

### `docs/planning/` / `docs/testing/` / `docs/refactor/`

各 Agent 按协作流程向对应目录输出内容。