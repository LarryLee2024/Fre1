# 各领域迁移检查清单 — 激进版

> **本文档提供逐模块、逐文件的迁移检查清单，确保不遗漏任何受影响文件。**
> **变更说明（v1.0 → v2.0）**：删除"纯兼容"阶段，所有 Phase 1 检查项合并到 Phase A 中同步执行。总检查项从 91 项扩展到 134 项（含 Observer 转换、Bundle → BSN、Resource → Entity、Relationship 迁移细分项）。
> **每完成一个检查项，更新状态为 `[x]`；确认不适用的标记 `[-]`。**

---

## 行前准备（Day 1，所有 Agent 共用）

- [ ] Cargo.toml: `bevy = "0.18.1"` → `bevy = "0.19"`
- [ ] `cargo fix --edition --allow-no-vcs` 运行
- [ ] `cargo check 2>&1 | tee build_errors.log` 运行并保存
- [ ] `src/main.rs` 确认 `fn main() -> AppExit` 签名兼容
- [ ] `bevy-inspector-egui` 版本兼容检查
- [ ] `cargo build --features dev` 确认 dev 模式下编译

---

## Phase A：核心系统重写（第 1–2 周，4 Agent 并行）

### A1：Effect 系统 Delayed 化

| 文件/模式 | 变更 | 状态 |
|-----------|------|------|
| `src/shared/fre_delayed.rs` | 新建 FreDelayed<T> 包装层 | [ ] |
| `effect/components.rs` | EffectTimer → FreDelayed + EffectState | [ ] |
| `effect/lifecycle.rs` | Timer 轮询 System → Observer + Delayed 链 | [ ] |
| `effect/plugin.rs` | 注册 Observer 替代手动 System 编排 | [ ] |
| `effect/systems/dot_system.rs` | DOT 定时器 → Delayed Chain | [ ] |
| `effect/systems/buff_expiry.rs` | Buff 到期 → Delayed<RemoveBuff> | [ ] |
| `stacking/components.rs` | 堆叠规则适配 FreDelayed 生命周期 | [ ] |
| `runtime/scheduler.rs` | 调度 Timer → Delayed 命令 | [ ] |
| grep `timer.tick\|just_finished\|EffectTimer` | 确认全部消除 | [ ] |

### A2：Event → Observer 全面转换

| 事件类型 | 当前路径 | Observer 路径 | 状态 |
|----------|----------|---------------|------|
| 跨域事件 | core/events.rs → EventReader | core/events.rs → On<T> Observer | [ ] |
| DamageApplied | combat/systems/ → EventReader | combat/systems/ → Observer | [ ] |
| DamageDealt | combat/systems/ → EventWriter | combat/systems/ → trigger() | [ ] |
| TurnPhaseChanged | turn/systems/ | turn/systems/ → Observer | [ ] |
| TurnStarted | turn/systems/ | turn/systems/ → Observer | [ ] |
| TurnEnded | turn/systems/ | turn/systems/ → Observer | [ ] |
| AbilityActivated | ability/systems/ | ability/systems/ → Observer | [ ] |
| AbilityCast | ability/systems/ | ability/systems/ → trigger() | [ ] |
| SpellCast | spell/systems/ | spell/systems/ → Observer | [ ] |
| SpellResolved | spell/systems/ | spell/systems/ → Observer | [ ] |
| ReactionTriggered | reaction/systems/ | reaction/systems/ → Observer | [ ] |
| UnitDied | combat/systems/ | combat/systems/ → Observer | [ ] |
| LevelUp | progression/systems/ | progression/systems/ → Observer | [ ] |
| QuestProgress | quest/systems/ | quest/systems/ → Observer | [ ] |
| ItemAcquired | inventory/systems/ | inventory/systems/ → Observer | [ ] |
| GoldChanged | economy/systems/ | economy/systems/ → Observer | [ ] |
| BattleStarted | combat/systems/ | combat/systems/ → Observer | [ ] |
| BattleEnded | combat/systems/ | combat/systems/ → Observer | [ ] |
| 其他所有 `EventWriter<X>.send()` | grep 结果按模块逐个替换 | | [ ] |
| grep `EventReader\|EventWriter` 零残留 | cargo check 自动验证 | | [ ] |

### A3：Domain Observer + RunConditions

