# Feature Flag Design — 特性标志架构设计

Version: 1.1
Status: Proposed

来源：`docs/其他/33遗漏2.md` D17

### 宪法条款映射

| 本文档规则 | 宪法条款 | 强制等级 |
|-----------|---------|---------|
| §3.1 PluginGroup 模式 | 🟥 1.3.2 依赖方向铁则 | 必须遵循 |
| §6.3 Core 层零 cfg | 🟥 1.1.4 逻辑与表现分离 | 必须遵循 |
| §7.1 Core 层绝对纯洁性 | 🟥 1.3.2 依赖方向铁则 | 必须遵循 |
| §5.1 累加性规则 | 🟩 1.1.7 只解决当前复杂度 | 必须遵循 |
| §5.3 CI 测试规则 | 🟩 20.7.1 主分支准入标准 | 必须遵循 |

交叉引用：`docs/architecture/layer-contracts.md`、`docs/architecture/project-structure.md`、`docs/architecture/infrastructure-design.md`

---

## 1. 目的

### 1.1 什么是 Feature Flag

Feature Flag 是**编译时**条件编译机制，用于在构建时决定是否包含某个子系统。它与运行时配置（Runtime Config）不同：

- **Feature Flag**：编译时决定，启用时代码编入二进制，禁用时代码完全移除
- **Runtime Config**：运行时决定，代码始终存在于二进制中，通过配置值控制行为

### 1.2 为什么需要 Feature Flag

1. **减小发布包体积**：调试工具、cheat 命令不进入发布构建
2. **减少编译时间**：开发时只编译需要的子系统
3. **安全隔离**：cheat、debug 等敏感功能不暴露给玩家
4. **可选功能**：MOD 支持、网络功能可选择性启用
5. **性能优化**：移除未使用的子系统减少运行时开销

### 1.3 Feature Flag vs Runtime Config

| 维度 | Feature Flag | Runtime Config |
|------|-------------|---------------|
| **作用范围** | 编译时，从二进制移除 | 运行时，始终在二进制中 |
| **使用场景** | 可选子系统（回放、网络） | 用户偏好（难度、语言） |
| **变更方式** | 需要重新编译 | 热重载 |
| **二进制影响** | 禁用时代码不存在 | 代码始终存在 |
| **典型示例** | `#[cfg(feature = "replay")]` | `Res<GameSettings>.enable_hints` |

### 1.4 判断标准

> **核心问题**：这个功能是否应该在某些构建中完全不存在？

如果**是** → 使用 Feature Flag。
如果**否**（只是默认关闭，但应该存在于所有构建中） → 使用 Runtime Config。

---

## 2. Feature Flag 分类

### 2.1 完整列表

| Flag | 用途 | 默认值 | 启用时机 |
|------|------|--------|---------|
| `replay` | 战斗回放录制/播放 | OFF | 发布构建、测试构建 |
| `debug_ui` | 调试面板与 UI | OFF | 仅开发构建 |
| `cheat` | Cheat/调试命令 | OFF | 仅开发/测试构建 |
| `modding` | MOD 支持 | OFF | 支持 MOD 的发布构建 |
| `network` | 多人/网络功能 | OFF | 未来多人发布 |
| `telemetry` | 使用数据收集 | OFF | 发布构建（opt-in） |
| `profiler` | 性能分析工具 | OFF | 开发性能分析构建 |

### 2.2 分类维度

#### 开发类（永不进入发布构建）

```
debug_ui   — 调试面板、World Inspector、状态查看
cheat      — 无敌、无限资源、跳关等调试命令
profiler   — 性能分析、内存追踪
```

#### 功能类（按需启用）

```
replay     — 战斗回放录制与播放
modding    — MOD 支持与加载
network    — 多人联网功能
telemetry  — 使用数据收集
```

### 2.3 默认配置策略

- **发布构建**：仅启用 `modding`（如果支持 MOD）
- **开发构建**：启用 `debug_ui` + `cheat` + `modding`
- **测试构建**：启用 `replay` + `cheat`
- **性能构建**：启用 `profiler`

---

## 3. Feature Flag 使用模式

### 3.1 插件注册（PluginGroup 模式）

> **优化来源**：`docs/其他/51.md` — "App 层 register_plugins 的意大利面条化"、"PluginGroup + cfg gate"。

🟥 **App 层禁止写满屏 `#[cfg]`**。使用 Bevy 的 `PluginGroup` 将条件编译下沉到 Plugin 内部：

