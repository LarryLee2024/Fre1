# Bevy 0.14

## 发布于 2024 年 7 月 4 日，Bevy 贡献者

![展示 Bevy 新体积雾、景深和屏幕空间反射的森林场景](https://bevy.org/news/bevy-0-14/cover.jpg)

[展示 Bevy 新体积雾、景深和屏幕空间反射的森林场景](https://github.com/IceSentry/bevy_forest_scene)

感谢 **256** 位贡献者、**993** 个拉取请求、社区审阅者以及我们的[**慷慨捐赠者**](https://bevy.org/donate)，我们很高兴在 [crates.io](https://crates.io/crates/bevy) 上发布 **Bevy 0.14**！

对于那些还不了解的人，Bevy 是一个用 Rust 构建的、令人耳目一新的简单数据驱动游戏引擎。您可以查看我们的[快速入门指南](https://bevy.org/learn/quick-start)立即试用。它永久免费且开源！您可以在 GitHub 上获取完整[源代码](https://github.com/bevyengine/bevy)。查看 [Bevy Assets](https://bevy.org/assets) 获取社区开发的插件、游戏和学习资源集合。

要将现有的 Bevy App 或 Plugin 更新到 **Bevy 0.14**，请查看我们的[0.13 到 0.14 迁移指南](https://bevy.org/learn/migration-guides/0-13-to-0-14/)。

自几个月前上一个版本发布以来，我们添加了大量新功能、错误修复和生活质量改进，以下是其中一些亮点：

- **虚拟几何体（Virtual Geometry）**：将网格预处理为"meshlet"，实现海量几何体的高效渲染
- **锐利屏幕空间反射（Sharp Screen Space Reflections）**：近似的实时光线步进屏幕空间反射
- **景深（Depth of Field）**：使特定深度的物体"失焦"，模拟物理镜头的特性
- **逐物体运动模糊（Per-Object Motion Blur）**：模糊相对于相机快速移动的物体
- **体积雾/光照（Volumetric Fog/Lighting）**：在 3D 空间中模拟雾，使光源产生美丽的"上帝光"
- **电影级颜色分级（Filmic Color Grading）**：使用一整套电影级颜色分级工具微调游戏中的色调映射
- **PBR 各向异性（PBR Anisotropy）**：改善表面粗糙度沿网格切线和副切线方向变化的材质渲染，如拉丝金属和头发
- **自动曝光（Auto Exposure）**：配置摄像机根据其观察内容动态调整曝光
- **点光源 PCF 阴影**：柔化点光源阴影，提高质量
- **动画混合（Animation blending）**：我们新的底层动画图添加了动画混合支持，为第一方和第三方图形化、资源驱动的动画工具奠定了基础
- **ECS Hooks 和 Observer**：自动（且立即）响应任意事件，如组件的添加和移除
- **更好的颜色**：类型安全颜色使您清楚自己在哪个色彩空间中操作，并提供一系列有用的方法
- **计算状态和子状态（Computed states and substates）**：通过这些对 `States` 抽象的类型安全扩展，建模复杂的应用状态变得轻而易举
- **圆角**：解决了 `bevy_ui` 最粗糙的边缘之一，您现在可以程序化地设置 UI 元素的圆角半径

首次，Bevy 0.14 采用了**发布候选（release candidate）**流程来帮助确保您可以放心地立即升级。我们与插件作者和普通用户密切合作，捕捉关键错误、打磨新功能的粗糙边缘，并完善迁移指南。在准备修复时，我们在 [crates.io 上发布了新的候选版本](https://crates.io/crates/bevy/versions?sort=date)，让核心生态系统 crate 更新并密切关注阻碍性问题。非常感谢[所有提供帮助的人](https://discord.com/channels/691052431525675048/1239930965267054623)：这些努力是使 Bevy 成为大小团队可以信赖的可靠工具的关键一步。

## 虚拟几何体（实验性）[#](https://bevy.org/news/bevy-0-14/#virtual-geometry-experimental)

作者：[JMS55](https://github.com/JMS55)、[atlv24](https://github.com/atlv24)、[zeux](https://github.com/zeux)、[ricky26](https://github.com/ricky26)

PR：[#10164](https://github.com/bevyengine/bevy/pull/10164)

经过几个月的艰苦工作，我们非常兴奋地为您带来新的虚拟几何体功能的实验性版本！

这个新的渲染功能与 Unreal Engine 5 的 Nanite 渲染器非常相似。您可以获取一个非常高多边形的网格，在构建时对其进行预处理以生成 [`MeshletMesh`](https://docs.rs/bevy/0.14/bevy/pbr/experimental/meshlet/struct.MeshletMesh.html)，然后在运行时渲染海量的几何体 —— 远超 Bevy 标准渲染器所能支持的。无需显式 LOD —— 这一切都是自动的，且近乎无缝。

此功能仍在开发中，与 Bevy 的标准渲染器相比有一些限制，因此请务必阅读文档并报告您遇到的任何错误。我们还有很多工作要做，敬请期待未来版本中更多的性能改进（以及相关的破坏性变更）！

请注意，此功能不使用 GPU "网格着色器"，因此旧 GPU 目前仍然兼容。但不推荐使用，且在不久的将来很可能不再支持。

除下面的使用指南外，请查看：

- [此功能的 Bevy 示例](https://github.com/bevyengine/bevy/blob/release-0.14.0/examples/3d/meshlet.rs)
- [此功能主要作者的技术深度文章](https://jms55.github.io/posts/2024-06-09-virtual-geometry-bevy-0-14)

想要使用虚拟几何体的用户应在运行时使用 `meshlet` cargo feature 编译，并在构建时使用 `meshlet_processor` cargo feature 将网格预处理为 meshlet 渲染器使用的特殊 meshlet 特定格式（[`MeshletMesh`](https://docs.rs/bevy/0.14/bevy/pbr/experimental/meshlet/struct.MeshletMesh.html)）。

启用 meshlet 功能会解锁一个新模块：[`bevy::pbr::experimental::meshlet`](https://docs.rs/bevy/0.14/bevy/pbr/experimental/meshlet/index.html)。

第一步，将 [`MeshletPlugin`](https://docs.rs/bevy/0.14/bevy/pbr/experimental/meshlet/struct.MeshletPlugin.html) 添加到您的应用中：

```rust
app.add_plugins(MeshletPlugin);
```

接下来，将您的 [`Mesh`](https://docs.rs/bevy/0.14/bevy/prelude/struct.Mesh.html) 预处理为 [`MeshletMesh`](https://docs.rs/bevy/0.14/bevy/pbr/experimental/meshlet/struct.MeshletMesh.html)。目前，这需要通过 `MeshletMesh::from_mesh()` 手动完成（再次强调，您需要启用 `meshlet_processor` feature）。此步骤相当慢，应提前完成一次，然后保存到资源文件中。请注意，支持的网格和材质类型有限制，请务必阅读文档。

通过 Bevy 的资源预处理系统自动进行 GLTF/场景转换的计划已经排上，但不幸未能在本版本中及时完成。目前，您需要自己搭建资源转换和管理系统。如果您想出了好的系统，请告诉我们！

现在，生成您的实体。与 `MaterialMeshBundle` 类似，有一个 `MaterialMeshletMeshBundle`，它使用 [`MeshletMesh`](https://docs.rs/bevy/0.14/bevy/pbr/experimental/meshlet/struct.MeshletMesh.html) 代替典型的 [`Mesh`](https://docs.rs/bevy/0.14/bevy/prelude/struct.Mesh.html)。

```rust
commands.spawn(MaterialMeshletMeshBundle {
    meshlet_mesh: meshlet_mesh_handle.clone(),
    material: material_handle.clone(),
    transform,
    ..default()
});
```

最后，关于材质的一点说明。Meshlet 实体使用与常规网格实体相同的 [`Material`](https://docs.rs/bevy/0.14/bevy/pbr/trait.Material.html) trait，但不使用标准材质方法。而是有 3 个新方法：`meshlet_mesh_fragment_shader`、`meshlet_mesh_prepass_fragment_shader` 和 `meshlet_mesh_deferred_fragment_shader`。支持 forward、forward with prepasses 和 deferred 渲染所有三种方法。

但请注意，无法访问顶点着色器。Meshlet 渲染使用硬编码的顶点着色器，不可更改。

Meshlet 材质的实际片段着色器代码与常规网格实体的片段着色器基本相同。关键区别在于，不再使用：

```rust
@fragment
fn fragment(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    // ...
}
```

You should use this:

```rust
#import bevy_pbr::meshlet_visibility_buffer_resolve::resolve_vertex_output

@fragment
fn fragment(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    let vertex_output = resolve_vertex_output(frag_coord);
    // ...
}
```

## 锐利屏幕空间反射（Sharp Screen-Space Reflections）[#](https://bevy.org/news/bevy-0-14/#sharp-screen-space-reflections)

作者：[pcwalton](https://github.com/pcwalton)

PR：[#13418](https://github.com/bevyengine/bevy/pull/13418)

拖动此图片进行比较

![无 SSR](https://bevy.org/news/bevy-0-14/no_ssr.jpg)![有 SSR](https://bevy.org/news/bevy-0-14/ssr.jpg)

[屏幕空间反射](https://lettier.github.io/3d-game-shaders-for-beginners/screen-space-reflection.html)（SSR）通过对深度缓冲区进行光线步进并从最终渲染帧复制样本来近似实时反射。我们的初始实现相对最小化，以提供灵活的基础进行扩展，但它基于 [Tomasz Stachowiak](https://gist.github.com/h3r2tic/9c8356bdaefbe80b1a22ae0aaee192db) 的制作质量光线步进代码，他是独立游戏宠儿 Bevy 游戏 [Tiny Glade](https://store.steampowered.com/app/2198150/Tiny_Glade/) 的创建者之一。因此，有几个注意事项需要牢记：

1. 目前，此功能构建在延迟渲染器之上，仅在该模式下支持。前向屏幕空间反射是可能的，尽管不常见（例如《毁灭战士：永恒》使用了它们）；但它们需要从上一帧进行追踪，这会增加复杂度。此补丁为在前向渲染路径中实现 SSR 敞开了大门，但其本身没有这样的实现。
2. WebGL 2 不支持屏幕空间反射，因为它们需要从深度缓冲区采样，而 `naga` 由于一个 bug 无法做到（错误地生成了 `sampler2DShadow` 而不是 `sampler2D`；这也是该平台上禁用景深效果的相同原因）。
3. 完全没有执行时域滤波或模糊。因此，SSR 目前仅适用于非常低粗糙度/光滑的表面。
4. 我们不通过层级 Z 缓冲进行加速，反射以全分辨率追踪。因此，您可能会根据场景和硬件注意到性能问题。

要向摄像机添加屏幕空间反射，请插入 [`ScreenSpaceReflectionsSettings`](https://docs.rs/bevy/0.14/bevy/pbr/struct.ScreenSpaceReflectionsSettings.html) 组件。除了 [`ScreenSpaceReflectionsSettings`](https://docs.rs/bevy/0.14/bevy/pbr/struct.ScreenSpaceReflectionsSettings.html) 之外，还必须存在 [`DepthPrepass`](https://docs.rs/bevy/0.14/bevy/core_pipeline/prepass/struct.DepthPrepass.html) 和 [`DeferredPrepass`](https://docs.rs/bevy/0.14/bevy/core_pipeline/prepass/struct.DeferredPrepass.html) 才能显示反射。方便的是，[`ScreenSpaceReflectionsBundle`](https://docs.rs/bevy/0.14/bevy/pbr/struct.ScreenSpaceReflectionsBundle.html) 为您打包了所有这些！虽然 [`ScreenSpaceReflectionsSettings`](https://docs.rs/bevy/0.14/bevy/pbr/struct.ScreenSpaceReflectionsSettings.html) 带有合理的默认值，它还包含了一些艺术家可以调整的设置。

## 体积雾和体积光照（光柱/上帝光）[#](https://bevy.org/news/bevy-0-14/#volumetric-fog-and-volumetric-lighting-light-shafts-god-rays)

作者：[pcwalton](https://github.com/pcwalton)

PR：[#13057](https://github.com/bevyengine/bevy/pull/13057)

并非所有的雾都一样。Bevy 现有的实现涵盖了[距离雾](https://en.wikipedia.org/wiki/Distance_fog)，它快速、简单，但不太真实。

在 Bevy 0.14 中，补充了基于[体积光照](https://en.wikipedia.org/wiki/Volumetric_lighting)的体积雾，它使用实际的 3D 空间而不是简单的摄像机距离来模拟雾。如您所料，这更漂亮，但计算成本也更高！

特别是，这允许创建令人惊艳的"上帝光"（更准确地说，曙暮光条）穿过雾气的效果。

拖动此图片进行比较

![无体积雾](https://bevy.org/news/bevy-0-14/without_volumetric_fog.jpg)![有体积雾](https://bevy.org/news/bevy-0-14/with_volumetric_fog.jpg)

Bevy 的算法——作为后处理效果实现——是 [Scratchapixel](https://www.scratchapixel.com/lessons/3d-basic-rendering/volume-rendering-for-developers/intro-volume-rendering.html) 和 [Alexandre Pestana 的博客文章](https://www.alexandre-pestana.com/volumetric-lights/) 中描述的技术组合。它在屏幕空间中使用光线步进（[由 h3r2tic 移植到 WGSL](https://gist.github.com/h3r2tic/9c8356bdaefbe80b1a22ae0aaee192db)），转换到阴影贴图空间进行采样，并结合了基于物理的吸收和散射建模。Bevy 采用了广泛使用的 Henyey-Greenstein 相位函数来建模非对称性；这基本上允许光柱在用户观看时淡入和淡出。

要向场景添加体积雾，请将 [`VolumetricFogSettings`](https://docs.rs/bevy/0.14/bevy/pbr/struct.VolumetricFogSettings.html) 添加到摄像机，并将 [`VolumetricLight`](https://docs.rs/bevy/0.14/bevy/pbr/struct.VolumetricLight.html) 添加到您希望产生体积效果的定向光。[`VolumetricFogSettings`](https://docs.rs/bevy/0.14/bevy/pbr/struct.VolumetricFogSettings.html) 有许多设置，允许您定义模拟的精度以及雾的外观。目前，仅支持与具有阴影贴图的定向光的交互。请注意，效果的开销与使用的定向光数量直接成正比，因此为了获得最佳效果，请谨慎使用 [`VolumetricLight`](https://docs.rs/bevy/0.14/bevy/pbr/struct.VolumetricLight.html)。

通过我们的 [`volumetric_fog` 示例](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/3d/volumetric_fog.rs)亲身体验。

## 逐物体运动模糊（Per-Object Motion Blur）[#](https://bevy.org/news/bevy-0-14/#per-object-motion-blur)

作者：[aevyrie](https://github.com/aevyrie)、[torsteingrindvik](https://github.com/torsteingrindvik)

PR：[#9924](https://github.com/bevyengine/bevy/pull/9924)

我们添加了一个后处理效果，可以沿运动方向模糊快速移动的物体。我们的实现使用运动向量，这意味着它可以与 Bevy 内置的 PBR 材质、蒙皮网格或任何写入运动向量和深度的其他内容一起使用。该效果用于传达高速运动，否则当图像完全清晰时，高速运动可能看起来像是闪烁或瞬移。

模糊程度与物体相对于摄像机的运动成正比。如果摄像机追踪一个快速移动的物体（如车辆），车辆将保持清晰，而静止物体将被模糊。反之，如果摄像机对准静止物体，而快速移动的车辆穿过画面，则只有快速移动的物体会被模糊。

该实现通过[摄像机快门角度](https://en.wikipedia.org/wiki/Rotary_disc_shutter)进行配置，它对应虚拟快门在帧期间打开的时间。实际上，这意味着效果随帧率缩放，因此运行在高刷新率下的用户不会受到过度模糊的影响。

您可以通过向摄像机实体添加 [`MotionBlurBundle`](https://docs.rs/bevy/0.14/bevy/core_pipeline/motion_blur/struct.MotionBlurBundle.html) 来启用运动模糊，如我们的 [`motion blur` 示例](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/3d/motion_blur.rs)所示。

## 电影级颜色分级（Filmic Color Grading）[#](https://bevy.org/news/bevy-0-14/#filmic-color-grading)

作者：[pcwalton](https://github.com/pcwalton)

PR：[#13121](https://github.com/bevyengine/bevy/pull/13121)

美术师希望为他们的游戏获得准确的外观，而颜色起着巨大的作用。

为此，Bevy [现有的色调映射工具](https://bevy.org/news/bevy-0-10/#more-tonemapping-choices)已扩展为包含一整套电影级颜色分级工具。除了[基础色调映射](https://docs.rs/bevy/0.14/bevy/core_pipeline/tonemapping/enum.Tonemapping.html)，您现在还可以配置：

- 白点调整。这受到 Unity 实现的启发，但经过简化和优化。色温和色调控制 CIE 1931 的 x 和 y 色度值的调整。跟随 Unity，调整是相对于 LMS 色彩空间中的 D65 标准照明体进行的。
- 色调旋转：将 RGB 值转换为 HSV，改变色调，再转换回来。
- 颜色校正：允许根据标准 ASC CDL 组合函数调整 gamma、增益和 lift 值。这可以分别对阴影、中间调和高光进行。为避免颜色突变，在图像的不同部分之间使用小的交叉淡入淡出。

我们尽可能贴近 [Blender](https://www.blender.org/) 的实现，以确保您在建模软件中看到的内容与在游戏中看到的内容一致。

![A very orange image of a test scene, with controls for exposure, temperature, tint and hue. Saturation, contrast, gamma, gain, and lift can all be configured for the highlights, midtones, and shadows separately.](https://bevy.org/news/bevy-0-14/filmic_color_grading.jpg)

我们提供了一个新的 [`color_grading`](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/3d/color_grading.rs) 示例，带有一个漂亮的 GUI 来更改所有颜色分级设置。非常适合复制粘贴到您自己游戏的开发工具中并调整设置！请注意，所有这些设置都可以在运行时更改：让美术师控制场景的精确氛围，或根据天气或一天中的时间动态变化。

## 自动曝光（Auto Exposure）[#](https://bevy.org/news/bevy-0-14/#auto-exposure)

作者：[Kurble](https://github.com/Kurble)、[alice-i-cecile](https://github.com/alice-i-cecile)

PR：[#12792](https://github.com/bevyengine/bevy/pull/12792)

自 **Bevy 0.13** 以来，您可以[配置摄像机的 EV100](https://bevy.org/news/bevy-0-13/#camera-exposure)，以基于物理的方式调整摄像机曝光。这也允许您为各种效果动态更改曝光值。然而，这是一个手动过程，需要您自己调整曝光值。

**Bevy 0.14** 引入了**自动曝光（Auto Exposure）**，它根据场景的亮度自动调整摄像机曝光。当您想营造非常高的动态范围的感觉时，这很有用，因为您的眼睛也会适应亮度的剧烈变化。请注意，这不是手动调整曝光值的替代品，而是在亮度快速变化时可用于创造戏剧效果的额外工具。查看此[示例](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/3d/auto_exposure.rs)录制的视频，看看它的效果！

Bevy 的自动曝光通过在后处理步骤中生成场景亮度的**直方图**来实现。然后基于直方图的平均值调整曝光。因为直方图是使用计算着色器计算的，自动曝光**在 WebGL 上不可用**。它默认也未启用，因此您需要将 [`AutoExposurePlugin`](https://docs.rs/bevy/0.14/bevy/core_pipeline/auto_exposure/struct.AutoExposurePlugin.html) 添加到您的应用中。

自动曝光由 [`AutoExposureSettings`](https://docs.rs/bevy/0.14/bevy/core_pipeline/auto_exposure/struct.AutoExposureSettings.html) 组件控制，您可以将其添加到摄像机实体。您可以配置以下几项：

- 曝光可以变化的 f 档相对**范围**。
- 曝光变化的**速度**。
- 可选的**测光遮罩**，允许您例如给图像中心更多权重。
- 可选的直方图**过滤器**，允许您忽略非常亮或非常暗的像素。

## 快速景深（Fast Depth of Field）[#](https://bevy.org/news/bevy-0-14/#fast-depth-of-field)

作者：[pcwalton](https://github.com/pcwalton)、[alice-i-cecile](https://github.com/alice-i-cecile)、[Kurble](https://github.com/Kurble)

PR：[#13009](https://github.com/bevyengine/bevy/pull/13009)

在渲染中，**景深**是一种模拟[物理镜头局限](https://en.wikipedia.org/wiki/Depth_of_field)的效果。由于光的工作方式，镜头（如人眼或电影摄像机）只能聚焦于特定范围（深度）内的物体，导致其他所有物体模糊失焦。

Bevy 现在内置了此效果，作为后处理着色器实现。有两种选择：快速高斯模糊或更物理精确的六边形散景技术。散景模糊通常比高斯模糊在美学上更令人愉悦，因为它更准确地模拟了相机的效果。散景圆圈的形状由光圈叶片数决定。在我们的案例中，我们使用六边形，这通常被认为是低质量相机的特征。

拖动此图片进行比较

![无景深](https://bevy.org/news/bevy-0-14/no_dof.jpg)![散景景深](https://bevy.org/news/bevy-0-14/bokeh_dof.jpg)

模糊量通常由 [f-number](https://en.wikipedia.org/wiki/F-number) 指定，我们使用它根据胶片尺寸和[视场角](https://en.wikipedia.org/wiki/Field_of_view)计算[焦距](https://en.wikipedia.org/wiki/Focal_length)。默认情况下，我们模拟标准电影摄像机，f-number 为 f/1，胶片尺寸对应经典的 Super 35 胶片格式。开发者可以根据需要自定义这些值。

要查看这个新 API 的用法，请查看专门的 [`depth_of_field` 示例](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/3d/depth_of_field.rs)。

## PBR 各向异性（PBR Anisotropy）[#](https://bevy.org/news/bevy-0-14/#pbr-anisotropy)

作者：[pcwalton](https://github.com/pcwalton)

PR：[#13450](https://github.com/bevyengine/bevy/pull/13450)

[各向异性材质](https://en.wikipedia.org/wiki/Anisotropy)会随运动轴发生变化，比如木材顺着纹理和逆着纹理加工时的表现截然不同。但在基于物理的渲染中，**各向异性**特指一种允许粗糙度沿网格的切线和副切线方向变化的功能。实际上，这会使镜面光拉伸成线条而不是圆形的光斑。这对于模拟拉丝金属、头发和类似表面非常有用。对各向异性的支持是主要游戏和图形引擎的常见功能；Unity、Unreal、Godot、three.js 和 Blender 都在不同程度上支持它。

拖动此图片进行比较

![无各向异性](https://bevy.org/news/bevy-0-14/without_anisotropy.jpg)![有各向异性](https://bevy.org/news/bevy-0-14/with_anisotropy.jpg)

[`StandardMaterial`](https://docs.rs/bevy/0.14/bevy/pbr/struct.StandardMaterial.html) 新增了两个参数：`anisotropy_strength` 和 `anisotropy_rotation`。各向异性强度（范围 0 到 1）表示网格切线和副切线之间粗糙度的差异程度。实际上，它控制镜面高光的拉伸程度。各向异性旋转允许粗糙度方向与模型的切线方向不同。

除了这两个固定参数外，还可以提供各向异性纹理，格式为 `KHR_materials_anisotropy` 指定的线性纹理格式。

和往常一样，到对应的 [`anisotropy` 示例](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/3d/anisotropy.rs)中试试吧。

## 点光源百分比渐近滤波（PCF）[#](https://bevy.org/news/bevy-0-14/#percentage-closer-filtering-pcf-for-point-lights)

作者：[pcwalton](https://github.com/pcwalton)

PR：[#12910](https://github.com/bevyengine/bevy/pull/12910)

百分比渐近滤波是一种标准的抗锯齿技术，用于获得更柔和、锯齿更少的阴影。其做法是使用高斯核从感兴趣像素附近的阴影贴图采样，对结果进行平均，以减少进出阴影时的突变。

因此，Bevy 的点光源现在看起来更柔和、更自然，终端用户代码无需任何更改。和以前一样，您可以通过在 3D 摄像机上设置 [`ShadowFilteringMethod`](https://docs.rs/bevy/0.14/bevy/pbr/enum.ShadowFilteringMethod.html) 组件来配置用于阴影抗锯齿的具体策略。

拖动此图片进行比较

![无 PCF 滤波](https://bevy.org/news/bevy-0-14/before_pcf.jpg)![有 PCF 滤波](https://bevy.org/news/bevy-0-14/after_pcf.jpg)

对百分比渐近阴影的完整支持[正在开发中](https://github.com/bevyengine/bevy/pull/13497)：像往常一样，非常欢迎测试和审阅。

## 亚像素形态抗锯齿（SMAA）[#](https://bevy.org/news/bevy-0-14/#subpixel-morphological-antialiasing-smaa)

作者：[pcwalton](https://github.com/pcwalton)、[alice-i-cecile](https://github.com/alice-i-cecile)

PR：[#13423](https://github.com/bevyengine/bevy/pull/13423)

锯齿边缘是游戏开发者存在的诅咒：人们发明了各种各样的抗锯齿技术来解决它们而不降低图像质量。除了 [MSAA](https://en.wikipedia.org/wiki/Multisample_anti-aliasing)、[FXAA](https://en.wikipedia.org/wiki/Fast_approximate_anti-aliasing) 和 [TAA](https://en.wikipedia.org/wiki/Temporal_anti-aliasing) 之外，Bevy 现在实现了 [SMAA](https://en.wikipedia.org/wiki/Morphological_antialiasing)：亚像素形态抗锯齿。

SMAA 是一种 2011 年的抗锯齿技术，它检测图像中的边界，然后对附近边界像素进行平均，消除了可怕的锯齿。尽管年代久远，但它十多年来一直是游戏的中流砥柱。有四种质量预设可用：低、中、高和超高。由于消费级硬件的进步，Bevy 的默认设置为高。

您可以在下面两幅图中看到它与无抗锯齿的对比：

拖动此图片进行比较

![无 AA](https://bevy.org/news/bevy-0-14/no_aa.jpg)![SMAA](https://bevy.org/news/bevy-0-14/smaa.jpg)

了解各种抗锯齿方法权衡的最佳方法是使用 [`anti_aliasing` 示例](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/3d/anti_aliasing.rs)在测试场景中进行实验，或者直接在您自己的游戏中尝试。

## 可见性范围（层级细节层次/HLOD）[#](https://bevy.org/news/bevy-0-14/#visibility-ranges-hierarchical-levels-of-detail-hlods)

作者：[pcwalton](https://github.com/pcwalton)、[cart](https://github.com/cart)

PR：[#12916](https://github.com/bevyengine/bevy/pull/12916)

当看远处的物体时，很难分辨出细节！这个显而易见的事实无论在实际生活中还是在渲染中都是成立的。因此，为远处的物体使用复杂的高保真模型是一种浪费：我们可以用简化的等效物替换它们的网格。

通过这种方式自动改变模型的**细节层次（LOD）**，我们可以渲染更大的场景（或具有更高绘制距离的同一开放世界），根据物体到玩家的距离动态交换网格。Bevy 现在支持了其中最基本的工具之一：**可见性范围**（有时称为层级细节层次，因为它允许用户用单个对象替换多个网格）。

通过在网格实体上设置 [`VisibilityRange`](https://docs.rs/bevy/0.14/bevy/render/view/struct.VisibilityRange.html) 组件，开发者可以自动控制网格相对于摄像机出现和消失的距离范围，并使用抖动在两种选项之间自动淡入淡出。隐藏网格发生在渲染管线的早期，因此此功能可以高效地用于细节层次优化。额外的好处是，此功能会按视图正确评估，因此不同视图可以显示不同的细节层次。

请注意，此功能与真正的网格 LOD（几何体本身自动简化）不同，后者将在未来推出。虽然网格 LOD 对优化很有用且不需要任何额外设置，但它们不如可见性范围灵活。游戏通常希望使用网格以外的对象来替换远处的模型，例如八面体或[公告牌](https://github.com/bevyengine/bevy/issues/3688)模拟物：首先实现可见性范围让用户可以灵活地立即开始实现这些解决方案。

您可以在 [`visibility_range` 示例](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/3d/visibility_range.rs)中了解此功能的用法。

## ECS Hooks 和 Observer [#](https://bevy.org/news/bevy-0-14/#ecs-hooks-and-observers)

作者：[james-j-obrien](https://github.com/james-j-obrien)、[cart](https://github.com/cart)

PR：[#10839](https://github.com/bevyengine/bevy/pull/10839)

虽然我们热爱在 Bevy 中通过紧密循环处理同质数据块，但并非每个任务都完美适合直接的 ECS 模型。响应变化和/或处理事件是任何应用中的关键任务，游戏也不例外。

Bevy 已经有多种不同的工具来处理这一点：

- **带缓冲的 [`Event`](https://docs.rs/bevy/0.14/bevy/ecs/event/trait.Event.html)**：多生产者、多消费者队列。灵活且高效，但需要作为 Schedule 的一部分定期轮询。事件在两帧后被丢弃。
- **通过 [`Added`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Added.html) 和 [`Changed`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Changed.html) 进行变更检测**：允许编写能够响应已添加或已变更组件的 Query。这些 Query 线性扫描匹配 Query 的组件的变更状态，以检查它们是否已被添加或变更。
- **[`RemovedComponents`](https://docs.rs/bevy/latest/bevy/ecs/prelude/struct.RemovedComponents.html)**：一种特殊形式的事件，当组件从实体中移除或带有该组件的实体被销毁时触发。

所有这些（以及 System 本身！）都使用了一种"拉"式机制：无论是否有人在监听，事件都会被发送，而监听器必须定期轮询来询问是否有变化。这是一个有用的模式，我们希望保留它！通过轮询，我们可以批量处理事件，获得更多上下文并改善数据局部性（让 CPU 飞起来）。

但它也有一些限制：

- 事件被触发和处理响应之间存在不可避免的延迟
- 轮询每帧都会引入少量（但非零）的开销

这种延迟是关键问题：

- 数据（如索引或层级结构）可以存在（即使是瞬间的）于无效状态
- 我们无法在单周期内处理任意的递归逻辑事件链

为了克服这些限制，**Bevy 0.14** 引入了**组件生命周期 Hooks** 和 **Observer**：两种互补的"推"式机制，灵感来自一直很棒的 [flecs](https://www.flecs.dev/flecs/) ECS。

#### 组件生命周期 Hooks

[组件 Hooks](https://docs.rs/bevy/0.14/bevy/ecs/component/struct.ComponentHooks.html) 是为特定组件类型注册的函数（能够与 ECS World 交互），它们自动响应"组件生命周期事件"而运行，例如当该组件被添加、覆盖或移除时。

对于给定的组件类型，每个生命周期事件只能注册一个 hook，且不能被覆盖。

Hooks 存在是为了执行与该组件相关的不变量（例如维护索引或层级结构正确性）。Hooks 不能被移除，并且始终优先于 observers：它们在所有 on-add/on-insert observers 之前运行，但在所有 on-remove observers 之后运行。因此，它们可以被视为更接近构造函数和析构函数，更适合维护关键的安全性或正确性不变量。Hooks 也比 observers 稍快，因为它们较低的灵活性意味着涉及更少的查找。

让我们看一个需要维护不变量的简单示例：一个实体（带有 `Target` 组件）瞄准另一个实体（带有 `Targetable` 组件）。

```rust
#[derive(Component)]
struct Target(Option<Entity>);

#[derive(Component)]
struct Targetable {
    targeted_by: Vec<Entity>
};
```

我们希望在目标实体被销毁时自动清除 `Target`：怎么做？

如果我们使用基于拉的方法（本例中为 `RemovedComponents`），实体被销毁和 `Target` 组件被更新之间可能存在延迟。我们可以使用 hooks 消除这个延迟！

让我们看看在 `Targetable` 上使用 hook 是什么样子：

```rust
// 不使用派生宏，我们用自定义 Component 实现来配置 hooks
impl Component for Targetable {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        // Whenever this component is removed, or an entity with
        // this component is despawned...
        hooks.on_remove(|mut world, targeted_entity, _component_id|{
            // Grab the data that's about to be removed
            let targetable = world.get::<Targetable>(targeted_entity).unwrap();
            for targeting_entity in targetable.targeted_by {
                // Track down the entity that's targeting us
                let mut targeting = world.get_mut::<Target>(targeting_entity).unwrap();
                // And clear its target, cleaning up any dangling references
                targeting.0 = None;
            }
        })
    }
}
```

#### Observers

Observers are on-demand systems that listen to "triggered" events. These events can be triggered for specific entities _or_ they can be triggered "globally" (no entity target).

In contrast to hooks, observers are a flexible tool intended for higher level application logic. They can watch for when user-defined events are triggered.

```rust
#[derive(Event)]
struct Message {
    text: String
}

world.observe(|trigger: Trigger<Message>| {
    println!("{}", trigger.event().message.text);
});
```

Observers are run _immediately_ when an event they are watching for is triggered:

```rust
// All registered `Message` observers are immediately run here
world.trigger(Message { text: "Hello".to_string() });
```

If an event is triggered via a [`Command`](https://docs.rs/bevy/0.14/bevy/ecs/world/trait.Command.html), the observers will run when the [`Command`](https://docs.rs/bevy/0.14/bevy/ecs/world/trait.Command.html) is flushed:

```rust
fn send_message(mut commands: Commands) {
    // This will trigger all `Message` observers when this system's commands are flushed
    commands.trigger(Message { text: "Hello".to_string() } );
}
```

Events can also be triggered with an entity target:

```rust
#[derive(Event)]
struct Resize { size: usize }

commands.trigger_targets(Resize { size: 10 }, some_entity);
```

You can trigger an event for more than one entity at the same time:

```rust
commands.trigger_targets(Resize { size: 10 }, [e1, e2]);
```

A "global" observer will be executed when _any_ target is triggered:

```rust
fn main() {
    App::new()
        .observe(on_resize)
        .run()
}

fn on_resize(trigger: Trigger<Resize>, query: Query<&mut Size>) {
    let size = query.get_mut(trigger.entity()).unwrap();
    size.value = trigger.event().size;
} 
```

Notice that observers can use system parameters like [`Query`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Query.html), just like a normal system.

You can also add observers that only run for _specific_ entities:

```rust
commands
    .spawn(Widget)
    .observe(|trigger: Trigger<Resize>| {
        println!("This specific widget entity was resized!");
    });
```

Observer 实际上只是一个带有 [`Observer`](https://docs.rs/bevy/0.14/ecs/observer/struct.Observer.html) 组件的实体。上面使用的所有 `observe()` 方法只是生成新的 observer 实体的简写。这就是一个"全局"observer 实体的样子：

```rust
commands.spawn(Observer::new(|trigger: Trigger<Message>| {}));
```

同样，观察特定实体的 observer 看起来像这样

```rust
commands.spawn(
    Observer::new(|trigger: Trigger<Resize>| {})
        .with_entity(some_entity)
);
```

这个 API 使得管理和清理 observer 变得容易。它还支持高级用例，例如在多个目标之间共享 observer！

现在我们对 observer 有了一些了解，让我们通过一个简单的游戏风格示例来检查 API：

Click to expand...

```rust
use bevy::prelude::*;

#[derive(Event)]
struct DealDamage {
    damage: u8,
}

#[derive(Event)]
struct LoseLife {
    life_lost: u8,
}

#[derive(Event)]
struct PlayerDeath;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Life(u8);

#[derive(Component)]
struct Defense(u8);

#[derive(Component, Deref, DerefMut)]
struct Damage(u8);

#[derive(Component)]
struct Monster;

fn main() {
    App::new()
        .add_systems(Startup, spawn_player)
        .add_systems(Update, attack_player)
        .observe(on_player_death);
}

fn spawn_player(mut commands: Commands) {
    commands
        .spawn((Player, Life(10), Defense(2)))
        .observe(on_damage_taken)
        .observe(on_losing_life);
}

fn attack_player(
    mut commands: Commands,
    monster_query: Query<&Damage, With<Monster>>,
    player_query: Query<Entity, With<Player>>,
) {
    let player_entity = player_query.single();

    for damage in &monster_query {
        commands.trigger_targets(DealDamage { damage: damage.0 }, player_entity);
    }
}

fn on_damage_taken(
    trigger: Trigger<DealDamage>,
    mut commands: Commands,
    query: Query<&Defense>,
) {
    let defense = query.get(trigger.entity()).unwrap();
    let damage = trigger.event().damage;
    let life_lost = damage.saturating_sub(defense.0);
    // Observers can be chained into each other by sending more triggers using commands.
    // This is what makes observers so powerful ... this chain of events is evaluated
    // as a single transaction when the first event is triggered.
    commands.trigger_targets(LoseLife { life_lost }, trigger.entity());
}

fn on_losing_life(
    trigger: Trigger<LoseLife>,
    mut commands: Commands,
    mut life_query: Query<&mut Life>,
    player_query: Query<Entity, With<Player>>,
) {
    let mut life = life_query.get_mut(trigger.entity()).unwrap();
    let life_lost = trigger.event().life_lost;
    life.0 = life.0.saturating_sub(life_lost);

    if life.0 == 0 && player_query.contains(trigger.entity()) {
        commands.trigger(PlayerDeath);
    }
}

fn on_player_death(_trigger: Trigger<PlayerDeath>, mut app_exit: EventWriter<AppExit>) {
    println!("You died. Game over!");
    app_exit.send_default();
}
```

未来，我们计划使用 hooks 和 observers 来[替换 `RemovedComponents`](https://github.com/bevyengine/bevy/issues/13928)、[使我们的层级管理更健壮](https://github.com/bevyengine/bevy/issues/12235)、创建 [`bevy_eventlistener`](https://github.com/aevyrie/bevy_eventlistener) 的第一方替代方案（作为我们 UI 工作的一部分），以及[构建关系系统](https://github.com/bevyengine/rfcs/pull/79)。这些是强大的通用工具：我们迫不及待想看到社区用它们创造出怎样的奇妙成果！

当你准备好开始使用时，请查看 [`component hooks`](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/ecs/component_hooks.rs) 和 [`observers`](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/ecs/observers.rs) 示例以获取更多 API 细节。

## glTF KHR_texture_transform 支持 [#](https://bevy.org/news/bevy-0-14/#gltf-khr-texture-transform-support)

作者：[janhohenheim](https://github.com/janhohenheim)、[yrns](https://github.com/yrns)、[Kanabenki](https://github.com/Kanabenki)

PR：[#11904](https://github.com/bevyengine/bevy/pull/11904)

GLTF 扩展 `KHR_texture_transform` 用于在应用纹理之前对其进行变换。通过读取此扩展，Bevy 现在可以支持多种新的工作流程。我们想在这里重点介绍的是轻松将纹理重复指定次数的能力。这对于创建需要在表面上平铺的纹理非常有用。我们将展示如何使用 Blender 完成此操作，但相同的原理适用于任何 3D 建模软件。

让我们看一个在 Blender 中准备好的示例场景，导出为 GLTF 文件并加载到 Bevy 中。我们首先使用 Blender 中最基本的着色器节点设置：

![Basic shader node setup](https://bevy.org/news/bevy-0-14/basic_nodes.jpg)

结果是在 Bevy 中的以下场景：

![Scene with stretched textures](https://bevy.org/news/bevy-0-14/bevy_no_rep.jpg)

哦不！一切都被拉伸了！这是因为我们设置的 UV 将纹理恰好映射到网格上一次。有几种方法可以处理这个问题，但最方便的是添加缩放纹理的着色器节点，使其重复：

![Repeating shader node setup](https://bevy.org/news/bevy-0-14/rep_nodes.jpg)

`Mapping` 节点的数据就是导出到 `KHR_texture_transform` 的数据。看红色部分。这些缩放因子决定了纹理在材质中应重复多少次。调整所有纹理的这个值可以得到更好的渲染效果：

![Scene with repeated textures](https://bevy.org/news/bevy-0-14/bevy_rep.jpg)

## UI 节点圆角 [#](https://bevy.org/news/bevy-0-14/#ui-node-border-radius)

作者：[chompaa](https://github.com/chompaa)、[pablo-lua](https://github.com/pablo-lua)、[alice-i-cecile](https://github.com/alice-i-cecile)、[bushrat011899](https://github.com/bushrat011899)

PR：[#12500](https://github.com/bevyengine/bevy/pull/12500)

UI 节点的圆角是 Bevy 用户长期要求的功能。现在它终于被支持了！

要对 UI 节点应用圆角，有一个新的组件 [`BorderRadius`](https://docs.rs/bevy/0.14/bevy/prelude/struct.BorderRadius.html)。[`NodeBundle`](https://docs.rs/bevy/0.14/bevy/prelude/struct.NodeBundle.html) 和 [`ButtonBundle`](https://docs.rs/bevy/0.14/bevy/prelude/struct.ButtonBundle.html) 现在有了这个组件的一个字段，叫做 `border_radius`：

```rs
commands.spawn(NodeBundle {
    style: Style {
        width: Val::Px(50.0),
        height: Val::Px(50.0),
        // 毕竟我们需要边框才能圆角嘛！
        border: UiRect::all(Val::Px(5.0)),
        ..default()
    },
    border_color: BorderColor(Color::BLACK),
    // 将所有角应用圆角。或者，你也可以使用 `BorderRadius::all`。
    border_radius: BorderRadius {
        top_left: Val::Px(50.0),
        top_right: Val::Px(50.0),
        bottom_right: Val::Px(50.0),
        bottom_left: Val::Px(50.0),
    },
    ..default()
});
```

There's a [new example](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/ui/rounded_borders.rs) showcasing this new API, a screenshot of which can be seen below:

![rounded_borders example](https://bevy.org/news/bevy-0-14/rounded_borders.jpg)

## Animation Blending with the `AnimationGraph` [#](https://bevy.org/news/bevy-0-14/#animation-blending-with-the-animationgraph)

Authors:[@pcwalton](https://github.com/pcwalton), [@rparrett](https://github.com/rparrett), [@james7132](https://github.com/james7132)

PRs:[#11989](https://github.com/bevyengine/bevy/pull/11989)

Through the eyes of a beginner, handling animation seems simple enough. Define a series of keyframes which transform the various bits of your model to match those poses. We slap some interpolation on there to smoothly move between them, and the user tells you when to start and stop the animation. Easy!

But modern animation pipelines (especially in 3D!) are substantially more complex: animators expect to be able to smoothly blend and programmatically alter different animations dynamically in response to gameplay. In order to capture this richness, the industry has developed the notion of an **animation graph**, which is used to couple the underlying [state machine](https://en.wikipedia.org/wiki/Finite-state_machine) of a game object to the animations that should be playing, and the transitions that should occur between each of the various states.

A player character may be walking, running, slashing a sword, defending with a sword... to create a polished effect, animators need to be able to change between these animations smoothly, change the speed of the walk cycle to match the movement speed along the ground and even perform multiple animations at once!

In Bevy 0.14, we've implemented the [Animation Composition RFC](https://github.com/bevyengine/rfcs/blob/main/rfcs/51-animation-composition.md), providing a low-level API that brings code- and asset-driven animation blending to Bevy.

```rust
#[derive(Resource)]
struct ExampleAnimationGraph(Handle<AnimationGraph>);

fn programmatic_animation_graph(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Create the nodes.
    let mut animation_graph = AnimationGraph::new();
    let blend_node = animation_graph.add_blend(0.5, animation_graph.root);
    animation_graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("models/animated/Fox.glb")),
        1.0,
        animation_graph.root,
    );
    animation_graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(1).from_asset("models/animated/Fox.glb")),
        1.0,
        blend_node,
    );
    animation_graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(2).from_asset("models/animated/Fox.glb")),
        1.0,
        blend_node,
    );

    // Add the graph to our collection of assets.
    let handle = animation_graphs.add(animation_graph);

    // Hold onto the handle
    commands.insert_resource(ExampleAnimationGraph(handle));
}
```

While it can be used to great effect today, most animators will ultimately prefer editing these graphs with a GUI. We plan to build a GUI on top of this API as part of the fabled Bevy Editor. Today, there are also third party solutions like [`bevy_animation_graph`](https://crates.io/crates/bevy_animation_graph).

To learn more and see what the asset-driven approach looks like, take a look at the new [`animation_graph` example](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/animation/animation_graph.rs).

## Improved Color API [#](https://bevy.org/news/bevy-0-14/#improved-color-api)

Authors:[@viridia](https://github.com/viridia), [@mockersf](https://github.com/mockersf)

PRs:[#12013](https://github.com/bevyengine/bevy/pull/12013)

Colors are a huge part of building a good game: UI, effects, shaders and more all need fully-featured, correct and convenient color tools.

Bevy now supports a broad selection of color spaces, each with their own type (e.g. [`LinearRgba`](https://docs.rs/bevy/0.14/bevy/color/struct.LinearRgba.html), [`Hsla`](https://docs.rs/bevy/0.14/bevy/color/struct.Hsla.html), [`Oklaba`](https://docs.rs/bevy/0.14/bevy/color/struct.Oklaba.html)), and offers a wide range of fully documented operations on and conversions between them.

The new API is more error-resistant, more idiomatic and allows us to save work by storing the [`LinearRgba`](https://docs.rs/bevy/0.14/bevy/color/struct.LinearRgba.html) type in our rendering internals. This solid foundation has allowed us to implement a wide range of useful operations, clustered into traits like [`Hue`](https://docs.rs/bevy/0.14/bevy/color/trait.Hue.html) or [`Alpha`](https://docs.rs/bevy/0.14/bevy/color/trait.Alpha.html), allowing you to operate over any color space with the required property. Critically, color mixing / blending is now supported: perfect for procedurally generating color palettes and working with animations.

```rust
use bevy_color::prelude::*;

// Each color space now corresponds to a specific type
let red = Srgba::rgb(1., 0., 0.);

// All non-standard color space conversions are done through the shortest path between
// the source and target color spaces to avoid a quadratic explosion of generated code.
// This conversion...
let red = Oklcha::from(red);
// ...is implemented using
let red = Oklcha::from(Oklaba::from(LinearRgba::from(red)));

// We've added the `tailwind` palette colors: perfect for quick-but-pretty prototyping!
// And the existing CSS palette is now actually consistent with the industry standard :p
let blue = tailwind::BLUE_500;

// The color space that you're mixing your colors in has a huge impact!
// Consider using the scientifically-motivated `Oklcha` or `Oklaba` for a perceptually uniform effect.
let purple = red.mix(blue, 0.5);
```

Most of the user-facing APIs still accept a colorspace-agnostic [`Color`](https://docs.rs/bevy/0.14/bevy/color/enum.Color.html) (which now wraps our color-space types), while rendering internals use the physically-based [`LinearRgba`](https://docs.rs/bevy/0.14/bevy/color/struct.LinearRgba.html) type. For an overview of the different color spaces, and what they're each good for, please check out our [color space usage](https://docs.rs/bevy/0.14/bevy/color/index.html#color-space-usage) documentation.

`bevy_color` offers a solid, type-safe foundation, but it's just getting started. If you'd like another color space or there are more things you'd like to do to your colors, please open an issue or PR and we'd be happy to help!

Also note that `bevy_color` is intended to operate effectively as a stand-alone crate: feel free to take a dependency on it for your non-Bevy projects as well.

## Extruded Shapes [#](https://bevy.org/news/bevy-0-14/#extruded-shapes)

Authors:[@lynn-lumen](https://github.com/lynn-lumen)

PRs:[#13270](https://github.com/bevyengine/bevy/pull/13270)

**Bevy 0.14** introduces an entirely new group of primitives: extrusions!

An extrusion is a 2D primitive (the base shape) that is _extruded_ into a third dimension by some depth. The resulting shape is a prism (or in the special case of the circle, a cylinder).

```rust
// Create an ellipse with width 2 and height 1.
let my_ellipse = Ellipse::from_size(2.0, 1.0);

// Create an extrusion of this ellipse with a depth of 1.
let my_extrusion = Extrusion::new(my_ellipse, 1.);
```

All extrusions are extruded along the Z-axis. This guarantees that an extrusion of depth 0 and the corresponding base shape are identical, just as one would expect.

#### Measuring and Sampling

Since all extrusions with base shapes that implement [`Measured2d`](https://docs.rs/bevy/0.14/bevy/math/prelude/trait.Measured2d.html) implement [`Measured3d`](https://docs.rs/bevy/0.14/bevy/math/prelude/trait.Measured3d.html), you can easily get the surface area or volume of an extrusion. If you have an extrusion of a custom 2D primitive, you can simply implement [`Measured2d`](https://docs.rs/bevy/0.14/bevy/math/prelude/trait.Measured2d.html) for your primitive and [`Measured3d`](https://docs.rs/bevy/0.14/bevy/math/prelude/trait.Measured3d.html) will be implemented automatically for the extrusion.

Likewise, you can sample the boundary and interior of any extrusion if the base shape of the extrusion implements [`ShapeSample<Output = Vec2>`](https://docs.rs/bevy/0.14/bevy/math/trait.ShapeSample.html) and [`Measured2d`](https://docs.rs/bevy/0.14/bevy/math/prelude/trait.Measured2d.html).

```rust
// Create a 2D capsule with radius 1 and length 2, extruded to a depth of 3
let extrusion = Extrusion::new(Capsule2d::new(1.0, 2.0), 3.0);

// Get the volume of the extrusion
let volume = extrusion.volume();

// Get the surface area of the extrusion
let surface_area = extrusion.area();

// Create a random number generator
let mut rng = StdRng::seed_from_u64(4);

// Sample a random point inside the extrusion
let interior_sample = extrusion.sample_interior(&mut rng);

// Sample a random point on the surface of the extrusion
let boundary_sample = extrusion.sample_boundary(&mut rng);
```

#### Bounding

You can also get bounding spheres and Axis Aligned Bounding Boxes (AABBs) for extrusions. If you have a custom 2D primitive that implements [`Bounded2d`](https://docs.rs/bevy/0.14/bevy/math/bounding/trait.Bounded2d.html), you can simply implement [`BoundedExtrusion`](https://docs.rs/bevy/0.14/bevy/math/bounding/trait.BoundedExtrusion.html)) for your primitive. The default implementation will give optimal results but may be slower than a solution fitted to your primitive.

#### Meshing

Extrusions do not exist in the world of maths only though. They can also be meshed and displayed on the screen!

And again, adding meshing support for your own primitives is made easy by Bevy! You simply need to implement meshing for your 2D primitive and then implement [`Extrudable`](https://docs.rs/bevy/0.14/bevy/render/mesh/trait.Extrudable.html) for your 2D primitive's [`MeshBuilder`](https://docs.rs/bevy/0.14/bevy/prelude/trait.MeshBuilder.html).

When implementing [`Extrudable`](https://docs.rs/bevy/0.14/bevy/render/mesh/trait.Extrudable.html), you have to provide information about whether segments of the perimeter of the base shape are to be shaded smooth or flat, and what vertices belong to each of these perimeter segments.

![a 2D heart primitive and its extrusion](https://bevy.org/news/bevy-0-14/heart_extrusion.jpg)

The [`Extrudable`](https://docs.rs/bevy/0.14/bevy/render/mesh/trait.Extrudable.html) trait allows you to easily implement meshing for extrusions of custom primitives. Of course, you could also implement meshing manually for your extrusion.

If you want to see a full implementation of this, you can check out the [custom primitives example](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/math/custom_primitives.rs).

## More Gizmos [#](https://bevy.org/news/bevy-0-14/#more-gizmos)

Authors:[@mweatherley](https://github.com/mweatherley), [@Kanabenki](https://github.com/Kanabenki), [@MrGVSV](https://github.com/MrGVSV), [@solis-lumine-vorago](https://github.com/solis-lumine-vorago), [@alice-i-cecile](https://github.com/alice-i-cecile)

PRs:[#12211](https://github.com/bevyengine/bevy/pull/12211)

Gizmos in Bevy allow developers to easily draw arbitrary shapes to help debugging or authoring content, but also to visualize specific properties of your scene, such has the AABB of your meshes.

In 0.14, several new gizmos have been added to [`bevy::gizmos`](https://docs.rs/bevy/0.14.0/bevy/gizmos/index.html):

#### Rounded box gizmos

Rounded boxes and cubes are great for visualizing regions and colliders.

If you set the `corner_radius` or `edge_radius` to a positive value, the corners will be rounded outwards. However, if you provide a negative value, the corners will flip and curve inwards.

![rounded gizmos cuboids](https://bevy.org/news/bevy-0-14/gizmos_rounded_cuboid.jpg) ![rounded gizmos rectangles](https://bevy.org/news/bevy-0-14/gizmos_rounded_rect.jpg)

#### Grid Gizmos

New grid gizmo types were added with [`Gizmos::grid_2d`](https://docs.rs/bevy/0.14.0/bevy/gizmos/prelude/struct.Gizmos.html#method.grid_2d) and [`Gizmos::grid`](https://docs.rs/bevy/0.14.0/bevy/gizmos/prelude/struct.Gizmos.html#method.grid) to draw a plane grid in either 2D or 3D, alongside [`Gizmos::grid_3d`](https://docs.rs/bevy/0.14.0/bevy/gizmos/prelude/struct.Gizmos.html#method.grid_3d) to draw a 3D grid.

Each grid type can be skewed, scaled and subdivided along its axis, and you can separately control which outer edges to draw.

![Grid gizmos screenshot](https://bevy.org/news/bevy-0-14/grid_gizmos.jpg)

#### Coordinate Axes Gizmo

The new [`Gizmos::axes`](https://docs.rs/bevy/0.14.0/bevy/gizmos/prelude/struct.Gizmos.html#method.axes) add a simple way to show the position, orientation and scale of any object from its [`Transform`](https://docs.rs/bevy/0.14.0/bevy/prelude/struct.Transform.html) plus a base size. The size of each axis arrow is proportional to the corresponding axis scale in the provided [`Transform`](https://docs.rs/bevy/0.14.0/bevy/prelude/struct.Transform.html).

![Axes gizmo screenshot](https://bevy.org/news/bevy-0-14/axes_gizmo.jpg)

#### Light Gizmos

The new [`ShowLightGizmo`](https://docs.rs/bevy/0.14.0/bevy/gizmos/light/struct.ShowLightGizmo.html) component implements a retained gizmo to visualize lights for [`SpotLight`](https://docs.rs/bevy/0.14.0/bevy/pbr/struct.SpotLight.html), [`PointLight`](https://docs.rs/bevy/0.14.0/bevy/pbr/struct.PointLight.html) and [`DirectionalLight`](https://docs.rs/bevy/0.14.0/bevy/pbr/struct.DirectionalLight.html). Most light properties are visually represented by the gizmos, and the gizmo color can be set to match the light instance or use a variety of other behaviors.

Similar to other retained gizmos, [`ShowLightGizmo`](https://docs.rs/bevy/0.14.0/bevy/gizmos/light/struct.ShowLightGizmo.html) can be configured per-instance or globally with [`LightGizmoConfigGroup`](https://docs.rs/bevy/0.14.0/bevy/gizmos/light/struct.LightGizmoConfigGroup.html).

![Light gizmos screenshot](https://bevy.org/news/bevy-0-14/light_gizmos.jpg)

## Gizmo Line Styles and Joints [#](https://bevy.org/news/bevy-0-14/#gizmo-line-styles-and-joints)

Authors:[@lynn-lumen](https://github.com/lynn-lumen)

PRs:[#12394](https://github.com/bevyengine/bevy/pull/12394)

Previous versions of Bevy supported drawing line gizmos:

```rust
fn draw_gizmos(mut gizmos: Gizmos) {
    gizmos.line_2d(Vec2::ZERO, Vec2::splat(-80.), RED);
}
```

However the only way to customize gizmos was to change their color, which may be limiting for some use cases. Additionally, the meeting points of two lines in a line strip, their _joints_, had little gaps.

As of Bevy 0.14, you can change the style of the lines and their joints for each gizmo config group:

```rust
fn draw_gizmos(mut gizmos: Gizmos) {
    gizmos.line_2d(Vec2::ZERO, Vec2::splat(-80.), RED);
}

fn setup(mut config_store: ResMut<GizmoConfigStore>) {
    // Get the config for you gizmo config group
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    // Set the line style and joints for this config group
    config.line_style = GizmoLineStyle::Dotted;
    config.line_joints = GizmoLineJoint::Bevel;
}
```

The new line styles can be used in both 2D and 3D and respect the `line_perspective` option of their config groups.

Available line styles are:

- `GizmoLineStyle::Dotted`: draws a dotted line with each dot being a square
- `GizmoLineStyle::Solid`: draws a solid line - this is the default behavior and the only one available before Bevy 0.14

![new gizmos line styles](https://bevy.org/news/bevy-0-14/gizmos_line_styles.jpg)

Similarly, the new line joints offer a variety of options:

- `GizmoLineJoint::Miter`, which extends both lines until they meet at a common miter point,
- `GizmoLineJoint::Round(resolution)`, which will approximate an arc filling the gap between the two lines. The `resolution` determines the amount of triangles used to approximate the geometry of the arc.
- `GizmoLineJoint::Bevel`, which connects the ends of the two joining lines with a straight segment, and
- `GizmoLineJoint::None`, which uses no joints and leaves small gaps - this is the default behavior and the only one available before Bevy 0.14.

![new gizmos line joints](https://bevy.org/news/bevy-0-14/gizmos_line_joints.jpg)

You can check out the [2D gizmos example](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/gizmos/2d_gizmos.rs), which demonstrates the use of line styles and joints!

## UI Node Outline Gizmos [#](https://bevy.org/news/bevy-0-14/#ui-node-outline-gizmos)

Authors:[@pablo-lua](https://github.com/pablo-lua), [@nicopap](https://github.com/nicopap), [@alice-i-cecile](https://github.com/alice-i-cecile)

PRs:[#11237](https://github.com/bevyengine/bevy/pull/11237)

When working with UI on the web, being able to quickly debug the size of all your boxes is wildly useful. We now have a native [layout tool](https://docs.rs/bevy/0.14/bevy/dev_tools/ui_debug_overlay/struct.DebugUiPlugin.html) which adds gizmos outlines to all [Nodes](https://docs.rs/bevy/0.14/bevy/ui/struct.Node.html)

An example of what the tool looks like after enabled

![Ui example with the overlay tool enabled](https://bevy.org/news/bevy-0-14/bevy_ui_outlines.jpg)

```rust
use bevy::prelude::*;

// You first have to add the DebugUiPlugin to your app
let mut app = App::new()
    .add_plugins(bevy::dev_tools::ui_debug_overlay::DebugUiPlugin);

// In order to enable the tool at runtime, you can add a system to toggle it
fn toggle_overlay(
    input: Res<ButtonInput<KeyCode>>,
    mut options: ResMut<bevy::dev_tools::ui_debug_overlay::UiDebugOptions>,
) {
    info_once!("The debug outlines are enabled, press Space to turn them on/off");
    if input.just_pressed(KeyCode::Space) {
        // The toggle method will enable the debug_overlay if disabled and disable if enabled
        options.toggle();
    }

}

// And add the system to the app
app.add_systems(Update, toggle_overlay);
```

## Contextually Clearing Gizmos [#](https://bevy.org/news/bevy-0-14/#contextually-clearing-gizmos)

Authors:[@Aceeri](https://github.com/Aceeri)

PRs:[#10973](https://github.com/bevyengine/bevy/pull/10973)

Gizmos are drawn via an immediate mode API. This means that every **update** you draw all gizmos you want to display, and only those will be shown. Previously, update referred to "once every time the `Main` schedule runs". This matches the frame rate, so it usually works great! But when you try to draw gizmos during `FixedMain`, they will flicker or be rendered multiple times. In Bevy 0.14, this now just works!

This can be extended for use with custom schedules. Instead of a single storage, there now are multiple [storages](https://docs.rs/bevy/0.14/bevy/gizmos/gizmos/struct.GizmoStorage.html) differentiated by a context type parameter. You can also set a type parameter on the [`Gizmos`](https://docs.rs/bevy/0.14/bevy/gizmos/gizmos/struct.Gizmos.html) system param to choose what storage to write to. You choose when storages you add get drawn or cleared: Any gizmos in the default storage (the `()` context) during the `Last` schedule will be shown.

## Query Joins [#](https://bevy.org/news/bevy-0-14/#query-joins)

Authors:[@hymm](https://github.com/hymm)

PRs:[#11535](https://github.com/bevyengine/bevy/pull/11535)

ECS Queries can now be combined, returning the data for entities that are contained in both queries.

```rust
fn helper_function(a: &mut Query<&A>, b: &mut Query<&B>){    
    let a_and_b: QueryLens<(Entity, &A, &B)> = a.join(b);
    assert!(a_and_b.iter().len() <= a.len());
    assert!(a_and_b.iter().len() <= b.len());
}
```

In most cases, you should continue to simply add more parameters to your original query. `Query<&A, &B>` will generally be clearer than joining them later. But when a complex system or helper function backs you into a corner, query joins are there if you need them.

If you're familiar with database terminology, this is an ["inner join"](https://www.w3schools.com/sql/sql_join.asp). Other types of query joins are being considered. Maybe you could take a crack at the [follow-up issue](https://github.com/bevyengine/bevy/issues/13633)?

## Computed States & Sub-States [#](https://bevy.org/news/bevy-0-14/#computed-states-sub-states)

Authors:[@lee-orr](https://github.com/lee-orr), [@marcelchampagne](https://github.com/marcelchampagne), [@MiniaczQ](https://github.com/MiniaczQ), [@alice-i-cecile](https://github.com/alice-i-cecile)

PRs:[#11426](https://github.com/bevyengine/bevy/pull/11426)

Bevy's [`States`](https://docs.rs/bevy/0.14/bevy/prelude/trait.States.html) are a simple but powerful abstraction for managing the control flow of your app.

But as users' games (and non-game applications!) grew in complexity, their limitations became more apparent. What happens if we want to capture the notion of "in a menu", but then have different states corresponding to which submenu should be open? What if we want to ask questions like "is the game paused", but that question only makes sense while we're within a game?

Finding a good abstraction for this required [several](https://github.com/bevyengine/bevy/pull/9957) [attempts](https://github.com/bevyengine/bevy/pull/10088) and a great deal of both experimentation and discussion.

While your existing [`States`](https://docs.rs/bevy/0.14/bevy/prelude/trait.States.html) code will work exactly as before, there are now two additional tools you can reach for if you're looking for more expressiveness: **computed states** and **sub states**.

Let's begin with a simple state declaration:

```rust
#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
enum GameState {
    #[default]
    Menu,
    InGame {
        paused: bool
    },
}
```

The addition of `pause` field means that simply checking for `GameState::InGame` doesn't work ... the states are different depending on its value and we may want to distinguish between game systems that run when the game is paused or not!

#### Computed States

While we can simply do `OnEnter(GameState::InGame{paused: true})`, we need to be able to reason about "while we're in the game, paused or not". To this end, we define the `InGame` computed state:

```rust
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct InGame;

impl ComputedStates for InGame {
    // Computed states can be calculated from one or many source states.
    type SourceStates = GameState;

    // Now, we define the rule that determines the value of our computed state.
    fn compute(sources: GameState) -> Option<InGame> {
        match sources {
            // We can use pattern matching to express the
            //"I don't care whether or not the game is paused" logic!
            GameState::InGame {..} => Some(InGame),
            _ => None,
        }
    }
}
```

#### Sub-States

In contrast, sub-states should be used when you want to keep manual control over the value through `NextState`, but still bind their existence to some parent state.

```rust
#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
// This macro means that `GamePhase` will only exist when we're in the `InGame` computed state.
// The intermediate computed state is helpful for clarity here, but isn't required:
// you can manually `impl SubStates` for more control, multiple parent states and non-default initial value!
#[source(InGame = InGame)]
enum GamePhase {
    #[default]
    Setup,
    Battle,
    Conclusion
}
```

#### Initialization

Initializing our states is easy: just call the appropriate method on `App` and all of the required machinery will be set up for you.

```rust
App::new()
   .init_state::<GameState>()
   .add_computed_state::<InGame>()
   .add_sub_state::<GamePhase>()
```

Just like any other state, computed states and substates work with all of the tools you're used to: the `State` and `NextState` resources, `OnEnter`, `OnExit` and `OnTransition` schedules and the `in_state` run condition. Make sure to visit [both](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/state/computed_states.rs) [examples](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/state/sub_states.rs) for more information!

The only exception is that, for correctness, computed states _cannot_ be mutated through `NextState`. Instead, they are strictly derived from their parent states; added, removed and updated automatically during state transitions based on the provided `compute` method.

All of Bevy's state tools are now found in a dedicated `bevy_state` crate, which can be controlled via a feature flag. Yearning for the days of state stacks? Wish that there was a method for re-entering states? All of the state machinery relies _only_ on public ECS tools: resources, schedules, and run conditions, making it easy to build on top of. We know that state machines are very much a matter of taste; so if our design isn't to your taste consider taking advantage of Bevy's modularity and writing your own abstraction or using one supplied by the community!

## State Scoped Entities [#](https://bevy.org/news/bevy-0-14/#state-scoped-entities)

Authors:[@MiniaczQ](https://github.com/MiniaczQ), [@alice-i-cecile](https://github.com/alice-i-cecile), [@mockersf](https://github.com/mockersf)

PRs:[#13649](https://github.com/bevyengine/bevy/pull/13649)

State scoped entities is a pattern that naturally emerged in community projects. **Bevy 0.14** has embraced it!

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum GameState {
 #[default]
  Menu,
  InGame,
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        // We mark this entity with the `StateScoped` component.
        // When the provided state is exited, the entity will be
        // deleted recursively with all children.
        StateScoped(GameState::InGame)
        SpriteBundle { ... }
    ))
}

App::new()
    .init_state::<GameState>()
    // We need to install the appropriate machinery for the cleanup code
    // to run, once for each state type.
    .enable_state_scoped_entities::<GameState>()
    .add_systems(OnEnter(GameState::InGame), spawn_player);
```

By binding entity lifetime to a state during setup, we can dramatically reduce the amount of cleanup code we have to write!

## State Identity Transitions [#](https://bevy.org/news/bevy-0-14/#state-identity-transitions)

Authors:[@MiniaczQ](https://github.com/MiniaczQ), [@alice-i-cecile](https://github.com/alice-i-cecile)

PRs:[#13579](https://github.com/bevyengine/bevy/pull/13579)

Users have sometimes asked for us to trigger exit and entry steps when moving from a state to itself. While this has its uses (refreshing is the core idea), it can be surprising and unwanted in other cases. We've found a compromise that lets users hook into this type of transition if it's something they need.

`StateEventTransition` events will now include transitions from a state to itself, which will also propagate to all dependent `ComputedStates` and `SubStates`.

Because it is a niche feature, `OnExit` and `OnEnter` schedules will ignore the new identity transitions by default, but you can visit the new [`custom_transitions`](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/state/custom_transitions.rs) example to see how you can bypass or change that behavior!

## GPU Frustum Culling [#](https://bevy.org/news/bevy-0-14/#gpu-frustum-culling)

Authors:[@pcwalton](https://github.com/pcwalton)

PRs:[#12889](https://github.com/bevyengine/bevy/pull/12889)

Bevy's rendering stack is often CPU-bound: by shifting more work onto the GPU, we can better balance the load and render more shiny things faster. Frustum culling is an optimization technique that automatically hides objects that are outside of a camera's view (its frustum). In Bevy 0.14, users can choose to have this work performed on the GPU, depending on the performance characteristics of their project.

Two new components are available to control frustum culling: `GpuCulling` and `NoCpuCulling`. Attach the appropriate combination of these components to a camera, and you're set.

```rust
commands.spawn((
    Camera3dBundle::default(),
    // Enable GPU frustum culling (does not automatically disable CPU frustum culling).
    GpuCulling,
    // Disable CPU frustum culling.
    NoCpuCulling
));
```

## World Command Queue [#](https://bevy.org/news/bevy-0-14/#world-command-queue)

Authors:[@james7132](https://github.com/james7132), [@james-j-obrien](https://github.com/james-j-obrien)

PRs:[#11823](https://github.com/bevyengine/bevy/pull/11823)

Working with [`Commands`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Commands.html) when you have exclusive world access has always been a pain. Create a [`CommandQueue`](https://docs.rs/bevy/0.14/bevy/ecs/world/struct.CommandQueue.html), generate a [`Commands`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Commands.html) out of that, send your commands and then apply it? Not exactly the most intuitive solution.

Now, you can access the [`World`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.World.html)'s own command queue:

```rust
let mut world = World::new();
let mut commands = world.commands();
commands.spawn(TestComponent);
world.flush_commands();
```

While this isn't the most performant approach (just apply the mutations directly to the world and skip the indirection), this API can be great for quickly prototyping with or easily testing your custom commands. It is also used internally to power component lifecycle hooks and observers.

As a bonus, one-shot systems now apply their commands (and other deferred system params) immediately when run! We already have exclusive world access: why introduce delays and subtle bugs?

## Reduced Multi-Threaded Execution Overhead [#](https://bevy.org/news/bevy-0-14/#reduced-multi-threaded-execution-overhead)

Authors:[@chescock](https://github.com/chescock), [@james7132](https://github.com/james7132)

PRs:[#11906](https://github.com/bevyengine/bevy/pull/11906)

The largest source of overhead in Bevy's multithreaded system executor is from [thread context switching](https://en.wikipedia.org/wiki/Context_switch), i.e. starting and stopping threads. Each time a thread is woken up it can take up to 30us if the cache for the thread is cold. Minimizing these switches is an important optimization for the executor. In this cycle we landed two changes that show improvements for this:

#### Run the multi-threaded executor at the end of each system task

The system executor is responsible for checking that the dependencies for a system have run already and evaluating the run criteria and then running a task for that system. The old version of the multithreaded executor ran as a separate task that was woken up after each task completed. This would sometimes cause a new thread to be woken up for the executor to process the system completing.

By changing it so the system task tries to run the multithreaded executor after each system completes, we ensure that the multithreaded executor always runs on a thread that is already awake. This prevents one source of context switches. In practice this reduces the number of context switches per a `Schedule` run by 1-3 times, for an improvement of around 30us per schedule. When an app has many schedules, this can add up!

#### Combined Event update system

There used to be one instance of the "event update system" for each event type. With just the `DefaultPlugins`, that results in 20+ instances of the system.

Each instance ran very quick, so the overhead of spawning the system tasks and waking up threads to run all these systems dominated the time it took for the `First` schedule to run. So combining all these into one system avoids this overhead and makes the `First` schedule run much faster. In testing this made running the schedule go from 140us to 25us. Again, not a _huge_ win, but we're all about saving every microsecond we can!

## Decouple `BackgroundColor` from `UiImage` [#](https://bevy.org/news/bevy-0-14/#decouple-backgroundcolor-from-uiimage)

Authors:[@benfrankel](https://github.com/benfrankel)

PRs:[#11165](https://github.com/bevyengine/bevy/pull/11165)

UI images can now be given solid background colors:

![UI image with background color](https://bevy.org/news/bevy-0-14/ui_image_background_color.jpg)

The [`BackgroundColor`](https://docs.rs/bevy/0.14/bevy/prelude/struct.BackgroundColor.html) component now works for UI images instead of applying a color tint on the image itself. You can still apply a color tint by setting `UiImage::color`. For example:

```rust
commands.spawn((
    ImageBundle {
        image: UiImage {
            handle: assets.load("logo.png"),
            color: DARK_RED.into(),
            ..default()
        },
        ..default()
    },
    BackgroundColor(ANTIQUE_WHITE.into()),
    Outline::new(Val::Px(8.0), Val::ZERO, CRIMSON.into()),
));
```

## Combined WinitEvent [#](https://bevy.org/news/bevy-0-14/#combined-winitevent)

Authors:[@UkoeHB](https://github.com/UkoeHB)

PRs:[#12100](https://github.com/bevyengine/bevy/pull/12100)

When handling inputs, the exact ordering of the events received is often very significant, even when the events are not the same type! Consider a simple drag-and-drop operation. When, exactly, did the user release the mouse button relative to the many tiny movements that they performed? Getting these details right goes a long way to a responsive, precise user experience.

We now expose the blanket [`WinitEvent`](https://docs.rs/bevy/0.14/bevy/winit/enum.WinitEvent.html) event stream, in addition to the existing separated event streams, which can be read and matched on directly whenever these problems arise.

## Recursive Reflect Registration [#](https://bevy.org/news/bevy-0-14/#recursive-reflect-registration)

Authors:[@MrGVSV](https://github.com/MrGVSV), [@soqb](https://github.com/soqb), [@cart](https://github.com/cart), [@james7132](https://github.com/james7132)

PRs:[#5781](https://github.com/bevyengine/bevy/pull/5781)

Bevy uses [reflection](https://docs.rs/bevy_reflect/latest/bevy_reflect/) in order to dynamically process data for things like serialization and deserialization. A Bevy app has a `TypeRegistry` to keep track of which types exist. Users can register their custom types when initializing the app or plugin.

```rust
#[derive(Reflect)]
struct Data<T> {
  value: T,
}

#[derive(Reflect)]
struct Blob {
  contents: Vec<u8>,
}

app
  .register_type::<Data<Blob>>()
  .register_type::<Blob>()
  .register_type::<Vec<u8>>()
```

In the code above, `Data<Blob>` depends on `Blob` which depends on `Vec<u8>`, which means that all three types need to be manually registered— even if we only care about `Data<Blob>`.

This is both tedious and error-prone, especially when these type dependencies are only used in the context of other types (i.e. they aren't used as standalone types).

In 0.14, any type that derives `Reflect` will automatically register all of its type dependencies. So when we register `Data<Blob>`, `Blob` will be registered as well (which will register `Vec<u8>`), thus simplifying our registration down to a single line:

```rust
app.register_type::<Data<Blob>>()
```

Note that removing the registration for `Data<Blob>` now also means that `Blob` and `Vec<u8>` may not be registered either, unless they were registered some other way. If those types are needed as standalone types, they should be registered separately.

## `Rot2` Type for 2D Rotations [#](https://bevy.org/news/bevy-0-14/#rot2-type-for-2d-rotations)

Authors:[@Jondolf](https://github.com/Jondolf), [@IQuick143](https://github.com/IQuick143), [@tguichaoua](https://github.com/tguichaoua)

PRs:[#11658](https://github.com/bevyengine/bevy/pull/11658)

Ever wanted to work with rotations in 2D and get frustrated with having to choose between quaternions and a raw `f32`? Us too!

We've added a convenient [`Rot2`](https://docs.rs/bevy/0.14/bevy/math/struct.Rot2.html) type for you, with plenty of helper methods. Feel free to replace that helper type you wrote, and submit little PRs for any useful functionality we're missing.

[`Rot2`](https://docs.rs/bevy/0.14/bevy/math/struct.Rot2.html) is a great complement to the [`Dir2`](https://docs.rs/bevy/0.14/bevy/math/struct.Dir2.html) type (formerly `Direction2d`). The former represents an angle, while the latter is a unit vector. These types are similar but not interchangeable, and the choice of representation depends heavily on the task at hand. You can rotate a direction using `direction = rotation * Dir2::X`. To recover the rotation, use `Dir2::X::rotation_to(direction)` or in this case the helper `Dir2::rotation_from_x(direction)`.

While these types aren't used widely within the engine yet, we _are_ aware of your pain and are evaluating [proposals](https://github.com/bevyengine/rfcs/pull/82) for how we can make working with transforms in 2D more straightforward and pleasant.

## Alignment API for Transforms [#](https://bevy.org/news/bevy-0-14/#alignment-api-for-transforms)

Authors:[@mweatherley](https://github.com/mweatherley)

PRs:[#12187](https://github.com/bevyengine/bevy/pull/12187)

**Bevy 0.14** adds a new [`Transform::align`](https://docs.rs/bevy/0.14/bevy/transform/components/struct.Transform.html#method.align) function, which is a more general form of [`Transform::look_to`](https://docs.rs/bevy/0.14/bevy/transform/components/struct.Transform.html#method.look_to), which allows you to specify any local axis you want to use for the main and secondary axes.

This allows you to do things like point the front of a spaceship at a planet you're heading toward while keeping the right wing pointed in the direction of another ship. or point the top of a ship in the direction of the tractor beam pulling it in, while the front rotates to match the bigger ship's direction.

Lets consider a ship where we're going to use the front of the ship and the right wing as local axes:

![before calling Transform::align](https://bevy.org/news/bevy-0-14/align-before-move.jpg)

```rust
// point the local negative-z axis in the global Y axis direction
// point the local x-axis in the global Z axis direction
transform.align(Vec3::NEG_Z, Vec3::Y, Vec3::X, Vec3::Z)
```

`align` will move it to match the desired positions as closely as possible:

![after calling Transform::align](https://bevy.org/news/bevy-0-14/align-after-move.jpg)

Note that not all rotations can be constructed and the [documentation](https://docs.rs/bevy/0.14/bevy/transform/components/struct.Transform.html#method.align) explains what happens in such scenarios.

## Random Sampling of Shapes and Directions [#](https://bevy.org/news/bevy-0-14/#random-sampling-of-shapes-and-directions)

Authors:[@13ros27](https://github.com/13ros27), [@mweatherley](https://github.com/mweatherley), [@lynn-lumen](https://github.com/lynn-lumen)

PRs:[#12484](https://github.com/bevyengine/bevy/pull/12484)

In the context of game development, it's often helpful to have access to random values, whether that's in the interest of driving behavior for NPCs, creating effects, or just trying to create variety. To help support this, a few random sampling features have been added to `bevy_math`, gated behind the `rand` feature. These are primarily geometric in nature, and they come in a couple of flavors.

First, one can sample random points from the boundaries and interiors of a variety of mathematical primitives: ![Image of several primitives side-by-side with points randomly sampled from their interiors](https://bevy.org/news/bevy-0-14/sampling_primitives.jpg)

In code, this can be performed in a couple different ways, using either the `sample_interior`/`sample_boundary` or `interior_dist`/`boundary_dist` APIs:

```rust
use bevy::math::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

let sphere = Sphere::new(1.5);

// Instantiate an Rng:
let rng = &mut ChaCha8Rng::seed_from_u64(7355608);

// Using these, sample a random point from the interior of this sphere:
let interior_pt: Vec3 = sphere.sample_interior(rng);
// or from the boundary:
let boundary_pt: Vec3 = sphere.sample_boundary(rng);

// Or, if we want a lot of points, we can use a Distribution instead...
// to sample 100000 random points from the interior:
let interior_pts: Vec<Vec3> = sphere.interior_dist().sample_iter(rng).take(100000).collect();
// or 100000 random points from the boundary:
let boundary_pts: Vec<Vec3> = sphere.boundary_dist().sample_iter(rng).take(100000).collect();
```

Note that these methods explicitly require an [`Rng`](https://docs.rs/rand/0.8.5/rand/trait.Rng.html) object, giving you control over the randomization strategy and seed.

The currently supported shapes are as follows:

2D: `Circle`, `Rectangle`, `Triangle2d`, `Annulus`, `Capsule2d`.

3D: `Sphere`, `Cuboid`, `Triangle3d`, `Tetrahedron`, `Cylinder`, `Capsule3d`, and extrusions of sampleable 2D shapes (`Extrusion`).

---

Similarly, the direction types (`Dir2`, `Dir3`, `Dir3A`) and quaternions (`Quat`) can now be constructed randomly using `from_rng`:

```rust
use bevy::math::prelude::*;
use rand::{random, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

// Instantiate an Rng:
let rng = &mut ChaCha8Rng::seed_from_u64(7355608);

// Get a random direction:
let direction = Dir3::from_rng(rng);

// Similar, but requires left-hand type annotations or inference:
let another_direction: Dir3 = rng.gen();

// Using `random` to grab a value using implicit thread-local rng:
let yet_another_direction: Dir3 = random();
```

## Tools for Profiling GPU Performance [#](https://bevy.org/news/bevy-0-14/#tools-for-profiling-gpu-performance)

Authors:[@LeshaInc](https://github.com/LeshaInc)

PRs:[#9135](https://github.com/bevyengine/bevy/pull/9135)

While [Tracy](https://github.com/bevyengine/bevy/blob/main/docs/profiling.md) already lets us measure CPU time per system, our GPU diagnostics are much weaker. In Bevy 0.14 we've added support for two classes of rendering-focused statistics via the [`RenderDiagnosticsPlugin`](https://docs.rs/bevy/0.14/bevy/render/diagnostic/struct.RenderDiagnosticsPlugin.html):

1. **Timestamp queries:** how long did specific bits of work take on the GPU?
2. **Pipeline statistics:** information about the quantity of work sent to the GPU.

While it may sound like timestamp queries are the ultimate diagnostic tool, they come with several caveats. Firstly, they vary quite heavily from frame-to-frame as GPUs dynamically ramp up and down clock speed due to workload (idle gaps in GPU work, e.g., a bunch of consecutive barriers, or the tail end of a large dispatch) or the physical temperature of the GPU. To get an accurate measurement, you need to look at summary statistics: mean, median, 75th percentile and so on.

Secondly, while timestamp queries will tell you how long something takes, but it will not tell you why things are slow. For finding bottlenecks, you want to use a GPU profiler from your GPU vendor (Nvidia's NSight, AMD's RGP, Intel's GPA or Apple's XCode). These tools will give you much more detailed stats about cache hit rate, warp occupancy, and so on. On the other hand they lock your GPU's clock to base speeds for stable results, so they won't give you a good indicator of real world performance.

[`RenderDiagnosticsPlugin`](https://docs.rs/bevy/0.14/bevy/render/diagnostic/struct.RenderDiagnosticsPlugin.html) tracks the following pipeline statistics, recorded in Bevy's [`DiagnosticsStore`](https://docs.rs/bevy/0.14/bevy/diagnostic/struct.DiagnosticsStore.html): Elapsed CPU time, Elapsed GPU time, [Vertex shader](https://www.khronos.org/opengl/wiki/Vertex_Shader) invocations, [Fragment shader](https://www.khronos.org/opengl/wiki/Fragment_Shader) invocations, [Compute shader](https://www.khronos.org/opengl/wiki/Compute_Shader) invocations, [Clipper invocations](http://gpa.helpmax.net/en/intel-graphics-performance-analyzers-help/metrics-descriptions/extended-metrics-description/rasterizer-metrics/clipper-invocations/), and [Clipper primitives](http://gpa.helpmax.net/en/intel-graphics-performance-analyzers-help/metrics-descriptions/extended-metrics-description/rasterizer-metrics/post-clip-primitives/).

You can also track individual render/compute passes, groups of passes (e.g. all shadow passes), and individual commands inside passes (like draw calls). To do so, instrument them using methods from the [`RecordDiagnostics`](https://docs.rs/bevy/0.14/bevy/render/diagnostic/trait.RecordDiagnostics.html) trait.

## New Geometric Primitives [#](https://bevy.org/news/bevy-0-14/#new-geometric-primitives)

Authors:[@vitorfhc](https://github.com/vitorfhc), [@Chubercik](https://github.com/Chubercik), [@andristarr](https://github.com/andristarr), [@spectria-limina](https://github.com/spectria-limina), [@salvadorcarvalhinho](https://github.com/salvadorcarvalhinho), [@aristaeus](https://github.com/aristaeus), [@mweatherley](https://github.com/mweatherley)

PRs:[#12508](https://github.com/bevyengine/bevy/pull/12508)

Geometric shapes find a variety of applications in game development, ranging from rendering simple items to the screen for display / debugging to use in colliders, physics, raycasting, and more.

For this, geometric shape primitives were [introduced in Bevy 0.13](https://bevy.org/news/bevy-0-13/#primitive-shapes), and work on this area has continued with Bevy 0.14, which brings the addition of [`Triangle3d`](https://docs.rs/bevy/0.14.0/bevy/math/primitives/struct.Triangle3d.html) and [`Tetrahedron`](https://docs.rs/bevy/0.14.0/bevy/math/primitives/struct.Tetrahedron.html) 3D primitives, along with [`Rhombus`](https://docs.rs/bevy/0.14.0/bevy/math/primitives/struct.Rhombus.html), [`Annulus`](https://docs.rs/bevy/0.14.0/bevy/math/primitives/struct.Annulus.html), [`Arc2d`](https://docs.rs/bevy/0.14.0/bevy/math/primitives/struct.Arc2d.html), [`CircularSegment`](https://docs.rs/bevy/0.14.0/bevy/math/primitives/struct.CircularSegment.html), and [`CircularSector`](https://docs.rs/bevy/0.14.0/bevy/math/primitives/struct.CircularSector.html) 2D primitives. As usual, these each have methods for querying geometric information like perimeter, area, and volume, and they all support meshing (where applicable) as well as gizmo display.

## Improve `Point` and rename it to `VectorSpace` [#](https://bevy.org/news/bevy-0-14/#improve-point-and-rename-it-to-vectorspace)

Authors:[@mweatherley](https://github.com/mweatherley), [@bushrat011899](https://github.com/bushrat011899), [@JohnTheCoolingFan](https://github.com/JohnTheCoolingFan), [@NthTensor](https://github.com/NthTensor), [@IQuick143](https://github.com/IQuick143), [@alice-i-cecile](https://github.com/alice-i-cecile)

PRs:[#12747](https://github.com/bevyengine/bevy/pull/12747)

Linear algebra is used everywhere in games, and we want to make sure it's easy to get right. That's why we've added a new `VectorSpace` trait, as part of our work to make `bevy_math` more general, expressive, and mathematically sound. Anything that implements `VectorSpace` behaves like a vector. More formally, the trait requires that implementations satisfy the vector space axioms for vector addition and scalar multiplication. We've also added a `NormedVectorSpace` trait, which includes an api for distance and magnitude.

These traits underpin the new curve and shape sampling apis. `VectorSpace` is implemented for `f32`, the `glam` vector types, and several of the new color-space types. It completely replaces `bevy_math::Point`.

The splines module in Bevy has been lacking some features for a long time. Splines are extremely useful in game development, so improving them would improve everything that uses them.

The biggest addition is NURBS support! It is a variant of a B-Spline with much more parameters that can be tweaked to create specific curve shapes. We also added a `LinearSpline`, which can be used to put straight line segments in a curve. `CubicCurve` now acts as a sequence of curve segments to which you can add new pieces, so you can mix various spline types together to form a single path.

## 2D Mesh Wireframes [#](https://bevy.org/news/bevy-0-14/#2d-mesh-wireframes)

Authors:[@msvbg](https://github.com/msvbg), [@IceSentry](https://github.com/IceSentry)

PRs:[#12135](https://github.com/bevyengine/bevy/pull/12135)

Wireframe materials are used to render the individual edges and faces of a mesh. They are often used as a debugging tool to visualize geometry, but can also be used for various stylized effects. Bevy supports displaying 3D meshes as wireframes, but lacked the ability to do this for 2D meshes until now.

To render your 2D mesh as a wireframe, add `Wireframe2dPlugin` to your app and a `Wireframe2d` component to your sprite. The color of the wireframe can be configured per-object by adding the `Wireframe2dColor` component, or globally by inserting a `Wireframe2dConfig` resource.

For an example of how to use the feature, have a look at the new [wireframe_2d example](https://github.com/bevyengine/bevy/blob/b17292f9d11cf3d3fb4a2fb3e3324fb80afd8c88/examples/2d/wireframe_2d.rs):

![A screenshot demonstrating the new 2D wireframe material](https://bevy.org/news/bevy-0-14/12135_Support_wireframes_for_2D_meshes.jpg)

## Custom Reflect Field Attributes [#](https://bevy.org/news/bevy-0-14/#custom-reflect-field-attributes)

Authors:[@MrGVSV](https://github.com/MrGVSV)

PRs:[#11659](https://github.com/bevyengine/bevy/pull/11659)

One of the features of Bevy's reflection system is the ability to attach arbitrary "type data" to a type. This is most often used to allow trait methods to be called dynamically. However, some users saw it as an opportunity to do other awesome things.

The amazing [bevy-inspector-egui](https://github.com/jakobhellermann/bevy-inspector-egui) used type data to great effect in order to allow users to configure their inspector UI per field:

```rust
use bevy_inspector_egui::prelude::*;
use bevy_reflect::Reflect;

#[derive(Reflect, Default, InspectorOptions)]
#[reflect(InspectorOptions)]
struct Slider {
    #[inspector(min = 0.0, max = 1.0)]
    value: f32,
}
```

Taking inspiration from this, Bevy 0.14 adds proper support for custom attributes when deriving `Reflect`, so users and third-party crates should no longer need to create custom type data specifically for this purpose. These attributes can be attached to structs, enums, fields, and variants using the `#[reflect(@...)]` syntax, where the `...` can be any expression that resolves to a type implementing `Reflect`.

For example, we can use Rust's built-in `RangeInclusive` type to specify our own range for a field:

```rust
use std::ops::RangeInclusive;
use bevy_reflect::Reflect;

#[derive(Reflect, Default)]
struct Slider {
    #[reflect(@RangeInclusive<f32>::new(0.0, 1.0))]
    // Since this accepts any expression,
    // we could have also used Rust's shorthand syntax:
    // #[reflect(@0.0..=1.0_f32)]
    value: f32,
}
```

Attributes can then be accessed dynamically using [`TypeInfo`](https://docs.rs/bevy/latest/bevy/reflect/enum.TypeInfo.html):

```rust
let TypeInfo::Struct(type_info) = Slider::type_info() else {
    panic!("expected struct");
};

let field = type_info.field("value").unwrap();

let range = field.get_attribute::<RangeInclusive<f32>>().unwrap();
assert_eq!(*range, 0.0..=1.0);
```

This feature opens up a lot of possibilities for things built on top of Bevy's reflection system. And by making it agnostic to any particular usage, it allows for a wide range of use cases, including aiding editor work down the road.

In fact, this feature has already been put to use by [`bevy_reactor`](https://github.com/viridia/bevy_reactor/blob/main/examples/complex/reflect_demo.rs) to power their custom inspector UI:

```rust
#[derive(Resource, Debug, Reflect, Clone, Default)]
pub struct TestStruct {
    pub selected: bool,

    #[reflect(@ValueRange::<f32>(0.0..1.0))]
    pub scale: f32,

    pub color: Srgba,
    pub position: Vec3,
    pub unlit: Option<bool>,

    #[reflect(@ValueRange::<f32>(0.0..10.0))]
    pub roughness: Option<f32>,

    #[reflect(@Precision(2))]
    pub metallicity: Option<f32>,

    #[reflect(@ValueRange::<f32>(0.0..1000.0))]
    pub factors: Vec<f32>,
}
```

![A custom UI inspector built using the code above in bevy_reactor](https://bevy.org/news/bevy-0-14/custom_attributes_demo.jpg)

## Query Iteration Sorting [#](https://bevy.org/news/bevy-0-14/#query-iteration-sorting)

Authors:[@Victoronz](https://github.com/Victoronz)

PRs:[#13417](https://github.com/bevyengine/bevy/pull/13417)

Bevy does not make any guarantees about the order of items. So if we wish to work with our query items in a certain order, we need to sort them! We might want to display the scores of the players in order, or ensure a consistent iteration order for the sake of networking stability. In 0.13 a sort could look like this:

```rust
#[derive(Component, Copy, Clone, Deref)]
pub struct Attack(pub usize)

fn handle_enemies(enemies: Query<(&Health, &Attack, &Defense)>) {
    // An allocation!
    let mut enemies: Vec<_> = enemies.iter().collect();
    enemies.sort_by_key(|(_, atk, ..)| *atk)
    for enemy in enemies {
        work_with(enemy)
    }
}
```

This can get especially unwieldy and repetitive when sorting within multiple systems. Even if we always want the same sort, different [`Query`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Query.html) types make it unreasonably difficult to abstract away as a user! To solve this, we implemented new sort methods on the [`QueryIter`](https://docs.rs/bevy/0.14/bevy/ecs/query/struct.QueryIter.html) type, turning the example into:

```rust
// To be used as a sort key, `Attack` now implements Ord.
#[derive(Component, Copy, Clone, Deref, PartialEq, Eq, PartialOrd, Ord)]
pub struct Attack(pub usize)

fn handle_enemies(enemies: Query<(&Health, &Attack, &Defense)>) {
    // Still an allocation, but undercover.
    for enemy in enemies.iter().sort::<&Attack>() {
        work_with(enemy)
    }
}
```

To sort our query with the `Attack` component, we specify it as the generic parameter to [`sort`](https://docs.rs/bevy/0.14/bevy/ecs/query/struct.QueryIter.html?search=Component#method.sort). To sort by more than one [`Component`](https://docs.rs/bevy/0.14/bevy/ecs/component/trait.Component.html), we can do so, independent of [`Component`](https://docs.rs/bevy/0.14/bevy/ecs/component/trait.Component.html) order in the original [`Query`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Query.html) type: `enemies.iter().sort::<(&Defense, &Attack)>()`

The generic parameter can be thought of as being a [lens](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Query.html#method.transmute_lens) or "subset" of the original query, on which the underlying sort is actually performed. The result is then internally used to return a new sorted query iterator over the original query items. With the default [`sort`](https://docs.rs/bevy/0.14/bevy/ecs/query/struct.QueryIter.html?search=Component#method.sort), the lens has to be fully [`Ord`](https://doc.rust-lang.org/stable/std/cmp/trait.Ord.html), like with [`slice::sort`](https://doc.rust-lang.org/nightly/std/primitive.slice.html#method.sort). If this is not enough, we also have the counterparts to the remaining 6 sort methods from [`slice`](https://doc.rust-lang.org/nightly/std/primitive.slice.html)!

The generic lens argument works the same way as with [`Query::transmute_lens`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Query.html#method.transmute_lens). We do not use filters, they are inherited from the original query. The [`transmute_lens`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Query.html#method.transmute_lens) infrastructure has some nice additional features, which allows for this:

```rust
fn handle_enemies(enemies: Query<(&Health, &Attack, &Defense, &Rarity)>) {
    for enemy in enemies.iter().sort_unstable::<Entity>() {
        work_with(enemy)
    }
}
```

Because we can add [`Entity`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Entity.html) to any lens, we can sort by it without including it in the original query!

These sort methods work with both [`Query::iter`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Query.html#method.iter) and [`Query::iter_mut`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Query.html#method.iter_mut)! The rest of the of the iterator methods on [`Query`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Query.html) do not currently support sorting. The sorts return [`QuerySortedIter`](https://docs.rs/bevy/0.14/bevy/ecs/query/struct.QuerySortedIter.html), itself an iterator, enabling the use of further iterator adapters on it.

Keep in mind that the lensing does add some overhead, so these query iterator sorts do not perform the same as a manual sort on average. However, this _strongly_ depends on workload, so best test it yourself if relevant!

## SystemBuilder [#](https://bevy.org/news/bevy-0-14/#systembuilder)

Authors:[@james-j-obrien](https://github.com/james-j-obrien)

PRs:[#13123](https://github.com/bevyengine/bevy/pull/13123)

Bevy users _love_ systems, so we made a builder for their systems so they can build systems from within systems. At runtime, using dynamically-defined component and resource types!

While you can use [`SystemBuilder`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.SystemBuilder.html) as an ergonomic alternative to the [`SystemState`](https://docs.rs/bevy/0.14/bevy/ecs/system/struct.SystemState.html) API for splitting the [`World`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.World.html) into disjoint borrows, its true values lies in its dynamic usage.

You can choose to create a different system based on runtime branches or, more intriguingly, the queries and so on can use runtime-defined component IDs. This is another vital step towards creating an ergonomic and safe API to work with [dynamic queries](https://bevy.org/news/bevy-0-13/#dynamic-queries), laying the groundwork for the devs who want to integrate scripting languages or bake in sophisticated modding support for their game.

```rust
// Start by creating builder from the world
let system = SystemBuilder::<()>::new(&mut world)
    // Various helper methods exist to add `SystemParam`.
    .resource::<R>()
    .query::<&A>()
    // Alternatively use `.param::<T>()` for any other `SystemParam` types.
    .param::<MyParam>()
    // Finish it all up with a call `.build`
    .build(my_system);
// The parameters the builder is initialized with will appear first in the arguments.
let system = SystemBuilder::<(Res<R>, Query<&A>)>::new(&mut world)
    .param::<MyParam>()
    .build(my_system);
// Parameters like `Query` that implement `BuildableSystemParam` can use
// `.builder::<T>()` to build in place.
let system = SystemBuilder::<()>::new(&mut world)
    .resource::<R>()
    // This turns our query into a `Query<&A, With<B>>`
    .builder::<Query<&A>>(|builder| { builder.with::<B>(); })
    .param::<MyParam>()
    .build(my_system);
world.run_system_once(system);
```

## Throttle Render Assets [#](https://bevy.org/news/bevy-0-14/#throttle-render-assets)

Authors:[@robtfm](https://github.com/robtfm), [@IceSentry](https://github.com/IceSentry), [@mockersf](https://github.com/mockersf)

PRs:[#12622](https://github.com/bevyengine/bevy/pull/12622)

Using a lot of assets? Uploading lots of bytes to the GPU in a short time might cause stutters due to the render world waiting for uploads to finish.

Often it's a more delightful experience if an application runs smoothly than if it stutters, and a few frames worth of delay before seeing an asset appear is often not even perceptible.

This experience is now possible:

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(RenderAssetBytesPerFrame::new(1_000_000_000)) // Tune to your situation by experimenting!
        .run();
}
```

That's it! The number provided should be chosen by figuring out a nice trade-off between no stuttering and an acceptable delay.

This feature relies on assets knowing how many bytes they will occupy when sent to the GPU. Currently this is known by images and meshes, with more assets types expected to be able to report this in the future.

## StandardMaterial UV Channel Selection [#](https://bevy.org/news/bevy-0-14/#standardmaterial-uv-channel-selection)

Authors:[@geckoxx](https://github.com/geckoxx)

PRs:[#13200](https://github.com/bevyengine/bevy/pull/13200)

Previously, StandardMaterial always defaulted to using ATTRIBUTE_UV_0 for each texture except lightmap, which isn't flexible enough for a lot of gltf files. In **Bevy 0.14**, a new UvChannel enum was added allowing you to select the channel to use for each texture in StandardMaterial.

Here's a before and after showing the support of ATTRIBUTE_UV_1 across textures:

![UV Channel Selection](https://bevy.org/news/bevy-0-14/uv_channel_selection.jpg)

## Remove limit on RenderLayers [#](https://bevy.org/news/bevy-0-14/#remove-limit-on-renderlayers)

Authors:[@tychedelia](https://github.com/tychedelia), [@robtfm](https://github.com/robtfm), [@UkoeHB](https://github.com/UkoeHB)

PRs:[#13317](https://github.com/bevyengine/bevy/pull/13317)

Render layers are used to quickly toggle the visibility of sets of objects, and control which objects can be seen by which cameras. This can be useful for things like debug views, gear preview screens, toggle-able diegetic UI and so on.

Before Bevy 0.14 the membership was defined by a bitmask which had limited slots available. Now, there is no longer any practical limit to how many layers you can define, which is particularly helpful for creative coding applications like [nannou](https://nannou.cc/)! We've made sure to keep the common case fast, but now use a growable mask that will allocate space for additional layers as needed. Remember, there's still a cost to check visibility per layer, but this allows for more dynamic uses where layers can be created on demand without worrying about going over a limit.

## `on_unimplemented` Diagnostics [#](https://bevy.org/news/bevy-0-14/#on-unimplemented-diagnostics)

Authors:[@bushrat011899](https://github.com/bushrat011899), [@alice-i-cecile](https://github.com/alice-i-cecile), [@Themayu](https://github.com/Themayu)

PRs:[#13347](https://github.com/bevyengine/bevy/pull/13347)

Bevy takes full advantage of the powerful type system Rust provides, but with that power can often come confusion when even minor mistakes are made.

```rust
use bevy::prelude::*;

struct MyResource;

fn main() {
    App::new()
        .insert_resource(MyResource)
        .run();
}
```

Running the above will produce a compiler error, let's see why...

Click to expand...

```txt
error[E0277]: the trait bound `MyResource: Resource` is not satisfied
   --> example.rs:6:32
    |
6   |     App::new().insert_resource(MyResource).run();
    |                --------------- ^^^^^^^^^^ the trait `Resource` is not implemented for `MyResource`
    |                |
    |                required by a bound introduced by this call
    |
    = help: the following other types implement trait `Resource`:
              AccessibilityRequested
              ManageAccessibilityUpdates
              bevy::a11y::Focus
              DiagnosticsStore
              FrameCount
              bevy::prelude::Axis<T>
              WinitActionHandlers
              ButtonInput<T>
            and 127 others
note: required by a bound in `bevy::prelude::App::insert_resource`
   --> /bevy/crates/bevy_app/src/app.rs:537:31
    |
537 |     pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
    |                               ^^^^^^^^ required by this bound in `App::insert_resource`
```

The compiler suggests we use a different type that implements `Resource`, or that we implement the trait on `MyResource`. The former doesn't help us at all, and the latter fails to mention the available derive macro.

With the release of Rust 1.78, Bevy can now provide more direct messages for certain types of errors during compilation using [diagnostic attributes](https://blog.rust-lang.org/2024/05/02/Rust-1.78.0.html#diagnostic-attributes).

```txt
error[E0277]: `MyResource` is not a `Resource`
   --> example.rs:6:32
    |
6   |     App::new().insert_resource(MyResource).run();
    |                --------------- ^^^^^^^^^^ invalid `Resource`
    |                |
    |                required by a bound introduced by this call
    |
    = help: the trait `Resource` is not implemented for `MyResource`
    = note: consider annotating `MyResource` with `#[derive(Resource)]`
    = help: the following other types implement trait `Resource`:
              AccessibilityRequested
...
```

Now, the error message has a more approachable entry point, and a new `note` section pointing to the derive macro for resources. If Bevy's suggestions _aren't_ the solution to your problem, the rest of the compiler error is still included just in case.

These diagnostics have been implemented for various traits across Bevy, and we hope to improve this experience as new features are added to Rust. For example, we'd really like to improve the experience of working with tuples of `Component`'s, but we're not quite there yet. You can read more about this change in the [pull request](https://github.com/bevyengine/bevy/pull/13347) and associated [issue](https://github.com/bevyengine/bevy/issues/12377).

## Motion Vectors and TAA for Animated Meshes [#](https://bevy.org/news/bevy-0-14/#motion-vectors-and-taa-for-animated-meshes)

Authors:[@pcwalton](https://github.com/pcwalton)

PRs:[#13572](https://github.com/bevyengine/bevy/pull/13572)

Back in **Bevy 0.11** we added [Temporal Anti Aliasing (TAA)](https://bevy.org/news/bevy-0-11/#temporal-anti-aliasing), which uses Motion Vectors to determine how fast an object is moving. However, in **Bevy 0.11** we only added Motion Vector support for "static" meshes, meaning TAA did not work for animated meshes using skeletal animation or morph targets.

In **Bevy 0.14**, we implemented [Per-Object Motion Blur](https://bevy.org/news/bevy-0-14/#per-object-motion-blur), which _also_ uses Motion Vectors and therefore would have that same limitation.

Fortunately in **Bevy 0.14** we implemented Motion Vectors for skinned meshes and meshes with morph targets, closing this gap and enabling TAA, Per-Object Motion Blur, and future Motion Vector features to work with animated meshes.

## Improved Matrix Naming [#](https://bevy.org/news/bevy-0-14/#improved-matrix-naming)

Authors:[@ricky26](https://github.com/ricky26)

PRs:[#13489](https://github.com/bevyengine/bevy/pull/13489)

Game engines generally provide a set of matrices to perform space transformations in the game world. Commonly, the following spaces are used:

- **Normalized Device Coordinates**: used by the graphics API directly
- **Clip Space**: coordinates after projection but before perspective divide
- **View Space**: coordinates in the camera's view
- **World Space**: global coordinates (this is the one we most often talk about!)
- **Model Space**: (or local space) coordinates relative to an entity

A common example is the 'model view projection matrix', which is the transformation from model space to NDC space (peculiarly in this shorthand, the view matrix is often a transformation from world _to view_ space, but the model matrix is a transformation _from model_ (or local) space to world space). Usually, matrices are referred to as part of that shorthand, so for example, the projection matrix transforms from view coordinates to NDC coordinates.

In a couple of places, Bevy had a view matrix, which was the transformation from view to world space (rather than from world to view space as above). Additionally, even when used consistently, the single-word shorthands are ambiguous and can cause confusion. We felt that a clearer convention was needed.

From now on, matrices in Bevy are named `y_from_x`, for example `world_from_local`, which would denote the transformation from local to world-space coordinates. One tidy benefit of this is that the inverse matrices are named `x_from_y`, and when multiplying between spaces, it's easy to see that it's correct.

For example, instead of writing:

```rust
let model_view_projection = projection * view * model;
```

You would now write:

```rust
let clip_from_local = clip_from_view * view_from_world * world_from_local;
```

## Typed glTF Labels [#](https://bevy.org/news/bevy-0-14/#typed-gltf-labels)

Authors:[@mockersf](https://github.com/mockersf), [@rparrett](https://github.com/rparrett), [@alice-i-cecile](https://github.com/alice-i-cecile)

PRs:[#13586](https://github.com/bevyengine/bevy/pull/13586)

If you've been using [`glTF`](https://www.khronos.org/gltf/) files for your scenes or looked at an example that does you've might have seen the _labels_ at the end of the asset path:

```rust
let model_pine = asset_server.load("models/trees/pine.gltf#Scene0");
let model_hen = asset_server.load("models/animals/hen.gltf#Scene0");
let animation_hen = asset_server.load("models/animals/hen.gltf#Aniamtion1"); // Oh no!
```

Notice the `#Scene0` syntax at the end. The glTF format is able to contain many things in a single file, including several scenes, animations, lights, and more.

These labels are a way of telling Bevy which part of the file we're loading.

However this is prone to user-error, and it looks like an error snuck in! The hen animation got the label `Aniamtion1` instead of `Animation1`.

No more! The above can now be re-written like so:

```rust
let hen = "models/animals/hen.gltf"; // Can re-use this more easily too
let model_pine = asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/trees/pine.gltf"));
let model_hen = asset_server.load(GltfAssetLabel::Scene(0).from_asset(hen));
let animation_hen = asset_server.load(GltfAssetLabel::Animation(0).from_asset(hen)); // No typo!
```

Check out [`glTF label docs`](https://docs.rs/bevy/0.14/bevy/gltf/enum.GltfAssetLabel.html) to know which parts you can query for.

## winit v0.30 [#](https://bevy.org/news/bevy-0-14/#winit-v0-30)

Authors:[@pietrosophya](https://github.com/pietrosophya), [@mockersf](https://github.com/mockersf)

PRs:[#13366](https://github.com/bevyengine/bevy/pull/13366)

[Winit v0.30](https://docs.rs/winit/0.30.0/winit/changelog/v0_30/index.html) changed its API to support a trait based architecture instead of a plain event-based one. Bevy 0.14 now implements that new architecture, making the event loop handling easier to follow.

It's now possible to define a custom `winit` user event, that can be used to trigger App updates, and that can be read inside systems to trigger specific behaviors. This is particularly useful to send events from outside the `winit` event loop and manage them inside Bevy systems (see the [`window/custom_user_event.rs`](https://github.com/bevyengine/bevy/blob/release-0.14.0/examples/window/custom_user_event.rs) example).

The `UpdateMode` enum now accepts only two values: `Continuous` and `Reactive`. The latter exposes 3 new properties to enable reactivity to device, user, or window events. The previous `UpdateMode::Reactive` is now equivalent to `UpdateMode::reactive()`, while `UpdateMode::ReactiveLowPower` maps to `UpdateMode::reactive_low_power()`.

- `Idle`: the loop has not started yet
- `Running` (previously called `Started`): the loop is running
- `WillSuspend`: the loop is going to be suspended
- `Suspended`: the loop is suspended
- `WillResume`: the loop is going to be resumed

Note: the `Resumed` state has been removed since the resumed app is just `Running`.

## Scene, Mesh, and Material glTF Extras [#](https://bevy.org/news/bevy-0-14/#scene-mesh-and-material-gltf-extras)

Authors:[@kaosat-dev](https://github.com/kaosat-dev)

PRs:[#13453](https://github.com/bevyengine/bevy/pull/13453)

The glTF 3D model file format allows passing additional user defined metadata in the _extras_ properties, and in addition to the glTF extras at the primitive/node level , Bevy now has specific GltfExtras for:

- scenes: **SceneGltfExtras** injected at the scene level if any
- meshes: **MeshGltfExtras**, injected at the mesh level if any
- materials: **MaterialGltfExtras**, injected at the mesh level if any: ie if a mesh has a material that has gltf extras, the component will be injected there.

You can now easily query for these specific extras

```rust
fn check_for_gltf_extras(
    gltf_extras_per_entity: Query<(
        Entity,
        Option<&Name>,
        Option<&GltfSceneExtras>,
        Option<&GltfExtras>,
        Option<&GltfMeshExtras>,
        Option<&GltfMaterialExtras>,
    )>,
) {
    // use the extras' data 
    for (id, name, scene_extras, extras, mesh_extras, material_extras) in
        gltf_extras_per_entity.iter()
    {

    }
}

```

This makes passing information from programs such as Blender to Bevy via gltf files more spec compliant, and more practical!

## Resource Entity Mapping in Scenes [#](https://bevy.org/news/bevy-0-14/#resource-entity-mapping-in-scenes)

Authors:[@brandon-reinhart](https://github.com/brandon-reinhart)

PRs:[#13650](https://github.com/bevyengine/bevy/pull/13650)

Bevy's `DynamicScene` is a collection of resources and entities that can be serialized to create collections like prefabs or savegame data. When a DynamicScene is deserialized and written into a World - such as when a saved game is loaded - the dynamic entity identifiers inside the scene must be mapped to their newly spawned counterparts.

Previously, this mapping was only available to Entity identifiers stored on Components. In Bevy 0.14, Resources can reflect `MapEntitiesResource` and implement the `MapEntities` trait to get access to the `EntityMapper`.

```rust
    // This resource reflects MapEntitiesResource and implements the MapEntities trait.
    #[derive(Resource, Reflect, Debug)]
    #[reflect(Resource, MapEntitiesResource)]
    struct TestResource {
        entity_a: Entity,
        entity_b: Entity,
    }

    // A simple and common use is a straight mapping of the old entity to the new.
    impl MapEntities for TestResource {
        fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
            self.entity_a = entity_mapper.map_entity(self.entity_a);
            self.entity_b = entity_mapper.map_entity(self.entity_b);
        }
    }
```

## CompassQuadrant and CompassOctant [#](https://bevy.org/news/bevy-0-14/#compassquadrant-and-compassoctant)

Authors:[@BobG1983](https://github.com/BobG1983), [@alice-i-cecile](https://github.com/alice-i-cecile)

PRs:[#13653](https://github.com/bevyengine/bevy/pull/13653)

There are many instances in game development where its important to know the compass facing for a given direction. This is particularly true in 2D games that use four or eight direction sprites, or want to map analog input into discrete movement directions.

In order to make this easier the enums `CompassQuadrant` (for a four-way division) and `CompassOctant` (for an eight-way division) have been added with implementations to and `From<Dir2>` for ease of use.

## Support `AsyncSeek` When Loading Assets [#](https://bevy.org/news/bevy-0-14/#support-asyncseek-when-loading-assets)

Authors:[@BeastLe9enD](https://github.com/BeastLe9enD)

PRs:[#12547](https://github.com/bevyengine/bevy/pull/12547)

Assets can be huge, and you don't always need all of the data contained in a single file.

Bevy allows you to add your [own asset loaders](https://github.com/bevyengine/bevy/blob/release-0.14.0/examples/asset/processing/asset_processing.rs). Starting in Bevy 0.14, you can now seek to an offset of your choice, reading partway through the file.

Perhaps you have the `.celestial` file format which encodes the universe, but you want to only look at lil' asteroids which always appear at some offset:

```rust
#[derive(Default)]
struct UniverseLoader;

#[derive(Asset, TypePath, Debug)]
struct JustALilAsteroid([u8; 128]); // Each lil' asteroid uses this much data

impl AssetLoader for UniverseLoader {
    type Asset = JustALilAsteroid;
    type Settings = ();
    type Error = std::io::Error;
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<JustALilAsteroid, Self::Error> {
        // The universe is big, and our lil' asteroids don't appear until this offset
        // in the celestial file format!
        let offset_of_lil_asteroids = 5_000_000_000_000;

        // Skip vast parts of the universe with the new async seek trait!
        reader
            .seek(SeekFrom::Start(offset_of_lil_asteroids))
            .await?;

        let mut asteroid_buf = [0; 128];
        reader.read_exact(&mut asteroid_buf).await?;

        Ok(JustALilAsteroid(asteroid_buf))
    }

    fn extensions(&self) -> &[&str] {
        &["celestial"]
    }
}
```

This works because Bevy's [`reader`](https://docs.rs/bevy/0.14/bevy/asset/io/type.Reader.html) type passed into the asset loader's `load` function now implements [`AsyncSeek`](https://docs.rs/futures-io/latest/futures_io/trait.AsyncSeek.html).

Real world use cases might for example be:

- You have packed several assets in an archive and you wish to skip to an asset within and read that
- You are dealing with big datasets such as map data and you know where to extract some locations of interest

## LoadState::Failed Now Has Error Info [#](https://bevy.org/news/bevy-0-14/#loadstate-failed-now-has-error-info)

Authors:[@bugsweeper](https://github.com/bugsweeper)

PRs:[#12709](https://github.com/bevyengine/bevy/pull/12709)

Rust prides itself on its error handling, and Bevy has been steadily catching up. Previously, when checking if an asset was loaded using [`AssetServer::get_load_state`](https://docs.rs/bevy/0.14/bevy/asset/struct.AssetServer.html#method.get_load_state), all you'd get back was a data-less [`LoadState::Failed`](https://docs.rs/bevy/0.14/bevy/asset/enum.LoadState.html) if something went wrong. Not very useful for debugging!

Now, a full [`AssetLoadError`](https://docs.rs/bevy/0.14/bevy/asset/enum.AssetLoadError.html) is included, with 14 different variants telling you exactly what went wrong. Great for troubleshooting, and it opens the door to proper error handling in more complex apps.

## `AppExit` Errors [#](https://bevy.org/news/bevy-0-14/#appexit-errors)

Authors:[@Brezak](https://github.com/Brezak), [@alice-i-cecile](https://github.com/alice-i-cecile)

PRs:[#13022](https://github.com/bevyengine/bevy/pull/13022)

When running an app, there might be many reasons to trigger an exit. Maybe the user has pressed the quit button, or the render thread has encountered an error and died. You might want to distinguish between these two situations and return an appropriate [exit code](https://doc.rust-lang.org/std/process/struct.ExitCode.html#impl-From%3Cu8%3E-for-ExitCode) from your application.

In **Bevy 0.14**, you can. The `AppExit` event is now an enum with two variants: `Success` and `Error`. The error variant also holds a non-zero code, which you're allowed to use however you wish. Since `AppExit` events now contain useful information, app runners and `App::run` now return the event that resulted in the application exiting.

For plugin developers, `App` has gained a new method, `App::should_exit`, which will check if any `AppExit` events were sent in the last two updates. To make sure `AppExit::Success` events won't drown out useful error information, this method will return any `AppExit::Error` events, even if they were sent after an `AppExit::Success`.

Finally, `AppExit` also implements the [`Termination`](https://doc.rust-lang.org/stable/std/process/trait.Termination.html) trait, so it can be returned from main.

```rust
use bevy::prelude::*;

fn exit_with_a_error_code(mut events: EventWriter<AppExit>) {
    events.send(AppExit::from_code(42));
}

fn main() -> AppExit {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_systems(Update, exit_with_a_error_code)
        .run() // There's no semicolon here, `run()` returns `AppExit`.
}
```

![App returning a 42 exit code](https://bevy.org/news/bevy-0-14/exit_with_a_42.jpg)

## Make dynamic_linking a no-op on WASM targets [#](https://bevy.org/news/bevy-0-14/#make-dynamic-linking-a-no-op-on-wasm-targets)

Authors:[@james7132](https://github.com/james7132)

PRs:[#12672](https://github.com/bevyengine/bevy/pull/12672)

WASM does not support dynamic libraries that can be linked to during runtime. Before, Bevy would fail to compile if you enabled the `dynamic_linking` feature.

```bash
$ cargo build --target wasm32-unknown-unknown --features bevy/dynamic_linking
error: cannot produce dylib for `bevy_dylib v0.13.2` as the target `wasm32-unknown-unknown` does not support these crate types
```

Now, Bevy will fallback to static linking for all WASM targets. If you enable `dynamic_linking` for development, you no longer need to disable it for WASM.

## Deprecate Dynamic Plugins [#](https://bevy.org/news/bevy-0-14/#deprecate-dynamic-plugins)

Authors:[@BD103](https://github.com/BD103)

PRs:[#13080](https://github.com/bevyengine/bevy/pull/13080)

`bevy_dynamic_plugin` was a tool added in Bevy's original 0.1 release: intended to serve as a tool for dynamically loading / linking Rust code for use with things like modding. Unfortunately, this feature didn't see much community uptake, and as a result had a vanishingly small number of contributions to refine and document it over the years.

Combined with a challenging, intrinsically unsafe API that was producing [worrying failures](https://github.com/bevyengine/bevy/issues/13073) for users, we've decided to deprecate `bevy_dynamic_plugin` and will be removing it completely in Bevy 0.15. If you were a happy user of this, simply copy the rather-small crate into your own project and proceed as before.

We still think that both modding and hot-reloading code for faster development times are valuable use cases that Bevy _should_ help support one day. Our hope is that by removing this as a first-party crate, we can spur on third-party experiments and avoid wasting users' time as they investigate a complex potential solution before concluding that it doesn't yet meet their needs.

## Bevy Working Groups [#](https://bevy.org/news/bevy-0-14/#bevy-working-groups)

Authors:[@alice-i-cecile](https://github.com/alice-i-cecile)

PRs:[#13162](https://github.com/bevyengine/bevy/pull/13162)

Bevy has a ton of incredibly talented contributors, so keeping track of what's going on and making informed decisions can be a real challenge. We're experimenting with [working groups](https://github.com/bevyengine/bevy/blob/main/CONTRIBUTING.md#join-a-working-group): ad hoc groups tackling harder issues by creating a design document, getting sign-off from the experts, and then implementing it. If you'd like to help make complex, high-impact changes to Bevy: join or form a working group!

## What's Next? [#](https://bevy.org/news/bevy-0-14/#what-s-next)

The features above may be great, but what else does Bevy have in flight? Peering deep into the mists of time (predictions are _extra_ hard when your team is almost all volunteers!), we can see some exciting work taking shape:

- **Better Scenes:** Scenes are one of Bevy's core building blocks: designed to be a powerful tool for authoring levels and creating reusable game objects, whether they're a radio button widget or a monster. We're working on a new scene system with a new syntax that will make defining scenes in assets _and_ in code more powerful and more pleasant. You can check out the(now slightly old) [project kickoff discussion](https://github.com/bevyengine/bevy/discussions/9538) for more information. We're also very close to putting out a design document outlining our plans and the current state of the implementation.
- **ECS Relations:** Relations (a first-class feature for linking entities together) is wildly desired but remarkably complex, driving features and refactors to our ECS internals. The [working group](https://discord.com/channels/691052431525675048/1237010014355456115) has been patiently laying out what we need to do and why in this [RFC](https://github.com/bevyengine/rfcs/pull/79).
- **Better Audio:** Bevy's built-in audio solution has never really hit the right notes. The [Better Audio working group](https://discord.com/channels/691052431525675048/1236113088793677888) is plotting a path forward.
- **Contributing Book:** Our documentation on how to contribute is scattered to the four corners of our repositories. By gathering this together, the [Contributing Book working group](https://discord.com/channels/691052431525675048/1236112637662724127) hopes to make it easier to discover and maintain.
- **Curve Abstraction:** Curves come up all of the time in game dev, and the mathmagicians that make up the [Curve Crew](https://discord.com/channels/691052431525675048/1236110755212820581) are [designing a trait](https://github.com/bevyengine/rfcs/pull/80) to unify and power them.
- **Better Text:** our existing text solution isn't up to the demands of modern UI. The "Lorem Ipsum" working group is [looking into](https://discord.com/channels/691052431525675048/1248074018612051978) replacing it with a better solution.
- **A Unified View on Dev Tools:** In 0.14, we've added a stub `bevy_dev_tools` crate: a place for tools and overlays that speed up game development such as performance monitors, fly cameras, or in-game commands to spawn game objects. We're working on adding more tools, and creating a [dev tool abstraction](https://github.com/bevyengine/rfcs/pull/77). This will give us a unified way to enable/disable, customize and group this grab bag of tools into toolboxes to create something like Quake console or VSCode Command Palette with tools from around the ecosystem.
- **Bevy Remote Protocol:** Communicating with actively running Bevy games is an incredibly powerful tool for building editors, debuggers and other tools. [We're developing](https://github.com/bevyengine/bevy/pull/13563) a reflection-powered protocol to create a solution that's ready to power a whole ecosystem.
- **A Modular, Maintainable Render Graph:** Bevy's existing rendering architecture is already quite good at providing reusable renderer features like `RenderPhases`, batching, and draw commands. However, the render graph interface itself is one remaining pain points. Since it's distributed across many files the control flow is hard to understand, and its heavy use of ECS resources for passing around rendering data actively works against modularity. While the exact design hasn't been finalized (and feedback is very welcome!), we've been actively working to [redesign the render graph](https://github.com/bevyengine/bevy/pull/13397) in order to build up to a larger refactor of the renderer towards modularity and ease of use.