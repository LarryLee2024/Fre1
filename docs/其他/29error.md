这个问题比日志更重要。

对于你的项目（长期维护 SRPG + Rust + Bevy + AI协作），我的答案是：

> **要统一错误体系，但不要搞一个全局万能 AppError。**
>
> 日志可以薄封装，错误必须有架构设计。

---

# 很多人第一版会这样

```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("io error")]
    Io(#[from] std::io::Error),

    #[error("json error")]
    Json(#[from] serde_json::Error),

    #[error("asset error")]
    Asset(String),

    #[error("skill error")]
    Skill(String),

    #[error("buff error")]
    Buff(String),

    #[error("battle error")]
    Battle(String),
}
```

然后全项目：

```rust
Result<T, AppError>
```

---

看起来统一。

实际上后面会变成：

```text
AppError
400+
variant
```

最后没人敢改。

---

# DDD项目更推荐

每个领域拥有自己的错误。

例如：

```rust
pub enum SkillError
```

---

```rust
pub enum BuffError
```

---

```rust
pub enum BattleError
```

---

```rust
pub enum QuestError
```

---

例如：

```rust
#[derive(Debug, Error)]
pub enum SkillError {
    #[error("skill not found")]
    NotFound,

    #[error("insufficient mp")]
    InsufficientMp,

    #[error("invalid target")]
    InvalidTarget,
}
```

---

# 为什么这样更适合AI

AI看到：

```rust
Result<(), SkillError>
```

立即知道：

```text
这是技能领域
```

---

而看到：

```rust
Result<(), AppError>
```

AI需要全局搜索。

---

对于大型项目：

```text
局部错误
远比
全局错误
更容易维护
```

---

# 那 infrastructure 呢？

这里确实应该有一层。

例如：

```text
shared/
└── error/
```

或者未来：

```text
infrastructure/
└── error/
```

---

但这里不要放：

```rust
AppError
```

---

而是放：

```rust
GameResult<T>
```

例如：

```rust
pub type GameResult<T> = Result<T, GameError>;
```

---

以及：

```rust
pub trait IntoGameError
```

---

以及：

```rust
pub trait LogError
```

---

这些基础设施。

---

# 我推荐的结构

```text
shared/
└── error/
    ├── mod.rs
    ├── game_error.rs
    ├── result.rs
    └── extensions.rs
```

---

其中：

```rust
pub enum GameError {
    Infrastructure(InfrastructureError),

    Domain(String),
}
```

---

注意：

这里不要放：

```rust
SkillNotFound
BuffExpired
```

这些。

---

这些属于领域。

---

# 最适合你的做法

## 第一层：领域错误

```text
skill/error.rs
```

---

```rust
pub enum SkillError
```

---

```text
buff/error.rs
```

---

```rust
pub enum BuffError
```

---

```text
battle/error.rs
```

---

```rust
pub enum BattleError
```

---

# 第二层：基础设施错误

```rust
pub enum InfrastructureError {
    Io,
    Asset,
    Serialization,
    SaveLoad,
}
```

---

# 第三层：边界转换

例如：

```rust
impl From<SkillError> for GameError
```

---

```rust
impl From<BuffError> for GameError
```

---

```rust
impl From<BattleError> for GameError
```

---

# 但对于你的项目

我甚至建议再激进一点。

## 核心领域

直接：

```rust
Result<T, SkillError>
```

---

```rust
Result<T, BattleError>
```

---

```rust
Result<T, BuffError>
```

---

不要急着统一。

---

因为你现在项目还没到：

```text
联网
数据库
云存档
多线程任务调度
```

那个复杂度。

---

# AI协作最佳实践

我会在你的 `coding-conventions.md` 里写：

```text
禁止：

Result<T, anyhow::Error>

Result<T, Box<dyn Error>>

Result<T, AppError>

---

允许：

Result<T, SkillError>

Result<T, BuffError>

Result<T, BattleError>

Result<T, SaveError>

Result<T, AssetError>
```

---

这样 AI 生成代码时：

```rust
fn cast_skill(...)
    -> Result<(), SkillError>
```

会天然保持领域边界。

---

# 如果是我给你的项目定最终规则

```text
shared/
├── logging/
├── error/
└── ids/
```

---

其中：

```text
logging
```

负责统一日志入口。

---

```text
error
```

负责：

GameResult
Error Trait
错误转换工具

````

---

而：

```text
SkillError
BattleError
BuffError
QuestError
SaveError
````

全部留在各自领域内部。

这是 Rust + DDD + AI协作项目里最容易长期维护的一种结构，也避免了后期出现那个臭名昭著的 1000 行 `AppError` 文件。

有，而且我认为这些比「要不要封装日志」「要不要统一错误」更重要，因为它们决定你项目 2 年后是否还能持续开发。

我只补充前面没说过的内容。

---

# 1. 不要让日志和错误成为领域模型的一部分

很多人后面会写：

```rust
pub struct DamageResult {
    damage: i32,
    logs: Vec<String>,
}
```

或者：

```rust
pub struct SkillResult {
    damage: i32,
    error_message: String,
}
```

这是架构污染。

---

领域层应该只产生：

```rust
UnitDamaged
SkillCastFailed
BuffApplied
```

这种事实。

---

至于：

```text
写日志
显示UI
弹提示
写回放
写存档
```

全部属于后续消费者。

---

正确：

```text
SkillDomain
    ↓
