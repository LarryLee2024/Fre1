# UI 模块测试评审意见

## §1 范围

- 模块路径: `src/ui/` (含 `panels/` 和 `widgets/` 子模块)
- 文件数: 26
- 已有测试: **10** (3 个文件含 inline `#[cfg(test)]`)
- 测试覆盖率: **约 12%** (10 个测试 / ~80 可测试函数)

## §2 评审标准

依据 `docs/test_spec.md` (Testing Constitution v3.1) + `domain_rules.md` + `architecture.md` 进行评审。

### 2.1 架构约束

| 约束 | 值 | 说明 |
|------|-----|------|
| Bevy 版本 | 0.18.1 | ECS 架构 |
| 测试框架 | 项目级 `test_harness` | 标准测试工具 |
| 模块类型 | UI 表现层 | egui 面板 + Bevy UI |
| 架构原则 | Logic 发消息，Presentation 响应 | UI 不直接修改游戏状态 |

### 2.2 非目标 (§1.1)

`test_spec.md` §1.1 排除:
> **UI视觉与动画测试** — 不验证像素输出、动画曲线、CSS 布局、主题颜色。

这意味着:
- ✅ **可测试**: 纯状态机逻辑、数据转换函数、ViewModel 构建、命令映射
- ❌ **不可测试**: Bevy UI 渲染输出、布局计算、颜色视觉呈现

## §3 详细评审结果

### 3.1 已有测试 (10 个)

| 文件 | 测试数 | 内容 |
|------|--------|------|
| `events.rs` | 4 | UiCommand 各变体构造与字段验证 |
| `settings.rs` | 3 | GameSettings 默认值、RON 序列化往返、ColorBlindMode 枚举 |
| `theme.rs` | 3 | faction_color 阵营映射、玩家蓝色系、敌方红色系 |

### 3.2 缺失测试清单

#### P0 - 必须实现

| 测试 | 类型 | 说明 |
|------|------|------|
| `test_clamp_camera_to_map_boundary` | 单元 | 验证相机边界钳制逻辑 (纯函数 `clamp_camera_to_map`) |
| `test_player_turn_detection` | 单元 | 验证 `player_turn()` 检测当前阵营 |
| `test_update_acted_unit_color_saturation` | 单元 | 验证已行动单位颜色变灰 (HSL 饱和度×0.2) |
| `test_update_ui_focus_state_blocks_input` | 单元 | 验证 `BlocksGameInput` 面板存在时 `blocks_input=true` |
| `test_faction_log_color_mapping` | 单元 | 验证 `combat_log_handler::faction_log_color` 映射 |
| `test_damage_popup_config_defaults` | 单元 | 验证 `DamagePopupConfig::default()` 值 |
| `test_update_damage_popups_fade_logic` | 单元 | 验证淡出比例计算 `(ratio - fade_start) / (1 - fade_start)` |
| `test_update_damage_popups_timeout_despawn` | 单元 | 验证超时后实体移除 |
| `test_despawn_popup_returns_none` | 单元 | 验证 `despawn_popup` 返回 `None` |
| `test_view_model_update_selected_unit_view` | 单元 | 验证 `update_selected_unit_view` 从游戏组件构建 ViewModel |
| `test_view_model_update_turn_info_view` | 单元 | 验证 `update_turn_info_view` 构建行动顺序列表 |
| `test_view_model_update_game_over_state` | 单元 | 验证胜负判定: 无敌人→胜利, 无玩家→失败 |

#### P1 - 推荐实现

| 测试 | 类型 | 说明 |
|------|------|------|
| `test_camera_smooth_move_lerp` | 单元 | 验证插值公式 `1 - exp(-speed * dt)` |
| `test_camera_focus_on_unit_turn_change` | 单元 | 验证回合切换时自动聚焦 |
| `test_show_attack_range_bounds` | 单元 | 验证范围生成在地图边界内 |
| `test_action_menu_spawn_despawn` | 单元 | 验证菜单生成与安全销毁 |
| `test_combat_log_handler_damage_format` | 单元 | 验证伤害日志格式化输出 |
| `test_combat_log_handler_heal_format` | 单元 | 验证治疗日志格式化输出 |
| `test_combat_log_handler_stun_format` | 单元 | 验证晕眩日志格式化输出 |
| `test_combat_log_handler_dot_format` | 单元 | 验证 DoT 日志格式化输出 |
| `test_combat_log_handler_hot_format` | 单元 | 验证 HoT 日志格式化输出 |
| `test_combat_log_handler_equip_format` | 单元 | 验证装备穿戴/脱卸日志格式化 |
| `test_spawn_popup_offset_calculation` | 单元 | 验证浮窗偏移量计算 |
| `test_layout_vbox_hbox_panel` | 单元 | 验证布局 Widget 函数返回正确 Node |
| `test_layout_divider_dimensions` | 单元 | 验证分隔线宽度/高度 |
| `test_layout_label_tuple` | 单元 | 验证 label 函数返回 (Text, TextFont, TextColor) |
| `test_resource_bar_spawn_structure` | 单元 | 验证资源条 Widget 生成正确子节点 |

