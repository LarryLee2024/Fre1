# 错误体系领域

Version: 1.0
Status: Proposed

错误体系领域管理项目中所有错误、失败和异常的分类、归属、设计和使用规范。

核心原则：
- 领域错误归领域，基础设施错误归基础设施，共享工具归 Shared
- 规则失败不是错误，是正常业务流程
- 全局 AppError 大枚举是反模式

---

# 术语定义

## 领域错误（DomainError）

业务领域内预期内可能发生的异常情况。例如：技能不存在、目标无效、配置缺失。

不是规则失败。不是基础设施错误。不是程序 Bug。

关键属性：
- 定义在各业务模块的 `domain/` 子目录
- 使用 `Result::Err` 返回
- 使用 `thiserror` 派生 `Error` trait
- 携带完整上下文信息（ID、名称、原因）

---

## 规则失败（RuleFailure）

业务规则的正常不满足，属于游戏逻辑的正常分支。例如：法力不足、目标超出范围、冷却未结束。

不是错误。不由 `Result::Err` 返回。

关键属性：
- 使用专门的结果枚举表达
- 是正常业务流程的一部分
- 不触发错误日志
- 不中断游戏流程

---

## 基础设施错误（InfrastructureError）

底层技术能力的异常。例如：文件未找到、序列化失败、网络连接断开。

不是领域错误。不是程序 Bug。不是共享工具。

关键属性：
- 定义在基础设施模块内部（`infrastructure/xxx/xxx_error.rs`）
- 不包含领域语义（不知道 SkillId、UnitId 等领域类型）
- 关注技术层面的失败原因
- 包含以下变体：Persistence（持久化错误）、Asset（资源加载错误）、Network（网络错误）、Config（配置错误）
- 只关注技术失败原因，由调用方在跨层时转换为领域语义

---

## 错误上下文（ErrorContext）

错误发生时的附加上下文信息，用于定位问题根因。不是错误类型。不是日志。

关键属性：
- 每个错误变体必须携带至少一个上下文字段（ID、名称、原因等）
- ErrorContext trait 提供 with_context 方法，在 Result 上附加上下文
- 上下文信息包括：关联的 ID、操作描述、当前状态、失败原因
- 无上下文的错误变体是禁止的
- 跨层错误转换时必须保持或增强上下文信息

---

## 程序 Bug

代码缺陷导致的非法状态或断言失败。例如：状态机非法跳转、数据一致性破坏。

不是领域错误。不是规则失败。

关键属性：
- 使用 `panic!` 或 `unreachable!` 表达
- 仅在核心业务领域外使用（测试、工具、编辑器）
- 🟥 核心业务领域绝对禁止 `unwrap()` / `expect()` / `panic!()`

---

## 共享错误工具（Shared Error Tools）

所有模块共享的错误处理基础设施。例如：`GameResult<T>` 类型别名、错误上下文 trait、日志记录 trait。

不是错误定义。不是错误枚举。

关键属性：
- 定义在 `shared/error/` 目录
- 不包含任何错误变体
- 不包含任何领域类型
- 提供错误处理工具而非错误分类

---

## 错误码

领域前缀加序号的错误唯一标识。例如：`S001`、`B003`、`BF002`。

不是异常消息。不是日志级别。

关键属性：
- 格式为领域前缀 + 3位序号
- 全项目唯一
- 便于日志检索和问题定位

完整错误码前缀映射表：

| 领域 | 前缀 | 起始编号 | 示例 |
|------|------|---------|------|
| Battle | B | 001 | B001, B002 |
| Skill | S | 001 | S001, S002 |
| Buff | BF | 001 | BF001, BF002 |
| Inventory | I | 001 | I001, I002 |
| Equipment | E | 001 | E001, E002 |
| Character | CH | 001 | CH001, CH002 |
| Turn | T | 001 | T001, T002 |
| Quest | Q | 001 | Q001, Q002 |
| Dialogue | D | 001 | D001, D002 |
| AI | AI | 001 | AI001, AI002 |
| Map | M | 001 | M001, M002 |
| Terrain | TR | 001 | TR001, TR002 |
| Save | SAVE | 001 | SAVE001, SAVE002 |
| Asset | ASSET | 001 | ASSET001, ASSET002 |
| Network | NET | 001 | NET001, NET002 |
| Config | CFG | 001 | CFG001, CFG002 |