| 模块 | if 守卫位置 | RunConditions 替换 | 状态 |
|------|-------------|-------------------|------|
| combat/damage.rs | `if battle_state.is_in_battle()` | `run_if(resource_exists::<BattleState>)` | [ ] |
| combat/execution.rs | `if phase == Phase::Execution` | `run_if(resource_equals::<TurnPhase>(Phase::Execution))` | [ ] |
| combat/turn.rs | `if turn_state.is_player_turn()` | `run_if(resource_equals::<TurnPhase>(TurnPhase::Player))` | [ ] |
| tactical/movement.rs | `if !unit.is_moving()` | `run_if(resource_exists::<MovementInProgress>)` | [ ] |
| tactical/range.rs | `if range_check_enabled()` | `run_if(resource_exists::<RangeCheckActive>)` | [ ] |
| spell/casting.rs | `if spell_state.is_casting()` | `run_if(resource_equals::<SpellPhase>(SpellPhase::Casting))` | [ ] |
| spell/slots.rs | `if spell_state.has_slots()` | `run_if(resource_equals::<SpellPhase>(SpellPhase::Active))` | [ ] |
| reaction/oa.rs | `if reaction_window.is_open()` | `run_if(resource_exists::<ReactionWindow>)` | [ ] |
| reaction/counterspell.rs | `if reaction_window.is_open()` | `run_if(resource_exists::<ReactionWindow>)` | [ ] |
| progression/xp.rs | `if progression_enabled()` | `run_if(resource_exists::<ProgressionActive>)` | [ ] |
| inventory/use_item.rs | `if inventory_state.is_open()` | `run_if(resource_equals::<InventoryPhase>(InventoryPhase::Active))` | [ ] |
| party/formation.rs | `if party_state.can_swap()` | `run_if(resource_exists::<FormationSwapAllowed>)` | [ ] |
| 其他 `if.*state\|if.*phase\|if.*is_\|if.*in_battle` | grep 结果逐个替换 | | [ ] |

### A4：Infrastructure API 适配

| 文件 | 变更 | 状态 |
|------|------|------|
| `infra/input/input_system.rs` | `Res<Input<KeyCode>>` → `Res<ButtonInput<KeyCode>>` | [ ] |
| `infra/input/mod.rs` | 类型别名 `type FreInput = ButtonInput<KeyCode>` | [ ] |
| `infra/input/action_map.rs` | 输入动作映射适配 | [ ] |
| `infra/save/scene_serializer.rs` | `DynamicScene::from_world()` API 适配 | [ ] |
| `infra/save/save_file.rs` | Scene 序列化 API 适配 | [ ] |
| `infra/replay/recorder.rs` | Event 录制 → Observer 兼容 | [ ] |
| `infra/replay/player.rs` | Event 回放 → Observer 兼容 | [ ] |
| `infra/replay/events.rs` | Replay 事件定义适配 | [ ] |
| `app/app_plugin.rs` | Plugin 组合验证 | [ ] |
| `shared/shared_plugin.rs` | Plugin API 兼容 | [ ] |
| `core/core_plugin.rs` | Plugin API 兼容 | [ ] |
| `content/content_plugin.rs` | AssetLoader API 兼容 | [ ] |
| `tools/dev_tools_plugin.rs` | Plugin API 兼容 | [ ] |
| `modding/modding_plugin.rs` | Plugin API 兼容 | [ ] |

---

## Phase B：架构现代化（第 3–4 周，4 Agent 并行）

### B1：UI 层 BSN 化

| 文件 | 当前模式 | BSN 模式 | 状态 |
|------|----------|----------|------|
| `app/scenes/plugin.rs` | `.spawn(Node{...}).with_children(...)` | `bsn! { ... }` | [ ] |
| `app/scenes/components.rs` | UI 组件定义 | BSN 标签适配 | [ ] |
| `app/scenes/state.rs` | 场景状态管理 | BSN 场景切换 | [ ] |
| `app/scenes/queue.rs` | 场景队列 | BSN 场景定义 | [ ] |
| `app/scenes/tests/unit/scene_tests.rs` | 测试适配 | 测试适配 | [ ] |

### B2：Bundle → 工厂函数

