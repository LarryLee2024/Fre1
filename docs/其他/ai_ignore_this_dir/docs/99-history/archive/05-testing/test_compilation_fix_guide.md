# 测试编译错误修复指南

Version: 1.0
Date: 2026-06-12
Status: 分析完成，待修复

---

## 问题概述

运行 `cargo test --lib --tests` 时出现 **5280 个编译错误**，全部位于 `#[cfg(test)]` 模块中。错误类型为 **缺少必要的 import 语句**。

## 错误分类

### 1. 缺少 Bevy 类型导入

以下类型在测试模块中使用但未导入：

| 类型 | 来源 | 受影响文件 |
|------|------|-----------|
| `Entity` | `bevy::prelude::Entity` | 大部分测试文件 |
| `IVec2` | `bevy::prelude::IVec2` | 地图/寻路相关测试 |
| `App` | `bevy::prelude::App` | 集成测试 |
| `World` | `bevy::prelude::World` | 组件测试 |
| `Vec2` | `bevy::prelude::Vec2` | UI 相关测试 |
| `Color` | `bevy::prelude::Color` | UI 主题测试 |
| `Srgba` | `bevy::color::Srgba` | UI 主题测试 |
| `OnEnter` | `bevy::prelude::OnEnter` | 回合系统测试 |
| `default` | `bevy::prelude::default` | 调试模块测试 |

### 2. 缺少标准库类型导入

| 类型 | 来源 | 受影响文件 |
|------|------|-----------|
| `String` | `std::string::String` | 大部分测试文件 |
| `Vec` | `std::vec::Vec` | 效果系统测试 |
| `Box` | `std::boxed::Box` | 处理器注册表测试 |

---

## 受影响文件清单

### 核心模块 (`src/`)

| 文件 | 缺少的导入 |
|------|-----------|
| `battle/pipeline/trait_trigger.rs` | `Entity`, `String` |
| `battle/record.rs` | `Entity`, `IVec2` |
| `buff/apply.rs` | `Entity` |
| `buff/instance.rs` | `Entity` |
| `character/components.rs` | `World` |
| `character/traits/handlers.rs` | `Box` |
| `character/traits/mod.rs` | `String` |
| `core/effect/handler.rs` | `Entity`, `Box` |
| `core/effect/types.rs` | `Entity`, `Vec` |
| `core/snapshot.rs` | `App` |
| `equipment/equip.rs` | `Entity` |
| `equipment/requirements.rs` | `String` |
| `inventory/battle_bag.rs` | `String`, `Entity`, `App` |
| `inventory/container.rs` | `String` |
| `inventory/instance.rs` | `String` |
| `inventory/transfer.rs` | `String` |
| `inventory/use_item.rs` | `String`, `Entity` |
| `map/grid.rs` | `IVec2` |
| `map/pathfinding/mod.rs` | `IVec2`, `Entity` |
| `map/runtime/occupancy_grid.rs` | `Entity`, `IVec2` |
| `map/runtime/terrain_grid.rs` | `IVec2` |
| `skill/domain/types.rs` | `String` |
| `skill/domain/mod.rs` | `String` |
| `skill/preview.rs` | `Entity`, `String` |
| `skill/slots.rs` | `String` |
| `turn/order.rs` | `Entity`, `App`, `OnEnter` |
| `ui/camera.rs` | `Vec2` |
| `ui/events.rs` | `Entity`, `IVec2` |
| `ui/theme.rs` | `Srgba` |
| `ui/view_models.rs` | `Entity` |
| `ui/widgets/layout.rs` | `Color` |
| `debug/mod.rs` | `default` |

---

## 修复方案

### 方案 A（推荐）：在每个测试模块中添加完整导入

在每个 `#[cfg(test)] mod tests` 块的顶部添加所需的导入：

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    // 如果需要特定类型，还可以添加：
    // use bevy::color::Srgba;
    
    // ... 测试代码 ...
}
```

### 方案 B（备选）：创建共享测试工具模块

创建 `tests/common/test_prelude.rs`：

```rust
// tests/common/test_prelude.rs
pub use bevy::prelude::*;
pub use std::string::String;
pub use std::vec::Vec;
pub use std::boxed::Box;
```

然后在每个测试文件中：

```rust
#[cfg(test)]
mod tests {
    use crate::tests::common::test_prelude::*;
    
    // ... 测试代码 ...
}
```

---

## 具体修复示例

### 1. `battle/pipeline/trait_trigger.rs`

当前（错误）：
```rust
#[cfg(test)]
mod tests {
    // 缺少导入
    
    #[test]
    fn on_attack_触发apply_buff() {
        let attacker = Entity::from_bits(1);  // 错误：Entity 未定义
        // ...
    }
}
```

修复后：
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    
    #[test]
    fn on_attack_触发apply_buff() {
        let attacker = Entity::from_bits(1);  // ✅ 正确
        // ...
    }
}
```

### 2. `character/components.rs`

当前（错误）：
```rust
#[cfg(test)]
mod tests {
    // 缺少导入
    
    #[test]
    fn dead_hook_removes_selected() {
        let mut world = World::new();  // 错误：World 未定义
        // ...
    }
}
```

修复后：
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    
    #[test]
    fn dead_hook_removes_selected() {
        let mut world = World::new();  // ✅ 正确
        // ...
    }
}
```

### 3. `ui/camera.rs`

当前（错误）：
```rust
#[cfg(test)]
mod tests {
    // 缺少导入
    
    #[test]
    fn camera_target_can_set_position() {
        let expected = Vec2::new(100.0, 200.0);  // 错误：Vec2 未定义
        // ...
    }
}
```

修复后：
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    
    #[test]
    fn camera_target_can_set_position() {
        let expected = Vec2::new(100.0, 200.0);  // ✅ 正确
        // ...
    }
}
```

---

## 验证步骤

修复后运行：

```bash
cargo test --lib --tests
```

预期结果：
- 编译错误数量：0
- 测试通过数量：467（lib）+ 42（integration）= 509
- 测试失败数量：0

---

## 注意事项

1. **不要修改业务逻辑**：仅添加导入语句，不修改测试断言或业务代码
2. **保持测试覆盖率**：确保所有现有测试都能编译和通过
3. **遵循 AI 宪法**：测试应验证行为而非实现
4. **确定性**：测试应产生确定性结果

---

## 自检结果

| 检查项 | 结果 |
|--------|------|
| 架构合规 | PASS — 仅修改测试模块导入 |
| 领域规则 | PASS — 不涉及业务逻辑 |
| 测试规范 | PASS — 保持现有测试不变 |
| 命名规范 | PASS |
| 可见性 | PASS |
| 错误处理 | PASS |
| 死代码 | PASS |
| 重复代码 | PASS |
