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

你应该使用以下方式：

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

#### Observer

Observer 是按需运行的系统，用于监听"触发的"事件。这些事件可以针对特定实体触发，也可以"全局"触发（无实体目标）。

与 hooks 不同，observer 是一种灵活的工具，适用于高层应用逻辑。它们可以监听用户定义的事件何时被触发。

```rust
#[derive(Event)]
struct Message {
    text: String
}

world.observe(|trigger: Trigger<Message>| {
    println!("{}", trigger.event().message.text);
});
```

Observer 在它们监听的事件被触发时_立即_运行：

```rust
// 所有已注册的 `Message` observer 会在此处立即运行
world.trigger(Message { text: "Hello".to_string() });
```

如果事件是通过 [`Command`](https://docs.rs/bevy/0.14/bevy/ecs/world/trait.Command.html) 触发的，observer 将在该 [`Command`](https://docs.rs/bevy/0.14/bevy/ecs/world/trait.Command.html) 被刷新时运行：

```rust
fn send_message(mut commands: Commands) {
    // 当此系统的命令被刷新时，这将触发所有 `Message` observer
    commands.trigger(Message { text: "Hello".to_string() } );
}
```

事件也可以带有实体目标触发：

```rust
#[derive(Event)]
struct Resize { size: usize }

commands.trigger_targets(Resize { size: 10 }, some_entity);
```

你可以同时为多个实体触发一个事件：

```rust
commands.trigger_targets(Resize { size: 10 }, [e1, e2]);
```

"全局" observer 会在_任何_目标被触发时执行：

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

注意，observer 可以使用像 [`Query`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Query.html) 这样的系统参数，就像普通系统一样。

你还可以添加仅针对_特定_实体运行的 observer：

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

点击展开……

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

这里有一个[新示例](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/ui/rounded_borders.rs)展示了这个新 API，其截图如下：

![rounded_borders example](https://bevy.org/news/bevy-0-14/rounded_borders.jpg)

## 使用 `AnimationGraph` 实现动画混合 [#](https://bevy.org/news/bevy-0-14/#animation-blending-with-the-animationgraph)

作者：[pcwalton](https://github.com/pcwalton)、[rparrett](https://github.com/rparrett)、[james7132](https://github.com/james7132)

PR：[#11989](https://github.com/bevyengine/bevy/pull/11989)

初学者的眼里，处理动画似乎非常简单。定义一系列关键帧，将模型的各个部分变换以匹配这些姿势。我们在上面加一些插值来平滑过渡，用户告诉你何时开始和停止动画。很简单！

但现代动画管线（尤其是在 3D 中！）要复杂得多：动画师期望能够平滑混合，并根据游戏玩法动态地程序化改变不同的动画。为了捕捉这种丰富性，业界发展了**动画图**的概念，用于将游戏对象的底层[状态机](https://en.wikipedia.org/wiki/Finite-state_machine)与应播放的动画以及各状态之间应发生的过渡结合起来。

一个玩家角色可能会行走、奔跑、挥剑、持剑防御……要创造出精美的效果，动画师需要能够在这些动画之间平滑切换，改变行走周期的速度以匹配地面移动速度，甚至同时执行多个动画！

在 Bevy 0.14 中，我们实现了[动画合成 RFC](https://github.com/bevyengine/rfcs/blob/main/rfcs/51-animation-composition.md)，提供了一个低层级 API，将代码驱动和资源驱动的动画混合能力带到了 Bevy。

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

    // 将图添加到我们的资源集合中
    let handle = animation_graphs.add(animation_graph);

    // 持有 handle
    commands.insert_resource(ExampleAnimationGraph(handle));
}
```

虽然它现在就可以发挥很大作用，但大多数动画师最终会更喜欢用 GUI 编辑这些图。我们计划在此 API 之上构建一个 GUI，作为传说中的 Bevy Editor 的一部分。现在，也有像 [`bevy_animation_graph`](https://crates.io/crates/bevy_animation_graph) 这样的第三方解决方案。

要了解更多并了解资源驱动的方法是什么样的，请查看新的 [`animation_graph` 示例](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/animation/animation_graph.rs)。

## 改进的颜色 API [#](https://bevy.org/news/bevy-0-14/#improved-color-api)

作者：[viridia](https://github.com/viridia)、[mockersf](https://github.com/mockersf)

PR：[#12013](https://github.com/bevyengine/bevy/pull/12013)

颜色是构建优秀游戏的重要组成部分：UI、特效、着色器以及更多功能都需要功能齐全、正确且便捷的颜色工具。

Bevy 现在支持广泛的色彩空间，每种色彩空间都有其自己的类型（例如 [`LinearRgba`](https://docs.rs/bevy/0.14/bevy/color/struct.LinearRgba.html)、[`Hsla`](https://docs.rs/bevy/0.14/bevy/color/struct.Hsla.html)、[`Oklaba`](https://docs.rs/bevy/0.14/bevy/color/struct.Oklaba.html)），并提供了一系列完整的、附带文档的操作以及它们之间的转换。

新的 API 更抗错误、更符合语言习惯，并且允许我们通过在渲染内部存储 [`LinearRgba`](https://docs.rs/bevy/0.14/bevy/color/struct.LinearRgba.html) 类型来节省工作。这个坚实的基础使我们能够实现一系列广泛的有用操作，聚类到像 [`Hue`](https://docs.rs/bevy/0.14/bevy/color/trait.Hue.html) 或 [`Alpha`](https://docs.rs/bevy/0.14/bevy/color/trait.Alpha.html) 这样的 trait 中，允许你在任何具有所需属性的色彩空间上进行操作。关键的是，现在支持颜色混合：非常适合程序化生成调色板和处理动画。

```rust
use bevy_color::prelude::*;

// 每个色彩空间现在对应一个特定类型
let red = Srgba::rgb(1., 0., 0.);

// 所有非标准色彩空间转换都通过源和目标色彩空间之间的最短路径完成，
// 以避免生成的代码出现二次爆炸。这个转换...
let red = Oklcha::from(red);
// ...实际上是通过以下方式实现的
let red = Oklcha::from(Oklaba::from(LinearRgba::from(red)));

// 我们添加了 `tailwind` 调色板颜色：非常适合快速而漂亮的原型开发！
// 现有的 CSS 调色板现在实际上与行业标准一致了 :p
let blue = tailwind::BLUE_500;

// 混合颜色时所处的色彩空间影响巨大！
// 考虑使用科学动机的 `Oklcha` 或 `Oklaba` 以获得感知均匀的效果。
let purple = red.mix(blue, 0.5);
```

大多数面向用户的 API 仍然接受与色彩空间无关的 [`Color`](https://docs.rs/bevy/0.14/bevy/color/enum.Color.html)（它现在包装了我们的色彩空间类型），而渲染内部则使用基于物理的 [`LinearRgba`](https://docs.rs/bevy/0.14/bevy/color/struct.LinearRgba.html) 类型。有关不同色彩空间及其各自适用场景的概述，请查看我们的[色彩空间使用](https://docs.rs/bevy/0.14/bevy/color/index.html#color-space-usage)文档。

`bevy_color` 提供了一个坚实、类型安全的基础，但它才刚刚起步。如果你想要另一个色彩空间，或者想对你的颜色做更多的事情，请提交 issue 或 PR，我们将很乐意提供帮助！

另外请注意，`bevy_color` 旨在作为一个独立的 crate 有效运行：你也可以在你的非 Bevy 项目中依赖它。

## 拉伸形状 [#](https://bevy.org/news/bevy-0-14/#extruded-shapes)

作者：[lynn-lumen](https://github.com/lynn-lumen)

PR：[#13270](https://github.com/bevyengine/bevy/pull/13270)

**Bevy 0.14** 引入了一个全新的图元组：拉伸体！

拉伸体是一个 2D 图元（基础形状），通过某个深度被"拉伸"到第三维度。生成的形状是一个棱柱（圆的特例是圆柱）。

```rust
// 创建一个宽 2 高 1 的椭圆。
let my_ellipse = Ellipse::from_size(2.0, 1.0);

// 创建这个椭圆的拉伸体，深度为 1。
let my_extrusion = Extrusion::new(my_ellipse, 1.);
```

所有拉伸体都沿着 Z 轴拉伸。这保证了深度为 0 的拉伸体与对应的基础形状完全相同，正如人们所期望的那样。

#### 测量和采样

由于所有基础形状实现了 [`Measured2d`](https://docs.rs/bevy/0.14/bevy/math/prelude/trait.Measured2d.html) 的拉伸体都实现了 [`Measured3d`](https://docs.rs/bevy/0.14/bevy/math/prelude/trait.Measured3d.html)，你可以轻松获得拉伸体的表面积或体积。如果你有一个自定义 2D 图元的拉伸体，只需为你的图元实现 [`Measured2d`](https://docs.rs/bevy/0.14/bevy/math/prelude/trait.Measured2d.html)，[`Measured3d`](https://docs.rs/bevy/0.14/bevy/math/prelude/trait.Measured3d.html) 将自动为拉伸体实现。

同样，如果拉伸体的基础形状实现了 [`ShapeSample<Output = Vec2>`](https://docs.rs/bevy/0.14/bevy/math/trait.ShapeSample.html) 和 [`Measured2d`](https://docs.rs/bevy/0.14/bevy/math/prelude/trait.Measured2d.html)，你可以对任何拉伸体的边界和内部进行采样。

```rust
// 创建一个半径为 1、长度为 2 的 2D 胶囊，拉伸深度为 3
let extrusion = Extrusion::new(Capsule2d::new(1.0, 2.0), 3.0);

// 获取拉伸体的体积
let volume = extrusion.volume();

// 获取拉伸体的表面积
let surface_area = extrusion.area();

// 创建一个随机数生成器
let mut rng = StdRng::seed_from_u64(4);

// 在拉伸体内部采样一个随机点
let interior_sample = extrusion.sample_interior(&mut rng);

// 在拉伸体表面采样一个随机点
let boundary_sample = extrusion.sample_boundary(&mut rng);
```

#### 包围体

你还可以获取拉伸体的包围球和轴对齐包围盒（AABB）。如果你有实现了 [`Bounded2d`](https://docs.rs/bevy/0.14/bevy/math/bounding/trait.Bounded2d.html) 的自定义 2D 图元，只需为你的图元实现 [`BoundedExtrusion`](https://docs.rs/bevy/0.14/bevy/math/bounding/trait.BoundedExtrusion.html)。默认实现会提供最优结果，但可能比适合你的图元的定制解要慢。

#### 网格化

拉伸体不仅仅存在于数学世界中。它们还可以被网格化并显示在屏幕上！

同样，为你的自定义图元添加网格化支持在 Bevy 中变得很容易！你只需要为你的 2D 图元实现网格化，然后为你的 2D 图元的 [`MeshBuilder`](https://docs.rs/bevy/0.14/bevy/prelude/trait.MeshBuilder.html) 实现 [`Extrudable`](https://docs.rs/bevy/0.14/bevy/render/mesh/trait.Extrudable.html)。

在实现 [`Extrudable`](https://docs.rs/bevy/0.14/bevy/render/mesh/trait.Extrudable.html) 时，你需要提供关于基础形状周长上的哪些线段应该平滑着色还是平面着色，以及哪些顶点属于这些周长线段的信息。

![一个 2D 心形图元及其拉伸体](https://bevy.org/news/bevy-0-14/heart_extrusion.jpg)

[`Extrudable`](https://docs.rs/bevy/0.14/bevy/render/mesh/trait.Extrudable.html) trait 允许你轻松为自定义图元的拉伸体实现网格化。当然，你也可以手动为拉伸体实现网格化。

如果你想看完整的实现，可以查看[自定义图元示例](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/math/custom_primitives.rs)。

## 更多 Gizmo [#](https://bevy.org/news/bevy-0-14/#more-gizmos)

作者：[mweatherley](https://github.com/mweatherley)、[Kanabenki](https://github.com/Kanabenki)、[MrGVSV](https://github.com/MrGVSV)、[solis-lumine-vorago](https://github.com/solis-lumine-vorago)、[alice-i-cecile](https://github.com/alice-i-cecile)

PR：[#12211](https://github.com/bevyengine/bevy/pull/12211)

Bevy 中的 Gizmo 允许开发者轻松绘制任意形状来帮助调试或创作内容，同时也可以可视化场景的特定属性，例如网格的 AABB。

在 0.14 中，[`bevy::gizmos`](https://docs.rs/bevy/0.14.0/bevy/gizmos/index.html) 添加了几个新的 gizmo：

#### 圆角框 Gizmo

圆角框和立方体非常适合可视化区域和碰撞体。

如果你将 `corner_radius` 或 `edge_radius` 设置为正值，角将向外圆角。但如果你提供负值，角将向内翻转弯曲。

![圆角 gizmo 长方体](https://bevy.org/news/bevy-0-14/gizmos_rounded_cuboid.jpg) ![圆角 gizmo 矩形](https://bevy.org/news/bevy-0-14/gizmos_rounded_rect.jpg)

#### 网格 Gizmo

新增了网格 gizmo 类型，包括 [`Gizmos::grid_2d`](https://docs.rs/bevy/0.14.0/bevy/gizmos/prelude/struct.Gizmos.html#method.grid_2d) 和 [`Gizmos::grid`](https://docs.rs/bevy/0.14.0/bevy/gizmos/prelude/struct.Gizmos.html#method.grid) 用于绘制 2D 或 3D 平面网格，以及 [`Gizmos::grid_3d`](https://docs.rs/bevy/0.14.0/bevy/gizmos/prelude/struct.Gizmos.html#method.grid_3d) 用于绘制 3D 网格。

每种网格类型都可以沿其轴进行倾斜、缩放和细分，并且你可以单独控制绘制哪些外边缘。

![网格 Gizmo 截图](https://bevy.org/news/bevy-0-14/grid_gizmos.jpg)

#### 坐标轴 Gizmo

新的 [`Gizmos::axes`](https://docs.rs/bevy/0.14.0/bevy/gizmos/prelude/struct.Gizmos.html#method.axes) 提供了一种简单的方法来显示任何对象的位置、方向和缩放信息，基于其 [`Transform`](https://docs.rs/bevy/0.14.0/bevy/prelude/struct.Transform.html) 加上一个基础大小。每个轴箭头的大小与提供的 [`Transform`](https://docs.rs/bevy/0.14.0/bevy/prelude/struct.Transform.html) 中相应轴的缩放成比例。

![坐标轴 Gizmo 截图](https://bevy.org/news/bevy-0-14/axes_gizmo.jpg)

#### 光源 Gizmo

新的 [`ShowLightGizmo`](https://docs.rs/bevy/0.14.0/bevy/gizmos/light/struct.ShowLightGizmo.html) 组件实现了一个持久化 gizmo，用于可视化 [`SpotLight`](https://docs.rs/bevy/0.14.0/bevy/pbr/struct.SpotLight.html)、[`PointLight`](https://docs.rs/bevy/0.14.0/bevy/pbr/struct.PointLight.html) 和 [`DirectionalLight`](https://docs.rs/bevy/0.14.0/bevy/pbr/struct.DirectionalLight.html) 等光源。大多数光源属性都由 gizmo 可视化表示，并且 gizmo 颜色可以设置为匹配光源实例或使用多种其他行为。

与其他持久化 gizmo 类似，[`ShowLightGizmo`](https://docs.rs/bevy/0.14.0/bevy/gizmos/light/struct.ShowLightGizmo.html) 可以通过 [`LightGizmoConfigGroup`](https://docs.rs/bevy/0.14.0/bevy/gizmos/light/struct.LightGizmoConfigGroup.html) 按实例或全局配置。

![光源 Gizmo 截图](https://bevy.org/news/bevy-0-14/light_gizmos.jpg)

## Gizmo 线条样式和连接点 [#](https://bevy.org/news/bevy-0-14/#gizmo-line-styles-and-joints)

作者：[lynn-lumen](https://github.com/lynn-lumen)

PR：[#12394](https://github.com/bevyengine/bevy/pull/12394)

以前的 Bevy 版本支持绘制线条 gizmo：

```rust
fn draw_gizmos(mut gizmos: Gizmos) {
    gizmos.line_2d(Vec2::ZERO, Vec2::splat(-80.), RED);
}
```

但是，自定义 gizmo 的唯一方法是更改它们的颜色，这对于某些用例来说可能具有局限性。此外，线条带中两条线的交汇点（即它们的_连接点_）存在小间隙。

从 Bevy 0.14 开始，你可以为每个 gizmo 配置组更改线条样式及其连接点：

```rust
fn draw_gizmos(mut gizmos: Gizmos) {
    gizmos.line_2d(Vec2::ZERO, Vec2::splat(-80.), RED);
}

fn setup(mut config_store: ResMut<GizmoConfigStore>) {
    // 获取你的 gizmo 配置组的配置
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    // 设置此配置组的线条样式和连接点
    config.line_style = GizmoLineStyle::Dotted;
    config.line_joints = GizmoLineJoint::Bevel;
}
```

新的线条样式可以在 2D 和 3D 中使用，并遵循其配置组的 `line_perspective` 选项。

可用的线条样式有：

- `GizmoLineStyle::Dotted`：绘制虚线，每个点都是一个正方形
- `GizmoLineStyle::Solid`：绘制实线——这是默认行为，也是 Bevy 0.14 之前唯一可用的样式

![新的 Gizmo 线条样式](https://bevy.org/news/bevy-0-14/gizmos_line_styles.jpg)

同样，新的线条连接点也提供了多种选项：

- `GizmoLineJoint::Miter`，将两条线延伸直到它们在一个共同的斜接点相交
- `GizmoLineJoint::Round(resolution)`，将近似一个圆弧填充两条线之间的间隙。`resolution` 决定用于近似弧线几何的三角形数量
- `GizmoLineJoint::Bevel`，用一条直线段连接两条连接线的末端
- `GizmoLineJoint::None`，不使用连接点并留下小间隙——这是默认行为，也是 Bevy 0.14 之前唯一可用的选项

![新的 Gizmo 线条连接点](https://bevy.org/news/bevy-0-14/gizmos_line_joints.jpg)

你可以查看 [2D gizmos 示例](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/gizmos/2d_gizmos.rs)，它演示了线条样式和连接点的使用！

## UI 节点轮廓 Gizmo [#](https://bevy.org/news/bevy-0-14/#ui-node-outline-gizmos)

作者：[pablo-lua](https://github.com/pablo-lua)、[nicopap](https://github.com/nicopap)、[alice-i-cecile](https://github.com/alice-i-cecile)

PR：[#11237](https://github.com/bevyengine/bevy/pull/11237)

在 Web 上使用 UI 时，能够快速调试所有盒子的大小非常有用。我们现在有一个原生[布局工具](https://docs.rs/bevy/0.14/bevy/dev_tools/ui_debug_overlay/struct.DebugUiPlugin.html)，可以为所有 [Node](https://docs.rs/bevy/0.14/bevy/ui/struct.Node.html) 添加 gizmo 轮廓

启用后工具的示例样子

![启用了叠加工具的 UI 示例](https://bevy.org/news/bevy-0-14/bevy_ui_outlines.jpg)

```rust
use bevy::prelude::*;

// 你首先需要将 DebugUiPlugin 添加到你的 app 中
let mut app = App::new()
    .add_plugins(bevy::dev_tools::ui_debug_overlay::DebugUiPlugin);

// 为了在运行时启用该工具，你可以添加一个系统来切换它
fn toggle_overlay(
    input: Res<ButtonInput<KeyCode>>,
    mut options: ResMut<bevy::dev_tools::ui_debug_overlay::UiDebugOptions>,
) {
    info_once!("调试轮廓已启用，按 Space 键打开/关闭");
    if input.just_pressed(KeyCode::Space) {
        // toggle 方法将启用 debug_overlay（如果已禁用）或禁用（如果已启用）
        options.toggle();
    }

}

// 将系统添加到 app
app.add_systems(Update, toggle_overlay);
```

## 按上下文清除 Gizmo [#](https://bevy.org/news/bevy-0-14/#contextually-clearing-gizmos)

作者：[Aceeri](https://github.com/Aceeri)

PR：[#10973](https://github.com/bevyengine/bevy/pull/10973)

Gizmo 通过即时模式 API 绘制。这意味着每次**更新**你都绘制所有想要显示的 gizmo，且只有这些会被显示。以前，更新指的是"每次 `Main` Schedule 运行时一次"。这与帧率匹配，所以通常效果很好！但当你尝试在 `FixedMain` 期间绘制 gizmo 时，它们会闪烁或被多次渲染。在 Bevy 0.14 中，这现在可以正常工作了！

这可以扩展用于自定义 Schedule。现在不再是单一的存储，而是有多个由上下文类型参数区分的[存储](https://docs.rs/bevy/0.14/bevy/gizmos/gizmos/struct.GizmoStorage.html)。你还可以在 [`Gizmos`](https://docs.rs/bevy/0.14/bevy/gizmos/gizmos/struct.Gizmos.html) System 参数上设置类型参数来选择写入哪个存储。你可以选择你添加的存储何时被绘制或清除：在 `Last` Schedule 期间，默认存储（`()` 上下文）中的任何 gizmo 都将被显示。

## Query Join [#](https://bevy.org/news/bevy-0-14/#query-joins)

作者：[hymm](https://github.com/hymm)

PR：[#11535](https://github.com/bevyengine/bevy/pull/11535)

ECS Query 现在可以组合起来，返回同时存在于两个查询中的实体的数据。

```rust
fn helper_function(a: &mut Query<&A>, b: &mut Query<&B>){    
    let a_and_b: QueryLens<(Entity, &A, &B)> = a.join(b);
    assert!(a_and_b.iter().len() <= a.len());
    assert!(a_and_b.iter().len() <= b.len());
}
```

在大多数情况下，你应该继续简单地给原始查询添加更多参数。`Query<&A, &B>` 通常比后面再 join 它们更清晰。但是当复杂的 System 或辅助函数把你逼入困境时，query join 就是你的后盾。

如果你熟悉数据库术语，这相当于一个["内连接"](https://www.w3schools.com/sql/sql_join.asp)。其他类型的 query join 正在考虑中。也许你可以尝试一下[后续 issue](https://github.com/bevyengine/bevy/issues/13633)？

## 计算状态和子状态 [#](https://bevy.org/news/bevy-0-14/#computed-states-sub-states)

作者：[lee-orr](https://github.com/lee-orr)、[marcelchampagne](https://github.com/marcelchampagne)、[MiniaczQ](https://github.com/MiniaczQ)、[alice-i-cecile](https://github.com/alice-i-cecile)

PR：[#11426](https://github.com/bevyengine/bevy/pull/11426)

Bevy 的 [`States`](https://docs.rs/bevy/0.14/bevy/prelude/trait.States.html) 是一个简单但强大的抽象，用于管理你的应用的控制流。

但随着用户的游戏（以及非游戏应用！）复杂度的增长，它们的局限性变得越来越明显。如果我们想要表达"在菜单中"的概念，但又需要根据不同子菜单的打开状态而区分不同的状态呢？如果我们想要询问"游戏是否暂停"这样的问题，但这个问题只在游戏中才有意义呢？

为此找到一个好的抽象需要[几次](https://github.com/bevyengine/bevy/pull/9957)[尝试](https://github.com/bevyengine/bevy/pull/10088)以及大量的实验和讨论。

虽然你现有的 [`States`](https://docs.rs/bevy/0.14/bevy/prelude/trait.States.html) 代码将完全像以前一样工作，但现在你可以使用两个额外的工具来获得更强的表现力：**计算状态**和**子状态**。

让我们从一个简单的状态声明开始：

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

添加 `pause` 字段意味着简单地检查 `GameState::InGame` 不再管用了……状态会根据其值而不同，我们可能希望区分游戏暂停时和未暂停时运行的游戏系统！

#### 计算状态

虽然我们可以简单地使用 `OnEnter(GameState::InGame{paused: true})`，但我们需要能够推理"当我们在游戏中时，无论是否暂停"。为此，我们定义了 `InGame` 计算状态：

```rust
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct InGame;

impl ComputedStates for InGame {
    // 计算状态可以从一个或多个源状态计算得出。
    type SourceStates = GameState;

    // 现在，我们定义确定计算状态值的规则。
    fn compute(sources: GameState) -> Option<InGame> {
        match sources {
            // 我们可以使用模式匹配来表达"我不在乎游戏是否暂停"的逻辑！
            GameState::InGame {..} => Some(InGame),
            _ => None,
        }
    }
}
```

#### 子状态

相比之下，当你希望通过 `NextState` 手动控制值，但仍然将其存在绑定到某个父状态时，应该使用子状态。

```rust
#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
// 这个宏意味着 `GamePhase` 只有在我们处于 `InGame` 计算状态时才存在。
// 中间的计算状态在这里有助于清晰性，但不是必需的：
// 你可以手动 `impl SubStates` 以获得更多控制、多个父状态和非默认初始值！
#[source(InGame = InGame)]
enum GamePhase {
    #[default]
    Setup,
    Battle,
    Conclusion
}
```

#### 初始化

初始化我们的状态很容易：只需在 `App` 上调用适当的方法，所有必要的机制就会为你设置好。

```rust
App::new()
   .init_state::<GameState>()
   .add_computed_state::<InGame>()
   .add_sub_state::<GamePhase>()
```

就像任何其他状态一样，计算状态和子状态与你习惯的所有工具一起工作：`State` 和 `NextState` 资源、`OnEnter`、`OnExit` 和 `OnTransition` Schedule，以及 `in_state` 运行条件。请务必查看[这两个](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/state/computed_states.rs)[示例](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/state/sub_states.rs)以获取更多信息！

唯一的例外是，为了正确性，计算状态_不能_通过 `NextState` 改变。相反，它们严格地从其父状态派生；根据提供的 `compute` 方法，在状态转换期间自动添加、移除和更新。

所有 Bevy 的状态工具现在都位于一个专门的 `bevy_state` crate 中，可以通过功能标志进行控制。怀念状态栈的日子？希望有重新进入状态的方法？所有的状态机制_仅_依赖于公共的 ECS 工具：资源、Schedule 和运行条件，因此很容易在其上构建。我们知道状态机很大程度上取决于个人品味；所以如果你的口味不适合我们的设计，可以考虑利用 Bevy 的模块化特性，编写你自己的抽象或使用社区提供的抽象！

## 状态作用域实体 [#](https://bevy.org/news/bevy-0-14/#state-scoped-entities)

作者：[MiniaczQ](https://github.com/MiniaczQ)、[alice-i-cecile](https://github.com/alice-i-cecile)、[mockersf](https://github.com/mockersf)

PR：[#13649](https://github.com/bevyengine/bevy/pull/13649)

状态作用域实体是社区项目中自然涌现的一种模式。**Bevy 0.14** 已采纳了它！

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum GameState {
 #[default]
  Menu,
  InGame,
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        // 我们使用 `StateScoped` 组件标记这个实体。
        // 当提供的状态退出时，该实体将被递归删除及其所有子实体。
        StateScoped(GameState::InGame)
        SpriteBundle { ... }
    ))
}

App::new()
    .init_state::<GameState>()
    // 我们需要安装适当的机制以使清理代码运行，每种状态类型一次。
    .enable_state_scoped_entities::<GameState>()
    .add_systems(OnEnter(GameState::InGame), spawn_player);
```

通过在设置期间将实体生命周期绑定到状态，我们可以大幅减少需要编写的清理代码量！

## 状态自转换 [#](https://bevy.org/news/bevy-0-14/#state-identity-transitions)

作者：[MiniaczQ](https://github.com/MiniaczQ)、[alice-i-cecile](https://github.com/alice-i-cecile)

PR：[#13579](https://github.com/bevyengine/bevy/pull/13579)

用户有时要求我们在从状态转移到自身时触发退出和进入步骤。虽然这有其用途（刷新的核心思想），但在其他情况下可能会出乎意料且不受欢迎。我们找到了一个折衷方案，允许用户在需要时挂钩到这种类型的转换。

`StateEventTransition` 事件现在将包括从状态到自身的转换，这也将传播到所有依赖的 `ComputedStates` 和 `SubStates`。

由于这是一个小众功能，`OnExit` 和 `OnEnter` Schedule 默认将忽略新的自转换，但你可以查看新的 [`custom_transitions`](https://github.com/bevyengine/bevy/tree/v0.14.0/examples/state/custom_transitions.rs) 示例，了解如何绕过或更改该行为！

## GPU 视锥体裁剪 [#](https://bevy.org/news/bevy-0-14/#gpu-frustum-culling)

作者：[pcwalton](https://github.com/pcwalton)

PR：[#12889](https://github.com/bevyengine/bevy/pull/12889)

Bevy 的渲染栈通常是 CPU 受限的：通过将更多工作转移到 GPU，我们可以更好地平衡负载，更快地渲染更多闪亮的东西。视锥体裁剪是一种优化技术，可自动隐藏摄像机视野（其视锥体）之外的物体。在 Bevy 0.14 中，用户可以根据其项目的性能特征选择在 GPU 上执行此工作。

有两个新组件可用于控制视锥体裁剪：`GpuCulling` 和 `NoCpuCulling`。将这些组件的适当组合附加到摄像机，就设置好了。

```rust
commands.spawn((
    Camera3dBundle::default(),
    // 启用 GPU 视锥体裁剪（不会自动禁用 CPU 视锥体裁剪）。
    GpuCulling,
    // 禁用 CPU 视锥体裁剪。
    NoCpuCulling
));
```

## World 命令队列 [#](https://bevy.org/news/bevy-0-14/#world-command-queue)

作者：[james7132](https://github.com/james7132)、[james-j-obrien](https://github.com/james-j-obrien)

PR：[#11823](https://github.com/bevyengine/bevy/pull/11823)

当你拥有独占 World 访问权限时使用 [`Commands`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Commands.html) 一直以来都很痛苦。创建一个 [`CommandQueue`](https://docs.rs/bevy/0.14/bevy/ecs/world/struct.CommandQueue.html)，从中生成一个 [`Commands`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Commands.html)，发送命令然后应用它？并非最直观的解决方案。

现在，你可以访问 [`World`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.World.html) 自己的命令队列：

```rust
let mut world = World::new();
let mut commands = world.commands();
commands.spawn(TestComponent);
world.flush_commands();
```

虽然这不是最高性能的方法（直接将变更应用到 World 并跳过间接层），但这个 API 非常适合快速原型设计或轻松测试自定义命令。它也在内部用于驱动组件生命周期 hooks 和 observers。

额外的好处是，一次性系统现在在运行时立即应用它们的命令（和其他延迟的 System 参数）！我们已经拥有独占 World 访问权限：为什么要引入延迟和细微的错误？

## 减少多线程执行开销 [#](https://bevy.org/news/bevy-0-14/#reduced-multi-threaded-execution-overhead)

作者：[chescock](https://github.com/chescock)、[james7132](https://github.com/james7132)

PR：[#11906](https://github.com/bevyengine/bevy/pull/11906)

Bevy 多线程 System 执行器中最大的开销来自[线程上下文切换](https://en.wikipedia.org/wiki/Context_switch)，即启动和停止线程。每次唤醒线程时，如果线程缓存是冷的，可能花费多达 30us。最小化这些切换是执行器的重要优化。在这个周期中，我们落地了两个对此有改进的更改：

#### 在每个 System 任务结束时运行多线程执行器

System 执行器负责检查 System 的依赖是否已运行、评估运行条件，然后为该 System 运行任务。旧版本的多线程执行器作为一个独立任务运行，在每个任务完成后被唤醒。这有时会导致为执行器处理 System 完成而唤醒一个新线程。

通过更改使得 System 任务在每个 System 完成后尝试运行多线程执行器，我们确保多线程执行器始终在已经唤醒的线程上运行。这防止了一个上下文切换源。实际上，这将每次 `Schedule` 运行的上下文切换次数减少了 1-3 倍，每个 Schedule 改进约 30us。当一个应用有很多 Schedule 时，这可以累加起来！

#### 合并事件更新 System

以前每种事件类型都有一个"事件更新 System"实例。仅使用 `DefaultPlugins`，就会产生 20 多个 System 实例。

每个实例运行得非常快，因此生成 System 任务和唤醒线程以运行所有这些 System 的开销主导了 `First` Schedule 运行所需的时间。因此，将所有这些问题合并到一个 System 中可以避免这种开销，并使 `First` Schedule 运行得更快。在测试中，这使得 Schedule 运行从 140us 降到了 25us。同样，这不是一个_巨大_的胜利，但我们致力于节省每一微秒！

## 将 `BackgroundColor` 从 `UiImage` 解耦 [#](https://bevy.org/news/bevy-0-14/#decouple-backgroundcolor-from-uiimage)

作者：[benfrankel](https://github.com/benfrankel)

PR：[#11165](https://github.com/bevyengine/bevy/pull/11165)

UI 图像现在可以拥有纯色背景：

![带有背景颜色的 UI 图像](https://bevy.org/news/bevy-0-14/ui_image_background_color.jpg)

[`BackgroundColor`](https://docs.rs/bevy/0.14/bevy/prelude/struct.BackgroundColor.html) 组件现在适用于 UI 图像，而不是在图像本身上应用颜色色调。你仍然可以通过设置 `UiImage::color` 来应用颜色色调。例如：

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

## 合并的 WinitEvent [#](https://bevy.org/news/bevy-0-14/#combined-winitevent)

作者：[UkoeHB](https://github.com/UkoeHB)

PR：[#12100](https://github.com/bevyengine/bevy/pull/12100)

在处理输入时，接收事件的精确顺序往往非常重要，即使事件不是同一类型！考虑一个简单的拖放操作。用户释放鼠标按钮的时间相对于他们执行的许多微小移动来说，究竟是什么时候？把这些细节做对对于获得响应迅速、精确的用户体验大有帮助。

除了现有的分离事件流之外，我们现在还暴露了全面的 [`WinitEvent`](https://docs.rs/bevy/0.14/bevy/winit/enum.WinitEvent.html) 事件流，每当出现这些问题时，可以直接读取和匹配。

## 递归 Reflect 注册 [#](https://bevy.org/news/bevy-0-14/#recursive-reflect-registration)

作者：[MrGVSV](https://github.com/MrGVSV)、[soqb](https://github.com/soqb)、[cart](https://github.com/cart)、[james7132](https://github.com/james7132)

PR：[#5781](https://github.com/bevyengine/bevy/pull/5781)

Bevy 使用[反射](https://docs.rs/bevy_reflect/latest/bevy_reflect/)来动态处理数据，例如序列化和反序列化。Bevy 应用有一个 `TypeRegistry` 来跟踪存在哪些类型。用户可以在初始化应用或插件时注册自定义类型。

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

在上面的代码中，`Data<Blob>` 依赖 `Blob`，而 `Blob` 依赖 `Vec<u8>`，这意味着所有三种类型都需要手动注册——即使我们只关心 `Data<Blob>`。

这既繁琐又容易出错，特别是当这些类型依赖只在其他类型的上下文中使用时（即它们不作为独立类型使用）。

在 0.14 中，任何派生 `Reflect` 的类型都会自动注册其所有类型依赖。因此当我们注册 `Data<Blob>` 时，`Blob` 也会被注册（进而注册 `Vec<u8>`），从而将我们的注册简化为一行：

```rust
app.register_type::<Data<Blob>>()
```

请注意，删除对 `Data<Blob>` 的注册现在也意味着 `Blob` 和 `Vec<u8>` 可能也不会被注册，除非它们以其他方式注册。如果这些类型需要作为独立类型使用，应单独注册。

## 2D 旋转的 `Rot2` 类型 [#](https://bevy.org/news/bevy-0-14/#rot2-type-for-2d-rotations)

作者：[Jondolf](https://github.com/Jondolf)、[IQuick143](https://github.com/IQuick143)、[tguichaoua](https://github.com/tguichaoua)

PR：[#11658](https://github.com/bevyengine/bevy/pull/11658)

曾经想在 2D 中处理旋转却因不得不在四元数和原始 `f32` 之间选择而感到沮丧吗？我们也是！

我们为你添加了一个便捷的 [`Rot2`](https://docs.rs/bevy/0.14/bevy/math/struct.Rot2.html) 类型，带有大量辅助方法。随时替换你编写的辅助类型，并为我们缺少的任何有用功能提交小 PR。

[`Rot2`](https://docs.rs/bevy/0.14/bevy/math/struct.Rot2.html) 是对 [`Dir2`](https://docs.rs/bevy/0.14/bevy/math/struct.Dir2.html) 类型（原名 `Direction2d`）的很好补充。前者表示角度，而后者是单位向量。这些类型相似但不可互换，表示方式的选择在很大程度上取决于手头的任务。你可以使用 `direction = rotation * Dir2::X` 旋转方向。要恢复旋转，使用 `Dir2::X::rotation_to(direction)` 或在这个情况下使用辅助方法 `Dir2::rotation_from_x(direction)`。

虽然这些类型尚未在引擎内广泛使用，但我们_确实_意识到你的痛苦，并正在评估[提案](https://github.com/bevyengine/rfcs/pull/82)，探讨如何使在 2D 中处理变换更加直接和愉快。

## 变换的对齐 API [#](https://bevy.org/news/bevy-0-14/#alignment-api-for-transforms)

作者：[mweatherley](https://github.com/mweatherley)

PR：[#12187](https://github.com/bevyengine/bevy/pull/12187)

**Bevy 0.14** 添加了一个新的 [`Transform::align`](https://docs.rs/bevy/0.14/bevy/transform/components/struct.Transform.html#method.align) 函数，它是 [`Transform::look_to`](https://docs.rs/bevy/0.14/bevy/transform/components/struct.Transform.html#method.look_to) 的更通用形式，允许你为轴和副轴指定任何想要的局部轴。

这允许你做一些事情，比如将飞船的前端指向你前往的行星，同时保持右翼指向另一艘飞船的方向。或者将飞船的顶部指向牵引光束拉拽的方向，同时前端旋转以匹配更大飞船的方向。

让我们考虑一艘飞船，我们将使用飞船的前端和右翼作为局部轴：

![before calling Transform::align](https://bevy.org/news/bevy-0-14/align-before-move.jpg)

```rust
// point the local negative-z axis in the global Y axis direction
// point the local x-axis in the global Z axis direction
transform.align(Vec3::NEG_Z, Vec3::Y, Vec3::X, Vec3::Z)
```

`align` will move it to match the desired positions as closely as possible:

![after calling Transform::align](https://bevy.org/news/bevy-0-14/align-after-move.jpg)

请注意，并非所有旋转都能构造，[文档](https://docs.rs/bevy/0.14/bevy/transform/components/struct.Transform.html#method.align)解释了在此类情况下会发生什么。

## 形状和方向的随机采样 [#](https://bevy.org/news/bevy-0-14/#random-sampling-of-shapes-and-directions)

作者：[13ros27](https://github.com/13ros27)、[mweatherley](https://github.com/mweatherley)、[lynn-lumen](https://github.com/lynn-lumen)

PR：[#12484](https://github.com/bevyengine/bevy/pull/12484)

在游戏开发的背景下，访问随机值通常很有帮助，无论是为了驱动 NPC 的行为、创建特效，还是仅仅为了创造多样性。为了支持这一点，`bevy_math` 添加了一些随机采样功能，通过 `rand` 功能标志控制。这些功能主要与几何相关，并且有几种形式。

首先，可以从各种数学图元的边界和内部采样随机点：

在代码中，可以通过几种不同的方式执行，使用 `sample_interior`/`sample_boundary` 或 `interior_dist`/`boundary_dist` API：

```rust
use bevy::math::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

let sphere = Sphere::new(1.5);

// 实例化一个 Rng：
let rng = &mut ChaCha8Rng::seed_from_u64(7355608);

// 使用这些方法，从球体内部采样一个随机点：
let interior_pt: Vec3 = sphere.sample_interior(rng);
// 或者从边界采样：
let boundary_pt: Vec3 = sphere.sample_boundary(rng);

// 或者，如果我们想要大量点，可以使用 Distribution 代替……
// 从内部采样 100000 个随机点：
let interior_pts: Vec<Vec3> = sphere.interior_dist().sample_iter(rng).take(100000).collect();
// 从边界采样 100000 个随机点：
let boundary_pts: Vec<Vec3> = sphere.boundary_dist().sample_iter(rng).take(100000).collect();
```

请注意，这些方法显式要求一个 [`Rng`](https://docs.rs/rand/0.8.5/rand/trait.Rng.html) 对象，让你可以控制随机化策略和种子。

当前支持的形状如下：

2D：`Circle`、`Rectangle`、`Triangle2d`、`Annulus`、`Capsule2d`。

3D：`Sphere`、`Cuboid`、`Triangle3d`、`Tetrahedron`、`Cylinder`、`Capsule3d`，以及可采样 2D 形状的拉伸体（`Extrusion`）。

---

类似地，方向类型（`Dir2`、`Dir3`、`Dir3A`）和四元数（`Quat`）现在可以使用 `from_rng` 随机构造：

```rust
use bevy::math::prelude::*;
use rand::{random, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

// 实例化一个 Rng：
let rng = &mut ChaCha8Rng::seed_from_u64(7355608);

// 获取一个随机方向：
let direction = Dir3::from_rng(rng);

// 类似，但需要左侧类型注解或推断：
let another_direction: Dir3 = rng.gen();

// 使用 `random` 通过隐式的线程局部 rng 获取值：
let yet_another_direction: Dir3 = random();
```

## 分析 GPU 性能的工具 [#](https://bevy.org/news/bevy-0-14/#tools-for-profiling-gpu-performance)

作者：[@LeshaInc](https://github.com/LeshaInc)

PR：[#9135](https://github.com/bevyengine/bevy/pull/9135)

虽然 [Tracy](https://github.com/bevyengine/bevy/blob/main/docs/profiling.md) 已经可以让我们测量每个系统的 CPU 时间，但我们的 GPU 诊断能力要弱得多。在 Bevy 0.14 中，我们通过 [`RenderDiagnosticsPlugin`](https://docs.rs/bevy/0.14/bevy/render/diagnostic/struct.RenderDiagnosticsPlugin.html) 增加了对两类渲染相关统计数据的支持：

1. **时间戳查询（Timestamp queries）：** GPU 上特定工作花费了多长时间？
2. **管线统计（Pipeline statistics）：** 发送到 GPU 的工作量的信息。

虽然时间戳查询听起来像是终极诊断工具，但它们有几个注意事项。首先，它们的帧间变化非常大，因为 GPU 会根据工作负载（GPU 工作中的空闲间隙，例如一连串连续的屏障，或大型调度的尾部）或 GPU 的物理温度动态调整时钟频率。要获得准确的测量结果，你需要查看汇总统计数据：平均值、中位数、第 75 百分位数等等。

其次，虽然时间戳查询会告诉你某个操作花费了多长时间，但它不会告诉你为什么会慢。要查找瓶颈，你需要使用 GPU 供应商提供的 GPU 分析器（Nvidia 的 NSight、AMD 的 RGP、Intel 的 GPA 或 Apple 的 XCode）。这些工具会为你提供关于缓存命中率、线程束占用率等更详细的统计数据。另一方面，它们会将 GPU 的时钟锁定在基本频率以获得稳定的结果，因此它们无法很好地反映实际性能。

[`RenderDiagnosticsPlugin`](https://docs.rs/bevy/0.14/bevy/render/diagnostic/struct.RenderDiagnosticsPlugin.html) 跟踪以下管线统计数据，记录在 Bevy 的 [`DiagnosticsStore`](https://docs.rs/bevy/0.14/bevy/diagnostic/struct.DiagnosticsStore.html) 中：CPU 运行时间、GPU 运行时间、[顶点着色器](https://www.khronos.org/opengl/wiki/Vertex_Shader)调用次数、[片段着色器](https://www.khronos.org/opengl/wiki/Fragment_Shader)调用次数、[计算着色器](https://www.khronos.org/opengl/wiki/Compute_Shader)调用次数、[裁剪器调用次数](http://gpa.helpmax.net/en/intel-graphics-performance-analyzers-help/metrics-descriptions/extended-metrics-description/rasterizer-metrics/clipper-invocations/)和[裁剪器图元数](http://gpa.helpmax.net/en/intel-graphics-performance-analyzers-help/metrics-descriptions/extended-metrics-description/rasterizer-metrics/post-clip-primitives/)。

你还可以跟踪单个渲染/计算 pass、pass 组（例如所有阴影 pass）以及 pass 内部的单个命令（如绘制调用）。为此，请使用 [`RecordDiagnostics`](https://docs.rs/bevy/0.14/bevy/render/diagnostic/trait.RecordDiagnostics.html) trait 中的方法进行检测。

## 新几何图元 [#](https://bevy.org/news/bevy-0-14/#new-geometric-primitives)

作者：[@vitorfhc](https://github.com/vitorfhc), [@Chubercik](https://github.com/Chubercik), [@andristarr](https://github.com/andristarr), [@spectria-limina](https://github.com/spectria-limina), [@salvadorcarvalhinho](https://github.com/salvadorcarvalhinho), [@aristaeus](https://github.com/aristaeus), [@mweatherley](https://github.com/mweatherley)

PR：[#12508](https://github.com/bevyengine/bevy/pull/12508)

几何形状在游戏开发中有多种应用，从将简单图形渲染到屏幕上进行显示/调试，到用于碰撞体、物理、光线投射等等。

为此，几何形状图元在 [Bevy 0.13 中引入](https://bevy.org/news/bevy-0-13/#primitive-shapes) 后，这一领域的工作在 Bevy 0.14 中继续进行，新增了 [`Triangle3d`](https://docs.rs/bevy/0.14.0/bevy/math/primitives/struct.Triangle3d.html) 和 [`Tetrahedron`](https://docs.rs/bevy/0.14.0/bevy/math/primitives/struct.Tetrahedron.html) 3D 图元，以及 [`Rhombus`](https://docs.rs/bevy/0.14.0/bevy/math/primitives/struct.Rhombus.html)、[`Annulus`](https://docs.rs/bevy/0.14.0/bevy/math/primitives/struct.Annulus.html)、[`Arc2d`](https://docs.rs/bevy/0.14.0/bevy/math/primitives/struct.Arc2d.html)、[`CircularSegment`](https://docs.rs/bevy/0.14.0/bevy/math/primitives/struct.CircularSegment.html) 和 [`CircularSector`](https://docs.rs/bevy/0.14.0/bevy/math/primitives/struct.CircularSector.html) 2D 图元。和往常一样，它们都有查询几何信息的方法，如周长、面积和体积，并且都支持网格化（在适用的情况下）以及 gizmo 显示。

## 改进 `Point` 并重命名为 `VectorSpace` [#](https://bevy.org/news/bevy-0-14/#improve-point-and-rename-it-to-vectorspace)

作者：[@mweatherley](https://github.com/mweatherley), [@bushrat011899](https://github.com/bushrat011899), [@JohnTheCoolingFan](https://github.com/JohnTheCoolingFan), [@NthTensor](https://github.com/NthTensor), [@IQuick143](https://github.com/IQuick143), [@alice-i-cecile](https://github.com/alice-i-cecile)

PR：[#12747](https://github.com/bevyengine/bevy/pull/12747)

线性代数在游戏中无处不在，我们希望确保它能被正确且轻松地使用。这就是为什么我们新增了一个 `VectorSpace` trait，作为使 `bevy_math` 更通用、更具表现力和数学上更健全的工作的一部分。任何实现 `VectorSpace` 的类型都表现得像一个向量。更正式地说，该 trait 要求实现满足向量加法和标量乘法的向量空间公理。我们还添加了一个 `NormedVectorSpace` trait，它包含距离和大小的 API。

这些 trait 支撑着新的曲线和形状采样 API。`VectorSpace` 为 `f32`、`glam` 向量类型以及几种新的颜色空间类型实现。它完全取代了 `bevy_math::Point`。

Bevy 中的样条模块长期以来一直缺少一些功能。样条在游戏开发中非常有用，因此改进它们将会改善所有使用它们的功能。

最大的新增功能是 NURBS 支持！它是 B 样条的一种变体，具有更多可调参数来创建特定的曲线形状。我们还添加了 `LinearSpline`，可用于在曲线中放置直线段。`CubicCurve` 现在作为一系列曲线段，你可以向其添加新片段，从而将各种样条类型混合在一起形成一条路径。

## 2D 网格线框 [#](https://bevy.org/news/bevy-0-14/#2d-mesh-wireframes)

作者：[@msvbg](https://github.com/msvbg), [@IceSentry](https://github.com/IceSentry)

PR：[#12135](https://github.com/bevyengine/bevy/pull/12135)

线框材质用于渲染网格的各个边缘和面。它们通常用作可视化几何体的调试工具，但也可用于各种风格化效果。Bevy 支持将 3D 网格显示为线框，但直到现在才支持 2D 网格。

要将 2D 网格渲染为线框，将 `Wireframe2dPlugin` 添加到你的应用，并将 `Wireframe2d` 组件添加到你的精灵。线框的颜色可以通过添加 `Wireframe2dColor` 组件进行逐对象配置，或通过插入 `Wireframe2dConfig` 资源进行全局配置。

关于如何使用该功能的示例，请查看新的 [wireframe_2d 示例](https://github.com/bevyengine/bevy/blob/b17292f9d11cf3d3fb4a2fb3e3324fb80afd8c88/examples/2d/wireframe_2d.rs)：

![展示新 2D 线框材质的截图](https://bevy.org/news/bevy-0-14/12135_Support_wireframes_for_2D_meshes.jpg)

## 自定义 Reflect 字段属性 [#](https://bevy.org/news/bevy-0-14/#custom-reflect-field-attributes)

作者：[@MrGVSV](https://github.com/MrGVSV)

PR：[#11659](https://github.com/bevyengine/bevy/pull/11659)

Bevy 反射系统的特性之一是将任意"类型数据"附加到类型上的能力。这最常用于允许 trait 方法被动态调用。然而，一些用户将其视为做其他很棒的事情的机会。

出色的 [bevy-inspector-egui](https://github.com/jakobhellermann/bevy-inspector-egui) 利用类型数据实现了很好的效果，允许用户逐字段配置其检查器 UI：

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

受此启发，Bevy 0.14 在派生 `Reflect` 时增加了对自定义属性的完善支持，用户和第三方 crate 不再需要专门为此创建自定义类型数据。这些属性可以使用 `#[reflect(@...)]` 语法附加到结构体、枚举、字段和变体上，其中 `...` 可以是任何解析为实现了 `Reflect` 的类型的表达式。

例如，我们可以使用 Rust 内置的 `RangeInclusive` 类型来为字段指定自己的范围：

```rust
use std::ops::RangeInclusive;
use bevy_reflect::Reflect;

#[derive(Reflect, Default)]
struct Slider {
    #[reflect(@RangeInclusive<f32>::new(0.0, 1.0))]
    // 由于这接受任何表达式，
    // 我们也可以使用 Rust 的简写语法：
    // #[reflect(@0.0..=1.0_f32)]
    value: f32,
}
```

然后可以使用 [`TypeInfo`](https://docs.rs/bevy/latest/bevy/reflect/enum.TypeInfo.html) 动态访问属性：

```rust
let TypeInfo::Struct(type_info) = Slider::type_info() else {
    panic!("expected struct");
};

let field = type_info.field("value").unwrap();

let range = field.get_attribute::<RangeInclusive<f32>>().unwrap();
assert_eq!(*range, 0.0..=1.0);
```

这个特性为基于 Bevy 反射系统构建的功能打开了大量可能性。通过使其不局限于任何特定用途，它支持广泛的使用场景，包括为未来的编辑器工作提供助力。

事实上，这个特性已经被 [`bevy_reactor`](https://github.com/viridia/bevy_reactor/blob/main/examples/complex/reflect_demo.rs) 用来驱动其自定义检查器 UI：

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

![使用上述代码在 bevy_reactor 中构建的自定义 UI 检查器](https://bevy.org/news/bevy-0-14/custom_attributes_demo.jpg)

## 查询迭代排序 [#](https://bevy.org/news/bevy-0-14/#query-iteration-sorting)

作者：[@Victoronz](https://github.com/Victoronz)

PR：[#13417](https://github.com/bevyengine/bevy/pull/13417)

Bevy 不对项目的顺序做任何保证。因此如果我们希望按一定顺序处理查询项目，就需要对它们进行排序！我们可能希望按顺序显示玩家的分数，或者为确保网络稳定性而保证一致的迭代顺序。在 0.13 中，排序可能看起来像这样：

```rust
#[derive(Component, Copy, Clone, Deref)]
pub struct Attack(pub usize)

fn handle_enemies(enemies: Query<(&Health, &Attack, &Defense)>) {
    // 一次分配！
    let mut enemies: Vec<_> = enemies.iter().collect();
    enemies.sort_by_key(|(_, atk, ..)| *atk)
    for enemy in enemies {
        work_with(enemy)
    }
}
```

在多个系统中进行排序时，这会变得特别笨重和重复。即使我们总是想要相同的排序，不同的 [`Query`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Query.html) 类型也使得用户很难抽象出来！为了解决这个问题，我们在 [`QueryIter`](https://docs.rs/bevy/0.14/bevy/ecs/query/struct.QueryIter.html) 类型上实现了新的排序方法，将示例转变为：

```rust
// 用作排序键，`Attack` 现在实现了 Ord。
#[derive(Component, Copy, Clone, Deref, PartialEq, Eq, PartialOrd, Ord)]
pub struct Attack(pub usize)

fn handle_enemies(enemies: Query<(&Health, &Attack, &Defense)>) {
    // 仍然是分配，但隐藏起来了。
    for enemy in enemies.iter().sort::<&Attack>() {
        work_with(enemy)
    }
}
```

要使用 `Attack` 组件对查询进行排序，我们将其指定为 [`sort`](https://docs.rs/bevy/0.14/bevy/ecs/query/struct.QueryIter.html?search=Component#method.sort) 的泛型参数。要按多个 [`Component`](https://docs.rs/bevy/0.14/bevy/ecs/component/trait.Component.html) 排序，我们可以这样做，与它们在原始 [`Query`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Query.html) 类型中的顺序无关：`enemies.iter().sort::<(&Defense, &Attack)>()`

泛型参数可以被视为原始查询的 [lens](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Query.html#method.transmute_lens) 或"子集"，实际的排序在其上执行。然后内部使用结果返回一个包含原始查询项目的排序后的查询迭代器。使用默认的 [`sort`](https://docs.rs/bevy/0.14/bevy/ecs/query/struct.QueryIter.html?search=Component#method.sort)，lens 必须完全是 [`Ord`](https://doc.rust-lang.org/stable/std/cmp/trait.Ord.html)，就像 [`slice::sort`](https://doc.rust-lang.org/nightly/std/primitive.slice.html#method.sort) 一样。如果这还不够，我们还有来自 [`slice`](https://doc.rust-lang.org/nightly/std/primitive.slice.html) 的其他 6 种排序方法的对应版本！

泛型 lens 参数的工作方式与 [`Query::transmute_lens`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Query.html#method.transmute_lens) 相同。我们不使用过滤器，它们继承自原始查询。[`transmute_lens`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Query.html#method.transmute_lens) 基础设施有一些很好的额外特性，可以实现这样的效果：

```rust
fn handle_enemies(enemies: Query<(&Health, &Attack, &Defense, &Rarity)>) {
    for enemy in enemies.iter().sort_unstable::<Entity>() {
        work_with(enemy)
    }
}
```

因为我们可以将 [`Entity`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Entity.html) 添加到任何 lens，所以无需在原始查询中包含它就能按它排序！

这些排序方法同时适用于 [`Query::iter`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Query.html#method.iter) 和 [`Query::iter_mut`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Query.html#method.iter_mut)！[`Query`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.Query.html) 上其余迭代器方法目前不支持排序。排序返回 [`QuerySortedIter`](https://docs.rs/bevy/0.14/bevy/ecs/query/struct.QuerySortedIter.html)，它本身也是一个迭代器，允许在其上使用进一步的迭代器适配器。

请记住，lensing 确实会增加一些开销，因此这些查询迭代器排序的平均性能不如手动排序。然而，这在很大程度上取决于工作负载，所以如果相关的话最好自己测试！

## SystemBuilder [#](https://bevy.org/news/bevy-0-14/#systembuilder)

作者：[@james-j-obrien](https://github.com/james-j-obrien)

PR：[#13123](https://github.com/bevyengine/bevy/pull/13123)

Bevy 用户_热爱_系统，所以我们为他们的系统做了一个构建器，让他们可以在系统内部构建系统。在运行时使用动态定义的组件和资源类型！

虽然你可以使用 [`SystemBuilder`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.SystemBuilder.html) 作为 [`SystemState`](https://docs.rs/bevy/0.14/bevy/ecs/system/struct.SystemState.html) API 的人体工学替代方案来将 [`World`](https://docs.rs/bevy/0.14/bevy/ecs/prelude/struct.World.html) 拆分为不相交的借用，但它真正的价值在于其动态使用。

你可以选择基于运行时分支创建不同的系统，或者更有趣的是，查询等可以使用运行时定义的组件 ID。这是朝着创建处理[动态查询](https://bevy.org/news/bevy-0-13/#dynamic-queries)的人体工学且安全的 API 迈出的又一关键步骤，为想要集成脚本语言或为游戏提供复杂 Mod 支持的开发者奠定了基础。

```rust
// 首先从 world 创建构建器
let system = SystemBuilder::<()>::new(&mut world)
    // 有多种辅助方法可用于添加 `SystemParam`
    .resource::<R>()
    .query::<&A>()
    // 或者为其他 `SystemParam` 类型使用 `.param::<T>()`
    .param::<MyParam>()
    // 最后调用 `.build` 完成
    .build(my_system);
// 构建器初始化时的参数将首先出现在参数中
let system = SystemBuilder::<(Res<R>, Query<&A>)>::new(&mut world)
    .param::<MyParam>()
    .build(my_system);
// 实现了 `BuildableSystemParam` 的参数（如 `Query`）可以使用
// `.builder::<T>()` 就地构建
let system = SystemBuilder::<()>::new(&mut world)
    .resource::<R>()
    // 这将我们的查询转换为 `Query<&A, With<B>>`
    .builder::<Query<&A>>(|builder| { builder.with::<B>(); })
    .param::<MyParam>()
    .build(my_system);
world.run_system_once(system);
```

## 节流渲染资源上传 [#](https://bevy.org/news/bevy-0-14/#throttle-render-assets)

作者：[@robtfm](https://github.com/robtfm), [@IceSentry](https://github.com/IceSentry), [@mockersf](https://github.com/mockersf)

PR：[#12622](https://github.com/bevyengine/bevy/pull/12622)

使用了大量资源？在短时间内上传大量字节到 GPU 可能会导致卡顿，因为渲染世界需要等待上传完成。

通常，应用程序运行流畅比卡顿的体验要好得多，而且看到资源出现之前几帧的延迟通常是不可感知的。

这种体验现在已经成为可能：

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(RenderAssetBytesPerFrame::new(1_000_000_000)) // 通过实验调整到适合你的情况！
        .run();
}
```

就是这样！提供的数字应该通过在不卡顿和可接受的延迟之间找到良好的平衡来选择。

此功能依赖于资源知道它们发送到 GPU 时将占用多少字节。目前图像和网格知道这一点，预计未来更多资源类型也能报告这一信息。

## StandardMaterial UV 通道选择 [#](https://bevy.org/news/bevy-0-14/#standardmaterial-uv-channel-selection)

作者：[@geckoxx](https://github.com/geckoxx)

PR：[#13200](https://github.com/bevyengine/bevy/pull/13200)

以前，StandardMaterial 默认对所有纹理（光照贴图除外）使用 ATTRIBUTE_UV_0，这对于很多 glTF 文件来说不够灵活。在 **Bevy 0.14** 中，新增了一个 UvChannel 枚举，允许你为 StandardMaterial 中的每个纹理选择要使用的通道。

以下展示了各纹理对 ATTRIBUTE_UV_1 支持的前后对比：

![UV 通道选择](https://bevy.org/news/bevy-0-14/uv_channel_selection.jpg)

## 移除 RenderLayers 上限 [#](https://bevy.org/news/bevy-0-14/#remove-limit-on-renderlayers)

作者：[@tychedelia](https://github.com/tychedelia), [@robtfm](https://github.com/robtfm), [@UkoeHB](https://github.com/UkoeHB)

PR：[#13317](https://github.com/bevyengine/bevy/pull/13317)

渲染层用于快速切换对象集合的可见性，并控制哪些对象可以被哪些相机看到。这对于调试视图、装备预览屏幕、可切换的界面内 UI 等非常有用。

在 Bevy 0.14 之前，成员身份由位掩码定义，可用的插槽有限。现在，你可以定义的层数不再有实际限制，这对于像 [nannou](https://nannou.cc/) 这样的创意编码应用特别有用！我们确保保持常见用例的性能，但现在使用可增长的掩码，会根据需要为额外层分配空间。请记住，检查每层的可见性仍然有成本，但这允许更动态的使用场景，可以按需创建层而无需担心超过限制。

## `on_unimplemented` 诊断信息 [#](https://bevy.org/news/bevy-0-14/#on-unimplemented-diagnostics)

作者：[@bushrat011899](https://github.com/bushrat011899), [@alice-i-cecile](https://github.com/alice-i-cecile), [@Themayu](https://github.com/Themayu)

PR：[#13347](https://github.com/bevyengine/bevy/pull/13347)

Bevy 充分利用了 Rust 提供的强大类型系统，但伴随这种强大力量而来的是，即使是微小的错误也可能造成困惑。

```rust
use bevy::prelude::*;

struct MyResource;

fn main() {
    App::new()
        .insert_resource(MyResource)
        .run();
}
```

运行以上代码会产生一个编译器错误，来看看为什么……

点击展开……

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

编译器建议我们使用一个实现了 `Resource` 的不同类型，或者在 `MyResource` 上实现该 trait。前者对我们完全没有帮助，而后者未能提及可用的派生宏。

随着 Rust 1.78 的发布，Bevy 现在可以使用[诊断属性](https://blog.rust-lang.org/2024/05/02/Rust-1.78.0.html#diagnostic-attributes)在编译期间为某些类型的错误提供更直接的提示信息。

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

现在，错误消息有了更友好的入口点，并且新增了一个 `note` 部分指向资源的派生宏。如果 Bevy 的建议_并非_你的问题的解决方案，编译器错误的其余部分仍然包含在内，以防万一。

这些诊断已在 Bevy 的各种 trait 上实现，我们希望随着 Rust 新增功能而不断改进这一体验。例如，我们非常想改善处理 `Component` 元组的体验，但尚未实现。你可以在[拉取请求](https://github.com/bevyengine/bevy/pull/13347)和相关[问题](https://github.com/bevyengine/bevy/issues/12377)中阅读更多关于此更改的信息。

## 为动画网格添加运动矢量和 TAA 支持 [#](https://bevy.org/news/bevy-0-14/#motion-vectors-and-taa-for-animated-meshes)

作者：[@pcwalton](https://github.com/pcwalton)

PR：[#13572](https://github.com/bevyengine/bevy/pull/13572)

早在 **Bevy 0.11** 中，我们添加了[时间性抗锯齿（TAA）](https://bevy.org/news/bevy-0-11/#temporal-anti-aliasing)，它使用运动矢量来确定物体的移动速度。然而，在 **Bevy 0.11** 中我们只为"静态"网格添加了运动矢量支持，这意味着 TAA 对使用骨骼动画或形态目标的动画网格无效。

在 **Bevy 0.14** 中，我们实现了[逐对象运动模糊](https://bevy.org/news/bevy-0-14/#per-object-motion-blur)，它_也_使用运动矢量，因此也会有同样的局限性。

幸运的是，在 **Bevy 0.14** 中我们为蒙皮网格和带有形态目标的网格实现了运动矢量，弥补了这一差距，使 TAA、逐对象运动模糊以及未来的运动矢量功能能够与动画网格一起使用。

## 改进的矩阵命名 [#](https://bevy.org/news/bevy-0-14/#improved-matrix-naming)

作者：[@ricky26](https://github.com/ricky26)

PR：[#13489](https://github.com/bevyengine/bevy/pull/13489)

游戏引擎通常会提供一组矩阵来执行游戏世界中的空间变换。常用的空间包括：

- **归一化设备坐标（NDC）**：由图形 API 直接使用
- **裁剪空间（Clip Space）**：投影之后但透视除法之前的坐标
- **视图空间（View Space）**：摄像机视角下的坐标
- **世界空间（World Space）**：全局坐标（这是我们最常讨论的！）
- **模型空间（Model Space）**：（或局部空间）相对于实体的坐标

一个常见的例子是"模型视图投影矩阵"，即从模型空间到 NDC 空间的变换（在这种简写中有点特别的是，视图矩阵通常是从世界到视图空间的变换，而模型矩阵是从模型（或局部）空间到世界空间的变换）。通常，矩阵以其简写名称来指代，例如投影矩阵就是从视图坐标到 NDC 坐标的变换。

在少数地方，Bevy 的视图矩阵实际上是从视图到世界空间的变换（而非上述从世界到视图空间）。此外，即使使用一致，单个单词的简写也很含糊，容易引起混淆。我们认为需要一个更清晰的命名约定。

从现在起，Bevy 中的矩阵将以 `y_from_x` 的格式命名，例如 `world_from_local` 表示从局部坐标到世界坐标的变换。这样做的一个简洁好处是，逆矩阵自然命名为 `x_from_y`，在进行空间乘法运算时，很容易看出是否正确。

例如，不再这样写：

```rust
let model_view_projection = projection * view * model;
```

而是这样写：

```rust
let clip_from_local = clip_from_view * view_from_world * world_from_local;
```

## 类型化 glTF 标签 [#](https://bevy.org/news/bevy-0-14/#typed-gltf-labels)

作者：[@mockersf](https://github.com/mockersf), [@rparrett](https://github.com/rparrett), [@alice-i-cecile](https://github.com/alice-i-cecile)

PR：[#13586](https://github.com/bevyengine/bevy/pull/13586)

如果你一直在为场景使用 [`glTF`](https://www.khronos.org/gltf/) 文件，或看过相关示例，你可能已经注意到资源路径末尾的 _labels_（标签）：

```rust
let model_pine = asset_server.load("models/trees/pine.gltf#Scene0");
let model_hen = asset_server.load("models/animals/hen.gltf#Scene0");
let animation_hen = asset_server.load("models/animals/hen.gltf#Aniamtion1"); // 哦不！
```

注意末尾的 `#Scene0` 语法。glTF 格式允许在单个文件中包含多种内容，包括多个场景、动画、灯光等等。

这些标签是告诉 Bevy 我们要加载文件中哪个部分的方式。

然而这种方式容易出错，看起来这里就混进了一个错误！母鸡动画的标签写成了 `Aniamtion1`（拼写错误）而不是 `Animation1`。

不再需要这样了！上面的代码现在可以重写为：

```rust
let hen = "models/animals/hen.gltf"; // 现在也可以更方便地复用
let model_pine = asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/trees/pine.gltf"));
let model_hen = asset_server.load(GltfAssetLabel::Scene(0).from_asset(hen));
let animation_hen = asset_server.load(GltfAssetLabel::Animation(0).from_asset(hen)); // 没有拼写错误！
```

请查看 [`glTF 标签文档`](https://docs.rs/bevy/0.14/bevy/gltf/enum.GltfAssetLabel.html) 了解你可以查询哪些部分。

## winit v0.30 [#](https://bevy.org/news/bevy-0-14/#winit-v0-30)

作者：[@pietrosophya](https://github.com/pietrosophya), [@mockersf](https://github.com/mockersf)

PR：[#13366](https://github.com/bevyengine/bevy/pull/13366)

[Winit v0.30](https://docs.rs/winit/0.30.0/winit/changelog/v0_30/index.html) 将其 API 改为基于 trait 的架构，而非原先纯基于事件的架构。Bevy 0.14 现已实现这一新架构，使事件循环处理更加清晰易追踪。

现在可以定义自定义的 `winit` 用户事件，用于触发 App 更新，并在系统内部读取以触发特定行为。这对于从 `winit` 事件循环外部发送事件并在 Bevy 系统内部进行管理特别有用（参见 [`window/custom_user_event.rs`](https://github.com/bevyengine/bevy/blob/release-0.14.0/examples/window/custom_user_event.rs) 示例）。

`UpdateMode` 枚举现在只接受两个值：`Continuous` 和 `Reactive`。后者暴露了 3 个新属性，用于启用对设备、用户或窗口事件的响应。之前的 `UpdateMode::Reactive` 现在等同于 `UpdateMode::reactive()`，而 `UpdateMode::ReactiveLowPower` 映射为 `UpdateMode::reactive_low_power()`。

- `Idle`：循环尚未开始
- `Running`（之前称为 `Started`）：循环正在运行
- `WillSuspend`：循环即将被挂起
- `Suspended`：循环已挂起
- `WillResume`：循环即将恢复

注意：`Resumed` 状态已被移除，因为恢复后的应用就是 `Running` 状态。

## 场景、网格和材质的 glTF Extras [#](https://bevy.org/news/bevy-0-14/#scene-mesh-and-material-gltf-extras)

作者：[@kaosat-dev](https://github.com/kaosat-dev)

PR：[#13453](https://github.com/bevyengine/bevy/pull/13453)

glTF 3D 模型文件格式允许在 _extras_ 属性中传递用户自定义的额外元数据。除了在 primitive/node 级别的 glTF extras 之外，Bevy 现在还为以下类别提供了特定的 GltfExtras：

- **场景**：**SceneGltfExtras**，如果有则在场景级别注入
- **网格**：**MeshGltfExtras**，如果有则在网格级别注入
- **材质**：**MaterialGltfExtras**，如果有则在网格级别注入：即如果一个网格的材质包含 gltf extras，该组件将被注入到那里。

现在你可以轻松查询这些特定的 extras：

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
    // 使用 extras 中的数据
    for (id, name, scene_extras, extras, mesh_extras, material_extras) in
        gltf_extras_per_entity.iter()
    {

    }
}

```

这使得通过 glTF 文件从 Blender 等程序向 Bevy 传递信息更加符合规范，也更加实用！

## 场景中的资源实体映射 [#](https://bevy.org/news/bevy-0-14/#resource-entity-mapping-in-scenes)

作者：[@brandon-reinhart](https://github.com/brandon-reinhart)

PR：[#13650](https://github.com/bevyengine/bevy/pull/13650)

Bevy 的 `DynamicScene` 是一个资源和实体的集合，可序列化为预制体或存档数据等集合。当 DynamicScene 被反序列化并写入 World 时（例如加载存档游戏），场景内的动态实体标识符必须映射到它们新生成的对应实体。

以前，这种映射仅适用于存储在 Component 上的实体标识符。在 Bevy 0.14 中，Resource 可以通过反射 `MapEntitiesResource` 并实现 `MapEntities` trait 来获得 `EntityMapper` 的访问权限。

```rust
    // 此资源反射 MapEntitiesResource 并实现 MapEntities trait。
    #[derive(Resource, Reflect, Debug)]
    #[reflect(Resource, MapEntitiesResource)]
    struct TestResource {
        entity_a: Entity,
        entity_b: Entity,
    }

    // 一个简单且常见的用法是将旧实体直接映射到新实体。
    impl MapEntities for TestResource {
        fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
            self.entity_a = entity_mapper.map_entity(self.entity_a);
            self.entity_b = entity_mapper.map_entity(self.entity_b);
        }
    }
```

## CompassQuadrant 和 CompassOctant [#](https://bevy.org/news/bevy-0-14/#compassquadrant-and-compassoctant)

作者：[@BobG1983](https://github.com/BobG1983), [@alice-i-cecile](https://github.com/alice-i-cecile)

PR：[#13653](https://github.com/bevyengine/bevy/pull/13653)

在游戏开发中，有许多场景需要知道给定方向对应的指南针朝向。这在 2D 游戏中尤为常见，例如使用四个或八个方向的精灵，或者需要将模拟输入映射为离散移动方向。

为了简化这一需求，新增了 `CompassQuadrant`（四方向划分）和 `CompassOctant`（八方向划分）枚举，并提供了与 `From<Dir2>` 之间的实现，方便使用。

## 在加载资源时支持 `AsyncSeek` [#](https://bevy.org/news/bevy-0-14/#support-asyncseek-when-loading-assets)

作者：[@BeastLe9enD](https://github.com/BeastLe9enD)

PR：[#12547](https://github.com/bevyengine/bevy/pull/12547)

资源文件可能非常庞大，而你不一定需要单个文件中的所有数据。

Bevy 允许你添加[自己的资源加载器](https://github.com/bevyengine/bevy/blob/release-0.14.0/examples/asset/processing/asset_processing.rs)。从 Bevy 0.14 开始，你可以查找（seek）到任意偏移位置，从文件中间读取数据。

假设你有一个 `.celestial` 文件格式编码了整个宇宙，但你只想查看总是在某个偏移位置出现的小行星：

```rust
#[derive(Default)]
struct UniverseLoader;

#[derive(Asset, TypePath, Debug)]
struct JustALilAsteroid([u8; 128]); // 每个小行星占这么多数据

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
        // 宇宙很大，我们的小行星在天体文件格式中要到这个偏移位置才开始出现
        let offset_of_lil_asteroids = 5_000_000_000_000;

        // 使用新的异步 seek trait 跳过宇宙的绝大部分内容！
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

这是因为传入资源加载器 `load` 函数的 Bevy [`reader`](https://docs.rs/bevy/0.14/bevy/asset/io/type.Reader.html) 类型现在实现了 [`AsyncSeek`](https://docs.rs/futures-io/latest/futures_io/trait.AsyncSeek.html)。

实际使用场景例如：

- 你将多个资源打包在一个归档文件中，希望跳转到其中某个资源并读取
- 你处理的是地图数据等大数据集，知道某些感兴趣位置的提取位置

## LoadState::Failed 现在包含错误信息 [#](https://bevy.org/news/bevy-0-14/#loadstate-failed-now-has-error-info)

作者：[@bugsweeper](https://github.com/bugsweeper)

PR：[#12709](https://github.com/bevyengine/bevy/pull/12709)

Rust 以其错误处理能力而自豪，Bevy 也一直在迎头赶上。以前，当使用 [`AssetServer::get_load_state`](https://docs.rs/bevy/0.14/bevy/asset/struct.AssetServer.html#method.get_load_state) 检查资源是否加载完成时，如果出现问题，你只能得到一个不包含任何数据的 [`LoadState::Failed`](https://docs.rs/bevy/0.14/bevy/asset/enum.LoadState.html)。这对调试来说用处不大！

现在，返回结果中包含完整的 [`AssetLoadError`](https://docs.rs/bevy/0.14/bevy/asset/enum.AssetLoadError.html)，共有 14 个不同的变体，精确告诉你出了什么问题。这对于故障排查非常有用，也为更复杂应用中实现完善的错误处理打开了大门。

## `AppExit` 错误 [#](https://bevy.org/news/bevy-0-14/#appexit-errors)

作者：[@Brezak](https://github.com/Brezak), [@alice-i-cecile](https://github.com/alice-i-cecile)

PR：[#13022](https://github.com/bevyengine/bevy/pull/13022)

运行应用时，触发退出的原因可能有很多。可能是用户按下了退出按钮，也可能是渲染线程遇到错误而崩溃了。你可能希望区分这两种情况，并从应用程序返回一个适当的[退出码](https://doc.rust-lang.org/std/process/struct.ExitCode.html#impl-From%3Cu8%3E-for-ExitCode)。

在 **Bevy 0.14** 中，你可以做到这一点。`AppExit` 事件现在是一个包含两个变体的枚举：`Success` 和 `Error`。`Error` 变体还持有一个非零的退出码，你可以按自己的意愿使用它。由于 `AppExit` 事件现在包含了有用的信息，应用运行器和 `App::run` 现在会返回导致应用退出的事件。

对于插件开发者来说，`App` 新增了一个方法 `App::should_exit`，它会检查最近两次更新中是否有任何 `AppExit` 事件被发送。为了确保 `AppExit::Success` 事件不会淹没有用的错误信息，即使 `AppExit::Error` 事件在 `AppExit::Success` 之后发送，该方法也会优先返回 `AppExit::Error` 事件。

最后，`AppExit` 还实现了 [`Termination`](https://doc.rust-lang.org/stable/std/process/trait.Termination.html) trait，因此可以直接从 main 函数返回。

```rust
use bevy::prelude::*;

fn exit_with_a_error_code(mut events: EventWriter<AppExit>) {
    events.send(AppExit::from_code(42));
}

fn main() -> AppExit {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_systems(Update, exit_with_a_error_code)
        .run() // 这里没有分号，`run()` 返回 `AppExit`。
}
```

![App 返回 42 退出码](https://bevy.org/news/bevy-0-14/exit_with_a_42.jpg)

## 在 WASM 目标上让 dynamic_linking 成为空操作 [#](https://bevy.org/news/bevy-0-14/#make-dynamic-linking-a-no-op-on-wasm-targets)

作者：[@james7132](https://github.com/james7132)

PR：[#12672](https://github.com/bevyengine/bevy/pull/12672)

WASM 不支持运行时动态链接库。以前，如果你启用了 `dynamic_linking` 功能，Bevy 在 WASM 目标上会编译失败。

```bash
$ cargo build --target wasm32-unknown-unknown --features bevy/dynamic_linking
error: cannot produce dylib for `bevy_dylib v0.13.2` as the target `wasm32-unknown-unknown` does not support these crate types
```

现在，Bevy 会在所有 WASM 目标上回退为静态链接。如果你为开发启用了 `dynamic_linking`，在 WASM 上不再需要手动禁用它。

## 弃用动态插件 [#](https://bevy.org/news/bevy-0-14/#deprecate-dynamic-plugins)

作者：[@BD103](https://github.com/BD103)

PR：[#13080](https://github.com/bevyengine/bevy/pull/13080)

`bevy_dynamic_plugin` 是 Bevy 最初 0.1 版本发布时添加的工具，旨在作为动态加载/链接 Rust 代码的工具，用于 Mod 制作等场景。遗憾的是，这个功能并没有获得太多社区采用，因此多年来用于改进和文档化的贡献也寥寥无几。

再加上其 API 设计困难且本质上不安全，给用户带来了[令人担忧的故障](https://github.com/bevyengine/bevy/issues/13073)，我们决定弃用 `bevy_dynamic_plugin`，并将在 Bevy 0.15 中完全移除它。如果你是这个功能的满意用户，只需将这个相当小的 crate 复制到自己的项目中，像以前一样继续使用即可。

我们仍然认为 Mod 制作和热重载代码以加快开发速度都是 Bevy _应该_ 在未来支持的有价值用例。我们希望通过将其从第一方 crate 中移除，能够激发第三方实验，避免用户在调查一个复杂的潜在解决方案后才得出结论说它还不满足需求——白白浪费时间。

## Bevy 工作组 [#](https://bevy.org/news/bevy-0-14/#bevy-working-groups)

作者：[@alice-i-cecile](https://github.com/alice-i-cecile)

PR：[#13162](https://github.com/bevyengine/bevy/pull/13162)

Bevy 拥有大量才华横溢的贡献者，因此跟踪项目进展并做出明智决策确实是一个挑战。我们正在尝试[工作组](https://github.com/bevyengine/bevy/blob/main/CONTRIBUTING.md#join-a-working-group)模式：由临时组成的团队通过创建设计文档、获得专家认可，然后实施来解决更棘手的问题。如果你想帮助 Bevy 做出复杂且具有高影响力的改变：加入或组建一个工作组！

## 下一步计划？ [#](https://bevy.org/news/bevy-0-14/#what-s-next)

以上功能可能很不错，但 Bevy 还有哪些正在进行的工作呢？深入时间的迷雾（当你的团队几乎全是志愿者时，预测_格外_困难！），我们可以看到一些激动人心的工作正在成型：

- **更好的场景系统：** 场景是 Bevy 的核心构建块之一，旨在成为创建关卡和可复用游戏对象的强大工具，无论是按钮控件还是怪物。我们正在开发一个新的场景系统，配合新的语法，使在资源中和代码中定义场景变得更加高效和愉悦。你可以查看（如今已略有些年头）的[项目启动讨论](https://github.com/bevyengine/bevy/discussions/9538)了解更多信息。我们也非常接近发布一份设计文档，概述我们的计划和当前实现状态。
- **ECS 关系：** 关系（一种连接实体的头等特性）是社区强烈需求的功能，但实现起来非常复杂，推动着我们对 ECS 内部的功能和重构。[工作组](https://discord.com/channels/691052431525675048/1237010014355456115)一直在耐心地通过这份 [RFC](https://github.com/bevyengine/rfcs/pull/79) 阐述我们需要做什么以及为什么这么做。
- **更好的音频：** Bevy 内置的音频解决方案从未真正奏效。[Better Audio 工作组](https://discord.com/channels/691052431525675048/1236113088793677888)正在规划前进的道路。
- **贡献手册：** 我们的贡献指南文档分散在各个仓库的各个角落。通过将其收集整理在一起，[Contributing Book 工作组](https://discord.com/channels/691052431525675048/1236112637662724127)希望使其更易于发现和维护。
- **曲线抽象：** 曲线在游戏开发中无处不在，由数学魔法师组成的 [Curve Crew](https://discord.com/channels/691052431525675048/1236110755212820581) 正在[设计一个 trait](https://github.com/bevyengine/rfcs/pull/80) 来统一和增强它们。
- **更好的文本：** 我们现有的文本解决方案无法满足现代 UI 的需求。"Lorem Ipsum"工作组正在[研究](https://discord.com/channels/691052431525675048/1248074018612051978)用更好的方案替换它。
- **统一的开发工具视图：** 在 0.14 中，我们添加了一个占位的 `bevy_dev_tools` crate：一个存放加速游戏开发的工具和叠加层的地方，例如性能监视器、飞行摄像机或生成游戏对象的游戏内命令。我们正在添加更多工具，并创建[开发工具抽象层](https://github.com/bevyengine/rfcs/pull/77)。这将为我们提供一种统一的方式来启用/禁用、自定义和将这些杂七杂八的工具分组到工具箱中，创建出像 Quake 控制台或 VSCode 命令面板那样汇集生态系统中各种工具的东西。
- **Bevy 远程协议：** 与正在运行的 Bevy 游戏进行通信，是构建编辑器、调试器和其他工具的强大能力。[我们正在开发](https://github.com/bevyengine/bevy/pull/13563)一个基于反射的协议，来创建能够支撑整个生态系统的解决方案。
- **模块化、可维护的渲染图：** Bevy 现有的渲染架构在提供可复用的渲染器功能（如 `RenderPhases`、批处理和绘制命令）方面已经做得相当不错。 然而，渲染图接口本身仍然是一个痛点。由于它分布在许多文件中，控制流难以理解，而且大量使用 ECS 资源来传递渲染数据也妨碍了模块化。虽然确切的设计尚未最终确定（非常欢迎反馈！），但我们一直在积极推进[重新设计渲染图](https://github.com/bevyengine/bevy/pull/13397)，以便为渲染器的更大规模重构奠定基础，实现模块化和易用性。