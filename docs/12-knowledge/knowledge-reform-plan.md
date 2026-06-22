---
id: 12-knowledge.README
title: Knowledge Management System — Repomix 驱动的四层知识架构
status: proposal
owner: claude
created: 2026-06-22
updated: 2026-06-22
tags:
  - knowledge-management
  - repomix
  - memory-system
  - restructuring
---

# Knowledge Management System — Repomix 驱动的知识架构改造方案

> **核心理念**: Code = Source of Truth。文档是用来记录"为什么"，不是用来描述"是什么"。
> **核心工具**: Repomix（Skill 生成 + 代码包搜索）+ CodeGraph（符号索引）+ Auto Memory（个人化记忆）

---

## 0. 现状诊断

### 0.1 数量

| 指标 | 值 |
|------|----|
| 活跃文档数 | ~224 文件 |
| 总行数 | ~140,000 行（活跃）+ ~193,000 行（已归档） |
| 平均每文档 | ~525 行 |
| 重复内容 | 08-knowledge/ 与 01~06 重叠约 60% |
| Auto Memory | 完全未使用 |

### 0.2 核心问题

```
问题树：
  docs/ 膨胀（637 文件 / 334k 行）
  ├── 50+ ADR 已全部实现但仍占活跃区
  ├── 08-knowledge 概览文档是 01~06 的重复
  ├── 3 份 bevy 示例目录在 00-governance + 08-knowledge 都有
  ├── 18 个 bevy 版本说明，对当前项目有效仅 0.19
  └── 已完成计划/审查/重构共 91 个文件未归档
  │
  后果：
  ├── 每次会话 AI 需要扫描海量文件定位有效信息
  ├── 没有利用 Repomix Skill 预加载机制
  ├── Auto Memory 空置导致跨会话记忆丢失
  └── CodeGraph 已索引代码但文档仍保留大量"代码长什么样"的重复描述
```

---

## 1. Repomix 在知识架构中的角色

Repomix 提供两种互补能力，分别取代不同的传统文档功能：

| Repomix 能力 | 取代什么 | 优势 |
|-------------|---------|------|
| **`generate_skill`** | 你手动维护的"架构概览文档" | 从代码自动生成，永远不会过时 |
| **`pack_codebase` + `grep_repomix_output`** | 手动翻目录找文件 | 一次 pack 后任意搜索，不必知道文件在哪 |

两者组合后，`docs/` 的职责大幅收窄：

| 传统用法 | 新用法 |
|---------|--------|
| "读 docs/01-architecture/README.md 了解架构" | `/skill fre-architecture` 加载 Skill |
| "翻 docs/02-domain/xxx_domain.md 找规则" | `grep_repomix_output` 在 skill 中搜索 |
| "docs/03-content/definitions/xxx-def.md 看 Def 结构" | CodeGraph 直接查 Rust struct 定义 |
| "docs/08-knowledge/xxx-overview.md 了解概览" | **不存在了** — skill 里有代码和注释 |
| "看 plan doc 追踪进度" | Auto Memory `project` 类型自动记录 |

---

## 2. 四层知识架构（Repomix 驱动版）

