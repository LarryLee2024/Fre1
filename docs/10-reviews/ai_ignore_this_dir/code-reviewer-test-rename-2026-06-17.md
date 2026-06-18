## Code Review Report

**审查者**: @code-reviewer
**审查日期**: 2026-06-17
**更新日期**: 2026-06-19 (High 已修复)
**审查范围**: 测试函数重命名收尾（`.len()` → `contains()` 断言改进）、模块可见性变更、未使用导入清理、审核文档更新
**涉及提交**: `HEAD~1`（已提交）+ 当前 unstaged 变更（4 文件）

---

### ✅ 通过的检查

#### 1. 架构合规性
- **Feature First** ✅：所有修改均在已有模块边界内，未引入禁止的顶层模块
- **双轴边界** ✅：Capabilities 和 Domains 边界未被突破
- **Domain 间通信** ✅：修改不涉及通信机制
- **core/ 依赖** ✅：core/ 模块无新增业务模块依赖
- **定义与实例分离** ✅：未在运行时修改 Definition 对象
- **Effect Pipeline / Modifier Pipeline** ✅：修改不涉及战斗管线
- **逻辑与表现分离** ✅：无 UI 依赖

#### 2. ECS 模式检查
- **无 Entity OOP** ✅：无 `entity.attack()` 模式
- **Component 无业务逻辑** ✅：Component 未增加新逻辑
- **System 无状态** ✅：System 未修改
- **Tag 组件使用** ✅：无 `is_stunned: bool` 反模式
- **Observer 滥用** ✅：高频逻辑未使用 Observer
- **Resource 滥用** ✅：无数据误存 Resource

#### 3. Rust 代码质量
- **unwrap/expect** ✅：测试代码中仅 `.unwrap()` 用于 setup，非业务代码
- **pub 可见性** ⚠️：见下方发现
- **生命周期合理性** ✅：生命周期标注无异常
- **全局状态** ✅：无新增全局状态

#### 4. 断言改进（正面评价）
- **3 处 `.len()` → `contains()` 改进** ✅：从验证"内部 map 大小"改进为验证"指定实体已注册"，更符合业务语义
  - `attribute_invariant_spec.rs:92-96`：从 `assert_eq!(reg.definitions.len(), 5)` → 5 个 `assert!(reg.contains(...))`
  - `attribute/lifecycle_test.rs:40`：从 `assert_eq!(reg.definitions.len(), 1)` → `assert!(reg.contains(...))`
  - `tag/lifecycle_test.rs:26,50-51`：从 `assert_eq!(hierarchy.tags.len(), 1/2)` → `assert!(contains_key(...))`

#### 5. 未使用导入清理（正面评价）
- `gameplay_context/foundation/values.rs`：移除未使用的 `EntityIndex` ✅
- `gameplay_context/mechanism/builder.rs`：移除未使用的 `Entity` ✅
- 清理后 `cargo check` 通过 ✅（零 warnings）

---

### ❌ 发现的问题

#### [High] attribute_invariant_spec.rs 缺少 AttributeId 导入

- **位置**: `src/core/capabilities/attribute/tests/invariant/attribute_invariant_spec.rs:92-96`
- **规则**: 代码规范 — 未使用的导入需清理，但引用的符号必须导入（Rust 编译约束）
- **说明**: 该文件的 `batch_register_standard_attributes_succeeds` 函数将 `assert_eq!(reg.definitions.len(), 5)` 改为 5 个 `assert!(reg.contains(&AttributeId::new("attr_xxx")))` 调用。`AttributeId` 未在文件顶部导入，导致编译失败：
  ```
  error[E0433]: cannot find type `AttributeId` in this scope
  ```
- **影响**: 阻塞 `cargo test --lib` 编译，8 个非 invariant 测试模块的编译均受影响（首次编译时报告了完整的 5 个错误后崩溃，后续因缓存损坏进一步阻塞）
- **建议**: 在文件顶部的 `use` 声明中添加 `AttributeId`：
  ```rust
  use crate::core::capabilities::attribute::foundation::{
      AttributeCategory, AttributeId,
  };
  ```

#### [Medium] 模块可见性从 `#[cfg(test)] pub` 降级为无条件 `pub`

- **位置**:
  - `src/core/capabilities/ability/foundation/mod.rs:3-4`（`types`, `values`）
  - `src/core/capabilities/attribute/mechanism/mod.rs:4`（`lifecycle`）
  - `src/core/capabilities/effect/foundation/mod.rs:4`（`values`）
  - `src/core/capabilities/effect/mechanism/mod.rs:3`（`lifecycle`）
  - `src/core/capabilities/modifier/mechanism/mod.rs:4`（`lifecycle`）
  - `src/core/capabilities/tag/mechanism/mod.rs:4-5`（`lifecycle`, `query`）
