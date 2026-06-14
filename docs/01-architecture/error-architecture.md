---
id: 01-architecture.error-architecture
title: Error Architecture
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
  - layer
---

# Error Architecture — 错误体系架构

Version: 1.0
Status: Proposed

本文档定义 SRPG 项目的错误体系架构，遵循"Doman Error 属领域，Infrastructure Error 属基础设施，共享工具属 Shared"的三层原则。

来源：`docs/其他/30.md` 错误架构设计、`ADR-004` 错误系统。

---

## 核心原则

### 三层错误模型

```
第一层：领域错误（Domain Error）
    → 放领域内部
    
第二层：基础设施错误（Infrastructure Error）
    → 放基础设施内部
    
第三层：共享错误工具（Shared Error Tools）
    → 放 shared/error
```

### 一句话判断标准 🟩

- **如果删掉 Bevy，这个错误还存在吗？** → Domain Error → 领域内部
- **如果游戏规则不变，能换一种实现方式吗？** → Infrastructure Error → 基础设施内部
- **这是所有模块都会用到的错误工具吗？** → Shared Error Tools → shared/error

---

## 第一层：领域错误（Domain Error）

### 放置位置

每个业务领域在自身模块的 `domain/` 子目录中定义自己的错误枚举。

```
core/skill/domain/skill_error.rs      # SkillError
core/buff/domain/buff_error.rs        # BuffError
core/battle/domain/battle_error.rs    # BattleError
core/inventory/domain/inventory_error.rs  # InventoryError
core/equipment/domain/equipment_error.rs  # EquipmentError
core/quest/domain/quest_error.rs      # QuestError
core/dialogue/domain/dialogue_error.rs    # DialogueError
core/ai/domain/ai_error.rs             # AiError
```

### 设计规范

#### 1. 每个领域独立错误枚举 🟥

🟥 **绝对禁止**：使用全局统一的 `AppError` 大枚举、`anyhow::Error`、`Box<dyn Error>` 作为业务层返回错误类型（宪法 13.9.1）。

```rust
// core/skill/domain/skill_error.rs
#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("[S001] 技能不存在: {skill_id}")]
    SkillNotFound { skill_id: SkillId },
    
    #[error("[S002] 无效目标: 施法者 {caster}, 目标 {target}")]
    InvalidTarget { caster: UnitId, target: UnitId },
    
    #[error("[S003] 资源不足: 技能 {skill_id}, 需要 {cost}")]
    InsufficientResource { skill_id: SkillId, cost: i32 },
    
    #[error("[S004] 冷却未结束: 技能 {skill_id}, 剩余 {remaining} 回合")]
    CooldownNotExpired { skill_id: SkillId, remaining: u32 },
    
    #[error("[S005] 需求不满足: 技能 {skill_id}, 原因: {reason}")]
    RequirementNotMet { skill_id: SkillId, reason: String },
}
```

#### 2. 错误码规范 🟩

- 🟩 格式：领域前缀 + 序号（宪法 13.9.3 推荐）
- 前缀对照表：

| 领域 | 前缀 | 示例 |
|------|------|------|
| Battle | B | B001, B002 |
| Skill | S | S001, S002 |
| Buff | BF | BF001, BF002 |
| Inventory | I | I001, I002 |
| Equipment | E | E001, E002 |
| Quest | Q | Q001, Q002 |
| Dialogue | D | D001, D002 |
| AI | AI | AI001, AI002 |
| Talent | T | T001, T002 |

#### 3. 错误必须携带完整上下文 🟥

🟥 **绝对禁止**：仅返回无上下文的错误变体（如 `InvalidTarget`）（宪法 13.9.3）。

```rust
// ❌ 错误：缺少上下文
SkillNotFound

// ✅ 正确：包含完整上下文
SkillNotFound { skill_id: SkillId }
```

#### 4. 失败分类学 🟥

🟥 必须严格区分四类失败场景，不得混用（宪法 13.9.2）：

| 分类 | 含义 | 实现 | 示例 |
|------|------|------|------|
| **RuleFailure** | 业务规则的正常不满足 | 专门的结果枚举 | 法力不足、目标超出范围 |
| **DomainError** | 领域内预期内的异常 | `Result::Err` | 技能配置不存在、BuffID无效 |
| **InfrastructureError** | 底层能力异常 | `Result::Err` | 资源加载失败、存档IO错误 |
| **Bug** | 非法状态、断言失败 | `panic!` / `unreachable!` | 状态机非法跳转 |

**RuleFailure 不用 `Result::Err`**——规则失败是正常业务流程，不是错误 🟥。