| Bundle 位置 | 当前类型 | 新工厂函数 | 状态 |
|-------------|----------|-----------|------|
| core/capabilities/ability/ | `AbilityBundle` | `spawn_ability()` | [ ] |
| core/capabilities/effect/ | `EffectBundle` | `spawn_effect_entity()` | [ ] |
| core/capabilities/execution/ | `ExecutionBundle` | `spawn_execution_context()` | [ ] |
| core/capabilities/modifier/ | `ModifierBundle` | `spawn_modifier()` | [ ] |
| core/capabilities/cue/ | `CueBundle` | `spawn_cue()` | [ ] |
| core/domains/tactical/ | `UnitBundle` | `spawn_unit()` | [ ] |
| core/domains/tactical/ | `GridCellBundle` | `spawn_grid_cell()` | [ ] |
| core/domains/terrain/ | `TerrainTileBundle` | `spawn_terrain_tile()` | [ ] |
| core/domains/faction/ | `FactionRelationBundle` | `spawn_faction_marker()` | [ ] |
| core/domains/combat/ | `CombatEntityBundle` | `spawn_combat_entity()` | [ ] |
| core/domains/spell/ | `SpellEffectBundle` | `spawn_spell_effect()` | [ ] |
| core/domains/reaction/ | `ReactionWindowBundle` | `spawn_reaction_window()` | [ ] |
| core/domains/progression/ | `XpOrbBundle` | `spawn_xp_orb()` | [ ] |
| core/domains/inventory/ | `ItemDropBundle` | `spawn_item_drop()` | [ ] |
| core/domains/party/ | `PartyMemberBundle` | `spawn_party_member()` | [ ] |
| core/domains/narrative/ | `DialogOptionBundle` | `spawn_dialog_option()` | [ ] |
| core/domains/quest/ | `QuestMarkerBundle` | `spawn_quest_marker()` | [ ] |
| core/domains/summon/ | `SummonBundle` | `spawn_summon()` | [ ] |
| 其他 `#[derive(Bundle)]` | grep 结果逐个替换 | | [ ] |

### B3：Resource → Singleton Entity

| Resource | Marker 组件 | 初始化位置 | 状态 |
|----------|-------------|-----------|------|
| `BattleState` | `BattleRoot` | `app/plugin.rs` | [ ] |
| `TurnState` | `BattleRoot` | `app/plugin.rs` (与 BattleState 同 Entity) | [ ] |
| `GameTime` | `TimeRoot` | `shared/shared_plugin.rs` | [ ] |
| `InputState` | `InputRoot` | `infra/input/plugin.rs` | [ ] |
| `GameRng` | `RngRoot` | `shared/shared_plugin.rs` | [ ] |

| 使用点 | 旧模式 | 新模式 | 状态 |
|--------|--------|--------|------|
| `Res<BattleState>` | 全部 grep 结果 | `Single<&BattleState, With<BattleRoot>>` | [ ] |
| `ResMut<BattleState>` | 全部 grep 结果 | `Single<&mut BattleState, With<BattleRoot>>` | [ ] |
| `Res<TurnState>` | 全部 grep 结果 | `Single<&TurnState, With<BattleRoot>>` | [ ] |
| `ResMut<TurnState>` | 全部 grep 结果 | `Single<&mut TurnState, With<BattleRoot>>` | [ ] |
| `Res<GameTime>` | 全部 grep 结果 | `Single<&GameTime, With<TimeRoot>>` | [ ] |
| `Res<InputState>` | 全部 grep 结果 | `Single<&InputState, With<InputRoot>>` | [ ] |
| `Res<GameRng>` | 全部 grep 结果 | `Single<&GameRng, With<RngRoot>>` | [ ] |

### B4：Relationship 接入

| 关系 | 旧 Entity 字段 | 新 Relationship | 状态 |
|------|---------------|-----------------|------|
| Buff → Caster | `Buff.caster: Entity` | `Relationship<CasterOf>` on Buff entity | [ ] |
| Buff → Target | `Buff.target: Entity` | `Relationship<TargetOf>` on Buff entity | [ ] |
| Effect → Source | `Effect.source: Entity` | `Relationship<SourcedFrom>` on Effect entity | [ ] |
| Summon → Owner | `Summon.owner: Entity` | `Relationship<SummonedBy>` on Summon entity | [ ] |
| 其他 Entity 关系字段 | grep `Entity` 字段 + fragment 标记 | | [ ] |

---

## Phase C：收尾 + 性能（第 5 周）

### C1：User Settings