```rust
// src/app/plugins.rs — 干净、无 cfg
pub struct MyGamePlugins;

impl PluginGroup for MyGamePlugins {
    fn build(self) -> PluginGroupBuilder {
        let mut group = PluginGroupBuilder::start::<Self>();
        
        // 核心插件：无条件注册
        group = group.add(CorePlugin)
            .add(BattlePlugin)
            .add(SkillPlugin)
            .add(BuffPlugin)
            .add(TurnPlugin);
        
        // 条件插件：在 PluginGroup 内部处理 cfg
        #[cfg(feature = "replay")] { group = group.add(ReplayPlugin); }
        #[cfg(feature = "debug_ui")] { group = group.add(DebugPlugin); }
        #[cfg(feature = "modding")] { group = group.add(ModdingPlugin); }
        #[cfg(feature = "network")] { group = group.add(NetworkPlugin); }
        #[cfg(feature = "telemetry")] { group = group.add(TelemetryPlugin); }
        #[cfg(feature = "profiler")] { group = group.add(ProfilerPlugin); }
        
        group
    }
}

// App 层组装：一行代码，无 cfg
pub fn setup_app(app: &mut App) {
    app.add_plugins(MyGamePlugins);
}
```

**优势**：
- App 层完全干净，无条件编译污染
- 新增 Feature 只需修改 PluginGroup，不影响 App 组装
- 每个 Plugin 内部处理自己的 cfg，职责清晰

### 3.2 系统注册

```rust
// src/app/plugins.rs

pub fn register_systems(app: &mut App) {
    // 始终启用的核心系统
    app.add_systems(Update, (
        turn_system,
        battle_system,
        skill_system,
    ));

    // 条件启用的系统
    #[cfg(feature = "cheat")]
    app.add_systems(Update, cheat_command_system);

    #[cfg(feature = "replay")]
    app.add_systems(Update, (
        replay_recording_system,
        replay_playback_system,
    ));

    #[cfg(feature = "profiler")]
    app.add_systems(Update, profiler_overlay_system);
}
```

### 3.3 结构体与组件

```rust
// 使用 Feature Flag 的结构体

#[cfg(feature = "replay")]
#[derive(Component)]
pub struct ReplayRecorder {
    pub frames: Vec<ReplayFrame>,
}

#[cfg(feature = "debug_ui")]
#[derive(Component)]
pub struct DebugOverlay {
    pub visible: bool,
}

#[cfg(feature = "telemetry")]
#[derive(Resource)]
pub struct TelemetryConfig {
    pub endpoint: String,
    pub enabled: bool,
}
```

### 3.4 资源与事件

```rust
// Feature Flag 保护的资源

#[cfg(feature = "network")]
#[derive(Resource)]
pub struct NetworkState {
    pub connected: bool,
    pub peer_id: String,
}

// Feature Flag 保护的事件

#[cfg(feature = "replay")]
#[derive(Event)]
pub struct ReplayFrameRecorded {
    pub frame: ReplayFrame,
}
```

---

## 4. Cargo.toml 配置

### 4.1 Feature 定义

```toml
[features]
# 默认启用 MOD 支持
default = ["modding"]

# 战斗回放
replay = []

# 调试面板
debug_ui = []

# Cheat 命令
cheat = []

# MOD 支持
modding = []

# 多人网络
network = []

# 使用数据收集
telemetry = []

# 性能分析
profiler = []

# 开发构建快捷方式（启用所有开发工具）
dev = ["debug_ui", "cheat", "profiler"]

# 完整构建（启用所有功能）
full = ["replay", "debug_ui", "cheat", "modding", "network", "telemetry", "profiler"]
```

### 4.2 条件依赖

某些 Feature Flag 需要额外的依赖：

```toml
[dependencies]
# 基础依赖（始终启用）
bevy = "0.18.1"
serde = { version = "1", features = ["derive"] }
ron = "0.8"

# Feature Flag 条件依赖
[dependencies.tracing]
version = "0.1"
optional = true

[dev-dependencies]
# 测试依赖
criterion = "0.5"
```

### 4.3 构建命令

```bash
# 发布构建（仅 MOD 支持）
cargo build --release

# 开发构建（调试 + Cheat + Profiler）
cargo run --features dev

# 完整构建（所有功能）
cargo build --features full

# 仅回放功能
cargo build --features replay

# 多人网络（未来）
cargo build --features network
```

---

## 5. Feature Flag 规则

### 5.1 累加性规则

Feature Flag 必须是**累加的**（additive）：

- 启用额外的 Feature Flag 不能破坏已有的 Flag
- 每个 Flag 独立，不依赖其他 Flag 的启用状态
- Flag 之间只允许"累加"关系（启用更多 = 功能更多）

