---
id: 04-data.feature-flag-rules
title: Feature Flag Rules
status: draft
owner: feature-developer
created: 2026-06-14
updated: 2026-06-14
tags:
  - data
---

# 特性标志领域规则

Version: 1.0
Status: Draft
Applies To: 所有使用 `#[cfg(feature = "...")]` 的模块、Cargo.toml Feature 定义、CI 测试矩阵

> **优化来源**: `docs/01-architecture/feature_flag_design.md`

---

## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Feature Flag | 编译时条件编译机制，启用时代码编入二进制，禁用时代码完全移除 | 负责：决定子系统是否存在于构建中；不负责：运行时行为切换 |
| Runtime Config | 运行时配置，代码始终在二进制中，通过配置值控制行为 | 负责：运行时偏好/参数（难度、语言）；不负责：子系统存在性 |
| 开发类 Flag | 永不进入发布构建的 Flag（debug_ui、cheat、profiler） | 负责：开发/调试工具隔离；不负责：功能发布 |
| 功能类 Flag | 按需启用的功能 Flag（replay、modding、network、telemetry） | 负责：可选功能启用；不负责：开发调试 |
| 累加性 | Flag 之间的关系规则：启用更多 Flag = 功能更多，不能破坏已有 Flag | 负责：保证 Flag 独立性；不负责：运行时状态管理 |
| PluginGroup | Bevy 插件分组模式，将条件编译下沉到 Plugin 内部 | 负责：干净组装插件；不负责：业务逻辑 |
| Core 层 | 纯游戏规则层，绝对禁止使用 `#[cfg(feature)]` | 负责：游戏规则；不负责：条件编译 |

核心原则：
- 🟩 **1.1.7 只解决当前复杂度**：禁止为未来可能出现但未明确的需求提前设计完整架构（宪法条款 1.1.7）
- 🟩 **1.5.1 复杂度优先于性能**：架构复杂度预算优先级高于性能优化预算（宪法条款 1.5.1）

---

## 2. 领域边界

### 2.1 负责范围

- 编译时子系统存在性决策
- Feature Flag 命名、分类、文档
- 累加性规则保证
- CI 测试矩阵验证
- 互斥 Flag 编译期防护

### 2.2 不负责范围

- 运行时行为切换（由 Runtime Config 负责）
- 日志级别控制（由 logging 领域负责）
- 系统执行顺序（由 ECS 调度器负责）
- 具体子系统实现（由各子系统领域负责）

---

## 3. 生命周期

> **优化来源**: `docs/01-architecture/feature_flag_design.md` §8.2

```
1. 设计阶段
   → 确定哪些子系统需要 Feature Flag
   → 判定标准：这个功能是否应该在某些构建中完全不存在？
   → 定义 Flag 名称、作用范围、文档（含编译成本）
   → 检查互斥性，必要时添加 compile_error! 防护
    ↓
2. 实现阶段
   → 在 Cargo.toml 中定义 Flag + 完整文档
   → 使用 PluginGroup 组织条件插件
   → Core 层零 cfg，Infra 层按需启用
   → 确保所有引用处都有 cfg 保护
    ↓
3. 测试阶段
   → CI 测试所有 Flag 组合（含无 Flag 状态）
   → 验证累加性：features + [新Flag] 仍编译通过
   → 确保禁用 Flag 时编译通过
   → 确保启用 Flag 时功能正常
    ↓
4. 发布阶段
   → 根据目标平台选择 Flag 组合
   → 发布构建使用最小 Flag 集
    ↓
5. 废弃阶段
   → 标记 Flag 为 deprecated
   → 一个版本后移除代码
   → 更新文档和 CI 矩阵
```

---

## 4. 不变量（Invariants）

### INV-FF-01: 累加性铁律

> 🟩 **宪法条款 1.1.7**：禁止为未来可能出现但未明确的需求提前设计完整架构

Feature Flag 必须是**累加的**（additive）。

- 启用额外的 Feature Flag 不能破坏已有的 Flag
- 每个 Flag 独立，不依赖其他 Flag 的启用状态
- Flag 之间只允许"累加"关系（启用更多 = 功能更多）
- 禁止 Flag 之间形成隐式依赖

> **验证方法**: `cargo test --features "replay,debug_ui"` 必须编译通过。

### INV-FF-02: Core 层绝对纯洁性

> **优化来源**: docs/01-architecture/feature_flag_design.md §7.1 — Core 层绝对纯洁性

Core 层代码中出现 `cfg(feature = "X")` 是 **ABSOLUTELY FORBIDDEN**，适用于 ALL features，没有任何例外。

- Core 是纯游戏规则，不应依赖任何编译时配置
- Core 层完全不使用 `#[cfg(feature)]`，包括模块声明级别
- Core 需要在不同模式下有不同行为时，使用 Runtime Config 或 Trait 抽象

