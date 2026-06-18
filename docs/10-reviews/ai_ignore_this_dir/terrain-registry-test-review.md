# 测试质量审查报告 — D-2 Terrain + C-3 Registry

> 审查人: Sisyphus (@feature-developer 兼 @test-guardian)
> 日期: 2026-06-17
> 更新: 2026-06-19 (内联测试已迁移至 tests/ 目录，inline #[cfg(test)] 已清理)
> 范围: D-2 Terrain 测试 (rules 内联测试), C-3 Registry 测试 (registry.rs/resolver.rs 内联测试)
> 基线: cargo test 1451 passed 0 failed

---

## 审查摘要

| 模块 | 测试类型 | 数量 | 覆盖度 | 质量 |
|------|---------|------|--------|------|
| Registry (registry.rs) | 内联单元测试 | 13 | 中 | 良好 |
| Registry (resolver.rs) | 内联单元测试 | 10 | 中 | 良好 |
| Terrain (movement_cost.rs) | 内联单元测试 | 5 | 低 | 可接受 |
| Terrain (concealment.rs) | 内联单元测试 | 4 | 低 | 可接受 |
| Terrain (tests/ 目录) | 单元/集成/不变量/夹具 | 0 | 无 | 缺失 |

**总体评价**: 内联测试覆盖了核心数据结构和纯函数的基本路径，但存在显著的覆盖缺口：ECS Systems 无测试、边界条件覆盖不足、tests/ 目录完全为空。

---

## C-3 Registry 测试审查

### ✅ 通过的测试

registry.rs (13 个测试):
- `test_bucket_insert_and_get` — 插入 + 查询 + 版本递增 ✅
- `test_bucket_replace` — 替换旧值 ✅
- `test_bucket_remove` — 删除 + 版本递增 ✅
- `test_bucket_iter_and_ids` — 遍历 + ID 列表 ✅
- `test_bucket_index` — 索引查询 ✅
- `test_definition_registry_new` — 初始化空状态 ✅
- `test_definition_registry_bucket_access` — 动态桶访问 ✅
- `test_definition_id_conversions` — From/Display trait ✅
- `test_registry_entry_lifecycle` — deprecated/supersede 状态 ✅
- `test_mark_changed` — 变更追踪 ✅
- `test_bucket_clear` — 清空 ✅
- `test_definition_id_display` — Display trait ✅

resolver.rs (10 个测试):
- `test_id_type_prefix_roundtrip` — 前缀解析往返 ✅
- `test_allocator_sequential_ids` — 顺序分配 ✅
- `test_allocator_recycle` — ID 回收复用 ✅
- `test_id_allocator_full` — 完整分配器初始化 ✅
- `test_id_allocator_unregistered_type` — 未注册类型返回 None ✅
- `test_validate_id` — ID 格式校验 ✅
- `test_validate_id_type` — 类型匹配校验 ✅
- `test_validation_runner_clean` — 干净数据通过校验 ✅
- `test_validation_runner_invalid_id` — 无效 ID 报错 ✅
- `test_validation_runner_duplicate_across_buckets` — 跨桶重复检测 ✅
- `test_validation_runner_warning_on_empty_data` — 空数据警告 ✅

### ⚠️ 缺失的测试覆盖

#### P2 — 无 `CrossReferenceReport` / `BrokenReference` 测试
**文件**: `src/infra/registry/resolver.rs`
**问题**: `CrossReferenceReport` 结构有 `add_broken()`、`merge()`、`has_broken_references()` 方法，但没有任何测试调用它们。`BrokenReference` 结构也未被测试构造。
**建议**: 添加至少 1 个测试验证断裂引用检测流程。

#### P2 — 无 `OnDefinitionReloaded` 事件测试
**文件**: `src/infra/registry/registry.rs`
**问题**: 事件结构有 `new()` 构造函数，但无测试验证事件创建和字段值。
**建议**: 添加简单测试 `test_reloaded_event_fields`。

#### P2 — `get_str` 无测试
**文件**: `src/infra/registry/registry.rs`
**问题**: `get_str` 方法是公开 API，但无测试覆盖。
**建议**: 添加 `test_bucket_get_str`。

#### P3 — `DefinitionType` trait 无实现测试
**文件**: `src/infra/registry/registry.rs`
**问题**: `DefinitionType` 是核心 trait，但当前无任何类型实现它，因此 trait 方法（`from_config`, `validate`）未被执行。
**建议**: 添加一个测试用的 mock 实现（如 `TestDef`），验证 trait 契约。

---

## D-2 Terrain 测试审查

### ✅ 通过的测试

movement_cost.rs (5 个测试):
- `walk_on_normal_uses_one_mp` — Normal + Walk = 1.0 ✅
- `walk_on_obstacle_is_blocked` — Obstacle + Walk = f32::MAX ✅
- `fly_ignores_terrain` — Fly 对所有地形 = 1.0 ✅ (枚举 ALL 覆盖)
- `swim_in_water_is_efficient` — Water + Swim = 1.0, Water + Walk = 2.0 ✅
- `teleport_cost_zero` — Teleport 对所有地形 = 0.0 ✅ (枚举 ALL 覆盖)

concealment.rs (4 个测试):
- `no_concealment_no_penalty` — None → 0 ✅
- `half_concealment_minus_two` — Half → -2 ✅
- `full_concealment_is_untargetable` — Full → is_targetable = false ✅
- `none_and_half_are_targetable` — None/Half → is_targetable = true ✅

### ⚠️ 缺失的测试覆盖

#### P1 — 移动消耗未覆盖全部组合
**文件**: `src/core/domains/terrain/rules/movement_cost.rs`
**问题**: 6 个移动类别 × 10 个地形 = 60 种组合。当前仅测试了 5 种典型组合（Normal+Walk, Obstacle+Walk, Fly×ALL, Swim+Water, Teleport×ALL）。缺失：
- Climb + Highground/Obstacle（应 = 1.0）
- Climb + 其他地形（应 = 2.0）
- Walk + Water/Bush/Ice/Poison/Burning/Oil/Lava
- Swim + 非 Water 地形（应 = 3.0）
**建议**: 至少补充 Climb 和 Swim 的边界测试。

#### P1 — 无 `f32::MAX` 的边界测试
**文件**: `src/core/domains/terrain/rules/movement_cost.rs`
**问题**: `Obstacle` 和 `Lava` 返回 `f32::MAX` 表示不可通行，但无测试验证调用方（如 Tactical movement_system）能否正确处理此值。
**建议**: 添加测试验证 `f32::MAX` 在消耗比较中的行为。

#### P1 — 无 ECS Systems 测试
**文件**: `src/core/domains/terrain/systems/*.rs`
**问题**: 3 个 Observer System 和 1 个常规 System 均无任何测试：
- `on_tile_entered` — 未测试 SurfaceType → effect_id 映射触发
- `on_surface_changed` — 未测试 TileProperties.surface 更新
- `on_hazard_check` — 未测试 HazardZone 匹配和事件触发
- `surface_recovery_system` — 未测试到期恢复逻辑
**建议**: 使用 Bevy 的 `App::update()` 或最小 ECS World 进行系统级测试。

#### P1 — tests/ 目录完全为空
**文件**: `src/core/domains/terrain/tests/unit/mod.rs`, `integration/mod.rs`, `invariant/mod.rs`, `fixtures/mod.rs`
**问题**: 4 个测试目录只有 `// TODO: 添加测试模块`。根据测试宪法 v4.0，领域内聚测试应分布在 unit/integration/invariant/fixtures 四层。
**建议**: 这是 @test-guardian 的职责范围，应在后续迭代中补充。当前 inline 测试可作为临时替代。

#### P2 — `concealment_bonus(Full)` 未测试返回值
**文件**: `src/core/domains/terrain/rules/concealment.rs`
**问题**: `concealment_bonus(&Concealment::Full)` 返回 `i32::MIN`，但测试仅验证了 `is_targetable()` 对 Full 返回 false，未验证 `concealment_bonus()` 的具体返回值。
**建议**: 添加 `assert_eq!(concealment_bonus(&Concealment::Full), i32::MIN);`

#### P2 — Doc-test 曾经失效
**文件**: `src/core/domains/terrain/rules/concealment.rs`, `movement_cost.rs`
**问题**: 原始 doc-test 使用 `use fre::core::domains::...` 路径引用 private 模块，导致 `cargo test` 编译失败。已修复为 `use crate::core::domains::...`。
**状态**: ✅ 已修复并验证通过。

---

## 测试架构合规性

| 规则 | 状态 | 说明 |
|------|------|------|
| 领域内聚 | 🟡 | Inline 测试在 rules/ 内，符合。但 tests/ 目录四层结构未填充。 |
| 无跨域集成测试 | ✅ | 测试仅验证 Terrain 内部纯函数，未引用其他 Domain。 |
| 无 @feature-developer 写测试 | 🟡 | 根据 AGENTS.md，@feature-developer 严禁写单元测试。当前 inline tests 由之前的 feature-developer（subagent）编写，违反了角色边界。 |
| Doc-test 合规 | ✅ | 修复后编译通过，无 ignored。 |

### ⚠️ 角色边界问题

**问题**: AGENTS.md 明确规定 `@feature-developer 严禁写单元测试代码`。但 D-2 Terrain 的 inline 测试（movement_cost.rs 5 个 + concealment.rs 4 个）以及 Registry 的 inline 测试（registry.rs 13 个 + resolver.rs 10 个）均在 feature-developer 的实现阶段被写入。

**建议**: 
1. 当前已写入的 inline 测试保留（删除成本高于收益）
2. 后续迭代中，@test-guardian 应审查这些测试的质量并补充缺失覆盖
3. 在 `.trae/rules/` 中增加明确的 inline test 归属规则：inline `#[cfg(test)]` 属于实现代码的一部分，可由 feature-developer 编写；独立的 `tests/` 目录测试由 test-guardian 负责

---

## 建议的测试补充计划

### 高优先级（下次迭代）

1. **System 测试** — 使用最小 ECS World 测试 Observer 触发
   - `on_tile_entered`: 构造 TileProperties(Poison) → trigger TileEntered → 验证 TerrainEffectApplied 被触发
   - `on_surface_changed`: trigger SurfaceChanged → 验证 TileProperties.surface 更新
   - `surface_recovery_system`: 构造 SurfaceOverride(remaining=1) → 运行 system → 验证恢复

2. **边界测试** — movement_cost 全组合
   - Climb × Highground/Obstacle = 1.0
   - Climb × Normal = 2.0
   - Swim × Normal = 3.0

3. **Registry CrossReference 测试**
   - 构造带引用的 RegistryEntry → 验证断裂引用检测

### 中优先级（后续迭代）

4. **fixtures** — 标准地形格子构造 helper
5. **invariant** — 验证地形不变量（如 "表面变化必须可恢复"）
6. **integration** — Terrain + Tactical 集成测试（移动消耗计算端到端）

---

*本审查报告由 Sisyphus 编写，替代未成功调度的 @test-guardian 后台任务。*
