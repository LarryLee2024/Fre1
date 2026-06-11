# UI 模块测试评审报告

**Version**: 1.0
**Date**: 2026-06-11
**Reviewer**: Test Guardian
**Scope**: `src/ui/` 全部代码文件 + `tests/` 中相关外部测试
**Standard**: `docs/test_spec.md` (Bevy SRPG Testing Constitution v3.1)
**Domain Reference**: `docs/domain/ui_rules_v1.md`

---

# 1. 评审范围

## 1.1 源代码文件

| 文件 | 行数 | 内联测试数 | 测试覆盖状态 |
|------|------|-----------|-------------|
| `mod.rs` | 137 | 0 | N/A（插件注册） |
| `action_menu.rs` | 213 | 0 | **未覆盖** |
| `camera.rs` | 227 | 0 | **未覆盖** |
| `combat_log_handler.rs` | 233 | 0 | **未覆盖** |
| `combat_preview.rs` | 92 | 0 | **未覆盖** |
| `combat_vfx_handler.rs` | 52 | 0 | **未覆盖** |
| `command_handler.rs` | 317 | 0 | **未覆盖** |
| `events.rs` | 80 | 4 | 良好 |
| `focus.rs` | 28 | 0 | **未覆盖** |
| `highlight.rs` | 145 | 0 | **未覆盖** |
| `settings.rs` | 146 | 3 | 良好 |
| `theme.rs` | 203 | 4 | 良好 |
| `tile_info.rs` | 106 | 0 | **未覆盖** |
| `vfx.rs` | 112 | 0 | **未覆盖** |
| `view_models.rs` | 476 | 0 | **未覆盖** |
| `widgets/layout.rs` | 127 | 0 | **未覆盖** |
| `widgets/popup.rs` | 62 | 0 | **未覆盖** |
| `widgets/resource_bar.rs` | 70 | 0 | **未覆盖** |
| `panels/action_hint.rs` | 38 | 0 | N/A（纯 UI） |
| `panels/combat_log_panel.rs` | 125 | 0 | N/A（纯 UI） |
| `panels/inventory_panel.rs` | 97 | 0 | N/A（纯 UI） |
| `panels/turn_indicator.rs` | 85 | 0 | N/A（纯 UI） |
| `panels/unit_info.rs` | 436 | 0 | N/A（纯 UI） |

**内联测试总计：11 个**（已修复为符合规范）

## 1.2 外部测试文件（与 UI 相关）

| 文件 | 测试数 | 覆盖范围 |
|------|--------|----------|
| `tests/feature/equipment.rs` | ~20 | 装备穿脱流程（间接测试 UI 交互） |
| `tests/feature/buff.rs` | ~10 | Buff 生命周期（间接测试 UI 显示） |

**外部测试总计：~30 个**

**全部测试总计：~41 个**

---

# 2. 评审标准

依据 `test_spec.md` 以下条款逐项评审：

| 条款 | 内容 | 评审重点 |
|------|------|----------|
| §3 Testing Philosophy | 测试验证 Behavior，不验证 Implementation | 断言是否关注 What 而非 How |
| §4 Test Pyramid | 70% Unit / 20% Integration / 8% Replay / 2% E2E | 各层级比例是否合理 |
| §5 Test Categories | Unit/Integration/Replay/Regression/E2E 定义 | 是否有缺失类别 |
| §6 Determinism Rules | 禁止随机、固定 Seed | 测试是否确定性 |
| §7 Test Case Schema | Test ID / Title / Given / When / Then / Assertions | 测试结构是否规范 |
| §7.1 Standard Test Data | Unit_001 / Unit_002 / Unit_003 | 是否使用标准数据 |
| §9 Coverage Strategy | 100% 核心领域规则覆盖 | 领域不变量是否全部测试 |
| §10 Error Testing | Invalid Input / Boundary Values | 边界和错误场景是否覆盖 |
| §13 AI Constraints | 禁止测试私有实现 | 是否越界测试内部细节 |
| §13.1 AI Self-Check | 6 项自检标注 | 是否有自检标注 |