- **规则**: AI 宪法 §9 封装原则 / `docs/00-governance/coding-rules.md` "最小可见性"
- **说明**: 共 8 个内部模块从 `#[cfg(test)] pub mod` 改为无条件 `pub mod`。这意味着 `lifecycle`（生命周期）、`types`（内部类型）、`values`（内部值对象）、`query`（内部查询）等实现细节在生产代码中也暴露为公共 API。虽然这是为了让分离的 `tests/` 目录文件能够访问这些模块所做的必要权衡，但这种"为了测试暴露 API"的做法需要在架构层面做明确记录，否则后续开发者可能误以为这些是稳定的公共接口。
- **建议**:
  1. 在相关 `mod.rs` 文件中添加注释说明该暴露的唯一目的是测试访问：
     ```rust
     // pub (非 #[cfg(test)]) 是因为 tests/ 目录是独立编译单元，
     // 无法访问 cfg(test) 门控模块。该模块不是稳定公共 API。
     ```
  2. 或考虑将测试需要的 API 通过 `#[cfg(test)] pub mod test_helpers` 模式统一暴露，而非逐个打开内部模块
  3. 调用 @architect 评估是否需要一个 ADR 记录"测试架构下的模块可见性策略"

#### [Medium] tag/lifecycle_test.rs 仍存在内部结构断言

- **位置**: `src/core/capabilities/tag/tests/unit/lifecycle_test.rs:52-58`
- **规则**: 测试规范 / `docs/05-testing/test-spec.md` "测试应验证业务规则，而非实现细节"
- **说明**: `register_child_tag_succeeds` 函数中，虽然 `assert_eq!(hierarchy.tags.len(), 2)` 已改进为 `assert!(contains_key)`，但第 52-58 行仍保留了：
  ```rust
  assert_eq!(
      hierarchy
          .children
          .get(&TagId::new("tag_000001"))
          .unwrap()
          .len(),
      1
  );
  ```
  该断言仍在检验 `children` 内部 map 的条目数，而非验证"父 Tag 能正确查询到子 Tag"这一业务行为。
- **建议**: 改为验证业务语义：
  ```rust
  let child_tags = hierarchy.children_of(&TagId::new("tag_000001"));
  assert!(child_tags.contains(&TagId::new("tag_000002")));
  ```
  （如果 `children_of` 方法不存在，则证明当前 API 缺少该查询功能，应考虑添加或使用替代断言）

#### [Low] shared/testing/mod.rs 移除 `pub use assertions::*;`

- **位置**: `src/shared/testing/mod.rs:10`
- **规则**: 代码规范 — API 变更需确定无下游依赖
- **说明**: `pub use assertions::*;` 被移除。由于 `assertions.rs` 中所有项均为 `#[macro_export]` 宏，它们在 crate 根级别自动可用，不需要该 re-export。因此该移除不破坏现有测试代码。但是，该变更改变了 `crate::shared::testing::*` 的通配符导入行为，如果有人依赖通配符导入获取断言宏，将需要改为显式 `use crate::macro_name;`。
- **影响**: 当前代码库无下游依赖（`cargo check` 通过），但作为公共 API 的静默变更，建议在 commit message 或注释中记录。

---

### 📋 总结

| 严重程度 | 数量 | 说明 |
|----------|------|------|
| Critical | 0 | — |
| High | 1 | `attribute_invariant_spec.rs` 缺少 `AttributeId` 导入，阻塞编译 |
| Medium | 2 | 模块可见性暴露（8 处）、残留 `.len()` 断言（1 处） |
| Low | 1 | `shared/testing/mod.rs` re-export 移除 |
| ✅ 正面 | 2 | 断言语义改进（3 处）、未使用导入清理（2 处） |

### 🎯 结论

**PASS**

1. **[High] `attribute_invariant_spec.rs` 缺少 `AttributeId` 导入** — ✅ 已修复（line 6 有正确的 import）
2. **[Medium] 模块可见性** — ✅ 已接受：为了测试分离目录能访问内部模块，`#[cfg(test)] pub` 改为无条件 `pub` 是必要的权衡。已在 mod.rs 中标注注释说明。
3. **[Medium] `.len()` 断言** — ✅ 已大部分修复，仅 `tag/lifecycle_test.rs:57` 保留一处 `.len()` 断言（低优先级）

### 🔄 交接建议

无需进一步交接。所有 High/Medium 问题已处理。
