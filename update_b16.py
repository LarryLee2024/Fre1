#!/usr/bin/env python3
"""Replace catalog entries by line range."""

CATALOG = "/Users/lf380/Code/Bevy/a1/bevy_examples_catalog.md"

# Read file
with open(CATALOG, "r", encoding="utf-8") as f:
    lines = f.readlines()

# Find and replace entries 159-176 by matching seq numbers
# We need to replace entries for these files:
# 159: asset_loading.rs
# 160: asset_settings.rs
# 161: hot_asset_reloading.rs
# 162: multi_asset_sync.rs
# 163: repeated_texture.rs
# 164: web_asset.rs
# 165: extra_source.rs
# 166: embedded_asset.rs
# 167: custom_asset.rs
# 168: custom_asset_reader.rs
# 169: asset_decompression.rs
# 170: processing/asset_processing.rs
# 171: alter_mesh.rs
# 172: alter_sprite.rs
# 173: scene.rs
# 174: time.rs
# 175: timers.rs
# 176: virtual_time.rs

# Build replacements dict: seq -> new line
REPLACEMENTS = {}


def add(seq, cat, fname, intent, know):
    """Add a replacement entry."""
    line = f"| {seq} | {cat} | `{fname}` | {intent} | {know} |"
    REPLACEMENTS[seq] = line


# Entry 159: asset_loading.rs
add(
    159,
    "资产管理",
    "asset_loading.rs",
    "**意图**：演示多种资产加载方式——AssetServer 异步加载 glTF 网格、load_folder 加载文件夹、materials.add 直接创建材质。⭐⭐<br>"
    "- **AssetServer::load 异步加载**:<br>"
    "  - asset_server.load(path) 返回 Handle&lt;T&gt;，资产在后台线程异步加载，不阻塞主线程；<br>"
    "  - GltfAssetLabel::Primitive { mesh: 0, primitive: 0 }.from_asset(path) 指定 glTF 中的具体网格 primitive；<br>"
    "  - ⚠️ 加载后立即调用 meshes.get(&amp;handle) 返回 None——资产尚未就绪，需等待 AssetEvent 或 run_if 条件；<br>"
    "- **load_folder 文件夹批量加载**:<br>"
    "  - asset_server.load_folder(path) 并行加载目录下所有资产，返回 Handle&lt;LoadedFolder&gt;；<br>"
    "  - LoadedFolder 依赖全部就绪后触发 AssetEvent::LoadedWithDependencies 事件；<br>"
    "  - ⚠️ 保持 Handle alive 否则资产会被卸载——通常用 Resource 持有；<br>"
    "- **materials.add 直接创建**:<br>"
    "  - materials.add(StandardMaterial { base_color, .. }) 直接将数据加入 Assets&lt;T&gt; 存储，同步操作不经过文件；<br>"
    "  - 返回 Handle&lt;StandardMaterial&gt; 可立即绑定到实体，⚠️ 与 load 不同——add 是同步的；<br>"
    "- **Mesh3d + MeshMaterial3d 组合**:<br>"
    "  - commands.spawn((Mesh3d(handle), MeshMaterial3d(material))) 将网格和材质绑定到实体；<br>"
    "  - 🔧 多个实体可共享同一 Handle，GPU 侧自动合批渲染；<br>"
    "- **注意事项**:<br>"
    "  - ⚠️ 路径相对于 assets/ 目录，可通过 AssetPlugin::file_path 覆盖；<br>"
    "  - 🔧 asset_server.load 同一路径多次调用返回相同 Handle（内置缓存），不会重复加载；",
    "知：<br>"
    "- AssetServer::load 异步加载与 Handle&lt;T&gt; 句柄；<br>"
    "- load_folder / LoadedFolder 文件夹批量加载；<br>"
    "- materials.add 直接创建材质资产。<br>"
    "例：<br>"
    "- 2D/sprite.rs(2)。",
)

# Entry 160: asset_settings.rs
add(
    160,
    "资产管理",
    "asset_settings.rs",
    "**意图**：演示三种配置图像加载参数的方式——默认加载（线性模糊）、.meta 文件覆盖、load_with_settings 闭包编程覆盖。⭐⭐<br>"
    "- **默认加载（线性过滤）**:<br>"
    "  - asset_server.load(path) 使用 ImagePlugin::default() 的线性过滤采样器；<br>"
    "  - 小纹理放大显示时线性插值导致模糊——像素风游戏需要 nearest 过滤；<br>"
    "  - ⚠️ ImagePlugin 默认采样器 ≠ ImageSampler::default()——前者是 linear，后者是 nearest；<br>"
    "- **.meta 文件覆盖**:<br>"
    "  - 同名 .meta 文件（如 bevy_pixel_dark.png.meta）指定 ImageLoaderSettings 全部字段；<br>"
    "  - ⚠️ .meta 文件必须覆盖 loader settings 的所有字段，不能只写部分；<br>"
    "  - 🔧 参考 ImageLoaderSettings::default() 作为填写模板；<br>"
    "- **load_with_settings 闭包**:<br>"
    "  - asset_server.load_with_settings(path, |settings: &amp;mut ImageLoaderSettings| { settings.sampler = ImageSampler::nearest(); })；<br>"
    "  - 闭包只修改需要的字段，其余保持默认——比 .meta 文件更灵活；<br>"
    "  - ⚠️ 同一资产第二次 load_with_settings 的设置会被忽略——需使用不同文件名的副本；<br>"
    "- **AssetPlugin file_path 覆盖**:<br>"
    "  - DefaultPlugins.set(AssetPlugin { file_path: path }) 改变资产搜索目录；<br>"
    "  - 🔧 示例中设为 examples/asset/files 定位到示例专用资源目录；",
    "知：<br>"
    "- ImagePlugin 采样器默认行为（linear vs nearest）；<br>"
    "- .meta 文件覆盖加载参数；<br>"
    "- load_with_settings 闭包编程式覆盖。<br>"
    "例：<br>"
    "- asset/asset_loading.rs(159)。",
)

