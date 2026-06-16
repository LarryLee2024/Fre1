---
id: 01-architecture.events-audit-design
title: Events Audit Design
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
  - design
---

# 领域事件 + 审计系统设计 — 解耦与调试基石

Version: 1.0
Status: Proposed

Source: `docs/其他/31遗漏.md`（高优先级第 2 项）

本文档定义 SRPG 项目的领域事件系统和审计系统的架构设计。领域事件实现模块间解耦，审计系统提供确定性回放、调试追踪和测试验证的基础设施。

交叉引用：
- `docs/02-domain/shared_layer_rules.md` — Shared Event 领域定义、AuditEvent 结构、EventWhitelist
- `docs/02-domain/ecs_communication_rules.md` — Message 通信机制、EventBus 分发
- `docs/02-domain/replay_rules.md` — 回放系统如何消费审计事件
- `content-pipeline.md` — 配置加载事件

---

## 概述

本设计包含两个紧密关联的子系统：

**Part A — 领域事件（Domain Event）**：标准化的跨模块通信载体，定义"发生了什么"，由生产者广播、消费者响应。

**Part B — 审计系统（Audit System）**：结构化的事件记录基础设施，定义"记录什么、怎么记录"，为回放、调试和测试提供数据源。

**关系**：领域事件是审计系统的数据源，审计系统是领域事件的持久化消费方。

---

# Part A：领域事件系统

## 设计原则

### 原则 1：事件只描述事实 🟥

🟥 领域事件是已经发生的事实的记录，不是命令或请求（宪法 2.2.6）。`SkillCasted` 表示"技能已释放"，不是"请释放技能"。

### 原则 2：事件跨模块、函数调用同模块 🟥

🟥 需要跨 Feature 广播的通信使用领域事件（Message），同一模块内的逻辑直接函数调用（宪法 2.2.5）。不滥用事件系统模拟函数调用。

### 原则 3：事件携带完整上下文 🟥

🟥 事件必须携带接收方处理所需的全部信息，禁止接收方反向查询发送方获取缺失数据（宪法 13.9.3 延伸）。

### 原则 4：事件不含处理逻辑 🟥

🟥 事件类型定义在 `shared/events/` 中，只携带数据字段，不包含业务方法（宪法 2.1.2 数据与行为分离）。

---

## 架构

> **优化来源**：`docs/其他/50.md` — "DomainEvent 大枚举与 Bevy 消息系统的架构割裂"、"独立 Struct + Auditable Trait"。

### 事件模型：独立 Struct + Auditable Trait

🟥 **废除 DomainEvent 大枚举**。每个事件必须是**独立的 Struct**，并独立注册为 Bevy Message。这避免了"新增一个事件要改十个文件"的 OCP 灾难，也避免了大枚举的缓存惩罚。

```rust
// ✅ 正确：每个事件是独立的 Struct
#[derive(Debug, Clone, Message)]
pub struct SkillCasted {
    pub caster: UnitId,
    pub skill_id: SkillId,
    pub targets: Vec<UnitId>,
}

#[derive(Debug, Clone, Message)]
pub struct DamageDealt {
    pub source: UnitId,
    pub target: UnitId,
    pub amount: i32,
    pub is_critical: bool,
    pub skill_id: Option<SkillId>,
}

#[derive(Debug, Clone, Message)]
pub struct HealApplied {
    pub source: UnitId,
    pub target: UnitId,
    pub amount: i32,
    pub skill_id: Option<SkillId>,
}

// ... 其他事件同理，每个独立注册
```

#### Auditable Trait

所有可审计的事件实现 `Auditable` Trait，审计系统通过 Trait 统一收集，无需硬编码 Reader：

```rust
/// 可审计事件 Trait — 所有需要被审计系统记录的事件必须实现
pub trait Auditable: Message + Clone + Send + Sync + 'static {
    /// 转换为可序列化的审计负载
    fn to_audit_payload(&self) -> AuditEventPayload;
    
    /// 事件类型名称（用于白名单查询）
    fn event_type_name() -> &'static str;
}

// 自动化收集：审计系统只需监听 Auditable 事件
// 无需为每个事件类型写单独的 MessageReader
```

**自动化注册**：编写 Bevy Plugin 或宏，在注册 Message 时自动将其接入审计系统，彻底消除 `audit_recording_system` 中的硬编码 Reader。

#### EventOrd：同 Tick 内确定性排序

> **优化来源**：`docs/其他/50.md` — "事件时间粒度补充"。

同 Tick 内可能有多个事件，需确定性排序：

```rust
/// 事件排序键 — 同一 Tick 内的事件按此值排序
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventOrd(pub u64);

// 规则：
// - 同一 Tick 内的事件按 EventOrd 升序排列
// - EventOrd 由发送方显式指定，不由系统自动分配
// - 不使用时钟时间戳排序（时钟不确定）
```

