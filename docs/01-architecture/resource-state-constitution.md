---
id: RESOURCE-STATE-CONSTITUTION
title: 资源与状态机宪法
status: accepted
stability: stable
layer: architecture
related:
  - ai-constitution-complete.md
tags:
  - resource
  - state-machine
  - lifecycle
---

> **原文来源**：`ai-constitution-complete.md` 第十三编（L1292-L1300）、第十五编（L1332-L1338）
>
> **锚定总宪法**：第十三编、第十五编

## 第十三编 资源与内容生产宪法
- 🟩 所有游戏设置必须通过统一的 Settings 体系管理
- 🟩 所有资源加载必须可追踪
- 🟩 所有资源的生命周期必须显式管理
- 🟩 所有资源必须分类统一管理
- 🟩 高频修改的资源必须优先支持热重载
- 🟩 编辑器是正式产品的一部分，不是开发工具
- 🟩 内容生产能力决定项目上限，工具链是长期项目的核心资产

---

## 第十五编 生命周期与状态机宪法
- 🟩 `OnEnter` 和 `OnExit` 系统必须保持轻量
- 🟩 重型初始化逻辑必须拆分成多个加载阶段
- 🟩 状态切换时 🟥 绝对禁止隐藏副作用
- 🟩 状态机只负责流程控制，🟥 绝对禁止包含业务细节
- 🟩 初始化过程必须可追踪、可恢复、可中断