```rust
// 🟩 正确：每个 Flag 独立
#[cfg(feature = "replay")]
fn replay_system() { /* ... */ }

#[cfg(feature = "debug_ui")]
fn debug_system() { /* ... */ }

// 🟥 错误：Flag 之间有隐式依赖
#[cfg(feature = "replay")]
fn replay_needs_debug() {
    // 假设 debug_ui 一定启用 → 累加性破坏
}
```

### 5.2 完整引用规则

Feature Flag 保护的代码，必须在**所有引用处**使用 `#[cfg(feature = "X")]`：

```rust
// 🟩 正确：所有引用处都有 cfg

// 导入
#[cfg(feature = "replay")]
use crate::replay::ReplayPlugin;

// 插件注册
#[cfg(feature = "replay")]
app.add_plugins(ReplayPlugin);

// 系统调用
#[cfg(feature = "replay")]
fn replay_system() { /* ... */ }

// 结构体使用
#[cfg(feature = "replay")]
fn process_replay(recorder: &ReplayRecorder) { /* ... */ }

// 🟥 错误：只在一处使用 cfg
use crate::replay::ReplayPlugin;  // ← 没有 cfg 保护

#[cfg(feature = "replay")]
app.add_plugins(ReplayPlugin);  // ← 编译错误：ReplayPlugin 不存在
```

### 5.3 CI 测试规则

> **优化来源**：`docs/其他/51.md` — "CI matrix: test ALL flag combinations"。

CI 必须测试**所有 Feature Flag 组合**，包括无 Flag 状态：

```yaml
# .github/workflows/test.yml
strategy:
  matrix:
    features:
      - ""                           # 无 Flag（最小构建）
      - "replay"                     # 仅回放
      - "debug_ui"                   # 仅调试
      - "cheat"                      # 仅 Cheat
      - "modding"                    # 仅 MOD
      - "replay,debug_ui"            # 回放 + 调试
      - "replay,cheat"               # 回放 + Cheat
      - "debug_ui,cheat"             # 调试 + Cheat
      - "dev"                        # 开发模式
      - "full"                       # 完整模式

steps:
  - name: Test with features
    run: cargo test --features ${{ matrix.features }}
    
  - name: Test no-default-features
    run: cargo test --no-default-features
    
  - name: Test all-features
    run: cargo test --all-features
```

### 5.4 Feature 文档规则

> **优化来源**：`docs/其他/51.md` — "Feature documentation: what it enables, compile-time cost, who should enable it"。

每个 Feature Flag 必须在 Cargo.toml 中有完整文档：

```toml
[features]
# 战斗回放录制与播放功能
# 启用后：ReplayPlugin、ReplayRecorder 组件、回放系统
# 禁用后：回放相关代码完全移除
# 编译时间影响：+5s（增加 serde 序列化依赖）
# 二进制大小影响：+200KB（回放序列化代码）
# 谁应该启用：QA、测试工程师、需要回放调试的开发者
replay = []

# 调试面板和 UI 功能（仅开发构建）
# 启用后：DebugPlugin、egui 调试面板、World Inspector
# 禁用后：调试 UI 完全移除
# 编译时间影响：+15s（egui 依赖较重）
# 二进制大小影响：+2MB（egui 渲染器）
# 谁应该启用：日常开发
debug_ui = []
```

### 5.5 文档规则（扩展）

每个 Feature Flag 必须在 Cargo.toml 中有注释说明：

```toml
[features]
# 战斗回放录制与播放功能
# 启用后：ReplayPlugin、ReplayRecorder 组件、回放系统
# 禁用后：回放相关代码完全移除
replay = []

# 调试面板和 UI 功能（仅开发构建）
# 启用后：DebugPlugin、egui 调试面板、World Inspector
# 禁用后：调试 UI 完全移除
debug_ui = []
```

---

## 6. Feature Flag 在各层的使用

### 6.1 App 层

```rust
// src/app/plugins.rs — 插件注册
#[cfg(feature = "replay")]
app.add_plugins(ReplayPlugin);

#[cfg(feature = "debug_ui")]
app.add_plugins(DebugPlugin);
```

### 6.2 Infrastructure 层

```rust
// src/infrastructure/replay/mod.rs
#[cfg(feature = "replay")]
pub mod replay_recorder;

#[cfg(feature = "replay")]
pub mod replay_playback;

#[cfg(feature = "telemetry")]
pub mod telemetry_collector;

#[cfg(feature = "network")]
pub mod network_sync;
```

### 6.3 Core 层

🟥 **Core 层禁止使用 Feature Flag 作为业务逻辑分支**（详见第 7 节）。