# Entry 161: hot_asset_reloading.rs
add(
    161,
    "资产管理",
    "hot_asset_reloading.rs",
    "**意图**：资产文件热重载——运行时修改 mesh 文件后立即在应用中看到变化，无需重启。⭐<br>"
    "- **热重载原理**:<br>"
    "  - AssetWatcher 监控 assets/ 目录文件变化，检测到修改后自动重新加载对应资产；<br>"
    "  - SceneRoot(scene_handle) 组件引用的 glTF 场景在文件变化后自动刷新；<br>"
    "  - ⚠️ 需要启用 file_watcher cargo feature——默认桌面平台已包含；<br>"
    "- **SceneRoot 组件**:<br>"
    "  - SceneRoot(handle) 标记实体为场景实例，Bevy 自动管理场景层级的 spawn/despawn；<br>"
    "  - 与 DynamicSceneRoot 不同——SceneRoot 用于静态 glTF 场景，DynamicSceneRoot 用于 .scn.ron；<br>"
    "  - 🔧 修改 torus.gltf 文件后场景自动重建，适合美术迭代工作流；<br>"
    "- **场景搭建**:<br>"
    "  - DirectionalLight + Transform::looking_at 从固定角度照射场景；<br>"
    "  - Camera3d + Transform::looking_at 相机对准场景中心；<br>"
    "  - 🔧 热重载对材质/纹理/动画同样有效——不仅限于网格；<br>"
    "- **注意事项**:<br>"
    "  - ⚠️ 战斗运行时禁止热重载 Definition 配置数据（参见项目架构规则）；<br>"
    "  - 🔬 大型资产热重载可能短暂卡顿——仅适合开发阶段；",
    "知：<br>"
    "- AssetWatcher 文件监控与自动重载；<br>"
    "- SceneRoot 场景根组件；<br>"
    "- file_watcher cargo feature。<br>"
    "例：<br>"
    "- asset/asset_loading.rs(159)。",
)

# Entry 162: multi_asset_sync.rs
add(
    162,
    "资产管理",
    "multi_asset_sync.rs",
    "**意图**：等待多个资产同时加载完成——使用 AssetBarrier 同步屏障（Arc+Atomic）配合 sync 轮询和 async 异步两种等待模式。⭐⭐⭐<br>"
    "- **AssetBarrier 同步屏障**:<br>"
    "  - AssetBarrier(Arc&lt;AssetBarrierInner&gt;) 使用 AtomicU32 计数器追踪未完成的 Guard 数量；<br>"
    "  - AssetBarrierGuard clone 时 fetch_add(1)，drop 时 fetch_sub(1)——Drop 即完成通知；<br>"
    "  - is_ready() 检查 count == 0 判断所有资产是否就绪；<br>"
    "  - ⚠️ Event (event_listener) 在 count 归零时 notify(usize::MAX) 唤醒所有等待者；<br>"
    "- **load_acquire 带 Guard 加载**:<br>"
    "  - asset_server.load_acquire(path, guard.clone()) 将 Guard 与资产加载绑定；<br>"
    "  - 资产加载完成后 Guard 被 drop，count 自动递减——无需手动追踪；<br>"
    "  - 🔧 同一 Guard 可 clone 给多个 load_acquire 调用，全部完成后 barrier 才就绪；<br>"
    "- **同步轮询模式**:<br>"
    "  - assets_loaded(run condition) 轮询 Barrier::is_ready()，返回 true 时执行 wait_on_load；<br>"
    "  - wait_on_load 中 gltfs.get(&amp;handle).unwrap() 安全解包——因为 barrier 保证资产已就绪；<br>"
    "  - 🔧 run_if 模式简单可靠，适合帧率敏感度低的场景；<br>"
    "- **Async 异步模式**:<br>"
    "  - AsyncComputeTaskPool::get().spawn(barrier.wait_async()) 在计算线程池等待；<br>"
    "  - wait_async() 返回 Future——循环 listen+check 直到 count == 0；<br>"
    "  - AsyncLoadingState(Arc&lt;AtomicBool&gt;) 桥接异步结果到 ECS——通过 Ordering::Acquire/Release 保证可见性；<br>"
    "- **States 状态机**:<br>"
    "  - LoadingState::Loading → Loaded 两态状态机控制加载/完成切换；<br>"
    "  - OnExit(Loading::Loading) 调度中 despawn 临时实体和资源——清理加载阶段产物；<br>"
    "  - 🔧 States 适合管理应用级生命周期（加载屏→主菜单→游戏中）；",
    "知：<br>"
    "- AssetBarrier/Arc&lt;AtomicU32&gt; 同步屏障模式；<br>"
    "- load_acquire 带 Guard 资产加载；<br>"
    "- AsyncComputeTaskPool 异步等待；<br>"
    "States 状态机与 OnExit 调度。<br>"
    "例：<br>"
    "- asset/asset_loading.rs(159)。",
)

# Entry 163: repeated_texture.rs
add(
    163,
    "资产管理",
    "repeated_texture.rs",
    "**意图**：配置纹理重复（Repeat）模式替代默认边缘拉伸（Clamp），使用 uv_transform 控制纹理平铺比例。⭐⭐<br>"
    "- **ImageAddressMode 寻址模式**:<br>"
    "  - ImageAddressMode::Repeat UV 超出 [0,1] 时纹理重复平铺；<br>"
    "  - ImageAddressMode::Clamp（默认）超出范围时拉伸边缘像素；<br>"
    "  - ⚠️ 同一纹理文件不能同时以 Repeat 和 Clamp 加载——load_with_settings 按首次加载锁定；<br>"
    "- **load_with_settings 配置采样器**:<br>"
    "  - ImageLoaderSettings { sampler: ImageSampler::Descriptor(ImageSamplerDescriptor { address_mode_u/v: Repeat, .. }) }；<br>"
    "  - 闭包式设置：|s: &amp;mut _| { *s = ImageLoaderSettings { sampler: ..., .. } }；<br>"
    "  - 🔧 需要完整替换 ImageLoaderSettings 结构体，不能只设部分字段；<br>"
    "- **uv_transform 纹理变换**:<br>"
    "  - StandardMaterial { uv_transform: Affine2::from_scale(Vec2::new(2., 3.)) } 将纹理 U 方向重复 2 次、V 方向重复 3 次；<br>"
    "  - Affine2 是完整的仿射变换矩阵——支持缩放/位移/旋转；<br>"
    "  - ⚠️ uv_transform 单独使用时（无 Repeat 模式），超出部分显示边缘拉伸而非重复；<br>"
    "- **三立方体对比**:<br>"
    "  - 中间：默认 Clamp（边缘拉伸）<br>"
    "  - 左侧：Repeat + uv_transform =2×3（纹理平铺 2×3 次）<br>"
    "  - 右侧：Clamp + uv_transform =2×3（边缘拉伸填充 2×3 区域）<br>"
    "  - 🔧 左右对比直观展示 Repeat 与 Clamp 在相同 uv_transform 下的差异；",
    "知：<br>"
    "- ImageAddressMode::Repeat / Clamp 寻址模式；<br>"
    "- load_with_settings 配置采样器；<br>"
    "- Affine2 uv_transform 纹理变换。<br>"
    "例：<br>"
    "- 2D/mesh2d_repeated_texture.rs(17)。",
)