```
┌──────────────────────────────────────────────────────────────────┐
│  L3: Skills（工作台）—— 按需加载，从代码生成                       │
│                                                                  │
│  使用 repomix `generate_skill` 从代码 + 关键文档生成预打包上下文：   │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │  Skill 名称           │ 来源（从代码而非文档生成）        │     │
│  ├─────────────────────────────────────────────────────────┤     │
│  │  fre-architecture     │ src/core/capabilities/ +         │     │
│  │                       │ src/core/domains/ 目录结构       │     │
│  ├─────────────────────────────────────────────────────────┤     │
│  │  fre-domain-rules     │ src/core/domains/*/rules/       │     │
│  ├─────────────────────────────────────────────────────────┤     │
│  │  fre-capabilities     │ src/core/capabilities/*/        │     │
│  ├─────────────────────────────────────────────────────────┤     │
│  │  fre-testing          │ docs/05-testing/（规则性文档保留） │     │
│  ├─────────────────────────────────────────────────────────┤     │
│  │  fre-ui               │ src/ui/ + docs/06-ui/           │     │
│  └─────────────────────────────────────────────────────────┘     │
│  位置: .claude/skills/<name>/                                    │
│  更新: 每次代码大的结构变更后重新生成                               │
│  使用: /skill fre-architecture                                   │
├──────────────────────────────────────────────────────────────────┤
│  L2: Auto Memory（自动记忆）—— 每次会话自动加载                     │
│                                                                  │
│  跨会话持久化，记录不可从代码推导的信息：                             │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │  user/     — 你的技术背景、偏好、角色（独游开发者）       │     │
│  │  feedback/ — 什么做法好/不好 + Why + How to apply        │     │
│  │  project/  — 当前目标、进度、blocker、sprint 状态         │     │
│  │  reference/— 外部资源（Grafana、Linear、Slack 等）        │     │
│  └─────────────────────────────────────────────────────────┘     │
│  位置: .claude/projects/.../memory/ + MEMORY.md                  │
│  更新: 每次对话自动读写，不需要手动维护                            │
├──────────────────────────────────────────────────────────────────┤
│  L1: Reference Docs（参考文档）—— 精简后仅保留"为什么"              │
│                                                                  │
│  删除所有"描述了代码长什么样"的文档，只保留：                       │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │  00-governance/   宪法 + 编码规则（行为规范，非代码描述）  │     │
│  │  01-architecture/ 仅未实现的 ADR（>90% 已实现→归档）      │     │
│  │  02-domain/       领域规则（不可从代码推导的业务约束）      │     │
│  │  （其他按需保留，整体目标减少 60% 文件）                   │     │
│  └─────────────────────────────────────────────────────────┘     │
│  位置: docs/                                                    │
│  原则: 只记录"为什么这样做"，不记录"做了什么"                       │
├──────────────────────────────────────────────────────────────────┤
│  L0: Code（源码）—— 最高的知识密度                                 │
│                                                                  │
│  CodeGraph 索引 + Repomix pack 覆盖所有场景：                      │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │  找符号定义    → codegraph_node(symbol=xxx)              │     │
│  │  理解流程      → codegraph_explore(query="how X works")  │     │
│  │  找调用者      → codegraph_callers(symbol=xxx)           │     │
│  │  全局搜索      → repomix pack + grep_repomix_output      │     │
│  │  读文件内容    → codegraph_node(file=xxx)                 │     │
│  └─────────────────────────────────────────────────────────┘     │
│  位置: src/                                                     │
└──────────────────────────────────────────────────────────────────┘
```

### 2.1 搜索路径（按顺序）

```
日常编码:
  1. CodeGraph（找定义/调用者/理解流程） — <1s
  2. Auto Memory（你之前的偏好/项目状态） — 自动加载
  3. Skill（领域规则/架构约束）           — /skill xxx
  4. Repomix pack（跨文件搜索）          — 一次 pack 后任意搜
  5. Raw docs（仅上述无法满足时）         — 手动打开

查阅外部:
  1. Context7（库/框架文档）
  2. WebSearch（社区/博客/Issue）
```

---

## 3. 分阶段实施计划

### Phase 1: 代码 → Skill（用 repomix 生成知识层）

**目标**: 先建能力，再删文档。先用 repomix 生成 Skills，确保知识不丢失。

#### 1.1 生成 5 个核心 Skill

使用 `repomix generate_skill` 从代码生成：

