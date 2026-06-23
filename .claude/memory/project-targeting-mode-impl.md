---
name: project-targeting-mode-impl
description: 目标选择模式子状态机实现（PickContext + BattleHudVm.targeting_mode）
metadata:
  type: project
  updated: 2026-06-23
---

# Targeting Mode 目标选择模式实现

目标选择模式是 BattleScreen 的 UI 子状态，实现了 Attack → 选目标 → 确认的交互循环。

## 架构

- **状态机**: `PickContext` 作为资源状态机（Normal → AttackTargeting）
- **ViewModel**: `BattleHudVm.targeting_mode: TargetingMode`（None | Attack）
- **交互流**: ActionMenu Attack 按钮 → PickContext::AttackTargeting → grid click 选目标 → bridge.rs 发 `UiCommand::Attack`

## 关键文件

| 文件 | 职责 |
|------|------|
| `src/ui/selection/bridge.rs` | 中心路由器：PickContext → handle_targeting_commit |
| `src/ui/widgets/action_menu/systems.rs` | Attack按钮切换目标模式，Cancel退出 |
| `src/ui/view_models/battle_hud.rs` | TargetingMode枚举 + field |
| `src/ui/widgets/action_menu/components.rs` | ActionType::Cancel 变体 |

## 状态转换

```
AttackButton click → TargetingMode::Attack → PickContext::AttackTargeting
  → Valid unit click → handle_targeting_commit → UiCommand::Attack{attacker_id, target_id}
  → Right-click / Cancel button → 退出目标模式（不清除选中）
  → Invalid target / self click → 忽略
```

**Why:** 目标选择是 SRPG 核心交互循环的第二大环节（选中→选目标→确认）。原设计是 domain 层的 `GameCommand::SelectTarget`，但 presentation-architect 评估后确认目标选择是纯 UI 子状态，domain 不参与中间状态。通过 PickContext 状态机实现。

**How to apply:**
- 新交互模式（如技能选范围）通过扩充 `TargetingMode` 枚举 + `PickContext` 变体实现
- `bridge.rs` 的 `handle_*` 函数按 `PickContext` 分发事件
- 不要将 UI 子状态泄漏到 domain 层
