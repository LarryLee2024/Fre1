如果你的 Error 体系已经按我们之前讨论的方案改造了：

```text
DomainError（领域错误）
RuleFailure（规则失败）
InfrastructureError（基础设施错误）
Domain Event（业务事件）
```

那么 50 万行级别项目的 Logging 不应该再是传统的：

```rust
info!("释放技能");
warn!("MP不足");
error!("角色不存在");
```

到处乱打日志。

---

# 核心原则

大型项目里：

```text
Error = 给程序员看的
Event = 给系统看的
Log = 给运维/调试看的
```

三者必须分离。

---

# 第一层：禁止领域代码直接写日志

例如：

```rust
fn cast_skill(...)
{
    info!("开始释放技能");  // ❌

    ...
}
```

这是小项目写法。

50万行以后：

* 日志格式不统一
* AI乱写
* 搜索困难
* 无法统计

最终变成：

```text
[INFO]
开始释放技能

[INFO]
角色开始施法

[INFO]
释放Fireball

[INFO]
施放技能
```

全是垃圾。

---

应该：

```rust
commands.trigger(
    SkillCastStarted {
        caster,
        skill,
    }
);
```

业务层只发事件。

---

# 第二层：统一 LogObserver

建立：

```text
infrastructure/
└── logging/
    ├── observers/
    │   ├── battle_logger.rs
    │   ├── skill_logger.rs
    │   ├── buff_logger.rs
    │   └── turn_logger.rs
    │
    └── LoggingPlugin
```

例如：

```rust
fn log_skill_cast(
    trigger: Trigger<SkillCastStarted>,
)
{
    let e = trigger.event();

    info!(
        caster=?e.caster,
        skill=?e.skill,
        "skill_cast_started"
    );
}
```

领域：

```rust
Ability Domain
```

完全不知道日志存在。

---

# 第三层：全面使用 tracing

不要：

```rust
log
env_logger
```

直接：

```rust
tracing
tracing-subscriber
```

因为以后一定会需要：

```text
结构化日志
过滤
性能分析
span
json输出
崩溃报告
```

---

基础初始化：

```rust
tracing_subscriber::fmt()
    .with_target(true)
    .with_thread_ids(true)
    .with_file(true)
    .with_line_number(true)
    .init();
```

---

# 第四层：日志分类

我建议直接定死。

## TRACE

超细粒度

```rust
trace!("effect_execute");
```

例如：

```text
Targeting
Tag Query
Attribute Query
```

---

## DEBUG

开发调试

```rust
debug!("damage_calculated");
```

例如：

```text
最终伤害
命中率计算
Buff堆叠
```

---

## INFO

正常业务流

```rust
info!("battle_started");
info!("turn_started");
info!("skill_cast");
```

用于 Replay。

---

## WARN

异常但可恢复

```rust
warn!("missing_localization");
warn!("asset_not_found");
```

例如：

```text
配置缺失
存档字段缺失
旧版本兼容
```

---

## ERROR

真正Bug

```rust
error!("entity_not_found");
```

你的项目原则应该是：

```text
ERROR预算 = 0
```

只要出现：

```rust
error!
```

就是要修。

---

# 第五层：日志必须结构化

不要：

```rust
info!("角色{}释放{}", actor, skill);
```

应该：

```rust
info!(
    actor=?actor,
    skill=?skill,
    "skill_cast"
);
```

输出：

```json
{
  "event":"skill_cast",
  "actor":"1001",
  "skill":"fireball"
}
```

以后：

* Kibana
* Grafana
* OpenTelemetry

都能直接接。

---

# 第六层：引入 Span

这是 tracing 最大价值。

例如：

```rust
#[instrument]
fn execute_skill(...)
{
}
```

自动生成：

```text
battle
 └─ turn
     └─ ability
         └─ effect
```

调用链。

以后查 Bug：

```text
Battle#1
 └─ Turn#7
     └─ Fireball
         └─ DamageEffect
```

一眼定位。

---

# 第七层：Replay日志与Debug日志分离

