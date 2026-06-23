---
id: CODING-CONSTITUTION
title: 代码组织与编写规范宪法
status: accepted
stability: stable
layer: governance
related:
  - ai-constitution-complete.md
tags:
  - coding
  - trait
  - macro
  - todo
  - naming
---

> **原文来源**：`ai-constitution-complete.md` 第十六编（L1341-L1569）
> **锚定总宪法**：第十六编

## 第十六编 代码组织与编写规范
### 16.1 代码组织规则
- 🟩 单一文件单一主题
- 🟩 优先按业务主题拆分文件，🟥 绝对禁止按技术类型拆分全局文件
- 🟥 绝对禁止创建全局顶层的 `systems.rs`、`components.rs` 巨文件
- 🟥 绝对禁止创建 `utils.rs`、`helpers.rs`、`common.rs` 垃圾桶文件
- ⚠️ 文件大小阈值：单文件超过 500 行且内聚性下降时主动提出拆分，超过 1000 行强制评估
- 🟩 AI 可读性优先，优先使用直白的线性逻辑，避免宏套宏、深度泛型、类型体操

### 16.2 函数设计宪法
- 🟩 单一职责：每个函数只能有一个主要职责
- 🟩 函数命名必须描述意图，而非描述实现过程
- 🟩 优先使用 Early Return 减少嵌套
- ⚠️ 超过 3 层嵌套时主动提出重构建议
- 🟩 代码重复出现三次以上时才进行抽象
- 🟩 可读性优先，其次才考虑复用性

### 16.3 Trait 宪法
- 🟩 Trait 用于定义能力或标记类型特征
- 🟥 禁止用Trait模拟类型层级（如 UnitTypeTrait 定义单位子类型），这违反"组合优于继承"
- 🟩 允许Marker Trait用于驱动自动注册系统（如 DomainEvent、ReplayEvent、AuditEvent），Marker Trait不携带行为，仅作为类型标签
- 🟩 Trait 只能用于定义需要扩展的接口，🟥 绝对禁止用于模拟继承树
- 🟥 绝对禁止为了"代码优雅"而创建无实际价值的 Trait
- 🟩 框架级trait（StrongId、RuleFailure、PipelineHook、ObservableEvent）必须使用Sealed Trait模式防止外部实现破坏不变量；设计为扩展点的trait（EffectHandler、ConditionChecker、DamageFormula）允许外部实现
- 🟩 关联类型优先原则：当trait的实现类型决定返回/错误/上下文类型时，必须使用关联类型（`type Error; type Output;`）而非泛型参数
- 🟩 Blanket Impl自动派生：当一个能力可由另一个能力自动推导时，必须使用blanket impl（`impl<T: Observable> Replayable for T {}`），禁止手动为每个类型重复实现

### 16.4 Feature 成熟度分级
- **Core 级**：核心战斗、角色、属性等基础系统，稳定性要求最高
- **Stable 级**：装备、任务、地图等成熟系统，新增功能不得破坏兼容性
- **Experimental 级**：玩法实验、辅助工具等功能，允许快速迭代
- 🟩 **运行时 Feature Flag**：复用现有 Semantic Tags（`sem:stable` / `sem:deprecated` / `sem:experimental`），不新增独立字段。运行时根据 Flag 过滤可用内容。

### 16.5 抽象与宏使用规范
- 🟩 **"三次才抽象"原则**：代码重复 3 次以上再抽象，且仅当业务语义相同（非仅函数签名相同或逻辑相似）。可读性优先于复用性。此原则针对运行时逻辑抽象，Typestate编译期类型安全保证不受此限
- 🟩 **宏只做重复结构**：宏只能用于消除声明式重复（如 `define_stat! { Health, Mana, Attack }`），🟥 禁止用宏生成业务逻辑
- 🟩 **Derive宏边界**：Derive宏只生成结构性样板代码（Trait impl的机械重复），属于"宏只做重复结构"的合法范畴；必须文档说明展开内容，`cargo expand`可查看；🟥 禁止derive宏生成包含业务判断的代码
- 🟩 **声明式宏 vs 过程宏**：声明式宏（`macro_rules!`）用于重复模式，允许；过程宏（`proc_macro`）用于代码生成，需 ADR 审批
- 🟩 **BSN 宏边界**：BSN 仅用于描述实体结构（组件组合），🟥 禁止在 BSN 中描述业务逻辑或引用 System/Observer（已由 ECS 规则 §3.7 强制）

