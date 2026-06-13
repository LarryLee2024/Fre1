# 领域事件 + 审计系统设计 — 解耦与调试基石

Version: 1.0
Status: Proposed

Source: `docs/其他/31遗漏.md`（高优先级第 2 项）

本文档定义 SRPG 项目的领域事件系统和审计系统的架构设计。领域事件实现模块间解耦，审计系统提供确定性回放、调试追踪和测试验证的基础设施。

交叉引用：
- `shared_layer_rules.md` — Shared Event 领域定义、AuditEvent 结构、EventWhitelist
- `ecs_communication_rules.md` — Message 通信机制、EventBus 分发
- `replay_rules.md` — 回放系统如何消费审计事件
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

### 原则 1：事件只描述事实

领域事件是已经发生的事实的记录，不是命令或请求。`SkillCasted` 表示"技能已释放"，不是"请释放技能"。

### 原则 2：事件跨模块、函数调用同模块

需要跨 Feature 广播的通信使用领域事件（Message），同一模块内的逻辑直接函数调用。不滥用事件系统模拟函数调用。

### 原则 3：事件携带完整上下文

事件必须携带接收方处理所需的全部信息，禁止接收方反向查询发送方获取缺失数据。

### 原则 4：事件不含处理逻辑

事件类型定义在 `shared/events/` 中，只携带数据字段，不包含业务方法。

---

## 架构

### DomainEvent 枚举

统一的领域事件类型目录，所有跨模块事件在此注册：

```rust
// src/shared/events/domain_event.rs

/// 领域事件统一枚举。
/// 所有跨模块的领域事件类型在此注册。
/// 事件只携带数据，不含处理逻辑。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    // ── 战斗事件 ──
    /// 技能已释放
    SkillCasted {
        caster: UnitId,
        skill_id: SkillId,
        targets: Vec<UnitId>,
    },
    /// 伤害已施加
    DamageDealt {
        source: UnitId,
        target: UnitId,
        amount: i32,
        is_critical: bool,
        skill_id: Option<SkillId>,
    },
    /// 治疗已施加
    HealApplied {
        source: UnitId,
        target: UnitId,
        amount: i32,
        skill_id: Option<SkillId>,
    },
    /// Buff 已施加
    BuffApplied {
        source: UnitId,
        target: UnitId,
        buff_id: BuffId,
        stacks: u32,
    },
    /// Buff 已移除
    BuffRemoved {
        target: UnitId,
        buff_id: BuffId,
        reason: BuffRemovalReason,
    },
    /// 角色死亡
    CharacterDied {
        unit: UnitId,
        killer: Option<UnitId>,
        final_hp: i32,
    },

    // ── 回合事件 ──
    /// 回合开始
    TurnStarted {
        turn_number: u32,
        active_unit: UnitId,
    },
    /// 回合结束
    TurnEnded {
        turn_number: u32,
    },
    /// 行动阶段开始
    ActionPhaseStarted {
        unit: UnitId,
    },

    // ── 移动事件 ──
    /// 单位移动
    UnitMoved {
        unit: UnitId,
        from: IVec2,
        to: IVec2,
        path_length: u32,
    },

    // ── 装备事件 ──
    /// 装备穿戴
    ItemEquipped {
        unit: UnitId,
        item_id: ItemId,
        slot: EquipmentSlotType,
    },
    /// 装备脱下
    ItemUnequipped {
        unit: UnitId,
        item_id: ItemId,
        slot: EquipmentSlotType,
    },

    // ── 物品事件 ──
    /// 物品使用
    ItemUsed {
        user: UnitId,
        item_id: ItemId,
        targets: Vec<UnitId>,
    },

    // ── 阶段转换事件 ──
    /// 战斗初始化完成
    BattleInitialized {
        stage_id: StageId,
        units: Vec<UnitId>,
    },
    /// 战斗结束
    BattleEnded {
        winner: FactionId,
        total_turns: u32,
    },
}
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

### 事件注册表

所有领域事件类型在 `docs/architecture.md` 的 Message 注册表中登记：

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

### 事件路由（基于 Bevy Message 系统）

事件通过 Bevy 的 Message 系统广播，遵循 `ecs_communication_rules.md` 中定义的 Message 发送管线：

```
生产者 System
    ↓  MessageWriter<T>.write(event)
EventBus 缓冲
    ↓  按类型分发
