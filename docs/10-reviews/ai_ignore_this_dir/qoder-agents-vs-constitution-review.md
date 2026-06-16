---
id: 10-reviews.qoder-agents-vs-constitution
title: .qoder/agents 与宪法 v5.0 冲突评审
status: draft
owner: code-reviewer
created: 2026-06-16
tags:
  - review
  - governance
  - agents
---

# .qoder/agents 与 ai-constitution-complete.md v5.0 冲突评审

**评审日期**: 2026-06-16
**评审范围**: `.qoder/agents/` 全部 7 个文件
**对比基准**: `docs/00-governance/ai-constitution-complete.md` v5.0

---

## 总览

| 状态 | 数量 | 文件 |
|------|------|------|
| ❌ 冲突 | 3 | architect.md、feature-developer.md、test-guardian.md |
| ⚠️ 有过时内容 | 3 | code-reviewer.md、domain-designer.md、refactor-guardian.md |
| ✅ 无冲突 | 1 | data-architect.md |

---

## 核心问题：工作目录路径不一致

Agent 文件中引用的路径与实际 `docs/` 目录结构不匹配：

| Agent 中的路径 | 实际正确路径 | 涉及文件 |
|----------------|--------------|----------|
| `docs/architecture.md` | `docs/01-architecture/` | architect、code-reviewer、feature-developer、test-guardian、refactor-guardian |
| `docs/adr/` | `docs/08-decisions/` | architect |
| `docs/domain/` | `docs/02-domain/` | architect、code-reviewer、domain-designer、feature-developer、test-guardian |
| `.lingma/rules/ai_constitution.md` | `docs/00-governance/ai-constitution-complete.md` | architect |
| `tests/common/fixtures.rs` | `<domain>/tests/fixtures/` | test-guardian |

---

## 逐文件评审

### ❌ 冲突文件

#### 1. architect.md

| 问题 | 详情 |
|------|------|
| 路径过时 | 第58行 `docs/architecture.md` → 应为 `docs/01-architecture/` |
| 路径过时 | 第59行 `docs/adr/` → 应为 `docs/08-decisions/` |
| 路径过时 | 第60行 `.lingma/rules/ai_constitution.md` → 应为 `docs/00-governance/ai-constitution-complete.md` |
| 路径过时 | 第129行 `docs/domain/` → 应为 `docs/02-domain/` |
| 路径过时 | 第133行 `docs/adr/` → 应为 `docs/08-decisions/` |
| 通信机制过时 | 第91行 Communication Design 缺少 Trigger（只有 Message/Observer/Hook） |
| 缺少双轴架构 | 未提及 Capabilities/Domains 双轴结构 |

#### 2. feature-developer.md

| 问题 | 详情 |
|------|------|
| 路径过时 | 第23行 `docs/architecture.md` → 应为 `docs/01-architecture/` |
| 路径过时 | 第24行 `docs/domain/` → 应为 `docs/02-domain/` |
| 通信机制过时 | 第48行只有 "Hook/Observer/Message"，缺少 Trigger |
| AI自检输出到代码 | 第110-122行要求代码中输出自检结果，宪法 v5.0 改为「文档参考，不输出到代码」 |

#### 3. test-guardian.md

| 问题 | 详情 |
|------|------|
| 路径过时 | 第33行 `docs/domain/` → 应为 `docs/02-domain/` |
| 路径过时 | 第34行 `docs/architecture.md` → 应为 `docs/01-architecture/` |
| 测试结构过时 | 第75-82行测试金字塔仍是旧结构（Unit 70%/Integration 20%/Replay 8%/E2E 2%） |
| 缺少 invariant 测试 | 未提及不变量测试这一最高价值层级 |
| 测试位置过时 | 第86行 `tests/common/fixtures.rs` → 应为 `<domain>/tests/fixtures/` |
| 缺少领域内聚 | 未描述「测试跟领域走（Feature First）」原则 |

---

### ⚠️ 有过时内容的文件

#### 4. code-reviewer.md

| 问题 | 详情 |
|------|------|
| 路径过时 | 第34行 `docs/architecture.md` 和 `docs/domain/` |
| 路径过时 | 第95行 `docs/domain/` |
| 通信机制过时 | 第41行 "Message/Observer/Command"，缺少 Trigger |
| 测试规范过时 | 第84行 "tests/ 目录结构规范"，应改为领域内聚结构 |

#### 5. domain-designer.md

| 问题 | 详情 |
|------|------|
| 路径过时 | 第36行 `docs/domain/` → 应为 `docs/02-domain/` |
| 路径过时 | 第48行 `docs/domain/*.md` → 应为 `docs/02-domain/` |
| 路径过时 | 第177行 `docs/domain/` → 应为 `docs/02-domain/` |

#### 6. refactor-guardian.md

| 问题 | 详情 |
|------|------|
| 路径过时 | 第82行 `architecture.md` → 应为 `docs/01-architecture/` |
| 路径过时 | 第120-121行 `architecture.md` 和 `docs/domain/` |

---

### ✅ 无冲突文件

#### 7. data-architect.md

内容与宪法一致，无路径问题，无架构冲突。

---

## 修复优先级

### P0 — 立即修复

1. **architect.md** — 5处路径错误 + 通信机制缺 Trigger
2. **feature-developer.md** — 2处路径错误 + 自检输出到代码
3. **test-guardian.md** — 3处路径错误 + 测试结构过时

### P1 — 尽快修复

4. **code-reviewer.md** — 3处路径错误 + 通信机制缺 Trigger
5. **domain-designer.md** — 3处路径错误
6. **refactor-guardian.md** — 3处路径错误

---

## 路径映射表

| 旧路径 | 新路径 |
|--------|--------|
| `docs/architecture.md` | `docs/01-architecture/` |
| `docs/adr/` | `docs/08-decisions/` |
| `docs/domain/` | `docs/02-domain/` |
| `.lingma/rules/ai_constitution.md` | `docs/00-governance/ai-constitution-complete.md` |
| `tests/common/fixtures.rs` | `<domain>/tests/fixtures/` |
