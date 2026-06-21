# 宏治理激进重构计划

> **日期**: 2026-06-21 | **来源**: 11条宏治理原则
> **状态**: 全部完成 ✅ | 宪法§16.6 ✅ / ADR-063 ✅ / `src/macros.rs` 已拆除

---

## 一、已完成工作

### Phase 1: 拆除 `src/macros.rs` ✅
- `impl_rule_failure!` → `shared/traits/macros.rs`
- `impl_domain_event!` → `shared/diagnostics/macros.rs`
- `src/macros.rs` 已删除，`src/lib.rs` 引用已清理
- 31 个调用点零修改（`#[macro_export]` 确保跨模块可见）

### Phase 2: 宪法 §16.6 宏治理 ✅
11 条原则已写入宪法：抽象优先级 → 宏跟能力走 → 禁止跨层宏依赖 → Declarative/Procedural 分离 → Derive服务于Trait → Cargo Expand可读性 → 不隐藏业务逻辑 → 可被函数替代 → 禁止宏嵌套宏 → 准入门槛 → 10宏拆分

### Phase 3: 架构规则 ✅
`.trae/rules/架构规则.md` 已补充宏治理章节。

### Phase 4: ADR-063 ✅
已通过架构师审核，状态：**Accepted**，编号：ADR-063（11条原则）。

---

## 二、当前宏分布（重构后）

| 宏 | 位置 | 行数 | 原则合规 |
|----|------|------|---------|
| `emit_info!` / `emit_warn!` / `emit_debug!` | `infra/logging/telemetry.rs` | 40 | ✅ 跟能力走 + 核心实现在函数 |
| `warn_once!` / `error_once!` | `infra/logging/rate_limit/mod.rs` | 25 | ✅ 跟能力走 |
| `register_domain_types!` | `shared/macros.rs` | 30 | ✅ Shared 层合理 |
| `impl_rule_failure!` | `shared/traits/macros.rs` | 30 | ✅ Trait 旁 |
| `impl_domain_event!` | `shared/diagnostics/macros.rs` | 20 | ✅ Trait 旁 |
| `define_string_id!` / `define_numeric_id!` | `shared/ids/foundation/macros.rs` | 219 | ✅ ID体系旁 |
| `assert_*` (7个) | `shared/testing/assertions.rs` | 107 | ✅ 测试专用 |
| `#[derive(DomainEvent)]` 等(3个) | `fre-macros/src/lib.rs` | 215 | ✅ 独立crate |

**跨层检查**: core/domains/ 和 core/capabilities/ 中零 `emit_info!` 使用 ✅

---

## 三、当前代码改进方案

### 建议 1: 监控 `shared/testing/assertions.rs` 宏数量

**现状**: 已有 8 个宏（assert_approx_eq、assert_hp_non_negative、assert_effect_stage、assert_ok、assert_err、assert_err_matches、assert_tag_contains、assert_tag_not_contains），接近第 11 原则的 10 宏拆分阈值。

**方案**: 当达到 10 个时按主题拆分为：
```
shared/testing/
├── assertions.rs     ← 通用断言（6个）
├── ecs_assert.rs     ← ECS专用断言（assert_tag_contains/assert_tag_not_contains）
└── hp_assert.rs      ← 战斗专用断言（assert_hp_non_negative）
```

**优先级**: P3（达到阈值前仅监控）

---

### 建议 2: `define_string_id!` 宏拆分

**现状**: `/Users/lf380/Code/Bevy/Fre/src/shared/ids/foundation/macros.rs` 中 `define_string_id!` 宏复杂度过高（219 行），包含 StrongId / Display / FromStr / PartialEq / Serialize / Deserialize 的批量实现。

**方案**: 
```text
shared/ids/foundation/
├── macros.rs                   ← 宏入口 + define_numeric_id!
├── string_id_macros.rs         ← define_string_id! 拆出（~180行）
```

**优先级**: P3（功能正常运行，仅可维护性问题）

---

### 建议 3: 建立宏 Code Review Checklist

**现状**: 宪法 §16.6 已定义 11 条原则，但缺乏具体的 Review 执行清单。

**方案**: 在 ADR-063 或架构规则的执行层面增加 Review Checklist：

```
□ 原则1（抽象优先级）：检查是否可用 Trait/泛型/函数替代
□ 原则2（跟能力走）：宏文件位置是否正确
□ 原则3（跨层依赖）：Domain 是否使用了 Infra 宏
□ 原则7（不隐藏业务逻辑）：宏是否做了"决策"而非"重复"
□ 原则8（可被函数替代）：核心实现在函数还是宏中
□ 原则9（嵌套深度）：展开链是否超过 2 层
□ 原则10（准入门槛）：调用点是否 ≥ 5
```

**优先级**: P2

---

### 建议 4: fre-macros proc-macro 代码审查

**现状**: `macros/fre-macros/src/lib.rs`（215 行）包含 3 个 proc-macro，需要验证原则 5（Derive 服务于 Trait）和原则 6（Cargo Expand 可读性）。

**验证**:
- `#[derive(DomainEvent)]` → 仅生成 `impl DomainEvent for T {}` ✅
- `#[derive(RuleFailure)]` → 仅生成 `impl RuleFailure for T` ✅
- `#[derive(DefinitionType)]` → 仅生成 `impl DefinitionType for T` ✅

**结论**: 全部合规，但建议添加 cargo expand 测试确保展开结果可审查。

**优先级**: P3

---

### 建议 5: 跨层宏依赖门禁

**现状**: 当前 core/domains 和 core/capabilities 中无 `emit_info!` 使用，合规 ✅。但需要建立 CI 门禁防止未来引入。

**方案**: 添加简单的 CI 检查脚本：
```bash
#!/bin/bash
# check-cross-layer-macros.sh
# 禁止 Domain/Capability 层使用 Infra 宏
if grep -rn "emit_info!\|emit_warn!\|emit_debug!" src/core/ --include="*.rs" | grep -v "/tests/"; then
  echo "❌ 跨层宏违规：Domain/Capability 不得使用 Infra 日志宏"
  exit 1
fi
echo "✅ 跨层宏检查通过"
```

**优先级**: P2

---

## 四、禁止项（更新版）

| # | 禁止 | 原因 |
|---|------|------|
| 1 | 一次性重构所有宏 | 已完成，无需再动 |
| 2 | 合并 `shared/macros.rs` 到子模块 | `register_domain_types!` 是横切宏，放 shared 层合理 |
| 3 | 为 <5 调用点的模式创建宏 | 违反原则 10 |
| 4 | 创建 Helper Macro | 隐藏控制流，违反原则 7 |
| 5 | proc-macro crate 依赖主 crate | 循环依赖 |
| 6 | Domain 层调用 Infra 宏 | 违反原则 3（禁止跨层宏依赖） |
| 7 | 业务宏嵌套业务宏 | 违反原则 9（展开深度 > 2 层） |
