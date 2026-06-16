---
id: 06-ai.README
title: AI
status: stable
owner: architect
created: 2026-06-14
updated: 2026-06-16
tags:
  - ai
---

# AI

AI 开发协作规则。

## 文档列表

| 文件 | 主题 |
|------|------|
| `../AGENTS.md` | 7-Agent 角色定义与协作流程 |
| `../01-architecture/collaboration-model.md` | AI 协作模型与 Handoff 协议 |

## 角色概览

| 角色 | 职责 | 工具权限 | 可写代码 |
|------|------|----------|----------|
| **@architect** | 架构设计，输出 ADR | Read, Grep, Glob, Write | 否（写文档） |
| **@domain-designer** | 领域建模，输出领域文档 | Read, Write, Grep | 否（写文档） |
| **@data-architect** | 数据架构设计，Schema/Registry/ID 策略 | Read, Grep, Glob, Write | 否（写文档） |
| **@feature-developer** | 功能实现，按架构与领域模型编码 | Read, Write, Edit, Glob, Grep, Bash | **是** |
| **@code-reviewer** | 代码审查，按优先级校验合规性 | Read, Grep, Glob | 否（只读） |
| **@test-guardian** | 测试守护，以领域规则优先 | Read, Grep, Glob, Write, Edit | 是（仅测试代码） |
| **@refactor-guardian** | 技术债扫描，输出债务清单 | Read, Grep, Glob, Bash | 否（只读） |

角色配置文件位于 `.qoder/agents/` 目录。
