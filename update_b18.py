#!/usr/bin/env python3
import re

CATALOG_PATH = "/Users/lf380/Code/Bevy/a1/bevy_examples_catalog.md"

GIZMOS_196 = ("196", "调试绘图", "`2d_gizmos.rs`")
GIZMOS_197 = ("197", "调试绘图", "`3d_gizmos.rs`")
GIZMOS_198 = ("198", "调试绘图", "`axes.rs`")
GIZMOS_199 = ("199", "调试绘图", "`light_gizmos.rs`")
DIAG_200 = ("200", "诊断", "`log_diagnostics.rs`")
DIAG_201 = ("201", "诊断", "`custom_diagnostic.rs`")
DIAG_202 = ("202", "诊断", "`enabling_disabling_diagnostic.rs`")
DEV_203 = ("203", "开发者工具", "`fps_overlay.rs`")
WIN_204 = ("204", "窗口管理", "`window_settings.rs`")
WIN_205 = ("205", "窗口管理", "`clear_color.rs`")
WIN_206 = ("206", "窗口管理", "`window_resizing.rs`")
WIN_207 = ("207", "窗口管理", "`transparent_window.rs`")
WIN_208 = ("208", "窗口管理", "`window_drag_move.rs`")
WIN_209 = ("209", "窗口管理", "`low_power.rs`")
WIN_210 = ("210", "窗口管理", "`scale_factor_override.rs`")
WIN_211 = ("211", "窗口管理", "`screenshot.rs`")
WIN_212 = ("212", "窗口管理", "`multiple_windows.rs`")
WIN_213 = ("213", "窗口管理", "`multi_window_text.rs`")
WIN_214 = ("214", "窗口管理", "`monitor_info.rs`")
WIN_215 = ("215", "窗口管理", "`custom_cursor_image.rs`")
PICK_216 = ("216", "拾取交互", "`simple_picking.rs`")
PICK_217 = ("217", "拾取交互", "`mesh_picking.rs`")
PICK_218 = ("218", "拾取交互", "`sprite_picking.rs`")
PICK_219 = ("219", "拾取交互", "`dragdrop_picking.rs`")
PICK_220 = ("220", "拾取交互", "`debug_picking.rs`")


def row(meta, intent, knowledge, examples):
    seq, cat, fname = meta
    if knowledge.startswith("知：<br>"):
        knowledge = knowledge[len("知：<br>") :]
    knowledge = knowledge.rstrip("。")
    if not knowledge.endswith("。"):
        knowledge += "。"
    if examples.startswith("例：<br>"):
        examples = examples[len("例：<br>") :]
    examples = examples.rstrip("。")
    if not examples.endswith("。"):
        examples += "。"
    return "| {} | {} | {} | {} | 知：<br>{}<br>例：<br>{} |".format(
        seq, cat, fname, intent, knowledge, examples
    )


