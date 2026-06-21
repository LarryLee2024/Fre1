# 宏治理激进重构计划

> **日期**: 2026-06-21 | **来源**: 7条宏治理原则
> **问题**: `src/macros.rs` 万能宏垃圾场反模式

---

## 一、当前宏分布审计

### 健康（已跟能力走）

| 宏 | 位置 | 所属模块 | 调用点 |
|----|------|---------|--------|
| `emit_info!` / `emit_warn!` / `emit_debug!` | `infra/logging/telemetry.rs` | 日志 | 500+ |
| `warn_once!` / `error_once!` | `infra/logging/rate_limit/mod.rs` | 日志限频 | 20+ |
| `define_string_id!` / `define_numeric_id!` | `shared/ids/foundation/macros.rs` | ID系统 | 25+ |
| `assert_approx_eq!` / `assert_*` (6个) | `shared/testing/assertions.rs` | 测试 | 100+ |
| `register_domain_types!` | `shared/macros.rs` | 类型注册 | 15+ |
| `derive_*` (3个 proc-macro) | `macros/fre-macros/src/lib.rs` | 独立crate | 30+ |

### 不健康（违反第一原则）

| 宏 | 当前位置 | 应归属 | 调用点 |
|----|---------|--------|--------|
| `impl_domain_event!` | `src/macros.rs` | `shared/diagnostics/` | 14 |
| `impl_rule_failure!` | `src/macros.rs` | `shared/traits/` | 17 |

---

## 二、执行计划

### Phase 1: 拆除 `src/macros.rs`

#### 1.1 `impl_rule_failure!` → `shared/traits/macros.rs`
- 新建 `src/shared/traits/macros.rs`
- 移入宏定义
- 在 `src/shared/traits/mod.rs` 中添加 `pub(crate) mod macros;`
- 更新所有调用点导入路径

#### 1.2 `impl_domain_event!` → `shared/diagnostics/macros.rs`
- 新建 `src/shared/diagnostics/macros.rs`
- 移入宏定义
- 在 `src/shared/diagnostics/mod.rs` 中添加 `pub(crate) mod macros;`
- 更新所有调用点导入路径

#### 1.3 删除 `src/macros.rs`
- 确认无其他引用后删除
- 同步删除 `src/lib.rs` 或根 mod 中的引用

### Phase 2: 宪法 §16.6 新增宏治理

在宪法 §16.5 之后新增 §16.6：

```
### 16.6 宏治理宪法

#### 第一原则：宏跟能力走
- 🟥 禁止建立全局 `src/macros/` 目录或万能宏文件
- 🟩 macro_rules! 必须归属于其服务的具体能力模块（如 infra/logging/macros.rs）
- 🟩 任何宏文件的首选位置是它服务的 trait/能力定义旁边

#### 第二原则：Declarative vs Procedural 分离
- 🟩 macro_rules! 声明式宏留在主 crate 的各模块中
- 🟩 proc_macro 必须放独立 crate（fre_macros/）
- 🟥 禁止在 proc-macro crate 中定义 macro_rules! 再 re-export

#### 第三原则：宏不得隐藏业务逻辑
- 🟩 宏只能用于：注册/派生/埋点/DSL/样板代码消除
- 🟥 禁止创建隐藏控制流的 Helper Macro（ok_or_return!、try_get!、some_or_continue!）
- 🟥 禁止用宏封装业务逻辑（do_damage!、spawn_enemy!、apply_buff!）

#### 第四原则：宏准入门槛
- 🟩 调用点 < 5 处：用函数，禁止引入宏
- 🟩 调用点 5~20 处：考虑泛型或函数
- 🟩 调用点 20+ 处：考虑宏
- 🟩 调用点 100+ 处：考虑 proc-macro derive
- 🟩 新增 proc-macro 必须经 ADR 审批

#### 第五原则：宏文件超过 10 个宏必须拆分
- 🟩 单文件 macro_rules! 超过 10 个时按主题拆分子文件
- 🟩 超过 50 行宏逻辑必须抽取帮助函数
```

### Phase 3: 架构规则更新

在 `.trae/rules/架构规则.md` 中补充宏治理章节。

### Phase 4: ADR 创建

创建 ADR-0XX-macro-governance，记录宏治理的架构决策。

---

## 三、禁止项

| # | 禁止 | 原因 |
|---|------|------|
| 1 | 一次性重构所有宏 | 当前仅 2 个宏位置不对，其他宏已跟能力走，无需动 |
| 2 | 合并 `shared/macros.rs` 到子模块 | `register_domain_types!` 是横切注册宏，放 shared 层合理 |
| 3 | 为 <5 调用点的模式创建宏 | 违反第四原则 |
| 4 | 创建 Helper Macro | 隐藏控制流，增加认知负担 |
| 5 | proc-macro crate 依赖主 crate | 会导致循环依赖 |
