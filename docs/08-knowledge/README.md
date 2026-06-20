---
id: 08-knowledge.README
title: Knowledge Base — 知识库总览
status: draft
owner: architect
created: 2026-06-19
tags:
  - knowledge
  - index
---

# Knowledge Base — Fre 项目知识库

> 通俗知识介绍，以「理解工作原理」为目标，非正式设计文档。

## 目录

| 领域 | 文章 | 说明 |
|------|------|------|
| 日志系统 | [日志系统深度解析](logging-overview.md) | 从宪法到代码：日志如何设计、如何流动、如何扩展 |
| 错误处理 | [错误处理架构深度解析](error-handling-overview.md) | Error vs Failure 严格分离：为什么这么设计、代码怎么跑 |
| *待补充* | | |

## 定位

Knowledge Base（`docs/08-knowledge/`）与 Architecture（`docs/01-architecture/`）不同：

- **Architecture** 是规范——告诉你要怎么做、不能怎么做
- **Knowledge** 是解释——告诉你为什么这么做、代码怎么跑起来的

新成员阅读顺序：Knowledge → 宪法 → ADR → 代码。
