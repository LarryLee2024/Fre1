---
id: capabilities.cue.schema.v1
title: Cue Schema — 表现信号数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition
replay-safe: true
---

# Cue Schema — 表现层信号数据架构

> **领域归属**: Capabilities — 行为表现层 | **依赖 Schema**: GameplayContext | **定义依据**: `docs/02-domain/capabilities/cue_domain.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `CueDef` | Definition | 表现信号的静态定义（类型、参数、触发时机） |
| `CueData` | Runtime | 信号触发时的数据载体（含定位、朝向等上下文） |
| `CueBinding` | Definition | 效果与 Cue 的绑定关系 |

---

## 2. Problem

Cue 是逻辑层与表现层的「单向桥梁」——所有 VFX/SFX/Animation/UI 反馈必须通过 Cue 发出。Schema 必须解决：
- 五种 CueType（VFX/SFX/Animation/Shake/Popup）的参数定义
- CueTag 触发时机（OnApply/OnTick/OnRemove/OnInterrupt）
- 信号参数的多样性和可扩展性
- Cue 的无限循环防护（表现层不得反向修改逻辑）

---

## 3. Schema Design

### 3.1 CueDef（Definition 层）

```rust
struct CueDef {
    /// 信号唯一标识
    id: CueDefId,

    /// 信号类型
    cue_type: CueType,

    /// 触发时机标签
    cue_tag: CueTag,

    /// 信号参数（取决于 cue_type）
    params: CueParams,

    /// 延迟触发（帧数）
    delay_frames: Option<u64>,

    /// 该 Cue 是否可被打断（新 Cue 到达时未执行完的旧 Cue 是否取消）
    interruptible: bool,

    /// 条件（可选，条件满足时才触发此 Cue）
    condition: Option<Condition>,

    /// 关键信号（必须被播放，不可丢弃）
    critical: bool,
}
```

### 3.2 CueType（Definition 层）

```rust
enum CueType {
    /// 视觉特效（粒子/光效/拖尾/爆炸）
    VFX(VFXParams),
    /// 音效（音效/语音/环境声）
    SFX(SFXParams),
    /// 动画（骨骼动画/状态切换）
    Animation(AnimationParams),
    /// 屏幕震动
    Shake(ShakeParams),
    /// UI 浮动文字（伤害数字/状态文字）
    Popup(PopupParams),
}

struct VFXParams {
    /// 特效资源 Key
    effect_key: String,
    /// 附着点（如 weapon_tip, chest, ground）
    attach_point: Option<String>,
    /// 是否跟随目标移动
    follow_target: bool,
    /// 持续时长
    duration_frames: Option<u64>,
    /// 缩放
    scale: Option<f32>,
    /// 颜色覆盖
    color_override: Option<String>,
}

struct SFXParams {
    /// 音效资源 Key
    sound_key: String,
    /// 音量 0.0–1.0
    volume: f32,
    /// 是否 3D 空间音效
    is_3d: bool,
    /// 音调偏移
    pitch_shift: Option<f32>,
}

struct AnimationParams {
    /// 动画名称
    animation_name: String,
    /// 播放速率
    speed: f32,
    /// 是否循环
    loop: bool,
    /// 淡入淡出时间
    crossfade_frames: Option<u64>,
}

struct ShakeParams {
    /// 振幅
    intensity: f32,
    /// 持续帧数
    duration_frames: u64,
    /// 衰减曲线
    falloff: ShakeFalloff,
}

enum ShakeFalloff {
    Linear,
    Exponential,
    None,
}

struct PopupParams {
    /// 文字内容 Key（本地化）
    text_key: LocalizationKey,
    /// 文字颜色
    color: String,
    /// 字体大小
    font_size: u8,
    /// 浮动方向
    float_direction: PopupDirection,
    /// 持续帧数
    duration_frames: u64,
}

enum PopupDirection {
    Up,
    Down,
    Left,
    Right,
    Random,
}
```

### 3.3 CueTag（Definition 层）

```rust
enum CueTag {
    /// 效果/技能被应用时触发
    OnApply,
    /// 持续效果的每次 Tick 触发
    OnTick,
    /// 效果被移除时触发
    OnRemove,
    /// 技能/效果被打断时触发
    OnInterrupt,
    /// 自定义触发时机
    Custom(String),
}
```

### 3.4 CueData（Runtime 层）

```rust
/// 信号触发时的完整数据载体。
struct CueData {
    /// CueDef ID
    cue_def_id: CueDefId,