消费者 System 1（UI 模块）
消费者 System 2（审计模块）
消费者 System 3（其他 Core 模块）
```

关键约束（参见 `ecs_communication_rules.md`）：
- 每个事件类型在 App 中只注册一次（`add_message::<T>()`）
- 消费者通过 `MessageReader<T>` 遍历读取
- 同一帧内发送的消息在该帧 `Update` 阶段末尾统一消费

---

# Part B：审计系统

## 设计原则

### 原则 1：审计不侵入业务

审计系统的存在不影响业务逻辑的执行路径。审计记录是旁路观察者，不是业务流程的一部分。

### 原则 2：白名单控制

不是所有事件都需要审计。`EventWhitelist` 精确控制哪些事件类型被记录，避免不必要的性能开销。

### 原则 3：确定性友好

审计记录为回放系统提供事件流，支持"相同初始条件 + 相同事件流 → 相同结果"的确定性保证。

### 原则 4：零成本当禁用

审计功能通过 feature flag 控制，禁用时编译器完全移除审计代码，零运行时开销。

---

## 架构

### AuditRecord 结构

审计轨迹中单条记录的结构：

```rust
// src/shared/audit/audit_record.rs

/// 单条审计记录。
/// 记录一个领域事件的完整快照，用于回放、调试和测试验证。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    /// 事件序号（单调递增，用于排序和回放）
    pub sequence: u64,
    /// 游戏 tick 编号（逻辑帧号）
    pub tick: u32,
    /// 领域事件
    pub event: DomainEvent,
    /// 全局状态哈希（用于确定性验证）
    pub state_hash: u64,
    /// 审计元数据
    pub metadata: AuditMetadata,
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
    pub fn record(&mut self, event: DomainEvent, state_hash: u64, metadata: AuditMetadata) {
        let record = AuditRecord {
            sequence: self.next_sequence,
            tick: self.current_tick,
            event,
            state_hash,
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
    pub fn compute_state_hash(&self, world: &World) -> u64 {
        // 对关键游戏状态生成哈希
        // 用于回放时验证确定性
        // 实现细节由 shared/audit 内部提供
        todo!("状态哈希计算实现")
    }
}
```

### EventWhitelist（事件白名单）

控制哪些事件类型被审计记录：

```rust
// src/shared/audit/event_whitelist.rs

/// 事件白名单 — 管理允许记录到审计轨迹的事件类型集合。
/// 新增事件必须先调用 register() 添加到白名单。
#[derive(Resource)]
pub struct EventWhitelist {
    approved: HashSet<String>,
}

impl EventWhitelist {
    /// 创建包含所有核心事件类型的默认白名单
    pub fn default_core() -> Self {
        let mut approved = HashSet::new();
        // 战斗事件
        approved.insert("SkillCasted".to_string());
        approved.insert("DamageDealt".to_string());
        approved.insert("HealApplied".to_string());
        approved.insert("BuffApplied".to_string());
        approved.insert("BuffRemoved".to_string());
        approved.insert("CharacterDied".to_string());
        // 回合事件
        approved.insert("TurnStarted".to_string());
        approved.insert("TurnEnded".to_string());
        // 移动事件
        approved.insert("UnitMoved".to_string());
        // 装备事件
        approved.insert("ItemEquipped".to_string());
        approved.insert("ItemUnequipped".to_string());
        // 物品事件
        approved.insert("ItemUsed".to_string());
        // 战斗流程事件
        approved.insert("BattleInitialized".to_string());
        approved.insert("BattleEnded".to_string());
        Self { approved }
    }

    /// 注册新的事件类型
    pub fn register(&mut self, event_type: &str) {
        self.approved.insert(event_type.to_string());
    }

    /// 检查事件类型是否被批准
    pub fn is_approved(&self, event_type: &str) -> bool {
        self.approved.contains(event_type)
    }

    /// 返回完整清单
    pub fn entries(&self) -> &HashSet<String> {
        &self.approved
    }
}
```

### 审计记录系统

监听领域事件、检查白名单、记录到 AuditTrail 的 System：

```rust
// src/shared/audit/audit_recording_system.rs

/// 审计记录系统。
/// 监听所有领域事件，检查白名单，记录到 AuditTrail。
pub fn audit_recording_system(
    mut audit_trail: ResMut<AuditTrail>,
    whitelist: Res<EventWhitelist>,
    mut skill_casted_reader: MessageReader<SkillCasted>,
    mut damage_dealt_reader: MessageReader<DamageDealt>,
    // ... 其他事件 reader
) {
    // 记录 SkillCasted 事件
    for event in skill_casted_reader.read() {
        if whitelist.is_approved("SkillCasted") {
            let metadata = AuditMetadata {
                turn_number: current_turn,
                phase: current_phase.to_string(),
                source: event.caster.to_string(),
            };
            audit_trail.record(
                DomainEvent::SkillCasted { ... },
                state_hash,
                metadata,
            );
        }
    }

    // ... 其他事件类型的记录逻辑
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

| 场景 | 约束 |
|------|------|
| audit feature 启用 | 审计记录不影响游戏逻辑路径，仅旁路记录 |
| audit feature 禁用 | 零运行时开销，审计代码不编译 |
| 每帧审计事件数 | 预计 < 50 条（正常战斗流程） |
| 状态哈希计算 | 可选，仅在需要确定性验证时执行 |
| 内存占用 | 每条 AuditRecord ≈ 200-500 字节，1000 场战斗 ≈ 2-5 MB |

---

## 审计消费方

### 消费方 1：回放系统

回放系统通过 AuditTrail 读取事件序列，重新执行游戏逻辑：

```
AuditTrail.records()
    ↓  提取事件序列
ReplayData.events
    ↓  逐步重放
游戏逻辑重新执行
```

详细设计见 `replay_rules.md`。

### 消费方 2：调试面板

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

---

## 目录结构

```
src/shared/audit/
├── mod.rs                    # 模块导出
├── audit_record.rs           # AuditRecord 结构
├── audit_metadata.rs         # AuditMetadata 结构
├── audit_trail.rs            # AuditTrail 资源
├── event_whitelist.rs        # EventWhitelist 资源
├── audit_recording_system.rs # 审计记录系统
└── state_hash.rs             # 状态哈希计算

src/shared/events/
├── mod.rs                    # 模块导出
├── domain_event.rs           # DomainEvent 枚举
└── (按领域拆分的事件定义文件)
```

---

## 允许的模式

### 模式 1：审计记录宏

```rust
// ✅ 允许：使用审计宏记录事件
audit_record!(
    audit_trail,
    DomainEvent::DamageDealt {
        source: attacker_id,
        target: target_id,
        amount: 42,
        is_critical: false,
        skill_id: None,
    },
    state_hash,
    AuditMetadata { turn_number: 3, phase: "Action".into(), source: "battle".into() }
);
```

### 模式 2：白名单动态注册

```rust
// ✅ 允许：MOD 或扩展事件注册到白名单
fn register_mod_events(mut whitelist: ResMut<EventWhitelist>) {
    whitelist.register("ModCustomEvent");
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

| 禁止行为 | 原因 | 违反后果 |
|----------|------|----------|
| 业务代码直接写入 AuditTrail | 侵入业务逻辑路径 | 审计与业务耦合，无法独立测试 |
| 未在白名单注册就记录事件 | 绕过审计控制 | 不可控的审计数据量 |
| 在审计事件中包含随机数 | 破坏确定性 | 回放结果不可复现 |
| 审计记录修改游戏状态 | 旁路系统不应有写权限 | 非确定性行为 |
| 在 audit feature 禁用时仍有审计代码执行 | 性能损失 | 零成本设计被破坏 |
| 共享事件中包含处理逻辑 | 事件只携带数据 | 事件与处理逻辑耦合 |
| 审计事件使用裸 Entity 而非 Strong ID | 不可序列化 | 回放系统无法使用 |

### 🟩 必须遵守

| 必须行为 | 原因 |
|----------|------|
| 新增事件类型必须先更新白名单 | 保证审计系统能识别新事件 |
| 事件字段携带完整上下文 | 消费方无需反向查询 |
| 审计事件使用 Strong ID | 支持序列化和回放 |
| 审计系统通过 Bevy Message 监听事件 | 不侵入业务代码 |
| Feature flag 控制审计编译 | 零成本当禁用 |

---

## AI 修改规则

### 如果新增领域事件类型

允许：
- 在 `DomainEvent` 枚举中添加新变体
- 为新事件添加字段（携带完整上下文）
- 在 `EventWhitelist` 中注册新事件类型

禁止：
- 在事件变体中添加业务方法
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