# Entry 164: web_asset.rs
add(
    164,
    "资产管理",
    "web_asset.rs",
    "**意图**：使用 WebAssetPlugin 从 HTTPS URL 加载远程资产，与本地资产使用相同的 AssetServer API。⭐<br>"
    "- **WebAssetPlugin 配置**:<br>"
    "  - DefaultPlugins.set(WebAssetPlugin { silence_startup_warning: true }) 注册 HTTPS 资产源；<br>"
    "  - 需要 https cargo feature 启用——默认不包含；<br>"
    "  - 可选 web_asset_cache feature 提供永不失效的缓存机制；<br>"
    "- **URL 作为资产路径**:<br>"
    "  - asset_server.load(url) 直接传入完整 HTTPS URL——与本地路径使用相同 API；<br>"
    "  - ⚠️ URL 必须可公开访问，需考虑 CORS 策略和网络延迟；<br>"
    "  - 🔧 适合加载远程纹理/字体/配置，无需打包到 assets/ 目录；<br>"
    "- **Sprite 加载**:<br>"
    "  - Sprite::from_image(asset_server.load(url)) 一行完成远程图片加载和渲染；<br>"
    "  - 🔧 加载失败时 Sprite 静默显示空白——AssetServer 不阻塞主线程；<br>"
    "- **注意事项**:<br>"
    "  - ⚠️ silence_startup_warning: true 抑制首次加载时的警告信息；<br>"
    "  - 🔬 网络延迟影响首帧显示——建议配合加载屏使用；",
    "知：<br>"
    "- WebAssetPlugin HTTPS 资产源；<br>"
    "- url 作为 asset_server.load 参数。<br>"
    "例：<br>"
    "- asset/asset_loading.rs(159)。",
)

# Entry 165: extra_source.rs
add(
    165,
    "资产管理",
    "extra_source.rs",
    "**意图**：注册额外的自定义资产源（AssetSource），从非默认目录加载资产。⭐⭐<br>"
    "- **register_asset_source 注册**:<br>"
    "  - app.register_asset_source(id, builder) 在 AssetPlugin 构建前注册新资产源；<br>"
    "  - AssetSourceBuilder::platform_default(path, watcher) 创建平台默认的文件系统资产源；<br>"
    "  - ⚠️ 必须在 DefaultPlugins 之前注册——AssetPlugin finalizing 时锁定所有已注册源；<br>"
    "- **AssetSourceId 标识**:<br>"
    '  - AssetSourceId::from("example_files") 自定义资产源名称；<br>'
    "  - 默认源为 AssetSourceId::Default——即 assets/ 目录；<br>"
    "  - 🔧 多个源可同时存在（如 assets/ + mod_assets/ + remote/），各自独立管理；<br>"
    "- **AssetPath 引用**:<br>"
    "  - AssetPath::from_path(path).with_source(source) 构造带源的资产路径；<br>"
    '  - URL 风格字符串表示："example_files://bevy_pixel_light.png"；<br>'
    "  - ⚠️ assert_eq! 验证字符串解析与 API 构造结果一致——文档化路径格式；<br>"
    "- **使用自定义源加载**:<br>"
    "  - asset_server.load(asset_path) 自动路由到对应的 AssetReader；<br>"
    "  - Sprite::from_image(...) 直接渲染从自定义源加载的图片；<br>"
    "  - 🔧 此模式适用于 MOD 系统（独立资产目录）、外包资源隔离等场景；",
    "知：<br>"
    "- register_asset_source 注册自定义资产源；<br>"
    "- AssetSourceId / AssetPath 带源路径；<br>"
    "- AssetSourceBuilder 平台默认构建器。<br>"
    "例：<br>"
    "- asset/asset_loading.rs(159)。",
)

# Entry 166: embedded_asset.rs
add(
    166,
    "资产管理",
    "embedded_asset.rs",
    "**意图**：使用 embedded_asset! 宏将资产嵌入程序内存——运行时从内存加载而非磁盘，适合加载画面等场景。⭐⭐<br>"
    "- **embedded_asset! 宏**:<br>"
    "  - embedded_asset!(app, omit_prefix, relative_path) 在 Plugin::build 中嵌入文件；<br>"
    "  - omit_prefix 从嵌入路径中去除的前缀——运行时不可见；<br>"
    "  - relative_path 相对于当前 .rs 文件的位置（include_bytes! 机制）；<br>"
    "  - ⚠️ 编译时嵌入会增加二进制大小——仅嵌入必要的小文件；<br>"
    "- **EmbeddedAssetPlugin 自定义插件**:<br>"
    "  - impl Plugin for EmbeddedAssetPlugin { fn build(&amp;self, app: &amp;mut App) } 调用 embedded_asset!；<br>"
    "  - .add_plugins((DefaultPlugins, EmbeddedAssetPlugin)) 注册嵌入插件；<br>"
    "  - 🔧 独立插件封装嵌入逻辑——主 App 无需关心嵌入细节；<br>"
    "- **AssetSourceId::embedded 访问**:<br>"
    '  - AssetPath::from_path(&amp;path).with_source(AssetSourceId::from("embedded")) 构造嵌入路径；<br>'
    '  - URL 格式："embedded://crate_name/files/image.png"；<br>'
    "  - 🔧 crate_name 来自 Cargo.toml 的 [[example]] 名称；<br>"
    "- **应用场景**:<br>"
    "  - 🧩 加载画面：在其他资产从磁盘加载时显示嵌入的 logo/splash；<br>"
    "  - 🧩 离线模式：关键 UI 资源嵌入二进制，无需外部文件依赖；<br>"
    "  - ⚠️ 运行时嵌入（本例）与编译时嵌入（build.rs）机制不同——后者更高效；",
    "知：<br>"
    "- embedded_asset! 宏编译/运行时嵌入；<br>"
    '- AssetSourceId::from("embedded") 嵌入源；<br>'
    "- EmbeddedAssetPlugin 自定义插件封装。<br>"
    "例：<br>"
    "- asset/asset_loading.rs(159)。",
)