#### P2 - 可选

| 测试 | 类型 | 说明 |
|------|------|------|
| `test_camera_edge_scroll_direction` | 单元 | 验证边缘滚动方向计算 |
| `test_camera_zoom_clamp` | 单元 | 验证缩放值钳制到 [0.3, 3.0] |
| `test_update_turn_indicator_format` | 单元 | 验证回合文本格式 "第 N 回合" |
| `test_check_game_over_state_transition` | 单元 | 验证胜负状态切换 |
| `test_inventory_panel_update_format` | 单元 | 验证背包面板文本格式化 |
| `test_combat_preview_lethal_color` | 单元 | 验证致命一击使用 crit_color |
| `test_highlight_selection_highlight_size` | 单元 | 验证选中高亮尺寸 75% tile |
| `test_vfx_damage_popup_text_format` | 单元 | 验证暴击文本 "N!" 格式 |

## §4 不变量覆盖率

### 4.1 核心不变量

| 不变量 | 测试 | 说明 |
|--------|------|------|
| `UiCommand` 各变体可构造 | ✅ | events.rs 已覆盖 |
| `GameSettings` 默认值合理 | ✅ | settings.rs 已覆盖 |
| `GameSettings` RON 序列化往返 | ✅ | settings.rs 已覆盖 |
| `ColorBlindMode` 枚举完整 | ✅ | settings.rs 已覆盖 |
| `faction_color` 阵营映射 | ✅ | theme.rs 已覆盖 |
| `clamp_camera_to_map` 边界钳制 | ❌ | 纯函数，未测试 |
| `player_turn` 阵营检测 | ❌ | 纯函数，未测试 |
| `update_acted_unit_color` 颜色变换 | ❌ | 纯逻辑，未测试 |
| `update_ui_focus_state` 焦点状态 | ❌ | 纯状态更新，未测试 |
| `faction_log_color` 映射 | ❌ | 纯函数，未测试 |
| `DamagePopupConfig` 默认值 | ❌ | 纯构造，未测试 |
| `update_damage_popups` 淡出逻辑 | ❌ | 纯数学，未测试 |
| `despawn_popup` 返回 None | ❌ | 纯函数，未测试 |
| ViewModel 构建逻辑 | ❌ | 纯数据转换，未测试 |
| `GameOverState` 胜负判定 | ❌ | 纯逻辑，未测试 |
| `combat_log_handler` 日志格式 | ❌ | 纯格式化，未测试 |
| `show_attack_range` 范围生成 | ❌ | 纯计算，未测试 |
| `show_move_range` 范围生成 | ❌ | 纯计算，未测试 |
| `clear_selection` 逻辑 | ❌ | 纯操作，未测试 |
| `layout` Widget 函数 | ❌ | 纯构造，未测试 |
| `resource_bar` Widget | ❌ | 纯构造，未测试 |

### 4.2 可测试的纯逻辑

模块中存在大量可测试的纯逻辑:

1. **相机控制**: `clamp_camera_to_map()` (纯函数), `camera_smooth_move()` 插值公式, 缩放钳制
2. **ViewModel 更新**: `update_selected_unit_view()` 数据转换, `update_turn_info_view()` 列表构建, `update_game_over_state()` 胜负判定
3. **战斗日志**: 8 个 handler 函数的格式化逻辑 (纯字符串拼接)
4. **特效**: `DamagePopupConfig` 默认值, 淡出比例计算, 超时判定
5. **焦点管理**: `update_ui_focus_state()` 布尔更新
6. **高亮**: `show_attack_range()` 范围生成, `show_move_range()` 范围生成
7. **Widget**: `vbox()`/`hbox()`/`panel()`/`label()`/`divider()` 返回值
8. **浮窗**: `despawn_popup()` 返回 None
9. **资源条**: `spawn_resource_bar()` 结构验证

## §5 测试金字塔

| 层级 | 期望 | 实际 | 状态 |
|------|------|------|------|
| 单元测试 | 高 | 10 | ❌ |
| 集成测试 | 中 | 0 | ❌ |
| 回放测试 | 最高 | 0 | ❌ |

**缺陷**: 单元测试严重不足。10 个测试仅覆盖 3 个文件 (events, settings, theme)，其余 23 个文件零测试。

## §6 确定性

- 所有纯逻辑函数均为确定性
- Bevy UI 渲染函数依赖 ECS World 状态，非确定性

## §7 Schema 合规

### §7.1 AI Self-Check (§13.1)
- 已测试文件 (3): **缺失** ❌
- 未测试文件 (23): N/A

### §7.2 Test IDs (§13.2)
- 所有测试: **缺失** ❌

### §7.3 Given/When/Then (§5.1)
- 所有测试: **缺失** ❌

## §8 代码质量

