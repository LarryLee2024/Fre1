---
name: project-ui-review-2026-06-22
description: UI 设计-代码偏移评审及 11 agent 批量修复（2026-06-22）
metadata:
  type: project
---

# UI 设计-代码偏移评审与批量修复

## 背景

presentation-architect 对 `docs/06-ui/` 设计文档与 `src/ui/` 代码做了全面评审，输出 `docs/10-reviews/ui-design-code-drift-review.md`。发现 P0 违规 4 项、P1 问题 4 项、P2 问题 9 项。

## 修复成果（11 agents，4 轮）

### 架构违规修复
- **SaveLoadScreen 工厂违规**: 新增 `PanelVariant::Placeholder` 变体，替换2处直接 `commands.spawn`
- **BattleScreen 硬编码数据**: 创建 `BattleHudData` Resource，CharacterCard 从 Resource 读取
- **UiBinding 激活**: CharacterCard 携带 `UiBinding::Hp`

### Screen 扩展
- **SettingsScreen**: 2 Toggle → 5 Toggle，分 Gameplay/Display 组
- **SkillPanel (Z7)**: 新建 skill_panel widget，集成 skill_slot 到 BattleScreen
- **PhaseText (Z2)**: 阶段文本+回合数，三语言 FTL 支持
- **TurnOrderBar (Z8)**: 新建 turn_order_bar widget，完成 9-zone 布局

### 基础设施
- **UiStore 扩展**: 新增 InventoryVm/ShopPanelVm/EconomyVm
- **EconomyProjection**: 激活，监听 CurrencyChanged 事件更新 EconomyVm
- **BuffIcon 增强**: BuffType 枚举+叠加层数+呼吸动画
- **CharacterPortrait**: 从头像 Widget（PortraitBorder 4 状态）

### 文档修正
- architecture.md §6.5: 铁律5添加豁免条款
- theme-localization.md §4.4: 明确 Primitives Text::new 豁免
- widget-composites.md §7: 目录结构强制→建议
- docs/06-ui/README.md: 状态表更新

## BattleScreen 9-zone 最终状态

Z1 TurnInfo ✅ | Z2 PhaseText+TurnNum ✅ | Z3 UnitSummary ⏳ | Z5 CharacterCard ✅ | Z6 ActionMenu ✅ | Z7 SkillPanel ✅ | Z8 TurnOrderBar ✅

## 仍待修复

- P0-3: Dirty<T> 消费链完整接入（需 Projection 就绪）
- P2-1/2: ScreenStack 导航激活（架构决策）
- P2-4: 剩余 7 个复合组件（BattleHud, CharacterStatusPanel 等）
- P3: Theme RON 配置、Pixel/HD2D 变体

**Why**: 评审指出代码与设计文档存在多处偏移，需集中修复避免架构腐化。

**How to apply**: 参考 `docs/10-reviews/ui-design-code-drift-review.md` §7 修复状态表，优先修 P0 再 P1。