| Skill 名称 | 包含内容 | 来源 |
|-----------|---------|------|
| `fre-architecture` | 核心 capabilities + domains 目录结构、Plugin 组合、主要类型 | `src/core/` |
| `fre-domain-rules` | 15 个 domain 的 rules/ 纯函数代码 | `src/core/domains/*/rules/` |
| `fre-capabilities` | 15 个 capability 的机制代码 | `src/core/capabilities/*/mechanism/` |
| `fre-testing` | 测试规范 + 测试模板 | `docs/05-testing/` |
| `fre-ui` | UI 架构 + Screen/Widget 定义 | `docs/06-ui/` + 相关代码 |

#### 1.2 验证 Skill 可用性

测试每个 Skill 能被正确加载且包含足够上下文来回答常见问题。

---

### Phase 2: 文档大清理（依赖 Phase 1 完成）

**目标**: 删除/归档所有"代码已表达清晰"的文档。

#### 2.1 08-knowledge 整目录处理

全部删除，概览文档是 01~06 的重复，Skill 和 CodeGraph 覆盖一切：

| 文件 | 替代方案 |
|------|---------|
| `capabilities-overview.md` | Skill `fre-capabilities` + CodeGraph |
| `communication-overview.md` | `codegraph_explore("communication flow")` |
| `domain-overview.md` | Skill `fre-domain-rules` |
| `error-handling-overview.md` | 代码中 thiserror 模式清晰 |
| `ids-overview.md` | `src/shared/ids/types.rs` 完整定义 |
| `localization-overview.md` | ADR-053 已实现，代码已验证 |
| `logging-overview.md` | `codegraph_explore("logging observer")` |
| `pipeline-overview.md` | `codegraph_explore("pipeline execution")` |
| `random-overview.md` | `codegraph_node(file=random)` |
| `reflect-overview.md` | Bevy 标准模式 |
| `registry-overview.md` | `codegraph_node(symbol=DefRegistry)` |
| `replay-overview.md` | `codegraph_explore("replay recording")` |
| `ui-overview.md` | Skill `fre-ui` |
| `bevy-0.1.md` ~ `0.18.md` | 项目只用 0.19 |
| `bevy-examples.md` + tilemap/tiled | 官方文档更快更准 |

#### 2.2 架构 ADR 归档

50+ ADR 已全部实现，代码已体现这些决策。

**保留的例外**（尚未实现的架构变更）：
- `ADR-064` — Camera 系统（🟡 Proposed）
- `ADR-065` — Map 内容管线（🟡 Proposed）
- `ADR-066` — SSPEC 标准（新近 Accepted）
- `ADR-050` — 游戏状态机（🟡 涉及场景管理，仍在演进）

其余全部移入 `docs/01-architecture/done/`。

#### 2.3 计划/审查/重构归档

全部已完成项目从活跃区移出：
- `docs/09-planning/` — 3 活跃 + 39 归档
- `docs/10-reviews/` — 2 活跃 + 34 归档
- `docs/11-refactor/` — 4 活跃 + 18 归档

#### 2.4 00-governance 清理

删除 `bevy-examples-catalog.md`、`bevy-ecs-tilemap-examples-catalog.md`、`bevy-ecs-tiled-examples-catalog.md`（重复且官方文档更优）。

---

### Phase 3: 记忆系统初始化

**目标**: 启用 Auto Memory，让跨会话记忆生效。

#### 3.1 写入初始记忆

在 `.claude/projects/.../memory/` 创建：

```markdown
user/role.md
  → "独立游戏开发者，维护 Fre SRPG 项目"

feedback/codegraph-first.md
  → "查代码先走 CodeGraph，不够再用 Read/文件搜索"
  → Why: CodeGraph 一次调用返回源码 + 调用链，比 grep+read 循环快 10 倍

feedback/terse-response.md
  → "回复要简洁，直接给结论，不用 emoji"
  → How: 用短句，不要总结性段落

project/current-sprint.md
  → "文档系统改造：四层知识架构，目标 70% 瘦身"
  → Status: Phase 1（Skill 生成）进行中

reference/external-services.md
  → "bugs/issues → GitHub；参考游戏设计 → BG3/铃兰之剑数据在 docs/04-data/bo3/ 和 ll/"
```

