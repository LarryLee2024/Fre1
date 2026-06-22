# Bevy 0.12

## 发布于 2023 年 11 月 4 日，Bevy 贡献者们

![Jos Feenstra 制作的放松型"小星球"城市建造游戏（使用 Bevy 开发）](https://bevy.org/news/bevy-0-12/cover.gif)

[Jos Feenstra 制作的放松型"小星球"城市建造游戏（使用 Bevy 开发）](https://twitter.com/i_am_feenster)

感谢 **185** 位贡献者、**567** 个 Pull Request、社区审阅者以及我们的[**慷慨赞助商**](https://bevy.org/donate)，我们很高兴在 [crates.io](https://crates.io/crates/bevy) 上发布 **Bevy 0.12**！

对于那些还不了解的人，Bevy 是一个用 Rust 构建的、令人耳目一新的简单数据驱动游戏引擎。你可以查看我们的[快速入门指南](https://bevy.org/learn/quick-start/introduction)来立即体验。它是免费且永远开源的！你可以在 GitHub 上获取完整的[源代码](https://github.com/bevyengine/bevy)。查看 [Bevy Assets](https://bevy.org/assets) 以获取社区开发的插件、游戏和学习资源集合。

要将现有的 Bevy App 或 Plugin 更新到 **Bevy 0.12**，请查看我们的[0.11 到 0.12 迁移指南](https://bevy.org/learn/migration-guides/0.11-0.12/)。

自我们上次发布几个月以来，我们增加了大量新功能、Bug 修复和生活质量改进，以下是其中一些亮点：

- **延迟渲染（Deferred Rendering）**：（可选的）延迟风格渲染支持，补充了 Bevy 现有的 Forward+ 渲染器，增加了对新效果和不同性能权衡的支持。Bevy 现在是一个"混合渲染器"，意味着你可以同时使用两者！
- **Bevy Asset V2**：一个全新的资源系统，增加了对资源预处理、资源配置（通过 .meta 文件）、多资源来源、递归依赖加载跟踪等更多功能的支持！
- **PCF 阴影过滤**：Bevy 现在通过百分比渐近过滤（Percentage-Closer Filtering）实现了更平滑的阴影。
- **StandardMaterial 光传输**：Bevy 的 PBR 材质现在支持光传输，使得模拟玻璃、水、塑料、植物、纸张、蜡、大理石等材质成为可能。
- **材质扩展（Material Extensions）**：材质现在可以基于其他材质构建。你可以轻松编写基于现有材质（如 Bevy 的 PBR StandardMaterial）构建的着色器。
- **Rust 风格着色器导入**：Bevy 的细粒度着色器导入系统现在使用 Rust 风格的导入，扩展了导入系统的能力和可用性。
- **Android 上的暂停与恢复**：Bevy 现在支持 Android 上的暂停和恢复事件，这是我们在 Android 支持方面的最后一块重要拼图。Bevy 现已支持 Android！
- **绘制命令的自动批处理与实例化**：绘制命令现在在可能的情况下会自动进行批处理/实例化，带来了显著的渲染性能提升。
- **渲染器优化**：Bevy 的渲染器数据流已被重新设计，以榨取更多性能，并为未来的 GPU 驱动渲染做好准备。
- **一次性系统（One Shot Systems）**：ECS 系统现在可以按需从其他系统中运行！
- **UI 材质**：向 Bevy UI 节点添加自定义材质着色器。

## 延迟渲染 [#](https://bevy.org/news/bevy-0-12/#deferred-rendering)

作者：@DGriffin91

两种最流行的"渲染风格"是：

- **前向渲染（Forward Rendering）**：在单个渲染通道中完成所有材质/光照计算
    - **优点**：使用更简单。在更多硬件上工作/性能更好。支持 MSAA。透明处理良好。
    - **缺点**：光照更昂贵/场景中支持的光源更少，没有预通道时某些渲染效果不可能实现（或更难实现）
- **延迟渲染（Deferred Rendering）**：执行一个或多个预通道来收集场景的相关信息，然后在之后的最终通道中在_屏幕空间_进行材质/光照计算。
    - **优点**：实现了一些前向渲染无法实现的渲染效果。这对全局光照（GI）技术尤其重要，通过仅着色可见片元来降低着色成本，可以支持场景中更多的光源
    - **缺点**：使用更复杂。需要执行预通道，在某些情况下可能比等效的前向渲染器更昂贵（尽管反过来也可能成立），使用更多纹理带宽（在某些设备上可能禁止），不支持 MSAA，透明度更难/不那么直接。

Bevy 的渲染器历史上一直是"前向渲染器"。更具体地说，它是一个[聚类前向 / Forward+](https://bevy.org/news/bevy-0-7/#unlimited-point-lights) 渲染器，意味着我们将视锥体分解为聚类并将光源分配给这些聚类，从而能够渲染比传统前向渲染器更多的光源。

然而，随着 Bevy 的发展，它逐渐进入了"混合渲染器"的领域。在之前的版本中，我们添加了[深度和法线预通道](https://bevy.org/news/bevy-0-10/#depth-and-normal-prepass)以启用[TAA](https://bevy.org/news/bevy-0-11/#temporal-anti-aliasing)、[SSAO](https://bevy.org/news/bevy-0-11/#screen-space-ambient-occlusion)和[Alpha 纹理阴影图](https://bevy.org/news/bevy-0-10/#shadow-mapping-using-prepass-shaders)。我们还添加了运动矢量预通道以启用 TAA。

在 **Bevy 0.12** 中，我们添加了可选的延迟渲染支持（建立在现有预通道工作的基础上）。每个材质可以选择是否走前向路径还是延迟路径，并且可以按材质实例进行配置。Bevy 还有一个新的[`DefaultOpaqueRendererMethod`](https://docs.rs/bevy/0.12.0/bevy/pbr/struct.DefaultOpaqueRendererMethod.html) 资源，用于配置全局默认值。默认设置为"forward"。全局默认值可以按材质覆盖。

让我们分解这个延迟渲染的组成部分：

![deferred](https://bevy.org/news/bevy-0-12/deferred.png)

当为 PBR [`StandardMaterial`](https://docs.rs/bevy/0.12.0/bevy/pbr/struct.StandardMaterial.html) 启用延迟渲染时，延迟预通道将 PBR 信息打包到 Gbuffer 中，可以分解为：

**基础颜色（Base Color）** ![base color](https://bevy.org/news/bevy-0-12/base_color.png)

**深度（Depth）** ![depth](https://bevy.org/news/bevy-0-12/depth.png)

**法线（Normals）** ![normals](https://bevy.org/news/bevy-0-12/normals.png)

**感知粗糙度（Perceptual Roughness）** ![perceptual roughness](https://bevy.org/news/bevy-0-12/perceptual_roughness.png)

**金属度（Metallic）** ![metallic](https://bevy.org/news/bevy-0-12/metallic.png)

延迟预通道还会生成一个"延迟光照通道 ID"纹理，用于确定为片元运行哪个光照着色器：

![lighting pass ID texture](https://bevy.org/news/bevy-0-12/deferred_pass2.png)

这些被传递到最终的延迟光照着色器中。

请注意，飞行头盔模型前面的立方体和地面使用的是前向渲染，这就是为什么它们在上述两个延迟光照纹理中都是黑色的。这说明你可以在同一个场景中同时使用前向和延迟材质！

需要注意的是，对于大多数用例，我们推荐默认使用前向，除非某个功能明确需要延迟，或者你的渲染条件更适合延迟风格。前向渲染的意外最少，在更多设备上表现更好。

## PCF 阴影过滤 [#](https://bevy.org/news/bevy-0-12/#pcf-shadow-filtering)

作者：@superdump (Rob Swain), @JMS55

阴影锯齿是 3D 应用中一个非常常见的问题：

![no pcf](https://bevy.org/news/bevy-0-12/no_pcf.png)

阴影中的那些"锯齿线"是阴影图"太小"而无法从此视角准确表示阴影的结果。上面的阴影图存储在 512x512 纹理中，这比大多数人用于大多数阴影的分辨率要低。选择这个是为了展示"糟糕"的锯齿情况。请注意，Bevy 默认使用 2048x2048 阴影图。

一种"解决方案"是提高分辨率。以下是使用 4096x4096 阴影图的效果：

![no pcf high resolution](https://bevy.org/news/bevy-0-12/no_pcf_high.png)

看起来好多了！然而，这仍然不是一个完美的解决方案。大尺寸阴影图并非在所有硬件上都可行。它们要昂贵得多。而且即使你能够承受超高分辨率的阴影，如果你将物体放在错误的位置，或者将光源指向错误的方向，仍然可能遇到这个问题。你可以使用 Bevy 的[级联阴影图](https://bevy.org/news/bevy-0-10/#cascaded-shadow-maps)（默认启用）来覆盖更大区域，在靠近相机的地方提供更高的细节，远处则较少细节。然而即使在这些条件下，你仍然可能会遇到这些锯齿问题。

**Bevy 0.12** 引入了 **PCF 阴影过滤**（Percentage-Closer Filtering），这是一种流行的技术，它从阴影图中采集多个样本，并与投影到光源参考系中的插值网格表面深度进行比较。然后计算深度缓冲区中比网格表面更接近光源的样本的百分比。简而言之，这创建了一种"模糊"效果，提高了阴影质量，当某个阴影没有足够的"阴影图细节"时尤为明显。请注意，PCF 目前仅支持[`DirectionalLight`](https://docs.rs/bevy/0.12.0/bevy/pbr/struct.DirectionalLight.html)和[`SpotLight`](https://docs.rs/bevy/0.12.0/bevy/pbr/struct.SpotLight.html)。

**Bevy 0.12** 的默认 PCF 方法是 Ignacio Castaño（在《见证者》（The Witness）中使用）的[`ShadowMapFilter::Castano13`](https://docs.rs/bevy/0.12.0/bevy/pbr/enum.ShadowFilteringMethod.html#variant.Castano13)方法。以下是使用 512x512 阴影图的效果：

拖动此图像进行比较

![Castano 13 PCF](https://bevy.org/news/bevy-0-12/pcf_castano.png)![PCF Off](https://bevy.org/news/bevy-0-12/no_pcf.png)

好多了！

我们还实现了 Jorge Jimenez（在《使命召唤：高级战争》（Call of Duty Advanced Warfare）中使用）的[`ShadowMapFilter::Jimenez14`](https://docs.rs/bevy/0.12.0/bevy/pbr/enum.ShadowFilteringMethod.html#variant.Jimenez14)方法。这可能比 Castano 略便宜，但可能会闪烁。它受益于[时间性抗锯齿（TAA）](https://bevy.org/news/bevy-0-11/#temporal-anti-aliasing)，可以减少闪烁。它还可以比 Castano 更平滑地混合阴影级联。

拖动此图像进行比较

![Jimenez 14 PCF](https://bevy.org/news/bevy-0-12/pcf_jimenez.png)![PCF Off](https://bevy.org/news/bevy-0-12/no_pcf.png)

## `StandardMaterial` 光传输 [#](https://bevy.org/news/bevy-0-12/#standardmaterial-light-transmission)

作者：Marco Buono (@coreh)

[`StandardMaterial`](https://docs.rs/bevy/0.12.0/bevy/pbr/struct.StandardMaterial.html) 现在支持许多与光传输相关的属性：

- `specular_transmission`
- `diffuse_transmission`
- `thickness`
- `ior`
- `attenuation_color`
- `attenuation_distance`

这些属性使你能够更真实地表现各种物理材质，包括**透明和磨砂玻璃、水、塑料、植物、纸张、蜡、大理石、瓷器等**。

漫射透射是 PBR 光照模型中一个廉价的补充，而镜面透射则是一种相当耗费资源的屏幕空间效果，可以精确模拟折射和模糊效果。

![transmission](https://bevy.org/news/bevy-0-12/transmission.jpg)

不同的光传输属性及其与现有 PBR 属性的交互。

为了补充新的光传输属性，引入了一个新的[`TransmittedShadowReceiver`](https://docs.rs/bevy/0.12.0/bevy/pbr/struct.TransmittedShadowReceiver.html)组件，可以添加到具有漫射透射材质的实体上，以接收来自网格另一侧投射的阴影。这对于渲染薄的双面半透明物体（如树叶或纸张）最为有用。

此外，[`Camera3d`](https://docs.rs/bevy/0.12.0/bevy/core_pipeline/core_3d/struct.Camera3d.html)组件新增了两个字段：`screen_space_specular_transmission_quality` 和 `screen_space_specular_transmission_steps`。这些用于控制屏幕空间镜面透射效果的质量（采样次数），以及当多个透射物体前后重叠时支持的"透明度层数"。

> **重要提示：** 每增加一层"透明度"都会在后台产生一次纹理拷贝，增加带宽成本，因此建议将此值保持在尽可能低的水平。

最后，已添加对以下 glTF 扩展的导入支持：

- `KHR_materials_transmission`
- `KHR_materials_ior`
- `KHR_materials_volume`

[查看这个视频](https://www.youtube.com/watch?v=t1XdxZKZ-us) 来观看实际效果！

### 兼容性 [#](https://bevy.org/news/bevy-0-12/#compatibility)

镜面透射和漫射透射都与所有支持的平台兼容，包括移动端和 Web。

可选的 `pbr_transmission_textures` Cargo 功能允许使用纹理来调节 `specular_transmission`、`diffuse_transmission` 和 `thickness` 属性。默认禁用，以减少标准材质使用的纹理绑定数量（在低端平台和较旧的 GPU 上这些资源受到严重限制！）。

[`DepthPrepass`](https://docs.rs/bevy/0.12.0/bevy/core_pipeline/prepass/struct.DepthPrepass.html)和 TAA 可以极大地提高屏幕空间镜面透射效果的质量，建议在支持的平台上与其一起使用。

### 实现细节 [#](https://bevy.org/news/bevy-0-12/#implementation-details)

镜面透射通过一个新的 `Transmissive3d` 屏幕空间折射阶段实现，该阶段加入了现有的 `Opaque3d`、`AlphaMask3d` 和 `Transparent3d` 阶段。在此阶段，主纹理的一个或多个快照被拍摄，用作折射效果的"背景"。

每个片元的表面法线和 IOR（折射率）与视线方向一起用于计算折射光线（通过斯涅尔定律）。然后，该光线穿过网格体积传播（距离由 `thickness` 属性控制），产生一个出射点。在该点对"背景"纹理进行采样。感知粗糙度与交错梯度噪声和多个螺旋采样点一起使用，产生模糊效果。

漫射透射通过第二个反向且偏移的全漫射 Lambertian 瓣实现，该瓣被添加到现有的 PBR 光照计算中。这是一种简单且相对廉价的近似，但效果相当不错。

## Bevy Asset V2 [#](https://bevy.org/news/bevy-0-12/#bevy-asset-v2)

作者：@cart

资源管线是游戏开发过程的核心部分。Bevy 旧的资源系统适用于某些类型的应用，但它有许多局限性，使其无法服务于其他类型应用的需求，特别是较高端的 3D 应用。

Bevy Asset V2 是一个全新的资源系统，它吸取了 Bevy Asset V1 最佳部分的同时，增加了对许多重要场景的支持：**资源导入/预处理**、**资源元数据文件**、**多资源来源**、**递归资源依赖加载事件**、**异步资源 I/O**、**更快且功能更丰富的资源句柄**等！

大多数面向用户的现有资源代码要么完全不需要更改，要么只需最小的更改。自定义 [`AssetLoader`](https://docs.rs/bevy/0.12.0/bevy/asset/trait.AssetLoader.html)或 [`AssetReader`](https://docs.rs/bevy/0.12.0/bevy/asset/io/trait.AssetReader.html)代码需要稍作改动，但一般来说改动应该非常小。Bevy Asset V2（尽管是一个全新的实现）基本上只是扩展了 Bevy 的能力。

### 资源预处理 [#](https://bevy.org/news/bevy-0-12/#asset-preprocessing)

![image process diagram](https://bevy.org/news/bevy-0-12/image_process.png)

资源预处理是指将给定类型的输入资源以某种方式处理（通常在开发期间），然后将结果作为应用中的最终资源使用。可以将其理解为"资源编译器"。

这实现了许多场景：

- **减少发布应用中的工作量**：许多资源并非以其发布时的理想形式_组合_的。场景可能以可读性优先的文本格式定义，加载速度较慢。图像可能以需要更多工作来解码和上传到 GPU 的格式定义，或者与 GPU 友好格式相比占用更多 GPU 空间（例如 PNG 图像 vs Basis Universal）。预处理使开发者能够提前转换为发版最优格式，使应用启动更快、占用更少资源且性能更好。它还使得将_原本_会在运行时完成的计算工作转移到开发时间成为可能。例如，为图像生成 mipmap。
- **压缩**：最小化资源在已部署应用中占用的磁盘空间和/或带宽
- **转换**：某些"资源源文件"默认格式不正确。你可以拥有类型 `A` 的资源并将其转换为类型 `B`。

如果你正在构建一个用最优格式测试硬件极限的应用……或者只是想减少启动/加载时间，资源预处理就是为你准备的。

关于我们选择的实现的深入技术分析，请查看 [Bevy Asset V2 Pull Request](https://github.com/bevyengine/bevy/pull/8624)。

### 启用预处理 [#](https://bevy.org/news/bevy-0-12/#enabling-pre-processing)

要启用资源预处理，只需像这样配置你的 [`AssetPlugin`](https://docs.rs/bevy/0.12.0/bevy/asset/struct.AssetPlugin.html)：

```rust
app.add_plugins(DefaultPlugins.set(
    AssetPlugin {
        mode: AssetMode::Processed,
        ..default()
    }
))
```

这将配置资源系统在 `imported_assets` 文件夹中查找资源，而不是 `assets`"源文件夹"。在开发期间，像这样启用 `asset_processor` cargo 功能：

```sh
cargo run --features bevy/asset_processor
```

这将在你的应用并行启动 [`AssetProcessor`](https://docs.rs/bevy/0.12.0/bevy/asset/processor/struct.AssetProcessor.html)。它将运行直到所有资源从其源（默认为 `assets` 文件夹）读取、处理完成，并将结果写入其目标（默认为 `imported_assets` 文件夹）。这与资源热重载配合使用。如果你进行了更改，[`AssetProcessor`](https://docs.rs/bevy/0.12.0/bevy/asset/processor/struct.AssetProcessor.html)将检测到它，资源将被重新处理，结果将在你的应用中热重载。

### 今天应该启用预处理吗？ [#](https://bevy.org/news/bevy-0-12/#should-you-enable-pre-processing-today)

在未来的 Bevy 版本中，我们会推荐大多数应用启用处理。我们_尚不_推荐大多数用例这样做，有几个原因：

1. 我们大多数内置资源还没有为其实现处理器。[`CompressedImageSaver`](https://docs.rs/bevy/0.12.0/bevy/render/texture/struct.CompressedImageSaver.html)是唯一的内置处理器，且功能集最小。
2. 我们尚未实现"资源迁移"。每当资源更改其设置格式（在元数据文件中使用时），我们需要能够自动将现有资源元数据文件迁移到新版本。
3. 随着人们采用处理功能，我们预计在回应反馈时会经历一些变化。

### 增量与依赖感知 [#](https://bevy.org/news/bevy-0-12/#incremental-and-dependency-aware)

**Bevy Asset V2** 只会处理已更改的资源。为此，它计算并存储每个资源源文件的哈希值：

```rust
hash: (132, 61, 201, 41, 85, 80, 72, 189, 132, 81, 252, 156, 4, 227, 196, 207),
```

它还跟踪处理资源时使用的资源依赖关系。如果依赖项已更改，依赖它的资源也将被重新处理！

### 事务性与可靠性 [#](https://bevy.org/news/bevy-0-12/#transactional-and-reliable)

**Bevy Asset V2** 使用预写日志（一种数据库常用的技术）来从崩溃/强制退出中恢复。它尽可能避免完全重新处理，只重新处理未完成的事务。

[`AssetProcessor`](https://docs.rs/bevy/0.12.0/bevy/asset/processor/struct.AssetProcessor.html)可以在任何时间点关闭（无论是主动还是意外），并在下次启动时从中断处继续处理！

如果 Bevy App 请求加载当前正在处理（或重新处理）的资源，加载操作将异步等待，直到处理后的资源及其元数据文件都已写入。这确保了加载的资源文件和元数据文件在给定加载中始终"匹配"。

### 资源元数据文件 [#](https://bevy.org/news/bevy-0-12/#asset-meta-files)

资源现在支持（可选的）`.meta` 文件。这可以配置诸如以下内容：

- **资源"行为"**
    - 配置 Bevy 的资源系统应如何处理该资源：
        - `Load`：加载资源而不进行处理
        - `Process`：在加载前预处理资源
        - `Ignore`：不处理也不加载资源
- **[`AssetLoader`](https://docs.rs/bevy/0.12.0/bevy/asset/trait.AssetLoader.html) 设置**
    - 你可以使用元数据文件设置任何你想要的 [`AssetLoader`](https://docs.rs/bevy/0.12.0/bevy/asset/trait.AssetLoader.html)
    - 配置加载器设置，如"如何过滤图像"、"调整 3D 场景中的上轴"等
- **[`Process`](https://docs.rs/bevy/0.12.0/bevy/asset/processor/trait.Process.html) 设置**（如果使用 `Process` 行为）
    - 你可以使用元数据文件设置任何你想要的 [`Process`](https://docs.rs/bevy/0.12.0/bevy/asset/processor/trait.Process.html) 实现
    - 配置处理器设置，如"使用什么类型的压缩"、"是否生成 mipmap"等

未处理图像的元数据文件看起来像这样：

```rust
(
    meta_format_version: "1.0",
    asset: Load(
        loader: "bevy_render::texture::image_loader::ImageLoader",
        settings: (
            format: FromExtension,
            is_srgb: true,
            sampler: Default,
        ),
    ),
)
```

配置为处理的图像的元数据文件看起来像这样：

```rust
(
    meta_format_version: "1.0",
    asset: Process(
        processor: "bevy_asset::processor::process::LoadAndSave<bevy_render::texture::image_loader::ImageLoader, bevy_render::texture::compressed_image_saver::CompressedImageSaver>",
        settings: (
            loader_settings: (
                format: FromExtension,
                is_srgb: true,
                sampler: Default,
            ),
            saver_settings: (),
        ),
    ),
)
```

如果启用了资源处理器，元数据文件将自动为资源生成。

处理后的图像最终的"输出"元数据看起来像这样：

```rust
(
    meta_format_version: "1.0",
    processed_info: Some((
        hash: (132, 61, 201, 41, 85, 80, 72, 189, 132, 81, 252, 156, 4, 227, 196, 207),
        full_hash: (81, 90, 244, 190, 16, 134, 202, 154, 3, 211, 78, 199, 216, 21, 132, 216),
        process_dependencies: [],
    )),
    asset: Load(
        loader: "bevy_render::texture::image_loader::ImageLoader",
        settings: (
            format: Format(Basis),
            is_srgb: true,
            sampler: Default,
        ),
    ),
)
```

这就是写入 `imported_assets` 文件夹的内容。

请注意，`Process` 资源模式已变为 `Load`。这是因为在已发布的应用中，我们将像加载任何其他图像资源一样"正常地"加载最终处理后的图像。请注意，在这种情况下，输入和输出资源都使用 [`ImageLoader`](https://docs.rs/bevy/0.12.0/bevy/render/texture/struct.ImageLoader.html)。然而，处理后的资源_可以_在上下文需要时使用不同的加载器。另请注意添加了 `processed_info` 元数据，用于确定是否需要重新处理资源。

最终处理后的资源和元数据文件可以像任何其他文件一样被查看和交互。然而，它们旨在成为只读的。配置应该在_源资源_上进行，而不是_最终处理后的资源_。

### `CompressedImageSaver` [#](https://bevy.org/news/bevy-0-12/#compressedimagesaver)

![processed sponza](https://bevy.org/news/bevy-0-12/processed_sponza.png)

使用 Bevy Asset V2 将纹理处理为 Basis Universal（带 mipmap）的 Sponza 场景

**Bevy 0.12** 附带了一个基础的 [`CompressedImageSaver`](https://docs.rs/bevy/0.12.0/bevy/render/texture/struct.CompressedImageSaver.html)，它将图像写入 [Basis Universal](https://github.com/BinomialLLC/basis_universal)（一种 GPU 友好的图像交换格式）并生成 [mipmap](https://en.wikipedia.org/wiki/Mipmap)。Mipmap 减少了从不同距离采样图像时的锯齿伪影。这填补了一个重要的空白，因为 Bevy 之前无法自己生成 mipmap（依赖于外部工具）。这可以通过 `basis-universal` cargo 功能启用。

### 预处理是可选的！ [#](https://bevy.org/news/bevy-0-12/#preprocessing-is-optional)

尽管最终（[在未来的 Bevy 版本中](https://bevy.org/news/bevy-0-12/#should-you-enable-pre-processing-today)）我们会推荐大多数人启用资源处理，但我们也承认 Bevy 被用于各种应用程序。资源处理引入了额外的复杂性和工作流变化，有些人不希望这样！

这就是为什么 Bevy 提供两种资源模式：

- [`AssetMode::Unprocessed`](https://docs.rs/bevy/0.12.0/bevy/asset/enum.AssetMode.html)：资源将直接从资源源文件夹（默认为 `assets`）加载，不进行任何预处理。它们被假定为"最终格式"。这是 Bevy 用户目前习惯的模式/工作流。
- [`AssetMode::Processed`](https://docs.rs/bevy/0.12.0/bevy/asset/enum.AssetMode.html)：资源将在开发时被预处理。它们将从其源文件夹（默认为 `assets`）读取，然后写入其目标文件夹（默认为 `imported_assets`）。

为了实现这一点，Bevy 采用了一种新颖的资源处理方法：处理后的资源和未处理的资源只是视角不同。它们使用相同的 `.meta` 格式，使用相同的 [`AssetLoader`](https://docs.rs/bevy/0.12.0/bevy/asset/trait.AssetLoader.html) 接口。

[`Process`](https://docs.rs/bevy/0.12.0/bevy/asset/processor/trait.Process.html) 实现可以使用任意逻辑定义，但我们强烈鼓励使用 [`LoadAndSave`](https://docs.rs/bevy/0.12.0/bevy/asset/processor/struct.LoadAndSave.html) [`Process`](https://docs.rs/bevy/0.12.0/bevy/asset/processor/trait.Process.html) 实现。[`LoadAndSave`](https://docs.rs/bevy/0.12.0/bevy/asset/processor/struct.LoadAndSave.html)接受任何 [`AssetLoader`](https://docs.rs/bevy/0.12.0/bevy/asset/trait.AssetLoader.html) 并将结果传递给一个 [`AssetSaver`](https://docs.rs/bevy/0.12.0/bevy/asset/saver/trait.AssetSaver.html)。

这意味着如果你已经有一个加载图像的 [`ImageLoader`](https://docs.rs/bevy/0.12.0/bevy/render/texture/struct.ImageLoader.html)，你所需要做的就是编写一个 `ImageSaver`，将图像以某种优化格式写出。这既节省了开发工作，又使得同时支持处理过和未处理的场景变得容易。

### 为随处运行而生 [#](https://bevy.org/news/bevy-0-12/#built-to-run-anywhere)

与游戏开发领域中许多其他资源处理器不同，Bevy Asset V2 的 [`AssetProcessor`](https://docs.rs/bevy/0.12.0/bevy/asset/processor/struct.AssetProcessor.html)有意设计为可在任何平台上运行。它不使用受平台限制的数据库，也不需要运行网络服务器的能力/权限。如果应用程序逻辑需要在运行时进行处理，它可以与已发布的应用一起部署。

一个值得注意的例外：我们还需要做一些修改才能使其在 Web 上运行，但它在设计时就考虑到了 Web 支持。

### 递归资源依赖加载事件 [#](https://bevy.org/news/bevy-0-12/#recursive-asset-dependency-load-events)

[`AssetEvent`](https://docs.rs/bevy/0.12.0/bevy/asset/enum.AssetEvent.html) 枚举现在有一个 [`AssetEvent::LoadedWithDependencies`](https://docs.rs/bevy/0.12.0/bevy/asset/enum.AssetEvent.html) 变体。当某个 [`Asset`](https://docs.rs/bevy/0.12.0/bevy/asset/trait.Asset.html)、其依赖项以及所有后代/递归依赖项都已加载时，会发出此事件。

这使得在 [`Asset`](https://docs.rs/bevy/0.12.0/bevy/asset/trait.Asset.html)"完全加载"后再执行某些操作变得容易。

### 多资源来源 [#](https://bevy.org/news/bevy-0-12/#multiple-asset-sources)

现在可以注册多个 [`AssetSource`](https://docs.rs/bevy/0.12.0/bevy/asset/io/struct.AssetSource.html)（它取代了旧的单一"资源提供者"系统）。

从"默认" [`AssetSource`](https://docs.rs/bevy/0.12.0/bevy/asset/io/struct.AssetSource.html) 加载与以前版本的 Bevy 看起来完全一样：

```rust
sprite.texture = assets.load("path/to/sprite.png");
```

但在 **Bevy 0.12** 中，你現在可以注册命名的 [`AssetSource`](https://docs.rs/bevy/0.12.0/bevy/asset/io/struct.AssetSource.html) 条目。例如，你可以注册一个从 HTTP 服务器加载资源的 `remote` [`AssetSource`](https://docs.rs/bevy/0.12.0/bevy/asset/io/struct.AssetSource.html)：

```rust
sprite.texture = assets.load("remote://path/to/sprite.png");
```

热重载、元数据文件和资源处理等功能在所有来源中都受支持。

你可以像这样注册一个新的 [`AssetSource`](https://docs.rs/bevy/0.12.0/bevy/asset/io/struct.AssetSource.html)：

```rust
// 从"other"文件夹读取资源，而不是默认的"assets"文件夹
app.register_asset_source(
    // 这是新来源的"名称"，在资源路径中使用。
    // 例如："other://path/to/sprite.png"
    "other",
    // 这是一个可重复的来源构建器。你可以配置 readers、writers、
    // processed readers、processed writers、asset watchers 等。
    AssetSource::build()
        .with_reader(|| Box::new(FileAssetReader::new("other")))
    )
)
```

### 内嵌资源 [#](https://bevy.org/news/bevy-0-12/#embedded-assets)

推动 **多资源来源** 的功能之一，是改进我们的"内嵌于二进制文件中的"资源加载。旧的 `load_internal_asset!` 方法存在许多问题（请参见[此 PR](https://github.com/bevyengine/bevy/pull/9885)中的相关部分）。

旧的系统看起来像这样：

```rust
pub const MESH_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(3252377289100772450);

load_internal_asset!(app, MESH_SHADER_HANDLE, "mesh.wgsl", Shader::from_wgsl);
```

这需要大量的样板代码，并且不能与资源系统的其余部分干净地集成。[`AssetServer`](https://docs.rs/bevy/0.12.0/bevy/asset/struct.AssetServer.html)不了解这些资源，热重载需要一个特殊的第二 [`AssetServer`](https://docs.rs/bevy/0.12.0/bevy/asset/struct.AssetServer.html)，并且你不能使用 [`AssetLoader`](https://docs.rs/bevy/0.12.0/bevy/asset/trait.AssetLoader.html) 加载资源（它们必须在内存中构造）。不太理想！

为了验证**多资源来源**的实现，我们构建了一个新的 `embedded` [`AssetSource`](https://docs.rs/bevy/0.12.0/bevy/asset/io/struct.AssetSource.html)，它用自然融入资源系统的方式取代了旧的 `load_internal_asset!` 系统：

```rust
// 在 `crates/bevy_pbr/src/render/mesh.rs` 中调用
embedded_asset!(app, "mesh.wgsl");

// 后续在应用中
let shader: Handle<Shader> = asset_server.load("embedded://bevy_pbr/render/mesh.wgsl");
```

这比旧方法少了很多样板代码！

而且由于 `embedded` 来源与其他任何资源来源一样，它可以干净地支持热重载……这与旧系统不同。要热重载内嵌在二进制文件中的资源（例如：获取你内嵌在二进制文件中的着色器的实时更新），只需启用新的 `embedded_watcher` cargo 功能。

好多了！

### 可扩展 [#](https://bevy.org/news/bevy-0-12/#extendable)

**Bevy Asset V2** 中几乎所有内容都可以通过 trait 实现进行扩展：

- **[`Asset`](https://docs.rs/bevy/0.12.0/bevy/asset/trait.Asset.html)**：定义新的资源类型
- **[`AssetReader`](https://docs.rs/bevy/0.12.0/bevy/asset/io/trait.AssetReader.html)**：定义自定义 [`AssetSource`](https://docs.rs/bevy/0.12.0/bevy/asset/io/struct.AssetSource.html) 读取逻辑
- **[`AssetWriter`](https://docs.rs/bevy/0.12.0/bevy/asset/io/trait.AssetWriter.html)**：定义自定义 [`AssetSource`](https://docs.rs/bevy/0.12.0/bevy/asset/io/struct.AssetSource.html) 写入逻辑
- **[`AssetWatcher`](https://docs.rs/bevy/0.12.0/bevy/asset/io/trait.AssetWatcher.html)**：定义自定义 [`AssetSource`](https://docs.rs/bevy/0.12.0/bevy/asset/io/struct.AssetSource.html) 监视/热重载逻辑
- **[`AssetLoader`](https://docs.rs/bevy/0.12.0/bevy/asset/trait.AssetLoader.html)**：为给定的 [`Asset`](https://docs.rs/bevy/0.12.0/bevy/asset/trait.Asset.html) 类型定义自定义加载逻辑
- **[`AssetSaver`](https://docs.rs/bevy/0.12.0/bevy/asset/saver/trait.AssetSaver.html)**：为给定的 [`Asset`](https://docs.rs/bevy/0.12.0/bevy/asset/trait.Asset.html) 类型定义自定义保存逻辑
- **[`Process`](https://docs.rs/bevy/0.12.0/bevy/asset/processor/trait.Process.html)**：定义完全自定义的处理器逻辑（或使用更规范的 [`LoadAndSave`](https://docs.rs/bevy/0.12.0/bevy/asset/processor/struct.LoadAndSave.html) [`Process`](https://docs.rs/bevy/0.12.0/bevy/asset/processor/trait.Process.html) 实现）

### 异步资源 I/O [#](https://bevy.org/news/bevy-0-12/#async-asset-i-o)

新的 [`AssetReader`](https://docs.rs/bevy/0.12.0/bevy/asset/io/trait.AssetReader.html) 和 [`AssetWriter`](https://docs.rs/bevy/0.12.0/bevy/asset/io/trait.AssetWriter.html)API 是异步的！这意味着天然异步的后端（如网络 API）可以直接对 Future 调用 `await`。

文件系统实现（如 [`FileAssetReader`](https://docs.rs/bevy/0.12.0/bevy/asset/io/file/struct.FileAssetReader.html)）将文件 I/O 卸载到单独的线程，当文件操作完成时 Future 会解析。

### 改进的热重载工作流 [#](https://bevy.org/news/bevy-0-12/#improved-hot-reloading-workflow)

以前版本的 Bevy 需要手动在应用代码中启用资源热重载（除了启用 `filesystem_watcher` cargo 功能之外）：

```rust
// 在旧版本 Bevy 中启用热重载
app.add_plugins(DefaultPlugins.set(AssetPlugin::default().watch_for_changes()))
```

这并不理想，因为应用的已发布版本通常不希望启用文件系统监视。

在 **Bevy 0.12** 中，我们改进了这一工作流，使新的 `file_watcher` cargo 功能默认在你的应用中启用文件监视。在开发期间，只需启用该功能运行你的应用：

```sh
cargo run --features bevy/file_watcher
```

在发布时，只需省略该功能。无需更改代码！

```sh
cargo build --release
```

### 更好的资源句柄 [#](https://bevy.org/news/bevy-0-12/#better-asset-handles)

资源句柄现在在其核心使用单个 [`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html) 来管理资源的生命周期。这极大地简化了内部实现，并使我们能够直接从句柄获取更多资源信息。

值得注意的是，在 **Bevy 0.12** 中，我们利用这一点从 [`Handle`](https://docs.rs/bevy/0.12.0/bevy/asset/enum.Handle.html) 直接提供 [`AssetPath`](https://docs.rs/bevy/0.12.0/bevy/asset/struct.AssetPath.html) 访问：

```rust
// 之前版本的 Bevy
let path = asset_server.get_handle_path(&handle);

// Bevy 0.12
let path = handle.path();
```

句柄现在在内部使用更小/更便宜的 [`AssetIndex`](https://docs.rs/bevy/0.12.0/bevy/asset/struct.AssetIndex.html)，它使用代际索引在密集存储中查找资源。

### 真正的写时复制资源路径 [#](https://bevy.org/news/bevy-0-12/#true-copy-on-write-asset-paths)

[`AssetServer`](https://docs.rs/bevy/0.12.0/bevy/asset/struct.AssetServer.html)和 [`AssetProcessor`](https://docs.rs/bevy/0.12.0/bevy/asset/processor/struct.AssetProcessor.html) 会进行大量的 [`AssetPath`](https://docs.rs/bevy/0.12.0/bevy/asset/struct.AssetPath.html) 克隆（跨多个线程）。在以前版本的 Bevy 中，[`AssetPath`](https://docs.rs/bevy/0.12.0/bevy/asset/struct.AssetPath.html)由 Rust 的 [`Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html) 类型支持。然而在 Rust 中，克隆一个"拥有的" [`Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html) 会导致内部值的克隆。这_不是_我们想要的 [`AssetPath`](https://docs.rs/bevy/0.12.0/bevy/asset/struct.AssetPath.html) 的"写时复制"行为。我们跨线程使用 [`AssetPath`](https://docs.rs/bevy/0.12.0/bevy/asset/struct.AssetPath.html)，因此我们_需要_从一个"拥有的"值开始。

为了防止所有这些字符串的克隆和重新分配，我们构建了自己的 [`CowArc`](https://docs.rs/bevy/0.12.0/bevy/utils/enum.CowArc.html) 类型，[`AssetPath`](https://docs.rs/bevy/0.12.0/bevy/asset/struct.AssetPath.html)在内部使用它。它有两个技巧：

1. "拥有的"变体是一个 `Arc<str>`，我们可以廉价地克隆而无需重新分配字符串。
2. 几乎所有在代码中定义的 [`AssetPath`](https://docs.rs/bevy/0.12.0/bevy/asset/struct.AssetPath.html) 值都来自 `&'static str`。我们创建了一个特殊的 [`CowArc::Static`](https://docs.rs/bevy/0.12.0/bevy/utils/enum.CowArc.html#variant.Static) 变体，保留了这种静态性，这意味着即使将借用转换为"拥有的 [`AssetPath`](https://docs.rs/bevy/0.12.0/bevy/asset/struct.AssetPath.html)"，我们也_零_分配。
## 在 Android 上暂停与恢复 [#](https://bevy.org/news/bevy-0-12/#suspend-and-resume-on-android)

作者：@mockersf

在 Android 上，应用程序在暂停时不再崩溃。相反，它们会被暂停，并且在应用程序恢复之前不会运行任何系统。

这解决了 Android 应用的最后一个"重大"障碍！Bevy 现在支持 Android！

在其他线程中运行的后台任务，如播放音频，不会被停止。当应用程序被暂停时，会发送一个 [`Lifetime`](https://docs.rs/bevy/0.12.0/bevy/window/enum.Lifetime.html) 事件 `ApplicationLifetime::Suspended`，对应于 [`onStop()`](https://developer.android.com/reference/android/app/Activity#onStop\(\)) 回调。你应该注意暂停不应在后台运行的任务，并在接收到 `ApplicationLifetime::Resumed` 事件（对应于 [`onRestart()`](https://developer.android.com/reference/android/app/Activity#onRestart\(\)) 回调）时恢复它们。

```rust
fn handle_lifetime_events(
    mut lifetime_events: EventReader<ApplicationLifetime>,
    music_controller: Query<&AudioSink>,
) {
    for event in lifetime_events.read() {
        match event {
            // 收到 `Suspended` 事件后，应用程序有 1 帧时间被暂停
            // 由于音频在独立线程中运行，需要将其停止
            ApplicationLifetime::Suspended => music_controller.single().pause(),
            // 收到 `Resumed` 后，音频可以继续播放
            ApplicationLifetime::Resumed => music_controller.single().play(),
            // `Started` 是目前的另一个事件，后续版本还会有更多
            _ => (),
        }
    }
}
```

## 材质扩展 [#](https://bevy.org/news/bevy-0-12/#material-extensions)

作者：@robtfm

Bevy 拥有强大的着色器导入系统，允许模块化（且细粒度）的着色器代码复用。在以前版本的 Bevy 中，这意味着从理论上讲，你可以导入 Bevy 的 PBR 着色器逻辑并在你自己的着色器中使用。然而在实践中这很有挑战性，因为你必须自己重新连接所有东西，这需要对基础材质有深入的了解。对于像 Bevy 的 PBR [`StandardMaterial`](https://docs.rs/bevy/0.12.0/bevy/pbr/struct.StandardMaterial.html) 这样复杂的材质，这充满了样板代码，导致代码重复，且容易出错。

在 **Bevy 0.12** 中，我们构建了一个**材质扩展**系统，允许定义基于现有材质构建的新材质：

![material extension](https://bevy.org/news/bevy-0-12/material_extension.png)

这是通过一个新的 [`ExtendedMaterial`](https://docs.rs/bevy/0.12.0/bevy/pbr/struct.ExtendedMaterial.html) 类型实现的：

```rust
app.add_plugin(
    MaterialPlugin::<ExtendedMaterial<StandardMaterial, QuantizedMaterial>>::default()
);

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
struct QuantizedMaterial {
    // 从高绑定编号开始，确保绑定不与基础材质冲突
    #[uniform(100)]
    quantize_steps: u32,
}

impl MaterialExtension for QuantizedMaterial {
    fn fragment_shader() -> ShaderRef {
        "quantized_material.wgsl".into()
    }
}

let material = ExtendedMaterial<StandardMaterial, QuantizedMaterial> {
    base: StandardMaterial::from(Color::rgb(0.1, 0.1, 0.8)),
    extension: QuantizedMaterial { quantize_steps: 2 },
};
```

我们还配合对 [`StandardMaterial`](https://docs.rs/bevy/0.12.0/bevy/pbr/struct.StandardMaterial.html) 着色器进行了重构，使得选择和组合你想要的各个部分变得更加容易：

```rust
// quantized_material.wgsl

struct QuantizedMaterial {
    quantize_steps: u32,
}

@group(1) @binding(100)
var<uniform> my_extended_material: QuantizedMaterial;

@fragment
fn fragment(
    input: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // 从 StandardMaterial 绑定生成 PbrInput 结构体
    var pbr_input = pbr_input_from_standard_material(input, is_front);

    // Alpha 丢弃
    pbr_input.material.base_color = alpha_discard(
        pbr_input.material,
        pbr_input.material.base_color
    );

    var out: FragmentOutput;

    // 应用光照
    out.color = apply_pbr_lighting(pbr_input);

    // 我们的"量化"效果
    out.color = vec4<f32>(vec4<u32>(out.color * f32(my_extended_material.quantize_steps))) / f32(my_extended_material.quantize_steps);

    // 应用着色器内后期处理。
    // 例如：雾、alpha 预乘等。对于非 HDR 相机：色调映射和去条带
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    return out;
}
```

这_极大地_简化了编写自定义 PBR 材质，使其对几乎所有人都变得可用！

## 绘制命令的自动批处理与实例化 [#](https://bevy.org/news/bevy-0-12/#automatic-batching-and-instancing-of-draw-commands)

作者：@superdump (Rob Swain)

**Bevy 0.12** 现在在可能的情况下会自动对绘制命令进行批处理/实例化。这减少了绘制调用的次数，带来了显著的性能提升！

这需要许多架构上的更改，包括我们如何存储和访问每个实体的网格数据（稍后会详细介绍）。

以下是旧的非批处理方法（0.11）与新的批处理方法（0.12）的一些基准测试：

### 2D Mesh Bevymark（每秒帧数，越高越好） [#](https://bevy.org/news/bevy-0-12/#2d-mesh-bevymark-frames-per-second-more-is-better)

这渲染了 160,000 个带有纹理四边形网格的实体（160 组，每组 1,000 个实体，每组共享一种材质）。这意味着我们可以对每组进行批处理，启用批处理后仅需 160 次实例化绘制调用。这使得帧率提升了 **200%（3 倍）**！

![0.12-2DMeshes](https://bevy.org/news/bevy-0-12/0.12-2DMeshes.svg)

在 M1 Max 上以 1080p 测试。

### 3D 网格"Many Cubes"（每秒帧数，越高越好） [#](https://bevy.org/news/bevy-0-12/#3d-mesh-many-cubes-frames-per-second-more-is-better)

这渲染了 160,000 个立方体，其中约 11,700 个在视图中可见。这些通过一个包含所有可见立方体的实例化绘制调用进行绘制，帧率提升高达 **100%（2 倍）**！

![0.12-3DMeshes](https://bevy.org/news/bevy-0-12/0.12-3DMeshes.svg)

在 M1 Max 上以 1080p 测试。

这些性能优势可以在所有平台上利用，包括 WebGL2！

### 什么可以被批处理？ [#](https://bevy.org/news/bevy-0-12/#what-can-be-batched)

批处理/实例化只能用于不需要"重新绑定"的 GPU 数据（绑定是使数据对着色器/管线可用，会产生运行时成本）。这意味着如果管线（着色器）、绑定组（着色器可访问的绑定数据）、顶点/索引缓冲区（网格）等不同，就无法进行批处理。

从高层次来看，目前具有相同材质和网格的实体可以被批处理。

我们正在研究如何使更多数据无需重新绑定即可访问，例如无绑定纹理、将网格合并到更大的缓冲区等。

### 选择退出 [#](https://bevy.org/news/bevy-0-12/#opting-out)

如果你希望某个实体退出自动批处理，你可以向其添加新的 [`NoAutomaticBatching`](https://docs.rs/bevy/0.12.0/bevy/render/batching/struct.NoAutomaticBatching.html) 组件。

这通常适用于你正在执行自定义、非标准的渲染器功能，而这些功能与批处理的假设不兼容的情况。例如，它假设视图绑定在绘制过程中是恒定的，并且使用 Bevy 内置的实体批处理逻辑。

## 通往 GPU 驱动渲染之路 [#](https://bevy.org/news/bevy-0-12/#the-road-to-gpu-driven-rendering)

作者：@superdump (Rob Swain), @james-j-obrien, @JMS55, @inodentry, @robtfm, @nicopap, @teoxoy, @IceSentry, @Elabajaba

Bevy 对 2D 和 3D 网格的渲染性能还有很大的提升空间。CPU 和 GPU 端都存在瓶颈，可以通过优化来减轻这些瓶颈，从而显著提高帧率。与 Bevy 一贯的做法一样，我们希望充分利用你使用的平台，从 WebGL2 和移动设备的限制到最高端的原生独立显卡都要考虑。坚实的基础可以支持所有这些。

在 **Bevy 0.12** 中，我们开始重新设计渲染数据结构、数据流和绘制模式，以解锁新的优化。这使我们能够在 **Bevy 0.12** 中实现**自动批处理和实例化**，同时也为未来其他重要的性能提升（如 GPU 驱动渲染）铺平道路。我们还没有完全准备好 GPU 驱动渲染，但我们在 **Bevy 0.12** 中已经开始走上这条路了！

### 什么是 CPU 和 GPU 驱动渲染？ [#](https://bevy.org/news/bevy-0-12/#what-are-cpu-and-gpu-driven-rendering)

CPU 驱动渲染是指在 CPU 上创建绘制命令。在 Bevy 中，这意味着"在 Rust 代码中"，更具体地说，是在渲染图节点中。这就是 Bevy 目前启动绘制的方式。

在 GPU 驱动渲染中，绘制命令由[计算着色器](https://www.khronos.org/opengl/wiki/Compute_Shader)在 GPU 上编码。这利用了 GPU 并行性，并解锁了在 CPU 上不可行的更高级的剔除优化，以及许多其他带来巨大性能优势的方法。

### 需要改变什么？ [#](https://bevy.org/news/bevy-0-12/#what-needs-to-change)

历史上，Bevy 的通用 GPU 数据模式是将每段数据按实体绑定，并按实体发出绘制调用。在某些情况下，我们确实以"数组风格"将数据存储在统一缓冲区中，并使用动态偏移量访问，但这仍然需要在每个偏移量处进行重新绑定。

所有这些重新绑定都有性能影响，无论是在 CPU 还是 GPU 上。在 CPU 上，这意味着编码绘制命令需要处理更多步骤，花费比必要更多的时间。在 GPU（以及图形 API）上，这意味着更多的重新绑定和单独的绘制调用。

避免重新绑定既是 CPU 驱动渲染的一项重大性能优势，也是实现 GPU 驱动渲染的必要条件。

为了避免重新绑定，我们追求的一般数据模式是：

- 对于每种数据类型（网格、材质、变换、纹理），创建一个包含该数据类型所有项目的单一数组（或少量数组）
- 少量绑定这些数组（理想情况下一次），避免按实体/按绘制的重新绑定

在 **Bevy 0.12** 中，我们已经认真地开始了这个过程！我们进行了多项架构更改，已经取得了成果。得益于这些更改，我们现在可以对具有完全相同网格和材质的实体[自动进行批处理和实例化绘制](https://bevy.org/news/bevy-0-12/#automatic-batching-and-instancing-of-draw-commands)。随着我们在这条道路上继续前进，我们可以在更广泛的场景中进行批处理/实例化，逐步减少 CPU 工作量，直到最终实现"完全 GPU 驱动"。

### 重新排序渲染集 [#](https://bevy.org/news/bevy-0-12/#reorder-render-sets)

作者：@superdump (Rob Swain), @james-j-obrien, @inodentry

对于某些实例化绘制方法，绘制的顺序需要预先知道，以便数据可以按顺序排列和查找。例如，当每个实例的数据存储在实例率顶点缓冲区中时。

**Bevy 0.12** 之前的渲染集顺序在这方面造成了一些问题，因为在知道绘制顺序之前数据就必须准备好（写入 GPU）。当我们的计划是在 GPU 上拥有一个有序的实体数据列表时，这并不理想！之前的集合顺序是：

![RenderSets-0.11](https://bevy.org/news/bevy-0-12/RenderSets-0.11.svg)

这在许多当前（和计划中的）渲染器功能中造成了摩擦（和次优的实例化）。最值得注意的是，在以前版本的 Bevy 中，这给精灵批处理带来了这些问题。

0.12 中新的渲染集顺序是：

![RenderSets-0.12](https://bevy.org/news/bevy-0-12/RenderSets-0.12.svg)

引入了 `PrepareAssets`，因为我们只希望在实体的资源准备好后才将其加入绘制队列。每帧数据准备仍在 `Prepare` 集中进行，具体在其子集 `PrepareResources` 中。现在它位于 `Queue` 和 `Sort` 之后，因此绘制的顺序是已知的。这对批处理也更有意义，因为在批处理时就可以知道渲染阶段中其他类型的实体是否需要被绘制。绑定组现在有了一个清晰的子集来指定它们应该在何处创建……`PrepareBindGroups`。

### BatchedUniformBuffer 和 GpuArrayBuffer [#](https://bevy.org/news/bevy-0-12/#batcheduniformbuffer-and-gpuarraybuffer)

好的，那么我们需要将相同类型的多段数据以尽可能少绑定、从中绘制多个实例的方式放入缓冲区。我们该怎么做呢？

在以前版本的 Bevy 中，每个实例的 `MeshUniform` 数据存储在一个统一缓冲区中，每个实例的数据按动态偏移量对齐。在绘制每个网格实体时，我们更新动态偏移量，这几乎和重新绑定一样昂贵。它看起来像这样：

![DynamicUniformBuffer](https://bevy.org/news/bevy-0-12/DynamicUniformBuffer.svg)

红色箭头是更新动态偏移量的"重新绑定"，蓝色框是实例数据，橙色框是用于动态偏移量对齐的填充，这是 GPU 和图形 API 的要求。

实例率顶点缓冲区是一种方式，但它们对特定顺序有严格限制。它们适合（或可能适合）用于网格实体变换等每实例数据，但不能用于材质数据。其他主要选项包括统一缓冲区、存储缓冲区和数据纹理。

WebGL2 不支持存储缓冲区，只支持统一缓冲区。在 WebGL2 上，统一缓冲区每次绑定的最小保证大小为 16kB。在支持存储缓冲区的地方，最小保证大小为 128MB。

数据纹理对于结构化数据来说使用起来要麻烦得多。而在不支持线性数据布局的平台上，它们的性能会更差。

鉴于这些限制，我们希望在支持存储缓冲区的平台上使用存储缓冲区，在不支持的平台上（例如：WebGL2）使用统一缓冲区。

#### BatchedUniformBuffer [#](https://bevy.org/news/bevy-0-12/#batcheduniformbuffer)

作者：@superdump (Rob Swain), @JMS55, @teoxoy, @robtfm, @konsolas

对于统一缓冲区，我们必须假设在 WebGL2 上我们可能一次只能访问 16kB 的数据。以 `MeshUniform` 为例，每个实例需要 144 字节，这意味着每次 16kB 绑定可以批处理 113 个实例。如果我们想绘制超过 113 个实体，我们需要一种方法来管理统一缓冲区数据，使其能够按实例批次以动态偏移量进行绑定。这就是 `BatchedUniformBuffer` 设计要解决的问题。

`BatchedUniformBuffer` 看起来像这样：

![BatchedUniformBuffer](https://bevy.org/news/bevy-0-12/BatchedUniformBuffer.svg)

红色箭头是更新动态偏移量的"重新绑定"，蓝色框是实例数据，橙色框是用于动态偏移量对齐的填充。

注意实例数据可以打包得更紧凑，在更少的空间内容纳相同数量的有效数据。此外，我们只需要为每个批次更新绑定的动态偏移量。

#### GpuArrayBuffer [#](https://bevy.org/news/bevy-0-12/#gpuarraybuffer)

作者：@superdump (Rob Swain), @JMS55, @IceSentry, @mockersf

鉴于我们需要为给定的数据类型同时支持统一缓冲区和存储缓冲区，这增加了实现新低级渲染器功能（无论是在 Rust 代码中还是着色器中）所需的复杂度。面对这种复杂性，一些开发者可能会选择只使用存储缓冲区（实际上放弃对 WebGL2 的支持）。

为了尽可能方便地支持这两种存储类型，我们开发了 [`GpuArrayBuffer`](https://docs.rs/bevy/0.12.0/bevy/render/render_resource/enum.GpuArrayBuffer.html)。这是一个通用的 `T` 值集合，对 `BatchedUniformBuffer` 和 [`StorageBuffer`](https://docs.rs/bevy/0.12.0/bevy/render/render_resource/struct.StorageBuffer.html) 进行抽象。它将为当前平台/GPU 使用正确的存储类型。

[`StorageBuffer`](https://docs.rs/bevy/0.12.0/bevy/render/render_resource/struct.StorageBuffer.html) 中的数据看起来像这样：

![StorageBuffer](https://bevy.org/news/bevy-0-12/StorageBuffer.svg)

红色箭头是"重新绑定"，蓝色框是实例数据。

所有实例数据可以直接一个接一个地放置，我们只需绑定一次。不需要任何动态偏移绑定，因此也不需要任何对齐填充。

[查看这个带注释的代码示例](https://gist.github.com/cart/3a9f190bd5e789a7d42317c28843ffca)，它演示了使用 [`GpuArrayBuffer`](https://docs.rs/bevy/0.12.0/bevy/render/render_resource/enum.GpuArrayBuffer.html) 来同时支持统一缓冲区和存储缓冲区绑定。

### 使用 GpuArrayBuffer 的 2D / 3D 网格实体 [#](https://bevy.org/news/bevy-0-12/#2d-3d-mesh-entities-using-gpuarraybuffer)

作者：@superdump (Rob Swain), @robtfm, @Elabajaba

2D 和 3D 网格实体渲染已迁移为使用 [`GpuArrayBuffer`](https://docs.rs/bevy/0.12.0/bevy/render/render_resource/enum.GpuArrayBuffer.html) 来处理网格统一数据。

仅避免网格统一数据缓冲区的重新绑定就带来了约 6% 的帧率提升！

## EntityHashMap 渲染器优化 [#](https://bevy.org/news/bevy-0-12/#entityhashmap-renderer-optimization)

作者：@superdump (Rob Swain), @robtfm, @pcwalton, @jancespivo, @SkiFire13, @nicopap

自 **Bevy 0.6** 以来，Bevy 的渲染器将数据从"主世界"提取到一个单独的"渲染世界"中。这实现了[流水线渲染](https://bevy.org/news/bevy-0-6/#pipelined-rendering-extract-prepare-queue-render)，即渲染应用在第 N 帧进行渲染，而主应用模拟第 N+1 帧。

设计的一部分涉及在帧之间清除渲染世界中的所有实体。这使得主世界和渲染世界之间能够保持一致的 Entity 映射，同时仍然能够在渲染世界中生成主世界中不存在的新实体。

不幸的是，这种 ECS 使用模式也带来了一些显著的性能问题。为了获得良好的"线性迭代读取性能"，我们希望使用"表存储"（Bevy 的默认 ECS 存储模型）。然而在渲染器中，实体每帧被清除和重新生成，组件跨渲染应用调度表中的许多系统和不同部分插入。这导致了许多"原型移动"，因为组件从各种渲染器上下文中被插入。当实体移动到新的原型时，其所有"表存储"组件都会被复制到新原型的表中。这在多次原型移动和/或大表移动时可能非常昂贵。

这不幸地留下了很多性能潜力未充分发挥。在很长一段时间内，讨论了很多关于如何改进的想法。

### 前进之路 [#](https://bevy.org/news/bevy-0-12/#the-path-forward)

两条主要的前进方向是：

1. 跨帧持久化渲染世界实体及其组件数据
2. 停止在渲染世界中使用实体表存储来存储组件数据

我们决定在 **Bevy 0.12** 中探索方案（2），因为持久化实体需要解决其他没有简单和满意答案的问题（例如：如何在不泄露数据的情况下保持世界完美同步）。我们最终可能会找到这些答案，但就目前而言，我们选择了阻力最小的路径！

我们最终使用了带优化哈希函数的 `HashMap<Entity, T>`，该哈希函数由 @SkiFire13 设计，灵感来自 [`rustc-hash`](https://github.com/rust-lang/rustc-hash)。这被公开为 [`EntityHashMap`](https://docs.rs/bevy/0.12.0/bevy/utils/type.EntityHashMap.html)，并且是在渲染世界中存储组件数据的新方式。

这[带来了显著的性能提升](https://github.com/bevyengine/bevy/pull/9903)。

### 使用方法 [#](https://bevy.org/news/bevy-0-12/#usage)

最简单的使用方法是使用新的 [`ExtractInstancesPlugin`](https://docs.rs/bevy/0.12.0/bevy/render/extract_instances/struct.ExtractInstancesPlugin.html)。这将提取匹配查询的所有实体，或者只提取那些可见的实体，同时将多个组件提取到一个目标类型中。

将需要一起访问的组件数据分组到一个目标类型中是一个好主意，以避免进行多次查找。

从可见实体中提取两个组件：

```rust
struct MyType {
    a: ComponentA,
    b: ComponentB,
}

impl ExtractInstance for MyType {
    type Query = (Read<ComponentA>, Read<ComponentB>);
    type Filter = ();

    fn extract((a, b): QueryItem<'_, Self::Query>) -> Option<Self> {
        Some(MyType {
          a: a.clone(),
          b: b.clone(),
        })
    }
}

app.add_plugins(ExtractInstancesPlugin::<MyType>::extract_visible());
```

## 精灵实例化 [#](https://bevy.org/news/bevy-0-12/#sprite-instancing)

作者：@superdump (Rob Swain)

在以前版本的 Bevy 中，精灵通过生成一个包含每个精灵 4 个顶点（包含位置、UV 和可能的颜色数据）的顶点缓冲区进行渲染。这被证明是非常有效的。然而，由于精灵使用不同的颜色而不得不将批次拆分为多次绘制，这是不理想的。

精灵渲染现在使用实例率顶点缓冲区来存储每个实例的数据。实例率顶点缓冲区在实例索引变化时步进，而不是在顶点索引变化时。新的缓冲区包含一个仿射变换矩阵，能够在一个变换中实现平移、缩放和旋转。它包含每个实例的颜色以及 UV 偏移和缩放。

这保留了以前方法的所有功能，增加了任何精灵都可以有颜色叠加且仍然在同一批次中绘制的额外灵活性，并且每个精灵仅使用 80 字节，而之前是 144 字节。

这导致了相比以前方法高达 **40%** 的性能提升！

## Rust 风格着色器导入 [#](https://bevy.org/news/bevy-0-12/#rusty-shader-imports)

作者：@robtfm

Bevy 着色器现在使用 Rust 风格的着色器导入：

```rust
// old（旧的）
#import bevy_pbr::forward_io VertexOutput

// new（新的）
#import bevy_pbr::forward_io::VertexOutput
```

像 Rust 导入一样，你可以使用花括号导入多个项目。多层嵌套现在也得到了支持！

```rust
// old
#import bevy_pbr::pbr_functions alpha_discard, apply_pbr_lighting 
#import bevy_pbr                mesh_bindings

// new
#import bevy_pbr::{
    pbr_functions::{alpha_discard, apply_pbr_lighting}, 
    mesh_bindings,
}
```

像 Rust 模块一样，你现在可以导入部分路径：

```rust
#import part::of::path

// 后续在着色器中
path::remainder::function();
```

你现在也可以使用完全限定路径而无需导入：

```rust
bevy_pbr::pbr_functions::pbr()
```

Rust 风格的导入消除了旧系统中的许多"API 怪异"陷阱，并扩展了导入系统的能力。通过复用 Rust 的语法和语义，我们消除了 Bevy 用户学习新系统的需要。

## glTF 自发光强度 [#](https://bevy.org/news/bevy-0-12/#gltf-emissive-strength)

作者：@JMS55

Bevy 现在在加载 glTF 资源时会读取并使用 `KHR_materials_emissive_strength` glTF 材质扩展。这增加了从 Blender 等程序导入 glTF 时对自发光材质的支持。这些立方体中的每一个的自发光强度都在递增：

![gltf emissive](https://bevy.org/news/bevy-0-12/gltf_emissive.png)

## 在 glTF 文件中导入第二 UV 贴图 [#](https://bevy.org/news/bevy-0-12/#import-second-uv-map-in-gltf-files)

作者：@pcwalton

**Bevy 0.12** 现在会导入第二 UV 贴图（`TEXCOORD1` 或 `UV1`）（如果在 glTF 文件中定义）并将其暴露给着色器。按照惯例，这通常用于光照贴图 UV。这是一个经常被请求的功能，它解锁了光照贴图场景（无论是在自定义用户着色器中还是在未来的 Bevy 版本中）。

## 线框改进 [#](https://bevy.org/news/bevy-0-12/#wireframe-improvements)

作者：@IceSentry

线框现在使用 Bevy 的 [`Material`](https://docs.rs/bevy/0.12.0/bevy/pbr/trait.Material.html) 抽象。这意味着它将自动使用新的批处理和实例化功能，同时更易于维护。这一更改也使添加彩色线框支持变得更加容易。你可以使用 [`WireframeColor`](https://docs.rs/bevy/0.12.0/bevy/pbr/wireframe/struct.WireframeColor.html) 组件全局或按网格配置颜色。现在也可以通过使用 [`NoWireframe`](https://docs.rs/bevy/0.12.0/bevy/pbr/wireframe/struct.NoWireframe.html) 组件来禁用线框渲染。

![wireframe](https://bevy.org/news/bevy-0-12/wireframe.png)

## 外部渲染器上下文 [#](https://bevy.org/news/bevy-0-12/#external-renderer-context)

作者：@awtterpip

历史上，Bevy 的 [`RenderPlugin`](https://docs.rs/bevy/0.12.0/bevy/render/struct.RenderPlugin.html) 完全负责初始化 [`wgpu`](https://github.com/gfx-rs/wgpu) 渲染上下文。然而，一些第三方 Bevy 插件（如正在开发中的 [`bevy_openxr`](https://github.com/awtterpip/bevy_openxr) 插件）需要对渲染器初始化有更多控制。

因此，在 **Bevy 0.12** 中，我们使得在启动时传入 [`wgpu`](https://github.com/gfx-rs/wgpu) 渲染上下文成为可能。这意味着第三方 [`bevy_openxr`](https://github.com/awtterpip/bevy_openxr/) 插件可以成为"正常的"Bevy 插件，而无需 fork Bevy！

以下是由 [`bevy_openxr`](https://github.com/awtterpip/bevy_openxr/) 提供的 Bevy VR 快速视频！

## 绑定组人体工程学 [#](https://bevy.org/news/bevy-0-12/#bind-group-ergonomics)

作者：@robtfm, @JMS55

在为低级渲染器功能定义"绑定组"时，我们使用以下 API：

```rust
render_device.create_bind_group(
    "my_bind_group",
    &my_layout,
    &[
        BindGroupEntry {
            binding: 0,
            resource: BindingResource::Sampler(&my_sampler),
        },
        BindGroupEntry {
            binding: 1,
            resource: my_uniform,
        },
    ],
);
```

这工作得相当好，但对于大量绑定组来说，`BindGroupEntry` 的样板代码使得读写所有内容（并保持索引更新）变得比必要的困难。

**Bevy 0.12** 增加了额外的选项：

```rust
// 使用元组项的索引自动设置索引
render_device.create_bind_group(
    "my_bind_group",
    &my_layout,
    &BindGroupEntries::sequential((&my_sampler, my_uniform)),
);
```

```rust
// 手动设置索引，但没有 BindGroupEntry 的样板代码！
render_device.create_bind_group(
    "my_bind_group",
    &my_layout,
    &BindGroupEntries::with_indexes((
        (2, &my_sampler),
        (3, my_uniform),
    )),
);
```
## 一次性系统 [#](https://bevy.org/news/bevy-0-12/#one-shot-systems)

作者：@alice-i-cecile @pascualex, @Trashtalk217, @Zeenobit

通常情况下，系统作为调度表（Schedule）的一部分，每帧运行一次。但这并不总是最合适的。也许你在响应一个非常罕见的事件，比如复杂的回合制游戏中的某个事件，或者只是不想为了每个按钮而在调度表中杂乱地添加一个新系统。一次性系统颠覆了这种逻辑，它使你可以使用强大且熟悉的系统语法按需运行任意逻辑。

```rust
#[derive(Resource, Default, Debug)]
struct Counter(u8);

fn increment(mut counter: ResMut<Counter>) {
    counter.0 += 1;
    println!("{}", counter.0);
}

fn foo(world: &mut World) {
    world.init_resource::<Counter>();
    let id = world.register_system(increment);
    let _ = world.run_system(id); // 打印 1
    let _ = world.run_system(id); // 打印 2
}
```

使用一次性系统有三个简单步骤：注册一个系统，存储其 `SystemId`，然后使用排他性世界访问或命令来运行相应的系统。

仅凭这一点就已经能实现很多功能了，然而当 `SystemId` 被包装到组件中时，它们才真正开始展现其威力。

```rust
use bevy::ecs::system::SystemId;

#[derive(Component)]
struct Callback(SystemId);

// 调用所有回调！
fn call_all(query: Query<&Callback>, mut commands: Commands) {
    for callback in query.iter() {
        commands.run_system(callback.0);
    }
}
```

然后可以将一次性系统附加到 UI 元素上，例如按钮、RPG 中的动作或任何其他实体。你甚至可能受到启发，用一次性系统和 [`aery`](https://docs.rs/aery/latest/aery/) 来实现 Bevy 的调度图（顺便说一下，告诉我们进展如何）。

一次性系统非常灵活。它们可以嵌套，因此你可以从一次性系统内部调用 `run_system`。可以同时注册同一个系统的多个实例，每个实例都有自己的 `Local` 变量和缓存的系统状态。它还与资源驱动的工作流配合得很好：在序列化回调中记录从字符串到标识符的映射，比尝试使用 Rust 函数来做要好得多！

尽管如此，一次性系统并非没有局限性。目前，排他性系统和为系统管道设计的系统（带有 `In` 参数或返回类型）完全不能使用。你也不能从自身调用一次性系统，递归是不可能的。最后，一次性系统总是按顺序执行，而不是并行执行。虽然这降低了复杂性和开销，但对于某些工作负载来说，这可能比使用带有并行执行器的调度表要慢得多。

然而，当你只是在进行原型设计或编写单元测试时，这会非常麻烦：需要两个完整的函数和一些奇怪的标识符？对于这些情况，你可以使用 `World::run_system_once` 方法。

```rust
use bevy::ecs::system::RunSystemOnce;

#[derive(Resource, Default, Debug)]
struct Counter(u8);

fn increment(mut counter: ResMut<Counter>) {
    counter.0 += 1;
    println!("{}", counter.0);
}

let mut world = World::new();
world.init_resource::<Counter>();
world.run_system_once(increment); // 打印 1
world.run_system_once(increment); // 打印 2
```

这对于单元测试系统和查询来说非常好，而且开销更低，使用更简单。然而，有一个注意事项。有些系统具有状态，无论是 `Local` 参数、变更检测还是 `EventReader`。这种状态不会在两次 `run_system_once` 调用之间保存，会产生奇怪的行为。`Local` 变量每次运行都会重置，而变更检测将_始终_检测数据为已添加/已更改。小心使用，你就不会出问题。

## system.map [#](https://bevy.org/news/bevy-0-12/#system-map)

作者：@JoJoJet

**Bevy 0.12** 添加了一个新的 [`system.map()`](https://docs.rs/bevy/0.12.0/bevy/ecs/system/trait.IntoSystem.html#method.map) 函数，它是 [`system.pipe()`](https://docs.rs/bevy/0.12.0/bevy/ecs/system/trait.IntoSystem.html#method.pipe) 的一种更便宜、更符合人体工程学的替代方案。

与 [`system.pipe()`](https://docs.rs/bevy/0.12.0/bevy/ecs/system/trait.IntoSystem.html#method.pipe) 不同，[`system.map()`](https://docs.rs/bevy/0.12.0/bevy/ecs/system/trait.IntoSystem.html#method.map) 只接受一个普通的闭包（而不是另一个系统），该闭包将系统的输出作为其参数：

```rust
app.add_systems(Update, my_system.map(error));

fn my_system(res: Res<T>) -> Result<(), Err> {
    // 在这里做一些可能失败的事情
}

// 一个记录错误的适配器
pub fn error<E: Debug>(result: Result<(), E>) {
    if let Err(warn) = result {
        error!("{:?}", warn);
    }
}
```

Bevy 提供了内置的 `error`、`warn`、`debug` 和 `info` 适配器，可以与 [`system.map()`](https://docs.rs/bevy/0.12.0/bevy/ecs/system/trait.IntoSystem.html#method.map) 一起使用，在这些级别记录错误。

## 简化并行迭代方法 [#](https://bevy.org/news/bevy-0-12/#simplify-parallel-iteration-method)

作者：@JoJoJet

**Bevy 0.12** 使并行查询迭代器 [`for_each()`](https://docs.rs/bevy/0.12.0/bevy/ecs/query/struct.QueryParIter.html#method.for_each) 与可变和不可变查询兼容，减少了 API 表面积，并消除了写两次 `mut` 的需要：

```rust
// old（旧的）
query.par_iter_mut().for_each_mut(|x| ...);

// new（新的）
query.par_iter_mut().for_each(|x| ...);
```

## 通过 EntityMut 实现不重叠的可变世界访问 [#](https://bevy.org/news/bevy-0-12/#disjoint-mutable-world-access-via-entitymut)

作者：@JoJoJet

**Bevy 0.12** 支持同时安全地访问多个 [`EntityMut`](https://docs.rs/bevy/0.12.0/bevy/ecs/world/struct.EntityMut.html) 值，这意味着你可以同时修改多个实体（并访问它们的_所有组件_）。

```rust
let [entity1, entity2] = world.many_entities_mut([id1, id2]);
*entity1.get_mut::<Transform>().unwrap() = *entity2.get::<Transform>().unwrap();
```

这也适用于查询：

```rust
// 这在以前版本的 Bevy 中是无法表达的
// 现在它是完全有效的！
fn system(q1: Query<&mut A>, q2: Query<EntityMut, Without<A>>) {
}
```

你现在可以可变地迭代所有实体并访问其中的任意组件：

```rust
for mut entity in world.iter_entities_mut() {
    let mut transform = entity.get_mut::<Transform>().unwrap();
    transform.translation.x += 2.0;
}
```

这需要将 [`EntityMut`](https://docs.rs/bevy/0.12.0/bevy/ecs/world/struct.EntityMut.html) 的访问范围缩小到_仅_其访问的实体（以前它具有允许直接 [`World`](https://docs.rs/bevy/0.12.0/bevy/ecs/world/struct.World.html) 访问的逃生口）。对于相当于旧"全局访问"方法的情况，请使用 [`EntityWorldMut`](https://docs.rs/bevy/0.12.0/bevy/ecs/world/struct.EntityWorldMut.html)。

## 统一的 configure_sets API [#](https://bevy.org/news/bevy-0-12/#unified-configure-sets-api)

作者：@geieredgar

在 **Bevy 0.11** 中引入的 Bevy [调度表优先 API](https://bevy.org/news/bevy-0-11/#schedule-first-ecs-apis) 将大多数 ECS 调度器 API 表面积统一在单一的 `add_systems` API 之下。然而，我们没有为 `configure_sets` 做统一的 API，这意味着存在两个不同的 API：

```rust
app.configure_set(Update, A.after(B));
app.configure_sets(Update, (A.after(B), B.after(C));
```

在 **Bevy 0.12** 中，我们已将它们统一到一个 API 下，以与我们其他地方使用的模式保持一致，并减少不必要的 API 表面积：

```rust
app.configure_sets(Update, A.after(B));
app.configure_sets(Update, (A.after(B), B.after(C));
```

## UI 材质 [#](https://bevy.org/news/bevy-0-12/#ui-materials)

作者：@MarkusTheOrt

Bevy 的材质系统已通过新的 [`UiMaterial`](https://docs.rs/bevy/0.12.0/bevy/ui/trait.UiMaterial.html) 引入到 Bevy UI 中：

![ui material](https://bevy.org/news/bevy-0-12/ui_material.png)

这个"圆形"UI 节点是用自定义着色器绘制的：

```rust
#import bevy_ui::ui_vertex_output::UiVertexOutput

struct CircleMaterial {
    @location(0) color: vec4<f32>
}

@group(1) @binding(0)
var<uniform> input: CircleMaterial;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - 1.0;
    let alpha = 1.0 - pow(sqrt(dot(uv, uv)), 100.0);
    return vec4<f32>(input.color.rgb, alpha);
}
```

就像其他 Bevy 材质类型一样，在代码中设置也很简单！

```rust
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
struct CircleMaterial {
    #[uniform(0)]
    color: Vec4,
}

impl UiMaterial for CircleMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/circle_shader.wgsl".into()
    }
}

// 在你的 App 中注册材质插件
app.add_plugins(UiMaterialPlugin::<CircleMaterial>::default())

// 稍后在应用中，用你的材质生成 UI 节点！
commands.spawn(MaterialNodeBundle {
    style: Style {
        position_type: PositionType::Absolute,
        width: Val::Px(250.0),
        height: Val::Px(250.0),
        ..default()
    },
    material: materials.add(CircleMaterial {
        color: Color::rgb(0.0, 1.0, 0.58).into(),
    }),
    ..default()
});
```

## UI 节点轮廓 [#](https://bevy.org/news/bevy-0-12/#ui-node-outlines)

作者：@ickshonpe

Bevy 的 UI 节点现在通过新的 [`Outline`](https://docs.rs/bevy/0.12.0/bevy/ui/struct.Outline.html) 组件支持"节点边框外部的"轮廓。[`Outline`](https://docs.rs/bevy/0.12.0/bevy/ui/struct.Outline.html) 在布局中不占用任何空间。这与 [`Style::border`](https://docs.rs/bevy/0.12.0/bevy/ui/struct.Style.html) 不同，后者作为节点的"一部分"存在于布局中：

![ui outlines](https://bevy.org/news/bevy-0-12/ui_outlines.png)

```rust
commands.spawn((
    NodeBundle::default(),
    Outline {
        width: Val::Px(6.),
        offset: Val::Px(6.),
        color: Color::WHITE,
    },
))
```

## 统一的 `Time` [#](https://bevy.org/news/bevy-0-12/#unified-time)

作者：@nakedible @maniwani @alice-i-cecile

Bevy 0.12 为 [`FixedUpdate`](https://docs.rs/bevy/0.12.0/bevy/app/struct.FixedUpdate.html) 带来了两个重大的生活质量改进。

- [`Time`](https://docs.rs/bevy/0.12.0/bevy/time/struct.Time.html) 现在为在 [`FixedUpdate`](https://docs.rs/bevy/0.12.0/bevy/app/struct.FixedUpdate.html) 中运行的系统返回上下文正确的值。（因此，`FixedTime` 已被移除。）
- [`FixedUpdate`](https://docs.rs/bevy/0.12.0/bevy/app/struct.FixedUpdate.html) 不再会滚雪球般地变成"死亡螺旋"（即因为 [`FixedUpdate`](https://docs.rs/bevy/0.12.0/bevy/app/struct.FixedUpdate.html) 步骤入队速度快于运行速度而导致应用冻结）。

[`FixedUpdate`](https://docs.rs/bevy/0.12.0/bevy/app/struct.FixedUpdate.html) 调度表及其配套的 `FixedTime` 资源是在 Bevy 0.10 中引入的，很快就暴露出 `FixedTime` 的不足。举几个例子，它的方法与 [`Time`](https://docs.rs/bevy/0.12.0/bevy/time/struct.Time.html) 不同，甚至不像 [`Time`](https://docs.rs/bevy/0.12.0/bevy/time/struct.Time.html) 那样跟踪"已用总时间"。拥有两个不同的"时间"API 也意味着你必须编写专门支持"固定时间步长"或"可变时间步长"的系统，而不是两者都支持。消除这种分裂是可取的，因为它可能导致后续插件之间的不兼容（这在其他游戏引擎的插件中有时会发生）。

现在，你可以直接编写读取 [`Time`](https://docs.rs/bevy/0.12.0/bevy/time/struct.Time.html) 的系统，并将它们安排在任何上下文中。

```rust
// 如果调度在 `FixedUpdate` 中，此系统将看到恒定的 delta 时间；
// 如果调度在 `Update` 中，则看到可变的 delta 时间。
fn integrate_velocity(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}
```

大多数系统应继续使用 [`Time`](https://docs.rs/bevy/0.12.0/bevy/time/struct.Time.html)，但在幕后，以前 API 的方法已被重构为四个时钟：

- `Time<Real>`
- `Time<Virtual>`
- `Time<Fixed>`
- `Time<()>`

`Time<Real>` 测量真实的、未经编辑的帧和应用持续时间。用于诊断/性能分析。它也被用于派生其他时钟。`Time<Virtual>` 可以加速、减速和暂停，而 `Time<Fixed>` 以固定增量追赶 `Time<Virtual>`。最后，`Time<()>` 在进入或退出 `FixedUpdate` 时自动被 `Time<Fixed>` 或 `Time<Virtual>` 的当前值覆盖。当系统借用 `Time` 时，它实际上借用的是 `Time<()>`。

尝试新的[时间示例](https://github.com/bevyengine/bevy/blob/main/examples/time/time.rs)来更好地感受这些资源。

关于缠绕问题的修复是限制了 `Time<Virtual>`` 从单帧中可以前进的量。这进而限制了 [`FixedUpdate`](https://docs.rs/bevy/0.12.0/bevy/app/struct.FixedUpdate.html) 可以为下一帧排队的次数，因此帧延迟或计算机从长时间睡眠中唤醒不再会导致死亡螺旋。所以现在，应用不会冻结，但在 [`FixedUpdate`](https://docs.rs/bevy/0.12.0/bevy/app/struct.FixedUpdate.html) 中发生的事情会看起来变慢，因为它将以临时降低的速率运行。

## ImageLoader 设置 [#](https://bevy.org/news/bevy-0-12/#imageloader-settings)

作者：@cart, @Kanabenki

为了利用 **Bevy Asset V2** 中新的 [`AssetLoader`](https://docs.rs/bevy/0.12.0/bevy/asset/trait.AssetLoader.html) 设置，我们为 [`ImageLoader`](https://docs.rs/bevy/0.12.0/bevy/render/texture/struct.ImageLoader.html) 添加了 [`ImageLoaderSettings`](https://docs.rs/bevy/0.12.0/bevy/render/texture/struct.ImageLoaderSettings.html)。

这意味着你现在可以在按图像的基础上配置采样器、SRGB 和格式。以下是默认设置，它们在 **Bevy Asset V2** 元数据文件中的形式如下：

```rust
(
    format: FromExtension,
    is_srgb: true,
    sampler: Default,
)
```

当设置为 `Default` 时，图像将使用 [`ImagePlugin::default_sampler`](https://docs.rs/bevy/0.12.0/bevy/render/prelude/struct.ImagePlugin.html#structfield.default_sampler) 中配置的任何设置。

然而，你可以将这些值设置为你想要的任何值！

```rust
(
    format: Format(Basis),
    is_srgb: true,
    sampler: Descriptor((
        label: None,
        address_mode_u: ClampToEdge,
        address_mode_v: ClampToEdge,
        address_mode_w: ClampToEdge,
        mag_filter: Nearest,
        min_filter: Nearest,
        mipmap_filter: Nearest,
        lod_min_clamp: 0.0,
        lod_max_clamp: 32.0,
        compare: None,
        anisotropy_clamp: 1,
        border_color: None,
    )),
)
```

## GamepadButtonInput [#](https://bevy.org/news/bevy-0-12/#gamepadbuttoninput)

作者：@bravely-beep

Bevy 通常提供两种方式来处理给定类型的输入：

- 事件：按事件发生的顺序接收输入事件流
- [`Input`](https://docs.rs/bevy/0.12.0/bevy/input/struct.Input.html) 资源：读取输入的_当前_状态

一个值得注意的例外是 [`GamepadButton`](https://docs.rs/bevy/0.12.0/bevy/input/gamepad/struct.GamepadButton.html)，它只能通过 [`Input`](https://docs.rs/bevy/0.12.0/bevy/input/struct.Input.html) 资源使用。**Bevy 0.12** 添加了一个新的 [`GamepadButtonInput`](https://docs.rs/bevy/0.12.0/bevy/input/gamepad/struct.GamepadButtonInput.html) 事件，填补了这一空白。

## SceneInstanceReady 事件 [#](https://bevy.org/news/bevy-0-12/#sceneinstanceready-event)

作者：@Shatur

**Bevy 0.12** 添加了一个新的 [`SceneInstanceReady`](https://docs.rs/bevy/0.12.0/bevy/scene/struct.SceneInstanceReady.html) 事件，使得监听特定场景实例是否就绪变得容易。这里的"就绪"意味着"已经作为实体完全生成"。

```rust
#[derive(Resource)]
struct MyScene(Entity);

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let scene = SceneBundle {
        scene: assets.load("some.gltf#MyScene"),
        ..default()
    };
    let entity = commands.spawn(scene).id();
    commands.insert_resource(MyScene(entity));
}

fn system(mut events: EventReader<SceneInstanceReady>, my_scene: Res<MyScene>) {
    for event in events.read() {
        if event.parent == my_scene.0 {
            // 场景实例已"就绪"
        }
    }
}
```

## 拆分计算可见性 [#](https://bevy.org/news/bevy-0-12/#split-computed-visibility)

作者：@JoJoJet

`ComputedVisibility` 组件现已被拆分为 [`InheritedVisibility`](https://docs.rs/bevy/0.12.0/bevy/render/view/struct.InheritedVisibility.html)（在层次结构中可见）和 [`ViewVisibility`](https://docs.rs/bevy/0.12.0/bevy/render/view/struct.ViewVisibility.html)（从某个视角可见），使得可以在两组数据上分别使用 Bevy 内置的变更检测。

## ReflectBundle [#](https://bevy.org/news/bevy-0-12/#reflectbundle)

作者：@Shatur

Bevy 现在通过 [`ReflectBundle`](https://docs.rs/bevy/0.12.0/bevy/ecs/reflect/struct.ReflectBundle.html) 支持"Bundle 反射"：

```rust
#[derive(Bundle, Reflect)]
#[reflect(Bundle)]
struct SpriteBundle {
    image: Handle<Image>,
    // 其他组件在此处
}
```

这使得可以使用 Bevy Reflect 创建和操作 ECS Bundle，意味着你可以在运行时动态执行这些操作。这对于脚本和资源场景非常有用。

## Reflect Commands [#](https://bevy.org/news/bevy-0-12/#reflect-commands)

作者：@NoahShomette

现在可以通过 [`Commands`](https://docs.rs/bevy/0.12.0/bevy/ecs/system/struct.Commands.html) 上的新函数，在普通系统中插入和移除反射组件！

```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Component(u32);

fn reflect_commands(mut commands: Commands) {
    let boxed_reflect_component: Box<dyn Reflect> = Box::new(Component(916));

    let entity = commands
        .spawn_empty()
        .insert_reflect(boxed_reflect_component.clone_value()).id();

    commands.entity(entity).remove_reflect(boxed_reflect_component.type_name().to_owned());

}
```

上述命令默认使用 [`AppTypeRegistry`](https://docs.rs/bevy/0.12.0/bevy/ecs/reflect/struct.AppTypeRegistry.html)。如果你使用不同的 TypeRegistry，那么可以改用带 `_with_registry` 的命令。

```rust
 #[derive(Resource)]
 struct TypeRegistryResource {
     type_registry: TypeRegistry,
 }

 impl AsRef<TypeRegistry> for TypeRegistryResource {
     fn as_ref(&self) -> &TypeRegistry {
         &self.type_registry
     }
 }

 fn reflect_commands_with_registry(mut commands: Commands) {
    let boxed_reflect_component: Box<dyn Reflect> = Box::new(Component(916));

    let entity = commands
        .spawn_empty()
        .insert_reflect_with_registry::<TypeRegistryResource>(boxed_reflect_component.clone_value()).id();

    commands.entity(entity).remove_reflect_with_registry::<TypeRegistryResource>(boxed_reflect_component.type_name().to_owned());

}
```

查看 [`ReflectCommandExt`](https://docs.rs/bevy/0.12.0/bevy/ecs/reflect/trait.ReflectCommandExt.html) 获取更多示例和文档。
## 限制后台 FPS [#](https://bevy.org/news/bevy-0-12/#limit-background-fps)

作者：@maniwani

如果应用没有窗口处于焦点状态，Bevy 现在将限制其更新速率（默认 60Hz）。

以前，在桌面操作系统（特别是 macOS）上运行的许多 Bevy 应用，在窗口最小化或完全覆盖时，即使启用了 VSync，CPU 使用率也会出现峰值。原因是许多桌面窗口管理器会忽略不可见窗口的 VSync。由于 VSync 通常限制了应用的更新频率，当它被有效地禁用时，这个速度限制就消失了。

现在，在后台运行的应用将在更新之间休眠以限制其 FPS。

一个注意事项是，大多数操作系统不会报告窗口是否可见，只会报告它是否具有焦点。因此限速是基于焦点，而非可见性。选择 60Hz 作为默认值是为了在窗口未聚焦但仍可见的情况下维持高 FPS。

## `AnimationPlayer` API 改进 [#](https://bevy.org/news/bevy-0-12/#animationplayer-api-improvements)

作者：@devinleamy

`AnimationPlayer` 现在有了新的方法来控制播放，以及用于检查动画是否正在播放或已完成的工具，并获取其 `AnimationClip` 句柄。

`set_elapsed` 已被移除，改为 `seek_to`。`elapsed` 现在返回实际经过的时间，不受动画速度影响。`stop_repeating` 已被移除，改为 `set_repeat(RepeatAnimation::Never)`。

```rust
let mut player = q_animation_player.single_mut();
// 检查动画是否完成。
if player.is_finished() {
    // 设置播放模式。
    player.set_repeat(RepeatAnimation::Forever);
    player.set_repeat(RepeatAnimation::Never);
    player.set_repeat(RepeatAnimation::Count(4));
}
// 获取正在播放的 AnimationClip 的句柄。
let clip_handle = player.animation_clip();
// 将当前剪辑快进到 1 秒处。
player.seek_to(1.0);
```

## 忽略模糊组件和资源 [#](https://bevy.org/news/bevy-0-12/#ignore-ambiguous-components-and-resources)

作者：@hymm

模糊报告是 Bevy 调度器的一个可选功能。启用后，它会报告修改相同数据但彼此之间没有排序关系的系统之间的冲突。虽然某些报告的冲突可能会导致微妙的 Bug，但许多并不会。Bevy 有几个现有方法和两个新方法来忽略这些冲突。

现有的 API：`ambiguous_with`，忽略特定集合之间的冲突；以及 `ambiguous_with_all`，忽略应用它的集合的所有冲突。此外，现在还有 2 个新的 API，允许你忽略对某种数据类型的冲突：`allow_ambiguous_component` 和 `allow_ambiguous_resource`。它们忽略世界中对该特定类型、组件或资源的所有系统之间的冲突。

```rust
#[derive(Resource)]
struct R;

// 这些系统在 R 上是模糊的
fn system_1(_: ResMut<R>) {}
fn system_2(_: Res<R>) {}

let mut app = App::new();
app.configure_schedules(ScheduleBuildSettings {
  ambiguity_detection: LogLevel::Error,
  ..default()
});
app.insert_resource(R);

app.add_systems(Update, ( system_1, system_2 ));
app.allow_ambiguous_resource::<R>();

// 运行应用不会产生错误。
app.update();
```

Bevy 现在使用此功能来忽略 `Assets<T>` 资源之间的冲突。这些模糊性大多数是在修改不同的资源，因此无关紧要。

## 空间音频 API 人体工程学 [#](https://bevy.org/news/bevy-0-12/#spatial-audio-api-ergonomics)

作者：@rparrett, @hymm, @mockersf

一个简单的"立体声"（非 HRTF）空间音频实现在 Bevy 0.10 发布前的最后一刻被英勇地[组合起来](https://bevy.org/news/bevy-0-10/#spatial-audio)，但实现有些简陋，且不够用户友好。用户需要编写自己的系统来更新音频接收器的发射器和监听器位置。

现在用户只需将 `TransformBundle` 添加到他们的 `AudioBundle` 中，Bevy 就会处理其余的事情！

```rust
commands.spawn((
    TransformBundle::default(),
    AudioBundle {
        source: asset_server.load("sounds/bonk.ogg"),
        settings: PlaybackSettings::DESPAWN.with_spatial(true),
    },
));
```

## 音高音频源 [#](https://bevy.org/news/bevy-0-12/#pitch-audio-source)

作者：@basilefff

现在可以按音高播放音频，这对于调试音频问题、作为占位符使用或程序化生成音频非常有用。

可以从其频率和持续时间创建一个 `Pitch` 音频源，然后作为 `PitchBundle` 中的源使用。

```rust
fn play_pitch(
    mut pitch_assets: ResMut<Assets<Pitch>>,
    mut commands: Commands,
) {
    // 这是一个持续 1 秒的 A 音
    let pitch_handle = pitch_assets.add(Pitch::new(440.0, Duration::new(1, 0)));
    // 立即播放
    commands.spawn(PitchBundle {
        source: pitch_handle,
        ..default()
    });
}
```

音频使用[正弦波](https://en.wikipedia.org/wiki/Sine_wave#Audio_example)以给定频率生成。通过同时播放多个音高音频源，可以创建更复杂的声音，比如和弦或泛音。

## 为 `Color` 结构体添加 HSL 方法 [#](https://bevy.org/news/bevy-0-12/#added-hsl-methods-to-color-struct)

作者：@idedary

你现在可以使用 `h()`、`s()`、`l()` 以及它们的 `set_h()`、`set_s()`、`set_l()` 和 `with_h()`、`with_s()`、`with_l()` 变体来操作 `Color` 结构体的_色相_、_饱和度_和_明度_值，而无需克隆。以前你只能通过 RGBA 值来做到这一点。

```rust
// 返回 HSL 分量值
let color = Color::ORANGE;
let hue = color.h();
// ...

// 更改 HSL 分量值
let mut color = Color::PINK;
color.set_s(0.5);
// ...

// 修改现有颜色并返回它们
let color = Color::VIOLET.with_l(0.7);
// ...
```

## 减少 Tracing 开销 [#](https://bevy.org/news/bevy-0-12/#reduced-tracing-overhead)

作者：@hymm, @james7132

Bevy 使用 [tracing](https://crates.io/crates/tracing) 库来测量系统运行时间（以及其他用途）。这对于确定帧时间的瓶颈所在以及衡量性能改进很有用。这些跟踪信息可以使用 [tracy](https://github.com/wolfpld/tracy) 工具进行可视化。然而，使用 tracing 的 span 会产生显著的开销。每次 span 开销的很大一部分来自于分配 span 的字符串描述。通过缓存系统、命令和并行迭代的 span，我们显著减少了使用 tracing 时的 CPU 时间开销。在引入系统 span 缓存的 PR 中，我们的"many foxes"压力测试从 5.35 ms 降到了 4.54 ms。在添加并行迭代 span 缓存的 PR 中，我们的"many cubes"压力测试从 8.89 ms 降到了 6.8 ms。

![tracing overhead](https://bevy.org/news/bevy-0-12/tracing-overhead-reduction.png)

## AccessKit 集成改进 [#](https://bevy.org/news/bevy-0-12/#accesskit-integration-improvements)

作者：@ndarilek

Bevy 0.10 的 [AccessKit](https://accesskit.dev/) 集成使得引擎能够非常容易地带头向无障碍树推送更新。但正如任何好的舞伴所知，有时候最好的领导方式是跟随。

此版本增加了 `ManageAccessibilityUpdates` 资源，当设置为 `false` 时，会阻止引擎自行更新无障碍树。这为支持 Bevy 和 AccessKit 集成的第三方 UI 直接向 Bevy 发送更新铺平了道路。当 UI 准备交还控制权时，`ManageAccessibilityUpdates` 设置为 `true`，Bevy 会从停下的地方继续并再次开始发送更新。

AccessKit 本身也得到了简化，此版本利用这一点缩小了我们集成的表面积。如果你对内部工作原理感到好奇或想提供帮助，`bevy_a11y` crate 现在比以往更容易上手。

## TypePath 迁移 [#](https://bevy.org/news/bevy-0-12/#typepath-migration)

作者：@soqb

作为在 **Bevy 0.11** 中引入的 [稳定 TypePath](https://bevy.org/news/bevy-0-11/#stable-typepath) 的后续工作，Bevy Reflect 现在使用 [`TypePath`](https://docs.rs/bevy/0.12.0/bevy/reflect/trait.TypePath.html) 代替 [`type_name`](https://doc.rust-lang.org/std/any/fn.type_name.html)。现在可以通过 [`TypeInfo`](https://docs.rs/bevy/0.12.0/bevy/reflect/enum.TypeInfo.html) 访问反射类型的 [`TypePath`](https://docs.rs/bevy/0.12.0/bevy/reflect/trait.TypePath.html)，并且 [`DynamicTypePath`](https://docs.rs/bevy/0.12.0/bevy/reflect/trait.DynamicTypePath.html) 和 [`type_name`](https://doc.rust-lang.org/std/any/fn.type_name.html) 方法已被移除。

## 改进的 bevymark 示例 [#](https://bevy.org/news/bevy-0-12/#improved-bevymark-example)

作者：@superdump (Rob Swain), @IceSentry

需要对 bevymark 示例进行改进，以便对批处理/实例化绘制更改进行基准测试。增加了以下模式：

- 绘制 2D 四边形网格而不是精灵：`--mode mesh2d`
- 改变每个实例的颜色数据，而不是仅改变每波鸟的颜色：`--vary-per-instance`
- 生成一定数量的材质/精灵纹理，并根据每实例变动设置，按波次或按实例随机选择：`--material-texture-count 10`
- 以随机 z 顺序（新默认）或绘制顺序生成鸟：`--ordered-z`

这使得可以在下一节中针对不同批处理/实例化情况进行基准测试。

## CI 改进 [#](https://bevy.org/news/bevy-0-12/#ci-improvements)

作者：@ameknite, @mockersf

为了帮助确保示例在 Bevy 仓库之外也可复用，如果示例使用来自 `bevy_internal` 而非 `bevy` 的导入，CI 现在将失败。

此外，每日移动端检查任务现在在更多 iOS 和 Android 设备上构建：

- 运行 iOS 15 的 iPhone 13
- 运行 iOS 16 的 iPhone 14
- 运行 iOS 17 的 iPhone 15
- 运行 Android 11 的 Xiaomi Redmi Note 11
- 运行 Android 12 的 Google Pixel 6
- 运行 Android 13 的 Samsung Galaxy S23
- 运行 Android 14 的 Google Pixel 8

## 示例工具改进 [#](https://bevy.org/news/bevy-0-12/#example-tooling-improvements)

作者：@mockersf

示例展示工具现在可以为 WebGL2 或 WebGPU 构建所有示例。这用于更新网站上的所有 Wasm 兼容示例，你可以在[此处](https://bevy.org/examples/)找到 WebGL2 版本，或在[此处](https://bevy.org/examples-webgpu/)找到 WebGPU 版本。

它现在还能够运行所有示例时截取屏幕截图：

```sh
cargo run -p example-showcase -- run --screenshot
```

有一些选项可以帮助执行，你可以使用 `--help` 查看。

这些屏幕截图显示在网站上的示例页面上，并且可以用来检查 PR 是否引入了可见的回归。

## CI 中的示例执行 [#](https://bevy.org/news/bevy-0-12/#example-execution-in-ci)

作者：@mockersf, @rparrett

现在所有示例都在 CI 中执行——在 Windows 上使用 DX12，在 Linux 上使用 Vulkan。在可能的情况下，会截取屏幕截图并与上次执行进行比较。如果示例崩溃，则会保存日志。移动端示例也在与每日移动端检查任务相同的设备上执行。

所有这些执行的报告已生成，可在[此处](https://thebevyflock.github.io/bevy-example-runner/)获取。

[![Example Report|1099x614](https://bevy.org/news/bevy-0-12/example-report.png)](https://thebevyflock.github.io/bevy-example-runner/)

如果你想帮助赞助在更多平台上的测试，请与我们联系！

## 接下来是什么？ [#](https://bevy.org/news/bevy-0-12/#what-s-next)

我们有很多正在进行的工作！其中一些可能会在 **Bevy 0.13** 中落地。

查看 [**Bevy 0.13 Milestone**](https://github.com/bevyengine/bevy/milestone/17) 获取正在考虑用于 **Bevy 0.13** 的最新工作列表。

- **Bevy 场景和 UI 演进**：我们正在努力构建 Bevy 的新场景和 UI 系统。我们正在试验一个全新的[整体性场景/UI 系统](https://github.com/bevyengine/bevy/discussions/9538)，希望这将为 Bevy 编辑器奠定基础，并使在 Bevy 中定义场景变得更加灵活、强大且符合人体工程学。
- **更多批处理/实例化改进**：将蒙皮网格数据放入存储缓冲区，以实现具有相同网格/蒙皮/材质的蒙皮网格实体的实例化绘制。将材质数据放入新的 GpuArrayBuffer 中，以实现对具有相同网格、材质类型和纹理但材质数据不同的实体的批量绘制。
- **GPU 驱动渲染**：我们计划通过在计算着色器（在支持的平台上）中创建绘制调用来驱动 GPU 渲染。我们有[使用 meshlet 的实验](https://github.com/bevyengine/bevy/pull/10164)，并计划探索其他方法。这将涉及将纹理放入无绑定纹理数组，并将网格放入一个大的缓冲区中以避免重新绑定。
- **曝光设置**：控制[相机曝光设置](https://github.com/bevyengine/bevy/pull/8407)以改变渲染的氛围和情绪！
- **GPU 拾取**：在 GPU 上以像素完美精度[高效地选择对象](https://github.com/bevyengine/bevy/pull/8784)！
- **逐物体运动模糊**：使用运动矢量[在物体移动时使其模糊](https://github.com/bevyengine/bevy/pull/9924)
- **UI 节点圆角和阴影**：在 Bevy UI 中支持[圆角和阴影](https://github.com/bevyengine/bevy/pull/8973)
- **系统步进**：通过[逐步运行系统](https://github.com/bevyengine/bevy/pull/8453)来调试应用，针对给定帧
- **自动同步点**：支持在具有依赖关系的系统之间[自动插入同步点](https://github.com/bevyengine/bevy/pull/9822)，消除了手动插入的需要，并解决了一个常见的错误来源。
- **光照贴图支持**：支持[渲染预烘焙光照贴图](https://github.com/bevyengine/bevy/pull/10231)
