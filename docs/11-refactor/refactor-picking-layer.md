# 重构计划：Picking 分层迁移（infra/picking/ → ui/picking/）

> 关联 ADR: ADR-PICK-000 | 当前 ADR-067 将被标记为 Superseded

## 目标状态 (Target)

- `src/infra/picking/` 被彻底删除
- `src/ui/picking/` 作为 Presentation Layer 的独立输入适配子层
- `src/ui/picking/` 目录结构遵循 ADR-PICK-000 的 Module Design
- `src/app/app_plugin.rs` 中 PickingPlugin 的注册从 Phase 8 迁移到 Phase 11（与 UiPlugin 一起）
- `PickingSettings` / `SpritePickingSettings` 配置归一化到 `ui/picking/backend/sprite_backend.rs`
- `Pickable::IGNORE` 策略从 `render.rs` 中移出到独立的 `ui_passthrough.rs`

## 当前状态 (Current)

- `src/infra/picking/mod.rs` — 2 行模块声明
- `src/infra/picking/plugin.rs` — 57 行，包含 PickingPlugin + 调试观察者
- `src/infra/picking/selection.rs` — 11 行，Selection Resource 定义
- `src/app/scenes/test_battle/render.rs` — 247 行，包含 on_unit_click/hover/unhover + Pickable 附着
- `src/ui/projections/selection.rs` — 215 行，混合了 Projection + 视觉高亮 + Camera 跟随
- `src/infra/mod.rs` — 包含 `pub mod picking;`
- Plugin 注册在 Phase 8 Infra

## 差距分析 (Gap)

| # | 当前 | 目标 | 差距等级 |
|---|------|------|---------|
| G1 | Picking 在 infra/ | Picking 在 ui/ | 🟥 架构违规 |
| G2 | Selection 定义在 infra/picking/selection.rs | Selection 定义在 ui/picking/foundation/ 或 Core Domain | 🟥 层级错误 |
| G3 | Plugin 注册在 Phase 8 | Plugin 注册在 Phase 11 | 🟥 时序错误 |
| G4 | on_unit_click 等在 test_battle/render.rs | 在 ui/picking/battle/click_handler.rs | 🟥 测试代码混入业务逻辑 |
| G5 | render.rs 同时做 spawn + picking handler | 职责分离，render.rs 只做视觉效果 | 🟨 违反单一职责 |
| G6 | 调试打印散落各处 | 无调试打印或统一通过 LogObserver | 🟨 代码质量 |
| G7 | `infra/mod.rs` 引用 picking | infra/mod.rs 不再引用 picking | 🟩 简单删除 |
| G8 | 无 picking 模块测试 | picking 各子模块有单元测试 | 🟨 测试覆盖 |

## 迁移步骤 (Migration Steps)

### 步骤 1: 创建目标目录结构

创建 `src/ui/picking/` 目录结构及所有模块文件（骨架 + 核心类型定义）：

**操作清单**:

1.1 创建目录:
```bash
mkdir -p src/ui/picking/backend
mkdir -p src/ui/picking/foundation
mkdir -p src/ui/picking/intent
mkdir -p src/ui/picking/selection
mkdir -p src/ui/picking/domain_bridge
mkdir -p src/ui/picking/context
mkdir -p src/ui/picking/battle
mkdir -p src/ui/picking/world
```

1.2 创建 `src/ui/picking/mod.rs`:
```rust
pub mod backend;
pub mod foundation;
pub mod intent;
pub mod selection;
pub mod domain_bridge;
pub mod context;
pub mod battle;
pub mod world;

pub use plugin::PickingUiPlugin;
```

1.3 创建 `src/ui/picking/foundation/pick_target.rs`:
- 定义 `PickTarget` 枚举（Unit, Tile, Entity, UI, None）
- 定义 `PickTarget::unit_id()` 等辅助方法

1.4 创建 `src/ui/picking/foundation/pick_intent.rs`:
- 定义 `PickIntent` 结构体
- 定义 `InteractionPhase` 枚举

1.5 创建 `src/ui/picking/foundation/pick_context.rs`:
- 定义 `PickContext` 枚举（Normal, AttackTargeting, SkillTargeting, MoveSelection, ItemTargeting）

1.6 创建 `src/ui/picking/foundation/selection_state.rs`:
- 定义 `SelectionState` Resource
- `selected_unit: Option<BattleUnitId>`（迁移自 infra 的 Selection）
- `phase: SelectionPhase`（Idle, UnitSelected, ActionSelected, Targeting）

1.7 创建 `src/ui/picking/foundation/hover_state.rs`:
- 定义 `HoverState` Resource（新增）
- `hovered_unit: Option<BattleUnitId>`

