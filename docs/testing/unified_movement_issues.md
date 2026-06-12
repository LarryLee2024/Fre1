# 统一移动系统测试发现的问题报告

**日期**: 2026-06-12
**状态**: ✅ 已修复
**范围**: `src/character/movement_execution.rs`、`src/map/data.rs`

---

## 问题总览

在编写统一移动系统的集成测试时，发现3个业务逻辑问题：

| # | 问题描述 | 严重度 | 根因定位 |
|---|---------|--------|----------|
| 1 | 路径起点不正确，`reconstruct_path` 返回的路径第一个元素是目标而非起点 | 🟥 Critical | `reconstruct_path` 实现问题 |
| 2 | 超出移动范围的请求未被拒绝，仍添加 `MovingUnit` | 🟥 Critical | 缺少范围验证 |
| 3 | 原地不动（起点=终点）仍会添加 `MovingUnit` | 🟡 Major | 缺少相同位置检查 |

---

## 问题1：路径起点不正确

### 测试用例
`tests/feature/unified_movement.rs:287` - `horizontal_movement_path_correct`

### 预期行为
```rust
// 从 (0,0) 移动到 (3,0)
// 路径应为: [(0,0), (1,0), (2,0), (3,0)]
assert_eq!(moving.path.first(), Some(&IVec2::new(0, 0)));
assert_eq!(moving.path.last(), Some(&IVec2::new(3, 0)));
```

### 实际行为
```
assertion failed: Path should start at origin
  left: Some(IVec2(3, 0))   // ← 路径第一个元素是目标
 right: Some(IVec2(0, 0))   // ← 期望是起点
```

### 根因分析
`reconstruct_path` 函数在目标不可达或特殊情况时，可能返回只包含目标的单元素路径 `[target]`，而不是完整路径。

**相关文件**: `src/map/pathfinding/algorithms.rs` - `reconstruct_path` 函数

### 建议修复
在 `movement_execution_system` 中添加路径验证：
```rust
let path = reconstruct_path(...);

// 验证路径有效性
if path.is_empty() || path.first() != Some(&start_coord) {
    return; // 无效路径，不执行移动
}
```

---

## 问题2：超出移动范围未被拒绝

### 测试用例
`tests/feature/unified_movement.rs:227` - `out_of_range_movement_rejected`

### 预期行为
```rust
// 单位 MoveRange=3，尝试移动到 (10,10)
// 应被拒绝，不添加 MovingUnit 或路径为空
```

### 实际行为
```
Out of range movement should have empty path
```
单位获得了 `MovingUnit` 组件，且路径非空。

### 根因分析
`find_reachable_tiles` 计算可达范围后，`reconstruct_path` 仍能构造出路径（可能是通过容差匹配或其他机制绕过了范围限制）。

**相关文件**: 
- `src/map/pathfinding/algorithms.rs` - `find_reachable_tiles` / `reconstruct_path`
- `src/character/movement_execution.rs:59-79` - 缺少范围验证

### 建议修复
在执行移动前验证目标是否在可达范围内：
```rust
let reachable = find_reachable_tiles(...);

// 验证目标在可达范围内
if !reachable.contains_key(&intent.target_coord) {
    return; // 目标不可达，拒绝移动
}

let path = reconstruct_path(...);
```

---

## 问题3：原地不动仍执行移动

### 测试用例
`tests/feature/unified_movement.rs:256` - `same_position_movement_skipped`

### 预期行为
```rust
// 从 (3,3) 移动到 (3,3)
// 不应添加 MovingUnit
```

### 实际行为
```
Same position movement should not add MovingUnit
```
单位获得了 `MovingUnit` 组件。

### 根因分析
`movement_execution_system` 没有检查起点和终点是否相同。

**相关文件**: `src/character/movement_execution.rs:41-100`

### 建议修复
在执行移动前添加相同位置检查：
```rust
let start_coord = grid_pos.coord;

// 原地不动，跳过移动
if start_coord == intent.target_coord {
    return;
}

let move_range = attrs.get(AttributeKind::MoveRange) as u32;
// ... 后续逻辑
```

---

## 修复优先级

```
P0 (立即修复):
  ├── 问题2: 超出范围验证 → 防止非法移动
  └── 问题1: 路径起点验证 → 确保动画正确播放

P1 (短期修复):
  └── 问题3: 原地不动检查 → 避免不必要的 MovingUnit 添加
```

---

## 修复总结

### 根因

所有三个问题的根因：`TerrainRegistry::default()` 创建空注册表，`"plain"` 地形未被注册。`find_reachable_tiles` 查询 `terrain_registry.get("plain")` 返回 `None`，导致 `base_cost = None`，`GroundCostCalculator::cost` 返回 `None`，BFS 找不到任何可达格子。

### 修复内容

| # | 问题 | 修复 | 文件 |
|---|------|------|------|
| 1 | 路径起点不正确 | `TerrainRegistry::default()` 调用 `register_defaults()` 注册内置地形（plain/forest/mountain/water） | `src/map/data.rs` |
| 2 | 超出移动范围未被拒绝 | `movement_execution.rs` 添加 `reachable.contains_key(&target_coord)` 验证 | `src/character/movement_execution.rs` |
| 3 | 原地不动仍执行移动 | `movement_execution.rs` 添加 `start_coord == target_coord` 提前返回 | `src/character/movement_execution.rs` |

### 验证

```
cargo test -j1 --test feature unified_movement → 10 passed, 0 failed
cargo test -j1 → 1152 passed, 0 failed (full suite)
```

---

## 架构合规性检查

| 检查项 | 结果 | 说明 |
|--------|------|------|
| 意图与执行分离 | PASS | MovementIntent 事件正确使用 |
| AI/玩家一致性 | PASS | 两者使用相同的 execution system |
| ECS 原则 | PASS | 系统监听 Message，添加 Component |
| 边界定义 | PASS ✅ | 执行系统已验证输入合法性（范围+路径） |

---

## 自检结果

**PASS:**
- issue1: `reconstruct_path` 路径正确（`TerrainRegistry` 默认值修复）
- issue2: 添加了目标可达性验证，超出范围的移动被拒绝
- issue3: 添加了相同位置检查，原地不动不添加 `MovingUnit`

---

**报告结束**
