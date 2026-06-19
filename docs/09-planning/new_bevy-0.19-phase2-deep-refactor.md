# Phase 2：深度架构重构

> 前置条件：Phase 1 全面升级 + 全特性采用完成
> 目标：架构对齐 0.19 范式，不是修修补补而是根本性重构
> 前置文档：`new_bevy-0.19-migration-master-plan.md`

## 准入条件

- [ ] Phase 1 所有准出条件满足
- [ ] Phase 1 代码已合并到主分支
- [ ] 创建重构分支 `feat/bevy-0.19-deep-refactor`

---

## Refactor 1：Relationship 替代 Entity 字段

> 参考：`docs/03-technical/bevy-0.19-migration/02-observer-enhancements.md`
> 价值：ECS 原生关系机制，比裸 Entity 字段更安全、可查询、可序列化

### 1.1 关系清单

| 旧模式 | 新 Relationship | 自引用 |
|--------|----------------|--------|
| `caster: Entity` | `CasterOf(Entity)` → `CastBy(Vec<Entity>)` | 否 |
| `owner: Entity` | `OwnerOf(Entity)` → `OwnedBy(Vec<Entity>)` | 否 |
| `summoner: Entity` | `SummonedBy(Entity)` → `Summoned(Vec<Entity>)` | 否 |
| `aura_source: Entity` | `AuraSource(Entity)` → `AuraAffectees(Vec<Entity>)` | 否 |
| `target: Entity`（自我治疗）| `HealingTarget(Entity)` → `HealingSources(Vec<Entity>)` | 是 |
| `parent_buff: Entity` | `BuffParent(Entity)` → `BuffChildren(Vec<Entity>)` | 否 |

### 1.2 实施步骤

- [ ] 在 `core/capabilities/` 中定义 Relationship 组件：
  ```rust
  #[derive(Component)]
  #[relationship(relationship_target = CastBy)]
  pub struct CasterOf(pub Entity);

  #[derive(Component)]
  #[relationship_target(relationship = CasterOf)]
  pub struct CastBy(pub Vec<Entity>);
  ```
- [ ] 为需要自引用的关系添加 `allow_self_referential`
- [ ] 逐模块迁移 Entity 字段到 Relationship：
  - [ ] ability/ — CasterOf
  - [ ] effect/ — CasterOf, EffectTarget
  - [ ] stacking(buff)/ — BuffParent, BuffChildren
  - [ ] summon/ — SummonedBy, OwnedBy
  - [ ] combat/ — CasterOf, DamageTarget
- [ ] 更新所有 Query 以使用 Relationship
- [ ] 更新 Save/Replay 序列化（Relationship 自动序列化）
- [ ] 更新测试

### 1.3 准出条件

- [ ] 所有跨实体引用使用 Relationship
- [ ] 无裸 Entity 字段用于关系引用
- [ ] cargo nextest run 全绿
- [ ] Replay 确定性未受影响
- [ ] Save/Load 兼容

---

## Refactor 2：SceneComponent 预制体化

> 参考：`docs/03-technical/bevy-0.19-migration/01-bsn-scene-system.md`
> 价值：保证"如果组件存在，完整场景也存在"

### 2.1 预制体清单

| 实体类型 | SceneComponent | 场景函数 |
|----------|---------------|---------|
| Character | `#[scene("character.bsn")]` 或 `#[scene(character())]` | `fn character() -> impl Scene` |
| Ability | `#[scene(ability())]` | `fn ability() -> impl Scene` |
| Buff | `#[scene(buff())]` | `fn buff() -> impl Scene` |
| Effect | `#[scene(effect())]` | `fn effect() -> impl Scene` |
| MapEntity | `#[scene(map_entity())]` | `fn map_entity() -> impl Scene` |

### 2.2 实施步骤

- [ ] 为关键组件添加 `#[derive(SceneComponent)]`
- [ ] 定义场景函数：
  ```rust
  #[derive(Component, SceneComponent)]
  #[scene(character())]
  struct Character;

  fn character() -> impl Scene {
      bsn! {
          Character
          Health(100)
          Mana(50)
          Children [ AttributeSet, BuffContainer ]
      }
  }
  ```
- [ ] 迁移所有 spawn_scene 调用到 SceneComponent 模式
- [ ] 验证：直接 spawn 带 SceneComponent 的实体会自动生成完整场景

### 2.3 准出条件

- [ ] 关键实体有 SceneComponent
- [ ] spawn 自动生成完整场景
- [ ] cargo nextest run 全绿

---

## Refactor 3：Observer 链路优化

> 价值：消除 Observer 地狱，防止链式触发无限循环

