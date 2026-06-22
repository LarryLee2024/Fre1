# Bevy 0.7

## 发布于 2022 年 4 月 15 日，作者：Carter Anderson ( ![一个猫耳人物挥舞触手的剪影，或称 Octocat：GitHub 的吉祥物和标志](https://bevy.org/assets/github_grey.svg) [@cart](https://www.github.com/cart) ![一个圆角矩形内指向右的三角形；Youtube 的标志](https://bevy.org/assets/youtube_grey.svg) [cartdev](https://www.youtube.com/cartdev) )

!['风格化蘑菇'场景，由 QumoDone 制作，在 Bevy 中渲染。此场景采用 Creative Commons Attribution 许可。](https://bevy.org/news/bevy-0-7/mushroom.png)

['风格化蘑菇'场景，由 QumoDone 制作，在 Bevy 中渲染。此场景采用 Creative Commons Attribution 许可。](https://sketchfab.com/3d-models/stylized-mushrooms-9d22e02ce2a548959b1c4c4c1d546842)

感谢 **123** 位贡献者、**349** 个 pull request，以及我们[慷慨的赞助者](https://github.com/sponsors/cart)，我很高兴地宣布 **Bevy 0.7** 已在 [crates.io](https://crates.io/crates/bevy) 上发布！

如果你还不了解 Bevy，它是一款基于 Rust 构建的、简洁的数据驱动游戏引擎。你可以查看[快速入门指南](https://bevy.org/learn/quick-start/introduction)来开始使用。Bevy 将永久免费且开源！你可以在 GitHub 上获取完整的[源代码](https://github.com/bevyengine/bevy)。查看 [Bevy Assets](https://bevy.org/assets) 获取社区开发的插件、游戏和学习资源集合。

要将现有的 Bevy App 或 Plugin 更新到 **Bevy 0.7**，请查看我们的 [0.6 到 0.7 迁移指南](https://bevy.org/learn/migration-guides/0.6-0.7/)。

与往常一样，本次发布有_大量_新功能、bug 修复和生活质量改进，但以下是一些亮点：

- 骨骼动画和网格蒙皮
- GLTF 动画导入
- 场景中无限*点光源
- 改进的集群前向渲染：动态/自适应集群和更快、更准确的集群分配
- 压缩纹理支持（KTX2 / DDS / .basis）：在场景中加载更多纹理，更快
- 计算着色器/管线特化：Bevy 灵活的着色器系统被移植到计算着色器，支持热重载、shader defs 和 shader imports
- 渲染到纹理：相机现在可以配置为渲染到纹理而不是窗口
- 着色器中灵活的网格顶点布局
- ECS 改进：使用系统名称排序、Query::many_mut、通过 ParamSets 在系统中使用冲突参数、WorldQuery 派生
- 文档改进：更好的示例、更多文档测试和更高覆盖率
- 更多音频控制：暂停、音量、速度和循环
- 功耗选项：仅在输入发生时更新 Bevy App

## 骨骼动画 [#](https://bevy.org/news/bevy-0-7/#skeletal-animation)

作者：@james7132、@mockersf、@lassade、@Looooong

Bevy 终于支持 3D 骨骼动画了！

场景致谢：[Tanabata evening - Kyoto inspired city scene](https://skfb.ly/6TsvL) 由 Mathias Tossens 制作，采用 [Creative Commons Attribution](http://creativecommons.org/licenses/by/4.0/) 许可。角色模型和动画来自 Mixamo 的免费资源。

骨骼动画现在可以使用新的 [`AnimationPlayer`](https://docs.rs/bevy/0.7.0/bevy/animation/struct.AnimationPlayer.html) 组件和 [`AnimationClip`](https://docs.rs/bevy/0.7.0/bevy/animation/struct.AnimationClip.html) 资源进行播放、暂停、拖动、循环、反转和速度控制：

```rust
#[derive(Component)]
struct Animations {
    dance: Handle<AnimationClip>,
}

fn start_dancing(mut query: Query<(&Animations, &mut AnimationPlayer)>) {
    for (animations, mut animation_player) in query.iter_mut() {
        animation_player.play(animations.dance.clone());
    }
}
```

[`AnimationPlayer`](https://docs.rs/bevy/0.7.0/bevy/animation/struct.AnimationPlayer.html) 也可以用于动画化任意的 [`Transform`](https://docs.rs/bevy/0.7.0/bevy/transform/components/struct.Transform.html) 组件，而不仅仅是骨骼！

这个关键功能已经期待已久，但我们希望以一种与[新 Bevy 渲染器](https://bevy.org/news/bevy-0-6/#the-new-bevy-renderer)完美融合的方式来构建它，而不是"硬塞进去"。这建立在我们新的[灵活网格顶点布局](https://bevy.org/news/bevy-0-7/#flexible-mesh-vertex-layouts)、[Shader Imports](https://bevy.org/news/bevy-0-6/#shader-imports) 和 [Material](https://bevy.org/news/bevy-0-6/#materials) 系统之上，确保这些逻辑灵活且可复用，即使在非标准网格和自定义渲染管线中也是如此。

我们才刚刚开始！多轨道动画混合和更高层次的动画状态管理应该在不久的将来到来。现在是开始为 Bevy 贡献动画功能的好时机。我们已经突破了大部分基础技术障碍，剩下的主要是高层次的 API 设计选择。我们已经有几个相关的草案 RFC 在开放中：[Animation Composition](https://github.com/bevyengine/rfcs/pull/51) 和 [Animation Primitives](https://github.com/bevyengine/rfcs/pull/49)。欢迎加入讨论！

## GLTF 动画导入 [#](https://bevy.org/news/bevy-0-7/#gltf-animation-importing)

作者：@mockersf

Bevy 的 GLTF 导入器已扩展为将 GLTF 动画导入到新的 [`AnimationPlayer`](https://docs.rs/bevy/0.7.0/bevy/animation/struct.AnimationPlayer.html) 系统中。这支持"骨骼动画"和任意的 transform 动画：

```rust
struct FoxAnimations {
    walk: Handle<AnimationClip>,
}

fn setup(mut commands: Commands) {
    commands.spawn_scene(asset_server.load("models/animated/Fox.glb#Scene0"));
    commands.insert_resource(FoxAnimations {
        walk: asset_server.load("models/animated/Fox.glb#Animation0"),
    });
}

fn play_on_load(
    animations: Res<FoxAnimations>,
    mut players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    for mut player in players.iter_mut() {
        player.play(animations.walk.clone()).repeat();
    }
}
```

## 无限*点光源 [#](https://bevy.org/news/bevy-0-7/#unlimited-point-lights)

作者：Rob Swain (@superdump)、@robtfm

Bevy 现在可以在支持存储缓冲区的平台上渲染具有任意数量点光源的场景（基本上除了 WebGL 之外的所有平台）。在上次 Bevy 发布（0.6）中，我们添加了[集群前向渲染](https://bevy.org/news/bevy-0-6/#clustered-forward-rendering)，这是一种通过将光源分配到可见体积的子体积（称为"集群"）来优化每个片段的光照计算成本的渲染技术。然而，为了平台兼容性（WebGL），我们最初将自己限制在 256 个光源，因为这是 uniform 缓冲区绑定所能容纳的数量。

在 **Bevy 0.7** 中，我们添加了在支持无界存储缓冲区的平台上自动"升级"到使用无界存储缓冲区进行集群前向渲染的能力，从而实现了无限*点光源。之所以有一个星号，是因为实际上这受到内存和硬件限制。

## 光照集群功能和优化 [#](https://bevy.org/news/bevy-0-7/#light-clustering-features-and-optimizations)

作者：Rob Swain (@superdump)、@robtfm、@dataphract、@cart

随着 256 个点光源上限的移除，光源数量的唯一限制是硬件支持和我们算法中的瓶颈。为了增加光源数量，我们对集群算法进行了一些优化。

- **动态光照集群**
    - 默认情况下，集群 x/y 切片现在根据场景中的光源动态配置，这可以在某些场景中显著提高性能。
    - 集群行为现在也可以由用户配置为 FixedZ（新的默认动态 x/y 行为，固定 z 切片数量）、自定义固定 x/y/z 切片值、单集群和"无集群"，当你知道某个集群配置会表现更好时可以控制。
    - 此外，在 0.6 中，所有集群覆盖的可见体积基本上匹配视锥体的完整可见体积。这意味着如果所有点光源都在前景中，光源之外的所有集群都是浪费的空间。在 0.7 中，可以将远端限制设置得比相机远端更近，这意味着光源可以分布在更多集群中，从而显著提高渲染性能。
- **迭代球体细化**：Bevy 现在使用 Just Cause 3 的迭代球体细化方法进行集群分配，这在某些基准测试中给我们带来了约 10% 的性能提升和更准确的集群分配（这也可以提高渲染性能）。
- **光源视锥体变更检测**：我们现在使用 Bevy ECS 的变更检测功能，只重新计算已更改光源的视锥体。
- **集群分配优化**：集群分配的数据访问模式和数据结构进行了各种调整，提高了性能。

以下是展示从旧的 256 个点光源限制到以 60fps 渲染 25,000 个点光源的视频！

（注意，25,000 个光源的示例禁用了调试光源球体，以确保光照计算是瓶颈）

我们还有[更多集群优化](https://github.com/bevyengine/bevy/pull/4345)正在进行中！

## 可配置的光源可见性 [#](https://bevy.org/news/bevy-0-7/#configurable-light-visibility)

作者：@robtfm

光源现在可以使用 Bevy 标准的 [`Visibility`](https://docs.rs/bevy/0.7.0/bevy/render/view/struct.Visibility.html) 组件来开启和关闭：

```rust
commands.spawn(PointLightBundle {
    visibility: Visibility {
        is_visible: false,
    },
    ..default()
});
```

## 压缩 GPU 纹理 [#](https://bevy.org/news/bevy-0-7/#compressed-gpu-textures)

作者：Rob Swain (@superdump)

随着场景变大，它们的资源也随之变大。压缩这些资源是节省空间的好方法。下面展示的 Amazon Bistro 场景拥有超过 1GB 的压缩纹理。

PNG 是一种流行的压缩格式，但它必须在 GPU 使用之前解压缩。对于大型场景来说，这可能是一个缓慢的过程。然后这些纹理以未压缩的形式使用，占用大量有限的内存。压缩 GPU 纹理可以直接以其压缩格式被 GPU 使用，加载时无需任何额外处理。这显著减少了加载时间。由于它们保持压缩状态，这也显著减少了 RAM 使用量。

Bistro 场景使用 PNG 纹理总共需要 12.9 秒来加载，但使用压缩纹理只需 1.5 秒——加载时间大约只有十分之一！未压缩纹理的总 RAM 使用量约为 12GB，而压缩纹理为 5GB，不到一半！

好处还不止于此——由于纹理是压缩的并且可以由 GPU 以该格式使用，读取它们使用的内存带宽更少，这可以带来性能优势。Bistro 场景使用压缩纹理后帧率提高了约 10%。

![bistro 压缩](https://bevy.org/news/bevy-0-7/bistro_compressed.png)

另一个好处是支持 mipmap，这使得纹理更平滑、噪点更少。Bevy 目前不支持为未压缩纹理自动生成 mipmap，因此使用压缩纹理是现在拥有 mipmap 的好方法！

总之，Bevy 现在支持从 `.dds`、`.ktx2` 和 `.basis` 文件加载压缩纹理。这包括对标准 ASTC、BCn 和 ETC2 格式的支持，以及可以在运行时转码为特定系统支持的标准格式的"通用"格式（如 ETC1S 和 UASTC）。GLTF 加载器也已扩展为支持加载这些格式。

这些功能可以使用 `dds`、`ktx2` 和 `basis-universal` cargo features 启用。

## 渲染到纹理 [#](https://bevy.org/news/bevy-0-7/#render-to-texture)

作者：@HackerFoo

Bevy 现在通过配置 `Camera` 上的 `render_target` 字段，初步支持渲染到纹理。这使得镜子、分屏、3D 空间中的 2D UI、传送门等场景成为可能。

请注意，当前的实现相对底层。通常需要与 Bevy 的 Render Graph 交互并定义新的相机类型。如果你想现在使用此功能，[render_to_texture 示例](https://github.com/bevyengine/bevy/blob/main/examples/3d/render_to_texture.rs)说明了所需的步骤。我们计划实现["高级渲染目标"](https://github.com/bevyengine/bevy/discussions/4191)，使渲染到纹理只需几行代码。敬请关注详情！

## Bevy 原生计算着色器 [#](https://bevy.org/news/bevy-0-7/#bevy-native-compute-shaders)

作者：@Ku95

Bevy 灵活的资源驱动着色器系统被移植到计算着色器/管线，支持热重载、[shader defs](https://bevy.org/news/bevy-0-6/#shader-preprocessor)、[shader imports](https://bevy.org/news/bevy-0-6/#shader-imports) 和基于用户可配置键的[管线特化](https://bevy.org/news/bevy-0-6/#pipeline-specialization)：

```rust
#import "shaders/game_of_life_texture_bind_group.wgsl"

[[stage(compute), workgroup_size(8, 8, 1)]]
fn game_of_life_update([[builtin(global_invocation_id)]] invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let alive = is_location_alive(location);

    // shader defs 可在运行时配置，提示编译新的着色器变体
#ifdef WRITE_OUTPUT
    storageBarrier();
    textureStore(texture, location, vec4<f32>(f32(alive)));
#endif
}
```

## 灵活的网格顶点布局 [#](https://bevy.org/news/bevy-0-7/#flexible-mesh-vertex-layouts)

作者：@cart、@parasyte

在 **Bevy 0.7** 中，现在可以轻松地让着色器支持任何 Mesh 顶点布局和任意顶点属性。Bevy 的"着色器管线特化"系统被扩展为支持"基于网格顶点布局的特化"。

对于大多数 Bevy 用户来说，这意味着 [Materials](https://bevy.org/news/bevy-0-6/#materials)，包括内置的 [`StandardMaterial`](https://docs.rs/bevy/0.7.0/bevy/pbr/struct.StandardMaterial.html) 和自定义着色器材质，现在自动支持任意 Mesh，前提是这些 Mesh 具有材质着色器所需的顶点属性。这也意味着如果你的 Mesh 缺少其材质所需的任何属性，渲染可以优雅地失败。

我们还利用此系统为新的[骨骼动画](https://bevy.org/news/bevy-0-7/#skeletal-animation)实现实现了关节权重和索引。

对于喜欢编写底层图形管线的 Bevy 用户，此功能使得根据 Mesh 顶点布局轻松高效地特化管线成为可能：

```rust
impl SpecializedMeshPipeline for SomeCustomPipeline {
    type Key = SomeCustomKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        // 这是一个匹配所请求要求的布局，
        // 但针对当前正在渲染的任何 mesh 进行调整
        let vertex_buffer_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
        ])?;

        Ok(RenderPipelineDescriptor {
            vertex: VertexState {
                buffers: vec![vertex_buffer_layout],
                /* 在这里定义其余的顶点状态 */
            },
            /* 在这里定义其余的 mesh 管线 */
        })
    }
```

## 相机标记组件 [#](https://bevy.org/news/bevy-0-7/#camera-marker-components)

作者：@jakobhellermann

在 **Bevy 0.7** 中，相机现在使用"标记组件"模式来确定"相机类型"（例如：3D、2D、UI），而不是使用字符串名称。

这意味着选择特定类型的相机现在更便宜且更容易：

```rust
fn move_3d_camera_system(transforms: Query<&mut Transform, With<Camera3d>>) {
    for mut camera in transforms.iter_mut() {
        // 在这里移动相机
    }
}
```

## 人体工学的系统排序 [#](https://bevy.org/news/bevy-0-7/#ergonomic-system-ordering)

作者：@cart、@aevyrie、@alice-i-cecile、@DJMcNab

Bevy 使用"标签"来定义其 ECS 系统并行运行时之间的排序约束。在之前的 Bevy 版本中，对系统排序的唯一方法是定义自定义标签：

```rust
#[derive(SystemLabel, Clone, Hash, Debug, PartialEq, Eq)]
struct UpdateVelocity;

app
  .add_system(update_velocity.label(UpdateVelocity))
  .add_system(movement.after(UpdateVelocity))
```

在 **Bevy 0.7** 中，不再需要手动定义标签。你可以使用函数对系统排序，就像添加系统时一样！

```rust
app
  .add_system(update_velocity)
  .add_system(movement.after(update_velocity))
```

这是通过使用系统的 [`TypeId`](https://doc.rust-lang.org/std/any/struct.TypeId.html) 进行"自动标记"来实现的（标签类型是 [`SystemTypeIdLabel`](https://docs.rs/bevy/0.7.0/bevy/ecs/system/struct.SystemTypeIdLabel.html)）。内部排序仍然使用标签。

Bevy ECS 标签系统很强大，自定义标签仍然有合理的用例（例如用同一个标签标记多个系统，以及作为插件作者导出稳定的公共 API）。但大多数常见用例可以利用人体工学的自动标记功能。

## Default 简写 [#](https://bevy.org/news/bevy-0-7/#default-shorthand)

作者：@cart

Bevy 在初始化实体时大量使用 Rust 的[结构体更新模式](https://doc.rust-lang.org/book/ch05-01-defining-structs.html#creating-instances-from-other-instances-with-struct-update-syntax)结合 `Default` trait。这显著减少了所需的输入量，使开发者只需填写他们想要更改的字段。

标准做法是写出 `..Default::default()`：

```rust
commands.spawn_bundle(SpriteBundle {
    texture: some_texture,
    ..Default::default()
})
```

这比手动填写每个字段的组件好得多：

```rust
commands.spawn(SpriteBundle {
    texture: some_texture,
    sprite: Default::default(),
    transform: Default::default(),
    global_transform: Default::default(),
    visibility: Default::default(),
});
```

然而，当你需要为数十或数百个实体执行此操作时，这可能感觉重复。我们添加了一种使这更容易的方法，无需使用宏：

```rust
commands.spawn_bundle(SpriteBundle {
    texture: some_texture,
    ..default()
})
```

这在功能上等同于 `..Default::default()`，只是更简洁。如果你愿意，你仍然可以使用较长的形式。`default()` 函数包含在 Bevy 的 prelude 中，所以你不需要手动导入它。人体工学的胜利！

## Query::many [#](https://bevy.org/news/bevy-0-7/#query-many)

作者：@alice-i-cecile

Bevy ECS 解决了一个难题：在仍然尊重 Rust 严格的可变性和所有权规则的同时，提供轻松快速的并行数据访问。自首次发布以来，我们已经支持在 ECS 查询中高效地访问特定实体：

```rust
struct SomeEntities {
    a: Entity,
    b: Entity,
}

fn system(mut query: Query<&mut Transform>, entities: Res<SomeEntities>) {
    let a_transform = query.get_mut(entities.a).unwrap();
}
```

然而，为了尊重 Rust 的可变性规则，我们需要禁止可能产生"别名可变性"的 API。经验丰富的 Bevy 用户可能认出了这个 Rust 借用检查器错误：

```rust
fn system(mut query: Query<&mut Transform>, entities: Res<SomeEntities>) {
    let a_transform = query.get_mut(entities.a).unwrap();
    // 这一行编译失败，因为 `query` 已经在上面被可变借用了
    let b_transform = query.get_mut(entities.b).unwrap();
}
```

_你_知道 Entity A 和 Entity B 在运行时是不同的实体。但 Rust 的借用检查器在编译时无法知道这一点！我相信你可以想象出游戏开发中需要同时可变访问多个组件的场景。这个借用检查器限制是一个常见的痛点，而变通方法……并不有趣（使用作用域确保冲突的访问被丢弃、复制数据、重新查询事物等）。

幸运的是，**Bevy 0.7** 引入了一套全新的 API 来解决这个问题！

```rust
fn system(mut query: Query<&mut Transform>, entities: Res<SomeEntities>) {
    // 接受一个实体数组并返回一个可变查询结果数组
    // 如果存在实体冲突或实体不存在，这将 panic
    let [a_transform, b_transform] = query.many_mut([entities.a, entities.b]);
}
```

有很多变体：

```rust
// 与 many_mut 相同，但返回 Result 而不是 panic
if let Ok([a_transform, b_transform]) = query.get_many_mut([entities.a, entities.b]) {
}

// 还有不可变/只读变体
let [a_transform, b_transform] = query.many([entities.a, entities.b]);
if let Ok([a_transform, b_transform]) = query.get_many([entities.a, entities.b]) {
}
```

它们都支持任意数量的实体：

```rust
let [a, b, c] = query.many([entity_a, entity_b, entity_c]);
```

## ParamSets [#](https://bevy.org/news/bevy-0-7/#paramsets)

作者：@bilsen

为了防止别名可变性，Bevy ECS 禁止具有相互冲突参数的系统。例如，如果两个 Query 都请求对同一"原型"中的同一组件进行写访问，这可能导致别名可变访问，因此 Bevy 禁止该系统并报错。

之前版本的 Bevy 支持在同一系统中使用 QuerySets 来处理冲突的 Query，它一次只允许访问集合中的一个 Query：

```rust
// 这些查询可能都为同一实体返回可变 A 组件，因此它们必须放在一个集合中才能被视为有效系统。
fn system(mut set: QuerySet<(QueryState<(&mut A, &B)>, QueryState<(&mut A, &C)>)>) {
    for (a, b) in set.q0().iter_mut() {
    }
}
```

**Bevy 0.7** 移除了 `QuerySet`，转而使用 `ParamSet`，它将 QuerySet 模式泛化为_任何_系统参数：

```rust
fn system(mut set: ParamSet<(Query<(&mut A, &B)>, Query<(&mut A, &C)>)>) {
    for (a, b) in set.p0().iter_mut() {
    }
}
```

但 ParamSets 不仅限于 Query！考虑这个示例，其中 `EventWriter<Jump>` 参数（内部访问 `Events<Jump>` 资源）与对该资源的原始访问冲突。以前，表达这一点是不可能的。但有了 ParamSets，就可以了！

```rust
fn system(mut set: ParamSet<(EventWriter<Jump>, ResMut<Events<Jump>>)>) {
    for jump_event in set.p1().drain() {
    }
}
```

我们仍然建议尽可能避免使用 ParamSets 以保持清晰。但它们有时是必要且有用的工具！

## Deref / DerefMut 派生 [#](https://bevy.org/news/bevy-0-7/#deref-derefmut-derives)

作者：@MrGVSV

Rust 在用新功能或含义扩展类型时鼓励使用 [newtype 模式](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)。这也是 Bevy 中一个有用的工具：

```rust
#[derive(Component)]
struct Items(Vec<Item>);

fn give_sword(mut query: Query<&mut Items>) { 
    for mut items in query.iter_mut() {
        items.0.push(Item::new("Flaming Poisoning Raging Sword of Doom"));
    }
}
```

这可以正常工作，但 `items.0` 末尾的 `.0` 非常显眼。Bevy 组织中的许多人认为 `.0` 不应该出现在公共 API 中。但 newtype 模式仍然有用！理想情况下，Rust 会提供一种方式来表达 `Items` 是一个新类型，同时透明地提供对内部存储的 `Vec<Item>` 的访问。Rust 团队正在讨论相关设计，但我们不想等待好东西！

幸运的是，std 中的 Deref / DerefMut trait 提供了我们想要的行为。用户已经可以手动实现这些 trait，但对于如此常见的模式，我们决定提供自己的 trait 派生是值得的。在 **Bevy 0.7** 中，你现在可以派生 Deref 和 DerefMut，从而获得更好的公共 API：

```rust
#[derive(Component, Deref, DerefMut)]
struct Items(Vec<Item>);

fn give_sword(mut query: Query<&mut Items>) { 
    for mut items in query.iter_mut() {
        // 不再有 .0 了！
        items.push(Item::new("Flaming Poisoning Raging Sword of Doom"));
    }
}
```

细心的 `std` 文档读者可能注意到 Rust 团队[建议仅对智能指针使用 `Deref`/`DerefMut`，以避免混淆](https://doc.rust-lang.org/std/ops/trait.Deref.html)。像 `Items` 这样的组件_不是_智能指针。我们选择忽略这个建议，因为这种模式有效，已经在 Rust 生态系统中广泛使用，而且良好的用户体验优先。

## WorldQuery 派生 [#](https://bevy.org/news/bevy-0-7/#worldquery-derives)

作者：@mvlabat

有时在构建 Bevy App 时，你可能会发现自己在查询中一遍又一遍地重复相同的组件集合：

```rust
fn move_players(mut players: Query<(&mut Transform, &mut Velocity, &mut PlayerStats)>) {
    for (transform, velocity, stats) in players.iter_mut() {
    }
}

fn player_gravity(mut players: Query<(Entity, &mut Transform, &mut Velocity, &mut PlayerStats)>) {
    for (entity, transform, velocity, stats) in players.iter_mut() {
    }
}
```

也许你已经厌倦了一次又一次地输入相同的组件。在 **Bevy 0.7** 中，你现在可以使用 [`WorldQuery`](http://docs.rs/bevy/0.7.0/bevy/ecs/query/trait.WorldQuery.html) 派生轻松创建自己的自定义 [`WorldQuery`](http://docs.rs/bevy/0.7.0/bevy/ecs/query/trait.WorldQuery.html) trait 实现：

```rust
#[derive(WorldQuery)]
#[world_query(mutable)]
struct PlayerMovementQuery<'w> {
    transform: &'w mut Transform,
    velocity: &'w mut Velocity,
    stats: &'w mut PlayerStats,
}

fn move_players(mut players: Query<PlayerMovementQuery>) {
    for player in players.iter_mut() {
    }
}

fn player_gravity(mut players: Query<(Entity, PlayerMovementQuery)>) {
    for (entity, player) in players.iter_mut() {
    }
}
```

## World::resource [#](https://bevy.org/news/bevy-0-7/#world-resource)

作者：@alice-i-cecile

我们注意到大多数直接 [`World`](https://docs.rs/bevy/0.7.0/bevy/ecs/world/struct.World.html) 资源访问会立即解包 `get_resource` 的结果：

```rust
let time = world.get_resource::<Time>().unwrap();
```

在 **Bevy 0.7** 中，我们添加了一个人体工学的变体，内部会 panic：

```rust
let time = world.resource::<Time>();
```

还有一个可变变体：

```rust
let mut time = world.resource_mut::<Time>();
```

`get_resource` 变体仍然可用于用户想要手动处理返回的 `Option` 的情况。

## AnyOf 查询 [#](https://bevy.org/news/bevy-0-7/#anyof-queries)

作者：@TheRawMeatball

Bevy ECS 查询现在支持 [`AnyOf`](http://docs.rs/bevy/0.7.0/bevy/ecs/query/struct.AnyOf.html)，它将返回匹配"给定组件查询中任意一个"的实体结果：

```rust
fn system(query: Query<AnyOf<(&A, &B)>>) {
    for (a, b) in query.iter() {
        // A 或 B 保证为 Some
        assert!(a.is_some() || b.is_some())
    }
}
```

对于上面的示例，[`AnyOf`](http://docs.rs/bevy/0.7.0/bevy/ecs/query/struct.AnyOf.html) 将返回有 A 没有 B、有 B 没有 A，以及同时有 A 和 B 的实体。

## &World 系统参数 [#](https://bevy.org/news/bevy-0-7/#world-system-param)

作者：@bilsen

现在"普通系统"可以拥有 `&World` 系统参数，提供对整个 [`World`](https://docs.rs/bevy/0.7.0/bevy/ecs/world/struct.World.html) 的完整只读访问：

```rust
fn system(world: &World, transforms: Query<&Transform>) {
}
```

请记住，`&World` 将与_任何_可变 Query 冲突：

![此代码无效](https://bevy.org/assets/error_icon.svg "此代码无效")

```rust
fn invalid_system(world: &World, transforms: Query<&mut Transform>) {
}
```

在这些情况下，考虑使用我们的新 [ParamSets](https://bevy.org/news/bevy-0-7/#paramsets) 来解决冲突：

```rust
fn valid_system(set: ParamSet<(&World, Query<&mut Transform>)>) {
}
```

## ECS 健全性/正确性改进 [#](https://bevy.org/news/bevy-0-7/#ecs-soundness-correctness-improvements)

作者：@BoxyUwU、@TheRawMeatball、@bjorn3

Bevy ECS 在本次发布中收到了大量健全性和正确性 bug 修复：

- 移除了 `EntityMut` 和 `Query` 上不健全的生命周期标注，在某些情况下可用于获得别名可变性
- 将 `World::entities_mut` 标记为 unsafe（因为手动修改实体元数据可能使安全假设失效）
- 移除了不健全的 `World::components_mut`（它允许替换组件元数据，使 World 中其他地方的假设失效）
- 修复了 `World::resource_scope` 的健全性 bug
- 在资源 id 初始化中使用 `ManuallyDrop` 代替 `forget()`，以避免在数据指针使用之前使其失效

我们现在还在 CI 中对 Bevy ECS 运行 [miri](https://github.com/rust-lang/miri) 解释器，以帮助检测和防止未来的健全性/正确性问题。

随着 Bevy ECS 的成熟，我们对 unsafe 代码块和健全性的标准也必须成熟。Bevy ECS 可能永远不会 100% 没有 unsafe 代码块，因为我们正在建模 Rust 在没有我们帮助的情况下无法推理的并行数据访问。但我们致力于尽可能多地移除 unsafe 代码，并提高 unsafe 代码的质量和范围。

## 音频控制 [#](https://bevy.org/news/bevy-0-7/#audio-control)

作者：@mockersf

Bevy 的音频系统自首次发布以来一直处于……极简状态。到目前为止，它只支持在音频资源上点击"播放"。第三方插件如 [bevy_kira_audio](https://github.com/NiklasEi/bevy_kira_audio) 用更灵活的音频解决方案填补了空白。

在 **Bevy 0.7** 中，我们开始扩展内置音频插件的功能。现在可以使用 [`AudioSink`](http://docs.rs/bevy/0.7.0/bevy/audio/struct.AudioSink.html) 资源来暂停、调整音量和设置播放速度。

播放音频现在返回一个 `Handle<AudioSink>`，可用于播放/暂停/设置速度/设置音量：

```rust
struct BeautifulMusic(Handle<AudioSink>);

fn setup_audio(
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    audio_sinks: Res<Assets<AudioSink>>,
) {
    let music = asset_server.load("BeautifulMusic.ogg");
    // 播放音频并升级为强引用 handle
    let sink_handle = audio_sinks.get_handle(audio.play(music));
    commands.insert_resource(BeautifulMusic(sink_handle));
}

// 稍后在另一个系统中
fn adjust_audio(music: Res<BeautifulMusic>, mut audio_sinks: ResMut<Assets<AudioSink>>) {
    if let Some(sink) = audio_sinks.get(music.0) {
        // 暂停播放
        sink.pause();
        // 重新开始播放
        sink.play();
        // 增加音量
        sink.set_volume(sink.volume() + 0.1);
        // 减慢播放速度
        sink.set_speed(0.5);
    }
}
```

你还可以循环播放音频：

```rust
audio.play_with_settings(music, PlaybackSettings::LOOP.with_volume(0.75));
```

我们计划继续迭代这些 API，提供更多功能和可用性改进！

## 精灵锚点 [#](https://bevy.org/news/bevy-0-7/#sprite-anchors)

作者：@mockersf

[`Sprite`](https://docs.rs/bevy/0.7.0/bevy/sprite/enum.Sprite.html) 组件现在可以定义 [`Anchor`](https://docs.rs/bevy/0.7.0/bevy/sprite/enum.Anchor.html) 点（也称为"枢轴"点），它决定了精灵的"原点"。精灵仍然默认为"中心"原点，但现在可以配置：

```rust
commands.spawn_bundle(SpriteBundle {
    texture: asset_server.load("bevy_logo.png"),
    sprite: Sprite {
        anchor: Anchor::TopRight,
        ..default()
    },
    ..default()
});
```

## EventLoop 节能模式 [#](https://bevy.org/news/bevy-0-7/#eventloop-power-saving-modes)

作者：@aevyrie

默认情况下，Bevy 会"尽可能快地"运行更新（受屏幕刷新率限制）。这对于大多数游戏来说很好，但某些应用程序类型（如 GUI 应用）需要优先考虑 CPU 和 GPU 功耗。

**Bevy 0.7** 添加了在 [`WinitConfig`] 中配置 [`UpdateMode`] 的能力，以配置 Bevy App 如何运行更新：

- **Continuous**：始终"尽快"更新（遵守 vsync 配置）
- **Reactive**：仅在有窗口事件、请求重绘或经过可配置的等待时间后更新
- **ReactiveLowPower**：仅在有用户输入（鼠标移动、键盘输入等）、请求重绘或经过可配置的等待时间后更新

这些设置可以分别为聚焦窗口和未聚焦窗口配置（使你能够在窗口失去焦点时节省功耗）。

**ReactiveLowPower** 可以_显著_降低功耗/资源使用量，但它并不适合所有应用程序类型，因为某些应用需要假设它们始终在尽可能快地被更新。因此这些设置是可选启用的。

此应用演示了可用的各种模式。请注意，Game 模式配置为在失去焦点时降低其 tick 率，这不是默认值：

## 文档改进 [#](https://bevy.org/news/bevy-0-7/#documentation-improvements)

作者：@alice-i-cecile 及更多

优秀的文档使学习、使用和构建 Bevy 更好。但作为一个年轻的引擎，它们仍在进行中。

### deny-missing-docs [#](https://bevy.org/news/bevy-0-7/#deny-missing-docs)

我们的文档团队（由 `@alice-i-cecile` 领导）在 Rust 的 `#[warn(missing_docs)]` lint 的帮助下，已经开始[系统性地修复这个问题](https://github.com/bevyengine/bevy/issues/3492)。自 0.6 以来，我们已经完整地记录了（并防止文档回退）：

- `bevy_tasks`，由 `@james7132`
- `bevy_app`，由 `@dbearden`
- `bevy_dylib`，由 `@KDecay`
- `bevy_internal`，由 `@sheepyhead`

在此期间还有[许多其他文档改进](https://github.com/bevyengine/bevy/pulls?q=is%3Apr+is%3Aclosed+label%3AC-Docs)，包括添加了许多有用的[文档测试](https://doc.rust-lang.org/rustdoc/documentation-tests.html)，我们对新代码中文档的标准也在不断提高。非常感谢所有让 Bevy 文档变得更好的人。

### 新贡献者 [#](https://bevy.org/news/bevy-0-7/#new-contributors)

如果你[有兴趣贡献](https://github.com/bevyengine/bevy/blob/main/CONTRIBUTING.md)，文档团队随时准备帮助新贡献者尽快合并他们的第一个 Bevy PR。有_大量_新贡献者帮助处理了文档，无论是作为作者还是审阅者。如果这是你：谢谢！

### 更好的示例 [#](https://bevy.org/news/bevy-0-7/#better-examples)

对于许多人来说，学习一个工具的最好方法是看到它实际运行。我们一直在稳步打磨我们的[示例](https://github.com/bevyengine/bevy/tree/latest/examples)，提供更好的解释、更多覆盖范围和更高的代码质量。如果你是 Bevy 新手，请查看大幅改进的 [Breakout 示例](https://github.com/bevyengine/bevy/blob/latest/examples/games/breakout.rs)！

## 开发文档 [#](https://bevy.org/news/bevy-0-7/#dev-docs)

作者：@james7132、@mockersf、@aevyrie

我们现在在每次变更合并时自动将 Bevy 的 `main` 开发分支部署到 [https://dev-docs.bevy.org](https://dev-docs.bevy.org/)。这将帮助 Bevy 文档作者轻松验证他们的变更。"最前沿"的 Bevy 用户可以了解我们正在处理的 API 变更。

![dev docs](https://bevy.org/news/bevy-0-7/dev_docs.png)

## 网站改进 [#](https://bevy.org/news/bevy-0-7/#website-improvements)

作者：@doup

Bevy Book 现在有一个更好的分页组件，显示上一节/下一节名称：

![pager](https://bevy.org/news/bevy-0-7/pager.png)

我们还添加了一个"改进此页面"页脚链接，使 Bevy Book 读者更容易贡献变更。

侧边栏进行了大修，提高了清晰度，并可以打开/关闭部分而无需点击它们：

![sidebar](https://bevy.org/news/bevy-0-7/sidebar.png)

网站的响应性也得到了改进，某些部分在移动设备上的布局更好。

## 场景查看器工具 [#](https://bevy.org/news/bevy-0-7/#scene-viewer-tool)

作者：Rob Swain (@superdump)、@mockersf、@IceSentry、@jakobhellermann

Bevy 现在有一个专用的场景查看器工具，可以加载任意 GLTF 场景文件。如果你检出主 Bevy 仓库，可以通过运行来试用它：

```sh
cargo run --release --example scene_viewer /some/path/castle.gltf
```

它有一个内置的"飞行相机"，以及播放动画和切换光照和阴影的工具。

![场景查看器](https://bevy.org/news/bevy-0-7/bevy_scene_viewer.png)