| 文件 | 变更 | 状态 |
|------|------|------|
| `app/settings_plugin.rs` | 新建 SettingsPlugin | [ ] |
| `app/settings_plugin.rs` | AudioSettings 定义 | [ ] |
| `app/settings_plugin.rs` | VideoSettings 定义 | [ ] |
| `app/settings_plugin.rs` | GameplaySettings 定义 | [ ] |
| `app/settings_plugin.rs` | `init_settings::<T>()` 注册 | [ ] |
| `app/app_plugin.rs` | Phase 10 注册 SettingsPlugin | [ ] |

### C2：Diagnostics Overlay

| 文件 | 变更 | 状态 |
|------|------|------|
| `tools/dev_tools_plugin.rs` | `#[cfg(feature = "dev")]` 下注册 DiagnosticsOverlayPlugin | [ ] |

### C3：font_size 替换

| 路径 | 变更 | 状态 |
|------|------|------|
| `grep -rn "font_size" src/` | `font_size: f32` → `TextFont { font_size: FontSize::Px(..) }` | [ ] |
| Text 相关所有文件 | TextSection/Text2d API 适配 | [ ] |

### C4：Contiguous Query

| 路径 | 变更 | 状态 |
|------|------|------|
| 批量查询热点 | `iter()` → `iter().contiguous_iter()` | [ ] |
| 只读查询 | 添加 `bypass_change_detection` | [ ] |
| 组件 Archetype 布局 | 经常一起查询的组件放在同 Archetype | [ ] |

### C5：Reflect 全覆盖

| 路径 | 变更 | 状态 |
|------|------|------|
| grep `#[derive(Component)]` 不含 `Reflect` | 添加 `Reflect` | [ ] |
| grep `#[derive(Event)]` 不含 `Reflect` | 添加 `Reflect` | [ ] |
| grep `#[derive(Resource)]` 不含 `Reflect` | 添加 `Reflect` | [ ] |
| grep `register_type` 缺失 | 补充 `app.register_type::<T>()` | [ ] |

---

## 文档更新（与 Phase C 并行）

| 文件 | 变更 | 状态 |
|------|------|------|
| `README.md`（项目根） | bevy 版本号 0.18.1 → 0.19 | [ ] |
| `docs/00-governance/ai-constitution-complete.md` | §1.1 引擎版本声明，新增 Observer/Delayed/Relationship 规则 | [ ] |
| `docs/01-architecture/README.md` | §4.2 通信机制更新（Observer 从可选→推荐→默认） | [ ] |
| `docs/01-architecture/README.md` | 新增 Bevy 0.19 迁移 ADR | [ ] |
| `docs/03-technical/bevy-0.19-migration/10-srpg-architecture-impact.md` | 补充实际迁移经验 | [ ] |
| `.trae/rules/编码规则.md` | 更新 EventReader → Observer 规则 | [ ] |
| `.trae/rules/ECS规则.md` | 新增 Delayed Commands / Observer / BSN 规则 | [ ] |

---

## 覆盖率统计

| 阶段 | Agent | 检查项数 | 文件影响数 |
|------|-------|---------|-----------|
| 行前准备 | 所有 | 6 | 3 |
| Phase A | A1 (Effect Delayed) | 9 | ~25 |
| Phase A | A2 (Event→Observer) | 20 | ~80 |
| Phase A | A3 (RunConditions) | 14 | ~50 |
| Phase A | A4 (Infra API) | 14 | ~55 |
| Phase B | B1 (UI BSN) | 5 | ~10 |
| Phase B | B2 (Bundle→Factory) | 19 | ~60 |
| Phase B | B3 (Resource→Entity) | 19 | ~30 |
| Phase B | B4 (Relationship) | 5 | ~20 |
| Phase C | C1 (User Settings) | 6 | 2 |
| Phase C | C2 (Diagnostics) | 1 | 1 |
| Phase C | C3 (font_size) | 2 | ~30 |
| Phase C | C4 (Contiguous) | 3 | ~20 |
| Phase C | C5 (Reflect) | 4 | ~80 |
| 文档更新 | — | 7 | 7 |
| **总计** | | **134** | **~300** |

---

> **维护者**: @feature-developer（执行跟踪） | **审查者**: @code-reviewer + @architect
> **最后更新**: 2026-06-19 | **版本**: v2.0 (激进版)