### 16.6 宏治理宪法

#### 第一原则：抽象优先级
宏是最后手段，而非首选。优先级如下：
```
1. Trait  —— 定义能力接口
2. 泛型   —— 编译期多态
3. 函数   —— 行为复用
4. macro_rules! —— 声明式样板消除
5. proc_macro   —— 大规模代码生成
```
🟩 只有前者无法解决时才允许升级到下一级
🟩 优先 `register::<T>()` 而非 `register_event!(T)` 这类函数可替代的宏

#### 第二原则：宏跟能力走
- 🟥 禁止建立全局 `src/macros/` 目录或万能宏文件
- 🟩 macro_rules! 必须归属于其服务的具体能力模块（如 infra/logging/macros.rs）
- 🟩 任何宏文件的首选位置是它服务的 trait/能力定义旁边

#### 第三原则：禁止跨层宏依赖
- 🟥 Domain 层不得依赖 Infra 层的宏（如 emit_info! 不能在 core/domains/ 中使用）
- 🟥 Shared 层不得依赖 Core 层或 Infra 层的宏
- 🟩 宏依赖方向必须遵循项目分层：Shared ← Core ← Infra
- 🟩 跨层通信应通过 Observer + 事件，而非直接宏调用

#### 第四原则：Declarative vs Procedural 分离
- 🟩 macro_rules! 声明式宏留在主 crate 的各模块中
- 🟩 proc_macro 必须放独立 crate（fre_macros/）
- 🟥 禁止在 proc-macro crate 中定义 macro_rules! 再 re-export

#### 第五原则：Derive 必须服务于 Trait
- 🟩 #[derive(...)] 必须生成 Trait 实现，不得生成隐藏业务行为
- 🟥 禁止 derive 自动注册系统、自动执行逻辑、修改世界状态
- 🟩 允许的 derive：#[derive(DomainEvent)]、#[derive(Observable)]、#[derive(Replayable)]
- 🟥 禁止的 derive：#[derive(AutoCombat)]（展开后执行战斗逻辑）

#### 第六原则：Cargo Expand 可读性原则
- 🟩 cargo expand 后代码必须可读，接近手写 Rust
- 🟥 禁止生成难以调试的嵌套状态机代码
- 🟩 derive 宏的输出必须能让人直接审查正确性

#### 第七原则：宏不得隐藏业务逻辑
- 🟩 宏只能用于：注册/派生/埋点/DSL/样板代码消除
- 🟥 禁止创建隐藏控制流的 Helper Macro（ok_or_return!、try_get!、some_or_continue!）
- 🟥 禁止用宏封装业务逻辑（do_damage!、spawn_enemy!、apply_buff!）

#### 第八原则：宏必须可被函数替代
- 🟩 宏优先作为语法糖，核心逻辑必须位于普通函数
- 🟥 禁止将核心实现完全隐藏于宏展开中
- 🟩 示例：emit_info! 是好宏，因为真正实现是 telemetry::emit()；单元测试/mock/profile 走函数
- 🟥 反例：emit_info! 内部直接一百行日志格式化逻辑（无函数后备）

#### 第九原则：禁止宏嵌套宏
- 🟥 禁止业务宏调用业务宏（宏展开深度原则上不超过 2 层）
- 🟩 允许业务宏调用底层公共宏（如 emit_info! → tracing::info!）
- 🟩 超过 2 层展开链时，必须重构为函数调用

#### 第十原则：宏准入门槛
- 🟩 调用点 < 5 处：用函数，禁止引入宏
- 🟩 调用点 5~20 处：考虑泛型或函数
- 🟩 调用点 20+ 处：考虑宏
- 🟩 调用点 100+ 处：考虑 proc-macro derive
- 🟩 新增 proc-macro 必须经 ADR 审批

#### 第十一原则：宏文件超过 10 个宏必须拆分
- 🟩 单文件 macro_rules! 超过 10 个时按主题拆分子文件
- 🟩 超过 50 行宏逻辑必须抽取帮助函数

### 16.7 DomainEvent 演进路线图

DomainEvent 的标记方式随项目规模增长分三个阶段演进，禁止跳过阶段直接升级：

