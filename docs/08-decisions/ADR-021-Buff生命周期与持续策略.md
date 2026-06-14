# ADR-021: Buff 生命周期与持续策略

## 状态

Proposed

## 背景

当前 Buff 系统的持续时间管理仅通过 `default_duration: u32` 字段实现，缺乏 `DurationPolicy` 枚举表达能力（直到死亡、直到移动、永久等）。叠层策略仅支持"同源同 id 刷新持续时间"的单一模式，无法表达"可叠 N 层/不可叠/叠满刷新"等常见 SRPG 需求。tick 生命周期的阶段执行（在 OnEnter TurnPhase::SelectUnit 时执行）需要明确规范。

本 ADR 定义 DurationPolicy 枚举、StackPolicy 枚举、tick 生命周期三要素。

## 引用的领域规则

- `docs/01-architecture/skill-buff-abstraction.md` — §4.6 Duration Policy、§4.7 StackPolicy、§4.8.2 Trigger 系统
- `docs/02-domain/skill/skill-rules.md` — 规则2：冷却管理
- `docs/02-domain/turn/turn-rules.md` — TurnPhase 周期

## 决策

### 1. DurationPolicy 枚举

```rust
/// 持续策略 — Buff 持续多久
pub enum DurationPolicy {
    /// 持续 N 回合（最常见）
    Turns(u32),
    /// 直到死亡才消失
    UntilDeath,
    /// 移动后消失
    UntilMove,
    /// 攻击后消失
    UntilAttack,
    /// 受伤后消失（伤害抵消后消失，如护盾）
    UntilDamaged,
    /// 战斗结束消失
    BattleEnd,
    /// 永久（直到手动移除）
    Permanent,
}
```

| 策略 | 说明 | 典型用例 |
|------|------|----------|
| Turns(3) | 持续 3 回合，tick 递减 | 中毒、护盾 |
| UntilDeath | 直到死亡才消失 | 标记、诅咒 |
| UntilMove | 移动后消失 | 蓄力、坚守 |
| UntilAttack | 攻击后消失 | 蓄力、反击准备 |
| UntilDamaged | 受伤后消失 | 护盾、隐身 |
| BattleEnd | 战斗结束消失 | 状态加成、场地效果 |
| Permanent | 永久（直到手动移除） | 被动光环、种族特质 |

### 2. `DurationDef` RON 格式

```ron
// RON 中的持续时间表示
duration: Turns(3)         // 持续 3 回合
duration: UntilDeath       // 直到死亡
duration: Permanent        // 永久
duration: UntilMove        // 移动后消失
```

`#[serde(default = "default_duration")]` 保证旧配置（使用 `default_duration: u32`）兼容：

```rust
fn default_duration() -> DurationDef {
    DurationDef::Turns(1)
}
```

### 3. StackPolicy 枚举

```rust
/// 叠层策略 — Buff 如何叠加
pub enum StackPolicy {
    /// 不可叠加，重复施加刷新持续时间
    NoStack,
    /// 可叠加 N 层，达到上限后刷新最旧层的持续时间
    Stackable(u32),
    /// 可叠加 N 层，达到上限后不再接受新叠加
    StackableNoRefresh(u32),
}
```

| 策略 | 说明 | 典型用例 |
|------|------|----------|
| NoStack | 不可叠，刷新 duration | 易伤、护盾 |
| Stackable(5) | 可叠 5 层 | 中毒、流血 |
| StackableNoRefresh(3) | 可叠 3 层，满了不再叠 | 狂怒层数 |

### 4. `StackDef` RON 格式

```ron
stack: NoStack              // 不可叠加
stack: Stackable(5)         // 可叠 5 层
stack: StackableNoRefresh(3) // 可叠 3 层，满上限后无效
```

### 5. tick 生命周期

当前 tick 在 `OnEnter(TurnPhase::SelectUnit)` 触发。按 ADR-021 规范化为固定阶段：

| 阶段 | 操作 | 说明 |
|------|------|------|
| TurnPhase::SelectUnit | resolve_status_effects | DoT/HoT 结算、tick_buffs、冷却 tick |
| Buff 施加时 | apply_buff | 添加修饰符、添加标签 |
| Buff 移除时 | remove_buff | 清理修饰符、清理标签、触发 BuffRemoved 事件 |
| 单位移动后 | 检查 UntilMove Buff | 若有 → 移除 |
| 单位攻击后 | 检查 UntilAttack Buff | 若有 → 移除 |

### 6. `apply_buff()` 的 StackPolicy 逻辑

当前 `apply_buff()` 的刷新逻辑需要扩展到支持三种 StackPolicy：

```rust
// 当前逻辑（同源同 id 刷新）
if existing.source == source && existing.buff_id == id {
    existing.remaining = duration;
    return existing.id;
}

// 目标逻辑
match stack_policy {
    NoStack => {
        // 检查是否有同 buff_id 的实例
        // 若有 → 刷新 duration，不新增
        // 若无 → 正常新增
    }
    Stackable(max) => {
        // 检查当前层数
        // 若 < max → 新增实例
        // 若 >= max → 移除最旧实例，新增实例
        // 同源同 id → 刷新 duration
    }
    StackableNoRefresh(max) => {
        // 检查当前层数
        // 若 < max → 新增实例
        // 若 >= max → 跳过
    }
}
```

### 7. DurationPolicy 过期检查

`tick_buffs()` 需要扩展为处理多种 DurationPolicy：

