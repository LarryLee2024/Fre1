---
id: 02-domain.cue.cue-rules
title: Cue Rules
status: draft
owner: domain-designer
created: 2026-06-15
updated: 2026-06-15
tags:
  - domain
  - cue
---

# Cue 表现事件领域

Version: 1.0
Status: Proposed
Source: `docs/其他/77.md` §六（缺失模块4：Cue）、§十（100分方案 GAS 链路）、`docs/01-architecture/README.md` §Logic/Presentation 分离、§领域事件与审计系统

Cue（GameplayCue）是统一的表现事件总线，强制落地 Logic/Presentation 分离原则。业务逻辑永远不直接调用 UI、动画、音效——它发射 Cue 事件，表现层订阅并响应。

**核心原则**：
- 🟩 Cue = 纯数据业务事件，不携带任何动画/UI/特效/音效资源引用
- 🟩 业务逻辑只发射 CueEvent，不关心表现层如何消费
- 🟩 表现层只订阅 CueEvent，零耦合回战斗逻辑
- 🟥 禁止 CueEvent 携带 Handle<Animation>、Handle<AudioSource>、Handle<Scene> 等资源引用
- 🟥 禁止业务逻辑通过 Cue 反向查询表现层状态

**领域定位**：

```
Ability ── 意图（Intent）：我要做什么
  ↓
Targeting ── 目标（Target）：对谁做
  ↓
Effect ── 结果（Result）：产生什么效果
  ↓
Stacking ── 堆叠（Stack）：如何叠加
  ↓
Execution ── 算式（Formula）：怎么计算
  ↓
Modifier ── 修饰（Modify）：属性如何变化
  ↓
Attribute ── 属性（Attribute）：最终数值
  ↓
Tag ── 分类（Classification）：状态标签
  ↓
Cue ── 表现信号（Presentation Signal）：告诉表现层发生了什么 ← 本领域
  ↓
Replay ── 回放（Replay）：记录指令流
```

---

# 术语定义

## Cue（表现事件信号）

业务逻辑向表现层发送的纯数据通知，描述"发生了什么"，不描述"怎么表现"。

不是动画。不是音效。不是 UI 面板。不是 Effect。

关键属性：
- 纯数据结构，只包含 Entity ID、数值、类型等业务数据
- 不引用任何表现资源（Handle、Path、AssetId）
- 由业务逻辑发射（Effect 执行后、Buff 施加后、状态变化后）
- 由表现层系统订阅并响应
- 与 BattleRecord/Replay 集成：Cue 本身不被回放，只有触发 Cue 的上游事件被回放

---

## CueEvent（Cue 事件结构）

每种 Cue 类型是一个独立的 Struct，作为 Bevy Message 注册。

不是 DomainEvent 大枚举。不是 CueHandler。不是 CueRegistry。

关键属性：
- 每个 CueEvent 是独立的 Struct（遵循项目"废除 DomainEvent 大枚举"原则）
- 通过 Bevy Message 系统分发（`EventWriter<T>` / `EventReader<T>`）
- 必须实现 `Send + Sync + 'static`
- 字段只包含原始数据类型（Entity、u32、i32、enum 等）

已定义的 CueEvent 类型：

| 事件 | 触发时机 | 数据字段 | 表现层消费示例 |
|------|----------|----------|----------------|
| DamageCue | 伤害应用后 | target: Entity, amount: u32, is_critical: bool, element: Option<ElementTag> | 播放飘字、受击特效 |
| HealCue | 治疗应用后 | target: Entity, amount: u32 | 播放回复飘字、绿色特效 |
| DeathCue | 实体死亡后 | entity: Entity, killer: Option<Entity> | 播放死亡动画、倒地特效 |
| BuffAppliedCue | Buff 施加后 | target: Entity, buff_id: BuffId, stacks: u32 | 播放 Buff 施加特效 |
| BuffRemovedCue | Buff 移除后 | target: Entity, buff_id: BuffId | 播放 Buff 消散特效 |
| StatusCue | 状态变化后 | entity: Entity, status: StatusType, active: bool | 播放状态图标、持续特效 |
| SkillCastCue | 技能释放后 | caster: Entity, skill_id: SkillId, target_pos: Option<GridPos> | 播放施法动画、弹道特效 |
| MovementCue | 移动完成后 | entity: Entity, from: GridPos, to: GridPos | 播放移动动画、脚步音效 |

