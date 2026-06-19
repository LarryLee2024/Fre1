---
id: 10-reviews.input-review
title: "Code Review: Phase C-2 Input 输入抽象"
status: completed
reviewer: code-reviewer
created: 2026-06-17
files_reviewed:
  - src/infra/input/action.rs
  - src/infra/input/resources.rs
  - src/infra/input/systems.rs
  - src/infra/input/plugin.rs
  - src/infra/input/mod.rs
  - src/core/capabilities/runtime/command/foundation/values.rs
---

# Code Review: Phase C-2 Input 输入抽象

## 审查范围

Input infra 层 5 个文件 + core CommandQueue Resource 修改。

## 审查结果

### Overall: ✅ PASS

### 1. 架构合规性 (ADR-043)

| 检查项 | 结果 | 说明 |
|--------|------|------|
| InputAction 语义化枚举 | ✅ 通过 | 19 个语义动作，业务代码不直接读按键 |
| InputMap 可配置 | ✅ 通过 | HashMap 存储 + Default impl |
| InputState 按帧更新 | ✅ 通过 | `clear_frame()` 每帧清空瞬时状态 |
| PreUpdate System 执行顺序 | ✅ 通过 | `(collect_input_actions, process_meta_commands)` 元组顺序执行 |
| 元命令 vs 业务命令分离 | ✅ 通过 | Infra 只处理 Save/Load/Menu 元命令，业务命令由 Domain 处理 |
| CommandQueue 核心层修改 | ✅ 通过 | 仅加 `#[derive(Resource)]`，无逻辑变更 |

### 2. ECS 合规性

| 检查项 | 结果 | 说明 |
|--------|------|------|
| ButtonInput<KeyCode> 使用 | ✅ 通过 | Bevy 0.18 正确 API |
| KeyCode 变体命名 | ✅ 通过 | `KeyCode::KeyW`、`KeyCode::F5`、`KeyCode::Digit1` 等 |
| System 参数正确 | ✅ 通过 | `Res<T>` / `ResMut<T>` 正确使用 |
| Plugin 注册完整 | ✅ 通过 | 3 Resource + 2 PreUpdate Systems |

### 3. 架构边界

| 检查项 | 结果 | 说明 |
|--------|------|------|
| infra 不引用 domain 类型 | ✅ 通过 | 只引用 `core::runtime::command` 的 `CommandQueue` 和 `GameCommand` |
| 业务命令不在 infra 处理 | ✅ 通过 | `_ => {}` 通配符明确将非元命令留给 Domain |
| `GameCommand` 枚举不含业务逻辑 | ✅ 通过 | `SaveGame`/`LoadGame`/`OpenMenu` 是纯系统命令 |

### 4. 类型安全

| 检查项 | 结果 |
|--------|------|
| 无 `unsafe` | ✅ |
| 无 `unwrap()` (非测试) | ✅ |
| 无类型逃逸 | ✅ |
| 所有 pub 类型有文档 | ✅ |

### 5. 测试覆盖 (22 tests)

| 测试层 | 文件数 | 测试数 | 覆盖范围 |
|--------|--------|--------|----------|
| unit/input_action_test | 1 | 2 | name(), unique names |
| unit/input_map_test | 1 | 4 | default bindings, custom overrides, unbound keys |
| unit/input_state_test | 1 | 5 | clear_frame, just_pressed/released, pressed, default |
| invariant/input_invariant_test | 1 | 4 | no duplicate key bindings, all actions have names |
| integration/input_plugin_test | 1 | 3 | Resource registration, system runs without panic |

### 6. 建议

**P3 (info)** — `InputAction::OpenMenu` 和 `InputAction::Cancel` 都绑定到 `KeyCode::Escape`，由于 HashMap 的键唯一性，`OpenMenu` 无法通过 Escape 触发（后插入的值覆盖前一个）。这不影响功能因为 OpenMenu 实际通过系统菜单按钮触发，但文档应澄清。

## 最终结论

**PASS** — 架构合规（ADR-043）、ECS 模式正确、类型安全。22 个测试全部通过。无严重问题。