```rust
// ✅ 正确：规则失败用专门的结果枚举
pub enum SkillCastResult {
    Success { damage: i32 },
    InsufficientMp { required: i32, actual: i32 },
    OutOfRange { range: i32, distance: i32 },
    CooldownActive { remaining: u32 },
}

// ❌ 错误：把规则失败当错误
pub enum SkillError {
    InsufficientMp { ... },  // 这不是错误，是规则失败
}
```

---

## 第二层：基础设施错误（Infrastructure Error）

### 放置位置

每个基础设施模块在自身目录中定义自己的错误枚举。

```
infrastructure/persistence/save/save_error.rs     # SaveError
infrastructure/persistence/load/load_error.rs      # LoadError
infrastructure/persistence/migration/migration_error.rs  # MigrationError
infrastructure/assets/asset_error.rs               # AssetError
infrastructure/networking/network_error.rs          # NetworkError
infrastructure/localization/localization_error.rs   # LocalizationError
```

### 设计规范

```rust
// infrastructure/persistence/save/save_error.rs
#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("[SAVE001] 存档文件未找到: {path}")]
    FileNotFound { path: String },
    
    #[error("[SAVE002] 序列化失败: {reason}")]
    SerializeFailed { reason: String },
    
    #[error("[SAVE003] 磁盘空间不足")]
    DiskFull,
    
    #[error("[SAVE004] 版本不兼容: 文件版本 {file_version}, 当前版本 {current_version}")]
    VersionMismatch { file_version: u32, current_version: u32 },
}
```

#### 关键约束

- 🟥 基础设施错误禁止包含领域语义（宪法 13.9.1）
- 🟥 `SaveError` 不应该知道 `SkillId`、`UnitId` 等领域类型
- 🟩 基础设施错误只关注技术层面的失败原因

---

## 第三层：共享错误工具（Shared Error Tools）

> **优化来源**：`docs/其他/49.md` — "shared/error 的循环依赖风险"、"GameResult 应该属于 App 层或 Infra 层"。

### 放置位置

```
shared/error/
├── mod.rs           # 公开导出
├── context.rs       # 错误上下文工具
└── extensions.rs    # 错误转换 trait（ErrorExt, LogIfError）
```

### 设计规范

#### 各层定义自己的 Result 别名

🟥 **GameResult 不应定义在 shared/ 层**。shared 层是最底层的工具层，如果它依赖 InfrastructureError 就会引入循环依赖。每层定义自己的 Result 别名：

```rust
// core/skill/domain/mod.rs
pub type SkillResult<T> = Result<T, SkillError>;

// core/battle/domain/mod.rs
pub type BattleResult<T> = Result<T, BattleError>;

// infrastructure/persistence/save/mod.rs
pub type SaveResult<T> = Result<T, SaveError>;

// infrastructure/mod.rs — 仅 Infra 层定义基础设施 Result
pub type InfraResult<T> = Result<T, InfrastructureError>;
```

#### 错误上下文工具

```rust
// shared/error/context.rs
pub trait ErrorContext<T, E> {
    fn with_context(self, f: impl FnOnce() -> String) -> Result<T, E>;
}

impl<T, E> ErrorContext<T, E> for Result<T, E> 
where E: std::fmt::Debug 
{
    fn with_context(self, f: impl FnOnce() -> String) -> Result<T, E> {
        self.map_err(|e| {
            tracing::error!("{}: {:?}", f(), e);
            e
        })
    }
}
```

#### 错误转换 Trait

```rust
// shared/error/extensions.rs
pub trait LogIfError<T, E> {
    fn log_if_error(self, context: &str) -> Option<T>;
}

impl<T, E: std::fmt::Debug> LogIfError<T, E> for Result<T, E> {
    fn log_if_error(self, context: &str) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(e) => {
                tracing::error!("[{}] {:?}", context, e);
                None
            }
        }
    }
}
```

### 关键约束

🟥 **shared/error 禁止**（宪法 13.9.1）：
- 定义任何领域错误变体（SkillError、BattleError 等）
- 引用任何领域类型（UnitId、SkillId 等）
- 创建超级 `AppError` 大枚举
- 使用 `anyhow::Error` 或 `Box<dyn Error>` 作为业务层返回类型
- 定义 `GameResult<T>` 或 `InfrastructureError`（属于 Infra 层或 App 层）

```rust
// ❌ 绝对禁止
pub enum AppError {
    Skill(SkillError),    // 领域错误不属于 shared
    Battle(BattleError), // 领域错误不属于 shared
    Save(SaveError),     // 基础设施错误不属于 shared
}

// ❌ 绝对禁止
pub type GameResult<T> = Result<T, anyhow::Error>;

// ❌ 绝对禁止
pub type GameResult<T> = Result<T, Box<dyn std::error::Error>>;
```

#### 错误库选型：thiserror vs miette

> **优化来源**：`docs/其他/49.md` — "thiserror vs miette 选型建议"。