---

## CueHandler（Cue 处理器）

表现层实现的 CueEvent 消费逻辑，将 Cue 数据转换为具体的动画/音效/UI 行为。

不是 CueEvent。不是 CueRegistry。不是业务逻辑。

关键属性：
- 每个 CueHandler 对应一种 CueEvent 类型
- 通过 Bevy System 函数实现（`fn handler(event: EventReader<DamageCue>, ...)`）
- Handler 内部可访问资源（Handle<AudioSource>、Handle<Scene> 等）
- Handler 可查询表现层组件（Transform、Sprite、AnimationPlayer 等）
- Handler 禁止修改业务层组件（Attributes、ActiveBuffs、SkillSlots 等）

---

## CueRegistry（Cue 注册表）

全局 Cue 事件类型注册表，管理 CueEvent 类型元数据。

不是 CueHandler。不是 CueEvent。不是 EffectHandlerRegistry。

关键属性：
- 记录所有已注册的 CueEvent 类型（TypeIdentifier）
- 提供按类型查询 Cue 元数据的接口
- 游戏初始化时注册所有 CueEvent 类型
- 新增 Cue 类型只需注册，不修改分发管线
- 主要用于调试面板展示和审计日志过滤

---

# 领域边界

## 本领域负责

- CueEvent 类型定义（DamageCue、DeathCue、HealCue 等）
- CueHandler trait / 接口规范定义
- CueRegistry 管理
- Cue 发射规范（何时发射、数据格式）
- Cue 与 BattleRecord/Replay 的集成规范
- Cue 事件的确定性排序保证

## 本领域不负责

- 实际的动画播放（由 Presentation 层负责）
- 实际的音效播放（由 Presentation 层负责）
- 实际的 UI 渲染（由 Presentation 层负责）
- 业务逻辑的执行（由 Effect/Trigger 等领域负责）
- CueEvent 的具体 Handler 实现（由 Presentation 层各模块负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 效果执行结果 | CueEvent Message（DamageCue/HealCue） | Presentation 层（飘字、特效） |
| 实体死亡 | CueEvent Message（DeathCue） | Presentation 层（死亡动画）、BattleRecord |
| Buff 状态变化 | CueEvent Message（BuffAppliedCue/BuffRemovedCue） | Presentation 层（Buff 特效）、BattleRecord |
| 技能释放 | CueEvent Message（SkillCastCue） | Presentation 层（施法动画） |
| 移动完成 | CueEvent Message（MovementCue） | Presentation 层（移动动画） |
| 状态变化 | CueEvent Message（StatusCue） | Presentation 层（状态特效） |

---

# 不变量

## 不变量1：CueEvent 只携带纯数据 🟥

任意时刻：

CueEvent 的所有字段必须是原始数据类型（Entity、u32、i32、enum 等），禁止携带 Handle、AssetId、PathBuf 等表现资源引用。

违反表现：
DamageCue 中出现 `effect_handle: Handle<Scene>` 字段。

---

## 不变量2：业务逻辑不消费 CueEvent 🟥

任意时刻：

Core 层的业务 System 不得通过 EventReader 读取 CueEvent。CueEvent 的消费者仅限 Presentation 层和 Audit 层。

违反表现：
Effect System 内部 `EventReader<DamageCue>` 读取后执行伤害逻辑。

---

## 不变量3：CueEvent 通过 Bevy Message 分发 🟥

任意时刻：

CueEvent 必须通过 Bevy 的 EventWriter/EventReader 机制分发，禁止通过自定义回调、函数指针、全局 Channel 等方式传递。