### 事件结构规范

每个领域事件必须包含以下字段（或语义等价物）：

| 字段 | 类型 | 说明 | 强制等级 |
|------|------|------|----------|
| source / caster / user | `UnitId` | 事件发起者 | 🟥 必须（适用时） |
| target / targets | `UnitId` / `Vec<UnitId>` | 事件目标 | 🟥 必须（适用时） |
| event-specific payload | 各类型特有 | 事件具体数据 | 🟥 必须 |

可选字段：

| 字段 | 类型 | 说明 |
|------|------|------|
| skill_id | `Option<SkillId>` | 关联技能 |
| amount | `i32` | 数值（伤害/治疗） |
| timestamp | `u64` | 事件时间戳（审计系统补充） |

### 事件注册表 🟥

🟥 所有领域事件类型必须在统一白名单文档中登记（宪法 2.2.7 领域事件白名单管理）。以下为宪法 13.10.2 定义的核心事件清单：

| 事件类型 | 发送方模块 | 接收方模块 |
|----------|-----------|-----------|
| `SkillCasted` | core/skill | core/battle, core/buff, ui/combat_log |
| `DamageDealt` | core/battle | ui/combat_vfx, ui/combat_log, shared/audit |
| `HealApplied` | core/battle | ui/combat_log, shared/audit |
| `BuffApplied` | core/buff | ui/combat_log, shared/audit |
| `BuffRemoved` | core/buff | ui/combat_log, shared/audit |
| `CharacterDied` | core/battle | ui/combat_log, core/turn, shared/audit |
| `TurnStarted` | core/turn | core/battle, ui/turn_panel, shared/audit |
| `TurnEnded` | core/turn | core/battle, ui/turn_panel, shared/audit |
| `UnitMoved` | core/battle | ui/combat_vfx, shared/audit |
| `ItemEquipped` | core/equipment | ui/combat_log, shared/audit |
| `ItemUnequipped` | core/equipment | ui/combat_log, shared/audit |
| `ItemUsed` | core/inventory | ui/combat_log, shared/audit |
| `BattleInitialized` | core/battle | shared/audit, ui |
| `BattleEnded` | core/battle | shared/audit, ui, core/turn |

### 事件路由（基于 Bevy Message 系统 + Observer 模式）

事件通过 Bevy 的 Message 系统广播，遵循 `docs/02-domain/ecs_communication_rules.md` 中定义的 Message 发送管线：

```
生产者 System
    ↓  MessageWriter<T>.write(event)
EventBus 缓冲
    ↓  按类型分发
消费者 System 1（UI 模块）
消费者 System 2（审计模块）
消费者 System 3（其他 Core 模块）
```

关键约束（参见 `docs/02-domain/ecs_communication_rules.md`）：
- 每个事件类型在 App 中只注册一次（`add_message::<T>()`）
- 消费者通过 `MessageReader<T>` 遍历读取
- 同一帧内发送的消息在该帧 `Update` 阶段末尾统一消费

> **优化来源**：`docs/其他/34.md` — S 级第 3 项「Domain Event 领域事件体系」深度点评

#### Observer/Trigger 模式（Bevy 0.15+ 推荐）

> **优化来源**: `docs/其他/74借鉴.md` §8 — Godot Signal → Bevy Event/Observer 对应关系

**跨引擎类比**：Bevy 的 Event/Observer 机制等价于 Godot 的 Signal（信号）系统。Godot 中模块间通信推荐使用 `signal` 而非直接函数调用，Bevy 中同理——跨模块通信必须使用 Event（领域事件），不直接模块互调。

```
Godot Signal          ↔  Bevy Event / Observer / Message
signal.emit()         ↔  MessageWriter<T>.write(event)
signal.connect()      ↔  MessageReader<T> / commands.entity().observe()
节点间直接调用（禁止）  ↔  模块间直接函数调用（禁止）
```

🟥 **装饰器型子系统（审计、统计、日志）应优先使用 Bevy 0.15+ 的 Observer/Trigger 模式，而非 EventReader 轮询。**

理由：
- 传统 `EventReader<T>` 是全局轮询，所有系统每帧都要检查是否有事件——当同屏 20 个单位时，性能开销线性增长
- `Trigger::<DamageApplied>::watch()` 是**实体级观察者**，事件只触发给相关实体，零全局轮询开销
- 审计系统本质是"装饰器"——不修改事件流，只旁路观察。Observer 模式完美匹配这一语义

