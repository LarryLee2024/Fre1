---
name: project-playable-combat-loop
description: 可玩战斗循环实现完成（敌方AI + 胜负状态转换 + 3集成测试 + 1 bug修复）
metadata:
  type: project
  updated: 2026-06-23
---

# 可玩战斗循环实现

## 完成内容

| 步骤 | 实现 | 文件 |
|------|------|------|
| Step 1 | 简单敌方 AI（team_id=="Player"→暂停，否则→自动 UnitActionComplete） | `combat/pipeline/driver.rs` |
| Step 2 | 战斗结束状态转换（BattleEnded → Result/GameOver） | `app/scenes/battle_end.rs` |
| Step 3 | ResultScreen（胜利标题 + 返回主菜单按钮） | `app/scenes/result/mod.rs` |
| Step 4 | GameOverScreen（Game Over 标题 + 返回主菜单按钮） | `app/scenes/game_over/mod.rs` |
| Step 5 | 集成测试（3 个测试，1909 套件全绿） | `tests/combat_flow.rs` |
| — | Bug 修复：damage_policy 减免计算（calc_mitigation 返回值错误） | `rules/damage_policy.rs` |

## 发现并修复的业务 Bug

`calc_mitigation()` 内部已做 `incoming_damage.saturating_sub(def_to_mitigation(def))`，返回的是"减免后伤害"而非"减免量"。调用方又做了一次减法，导致最终伤害为 0。修复：重命名为 `calc_mitigation_amount()`，直接返回减免量。

## 测试覆盖

- `CMB-FLOW-001`: player_attack_damges_enemy — AttackRequested → HP-10 ✅
- `CMB-FLOW-002`: victory_state_transition — BattleEnded(true) → Result ✅
- `CMB-FLOW-003`: defeat_state_transition — BattleEnded(false) → GameOver ✅

## 游戏循环现状

```
MainMenu → PartySetup → Combat → Result/GameOver → MainMenu
  ✅         ✅           ✅          ✅           ✅
战斗中：玩家行动 ✅ | 敌方自动跳过 ✅ | 胜负判定 ✅ | 结算屏幕 ✅
```

**Why:** 这是 MVP 可玩循环的最后缺失环节。简单敌方 AI（自动结束回合）是最小可行 AI，后续可迭代为真正的决策系统。

**How to apply:**
- 新增 AI 决策逻辑：扩展 `driver.rs` 中的敌方分支，替换 `.trigger(UnitActionComplete)` 为真正的 AI 决策
- 新增技能系统：SkillPanel 内容已存在，但需要技能数据从 content/ 域到 BattleHudVm 的投影