---

# 领域边界

## 本领域负责

- 错误的分类标准（领域/基础设施/共享）
- 错误的归属规则（哪个错误放在哪个目录）
- 错误的设计规范（上下文、错误码、thiserror 使用）
- 规则失败与程序错误的区分标准
- `GameResult<T>` 和错误转换 trait 的设计
- 业务层 Panic 禁令的范围

## 本领域不负责

- 具体领域错误的定义（由各业务领域自行定义）
- 具体基础设施错误的定义（由各基础设施模块自行定义）
- 日志系统的实现（由 Infrastructure 的 Logging 模块负责）
- 可观测性的具体实现（由 Infrastructure 的 Audit 模块负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 领域错误返回 | Result::Err | 调用方 |
| 基础设施错误 | InfrastructureError | Infrastructure 模块 |
| 错误转换 | map_err | 跨层调用 |
| 规则失败 | 专门结果枚举 | 调用方 |

---

# 生命周期

## 状态列表

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Proposed | 新错误类型提出 | Accepted |
| Accepted | 错误定义通过审查 | Active |
| Active | 错误定义在代码中使用 | Deprecated |
| Deprecated | 错误定义过时 | Removed |

## 状态转换图

```
Proposed → Accepted → Active → Deprecated → Removed
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Proposed | Accepted | 错误归属层正确、携带完整上下文、有错误码 |
| Accepted | Active | 错误定义实现并集成到代码 |
| Active | Deprecated | 业务逻辑变更导致错误不再适用 |
| Deprecated | Removed | 所有引用已替换为新错误 |

---

# 不变量

## 不变量1：领域错误归属领域

任意时刻：

所有领域错误（SkillError、BattleError、BuffError 等）定义在对应业务模块的 `domain/` 子目录中。

违反表现：

`src/shared/error/` 中出现 `SkillError`。`src/infrastructure/` 中出现 `BuffError`。

---

## 不变量2：共享工具不含错误变体

任意时刻：

`shared/error/` 中不定义任何错误变体，只提供 `GameResult<T>`、`ErrorContext`、`LogIfError` 等工具。

违反表现：

`shared/error/` 中出现 `enum AppError` 或 `enum GameError` 包含 `Skill(SkillError)` 变体。

---

## 不变量3：规则失败不用 Result::Err

任意时刻：

业务规则的正常不满足使用专门的结果枚举表达，不使用 `Result::Err` 返回。

违反表现：

法力不足用 `Err(SkillError::InsufficientMp)` 返回。目标超出范围用 `Err(SkillError::OutOfRange)` 返回。

---

## 不变量4：错误必须携带完整上下文

任意时刻：

所有错误变体必须携带足够定位问题的上下文信息（ID、名称、原因等）。

违反表现：

`SkillNotFound` 不带任何上下文。`InvalidTarget` 不带施法者和目标 ID。

---

# 规则

## 规则1：错误分领域定义

允许：
- 每个业务领域在自己的 `domain/` 子目录定义错误枚举
- 基础设施模块在自己的目录定义基础设施错误
- Shared 只提供错误处理工具

禁止：
- 创建全局统一的 AppError 大枚举
- 使用 `anyhow::Error` 或 `Box<dyn Error>` 作为业务层返回类型
- 在 `shared/` 中定义任何错误变体
- 在 `infrastructure/` 中定义任何领域错误

必须：
- 技能相关错误用 `SkillError`（在 `core/skill/domain/`）
- 战斗相关错误用 `BattleError`（在 `core/battle/domain/`）
- Buff 相关错误用 `BuffError`（在 `core/buff/domain/`）

---

## 规则2：规则失败用结果枚举

允许：
- 使用专门的结果枚举表达规则失败
- 结果枚举包含成功和失败两个分支

禁止：
- 把规则失败用 `Result::Err` 返回
- 把法力不足、目标超出范围等正常业务流程当作错误

必须：
- 法力不足 → `SkillCastResult::InsufficientMp { required, actual }`
- 目标超出范围 → `SkillCastResult::OutOfRange { range, distance }`
- 冷却未结束 → `SkillCastResult::CooldownActive { remaining }`

---

## 规则3：错误必须携带完整上下文

允许：
- 使用带命名参数的错误变体
- 使用错误码前缀标注领域和序号

禁止：
- 无上下文的错误变体（如 `SkillNotFound` 不带 ID）
- 仅返回 `"failed"` 的错误信息
- 在核心业务代码中使用 `unwrap()` / `expect()` / `panic!()`

必须：
- 每个错误变体携带至少一个上下文字段
- 错误码格式为领域前缀 + 3位序号
- 错误消息用 `#[error]` 属性标注