```rust
// ✅ 推荐：审计系统作为 Observer 订阅领域事件
commands.entity(unit_entity).observe(
    |trigger: Trigger<UnitDamaged>, mut audit: ResMut<AuditTrail>| {
        audit.record(DomainEvent::DamageDealt { ... });
    }
);

// ⚠️ 备选：传统 EventReader（适用于全局广播场景）
fn audit_from_events(mut reader: MessageReader<DamageApplied>, ...) { ... }
```

选择准则：
| 场景 | 推荐模式 | 原因 |
|------|---------|------|
| 审计/统计/日志（装饰器） | Observer/Trigger | 零轮询、实体级精准订阅 |
| UI 更新（全局广播） | MessageReader | UI 需要感知所有事件 |
| 跨模块核心逻辑 | MessageReader | 需要帧级顺序保证 |

#### 审计作为 Domain Event 的装饰器

🟥 **审计系统是 Domain Event 的装饰器（Decorator），不是独立路径。审计通过 Observer 订阅事件，不创建独立的事件通道。**

> **优化来源**：`docs/其他/34.md` — B 级第 17 项「Audit Trail：可以直接复用 Domain Event」

```
Domain Event（统一事件源）
    ├── UI 消费者（MessageReader）
    ├── 核心逻辑消费者（MessageReader）
    └── 审计装饰器（Observer/Trigger）  ← 旁路观察，不修改事件流
```

### GameplayCue 模式：逻辑→表现分离的基石

> **优化来源**：`docs/其他/74借鉴.md` §4 — UE GameplayCue：很多人忽略，但这是逻辑与表现分离的关键模式

GameplayCue 是 UE GAS 中一个容易被忽视但极其重要的子系统。它解决的核心问题是：**战斗逻辑（伤害计算、Buff 生效）与视觉表现（特效、音效、飘字）的彻底分离**。

#### GameplayCue 的核心思想

```
Effect 执行（纯逻辑）
    ↓ 发出 DomainEvent
DamageApplied Event
    ├── VFX 系统监听 → 播放火焰特效
    ├── Audio 系统监听 → 播放爆炸音效
    └── UI 系统监听 → 显示伤害飘字

三个消费者互不依赖，各自独立响应同一个事件。
```

#### 本项目中的 GameplayCue 映射

| UE GameplayCue | 本项目实现 | 说明 |
|----------------|-----------|------|
| `GameplayCue` | `DomainEvent`（领域事件） | 效果执行后发出的事件，携带完整上下文 |
| `GameplayCueNotify_Static` | `VFX System`（MessageReader） | 监听 DamageApplied → 播放火焰/冰霜特效 |
| `GameplayCueNotify_Burst` | `Audio System`（MessageReader） | 监听 DamageApplied → 播放爆炸/治疗音效 |
| `GameplayCueNotify_BurstCumulative` | `UI System`（MessageReader） | 监听 DamageApplied → 显示伤害飘字 |

#### 完整示例：火球术的 GameplayCue 流程

```
1. 玩家释放火球术
   ↓
2. Effect Pipeline 执行 DamageEffect（纯逻辑，不关心表现）
   ↓
3. Execute 阶段发出 DamageApplied Event {
       source: "mage_01",
       target: "goblin_03",
       amount: 237,
       is_critical: true,
       element: Damage.Fire,
   }
   ↓
4. 三个表现消费者并行响应（互不依赖）：
   VFX System:   读取 element=Fire → 播放火焰爆炸特效
   Audio System:  读取 amount=237 → 播放重击音效
   UI System:     读取 amount=237, is_critical=true → 显示暴击飘字"237！"
```

#### 显式规则：效果执行后必须发出 DomainEvent

> **铁律**：效果执行后必须发出 DomainEvent，表现层通过 DomainEvent 触发，不执行逻辑。

这是 GameplayCue 模式在本项目中的核心约束：

```rust
// ✅ 正确：Effect 执行后发出事件，表现层独立响应
fn execute_damage(ctx: &mut ExecuteContext) -> Option<EffectResult> {
    // 纯逻辑：计算伤害、修改 HP
    target.hp -= final_damage;
    // 发出事件（不关心谁消费）
    ctx.events.send(DamageApplied { source, target, amount: final_damage, ... });
    Some(EffectResult { target_died: target.hp <= 0 })
}

// VFX System 只管特效，不关心伤害计算
fn vfx_damage_handler(mut reader: MessageReader<DamageApplied>) {
    for event in reader.read() {
        spawn_fire_effect(event.target);  // 根据 element 选特效
    }
}

// UI System 只管飘字，不关心特效
fn ui_damage_handler(mut reader: MessageReader<DamageApplied>) {
    for event in reader.read() {
        show_damage_number(event.target, event.amount, event.is_critical);
    }
}
```

