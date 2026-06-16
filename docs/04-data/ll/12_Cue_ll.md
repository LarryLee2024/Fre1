---
id: 04-data.ll.12_Cue
title: "Cue 领域数据架构（铃兰参考）"
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
tags:
  - data-architect
  - cue
  - presentation
  - ll
---

# Data Architecture Proposal — Cue（表现信号总线）

## Domain Ownership

归属领域：**Cue**（表现信号总线）
管辖范围：GameplayCue 统一表现事件、逻辑→表现解耦、VFX/SFX/UI 事件分发
上游依赖：Effect、Tag、Trigger
下游消费：UI、VFX、SFX、Debug

## Problem

业务逻辑（Effect Pipeline）产生的战斗结果需要触发表现层（动画、音效、UI 飘字），但当前实现中部分表现调用散落在 battle 事件处理中，缺乏统一的 Cue 事件总线。需要：
1. 定义 Cue 的 Definition/Instance 四层边界
2. 规范 Cue 事件格式，确保 Replay 兼容
3. 建立 Effect → Cue → 表现层 的标准数据流

## Schema Design

### Definition Layer（静态定义，运行时不可变）

**CueDef**（RON 输入）：
```rust
// src/core/cue/def.rs
pub struct CueDef {
    pub id: String,           // Cue 唯一 ID
    pub name: String,         // 显示名
    pub name_key: Option<String>, // i18n Key
    pub cue_type: CueType,    // 表现类型
    pub tags: Vec<String>,    // 关联 Tag ID
    pub params: CueParams,    // 表现参数
}
```

**CueType 枚举**（8 种表现类型）：
```rust
pub enum CueType {
    Damage,        // 伤害飘字/特效
    Heal,          // 治疗飘字/特效
    BuffApply,     // Buff 施加特效
    BuffRemove,    // Buff 移除特效
    Death,         // 死亡特效
    Shield,        // 护盾特效
    Displacement,  // 位移特效
    Summon,        // 召唤特效
}
```

**CueParams**（表现参数，纯数据）：
```rust
pub struct CueParams {
    pub vfx_prefab: Option<String>,   // VFX 资源路径
    pub sfx_event: Option<String>,    // SFX 事件名
    pub float_text_color: Option<String>, // 飘字颜色
    pub screen_shake: bool,           // 是否震动屏幕
    pub duration_ms: u32,             // 表现持续时间（毫秒）
}
```

**CueRegistry**（Resource，Layer 1）：
```rust
pub struct CueRegistry {
    definitions: HashMap<CueId, CueDefinition>,
}
// 实现: Registry, LoadableRegistry, ValidatableRegistry
```

### Instance Layer（实例状态，每个 Entity 一份）

Cue 本身不持有 Instance —— 它是**无状态事件总线**。表现层通过订阅 CueEvent 获取信息。

**CueEvent**（事件，跨 Feature 广播）：
```rust
// 跨模块通过 Message 广播
pub struct CueEvent {
    pub cue_type: CueType,
    pub source_entity: Entity,
    pub target_entity: Option<Entity>,
    pub value: Option<i32>,           // 数值（伤害/治疗量）
    pub tags: Vec<GameplayTag>,       // 关联标签
    pub timestamp: u32,               // 回合号（Replay 兼容）
}
```

### Runtime Layer（运行时状态，临时计算结果）

Cue 不持有 Runtime 状态 —— 它是纯事件分发器。

### Persistence Layer（存档状态，需要持久化）

Cue 不需要持久化 —— 它是即时事件，Replay 通过 Command Stream 重建。

### RON 配置示例

```ron
// content/cues/cues.ron（已有，需扩展）
(
    cues: [
        (
            id: "cue_damage",
            name: "伤害表现",
            name_key: Some("cue.damage.name"),
            cue_type: Damage,
            tags: [],
            params: (
                vfx_prefab: Some("vfx/damage_pop.ron"),
                sfx_event: Some("sfx/hit"),
                float_text_color: Some("#FF4444"),
                screen_shake: false,
                duration_ms: 500,
            ),
        ),
        (
            id: "cue_heal",
            name: "治疗表现",
            name_key: Some("cue.heal.name"),
            cue_type: Heal,
            tags: [],
            params: (
                vfx_prefab: Some("vfx/heal_pop.ron"),
                sfx_event: Some("sfx/heal"),
                float_text_color: Some("#44FF44"),
                screen_shake: false,
                duration_ms: 500,
            ),
        ),
        (
            id: "cue_death",
            name: "死亡表现",
            name_key: Some("cue.death.name"),
            cue_type: Death,
            tags: [],
            params: (
                vfx_prefab: Some("vfx/death_burst.ron"),
                sfx_event: Some("sfx/death"),
                float_text_color: None,
                screen_shake: true,
                duration_ms: 1000,
            ),
        ),
    ],
)
```