Core 层**完全不使用 `#[cfg(feature)]`**，包括模块声明级别。所有条件编译推到 Plugin 层。

```rust
// src/core/mod.rs — 干净，无 cfg
pub mod skill;
pub mod buff;
pub mod battle;
pub mod character;
pub mod equipment;
pub mod inventory;
pub mod turn;
pub mod ai;

// 🟥 禁止：Core 中的任何 cfg
// #[cfg(feature = "replay")]
// pub mod replay_domain;  // ← 绝对禁止
```

#### Core 与 Infra 的 Trait 抽象

当 Core 层需要与 Infra 层交互时，使用 **Trait + NoOp 实现**替代 `#[cfg]` 宏：

```rust
// Core 层定义端口 trait
// core/telemetry/port.rs
pub trait TelemetryPort {
    fn report(&self, event: &GameEvent);
}

// Infra 层提供真实实现（telemetry feature 启用时）
// infrastructure/telemetry/real_telemetry.rs
pub struct RealTelemetry;
impl TelemetryPort for RealTelemetry {
    fn report(&self, event: &GameEvent) { /* ... */ }
}

// App 层注入 NoOp 实现（telemetry feature 禁用时）
pub struct NoOpTelemetry;
impl TelemetryPort for NoOpTelemetry {
    fn report(&self, _event: &GameEvent) { /* 空实现 */ }
}
```

**优势**：Core 层代码零 cfg 污染，运行时多态替代编译时宏，代码更干净、可测试。

### 6.4 Debug 层

```rust
// src/debug/mod.rs
// Debug 层整体受 debug_ui Feature Flag 保护
#[cfg(feature = "debug_ui")]
pub mod panels;

#[cfg(feature = "debug_ui")]
pub mod viewers;

#[cfg(feature = "debug_ui")]
pub struct DebugPlugin;
```

---

## 7. 禁止事项

### 7.1 Core 层绝对纯洁性

> **优化来源**：`docs/其他/51.md` — "Core 层绝对纯洁性"、"cfg(feature) in Core is ABSOLUTELY FORBIDDEN for ALL features"。

🟥 **Core 层代码中出现 `cfg(feature = "X")` 是 ABSOLUTELY FORBIDDEN，适用于 ALL features，没有任何例外。**

Core 是纯游戏规则，不应依赖任何编译时配置。如果 Core 需要在不同模式下有不同行为，应使用 Runtime Config 或 Trait 抽象。

```rust
// 🟥 严禁：Core 中的任何 Feature Flag
fn calculate_damage(skill: &SkillDef) -> i32 {
    #[cfg(feature = "replay")]
    let damage = base_damage * 1.5;  // ← 绝对禁止
    #[cfg(not(feature = "replay"))]
    let damage = base_damage;
    damage
}

// 🟥 严禁：Core 中的模块级 Feature Flag
#[cfg(feature = "replay")]
pub mod replay_domain;  // ← 绝对禁止

// 🟩 正确：Core 逻辑与 Feature Flag 无关
fn calculate_damage(skill: &SkillDef) -> i32 {
    base_damage  // 无论是否启用任何 feature，逻辑一致
}

// 🟩 正确：使用 Trait 抽象替代 cfg
// Core 层定义 trait TelemetryPort { fn report(...); }
// Infra 层在 telemetry feature 启用时提供真实实现
// 未启用时 App 层注入 NoOpTelemetry（空实现）
```

### 7.2 Feature Hell 防护

> **优化来源**：`docs/其他/51.md` — "Feature Hell prevention"、"compile_error! for mutually exclusive features"。

🟥 **互斥 Feature 必须有编译期防护**：

```rust
// src/lib.rs 或 build.rs
// 互斥 Feature 检测
#[cfg(all(feature = "server", feature = "client"))]
compile_error!("Features 'server' and 'client' are mutually exclusive");

#[cfg(all(feature = "modding", feature = "no_mod"))]
compile_error!("Features 'modding' and 'no_mod' are mutually exclusive");
```

### 7.3 累加性铁律验证

> **优化来源**：`docs/其他/51.md` — "Additive iron rule"。

🟥 **CI 必须验证累加性**：`features + [debug_ui]` 仍能编译通过。

```yaml
# CI 累加性测试
- name: Test additive rule
  run: |
    cargo test --no-default-features
    cargo test --features debug_ui
    cargo test --features "replay,debug_ui"
    cargo test --all-features
```

### 7.4 Feature Flag 禁止形成依赖

🟥 **禁止**：Feature Flag 之间形成依赖关系（除累加关系外）。

