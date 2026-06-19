---
id: 02-domain.cue
title: Cue（表现层信号）领域规则 v1.0
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-19
tags:
  - domain
  - cue
  - capabilities
---


## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Cue | 表现层触发信号，逻辑层通知表现层"该播放什么"的桥梁 | 负责：信号的定义、分类与分发到 Infra 表现层，Cue 的 LocalizationKey（name_key/desc_key）；不负责：信号的实际表现执行 |
| CueType | 表现信号类型枚举，定义信号的表现类别 | 负责：信号分类（VFX/SFX/Animation/Shake/Popup）；不负责：信号的具体参数 |
| CueData | 表现信号的数据载体，包含信号标识、上下文和参数 | 负责：携带表现层执行所需的信息；不负责：表现层的执行方式 |
| CueTag | 信号关联的业务时机标签，标记信号在效果生命周期的哪个阶段触发 | 负责：触发时机标识（OnApply/OnRemove/OnTick）；不负责：触发时机的判断逻辑 |
| CueContainer | 实体上注册的表现信号容器，管理该实体关联的所有 Cue | 负责：信号的注册/移除管理；不负责：信号的分发执行 |
| CueDispatch | 信号分发器，将 Cue 事件分发到 Infra 表现层对应的处理系统 | 负责：根据 CueType 路由到对应表现层系统；不负责：表现层的实际渲染/播放 |

### CueType 分类

```
CueType
 ├── VFX（视觉特效）      — 粒子/光效/拖尾/爆炸等
 │    示例：火球术的爆炸粒子、治疗术的绿色光效
 ├── SFX（音效）          — 音效/语音/环境声
 │    示例：挥剑声、法术吟唱、命中音效
 ├── Animation（动画）    — 骨骼动画/状态切换/蒙太奇
 │    示例：受击动画、死亡动画、施法动作
 ├── Shake（屏幕震动）     — 镜头抖动/冲击波
 │    示例：爆炸时屏幕震动、地震术
 └── Popup（UI 浮动文字）  — 伤害数字/治疗数字/状态提示
       示例：-15 伤害数字飘出、+23 治疗数字、IMMUNE 文字
```

### CueTag 触发时机

```
CueTag
 ├── OnApply（施加时）     — 效果/技能被应用时触发
 │    示例：火球炸裂特效、中毒施加时的绿色闪光
 ├── OnTick（周期 Tick 时） — 持续效果的每次 Tick 触发
 │    示例：中毒每跳时的伤害数字飘出
 ├── OnRemove（移除时）     — 效果被移除时触发
 │    示例：Buff 消失时的消散特效、中毒解除时的净化光效
 └── OnInterrupt（被打断时） — 技能/效果被打断时触发
       示例：施法被打断时的火花四溅
```

### Cue 作为逻辑与表现的桥梁

```
Core 层（逻辑）                          Infra 层（表现）
                                          
Ability/Effect/Condition/etc.           渲染/音效/动画/UI
        │                                      │
        │── CueTriggered 事件 ──→              │
        │    {                                 │
        │      cue_type: VFX,                 │──→ VFX 系统：播放粒子
        │      cue_id: "fireball_explosion",  │──→ SFX 系统：播放音效
        │      context: {pos, direction},      │──→ 动画系统：播放爆炸动画
        │      cue_tag: OnApply               │──→ 震动系统：屏幕震动
        │    }                                 │
        │                                      │
        │  逻辑层不知道也不关心                │  表现层负责如何实现
        │  表现层如何实现                      │  逻辑层只发信号
```

### 已对齐项目术语

- **Effect**：CueTag(OnApply/OnTick/OnRemove) 与 Effect 生命周期的三阶段对应
- **Ability**：技能激活/执行/取消时触发 Cue
- **Event**：Cue 通过 Event 领域的事件机制（CueTriggered）分发到 Infra 层
- **Condition**：Condition 评估结果（如免疫触发）可触发 Cue 显示 IMMUNE 文字

---

## 2. Cue 状态机

```
Defined（已定义——在内容配置中定义）
   │  [注册到 CueContainer]
   ▼
Registered（已注册——等待触发条件满足）
   │  [触发时机到达]
   ▼
Triggered（已触发——CueTriggered 事件已发送）
   │  [事件分发到 Infra 表现层]
   ▼
Dispatched（已分发——Infra 层已接收）
   │  [表现层执行中/已完成]
   ▼
Consumed（消耗完毕——表现层反馈已执行）
```

### 状态转换规则

| 转换 | 触发条件 | 动作 |
|------|---------|------|
| Defined → Registered | Cue 注册到实体的 CueContainer | 准备就绪，等待触发 |
| Registered → Triggered | 对应的 CueTag 时机到达 | 发布 CueTriggered 事件 |
| Triggered → Dispatched | Event 事件分发到 Infra 层 | Infra 层接收并处理 |
| Dispatched → Consumed | 表现层执行完毕（可选反馈） | 记录已消费 |
| 禁止 | 表现层反向影响逻辑层 | Cue 是单向信号，表现层不应通过 Cue 反向修改逻辑 |

---

## 3. 不变量（Invariants）

### 3.1 Cue 是只读信号
- **条件**：任何 Cue 被触发和分发时
- **不变量**：Cue 是单向信号——逻辑层 → 表现层，禁止表现层通过 Cue 反向修改逻辑层状态
- **违反后果**：表现层与逻辑层耦合，架构分层被破坏

### 3.2 Cue 不影响游戏逻辑
- **条件**：任何 Cue 的执行过程中
- **不变量**：Cue 触发/分发/执行的任何环节不得影响游戏逻辑的进行
- **违反后果**：Cue 执行失败导致技能/效果执行被阻塞

