# 重构计划：清理 Picking 相关调试代码

> 关联 ADR: ADR-068 §Forbidden | 宪法 §1.5(3) Replay First

## 目标状态 (Target)

- `println!` 宏不在 Picking/Selection 相关代码中出现（0 个实例）
- 调试输出使用 `tracing` 宏（tracing::debug!, tracing::trace!），带 `target` 标签
- 全局 debug observer 只在 dev feature 下启用，不在 release 构建中出现
- 删除所有 `[DEBUG]` / `[Picking]` / `[Camera]` 前缀的 `println!` 打印

## 当前状态 (Current)

| 文件 | 行号 | 调试代码 | 问题 |
|------|------|---------|------|
| `src/infra/picking/plugin.rs` | 14-34 | `debug_click_handler` + `debug_hover_handler` — 全局 `println!` | release 构建中不干净 |
| `src/app/scenes/test_battle/render.rs` | 75-102 | `on_unit_click` 中 3 处 `println!` + `[DEBUG]` 前缀 | 生产代码包含调试输出 |
| `src/app/scenes/test_battle/render.rs` | 199 | `println!("[DEBUG] Unit entity: ...")` | 重构旧代码残留 |
| `src/ui/projections/selection.rs` | 136 | `info!(target: "ui", "[Selection] ...")` | 可保留（使用 tracing，非 println） |
| `src/ui/projections/selection.rs` | 210 | `info!(target: "camera", "[Camera] ...")` | 可保留（使用 tracing，非 println） |
| `src/app/scenes/test_battle/spawn.rs` | 111 | `println!("[DEBUG] ...")` | Picking 不直接相关但也是调试残留 |

## 差距分析

| # | 当前 | 目标 | 等级 |
|---|------|------|------|
| D1 | `println!` 出现在 picking 相关代码 | 0 个 `println!` | 🟨 |
| D2 | 全局 debug observer 无条件注册 | 仅 dev feature 下注册 | 🟨 |
| D3 | `info!` 日志使用 `[Selection]` / `[Camera]` 前缀 | 使用结构化日志（target + 结构化字段） | 🟩 |
| D4 | per-entity `println!` 在 spawn 循环中 | 删除 | 🟩 |

## 迁移步骤

### 步骤 1: 删除全局 debug observer

修改 `src/ui/picking/plugin.rs`（迁移后的位置）：

```rust
// ❌ 删除 — 全局 debug observer 不进入 release

// ✅ 替换为 — dev feature gate
#[cfg(feature = "dev")]
fn dev_debug_click_handler(ev: On<Pointer<Click>>) {
    let hit_pos = ev.event().hit.position
        .map(|p| format!("({:.0},{:.0})", p.x, p.y))
        .unwrap_or_default();
    tracing::debug!(
        target: "picking",
        pointer_event = "click",
        target = ?ev.event_target(),
        button = ?ev.event().button,
        position = %hit_pos,
    );
}
```

### 步骤 2: 清理 render.rs 中的 println!

修改 `src/app/scenes/test_battle/render.rs`：

2.1 删除 `on_unit_click` 函数中的 `println!` 调用：
```rust
// ❌ 删除以下行:
// println!("[DEBUG] on_unit_click fired! ...");
// println!("[DEBUG] Right-click deselected");
// println!("[Picking] Unit selected: {} ...");
```

2.2 将需要的日志改为 tracing:
```rust
// ✅ 保留为 tracing::debug!
tracing::debug!(
    target: "picking",
    event = "unit_selected",
    unit_id = %uid.id,
    entity = ?ev.event_target(),
);
```

2.3 删除 `attach_unit_visuals` 中的 `println!`:
```rust
// ❌ 删除:
// println!("[DEBUG] Unit entity: {:?} id={}", entity, uid.id);
// ✅ 如果需要保留日志:
tracing::trace!(
    target: "render",
    event = "visual_attached",
    entity = ?entity,
    unit_id = %uid.id,
);
```

### 步骤 3: 使用结构化日志

将所有 `info!` 日志中的 `[Selection]` / `[Camera]` 前缀替换为结构化日志：

```rust
// ❌ 旧:
info!(target: "ui", "[Selection] Projected '{}' → ...", uid.id);

// ✅ 新:
info!(
    target: "ui",
    event = "selection_projected",
    unit_id = %uid.id,
    hp = hp.current,
    max_hp = hp.maximum,
    "Selection projected: unit_id={} HP={}/{}",
    uid.id, hp.current, hp.maximum,
);
```

### 步骤 4: 检查其他文件中的 println!

搜索全项目中 Picking/Selection 相关的 `println!`:

```bash
grep -rn "println\|\[DEBUG\]\|\[Picking\]" src/app/scenes/test_battle/ src/infra/picking/ src/ui/picking/ src/ui/projections/selection.rs
```

## 通过标准

- [ ] `git grep "println" src/ui/picking/` 返回空
- [ ] `git grep "println" src/ui/projections/selection.rs` 返回空
- [ ] `git grep "println" src/app/scenes/test_battle/` 返回 0（如果 spawn.rs 中的也清理）
- [ ] `git grep "debug_click_handler\|debug_hover_handler" src/` 不返回无条件注册的代码
- [ ] `cargo build --release` 不产生任何 picking 相关的 warning
- [ ] `cargo clippy -- -D warnings` 通过

## 预估工作量

| 步骤 | 文件 | 行数 | 时间 |
|------|------|------|------|
| 1. 删除 debug observer | 修改 plugin.rs | ~20 行 | 0.3h |
| 2. 清理 render.rs println! | 修改 render.rs | ~15 行 | 0.3h |
| 3. 结构化日志 | 修改 projections/selection.rs | ~15 行 | 0.5h |
| 4. 全面检查 | | 0 | 0.2h |
| **合计** | | **~50 行** | **~1.3h** |