def main():
    with open(CATALOG_PATH, "r", encoding="utf-8") as f:
        content = f.read()

    lines = content.split("\n")

    start_idx = None
    for i, line in enumerate(lines):
        stripped = line.strip()
        if (
            stripped.startswith("|")
            and "`2d_gizmos.rs`" in stripped
            and "调试绘图" in stripped
        ):
            start_idx = i
            break

    if start_idx is None:
        print("ERROR: Could not find entry for 2d_gizmos.rs")
        return

    end_idx = None
    found_last = False
    for i in range(start_idx, len(lines)):
        stripped = lines[i].strip()
        if (
            stripped.startswith("|")
            and "`debug_picking.rs`" in stripped
            and "拾取交互" in stripped
        ):
            found_last = True
        elif found_last:
            end_idx = i
            break

    if end_idx is None:
        print("ERROR: Could not find end entry for debug_picking.rs")
        return

    print(f"Found entries at lines {start_idx + 1} to {end_idx + 1}")

    new_lines = []

    # 196 - 2D Gizmos
    new_lines.append(
        row(
            GIZMOS_196,
            (
                "**意图**：2D 即时模式绘图 API——线条/射线/网格/三角形/矩形/圆弧/箭头/曲线/十字等调试图元；"
                "自定义 GizmoConfigGroup 分组独立控制。⭐⭐<br>"
                "- **Gizmos 核心 API**：<br>"
                "  - gizmos.line_2d(start, end, color) 绘制线段；<br>"
                "  - gizmos.ray_2d(start, direction, color) 绘制射线；<br>"
                "  - gizmos.grid_2d(iso, cells, spacing, color) 绘制网格并支持 .outer_edges()；<br>"
                "  - gizmos.rect_2d / cross_2d / circle_2d / ellipse_2d / arc_2d 绘制 2D 图元；<br>"
                "  - gizmos.arrow_2d 支持 .with_double_end() .with_tip_length() 建造器模式；<br>"
                "  - gizmos.curve_gradient_2d / linestrip_gradient_2d 渐变曲线/线带；<br>"
                "  - gizmos.rounded_rect_2d 带 corner_radius 圆角矩形；<br>"
                "- **自定义 ConfigGroup**：<br>"
                "  - #[derive(GizmoConfigGroup)] + init_gizmo_group 注册独立配置组；<br>"
                "  - Gizmos&lt;MyGizmos&gt; 独立于默认组，可单独开关/设线宽；<br>"
                "  - ⚠️ config_store.config_mut 通过元组 (config, line_config) 访问；<br>"
                "- **线样式与连接**：<br>"
                "  - GizmoLineStyle::Solid / Dotted / Dashed { gap_scale, line_scale } 三种线样式；<br>"
                "  - GizmoLineJoint::Bevel / Miter / Round(n) / None 四种线连接；<br>"
                "  - ⚠️ config.enabled ^= true 使用位异或切换布尔值；<br>"
                "- **时间暂停**：<br>"
                "  - Time&lt;Virtual&gt; 支持 pause/unpause 暂停虚拟时间；<br>"
                "  - ⚠️ Space 键切换暂停，gizmos 动画冻结但 UI 仍可交互；"
            ),
            "知：<br>- Gizmos 即时绘图概念；<br>- Isometry2d 恒等变换；<br>- Interval::EVERYWHERE + FunctionCurve 曲线定义。",
            "例：<br>- 2D/2d_shapes.rs(12)。",
        )
    )

    # 197 - 3D Gizmos
    new_lines.append(
        row(
            GIZMOS_197,
            (
                "**意图**：3D 即时模式绘图——网格/球体/矩形/十字/射线/圆弧/圆/箭头/圆角立方体；"
                "支持 GizmoAsset 静态绘制和 GizmoConfigStore 遍历。⭐⭐<br>"
                "- **3D 图元 API**：<br>"
                "  - gizmos.grid / sphere / primitive_3d / rect / cube / cross / ray / circle / arrow 3D 版本；<br>"
                "  - gizmos.curve_gradient_3d 3D 渐变曲线；<br>"
                "  - my_gizmos.rounded_cuboid(min, max, color) .edge_radius() .arc_resolution()；<br>"
                "- **GizmoAsset 高性能模式**：<br>"
                "  - GizmoAsset::new() 创建资产，gizmo_assets.add(gizmo) 存储；<br>"
                "  - commands.spawn(Gizmo { handle, line_config }) 作为组件渲染；<br>"
                "  - ⚠️ 大量静态线用 GizmoAsset 比每帧 Gizmos 参数高效得多；<br>"
                "- **depth_bias 深度偏移**：<br>"
                "  - config.depth_bias = -1. 让 Gizmos 绘制在所有物体之上；<br>"
                "  - config.line.perspective ^= true 切换线透视缩放；<br>"
                "- **AABB 调试**：<br>"
                "  - AabbGizmoConfigGroup.config_mut().draw_all ^= true 全局显示所有 AABB；<br>"
                "  - 🧩 config_store.iter_mut() 可同时修改所有配置组的 depth_bias；"
            ),
            "知：<br>- Isometry3d 恒等变换；<br>- FunctionCurve 3D 参数曲线。",
            "例：<br>- gizmos/2d_gizmos.rs(206)；<br>- 3D/3d_scene.rs(32)。",
        )
    )

    # 198 - Axes
    new_lines.append(
        row(
            GIZMOS_198,
            (
                "**意图**：使用 gizmos.axes(transform, length) 可视化实体坐标轴；"
                "配合 Aabb 自动缩放轴长度；演示 Transform 插值和确定性随机。⭐⭐<br>"
                "- **axes Gizmo**：<br>"
                "  - gizmos.axes(transform, length) 绘制 RGB 三色轴(X=红 Y=绿 Z=蓝)；<br>"
                "  - length = aabb.half_extents.length() 轴长度自适应实体包围盒；<br>"
                "- **Transform 插值**：<br>"
                "  - translation: Vec3::lerp 线性插值位置；<br>"
                "  - rotation: Quat::slerp 球面线性插值旋转；<br>"
                "  - scale: elerp 指数插值缩放（对数空间线性插值后 exp2 回到线性空间）；<br>"
                "  - 🔧 elerp 保证缩放始终为正数，避免负缩放翻转法线；<br>"
                "- **确定性随机**：<br>"
                "  - ChaCha8Rng::seed_from_u64(固定种子) 确保每次运行结果一致；<br>"
                "  - 🔧 SeededRng 作为 Resource 注入系统，随机变换可复现；<br>"
                "- **TransitionDuration 模式**：<br>"
                "  - progress 秒数累加到 TRANSITION_DURATION 后切换目标；<br>"
                "  - ⚠️ tracking.progress = 0.0 重置而非 += 避免精度漂移；"
            ),
            "知：<br>- Aabb 包围盒；<br>- ChaCha8Rng 确定性 RNG。",
            "例：<br>- gizmos/3d_gizmos.rs(207)；<br>- math/bounding_2d.rs(329)。",
        )
    )

    # 199 - Light Gizmos
    new_lines.append(
        row(
            GIZMOS_199,
            (
                "**意图**：使用 LightGizmoConfigGroup 可视化 PointLight/SpotLight/DirectionalLight 的范围、方向和颜色；"
                "支持四种着色模式切换。⭐⭐<br>"
                "- **LightGizmoConfigGroup**：<br>"
                "  - config_store.config_mut 获取灯光 Gizmo 配置；<br>"
                "  - light_config.draw_all = true 显示所有灯光 Gizmo；<br>"
                "  - LightGizmoColor 枚举四种着色模式：Manual / Varied / MatchLightColor / ByLightType；<br>"
                "- **灯光类型可视化**：<br>"
                "  - PointLight：球体范围 + 向下射线；<br>"
                "  - SpotLight：锥体（outer_angle/inner_angle）+ 方向射线；<br>"
                "  - DirectionalLight：平行光方向射线；<br>"
                "- **TextUiWriter 动态更新**：<br>"
                "  - commands.entity(e).despawn_related::&lt;Children&gt;() 清除子节点重建；<br>"
                "  - writer.text(entity, span_index) 按 span 索引更新富文本；<br>"
                "  - 🧩 TextSpan 作为子实体构建富文本，每个 span 独立 TextColor；<br>"
                "- **rotate_around 旋转相机**：<br>"
                "  - transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(dt)) 绕原点轨道旋转；"
            ),
            "知：<br>- LightGizmoConfigGroup；<br>- PointLight / SpotLight / DirectionalLight。",
            "例：<br>- 3D/lighting.rs(35)；<br>- gizmos/3d_gizmos.rs(207)。",
        )
    )

    # 200 - Log Diagnostics
    new_lines.append(
        row(
            DIAG_200,
            (
                "**意图**：通过 LogDiagnosticsPlugin + 诊断插件组合将 FPS / 帧时间 / 实体数 / 系统信息输出到控制台；"
                "支持运行时过滤诊断类别。⭐⭐<br>"
                "- **诊断插件组合**：<br>"
                "  - LogDiagnosticsPlugin::default() 定期打印诊断到 console；<br>"
                "  - FrameTimeDiagnosticsPlugin::default() FPS + 帧计数 + 帧时间；<br>"
                "  - EntityCountDiagnosticsPlugin::default() 实体总数；<br>"
                "  - SystemInformationDiagnosticsPlugin 进程/系统 CPU 和内存使用率；<br>"
                "  - RenderDiagnosticsPlugin 渲染层诊断（Verbose）；<br>"
                "  - ⚠️ 诊断插件必须在 DefaultPlugins 之后添加（依赖 Time 插件）；<br>"
                "- **LogDiagnosticsState 过滤**：<br>"
                "  - enable_filtering() / disable_filtering() 切换过滤模式；<br>"
                "  - extend_filter(diagnostics) / remove_filter 增删诊断路径；<br>"
                "- **Resource 条件运行**：<br>"
                "  - .run_if(resource_exists_and_changed.or(...)) 仅状态变化时更新 UI；<br>"
                "  - 🔧 .or() 组合两个条件，任一满足即运行；<br>"
                "- **UI 状态面板**：<br>"
                "  - LogDiagnosticsStatus / LogDiagnosticsFilters 作为 Resource 管理过滤状态；<br>"
                "  - children! 宏简化 UI 子节点声明；"
            ),
            "知：<br>- LogDiagnosticsPlugin / FrameTimeDiagnosticsPlugin；<br>- DiagnosticPath 诊断路径。",
            "例：<br>- app/empty_defaults.rs(109)。",
        )
    )

    # 201 - Custom Diagnostic
    new_lines.append(
        row(
            DIAG_201,
            (
                "**意图**：使用 Diagnostic::new(path).with_suffix() + register_diagnostic 注册自定义诊断指标，"
                "通过 Diagnostics::add_measurement 写入数据。⭐<br>"
                "- **自定义诊断注册**：<br>"
                "  - DiagnosticPath::const_new 定义唯一路径；<br>"
                '  - Diagnostic::new(path).with_suffix(" iterations") 创建诊断并设后缀；<br>'
                "  - .register_diagnostic() 在 App 装配时注册；<br>"
                "  - ⚠️ 必须在 add_systems 之前注册，否则 add_measurement 无效；<br>"
                "- **数据写入**：<br>"
                "  - diagnostics.add_measurement(&amp;path, || 10.0) 每次系统运行写入一个测量值；<br>"
                "  - LogDiagnosticsPlugin 自动消费并打印；<br>"
                "  - 🔧 闭包返回值即为本次测量值，可来自任意计算；<br>"
                "- **最小示例**：<br>"
                "  - 整个示例仅 30 行，展示了从注册到输出的完整流程；"
            ),
            "知：<br>- DiagnosticPath；<br>- LogDiagnosticsPlugin。",
            "例：<br>- diagnostics/log_diagnostics.rs(210)。",
        )
    )

    # 202 - Enabling Disabling Diagnostic
    new_lines.append(
        row(
            DIAG_202,
            (
                "**意图**：通过 DiagnosticsStore::iter_mut() + diag.is_enabled 切换诊断开关；"
                "配合 on_timer 定时触发。⭐<br>"
                "- **DiagnosticsStore 操作**：<br>"
                "  - ResMut&lt;DiagnosticsStore&gt; 获取诊断存储资源；<br>"
                "  - store.iter_mut() 遍历所有已注册诊断；<br>"
                "  - diag.is_enabled = !diag.is_enabled 布尔切换开关；<br>"
                "- **on_timer 条件**：<br>"
                "  - .run_if(on_timer(Duration::from_secs_f32(10.0))) 每 10 秒触发一次；<br>"
                "  - ⚠️ on_timer 来自 bevy::time::common_conditions，非每帧运行；<br>"
                "- **运行时行为**：<br>"
                "  - 禁用后诊断仍注册但不收集数据，零开销；<br>"
                "  - 重新启用后恢复收集，历史数据保留；<br>"
                "  - 🧩 适合性能敏感场景按需开关诊断；"
            ),
            "知：<br>- DiagnosticsStore；<br>- on_timer 条件运行。",
            "例：<br>- diagnostics/log_diagnostics.rs(210)。",
        )
    )

    # 203 - FPS Overlay
    new_lines.append(
        row(
            DEV_203,
            (
                "**意图**：FpsOverlayPlugin 一行代码显示 FPS 覆盖层；"
                "支持实时修改字体大小 / 颜色 / 可见性 / 帧时间图。⭐⭐<br>"
                "- **FpsOverlayPlugin 配置**：<br>"
                "  - FpsOverlayConfig { text_config, text_color, refresh_interval, enabled, frame_time_graph_config } 完整配置；<br>"
                "  - TextFont { font_size: 42.0 } 控制覆盖层字号；<br>"
                "  - refresh_interval: Duration::from_millis(100) 刷新间隔；<br>"
                "- **FrameTimeGraphConfig 帧时间图**：<br>"
                "  - enabled: true 显示实时帧时间折线图；<br>"
                "  - min_fps / target_fps 控制图的 Y 轴范围和基准线；<br>"
                "  - 🔧 target_fps 设为目标帧率（如 144），低于此值的帧用红色标记；<br>"
                "- **运行时修改**：<br>"
                "  - ResMut&lt;FpsOverlayConfig&gt; 直接修改 Resource 即生效；<br>"
                "  - overlay.text_color / text_config.font_size / enabled 实时变化；<br>"
                "  - ⚠️ 需要 Camera2d 或 Camera3d 才能显示覆盖层；<br>"
                "- **Color 常量**：<br>"
                "  - impl OverlayColor { const RED/GREEN: Color = Color::srgb(...) } 关联常量定义颜色；<br>"
                "  - 🔧 const Color 要求在 impl 块内声明，不能用 let 绑定；"
            ),
            "知：<br>- FpsOverlayPlugin / FpsOverlayConfig；<br>- FrameTimeGraphConfig。",
            "例：<br>- diagnostics/log_diagnostics.rs(210)。",
        )
    )

    # 204 - Window Settings
    new_lines.append(
        row(
            WIN_204,
            (
                "**意图**：全面演示窗口设置——标题 / PresentMode / 主题 / 光标 / 按钮 / 层级 / 可见延迟 / 光标图标切换。⭐⭐⭐<br>"
                "- **Window 组件配置**：<br>"
                "  - title / name / resolution / present_mode / window_theme 完整设置；<br>"
                "  - enabled_buttons { maximize, minimize, close } 禁用窗口按钮；<br>"
                "  - visible: false 初始隐藏，延迟 3 帧后 make_visible 避免白屏；<br>"
                "  - ⚠️ WindowPlugin { primary_window: Some(Window {...}) } 必须通过 Plugin 设置；<br>"
                "- **PresentMode 切换**：<br>"
                "  - AutoVsync（垂直同步）vs AutoNoVsync（无垂直同步/最高帧率）；<br>"
                "- **CursorOptions 光标控制**：<br>"
                "  - visible: bool 控制光标显示/隐藏；<br>"
                "  - grab_mode: CursorGrabMode::None/Locked/Confined 三种抓取模式；<br>"
                "- **光标图标切换**：<br>"
                "  - CursorIcon 枚举（Default/Pointer/Wait/Text 等）或 CustomCursor::Image；<br>"
                "  - CursorIcons Resource 存储图标列表，左/右键循环切换；<br>"
                "- **WindowLevel 窗口层级**：<br>"
                "  - AlwaysOnBottom / Normal / AlwaysOnTop 三种层级；<br>"
                "  - ⚠️ 平台兼容性差异大，仅部分操作系统支持；<br>"
                "- **toggle_theme 主题切换**：<br>"
                "  - WindowTheme::Light / Dark 窗口暗色/亮色主题；"
            ),
            "知：<br>- Window 组件全字段；<br>- CursorIcon / CursorOptions。",
            "例：<br>- app/empty_defaults.rs(109)。",
        )
    )

    # 205 - Clear Color
    new_lines.append(
        row(
            WIN_205,
            (
                "**意图**：通过 ClearColor Resource 设置窗口背景清除颜色；支持运行时按空格切换。⭐<br>"
                "- **ClearColor 资源**：<br>"
                "  - .insert_resource(ClearColor(Color::srgb(0.5, 0.5, 0.9))) 初始背景色；<br>"
                "  - ResMut&lt;ClearColor&gt; 运行时修改 clear_color.0；<br>"
                "  - ⚠️ ClearColor 是 Resource 不是 Component，必须用 insert_resource；<br>"
                "- **像素未绘制区域**：<br>"
                "  - 未被任何 Camera 覆盖的像素保留 ClearColor 颜色；<br>"
                "  - 🔧 如果场景覆盖整个窗口，ClearColor 不可见；<br>"
                "- **最小示例**：<br>"
                "  - 仅 24 行，是理解 Bevy 窗口背景色最简示例；<br>"
                "  - 🧩 配合透明窗口(transparent_window.rs)时 ClearColor.alpha=0；"
            ),
            "知：<br>- ClearColor Resource。",
            "例：<br>- app/empty_defaults.rs(109)。",
        )
    )

    # 206 - Window Resizing
    new_lines.append(
        row(
            WIN_206,
            (
                "**意图**：通过 window.resolution.set(w,h) 程序化调整窗口大小；"
                "监听 WindowResized 事件响应尺寸变化。⭐⭐<br>"
                "- **窗口分辨率设置**：<br>"
                "  - window.resolution.set(x, y) 以像素为单位设置窗口尺寸；<br>"
                "  - ResolutionSettings Resource 存储 large/medium/small 三档预设；<br>"
                "- **WindowResized 事件**：<br>"
                "  - MessageReader&lt;WindowResized&gt; 监听窗口尺寸变化事件；<br>"
                "  - e.width / e.height 获取变化后的窗口尺寸；<br>"
                "  - 🔧 MessageReader 替代旧版 EventReader，用法类似；<br>"
                "- **动态 UI 文本**：<br>"
                "  - Single 精确查询文本实体；<br>"
                '  - text.0 = format!("{:.1} x {:.1}", w, h) 格式化显示分辨率；'
            ),
            "知：<br>- WindowResolution；<br>- WindowResized 事件。",
            "例：<br>- window/window_settings.rs(214)。",
        )
    )

    # 207 - Transparent Window
    new_lines.append(
        row(
            WIN_207,
            (
                "**意图**：设置 Window.transparent=true + decorations=false + ClearColor(NONE) 实现透明无边框窗口。⭐<br>"
                "- **透明窗口配置**：<br>"
                "  - transparent: true 允许 ClearColor 的 alpha 值生效；<br>"
                "  - decorations: false 隐藏标题栏和边框；<br>"
                "  - ClearColor(Color::NONE) 背景色完全透明；<br>"
                "  - ⚠️ 平台兼容性差异：macOS 用 PostMultiplied，Linux 用 PreMultiplied；<br>"
                "- **CompositeAlphaMode**：<br>"
                '  - #[cfg(target_os = "macos")] macOS 专用 CompositeAlphaMode::PostMultiplied；<br>'
                '  - #[cfg(target_os = "linux")] Linux 专用 CompositeAlphaMode::PreMultiplied；<br>'
                "- **使用场景**：<br>"
                "  - 桌面挂件、覆盖层、小部件等非标准窗口；<br>"
                "  - 🔧 配合 decorations=false 可实现完全自定义外观窗口；"
            ),
            "知：<br>- Window::transparent；<br>- CompositeAlphaMode。",
            "例：<br>- window/window_settings.rs(214)。",
        )
    )

    # 208 - Window Drag Move
    new_lines.append(
        row(
            WIN_208,
            (
                "**意图**：无装饰窗口通过 start_drag_move() / start_drag_resize() 实现鼠标拖拽移动和调整大小。⭐⭐<br>"
                "- **拖拽移动**：<br>"
                "  - window.start_drag_move() 在左键按下时触发窗口拖拽移动；<br>"
                "  - ⚠️ 必须在 MouseButton::Left just_pressed 后调用；<br>"
                "- **拖拽调整大小**：<br>"
                "  - window.start_drag_resize(CompassOctant) 指定 8 个方向之一；<br>"
                "  - CompassOctant::North/NorthEast/East/.../NorthWest 八个方位；<br>"
                "  - 🔧 DIRECTIONS 常量数组存储 8 个方向，索引循环切换；<br>"
                "- **LeftClickAction 状态机**：<br>"
                "  - Resource 枚举 Nothing/Move/Resize 控制左键行为；<br>"
                "  - A 键循环切换三种模式，S/D 键调整 resize 方向；<br>"
                "- **事件传播**：<br>"
                "  - event.propagate(false) 阻止事件继续传播到下层实体；"
            ),
            "知：<br>- Window::start_drag_move / start_drag_resize；<br>- CompassOctant。",
            "例：<br>- window/window_settings.rs(214)。",
        )
    )

    # 209 - Low Power
    new_lines.append(
        row(
            WIN_209,
            (
                "**意图**：WinitSettings::game() 连续渲染 vs desktop_app() 反应式低功耗；"
                "演示 RequestRedraw / WakeUp 事件触发重绘。⭐⭐<br>"
                "- **WinitSettings 两种模式**：<br>"
                "  - WinitSettings::game()：焦点时连续渲染，失焦时 60Hz 限帧；<br>"
                "  - WinitSettings::desktop_app()：焦点时仅输入事件触发更新，失焦时鼠标悬停触发；<br>"
                "  - 🔧 自定义 WinitSettings { focused_mode, unfocused_mode } 精细控制；<br>"
                "- **RequestRedraw 事件**：<br>"
                "  - MessageWriter&lt;RequestRedraw&gt; 发送重绘请求，强制下一帧更新；<br>"
                "  - 🔧 适合 desktop_app() 模式下播放 UI 动画时按需触发；<br>"
                "- **WinitUserEvent::WakeUp**：<br>"
                "  - event_loop_proxy.send_event(WakeUp) 从外部线程唤醒主线程；<br>"
                "- **UpdateMode 四种配置**：<br>"
                "  - Game / Application / ApplicationWithRequestRedraw / ApplicationWithWakeUp；<br>"
                "  - Space 键循环切换，帧计数器显示更新频率差异；"
            ),
            "知：<br>- WinitSettings / UpdateMode；<br>- RequestRedraw 事件。",
            "例：<br>- window/window_settings.rs(214)。",
        )
    )

    # 210 - Scale Factor Override
    new_lines.append(
        row(
            WIN_210,
            (
                "**意图**：通过 WindowResolution::with_scale_factor_override() 覆盖操作系统 DPI 缩放因子；"
                "运行时动态调整。⭐⭐<br>"
                "- **缩放因子设置**：<br>"
                "  - WindowResolution::new(500, 300).with_scale_factor_override(1.0) 初始覆盖；<br>"
                "  - window.resolution.set_scale_factor_override(Some(n) / None) 运行时切换；<br>"
                "  - ⚠️ factor=1.0 表示无缩放（物理像素=逻辑像素），2.0 表示 Retina 双倍；<br>"
                "- **scale_factor_override 读取**：<br>"
                "  - window.resolution.scale_factor_override() 返回 Option&lt;f32&gt;；<br>"
                "  - window.scale_factor() 返回最终生效的缩放因子（含默认值）；<br>"
                "  - 🔧 .xor(Some(1.0)) 在 Some↔None 间切换；<br>"
                "- **动态调整**：<br>"
                "  - ArrowUp/Down 调整缩放因子值，Enter 切换覆盖/默认；<br>"
                "  - .map(|n| (n - 1.0).max(1.0)) 保证最小值为 1.0；"
            ),
            "知：<br>- WindowResolution / scale_factor_override。",
            "例：<br>- window/window_settings.rs(214)。",
        )
    )

    # 211 - Screenshot
    new_lines.append(
        row(
            WIN_211,
            (
                "**意图**：Screenshot::primary_window() + observe(save_to_disk) 一行代码实现截图；"
                "支持自定义保存路径和 Capturing 状态光标反馈。⭐⭐<br>"
                "- **截图 API**：<br>"
                "  - commands.spawn(Screenshot::primary_window()) 创建截图实体；<br>"
                "  - .observe(save_to_disk(path)) 观察截图完成事件并保存到磁盘；<br>"
                '  - 🔧 保存路径 format!("./screenshot-{}.png", counter) 自动编号；<br>'
                "- **Capturing 状态检测**：<br>"
                "  - Query&lt;Entity, With&lt;Capturing&gt;&gt; 查询正在捕获的截图实体；<br>"
                "  - 有正在捕获的截图时显示 Progress 光标，完成后移除；<br>"
                "- **CursorIcon 反馈**：<br>"
                "  - CursorIcon::from(SystemCursorIcon::Progress) 设置忙碌光标；<br>"
                "  - commands.entity(window).remove::&lt;CursorIcon&gt;() 恢复默认光标；"
            ),
            "知：<br>- Screenshot 组件；<br>- save_to_disk Observer。",
            "例：<br>- 3D/render_to_texture.rs(56)。",
        )
    )

    # 212 - Multiple Windows
    new_lines.append(
        row(
            WIN_212,
            (
                "**意图**：创建第二个 Window 实体 + RenderTarget::Window(WindowRef::Entity) 实现多窗口 3D 渲染；"
                "UiTargetCamera 指定 UI 归属。⭐⭐<br>"
                "- **第二窗口创建**：<br>"
                '  - commands.spawn(Window { title: "Second window", ..default() }).id() 获取 Entity；<br>'
                "  - ⚠️ 不设置 primary_window 时默认窗口仍由 WindowPlugin 自动创建；<br>"
                "- **RenderTarget 绑定**：<br>"
                "  - RenderTarget::Window(WindowRef::Entity(second_window)) 将相机绑定到指定窗口；<br>"
                "  - 两个 Camera3d 分别渲染不同角度的同一场景；<br>"
                "- **UiTargetCamera**：<br>"
                "  - UiTargetCamera(first_window_camera) 指定 UI 文本渲染到哪个相机；<br>"
                "  - 🔧 多窗口时必须显式指定，否则 UI 可能渲染到错误窗口；<br>"
                "- **场景共享**：<br>"
                "  - SceneRoot / DirectionalLight 两个相机共享，无需重复创建；"
            ),
            "知：<br>- WindowRef / RenderTarget；<br>- UiTargetCamera。",
            "例：<br>- window/window_settings.rs(214)。",
        )
    )

    # 213 - Multi Window Text
    new_lines.append(
        row(
            WIN_213,
            (
                "**意图**：多窗口不同缩放因子下的 Text UI 和 Text2d 渲染；"
                "演示 RenderLayers 分层隔离和双窗口共享实体。⭐⭐<br>"
                "- **双窗口缩放差异**：<br>"
                "  - 主窗口 scale_factor_override=1.0（标准），副窗口 override=2.0（Retina）；<br>"
                "  - Text2d 实体同时属于两个 RenderLayer 时，布局用较高缩放因子生成；<br>"
                "- **RenderLayers 分层**：<br>"
                "  - primary camera 无 RenderLayers（默认 layer 0）；<br>"
                "  - secondary camera RenderLayers::layer(1) 仅渲染 layer 1 实体；<br>"
                "  - RenderLayers::from_layers(&amp;[0, 1]) 实体同时被两个相机渲染；<br>"
                "- **UiTargetCamera 与 IsDefaultUiCamera**：<br>"
                "  - 无 UiTargetCamera 的 UI 节点搜索 IsDefaultUiCamera 标记相机；<br>"
                "  - 副窗口 UI 必须显式 UiTargetCamera(secondary_camera)；<br>"
                "- **Text2d vs Text UI**：<br>"
                "  - Text UI (Node+Text) 仅被一个相机渲染，忽略 RenderLayers；<br>"
                "  - Text2d 受 Transform 影响，可被多个 RenderLayers 相机渲染；"
            ),
            "知：<br>- RenderLayers 分层；<br>- WindowResolution scale_factor_override。",
            "例：<br>- window/multiple_windows.rs(222)。",
        )
    )

    # 214 - Monitor Info
    new_lines.append(
        row(
            WIN_214,
            (
                "**意图**：监听 Monitor Added/Removed 事件，为每个显示器创建全屏窗口并显示硬件信息。⭐⭐<br>"
                "- **Monitor 组件**：<br>"
                "  - Query&lt;(Entity, &amp;Monitor), Added&lt;Monitor&gt;&gt; 检测新连接的显示器；<br>"
                "  - monitor.name / physical_width / physical_height / refresh_rate_millihertz / scale_factor；<br>"
                "- **多窗口全屏**：<br>"
                "  - WindowMode::Fullscreen(MonitorSelection::Entity(entity), VideoModeSelection::Current) 指定显示器全屏；<br>"
                "  - WindowPosition::Centered(MonitorSelection::Entity(entity)) 窗口居中于指定显示器；<br>"
                "  - ⚠️ primary_window: None 禁止自动创建主窗口；<br>"
                "- **RemovedComponents&lt;Monitor&gt;**：<br>"
                "  - 监听显示器断开事件，遍历 MonitorRef 组件关联，despawn 对应窗口和 UI 实体；<br>"
                "  - 🧩 MonitorRef(Entity) 标记组件关联窗口与显示器实体；<br>"
                "- **每显示器独立 UI**：<br>"
                "  - UiTargetCamera(camera) 将 UI 绑定到对应窗口相机；"
            ),
            "知：<br>- Monitor 组件；<br>- WindowMode::Fullscreen。",
            "例：<br>- window/window_settings.rs(214)。",
        )
    )

    # 215 - Custom Cursor Image
    new_lines.append(
        row(
            WIN_215,
            (
                "**意图**：CustomCursor::Image + TextureAtlas 实现自定义光标图集动画；"
                "支持 flip_x / flip_y / rect 裁剪切换。⭐⭐<br>"
                "- **CustomCursorImage 配置**：<br>"
                '  - handle: asset_server.load("cursors/...") 光标图像句柄；<br>'
                "  - texture_atlas: Some(TextureAtlas { layout, index }) 图集帧选择；<br>"
                "  - hotspot: (0, 0) 光标热点（点击位置）；<br>"
                "  - flip_x / flip_y 翻转光标图像；<br>"
                "  - rect: Option&lt;URect&gt; 裁剪图像子区域；<br>"
                "- **光标动画**：<br>"
                "  - AnimationConfig { first/last/increment/fps/frame_timer } 帧循环配置；<br>"
                "  - Timer::new(Duration::from_secs_f32(1.0/fps), TimerMode::Once) 单次计时器驱动帧切换；<br>"
                "- **Local 缓存**：<br>"
                "  - Local&lt;Option&lt;TextureAtlas&gt;&gt; 缓存上一次的 atlas 用于恢复；<br>"
                "  - image.texture_atlas.take() 取出并清空，Some(a) 保存；<br>"
                "  - 🔧 Local 在系统多次调用间保持状态，无需 Resource；<br>"
                "- **rect 裁剪切换**：<br>"
                "  - const SECTIONS 预定义 4 个区域 + None；<br>"
                "  - .iter().cycle().skip_while().nth(1) 循环查找下一个；"
            ),
            "知：<br>- CustomCursor / CustomCursorImage；<br>- TextureAtlas 图集动画。",
            "例：<br>- window/window_settings.rs(214)。",
        )
    )

    # 216 - Simple Picking
    new_lines.append(
        row(
            PICK_216,
            (
                "**意图**：最简 Picking 示例——MeshPickingPlugin + .observe(On&lt;Pointer&lt;Click/Drag/Over/Out&gt;&gt;) 实现点击生成立方体和拖拽旋转。⭐⭐<br>"
                "- **MeshPickingPlugin**：<br>"
                "  - .add_plugins((DefaultPlugins, MeshPickingPlugin)) 一行启用 Mesh 拾取；<br>"
                "  - ⚠️ 非默认插件，必须显式添加；<br>"
                "- **Observer 拾取事件**：<br>"
                "  - .observe(on_click_spawn_cube) 注册 Click 观察者；<br>"
                "  - On&lt;Pointer&lt;Click&gt;&gt; / On&lt;Pointer&lt;Drag&gt;&gt; / On&lt;Pointer&lt;Over&gt;&gt; / On&lt;Pointer&lt;Out&gt;&gt; 四种事件；<br>"
                "  - 🔧 Observer 可直接附加到任何实体，无需组件标记；<br>"
                "- **drag 旋转**：<br>"
                "  - drag.delta.x / drag.delta.y 获取拖拽偏移量；<br>"
                "  - transform.rotate_y(dx * 0.02) / rotate_x(dy * 0.02) 根据偏移旋转；<br>"
                "- **Text 颜色变化**：<br>"
                "  - Over 事件变 Cyan，Out 事件恢复 White；<br>"
                "  - Query&lt;&amp;mut TextColor&gt; 精确修改文本颜色；"
            ),
            "知：<br>- MeshPickingPlugin；<br>- On&lt;Pointer&lt;Click/Drag&gt;&gt;。",
            "例：<br>- 3D/3d_scene.rs(32)；<br>- ui/button.rs(232)。",
        )
    )

    # 217 - Mesh Picking
    new_lines.append(
        row(
            PICK_217,
            (
                "**意图**：3D Mesh 拾取——所有图元类型 + Extrusion 挤压体；"
                "泛型 Observer 切换材质高亮；PointerInteraction 调试可视化。⭐⭐⭐<br>"
                "- **MeshPickingSettings**：<br>"
                "  - require_markers=true 时需添加 Pickable 组件才能被拾取；<br>"
                "  - Pickable::IGNORE 禁用特定实体的拾取（如地面）；<br>"
                "  - 🔧 调试阶段用默认全拾取，生产环境用 require_markers 优化性能；<br>"
                "- **泛型 Observer 模式**：<br>"
                "  - update_material_on 返回 impl Fn 闭包；<br>"
                "  - 一个函数生成四种事件（Over/Out/Press/Release）的 Observer；<br>"
                "- **PointerInteraction 调试**：<br>"
                "  - Query&lt;&amp;PointerInteraction&gt; 获取所有指针交互状态；<br>"
                "  - interaction.get_nearest_hit() 返回 Option&lt;(Entity, Hit)&gt;；<br>"
                "  - hit.position / hit.normal 获取命中点位置和法线；<br>"
                "  - 🧩 Gizmos::sphere + arrow 可视化命中点和法线方向；<br>"
                "- **Extrusion 挤压体**：<br>"
                "  - Extrusion::new(Rectangle::default(), 1.) 将 2D 图元挤压为 3D；<br>"
                "  - 支持 Rectangle/Capsule2d/Annulus/Circle/Ellipse/RegularPolygon/Triangle2d；"
            ),
            "知：<br>- MeshPickingPlugin / Pickable；<br>- PointerInteraction / Hit。",
            "例：<br>- picking/simple_picking.rs(226)。",
        )
    )

    # 218 - Sprite Picking
    new_lines.append(
        row(
            PICK_218,
            (
                "**意图**：Sprite 和 Sprite Atlas 拾取——仅不透明像素可触发；"
                "演示 Anchor 9 种锚点的拾取行为差异。⭐⭐<br>"
                "- **Sprite 拾取特性**：<br>"
                "  - 默认仅不透明像素（alpha &gt; 0）可触发拾取事件；<br>"
                "  - 透明像素区域点击会穿透到下层实体；<br>"
                "  - 🔧 这是 sprite_picking 与 mesh_picking 的核心区别；<br>"
                "- **Anchor 锚点测试**：<br>"
                "  - 9 种 Anchor 枚举值（TOP_LEFT 到 BOTTOM_RIGHT）排列为 3x3 网格；<br>"
                "  - 每个锚点的 Sprite + 背景黑色方块直观对比拾取区域；<br>"
                "- **recolor_on 泛型 Observer**：<br>"
                "  - fn recolor_on(color) 返回 impl Fn 通用颜色切换；<br>"
                "  - Over/Cyan，Out/Red，Press/Blue，Release/Cyan 四种状态颜色；<br>"
                "- **Sprite Atlas 动画 + 拾取**：<br>"
                "  - AnimationIndices + AnimationTimer 驱动帧切换；<br>"
                "  - Pickable::default() 附加到 atlas 实体，动画期间持续可拾取；"
            ),
            "知：<br>- Sprite 拾取（不透明像素）；<br>- Anchor 枚举。",
            "例：<br>- picking/simple_picking.rs(226)；<br>- 2D/sprite_sheet.rs(10)。",
        )
    )

    # 219 - DragDrop Picking
    new_lines.append(
        row(
            PICK_219,
            (
                "**意图**：使用 DragStart / DragEnter / DragOver / DragLeave / DragDrop / DragEnd "
                "六种拾取事件实现完整拖放工作流。⭐⭐⭐<br>"
                "- **拖放事件流**：<br>"
                "  - DragStart → DragEnter → DragOver → DragDrop/DragLeave → DragEnd；<br>"
                "  - ⚠️ 事件顺序严格：DragStart 总在 DragEnter 之前；<br>"
                "- **GhostPreview 幽灵预览**：<br>"
                "  - DragEnter 时在 hit.position 生成半透明 Circle；<br>"
                "  - DragOver 时更新 ghost_transform.translation 跟随指针；<br>"
                "  - DragDrop/DragLeave 时 despawn 幽灵实体；<br>"
                "  - 🧩 Pickable::IGNORE 确保幽灵不干扰拾取事件；<br>"
                "- **event.propagate(false)**：<br>"
                "  - 阻止事件传播到下层实体，防止父/子实体重复处理；<br>"
                "  - ⚠️ 每个 Observer 都需要调用，遗漏会导致事件重复触发；<br>"
                "- **DroppedElement 持久化**：<br>"
                "  - DragDrop 成功后生成 Solid Circle 作为放置结果；<br>"
                "- **UI + Mesh 混合拾取**：<br>"
                "  - DraggableButton 是 UI Node，DropArea 是 Mesh2d + ColorMaterial；<br>"
                "  - 🧩 UI 和 Mesh 拾取后端可同时工作，互不干扰；"
            ),
            "知：<br>- DragDrop 事件完整流程；<br>- event.propagate。",
            "例：<br>- picking/simple_picking.rs(226)；<br>- ui/button.rs(232)。",
        )
    )

    # 220 - Debug Picking
    new_lines.append(
        row(
            PICK_220,
            (
                "**意图**：DebugPickingPlugin + DebugPickingMode 三级调试（Disabled/Normal/Noisy）；"
                "F3 键循环切换日志级别。⭐⭐<br>"
                "- **DebugPickingPlugin**：<br>"
                "  - .add_plugins((MeshPickingPlugin, DebugPickingPlugin)) 启用拾取调试；<br>"
                '  - LogPlugin { filter: "bevy_dev_tools=trace" } 显示 trace 级别拾取日志；<br>'
                "  - ⚠️ filter 必须设为 trace 否则 Noisy 模式日志不可见；<br>"
                "- **DebugPickingMode 三级**：<br>"
                "  - Disabled：禁用所有拾取日志；<br>"
                "  - Normal：仅关键事件日志（Click/Over/Out）；<br>"
                "  - Noisy：所有拾取事件详细日志（每帧 Over/Move 等）；<br>"
                "  - 🔧 insert_resource(DebugPickingMode::Normal) 初始级别；<br>"
                "- **distributive_run_if**：<br>"
                "  - .distributive_run_if(input_just_pressed(F3)) 条件运行系统；<br>"
                "  - ⚠️ distributive_run_if 与 run_if 的区别：前者每个系统独立判断条件；<br>"
                "- **闭包系统**：<br>"
                "  - |mut mode: ResMut| { ... } 匿名闭包作为系统；<br>"
                "  - 🔧 简单逻辑可用闭包避免定义独立函数；"
            ),
            "知：<br>- DebugPickingPlugin；<br>- DebugPickingMode。",
            "例：<br>- picking/simple_picking.rs(226)。",
        )
    )

    new_content_lines = lines[:start_idx] + new_lines + lines[end_idx:]

    with open(CATALOG_PATH, "w", encoding="utf-8") as f:
        f.write("\n".join(new_content_lines))

    print(f"Successfully updated entries 196-220 ({len(new_lines)} entries)")


if __name__ == "__main__":
    main()