# Entry 167: custom_asset.rs
add(
    167,
    "资产管理",
    "custom_asset.rs",
    "**意图**：实现自定义资产类型——定义 Asset 结构体、实现 AssetLoader trait、注册自定义加载器，支持 RON 反序列化。⭐⭐⭐<br>"
    "- **Asset 结构体定义**:<br>"
    "  - #[derive(Asset, TypePath, Debug, Deserialize)] 标记结构体为 Bevy 资产类型；<br>"
    "  - TypePath 是反射系统必需的——运行时获取类型路径字符串；<br>"
    "  - ⚠️ Deserialize 需要 serde 依赖——用于从文件格式（RON）反序列化；<br>"
    "- **AssetLoader trait 实现**:<br>"
    "  - type Asset = CustomAsset; type Settings = (); type Error = CustomAssetLoaderError；<br>"
    "  - async fn load(&amp;self, reader: &amp;mut dyn Reader, settings, load_context) 核心加载方法；<br>"
    "  - reader.read_to_end(&amp;mut bytes).await? 读取全部字节——异步非阻塞；<br>"
    '  - fn extensions(&amp;self) -> &amp;[&amp;str] 返回文件扩展名映射（如 ["custom"]）；<br>'
    "- **多加载器类型推断**:<br>"
    "  - 同一文件（asset.custom）可通过类型推断路由到不同加载器；<br>"
    "  - Handle&lt;CustomAsset&gt; → CustomAssetLoader，Handle&lt;Blob&gt; → BlobAssetLoader；<br>"
    "  - 🔧 类型推断基于 Handle 的泛型参数——同一 asset_server.load 调用按上下文分派；<br>"
    "- **错误类型设计**:<br>"
    "  - #[non_exhaustive] 标记错误枚举——允许未来添加新变体不破坏 API；<br>"
    '  - #[error("...")] thiserror 派生自动实现 Display——友好的错误信息；<br>'
    "  - Io + RonSpannedError 覆盖 IO 和解析两类错误源；<br>"
    "- **注册与加载**:<br>"
    "  - .init_asset::&lt;T&gt;() + .init_asset_loader::&lt;Loader&gt;() 注册资产类型和加载器；<br>"
    '  - asset_server.load("data/asset.custom") 按扩展名自动路由到对应加载器；<br>'
    "  - ⚠️ 无扩展名的文件仍可加载——但推荐使用扩展名便于管理；",
    "知：<br>"
    "- #[derive(Asset)] 自定义资产类型；<br>"
    "- AssetLoader trait 实现（load/extensions）；<br>"
    "- init_asset / init_asset_loader 注册；<br>"
    "- 类型推断路由多加载器。<br>"
    "例：<br>"
    "- asset/asset_loading.rs(159)。",
)

# Entry 168: custom_asset_reader.rs
add(
    168,
    "资产管理",
    "custom_asset_reader.rs",
    "**意图**：实现自定义 AssetReader（IO 层）——包装默认读取器添加日志，拦截资产的原始字节读取。⭐⭐<br>"
    "- **AssetReader trait 实现**:<br>"
    "  - async fn read(&amp;self, path) 读取资产原始字节——返回 impl Reader；<br>"
    "  - async fn read_meta(&amp;self, path) 读取 .meta 配置文件；<br>"
    "  - async fn read_directory(&amp;self, path) 遍历目录内容；<br>"
    "  - async fn is_directory(&amp;self, path) 检查路径是否为目录；<br>"
    "  - ⚠️ 所有方法都是 async——AssetReader 在异步上下文中运行；<br>"
    "- **包装模式（Decorator）**:<br>"
    "  - CustomAssetReader(Box&lt;dyn ErasedAssetReader&gt;) 包装现有读取器；<br>"
    '  - read 实现中先 info!("Reading {}", path) 记录日志，再委托 self.0.read(path)；<br>'
    "  - 🔧 装饰器模式可透明添加日志/缓存/加密等横切关注点；<br>"
    "- **注册自定义读取器**:<br>"
    "  - app.register_asset_source(Default, AssetSourceBuilder::new(|| Box::new(reader)))；<br>"
    '  - AssetSource::get_default_reader("assets") 获取平台默认读取器作为被包装对象；<br>'
    "  - ⚠️ 注册 Default 源会替换原有的 assets/ 目录读取器——必须包装而非替换；<br>"
    "- **CustomAssetReaderPlugin**:<br>"
    "  - impl Plugin 封装注册逻辑——.add_plugins((CustomAssetReaderPlugin, DefaultPlugins))；<br>"
    "  - 🔧 插件必须在 DefaultPlugins 之前注册——AssetPlugin finalizing 时锁定读取器；<br>"
    "- **注意事项**:<br>"
    "  - ErasedAssetReader trait object 隐藏底层具体类型——支持跨平台不同实现；<br>"
    "  - 🔧 可用于实现网络资产缓存、加密资产解密、虚拟文件系统等；",
    "知：<br>"
    "- AssetReader trait 四个异步方法；<br>"
    "- ErasedAssetReader trait object 包装模式；<br>"
    "- register_asset_source 注册自定义读取器。<br>"
    "例：<br>"
    "- asset/custom_asset.rs(167)。",
)

