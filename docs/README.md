# Fre 项目文档目录

> 50 万行 | 5 年维护 | AI 主开发 | Content First | DDD | SRPG

```
docs/
├── 00-governance/         宪法——最高的行为准则，所有架构/编码/AI 行为以此为准
├── 01-architecture/       架构——DDD 三层+横切四层、ADR 决策记录
├── 02-domain/             领域——32+ 领域规则（15 Capabilities + 1 Presentation + 16 Domains）
├── 03-content/            内容——5 层 Content 平台（Vocabulary/Capability/Entity/Gameplay/World）
├── 03-technical/          技术——技术设计文档（Localization 架构等）
├── 04-data/               数据——Schema、Data Laws、Save/Replay 兼容
├── 05-testing/            测试——四层测试宪法
├── 06-ai/                 AI——AI 开发协作规则
├── 06-ui/                  UI——L3 表现层架构（Projection/ViewModel/Screen/Widget）
├── 07-operations/         运维——部署、CI/CD、环境配置
├── 08-knowledge/          知识——项目知识库
├── 09-planning/           规划——功能/迭代实施计划
├── 10-reviews/            审查——代码/架构审查报告
├── 11-refactor/           重构——技术债扫描和重构计划
├── 98-roadmap/            路线图——长期规划
└── 99-history/            历史——废弃/归档文档
```

## 快速导航

| 想做什么 | 先去哪里 |
|---------|---------|
| 理解架构全貌 | `01-architecture/README.md` + ADR-056（Agent 治理） |
| 查业务规则 | `02-domain/README.md`（32+ 领域规则索引） |
| 查数据 Schema | `04-data/README.md`（40+ Schema 文件） |
| 查 Content Def | `03-content/README.md`（5 层 31 Def） |
| 查 UI 架构 | `06-ui/README.md`（7 子目录，25+ 文件） |
| 写测试 | `05-testing/test-spec.md`（四层测试规则） |
| 查 Agent 角色 | `AGENTS.md`（9 Agent 三级分治） |
| 不理解什么能做什么不能做 | `00-governance/ai-constitution-complete.md`（最高约束力） |

## 使用原则

- **宪法最高**：`00-governance/` 的约束力高于本文档所有其他内容
- **ADR 驱动变更**：架构变更必须通过 ADR（`01-architecture/`），禁止未经记录的架构修改
- **领域规则先于实现**：新增功能前先确认 `02-domain/` 中有对应的领域规则
- **Content First**：数值配置走 `content/`，不走代码；Def 设计走 `03-content/`，不走 `src/`

## 文档状态

| 状态 | 含义 |
|------|------|
| ✅ stable | 稳定，已通过审查 |
| ✅ accepted | 已接受，待全面实施 |
| 🟡 draft / in_progress | 编写中，内容可能变化 |
| 📋 proposed | 提议中，待审查 |