---

# 3. 领域不变量覆盖评审

依据 `ui_rules_v1.md` 中定义的 5 个不变量：

| 不变量 | 描述 | 测试覆盖 | 评审结论 |
|--------|------|----------|----------|
| INV-UI-1 | UI 不操作 ECS | **部分覆盖** | `events.rs` 测试验证 UiCommand 定义，但无测试验证 handle_ui_commands 不直接修改 ECS |
| INV-UI-2 | ViewModel 隔离 | **无覆盖** | `view_models.rs` 零测试，无法验证 UI 只读 ViewModel |
| INV-UI-3 | handle_ui_commands 仅玩家回合 | **无覆盖** | `command_handler.rs` 零测试，无法验证玩家回合限制 |
| INV-UI-4 | BlocksGameInput 阻止输入 | **无覆盖** | `focus.rs` 零测试，无法验证焦点管理 |
| INV-UI-5 | UI 不保存业务真相 | **部分覆盖** | ViewModel 是只读投影，但无测试验证 |

**覆盖率：0/5 完全覆盖，2/5 部分覆盖，3/5 无覆盖**

---

# 4. 业务规则覆盖评审

| 规则 | 描述 | 测试覆盖 | 评审结论 |
|------|------|----------|----------|
| BR-UI-1 | UiCommand 是唯一交互通道 | **部分覆盖** | `events.rs` 测试验证命令定义，但无测试验证命令处理 |
| BR-UI-2 | ViewModel 刷新策略 | **无覆盖** | 无测试验证刷新时机 |
| BR-UI-3 | Cancel 上下文推断 | **无覆盖** | 无测试验证 3 种取消场景 |
| BR-UI-4 | 主题统一样式 | **覆盖** | `theme.rs` 测试验证阵营颜色映射 |

**覆盖率：1/4 完全覆盖，1/4 部分覆盖，2/4 无覆盖**

---

# 5. 现有测试质量评审

## 5.1 内联测试逐文件评审

### events.rs（4 个测试）

| 测试 | 验证目标 | Behavior/Implementation | 合规性 |
|------|----------|------------------------|--------|
| `ui_command_variants_constructible` | 变体构造 | Behavior — 编译时验证 | 良好 |
| `ui_command_select_unit_carries_entity` | 字段携带 | **Implementation** — 检查结构体字段 | 轻微违规 |
| `ui_command_skill_carries_skill_id` | 字段携带 | **Implementation** | 轻微违规 |
| `ui_command_move_unit_carries_coord` | 字段携带 | **Implementation** | 轻微违规 |

**问题：** 后 3 个测试仅验证 Message 结构体字段赋值，属于实现细节。但 Message 的字段结构由编译器保证，测试价值较低。

**建议：** 保留现有测试（验证字段正确性），但应补充行为测试（如 `handle_ui_commands` 处理命令后的状态变化）。

### settings.rs（3 个测试）

| 测试 | 验证目标 | Behavior/Implementation | 合规性 |
|------|----------|------------------------|--------|
| `game_settings_default_has_reasonable_values` | 默认值 | Behavior — 数据契约 | 良好 |
| `game_settings_ron_roundtrip_preserves_all_fields` | 序列化往返 | Behavior — 数据契约 | 良好 |
| `color_blind_mode_all_variants_roundtrip` | 枚举完整性 | Behavior — 边界测试 | 良好 |

**问题：** 无。测试质量高，验证数据契约和边界条件。

### theme.rs（4 个测试）

| 测试 | 验证目标 | Behavior/Implementation | 合规性 |
|------|----------|------------------------|--------|
| `faction_color_distinguishes_player_and_enemy` | 颜色区分 | Behavior | 良好 |
| `faction_color_player_is_blue_tinted` | 蓝色系 | Behavior | 良好 |
| `faction_color_enemy_is_red_tinted` | 红色系 | Behavior | 良好 |
| `ui_theme_default_values_complete` | 默认值完整性 | Behavior — 数据契约 | 良好 |