| 场景 | 推荐库 | 理由 |
|------|--------|------|
| Core 层领域错误 | `thiserror` | 小巧、编译快、零依赖 |
| Infra 层错误 | `thiserror` | 同上 |
| CLI 工具 | `miette` | 丰富的诊断信息、snippets |
| 内容校验（RON 解析） | `miette` | 友好的错误位置提示 |
| 测试代码 | `anyhow` | 快速原型、不需要结构化错误 |

🟥 **Core 和 Infra 层禁止使用 `miette`**——它会引入大量依赖，拖慢编译速度。

---

## 各层错误使用方式

### Core 层使用领域错误

```rust
// core/skill 中
fn cast_skill(...) -> Result<SkillCastResult, SkillError> {
    // 业务逻辑错误直接用领域错误
}

fn try_cast_skill(...) -> SkillCastResult {
    // 规则失败用专门的结果枚举，不用 Result::Err
}
```

### Infrastructure 层使用基础设施错误

```rust
// infrastructure/persistence/save 中
fn save_game(...) -> Result<SaveData, SaveError> {
    // 技术错误用基础设施错误
}
```

### 跨层错误处理

> **优化来源**：`docs/其他/49.md` — "跨层错误转换的 Source Chain 断链"、"依赖倒置与 Port/Adapter 模式"。

🟥 **Core 层绝对不能直接依赖 Infra 层的类型（包括错误类型）**。使用 Port/Adapter 模式隔离：

```rust
// ❌ 致命反模式：Core 直接调用 Infra 并用 .to_string() 转换（断链！）
fn load_character() -> Result<Character, SkillError> {
    let data = asset_loader.load(path)
        .map_err(|e| SkillError::AssetLoadFailed { 
            path: path.to_string(),
            reason: e.to_string()  // ❌ source chain 断裂，丢失底层根因
        })?;
}

// ✅ 正确做法：Port/Adapter 模式
// Core 层定义端口 trait 和自己的错误类型
// core/skill/domain/skill_error.rs
pub enum SkillError {
    SkillDataNotFound { skill_id: SkillId },
    // ... 不包含 Infra 层的 AssetError
}

// Core 层只接收已加载好的数据
fn cast_skill(skill_data: &SkillData, caster: &Unit) -> Result<SkillCastResult, SkillError> {
    // 如果数据缺失，Core 抛出 SkillError::SkillDataNotFound
    // 资源加载失败由 Infra 层自行处理并上报 InfrastructureError
}

// Infra 层的 AssetLoader 处理加载失败
// infrastructure/assets/asset_loader.rs
fn load_skill_data(path: &str) -> Result<SkillData, AssetError> {
    // 资源加载失败在此处理，不会穿透到 Core 层
}
```

**关键原则**：资源加载的失败应该由 Infra 层的 AssetLoader 或 Baker System 自己处理并上报 `InfrastructureError`，**根本不应该穿透到 Core 层的业务逻辑中**。

---

## 错误架构总图

> **优化来源**：`docs/其他/49.md` — "Bevy System 的错误吞没断层"、"全局 ErrorEvent 与 System 错误处理范式"。

### Bevy System 错误处理铁律

🟥 Bevy 的普通 System（`fn(Query, Res) -> ()`）默认不支持返回 `Result`。如果 System 内部调用业务函数返回了 `Err`，程序员极易顺手写个 `.unwrap()` 或默默吞掉错误，导致"技能没放出来但游戏没报错"的幽灵 Bug。

**铁律：所有 System 内部的业务调用，失败时必须发送事件，严禁吞没！** 🟥

> ⚠️ **注意**：`GameErrorEvent` 中的泛型参数是各领域错误枚举的联合体（通过枚举包装），**不是**全局 `AppError`。每个领域仍使用独立的错误枚举（`SkillError`、`BattleError` 等），`GameErrorEvent` 仅作为 Bevy System 层的事件通道，将领域错误传递给 UI 层统一处理。

```rust
// 定义全局错误事件
#[derive(Event)]
pub struct GameErrorEvent(pub DomainError);  // DomainError 是各领域错误的联合包装，非全局 AppError

// ✅ 正确：System 内部错误上报全局事件
fn process_skill_input(
    skill_query: Query<&SkillDef>,
    mut err_events: EventWriter<GameErrorEvent>,
) {
    match try_cast_skill(...) {
        Ok(result) => { /* 处理 UI 表现 */ }
        Err(e) => {
            // ✅ 上报全局事件，由 UI 层统一弹窗或提示
            err_events.send(GameErrorEvent(e));
        }
    }
}

// ❌ 严禁吞没：
// tracing::error!("{}", e);  // 静默日志 = 幽灵 Bug
// .unwrap();                  // 直接 Panic
```

