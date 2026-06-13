# 业务代码修改建议

**Version**: 3.0
**Date**: 2026-06-11
**Reviewer**: Test Guardian
**Source**: UI 模块 + Character 模块 + Debug 模块测试执行过程中的编译警告和架构问题
**Standard**: `docs/test_spec.md`, `code_style.md`, `ai_constitution.md`

---

# 1. 问题来源

在执行 `cargo test` 过程中，编译器输出以下警告，表明业务代码存在潜在问题。此外，Debug 模块测试过程中发现架构问题影响可测试性。

---

# 2. 问题清单

## 2.1 `src/inventory/transfer.rs` - 无效的 drop 调用

**严重性**: P2
**位置**: `src/inventory/transfer.rs:83-84`
**问题**: 对引用类型调用 `drop()` 无实际效果

```rust
// 当前代码
drop(from_container);  // from_container 是 &Container
drop(to_container);    // to_container 是 &Container
```

**原因分析**: `drop()` 函数接收所有权值，对引用调用 `drop()` 不会销毁资源。这是代码逻辑错误，可能意图是提前释放借用。

**修改建议**:
```rust
// 方案 1: 使用 let _ = 忽略（如果只是想提前结束借用）
let _ = from_container;
let _ = to_container;

// 方案 2: 如果需要提前释放，应该 drop 通过 .borrow_mut() 获取的值
```

**影响范围**: 背包物品转移逻辑
**风险**: 低 - 当前代码无实际效果，但可能导致借用检查器行为不一致

---

## 2.2 `src/equipment/equip.rs` - 未使用的变量

**严重性**: P3
**位置**: `src/equipment/equip.rs:480, 569, 603`
**问题**: 声明了 `mut tags` 但从未使用

```rust
// 当前代码
let mut tags = GameplayTags::default();  // 警告: unused variable
```

**原因分析**: 变量声明后未被使用，可能是重构后遗留的代码。

**修改建议**:
```rust
// 方案 1: 添加下划线前缀表示有意忽略
let _tags = GameplayTags::default();

// 方案 2: 如果完全不需要，删除该行
```

**影响范围**: 装备穿戴逻辑
**风险**: 无 - 仅代码整洁问题

---

## 2.3 `src/skill/preview.rs` - 未使用的字段和函数

**严重性**: P3
**位置**: `src/skill/preview.rs:14-20, 26, 55, 64`
**问题**: 多个字段和函数未被使用

```rust
// 未使用的字段
pub struct SkillExecutionContext {
    pub source: Entity,      // never read
    pub target: Entity,      // never read
    pub skill_id: String,    // never read
    pub source_tags: GameplayTags,  // never read
    pub target_tags: GameplayTags,  // never read
}

// 未使用的函数
pub fn from_query(...) { ... }  // never used

// 未使用的字段
pub struct SkillPreview {
    pub skill_name: String,  // never read
}

// 未使用的枚举变体字段
BuffApplied { buff_name: String },  // never read
```

**原因分析**: 这些字段/函数可能是为未来功能预留的，或者是重构后遗留的代码。

**修改建议**:
```rust
// 方案 1: 如果确认不再需要，删除未使用的字段/函数
// 方案 2: 如果将来会使用，添加 #[allow(dead_code)] 注释说明意图
#[allow(dead_code)]  // 预留给技能预览功能
pub struct SkillExecutionContext { ... }
```

**影响范围**: 技能预览系统
**风险**: 低 - 仅代码整洁问题

---

## 2.4 `src/ai/targeting.rs` - 未使用的字段

**严重性**: P3
**位置**: `src/ai/targeting.rs:17`
**问题**: `acted` 字段未被读取

```rust
pub(crate) struct UnitSnapshot {
    // ...
    pub acted: bool,  // never read
}
```

**原因分析**: `UnitSnapshot` 的 `acted` 字段在 AI 目标选择逻辑中未被使用，可能是重构后遗留。