Domain Event
    ↓
├─ Logger
├─ UI
├─ Replay
├─ Analytics
└─ Achievement
```

---

# 2. 建立 Error Code 体系

大部分项目后期调试最痛苦的是：

```text
InvalidTarget

到底是哪种InvalidTarget？
```

---

推荐：

```rust
pub enum SkillError {
    S001_TargetNotFound,
    S002_InvalidTarget,
    S003_NotEnoughMp,
}
```

或者：

```rust
pub const S001: &str = "TargetNotFound";
```

---

未来日志：

```text
[S003] NotEnoughMp
```

AI、测试、回放都更容易定位。

---

# 3. 错误要区分“程序错误”和“规则失败”

这是很多Rust项目最大的坑。

例如：

```text
MP不足
```

不是错误。

这是规则结果。

---

很多人：

```rust
return Err(SkillError::NotEnoughMp);
```

---

实际上：

```rust
CastSkillResult::Failed(NotEnoughMp)
```

更合理。

---

真正的 Error 应该是：

```text
技能配置不存在
资源损坏
Entity丢失
状态机非法
```

这种理论上不该发生的情况。

---

你的项目以后建议：

```text
RuleFailure
和
Error
分离
```

---

# 4. 日志不要直接出现 Entity

这是 Bevy 项目常见问题。

例如：

```rust
info!(
    attacker=?entity,
);
```

---

日志：

```text
attacker=Entity(823v4)
```

半年后没人认识。

---

推荐建立：

```rust
UnitId
SkillId
BuffId
```

---

日志：

```text
attacker=knight_001
target=goblin_archer
skill=fireball
```

---

否则日志几乎不可读。

---

# 5. 为 AI 建立“允许记录”的事件白名单

不要靠 Agent 自由发挥。

建立：

```text
docs/conventions/domain_events.md
```

---

例如：

```text
BattleStarted
BattleEnded

TurnStarted
TurnEnded

UnitMoved
UnitAttacked
UnitDamaged
UnitDied

BuffApplied
BuffRemoved

SkillCastStarted
SkillCastFinished

QuestAccepted
QuestCompleted
```

---

以后：

```text
新增领域事件
↓
必须进入白名单
↓
才能写日志
```

---

否则几年后：

```text
287种事件
```

没人知道哪些重要。

---

# 6. 日志与回放统一来源

很多项目会出现：

```text
Logger
Replay
```

两套系统。

---

推荐：

```text
Domain Event
    ↓
Event Journal
```

---

然后：

```text
Journal
├─ Logger
├─ Replay
├─ Analytics
└─ Debug
```

---

这样未来做：

```text
战斗回放
录像
BUG复现
```

几乎免费。

---

# 7. Error 和 Log 都要支持 Context

未来你最常见的问题：

```text
为什么这里失败？
```

---

不是错误本身不够。

而是上下文丢了。

---

推荐：

```rust
SkillError::InvalidTarget {
    caster: UnitId,
    target: UnitId,
    skill: SkillId,
}
```

---

而不是：

```rust
SkillError::InvalidTarget
```

---

AI 排查效率会提升非常明显。

---

# 8. 建立“不允许 panic”规则

对于 SRPG 项目。

我建议：

```text
业务层禁止 panic
```

---

禁止：

```rust
unwrap()
expect()
panic!()
```

出现在：

```text
battle
skill
buff
quest
turn
```

领域代码。

---

只允许出现在：

```text
tests
tools
editor
prototype
```

---

这个规则比日志规范价值还大。

---

# 9. 给 AI 一份 Failure Taxonomy（失败分类）

这个很少有人做。

例如：

```text
RuleFailure
├─ NotEnoughMp
├─ InvalidTarget
├─ Cooldown

DomainError
├─ SkillMissing
├─ BuffMissing

InfrastructureError
├─ AssetLoadFailed
├─ SaveFailed
├─ IoFailed

Bug
├─ IllegalState
├─ EntityMissing
```

---

以后 AI 生成代码时：

先判断：

```text
这是规则失败？
还是真正错误？
```

再决定返回什么。

代码质量会明显提高。

---

# 10. 未来比 logging 更值得抽象的是 Audit Trail

日志是给开发者看的。

Audit Trail（审计轨迹）是给系统看的。

例如：

```text
Turn 5

Knight
  Cast Fireball
  Damage Goblin 43

Goblin
  Died

Buff Poison
  Triggered
```

---

这是结构化记录。

不是字符串日志。

---

未来：

```text
回放
录像
调试
自动测试
存档验证
```

全部依赖它。

---

如果我是你，接下来会优先建立这几个基础设施，而不是继续研究日志 API：

```text
shared/
├── ids/
├── error/
├── logging/
├── events/
└── audit/
```

其中真正长期价值最高的其实是：

```text
ids
events
audit
```

因为它们会直接影响：

```text
技能系统
Buff系统
回放系统
自动测试
AI协作
Bug复现
```

而日志和错误更多是附着在这些体系之上的。