### 3.3 Cue 触发时机与逻辑同步
- **条件**：CueTag(OnApply) 和 EffectApplied 事件之间
- **不变量**：CueTag(OnApply) 必须在 Effect 逻辑完成后、下一帧渲染前触发
- **违反后果**：表现与逻辑不同步（如伤害数字先弹出、伤害效果后生效）

### 3.4 Cue 的可选性
- **条件**：游戏运行中
- **不变量**：所有 Cue 必须可以被独立禁用（性能模式/无障碍模式/无头模式）
- **违反后果**：Cue 不可禁用时，无头模式/性能模式无法正常工作

### 3.5 Cue 数据不应包含业务敏感信息
- **条件**：创建 CueData 时
- **不变量**：CueData 不得包含未公开的游戏数据（如未探索区域的实体属性）
- **违反后果**：表现层泄露了本不应展示的游戏数据

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：表现层通过 Cue 反向修改 Core 层数据 — 理由：Cue 是单向通信桥梁，反向修改破坏架构分层
- 🟥 禁止：Cue 执行失败影响游戏逻辑执行 — 理由：Cue 是可选的表现增强，逻辑应完整独立于表现
- 🟥 禁止：逻辑层直接调用表现层 API 播放特效/音效 — 理由：必须通过 Cue 事件解耦，禁止逻辑层直接依赖表现层
- 🟥 禁止：Cue 携带过大的数据载荷（如完整纹理/音频数据） — 理由：CueData 是轻量信号，重资源通过 asset 系统加载
- 🟥 禁止：玩家输入/UI 操作通过 Cue 影响逻辑层 — 理由：输入应走 Input→Command 路径，不经过 Cue
- 🟥 禁止：CueDef 中直接存储用户可见文本的自然语言文本 — 理由：必须使用 name_key/desc_key: LocalizationKey 引用。违反宪法 §22 Localization First。

---

## 5. 流程定义

### 5.1 Cue 触发

- **输入**：触发时机（CueTag）、触发信号定义（CueData）
- **处理**：
  1. 检查该 Cue 是否被禁用（不变量 3.4）
  2. 创建 CueData 实例（携带 cue_id、context、parameters）
  3. 校验 CueData 中不包含业务敏感信息（不变量 3.5）
  4. 通过 Event 领域发布 CueTriggered 事件
- **输出**：CueTriggered 事件
- **失败处理**：Cue 被禁用时静默跳过，不产生任何事件

### 5.2 Cue 分发

- **输入**：CueTriggered 事件
- **处理**：
  1. Event 系统接收 CueTriggered 事件
  2. 根据 CueType 路由到对应表现层系统：
     - VFX → Infra.presentation.vfx_player
     - SFX → Infra.presentation.sfx_player
     - Animation → Infra.presentation.anim_controller
     - Shake → Infra.presentation.camera/shake
     - Popup → Infra.presentation.ui/hud
  3. 表现层系统接收 CueData 并自行处理
- **输出**：分发结果
- **失败处理**：某个表现层系统未就绪时，该 Cue 被静默忽略（其他 Cue 不受影响）

### 5.3 Cue 注册

- **输入**：Cue 定义（CueType、CueTag、cue_id、参数模板）、所属实体
- **处理**：
  1. 校验 Cue 参数模板是否完整
  2. 注册到实体的 CueContainer
  3. 绑定到对应 Effect/Ability 的触发时机
- **输出**：注册确认
- **失败处理**：参数不完整时注册失败

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| CueTriggered | Cue 触发条件满足时 | cue_type, cue_id, cue_tag, context_data, parameters | Infra 全表现层（VFX/SFX/Animation/UI）、日志（LogCode: CNT009） |
| CueSuppressed | Cue 因被禁用/性能限制被跳过时 | cue_id, cue_type, reason（disabled/performance/distance） | 日志（LogCode: CNT010）、性能监控 |

### CueTriggered 事件分发图

```
CueTriggered（由 EventBus 分发）
    │
    ├──→ Infra/rendering/vfx_player：播放 VFX 特效
    ├──→ Infra/audio/sfx_player：播放音效
    ├──→ Infra/animation/anim_controller：触发动画
    ├──→ Infra/ui/hud：显示伤害数字/Popup
    └──→ Infra/presentation：屏幕震动/镜头效果
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：Cue 能力领域位于 `core/capabilities/cue/`，foundation/ 定义 cue_type.rs、cue_data.rs、cue_tag.rs，mechanism/ 定义 components.rs（CueContainer）和 cue_dispatch_system.rs，符合 C1→C2 分层
- ✅ 术语一致：CueType、CueData、CueTag、CueDispatch 与架构文档第六节完全一致
- ✅ 表现解耦：Cue 是逻辑层（Core）与表现层（Infra/presentation/）之间的唯一桥梁，Core 不直接调用表现层 API
- ✅ 可选机制：Cue 可被独立禁用，支持无头模式/性能模式
- ✅ 职责明确：Cue 只做"信号"，不做"表现执行"（Infra 的职责）、不做"事件通信"（Event 的职责）
- ✅ LocalizationKey：Cue 使用 LocalizationKey 而非硬编码文本（宪法 §22）

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖 Cue 定义、注册、触发、分发等全场景
- [x] 所有不变量和约束条件已识别（5 条不变量）
- [x] 禁止事项已明确列出（5 条禁止）
- [x] CueType 五种分类定义清晰（VFX/SFX/Animation/Shake/Popup）
- [x] 与 Effect 生命周期的三阶段对齐（OnApply/OnTick/OnRemove）
- [x] 每个操作有完整的流程定义（触发、分发、注册）