很多SRPG后期踩坑。

不要：

```text
日志 = 回放
```

这是错的。

应该：

```text
Battle Event
↓
Replay System

Battle Event
↓
Logger
```

变成：

```text
Event
├─ Replay
├─ Analytics
├─ Logger
└─ UI
```

四路消费。

---

# 第八层：日志风暴保护

Bevy ECS 特别容易出问题。

例如：

```rust
for entity in query.iter() {
    warn!("missing buff");
}
```

1000单位：

```text
1000条warn
```

直接炸。

必须封装：

```rust
warn_once!
error_once!
```

或者：

```rust
RateLimitedLogger
```

例如：

```rust
warn_every!(
    Duration::from_secs(60),
    "asset_missing"
);
```

---

# 最终推荐架构（50万行版本）

```text
shared/
├── event/
├── error/
└── diagnostics/
    ├── LogCategory
    ├── LogCode
    └── LogContext

infrastructure/
└── logging/
    ├── LoggingPlugin
    ├── observers/
    │   ├── battle_logger
    │   ├── skill_logger
    │   ├── buff_logger
    │   └── turn_logger
    │
    ├── sinks/
    │   ├── console
    │   ├── file
    │   ├── telemetry
    │   └── crash_report
    │
    └── rate_limit/
```

核心规则只有一句：

```text
领域层不写日志
领域层发Event

Logging属于Infrastructure
通过Observer监听Domain Event生成日志

统一使用tracing
统一结构化日志
统一Span链路
```

对于你这个 Bevy SRPG（Character → Ability → Buff → Turn → Battle）架构，这是我认为能够稳定支撑 50 万行代码、多人协作和 AI 大规模生成代码的方案。真正的大项目里，日志系统本质上是一个 Event Consumer，而不是到处 `info!()` 的字符串打印器。
有，而且我认为下面这些细节的重要性其实不低于前面那些。

很多项目到了 20~30 万行以后，真正难受的已经不是「有没有日志」，而是：

```text
日志太多
日志搜不到
日志无法关联
日志无法定位责任模块
日志无法做统计
日志无法被AI利用
```

所以后面这些设计才是真正的“大型项目优化”。

---

# 1. 给日志引入 LogCode

不要依赖文本搜索。

错误：

```rust
warn!("attribute not found");
warn!("attribute missing");
warn!("attribute does not exist");
```

以后根本搜不到。

---

建立统一编码：

```rust
pub enum LogCode {
    // Battle
    BTL001,
    BTL002,

    // Ability
    ABI001,
    ABI002,

    // Effect
    EFF001,

    // Content
    CNT001,
}
```

日志：

```rust
warn!(
    code=?LogCode::ABI001,
    ability=?ability,
    "ability_not_found"
);
```

以后：

```text
ABI001
```

全项目唯一。

---

# 2. 引入 CorrelationId

这是大型战斗调试神器。

例如：

```text
Battle #1
Turn #8
Ability #Fireball
```

全部打出来：

```rust
BattleId
TurnId
ActionId
```

---

日志：

```json
{
  "battle":"battle_1001",
  "turn":"turn_8",
  "action":"action_991",
  "event":"damage_applied"
}
```

以后查：

```text
这一发火球发生了什么？
```

直接过滤：

```text
action_991
```

全链路出来。

---

# 3. 引入 DiagnosticContext

不要每个日志都手写。

例如：

```rust
info!(
    battle=?battle,
    turn=?turn,
    actor=?actor,
);
```

写10000次。

---

统一：

```rust
pub struct DiagnosticContext {
    battle: BattleId,
    turn: TurnId,
    actor: Entity,
}
```

日志：

```rust
ctx.info("skill_cast");
```

自动带：

```json
{
  "battle":"...",
  "turn":"...",
  "actor":"..."
}
```

---

# 4. Content日志单独分类

SRPG最大来源不是代码Bug。

而是：

```text
配置错误
```

例如：

```ron
damage = -999
```

---

建立：