```rust
pub fn tick_buffs(
    buffs: &mut ActiveBuffs,
    attrs: &mut Attributes,
    tags: &mut GameplayTags,
    persistent: &PersistentTags,
    phase: TickPhase,  // 当前处于什么阶段
) {
    match phase {
        TickPhase::TurnStart => {
            // 处理 DurationPolicy::Turns(n)
            // decrement remaining_turns
            // remaining_turns == 0 → 过期
        }
        TickPhase::AfterMove => {
            // 移除 DurationPolicy::UntilMove 的 Buff
        }
        TickPhase::AfterAttack => {
            // 移除 DurationPolicy::UntilAttack 的 Buff
        }
        TickPhase::AfterDamaged => {
            // 移除 DurationPolicy::UntilDamaged 的 Buff
        }
        TickPhase::Manual => {
            // 主动移除：不扩容
        }
    }
}
```

## Module Design

### 类型定义位置

```
src/core/buff/
├── domain/
│   ├── mod.rs               ← BuffRegistry
│   ├── types.rs             ← DurationPolicy, StackPolicy, DurationDef, StackDef（新增）
│   └── buff_error.rs
├── apply.rs                 ← 更新 apply_buff 处理 StackPolicy
├── instance.rs              ← ActiveBuffs 增加 stack_counts 字段（可选）
├── resolve.rs               ← 扩展 tick_buffs 处理 DurationPolicy
└── trigger.rs               ← 后续 ADR
```

### 迁移路径

| 步骤 | 变更 | 影响 |
|------|------|------|
| 1 | 定义 DurationPolicy/StackPolicy 枚举 | 新增类型，无破坏 |
| 2 | 更新 BuffData 新增 duration/stack 字段 | 新增可选字段，默认兼容旧行为 |
| 3 | 更新 apply_buff() 使用 StackPolicy 逻辑 | 核心逻辑变更，需充分测试 |
| 4 | 更新 tick_buffs() 使用 DurationPolicy | 核心逻辑变更 |
| 5 | 更新 8 个 RON Buff 文件配置 | 横向变更，不影响编译 |
| 6 | 移除 default_duration 旧字段 | 破坏性变更，最后执行 |

## Communication Design

### 内部调用

- `tick_buffs()` 由 `resolve_status_effects()` 在 TurnSelect 阶段调用
- `apply_buff()` 由 EffectHandler 和外部调用者触发
- 过期检查不产生新事件（BuffRemoved 由 resolve.rs 发送）

### 与 Turn 系统的关系

Buff 的 tick 生命周期与回合阶段同步：

```
OnEnter(TurnPhase::SelectUnit)
  └─ resolve_status_effects
       ├─ DoT/HoT 结算
       ├─ DurationPolicy::Turns 过期检查
       ├─ tick_buffs（递减 + 过期清理）
       ├─ rebuild_tags
       └─ cooldowns.tick()
```

## 边界定义

- 允许：`core/buff/` 定义 DurationPolicy/StackPolicy 枚举
- 允许：`core/buff/resolve.rs` 接收 `TickPhase` 参数
- 禁止：DurationPolicy 的业务逻辑泄漏到 `core/turn/` 模块
- 禁止：tick_buffs 在非 TurnPhase::SelectUnit 阶段执行（TillDeath/TillMove 等阶段通过事件触发）
- 禁止：DurationPolicy 枚举变体包含执行逻辑（必须为纯数据标记）
- 禁止：StackPolicy 的层数上限超过 99（u32 最大合理值）

## Forbidden（禁止事项）

- 🟥 禁止：使用 `default_duration: u32` 替代 DurationPolicy — 理由：缺乏语义表达能力
- 🟥 禁止：tick 生命周期在 TurnEnd 以外的阶段执行 — 理由：与回合生命周期同步
- 🟥 禁止：Buff 永不过期且没有 DurationPolicy::Permanent 标记 — 理由：可审计性
- 🟥 禁止：不同 StackPolicy 的 Buff 实例共享同一个 instance_id — 理由：叠层追踪
- 🟥 禁止：StackPolicy::Stackable 超过上限时不处理（应移除最旧实例）— 理由：内存泄漏
- 🟥 禁止：UntilDamaged Buff 在任意伤害来源时都触发（应和 Modifier 管线联动记录伤害来源）— 理由：需要护盾/减伤判断

## Definition / Instance Design

- Definition：DurationDef（RON 表示）、DurationPolicy（运行时）、StackDef（RON 表示）、StackPolicy（运行时）
- Instance：BuffInstance.remaining_turns（仅对 DurationPolicy::Turns 有效）
- Instance：ActiveBuffs（Vec\<BuffInstance\>，叠层策略在此实现）

## 后果

### 正面
- DurationPolicy 提供 7 种持续策略，覆盖绝大部分 SRPG 需求
- StackPolicy 提供 3 种叠层策略，消除现有"硬编码同源刷新"的限制
- 现有 default_duration 字段可通过 Phase 迁移平滑过渡

### 负面
- tick_buffs 需要区分 TickPhase，增加少量代码复杂度
- Stackable(max) 需要追踪实例创建顺序（Vec 顺序天然支持）
- 8 个 RON 文件需要逐个更新

## 替代方案

| 方案 | 优点 | 缺点 | 为何放弃 |
|------|------|------|----------|
| 保持 u32 字段 | 简单 | 无法表达复杂持续策略 | DurationPolicy 更灵活 |
| DurationPolicy 只保留 Turns 和 Permanent | 实现简单 | 无法覆盖护盾/蓄力等场景 | 不够用 |
| StackPolicy 仅在 BuffDef 中定义 | 配置驱动 | 运行时需要枚举匹配 | 当前方案已覆盖 |