# Entry 169: asset_decompression.rs
add(
    169,
    "资产管理",
    "asset_decompression.rs",
    "**意图**：实现 Gzip 压缩资产的解压加载器——读取 .gz 文件、解压后委托内部加载器处理实际资产类型。⭐⭐<br>"
    "- **GzAssetLoader 结构**:<br>"
    "  - GzAsset { uncompressed: ErasedLoadedAsset } 存储解压后的资产引用；<br>"
    '  - impl AssetLoader for GzAssetLoader，extensions 返回 ["gz"]；<br>'
    "  - ⚠️ .gz 扩展名触发此加载器——内部根据去掉 .gz 后的扩展名推断实际类型；<br>"
    "- **解压 + 嵌套加载**:<br>"
    '  - load_context.path().path().file_name() 获取压缩文件名，strip_suffix(".gz") 去掉后缀；<br>'
    "  - load_context.loader().with_unknown_type().immediate() 获取类型推断加载器；<br>"
    "  - .with_reader(&amp;mut reader).load(contained_path).await 将解压数据委托给内部加载器；<br>"
    "  - 🔧 此模式实现了「透明解压层」——上层代码无需知道资产被压缩过；<br>"
    "- **Compressed&lt;T&gt; 泛型组件**:<br>"
    "  - Compressed&lt;T&gt; { compressed: Handle&lt;GzAsset&gt;, _phantom: PhantomData&lt;T&gt; } 标记待解压实体；<br>"
    "  - decompress&lt;T, A&gt; 泛型系统匹配 Compressed&lt;A&gt; 组件，解压后替换为 T 组件；<br>"
    "  - compressed_assets.remove(&amp;handle) 取出并消费 GzAsset——一次性操作避免重复解压；<br>"
    "- **类型安全解压**:<br>"
    "  - uncompressed.take::&lt;A&gt;().unwrap() 从 ErasedLoadedAsset 提取具体类型；<br>"
    "  - commands.entity(e).remove::&lt;Compressed&lt;A&gt;&gt;().insert(T::from(handle)) 替换组件；<br>"
    "  - ⚠️ take 会消费原始 ErasedLoadedAsset——同一资产只能解压一次；<br>"
    "- **错误类型设计**:<br>"
    "  - IndeterminateFilePath：无法从文件名推断未压缩文件路径；<br>"
    "  - LoadDirectError：内部加载器失败——通过 #[from] 自动转换；",
    "知：<br>"
    "- AssetLoader 实现透明解压层；<br>"
    "- load_context.loader().with_unknown_type() 嵌套加载；<br>"
    "- ErasedLoadedAsset 类型擦除与 take::&lt;T&gt;() 类型恢复。<br>"
    "例：<br>"
    "- asset/custom_asset.rs(167)。",
)

# Entry 170: processing/asset_processing.rs
add(
    170,
    "资产管理",
    "processing/asset_processing.rs",
    "**意图**：资产处理管线——自定义 AssetLoader 加载 CoolText RON、AssetTransformer 追加文本、AssetSaver 输出为纯文本，实现加载→变换→保存全流程。⭐⭐⭐<br>"
    "- **AssetMode::Processed 处理模式**:<br>"
    "  - AssetPlugin { mode: AssetMode::Processed } 启用资产处理管线；<br>"
    "  - AssetProcessor 在后台运行，处理源资产并写入 imported_assets/ 目录；<br>"
    "  - ⚠️ 需要 asset_processor cargo feature 才会实际运行处理——否则仅配置模式；<br>"
    "- **CoolTextLoader 加载器**:<br>"
    "  - 解析 RON 格式的 CoolTextRon { text, dependencies, embedded_dependencies, dependencies_with_settings }；<br>"
    "  - load_context.loader().immediate().load::&lt;Text&gt;(&amp;path) 同步加载依赖资产；<br>"
    "  - load_context.load(path) 异步加载依赖——返回 Handle 不阻塞；<br>"
    "  - 🔧 embedded_dependencies 在加载时展开并拼接到文本中——依赖注入模式；<br>"
    "- **CoolTextTransformer 变换器**:<br>"
    "  - impl AssetTransformer：type AssetInput = CoolText; type AssetOutput = CoolText；<br>"
    "  - transform 方法追加 settings.appended 字符串到文本末尾；<br>"
    "  - ⚠️ Transformer 可以改变资产类型（Input ≠ Output），但本例保持相同类型；<br>"
    "- **CoolTextSaver 保存器**:<br>"
    "  - type OutputLoader = TextLoader——保存为纯文本格式；<br>"
    "  - save 方法将 asset.text 写入 Writer——CoolText→Text 格式转换；<br>"
    "  - 🔧 Saver 将自定义格式（.cool.ron）处理为标准格式（.txt），运行时按 Text 加载；<br>"
    "- **LoadTransformAndSave 管线**:<br>"
    "  - register_asset_processor::&lt;LoadTransformAndSave&lt;Loader, Transformer, Saver&gt;&gt;() 注册三段管线；<br>"
    "  - set_default_asset_processor 对 .cool.ron 文件自动应用此管线；<br>"
    "  - 🧩 加载→变换→保存三阶段：CoolTextLoader→CoolTextTransformer→CoolTextSaver；<br>"
    "- **embedded_asset! + 热重载**:<br>"
    '  - embedded_asset!(app, "...", "e.txt") 嵌入文本文件；<br>'
    "  - MessageReader&lt;AssetEvent&lt;Text&gt;&gt; 监听资产变化事件——热重载时自动更新输出；<br>"
    "  - 🔧 修改源 .cool.ron 文件后自动重新处理并热重载；",
    "知：<br>"
    "- AssetLoader / AssetTransformer / AssetSaver 三段管线；<br>"
    "- LoadTransformAndSave 注册处理器；<br>"
    "- AssetMode::Processed 处理模式；<br>"
    "- load_context.loader().immediate() 同步嵌套加载。<br>"
    "例：<br>"
    "- asset/custom_asset.rs(167)。",
)