### 3.1 问题识别

当前风险：
- DamageApplied → BuffTrigger → DamageApplied → BuffTrigger → ...
- Observer 链式触发导致不可预测的行为

### 3.2 解决方案

- [ ] 定义 Observer 链路规则：
  - Observer 只能跨领域通信（Ability→Effect, Effect→Buff, Buff→Character）
  - 领域内部仍然走 System
  - 禁止同领域 Observer 互相触发
- [ ] 实现 Observer 循环检测：
  ```rust
  /// Observer 执行深度追踪
  #[derive(Resource, Default)]
  struct ObserverDepth(u32);

  const MAX_OBSERVER_DEPTH: u32 = 8;
  ```
- [ ] 为所有 Observer 添加深度检查
- [ ] 重构循环触发链路为显式 System 调度

### 3.3 准出条件

- [ ] 无 Observer 无限循环风险
- [ ] Observer 链路深度有上限
- [ ] cargo nextest run 全绿

---

## Refactor 4：批处理思维重构

> 参考：`docs/03-technical/bevy-0.19-migration/05-contiguous-query.md`
> 价值：将 Attribute/Buff/Effect 系统从"逐实体"重构为"批量数据"

### 4.1 重构清单

| 系统 | 当前模式 | 目标模式 |
|------|---------|---------|
| Health Regen | 逐实体 Query | contiguous_iter 批量 |
| Buff Tick | 逐实体 Query | contiguous_iter 批量 |
| Effect Tick | 逐实体 Query | contiguous_iter 批量 |
| Attribute Calc | 逐实体 Query | contiguous_iter 批量 |
| Cooldown Tick | 逐实体 Query | contiguous_iter 批量 |

### 4.2 实施步骤

- [ ] 在 `shared/` 中封装批量运算工具：
  ```rust
  /// 批量属性运算
  fn batch_add_mut(base: &mut [f32], delta: &[f32], max: &[f32]) {
      for ((b, d), m) in base.iter_mut().zip(delta).zip(max) {
          *b = (*b + *d).min(*m);
      }
  }
  ```
- [ ] 逐系统重构为 contiguous_iter 版本
- [ ] 高频数据 bypass_change_detection
- [ ] 建立 criterion 基准测试

### 4.3 准出条件

- [ ] 所有批量运算使用 contiguous_iter
- [ ] criterion 基准测试显示性能提升
- [ ] cargo nextest run 全绿

---

## Refactor 5：战斗特效系统

> 参考：`docs/03-technical/bevy-0.19-migration/08-rendering-and-devtools.md`
> 价值：利用 0.19 后处理效果增强战斗反馈

### 5.1 特效清单

| 特效 | 0.19 特性 | 触发场景 |
|------|----------|---------|
| 受伤红色脉冲 | Vignette | DamageApplied |
| 战术技能扭曲 | Lens Distortion | TacticalAbility |
| 地图网格显示 | Infinite Grid | Debug 模式 |
| 实体位置调试 | Transform Gizmo | Debug 模式 |

### 5.2 实施步骤

- [ ] 创建 `src/infra/vfx/` 模块
- [ ] 实现 Vignette 受伤特效
- [ ] 实现 Lens Distortion 战术特效
- [ ] 实现 Infinite Grid 调试网格
- [ ] Feature Flag 控制

### 5.3 准出条件

- [ ] 受伤时有红色脉冲反馈
- [ ] Debug 模式下有地图网格
- [ ] cargo nextest run 全绿

---

## Refactor 6：Asset Saving + 编辑器基础

> 参考：`docs/03-technical/bevy-0.19-migration/09-asset-system.md`
> 价值：为未来编辑器铺路

### 6.1 实施步骤

- [ ] 实现 Asset Saving 基础设施
- [ ] 地图数据保存/加载
- [ ] 技能配置保存/加载
- [ ] 战斗回放保存
- [ ] 评估 Feathers Widget 用于编辑器 UI

### 6.2 准出条件

- [ ] 运行时资产保存正常工作
- [ ] cargo nextest run 全绿

---

## Phase 2 总准出条件

- [ ] cargo check 零错误
- [ ] cargo clippy 零警告
- [ ] cargo nextest run 全绿
- [ ] 所有跨实体引用使用 Relationship
- [ ] 关键实体有 SceneComponent
- [ ] Observer 无无限循环风险
- [ ] 批量运算使用 contiguous_iter
- [ ] 战斗特效正常工作
- [ ] Replay 确定性未受影响
- [ ] Save/Load 兼容性未受影响
- [ ] Effect/Modifier 管线未被绕过