**问题：** 无。测试质量高，验证行为和边界条件。

---

# 6. test_spec.md 合规性评审

## 6.1 §3 Testing Philosophy — 测试行为而非实现

| 评级 | 说明 |
|------|------|
| **基本合规** | 大部分测试验证行为，但 events.rs 中有 3 个测试仅验证结构体字段 |

## 6.2 §4 Test Pyramid — 测试金字塔

| 层级 | 要求 | 实际 | 差距 |
|------|------|------|------|
| Unit Test | 70% | ~11 个（100%） | 比例过高 |
| Integration Test | 20% | 0 个（0%） | **缺失** |
| Replay Test | 8% | 0 个（0%） | **缺失** |
| E2E Test | 2% | 0 个（0%） | 可接受 |

**结论：测试金字塔严重失衡，只有单元测试，缺少集成测试和回放测试。**

## 6.3 §6 Determinism Rules — 确定性

| 评级 | 说明 |
|------|------|
| **合规** | 所有测试均为确定性，无随机性 |

## 6.4 §7 Test Case Schema — 测试用例结构

| 评级 | 说明 |
|------|------|
| **合规** | 所有测试已添加 Test ID 和 Given/When/Then 结构（已修复） |

## 6.5 §7.1 Standard Test Data — 标准测试数据

| 评级 | 说明 |
|------|------|
| **不合规** | 测试使用自定义数据，未使用 Unit_001/002/003 标准数据 |

## 6.6 §9 Coverage Strategy — 领域规则覆盖

| 评级 | 说明 |
|------|------|
| **不合规** | 5 个不变量中 0 个完全覆盖，4 个业务规则中 1 个完全覆盖 |

## 6.7 §10 Error Testing — 错误测试

| 评级 | 说明 |
|------|------|
| **不合规** | 无任何错误测试（空输入、无效状态、边界值） |

## 6.8 §13 AI Constraints — 禁止测试私有实现

| 评级 | 说明 |
|------|------|
| **轻微违规** | events.rs 中 3 个测试验证结构体字段赋值 |

## 6.9 §13.1 AI Self-Check — 自检标注

| 评级 | 说明 |
|------|------|
| **合规** | 所有测试文件已添加 AI Self-Check 标注（已修复） |

---

# 7. 问题分类统计

## 7.1 按严重程度

| 严重程度 | 数量 | 描述 |
|----------|------|------|
| **P0 Critical** | 2 | view_models.rs 零测试、command_handler.rs 零测试 |
| **P1 High** | 3 | focus.rs 零测试、camera.rs 零测试、无集成测试 |
| **P2 Medium** | 4 | 无回放测试、无错误测试、Standard Test Data 不一致、测试金字塔失衡 |
| **P3 Low** | 2 | events.rs 字段测试为实现细节、部分 Widget 函数未使用 |

## 7.2 按问题类型

| 类型 | 数量 | 具体问题 |
|------|------|----------|
| 覆盖缺失 | 5 | view_models.rs、command_handler.rs、focus.rs、camera.rs、highlight.rs |
| 规范不合规 | 3 | Standard Test Data、Error Testing、Test Pyramid |
| 测试质量 | 2 | events.rs 字段测试、无集成测试 |
| 架构违规风险 | 2 | 无测试验证 INV-UI-1/2/3 |

---

# 8. 优先级建议

## P0 — 必须立即补充（阻塞合并）

### 8.1 单元测试：ViewModel 更新逻辑

- **文件**：`src/ui/view_models.rs` 内联测试
- **目标**：
  - `SelectedUnitView` 默认值为空
  - `CombatPreviewView` 仅在 SelectTarget 阶段显示
  - `GameOverState` 胜负判定（无敌人→胜利，无玩家→失败）
  - `TurnInfoView` 刷新策略

### 8.2 单元测试：UI 焦点管理