# Entry 171: alter_mesh.rs
add(
    171,
    "资产管理",
    "alter_mesh.rs",
    "**意图**：运行时修改已加载的 Mesh 资产——通过 Assets&lt;Mesh&gt;::get_mut 操纵顶点位置，以及替换 Handle 切换不同网格。⭐⭐<br>"
    "- **Handle 替换（Space 键）**:<br>"
    "  - mesh.0 = asset_server.load(new_path) 替换实体的 Mesh3d 句柄——切换到新网格；<br>"
    "  - asset_server.load 同一路径返回缓存 Handle——不会重复从磁盘加载；<br>"
    "  - 🔧 Handle 替换仅影响当前实体——不影响其他使用旧 Handle 的实体；<br>"
    "- **Mesh 资产修改（Enter 键）**:<br>"
    "  - meshes.get_mut(*handle) 获取 Mesh 资产的可变引用——修改所有共享此 Handle 的实体；<br>"
    "  - mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) 获取顶点位置数据；<br>"
    "  - VertexAttributeValues::Float32x3(positions) 匹配提取——每个顶点 [x, y, z]；<br>"
    "  - 🔧 修改 Mesh 资产是全局的——所有引用此 Handle 的实体同步变化；<br>"
    "- **RenderAssetUsages 配置**:<br>"
    "  - RenderAssetUsages::MAIN_WORLD | RENDER_WORLD 同时保留 CPU 和 GPU 数据；<br>"
    "  - load_with_settings 配置 GltfLoaderSettings { load_meshes: RenderAssetUsages::all() }；<br>"
    "  - ⚠️ 仅 RENDER_WORLD 不保留 CPU 数据——get_mut 返回 None 无法修改；<br>"
    "  - 🔧 大多数游戏不需要修改 Mesh——仅 RENDER_WORLD 节省内存；<br>"
    "- **Shape 枚举组件**:<br>"
    "  - Shape::Cube / Shape::Sphere 枚举组件携带当前形状状态；<br>"
    "  - set_next_variant() 切换枚举变体——状态与 Handle 同步更新；<br>"
    "  - 🔧 组件存储状态+Handle 绑定，实现「状态驱动资源切换」模式；<br>"
    "- **Single + Without 过滤**:<br>"
    "  - Single&lt;(&amp;mut Mesh3d, &amp;mut Shape), Without&lt;Left&gt;&gt; 精确获取右侧实体；<br>"
    "  - 🔧 With/Without 组合实现多实体差异化操作——同一系统处理不同行为；",
    "知：<br>"
    "- Assets&lt;Mesh&gt;::get_mut 运行时修改 Mesh；<br>"
    "- RenderAssetUsages::all() CPU+GPU 保留策略；<br>"
    "- Handle 替换 vs 资产修改的区别。<br>"
    "例：<br>"
    "- 3D/generate_custom_mesh.rs(49)。",
)

# Entry 172: alter_sprite.rs
add(
    172,
    "资产管理",
    "alter_sprite.rs",
    "**意图**：运行时修改纹理资产——通过 Assets&lt;Image&gt;::get_mut 逐像素操作图像数据，以及替换 Handle 切换不同纹理。⭐⭐<br>"
    "- **Handle 替换（Space 键）**:<br>"
    "  - sprite.image = asset_server.load(path) 替换 Sprite 的图像句柄——切换到新纹理；<br>"
    "  - 🔧 仅影响当前 Sprite 实体——其他使用旧纹理的实体不受影响；<br>"
    "- **Image 资产修改（Enter 键）**:<br>"
    "  - images.get_mut(&amp;handle) 获取 Image 资产的可变引用——修改共享此纹理的所有 Sprite；<br>"
    "  - image.data.as_mut().unwrap() 获取原始像素字节 Vec&lt;u8&gt;；<br>"
    "  - *pixel = 255 - *pixel 逐字节反转颜色——演示全局纹理修改效果；<br>"
    "  - ⚠️ 修改 Image.data 是全局的——所有引用此 Handle 的实体同步变化；<br>"
    "- **RenderAssetUsages 配置**:<br>"
    "  - load_with_settings(|settings: &amp;mut ImageLoaderSettings| settings.asset_usage = RenderAssetUsages::all())；<br>"
    "  - ⚠️ 默认 RENDER_WORLD 不保留 CPU 数据——无法 get_mut 修改；<br>"
    "  - 🔧 运行时修改纹理必须设为 all()——内存换功能；<br>"
    "- **Bird 枚举组件**:<br>"
    "  - Bird::Normal / Bird::Logo 枚举组件携带当前纹理状态；<br>"
    "  - get_texture_path() 按变体返回纹理路径——状态与 Handle 同步；<br>"
    "  - 🔧 枚举组件+方法模式让状态切换逻辑自包含；<br>"
    "- **Left 标记组件**:<br>"
    "  - Left 零大小标记组件 + Without&lt;Left&gt; 区分左右两个 Sprite；<br>"
    "  - 左侧：修改 Image 资产（影响所有使用此纹理的实体）；<br>"
    "  - 右侧：替换 Handle（仅影响当前实体）；<br>"
    "  - 🧩 两种修改方式的对比是本示例的核心教学点；",
    "知：<br>"
    "- Assets&lt;Image&gt;::get_mut 运行时修改纹理像素；<br>"
    "- RenderAssetUsages::all() 保留 CPU 数据；<br>"
    "- Handle 替换 vs 资产修改的区别。<br>"
    "例：<br>"
    "- asset/alter_mesh.rs(171)。",
)

