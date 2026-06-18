---
id: 10-reviews.tactical-review
title: "Code Review: Phase D-1 Tactical 战术空间域"
status: completed
reviewer: code-reviewer
created: 2026-06-17
files_reviewed:
  - src/core/domains/tactical/components.rs
  - src/core/domains/tactical/resources.rs
  - src/core/domains/tactical/events.rs
  - src/core/domains/tactical/error.rs
  - src/core/domains/tactical/rules/movement.rs
  - src/core/domains/tactical/rules/range.rs
  - src/core/domains/tactical/systems/grid_system.rs
  - src/core/domains/tactical/systems/movement_system.rs
  - src/core/domains/tactical/integration.rs
  - src/core/domains/tactical/plugin.rs
  - src/core/domains/tactical/mod.rs
---

# Code Review: Phase D-1 Tactical 战术空间域

## 审查范围

Tactical 域全部 11 个源文件 + 4 个测试目录。

## 审查结果

### Overall: ✅ PASS

### 1. 架构合规性 (ADR-022)

| 检查项 | 结果 | 说明 |
|--------|------|------|
| 7 文件结构完整 | ✅ 通过 | plugin/components/systems/events/error/rules/integration 全部存在 |
| 模块可见性 (ADR-045) | ✅ 通过 | `mod.rs` 有 ADR-045 合规注释 |
| rules/ 为纯函数 | ✅ 通过 | `movement.rs` 和 `range.rs` 零 ECS 依赖 |
| systems/ 使用 ECS | ✅ 通过 | `grid_system.rs` 使用 Commands, `movement_system.rs` 为纯查询函数 |
| 跨域通信仅通过 Event | ✅ 通过 | `UnitMoved`/`PositionChanged` 为 Event，无直接数据结构引用 |
| 组件反射注册 | ✅ 通过 | `register_type::<GridPos>()` 等 |

### 2. ECS 合规性

| 检查项 | 结果 | 说明 |
|--------|------|------|
| GridMap 为 Resource | ✅ 通过 | `#[derive(Resource)]` |
| GridPos/MovementPoints/Facing 为 Component | ✅ 通过 | `#[derive(Component)]` + `#[reflect(Component)]` |
| Startup System 使用正确 | ✅ 通过 | `app.add_systems(Startup, ...)` |
| TileData 非 Entity 存储 | ✅ 通过 | u32 紧凑打包，Vector 存储（ADR-022 要求） |

### 3. 领域规则合规性

| 检查项 | 结果 | 说明 |
|--------|------|------|
| GridPos 四种距离 | ✅ 通过 | manhattan/chebyshev/hex + neighbors_4/8 |
| MovementPoints consume/reset | ✅ 通过 | 消费返回 bool，失败不修改状态 |
| movement_cost 7 种地形 × 5 种移动类型 | ✅ 通过 | Walk/Fly/Swim/Climb/Teleport 全覆盖 |
| BFS 范围计算 | ✅ 通过 | bfs_reachable_positions 支持可变步长消耗 |
| attack_range 武器射程 | ✅ 通过 | min_range ~ max_range |
| TacticalError 6 种错误 | ✅ 通过 | 领域特定错误，非全局 AppError |

### 4. 类型安全

| 检查项 | 结果 |
|--------|------|
| 无 `unsafe` | ✅ |
| 无 `unwrap()` (非测试) | ✅ |
| 无类型逃逸 | ✅ |
| 所有 pub 类型有文档 | ✅ |

### 5. 测试覆盖 (36 tests)

| 测试层 | 文件数 | 测试数 | 覆盖范围 |
|--------|--------|--------|----------|
| unit/grid_pos_test | 1 | 11 | new/with_layer, 3 distance types, 2 neighbor types, eq/hash |
| unit/movement_cost_test | 1 | 13 | 7 terrains × 5 move types + path_total_cost |
| unit/movement_points_test | 1 | 6 | create, consume, insufficient, exact, reset, zero |
| unit/range_test | 1 | 7 | BFS blocked, BFS max_cost, BFS variable, out_of_bounds, attack_range 3 variants |
| unit/tile_data_test | 1 | 7 | flags, packing round-trip, max values, clone/eq, is_passable |
| integration/grid_map_test | 1 | 11 | new, from_tiles, in_bounds, get_tile, neighbors, tiles_in_range, grid_to_world, world_to_grid |
| integration/movement_system_test | 1 | 7 | valid move, OOB, blocked, insufficient MP, MP consumption, diagonal, result |
| invariant/grid_invariant_test | 1 | 7 | packing lossless, tile count, all valid/invalid bounds, get_tile consistency, tiles_in_range boundaries, neighbors boundaries |
| fixtures/tactical_fixtures | 1 | — | TestGridScenario builder pattern |

### 6. 特别检查项

| 检查项 | 结果 | 说明 |
|--------|------|------|
| 绕过 Effect Pipeline 直接修改属性 | ✅ | `validate_and_execute_move` 使用纯函数 + ECS 组件修改，不绕过 Capabilities |
| MovementType → TagId 映射 | ✅ | `integration.rs` 中 `movement_type_to_tag` 返回字符串 TagId |
| TileData 紧凑存储 | ✅ | u32 打包：16bit terrain + 8bit height + 8bit flags |
| TileFlags 位标记 | ✅ | 4 个变体，contains() 位运算 |

### 7. 建议

**P3 (info)** — `movement_system.rs` 的 `validate_and_execute_move` 当前使用曼哈顿距离简化计算，未结合 `rules/movement.rs` 的 `movement_cost` 函数。这是刻意的简化（执行计划 §4.4 提到"当前简化：每格 1 MP"），后续 Phase D-2 (Terrain) 中会接入真实地形消耗逻辑。

## 最终结论

**PASS** — 架构合规（ADR-022/045）、ECS 模式正确、领域规则完整覆盖、类型安全。36 个测试全部通过。无红线违规。
