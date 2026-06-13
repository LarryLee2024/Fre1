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

### 一句话判断标准

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

#### 1. 每个领域独立错误枚举

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

#### 2. 错误码规范

- 格式：领域前缀 + 序号
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

#### 3. 错误必须携带完整上下文

🟥 **绝对禁止**：无上下文的错误变体。

```rust
// ❌ 错误：缺少上下文
SkillNotFound

// ✅ 正确：包含完整上下文
SkillNotFound { skill_id: SkillId }
```

#### 4. 失败分类学

必须严格区分四类失败场景：

| 分类 | 含义 | 实现 | 示例 |
|------|------|------|------|
| **RuleFailure** | 业务规则的正常不满足 | 专门的结果枚举 | 法力不足、目标超出范围 |
| **DomainError** | 领域内预期内的异常 | `Result::Err` | 技能配置不存在、BuffID无效 |
| **InfrastructureError** | 底层能力异常 | `Result::Err` | 资源加载失败、存档IO错误 |
| **Bug** | 非法状态、断言失败 | `panic!` / `unreachable!` | 状态机非法跳转 |

**RuleFailure 不用 `Result::Err`**——规则失败是正常业务流程，不是错误。

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

- 🟥 基础设施错误禁止包含领域语义
- 🟥 `SaveError` 不应该知道 `SkillId`、`UnitId` 等领域类型
- 🟩 基础设施错误只关注技术层面的失败原因

---

## 第三层：共享错误工具（Shared Error Tools）

### 放置位置

```
shared/error/
├── mod.rs           # 公开导出
├── result.rs        # GameResult<T> 类型别名
├── context.rs       # 错误上下文工具
└── extensions.rs    # 错误转换 trait
```

### 设计规范

#### GameResult 类型别名

```rust
// shared/error/result.rs
use infrastructure::persistence::save::SaveError;

/// 基础设施层错误类型别名
pub type GameResult<T> = Result<T, InfrastructureError>;

/// 通用错误类型（用于需要返回多种错误的基础设施层代码）
#[derive(Debug, thiserror::Error)]
pub enum InfrastructureError {
    #[error("持久化错误: {0}")]
    Persistence(#[from] SaveError),
    
    #[error("资源加载错误: {0}")]
    Asset(#[from] AssetError),
    
    #[error("网络错误: {0}")]
    Network(#[from] NetworkError),
    
    #[error("配置错误: {0}")]
    Config(String),
}
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

🟥 **shared/error 禁止**：
- 定义任何领域错误变体（SkillError、BattleError 等）
- 引用任何领域类型（UnitId、SkillId 等）
- 创建超级 `AppError` 大枚举
- 使用 `anyhow::Error` 或 `Box<dyn Error>` 作为业务层返回类型

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

```rust
// 当 Core 调用 Infrastructure 时
fn load_character() -> Result<Character, SkillError> {
    let data = asset_loader.load(path)
        .map_err(|e| SkillError::AssetLoadFailed { 
            path: path.to_string(),
            reason: e.to_string() 
        })?;
    // ...
}
```

跨层错误通过**错误转换**处理，将底层错误映射为领域语义。

---

## 错误架构总图

```
┌────────────────────────────────────────────────┐
│  App 层                                        │
│  - 错误处理兜底                                │
│  - 日志记录                                    │
│  - 用户提示                                    │
├────────────────────────────────────────────────┤
│  UI 层                                         │
│  - 错误提示展示                                │
│  - ViewModel 错误状态映射                       │
├────────────────────────────────────────────────┤
│  Core 层                          DomainError   │
│  skill_error.rs                                 │
│  buff_error.rs                                  │
│  battle_error.rs                                │
│  ...                                            │
├────────────────────────────────────────────────┤
│  Shared 层                       Error Tools   │
│  GameResult<T>                                  │
│  ErrorContext                                   │
│  LogIfError                                    │
├────────────────────────────────────────────────┤
│  Infra 层                    InfrastructureError│
│  save_error.rs                                  │
│  asset_error.rs                                 │
│  network_error.rs                               │
│  ...                                            │
└────────────────────────────────────────────────┘
```

### 错误流向

```
Infrastructure Error → 转换 → Domain Error → UI 展示
                         ↑
                    错误转换在调用方进行
                    不改变底层错误定义
```

---

## thiserror 使用规范

### Entity 字段处理

Bevy `Entity` 不实现 `std::error::Error`，在 thiserror 中不能作为 `source` 字段。

```rust
// ❌ 错误：Entity 作为 source 字段
#[error("...")]
EntityNotFound(#[source] Entity),  // 编译失败！

// ✅ 正确：重命名字段，使用 Debug 格式
#[error("[B001] 单位未找到: {entity:?}")]
UnitNotFound { entity: Entity },  // 注意：不是 source，不需要 #[source]
```

### 错误码前缀映射

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

## 业务层 Panic 禁令

🟥 **绝对禁止**：在以下核心业务领域代码中使用 `unwrap()`、`expect()`、`panic!()`：

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
├── game_result.rs       # GameResult   ← 需迁移到 shared/error/
└── mod.rs               # 模块入口    ← 分拆
```

### 目标状态

```
core/skill/domain/skill_error.rs
core/buff/domain/buff_error.rs
core/battle/domain/battle_error.rs
core/inventory/domain/inventory_error.rs
core/equipment/domain/equipment_error.rs

shared/error/
├── mod.rs
├── result.rs           # GameResult<T>
├── context.rs           # ErrorContext trait
└── extensions.rs        # LogIfError trait

infrastructure/persistence/save/save_error.rs
infrastructure/assets/asset_error.rs
```

### 迁移步骤

1. 在各 `core/xxx/domain/` 目录创建错误文件
2. 迁移错误枚举到对应领域
3. 创建 `shared/error/` 模块，放置 `GameResult<T>` 和错误工具
4. 更新所有 `use` 引用
5. 验证编译通过
6. 删除 `src/core/error/` 临时目录