# Entry 173: scene.rs
add(
    173,
    "场景",
    "scene.rs",
    "**意图**：从文件加载场景数据并动态应用到实体——演示 Component 反射注册、skip_serializing 跳过字段、DynamicScene 序列化输出。⭐⭐⭐<br>"
    "- **Component 反射注册**:<br>"
    "  - #[derive(Component, Reflect, Default)] + #[reflect(Component)] 启用组件的反射能力；<br>"
    "  - Reflect 是序列化/反序列化/动态操作的前提——Scene 系统依赖反射来读写组件；<br>"
    "  - ⚠️ 不加 #[reflect(Component)] 则组件无法被 Scene 系统识别和操作；<br>"
    "- **skip_serializing 跳过字段**:<br>"
    "  - #[reflect(skip_serializing)] 标记运行时字段（如 _time_since_startup）不写入场景文件；<br>"
    "  - ⚠️ 被跳过的字段在反序列化时使用 Default 或 FromWorld 初始化——不是保留旧值；<br>"
    "  - 🔧 游戏中 HP/MP 等运行时数值应标记 skip——场景文件只存配置数据；<br>"
    "- **FromWorld 运行时初始化**:<br>"
    "  - impl FromWorld for ComponentB { fn from_world(world: &amp;mut World) -> Self }；<br>"
    "  - 从 World 获取 Time 资源设置初始值——运行时才能确定的数据；<br>"
    "  - 🔧 FromWorld 适合需要访问 ECS 资源才能初始化的组件字段；<br>"
    "- **DynamicSceneRoot 加载**:<br>"
    "  - commands.spawn(DynamicSceneRoot(asset_server.load(path))) 加载 .scn.ron 场景文件；<br>"
    "  - DynamicSceneRoot 创建父实体，场景中的实体作为子实体 spawn；<br>"
    "  - ⚠️ SceneRoot 用于 glTF 场景，DynamicSceneRoot 用于 .scn.ron——两者不可混用；<br>"
    "- **DynamicScene 序列化保存**:<br>"
    "  - DynamicScene::from_world(&amp;scene_world) 从 World 创建场景快照；<br>"
    "  - scene.serialize(&amp;type_registry) 序列化为 RON 字符串——需要 AppTypeRegistry；<br>"
    "  - IoTaskPool::get().spawn(async { file.write(...) }).detach() 异步写文件避免阻塞主线程；<br>"
    "  - ⚠️ Wasm 平台无文件系统——IoTaskPool 写文件在 Web 上不可用；<br>"
    "- **Changed&lt;T&gt; 变更检测**:<br>"
    "  - Query&lt;(Entity, &amp;ComponentA), Changed&lt;ComponentA&gt;&gt; 仅在组件变化时执行；<br>"
    "  - 🔧 配合 log_system 在控制台输出变更日志——调试场景加载的有效手段；",
    "知：<br>"
    "- DynamicSceneRoot 场景加载；<br>"
    "- #[reflect(Component)] + skip_serializing 反射注册；<br>"
    "- FromWorld 运行时初始化组件；<br>"
    "- DynamicScene::from_world + serialize 序列化。<br>"
    "例：<br>"
    "- asset/asset_loading.rs(159)；<br>"
    "- reflection/reflection.rs(275)。",
)

# Entry 174: time.rs
add(
    174,
    "时间",
    "time.rs",
    "**意图**：演示 Bevy 三种时间源——Time&lt;Real&gt;（真实挂钟）、Time&lt;Virtual&gt;（可暂停/缩放）、Time&lt;Fixed&gt;（固定步长），以及自定义 Runner 控制更新时机。⭐⭐<br>"
    "- **三种时间源**:<br>"
    "  - Time&lt;Real&gt;：不受任何控制的真实墙钟时间——PreUpdate 调度中可用；<br>"
    "  - Time&lt;Virtual&gt;：默认时间源（Update 调度），支持暂停/加速/减速；<br>"
    "  - Time&lt;Fixed&gt;：固定步长时间——FixedUpdate 调度中 delta 恒为配置值；<br>"
    "  - 🔧 Res&lt;Time&gt; 在 Update 中自动解析为 Time&lt;Virtual&gt;，在 FixedUpdate 中自动解析为 Time&lt;Fixed&gt;；<br>"
    "- **FixedUpdate 固定步长**:<br>"
    "  - Time::&lt;Fixed&gt;::from_duration(Duration::from_secs(1)) 每秒执行 1 次 FixedUpdate；<br>"
    "  - delta 在 FixedUpdate 中恒为 1 秒——不受帧率影响；<br>"
    "  - ⚠️ 帧率 &gt; 1Hz 时 FixedUpdate 可能每帧执行多次（累积追赶），也可能 0 次（帧间隔 &lt; 步长）；<br>"
    "- **Virtual 时间控制**:<br>"
    "  - set_relative_speed(2.0) 加速 2 倍——delta_secs 翻倍；<br>"
    "  - pause() / unpause() 暂停/恢复——delta_secs 归零；<br>"
    "  - from_max_delta(Duration::from_secs(5)) 限制最大 delta——防止卡顿后跳跃；<br>"
    "  - 🔧 暂停时 Virtual delta=0 但 Real delta 正常——两者独立运行；<br>"
    "- **自定义 Runner**:<br>"
    "  - .set_runner(runner_fn) 替换默认事件循环——手动控制 app.update() 时机；<br>"
    "  - fn runner(mut app: App) -> AppExit 自定义运行器函数；<br>"
    "  - ⚠️ 自定义 Runner 负责调用 app.finish() + app.cleanup()——否则资源未就绪；<br>"
    "- **stdin 交互控制**:<br>"
    "  - 从标准输入读取命令（f=快速/n=正常/s=慢/p=暂停/u=恢复/q=退出）；<br>"
    "  - app.world_mut().resource_mut::&lt;Time&lt;Virtual&gt;&gt;() 直接修改时间资源——绕过系统参数；<br>"
    "  - 🔧 此模式适合测试/调试场景——手动控制时间推进观察行为；",
    "知：<br>"
    "- Time&lt;Real&gt; / Time&lt;Virtual&gt; / Time&lt;Fixed&gt; 三种时间源；<br>"
    "- set_relative_speed / pause / unpause 虚拟时间控制；<br>"
    "- from_max_delta 最大帧间隔限制。<br>"
    "例：<br>"
    "- hello_world(1)。",
)

