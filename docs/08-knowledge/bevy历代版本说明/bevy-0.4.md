# Bevy 0.4

## 发布于 2020 年 12 月 19 日，作者 Carter Anderson ( ![GitHub 图标，一个带猫耳挥舞触手的剪影](https://bevy.org/assets/github_grey.svg) [@cart](https://www.github.com/cart) ![YouTube 图标，一个带圆角矩形的右指三角形](https://bevy.org/assets/youtube_grey.svg) [cartdev](https://www.youtube.com/cartdev) )

![Colonize 的截图：一款由 @indiv0 使用 Bevy 开发的 Dwarf-Fortress/Rimworld 风格游戏](https://bevy.org/news/bevy-0-4/colonize.png)

[Colonize 的截图：一款由 @indiv0 使用 Bevy 开发的 Dwarf-Fortress/Rimworld 风格游戏](https://github.com/indiv0/colonize/)

距离发布 Bevy 0.3 仅仅一个多月，感谢 **66** 位贡献者、**178** 个拉取请求以及我们的[**慷慨赞助商**](https://github.com/sponsors/cart），我很高兴地在 [crates.io](https://crates.io/crates/bevy) 上宣布 **Bevy 0.4** 发布！

对于那些还不知道的人，Bevy 是一个用 Rust 构建的、令人耳目一新的简单数据驱动游戏引擎。你可以查看[快速入门指南](https://bevy.org/learn/quick-start/introduction)来开始使用。Bevy 也是永久免费和开源的！你可以在 GitHub 上获取完整的[源代码](https://github.com/bevyengine/bevy)。

以下是本次发布的一些亮点：

## WASM + WebGL2 [#](https://bevy.org/news/bevy-0-4/#wasm-webgl2)

作者：@mrk-its

Bevy 现在有了 WebGL2 渲染后端！@mrk-its 一直在努力构建 [Bevy WebGL2 Plugin](https://github.com/mrk-its/bevy_webgl2) 并扩展 `bevy_render` 以满足 Web 的需求。他还搭建了一个漂亮的网站，展示了在 Web 上运行的各种 Bevy 示例和游戏。

我认为结果是显而易见的：

### [Bevy WebGL2 Showcase](https://mrk.sed.pl/bevy-showcase/) [#](https://bevy.org/news/bevy-0-4/#bevy-webgl2-showcase)

[![webgl2 展示](https://bevy.org/news/bevy-0-4/webgl_showcase.png)](https://mrk.sed.pl/bevy-showcase/)

## 跨平台主函数 [#](https://bevy.org/news/bevy-0-4/#cross-platform-main-function)

作者：@cart

在大多数受支持的 Bevy 平台上，你只需使用普通的 main 函数即可（例如：Windows、MacOS、Linux 和 Web）。以下是在这些平台上运行的最小的 Bevy 应用：

```rust
use bevy::prelude::*;

fn main() {
    App::build().run();
}
```

然而，某些平台（目前是 Android 和 iOS）需要额外的样板代码。这种神秘的魔法容易出错、占用空间，而且看起来不太美观。在此之前，Bevy 用户必须自己提供样板代码……但现在不必了！**Bevy 0.4** 添加了一个新的 `#[bevy_main]` 过程宏，它会自动为你插入相关的样板代码。这是朝着我们"一次编写，随处运行"目标迈出的一大步。

这个 Bevy 应用包含在 Windows、MacOS、Linux、Android、iOS 和 Web 上运行所需的全部代码：

```rust
use bevy::prelude::*;

#[bevy_main]
fn main() {
    App::build().run();
}
```

## 着色器热重载 [#](https://bevy.org/news/bevy-0-4/#live-shader-reloading)

作者：@yrns

Bevy 现在可以在运行时更新着色器的更改，让你无需重启应用即可获得即时反馈。这个视频没有加速！

## ECS 改进 [#](https://bevy.org/news/bevy-0-4/#ecs-improvements)

作者：@cart

没有新一轮的 ECS 改进，怎么能算 Bevy 更新呢！

### 灵活的 ECS 参数 [#](https://bevy.org/news/bevy-0-4/#flexible-ecs-parameters)

之前版本的 Bevy 强制你按照特定的顺序提供系统参数：

```rust
/// 该系统遵循 [Commands][Resources][Queries] 的顺序，编译正常
fn valid_system(mut commands: Commands, time: Res<Time>, query: Query<&Transform>) {
}

/// 该系统不符合 required 的顺序，导致编译失败
fn invalid_system(query: Query<&Transform>, mut commands: Commands, time: Res<Time>) {
}
```

新手们经常掉进这个陷阱。这些完全武断的限制是内部实现的一个特点。`IntoSystem` trait 仅为特定顺序实现。支持所有顺序会指数级地影响编译时间。内部实现也是通过一个[出了名复杂的宏](https://github.com/bevyengine/bevy/blob/9afe196f1690a6a6e47bf67ac740b4edeffd97bd/crates/bevy_ecs/src/system/into_system.rs#L158)构建的。

为了解决这个问题，我完全重写了我们生成系统的方式。我们现在使用一个 `SystemParam` trait，为每个参数类型实现该 trait。这带来了许多好处：

- **显著更快的编译时间**：我们看到全新编译时间减少了 **~25%**
- **使用你想要的任何参数顺序**：不再有任意的顺序限制！
- **轻松添加新参数**：现在对我们（和用户）来说，创建新参数都很容易。只需实现 `SystemParam` trait！
- **更简单的实现**：新的实现更小，也更容易维护和理解。

```rust
// 在 Bevy 0.4 中，这个系统现在是完全有效的。酷！
fn system(query: Query<&Transform>, commands: &mut Commands, time: Res<Time>) {
}
```

注意，在 **Bevy 0.4** 中，commands 现在看起来是 `commands: &mut Commands`，而不是 `mut commands: Commands`。

### 简化的查询过滤器 [#](https://bevy.org/news/bevy-0-4/#simplified-query-filters)

到目前为止，Bevy 的查询过滤器是与组件混杂在一起的：

```rust
fn system(query: Query<With<A, Without<B, (&Transform, Changed<Velocity>)>>>) {
}
```

困惑吗？你不会是第一个！你可以将上面的查询理解为"给我所有拥有 `A` 组件、_没有_ `B` 组件、并且 Velocity 组件发生变化的实体的 `Transform` 和 `Velocity` 组件的不可变引用"。

首先，通过 With/Without 进行的类型嵌套使得理解起来非常不清楚。此外，很难判断 `Changed<Velocity>` 参数的作用。它只是一个过滤器吗？它也会返回一个 Velocity 组件吗？如果是，它是不可变的还是可变的？

将这些概念分离是有意义的。在 **Bevy 0.4** 中，查询过滤器与查询组件是分开的。上面的查询看起来是这样的：

```rust
// 带过滤器的查询
fn system(query: Query<(&Transform, &Velocity), (With<A>, Without<B>, Changed<Velocity>)>) {
}

// 不带过滤器的查询
fn system(query: Query<(&Transform, &Velocity)>) {
}
```

这使得一眼就能看出查询在做什么变得容易得多。它还带来了更可组合的行为。例如，你现在可以过滤 `Changed<Velocity>` 而不必实际获取 `Velocity` 组件。

现在过滤器是一个单独的类型，你可以为想要重用的过滤器创建类型别名：

```rust
type ChangedVelocity = (With<A>, Without<B>, Changed<Velocity>);

fn system(query: Query<(&Transform, &Velocity), ChangedVelocity>) {
}
```

### 系统输入、输出和链式调用 [#](https://bevy.org/news/bevy-0-4/#system-inputs-outputs-and-chaining)

系统现在可以有输入和输出。这开辟了各种有趣的行为，例如系统错误处理：

```rust
fn main() {
  App::build()
    .add_system(result_system.system().chain(error_handler.system()))
    .run();
}

fn result_system(query: Query<&Transform>) -> Result<()> {
  let transform = query.get(SOME_ENTITY)?;
  println!("找到实体变换：{:?}", transform);
  Ok(())
}

fn error_handler_system(In(result): In<Result<()>>, error_handler: Res<MyErrorHandler>) {
  if let Err(err) = result {
      error_handler.handle_error(err);
  }
}
```

[`System`](https://docs.rs/bevy/0.4.0/bevy/prelude/trait.System.html) trait 现在看起来是这样的：

```rust
// 没有输入和输出
System<In = (), Out = ()>

// 接受一个 usize 作为输入并返回一个 f32
System<In = usize, Out = f32>
```

我们在新的 Schedule 实现中使用了这个功能。

### Schedule V2 [#](https://bevy.org/news/bevy-0-4/#schedule-v2)

Bevy 旧的 Schedule 很好。系统注册易于阅读和组合。但它也有显著的局限性：

- 只允许一个 Schedule
- 非常静态：你只能使用我们提供的工具：
    - stage 只是系统的列表
    - stage 被添加到 schedule 中
    - stage 使用硬编码的系统运行器
- 不能在运行时切换 schedule
- 不能轻松支持"固定时间步长"场景

为了解决这些问题，我从头编写了一个新的 Schedule 系统。在你担心之前，这些基本上都是*非破坏性*的变更。你熟悉和喜爱的高级"app builder"语法仍然可用：

```rust
app.add_system(my_system.system())
```

#### Stage Trait [#](https://bevy.org/news/bevy-0-4/#stage-trait)

Stage 现在是一个 trait。你现在可以实现自己的 [`Stage`](https://docs.rs/bevy/0.4.0/bevy/ecs/trait.Stage.html) 类型了！

```rust
struct MyStage;

impl Stage for MyStage {
    fn run(&mut self, world: &mut World, resources: &mut Resources) {
        // 在这里做 stage 相关的事情。
        // 你有对 World 和 Resources 的独占访问权限，所以你可以做任何事情
    }
}
```

#### Stage 类型：[`SystemStage`](https://docs.rs/bevy/0.4.0/bevy/ecs/struct.SystemStage.html) [#](https://bevy.org/news/bevy-0-4/#stage-type-systemstage)

这基本上是一个"普通"的 stage。你可以向其中添加系统，并决定这些系统将如何执行（并行、串行或自定义逻辑）：

```rust
// 并行运行系统（使用默认的并行执行器）
let parallel_stage =
    SystemStage::parallel()
        .with_system(a.system())
        .with_system(b.system());

// 串行运行系统（按注册顺序）
let serial_stage =
    SystemStage::serial()
        .with_system(a.system())
        .with_system(b.system());

// 你也可以编写自己的自定义 SystemStageExecutor
let custom_executor_stage =
    SystemStage::new(MyCustomExecutor::new())
        .with_system(a.system())
        .with_system(b.system());
```

#### Stage 类型：[`Schedule`] [#](https://bevy.org/news/bevy-0-4/#stage-type-schedule)

你没看错！[`Schedule`] 现在实现了 [`Stage`](https://docs.rs/bevy/0.4.0/bevy/ecs/trait.Stage.html) trait，这意味着你可以将 Schedule 嵌套在其他 Schedule 中：

```rust
let schedule = Schedule::default()
    .with_stage("update", SystemStage::parallel()
        .with_system(a.system())
        .with_system(b.system())
    )
    .with_stage("nested", Schedule::default()
        .with_stage("nested_stage", SystemStage::serial()
            .with_system(b.system())
        )
    );
```

[`Schedule`](https://docs.rs/bevy/0.4.0/bevy/ecs/struct.Schedule.html)

#### 运行条件 [#](https://bevy.org/news/bevy-0-4/#run-criteria)

你可以向任何 [`SystemStage`](https://docs.rs/bevy/0.4.0/bevy/ecs/struct.SystemStage.html) 或 [`Schedule`] 添加"运行条件"。

```rust
// "运行条件"只是一个返回 `ShouldRun` 结果的系统
fn only_on_10_criteria(value: Res<usize>) -> ShouldRun {
    if *value == 10 { 
        ShouldRun::Yes 
    } else { 
        ShouldRun::No
    }
}

app
    // 这个 stage 仅在 Res<usize> 的值为 10 时运行
    .add_stage_after(stage::UPDATE, "only_on_10_stage", SystemStage::parallel()
        .with_run_criteria(only_on_10_criteria.system())
        .with_system(my_system.system())
    )
    // 这个 stage 只运行一次
    .add_stage_after(stage::RUN_ONCE, "one_and_done", Schedule::default()
        .with_run_criteria(RunOnce::default())
        .with_system(my_system.system())
    )
```

#### 固定时间步长 [#](https://bevy.org/news/bevy-0-4/#fixed-timestep)

你现在可以按"固定时间步长"运行 stage。

```rust
// 这个 stage 每 0.4 秒运行一次
app.add_stage_after(stage::UPDATE, "fixed_update", SystemStage::parallel()
    .with_run_criteria(FixedTimestep::step(0.4))
    .with_system(my_system.system())
)
```

这建立在 `ShouldRun::YesAndLoop` 之上，它确保 schedule 持续循环，直到消耗完所有累积的时间。

如果你想了解更多关于固定时间步长的信息，请查看这篇优秀的文章["Fix Your Timestep!"](https://gafferongames.com/post/fix_your_timestep/)。

#### 类型化 Stage 构建器 [#](https://bevy.org/news/bevy-0-4/#typed-stage-builders)

现在 stage 可以是任何类型，我们需要一种方式让 [`Plugin`](https://docs.rs/bevy/0.4.0/bevy/app/trait.Plugin.html) 与任意 stage 类型交互：

```rust
app
    // 这个"高级"构建器模式仍然有效（并假定 stage 是一个 SystemStage）
    .add_system(some_system.system())
    // 这个"低级"构建器等价于 add_system()
    .stage(stage::UPDATE, |stage: &mut SystemStage|
        stage.add_system(some_system.system())
    )
    // 这对于自定义 stage 类型也有效
    .stage(MY_CUSTOM_STAGE, |stage: &mut MyCustomStage|
        stage.do_custom_thing()
    )
```

### 弃用的 For-Each 系统 [#](https://bevy.org/news/bevy-0-4/#deprecated-for-each-systems)

之前版本的 Bevy 支持"for-each"系统，看起来像这样：

```rust
// 每次更新时，该系统对每个带有 Transform 组件的实体运行一次
fn system(time: Res<Time>, entity: Entity, transform: Mut<Transform>) {
    // 在这里做每个实体的逻辑
}
```

从现在开始，上面的系统应该这样写：

```rust
// 每次更新时，该系统运行一次并在内部迭代每个实体
fn system(time: Res<Time>, query: Query<(Entity, &mut Transform)>) {
    for (entity, mut transform) in query.iter_mut() {
        // 在这里做每个实体的逻辑
    }
}
```

For-each 系统看起来不错，有时还能省点打字。为什么要移除它们？

1. For-each 系统在多个方面有根本性的限制。它们无法迭代已移除的组件、无法过滤、无法控制迭代，也无法同时使用多个查询。这意味着一旦需要这些功能，就必须将它们转换为"查询系统"。
2. Bevy 总体上应该保持"一种做事方式"。For-each 系统是一种稍微更符合人体工程学的方式来定义一小部分系统类型。这迫使人们在不需要做的时候做出"设计决策"。它也使得示例和教程因为人们对其中一种的偏好而不一致。
3. 存在一些新手经常遇到的"陷阱"，在我们的支持论坛中不断出现，让新手感到困惑：
    - 用户期望 `&mut T` 查询能在 foreach 系统中工作（例如：`fn system(a: &mut A) {}`）。这些不能工作，因为我们需要 `Mut<T>` 跟踪指针来确保变更跟踪始终如预期地工作。等价的 `Query<&mut A>` 可以工作，因为我们在迭代查询时可以返回跟踪指针。
    - 一个"在某些条件下运行此 for-each 系统"的 bug 足够常见，以至于我们不得不在 Bevy 手册中介绍它。
4. 它们增加了编译时间。移除 for-each 系统在全新编译上为我节省了大约 5 秒。
5. 它们的内部实现需要一个复杂的宏。这影响了可维护性。

## 状态 [#](https://bevy.org/news/bevy-0-4/#states)

作者：@cart

应大众要求，Bevy 现在支持状态（States）。这些是逻辑上的"应用状态"，允许你根据应用所处的状态来启用/禁用系统。

状态被定义为普通的 Rust 枚举：

```rust
#[derive(Clone)]
enum AppState {
    Loading,
    Menu,
    InGame
}
```

然后你像这样将它们作为资源添加到你的应用中：

```rust
// 添加一个新的默认为 Loading 状态的 AppState 资源
app.add_resource(State::new(AppState::Loading))
```

要根据当前状态运行系统，添加一个 [`StateStage`](https://docs.rs/bevy/0.4.0/bevy/ecs/struct.StateStage.html)：

```rust
app.add_stage_after(stage::UPDATE, STAGE, StateStage::<AppState>::default())
```

然后你可以像这样为每个状态值/生命周期事件添加系统：

```rust
app
    .on_state_enter(STAGE, AppState::Menu, setup_menu.system())
    .on_state_update(STAGE, AppState::Menu, menu.system())
    .on_state_exit(STAGE, AppState::Menu, cleanup_menu.system())
    .on_state_enter(STAGE, AppState::InGame, setup_game.system())
    .on_state_update(STAGE, AppState::InGame, movement.system())
```

注意有不同的"生命周期事件"：

- **on_enter**：首次进入一个状态时运行一次
- **on_exit**：退出一个状态时运行一次
- **on_update**：每次运行 stage 时恰好运行一次（在任何 on_enter 或 on_exit 事件运行之后）

你可以像这样从系统内部排队一个状态变更：

```rust
fn system(mut state: ResMut<State<AppState>>) {
    state.set_next(AppState::InGame).unwrap();
}
```

排队的状态变更在 [`StateStage`](https://docs.rs/bevy/0.4.0/bevy/ecs/struct.StateStage.html) 结束时被应用。如果你在 [`StateStage`](https://docs.rs/bevy/0.4.0/bevy/ecs/struct.StateStage.html) 内部改变状态，生命周期事件将在同一次更新/帧中发生。你可以任意多次地这样做（即它会继续运行状态生命周期系统，直到没有更多的变更排队）。这确保了多个状态变更可以在同一帧内被应用。

## GLTF 改进 [#](https://bevy.org/news/bevy-0-4/#gltf-improvements)

作者：@iwikal, @FuriouZz, @rod-salazar

Bevy 的 GLTF 加载器现在可以导入相机了。以下是在 Blender 中设置的简单场景：

![gltf 相机 blender](https://bevy.org/news/bevy-0-4/gltf_camera_blender.png)

这是在 Bevy 中的样子（光照不同，因为我们还没有导入灯光）：

![gltf 相机 bevy](https://bevy.org/news/bevy-0-4/gltf_camera_bevy.png)

还有一些其他改进：

- 从 GLTF 导入图像时的像素格式转换
- 默认材质加载
- 层级修复

## 将场景生成为子节点 [#](https://bevy.org/news/bevy-0-4/#spawn-scenes-as-children)

作者：@mockersf

场景现在可以像这样作为子节点生成：

```rust
commands
    .spawn((
        Transform::from_translation(Vec3::new(0.5, 0.0, 0.0)),
        GlobalTransform::default(),
    ))
    .with_children(|parent| {
        parent.spawn_scene(asset_server.load("scene.gltf"));
    });
```

通过在父节点下生成，这使得你可以平移/旋转/缩放同一场景的多个实例：

![场景子节点](https://bevy.org/news/bevy-0-4/scene_children.png)

## 动态链接 [#](https://bevy.org/news/bevy-0-4/#dynamic-linking)

作者：@bjorn3, @cart

@bjorn3 发现你可以强制 Bevy 进行动态链接。

这*显著*减少了迭代编译时间。看看使用[快速编译配置](https://bevy.org/learn/quick-start/getting-started/setup/) _和_ 动态链接时，编译对 `3d_scene.rs` 示例的一个更改需要多长时间：

![快速动态](https://bevy.org/news/bevy-0-4/dynamic_fast.png)

### 编译对 3d_scene 示例的更改所需的时间（秒，越少越好） [#](https://bevy.org/news/bevy-0-4/#time-to-compile-change-to-3d-scene-example-in-seconds-less-is-better)

![快速编译](https://bevy.org/news/bevy-0-4/fast_compiles.svg)

我们添加了一个 cargo feature，可以在开发期间轻松启用动态链接：

```sh
# 对于 bevy 应用
cargo run --features bevy/dynamic

# 对于 bevy 示例
cargo run --features dynamic --example breakout
```

只需记住在发布你的游戏时禁用这个 feature。

## 文本布局改进 [#](https://bevy.org/news/bevy-0-4/#text-layout-improvements)

作者：@AlisCode, @tigregalis

之前的 Bevy 版本使用了一个自定义的、朴素的文本布局系统。它有许多 bug 和限制，比如臭名昭著的"波浪文本"bug：

![波浪文本](https://bevy.org/news/bevy-0-4/wavy_text.png)

新的文本布局系统使用了 glyph_brush_layout，修复了布局错误并添加了许多新的布局选项。注意示例中使用的"Fira Sans"字体有一些风格上的"波浪感"……这不是 bug：

![文本布局](https://bevy.org/news/bevy-0-4/text_layout.png)

## 渲染器优化 [#](https://bevy.org/news/bevy-0-4/#renderer-optimization)

作者：@cart

Bevy 的渲染 API 旨在易于使用和扩展。我想先确定一个良好的 API，但这导致了大量性能相关的 TODO，造成了一些严重的开销。

对于 **Bevy 0.4**，我决定尽可能多地解决这些 TODO。还有很多工作要做（比如实例化和批处理），但 Bevy 的性能已经比以前*好得多*了。

### 增量一切 [#](https://bevy.org/news/bevy-0-4/#incrementalize-everything)

Bevy 的大部分高级渲染抽象都是为了增量更新而设计的，但在我最初构建引擎时，ECS 变更检测还没有实现。现在我们有了所有这些好用的优化工具，是时候使用它们了！

在第一个优化轮次中，我尽可能多地实现了增量：

- 为 RenderResourceNode、Sprites 和 Transforms 添加了变更检测，在这些值不变时提升了性能
- 仅在资源发生变化时同步 GPU 数据
- 在所有引用同一资源的实体间共享资源 RenderResourceBindings
- Mesh 提供系统现在仅在需要时更新网格特化
- 停止每帧清空绑定组，改为每隔一帧移除过期的绑定组
- 缓存未匹配的渲染资源绑定结果（防止每个实体每帧的冗余计算）
- 当状态实际上没有变化时，不发送渲染通道状态变更命令

#### 绘制 10,000 个静态精灵的帧时间（毫秒，越少越好） [#](https://bevy.org/news/bevy-0-4/#frame-time-to-draw-10-000-static-sprites-in-milliseconds-less-is-better)

![bevy 第一轮静态](https://bevy.org/news/bevy-0-4/bevy_round1_static.svg)

#### 绘制 10,000 个移动精灵的帧时间（毫秒，越少越好） [#](https://bevy.org/news/bevy-0-4/#frame-time-to-draw-10-000-moving-sprites-in-milliseconds-less-is-better)

![bevy 第一轮动态](https://bevy.org/news/bevy-0-4/bevy_round1_dynamic.svg)

### 优化文本渲染（和其他即时渲染） [#](https://bevy.org/news/bevy-0-4/#optimize-text-rendering-and-other-immediate-rendering)

文本渲染（以及任何使用 `SharedBuffers` 即时渲染抽象的东西）在之前的 Bevy 版本中*极其*缓慢。这是因为 `SharedBuffers` 抽象是一个占位实现，实际上并没有共享缓冲区。通过实现"真正的" `SharedBuffers` 抽象，我们获得了相当显著的文本渲染速度提升。

#### 绘制"text_debug"示例的帧时间（毫秒，越少越好） [#](https://bevy.org/news/bevy-0-4/#frame-time-to-draw-text-debug-example-in-milliseconds-less-is-better)

![文本渲染](https://bevy.org/news/bevy-0-4/text_rendering.svg)

### 邮箱垂直同步 [#](https://bevy.org/news/bevy-0-4/#mailbox-vsync)

Bevy 现在默认使用 wgpu 的"邮箱垂直同步"。这减少了支持该平台上的输入延迟。

## 反射 [#](https://bevy.org/news/bevy-0-4/#reflection)

作者：@cart

Rust 有一个相当大的"反射"缺口。对于那些不知道的人，"反射"是一类语言特性，使你能够在运行时与语言构造进行交互。它们为传统上静态的语言概念增添了一种"动态性"。

我们在 Rust 中已经有了一些反射的零散部分，比如 [`TypeId`](https://doc.rust-lang.org/stable/std/any/struct.TypeId.html) 和 [`type_name`](https://doc.rust-lang.org/std/any/fn.type_name.html)。但当涉及到与数据类型交互时……我们还没有任何东西。这很不幸，因为有些问题本质上是动态的。

在我最初构建 Bevy 时，我断定引擎会从这些特性中受益。反射是场景系统、Godot 风格（或 Unity 风格）的属性动画系统和编辑器检查工具的良好基础。我构建了 `bevy_property` 和 `bevy_type_registry` crate 来满足这些需求。

它们完成了工作，但它们是针对 Bevy 需求定制的，充满了自定义的术语（而不是直接反映 Rust 语言构造），没有处理 trait，并且对数据访问方式有许多根本性的限制。

在此版本中，我们用新的 [`bevy_reflect`](https://docs.rs/bevy/0.4.0/bevy/reflect/index.html) crate 替换了旧的 `bevy_property` 和 `bevy_type_registry` crate。Bevy Reflect 旨在成为一个"通用"的 Rust 反射 crate。我希望它对非 Bevy 项目和对 Bevy 一样有用。我们现在将其用于场景系统，但在未来，我们将用它来动画化组件字段和自动生成 Bevy 编辑器检查器小部件。

Bevy Reflect 通过派生 [`Reflect`](https://docs.rs/bevy/0.4.0/bevy/reflect/trait.Reflect.html) trait 使你能够动态地与 Rust 类型交互：

```rust
#[derive(Reflect)]
struct Foo {
    a: u32,
    b: Vec<Bar>,
    c: Vec<u32>,
}

#[derive(Reflect)]
struct Bar {
    value: String
}

// 我将使用这个值来演示 `bevy_reflect` 的功能
let mut foo = Foo {
    a: 1,
    b: vec![Bar { value: "hello world" }]
    c: vec![1, 2]
};
```

### 使用字段名称进行交互 [#](https://bevy.org/news/bevy-0-4/#interact-with-fields-using-their-names)

```rust
assert_eq!(*foo.get_field::<u32>("a").unwrap(), 1);

*foo.get_field_mut::<u32>("a").unwrap() = 2;

assert_eq!(foo.a, 2);
```

### 用新值修补你的类型 [#](https://bevy.org/news/bevy-0-4/#patch-your-types-with-new-values)

```rust
let mut dynamic_struct = DynamicStruct::default();
dynamic_struct.insert("a", 42u32);
dynamic_struct.insert("c", vec![3, 4, 5]);

foo.apply(&dynamic_struct);

assert_eq!(foo.a, 42);
assert_eq!(foo.c, vec![3, 4, 5]);
```

### 使用"路径字符串"查找嵌套字段 [#](https://bevy.org/news/bevy-0-4/#look-up-nested-fields-using-path-strings)

```rust
let value = *foo.get_path::<String>("b[0].value").unwrap();
assert_eq!(value.as_str(), "hello world");
```

### 遍历结构体字段 [#](https://bevy.org/news/bevy-0-4/#iterate-over-struct-fields)

```rust
for (i, value: &Reflect) in foo.iter_fields().enumerate() {
    let field_name = foo.name_at(i).unwrap();
    if let Ok(value) = value.downcast_ref::<u32>() {
        println!("{} 是一个 u32，值为：{}", field_name, *value);
    } 
}
```

### 使用 Serde 自动序列化和反序列化 [#](https://bevy.org/news/bevy-0-4/#automatically-serialize-and-deserialize-with-serde)

这不需要手动实现 Serde！

```rust
let mut registry = TypeRegistry::default();
registry.register::<u32>();
registry.register::<String>();
registry.register::<Bar>();

let serializer = ReflectSerializer::new(&foo, &registry);
let serialized = ron::ser::to_string_pretty(&serializer, ron::ser::PrettyConfig::default()).unwrap();

let mut deserializer = ron::de::Deserializer::from_str(&serialized).unwrap();
let reflect_deserializer = ReflectDeserializer::new(&registry);
let value = reflect_deserializer.deserialize(&mut deserializer).unwrap();
let dynamic_struct = value.take::<DynamicStruct>().unwrap();

/// reflect 有自己的 partial_eq 实现
assert!(foo.reflect_partial_eq(&dynamic_struct).unwrap());
```

### Trait 反射 [#](https://bevy.org/news/bevy-0-4/#trait-reflection)

你现在可以在给定的 `&dyn Reflect` 引用上调用一个 trait，而无需知道底层类型！这是一种魔法，在大多数情况下应该避免。但在少数完全必要的情况下，它非常有用：

```rust
#[derive(Reflect)]
#[reflect(DoThing)]
struct MyType {
    value: String,
}

impl DoThing for MyType {
    fn do_thing(&self) -> String {
        format!("{} World!", self.value)
    }
}

#[reflect_trait]
pub trait DoThing {
    fn do_thing(&self) -> String;
}

// 首先，让我们将类型装箱为 Box<dyn Reflect>
let reflect_value: Box<dyn Reflect> = Box::new(MyType {
    value: "Hello".to_string(),
});

/* 
这意味着我们不再直接访问 MyType 或其方法。我们只能在 reflect_value 上调用 Reflect 方法。
如果我们想在我们的类型上调用 `do_thing` 呢？
我们可以使用 reflect_value.get::<MyType>() 进行向下转型，但如果我们编译时不知道类型呢？
*/

// 通常在 Rust 中，到这一步我们就没办法了。让我们用新的反射能力来做些酷的事情！
let mut type_registry = TypeRegistry::default()
type_registry.register::<MyType>();

/*
我们放在 DoThing trait 上的 #[reflect] 属性生成了一个新的 `ReflectDoThing` 结构体，
它实现了 TypeData。这被添加到了 MyType 的 TypeRegistration 中。
*/

let reflect_do_thing = type_registry
    .get_type_data::<ReflectDoThing>(reflect_value.type_id())
    .unwrap();

// 我们可以使用这个生成的类型将我们的 `&dyn Reflect` 引用转换为 `&dyn DoThing` 引用
let my_trait: &dyn DoThing = reflect_do_thing.get(&*reflect_value).unwrap();

// 这意味着我们现在可以调用 do_thing()。魔法！
println!("{}", my_trait.do_thing());
```

## 3D 纹理资源 [#](https://bevy.org/news/bevy-0-4/#3d-texture-assets)

作者：@bonsairobo

Texture 资源现在支持 3D 纹理。新的 `array_texture.rs` 示例演示了如何加载 3D 纹理并从每个"层"采样。

![数组纹理](https://bevy.org/news/bevy-0-4/array_texture.png)

## 日志和性能分析 [#](https://bevy.org/news/bevy-0-4/#logging-and-profiling)

作者：@superdump, @cart

Bevy 终于有了内置的日志功能，现在通过新的 [`LogPlugin`](https://docs.rs/bevy/0.4.0/bevy/log/struct.LogPlugin.html) 默认启用。我们评估了各种日志记录库，最终选择了新的 [`tracing`](https://docs.rs/tracing/latest/tracing/) crate。`tracing` 是一个结构化日志记录器，能很好地处理异步/并行日志记录（非常适合像 Bevy 这样的引擎），并且除了"普通"日志记录之外，还支持性能分析。

[`LogPlugin`](https://docs.rs/bevy/0.4.0/bevy/log/struct.LogPlugin.html) 将每个平台配置为默认记录到相应的后端：桌面端记录到终端，Web 端记录到控制台，以及 Android 端记录到 Android Logs / logcat。我们构建了一个新的 Android `tracing` 后端，因为之前还不存在。

### 日志记录 [#](https://bevy.org/news/bevy-0-4/#logging)

Bevy 的内部插件现在会生成 `tracing` 日志。你可以像这样轻松地向自己的应用逻辑添加日志：

```rust
// 这些在 bevy::prelude::* 中默认导入
trace!("非常嘈杂");
debug!("有助于调试");
info!("默认值得打印的有用信息");
warn!("发生了不好的事情但还不算失败，但值得一提");
error!("某件事失败了");
```

这些行会在终端中产生漂亮的打印日志：

![日志](https://bevy.org/news/bevy-0-4/logs.png)

`tracing` 有大量有用的功能，比如结构化日志记录和过滤。[查看他们的文档了解更多信息。](https://docs.rs/tracing/*/tracing/)

### 性能分析 [#](https://bevy.org/news/bevy-0-4/#profiling)

我们添加了一个选项，可以通过启用 `trace` feature 来为所有 ECS 系统添加"tracing spans"。我们还内置了对 `tracing-chrome` 扩展的支持，这会使 Bevy 以"chrome tracing"格式输出跟踪数据。

如果你使用 `cargo run --features bevy/trace,bevy/trace_chrome` 运行你的应用，你将得到一个 JSON 文件，可以在 Chrome 浏览器中通过访问 `chrome://tracing` URL 打开：

![性能分析](https://bevy.org/news/bevy-0-4/profiling.png)

@superdump 向上游的 `tracing_chrome` 添加了对那些漂亮的"span 名称"的支持。

## HIDPI [#](https://bevy.org/news/bevy-0-4/#hidpi)

作者：@mockersf, @blunted2night, @cart

Bevy 现在正确处理 HIDPI / Retina / 高像素密度显示器：

- 创建窗口时现在会考虑操作系统报告的像素密度。如果一个 Bevy 应用请求在 2x 像素密度显示器上创建一个 1280x720 的窗口，它将创建一个 2560x1440 的窗口。
- 窗口宽度/高度现在以"逻辑单位"报告（上例中的 1280x720）。使用 `window.physical_width()` 和 `window.physical_height()` 方法仍然可以获取物理单位。
- 窗口"交换链"使用物理分辨率创建，以确保我们仍然有清晰的渲染（上例中的 2560x1440）。
- Bevy UI 已经过调整以正确处理 HIDPI 缩放。

这里还有一些工作要做。虽然 Bevy UI 以清晰的 HIDPI 分辨率渲染图像和方框，但文本仍然使用逻辑分辨率渲染，这意味着它在 HIDPI 显示器上可能不如它应该的那样清晰。

## 定时器改进 [#](https://bevy.org/news/bevy-0-4/#timer-improvements)

作者：@amberkowalski, @marcusbuffett, @CleanCut

Bevy 的 Timer 组件/资源获得了一系列生活质量改进：暂停、字段访问方法、人体工程学改进以及内部重构/代码质量改进。Timer 组件也不再默认跳动。Timer 资源和 newtyped Timer 组件无法默认跳动，所以让（相对少见的）"未包装的组件 Timer"自动跳动有点不一致。

Timer API 现在看起来是这样的：

```rust

struct MyTimer {
    timer: Timer,
}

fn main() {
    App::build()
        .add_resource(MyTimer {
            // 一个五秒的非重复定时器
            timer: Timer::from_seconds(5.0, false),
        })
        .add_system(timer_system.system())
        .run();
}

fn timer_system(time: Res<Time>, my_timer: ResMut<MyTimer>) {
    if my_timer.timer.tick(time.delta_seconds()).just_finished() {
        println("五秒已经过去");
    }
}
```

## 任务系统改进 [#](https://bevy.org/news/bevy-0-4/#task-system-improvements)

作者：@aclysma

@aclysma 改变了 Bevy 任务调度的工作方式，这将 `breakout.rs` 示例游戏的性能提升了 **~20%**，并解决了一个当任务池配置为只有一个线程时的[死锁](https://github.com/bevyengine/bevy/pull/892)问题。现在，当只有一个任务要运行时，任务会立即在调用线程上执行，这减少了将工作移动到其他线程/阻塞等待它们完成的开销。

## Apple Silicon 支持 [#](https://bevy.org/news/bevy-0-4/#apple-silicon-support)

作者：@frewsxcv, @wyhaya, @scoopr

得益于 winit（@scoopr）和 coreaudio-sys（@wyhaya）的上游工作，Bevy 现在可以在 Apple Silicon 上运行。@frewsxcv 和 @wyhaya 更新了 Bevy 的依赖项，并验证了它在 Apple 的新芯片上可以构建/运行。

## 新示例 [#](https://bevy.org/news/bevy-0-4/#new-examples)

### Bevy 贡献者 [#](https://bevy.org/news/bevy-0-4/#bevy-contributors)

作者：@karroffel

@karroffel 添加了一个有趣的示例，将每个 Bevy 贡献者表示为一个"Bevy 鸟"。它会从 git 中抓取最新的贡献者列表。

![贡献者](https://bevy.org/news/bevy-0-4/contributors.png)

### BevyMark [#](https://bevy.org/news/bevy-0-4/#bevymark)

作者：@robdavenport

一个"bunnymark 风格"的基准测试，展示了 Bevy 的精灵渲染性能。这在实现上述渲染器优化时非常有用。

![bevymark](https://bevy.org/news/bevy-0-4/bevymark.png)