```rust
// 🟥 严禁：Flag 之间的隐式依赖
#[cfg(all(feature = "replay", feature = "debug_ui"))]
fn replay_debug_only() {
    // 假设 replay 一定与 debug_ui 同时启用 → 累加性破坏
}
```

### 7.5 Feature Flag 禁止用于运行时切换

🟥 **禁止**：Feature Flag 被用于运行时行为切换。

```rust
// 🟥 严禁：运行时切换应使用 Config
fn get_difficulty() -> Difficulty {
    #[cfg(feature = "cheat")]
    return Difficulty::Easy;  // ← 这是运行时偏好，不是编译时选择
    #[cfg(not(feature = "cheat"))]
    return Difficulty::Normal;
}

// 🟩 正确：运行时行为使用 Config
fn get_difficulty(settings: &GameSettings) -> Difficulty {
    settings.difficulty  // 从运行时配置读取
}
```

### 7.6 其他禁止事项

| 禁止操作 | 原因 |
|----------|------|
| Core 代码中使用 `cfg(feature = "ui")` 作为业务逻辑分支 | Core 不应依赖 UI 是否存在 |
| Feature Flag 保护的代码没有文档说明 | 无法理解 Flag 的作用范围 |
| CI 不测试 Feature Flag 组合 | 可能引入编译错误 |
| Feature Flag 名称与模块名冲突 | 容易混淆 |
| 在 shared/ 中使用 Feature Flag | shared 是基础能力，不应有条件编译 |

---

## 8. Feature Flag 与架构层的关系

### 8.1 各层允许的 Feature Flag

| 层 | 允许的 Feature Flag | 说明 |
|----|-------------------|------|
| App | 全部 | 组装层，通过 PluginGroup 决定注册哪些插件 |
| Core | **禁止** | 绝对不允许任何 cfg(feature)，包括模块声明级 |
| Shared | 禁止 | 基础能力不应有条件编译 |
| Infrastructure | 全部 | 技术实现层，子系统按需启用 |
| Content | 禁止 | 内容加载不应有条件编译 |
| Modding | `modding` | MOD 支持整体受 Flag 控制 |
| Debug | `debug_ui` | 调试工具整体受 Flag 控制 |

### 8.2 Feature Flag 的生命周期

> **优化来源**：`docs/其他/51.md` — "版本管理与迭代规范"、"废弃 Flag 的流程"。

```
1. 设计阶段
   → 确定哪些子系统需要 Feature Flag
   → 定义 Flag 名称、作用范围、文档（含编译成本）
   → 检查互斥性，添加 compile_error! 防护
    ↓
2. 实现阶段
   → 在 Cargo.toml 中定义 Flag + 完整文档
   → 使用 PluginGroup 组织条件插件
   → Core 层零 cfg，Infra 层按需启用
   → 确保所有引用处都有 cfg 保护
    ↓
3. 测试阶段
   → CI 测试所有 Flag 组合（含无 Flag 状态）
   → 验证累加性：features + [debug_ui] 仍编译通过
   → 确保禁用 Flag 时编译通过
   → 确保启用 Flag 时功能正常
    ↓
4. 发布阶段
   → 根据目标平台选择 Flag 组合
   → 发布构建使用最小 Flag 集
    ↓
5. 废弃阶段（新增）
   → 标记 Flag 为 deprecated
   → 一个版本后移除代码
   → 更新文档和 CI 矩阵
```

### 8.3 Feature Flag 命名规范

> **优化来源**：`docs/其他/51.md` — "Flag 命名的版本规范"。

- 使用 snake_case：`debug_ui`、`replay`、`modding`
- 避免与模块名冲突
- 开发类 Flag 使用描述性名称：`profiler`、`cheat`
- 功能类 Flag 使用功能名称：`replay`、`network`

---

## 附录：Feature Flag 快速参考

### 常用构建命令

```bash
# 开发（推荐日常使用）
cargo run --features dev

# 发布（最小构建）
cargo build --release

# 无 Flag 测试
cargo test --no-default-features

# 测试特定 Flag
cargo test --features replay

# 测试所有组合
cargo test --all-features

# 验证累加性
cargo test --features "replay,debug_ui"
```

### 添加新 Feature Flag 的步骤

1. 在 `Cargo.toml` 的 `[features]` 中添加定义（含完整文档：作用、编译成本、使用者）
2. 检查互斥性，必要时添加 `compile_error!`
3. 使用 PluginGroup 组织条件插件（不要在 App 层写 cfg）
4. Core 层**零 cfg**，Infra 层按需启用
5. 确保**所有引用处**都有 cfg 保护
6. 更新本文档的 Flag 列表
7. 在 CI 中添加测试组合
8. 验证累加性：`cargo test --all-features` 通过