## Dependency Analysis

```
Effect Pipeline
    │
    ├──→ CueEvent（Message 广播）
    │       │
    │       ├──→ UI 层（飘字、Buff 列表刷新）
    │       ├──→ VFX 层（粒子特效）
    │       ├──→ SFX 层（音效播放）
    │       └──→ Debug 层（战斗日志）
    │
    └──→ BattleRecord（审计记录，含 cue_type）
```

**依赖方向**：
- Cue 依赖：Effect（上游触发）、Tag（标签过滤）
- Cue 被依赖：UI、VFX、SFX、Debug（下游消费）
- **禁止**：Cue → Core 业务逻辑（Cue 不反向依赖战斗逻辑）

## Validation Rules

| 规则 | 等级 | 说明 |
|------|------|------|
| VR-CUE-001 | 🟥 绝对禁止 | Cue 不得包含业务逻辑（伤害计算、Buff 结算等） |
| VR-CUE-002 | 🟥 绝对禁止 | Cue 不得直接修改 ECS 组件状态 |
| VR-CUE-003 | 🟥 绝对禁止 | Cue 不得反向依赖 Core/Battle 模块 |
| VR-CUE-004 | 🟩 必须遵守 | CueEvent 必须携带完整上下文（source/target/value/tags） |
| VR-CUE-005 | 🟩 必须遵守 | Cue 只携带纯数据事件，不携带 VFX/SFX 资源引用（通过 CueParams 间接引用） |
| VR-CUE-006 | 🟩 必须遵守 | 新增 CueType 只需注册到 CueRegistry，不修改 Cue 分发代码 |
| VR-CUE-007 | 🟨 优先选择 | Cue 事件应包含回合号 timestamp，确保 Replay 可重建表现序列 |

## Replay Compatibility

- ✅ CueEvent 是纯数据事件，可序列化
- ✅ CueEvent 携带 timestamp（回合号），Replay 可按序重建
- ✅ Cue 不持有状态，不影响游戏逻辑确定性
- ✅ 表现层可以安全地在 Replay 中跳过或重放

## Save Compatibility

- ✅ Cue 不需要持久化（即时事件）
- ✅ 旧版本无 Cue 字段时，表现层使用默认空实现

## Migration Strategy

1. **Phase 1**：定义 CueDef/CueType/CueRegistry，加载 `content/cues/cues.ron`
2. **Phase 2**：在 Effect Pipeline 末端插入 CueEvent 广播
3. **Phase 3**：UI/VFX/SFX 层订阅 CueEvent
4. **Phase 4**：清理散落在 battle 事件中的直接表现调用

## Future Extension

- CueParams 可扩展新的表现类型（屏幕滤镜、摄像机运动等）
- CueType 可新增变体（如 `LevelUp`、`ItemGet`）
- 支持 Cue 链式触发（Cue → Cue，需设置 chain_depth 限制）

## Risks

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| Cue 事件风暴 | 高频战斗中 CueEvent 过多影响性能 | 限制每帧 CueEvent 数量，合并同类表现 |
| 表现层依赖过重 | UI 过度依赖 CueEvent 格式 | 定义 CueEvent 接口契约，版本化 |

## Constitution Check

| 宪法条款 | 检查结果 |
|----------|----------|
| Data Law 001（Definition/Instance 分离） | ✅ CueDef 静态定义，CueEvent 无状态事件 |
| Data Law 002（Rule/Content 分离） | ✅ Cue 规则在代码中，内容在 RON 中 |
| Data Law 009（所有表现必须经过 Cue） | ✅ Cue 是表现层唯一入口 |
| Data Law 010（Replay 优先于便利） | ✅ CueEvent 可序列化，含 timestamp |
| ADR-017（i18n Key 命名） | ✅ name_key 格式 `cue.<permanent_id>.name` |

---

# 代码实现映射

| 概念 | 源码位置 | 说明 |
|------|----------|------|
| CueDef | `src/core/cue/def.rs` | RON 反序列化结构 |
| CueType | `src/core/cue/types.rs` | 8 种表现类型枚举 |
| CueRegistry | `src/core/cue/registry.rs` | Layer 1 Resource |
| CueEvent | `src/core/cue/events.rs` | 跨模块 Message |
| content/cues/ | `content/cues/cues.ron` | RON 配置文件 |