#### 没有 GameplayCue 的反面教材

> **警告**：没有 GameplayCue = 伤害函数里写满 `spawn_vfx()`/`play_sfx()`/`show_damage_number()` 面条代码

```rust
// 🟥 反模式：逻辑与表现耦合
fn execute_damage(ctx: &mut ExecuteContext) {
    target.hp -= final_damage;
    // 面条代码：逻辑中夹杂表现
    spawn_vfx("fire_explosion", target.position);      // 🟥 表现逻辑
    play_sfx("heavy_hit", target.position);            // 🟥 表现逻辑
    show_damage_number(target.position, final_damage); // 🟥 表现逻辑
    shake_camera(0.5);                                 // 🟥 表现逻辑
    // 如果要换特效？如果要加回放支持？如果要静音模式？
    // 每个 Effect 都要改一遍 → 灾难
}
```

### 📋 S-Tier 优先级：Domain Event + Replay/确定性随机 🟥

> **优化来源**：`docs/其他/34.md` — S 级总评「这三个做对了，后面加 1000 个技能都只是在地基上盖楼」

🟥 **Domain Event 系统 + Replay/Deterministic Random 被标注为 S-tier，必须在内容扩展之前完成（宪法 2.2.6）。**

优先级排序（来自 34.md S 级 5 项）：
1. **全局强类型 ID 体系**（第 1 周） — 见 `ids_design.md`
2. **Content Registry 统一注册中心**（第 1 周）
3. **Domain Event 领域事件体系**（第 1 周） — 本文档
4. **Replay 架构前置设计**（第 1 周） — 只需守住"所有状态变更走 Command"铁律
5. **Deterministic Random 确定性随机**（第 1 周） — `GameRng` Resource + 禁止 `rand::thread_rng()`

最小可行性架构（MVA）节奏：
- **第一周**：定义核心 Event（TurnStarted, UnitDamaged 等）+ GameRng Resource → 跑通 Demo
- **第一个月**：用 enum Effect 实现 3 个技能 + 基于确定性随机的简单录像回放
- **第三个月**：引入 Data Validator、Namespace 等 B/C 级工具

---

# Part B：审计系统

## 设计原则

### 原则 1：审计不侵入业务 🟥

🟥 审计系统的存在不影响业务逻辑的执行路径。审计记录是旁路观察者，不是业务流程的一部分（宪法 13.10.3）。

### 原则 2：白名单控制 🟥

🟥 不是所有事件都需要审计。`EventWhitelist` 精确控制哪些事件类型被记录，避免不必要的性能开销（宪法 2.2.7 领域事件白名单管理）。

### 原则 3：确定性友好 🟩

🟩 审计记录为回放系统提供事件流，支持"相同初始条件 + 相同事件流 → 相同结果"的确定性保证（宪法 18.4.1）。

### 原则 4：零成本当禁用 🟥

🟥 审计功能通过 feature flag 控制，禁用时编译器完全移除审计代码，零运行时开销。

---

## 架构

### AuditRecord 结构

审计轨迹中单条记录的结构：

```rust
// src/shared/audit/audit_record.rs

/// 单条审计记录。
/// 记录一个领域事件的完整快照，用于调试和测试验证。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    /// 事件序号（单调递增，用于排序）
    pub sequence: u64,
    /// 游戏 tick 编号（逻辑帧号）
    pub tick: u32,
    /// 同一 Tick 内的事件排序键
    pub event_ord: EventOrd,
    /// 领域事件（使用 Auditable Trait 的序列化负载）
    pub event: AuditEventPayload,
    /// 审计元数据
    pub metadata: AuditMetadata,
}
```

> **优化来源**：`docs/其他/50.md` — "state_hash 降级为 Tick 级计算"。

🟥 **禁止在单个事件上附加 state_hash**。计算全局状态哈希极其昂贵，只在 TickEnd 或 TurnEnd 时计算一次：

```rust
/// Tick 级审计快照 — 在 TickEnd 时生成
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickAuditSnapshot {
    /// Tick 编号
    pub tick: u32,
    /// 该 Tick 内所有事件的 sequence 范围
    pub sequence_range: (u64, u64),
    /// 全局状态哈希（只计算一次）
    pub state_hash: u64,
}
```

### AuditMetadata 结构

审计事件的上下文信息：

```rust
// src/shared/audit/audit_metadata.rs

/// 审计事件的上下文元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetadata {
    /// 事件发生的回合数
    pub turn_number: u32,
    /// 当前阶段名称
    pub phase: String,
    /// 事件来源标识
    pub source: String,
}
```

### AuditTrail 资源

审计轨迹是 Bevy Resource，收集所有审计事件：

