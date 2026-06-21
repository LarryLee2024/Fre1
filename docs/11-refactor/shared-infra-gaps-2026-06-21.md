---
id: 11-refactor.shared-infra-gaps-2026-06-21
title: "Shared & Infra 层差距分析与补全计划"
status: in_progress
owner: architect
created: 2026-06-21
tags:
  - shared
  - infra
  - gap-analysis
  - refactoring
---

# Shared & Infra 层差距分析与补全计划

> **扫描日期**: 2026-06-21 | **范围**: src/shared/ + src/infra/ + 相关 ADR

---

## 一、审核结论

### shared/ 层健康度

| 模块 | 状态 | 说明 |
|------|------|------|
| `ids/` | ✅ 完整 | StrongId 体系、foundation/mapping/types、宏、测试 |
| `time/` | ✅ 完整 | GameTime (frame+turn)、测试 |
| `random/` | ✅ 完整 | SeededRng(ChaCha12)、DeterministicRng(4流)、测试 |
| `diagnostics/` | ✅ 完整 | ObservableEvent、DomainEvent、LogCode、Domain、FieldCollector |
| `traits/` | ✅ 完整 | RuleFailure、sealed、宏 |
| `error/` | ✅ 完整 | ErrorContext、测试 |
| `testing/` | ✅ 完整(有瑕疵) | assertions、TestRng、fixtures — **doc 注释陈旧** |
| `localization_key.rs` | ✅ 完整 | LocalizationKey 类型 |
| `macros.rs` | ✅ 完整 | `register_domain_types!` |
| `shared_plugin.rs` | ⚠️ 注册不完全 | 只注册 GameTime，缺少 DeterministicRng |
| `prelude/` | ❌ **空实现** | mod.rs 只有一个 doc 注释，没有任何导出 |
| `validation/` | ❌ **TODO 桩** | 空的，只有 TODO 注释 |
| `math/` | ❌ **TODO 桩** | 空的，只有 TODO 注释 |
| `collections/` | ❌ **TODO 桩** | 空的，只有 TODO 注释 |
| `hashing/` | ❌ **TODO 桩** | 空的，只有 TODO 注释 |
| `path/` | ❌ **TODO 桩** | 空的，只有 TODO 注释 |
| `constants/` | ⚠️ 过于精简 | 只有 MAX_OBSERVER_DEPTH |

### infra/ 层健康度

| 模块 | 状态 | 说明 |
|------|------|------|
| `input/` | ✅ 完整 | action/plugin/resources/systems + 三层测试 |
| `localization/` | ✅ 完整 | foundation/io/storage/facade/ui/validation + 测试 |
| `logging/` | ✅ 完整 | telemetry + 15域 observers + rate_limit + sinks + metrics |
| `pipeline/` | ✅ 完整 | hooks/plugin + 测试 |
| `registry/` | ✅ 完整 | registry/resolver/plugin + 测试 |
| `replay/` | ✅ 完整 | events/plugin/resources/systems + 三层测试 |
| `save/` | ✅ 完整 | events/load_save/resources/systems + 三层测试 |

### 文档一致性

| 检查项 | 结果 | 说明 |
|--------|------|------|
| ADR-045 index 状态 vs 实际文件 | ❌ **不一致** | ADR index 写"Proposed"，文件实际是"Accepted" |
| ADR-053 index 状态 | ⚠️ **Proposed** | Localization ADR 仍未 Accepted |
| 测试/random doc 注释 | ❌ **陈旧** | 仍说 DeterministicRng 用"自制 MurmurHash3" |

---

## 二、执行计划

### Phase 1: Prelude 填充 — 最大易用性提升

**优先级: P0 | 工时: 0.5h | 风险: 🟢**

`shared/prelude/mod.rs` 当前完全空。对于 50 万行项目，每个 domain 都要手动 import 大量 shared 类型。

**实现**: 添加 canonical 的 re-export，使 domain 代码只需 `use crate::shared::prelude::*;` 即可访问常用 shared 类型：

```rust
pub use crate::shared::constants::*;
pub use crate::shared::diagnostics::{
    AuditEvent, Domain, DomainEvent, FieldCollector, LogCode, ObservableEvent, ReplayEvent,
};
pub use crate::shared::localization_key::LocalizationKey;
pub use crate::shared::time::GameTime;
pub use crate::shared::traits::RuleFailure;
```

**关键原则**：prelude 只导出**最常用**的 shared 类型，不导出全部。避免名称空间污染。

### Phase 2: 填充 5 个 TODO 桩模块

**优先级: P2 | 工时: 4h | 风险: 🟡**

5 个模块均为 P2 TODO（2026-06-20 标记），基于"三次才抽象"原则，只提供当前项目实际需要的功能，不设计通用库级抽象。