    /// 触发时关联的实体
    source_entity: Option<EntityId>,
    target_entity: Option<EntityId>,

    /// 位置（用于 VFX/SFX 定位）
    position: Option<GridPosition>,
    /// 方向
    direction: Option<GridDirection>,

    /// 数值（用于 Popup 显示伤害/治疗数字）
    numeric_value: Option<f32>,

    /// 扩展数据
    extensions: HashMap<String, String>,

    /// 是否为关键信号
    critical: bool,
}
```

### 3.5 CueDefConfig（Definition 层 — 配置格式）

```yaml
# RON 配置示例 — Cue 定义
CueDefConfig:
  cues:
    # 示例1: 火焰爆炸 VFX
    - id: "cue_000001"
      cue_type:
        VFX:
          effect_key: "vfx_fire_explosion"
          attach_point: "target_center"
          follow_target: false
          duration_frames: 30
          scale: 1.5
      cue_tag: OnApply
      critical: false

    # 示例2: 受击音效
    - id: "cue_000002"
      cue_type:
        SFX:
          sound_key: "sfx_hit_heavy"
          volume: 0.8
          is_3d: true
      cue_tag: OnApply

    # 示例3: 死亡动画
    - id: "cue_000003"
      cue_type:
        Animation:
          animation_name: "death"
          speed: 1.0
          loop: false
          crossfade_frames: 5
      cue_tag: OnRemove
      critical: true

    # 示例4: 暴击伤害数字
    - id: "cue_000004"
      cue_type:
        Popup:
          text_key: "cue.cue_000004.text"
          color: "#FFD700"
          font_size: 24
          float_direction: Up
          duration_frames: 60
      cue_tag: OnApply
      condition:
        TagRequirement:
          mode: HasAny
          target_tags: ["tag_000090"]   # Combat.CriticalHit
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 可热重载 | 备注 |
|----------|-------|--------|----------|------|
| `CueDef` | Definition | 是（配置文件） | 是 | 信号定义 |
| `CueBinding` | Definition | 是（EffectDef 内嵌） | 是 | 绑定关系 |
| `CueData` | Runtime | 否 | 否 | 触发时瞬时数据 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → ConditionSchema | CueDef.condition |
| 依赖 | → GameplayContextSchema | CueData 从上下文派生 |
| 被依赖 | ← EffectSchema | Effect.cues 引用 CueDefId |
| 被依赖 | ← AbilitySchema | 技能激活/取消时触发 Cue |
| 被依赖 | ← EventSchema | CueTriggered 事件订阅 |

---

## 6. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | 资源 Key 非空 | Def 加载 | effect_key/sound_key/animation_name 不为空 |
| V2 | 音量在合法范围 | Def 加载 | 0.0 ≤ SFXParams.volume ≤ 1.0 |
| V3 | Cue 无副作用 | 运行时 | Cue 处理中修改逻辑数据的断言不通过 |

---

## 7. Replay Compatibility

Cue 是纯表现信号，不影响游戏逻辑状态回放。回放时根据需要决定是否播放 Cue（回放模式可禁用 Cue 以减少性能开销）。

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| Cue 触发 | 🟩 不影响回放逻辑 | Cue 不修改 ECS 数据 |
| Cue 执行 | 🟩 可屏蔽 | 回放时可选择跳过所有 Cue |

---

## 8. Save Compatibility

Cue 是纯运行时表现，不参与存档。

---

## 9. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2（未来） | 新增 Haptics CueType | 新增枚举 variant |

---

## 10. Future Extension

- **Cue 序列**: 一组 Cue 按顺序播放（如施法→光效→音效→震动）
- **Cue 预算**: 控制每帧播放的 Cue 数量上限（性能保护）
- **Cue 日志**: 所有 Cue 事件写入日志，供回放调试和性能分析

---

## 11. Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| Cue 风暴 | 同一帧触发大量 Cue 导致性能问题 | CueDispatch 设置每帧上限 + 优先级队列 |
| Cue 丢失 | 非关键 Cue 在高负载时被丢弃 | 区分 critical/non-critical |
| 表现层反向依赖 | 表现层 Cue 播放失败阻塞逻辑 | Cue 触发是"fire-and-forget"，逻辑不等待表现完成 |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| 表现必须经过 Cue | ✅ | 所有 VFX/SFX/动画/UI 反馈通过 Cue |
| Logic/Presentation Separation | ✅ | 单向信号，表现层不修改逻辑层 |
| Replay First | ✅ | Cue 不影响回放逻辑 |