```rust
// src/shared/audit/audit_trail.rs

/// 审计轨迹 — Bevy Resource。
/// 按时间顺序收集所有被审计的领域事件。
/// 回放系统、调试面板、测试验证器从这里消费数据。
#[derive(Resource)]
pub struct AuditTrail {
    /// 事件序列（按 sequence 排序）
    records: Vec<AuditRecord>,
    /// 下一个序号
    next_sequence: u64,
    /// 当前 tick 编号
    current_tick: u32,
}

impl AuditTrail {
    /// 记录一条审计事件
    pub fn record(&mut self, event: AuditEventPayload, metadata: AuditMetadata) {
        let record = AuditRecord {
            sequence: self.next_sequence,
            tick: self.current_tick,
            event_ord: EventOrd(0),  // 由调用方指定
            event,
            metadata,
        };
        self.records.push(record);
        self.next_sequence += 1;
    }

    /// 推进 tick 编号
    pub fn advance_tick(&mut self) {
        self.current_tick += 1;
    }

    /// 获取所有记录（只读）
    pub fn records(&self) -> &[AuditRecord] {
        &self.records
    }

    /// 获取指定 tick 范围的记录
    pub fn records_in_tick_range(&self, start: u32, end: u32) -> Vec<&AuditRecord> {
        self.records.iter()
            .filter(|r| r.tick >= start && r.tick <= end)
            .collect()
    }

    /// 计算当前状态哈希
    /// TODO: 实现具体的状态哈希计算逻辑，按 LogicalId 排序后遍历关键状态字段
    /// 详见 determinism_rules.md §2.5 状态哈希规范
    pub fn compute_state_hash(&self, world: &World) -> u64 {
        // 实现要点：
        // 1. 遍历所有存活单位，按 LogicalId 排序
        // 2. 哈希每个单位的关键属性（HP、位置等）
        // 3. 哈希活跃 Buff 列表
        // 4. 哈希当前回合号和阶段
        // 详细规范见 determinism_rules.md §2.5
        unimplemented!("状态哈希计算 — 待实现，遵循 determinism_rules.md §2.5")
    }
}
```

### EventWhitelist（事件白名单） 🟥

🟥 所有领域事件必须纳入统一白名单文档管理，新增事件必须先更新白名单（宪法 2.2.7）。
🟥 绝对禁止：为临时副作用随意新增领域事件（宪法 2.2.7）。

> **优化来源**：`docs/其他/50.md` — "EventWhitelist 与 state_hash 的性能雪崩"、"TypeId 白名单 + Tick 级哈希"。

控制哪些事件类型被审计记录：

```rust
// src/shared/audit/event_whitelist.rs

use std::any::TypeId;

/// 事件白名单 — 使用 TypeId 实现零分配 O(1) 查询。
/// 新增事件必须先调用 register() 添加到白名单。
#[derive(Resource)]
pub struct EventWhitelist {
    approved: HashMap<TypeId, &'static str>,
}

impl EventWhitelist {
    /// 创建包含所有核心事件类型的默认白名单
    pub fn default_core() -> Self {
        let mut approved = HashMap::new();
        // 战斗事件
        approved.insert(TypeId::of::<SkillCasted>(), "SkillCasted");
        approved.insert(TypeId::of::<DamageDealt>(), "DamageDealt");
        approved.insert(TypeId::of::<HealApplied>(), "HealApplied");
        approved.insert(TypeId::of::<BuffApplied>(), "BuffApplied");
        approved.insert(TypeId::of::<BuffRemoved>(), "BuffRemoved");
        approved.insert(TypeId::of::<CharacterDied>(), "CharacterDied");
        // 回合事件
        approved.insert(TypeId::of::<TurnStarted>(), "TurnStarted");
        approved.insert(TypeId::of::<TurnEnded>(), "TurnEnded");
        // 移动事件
        approved.insert(TypeId::of::<UnitMoved>(), "UnitMoved");
        // 装备事件
        approved.insert(TypeId::of::<ItemEquipped>(), "ItemEquipped");
        approved.insert(TypeId::of::<ItemUnequipped>(), "ItemUnequipped");
        // 物品事件
        approved.insert(TypeId::of::<ItemUsed>(), "ItemUsed");
        // 战斗流程事件
        approved.insert(TypeId::of::<BattleInitialized>(), "BattleInitialized");
        approved.insert(TypeId::of::<BattleEnded>(), "BattleEnded");
        Self { approved }
    }

    /// 注册新的事件类型（使用 TypeId，零分配）
    pub fn register<T: 'static>(&mut self, name: &'static str) {
        self.approved.insert(TypeId::of::<T>(), name);
    }

    /// 检查事件类型是否被批准（O(1) 查询）
    pub fn is_approved<T: 'static>(&self) -> bool {
        self.approved.contains_key(&TypeId::of::<T>())
    }
}
```

