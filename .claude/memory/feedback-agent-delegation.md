---
name: feedback-agent-delegation
description: 复杂多步骤任务应派发专用 agent 并行处理
metadata:
  type: feedback
---

# 复杂任务优先派发 Agent

用户习惯用"安排相关智能体继续完善"来指示多步骤修复。不应自己逐一手动修复。

**Why**: 11 个 agent 并行工作比单线程手动修复快 5-10 倍，且互不阻塞。

**How to apply**: 收到批量修复/完善需求时，立即拆分为独立子任务，派发多个 feature-developer agent 并行执行。每个 agent 的 scope 要小且独立（不重叠文件）。自己同步处理文档更新等轻量工作。
