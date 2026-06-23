---
name: planning-docs-fully-archived
description: docs/09-planning/ 全部 44 份文档已完成归档，无活跃规划
metadata:
  type: project
  updated: 2026-06-23
---

# Planning Docs 全部归档

`docs/09-planning/` 当前状态：

- 根目录仅保留 `README.md`
- `done/` 包含 **44 份**已归档规划文档
- 最后归档：`ui-layout-system-plan.md`（2026-06-23）

## ui-layout-system-plan 归档最终状态

**P0 + P1 全部实现（共 10 项）：**

| 项目 | 实现位置 |
|------|----------|
| UiSizing Resource | `src/ui/theme/sizing.rs` |
| 9-zone Absolute 定位 | `src/ui/screens/battle/layout.rs` (7 区) |
| Visibility 矩阵 | `src/ui/screens/battle/visibility.rs` |
| TurnIndicator Widget | `src/ui/widgets/turn_indicator/` |
| SkillPanel 切换 | `battle_hud.rs` skill_panel_open |
| 目标选择模式 | `bridge.rs` + `BattleHudVm.targeting_mode` |
| Victory/Defeat | `projections/battle.rs` on_battle_ended_projection |
| EndTurn 动作链 | `button → UiCommand::EndTurn → CommandQueue → domain` |
| Zone 守卫（空状态） | Z5 current_unit_id != 0, Z6/Z7 player_controlled |
| TurnOrderBar 骨架 | `src/ui/widgets/turn_order_bar/`（Z8 区容器已建） |

**P2 已 Defer（共 5 项）：**
- 响应式布局断点
- CharacterCard 紧凑变体
- UnitSummary 区（Z3）内容
- 动画
- TurnOrderBar 水平滚动

**Why:** 这是规划文档目录的彻底清理。此后进入新功能开发周期，不再有挂起的策划文档需要处理。

**How to apply:**
- 新规划文档直接放入 `docs/09-planning/`，完成后移入 `done/`
- P2 项目如果启动（例如响应式布局），需新开规划文档
- `ui-layout-system-plan.md` 仍在 `done/` 中可作为 Zone Layout 架构参考