### 审计记录系统

> **优化来源**：`docs/其他/50.md` — "审计记录系统的代码冗余问题"。

基于 `Auditable` Trait 的泛化审计收集，无需为每个事件类型硬编码 Reader：

```rust
// src/shared/audit/audit_recording_system.rs

/// 审计记录系统 — 基于 Auditable Trait 统一收集。
/// 监听所有实现了 Auditable 的事件，检查白名单，记录到 AuditTrail。
pub fn audit_recording_system<T: Auditable>(
    mut audit_trail: ResMut<AuditTrail>,
    whitelist: Res<EventWhitelist>,
    mut reader: MessageReader<T>,
) {
    if !whitelist.is_approved::<T>() {
        return;
    }
    
    for event in reader.read() {
        let metadata = AuditMetadata {
            turn_number: current_turn,
            phase: current_phase.to_string(),
            source: event.event_type_name().to_string(),
        };
        audit_trail.record(
            event.to_audit_payload(),
            metadata,
        );
    }
}

// 注册时使用宏自动为每个 Auditable 事件创建审计 System
macro_rules! register_auditable_system {
    ($app:expr, $event_type:ty) => {
        $app.add_systems(Update, audit_recording_system::<$event_type>);
    };
}
```

### 审计系统性能设计

#### Feature Flag 控制

```toml
# Cargo.toml
[features]
default = ["audit"]
audit = []  # 启用审计系统
# 不启用 audit feature 时，审计代码完全不编译
```

#### 条件编译

```rust
// 审计记录宏 — 当 audit feature 禁用时展开为空
#[cfg(feature = "audit")]
macro_rules! audit_record {
    ($trail:expr, $event:expr, $hash:expr, $meta:expr) => {
        $trail.record($event, $hash, $meta);
    };
}

#[cfg(not(feature = "audit"))]
macro_rules! audit_record {
    ($trail:expr, $event:expr, $hash:expr, $meta:expr) => {
        // 零成本：编译器完全移除
    };
}
```

#### 性能约束

> **优化来源**：`docs/其他/50.md` — "审计记录系统的代码冗余问题"、"反膨胀设计"。

| 场景 | 约束 |
|------|------|
| audit feature 启用 | 审计记录不影响游戏逻辑路径，仅旁路记录 |
| audit feature 禁用 | 零运行时开销，审计代码不编译 |
| 每帧审计事件数 | 预计 < 50 条（正常战斗流程） |
| 状态哈希计算 | Tick 级计算（非每事件），仅 TickEnd/TurnEnd 执行 |
| 内存占用 | 审计轨迹**不持久化在内存中**，分块流式写入文件 |

#### 反膨胀：流式写入

🟥 **审计轨迹不存储在内存中整个战斗期间的数据**。每 1000 条事件分块写入磁盘文件，释放内存压力：

```rust
/// 审计轨迹 — 流式写入模式
#[derive(Resource)]
pub struct AuditTrail {
    /// 当前块缓冲区（最多 1000 条）
    buffer: Vec<AuditRecord>,
    /// 下一个序号
    next_sequence: u64,
    /// 当前 tick 编号
    current_tick: u32,
    /// 写入文件句柄
    writer: BufWriter<File>,
    /// 块大小阈值
    chunk_size: usize,  // 默认 1000
}

impl AuditTrail {
    pub fn record(&mut self, event: AuditEventPayload, metadata: AuditMetadata) {
        let record = AuditRecord { ... };
        self.buffer.push(record);
        
        // 分块写入磁盘
        if self.buffer.len() >= self.chunk_size {
            self.flush_to_disk();
        }
    }
    
    fn flush_to_disk(&mut self) {
        // 序列化并写入文件，释放内存
        let data = serde_json::to_vec(&self.buffer).unwrap();
        self.writer.write_all(&data).unwrap();
        self.buffer.clear();
    }
}
```

#### Feature Flag 零成本

🟥 **必须使用 concrete types + cfg gate**，禁止使用 `Box<dyn Any>` 做动态分发：

```rust
// ✅ 正确：具体类型 + cfg gate
#[cfg(feature = "audit")]
pub struct AuditTrail { /* ... */ }

#[cfg(not(feature = "audit"))]
pub struct AuditTrail;  // 零大小占位符，编译器完全优化掉

// ❌ 错误：Box<dyn Any> 动态分发
pub struct AuditTrail {
    inner: Option<Box<dyn Any>>,  // 运行时开销 + 类型不安全
}
```

---

## 审计消费方

> **优化来源**：`docs/其他/50.md` — "回放系统的因果倒置"、"双轨制日志（Command Log vs Audit Trail）"。

