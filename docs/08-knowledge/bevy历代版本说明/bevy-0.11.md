# Bevy 0.11

## 发布于 2023 年 7 月 9 日，作者：Bevy 贡献者

![文章的封面图](https://bevy.org/news/bevy-0-11/with_ssao.png)

感谢 **166** 位贡献者、**522** 个 pull request、社区审阅者以及我们的[**慷慨赞助商**](https://bevy.org/donate)，我们很高兴在 [crates.io](https://crates.io/crates/bevy) 上发布 **Bevy 0.11**！

对于那些还不了解的人，Bevy 是一个用 Rust 构建的、令人耳目一新的简单数据驱动游戏引擎。你可以查看我们的[快速入门指南](https://bevy.org/learn/quick-start/introduction/)来立即尝试。它永久免费且开源！你可以在 GitHub 上获取完整的[源代码](https://github.com/bevyengine/bevy)。查看 [Bevy Assets](https://bevy.org/assets) 获取社区开发的插件、游戏和学习资源合集。

要将现有的 Bevy App 或 Plugin 更新到 **Bevy 0.11**，请查看我们的 [0.10 到 0.11 迁移指南](https://bevy.org/learn/migration-guides/0.10-0.11/)。

自几个月前的上次发布以来，我们添加了大量新功能、Bug 修复和生活质量改进，以下是一些亮点：

- **屏幕空间环境光遮蔽（SSAO）**：通过模拟"间接"漫反射光的遮蔽来提升场景渲染质量
- **时间抗锯齿（TAA）**：一种流行的抗锯齿技术，使用运动向量将当前帧与过去帧混合，以平滑伪影
- **形态目标（Morph Targets）**：在网格上动画化顶点位置，在预定义状态之间过渡。非常适合角色自定义等场景！
- **鲁棒对比度自适应锐化（RCAS）**：智能地锐化渲染效果，与 TAA 配合良好
- **WebGPU 支持**：Bevy 现在可以使用现代 WebGPU Web API 在 Web 上更快地渲染，并支持更多功能
- **改进的着色器导入**：Bevy 着色器现在支持细粒度的导入和其他新功能
- **视差映射（Parallax Mapping）**：材质现在支持可选的高度图，通过视差效果让平面表面产生深度感
- **调度优先的 ECS API**：更简单、更符合人体工程学的 ECS 系统调度 API
- **即时模式 Gizmo 渲染**：轻松高效地渲染 2D 和 3D 形状，用于调试和编辑器场景
- **ECS 音频 API**：更直观、更地道的方式来播放音频
- **UI 边框**：UI 节点现在可以配置边框了！
- **网格 UI 布局**：Bevy UI 现在支持 CSS 风格的网格布局
- **UI 性能改进**：UI 批处理算法已更改，带来了显著的性能提升

## 屏幕空间环境光遮蔽 [#](https://bevy.org/news/bevy-0-11/#screen-space-ambient-occlusion)

作者：@JMS55, @danchia, @superdump

拖拽图片进行比较

![没有 SSAO 的 Sponza 场景，包含大量波斯风格的丝绒窗帘，看起来有些别扭](https://bevy.org/news/bevy-0-11/no_ssao.png)![启用了 SSAO 的 Sponza 场景，窗帘看起来更加真实和立体。SSAO 加深了褶皱间的凹陷，使窗帘看起来更有质感](https://bevy.org/news/bevy-0-11/with_ssao.png)

**仅 SSAO** ![ssao_only](https://bevy.org/news/bevy-0-11/ssao_only.png)

Bevy 现在支持屏幕空间环境光遮蔽（SSAO）。虽然 Bevy 已经通过阴影映射支持来自直接光源（[`DirectionalLight`](https://docs.rs/bevy/0.11.0/bevy/pbr/struct.DirectionalLight.html)、[`PointLight`](https://docs.rs/bevy/0.11.0/bevy/pbr/struct.PointLight.html)、[`SpotLight`](https://docs.rs/bevy/0.11.0/bevy/pbr/struct.SpotLight.html)）的阴影，但 Bevy 现在也支持来自_间接_漫反射光照（如 [`AmbientLight`](https://docs.rs/bevy/0.11.0/bevy/pbr/struct.AmbientLight.html) 或 [`EnvironmentMapLight`](https://docs.rs/bevy/0.11.0/bevy/pbr/struct.EnvironmentMapLight.html)）的阴影。

这些阴影通过屏幕空间深度和法线预计算来估算周围几何体阻挡了多少入射光，从而赋予场景更"落地"的感觉。你可以在新的 [SSAO 示例](https://github.com/bevyengine/bevy/blob/v0.11.0/examples/3d/ssao.rs) 中试用。

请注意，将 SSAO 与新加入的时间抗锯齿结合使用，会带来_大幅_的质量提升和噪点减少。

平台支持目前有限——仅支持 Vulkan、DirectX12 和 Metal。WebGPU 支持将在后续版本中添加。WebGL 可能不会支持，因为它没有计算着色器。

特别感谢 Intel 的开源项目 [XeGTAO](https://github.com/GameTechDev/XeGTAO)，它在开发此功能时提供了巨大帮助。

## 时间抗锯齿 [#](https://bevy.org/news/bevy-0-11/#temporal-anti-aliasing)

作者：@JMS55, @DGriffin91

拖拽图片进行比较

![使用 MSAA 的头盔模型抗锯齿效果。网格之间的边缘抗锯齿表现良好，但在锐利阴影和高光上可见锯齿](https://bevy.org/news/bevy-0-11/msaa_helmet.png)![使用 TAA，几乎看不到锯齿，但感觉有点"模糊"](https://bevy.org/news/bevy-0-11/taa_helmet.png)

除了 MSAA 和 FXAA，Bevy 现在还支持时间抗锯齿（TAA）作为抗锯齿选项。

TAA 的工作原理是将新渲染的帧与过去的帧混合，以平滑图像中的锯齿伪影。TAA 在业界越来越受欢迎，因为它能够掩盖如此多的渲染伪影：它平滑了阴影（包括全局光照和"投射"阴影）、网格边缘、纹理，并减少了反射表面上光的高光锯齿。然而，由于"平滑"效果如此明显，有些人更喜欢其他方法。

以下是 Bevy 支持的每种抗锯齿方法的优缺点快速概览：

- **多重采样抗锯齿（MSAA）**
    - 在平滑网格边缘方面表现良好（几何抗锯齿）。对高光锯齿没有帮助。性能消耗随三角形数量增加而增长，在三角形较多的场景中表现很差
- **快速近似抗锯齿（FXAA）**
    - 在几何和高光锯齿方面都做得不错。在所有场景中性能消耗很小。结果有些模糊，质量较低
- **时间抗锯齿（TAA）**
    - 在几何和高光锯齿方面表现非常好。在处理时间锯齿方面表现出色，即高频细节随时间、相机移动或动画而闪烁的问题。性能消耗适中，仅随屏幕分辨率增加。可能出现"鬼影"现象，即网格或光照效果可能留下随时间淡出的拖影。虽然 TAA 有助于减少时间锯齿，但它也可能引入额外的时间锯齿，尤其是在薄几何体或远距离渲染的纹理细节上。需要额外 2 个视图的 GPU 内存，以及启用运动向量和深度预处理。需要精确的运动向量和深度预处理，这使自定义材质变得更加复杂

TAA 的实现是一系列权衡的结果，并依赖于容易出错的启发式算法。在 Bevy 0.11 中，TAA 被标记为实验性功能，原因如下：

- TAA 目前无法与以下 Bevy 功能同时使用：蒙皮、形态目标和视差映射
- TAA 目前倾向于使图像略微变软，可以通过后处理锐化来解决
- 我们的 TAA 启发式算法目前不可由用户配置（这些启发式算法可能会发生变化和演进）

我们将在未来的版本中继续改进质量、兼容性和性能。如果你遇到任何 Bug，请报告！

你可以在 Bevy 改进后的[抗锯齿示例](https://github.com/bevyengine/bevy/blob/v0.11.0/examples/3d/anti_aliasing.rs)中比较我们所有的抗锯齿方法。

## 鲁棒对比度自适应锐化 [#](https://bevy.org/news/bevy-0-11/#robust-contrast-adaptive-sharpening)

作者：@Elabajaba

像 TAA 和 FXAA 这样的效果会导致最终渲染变得模糊。锐化后处理效果可以帮助抵消这一点。在 **Bevy 0.11** 中，我们移植了 AMD 的鲁棒对比度自适应锐化（RCAS）。

拖拽图片进行比较

![TAA](https://bevy.org/news/bevy-0-11/rcas_off.png)![TAA+RCAS](https://bevy.org/news/bevy-0-11/rcas_on.png)

注意头盔皮革部分的纹理清晰多了！

## 形态目标 [#](https://bevy.org/news/bevy-0-11/#morph-targets)

作者：@nicopap, @cart

自 0.7 版本以来，Bevy 就支持 3D 动画。

但它只支持_骨骼_动画，把一种常见的动画类型——_形态目标_（也叫 blendshapes、keyshapes，以及其他各种名称）——留在了路边。这是所有 3D 角色动画的鼻祖！[古惑狼](https://en.wikipedia.org/wiki/Crash_Bandicoot_\(video_game\)#Gameplay)的奔跑循环就使用了形态目标。

角色模型由 [Samuel Rosario](https://www.artstation.com/zambrah) 提供（© 保留所有权利），经许可使用。由 nicopap 修改，使用了 Demeter Dzadik 为 Blender Studios 制作的 [Snow](https://studio.blender.org/characters/snow/v2/) 角色纹理 [(🅯 CC-BY)](https://creativecommons.org/licenses/by/4.0/)。

如今，动画师通常使用骨骼绑定的骨架进行大范围动作，并使用形态目标来完善细节动作。

然而，对于游戏资产来说，艺术家用于面部和手部的复杂骨架绑定过于沉重。通常，姿势会被"烘焙"成形态姿势，面部表情的过渡则通过引擎中的形态目标来处理。

形态目标是一种非常简单的动画方法。拿一个模型，有一个基础顶点位置，移动顶点来创建多个姿势：

**默认**

![角色面部中性表情的线框渲染](https://bevy.org/news/bevy-0-11/default-pose-bw.png)

**皱眉**

![角色皱眉的线框渲染](https://bevy.org/news/bevy-0-11/frown-pose-bw.png)

**假笑**

![角色假笑的线框渲染](https://bevy.org/news/bevy-0-11/smirk-pose-bw.png)

将这些姿势存储为默认基础网格与变体姿势之间的差值，然后在运行时_混合_每个姿势。现在我们有了与基础网格的差值，只需将其加到基础顶点位置上，就能得到变体姿势。

就是这样，形态目标着色器看起来像这样：

```rust
fn morph_vertex(vertex: Vertex) {
    for (var i: u32 = 0u; i < pose_count(); i++) {
        let weight = weight_for_pose(i);
        vertex.position += weight * get_difference(vertex.index, position_offset, i);
        vertex.normal += weight * get_difference(vertex.index, normal_offset, i);
    }
}
```

在 Bevy 中，我们将每个姿势的权重存储在 `MorphWeights` 组件中。

```rust
fn set_weights_system(mut morph_weights: Query<&mut MorphWeights>) {
    for mut entity_weights in &mut morph_weights {
        let weights = entity_weights.weights_mut();

        weights[0] = 0.5;
        weights[1] = 0.25;
    }
}
```

现在假设我们有两个形态目标，（1）皱眉姿势，（2）假笑姿势：

**[0.0, 0.0]**

默认姿势

![中性面部表情](https://bevy.org/news/bevy-0-11/morph_target_default-0.png)

**[1.0, 0.0]**

仅皱眉

![皱眉](https://bevy.org/news/bevy-0-11/morph_target_frown-0.png)

**[0.0, 1.0]**

仅假笑

![假笑](https://bevy.org/news/bevy-0-11/morph_target_smirk.png)

**[0.5, 0.0]**

半皱眉

![轻微皱眉](https://bevy.org/news/bevy-0-11/morph_target_frown-half-0.png)

**[1.0, 1.0]**

两者最大

![做鬼脸](https://bevy.org/news/bevy-0-11/morph_target_both-0.png)

**[0.5, 0.25]**

两者各一点

![轻微皱眉/假笑](https://bevy.org/news/bevy-0-11/morph_target_smirk-quarter-frown-half-0.png)

虽然概念上很简单，但它需要向 GPU 传输大量数据。数千个顶点，每个 288 位，多种模型变体，有时上百种。

我们将顶点数据作为像素存储在一个 3D 纹理中。这使得形态目标不仅能在 WebGPU 上运行，也能在 WebGL2 wgpu 后端的运行。

这可以通过多种方式改进，但对于初始实现来说已经足够了。

## 视差映射 [#](https://bevy.org/news/bevy-0-11/#parallax-mapping)

作者：@nicopap

Bevy 现在支持视差映射和高度图。在为材质赋予"深度幻觉"方面，视差映射让法线映射相形见绌。本视频的上半部分使用了视差映射加法线映射，而下半部分仅使用法线映射：

地球视图、高程和夜景视图由 NASA 提供（公有领域）

注意变化的不仅仅是像素的着色，还有它们在屏幕上的实际位置。山顶遮挡了它们后面的山脊。高山的移动速度比沿海地区快。

视差映射根据几何体表面的视角和深度来移动像素。为平面表面增添了真正的 3D 深度。

所有这些，都不需要向几何体添加一个顶点。整个地球仪恰好有 648 个顶点。与更初级的着色器（如置换映射）不同，视差映射只需要一个额外的灰度图像，称为 `depth_map`。

游戏通常对鹅卵石或砖墙使用视差映射，所以让我们在 Bevy 中制作一堵砖墙！首先，我们生成一个网格：

```rust
commands.spawn(PbrBundle {
    mesh: meshes.add(shape::Box::new(30.0, 10.0, 1.0).into()),
    material: materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    }),
    ..default()
});
```

![一个 3D 沙漠场景，有两堵平坦的白墙和一条鹅卵石小路蜿蜒其间](https://bevy.org/news/bevy-0-11/parallax_mapping_none_mini.jpg)

当然，这只是一个平坦的白色盒子，我们没有添加任何纹理。所以让我们添加一个法线映射：

```rust
normal_map_texture: Some(assets.load("normal_map.png")),
```

![相同场景，带法线映射](https://bevy.org/news/bevy-0-11/parallax_mapping_normals_mini.jpg)

这好多了。阴影也会根据光照方向而变化！然而，角落上的高光过于强烈，几乎像噪点一样。

让我们看看高度图如何帮助解决这个问题：

```rust
depth_map: Some(assets.load("depth_map.png")),
```

![相同场景，带高度纹理](https://bevy.org/news/bevy-0-11/parallax_mapping_depth_mini.jpg)

我们消除了噪点！还有那种令人愉悦的 3D 感觉，让人想起 90 年代游戏预渲染的过场动画序列。

那么，这是怎么回事，为什么视差映射能消除墙上难看的高光呢？

这是因为视差映射将砖块之间的缝隙凹陷下去，使它们被砖块本身遮挡住。

![上段文字的示意图](https://bevy.org/news/bevy-0-11/ridge-light-view-1.svg)

由于法线映射并不会"移动"阴影区域，只是以不同方式着色，所以我们得到了那些尴尬的高光。有了视差映射，它们就消失了。

拖拽图片进行比较

![仅法线](https://bevy.org/news/bevy-0-11/parallax_mapping_normals.jpg)![视差与法线映射](https://bevy.org/news/bevy-0-11/parallax_mapping_depth.jpg)

Bevy 中的视差映射仍然非常有限。最麻烦的一点是它不是一个标准的 glTF 功能，这意味着如果深度纹理来自 GLTF 文件，则需要通过编程方式添加到材质中。

此外，视差映射与时间抗锯齿着色器不兼容，在曲面上效果不佳，也不会影响物体的轮廓。

然而，这些并不是视差映射的根本性限制，可能会在未来得到修复。

## 天空盒 [#](https://bevy.org/news/bevy-0-11/#skyboxes)

作者：@JMS55, @superdump

![天空盒](https://bevy.org/news/bevy-0-11/skybox.png)

Bevy 现在内置支持将 HDRI 环境显示为场景背景。

只需将新的 [`Skybox`](https://docs.rs/bevy/0.11.0/bevy/core_pipeline/struct.Skybox.html) 组件挂载到你的 [`Camera`](https://docs.rs/bevy/0.11.0/bevy/render/camera/struct.Camera.html) 上。它与现有的 [`EnvironmentMapLight`](https://docs.rs/bevy/0.11.0/bevy/pbr/struct.EnvironmentMapLight.html) 配合良好，后者会使用环境映射来照亮场景。

我们还计划在未来某个时候添加对内建程序化天空盒的支持！

## WebGPU 支持 [#](https://bevy.org/news/bevy-0-11/#webgpu-support)

作者：@mockersf，以及 Bevy 开发过程中的许多其他人

![webgpu](https://bevy.org/news/bevy-0-11/webgpu.svg)

Bevy 现在在 Web 上支持 WebGPU 渲染（除了 WebGL 2）。WebGPU 支持仍在推广中，但如果你有[支持的浏览器](https://caniuse.com/webgpu)，你可以探索我们新的[在线 WebGPU 示例](https://bevy.org/examples-webgpu)页面。

### 什么是 WebGPU？ [#](https://bevy.org/news/bevy-0-11/#what-is-webgpu)

WebGPU 是一个[令人兴奋的新 Web 标准](https://github.com/gpuweb/gpuweb)，用于进行现代 GPU 图形和计算。它从 Vulkan、Direct3D 12 和 Metal 中汲取灵感。实际上，它通常是在这些 API 之上实现的。WebGPU 让我们能够访问比 WebGL2 更多的 GPU 功能（例如计算着色器），并且有可能更快。这意味着 Bevy 更多的原生渲染器功能现在也可以在 Web 上使用。它还使用了新的 [WGSL 着色器语言](https://www.w3.org/TR/WGSL)。我们对 WGSL 随时间的演变非常满意，Bevy 内部也在着色器中使用它。我们还添加了易用性功能，如导入！但使用 Bevy，你仍然可以选择使用 GLSL。

### 工作原理 [#](https://bevy.org/news/bevy-0-11/#how-it-works)

Bevy 构建在 [wgpu](https://github.com/gfx-rs/wgpu) 库之上，这是一个现代的低级 GPU API，可以针对几乎所有流行的 API：Vulkan、Direct3D 12、Metal、OpenGL、WebGL2 和 WebGPU。最佳的后端 API 会根据平台自动选择。它是一个"原生"渲染 API，但总体上遵循 WebGPU 的术语和 API 设计。与 WebGPU 不同，它可以提供对原生 API 的直接访问，这意味着 Bevy [享有"集所有优势于一身"的境地](https://bevy.org/news/bevy-webgpu/#how-it-works)。

### WebGPU 示例 [#](https://bevy.org/news/bevy-0-11/#webgpu-examples)

点击下方图片之一，查看我们的在线 WebGPU 示例（如果你的[浏览器支持](https://caniuse.com/webgpu)的话）：

[![webgpu 示例](https://bevy.org/news/bevy-0-11/webgpu_examples.png)](https://bevy.org/examples-webgpu)

## 改进的着色器导入 [#](https://bevy.org/news/bevy-0-11/#improved-shader-imports)

作者：@robtfm

Bevy 的渲染引擎拥有大量出色的选项和功能。例如，PBR `StandardMaterial` 管线支持桌面/WebGPU 和 WebGL、6 个可选网格属性、4 个可选纹理以及大量可选功能，如雾效、蒙皮和阿尔法混合模式，而且每个版本都会增加更多功能。

许多功能组合需要专门的着色器变体，而着色器代码超过 3000 行，分布在 50 个文件中，基于文本替换的着色器处理器已经开始捉襟见肘了。

这个版本我们改用 [naga_oil](https://github.com/bevyengine/naga_oil)，它为我们提供了一个基于模块的着色器框架。它将每个文件单独编译为 naga 的 IR，然后按需将它们组合成最终的着色器。这还没有带来太多可见的影响，但确实提供了一些直接的好处：

- 引擎的着色器代码更易于导航，更少神秘感。以前只有一个全局作用域，所以即使只是间接导入的项目也可以被引用。这有时使得很难定位引用背后的实际代码。现在项目必须显式导入，所以你总是可以通过查看当前文件就知道变量或函数来自哪里：
    ![导入的项目](https://bevy.org/news/bevy-0-11/imported_items.png)
- 着色器现在有 codespan 报告，错误会指向着色器文件和行号，避免在复杂着色器代码库中大量抓狂：
    ![codespan](https://bevy.org/news/bevy-0-11/codespan.png)
- naga_oil 的预处理器支持更多的条件指令，你可以使用 `#else if` 和 `#else ifndef`，以及之前就支持的 `#else ifdef`
- 函数、变量和结构体都有适当的作用域，因此着色器文件不需要使用全局唯一名称来避免冲突
- Shader defs 可以直接添加到模块中。例如，任何导入 `bevy_pbr::mesh_view_types` 的着色器现在都会自动定义 `MAX_DIRECTIONAL_LIGHTS`，不再需要记得为每个使用该模块的新管线添加它。

未来的可能性更令人兴奋。使用 naga IR 为一系列不错的功能打开了大门，我们希望在未来版本中带来这些功能：

- 自动绑定槽位分配将允许插件扩展核心视图绑定组，这意味着用于光照和阴影方法、常见材质属性等功能的独立插件变得可行。这将使我们能够模块化核心管线，使代码库的增长——同时保持对多个目标的支持——更加可持续
- "虚拟"着色器函数将允许用户修改核心函数（如光照），并可能导致模板风格的材质系统，用户可以提供"钩子"，在管线的正确位置被调用
- 语言互操作：混合搭配 glsl 和 wgsl，因此 Bevy 的 PBR 管线功能可以从你的 GLSL 材质着色器中访问，或者为 GLSL 编写的工具函数可以在 WGSL 代码中使用。我们希望这也能扩展到 spirv（以及 rust-gpu）
- 更多我们还没想到的酷东西。能够在运行时检查和修改着色器非常强大，使很多成为可能！

## UI 节点边框 [#](https://bevy.org/news/bevy-0-11/#ui-node-borders)

作者：@ickshonpe

UI 节点现在可以绘制边框了，其颜色可以通过新的 [`BorderColor`](https://docs.rs/bevy/0.11.0/bevy/ui/struct.BorderColor.html) 组件进行配置：

![边框](https://bevy.org/news/bevy-0-11/borders.png)

```rust
commands.spawn(ButtonBundle {
    style: Style {
        border: UiRect::all(Val::Px(5.0)),
        ..default()
    },
    border_color: BorderColor(Color::rgb(0.9, 0.9, 0.9)),
    ..default()
})
```

边框的每一侧都可以单独配置：

![边框各侧](https://bevy.org/news/bevy-0-11/border-sides.png)

## 网格 UI 布局 [#](https://bevy.org/news/bevy-0-11/#grid-ui-layout)

作者：@nicoburns

在 Bevy UI 中，我们启用了所使用的布局库（[Taffy](https://github.com/DioxusLabs/taffy)）中的新 `grid` 功能。这实现了 CSS 风格的网格布局：

![网格](https://bevy.org/news/bevy-0-11/grid.png)

这可以在 [`Style`](https://docs.rs/bevy/0.11.0/bevy/ui/struct.Style.html) 组件上进行配置：

```rust
Style {
    /// 对该节点使用网格布局
    display: Display::Grid,
    /// 使网格具有 1:1 的宽高比
    /// 这意味着宽度将根据高度进行调整
    aspect_ratio: Some(1.0),
    // 在网格周围添加 24px 的内边距
    padding: UiRect::all(Val::Px(24.0)),
    /// 设置网格为 4 列，每列大小为 minmax(0, 1fr)
    /// 这会创建 4 个完全等宽的列
    grid_template_columns: RepeatedGridTrack::flex(4, 1.0),
    /// 设置网格为 4 行，每行大小为 minmax(0, 1fr)
    /// 这会创建 4 个完全等高行
    grid_template_rows: RepeatedGridTrack::flex(4, 1.0),
    /// 设置行和列之间的间距/间隔为 12px
    row_gap: Val::Px(12.0),
    column_gap: Val::Px(12.0),
    ..default()
},
```

## 调度优先的 ECS API [#](https://bevy.org/news/bevy-0-11/#schedule-first-ecs-apis)

作者：@cart

在 **Bevy 0.10** 中，我们引入了 [ECS Schedule V3](https://bevy.org/news/bevy-0-10/#ecs-schedule-v3)，它_极大_地改进了 Bevy ECS 系统调度的能力：调度器 API 的人体工程学、系统链式调用、在任何调度点运行排他系统并应用延迟系统操作的能力、统一的调度、可配置的 System Set、运行条件以及更好的 State 系统。

然而，很快我们就发现新系统仍有一些需要改进的地方：

- **Base Set 难以理解且容易出错**：到底_什么是_ Base Set？什么时候使用它们？它们为什么存在？为什么我的排序因为不兼容的 Base Set 排序而隐式无效？为什么有些调度有默认的 Base Set 而其他的没有？[Base Set 太令人困惑了！](https://github.com/bevyengine/bevy/pull/8079#base-set-confusion)
- **调度 System 的方式太多**：我们积累了太多的调度 API。截至 Bevy **0.10**，我们有[_六_种不同的方式](https://github.com/bevyengine/bevy/pull/8079#unify-system-apis)来将系统添加到"startup"调度。太多了！
- **太多隐式配置**：既有默认的 Schedule，也有默认的 Base Set。在某些情况下系统有默认的调度或默认的 Base Set，但在其他情况下却没有！[系统的调度和配置应该是明确和清晰的](https://github.com/bevyengine/bevy/pull/8079#schedule-should-be-clear)。
- **将系统添加到调度不够直观**：像 `add_system(foo.in_schedule(CoreSchedule::Startup))` 这样的东西既不好输入也不好看。我们创建了特殊情况的辅助方法，如 `add_startup_system(foo)`，但[这需要更多的内部代码，用户定义的调度无法受益于特殊处理，而且完全隐藏了 `CoreSchedule::Startup` 符号！](https://github.com/bevyengine/bevy/pull/8079#ergonomic-system-adding)。

### 理清复杂性 [#](https://bevy.org/news/bevy-0-11/#unraveling-the-complexity)

如果你试图理解这些概念时开始眼花缭乱，或者像"隐式添加到 `CoreSet::Update` Base Set"这样的表述让你感到恐惧……别担心。经过[大量仔细的思考](https://github.com/bevyengine/bevy/pull/8079)，我们理清了复杂性，构建了清晰简单的东西。

在 **Bevy 0.11** 中，"调度心智模型"简单多了，这要归功于**调度优先的 ECS API**：

```rust
app
    .add_systems(Startup, (a, b))
    .add_systems(Update, (c, d, e))
    .add_systems(FixedUpdate, (f, g))
    .add_systems(PostUpdate, h)
    .add_systems(OnEnter(AppState::Menu), enter_menu)
    .add_systems(OnExit(AppState::Menu), exit_menu)
```

- **调度系统的方式_恰好只有_一种**
    - 调用 `add_systems`，指定调度名称，然后指定一个或多个系统
- **Base Set 已被完全移除，取而代之的是具有友好/简短名称的 Schedule**
    - 例如：`CoreSet::Update` Base Set 变成了 `Update`
- **没有隐式或暗示的配置**
    - 默认的 Schedule 和默认的 Base Set 都不存在
- **语法美观且符合人体工程学**
    - Schedule 放在前面，这样格式化时会"对齐"

作为对比，展开看看以前是什么样的！

```rust
app
    // Startup 系统变体 1.
    // 有一个隐含的默认 StartupSet::Startup base set
    // 有一个隐含的 CoreSchedule::Startup schedule
    .add_startup_systems((a, b))
    // Startup 系统变体 2.
    // 有一个隐含的默认 StartupSet::Startup base set
    // 有一个隐含的 CoreSchedule::Startup schedule
    .add_systems((a, b).on_startup())
    // Startup 系统变体 3.
    // 有一个隐含的默认 StartupSet::Startup base set
    .add_systems((a, b).in_schedule(CoreSchedule::Startup))
    // Update 系统变体 1.
    // 隐含 `CoreSet::Update` base set 和 `CoreSchedule::Main`
    .add_system(c)
    // Update 系统变体 2 (注意 add_system 与 add_systems 的区别)
    // 隐含 `CoreSet::Update` base set 和 `CoreSchedule::Main`
    .add_systems((d, e))
    // 没有隐含的默认 base set，因为 CoreSchedule::FixedUpdate 没有
    .add_systems((f, g).in_schedule(CoreSchedule::FixedUpdate))
    // 隐含 `CoreSchedule::Main`，in_base_set 覆盖了默认的 CoreSet::Update set
    .add_system(h.in_base_set(CoreSet::PostUpdate))
    // 没有隐含的默认 base set
    .add_systems(enter_menu.in_schedule(OnEnter(AppState::Menu)))
    // 没有隐含的默认 base set
    .add_systems(exit_menu.in_schedule(OnExit(AppState::Menu)))
```

注意常规的"system set"仍然存在！你仍然可以使用 set 来组织和排序你的系统：

```rust
app.add_systems(Update, (
    (walk, jump).in_set(Movement),
    collide.after(Movement),
))
```

`configure_set` API 也已调整以保持一致性：

```rust
// Bevy 0.10
app.configure_set(Foo.after(Bar).in_schedule(PostUpdate))
// Bevy 0.11
app.configure_set(PostUpdate, Foo.after(Bar))
```

## 嵌套系统元组和链式调用 [#](https://bevy.org/news/bevy-0-11/#nested-system-tuples-and-chaining)

作者：@cart

现在可以在 `.add_systems` 调用中无限嵌套系统元组了！

```rust
app.add_systems(Update, (
    (a, (b, c, d, e), f),
    (g, h),
    i
))
```

乍一看，这似乎不是很有用。但与按元组配置结合使用时，它允许你轻松而清晰地表达调度：

```rust
app.add_systems(Update, (
    (attack, defend).in_set(Combat).before(check_health),
    check_health,
    (handle_death, respawn).after(check_health)
))
```

`.chain()` 也被适配以支持任意嵌套！上面例子中的排序可以这样重新表达：

```rust
app.add_systems(Update,
    (
        (attack, defend).in_set(Combat),
        check_health,
        (handle_death, respawn)
    ).chain()
)
```

这将先运行 `attack` 和 `defend`（并行），然后运行 `check_health`，再运行 `handle_death` 和 `respawn`（并行）。

这实现了强大而富有表现力的"图状"排序表达式：

```rust
app.add_systems(Update,
    (
        (a, (b, c, d).chain()),
        (e, f),
    ).chain()
)
```

这将并行运行 `a` 和 `b->c->d`，然后在这些完成后，再并行运行 `e` 和 `f`。

## Gizmo [#](https://bevy.org/news/bevy-0-11/#gizmos)

作者：@devil-ira, @mtsr, @aevyrie, @jannik4, @lassade, @The5-1, @Toqozz, @nicopap

能够绘制简单的 2D 和 3D 形状和线条对编辑器控件和调试视图等场景通常很有帮助。游戏开发是一种非常"空间"的事情，能够快速绘制形状就是视觉版的"print line 调试"。它有助于回答诸如"这条射线方向对吗？"和"这个碰撞体够大吗？"之类的问题。

在 **Bevy 0.11** 中，我们添加了一个"即时模式"[`Gizmos`](https://docs.rs/bevy/0.11.0/bevy/gizmos/gizmos/struct.Gizmos.html) 绘制 API，使这些事情变得简单高效。在 2D 和 3D 中，你可以绘制线条、矩形、圆形、弧线、球体、立方体、线段条等等！

**2D Gizmo** ![2d gizmos](https://bevy.org/news/bevy-0-11/2d_gizmos.png) **3D Gizmo** ![3d gizmos](https://bevy.org/news/bevy-0-11/3d_gizmos.png)

从任何系统中你都可以生成形状（适用于 2D 和 3D）：

```rust
fn system(mut gizmos: Gizmos) {
    // 2D
    gizmos.line_2d(Vec2::new(0., 0.), Vec2::new(0., 10.), Color::RED);
    gizmos.circle_2d(Vec2::new(0., 0.), 40., Color::BLUE);
    // 3D
    gizmos.circle(Vec3::ZERO, Vec3::Y, 3., Color::BLACK);
    gizmos.ray(Vec3::new(0., 0., 0.), Vec3::new(5., 5., 5.), Color::BLUE);
    gizmos.sphere(Vec3::ZERO, Quat::IDENTITY, 3.2, Color::BLACK)
}
```

由于该 API 是"即时模式"的，gizmo 只会在它们被"排队"的帧上绘制，这意味着你无需担心清理 gizmo 状态！

Gizmo 是批量绘制的，所以非常便宜。你可以拥有成百上千个！

## ECS 音频 API [#](https://bevy.org/news/bevy-0-11/#ecs-audio-apis)

作者：@inodentry

Bevy 的音频播放 API 已经重新设计，以便更干净地集成到 Bevy 的 ECS 中。

在以前的 Bevy 版本中，你这样播放音频：

```rust
#[derive(Resource)]
struct MyMusic {
    sink: Handle<AudioSink>,
}

fn play_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    audio_sinks: Res<Assets<AudioSink>>
) {
    let weak_handle = audio.play(asset_server.load("my_music.ogg"));
    let strong_handle = audio_sinks.get_handle(weak_handle);
    commands.insert_resource(MyMusic {
        sink: strong_handle,
    });
}
```

仅仅是播放一个声音就需要这么多样板代码！然后要调整播放，你需要像这样访问 [`AudioSink`](https://docs.rs/bevy/0.11.0/bevy/audio/struct.AudioSink.html)：

```rust
fn pause_music(my_music: Res<MyMusic>, audio_sinks: Res<Assets<AudioSink>>) {
    if let Some(sink) = audio_sinks.get(&my_music.sink) {
        sink.pause();
    }
}
```

将音频播放视为资源产生了一系列问题，并且显然与 Bevy Scenes 等功能配合不佳。在 **Bevy 0.11** 中，音频播放被表示为一个带有 [`AudioBundle`](https://docs.rs/bevy/0.11.0/bevy/audio/type.AudioBundle.html) 组件的 [`Entity`](https://docs.rs/bevy/0.11.0/bevy/ecs/entity/struct.Entity.html)：

```rust
#[derive(Component)]
struct MyMusic;

fn play_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioBundle {
            source: asset_server.load("my_music.ogg"),
            ..default()
        },
        MyMusic,
    ));
}
```

[`PlaybackSettings`](https://docs.rs/bevy/0.11.0/bevy/audio/struct.PlaybackSettings.html) 结构体中的 `mode` 字段提供了一种直接的方式来管理这些音频实体的生命周期。

通过传入 [`PlaybackMode`](https://docs.rs/bevy/0.11.0/bevy/audio/enum.PlaybackMode.html)，你可以选择是播放一次还是重复播放，分别使用 `Once` 和 `Loop`。如果你预计音频可能会再次播放，可以使用 `Despawn` 临时卸载它以节省资源，或者如果是一次性效果，使用 `Remove` 立即释放其内存。

```rust
AudioBundle {
    source: asset_server.load("hit_sound.ogg"),
    settings: PlaybackSettings {
        mode: PlaybackMode::Despawn,
        ..default()
    }
}
```

简单多了！要调整播放，你可以查询 [`AudioSink`](https://docs.rs/bevy/0.11.0/bevy/audio/struct.AudioSink.html) 组件：

```rust
fn pause_music(query_music: Query<&AudioSink, With<MyMusic>>) {
    if let Ok(sink) = query.get_single() {
        sink.pause();
    }
}
```

## 全局音频音量 [#](https://bevy.org/news/bevy-0-11/#global-audio-volume)

作者：@mrchantey

Bevy 现在有了全局音量级别，可以通过 [`GlobalVolume`] 资源配置：

```rust
app.insert_resource(GlobalVolume::new(0.2));
```

## 场景中的资源支持 [#](https://bevy.org/news/bevy-0-11/#resource-support-in-scenes)

作者：@Carbonhell, @Davier

Bevy 的场景格式是一个非常实用的工具，用于将游戏状态序列化和反序列化到场景文件中。

以前，捕获的状态仅限于实体及其组件。在 **Bevy 0.11** 中，场景现在也支持序列化资源。

这为场景格式添加了一个新的 `resources` 字段：

```rust
(
    resources: {
        "my_game::stats::TotalScore": (
            score: 9001,
        ),
    },
    entities: {
        // 实体场景数据...
    },
)
```

## 场景过滤 [#](https://bevy.org/news/bevy-0-11/#scene-filtering)

作者：@MrGVSV

当序列化数据到场景时，默认会序列化所有组件和[资源](https://bevy.org/news/bevy-0-11/#resource-support-in-scenes)。在之前的版本中，你必须使用给定的 `TypeRegistry` 作为过滤器，排除你不希望包含的类型。

在 0.11 中，现在有了专门的 `SceneFilter` 类型，使得过滤更简单、更清晰、更直观。它可以与 [`DynamicSceneBuilder`](https://docs.rs/bevy/0.11.0/bevy/prelude/struct.DynamicSceneBuilder.html) 一起使用，以对实际序列化的内容进行精细控制。

我们可以 `allow`（允许）一个类型的子集：

```rust
let mut builder = DynamicSceneBuilder::from_world(&world);
let scene = builder
    .allow::<ComponentA>()
    .allow::<ComponentB>()
    .extract_entity(entity)
    .build();
```

或者 `deny`（拒绝）它们：

```rust
let mut builder = DynamicSceneBuilder::from_world(&world);
let scene = builder
    .deny::<ComponentA>()
    .deny::<ComponentB>()
    .extract_entity(entity)
    .build();
```

## 默认字体 [#](https://bevy.org/news/bevy-0-11/#default-font)

作者：@mockersf

Bevy 现在支持可配置的默认字体，并嵌入了一个微小的默认字体（[Fira Mono](https://fonts.google.com/specimen/Fira+Mono) 的最小化版本）。如果你在整个项目中使用了通用字体，这将非常有用。而且它使得在使用"占位字体"进行原型开发时更加容易，无需担心在每个节点上设置字体。

![默认字体](https://bevy.org/news/bevy-0-11/default_font.png)

## UI 纹理图集支持 [#](https://bevy.org/news/bevy-0-11/#ui-texture-atlas-support)

作者：@mwbryant

以前，UI `ImageBundle` 节点只能使用完整图像的句柄，没有一种符合人体工程学的方式在 UI 中使用 `TextureAtlas`。在这个版本中，我们增加了对 `AtlasImageBundle` UI 节点的支持，将现有的 `TextureAtlas` 支持带入了 UI。

这是通过合并现有的文本渲染选择使用哪个字形的机制和 `TextureAtlasSprite` 的机制来实现的。

## 手柄震动 API [#](https://bevy.org/news/bevy-0-11/#gamepad-rumble-api)

作者：@johanhelsing, @nicopap

你现在可以使用 `EventWriter<GamepadRumbleRequest>` 系统参数来触发控制器的力反馈电机。

[`gilrs`](https://crates.io/crates/gilrs) 是 Bevy 用于手柄支持的 crate，它允许控制力反馈电机。遗憾的是，在 Bevy 中没有简便的方法来访问力反馈 API，除非进行繁琐的记账工作。

现在 Bevy 有了 `GamepadRumbleRequest` 事件来做到这一点。

```rust
fn rumble_system(
    gamepads: Res<Gamepads>,
    mut rumble_requests: EventWriter<GamepadRumbleRequest>,
) {
    for gamepad in gamepads.iter() {
        rumble_requests.send(GamepadRumbleRequest::Add {
            gamepad,
            duration: Duration::from_secs(5),
            intensity: GamepadRumbleIntensity::MAX,
        });
    }
}
```

`GamepadRumbleRequest::Add` 事件触发力反馈电机，控制震动的持续时间、要激活的电机以及震动强度。`GamepadRumbleRequest::Stop` 立即停止所有电机。

## 新的默认色调映射方法 [#](https://bevy.org/news/bevy-0-11/#new-default-tonemapping-method)

作者：@JMS55

在 **Bevy 0.10** 中，我们[使色调映射可配置，并提供了大量新的色调映射选项](https://bevy.org/news/bevy-0-10/#more-tonemapping-choices)。在 **Bevy 0.11** 中，我们将默认的色调映射方法从"Reinhard luminance"色调映射切换为"TonyMcMapface"：

拖拽图片进行比较

![Reinhard-luminance](https://bevy.org/news/bevy-0-11/tm_reinhard_luminance.png)![TonyMcMapface](https://bevy.org/news/bevy-0-11/tm_tonymcmapface.png)

TonyMcMapface（[由 Tomasz Stachowiak 创建](https://github.com/h3r2tic/tony-mc-mapface)）是一种更加中性的显示变换，试图尽可能接近输入的"光"。这有助于保留场景中的艺术选择。值得注意的是，亮色会在整个光谱上降低饱和度（与 Reinhard luminance 不同）。与 Reinhard luminance 相比，它与 bloom 的配合也好得多。

## EntityRef 查询 [#](https://bevy.org/news/bevy-0-11/#entityref-queries)

作者：@james7132

[`EntityRef`](https://docs.rs/bevy/0.11.0/bevy/ecs/world/struct.EntityRef.html) 现在实现了 [`WorldQuery`](https://docs.rs/bevy/0.11.0/bevy/ecs/query/trait.WorldQuery.html)，这使得在 ECS 系统中查询任意组件更加容易：

```rust
fn system(query: Query<EntityRef>) {
    for entity in &query {
        if let Some(mesh) = entity.get::<Handle<Mesh>>() {
            let transform = entity.get::<Transform>().unwrap();
        }
    }
}
```

请注意，[`EntityRef`](https://docs.rs/bevy/0.11.0/bevy/ecs/world/struct.EntityRef.html) 查询默认会访问整个 [`World`](https://docs.rs/bevy/0.11.0/bevy/ecs/world/struct.World.html) 中的每个实体和每个组件。这意味着它们会与任何"可变"查询冲突：

```rust
/// 这些查询会冲突，使该系统无效
fn system(query: Query<EntityRef>, mut enemies: Query<&mut Enemy>) { }
```

为了解决冲突（或减少访问的实体数量），你可以添加过滤器：

```rust
/// 这些查询不会冲突
fn system(
    players: Query<EntityRef, With<Player>>,
    mut enemies: Query<&mut Enemy, Without<Player>>
) {
    // 仅遍历玩家
    for entity in &players {
        if let Some(mesh) = entity.get::<Handle<Mesh>>() {
            let transform = entity.get::<Transform>().unwrap();
        }
    }
}
```

请注意，直接查询你想要的组件通常会更符合人体工程学（也更高效）：

```rust
fn system(players: Query<(&Transform, &Handle<Mesh>), With<Player>>) {
    for (transform, mesh) in &players {
    }
}
```

## 截图 API [#](https://bevy.org/news/bevy-0-11/#screenshot-api)

作者：@TheRawMeatball

Bevy 现在有一个简单的截图 API，可以将指定窗口的截图保存到磁盘：

```rust
fn take_screenshot(
    mut screenshot_manager: ResMut<ScreenshotManager>,
    input: Res<Input<KeyCode>>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    if input.just_pressed(KeyCode::Space) {
        screenshot_manager
            .save_screenshot_to_disk(primary_window.single(), "screenshot.png")
            .unwrap();
    }
}
```

## RenderTarget::TextureView [#](https://bevy.org/news/bevy-0-11/#rendertarget-textureview)

作者：@mrchantey

[`Camera`](https://docs.rs/bevy/0.11.0/bevy/render/camera/struct.Camera.html) 的 [`RenderTarget`](https://docs.rs/bevy/0.11.0/bevy/render/camera/enum.RenderTarget.html) 现在可以设置为一个 wgpu [`TextureView`](https://docs.rs/bevy/0.11.0/bevy/render/render_resource/struct.TextureView.html)。这允许第三方 Bevy Plugin 管理 [`Camera`](https://docs.rs/bevy/0.11.0/bevy/render/camera/struct.Camera.html) 的纹理。一个特别有趣的用例是 XR/VR 支持。已经有几位社区成员[验证了这一点！](https://github.com/bevyengine/bevy/issues/115#issuecomment-1436749201)

## 改进的文本换行 [#](https://bevy.org/news/bevy-0-11/#improved-text-wrapping)

作者：@ickshonpe

以前版本的 Bevy 不能正确地进行文本换行，因为它在计算布局之前就计算了实际的文本。**Bevy 0.11** 增加了一个"文本测量步骤"，在布局之前计算文本大小，然后在布局_之后_计算实际文本。

![文本换行](https://bevy.org/news/bevy-0-11/text_wrap.png)

[`BreakLineOn`](https://docs.rs/bevy/0.11.0/bevy/text/enum.BreakLineOn.html) 设置中还有一个新的 `NoWrap` 变体，可以在需要时完全禁用文本换行。

## 更快的 UI 渲染批处理 [#](https://bevy.org/news/bevy-0-11/#faster-ui-render-batching)

作者：@ickshonpe

我们在某些情况下获得了巨大的 UI 性能提升，通过避免在纹理发生变化但下一个节点没有纹理时拆分 UI 批次。

这是我们的"many buttons"压力测试的分析结果。红色是优化前，黄色是优化后：

![UI 分析](https://bevy.org/news/bevy-0-11/ui_profile.png)

## 更好的反射代理 [#](https://bevy.org/news/bevy-0-11/#better-reflect-proxies)

作者：@MrGVSV

Bevy 的反射 API 有一些结构体，统称为"动态"类型。这些包括 [`DynamicStruct`](https://docs.rs/bevy/0.11.0/bevy/reflect/struct.DynamicStruct.html)、[`DynamicTuple`](https://docs.rs/bevy/0.11.0/bevy/reflect/struct.DynamicTuple.html) 等，它们用于在运行时动态构造任何形状或形式的类型。这些类型也用于创建通常被称为"代理"的东西，即用于表示实际具体类型的动态类型。

这些代理是 [`Reflect::clone_value`](https://docs.rs/bevy/0.11.0/bevy/reflect/trait.Reflect.html#tymethod.clone_value) 方法的底层实现，该方法在底层生成这些代理，以构造数据的运行时克隆。

不幸的是，这导致了一些[微妙的陷阱](https://github.com/bevyengine/bevy/issues/6601)，可能会让用户措手不及，例如代理的哈希值与它们所代表的具体类型的哈希值不同、代理不被认为与其具体对应物等价等。

虽然这个版本不一定修复了这些问题，但它为未来修复这些问题打下了坚实的基础。它通过改变代理的定义方式来实现这一点。

在 0.11 之前，代理仅通过克隆具体类型的 [`Reflect::type_name`](https://docs.rs/bevy/0.11.0/bevy/reflect/trait.Reflect.html#tymethod.type_name) 字符串并将其作为自己的 `Reflect::type_name` 返回来定义。

现在在 0.11 中，代理通过复制对具体类型的静态 [`TypeInfo`](https://docs.rs/bevy/0.11.0/bevy/reflect/enum.TypeInfo.html) 的引用来定义。这将允许我们在不要求 `TypeRegistry` 的情况下，动态地访问更多具体类型的类型信息。在[未来的版本](https://github.com/bevyengine/bevy/pull/8695)中，我们将利用这一点，直接将哈希和比较策略存储在 `TypeInfo` 中，以缓解上述代理问题。

## `FromReflect` 人体工程学改进 [#](https://bevy.org/news/bevy-0-11/#fromreflect-ergonomics)

作者：@MrGVSV

Bevy 的[反射 API](https://docs.rs/bevy_reflect/latest/bevy_reflect/index.html) 通常使用类型擦除的 `dyn Reflect` trait 对象来传递数据。这通常可以使用 `<dyn Reflect>::downcast_ref::<T>` 向下转换回其具体类型；但是，如果底层数据已转换为"动态"表示（例如，结构体类型的 `DynamicStruct`，列表类型的 `DynamicList` 等），则此方法不起作用。

```rust
let data: Vec<i32> = vec![1, 2, 3];

let reflect: &dyn Reflect = &data;
let cloned: Box<dyn Reflect> = reflect.clone_value();

// `reflect` 实际上是 `Vec<i32>`
assert!(reflect.is::<Vec<i32>>());
assert!(reflect.represents::<Vec<i32>>());

// `cloned` 是一个 `DynamicList`，但代表一个 `Vec<i32>`
assert!(cloned.is::<DynamicList>());
assert!(cloned.represents::<Vec<i32>>());

// `cloned` 等价于原始的 `reflect`，尽管不是 `Vec<i32>`
assert!(cloned.reflect_partial_eq(reflect).unwrap_or_default());
```

为了解决这个问题，[`FromReflect`](https://docs.rs/bevy_reflect/latest/bevy_reflect/trait.FromReflect.html) trait 可以将任何 `dyn Reflect` trait 对象转换回其具体类型——无论它实际上是该类型还是其动态表示。而且它甚至可以使用 [`ReflectFromReflect`](https://docs.rs/bevy_reflect/latest/bevy_reflect/struct.ReflectFromReflect.html) 类型数据动态调用。

在 0.11 之前，用户必须为每个需要的类型手动派生 `FromReflect`，并手动注册 `ReflectFromReflect` 类型数据。这使得使用起来很繁琐，并且意味着它经常被遗忘，导致下游用户在反射转换方面遇到困难。

现在在 0.11 中，对于所有派生 `Reflect` 的类型，`FromReflect` 会自动派生，`ReflectFromReflect` 会自动注册。这意味着大多数类型默认都具有 `FromReflect` 能力，从而减少了样板代码，并增强了以 `FromReflect` 为中心的逻辑。

用户仍然可以通过为其类型添加 `#[reflect(from_reflect = false)]` 属性来选择退出此行为。

```rust
#[derive(Reflect)]
struct Foo;

#[derive(Reflect)]
#[reflect(from_reflect = false)]
struct Bar;

fn test<T: FromReflect>(value: T) {}

test(Foo); // <-- 正常！
test(Bar); // <-- 错误！`Bar` 没有实现 `FromReflect` trait
```

## Deref 派生属性 [#](https://bevy.org/news/bevy-0-11/#deref-derive-attribute)

作者：@MrGVSV

Bevy 代码倾向于大量使用[新类型](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)模式，这就是为什么我们有专门的 [`Deref`](https://docs.rs/bevy/latest/bevy/prelude/derive.Deref.html) 和 [`DerefMut`](https://docs.rs/bevy/latest/bevy/prelude/derive.DerefMut.html) 派生宏。

这以前只对单个字段的结构体有效：

```rust
#[derive(Resource, Deref, DerefMut)]
struct Score(i32);
```

对于 0.11，我们改进了这些派生宏，添加了 `#[deref]` 属性，使它们可以在具有多个字段的结构体上使用。这使得处理泛型新类型更加容易：

```rust
#[derive(Component, Deref, DerefMut)]
struct Health<T: Character> {
    #[deref] // <- 使用 `health` 字段作为 `Deref` 和 `DerefMut` 的目标
    health: u16,
    _character_type: PhantomData<T>,
}
```

## 更简单的 RenderGraph 构建 [#](https://bevy.org/news/bevy-0-11/#simpler-rendergraph-construction)

作者：@IceSentry, @cart

向 `RenderGraph` 添加 `Node` 需要大量样板代码。在这个版本中，我们尝试为最常见的操作减少这些代码。没有移除任何现有的 API，这些只是用于简化 `RenderGraph` 使用的辅助方法。

我们向 `App` 添加了 `RenderGraphApp` trait。这个 trait 包含各种辅助函数，用于减少向图中添加节点和边的样板代码。

`RenderGraph` `Node` 的另一个痛点是必须将视图实体传递给每个节点，并在该视图上手动更新查询。为了解决这个问题，我们添加了 `ViewNode` trait 和 `ViewNodeRunner`，它们会自动处理在视图实体上运行 `Query`。我们还使视图实体成为 `RenderGraph` 的一等概念。所以你现在可以从图中的任何位置访问图当前正在处理的视图实体，而无需在每 `Node` 之间传递它。

所有这些新 API 都假设你的 Node 实现了 `FromWorld` 或 `Default`。

以下是 `BloomNode` 的实际使用示例：

```rust
// 将节点添加到 3D 图中
render_app
    // 要运行 ViewNode，你需要创建一个 ViewNodeRunner
    .add_render_graph_node::<ViewNodeRunner<BloomNode>>(
        CORE_3D,
        core_3d::graph::node::BLOOM,
    );

// 定义节点
#[derive(Default)]
struct BloomNode;
// 这可以替换任何现有 `Node` 中操作视图的 `impl Node` 块
impl ViewNode for BloomNode {
    // 你需要将视图查询定义为关联类型
    type ViewQuery = (
        &'static ExtractedCamera,
        &'static ViewTarget,
        &'static BloomSettings,
    );
    // 你不再需要 Node::input() 或 Node::update() 了。如果你仍然需要它们，它们仍然可用，但有空的默认实现。
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        // 这是你查询的结果。如果为空，则不会调用 run 函数
        (camera, view_target, bloom_settings): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        // 使用 ViewNode 时，你可能不需要视图实体，但如果你需要，这是获取它的方法
        let view_entity = graph.view_entity();

        // 运行节点
    }
}
```

## 枚举变体字段上的 `#[reflect(default)]` [#](https://bevy.org/news/bevy-0-11/#reflect-default-on-enum-variant-fields)

作者：@MrGVSV

当使用 `FromReflect` trait 时，标记为 `#[reflect(default)]` 的字段如果不存在于反射对象上，将被设置为其 `Default` 值。

以前，这只在结构体字段上支持。现在，它也支持在所有枚举变体字段上。

```rust
#[derive(Reflect)]
enum MyEnum {
    Data {
        #[reflect(default)]
        a: u32,
        b: u32,
    },
}

let mut data = DynamicStruct::default ();
data.insert("b", 1);

let dynamic_enum = DynamicEnum::new("Data", data);

let my_enum = MyEnum::from_reflect( & dynamic_enum).unwrap();
assert_eq!(u32::default(), my_enum.a);
```

## 延迟的资源热重载 [#](https://bevy.org/news/bevy-0-11/#delayed-asset-hot-reloading)

作者：@JMS55

Bevy 现在在"文件系统上的资源已更改"事件发生后等待 50 毫秒才重新加载资源。没有延迟的重载会导致在某些系统上读取到无效的资源内容。等待时间是可配置的。

## 自定义 glTF 顶点属性 [#](https://bevy.org/news/bevy-0-11/#custom-gltf-vertex-attributes)

作者：@komadori

现在可以从 glTF 文件中加载带有自定义顶点属性的网格。自定义属性可以映射到 Bevy 的 [`MeshVertexAttribute`](https://docs.rs/bevy/0.11.0/bevy/render/mesh/struct.MeshVertexAttribute.html) 格式，该格式在 [`GltfPlugin`](https://docs.rs/bevy/0.11.0/bevy/gltf/struct.GltfPlugin.html) 设置中供 [`Mesh`](https://docs.rs/bevy/0.11.0/bevy/render/mesh/struct.Mesh.html) 类型使用。这些属性随后可以在 Bevy 着色器中使用。示例请查看我们的[新示例](https://github.com/bevyengine/bevy/blob/v0.11.0/examples/2d/custom_gltf_vertex_attribute.rs)。

![自定义顶点属性](https://bevy.org/news/bevy-0-11/custom_vertex.png)

## 稳定的 TypePath [#](https://bevy.org/news/bevy-0-11/#stable-typepath)

作者：@soqb, @tguichaoua

Bevy 历史上一直在许多地方使用 [`std::any::type_name`](https://doc.rust-lang.org/std/any/fn.type_name.html) 来用友好的名称标识 Rust 类型：Bevy Reflect、Bevy Scenes、Bevy Assets、Bevy ECS 等。不幸的是，Rust 对 [`type_name`](https://doc.rust-lang.org/std/any/fn.type_name.html) 的稳定性或格式不作任何保证，这使得在理论上建立在其之上是摇摇欲坠的（尽管在实践中它一直很稳定）。

也没有内置的方法来检索类型名称的"部分"。如果你想要短名称、不带内部类型的泛型名称、模块名称或 crate 名称，你必须对 [`type_name`](https://doc.rust-lang.org/std/any/fn.type_name.html) 进行字符串操作（这可能容易出错/不简单）。

此外，[`type_name`](https://doc.rust-lang.org/std/any/fn.type_name.html) 不能被自定义。在某些情况下，作者可能选择用其完整模块路径以外的内容来标识一个类型（例如，如果他们更喜欢较短的路径，或者想抽象掉私有/内部模块）。

由于这些原因，我们开发了一个新的稳定版 [`TypePath`](https://docs.rs/bevy/0.11.0/bevy/reflect/trait.TypePath.html)，它自动为任何派生 [`Reflect`](https://docs.rs/bevy/0.11.0/bevy/reflect/trait.Reflect.html) 的类型实现。此外，在未派生 [`Reflect`](https://docs.rs/bevy/0.11.0/bevy/reflect/trait.Reflect.html) 的情况下，也可以手动派生它。

```rust
mod my_mod {
    #[derive(Reflect)]
    struct MyType;
}

/// 输出："my_crate::my_mod::MyType"
println!("{}", MyType::type_path());
/// 输出："MyType"
println!("{}", MyType::short_type_path());
/// 输出："my_crate"
println!("{}", MyType::crate_name().unwrap());
/// 输出："my_crate::my_mod"
println!("{}", MyType::module_path().unwrap());
```

这也适用于泛型，有时会很有用：

```rust
// 输出："Option<MyType>"
println!("{}", Option::<MyType>::short_type_path());
// 输出："Option"
println!("{}", Option::<MyType>::type_ident().unwrap());
```

[`TypePath`](https://docs.rs/bevy/0.11.0/bevy/reflect/trait.TypePath.html) 可以被类型作者自定义：

```rust
#[derive(TypePath)]
#[type_path = "some_crate::some_module"]
struct MyType;
```

我们正在将 Bevy 内部的 [`type_name`](https://doc.rust-lang.org/std/any/fn.type_name.html) 使用移植到 [`TypePath`](https://docs.rs/bevy/0.11.0/bevy/reflect/trait.TypePath.html)，这应该在 **Bevy 0.12** 中完成。

## 系统元组的 `run_if` [#](https://bevy.org/news/bevy-0-11/#run-if-for-tuples-of-systems)

作者：@geieredgar

现在可以为系统元组添加["运行条件"](https://bevy.org/news/bevy-0-10/#run-conditions)：

```rust
app.add_systems(Update, (run, jump).run_if(in_state(GameState::Playing)))
```

这将精确评估"运行条件"一次，并将结果用于元组中的每个系统。

这使我们能够移除状态的 `OnUpdate` system set（以前用于在给定状态下运行系统组）。

## `Has` 查询 [#](https://bevy.org/news/bevy-0-11/#has-queries)

作者：@wainwrightmark

你现在可以在查询中使用 `Has<Component>`，如果组件存在则返回 true，不存在则返回 false：

```rust
fn system(query: Query<Has<Player>>) {
    for has_player in &query {
        if has_player {
            // 做某事
        }
    }
}
```

## 派生 `Event` [#](https://bevy.org/news/bevy-0-11/#derive-event)

作者：@CatThingy

Bevy 的 [`Event`](https://docs.rs/bevy/0.11.0/bevy/ecs/event/trait.Event.html) trait 现在需要派生，而不是为所有内容自动实现：

```rust
#[derive(Event)]
struct Collision {
    a: Entity,
    b: Entity,
}
```

这可以防止某些类别的错误，使 [`Event`](https://docs.rs/bevy/0.11.0/bevy/ecs/event/trait.Event.html) 类型更具自文档性，并与 Bevy ECS 的其他 trait（如 Components 和 Resources）保持一致。它还为配置 [`Event`](https://docs.rs/bevy/0.11.0/bevy/ecs/event/trait.Event.html) 存储类型打开了大门，我们计划在未来的版本中实现。

## 三次曲线示例 [#](https://bevy.org/news/bevy-0-11/#cubic-curve-example)

作者：@Kjolnyr

一个展示如何绘制 3D 曲线并沿路径移动物体的示例：

![三次曲线](https://bevy.org/news/bevy-0-11/cubic_curve.png)

## 尺寸约束示例 [#](https://bevy.org/news/bevy-0-11/#size-constraints-example)

作者：@ickshonpe

一个交互式示例，展示各种 [`Style`](https://docs.rs/bevy/0.11.0/bevy/ui/struct.Style.html) 尺寸约束如何影响 UI 节点。

![尺寸约束](https://bevy.org/news/bevy-0-11/size_constraints.png)

## 显示与可见性示例 [#](https://bevy.org/news/bevy-0-11/#display-and-visibility-example)

作者：@ickshonpe

一个展示显示和可见性设置如何影响 UI 节点的示例。

![显示与可见性](https://bevy.org/news/bevy-0-11/display_and_visibility.png)

## 不再使用 Bors！ [#](https://bevy.org/news/bevy-0-11/#no-more-bors)

作者：@cart, @mockersf

Bevy 历史上使用 Bors 合并系统来确保我们永远不会合并一个破坏 CI 验证的 GitHub pull request。这是一个关键的基础设施，确保我们可以安全有效地协作。幸运的是，GitHub _终于_推出了[Merge Queues](https://github.blog/changelog/2023-02-08-pull-request-merge-queue-public-beta/)，它解决了与 Bors 相同的问题，并且与 GitHub 的集成更加紧密。

在这个发布周期中，我们迁移到了 Merge Queues，我们对使用体验非常满意！

## 新的 CI 任务 [#](https://bevy.org/news/bevy-0-11/#new-ci-jobs)

作者：@mockersf

我们添加了一些新的 CI 任务，改善 Bevy 的开发体验：

- 一个每日任务，在真实的 Android 和 iOS 设备上运行 Bevy 的移动示例！这有助于防止编译器可能无法捕获的回归问题
- 添加了在 CI 中截图的能力，可用于验证 Bevy 示例运行的结果
- 一个在缺少功能或示例文档更新的 PR 上留下 GitHub 评论的任务

## 下一步是什么？ [#](https://bevy.org/news/bevy-0-11/#what-s-next)

我们有很多已经基本完成的工作，因此很可能会在 **Bevy 0.12** 中发布：

- **Bevy Asset V2**：一个全新的资源系统，增加了"资源预处理"、可选的资源 .meta 文件、递归资源依赖跟踪和事件、异步资源 IO、更好的资源句柄、更高效的资源存储以及各种可用性改进！这项工作[已经基本完成](https://github.com/bevyengine/bevy/pull/8624)。它_差点_就进入了 Bevy 0.11，但需要更多时间完善。
- **PBR 材质光透射**：透射/屏幕空间折射允许模拟玻璃、塑料、液体和凝胶、宝石、蜡等材质。这个也[基本就绪了](https://github.com/bevyengine/bevy/pull/8015)！
- **TAA 改进**：我们正在为 TAA 进行一些改进，将提升其质量、速度和引擎内的支持。
- **GPU 拾取**：通过使用颜色 ID 在渲染中识别网格，在 GPU 上[高效准确地选择实体](https://github.com/bevyengine/bevy/pull/8784)。
- **定向光和聚光阴影的 PCF**：[减少阴影边缘的锯齿](https://github.com/bevyengine/bevy/pull/8006)
- **UI 节点边框圆角和阴影**：为你的 UI 节点添加[曲率和"投影"](https://github.com/bevyengine/bevy/pull/8973)！
- **延迟渲染**：Bevy 已经通过可选的独立深度和法线通道实现了"混合模式"前向渲染。我们目前也在实验支持"完全延迟"渲染，这为新的效果和不同的性能权衡打开了大门。

从高层次来看，我们计划在下一个周期专注于资源系统、UI、渲染功能和场景。

查看 [**Bevy 0.12 Milestone**](https://github.com/bevyengine/bevy/milestone/14) 获取正在考虑用于 **Bevy 0.12** 的最新工作列表。
