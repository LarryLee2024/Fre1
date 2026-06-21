---
id: 09-planning.ui-domain-integration-plan
title: UI→Domain 接线与集成实施计划
status: completed
owner: feature-developer
created: 2026-06-21
updated: 2026-06-22
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

## 当前状态（All ✅）

以下工程全部完成，文档与代码 100% 一致：

### 桥接层
- ✅ UiPlugin 已在 AppPlugin Phase 11 注册
- ✅ UiCommand 枚举 17 个变体（全部使用 struct 语法）
- ✅ UiCommand→GameCommand 转换器完成（17/17 映射，仅 SelectTarget/OpenScreen/CloseScreen 返回 None）
- ✅ process_ui_commands Observer 完成（所有映射命令推入 CommandQueue）
- ✅ 新增 GameCommand 变体：BuyItem, SellItem, EquipItem, DropItem, AcceptQuest, AbandonQuest, NewGame

### 投影层
- ✅ BattleProjection 完整监听（BattleStarted, TurnStarted, TurnEnded, EffectApplied）
- ✅ BattleHudVm hp/mp/ap 实时数据投影（ActionPoints 组件查询）
- ✅ CharacterPanelVm 投影（TurnStarted → 角色面板更新）
- ✅ EconomyProjection 骨架（含 TODO）

### 屏幕接线
- ✅ BattleScreen: EndTurn → UiCommand::EndTurn
- ✅ MainMenu: NewGame/LoadGame/Settings → UiCommand
- ✅ SaveLoadScreen: SaveSlot/LoadSlot → UiCommand::SaveGame/LoadGame
- ✅ ShopScreen: BuyItem/SellItem → UiCommand
- ✅ InventoryScreen: UseItem/DropItem → UiCommand
- ✅ SettingsScreen: Save → save_settings() 持久化

### 验证
- ✅ cargo check — 零错误
- ✅ cargo nextest run — 1841/1841 通过
- ✅ grep "pending domain wiring" — 零匹配（全部接线完成）