**涉及文件**:
- 新建: `src/ui/picking/mod.rs`
- 新建: `src/ui/picking/foundation/mod.rs`
- 新建: `src/ui/picking/foundation/pick_target.rs`
- 新建: `src/ui/picking/foundation/pick_intent.rs`
- 新建: `src/ui/picking/foundation/pick_context.rs`
- 新建: `src/ui/picking/foundation/selection_state.rs`
- 新建: `src/ui/picking/foundation/hover_state.rs`

### 步骤 2: 迁移 Core 类型到 foundation/

从 `src/infra/picking/selection.rs` 和 `src/ui/projections/selection.rs` 提取核心类型定义：

2.1 删除 `src/infra/picking/selection.rs` (将 Selection 升级为 SelectionState)
2.2 清理 `src/infra/picking/mod.rs` (移除 selection pub mod)

**涉及文件**:
- 修改: `src/infra/picking/mod.rs`（删除 `pub mod selection;`）
- 删除: `src/infra/picking/selection.rs`（内容已迁移到 ui/picking/foundation/）
- 修改: `src/ui/picking/foundation/selection_state.rs`（最终确认）

### 步骤 3: 迁移 Backend 配置

将 `PickingPlugin` 从 `src/infra/picking/plugin.rs` 迁移到 `src/ui/picking/backend/sprite_backend.rs`：

3.1 创建 `src/ui/picking/backend/sprite_backend.rs`:
```rust
// 配置 SpritePickingPlugin + SpritePickingMode::BoundingBox
// 配置 PickingSettings
// 不含任何 observer 注册（observer 在 intent/ 中）
```

3.2 创建 `src/ui/picking/backend/ui_passthrough.rs`:
```rust
// 定义 UI 穿透策略
// mark_battle_ui_passthrough system（将 Pickable::IGNORE 应用到 UI 根节点）
```

3.3 创建 `src/ui/picking/plugin.rs`:
```rust
pub struct PickingUiPlugin;

impl Plugin for PickingUiPlugin {
    fn build(&self, app: &mut App) {
        // 1. 注册 foundation 类型
        app.init_resource::<SelectionState>();
        app.init_resource::<HoverState>();
        app.register_type::<SelectionState>();
        
        // 2. 配置 backend
        app.add_plugins(SpritePickingBackendPlugin);
        app.add_systems(Startup, apply_ui_passthrough);
        
        // 3. 注册 intent observers
        // 4. 注册 selection state machine
        // 5. 注册 domain bridge
    }
}
```

**涉及文件**:
- 新建: `src/ui/picking/backend/sprite_backend.rs`
- 新建: `src/ui/picking/backend/ui_passthrough.rs`
- 新建: `src/ui/picking/backend/mod.rs`
- 新建: `src/ui/picking/plugin.rs`
- 修改: `src/ui/picking/mod.rs`（添加 pub mod plugin 等）

### 步骤 4: 迁移 Intent 处理

将 `on_unit_click`、`on_unit_hover`、`on_unit_unhover` 从 `render.rs` 迁移到 `ui/picking/`：

4.1 创建 `src/ui/picking/intent/click_intent.rs`:
```rust
// Observer: On<Pointer<Click>> → 读取 PickContext → 创建 PickIntent
// 左键: PickIntent { phase: Commit, context: current }
// 右键: PickIntent { phase: Commit, target: None }（取消选择）
pub fn on_pointer_click(
    ev: On<Pointer<Click>>,
    unit_query: Query<&UnitIdComponent>,
    context: Res<PickContext>,
    mut intent_writer: EventWriter<PickIntentReceived>,
) {
    // 1. 获取 event_target()
    // 2. 查询 UnitIdComponent 获取 BattleUnitId
    // 3. 读取当前 PickContext
    // 4. 生成 PickIntentReceived(PickIntent)
    // 5. ✅ 不再直接写 Selection
}
```

4.2 创建 `src/ui/picking/intent/hover_intent.rs`:
```rust
// Observer: On<Pointer<Over>> → 更新 Sprite.color（高亮）
// Observer: On<Pointer<Out>> → 恢复 Sprite.color
// ✅ 仅修改视觉表现，不修改 Selection
pub fn on_unit_hover_visual(
    ev: On<Pointer<Over>>,
    mut sprites: Query<&mut Sprite>,
) { /* sprite.color = HIGHLIGHT_COLOR */ }

pub fn on_unit_unhover_visual(
    ev: On<Pointer<Out>>,
    mut sprites: Query<&mut Sprite>,
    participants: Query<&CombatParticipant>,
    selection: Res<SelectionState>,
) { /* 根据队伍色 + 选中状态恢复颜色 */ }
```