违反表现：
业务代码中出现 `on_damage_callback: Box<dyn Fn(DamageCue)>` 形式的回调注册。

---

## 不变量4：Cue 发射不阻塞业务执行 🟥

任意时刻：

CueEvent 的发射（EventWriter::send）不得阻塞业务逻辑的执行。Cue 是"发射后不管"（fire-and-forget）模式，业务逻辑不等待表现层响应。

违反表现：
业务 System 在发射 CueEvent 后立即查询表现层组件状态（反向依赖）。

---

## 不变量5：Cue 不参与确定性回放 🟥

任意时刻：

CueEvent 本身不被 Replay 系统记录和回放。Replay 记录的是触发 Cue 的上游事件（Ability 使用、Effect 执行等），回放时上游事件重新执行会自然重新发射 Cue。

违反表现：
BattleRecord 中存储了 CueEvent 的序列化数据用于回放。

---

# 流程定义

## Cue 发射管线

```
业务逻辑执行（Effect Execute / Buff Apply / Movement Complete）
    ↓
产生业务结果（EffectResult / BuffResult / MoveResult）
    ↓
构造 CueEvent（纯数据，Entity ID + 数值 + 类型）
    ↓
EventWriter::<XxxCue>::send(cue_event)
    ↓
Bevy Event 系统分发
    ↓
Presentation 层 EventReader 消费
    ↓
播放动画 / 音效 / UI 更新
```

## Cue 与 BattleRecord 集成

```
业务逻辑执行
    ↓
产生 EffectResult（含 target_died、damage_dealt 等）
    ↓
构造 CueEvent → 发送给 Presentation 层
    ↓
同时构造 AuditEvent → 发送给 BattleRecord
    ↓
BattleRecord 记录 AuditEvent（非 CueEvent）
```

## 新增 Cue 类型流程

```
1. 定义 CueEvent Struct（纯数据字段）
    ↓
2. 注册为 Bevy Message（app.add_event::<XxxCue>()）
    ↓
3. 在 CueRegistry 中注册类型元数据
    ↓
4. 在业务逻辑中添加 EventWriter::<XxxCue>::send()
    ↓
5. Presentation 层实现 EventReader::<XxxCue> Handler
    ↓
6. 添加单元测试验证 Cue 发射
```

---

# 数据结构

> 以下数据结构在术语定义章节中已定义，此处仅做索引参考。

| 数据结构 | 定义位置 | 说明 |
|----------|---------|------|
| DamageCue | 术语定义 · CueEvent | 伤害表现事件（target, amount, is_critical, element） |
| HealCue | 术语定义 · CueEvent | 治疗表现事件（target, amount） |
| DeathCue | 术语定义 · CueEvent | 死亡表现事件（entity, killer） |
| BuffAppliedCue | 术语定义 · CueEvent | Buff 施加表现事件（target, buff_id, stacks） |
| BuffRemovedCue | 术语定义 · CueEvent | Buff 移除表现事件（target, buff_id） |
| StatusCue | 术语定义 · CueEvent | 状态变化表现事件（entity, status, active） |
| SkillCastCue | 术语定义 · CueEvent | 技能释放表现事件（caster, skill_id, target_pos） |
| MovementCue | 术语定义 · CueEvent | 移动表现事件（entity, from, to） |

---

# 禁止事项

- 🟥 禁止：CueEvent 搭载表现资源引用（Handle/AssetId/Path） — 理由：Cue 是纯数据信号，不绑定具体表现形式
- 🟥 禁止：业务逻辑消费 CueEvent — 理由：Core 层反向依赖 Presentation 层
- 🟥 禁止：通过回调/Channel 等非 Bevy Message 方式分发 Cue — 理由：破坏 ECS 数据流模型
- 🟥 禁止：Cue 发射阻塞业务执行 — 理由：fire-and-forget 模式，业务不等待表现
- 🟥 禁止：CueEvent 参与 Replay 回放 — 理由：Replay 记录上游事件，Cue 是派生产物
- 🟥 禁止：Presentation 层通过 Cue 反向修改业务状态 — 理由：单向数据流，Core → Presentation
- 🟥 禁止：使用 DomainEvent 大枚举包裹 CueEvent — 理由：每个 CueEvent 必须是独立 Struct（对齐领域事件审计架构）
- 🟥 禁止：CueHandler 中查询/修改业务层组件（Attributes/ActiveBuffs/SkillSlots） — 理由：Presentation 层只读业务状态

