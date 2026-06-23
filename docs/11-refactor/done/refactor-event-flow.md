# 重构计划：Picking 事件流重构（直接 Observer → PickIntent → Domain Event）

> 关联 ADR: ADR-068 §Communication Design | 当前状态: 参看 docs/01-architecture/ADR-067

## 目标状态 (Target)

事件流 4 层管道，每层职责明确：

```
Layer 1: Pointer Event（Bevy 原生）
  On<Pointer<Click>>, On<Pointer<Over>>, On<Pointer<Out>>
  │  命中检测由 Bevy picking backend 完成
  ▼

Layer 2: PickIntent（ui/picking/intent/）
  PickIntent { target: PickTarget, context: PickContext, phase: InteractionPhase }
  │  上下文注入点 — PickContext 在此层附加
  ▼

Layer 3: Domain Event（通过 ui/picking/domain_bridge/）
  UnitClicked, TileClicked, SelectionChanged
  │  Core Domain 消费这些事件
  ▼

Layer 4: Selection State Machine / Game Mode Handler（core/domains/）
  更新 SelectionState, 触发战斗 action 等
  │  业务逻辑在此层
  ▼

Output: 触发 CameraRequest / UiStore Projection
```

## 当前状态 (Current)

```
用户点击 Sprite
  │
  ▼
bevy_picking pipeline (PreUpdate)
  │  SpritePickingPlugin 检测命中
  │  InteractionPlugin → Pointer<Click>
  │
  ▼
Observer: on_unit_click (注册在 test_battle/render.rs)
  │  ✅ 写入 Selection Resource
  │  ❌ 直接写 Selection，没有中间层
  │  ❌ 使用 Entity 而非 BattleUnitId
  │  ❌ 没有 PickContext
  │  ❌ 没有 Preview/Commit 分离
  │  ❌ 在 render.rs 中定义（测试代码混入业务逻辑）
  │
  ├──→ Selection Resource (infra/picking/selection.rs)
  │       ❌ Option<Entity> 存储
  │
  ├──→ println! 调试输出
  │
  ▼
投影系统 (ui/projections/selection.rs)
  │  is_changed() 轮询检测
  │  ✅ 更新 UiStore
  │  ❌ 混合视觉高亮逻辑
  │  ❌ 混合 Camera 跟随
  │  ❌ 硬编码虚拟数据填充
  │
  ├──→ UiStore ViewModel
  ├──→ Sprite.color（直接修改）
  └──→ TargetPose（直接修改，绕过 CameraRequest）
```

## 差距分析 (Gap)

| # | 当前 | 目标 | 等级 |
|---|------|------|------|
| E1 | `On<Pointer<Click>>` 直接写 Selection | 经过 PickIntent → Domain Event → Selection | 🟥 |
| E2 | 使用 `selection.is_changed()` 轮询 | 使用 `SelectionChanged` 事件驱动 | 🟥 |
| E3 | Camera 跟随直接写 `TargetPose` | 通过 `trigger(CameraRequest::Follow)` | 🟥 |
| E4 | 没有 PickContext（全局模式） | PickContext 根据游戏状态注入 | 🟥 |
| E5 | Hover 和 Click 共用同一个 Selection | HoverState 独立 | 🟨 |
| E6 | Preview/Commit 不分离（hover 和 click handler 无区别） | Preview 无副作用，Commit 触发状态变更 | 🟨 |
| E7 | `println!` 调试输出 | 无调试输出或通过 LogObserver | 🟩 |
| E8 | per-entity observer 注册 | 全局 observer + Query 过滤 | 🟨 |

## 迁移步骤

### 步骤 1: 创建 PickIntent 类型和事件管道

新建 `src/ui/picking/foundation/pick_intent.rs`:

```rust
use crate::core::domains::combat::components::BattleUnitId;
use crate::core::domains::tactical::components::GridPos;

/// PickIntent — Picking 的输出事件，Selection 的输入。
/// Picking 只能产出此事件，不能直接修改游戏状态。
#[derive(Event, Debug, Clone)]
pub struct PickIntent {
    /// 拾取目标
    pub target: PickTarget,
    /// 鼠标按钮
    pub button: PointerButton,
    /// 当前上下文
    pub context: PickContext,
    /// 交互阶段
    pub phase: InteractionPhase,
}

/// PickTarget — Picking 识别出的目标。
/// 使用领域 ID 而非 Entity。
#[derive(Debug, Clone, PartialEq)]
pub enum PickTarget {
    /// 单位（使用 BattleUnitId）
    Unit(BattleUnitId),
    /// 网格（使用 GridPos）
    Tile(GridPos),
    /// 非单位实体（仍然使用 Entity，但需要额外校验）
    Entity(Entity),
    /// UI 元素（由 Interaction 系统处理）
    UI(Entity),
    /// 空点击（右键点击空白处）
    None,
}

/// InteractionPhase — 交互阶段。
#[derive(Debug, Clone, PartialEq)]
pub enum InteractionPhase {
    Preview,  // 悬停预览，不允许副作用
    Commit,   // 点击确认，可触发状态变更
}

/// PickIntentReceived — 内部 trigger 事件（在 ui/picking 模块内消费）
#[derive(Event, Debug, Clone)]
pub struct PickIntentReceived(pub PickIntent);
```

