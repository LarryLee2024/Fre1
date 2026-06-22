# Bevy 0.18

## Posted on January 13, 2026 by Bevy Contributors

![Toroban: an infinitely wrapping puzzle game built in Bevy. Out now!](https://bevy.org/news/bevy-0-18/toroban.jpg)

[Toroban: an infinitely wrapping puzzle game built in Bevy. Out now!](https://store.steampowered.com/app/1961850/Toroban/)

感谢 **174** 位贡献者、**659** 个 pull request、社区审阅者以及我们[**慷慨的捐赠者**](https://bevy.org/donate)，我们很高兴地宣布 **Bevy 0.18** 已在 [crates.io](https://crates.io/crates/bevy) 上发布！

如果你还不了解，Bevy 是一个基于 Rust 构建的、令人耳目一新的简洁数据驱动游戏引擎。你可以查看我们的[快速入门指南](https://bevy.org/learn/quick-start)来立即试用。它是免费且永远开源的！你可以在 GitHub 上获取完整的[源代码](https://github.com/bevyengine/bevy)。查看 [Bevy Assets](https://bevy.org/assets) 获取社区开发的插件、游戏和学习资源合集。

要将现有的 Bevy App 或 Plugin 更新到 **Bevy 0.18**，请查看我们的 [0.17 到 0.18 迁移指南](https://bevy.org/learn/migration-guides/0-17-to-0-18/)。

自上次发布以来的几个月里，我们添加了_大量_新功能、Bug 修复和易用性改进，以下是一些亮点：

- **大气遮挡与 PBR 着色**：程序化大气现在会影响光线到达场景中物体的方式！
- **广义大气散射介质**：程序化大气现在可以自定义，支持任意大气类型：沙漠天空、雾蒙蒙的海岸线、非地球类行星等。
- **Solari 改进**：Bevy 的实验性实时光线追踪渲染器获得了_大量_新功能和改进。
- **PBR 着色修复**：Bevy PBR 材质中一些长期存在的问题已得到修复，带来了明显的质量提升。
- **字体变体**：Bevy 现在支持可变字重字体、文字删除线、下划线和 OpenType 字体特性。
- **自动方向导航**：Bevy UI 元素现在可以选择自动方向导航，使得使用游戏手柄和键盘导航 UI 变得更加容易。
- **全屏材质**：一种新的高级材质类型，可以轻松定义全屏后处理着色器。
- **Cargo Feature 集合**：Bevy 现在具有面向场景的高级 Cargo feature，如 2D、3D 和 UI，可以轻松只编译应用所需的引擎部分。
- **第一方相机控制器**：Bevy 现在内置了基础的"飞行"相机和"平移"相机。

## 大气遮挡与 PBR 着色 [#](https://bevy.org/news/bevy-0-18/#atmosphere-occlusion-and-pbr-shading)

Authors:[@mate-h](https://github.com/mate-h)

PRs:[#21383](https://github.com/bevyengine/bevy/pull/21383)

程序化大气现在会影响光线到达场景中物体的方式！阳光在穿过大气时会自动获取正确的颜色，当日落或日出时阳光更接近地平线时会呈现橙色或红色。

这与体积雾和所有渲染模式无缝协作，因此你的场景将开箱即用地获得更加协调和逼真的光照效果。

查看更新后的 [`atmosphere` 示例](https://github.com/bevyengine/bevy/blob/latest/examples/3d/atmosphere.rs) 来查看实际效果！

![atmosphere shading](https://bevy.org/news/bevy-0-18/atmosphere_shading.jpg)

## 广义大气散射介质 [#](https://bevy.org/news/bevy-0-18/#generalized-atmospheric-scattering-media)

Authors:[@ecoskey](https://github.com/ecoskey)

PRs:[#20838](https://github.com/bevyengine/bevy/pull/20838)

![generalized atmosphere](https://bevy.org/news/bevy-0-18/generalized_atmosphere.jpg)

在 Bevy 0.18 中渲染的类火星大气

到目前为止，Bevy 的大气散射系统一直快速且美观，但定制性有限。只有有限的方式可以自定义现有参数，这将系统限制在主要模拟地球类场景。

**Bevy 0.18** 引入了新的 [`ScatteringMedium`](https://docs.rs/bevy/0.18.0/bevy/pbr/struct.ScatteringMedium.html) 资产，用于设计各种大气散射介质：清澈的沙漠天空、雾蒙蒙的海岸线，甚至其他行星的大气！我们充分利用了 Bevy 的资产系统——辅以一些自定义优化——以确保即使对于复杂的散射介质，渲染也能保持快速。

```rust
fn setup_camera(
    mut commands: Commands,
    mut media: ResMut<Assets<ScatteringMedium>>,
) {
    let medium = media.add(ScatteringMedium::new(
        256,
        256,
        [
            ScatteringTerm {
                absorption: Vec3::ZERO,
                scattering: Vec3::new(5.802e-6, 13.558e-6, 33.100e-6),
                falloff: Falloff::Exponential { strength: 12.5 },
                phase: PhaseFunction::Rayleigh,
            },
            ScatteringTerm {
                absorption: Vec3::splat(3.996e-6),
                scattering: Vec3::splat(0.444e-6),
                falloff: Falloff::Exponential { strength: 83.5 },
                phase: PhaseFunction::Mie { asymmetry: 0.8 },
            },
            ScatteringTerm {
                absorption: Vec3::new(0.650e-6, 1.881e-6, 0.085e-6),
                scattering: Vec3::ZERO,
                falloff: Falloff::Tent {
                    center: 0.75,
                    width: 0.3,
                },
                phase: PhaseFunction::Isotropic,
            },
        ],
    ));

    commands.spawn((
        Camera3d,
        Atmosphere::earthlike(medium)
    ));
}
```

## Solari 改进 [#](https://bevy.org/news/bevy-0-18/#solari-improvements)

Authors:[@JMS55](https://github.com/JMS55), [@SparkyPotato](https://github.com/SparkyPotato)

PRs:[#21391](https://github.com/bevyengine/bevy/pull/21391), [#21355](https://github.com/bevyengine/bevy/pull/21355), [#21810](https://github.com/bevyengine/bevy/pull/21810)

![solari specular](https://bevy.org/news/bevy-0-18/solari_specular.jpg)

Solari——Bevy 面向未来的实时光线追踪渲染器——在本次发布中获得了许多改进。主要包括：

- 支持镜面材质和反射
- 更快的光照响应
- 大量质量/精度改进
- 方向光的基于物理的软阴影
- 在更大场景上提升了性能

![solari pica pica](https://bevy.org/news/bevy-0-18/solari_pica_pica.jpg)

完整详情请查看作者的[完整博客文章](https://jms55.github.io/posts/2025-12-27-solari-bevy-0-18)。

## PBR 着色修复 [#](https://bevy.org/news/bevy-0-18/#pbr-shading-fixes)

Authors:[@aevyrie](https://github.com/aevyrie)

PRs:[#22372](https://github.com/bevyengine/bevy/pull/22372), [#22454](https://github.com/bevyengine/bevy/pull/22454)

Bevy 的 PBR 材质旨在提供标准化且可预测的物理着色模型。我们的初始 PBR 着色器大约在_四年前_完成。它总体上表现良好，并且随时间推移有了显著改进，但它确实存在一些明显感觉...不太对的问题。

Bevy 的材质有时被描述为"过于光亮"或"过于明亮"，而那些更了解着色器的人经常将问题归咎于一个叫 Fresnel 的东西。幸运的是，我们_终于_隔离了核心问题并修复了它们。希望结果不言自明！

拖动此图像进行对比

![Before Fixes](https://bevy.org/news/bevy-0-18/render_before.jpg)![After Fixes](https://bevy.org/news/bevy-0-18/render_after.jpg)

有两个核心问题：

1. 点光/面光的镜面分量过亮。
2. 我们的"环境贴图光照"着色器使用了"粗糙度相关 Fresnel"，如[此处定义](https://bruop.github.io/ibl/)。这通常无法按预期工作，因此我们切换为更直接的方法...直接使用"典型"Fresnel 项。

更多对比图片请参见上方链接的两个 PR。现在我们可以放心了。Fresnel 不会再伤害我们了。

## 全屏材质 [#](https://bevy.org/news/bevy-0-18/#fullscreen-material)

Authors:[@IceSentry](https://github.com/IceSentry)

PRs:[#20414](https://github.com/bevyengine/bevy/pull/20414)

在 Bevy 的早期版本中，定义自定义全屏效果的唯一方式是定义新的底层渲染特性。这种方法灵活性最高，但对于常见用例来说复杂度过高。

为了解决这个问题，我们引入了新的高级 [`FullscreenMaterial`](https://docs.rs/bevy/0.18.0/bevy/core_pipeline/fullscreen_material/trait.FullscreenMaterial.html) trait 和 [`FullscreenMaterialPlugin`](https://docs.rs/bevy/0.18.0/bevy/core_pipeline/fullscreen_material/struct.FullscreenMaterialPlugin.html)，让你可以轻松运行全屏着色器并指定它相对于引擎中其他渲染通道的运行顺序。

```rust
impl FullscreenMaterial for ChromaticAberration {
    fn fragment_shader() -> ShaderRef {
        "chromatic_aberration.wgsl".into()
    }

    // 确定材质何时运行的 Render Graph 边
    fn node_edges() -> Vec<InternedRenderLabel> {
        vec![
            // 此材质在 3D 色调映射之后运行
            Node3d::Tonemapping.intern(),
            // 这是 FullscreenMaterial 的标签
            Self::node_label().intern(),
            // 此材质将在主 3D 后处理通道结束之前运行
            Node3d::EndMainPassPostProcessing.intern(),
        ]
    }
}
```

查看我们新的 [`fullscreen_material`](https://github.com/bevyengine/bevy/blob/release-0.18.0/examples/shader_advanced/fullscreen_material.rs) 示例，了解"色差"全屏材质的完整演示：

![fullscreen material](https://bevy.org/news/bevy-0-18/fullscreen_material.jpg)

## 更多标准 Widget [#](https://bevy.org/news/bevy-0-18/#more-standard-widgets)

Authors:[@viridia](https://github.com/viridia), [@PPakalns](https://github.com/PPakalns)

PRs:[#21636](https://github.com/bevyengine/bevy/pull/21636), [#21743](https://github.com/bevyengine/bevy/pull/21743), [#21294](https://github.com/bevyengine/bevy/pull/21294)

我们正在继续完善在 **Bevy 0.17** 中首次引入的标准 widget 集合。请注意，Bevy 的标准 widget 是"逻辑 widget"，它们是"无主题的"。

### Popover [#](https://bevy.org/news/bevy-0-18/#popover)

[`Popover`](https://docs.rs/bevy/0.18.0/bevy/ui_widgets/popover/struct.Popover.html) 组件可以放置在绝对定位的 UI 节点上，提供自动弹出定位功能。这受到了流行的 [`floating-ui`](https://www.npmjs.com/package/@floating-ui/core) npm 包的启发。

Popover 将相对于锚点元素放置，并定位以确保不被窗口边缘裁剪。你可以指定一组首选"放置位置"：上、下、左或右，以及每个位置的对齐选项。如果弹出窗口太大以至于无法在不被裁剪的情况下放置，它将选择可见性最高的放置方式（由被裁剪的面积决定）。（未来版本可能还会提供将弹出窗口限制为不超过窗口大小的选项，但这在我们有更好的滚动支持时会更有用。）

这种自动定位是动态的，这意味着如果锚点元素移动、位于滚动容器内或窗口被调整大小，popover 可能会"翻转"到另一侧以保持完全可见。

Popover 可以用于下拉菜单，也可以用于工具提示。

### 菜单 [#](https://bevy.org/news/bevy-0-18/#menu)

[`MenuPopup`](https://docs.rs/bevy/0.18.0/bevy/ui_widgets/struct.MenuPopup.html) 组件使用 [`Popover`](https://docs.rs/bevy/0.18.0/bevy/ui_widgets/popover/struct.Popover.html) 来提供下拉菜单 widget。这增加了打开和关闭菜单的事件，以及使用焦点系统的键盘导航和激活。

### `RadioButton` 和 `RadioGroup` 的改进 [#](https://bevy.org/news/bevy-0-18/#improvements-to-radiobutton-and-radiogroup)

在用户测试之后，我们以完全向后兼容的方式改进了现有 [`RadioButton`](https://docs.rs/bevy/0.18.0/bevy/ui_widgets/struct.RadioButton.html) 和 [`RadioGroup`](https://docs.rs/bevy/0.18.0/bevy/ui_widgets/struct.RadioGroup.html) widget 的细节：

- 用户交互的事件传播现在即使在 widget 被禁用时也会被取消。之前，一些相关的事件传播没有被正确取消。
- [`RadioButton`](https://docs.rs/bevy/0.18.0/bevy/ui_widgets/struct.RadioButton.html) 现在在被选中时会发出 `ValueChange<bool>` 实体事件，即使通过 [`RadioGroup`](https://docs.rs/bevy/0.18.0/bevy/ui_widgets/struct.RadioGroup.html) 选中也是如此。与其他 `Checkable` widget 保持一致。由于 [`RadioButton`](https://docs.rs/bevy/0.18.0/bevy/ui_widgets/struct.RadioButton.html) 无法通过直接用户交互取消选中，因此无法为 [`RadioButton`](https://docs.rs/bevy/0.18.0/bevy/ui_widgets/struct.RadioButton.html) 触发值为 `false` 的 [`ValueChange`](https://docs.rs/bevy/0.18.0/bevy/ui_widgets/struct.ValueChange.html) 事件。
- 如果 [`RadioButton`](https://docs.rs/bevy/0.18.0/bevy/ui_widgets/struct.RadioButton.html) 可聚焦，则可以在聚焦时使用 **Space** 或 **Enter** 键触发值变更事件。
- [`RadioGroup`](https://docs.rs/bevy/0.18.0/bevy/ui_widgets/struct.RadioGroup.html) 现在是可选的，可以用自定义实现替换。

## Bevy Feathers Widget：Color Plane [#](https://bevy.org/news/bevy-0-18/#bevy-feathers-widget-color-plane)

Authors:[@viridia](https://github.com/viridia)

PRs:[#21743](https://github.com/bevyengine/bevy/pull/21743)

在上次发布中，我们引入了 [Bevy Feathers](https://bevy.org/news/bevy-0-17/#bevy-feathers-widgets-for-tooling-experimental)，一个用于构建工具（如即将推出的 Bevy Editor）的实验性新 widget 库。

在 **Bevy 0.18** 中，我们添加了 [`ColorPlane`](https://docs.rs/bevy/0.18.0/bevy/feathers/controls/enum.ColorPlane.html) widget：一个二维颜色选择器，允许在颜色空间中选择两个不同的通道，一个沿水平轴，一个沿垂直轴。它可以配置为显示多种不同的颜色空间：色相与亮度、色相与饱和度、红色与蓝色等。

![color plane widget](https://bevy.org/news/bevy-0-18/color_plane.jpg)

## 第一方相机控制器 [#](https://bevy.org/news/bevy-0-18/#first-party-camera-controllers)

Authors:[@alice-i-cecile](https://github.com/alice-i-cecile), [@syszery](https://github.com/syszery)

PRs:[#20215](https://github.com/bevyengine/bevy/pull/20215), [#21450](https://github.com/bevyengine/bevy/pull/21450), [#21520](https://github.com/bevyengine/bevy/pull/21520)

要理解并与场景交互，你必须通过相机的镜头来观察它。但控制相机有很多方式！

让相机控制器感觉_恰到好处_既困难又至关重要：它们对游戏体验和软件可用性都有重大影响。

历史上，Bevy 完全将其交给个别游戏开发者：相机控制器需要深度定制和无尽的微调。然而，Bevy 作为游戏引擎需要_自己的_相机控制器：允许用户在开发期间（而非游戏过程中）快速轻松地探索场景。

为此，我们创建了 [`bevy_camera_controller`](https://docs.rs/bevy/0.18.0/bevy/camera_controller/index.html)：为我们提供了一个存储、共享和完善相机控制器的地方，以便于开发和最终的 Bevy Editor。我们以几个相机控制器作为起点，详见下文。这些对于_正常_游戏逻辑可能不太有用，但它们非常适合调试和构建工具。它们的代码也可以被复制并用作游戏特定相机逻辑的基础！

### `FreeCamera` [#](https://bevy.org/news/bevy-0-18/#freecamera)

我们引入的第一个相机控制器是"自由相机"，设计用于在场景中快速移动，完全忽略物理和几何体。你可能听说过"飞行相机"控制器，它是"自由相机"控制器的一个特化版本，专为快速流畅地覆盖大面积地形而设计。

许多 Bevy 示例现在使用 [`FreeCamera`](https://docs.rs/bevy/0.18.0/bevy/camera_controller/free_camera/struct.FreeCamera.html)，包括 `solari` 示例（演示 Bevy 的实验性光线追踪渲染器）。

要将自由相机控制器添加到你的项目中（通常在 `dev_mode` feature flag 下），请将 [`FreeCameraPlugin`](https://docs.rs/bevy/0.18.0/bevy/camera_controller/free_camera/struct.FreeCameraPlugin.html) 和 [`FreeCamera`](https://docs.rs/bevy/0.18.0/bevy/camera_controller/free_camera/struct.FreeCamera.html) 组件添加到你的相机实体。

要配置设置（速度、行为、按键绑定）或启用/禁用控制器，请修改 [`FreeCamera`](https://docs.rs/bevy/0.18.0/bevy/camera_controller/free_camera/struct.FreeCamera.html) 组件。我们已尽力选择好的默认值，但你的场景细节（尤其是缩放！）会对感觉产生很大影响。

### `PanCamera` [#](https://bevy.org/news/bevy-0-18/#pancamera)

[`PanCamera`](https://docs.rs/bevy/0.18.0/bevy/camera_controller/pan_camera/struct.PanCamera.html) 控制器是一个简单有效的工具，专为 2D 游戏或任何需要轻松平移相机和缩放的项目而设计。它允许你使用 WASD 键移动相机，并使用鼠标滚轮或 +/- 键缩放。

通过添加 [`PanCameraPlugin`](https://docs.rs/bevy/0.18.0/bevy/camera_controller/pan_camera/struct.PanCameraPlugin.html) 并将 [`PanCamera`](https://docs.rs/bevy/0.18.0/bevy/camera_controller/pan_camera/struct.PanCamera.html) 组件附加到你的相机实体，你可以快速将此控制器添加到项目中。

要配置相机的缩放级别、速度或按键绑定，只需修改 [`PanCamera`](https://docs.rs/bevy/0.18.0/bevy/camera_controller/pan_camera/struct.PanCamera.html) 组件。默认设置应该适用于大多数用例，但你可以根据特定需求进行调整，特别是对于大规模或高分辨率 2D 场景。

## 自动方向导航 [#](https://bevy.org/news/bevy-0-18/#automatic-directional-navigation)

Authors:[@jbuehler23](https://github.com/jbuehler23)

PRs:[#21668](https://github.com/bevyengine/bevy/pull/21668), [#22340](https://github.com/bevyengine/bevy/pull/22340)

Bevy 现在支持 UI 元素的**自动方向导航**！通过一些全局设置，你所有的 UI 元素现在都可以使用游戏手柄或方向键进行导航。不再需要为菜单和 UI 屏幕繁琐地手动连接导航线路。

以前，为 UI 创建方向导航需要使用 [`DirectionalNavigationMap`](https://docs.rs/bevy/0.18.0/bevy/input_focus/directional_navigation/struct.DirectionalNavigationMap.html) 手动定义可聚焦元素之间的每个连接。对于动态 UI 或复杂布局，这既耗时又容易出错。

现在，你只需将 [`AutoDirectionalNavigation`](https://docs.rs/bevy/0.18.0/bevy/ui/auto_directional_navigation/struct.AutoDirectionalNavigation.html) 组件添加到 UI 实体，Bevy 就会根据空间位置自动计算导航连接。系统参数会智能地在 8 个罗盘方向（北、东北、东等）中查找最近的邻居，考虑：

- **距离**：更近的元素优先
- **对齐**：更直接与导航方向对齐的元素更受青睐
- **重叠**：对于基本方向（北/南/东/西），系统确保足够的垂直重叠

### 使用方法 [#](https://bevy.org/news/bevy-0-18/#how-to-use-it)

只需将 [`AutoDirectionalNavigation`](https://docs.rs/bevy/0.18.0/bevy/ui/auto_directional_navigation/struct.AutoDirectionalNavigation.html) 组件添加到你希望用户能够在之间导航的 UI 实体：

```rust
commands.spawn((
    Button,
    Node { /* ... */ },
    AutoDirectionalNavigation::default(),
    // ... other components
));
```

要使用自动导航功能，请使用 [`AutoDirectionalNavigator`](https://docs.rs/bevy/0.18.0/bevy/ui/auto_directional_navigation/struct.AutoDirectionalNavigator.html) 系统参数替代 [`DirectionalNavigation`](https://docs.rs/bevy/0.18.0/bevy/input_focus/directional_navigation/struct.DirectionalNavigation.html) 系统参数：

```rust
fn my_navigation_system(mut auto_directional_navigator: AutoDirectionalNavigator) {
    // ...
    auto_directional_navigator.navigate(CompassOctant::East);
    // ...
}
```

### 配置 [#](https://bevy.org/news/bevy-0-18/#configuration)

你可以使用 [`AutoNavigationConfig`](https://docs.rs/bevy/0.18.0/bevy/input_focus/directional_navigation/struct.AutoNavigationConfig.html) 资源来调整行为：

```rust
app.insert_resource(AutoNavigationConfig {
    // 所需的最小重叠（0.0 = 任意重叠，1.0 = 完美对齐）
    min_alignment_factor: 0.0,
    // 可选的最大连接距离
    max_search_distance: Some(500.0),
    // 是否强烈偏好对齐良好的节点
    prefer_aligned: true,
});
```

### 手动覆盖 [#](https://bevy.org/news/bevy-0-18/#manual-override)

自动导航会尊重手动定义的边。如果你想覆盖特定连接（例如屏幕边缘循环），你仍然可以使用 `DirectionalNavigationMap::add_edge()` 或 `add_symmetrical_edge()`，这些连接将优先于自动生成的连接。你也可以直接调用 `auto_generate_navigation_edges()`，如果你有多个 UI 层的话（虽然可能不常用）

## Cargo Feature 集合 [#](https://bevy.org/news/bevy-0-18/#cargo-feature-collections)

Authors:[@cart](https://github.com/cart)

PRs:[#21472](https://github.com/bevyengine/bevy/pull/21472)

历史上，Bevy 开发者一直过着两种生活之一：

1. 使用 Bevy 的所有默认 feature，可能会编译许多不需要或不需要的 feature。
2. 禁用 Bevy 的默认 feature 并手动定义完整的 feature 列表。

生活在第 (2) 种世界中是一种令人沮丧的体验，因为 bevy 的 feature 列表_极其庞大_，而且完成特定任务所需的 feature 在不同版本之间经常变化。这是一项_专家级_任务，需要深入了解引擎内部才能正确完成。

**Bevy 0.18** 向 `bevy` crate 引入了高级"cargo feature 集合"：`2d`、`3d` 和 `ui`。这使开发者可以轻松选择他们想要构建的应用类型，只编译该应用所需的 Bevy 部分。

这意味着像将 Bevy 作为 UI 框架使用而不引入引擎其余部分这样的场景，现在变得如此简单：

```toml
bevy = { version = "0.18", default-features = false, features = ["ui"] }
```

我们还添加了中级 feature 集合如 `2d_api`，即 Bevy 的 2D API _不包含默认 Bevy 渲染器_。这使得替换默认 Bevy 渲染器为自定义渲染器变得更加容易。

例如，`2d` profile 如下所示：

```toml
2d = [
  "default_app",
  "default_platform",
  "2d_api",
  "2d_bevy_render",
  "ui",
  "scene",
  "audio",
  "picking",
]
```

构建自定义 2D 渲染器的人现在只需移除 `2d_bevy_render` 并提供自己的渲染器。

开发者现在可以从这些中级部分定义自己的高级 cargo feature profile，使得定义你想要构建到应用中的 Bevy 子集变得_更加容易_。

## 字体变体 [#](https://bevy.org/news/bevy-0-18/#font-variations)

Authors:[@ickshonpe](https://github.com/ickshonpe), [@hansler](https://github.com/hansler)

PRs:[#19020](https://github.com/bevyengine/bevy/pull/19020), [#21555](https://github.com/bevyengine/bevy/pull/21555), [#21559](https://github.com/bevyengine/bevy/pull/21559), [#22038](https://github.com/bevyengine/bevy/pull/22038)

**Bevy 0.18** 带来了更多对字体表达方式的控制！

### 文字删除线和下划线 [#](https://bevy.org/news/bevy-0-18/#text-strikethroughs-and-underlines)

`bevy_text` 现在支持删除线和下划线。要显示带删除线或下划线的文字，只需将 [`Strikethrough`](https://docs.rs/bevy/0.18.0/bevy/prelude/struct.Strikethrough.html) 或 [`Underline`](https://docs.rs/bevy/0.18.0/bevy/prelude/struct.Underline.html) 组件添加到任何 [`Text`](https://docs.rs/bevy/0.18.0/bevy/prelude/struct.Text.html)、[`Text2d`](https://docs.rs/bevy/0.18.0/bevy/prelude/struct.Text2d.html) 或 [`TextSpan`](https://docs.rs/bevy/0.18.0/bevy/prelude/struct.TextSpan.html) 实体。你可以使用 [`StrikethroughColor`](https://docs.rs/bevy/0.18.0/bevy/prelude/struct.StrikethroughColor.html) 和 [`UnderlineColor`](https://docs.rs/bevy/0.18.0/bevy/prelude/struct.UnderlineColor.html) 组件分别设置删除线和下划线的颜色。

### 字重 [#](https://bevy.org/news/bevy-0-18/#font-weights)

Bevy 现在支持字重，允许你利用[可变字重字体](https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Fonts/Variable_fonts)，它们将字体的平滑变化嵌入到单个文件中！[`TextFont`](https://docs.rs/bevy/0.18.0/bevy/prelude/struct.TextFont.html) 现在有一个 `weight: FontWeight` 字段。[`FontWeight`](https://docs.rs/bevy/0.18.0/bevy/prelude/struct.FontWeight.html) 是 `u16` 的 newtype（限制在 1-1000 范围内），较低的值表示细体字形，较高的值表示粗体字形。

![font weights](https://bevy.org/news/bevy-0-18/font_weights.jpg)

### OpenType 字体特性 [#](https://bevy.org/news/bevy-0-18/#opentype-font-features)

[OpenType 字体特性](https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Fonts/OpenType_fonts)允许对文字显示方式进行细粒度控制，包括[连字](https://en.wikipedia.org/wiki/Ligature_\(writing\))、[小型大写字母](https://en.wikipedia.org/wiki/Small_caps)以及[更多](https://learn.microsoft.com/en-us/typography/opentype/spec/featurelist)。

这些特性现在可以在 Bevy 中使用，允许用户为其 UI 添加排版润色（如可选连字和旧式数字）。它还允许阿拉伯语或天城文等复杂文字使用其预期的连字进行更正确的渲染。

使用示例：

```rust
commands.spawn((
  TextSpan::new("Ligatures: ff, fi, fl, ffi, ffl"),
  TextFont {
    font: opentype_font_handle,
    font_features: FontFeatures::builder()
      .enable(FontFeatureTag::STANDARD_LIGATURES)
      .set(FontFeatureTag::WIDTH, 300)
      .build(),
    ..default()
  },
));
```

[`FontFeatures`](https://docs.rs/bevy/0.18.0/bevy/text/struct.FontFeatures.html) 也可以从列表构造：

```rust
TextFont {
  font: opentype_font_handle,
  font_features: [
    FontFeatureTag::STANDARD_LIGATURES,
    FontFeatureTag::STYLISTIC_ALTERNATES,
    FontFeatureTag::SLASHED_ZERO
  ].into(),
  ..default()
}
```

请注意，OpenType 字体特性仅适用于支持它们的 `.otf` 字体，不同字体可能支持 OpenType 特性的不同子集。

## 可选取的文本段落 [#](https://bevy.org/news/bevy-0-18/#pick-able-text-sections)

Authors:[@ickshonpe](https://github.com/ickshonpe)

PRs:[#22047](https://github.com/bevyengine/bevy/pull/22047)

属于 UI 文本节点的单个文本段落现在是可选取的，允许它们被选中，并且可以添加观察者来响应用户交互。

此功能在创建类似超链接的行为时非常有用，允许用户在游戏中为特定关键词创建鼠标悬停工具提示。

## 安全的多任意组件可变访问 [#](https://bevy.org/news/bevy-0-18/#safe-mutable-access-to-multiple-arbitrary-components)

Authors:[@hymm](https://github.com/hymm)

PRs:[#21780](https://github.com/bevyengine/bevy/pull/21780)

在使用 ECS 时，访问数据最高效的方式是批量查询多个实体上的相同组件：

```rust
for (mut a, mut b) in &mut query {
    /* logic here */
}
```

但在某些场景中，用户想要为_特定_实体访问多个_任意_组件：

```rust
let (mut a, mut b) = entity.get_components_mut::<(&mut A, &mut B)>()?;
```

这种方法类似于用户可能在其他游戏引擎中熟悉的传统"游戏对象"模型，在实现复杂的、非性能关键逻辑时特别有用。它也往往更容易与其他语言和外部工具结合使用，使其成为脚本、模组和集成工作的诱人设计。

理论上，这种访问方式相当直接：访问组件并确保我们不会违反 Rust 的可变别名规则。但[实现此功能的初步尝试](https://github.com/bevyengine/bevy/pull/13375)遇到了一些困难，在代码审查中检测到微妙的健全性问题。因此，我们之前为此引入了 unsafe 方法：`get_components_mut_unchecked`。由于跳过了别名检查，这些方法相对较快，但 unsafe 令人恐惧且使用繁琐，尤其对于旨在方便使用且通常对新手最有吸引力的 API。

在 **Bevy 0.18** 中，我们终于引入了安全的等效方法，形式为 [`EntityMut::get_components_mut`](https://docs.rs/bevy/latest/bevy/prelude/struct.EntityMut.html#method.get_components_mut) 和 [`EntityWorldMut::get_components_mut`](https://docs.rs/bevy/latest/bevy/prelude/struct.EntityWorldMut.html#method.get_components_mut)：

```rust
let (mut a, mut b) = entity.get_components_mut::<(&mut A, &mut B)>()?;
```

这些方法允许你访问单个实体上多个组件的可变或不可变引用。为了确保我们不会向相同数据发出多个可变引用，这些 API 使用二次时间复杂度（基于访问的组件数量）的运行时检查，在请求非法访问时安全地报错。

二次时间复杂度是个坏消息，但在许多情况下，请求的组件列表非常小：两个、三个甚至十个不同的组件在热循环之外检查并不糟糕。然而，某些应用程序（如脚本接口）可能仍然最适合使用 unsafe API，依赖其他方法以更低的性能成本确保健全性。

## glTF 扩展 [#](https://bevy.org/news/bevy-0-18/#gltf-extensions)

Authors:[@christopherbiscardi](https://github.com/christopherbiscardi)

PRs:[#22106](https://github.com/bevyengine/bevy/pull/22106)

[glTF](https://en.wikipedia.org/wiki/GlTF) 是一种流行的 3D 模型和场景开放格式，作为 Bevy 的主要 3D 格式。然而在制作游戏时，仅依赖对象的内置数据字段是不够的。附加信息如物理碰撞体或特殊渲染属性通常最好直接保存在模型中。

glTF 有两种机制来扩展 glTF 文件中的附加用户数据：extras 和 extensions。

**Extras** 旨在存放任意的应用程序特定数据，通常由用户直接在 Blender 的自定义属性等工具中创建。Bevy 对 extras 有良好的历史支持；如果你在 Blender 中添加自定义属性，该数据将最终出现在相关实体的 [`GltfExtras`](https://docs.rs/bevy/0.18.0/bevy/gltf/struct.GltfExtras.html) 组件之一中。

**Extensions** 旨在存放可跨应用程序共享的数据。它们更加灵活，允许在 glTF 文件内部更多位置添加新数据，因此也更强大。Extensions 可以添加新的对象类型，如来自 `KHR_lights_punctual` 扩展的 `lights`，以及任意缓冲区、glTF 文件根目录的数据等。

在 0.18 之前，处理像 [`KHR_lights_punctual`](https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KH_lights_punctual/README.md) 这样扩展的代码是硬编码在 Bevy 的 glTF 加载器中的。现在，用户可以实现 [`GltfExtensionHandler`](https://docs.rs/bevy/0.18.0/bevy/gltf/extensions/trait.GltfExtensionHandler.html) trait 来对 glTF 数据进行有状态的处理。

处理_扩展_数据只是故事的一半，因为要处理扩展数据，你还必须能够处理非扩展数据，如网格、材质、动画等。

扩展处理器可以为各种用例编写，包括：

- 在实体上插入 Bevy Component 数据
- 将所有 [`Mesh3d`](https://docs.rs/bevy/0.18.0/bevy/mesh/struct.Mesh3d.html) 组件转换为 [`Mesh2d`](https://docs.rs/bevy/0.18.0/bevy/mesh/struct.Mesh2d.html)
- 构建 [`AnimationGraph`](https://docs.rs/bevy/0.18.0/bevy/animation/graph/struct.AnimationGraph.html) 并将其插入到动画根节点
- 用自定义材质替换 [`StandardMaterial`](https://docs.rs/bevy/0.18.0/bevy/pbr/struct.StandardMaterial.html)
- 插入光照贴图

我们添加了两个新示例来展示常见用例：

- [`gltf_extension_animation_graph`](https://github.com/bevyengine/bevy/blob/latest/examples/gltf/gltf_extension_animation_graph.rs) 构建 [`AnimationGraph`](https://docs.rs/bevy/0.18.0/bevy/animation/graph/struct.AnimationGraph.html) 并将其插入到 Scene 中的动画根节点，这意味着当该 Scene 后来被生成时，可以使用同一实体上的 [`AnimationPlayer`](https://docs.rs/bevy/0.18.0-rc.2/bevy/animation/struct.AnimationPlayer.html) 来播放动画。
- [`gltf_extension_mesh_2d`](https://github.com/bevyengine/bevy/blob/latest/examples/gltf/gltf_extension_mesh_2d.rs) 使用 [`GltfExtensionHandler`](https://docs.rs/bevy/0.18.0/bevy/gltf/extensions/trait.GltfExtensionHandler.html) 将 3D 网格和材质组件替换为它们的 2D 对应物。如果你使用 Blender 等软件构建 2D 世界，这非常有用。

### 与外部创作工具的集成 [#](https://bevy.org/news/bevy-0-18/#integration-with-external-authoring-tools)

扩展通常需要一个既_生成_数据又_使用_数据的应用程序。

例如，[Skein](https://github.com/rust-adventure/skein) 定义了一个 glTF 扩展，允许向 glTF 对象添加 Bevy Components。这通常由 Blender 生成，并由 Skein 的 [`GltfExtensionHandler`](https://docs.rs/bevy/0.18.0/bevy/gltf/extensions/trait.GltfExtensionHandler.html) 在 Bevy 中消费。这些组件然后在生成 [`Scene`](https://docs.rs/bevy/0.18.0/bevy/prelude/struct.Scene.html) 时与 [`Transform`](https://docs.rs/bevy/0.18.0/bevy/prelude/struct.Transform.html) 和 [`Mesh3d`](https://docs.rs/bevy/0.18.0/bevy/mesh/struct.Mesh3d.html) 等内置组件一起插入到场景中的实体上。

使用 glTF Extensions 来存放这些数据意味着其他关卡编辑器如 Trenchbroom 也可以将相同格式写入 glTF 文件。任何将组件数据写入 glTF 文件的第三方软件都可以使用 Skein 的 [`GltfExtensionHandler`](https://docs.rs/bevy/0.18.0/bevy/gltf/extensions/trait.GltfExtensionHandler.html)，使得组件在生成 [`Scene`](https://docs.rs/bevy/0.18.0/bevy/prelude/struct.Scene.html) 时"开箱即用"。

## 短类型路径资产处理器 [#](https://bevy.org/news/bevy-0-18/#short-type-path-asset-processors)

Authors:[@andriyDev](https://github.com/andriyDev)

PRs:[#21339](https://github.com/bevyengine/bevy/pull/21339)

资产处理器允许在"发布时"操作资产，将其转换为在运行时加载数据时更优化的形式。这可以使用默认处理器（处理所有具有特定文件扩展名的资产）来完成，也可以在资产的 meta 文件中指定处理器。

在 Bevy 的早期版本中，处理器必须在资产的 meta 文件中**完整**指定。例如：

```ron
(
    meta_format_version: "1.0",
    asset: Process(
        processor: "bevy_asset::processor::process::LoadTransformAndSave<asset_processing::CoolTextLoader, asset_processing::CoolTextTransformer, asset_processing::CoolTextSaver>",
        settings: (
            loader_settings: (),
            transformer_settings: (),
            saver_settings: (),
        ),
    ),
)
```

如你所见，处理器类型可能非常冗长！为了使这些 meta 文件更容易操作，我们现在也支持使用资产的"短类型路径"。这将如下所示：

```ron
(
    meta_format_version: "1.0",
    asset: Process(
        processor: "LoadTransformAndSave<CoolTextLoader, CoolTextTransformer, CoolTextSaver>",
        settings: (
            loader_settings: (),
            transformer_settings: (),
            saver_settings: (),
        ),
    ),
)
```

## 轻松截图和录屏 [#](https://bevy.org/news/bevy-0-18/#easy-screenshot-and-video-recording)

Authors:[@mockersf](https://github.com/mockersf)

PRs:[#21235](https://github.com/bevyengine/bevy/pull/21235), [#21237](https://github.com/bevyengine/bevy/pull/21237)

制作出色的美丽游戏只是成功的一半：你还需要能够向人们展示它！

Bevy 自 0.11 起就能够截取渲染内容的屏幕截图。尽管此功能对于快速创建营销材料非常有用，但设置过程相对复杂。

这个过程已经被简化，新的 [`EasyScreenshotPlugin`](https://docs.rs/bevy/0.18.0/bevy/dev_tools/struct.EasyScreenshotPlugin.html) 允许你按下一个按钮即可截取格式一致的屏幕截图。使用其默认设置，一旦你将此插件添加到应用程序，按下 `PrintScreen` 键时就会截取 PNG 屏幕截图。你可以更改触发键，或在 PNG、JPEG 和 BMP 之间更改屏幕截图格式。

我们更进一步，允许你直接从 Bevy 录制视频，通过新的 [`EasyScreenRecordPlugin`](https://dev-docs.bevy.org/bevy/dev_tools/struct.EasyScreenRecordPlugin.html)。此插件添加了一个切换键（默认为空格键），用于切换屏幕录制。录制也可以通过 [`RecordScreen`](https://dev-docs.bevy.org/bevy/dev_tools/enum.RecordScreen.html) 消息以编程方式启动和停止。

由于视频编解码器方面的挑战，Windows 上目前不支持屏幕录制。虽然所有 dev-tools 功能默认关闭，但由于此限制，屏幕录制的激活稍微复杂一些。要启用它，请在 `bevy_dev_tools` crate 中切换 `screenrecording` feature。

## 从 Schedule 中移除系统 [#](https://bevy.org/news/bevy-0-18/#remove-systems-from-schedules)

Authors:[@hymm](https://github.com/hymm)

PRs:[#20298](https://github.com/bevyengine/bevy/pull/20298)

以前，阻止调度系统运行的唯一方式是使用[运行条件](https://docs.rs/bevy/0.18.0/bevy/prelude/trait.SystemCondition.html)。这对于动态切换系统是否运行很有效，但每次运行调度时都会带来少量开销。

现在，你可以使用 [`remove_systems_in_set`](https://docs.rs/bevy/0.18.0/bevy/prelude/struct.App.html#method.remove_systems_in_set) 完全从调度中移除系统，这会强制进行昂贵的调度重建，但完全移除了该开销并将系统从任何调试工具中移除。

运行条件（以及按需运行的专用调度）仍然是大多数情况下的更好工具，但完全移除系统在选择退出不需要的插件行为、模组或更改游戏设置时可能是一个有吸引力的选项。

```rust
app.add_systems((system_a, (system_b, system_c).in_set(MySet)));

// 移除一个系统
schedule.remove_systems_in_set(my_system, ScheduleCleanupPolicy::RemoveSystemsOnly);

// 移除集合中的系统
app.remove_systems_in_set(MySet, ScheduleCleanupPolicy::RemoveSetAndSystems);
```

## UI 节点可以忽略父级滚动位置 [#](https://bevy.org/news/bevy-0-18/#ui-nodes-can-ignore-parent-scroll-position)

Authors:[@PPakalns](https://github.com/PPakalns)

PRs:[#21648](https://github.com/bevyengine/bevy/pull/21648)

我们添加了 [`IgnoreScroll`](https://docs.rs/bevy/0.18.0/bevy/prelude/struct.IgnoreScroll.html) 组件，用于控制 UI 元素是否沿特定轴忽略其父级的 `ScrollPosition`。

这可用于在可滚动 UI 布局中实现基本的粘性行和列标题。查看 [`scroll` 示例](https://github.com/bevyengine/bevy/blob/latest/examples/ui/scroll.rs) 进行演示！

## 颜色和布局的插值 [#](https://bevy.org/news/bevy-0-18/#interpolation-for-colors-and-layout)

Authors:[@viridia](https://github.com/viridia)

PRs:[#21633](https://github.com/bevyengine/bevy/pull/21633)

Bevy 的 [`StableInterpolate`](https://docs.rs/bevy/0.18.0/bevy/math/trait.StableInterpolate.html) trait 是动画的绝佳基础，但遗憾的是有一个重要的类型它无法处理：来自 `bevy_ui` 的 [`Val`](https://docs.rs/bevy/0.18.0/bevy/prelude/enum.Val.html) 类型，用于控制 UI 元素的布局。[`Val`](https://docs.rs/bevy/0.18.0/bevy/prelude/enum.Val.html) 是一个枚举，表示不同的长度单位如像素和百分比，通常不可能或没有意义尝试在不同单位之间进行插值。

然而，通常需要以不需要混合单位的方式动画 [`Val`](https://docs.rs/bevy/0.18.0/bevy/prelude/enum.Val.html)：通常我们只想滑动或拉伸 widget 如开关的长度。只要我们在运行时检查两个插值控制点是否在同一单位中，我们就可以这样做。

新的 [`TryStableInterpolate`](https://docs.rs/bevy/0.18.0/bevy/math/trait.TryStableInterpolate.html) trait 引入了可能失败的插值概念，通过返回 `Result`。请注意，这里的"失败"不一定是坏事：它只是意味着动画播放器需要以其他方式修改参数，例如"跳转"到新关键帧而不进行平滑插值。这使我们能够创建包含两种参数的复杂动画：一种进行插值，一种不进行插值。

我们为所有实现了 [`StableInterpolate`](https://docs.rs/bevy/0.18.0/bevy/math/trait.StableInterpolate.html) 的类型添加了 [`TryStableInterpolate`](https://docs.rs/bevy/0.18.0/bevy/math/trait.TryStableInterpolate.html) 的 blanket 实现，这些永远不会失败。还有针对 [`Color`](https://docs.rs/bevy/0.18.0/bevy/color/enum.Color.html) 和 [`Val`](https://docs.rs/bevy/0.18.0/bevy/prelude/enum.Val.html) 的额外实现，如果控制点不在同一单位/颜色空间中则可能失败。

## 可寻址资产读取器 [#](https://bevy.org/news/bevy-0-18/#seekable-asset-readers)

Authors:[@andriyDev](https://github.com/andriyDev), [@cart](https://github.com/cart)

PRs:[#22182](https://github.com/bevyengine/bevy/pull/22182)

在 **Bevy 0.15** 中，我们将 `Reader` 上的 `AsyncSeek` 超级 trait 替换为 `AsyncSeekForward`。这允许我们的 `Reader` trait 适用于更多场景（例如，它可能允许像 HTTP 请求这样不支持向后寻址的情况）。但这也意味着当寻址功能可用时，我们无法再完全使用它。

在 **Bevy 0.18** 中，当资产源支持时，我们可以将 `Reader` trait 升级为 `SeekableReader`。

```rust
let seekable_reader = reader.seekable()?;
seekable_reader.seek(SeekFrom::Start(10)).await?;
```

这使得需要寻址的 `AssetLoader` 可以失败，或者为其用例选择合适的回退行为（例如读取到 `Vec`，它是可寻址的）。