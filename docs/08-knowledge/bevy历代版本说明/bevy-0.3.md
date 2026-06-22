# Bevy 0.3

## 发布于 2020 年 11 月 3 日，作者：Carter Anderson ( ![一个猫耳人物挥舞触手的剪影，或称 Octocat：GitHub 的吉祥物和标志](https://bevy.org/assets/github_grey.svg) [@cart](https://www.github.com/cart) ![一个圆角矩形内指向右的三角形；Youtube 的标志](https://bevy.org/assets/youtube_grey.svg) [cartdev](https://www.youtube.com/cartdev) )

![Sheep Game by @schneckerstein](https://bevy.org/news/bevy-0-3/sheep_game.png)

[Sheep Game by @schneckerstein](https://twitter.com/schneckerstein/status/1309491121555410945)

在发布 Bevy 0.2 一个多月后，感谢 **59** 位贡献者、**122** 个 pull request，以及我们[慷慨的赞助者](https://github.com/sponsors/cart)，我很高兴地宣布 **Bevy 0.3** 已在 [crates.io](https://crates.io/crates/bevy) 上发布！

如果你还不了解 Bevy，它是一款基于 Rust 构建的、简洁的数据驱动游戏引擎。你可以查看[快速入门指南](https://bevy.org/learn/quick-start/introduction)来开始使用。Bevy 将永久免费且开源！你可以在 GitHub 上获取完整的[源代码](https://github.com/bevyengine/bevy)。

以下是本次发布的一些亮点：

## 初始 Android 支持 [#](https://bevy.org/news/bevy-0-3/#initial-android-support)

作者：@enfipy、@PrototypeNM1、@endragor、@naithar

你可以通过[按照此处的说明](https://github.com/bevyengine/bevy/blob/v0.3.0/examples/README.md#android)来试用 [Bevy Android 示例](https://github.com/bevyengine/bevy/tree/v0.3.0/examples/android)。虽然许多功能可以正常工作，但请注意这是_非常新_的功能。有些功能可能可以工作，而其他的可能不行。现在是深入参与并帮助我们弥补差距的好时机！

![android](https://bevy.org/news/bevy-0-3/android.png)

这是一项跨越多个项目的大型团队协作：

- Bevy：重写 bevy-glsl-to-spirv 以支持 android / 静态库 (@PrototypeNM1, @enfipy)
- Bevy：`bevy_asset` 后端使用 Android Asset Manager (@enfipy)
- Bevy：触摸支持 (@naithar)
- Bevy：纹理格式修复 (@enfipy)
- Bevy：UI 触摸修复、触摸力度和 android 示例 (@enfipy)
- Cpal：android 音频支持 (@endragor)
- android-ndk-rs / cargo-apk：修复以支持 Bevy 项目结构 (@PrototypeNM1)

## 初始 iOS 支持 [#](https://bevy.org/news/bevy-0-3/#initial-ios-support)

作者：@simlay、@MichaelHills、@Dash-L、@naithar

Bevy 现在可以在 iOS 上运行了！

![](https://bevy.org/news/bevy-0-3/ios.png)

你可以通过[按照此处的说明](https://github.com/bevyengine/bevy/tree/v0.3.0/examples#ios)来试用 [Bevy iOS 示例](https://github.com/bevyengine/bevy/tree/v0.3.0/examples/ios)。这也是一个新功能：有些功能可以工作，而其他的可能不行。

这又是一个跨越多个项目的大型团队协作：

- Bevy：XCode 项目 / 示例 (@simlay，在 @MichaelHills 的帮助下)
- Bevy：使用 shaderc 运行时着色器编译 (@MichaelHills)
- Bevy：Rodio 升级 (@Dash-L)
- Bevy：触摸支持 (@naithar)
- Winit：修复 iOS 竖屏视图 (@MichaelHills)
- RustAudio：iOS 支持 (@simlay 和 @MichaelHills)

已知问题：

- [音频尚不能完全工作](https://github.com/RustAudio/cpal/pull/485)

## WASM 资源加载 [#](https://bevy.org/news/bevy-0-3/#wasm-asset-loading)

作者：@mrk-its（并由 @cart 移植到新的 AssetIo）

@mrk-its 一直在努力扩展 Bevy 的 WASM 支持。在本次发布中，我们实现了 WASM 资源加载。现在，当你发布到 WASM 时，可以像在任何其他平台上一样加载资源：

```rust
asset_server.load("sprite.png");
```

如果资源尚未加载，这将通过 `fetch()` 请求通过 HTTP 获取资源。

@mrk-its 还在构建一个自定义的 WebGL2 `bevy_render` 后端。它已经相当可用，但还没有_完全_准备好。敬请期待更多消息！

## 触摸输入 [#](https://bevy.org/news/bevy-0-3/#touch-input)

作者：@naithar

Bevy 现在支持触摸：

```rust
fn touch_system(touches: Res<Touches>) {
    // 你可以像这样迭代所有当前触摸并获取它们的状态：
    for touch in touches.iter() {
        println!("active touch: {:?}", touch);
    }

    for touch in touches.iter_just_pressed() {
        println!("just pressed {:?}", touch);
    }

    for touch in touches.iter_just_released() {
        println!("just released {:?}", touch);
    }

    for touch in touches.iter_just_cancelled() {
        println!("just cancelled {:?}", touch);
    }
}
```

你也可以使用 `Events<TouchInput>` 资源来消费原始触摸事件。

## 资源系统改进 [#](https://bevy.org/news/bevy-0-3/#asset-system-improvements)

作者：@cart

### 资源 Handle 引用计数 [#](https://bevy.org/news/bevy-0-3/#asset-handle-reference-counting)

当资源的"handle 引用计数"归零时，资源现在会自动释放。这意味着你不再需要手动考虑释放资源：

```rust
// 调用 load() 现在返回一个强引用 handle：
let handle = asset_server.load("sprite.png");

// 注意你不再需要 unwrap() 加载的 handle。人体工学的胜利！

// 克隆一个 handle 会使引用计数增加一
let second_handle = handle.clone();

// 生成一个精灵并赋予它我们的 handle
commands.spawn(SpriteComponents {
    material: materials.add(handle.into()),
    ..Default::default()
});

// 稍后在其他系统中：
commands.despawn(sprite_entity);

// 不再有活跃的 handle 指向 "sprite.png"，所以它将在下一次更新前被释放
```

### 资源加载器现在可以加载多个资源 [#](https://bevy.org/news/bevy-0-3/#asset-loaders-can-now-load-multiple-assets)

在过去的版本中，`AssetLoaders` 只能生成单个类型的单个资源。在 **Bevy 0.3** 中，它们现在可以为任意类型生成任意数量的资源。在加载像 GLTF 文件这样的资源时，旧的行为非常受限，因为 GLTF 文件可能产生许多网格、纹理和场景。

### 子资源加载 [#](https://bevy.org/news/bevy-0-3/#sub-asset-loading)

有时你只想从资源源加载特定的资源。你现在可以像这样加载子资源：

```rust
// Mesh0/Primitive0 引用 "my_scene.gltf" 中的第一个网格图元
let mesh = asset_server.load("my_scene.gltf#Mesh0/Primitive0");
```

### AssetIo Trait [#](https://bevy.org/news/bevy-0-3/#assetio-trait)

`AssetServer` 现在由 `AssetIo` trait 支撑。这允许我们从任何想要的存储中加载资源。这意味着在桌面上我们从文件系统加载，在 Android 上我们使用 Android Asset Manager，在 Web 上我们使用 `fetch()` API 发出 HTTP 请求。

### 资源依赖 [#](https://bevy.org/news/bevy-0-3/#asset-dependencies)

资源现在可以依赖于其他资源，当原始资源被加载时，依赖的资源将自动加载。这在加载像"场景"这样的东西时很有用，因为场景可能引用其他资源源。我们在新的 GLTF 加载器中利用了这一点。

### 移除 AssetServer::load_sync() [#](https://bevy.org/news/bevy-0-3/#removed-assetserver-load-sync)

这可能会引起一些争议，但 `AssetServer::load_sync()` 必须被移除！这个 API 不对 WASM 友好，鼓励用户为了方便而阻塞游戏执行（这会导致"卡顿"），并且与新的 AssetLoader API 不兼容。资源加载现在始终是异步的。`load_sync()` 的用户应该改为使用 `load()` 加载资源，在系统中检查加载状态，并相应地更改游戏状态。

## GLTF 场景加载器 [#](https://bevy.org/news/bevy-0-3/#gltf-scene-loader)

作者：@cart

在此之前，GLTF 加载器的功能极其有限。它只能加载 GLTF 文件中的第一个带有单个纹理的网格。对于 **Bevy 0.3**，我们利用资源系统的改进编写了一个新的 `GltfLoader`，它将 GLTF 文件作为 Bevy `Scenes` 加载，同时加载文件中的所有网格和纹理。

以下是 Bevy 加载 Khronos Flight Helmet 示例的效果，它包含多个网格和纹理！

![flight helmet](https://bevy.org/news/bevy-0-3/flight_helmet.png)

以下是加载 GLTF 文件并将其作为场景生成的完整系统代码：

```rust
fn load_gltf_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let scene_handle = asset_server.load("models/FlightHelmet/FlightHelmet.gltf");
    commands.spawn_scene(scene_handle);
}
```

## Bevy ECS 改进 [#](https://bevy.org/news/bevy-0-3/#bevy-ecs-improvements)

作者：@cart

### 查询人体工学改进 [#](https://bevy.org/news/bevy-0-3/#query-ergonomics)

在本次发布中，我终于能够移除在 Bevy ECS 中我_真正讨厌_的东西。在之前的 Bevy 版本中，迭代 `Query` 中的组件看起来像这样：

```rust
for (a, b) in &mut query.iter() {
    // 这里的 `&mut` 感觉非常不自然
}

// 或者如果你喜欢，可以这样做
for (a, b) in query.iter().iter() {
    // query.iter().iter()？真的吗？？？
}
```

类似地，检索特定实体的组件看起来像这样：

```rust
if let Ok(mut result) = query.entity(entity) {
    if let Some((a, b)) = result.get() {
        // 在这里访问组件
    }
}
```

在 **Bevy 0.3** 中，你只需要这样做：

```rust
// 迭代
for (a, b) in query.iter() {
    // 甜蜜的人体工学体验
}

// 实体查找
if let Ok((a,b)) = query.get(entity) {
    // 样板代码消失了！
}
```

你可能会自然地想：

_为什么这花了这么长时间？为什么移除一个 `&mut` 会这么难？_

说来话长！总结如下：

- 旧 API 之所以那样设计是有原因的。它是良好的设计选择的结果，旨在防止在并行环境中发生不安全的内存访问。
- `query.iter()` 并没有真正返回一个迭代器。它返回一个_包装器_，该包装器持有组件存储上的原子锁。`query.entity()` 返回的类型也是如此。
- 移除这些"包装器类型"会导致不安全行为，因为另一个 Query 可能以违反 Rust 可变性规则的方式访问相同的组件。
- 由于迭代器实现和 rust 编译器的特性，移除包装器类型会导致迭代性能下降约 2-3 倍。

幸运的是，我们最终找到了解决所有这些问题的方法。新添加的 `QuerySets` 允许我们完全移除锁（和包装器类型）。通过完全重写 `QueryIter`，我们能够避免移除包装器带来的性能损失。请继续阅读了解详情！

### 100% 无锁并行 ECS [#](https://bevy.org/news/bevy-0-3/#100-lockless-parallel-ecs)

Bevy ECS 现在完全无锁。在 Bevy 0.2 中，我们将直接 `World` 访问和"for-each"系统设为无锁。这是可能的，因为 Bevy ECS 调度器确保系统只以尊重 Rust 可变性规则的方式并行运行。

我们无法从 `Query` 系统中移除锁，因为存在像这样的系统：

```rust
fn conflicting_query_system(mut q0: Query<&mut A>, mut q1: Query<(&mut A, &B)>) {
    let a = q0.get_mut(some_entity).unwrap();
    let (another_a, b) = q1.get_mut(some_entity).unwrap();
    // 啊啊啊！！！我们对 some_entity 的 A 组件有两个可变引用！
    // 非常不安全！
}
```

锁确保了第二个 `q1.get_mut(some_entity)` 访问会 panic，从而保证我们的安全。在 **Bevy 0.3** 中，像 `conflicting_query_system` 这样的系统在构建调度时就会失败。默认情况下，_系统不能有冲突的查询_。

然而，在某些情况下，系统_需要_冲突的查询来完成它需要做的事情。对于这些情况，我们添加了 `QuerySets`：

```rust
fn system(mut queries: QuerySet<(Query<&mut A>, Query<(&mut A, &B)>)>) {
    for a in queries.q0_mut().iter_mut() {
    }

    for (a, b) in queries.q1_mut().iter_mut() {
    }
}
```

通过将冲突的 `Queries` 放入 `QuerySet` 中，Rust 借用检查器保护我们免受不安全的查询访问。

因此，我们能够从 `query.iter()` 和 `query.get(entity)` 中移除_所有_安全检查，这意味着这些方法现在与它们在 `World` 中的对应方法（我们在 Bevy 0.2 中将其设为无锁）一样快。

### 性能改进 [#](https://bevy.org/news/bevy-0-3/#performance-improvements)

Bevy 在本次发布中有许多不错的性能改进：

- 从 Query 访问中移除原子锁，使 Bevy ECS 100% 无锁
- 从 Query 访问中移除原型"安全检查"。此时我们已经验证了给定的 Query 访问是安全的，所以不需要在每次调用时再次检查。
- 重写了 `QueryIter`，使其更简单（因此更容易控制优化），这使我们能够在不损失性能的情况下移除迭代器包装器。这也解决了一些性能不一致的问题，即某些系统排列组合性能最优而其他的则不是。现在一切都在"快速路径"上！
- 从上游 hecs 移植了一些性能改进，改进了对高度碎片化原型的迭代，并改进了组件插入时间

#### 获取实体组件（每 10 万次，单位：毫秒，越低越好）[#](https://bevy.org/news/bevy-0-3/#getting-an-entity-s-component-per-100k-in-milliseconds-smaller-is-better)

注意：这些数字是获取组件 100,000 次的结果，而不是单次组件查找

![获取实体组件](https://bevy.org/news/bevy-0-3/ecs_get_component.svg)

这是获得最大提升的地方。通过从 Query 系统中移除锁和安全检查，我们能够_显著_降低从系统内检索特定实体组件的成本。

我加入了与 [Legion ECS](https://github.com/amethyst/legion)（另一个具有并行调度器的优秀基于原型的 ECS）的比较，以说明为什么 Bevy 的新方法如此出色。Legion 在其系统中暴露了一个直接的"类 World" API（称为 SubWorld）。SubWorld 的 entry API _无法_预先知道将传入什么类型，这意味着它_必须_进行（相对）昂贵的安全检查，以确保用户不会请求不应该访问的内容。

Bevy 的调度器预先检查 `Queries` 一次，这允许系统在没有任何额外检查的情况下访问其结果。

测试是在每次系统迭代中查找（并修改）特定实体的组件 100,000 次。以下是每种情况下测试执行方式的简要说明：

- bevy (world)：直接使用 `world.get_mut::<A>(entity)` 进行 `World` 访问
- bevy (system)：包含 `Query<&mut A>` 的系统，使用 `query.get_mut(entity)` 访问组件
- legion (world)：直接使用 `let entry = world.entry(entity); entry.get_component_mut::<A>()` 进行 `World` 访问
- legion (system)：具有 `SubWorld` 访问权限的系统，使用 `let entry = world.entry(entity); entry.get_component_mut::<A>()`

值得注意的是，使用 `query.get_component::<T>(entity)` 而不是 `query.get(entity)` 确实需要安全检查，原因与 legion entry API 相同。我们无法预先知道调用者将传入什么组件类型，这意味着我们必须检查它以确保它匹配 `Query`。

此外，以下是一些相关的 [ecs_bench_suite](https://github.com/rust-gamedev/ecs_bench_suite) 结果（省略的基准测试没有显著变化）：

#### 组件插入（单位：微秒，越低越好）[#](https://bevy.org/news/bevy-0-3/#component-insertion-in-microseconds-smaller-is-better)

![组件插入](https://bevy.org/news/bevy-0-3/ecs_simple_insert.svg)

#### 组件添加/移除（单位：毫秒，越低越好）[#](https://bevy.org/news/bevy-0-3/#component-add-remove-in-milliseconds-smaller-is-better)

![组件添加/移除](https://bevy.org/news/bevy-0-3/ecs_add_remove.svg)

#### 碎片化迭代（单位：纳秒，越低越好）[#](https://bevy.org/news/bevy-0-3/#fragmented-iteration-in-nanoseconds-smaller-is-better)

![碎片化迭代](https://bevy.org/news/bevy-0-3/ecs_frag_iter.svg)

### 线程本地资源 [#](https://bevy.org/news/bevy-0-3/#thread-local-resources)

某些资源类型不能（或不应该）在线程之间传递。对于像窗口管理、输入和音频这样的底层 API 来说通常如此。现在可以将"线程本地资源"添加到 `Resources` 集合中，只能使用"线程本地系统"从主线程访问：

```rust
// 在你的应用设置中
app.add_thread_local_resource(MyResource);

// 一个线程本地系统
fn system(world: &mut World, resources: &mut Resources) {
    let my_resource = resources.get_thread_local::<MyResource>().unwrap();
}
```

### Query API 变更 [#](https://bevy.org/news/bevy-0-3/#query-api-changes)

首先，为了提高清晰度，我们将 `query.get::<Component>(entity)` 重命名为 `query.get_component::<Component>(entity)`。现在我们使用 `query.get(entity)` 返回特定实体的"完整"查询结果。

为了允许多个并发读取 Query（在安全的情况下），我们添加了独立的 `query.iter()` 和 `query.iter_mut()` API，以及 `query.get(entity)` 和 `query.get_mut(entity)`。"只读"的 Query 现在可以通过不可变借用检索其结果。

## 网格改进 [#](https://bevy.org/news/bevy-0-3/#mesh-improvements)

### 灵活的网格顶点属性 [#](https://bevy.org/news/bevy-0-3/#flexible-mesh-vertex-attributes)

作者：@julhe

Bevy 网格以前要求恰好三个"顶点属性"：`position`、`normal` 和 `uv`。这对大多数情况有效，但有许多情况需要其他属性，例如"顶点颜色"或"用于动画的骨骼权重"。**Bevy 0.3** 添加了对自定义顶点属性的支持。网格可以定义它们想要的任何属性，着色器可以使用它们想要的任何属性！

[这里有一个示例](https://github.com/bevyengine/bevy/blob/v0.3.0/examples/shader/mesh_custom_attribute.rs)，展示了如何定义一个自定义着色器来使用带有额外"顶点颜色"属性的网格。

![custom_vertex_attribute](https://bevy.org/news/bevy-0-3/custom_vertex_attribute.png)

### 索引缓冲区特化 [#](https://bevy.org/news/bevy-0-3/#index-buffer-specialization)

作者：@termhn

渲染网格通常涉及使用顶点"索引"来减少重复的顶点信息。Bevy 以前将这些索引的精度硬编码为 `u16`，这对某些情况来说太小了。现在渲染管线可以根据配置的索引缓冲区进行"特化"，默认为 `u32` 以覆盖大多数用例。

## Transform 再次重写 [#](https://bevy.org/news/bevy-0-3/#transform-re-rewrite)

作者：@MarekLg（设计协助：@AThilenius、@bitshifter、@termhn 和 @cart）

Transform 对于正确实现非常重要。它们在引擎的许多部分中使用，用户代码不断接触它们，而且计算成本相对较高：特别是 Transform 层级。

在上次发布中，我们大幅简化了 Bevy 的 Transform 系统，使用统一的 `Transform` 和 `GlobalTransform` 代替了多个独立的 `Translation`、`Rotation` 和 `Scale` 组件（它们会被同步到 `Transform` 和 `GlobalTransform`）。这使得面向用户的 API/数据流以及底层实现都更加简单。`Transform` 组件由 4x4 矩阵支撑。我按下了那个绿色的"合并"按钮……很高兴我们终于解决了 Transform 问题！

事实证明还有更多的工作要做！[@AThilenius 指出](https://github.com/bevyengine/bevy/issues/229#issuecomment-698953161)，使用 4x4 矩阵作为仿射变换的数据源会随时间累积误差。此外，Transform API 仍然有些繁琐。[在 @termhn 的建议下](https://github.com/bevyengine/bevy/issues/229#issuecomment-699172675)，我们决定研究使用"相似变换"（similarity）作为数据源。这带来了以下好处：

1. 不再有误差累积
2. 我们可以直接暴露 translation/rotation/scale 字段，大大简化了 API
3. 在某些情况下存储更便宜，计算层级也更便宜

我们共同认为这是一个好的前进方向，现在我们有了一个更好的重写。是的，这是_又一个_破坏性变更，但这就是为什么我们将 Bevy 标记为处于"实验阶段"。现在是尽可能多地打破东西的时候，以确保我们找到经得起时间考验的好 API。

以下是新的 `Transform` API 在 Bevy ECS 系统中的样子：

```rust
fn system(mut transform: Mut<Transform>) {
    // 沿正 x 轴移动
    transform.translation += Vec3::new(1.0, 0.0, 0.0);

    // 绕 y 轴旋转 180 度（π）
    transform.rotation *= Quat::from_rotation_y(PI);

    // 缩放 2 倍
    transform.scale *= 2.0;
}
```

与上一版本相比，这更容易使用、更正确，而且应该也稍微更快一些。

## 手柄设置 [#](https://bevy.org/news/bevy-0-3/#gamepad-settings)

作者：@simpuid

新添加的 `GamepadSettings` 资源使开发者能够按控制器、按轴/按钮自定义手柄设置：

```rust
fn system(mut gamepad_settings: ResMut<GamepadSettings>) {
    gamepad_settings.axis_settings.insert(
        GamepadAxis(Gamepad(0), GamepadAxisType::LeftStickX),
        AxisSettings {
            positive_high: 0.8,
            positive_low: 0.01,
            ..Default::default()
        },
    );
}
```

## 插件组 [#](https://bevy.org/news/bevy-0-3/#plugin-groups)

作者：@cart

如果你使用过 Bevy，你可能熟悉 `App` 初始化的这部分：

```rust
app.add_default_plugins();
```

这会添加所有"核心"引擎功能（渲染、输入、音频、窗口管理等）的插件。这很直接，但也非常静态。如果你不想添加_所有_默认插件呢？如果你想创建自己的自定义插件集呢？

为了解决这个问题，我们添加了 `PluginGroups`，它们是有序的插件集合，可以单独启用或禁用：

```rust
// 这个：
app.add_default_plugins()

// 已被替换为：
app.add_plugins(DefaultPlugins)

// 你可以在 PluginGroup 中禁用特定插件：
app.add_plugins_with(DefaultPlugins, |group| {
    group.disable::<RenderPlugin>()
         .disable::<AudioPlugin>()
});

// 你也可以创建自己的 PluginGroups：
pub struct HelloWorldPlugins;

impl PluginGroup for HelloWorldPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(PrintHelloPlugin)
             .add(PrintWorldPlugin);
    }
}

app.add_plugins(HelloWorldPlugins);
```

## 动态窗口设置 [#](https://bevy.org/news/bevy-0-3/#dynamic-window-settings)

作者：@mockersf

Bevy 提供了一个与后端无关的窗口管理 API。在此之前，窗口设置只能在应用启动时设置一次。如果你想动态设置窗口设置，你必须直接与窗口后端（例如：winit）交互。

在本次发布中，我们添加了使用 Bevy 窗口抽象在运行时动态设置窗口属性的能力：

```rust
// 这个系统动态地将窗口标题设置为自启动以来的秒数。因为为什么不呢？
fn change_title(time: Res<Time>, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_title(format!(
        "Seconds since startup: {}", time.seconds_since_startup
    ));
}
```

## 文档可搜索性 [#](https://bevy.org/news/bevy-0-3/#documentation-search-ability)

作者：@memoryruins

`bevy` crate 文档搜索函数现在会返回所有子 crate（如 bevy_sprite）的结果。由于 re-exported crate 的文档生成方式，默认情况下 `bevy` 搜索索引只覆盖"prelude"。@memoryruins 找到了一种方法来修复这个问题，通过创建新模块并在这些模块中导出每个 crate 的内容（而不是对 crate 进行别名）。

![docs](https://bevy.org/news/bevy-0-3/docs.png)
