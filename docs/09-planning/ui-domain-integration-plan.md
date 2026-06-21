---
id: 09-planning.ui-domain-integration-plan
title: UI→Domain 接线与集成实施计划
status: active
owner: feature-developer
created: 2026-06-21
updated: 2026-06-21
tags:
  - ui
  - integration
  - command
  - projection
  - domain
---

# UI→Domain 接线与集成实施计划

## 目标

让 UI 层真正对接 Domain 层，实现"点击按钮→Domain 执行→ViewModel 更新→UI 刷新"的完整闭环。

## 当前状态

- ✅ UiPlugin 已在 AppPlugin Phase 11 注册
- ✅ UiCommand 枚举 17 个变体
- ✅ UiCommand→GameCommand 转换器
- ✅ BattleProjection 基础监听（BattleStarted, TurnStarted, TurnEnded, EffectApplied）
- ✅ BattleHudVm hp/mp/ap 实时数据投影（ActionPoints 组件）
- ✅ CharacterPanelVm 投影（TurnStarted → 角色面板更新）
- ✅ EconomyProjection 骨架 TODO 完善
- ✅ process_ui_commands 接线系统完成（Round 1a + 1b）
- 🟡 技能面板冷却数据尚未接入（EffectApplied → SkillPanelVm）
- 🟡 角色属性（HP/MP/Level）尚无专属域组件

## 任务

| # | 任务 | 工作量 | 前置 | 说明 | 状态 |
|---|------|--------|------|------|------|
| A1 | UiCommand→GameCommand 接线系统 | 1 天 | 无 | 创建 process_ui_commands 系统，监听 UiCommand→CommandQueue | ✅ 完成 |
| A2 | BattleProjection 基础投影 | 1 天 | 无 | 监听 BattleStarted/TurnEnded，正确更新 ViewModel | ✅ 完成 |
| A2b | BattleProjection 数据增强 | 1 天 | A2 | BattleHudVm hp/mp/ap 实时数据 + CharacterPanelVm 投影 | ✅ 完成 |
| A3 | 端到端验证 | 0.5 天 | A1-A2b | cargo check + nextest 验证完整链路 | ✅ 完成 |

**总计**: ~3.5 天
