# ADR: 统一调试面板架构

## 状态

Proposed

## 背景

当前调试模块包含多个独立的 egui 面板，每个面板通过不同的快捷键控制：

| 面板 | 快捷键 | 功能 |
|------|--------|------|
| Battle Debugger | F1 | 回合状态快照 |
| Buff Viewer | F2 | 单位 Buff 状态 |
| Debug Overlay | F3 | Gizmos 可视化开关 |
| Damage & Attribute | F4 | 伤害分解与属性修饰 |
| Turn Queue | F5 | 行动队列预览 |
| Debug Stepping | F6/F7 | 系统单步调试 |
| World Inspector | F12 | ECS 世界检查器 |

**当前问题**：

1. **屏幕空间占用**：多个独立面板同时打开时遮挡游戏画面
2. **导航碎片化**：开发者需要记住多个快捷键，切换面板不直观
3. **状态管理分散**：每个面板独立管理显隐状态
4. **位置规划复杂**：需要精心设计每个面板的位置避免重叠
5. **扩展性差**：新增调试视图需要添加新的快捷键和位置规划

## 决策

### 核心原则：统一入口，分区展示

将所有调试面板集成到一个统一的主面板中，使用侧边栏导航切换不同视图。

### 架构设计

#### 1. 统一面板结构

```
┌─────────────────────────────────────────────────────────┐
│  Debug Panel (F1 切换显隐)                               │
├──────────┬──────────────────────────────────────────────┤
│          │                                              │
│  导航栏   │              内容区域                        │
│          │                                              │
│  ▸ Battle│  [根据选中的导航项显示对应内容]                │
│  ▸ Buff  │                                              │
│  ▸ Overlay│                                             │
│  ▸ Damage│                                              │
│  ▸ Turn  │                                              │
│  ▸ Stepping│                                            │
│  ▸ Grid  │                                              │
│  ▸ AI    │                                              │
│  ▸ Equip │                                              │
│  ▸ Settings│                                            │
│          │                                              │
├──────────┴──────────────────────────────────────────────┤
│  [状态栏: Stepping状态 | 快捷键提示]                     │
└─────────────────────────────────────────────────────────┘
```

#### 2. 核心数据结构

```rust
/// 统一调试面板状态
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct DebugPanelState {
    /// 主面板显隐（F1 控制）
    pub show_panel: bool,
    
    /// 当前选中的导航项
    pub active_view: DebugView,
    
    /// 各视图的独立状态（保留向后兼容）
    pub battle_debugger: BattleDebug ViewState,
    pub buff_viewer: BuffViewState,
    pub overlay: OverlayViewState,
    pub damage_attribute: DamageAttributeViewState,
    pub turn_queue: TurnQueueViewState,
    pub stepping: SteppingViewState,
    pub grid_viewer: GridViewerState,
    pub ai_viewer: AiViewerState,
    pub equipment_viewer: EquipmentViewState,
    pub settings: SettingsViewState,
}

/// 调试视图枚举
#[derive(Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum DebugView {
    #[default]
    Battle,
    Buff,
    Overlay,
    DamageAttribute,
    TurnQueue,
    Stepping,
    Grid,
    Ai,
    Equipment,
    Settings,
}
```

#### 3. 导航栏实现

```rust
/// 渲染左侧导航栏
fn render_navigation(ui: &mut egui::Ui, state: &mut DebugPanelState) {
    ui.vertical(|ui| {
        ui.set_min_width(80.0);
        
        let views = [
            (DebugView::Battle, "Battle", "F1"),
            (DebugView::Buff, "Buff", "F2"),
            (DebugView::Overlay, "Overlay", "F3"),
            (DebugView::DamageAttribute, "Damage", "F4"),
            (DebugView::TurnQueue, "Turn", "F5"),
            (DebugView::Stepping, "Stepping", "F6"),
            (DebugView::Grid, "Grid", ""),
            (DebugView::Ai, "AI", ""),
            (DebugView::Equipment, "Equip", ""),
            (DebugView::Settings, "Settings", ""),
        ];
        
        for (view, label, shortcut) in views {
            let is_selected = state.active_view == view;
            let button = ui.selectable_label(is_selected, label);
            
            if button.clicked() {
                state.active_view = view;
            }
            
            // 显示快捷键提示
            if !shortcut.is_empty() {
                ui.small(format!("({})", shortcut));
            }
        }
    });
}
```

#### 4. 内容区域渲染

```rust
/// 根据选中的视图渲染内容
fn render_view_content(ui: &mut egui::Ui, state: &mut DebugPanelState, /* 其他资源 */) {
    match state.active_view {
        DebugView::Battle => battle_debugger::render(ui, &state.battle_debugger),
        DebugView::Buff => buff_viewer::render(ui, &state.buff_viewer),
        DebugView::Overlay => overlay::render(ui, &mut state.overlay),
        DebugView::DamageAttribute => damage_attribute::render(ui, &mut state.damage_attribute),
        DebugView::TurnQueue => turn_queue::render(ui, &state.turn_queue),
        DebugView::Stepping => stepping::render(ui, &mut state.stepping),
        DebugView::Grid => grid_viewer::render(ui, &mut state.grid_viewer),
        DebugView::Ai => ai_viewer::render(ui, &state.ai_viewer),
        DebugView::Equipment => equipment_viewer::render(ui, &state.equipment_viewer),
        DebugView::Settings => settings::render(ui, &mut state.settings),
    }
}
```

#### 5. 快捷键处理