| 阶段 | 事件数量 | 方案 | 核心原则 |
|------|---------|------|---------|
| **阶段1**（当前50→关闭） | 20~50 | `impl_domain_event!()` 宏 | 显式可 grep，低级模板消除 |
| **阶段2**（当前） | 50~150 | Blanket Impl | 零宏零重复，Event+Debug+Clone 自动派生 |
| **阶段3**（未来） | 150+ | `#[derive(DomainEvent)]` + 元数据 | 只生成 const，不生成行为 |

#### 阶段2：Blanket Impl（当前推荐）
```rust
pub trait DomainEvent: Event + Debug + Clone + Send + Sync + 'static {}
impl<T> DomainEvent for T where T: Event + Debug + Clone + Send + Sync + 'static {}
```
任何 `#[derive(Event, Debug, Clone)]` 的 struct 自动是 DomainEvent，零样板代码。

#### 阶段3：#[derive(DomainEvent)] 准入铁律
当项目超过 150 个事件、需要 DOMAIN/CODE 等元数据时，才允许引入 derive。且必须遵守以下三条铁律：

🔒 **铁律1：只生成 const**
```rust
#[derive(DomainEvent)]
#[domain(Combat)]
#[code(COM001)]
pub struct TurnEnded;
// 展开后只生成：
// impl DomainEvent for TurnEnded {
//     const DOMAIN: Domain = Domain::Combat;
//     const CODE: EventCode = EventCode::COM001;
// }
```
❌ 禁止生成函数、系统、资源、spawn 逻辑

🔒 **铁律2：不允许访问 AST 语义**
- ❌ 禁止读取 struct 字段
- ❌ 禁止根据字段名/类型生成 impl
- ✅ 只允许 `#[...]` attribute + ident

🔒 **铁律3：必须可手写为等价代码**
- 🟩 cargo expand 后必须是能手动写出的 Rust 代码
- 🟩 禁止生成黑盒行为

### 16.8 TODO / FIXME / HACK 规范
🟥 禁止无上下文的 TODO/FIXME。结构化注释是 AI 协作项目的核心工程资产。

#### 格式定义
```rust
// TODO[优先级][领域][日期]:
// 原因: [问题说明]
// 完成条件: [如何验证完成]
// 关联ADR: [可选]
// 负责人: [可选]

// FIXME[优先级][领域][日期]:
// 问题: [Bug 描述]
// 复现步骤: [如何复现]

// HACK[关联ADR/原因]:
// [临时绕过说明]
// [何时删除]
```

#### 优先级标准
| 等级 | 含义 | 门禁要求 |
|------|------|----------|
| P0 | 必须修，阻塞发布 | 禁止进入主分支 |
| P1 | 一个迭代内解决 | CI 拦截 |
| P2 | 正常技术债 | 登记跟踪 |
| P3 | 可长期存在 | 无强制要求 |

#### TODO vs FIXME
- **TODO** = 缺功能（未来要做）
- **FIXME** = 有 Bug（已知问题）
- **HACK** = 已知丑陋但暂时无法避免的绕过

#### 合法示例
```rust
// TODO[P2][ATTRIBUTE][2026-06-16]:
// 原因: Aggregator 当前每次全量重算。
// 完成条件: Battle Benchmark 提升 20%+

// FIXME[P1][REPLAY][2026-06-16]:
// 问题: Buff dispel 时触发两次移除事件。
// 复现步骤: 施加 Buff → dispel → 观察事件日志

// HACK[ADR-021]:
// 临时绕过 Trigger 递归问题，待 Runtime v2 重构后删除。
```

#### 非法示例
```rust
// TODO: 优化        ← 无优先级、无领域、无日期、无上下文
// TODO: 重构        ← 不知道什么时候、为什么
// FIXME            ← 无任何信息
```

### 16.9 测试命名规范
- 🟩 测试函数名用英文 snake_case 描述预期行为，使用业务术语如 `damage_applies_armor_reduction`、`buff_removed_on_expiry`、`hp_never_goes_below_zero`
- 🟩 文件名保持英文 snake_case

合法示例：
```rust
#[test]
fn damage_applies_armor_reduction_correctly() { ... }

#[test]
fn buff_removed_on_expiry() { ... }

#[test]
fn hp_never_goes_below_zero() { ... }
```

非法示例：
```rust
#[test]
fn test_damage() { ... }              // 无业务语义
#[test]
fn a() { ... }                        // 无意义命名
```
