---
id: 11-refactor.picking-refactor-plan
title: Picking 架构重构行动计划
status: active
created: 2026-06-23
owner: architect
---

# Picking 架构重构行动计划

> 关联 ADR: ADR-068 (Picking 架构总纲) | ADR-067 (已被取代) | 宪法 §1.5
> 前置文档: docs/02-domain/domains/tactical_domain.md §11, docs/04-data/infrastructure/picking_schema.md, docs/04-data/domains/tactical_schema.md §6, docs/06-ui/01-architecture/architecture.md §3.7

## 总览

| Phase | 名称 | 工时 | 核心产出 |
|-------|------|------|---------|
| 0 | 宪法确认 | 0h ✅ 已全部完成 | ADR-068 + 领域规则 + Schema + UI 架构 |
| 1 | 目录迁移 | ~3h | `ui/picking/` + `ui/selection/` + 领域事件 |
| 2 | 事件流管道 | ~3h | Pointer → PickIntent → Domain Event |
| 3 | Selection 重构 | ~4h | SelectionState 五态 + BattleUnitId + CameraRequest |
| 4 | 清理调试代码 | ~1h | 删除 println! / debug_observer |
| 5 | Bevy 兼容性 | ~2h | ViewVisibility + UI 穿透验证 |
| 6 | 验证 | ~2h | build + clippy + test + 手动测试 |
| 7 | 文档归档 | ~0.5h | ADR 状态更新 + 11-refactor README |

**总计**: ~15.5h

---

## Phase 1：目录迁移（~3h）

### Step 1: 创建目录结构

```
src/ui/picking/
├── mod.rs
├── plugin.rs          # PickingUiPlugin
├── backend/
│   ├── mod.rs
│   ├── sprite.rs      # SpritePickingPlugin 配置
│   └── passthrough.rs # UI 穿透策略
└── intent/
    ├── mod.rs
    ├── click.rs       # On<Pointer<Click>> → PickIntent
    └── hover.rs       # On<Pointer<Over|Out>> → PickIntent

src/ui/selection/
├── mod.rs
├── state.rs           # SelectionState 五态
├── pick_context.rs     # PickContext Resource
└── bridge.rs          # PickIntent → Domain Event
```

### Step 2: 定义 BattleUnitId 在 `src/shared/ids/`

- 新增文件: `src/shared/ids/battle_unit_id.rs`
- 或添加到 `src/shared/ids/types/runtime_id.rs`

### Step 3: 定义领域事件在 `src/core/domains/tactical/`

```
src/core/domains/tactical/events.rs
- UnitClicked
- TileClicked
- TargetConfirmed
- SelectionChanged + SelectionPhase
```

### Step 4: 迁移 infra/picking/ 配置到 ui/picking/backend/

- `SpritePickingSettings` + `PickingSettings` → `ui/picking/backend/sprite.rs`
- `Pickable::IGNORE` 策略 → `ui/picking/backend/passthrough.rs`
- `Selection` Resource 删除 → 替换为 `SelectionState`

### Step 5: 创建 ui/picking/intent/

- `click.rs`: `On<Pointer<Click>>` → 构造 `PickIntent`，写入 `PickContext`
- `hover.rs`: `On<Pointer<Over>>` / `On<Pointer<Out>>` → 构造 `PickIntent::Preview`

### Step 6: 创建 ui/selection/

- `state.rs`: `SelectionState` Resource + 状态转移方法
- `pick_context.rs`: `PickContext` enum + Resource
- `bridge.rs`: 消费 `PickIntent` → `trigger(UnitClicked)` / `trigger(TileClicked)`

### Step 7: 更新 ui/plugin.rs

从 7 步扩展到 9 步（PickingUiPlugin step 7 + SelectionPlugin step 8）

### Step 8: 删除 src/infra/picking/

删除旧目录，更新 `src/infra/mod.rs`

### Step 9: 更新 app_plugin.rs

将 `infra::picking::PickingPlugin` 替换为 `ui::picking::PickingUiPlugin` + `ui::selection::SelectionPlugin`

### 验证: `cargo check`

---

## Phase 2：事件流管道（~3h）

### Step 1-2: PickTarget + PickContext 类型（含在 Phase 1）

### Step 3: click.rs observer

```rust
fn on_pointer_click(ev: On<Pointer<Click>>, q: Query<&UnitIdComponent>, ctx: Res<PickContext>) {
    if ev.button != Primary { return; }
    let target = PickTarget::Unit(q.get(ev.target()).id.clone());
    commands.trigger(PickIntent { target, phase: Commit, context: ctx.0 });
}
```

### Step 4: hover.rs observer

```rust
fn on_pointer_over(ev: On<Pointer<Over>>, ...) {
    commands.trigger(PickIntent { ..., phase: Preview });
}
fn on_pointer_out(ev: On<Pointer<Out>>, ...) {
    commands.trigger(PickIntent { ..., phase: PreviewEnd });
}
```

### Step 5: bridge.rs — PickIntent → Domain Event

```rust
fn on_pick_intent(ev: On<PickIntent>, ...) {
    match (ev.target, ev.context) {
        (Unit(id), Normal) => commands.trigger(UnitClicked { unit_id: id, .. }),
        (Unit(id), AttackTargeting) => commands.trigger(UnitClicked { .. }),
        (Tile(pos), _) => commands.trigger(TileClicked { tile: pos, .. }),
        (Empty, _) => { /* 清除选择 */ }
    }
}
```

