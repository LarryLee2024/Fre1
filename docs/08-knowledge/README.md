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
| 错误处理 | [错误处理架构深度解析](error-handling-overview.md) | Error vs Failure 严格分离：架构设计、15 域迁移完成、1579 测试全绿 |
| ID 系统 | [强类型 ID 系统深度解析](ids-overview.md) | 类型安全的标识符策略：String ID vs Numeric ID、宏详解、22 个类型 |
| 国际化 | [Localization 国际化深度解析](localization-overview.md) | 从宪法到代码：5 层约束体系、Fluent .ftl、Key 代码生成、三级回退链、Fake Locale |
| Reflect | [Bevy Reflect 深度解析](reflect-overview.md) | 运行时类型反射原理、宪法三规则、类型注册三件套、Save/Replay 中的角色、高频计算禁令 |
| 系统间通信 | [系统间通信深度解析](communication-overview.md) | 四级通信（Hook/Trigger/Observer/Message）、CommandQueue 命令系统、UI Action 模式、Integration 桥梁、日志 Observer 56 监听器、三条端到端数据流 |
| GAS-Lite 能力系统 | [GAS-Lite 能力系统深度解析](capabilities-overview.md) | 15 个能力模块全景、三端到端数据流、Def→Spec→Instance 分离、宪法约束 Data Laws #004-#009 |
| Pipeline 管线系统 | [Pipeline 管线系统深度解析](pipeline-overview.md) | 通用 Pipeline 引擎设计、四条业务管线（Ability / Modifier / Combat / Content）、驾驶员模式、全协作时序图 |
| UI 表现层 | [UI 表现层深度解析](ui-overview.md) | 三层渲染栈（Primitives→Widgets→Screens）、五条铁律、Theme 令牌系统、Observer 事件路由、工厂模式、完整四层数据流设计、当前实现状态一览 |
| Replay 回放 | [Replay 回放系统深度解析](replay-overview.md) | 从场景故事到三层架构、录制/回放全流程、确定性 RNG 四流、Combat 桥接层、86 个测试、ADR 决策索引 |
| 随机数系统 | [随机数系统深度解析](random-overview.md) | 三套 RNG 系统（SeededRng/GameRng/DeterministicRng）、四流隔离设计、MurmurHash3 算法、回放同步机制、迁移路线图 |

## 定位

Knowledge Base（`docs/08-knowledge/`）与 Architecture（`docs/01-architecture/`）不同：

- **Architecture** 是规范——告诉你要怎么做、不能怎么做
- **Knowledge** 是解释——告诉你为什么这么做、代码怎么跑起来的

新成员阅读顺序：Knowledge → 宪法 → ADR → 代码。