```rust
ContentWarning
ContentError
```

专门处理：

```text
技能配置
Buff配置
地图配置
AI配置
```

不要和程序错误混一起。

---

# 5. 启动阶段日志审计

很多项目忽略。

启动时：

```text
加载了多少Def
多少Ability
多少Buff
多少Tag
```

输出：

```text
Content Audit
------------
AbilityDef: 120
EffectDef: 550
BuffDef: 280
TagDef: 430
```

以后：

```text
怎么少了一个技能？
```

一眼看出。

---

# 6. 自动统计Top N错误

不是打印。

而是聚合。

例如：

```text
Battle结束
```

自动输出：

```text
Top Warnings

CNT001 × 58
ABI003 × 21
TAG004 × 8
```

比刷屏有用一万倍。

---

# 7. 日志生命周期

大型项目必须定义。

例如：

```text
Trace
保留0天

Debug
保留7天

Info
保留30天

Error
永久
```

否则：

```text
logs/
  200GB
```

迟早发生。

---

# 8. 为AI增加可读日志

这个是2026年的新需求。

不要：

```rust
error!("failed");
```

---

应该：

```rust
error!(
    code="ABI001",
    cause="ability_def_not_found",
    suggestion="check content/abilities/*.ron"
);
```

AI Agent分析日志时非常强。

---

# 9. 引入 Metrics 而不是日志

很多人乱打日志统计。

例如：

```rust
info!("ability_used");
info!("ability_used");
info!("ability_used");
```

---

正确：

```rust
counter!(
    "ability_used",
    1,
    "ability" => "fireball"
);
```

日志负责：

```text
发生了什么
```

Metrics负责：

```text
发生了多少次
```

分离。

---

# 10. Battle Timeline

这个特别适合SRPG。

不要：

```text
2026-06-18 10:12:33
```

这种时间。

---

改成：

```text
Battle#1
Turn#3
Action#12
```

例如：

```text
[T3 A12] skill_cast
[T3 A12] damage_apply
[T3 A12] buff_add
```

比真实时间有意义得多。

---

# 11. 领域级 Logger Trait（可选）

如果未来做：

```text
服务器版
客户端版
Replay版
```

可以抽象：

```rust
pub trait DomainLogger {
    fn skill_cast(...);
    fn buff_added(...);
}
```

但只有多人项目才值得。

你目前单人开发可以不做。

---

# 12. Debug Snapshot

这个是很多商业项目才有的。

发生错误时：

```rust
BattleSnapshot
```

自动记录：

```text
当前回合
单位状态
Buff列表
Attribute列表
```

保存：

```text
snapshot_20260619.json
```

以后复现Bug：

```text
加载Snapshot
重放
```

直接定位。

---

# 13. 日志预算（非常重要）

直接写进你的 AI 宪法。

```text
禁止在每帧输出Info

禁止在Query循环输出Warn

禁止在Effect Tick输出Debug

禁止在Update系统输出Trace
```

否则：

```text
60FPS
×1000实体
×100 Buff
```

瞬间百万日志。

---

# 14. 给每个领域分配前缀

例如：

```text
CHR Character
ABI Ability
EFF Effect
BUF Buff
TAG Tag
TGT Targeting
BAT Battle
MAP Map
CNT Content
```

以后：

```text
ABI001
EFF003
BUF012
```

看到编号就知道归属。

---

如果是你的 Bevy SRPG 架构，我会把 **Logging、Metrics、Replay、Snapshot** 看成四个独立基础设施：

```text
Domain Event
    │
    ├── Logger      （人看）
    ├── Metrics     （统计）
    ├── Replay      （回放）
    └── Snapshot    （复现）
```

很多项目最后失败，就是把这四个东西全塞进日志里。日志最终变成万能垃圾桶。真正能支撑 50 万行代码的架构，反而是让日志只负责一件事：

```text
记录发生了什么
```

剩下的统计、回放、复现、监控全部拆出去。这样 5 年后代码量翻十倍，系统依然不会失控。