### 步骤 2: 创建 Pointer Click → PickIntent 转换

新建 `src/ui/picking/intent/click_intent.rs`:

```rust
use bevy::prelude::*;
use bevy::input::pointer::{PointerButton, PointerClick};
use crate::core::domains::combat::components::UnitIdComponent;
use crate::ui::picking::foundation::pick_intent::*;
use crate::ui::picking::context::PickContext;

/// 全局 Pointer<Click> 观察者。
/// 将 Bevy 原生点击事件转换为 PickIntent。
/// 不依赖 per-entity observer。
pub fn on_pointer_click(
    ev: On<Pointer<Click>>,
    unit_query: Query<&UnitIdComponent>,
    context: Res<PickContext>,
    mut intent_events: EventWriter<PickIntentReceived>,
) {
    let target_entity = ev.event_target();
    let button = ev.event().button;
    
    // 右键 → 取消选择
    if button == PointerButton::Secondary {
        intent_events.send(PickIntentReceived(PickIntent {
            target: PickTarget::None,
            button,
            context: context.clone(),
            phase: InteractionPhase::Commit,
        }));
        return;
    }
    
    // 尝试识别为单位
    if let Ok(uid) = unit_query.get(target_entity) {
        intent_events.send(PickIntentReceived(PickIntent {
            target: PickTarget::Unit(uid.id.clone()),  // BattleUnitId
            button,
            context: context.clone(),
            phase: InteractionPhase::Commit,
        }));
        return;
    }
    
    // 其他 Entity 类型的命中
    intent_events.send(PickIntentReceived(PickIntent {
        target: PickTarget::Entity(target_entity),
        button,
        context: context.clone(),
        phase: InteractionPhase::Commit,
    }));
}
```

### 步骤 3: 创建 PickIntent → Domain Event 桥接

新建 `src/ui/picking/domain_bridge/unit_bridge.rs`:

```rust
use bevy::prelude::*;
use crate::core::domains::combat::events::UnitClicked;
use crate::core::domains::tactical::events::TileClicked;
use crate::ui::picking::foundation::pick_intent::*;

/// 消费 PickIntent，转换为领域事件。
/// 只有当 phase == Commit 时才触发领域事件。
pub fn bridge_pick_intent_to_domain(
    ev: On<PickIntentReceived>,
    mut commands: Commands,
) {
    let intent = ev.0;
    
    if intent.phase != InteractionPhase::Commit {
        return;  // Preview 阶段不触发领域事件
    }
    
    match intent.target {
        PickTarget::Unit(unit_id) => {
            commands.trigger(UnitClicked {
                unit_id,
                button: intent.button,
                context: intent.context,
            });
        }
        PickTarget::Tile(tile_pos) => {
            commands.trigger(TileClicked {
                tile: tile_pos,
                button: intent.button,
            });
        }
        PickTarget::None => {
            // 空点击 → 触发取消选择
            commands.trigger(UnitClicked {
                unit_id: None,
                button: intent.button,
                context: intent.context,
            });
        }
        _ => { /* Entity/UI 暂不处理 */ }
    }
}
```

### 步骤 4: 创建 PickIntent → Camera 桥接

新建 `src/ui/picking/domain_bridge/camera_bridge.rs`:

```rust
use bevy::prelude::*;
use crate::core::domains::combat::events::UnitClicked;
use crate::infra::camera::foundation::request::CameraRequest;
use crate::infra::camera::foundation::target::CameraTarget;

/// 消费 UnitClicked 领域事件，触发 Camera 跟随。
/// 完全通过 CameraRequest 通信，不再直接写 TargetPose。
pub fn on_unit_clicked_camera_follow(
    ev: On<UnitClicked>,
    mut commands: Commands,
) {
    if let Some(unit_id) = ev.unit_id {
        commands.trigger(CameraRequest::Follow {
            target: CameraTarget::UnitId(unit_id),
        });
    }
}
```

### 步骤 5: 创建 SelectionChanged 事件

在 `src/ui/picking/foundation/selection_state.rs` 中定义：

```rust
/// SelectionChanged — 当选择状态变更时触发。
/// 由 Selection 状态机产出，Projection 系统消费。
#[derive(Event, Debug, Clone)]
pub struct SelectionChanged {
    pub previous: Option<BattleUnitId>,
    pub current: Option<BattleUnitId>,
    pub previous_phase: SelectionPhase,
    pub current_phase: SelectionPhase,
}
```

### 步骤 6: 重构 Projection 系统