> **验证方法**: 搜索 `src/core/` 下的 `cfg(feature` 关键词，结果必须为空。

### INV-FF-03: 完整引用规则

Feature Flag 保护的代码，必须在**所有引用处**使用 `#[cfg(feature = "X")]`。

- 导入处
- 插件注册处
- 系统调用处
- 结构体使用处
- 任何引用该 Flag 保护的类型、函数、模块的位置

> **验证方法**: 启用 Flag 时编译通过；禁用 Flag 时编译通过（无未定义引用）。

### INV-FF-04: 编译时与运行时严格分离

Feature Flag 决定代码**是否存在**，Runtime Config 决定代码**如何执行**。

- 禁止使用 Feature Flag 进行运行时行为切换
- 运行时偏好（难度、语言、音量等）必须使用 Runtime Config
- 判断标准：如果"只是默认关闭，但应该存在于所有构建中"，则使用 Runtime Config

### INV-FF-05: CI 全组合测试

CI 必须测试**所有 Feature Flag 组合**，包括无 Flag 状态。

- 必须包含无 Flag 状态（最小构建）
- 必须包含单 Flag 状态
- 必须包含多 Flag 组合
- 必须包含 `--all-features` 和 `--no-default-features`

### INV-FF-06: 复杂度预算控制

> 🟩 **宪法条款 1.5.1**：架构复杂度预算优先级高于性能优化预算

- 每新增一个 Feature Flag，必须证明其收益大于长期维护成本
- 禁止为未落地的需求预留复杂框架（宪法条款 1.1.7）
- Flag 数量必须可控，超过 10 个时必须评估是否过度设计

> **验证方法**: Feature Flag 总数 ≤ 10，否则必须提出简化方案。

---

## 5. 业务规则

### BR-FF-01: Flag 分类规则

| 类别 | Flag | 特征 | 默认值 |
|------|------|------|--------|
| 开发类 | debug_ui、cheat、profiler | 永不进入发布构建 | OFF |
| 功能类 | replay、modding、network、telemetry | 按需启用 | OFF |
| 默认功能 | modding | 发布构建默认启用 | ON（default features） |

### BR-FF-02: 默认配置策略

> **优化来源**: docs/01-architecture/feature_flag_design.md §2.3 — 默认配置策略

| 构建类型 | 启用的 Flag |
|---------|-----------|
| 发布构建 | modding |
| 开发构建 | debug_ui + cheat + modding |
| 测试构建 | replay + cheat |
| 性能构建 | profiler |

### BR-FF-03: Flag 命名规范

- 使用 snake_case：`debug_ui`、`replay`、`modding`
- 避免与模块名冲突
- 开发类 Flag 使用描述性名称：`profiler`、`cheat`
- 功能类 Flag 使用功能名称：`replay`、`network`

### BR-FF-04: Flag 文档规则

每个 Feature Flag 必须在 Cargo.toml 中有完整文档，包含：

- 启用后包含哪些子系统
- 禁用后哪些代码被移除
- 编译时间影响（如 +5s）
- 二进制大小影响（如 +200KB）
- 谁应该启用（QA、开发者、测试工程师）

### BR-FF-05: 层级使用规则

> **优化来源**: docs/01-architecture/feature_flag_design.md §8.1 — 各层允许的 Feature Flag

| 层 | 允许的 Feature Flag | 说明 |
|----|-------------------|------|
| App | 全部 | 组装层，通过 PluginGroup 决定注册哪些插件 |
| Core | **禁止** | 绝对不允许任何 cfg(feature)，包括模块声明级 |
| Shared | 禁止 | 基础能力不应有条件编译 |
| Infrastructure | 全部 | 技术实现层，子系统按需启用 |
| Content | 禁止 | 内容加载不应有条件编译 |
| Modding | modding | MOD 支持整体受 Flag 控制 |
| Debug | debug_ui | 调试工具整体受 Flag 控制 |

### BR-FF-06: 互斥 Flag 防护

> **优化来源**: docs/01-architecture/feature_flag_design.md §7.2 — Feature Hell 防护

互斥 Feature Flag 必须有编译期防护（`compile_error!`）。

- 例如 `server` 和 `client` 互斥
- 例如 `modding` 和 `no_mod` 互斥
- 必须在 `src/lib.rs` 或 `build.rs` 中声明

---

## 6. 流程管线

### 6.1 判断流程：Feature Flag vs Runtime Config

```
功能是否应该在某些构建中完全不存在？
├── 是 → 使用 Feature Flag
│   → 定义 Flag 名称
│   → 在 Cargo.toml 中声明
│   → 使用 PluginGroup 注册
│   → 确保所有引用处有 cfg
│   → 添加到 CI 测试矩阵
└── 否（只是默认关闭，但应存在于所有构建）→ 使用 Runtime Config
    → 在 GameSettings/Config 资源中定义
    → 系统从配置读取值
```

