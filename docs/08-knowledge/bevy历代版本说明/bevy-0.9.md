# Bevy 0.9

## 发布于 2022 年 11 月 12 日，作者：Carter Anderson ( ![一个猫耳人物挥舞触手的剪影，或称 Octocat：GitHub 的吉祥物和标志](https://bevy.org/assets/github_grey.svg) [@cart](https://www.github.com/cart) ![一个圆角矩形内指向右的三角形；Youtube 的标志](https://bevy.org/assets/youtube_grey.svg) [cartdev](https://www.youtube.com/cartdev) )

![代表本文的图片](https://bevy.org/news/bevy-0-9/bloom_lion.png)

感谢 **159** 位贡献者、**430** 个 pull request、社区审阅者，以及我们[慷慨的赞助者](https://github.com/sponsors/cart)，我很高兴地宣布 **Bevy 0.9** 已在 [crates.io](https://crates.io/crates/bevy) 上发布！

如果你还不了解 Bevy，它是一款基于 Rust 构建的、简洁的数据驱动游戏引擎。你可以查看我们的[快速入门指南](https://bevy.org/learn/quick-start/introduction)来立即试用。它永久免费且开源！你可以在 GitHub 上获取完整的[源代码](https://github.com/bevyengine/bevy)。查看 [Bevy Assets](https://bevy.org/assets) 获取社区开发的插件、游戏和学习资源集合。

要将现有的 Bevy App 或 Plugin 更新到 **Bevy 0.9**，请查看我们的 [0.8 到 0.9 迁移指南](https://bevy.org/learn/migration-guides/0.8-0.9/)。

自几个月前上次发布以来，我们添加了_大量_新功能、bug 修复和生活质量改进，但以下是一些亮点：

- **HDR 后处理、色调映射和泛光（Bloom）**：Bevy 有了新的 HDR 后处理和色调映射管线，我们用它实现了"泛光"后处理效果！
- **FXAA**：添加了快速近似抗锯齿，为用户提供了一种廉价的屏幕空间抗锯齿选项。
- **去色带抖动（Deband Dithering）**：使用这个新的后处理效果隐藏渐变精度误差！
- **其他后处理改进**：视图目标双缓冲和自动渲染目标格式处理。
- **新场景格式**：Bevy 的新场景格式更小、更容易手动组合，也更容易阅读。提供"人类可读"和"二进制"两种变体！
- **代码驱动场景构建**：使用查询和特定实体引用从现有应用动态构建场景。
- **改进的实体/组件 API**：生成带有组件的实体现在比以往更简单、更符合人体工学！
- **独占系统重写**：独占系统（具有 ECS World 独占可变访问权的系统）现在就是"普通"系统，可用性显著提高。
- **枚举反射**：Bevy Reflect 现在可以反射枚举类型，将其暴露给 Bevy 的场景系统，并为枚举的编辑器工具打开了大门。
- **时间着色器全局变量**：时间现在作为全局变量传递给着色器，使自定义着色器中的时间驱动动画变得简单！
- **插件设置**：插件现在可以有设置，可以在插件组中覆盖，简化了插件配置。
- **Bevy UI Z-Indices**：使用局部和全局 z-index 控制 UI 元素如何相互叠加

## HDR 后处理、色调映射和泛光 [#](https://bevy.org/news/bevy-0-9/#hdr-post-processing-tonemapping-and-bloom)

作者：@ChangeCaps、@jakobhellermann、@cart、@JMS55

Bevy 现在支持"泛光"后处理效果，这得益于我们 HDR（高动态范围）渲染管线的大量内部改进。

![bloom](https://bevy.org/news/bevy-0-9/bloom.png)

泛光在明亮光源周围创建"模糊"效果，模拟了相机（和我们的眼睛）在现实世界中感知光线的方式。高质量泛光建立在 HDR 渲染管线之上，它使用比其他地方使用的标准每通道 8 位（rgba）更多的位数来表示光线和颜色。在之前的版本中，Bevy 已经在 PBR 着色器中[内部进行了 HDR 光照](https://bevy.org/news/bevy-0-5/#physically-based-rendering-pbr)，但因为我们渲染到"普通"（低动态范围）纹理，当我们将 HDR 光照映射到 LDR 纹理（使用称为色调映射的过程）时，我们必须丢失额外的高动态范围信息。

在 **Bevy 0.9** 中，你现在可以将相机配置为渲染到 HDR 纹理，这将在"主通道"渲染完成后保留高动态范围信息：

```rust
Camera {
    // 目前默认为 false，但我们可能会
    // 在未来的版本中将其切换为 true
    hdr: true,
    ..default()
}
```

这使得泛光等后处理效果可以访问原始 HDR 信息。当启用 HDR 纹理时，我们将"色调映射"延迟到"HDR 后处理效果"在我们的 [Render Graph](https://bevy.org/news/bevy-0-6/#render-graphs-and-sub-graphs) 中运行之后。

通过向启用了 HDR 纹理的相机添加 [`BloomSettings`](https://docs.rs/bevy/0.9.0/bevy/core_pipeline/bloom/struct.BloomSettings.html) 组件来启用泛光：

```rust
commands.spawn((
    Camera3dBundle {
        camera: Camera {
            hdr: true,
            ..default()
        },
        ..default()
    },
    BloomSettings::default(),
));
```

如果配置不当，泛光效果可能会过于强烈。[`BloomSettings`](https://docs.rs/bevy/0.9.0/bevy/core_pipeline/bloom/struct.BloomSettings.html) 有许多选项可以调整，但最相关的是 `intensity`，它可以用（也应该用）来调整效果的应用程度。

说真的……这个效果可能会很烦人：

![太多 bloom](https://bevy.org/news/bevy-0-9/too_much_bloom.png)

在大多数情况下，最好偏向于微妙。

HDR 渲染也适用于 2D，这意味着你也可以在 2D 中使用泛光效果！

![2D bloom](https://bevy.org/news/bevy-0-9/2d_bloom.png)

## FXAA：快速近似抗锯齿 [#](https://bevy.org/news/bevy-0-9/#fxaa-fast-approximate-anti-aliasing)

作者：@DGriffin91、@cart

**Bevy 0.9** 添加了 FXAA（快速近似抗锯齿）支持。FXAA 是一种流行（且廉价！）的抗锯齿方法，它使用亮度数据对比度来识别边缘并对其进行模糊：

![no_aa](https://bevy.org/news/bevy-0-9/no_aa.png) ![fxaa](https://bevy.org/news/bevy-0-9/fxaa.png)

Bevy 已经支持 MSAA（多重采样抗锯齿），它在渲染几何边缘时进行多次采样，使这些边缘更清晰：

![msaa](https://bevy.org/news/bevy-0-9/msaa.png)

选择抗锯齿实现全是关于权衡：

- **MSAA**：清晰、高质量的几何边缘。保持图像的其他部分（如纹理和阴影）不变，这可以是优点（更清晰的输出）也可以是缺点（更多锯齿）。比 FXAA 更昂贵。
- **FXAA**：在模糊时考虑整个图像，包括纹理，这可以是优点（纹理和阴影获得抗锯齿）也可以是缺点（整个图像变得更模糊）。运行开销低（移动或网页抗锯齿的好选择）。

现在我们的后处理管线正在成熟，我们计划在未来的 Bevy 版本中添加更多抗锯齿选项。我们已经在开发 TAA（时间抗锯齿）和 SMAA（亚像素形态抗锯齿）实现！

## 去色带抖动 [#](https://bevy.org/news/bevy-0-9/#deband-dithering)

作者：@aevyrie

"色带"是使用 8 位颜色通道（几乎所有设备/屏幕都需要）时的已知限制。

当试图为低噪声纹理渲染平滑渐变时，这一点最为明显（例如："纯绿色"材质上的光照）：

![banding](https://bevy.org/news/bevy-0-9/banding.png)

如果你仔细观察绿色平面_或_棕褐色立方体，你会注意到每种颜色深浅都有明显的色带。这个问题的一个流行解决方案是对最终图像进行"抖动"。

**Bevy 0.9** 现在在色调映射阶段默认执行"去色带抖动"：

![debanding](https://bevy.org/news/bevy-0-9/debanding.png)

你可以按相机启用和禁用此功能：

```rust
  commands.spawn(Camera3dBundle {
      tonemapping: Tonemapping::Enabled {
          deband_dither: true,
      },
      ..default()
  });
```

## 后处理：视图目标双缓冲 [#](https://bevy.org/news/bevy-0-9/#post-processing-view-target-double-buffering)

作者：@cart

渲染后处理效果需要一个输入纹理（包含"当前"渲染）和一个输出纹理（应用了效果的"新"渲染）。之前版本的 Bevy 只有一个主"视图目标"图像。这意味着朴素地，后处理效果需要管理并渲染到它们自己的"中间"纹理，然后将其_写回_主目标。这显然是低效的，因为每个效果都有一个新的纹理分配_以及_将中间纹理复制回主纹理的额外工作。

为了解决这个问题，在 **Bevy 0.9** 中，我们现在对视图目标纹理进行"双缓冲"，这意味着我们有两个副本在它们之间翻转。在给定时刻，一个是当前的"主"纹理，另一个是"下一个"主纹理。后处理效果开发者现在可以触发"后处理写入"，它返回一个 `source` 和 `destination` 纹理。它假设一个效果将 `source` 写入 `destination`（带或不带修改）。`destination` 然后将成为新的"主"纹理。

```rust
let post_process = view_target.post_process_write();
render_some_effect(render_context, post_process.source, post_process.destination);
```

这减少了后处理效果开发者的复杂性负担，并保持了我们管线的高效。新的 [FXAA 效果](https://bevy.org/news/bevy-0-9/#fxaa-fast-approximate-anti-aliasing)就是使用这个新系统实现的。后处理插件开发者可以参考该实现。

## 改进的渲染目标纹理格式处理 [#](https://bevy.org/news/bevy-0-9/#improved-render-target-texture-format-handling)

作者：@VitalyAnkh、@cart

**Bevy 0.9** 现在检测并使用每个窗口/表面首选的 [`TextureFormat`](https://docs.rs/bevy/0.9.0/bevy/render/render_resource/enum.TextureFormat.html)，而不是使用硬编码的编译时选择的每平台格式。这意味着我们自动支持不常见的平台和配置。此外，Bevy 的主通道和后处理通道现在渲染到稳定的/一致的 [`TextureFormats`](https://docs.rs/bevy/0.9.0/bevy/render/render_resource/enum.TextureFormat.html)（例如：HDR 为 `Rgba16Float`）。我们从这些"标准"纹理到最后的渲染目标首选格式进行最终 blit。这简化了渲染管线构建，允许跨渲染目标复用渲染管线（即使它们的格式不匹配），并提供一致且可预测的渲染管线行为。

这也意味着当渲染到纹理时，纹理格式不再需要与表面的纹理格式匹配。例如，你现在可以渲染到一个只有红色通道的纹理：

![渲染到红色纹理](https://bevy.org/news/bevy-0-9/render_to_texture_red.png)

## 新场景格式 [#](https://bevy.org/news/bevy-0-9/#new-scene-format)

作者：@MrGVSV

**Bevy 0.9** 引入了一个_大大改进_的场景格式，使场景更小、更容易手动组合，也更容易阅读。这得益于 Bevy Reflect（Bevy 的 Rust 运行时反射系统）的大量改进。对 Bevy 场景格式的大部分改进实际上是对所有 Bevy Reflect 序列化的通用改进！

```rust
// 新的 Bevy 场景格式
(
  entities: {
    0: (
      components: {
        "game::Player": (
          name: "Reyna",
          position: (
            x: 0.0,
            y: 0.0,
          ),
        ),
        "game::Health": (
          current: 5,
          max: 10,
        ),
        "game::Team": A,
      },
    ),
    1: (
      components: {
        "game::Player": (
          name: "Sova",
          position: (
            x: 10.0,
            y: 0.0,
          ),
        ),
        "game::Health": (
          current: 10,
          max: 10,
        ),
        "game::Team": B,
      },
    ),
  },
)
```

与旧格式对比：

```rust
// 旧的 Bevy 场景格式
[
  (
    entity: 0,
    components: [
      {
        "type": "game::Player",
        "struct": {
          "name": {
            "type": "alloc::string::String",
            "value": "Reyna",
          },
          "position": {
            "type": "glam::f32::vec2::Vec2",
            "struct": {
              "x": {
                "type": "f32",
                "value": 0.0,
              },
              "y": {
                "type": "f32",
                "value": 0.0,
              },
            },
          },
        },
      },
      {
        "type": "game::Health",
        "struct": {
          "current": {
            "type": "usize",
            "value": 5,
          },
          "max": {
            "type": "usize",
            "value": 10,
          },
        },
      },
      {
        "type": "game::Team",
        "value": A,
      },
    ],
  ),
  (
    entity: 1,
    components: [
      {
        "type": "game::Player",
        "struct": {
          "name": {
            "type": "alloc::string::String",
            "value": "Sova",
          },
          "position": {
            "type": "glam::f32::vec2::Vec2",
            "struct": {
              "x": {
                "type": "f32",
                "value": 10.0,
              },
              "y": {
                "type": "f32",
                "value": 0.0,
              },
            },
          },
        },
      },
      {
        "type": "game::Health",
        "struct": {
          "current": {
            "type": "usize",
            "value": 10,
          },
          "max": {
            "type": "usize",
            "value": 10,
          },
        },
      },
      {
        "type": "game::Team",
        "value": B,
      },
    ],
  ),
]
```

改进太多了，可能很难全部挑出来！

### 更简单的结构体语法 [#](https://bevy.org/news/bevy-0-9/#simpler-struct-syntax)

结构体现在使用结构体风格的格式化，而不是复杂的基于映射的表示。

```rust
// 旧
{
    "type": "game::Health",
    "struct": {
        "current": {
            "type": "usize",
            "value": 5,
        },
        "max": {
            "type": "usize",
            "value": 10,
        },
    },
},

// 新
"game::Health": (
    current: 5,
    max: 10,
),
```

### 更简单的原始类型序列化 [#](https://bevy.org/news/bevy-0-9/#simpler-primitive-serialization)

类型现在可以选择加入直接 serde 序列化，这使得原始值的使用变得更加友好：

```rust
// 旧
"name": {
    "type": "alloc::string::String",
    "value": "Reyna",
},

// 新
name: "Reyna",
```

### 更好的枚举语法 [#](https://bevy.org/news/bevy-0-9/#nicer-enum-syntax)

考虑枚举：

```rust
pub enum Team {
    A,
    B,
}
```

让我们比较一下它的序列化方式：

```rust
// 旧
{
    "type": "game::Team",
    "value": A,
},

// 新
"game::Team": A,
```

另外请注意，Bevy Reflect 在 **Bevy 0.9** 之前甚至不直接支持枚举。旧版本的 Bevy 需要将 `#[reflect_value]` 与正常的 serde 结合使用来处理枚举，这要复杂得多。详情请参阅本文的[枚举反射](https://bevy.org/news/bevy-0-9/#enum-reflection)部分！

### 更好的元组 [#](https://bevy.org/news/bevy-0-9/#nicer-tuples)

```rust
// 旧
{
  "type": "(f32, f32)",
  "tuple": [
    {
      "type": "f32",
      "value": 1.0
    },
    {
      "type": "f32",
      "value": 2.0
    }
  ]
}

// 新
{
  "(f32, f32)": (1.0, 2.0)
}
```

### 顶级结构体 [#](https://bevy.org/news/bevy-0-9/#top-level-struct)

Bevy 场景现在有一个顶级结构体，这允许我们在未来向 Bevy 场景格式添加额外的值和元数据（如版本号、ECS 资源、资产等）。

```rust
// 旧
[
    /* 这里放实体 */
]

// 新
(
    entities: (
        /* 这里放实体 */
    )
)
```

### 适当时使用映射 [#](https://bevy.org/news/bevy-0-9/#use-maps-where-appropriate)

实体 ID 和组件值在 Bevy ECS 中必须是唯一的。为了更好地表示这一点，我们现在使用映射语法而不是列表。

```rust
// 旧
[
  (
    entity: 0,
    components: [ ],
  ),
  (
    entity: 1,
    components: [ ],
  ),
]

// 新
(
  entities: {
    0: (
      components: { },
    ),
    1: (
      components: { },
    ),
  },
)
```

## 二进制场景格式 [#](https://bevy.org/news/bevy-0-9/#binary-scene-formats)

作者：@MrGVSV

Bevy 场景可以序列化和反序列化为/从二进制格式，如 [`bincode`](https://crates.io/crates/bincode/2.0.0-rc.1)、[`postcard`](https://crates.io/crates/postcard) 和 [`rmp_serde`](https://crates.io/crates/rmp-serde)。这需要为新场景格式添加对"非自描述"格式的支持。

对于 postcard，这可以小近 5 倍（上述场景为 4.53 倍）！如果你想保持场景文件大小较小，或者通过网络发送场景，这非常有用。

## 动态场景构建器 [#](https://bevy.org/news/bevy-0-9/#dynamic-scene-builder)

作者：@mockersf

Bevy 场景现在可以使用新的 [`DynamicSceneBuilder`](https://docs.rs/bevy/0.9.0/bevy/scene/struct.DynamicSceneBuilder.html) 动态构建。之前版本的 Bevy 已经支持[将"整个世界"写入场景](https://github.com/bevyengine/bevy/blob/v0.8.1/examples/scene/scene.rs#L78)，但在某些情况下，用户可能只想将_特定_实体写入场景。**Bevy 0.9** 的 [`DynamicSceneBuilder`](https://docs.rs/bevy/0.9.0/bevy/scene/struct.DynamicSceneBuilder.html) 使这成为可能：

```rust
// 将玩家写入场景
fn system(world: &World, players: Query<Entity, With<Player>>) {
  let builder = DynamicSceneBuilder::from_world(world);
  builder.extract_entities(players.iter());
  let dynamic_scene = builder.build();
}
```

`extract_entities` 接受任何 `Entity` 迭代器。

你也可以传入特定实体：

```rust
builder.extract_entity(entity);
```

## 更多场景构建工具 [#](https://bevy.org/news/bevy-0-9/#more-scene-construction-tools)

作者：@mockersf

[`Scenes`](https://docs.rs/bevy/0.9.0/bevy/scene/struct.Scene.html) 现在可以克隆：

```rust
let scene = scene.clone_with(type_registry).unwrap();
```

[`DynamicScenes`](https://docs.rs/bevy/0.9.0/bevy/scene/struct.DynamicScene.html) 现在可以转换为 [`Scenes`](https://docs.rs/bevy/0.9.0/bevy/scene/struct.Scene.html)：

```rust
let scene = Scene::from_dynamic_scene(dynamic_scene, type_registry).unwrap();
```

## 改进的实体/组件 API [#](https://bevy.org/news/bevy-0-9/#improved-entity-component-apis)

作者：@DJMcNab、@cart

生成带有组件的实体以及向实体添加/移除组件变得更加简单！

首先介绍一些基础知识：Bevy ECS 使用 [`Components`](https://docs.rs/bevy/0.9.0/bevy/ecs/component/trait.Component.html) 向实体添加数据和逻辑。为了使实体组合更容易，Bevy ECS 还有 [`Bundles`](https://docs.rs/bevy/0.9.0/bevy/ecs/bundle/trait.Bundle.html)，它定义了要一起添加的组件组。

就像之前的 Bevy 版本一样，Bundles 可以是组件的元组：

```rust
(Player { name: "Sova" }, Health::new(10), Team::A)
```

[`Bundle`](https://docs.rs/bevy/0.9.0/bevy/ecs/bundle/trait.Bundle.html) trait 也可以被派生：

```rust
#[derive(Bundle)]
struct PlayerBundle {
  player: Player,
  health: Health,
  team: Team,
}
```

在 **Bevy 0.9** 中，[`Component`](https://docs.rs/bevy/0.9.0/bevy/ecs/component/trait.Component.html) 类型现在_也_自动实现了 [`Bundle`](https://docs.rs/bevy/0.9.0/bevy/ecs/bundle/trait.Bundle.html) trait，这允许我们将所有实体组件操作整合到新的 `spawn`、`insert` 和 `remove` API 下。以前，我们有 [`Bundle`](https://docs.rs/bevy/0.9.0/bevy/ecs/bundle/trait.Bundle.html) 的单独变体（例如：`insert_bundle(SomeBundle)`）和 [`Component`](https://docs.rs/bevy/0.9.0/bevy/ecs/component/trait.Component.html) 的单独变体（例如：`.insert(SomeComponent)`）。

[`Bundle`](https://docs.rs/bevy/0.9.0/bevy/ecs/bundle/trait.Bundle.html) trait 现在也为 [`Bundles`](https://docs.rs/bevy/0.9.0/bevy/ecs/bundle/trait.Bundle.html) 的元组实现了，而不仅仅是 [`Components`](https://docs.rs/bevy/0.9.0/bevy/ecs/component/trait.Component.html) 的元组。这一点的价值稍后会变得清晰。

首先，`spawn` 现在接受一个 bundle：

```rust
// 旧（变体 1）
commands.spawn().insert_bundle(SpriteBundle::default());

// 旧（变体 2）
commands.spawn_bundle(SpriteBundle::default());

// 新
commands.spawn(SpriteBundle::default());
```

我们已经节省了一些字符，但我们才刚刚开始！因为 [`Component`](https://docs.rs/bevy/0.9.0/bevy/ecs/component/trait.Component.html) 实现了 [`Bundle`](https://docs.rs/bevy/0.9.0/bevy/ecs/bundle/trait.Bundle.html)，我们现在也可以将单个组件传入 `spawn`：

```rust
// 旧
commands.spawn().insert(Player { name: "Sova" });

// 新
commands.spawn(Player { name: "Sova" });
```

当我们引入 [`Bundle`](https://docs.rs/bevy/0.9.0/bevy/ecs/bundle/trait.Bundle.html) 元组时，事情变得更加有趣，这允许我们将许多操作（涵盖组件和 bundles）组合到一个 `spawn` 调用中：

```rust
// 旧
commands
  .spawn_bundle(PlayerBundle::default())
  .insert_bundle(TransformBundle::default())
  .insert(ActivePlayer);

// 新
commands.spawn((
  PlayerBundle::default(),
  TransformBundle::default(),
  ActivePlayer,
));
```

这_更_容易输入和阅读。而且最重要的是，从 Bevy ECS 的角度来看，这是一个单一的"bundle 生成"而不是多个操作，这减少了["原型移动"](https://bevy.org/news/bevy-0-5/#component-storage-the-problem)。这使得这个单一的生成操作更加高效！

这些原则也适用于 insert API：

```rust
// 旧
commands
  .insert_bundle(PlayerBundle::default())
  .insert(ActivePlayer);

// 新
commands.insert((PlayerBundle::default(), ActivePlayer));
```

它们也适用于 remove API：

```rust
// 旧
commands
  .remove_bundle::<PlayerBundle>()
  .remove::<ActivePlayer>();

// 新
commands.remove::<(PlayerBundle, ActivePlayer)>();
```

## 独占系统重写 [#](https://bevy.org/news/bevy-0-9/#exclusive-system-rework)

作者：@cart、@maniwani

为了准备新合并（但尚未实现）的 [Stageless RFC](https://github.com/bevyengine/rfcs/pull/45) 中概述的更大调度器变更，我们已经开始模糊"独占系统"（具有 ECS [`World`](https://docs.rs/bevy/0.9.0/bevy/ecs/world/struct.World.html) "独占"完全可变访问权的系统）和普通系统之间的界限，历史上它们是具有严格分隔的不同类型。

在 **Bevy 0.9** 中，独占系统现在实现了普通的 [`System`](https://docs.rs/bevy/0.9.0/bevy/ecs/system/trait.System.html) trait！这最终会有更大的影响，但在 **Bevy 0.9** 中，这意味着你不再需要在将独占系统添加到调度器时调用 `.exclusive_system()`：

```rust
fn some_exclusive_system(world: &mut World) { }

// 旧
app.add_system(some_exclusive_system.exclusive_system())

// 新
app.add_system(some_exclusive_system)
```

我们还扩展了独占系统以支持更多系统参数，这大大改善了编写独占系统的用户体验，并通过跨执行缓存状态使其更高效。

[`SystemState`](https://docs.rs/bevy/0.9.0/bevy/ecs/system/struct.SystemState.html) 允许在独占系统内部使用"普通"系统参数：

```rust
// 旧
fn some_system(world: &mut World) {
  let mut state: SystemState<(Res<Time>, Query<&mut Transform>)> =
      SystemState::new(&mut world);
  let (time, mut transforms) = state.get_mut(world);
}

// 新
fn some_system(world: &mut World, state: &mut SystemState<(Res<Time>, Query<&mut Transform>)>) {
  let (time, mut transforms) = state.get_mut(world);
}
```

[`QueryState`](https://docs.rs/bevy/0.9.0/bevy/ecs/query/struct.QueryState.html) 启用对单个查询的缓存访问：

```rust
// 旧
fn some_system(world: &mut World) {
  let mut transforms = world.query::<&Transform>();
  for transform in transforms.iter(world) {
  }
}

// 新
fn some_system(world: &mut World, transforms: &mut QueryState<&Transform>) {
  for transform in transforms.iter(world) {
  }
}
```

[`Local`](https://docs.rs/bevy/0.9.0/bevy/ecs/system/struct.Local.html) 允许在独占系统内部存储本地数据：

```rust
// 旧
#[derive(Resource)]
struct Counter(usize);
fn some_system(world: &mut World) {
  let mut counter = world.resource_mut::<Counter>();
  counter.0 += 1;
}

// 新
fn some_system(world: &mut World, mut counter: Local<usize>) {
  *counter += 1;
}
```

## Bevy ECS 现在使用 GATS！[#](https://bevy.org/news/bevy-0-9/#bevy-ecs-now-uses-gats)

作者：@BoxyUwU

Rust 1.65.0 [稳定化了 GATs](https://blog.rust-lang.org/2022/11/03/Rust-1.65.0.html)（泛型关联类型），这使我们能够显著简化 Bevy ECS 查询内部实现。

一直以来，Bevy ECS 一直在用复杂的 trait 嵌套（`WorldQuery`、`WorldQueryGats`（一个"缺乏真正的 GATs 的 hack"trait）和 `Fetch`）来绕过缺乏 GATs 的问题。

在 **Bevy 0.9** 中，我们现在只有一个 [`WorldQuery`](https://docs.rs/bevy/0.9.0/bevy/ecs/query/trait.WorldQuery.html) trait！这使得 Bevy ECS 更容易维护、扩展、调试、记录和理解。

## 派生 Resource [#](https://bevy.org/news/bevy-0-9/#derive-resource)

作者：@devil-ira、@alice-i-cecile

[`Resource`](https://docs.rs/bevy/0.9.0/bevy/ecs/system/trait.Resource.html) trait 现在不再自动为所有类型实现。它必须被派生：

```rust
#[derive(Resource)]
struct Counter(usize);
```

这个变更紧随为 [`Component`](https://docs.rs/bevy/0.9.0/bevy/ecs/component/trait.Component.html) 类型做出[相同决定](https://bevy.org/news/bevy-0-6/#the-new-component-trait-and-derive-component)之后。简而言之：

1. 为每个类型自动实现 [`Resource`](https://docs.rs/bevy/0.9.0/bevy/ecs/system/trait.Resource.html) 使得很容易意外插入"错误"的值，例如插入构造函数指针而不是值本身：
    
    ```rust
    struct Counter(usize);
    // 这会将构造函数指针作为资源插入！
    // 很奇怪且令人困惑！
    app.insert_resource(Counter);
    // 这才是正确的做法！
    app.insert_resource(Counter(0));
    ```
    
2. 派生 [`Resource`](https://docs.rs/bevy/0.9.0/bevy/ecs/system/trait.Resource.html) 以结构化的方式记录意图。没有派生的话，resource-ness 默认是隐式的。
    
3. 自动实现意味着插件可以以冲突的方式使用相同的"通用"类型（例如：`std` 类型如 `Vec<usize>`）。默认不实现意味着插件不能以冲突的方式使用这些通用类型。它们必须创建新类型。
    
4. 这为使用 Rust 类型系统配置资源类型打开了大门（就像我们已经为组件所做的那样）。

## 系统歧义解析 API 改进 [#](https://bevy.org/news/bevy-0-9/#system-ambiguity-resolution-api-improvements)

作者：@JoJoJet、@alice-i-cecile

Bevy ECS 默认调度系统并行运行。它会安全地调度系统并行运行，尊重系统之间的依赖关系并强制执行 Rust 的可变性规则。默认情况下，这意味着如果系统 A 读取一个资源而系统 B 写入该资源（且它们之间没有定义排序），那么系统 A 可能在系统 B_之前_或_之后_执行。我们称这些系统为"有歧义的"。在某些情况下这种歧义可能很重要，在其他情况下可能不重要。

Bevy 已经有一个[系统歧义检测系统](https://bevy.org/news/bevy-0-5/#ambiguity-detection-and-resolution)，它使用户能够检测有歧义的系统并解决歧义（通过添加排序约束或忽略歧义）。用户可以将系统添加到"歧义集"中以忽略这些集中系统之间的歧义：

```rust
#[derive(AmbiguitySet)]
struct AmbiguousSystems;

app
  .add_system(a.in_ambiguity_set(AmbiguousSystems))
  .add_system(b.in_ambiguity_set(AmbiguousSystems))
```

这有点难以理解，并引入了比必要更多的样板代码。

在 **Bevy 0.9** 中，我们用更简单的 `ambiguous_with` 调用替换了歧义集：

```rust
app
  .add_system(a)
  .add_system(b.ambiguous_with(a))
```

这建立在现有的 [`SystemLabel`] 方法之上，这意味着你也可以使用标签来实现"集式"歧义解析：

```rust
#[derive(SystemLabel)]
struct Foo;

app
  .add_system(a.label(Foo))
  .add_system(b.label(Foo))
  .add_system(b.ambiguous_with(Foo))
```

## Bevy ECS 优化 [#](https://bevy.org/news/bevy-0-9/#bevy-ecs-optimizations)

作者：@james7132、@JoJoJet

由于 `@james7132`，我们在 **Bevy 0.9** 中取得了一些_巨大_的性能提升：

- Query fetch 抽象被[重新设计](https://github.com/bevyengine/bevy/pull/4800)，将公共部分提升到单次迭代之外，在某些基准测试中将迭代器性能提高了约 10-20%。`Query::get` 性能也有一些改进。
- 从我们的数据访问 API 中[移除了一些不必要的分支](https://github.com/bevyengine/bevy/pull/6461)，在大多数 ECS 基准测试中将性能提高了约 5-20%！
- 并行执行器[现在在 `prepare_systems` 步骤运行时就开始运行系统](https://github.com/bevyengine/bevy/pull/4919)，当有许多工作量很小的系统时，减少了大量延迟。这从我们的 `many_foxes` 动画基准测试中减少了近 1 毫秒（约 12% 的改进）。这是_非常_重要的事情！
- 迭代器现在在迭代查询时[跳过空原型和表](https://github.com/bevyengine/bevy/pull/4724)，当原型为空时，这显著减少了每个原型的迭代开销。

`@JoJoJet` [还优化](https://github.com/bevyengine/bevy/pull/6400)了 `Query::get_many` 访问，用循环替换了 `array::map`，将 `get_many` 优化了约 20-30%！

## ECS 变更检测绕过 [#](https://bevy.org/news/bevy-0-9/#ecs-change-detection-bypass)

作者：@alice-i-cecile

得益于一些非常巧妙的 Rust 用法，Bevy ECS 可以自动检测组件和资源的变更。

然而有时，用户可能做出不希望被检测到的变更。在 **Bevy 0.9** 中，变更检测现在可以被绕过：

```rust
fn system(mut transforms: Query<&mut Transform>) {
  for transform in &mut transforms {
    transform.bypass_change_detection().translation.x = 1.0;
  }
}
```

## 枚举反射 [#](https://bevy.org/news/bevy-0-9/#enum-reflection)

作者：@MrGVSV、@Davier、@nicopap

Bevy Reflect 现在原生支持 Rust 枚举！Bevy Reflect 是 Bevy 的"Rust 反射系统"，它允许我们在运行时动态地访问关于值和类型的 Rust 类型信息。

在过去的 Bevy 版本中，我们需要通过将枚举类型视为"反射值"来绕过 Bevy Reflect 缺乏枚举支持的问题，这需要每个类型更多的工作，并且提供的类型反射信息更少：

```rust
// 旧
#[derive(Copy, Clone, PartialEq, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect_value(PartialEq, Serialize, Deserialize)]
enum SomeEnum {
  A,
  B(usize),
  C {
    foo: f32,
    bar: bool,
  },
}

// 新
#[derive(Reflect)]
enum SomeEnum {
  A,
  B(usize),
  C {
    foo: f32,
    bar: bool,
  },
}
```

不再需要魔法咒语了！

与其他反射类型一样，枚举反射提供了许多新的运行时功能：

```rust
// 访问变体名称
let value = SomeEnum::A;
assert_eq!("A", value.variant_name());

// 获取变体类型
match value.variant_type() {
  VariantType::Unit => {},
  VariantType::Struct => {},
  VariantType::Tuple => {},
}

let mut value = SomeEnum::C {
  foo: 1.23,
  bar: false
};

// 按名称读写特定字段
*value.field_mut("bar").unwrap() = true;

// 迭代整个字段集合
for field in value.iter_fields() {
}

// 检测值类型并检索有关它的信息
if let TypeInfo::Enum(info) = value.type_info() {
  if let VariantInfo::Struct(struct_info) = value.variant("C") {
    let first_field = struct_info.field_at(0).unwrap();
    assert_eq!(first_field.name(), "foo");
  }
}
```

在枚举上派生 [`Reflect`](https://docs.rs/bevy/0.9.0/bevy/reflect/trait.Reflect.html) 也会自动添加对"基于反射的序列化"的支持，在 **Bevy 0.9** 中，现在[有了_好得多_的语法](https://bevy.org/news/bevy-0-9/#new-scene-format)。

## 其他 Bevy Reflect 改进 [#](https://bevy.org/news/bevy-0-9/#other-bevy-reflect-improvements)

作者：@MrGVSV、@makspll、@Shatur、@themasch、@NathanSWard

我们对 Bevy Reflect 做了很多其他改进！

"容器"Reflect traits（Map、List、Array、Tuple）现在可以被 drain 以获取拥有所有权的值：

```rust
let container: Box<dyn List> = Box::new(vec![1.0, 2.0]);
let values: Vec<Box<dyn Reflect>> = container.drain();
```

反射字段现在可以选择退出序列化而不_同时_退出整个反射：

```rust
#[derive(Reflect)]
struct Foo {
  a: i32,
  // 对反射完全不可见，包括序列化
  #[reflect(ignore)]
  b: i32,
  // 仍然可以被反射，但序列化时会被跳过
  #[reflect(skip_serializing)]
  c: i32,
}
```

装箱的"反射类型"traits（Struct、Enum、List 等）现在可以转换为更通用的 `Box<dyn Reflect>`：

```rust
let list: Box<dyn List> = Box::new(vec![1.0, 2.0]);
let reflect: Box<dyn Reflect> = list.into_reflect();
```

现在可以获取反射类型的拥有所有权变体：

```rust
let value: Box<Sprite> = Box::new(Sprite::default());
if let ReflectOwned::Struct(owned) = value.reflect_owned() {
  // owned 是 Box<dyn Struct>
}
```

"反射路径 API"中的数组现在可以使用列表语法：

```rust
#[derive(Reflect)]
struct Foo {
    bar: [u8; 3],
}

let foo = Foo {
  bar: [10, 20, 30],
};

assert_eq!(*foo.get_path("bar[1]").unwrap(), 20);
```

反射列表现在有 pop 操作：

```rust
let mut list: Box<dyn List> = Box::new(vec![1u8, 2u8]);
let value: Box<dyn Reflect> = list.pop().unwrap();
assert_eq!(*value.downcast::<u8>().unwrap(), 2u8);
```

## 示例：手柄查看器 [#](https://bevy.org/news/bevy-0-9/#example-gamepad-viewer)

作者：@rparrett

Bevy 现在有一个手柄输入查看器应用，可以使用 `cargo run --example gamepad_viewer` 从 Bevy 仓库运行。

## 轴和按钮设置验证 [#](https://bevy.org/news/bevy-0-9/#axis-and-button-settings-validation)

作者：@mfdorst、@targrub

[`InputAxis`] 和 [`ButtonSettings`] 现在使用 getter 和 setter 来确保设置的完整性。setter 将返回错误而不是允许无效状态。

例如，尝试将按钮的"按下阈值"设置为低于"释放阈值"的值将导致错误：

```rust
button_settings.set_release_threshold(0.65);
// 这太低了！
assert!(button_settings.try_set_press_threshold(0.6).is_err())
```

## ScanCode 输入资源 [#](https://bevy.org/news/bevy-0-9/#scancode-input-resource)

作者：@Bleb1k

**Bevy 0.9** 添加了 `Input<ScanCode>` 资源，它的行为类似于 `Input<KeyCode>`，但忽略键盘布局：

```rust
fn system(scan_code: Res<Input<ScanCode>>, key_code: Res<Input<KeyCode>>) {
  // 33 是物理键盘上 F 键的扫描码
  if scan_code.pressed(ScanCode(33)) {
    log!("物理 F 键在键盘上被按下");
  }
  
  if keycode.pressed(KeyCode::F) {
    log!("逻辑 F 键在键盘上被按下，考虑布局。");
  }
}
```

## 时间着色器全局变量 [#](https://bevy.org/news/bevy-0-9/#time-shader-globals)

作者：@IceSentry

Bevy 着色器_终于_可以访问内置的时间值了，消除了用户计算时间值并手动传递的需要。时间在着色器中非常有用，因为它为动画化值打开了大门。

这是一个使用时间在黑色和红色之间进行动画的简单着色器：

```rust
@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    return vec4<f32>(sin(globals.time * 10.0), 0.0, 0.0, 1.0);
}
```

Bevy 着色器现在可以访问以下全局变量：

- `time`：自启动以来的时间（秒），1 小时后回绕为 0
- `delta_time`：自上一帧以来的时间（秒）
- `frame_count`：自应用启动以来的帧计数，达到 `u32` 最大值后回绕为 0

## 高实体渲染器慢化优化 [#](https://bevy.org/news/bevy-0-9/#high-entity-renderer-slowdown-optimization)

作者：@TheRawMeatball

[Bevy 的渲染器](https://bevy.org/news/bevy-0-6/#the-new-bevy-renderer)在"主世界"和"渲染世界"之间同步实体状态，这实现了并行流水线渲染。为了实现这一点，我们每帧清除渲染实体以确保提取状态的完整性。

然而，很明显我们每帧清除实体的方法在实体数量非常大时会产生显著的每实体开销。

在 **Bevy 0.9** 中，我们显著优化了实体清除，将清除 5,000,000 个实体的成本从约 360 微秒降低到约 120 微秒。我们还在考虑一个"保留状态"提取模型，利用 Bevy ECS 内置的变更检测，这将完全消除清除实体的需求（并更一般地优化提取过程）。不过实现这个将是更大的工作量！

## 顶点属性完全可选 [#](https://bevy.org/news/bevy-0-9/#vertex-attributes-fully-optional)

作者：@IceSentry

在之前的版本中，我们[通过在网格顶点属性上进行特化，使顶点属性可选成为可能](https://bevy.org/news/bevy-0-7/#flexible-mesh-vertex-layouts)。但我们保留了几个常见属性作为必需：position 和 normal。**Bevy 0.9** 完成了这项工作。所有标准网格顶点属性现在完全可选。如果你的网格由于某种原因不需要 position，Bevy 不会阻止你！

## 暴露 Multi Draw Indirect [#](https://bevy.org/news/bevy-0-9/#expose-multi-draw-indirect)

作者：@Neo-Zhixing

Wgpu 在支持的平台上可以选择性地支持"multi draw indirect" API，这是实现高效"GPU 驱动渲染"的关键部分。Bevy 现在通过其"tracked render pass"抽象暴露这些 API，使开发者能够使用这些 API 构建渲染功能。

## KTX2 数组/立方体贴图/立方体贴图数组纹理 [#](https://bevy.org/news/bevy-0-9/#ktx2-array-cubemap-cubemap-array-textures)

作者：Rob Swain (@superdump)

Bevy 现在可以正确加载 KTX2 数组、立方体贴图和立方体贴图数组纹理资源，这为天空盒等场景打开了大门：

Bevy 尚未对天空盒提供高级支持，但我们有一个[示例说明了如何由用户实现此功能](https://github.com/bevyengine/bevy/blob/v0.9.0/examples/3d/skybox.rs)

## Camera::viewport_to_world [#](https://bevy.org/news/bevy-0-9/#camera-viewport-to-world)

作者：@devil-ira

通常需要将"屏幕上的位置"转换为在该位置面向相机外部的射线。例如，如果你想点击 3D 场景中的某个东西来选择它，你可能会从相机视图中的该点投射一条射线，看看它是否与场景中的任何"碰撞器"相交。

Bevy 相机现在有一个 [`viewport_to_world`](https://docs.rs/bevy/0.9.0/bevy/render/camera/struct.Camera.html#method.viewport_to_world) 函数，它提供了这个功能：

```rust
let ray = camera.viewport_to_world(transform, cursor_position).unwrap();
if let Some(entity) = physics_context.cast_ray(ray.origin, ray.direction) {
  // 选择实体
}
```

以下基于光标的使用 [`viewport_to_world`](https://docs.rs/bevy/0.9.0/bevy/render/camera/struct.Camera.html#method.viewport_to_world) 计算从光标"发出"的射线，然后将其馈送到 [`bevy_rapier`](https://rapier.rs/docs/user_guides/bevy_plugin/getting_started_bevy) 物理库以检测和拾取光标下的卡片：

## 多方向光 [#](https://bevy.org/news/bevy-0-9/#multiple-directional-lights)

作者：@kurtkuehnert

Bevy 现在支持多个方向光（新的限制是一次 10 个）。就像我们为点光源所做的那样，我们可能会在未来在支持存储缓冲区的平台上使其无界，但这保持了所有平台兼容性的很好第一步。

![多个方向光](https://bevy.org/news/bevy-0-9/multiple_directional.png)

## 精灵矩形 [#](https://bevy.org/news/bevy-0-9/#sprite-rects)

作者：@inodentry

[`Sprites`](https://docs.rs/bevy/0.9.0/bevy/sprite/struct.Sprite.html) 现在可以定义"矩形"来选择其纹理的特定区域作为"精灵"：

```rust
Sprite {
  rect: Some(Rect {
    min: Vec2::new(100.0, 0.0),
    max: Vec2::new(200.0, 100.0),
  }),
  ..default()
}
```

![sprite rect](https://bevy.org/news/bevy-0-9/sprite_rect.png)

这类似于 [`TextureAtlasSprite`](https://docs.rs/bevy/0.9.0/bevy/sprite/struct.TextureAtlasSprite.html) / "精灵表"的工作方式，但不需要定义纹理图集。

## 插件设置 [#](https://bevy.org/news/bevy-0-9/#plugin-settings)

作者：@cart、@mockersf

在过去的 Bevy 版本中，"不可变"的插件设置被表示为普通 ECS 资源，在插件初始化时读取。这带来了一些问题：

1. 如果用户在插件初始化之后插入插件设置资源，它将被静默忽略（并使用默认值）
2. 用户可以在插件初始化之后修改插件设置资源。这造成了对已无法更改的设置的虚假控制感。

这些对于 `WindowDescriptor` 资源来说尤其有问题和令人困惑，但这是一个普遍问题。

为了解决这个问题，在 **Bevy 0.9** 中，我们将插件设置移到了插件本身上，并创建了覆盖默认设置的新 API：

```rust
app.add_plugins(DefaultPlugins
  .set(AssetPlugin {
    watch_for_changes: true,
    ..default()
  })
  .set(WindowPlugin {
    window: WindowDescriptor {
      width: 400.0,
      ..default()
    },
    ..default()
  })
)
```

这使得设置和插件之间的联系清晰，并将这些"插件初始化"设置与"运行时可配置"设置（仍然表示为 ECS 资源）区分开来。

## 插件现在默认是唯一的 [#](https://bevy.org/news/bevy-0-9/#plugins-are-now-unique-by-default)

作者：@mockersf

插件现在默认是唯一的。尝试向应用添加唯一的插件超过一次将导致错误。不打算唯一的插件可以覆盖默认的 `is_unique` 方法：

```rust
impl Plugin for MyPlugin {
  fn build(&self, app: &mut App) {
    app.add_system(some_system);
  }

  fn is_unique(&self) -> bool {
    false
  }
}
```

## 任务池：作用域上的嵌套生成 [#](https://bevy.org/news/bevy-0-9/#task-pool-nested-spawns-on-scope)

作者：@hymm

Bevy 的任务池现在支持"作用域上的嵌套生成"：

```rust
let results = task_pool.scope(|scope| {
    scope.spawn(async move {
        scope.spawn(async move { 1 });
        2
    });
});

assert!(results.contains(&1));
assert!(results.contains(&2));
```

这使得在执行其他任务时可以向任务池作用域添加新任务！这是实现新合并（但尚未实现）的 [Stageless RFC](https://github.com/bevyengine/rfcs/pull/45) 的要求，但它为在 Bevy 中生成异步任务的人们启用了新模式！

## 任务池 Panic 处理 [#](https://bevy.org/news/bevy-0-9/#task-pool-panic-handling)

作者：@james7132

Bevy 使用自己的自定义异步任务池来管理调度并行异步任务。在之前的 Bevy 版本中，如果任务在这些池中的一个 panic 了，它将是不可恢复的，除非每个调度的任务都使用 `catch_unwind`（这不可行）。这还会永久终止全局任务池中的工作线程。

**Bevy 0.9** 通过在任务池执行器内部调用 `catch_unwind` 解决了这个问题。

## 层级查询方法 [#](https://bevy.org/news/bevy-0-9/#hierarchy-query-methods)

作者：@devil-ira

为了使导航层级更容易，我们向 `Query<&Children>` 和 `Query<&Parent>` 添加了一些便捷方法：

```rust
#[derive(Resource)]
struct SomeEntity(Entity);

fn system(children: Query<&Children>, some_entity: Res<SomeEntity>) {
  // 迭代 some_entity 的所有后代
  for entity in children.iter_descendants(some_entity.0) {
  }
}

fn other_system(parents: Query<&Parent>, some_entity: Res<SomeEntity>) {
  // 迭代 some_entity 的所有祖先
  for entity in parents.iter_ancestors(some_entity.0) {
  }
}
```

## Bevy UI：原点现在在左上角 [#](https://bevy.org/news/bevy-0-9/#bevy-ui-the-origin-is-now-in-the-top-left)

作者：@mahulst

Bevy UI 现在将窗口的"左上角"视为"原点"，并向下延伸（Y 向下）。为了说明，考虑以下在"默认"位置（在原点）生成的组件的情况。

### 左上角原点（新）[#](https://bevy.org/news/bevy-0-9/#top-left-origin-new)

![origin top left](https://bevy.org/news/bevy-0-9/origin_top_left.png)

### 左下角原点（旧）[#](https://bevy.org/news/bevy-0-9/#bottom-left-origin-old)

![origin bottom left](https://bevy.org/news/bevy-0-9/origin_bottom_left.png)

我们选择进行此更改是因为几乎_整个_ UI 生态系统都使用左上角作为原点（Web、Godot、GTK、GPU 图像等）。

在 Bevy 还处于早期的那些日子里，我（`@cart`）最初选择左下角（Y 向上）是为了与 Bevy 的世界空间 2D 和 3D 坐标系统保持一致。理论上，我认为这会使一切都更容易理解。但实际上，这种一致性并没有给我们带来什么好处。而且在 UI 默认行为方面，这种行为违背了用户的期望。UI 倾向于向下延伸（从顶部），而不是向上延伸（从底部），所以覆盖默认值是常见做法。

幸运的是，在 **Bevy 0.9** 中，我们现在与生态系统中的其他部分对齐了！

## Bevy UI：Z-Indices [#](https://bevy.org/news/bevy-0-9/#bevy-ui-z-indices)

作者：@oceantume

Bevy UI 元素现在对它们的"z index"（它们是否在彼此"前面"或"后面"）有更多控制。在之前的 Bevy 版本中，这完全由层级决定：子级堆叠在父级和较早的兄弟级之上。这是一个很好的"默认值"，适用于大部分 UI，但某些类型的 UI 需要对元素排序有更多控制。

如果你是一名 Web 开发者，并且曾经使用过 `z-index` CSS 属性，那就是我们这里讨论的问题。

**Bevy 0.9** 添加了新的 [`ZIndex`](https://docs.rs/bevy/0.9.0/bevy/ui/enum.ZIndex.html) 组件，它是一个具有两种模式的枚举：

- `ZIndex::Local(i32)`：覆盖相对于其兄弟级的深度。
- `ZIndex::Global(i32)`：覆盖相对于 UI 根的深度。设置这个本质上允许 UI 元素"逃出"相对于父级的 z 排序，转而相对于整个 UI 进行排序。

在上下文中（局部 vs 全局）具有更高 z 级别的 UI 项目将显示在具有较低 z 级别的 UI 项目前面。z 级别内的平局回退到层级排序。"较晚"的子级堆叠在"较早"的子级之上。

为了说明，考虑以下 UI：

```txt
root (green)
  child1 (red)
  child2 (blue)
```

默认情况下这些的 z-index 都为 0。根在底部，每个后续子级堆叠在"上面"：

![z-index default](https://bevy.org/news/bevy-0-9/z-default.png)

如果我们希望蓝色子级堆叠在较早的红色子级"后面"，我们可以将其 z-index 设置为小于默认值 0 的"局部"值：

```rust
blue.z_index = ZIndex::Local(-1);
```

![z-index blue local](https://bevy.org/news/bevy-0-9/z-blue-local.png)

如果我们希望蓝色子级堆叠在绿色根"后面"，我们可以将其 z-index 设置为小于默认值 0 的"全局"值：

```rust
blue.z_index = ZIndex::Global(-1);
```

![z-index blue global](https://bevy.org/news/bevy-0-9/z-blue-global.png)

非常有用的东西！

## Bevy UI 缩放 [#](https://bevy.org/news/bevy-0-9/#bevy-ui-scaling)

作者：@Weibye

Bevy UI 的全局"像素缩放"现在可以使用 [`UiScale`](https://docs.rs/bevy/0.9.0/bevy/ui/struct.UiScale.html) 资源设置：

```rust
// 将 UI 像素单位渲染大 2 倍
app.insert_resource(UiScale { scale: 2.0 })
```

这允许开发者在灵活性有益的情况下向用户暴露任意缩放配置。

## 音频播放切换 [#](https://bevy.org/news/bevy-0-9/#audio-playback-toggling)

作者：@lovelymono

现在可以切换音频播放，它将在播放和暂停之间翻转：

```rust
// 旧，手动切换（仍然可能）
if audio_sink.is_paused() {
    audio_sink.play();
} else {
    audio_sink.pause();
}

// 新，自动切换
audio_sink.toggle();
```

## 时间缩放 [#](https://bevy.org/news/bevy-0-9/#time-scaling)

作者：@maniwani

"全局"时间缩放现在可以在 [`Time`](https://docs.rs/bevy/0.9.0/bevy/time/struct.Time.html) 上配置，它缩放 `Time::delta_seconds()` 等公共函数返回的值。

```rust
time.set_relative_speed(2.0);
```

在需要未缩放值的情况下，你可以使用这些函数的新"原始"变体：

```rust
// 自上次更新以来经过的秒数，考虑了时间缩放。
let delta = time.delta_seconds();

// 自上次更新以来经过的秒数，忽略了时间缩放。
let raw_delta = time.raw_delta_seconds();
```

## 时间回绕 [#](https://bevy.org/news/bevy-0-9/#time-wrapping)

作者：@IceSentry

某些场景（如[着色器](https://bevy.org/news/bevy-0-9/#time-shader-globals)）需要将经过的时间值表示为 `f32`，这很快会遇到精度问题。为了解决这个问题，[`Time`](https://docs.rs/bevy/0.9.0/bevy/time/struct.Time.html) 已被扩展为支持"时间回绕"：

```rust
// 每小时回绕一次
time.wrapping_period = Duration::from_secs(60 * 60):

// 如果自应用启动以来经过了一小时零 6 秒，
// 这将返回 6 秒。
let wrapped = time.seconds_since_startup_wrapped_f32();
```

## 下一步是什么？[#](https://bevy.org/news/bevy-0-9/#what-s-next)

以下是一些正在规划中的事情

- **高级后处理栈**：现在我们有了核心后处理管线，我们需要构建一个更高级的系统，使用户更容易在每个相机基础上选择、配置和重新排序后处理效果。此外，出于性能原因，我们希望将尽可能多的后处理效果合并到一个通道中，因此我们需要一组有主见的后处理 API 来促进这一点。
- **更多后处理效果**：更多抗锯齿选项（TAA、SMAA）、更多色调映射算法选项（例如：ACES）、SSAO
- **资源预处理**：我们将大力投资我们的资源管线，重点关注：
    1. 预处理资源以在"开发时间"执行昂贵的工作，以便 Bevy App 可以部署更漂亮、更小和/或加载更快的资源。
    2. 启用使用 .meta 文件配置资源。例如，你可以定义纹理压缩级别、应使用的过滤器或目标格式。
- **Bevy UI 改进**：我们将继续改进 Bevy UI 的功能并扩展其组件库，重点关注启用编辑器体验。
- **更多场景改进**：嵌套场景、隐式默认值和内联资源。
- **Bevy 编辑器**：我们将开始原型化 Bevy 编辑器体验，从场景编辑器工具开始。
- **Stageless ECS**：现在 [Stageless RFC](https://github.com/bevyengine/rfcs/pull/45) 已经合并，我们可以开始实现 stageless 调度了！有关即将推出的改进概述，请参阅 RFC。这将是游戏规则的改变者！

我们还在寻找一些关键领域的专家。我们目前的大多数开发者都专注于上述工作，所以如果你有兴趣并在以下领域有经验，我们很乐意听到你的声音！

- **动画**：动画混合、程序化动画和更高层次的动画系统。查看 GitHub 上标记为 [`A-Animation`](https://github.com/bevyengine/bevy/labels/A-Animation) 的问题，并在我们的 Discord 的 [`#animation-dev`](https://discord.com/channels/691052431525675048/774027865020039209) 频道上自我介绍。
- **音频**：我们需要对音频播放有更多控制，特别是在叠加效果方面。查看 GitHub 上标记为 [`A-Audio`](https://github.com/bevyengine/bevy/labels/A-Audio) 的问题，并在我们的 Discord 的 [`#audio-dev`](https://discord.com/channels/691052431525675048/749430447326625812) 频道上自我介绍。