**ErrorMonitor System**：专门消费 `GameErrorEvent`，转化为 Toast 提示或弹窗：

```rust
fn error_monitor_system(
    mut events: EventReader<GameErrorEvent>,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    for GameErrorEvent(err) in events.read() {
        toast_writer.send(ToastEvent {
            message: format!("{}", err),
            level: ToastLevel::Error,
        });
    }
}
```

### Graceful Degradation（优雅降级） 🟩

- 当 System 遇到 `InfrastructureError`（如贴图加载失败）时，使用"缺失占位符"继续运行
- 🟥 严禁在 Update 阶段的 System 中 Panic（宪法 13.9.4）
- 🟩 强制使用 `tracing::error!` 而非 `println!`，必须携带 span 上下文（宪法 13.1.1）

```
Infrastructure Error → 转换 → GameErrorEvent → ErrorMonitor → UI Toast
                         ↑
                    ErrorMonitor 统一消费
                    不侵入业务逻辑
```

---

## thiserror 使用规范

### Entity 字段处理 🟩

🟥 Bevy `Entity` 不实现 `std::error::Error`，在 thiserror 中不能作为 `source` 字段。
🟥 日志和错误信息中必须输出业务可读的字符串 ID，禁止直接打印 `Entity(xxx)` 原生格式（宪法 1.2.2）。

```rust
// ❌ 错误：Entity 作为 source 字段
#[error("...")]
EntityNotFound(#[source] Entity),  // 编译失败！

// ✅ 正确：重命名字段，使用 Debug 格式
#[error("[B001] 单位未找到: {entity:?}")]
UnitNotFound { entity: Entity },  // 注意：不是 source，不需要 #[source]
```

### 错误码前缀映射 🟩

| 领域 | 前缀 | 起始编号 |
|------|------|---------|
| Battle | B | 001 |
| Skill | S | 001 |
| Buff | BF | 001 |
| Inventory | I | 001 |
| Equipment | E | 001 |
| Quest | Q | 001 |
| Dialogue | D | 001 |
| AI | AI | 001 |
| Save | SAVE | 001 |
| Asset | ASSET | 001 |
| Network | NET | 001 |
| Config | CFG | 001 |

---

## 业务层 Panic 禁令 🟥

🟥 **绝对禁止**：在以下核心业务领域代码中使用 `unwrap()`、`expect()`、`panic!()`（宪法 13.9.4）：

- `core/battle/`
- `core/skill/`
- `core/buff/`
- `core/character/`
- `core/equipment/`
- `core/inventory/`
- `core/quest/`
- `core/turn/`
- `core/ai/`

仅允许使用场景：
- 测试代码
- 工具代码
- 编辑器代码
- 原型验证代码
- `shared/testing/` 中的测试固件

---

## 迁移计划

### 当前状态

```
src/core/error/
├── battle_error.rs      # BattleError  ← 需迁移到 core/battle/domain/
├── buff_error.rs        # BuffError    ← 需迁移到 core/buff/domain/
├── skill_error.rs       # SkillError   ← 需迁移到 core/skill/domain/
├── inventory_error.rs   # InventoryError ← 需迁移到 core/inventory/domain/
├── game_result.rs       # GameResult   ← 需迁移到 infrastructure/（非 shared！）
└── mod.rs               # 模块入口    ← 分拆
```

### 目标状态

```
core/skill/domain/skill_error.rs      # SkillError + SkillResult<T>
core/buff/domain/buff_error.rs        # BuffError + BuffResult<T>
core/battle/domain/battle_error.rs    # BattleError + BattleResult<T>
core/inventory/domain/inventory_error.rs  # InventoryError
core/equipment/domain/equipment_error.rs  # EquipmentError

shared/error/
├── mod.rs
├── context.rs           # ErrorContext trait
└── extensions.rs        # LogIfError trait（仅工具，无 Error 类型）

infrastructure/mod.rs    # InfrastructureError + InfraResult<T>
infrastructure/persistence/save/save_error.rs  # SaveError
infrastructure/assets/asset_error.rs           # AssetError

app/error.rs             # GameErrorEvent（Bevy System 错误上报）
```

### 迁移步骤

1. 在各 `core/xxx/domain/` 目录创建错误文件，定义 `XxxError` + `XxxResult<T>`
2. 将 `GameResult<T>` 和 `InfrastructureError` 从 `shared/error/` 迁移到 `infrastructure/`
3. 在 `shared/error/` 中仅保留 `ErrorContext` trait 和 `LogIfError` trait
4. 定义 `GameErrorEvent` 事件，在 App 层注册
5. 更新所有 `use` 引用
6. 验证编译通过
7. 删除 `src/core/error/` 临时目录
8. 添加 CI 测试：`cargo test --no-default-features`、`cargo test --all-features`