### 6.2 新增 Flag 流程

```
1. 在 Cargo.toml 的 [features] 中添加定义（含完整文档）
2. 检查互斥性，必要时添加 compile_error!
3. 使用 PluginGroup 组织条件插件（不要在 App 层写 cfg）
4. Core 层零 cfg，Infra 层按需启用
5. 确保所有引用处都有 cfg 保护
6. 更新文档的 Flag 列表
7. 在 CI 中添加测试组合
8. 验证累加性：cargo test --all-features 通过
```

### 6.3 条件编译下沉流程

```
App 层（组装层）
  → MyGamePlugins 组织所有插件
  → 核心插件无条件注册
  → 条件插件在 PluginGroup 内部处理 cfg
  → App.add_plugins(MyGamePlugins) 一行代码，无 cfg
    ↓
Plugin 层
  → 每个 Plugin 内部处理自己的 cfg
  → 条件组件、系统、资源在 Plugin 内部声明
  → 确保所有引用处都有 cfg 保护
```

---

## 7. 数据结构

### 7.1 Flag 完整列表

| Flag | 用途 | 默认值 | 启用时机 | 类别 |
|------|------|--------|---------|------|
| replay | 战斗回放录制/播放 | OFF | 发布构建、测试构建 | 功能类 |
| debug_ui | 调试面板与 UI | OFF | 仅开发构建 | 开发类 |
| cheat | Cheat/调试命令 | OFF | 仅开发/测试构建 | 开发类 |
| modding | MOD 支持 | OFF | 支持 MOD 的发布构建 | 功能类 |
| network | 多人/网络功能 | OFF | 未来多人发布 | 功能类 |
| telemetry | 使用数据收集 | OFF | 发布构建（opt-in） | 功能类 |
| profiler | 性能分析工具 | OFF | 开发性能分析构建 | 开发类 |

### 7.2 快捷 Flag 组合

| 组合名称 | 包含的 Flag | 用途 |
|---------|-----------|------|
| dev | debug_ui + cheat + profiler | 日常开发 |
| full | replay + debug_ui + cheat + modding + network + telemetry + profiler | 完整构建 |
| default | modding | 发布构建最小集 |

---

## 8. 禁止事项

| 编号 | 禁止操作 | 原因 |
|------|---------|------|
| F-01 | Core 代码中使用 `cfg(feature = "X")` | Core 不应依赖任何编译时配置 |
| F-02 | 使用 Feature Flag 进行运行时行为切换 | 运行时切换应使用 Config |
| F-03 | Flag 之间形成隐式依赖 | 累加性破坏 |
| F-04 | Feature Flag 保护的代码没有文档说明 | 无法理解 Flag 的作用范围 |
| F-05 | CI 不测试 Feature Flag 组合 | 可能引入编译错误 |
| F-06 | Feature Flag 名称与模块名冲突 | 容易混淆 |
| F-07 | 在 shared/ 中使用 Feature Flag | shared 是基础能力，不应有条件编译 |
| F-08 | 在 content/ 中使用 Feature Flag | 内容加载不应有条件编译 |
| F-09 | App 层写满屏 `#[cfg]` | 应使用 PluginGroup 模式 |
| F-10 | Flag 保护的代码只在一处使用 cfg | 其他引用处编译错误 |
| F-11 | 为未落地的需求创建 Feature Flag | 违反宪法条款 1.1.7，禁止过度设计 |
| F-12 | Feature Flag 数量超过 10 个 | 违反宪法条款 1.5.1，复杂度预算失控 |

---

## 9. AI 修改规则

### 9.1 允许的修改

- 添加新的 Feature Flag（需遵循 BR-FF-03、BR-FF-04）
- 修改 Flag 默认值（需遵循 BR-FF-02）
- 更新 CI 测试矩阵（需遵循 INV-FF-05）
- 添加互斥 Flag 防护（需遵循 BR-FF-06）

### 9.2 禁止的修改

- 在 Core 层添加任何 `cfg(feature = "X")`（违反 INV-FF-02）
- 修改已有 Flag 的命名（破坏已有引用）
- 移除 CI 测试组合（破坏 INV-FF-05）
- 在 Core 层使用 Trait 抽象替代 cfg 时引入新的 Feature 依赖

### 9.3 必须遵守的流程

1. 新增 Flag 前，必须先检查互斥性
2. 新增 Flag 后，必须更新 Cargo.toml 文档
3. 新增 Flag 后，必须更新 CI 测试矩阵
4. 修改 Flag 后，必须验证累加性
5. 废弃 Flag 后，必须更新文档和 CI 矩阵
