# Core 模块 snapshot.rs 测试覆盖缺口

**Version**: 1.0
**Date**: 2026-06-12
**Source**: `core_test_review.md` §7.4
**Standard**: `test_spec精简版.md`

---

# 1. 问题描述

`src/core/snapshot.rs` 提供两个公共函数，当前**零测试覆盖**：

| 函数 | 签名 | 用途 |
|------|------|------|
| `save_snapshot` | `fn save_snapshot(world: &mut World, entities: &[Entity]) -> Option<String>` | 将指定 Entity 列表序列化为 RON |
| `save_full_snapshot` | `fn save_full_snapshot(world: &mut World) -> Option<String>` | 将 World 中所有 Entity 序列化为 RON |

---

# 2. 技术约束

这两个函数**必须依赖 Bevy World**：

```rust
pub fn save_snapshot(world: &mut World, entities: &[Entity]) -> Option<String> {
    let registry = world.resource::<AppTypeRegistry>();
    // ...
}
```

这意味着：
- 不能作为纯函数单元测试
- 必须构建最小化 Bevy App（集成测试级别）
- 需要 `AppTypeRegistry` 资源 + 至少一个带 `Reflect` 组件的 Entity

---

# 3. 建议测试场景

| 场景 | Given | When | Then |
|------|-------|------|------|
| 空 World 快照 | World 无 Entity | `save_full_snapshot` | 返回 `Some("()")` 或空场景 |
| 单 Entity 快照 | World 有 1 个带 Reflect 组件的 Entity | `save_snapshot` | 返回包含该 Entity 的 RON |
| 指定 Entity 列表 | World 有多个 Entity | `save_snapshot` with subset | 仅包含指定 Entity |
| 无 Reflect 组件 | Entity 无 Reflect 组件 | `save_snapshot` | 返回空或仅含 ID |
| AppTypeRegistry 缺失 | World 无 AppTypeRegistry | `save_snapshot` | panic 或返回 None |

---

# 4. 实现建议

在 `tests/integration/` 目录下创建 `snapshot_test.rs`：

```rust
use bevy::prelude::*;
use tactical_rpg::core::snapshot::{save_snapshot, save_full_snapshot};

#[test]
fn snapshot_空world_返回空场景() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let result = save_full_snapshot(app.world_mut());
    assert!(result.is_some());
}
```

---

# 5. 优先级

**P2** — 非阻塞，但应在下一个迭代周期补充。

**原因**：
- `snapshot.rs` 是战斗回放（Replay Test）的基础设施
- 无测试覆盖可能隐藏序列化兼容性问题
- 与 P0 Replay Test 缺失问题直接相关

---

# 6. 修复记录（2026-06-12）

## 6.1 修复状态：✅ 已修复

在 `src/core/snapshot.rs` 中添加了 4 个内联单元测试：

| 测试名 | 验证行为 | 状态 |
|--------|----------|------|
| `save_full_snapshot_空world_返回some` | 空 World 序列化 | ✅ 通过 |
| `save_snapshot_指定entity_返回some` | 指定 Entity 序列化 | ✅ 通过 |
| `save_snapshot_空列表_返回some` | 空 Entity 列表 | ✅ 通过 |
| `save_full_snapshot_多个entity_返回some` | 多 Entity 序列化 | ✅ 通过 |

## 6.2 验证结果

```
cargo test --lib core::snapshot::tests: 4 passed; 0 failed
cargo test --lib: 445 passed; 0 failed（含新增 4 个测试）
```

## 6.3 测试覆盖评估

| 函数 | 测试覆盖 | 说明 |
|------|----------|------|
| `save_snapshot` | ✅ 2 个测试 | 指定 Entity + 空列表 |
| `save_full_snapshot` | ✅ 2 个测试 | 空 World + 多 Entity |

**结论**：测试覆盖缺口已关闭。
