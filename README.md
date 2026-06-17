# Fre — Bevy 0.18+ SRPG 项目

> 基于 Bevy 0.18.1 的回合制战棋（SRPG）项目，采用 DDD 领域驱动设计 + GAS-Lite 能力系统。

## 项目概况

| 项 | 值 |
|------|------|
| 引擎 | Bevy 0.18.1 |
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
| `00-governance/` | 8 | 治理规则（宪法、编码规范、架构设计） |
| `01-architecture/` | 1 | 架构设计文档 |
| `02-domain/` | 31 | 领域规则（`capabilities/` 15 能力机制 + `domains/` 15 业务域） |
| `04-data/` | 1 | 数据架构（Schema、Save/Replay 兼容） |
| `05-testing/` | 4 | 测试规范（测试宪法、测试规则） |
| `08-decisions/` | 0 | 架构决策记录（ADR） |
| `10-reviews/` | 7 | 代码审查记录 |

### 核心文档入口

| 文档 | 说明 | 优先级 |
|------|------|--------|
| `docs/01-architecture/README.md` | 纵向三层+横切四层架构总纲 | 最高 |
| `docs/00-governance/ai-constitution-complete.md` | 项目总宪法 v5.0（21 编） | 最高 |
| `docs/02-domain/README.md` | 领域规则索引（capabilities/ + domains/） | 必须遵守 |
| `docs/05-testing/test-spec.md` | 测试宪法 v4.0 | 必须遵守 |

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

## 许可证

私有项目，未经授权禁止使用。