---

# 与相邻领域的关系

| 相邻领域 | 关系 | 边界 |
|----------|------|------|
| **Effect** | Effect 执行后发射对应 CueEvent（DamageCue/HealCue） | Effect 拥有业务执行；Cue 拥有表现信号定义 |
| **Trigger** | Trigger 链可能触发额外 CueEvent（如被动技能触发特效） | Trigger 拥有触发逻辑；Cue 拥有事件定义 |
| **Buff** | Buff 施加/移除时发射 BuffAppliedCue/BuffRemovedCue | Buff 拥有生命周期；Cue 拥有表现信号 |
| **BattleRecord** | BattleRecord 记录 AuditEvent，不记录 CueEvent | Audit 是审计数据；Cue 是表现信号 |
| **Replay** | Replay 回放上游事件，Cue 自然重新发射 | Replay 拥有确定性回放；Cue 是派生产物 |
| **Presentation** | Presentation 层订阅 CueEvent 并执行具体表现 | Presentation 拥有表现实现；Cue 拥有信号定义 |
| **Battle** | Battle 流程中的关键节点发射 CueEvent（回合开始/结束） | Battle 拥有流程控制；Cue 拥有信号定义 |

---

# AI 修改规则

## 宪法合规检查清单

修改本领域代码前，必须逐项确认：
- 🟥 CueEvent 所有字段为纯数据类型（无 Handle/AssetId/Path）
- 🟥 新增 CueEvent 通过 Bevy Message 注册（app.add_event）
- 🟥 CueEvent 在 CueRegistry 中注册元数据
- 🟥 业务逻辑不消费 CueEvent（Core 层无 EventReader<CueEvent>）
- 🟥 Presentation 层不修改业务状态
- 🟥 Cue 发射不阻塞业务执行

## 如果新增 CueEvent 类型

允许：
- 定义新的 CueEvent Struct（纯数据字段）
- 在 CueRegistry 中注册类型
- 在业务逻辑中添加 EventWriter 发射点
- 在 Presentation 层实现 Handler

禁止：
- CueEvent 携带表现资源引用
- 业务 System 读取 CueEvent
- 不注册到 CueRegistry

优先检查：
- 字段是否全部为原始数据类型
- 触发时机是否明确（哪个业务节点发射）
- Presentation 层 Handler 是否已规划
- 是否与现有 CueEvent 类型重复

---

# 交叉引用

| 主题 | 详细文档 |
|------|----------|
| Logic/Presentation 分离原则 | `docs/01-architecture/README.md` §Logic/Presentation 分离 |
| GameplayCue 模式说明 | `docs/01-architecture/README.md` §领域事件与审计系统 |
| 废除 DomainEvent 大枚举 | `docs/01-architecture/README.md` §领域事件与审计系统 |
| 领域事件审计架构 | `docs/01-architecture/events_audit_design.md` |
| Effect Pipeline（Cue 发射来源） | `docs/02-domain/effect/effect-rules.md` |
| Trigger 系统（Cue 发射来源） | `docs/02-domain/trigger/trigger-rules.md` |
| Buff 系统（Cue 发射来源） | `docs/02-domain/buff/buff-rules.md` |
| BattleRecord（审计数据，非 Cue） | `docs/01-architecture/events_audit_design.md` §双轨制日志 |
| Replay 系统（Cue 不参与回放） | `docs/02-domain/03-technical/replay-rules.md` |
| GAS 链路中 Cue 的位置 | `docs/其他/77.md` §十（100分方案） |
| Cue 模块设计说明 | `docs/其他/77.md` §六（缺失模块4：Cue） |