修改 `src/ui/projections/selection.rs`：

```rust
/// 新的投影系统 — 由 SelectionChanged 事件驱动，不再轮询。
pub fn on_selection_changed_projection(
    ev: On<SelectionChanged>,
    mut store: ResMut<UiStore>,
    unit_query: Query<(&UnitIdComponent, &HitPoints)>,
    mut panel_dirty: Query<&mut Dirty<CharacterPanelVm>>,
    mut hud_dirty: Query<&mut Dirty<BattleHudVm>>,
) {
    let Some(current) = ev.current else {
        // 取消选中 → 清空 ViewModel
        reset_viewmodel(&mut store);
        mark_all_dirty(&mut panel_dirty, &mut hud_dirty);
        return;
    };
    
    // 通过 BattleUnitId 查询 ECS 数据（entity 解析需通过 Domain 的 integration/ 层）
    // ...
}
```

### 步骤 7: 统一事件注册

在 `src/ui/picking/plugin.rs` 中注册事件管道链:

```rust
impl Plugin for PickingUiPlugin {
    fn build(&self, app: &mut App) {
        // ... backend 配置
        
        // Event 注册
        app.add_event::<PickIntentReceived>();
        app.add_event::<SelectionChanged>();
        
        // Observer 注册（按执行顺序）
        // 1. Pointer Event → PickIntent
        app.add_observer(on_pointer_click);
        app.add_observer(on_pointer_over);
        app.add_observer(on_pointer_out);
        
        // 2. PickIntent → Domain Event
        app.add_observer(bridge_pick_intent_to_domain);
        
        // 3. Domain Event → Camera
        app.add_observer(on_unit_clicked_camera_follow);
        
        // 4. Domain Event → Selection State
        app.add_observer(handle_unit_selected);
        
        // ... resources
    }
}
```

### 步骤 8: 删除旧事件流代码

- 删除 `src/app/scenes/test_battle/render.rs` 中的 `on_unit_click`, `on_unit_hover`, `on_unit_unhover`
- 删除 `src/ui/projections/selection.rs` 中的 Camera 跟随代码
- 删除 `src/ui/projections/selection.rs` 中的 `on_selection_visual` 函数
- 删除 `camera_follow_selection` 函数（由 bridge 替代）

## 风险与缓解

| # | 风险 | 影响 | 缓解 |
|---|------|------|------|
| R1 | 事件管道延迟导致选中响应慢 | 用户体验下降 | Observer 在 PreUpdate 执行（与 picking 同一帧），理论延迟为零 |
| R2 | PickIntent 事件被多个 Observer 竞争消费 | 重复选中 | 每个事件只有一个消费者（domain_bridge 路由后其他系统不应再消费） |
| R3 | CameraRequest 被当前 Focus 状态忽略 | 镜头不跟随 | CameraRequest::Follow 在任何状态下都应处理（除 Focus 外）— 需要修改 camera state_machine 以支持 |
| R4 | Entity→BattleUnitId 查询失败 | 点击无响应 | 保留 Entity 作为 fallback 但在日志中警告 |

## 事件流对照表

| 场景 | 旧流程 | 新流程 |
|------|--------|--------|
| 左键点击单位 | Pointer<Click> → on_unit_click → Selection | Pointer<Click> → PickIntent(Unit) → UnitClicked → SelectionState |
| 右键点击 | → on_unit_click(右键) → Selection=None | → PickIntent(None, Commit) → UnitClicked(unit_id:None) → SelectionState=None |
| 悬停高亮 | Pointer<Over> → on_unit_hover → sprite.color | Pointer<Over> → hover_intent(Preview) → sprite.color (直接) |
| 取消高亮 | Pointer<Out> → on_unit_unhover → sprite.color | Pointer<Out> → hover_cleanup → sprite.color |
| Camera 跟随 | selection.is_changed() → camera_follow_selection → TargetPose | SelectionChanged → camera_bridge → CameraRequest::Follow |

## 预估工作量

| 步骤 | 文件 | 行数 | 时间 |
|------|------|------|------|
| 1. PickIntent 类型 | 新建 2 个文件 | ~80 行 | 0.5h |
| 2. Click→Intent 转换 | 新建 1 个文件 | ~60 行 | 0.5h |
| 3. Intent→Domain 桥接 | 新建 2 个文件 | ~50 行 | 0.5h |
| 4. Intent→Camera 桥接 | 新建 1 个文件 | ~30 行 | 0.3h |
| 5. SelectionChanged 事件 | 修改 1 个文件 | ~20 行 | 0.2h |
| 6. 重写 Projection | 修改 1 个文件 | ~100 行 | 1.5h |
| 7. 注册事件管道 | 修改 1 个文件 | ~30 行 | 0.3h |
| 8. 删除旧代码 | 修改 2 个文件 | -70 行 | 0.5h |
| **合计** | | **~300 行** | **~4.3h** |
