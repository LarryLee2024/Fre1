# Fre — Bevy 0.19+ SRPG 项目

> 基于 Bevy 0.19 的回合制战棋（SRPG）项目，采用 DDD 领域驱动设计 + GAS-Lite 能力系统。

## 项目概况

| 项 | 值 |
|------|------|
| 引擎 | Bevy 0.19 |
| 品类 | 单机回合制战棋 SRPG |
| 语言 | Rust 2024 Edition |
| 架构 | DDD 纵向三层 + 横切四层，Core 层 Capabilities/Domains 双轴结构 |
| 对标 | D&D 5e / BG3 / 铃兰之剑 |

## 架构总览

```
src/
├── shared/       # L0：底层原子层（强类型 ID、数学工具、确定性 RNG）
├── core/         # L1：领域规则层（Capabilities 15 能力 + Domains 15 业务域）
├── infra/        # L2：技术实现层（渲染、持久化、输入、寻路）
├── app/          # 横切1：启动装配层
├── content/      # 横切2：内容桥接层（数据驱动核心）
├── tools/        # 横切3：开发工具层
└── modding/      # 横切4：Mod 扩展层
```

依赖方向：`Shared ← Core ← Infrastructure`，横切层可依赖纵向层，禁止反向。

## 核心设计原则

1. **Feature First** — 按业务领域组织代码，禁止按技术类型拆分
2. **Capabilities/Domains 双轴** — Capabilities 管机制，Domains 管业务
3. **四级通信** — Hook / Trigger / Observer / Message
4. **数据驱动** — 玩法规则下沉 Domain/rules/，数值配置归 content/
5. **测试跟领域走** — 领域内聚四层测试（unit/integration/invariant/fixtures）

## 快速开始

```bash
# 构建
cargo build

# 运行
cargo run

# 运行测试
cargo test

# 代码检查
cargo clippy -- -D warnings
```

## 目录结构

```
Fre/
├── src/                    # 源码主目录
├── docs/                   # 文档与规范（见下方）
├── tests/                  # 跨领域测试
├── assets/                 # 游戏资源与配置数据
├── .trae/rules/            # AI 编码规则集（15 文件）
├── .qoder/agents/          # AI Agent 定义（7 角色）
├── AGENTS.md               # Agent 协作总纲
└── README.md               # 本文件
```

## 文档目录 (`docs/`)

| 目录 | 文件数 | 说明 |
|------|--------|------|
| `00-governance/` | 4 | 治理规则（宪法、编码规范、架构设计、UI 规格修正案） |
| `01-architecture/` | 45 | 架构设计文档（总纲 + ADR 决策记录，5 个专题子目录） |
| `02-domain/` | 36 | 领域规则（15 Capabilities + 15 Domains + Presentation + 状态/元素等） |
| `03-content/` | 41 | 内容架构（Def Schema / Registry / Validation / Localization） |
| `04-data/` | 50 | 数据架构（Schema、Foundation、Capabilities、Domains、Infrastructure） |
| `05-testing/` | 4 | 测试规范（测试宪法、测试规则、守护扫描） |
| `06-ui/` | 28 | UI 架构（Projection / ViewModel / Screen / Widget，7 子目录） |
| `08-knowledge/` | 38 | 知识库（含已完成归档文章） |
| `09-planning/` | 45 | 功能/迭代实施计划（含已归档） |
| `10-reviews/` | 37 | 代码/架构审查报告（含已归档） |
| `11-refactor/` | 24 | 技术债扫描和重构计划（含已归档） |

### 核心文档入口

| 文档 | 说明 | 优先级 |
|------|------|--------|
| `docs/01-architecture/README.md` | 纵向三层+横切四层架构总纲 + ADR 索引 | 最高 |
| `docs/00-governance/ai-constitution-complete.md` | 项目总宪法 v5.0（21 编） | 最高 |
| `docs/02-domain/README.md` | 领域规则索引（capabilities/ + domains/） | 必须遵守 |
| `docs/05-testing/test-spec.md` | 测试宪法 v4.0 | 必须遵守 |
| `docs/08-knowledge/README.md` | 知识库索引（9 篇文章，理解工作原理） | 推荐阅读 |

## AI 协作

本项目 7 个专用 Agent 协作开发，详见 `AGENTS.md`：

| Agent | 职责 |
|-------|------|
| @architect | 架构设计，输出 ADR |
| @domain-designer | 领域建模，输出领域规则 |
| @data-architect | 数据架构，设计 Schema |
| @feature-developer | 功能实现，按架构编码 |
| @code-reviewer | 代码审查，只提意见 |
| @test-guardian | 测试守护，领域规则优先 |
| @refactor-guardian | 技术债扫描，优先删代码 |

## `.trae/rules/` — AI 编码规则集

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

## 许可证

私有项目，未经授权禁止使用。
