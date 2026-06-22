# Bevy 0.6

## 发布于 2022 年 1 月 8 日，作者 Carter Anderson ( ![一个带猫耳挥舞触手的剪影，即 Octocat：GitHub 的吉祥物和标志](https://bevy.org/assets/github_grey.svg) [@cart](https://www.github.com/cart) ![一个带圆角矩形的右指三角形；YouTube 的标志](https://bevy.org/assets/youtube_grey.svg) [cartdev](https://www.youtube.com/cartdev) )

![由 @mockersf 在新 Bevy 渲染器中渲染的 Lumberyard Bistro 场景](https://bevy.org/news/bevy-0-6/bistro_night.png)

由 @mockersf 在新 Bevy 渲染器中渲染的 Lumberyard Bistro 场景

感谢 **170** 位贡献者、**623** 个拉取请求以及我们的[**慷慨赞助商**](https://github.com/sponsors/cart），我很高兴地在 [crates.io](https://crates.io/crates/bevy) 上宣布 **Bevy 0.6** 发布！

对于那些还不知道的人，Bevy 是一个用 Rust 构建的、令人耳目一新的简单数据驱动游戏引擎。你可以查看[快速入门指南](https://bevy.org/learn/quick-start/introduction)来开始使用。Bevy 也是永久免费和开源的！你可以在 GitHub 上获取完整的[源代码](https://github.com/bevyengine/bevy)。查看 [Bevy Assets](https://bevy.org/assets) 可以找到社区开发的插件、游戏和学习资源合集。

要将现有的 Bevy 应用或插件更新到 **Bevy 0.6**，请查看我们的 [0.5 到 0.6 迁移指南](https://bevy.org/learn/migration-guides/0.5-0.6/)。

本次发布包含了大量的改进、错误修复和生活质量优化。以下是一些亮点：

- 一个全新的现代渲染器，更美观、更快，且更易于扩展
- 方向光和点光源阴影
- 分簇前向渲染
- 视锥体裁剪
- 显著更快的精灵渲染，样板代码更少
- 原生 WebGL2 支持。你可以通过[在浏览器中运行 Bevy 示例](https://bevy.org/examples)来测试！
- 高级自定义材质
- 更强大的着色器：预处理器、导入、WGSL 支持
- Bevy ECS 的人体工程学和性能改进。不再需要 `.system()`！

继续阅读以了解详情！

## 新 Bevy 渲染器 [#](https://bevy.org/news/bevy-0-6/#the-new-bevy-renderer)

**Bevy 0.6** 引入了一个全新的现代渲染器，它：

- **更快**：更高的并行度，更少的每个实体计算量，更高效的 CPU->GPU 数据流，以及（即将启用的）管线化渲染
- **更美观**：我们随新渲染器一起发布了多项图形改进，例如方向光和点光源阴影、分簇前向渲染（因此你可以在场景中使用更多灯光）以及球形区域光。我们还有大量正在开发的新功能（级联阴影贴图、泛光、粒子、阴影滤镜等！）
- **更简单**：更少的抽象层，更简单的数据流，改进的底层、中层和高层接口，直接的 wgpu 访问
- **核心模块化**：标准化的 2d 和 3d 核心管线、可扩展的渲染阶段和视图、可组合的实体/组件驱动的绘制函数、着色器导入、通过"子图"实现的可扩展和可重复的渲染管线
- **行业验证**：我们从经过实战考验的渲染器架构中汲取了灵感，例如 [Bungie 的管线化 Destiny 渲染器](https://advances.realtimerendering.com/destiny/gdc_2015/Tatarchuk_GDC_2015__Destiny_Renderer_web.pdf)。我们还从（并与）Rust 领域的其他渲染器开发者密切合作，即 @aclysma（[rafx](https://github.com/aclysma/rafx)）和 @cwfitzgerald（[rend3](https://github.com/BVE-Reborn/rend3)）。没有他们，新 Bevy 渲染器就不会是今天这个样子，我强烈推荐查看他们的项目！

我保证下面会对所有这些华丽的流行语进行详细说明。我相信新 Bevy 渲染器将成为 Bevy 图形生态系统的（希望也是 Rust 图形生态系统整体的）一个凝聚点。我们仍然有大量工作要做，但我为我们迄今为止取得的成就感到自豪，并对未来充满期待！

![bistro 白天](https://bevy.org/news/bevy-0-6/bistro_day.png)

### 为什么要构建新渲染器？ [#](https://bevy.org/news/bevy-0-6/#why-build-a-new-renderer)

在介绍新功能之前，有必要讨论一下为什么我们开始如此大规模的工作。旧的 Bevy 渲染器在很多方面做得很好：

- **模块化渲染逻辑**（通过渲染图）
- **多后端**（包括第一方和第三方）
- **高级数据驱动 API**：这使得编写自定义的每个实体渲染逻辑变得简单且符合人体工程学

然而，它也有许多**显著的缺点**：

- **复杂**："高级易用性"的代价是实现复杂性、性能开销和自创术语的显著增加。用户在尝试在除"高级"之外的任何级别操作时，往往会感到力不从心。在管理"渲染资源"时，很容易做"错"事情，并且很难判断"哪里出了问题"。
- **通常很慢**：像"精灵渲染"这样的功能是建立在上述昂贵的高级抽象之上的。与生态系统中的其他选项相比，性能……不尽如人意。
- **面向用户的内部实现**：它直接在每个实体上存储了大量内部渲染状态。这占用了空间，计算状态的开销很大，并且用一堆"请勿触碰"的渲染组件污染了面向用户的 API。这些状态（或至少是组件元数据）需要写入/读取场景，这也是次优且容易出错的。
- **重复渲染逻辑很麻烦**：视口、渲染到多个纹理/窗口和阴影贴图是可能的，但它们需要硬编码、特殊处理和样板代码。这与我们的模块化和清晰度目标并不一致。

### 为什么是现在？ [#](https://bevy.org/news/bevy-0-6/#why-now)

上述缺点在 Bevy 的早期是可以接受的，但随着 Bevy 从一个[单人副业项目](https://bevy.org/news/introducing-bevy)成长为 GitHub 上最流行的 Rust 游戏引擎（以及[最流行的开源游戏引擎之一……毫不夸张地说](https://github.com/topics/game-engine)），这些缺点显然成为了阻碍。当我们有数百名贡献者、一名全职开发者、成千上万的个人用户以及越来越多的公司花钱请人开发 Bevy 应用和功能时，"尚可"的渲染器已不再足够。是时候做出改变了。

要深入了解我们的决策和开发过程（包括我们考虑过的替代方案），请查看[新渲染器跟踪问题](https://github.com/bevyengine/bevy/issues/2535)。

### 管线化渲染：Extract、Prepare、Queue、Render [#](https://bevy.org/news/bevy-0-6/#pipelined-rendering-extract-prepare-queue-render)

作者：@cart

管线化渲染是新渲染器的基石。它实现了多个目标：

- **提高并行度**：我们现在可以在渲染当前帧的同时，开始运行下一帧的主应用逻辑。考虑到渲染通常是瓶颈，当应用工作量也很大时，这可能是一个巨大的胜利。
- **更清晰的数据流和结构**：管线化需要在"应用逻辑"和"渲染逻辑"之间划清界限，并设置一个固定的同步点（我们称之为"extract"步骤）。这使得推理数据流和所有权变得更加容易。代码可以沿着这些界限组织，从而提高清晰度。

从高层次来看，传统的"非管线化渲染"看起来像这样：

![非管线化渲染](https://bevy.org/news/bevy-0-6/unpipelined_rendering.svg)

管线化渲染看起来像这样：

![管线化渲染](https://bevy.org/news/bevy-0-6/pipelined_rendering.svg)

好多了！

Bevy 应用现在分为主应用（Main App），这是应用逻辑发生的地方，以及渲染应用（Render App），它拥有自己独立的 ECS World 和 Schedule。渲染应用由以下 ECS 阶段（Stage）组成，开发者在组合新的渲染功能时将 ECS 系统添加到这些阶段中：

- **Extract**：这是主世界和渲染世界之间的一个同步点。相关的实体、组件和资源从主世界读取，并写入渲染世界中对应的实体、组件和资源。目标是使这一步尽可能快，因为这是唯一不能并行运行的逻辑。一个好的经验法则是只提取渲染所需的最少量数据，例如只考虑"可见"实体并且只复制相关的组件。
- **Prepare**：提取的数据然后通过写入 GPU 来"准备"。这通常涉及写入 GPU 缓冲区和纹理，以及创建绑定组。
- **Queue**：这将"排队"渲染作业，这些作业消费"准备好的"数据。
- **Render**：这运行渲染图，从 Extract、Prepare 和 Queue 步骤存储在渲染世界中的结果生成实际的渲染命令。

所以管线化渲染实际上看起来更像这样，下一次应用更新在 extract 步骤之后发生：

![管线化渲染阶段](https://bevy.org/news/bevy-0-6/pipelined_rendering_stages.svg)

快速说明一下，管线化渲染实际上尚**未**并行发生。我[有一个分支](https://github.com/cart/bevy/tree/actual-pipelining)启用了并行管线化，但在单独的线程中运行应用逻辑目前会破坏"非发送"资源（因为主应用被移动到了单独的线程，破坏了非发送保证）。这个问题很快会得到修复，我只是想尽快将新渲染器交到人们手中！当我们启用并行管线化时，不需要更改任何面向用户的代码。

### 渲染图和子图 [#](https://bevy.org/news/bevy-0-6/#render-graphs-and-sub-graphs)

作者：@cart

![渲染图](https://bevy.org/news/bevy-0-6/render_graph.svg)

新 Bevy 渲染器有一个渲染图，[很像旧的 Bevy 渲染器](https://bevy.org/news/introducing-bevy/#render-graph)。渲染图是一种以模块化方式逻辑建模 GPU 命令构建的方法。图节点将 GPU 资源（如纹理和缓冲区，有时还有实体）相互传递，形成一个有向无环图。当一个图节点运行时，它使用其图输入和渲染世界来构建 GPU 命令列表。

这个 API 最大的变化是我们现在支持子图（Sub Graph），它基本上是"命名空间化"的渲染图，可以从图中的任何节点使用任意输入运行。这使我们能够定义诸如"2d"和"3d"子图之类的东西，用户可以将其自定义逻辑插入其中。这同时开启了两扇门：

- 能够重复渲染逻辑，但用于不同的视图（分屏、镜子、渲染到纹理、阴影贴图）。
- 用户能够扩展这种重复逻辑。

### 拥抱 wgpu [#](https://bevy.org/news/bevy-0-6/#embracing-wgpu)

作者：@cart

Bevy 一直使用 [wgpu](https://github.com/gfx-rs/wgpu)，一个支持大多数图形后端（Vulkan、Metal、DX12、OpenGL、WebGL2 和 WebGPU，以及 WIP DX11 支持）的原生 GPU 抽象层。但旧的渲染器将它隐藏在我们自己的硬件抽象层后面。在实践中，这很大程度上只是 wgpu API 的一个镜像。它给了我们构建自己的图形后端而不打扰 wgpu 团队的能力，但实践也创造了大量痛苦（由于是一个不完美的镜像）、开销（由于引入了一个动态 API 并对 GPU 资源集合要求全局互斥锁）和复杂性（bevy_render -> wgpu -> Vulkan）。作为回报，我们没有得到太多实际的好处……只是稍微多一点自主权。

事实的真相是，wgpu 已经**完全**占据了我们希望它占据的空间：

- 多后端，目标是支持尽可能多的平台
- 一个"基线"功能集，几乎在任何地方都可以使用一致的 API
- 一个"限制"和"功能"系统，允许选择加入任意（有时是后端特定的）功能，并检测这些功能何时可用。这在我们开始添加光线追踪和 VR 支持时非常重要。
- 一个现代的 GPU API，但没有原始 Vulkan 的痛苦和复杂性。非常适合面向用户的 Bevy 渲染器扩展。

然而，最初有几点原因使我们没有将其作为"面向公众的 API"：

- **复杂性**：wgpu 过去是建立在 gfx-hal（同样由 wgpu 团队构建和管理的旧 GPU 抽象层）之上的。这种多层抽象分布在多个仓库中，使得对内部的理解和贡献变得困难。此外，我对"在 Bevy API 中公开暴露的第三方依赖"有一条规则：如果我们需要，我们必须能够自如地 fork 和维护它们（例如：上游停止维护、愿景分歧等）。对于旧的架构，我并不特别放心这样做。
- **许可**：wgpu 过去采用"copyleft"的 MPL 许可证，这引发了关于与专有图形 API（例如 Switch 等主机）集成的担忧。
- **WebGL2 支持**：wgpu 过去没有 WebGL2 后端。Bevy 旧的渲染器有一个自定义的 WebGL2 后端，我们不愿意放弃对 Web 作为平台的支持。

在我们提出这些担忧之后，**几乎立即**，@kvark 发起了一个[重新许可的工作](https://github.com/gfx-rs/wgpu/issues/392)，将 wgpu 切换为 Rust 标准的双 MIT/Apache-2.0 许可。他们还移除了 gfx-hal，转而采用[更简单、更扁平的架构](https://gfx-rs.github.io/2021/08/18/release-0.10.html)。不久之后，@zicklag [添加了一个 WebGL2 后端](https://github.com/gfx-rs/wgpu/pull/1686)。解决了我所有剩余的顾虑后，我清楚地看到 @kvark 的优先事项与我的是一致的，我可以相信他们会根据社区的反馈进行调整。

新 Bevy 渲染器抛弃了我们旧的中间 GPU 抽象层，转而直接使用 wgpu 作为我们的"低级"GPU API。结果是更简单（且更快）的架构，并且可以完全直接地访问 wgpu。到目前为止，来自 Bevy 渲染器功能开发者的反馈**非常积极**。

Bevy 也更新为使用最新最好的 wgpu 版本：[0.12](https://github.com/gfx-rs/wgpu/blob/master/CHANGELOG.md#wgpu-012-2021-12-18)。

### ECS 驱动的渲染 [#](https://bevy.org/news/bevy-0-6/#ecs-driven-rendering)

作者：@cart

新渲染器是我所说的"ECS 驱动"：

- 正如我们之前所介绍的，渲染世界是使用从主世界提取的数据来填充的。
- 场景从一个或多个视图渲染，这些视图只是渲染世界中带有与该视图相关的组件的实体。视图实体可以用任意组件扩展，这使得使用自定义视图数据和逻辑来扩展渲染器变得容易。相机不是唯一的视图类型。渲染应用可以为任意概念定义视图，例如"阴影贴图视角"。
- 视图可以有零个或多个通用的 `RenderPhase<T: PhaseItem>` 组件，其中 T 定义了阶段中渲染内容的"类型和范围"（例如："主通道中的透明 3d 实体"）。本质上，[`RenderPhase`](https://docs.rs/bevy/0.6.0/bevy/render/render_phase/struct.RenderPhase.html) 是一个（可能已排序）要绘制的实体列表。
- RenderPhase 中的实体使用 DrawFunctions 绘制，这些函数读取渲染世界中的 ECS 数据并生成 GPU 命令。
- DrawFunctions 可以（可选地）由模块化的 DrawCommands 组成。这些通常范围限定在特定操作，例如 `SetStandardMaterialBindGroup`、`DrawMesh`、`SetItemPipeline` 等。Bevy 提供了许多内置的 DrawCommands，用户也可以定义自己的 DrawCommands。
- 渲染图节点通过迭代每个 RenderPhase 的实体并运行适当的 Draw Functions，将特定视图的 RenderPhases 转换为 GPU 命令。

如果这看起来复杂……别担心！这些是我所说的"中层"渲染器 API。它们为经验丰富的渲染功能开发者提供了必要的工具，以相对轻松地构建模块化渲染插件。我们还提供了易于使用的高级 API，如材质，这涵盖了大多数"自定义着色器逻辑"的用例。

### Bevy 的核心管线 [#](https://bevy.org/news/bevy-0-6/#bevy-s-core-pipeline)

作者：@cart, Rob Swain (@superdump), @KirmesBude, @mockersf

新渲染器默认**非常**灵活且不固执己见。然而，**过于**灵活并不总是可取的。我们想要一个丰富的 Bevy 渲染器插件生态系统，开发者有足够的自由来实现他们想要的东西，同时仍然最大化插件之间的兼容性。

新的 [`bevy_core_pipeline`](https://docs.rs/bevy/0.6.0/bevy/core_pipeline/index.html) crate 是我们对这个问题的回答。它定义了一组"核心"视图/相机（2d 和 3d）、子图（ClearPass、MainPass2d、MainPass3d）和渲染阶段（`Transparent2d`、`Opaque3d`、`AlphaMask3d`、`Transparent3d`）。这为渲染功能开发者提供了"共同基础"，使他们可以在保持相互兼容的同时进行构建。只要开发者在这些约束范围内操作，他们就应该能够与更广泛的生态系统兼容。开发者也可以自由地在这些约束之外操作，但这也会增加不兼容的可能性。

Bevy 的内置渲染功能构建在核心管线之上（例如：[`bevy_sprite`](https://docs.rs/bevy/0.6.0/bevy/sprite/index.html) 和 [`bevy_pbr`](https://docs.rs/bevy/0.6.0/bevy/pbr/index.html)）。核心管线将继续扩展，包括标准化的"后处理"效果栈等。

### 材质 [#](https://bevy.org/news/bevy-0-6/#materials)

作者：@cart

新的渲染器结构为开发者提供了对实体绘制方式的细粒度控制。开发者可以手动定义 Extract、Prepare 和 Queue 系统，以在自定义或内置的 [`RenderPhase`](https://docs.rs/bevy/0.6.0/bevy/render/render_phase/struct.RenderPhase.html) 中使用任意渲染命令绘制实体。然而，这种级别的控制需要理解渲染管线内部原理，并且涉及比大多数用户愿意容忍更多的样板代码。有时你只想将自定义材质着色器插入到现有的管线中！

新的 [`Material`](https://docs.rs/bevy/0.6.0/bevy/pbr/trait.Material.html) trait 使用户能够忽略繁琐的细节，转而使用更简单的接口：只需实现 [`Material`](https://docs.rs/bevy/0.6.0/bevy/pbr/trait.Material.html) trait 并为你的类型添加一个 [`MaterialPlugin`](https://docs.rs/bevy/0.6.0/bevy/pbr/struct.MaterialPlugin.html)。新的 [shader_material.rs](https://github.com/bevyengine/bevy/blob/v0.6.0/examples/shader/shader_material.rs) 示例说明了这一点。

```rust
// 为 CustomMaterial 注册插件
app.add_plugin(MaterialPlugin::<CustomMaterial>::default())

impl Material for CustomMaterial {
    // 顶点和片段着色器是可选的
    // 如果未定义，它们使用默认的"网格管线着色器"
    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/custom_material.wgsl"))
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        /* 在此返回绑定组布局 */
    }

    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        /* 在此返回绑定组 */
    }
}
```

还有一个 [`SpecializedMaterial`](https://docs.rs/bevy/0.6.0/bevy/pbr/trait.SpecializedMaterial.html) 变体，它支持使用自定义的每个实体键来"特化"着色器和管线。这种额外的灵活性并不总是需要的，但当您需要时，您会很高兴拥有它！例如，内置的 StandardMaterial 使用特化来切换实体是否应在着色器中接收光照。

我们还有更大的计划来让 [`Material`](https://docs.rs/bevy/0.6.0/bevy/pbr/trait.Material.html) 变得更好：

- **绑定组派生**：这应该能减少将材质传递到 GPU 的样板代码。
- **材质实例化**：材质使我们能够将高级网格实例化实现为内置和自定义材质的一个简单配置项。

### 可见性和视锥体裁剪 [#](https://bevy.org/news/bevy-0-6/#visibility-and-frustum-culling)

作者：Rob Swain (@superdump)

[![视锥体](https://bevy.org/news/bevy-0-6/ViewFrustum.svg)](https://en.wikipedia.org/wiki/Viewing_frustum#/media/File:ViewFrustum.svg)

绘制东西很昂贵！它需要将数据从 CPU 写入 GPU，构建绘制调用，并运行着色器。通过**不**绘制相机看不到的东西，我们可以节省大量时间。"视锥体裁剪"是排除相机"视锥体"边界之外的对象的行为，以避免浪费工作去绘制它们。对于大型场景，这可能是流畅的 60 帧每秒和卡顿到完全停止之间的区别。

**Bevy 0.6** 现在自动对 3d 对象使用它们的轴对齐包围盒进行视锥体裁剪。我们可能也会在未来的版本中为 2d 对象启用此功能，但那里的收益将不那么明显，因为由于新的批处理渲染，绘制精灵现在便宜得多。

### 方向阴影 [#](https://bevy.org/news/bevy-0-6/#directional-shadows)

作者：Rob Swain (@superdump)

方向光现在可以投射"方向阴影"，这是一种从无限远的光源投射的"太阳般"的阴影。可以通过将 `DirectionalLight::shadows_enabled` 设置为 `true` 来启用。

![方向光](https://bevy.org/news/bevy-0-6/directional_light.png)

注意：方向阴影目前需要比必要更多的手动配置（查看 shadow_biases.rs 示例中 [`DirectionalLight` 设置中的 `shadow_projection` 字段](https://github.com/bevyengine/bevy/blob/main/examples/3d/shadow_biases.rs)）。我们很快将通过级联阴影贴图使其自动化，并在更大的范围内提供更好的质量。

### 点光源阴影 [#](https://bevy.org/news/bevy-0-6/#point-light-shadows)

作者：@mtsr, Rob Swain (@superdump), @cart

点光源现在可以投射"全向阴影"，可以通过将 `PointLight::shadows_enabled` 设置为 `true` 来启用：

![点光源](https://bevy.org/news/bevy-0-6/point_light.png)

### 启用和禁用实体阴影 [#](https://bevy.org/news/bevy-0-6/#enabling-and-disabling-entity-shadows)

作者：Rob Swain (@superdump)

网格实体可以通过添加 [`NotShadowCaster`](https://docs.rs/bevy/0.6.0/bevy/pbr/struct.NotShadowCaster.html) 组件来选择不投射阴影。

```rust
commands.entity(entity).insert(NotShadowCaster);
```

同样，它们可以通过添加 [`NotShadowReceiver`](https://docs.rs/bevy/0.6.0/bevy/pbr/struct.NotShadowReceiver.html) 组件来选择不接收阴影。

```rust
commands.entity(entity).insert(NotShadowReceiver);
```

### 球形区域光 [#](https://bevy.org/news/bevy-0-6/#spherical-area-lights)

作者：@Josh015

`PointLight` 组件现在可以定义一个 `radius` 值，它控制发射光的球体的大小。一个正常的零大小的"点光源"的半径为零。

![球形区域光](https://bevy.org/news/bevy-0-6/spherical_area_lights.png)

（注意：带有半径的光通常不会在世界中占据物理空间……我添加了网格以帮助说明光的位置和大小）

### 可配置的 Alpha 混合模式 [#](https://bevy.org/news/bevy-0-6/#configurable-alpha-blend-modes)

作者：Rob Swain (@superdump)

Bevy 的 StandardMaterial 现在有一个 `alpha_mode` 字段，可以设置为 `AlphaMode::Opaque`、`AlphaMode::Mask(f32)` 或 `AlphaMode::Blend`。加载 GLTF 场景时会正确设置此字段。

![alpha 混合模式](https://bevy.org/news/bevy-0-6/alpha_blend.png)

### 分簇前向渲染 [#](https://bevy.org/news/bevy-0-6/#clustered-forward-rendering)

作者：Rob Swain (@superdump)

现代场景通常有许多点光源。但在渲染场景时，随着场景中灯光数量的增加，为每个渲染片段计算每个光照的照明会迅速变得过于昂贵。[分簇前向渲染](http://www.aortiz.me/2018/12/21/CG.html)是一种流行的方法，它通过将视锥体划分为"簇"（子体积的三维网格）来增加场景中可以拥有的灯光数量。每个簇根据灯光是否会影响该簇来分配灯光。这是一种"裁剪"形式，使片段能够忽略未分配给其簇的灯光。

在实践中，这可以显著增加场景中的灯光数量：

![分簇前向渲染](https://bevy.org/news/bevy-0-6/clustered_forward_rendering.png)

簇是视锥体的三维细分。它们在投影空间中是立方体，因此对于透视投影，它们在视图空间中被拉伸和倾斜。在屏幕空间中调试时，您正沿着一行簇查看，因此它们看起来像正方形。正方形内的不同颜色表示网格表面在场景中处于不同深度，因此它们属于不同的簇：

![簇](https://bevy.org/news/bevy-0-6/clusters.png)

当前的实现限制为最多 256 个灯光，因为我们最初优先考虑跨平台兼容性，以便每个人都能受益。WebGL2 特别不支持存储缓冲区，因此当前实现受到最大统一缓冲区大小的限制。通过使用存储缓冲区，我们可以在其他平台上支持更多的灯光，我们将在未来的版本中添加对此的支持。

[点击此处](https://youtu.be/dElYzzNovEk)观看说明 Bevy 分簇前向渲染的视频。

### 精灵批处理 [#](https://bevy.org/news/bevy-0-6/#sprite-batching)

作者：@cart

精灵现在根据其在 z 层级内的纹理进行批量渲染。它们还会跨 z 层级进行机会性批处理。这带来了显著的性能提升，因为它大大减少了所需的绘制调用数量。结合新 Bevy 渲染器的其他性能改进，事情开始变得非常有趣！在我的机器上，旧的 Bevy 渲染器在我们的"bevymark"基准测试中通常在大约 8,000 个精灵时开始低于 60fps。使用新渲染器在同样的机器上，我可以获得大约 100,000 个精灵！

![bevymark](https://bevy.org/news/bevy-0-6/bevymark.png)

我的机器：Nvidia GTX 1070, Intel i7 7700k, 16GB 内存, Arch Linux

### 精灵人体工程学 [#](https://bevy.org/news/bevy-0-6/#sprite-ergonomics)

精灵实体现在更简单了：

```rust
fn spawn_sprite(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("player.png"),
        ..Default::default()
    });
}
```

不再需要管理精灵材质！它们的纹理句柄现在是一个直接的组件，颜色现在可以直接在 [`Sprite`](https://docs.rs/bevy/0.6.0/bevy/sprite/struct.Sprite.html) 组件上设置。

作为对比，展开查看旧的 Bevy 0.5 代码：

```rust
// 旧的（Bevy 0.5）
fn spawn_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let texture_handle = asset_server.load("player.png");
    commands.spawn_bundle(SpriteBundle {
        material: materials.add(texture_handle.into()),
        ..Default::default()
    });
}
```

### WGSL 着色器 [#](https://bevy.org/news/bevy-0-6/#wgsl-shaders)

Bevy 现在使用 [WGSL](https://www.w3.org/TR/WGSL/) 作为我们的内置着色器和示例。WGSL 是一种正在为 WebGPU 开发的新着色器语言（尽管它和 GLSL 一样是一种"跨平台"着色器语言）。Bevy 仍然支持 GLSL 着色器，但 WGSL 足够好，目前我们将其视为"官方推荐的"着色器语言。WGSL 仍在开发和打磨中，但考虑到它获得的巨大投入，我相信它值得押注。请将此视为"官方 Bevy 着色器语言"讨论的开始，而非结束。

```rust
[[group(0), binding(0)]]
var<uniform> view: View;

[[group(1), binding(0)]]
var<uniform> mesh: Mesh;

struct Vertex {
    [[location(0)]] position: vec3<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = view.view_proj * mesh.model * vec4<f32>(vertex.position, 1.0);
    return out;
}
```

### 着色器预处理器 [#](https://bevy.org/news/bevy-0-6/#shader-preprocessor)

作者：@cart, Rob Swain (@superdump), @mockersf

Bevy 现在拥有自己的自定义着色器预处理器。它目前支持 `#import`、`#ifdef FOO`、`#ifndef FOO`、`#else` 和 `#endif`，但我们将在未来扩展更多功能，以实现简单、灵活的着色器代码重用和扩展。

着色器预处理器通常用于有条件地启用着色器代码：

```rust
#ifdef TEXTURE
[[group(1), binding(0)]]
var sprite_texture: texture_2d<f32>;
#endif
```

这种模式在定义复杂/可配置的着色器（例如 Bevy 的 PBR 着色器）时非常有用。

### 着色器导入 [#](https://bevy.org/news/bevy-0-6/#shader-imports)

作者：@cart

新的预处理器支持导入其他着色器文件（这会拉入它们的全部内容）。有两种形式：

资源路径导入：

```rust
#import "shaders/cool_function.wgsl"

[[stage(fragment)]]
fn fragment(input: VertexOutput) -> [[location(0)]] vec4<f32> {
    return cool_function();
}
```

插件提供的导入，可以由 Bevy 插件使用任意路径注册：

```rust
#import bevy_pbr::mesh_view_bind_group

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    let world_position = vec4<f32>(vertex.position, 1.0);
    var out: VertexOutput;
    // `view` 变量来自导入的绑定组
    out.clip_position = view.view_proj * world_position;
    return out;
}
```

我们还计划尝试使用 Naga 来实现特定命名符号的"部分导入"（例如：从文件导入特定函数或结构体）。这是一个"遥远"的想法，但这也可以使用 Naga 的中间着色器表示作为将用不同语言编写的着色器代码片段组合成一个着色器的方法。

### 管线特化 [#](https://bevy.org/news/bevy-0-6/#pipeline-specialization)

作者：@cart

当着色器使用预处理器并具有多个排列时，相关的"渲染管线"需要更新以适应该排列（例如：不同的顶点属性、绑定组等）。为了使这个过程简单直接，我们添加了 SpecializedPipeline trait，它允许为给定的键定义特化：

```rust
impl SpecializedPipeline for MyPipeline {
    type Key = MyPipelineKey;
    // 这里的键在将实体排入 RenderPhase 时指定
    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // 在此为给定的键定义特化管线
    }
}
```

这个 trait 的实现者然后可以轻松且廉价地访问特化管线的变体（具有自动的每个键缓存和热重载）。如果这感觉太抽象/高级，别担心！这是一个"中层高级用户工具"，不是大多数 Bevy 应用开发者需要处理的东西。

### 更简单的着色器栈 [#](https://bevy.org/news/bevy-0-6/#simpler-shader-stack)

Bevy 现在使用 [Naga](https://github.com/gfx-rs/naga) 满足所有着色器需求。因此，我们能够移除所有复杂的非 Rust 着色器依赖项：`glsl_to_spirv`、`shaderc` 和 `spirv_reflect`。`glsl_to_spirv` 是平台特定构建依赖项和错误的主要来源，所以这是一个巨大的胜利！

### 移植到新渲染器的功能 [#](https://bevy.org/news/bevy-0-6/#features-ported-to-the-new-renderer)

内部 Bevy crate 的渲染逻辑在许多情况下不得不重写以利用新渲染器。以下人员帮助了这项工作：

- bevy_sprites：@cart, @StarArawn, @Davier
- bevy_pbr：Rob Swain (@superdump), @aevyrie, @cart, @zicklag, @jakobhellermann
- bevy_ui：@Davier
- bevy_text：@Davier
- bevy_gltf：Rob Swain (@superdump)

### WebGL2 支持 [#](https://bevy.org/news/bevy-0-6/#webgl2-support)

作者：@zicklag, @mrk-its, @mockersf, Rob Swain (@superdump)

Bevy 现在内置支持使用 WebGL2 / WASM 部署到 Web，这要感谢 @zicklag 向 wgpu 添加的原生 WebGL2 后端。现在不再需要第三方的 `bevy_webgl2` 插件。任何 Bevy 应用都可以通过运行以下命令部署到 Web：

```rust
cargo build --target wasm32-unknown-unknown
wasm-bindgen --out-dir OUTPUT_DIR --target web TARGET_DIR
```

新 Bevy 渲染器的开发者在初始渲染功能实现中优先考虑了跨平台兼容性，因此必须仔细在 WebGL2 的限制内操作（例如：WebGL2 不支持存储缓冲区和计算着色器），但结果是值得的！随着时间的推移，将实现利用更现代/高级功能（如计算着色器）的特性。但对我们来说，让每个人都能为他们的游戏和应用获得扎实的视觉体验非常重要，无论他们目标是什么平台。

你可以使用我们的新 [Bevy 示例](https://bevy.org/examples) 页面在你的浏览器中尝试 Bevy 的 WASM 支持：

[![wasm bevy 示例](https://bevy.org/news/bevy-0-6/bevy_examples_wasm.png)](https://bevy.org/examples)

### 无限反向 Z 透视投影 [#](https://bevy.org/news/bevy-0-6/#infinite-reverse-z-perspective-projection)

作者：Rob Swain (@superdump)

为了提高"有用范围"内的精度，业界已广泛采用带有"无限"远平面的"反向投影"。新的 Bevy 渲染器已适应使用"右手系无限反向 z"投影。[这篇 Nvidia 文章](https://developer.nvidia.com/content/depth-precision-visualized)很好地解释了为什么这如此有价值。

### 计算着色器 [#](https://bevy.org/news/bevy-0-6/#compute-shaders)

新渲染器使用户能够编写计算着色器。我们新的["计算着色器康威生命游戏"示例](https://github.com/bevyengine/bevy/blob/v0.6.0/examples/shader/compute_shader_game_of_life.rs)（由 @jakobhellermann 编写）说明了如何在 Bevy 中编写计算着色器。

![计算生命游戏](https://bevy.org/news/bevy-0-6/compute.png)

### 新的多窗口示例 [#](https://bevy.org/news/bevy-0-6/#new-multiple-windows-example)

作者：@DJMcNab

"多窗口"示例已更新为使用新的渲染器 API。得益于新的渲染器 API，这个示例现在[看起来好多了](https://github.com/bevyengine/bevy/blob/v0.6.0/examples/window/multiple_windows.rs)（当我们添加高级渲染目标时，它看起来会更好）。

![多窗口](https://bevy.org/news/bevy-0-6/multiple_windows.png)

### Crevice [#](https://bevy.org/news/bevy-0-6/#crevice)

作者：@cart, @mockersf, Rob Swain (@superdump)

Bevy 旧的 `Bytes` 抽象已被 [crevice](https://github.com/LPGhatguy/crevice) crate（由 @LPGhatguy 编写）的一个 fork 所取代，这使得可以将普通的 Rust 类型写入 GPU 友好的数据布局中。即 std140（统一缓冲区默认使用此布局）和 std430（存储缓冲区默认使用此布局）。Bevy 导出了 `AsStd140` 和 `AsStd430` 派生宏：

```rust
#[derive(AsStd140)]
pub struct MeshUniform {
    pub transform: Mat4,
    pub inverse_transpose_model: Mat4,
}
```

将 `AsStd140` 派生宏与我们新的 `UniformVec<T>` 类型结合使用，可以轻松地将 Rust 类型写入着色器就绪的统一缓冲区：

```rust
// WGSL 着色器
struct Mesh {
    model: mat4x4<f32>;
    inverse_transpose_model: mat4x4<f32>;
};

[[group(2), binding(0)]]
var<uniform> mesh: Mesh;
```

我们（短期内）fork crevice 有几个原因：

- 为了合并 @ElectronicRU 的[数组支持 PR](https://github.com/LPGhatguy/crevice/pull/27/)，因为我们需要在统一缓冲区中支持数组。
- 为了重新导出 crevice 派生宏，并为 Bevy 提供"开箱即用"的体验。

最终，我们希望如果可能的话回到上游。非常感谢 crevice 开发者构建了如此有用的软件！

### UV 球体网格形状 [#](https://bevy.org/news/bevy-0-6/#uv-sphere-mesh-shape)

作者：@nside

Bevy 现在有一个内置的"uv 球体"网格基元。

```rust
Mesh::from(UVSphere {
    radius: 1.0,
    sectors: 16,
    stacks: 32,
})
```

![uv 球体](https://bevy.org/news/bevy-0-6/uv_sphere.png)

### 平面法线计算 [#](https://bevy.org/news/bevy-0-6/#flat-normal-computation)

作者：@jakobhellermann

`Mesh` 类型现在有一个 `compute_flat_normals()` 函数。导入的没有法线的 GLTF 网格现在会自动计算平面法线，[符合 GLTF 规范](https://www.khronos.org/registry/glTF/specs/2.0/glTF-2.0.html#meshes)。

![平面法线](https://bevy.org/news/bevy-0-6/flat_normals.png)

### 更快的 GLTF 加载 [#](https://bevy.org/news/bevy-0-6/#faster-gltf-loading)

作者：@DJMcNab, @mockersf

@DJMcNab 修复了 GLTF 节点的严重非线性加载问题，使它们加载得更快。一个复杂的场景从 40 秒减少到 0.2 秒。太棒了！

@mockersf 使 GLTF 纹理在 Bevy 的"IO 任务池"中异步加载，在某些情况下几乎使 GLTF 场景加载时间减半。

我们还正在添加"压缩纹理加载"，这将大大提高 GLTF 场景加载速度，特别是对于大型场景！

## Bevy ECS [#](https://bevy.org/news/bevy-0-6/#bevy-ecs)

### 不再需要 `.system()`！ [#](https://bevy.org/news/bevy-0-6/#no-more-system)

作者：@DJMcNab, @Ratysz

Bevy ECS 的最高优先事项之一是"人体工程学"。过去我曾大胆宣称 Bevy ECS 是[现存最符合人体工程学的 ECS](https://bevy.org/news/introducing-bevy/#ergonomics)。我们在研发上投入了大量精力，开创了新的 API 技术，我相信结果不言自明：

```rust
// 这是一个独立的 Bevy 0.5 应用，将一个简单的 `gravity` 系统添加到应用的 schedule 中
// 并自动与其他系统并行运行
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_system(gravity.system())
        .run();
}

fn gravity(time: Res<Time>, mut query: Query<&mut Transform>) {
    for mut transform in query.iter_mut() {
        transform.translation.y += -9.8 * time.delta_seconds();
    }
}
```

我相信我们已经是市场上最好的（特别是考虑到我们的自动并行化和变更检测），但有一个东西阻碍了我们达到完美……那个讨厌的 `.system()`！我们曾多次尝试移除它，但由于 rustc 的限制和安全问题，它总是躲过我们。最终，@DJMcNab [找到了一个解决方案](https://github.com/bevyengine/bevy/pull/2398)。因此，在 Bevy 0.6 中，你现在可以这样注册上面的系统：

```rust
// 纯粹的幸福！
App::new()
    .add_plugins(DefaultPlugins)
    .add_system(gravity)
    .run();
```

### 新的 Component Trait 和 #[derive(Component)] [#](https://bevy.org/news/bevy-0-6/#the-new-component-trait-and-derive-component)

作者：@Frizi

在 **Bevy 0.6** 中，类型不再默认实现 [`Component`](https://docs.rs/bevy/0.6.0/bevy/ecs/component/trait.Component.html) trait。在你生气之前……请稍等片刻。我保证这是为了最好！在过去的 Bevy 版本中，我们通过这个"全局 impl"为类型"自动实现"了 [`Component`](https://docs.rs/bevy/0.6.0/bevy/ecs/component/trait.Component.html)：

```rust
impl<T: Send + Sync + 'static> Component for T {}
```

这消除了用户手动为其类型实现 [`Component`](https://docs.rs/bevy/0.6.0/bevy/ecs/component/trait.Component.html) 的需要。早期这似乎是一个只有好处没有坏处的人体工程学胜利。但自那以后，Bevy ECS、我们对问题空间的理解以及对未来的计划已经发生了很大变化：

- **原来并非所有东西**都应该是组件**：我们的用户**经常**意外地将非组件类型添加为组件。新用户意外地将 Bundle 和类型构造函数添加为组件是我们 [Discord](https://discord.gg/bevy) 上 `#help` 频道最常见的讨论主题。这类错误非常难以调试，因为事情只是悄悄地"不工作"。当不是所有的东西都是组件时，rustc 可以在你搞砸时用信息丰富的错误消息好好地提醒你。
- **优化**：如果我们为所有东西自动实现 Component，我们就无法用关联类型自定义 [`Component`](https://docs.rs/bevy/0.6.0/bevy/ecs/component/trait.Component.html) trait。这阻止了一整类优化。例如，Bevy ECS 现在有[多种组件存储类型](https://bevy.org/news/bevy-0-5/#hybrid-component-storage-the-solution)。通过将存储类型移入 [`Component`](https://docs.rs/bevy/0.6.0/bevy/ecs/component/trait.Component.html)，我们使 rustc 能够优化通常需要在运行时进行的检查。@Frizi 通过将存储类型移入 [`Component`](https://docs.rs/bevy/0.6.0/bevy/ecs/component/trait.Component.html) 能够[显著提高我们的查询迭代器性能](https://github.com/bevyengine/bevy/pull/2254#issuecomment-857863116)。我预计我们会在这方面找到更多优化空间。
- **自动注册**：将更多逻辑移入 [`Component`](https://docs.rs/bevy/0.6.0/bevy/ecs/component/trait.Component.html) 也使我们能够在未来做更花哨的事情，比如"在派生 [`Component`](https://docs.rs/bevy/0.6.0/bevy/ecs/component/trait.Component.html) 时自动注册 Reflect 实现"。非全局的 [`Component`](https://docs.rs/bevy/0.6.0/bevy/ecs/component/trait.Component.html) 实现确实增加了一点点样板代码，但它们也有可能大幅减少应用的"总样板代码"。
- **文档**：派生 [`Component`](https://docs.rs/bevy/0.6.0/bevy/ecs/component/trait.Component.html) 作为一种自我文档化的形式。现在一眼就能看出哪些类型是组件。
- **组织性**：在 Bevy 0.5 中，[`Component`](https://docs.rs/bevy/0.6.0/bevy/ecs/component/trait.Component.html) 特定的配置（如"存储类型"）必须在某处集中式插件中注册。将组件配置移入 [`Component`](https://docs.rs/bevy/0.6.0/bevy/ecs/component/trait.Component.html) trait 允许用户将"组件类型信息"放在类型本身旁边。
- **事件处理**：非全局的 [`Component`](https://docs.rs/bevy/0.6.0/bevy/ecs/component/trait.Component.html) 实现最终将允许我们向 [`Component`](https://docs.rs/bevy/0.6.0/bevy/ecs/component/trait.Component.html) trait 添加诸如 `on_insert(world: &mut World)` 之类的事件处理程序。非常有用！

希望到现在为止你已经相信这是正确的举措。如果没有……我很抱歉……你仍然需要在 Bevy 0.6 中手动实现 [`Component`](https://docs.rs/bevy/0.6.0/bevy/ecs/component/trait.Component.html)。你可以选择派生 [`Component`](https://docs.rs/bevy/0.6.0/bevy/ecs/component/trait.Component.html)：

```rust
// 默认为"Table"存储
#[derive(Component)]
struct SomeComponent;

// 覆盖默认存储
#[derive(Component)]
#[component(storage = "SparseSet")]
struct SomeComponent;
```

或者你可以手动实现它：

```rust
struct SomeComponent;

impl Component for SomeComponent {
    type Storage = TableStorage;
}

impl Component for SomeComponent {
    type Storage = SparseSetStorage;
}
```

### 可变查询的 iter() [#](https://bevy.org/news/bevy-0-6/#iter-for-mutable-queries)

作者：@Guvante

可变查询现在可以不可变地迭代，返回组件的不可变引用：

```rust
fn system(mut players: Query<&mut Player>) {
    for player in players.iter() {
        // player 是一个不可变引用
    }

    for mut player in players.iter_mut() {
        // player 是一个可变引用
    }
}
```

与在以前的 Bevy 版本中需要复杂 QuerySet 来避免冲突的不可变和可变查询相比：

```rust
// 真难看！
fn system(mut players: QuerySet<(QueryState<&Player>, QueryState<&mut Player>)>) {
    for player in players.q0().iter() {
        // player 是一个不可变引用
    }

    for mut player in players.q1().iter_mut() {
        // player 是一个可变引用
    }
}
```

### SystemState [#](https://bevy.org/news/bevy-0-6/#systemstate)

你是否曾经想直接对 Bevy World 使用"系统参数"？有了 [`SystemState`](https://docs.rs/bevy/0.6.0/bevy/ecs/system/struct.SystemState.html)，现在你可以了！

```rust
let mut system_state: SystemState<(Res<A>, Query<&B>)> = SystemState::new(&mut world);
let (a, query) = system_state.get(&world);
```

对于那些直接使用 `World` 的人来说，这是一个游戏规则改变者。它使得可以可变地访问多个不相交的组件和资源（通常消除了对更昂贵的抽象（如 `WorldCell`）的需求）。

[`SystemState`](https://docs.rs/bevy/0.6.0/bevy/ecs/system/struct.SystemState.html) 会进行与普通 Bevy 系统相同的缓存，因此重用同一个 [`SystemState`](https://docs.rs/bevy/0.6.0/bevy/ecs/system/struct.SystemState.html) 可以实现极快的 World 访问。

### 子应用 [#](https://bevy.org/news/bevy-0-6/#sub-apps)

作者：@cart, @zicklag, @bjorn3

新的 Bevy 渲染器需要"主应用"和"渲染应用"之间的严格分离。为了实现这一点，我们添加了"子应用"的概念：

```rust
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, AppLabel)]
pub struct RenderApp;

let mut render_app = App::empty();
app.add_sub_app(RenderApp, render_app, move |app_world, render_app| {
    // 在此执行应用逻辑
});

// 之后
app.sub_app_mut(RenderApp)
    .add_system(some_system);
    .add_system(some_other_system);
```

我们计划在未来公开更多关于调度、运行和操作子应用的控制。

### Query::iter_combinations [#](https://bevy.org/news/bevy-0-6/#query-iter-combinations)

作者：@Frizi

你现在可以为给定的查询迭代所有 N 个实体的组合：

```rust
fn system(query: Query<&Player>) {
    // 精确迭代两个实体的每种可能组合一次
    for [p1, p2] in query.iter_combinations() {
    }

    // 精确迭代三个实体的每种可能组合一次
    for [p1, p2, p3] in query.iter_combinations() {
    }
}
```

这对于诸如"检查实体与所有其他实体的碰撞"之类的事情特别有用。还有一个 `iter_combinations_mut` 变体。不过要小心……随着组合中实体数量的增加，其时间复杂度呈指数增长。能力越大，责任越大！

新的 [iter_combinations 示例](https://github.com/bevyengine/bevy/blob/v0.6.0/examples/ecs/iter_combinations.rs) 说明了如何使用这个新 API 计算"太阳系"中物体之间的引力：

![iter_combinations](https://bevy.org/news/bevy-0-6/iter_combinations.png)

### 优化的系统命令 [#](https://bevy.org/news/bevy-0-6/#optimized-system-commands)

作者：@NathanSWard

系统命令通过更改命令缓冲区的存储和重用方式，获得了不错的性能提升：

#### 实体生成基准测试时长（以微秒为单位，越少越好） [#](https://bevy.org/news/bevy-0-6/#entity-spawn-benchmark-duration-in-microseconds-less-is-better)

这个基准测试生成了具有各种组件组成的实体，以确保我们覆盖了各种情况。请将这些数字视为相对的，而非绝对的。

![命令性能](https://bevy.org/news/bevy-0-6/commands_perf.svg)

### 系统参数生命周期 [#](https://bevy.org/news/bevy-0-6/#system-param-lifetimes)

作者：@cart, @BoxyUwU, @TheRawMeatball

系统和查询的生命周期通过拆分出 `'system` 和 `'world` 生命周期，并在可能的情况下显式使用它们，变得更加明确。这使得 Rust 能够更有效地推理 ECS 生命周期，特别是对于只读生命周期。这一点尤其重要，因为它使新的 Bevy 渲染器能够说服 wgpu 说 ECS 资源实际上与渲染世界一样长。

请注意，这确实使 SystemParam 派生上的生命周期因此变得稍微复杂一些：

```rs
#[derive(SystemParam)]
struct CustomParam<'w, 's> {
    res: Res<'w, AssetServer>,
    query: Query<'w, 's, Read<Transform>>,
    local: Local<'s, i32>,
}
```

### 健全性/正确性改进 [#](https://bevy.org/news/bevy-0-6/#soundness-correctness-improvements)

作者：@BoxyUwU, @TheRawMeatball, @Frizi, @thebluefish, @sapir, @bjorn3, @DJMcNab

本次发布中，Bevy ECS 收到了大量健全性和正确性的错误修复，以及一些 unsafe 代码块的移除。查询和内部存储（如 Tables 和 BlobVecs）在这些方面尤其进行了许多修复和改进。随着 Bevy ECS 的成熟，我们对 unsafe 代码块和健全性的标准也必须提高。Bevy ECS 可能永远不会 100% 没有 unsafe 代码块，因为我们在建模 Rust 在没有我们帮助的情况下根本无法推理的并行数据访问。但我们致力于尽可能多地移除 unsafe 代码（我们正在进行一些重构以进一步改善这种情况）。

### 层级便利函数 [#](https://bevy.org/news/bevy-0-6/#hierarchy-convenience-functions)

作者：@TheRawMeatball, @jihiggins

```rust
// 销毁一个实体的所有后代（它的子节点，它子节点的子节点，等等）
commands.entity(e).despawn_descendants();

// 从实体中移除给定的子节点
commands.entity(parent).remove_children(&[child1, child2]);
```

## UI [#](https://bevy.org/news/bevy-0-6/#ui)

### Overflow::Hidden [#](https://bevy.org/news/bevy-0-6/#overflow-hidden)

作者：@Davier

UI 现在遵循 flexbox 的 `Overflow::Hidden` 属性。这可以用来裁剪子内容，在构建可滚动列表之类的东西时很有用：

![overflow hidden](https://bevy.org/news/bevy-0-6/overflow.png)

### Text2D 变换 [#](https://bevy.org/news/bevy-0-6/#text2d-transforms)

作者：@nside, @CleanCut

`Text2d` 现在支持使用 Transform 组件进行任意变换：

![text 2d 变换](https://bevy.org/news/bevy-0-6/text2d_transforms.png)

请注意，虽然 `Transform::scale` 确实有其用途，但通常仍然建议使用"字体大小"来调整文本大小，以确保其渲染"清晰"。

### 窗口透明度 [#](https://bevy.org/news/bevy-0-6/#window-transparency)

作者：@louisgjohnson

Winit 的"窗口透明"功能现在在 Bevy 的 Window 类型中暴露。这使得用户可以构建没有背景或窗口装饰的"类似小部件"的应用（在支持它的平台上）。这是一个在透明背景上运行的 Bevy 应用，在我的 Linux 桌面背景上渲染了一个 Bevy Logo 精灵。无缝！很酷！

![透明窗口](https://bevy.org/news/bevy-0-6/transparent_window.png)

## 变换 [#](https://bevy.org/news/bevy-0-6/#transforms)

### 友好的方向向量 [#](https://bevy.org/news/bevy-0-6/#friendly-directional-vectors)

作者：@guimcaballero

Bevy 变换现在有友好的"方向"函数，返回相对向量：

```rust
// 指向变换的左边
let left: Vec4 = transform.left();
// 指向变换的右边
let right: Vec4 = transform.right();
// 指向变换的上方
let up: Vec4 = transform.up();
// 指向变换的下方
let down: Vec4 = transform.down();
// 指向变换的前方
let forward: Vec4 = transform.forward();
// 指向变换的后方
let back: Vec4 = transform.back();
```

### 变换构建器方法 [#](https://bevy.org/news/bevy-0-6/#transform-builder-methods)

作者：@Lythenas

变换现在有帮助性的 `with_translation()`、`with_rotation()` 和 `with_scale()` 构建器方法：

```rust
Transform::from_xyz(0.0, 0.0, 10.0).with_scale(Vec3::splat(2.0))
```

## Rust 2021 [#](https://bevy.org/news/bevy-0-6/#rust-2021)

作者：@mockersf, @YohDeadfall

Bevy 已更新为使用 Rust 2021。这意味着我们可以默认利用新的 Cargo 功能解析器（Bevy 和新 wgpu 版本都需要这个）。确保将你的 crate 更新到 Rust 2021，否则你需要在你的 Cargo.toml 中使用 `resolver = "2"` 手动启用新的功能解析器。

```toml
[package]
name = "your_app"
version = "0.1.0"
edition = "2021"
```

请注意，"虚拟 Cargo 工作区"仍然需要手动定义 `resolver = "2"`，即使在 Rust 2021 中也是如此。详情请[参考 Rust 2021 文档](https://doc.rust-lang.org/edition-guide/rust-2021/default-cargo-resolver.html#details)。

```toml
[workspace]
resolver = "2" # 重要！wgpu/Bevy 需要这个！
members = [ "my_crate1", "my_crate2" ]
```

## 输入 [#](https://bevy.org/news/bevy-0-6/#input)

### Gamepads 资源 [#](https://bevy.org/news/bevy-0-6/#gamepads-resource)

作者：@CrazyRoka

**Bevy 0.6** 添加了一个 `Gamepads` 资源，它会自动维护一个已连接手柄的集合。

```rust
fn system(gamepads: Res<Gamepads>) {
    // 迭代每个活跃的手柄
    for gamepad in gamepads.iter() {
    }
}
```

### 输入"任意"变体 [#](https://bevy.org/news/bevy-0-6/#input-any-variants)

作者：@DJMcNab

`Input` 集合现在有一个 `any_pressed()` 函数，当给定的任何输入被按下时返回 true。

```rust
 fn system(input: Res<Input<KeyCode>>) {
    if input.any_pressed([KeyCode::LShift, KeyCode::RShift]) {
        println!("两个 shift 键中的一个或两个都被按下");
    }
 }
```

## 性能分析 [#](https://bevy.org/news/bevy-0-6/#profiling)

### 更多跨度 [#](https://bevy.org/news/bevy-0-6/#more-spans)

作者：@cart, @mockersf, @hymm

新的渲染器现在具有帧、渲染应用 schedule 和渲染图（带有命名的子图跨度）的 tracing 跨度。系统执行器现在具有更细粒度的跨度，填补了大部分剩余空白。应用系统命令现在也有跨度。

（忽略跨度中的那些奇怪字符……我们正在[调查](https://github.com/bevyengine/bevy/issues/3563)这个问题）

![渲染应用性能分析](https://bevy.org/news/bevy-0-6/render_app_profiling.png)

### Tracy 后端 [#](https://bevy.org/news/bevy-0-6/#tracy-backend)

作者：Rob Swain (@superdump)

Bevy 现在通过 `trace_tracy` Cargo feature 支持 [tracy](https://github.com/wolfpld/tracy) 性能分析器。

![tracy](https://bevy.org/news/bevy-0-6/tracy.png)

### FromReflect Trait 和派生宏 [#](https://bevy.org/news/bevy-0-6/#fromreflect-trait-and-derive)

作者：@Davier

类型现在可以派生新的 `FromReflect` trait，这使得可以使用任意的 `Reflect` impl 来创建类型的"克隆"。这目前用于使反射集合类型（如 `Vec`）正常工作，但它对于往返转换为 `Reflect` 类型和从 `Reflect` 类型转换回来也很有用。

```rust
#[derive(Reflect, FromReflect)]
struct Foo {
    bar: usize,
}

let foo = Foo::from_reflect(&dyn some_reflected_value).unwrap(); 
```

## Bevy 错误码 [#](https://bevy.org/news/bevy-0-6/#bevy-error-codes)

作者：@mockersf, @NiklasEi

为了更容易搜索和讨论常见的 Bevy 错误，我们决定添加一个正式的[错误码系统](https://github.com/bevyengine/bevy/tree/main/errors)，很像 [rustc 使用的那套](https://github.com/rust-lang/rust/tree/master/compiler/rustc_error_codes/src/error_codes)。

错误码及其描述在 Bevy 网站上还有一个[自动生成的页面](https://bevy.org/learn/errors/)。

## Bevy Assets [#](https://bevy.org/news/bevy-0-6/#bevy-assets)

作者：@mockersf

包含 Bevy 插件、crate、应用和学习资源列表的精选 awesome-bevy GitHub 仓库现已重生为 [Bevy Assets](https://github.com/bevyengine/bevy-assets)！

Bevy Assets 引入了：

- 结构化的 toml 格式
- 资产图标
- [bevy-website 集成](https://bevy.org/assets)

这仅仅是个开始！我们计划与 [crates.io](http://crates.io/) 和 GitHub 集成，改进索引/标签/可搜索性，添加资产特定页面，更漂亮的样式，内容交付等等。最终我们希望这能成长为一个能够实现一流的、现代资产驱动工作流程的东西。

我们已经自动迁移了现有的 awesome-bevy 条目，但我们鼓励创建者自定义它们！如果你正在开发与 Bevy 相关的东西，我们强烈建议你[添加一个 Bevy Assets 条目](https://github.com/bevyengine/bevy-assets)。

## 双 MIT / Apache-2.0 许可 [#](https://bevy.org/news/bevy-0-6/#dual-mit-apache-2-0-license)

作者：@cart, @DJMcNab

感谢相关贡献者（[全部 246 位](https://github.com/bevyengine/bevy/issues/2373)），Bevy 现在采用 MIT **和** Apache-2.0 双许可，由开发者选择。这意味着开发者可以灵活地选择最适合其特定需求的许可证。我想强调的是，这现在**比仅 MIT 更不**严格，而不是更严格。

我最初选择专门在 MIT 许可下发布 Bevy 有多种原因：

- 人们和公司通常比其他任何许可都更了解和信任 MIT 许可。Apache 2.0 不太为人所知和信任。
- 它简短且易于理解
- 许多人不熟悉"多种许可选项……选择你最喜欢的"方法。我不想不必要地吓跑人们。
- 其他开源引擎如 Godot 使用仅 MIT 许可取得了很大成功

然而，出现了一些问题，使得在 MIT 和 Apache-2.0 下双许可 Bevy 变得很有吸引力：

- MIT 许可（有争议地）要求二进制文件为每个使用中的 MIT 库复制无数份相同的许可模板。Apache-2.0 允许我们将模板压缩到单个实例中。
- Apache-2.0 许可具有防止专利流氓的保护措施和明确的贡献授权条款。
- Rust 生态系统主要是 Apache-2.0。在该许可下可用有利于互操作，并为将 Bevy 代码上游到其他项目（Rust、异步生态系统等）打开了大门。
- Apache 许可与 GPLv2 不兼容，但 MIT 兼容。

## Bevy 组织变更 [#](https://bevy.org/news/bevy-0-6/#bevy-org-changes)

### 更多的拉取请求合并者！ [#](https://bevy.org/news/bevy-0-6/#more-pull-request-mergers)

我已经达到我的可扩展性限制一段时间了。构建我需要的引擎功能、快速审查每一个拉取请求以及保持我的心理健康一直……咳咳……具有挑战性。我已经走到了这一步……有时是通过过度工作，有时是让 PR 未合并的时间比我预期的更长。通过扩展，我们可以两全其美！

- @mockersf 现在拥有"无争议变更"的合并权限
- @alice-i-cecile 现在拥有"无争议文档变更"的合并权限

### 新的问题标签 [#](https://bevy.org/news/bevy-0-6/#new-issue-labels)

经过[关于命名约定和颜色的大量讨论](https://github.com/bevyengine/bevy/issues/2256)，我们终于有了一套[全新的问题标签](https://github.com/bevyengine/bevy/labels)（松散地受 rust 仓库启发）。[Bevy 分类团队](https://github.com/bevyengine/bevy/blob/main/CONTRIBUTING.md#how-were-organized)终于可以充分表达自己了！

### 全面的 CONTRIBUTING.md [#](https://bevy.org/news/bevy-0-6/#comprehensive-contributing-md)

作者：@alice-i-cecile

我们现在有一个相对完整的[贡献者指南](https://github.com/bevyengine/bevy/blob/main/CONTRIBUTING.md)。如果你有兴趣为 Bevy 贡献代码或文档，这是一个很好的起点！

### CI 构建系统改进 [#](https://bevy.org/news/bevy-0-6/#ci-build-system-improvements)

作者：@mockersf, @NathanSWard, @NiklasEi

我们在本次发布中对 CI 进行了大量改进：

- 我们现在在 cargo doc 警告时失败
- 我们现在使用 [cargo deny](https://github.com/EmbarkStudios/cargo-deny) 来防止漏洞、重复依赖和无效许可证
- PR 现在会自动标记为 `S-Needs-Triage` 标签
- CI 稳定性和速度改进
- 我们现在检查基准测试是否能构建
- 我们现在对 compile_fail 测试断言编译器错误，从而提供更严格的保证
- 示例现在使用 lavapipe（而不是 swiftshader）运行，以实现更快的 CI 验证

## Bevy 下一步计划？ [#](https://bevy.org/news/bevy-0-6/#what-s-next-for-bevy)

Bevy 的开发继续加速，我们目前无意放慢速度！除了我们正在进行的[许多 RFC](https://github.com/bevyengine/rfcs/pulls) 之外，我们还计划在未来几个月内解决以下问题：

### "火车"发布周期 [#](https://bevy.org/news/bevy-0-6/#the-train-release-schedule)

在最近两个 Bevy 版本中，我们对核心系统进行了大规模、彻底的更改。**Bevy 0.5** 是"我们重写 Bevy ECS 的那个版本"。**Bevy 0.6** 是"我们重写 Bevy 渲染器的那个版本"。这些大规模的重组需要时间，结果也阻碍了一堆其他有用的功能和错误修复。它们还造成了"冲刺"压力，需要快速完成大型功能以解锁发布。冲刺是不健康的，应该不惜一切代价避免！

[Bevy 社区](https://bevy.org/community/)已经达成了相对共识，我们应该有更规律、更可预测的发布周期。大型功能不再能够阻塞系统。

从现在起，我们将**大约**每三个月发布一次（这是一个上限……有时如果合理我们可能会提前发布）。在一个发布周期结束后，我们将开始准备发布。如果需要进行小调整，或者"生活中发生了什么事"……我们会很乐意推迟发布。但我们不会再因为"大项目"而推迟发布。

我们在平衡许多不同的考虑：

- 建立 Bevy 贡献者的信任，让他们的变更能够及时落地
- 建立 Bevy 用户的信任，让他们获得定期更新和错误修复
- 在发布之间留出足够的时间以减少 Bevy 插件生态系统的变动（Bevy 尚未"稳定"，但更长的发布周期提供了合理的"生态系统稳定性"窗口）
- 在发布中提供足够的内容以产生"热度"。Bevy 发布博文往往是社区的"集结号"，我不想失去这一点。
- 建立核心开发者适当的工作/生活平衡（冲刺是有害的！）

我们将随着时间的推移改进这个过程，看看什么最有效。

### 更多渲染器功能 [#](https://bevy.org/news/bevy-0-6/#more-renderer-features)

- **后处理栈 / HDR / 泛光**：HDR 和泛光[几乎进入了 Bevy 0.6](https://github.com/bevyengine/bevy/pull/2876)，但我们决定暂缓，以便我们可以对它们进行一些打磨，并构建一个合适的"模块化后处理栈"。
- **骨骼动画**：最终 Bevy 将拥有一个通用的、基于属性的动画系统（我们已经有一个[可工作的实现](https://github.com/bevyengine/bevy/pull/1429)）。我们一直在推迟添加骨骼动画，以便将其纳入该系统，但回过头来看，这是一个错误。人们**现在**就需要骨骼动画。短期内，我们将构建一个范围限定的 3d 骨骼动画系统，只是为了启动。之后，我们会将其移植到通用系统（无论它何时准备就绪）。
- **屏幕空间环境光遮蔽 (SSAO)**：一种流行且直接的环境光遮蔽近似方法，可以显著提高渲染质量。
- **全局光照 (GI)**：GI 将极大地提升"真实感"的感觉，因此短期内至少优先考虑一种形式的 GI 是值得的。这是一个复杂的话题，需要进行实验。
- **压缩纹理**：这将使场景加载更快，并减少 GPU 内存使用。
- **阴影滤镜和级联**：Rob Swain (@superdump) 已经在这方面做了大量工作，所以我们希望很快能在 Bevy 发布中看到成果。
- **PBR 着色器代码重用**：我们将使 PBR 着色器更加模块化，并使导入 PBR 着色器的特定部分更容易，从而更易于定义自定义 PBR 着色器。

### UI 刷新 [#](https://bevy.org/news/bevy-0-6/#ui-refresh)

我们将在今年开启 Bevy Editor 的建设。为此，我们需要对 Bevy UI 进行一系列改进：

- 改进的"数据驱动 UI"（可能是"响应式"）
- 一套完善的预构建小部件
- 总体上改进的用户体验

我们现在在 Bevy 社区中看到了大量的 UI 实验。在接下来的几个月里，我们将完善我们的范围，并开始"选择一个赢家"。

### 资产预处理 [#](https://bevy.org/news/bevy-0-6/#asset-preprocessing)

资产预处理是生产级游戏引擎的关键部分。它可以减少启动时间，减少 CPU 和 GPU 内存占用，支持更复杂的开发工作流程，使游戏资产的发布更容易，并减少部署游戏的最终大小。我们至今没有资产预处理系统也撑到了现在……但勉强而已。尽快解决这个问题是我的高优先级事项。

### 场景改进 [#](https://bevy.org/news/bevy-0-6/#scene-improvements)

嵌套场景、属性覆盖、内联资产和更漂亮的语法都在议程上。我们已经在这方面有了一些不错的实验性成果，所以我们应该会看到相对较快的进展。

### 新版 Bevy 手册 [#](https://bevy.org/news/bevy-0-6/#the-new-bevy-book)

当前的 Bevy 手册是学习如何设置 Bevy 和开始编写 Bevy 应用的好方法。但它几乎没有覆盖到 Bevy 能力的表面。

为了解决这个问题，@alice-i-cecile 已经开始[编写](https://github.com/bevyengine/bevy-website/pull/182)一本新的 Bevy 手册，目标是成为 Bevy 的完整学习资源。如果你有兴趣提供帮助，请联系他们！

### Bevy 月度通讯 [#](https://bevy.org/news/bevy-0-6/#the-bevy-monthly-newsletter)

[Bevy 社区](https://bevy.org/community/)是一个充满活力的活跃场所。目前大多数社区内容都发布在 [Bevy Discord](https://discord.gg/bevy) 的 `#showcase` 板块。即将推出的 Bevy 月度通讯将是一个整合的、可分享的资源，我们将发布到 Reddit 和 Twitter 等地方。