### 双轨制日志架构（Dual-Track Logging）

🟥 **回放系统绝不通过 AuditTrail 驱动逻辑重演**——这是 Event Sourcing 最常见的误区。必须区分两条独立的数据轨道：

```
Track A: Command Stream（输入流）—— 用于确定性回放
    内容：PlayerInput, AiDecision, RngSeed
    用途：确定性回放、帧同步联机、断线重连
    特性：极小、必须严格保序、参与状态机重演
    
Track B: Audit Trail（审计流）—— 用于调试/统计
    内容：DomainEvent（伤害、死亡、Buff 触发）
    用途：开发期 Debug、战后统计面板、成就触发、AI 行为分析
    特性：较大、旁路记录、绝不参与逻辑重演
```

**关键区别**：
- Command（命令）是**输入**：玩家指令 A 在坐标 X 释放技能 Y
- Event（事件）是**结果**：A 对 B 造成了 50 点伤害
- 回放系统重放"命令"让引擎自己跑出相同结果，而非重放"伤害事件"

### 消费方 1：回放系统（消费 Command Stream）

回放系统读取 Command Stream，将指令重新喂给确定性的战斗 FSM：

```
CommandStream.commands()
    ↓  逐步喂给 FSM
战斗 FSM 重新执行（相同种子 + 相同命令 → 相同结果）
    ↓  对比 state_hash
验证确定性
```

详细设计见 `docs/02-domain/replay_rules.md`。

### 消费方 2：调试面板（消费 Audit Trail）

调试面板通过 AuditTrail 展示战斗事件时间线：

```
AuditTrail.records()
    ↓  过滤和格式化
Debug Panel 时间线视图
    ↓  选中事件
显示事件详情
```

### 消费方 3：测试验证器

测试验证器通过 AuditTrail 检查游戏不变量：

```
AuditTrail.records()
    ↓  遍历事件序列
不变量检查
    ↓  检查失败
测试断言
```

### 消费方 4：AI 分析

AI 系统通过 AuditTrail 分析战斗模式：

```
AuditTrail.records()
    ↓  统计分析
伤害分布、Buff 使用频率、移动模式
```

### 消费方 5：战报统计

回放结束后，从 AuditTrail 生成战报统计（不参与逻辑重演）：

```
AuditTrail.records()
    ↓  聚合统计
伤害输出排行、Buff 覆盖率、移动距离统计
```

---

## 目录结构

```
src/shared/audit/
├── mod.rs                    # 模块导出
├── audit_record.rs           # AuditRecord 结构
├── audit_metadata.rs         # AuditMetadata 结构
├── audit_trail.rs            # AuditTrail 资源（流式写入）
├── tick_snapshot.rs          # TickAuditSnapshot（Tick 级哈希）
├── event_whitelist.rs        # EventWhitelist 资源（TypeId 查询）
├── auditable.rs              # Auditable Trait 定义
└── state_hash.rs             # 状态哈希计算

src/shared/events/
├── mod.rs                    # 模块导出
├── battle_events.rs          # SkillCasted, DamageDealt, HealApplied, ...
├── turn_events.rs            # TurnStarted, TurnEnded, ActionPhaseStarted
├── movement_events.rs        # UnitMoved
├── equipment_events.rs       # ItemEquipped, ItemUnequipped
├── item_events.rs            # ItemUsed
├── flow_events.rs            # BattleInitialized, BattleEnded
└── (每个事件是独立的 Struct，实现 Auditable Trait)
```

---

## 允许的模式

### 模式 1：审计记录（通过 Auditable Trait）

```rust
// ✅ 允许：业务代码发送事件，审计系统自动收集
fn some_battle_system(
    mut damage_writer: MessageWriter<DamageDealt>,
    // 不需要 ResMut<AuditTrail> — 审计系统通过 Auditable Trait 自动监听
) {
    damage_writer.write(DamageDealt {
        source: attacker_id,
        target: target_id,
        amount: 42,
        is_critical: false,
        skill_id: None,
    });
}
```

### 模式 2：白名单动态注册

```rust
// ✅ 允许：MOD 或扩展事件注册到白名单（使用 TypeId）
fn register_mod_events(mut whitelist: ResMut<EventWhitelist>) {
    whitelist.register::<ModCustomEvent>("ModCustomEvent");
}
```

### 模式 3：审计旁路检查

```rust
// ✅ 允许：业务代码中只发事件，不直接操作审计
fn some_battle_system(
    mut damage_writer: MessageWriter<DamageApplied>,
    // 不需要 ResMut<AuditTrail> — 审计系统自行监听
) {
    damage_writer.write(DamageApplied { ... });
}
```

---

## 禁止事项