**修改建议**:
```rust
// 方案 1: 如果 AI 需要考虑单位是否已行动，应使用该字段
// 方案 2: 如果不需要，删除该字段
```

**影响范围**: AI 目标选择
**风险**: 低 - 可能影响 AI 决策质量（如果本应使用但未使用）

---

## 2.5 `src/character/movement.rs` - 未使用的常量

**严重性**: P3
**位置**: `src/character/movement.rs:18`
**问题**: `MOVE_SPEED` 常量未被使用

```rust
const MOVE_SPEED: f32 = 0.15;  // never used
```

**原因分析**: 常量声明后未被使用，可能在其他地方硬编码了相同值。

**修改建议**:
```rust
// 方案 1: 在移动系统中使用该常量
// 方案 2: 如果不再需要，删除该常量
```

**影响范围**: 单位移动系统
**风险**: 低 - 仅代码整洁问题

---

## 2.6 `src/character/template.rs` - 未使用的字段

**严重性**: P3
**位置**: `src/character/template.rs:20`
**问题**: `background` 字段未被读取

```rust
pub struct UnitTemplate {
    // ...
    pub background: String,  // never read
}
```

**原因分析**: 角色模板的背景故事字段未被使用，可能是为未来功能预留。

**修改建议**:
```rust
// 方案 1: 如果角色背景功能已废弃，删除该字段
// 方案 2: 如果将来会使用，保留并添加注释
```

**影响范围**: 角色模板系统
**风险**: 低 - 仅代码整洁问题

---

## 2.7 `src/core/registry_loader.rs` - 未使用的测试结构体字段

**严重性**: P3
**位置**: `src/core/registry_loader.rs:163`
**问题**: 测试结构体的 `value` 字段未被读取

```rust
struct TestItem {
    id: String,
    value: i32,  // never read
}
```

**原因分析**: 测试辅助结构体的字段未被使用，仅用于验证反序列化。

**修改建议**:
```rust
// 无需修改 - 测试代码中的 unused 警告可忽略
// 或添加 #[allow(dead_code)]
```

**影响范围**: 无 - 仅测试代码
**风险**: 无

---

## 2.8 `src/battle/record.rs` - 非 snake_case 函数名

**严重性**: P3
**位置**: `src/battle/record.rs:450`
**问题**: 测试函数名不符合 snake_case 规范

```rust
fn 战斗记录_最近N条() { ... }  // 警告: should have a snake case name
```

**原因分析**: 使用中文函数名，不符合 Rust 命名规范。

**修改建议**:
```rust
// 重命名为 snake_case 英文
fn battle_record_recent_n_entries() { ... }
```

**影响范围**: 仅测试代码
**风险**: 无

---

## 2.9 `src/inventory/use_item.rs` - 非 snake_case 函数名

**严重性**: P3
**位置**: `src/inventory/use_item.rs:340, 374`
**问题**: 测试函数名不符合 snake_case 规范

```rust
fn 消耗品_GrantTempTrait返回PendingEffect() { ... }
fn 消耗品_CastSkill返回PendingEffect() { ... }
```

**修改建议**:
```rust
fn consumable_grant_temp_trait_returns_pending_effect() { ... }
fn consumable_cast_skill_returns_pending_effect() { ... }
```

**影响范围**: 仅测试代码
**风险**: 无

---

## 2.10 `src/map/pathfinding/mod.rs` - 非 snake_case 函数名

**严重性**: P3
**位置**: `src/map/pathfinding/mod.rs:516`
**问题**: 测试函数名不符合 snake_case 规范

```rust
fn 回溯路径_L形路径() { ... }
```

**修改建议**:
```rust
fn backtrack_path_l_shaped() { ... }
```

**影响范围**: 仅测试代码
**风险**: 无

---

# 3. 未使用的 UI Widget 函数

**严重性**: P2
**位置**: `src/ui/widgets/layout.rs`
**问题**: 多个 Widget 函数未被使用

