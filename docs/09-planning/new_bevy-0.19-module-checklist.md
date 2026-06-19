# Bevy 0.19 激进重构逐模块检查清单

> 策略：激进重构，不计代价，一步到位
> 每个模块列出所有重构项，不区分 Phase
> 状态：⬜ 未开始 | 🟡 进行中 | ✅ 完成 | ❌ 阻塞

---

## L0：Shared 层

### shared/ids/
- [ ] cargo check 通过（0.19）
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### shared/error/
- [ ] cargo check 通过
- [ ] nextest 通过

### shared/random/
- [ ] cargo check 通过
- [ ] nextest 通过

### shared/testing/
- [ ] cargo check 通过
- [ ] 添加 run_frames() 测试工具（支持 Observer + Delayed Command 测试）
- [ ] 添加 Observer 深度追踪测试工具
- [ ] nextest 通过

### shared/time/
- [ ] cargo check 通过
- [ ] 检查 Time API 变更
- [ ] nextest 通过

### shared/math/
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### shared/hashing/
- [ ] cargo check 通过
- [ ] nextest 通过

### shared/path/
- [ ] cargo check 通过
- [ ] nextest 通过

### shared/collections/
- [ ] cargo check 通过
- [ ] nextest 通过

### shared/diagnostics/
- [ ] cargo check 通过
- [ ] nextest 通过

### shared/validation/
- [ ] cargo check 通过
- [ ] nextest 通过

### shared/traits/
- [ ] cargo check 通过

### shared/prelude/
- [ ] cargo check 通过

### shared/localization_key.rs
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive

### shared/shared_plugin.rs
- [ ] cargo check 通过
- [ ] 检查 Plugin API 变更

---

## L1：Capabilities

### ability/
- [ ] cargo check 通过
- [ ] Observer if 守卫 → run_if（提取到 conditions.rs）
- [ ] spawn → spawn_scene + bsn!
- [ ] CasterOf Entity 字段 → Relationship
- [ ] 补齐 Reflect derive
- [ ] SceneComponent 化
- [ ] nextest 通过

### aggregator/
- [ ] cargo check 通过
- [ ] 批量运算 → contiguous_iter
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### attribute/
- [ ] cargo check 通过
- [ ] 批量运算 → contiguous_iter（Health += Regen 等）
- [ ] bypass_change_detection 高频数据
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### condition/
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### cue/
- [ ] cargo check 通过
- [ ] font_size: f32 → FontSize::Px(f32)
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### effect/（核心管线）
- [ ] cargo check 通过
- [ ] Observer if 守卫 → run_if
- [ ] EffectTarget Entity 字段 → Relationship
- [ ] spawn → spawn_scene + bsn!
- [ ] 补齐 Reflect derive
- [ ] SceneComponent 化
- [ ] Effect Tick → contiguous_iter
- [ ] 确认 Effect/Modifier 管线未受影响
- [ ] nextest 通过

### event/
- [ ] cargo check 通过
- [ ] 检查 Observer/Trigger API 变更
- [ ] nextest 通过

### execution/
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### gameplay_context/
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### modifier/（核心管线）
- [ ] cargo check 通过
- [ ] 确认 Effect/Modifier 管线未受影响
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### runtime/command/
- [ ] cargo check 通过
- [ ] 创建 DelayedBattleCommand 封装
- [ ] 创建 DelayedCommandLoop 工具
- [ ] nextest 通过

### runtime/pipeline/
- [ ] cargo check 通过
- [ ] nextest 通过

### runtime/registry/
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### runtime/replay/
- [ ] cargo check 通过
- [ ] 确认 Replay 确定性未受影响
- [ ] Relationship 序列化兼容
- [ ] nextest 通过

### runtime/scheduler/
- [ ] cargo check 通过
- [ ] nextest 通过

### spec/
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### stacking/（Buff 系统）
- [ ] cargo check 通过
- [ ] Observer if 守卫 → run_if
- [ ] BuffParent Entity 字段 → Relationship
- [ ] Buff Tick → contiguous_iter
- [ ] Timer → Delayed Commands（Buff 过期）
- [ ] 补齐 Reflect derive
- [ ] SceneComponent 化
- [ ] nextest 通过

