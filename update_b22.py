#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Update Bevy examples catalog entries 108-140."""

import re

CATALOG = "/Users/lf380/Code/Bevy/a1/bevy_examples_catalog.md"

# fmt: off
DATA = [
    # === App — 应用与运行时 ===
    (108, "应用基础",
     "empty.rs",
     "**意图**：最小 Bevy 应用——仅创建 App 实例后立即退出，无任何插件或系统。⭐<br>"
     "- **App::new() 最小结构**:<br>"
     "  - App::new() 创建空应用实例，不添加任何插件；<br>"
     "  - .run() 启动应用主循环——无窗口无事件循环时立即退出；<br>"
     "  - ⚠️ 没有 DefaultPlugins 意味着无渲染/无输入/无音频/无窗口；<br>"
     "- **调试用途**:<br>"
     "  - 🔧 cargo check --example empty 快速验证 Bevy 依赖是否正确链接；<br>"
     "  - 🧩 可作为 Plugin 开发的最小测试容器；<br>"
     "- **与 hello_world(1) 的区别**:<br>"
     "  - hello_world 添加了 Update 系统，本例连系统都没有；<br>"
     "  - 🔧 空应用编译通过即证明环境配置正确；",
     "知：<br>- App::new() 与 .run() 基本结构。<br>例：<br>- hello_world(1)。"),

    (109, "应用基础",
     "empty_defaults.rs",
     "**意图**：带 DefaultPlugins 的空应用——创建窗口但不添加任何自定义系统。⭐<br>"
     "- **DefaultPlugins 包含**:<br>"
     "  - WindowPlugin: 创建并管理窗口；<br>"
     "  - RenderPlugin: 初始化 WGPU 渲染管线；<br>"
     "  - InputPlugin: 键鼠/触摸输入处理；<br>"
     "  - AssetPlugin: 资产加载与管理；<br>"
     "  - ⚠️ DefaultPlugins 是大多数应用的起点，提供完整运行时；<br>"
     "- **MinimalPlugins 对比**:<br>"
     "  - MinimalPlugins = TaskPoolPlugin + TypeRegistrationPlugin + FrameCountPlugin + TimePlugin；<br>"
     "  - 🔧 MinimalPlugins 适合无窗口的逻辑测试（headless 模式）；<br>"
     "- **与 empty.rs 的区别**:<br>"
     "  - empty.rs 无插件→立即退出，本例有窗口→等待关闭；<br>"
     "  - 🔧 DefaultPlugins 可单独禁用某个插件：.disable::\<SomePlugin\>()；",
     "知：<br>- DefaultPlugins 插件组提供完整运行时。<br>例：<br>- app/empty.rs(108)。"),

    (110, "应用基础",
     "plugin.rs",
     "**意图**：演示自定义 Plugin 的创建和注册——周期性打印消息的 PrintMessagePlugin。⭐⭐<br>"
     "- **Plugin trait 实现**:<br>"
     "  - impl Plugin for PrintMessagePlugin { fn build(&self, app: &mut App) } 是插件入口；<br>"
     "  - build() 中调用 app.insert_resource() 和 app.add_systems() 注册资源与系统；<br>"
     "  - ⚠️ Plugin 是 Bevy 的核心扩展机制，所有功能模块都通过 Plugin 注册；<br>"
     "- **插件配置模式**:<br>"
     "  - PrintMessagePlugin { wait_duration, message } 通过结构体字段传递配置；<br>"
     "  - 🔧 配置字段在 build() 中读取，转换为 Resource 存储——数据与逻辑分离；<br>"
     "- **Resource + Timer 模式**:<br>"
     "  - PrintMessageState(Resource) 存储 timer 和 message，系统通过 ResMut 访问；<br>"
     "  - Timer::new(duration, TimerMode::Repeating) 创建循环计时器；<br>"
     "  - state.timer.tick(time.delta()).is_finished() 检查计时器是否完成一次周期；<br>"
     "- **插件注册**:<br>"
     "  - .add_plugins((DefaultPlugins, PrintMessagePlugin { ... })) 元组批量注册；<br>"
     "  - 🧩 插件可嵌套：插件的 build() 中可调用 app.add_plugins() 添加子插件；",
     "知：<br>- Plugin trait 与 build() 方法；<br>- Resource 存储全局状态；<br>- Timer 循环计时。<br>例：<br>- app/empty_defaults.rs(109)。"),

    (111, "应用基础",
     "plugin_group.rs",
     "**意图**：演示 PluginGroup 的创建和注册——将多个 Plugin 打包为一组统一管理。⭐⭐<br>"
     "- **PluginGroup trait**:<br>"
     "  - impl PluginGroup for HelloWorldPlugins { fn build(self) -> PluginGroupBuilder }；<br>"
     "  - PluginGroupBuilder::start::\<Self\>().add(PrintHelloPlugin).add(PrintWorldPlugin) 链式构建；<br>"
     "  - ⚠️ PluginGroup 适合将相关插件打包（如 DefaultPlugins 包含 20+ 插件）；<br>"
     "- **插件组操作**:<br>"
     "  - .disable::\<PrintWorldPlugin\>() 禁用组内某个插件；<br>"
     "  - .add_before::\<PrintHelloPlugin\>(other) 在指定插件前插入新插件；<br>"
     "  - 🔧 运行时可动态调整插件组内容，适合按需加载功能模块；<br>"
     "- **与 Plugin 的区别**:<br>"
     "  - Plugin 通过 build(&self, app) 注册，PluginGroup 通过 build(self) 返回 PluginGroupBuilder；<br>"
     "  - 🧩 DefaultPlugins 和 MinimalPlugins 都是 PluginGroup 的实现；",
     "知：<br>- PluginGroup trait 与 PluginGroupBuilder；<br>- 插件组的 disable/add_before 操作。<br>例：<br>- app/plugin.rs(110)。"),

    (112, "应用基础",
     "startup_system.rs",
     "**意图**：演示 Startup System——应用启动时运行一次的系统，用于初始化场景状态。⭐<br>"
     "- **Startup vs Update**:<br>"
     "  - Startup 系统在应用启动时执行一次，适合创建初始实体/资源；<br>"
     "  - Update 系统每帧执行，适合处理实时逻辑；<br>"
     "  - ⚠️ Startup 系统在所有 Update 系统之前执行；<br>"
     "- **注册方式**:<br>"
     "  - app.add_systems(Startup, startup_system) 注册启动系统；<br>"
     "  - 🔧 多个 Startup 系统默认并行执行，用 .chain() 可强制顺序；<br>"
     "- **典型用途**:<br>"
     "  - 创建相机、灯光、地面等场景基础实体；<br>"
     "  - 插入初始 Resource（如 GameRules、GameState）；<br>"
     "  - 🧩 Startup 系统是游戏初始化的标准入口点；",
     "知：<br>- Startup Schedule 与初始化系统。<br>例：<br>- hello_world(1)。"),

    (113, "应用基础",
     "logs.rs",
     "**意图**：演示 Bevy 日志系统——五级日志输出 + once! 宏防止重复打印。⭐⭐<br>"
     "- **五级日志**:<br>"
     "  - trace! / debug! / info! / warn! / error! 从低到高五个级别；<br>"
     "  - 默认 trace 和 debug 不显示，通过 LogPlugin 或 RUST_LOG 环境变量控制；<br>"
     "  - ⚠️ RUST_LOG=trace 显示所有级别，RUST_LOG=info,bevy_ecs=warn 按模块过滤；<br>"
     "- **once! 宏**:<br>"
     "  - trace_once! / debug_once! / info_once! / warn_once! / error_once! 每个调用点只打印一次；<br>"
     "  - 🔧 防止每帧重复打印的常用手段，适合初始化提示/一次性警告；<br>"
     "  - once!({}) 闭包版——整个代码块只执行一次；<br>"
     "- **LogPlugin 配置**:<br>"
     "  - bevy::log::LogPlugin { level, filter, ..default() } 控制日志级别和过滤；<br>"
     "  - 🔧 filter 字段支持 env_filter 语法，灵活控制模块级日志；<br>"
     "- **panic 处理**:<br>"
     "  - panic!() 会中断应用，本例用 P 键触发以演示 panic 行为；",
     "知：<br>- LogPlugin 日志插件；<br>- tracing 日志级别。<br>例：<br>- app/empty_defaults.rs(109)。"),

    (114, "应用基础",
     "log_layers.rs",
     "**意图**：添加自定义 tracing Layer 到日志系统——拦截日志事件并自定义输出格式。⭐⭐⭐<br>"
     "- **tracing Layer 机制**:<br>"
     "  - impl\<S: Subscriber\> Layer\<S\> for CustomLayer 实现自定义日志层；<br>"
     "  - on_event() 回调拦截每个日志事件，可读取 level/target/name；<br>"
     "  - 🔧 Layer 是 tracing 的核心扩展点，可组合多个层实现复杂日志管线；<br>"
     "- **custom_layer 函数**:<br>"
     "  - fn custom_layer(app: &mut App) -> Option\<BoxedLayer\> 注册额外层；<br>"
     "  - 多层可组合为 Vec\<Layer\>，每个层独立处理日志事件；<br>"
     "  - ⚠️ custom_layer 添加额外层，不会替换默认 fmt 层；<br>"
     "- **fmt_layer 替换**:<br>"
     "  - fn fmt_layer(app: &mut App) -> Option\<BoxedFmtLayer\> 替换默认格式化层；<br>"
     "  - .without_time() 禁用时间戳，.map_fmt_fields(debug_alt) 切换调试格式；<br>"
     "  - 🔧 fmt_layer 可完全自定义日志输出格式（JSON/自定义格式等）；<br>"
     "- **LogPlugin 注册**:<br>"
     "  - LogPlugin { custom_layer, fmt_layer, ..default() } 同时配置两层；",
     "知：<br>- tracing Layer 扩展机制；<br>- custom_layer 与 fmt_layer 区别。<br>例：<br>- app/logs.rs(113)。"),

    (115, "应用基础",
     "log_layers_ecs.rs",
     "**意图**：通过 mpsc 通道将 tracing 日志事件桥接到 Bevy ECS——在 UI 中显示实时日志。⭐⭐⭐<br>"
     "- **mpsc 桥接架构**:<br>"
     "  - CaptureLayer 拦截日志→mpsc::Sender 发送→mpsc::Receiver 接收→ECS Message 系统消费；<br>"
     "  - 🔧 这是 Layer（非 ECS）与 ECS 之间的标准通信模式；<br>"
     "- **NonSend Resource**:<br>"
     "  - CapturedLogMessages(mpsc::Receiver) 必须是非 Send 的——Receiver 实现了 !Sync；<br>"
     "  - ⚠️ NonSend 资源不能跨线程访问，只能在主线程系统中使用；<br>"
     "- **Message 系统**:<br>"
     "  - LogMessage(Message) 派生消息类型，通过 MessageWriter 发送；<br>"
     "  - MessageReader\<LogMessage\> 消费日志消息，逐条插入 UI Text；<br>"
     "  - 🔧 Message 是 Bevy 新一代事件系统，替代旧的 EventWriter/EventReader；<br>"
     "- **LogViewer UI**:<br>"
     "  - LogViewerRoot(Node) 作为日志容器，子实体动态生成 Text + TextSpan；<br>"
     "  - TextSpan 按级别着色（ERROR=红/WARN=橙/INFO=绿/DEBUG=蓝/TRACE=紫）；<br>"
     "  - 🧩 实时日志 UI 是调试工具的常见组件；",
     "知：<br>- mpsc 通道桥接 Layer 与 ECS；<br>- NonSend Resource；<br>- Message 消息系统。<br>例：<br>- app/log_layers.rs(114)。"),

    (116, "应用基础",
     "custom_loop.rs",
     "**意图**：自定义 Runner 手动控制 App 更新循环——从 stdin 读取输入驱动 ECS 更新。⭐⭐⭐<br>"
     "- **set_runner 替换默认循环**:<br>"
     "  - app.set_runner(my_runner) 用自定义函数替换默认事件循环；<br>"
     "  - fn my_runner(mut app: App) -> AppExit 是 Runner 签名；<br>"
     "  - ⚠️ 自定义 Runner 必须调用 app.finish() 和 app.cleanup() 完成插件构建；<br>"
     "- **手动更新模式**:<br>"
     "  - app.world_mut().resource_mut::\<Input\>() 直接访问 World 修改资源；<br>"
     "  - app.update() 手动触发一次 ECS 更新——每读一行 stdin 就更新一次；<br>"
     "  - 🔧 这种模式适合非实时应用（如 CLI 工具/批处理/测试框架）；<br>"
     "- **AppExit 控制**:<br>"
     "  - app.should_exit() 检查是否应该退出；<br>"
     "  - MessageWriter\<AppExit\> 在系统中发送退出事件；<br>"
     "  - 🔧 input.0 == \"exit\" 时发送 AppExit::Success 退出应用；<br>"
     "- **Local 状态**:<br>"
     "  - Local\<CounterState\> 系统私有状态，跨调用保持值；",
     "知：<br>- set_runner 自定义运行循环；<br>- app.update() 手动触发；<br>- AppExit 退出控制。<br>例：<br>- app/empty_defaults.rs(109)。"),

    (117, "应用基础",
     "return_after_run.rs",
     "**意图**：演示 App::run() 返回后继续执行——窗口关闭后回到 main 函数继续运行。⭐<br>"
     "- **run() 阻塞行为**:<br>"
     "  - App::run() 在桌面平台会阻塞直到窗口关闭；<br>"
     "  - ⚠️ 在 iOS 和 Web 上 run() 永不返回——不要在之后写代码；<br>"
     "  - ⚠️ 事件循环终止后无法重建窗口——这是平台限制；<br>"
     "- **窗口关闭恢复**:<br>"
     "  - println!(\"Bevy App has exited\") 在 run() 返回后执行；<br>"
     "  - 🔧 适合需要在游戏退出后执行清理/日志/返回上层调用的场景；<br>"
     "- **WindowPlugin 配置**:<br>"
     "  - primary_window: Some(Window { title: ... }) 设置窗口标题；<br>"
     "  - 🔧 关闭窗口即触发 run() 返回，无需额外代码；",
     "知：<br>- App::run() 阻塞与返回行为。<br>例：<br>- app/empty_defaults.rs(109)。"),

    (118, "应用基础",
     "headless.rs",
     "**意图**：无窗口 Headless 模式——使用 ScheduleRunnerPlugin 在无窗口环境下运行 ECS 逻辑。⭐⭐<br>"
     "- **ScheduleRunnerPlugin**:<br>"
     "  - ScheduleRunnerPlugin::run_once() 应用运行一次后退出；<br>"
     "  - ScheduleRunnerPlugin::run_loop(Duration) 按固定间隔循环运行；<br>"
     "  - ⚠️ 需要 --no-default-features 禁用 bevy_window 才能真正无窗口；<br>"
     "- **两次 App 运行**:<br>"
     "  - 第一次：run_once() 打印 hello world 后退出；<br>"
     "  - 第二次：run_loop(1/60s) 以 60fps 循环运行计数器；<br>"
     "  - 🔧 第二次运行禁用 LogPlugin 避免重复注册（全局唯一）；<br>"
     "- **Local 状态**:<br>"
     "  - Local\<CounterState\> 每帧递增计数器，每 60 帧打印一次；<br>"
     "  - 🔧 Local 是系统私有状态，适合帧计数/累加器等场景；<br>"
     "- **应用场景**:<br>"
     "  - 🧩 纯逻辑服务器/AI 训练/自动化测试/CI 环境；<br>"
     "  - 🔧 cargo run --example headless --no-default-features --features bevy_log；",
     "知：<br>- ScheduleRunnerPlugin 无窗口运行；<br>- run_once vs run_loop。<br>例：<br>- app/empty_defaults.rs(109)。"),

    (119, "应用基础",
     "no_renderer.rs",
     "**意图**：显示空窗口但不启动渲染器——适合集成测试和 CI 环境。⭐<br>"
     "- **禁用渲染器**:<br>"
     "  - RenderPlugin { render_creation: WgpuSettings { backends: None, .. }.into() } 禁用 GPU 后端；<br>"
     "  - ⚠️ backends: None 意味着不创建渲染设备，窗口存在但无内容；<br>"
     "  - 🔧 比 headless.rs 多一个窗口（用于测试窗口事件），但无渲染开销；<br>"
     "- **与 headless.rs 的区别**:<br>"
     "  - headless.rs 完全无窗口（需禁用 bevy_window）；<br>"
     "  - no_renderer.rs 有窗口但无渲染器——可测试窗口事件/输入/文件拖放；<br>"
     "  - 🧩 适合测试不依赖渲染的系统（如 UI 布局/输入处理/状态管理）；<br>"
     "- **DefaultPlugins 保留**:<br>"
     "  - 保留 DefaultPlugins 但禁用渲染，其他插件正常工作；",
     "知：<br>- RenderPlugin backends 配置；<br>- 无渲染器测试模式。<br>例：<br>- app/empty_defaults.rs(109)。"),

    (120, "应用基础",
     "headless_renderer.rs",
     "**意图**：无窗口渲染到 GPU 图像并保存到文件——离屏渲染管线完整演示。⭐⭐⭐<br>"
     "- **五步离屏渲染管线**:<br>"
     "  - 1. Camera 渲染到 RenderTarget::Image GPU 纹理；<br>"
     "  - 2. ImageCopyDriver（RenderGraph 节点）从 GPU 纹理复制到 Buffer；<br>"
     "  - 3. receive_image_from_buffer 在 Render 阶段后从 Buffer 读取到 mpsc 通道；<br>"
     "  - 4. scene::update 在 PostUpdate 阶段从通道接收数据保存为 PNG；<br>"
     "  - 5. single_image 模式下保存后自动退出；<br>"
     "- **RenderGraph 自定义节点**:<br>"
     "  - impl render_graph::Node for ImageCopyDriver 实现 GPU 数据复制；<br>"
     "  - graph.add_node(ImageCopy, ImageCopyDriver) 注册到渲染图；<br>"
     "  - 🔧 RenderGraph 是 Bevy 渲染管线的核心扩展点；<br>"
     "- **GPU→CPU 数据回读**:<br>"
     "  - buffer.slice(..).map_async(MapMode::Read) 将 GPU Buffer 映射到 CPU；<br>"
     "  - render_device.poll(PollType::wait_indefinitely()) 阻塞等待 GPU 完成；<br>"
     "  - ⚠️ WebGPU 安全模型要求 GPU/CPU 同时只能一方访问 Buffer；<br>"
     "- **ScheduleRunnerPlugin**:<br>"
     "  - ScheduleRunnerPlugin::run_loop(1/60s) 替代 WinitPlugin 驱动循环；<br>"
     "  - .disable::\<WinitPlugin\>() 禁用窗口事件循环；",
     "知：<br>- RenderGraph 自定义节点；<br>- GPU Buffer 映射与数据回读；<br>- 离屏渲染管线。<br>例：<br>- 3D/render_to_texture.rs(56)；<br>- app/headless.rs(118)。"),

    (121, "应用基础",
     "thread_pool_resources.rs",
     "**意图**：自定义 Bevy 内部线程池——限制并行执行使用的线程数量。⭐<br>"
     "- **TaskPoolPlugin 配置**:<br>"
     "  - TaskPoolPlugin { task_pool_options: TaskPoolOptions::with_num_threads(4) }；<br>"
     "  - 🔧 默认线程数 = CPU 核心数，手动限制可避免与其他进程竞争；<br>"
     "- **应用场景**:<br>"
     "  - CI/Docker 环境限制资源使用；<br>"
     "  - 🧩 与 headless 模式配合用于服务器部署；<br>"
     "  - 🔧 也可通过 init_resource::\<TaskPoolOptions\> 运行时修改；<br>"
     "- **注意事项**:<br>"
     "  - ⚠️ 线程数过少会降低 ECS 并行度，影响性能；<br>"
     "  - 🔬 线程池影响 System 并行调度和 Asset 加载线程；",
     "知：<br>- TaskPoolPlugin 线程池配置。<br>例：<br>- app/empty_defaults.rs(109)。"),

    (122, "应用基础",
     "without_winit.rs",
     "**意图**：无 Winit 事件循环的应用——禁用 WinitPlugin 后单次运行即退出。⭐<br>"
     "- **禁用 WinitPlugin**:<br>"
     "  - DefaultPlugins.build().disable::\<WinitPlugin\>() 移除窗口事件循环；<br>"
     "  - ⚠️ 没有 Winit 意味着没有窗口创建/事件处理/帧率控制；<br>"
     "- **与 headless.rs 的区别**:<br>"
     "  - headless.rs 使用 ScheduleRunnerPlugin 驱动循环；<br>"
     "  - without_winit.rs 无替代循环——App::run() 执行一次 Update 后退出；<br>"
     "  - 🔧 适合单次初始化 + 快照/截图场景；<br>"
     "- **Camera3d 仍可创建**:<br>"
     "  - 即使无 Winit，DefaultPlugins 中的 RenderPlugin 仍可初始化 GPU；<br>"
     "  - 🧩 可用于离屏渲染测试（无窗口但有 GPU 上下文）；",
     "知：<br>- WinitPlugin 窗口事件循环；<br>- disable 禁用插件。<br>例：<br>- app/empty_defaults.rs(109)。"),

    (123, "应用基础",
     "drag_and_drop.rs",
     "**意图**：处理文件拖放事件——监听 FileDragAndDrop 消息获取拖入文件路径。⭐<br>"
     "- **FileDragAndDrop 消息**:<br>"
     "  - MessageReader\<FileDragAndDrop\> 读取拖放事件；<br>"
     "  - FileDragAndDrop::DroppedFile { window, path } 包含文件路径；<br>"
     "  - ⚠️ 仅桌面平台支持文件拖放，Web/WASM 需要额外处理；<br>"
     "- **事件处理模式**:<br>"
     "  - for drag_and_drop in reader.read() 遍历所有待处理事件；<br>"
     "  - 🔧 info!(\"{:?}\", drag_and_drop) 打印事件详情用于调试；<br>"
     "- **应用场景**:<br>"
     "  - 🧩 拖入图像/模型/配置文件实现资产导入；<br>"
     "  - 🔧 配合 AssetServer::load(path) 加载拖入的文件；",
     "知：<br>- FileDragAndDrop 消息类型；<br>- MessageReader 消费事件。<br>例：<br>- app/empty_defaults.rs(109)。"),

    # === ECS — 实体组件系统 ===
    (124, "ECS入门",
     "ecs_guide.rs",
     "**意图**：Bevy ECS 全面入门——Component/System/Resource/Query/State/系统排序完整演示。⭐⭐⭐<br>"
     "- **Component 组件**:<br>"
     "  - #[derive(Component)] 派生组件，普通 Rust 结构体/枚举即可；<br>"
     "  - Player { name } / Score { value } / PlayerStreak 枚举——组件携带数据；<br>"
     "  - ⚠️ 组件是纯数据，不包含逻辑——行为由 System 驱动；<br>"
     "- **Resource 资源**:<br>"
     "  - #[derive(Resource)] 派生资源，全局共享数据（非实体级别）；<br>"
     "  - GameState / GameRules——游戏状态和规则配置；<br>"
     "  - 🔧 Res\<T\> 只读访问，ResMut\<T\> 可写访问；<br>"
     "- **System 系统**:<br>"
     "  - 普通函数即系统，参数由 Bevy 自动注入；<br>"
     "  - Query\<(&Player, &mut Score)\> 查询拥有指定组件的实体；<br>"
     "  - Commands 延迟命令缓冲——spawn/despawn/insert/remove；<br>"
     "- **系统排序**:<br>"
     "  - .chain() 顺序执行，.before()/.after() 声明依赖；<br>"
     "  - SystemSet 自定义系统集合，.configure_sets() 配置顺序；<br>"
     "  - 🔧 默认并行执行，仅在数据冲突时自动序列化；<br>"
     "- **Exclusive System**:<br>"
     "  - fn exclusive_player_system(world: &mut World) 直接访问 World；<br>"
     "  - ⚠️ 会阻塞并行执行，应尽量避免——仅在必须直接操作 World 时使用；<br>"
     "- **Startup vs Update**:<br>"
     "  - Startup 系统运行一次（初始化），Update 系统每帧运行；<br>"
     "  - 🔧 Startup 适合创建实体/资源，Update 适合处理输入/动画/游戏逻辑；",
     "知：<br>- Component/Resource/System 三大概念；<br>- Query 查询与 Commands 命令；<br>- 系统排序与并行。<br>例：<br>- hello_world(1)。"),

    (125, "ECS入门",
     "startup_system.rs",
     "**意图**：Startup System 基础——区分启动系统（运行一次）与普通系统（每帧运行）。⭐<br>"
     "- **Startup 系统注册**:<br>"
     "  - app.add_systems(Startup, startup_system) 注册为启动系统；<br>"
     "  - app.add_systems(Update, normal_system) 注册为每帧更新系统；<br>"
     "  - ⚠️ Startup 在所有 Update 之前执行一次；<br>"
     "- **执行顺序**:<br>"
     "  - startup_system 先执行（打印 \"ran first\"）；<br>"
     "  - normal_system 后执行（打印 \"ran second\"）；<br>"
     "  - 🔧 Startup 系统保证在第一个 Update 帧之前完成；<br>"
     "- **典型用途**:<br>"
     "  - 创建 Camera/灯光/地面等场景基础；<br>"
     "  - 插入初始 Resource 和配置数据；<br>"
     "  - 🧩 是所有 Bevy 应用的标准初始化入口；",
     "知：<br>- Startup Schedule 与初始化系统。<br>例：<br>- ecs/ecs_guide.rs(124)。"),

    (126, "ECS查询",
     "change_detection.rs",
     "**意图**：组件和资源变化检测——Changed/Added 过滤器 + Ref 变化追踪 + set_if_neq 优化。⭐⭐⭐<br>"
     "- **Changed\<T\> 过滤器**:<br>"
     "  - Query\<Ref\<MyComponent\>, Changed\<MyComponent\>\> 只返回本帧变化的实体；<br>"
     "  - ⚠️ Changed 检测基于 Mut 可变引用——仅当系统获取 &mut T 时触发；<br>"
     "- **Added\<T\> 过滤器**:<br>"
     "  - component.is_added() 判断组件是否在本帧首次添加；<br>"
     "  - 🔧 适合初始化逻辑：实体创建后的首次设置；<br>"
     "- **Ref 变化追踪**:<br>"
     "  - Ref\<T\> 提供 is_changed() / is_added() / changed_by() 方法；<br>"
     "  - changed_by() 需要 track_location feature，返回修改位置（文件+行号）；<br>"
     "  - 🔧 多系统修改同一组件时，changed_by() 可定位修改来源；<br>"
     "- **set_if_neq 优化**:<br>"
     "  - component.set_if_neq(new_value) 仅在值不同时触发变化检测；<br>"
     "  - ⚠️ 直接赋值即使值相同也会触发 Changed——用 set_if_neq 避免无效检测；<br>"
     "- **Resource 变化检测**:<br>"
     "  - Res\<T\> 的 is_changed() / is_added() 方法与组件相同；<br>"
     "  - 🧩 Resource 变化检测适合配置热重载/全局状态变更通知；",
     "知：<br>- Changed/Added Query 过滤器；<br>- Ref 变化追踪；<br>- set_if_neq 优化。<br>例：<br>- ecs/ecs_guide.rs(124)。"),

    (127, "ECS观察者",
     "removal_detection.rs",
     "**意图**：组件移除检测——使用 Observer + On\<Remove, T\> 监听组件被移除的时刻。⭐⭐<br>"
     "- **Observer 移除监听**:<br>"
     "  - app.add_observer(react_on_removal) 注册移除监听器；<br>"
     "  - fn react_on_removal(remove: On\<Remove, MyComponent\>, ...) 接收移除事件；<br>"
     "  - 🔧 Observer 在组件移除后立即执行，比轮询检测更高效；<br>"
     "- **On\<Remove, T\> 事件**:<br>"
     "  - remove.entity 获取被移除组件的实体；<br>"
     "  - ⚠️ 移除后不能再访问该组件数据——需在移除前保存必要信息；<br>"
     "- **与 lifecycle hook 的区别**:<br>"
     "  - Observer 是系统级别的事件处理，hook 是组件级别的底层回调；<br>"
     "  - 🔧 高频移除场景用 hook 更高效，低频场景用 Observer 更灵活；<br>"
     "- **应用场景**:<br>"
     "  - 🧩 Buff 过期时的清理逻辑、实体死亡时的特效触发；<br>"
     "  - 🔧 维护索引/缓存的同步更新；",
     "知：<br>- Observer 观察者模式；<br>- On\<Remove, T\> 移除事件。<br>例：<br>- ecs/observers.rs(132)。"),

    (128, "ECS层级",
     "hierarchy.rs",
     "**意图**：父子层级关系——with_children 宏创建子实体，Transform 自动传播。⭐⭐<br>"
     "- **with_children 宏**:<br>"
     "  - .with_children(|parent| { parent.spawn(...) }) 在 spawn 时直接创建子实体；<br>"
     "  - parent 是 ChildSpawnerCommands，API 与 Commands 相同；<br>"
     "  - 🔧 子实体的 Transform 相对于父实体（局部坐标）；<br>"
     "- **add_child 追加子实体**:<br>"
     "  - commands.entity(parent).add_child(child) 将已有实体添加为子实体；<br>"
     "  - 🔧 适合动态构建层级（如运行时生成的装备/技能树）；<br>"
     "- **Transform 传播**:<br>"
     "  - 父 Transform 变化→GlobalTransform 自动更新→子 GlobalTransform 重新计算；<br>"
     "  - ⚠️ 子实体旋转在父旋转基础上叠加（先父后子）；<br>"
     "- **Children 遍历**:<br>"
     "  - Query\<&Children\> 获取子实体列表，for child in children 遍历；<br>"
     "  - commands.entity(child).despawn() 移除子实体（递归销毁所有后代）；<br>"
     "  - 🧩 父子层级是 UI 布景/骨骼动画/场景图的核心机制；",
     "知：<br>- with_children / add_child 层级构建；<br>- Transform 父子传播；<br>- Children 遍历与销毁。<br>例：<br>- ecs/ecs_guide.rs(124)。"),

    (129, "ECS泛型",
     "generic_system.rs",
     "**意图**：泛型 System 复用——cleanup_system::\<T\>() 用 turbofish 语法按标记组件清理实体。⭐⭐<br>"
     "- **泛型系统模式**:<br>"
     "  - fn cleanup_system\<T: Component\>(commands, query\<Entity, With\<T\>\>) 泛型清理系统；<br>"
     "  - cleanup_system::\<MenuClose\>() / cleanup_system::\<LevelUnload\>() 不同标记不同清理；<br>"
     "  - 🔧 泛型系统是减少重复代码的标准手法；<br>"
     "- **标记组件**:<br>"
     "  - MenuClose / LevelUnload 零大小标记组件——仅用于区分实体类别；<br>"
     "  - ⚠️ 标记组件不携带数据，纯粹作为类型级别的分类标签；<br>"
     "- **OnExit 清理**:<br>"
     "  - .add_systems(OnExit(AppState::MainMenu), cleanup_system::\<MenuClose\>)；<br>"
     "  - 🔧 状态退出时自动清理对应标记的实体，避免内存泄漏；<br>"
     "- **State 配合**:<br>"
     "  - AppState 枚举 + in_state() 条件运行——不同状态激活不同系统；<br>"
     "  - 🧩 泛型 + State + 标记组件 = 可组合的状态清理模式；",
     "知：<br>- 泛型 System 与 turbofish 语法；<br>- 标记组件模式；<br>- OnExit 状态清理。<br>例：<br>- ecs/ecs_guide.rs(124)。"),

    (130, "ECS系统",
     "system_closure.rs",
     "**意图**：闭包作为 System——匿名函数/闭包/Captured 变量均可作为系统运行。⭐⭐<br>"
     "- **三种闭包形式**:<br>"
     "  - 简单闭包：|| { info!(\"...\") } 无状态无参数；<br>"
     "  - 初始化闭包：|value: String| { move || { ... } } 返回闭包作为系统；<br>"
     "  - 内联闭包：.add_systems(Update, || { info!(\"...\") }) 直接写在注册处；<br>"
     "  - 🔧 闭包适合简单的一次性逻辑，复杂逻辑仍用命名函数；<br>"
     "- **Local 状态保存**:<br>"
     "  - complex_closure(\"foo\".into()) 传入初始值，闭包内 value 在调用间保持；<br>"
     "  - ⚠️ 闭包捕获的外部变量状态也跨调用保持；<br>"
     "- **move 闭包**:<br>"
     "  - move || { info!(outside_variable) } 捕获外部变量的所有权；<br>"
     "  - 🔧 move 闭包适合需要持有外部数据的系统；<br>"
     "- **注意事项**:<br>"
     "  - ⚠️ 闭包系统不能有可变状态（除 Local），否则每次调用都会重新创建；<br>"
     "  - 🧩 闭包系统适合原型开发/测试/简单回调；",
     "知：<br>- 闭包作为 System；<br>- move 闭包与变量捕获。<br>例：<br>- ecs/ecs_guide.rs(124)。"),

    (131, "ECS自定义",
     "system_param.rs",
     "**意图**：自定义 SystemParam——封装 Query + Resource 为可复用的高级参数类型。⭐⭐⭐<br>"
     "- **SystemParam 派生**:<br>"
     "  - #[derive(SystemParam)] 派生宏创建自定义参数；<br>"
     "  - PlayerCounter { players: Query, count: ResMut } 组合查询和资源；<br>"
     "  - 🔧 SystemParam 是构建复杂系统参数的标准方式；<br>"
     "- **生命周期标注**:<br>"
     "  - PlayerCounter\<'w, 's\> 必须带生命周期——与 World('w) 和系统状态('s) 关联；<br>"
     "  - ⚠️ 忘记生命周期标注是最常见的编译错误；<br>"
     "- **impl 方法**:<br>"
     "  - PlayerCounter 可添加方法（如 count()），封装复杂查询逻辑；<br>"
     "  - 🔧 方法可组合多次查询/修改，对外暴露简洁 API；<br>"
     "- **使用方式**:<br>"
     "  - fn count_players(mut counter: PlayerCounter) 直接作为系统参数；<br>"
     "  - counter.count() 调用封装的方法；<br>"
     "  - 🧩 SystemParam 是 Bevy 框架扩展的核心机制（如 QueryDsl、Populated 等内置参数）；",
     "知：<br>- #[derive(SystemParam)] 派生宏；<br>- 生命周期标注；<br>- 封装查询逻辑。<br>例：<br>- ecs/ecs_guide.rs(124)。"),

    (132, "ECS观察者",
     "observers.rs",
     "**意图**：Observer 观察者模式——监听组件生命周期事件(Add/Remove)和自定义事件(EntityEvent)。⭐⭐⭐<br>"
     "- **组件生命周期 Observer**:<br>"
     "  - app.add_observer(on_add_mine) 监听 Mine 组件的 Add 事件；<br>"
     "  - On\<Add, Mine\> / On\<Remove, Mine\> 分别在组件添加/移除时触发；<br>"
     "  - 🔧 Observer 是 ECS 中的响应式编程模式，替代轮询检测；<br>"
     "- **自定义 Event**:<br>"
     "  - #[derive(Event)] struct ExplodeMines 普通事件——触发顶层 Observer；<br>"
     "  - #[derive(EntityEvent)] struct Explode 实体事件——可定向到特定实体；<br>"
     "  - ⚠️ EntityEvent 既触发顶层 Observer，也触发目标实体的 Observer；<br>"
     "- **Observer 复用**:<br>"
     "  - Observer::new(explode_mine) 创建观察者实例；<br>"
     "  - observer.watch_entity(entity) 让同一观察者监听多个实体；<br>"
     "  - 🔧 比每实体创建 Observer 更高效——减少实体数量；<br>"
     "- **SpatialIndex 资源**:<br>"
     "  - HashMap 维护网格空间索引，Observer 自动同步增删；<br>"
     "  - 🧩 Observer + 索引维护是游戏引擎中常见的 ECS 模式；",
     "知：<br>- Observer 观察者模式；<br>- Event vs EntityEvent；<br>- 组件生命周期事件。<br>例：<br>- ecs/ecs_guide.rs(124)。"),

    (133, "ECS事件",
     "observer_propagation.rs",
     "**意图**：事件通过实体层级冒泡传播——EntityEvent 的 propagate 机制实现伤害穿透装甲。⭐⭐⭐<br>"
     "- **EntityEvent 传播**:<br>"
     "  - #[entity_event(propagate, auto_propagate)] 启用事件冒泡；<br>"
     "  - 攻击命中盔甲子实体→伤害未被完全阻挡→事件冒泡到父实体（角色）；<br>"
     "  - 🔧 propagate 机制类似 DOM 事件冒泡——子→父层级传递；<br>"
     "- **on_propagate 控制**:<br>"
     "  - attack.propagate(false) 阻止事件继续冒泡；<br>"
     "  - attack.propagate(true) 允许事件继续冒泡；<br>"
     "  - ⚠️ auto_propagate=true 时默认传播，需手动阻止；<br>"
     "- **攻击-装甲模型**:<br>"
     "  - Armor(u16) 组件存储护甲值，damage.saturating_sub(armor) 计算穿透伤害；<br>"
     "  - 穿透伤害 > 0 时事件继续冒泡到父实体；<br>"
     "  - 穿透伤害 = 0 时事件被装甲完全阻挡，propagate(false)；<br>"
     "- **根实体处理**:<br>"
     "  - take_damage 在角色实体上监听——接收最终穿透伤害；<br>"
     "  - HP 降为 0 时 despawn 实体并退出应用；<br>"
     "  - 🧩 冒泡机制适合护盾/护甲/伤害分层等层级化伤害模型；",
     "知：<br>- EntityEvent propagate 冒泡；<br>- auto_propagate 自动传播；<br>- 层级化伤害模型。<br>例：<br>- ecs/hierarchy.rs(128)；<br>- ecs/observers.rs(132)。"),

    (134, "ECS组件",
     "component_hooks.rs",
     "**意图**：组件生命周期钩子——on_add/on_insert/on_replace/on_remove 四个阶段的底层回调。⭐⭐⭐<br>"
     "- **四个生命周期钩子**:<br>"
     "  - on_add: 组件首次添加到实体时触发（不含已存在的情况）；<br>"
     "  - on_insert: 组件插入时触发（含已存在的情况，on_add 之后）；<br>"
     "  - on_replace: 组件被替换前触发（可访问旧值）；<br>"
     "  - on_remove: 组件移除后触发（组件数据仍可访问）；<br>"
     "  - ⚠️ 每个组件每种钩子只能注册一个——多个钩子会覆盖；<br>"
     "- **register_component_hooks**:<br>"
     "  - world.register_component_hooks::\<MyComponent\>() 必须在无实体使用该组件时调用；<br>"
     "  - 🔧 通常在 World 初始化阶段（Startup）注册；<br>"
     "- **HookContext 参数**:<br>"
     "  - entity: 触发钩子的实体；<br>"
     "  - component_id: 组件 ID（动态组件场景有用）；<br>"
     "  - caller: 触发位置（需 track_location feature）；<br>"
     "- **MyComponentIndex 示例**:<br>"
     "  - on_add 时将组件加入 HashMap 索引；<br>"
     "  - on_replace 时从索引移除旧值；<br>"
     "  - on_remove 时从索引移除并 despawn 实体；<br>"
     "  - 🧩 组件钩子适合维护全局索引/缓存同步；",
     "知：<br>- on_add/on_insert/on_replace/on_remove 四个钩子；<br>- register_component_hooks 注册；<br>- HookContext 上下文。<br>例：<br>- ecs/ecs_guide.rs(124)。"),

    (135, "ECS组件",
     "immutable_components.rs",
     "**意图**：不可变组件——声明后只能读取/移除/替换，不能 get_mut。⭐⭐<br>"
     "- **不可变声明**:<br>"
     "  - #[component(immutable)] 标记组件为不可变；<br>"
     "  - ⚠️ 不可变组件不能 get_mut——只能 take + insert 替换；<br>"
     "  - 🔧 不可变组件确保所有修改都通过 hook 捕获——维护索引安全；<br>"
     "- **take + insert 模式**:<br>"
     "  - entity.take::\<T\>() 取出组件值（等同于移除）；<br>"
     "  - entity.insert(new_value) 重新插入——触发 on_replace + on_insert；<br>"
     "  - 🔧 这是不可变组件的唯一修改方式；<br>"
     "- **Name 组件示例**:<br>"
     "  - 不可变 Name + on_insert/on_replace hook = 完整的名称索引；<br>"
     "  - 所有修改都通过 hook 捕获，索引始终同步；<br>"
     "  - 🧩 Name 是 Bevy 内置不可变组件的标准范例；<br>"
     "- **动态不可变组件**:<br>"
     "  - ComponentDescriptor::new_with_layout 创建动态不可变组件；<br>"
     "  - insert_by_id / get_by_id / get_mut_by_id 按 ID 操作；<br>"
     "  - ⚠️ 不可变组件的 get_mut_by_id 返回 Err——只能读不能写；",
     "知：<br>- #[component(immutable)] 不可变声明；<br>- take + insert 替换模式；<br>- hook 维护索引。<br>例：<br>- ecs/ecs_guide.rs(124)。"),

    (136, "ECS消息",
     "message.rs",
     "**意图**：Message 消息系统——MessageWriter 发送 / MessageMutator 变更 / MessageReader 消费，链式系统确保顺序。⭐⭐⭐<br>"
     "- **Message 定义**:<br>"
     "  - #[derive(Message)] 派生消息类型；<br>"
     "  - app.add_message::\<DealDamage\>() 注册消息类型；<br>"
     "  - ⚠️ 必须先注册才能使用，否则编译错误；<br>"
     "- **发送消息**:<br>"
     "  - MessageWriter\<T\>::write(msg) 发送消息；<br>"
     "  - MessageWriter\<T\>::write_default() 发送默认值消息；<br>"
     "  - 🔧 Writer 在系统中使用，支持多条消息批量发送；<br>"
     "- **变更消息**:<br>"
     "  - MessageMutator\<T\>::read() 返回 &mut T 迭代器——可就地修改未读消息；<br>"
     "  - 🔧 Mutator 适合消息管线中的中间处理（如扣除护甲）；<br>"
     "- **消费消息**:<br>"
     "  - MessageReader\<T\>::read() 返回 &T 迭代器——只读消费；<br>"
     "  - 🔧 多个系统可消费同一消息——广播模式；<br>"
     "- **系统排序**:<br>"
     "  - .chain() 确保 Writer → Mutator → Reader 顺序执行；<br>"
     "  - ⚠️ 无 chain 时 Reader 可能在 Writer 之前运行，导致消息延迟一帧；<br>"
     "  - 🧩 消息管线模式：发送→修改→应用→通知，是游戏逻辑的标准数据流；",
     "知：<br>- Message 派生与注册；<br>- MessageWriter/MessageMutator/MessageReader；<br>- .chain() 确保顺序。<br>例：<br>- ecs/ecs_guide.rs(124)。"),

    (137, "ECS消息",
     "send_and_receive_messages.rs",
     "**意图**：同一系统中收发同类型 Message——ParamSet 和 Local\<MessageCursor\> 两种解决方案。⭐⭐⭐<br>"
     "- **借用冲突问题**:<br>"
     "  - MessageWriter + MessageReader 同类型会借用同一 Messages 资源——Rust 拒绝；<br>"
     "  - ⚠️ 这是 Rust 所有权规则的限制，不是 Bevy 的 bug；<br>"
     "- **方案一：ParamSet**:<br>"
     "  - ParamSet\<(MessageReader\<T\>, MessageWriter\<T\>)\> 分时借用；<br>"
     "  - param_set.p0().read() 先读，param_set.p1().write() 后写——不同时访问；<br>"
     "  - 🔧 简单直观，适合大多数场景；<br>"
     "- **方案二：Local\<MessageCursor\>**:<br>"
     "  - Local\<MessageCursor\<T\>\> 手动管理游标状态；<br>"
     "  - ResMut\<Messages\<T\>\> 直接访问消息资源——同时读写；<br>"
     "  - 🔧 更灵活但更复杂，适合需要精确控制消息处理的场景；<br>"
     "- **不同消息类型**:<br>"
     "  - 不同类型的消息可以同时读写——不受借用规则限制；<br>"
     "  - 🔧 read_and_write_different_message_types 演示无冲突的跨类型操作；",
     "知：<br>- MessageWriter + MessageReader 借用冲突；<br>- ParamSet 分时借用；<br>- Local\<MessageCursor\> 手动管理。<br>例：<br>- ecs/message.rs(136)。"),

    (138, "ECS错误",
     "error_handling.rs",
     "**意图**：系统和观察者的错误处理——Result 返回 + set_error_handler + 管道错误处理。⭐⭐⭐<br>"
     "- **系统返回 Result**:<br>"
     "  - fn setup(...) -> Result 使用 ? 运算符传播错误；<br>"
     "  - Sphere::new(1.0).mesh().ico(7)? 可能失败的操作用 ? 处理；<br>"
     "  - ⚠️ 默认错误处理器是 panic——需设置替代处理器；<br>"
     "- **全局错误处理器**:<br>"
     "  - app.set_error_handler(warn) 设置全局错误处理为警告日志；<br>"
     "  - 内置处理器：panic / error / warn / info / debug / trace / ignore；<br>"
     "  - 🔧 warn 是开发阶段推荐——既不崩溃又能看到错误；<br>"
     "- **管道错误处理**:<br>"
     "  - failing_system.pipe(|result: In\<Result\>| { result.0.inspect_err(...) })；<br>"
     "  - 🔧 管道可对特定系统做自定义错误处理（如重试/降级）；<br>"
     "- **命令错误处理**:<br>"
     "  - commands.queue_handled(closure, |error, context| { error!(...) }) 命令级错误处理；<br>"
     "  - ⚠️ 失败命令默认 panic，queue_handled 可自定义处理；<br>"
     "- **可失败观察者**:<br>"
     "  - fn fallible_observer(...) -> Result 观察者也可返回 Result；<br>"
     "  - 🧩 错误处理是生产级 Bevy 应用的必备基础设施；",
     "知：<br>- 系统返回 Result；<br>- set_error_handler 全局错误处理；<br>- pipe 管道错误处理。<br>例：<br>- ecs/observers.rs(132)。"),

    (139, "ECS参数",
     "fallible_params.rs",
     "**意图**：可失败系统参数——Single/Option\<Single\>/Populated 在条件不满足时静默跳过系统执行。⭐⭐⭐<br>"
     "- **Single 参数**:<br>"
     "  - Single\<(&mut Transform, &Player)\> 要求恰好一个匹配实体；<br>"
     "  - ⚠️ 匹配 0 个或多个时系统被静默跳过（不报错不 panic）；<br>"
     "  - 🔧 适合确保唯一性的场景（如唯一玩家/唯一相机）；<br>"
     "- **Option\<Single\> 参数**:<br>"
     "  - Option\<Single\<&Transform, With\<Enemy\>\>\> 可选的唯一匹配；<br>"
     "  - 匹配 0 个或 1 个时正常工作，多个时系统被跳过；<br>"
     "  - 🔧 适合「有则处理，无则跳过」的场景（如追踪目标）；<br>"
     "- **Populated 参数**:<br>"
     "  - Populated\<(&mut Transform, &mut Enemy)\> 要求至少一个匹配实体；<br>"
     "  - ⚠️ 无匹配实体时系统被跳过——避免空迭代的性能浪费；<br>"
     "  - 🔧 适合批量处理（如移动所有敌人）——无敌人时自动跳过；<br>"
     "- **Res/ResMut 失败**:<br>"
     "  - Res\<T\> 资源不存在时调用默认错误处理器（默认 panic）；<br>"
     "  - set_error_handler(warn) 可改为警告日志；<br>"
     "- **错误处理配合**:<br>"
     "  - set_error_handler(warn) 使参数验证失败只输出警告而非 panic；<br>"
     "  - 🧩 可失败参数 + 错误处理器 = 优雅的条件执行模式；",
     "知：<br>- Single 唯一匹配；<br>- Option\<Single\> 可选匹配；<br>- Populated 至少一个匹配。<br>例：<br>- ecs/ecs_guide.rs(124)。"),

    (140, "ECS时间",
     "fixed_timestep.rs",
     "**意图**：FixedUpdate 固定时间步长系统——按固定间隔运行物理/逻辑，独立于帧率。⭐⭐<br>"
     "- **FixedUpdate Schedule**:<br>"
     "  - .add_systems(FixedUpdate, fixed_update) 注册固定步长系统；<br>"
     "  - Time\<Fixed\>::from_seconds(0.5) 设置每 0.5 秒执行一次（2Hz）；<br>"
     "  - ⚠️ FixedUpdate 独立于 Update——帧率波动不影响执行频率；<br>"
     "- **Time\<Fixed\> 资源**:<br>"
     "  - fixed_time.overstep() 获取超出固定步长的累积时间；<br>"
     "  - 🔧 overstep 用于判断是否需要多次固定步长追赶；<br>"
     "- **与 Update 的对比**:<br>"
     "  - Update 每帧执行——间隔随帧率变化（60fps=16.6ms，30fps=33.3ms）；<br>"
     "  - FixedUpdate 每固定间隔执行——间隔恒定（如 0.5s）；<br>"
     "  - 🔧 物理模拟/网络同步/回合制逻辑应使用 FixedUpdate 保证确定性；<br>"
     "- **Local 状态**:<br>"
     "  - Local\<f32\> 记录上次执行时间，计算实际时间间隔；<br>"
     "  - 🔧 Local 适合性能监控/帧间隔统计；<br>"
     "- **应用场景**:<br>"
     "  - 🧩 物理步进、网络同步、确定性模拟、回合制游戏逻辑；",
     "知：<br>- FixedUpdate 固定步长调度；<br>- Time\<Fixed\> 与 overstep；<br>- 确定性时间步长。<br>例：<br>- ecs/ecs_guide.rs(124)。"),
]
# fmt: on


def build_row(seq, cat, filename, intent, prereq):
    """Build a markdown table row."""
    return f"| {seq} | {cat} | `{filename}` | {intent} | {prereq} |"


def main():
    with open(CATALOG, "r", encoding="utf-8") as f:
        lines = f.readlines()

    # Build lookup: seq -> new row
    new_rows = {}
    for seq, cat, fname, intent, prereq in DATA:
        new_rows[seq] = build_row(seq, cat, fname, intent, prereq)

    result = []
    i = 0
    replaced = set()
    while i < len(lines):
        line = lines[i]
        # Match table rows like "| 108 | ..."
        m = re.match(r"^\|\s*(\d+)\s*\|", line)
        if m:
            seq = int(m.group(1))
            if seq in new_rows:
                result.append(new_rows[seq] + "\n")
                replaced.add(seq)
                i += 1
                continue
        result.append(line)
        i += 1

    with open(CATALOG, "w", encoding="utf-8") as f:
        f.writelines(result)

    print(f"Replaced {len(replaced)} entries: {sorted(replaced)}")
    missing = set(new_rows.keys()) - replaced
    if missing:
        print(f"WARNING: Missing entries: {sorted(missing)}")


if __name__ == "__main__":
    main()