```rust
/// 统一快捷键处理系统
pub fn debug_hotkey_system(
    mut state: ResMut<DebugPanelState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // F1: 切换主面板显隐
    if keyboard.just_pressed(KeyCode::F1) {
        state.show_panel = !state.show_panel;
        return; // 面板关闭时忽略其他快捷键
    }
    
    // 仅在面板打开时处理视图切换快捷键
    if !state.show_panel {
        return;
    }
    
    // F2-F5: 切换视图（保持向后兼容）
    if keyboard.just_pressed(KeyCode::F2) {
        state.active_view = DebugView::Buff;
    }
    if keyboard.just_pressed(KeyCode::F3) {
        // F3 保持原有行为：切换 Overlay 面板内的开关
        state.active_view = DebugView::Overlay;
        state.overlay.toggle_all();
    }
    if keyboard.just_pressed(KeyCode::F4) {
        state.active_view = DebugView::DamageAttribute;
    }
    if keyboard.just_pressed(KeyCode::F5) {
        state.active_view = DebugView::TurnQueue;
    }
    
    // F6/F7: Stepping 控制（无论面板是否打开都生效）
    if keyboard.just_pressed(KeyCode::F6) {
        state.stepping.toggle_enabled();
    }
    if keyboard.just_pressed(KeyCode::F7) && state.stepping.is_enabled() {
        state.stepping.step();
    }
}
```

### 边界定义

**统一调试面板负责**：
- 管理所有调试视图的显隐状态
- 提供统一的导航入口
- 协调各视图的数据访问
- 处理快捷键输入

**统一调试面板不负责**：
- 各视图的具体渲染逻辑（由各 viewer 模块实现）
- 业务状态的修改（只读访问）
- Gizmos 可视化（仍在 Last Schedule 中独立执行）

### 迁移策略

#### 第一阶段：创建统一面板框架
1. 定义 `DebugPanelState` 和 `DebugView` 枚举
2. 实现导航栏和内容区域容器
3. 集成现有的 `debug_hotkey_system`

#### 第二阶段：迁移现有视图
1. 将各 viewer 的渲染逻辑提取为 `render()` 函数
2. 在统一面板中调用各视图的 `render()` 函数
3. 保留各视图的独立状态结构

#### 第三阶段：优化体验
1. 添加视图切换动画
2. 实现面板大小记忆
3. 添加搜索/过滤功能（可选）

#### 第四阶段：清理旧代码
1. 移除独立的条件渲染系统
2. 更新快捷键文档
3. 更新测试用例

### 测试策略

**单元测试**：
- 测试 `DebugView` 枚举的状态转换
- 测试导航栏的选中状态管理
- 测试快捷键的优先级处理

**功能测试**：
- 测试面板显隐切换
- 测试视图切换
- 测试各视图的数据渲染

**场景测试**：
- 测试多个视图快速切换
- 测试面板大小调整
- 测试快捷键冲突处理

## 后果

### 正面影响

1. **统一入口**：一个快捷键（F1）控制所有调试功能
2. **节省空间**：单个面板比多个独立面板更节省屏幕空间
3. **导航清晰**：侧边栏提供直观的视图切换
4. **易于扩展**：新增调试视图只需添加新的 `DebugView` 枚举值
5. **状态集中**：所有调试状态在一个 Resource 中管理
6. **向后兼容**：F2-F5 快捷键保持原有功能

### 负面影响

1. **初期工作量**：需要重构现有面板代码
2. **学习成本**：开发者需要适应新的导航方式
3. **复杂度增加**：统一面板的内部状态管理更复杂
4. **性能开销**：单个面板渲染所有视图可能有微小开销（可忽略）

### 风险缓解

- **学习成本**：保留 F2-F5 快捷键作为快速切换入口
- **性能问题**：只渲染当前选中的视图，其他视图懒加载
- **状态管理**：使用 ECS Resource 确保状态一致性

## 替代方案

### 方案 B：顶部 Tab 栏（不采用）

使用顶部 Tab 栏切换视图，类似浏览器标签页。

**拒绝原因**：
- Tab 数量过多（10+），顶部空间不足
- Tab 文字过长时需要截断，影响可读性
- 不如侧边栏导航直观

### 方案 C：折叠面板 Accordion（不采用）

使用折叠面板，点击标题展开/折叠各视图。

**拒绝原因**：
- 无法同时查看多个视图
- 面板会变得很长，需要滚动
- 不如侧边栏导航清晰

### 方案 D：保持独立面板（不采用）

保持现有独立面板，仅添加一个"调试面板"主入口。

**拒绝原因**：
- 没有真正解决屏幕空间占用问题
- 仍然是多个独立窗口
- 状态管理仍然分散

## 实现清单

- [ ] 定义 `DebugPanelState` 和 `DebugView` 枚举
- [ ] 实现统一面板容器（导航栏 + 内容区域）
- [ ] 迁移 Battle Debugger 视图
- [ ] 迁移 Buff Viewer 视图
- [ ] 迁移 Overlay 视图
- [ ] 迁移 Damage & Attribute 视图
- [ ] 迁移 Turn Queue 视图
- [ ] 迁移 Stepping 视图
- [ ] 迁移 Grid Viewer 视图
- [ ] 迁移 AI Viewer 视图
- [ ] 迁移 Equipment Viewer 视图
- [ ] 迁移 Settings 视图
- [ ] 更新快捷键处理系统
- [ ] 添加视图切换动画（可选）
- [ ] 更新单元测试
- [ ] 更新功能测试
- [ ] 更新文档

## 参考

- `src/debug/mod.rs` - 当前调试模块入口
- `src/debug/viewers/` - 各调试视图实现
- `docs/domain/debug_rules.md` - 调试领域规则
- `docs/architecture.md` - 架构规范
- AGENTS.md - ECS 架构约束

---

**创建时间**: 2026-06-12  
**作者**: MiMoCode Compose Agent  
**状态**: Proposed - 待讨论