### tag/
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### targeting/
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### trigger/
- [ ] cargo check 通过
- [ ] 检查 Trigger API 变更
- [ ] Observer if 守卫 → run_if
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

---

## L1：Domains

### combat/（最复杂域）
- [ ] cargo check 通过
- [ ] Observer if 守卫 → run_if（P0 优先级）
- [ ] 创建 combat/conditions.rs（battle_is_running 等）
- [ ] CasterOf/DamageTarget → Relationship
- [ ] spawn → spawn_scene + bsn!
- [ ] Timer → Delayed Commands（死亡动画/连击）
- [ ] 补齐 Reflect derive
- [ ] SceneComponent 化
- [ ] Vignette 受伤特效
- [ ] 确认战斗管线未受影响
- [ ] nextest 通过

### spell/
- [ ] cargo check 通过
- [ ] Observer if 守卫 → run_if
- [ ] Timer → Delayed Commands（法术冷却/DOT）
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### tactical/
- [ ] cargo check 通过
- [ ] Lens Distortion 战术特效
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### terrain/
- [ ] cargo check 通过
- [ ] Infinite Grid 调试网格
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### faction/
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### reaction/
- [ ] cargo check 通过
- [ ] Observer if 守卫 → run_if
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### inventory/
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### progression/
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### party/
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### camp_rest/
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### narrative/
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### quest/
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### economy/
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### crafting/
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### summon/
- [ ] cargo check 通过
- [ ] SummonedBy/OwnedBy → Relationship
- [ ] spawn → spawn_scene + bsn!
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

---

## L2：Infra 层

### input/
- [ ] cargo check 通过
- [ ] 检查 Input API 变更
- [ ] EditableText 集成
- [ ] nextest 通过

### localization/
- [ ] cargo check 通过
- [ ] Timer → Delayed Commands
- [ ] nextest 通过

### logging/
- [ ] cargo check 通过
- [ ] Observer if 守卫 → run_if（10 个领域 logger）
- [ ] nextest 通过

### pipeline/
- [ ] cargo check 通过
- [ ] nextest 通过

### registry/
- [ ] cargo check 通过
- [ ] 补齐 Reflect derive
- [ ] nextest 通过

### replay/
- [ ] cargo check 通过
- [ ] 确认 Replay 确定性未受影响
- [ ] Relationship 序列化兼容
- [ ] nextest 通过

### save/
- [ ] cargo check 通过
- [ ] 确认 Save/Load 兼容性
- [ ] Relationship 序列化兼容
- [ ] Handle 序列化
- [ ] nextest 通过

### settings/（新增）
- [ ] 创建模块
- [ ] 定义 AudioSettings
- [ ] 定义 VideoSettings
- [ ] 定义 GameplaySettings
- [ ] 定义 ControlSettings
- [ ] 注册 PreferencesPlugin
- [ ] auto_save_preferences 系统

### vfx/（新增）
- [ ] 创建模块
- [ ] Vignette 受伤特效
- [ ] Lens Distortion 战术特效
- [ ] Infinite Grid 调试网格
- [ ] Feature Flag 控制

---

## 横切层

### app/
- [ ] cargo check 通过
- [ ] 检查 AppBuilder API 变更
- [ ] 检查 State/States trait 变更
- [ ] 注册 PreferencesPlugin
- [ ] 注册 Settings Group
- [ ] nextest 通过

### content/
- [ ] cargo check 通过
- [ ] 检查 AssetServer/Assets API 变更
- [ ] Timer → Delayed Commands（hot_reload）
- [ ] 补齐资产类型 Reflect derive
- [ ] Handle 序列化
- [ ] nextest 通过

### modding/
- [ ] cargo check 通过

### tools/
- [ ] 删除 bevy-inspector-egui 相关代码
- [ ] 添加 DiagnosticsOverlay（dev feature）
- [ ] 添加 Text Gizmos（dev feature）
- [ ] 添加 Transform Gizmo（dev feature）
- [ ] cargo check 通过

---

## 统计

| 层级 | 模块数 | 重构项 |
|------|--------|--------|
| Shared | 14 | 20 |
| Capabilities | 20 | 52 |
| Domains | 15 | 38 |
| Infra | 9（含新增2） | 28 |
| 横切层 | 4 | 15 |
| **合计** | **62** | **153** |