4.3 创建 `src/ui/picking/intent/intent_router.rs`:
```rust
// 消费 PickIntentReceived，根据 PickContext 路由
// 如果 context == Normal → 触发 SelectionState 更新
// 如果 context == AttackTargeting → 触发 CombatAction 请求
pub fn route_pick_intent(
    ev: On<PickIntentReceived>,
    mut selection: ResMut<SelectionState>,
) {
    let intent = ev.0;
    match intent.context {
        PickContext::Normal => {
            // 通知 SelectionState
        }
        PickContext::AttackTargeting { .. } => {
            // 触发攻击目标确认
        }
        // ...
    }
}
```

4.4 修改 `src/app/scenes/test_battle/render.rs`：
- 删除 `on_unit_click`, `on_unit_hover`, `on_unit_unhover` 函数
- 在 `attach_unit_visuals` 中保留 `.observe(on_unit_click)` 吗？**不** — observer 应该在 intent 层全局注册而非 per-entity
- 保留 Pickable::default() 附着
- 删除调试 println

**涉及文件**:
- 新建: `src/ui/picking/intent/click_intent.rs`
- 新建: `src/ui/picking/intent/hover_intent.rs`
- 新建: `src/ui/picking/intent/intent_router.rs`
- 新建: `src/ui/picking/intent/mod.rs`
- 修改: `src/app/scenes/test_battle/render.rs`（删除 picking handlers）

### 步骤 5: 迁移 Selection 状态管理

将 Selection 状态机从 `src/ui/projections/selection.rs` 迁移到 `ui/picking/selection/`：

5.1 创建 `src/ui/picking/selection/selection_manager.rs`:
```rust
// 消费 PickIntent 或 Domain Event，更新 SelectionState
// 处理选中/取消/目标确认等状态转换
pub fn handle_unit_selected(
    intent: On<PickIntentReceived>,
    mut selection: ResMut<SelectionState>,
    mut sel_changed: EventWriter<SelectionChanged>,
) {
    // 只有 phase == Commit 且 target == Unit 时更新
    // 更新 SelectionState
    // 触发 SelectionChanged 事件（驱动 Projection）
}
```

5.2 创建 `src/ui/picking/selection/focus_manager.rs`:
```rust
// Focus 状态管理（跨 Tab/Click 保持）
```

5.3 创建 `src/ui/picking/selection/highlight_system.rs`:
```rust
// 从 render.rs 和 projections/selection.rs 中提取视觉高亮逻辑
// 读取 SelectionState + HoverState → 更新 Sprite.color
// 这是唯一合法的"直接修改 Sprite.color"的行为
pub fn apply_selection_highlight(
    selection: Res<SelectionState>,
    hover: Res<HoverState>,
    mut sprites: Query<&mut Sprite>,
    unit_query: Query<&UnitIdComponent>,
) { /* ... */ }
```

**涉及文件**:
- 新建: `src/ui/picking/selection/selection_manager.rs`
- 新建: `src/ui/picking/selection/focus_manager.rs`
- 新建: `src/ui/picking/selection/highlight_system.rs`
- 新建: `src/ui/picking/selection/mod.rs`

### 步骤 6: 重构 projections/selection.rs

当前 `src/ui/projections/selection.rs` 是重构的核心目标，它同时承担了 4 个不同的职责，必须拆分：

6.1 将 **ViewModel 投影** 保留在 `projections/selection.rs` 中并重写：
- 删除 visual highlight（已迁移到 highlight_system.rs）
- 删除 camera follow（已迁移到 events/CameraRequest）
- 删除 ActionMenu 和 SkillPanel 的硬编码数据填充（这些应该由 Domain 数据驱动）
- 保留 `on_selection_changed` → UiStore 更新的核心投影逻辑
- 使用 `SelectionChanged` 事件触发，而非 `selection.is_changed()`

6.2 将 **Camera 跟随** 迁移到独立的 Observer：
- 新建 observer 消费 `SelectionChanged` 事件
- 通过 `commands.trigger(CameraRequest::Follow { target: CameraTarget::UnitId(id) })` 实现
- 不再直接写 `TargetPose`

6.3 修正 ViewModel 数据填充：
- `character_id` 使用 `BattleUnitId` 而非 `entity.to_bits() as u32`
- `name_key` 使用 `UnitIdComponent.id` 而非硬编码
- 删除硬编码的技能数据（skills HashMap）

**涉及文件**:
- 修改: `src/ui/projections/selection.rs`（大幅重写）
- 新建: `src/ui/picking/domain_bridge/unit_bridge.rs`
- 新建: `src/ui/picking/domain_bridge/camera_bridge.rs`（触发 CameraRequest）

### 步骤 7: 清理 src/infra/picking/

7.1 删除 `src/infra/picking/` 目录
7.2 修改 `src/infra/mod.rs` 删除 `pub mod picking;`
7.3 修改 `src/app/app_plugin.rs` 注册顺序（Phase 8 移除 PickingPlugin）