- **文件**：`src/ui/focus.rs` 内联测试
- **目标**：
  - `UiFocusState` 默认值 blocks_input=false
  - `BlocksGameInput` 组件存在时 blocks_input=true
  - 无 `BlocksGameInput` 组件时 blocks_input=false

### 8.3 单元测试：相机边界钳制

- **文件**：`src/ui/camera.rs` 内联测试
- **目标**：
  - `clamp_camera_to_map` 超出边界时钳制
  - 缩放值钳制到 [0.3, 3.0]
  - 平滑移动插值公式

## P1 — 尽快补充（1 周内）

### 8.4 集成测试：UI 命令处理流程

- **文件**：`tests/feature/ui_commands.rs`（新建）
- **目标**：
  - 验证 UiCommand → handle_ui_commands → 状态变更
  - 验证 Cancel 上下文推断（3 种场景）
  - 验证 handle_ui_commands 仅在玩家回合执行

### 8.5 集成测试：ViewModel 刷新时机

- **文件**：`tests/feature/ui_view_models.rs`（新建）
- **目标**：
  - 验证 SelectedUnitView 仅在 HoveredEntity 变化时刷新
  - 验证 TurnInfoView 在 TurnState/TurnOrder 变化时刷新

### 8.6 错误测试补充

- 空 UiCommand 时的行为
- 无效技能 ID 时的行为
- 目标坐标无单位时的行为

## P2 — 计划补充（2 周内）

### 8.7 Test Case Schema 合规

为所有测试添加 Test ID 和 Given/When/Then 注释。Test ID 命名规则：

| 前缀 | 范围 |
|------|------|
| UI-CMD-xxx | events.rs 测试 |
| UI-SET-xxx | settings.rs 测试 |
| UI-THM-xxx | theme.rs 测试 |
| UI-VIEW-xxx | view_models.rs 测试（新增） |
| UI-FOCUS-xxx | focus.rs 测试（新增） |
| UI-CAM-xxx | camera.rs 测试（新增） |
| UI-CMDH-xxx | command_handler.rs 测试（新增） |
| UI-HLT-xxx | highlight.rs 测试（新增） |

### 8.8 Standard Test Data 适配

创建 UI 测试专用的标准数据辅助函数：

```rust
// tests/common/ui_fixtures.rs
pub fn unit_001_view() -> SelectedUnitView {
    SelectedUnitView {
        name: "Unit_001".to_string(),
        hp: 100,
        max_hp: 100,
        // ...
    }
}
```

### 8.9 Replay 测试补充

扩展 `golden_battle.rs`，增加 UI 交互场景：
- 玩家选择单位 → 移动 → 攻击
- AI 行动完整流程
- 取消操作流程

## P3 — 低优先级

### 8.10 清理实现细节测试

- events.rs 中 3 个字段测试可保留（验证字段正确性）
- 补充行为测试（命令处理后的状态变化）

### 8.11 补充 Widget 函数测试

- `vbox()`/`hbox()`/`panel()` 返回值验证
- `spawn_popup()` 偏移量计算
- `spawn_resource_bar()` 结构验证

---

# 9. 测试矩阵

## 9.1 领域不变量 × 测试覆盖

| 不变量 | Unit Test | Integration Test | Replay Test | 状态 |
|--------|:---------:|:----------------:|:-----------:|------|
| INV-UI-1 UI 不操作 ECS | 部分 | **需补充** | — | 不足 |
| INV-UI-2 ViewModel 隔离 | **需补充** | **需补充** | — | 缺失 |
| INV-UI-3 仅玩家回合 | **需补充** | **需补充** | — | 缺失 |
| INV-UI-4 BlocksGameInput | **需补充** | — | — | 缺失 |
| INV-UI-5 UI 不保存业务真相 | 部分 | — | — | 不足 |

## 9.2 业务规则 × 测试覆盖