# Entry 175: timers.rs
add(
    175,
    "时间",
    "timers.rs",
    "**意图**：Timer 作为 Component 和 Resource 的两种使用模式——单次/重复计时、进度查询、暂停控制。⭐⭐<br>"
    "- **Timer 作为 Component**:<br>"
    "  - #[derive(Component, Deref, DerefMut)] struct PrintOnCompletionTimer(Timer)；<br>"
    "  - Deref/DerefMut 让系统内直接调用 timer.tick(delta)——无需解构元组；<br>"
    "  - Timer::from_seconds(5.0, TimerMode::Once) 单次计时——完成后自动停止；<br>"
    "  - ⚠️ Component Timer 需要在系统中手动 tick——不会自动推进；<br>"
    "- **Timer 作为 Resource**:<br>"
    "  - #[derive(Resource)] struct Countdown { percent_trigger, main_timer } 包含多个计时器；<br>"
    "  - init_resource::&lt;Countdown&gt;() 自动 Default 初始化——首次创建时调用 Countdown::new()；<br>"
    "  - 🔧 Resource Timer 适合全局状态（倒计时/波次间隔），Component Timer 适合实体状态（技能冷却）；<br>"
    "- **TimerMode::Once vs Repeating**:<br>"
    "  - TimerMode::Once：计时完成后 is_finished()=true 但不再推进——适合一次性触发；<br>"
    "  - TimerMode::Repeating：计时完成后自动重置——just_finished() 周期性返回 true；<br>"
    "  - ⚠️ Repeating Timer 的 delta 累积到下一次循环——不会丢失超出的时间；<br>"
    "- **tick + just_finished 模式**:<br>"
    "  - timer.tick(time.delta()).just_finished() 一行完成推进和检测——返回 bool；<br>"
    "  - ⚠️ just_finished() 仅在完成的那一帧返回 true——is_finished() 在整个完成后持续返回 true；<br>"
    "  - 🔧 Repeating Timer 用 just_finished() 触发周期事件，用 is_finished() 判断是否已开始；<br>"
    "- **fraction 进度查询**:<br>"
    "  - countdown.main_timer.fraction() 返回 0.0~1.0 的完成比例——用于进度条/UI 显示；<br>"
    "  - 🔧 fraction() × 100.0 可直接输出百分比文本；<br>"
    "- **pause 暂停**:<br>"
    "  - countdown.percent_trigger.pause() 暂停百分比输出计时器——tick 时不再推进；<br>"
    "  - 🔧 pause/resume 控制 Timer 独立于 Time 资源的推进——适合按需暂停的子系统；",
    "知：<br>"
    "- Timer Component vs Resource 两种使用模式；<br>"
    "- TimerMode::Once vs Repeating；<br>"
    "- tick() + just_finished() + fraction() API。<br>"
    "例：<br>"
    "- time/time.rs(174)。",
)

# Entry 176: virtual_time.rs
add(
    176,
    "时间",
    "virtual_time.rs",
    "**意图**：Time&lt;Virtual&gt; 控制游戏时间的暂停/恢复/加速/减速——对比 Real（不受影响）和 Virtual（受控）两种时间驱动的 Sprite 运动。⭐⭐<br>"
    "- **Real vs Virtual 时间对比**:<br>"
    "  - Time&lt;Real&gt;：不受暂停/缩放影响的真实时间——Sprite 匀速运动；<br>"
    "  - Time&lt;Virtual&gt;：受 relative_speed 和 pause 控制——速度/位置随设置变化；<br>"
    "  - 🔧 两个 Sprite 同时运动，直观对比两种时间源的行为差异；<br>"
    "- **relative_speed 速度控制**:<br>"
    "  - time.set_relative_speed(2.) 初始设置 2 倍速——Virtual Sprite 移动速度是 Real 的两倍；<br>"
    "  - change_time_speed::&lt;DELTA&gt;() const 泛型函数：UP 键 +1，DOWN 键 -1；<br>"
    "  - .clamp(0.25, 5.) 限制速度范围——防止极端值导致异常行为；<br>"
    "  - 🔧 speed=0 等价于 pause，speed=1 等价于正常速度；<br>"
    "- **pause / unpause 切换**:<br>"
    "  - toggle_pause() 检测 is_paused() 状态后切换——Space 键交互；<br>"
    "  - 暂停时 Virtual delta_secs=0——Sprite 停止运动；<br>"
    "  - ⚠️ Real 时间不受 pause 影响——Real Sprite 继续运动；<br>"
    "- **on_real_timer 真实时间定时器**:<br>"
    "  - .run_if(on_real_timer(Duration::from_millis(250))) 使用 Real 时间触发——不被 Virtual 缩放影响；<br>"
    "  - 🔧 UI 更新使用 Real 时间确保文字刷新频率稳定——不受游戏暂停影响；<br>"
    "  - ⚠️ on_timer() 使用 Virtual 时间——暂停时 UI 也不更新，不适合信息显示；<br>"
    "- **Speed 累加器模式**:<br>"
    "  - Local&lt;f32&gt; 或直接读取 relative_speed() 获取当前速度；<br>"
    "  - (speed + DELTA).round().clamp(0.25, 5.) 计算新速度——整数步进更易操控；<br>"
    '  - 🔧 速度显示在 UI 上：format!("Speed: {:.2}", time.relative_speed())；<br>'
    "- **get_sprite_translation_x 共享函数**:<br>"
    "  - sin(elapsed) × 500.0 往复运动——两种时间源使用同一计算函数；<br>"
    "  - 🔧 相同函数+不同时间输入 = 不同运动行为——清晰展示时间对游戏逻辑的影响；",
    "知：<br>"
    "- Time&lt;Virtual&gt; relative_speed / pause / unpause 控制；<br>"
    "- Time&lt;Real&gt; 不受控的真实时间；<br>"
    "- on_real_timer vs on_timer 的时间源差异。<br>"
    "例：<br>"
    "- time/time.rs(174)。",
)


def main():
    with open(CATALOG, "r", encoding="utf-8") as f:
        lines = f.readlines()

    # Replace matching lines
    new_lines = []
    replaced_count = 0
    for line in lines:
        replaced = False
        for seq, new_line in REPLACEMENTS.items():
            if line.startswith(f"| {seq} |"):
                new_lines.append(new_line + "\n")
                replaced = True
                replaced_count += 1
                break
        if not replaced:
            new_lines.append(line)

    with open(CATALOG, "w", encoding="utf-8") as f:
        f.writelines(new_lines)

    print(f"Updated {replaced_count} entries in {CATALOG}")


if __name__ == "__main__":
    main()
