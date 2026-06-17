# UI 模块业务代码问题记录

**Date**: 2026-06-12
**Scope**: `src/ui/` 模块测试过程中发现的业务代码问题
**Standard**: `docs/test_spec精简版.md`, `ai_constitution.md`

---

# 1. 问题清单

## 1.1 camera.rs - clamp_camera_to_map 为私有函数

**严重性**: P2（建议改进）
**文件**: `src/ui/camera.rs`
**行号**: 220-227

**问题描述**:
`clamp_camera_to_map` 函数为私有（`fn` 而非 `pub fn`），导致无法直接测试相机边界钳制逻辑。

**当前代码**:
```rust
fn clamp_camera_to_map(transform: &mut Transform, map: &GameMap) {
    let half_w = map.width as f32 * map.tile_size / 2.0;
    let half_h = map.height as f32 * map.tile_size / 2.0;
    transform.translation.x = transform.translation.x.clamp(-half_w, half_w);
    transform.translation.y = transform.translation.y.clamp(-half_h, half_h);
}
```

**影响**:
- 无法直接测试边界钳制的精确行为
- 只能通过集成测试间接验证（需要完整 ECS 环境）

**修改建议**:
1. 将 `clamp_camera_to_map` 改为 `pub(crate)` 可见性，允许模块内测试
2. 或者提取纯逻辑到独立的公共函数（推荐）：
   ```rust
   /// 钳制值到边界内（纯函数，可测试）
   pub fn clamp_to_boundary(value: f32, min: f32, max: f32) -> f32 {
       value.clamp(min, max)
   }
   
   fn clamp_camera_to_map(transform: &mut Transform, map: &GameMap) {
       let half_w = map.width as f32 * map.tile_size / 2.0;
       let half_h = map.height as f32 * map.tile_size / 2.0;
       transform.translation.x = clamp_to_boundary(transform.translation.x, -half_w, half_w);
       transform.translation.y = clamp_to_boundary(transform.translation.y, -half_h, half_h);
   }
   ```

**测试覆盖情况**:
- ✅ 已通过 `camera_zoom_clamped_to_valid_range` 测试缩放钳制逻辑
- ⚠️ 相机位置钳制逻辑未直接测试（私有函数限制）

---

## 1.2 view_models.rs - update_selected_unit_view 过于复杂

**严重性**: P2（建议改进）
**文件**: `src/ui/view_models.rs`
**行号**: 154-409

**问题描述**:
`update_selected_unit_view` 函数过长（250+ 行），包含多个 `unwrap_or_default()` 分支，违反单一职责原则。

**影响**:
- 难以测试每个分支的精确行为
- 维护成本高

**修改建议**:
1. 将数据提取逻辑拆分为独立的辅助函数：
   - `extract_core_attrs()`
   - `extract_combat_attrs()`
   - `extract_support_attrs()`
   - `extract_skills()`
   - `extract_traits()`
   - `extract_buffs()`
   - `extract_equipment()`
   - `extract_inventory()`
2. 每个辅助函数可独立测试

**测试覆盖情况**:
- ✅ 已通过 `selected_unit_view_default_is_empty` 测试默认值
- ⚠️ 各数据提取分支未单独测试（函数复杂度限制）

---

## 1.3 events.rs - Entity::from_bits 硬编码

**严重性**: P3（低优先级）
**文件**: `src/ui/events.rs`
**行号**: 56, 86

**问题描述**:
测试中使用 `Entity::from_bits(1)` 和 `Entity::from_bits(42)` 硬编码值，未使用标准测试数据（Unit_001/002/003）。

**影响**:
- 不符合 `test_spec精简版.md` §7.1 标准测试数据要求
- 但对于 UI 命令测试，Entity 值仅作为标识符，不影响业务逻辑

**修改建议**:
当前可接受，因为 UiCommand 仅携带 Entity 标识符，不涉及具体单位数据。如需严格合规，可定义常量：
```rust
const TEST_ENTITY_ID: u64 = 42;
```

---

# 2. 问题统计

| 严重性 | 数量 | 说明 |
|--------|------|------|
| P0 | 0 | 无阻塞性问题 |
| P1 | 0 | 无必须修复问题 |
| P2 | 2 | 建议改进（可测试性、代码结构） |
| P3 | 1 | 低优先级（命名规范） |

---

# 3. 结论

UI 模块业务代码整体质量良好，无 P0/P1 级别问题。主要改进方向：

1. **可测试性**: 将私有纯函数改为公共或提取到可测试模块
2. **代码结构**: 拆分过长函数，提高可维护性
3. **规范合规**: 低优先级的命名规范改进

**当前状态**: 所有 25 个 UI 测试通过，测试覆盖率从 11 提升到 25（+127%）。