#### 2.1 `shared/collections/`

当前用法分析后，提供最急需的集合扩展：

- `GroupByMap` — 按 key 分组的扩展方法
- `TakeWhileInclusive` — 包含最后一个匹配元素的 take_while
- `PartitionMap` — 一次遍历分割为两个 Vec

#### 2.2 `shared/hashing/`

- `FastHasher` — aHash 封装（非加密高速哈希，用于 HashMap/HashSet 性能优化）
- 提供 `new_fast_hashmap()` / `new_fast_hashset()` 工厂函数

#### 2.3 `shared/math/`

- 网格距离计算（HexGrid 距离、Manhattan 距离）
- `FloatEq` — 浮点比较 trait（带 epsilon）
- 插值函数（lerp、inv_lerp、smoothstep）

#### 2.4 `shared/validation/`

- `ValidationResult<T, E>` — 链式校验结果类型
- `Validator<T, E>` — 校验器 trait
- `ValidationChain<T, E>` — 链式校验构造器

#### 2.5 `shared/path/`

- `config_dir()` — 项目配置目录路径
- `data_dir()` — 数据目录路径
- `ensure_dir()` — 确保目录存在的工具函数

### Phase 3: SharedPlugin 补全注册

**优先级: P2 | 工时: 0.3h | 风险: 🟢**

当前 SharedPlugin 只注册了 GameTime。需要补全注册确定性 RNG 资源：

```rust
// shared_plugin.rs
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameTime::default())
            .init_resource::<DeterministicRng>()    // 新增: FromWorld → seed=0
            .add_systems(PreUpdate, advance_game_time);
    }
}
```

### Phase 4: 文档修复

**优先级: P2 | 工时: 0.5h | 风险: 🟢**

1. **ADR-045 index 状态修正**: ADR index 中 "Proposed" → "Accepted"
2. **`testing/deterministic.rs` 注释更新**: "自实现 MurmurHash3" → "包装 ChaCha12 CSPRNG"
3. **`testing/deterministic.rs` 位置更新**: "core/.../replay/foundation/" → "shared/random/"

### Phase 5: Constants 扩展

**优先级: P3 | 工时: 0.3h | 风险: 🟢**

添加游戏全局共享常量。仅包含确实跨多个域使用的常量，域内常量留在对应域中。

候选:
- `MAX_PARTY_SIZE` — 队伍最大人数
- `MAX_INVENTORY_SIZE` — 背包最大格数
- `MAX_BUFF_STACK` — Buff 最大堆叠数（默认值）

### Phase 6: 跨 Infra 集成测试

**优先级: P3 | 工时: 2h | 风险: 🟡**

当前的 infra 模块测试独立，缺少跨组合的端到端验证：

1. **Save→Replay 往返测试**: 保存游戏 → 修改实体 → 回放 → 验证状态一致
2. **Pipeline→Registry 集成**: 管线中读取 Registry 配置并执行的流程测试
3. **Replay→RNG 确定性验证**: 多流 RNG 在回放中的确定性断言

---

## 三、执行状态

| Phase | 内容 | 优先级 | 状态 |
|-------|------|--------|------|
| Phase 1 | Prelude 填充 | P0 | ✅ 完成 |
| Phase 2.1 | collections/ 实现 | P2 | 🟡 执行中 |
| Phase 2.2 | hashing/ 实现 | P2 | ✅ 完成 |
| Phase 2.3 | math/ 实现 | P2 | ✅ 完成 |
| Phase 2.4 | validation/ 实现 | P2 | 🟡 执行中 |
| Phase 2.5 | path/ 实现 | P2 | ✅ 完成 |
| Phase 3 | SharedPlugin 补全 | P2 | ✅ 完成 |
| Phase 4 | 文档修复 | P2 | ✅ 完成 |
| Phase 5 | Constants 扩展 | P3 | ✅ 完成 |
| Phase 6 | 跨 Infra 集成测试 | P3 | 🟡 执行中 |

## 四、禁止项

| # | 禁止 | 原因 |
|---|------|------|
| 1 | prelude 导出全部 shared 模块 | 按需导出，避免名称污染 |
| 2 | 在 shared/ 中添加框架/运行时语义 | shared 是"零语义"层，不能引入 Bevy Resource/Component |
| 3 | 在 shared/ 中添加业务域常量 | 业务常量归各自的 domain，shared/constants/ 只放跨域基础常量 |
| 4 | 为未明确的未来需求设计完整抽象 | 三次才抽象，只解决当前复杂度 |
| 5 | infra 中使用 `#[cfg(test)] mod tests` 内联 | 测试必须放在 `tests/` 子目录 |
