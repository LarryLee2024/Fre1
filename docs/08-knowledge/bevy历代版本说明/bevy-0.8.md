# Bevy 0.8

## 发布于 2022 年 7 月 30 日，作者 Carter Anderson ( ![一个带猫耳挥舞触手的剪影，即 Octocat：GitHub 的吉祥物和标志](https://bevy.org/assets/github_grey.svg) [@cart](https://www.github.com/cart) ![一个带圆角矩形的右指三角形；YouTube 的标志](https://bevy.org/assets/youtube_grey.svg) [cartdev](https://www.youtube.com/cartdev) )

![由 rmemr 构建的使用 Bevy 的 Witcher 3 地形纹理工具中的 Bevy 形山](https://bevy.org/news/bevy-0-8/bevy_terrain.jpg)

[由 rmemr 构建的使用 Bevy 的 Witcher 3 地形纹理工具中的 Bevy 形山](https://codeberg.org/rmemr/w3.terrain-texturing)

感谢 **130** 位贡献者、**461** 个拉取请求、社区审查者以及我们的[**慷慨赞助商**](https://github.com/sponsors/cart），我很高兴地在 [crates.io](https://crates.io/crates/bevy) 上宣布 **Bevy 0.8** 发布！

对于那些还不知道的人，Bevy 是一个用 Rust 构建的、令人耳目一新的简单数据驱动游戏引擎。你可以查看我们的[快速入门指南](https://bevy.org/learn/quick-start/introduction)立即尝试。它是永久免费和开源的！你可以在 GitHub 上获取完整的[源代码](https://github.com/bevyengine/bevy)。查看 [Bevy Assets](https://bevy.org/assets) 可以找到社区开发的插件、游戏和学习资源合集。

要将现有的 Bevy 应用或插件更新到 **Bevy 0.8**，请查看我们的 [0.7 到 0.8 迁移指南](https://bevy.org/learn/migration-guides/0.7-0.8/)。

自几个月前的上次发布以来，我们添加了大量的新功能、错误修复和生活质量优化，以下是一些亮点：

- **新材质系统**：由于新的 Material trait 和 AsBindGroup 派生宏，自定义着色器现在**容易得多**。
- **相机驱动渲染**：每个相机现在配置它渲染什么以及如何渲染。轻松地将相机渲染层叠在一起，进行分屏操作，或仅用几行代码渲染到纹理。
- **内置着色器模块化**：许多内置着色器类型和函数现在可以导入。值得注意的是，自定义着色器现在可以导入 PBR 着色器逻辑。
- **聚光灯**：一种新灯光类型，从固定点以锥形发射光线。
- **可见性继承**：隐藏一个实体现在也会隐藏层级中其所有后代。
- **升级到 wgpu 0.13**：使用新的、更符合人体工程学的 WGSL 着色器语法。
- **自动网格切线生成**：如果网格缺少切线，使用 mikktspace 生成它们。
- **渲染器优化**：并行视锥体裁剪和非稳定排序用于非批处理渲染阶段，带来了巨大收益！
- **场景 Bundle**：使用普通的 Bevy bundle 轻松生成场景，并用新的组件和子节点扩展它们。
- **脚本和 Modding 进度：无类型 ECS API**：向第三方脚本语言支持迈出一步！通过指针直接与 Bevy ECS 内部交互。
- **ECS 查询人体工程学和可用性**：查询现在实现了 `IntoIter`，可变查询可以转换为不可变查询。
- **ECS 内部重构**：对 Bevy ECS 内部的彻底更改，使其更简单、更安全、更易于维护。
- **反射改进**：支持反射更多类型、ECS 资源反射、无类型反射、改进的内部实现。
- **层级命令**：层级更新现在使用"事务命令"以确保层级始终一致。
- **Bevy UI 现在使用 Taffy**：我们已切换到一个协作 fork 的（并帮助维护）现已废弃的 Stretch UI 布局库。指数级膨胀的 Bug 终于消失了！

## 新材质系统 [#](https://bevy.org/news/bevy-0-8/#new-material-system)

作者：@cart, @Wrapperup, @johanhelsing

Bevy 有一个全新的 [`Material`](https://docs.rs/bevy/0.8.0/bevy/pbr/trait.Material.html) 系统，使定义自定义着色器变得轻而易举。Bevy 之前的材质系统需要数百行"中层"样板代码。这从来都不是长期计划，只是一个中间步骤。在 **Bevy 0.8** 中，自定义着色器材质简单如下：

```rust
#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct CoolMaterial {
    #[uniform(0)]
    color: Color,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Handle<Image>,
}

// Material trait 有许多可选函数用于配置。
// 在这种情况下，我们需要设置的只是一个片段着色器。人体工程学！
impl Material for CoolMaterial {
    fn fragment_shader() -> ShaderRef {
        "cool_material.wgsl".into()
    }
}
```

以及 `cool_material.wgsl` 着色器：

```rust
struct CoolMaterial {
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: CoolMaterial;
@group(1) @binding(1)
var color_texture: texture_2d<f32>;
@group(1) @binding(2)
var color_sampler: sampler;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    return material.color * textureSample(color_texture, color_sampler, uv);
}
```

就这样，我们有了一个可配置的着色器材质！

这可以用来创建你想要的任何效果！例如：`@DGriffin91` 制作了这个很酷的"发光球体"效果：

他们还制作了一个[很好的视频教程](https://www.youtube.com/watch?v=O6A_nVmpvhc)，概述了如何创建这种材质。

这要归功于新的 [`AsBindGroup`](https://docs.rs/bevy/0.8.0/bevy/render/render_resource/trait.AsBindGroup.html) trait/派生宏，它完成了将材质转换为 GPU 兼容数据类型、将其写入 GPU 以及创建最终 [`BindGroup`](https://docs.rs/bevy/0.8.0/bevy/render/render_resource/struct.BindGroup.html) 的所有艰苦工作。[`AsBindGroup`](https://docs.rs/bevy/0.8.0/bevy/render/render_resource/trait.AsBindGroup.html) trait 非常强大：它支持将多个字段组合到同一个统一绑定中，配置纹理绑定类型（2D、3D、滤镜等），以及更多功能。详情请查看 [`AsBindGroup`](https://docs.rs/bevy/0.8.0/bevy/render/render_resource/trait.AsBindGroup.html) 文档。

所有内置材质，例如 PBR [`StandardMaterial`](https://docs.rs/bevy/0.8.0/bevy/pbr/struct.StandardMaterial.html)，都已移植到这个新系统。我们努力使 Bevy 的"内部"与用户代码保持一致，这也不例外！[`Material`](https://docs.rs/bevy/0.8.0/bevy/pbr/trait.Material.html) 也可以与我们更高级的着色器功能无缝协作，例如[着色器管线特化](https://github.com/bevyengine/bevy/blob/v0.8.0/examples/shader/shader_defs.rs)。

还有一个等效的 [`Material2d`](https://docs.rs/bevy/0.8.0/bevy/sprite/trait.Material2d.html) trait，它为 2D 启用自定义材质。

## 相机驱动渲染 [#](https://bevy.org/news/bevy-0-8/#camera-driven-rendering)

作者：@cart

在以前的 Bevy 版本中，[`Cameras`](https://docs.rs/bevy/0.8.0/bevy/render/camera/struct.Camera.html) 是作为一个"全局" [`RenderGraph`](https://docs.rs/bevy/0.8.0/bevy/render/render_graph/struct.RenderGraph.html) 的一部分被选择和运行的。每种类型只能有一个"活动"相机，且该相机只能渲染到一个目标。从多个视角同时渲染的唯一方法是手动使用重复逻辑扩展渲染图。这充满了复杂的底层渲染器样板代码，对普通 Bevy 用户来说不可接近。

在 **Bevy 0.8** 中，每个 [`Camera`](https://docs.rs/bevy/0.8.0/bevy/render/camera/struct.Camera.html) 现在配置它渲染什么、如何渲染以及渲染到什么。一个启用的相机将启动一个新的 [`RenderGraph`](https://docs.rs/bevy/0.8.0/bevy/render/render_graph/struct.RenderGraph.html) 运行到指定的 [`RenderTarget`](https://docs.rs/bevy/0.8.0/bevy/render/camera/enum.RenderTarget.html)。渲染图定义了给定相机的[模块化渲染逻辑](https://bevy.org/news/bevy-0-6/#render-graphs-and-sub-graphs)，而渲染目标定义了图将要渲染到的窗口或纹理。

这使得以前需要数百（或数千）行代码的场景变得像在 [`Camera`](https://docs.rs/bevy/0.8.0/bevy/render/camera/struct.Camera.html) 实体上设置几个字段一样简单：

### 渲染到纹理 [#](https://bevy.org/news/bevy-0-8/#render-to-textures)

将 [`Camera`](https://docs.rs/bevy/0.8.0/bevy/render/camera/struct.Camera.html) 渲染到纹理现在是一行代码：

```rust
camera.target = RenderTarget::Image(image_handle);
```

这开启了数不清的场景：传送门、将 UI 渲染到纹理并在 3d 空间中渲染它、游戏内安全摄像头、在 UI 小部件中渲染的实时玩家头像……天空才是极限！

这是一个"传送门效果"的示例：

这是通过将第二个相机渲染到纹理，使其朝向与主玩家的相机同步，并在主相机的场景中使用该纹理来实现的。

### 分屏 [#](https://bevy.org/news/bevy-0-8/#split-screen)

每个 [`Camera`](https://docs.rs/bevy/0.8.0/bevy/render/camera/struct.Camera.html) 现在有一个可选的 [`Viewport`](https://docs.rs/bevy/0.8.0/bevy/render/camera/struct.Viewport.html)，如果设置，将绘制到 [`RenderTarget`](https://docs.rs/bevy/0.8.0/bevy/render/camera/enum.RenderTarget.html) 的一部分而不是整个。如果你生成两个活动相机，并将每个相机的 [`Viewport`](https://docs.rs/bevy/0.8.0/bevy/render/camera/struct.Viewport.html) 设置为绘制到窗口的一半，你就拥有了简单、无痛的分屏！

![分屏](https://bevy.org/news/bevy-0-8/split_screen.jpg)

### 分层渲染 [#](https://bevy.org/news/bevy-0-8/#layered-rendering)

相机现在可以使用新的"相机优先级"字段相互层叠：

```rust
// 这个相机默认为优先级 0，被渲染为"第一个"/"在后面"
commands.spawn_bundle(Camera3dBundle::default());
commands.spawn_bundle(Camera3dBundle {
    camera_3d: Camera3d {
        // 渲染此相机时不清除颜色
        clear_color: ClearColorConfig::None,
        ..default()
    },
    camera: Camera {
        // 在主相机之后/之上渲染
        priority: 1,
        ..default()
    },
    ..default()
});
```

"优先级"决定了相机绘制顺序。上面[渲染到纹理示例](https://bevy.org/news/bevy-0-8/#render-to-textures)中的"传送门"效果使用优先级先渲染"传送门"相机，确保它准备好供主相机使用。

这是一个将两个相机渲染到同一窗口的简单示例：

![相机分层](https://bevy.org/news/bevy-0-8/camera_layering.png)

这可以用于诸如"自定义 UI 通道"、"小地图"等场景。

### 符合人体工程学的目标大小访问 [#](https://bevy.org/news/bevy-0-8/#ergonomic-target-size-access)

相机现在本地存储它们的 [`RenderTarget`](https://docs.rs/bevy/0.8.0/bevy/render/camera/enum.RenderTarget.html) 大小，这使得获取大小更加简单：

```rust
// 比在目标 Window 或 Image 上手动查找大小好多了，
// 就像在以前的 Bevy 版本中必须做的那样！
let target_size = camera.logical_target_size();
let viewport_size = camera.logical_viewport_size();
```

这也意味着将相机世界坐标转换为屏幕坐标比以前容易得多：

```rust
// Bevy 0.7（旧的）
camera.world_to_screen(windows, images, camera_transform, world_position);

// Bevy 0.8（新的）
camera.world_to_viewport(camera_transform, world_position);
```

### 新的相机 Bundle [#](https://bevy.org/news/bevy-0-8/#new-camera-bundles)

旧的 `OrthographicCameraBundle` 和 `PerspectiveCameraBundle` 已被 [`Camera3dBundle`](https://docs.rs/bevy/0.8.0/bevy/core_pipeline/core_3d/struct.Camera3dBundle.html) 和 [`Camera2dBundle`](https://docs.rs/bevy/0.8.0/bevy/core_pipeline/core_2d/struct.Camera2dBundle.html) 取代。在大多数情况下，迁移只需要将旧名称替换为新名称。3D 相机默认使用"透视"投影，但它们仍然可以使用 bundle 中新的 [`Projection`](https://docs.rs/bevy/0.8.0/bevy/core_pipeline/core_3d/struct.Camera3dBundle.html#structfield.projection) 组件切换到正交投影。

### 不再有 CameraUiBundle！ [#](https://bevy.org/news/bevy-0-8/#no-more-camerauibundle)

Bevy UI 现在不再需要单独的相机实体来工作。UI 对所有相机类型都"开箱即用"，并且可以使用 [`UiCameraConfig`](https://docs.rs/bevy/0.8.0/bevy/ui/entity/struct.UiCameraConfig.html) 组件按相机启用或禁用。

```rust
commands
    .spawn_bundle(Camera3dBundle::default())
    .insert(UiCameraConfig {
        show_ui: false,
    });
```

### 自定义渲染图 [#](https://bevy.org/news/bevy-0-8/#custom-render-graphs)

每个 [`Camera`](https://docs.rs/bevy/0.8.0/bevy/render/camera/struct.Camera.html) 的默认 2D 和 3D [`RenderGraphs`](https://docs.rs/bevy/0.8.0/bevy/render/render_graph/struct.RenderGraph.html) 可以通过设置 [`CameraRenderGraph`](https://docs.rs/bevy/0.8.0/bevy/render/camera/struct.CameraRenderGraph.html) 组件来覆盖：

```rust
commands.spawn_bundle(Camera3dBundle {
    camera_render_graph: CameraRenderGraph::new(some_custom_graph),
    ..default()
})
```

这使得你可以使用你需要的任何自定义渲染逻辑来绘制相机！例如，你可以用延迟渲染替换内置的分簇前向渲染。请注意，这通常不是必需的：大多数自定义渲染场景将通过高级[材质](https://bevy.org/news/bevy-0-8/#new-material-system)或扩展内置渲染图来覆盖。而使用默认渲染图将确保与其他插件的最大兼容性。

### 启用/禁用相机 [#](https://bevy.org/news/bevy-0-8/#enabling-disabling-cameras)

如果相机是"活动的"，它将渲染到它的 [`RenderTarget`](https://docs.rs/bevy/0.8.0/bevy/render/camera/enum.RenderTarget.html)。要激活或停用相机，设置新的 `is_active` 字段：

```rust
camera.is_active = true;
```

### RenderLayers [#](https://bevy.org/news/bevy-0-8/#renderlayers)

Bevy 现有的 [`RenderLayers`](https://docs.rs/bevy/0.8.0/bevy/render/view/visibility/struct.RenderLayers.html) 系统可以用来告诉一个 [`Camera`](https://docs.rs/bevy/0.8.0/bevy/render/camera/struct.Camera.html) 只渲染特定层上的实体。**相机驱动渲染**与此功能配合良好。这允许将一组实体渲染到一个相机，另一组实体渲染到另一个相机。我们已经将 [`RenderLayers`](https://docs.rs/bevy/0.8.0/bevy/render/view/visibility/struct.RenderLayers.html) 系统移植到所有带有 [`Visibility`](https://docs.rs/bevy/0.8.0/bevy/render/view/struct.Visibility.html) 的实体上，所以这一切都能"开箱即用"。

## 聚光灯 [#](https://bevy.org/news/bevy-0-8/#spotlights)

作者：@robtfm

Bevy 现在有一个 [`SpotLight`](https://docs.rs/bevy/0.8.0/bevy/pbr/struct.SpotLight.html) 实体类型，它从空间中的一个点以锥形发射光线。

![聚光灯](https://bevy.org/news/bevy-0-8/spotlight.png)

## 可见性继承 [#](https://bevy.org/news/bevy-0-8/#visibility-inheritance)

作者：@james7132, @cart

实体层级中的可见性（使用 [`Visibility`](https://docs.rs/bevy/0.8.0/bevy/render/view/struct.Visibility.html) 组件）现在会沿着层级向下传播。这非常有用，因为游戏中的实体通常在其下面嵌套了**许多**实体。一个"玩家实体"通常由许多部分组成：玩家精灵或网格、玩家穿的/拿着的、视觉效果等等。

可见性继承意味着你只需要在代码中隐藏顶层的"玩家"实体，其下方的所有内容都会自动隐藏。

这个"飞行头盔"场景由主头盔实体下的许多"部件"嵌套组成。现在隐藏所有这些"子实体"就像隐藏顶层头盔实体一样容易。

```rust
fn hide_helmets(mut helmet_visibilities: Query<&mut Visibility, With<Helmet>>) {
    let mut helmet_visibility = helmet_visibilities.single_mut();
    helmet_visibility.is_visible = false;
}
```

在以前的 Bevy 版本中，你必须手动隐藏每个部件！

"继承的可见性"在 [`PostUpdate`](https://docs.rs/bevy/0.8.0/bevy/app/enum.CoreStage.html#variant.PostUpdate) 阶段计算，并存储在 [`ComputedVisibility`](https://docs.rs/bevy/0.8.0/bevy/render/view/struct.ComputedVisibility.html) 组件上。[`ComputedVisibility`](https://docs.rs/bevy/0.8.0/bevy/render/view/struct.ComputedVisibility.html) 现在有以下函数：

- `is_visible_in_hierarchy()`：实体根据"可见性继承"是否可见。
- `is_visible_in_view()`：实体在任何视图中是否可见。用于在"视锥体裁剪"等情况下剔除实体。
- `is_visible()`：确定实体是否会被绘制的规范方法。结合了"视图可见性"和"层级可见性"。

### SpatialBundle 和 VisibilityBundle [#](https://bevy.org/news/bevy-0-8/#spatialbundle-and-visibilitybundle)

作者：@mockersf, @rparrett

随着[可见性继承](https://bevy.org/news/bevy-0-8/#visibility-inheritance)的加入，可见性传播要求层级中的所有元素都具有适当的可见性组件。在构建场景时，开发者通常希望将实体分组到父级"组织性"实体下，这些实体仅用于将实体分组、重新定位它们以及整体隐藏它们。这些"组织性"实体仍然需要可见性组件来将 [`Transform`](https://docs.rs/bevy/0.8.0/bevy/transform/components/struct.Transform.html) 和 [`Visibility`](https://docs.rs/bevy/0.8.0/bevy/render/view/struct.Visibility.html) 传播到 [`GlobalTransform`](https://docs.rs/bevy/0.8.0/bevy/transform/components/struct.GlobalTransform.html) 和 [`ComputedVisibility`](https://docs.rs/bevy/0.8.0/bevy/render/view/struct.ComputedVisibility.html)。

为了简化这一点，我们添加了一个新的 [`SpatialBundle`](https://docs.rs/bevy/0.8.0/bevy/render/prelude/struct.SpatialBundle.html)，它添加了上述组件。这允许实体配置和传播可见性和变换数据，而无需承担实际渲染它的开销。

```rust
commands
    // 此实体控制其下实体的位置和可见性。
    .spawn_bundle(SpatialBundle {
        transform: Transform::from_xyz(10.0, 20.0, 30.0),
        visibility: Visibility {
            is_visible: true,
        },
        ..default()
    }).with_children(|parent| {
        parent
            .spawn_bundle(TableBundle::default())
            .spawn_bundle(ShopKeeperBundle::default())
            .spawn_bundle(PotionBundle::default());
    });
```

为确保可见性和变换被传播，请确保整个层级（根到叶子）都有这些组件：

```rust
commands
    .spawn_bundle(SpatialBundle::default())
    .with_children(|parent| {
        parent
            .spawn_bundle(SpatialBundle::default())
            .with_children(|parent| {
                parent.spawn_bundle(SpatialBundle::default());
            });
    });
```

如果你知道你不需要 [`Transform`](https://docs.rs/bevy/0.8.0/bevy/transform/components/struct.Transform.html) 传播（或者你的实体已经拥有这些组件），你可以改用新的 [`VisibilityBundle`](https://docs.rs/bevy/0.8.0/bevy/render/view/struct.VisibilityBundle.html)，它只添加可见性传播所需的组件。

## 内置着色器模块化 [#](https://bevy.org/news/bevy-0-8/#built-in-shader-modularization)

作者：Rob Swain (@superdump)

在 **Bevy 0.8** 中，我们开始模块化我们的内置着色器。值得注意的是，这意味着你现在可以导入并运行内置的 PBR 着色器/光照逻辑：

```rust
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::pbr_types
#import bevy_pbr::pbr_functions

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var pbr_input: PbrInput = pbr_input_new();
    // 将基色设置为红色
    pbr_input.material.base_color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    /* 在此设置其他 PbrInput 字段 */
    return tone_mapping(pbr(pbr_input))
}
```

我们还模块化了网格和视图绑定着色器逻辑。当与[新材质系统](https://bevy.org/news/bevy-0-8/#new-material-system)结合使用时，用户现在可以合理地编写自定义 PBR 材质，而无需在其着色器中重新定义所有 PBR 逻辑。新的[数组纹理示例](https://github.com/bevyengine/bevy/blob/v0.8.0/examples/shader/array_texture.rs)说明了如何使用这些新 API 定义自定义 PBR 着色器材质：

![数组纹理](https://bevy.org/news/bevy-0-8/array_texture.png)

我们还计划在这里发展用户体验。既然我们已经把一切都分解并模块化了，我们将致力于减少扩展此逻辑所需的样板代码量（减少导入，消除设置所有 PbrInput 字段的需要等）。

## wgpu 0.13：新的 WGSL 着色器语法 [#](https://bevy.org/news/bevy-0-8/#wgpu-0-13-new-wgsl-shader-syntax)

作者：@mockersf, wgpu 和 naga 贡献者

我们已经更新到最新最好的 wgpu 版本。wgpu 0.13 带来了大量修复和改进，但最明显的变化是新的更加符合人体工程学的 WGSL"属性"语法：

绑定现在看起来像这样：

```rust
// wgpu 0.12（旧的）
[[group(1), binding(0)]]
var<uniform> material: Material;

// wgpu 0.13（新的）
@group(1) @binding(0)
var<uniform> material: Material;
```

着色器阶段入口点和输入现在看起来像这样：

```rust
// wgpu 0.12（旧的）
[[stage(vertex)]]
fn vertex([[location(0)]] position: vec3<f32>, [[location(1)]] normal: vec3<f32>) -> VertexOutput {
}

// wgpu 0.13（新的）
@vertex
fn vertex(@location(0) position: vec3<f32>, @location(1) normal: vec3<f32>) -> VertexOutput {
}
```

眼睛舒服多了！（手指也是！）

## 场景 Bundle [#](https://bevy.org/news/bevy-0-8/#scene-bundle)

作者：@mockersf

在以前的 Bevy 版本中，场景是这样生成的：

```rust
commands.spawn_scene(asset_server.load("some.gltf#Scene0"));
```

这虽然能用，但在实践中很难真正使用场景。重新定位场景需要要么手动获取场景的实体 ID，然后在它生成时重新定位它，要么创建一个带有相关变换组件的父实体并"将场景生成为子节点"。此外，用新组件扩展场景也很有挑战性。

在 **Bevy 0.8** 中，我们添加了一个 [`SceneBundle`](https://docs.rs/bevy/0.8.0/bevy/scene/struct.SceneBundle.html)，将场景生成与我们其他的实体构建 API 保持一致：

```rust
commands.spawn_bundle(SceneBundle {
    scene: asset_server.load("some.gltf#Scene0"),
    ..default()
})
```

当场景加载时，它会自动生成在由 `spawn_bundle` 命令创建的实体之下。这使得变换场景和在"根"添加新组件变得容易得多：

```rust
commands
    .spawn_bundle(SceneBundle {
        scene: asset_server.load("player.gltf#Scene0"),
        transform: Transform::from_xyz(10.0, 20.0, 30.0),
        ..default()
    })
    .insert(Player::default());
```

## 并行视锥体裁剪 [#](https://bevy.org/news/bevy-0-8/#parallel-frustum-culling)

作者：@aevyrie

Bevy 使用["视锥体裁剪"](https://bevy.org/news/bevy-0-6/#visibility-and-frustum-culling)来跳过绘制相机视野之外的实体。在 **Bevy 0.8** 中，视锥体裁剪现在以并行方式完成。当剔除数千个实体时，这带来了显著的性能提升：

### 视锥体裁剪系统时间 vs 剔除实体数量（越低越好） [#](https://bevy.org/news/bevy-0-8/#frustum-culling-system-time-vs-number-of-entities-culled-lower-is-better)

![并行视锥体裁剪](https://bevy.org/news/bevy-0-8/parallel_frustum_culling.png)

注意"parallel b"代表"并行批处理大小"（每批中的实体数量）。我们在 **Bevy 0.8** 中选择了 1024 作为批处理大小，因为它的性能更好。

## 自动网格切线生成 [#](https://bevy.org/news/bevy-0-8/#automatic-mesh-tangent-generation)

作者：Rob Swain (@superdump), @jakobhellermann, @DJMcNab

顶点切线与法线贴图一起使用，在渲染时赋予网格更详细的法线。一些导入的网格有法线贴图，但没有计算顶点切线。Bevy 现在可以根据事实上的行业标准 MikkTSpace 库/算法（Godot、Unity、Unreal 和 Blender 都使用这个）自动为缺少切线的 [`Mesh`](https://docs.rs/bevy/0.8.0/bevy/render/mesh/struct.Mesh.html) 生成顶点切线。

我们已经开始维护[我们自己的 fork](https://github.com/bevyengine/bevy/tree/v0.8.0/crates/bevy_mikktspace) 的 [gltf-rs/mikktspace crate](https://github.com/gltf-rs/mikktspace)，这样我们可以：

- 以 Bevy 所需的速度更新依赖项；
- [开始限制 unsafe 代码](https://github.com/bevyengine/bevy/pull/4932)，因为它目前使用从原始的用 C 编写的 `mikktspace.h` 自动生成的 unsafe Rust 代码。

## 默认使用线性纹理过滤 [#](https://bevy.org/news/bevy-0-8/#default-to-linear-texture-filtering)

作者：@aevyrie, @cart

Bevy 中的图像现在默认使用线性纹理过滤，这与其他游戏开发生态系统更加一致（Unity 和 Godot 都默认使用过滤纹理）。

这意味着需要未过滤像素的纹理（例如"像素艺术"精灵）必须覆盖此默认设置，可以按图像设置：

```rust
image.sampler_descriptor = ImageSampler::nearest();
```

或全局使用新的 [`ImageSettings`](https://docs.rs/bevy/0.8.0/bevy/render/texture/struct.ImageSettings.html) 资源：

```rust
app.insert_resource(ImageSettings::default_nearest())
```

这样，我们就能得到清晰的像素艺术：

![精灵](https://bevy.org/news/bevy-0-8/sprite.png)

## 新的 GlobalTransform 矩阵表示 [#](https://bevy.org/news/bevy-0-8/#new-globaltransform-matrix-representation)

作者：@HackerFoo

[`GlobalTransform`](https://docs.rs/bevy/0.8.0/bevy/transform/components/struct.GlobalTransform.html) 组件（代表实体的"世界空间"变换）的内部表示已从"相似变换"（平移 [`Vec3`](https://docs.rs/bevy/0.8.0/bevy/math/struct.Vec3.html) / 旋转 [`Quat`](https://docs.rs/bevy/0.8.0/bevy/math/struct.Quat.html) / 缩放 [`Vec3`](https://docs.rs/bevy/0.8.0/bevy/math/struct.Vec3.html)）改为"仿射 3D 变换"（[`Mat3A`](https://docs.rs/bevy/0.8.0/bevy/math/struct.Mat3A.html) 和一个 [`Vec3`](https://docs.rs/bevy/0.8.0/bevy/math/struct.Vec3.html) 平移）。

值得注意的是，这允许表示剪切。剪切是一个有争议的话题。引擎和物理程序员往往讨厌它。艺术家往往喜欢它。鉴于大多数艺术家工具和游戏引擎在其等效类型中都支持剪切，我们相信提供这个选项很重要。

## ShaderType 派生宏 [#](https://bevy.org/news/bevy-0-8/#shadertype-derive)

作者：@teoxoy

**Bevy 0.8** 现在使用 [`ShaderType`](https://docs.rs/bevy/0.8.0/bevy/render/render_resource/trait.ShaderType.html) trait/派生宏（由 [encase](https://github.com/teoxoy/encase) crate 提供）来轻松地将 Rust 数据类型转换为 GPU 兼容的着色器数据类型。

```rust
// ShaderType 要求每个字段都实现 ShaderType，
// Bevy 的数学类型和 Color 类型都实现了它。
#[derive(ShaderType)]
struct SpriteData {
    position: Vec2,
    color: Color,
}
```

在 WGSL 着色器端，它看起来像这样：

```rust
struct SpriteData {
    position: vec2<f32>,
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> sprite_data: SpriteData;
```

如果你需要定义自定义着色器统一或缓冲区绑定，这个 trait 可以在[新材质系统](https://bevy.org/news/bevy-0-8/#new-material-system)中使用。

[`ShaderType`](https://docs.rs/bevy/0.8.0/bevy/render/render_resource/trait.ShaderType.html) 取代了以前的 Bevy 版本中使用的 `AsStd140` 和 `AsStd430` trait/派生宏（由 [crevice](https://github.com/LPGhatguy/crevice) crate 提供）。这简化了（并澄清了）Bevy 中"如何将数据传输到 GPU"的故事，同时还增加了新功能，例如支持无界 Rust vec（用于存储缓冲区）以及统一和存储缓冲区的可配置动态偏移。

例如，Bevy 的内置光照管线已经调整以利用这一点：

```rust
#[derive(ShaderType)]
pub struct GpuPointLightsStorage {
    #[size(runtime)]
    data: Vec<GpuPointLight>,
}
```

## 渲染阶段排序优化 [#](https://bevy.org/news/bevy-0-8/#render-phase-sorting-optimization)

作者：@james7132, Rob Swain (@superdump)

Bevy 使用"渲染阶段"来收集渲染通道的每个实体渲染逻辑。这些阶段可以被排序以控制绘制顺序，原因有多种：透明度正确性的从后到前深度排序，以及不透明渲染期间早期片段丢弃的前到后排序。

在可能的情况下，**Bevy 0.8** 现在使用"非稳定排序"（当前是"基数排序"），这带来了显著的性能提升：

### many_cubes 压力测试"不透明阶段"排序时间（以毫秒为单位，越少越好） [#](https://bevy.org/news/bevy-0-8/#many-cubes-stress-test-opaque-phase-sort-times-in-milliseconds-less-is-better)

![非稳定排序](https://bevy.org/news/bevy-0-8/unstable_sort.svg)

## 顶点颜色 [#](https://bevy.org/news/bevy-0-8/#vertex-colors)

作者：@HackerFoo, @thebracket

Bevy 的 2D 和 3D 管线以及 [`Materials`](https://docs.rs/bevy/0.8.0/bevy/pbr/trait.Material.html) 现在支持顶点颜色，前提是给定的 [`Mesh`](https://docs.rs/bevy/0.8.0/bevy/render/mesh/struct.Mesh.html) 提供了它们。PBR [`StandardMaterial`](https://docs.rs/bevy/0.8.0/bevy/pbr/struct.StandardMaterial.html) 和 2D [`ColorMaterial`](https://docs.rs/bevy/0.8.0/bevy/sprite/struct.ColorMaterial.html) 建立在此基础上，如果设置了顶点颜色，将使用它们：

![顶点颜色](https://bevy.org/news/bevy-0-8/vertex_colors.png)

## 正多边形和圆形网格基元 [#](https://bevy.org/news/bevy-0-8/#regular-polygon-and-circle-mesh-primitives)

作者：@rparrett

Bevy 现在有 [`Circle`](https://docs.rs/bevy/0.8.0/bevy/prelude/shape/struct.Circle.html) 和 [`RegularPolygon`](https://docs.rs/bevy/0.8.0/bevy/prelude/shape/struct.RegularPolygon.html) [`Mesh`](https://docs.rs/bevy/0.8.0/bevy/render/mesh/struct.Mesh.html) 形状：

![形状](https://bevy.org/news/bevy-0-8/shapes.png)

```rust
let pentagon = RegularPolygon::new(10., 5);
let handle = meshes.add(Mesh::from(pentagon));
commands.spawn_bundle(ColorMesh2dBundle {
    mesh: handle.into(),
    ..default()
});
```

## 脚本和 Modding 进度：无类型 ECS API [#](https://bevy.org/news/bevy-0-8/#scripting-and-modding-progress-untyped-ecs-apis)

作者：@jakobhellermann

Bevy 官方只支持 Rust 作为"定义应用逻辑的唯一方式"。我们有[很好的理由](https://github.com/bevyengine/bevy/issues/114#issuecomment-672397351)，而且这个理念在短期内可能不会改变。但我们**确实**想提供社区构建第三方脚本/modding 插件所需的工具，供他们选择语言使用。

当我们[发布 Bevy ECS V2](https://bevy.org/news/bevy-0-5/#bevy-ecs-v2) 时，我们特意考虑到这些情况构建了内部 ECS 存储。但我们没有公开暴露无需普通 Rust 类型就能与 ECS 数据交互的 API。

**Bevy 0.8** 添加了公开的"无类型"ECS API，使得可以使用 [`ComponentId`](https://docs.rs/bevy/0.8.0/bevy/ecs/component/struct.ComponentId.html) 而不是实际的 Rust 类型来检索组件的[生命周期指针](https://bevy.org/news/bevy-0-8/#ecs-lifetimed-pointers)和资源数据。

```rust
let health_ptr: Ptr = world.entity(player).get_by_id(heath_component_id).unwrap();
```

这些与我们的反射 API 结合时，提供了开始构建脚本支持所需的工具！

`@jakobhellermann` 已经开始[为 Bevy 构建他们自己的 JavaScript/TypeScript 插件](https://github.com/jakobhellermann/bevy_mod_js_scripting/blob/main/assets/scripts/debug.ts)。请注意：

1. 这个插件仍然是一个非常早期的进行中工作，尚未准备好用于项目。
2. 这是一个非官方的社区努力。Bevy 不会添加官方的 JavaScript/TypeScript 支持。

以下是一个来自其仓库的 TypeScript 代码片段，从脚本中查询 Bevy ECS 数据：

```typescript
const ballQuery = world.query({
    components: [ballId, transformId, velocityId],
});
for (const item of ballQuery) {
    let [ball, transform, velocity] = item.components;
    velocity = velocity[0];

    info(velocity.toString());
}
```

## Query IntoIter [#](https://bevy.org/news/bevy-0-8/#query-intoiter)

作者：@TheRawMeatball

Bevy ECS [`Queries`](https://docs.rs/bevy/0.8.0/bevy/ecs/system/struct.Query.html) 现在实现了 [`IntoIterator`](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html) trait，提供了对 Rust 更简洁迭代语法的访问：

```rust
// 在以前的 Bevy 版本中，必须手动调用 iter/iter_mut：
fn system(mut players: Query<&mut Player>) {
    for player in players.iter() {
    }
    
    for mut player in players.iter_mut() {
    }
}

// 在 Bevy 0.8 中，你可以选择使用此语法：
fn system(mut players: Query<&mut Player>) {
    for player in &players {
    }
    
    for mut player in &mut players {
    }
}
```

## Query::iter_many [#](https://bevy.org/news/bevy-0-8/#query-iter-many)

作者：@devil-ira

现在可以向 [`Query`](https://docs.rs/bevy/0.8.0/bevy/ecs/system/struct.Query.html) 传递一个实体列表，使用 [`Query::iter_many`](https://docs.rs/bevy/0.8.0/bevy/ecs/system/struct.Query.html#method.iter_many) 进行迭代，这比在每个单独的实体上调用 `get(entity)` 要符合人体工程学得多：

```rust
#[derive(Component)]
struct BlueTeam {
    members: Vec<Entity>,
}

fn system(blue_team: Res<BlueTeam>, players: Query<&Player>) {
    info!("蓝队集合！");
    for player in players.iter_many(&blue_team.members) {
        info!("{}", player.name);
    }    
}
```

还有一个 [`Query::iter_many_mut`](https://docs.rs/bevy/0.8.0/bevy/ecs/system/struct.Query.html#method.iter_many_mut)，提供了类似的可变查询功能。但为了确保不允许别名可变性，它没有实现 iterator。请改用此模式：

```rust
let mut iter = players.iter_many_mut(&blue_team.members);
while let Some(mut player) = iter.fetch_next() {
    player.score += 1;
}
```

## 将可变查询转换为只读查询 [#](https://bevy.org/news/bevy-0-8/#convert-mutable-queries-to-read-only)

作者：@harudagondi

可变 [`Queries`](https://docs.rs/bevy/0.8.0/bevy/ecs/system/struct.Query.html) 现在可以转换为其只读版本，这使得构建和使用查询之上的抽象更加容易：

```rust
fn system(mut players: Query<&mut Player>) {
    for mut player in &mut players {
        // 在此修改玩家
    }

    log_players(players.to_readonly());
}

fn log_players(players: Query<&Players>) {
    for player in &players {
        info!("{player:?}");
    }
}
```

## ECS"生命周期指针" [#](https://bevy.org/news/bevy-0-8/#ecs-lifetimed-pointers)

作者：@TheRawMeatball, @BoxyUwU, @jakobhellermann

Bevy ECS 已被重构为使用生命周期化、类型擦除的指针而不是"原始指针"，这显著提高了我们内部的安全性和可读性，而不影响性能或灵活性。

从高层次来看，这使我们能够在内部"保留"World 借用的生命周期，同时仍然使用类型擦除的 API 来支持诸如[第三方脚本语言](https://bevy.org/news/bevy-0-8/#scripting-and-modding-progress-untyped-ecs-apis)等场景。

通过保留生命周期，我们可以更多地依赖 Rust 的借用检查器在我们做不安全的事情时提醒我们。而且，碰巧的是，这抓住了一些健全性错误！

## ECS 查询内部重构 [#](https://bevy.org/news/bevy-0-8/#ecs-query-internals-refactors)

作者：@BoxyUwU

`@BoxyUwU` 一直在努力重构我们的 [`Query`](https://docs.rs/bevy/0.8.0/bevy/ecs/system/struct.Query.html) 内部，使其更简单、更易读：

- `ReadOnlyFetch` 被替换为 [`ReadOnlyWorldQuery`](https://docs.rs/bevy/0.8.0/bevy/ecs/query/trait.ReadOnlyWorldQuery.html)，将这个 trait 约束"提升了一个层次"，使其更容易在类型系统中表达。
- "QF Fetch 泛型"已从 [`Query`](https://docs.rs/bevy/0.8.0/bevy/ecs/system/struct.Query.html) 和 [`QueryState`](https://docs.rs/bevy/0.8.0/bevy/ecs/query/struct.QueryState.html) 的方法和类型中移除，转而使用 [`WorldQuery`](https://docs.rs/bevy/0.8.0/bevy/ecs/query/trait.WorldQuery.html)，使过滤器和普通 fetch 在类型系统中保持一致且更易表达。

我们在下一个版本中还有更多类似的更改计划。Bevy ECS 内部正在变得相当容易理解！

## ECS 优化 [#](https://bevy.org/news/bevy-0-8/#ecs-optimizations)

作者：@DJMcNab, @james7132

本次 Bevy ECS 进行了一系列优化：

- `@james7132` 通过减少执行的拷贝次数，加快了实体在表之间的移动速度。对于较大的组件，这是一个巨大的胜利。`many_cubes` 压力测试在 `prepare_uniform_components` 系统中看到了约 16% 的速度提升，该系统严重依赖命令/表移动。
- `@DJMcNab` 移除了 ECS 存储内部的无操作 drop 函数调用，这将 `many_cubes` 压力测试的 drop 时间从约 150μs 减少到约 80μs。
- `@james7132` 将 [`ComponentSparseSet`](http://docs.rs/bevy/0.8.0/bevy/ecs/storage/struct.ComponentSparseSet.html) 的索引从 `usize` 改为 `u32`，这使得它们占用更少的空间/使某些操作更加缓存友好。通过此更改，稀疏集合迭代速度提高了约 15%。

## 标签优化 [#](https://bevy.org/news/bevy-0-8/#label-optimizations)

作者：@JoJoJet

Bevy 依赖"标签"来标识系统、阶段和应用之类的东西。这对于诸如[定义系统依赖](https://bevy.org/news/bevy-0-5/#explicit-system-dependencies-and-system-labels)之类的事情很有用。[`SystemLabel`](https://docs.rs/bevy/0.8.0/bevy/ecs/schedule/trait.SystemLabel.html)、[`StageLabel`](https://docs.rs/bevy/0.8.0/bevy/ecs/schedule/trait.StageLabel.html) 和 [`AppLabel`](https://docs.rs/bevy/0.8.0/bevy/app/derive.AppLabel.html) trait 建立在相同的底层"类型化标签"系统之上。它允许开发者在保持类型安全的同时定义自定义标签。比使用字符串或整数标签好多了！

在 **Bevy 0.8** 中，我们通过移除装箱/trait 对象，转而使用单一的廉价复制和比较的"系统标签 id"类型，优化了标签的内部表示。

这种新表示使 schedule 构建速度提高了约 30%！

### 准备和计算包含 100 个系统的 schedule 的时间（以毫秒为单位，越少越好） [#](https://bevy.org/news/bevy-0-8/#time-to-prepare-and-compute-schedule-with-100-systems-in-milliseconds-less-is-better)

![标签基准](https://bevy.org/news/bevy-0-8/label_bench.svg)

这一变化还从我们的标签派生宏中移除了多个 trait 要求：

```rust
// 旧的（Bevy 0.7）
#[derive(SystemLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MovementSystem {
    Velocity,
    Gravity,
}

// 新的（Bevy 0.8）
#[derive(SystemLabel, Clone)]
enum MovementSystem {
    Velocity,
    Gravity,
}
```

## 层级命令 [#](https://bevy.org/news/bevy-0-8/#hierarchy-commands)

作者：@james7132

Bevy 的实体层级系统基于两个组件：[`Parent`](https://docs.rs/bevy/0.8.0/bevy/hierarchy/struct.Parent.html)（指向实体的父级）和 [`Children`](https://docs.rs/bevy/0.8.0/bevy/hierarchy/struct.Children.html)（指向实体子级列表）。这种分离很重要，因为它使得查询"层级根"变得简单且廉价：

```rust
fn system(roots: Query<Entity, Without<Parent>>) { }
```

在以前的 Bevy 版本中，我们构建了一个复杂的系统来"维护"层级的完整性。随着 [`Parent`](https://docs.rs/bevy/0.8.0/bevy/hierarchy/struct.Parent.html) / [`Children`](https://docs.rs/bevy/0.8.0/bevy/hierarchy/struct.Children.html) 组件的添加/移除/更改，我们会尽最大努力在整个层级中同步一切。

然而，这意味着在某一时间点，层级可能"不同步"且不正确。

我们解决这个问题的方法是移除延迟的"层级维护系统"，转而使层级更改成为"事务性的"。层级更改现在通过事务性的 [`Commands`](https://docs.rs/bevy/0.8.0/bevy/ecs/system/struct.Commands.html) 完成，并且不再可能直接单独修改组件字段。这确保在某一时间点，层级是"正确的"。

对于大多数 Bevy 开发者来说，这是一个非破坏性的变更，因为大多数层级已经使用 `with_children` 命令构建：

```rust
commands
    .spawn_bundle(SpatialBundle::default())
    .with_children(|parent| {
        parent.spawn_bundle(SpriteBundle {
            texture: player_texture,
            ..default()
        });
        parent.spawn_bundle(SpriteBundle {
            texture: hat_texture,
            ..default()
        });
    });
```

然而，对于在运行时从父级添加/移除子实体的逻辑，必须使用以下命令：

```rust
// 从父级移除给定的子节点
commands.entity(some_parent).remove_children(&[child1, child2]);
// 将给定的子节点推送到父级 Children 列表的"末尾"
commands.entity(some_parent).push_children(&[child3, child4]);
// 在父级 Children 列表的给定索引处插入给定的子节点
commands.entity(some_parent).insert_children(1, &[child5]);
```

我们还添加了 [`HierarchyEvent`](https://docs.rs/bevy/0.8.0/bevy/hierarchy/enum.HierarchyEvent.html)，使得开发者可以追踪层级的变化。

仍然有少量的小漏洞需要关闭，但保持在"快乐路径"上现在容易多了：

- 仅移除其中一个组件是可能的（尽管强烈不推荐）
- 手动仅添加其中一个组件的默认值仍然是可能的（尽管强烈不推荐）

我们正在讨论解决这类问题的方法，例如[原型规则/不变量](https://github.com/bevyengine/bevy/issues/1481)。

## Bevy 反射改进 [#](https://bevy.org/news/bevy-0-8/#bevy-reflection-improvements)

Bevy 的"Rust 反射"系统 `bevy_reflect` 是 Bevy 场景系统的核心基础组件。它提供了一种在运行时动态与 Rust 类型交互而无需知道其实际类型的方法。本次发布我们对其进行了大量投资，为脚本支持和场景系统改进做准备。

[`bevy_reflect`](https://crates.io/crates/bevy_reflect) 旨在成为一个"通用"的 Rust 反射系统。它可以在没有 Bevy 的情况下使用。我们相信它填补了 Rust 生态系统中的一个非常现实的空白，我们鼓励更广泛的 Rust 社区使用它（并贡献！）。

### "无类型"反射 [#](https://bevy.org/news/bevy-0-8/#untyped-reflection)

作者：@jakobhellermann

[`Reflect`](https://docs.rs/bevy/0.8.0/bevy/reflect/trait.Reflect.html) 派生宏现在会自动为每个反射类型向 [`TypeRegistry`](https://docs.rs/bevy/0.8.0/bevy/reflect/struct.TypeRegistry.html) 添加一个新的 [`ReflectFromPtr`](https://docs.rs/bevy/0.8.0/bevy/reflect/struct.ReflectFromPtr.html) 结构体。这使得可以将新的[无类型 ECS API](https://bevy.org/news/bevy-0-8/#scripting-and-modding-progress-untyped-ecs-apis) 与反射系统结合使用。这有助于实现诸如第三方脚本和 modding 等功能。

### Default Trait 反射 [#](https://bevy.org/news/bevy-0-8/#default-trait-reflection)

作者：@jakobhellermann

现在可以使用其 [`Default`](https://doc.rust-lang.org/std/default/trait.Default.html) trait 实现来构造 [`Reflect`](https://docs.rs/bevy/0.8.0/bevy/reflect/trait.Reflect.html) 类型，前提是它们将其注册为 [`Reflect`](https://docs.rs/bevy/0.8.0/bevy/reflect/trait.Reflect.html) 派生宏的一部分：

```rust
#[derive(Reflect, Default)]
#[reflect(Default)]
struct MyStruct {
    foo: String,
    bar: usize,
}

let registration = type_registry.get(TypeId::of::<MyStruct>()).unwrap();
let reflect_default = registration.data::<ReflectDefault>().unwrap();
// 这包含一个带有默认值的 MyStruct 实例
let my_struct: Box<dyn Reflect> = reflect_default.default();
```

这使得可以在**没有任何**编译时信息的情况下为实体构造组件，这对于脚本和场景等动态场景很有用。

### 数组反射 [#](https://bevy.org/news/bevy-0-8/#array-reflection)

作者：@NathanSWard, @MrGVSV

Bevy 的反射系统现在支持反射 Rust 数组，可以使用新的 [`Array`](https://docs.rs/bevy/0.8.0/bevy/reflect/trait.Array.html) trait 以类型擦除的方式进行交互。

```rust
#[derive(Reflect)]
struct Sprite {
    position: [f32; 2],
}

let sprite = Sprite {
    position: [0.1, 0.2],
};

let position = sprite.field("position").unwrap();
if let ReflectRef::Array(array) = position.reflect_ref() {
    let x = array.get(0).unwrap();
    assert_eq!(x.downcast_ref::<f32>(), Some(&0.1));
}
```

### 静态 TypeInfo [#](https://bevy.org/news/bevy-0-8/#static-typeinfo)

作者：@MrGVSV

[`Reflect`](https://docs.rs/bevy/0.8.0/bevy/reflect/trait.Reflect.html) trait 提供了对类型**特定实例**的动态访问，但某些场景（例如反序列化）受益于在拥有类型实例**之前**知道类型信息。这为构建基于 Reflect 的 `serde` 替代方案（或者至少是一个无 serde 的 Bevy 场景反序列化器）打开了大门。

**Bevy 0.8** 增加了为任何实现 [`Reflect`](https://docs.rs/bevy/0.8.0/bevy/reflect/trait.Reflect.html) 的类型检索 [`TypeInfo`](https://docs.rs/bevy/0.8.0/bevy/reflect/enum.TypeInfo.html) 的能力：

```rust
#[derive(Reflect)]
struct Foo {
    a: f32,
    b: usize,
}

let info = Foo::type_info();
if let TypeInfo::Struct(info) = info {
  assert!(info.is::<Foo>());
  assert_eq!(info.type_name(), std::any::type_name::<Foo>(),);
  assert!(info.field("a").unwrap().is::<f32>());
  assert!(info.field_at(1).unwrap().is::<usize>());
}
```

注意 `type_info()` 返回 `&'static TypeInfo`：它会在第一次请求时延迟分配并存储 [`TypeInfo`](https://docs.rs/bevy/0.8.0/bevy/reflect/enum.TypeInfo.html)，然后在后续每次请求时重用。

泛型类型也支持 [`TypeInfo`](https://docs.rs/bevy/0.8.0/bevy/reflect/enum.TypeInfo.html)：

```rust
#[derive(Reflect)]
struct Foo<T> {
    value: T
}

let info = Foo::<f32>::type_info();
if let TypeInfo::Struct(info) = info {
  assert!(info.field("value").unwrap().is::<f32>());
}
```

### 资源反射 [#](https://bevy.org/news/bevy-0-8/#resource-reflection)

作者：@Shatur

Bevy ECS 资源现在可以反射：

```rust
#[derive(Reflect)]
#[reflect(Resource)]
struct Scoreboard {
    points: usize,
}
```

这会在类型的 [`TypeRegistry`](https://docs.rs/bevy/0.8.0/bevy/reflect/struct.TypeRegistry.html) 条目中注册一个 [`ReflectResource`](https://docs.rs/bevy/0.8.0/bevy/ecs/reflect/struct.ReflectResource.html)，使得可以在 ECS [`World`](https://docs.rs/bevy/0.8.0/bevy/ecs/world/struct.World.html) 中对资源进行类型擦除的读/写操作。

### 漂亮的 Reflect Debug 格式化 [#](https://bevy.org/news/bevy-0-8/#pretty-reflect-debug-formatting)

作者：@MrGVSV

"Debug 打印" [`Reflect`](https://docs.rs/bevy/0.8.0/bevy/reflect/trait.Reflect.html) 引用现在提供漂亮/有用的输出。

考虑以下示例：

```rust
#[derive(Reflect)]
struct Foo {
    a: f32,
    b: Bar,
}

#[derive(Reflect)]
struct Bar {
    x: String,
    y: usize,
}
let foo = Foo {
    a: 42.0,
    b: Bar {
        x: "hello".to_string(),
        y: 123,
    },
};

let foo_reflect: &dyn Reflect = &foo;
println!("{:#?}", foo_reflect);
```

在以前的 Bevy 版本中，这会打印：

```txt
Reflect(my_crate::Foo)
```

在 **Bevy 0.8** 中，它打印：

```txt
my_crate::Foo {
    a: 42.0,
    b: my_crate::Bar {
        x: "hello",
        y: 123,
    },
}
```

好多了！

### bevy_reflect 内部重构 [#](https://bevy.org/news/bevy-0-8/#bevy-reflect-internal-refactors)

作者：@MrGVSV, @PROMETHIA-27, @jakobhellermann

既然 `bevy_reflect` 开始获得一些认真的投入和使用，我们已经投入时间重构内部实现，使它们更易于维护和扩展：

- ** [`Reflect`](https://docs.rs/bevy/0.8.0/bevy/reflect/trait.Reflect.html) 派生宏重组**：派生逻辑被分解为更小、更易于维护的部分。添加了"元数据结构体"来收集和组织派生输入。（`@MrGVSV`）
- ** [`Reflect`](https://docs.rs/bevy/0.8.0/bevy/reflect/trait.Reflect.html) trait 现在是安全的**：健全性不再取决于实现者是否做了正确的事情，这要感谢对 [`Reflect`](https://docs.rs/bevy/0.8.0/bevy/reflect/trait.Reflect.html) 接口的一些更改。因此，我们能够从 [`Reflect`](https://docs.rs/bevy/0.8.0/bevy/reflect/trait.Reflect.html) trait 中移除 `unsafe` 关键字。（`@PROMETHIA-27`）
- ** `Serialize` 逻辑现在使用 [`TypeRegistry`](https://docs.rs/bevy/0.8.0/bevy/reflect/struct.TypeRegistry.html) 类型数据实现，就像其他反射 trait 逻辑一样，而不是硬编码到 [`Reflect`](https://docs.rs/bevy/0.8.0/bevy/reflect/trait.Reflect.html) 实现中。（`@jakobhellermann`）

## 渲染世界 Extract [#](https://bevy.org/news/bevy-0-8/#render-world-extract)

作者：@DJMcNab, @cart

注意：这里讨论的渲染器 API 适用于高级自定义渲染功能的开发者和核心 Bevy 渲染器开发者。如果这显得冗长或意图令人困惑，别担心！

Bevy 的[新渲染器](https://bevy.org/news/bevy-0-6/#the-new-bevy-renderer)从"主" Bevy 应用中"提取"渲染所需的数据，从而实现并行的[管线化渲染](https://bevy.org/news/bevy-0-6/#pipelined-rendering-extract-prepare-queue-render)。为了促进这一点，在以前的 Bevy 版本中，我们使 ECS [`RenderStage::Extract`](https://docs.rs/bevy/0.8.0/bevy/render/enum.RenderStage.html#variant.Extract) 变得"特殊"（并且有点奇怪）。该阶段的系统在"主"应用世界上运行，但将系统的 [`Commands`](https://docs.rs/bevy/0.8.0/bevy/ecs/system/struct.Commands.html) 应用到"渲染"应用世界。

这实现了目标，但它：

1. **令人困惑**：渲染功能开发者必须"知道"这个阶段在 schedule 中与其他"正常"ECS 阶段的行为不同。隐式地，[`Commands`](https://docs.rs/bevy/0.8.0/bevy/ecs/system/struct.Commands.html) 的行为不同，ECS 数据访问是"翻转的"。使用"正常"的实体生成 API **不会按预期工作**，因为 [`Commands`](https://docs.rs/bevy/0.8.0/bevy/ecs/system/struct.Commands.html) 参数内部仍然使用主应用的 Entities 集合。
2. **阻止并行化**：直接修改现有的"渲染世界"资源需要对 `ResMut<RenderWorld>` 的独占访问，这阻止了这些系统并行运行。使这种访问并行需要使用 [`Commands`](https://docs.rs/bevy/0.8.0/bevy/ecs/system/struct.Commands.html) 进行不必要的分配，这对于"大型"（或增量更新）的提取来说是低效的。

```rust
// 旧的：Bevy 0.7（有限并行）
fn extract_score(score: Res<Score>, mut render_world: ResMut<RenderWorld>) {
    *render_world.resource_mut::<ExtractedScore>() = ExtractedScore::from(score);
}

// 旧的：Bevy 0.7（不必要的分配/阻止增量更新）
fn extract_score(mut commands: Commands, score: Res<Score>) {
    commands.insert_resource(ExtractedScore::from(&score));
}
```

在 **Bevy 0.8** 中，我们使 extract 阶段变得"正常"。它直接在"渲染世界"上运行，就像其他渲染应用阶段一样。要从主应用世界"提取"数据，只需将相关的系统参数包装在新的 [`Extract`](https://docs.rs/bevy/0.8.0/bevy/render/struct.Extract.html) 类型中，以从主应用世界检索该参数：

```rust
// 新的：Bevy 0.8（并行且不奇怪！）
fn extract_score(mut extracted_score: ResMut<ExtractedScore>, score: Extract<Res<Score>>) {
    *extracted_score = ExtractedScore::from(&score);
}
```

extract 系统现在是并行的，数据访问与其他渲染器 ECS 阶段一致，系统的意图也更加清晰。

## ExtractResource Trait 和 Plugin [#](https://bevy.org/news/bevy-0-8/#extractresource-trait-and-plugin)

作者：Rob Swain (@superdump)

一些 ECS 资源具有非常简单的提取逻辑：

```rust
fn extract_cool_color(mut extracted_cool_color: ResMut<CoolColor>, cool_color: Extract<Res<CoolColor>>) {
    *extracted_cool_color = cool_color.clone();
}
```

与其强迫开发者手动编写这些，**Bevy 0.8** 现在提供了 [`ExtractResource`](https://docs.rs/bevy/0.8.0/bevy/render/extract_resource/trait.ExtractResource.html) trait / 派生宏：

```rust
#[derive(ExtractResource, Clone)]
struct CoolColor {
    color: Color,
}
```

然后，只需将 [`ExtractResourcePlugin<CoolColor>`](https://docs.rs/bevy/0.8.0/bevy/render/extract_resource/struct.ExtractResourcePlugin.html) 添加到你的 [`App`](https://docs.rs/bevy/0.8.0/bevy/app/struct.App.html) 中，资源将自动被提取。

[`ExtractResource`](https://docs.rs/bevy/0.8.0/bevy/render/extract_resource/trait.ExtractResource.html) 也可以手动实现，如果你需要自定义逻辑（或类型需要更改）：

```rust
impl ExtractResource for ExtractedCoolColor {
    type Source = CoolColor;
    fn extract_resource(source: &CoolColor) -> Self {
        Self {
            color: source.color.as_rgba_linear(),
        }
    }
}
```

## Taffy 迁移：焕然一新的 UI 布局库 [#](https://bevy.org/news/bevy-0-8/#taffy-migration-a-refreshed-ui-layout-library)

作者：@alice-i-cecile, @jkelleyrtp, @Weibye, @TimJentzsch, @colepoirier

Bevy 已从已废弃的 [`stretch`](https://crates.io/crates/stretch) UI 布局 crate 迁移到其新的社区维护的硬 fork：[`taffy`](https://crates.io/crates/taffy)。与 [Dioxus](https://dioxuslabs.com/) 团队一起，我们大幅清理了代码库，解决了深层 UI 树的一个关键性能问题，并刷新了文档。

我们期待其持续的维护和发展，因为团队将继续改进其性能、修复错误，并增加对替代布局范式的支持。

## ECS 健全性/正确性改进 [#](https://bevy.org/news/bevy-0-8/#ecs-soundness-correctness-improvements)

作者：@TheRawMeatball, @BoxyUwU, @SkiFire13, @jakobhellermann

本次发布中，Bevy ECS 收到了大量健全性和正确性的错误修复：

- **移除了 `EntityMut::get_unchecked`**：唯一能安全使用此 API 的方式已经封装在 `EntityMut::get` 中。因此，没有理由保留这个 unsafe API。
- **修复了某些 `Or`/`AnyOf`/`Option` 组件访问的不健全问题**：以前的 Bevy 版本允许这些查询的不安全变体。我们现在正确防止了这些使用。
- **改进了 `CommandQueue` 的健全性**：现在可以安全地存储带有填充或未初始化字节的 Commands。还移除了一个"移动后使用"的情况。
- **修复了 Miri 检测到的一些内存泄漏**：Miri 检测到我们的 `BlobVec` drop 实现中存在泄漏。现在已修复。
- **在 `bevy_ecs` 中添加了对缺失 SAFETY 注释的 lint 检查**：我们现在要求 `bevy_ecs` 中的 unsafe 代码块必须有 safety 注释。

随着 Bevy ECS 的成熟，我们对 unsafe 代码块和健全性的标准也必须提高。Bevy ECS 可能永远不会 100% 没有 unsafe 代码块，因为我们在建模 Rust 在没有我们帮助的情况下无法推理的并行数据访问。但我们致力于尽可能多地移除 unsafe 代码，并提高剩余 unsafe 代码的质量。

## Android 进度：我们还没到，但更近了！ [#](https://bevy.org/news/bevy-0-8/#android-progress-we-aren-t-there-yet-but-we-re-closer)

作者：@mockersf

Bevy 现在"有点"能在 Android 上运行了！

然而，Android 支持**还没有**准备好。我们管理渲染上下文的方式存在问题必须解决，这些问题有时会在启动时破坏渲染，并且当应用被最小化时**总是**破坏渲染。音频也还不能工作。

以下是在我的 Pixel 6 上运行的 `load_gltf` Bevy 示例：

![android](https://bevy.org/news/bevy-0-8/android.png)

话虽如此，这是向前迈出的重要一步，因为 Bevy 开发者现在可以在 Android 上构建、部署（在某些情况下还可以测试）Bevy 应用！

如果你渴望在移动平台上测试 Bevy，我们的 [iOS 支持](https://github.com/bevyengine/bevy/blob/v0.8.0/examples/README.md#ios) 要成熟得多。Bevy 开发者已经开始[向 Apple App Store 发布基于 Bevy 的 iOS 应用](https://noumenal.app/)了！

## CI / 构建系统改进 [#](https://bevy.org/news/bevy-0-8/#ci-build-system-improvements)

作者：@mockersf, @NiklasEi

像往常一样，Bevy 的 CI 在本周期进行了大量改进：

- 示例现在在验证构建时以 WASM 方式运行。截图并作为构建输出的一部分存储，以确保渲染正常工作（`@mockersf`）。
- Bevy 示例现在每天在 Windows VM 上运行一次，以确保它们没有被破坏（`@mockersf`）。
- 许可证文件现在自动添加到所有发布的 crate 中（`@NiklasEi`）。
- 现在有一个工作流可以自动生成一个 PR，用于为所有 Bevy crate 提升版本号（`@mockersf`）。
- 为了减少偶尔的 Rust 夜间版破坏的影响，我们对夜间工具链进行了参数化，使得更容易固定到特定的夜间版本。（`@mockersf`）

## 示例：后处理 [#](https://bevy.org/news/bevy-0-8/#example-post-processing)

作者：@Vrixyz

我们添加了一个新的示例，展示了如何使用新的[相机驱动渲染](https://bevy.org/news/bevy-0-8/#camera-driven-rendering)和[着色器材质](https://bevy.org/news/bevy-0-8/#new-material-system)来使用全屏四边形和"渲染到纹理"构建"色差"后处理效果。

![后处理](https://bevy.org/news/bevy-0-8/post_processing.png)

我们计划在未来的发布中构建更正式的后处理 API，但这种方法相对直接，并打开了许多大门。比扩展底层的 [`RenderGraph`](https://docs.rs/bevy/0.8.0/bevy/render/render_graph/struct.RenderGraph.html) 简单多了！

通过克隆 Bevy 仓库并运行 `cargo run --release --example post_processing` 来运行它。

## 示例：许多动画狐狸 [#](https://bevy.org/news/bevy-0-8/#example-many-animated-foxes)

作者：@superdump

这是对我们的[骨骼动画系统](https://bevy.org/news/bevy-0-7/#skeletal-animation)的一个有趣的压力测试，它渲染了许多动画狐狸在围成一圈行走。

![许多狐狸](https://bevy.org/news/bevy-0-8/many_foxes.png)

通过克隆 Bevy 仓库并运行 `cargo run --release --example many_foxes` 来运行它。

## WASM 示例构建工具 [#](https://bevy.org/news/bevy-0-8/#wasm-example-build-tool)

作者：@mockersf

我们构建了一个工具，使得在你的浏览器中构建和运行 Bevy 的示例更加容易：

在 Bevy 仓库的根目录中，运行以下命令：

```sh
cargo run -p build-wasm-example -- lighting
```

这将运行 `cargo build` 和 `wasm-bindgen` 命令，并将输出放在 `examples/wasm` 文件夹中。在那里运行你喜欢的"本地 web 服务器"命令，例如 `python3 -m http.server`，并在浏览器中打开该 URL！

## 网站：改进的示例页面 [#](https://bevy.org/news/bevy-0-8/#website-improved-examples-page)

作者：@doup, @ickk

[Bevy WASM 示例](https://bevy.org/examples/)页面已经重新设计：

- 它们现在在 Bevy 应用内容和资源加载时显示加载条
- 页面的设计/布局现在好多了

![示例页面](https://bevy.org/news/bevy-0-8/examples_page.png)

## Bevy 组织变更 [#](https://bevy.org/news/bevy-0-8/#bevy-org-changes)

随着 Bevy 的成长，我们不断重新评估我们的开发流程，以适应日益增长的开发量。我早已过了能做每个决定的阶段，并且随着我对社区成员知识和经验信任的增长，我一直在慢慢下放责任。

在本发布周期中，Bevy 组织有两个重大变化：

1. 所有具有"委派合并权限"的现有开发者（`@mockersf` 和 `@alice-i-cecile`）现在拥有"维护者"的头衔。
2. Rob Swain（`@superdump`）现在是维护者。你可能从他们在 Bevy 渲染器上的工作中认识他们。他们一直是名副其实的自然力量，推动着分簇前向渲染、方向光和点光源阴影、可见性/视锥体裁剪、Alpha 混合、压缩 GPU 纹理等工作。Rob 展示了对渲染算法、Bevy 内部以及 Bevy 项目方向的深刻理解。我肯定期待他们接下来构建什么！

成为"维护者"现在是这样运作的：

1. **维护者现在对他们可以合并的 PR 的"领域"没有（硬性）限制**。不再有"仅文档"、"仅渲染"等限制。现在，每个维护者都有责任评估自己擅长的领域。这确实在一定程度上增加了风险，但我认为这是让维护者有机成长的重要一步。
2. **维护者可以在至少两个社区批准的情况下合并"相对无争议"的 PR**。维护者将共同决定和强制执行什么是无争议的。有争议的 PR 应该用 `S-Controversial` 标签标记。注意"两个社区批准"是最低要求。维护者负责确保合适的人员已经批准了一个 PR。
3. **维护者可以在没有两个社区批准的情况下合并"完全琐碎的"PR**。"完全琐碎"的一些例子：拼写错误修复、移除未使用的依赖或代码，以及小的"API 一致性"修复。
4. **计时器上的有争议决策**：对于所有有争议的 PR（包括 RFC），如果两个维护者批准，PR 可以标记为 `S-Ready-For-Final-Review` 标签。一旦这个标签被添加并且我已被 ping，一个计时器就开始了。如果我在一个半月（45 天）内没有回复可操作反馈、"暂停按钮"/"我们还没准备好"或否决，维护者可以自由合并 PR。这使我在重要的领域能够指定项目方向，同时，在合理的情况下，赋予维护者并行推进事务的能力。我们将随着进展校准这种方法，以确保我们在进度、质量和一致愿景之间取得正确的平衡。
5. **我仍然保留否决所有代码更改和进行单方面代码更改的权利**。这包括撤销通过第（4）条合并的"有争议的更改"。

我们在上一个周期的大部分时间里都使用了这个流程，我喜欢它到目前为止的工作方式：更多的信任，每个决定有更多的眼睛，更快的开发速度，不再有琐碎的修复处于悬而未决的状态。我仍然可以在重要的时候强制执行一致的愿景，但社区有能力推动事务向前发展。

## 接下来是什么？ [#](https://bevy.org/news/bevy-0-8/#what-s-next)

- **后处理**：我们有很多后处理工作在进行中（其中一些几乎进入了这个版本）。下一个版本将使编写后处理效果更容易（感谢中间 HDR 纹理和单独的色调映射步骤），它还将包括内置效果，如泛光和缩放。
- **资产预处理**：我们将大力投资我们的资产管线，重点是：
    1. 预处理资产以在"开发期间"完成昂贵的工作，以便 Bevy 应用可以部署更美观、更小和/或加载更快的资产。
    2. 支持使用 `.meta` 文件配置资产。例如，你可以定义纹理压缩级别、应使用的滤镜或目标格式。
- **场景系统改进**：本次发布在反射方面进行了大量投资。我们现在可以在其上构建下一代的场景系统，提供更漂亮的场景格式、嵌套场景和改进的工作流程。
- **Bevy UI 改进**：为视觉化的 Bevy Editor 做准备，我们将改进 Bevy UI 的功能和用户体验。
- **Bevy Jam #2**：[Bevy Jam #1](https://itch.io/jam/bevy-jam-1) 取得了巨大成功：74 个参赛作品、1,618 个评分以及大量良好的社区氛围。现在 **Bevy 0.8** 已经发布，是时候再次 jam 了！我们将很快发布详情。要关注最新动态，请在 Twitter 上关注 [@BevyEngine](https://twitter.com/BevyEngine) 并加入[官方 Bevy Discord](https://discord.gg/bevy)。
