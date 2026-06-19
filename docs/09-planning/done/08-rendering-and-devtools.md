# 渲染变更与开发工具

> 注意：本项目是 2D SRPG，大部分 3D 渲染变更与项目无关。本文档仅做知识记录，标记优先级。

---

## 1. 渲染变更（C级 — 直接忽略）

### 1.1 Solari 改进

- 实时光线追踪渲染器改进
- 镜面和非金属材质修复
- 性能提升和时间稳定性增强
- **与 2D SRPG 无关**

### 1.2 后处理效果

#### Vignette

```rust
commands.spawn((Camera3d::default(), Vignette { intensity: 1.0, radius: 0.75, .. }));
```

- 经典镜头效果，可用于受伤时红色脉冲
- **2D 项目暂不需要，但未来战斗特效可能用到**

#### Lens Distortion

```rust
commands.spawn((Camera3d::default(), LensDistortion { intensity: 0.5, .. }));
```

- 桶形/枕形畸变
- **2D 项目暂不需要**

### 1.3 Render Recovery

- GPU 错误恢复机制
- `DeviceLost` → `Recover` / `OutOfMemory` → `StopRendering`
- **长期运行应用有价值，SRPG 暂时不需要**

### 1.4 Render Graph as Systems

- `RenderGraph` 被 `Schedule` 替代
- 渲染通道现在是普通 System
- **隐藏信号：Trait → Schedule → System 是 Bevy 官方方向**
- **启示：设计扩展点时优先用 System/Schedule/Observer，不要发明 Trait 框架**

### 1.5 Skinned Mesh Culling 改进

- 基于实际关节位置计算边界
- **2D 项目无关**

### 1.6 Partial Bindless

- Metal 平台部分 bindless 支持
- **2D 项目无关**

### 1.7 Parallax Corrected Cubemaps

- 室内环境反射修正
- **2D 项目无关**

### 1.8 White Furnace Test

- PBR 着色器正确性修复
- **2D 项目无关**

---

## 2. 开发工具

### 2.1 Diagnostics Overlay（S级 — 立即采用）

```rust
// FPS 显示
commands.spawn(DiagnosticsOverlay::fps());

// 网格和标准材质统计
commands.spawn(DiagnosticsOverlay::mesh_and_standard_materials());
```

自定义诊断面板：

```rust
commands.spawn(DiagnosticsOverlay::new("MyDiagnostics", vec![
    DiagnosticsOverlayItem {
        path: MyDiagnostics::COUNTER,
        statistic: DiagnosticsOverlayStatistic::Value,
        precision: 4,
    }
]));
```

统计模式：

| 模式 | 说明 |
|------|------|
| `SmoothedMovingAverage`（默认） | 平滑移动平均，适合帧率等波动指标 |
| `Value` | 最新值，适合计数器等瞬时指标 |
| `RawMovingAverage` | 原始移动平均，无平滑处理 |

**对项目的价值：**

- 以前需要 `FrameTimeDiagnosticsPlugin` + 自己写 UI
- 现在一行代码显示 FPS / Frame Time
- 建议作为 Debug Feature 开启，线上关闭

### 2.2 Text Gizmos（B级 — 调试时使用）

```rust
fn draw_text(mut gizmos: Gizmos) {
    gizmos.text_2d(
        Isometry2d::IDENTITY,
        "Hello Bevy",
        40.0,
        Vec2::ZERO,
        Color::WHITE,
    );
}
```

- 仅支持 ASCII
- 固定字体
- 严格用于开发调试
- 不适合游戏内文字（用 `Text2D`）

### 2.3 Transform Gizmo（C级 — 编辑器阶段）

- 3D 视口中的平移/旋转/缩放手柄
- 需要 `TransformGizmoPlugin` + `TransformGizmoCamera` + `TransformGizmoFocus`
- **2D 项目暂不需要，等编辑器阶段**

### 2.4 Infinite Grid（C级 — 编辑器阶段）

- 透明地面网格
- `InfiniteGridPlugin` + `InfiniteGrid` 组件
- `InfiniteGridSettings` 可配置颜色/淡出距离/线条比例
- **2D 项目暂不需要，等编辑器阶段**

### 2.5 AccessibleLabel（B级 — 可访问性）

- 将 a11y label 从 `AccessibilityNode` 中分离
- 可在 BSN 模板中作为 mixin 使用
- **项目后期可访问性阶段再考虑**

---

## 3. 渲染变更的隐藏架构信号

### 3.1 Trait → System 收敛

`RenderGraph` 从 Trait Node 迁移到 System，说明 Bevy 官方方向：

- ✅ 优先：System / Schedule / Observer
- ❌ 避免：Trait Executor / Trait Pipeline / Trait Node

### 3.2 工具开发第一公民

`EditableText` + `TransformGizmo` + `InfiniteGrid` + `Feathers` + `Settings` + `BSN` —— 全部在补编辑器能力。

说明 Bevy 正在从"游戏引擎"向"游戏引擎 + 编辑器框架"演化。

### 3.3 对项目的启示

- 未来做编辑器时，尽量等待官方方案
- 不要现在造编辑器轮子
- 扩展点设计用 System/Schedule/Observer，不用 Trait

---

## 4. 项目迁移建议

### 4.1 立即采用

| 变更 | 优先级 | 说明 |
|------|--------|------|
| DiagnosticsOverlay | S | 开发调试用，Feature Flag 控制 |

### 4.2 调试时使用

| 变更 | 优先级 | 说明 |
|------|--------|------|
| Text Gizmos | B | 调试时临时标注 |

### 4.3 直接忽略

| 变更 | 原因 |
|------|------|
| 所有 3D 渲染变更 | 2D 项目不涉及 |
| 编辑器工具（Transform Gizmo / Infinite Grid） | 等编辑器阶段 |
| 后处理效果 | 2D 暂不需要 |

### 4.4 未来再评估

| 变更 | 评估时机 |
|------|----------|
| Render Recovery | 长期运行场景 |
| Vignette / Lens Distortion | 战斗特效 |
| AccessibleLabel | 可访问性阶段 |
