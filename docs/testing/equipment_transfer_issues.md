# 业务代码修改建议

**Version**: 1.0
**Date**: 2026-06-12
**Source**: Equipment 模块测试执行过程中的编译错误
**Standard**: `test_spec精简版.md`, `code_style.md`, `ai_constitution.md`

---

# 1. 问题来源

在执行 `cargo test --lib equipment` 过程中，编译器输出以下错误，表明业务代码存在接口签名不匹配问题。

---

# 2. 问题清单

## 2.1 `src/inventory/transfer.rs` - `can_merge_with` 方法调用参数不足

**严重性**: P1（阻塞编译）
**位置**: `src/inventory/transfer.rs:68` 和 `src/inventory/transfer.rs:151`
**问题**: `can_merge_with` 方法需要 2 个参数，但调用处只传了 1 个

```rust
// 当前代码（line 68）
.any(|s| s.instance.def_id == def_id && s.can_merge_with(&stack));

// 当前代码（line 151）
let can_merge = to.stacks.iter().any(|s| s.can_merge_with(&new_stack));
```

**原因分析**:
`can_merge_with` 方法签名已更新为 `pub fn can_merge_with(&self, other: &ItemStack, def: &ItemDef)`，但 `transfer.rs` 中的调用未同步更新。这可能是 `can_merge_with` 方法在某次重构中增加了 `def` 参数（用于检查物品类型一致性），但调用方未跟进修改。

**修改建议**:
```rust
// line 68 修改为：
.any(|s| {
    s.instance.def_id == def_id
        && s.can_merge_with(&stack, item_registry.get(&def_id).unwrap())
})

// line 151 修改为：
let can_merge = to.stacks.iter().any(|s| {
    s.can_merge_with(&new_stack, registry.get(&new_stack.instance.def_id).unwrap())
});
```

**影响范围**: 背包物品转移逻辑
**风险**: 高 - 当前代码无法编译，阻塞所有测试执行

---

# 3. 优先级总结

| 优先级 | 问题 | 位置 | 说明 |
|--------|------|------|------|
| **P1** | `can_merge_with` 参数不足 | transfer.rs:68, 151 | 阻塞编译，需立即修复 |

---

# 4. 建议执行顺序

1. **立即修复**: `transfer.rs` 的 `can_merge_with` 调用参数
2. **验证**: 修复后运行 `cargo test --lib equipment` 确认通过

---

# 5. 结论

测试执行过程中发现 **1 个业务代码问题**，其中：
- **1 个 P1 问题**：接口签名不匹配，阻塞编译

此问题不影响测试代码质量，但阻塞测试执行，需优先修复业务代码。
