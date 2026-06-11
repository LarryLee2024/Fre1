# 游戏运行时问题诊断报告

Version: 1.0
Date: 2026-06-12
Status: 诊断完成，待修复

---

## 问题总览

| # | 问题 | 严重度 | 优先级 | 根因定位 |
|---|------|--------|--------|----------|
| 1 | 地形多样性缺失 | 🟥 Critical | P0 | RON 文件缺少 char_code 字段 |
| 2 | 开场棋子移动卡顿 | 🟡 Major | P1 | AI 计时器延迟 + BFS 首帧计算 |
| 3 | 角色信息面板闪烁 | 🟥 Critical | P0 | TileSprite 拦截 Pointer 事件 |
| 4 | 棋子移动路径异常 | 🟥 Critical | P0 | reconstruct_path 可能返回不完整路径 |
| 5 | 移动轨迹显示缺失 | 🟡 Major | P1 | 路径过短或渲染层级问题 |
| 6 | 相机视口移动交互 | 🟢 Minor | P2 | 边缘滚动体验不佳，建议改为右键拖动 |

---

## 问题1：地形多样性缺失

### 问题描述

当前游戏地图仅显示草地（plain）地形，与 tutorial.ron 中定义的多种地形（山/林/水）不符。历史版本曾正确显示多种地形。

### 可能原因分析

1. **关卡配置文件加载失败** — LevelRegistry 为空，回退到全平地
2. **地形注册表加载失败** — TerrainRegistry 为空，无法解析地形 ID
3. **字符映射缺失** — terrain_grid 中的字符（M/F/W/P）无法映射到地形 ID

### 初步定位结果

**根因：地形 RON 文件缺少 `char_code` 字段**

定位路径：

1. `assets/terrains/plain.ron` 等文件内容：
   ```ron
   (id: "plain", name: "草", move_cost: 1, defense_bonus: 0, color: (0.56, 0.73, 0.35), passable: true,)
   ```
   缺少 `char_code` 字段。

2. `TerrainDefRon` 结构体中 `char_code` 标记了 `#[serde(default)]`，反序列化时默认为 `None`。

3. `TerrainRegistry::char_map()` 方法过滤 `char_code.is_some()`，所有地形的 char_code 均为 None → 返回空 HashMap。

4. `LevelConfig::from_def()` 中，空 char_map 导致所有字符回退到 `"plain"`：
   ```rust
   let terrain_id = char_map
       .get(&ch)
       .cloned()
       .unwrap_or_else(|| "plain".to_string());
   ```

5. `TerrainRegistry::register_defaults()` 虽然包含 char_code（如 `Some('P')`），但仅在注册表为空时调用。RON 文件成功加载后注册表非空，兜底逻辑不执行。

**数据驱动覆盖了硬编码默认值，但 RON 文件本身缺少关键字段，导致功能退化。**

### 建议解决方案

**方案A（推荐）：为 RON 文件添加 char_code 字段**

修改 `assets/terrains/` 下的 4 个文件：

- `plain.ron`: 添加 `char_code: Some('P')`
- `forest.ron`: 添加 `char_code: Some('F')`
- `mountain.ron`: 添加 `char_code: Some('M')`
- `water.ron`: 添加 `char_code: Some('W')`

**方案B（备选）：修改 register_defaults 逻辑**

将 `register_defaults()` 改为"补全缺失字段"而非"仅空时注册"，确保 RON 加载后仍能补充 char_code。但这违反数据驱动原则（硬编码覆盖外部配置），不推荐。

### 涉及文件

