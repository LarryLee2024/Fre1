# Bevy 0.13

## 发布于 2024 年 2 月 17 日，作者：Bevy 贡献者

![来自 Jarl（一款正在制作中的、使用 Bevy 制作的奇幻殖民地建造游戏）的游戏画面](https://bevy.org/news/bevy-0-13/jarl.webp)

[来自 Jarl（一款正在制作中的、使用 Bevy 制作的奇幻殖民地建造游戏）的游戏画面](https://www.jarl-game.com/)

感谢 **198** 位贡献者、**672** 个拉取请求、社区审阅者以及我们的[**慷慨的赞助者**](https://bevy.org/community/donate)，我们很高兴地在 [crates.io](https://crates.io/crates/bevy) 上发布 **Bevy 0.13**！

对于那些不了解的人，Bevy 是一个用 Rust 构建的、令人耳目一新的简单数据驱动游戏引擎。您可以查看我们的[快速入门指南](https://bevy.org/learn/quick-start)立即尝试。它是自由且永远开源的！您可以在 GitHub 上获取完整的[源代码](https://github.com/bevyengine/bevy)。查看 [Bevy Assets](https://bevy.org/assets) 获取社区开发的插件、游戏和学习资源合集。要亲身体验引擎的功能，请查看[最新 Bevy Jam](https://itch.io/jam/bevy-jam-4/entries) 的参赛作品，包括获胜者 [That's a LOT of beeeeees](https://andrewb330.itch.io/thats-a-lot-of-beeeeees)。

要将现有的 Bevy App 或 Plugin 更新到 **Bevy 0.13**，请查看我们的[0.12 到 0.13 迁移指南](https://bevy.org/learn/migration-guides/0-12-to-0-13/)。

自从我们上次发布几个月以来，我们增加了_大量_新功能、Bug 修复和生活质量改进，但以下是一些亮点：

- **光照贴图（Lightmaps）：** 一种快速、流行的烘焙全局光照技术，用于静态几何体（在 The Lightmapper 等程序中外部烘焙）。
- **辐照度体素 / 体素全局光照（Irradiance Volumes / Voxel Global Illumination）：** 一种烘焙形式的全局光照，在长方体中的体素中心采样光照（在 Blender 等程序中外部烘焙）。
- **近似间接镜面遮挡（Approximate Indirect Specular Occlusion）：** 通过镜面遮挡减少镜面光泄漏，提高光照真实感。
- **反射探针（Reflection Probes）：** 一种烘焙形式的轴对齐环境贴图，允许为静态几何体实现逼真的反射（在 Blender 等程序中外部烘焙）。
- **基本形状（Primitive shapes）：** 基本形状是游戏引擎和电子游戏的核心构建块：我们添加了一个精致、可直接使用的集合！
- **系统步进（System stepping）：** 完全暂停并通过你的游戏逐帧或逐系统前进，以交互方式调试游戏逻辑，同时渲染持续更新。
- **动态查询（Dynamic queries）：** 在系统内部优化查询极具表现力，并且是运行时定义类型和第三方 Modding 与脚本集成的最后一块大拼图。
- **自动推断命令刷新点（Automatically inferred command flush points）：** 厌倦了思考在哪里放置 `apply_deferred`，并困惑为什么你的命令没有被应用？我们也是！现在，Bevy 的调度器使用普通的 `.before` 和 `.after` 约束，并检查系统参数以自动推断（并去重）同步点。
- **切片、平铺和九宫格 2D 图像（Slicing, tiling and nine-patch 2D images）：** 九宫格布局是一种流行的工具，用于平滑缩放风格化的图块集和 UI。现在 Bevy 也有了！
- **相机驱动 UI（Camera-Driven UI）：** UI 实体树现在可以有选择地添加到_任何_相机，而不是全局应用于所有相机，从而实现分屏 UI 等功能！
- **相机曝光（Camera Exposure）：** 通过 EV100、光圈值、快门速度和 ISO 感光度实现对相机曝光的逼真/"真实世界"控制。灯光也已调整，使其单位更加真实。
- **动画插值模式（Animation interpolation modes）：** Bevy 现在支持导出的 glTF 动画中的非线性插值模式。

## 初始烘焙光照 [#](https://bevy.org/news/bevy-0-13/#initial-baked-lighting)

实时计算光照是昂贵的；但对于场景中永不移动的元素（如房间或地形），我们可以通过提前使用**全局光照**计算光照，然后将结果存储在永不改变的"烘焙"形式中，以更低的成本获得更漂亮的光照和阴影。全局光照是一种更真实（也更昂贵）的光照方法，通常使用光线追踪。与 Bevy 的默认渲染不同，它考虑了从其他物体反弹的光线，通过包含间接光照产生更真实的效果。

### 光照贴图 [#](https://bevy.org/news/bevy-0-13/#lightmaps)

作者：@pcwalton

![光照贴图](https://bevy.org/news/bevy-0-13/lightmap.jpg)

**光照贴图**是存储预计算全局光照结果的纹理。几十年来，它们一直是实时图形的主力。**Bevy 0.13** 添加了对渲染在其他程序（如 [The Lightmapper](https://github.com/Naxela/The_Lightmapper)）中计算的光照贴图的初始支持。最终，我们希望添加直接在 Bevy 中烘焙光照贴图的支持，但这第一步已经解锁了光照贴图工作流！

如[光照贴图示例](https://github.com/bevyengine/bevy/blob/main/examples/3d/lightmaps.rs)所示，只需加载你烘焙好的光照贴图图像，然后在相应的网格上插入 [`Lightmap`](https://docs.rs/bevy/0.13.0/bevy/pbr/struct.Lightmap.html) 组件即可。

### 辐照度体素 / 体素全局光照 [#](https://bevy.org/news/bevy-0-13/#irradiance-volumes-voxel-global-illumination)

作者：@pcwalton

![辐照度体素](https://bevy.org/news/bevy-0-13/irradiance_volume.jpg)

**辐照度体素**（或体素全局光照）是一种用于近似间接光的技术，首先将场景划分为立方体（体素），然后对每个体素中心的光量进行采样。然后，当物体在空间中移动时，这些光照会被添加到该空间内的物体上，适当地改变这些物体上的环境光水平。

我们选择为此使用基于《半条命 2》的环境立方体算法。这使我们能够匹配 Blender 的 [Eevee 渲染器](https://docs.blender.org/manual/en/latest/render/eevee/index.html)，为用户提供一条简单且免费的道路，来为他们自己的场景创建好看的辐照度体素。

注意这个球体在移动时是如何微妙地拾取环境颜色的，这要归功于辐照度体素：

目前，你需要使用 Blender 等外部工具来烘焙辐照度体素，但将来我们希望支持直接在 Bevy 中烘焙辐照度体素！

## 最小反射探针 [#](https://bevy.org/news/bevy-0-13/#minimal-reflection-probes)

作者：@pcwalton

**环境贴图**是用于模拟 3D 场景中光照、反射和天空盒的 2D 纹理。**反射探针**将环境贴图泛化，允许在同一个场景中使用多个环境贴图，每个环境贴图都有自己的轴对齐边界框。这是基于物理渲染器的标准功能，其灵感来自 [Blender 的 Eevee 渲染器中的相应功能](https://docs.blender.org/manual/en/latest/render/eevee/light_probes/reflection_cubemaps.html)。

在[反射探针 PR](https://github.com/bevyengine/bevy/pull/11366) 中，我们添加了对这些的基本支持，为 Bevy 游戏中漂亮、高性能的反射奠定了基础。与上面讨论的烘焙全局光照工作一样，这些目前必须在外部预计算，然后导入到 Bevy 中。如 PR 中所讨论的，有不少注意事项：WebGL2 支持几乎不存在，由于没有混合，会观察到尖锐和突然的过渡，并且世界中给定类型（漫反射或镜面反射）的所有立方体贴图必须具有相同的大小、格式和 mipmap 数量。

![反射探针](https://bevy.org/news/bevy-0-13/reflection_probes.jpg)

## 近似间接镜面遮挡 [#](https://bevy.org/news/bevy-0-13/#approximate-indirect-specular-occlusion)

作者：@aevyrie

Bevy 当前的 PBR 渲染器会使图像过亮，尤其是在掠射角，菲涅耳效应往往会使表面表现得像镜子。这种过亮的发生是因为表面必须反射_某些东西_，但在没有路径追踪或屏幕空间反射的情况下，渲染器必须猜测正在反射_什么_。它能做的最好猜测就是采样环境立方体贴图，即使光线在到达环境光之前可能会撞到其他东西。这种忽略光线遮挡的伪影称为镜面光泄漏。

考虑一个汽车轮胎；虽然橡胶可能有光泽，但你不希望在轮罩内部看到明亮的镜面高光，因为汽车本身正在阻挡（遮挡）本会引起这些反射的光线。完全检查遮挡在计算上可能很昂贵。

**Bevy 0.13** 添加了对**近似间接镜面遮挡**的支持，它使用我们现有的[屏幕空间环境光遮蔽](https://bevy.org/news/bevy-0-11/#screen-space-ambient-occlusion)来_近似_镜面遮挡，可以在实时高效运行，同时仍然产生相当高质量的结果：

拖动此图像进行比较

![镜面遮挡开启](https://bevy.org/news/bevy-0-13/specular_occlusion_on.jpg)![镜面遮挡关闭](https://bevy.org/news/bevy-0-13/specular_occlusion_off.jpg)

模型致谢：[BMW R1200GS Motorcycle](https://sketchfab.com/3d-models/bmw-r1200gs-motorcycle-6550451b0ae547039585a44286b2f530) 由 Moto3D 制作，根据 [CC-BY-4.0](http://creativecommons.org/licenses/by/4.0/) 许可。

将来，这可以通过屏幕空间反射（SSR）进一步改进。然而，传统的观点是，你应该将镜面遮挡与 SSR 一起使用，因为 SSR 仍然会受到光泄漏伪影的影响。

## 基本形状 [#](https://bevy.org/news/bevy-0-13/#primitive-shapes)

作者：@Jondolf, @NiseVoid, @aevyrie

几何形状贯穿游戏开发的方方面面，从基本网格形状和调试 gizmo 到物理碰撞体和光线投射。尽管在多个领域中如此常用，但 Bevy 之前并没有任何通用的形状表示。

这种情况在 **Bevy 0.13** 中正在改变，我们引入了第一方的**基本形状**！它们是轻量级的几何基元，旨在实现最大的互操作性和可重用性，允许 Bevy 和第三方插件使用同一套基本形状，并提高生态系统内的内聚性。有关更多详细信息，请参阅原始 [RFC](https://github.com/bevyengine/rfcs/blob/main/rfcs/12-primitive-shapes.md)。

内置的[基本形状集合](https://docs.rs/bevy/0.13.0/bevy/math/primitives/index.html)已经相当可观：

|2D|3D|
|---|---|
|[`Rectangle`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Rectangle.html)|[`Cuboid`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Cuboid.html)|
|[`Circle`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Circle.html)|[`Sphere`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Sphere.html)|
|[`Ellipse`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Ellipse.html)||
|[`Triangle2d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Triangle2d.html)||
|[`Plane2d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Plane2d.html)|[`Plane3d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Plane3d.html)|
|[`Line2d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Line2d.html)|[`Line3d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Line3d.html)|
|[`Segment2d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Segment2d.html)|[`Segment3d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Segment3d.html)|
|[`Polyline2d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Polyline2d.html), [`BoxedPolyline2d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.BoxedPolyline2d.html)|[`Polyline3d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Polyline3d.html), [`BoxedPolyline3d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.BoxedPolyline3d.html)|
|[`Polygon`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Polygon.html), [`BoxedPolygon`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.BoxedPolygon.html)||
|[`RegularPolygon`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.RegularPolygon.html)||
|[`Capsule2d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Capsule2d.html)|[`Capsule3d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Capsule3d.html)|
||[`Cylinder`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Cylinder.html)|
||[`Cone`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Cone.html)|
||[`ConicalFrustum`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.ConicalFrustum.html)|
||[`Torus`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Torus.html)|

[更多基本形状](https://github.com/bevyengine/bevy/issues/10572)将在未来版本中添加。

基本形状的一些用例包括网格化、gizmo、包围体、碰撞体和光线投射功能。其中一些已经在 0.13 中落地！

### 渲染 [#](https://bevy.org/news/bevy-0-13/#rendering)

基本形状可以使用网格和 gizmo 进行渲染。在本节中，我们将仔细研究新的 API。

下面，您可以看到使用网格和 gizmo 渲染的长方体和圆环。您可以查看新的[渲染基本形状](https://bevy.org/examples/Math/render-primitives)示例中所有可渲染的基本形状。

![左边：用 gizmo 渲染的长方体。它由 12 条白线组成。右边：用网格渲染的长方体。它由 6 个白色面组成。](https://bevy.org/news/bevy-0-13/cuboids.png)

![左边：用 gizmo 渲染的圆环。它由许多小环组成，全部由 4 个大环连接。右边：用网格渲染的圆环。一个看起来像甜甜圈的形状。](https://bevy.org/news/bevy-0-13/tori.png)

#### 网格化 [#](https://bevy.org/news/bevy-0-13/#meshing)

作者：@Jondolf

以前版本的 Bevy 有诸如 [`Quad`](https://docs.rs/bevy/0.13.0/bevy/prelude/shape/struct.Quad.html)、[`Box`](https://docs.rs/bevy/0.13.0/bevy/prelude/shape/struct.Box.html) 和 [`UVSphere`](https://docs.rs/bevy/0.13.0/bevy/prelude/shape/struct.UVSphere.html) 之类的类型，用于从基本形状创建网格。这些已被弃用，取而代之的是使用新的几何基本形状的构建器式 API。

支持网格化的基本形状实现了 [`Meshable`](https://docs.rs/bevy/0.13.0/bevy/prelude/trait.Meshable.html) trait。对于某些形状，[`mesh`](https://docs.rs/bevy/0.13.0/bevy/prelude/trait.Meshable.html#tymethod.mesh) 方法直接返回一个 [`Mesh`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Mesh.html)：

```rust
let before = Mesh::from(Quad::new(Vec2::new(2.0, 1.0)));
let after = Rectangle::new(2.0, 1.0).mesh(); // Mesh::from 也支持
```

但对于大多数基本形状，它返回一个构建器以进行可选配置：

```rust
// 创建一个具有指定顶点数的圆形网格
let before = Mesh::from(Circle {
    radius: 1.0,
    vertices: 64,
});
let after = Circle::new(1.0).mesh().resolution(64).build();
```

以下是使用新基本形状进行网格化的更多示例。

```rust
// 二十面体
let before = meshes.add(
    Mesh::try_from(Icosphere {
        radius: 2.0,
        subdivisions: 8,
    })
    .unwrap()
);
let after = meshes.add(Sphere::new(2.0).mesh().ico(8).unwrap());

// 长方体
// （注意 Assets::add 现在也会自动处理网格转换）
let before = meshes.add(Mesh::from(shape::Box::new(2.0, 1.0, 1.0)));
let after = meshes.add(Cuboid::new(2.0, 1.0, 1.0));

// 平面
let before = meshes.add(Mesh::from(Plane::from_size(5.0)));
let after = meshes.add(Plane3d::default().mesh().size(5.0, 5.0));
```

随着基本形状的加入，更多的形状也支持网格化，例如 [`Ellipse`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Ellipse.html)、[`Triangle2d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Triangle2d.html) 和 [`Capsule2d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Capsule2d.html)。但是，请注意并非所有基本形状都实现了网格化，例如 [`Polygon`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Polygon.html) 和 [`Cone`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Cone.html)。

下面您可以在 [`2d_shapes`](https://bevy.org/examples/2D%20Rendering/2d-shapes/) 和 [`3d_shapes`](https://bevy.org/examples/3D%20Rendering/3d-shapes/) 示例中看到一些网格。

![一个包含 2D 网格形状的示例](https://bevy.org/news/bevy-0-13/2d_shapes.png)

![一个包含 3D 网格形状的示例](https://bevy.org/news/bevy-0-13/3d_shapes.png)

网格形状维度的一些默认值也已更改，以保持更加一致。

#### Gizmo [#](https://bevy.org/news/bevy-0-13/#gizmos)

作者：@RobWalt

基本形状也可以使用 [`Gizmos`](https://docs.rs/bevy/0.13.0/bevy/gizmos/prelude/struct.Gizmos.html) 进行渲染。有两个新的泛型方法：

- [`gizmos.primitive_2d(primitive, position, angle, color)`](https://docs.rs/bevy/0.13.0/bevy/gizmos/prelude/trait.GizmoPrimitive2d.html)
- [`gizmos.primitive_3d(primitive, position, rotation, color)`](https://docs.rs/bevy/0.13.0/bevy/gizmos/prelude/trait.GizmoPrimitive2d.html)

一些基本形状可以有额外的配置选项，类似于现有的 [`Gizmos`](https://docs.rs/bevy/0.13.0/bevy/gizmos/prelude/struct.Gizmos.html) 绘制方法。例如，使用 [`Sphere`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Sphere.html) 调用 [`primitive_3d`](https://docs.rs/bevy/0.13.0/bevy/gizmos/prelude/trait.GizmoPrimitive2d.html) 会返回一个 [`SphereBuilder`](https://docs.rs/bevy/0.13.0/bevy/gizmos/primitives/dim3/struct.SphereBuilder.html)，它提供了一个 `segments` 方法来控制球体的细节级别。

```rust
let sphere = Sphere { radius };
gizmos
    .primitive_3d(sphere, center, rotation, color)
    .segments(segments);
```

### 包围体 [#](https://bevy.org/news/bevy-0-13/#bounding-volumes)

作者：@NiseVoid, @Jondolf

在游戏开发中，空间检查有几种有价值的用例，例如获取摄像机视锥体中或玩家附近的所有实体，或查找可能相交的物理对象对。为了加速此类检查，使用包围体来近似更复杂的形状。

**Bevy 0.13** 添加了一些新的公开可用的包围体：[`Aabb2d`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.Aabb2d.html)、[`Aabb3d`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.Aabb3d.html)、[`BoundingCircle`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.BoundingCircle.html) 和 [`BoundingSphere`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.BoundingSphere.html)。这些可以手动创建，也可以从基本形状生成。

每个包围体都实现了 [`BoundingVolume`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/trait.BoundingVolume.html) trait，提供一些通用功能和辅助方法。[`IntersectsVolume`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/trait.IntersectsVolume.html) trait 可用于测试与这些包围体的相交。这个 trait 是为包围体本身实现的，所以你可以测试它们之间的相交。这在所有现有的包围体类型之间都受支持，但仅限于同一维度。

以下是包围体如何构建以及如何执行相交测试的示例：

```rust
// 我们创建一个轴对齐的包围盒，以 position 为中心
let position = Vec2::new(100., 50.);
let half_size = Vec2::splat(20.);
let aabb = Aabb2d::new(position, half_size);

// 我们创建一个以 position 为中心的包围圆
let position = Vec2::new(80., 70.);
let radius = 30.;
let bounding_circle = BoundingCircle::new(position, radius);

// 我们检查包围体是否相交
let intersects = bounding_circle.intersects(&aabb);
```

还有两个用于生成包围体的 trait：[`Bounded2d`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/trait.Bounded2d.html) 和 [`Bounded3d`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/trait.Bounded3d.html)。这些是为新的基本形状实现的，因此您可以轻松计算它们的包围体：

```rust
// 我们创建一个基本形状，这里是一个六边形
let hexagon = RegularPolygon::new(50., 6);

let translation = Vec2::new(50., 200.);
let rotation = PI / 2.; // 以弧度表示的旋转角度

// 现在我们可以从这个基本形状获取 Aabb2d 或 BoundingCircle。
// 这些方法是 Bounded2d trait 的一部分。
let aabb = hexagon.aabb_2d(translation, rotation);
let circle = hexagon.bounding_circle(translation, rotation);
```

#### 射线投射和体素投射 [#](https://bevy.org/news/bevy-0-13/#ray-casting-and-volume-casting)

包围体还支持基本的射线投射和体素投射。射线投射测试一个包围体是否与给定的射线相交，该射线从原点沿某个方向投射，直到最大距离。体素投射的工作原理类似，但就像是沿着射线移动一个包围体。

此功能通过新的 [`RayCast2d`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.RayCast2d.html)、[`RayCast3d`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.RayCast3d.html)、[`AabbCast2d`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.AabbCast2d.html)、[`AabbCast3d`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.AabbCast3d.html)、[`BoundingCircleCast`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.BoundingCircleCast.html) 和 [`BoundingSphereCast`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.BoundingSphereCast.html) 类型提供。它们可用于检查与包围体的相交，并计算从投射原点到交点的距离。

下面，您可以看到射线投射、体素投射和相交测试的实际效果：

为了使不同维度的射线投射更容易推理，旧的 [`Ray`](https://docs.rs/bevy/0.12.1/bevy/math/struct.Ray.html) 类型也被拆分为 [`Ray2d`](https://docs.rs/bevy/0.13.0/bevy/math/struct.Ray2d.html) 和 [`Ray3d`](https://docs.rs/bevy/0.13.0/bevy/math/struct.Ray3d.html)。新的 [`Direction2d`](https://docs.rs/bevy/0.13.0/bevy/math/primitives/struct.Direction2d.html) 和 [`Direction3d`](https://docs.rs/bevy/0.13.0/bevy/math/primitives/struct.Direction3d.html) 类型用于确保射线方向保持归一化，提供了向量始终是单位长度的类型级保证。这些已经在其他一些 API 中使用，例如某些基本形状和 gizmo 方法。

## 系统步进 [#](https://bevy.org/news/bevy-0-13/#system-stepping)

作者：@dmlary

**Bevy 0.13** 添加了对**系统步进**的支持，它为系统增加了调试器式的执行控制。

[`Stepping`](https://docs.rs/bevy/0.13.0/bevy/ecs/schedule/stepping/Stepping.html) 资源控制调度器中哪些系统在每一帧执行，并提供步进、断点和继续功能以实现实时调试。

```rust
let mut stepping = Stepping::new();
```

你将想要步进调试的调度器添加到 [`Stepping`](https://docs.rs/bevy/0.13.0/bevy/ecs/schedule/stepping/Stepping.html) 资源中。这些调度器中的系统可以被视为"步进帧"。在"步进帧"中的系统不会运行，除非发生相关的步进或继续操作。未添加的调度器将在每次更新时正常运行，即使在步进期间也是如此。这使得渲染等核心功能能够继续工作。

```rust
stepping.add_schedule(Update);
stepping.add_schedule(FixedUpdate);
```

步进默认是禁用的，即使插入了资源也是如此。要在应用中启用它，功能标志、开发者控制台和隐藏的热键都能很好地工作。

```rust
#[cfg(feature = "my_stepping_flag")]
stepping.enable();
```

最后，你将 [`Stepping`](https://docs.rs/bevy/0.13.0/bevy/ecs/schedule/stepping/Stepping.html) 资源添加到 ECS [`World`](https://docs.rs/bevy/0.13.0/bevy/ecs/world/struct.World.html) 中。

```rust
app.insert_resource(stepping);
```

### 系统步进和继续帧 [#](https://bevy.org/news/bevy-0-13/#system-step-continue-frame)

"步进帧"操作运行步进光标处的系统，并在下一个渲染帧期间推进光标。这对于查看系统所做的单个更改以及在执行系统之前查看世界状态非常有用。

```rust
stepping.step_frame()
```

"继续帧"操作将在下一帧期间从步进光标开始执行系统，直到步进帧的末尾。如果遇到设置了断点的系统，它可能会在步进帧结束之前停止。这对于快速前进通过整个帧、到达下一帧的开始，或与断点结合使用非常有用。

```rust
stepping.continue_frame()
```

这个视频演示了这些操作在 breakout 示例上使用自定义 `egui` 界面的效果。当我们点击 `step` 按钮时，可以看到步进光标在系统列表中移动。当点击 `continue` 按钮时，你可以看到游戏每次点击前进一个步进帧。

### 断点 [#](https://bevy.org/news/bevy-0-13/#breakpoints)

当一个调度器增长到一定程度时，仅仅为了查看几个系统的效果而步进通过调度器中的每个系统可能需要很长时间。在这种情况下，步进提供了系统断点。

这个视频说明了在 `check_for_collisions()` 上设置断点如何与"step"和"continue"操作配合：

### 在步进期间禁用系统 [#](https://bevy.org/news/bevy-0-13/#disabling-systems-during-stepping)

在调试期间，禁用系统以缩小问题来源的范围可能很有帮助。`Stepping::never_run()` 和 `Stepping::never_run_node()` 可用于在启用步进时禁用系统。

### 从步进中排除系统 [#](https://bevy.org/news/bevy-0-13/#excluding-systems-from-stepping)

可能需要确保某些系统在启用步进时仍然运行。虽然最佳实践是将它们放在一个未添加到 `Stepping` 资源的调度器中，但也可以将系统配置为在启用步进时始终运行。这对于事件和输入处理系统尤其有用。

可以通过调用 `Stepping::always_run()` 或 `Stepping::always_run_node()` 将系统配置为始终运行。当一个系统被配置为始终运行时，它将在每个渲染帧运行，即使启用了步进。

### 局限性 [#](https://bevy.org/news/bevy-0-13/#limitations)

在这个初始步进实现中有一些局限性：

- **读取事件的系统可能无法正确步进**：因为在启用步进时帧仍然正常推进，事件可能在步进系统读取它们之前被清除。这里的最佳方法是将基于事件的系统配置为始终运行，或者将它们放在未添加到 `Stepping` 的调度器中。与断点一起使用"Continue"也可能在这种情况下工作。
- **条件系统在步进时可能无法按预期运行**：与基于事件的系统类似，如果运行条件只在短时间内为真，则系统在步进时可能不会运行。

### 详细示例 [#](https://bevy.org/news/bevy-0-13/#detailed-examples)

- [基于文本的步进示例](https://github.com/bevyengine/bevy/blob/main/examples/ecs/system_stepping.rs)
- 非交互式 [bevy UI 示例步进插件](https://github.com/bevyengine/bevy/blob/main/examples/games/stepping.rs)，用于 breakout 示例
- 演示视频中使用的交互式 [egui 步进插件](https://gist.github.com/dmlary/3fd57ebf1f88bb9afa8a6604737dac97)

## 相机曝光 [#](https://bevy.org/news/bevy-0-13/#camera-exposure)

作者：@superdump (Rob Swain), @JMS55, @cart

在现实世界中，相机捕获的图像的亮度由其曝光决定：相机传感器或胶片吸收的光量。这由相机的几个机制控制：

- **光圈（Aperture）**：以光圈值（F-Stops）测量，光圈打开和关闭以控制允许进入相机传感器或胶片的光量，通过物理阻挡特定角度的光线，类似于眼睛的瞳孔。
- **快门速度（Shutter Speed）**：相机快门打开的时间长度，即相机传感器或胶片暴露在光线下的持续时间。
- **ISO 感光度（ISO Sensitivity）**：相机传感器或胶片对光的敏感程度。较高的值表示对光更敏感。

这些中的每一个都在最终图像接收到的光量中发挥作用。它们可以组合成一个最终的 EV 数字（曝光值），例如半标准的 EV100（ISO 100 的曝光值）。较高的 EV100 数字意味着需要更多的光线才能获得相同的结果。例如，晴天场景可能需要大约 15 的 EV100，而光线昏暗的室内场景可能需要大约 7 的 EV100。

在 **Bevy 0.13** 中，你现在可以使用新的 [`Exposure`](https://docs.rs/bevy/0.13.0/bevy/render/camera/struct.Exposure.html) 组件在每相机基础上配置 EV100。你可以直接使用 [`Exposure::ev100`](https://docs.rs/bevy/0.13.0/bevy/render/camera/struct.Exposure.html#structfield.ev100) 字段设置它，或者你可以使用新的 [`PhysicalCameraParameters`](https://docs.rs/bevy/0.13.0/bevy/render/camera/struct.PhysicalCameraParameters.html) 结构体，使用"真实世界"的相机设置（如光圈值、快门速度和 ISO 感光度）来计算 ev100。

这很重要，因为 Bevy 的"基于物理"的渲染器（PBR）有意立足于现实。我们的目标是让人们能够在他们的灯光和材质中使用现实世界的单位，并让它们的行为尽可能接近现实。

拖动此图像进行比较

![EV100 9.7](https://bevy.org/news/bevy-0-13/exposure_97.jpg)![EV100 15](https://bevy.org/news/bevy-0-13/exposure_15.jpg)

请注意，以前版本的 Bevy 为其某些光照类型硬编码了静态的 EV100。在 **Bevy 0.13** 中，它是可配置的，并且在所有光照类型中保持一致。我们还将默认 EV100 提高到 9.7，这是[我们选择的最佳匹配 Blender 默认曝光的数字](https://github.com/bevyengine/bevy/issues/11577#issuecomment-1942873507)。它恰好也是一个很好的"中间值"，介于室内照明和多云室外照明之间。

你可能会注意到点光源现在需要_显著_更高的强度值（以流明为单位）。这种（有时）百万流明的值可能感觉过高。只需安慰自己：（1）在阴天的室外环境中，确实需要大量的光才能有意义地记录下来，（2）Blender 以这种规模导出灯光（并且我们已校准为尽可能接近它们）。

## 相机驱动 UI [#](https://bevy.org/news/bevy-0-13/#camera-driven-ui)

作者：@bardt, @oceantume

历史上，Bevy 的 UI 元素是在主窗口的上下文中缩放和定位的，而不管相机设置如何。这种方法使得某些 UI 体验（如分屏多人游戏）难以实现，而其他体验（如在多个窗口中拥有 UI）则完全不可能。

**Bevy 0.13** 引入了**相机驱动 UI**。每个相机现在都可以拥有自己的 UI 根节点，根据其视口、缩放因子和目标（可以是辅助窗口甚至纹理）进行渲染。

这一变化解锁了各种新的 UI 体验，包括分屏多人游戏、多个窗口中的 UI、在 3D 世界中显示非交互式 UI 等等。

![具有独立 UI 根节点的分屏画面](https://bevy.org/news/bevy-0-13/split-screen.png)

如果世界中只有一个相机，你不需要做任何事情；你的 UI 将显示在该相机的视口中。

```rust
commands.spawn(Camera3dBundle {
    // 相机可以拥有自定义视口、目标等。
});
commands.spawn(NodeBundle {
    // UI 将被渲染到单个相机的视口
});
```

当需要更多控制，或者有多个相机时，我们引入了 [`TargetCamera`](https://docs.rs/bevy/0.13.0/bevy/ui/struct.TargetCamera.html) 组件。这个组件可以添加到根 UI 节点，以指定它应该被渲染到哪个相机。

```rust
// 对于分屏多人游戏，我们设置 2 个相机和 2 个 UI 根节点
let left_camera = commands.spawn(Camera3dBundle {
    // 视口设置为屏幕左半部分
}).id();

commands
    .spawn((
        TargetCamera(left_camera),
        NodeBundle {
            //...
        }
    ));

let right_camera = commands.spawn(Camera3dBundle {
    // 视口设置为屏幕右半部分
}).id();

commands
    .spawn((
        TargetCamera(right_camera),
        NodeBundle {
            //...
        })
    );
```

随着这一变化，我们还移除了 [`UiCameraConfig`](https://docs.rs/bevy/0.12.1/bevy/ui/camera_config/struct.UiCameraConfig.html) 组件。如果你曾使用它来隐藏 UI 节点，你可以通过在根节点上配置 [`Visibility`](https://docs.rs/bevy/0.13.0/bevy/render/view/enum.Visibility.html) 组件来实现相同的效果。

```rust
commands.spawn(Camera3dBundle::default());
commands.spawn(NodeBundle {
    visibility: Visibility::Hidden, // UI 将被隐藏
    // ...
});
```

## 纹理切片和平铺 [#](https://bevy.org/news/bevy-0-13/#texture-slicing-and-tiling)

作者：@ManevilleF

3D 渲染得到了很多关注，但 2D 功能也同样重要！我们很高兴在 **Bevy 0.13** 中将基于 CPU 的_切片和平铺_添加到 `bevy_sprite` 和 `bevy_ui` 中！

此行为由一个新的可选组件控制：[`ImageScaleMode`](https://docs.rs/bevy/0.13.0/bevy/prelude/enum.ImageScaleMode.html)。

### 9 宫格切片 [#](https://bevy.org/news/bevy-0-13/#9-slicing)

向具有精灵或 UI bundle 的实体添加 `ImageScaleMode::Sliced` 可启用 [9 宫格切片](https://en.wikipedia.org/wiki/9-slice_scaling)，在调整大小时保持图像比例，避免纹理拉伸。

![拉伸 vs 切片纹理](https://bevy.org/news/bevy-0-13/slice_vs_stretched.png)

这对于 UI 非常有用，允许你的漂亮纹理即使在元素大小改变时也能看起来正确。

![切片按钮](https://bevy.org/news/bevy-0-13/ui_slice.png)

边框纹理来自 [Kenney](https://kenney.nl/assets/fantasy-ui-borders)

```rust
commands.spawn((
    SpriteSheetBundle::default(),
    ImageScaleMode::Sliced(TextureSlicer {
        // 图像边框在各个方向上都是 20 像素
        border: BorderRect::square(20.0),
        // 我们不会将角落拉伸超过其实际大小（20px）
        max_corner_scale: 1.0,
        ..default()
    }),
));
```

### 平铺 [#](https://bevy.org/news/bevy-0-13/#tiling)

向你的 2D 精灵实体添加 `ImageMode::Tiled { .. }` 可启用_纹理平铺_：重复图像直到填满整个区域。这通常用于背景和表面。

```rust
commands.spawn((
    SpriteSheetBundle::default(),
    ImageScaleMode::Tiled {
        // 图像将在水平方向重复
        tile_x: true,
        // 图像将在垂直方向重复
        tile_y: true,
        // 如果绘制矩形大于图像大小，纹理将重复
        stretch_value: 1.0,
    },
));
```

## 动态查询 [#](https://bevy.org/news/bevy-0-13/#dynamic-queries)

作者：@james-j-obrien, @jakobhellermann, @Suficio

在 Bevy ECS 中，查询使用类型驱动的 DSL。查询的完整类型（要访问的组件，要使用的过滤器）必须在编译时指定。

有时我们无法在编译时知道查询想要访问的数据。某些场景根本无法使用静态查询：

- 在 Lua 或 JavaScript 等脚本语言中定义查询。
- 从脚本语言定义新的组件并查询它们。
- 向实体检查器（如 [`bevy-inspector-egui`](https://crates.io/crates/bevy-inspector-egui)）添加运行时过滤器。
- 添加[类似 Quake 的控制台](https://github.com/doonv/bevy_dev_console)，以便在运行时从提示符修改或查询组件。
- 创建[具有远程能力的编辑器](https://makeshift-bevy-web-editor.vercel.app/)。

动态查询使所有这些成为可能。而这些还只是我们目前听说过的计划！

定义 [`Query`](https://docs.rs/bevy/0.13.0/bevy/ecs/system/struct.Query.html) 的标准方法是将其用作系统参数：

```rust
fn take_damage(mut player_health: Query<(Entity, &mut Health), With<Player>>) {
    // ...
}
```

**这不会改变。** 对于大多数（如果不是全部）游戏玩法用例，你将继续愉快地使用简单至极的 [`Query`](https://docs.rs/bevy/0.13.0/bevy/ecs/system/struct.Query.html) API。

然而，考虑这种情况：作为游戏或 mod 开发者，我想通过文本提示列出具有特定组件的实体。类似于 Quake 控制台的工作方式。那会是什么样子？

```rust
#[derive(Resource)]
struct UserQuery(String);

// user_query 由用户在游戏运行时以文本提示形式输入。
// 在系统中，很明显我们不能使用 `Query`。
fn list_entities_system(user_query: Res<UserQuery>, query: Query<FIXME, With<FIXME>>) {}

// 即使使用更高级的 `World` API，我们也无法继续。
fn list_entities(user_query: String, world: &mut World) {
    // FIXME: 这里放什么类型？
    let query = world.query::<FIXME>();
}
```

根据 `user_query` 的值来选择类型是不可能的！[`QueryBuilder`](https://docs.rs/bevy/0.13.0/bevy/ecs/prelude/struct.QueryBuilder.html) 解决了这个问题。

```rust
fn list_entities(
    user_query: String,
    type_registry: &TypeRegistry,
    world: &mut World,
) -> Option<()> {
    let name = user_query.split(' ').next()?;
    let type_id = type_registry.get_with_short_type_path(name)?.type_id();
    let component_id = world.components().get_id(type_id)?;

    let query = QueryBuilder::<FilteredEntityRef>::new(&mut world)
        .ref_id(component_id)
        .build();

    for entity_ref in query.iter(world) {
        let ptr = entity_ref.get_by_id(component_id);
        // 将 `ptr` 转换为 `&dyn Reflect` 并使用它。
    }
    Some(())
}
```

它仍然是一个容易出错、复杂且不安全的 API，但它使以前不可能的事情成为可能。我们期望第三方 crate 为 `QueryBuilder` API 提供方便的包装器，其中一些无疑会向上游合并。

## 查询变换 [#](https://bevy.org/news/bevy-0-13/#query-transmutation)

作者：@hymm, james-j-obrien

你是否曾经想过将一个查询传递给一个函数，但你没有 `Query<&Transform>`，而是有一个 `Query<(&Transform, &Velocity), With<Enemy>>`？在 **Bevy 0.13** 中，你可以做到，这要归功于新的 [`QueryLens`](https://docs.rs/bevy/0.13.0/bevy/ecs/system/struct.QueryLens.html) 和 [`Query::transmute_lens()`](https://docs.rs/bevy/0.13.0/bevy/ecs/system/struct.Query.html#method.transmute_lens) 方法。

查询变换允许你将查询更改为不同的查询类型，只要访问的组件是原始查询的子集。如果你确实尝试访问原始查询中不存在的数据，此方法将 panic。

```rust
fn reusable_function(lens: &mut QueryLens<&Transform>) {
    let query = lens.query();
    // 对查询做一些事情...
}

// 我们可以在一个接受精确查询的系统中使用该函数。
fn system_1(mut query: Query<&Transform>) {
    reusable_function(&mut query.as_query_lens());
}

// 我们也可以将其用于不完全匹配的查询，
// 通过变换它。
fn system_2(mut query: Query<(&mut Transform, &Velocity), With<Enemy>>) {
    let mut lens = query.transmute_lens::<&Transform>();
    reusable_function(&mut lens);
}
```

请注意，[`QueryLens`](https://docs.rs/bevy/0.13.0/bevy/ecs/system/struct.QueryLens.html) 仍然会迭代与派生它的原始 [`Query`](https://docs.rs/bevy/0.13.0/bevy/ecs/system/struct.Query.html) 相同的实体。从 `Query<(&Transform, &Velocity)>` 获取的 `QueryLens<&Transform>` 将只包含同时具有 `Transform` 和 `Velocity` 组件的实体的 `Transform`。

除了移除参数之外，你还可以以有限的方式将它们更改为不同的智能指针类型。其中最有用的一个是将 `&mut` 更改为 `&`。有关更多详细信息，请参阅[文档](https://docs.rs/bevy/latest/bevy/ecs/system/struct.Query.html#method.transmute_lens)。

需要考虑到的是，变换不是免费的。它通过创建一个新状态并复制原始查询内部的缓存数据来工作。这不是一个昂贵的操作，但你应该避免在热循环中这样做。

## `WorldQuery` Trait 拆分 [#](https://bevy.org/news/bevy-0-13/#worldquery-trait-split)

作者：@wainwrightmark @taizu-jin

一个 [`Query`](https://docs.rs/bevy/0.13.0/bevy/ecs/system/struct.Query.html) 有两个类型参数：一个用于要获取的数据，第二个可选的用于过滤器。

在以前版本的 Bevy 中，两个参数都只需 [`WorldQuery`](https://docs.rs/bevy/0.12.0/bevy/ecs/query/trait.WorldQuery.html)：没有什么能阻止你将用作过滤器的类型放在数据位置（或反之亦然）。

除了使 [`Query`](https://docs.rs/bevy/0.13.0/bevy/ecs/system/struct.Query.html) 项的类型签名更复杂（见下面的示例）之外，这通常工作正常，因为大多数过滤器在任一位置的行为都是相同的。

不幸的是，[`Changed`](https://docs.rs/bevy/0.13.0/bevy/ecs/query/struct.Changed.html) 和 [`Added`](https://docs.rs/bevy/0.13.0/bevy/ecs/query/struct.Added.html) 并非如此，它们在数据位置具有不同（且未记录）的行为，这可能导致用户代码中的错误。

为了允许我们在编译时防止这种类型的错误，[`WorldQuery`](https://docs.rs/bevy/0.12.0/bevy/ecs/query/trait.WorldQuery.html) trait 已被两个 trait 取代：[`QueryData`](https://docs.rs/bevy/0.13.0/bevy/ecs/query/trait.QueryData.html) 和 [`QueryFilter`](https://docs.rs/bevy/0.13.0/bevy/ecs/query/trait.QueryFilter.html)。[`Query`](https://docs.rs/bevy/0.13.0/bevy/ecs/system/struct.Query.html) 的数据参数现在必须是 [`QueryData`](https://docs.rs/bevy/0.13.0/bevy/ecs/query/trait.QueryData.html)，过滤器参数必须是 [`QueryFilter`](https://docs.rs/bevy/0.13.0/bevy/ecs/query/trait.QueryFilter.html)。

大多数用户代码应该不受影响或易于迁移。

```rust
// 可能是一个微妙的 Bug：`With` 过滤器在数据位置 - 在 0.13 中无法编译
fn my_system(query: Query<(Entity, With<ComponentA>)>)
{
    // 查询项的类型签名是 `(Entity, ())`，可用但笨重
  for (entity, _) in query.iter(){
  }
}

// 惯用写法，在 0.12 和 0.13 中都能编译
fn my_system(query: Query<Entity, With<ComponentA>>)
{
  for entity in query.iter(){
  }
}
```

## 自动插入 `apply_deferred` 系统 [#](https://bevy.org/news/bevy-0-13/#automatically-insert-apply-deferred-systems)

作者：@hymm

在编写游戏玩法代码时，你可能经常遇到一个系统希望立即看到另一个系统中排队的命令的效果。在 **Bevy 0.13** 之前，你必须在两者之间手动插入一个 `apply_deferred` 系统，这是一个特殊的系统，它会在遇到时使这些命令被应用。Bevy 现在会检测何时一个带有命令的系统相对于其他系统排序，并自动为你插入 `apply_deferred`。

```rust
// 0.13 之前
app.add_systems(
    Update,
    (
        system_with_commands,
        apply_deferred,
        another_system,
    ).chain()
);
```

```rust
// 0.13 之后
app.add_systems(
    Update,
    (
        system_with_commands,
        another_system,
    ).chain()
);
```

这解决了一个常见的初学者陷阱：如果两个系统已排序，第二个系统难道不应该总是看到第一个系统的结果吗？

自动插入的 `apply_deferred` 系统通过尽可能自动合并来进行优化。在大多数情况下，建议移除所有手动插入的 `apply_deferred` 系统，因为允许 Bevy 根据需要插入和合并这些系统通常既更快又涉及更少的样板代码。

```rust
// 这将只添加一个 apply_deferred 系统。
app.add_systems(
    Update,
    (
        (system_1_with_commands, system_2).chain(),
        (system_3_with_commands, system_4).chain(),
    )
);
```

如果这个新行为对你不适用，请查阅迁移指南。有几个新的 API 允许你选择退出。

## 更灵活的一次性系统 [#](https://bevy.org/news/bevy-0-13/#more-flexible-one-shot-systems)

作者：@Nathan-Fenner

在 **Bevy 0.12** 中，我们引入了[一次性系统](https://bevy.org/news/bevy-0-12/#one-shot-systems)，这是一种按需调用系统的便捷方式，无需将它们添加到调度器中。最初的实现在哪些系统可以和不能用作一次性系统方面存在一些限制。在 **Bevy 0.13** 中，这些限制已被解决。

一次性系统现在支持输入和输出。

```rust
fn increment_sys(In(increment_by): In<i32>, mut counter: ResMut<Counter>) -> i32 {
    counter.0 += increment_by;
    counter.0
}

let mut world = World::new();
let id = world.register_system(increment_sys);

world.insert_resource(Counter(1));
let count_one = world.run_system_with_input(id, 5).unwrap(); // 计数器增加 5，返回 6
let count_two = world.run_system_with_input(id, 2).unwrap(); // 计数器增加 2，返回 8
```

运行系统现在返回系统输出作为 `Ok(output)`。请注意，由于其延迟性质，通过命令调用一次性系统时无法返回输出。

独占系统现在可以注册为一次性系统：

```rust
world.register_system(|world: &mut World| { /* 做任何事情 */ });
```

Boxed 系统现在可以使用 `register_boxed_system` 注册。

这些改进全面完善了一次性系统：它们现在应该像任何其他 Bevy 系统一样工作。

## wgpu 0.19 升级和渲染性能改进 [#](https://bevy.org/news/bevy-0-13/#wgpu-0-19-upgrade-and-rendering-performance-improvements)

作者：@Elabajaba, @JMS55

在 **Bevy 0.13** 中，我们将 `wgpu` 从 0.17 升级到了 0.19，其中包括期待已久的 `wgpu` [arcanization](https://gfx-rs.github.io/2023/11/24/arcanization.html)，这使我们能够[异步编译着色器](https://github.com/bevyengine/bevy/pull/10812)以避免着色器编译卡顿，以及[多线程绘制调用创建](https://github.com/bevyengine/bevy/pull/9172)以在 CPU 受限的场景中获得更好的性能。

由于 wgpu 0.19 中的变化，我们在 Bevy 中添加了一个新的 `webgpu` 功能，这是在针对 WebGPU 进行 WebAssembly 构建时所必需的。针对 WebGPU 时，不再需要禁用 `webgl2` 功能，但新的 `webgpu` 功能在启用时当前会覆盖 `webgl2` 功能。库作者，请不要默认启用 `webgpu` 功能。未来我们计划允许你在同一个 WebAssembly 二进制文件中同时支持 WebGL2 和 WebGPU，但我们还没有完全实现。

我们交换了材质和网格的绑定组，因此网格数据现在位于绑定组 1，材质数据位于绑定组 2。这结合更改不透明通道的排序函数以按渲染管线和网格排序，大大提高了我们的绘制调用批处理。以前我们是按距离相机的距离排序的。这些批处理改进意味着我们进行的绘制调用更少，从而提高了 CPU 性能，尤其是在更大的场景中。我们还移除了着色器中的 `get_instance_index` 函数，因为它只是为了解决一个上游 Bug，该 Bug 已在 wgpu 0.19 中修复。对于其他着色器或渲染更改，请参阅[迁移指南](https://bevy.org/learn/migration-guides/0-12-to-0-13/)和 [wgpu 的变更日志](https://github.com/gfx-rs/wgpu/blob/v0.19/CHANGELOG.md)。

许多对 Bevy 和 `wgpu` 的小改动加在一起，在真实的 3D 场景中对我们的性能产生了适度但可衡量的提升！我们在同一台机器上对四个复杂场景进行了快速测试，比较了 **Bevy 0.12** 和 **Bevy 0.13**：[Bistro](https://github.com/DGriffin91/bevy_bistro_scene)、[Sponza](https://github.com/DGriffin91/bevy_sponza_scene)、[San Miguel](https://github.com/DGriffin91/bevy_san_miguel_scene) 和 [Hidden Alley](https://blog.polyhaven.com/hidden-alley/)。

![一个高多边形、逼真光照的美丽咖啡馆截图，背景有一棵树。](https://bevy.org/news/bevy-0-13/San_Miguel_13.jpg)

正如你所见，这些场景比大多数视频游戏环境要详细得多，但那个截图是在 Bevy 中以超过 60 FPS、1440p 分辨率渲染的！在 Bevy 0.12 和 Bevy 0.13 之间，我们在测试的场景中看到帧时间减少了约 5-10%。干得好！

![一个图表显示不同场景的帧率。Bistro 从 90 FPS 提高到 120 FPS，而其他场景略有改善。所有场景都在 60-120 FPS 之间。](https://bevy.org/news/bevy-0-13/rendering-perf-graph.svg)

## 从 RAM 中卸载渲染资源 [#](https://bevy.org/news/bevy-0-13/#unload-rendering-assets-from-ram)

作者：@JMS55, @mockersf, @brianreavis

网格和用于定义其材质的纹理占用大量内存：在许多游戏中，内存使用是分辨率和多边形数量的最大限制因素！此外，将这些数据从系统 RAM（CPU 使用）传输到 VRAM（GPU 使用）可能是一个真正的性能瓶颈。

**Bevy 0.13** 添加了在成功将数据传输到 VRAM 后，从系统 RAM 中卸载这些数据的能力。要为你的资产配置此行为，请设置 [`RenderAssetUsages`](https://docs.rs/bevy/0.13.0/bevy/render/render_asset/struct.RenderAssetUsages.html) 字段，以指定是将数据保留在主（CPU）世界、渲染（GPU）世界，还是两者都保留。

此行为目前对大多数资源类型默认是关闭的，因为它[有一些注意事项](https://github.com/bevyengine/bevy/pull/11212)（因为该资产对 CPU 上的逻辑变得不可用），但我们强烈建议尽可能为你的资源启用它，以获得显著的内存使用优势（我们将来可能会默认启用它）。

纹理图集和字体图集现在只提取实际使用的数据到 VRAM，而不是每帧浪费工作将_所有_可能的图像或字符发送到 VRAM。很整洁！

## 通过更智能的排序实现更好的批处理 [#](https://bevy.org/news/bevy-0-13/#better-batching-through-smarter-sorting)

作者：@Elabajaba

加速渲染的核心技术之一是同时绘制许多相似的对象。在这种情况下，Bevy 已经在使用一种称为"批处理"的技术，它允许我们合并多个相似的操作，减少正在进行的昂贵绘制调用（对 GPU 的指令）的数量。

然而，我们定义这些批次的策略远非最优。以前，我们按到相机的距离排序，然后_再_检查在该排序列表中是否有多个相同的网格相邻。在真实的场景中，这不太可能找到许多可合并的候选者！

在 **Bevy 0.13** 中，我们首先按渲染管线（实际上是使用的材质类型）排序，然后按网格标识排序。这种策略产生更好的批处理，在[测试的真实场景](https://syntystore.com/products/polygon-fantasy-kingdom)中，将整体 FPS 提高了两位数百分比！

![一个图表显示批处理改进。阴影非常昂贵，在所有测试的案例中，FPS 至少提高了 20%。](https://bevy.org/news/bevy-0-13/better_batching.svg)

## 动画插值方法 [#](https://bevy.org/news/bevy-0-13/#animation-interpolation-methods)

作者：@mockersf

通常，动画由其**关键帧**定义：物体在时间线上的位置（和其他状态）的快照。但是在这些关键帧之间发生了什么？游戏引擎需要在它们之间进行**插值**，平滑地从一种状态过渡到下一种状态。

最简单的插值方法是线性插值：动画对象在每单位时间内向下一关键帧移动相等的距离。但这并不总是期望的效果！定格动画风格和更精细平滑的动画都有其适用的场景。

Bevy 现在支持动画中的步进插值和三次样条插值。大多数情况下，这将会从 glTF 文件中正确解析，但在手动设置 [`VariableCurve`](https://docs.rs/bevy/0.13.0/bevy/animation/struct.VariableCurve.html) 时，有一个新的 [`Interpolation`](https://docs.rs/bevy/0.13.0/bevy/animation/enum.Interpolation.html) 字段可以设置。

![演示不同类型的插值](https://bevy.org/news/bevy-0-13/interpolation_methods.gif)

## `Animatable` Trait [#](https://bevy.org/news/bevy-0-13/#animatable-trait)

作者：@james7132

当你想到"动画"时：你可能想象的是物体在空间中移动，来回平移，旋转它们，甚至可能挤压和拉伸它们。但在现代游戏开发中，动画是一组强大的共享工具和概念，用于"随时间改变事物"。变换只是开始：颜色、粒子效果、不透明度，甚至像可见性这样的布尔值都可以被动画化！

在 **Bevy 0.13** 中，我们朝着[这个愿景](https://github.com/bevyengine/rfcs/blob/main/rfcs/51-animation-composition.md)迈出了第一步，引入了 [`Animatable`](https://docs.rs/bevy/0.13.0/bevy/prelude/trait.Animatable.html) trait。

```rust
/// 一个可动画化的值类型。
pub trait Animatable: Reflect + Sized + Send + Sync + 'static {
    /// 在 `a` 和 `b` 之间以 `time` 的插值因子进行插值。
    ///
    /// 这里的 `time` 参数可能不会被限制在 `[0.0, 1.0]` 范围内。
    fn interpolate(a: &Self, b: &Self, time: f32) -> Self;

    /// 混合一个或多个值。
    ///
    /// 当没有输入时，实现者应返回一个默认值。
    fn blend(inputs: impl Iterator<Item = BlendInput<Self>>) -> Self;

    /// 使用 [`World`] 中的资源对值进行后处理。
    /// 大多数可动画化类型不需要实现此方法。
    fn post_process(&mut self, _world: &World) {}
}
```

这是迈向动画混合和资源驱动的动画图的第一步，这对于在 Bevy 中发布大规模 3D 游戏至关重要。但就目前而言，这只是一个构建块。我们已经为几个关键类型（`Transform`、`f32` 和 `glam` 的 `Vec` 类型）实现了这个 trait，并发布了该 trait。将其插入到你的游戏和 crate 中，与其他贡献者合作，帮助 `bevy_animation` 变得和引擎的其他部分一样令人愉快和功能丰富。

## 无扩展名资源支持 [#](https://bevy.org/news/bevy-0-13/#extensionless-asset-support)

作者：@bushrat011899

在以前版本的 Bevy 中，为特定资源选择 [`AssetLoader`](https://docs.rs/bevy/0.13.0/bevy/asset/trait.AssetLoader.html) 的默认方法完全基于文件扩展名。[最近添加的 .meta 文件](https://bevy.org/news/bevy-0-12/#asset-meta-files)允许指定更细粒度的加载行为，但仍然需要文件扩展名。在 **Bevy 0.13** 中，现在可以使用资源类型来推断 [`AssetLoader`](https://docs.rs/bevy/0.13.0/bevy/asset/trait.AssetLoader.html)。

```rust
// 使用 AudioSettingsAssetLoader
let audio = asset_server.load("data/audio.json");

// 使用 GraphicsSettingsAssetLoader
let graphics = asset_server.load("data/graphics.json");
```

这是可能的，因为每个 [`AssetLoader`](https://docs.rs/bevy/0.13.0/bevy/asset/trait.AssetLoader.html) 都必须声明它加载的**类型**，而不仅仅是它支持的扩展名。由于 [`AssetServer`](https://docs.rs/bevy/0.13.0/bevy/asset/struct.AssetServer.html) 上的 [`load`](https://docs.rs/bevy/0.13.0/bevy/asset/struct.AssetServer.html#method.load) 方法已经对要返回的资源类型是泛型的，因此该信息已经可供 [`AssetServer`](https://docs.rs/bevy/0.13.0/bevy/asset/struct.AssetServer.html) 使用。

```rust
// 上面的示例，显示类型
let audio: Handle<AudioSettings> = asset_server.load::<AudioSettings>("data/audio.json");
let graphics: Handle<GraphicsSettings> = asset_server.load::<GraphicsSettings>("data/graphics.json");
```

现在，我们也可以用它来选择 [`AssetLoader`](https://docs.rs/bevy/0.13.0/bevy/asset/trait.AssetLoader.html) 本身。

加载资源时，通过按顺序检查以下内容来选择加载器：

1. 资源 `meta` 文件
2. 要返回的 `Handle<A>` 的类型
3. 文件扩展名

```rust
// 这将从上下文中推断为 glTF 资源，忽略文件扩展名
let gltf_handle = asset_server.load("models/cube/cube.gltf");

// 由于标签的原因，这仍然依赖于文件扩展名
let cube_handle = asset_server.load("models/cube/cube.gltf#Mesh0/Primitive0");
//                                                        ^^^^^^^^^^^^^^^^^
//                                                        | 资源路径标签
```

### 文件扩展名现在是可选的 [#](https://bevy.org/news/bevy-0-13/#file-extensions-are-now-optional)

由于资源类型可用于推断加载器，要加载的文件和 [`AssetLoader`](https://docs.rs/bevy/0.13.0/bevy/asset/trait.AssetLoader.html) 都不需要具有文件扩展名。

```rust
pub trait AssetLoader: Send + Sync + 'static {
    /* 省略 */

    /// 返回此 [`AssetLoader`] 支持的扩展名列表，不带前导点号。
    fn extensions(&self) -> &[&str] {
        // 现在提供了默认实现
        &[]
    }
}
```

以前，没有扩展名的资源加载器使用起来非常麻烦。现在，它们可以像任何其他加载器一样轻松使用。同样，如果文件缺少扩展名，Bevy 现在可以选择合适的加载器。

```rust
let license = asset_server.load::<Text>("LICENSE");
```

为了良好的项目管理，仍然推荐使用合适的文件扩展名，但这现在是一个建议而不是硬性要求。

### 同一资源的多个资源加载器 [#](https://bevy.org/news/bevy-0-13/#multiple-asset-loaders-with-the-same-asset)

现在，只要资源类型不同，单个路径可以被多个资源句柄使用。

```rust
// 加载音效用于播放
let bang = asset_server.load::<AudioSource>("sound/bang.ogg");

// 加载同一音效的原始字节（例如，通过网络发送）
let bang_blob = asset_server.load::<Blob>("sound/bang.ogg");

// 返回 bang 句柄，因为它已经加载过了
let bang_again = asset_server.load::<AudioSource>("sound/bang.ogg");
```

请注意，上面的示例为清晰起见使用了 [turbofish](https://turbo.fish/) 语法。在实践中，这并非必需，因为加载的资源类型通常可以在调用点推断出来。

```rust
#[derive(Resource)]
struct SoundEffects {
    bang: Handle<AudioSource>,
    bang_blob: Handle<Blob>,
}

fn setup(mut effects: ResMut<SoundEffects>, asset_server: Res<AssetServer>) {
    effects.bang = asset_server.load("sound/bang.ogg");
    effects.bang_blob = asset_server.load("sound/bang.ogg");
}
```

[`custom_asset` 示例](https://bevy.org/examples/Assets/custom-asset/)已经更新以演示这些新功能。

## 纹理图集重构 [#](https://bevy.org/news/bevy-0-13/#texture-atlas-rework)

作者：@ManevilleF

纹理图集高效地将多个图像组合成一个较大的单一纹理，称为图集。

**Bevy 0.13** 对其进行了重大重构，以减少样板代码并使其更加数据导向。告别 `TextureAtlasSprite` 和 `UiTextureAtlasImage` 组件（以及它们对应的 `Bundle` 类型）。现在，通过向普通的精灵和图像实体添加一个_额外的_组件来启用纹理图集：[`TextureAtlas`](https://docs.rs/bevy/0.13.0/bevy/sprite/struct.TextureAtlas.html)。

### 为什么？ [#](https://bevy.org/news/bevy-0-13/#why)

纹理图集（有时称为精灵表）只是绘制给定纹理的自定义_区域_。这_仍然是_类似精灵或类似图像的行为，我们只是绘制一个子集。新的 [`TextureAtlas`](https://docs.rs/bevy/0.13.0/bevy/sprite/struct.TextureAtlas.html) 组件通过存储以下内容来拥抱这一点：

- 一个 `Handle<TextureAtlasLayout>`，一个将索引映射到纹理的 `Rect` 区域的资源
- 一个 `usize` 索引，定义我们要显示布局的哪个区域 `Rect`

## 灯光 `RenderLayers` [#](https://bevy.org/news/bevy-0-13/#light-renderlayers)

作者：@robftm

[`RenderLayers`](https://docs.rs/bevy/latest/bevy/render/view/struct.RenderLayers.html) 是 Bevy 通过过滤相机可以看到的内容来快速批量隐藏和显示实体的解决方案……非常适合自定义角色所持物品的第一人称视角（或确保吸血鬼不会出现在你的镜子里！）等用途。

[`RenderLayers`](https://docs.rs/bevy/latest/bevy/render/view/struct.RenderLayers.html) [现在与灯光兼容](https://github.com/bevyengine/bevy/pull/10742)，修复了一个严重的限制，确保这个出色的功能能够恰当地发挥作用！

## 绑定组布局条目 [#](https://bevy.org/news/bevy-0-13/#bind-group-layout-entries)

作者：@IceSentry

我们添加了一个新的 API，灵感来自 0.12 中的绑定组条目 API，用于声明绑定组布局。这个新的 API 基于使用内置函数来定义绑定组布局资源，并根据其位置自动设置索引。

以下是声明新布局的简短示例：

```rust
let layout = render_device.create_bind_group_layout(
    "post_process_bind_group_layout",
    &BindGroupLayoutEntries::sequential(
        ShaderStages::FRAGMENT,
        (
            texture_2d_f32(),
            sampler(SamplerBindingType::Filtering),
            uniform_buffer::<PostProcessingSettings>(false),
        ),
    ),
);
```

## `RenderGraph` 的类型安全标签 [#](https://bevy.org/news/bevy-0-13/#type-safe-labels-for-the-rendergraph)

作者：@DasLixou

Bevy 在定义标签时大量使用了 Rust 的类型系统，让开发者利用工具来捕获拼写错误并简化重构。但这并不适用于 Bevy 的渲染图。在渲染图中，硬编码的——且可能重叠的——字符串被用来定义节点和子图。

```rust
// 0.13 之前
impl MyRenderNode {
    pub const NAME: &'static str = "my_render_node"
}
```

在 **Bevy 0.13** 中，我们使用了一种更健壮的方式来命名渲染节点和渲染图，借助了 `bevy_ecs` 已经使用的类型安全标签模式。

```rust
// 0.13 之后
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct PrettyFeature;
```

有了这些，常量值的长路径变得更短更清晰：

```rust
// 0.13 之前
render_app
    .add_render_graph_node::<ViewNodeRunner<PrettyFeatureNode>>(
        core_3d::graph::NAME,
        PrettyFeatureNode::NAME,
    )
    .add_render_graph_edges(
        core_3d::graph::NAME,
        &[
            core_3d::graph::node::TONEMAPPING,
            PrettyFeatureNode::NAME,
            core_3d::graph::node::END_MAIN_PASS_POST_PROCESSING,
        ],
    );

// 0.13 之后
use bevy::core_pipeline::core_3d::graph::{Node3d, Core3d};

render_app
    .add_render_graph_node::<ViewNodeRunner<PrettyFeatureNode>>(
        Core3d,
        PrettyFeature,
    )
    .add_render_graph_edges(
        Core3d,
        (
            Node3d::Tonemapping,
            PrettyFeature,
            Node3d::EndMainPassPostProcessing,
        ),
    );
```

当你需要渲染节点的动态标签时，仍然可以通过元组结构体等方式实现：

```rust
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct MyDynamicLabel(&'static str);
```

这特别好的地方在于我们不需要在这里存储字符串：我们可以使用整数、自定义枚举或任何其他可哈希的类型。

## Winit 升级 [#](https://bevy.org/news/bevy-0-13/#winit-upgrade)

作者：@Thierry Berger, @mockersf

通过我们的贡献者和审阅者的英勇努力，Bevy [现已升级](https://github.com/bevyengine/bevy/pull/10702)到使用 `winit 0.29`。[`winit`](https://docs.rs/winit/latest/winit/) 是我们的窗口库：它抽象了最终用户可能拥有的所有不同操作系统和输入设备，并提供了一个统一的 API 以实现一次编写、随处运行的体验。虽然这带来了通常的一系列有价值的 [Bug 修复和稳定性改进](https://github.com/rust-windowing/winit/blob/master/CHANGELOG.md#0292)，但关键的变化在于如何处理 [`KeyCode`](https://docs.rs/bevy/latest/bevy/input/keyboard/enum.KeyCode.html)。

以前，[`KeyCode`](https://docs.rs/bevy/latest/bevy/input/keyboard/enum.KeyCode.html) 表示键盘上按键的逻辑含义：在同一键盘上交换 QWERTY 和 AZERTY 键盘布局时，按下同一个按钮会产生不同的结果！现在，[`KeyCode`](https://docs.rs/bevy/latest/bevy/input/keyboard/enum.KeyCode.html) 表示按键的物理位置。WASD 游戏的爱好者知道这对于游戏来说是更好的默认设置。对于大多数 Bevy 开发者来说，你可以保持现有代码不变，只需让非 QWERTY 键盘或布局的用户受益于更好的默认按键绑定。如果你需要有关按下的逻辑键的信息，请使用 [`ReceivedCharacter`](https://docs.rs/bevy/latest/bevy/prelude/struct.ReceivedCharacter.html) 事件。

## 多个 Gizmo 配置 [#](https://bevy.org/news/bevy-0-13/#multiple-gizmo-configurations)

作者：@jeliag

Gizmo 允许你使用即时模式 API 快速绘制形状。以下是如何使用它们：

```rust
// Bevy 0.12.1
fn set_gizmo_width(mut config: ResMut<GizmoConfig>) {
    // 设置所有 gizmo 的线宽，使用此全局配置资源。
    config.line_width = 5.0;
}

fn draw_circles(mut gizmos: Gizmos) {
    // 绘制两个线宽为 5 像素的圆
    gizmos.circle_2d(vec2(100., 0.), 120., Color::NAVY);
    gizmos.circle_2d(vec2(-100., 0.), 120., Color::ORANGE);
}
```

添加 [`Gizmos`](https://docs.rs/bevy/0.13.0/bevy/gizmos/prelude/struct.Gizmos.html) 系统参数并调用几个方法。很酷！

Gizmo 对于 crate 作者来说也很棒，他们可以使用相同的 API。例如，[`oxidized_navigation`](https://crates.io/crates/oxidized_navigation) 导航网格库使用 gizmo 进行调试覆盖。很不错！

然而，只有一个全局配置。因此，依赖项很可能会影响游戏的 gizmo。它甚至可能使它们完全无法使用。

不怎么好。如何解决这个问题？Gizmo 组。

现在，[`Gizmos`](https://docs.rs/bevy/0.13.0/bevy/gizmos/prelude/struct.Gizmos.html) 带有一个可选参数。默认情况下，它使用全局配置：

```rust
fn draw_circles(mut default_gizmos: Gizmos) {
    default_gizmos.circle_2d(vec2(100., 0.), 120., Color::NAVY);
}
```

但是使用 [`GizmoConfigGroup`](https://docs.rs/bevy/0.13.0/bevy/gizmos/config/trait.GizmoConfigGroup.html) 参数，`Gizmos` 可以选择不同的配置：

```rust
fn draw_circles(
    mut default_gizmos: Gizmos,
    // 这使用独立的 NavigationGroup 配置
    mut navigation_gizmos: Gizmos<NavigationGroup>,
) {
    // 两个具有不同轮廓宽度的圆
    default_gizmos.circle_2d(vec2(100., 0.), 120., Color::NAVY);
    navigation_gizmos.circle_2d(vec2(-100., 0.), 120., Color::ORANGE);
}
```

通过派生 `GizmoConfigGroup` 并将其注册到 `App` 来创建你自己的 gizmo 配置组：

```rust
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct NavigationGroup;

impl Plugin for NavigationPlugin {
    fn build(&mut self, app: &mut App) {
        app
            .init_gizmo_group::<NavigationGroup>()
            // ... 其余插件初始化
    }
}
```

这就是如何将 gizmo 组的配置设置为不同值的方法：

```rust
// Bevy 0.13.0
set_gizmo_width(mut config_store: ResMut<GizmoConfigStore>) {
    let config = config_store.config_mut::<DefaultGizmoConfigGroup>().0;
    config.line_width = 20.0;

    let navigation_config = config_store.config_mut::<NavigationGroup>().0;
    navigation_config.line_width = 10.0;
}
```

现在，导航 gizmo 拥有完全独立的配置，不会与游戏的 gizmo 冲突。

不仅如此，游戏开发者还可以根据需要使用自己的调试工具集成和切换导航 gizmo。无论是热键、调试覆盖 UI 按钮还是 RPC 调用。世界由你掌控。

## glTF 扩展 [#](https://bevy.org/news/bevy-0-13/#gltf-extensions)

作者：@CorneliusCornbread

**[glTF](https://www.khronos.org/gltf/)** 是一种流行的标准化开放文件格式，用于在不同程序之间存储和共享 3D 模型与场景。然而，标准的问题在于你最终想要_定制_它，哪怕只是一点点，以更好地满足你的需求。Khronos Group 明智地预见到了这一点，并定义了一种标准化的定制格式的方法，称为**[扩展](https://kcoley.github.io/glTF/extensions/)**。

扩展可以从其他工具（如 Blender）轻松导出，并包含[各种](https://github.com/KhronosGroup/glTF/blob/main/extensions/README.md)有用的信息：从尖端的基于物理的材质信息（如各向异性）到性能提示（如如何实例化网格）。

由于 Bevy 将加载的 glTF 解析为我们自己的基于实体的对象层次结构，因此当你想要进行新的渲染工作时，访问这些信息可能很困难！通过 [CorneliusCornbread 的更改](https://github.com/bevyengine/bevy/pull/11138)，你可以配置加载器将 glTF 文件的原始副本与加载的资源一起存储，从而允许你根据需要解析和重新处理此信息。

## 资源转换器 [#](https://bevy.org/news/bevy-0-13/#asset-transformers)

作者：@thepackett, @RyanSpaker

资源处理的核心涉及实现 `Process` trait，它获取一些代表资源的字节数据，对其进行转换，然后返回处理后的字节数据。然而，手动实现 `Process` trait 有些复杂，因此编写了一个通用的 `LoadAndSave<L: AssetLoader, S: AssetSaver>` `Process` 实现，以使资源处理更加符合人体工程学。

使用 `LoadAndSave` `Process` 实现，以前的资源处理管道有四个阶段：

1. `AssetReader` 读取某些资源源（文件系统、http 等）并获取资源的字节数据。
2. `AssetLoader` 读取字节数据并将其转换为 Bevy `Asset`。
3. `AssetSaver` 获取 Bevy `Asset`，处理它，然后将其转换回字节数据。
4. `AssetWriter` 然后将资源字节数据写回资源源。

`AssetSaver` 负责转换资源和将其转换为字节数据。然而，这对代码可重用性造成了一些问题。每次你想要转换某个资源（如图像）时，你都需要重写将资源转换为字节数据的部分。为了解决这个问题，`AssetSaver` 现在只负责将资源转换为字节数据，并引入了 `AssetTransformer`，它负责转换资源。一个新的 `LoadTransformAndSave<L: AssetLoader, T: AssetTransformer, S: AssetSaver>` `Process` 实现被添加以利用新的 `AssetTransformer`。

使用 `LoadTransformAndSave` `Process` 实现的新的资源处理管道有五个阶段：

1. `AssetReader` 读取某些资源源（文件系统、http 等）并获取资源的字节数据。
2. `AssetLoader` 读取字节数据并将其转换为 Bevy `Asset`。
3. `AssetTransformer` 获取一个资源并以某种方式对其进行转换。
4. `AssetSaver` 获取 Bevy `Asset` 并将其转换回字节数据。
5. `AssetWriter` 然后将资源字节数据写回资源源。

除了具有更好的代码可重用性之外，这一更改还鼓励为各种常见资源类型编写 `AssetSaver`，这些 `AssetSaver` 可用于向 `AssetServer` 添加运行时资源保存功能。

以前的 `LoadAndSave` `Process` 实现仍然存在，因为在某些情况下不需要资源转换步骤，例如将资源保存为压缩格式时。

请参阅[资源处理示例](https://github.com/bevyengine/bevy/blob/main/examples/asset/processing/asset_processing.rs)，以更详细地了解如何使用 `LoadTransformAndSave` 处理自定义资源。

## 实体优化 [#](https://bevy.org/news/bevy-0-13/#entity-optimizations)

作者：@Bluefinger, @notverymoe, @scottmcm, @james7132, @NathanSWard

`Entity`（Bevy 用于实体的 64 位唯一标识符）在这个周期中收到了几项更改，为 Relations 以及_相关的_、很好的性能优化奠定了一些基础。这里的工作涉及深入钻研编译器代码生成/汇编输出，运行大量基准测试和测试，以确保所有更改不会导致破坏或重大问题。虽然这里的工作主要涉及_安全_代码，但许多底层假设正在被改变，这可能会影响其他地方的代码。这是 Bevy 0.13 中最"微优化"导向的一组更改。

- [#9797](https://github.com/bevyengine/bevy/pull/9797)：创建了一个统一的标识符类型，为我们能够在 `Entity` 类型和期待已久的 Relations 中使用相同快速、复杂的代码铺平了道路
- [#9907](https://github.com/bevyengine/bevy/pull/9907)：允许我们在相同数量的位中存储 `Option<Entity>`，通过修改 Entity 类型的布局，精确地保留一个 `u64` 值用于 `None` 变体
- [#10519](https://github.com/bevyengine/bevy/pull/10519)：切换到为 `Entity` 手动制作的 `PartialEq` 和 `Hash` 实现，以提高速度并在热循环中节省指令
- [#10558](https://github.com/bevyengine/bevy/pull/10558)：结合了 [#9907](https://github.com/bevyengine/bevy/pull/9907) 和 [#10519](https://github.com/bevyengine/bevy/pull/10519) 的方法进一步优化了 `Entity` 的布局，并优化了我们的 `PartialOrd` 和 `Ord` 实现！
- [#10648](https://github.com/bevyengine/bevy/pull/10648)：进一步优化了我们的实体哈希，改变了哈希中的乘法方式，在优化的编译器输出中节省了一条宝贵的汇编指令

完全归功于在 [#2372](https://github.com/bevyengine/bevy/pull/2372) 和 [#3788](https://github.com/bevyengine/bevy/pull/3788) 中进行了类似工作的作者：虽然他们的工作最终没有被合并，但它是基于这些最新更改的宝贵灵感和先行艺术来源。

![优化工作的基准测试结果](https://bevy.org/news/bevy-0-13/entity_hash_optimsation_benches.png)

上面的结果显示了我们从起点（`optimised_eq` 是引入基准测试的第一个 PR）到我们现在所有优化到位（`optimised_entity`）的情况。各方面都有改进，带来了清晰的性能优势，应该会影响代码库的多个领域，而不仅仅是在哈希实体时。

链接的 PR 中有大量详实、解释充分的细节，包括一些令人着迷的汇编输出分析。如果你对此感兴趣，请在后台打开一些新标签页！

## 将 `Query::for_each` 移植到 `QueryIter::fold` 重写 [#](https://bevy.org/news/bevy-0-13/#porting-query-for-each-to-queryiter-fold-override)

作者：@james7132

目前，为了从迭代查询中获得全部性能，必须使用 `Query::for_each` 来利用编译器可以应用的自动向量化和内部迭代优化。然而，这不是惯用的 Rust，也不是迭代器方法，所以你无法在迭代器链上使用它。然而，对于某些迭代器方法，有可能获得相同的好处，[@james7132 的 #6773](https://github.com/bevyengine/bevy/pull/6773/) 力求实现这一点。通过提供对 `QueryIter::fold` 的重写，可以将 `Query::for_each` 的迭代策略移植到 `Query::iter` 等上，以实现相同的增益。并非_每个_迭代器方法目前都能从中受益，因为它们需要重写 `QueryIter::try_fold`，但这目前仍然是仅限 nightly 的优化。同样的方法也在 Rust 标准库中使用。

这在几个地方减少了重复代码，例如不再需要同时有 `Query::for_each` 和 `Query::for_each_mut`，因为只需调用 `Query::iter` 或 `Query::iter_mut` 即可。所以像这样的代码：

```rust
fn some_system(mut q_transform: Query<&mut Transform, With<Npc>>) {
    q_transform.for_each_mut(|transform| {
        // 做一些事情...
    });
}
```

变成了：

```rust
fn some_system(mut q_transform: Query<&mut Transform, With<Npc>>) {
    q_transform.iter_mut().for_each(|transform| {
        // 做一些事情...
    });
}
```

还对主分支和 PR 之间的汇编输出进行了比较，在旧的 `Query::for_each` 和新的 `QueryIter::for_each()` 输出之间没有看到实质性的差异，验证了该方法并确保了内部迭代优化被应用。

另外，`Query::par_for_each` 中的相同内部迭代优化现在重用了 `for_each` 中的代码，也在此处去重了代码，并允许用户使用 `par_iter().for_each()`。总的来说，这意味着不再需要 `Query::for_each`、`Query::for_each_mut`、`Query::_par_for_each`、`Query::par_for_each_mut`，因此这些方法已在 0.13 中弃用，并将在 0.14 中移除。

## 减少 `TableRow` 的 `as` 转换 [#](https://bevy.org/news/bevy-0-13/#reducing-tablerow-as-casting)

作者：@bushrat011899

并非我们 ECS 内部的所有改进都专注于性能。一些小的更改是为了提高类型安全性和清理代码库，减少在各个调用点上对 `TableRow` 进行的 `as` 转换。`as` 转换的问题在于，在某些情况下，转换会静默截断值而失败，这可能会导致访问错误行等严重问题。[#10811](https://github.com/bevyengine/bevy/pull/10811) 由 @bushrat011899 提出，旨在清理 `TableRow` 周围的 API，提供由 `assert` 支持的便捷方法，以确保转换操作永远不会失败，或者如果失败，它们会正确地 panic。

自然，在潜在的热代码路径中_添加_断言引起了一些担忧，需要大量的基准测试工作来确认是否存在回归以及程度如何。通过仔细放置新的 `assert`，这些情况检测到的回归在大约 0.1% 的范围内，完全在噪声范围内。但好处是 API 更不容易出错，代码更健壮。对于像 `bevy_ecs` 这样复杂的不安全代码库，每一点帮助都很重要。

## 事件存活更长时间 [#](https://bevy.org/news/bevy-0-13/#events-live-longer)

事件是向系统之间以及系统之间传递数据的有用工具。

在内部，Bevy 事件是双缓冲的，因此一个给定的事件在缓冲区交换两次后会被静默丢弃。`Events<T>` 资源以这种方式设置，以便事件在可预测的时间后丢弃，防止它们的队列无限增长并导致内存泄漏。

在 0.12.1 之前，事件队列每次更新（即每帧）交换一次。这对于在 `FixedUpdate` 中包含逻辑的游戏来说是个问题，因为这意味着事件通常会在下一个 `FixedUpdate` 中的系统读取它们之前消失。

Bevy 0.12.1 将交换节奏改为"每次运行 `FixedUpdate` 一次或多次的更新"（仅在安装了 `TimePlugin` 的情况下）。这一更改确实解决了原始问题，但随后在另一个方向引起了问题。用户惊讶地发现他们某些带有 `run_if` 条件的系统会迭代比预期旧得多的事件。（事后看来，我们应该将其视为破坏性更改并将其推迟到本版本。）这一更改还引入了一个 Bug（在本版本中已修复），即只有一种类型的事件被丢弃。

对于这种 `Update` 和 `FixedUpdate` 之间持久但非预期的耦合，一个提议的未来解决方案是使用事件时间戳来改变 `EventReader<T>` 可见的事件默认范围。这样，`Update` 中的系统将跳过任何比一帧更旧的事件，而 `FixedUpdate` 中的系统仍然可以看到它们。

现在，可以通过简单地移除 `EventUpdateSignal` 资源来恢复 `<=0.12.0` 的行为。

```rust
fn main() {
    let mut app = App::new()
        .add_plugins(DefaultPlugins);
    
    /* ... */

    // 如果这个资源不存在，事件将以与 <=0.12.0 相同的方式丢弃。
    app.world.remove_resource::<EventUpdateSignal>();
    
    /* ... */

    app.run();
}
```

## 接下来是什么？ [#](https://bevy.org/news/bevy-0-13/#what-s-next)

我们有很多正在进行的工作！其中一些可能会在 **Bevy 0.14** 中落地。

查看 [**Bevy 0.14 Milestone**](https://github.com/bevyengine/bevy/milestone/20) 获取贡献者正在为 **Bevy 0.14** 关注的最新工作列表。

### 更多编辑器实验 [#](https://bevy.org/news/bevy-0-13/#more-editor-experimentation)

由才华横溢的 JMS55 领导，我们已经开放了一个自由形式的[游乐场](https://github.com/bevyengine/bevy_editor_prototypes)，用于定义和回答关于 `bevy_editor` 设计的[关键问题](https://github.com/bevyengine/bevy_editor_prototypes/discussions/1)：不是通过讨论，而是通过具体的原型设计。我们应该使用进程内编辑器（对游戏崩溃的鲁棒性较差）还是外部编辑器（更复杂）？我们应该发布一个编辑器二进制文件（对非程序员很好）还是将其嵌入到游戏本身中（非常可破解）？让我们通过实践来找出答案！

那里有一些令人难以置信的模型、功能原型和第三方编辑器相关项目。一些亮点：

[![图形化 Bevy 编辑器的 UI 模型](https://bevy.org/news/bevy-0-13/editor_mockup.png)](https://bevy.org/news/bevy-0-13/editor_mockup.png) (1) bevy_editor_mockup

[![基于 `bevy_egui` 的节点式动画图编辑器](https://bevy.org/news/bevy-0-13/locomotion_graph.png)](https://bevy.org/news/bevy-0-13/locomotion_graph.png) (2) bevy_animation_graph

[![`space_editor` 的截图，展示了一个带有 gizmo 的功能性场景编辑器](https://bevy.org/news/bevy-0-13/space_editor.png)](https://bevy.org/news/bevy-0-13/space_editor.png) (3) space_editor

[![Blender 的截图，Blender UI 修改 Bevy 组件值](https://bevy.org/news/bevy-0-13/bevy_components.jpg)](https://bevy.org/news/bevy-0-13/bevy_components.jpg) (4) bevy_components

[![一个基于 Web 的编辑器实时更改 Bevy 实体的录制](https://bevy.org/news/bevy-0-13/makeshift_web.jpg)](https://bevy.org/news/bevy-0-13/makeshift_web.jpg) (5) bevy_remote

1. Discord 上的 `@!!&Amy` 制作的 Bevy 品牌编辑器 UI 模型，想象基于 ECS 的编辑器[可能的外观](https://amytimed.github.io/bevy_editor_mockup/editor/)
2. [`bevy_animation_graph`](https://crates.io/crates/bevy_animation_graph)：一个功能完整的资源驱动型动画图 crate，为 Bevy 提供自己的节点式编辑器
3. [`space_editor`](https://github.com/rewin123/space_editor)：一个精致的 Bevy 原生第三方场景编辑器，你现在就可以使用！
4. [`Blender_bevy_components_workflow`](https://github.com/kaosat-dev/Blender_bevy_components_workflow)：一个令人印象深刻的功能性工具生态系统，允许你今天就将 Blender 用作无缝的关卡和场景编辑器。
5. `@coreh` 关于[基于反射的远程协议](https://github.com/coreh/bevy/pull/1)的实验，结合交互式 Web 编辑器，允许开发者从其他进程、语言甚至设备检查和操控他们的 Bevy 游戏！[在线尝试](https://makeshift-bevy-web-editor.vercel.app/)！

看到这些进展非常令人兴奋，我们渴望将这种精力和经验引导到官方的第一方努力中。

### `bevy_dev_tools` [#](https://bevy.org/news/bevy-0-13/#bevy-dev-tools)

流畅游戏开发的秘诀是优秀的工具。是时候为 Bevy 开发者提供他们检查、调试和分析游戏所需的工具，作为第一方体验的一部分了。从 FPS 计量器到系统步进，再到优秀的 [`bevy-inspector-egui`](https://crates.io/crates/bevy-inspector-egui) 的第一方等价物：在 Bevy 本身中为这些工具提供家园有助于我们打磨它们，引导新用户朝正确的方向前进，并允许我们在 `bevy_editor` 本身中使用它们。

### 新的场景格式 [#](https://bevy.org/news/bevy-0-13/#a-new-scene-format)

[场景](https://github.com/bevyengine/bevy/tree/latest/examples/scene)是 Bevy 将 ECS 数据序列化到磁盘的通用答案：跟踪实体、组件和资源，既可以用于保存游戏，也可以用于加载预先制作的关卡。然而，现有的基于 .ron 的场景格式难以手写，过于冗长且脆弱；你的代码（或你的依赖项的代码！）的更改会迅速使保存的场景失效。Cart 一直在酝酿一种[修订后的场景格式](https://github.com/bevyengine/bevy/discussions/9538)，具有紧密的 IDE 和代码集成，解决了这些问题，并使在 Bevy 中创作内容（包括 UI！）变得愉快。无论是编写代码、编写场景文件，还是从 GUI 生成它。

### `bevy_ui` 改进 [#](https://bevy.org/news/bevy-0-13/#bevy-ui-improvements)

`bevy_ui` 有相当多的问题和限制，[既有平凡也有架构性的](https://www.leafwing-studios.com/blog/ecs-gui-framework/)；然而，我们可以并且正在做一些切实可行的事情来改进这一点：改进的场景格式为定义布局提供了终结样板代码的方法，[圆角](https://github.com/bevyengine/bevy/pull/8973)只需要审阅者的一些关注，强大且备受喜爱的对象拾取功能（来自 [`bevy_mod_picking`]）计划向上游移植，用于 UI 和游戏玩法。今天已经存在一系列惊人的[第三方 UI 解决方案](https://bevy.org/assets/#ui)，从中学习并致力于 UI 逻辑和响应性的核心架构是重中之重。

### Meshlet 渲染 [#](https://bevy.org/news/bevy-0-13/#meshlet-rendering)

将网格分割成称为 meshlet 的三角形簇，带来了许多效率提升。在 0.13 开发周期中，我们在这个功能上取得了[很多进展](https://github.com/bevyengine/bevy/pull/10164)。我们实现了一个 GPU 驱动的 meshlet 渲染器，可以扩展到三角形密度更高的场景，具有更低的 CPU 负载。然而，内存使用非常高，而且我们还没有实现 LOD 或压缩。与其半生不熟地发布它，我们将继续迭代，并且非常兴奋地（希望）在未来的版本中为你带来这个功能。

![作为 meshlet 簇渲染的斯坦福龙网格](https://bevy.org/news/bevy-0-13/meshlet_preview.png)

### 稳步迈向 Relations [#](https://bevy.org/news/bevy-0-13/#the-steady-march-towards-relations)

[实体-实体关系](https://github.com/bevyengine/bevy/issues/3742)，即直接在 ECS 中跟踪和管理实体之间连接的能力，多年来一直是最受请求的 ECS 功能之一。继 [`flecs` 开辟的道路](https://ajmmertens.medium.com/building-games-in-ecs-with-entity-relationships-657275ba2c6c)之后，`#ecs-dev` 中的疯狂科学家们正在稳步[重塑我们的内部结构](https://github.com/orgs/bevyengine/projects/15)、[试验外部实现](https://crates.io/crates/aery)，并交付构建快速、健壮且符合人体工程学的解决方案所需的通用构建块（如动态查询或[生命周期钩子](https://github.com/bevyengine/bevy/pull/10756)）。