### 8.1 优点
- **架构清晰**: UI 层遵循 "Logic 发消息，Presentation 响应" 原则
- **ViewModel 分离**: `view_models.rs` 将游戏数据转换为 UI 数据，职责清晰
- **可复用 Widget**: `widgets/` 提供 layout/popup/resource_bar 构建块
- **主题系统**: `UiTheme` 集中管理颜色/字号/间距
- **事件驱动**: `UiCommand` Message 模式，UI 不直接操作 ECS

### 8.2 问题
- **测试覆盖不足**: 10 个测试 / ~80 可测试函数 = 12%
- **无 AI Self-Check**: 所有文件缺失 §13.1 标注
- **无 Test IDs**: 所有测试缺失 §13.2 标注
- **无 Given/When/Then**: 所有测试缺失 §5.1 格式
- **ViewModel 逻辑未测试**: `view_models.rs` (476 行) 是模块最大文件，含复杂数据转换逻辑，零测试

### 8.3 关键未测试模块

| 模块 | 行数 | 可测试逻辑 | 说明 |
|------|------|------------|------|
| `view_models.rs` | 476 | ViewModel 构建、胜负判定 | 最大文件，核心数据转换 |
| `command_handler.rs` | 317 | 命令映射、状态转换 | 核心交互逻辑 |
| `camera.rs` | 227 | 边界钳制、插值、聚焦 | 纯计算逻辑 |
| `combat_log_handler.rs` | 233 | 日志格式化 (8 个 handler) | 纯字符串拼接 |
| `highlight.rs` | 145 | 范围生成、选中高亮 | 纯计算逻辑 |
| `action_menu.rs` | 213 | 菜单生成、交互处理 | 状态机逻辑 |
| `unit_info.rs` | 436 | 面板文本更新 | 数据格式化 |
| `theme.rs` | 203 | 默认值、阵营映射 | 已有 3 测试 |

## §9 问题统计

| 类别 | 数量 | 严重性 |
|------|------|--------|
| 缺失单元测试 | 30+ | P0 |
| 缺失 AI Self-Check | 26 文件 | P0 |
| 缺失 Test IDs | 10 测试 | P0 |
| 缺失 Given/When/Then | 10 测试 | P0 |
| ViewModel 逻辑未测试 | 1 模块 | P0 |
| 命令处理逻辑未测试 | 1 模块 | P1 |
| 相机控制逻辑未测试 | 1 模块 | P1 |
| 战斗日志格式未测试 | 8 函数 | P1 |
| Widget 函数未测试 | 10+ 函数 | P1 |

## §10 优先级

### P0 - 立即修复
1. 添加 `test_clamp_camera_to_map_boundary` 单元测试
2. 添加 `test_player_turn_detection` 单元测试
3. 添加 `test_update_acted_unit_color_saturation` 单元测试
4. 添加 `test_update_ui_focus_state_blocks_input` 单元测试
5. 添加 `test_faction_log_color_mapping` 单元测试
6. 添加 `test_damage_popup_config_defaults` 单元测试
7. 添加 `test_update_damage_popups_fade_logic` 单元测试
8. 添加 `test_update_damage_popups_timeout_despawn` 单元测试
9. 添加 `test_despawn_popup_returns_none` 单元测试
10. 添加 `test_view_model_update_selected_unit_view` 单元测试
11. 添加 `test_view_model_update_turn_info_view` 单元测试
12. 添加 `test_view_model_update_game_over_state` 单元测试
13. 为所有文件添加 AI Self-Check 标注

### P1 - 后续
14. 添加 `test_camera_smooth_move_lerp` 单元测试
15. 添加 `test_show_attack_range_bounds` 单元测试
16. 添加 `test_action_menu_spawn_despawn` 单元测试
17. 添加 8 个 `test_combat_log_handler_*_format` 单元测试
18. 添加 5 个 Widget 函数单元测试

### P2 - 可选
19. 添加相机边缘滚动/缩放钳制测试
20. 添加面板文本格式化测试

## §11 总结

| 指标 | 值 |
|------|-----|
| 总分 | **2.0 / 5.0** |
| 已有测试 | 10 |
| 缺失测试 | 30+ |
| P0 问题 | 13 |
| P1 问题 | 5 |
| P2 问题 | 2 |
| 代码质量 | 中等 (架构好，测试差) |
| 确定性 | 高 (纯逻辑多) |
| 架构 | 良好 (ViewModel + Message 模式) |

**结论**: UI 模块架构设计良好，遵循 "Logic 发消息，Presentation 响应" 原则，ViewModel 分离清晰。但测试严重不足: 26 个文件中仅 3 个有测试，10 个测试覆盖约 12% 的可测试逻辑。`view_models.rs` (476 行) 是模块最大文件，含复杂数据转换逻辑，零测试覆盖是不可接受的。

## §12 AI Self-Check

```
AI-Self-Check: PASS
- 测试评审: 完成
- 文件完整性: 26/26 文件已检查
- 测试覆盖: 10/~80 可测试函数 (12%)
- 缺陷识别: 13 P0 + 5 P1 + 2 P2
```
