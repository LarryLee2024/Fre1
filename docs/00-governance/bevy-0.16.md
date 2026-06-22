# Bevy 0.16

## Posted on April 24, 2025 by Bevy Contributors

![A planet from EmbersArc's in-development spaceflight simulation game, rendered with custom shaders in Bevy](https://bevy.org/news/bevy-0-16/planet.jpg)

[A planet from EmbersArc's in-development spaceflight simulation game, rendered with custom shaders in Bevy](https://bsky.app/profile/embersarc.bsky.social)

感谢 **261** 位贡献者、**1244** 个 pull request、社区审阅者以及我们[**慷慨的捐赠者**](https://bevy.org/donate)，我们很高兴地宣布 **Bevy 0.16** 已在 [crates.io](https://crates.io/crates/bevy) 上发布！

如果你还不了解，Bevy 是一个基于 Rust 构建的、令人耳目一新的简洁数据驱动游戏引擎。你可以查看我们的[快速入门指南](https://bevy.org/learn/quick-start)来立即试用。它是免费且永远开源的！你可以在 GitHub 上获取完整的[源代码](https://github.com/bevyengine/bevy)。查看 [Bevy Assets](https://bevy.org/assets) 获取社区开发的插件、游戏和学习资源合集。

要将现有的 Bevy App 或 Plugin 更新到 **Bevy 0.16**，请查看我们的 [0.15 到 0.16 迁移指南](https://bevy.org/learn/migration-guides/0-15-to-0-16/)。

自上次发布以来的几个月里，我们添加了_大量_新功能、Bug 修复和易用性改进，以下是一些亮点：

- **GPU 驱动渲染：** Bevy 现在在 GPU 上完成了更多渲染工作（在可能的情况下），使 Bevy 在大型复杂场景上显著加快。
- **程序化大气散射：** 以低性能成本模拟任意时间的真实物理地球类天空。
- **贴花：** 将纹理动态分层到已渲染的网格上。
- **遮挡剔除：** 通过不渲染被其他物体遮挡的物体来提高性能。
- **ECS 关系：** 最热门的 ECS 功能之一终于来了：允许你轻松且健壮地建模和处理实体-实体连接。有一些注意事项，但我们很高兴今天能向用户提供一个简单而健壮的解决方案。
- **改进的 Spawn API：** 生成实体层级结构现在变得容易得多！
- **统一错误处理：** Bevy 现在一等支持错误处理，使其变得简单、灵活且符合人体工程学，同时使调试更加容易！
- **`no_std` 支持：** `bevy` 本身和大量子 crate 不再依赖 Rust 的标准库，让你可以在从现代游戏设备到 GameBoy Advance 的所有平台上使用同一个引擎。
- **更快的 Transform 传播：** 我们大幅提升了同时处理更多对象时 transform 传播的性能，特别是对于静态对象。

## GPU 驱动渲染 [#](https://bevy.org/news/bevy-0-16/#gpu-driven-rendering)

Authors:[@pcwalton](https://github.com/pcwalton), [@tychedelia](https://github.com/tychedelia), [@robswain](https://github.com/robswain)

PRs:[#16427](https://github.com/bevyengine/bevy/pull/16427)

多年来，实时渲染的趋势越来越倾向于将工作从 CPU 转移到 GPU。这一领域最新的发展之一是_GPU 驱动渲染_，即 GPU 获取场景的表示并自行计算要绘制的内容。

**Bevy 0.16** 为大多数"标准" 3D 网格渲染添加了 GPU 驱动渲染支持，包括蒙皮网格。这大幅减少了渲染器在较大场景上所需的 CPU 时间。它在支持的平台上自动启用；除非你的应用程序挂接到渲染管线，否则升级到 **Bevy 0.16** 将自动为你的网格启用 GPU 驱动渲染。这加入了 **Bevy 0.14** 和 **0.15** 为[虚拟几何](https://bevy.org/news/bevy-0-14/#virtual-geometry-experimental)添加的 GPU 驱动渲染支持。

在 Activision 的 [Caldera 场景](https://github.com/Activision/caldera)（来自 Call of Duty Warzone）的"重型"酒店部分，具有 GPU 驱动渲染的 **Bevy 0.16** 的性能大约比 **Bevy 0.15** 好 3 倍！（这包括这些版本之间的_所有_优化）

![Caldera scene rendered in Bevy](https://bevy.org/news/bevy-0-16/caldera.jpg)

在移动版 Nvidia 4090 上使用 Vulkan / Linux，**Bevy 0.16** 以 10.16ms（约 101 FPS）运行该场景，而 **Bevy 0.15** 为 33.55ms（约 30 FPS）。巨大的胜利！

### 概述：CPU 驱动渲染

要解释 GPU 驱动渲染如何工作，最简单的方法是先描述 CPU 驱动渲染的工作方式：

1. CPU 通过视锥剔除和可能的遮挡剔除来确定可见的对象。
2. 对于每个此类对象：
    - CPU 将对象的变换发送到 GPU，可能还包括其他数据如关节权重。
    - CPU 告诉 GPU 网格数据在哪里。
    - CPU 将材质数据写入 GPU。
    - CPU 告诉 GPU 渲染对象所需的纹理和其他缓冲区的位置（灯光数据等）。
    - CPU 发出绘制调用。
    - GPU 渲染对象。

### 概述：GPU 驱动渲染

相比之下，Bevy 中的 GPU 驱动渲染是这样工作的：

1. CPU 向 GPU 提供一个包含所有对象变换信息的单一缓冲区，以便着色器可以一次处理多个对象。
2. 如果自上一帧以来有新对象被生成，CPU 会填写表来指定新对象的网格数据位置。
3. 如果自上一帧以来有材质被修改，CPU 会将这些材质的信息上传到 GPU。
4. CPU 创建本帧要渲染的对象列表。每个对象仅通过一个整数 ID 引用，因此这些列表很小。列表的数量取决于场景的大小和复杂度，但即使是大型场景也很少超过 15 个这样的列表。
5. 对于每个这样的列表：
    - CPU 发出一个_单一的_绘制调用。
    - GPU 处理列表中的所有对象，确定哪些是真正可见的。
    - GPU 渲染每个可见对象。

对于可能有数万个对象的大型场景，GPU 驱动渲染通常会将 CPU 渲染开销减少 3 倍或更多。它对于遮挡剔除也是必要的，因为 GPU 变换步骤（上面的 5(b)）。

### GPU 驱动渲染技术

在内部，GPU 驱动渲染与其说是单一技术，不如说是多种技术的组合。这些包括：

- _多绘制间接_（MDI），一种 GPU API，允许在一次绘制调用中绘制多个网格，GPU 通过在 GPU 内存中填写表来提供细节。为了有效使用 MDI，Bevy 使用了一个新的子系统，即_网格分配器_，它管理在 GPU 内存中将网格打包在一起的细节。
    - _多绘制间接计数_（MDIC），多绘制间接的扩展，允许 GPU 以最小开销确定要绘制的网格_数量_。
- _无绑定资源_，允许 Bevy 将多个对象的纹理（和其他资源）作为一组提供，而不必在 CPU 上逐一绑定纹理。这些资源由一个称为_材质分配器_的新子系统管理。
- _GPU 变换和剔除_，允许 Bevy 在 GPU 上而非 CPU 上计算每个对象的位置和可见性。
- _保留渲染世界_，允许 CPU 避免处理和上传自上一帧以来未更改的数据。
- _缓存管线特化_，利用 Bevy 的组件级变更检测来更快地确定网格的渲染状态是否与前一帧相同。

### GPU 驱动平台兼容性

目前，并非所有平台都完全支持此功能。下表总结了 GPU 驱动渲染各个部分的平台支持情况：

|OS|图形 API|GPU 变换|多绘制与 GPU 剔除|无绑定资源|
|---|---|---|---|---|
|Windows|Vulkan|✅|✅|✅|
|Windows|Direct3D 12|✅|❌|❌|
|Windows|OpenGL|✅|❌|❌|
|Linux|Vulkan|✅|✅|✅|
|Linux|OpenGL|✅|❌|❌|
|macOS|Metal|✅|❌|➖¹|
|iOS|Metal|✅|❌|➖¹|
|Android|Vulkan|➖²|➖²|➖²|
|Web|WebGPU|✅|❌|❌|
|Web|WebGL 2|❌|❌|❌|

¹ Bevy 确实在 Metal 上支持无绑定资源，但目前限制明显更低，可能导致更多的绘制调用。

² 一些已知在 Bevy 工作负载中表现出 bug 的 Android 驱动程序被加入黑名单，会导致 Bevy 回退到 CPU 驱动渲染。

在大多数情况下，你不需要做任何特殊操作来让你的应用程序支持 GPU 驱动渲染。有两个主要例外：

1. 具有自定义 WGSL 着色器的材质默认将继续使用 CPU 驱动渲染。为了让你的材质使用 GPU 驱动渲染，你需要在 `AsBindGroup` 上使用新的 `#[bindless]` feature。详情请参见 `AsBindGroup` 文档和 `shader_material_bindless` 示例。如果你使用 `ExtendedMaterial`，请查看新的 `extended_material_bindless` 示例。
2. 底层挂接到渲染器的应用程序和插件需要更新以支持 GPU 驱动渲染。新更新的 `custom_phase_item` 和 `specialized_mesh_pipeline` 示例可能作为指南对此有用。

### GPU 驱动渲染的下一步是什么？

Bevy 当前的 GPU 驱动渲染并不是终点。未来还有大量潜在的工作要做：

- **Bevy 0.16** 仅支持 3D 管线的 GPU 驱动渲染，但这些技术同样适用于 2D 管线。未来的 Bevy 版本应支持 2D 网格渲染、精灵、UI 等的 GPU 驱动渲染。
- Bevy 目前使用 CPU 驱动渲染绘制具有变形目标的对象。这是我们计划在未来解决的问题。请注意，存在具有变形目标的对象并不阻止没有变形目标的对象使用 GPU 驱动渲染绘制。
- 未来，部分 GPU 驱动渲染基础设施可以移植到不支持完整功能集的平台，在这些平台上提供一些性能改进。例如，即使在 WebGL 2 上，渲染器也可以利用材质分配器更高效地打包数据。
- 我们正在关注新的 API 功能，如 [Vulkan 设备生成命令](https://www.supergoodcode.com/device-generated-commands/) 和 [Direct3D 12 工作图](https://devblogs.microsoft.com/directx/d3d12-work-graphs/)。这些将允许未来的 Bevy 版本将更多工作卸载到 GPU，例如透明对象的排序。虽然想清楚如何在单一渲染器中统一这些不同的 API 将是一个挑战，但这个领域的未来可能性令人兴奋。

如果你对其中任何任务感兴趣，欢迎在 [我们的 Discord](https://discord.gg/bevy) 或通过 [GitHub issues](https://github.com/bevyengine/bevy/issues) 提问。

## 程序化大气散射 [#](https://bevy.org/news/bevy-0-16/#procedural-atmospheric-scattering)

Authors:[@ecoskey](https://github.com/ecoskey), [@mate-h](https://github.com/mate-h)

PRs:[#16314](https://github.com/bevyengine/bevy/pull/16314), [#17555](https://github.com/bevyengine/bevy/pull/17555), [#17672](https://github.com/bevyengine/bevy/pull/17672), [#18582](https://github.com/bevyengine/bevy/pull/18582)

**Bevy 0.16** 引入了程序化大气散射，一个可定制的系统，用于在实时中模拟日落、日出和动态昼夜循环：

感谢 `@aevyrie` 带来的精彩 [atmosphere showcase](https://github.com/aevyrie/bevy/tree/atmosphere_showcase)！它使用了一个花哨的自定义曝光曲线来强调接近黄昏时的色彩。

启用大气渲染很简单，只需将新的 [`Atmosphere`](https://docs.rs/bevy/0.16/bevy/pbr/struct.Atmosphere.html) 组件添加到你的相机！

```rs
commands.spawn((
    Camera3d::default(),
    Atmosphere::EARTH,
));

// the atmosphere will consider all directional lights in the scene as "suns"
commands.spawn(DirectionalLight::default());
```

启用时，主要的 Bevy 天空盒会叠加一个基于场景中方向光实时更新的天空盒，默认距离雾会被考虑方向光和其他大气参数的雾所替代。在晴朗的日子里远处的物体将淡入蓝色，在日落时会染上橙色和粉红色！此外，由于大气是在天空盒_之上_合成的，创建夜间星空很容易...只需生成天空盒，随着黎明时天空变亮，它会自然消退。

与大多数 PBR 技术一样，它是_正确的_，但可能需要一些调整才能达到最佳效果。所有大气参数也可以自定义：例如，高沙漠天空由于缺乏湿度可能表现出更少的 Mie 散射。包含的示例 `examples/3d/atmosphere.rs` 包含一些关于灯光和相机设置的建议。

目前有一些限制需要注意：大气目前不会影响 [`EnvironmentMapLight`](https://docs.rs/bevy/0.16/bevy/pbr/environment_map/struct.EnvironmentMapLight.html) 或方向光对表面的直接光照，因此反射可能不完全准确。我们还在努力将大气散射与体积雾集成（请参见[下一步是什么？](https://bevy.org/news/bevy-0-16/#what-s-next)部分了解我们的雄心计划！）。

由于 Sébastien Hillaire 的 [EGSR 2020 论文](https://sebh.github.io/publications/egsr2020.pdf) 中描述的许多优化，我们的实现非常快速，即使在移动设备和 WebGPU 上也能很好地工作。秘诀在于，由于大气基本对称，我们可以提前预计算光线行进内部循环的大部分。

## 贴花 [#](https://bevy.org/news/bevy-0-16/#decals)

Authors:[@naasblod](https://github.com/naasblod), [@JMS55](https://github.com/JMS55), [@pcwalton](https://github.com/pcwalton)

PRs:[#16600](https://github.com/bevyengine/bevy/pull/16600), [#17315](https://github.com/bevyengine/bevy/pull/17315)

**贴花**是可以动态分层到现有网格上的纹理，符合其几何形状。这比简单地更改网格纹理有两个好处：

1. 你可以动态添加它们以响应玩家操作。最著名的是 FPS 游戏中的弹孔使用贴花来实现这一点。
2. 你不需要为每种组合创建全新的纹理，这使它们在创建具有涂鸦或建筑外墙裂缝等细节的关卡时更加高效和灵活。

像渲染中的许多事情一样，实现此功能有大量方法，每种都有其权衡。在 **Bevy 0.16** 中，我们选择了两种互补的方法：**前向贴花**和**聚类贴花**。

### 前向贴花

![decals](https://bevy.org/news/bevy-0-16/decals.jpg)

我们的前向贴花（或者更精确地说，接触投影贴花）实现灵感来自 [Alexander Sannikovs 关于 Path of Exile 2 渲染技术的演讲](https://www.youtube.com/watch?v=TrHHTQqmAaM)，并从 [`bevy_contact_projective_decals`](https://github.com/naasblod/bevy_contact_projective_decals) 生态系统 crate 上游化。由于此技术的性质，从非常陡峭的角度查看贴花会导致变形。这可以通过创建比效果更大的纹理来缓解，给贴花更多的拉伸空间。要创建前向贴花，生成一个 [`ForwardDecal`](https://docs.rs/bevy/0.16/bevy/pbr/decal/struct.ForwardDecal.html) 实体，它使用 [`ForwardDecalMaterialExt`](https://docs.rs/bevy/0.16/bevy/pbr/decal/struct.ForwardDecalMaterialExt.html) 材质扩展的 [`ForwardDecalMaterial`](https://docs.rs/bevy/0.16/bevy/pbr/decal/type.ForwardDecalMaterial.html)。

### 聚类贴花

![clustered decals](https://bevy.org/news/bevy-0-16/clustered-decals.jpg)

聚类贴花（或贴花投影器）通过从 1x1x1 立方体向 +Z 方向找到的表面投影图像来工作。它们是可聚类的对象，就像点光源和光探针一样，这意味着贴花仅对投影器范围内的对象进行评估。要创建聚类贴花，生成一个 [`ClusteredDecal`](https://docs.rs/bevy/0.16/bevy/pbr/decal/clustered/struct.ClusteredDecal.html) 实体。

最终，前向贴花提供更广泛的硬件和驱动程序支持，而聚类贴花质量更高且不需要创建边界几何体，提高了性能。目前聚类贴花需要无绑定纹理，因此不支持 WebGL2、WebGPU、iOS 和 Mac 目标。前向贴花_可以_在这些目标上使用。

## 实验性遮挡剔除 [#](https://bevy.org/news/bevy-0-16/#experimental-occlusion-culling)

Authors:[@pcwalton](https://github.com/pcwalton), [@JMS55](https://github.com/JMS55)

PRs:[#12755](https://github.com/bevyengine/bevy/pull/12755), [#17413](https://github.com/bevyengine/bevy/pull/17413), [#17934](https://github.com/bevyengine/bevy/pull/17934), [#17951](https://github.com/bevyengine/bevy/pull/17951)

**遮挡剔除**的理念是，我们不需要绘制从相机角度被其他不透明物体完全挡住的东西。例如：我们不需要绘制隐藏在墙后面的人，即使他们在视锥剔除使用的范围内。

Bevy 已经有一个可选的 [Depth Prepass](https://bevy.org/news/bevy-0-10/#depth-and-normal-prepass)，它渲染场景的简单版本并捕获 2D 深度缓冲区。然后可以用于在更昂贵的主通道中跳过隐藏的对象。但这不会跳过顶点着色的开销，片段着色器中的深度检查也会增加开销。

在 **Bevy 0.16** 中，我们添加了现代[两阶段遮挡剔除](https://medium.com/@mil_kru/two-pass-occlusion-culling-4100edcad501)（与传统的"潜在可见集"设计相对）。这种方法已经被我们的[虚拟几何](https://bevy.org/news/bevy-0-14/#virtual-geometry-experimental)渲染系统使用，并且与我们在本周期内建立的 GPU 驱动渲染架构配合得很好！更多实现细节请[查看此 PR](https://github.com/bevyengine/bevy/pull/17413)。

目前，此功能标记为实验性，因为已知的精度问题可能会将网格标记为被遮挡即使它们并非如此。实际上，我们不认为这是一个严重的问题，所以请告诉我们它的使用情况！要尝试新的网格遮挡剔除，将 [`DepthPrepass`](https://docs.rs/bevy/0.16/bevy/core_pipeline/prepass/struct.DepthPrepass.html) 和 [`OcclusionCulling`](https://docs.rs/bevy/0.16/bevy/render/experimental/occlusion_culling/struct.OcclusionCulling.html) 组件添加到你的相机。

一个重要提示：遮挡剔除在所有场景上并不都会更快。小型场景或使用更简单非 PBR 渲染的场景在开启遮挡剔除时尤其可能变慢。启用遮挡剔除会带来开销...它跳过的工作必须比运行检查的成本更昂贵才值得！

一如既往：你需要衡量你的性能来改进它。

如果你是一名想要帮助我们解决这些精度问题并稳定此功能的渲染工程师，我们正在参考 Hans-Kristian Arntzen 在 [Granite](https://github.com/Themaister/Granite) 中的设计。请在 [issue #14062](https://github.com/bevyengine/bevy/issues/14062) 参与讨论（并阅读我们的[贡献指南](https://bevy.org/learn/contribute/introduction/)），我们可以帮助你入门。

## 变形光晕 [#](https://bevy.org/news/bevy-0-16/#anamorphic-bloom)

Authors:[@aevyrie](https://github.com/aevyrie)

PRs:[#17096](https://github.com/bevyengine/bevy/pull/17096)

光晕是从明亮光源区域溢出的柔和辉光：在现实世界中，这是由相机（或眼睛）上的传感器完全饱和造成的。在游戏中，它是一个强大的艺术工具，可用于创建从赛博朋克霓虹灯到优雅发光的窗户再到令人满足的几何风格街机游戏的一切。

Bevy 自 0.9 版本以来就有光晕，但我们正在给艺术家另一个简单的调整杠杆：通过设置相机上 [`Bloom`](https://docs.rs/bevy/0.16/bevy/core_pipeline/bloom/struct.Bloom.html) 组件的二维 `scale` 参数来拉伸、压缩和以其他方式变形效果的能力。

![A realistic high-polygon model of a fancy black Porsche 911 demonstrating anamorphic bloom in its stretched-out tail light glow. Rendered in Bevy 0.16!](https://bevy.org/news/bevy-0-16/anamorphic-car-bloom.jpg)

当严重偏斜（通常是水平方向）时，此效果称为**变形光晕**。此效果与电影般的未来感氛围相关，并模拟某些胶片相机在将更宽的图像压缩到更窄胶片上时的不寻常几何形状。但不管它为什么发生，它看起来就是很棒！

## ECS 关系 [#](https://bevy.org/news/bevy-0-16/#ecs-relationships)

Authors:[@cart](https://github.com/cart)

PRs:[#17398](https://github.com/bevyengine/bevy/pull/17398)

在构建 Bevy 应用程序时，将实体"链接"在一起通常很有用。在 Bevy 中最常见的情况是将父级和子级实体连接在一起。在以前的 Bevy 版本中，子级会有一个 `Parent` 组件，存储对父级实体的引用，而父级实体会有一个 `Children` 组件，存储所有子级实体的列表。为了确保这些连接保持有效，开发者不允许直接修改这些组件。相反，所有更改都必须通过专门的命令完成。

这可以工作，但有一些相当明显的缺点：

1. 维护层级结构与核心 ECS 数据模型是"分离"的。这使得改进我们的生成 API 变得很困难，也使与层级结构的交互变得不太自然。
2. 系统是专门的且不可重用。想要定义自己关系类型的开发者必须重新发明轮子。
3. 为了确保数据完整性，需要进行昂贵的扫描以避免重复。

在 **Bevy 0.16** 中，我们添加了**关系**的初始支持：一个通用的、高效的、基于组件的系统，用于双向链接实体。这就是定义新的 [`Relationship`](https://docs.rs/bevy/0.16/bevy/ecs/relationship/trait.Relationship.html) 的样子：

```rust
/// This is a "relationship" component.
/// Add it to an entity that "likes" another entity.
#[derive(Component)]
#[relationship(relationship_target = LikedBy)]
struct Likes(pub Entity);

/// This is the "relationship target" component.
/// It will be automatically inserted and updated to contain
/// all entities that currently "like" this entity.
#[derive(Component, Deref)]
#[relationship_target(relationship = Likes)]
struct LikedBy(Vec<Entity>);

// Later in your app
let e1 = world.spawn_empty().id();
let e2 = world.spawn(Likes(e1)).id();
let e3 = world.spawn(Likes(e1)).id();

// e1 is liked by e2 and e3 
let liked_by = world.entity(e1).get::<LikedBy>().unwrap();
assert_eq!(&**liked_by, &[e2, e3]);
```

[`Relationship`](https://docs.rs/bevy/0.16/bevy/ecs/relationship/trait.Relationship.html) 组件是"真实来源"，[`RelationshipTarget`](https://docs.rs/bevy/0.16/bevy/prelude/trait.RelationshipTarget.html) 组件被更新以反映该真实来源。这意味着添加/移除关系应始终通过 [`Relationship`](https://docs.rs/bevy/0.16/bevy/ecs/relationship/trait.Relationship.html) 组件完成。

出于性能原因，我们使用这种"真实来源"模型而不是允许两个组件都"驱动"。允许双方写入需要在插入期间进行昂贵的扫描以确保它们同步且没有重复。"关系作为真实来源"的方法允许我们使添加关系成为常数时间（这是对旧的 Bevy 父级/子级方法的改进！）。

关系构建在 Bevy 的 [Component Hooks](https://bevy.org/news/bevy-0-14/#ecs-hooks-and-observers) 之上，通过直接插入组件的 add/remove/update 生命周期来立即且高效地维护 [`Relationship`](https://docs.rs/bevy/0.16/bevy/ecs/relationship/trait.Relationship.html) 和 [`RelationshipTarget`](https://docs.rs/bevy/0.16/bevy/prelude/trait.RelationshipTarget.html) 之间的连接。结合新的[不可变组件](https://bevy.org/news/bevy-0-16/#immutable-components)功能（关系组件是不可变的），这确保了无论开发者做什么都能维护数据完整性！

Bevy 现有的层级系统已被新的 [`ChildOf`](https://docs.rs/bevy/0.16/bevy/prelude/struct.ChildOf.html) [`Relationship`](https://docs.rs/bevy/0.16/bevy/ecs/relationship/trait.Relationship.html) 和 [`Children`](https://docs.rs/bevy/0.16/bevy/ecs/hierarchy/struct.Children.html) [`RelationshipTarget`](https://docs.rs/bevy/0.16/bevy/prelude/trait.RelationshipTarget.html) 完全替换。添加子级现在就像这样简单：

```rust
commands.spawn(ChildOf(some_parent));
```

同样重新设置实体的父级就像这样简单：

```rust
commands.entity(some_entity).insert(ChildOf(new_parent));
```

我们也借此机会更一般地改进了我们的生成 API。详情请阅读下一节！

请注意，这只是关系的第一步。我们计划扩展其能力：

1. **多对多关系**：当前系统是一对多的（例如 [`ChildOf`](https://docs.rs/bevy/0.16/bevy/prelude/struct.ChildOf.html) [`Relationship`](https://docs.rs/bevy/0.16/bevy/ecs/relationship/trait.Relationship.html) 指向"一个"目标实体，[`Children`](https://docs.rs/bevy/0.16/bevy/ecs/hierarchy/struct.Children.html) [`RelationshipTarget`](https://docs.rs/bevy/0.16/bevy/prelude/trait.RelationshipTarget.html) 可以被"多个"子级实体指向）。一些关系可能受益于支持多个关系目标。
2. **碎片化关系**：在当前系统中，关系组件基于其_类型_碎片化 ECS 原型，就像普通组件一样（例如 `(Player, ChildOf(e1))` 和 `(Player, ChildOf(e2))` 存在于同一原型中）。碎片化关系将是一个可选系统，基于其_值_碎片化原型，这将导致具有相同关系目标的实体存储在一起。这充当索引，使按值查询更快，并使某些访问模式更缓存友好。

## 改进的 Spawn API [#](https://bevy.org/news/bevy-0-16/#improved-spawn-api)

Authors:[@cart](https://github.com/cart)

PRs:[#17521](https://github.com/bevyengine/bevy/pull/17521)

在 Bevy 中生成层级结构一直有些繁琐：

```rust
commands
    .spawn(Player)
    .with_children(|p| {
        p.spawn(RightHand).with_children(|p| {
            p.spawn(Glove);
            p.spawn(Sword);
        });
        p.spawn(LeftHand).with_children(|p| {
            p.spawn(Glove);
            p.spawn(Shield);
        });
    });
```

我们有宏伟的计划通过[下一代 Scene / UI 系统](https://github.com/bevyengine/bevy/discussions/14437)（BSN）来改善 Bevy 的生成体验。这条路上的重要一步是使直接通过数据表达层级结构成为可能，而不是使用构建器方法。关系的增加进一步增加了构建这样一个系统的价值，因为_所有_关系都可以从中受益。

在 **Bevy 0.16** 中，我们大幅改善了生成层级结构的人体工程学：

```rust
commands.spawn((
    Player,
    children![
        (RightHand, children![Glove, Sword]),
        (LeftHand, children![Glove, Shield]),
    ],
));
```

这建立在现有的 Bundle API 之上，添加了对"bundle effects"的支持，这些效果在 Bundle 插入后立即应用。值得注意的是，这使开发者能够定义返回像这样的层级结构的函数：

```rust
fn player(name: &str) -> impl Bundle {
    (
        Player,
        Name::new(name),
        children![
            (RightHand, children![Glove, Sword]),
            (LeftHand, children![Glove, Shield]),
        ]
    )
}

// later in your app
commands.spawn(player("Bob"));
```

在大多数情况下，出于人体工程学原因应优先使用 `children!` 宏。它展开为以下 API：

```rust
commands.spawn((
    Player,
    Children::spawn((
        Spawn((
            RightHand,
            Children::spawn((Spawn(Glove), Spawn(Sword))),
        )),
        Spawn((
            LeftHand,
            Children::spawn((Spawn(Glove), Spawn(Shield))),
        )),
    )),
));
```

有许多 spawn 包装器变体，提供额外的灵活性：

```rust
world.spawn((
    Name::new("Root"),
    Children::spawn((
        Spawn(Name::new("Child1")),   
        SpawnIter(["Child2", "Child3"].into_iter().map(Name::new)),
        SpawnWith(|parent: &mut ChildSpawner| {
            parent.spawn(Name::new("Child4"));
            parent.spawn(Name::new("Child5"));
        })
    )),
))
```

值得注意的是，此 API 适用于_所有_关系类型。例如，你可以像这样生成 `Likes` / `LikedBy` 关系层级结构（如上面关系部分定义的）：

```rust
world.spawn((
    Name::new("Monica"),
    LikedBy::spawn((
        Spawn(Name::new("Naomi")),
        Spawn(Name::new("Dwight")),
    ))
))
```

还有一个 `related!` 宏，它与 `children!` 做同样的事情，但适用于任何关系类型：

```rust
world.spawn((
    Name::new("Monica"),
    related!(LikedBy[
        Name::new("Naomi"),
        Name::new("Dwight"),
    ]),
))
```

此 API 还允许我们通过减少重新分配来优化层级结构构建时间，因为我们通常（除了 `SpawnWith` 等情况）可以静态确定实体将有多少关联实体，并在 `RelationshipTarget` 组件（例如 `Children`）中为它们预分配空间。

## 统一的 ECS 错误处理 [#](https://bevy.org/news/bevy-0-16/#unified-ecs-error-handling)

Authors:[@NthTensor](https://github.com/NthTensor), [@JeanMertz](https://github.com/JeanMertz), [@JaySpruce](https://github.com/JaySpruce), [@cart](https://github.com/cart), [@alice-i-cecile](https://github.com/alice-i-cecile)

PRs:[#16589](https://github.com/bevyengine/bevy/pull/16589), [#17043](https://github.com/bevyengine/bevy/pull/17043), [#17753](https://github.com/bevyengine/bevy/pull/17753), [#18144](https://github.com/bevyengine/bevy/pull/18144), [#18454](https://github.com/bevyengine/bevy/pull/18454)

Bevy 历史上一直将错误处理留给用户练习。许多 Bevy API 返回 Results / 错误，但 Bevy ECS 系统和命令本身并不支持返回 Results / 错误。此外，Bevy 提供了一些"简写"panic API，比解包结果需要更少的输入。这些鼓励开发者编写过于 panic 的行为，即使这不是期望的。

理想情况下开发者可以_选择_他们的系统是否应该 panic。例如，在开发时 panic 以快速发现错误（并强制你解决它们），但在部署到生产环境时只打印错误到控制台以避免破坏用户体验。

**Bevy 0.16** 引入了一个新的统一错误处理范式，帮助你发布无崩溃的游戏（和其他应用程序！），而不会牺牲 panic 带来的响亮快速开发。

我们优先考虑简单一致的设计，以及一些便于调试的额外功能：

- Bevy 系统、观察者和命令现在支持返回 [`Result`](https://docs.rs/bevy/0.16/bevy/ecs/error/type.Result.html)，它是 `Result<(), BevyError>` 的类型别名
- [`Result`](https://docs.rs/bevy/0.16/bevy/ecs/error/type.Result.html) 期望新的 [`BevyError`](https://docs.rs/bevy/0.16/bevy/ecs/error/struct.BevyError.html)，它接受任何错误类型（很像 [`anyhow`](https://docs.rs/anyhow/latest/anyhow/)）。这使得在系统/观察者/命令中使用 [`?` 运算符](https://doc.rust-lang.org/rust-by-example/std/result/question_mark.html)捕获和返回错误变得简单且符合人体工程学。
- [`BevyError`](https://docs.rs/bevy/0.16/bevy/ecs/error/struct.BevyError.html) 自动捕获[高质量自定义回溯](https://github.com/bevyengine/bevy/pull/18144)。默认情况下这些被过滤为仅保留基本要素，减少了 Rust 和 Bevy 内部的大量噪音。
- 系统、观察者和命令返回的错误现在由 [`GLOBAL_ERROR_HANDLER`](https://docs.rs/bevy/0.0.1/bevy/ecs/error/static.GLOBAL_ERROR_HANDLER.html) 处理。默认为 panic，但可以设置为任何内容（日志、panic、自定义逻辑等）。我们通常鼓励使用 panic 默认值进行开发，然后在部署到生产环境时切换为记录错误。

我们现在鼓励开发者在可能的情况下向上冒泡错误，而不是在错误发生的地方立即 panic。

而不是：

```rust
use bevy::prelude::*;

fn move_player(mut query: Query<&mut Transform, With<Player>>) {
    let mut player_transform = query.single_mut().unwrap();
    player_transform.translation.x += 1.0;
}
```

尝试：

```rust
use bevy::prelude::*;

fn move_player(mut query: Query<&mut Transform, With<Player>>) -> Result {
    let mut player_transform = query.single_mut()?;
    player_transform.translation.x += 1.0;
    Ok(())
}
```

## Bevy `no_std` 支持 [#](https://bevy.org/news/bevy-0-16/#bevy-no-std-support)

Authors:[@bushrat011899](https://github.com/bushrat011899)

PRs:[#15281](https://github.com/bevyengine/bevy/pull/15281), [#15463](https://github.com/bevyengine/bevy/pull/15463), [#15464](https://github.com/bevyengine/bevy/pull/15464), [#15465](https://github.com/bevyengine/bevy/pull/15465), [#15528](https://github.com/bevyengine/bevy/pull/15528), [#15810](https://github.com/bevyengine/bevy/pull/15810), [#16256](https://github.com/bevyengine/bevy/pull/16256), [#16633](https://github.com/bevyengine/bevy/pull/16633), [#16758](https://github.com/bevyengine/bevy/pull/16758), [#16874](https://github.com/bevyengine/bevy/pull/16874), [#16995](https://github.com/bevyengine/bevy/pull/16995), [#16998](https://github.com/bevyengine/bevy/pull/16998), [#17027](https://github.com/bevyengine/bevy/pull/17027), [#17028](https://github.com/bevyengine/bevy/pull/17028), [#17030](https://github.com/bevyengine/bevy/pull/17030), [#17031](https://github.com/bevyengine/bevy/pull/17031), [#17086](https://github.com/bevyengine/bevy/pull/17086), [#17470](https://github.com/bevyengine/bevy/pull/17470), [#17490](https://github.com/bevyengine/bevy/pull/17490), [#17491](https://github.com/bevyengine/bevy/pull/17491), [#17505](https://github.com/bevyengine/bevy/pull/17505), [#17507](https://github.com/bevyengine/bevy/pull/17507), [#17955](https://github.com/bevyengine/bevy/pull/17955), [#18061](https://github.com/bevyengine/bevy/pull/18061), [#18333](https://github.com/bevyengine/bevy/pull/18333)

Bevy 现在支持 `no_std` 目标，允许它在更广泛的平台上使用。

用户的早期报告显示 Bevy 可以在裸机桌面、嵌入式设备甚至像 GameBoy Advance 这样的复古游戏机上工作：

感谢 [Chris Biscardi](https://www.youtube.com/@chrisbiscardi) 创建了这个很棒的演示！

Bevy `no_std` 支持已经[讨论](https://github.com/bevyengine/bevy/discussions/705)了超过 4 年，但最初被放弃以避免管理 `no_std` 支持可能带来的额外复杂性。要与 `no_std` 兼容，你的 crate 及其_所有_依赖项也必须是 `no_std` 的。在超过一百个依赖项中协调这种支持根本不可行，更不用说失去对 Rust 标准库的访问了。

从那时起，Rust 对 `no_std` 的支持发生了巨大变化，在 [Rust 1.81](https://releases.rs/docs/1.81.0/#stabilized-apis) 中支持了 [`Error`](https://doc.rust-lang.org/stable/core/error/trait.Error.html) 等关键 API。从跟踪 issue [#15460](https://github.com/bevyengine/bevy/issues/15460) 和 [`no_std` 工作组](https://discord.com/channels/691052431525675048/1303128171352293410) 开始，Bevy 的各种 crate 被单独制作为 `no_std` 兼容（在可能的情况下）。为了协助这项工作，开发了 [`bevy_platform`](https://crates.io/crates/bevy_platform/)，目标是为 `std` 项提供有主见的替代方案。

这项工作在 **Bevy 0.16** 开发期间达到了一个重要的里程碑：我们主要的 `bevy` crate 支持 `no_std`。要在 `no_std` 平台上使用 Bevy，只需禁用默认 feature 并像任何其他 `no_std` 依赖一样使用 Bevy。

```toml
[dependencies]
bevy = { version = "0.16", default-features = false }
```

请注意，并非所有 Bevy 的功能都与 `no_std` 兼容。渲染、音频和资产是值得注意的缺失 API，你需要找到适合你平台的替代方案。但是，Bevy 强大的 [`Plugin`](https://docs.rs/bevy/latest/bevy/app/trait.Plugin.html) 系统允许社区为其特定平台构建这些功能的支持。

对于那些为 Bevy 开发库的社区成员，如果可以的话我们鼓励你尝试 `no_std` 支持！有一个新的 [`no_std` 库](https://github.com/bevyengine/bevy/tree/main/examples/no_std/library)示例，演示如何制作与 `std` 和 `no_std` 用户兼容的 crate，附有详细的注释和建议。在发布候选期间，相当多的库已经成功尝试了 `no_std` 支持，如 [`bevy_rand`](https://github.com/Bluefinger/bevy_rand) 和 [`bevy_replicon`](https://github.com/projectharmonia/bevy_replicon/tree/bevy-0.16-dev)。

确定哪些 `no_std` 目标支持 Bevy 仍在进行中。如果你有一个不寻常的平台想要尝试让 Bevy 工作，请查看 Bevy Discord 服务器上的 [`#unusual-platforms`](https://discord.com/channels/691052431525675048/1284885928837517432) 频道获取建议！

## 更快的 Transform 传播 [#](https://bevy.org/news/bevy-0-16/#faster-transform-propagation)

Authors:[@aevyrie](https://github.com/aevyrie)

PRs:[#17840](https://github.com/bevyengine/bevy/pull/17840), [#18589](https://github.com/bevyengine/bevy/pull/18589)

Bevy（和其他 3D 软件）中的 Transform 有两种形式：

1. [`GlobalTransform`](https://docs.rs/bevy/0.16.0/bevy/transform/components/struct.GlobalTransform.html)：表示对象的绝对世界空间位置、缩放和旋转。
2. [`Transform`](https://docs.rs/bevy/0.16.0/bevy/transform/components/struct.Transform.html)：表示对象相对于其父级的位置、缩放和旋转。这也称为"局部"变换。

为了计算每个对象的 [`GlobalTransform`](https://docs.rs/bevy/0.16.0/bevy/transform/components/struct.GlobalTransform.html)（这是渲染和物理所关心的！），我们需要递归地沿父级-子级层级结构组合所有对象的 [`Transform`](https://docs.rs/bevy/0.16.0/bevy/transform/components/struct.Transform.html)。这个过程称为变换传播，可能非常昂贵，尤其是在场景中有许多实体时。

**Bevy 0.16** 带来了_两个_令人印象深刻的性能优化：

1. **改进的并行化策略**：虽然我们已经跨线程分配工作，但更好的工作共享、跨树的并行化以及叶节点与非叶节点的拆分来优化缓存一致性产生了巨大差异。
2. **为没有对象移动的树节省工作**：关卡几何体和道具通常不会每帧移动，因此此优化适用于_许多_情况！我们现在将"脏位"沿层级结构向上传播到祖先；允许变换传播在遇到没有脏位的实体时忽略整个子树。

结果不言自明：综合来看，我们在巨大的（127,515 个对象）[Caldera Hotel](https://github.com/Activision/caldera) 场景（来自 Call of Duty: Warzone）上的测试显示，变换传播在 M4 Max Macbook 上的 **Bevy 0.15** 中花费了 1.1 ms，而在 **Bevy 0.16** 中经过这些更改后为 0.1 ms。即使是完全动态的场景（如我们的 [`many_foxes`](https://github.com/bevyengine/bevy/blob/main/examples/stress_tests/many_foxes.rs) 压力测试）由于改进的并行性也显著加快了。不错！这项工作对于更典型的硬件更为重要：在中低端硬件上的大型场景中，变换传播可能占用整个帧预算的 25%。哎呀！

虽然这是令人印象深刻的 11 倍性能改进，但节省时间的绝对量级是关键指标。在 60 FPS 时每帧约 16 ms，这为你_整个_游戏的 CPU 预算节省了 6%。使巨大的开放世界或极其复杂的 CAD 装配比以往任何时候都更加可行。

![A screenshot of a tracy histogram showing the effects of these changes on Caldera. 0.15 peaks at 1.1 ms, while 0.16 peaks at 0.1 ms.](https://bevy.org/news/bevy-0-16/caldera-transform-propagation-bench.jpg)

如果你对这些优化的详细技术细节感兴趣，请查看[代码本身](https://github.com/bevyengine/bevy/blob/b0c446739888705d3e95b640e9d13e0f1f53f06d/crates/bevy_transform/src/systems.rs#L12)。它的注释非常好，非常适合学习。

这些优化从 [`big_space`](https://github.com/aevyrie/big_space) crate 上游化（由该 crate 的作者！）

## 镜面着色和贴图 [#](https://bevy.org/news/bevy-0-16/#specular-tints-and-maps)

Authors:[@pcwalton](https://github.com/pcwalton)

PRs:[#14069](https://github.com/bevyengine/bevy/pull/14069)

如果你对光线有眼光（或接受过视觉艺术训练），你会注意到闪亮的弯曲表面上有额外明亮的光斑。那就是镜面高光！在 **Bevy 0.16** 中，我们实现了标准物理渲染（PBR）镜面高光的特性：着色其颜色的能力。

![A shiny floating sphere with an iridescent multicolored sheen. It reminds you of a Christmas tree ornament. You can see a reflection of a city scene in it.](https://bevy.org/news/bevy-0-16/specular-tint-sphere.jpg)

这可以通过简单地设置对象 [`StandardMaterial`](https://docs.rs/bevy/0.16/bevy/pbr/struct.StandardMaterial.html) 上的 `specular_tint` 字段在材质上均匀完成。

像许多其他材质属性（颜色、法线、发光度、粗糙度等）一样，这可以通过使用纹理贴图在材质上变化，该贴图通过二维 UV 空间图像描述此属性的变化方式。

贴图相对昂贵：你需要一整张 2D 图像，而不是单个浮点数或颜色，并且 GPU 对每个材质可以使用的纹理数量有限制，除非它们支持无绑定纹理。因此，镜面贴图默认关闭，并通过 `pbr_specular_textures` Cargo feature 门控。

为了支持这项工作，我们现在支持 KHR_materials_specular glTF 扩展，允许美术在 Blender 等 3D 建模工具中设置这些属性，然后将其导入 Bevy。

## 实验性 WESL 着色器 [#](https://bevy.org/news/bevy-0-16/#experimental-wesl-shaders)

Authors:[@tychedelia](https://github.com/tychedelia)

PRs:[#17953](https://github.com/bevyengine/bevy/pull/17953)

Bevy 继续在图形技术的最前沿生活。今天，Bevy 支持 [WESL](https://wesl-lang.dev/) 着色器！

大多数 Bevy 着色器用 [WGSL](https://www.w3.org/TR/WGSL/) 编写，一种为简洁而构建的现代着色器语言。但虽然 WGSL 在着色器语言中相当简单，它在高级特性方面也有很多不足。目前，Bevy 有自己的 WGSL 扩展版本，添加了条件编译、文件间导入和其他有用特性的支持。

WESL 是一种全新的着色器语言，扩展了 WGSL（通常以类似于 Bevy 的方式），旨在将常见的语言便利带到 GPU。WESL 不仅包括条件编译和文件间导入，而且还在发展支持泛型、包管理器等。

值得注意的是，WESL 仍处于相对早期的开发阶段，并非所有功能都完全正常，也并非所有功能都在 Bevy 中得到支持。因此，WESL 支持通过 cargo feature `shader_format_wesl` 门控，默认禁用。

尽管有额外的功能，WESL 很容易在现有 WGSL 着色器之上使用。因为它是 WGSL 的超集（WGSL 是有效的 WESL）。这使得将现有 WGSL 迁移到 WESL 很容易，但值得一提的是 Bevy 自己的"扩展 WGSL 语法"需要移植到其 WESL 对应物。WESL 团队（帮助编写了这些说明！）正在积极听取反馈，Bevy 也是。如果你确实选择在 WGSL 之外或替代 WGSL 使用 WESL，你的想法、功能请求和遇到的任何痛点可以在[这里](https://github.com/wgsl-tooling-wg/wesl-rs)分享。

如果你有兴趣尝试 WESL，请查看新的 [Material - WESL](https://bevy.org/examples/shaders/shader-material-wesl/) 示例。在生产环境中使用之前，请务必查看原始 [PR](https://github.com/bevyengine/bevy/pull/17953) 获取完整的注意事项列表。

WESL 是着色器语言的一个令人兴奋的前沿。你可以在[这里](https://wesl-lang.dev/)跟踪它们的进度和计划。

## 虚拟几何改进 [#](https://bevy.org/news/bevy-0-16/#virtual-geometry-improvements)

Authors:[@JMS55](https://github.com/JMS55)

PRs:[#17765](https://github.com/bevyengine/bevy/pull/17765), [#16947](https://github.com/bevyengine/bevy/pull/16947), [#16941](https://github.com/bevyengine/bevy/pull/16941)

虚拟几何（`meshlet` cargo feature）是 Bevy 的 Nanite 类渲染系统，允许比其他方式更高水平的几何密度，并将美术从手动创建 LOD 中解放出来。

在 **Bevy 0.16** 中，虚拟几何由于新的算法和 GPU API 获得了一些性能改进。

更多细节请阅读[作者的博客文章](https://jms55.github.io/posts/2025-03-27-virtual-geometry-bevy-0-16)。

用户不需要重新生成他们的 [`MeshletMesh`](https://docs.rs/bevy/0.16/bevy/pbr/experimental/meshlet/struct.MeshletMesh.html) 资产，但建议这样做以利用 **Bevy 0.16** 中改进的聚类算法。

![Screenshot of the meshlet example](https://bevy.org/news/bevy-0-16/meshlet_bunnies.jpg)

## 不可变组件 [#](https://bevy.org/news/bevy-0-16/#immutable-components)

Authors:[@bushrat011899](https://github.com/bushrat011899)

PRs:[#16372](https://github.com/bevyengine/bevy/pull/16372), [#16668](https://github.com/bevyengine/bevy/pull/16668), [#18532](https://github.com/bevyengine/bevy/pull/18532)

**Bevy 0.16** 添加了**不可变组件**的概念：一旦插入就无法更改的数据（例如你不能查询 `&mut MyComponent`）。修改不可变组件的唯一方法是在其上插入一个新实例。

可以通过添加 `#[component(immutable)]` 属性使组件不可变：

```rust
#[derive(Component)]
#[component(immutable)]
pub struct MyComponent(pub u32);
```

通过接受此限制，我们可以确保组件生命周期钩子和观察者（add/remove/insert/replace）捕获发生的_每个_更改，使它们能够维护复杂的不变量。

为了说明这一点，考虑以下示例，我们跟踪所有 `SumMe` 组件的全局总和：

```rust
#[derive(Component)]
#[component(immutable)]
pub struct SumMe(pub u32);

// This will always hold the sum of all `SumMe` components on entities.
#[derive(Resource)]
struct TotalSum(u32);

// This observer will trigger when spawning or inserting a SumMe component
fn add_when_inserting(
    trigger: Trigger<OnInsert, SumMe>,
    query: Query<&SumMe>,
    mut total_sum: ResMut<TotalSum>,
) {
    if let Ok(sum_me) = query.get(trigger.target()) {
        total_sum.0 += sum_me.0;
    }
}

// This observer will trigger when despawning or removing a SumMe component
fn subtract_when_removing(
    trigger: Trigger<OnRemove, SumMe>,
    query: Query<&SumMe>,
    mut total_sum: ResMut<TotalSum>,
) {
    if let Ok(sum_me) = query.get(trigger.target()) {
        total_sum.0 -= sum_me.0;
    }
}

// Changing this to `&mut SumMe` would fail to compile!
fn modify_values(mut commands: Commands, query: Query<(Entity, &SumMe)>) {
    for (entity, sum_me) in query.iter() {
        // We can read the value, but not write to it.
        let current_value = sum_me.0;
        // This will overwrite: indirectly mutating the value
        // and triggering both observers: removal first, then insertion
        commands.entity(entity).insert(SumMe(current_value + 1));
    }
}
```

虽然不可变组件是一个小众工具，但它们非常适合很少更改（或小数量）的组件，其中正确性是关键。例如，**Bevy 0.16** 使用不可变组件（和 hooks！）作为我们闪亮新**关系**的基础，以确保关系的两端（例如 [`ChildOf`](https://docs.rs/bevy/0.16/bevy/ecs/hierarchy/struct.ChildOf.html) 和 [`Children`](https://docs.rs/bevy/0.16/bevy/ecs/hierarchy/struct.Children.html)）保持同步。

我们渴望使用这些新工具开发一等索引解决方案，并期待听到你的想法。敬请关注；我们只是触及了表面！

## 实体克隆 [#](https://bevy.org/news/bevy-0-16/#entity-cloning)

Authors:[@eugineerd](https://github.com/eugineerd), [@JaySpruce](https://github.com/JaySpruce)

PRs:[#16132](https://github.com/bevyengine/bevy/pull/16132), [#16826](https://github.com/bevyengine/bevy/pull/16826)

Bevy 现在一等支持克隆实体。虽然之前可以使用反射和 `ReflectComponent` 功能来完成此操作，但常见实现速度较慢且需要注册所有可克隆组件。使用 **Bevy 0.16**，实体克隆被原生支持，只需向组件添加 `#[derive(Clone)]` 即可使其可克隆。

```rust
#[derive(Component, Clone)]
#[require(MagicalIngredient)]
struct Potion;

#[derive(Component, Default, Clone)]
struct MagicalIngredient {
    amount: f32,
}

fn process_potions(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut potions: Query<(Entity, &mut MagicalIngredient), With<Potion>>,
) {
    // Create a new potion
    if input.just_pressed(KeyCode::KeyS) {
        commands.spawn(
            (Name::new("Simple Potion"), Potion)
        );
    }
    // Add as much magic as we want
    else if input.just_pressed(KeyCode::KeyM) {
        for (_, mut ingredient) in potions.iter_mut() {
            ingredient.amount += 1.0
        }
    }
    // And then duplicate all the potions!
    else if input.just_pressed(KeyCode::KeyD) {
        for (potion, _) in potions.iter() {
            commands.entity(potion).clone_and_spawn();
        }
    }
}

```

`clone_and_spawn` 生成一个具有所有可克隆组件的新实体，跳过那些无法克隆的组件。如果你的用例需要不同的行为，有更专门的方法：

- `clone_components` 将组件从源实体克隆到指定目标实体，而不是生成新实体。
- `move_components` 在将组件克隆到目标实体后从源实体移除组件。
- `clone_and_spawn_with` 和 `clone_with` 允许在执行克隆之前通过访问 `EntityClonerBuilder` 来自定义克隆行为。

`EntityClonerBuilder` 可用于配置克隆的执行方式——例如，通过过滤应该克隆哪些组件、修改如何克隆 `required` 组件，或控制是否应递归克隆通过关系链接的实体。

一个重要提示：具有泛型类型参数的组件默认不可克隆。对于这些情况，你应该向组件添加 `#[derive(Reflect)]` 和 `#[reflect(Component)]` 并注册它以使实体克隆功能正常工作。

```rust
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
struct GenericComponent<T> {
    // ...
}

fn main(){
    // ...
    app.register_type::<GenericComponent<i32>>;
    // ...
}
```

如果你计划实现组件的自定义克隆行为或需要更多解释，请参阅 `EntityCloner` 的文档。

## 实体禁用/默认查询过滤器 [#](https://bevy.org/news/bevy-0-16/#entity-disabling-default-query-filters)

Authors:[@NiseVoid](https://github.com/NiseVoid), [@alice-i-cecile](https://github.com/alice-i-cecile)

PRs:[#13120](https://github.com/bevyengine/bevy/pull/13120), [#17514](https://github.com/bevyengine/bevy/pull/17514), [#17768](https://github.com/bevyengine/bevy/pull/17768)

**Bevy 0.16** 添加了通过添加 [`Disabled`](https://docs.rs/bevy/0.16/bevy/ecs/entity_disabling/struct.Disabled.html) 组件来禁用实体的能力。这（默认）将从寻找它的系统和查询中隐藏实体（及其所有组件）。

这是使用新添加的**默认查询过滤器**实现的。这些按其名称所示工作：每个查询都将表现得好像它们有 `Without<Disabled>` 过滤器，除非它们明确提及 [`Disabled`](https://docs.rs/bevy/0.16/bevy/ecs/entity_disabling/struct.Disabled.html)（通常通过 `With<Disabled>` 或 `Has<Disabled>` 参数）。

因为这是通用的，开发者也可以定义额外的禁用组件，可以通过 [`App::register_disabling_component`] 注册。拥有多个不同的禁用组件可能很有用，如果你希望每种禁用形式都有自己的语义/行为：你可能使用此功能来隐藏网络实体、冻结屏幕外区块中的实体、创建加载并准备好生成的预制体实体集合，或其他完全不同的东西。

请注意，为了最大控制和明确性，只有你直接添加禁用组件的实体被禁用。它们的子级或其他关联实体不会自动禁用！

要禁用实体_及其_子级，请考虑新的 [`Commands::insert_recursive`](https://docs.rs/bevy/0.16/bevy/prelude/struct.EntityCommands.html#method.insert_recursive) 和 [`Commands::remove_recursive`](https://docs.rs/bevy/0.16/bevy/prelude/struct.EntityCommands.html#method.remove_recursive)。

## 更好的位置跟踪 [#](https://bevy.org/news/bevy-0-16/#better-location-tracking)

Authors:[@SpecificProtagonist](https://github.com/SpecificProtagonist), [@chescock](https://github.com/chescock)

PRs:[#15607](https://github.com/bevyengine/bevy/pull/15607), [#16047](https://github.com/bevyengine/bevy/pull/16047), [#16778](https://github.com/bevyengine/bevy/pull/16778), [#17075](https://github.com/bevyengine/bevy/pull/17075), [#17602](https://github.com/bevyengine/bevy/pull/17602)

Bevy 的统一数据模型允许内省和调试工具为整个引擎工作：例如，上次发布的 `track_change_detection` feature flag 让你[自动跟踪](https://bevy.org/news/bevy-0-15/#change-detection-source-location-tracking)哪行源代码插入或更改了任何组件或资源。

现在它还跟踪哪些代码

- 触发了钩子或观察者：`HookContext.caller`、`Trigger::caller()`
- 发送了事件：`EventId.caller`
- 生成或销毁了实体（直到实体索引被重用）：`EntityRef::spawned_by()`、`Entities::entity_get_spawned_or_despawned_by()`

作为副作用，这在某些情况下会导致更好的错误消息，例如这个明显改进的销毁消息：

`Entity 0v1 was despawned by src/main.rs:10:11`

由于不再适用其旧名称，feature flag 现在称为 `track_location`。

## 文本阴影 [#](https://bevy.org/news/bevy-0-16/#text-shadows)

Authors:[@ickshonpe](https://github.com/ickshonpe)

PRs:[#17559](https://github.com/bevyengine/bevy/pull/17559)

Bevy UI 现在支持文本阴影：

![image of button with text shadow](https://bevy.org/news/bevy-0-16/text_shadow.jpg)

只需将 [`TextShadow`](https://docs.rs/bevy/0.16/bevy/prelude/struct.TextShadow.html) 组件添加到任何 [`Text`](https://docs.rs/bevy/0.16/bevy/prelude/struct.Text.html) 实体。[`TextShadow`](https://docs.rs/bevy/0.16/bevy/prelude/struct.TextShadow.html) 可用于配置阴影的偏移和颜色。

## 输入焦点改进 [#](https://bevy.org/news/bevy-0-16/#input-focus-improvements)

Authors:[@viridia](https://github.com/viridia), [@alice-i-cecile](https://github.com/alice-i-cecile)

PRs:[#15611](https://github.com/bevyengine/bevy/pull/15611), [#16795](https://github.com/bevyengine/bevy/pull/16795)

"输入焦点"的概念对于无障碍性很重要。视觉或运动障碍的用户可能难以使用鼠标；通过焦点导航他们可以仅使用键盘控制 UI。这不仅帮助残障用户，也帮助高级用户。

同样的通用思想可以应用于游戏控制器和其他输入设备。这在游戏中的屏幕上控件数量超过游戏手柄上的按钮数量时经常看到，或者对于具有物品网格的复杂"库存"页面。

`bevy_a11y` crate 很久以来就有一个 `Focus` 资源，但它的位置使其难以使用。这已被一个新的 `InputFocus` 资源取代，它位于新 crate `bevy_input_focus` 中，现在包含一堆辅助函数，使实现焦点感知 widget 变得更容易。

这个新 crate 还支持"可插拔焦点导航策略"，目前有两种：一种是"Tab 导航"策略，使用 TAB 键实现传统的桌面顺序导航，另一种是更"控制台风格"的 2D 导航策略，使用空间搜索和显式导航链接的混合。

## 保留 `Gizmo`s [#](https://bevy.org/news/bevy-0-16/#retained-gizmos)

Authors:[@tim-blackbird](https://github.com/tim-blackbird)

PRs:[#15473](https://github.com/bevyengine/bevy/pull/15473)

在 Bevy 的早期版本中，gizmos 始终以"即时模式"风格渲染：它们只渲染一帧就消失了。这非常适合原型设计，但也有性能成本。

使用保留的 gizmos，你现在可以生成持久存在的 gizmos，实现更高的性能！对于一组静态线条，我们测量到性能提升了约 65-80 倍！

例如，以下是生成持久球体的方法：

```rust
fn setup(
    mut commands: Commands,
    mut gizmo_assets: ResMut<Assets<GizmoAsset>>
) {
    let mut gizmo = GizmoAsset::default();

    // A sphere made out of one million lines!
    gizmo
        .sphere(default(), 1., CRIMSON)
        .resolution(1_000_000 / 3);

    commands.spawn(Gizmo {
        handle: gizmo_assets.add(gizmo),
        ..default()
    });
}
```

即时模式 `Gizmos` API 如果你想要的话仍然存在。它仍然是轻松调试的好选择。

## 透明精灵选取 [#](https://bevy.org/news/bevy-0-16/#transparent-sprite-picking)

Authors:[@vandie](https://github.com/vandie)

PRs:[#16388](https://github.com/bevyengine/bevy/pull/16388)

在大多数情况下，当你在制作游戏时，你不希望点击精灵的透明部分算作对该精灵的点击。如果你正在制作一个具有大量重叠精灵（具有透明区域）的 2D 游戏，这一点尤其正确。

在以前的 Bevy 版本中，精灵交互作为简单的边界框检查处理。如果你的光标在精灵框的边界内，它会阻止与其后面精灵的交互，即使精灵的区域完全透明。

在 **Bevy 0.16** 中，当与实体精灵的透明部分交互时，这些交互将传递到其下方的实体。无需更改你的代码库！

默认情况下，对于 alpha 值为 `0.1` 或更低的精灵部分将发生穿透。如果你希望恢复到精灵的矩形检查或更改阈值，可以通过覆盖 `SpritePickingSettings` 来完成。

```rust
// Change the alpha value cutoff to 0.2 instead of the default 0.1
app.insert_resource(SpritePickingSettings {
  picking_mode: SpritePickingMode::AlphaThreshold(0.2)
});

// Revert to Bounding Box picking mode
app.insert_resource(SpritePickingSettings {
  picking_mode: SpritePickingMode::BoundingBox
});
```

## 反射：函数重载（泛型和可变参数函数）[#](https://bevy.org/news/bevy-0-16/#reflection-function-overloading-generic-variadic-functions)

Authors:[@MrGVSV](https://github.com/MrGVSV)

PRs:[#15074](https://github.com/bevyengine/bevy/pull/15074)

**Bevy 0.15** 向 `bevy_reflect`（Bevy 的类型反射 crate）添加了反射函数的支持。这允许 Rust 函数使用在运行时生成的参数列表动态调用——而且是安全的！

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

let reflect_add: DynamicFunction = add.into_function();

let args = ArgList::new()
  .push_owned(25_i32)
  .push_owned(75_i32);

let result = reflect_add.call(args).unwrap();

let sum = result.unwrap_owned().try_take::<i32>().unwrap();
assert_eq!(sum, 100);
```

然而，由于 Rust 的限制，无法使这些动态函数泛型化。这意味着必须为所有期望的单态化创建单独的函数并在运行时手动映射。

```rust
fn add<T: Add<Output=T>>(a: T, b: T) -> T {
    a + b
}

let reflect_add_i32 = add::<i32>.into_function();
let reflect_add_u32 = add::<u32>.into_function();
let reflect_add_f32 = add::<f32>.into_function();
// ...
```

虽然原始 Rust 限制仍然存在，Bevy 0.16 通过添加对函数重载的支持改善了开发者体验。术语"函数重载"可能对来自其他编程语言的开发者很熟悉，但本质上它意味着一个函数可以有多个参数签名。

这允许我们简化前面的示例：

```rust
let reflect_add = add::<i32>.into_function()
  .with_overload(add::<u32>)
  .with_overload(add::<f32>);
```

第一个 `add::<i32>` 作为基础情况，每个重载在其上操作。当函数被调用时，根据提供参数的类型选择相应的重载。

并且由于函数重载允许多个参数签名的事实，这也意味着我们可以定义接受可变数量参数的函数，通常称为"可变参数函数"。

这允许一些有趣的用例：

```rust
#[derive(Reflect)]
struct Player {
    name: Option<String>,
    health: u32,
}

// Creates a `Player` with one of the following based on the provided arguments:  
// - No name and 100 health  
// - A name and 100 health  
// - No name and custom health  
// - A name and custom health
let create_player = (|| Player {
    name: None,
    health: 100,
  })
  .into_function()
  .with_overload(|name: String| Player {
    name: Some(name),
    health: 100,
  })
  .with_overload(|health: u32| Player {
    name: None,
    health
  })
  .with_overload(|name: String, health: u32| Player {
    name: Some(name),
    health,
  });
```

## 曲线导数 [#](https://bevy.org/news/bevy-0-16/#curve-derivatives)

Authors:[@mweatherley](https://github.com/mweatherley)

PRs:[#16503](https://github.com/bevyengine/bevy/pull/16503)

`bevy_math` 收集了大量的曲线和处理曲线的方法，这些方法对从动画到颜色渐变再到游戏逻辑的一切都很有用。

你可能想要对曲线做的最自然和最重要的事情之一是检查它的**导数**：它的变化速率。你甚至可能想要它的**二阶导数**：变化速率的变化速率。

在 **Bevy 0.16** 中，你现在可以轻松计算这些！

```rust
let points = [
    vec2(-1.0, -20.0),
    vec2(3.0, 2.0),
    vec2(5.0, 3.0),
    vec2(9.0, 8.0),
];

// A cubic spline curve that goes through `points`.
let curve = CubicCardinalSpline::new(0.3, points).to_curve().unwrap();

// Calling `with_derivative` causes derivative output to be included in the output of the curve API.
let curve_with_derivative = curve.with_derivative();

// A `Curve<f32>` that outputs the speed of the original.
let speed_curve = curve_with_derivative.map(|x| x.derivative.norm());
```

我们已经为大多数原生曲线类型实现了所需的 trait：样条线、直线和各种复合曲线。接受任意函数的曲线不涵盖（构建你自己的专用曲线类型），因为 Rust 没有一等可微函数的概念！

## `AssetChanged` 查询过滤器 [#](https://bevy.org/news/bevy-0-16/#assetchanged-query-filter)

Authors:[@tychedelia](https://github.com/tychedelia)

PRs:[#16810](https://github.com/bevyengine/bevy/pull/16810)

在 Bevy 中，"资产"是我们想要持有单个副本的数据，即使被许多实体使用：声音、图像和 3D 模型等。像 `Image` 这样的资产存储在 [`Assets<Image>`] 资源中。然后实体具有像 [`Sprite`](https://docs.rs/bevy/0.16/bevy/prelude/struct.Sprite.html) 这样的组件，它内部持有 [`Handle<Image>`]，标识该实体使用哪个资产。

虽然这对于在制作塔防游戏时避免在内存中存储一万个食人魔网格非常有效，但这种相对间接的模式使得很难确定你依赖的资产何时发生了变化。这是因为 [`Handle<T>`](https://docs.rs/bevy/0.16/bevy/asset/enum.Handle.html) 可能会更改，指向新资产，或者 [`Assets<T>`](https://docs.rs/bevy/0.16/bevy/asset/struct.Assets.html) 中的基础资产可能会更改，修改底层数据。

虽然查询 `Changed<Sprite>` 会捕获句柄的更改，但它_不会_捕获基础资产的更改，导致难以检测的令人沮丧的 bug，因为它们仅在事物以不寻常的方式更改时才发生。

为了解决这个问题，我们添加了 [`AssetChanged`](https://docs.rs/bevy/0.16/bevy/asset/prelude/struct.AssetChanged.html) 查询过滤器，它适用于实现新的 [`AsAssetId`](https://docs.rs/bevy/0.16/bevy/asset/trait.AsAssetId.html) trait 的任何类型（如 [`Sprite`](https://docs.rs/bevy/0.16/bevy/prelude/struct.Sprite.html)）。像 `Query<&mut Aabb, With<AssetChanged<Mesh3d>>>` 这样的东西现在 Just Works™，允许你在基础资产因任何原因更改时重新计算数据。

## 资产处理不再自动生成 .meta 文件 [#](https://bevy.org/news/bevy-0-16/#asset-processing-no-longer-auto-generates-meta-files)

Authors:[@andriyDev](https://github.com/andriyDev)

PRs:[#17216](https://github.com/bevyengine/bevy/pull/17216)

在 **Bevy 0.12** 中，我们[引入了 Assets V2](https://bevy.org/news/bevy-0-12/#bevy-asset-v2)。这是对我们资产系统的完全重写，添加了"资产预处理"（在开发时将资产处理成更高效形式的能力，在部署游戏之前）等功能。然而这为项目中的每个资产创建了"meta 文件"——这意味着当你开始使用资产预处理时，你的整个 `assets` 文件夹会自动填满这些 meta 文件（即使对于不需要任何预处理的资产）。

为了减轻这种痛苦，启用资产预处理不再自动写入 meta 文件！这使得启用资产预处理并逐步采用变得更加容易。

此外，我们添加了 `AssetServer::write_default_loader_meta_file_for_path` 和 `AssetProcessor::write_default_meta_file_for_path`，允许用户在必要时显式生成资产的默认 meta 文件。

考虑使用以下方式启用资产处理：

```rust
app.add_plugins(DefaultPlugins.set(
    AssetPlugin {
        mode: AssetMode::Processed,
        ..default()
    }
));
```

启用 `bevy/asset_processor` feature 将自动为你处理文件。详情请参见[资产处理示例](https://github.com/bevyengine/bevy/blob/main/examples/asset/processing/asset_processing.rs)！

## 网格标签 [#](https://bevy.org/news/bevy-0-16/#mesh-tags)

Authors:[@tychedelia](https://github.com/tychedelia)

PRs:[#17648](https://github.com/bevyengine/bevy/pull/17648)

Bevy 强大支持[自动实例化](https://bevy.org/examples/shaders/automatic-instancing/)任何共享相同网格和材质的实体。然而，有时引用并非所有材质实例都相同的数据仍然很有用。以前，这需要编写大量自定义渲染代码或通过创建更多材质来放弃自动实例化的性能优势。

新的 `MeshTag` 组件允许向网格-材质实体添加自定义 `u32` 标签，可以在材质的顶点着色器中引用。结合存储纹理或 **Bevy 0.15** 中添加的 [`ShaderStorageBuffer` 资产](https://bevy.org/news/bevy-0-15/#shader-storage-buffer-asset)，这提供了一种灵活的新机制来按实例访问外部数据或以其他方式标记你的网格实例。

使用网格-材质实体生成网格标签：

```rust
commands.spawn((
    // Clone a mesh and material handle to enable automatic instancing 
    Mesh3d(mesh_handle.clone()),
    MeshMaterial3d(material_handle.clone()),
    // The mesh tag can be any `u32` that is meaningful to your application, like
    // a particular variant of an enum or an index into some external data 
    MeshTag(1234), 
));
```

在顶点着色器中查找标签：

```wgsl
#import bevy_pbr::mesh_functions

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    // Lookup the tag for the given mesh
    let tag = mesh_functions::get_tag(vertex.instance_index);

    // Index into a storage buffer, read a storage texture texel, etc...
}

```

## Trace 中的 GPU 时间戳 [#](https://bevy.org/news/bevy-0-16/#gpu-timestamps-in-traces)

Authors:[@JMS55](https://github.com/JMS55)

PRs:[#18490](https://github.com/bevyengine/bevy/pull/18490)

如果你想优化某事，你必须首先衡量和理解它。在查看应用程序性能时，[tracy](https://github.com/wolfpld/tracy) 是我们选择的工具。它让我们清楚地了解工作需要多长时间、相对于其他工作在每帧中何时发生以及各种线程如何使用。请阅读我们的[性能分析文档](https://github.com/bevyengine/bevy/blob/main/docs/profiling.md)开始！

但直到现在，它有一个关键限制：GPU 上完成的工作没有显示，迫使开发者打开专门的 GPU 工具（如 [NSight](https://developer.nvidia.com/nsight-systems) 或 [RenderDoc](https://renderdoc.org/)）并努力拼凑出直觉。

在 0.16 中，我们将 [Bevy 0.14 中添加的渲染诊断](https://bevy.org/news/bevy-0-14/#tools-for-profiling-gpu-performance) 连接到 [tracy](https://github.com/wolfpld/tracy)，在单个方便的地方创建 Bevy 应用程序中所有工作的完整图景。

也就是说，我们目前只检测了几个通道。虽然我们将来会改进这一点，但你需要在自己的自定义渲染代码中添加 span，而专门的 GPU 诊断工具将始终更强大：捕获所有 GPU 相关工作并提供更详细的信息。

特别感谢 [@wumpf](https://github.com/Wumpf) 在出色的 [wgpu-profiler](https://github.com/Wumpf/wgpu-profiler) 工具中开创了这项工作，并演示了如何将 [wgpu] 和 [tracy](https://github.com/wolfpld/tracy) 连接在一起。

## docs.rs 上的 Trait 标签 [#](https://bevy.org/news/bevy-0-16/#trait-tags-on-docs-rs)

Authors:[@SpecificProtagonist](https://github.com/SpecificProtagonist), [@Jondolf](https://github.com/Jondolf)

PRs:[#17758](https://github.com/bevyengine/bevy/pull/17758)

Bevy 提供了几个核心 trait 来定义类型如何使用。例如，要将数据附加到实体，它必须实现 `Component`。在阅读 Rust API 文档时，确定类型是否为 `Component`（或其他核心 Bevy trait）需要滚动浏览所有文档直到找到相关 trait。但在 **Bevy 0.16** 中，在 docs.rs 上 Bevy 现在显示标签指示类型实现了哪些核心 Bevy trait：

![Rustdoc showing a "Component" label below "Camera" type](https://bevy.org/news/bevy-0-16/trait-tags.jpg)

这适用于 trait `Plugin` / `PluginGroup`、`Component`、`Resource`、`Asset`、`Event`、`ScheduleLabel`、`SystemSet`、`SystemParam`、`Relationship` 和 `RelationshipTarget`。

这目前通过 [javascript](https://github.com/bevyengine/bevy/tree/release-0.16.0/docs-rs) 完成。这对 Bevy 以外的其他 Rust 框架也应该有用，我们将与 rustdoc 团队合作，了解如何将其内置并更加通用。如果你感兴趣请联系我们，这样我们可以开始一个规范！

## 下一步是什么？[#](https://bevy.org/news/bevy-0-16/#what-s-next)

上述功能可能很棒，但 Bevy 还有什么正在进行中？深入时间的迷雾中（当你的团队几乎全是志愿者时，预测_格外_困难！），我们可以看到一些令人兴奋的工作正在成型：

- **改进的观察者 API：** 观察者非常受欢迎，但有一些奇怪的怪癖。我们正在寻找消除这些的方法，使它们成为编写 UI 一次性逻辑的最简单方式。
- **资源即实体：** 如果钩子、观察者、关系等能与资源一起工作那就太好了。与其重复所有代码，我们希望[在内部使它们成为单例实体上的组件](https://github.com/bevyengine/bevy/pull/17485)。
- **.bsn 文件格式和 bsn! 宏：** 随着基础的奠定（必需组件、改进的生成和关系！），是时候构建 [bevy#14437](https://github.com/bevyengine/bevy/discussions/14437) 中描述的简洁而健壮的 Bevy 原生场景格式（和匹配的宏）了。
- **光照纹理：** 也称为"光照饼干"，它们非常适合从斑驳的阳光到阴影盒的一切。
- **NVIDIA 深度学习超级采样：** DLSS 是一种神经网络驱动的时间抗锯齿和上采样方法，适用于 NVIDIA RTX GPU。我们正在努力将 DLSS 集成到 Bevy 中，以在支持的平台上提供比 Bevy 当前 TAA 更便宜且更高质量的抗锯齿解决方案。
- **统一的体积系统：** 体积光、雾、级联阴影贴图和大气散射：有大量的渲染功能根本上关心开放空气（或水！）的体积的光学属性。我们希望统一和扩展这些功能，使物理渲染更易于使用、更美丽。
- **光线追踪基础：** 硬件加速光线追踪正风靡一时，在 `wgpu` 的帮助下我们准备迈出第一步，走向动态光线追踪全局照明的世界。
- **更多以游戏为重点的示例：** 新用户继续涌向 Bevy，需要最新的学习材料。我们以 API 为重点的示例方法不够：我们需要开始演示如何使用 Bevy 完成常见的游戏开发任务，如制作库存、保存用户偏好或在地图上放置结构。