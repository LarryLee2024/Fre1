---
name: decision-capability-not-service
description: 能力系统采用 Capability 模式而非 Service 模式
metadata:
  type: decision
---

# 能力系统采用 Capability 而非 Service

**日期**: 2026-06
**关联**: ADR-010, ADR-046
**待升迁**: 否（已由 ADR 覆盖）

## 决策

能力系统采用 Capability 架构（Foundation → Mechanism → Runtime），而非传统的 Service/Manager 模式。

## 原因

1. ECS 组合优于继承——Capability 是 Component + System 的打包单位
2. Service 模式在 Bevy ECS 中会导致 System 参数爆炸
3. Capability 的三层结构（F→M→R）比 Service 的平面结构更容易测试

## 被否决的替代方案

- Service 模式：导致 God Service 问题
- Pure Function 库：缺少生命周期管理，无法处理跨帧操作

## 后果

- 每个 Capability 自包含，Plugin 是唯一对外入口
- 新增 Capability 成本固定
- 跨 Capability 通信必须通过 Event/Trigger