### 🟥 绝对禁止

| 禁止行为 | 原因 | 违反后果 | 宪法条款 |
|----------|------|----------|----------|
| 业务代码直接写入 AuditTrail | 侵入业务逻辑路径 | 审计与业务耦合，无法独立测试 | 13.10.3 |
| 未在白名单注册就记录事件 | 绕过审计控制 | 不可控的审计数据量 | 2.2.7 |
| 在审计事件中包含随机数 | 破坏确定性 | 回放结果不可复现 | 18.4.1 |
| 审计记录修改游戏状态 | 旁路系统不应有写权限 | 非确定性行为 | 13.10.3 |
| 在 audit feature 禁用时仍有审计代码执行 | 性能损失 | 零成本设计被破坏 | 15.0.8 |
| 使用 DomainEvent 大枚举 | OCP 灾难 + 缓存惩罚 | 新增事件改十个文件 | 2.2.7 |
| 审计事件使用裸 Entity 而非 Strong ID | 不可序列化 | 回放系统无法使用 | 1.2.1 |
| 使用 Box<dyn Any> 做审计 feature 分发 | 运行时开销 | 零成本设计被破坏 | 15.0.8 |
| 审计轨迹全量存储在内存中 | 内存膨胀 | 长时间战斗 OOM | 15.0.3 |
| 在单个事件上计算 state_hash | 性能雪崩 | 帧率暴跌 | 15.0.3 |
| 用字符串日志（println!/log::info!）代替 AuditRecord | 无法回放、无法测试验证 | 调试效率低下 | 13.1.1 |

### AuditRecord 是 Replay/测试/统计的统一数据源

> **优化来源**：`docs/其他/74借鉴.md` §22 — Audit ≠ 字符串日志，AuditRecord 是结构化事件记录

74借鉴.md §22 明确强调：**Audit 不是字符串日志**。`println!("造成 50 点伤害")` 不是审计——它无法被程序解析、无法被回放系统消费、无法被测试验证器校验。

AuditRecord 是**结构化事件记录**，是以下三大消费方的统一数据源：

| 消费方 | 使用方式 | 数据需求 |
|--------|---------|---------|
| **Replay 回放系统** | 通过 Command Stream 重放，AuditRecord 用于验证确定性 | 结构化、可序列化、带 Tick/Seq |
| **测试验证器** | 遍历 AuditRecord 检查游戏不变量 | 结构化、可断言、完整上下文 |
| **战报统计** | 聚合 AuditRecord 生成伤害分布、Buff 覆盖率等统计 | 结构化、可聚合、带数值字段 |

```
字符串日志（不可用）：
  log::info!("A 对 B 造成了 50 点伤害");  // 无法解析、无法验证

AuditRecord（可用）：
  AuditRecord {
      event: DamageDealt { source: A, target: B, amount: 50, is_critical: false },
      tick: 42,
      sequence: 1234,
      metadata: AuditMetadata { turn_number: 3, phase: "Action", .. },
  }  // 可序列化、可回放、可验证、可统计

### 🟩 必须遵守

| 必须行为 | 原因 | 宪法条款 |
|----------|------|----------|
| 新增事件类型必须先更新白名单 | 保证审计系统能识别新事件 | 2.2.7 |
| 事件字段携带完整上下文 | 消费方无需反向查询 | 13.9.3 |
| 审计事件使用 Strong ID | 支持序列化和回放 | 1.2.1 |
| 审计系统通过 Bevy Message 监听事件 | 不侵入业务代码 | 13.10.3 |
| Feature flag 控制审计编译 | 零成本当禁用 | 15.0.8 |

---

## AI 修改规则

### 如果新增领域事件类型

允许：
- 创建新的独立事件 Struct（如 `pub struct TrapTriggered { ... }`）
- 为新事件实现 `Auditable` Trait
- 在 `EventWhitelist` 中注册新事件类型（使用 `TypeId`）

禁止：
- 在事件 Struct 中添加业务方法
- 事件字段使用裸 Entity 或 String（应使用 Strong ID）
- 未在白名单注册就使用新事件

优先检查：
- 事件是否真的需要跨模块广播（同模块内应直接函数调用）
- 事件字段是否携带接收方所需的完整上下文
- 是否与现有事件类型语义重复

### 如果修改审计系统

允许：
- 新增审计消费方
- 优化审计记录性能
- 改进状态哈希算法

禁止：
- 审计记录影响业务逻辑路径
- 审计系统修改游戏状态
- 在 audit feature 禁用时引入审计代码

优先检查：
- 修改是否保持零成本当禁用
- 新增消费方是否正确使用 AuditTrail 只读接口
- 状态哈希算法是否覆盖所有影响确定性的状态
