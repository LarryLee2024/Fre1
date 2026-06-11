# Debug 模块测试评审意见

## §1 范围

- 模块路径: `src/debug/` (含 `viewers/` 子模块)
- 文件数: 15
- 总代码量: 约 1500 行 (100% UI 可视化代码)
- 已有测试: **0**
- 测试覆盖率: **0%**

## §2 测试评审标准

依据 `docs/test_spec.md` (Testing Constitution v3.1) + `domain_rules.md` + `architecture.md` 进行评审。

### 2.1 架构约束

| 约束 | 值 | 说明 |
|------|-----|------|
| Bevy 版本 | 0.18.1 | ECS 架构 |
| 测试框架 | 项目级 `test_harness` | 标准测试工具 |
| 快照工具 | `insta` | v1.x |
| Mock 框架 | `mockall` | v0.13 |
| 异步运行时 | `tokio` | mpsc channel |
| 模块类型 | UI 可视化 | egui 面板 + gizmos |

### 2.2 非目标 (§1.1)

`test_spec.md` §1.1 明确排除:
> **UI视觉与动画测试** — 不验证像素输出、动画曲线、CSS 布局、主题颜色。

这意味着:
- ✅ **可测试**: 纯状态机逻辑、数据转换函数、布尔开关
- ❌ **不可测试**: egui 渲染输出、gizmos 视觉呈现、颜色/布局

## §3 不变量覆盖率

### 3.1 核心不变量

| 不变量 | 测试 | 说明 |
|--------|------|------|
| DebugPanelState.toggle_*() | ❌ | bool 翻转逻辑 |
| DebugOverlay.F3 全开/全关 | ❌ | 四个 bool 联动 |
| GridViewerState 分页 | ❌ | scroll_row/page_rows 纯计算 |
| render_damage_panel 输出 | ❌ | 过滤/take 20 |
| render_attribute_panel 分组 | ❌ | 按 kind 分组 |
| equipment_viewer 按槽位分组 | ❌ | 纯数据转换 |

### 3.2 可测试的纯逻辑

虽然整个模块是 UI 代码，但以下逻辑可作为纯状态机测试:

```rust
// 可测试: DebugPanelState.toggle_ai() 翻转
// 可测试: DebugOverlay F3 全开/全关逻辑
// 可测试: GridViewerState 分页计算
// 可测试: render_damage_panel 过滤逻辑
// 可测试: render_attribute_panel 分组逻辑
// 可测试: equipment_viewer 按槽位分组
```

## §4 测试金字塔

| 层级 | 期望 | 实际 | 状态 |
|------|------|------|------|
| 单元测试 | 高 | 0 | ❌ |
| 集成测试 | 中 | 0 | ❌ |
| 回放测试 | 最高 | 0 | ❌ |

**严重缺陷**: 整个 debug 模块没有任何测试。

## §5 确定性

- 所有纯逻辑函数均为确定性
- UI 渲染函数依赖 egui 状态，非确定性

## §6 Schema 合规

### §6.1 AI Self-Check (§13.1)
- 所有文件: **缺失** ❌

### §6.2 Test IDs (§13.2)
- 所有测试: **缺失** ❌

### §6.3 Given/When/Then (§5.1)
- 无测试可评估

## §7 缺失测试清单

### P0 - 必须实现

| 测试 | 类型 | 说明 |
|------|------|------|
| `test_debug_panel_state_toggle` | 单元 | 验证 toggle_ai() 翻转 |
| `test_debug_overlay_f3_all_on_off` | 单元 | 验证 F3 全开/全关逻辑 |
| `test_grid_viewer_state_pagination` | 单元 | 验证 scroll_row/page_rows 计算 |
| `test_render_damage_panel_filter` | 单元 | 验证过/take 20 |
| `test_render_attribute_panel_grouping` | 单元 | 验证按 kind 分组 |

### P1 - 推荐实现

| 测试 | 类型 | 说明 |
|------|------|------|
| `test_equipment_viewer_slot_grouping` | 单元 | 验证按槽位分组 |
| `test_hotkey_system_state_transitions` | 单元 | 验证 F1/F2/F4/F5 状态转换 |

### P2 - 可选

| 测试 | 类型 | 说明 |
|------|------|------|
| `test_debug_plugin_system_registration` | 集成 | 验证 DebugPlugin 注册所有系统 |

## §8 代码质量

### 8.1 优点
- 纯状态机逻辑清晰
- 无外部依赖（纯数据转换）
- 无副作用（除 UI 渲染外）

### 8.2 问题
- **零测试覆盖**: 整个模块无任何测试
- **无 AI Self-Check**: 所有文件缺失 §13.1 标注
- **无 Test IDs**: 所有测试缺失 §13.2 标注
- **无 Given/When/Then**: 无测试可评估

### 8.3 架构问题

`DebugPlugin` 注册了大量系统，但:
- 系统间无测试验证交互
- 条件渲染逻辑（`if overlay.show_*`）无测试
- 热键处理无状态转换测试

## §9 问题统计

| 类别 | 数量 | 严重性 |
|------|------|--------|
| 零测试覆盖 | 1 | P0 |
| 缺失 AI Self-Check | 15 文件 | P0 |
| 缺失 Test IDs | 0 (无测试) | N/A |
| 缺失 Given/When/Then | 0 (无测试) | N/A |
| 可测试纯逻辑未测试 | 6 | P0 |
| UI 渲染不可测试 | 8+ | N/A (非目标) |

## §10 优先级

### P0 - 立即修复
1. 添加 `test_debug_panel_state_toggle` 单元测试
2. 添加 `test_debug_overlay_f3_all_on_off` 单元测试
3. 添加 `test_grid_viewer_state_pagination` 单元测试
4. 添加 `test_render_damage_panel_filter` 单元测试
5. 添加 `test_render_attribute_panel_grouping` 单元测试
6. 为所有文件添加 AI Self-Check 标注

### P1 - 后续
7. 添加 `test_equipment_viewer_slot_grouping` 单元测试
8. 添加 `test_hotkey_system_state_transitions` 单元测试

### P2 - 可选
9. 添加 `test_debug_plugin_system_registration` 集成测试

## §11 总结

| 指标 | 值 |
|------|-----|
| 总分 | **0.5 / 5.0** |
| 已有测试 | 0 |
| 缺失测试 | 6+ |
| P0 问题 | 6 |
| P1 问题 | 2 |
| P2 问题 | 1 |
| 代码质量 | 差 (零覆盖) |
| 确定性 | 高 (纯状态机) |
| 架构 | 合理 (ECS 系统) |

**结论**: Debug 模块是纯 UI 可视化代码，`test_spec.md` §1.1 排除 "UI视觉与动画测试"。但模块中存在 6 处可测试的纯状态机逻辑（DebugPanelState toggle、DebugOverlay F3 联动、GridViewerState 分页、render_damage_panel 过滤、render_attribute_panel 分组、equipment_viewer 槽位分组），这些应添加单元测试。当前零测试覆盖是不可接受的。

## §12 AI Self-Check

```
AI-Self-Check: PASS
- 测试评审: 完成
- 文件完整性: 15/15 文件已检查
- 测试覆盖: 0/6 可测试逻辑已覆盖
- 缺陷识别: 6 P0 + 2 P1 + 1 P2
```