```rust
pub fn vbox(theme: &UiTheme) -> Node { ... }           // never used
pub fn vbox_with_gap(gap: f32) -> Node { ... }         // never used
pub fn hbox_with_gap(gap: f32) -> Node { ... }         // never used
pub fn panel(theme: &UiTheme) -> Node { ... }          // never used
pub fn panel_absolute(...) -> Node { ... }             // never used
pub fn divider(theme: &UiTheme) -> (Node, BackgroundColor) { ... }  // never used
```

**原因分析**: 这些 Widget 函数是为 UI 组件库预留的，但当前未被使用。

**修改建议**:
```rust
// 方案 1: 如果 UI 组件库计划使用，保留并添加 #[allow(dead_code)]
// 方案 2: 如果不再需要，删除未使用的函数
```

**影响范围**: UI 组件库
**风险**: 低 - 可能影响未来 UI 开发

---

# 4. 优先级总结

| 优先级 | 问题 | 位置 | 说明 |
|--------|------|------|------|
| **P2** | 无效 drop 调用 | transfer.rs:83-84 | 逻辑错误，需确认意图 |
| **P2** | 未使用的 Widget 函数 | layout.rs | 可能影响未来 UI 开发 |
| **P3** | 未使用的变量/字段 | equip.rs, targeting.rs, template.rs | 代码整洁问题 |
| **P3** | 未使用的常量 | movement.rs:18 | 代码整洁问题 |
| **P3** | 非 snake_case 函数名 | record.rs, use_item.rs, pathfinding | 命名规范问题 |

---

# 5. 建议执行顺序

1. **立即修复**: `transfer.rs` 的 drop 调用问题（P2）
2. **本周内**: 清理 `equip.rs` 的未使用变量（P3）
3. **计划中**: 评估 `preview.rs` 和 `targeting.rs` 的未使用字段是否需要保留
4. **低优先级**: 重命名非 snake_case 的测试函数

---

# 6. 结论

测试执行过程中发现 **12 个业务代码问题**，其中：
- **2 个 P2 问题**：需要确认意图并修复（无效 drop 调用、未使用的 Widget 函数）
- **10 个 P3 问题**：代码整洁和命名规范问题

这些问题不影响测试通过，但建议在后续开发中逐步清理，以保持代码质量。

---

# 附录：Character 模块问题详情

以下问题在 `cargo test --lib character::` 执行过程中由编译器报告：

## A.1 `src/character/movement.rs` - 未使用的常量

**严重性**: P3
**位置**: `src/character/movement.rs:18`
**问题**: `MOVE_SPEED` 常量未被使用

```rust
const MOVE_SPEED: f32 = 0.15;  // never used
```

**修改建议**: 在移动系统中使用该常量，或删除。

## A.2 `src/character/template.rs` - 未使用的字段

**严重性**: P3
**位置**: `src/character/template.rs:20`
**问题**: `UnitTemplate` 的 `background` 字段未被读取

```rust
pub struct UnitTemplate {
    // ...
    pub background: String,  // never read
}
```

**修改建议**: 如果角色背景功能已废弃，删除该字段；否则保留并添加注释。

## A.3 跨模块编译警告（影响 character 测试执行）

以下警告来自其他模块，但影响 `cargo test` 整体执行：

| 位置 | 问题 | 严重性 |
|------|------|--------|
| `battle/events.rs:114` | 未使用导入 `bevy::prelude::*` | P3 |
| `equipment/definition.rs:266` | 未使用导入 `AttributeKind`, `ModifierOp` | P3 |
| `equipment/equip.rs:429` | 未使用导入 `TagName` | P3 |
| `inventory/container.rs:232` | 未使用导入 `InstanceIdCounter` | P3 |
| `inventory/use_item.rs:193` | 未使用导入 `ItemInstance` | P3 |
| `skill/preview.rs:142` | 未使用导入 `GameplayTags` | P3 |
| `core/registry_loader.rs:163` | 字段 `value` 未被读取 | P3 |
| `battle/record.rs:450` | 非 snake_case 函数名 `战斗记录_最近N条` | P3 |
| `inventory/use_item.rs:340,374` | 非 snake_case 函数名 | P3 |
| `map/pathfinding/mod.rs:516` | 非 snake_case 函数名 `回溯路径_L形路径` | P3 |