- [assets/terrains/plain.ron](file:///d:/Code/Bevy/Fre/assets/terrains/plain.ron)
- [assets/terrains/forest.ron](file:///d:/Code/Bevy/Fre/assets/terrains/forest.ron)
- [assets/terrains/mountain.ron](file:///d:/Code/Bevy/Fre/assets/terrains/mountain.ron)
- [assets/terrains/water.ron](file:///d:/Code/Bevy/Fre/assets/terrains/water.ron)
- [src/map/data.rs](file:///d:/Code/Bevy/Fre/src/map/data.rs) — `TerrainRegistry::char_map()` (L62-L67), `LevelConfig::from_def()` (L155-L170)

---

## 问题2：开场棋子移动卡顿

### 问题描述

游戏初始阶段，对方两个棋子依次移动时出现明显卡顿现象。

### 可能原因分析

1. **AI 计时器延迟** — 每个 AI 单位行动前有 0.8 秒等待
2. **BFS 首帧计算开销** — 首次调用 find_reachable_tiles 时计算量大
3. **资源首次加载** — 首次移动时触发某些资源的延迟加载
4. **OccupancyGrid 重建** — 移动后重建占用网格的开销
5. **帧率波动** — animate_movement 使用 delta_secs，帧时间不稳定时插值不平滑

### 初步定位结果

**主因：AI 计时器设计导致感知卡顿 + BFS 首帧计算峰值**

定位路径：

1. **AI 计时器**（[src/ai/decision.rs](file:///d:/Code/Bevy/Fre/src/ai/decision.rs) L64-L67）：
   ```rust
   ai_timer.timer.tick(time.delta());
   if !ai_timer.timer.just_finished() { return; }
   ```
   AiTimer 默认 0.8 秒。两个 AI 单位依次行动时，用户感知为：0.8s 等待 → 移动 → 0.8s 等待 → 移动，总延迟约 1.6 秒。

2. **BFS 计算在同帧执行**（[src/ai/decision.rs](file:///d:/Code/Bevy/Fre/src/ai/decision.rs) L131-L140）：
   AI 计时器到期后，同一帧内执行：快照收集 → 目标选择 → BFS 寻路 → 路径回溯 → 移动设置。BFS 对 10x8 地图的开销通常 < 1ms，但在首帧可能因缓存未命中而稍慢。

3. **移动动画速度**（[src/ai/decision.rs](file:///d:/Code/Bevy/Fre/src/ai/decision.rs) L204）：
   `speed: 0.15` — 每格 0.15 秒，5 格路径仅需 0.75 秒。动画本身流畅，卡顿感主要来自 AI 间的等待间隔。

4. **OccupancyGrid**：检查代码发现 OccupancyGrid 在移动完成后需要更新，但当前实现中未见每帧重建逻辑（项目记忆中提到"OccupancyGrid 每帧 rebuild"），这可能是性能瓶颈之一。

### 建议解决方案

| 方案 | 描述 | 影响 |
|------|------|------|
| 缩短 AI 计时器 | 将 0.8s 降为 0.3-0.5s | 减少感知等待 |
| 分帧执行 BFS | AI 计时器到期后，下一帧再执行 BFS | 避免单帧峰值 |
| 预计算可达范围 | 回合开始时预计算所有单位的可达范围 | 消除决策帧延迟 |
| 添加过渡动画 | AI 决策期间显示"思考中"指示器 | 改善感知体验 |

### 涉及文件

- [src/ai/decision.rs](file:///d:/Code/Bevy/Fre/src/ai/decision.rs) — `enemy_ai_system` (L20-L236)
- [src/character/movement.rs](file:///d:/Code/Bevy/Fre/src/character/movement.rs) — `animate_movement` (L86-L158)
- [src/map/pathfinding/algorithms.rs](file:///d:/Code/Bevy/Fre/src/map/pathfinding/algorithms.rs) — `find_reachable_tiles` (L79-L151)

---

## 问题3：角色信息面板闪烁

### 问题描述

鼠标悬停在坐标(2,5)的己方棋子及第二列格子的其他棋子上时，角色信息面板持续闪烁，表现类似无限次点击触发。第三列及右侧格子的棋子未出现此问题。

### 可能原因分析

1. **HoveredEntity 高频变化** — Pointer<Over>/<Out> 事件交替触发
2. **TileSprite 拦截鼠标事件** — 地图格子精灵与单位精灵重叠
3. **SelectedUnitView 频繁重建** — is_changed() 每帧返回 true
4. **UI 布局抖动** — 面板内容变化导致布局重排，触发新的悬停事件

### 初步定位结果

**根因：TileSprite 实体拦截 Pointer 事件，导致 HoveredEntity 在 Unit 和 None 之间高频切换**

定位路径：

1. **TileSprite 未设置 Pickable::IGNORE**（[src/map/grid.rs](file:///d:/Code/Bevy/Fre/src/map/grid.rs) L126-L147）：
   ```rust
   commands.spawn((
       Sprite::from_color(terrain_color, Vec2::splat(tile_size - 2.0)),
       Transform::from_xyz(world_pos.x, world_pos.y, 0.0),
       TileSprite,
       // 缺少 Pickable::IGNORE!
   ));
   ```
   对比单位子实体（[src/character/spawn.rs](file:///d:/Code/Bevy/Fre/src/character/spawn.rs) L237）：
   ```rust
   Pickable::IGNORE,  // 子实体不拦截鼠标事件
   ```

2. **Bevy picking 机制**：Sprite 组件默认可被 picking 系统拾取。TileSprite 有 Sprite 组件但无 Pickable::IGNORE，因此会拦截鼠标事件。

3. **事件交替触发**：当鼠标在单位上方时：
   - 指针同时命中 Unit sprite（z=1.0）和 TileSprite（z=0.0）
   - Bevy picking 按 z-order 优先选择最上层（Unit），触发 Pointer<Over> → HoveredEntity = Some(unit)
   - 但如果 Unit sprite 的命中区域与 TileSprite 有微小偏移，或帧间鼠标微移，picking 可能交替选择两者
   - TileSprite 不是 Unit，不触发 on_unit_pointer_over，但触发 on_unit_pointer_out → HoveredEntity = None
   - 下一帧又选中 Unit → HoveredEntity = Some(unit) → 循环

4. **第二列特异性**：闪烁仅在特定列出现，可能是因为这些位置的 TileSprite 与 Unit sprite 的 z-order 或命中区域存在特殊重叠。第三列及右侧未出现，可能是单位位置与格子对齐更精确。

5. **面板更新链**：
   - HoveredEntity 变化 → `update_selected_unit_view` 重建 SelectedUnitView
   - SelectedUnitView 变化 → `update_unit_info` 重绘面板
   - 每帧交替 → 面板在"显示单位信息"和"空状态"之间闪烁

### 建议解决方案

**方案A（推荐）：为 TileSprite 添加 Pickable::IGNORE**

在 [src/map/grid.rs](file:///d:/Code/Bevy/Fre/src/map/grid.rs) 的 spawn_map 函数中，为每个 TileSprite 实体添加 `Pickable::IGNORE`：

```rust
commands.spawn((
    Sprite::from_color(terrain_color, Vec2::splat(tile_size - 2.0)),
    Transform::from_xyz(world_pos.x, world_pos.y, 0.0),
    TileSprite,
    Pickable::IGNORE,  // 地形格子不拦截鼠标事件
));
```

**方案B（补充）：添加 HoveredEntity 防抖**

在 `on_unit_pointer_over` 和 `on_unit_pointer_out` 中添加延迟检测，避免高频切换。但这会增加复杂度，且不治本。

### 涉及文件

- [src/map/grid.rs](file:///d:/Code/Bevy/Fre/src/map/grid.rs) — `spawn_map` (L126-L147)
- [src/input.rs](file:///d:/Code/Bevy/Fre/src/input.rs) — `on_unit_pointer_over` (L75-L84), `on_unit_pointer_out` (L88-L92)
- [src/ui/view_models.rs](file:///d:/Code/Bevy/Fre/src/ui/view_models.rs) — `update_selected_unit_view` (L177-L230)
- [src/ui/panels/unit_info.rs](file:///d:/Code/Bevy/Fre/src/ui/panels/unit_info.rs) — `update_unit_info` (L107-L436)

---

## 问题4：棋子移动路径异常（斜角飞行）

### 问题描述

选择己方棋子移动至斜角格子时，棋子未按直角折线路径移动，而是以斜角直线"飞行"至目标位置。

### 可能原因分析

1. **reconstruct_path 返回不完整路径** — 仅包含起点和终点，缺少中间路径点
2. **路径回溯算法逻辑错误** — 成本计算与 BFS 不一致
3. **animate_movement 插值逻辑问题** — 未按路径点逐段插值
4. **BFS 四方向限制未生效** — 寻路时允许了对角线移动

### 初步定位结果

**根因：reconstruct_path 在某些情况下返回不完整路径（仅含终点），animate_movement 从起点直线插值到终点**

定位路径：

1. **路径回溯的脆弱性**（[src/map/pathfinding/algorithms.rs](file:///d:/Code/Bevy/Fre/src/map/pathfinding/algorithms.rs) L10-L76）：
   ```rust
   // 核心条件：精确匹配剩余移动力
   if prev_remaining == remaining + cost && prev_remaining > best_remaining {
       best_prev = Some(prev);
       best_remaining = prev_remaining;
   }
   ```
   问题点：
   - **精确匹配条件**：`prev_remaining == remaining + cost` 要求完全相等。如果 BFS 和回溯之间的成本计算有任何差异，此条件永远不满足。
   - **提前终止**：`match best_prev { None => break }` — 找不到前驱时直接终止，返回不完整路径。
   - **方向优先级**：`prev_remaining > best_remaining` 使用严格大于，同剩余力的多个前驱中只选第一个，可能跳过正确路径。

2. **BFS 与回溯的成本一致性**：
   - BFS 中：`cost = calculator.cost(terrain_grid.get(next), ...)` — 计算**目标格**的地形成本
   - 回溯中：`cost = calculator.cost(terrain_grid.get(current), ...)` — 计算**当前格**的地形成本
   - 在回溯方向上，`current` 是目标格，`prev` 是来源格。从 `prev` 移动到 `current` 的成本应为 `current` 的地形成本。代码中 `terrain_grid.get(current)` 获取的是当前格地形，逻辑上正确。
   - 但如果 `TerrainCostCalculator` 对同一地形返回不同值（如基于标签的动态成本），且回溯时标签状态与 BFS 时不同，则会出现不一致。

3. **animate_movement 的行为**（[src/character/movement.rs](file:///d:/Code/Bevy/Fre/src/character/movement.rs) L86-L158）：
   当 `MovingUnit.path` 仅含一个元素 `[target]` 时：
   - `current_index = 0`, `target_coord = target`
   - `start_coord = gp.coord`（单位当前位置）
   - 线性插值从当前位置直接到目标位置 → **斜角直线飞行**

4. **复现条件**：当起点和终点不在同一行/列，且路径回溯因成本不匹配而提前终止时，返回的路径仅含 `[target]`，导致斜角飞行。

### 建议解决方案

**方案A（推荐）：修复 reconstruct_path 的鲁棒性**

1. 将精确匹配改为范围匹配或容差匹配
2. 添加回退逻辑：当找不到前驱时，尝试曼哈顿距离最近的有效前驱
3. 路径验证：回溯完成后检查路径连续性（相邻点曼哈顿距离 ≤ 1），不连续时重新计算

**方案B（补充）：在 animate_movement 中添加路径完整性检查**

如果路径不连续（存在斜角跳跃），自动插入中间路径点，将斜角移动分解为两步直角移动。

**方案C（防御）：reconstruct_path 失败时使用 A* 回退**

当 BFS 回溯失败时，使用 A* 算法重新计算路径，确保始终返回有效的直角路径。

### 涉及文件

- [src/map/pathfinding/algorithms.rs](file:///d:/Code/Bevy/Fre/src/map/pathfinding/algorithms.rs) — `reconstruct_path` (L10-L76)
- [src/character/movement.rs](file:///d:/Code/Bevy/Fre/src/character/movement.rs) — `animate_movement` (L86-L158)
- [src/character/components.rs](file:///d:/Code/Bevy/Fre/src/character/components.rs) — `MovingUnit` (L161-L185)

---

## 问题5：移动轨迹显示缺失

### 问题描述

棋子移动过程中，应显示表示移动痕迹的线条和箭头，但目前该功能缺失。

### 可能原因分析

1. **spawn_path_arrows 未被调用** — 移动时未触发路径箭头生成
2. **路径为空或过短** — 仅 1 个元素的路径不生成线段
3. **渲染层级问题** — 箭头被地形或单位遮挡
4. **颜色/透明度问题** — 箭头颜色与背景融合不可见
5. **箭头被立即清除** — 生成和清除在同一帧执行

### 初步定位结果

**主因：路径过短（仅含终点）导致不生成线段 + 可能的渲染层级问题**

定位路径：

1. **spawn_path_arrows 的生成条件**（[src/character/movement.rs](file:///d:/Code/Bevy/Fre/src/character/movement.rs) L19-L50）：
   ```rust
   // 为每对相邻路径点生成线段
   for i in 0..world_points.len().saturating_sub(1) {
       spawn_line_segment(commands, world_points[i], world_points[i + 1]);
   }
   ```
   当路径仅含 1 个元素时，`world_points.len().saturating_sub(1) = 0`，不生成任何线段。

2. **与问题4的关联**：由于 reconstruct_path 返回不完整路径（仅含终点），spawn_path_arrows 收到的 path 参数可能只有 1 个元素，无法生成线段和箭头。

3. **调用点确认**：
   - 玩家移动：[src/ui/command_handler.rs](file:///d:/Code/Bevy/Fre/src/ui/command_handler.rs) L133 — `spawn_path_arrows(&mut commands, &map, &path)` ✅ 已调用
   - AI 移动：[src/ai/decision.rs](file:///d:/Code/Bevy/Fre/src/ai/decision.rs) L196 — `spawn_path_arrows(&mut commands, &map, &path)` ✅ 已调用

4. **渲染参数**：
   - 线段颜色：`Color::srgba(1.0, 1.0, 0.4, 0.7)` — 半透明黄色，在绿色草地上应可见
   - 线段宽度：4.0px
   - z-order：0.5（地形 z=0.0，单位 z=1.0+）— 应在地形之上、单位之下

5. **箭头清除时机**（[src/character/movement.rs](file:///d:/Code/Bevy/Fre/src/character/movement.rs) L147-L152）：
   移动完成后才清除箭头，不存在"生成后立即清除"的问题。

### 建议解决方案

**方案A（依赖问题4修复）：修复路径生成后，轨迹自然显示**

问题4修复后，reconstruct_path 返回完整路径，spawn_path_arrows 即可正常生成线段和箭头。

**方案B（独立优化）：增强轨迹可见性**

1. 增加线段宽度（4.0 → 6.0）
2. 提高不透明度（0.7 → 0.9）
3. 为线段添加发光效果或边框
4. 在每个路径点添加小圆点标记

**方案C（功能增强）：添加移动后轨迹保留**

当前箭头在移动完成后立即清除。可考虑保留轨迹一段时间（如 0.5 秒渐隐），增强视觉反馈。

### 涉及文件

- [src/character/movement.rs](file:///d:/Code/Bevy/Fre/src/character/movement.rs) — `spawn_path_arrows` (L19-L50), `animate_movement` (L86-L158)
- [src/ui/command_handler.rs](file:///d:/Code/Bevy/Fre/src/ui/command_handler.rs) — `handle_ui_commands` (L133)
- [src/ai/decision.rs](file:///d:/Code/Bevy/Fre/src/ai/decision.rs) — `enemy_ai_system` (L196)

---

## 问题6：相机视口移动交互评估

### 问题描述

当前相机视口移动跟随鼠标位置（边缘滚动），用户建议改为按住右键拖动鼠标移动相机视口。

### 当前实现分析

当前相机控制（[src/ui/camera.rs](file:///d:/Code/Bevy/Fre/src/ui/camera.rs)）支持以下输入方式：

| 方式 | 实现 | 体验评价 |
|------|------|----------|
| WASD/方向键 | `keyboard.pressed(KeyCode::KeyW)` 等 | ✅ 标准操作，体验良好 |
| 边缘滚动 | 鼠标距屏幕边缘 30px 内触发 | ⚠️ 容易误触发，不够直观 |
| 滚轮缩放 | `MouseWheel` 事件 | ✅ 标准操作 |
| Space 聚焦 | 聚焦当前行动单位 | ✅ 便捷功能 |

**边缘滚动的问题**：
- 鼠标在屏幕边缘移动时，相机持续移动，给人"相机跟随鼠标"的感觉
- 在小窗口或高分辨率下，30px 的触发区域容易被误触
- SRPG 游戏中，鼠标常用于选择单位和格子，边缘滚动会干扰精确操作

### 评估结论

**建议采纳：添加右键拖动控制，保留边缘滚动作为可选项**

理由：
1. **SRPG 惯例**：火焰纹章、FFT、三角战略等 SRPG 均使用右键/中键拖动移动相机
2. **操作分离**：左键选择/确认，右键移动相机，操作语义清晰
3. **精确控制**：拖动提供精确的相机位移控制，不会误触发
4. **兼容性**：可同时保留边缘滚动（可配置开关），满足不同操作习惯

### 建议解决方案

**实现右键拖动相机控制**

在 `camera_control` 系统中添加：

```rust
// 右键拖动相机
if mouse_button.pressed(MouseButton::Right) {
    if let Some(delta) = window.cursor_position().and_then(|pos| {
        // 计算鼠标移动增量
        // 使用上一帧鼠标位置计算差值
    }) {
        transform.translation.x -= delta.x * drag_speed / scale;
        transform.translation.y += delta.y * drag_speed / scale;
        target.position = None; // 取消聚焦
    }
}
```

需要添加：
- `Local<Vec2>` 存储上一帧鼠标位置
- 拖动速度常量（建议 1.0，1:1 跟随鼠标）
- 右键拖动时取消相机聚焦目标

**可选：添加边缘滚动开关**

在设置中添加 `edge_scroll_enabled: bool` 配置，默认关闭。

### 涉及文件

- [src/ui/camera.rs](file:///d:/Code/Bevy/Fre/src/ui/camera.rs) — `camera_control` (L82-L155)

---

## 修复优先级与依赖关系

```
P0 (立即修复):
  ├── 问题1: 地形多样性缺失 → 修改 RON 文件，5 分钟
  ├── 问题3: 面板闪烁 → 添加 Pickable::IGNORE，5 分钟
  └── 问题4: 斜角飞行 → 修复 reconstruct_path，30-60 分钟

P1 (短期修复):
  ├── 问题5: 轨迹缺失 → 依赖问题4修复，验证后可能无需额外修改
  └── 问题2: 移动卡顿 → 调整 AI 计时器 + 性能分析，30 分钟

P2 (体验优化):
  └── 问题6: 相机控制 → 添加右键拖动，30 分钟
```

**依赖关系**：问题5的轨迹缺失主要由问题4的路径不完整导致，修复问题4后问题5大概率自动解决。

---

## 自检结果

| 检查项 | 结果 |
|--------|------|
| 架构合规 | PASS — 未发现架构违规 |
| 领域规则 | PASS — 诊断过程遵循领域边界 |
| 测试规范 | N/A — 诊断报告，不涉及测试代码 |
| 命名规范 | PASS |
| 可见性 | PASS |
| 错误处理 | PASS |
| 死代码 | PASS |
| 重复代码 | PASS |