#### 3.2 建立 Agent Memory 同步机制

将 agent 记忆中与"你个人"相关的信息（偏好、反馈）同步到 Auto Memory，agent 记忆保留纯技术细节。

---

### Phase 4: CLAUDE.md 精简

**目标**: 由当前 ~150 行精简至 ~60 行，只保留不可从代码推导的规则。

**删除/移入 Skill**:
- `docs/` 完整快速参考表（占用 ~40 行）→ `/skill fre-architecture` 替代
- 完整 Agent 分工说明（占用 ~30 行）→ `AGENTS.md` 保留，CLAUDE.md 只留 1 行链接
- 全部 `01-architecture/README.md` 的目录结构镜像（~50 行）

**保留**:
- Build & Test 快捷键
- P0 Red Lines（最高优先级规则）
- Tool Priority 排序
- Agent 使用一句话指导
- Testing Rules 核心

---

## 4. 预期效果

| 指标 | 改造前 | 改造后 | 降幅 |
|------|-------|-------|------|
| 活跃文档数 | ~224 | ~80 | **-64%** |
| 总行数 | ~140,000 | ~40,000 | **-71%** |
| 每次会话 AI 搜索范围 | 637 文件 / 334k 行 | MEMORY.md ~150 行 + 按需 Skill | **-99%** |
| 找架构定义耗时 | 5-15 次 Read 调用 | 1 次 `/skill fre-architecture` | **-90%** |
| 跨会话记忆 | 无 | Auto Memory 持续积累 | — |
| 知识过时风险 | 文档与代码不同步 | Skill 从代码生成，天然同步 | — |

---

## 5. 长期维护

### 5.1 文档创建规则

```
什么时候写文档？
  ✅ 记录"为什么这样设计"（决策原因、权衡、替代方案）
  ❌ 记录"代码长什么样"（CodeGraph 和 Repomix 已覆盖）
  ✅ 记录"不可从代码推导的约束"（业务规则、不变量、依赖顺序）
  ❌ 记录"如何使用某个函数"（签名 + 文档注释足够）
```

### 5.2 Skill 更新策略

| 触发条件 | 操作 |
|---------|------|
| 新 Capability/Domain 添加 | 重新生成对应 Skill |
| 架构重构导致目录结构变化 | 重新生成 `fre-architecture` |
| 每两周 | 检查 Skill 是否与代码同步 |

### 5.3 文档生命周期

```
文档通过此方式自然消亡：
  1. 代码实现了文档描述的决策
  2. Repomix/CodeGraph 可覆盖文档的查询场景
  3. Auto Memory 积累了足够的使用偏好
  → 文档可归档或删除
```

### 5.4 使用习惯

| 场景 | 做法 |
|------|------|
| 开始新工作 | `/skill fre-architecture` 快速加载架构上下文 |
| 查函数/类型 | 直接问 "X 在哪定义/怎么用" → CodeGraph |
| 全局搜索 | 代码 → `grep_repomix_output`；文档 → `grep -r` |
| 学到的偏好 | 我自动写入 Auto Memory，无需你操作 |
| 需要外部库帮助 | `Context7` 查最新文档 |
| 回顾工作 | `git log` 比计划文档准确 |

---

## 6. 实施时间线

| Phase | 内容 | 预估 | 执行顺序 | 依赖 |
|-------|------|------|---------|------|
| P1 | repomix 生成 5 个 Skill | ~30min | 1 | 无 |
| P2 | 文档大清理 | ~2h | 2 | 需 P1 完成后确保知识不丢 |
| P3 | Auto Memory 初始化 | ~30min | 3 | 无 |
| P4 | CLAUDE.md 精简 | ~20min | 4 | 无 |
| **合计** | | **~3h 20min** | **P1→P2→P3→P4** | |