---

# 附录 B：Debug 模块架构问题

## B.1 `src/debug/viewers/damage_viewer.rs` - render_damage_panel 无法单元测试

**严重性**: P2
**位置**: `src/debug/viewers/damage_viewer.rs:10`
**问题**: `render_damage_panel` 函数需要 `&mut egui::Ui` 参数，无法在单元测试中调用

```rust
pub fn render_damage_panel(
    ui: &mut bevy_inspector_egui::egui::Ui,  // 依赖 egui 上下文
    battle_record: &BattleRecord,
    _units: &Query<&UnitName>,
) {
    // 过滤逻辑：filter_map + take(20)
    // 无法在不启动 egui 的情况下测试
}
```

**原因分析**: 函数签名将过滤逻辑与 egui 渲染耦合，违反了 Logic/Presentation 分离原则（architecture.md §4）。

**修改建议**:
```rust
// 方案 1: 提取纯过滤逻辑为独立函数
pub fn filter_damage_entries(record: &BattleRecord) -> Vec<DamageEntry> {
    record.entries.iter().rev()
        .filter_map(|e| { /* 过滤逻辑 */ })
        .take(20)
        .collect()
}

// 方案 2: 保留现有实现，接受不可测试性（§1.1 排除 UI 测试）
```

**影响范围**: Debug 模块伤害分解面板
**风险**: 低 - 仅影响测试覆盖，不影响运行时行为

---

## B.2 `src/debug/viewers/attribute_viewer.rs` - render_attribute_panel 无法单元测试

**严重性**: P2
**位置**: `src/debug/viewers/attribute_viewer.rs:13`
**问题**: `render_attribute_panel` 函数需要 `&mut egui::Ui` 参数，无法在单元测试中调用

```rust
pub fn render_attribute_panel(
    ui: &mut bevy_inspector_egui::egui::Ui,  // 依赖 egui 上下文
    units: &Query<...>,
) {
    // 分组逻辑：collect + sort_by_key + dedup
    // 无法在不启动 egui 的情况下测试
}
```

**原因分析**: 函数签名将分组逻辑与 egui 渲染耦合，违反了 Logic/Presentation 分离原则（architecture.md §4）。

**修改建议**:
```rust
// 方案 1: 提取纯分组逻辑为独立函数
pub fn group_modifiers_by_kind(attrs: &Attributes) -> Vec<AttributeKind> {
    let mut kinds: Vec<AttributeKind> = attrs.modifiers.iter().map(|m| m.kind).collect();
    kinds.sort_by_key(|k| format!("{:?}", k));
    kinds.dedup();
    kinds
}

// 方案 2: 保留现有实现，接受不可测试性（§1.1 排除 UI 测试）
```

**影响范围**: Debug 模块属性修饰符面板
**风险**: 低 - 仅影响测试覆盖，不影响运行时行为

---

## B.3 `src/debug/viewers/grid_viewer.rs` - 分页逻辑与 egui 耦合

**严重性**: P3
**位置**: `src/debug/viewers/grid_viewer.rs:70-86`
**问题**: 分页计算逻辑嵌入在 egui 回调中，无法直接测试

```rust
// 分页逻辑嵌入在 egui 按钮回调中
if ui.button("▲ 上页").clicked() {
    viewer_state.scroll_row = (viewer_state.scroll_row - viewer_state.page_rows).max(0);
}
```

**原因分析**: 分页逻辑与 UI 交互耦合，但数学计算本身是纯函数。

**修改建议**: 当前已通过在测试中复制逻辑验证正确性（v2.0 测试），可接受。

**影响范围**: Grid Viewer 分页功能
**风险**: 无 - 已通过测试验证