---

## 规则4：thiserror 使用规范

允许：
- 使用 `#[error]` 属性标注错误消息
- 使用 `#[from]` 自动转换错误类型
- 使用 Debug 格式化 `{:?}` 显示非 Error 类型字段

禁止：
- 把 Bevy `Entity` 作为 `#[source]` 字段（Entity 不实现 StdError）
- 在错误中省略关键上下文信息

必须：
- Entity 字段使用 `{:?}` Debug 格式化，不用 `#[source]`
- 错误消息中包含错误码前缀

---

## 规则5：跨层错误转换

允许：
- 使用 `map_err` 将底层错误转换为领域错误
- 在调用方进行错误转换，不修改底层错误定义

禁止：
- 在 `shared/error/` 中创建包含领域语义的错误枚举
- 让基础设施错误包含领域类型（如 SkillId）

必须：
- 基础设施层错误只关注技术失败原因
- 调用方将基础设施错误映射为领域语义

---

## 规则6：Entity 字段在各层错误中的处理

允许：
- 领域错误中使用 Entity 字段（如 `UnitNotFound { entity: Entity }`）
- Entity 字段使用 `{:?}` Debug 格式化，不用 `#[source]`

禁止：
- 基础设施错误中包含 Entity 字段（Entity 是 ECS 概念，基础设施层不感知）
- 共享错误工具中使用 Entity 字段（共享层不依赖 ECS）
- Entity 字段作为 `#[source]`（Entity 不实现 StdError）

必须：
- 领域错误中的 Entity 字段使用 Debug 格式化显示
- 错误消息中 Entity 的显示格式为 `{entity:?}`

---

## 规则7：错误上下文 Trait 使用

允许：
- 使用 ErrorContext trait 的 with_context 方法在 Result 上附加上下文
- 使用 LogIfError trait 的 log_if_error 方法记录错误并返回 Option

禁止：
- 不使用 ErrorContext 就传播底层错误（丢失上下文）
- 使用 `let _ = result` 忽略错误

必须：
- 跨层调用时使用 map_err 或 ErrorContext 保持上下文信息
- 错误消息用 `#[error]` 属性标注格式化内容
- 错误码前缀包含在错误消息中

---

# 管线

## 错误处理管线

```
业务逻辑 → 产生错误或结果 → 调用方处理 → 日志记录或用户提示
```

### Step1：业务逻辑产生结果

输入：业务操作请求
处理：执行业务逻辑，可能产生领域错误、规则失败、或成功结果
输出：`Result<T, DomainError>` 或 `XxxResult`
禁止：跳过错误处理直接 panic

### Step2：调用方处理

输入：错误或结果
处理：根据错误类型决定处理策略
输出：处理结果（转换、恢复、传播）
禁止：忽略错误（`let _ = result`）

### Step3：日志记录

输入：需要记录的错误
处理：使用 tracing 结构化日志记录
输出：日志条目
禁止：在核心业务代码中直接 `info!` 输出核心业务事件（走领域事件链路）

### Step4：用户提示

输入：需要展示给用户的错误
处理：UI 层将错误转换为用户可读消息
输出：UI 提示
禁止：UI 层直接处理原始错误码（应通过 ViewModel 映射）

---

# 数据结构

## SkillError（示例）

职责：技能领域的错误分类

结构：
- SkillNotFound：{ skill_id: SkillId } — 技能不存在
- InvalidTarget：{ caster: UnitId, target: UnitId } — 无效目标
- InsufficientResource：{ skill_id: SkillId, cost: i32 } — 资源不足
- CooldownNotExpired：{ skill_id: SkillId, remaining: u32 } — 冷却未结束
- RequirementNotMet：{ skill_id: SkillId, reason: String } — 需求不满足

