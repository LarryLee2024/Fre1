# B0002: `transfer_item` 纯函数未检查目标容器容量

## 问题描述

`transfer_item` 纯函数（`src/inventory/transfer.rs:119`）在转移物品时，仅检查目标容器的**重量限制**，未检查**槽位容量限制**。当目标容器已满（`is_full()` 为 true）时，函数仍执行转移，导致源容器物品被减少但目标容器丢弃物品。

## 受影响测试

| 测试文件 | 测试用例 | 结果 |
|---|---|---|
| `tests/feature/inventory.rs` | `纯函数transfer_item_目标满返回full` | FAILED |
| `tests/feature/inventory.rs` | `目标容器满时转移失败_物品留在源容器` | FAILED |
| `tests/system/systems.rs` | `目标满时转移失败` | FAILED |

## 问题分析

```rust
// src/inventory/transfer.rs:119-156
pub fn transfer_item(...) -> ContainerResult {
    // ...检查重量...
    if to.max_weight > 0.0 {
        if let Some(def) = registry.get(&new_stack.instance.def_id) {
            let added_weight = new_stack.total_weight(def);
            if to.current_weight(registry) + added_weight > to.max_weight {
                return ContainerResult::Overweight;  // ✅ 重量检查
            }
        }
    }
    // ❌ 缺少: if to.is_full() { return ContainerResult::Full; }
    
    from.reduce_stack(instance_id, to_remove);  // 源容器已减少
    to.add_stack(&mut new_stack, registry);      // 目标满时 add_stack 丢弃物品
    ContainerResult::Ok                          // 始终返回 Ok
}
```

**关键缺陷**：
1. 缺少 `is_full()` 容量检查
2. `add_stack` 在容量满时静默丢弃物品（不返回错误）
3. 函数始终返回 `ContainerResult::Ok`，即使物品丢失

## 建议修复

在 `reduce_stack` 之前添加容量检查：

```rust
pub fn transfer_item(...) -> ContainerResult {
    // ...现有重量检查...
    
    // 添加容量检查
    if to.is_full() {
        return ContainerResult::Full;
    }
    
    from.reduce_stack(instance_id, to_remove);
    to.add_stack(&mut new_stack, registry);
    ContainerResult::Ok
}
```

## 验证方法

修复后运行：
```bash
cargo test --test feature -- inventory::纯函数transfer_item_目标满返回full
cargo test --test feature -- inventory::目标容器满时转移失败_物品留在源容器
cargo test --test system -- systems::目标满时转移失败
```

预期：3 个测试全部通过