| 规则 | Unit Test | Integration Test | Replay Test | 状态 |
|------|:---------:|:----------------:|:-----------:|------|
| BR-UI-1 UiCommand 唯一通道 | 部分 | **需补充** | — | 不足 |
| BR-UI-2 ViewModel 刷新策略 | **需补充** | **需补充** | — | 缺失 |
| BR-UI-3 Cancel 上下文推断 | — | **需补充** | — | 缺失 |
| BR-UI-4 主题统一样式 | 覆盖 | — | — | 良好 |

---

# 10. 测试执行结果

## 10.1 执行命令

```bash
cargo test --lib ui::events::tests
cargo test --lib ui::settings::tests
cargo test --lib ui::theme::tests
```

## 10.2 执行结果

| 测试文件 | 测试数 | 通过 | 失败 | 耗时 |
|----------|--------|------|------|------|
| `events.rs` | 4 | 4 | 0 | 0.00s |
| `settings.rs` | 3 | 3 | 0 | 0.00s |
| `theme.rs` | 4 | 4 | 0 | 0.00s |
| **总计** | **11** | **11** | **0** | **0.00s** |

**测试结果：全部通过 ✅**

## 10.3 编译警告

测试执行过程中发现以下业务代码警告（不影响测试通过）：

| 文件 | 警告类型 | 严重性 |
|------|----------|--------|
| `inventory/transfer.rs` | 无效 drop 调用 | P2 |
| `equipment/equip.rs` | 未使用变量 | P3 |
| `skill/preview.rs` | 未使用字段/函数 | P3 |
| `ai/targeting.rs` | 未使用字段 | P3 |
| `character/movement.rs` | 未使用常量 | P3 |
| `character/template.rs` | 未使用字段 | P3 |
| `ui/widgets/layout.rs` | 未使用函数 | P2 |

**详细修改建议见 `docs/testing/business_code_issues.md`**

---

# 11. 总体评估

| 维度 | 评级 | 说明 |
|------|------|------|
| 领域规则覆盖 | **D** | 5 个不变量中 0 个完全覆盖，4 个业务规则中 1 个完全覆盖 |
| 测试金字塔 | **F** | 只有单元测试，缺少集成测试和回放测试 |
| 测试质量 | **B** | 现有测试质量高，验证行为和边界条件 |
| 规范合规 | **B+** | 已修复 Test Case Schema 和 AI Self-Check |
| 错误/边界覆盖 | **F** | 无任何错误测试 |
| 确定性 | **A** | 所有测试均为确定性 |
| 测试执行 | **A** | 11/11 测试全部通过 |

**综合评级：C-**

核心问题：
1. **view_models.rs 和 command_handler.rs 零测试** — 两个核心模块完全无测试保护
2. **无集成测试** — 无法验证 UI 交互流程
3. **测试金字塔严重失衡** — 只有单元测试，缺少集成和回放测试

**已完成修复：**
- ✅ 所有测试添加 AI Self-Check 标注
- ✅ 所有测试添加 Test ID 和 Given/When/Then 结构
- ✅ 测试函数重命名为 snake_case 英文
- ✅ 补充 theme.rs 默认值完整性测试
- ✅ 11/11 测试全部通过

---

# 12. 行动计划摘要

| 优先级 | 行动项 | 预计新增测试数 |
|--------|--------|---------------|
| P0 | 单元测试：view_models.rs | 6 |
| P0 | 单元测试：focus.rs | 3 |
| P0 | 单元测试：camera.rs | 4 |
| P1 | 集成测试：UI 命令处理流程 | 4 |
| P1 | 集成测试：ViewModel 刷新时机 | 3 |
| P1 | 错误测试补充 | 3 |
| P2 | Standard Test Data 适配 | 0（重构现有） |
| P2 | Replay 测试补充 | 3 |
| P3 | Widget 函数测试 | 5 |

**预计新增测试：31 个**

---

# 13. 代码修改建议

测试执行过程中发现的业务代码问题，详见 `docs/testing/business_code_issues.md`。