要求：
- 每个变体必须携带完整上下文
- 错误码使用 S 前缀（S001-S005）
- 使用 `#[error]` 属性标注格式化消息

---

## SkillCastResult（规则失败示例）

职责：技能释放的规则失败分类

结构：
- Success：{ damage: i32 } — 成功
- InsufficientMp：{ required: i32, actual: i32 } — 法力不足
- OutOfRange：{ range: i32, distance: i32 } — 超出范围
- CooldownActive：{ remaining: u32 } — 冷却中

要求：
- 这不是错误，是正常业务流程的分支
- 使用专门的结果枚举而非 `Result::Err`
- 包含足够的信息让 UI 展示具体原因

---

## GameResult<T>（共享工具）

职责：基础设施层统一类型别名

结构：
- Type alias: `Result<T, InfrastructureError>`
- InfrastructureError 包含 SaveError、AssetError、NetworkError 等基础设施错误变体

要求：
- 不包含任何领域错误变体
- 不包含 SkillId、UnitId 等领域类型
- 只用于基础设施层代码的统一错误处理

---

# 禁止事项

禁止：创建全局 AppError 大枚举

原因：全局大枚举会导致所有模块耦合同一错误定义，任何修改都影响所有依赖方，是经典的"万能垃圾桶"反模式。

违反后果：模块间强耦合、修改一个错误影响全项目、错误分类模糊。

---

禁止：在核心业务领域使用 unwrap / expect / panic

原因：核心业务逻辑（战斗、技能、Buff 等）必须优雅处理所有错误，任何 panic 都是程序缺陷。

违反后果：游戏运行时崩溃、存档丢失、用户体验灾难。

---

禁止：把规则失败当作 Result::Err 返回

原因：法力不足、目标超出范围、冷却未结束是游戏逻辑的正常分支，不是程序错误。用 `Err` 返回会导致调用方误判为异常情况。

违反后果：错误处理代码中充斥正常业务逻辑判断，真正的错误被淹没。

---

禁止：基础设施错误包含领域语义

原因：`SaveError` 不应该知道 `SkillId`。基础设施错误只关注技术层面的失败原因。

违反后果：基础设施层耦合领域层，无法独立替换实现。

---

禁止：shared/error 包含错误变体

原因：shared/error 只提供错误处理工具（GameResult、ErrorContext、LogIfError），不定义任何具体错误。定义错误变体会导致 shared 变成垃圾桶。

违反后果：shared 成为新的万能错误垃圾桶。

---

# AI 修改规则

## 如果新增领域错误

允许：
- 在对应业务模块的 `domain/` 子目录新增错误枚举
- 为每个变体携带完整上下文和错误码
- 使用 `thiserror` 派生 `Error` trait

禁止：
- 在 `shared/error/` 中新增错误变体
- 不带上下文信息的错误变体
- 使用 `anyhow::Error` 作为业务层错误类型

优先检查：
- 错误归属的领域是否正确（三问判断法）
- 错误是否真的是错误（不是规则失败）
- 错误码前缀是否与领域匹配

---

## 如果新增基础设施错误

允许：
- 在对应基础设施模块中新增错误枚举
- 使用基础设施错误码前缀（SAVE、ASSET、NET 等）

禁止：
- 在基础设施错误中引用领域类型
- 在 `shared/error/` 中定义基础设施错误变体

优先检查：
- 错误是否真的是基础设施问题（不是领域逻辑问题）
- 错误码前缀是否正确
- 是否与现有基础设施错误重复

---

## 如果遇到跨层错误处理

允许：
- 使用 `map_err` 将底层错误转换为领域错误
- 在调用方进行错误转换
- 使用 `shared/error/` 的 ErrorContext trait 添加上下文

禁止：
- 修改底层错误定义来适配上层需求
- 创建包含领域语义的基础设施错误

优先检查：
- 错误转换是否在调用方进行
- 转换后的错误是否携带足够上下文
- 底层错误定义是否保持纯净

---

## 如果测试失败（错误相关）

排查顺序：
1. 错误归属是否正确（领域/基础设施/共享）
2. 错误是否携带完整上下文
3. 规则失败是否被误用为 Result::Err
4. thiserror 使用是否正确（Entity 字段不用 #[source]）