**涉及文件**:
- 删除: `src/infra/picking/mod.rs`
- 删除: `src/infra/picking/plugin.rs`
- 删除: `src/infra/picking/selection.rs`（如果步骤 2 未删除）
- 删除: `src/infra/picking/` 整个目录
- 修改: `src/infra/mod.rs`
- 修改: `src/app/app_plugin.rs`

### 步骤 8: 注册 PickingUiPlugin

8.1 在 `app_plugin.rs` 的 Phase 11（UiPlugin 之前或作为 UiPlugin 子模块）注册 `PickingUiPlugin`:
```rust
// Phase 11: UI Presentation Layer (L3)
.add_plugins(ui::UiPlugin)  // UiPlugin 内部包含 PickingUiPlugin
```

8.2 或者在 `src/ui/mod.rs`:
```rust
// ui::UiPlugin 内部注册 picking 子插件
app.add_plugins(picking::PickingUiPlugin);
```

**涉及文件**:
- 修改: `src/app/app_plugin.rs`
- 修改: `src/ui/mod.rs` 或 `src/ui/plugin.rs`
- 修改: `docs/01-architecture/README.md`（更新 ADR 索引和 Phase 描述）

### 步骤 9: 更新引用

搜索所有引用 `infra::picking::` 的代码，更新为 `ui::picking::`：

9.1 修改 `src/ui/projections/selection.rs` 的 import:
```rust
// 旧: use crate::infra::picking::selection::Selection;
// 新: use crate::ui::picking::foundation::selection_state::SelectionState;
```

9.2 修改 `src/app/scenes/test_battle/render.rs` 的 import:
```rust
// 旧: use crate::infra::picking::selection::Selection;
// 新: ✓ 不再需要 import（Selection 访问通过 Domain Event）
```

**涉及文件**:
- `src/app/scenes/test_battle/render.rs`
- `src/ui/projections/selection.rs`

### 步骤 10: 测试验证

10.1 确保 `cargo build` 通过
10.2 确保 `cargo clippy -- -D warnings` 通过
10.3 确保 `cargo nextest run` 通过
10.4 手动测试：点击单位 → 选中 → Hover → Camera 跟随

## 风险与缓解 (Risks)

| # | 风险 | 影响 | 缓解措施 |
|---|------|------|---------|
| R1 | PickingUiPlugin 注册时序问题 | UI 功能不可用 | 在 UiPlugin 中注册，确保其后 UiStore 已初始化 |
| R2 | Entity→BattleUnitId 转换断裂 | 点击单位不选中 | 保留 UnitIdComponent Query 作为旧接口，逐步迁移 |
| R3 | Camera 跟随缺失 | 选中单位后镜头不动 | 新 Camera bridge 在 SelectionChanged 事件中立即触发 Follow |
| R4 | Hover 高亮消失 | 悬停无反馈 | hover_intent.rs 中的 On<Pointer<Over>> 全局注册，不依赖 per-entity observer |
| R5 | Observer 重复触发 | 点击一次触发多次 | 确保 On<Pointer<Click>> observer 全局唯一，不在 per-entity 重复注册 |

## 预估工作量 (Effort)

| 步骤 | 文件操作 | 新增代码 | 删除代码 | 修改代码 | 预估小时 |
|------|---------|---------|---------|---------|---------|
| 1. 创建目录结构 | 新建 ~15 个文件 | ~300 行 | 0 | 0 | 1h |
| 2. 迁移 Core 类型 | 新建 3 个, 删除 1 个 | ~80 行 | ~11 行 | ~5 行 | 0.5h |
| 3. 迁移 Backend 配置 | 新建 3 个, 删除 1 个 | ~80 行 | ~57 行 | 0 | 0.5h |
| 4. 迁移 Intent 处理 | 新建 4 个, 修改 1 个 | ~150 行 | ~80 行 | ~30 行 | 1.5h |
| 5. 迁移 Selection 管理 | 新建 4 个 | ~200 行 | 0 | 0 | 1.5h |
| 6. 重构 projections | 修改 1 个, 新建 2 个 | ~100 行 | ~150 行 | ~150 行 | 2h |
| 7. 清理 infra/picking | 删除 3 个, 修改 2 个 | 0 | ~70 行 | ~10 行 | 0.5h |
| 8. 注册 PickingUiPlugin | 修改 2-3 个 | ~20 行 | 0 | ~15 行 | 0.5h |
| 9. 更新引用 | 修改 2-3 个 | 0 | 0 | ~10 行 | 0.5h |
| 10. 测试验证 | 0 | 0 | 0 | 0 | 1h |
| **合计** | | **~930 行** | **~370 行** | **~220 行** | **~9.5h** |

> 注意：以上估算不包含测试代码编写，测试编写由 @test-guardian 负责。
