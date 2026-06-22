# Bevy 0.15

## 发布于 2024 年 11 月 29 日，Bevy 贡献者

![被体积光照射的蛇形雕像，笼罩在体积雾中](https://bevy.org/news/bevy-0-15/cover.png)

[被体积光照射的蛇形雕像，笼罩在体积雾中](https://sketchfab.com/3d-models/snake-statue-794b77a3e4654a669cf259d20dc89ec7)

感谢 **294** 位贡献者、**1217** 个拉取请求、社区审阅者以及我们的[**慷慨捐赠者**](https://bevy.org/donate)，我们很高兴在 [crates.io](https://crates.io/crates/bevy) 上发布 **Bevy 0.15**！

对于那些还不了解的人，Bevy 是一个用 Rust 构建的、清新简洁的数据驱动游戏引擎。你可以查看我们的[快速入门指南](https://bevy.org/learn/quick-start)立即体验。它是免费的，永远开源！你可以在 GitHub 上获取完整的[源代码](https://github.com/bevyengine/bevy)。查看 [Bevy Assets](https://bevy.org/assets) 获取社区开发的插件、游戏和学习资源合集。

要将现有的 Bevy App 或 Plugin 更新到 **Bevy 0.15**，请查看我们的 [0.14 到 0.15 迁移指南](https://bevy.org/learn/migration-guides/0-14-to-0-15/)。

自从上次发布几个月以来，我们添加了大量新功能、Bug 修复和生活质量改进，以下是一些亮点：

- **Required Components（必需组件）**：重新思考了实体生成机制，显著改善了 Bevy 用户体验
- **Entity Picking / Selection（实体拾取/选择）**：跨上下文选择实体的模块化系统
- **Animation Improvements（动画改进）**：通用实体动画、动画遮罩、叠加混合和动画事件
- **Curves（曲线）**：新的 `Curve` trait、循环样条、常用缓动函数、颜色渐变曲线
- **Reflection Improvements（反射改进）**：函数反射、唯一反射、远程类型反射
- **Bevy Remote Protocol (BRP)（Bevy 远程协议）**：允许外部客户端（如编辑器）与运行中的 Bevy 游戏交互的新协议
- **Visibility Bitmask Ambient Occlusion (VBAO)（可见性位掩码环境光遮蔽）**：改进的 GTAO 算法，提升了环境光遮蔽质量
- **Chromatic Aberration（色差）**：新的后处理效果，模拟镜头无法将光线聚焦到同一点的现象
- **Volumetric Fog Improvements（体积雾改进）**："雾体积"定义了体积雾的渲染位置和形态，同时支持点光源和聚光灯
- **Order Independent Transparency（顺序无关透明度）**：新的可选透明度算法，改善了透明物体随相机距离变化的稳定性/质量
- **Improved Text Rendering（改进的文本渲染）**：我们已切换到 Cosmic Text 进行文本渲染，显著提升了文本渲染能力，特别是对于需要字体塑形和双向文本的非拉丁语言
- **Gamepads as Entities（手柄作为实体）**：手柄现在以实体表示，使交互更加简便
- **UI Box Shadows（UI 盒阴影）**：Bevy UI 节点现在可以渲染可配置的盒阴影

Bevy 0.15 是使用我们新的**发布候选**流程准备的，旨在让你能够放心地立即升级。我们与插件作者和普通用户密切合作，捕捉关键 Bug、打磨新功能并完善迁移指南。对于每个发布候选，我们都会准备修复、[在 crates.io 上发布新的候选版本](https://crates.io/crates/bevy/versions?sort=date)，让核心生态系统的 crate 更新，并密切关注阻塞性问题。衷心感谢[所有提供帮助的人](https://discord.com/channels/691052431525675048/1295069829740499015)！这些努力是使 Bevy 成为大团队和小团队都能信赖的可靠工具的关键一步。

## Required Components（必需组件）[#](https://bevy.org/news/bevy-0-15/#required-components)

作者：[@cart](https://github.com/cart), [@Jondolf](https://github.com/Jondolf)

PR：[#14791](https://github.com/bevyengine/bevy/pull/14791), [#15458](https://github.com/bevyengine/bevy/pull/15458), [#15269](https://github.com/bevyengine/bevy/pull/15269)

![精灵组件需要变换组件的图示](https://bevy.org/news/bevy-0-15/required_component.svg)

首先：系好安全带，因为 **Required Components** 是自 Bevy 首次发布以来对 Bevy API 层面最深刻的改进之一。

自 Bevy 创建以来，`Bundle` 一直是我们生成特定"类型"实体的抽象。`Bundle` 就是一个 Rust 类型，其中每个字段都是一个 `Component`：

```rust
#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    team: Team,
    sprite: Sprite,
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
}
```

然后每当需要生成一个新玩家时，开发者会初始化并在实体上插入一个 `PlayerBundle`：

```rust
commands.spawn(PlayerBundle {
    player: Player { 
        name: "hello".into(),
        ..default()
    },
    team: Team::Blue,
    ..default()
});
```

这会插入 `PlayerBundle` 中的所有组件，包括那些没有明确设置的。`Bundle` 的概念是可用的（它让我们走到了今天），但它远非理想：

1. 这是一套开发者需要学习的全新 API。想要生成 `Player` 实体的人需要知道 `PlayerBundle` 存在。
2. Bundle API 在插入后就不存在于运行时了……它们是额外的、仅存在于生成时的概念，开发者需要额外考虑。你不会编写 `PlayerBundle` 的行为，你编写的是 `Player` 的行为。
3. `Player` 组件*需要* `PlayerBundle` 中的组件才能作为 `Player` 运作。单独生成 `Player` 是可能的，但很可能（取决于实现）无法按预期工作。
4. Bundle 总是"扁平的"（按惯例）。定义 `Player` 组件的人需要定义*所有组件依赖*。`Sprite` 需要 `Transform` 和 `Visibility`，`Transform` 需要 `GlobalTransform`，`Visibility` 需要 `InheritedVisibility` 和 `ViewVisibility`。这种"依赖继承"的缺失使得定义 bundle 变得比需要的更困难且容易出错。它要求 API 的使用者深入了解本质上是实现细节的东西。当这些细节发生变化时，*`Bundle` 的开发者需要知道并相应地更新 `Bundle`*。支持嵌套 bundle，但对于用户来说使用起来很*痛苦*，而且我们在上游 Bevy bundle 中已经禁止使用它们一段时间了。
5. `PlayerBundle` 实际上是由 `Player` 组件的需求定义的，但在生成时有可能*完全不提及 `Player` 符号*。例如：`commands.spawn(PlayerBundle::default())`。考虑到 `Player` 是"驱动概念"，这很奇怪。
6. Bundle 在 API 中引入了显著的"重复"。注意上面示例中的 `player: Player` 和 `team: Team`。
7. Bundle 引入了额外的（可以说是过度的）嵌套和 `..default()` 使用。

以上每一点都对日常使用 Bevy 的体验有相当大的影响。在 **Bevy 0.15** 中，我们引入了 **Required Components**，通过从根本上重新思考这一切的工作原理来解决这些问题。

**Required Components** 是我们[下一代场景/UI](https://github.com/bevyengine/bevy/discussions/14437) 工作的第一步，旨在使 Bevy 成为一流的应用/场景/UI 开发框架。**Required Components** 本身就直接改善了 Bevy 开发者的生活，同时它们也为使 Bevy 即将推出的下一代场景系统（以及即将推出的 Bevy Editor）真正特别奠定了基础。

### 它们是什么？

**Required Components** 使开发者能够定义给定组件需要哪些组件：

```rust
#[derive(Component, Default)]
#[require(Team, Sprite)]
struct Player {
    name: String,
}
```

当插入 `Player` 组件时，它的 **Required Components** *以及这些组件所需的组件* 也会自动插入！

```rust
commands.spawn(Player::default());
```

上面的代码自动插入了 `Team` 和 `Sprite`。`Sprite` 需要 `Transform` 和 `Visibility`，所以它们也被自动插入。同样，`Transform` 需要 `GlobalTransform`，`Visibility` 需要 `InheritedVisibility` 和 `ViewVisibility`。

这段代码产生的结果与前一节的 `PlayerBundle` 代码相同：

```rust
commands.spawn((
    Player {
        name: "hello".into(),
        ..default()
    },
    Team::Blue,
))
```

好多了，对吧？`Player` 类型的定义更简单、更不容易出错，生成它的代码更少且更易读。

### 效率

我们实现 **Required Components** 的方式使其实际上是"免费的"：

1. 如果调用者没有手动插入 Required Components，它们才会被初始化和插入。没有冗余！
2. Required Components 与普通组件一起插入，这意味着（对于 ECS 爱好者来说）没有额外的原型变更或表移动。从这个角度看，`Player` 示例的 Required Components 版本与手动预先定义所有组件的 `PlayerBundle` 方法是相同的。
3. Required Components 缓存在原型图上，这意味着计算给定插入类型需要哪些必需组件只发生一次。

### 组件初始化

默认情况下，**Required Components** 会使用该组件的 `Default` 实现（如果不存在则会编译失败）：

```rust
#[derive(Component)]
#[require(Team)] // Team::Red 是默认值
struct Player {
    name: String,
}

#[derive(Component, Default)]
enum Team {
    #[default]
    Red,
    Blue,
}
```

可以通过传入一个返回组件的函数来覆盖：

```rust
#[derive(Component)]
#[require(Team(blue_team))]
struct Player {
    name: String,
}

fn blue_team() -> Team {
    Team::Blue
}
```

为了节省空间，你也可以直接在 require 中传入一个闭包：

```rust
#[derive(Component)]
#[require(Team(|| Team::Blue))]
struct Player {
    name: String,
}
```

### 这有点像继承吗？

**Required Components** *可以*被认为是一种继承形式。但它显然*不是*传统的面向对象继承。相反，它是"通过组合实现的继承"。一个 `Button` 控件应该（并且应该）要求 `Node` 使其成为"UI 节点"。在某种意义上，`Button` "是一个" `Node`，就像在传统继承中一样。但与传统的继承不同：

1. 它被表达为"有一个"关系，而不是"是一个"关系。
2. `Button` 和 `Node` 仍然是两个完全独立的类型（各自拥有自己的数据），在 ECS 中需要分别查询。
3. `Button` 可以*额外*要求更多组件，而不仅仅是 `Node`。你不受限于"直线型"的标准面向对象继承。组合仍然是主导模式。
4. 你*不需要*要求组件才能添加它们。你仍然可以在生成时以正常的"组合风格"附加任何你想要的额外组件来添加行为。

### Bundles 会怎样？

`Bundle` trait 将继续存在，它仍然是 insert API 的基本构建块（组件元组仍然实现 `Bundle`）。开发者仍然可以使用 `Bundle` 派生宏定义自己的自定义 bundle。Bundles 与 **Required Components** 配合良好，因此你可以将它们结合使用。

也就是说，从 Bevy **0.15** 开始，我们已弃用所有内置 bundle，如 `SpriteBundle`、`NodeBundle`、`PbrBundle` 等，转而使用 **Required Components**。总的来说，**Required Components** 现在是首选/惯用的方法。我们鼓励 Bevy 插件和应用开发者将他们的 bundle 迁移到 **Required Components**。

### 将 Bevy 移植到 Required Components

如上所述，*所有*内置 Bevy bundle 已弃用，转而使用 **Required Components**。我们还进行了 API 更改以利用这一新范式。这确实意味着在某些地方有破坏性变更，但这些变更非常好，我们认为人们不会抱怨太多 :)

总的来说，我们正在朝着我们的[下一代场景/UI](https://github.com/bevyengine/bevy/discussions/14437) 文档指定的方向前进。一些一般性设计指南：

1. 生成实体时，通常应该有一个"驱动概念"组件。实现新的实体类型/行为时，给它一个概念名称……这就是你的"驱动组件"的名称（例如："player"概念就是 `Player` 组件）。该组件应要求执行其功能所需的任何额外组件。
2. 人们应该直接以组件及其字段的方式思考生成。更倾向于在"概念组件"上直接使用组件字段作为该功能的"公共 API"。
3. 更倾向于简单的 API / 不要过度组件化。默认情况下，如果你需要给概念附加新属性，只需将它们作为字段添加到该概念的组件中。只有在你有一个好理由时才拆分成新的组件/概念，并且这个理由是由用户体验或性能驱动的（并且用户体验权重更高）。如果一个给定的"概念"（例如：`Sprite`）被拆分成 10 个组件，用户将*非常*难以理解和操作。
4. 不要直接使用 Asset handle 作为组件，而是定义包含必要 handle 的新组件。原始 Asset handle 作为组件存在各种问题（一个主要问题是无法为它们定义上下文特定的 **Required Components**），因此我们移除了 `Handle<T>` 的 `Component` 实现，以鼓励（好吧……强制）人们采用这种模式。

#### UI

Bevy UI 从 **Required Components** 中受益匪浅。UI 节点需要多种组件才能正常工作，现在所有这些需求都集中在 `Node` 上。定义一个新的 UI 节点类型现在就像给你的组件添加 `#[require(Node)]` 一样简单。

```rust
#[derive(Component)]
#[require(Node)]
struct MyNode;

commands.spawn(MyNode);
```

`Style` 组件的字段已移至 `Node`。`Style` 从来就不是一个全面的"样式表"，而只是所有 UI 节点共享的属性集合。一个"真正的"ECS 样式系统会跨组件（`Node`、`Button` 等）设置样式属性，我们[确实有计划构建一个真正的样式系统](https://github.com/bevyengine/bevy/discussions/14437)。所有"计算后"的节点属性（如布局后节点的大小）已移至 `ComputedNode` 组件。

这一变更使得在 Bevy 中生成 UI 节点*更加*清晰和简洁：

```rust
commands.spawn(Node {
    width: Val::Px(100.),
    ..default()
});
```

与之前的方式对比！

```rust
commands.spawn(NodeBundle {
    style: Style {
        width: Val::Px(100.),
        ..default()
    },
    ..default()
})
```

像 `Button`、`ImageNode`（之前是 `UiImage`）和 `Text` 这样的 UI 组件现在都需要 `Node`。值得注意的是，`Text` 已经过重新设计，更易于使用且更组件驱动（我们将在下一节中详细介绍）：

```rust
commands.spawn(Text::new("Hello there!"));
```

`MaterialNode<M: UiMaterial>` 现在是一个用于"UI 材质着色器"的合适组件，它也要求 `Node`：

```rust
commands.spawn(MaterialNode(my_material));
```

#### Text

Bevy 的 Text API 已经过重新设计，更简单且更组件驱动。仍然有两个主要的文本组件：`Text`（UI 文本组件）和 `Text2d`（世界空间 2D 文本组件）。

首先改变的是这些主要组件实际上*只是*一个 `String` 的新类型包装：

```rust
commands.spawn(Text("hello".to_string()))
commands.spawn(Text::new("hello"))
commands.spawn(Text2d("hello".to_string()))
commands.spawn(Text2d::new("hello"))
```

生成这些组件之一，你就有文本了！这两个组件现在都需要以下组件：

- `TextFont`：配置字体/大小
- `TextColor`：配置颜色
- `TextLayout`：配置文本的布局方式。

`Text`（UI 组件）还需要 `Node`，因为它*是*一个节点。类似地，`Text2d` 需要一个 `Transform`，因为它位于世界空间。

`Text` 和 `Text2d` 都是一个独立的文本"块"。这些顶层文本组件也会贡献一个文本"跨度"到该"块"。如果你需要具有多种颜色/字体/大小的"富文本"，你可以将 `TextSpan` 实体作为 `Text` 或 `Text2d` 的子级添加。`TextSpans` 使用相同的 `TextFont`/`TextLayout` 组件来配置文本。每个 `TextSpan` 会将其跨度贡献给父级文本：

```rust
// `Text` UI 节点将渲染"hello world!"，其中"hello"为红色，"world!"为蓝色
commands.spawn(Text::default())
    .with_child((
        TextSpan::new("hello"),
        TextColor::from(RED),
    ))
    .with_child((
        TextSpan::new(" world!"),
        TextColor::from(BLUE),
    ));
```

这产生完全相同的输出，但在顶层 `Text` 组件上使用了"默认"跨度：

```rust
commands.spawn((
    Text::new("hello"),
    TextColor::from(RED),
))
.with_child((
    TextSpan::new(" world!"),
    TextColor::from(BLUE),
));
```

这种"实体驱动"的文本跨度方法取代了之前 Bevy 版本中使用的"内部跨度数组"方法。这带来了显著的好处。首先，它允许你使用正常的 Bevy ECS 工具，如标记组件和查询，来标记文本跨度并直接访问它。这比使用数组中的索引（难以猜测且随数组内容变化而不稳定）更容易且更具弹性：

```rust
#[derive(Component)]
struct NameText;

commands.spawn(Text::new("Name: "))
    .with_child((
        TextSpan::new("Unknown"),
        NameText, 
    ));

fn set_name(mut names: Query<&mut TextSpan, With<NameText>>) {
    names.single_mut().0 = "George".to_string();
}
```

文本跨度作为实体与 Bevy Scenes（包括即将推出的[下一代场景/UI](https://github.com/bevyengine/bevy/discussions/14437) 系统）配合得更好，并且允许它与现有的工具（如实体检查器、动画系统、计时器等）很好地集成。

#### Sprites

Sprites 基本保持不变。除了 **Required Components** 移植（`Sprite` 现在需要 `Transform` 和 `Visibility`）之外，我们还进行了一些组件合并。`TextureAtlas` 组件现在是一个可选的 `Sprite::texture_atlas` 字段。同样，`ImageScaleMode` 组件现在是 `Sprite::image_mode` 字段。生成 sprite 现在超级简单！

```rust
commands.spawn(Sprite {
    image: assets.load("player.png"),
    ..default()
});
```

#### Transforms

`Transform` 现在需要 `GlobalTransform`。如果你希望你的实体具有"层次化变换"，请要求 `Transform`（它会添加 `GlobalTransform`）。如果你只希望你的实体具有"扁平"的全局变换，请要求 `GlobalTransform`。

大多数旨在存在于世界空间的 Bevy 组件现在都需要 `Transform`。

#### Visibility

`Visibility` 组件现在需要 `InheritedVisibility` 和 `ViewVisibility`，这意味着如果你希望实体可见，现在只需要求 `Visibility`。Bevy 的内置"可见"组件，如 `Sprite`，需要 `Visibility`。

#### Cameras

`Camera2d` 和 `Camera3d` 组件现在各自需要 `Camera`。`Camera` 需要各种相机组件（`Frustum`、`Transform` 等）。这意味着你可以像这样生成 2D 或 3D 相机：

```rust
commands.spawn(Camera2d::default());
commands.spawn(Camera3d::default());
```

`Camera2d` 和 `Camera3d` 还需要设置相关默认渲染图并启用 2D 和 3D 上下文相关默认渲染功能的组件。

你当然可以显式设置其他组件的值：

```rust
commands.spawn((
    Camera3d::default(),
    Camera {
        hdr: true,
        ..default()
    },
    Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        ..default()
    },
));
```

Bevy 有许多启用"相机渲染功能"的组件：`MotionBlur`、`TemporalAntiAliasing`、`ScreenSpaceAmbientOcclusion` 和 `ScreenSpaceReflections`。其中一些相机功能依赖于*其他*相机功能组件才能正常运行。这些依赖关系现在通过 **Required Components** 表达和强制执行。例如，`MotionBlur` 现在需要 `DepthPrepass` 和 `MotionVectorPrepass`。这使得启用相机功能变得更加容易！

```rust
commands.spawn((
    Camera3d::default(),
    MotionBlur,
))
```

#### Meshes

旧的网格方法依赖于直接添加 `Handle<Mesh>` 和 `Handle<M: Material>` 组件（通过 `PbrBundle` 和 `MaterialMeshBundle`），这两者都与 required components 不兼容。

在 **Bevy 0.15** 中，你使用 `Mesh3d` 和 `MeshMaterial3d<M: Material>` 在 3D 中渲染网格：

```rust
commands.spawn((
    Mesh3d(mesh),
    MeshMaterial3d(material),
));
```

`Mesh3d` 需要 `Transform` 和 `Visibility`。

还有 2D 等效组件：

```rust
commands.spawn((
    Mesh2d(mesh),
    MeshMaterial2d(material),
));
```

#### Meshlets

Bevy 的"虚拟几何"实现（类似于 Nanite）也已移植。它使用与 `Mesh3d` 和 `Mesh2d` 相同的模式：

```rust
commands.spawn((
    MeshletMesh3d(mesh),
    MeshMaterial3d(material),
));
```

#### Lights

光源的移植不涉及组件结构的重大变化。所有空间光源类型（`PointLight`、`DirectionalLight`、`SpotLight`）现在都需要 `Transform` 和 `Visibility`，并且每个光源组件都需要相关的光源特定配置组件（例如：`PointLight` 需要 `CubemapFrusta` 和 `CubemapVisibleEntities`）。

现在生成特定类型的光源非常简单：

```rust
commands.spawn(PointLight {
    intensity: 1000.0,
    ..default()
});
```

`LightProbe` 组件现在也需要 `Transform` 和 `Visibility`。

#### Volumetric Fog

`FogVolume` 组件现在需要 `Transform` 和 `Visibility`，这意味着你现在可以像这样添加体积雾：

```rust
commands.spawn(FogVolume {
    density_factor: 0.2,
    ..default()
});
```

#### Scenes

场景以前使用原始的 `Handle<Scene>` 组件，通过 `SceneBundle` 生成。**Bevy 0.15** 引入了 `SceneRoot` 组件，它包装了场景 handle 并需要 `Transform` 和 `Visibility`：

```rust
commands.spawn(SceneRoot(some_scene));
```

同样，现在有 `DynamicSceneRoot`，它与 `SceneRoot` 完全相同，但它包装的是 `Handle<DynamicScene>` 而不是 `Handle<Scene>`。

#### Audio

音频也使用了通过 `AudioBundle` 生成的原始 `Handle<AudioSource>`。我们添加了一个新的 `AudioPlayer` 组件，生成时会触发音频播放：

```rust
command.spawn(AudioPlayer(assets.load("music.mp3")));
```

`AudioPlayer` 需要 `PlaybackSettings` 组件。

来自任意 `Decodable` trait 实现的非标准音频可以使用 `AudioSourcePlayer` 组件，它也要求 `PlaybackSettings`。

### IDE 集成

**Required Components** 与 Rust Analyzer 配合良好。你可以在 required components 上"转到定义"/按 `F12` 来导航到它们在代码中定义的位置。

### 运行时 Required Components

在某些情况下，无法直接控制某个组件的开发者可能希望在该类型通过 `#[require(Thing)]` 直接提供的 Required Components 之上添加*额外的* **Required Components**。这是支持的！

```rust
// 让 `Bird` 要求 `Wings`，使用 `Default` 构造器。
app.register_required_components::<Bird, Wings>();

// 让 `Wings` 要求 `FlapSpeed`，使用自定义构造器。
app.register_required_components_with::<Wings, FlapSpeed>(|| FlapSpeed::from_duration(1.0 / 80.0));
```

请注意，只允许*添加* Required Components。显式不支持从你不拥有的类型中移除 Required Components，因为这可能会使上游的假设失效。

总的来说，这应该用于非常具体、有针对性的场景，比如物理插件为其无法控制的核心类型添加额外的元数据。添加新的组件要求可能会改变应用的性能特征或以意外方式破坏应用。如果有疑问，就不要做！

## Chromatic Aberration（色差）[#](https://bevy.org/news/bevy-0-15/#chromatic-aberration)

作者：[@pcwalton](https://github.com/pcwalton)

PR：[#13695](https://github.com/bevyengine/bevy/pull/13695)

我们添加了[色差](https://en.wikipedia.org/wiki/Chromatic_aberration)效果，这是一种常见的后处理效果，模拟镜头无法将所有颜色的光线聚焦到同一点的现象。它常用于冲击效果和/或恐怖游戏。我们的实现使用了 Inside 游戏中的技术（Gjøl & Svendsen 2016），允许开发者自定义特定的颜色模式以实现不同的效果。

![色差效果](https://bevy.org/news/bevy-0-15/chromatic_aberration.png)

要使用它，将 [`ChromaticAberration`](https://docs.rs/bevy/0.15/bevy/core_pipeline/post_process/struct.ChromaticAberration.html) 组件添加到你的相机：

```rust
commands.spawn((Camera3d::default(), ChromaticAberration));
```

## Visibility Bitmask Ambient Occlusion (VBAO)（可见性位掩码环境光遮蔽）[#](https://bevy.org/news/bevy-0-15/#visibility-bitmask-ambient-occlusion-vbao)

作者：[@dragostis](https://github.com/dragostis)

PR：[#13454](https://github.com/bevyengine/bevy/pull/13454)

**Bevy 0.15** 引入了一种新的屏幕空间环境光遮蔽（SSAO）算法：[Visibility Bitmask Ambient Occlusion](https://arxiv.org/abs/2301.11376)（VBAO）。VBAO 在 GTAO 的基础上添加了一个位掩码，允许遮挡所考虑半圆的多个"扇区"，而不仅仅是一个角度。这提高了技术的准确性，在薄几何体（如下面场景中的椅子腿）上尤其重要：

拖动此图像进行比较

![GTAO](https://bevy.org/news/bevy-0-15/gtao.jpg)![VBAO](https://bevy.org/news/bevy-0-15/vbao.jpg)

VBAO 带来了足够显著的质量改进，我们已完全替换了旧的 GTAO 算法。只需将现有的 [`ScreenSpaceAmbientOcclusion`](https://docs.rs/bevy/0.15/bevy/pbr/struct.ScreenSpaceAmbientOcclusion.html) 组件添加到你的相机即可启用。

## Volumetric Fog Support for Point Lights and Spotlights（点光源和聚光灯的体积雾支持）[#](https://bevy.org/news/bevy-0-15/#volumetric-fog-support-for-point-lights-and-spotlights)

作者：[@Soulghost](https://github.com/Soulghost)

PR：[#15361](https://github.com/bevyengine/bevy/pull/15361)

体积雾是在 [Bevy 0.14 中引入的](https://bevy.org/news/bevy-0-14/#volumetric-fog-and-volumetric-lighting-light-shafts-god-rays)。最初，只有方向光可以与之交互。在 Bevy 0.15 中，点光源和聚光灯也支持体积雾：

![体积雾](https://bevy.org/news/bevy-0-15/volumetric_fog.jpg)

要为场景添加体积雾，将 [VolumetricFog](https://docs.rs/bevy/0.15/bevy/pbr/struct.VolumetricFog.html) 添加到相机，并将 [VolumetricLight](https://docs.rs/bevy/0.15/bevy/pbr/struct.VolumetricLight.html) 添加到你想要体积雾效果的方向光、点光源或聚光灯。

```rust
// 将 VolumetricFog 添加到相机。
commands
    .spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            ..default()
        },
    ))
    .insert(VolumetricFog {
        // 此值显式设为 0，因为我们没有环境贴图光源。
        ambient_intensity: 0.0,
        ..default()
    });

// 将 VolumetricLight 添加到点光源。
commands.spawn((
    PointLight {
        shadows_enabled: true,
        range: 150.0,
        color: RED.into(),
        intensity: 1000.0,
        ..default()
    },
    VolumetricLight,
    Transform::from_xyz(-0.4, 1.9, 1.0),
));

// 将 VolumetricLight 添加到聚光灯。
commands.spawn((
    SpotLight {
        intensity: 5000.0, // 流明
        color: Color::WHITE,
        shadows_enabled: true,
        inner_angle: 0.76,
        outer_angle: 0.94,
        ..default()
    },
    VolumetricLight,
    Transform::from_xyz(-1.8, 3.9, -2.7).looking_at(Vec3::ZERO, Vec3::Y),
));
```

## Fog Volumes（雾体积）[#](https://bevy.org/news/bevy-0-15/#fog-volumes)

作者：[@pcwalton](https://github.com/pcwalton), [@jirisvd](https://github.com/jirisvd)

PR：[#14099](https://github.com/bevyengine/bevy/pull/14099), [#14868](https://github.com/bevyengine/bevy/pull/14868)

**Bevy 0.15** 添加了"雾体积"的概念。这些是带有 [`FogVolume`](https://docs.rs/bevy/0.15.0/bevy/pbr/struct.FogVolume.html) 组件的实体，它定义了雾的包围盒，可以通过缩放和定位来定义雾的渲染位置。

带有 [`VolumetricFog`](https://docs.rs/bevy/0.15.0/bevy/pbr/struct.VolumetricFog.html) 组件的相机将渲染其视野中的任何 [`FogVolume`](https://docs.rs/bevy/0.15.0/bevy/pbr/struct.FogVolume.html) 实体。雾体积还可以定义密度纹理，这是一种指定每个点雾密度的 3D 体素纹理：

![雾体积](https://bevy.org/news/bevy-0-15/fog_volume.png)

[`FogVolume`](https://docs.rs/bevy/0.15.0/bevy/pbr/struct.FogVolume.html) 有一个 `density_texture_offset`，允许 3D 纹理"滚动"。这可以实现诸如云彩"穿过"体积的效果：

## Order Independent Transparency（顺序无关透明度）[#](https://bevy.org/news/bevy-0-15/#order-independent-transparency)

作者：[@IceSentry](https://github.com/IceSentry)

PR：[#14876](https://github.com/bevyengine/bevy/pull/14876)

在此之前，Bevy 仅使用 alpha 混合来渲染透明网格。我们现在可以选择在渲染透明网格时使用顺序无关透明度。这不仅仅是排序网格，而是对每个贡献给透明三角形的像素进行排序。如果你的场景中有很多透明层，这将非常有用。

目前的实现相当简单，使用大量 GPU 内存，但只要配置了足够的层数，它应该始终渲染出完美准确的透明度。

此功能仍在开发中，我们将继续改进。

此功能由 Foresight Spatial Labs 贡献给 Bevy。它基于他们在应用程序中使用的内部实现。

## User-Friendly CPU Drawing（用户友好的 CPU 绘制）[#](https://bevy.org/news/bevy-0-15/#user-friendly-cpu-drawing)

作者：[@inodentry](https://github.com/inodentry)

PR：[#10392](https://github.com/bevyengine/bevy/pull/10392)

有很多情况下你可能想直接从 CPU 代码设置像素的颜色。程序化资源、某些艺术风格，或者仅仅因为它更简单。当你只想更改几个特定像素时，不需要折腾着色器和材质！

在之前的 Bevy 版本中，这很困难且繁琐。Bevy 允许你访问 [`Image`](https://docs.rs/bevy/0.15/bevy/prelude/struct.Image.html) 的原始数据字节，但你必须计算目标像素坐标对应的字节偏移量，确保根据 [`TextureFormat`](https://docs.rs/bevy/0.15/bevy/render/render_resource/enum.TextureFormat.html) 正确编码字节，等等。非常底层！

在 Bevy 0.15 中，现在有了用户友好的 API 来读取和写入 [`Image`](https://docs.rs/bevy/0.15/bevy/prelude/struct.Image.html) 中像素的颜色。棘手的底层细节已经为你处理好了！你甚至可以使用 `bevy_color` 的炫酷色彩空间 API！

```rust
fn my_system(mut images: ResMut<Assets<Image>>, mut commands: Commands) {
    // 创建一个新图像。
    let mut image = Image::new_fill(
        // 64x64 尺寸
        Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &Srgba::WHITE.to_u8_array(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::all(),
    );

    // 这是新增的：

    // 将 x:23, y:32 的像素设为洋红色
    image.set_color_at(23, 32, Color::srgb(1.0, 0.0, 1.0))
        .expect("写入颜色时出错");

    // 使用 Oklch 色彩空间指定颜色设置 10,10 处的像素：
    image.set_color_at(10, 10, Color::oklch(0.3, 0.2, 0.5))
        .expect("写入颜色时出错");

    // 读取我们刚写入的像素的字节：
    let bytes = image.pixel_bytes(UVec3::new(10, 10, 0)).unwrap();

    // 读取（近似的）颜色（以 sRGB 格式）：
    let color = image.get_color_at(10, 10);

    // 我们可以将新图像添加到 Bevy 的资产中
    // 并生成一个 sprite 来显示它：
    commands.spawn(Sprite {
        image: images.add(image),
        ..default()
    });
}
```

注意：[`Color`](https://docs.rs/bevy/0.15/bevy/color/enum.Color.html) 相关的方法是有损的。它们需要转换为/从 [`Image`](https://docs.rs/bevy/0.15/bevy/prelude/struct.Image.html) 的 [`TextureFormat`](https://docs.rs/bevy/0.15/bevy/render/render_resource/enum.TextureFormat.html)。如果你读回写入的颜色，会略有不同。

## Entity Picking / Selection（实体拾取/选择）[#](https://bevy.org/news/bevy-0-15/#entity-picking-selection)

作者：[@aevyrie](https://github.com/aevyrie), [@NthTensor](https://github.com/NthTensor), [@TotalKrill](https://github.com/TotalKrill), [@jnhyatt](https://github.com/jnhyatt), [@Jondolf](https://github.com/Jondolf)

PR：[#13677](https://github.com/bevyengine/bevy/pull/13677), [#14686](https://github.com/bevyengine/bevy/pull/14686), [#14695](https://github.com/bevyengine/bevy/pull/14695), [#14757](https://github.com/bevyengine/bevy/pull/14757), [#15800](https://github.com/bevyengine/bevy/pull/15800)

![一组几何形状，指针显示悬停网格上的一个点。指示器垂直于表面。](https://bevy.org/news/bevy-0-15/mesh_picking.png)

在任何游戏中，能够点击对象来选择它们是一项至关重要且看似简单的任务。自 2020 年以来，在 Bevy 中实现这一点主要意味着引入 `@aevyrie` 备受喜爱的生态 crate [`bevy_mod_picking`](https://crates.io/crates/bevy_mod_picking/) 及其简单的射线投射伴侣 [`bevy_mod_raycast`](https://crates.io/crates/bevy_mod_raycast/)。

多年来，这个 crate 已经过精炼和实战测试，由 [Foresight Spatial Labs](https://www.fslabs.ca/)（一家使用 Bevy 的 CAD 创建公司，Aevyrie 工作的地方）和更广泛的开源游戏开发者社区共同打磨，被用于从第一人称射击游戏到点击冒险游戏的各种场景。Bevy 非常荣幸有机会与 [`bevy_mod_picking`](https://crates.io/crates/bevy_mod_picking/) 背后的团队合作，并将整个项目采纳到 Bevy 中。集成一个大项目需要大量工作，我们非常感谢使 `bevy_picking` 成为引擎稳定的一流功能的贡献者。

新的 `bevy_picking` crate 紧密遵循现有的模块化架构：

1. 从鼠标、触摸和笔设备收集输入。每个指向设备（人类默认配备 10 个）获得一个屏幕空间的 [`PointerLocation`](https://docs.rs/bevy/0.15.0/bevy/picking/backend/prelude/struct.PointerLocation.html)。
2. 每个模块化的 [backend](https://docs.rs/bevy/0.15.0/bevy/picking/backend/index.html) 执行领域特定的工作（如射线投射），计算出这些指针位置如何映射到它们正在观察的对象上的 [`PointerHits`](https://docs.rs/bevy/0.15.0/bevy/picking/backend/struct.PointerHits.html)。
3. 来自每个后端的命中信息被合并和排序，以生成一个连贯的 [`HoverMap`](https://docs.rs/bevy/0.15.0/bevy/picking/focus/struct.HoverMap.html)，它列出了每个指针悬停的实体。
4. 为每个悬停的实体触发高级事件（包括普通事件和观察者事件！），捕获复杂的行为，如点击、拖拽或释放各种对象。

在 Bevy 0.15 中，我们提供了三个第一方拾取后端，分别用于 UI、精灵和网格。目前每个后端都有各自的注意事项：

- UI：旧的 [`Interaction`](https://docs.rs/bevy/0.15.0/bevy/prelude/enum.Interaction.html) 和新 [`PickingInteraction`](https://docs.rs/bevy/0.15.0/bevy/picking/focus/enum.PickingInteraction.html) 组件[暂时](https://github.com/bevyengine/bevy/issues/15550)共存，具有微妙的行文差异。
- Sprite：拾取总是使用完整的矩形，并且[不考虑 alpha 透明度](https://github.com/bevyengine/bevy/issues/14929)。
- Mesh：这是对整个网格的朴素射线投射。如果你遇到性能问题，应该使用简化的网格和加速数据结构（如 [BVH](https://en.wikipedia.org/wiki/Bounding_volume_hierarchy)）来加速。因此，此功能默认是禁用的。可以通过添加 [`MeshPickingPlugin`](https://docs.rs/bevy/0.15.0/bevy/picking/mesh_picking/struct.MeshPickingPlugin.html) 来启用。

我们预计 [`bevy_rapier`](https://crates.io/crates/bevy_rapier3d) 和 [`avian`](https://crates.io/crates/avian3d)（Bevy 两个最流行的生态物理 crate）将添加自己的加速碰撞器拾取后端，以配合新上游化的 API。除非你正在调试、构建编辑器或确实关心原始网格的确切三角形，否则应使用这些 crate 之一进行高效的网格拾取。

### 使用方法

有两种很好的方式可以开始使用 API：

首先，你可能希望根据正在执行的操作快速更新对象（无论是 UI 还是游戏对象）的状态，通常是高亮它们或更改颜色。为此，只需查询 [`PickingInteraction`](https://docs.rs/bevy/0.15.0/bevy/picking/focus/enum.PickingInteraction.html) 组件的变化，它会根据当前的拾取状态变化。

其次，你可能希望动态响应各种指针驱动的事件。为此，我们建议使用观察者。这里，我们生成一个简单的文本节点并响应指针事件：

```rust
// 点击时打印消息的 UI 文本：
commands
    .spawn(Text::new("Click Me!"))
    .observe(on_click_print_hello);

// 拖拽时旋转的立方体：
commands
    .spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ))
    .observe(on_drag_spin);
}

fn on_click_print_hello(click: Trigger<Pointer<Click>>) {
    println!("{} 被点击了！", click.entity());
}

fn on_drag_spin(drag: Trigger<Pointer<Drag>>, mut transforms: Query<&mut Transform>) {
    let mut transform = transforms.get_mut(drag.entity()).unwrap();
    transform.rotate_y(drag.delta.x * 0.02);
}
```

如果你想覆盖实体与拾取的交互方式，请添加 [`PickingBehavior`](https://docs.rs/bevy/0.15.0/bevy/picking/struct.PickingBehavior.html) 组件并进行配置。

## Bubbling Observers（冒泡观察者）[#](https://bevy.org/news/bevy-0-15/#bubbling-observers)

作者：[@NthTensor](https://github.com/NthTensor)

PR：[#13991](https://github.com/bevyengine/bevy/pull/13991), [#15385](https://github.com/bevyengine/bevy/pull/15385)

实际上，几乎每个指针交互（如鼠标点击）都是罕见的（人类很慢！），并且通常需要复杂的响应。

这种模式在 UI 中特别有用，未处理的交互通常适用于*包含*顶层实体的窗格，但在游戏内交互中也有价值：点击单位上的剑应该选中整个单位！

为了支持这一点，我们扩展了 [`Event`](https://docs.rs/bevy/0.15.0/bevy/ecs/event/trait.Event.html) trait，包含一个关联的 `Traversal` 类型和一个关联的 `AUTO_PROPAGATE` 常量。此行为是可选的：当你派生 `Event` 类型时，它们分别默认设为 `()` 和 `false`。

对于 [`Pointer<E>`](https://docs.rs/bevy/0.15.0/bevy/picking/events/struct.Pointer.html) 事件类型，我们选择这样实现：

```rust
impl <E> Event for Pointer<E>{
    type Traversal = &Parent;
    const AUTO_PROPAGATE: bool = true;
}
```

这意味着，除非你调用 [`Trigger::propagate(false)`](https://docs.rs/bevy/0.15.0/bevy/ecs/prelude/struct.Trigger.html#method.propagate)，指针事件将沿层次结构向上冒泡（访问 [`Parent`](https://docs.rs/bevy/0.15.0/bevy/hierarchy/struct.Parent.html) 组件中存储的 `Entity`），直到到达实体根节点。

任何实现 [`Traversal`](https://docs.rs/bevy/0.15.0/bevy/ecs/traversal/trait.Traversal.html) trait 的类型都可以用作关联类型，并且可以从世界中访问任意的只读查询数据。虽然对于*许多*应用程序来说，使用标准实体层次结构是明智的选择，但冒泡可以用于使用你自己的[原始关系](https://github.com/bevyengine/bevy/issues/3742)进行任意事件传播。让我们知道你做了什么：用户反馈对于构建更好的 Bevy 不可或缺！

## Virtual Geometry Improvements（虚拟几何改进）[#](https://bevy.org/news/bevy-0-15/#virtual-geometry-improvements)

作者：[@JMS55](https://github.com/JMS55)

PR：[#14193](https://github.com/bevyengine/bevy/pull/14193), [#14623](https://github.com/bevyengine/bevy/pull/14623), [#15023](https://github.com/bevyengine/bevy/pull/15023), [#15084](https://github.com/bevyengine/bevy/pull/15084), [#15643](https://github.com/bevyengine/bevy/pull/15643), [#15846](https://github.com/bevyengine/bevy/pull/15846), [#15886](https://github.com/bevyengine/bevy/pull/15886), [#15955](https://github.com/bevyengine/bevy/pull/15955), [#16049](https://github.com/bevyengine/bevy/pull/16049), [#16111](https://github.com/bevyengine/bevy/pull/16111)

虚拟几何体（`meshlet` 功能）在 Bevy 0.15 中获得了大量改进。它仍未达到生产就绪状态，将保持为实验性模块，但自上次发布以来性能已大幅提升。

有关所有有趣的细节，请阅读[作者的博客文章](https://jms55.github.io/posts/2024-11-14-virtual-geometry-bevy-0-15)。

对于该功能的现有用户：

- 你的 GPU 现在必须支持 `WgpuFeatures::SHADER_INT64_ATOMIC_MIN_MAX` 才能使用此功能。正如上次发布中预先警告的那样，较旧的 GPU 可能不再兼容。
- 你必须重新生成 MeshletMesh 资源。在 Bevy 0.14 中生成的 MeshletMesh 资源与 Bevy 0.15 不兼容。
- 确保阅读迁移指南和更新的 rustdoc 以获取有关如何升级项目的完整详细信息。

## Animation Masks（动画遮罩）[#](https://bevy.org/news/bevy-0-15/#animation-masks)

作者：[@pcwalton](https://github.com/pcwalton)

PR：[#15013](https://github.com/bevyengine/bevy/pull/15013)

动画现在支持遮罩掉动画目标（关节）。这是在动画混合图（`AnimationGraph`）级别实现的，可用于在同一模型的不同部分上播放不同的动画而互不干扰。例如，你可以在角色的上半身和下半身上播放不同的动画。

在这个视频中，狐狸的头部和腿部正在播放两个独立的动画，这要归功于动画遮罩：

## Generalized Animation（通用动画）[#](https://bevy.org/news/bevy-0-15/#generalized-animation)

作者：[@pcwalton](https://github.com/pcwalton), [@mweatherley](https://github.com/mweatherley)

PR：[#15282](https://github.com/bevyengine/bevy/pull/15282), [#15434](https://github.com/bevyengine/bevy/pull/15434)

[`AnimationClip`](https://docs.rs/bevy/0.15/bevy/animation/struct.AnimationClip.html) 现在可用于使用任意曲线对组件字段进行动画。

```rust
animation_clip.add_curve_to_target(
    animation_target_id,
    AnimatableCurve::new(
        animated_field!(TextFont::font_size),
        // 在动画持续时间内振荡字体大小。
        FunctionCurve::new(
            Interval::UNIT, 
            |t| 25.0 * f32::sin(TAU * t) + 50.0
        )
    )
);
```

这适用于任何命名字段，并使用新的 `Curve` API，支持任意曲线类型。动画化 `Transform` 字段可能是最常见的用例：

```rust
animation_clip.add_curve_to_target(
    animation_target_id,
    AnimatableCurve::new(
        animated_field!(Transform::translation),
        // 使用内置缓动曲线构造函数构造 `Curve<Vec3>`
        EasingCurve::new(
            vec3(-10., 2., 0.),
            vec3(6., 2., 0.),
            EaseFunction::CubicInOut,
        )
    )
);
```

Bevy 用于 GLTF 动画等场景的内部动画处理也使用了相同的 API！

如果你需要比"动画化特定组件字段"更复杂的逻辑，可以实现 [`AnimatableProperty`](https://docs.rs/bevy/0.15/bevy/animation/animation_curves/trait.AnimatableProperty.html)，它可以在 [`AnimatableCurve`](https://docs.rs/bevy/0.15/bevy/animation/animation_curves/struct.AnimatableCurve.html) 中替代 [`animated_field!`](https://docs.rs/bevy/0.15/bevy/animation/macro.animated_field.html) 使用。

## Animation Graph: Additive Blending（动画图：叠加混合）[#](https://bevy.org/news/bevy-0-15/#animation-graph-additive-blending)

作者：[@pcwalton](https://github.com/pcwalton)

PR：[#15631](https://github.com/bevyengine/bevy/pull/15631)

Bevy 的动画图（`AnimationGraph`），用于组合同时播放的动画，现在支持*叠加混合*。

叠加混合是一种技术，允许将独立制作的动画应用到任意基础动画之上。例如，角色挥动武器的动画可以叠加在行走或奔跑动画之上。

在动画图本身中，这是通过使用 `Add` 节点实现的。上述情况可以用一个类似下面的动画图来描述（省略了权重）：

```
┌─────┐                              
│Walk ┼─┐                            
└─────┘ │ ┌─────┐                    
        ┼─┼Blend┼─┐                  
┌─────┐ │ └─────┘ │ ┌─────┐   ┌─────┐
│Run  ┼─┘         ┼─┤Add  ┼───┼Root │
└─────┘   ┌─────┐ │ └─────┘   └─────┘
          │Swing┼─┘                  
          └─────┘                    
```

`Add` 节点的工作原理是将第一个输入（这里是 'Walk' 和 'Run' 片段的混合）保持原样，然后将后续输入叠加在上面应用。在代码中，图的构造如下：

```rust
let mut animation_graph = AnimationGraph::new();

// 将 `Add` 节点附加到根节点。
let add_node = animation_graph.add_additive_blend(1.0, animation_graph.root);

// 添加 `Blend` 节点和叠加片段作为子节点；`Blend` 结果
// 将用作基础，因为它被首先列出。
let blend_node = animation_graph.add_blend(1.0, add_node);
animation_graph.add_clip(swing_clip_handle, 1.0, add_node);

// 最后，混合 'Walk' 和 'Run' 片段作为基础。
animation_graph.add_clip(walk_clip_handle, 0.5, blend_node);
animation_graph.add_clip(run_clip_handle, 0.5, blend_node);
```

## Animation Events（动画事件）[#](https://bevy.org/news/bevy-0-15/#animation-events)

作者：[@atornity](https://github.com/atornity), [@cart](https://github.com/cart)

PR：[#15538](https://github.com/bevyengine/bevy/pull/15538)

在动画的特定时间点触发游戏事件是一种常见模式，用于同步游戏的视觉、听觉和机制部分。在 **Bevy 0.15** 中，我们为 [`AnimationClip`](https://docs.rs/bevy/0.15/bevy/animation/struct.AnimationClip.html) 添加了"动画事件"支持，这意味着你可以在 [`AnimationClip`](https://docs.rs/bevy/0.15/bevy/animation/struct.AnimationClip.html) 播放过程中的某个时间点触发特定的 [`Event`](https://docs.rs/bevy/0.15/bevy/ecs/event/trait.Event.html)：

```rust
#[derive(Event, Clone)]
struct PlaySound {
    sound: Handle<AudioSource>,
}

// 这将在 `animation_clip` 的 1.5 秒标记处触发 PlaySound 事件
animation_clip.add_event(1.5, PlaySound {
    sound: assets.load("sound.mp3"),
});

app.add_observer(|trigger: Trigger<PlaySound>, mut commands: Commands| {
    let sound = trigger.event().sound.clone();
    commands.spawn(AudioPlayer::new(sound));
});
```

你也可以为特定的动画目标（如骨骼）触发事件：

```rust
animation_clip.add_event_to_target(AnimationTargetId::from_iter(["LeftLeg", "LeftFoot"], 0.5, TouchingGround);
```

这实现了诸如"每当脚步在动画中接触地面时触发尘土效果"的功能：

## Bevy Remote Protocol (BRP)（Bevy 远程协议）[#](https://bevy.org/news/bevy-0-15/#bevy-remote-protocol-brp)

作者：[@mweatherley](https://github.com/mweatherley)

PR：[#14880](https://github.com/bevyengine/bevy/pull/14880)

Bevy 远程协议允许远程与正在运行的 Bevy 应用程序的 ECS 进行交互。例如，这可用于在运行时检查和编辑实体及其组件。我们预计这将用于创建诸如检查器之类的工具，从单独的进程监视 ECS 的内容。我们计划在即将推出的 Bevy Editor 中使用 BRP 与远程 Bevy 应用程序通信。

目前，你可以使用 BRP 执行以下操作：

- 获取实体上一组组件的序列化值
- 执行查询以查找匹配一组组件的所有实体并检索匹配值
- 使用给定的一组组件值创建新实体
- 对于给定实体，插入或移除一组组件
- 反生成一个实体
- 重新父级化一个或多个实体
- 列出 ECS 中注册或在实体上存在的组件

以下是通过 HTTP 使用 BRP 所需的最小应用设置：

```rust
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // "核心"插件，处理由传输层提供的远程请求
            RemotePlugin::default(),
            // 通过 HTTP 提供远程请求传输
            RemoteHttpPlugin::default(),
        ))
        .run();
}
```

以下是一个示例请求：

```json
{
    "method": "bevy/get",
    "id": 0,
    "params": {
        "entity": 4294967298,
        "components": [
            "bevy_transform::components::transform::Transform"
        ]
    }
}
```

以及一个示例响应：

```json
{
    "jsonrpc": "2.0",
    "id": 0,
    "result": {
        "bevy_transform::components::transform::Transform": {
            "rotation": { "x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0 },
            "scale": { "x": 1.0, "y": 1.0, "z": 1.0 },
            "translation": { "x": 0.0, "y": 0.5, "z": 0.0 }
        }
    }
}
```

## Gamepads as Entities（手柄作为实体）[#](https://bevy.org/news/bevy-0-15/#gamepads-as-entities)

作者：[@s-puig](https://github.com/s-puig), [@Shatur](https://github.com/Shatur)

PR：[#12770](https://github.com/bevyengine/bevy/pull/12770), [#16222](https://github.com/bevyengine/bevy/pull/16222), [#16233](https://github.com/bevyengine/bevy/pull/16233)

手柄现在表示为实体，这使得它们更容易使用！[`Gamepad`](https://docs.rs/bevy/0.15/bevy/input/gamepad/struct.Gamepad.html) 组件提供按钮和轴的状态，以及诸如供应商和产品 ID 之类的元数据。[`GamepadSettings`](https://docs.rs/bevy/0.15/bevy/input/gamepad/struct.GamepadSettings.html) 组件为给定的 [`Gamepad`](https://docs.rs/bevy/0.15/bevy/input/gamepad/struct.Gamepad.html) 提供可配置的设置，如死区和灵敏度。手柄的名称现在存储在 Bevy 的标准 [`Name`](https://docs.rs/bevy/0.15/bevy/core/struct.Name.html) 组件中。

在 Bevy 0.14 中，你需要这样写：

```rust
fn gamepad_system(
   gamepads: Res<Gamepads>,
   button_inputs: Res<ButtonInput<GamepadButton>>,
   button_axes: Res<Axis<GamepadButton>>,
   axes: Res<Axis<GamepadAxis>>,
) {
    for gamepad in &gamepads {
        if button_inputs.just_pressed(
            GamepadButton::new(gamepad, GamepadButtonType::South)
        ) {
            info!("刚刚按下了 South");
        }

        let right_trigger = button_axes
           .get(GamepadButton::new(
               gamepad,
               GamepadButtonType::RightTrigger2,
           ))
           .unwrap();
        if right_trigger.abs() > 0.01 {
            info!("RightTrigger2 值为 {}", right_trigger);
        }

        let left_stick_x = axes
           .get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX))
           .unwrap();
        if left_stick_x.abs() > 0.01 {
            info!("LeftStickX 值为 {}", left_stick_x);
        }
    }
}
```

在 0.15 中，我们可以这样更简单地写：

```rust
fn gamepad_system(gamepads: Query<&Gamepad>) {
    for gamepad in &gamepads {
        if gamepad.just_pressed(GamepadButton::South) {
            println!("刚刚按下了 South");
        }

        let right_trigger = gamepad.get(GamepadButton::RightTrigger2).unwrap();
        if right_trigger.abs() > 0.01 {
            info!("RightTrigger2 值为 {}", right_trigger);
        }

        let left_stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap();
        if left_stick_x.abs() > 0.01 {
            info!("LeftStickX 值为 {}", left_stick_x);
        }
    }
}
```

好多了！

## Box Shadows（盒阴影）[#](https://bevy.org/news/bevy-0-15/#box-shadows)

作者：[@ickshonpe](https://github.com/ickshonpe)

PR：[#15204](https://github.com/bevyengine/bevy/pull/15204)

![Bevy 盒阴影的演示。浅蓝色背景上有 12 个形状，它们的边框半径、宽高比和阴影柔和度各不相同。阴影使按钮看起来悬浮在页面上方。](https://bevy.org/news/bevy-0-15/box_shadow.png)

Bevy UI 现在支持盒阴影！盒阴影可用于实现特定的艺术效果，例如创建深度感来提示用户某个元素是可交互的。

通过将新的 [`BoxShadow`](https://docs.rs/bevy/0.15/bevy/prelude/struct.BoxShadow.html) 组件添加到任何 [`Node`](https://docs.rs/bevy/0.15/bevy/prelude/struct.Node.html) 实体，你可以在控件后面绘制漂亮的阴影。

```rust
#[derive(Component)]
pub struct BoxShadow {
    /// 阴影的颜色
    pub color: Color,
    /// 水平偏移
    pub x_offset: Val,
    /// 垂直偏移
    pub y_offset: Val,
    /// 阴影向外扩散的程度。
    ///
    /// 负值会使阴影向内收缩。
    /// 百分比值基于 UI 节点的宽度。
    pub spread_radius: Val,
    /// 阴影的模糊程度
    pub blur_radius: Val,
}
```

我们计划在未来进行改进：支持使用阴影创建内凹/下沉效果，并为图像和文本添加阴影支持。如果你对这些可能性感兴趣，欢迎参与贡献！我们乐于帮助[新贡献者](https://bevy.org/learn/contribute/introduction/)实现他们关心的功能。

## Cosmic Text（Cosmic Text 文本渲染）[#](https://bevy.org/news/bevy-0-15/#cosmic-text)

作者：[@tigregalis](https://github.com/tigregalis), [@TotalKrill](https://github.com/TotalKrill)

PR：[#10193](https://github.com/bevyengine/bevy/pull/10193)

历史上，Bevy 使用 `ab_glyph` 库来渲染文本。它处理简单的拉丁文本渲染还算不错。但 Bevy 旨在成为一个通用的、可用于任何语言的应用程序框架，而 `ab_glyph` 在许多方面无法满足我们的需求。

自从我们选择 `ab_glyph` 以来，Rust 文本处理领域已经发生了显著的发展。幸运的是，现在有很多不错的选择。我们选择了 [`cosmic-text`](https://github.com/pop-os/cosmic-text)，因为它功能强大且已在生产应用中使用（Iced、Cosmic Desktop、Zed、Lapce 等）。值得注意的是，`cosmic-text` 为我们提供了以下支持：

- **字体塑形**：能够获取字符串字符码并执行布局和转换规则。这可能涉及移动、修改和组合字符（如连字）。这对于非拉丁语言*极为*重要。
- **系统字体加载**：能够扫描系统上安装的可用字体并加载它们。
- **双向文本**：并非所有语言都是从左到右的！Cosmic Text 为我们提供了双向文本支持。
- **文本编辑**：Cosmic Text 有自己的内部文本编辑模型，我们可以利用它。

在 **Bevy 0.15** 中，我们将文本渲染移植到了 `cosmic-text`。这主要是内部更改（与本版本中的其他"高级"文本 API 更改不同，例如 Required Components 移植）。

也就是说，你肯定会注意到我们渲染文本能力的提升！以下是 Bevy 使用 Noto Sans Arabic 字体从右到左渲染阿拉伯语文本的效果：

![阿拉伯语文本](https://bevy.org/news/bevy-0-15/arabic_text.png)

请注意，我们尚未接入 `cosmic-text` 的"系统字体加载"功能，但我们正在努力中！

## UI Scrolling（UI 滚动）[#](https://bevy.org/news/bevy-0-15/#ui-scrolling)

作者：[@Piefayth](https://github.com/Piefayth), [@nicoburns](https://github.com/nicoburns)

PR：[#15291](https://github.com/bevyengine/bevy/pull/15291)

Bevy 0.15 引入了 UI 容器的滚动支持。

一个 `overflow` 属性设为 `Overflow::scroll()` 的 UI `Node` 会将其内容偏移节点上 `ScrollPosition` 组件的正 `offset_x` 和 `offset_y` 值。

滚动是通过直接修改 `ScrollPosition` 实现的；目前没有内置的滚动输入处理器。一个新的 [`scroll`](https://github.com/bevyengine/bevy/tree/v0.15.0/examples/ui/scroll.rs) 示例演示了处理简单鼠标滚轮滚动的方式。没有 `OverflowAxis::Scroll` 的节点轴将忽略对 `ScrollPosition` 的更改。

## Retained Rendering World（持久化渲染世界）[#](https://bevy.org/news/bevy-0-15/#retained-rendering-world)

作者：[@re0312](https://github.com/re0312), [@trashtalk217](https://github.com/trashtalk217), [@kristoff3r](https://github.com/kristoff3r), [@tychedelia](https://github.com/tychedelia)

PR：[#14449](https://github.com/bevyengine/bevy/pull/14449), [#15320](https://github.com/bevyengine/bevy/pull/15320), [#15582](https://github.com/bevyengine/bevy/pull/15582), [#15756](https://github.com/bevyengine/bevy/pull/15756)

一段时间以来，Bevy 一直拥有一个"[并行流水线渲染器](https://bevy.org/news/bevy-0-6/#pipelined-rendering-extract-prepare-queue-render)"。为了实现这一点，除了主世界（Main World）之外，我们还添加了一个渲染世界（Render World）（`World` 保存 ECS 数据，如实体、组件和资源）。主世界是应用逻辑的真实来源。当渲染世界渲染当前帧时，主世界可以模拟下一帧。有一个简短的"提取步骤"，我们在此同步两者并将相关数据从主世界复制到渲染世界。

在之前的 Bevy 版本中，我们采用了"即时模式"方法进行主世界 -> 渲染世界的同步：每帧完全清除渲染世界的实体。这实现了几个目标：

1. 它确保实体 ID"对齐"，允许我们在两个地方复用实体。
2. 它避免了需要解决"不同步问题"。通过每帧清除并重新同步，我们确保两个世界始终完美同步。

"即时模式"流水线渲染方法也有先例：Bungie 的《命运》渲染器就使用了它并取得了很好的效果！

然而，我们很快发现每帧清除有重大缺点：

1. 清除过程本身有开销。
2. "表"ECS 存储相对于替代方案，每帧重建成本可能很高，因为需要"原型移动"。因此，我们采用了许多变通方法，例如将存储移到 ECS 之外。
3. 每帧完全重新同步意味着做了不需要重做的工作。ECS 为我们提供了数据如何变化的全局视图。我们应该利用这一点！

在 **Bevy 0.15** 中，我们切换到了"持久化渲染世界"。我们不再每帧清除。我们不再依赖共享的实体 ID 空间。相反：

1. 每个世界有自己的实体。
2. 对于相关的实体，我们将这种关系存储为组件（例如：渲染世界实体有一个 `MainEntity` 组件，主世界实体有一个 `RenderEntity` 组件）。如果带有 `SyncToRenderWorld` 的主世界实体被生成，我们会在渲染世界中生成一个等效实体。如果主世界实体被反生成，我们会反生成渲染世界中的相关实体。

确保同步完美*不*是一个容易的问题。在本周期内填补所有漏洞花费了大量时间，我们可能会在未来继续发展我们的同步策略。但我们认为"持久化"从根本上对 Bevy 更有利，并且我们很高兴打下这个基础！

## Curves（曲线）[#](https://bevy.org/news/bevy-0-15/#curves)

作者：[@mweatherley](https://github.com/mweatherley)

PR：[#14630](https://github.com/bevyengine/bevy/pull/14630), [#14976](https://github.com/bevyengine/bevy/pull/14976), [#15675](https://github.com/bevyengine/bevy/pull/15675), [#16637](https://github.com/bevyengine/bevy/pull/16637)

新的 [`Curve<T>`](https://docs.rs/bevy/0.15.0/bevy/math/trait.Curve.html) trait 为曲线提供了一个共享接口，描述了当我们在某个域上变化 `f32` 参数 `t` 时，`T` 类型的值如何变化。

什么在变化，以及它在*什么域上*变化都是非常灵活的。你可以将泛型参数 `T` 设为从位置、到伤害、再到颜色的任何内容（就像我们为[颜色渐变](https://docs.rs/bevy/0.15.0/bevy/color/struct.ColorCurve.html)创建强大抽象那样）。

进度参数 `t` 通常代表时间（如动画中），但它也可以表示诸如起始值和结束值之间的进度分数/百分比或距离（例如映射到 2D 或 3D 空间的曲线）。

### 构造曲线

每条曲线可以以多种方式定义。例如，曲线可以是：

- 由函数定义
- 从样本插值
- 使用样条构造
- 由缓动函数生成

查看 [`Curve<T>`](https://docs.rs/bevy/0.15.0/bevy/math/trait.Curve.html) trait 上的构造函数以获取更多细节。

### 修改曲线

程序化修改曲线是创建具有所需行为的曲线以及动态改变曲线的强大工具。

Bevy 0.15 提供了许多灵活的自适应器，用于获取现有曲线并修改其输出和/或参数化。

例如：

```rust
let timed_angles = [
  (0.0, 0.0),
  (1.0, -FRAC_PI_2),
  (2.0, 0.0),
  (3.0, FRAC_PI_2),
  (4.0, 0.0)
];

// 一个插值我们的 (time, angle) 对列表的曲线。在每个时间点，
// 它产生角度，所以它是一个在 `[0, 4]` 上参数化的 `Curve<f32>`。
let angle_curve = UnevenSampleAutoCurve::new(timed_angles).unwrap();

// 将这些角度解释为 `Curve<Rot2>` 的旋转角度。
let rotation_curve = angle_curve.map(Rot2::radians);

// 更改参数化区间，使整个循环在
// 仅 1 秒而不是 4 秒内完成。
let fast_rotation_curve = rotation_curve.reparametrize_linear(Interval::UNIT).unwrap();
```

还有许多其他自适应器可用。例如：

- 曲线可以反转、重复或乒乓
- 两条曲线可以链接在一起形成更长的曲线
- 两条曲线可以压缩在一起形成元组值的曲线

### 从曲线采样

采样是询问"这条曲线在 `t` 的某个特定值处的值是多少"的过程。只需调用 [`Curve::sample`](https://docs.rs/bevy/0.15.0/bevy/math/trait.Curve.html#method.sample) 即可！

就像矢量图形可以光栅化为像素一样，曲线可以光栅化为规则的、离散的区间。通过重采样为基于原始曲线样本插值的近似值，我们可以使不同来源的曲线在数据层面变得统一。

虽然这看起来有点奇特，但这种技术对于序列化曲线或通过数值方法近似属性至关重要。

```rust
// 一个由函数定义的曲线，可能难以作为数据存储。
let exponential_curve = FunctionCurve::new(
  interval(0.0, 10.0).unwrap(), 
  |t| f32::exp(2.0 * t)
);

// 通过在 100 个段上重采样来近似原始曲线的曲线。
// 内部只保存样本和参数区间。
let raster_curve = exponential_curve.resample_auto(100).unwrap();
```

## Common Easing Functions（常用缓动函数）[#](https://bevy.org/news/bevy-0-15/#common-easing-functions)

作者：[@RobWalt](https://github.com/RobWalt), [@mockersf](https://github.com/mockersf), [@mweatherley](https://github.com/mweatherley)

PR：[#14788](https://github.com/bevyengine/bevy/pull/14788), [#15675](https://github.com/bevyengine/bevy/pull/15675), [#15711](https://github.com/bevyengine/bevy/pull/15711)

"缓动函数"可用于轻松构造在两个值之间插值的曲线。有许多"常见"的缓动函数，每个都有不同的"特征"。它们通常用于"补间"场景中，为插值赋予生命力。

**Bevy 0.15** 添加了一个新的 `Ease` trait，它定义了如何插值给定类型的值。`Ease` 类型包括：

- 向量类型（`f32`、`Vec2`、`Vec3`……）；
- 方向类型（`Dir2`、`Dir3`、`Dir3A`）；
- 旋转类型（`Rot2`、`Quat`）。

我们还添加了一个 `EaseFunction` 枚举，它定义了多种常用缓动函数。新的 `EasingCurve` 类型使用这些作为输入，从给定的缓动参数定义最终的 `Curve`。

例如，我们可以使用缓动函数在两个旋转之间插值：

```rust
// 在无旋转和绕 Y 轴旋转 PI/2 角度之间缓动。
let rotation_curve = EasingCurve::new(
    Quat::IDENTITY,
    Quat::from_rotation_y(FRAC_PI_2),
    EaseFunction::ElasticInOut,
)
.reparametrize_linear(interval(0.0, 4.0).unwrap())
.unwrap();
```

## Cyclic Splines（循环样条）[#](https://bevy.org/news/bevy-0-15/#cyclic-splines)

作者：[@mweatherley](https://github.com/mweatherley)

PR：[#14106](https://github.com/bevyengine/bevy/pull/14106)

大多数三次样条构造现在支持创建闭合循环而不仅仅是路径，如果需要的话。这对于构建诸如 NPC 或其他游戏实体的周期性路径非常方便。

唯一的区别是需要调用 `to_curve_cyclic` 而不是 `to_curve`。支持的样条构造包括：

- Hermite 样条（`CubicHermite`）
- Cardinal 样条（`CubicCardinalSpline`）
- B 样条（`CubicBSpline`）
- 线性样条（`LinearSpline`）

![使用三次 Cardinal 样条构造的闭合循环](https://bevy.org/news/bevy-0-15/cyclic-spline.png)

## `PartialReflect`（部分反射）[#](https://bevy.org/news/bevy-0-15/#partialreflect)

作者：[@soqb](https://github.com/soqb), [@nicopap](https://github.com/nicopap)

PR：[#7207](https://github.com/bevyengine/bevy/pull/7207)

Bevy 拥有强大的[反射](https://docs.rs/bevy_reflect/0.15/bevy_reflect/)系统，允许你在运行时内省和构建类型。它通过将数据作为 [`Reflect`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/trait.Reflect.html) trait 对象（如 `Box<dyn Reflect>`）传递来实现这一点。这具有擦除编译时类型信息的效果，允许在不知道 trait 对象背后确切类型的情况下存储和移动数据。

由于这种类型擦除，`bevy_reflect` 还可以实现一些有趣的技巧。例如，在很多情况下，类型需要逐个字段地构建，例如在反序列化期间。当你在编译时知道类型时，这没问题，但在运行时变得非常具有挑战性。为了解决这个问题，`bevy_reflect` 引入了*动态*类型的概念。

动态类型作为一种方式存在，用于以类似于具体类型的方式动态构造和存储反射数据。在幕后，`bevy_reflect` 使用这些类型来构建目标类型的表示。它可以做到这一点，因为我们隐藏了 `dyn Reflect` trait 对象背后的实际类型。

不幸的是，这带来一个非常常见的问题：很容易意外地认为一个 `dyn Reflect` 是具体类型，而实际上它只是代表该具体类型的动态类型。

为了解决这个问题，Bevy 0.15 基于[唯一反射 RFC](https://github.com/bevyengine/rfcs/pull/56) 重新设计了 `Reflect` trait。它将拆分为两个独立的 trait：`Reflect` 和 [`PartialReflect`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/trait.PartialReflect.html)。

`PartialReflect` 很像之前版本的 `Reflect` trait。它允许访问基本的反射能力，并允许在 `dyn PartialReflect` trait 对象后面进行类型擦除。它允许具体类型和动态类型互换使用。

另一方面，`Reflect` 成为了一个严格得多的 trait。它是 `PartialReflect` 的子集，保证 trait 对象底层的类型正是它所说的具体类型。

这种拆分允许基于反射的 API 和用户代码更明确地说明它们正在处理的 trait 对象的动态性。它将类型是否是动态的知识移至编译时，防止了使用动态类型时的许多常见陷阱，包括知道何时需要单独处理它们。

## Reflect Remote Types（反射远程类型）[#](https://bevy.org/news/bevy-0-15/#reflect-remote-types)

作者：[@MrGVSV](https://github.com/MrGVSV)

PR：[#6042](https://github.com/bevyengine/bevy/pull/6042)

[`bevy_reflect`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/) crate 依赖于类型实现 [`Reflect`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/trait.Reflect.html) 来使它们可反射。未实现 `Reflect` 的结构体和枚举的字段必须使用 `#[reflect(ignore)]` 专门忽略。由于 Rust 的[孤儿规则](https://doc.rust-lang.org/book/ch10-02-traits.html#implementing-a-trait-on-a-type)，对于当前 crate 不拥有的类型通常就是这种情况。

遵循 [`serde` 的示例](https://serde.rs/remote-derive.html)，Bevy 0.15 引入了一种使用新的 [`#[reflect_remote(...)]`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/attr.reflect_remote.html) 属性宏来反射远程类型的方法。这允许用户定义一个模型供反射基于其行为，同时仍然操作实际类型。

```rust
// 假设此类型定义在名为 `external_crate` 的 crate 中
#[derive(Default)]
struct Name {
    pub value: String,
}

// 我们可以定义我们的模型，包括其他派生宏和反射属性
#[reflect_remote(external_crate::Name)]
#[derive(Default)]
#[reflect(Default)]
struct NameWrapper {
    pub value: String,
}

// 现在我们可以将 `Name` 用作反射类型中的字段，而不必忽略它
#[derive(Reflect)]
struct Player {
    #[reflect(remote = NameWrapper)]
    name: external_crate::Name,
}
```

在底层，这是通过将我们的模型转换为实际类型的透明包装器来实现的：

```rust
#[repr(transparent)]
struct NameWrapper(pub external_crate::Name);
```

然后宏使用模型生成所有反射 trait 的实现，由一个用于在包装器和远程类型之间切换的新 [`ReflectRemote`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/trait.ReflectRemote.html) trait 驱动。还会生成编译时断言，以帮助确保模型和实际类型保持同步。

虽然此功能的许多方面已经完成，包括泛型支持、枚举支持和嵌套，但仍然存在一些限制，我们希望在未来版本中解决，包括支持反射具有私有字段的远程类型。

## The `Reflectable` Trait（`Reflectable` Trait）[#](https://bevy.org/news/bevy-0-15/#the-reflectable-trait)

作者：[@MrGVSV](https://github.com/MrGVSV)

PR：[#5772](https://github.com/bevyengine/bevy/pull/5772)

[`bevy_reflect`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/) 由许多不同的 trait 协同工作提供完整的反射 API。这些包括像 [`Reflect`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/trait.Reflect.html) 这样的 trait，也包括其他像 [`TypePath`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/trait.TypePath.html)、[`Typed`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/trait.Typed.html) 和 [`GetTypeRegistration`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/trait.GetTypeRegistration.html) 这样的 trait。

这使得在泛型参数上添加正确的约束有点令人困惑，并且很容易忘记包含其中一个 trait。

为了简化这一点，0.15 引入了 [`Reflectable`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/trait.Reflectable.html) trait。上述所有 trait 都是 `Reflectable` 的超 trait，允许在必要时替代所有它们使用。

## Function Reflection（函数反射）[#](https://bevy.org/news/bevy-0-15/#function-reflection)

作者：[@MrGVSV](https://github.com/MrGVSV), [@nixpulvis](https://github.com/nixpulvis), [@hooded-shrimp](https://github.com/hooded-shrimp)

PR：[#13152](https://github.com/bevyengine/bevy/pull/13152), [#14098](https://github.com/bevyengine/bevy/pull/14098), [#14141](https://github.com/bevyengine/bevy/pull/14141), [#14174](https://github.com/bevyengine/bevy/pull/14174), [#14201](https://github.com/bevyengine/bevy/pull/14201), [#14641](https://github.com/bevyengine/bevy/pull/14641), [#14647](https://github.com/bevyengine/bevy/pull/14647), [#14666](https://github.com/bevyengine/bevy/pull/14666), [#14704](https://github.com/bevyengine/bevy/pull/14704), [#14813](https://github.com/bevyengine/bevy/pull/14813), [#15086](https://github.com/bevyengine/bevy/pull/15086), [#15145](https://github.com/bevyengine/bevy/pull/15145), [#15147](https://github.com/bevyengine/bevy/pull/15147), [#15148](https://github.com/bevyengine/bevy/pull/15148), [#15205](https://github.com/bevyengine/bevy/pull/15205), [#15484](https://github.com/bevyengine/bevy/pull/15484)

Rust 在动态上下文中使用函数的选项是有限的。我们被迫要么将函数强制转换为函数指针（例如 `fn(i32, i32) -> i32`），要么将其转换为 trait 对象（例如 `Box<dyn Fn(i32, i32) -> i32>`）。

在这两种情况下，只有具有相同签名（输入和输出）的函数才能作为相同类型的对象存储。对于真正的动态上下文，例如使用脚本语言或按名称获取函数，这可能是一个显著的限制。

Bevy 的 [`bevy_reflect`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/) crate 已经通过反射消除了对编译时类型知识的需求。在 Bevy 0.15 中，函数也可以被反射！

此功能是 opt-in 的，需要在 `bevy` 上启用 `reflect_functions` 功能（或者如果直接使用该 crate，则在 `bevy_reflect` 上启用 `functions` 功能）。

它的工作原理是将参数和返回类型派生了 [`Reflect`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/trait.Reflect.html) 的常规函数转换为 [`DynamicFunction`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/func/struct.DynamicFunction.html) 类型，使用新的 [`IntoFunction`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/func/trait.IntoFunction.html) trait。

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

let function = add.into_function();
```

有了 `DynamicFunction`，我们可以生成参数列表到 [`ArgList`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/func/args/struct.ArgList.html) 中并调用函数：

```rust
let args = ArgList::new()
    .push_owned(25_i32)
    .push_owned(75_i32);

let result = function.call(args);
```

调用函数返回一个 [`FunctionResult`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/func/type.FunctionResult.html)，其中包含我们的 [`Return`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/func/enum.Return.html) 数据或出错时的 [`FunctionError`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/func/enum.FunctionError.html)。

```rust
match result {
    Ok(Return::Owned(value)) => {
        let value = value.try_take::<i32>().unwrap();
        println!("得到: {}", value);
    }
    Err(err) => println!("错误: {:?}", err),
    _ => unreachable!("我们的函数总是返回拥有的值"),
}
```

#### 闭包反射

此功能不仅适用于常规函数——它也适用于闭包！

对于不可变捕获环境闭包，我们可以继续使用 `DynamicFunction` 和 `IntoFunction`。对于可变捕获环境的闭包，有 [`DynamicFunctionMut`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/func/struct.DynamicFunctionMut.html) 和 [`IntoFunctionMut`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/func/trait.IntoFunctionMut.html)。

```rust
let mut total = 0;

let increment = || total += 1;

let mut function = increment.into_function_mut();

function.call(ArgList::new()).unwrap();
function.call(ArgList::new()).unwrap();
function.call(ArgList::new()).unwrap();

// 丢弃函数以释放 `total` 的可变借用。
// 或者，我们的最后一次调用也可以使用 `call_once`。
drop(function);

assert_eq!(total, 3);
```

#### `FunctionInfo`

反射函数通过 [`FunctionInfo`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/func/struct.FunctionInfo.html) 持有其类型元数据，它由 [`TypedFunction`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/func/trait.TypedFunction.html) trait 自动生成。这允许它们返回关于函数的信息，包括其名称、参数和返回类型。

```rust
let info = String::len.get_function_info();

assert_eq!(info.name().unwrap(), "alloc::string::String::len");
assert_eq!(info.arg_count(), 1);
assert!(info.args()[0].is::<&String>());
assert!(info.return_info().is::<usize>());
```

需要注意的是，闭包、匿名函数和函数指针不会自动获得名称。对于这些情况，可以手动提供名称。

所有参数（包括 `self` 参数）也是如此：名称不会自动生成，如果需要，必须手动提供。

使用 `FunctionInfo`，`DynamicFunction` 在调试打印时会打印出其签名。

```rust
dbg!(String::len.into_function());
// 输出：
// DynamicFunction(fn alloc::string::String::len(_: &alloc::string::String) -> usize)
```

#### 手动构造

对于 `IntoFunction` 无法工作的情况，例如参数过多的函数或具有更复杂生命周期的函数，`DynamicFunction` 也可以手动构造。

```rust
// 注意：此函数可以与 `IntoFunction` 一起使用，
// 但出于演示目的，我们手动构造它。
let add_to = DynamicFunction::new(
    |mut args| {
        let a = args.take::<i32>()?;
        let b = args.take_mut::<i32>()?;

        *b += a;

        Ok(Return::unit())
    },
    FunctionInfo::named("add_to")
        .with_arg::<i32>("a")
        .with_arg::<&mut i32>("b")
        .with_return::<()>(),
);
```

#### 函数注册表

为了更容易地使用反射函数，添加了一个专用的 [`FunctionRegistry`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/func/struct.FunctionRegistry.html)。它的工作方式类似于 [`TypeRegistry`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/struct.TypeRegistry.html)，函数可以按名称注册和检索。

```rust
let mut registry = FunctionRegistry::default();
registry
    // 命名函数可以直接注册
    .register(add)?
    // 未命名函数（例如闭包）必须使用名称注册
    .register_with_name("add_3", |a: i32, b: i32, c: i32| a + b + c)?;

let add = registry.get("my_crate::math::add").unwrap();
let add_3 = registry.get("add_3").unwrap();
```

为了更好地与 Bevy 的其他部分集成，添加了一个新的 [`AppFunctionRegistry`](https://docs.rs/bevy_reflect/0.15/bevy_ecs/reflect/struct.AppTypeRegistry.html) 资源以及 [`App`](https://docs.rs/bevy_reflect/0.15/bevy_app/struct.App.html) 上的注册方法。

#### `Function` Trait

一个新的反射 trait——恰当地命名为 [`Function`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/func/trait.Function.html)——已经被添加以对应函数。

由于 Rust 的限制，我们无法为所有函数实现此 trait，但它确实使得将 `DynamicFunction` 作为 [`PartialReflect`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/trait.PartialReflect.html) trait 对象传递成为可能。

```rust
#[derive(Reflect)]
#[reflect(from_reflect = false)]
struct EventHandler {
    callback: DynamicFunction<'static>,
}

let event_handler: Box<dyn Struct> = Box::new(EventHandler {
    callback: (|| println!("事件触发了！")).into_function(),
});

let field = event_handler.field("callback").unwrap();

if let ReflectRef::Function(callback) = field.reflect_ref() {
    callback.reflect_call(ArgList::new()).unwrap();
}
```

#### 限制

虽然此功能已经相当强大，但仍有一些限制。

首先，`IntoFunction`/`IntoFunctionMut` 仅适用于最多 16 个参数的函数，并且只支持返回与第一个参数（通常是方法中的 `self`）生命周期绑定的借用数据。

其次，由于函数反射 trait 的定义方式，`Function` trait 不能为所有函数实现。

第三，所有参数和返回类型必须派生了 `Reflect`。这对于某些类型（如 `&str`）可能令人困惑，因为只有 `&'static str` 实现了 `Reflect`，而其借用版本将是 `&&'static str`。

最后，虽然支持泛型函数，但它们必须首先手动单态化。这意味着如果你有一个泛型函数如 `fn foo<T>()`，你必须像 `foo::<i32>.into_function()` 这样创建 `DynamicFunction`。

大多数这些限制是由于 Rust 本身造成的。[缺乏可变参数](https://poignardazur.github.io/2024/05/25/report-on-rustnl-variadics/)和[一致性问题](https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#coherence-leak-check)是需要解决的两个最大困难。尽管如此，我们将在未来版本中寻找改善此功能的人体工程学和能力的方法。

我们已经有一个 [PR](https://github.com/bevyengine/bevy/pull/15074) 来添加对重载函数的支持：具有可变参数数量和参数类型的函数。

## `TypeInfo` Improvements（`TypeInfo` 改进）[#](https://bevy.org/news/bevy-0-15/#typeinfo-improvements)

作者：[@MrGVSV](https://github.com/MrGVSV)

PR：[#15475](https://github.com/bevyengine/bevy/pull/15475), [#13321](https://github.com/bevyengine/bevy/pull/13321), [#13320](https://github.com/bevyengine/bevy/pull/13320), [#15235](https://github.com/bevyengine/bevy/pull/15235)

使用 [`bevy_reflect`](https://docs.rs/bevy_reflect/0.15/)，可以从反射类型中获取编译时类型信息作为 [`TypeInfo`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/enum.TypeInfo.html)。

Bevy 0.15 为使用 `TypeInfo` 添加了许多改进和便捷方法。

#### 泛型参数信息

第一个新增功能是能够获取类型泛型参数的信息。这不仅包括参数的类型，还包括其名称，如果是 const 参数，还包括其默认值。

```rust
#[derive(Reflect)]
struct MyStruct<T>(T);

let generics = MyStruct::<f32>::type_info().generics();

let t = generics.get(0).unwrap();
assert_eq!(t.name(), "T");
assert!(t.ty().is::<f32>());
assert!(!t.is_const());
```

#### 嵌套 `TypeInfo`

Rust 中的几乎所有类型都由其他类型组成。结构体、映射、列表——它们都包含其他类型。

在之前的 Bevy 版本中，`TypeInfo` 只允许你有限地访问这些嵌套类型的类型信息。它主要只提供类型的 [`TypeId`](https://doc.rust-lang.org/std/any/struct.TypeId.html) 和 [`TypePath`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/trait.TypePath.html)。

然而，在 Bevy 0.15 中，你现在可以直接访问这些嵌套类型的 `TypeInfo`。

```rust
#[derive(Reflect)]
struct Row {
  id: usize
}

let struct_info: StructInfo = Row::type_info().as_struct();

let field: NamedField = struct_info.field("id").unwrap();

// `NamedField` 现在公开了一种获取字段类型 `TypeInfo` 的方法
let field_info: TypeInfo = field.type_info().unwrap();
assert!(field_info.is::<usize>());
```

#### `TypeInfo` 便捷转换

在大多数情况下，`TypeInfo` 需要首先进行模式匹配到正确的变体，才能完全访问类型的编译时信息。当你已经提前知道变体时，这可能有点烦人。这通常发生在编写测试时，但也出现在尝试获取类型的 [`ReflectRef`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/enum.ReflectRef.html) 数据以及其 `TypeInfo` 时。它通常看起来像这样：

```rust
// 我们必须对 `ReflectRef` 进行模式匹配...
let ReflectRef::List(list) = reflected_value.reflect_ref() else {
    panic!("期望一个列表");
};

// ...并且仍然需要对 `TypeInfo` 进行模式匹配
let TypeInfo::List(list_info) = reflected_value.get_represented_type_info().unwrap() else {
    panic!("期望一个列表信息");
};
```

在这种情况下，变体已经通过 `ReflectRef` 验证，但 `TypeInfo` 仍然必须进行模式匹配。

在 Bevy 0.15 中，为 `TypeInfo`、`ReflectRef`、[`ReflectMut`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/enum.ReflectMut.html) 和 [`ReflectOwned`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/enum.ReflectOwned.html) 添加了便捷方法，可以方便地转换为预期的变体，或在失败时返回错误。

```rust
// 我们可以简单地验证一次反射值的类型...
let ReflectRef::List(list) = reflected_value.reflect_ref() else {
    panic!("期望一个列表");
};

// ...然后只断言 `TypeInfo`
let list_info = reflected_value.get_represented_type_info().unwrap().as_list().unwrap();
```

如果上面代码片段中的 `.as_list()` 转换失败，它将返回一个错误，详细说明我们期望的[种类](https://docs.rs/bevy_reflect/0.15/bevy_reflect/enum.ReflectKind.html)（即 `List`）以及实际得到的内容（例如 `Array`、`Struct` 等）。

这在相反方向也有效：

```rust
let TypeInfo::List(list_info) = reflected_value.get_represented_type_info().unwrap() else {
    panic!("期望一个列表信息");
};

let list = reflected_value.reflect_ref().as_list().unwrap();
```

## The `Type` Type（`Type` 类型）[#](https://bevy.org/news/bevy-0-15/#the-type-type)

作者：[@MrGVSV](https://github.com/MrGVSV)

PR：[#14838](https://github.com/bevyengine/bevy/pull/14838)

Rust 的 [`TypeId`](https://doc.rust-lang.org/std/any/struct.TypeId.html) 是一个类型的唯一标识符，使其成为映射中的键以及运行时检查两个类型是否相同的完美候选。由于它本质上只是两个 `u64` 值，因此复制、比较和哈希都非常便宜。

但使用 `TypeId` 的一个缺点是它不包含关于类型的任何其他信息，包括其名称。这可能使调试有些令人沮丧，因为你无法轻易判断一个 `TypeId` 对应哪个类型。

由于 [`bevy_reflect`](https://docs.rs/bevy_reflect/0.15/) 大量使用 `TypeId`，0.15 引入了一个新类型来帮助缓解调试问题，同时仍然保持 `TypeId` 的优势：[`Type`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/struct.Type.html)。

[`Type`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/struct.Type.html) 是 `TypeId` 的一个简单包装器，同时存储了 [`TypePathTable`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/struct.TypePathTable.html)。像 `TypeId` 一样，它是 `Copy`、`Eq` 和 `Hash` 的，后两者委托给底层的 `TypeId`。但与 `TypeId` 不同的是，其 `Debug` 实现会打印它所代表的类型的[类型路径](https://docs.rs/bevy_reflect/0.15/bevy_reflect/trait.TypePath.html#tymethod.type_path)。这种可调试性是以额外 32 字节为代价的，但通常非常值得，特别是如果这些数据本来也会存储在其他地方的话。

它可以从任何实现了 [`TypePath`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/trait.TypePath.html) 的类型构造：

```rust
let ty = Type::of::<String>();

let mut map = HashMap::<Type, i32>::new();
map.insert(ty, 25);

let debug = format!("{:?}", map);
assert_eq!(debug, "{alloc::string::String: 25}");
```

## Reflection Support for Sets（反射对 Set 的支持）[#](https://bevy.org/news/bevy-0-15/#reflection-support-for-sets)

作者：[@RobWalt](https://github.com/RobWalt)

PR：[#13014](https://github.com/bevyengine/bevy/pull/13014)

在 `bevy_reflect` 内部，每个反射的 Rust 对象最终都会被映射到少数几个 [`ReflectKind`](https://docs.rs/bevy/0.15.0/bevy/reflect/enum.ReflectKind.html) 变体之一。

在 Bevy 0.15 之前，集合（如 [`HashSet`](https://doc.rust-lang.org/stable/std/collections/struct.HashSet.html)）被视为不透明的"值"：无法通过反射查看或修改其内容。通过这些更改，我们现在可以正确表示各种集合，这对于运行时调试工具（如 [`bevy-inspector-egui`](https://github.com/jakobhellermann/bevy-inspector-egui)）特别方便！

## Change Detection Source Location Tracking（变更检测源位置追踪）[#](https://bevy.org/news/bevy-0-15/#change-detection-source-location-tracking)

作者：[@aevyrie](https://github.com/aevyrie)

PR：[#14034](https://github.com/bevyengine/bevy/pull/14034)

在任何复杂程序中跟踪值和时间的变更位置可能很棘手，Bevy 应用程序也不例外。幸运的是，我们统一的基于 ECS 的数据模型使我们能够轻松添加开箱即用的调试工具，无需用户配置。

当你启用 `track_change_detection` 功能标志时，Bevy 将记录修改你的组件或资源的精确代码行，与值一起保存。虽然这对于普通使用显然过于昂贵，但对于调试棘手问题来说简直是天赐之物，因为值可以通过你选择的调试器直接记录或读取。

如 [`change_detection` 示例](https://github.com/bevyengine/bevy/blob/main/examples/ecs/change_detection.rs)所示，只需启用该功能并在任何 [`Ref`](https://docs.rs/bevy/0.15.0/bevy/ecs/change_detection/struct.Ref.html)、[`Mut`](https://docs.rs/bevy/0.15.0/bevy/ecs/change_detection/struct.Mut.html)、[`Res`](https://docs.rs/bevy/0.15.0/bevy/ecs/change_detection/struct.Res.html) 或 [`ResMut`](https://docs.rs/bevy/0.15.0/bevy/ecs/change_detection/struct.ResMut.html) 智能指针上调用 `my_component.changed_by()`，即可获得一个有用的字符串，直接指向最后修改数据的那行代码！

## Optimized Iteration of Mixed Sparse Set and Table Components（混合稀疏集和表组件的优化迭代）[#](https://bevy.org/news/bevy-0-15/#optimized-iteration-of-mixed-sparse-set-and-table-components)

作者：[@re0312](https://github.com/re0312)

PR：[#14049](https://github.com/bevyengine/bevy/pull/14049), [#14673](https://github.com/bevyengine/bevy/pull/14673)

在 Bevy 中，组件可以根据实现 [`Component`](https://docs.rs/bevy/0.15/bevy/ecs/component/trait.Component.html#associatedconstant.STORAGE_TYPE) trait 时设置的 [`StorageType`](https://docs.rs/bevy/0.15/bevy/ecs/component/enum.StorageType.html)，使用两种不同机制之一进行[存储](https://docs.rs/bevy/0.15/bevy/ecs/component/trait.Component.html#associatedconstant.STORAGE_TYPE)。

表存储是传统的原型 ECS 存储，其中组件数据与其他共享相同组件集的实体一起密集打包到原始数据表中。相比之下，稀疏集存储将组件信息保留在表外，按原型（它们拥有的组件集）分离实体，而不会分割原本共享的表。

由于稀疏集组件使用的类映射存储策略，它们具有更快的插入和移除速度，但代价是较慢的随机访问迭代。这是一个合理的权衡，但历史上 Bevy 开发者不太可能使用。

这是因为一个长期存在的 Bug 导致如果查询或其过滤器中的组件之一是稀疏集，迭代就会使用较慢的回退稀疏式迭代，无论是否有必要。修复后，对于这些场景，查询迭代速度（使用并行迭代时）提高了 1.8 到 3.5 倍！

稀疏集组件中数据的迭代仍然相对较慢，但它们终于应该成为任何需要重复插入或无数据组件的良好默认选择。

## Expose winit's `MonitorHandle`（暴露 winit 的 `MonitorHandle`）[#](https://bevy.org/news/bevy-0-15/#expose-winit-s-monitorhandle)

作者：[@tychedelia](https://github.com/tychedelia)

PR：[#13669](https://github.com/bevyengine/bevy/pull/13669)

新的 `Monitor` 组件简化了多显示器设置的工作流程，提供了对显示器属性（如分辨率、刷新率、位置和缩放因子）的便捷访问。此功能对于需要在特定显示器上生成窗口、收集显示器详细信息或根据可用硬件调整应用的开发者特别有用。这对于创意设置（如多投影仪安装或 LED 视频墙）特别有用，在这些场景中，精确控制显示环境至关重要。

可以查询 `Monitor` 并将其用于生成或调整窗口大小等操作：

```rust
fn spawn_windows(
    mut commands: Commands,
    monitors: Query<(Entity, &Monitor)>,
) {
    for (entity, monitor) in monitors_added.iter() {
        commands.spawn(Window {
            mode: WindowMode::Fullscreen(MonitorSelection::Entity(entity)),
            position: WindowPosition::Centered(MonitorSelection::Entity(entity)),
            ..default()
        });
    }
}
```

## Custom Cursors（自定义光标）[#](https://bevy.org/news/bevy-0-15/#custom-cursors)

作者：[@eero-lehtinen](https://github.com/eero-lehtinen)

PR：[#14284](https://github.com/bevyengine/bevy/pull/14284)

以前，Bevy 的原生窗口光标只支持一组固定的内置操作系统光标。Bevy 现在也支持任意图像作为"自定义光标"。自定义光标仍然使用操作系统的原生设施，这使得即使应用帧率下降，它们也能保持完全响应。

插入带有 [`CustomCursor`](https://docs.rs/bevy/0.15/bevy/winit/cursor/enum.CustomCursor.html) 的 [`CursorIcon`](https://docs.rs/bevy/0.15/bevy/winit/cursor/enum.CursorIcon.html) 组件来设置 [`Window`](https://docs.rs/bevy/0.15/bevy/prelude/struct.Window.html) 实体的光标：

```rust
commands
    .entity(window)
    .insert(CursorIcon::Custom(CustomCursor::Image {
        handle: asset_server.load("cursor_icon.png"),
        hotspot: (5, 5),
    }));
```

## Uniform Mesh Sampling（均匀网格采样）[#](https://bevy.org/news/bevy-0-15/#uniform-mesh-sampling)

作者：[@mweatherley](https://github.com/mweatherley)

PR：[#14071](https://github.com/bevyengine/bevy/pull/14071)

现在可以随机采样网格的表面。这可用于放置场景装饰或粒子效果等功能。

它包括：

1. `Mesh::triangles` 方法，允许提取 `Mesh` 的三角形列表（`Triangle3d`）。
2. `UniformMeshSampler` 类型，允许创建一个 [`Distribution`](https://docs.rs/rand/0.8.5/rand/distributions/trait.Distribution.html)，从三角形集合中均匀采样空间点（`Vec3`）。

将两者结合起来即可使用：

```rust
let mut rng = StdRng::seed_from_u64(8765309);

// 获取网格中三角形的迭代器。如果网格格式错误
// 或顶点/索引数据格式不正确，可能会失败。
let triangles = my_mesh.triangles().unwrap();

// 构造分布。在某些情况下可能会失败——最明显的是
// 网格表面面积为 0 时。
let distribution = UniformMeshSampler::try_new(triangles).unwrap();

// 从网格表面均匀采样 1000 个点。
let samples: Vec<Vec3> = distribution.sample_iter(&mut rng).take(1000).collect();
```

## `EventMutator`[#](https://bevy.org/news/bevy-0-15/#eventmutator)

作者：[@BobG1983](https://github.com/BobG1983)

PR：[#13818](https://github.com/bevyengine/bevy/pull/13818)

在处理复杂的事件驱动逻辑时，你可能会发现想要有条件地修改事件而不改变其类型或重新发出它们。虽然这总是可能的，但相当繁琐：

```rust
// 我们需要使用系统本地的 `EventCursor`（以前称为 `ManualEventReader`）
// 手动跟踪此系统已读取的事件。
fn mutate_events(mut events: ResMut<Events<MyEvent>>, mut local_cursor: Local<EventCursor<MyEvent>>){    
    for event in local_cursor.read_mut(&mut *events){
        event.some_mutation();
    }
}
```

现在，你可以简单地使用新的 [`EventMutator`](https://docs.rs/bevy/0.15/bevy/ecs/event/struct.EventMutator.html) 系统参数，它会自动为你跟踪这些簿记信息。

```rust
fn mutate_events(mut event_mutator: EventMutator<MyEvent>>){    
    for event in event_mutator.read(){
        event.some_mutation();
    }
}
```

## Isometry Types（等距变换类型）[#](https://bevy.org/news/bevy-0-15/#isometry-types)

作者：[@mweatherley](https://github.com/mweatherley), [@Jondolf](https://github.com/Jondolf)

PR：[#14269](https://github.com/bevyengine/bevy/pull/14269)

向量和四元数在 3D 中常用于描述对象的相对和绝对位置与朝向。然而，当执行更复杂的变换时，例如从全局参考系到对象局部空间再返回，或组合多个平移和旋转，它们可能变得相当笨拙且难以推理。

在 **Bevy 0.15** 中引入的新 [`Isometry2d`](https://docs.rs/bevy/0.15/bevy/math/struct.Isometry2d.html) 和 [`Isometry3d`](https://docs.rs/bevy/0.15/bevy/math/struct.Isometry3d.html) 类型是一个简单而强大的工具，用于有效描述这些种类的变换。等距变换表示一个旋转后跟一个平移，类似于缩放为 1 的 [`Transform`](https://docs.rs/bevy/0.15/bevy/transform/components/struct.Transform.html)。

```rust
// 从平移和旋转创建一个等距变换。
let iso1 = Isometry3d::new(Vec3::new(2.0, 1.0, 3.0), Quat::from_rotation_z(FRAC_PI_2));

// 使用等距变换变换一个点。
let point = Vec3::new(4.0, 4.0, 4.0);
let result = iso1.transform_point(point); // 或 iso1 * point
assert_relative_eq!(result, Vec3::new(-2.0, 5.0, 7.0));

// 创建另一个等距变换。
let iso2 = Isometry3d::from_rotation(Quat::from_rotation_z(FRAC_PI_2));

// 计算相对平移和旋转。
let relative_iso = iso1.inverse_mul(iso2); // 或 iso1.inverse() * iso2
```

等距变换在数学上下文中最有用，其中不需要缩放，例如在描述对象的相对位置以进行相交测试和其他几何查询时。然而，它们现在也用于一些 API，包括 gizmo 方法：

```rust
// 使用等距变换指定矩形位置和朝向。
gizmos.rect_2d(Isometry2d::new(translation, Rot2::degrees(45.0)), Vec2::splat(250.0), CYAN);

// 许多方法接受 `impl Into<Isometry3d>`，因此如果不需要完整的等距变换，
// 只提供平移或旋转就足够了。
gizmos.sphere(translation, 1.0, PURPLE);
```

[`Transform`](https://docs.rs/bevy/0.15/bevy/transform/components/struct.Transform.html) 和 [`GlobalTransform`](https://docs.rs/bevy/0.15/bevy/transform/components/struct.GlobalTransform.html) 也可以使用 [`to_isometry`](https://docs.rs/bevy/0.15/bevy/transform/components/struct.Transform.html#method.to_isometry) 方法转换为 [`Isometry3d`](https://docs.rs/bevy/0.15/bevy/math/struct.Isometry3d.html)，当你已经可以访问实体变换时，提供了一种方便的方式来使用这些 API。

请注意，与 [`Transform`](https://docs.rs/bevy/0.15/bevy/transform/components/struct.Transform.html) 不同，这些等距类型*不是*组件。它们纯粹是数学上的便捷类型。

## Lifecycle Hook & Observer Trigger for Replaced Values（针对替换值的生命周期 Hook 和观察者触发器）[#](https://bevy.org/news/bevy-0-15/#lifecycle-hook-observer-trigger-for-replaced-values)

作者：[@BigWingBeat](https://github.com/BigWingBeat)

PR：[#14212](https://github.com/bevyengine/bevy/pull/14212)

Bevy 0.14 引入了[组件生命周期 Hook 和观察者](https://bevy.org/news/bevy-0-14/#ecs-hooks-and-observers)，并为组件可以添加到实体或从实体移除的每种方式包含了几个内置的观察者触发器：`OnAdd`、`OnInsert` 和 `OnRemove`。然而，这个 API 存在一个漏洞。虽然 `OnRemove` 是 `OnAdd` 的对应项，但 `OnInsert` 没有这样的对应项，这意味着某些操作没有对应的生命周期 Hook 或观察者触发器：

```rust
use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    prelude::{Commands, Component, Deref, DerefMut, Entity, Query, Resource},
    utils::HashMap,
};

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
struct SomeId(u32);

#[derive(Resource, Deref, DerefMut)]
struct EntityLookupById(HashMap<SomeId, Entity>);

impl Component for SomeId {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks
            .on_insert(|mut world, entity, _| {
                let this = *world.entity(entity).get::<Self>().unwrap();
                world
                    .resource_mut::<EntityLookupById>()
                    .insert(this, entity);
            })
            .on_remove(|mut world, entity, _| {
                let this = *world.entity(entity).get::<Self>().unwrap();
                world.resource_mut::<EntityLookupById>().remove(&this);
            });
    }
}

fn some_system(mut commands: Commands, query: Query<(Entity, &SomeId)>) {
    let mut iter = query.iter();

    let Some((a_entity, _)) = iter.next() else {
        return;
    };

    let Some((_, &b_id)) = iter.next() else {
        return;
    };

    commands.entity(a_entity).insert(b_id);
}
```

在这个示例中，系统将一个已拥有组件值的实体的组件值替换为新的值，这会覆盖之前的组件值。这导致新值的 `on_insert` 生命周期 Hook 运行，但旧值的 `on_remove` Hook 不会运行。结果，之前 ID 值的哈希映射条目仍然存在，即使它已经被替换。

Bevy 0.15 为这种场景引入了一个新的组件生命周期 Hook 和观察者触发器：`on_replace`/`OnReplace`。此 Hook 在所有情况下都在 `on_remove` Hook 之前运行，并且在上述组件值被完全替换的场景中额外运行。该 Hook 在替换发生前运行，让你可以访问即将被丢弃的值以执行簿记和清理。

上述示例只需将 `on_remove` Hook 替换为新的 `on_replace` Hook 即可修复：

```diff
21                     .resource_mut::<EntityLookupById>()                                          
22                     .insert(this, entity);       
23             })                                   
-24             .on_remove(|mut world, entity, _| {  
+24             .on_replace(|mut world, entity, _| {
25                 let this = *world.entity(entity).get::<Self>().unwrap();                         
26                 world.resource_mut::<EntityLookupById>().remove(&this);                          
27             });                                  
```

请注意，如果组件值仅仅是*被修改*，它*不会*运行——在这些情况下，你应使用变更检测。

## Pack Multiple Vertex and Index Arrays Together into Growable Buffers（将多个顶点和索引数组打包到可增长的缓冲中）[#](https://bevy.org/news/bevy-0-15/#pack-multiple-vertex-and-index-arrays-together-into-growable-buffers)

作者：[@pcwalton](https://github.com/pcwalton)

PR：[#14257](https://github.com/bevyengine/bevy/pull/14257), [#15566](https://github.com/bevyengine/bevy/pull/15566), [#15569](https://github.com/bevyengine/bevy/pull/15569)

**Bevy 0.15** 改变了网格在 GPU 上的存储方式，大大提高了 CPU 性能。与 Bevy 0.14 中每个网格使用单独的顶点和索引缓冲不同，现在它们分别合并到可配置大小的"slab"中。这减少了需要更改绑定组的频率，为我们带来了高达 2 倍的速度提升！

`MeshAllocatorSettings` 资源允许调整 slab 大小、增长率和截止值，以最好地满足你的应用需求。对于大多数场景，默认值已经是一个显著的改进。

WebGL2 不支持将顶点缓冲打包在一起，因此在此平台上只有索引缓冲会被合并。

在 [Bistro](https://github.com/DGriffin91/bevy_bistro_scene) 场景上的一些测量：

整体帧时间从 8.74 ms 提升到 5.53 ms（1.58 倍加速）
渲染系统时间从 6.57 ms 提升到 3.54 ms（1.86 倍加速）
不透明通道时间从 4.64 ms 提升到 2.33 ms（1.99 倍加速）

## Rewrite Screenshots（重写截图功能）[#](https://bevy.org/news/bevy-0-15/#rewrite-screenshots)

作者：[@tychedelia](https://github.com/tychedelia)

PR：[#14833](https://github.com/bevyengine/bevy/pull/14833)

现在可以使用新的基于观察者的 API 来截取屏幕截图，该 API 允许针对任何可以与 `Camera` 一起使用的 `RenderTarget`，而不仅仅是窗口。

```rust
// 截取主窗口
commands
    .spawn(Screenshot::primary_window())
    .observe(save_to_disk(path));

// 或者截取 `Handle<Image>`
commands
    .spawn(Screenshot::image(render_target))
    .observe(save_to_disk(path));
```

观察者会触发一个 `ScreenshotCaptured` 事件，其中包含一个可用于保存到磁盘、后处理或生成缩略图的 Image。这种灵活的方法使得从渲染管线的任何部分捕获内容变得更加容易，无论是窗口、离屏渲染目标，还是自定义渲染通道中的纹理。

## `SystemParamBuilder`[#](https://bevy.org/news/bevy-0-15/#systemparambuilder)

作者：[@chescock](https://github.com/chescock)

PR：[#14050](https://github.com/bevyengine/bevy/pull/14050), [#14821](https://github.com/bevyengine/bevy/pull/14821), [#14818](https://github.com/bevyengine/bevy/pull/14818), [#15189](https://github.com/bevyengine/bevy/pull/15189), [#14817](https://github.com/bevyengine/bevy/pull/14817)

Bevy 0.14 引入了 [`SystemBuilder` 类型](https://bevy.org/news/bevy-0-14/#systembuilder) 来允许使用动态查询创建系统。在 Bevy 0.15 中，这已扩展到更多类型的系统参数！

`SystemBuilder` 类型已被替换为 `SystemParamBuilder<P>` trait，以使其更容易组合构建器。参数聚合，包括[元组、`ParamSet`](https://github.com/bevyengine/bevy/pull/14050)、[`Vec<T>`](https://github.com/bevyengine/bevy/pull/14821) 和[使用 `#[derive(SystemParam)]` 的自定义参数](https://github.com/bevyengine/bevy/pull/14818)，现在可以在动态系统中使用。例如，`ParamSet<Vec<Query<FilteredEntityMut>>>` 可用于传递可能冲突的可变数量动态查询。

新的 [`FilteredResources` 和 `FilteredResourcesMut`](https://github.com/bevyengine/bevy/pull/15189) 类型可以访问运行时配置的一组资源，类似于现有的 `FilteredEntityRef` 和 `FilteredEntityMut` 访问一个实体上的一组组件。

最后，一个新的 [`DynSystemParam`](https://github.com/bevyengine/bevy/pull/14817) 类型允许系统使用动态类型的参数，然后向下转型。这对于使用 trait 对象实现系统的一部分特别有用，其中每个 trait 实现可以使用不同的系统参数类型。

组合起来，这些可以用于构建一个运行运行时定义脚本的系统，其中脚本需要可变数量的查询和资源参数。或者，它们可用于从运行时组装的部件构建系统！

```rust
fn buildable_system(
    query_a: Query<&A>,
    query_b: Query<&B>,
    queries_with_locals: Vec<(Query<FilteredEntityMut>, Local<usize>)>,
    mut dynamic_params: ParamSet<Vec<DynSystemParam>>,
    resources: FilteredResourcesMut,
) {
    // `ParamSet<Vec>` 中的参数按索引访问。
    let mut dyn_param_0: DynSystemParam = dynamic_params.get_mut(0);
    // `DynSystemParam` 中的参数通过向下转型为原始类型来访问。
    let param: Local<&str> = dyn_param_0.downcast_mut::<Local<&str>>().unwrap();
    // `FilteredResources` 和 `FilteredResourcesMut` 有按类型或按 ID 获取资源的方法。
    let res: Ref<R> = resources.get::<R>().unwrap();
}

let param_builder = (
    // 不需要配置的参数可以使用 `ParamBuilder` 或其工厂方法构建。
    ParamBuilder,
    ParamBuilder::query(),
    // 参数 `Vec` 可以使用构建器 `Vec` 来构建。
    vec![
        // 参数元组可以使用构建器元组来构建。
        (
            // 查询使用回调构建，该回调提供 `QueryBuilder` 来配置查询。
            QueryParamBuilder::new(|builder| { builder.data::<&A>(); }),
            // Locals 通过传递 local 的初始值来构建。
            LocalBuilder(123),
        ),
    ],
    // `ParamSet` 可以为元组或 `Vec` 构建。
    ParamSetBuilder(vec![
        // `DynSystemParam` 使用任何类型的构建器构建，并且可以向下转型为该类型。
        DynParamBuilder::new(LocalBuilder("hello")),
        DynParamBuilder::new(ParamBuilder::resource::<R>()),
        // 类型可以是任何系统参数，甚至是元组或 `Vec`！
        DynParamBuilder::new((ParamBuilder::query::<&A>(), ParamBuilder::query::<&B>())),
    ]),
    // `FilteredResources` 和 `FilteredResourcesMut` 使用回调构建，
    // 该回调提供构建器来配置资源访问。
    FilteredResourcesMutParamBuilder::new(|builder| { builder.add_read::<R>(); }),
);

let system = param_builder
    .build_state(&mut world)
    .build_system(buildable_system);

// 构建的系统就像任何其他系统一样，可以添加到调度中。
schedule.add_systems(system);
```

## State Scoped Events（状态作用域事件）[#](https://bevy.org/news/bevy-0-15/#state-scoped-events)

作者：[@UkoeHB](https://github.com/UkoeHB)

PR：[#15085](https://github.com/bevyengine/bevy/pull/15085)

状态作用域事件在退出某个状态时会自动清除（类似于 [StateScoped 实体](https://bevy.org/news/bevy-0-14/#state-scoped-entities)）。当你想保证干净的状态转换时，这很有用。

通常，你会通过以下方式配置事件：

```rust
fn setup(app: &mut App) {
    app.add_event::<MyGameEvent>();
}
```

如果你希望在退出特定状态时清除事件，将其改为：

```rust
fn setup(app: &mut App) {
    app.add_state_scoped_event::<MyGameEvent>(GameState::Play);
}
```

## `EntityRefExcept` and `EntityMutExcept`[#](https://bevy.org/news/bevy-0-15/#entityrefexcept-and-entitymutexcept)

作者：[@pcwalton](https://github.com/pcwalton)

PR：[#15207](https://github.com/bevyengine/bevy/pull/15207)

[`EntityMut`](https://docs.rs/bevy/0.15/bevy/ecs/prelude/struct.EntityMut.html) 和 [`EntityRef`](https://docs.rs/bevy/0.15/bevy/ecs/prelude/struct.EntityRef.html) 是强大的工具，可以以任意方式一次性与给定实体的所有组件进行交互。这些类型实现了 `QueryData`，因此你可以将它们添加到任何你想要的 `Query` 中！

然而，由于它们可以访问*任何*组件信息，Rust 对可变别名引用的禁止阻止你同时访问其他组件信息，即使你保证不会读取任何正在被写入的数据。

```rust
// 此系统是被禁止的！
//
// 在函数体内，我们可以在读取其值时选择修改 `AnimationPlayer` 本身！
fn animate_anything(query: Query<(&AnimationPlayer, EntityMut)> ){}
```

为了让你绕过这个限制，我们引入了一组对应的工具：[`EntityMutExcept`](https://docs.rs/bevy/0.15/bevy/ecs/world/struct.EntityMutExcept.html) 和 [`EntityRefExcept`](https://docs.rs/bevy/0.15/bevy/ecs/world/struct.EntityRefExcept.html)，它们的工作方式与 [`EntityMut`](https://docs.rs/bevy/0.15/bevy/ecs/prelude/struct.EntityMut.html) 和 [`EntityRef`](https://docs.rs/bevy/0.15/bevy/ecs/prelude/struct.EntityRef.html) 类似，但*不*提供对你声明为禁止访问的一组组件的访问。

```rust
/// 看，没有可变别名！
fn animate_anything(query: Query<(&AnimationPlayer, EntityMutExcept<AnimationPlayer>)> ){}
```

## Cached One-shot Systems（缓存的一次性系统）[#](https://bevy.org/news/bevy-0-15/#cached-one-shot-systems)

作者：[@benfrankel](https://github.com/benfrankel)

PR：[#14920](https://github.com/bevyengine/bevy/pull/14920)

**Bevy 0.15** 引入了一个便捷的新"缓存"API 来运行一次性系统：

```rust
// 旧的未缓存 API：
let foo_id = commands.register_system(foo);
commands.run_system(foo_id);

// 新的缓存 API：
commands.run_system_cached(foo);
```

这允许你调用 `register_system_cached` 而无需担心产生重复的系统。

```rust
// 未缓存 API：
let id1 = world.register_system(quux);
let id2 = world.register_system(quux);
assert!(id1 != id2);

// 缓存 API：
let id1 = world.register_system_cached(quux);
let id2 = world.register_system_cached(quux);
assert!(id1 == id2);
```

### 与 `run_system_once` 的对比

`run_system_once` 设置一个系统，运行一次，然后拆除。这意味着像 `Local` 和 `EventReader` 这样的系统参数依赖于运行之间的持久状态，会被丢失。任何像 `Query` 这样依赖缓存计算来提高性能的系统参数每次都必须重建缓存，这可能代价高昂。因此，`run_system_once` 仅推荐用于诊断用途（例如单元测试），而 `run_system` 或 `run_system_cached` 应优先用于"真正的"代码。

### 限制

使用缓存 API，不同的系统不能缓存到同一个 `CachedSystemId<S>` 下。对于类型 `S`，最多只能有一个不同的系统。这在 `size_of::<S>() == 0` 时为真，在实践中几乎总是如此。为了强制执行正确性，新的 API 会在你尝试使用非零大小的函数（如函数指针或捕获闭包）时给出编译时错误。

## Fallible System Parameters（可失败的系统参数）[#](https://bevy.org/news/bevy-0-15/#fallible-system-parameters)

作者：[@MiniaczQ](https://github.com/MiniaczQ)

PR：[#15276](https://github.com/bevyengine/bevy/pull/15276), [#15476](https://github.com/bevyengine/bevy/pull/15476), [#15488](https://github.com/bevyengine/bevy/pull/15488)

在 Bevy 0.14 及之前，以下代码会 panic：

```rust
#[derive(Resource)]
struct MyResource;

fn my_system(my_resource: Res<MyResource>) {}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
    app.add_systems(my_system);
    // 此处 panic：`my_system` 无法获取 `MyResource`，因为它从未被添加。
    app.run();
}
```

但在 Bevy 0.15 中，`my_system` 根本不会被执行，并会记录一个警告。

这适用于所有基于系统的功能：

- 系统和观察者将被跳过。
- 运行条件将被跳过并返回 `false`。

复合系统，如 `system_a.pipe(system_b)`，如果缺少任何所需数据，当前也会被跳过。

从此功能中受益的现有参数有：`Res` 和 `ResMut`，以及它们的兄弟 `NonSend` 和 `NonSendMut`。构建在其他参数之上的参数：元组、`DynSystemParam` 和 `ParamSet` 被视为存在，当且仅当它们的所有系统参数都存在。

此外，还引入了一些新的系统参数来简化现有代码：

- [`Single<D, F>`](https://docs.rs/bevy/0.15/bevy/ecs/prelude/struct.Single.html) - 像 `Query<D, F>::single` 一样工作，如果查询包含 0 个或多于 1 个匹配项则失败，
- `Option<Single<D, F>>` - 像 `Query<D, F>::single` 一样工作，如果查询包含多于 1 个匹配项则失败，
- [`Populated<D, F>`](https://docs.rs/bevy/0.15/bevy/ecs/prelude/struct.Populated.html) - 像 `Query<D, F>` 一样工作，如果查询没有匹配项则失败。

### 警告

可失败的系统参数带有一个原始的警告机制。目前，系统可以以两种方式之一运行：

- （默认）仅警告一次，
- 从不警告。

默认值可以按如下方式更改：

```rust
// 对于系统
app.add_systems(my_system.never_param_warn());
// 对于观察者
app.add_observer(my_observer.never_param_warn());
// 对于运行条件
app.add_systems(my_system.run_if(my_condition.never_param_warn()));
```

请告诉我们你还想要哪些其他警告策略！

## Passing Data Into Systems By Reference（通过引用将数据传递到系统）[#](https://bevy.org/news/bevy-0-15/#passing-data-into-systems-by-reference)

作者：[@ItsDoot](https://github.com/ItsDoot)

PR：[#15184](https://github.com/bevyengine/bevy/pull/15184)

系统管道是一个强大的（尽管相对小众的）工具，用于将数据直接从一个系统传递到另一个系统。虽然这对于[错误处理](https://github.com/bevyengine/bevy/blob/main/examples/ecs/system_piping.rs)很有用，但它是一个通用工具，用于通过将匹配的输入和输出粘合在一起来组合逻辑片段。

这个机制后来被重新用于[一次性系统](https://bevy.org/news/bevy-0-12/#one-shot-systems)，允许你调用 [`World::run_system_with_input`](https://docs.rs/bevy/0.15.0/bevy/ecs/prelude/struct.World.html#method.run_system_with_input) 来使用你提供的任何输入评估系统，并获取返回值。非常适合编写测试！

然而，这套工具一直有一个令人沮丧且困惑的限制：传递到系统中的任何数据都必须具有静态生命周期。这似乎很荒谬；数据直接从一个所有者传递到下一个，系统就像单个单元一样运行。

通过灵活运用一些类型魔法，这个限制已经被解除了！

```rust
let mut world = World::new();

let mut value = 2;

// 这总是有效的：
fn square(In(input): In<usize>) -> usize {
    input * input
}
value = world.run_system_with_input(value, square);

// 现在也可以：
fn square_ref(InRef(input): InRef<usize>) -> usize {
    *input * *input
}
value = world.run_system_with_input(&value, square_ref);

// 可变方式：
fn square_mut(InMut(input): InMut<usize>) {
    *input *= *input;
}
world.run_system_with_input(&mut value, square_mut);
```

我们很期待看到你用这个新获得的能力做什么。

## List Components in QueryEntityError::QueryDoesNotMatch（在 QueryEntityError::QueryDoesNotMatch 中列出组件）[#](https://bevy.org/news/bevy-0-15/#list-components-in-queryentityerror-querydoesnotmatch)

作者：[@SpecificProtagonist](https://github.com/SpecificProtagonist)

PR：[#15435](https://github.com/bevyengine/bevy/pull/15435)

当通过查询访问实体失败时，错误现在包含实体拥有的组件的名称：

```
QueryDoesNotMatch(0v1 with components Sprite, Transform, GlobalTransform, Visibility, InheritedVisibility, ViewVisibility, SyncToRenderWorld)
```

## `no_std` Progress（`no_std` 进展）[#](https://bevy.org/news/bevy-0-15/#no-std-progress)

作者：[@bushrat011899](https://github.com/bushrat011899)

PR：[#15281](https://github.com/bevyengine/bevy/pull/15281)

Bevy 严重依赖 Rust 的[标准库](https://doc.rust-lang.org/std/)，这使得在嵌入式、小众平台甚至某些游戏主机上使用变得具有挑战性。但如果不是这样呢？

我们已经启动了一项新计划，挑战对标准库的依赖，最终目标是提供与 [`no_std`](https://docs.rust-embedded.org/book/intro/no-std.html) 兼容的 Bevy 子集，使其能够在更广泛的平台上使用。

第一个非常简单的步骤是启用一组新的 lint：

- [`std_instead_of_core`](https://rust-lang.github.io/rust-clippy/master/index.html#std_instead_of_core)
- [`std_instead_of_alloc`](https://rust-lang.github.io/rust-clippy/master/index.html#std_instead_of_alloc)
- [`alloc_instead_of_core`](https://rust-lang.github.io/rust-clippy/master/index.html#alloc_instead_of_core)

对于不熟悉 `no_std` Rust 的人来说，标准库 `std` 的许多功能来自两个较小的 crate：[`core`](https://doc.rust-lang.org/core/) 和 [`alloc`](https://doc.rust-lang.org/alloc/)。`core` crate 在几乎所有 Rust 目标上都可用，提供了 Rust 语言依赖的基础设施，如迭代器、`Result` 等。补充之下，`alloc` crate 提供了与分配相关的功能，如 `Vec`、`Box` 和 `String`。

Rust 对平台的支持遵循[三级策略](https://doc.rust-lang.org/rustc/platform-support.html)，其中第一级保证可用且始终提供 `std` crate，而第二级和第三级*可能*有 `std` crate，但通常没有。原因是有些平台根本不支持 `std` crate 所需的功能，如文件系统、网络或线程。

但 Bevy 为什么要在意这些平台呢？当一个新的平台被添加到 Rust 中时，它通常缺乏第一级支持。即使是现代主机，如 Nintendo Switch、PlayStation 5 或 Xbox Series，由于保密协议和平台特殊性，也没有第一级支持。为 Bevy 添加 `no_std` 支持将使为这些平台开发商业团队更容易上手和保持更新。

除了商业相关的现代主机之外，还有一个充满活力的嵌入式和复古爱好者社区，他们为可能永远不支持标准库的平台进行开发。诸如 [`agb`](https://crates.io/crates/agb) 和 [`psx`](https://crates.io/crates/psx) 之类的 crate 分别提供了在 GameBoy Advance 和 PlayStation One 上开发游戏的支持。有了 Bevy 中的 `no_std` 支持，用户也许能够利用更广泛的 Rust 生态系统在这些平台上运行他们的软件。

我们距离 Bevy 中真正的 `no_std` 支持还有一段距离，但最初的一些更改已经被接受，还有更多更改计划在下一个 0.16 版本中进行。

如果这项工作听起来有趣，请查看 GitHub 上的 [`no_std` 跟踪问题](https://github.com/bevyengine/bevy/issues/15460)，你可以在那里找到拉取请求列表，甚至是在 `no_std` 环境中运行 Bevy 的原型。

## `GltfMaterialName` Component（`GltfMaterialName` 组件）[#](https://bevy.org/news/bevy-0-15/#gltfmaterialname-component)

作者：[@Soulghost](https://github.com/Soulghost)

PR：[#13912](https://github.com/bevyengine/bevy/pull/13912)

glTF 3D 模型文件格式允许单个网格关联多个材质。例如，一个茶壶可能由一个网格组成，但每个部分可能有不同的材质。当单个网格被分配多个材质时，它被分割成几个基本节点，每个基本节点分配一个唯一的材质。

```json
{
  "meshes": [
    {
      "name": "Cube",
      "primitives": [
        {
          "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 },
          "indices": 3,
          "material": 0
        },
        {
          "attributes": { "POSITION": 4, "NORMAL": 5, "TEXCOORD_0": 6 },
          "indices": 7,
          "material": 1
        },
        {
          "attributes": { "POSITION": 8, "NORMAL": 9, "TEXCOORD_0": 10 },
          "indices": 11,
          "material": 2
        },
        {
          "attributes": { "POSITION": 12, "NORMAL": 13, "TEXCOORD_0": 14 },
          "indices": 15,
          "material": 3
        }
      ]
    }
  ]
}
```

在 Bevy 0.14 及之前，这些基本节点使用"Mesh.Index"格式命名，这使查询变得复杂。一个新的组件 [GltfMaterialName](https://docs.rs/bevy/0.15/bevy/gltf/struct.GltfMaterialName.html) 现在被添加到每个具有材质的基本节点上，让你可以通过使用此组件配合材质名称快速查找基本节点。

```rust
fn find_top_material_and_mesh(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
    mesh_materials: Query<(
        &MeshMaterial3d<StandardMaterial>,
        &Mesh3d,
        &GltfMaterialName,
    )>,
) {
    for (mat_handle, mesh_handle, name) in &mesh_materials {
        // 按名称定位材质和关联的子网格
        if name.0 == "Top" {
            if let Some(material) = materials.get_mut(mat_handle) {
                // ...
            }

            if let Some(mesh) = meshes.get_mut(mesh_handle) {
                // ...
            }
        }
    }
}
```

## GPU Readback（GPU 回读）[#](https://bevy.org/news/bevy-0-15/#gpu-readback)

作者：[@tychedelia](https://github.com/tychedelia)

PR：[#15419](https://github.com/bevyengine/bevy/pull/15419)

新的 `Readback` 组件使用基于观察者的 API 简化了从 GPU 获取数据到 CPU 的棘手过程。

```rust
commands.spawn(Readback::buffer(buffer.clone())).observe(
    |trigger: Trigger<ReadbackComplete>| {
        let data = trigger.event().to_shader_type();
        // ...
    },
);
```

通常，手动从 GPU 检索数据涉及大量样板代码和 GPU 资源的仔细管理。你必须处理同步问题，确保 GPU 已完成处理，并在内存空间之间处理数据复制——这并不简单！

新的 `Readback` 组件简化了这一过程。当在主世界中生成时，`Readback` 会将一个 `Handle<Image>` 或 `Handle<ShaderStorageBuffer>` 排队，以便在未来帧中异步读取并从 GPU 复制回 CPU，此时它会触发一个包含资源原始字节的 `ReadbackComplete` 事件。

这对于调试、保存 GPU 生成的数据或使用 GPU 的结果执行 CPU 端计算特别有用。它非常适合需要分析模拟数据、捕获渲染帧或在 GPU 上处理大数据集并检索结果以供 CPU 进一步使用的场景。

## Android: Configurable `GameActivity` and `NativeActivity`（Android：可配置的 `GameActivity` 和 `NativeActivity`）[#](https://bevy.org/news/bevy-0-15/#android-configurable-gameactivity-and-nativeactivity)

作者：[@Litttlefish](https://github.com/Litttlefish)

PR：[#12095](https://github.com/bevyengine/bevy/pull/12095)

Bevy 现在使用 `GameActivity` 作为 Android 项目的默认 `Activity`，取代了 `NativeActivity`。`NativeActivity` 仍然可用，但已置于功能标志之后。

这一更改将 Bevy 升级到更现代的 Android 栈，并包括 SDK 最低版本的提升，以符合 [PlayStore 当前的版本要求](https://developer.android.com/distribute/best-practices/develop/target-sdk)。我们还切换到了基于 [`cargo-ndk`](https://docs.rs/crate/cargo-ndk/3.5.4) 的构建，这默认给了我们更多的控制权。提供了 `GameActivity` 和 `NativeActivity` 的 Gradle 项目。

`GameActivity` 带来了游戏交互的改进（`SurfaceView` 渲染、改进的触摸和输入处理）、更频繁的更新以及对 [JetPack](https://developer.android.com/jetpack) 生态系统其他部分的访问。它能够更好地与 Rust 代码集成，无需过多的 JNI 处理。你可以[在此处](https://developer.android.com/games/agdk/game-activity)阅读更多关于 `GameActivity` 的信息。

## Reflection Serialization Improvements（反射序列化改进）[#](https://bevy.org/news/bevy-0-15/#reflection-serialization-improvements)

作者：[@MrGVSV](https://github.com/MrGVSV), [@aecsocket](https://github.com/aecsocket)

PR：[#8611](https://github.com/bevyengine/bevy/pull/8611), [#15482](https://github.com/bevyengine/bevy/pull/15482), [#15548](https://github.com/bevyengine/bevy/pull/15548), [#13888](https://github.com/bevyengine/bevy/pull/13888)

#### 带注册表上下文的序列化

[`bevy_reflect`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/) 提供了一种简单的方法来序列化和反序列化几乎任何实现了 [`Reflect`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/trait.Reflect.html) 的类型。它完全依赖反射 API 和 [`TypeRegistry`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/struct.TypeRegistry.html) 来实现，无需在编译时知道类型。

然而，有时类型的序列化/反序列化需要更显式的控制。在这种情况下，可以通过在 `TypeRegistry` 中为该类型注册 [`ReflectSerialize`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/trait.ReflectSerialize.html)/[`ReflectDeserialize`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/trait.ReflectDeserialize.html) 类型数据来提供自定义的 `Serialize`/`Deserialize` 实现。

这种方法对于大多数情况来说工作得足够好。然而，有时你只想单独处理你的类型，而对其余字段继续使用反射。例如，你可能想将你的类型序列化为一个包含一些额外条目的映射，但你仍然希望为每个值使用反射序列化器。

不幸的是，这不仅在序列化器内部嵌套得很差，而且意味着你需要手动捕获对 `TypeRegistry` 的引用，以便将其传递给嵌套的反射序列化器。这基本上意味着你无法同时使用自定义逻辑和基于反射的序列化。

幸运的是，Bevy 0.15 引入了 [`SerializeWithRegistry`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/serde/trait.SerializeWithRegistry.html) 和 [`DeserializeWithRegistry`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/serde/trait.DeserializeWithRegistry.html) trait，它们的工作方式很像 `Serialize` 和 `Deserialize`，但多了一个 `TypeRegistry` 参数。这允许你执行自定义逻辑，同时仍然能够继续对其余部分使用反射。

```rust
impl SerializeWithRegistry for MyType {
    fn serialize<S>(&self, serializer: S, registry: &TypeRegistry) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let state = serializer.serialize_map(None)?;

        // ...自定义逻辑...

        state.serialize_entry(
            "data",
            // 继续使用基于反射的序列化
            &ReflectSerializer::new(
                self.data,
                registry,
            ),
        )?;

        state.end()
    }
}
```

有了自定义的序列化和反序列化逻辑，你可以为你的类型注册 [`ReflectSerializeWithRegistry`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/serde/trait.ReflectSerializeWithRegistry.html) 和 [`ReflectDeserializeWithRegistry`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/serde/trait.ReflectDeserializeWithRegistry.html) 类型数据，以便反射序列化器/反序列化器对你的类型的所有实例使用你的自定义逻辑。

#### Reflect 序列化/反序列化处理器

除了 `SerializeWithRegistry` 和 `DeserializeWithRegistry` 之外，还为使用反射机制进行序列化/反序列化的用户添加了一个新工具。当使用 `ReflectSerializer` 或 `ReflectDeserializer` 时，你现在可以使用 `with_processor` 并传入一个*序列化/反序列化处理器*。这个处理器允许你覆盖特定值和特定类型的序列化/反序列化逻辑，同时还可以在处理器内部捕获你可能需要的任何上下文。

这个功能的典型用例是在反射反序列化时，在资源加载器中正确反序列化 `Handle<T>`。假设我们有一个如下所示的资源：

```rust
#[derive(Debug, Clone, Reflect)]
struct AnimationGraph {
    nodes: Vec<Box<dyn AnimationNode>>,
}

trait AnimationNode: Send + Sync + Reflect { /* .. */ }

#[derive(Debug, Clone, Reflect)]
struct ClipNode {
    clip: Handle<AnimationClip>
}

impl AnimationNode for ClipNode { /* .. */ }

#[derive(Debug, Clone, Reflect)]
struct AdjustSpeedNode {
    speed_multiplier: f32,
}

impl AnimationNode for AdjustSpeedNode { /* .. */ }
```

```ron
(
    animation_graph: (
        nodes: [
            {
                "my_app::animation::node::ClipNode": (
                    clip: "animations/run.anim.ron",
                )
            },
            {
                "my_app::animation::node::AdjustSpeedNode": (
                    speed_multiplier: 1.5,
                )
            }
        ]
    )
)
```

当我们为这个 `AnimationGraph` 编写 `AssetLoader` 时，我们可以访问 `&mut LoadContext`，我们可以用它来启动新的资源加载操作，并获取该资源的 `Handle`。我们也可以使用现有的 `ReflectDeserializer` 来反序列化 `Box<dyn AnimationNode>`。然而，当反序列化器遇到 `Handle<AnimationClip>` 时，它会被反序列化为 `Handle::default`，并且不会启动任何资源加载，使得 handle 变得无用。

使用 [`ReflectDeserializerProcessor`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/serde/trait.ReflectDeserializerProcessor.html)，我们可以传入一个处理器，它捕获 `&mut LoadContext`，并在遇到 `Handle<T>` 时，启动对 `T` 的资源加载，并将该加载的结果分配给正在反序列化的字段。

```rust
struct HandleProcessor<'a> {
    load_context: &'a mut LoadContext,
}

impl ReflectDeserializerProcessor for HandleProcessor<'_> {
    fn try_deserialize<'de, D>(
        &mut self,
        registration: &TypeRegistration,
        _registry: &TypeRegistry,
        deserializer: D,
    ) -> Result<Result<Box<dyn PartialReflect>, D>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Some(reflect_handle) = registration.data::<ReflectHandle>() else {
            // 我们不想反序列化这个——将反序列化器返还
            // 执行默认的反序列化逻辑
            return Ok(Err(deserializer));
        };

        let asset_type_id = reflect_handle.asset_type_id();
        let asset_path = deserializer.deserialize_str(AssetPathVisitor)?;

        let handle: Handle<LoadedUntypedAsset> = self.load_context
            .loader()
            .with_dynamic_type(asset_type_id)
            .load(asset_path);
        Ok(Box::new(handle))
    }
}
```

结合 [`ReflectSerializerProcessor`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/serde/trait.ReflectSerializerProcessor.html)，这可以用于将 `Handle` 与字符串资源路径之间进行往返序列化。

处理器优先于所有其他 serde 逻辑，包括 `De/SerializeWithRegistry`，因此它可以用于覆盖任何反射序列化逻辑。

#### 上下文序列化错误

有时在使用反射序列化器和反序列化器时，很难追踪错误的来源。由于我们无法知道一个类型在运行时是否可以被序列化，一个不可序列化的类型可能会潜入到本应可序列化的类型中。

在 Bevy 0.15 中，一个新的默认 [`debug`](https://docs.rs/bevy_reflect/0.15/bevy_reflect/index.html#debug) 功能已添加到 `bevy_reflect` crate 中，它允许序列化器和反序列化器保留上下文信息，以便在错误发生时提供类型的"堆栈"。

这些消息可用于更容易地追踪错误的来源：

```
type `bevy_utils::Instant` did not register the `ReflectSerialize` type data. For certain types, this may need to be registered manually using `register_type_data` (stack: `bevy_time::time::Time<bevy_time::real::Real>` -> `bevy_time::real::Real` -> `bevy_utils::Instant`)
```

## Simplified Multi-Entity Access（简化的多实体访问）[#](https://bevy.org/news/bevy-0-15/#simplified-multi-entity-access)

作者：[@ItsDoot](https://github.com/ItsDoot)

PR：[#15614](https://github.com/bevyengine/bevy/pull/15614)

当使用 Bevy ECS 的一些更高级的功能时，如 Hook 或排他性系统，通常需要直接从 `World` 中获取实体：

```rust
#[derive(Component)]
#[component(on_add = on_foo_added)]
struct Foo;

fn on_foo_added(world: DeferredWorld, entity: Entity, _: ComponentId) {
    let has_foo = world.entity(entity);
    println!("{:?} has a Foo", has_foo.id());
}
```

在之前的 Bevy 版本中，你可以使用各种不同的函数从 `World` 中获取多个实体：

- `World::many_entities<N>(&self, [Entity; N]) -> [EntityRef; N]`
- `World::many_entities_mut<N>(&mut self, [Entity; N]) -> [EntityMut; N]`
- `World::get_many_entities<N>(&self, [Entity; N]) -> Result<[EntityRef; N], Entity>`
- `World::get_many_entities_dynamic(&self, &[Entity]) -> Result<Vec<EntityRef>, Entity>`
- `World::get_many_entities_mut<N>(&mut self, [Entity; N]) -> Result<[EntityMut; N], QueryEntityError>`
- `World::get_many_entities_dynamic_mut(&self, &[Entity]) -> Result<Vec<EntityMut>, QueryEntityError>`
- `World::get_many_entities_from_set_mut(&mut self, &EntityHashSet) -> Result<Vec<EntityMut>, QueryEntityError>`

如你所见，有很多函数名字非常长！但它们的要点是，我们希望能够给出一堆实体 ID，并收到一堆实体引用。肯定有更好的方法！

在 `0.15` 中，所有这些函数已被弃用，现在你只需要会 panic 的 `World::entity`/`World::entity_mut` 或不会 panic 的 `World::get_entity`/`World::get_entity_mut`：

```rust
let e1: Entity = world.spawn_empty().id();
let e2: Entity = world.spawn_empty().id();

// 注意：使用 World::get_entity 或 World::get_entity_mut 来接收 Result

// 你仍然可以像正常一样传递单个 ID：
let eref = world.entity(e1);  
let emut = world.entity_mut(e1);

// 但你也可以传入一个 ID 数组（支持任意数量 N！）：
let [eref1, eref2]: [EntityRef; 2] = world.entity([e1, e2]);
let [emut1, emut2]: [EntityMut; 2] = world.entity_mut([e1, e2]);

// 或者一个 ID 切片：
let ids = vec![e1, e2];
let eref_vec: Vec<EntityRef> = world.entity(&ids);
let emut_vec: Vec<EntityMut> = world.entity_mut(&ids);

// 甚至是一组 ID：
let ids = EntityHashSet::from_iter([e1, e2]);
let eref_map: EntityHashMap<EntityRef> = world.entity(&ids);
let emut_map: EntityHashMap<EntityMut> = world.entity_mut(&ids);
```

这可能*感觉*像是魔法，但都是标准的 Rust 代码！`World::entity` 系列函数接受的 `Entity` ID 参数被改为接受任何实现了新引入 trait [`WorldEntityFetch`](https://docs.rs/bevy/0.15/bevy/ecs/world/trait.WorldEntityFetch.html) 的类型。查看该 trait 和 [`World::entity`](https://docs.rs/bevy/0.15/bevy/ecs/prelude/struct.World.html#method.entity) 以了解更多关于它是如何实现的。

## Hierarchy Traversal Tools（层次遍历工具）[#](https://bevy.org/news/bevy-0-15/#hierarchy-traversal-tools)

作者：[@alice-i-cecile](https://github.com/alice-i-cecile)

PR：[#15627](https://github.com/bevyengine/bevy/pull/15627)

我们改进了 [`HierarchyQueryExt`](https://docs.rs/bevy/0.15/bevy/hierarchy/trait.HierarchyQueryExt.html) [扩展 trait](https://rust-lang.github.io/rfcs/0445-extension-trait-conventions.html)，使其更容易遍历由 [`Parent`](https://docs.rs/bevy/0.15/bevy/hierarchy/struct.Parent.html) 和 [`Children`](https://docs.rs/bevy/0.15/bevy/hierarchy/struct.Children.html) 组件定义的实体层次结构。

现在完整的方法集是：

- `parent`（新增）
- `children`（新增）
- `root_ancestor`（新增）
- `iter_leaves`（新增）
- `iter_siblings`（新增）
- `iter_descendants`
- `iter_descendants_depth_first`（新增）
- `iter_ancestors`

所有这些操作以前都是可能的，但我们希望这个 API 使使用层次结构更加愉快，特别是对于 UI 和动画。

## Shader Storage Buffer Asset（着色器存储缓冲资源）[#](https://bevy.org/news/bevy-0-15/#shader-storage-buffer-asset)

作者：[@tychedelia](https://github.com/tychedelia)

PR：[#14663](https://github.com/bevyengine/bevy/pull/14663)

一个新的资源 `ShaderStorageBuffer` 已添加，以简化在自定义材质和计算着色器中使用存储缓冲的工作。存储缓冲是大型的、GPU 可访问的内存缓冲，设计用于存储可以被着色器读取或写入的数据。与更小、更受限制的统一缓冲相比，存储缓冲允许你存储大量数据，使其成为需要处理大型数据集的通用任务的完美选择。例如，在物理模拟（如粒子系统）中管理复杂数据、保存场景中数千个对象的变换数据，或存储用于动态地形生成的程序化几何信息。当渲染管线的不同阶段（如计算着色器和渲染通道）需要高效共享和更新大量数据时，存储缓冲特别有用。

```rust
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {
    #[storage(0, read_only)]
    colors: Handle<ShaderStorageBuffer>,
}

fn setup(
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    // 存储缓冲的示例数据
    let color_data: Vec<[f32; 4]> = vec![
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0, 0.0, 1.0],
        [0.0, 1.0, 1.0, 1.0],
    ];

    let colors = buffers.add(ShaderStorageBuffer::from(color_data));

    // 使用存储缓冲创建自定义材质
    let custom_material = CustomMaterial { colors };

    materials.add(custom_material);
}
```

通过在材质上使用 `AsBindGroup` 声明 `Handle<ShaderStorageBuffer>`，这个缓冲现在可以在着色器中访问：

```wgsl
@group(2) @binding(0) var<storage, read> colors: array<vec4<f32>, 5>;
```

## Accumulated Mouse Inputs（累积鼠标输入）[#](https://bevy.org/news/bevy-0-15/#accumulated-mouse-inputs)

作者：[@Aztro-dev](https://github.com/Aztro-dev), [@alice-i-cecile](https://github.com/alice-i-cecile)

PR：[#14044](https://github.com/bevyengine/bevy/pull/14044)

"玩家这帧移动了多少鼠标"是游戏中一个常见的问题，当玩家试图瞄准或滚动地图时。不幸的是，操作系统以及因此的 [`winit`](https://docs.rs/winit/latest/winit/) 只为我们提供了一个事件流，以单独的 [`MouseMotion`](https://docs.rs/bevy/0.15.0/bevy/input/mouse/struct.MouseMotion.html) 事件的形式。

要获取大多数游戏系统关心的汇总信息（以及等效的 [`MouseScroll`](https://docs.rs/bevy/0.15.0/bevy/input/mouse/struct.MouseScroll.html) 信息），你必须自己求和。

```rust
pub fn accumulate_mouse_motion_system(
    mut mouse_motion_event: EventReader<MouseMotion>,
    mut accumulated_mouse_motion: ResMut<AccumulatedMouseMotion>,
) {
    let mut delta = Vec2::ZERO;
    for event in mouse_motion_event.read() {
        delta += event.delta;
    }
    accumulated_mouse_motion.delta = delta;
}
```

Bevy 现在为你做了这项工作，通过新的 [`AccumulatedMouseMotion`](https://docs.rs/bevy/0.15.0/bevy/input/mouse/struct.AccumulatedMouseMotion.html) 和 [`AccumulatedMouseScroll`](https://docs.rs/bevy/0.15.0/bevy/input/mouse/struct.AccumulatedMouseScroll.html) 资源暴露。

## Stable Interpolation and Smooth Following（稳定插值和平滑跟随）[#](https://bevy.org/news/bevy-0-15/#stable-interpolation-and-smooth-following)

作者：[@mweatherley](https://github.com/mweatherley)

PR：[#13741](https://github.com/bevyengine/bevy/pull/13741)

当动画化相机或编写单位 AI（不是那种 AI！）时，持续向目标移动某个对象是一个基本的常用操作。简单地[线性插值](https://en.wikipedia.org/wiki/Linear_interpolation)到目标似乎足够简单，但正如 [Freya Holmer 解释的](https://www.youtube.com/watch?v=LSNQuFEDOyQ)，确保这种插值与时间步长无关既至关重要又异常棘手。

我们已经为你做了数学计算；你只需要使用 [`StableInterpolate`](https://docs.rs/bevy/0.15/bevy/math/trait.StableInterpolate.html) trait 的 `interpolate_stable` 和 `smooth_nudge` 方法，并调整 `decay_rate` 参数来真正优化你的*游戏手感*。不用担心：它甚至适用于四元数！稳定、平滑的相机控制器从未如此简单。

## What's Next?（下一步是什么？）[#](https://bevy.org/news/bevy-0-15/#what-s-next)

以上功能可能很棒，但 Bevy 还在开发什么呢？深入洞察时间的迷雾（当你的团队几乎全是志愿者时，预测*格外*困难！），我们可以看到一些令人兴奋的工作正在成形：

- **Bevy Scene Notation（Bevy 场景符号）：** Required Components 标志着 Cart 的[总体规划](https://github.com/bevyengine/bevy/discussions/14437)的第一步。在接下来的几个月里，他将埋头开发 Bevy 特定的文件格式（包括匹配的宏和 IDE 支持）、`Construct` trait（以便轻松地在场景中包含资源数据）、补丁（用于分层修改场景）以及尝试用于 UI 的响应式方法。
- **更好的字体支持：** 虽然 `cosmic_text` 在文本塑形和渲染方面是一个巨大的飞跃，但我们处理字体和字型的方法仍然相当粗糙。双向文本、系统字体支持、方便的 Markdown 风格"将文本的此部分加粗"API、字体回退等功能正在规划中。
- **基于 Picking 的 UI 交互：** `bevy_picking` 引入了一种更强大、更具表现力的方式来处理指针交互，但我们[没有在 `bevy_ui` 内部充分利用其全部能力](https://github.com/bevyengine/bevy/issues/15550)。虽然拾取事件很好，但对于响应式控件样式，"用户在这个按钮上做了什么"的单一真实来源至关重要。
- **`bevy_lint`：** 尽管我们努力，但*确实*可能误用 Bevy 的 API！作为更广泛的 [`bevy_cli`](https://github.com/theBevyFlock/bevy_cli) 项目的一部分，Bevy 社区开发了一个 Bevy 特定的 linter，用于捕获常见的错误或危险，并正在寻找早期采用者来试用！
- **焦点抽象：** 跟踪哪个 UI 元素拥有焦点对于屏幕阅读器、手柄和键盘用户能够舒适地导航 UI 至关重要。我们计划基于 `bevy_picking` 的成功，开发一个补充性的[焦点追踪解决方案](https://github.com/bevyengine/bevy/issues/15378)，以及一些简单的后端来选择加入基于键盘或手柄的 UI 导航。
- **不可变组件：** 组件 Hook 和观察者在响应变更和维护不变量方面非常强大，但通过简单地修改组件就可以轻松绕过。疯狂科学团队一直在[尝试](https://github.com/bevyengine/bevy/issues/16208)一种选择退出直接修改的方法，为更健壮的层次结构、复杂的观察者驱动的反应和第一方组件索引解决方案打开大门。
- **真正的持久化渲染：** 虽然渲染世界在 Bevy 0.15 中*技术上*是持久化的，但我们现有的大部分代码仍然每帧生成和反生成实体，以降低迁移过程中引入 Bug 的风险。我们期待逐步改变这一点并分析性能影响！
- **`no_std` Bevy：** 为了更好地支持奇怪的平台（如 [Playdate](https://play.date/)！），并使开发者在现代游戏主机上尝试 Bevy 时更轻松，我们一直在[致力于](https://github.com/bevyengine/bevy/issues/15460)确保 Bevy（的大部分）可以在没有 Rust 标准库的情况下编译和运行。