### Step 6: SelectionState 状态机

```rust
impl SelectionState {
    fn on_unit_clicked(&mut self, event: &UnitClicked) -> Option<SelectionChanged> {
        match self.context {
            Normal => { self.selected = Some(event.unit_id); .. }
            AttackTargeting => { self.targeted = Some(event.unit_id); .. }
        }
    }
}
```

### Step 7: 替换 test_battle/render.rs 旧 observer

- 删除 `on_unit_click` (Entity observer)
- 删除 `on_unit_hover` (Entity observer)
- 删除 `on_unit_unhover` (Entity observer)
- 替换为全局 observer 在 `ui/picking/intent/`

### 验证: `cargo check` + `cargo nextest run`

---

## Phase 3：Selection 重构（~4h）

### Step 1: 创建 SelectionState Resource

```rust
#[derive(Resource)]
pub struct SelectionState {
    pub hovered: Option<PickTarget>,
    pub focused: Option<PickTarget>,
    pub selected: Option<PickTarget>,
    pub targeted: Option<PickTarget>,
    pub activated: Option<PickTarget>,
    pub context: PickContext,
}
```

### Step 2: 创建 SelectionChanged 事件 + SelectionPhase

```rust
pub struct SelectionChanged {
    pub previous: Option<PickTarget>,
    pub current: Option<PickTarget>,
    pub phase: SelectionPhase,
}
```

### Step 3: 创建 ui/projections/selection.rs

- 消费 `SelectionChanged` → 更新 `UiStore.battle_hud` / `UiStore.character_panel`
- 标记 `Dirty<BattleHudVm>` / `Dirty<CharacterPanelVm>`

### Step 4: Camera 跟随通过 CameraRequest

```rust
fn on_selection_changed(ev: On<SelectionChanged>, ..) {
    if let Some(PickTarget::Unit(id)) = &ev.current {
        commands.trigger(CameraRequest::Follow { unit_id: id });
    }
}
```

- 删除 `camera_follow_selection` 系统（直接写 TargetPose 的违规代码）
- 使用 `CameraRequest::Follow` 代替

### Step 5: 视觉高亮系统

```rust
fn on_selection_highlight(ev: On<SelectionChanged>, mut sprites: Query<&mut Sprite>, ..) {
    match ev.phase {
        SelectionPhase::Hovered => { /* 金色高亮 */ }
        SelectionPhase::Selected => { /* 青色高亮 */ }
        SelectionPhase::Cleared => { /* 恢复队伍色 */ }
    }
}
```

### Step 6: 清理硬编码值

- `store.character_panel.mp = 50.0` → 从领域组件读取
- `store.battle_hud.mp = 50.0` → 同上
- `store.character_panel.level = 1` → 从 Level 组件读取

### Step 7: BattleUnitId → Entity 映射

```rust
// 在桥接层保存映射表
struct BattleUnitIdMap(HashMap<BattleUnitId, Entity>);
```

### 验证: `cargo check` + `cargo nextest run`

---

## Phase 4：清理调试代码（~1h）

### Step 1: 删除 infra/picking/plugin.rs 中的 debug_click_handler

### Step 2: 删除 infra/picking/plugin.rs 中的 debug_hover_handler

### Step 3: 清理 test_battle/render.rs 中的 println!

### Step 4: 清理全局 add_observer 调用

---

## Phase 5：Bevy 兼容性（~2h）

### Step 1: is_window_picking_enabled = true（已做）

### Step 2: 验证 UI 穿透覆盖所有后代节点

- 检查 `mark_battle_ui_passthrough` 是否覆盖所有 BattleScreen 后代
- Zone 容器已有 Pickable::IGNORE
- Widget/按钮等子节点需要排查

### Step 3: 验证 ViewVisibility 时序

- 确认 SpritePickingPlugin 不会在首帧因 ViewVisibility::HIDDEN 跳过单位
- 如果首帧有问题: 在 `attach_unit_visuals` 中显式插入 `ViewVisibility::VISIBLE`

---

## Phase 6：验证（~2h）

### Step 1-3: 自动化验证

```bash
cargo build
cargo clippy -- -D warnings
cargo nextest run
```

### Step 4: 手动测试清单

- [ ] 左键点击己方单位 → 选中（青色高亮 + CharacterCard 更新）
- [ ] 左键点击敌方单位 → 选中
- [ ] 右键点击 → 取消选中
- [ ] 鼠标悬停 → 金色高亮
- [ ] 移出 → 恢复选中色/队伍色
- [ ] 点击空地 → 取消选中
- [ ] Camera 跟随选中单位
- [ ] ActionMenu 联动
- [ ] SkillPanel 显示
- [ ] End Turn 按钮

### Step 5: 归档 docs/11-refactor/ → docs/10-reviews/

---

## Phase 7：文档归档（~0.5h）

### Step 1: 更新 docs/11-refactor/README.md

标记所有完成项

### Step 2: 更新 ADR 状态

- ADR-068: Proposed → Approved
- ADR-067: 确认 Superseded

### Step 3: 确认文档完整性

- docs/01-architecture/README.md ADR 表
- docs/06-ui/01-architecture/architecture.md picking/selection 节
