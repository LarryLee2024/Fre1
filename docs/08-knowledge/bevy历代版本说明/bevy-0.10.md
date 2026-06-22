# Bevy 0.10

## 发布于 2023 年 3 月 6 日，作者：Bevy 贡献者

![展示 Bevy 新材质混合模式与雾效的废墟场景。基于 Casey Hardy 的原创场景（CC Attribution）](https://bevy.org/news/bevy-0-10/ruins.png)

[展示 Bevy 新材质混合模式与雾效的废墟场景。基于 Casey Hardy 的原创场景（CC Attribution）](https://github.com/coreh/bevy-demo-ruins)

感谢 **173** 位贡献者、**689** 个 pull request、社区审阅者以及我们的[**慷慨赞助商**](https://bevy.org/donate)，我们很高兴在 [crates.io](https://crates.io/crates/bevy) 上发布 **Bevy 0.10**！

对于那些还不了解的人，Bevy 是一个用 Rust 构建的、令人耳目一新的简单数据驱动游戏引擎。你可以查看我们的[快速入门指南](https://bevy.org/learn/quick-start/introduction)来立即尝试。它永久免费且开源！你可以在 GitHub 上获取完整的[源代码](https://github.com/bevyengine/bevy)。查看 [Bevy Assets](https://bevy.org/assets) 获取社区开发的插件、游戏和学习资源合集。

要将现有的 Bevy App 或 Plugin 更新到 **Bevy 0.10**，请查看我们的 [0.9 到 0.10 迁移指南](https://bevy.org/learn/migration-guides/0.9-0.10/)。

自几个月前的上次发布以来，我们添加了大量新功能、Bug 修复和生活质量改进，以下是一些亮点：

- **ECS Schedule v3**：Bevy 现在拥有更简单、更灵活的调度。系统现在存储在统一的调度中，命令可以通过 `apply_system_buffers` 显式应用，还有大量的生活质量改进和 Bug 修复。
- **级联阴影映射**：覆盖更远距离的更高品质阴影映射，质量随相机位置变化。
- **环境映射光照**：基于 360 度环境图像的照明，可以廉价且大幅提升场景的视觉质量。
- **深度和法线预处理**：在主通道之前渲染场景的深度和法线纹理，从而启用新效果（在某些情况下）并提高性能。阴影映射使用预处理着色器，这使得透明纹理可以投射阴影。
- **平滑的骨骼动画过渡**：在两个同时播放的骨骼动画之间平滑过渡！
- **改进的 Android 支持**：Bevy 现已在更多 Android 设备上开箱即用（有一些注意事项）
- **改造的 Bloom**：Bloom 现在看起来更好、更易控制，而且视觉伪影更少。
- **距离雾与大气雾**：使用 3D 距离雾和大气雾效为场景增添深度和氛围！
- **StandardMaterial 混合模式**：通过更多 PBR 材质混合模式实现各种有趣的效果。
- **更多色调映射选择**：从 7 种流行的色调映射算法中选择一种用于你的 HDR 场景，以实现你想要的视觉风格。
- **色彩分级**：控制每个相机的曝光、伽马值、"色调映射前饱和度"和"色调映射后饱和度"。
- **并行管线化渲染**：应用逻辑和渲染逻辑现在自动并行运行，带来显著的性能提升。
- **窗口作为实体**：窗口现在表示为实体而非资源，改善了用户体验并解锁了新的场景。
- **渲染器优化**：这个周期我们在优化渲染器上投入了大量精力。Bevy 的渲染器比以往更敏捷！
- **ECS 优化**：同样，我们大幅提升了许多常见 ECS 操作的性能。Bevy 应用获得了可观的加速！

## ECS Schedule v3 [#](https://bevy.org/news/bevy-0-10/#ecs-schedule-v3)

作者：@alice-i-cecile, @maniwani, @WrongShoe, @cart, @jakobhellermann, @JoJoJet, @geieredgar 以及更多贡献者

感谢我们 ECS 团队的出色工作，备受期待的["无阶段"调度 RFC](https://github.com/bevyengine/rfcs/blob/main/rfcs/45-stageless.md) 已经实现！

**Schedule v3** 是大量设计和实现工作的结晶。调度 API 是 Bevy 开发体验的核心和定义性部分，因此我们必须对这个 API 的下一次演进非常深思熟虑和细致。除了 [RFC PR](https://github.com/bevyengine/rfcs/pull/45)，由 `@maniwani` 提交的[初始实现 PR](https://github.com/bevyengine/bevy/pull/6587) 和由 `@alice-i-cecile` 提交的 [Bevy 引擎内部移植 PR](https://github.com/bevyengine/bevy/pull/7267) 是了解我们流程和设计理由的好起点。众所周知，计划与实施是两回事。我们最终的实现与最初的 RFC 有所不同（是个好的方面）。

这里有大量的变化，但我们非常注意确保现有应用的[迁移路径](https://bevy.org/learn/migration-guides/0.9-0.10/#migrate-engine-to-schedule-v3-stageless)相对简单。别担心！

让我们看看 0.10 带来了什么！

### 统一的调度 [#](https://bevy.org/news/bevy-0-10/#a-single-unified-schedule)

你是否曾经想指定 `system_a` 在 `system_b` 之前运行，却收到了令人困惑的警告，说 `system_b` 因为处于不同的阶段而无法找到？

现在没有了！单个 [`Schedule`](https://docs.rs/bevy/0.10.0/bevy/ecs/schedule/struct.Schedule.html) 中的所有系统现在都存储在一个单一的数据结构中，全局感知正在发生的事情。

这简化了我们的内部逻辑，使你的代码对重构更加健壮，并允许插件作者指定高级不变量（例如"移动必须在碰撞检测之前发生"），而无需将自己锁定在确切的调度位置。

[![main_schedule_diagram](https://bevy.org/news/bevy-0-10/main_schedule_diagram.svg)](https://bevy.org/news/bevy-0-10/main_schedule_diagram.svg)

这张使用 [@jakobhellermann 的 `bevy_mod_debugdump` crate](https://github.com/jakobhellermann/bevy_mod_debugdump) 制作的图表展示了 Bevy 默认调度的简化版本。

### 添加系统 [#](https://bevy.org/news/bevy-0-10/#adding-systems)

[`System`](https://docs.rs/bevy/0.10.0/bevy/ecs/system/trait.System.html)（它们只是[普通的 Rust 函数！](https://github.com/bevyengine/bevy/tree/v0.10.0/crates/bevy_ecs#systems)）是在 Bevy ECS 中定义游戏逻辑的方式。使用 **Schedule v3**，你可以像以前版本一样将系统添加到你的 [`App`](https://docs.rs/bevy/0.10.0/bevy/app/struct.App.html) 中：

```rust
app.add_system(gravity)
```

然而 **Schedule v3** 还有一些新花样！你现在可以一次添加多个系统：

```rust
app.add_systems((apply_acceleration, apply_velocity))
```

默认情况下，Bevy 并行运行系统。在以前的 Bevy 版本中，你这样排序系统：

```rust
app
    .add_system(walk.before(jump))
    .add_system(jump)
    .add_system(collide.after(jump))
```

你仍然可以这样做！但现在你可以使用 `add_systems` 来压缩代码：

```rust
// 整洁多了！
app.add_systems((
    walk.before(jump),
    jump,
    collide.after(jump),
))
```

`before()` 和 `after()` 绝对是有用的工具！不过，得益于新的 `chain()` 函数，现在以特定顺序运行系统变得_容易得多_：

```rust
// 这等价于前面的示例
app.add_systems((walk, jump, collide).chain())
```

`chain()` 将按照系统定义的顺序运行它们。链式调用也与按系统配置配合使用：

```rust
app.add_systems((walk.after(input), jump, collide).chain())
```

### 可配置的 System Set [#](https://bevy.org/news/bevy-0-10/#configurable-system-sets)

在 **Schedule v3** 中，"system set"的概念已被重新定义，以支持对系统的运行和调度方式更自然、更灵活的控制。旧的"system label"概念与"set"概念合并，产生了一个直截了当但功能强大的抽象。

[`SystemSet`](https://docs.rs/bevy/0.10.0/bevy/ecs/schedule/trait.SystemSet.html) 是命名过的系统集合，为其所有成员共享系统配置。相对于 [`SystemSet`](https://docs.rs/bevy/0.10.0/bevy/ecs/schedule/trait.SystemSet.html) 对系统排序，会将该排序应用于该集合中的_所有_系统，同时保留每个系统的独立配置。

让我们直接看看这是什么样子。你这样定义 [`SystemSet`](https://docs.rs/bevy/0.10.0/bevy/ecs/schedule/trait.SystemSet.html)：

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum PhysicsSet {
    Movement,
    CollisionDetection,
}
```

你可以通过调用 [`in_set`](https://docs.rs/bevy/0.10.0/bevy/ecs/schedule/trait.IntoSystemConfig.html#method.in_set) 方法将系统添加到集合中：

```rust
app.add_system(gravity.in_set(PhysicsSet::Movement))
```

你可以将其与上述新的系统功能结合使用：

```rust
app.add_systems(
    (apply_acceleration, apply_velocity)
        .chain()
        .in_set(PhysicsSet::Movement)
)
```

系统可以属于任意数量的集合：

```rust
app.add_system(
    move_player
        .in_set(MoveSet::Player)
        .in_set(PhysicsSet::Movement)
)
```

对集合的配置像这样添加：

```rust
app.configure_set(
    // 在 CollisionDetection 集合中的系统之前运行 Movement 集合中的系统
    PhysicsSet::Movement.before(PhysicsSet::CollisionDetection)
)
```

集合可以嵌套在其他集合内部，这样它们会继承父集合的配置：

```rust
app.configure_set(MoveSet::Enemy.in_set(PhysicsSet::Movement))
```

集合可以被多次配置：

```rust
// 在 PlayerPlugin 中：
app.configure_set(MoveSet::Player.before(MoveSet::Enemy))

// 在 PlayerTeleportPlugin 中
app.configure_set(MoveSet::Player.after(PortalSet::Teleport))
```

关键在于，系统配置是严格累加的：你不能_移除_其他地方添加的规则。这既是"反意大利面条式代码"的考虑，也是"插件隐私"的考虑。当这个规则与 Rust 强大的类型隐私规则结合时，插件作者可以仔细决定需要维护哪些确切的不变量，并在内部重组代码和系统而不破坏使用者。

配置规则_必须彼此兼容_：任何悖论（如系统集合包含自身、一个系统必须同时在集合之前和之后运行、排序循环等）都会导致运行时 panic，并附带有用的错误信息。

### 直接调度排他系统 [#](https://bevy.org/news/bevy-0-10/#directly-schedule-exclusive-systems)

"排他系统"是对整个 ECS [`World`](https://docs.rs/bevy/0.10.0/bevy/ecs/world/struct.World.html) 具有可变直接访问权限的 [`System`](https://docs.rs/bevy/0.10.0/bevy/ecs/system/trait.System.html)。因此，它们不能与其他 [`System`](https://docs.rs/bevy/0.10.0/bevy/ecs/system/trait.System.html) 并行运行。

自从 Bevy 诞生以来，Bevy 开发者就希望能够相对于普通系统来调度排他系统（以及刷新命令）。

现在你可以了！排他系统现在可以像任何其他系统一样被调度和排序。

```rust
app
    .add_system(ordinary_system)
    // 这可以工作！
    .add_system(exclusive_system.after(ordinary_system))
```

这特别强大，因为**命令刷新**（应用系统中排队等待的 [`Commands`](https://docs.rs/bevy/0.10.0/bevy/ecs/system/struct.Commands.html)，用于执行生成和销毁实体等操作）现在只需在 `apply_system_buffers` 排他系统中执行即可。

```rust
app.add_systems(
    (
        // 这个系统产生一些命令
        system_a,
        // 这将应用来自 system_a 的排队命令
        apply_system_buffers,
        // 这个系统将可以访问 system_a 命令的结果
        system_b,
    // 这个链式调用确保上述系统按照
    // 定义的顺序运行
    ).chain()
)
```

不过要小心使用这种模式：很容易最终得到许多顺序不佳的排他系统，造成瓶颈和混乱。

你会用这么强大的能力做什么？我们很想知道！

### 使用 Schedule 管理复杂控制流 [#](https://bevy.org/news/bevy-0-10/#managing-complex-control-flow-with-schedules)

但如果你想对你的 [`Schedule`](https://docs.rs/bevy/0.10.0/bevy/ecs/schedule/struct.Schedule.html) 做一些_奇怪_的事情呢？一些非线性、分支或循环的事情。你应该用什么？

事实证明，Bevy_已经_有一个很好的工具来做这个：在排他系统内部运行的 schedule。这个想法很简单：

1. 构建一个 schedule，存储你想要运行的任何复杂逻辑。
2. 将该 schedule 存储在一个资源中。
3. 在一个排他系统中，执行任何任意的 Rust 逻辑来决定你的 schedule 是否以及如何运行。
4. 临时将 schedule 从 [`World`](https://docs.rs/bevy/0.10.0/bevy/ecs/world/struct.World.html) 中取出，在世界其余部分上运行它以同时修改 schedule 和 world，然后再放回去。

随着新的 [`Schedules`](https://docs.rs/bevy/0.10.0/bevy/ecs/schedule/struct.Schedules.html) 资源和 `world.run_schedule()` API 的加入，它比以往任何时候都更 ✨ 符合人体工程学 ✨。

```rust
// 一个 Schedule！
let mut my_schedule = Schedule::new();
schedule.add_system(my_system);

// 我们新 Schedule 的标签！
#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct MySchedule;

// 一个用于运行此 schedule 的排他系统
fn run_my_schedule(world: &mut World) {
    while very_complex_logic() {
        world.run_schedule(MySchedule);
    }
}

// 见证人体工程学！
app
    .add_schedule(MySchedule, my_schedule)
    .add_system(run_my_schedule);
```

Bevy 在 **Bevy 0.10** 中对五个相当不同的场景使用了这种模式：

1. **启动系统**：这些现在存在于它们自己的 schedule 中，在应用启动时运行一次。
2. **固定时间步长系统**：又是另一个 schedule？！运行此 schedule 的排他系统累积时间，运行一个 while 循环，重复运行 `CoreSchedule::FixedUpdate`，直到所有累积的时间都被消耗完。
3. **进入和退出状态**：大量的 schedule。每个运行进入和退出状态变体逻辑的系统集合都存储在自己的 schedule 中，根据 `apply_state_transitions::<S>` 排他系统中的状态变化来调用。
4. **渲染**：所有渲染逻辑都存储在自己的 schedule 中，以便能够相对于游戏逻辑异步运行。
5. **控制最外层循环**：为了处理"先启动 schedule，再主 schedule"的逻辑，我们将其全部包装在一个最小开销的 `CoreSchedule::Outer` 中，然后在此处将我们的 schedule 作为唯一的排他系统运行。

从 [`CoreSchedule`](https://docs.rs/bevy/0.10.0/bevy/app/enum.CoreSchedule.html) 开始跟踪线索，获取更多信息。

### 运行条件 [#](https://bevy.org/news/bevy-0-10/#run-conditions)

[`System`](https://docs.rs/bevy/0.10.0/bevy/ecs/system/trait.System.html) 可以有任意数量的**运行条件**，它们"只是"返回 `bool` 的系统。如果一个系统的_所有_**运行条件**返回的 `bool` 都是 `true`，系统将运行。否则，系统将在当前 schedule 运行时被跳过：

```rust
// 让我们创建自己的运行条件
fn game_win_condition(query: Query<&Player>, score: Res<Score>) -> bool {
    let player = query.single();
    player.is_alive() && score.0 > 9000
}

app.add_system(win_game.run_if(game_win_condition));
```

**运行条件**还有一些"组合器"操作，感谢 [@JoJoJet](https://github.com/bevyengine/bevy/pull/7547) 和 [@Shatur](https://github.com/bevyengine/bevy/pull/7559)：

它们可以用 `not()` 取反：

```rust
app.add_system(continue_game.run_if(not(game_win_condition)))
```

它们还可以用 `and_then` 和 `or_else` 组合：

```rust
app.add_system(move_player.run_if(is_alive.or_else(is_zombie)))
```

Bevy 0.10 附带了一组可爱的内置[常用运行条件](https://docs.rs/bevy/0.10.0/bevy/ecs/schedule/common_conditions/index.html)。你可以轻松地在有待处理事件、定时器到期、资源变化、输入状态变化、状态变化等情况下运行系统（感谢 [`@maniwani`](https://github.com/bevyengine/bevy/pull/6587)、[`@inodentry`](https://github.com/bevyengine/bevy/pull/7579)、[`@jakobhellermann`](https://github.com/bevyengine/bevy/pull/7806) 和 [`@jabuwu`](https://github.com/bevyengine/bevy/pull/7866)）。

**运行条件**也可以作为一个轻量级的优化工具。运行条件在主线程上评估，每个运行条件在每次 schedule 更新时只评估一次，在集合中依赖它的第一个系统运行时。被运行条件禁用的系统不会生成任务，这可以在多个系统上累积效果。不过，像往常一样：要进行基准测试！

**运行条件**已经取代了以前 Bevy 版本中的"运行标准"。我们终于可以摆脱那可怕的"循环运行标准"了！[`ShouldRun::YesAndCheckAgain`](https://docs.rs/bevy/0.9.1/bevy/ecs/schedule/enum.ShouldRun.html) 对于引擎开发者和用户来说都不太容易理解。当你的类布尔枚举有四个可能值时，这总是一个不好的信号。如果你渴望更复杂的控制流：使用[上一节](https://bevy.org/news/bevy-0-10/#directly-schedule-exclusive-systems)中的"排他系统中的 schedule"模式。对于其他 99% 的用例，享受基于 `bool` 的更简单的运行条件吧！

### 更简单的状态 [#](https://bevy.org/news/bevy-0-10/#simpler-states)

**Schedule v3** 添加了一个新的、更简单的"状态系统"。[`State`](https://docs.rs/bevy/0.10.0/bevy/ecs/schedule/trait.States.html) 允许你轻松配置不同的 [`App`](https://docs.rs/bevy/0.10.0/bevy/app/struct.App.html) 逻辑，使其根据 [`App`](https://docs.rs/bevy/0.10.0/bevy/app/struct.App.html) 的当前"状态"运行。

你这样定义 [`State`](https://docs.rs/bevy/0.10.0/bevy/ecs/schedule/trait.States.html)：

```rust
#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
enum AppState {
    #[default]
    MainMenu,
    InGame,
}
```

枚举的每个变体对应 [`App`](https://docs.rs/bevy/0.10.0/bevy/app/struct.App.html) 可以处于的不同状态。

你这样将 [`State`](https://docs.rs/bevy/0.10.0/bevy/ecs/schedule/trait.States.html) 添加到你的 [`App`](https://docs.rs/bevy/0.10.0/bevy/app/struct.App.html) 中：

```rust
app.add_state::<AppState>()
```

这将设置你的 [`App`](https://docs.rs/bevy/0.10.0/bevy/app/struct.App.html) 使用给定的状态。它会添加 [`State`](https://docs.rs/bevy/0.10.0/bevy/ecs/schedule/struct.State.html) 资源，可以用来查找 [`App`](https://docs.rs/bevy/0.10.0/bevy/app/struct.App.html) 当前所处的状态：

```rust
fn check_state(state: Res<State<AppState>>) {
    info!("我们处于 {} 状态", state.0);
}
```

此外，`add_state` 会为每个可能的值创建一个 `OnUpdate` 集合，然后你可以将系统添加到其中。这些集合作为正常应用更新的一部分运行，但仅在应用处于给定状态时运行：

```rust
app
    .add_systems(
        (main_menu, start_game)
            .in_set(OnUpdate(AppState::MainMenu))
    )
    .add_system(fun_gameplay.in_set(OnUpdate(AppState::InGame)));
```

它还会为每个状态创建 `OnEnter` 和 `OnExit` schedule，这些 schedule 只在从一种状态转换到另一种状态时运行：

```rust
app
    .add_system(load_main_menu.in_schedule(OnEnter(AppState::MainMenu)))
    .add_system(cleanup_main_menu.in_schedule(OnExit(AppState::MainMenu)))
```

`add_state` 还添加了 [`NextState`](https://docs.rs/bevy/0.10.0/bevy/ecs/schedule/struct.NextState.html) 资源，可以用来排队状态更改：

```rust
fn start_game(
    button_query: Query<&Interaction, With<StartGameButton>>,
    mut next_state: ResMut<NextState<AppState>>,
){
    if button_query.single() == Interaction::Pressed {
        next_state.set(AppState::InGame);
    }
}
```

这取代了 Bevy 以前的状态系统，那个系统非常难以处理。它有状态栈、复杂的排队转换和错误处理（大多数人直接 unwrap）。状态栈学习起来非常复杂，极易导致令人沮丧的 Bug，而且大多数情况下被忽略了。

因此，在 **Bevy 0.10** 中，状态现在是"无栈的"：每种类型一次只有一个排队状态。经过大量的 alpha 测试，我们有相当的信心，迁移应该不会太难。如果你依赖于状态栈，你有很多选择：

- 在核心状态系统之上构建"栈"逻辑
- 将你的状态拆分为多个状态，捕获应用状态的正交元素
- 使用与 Bevy 第一方版本相同的模式构建你自己的状态栈抽象。没有任何新的状态逻辑是硬编码的！如果你构建了什么，[让社区的其他成员知道](https://bevy.org/assets)，这样你们可以合作！

### Base Set：让默认行为正确 [#](https://bevy.org/news/bevy-0-10/#base-sets-getting-default-behavior-right)

细心的读者可能会指出：

1. Bevy 自动并行运行其系统。
2. [系统的顺序是非确定性的，除非它们之间有显式的排序关系](https://github.com/bevyengine/bevy/blob/latest/examples/ecs/nondeterministic_system_order.rs)
3. 所有系统现在都存储在一个单一的 `Schedule` 对象中，它们之间没有任何屏障
4. 系统可以属于任意数量的 system set，每个 set 都可以添加自己的行为
5. Bevy 是一个强大的引擎，拥有许多内部系统。

这难道不会导致彻底的混乱和乏味的意大利面条式工作来解决每一个排序歧义吗？许多用户_喜欢_阶段的，它们有助于理解 [`App`](https://docs.rs/bevy/0.10.0/bevy/app/struct.App.html) 的结构！

好吧，我们很高兴你问了，修辞性的怀疑者。为了减少这种混乱（并简化迁移），**Bevy 0.10** 附带了一组全新的由 [`DefaultPlugins`](https://docs.rs/bevy/0.10.0/bevy/struct.DefaultPlugins.html) 提供的 system set：[`CoreSet`](https://docs.rs/bevy/0.10.0/bevy/app/enum.CoreSet.html)、[`StartupSet`](https://docs.rs/bevy/0.10.0/bevy/app/enum.StartupSet.html) 和 [`RenderSet`](https://docs.rs/bevy/0.10.0/bevy/render/enum.RenderSet.html)。它们的名称与旧的 [`CoreStage`](https://docs.rs/bevy/0.9.1/bevy/app/enum.CoreStage.html)、[`StartupStage`](https://docs.rs/bevy/0.9.1/bevy/app/enum.StartupStage.html) 和 [`RenderStage`](https://docs.rs/bevy/0.9.1/bevy/render/enum.RenderStage.html) 的相似性并非巧合。就像阶段一样，每个集合之间有命令刷新点，现有系统已经被直接迁移。

阶段中心架构的某些部分是吸引人的：清晰的高层结构、在刷新点上的协调（以减少过度瓶颈）以及良好的默认行为。为了保留这些部分（同时切除那些令人沮丧的部分），我们引入了 **Base Set** 的概念（[由 @cart 添加](https://github.com/bevyengine/bevy/pull/7466)）。**Base Set** 只是普通的 [`SystemSet`](https://docs.rs/bevy/0.10.0/bevy/ecs/schedule/trait.SystemSet.html)，除了：

1. 每个系统最多可以属于一个 base set。
2. 没有指定 base set 的系统将被添加到调度的默认 base set 中（如果调度有一个的话）。

```rust
// 你像定义普通 set 一样定义 base set，加上 system_set(base) 属性
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
#[system_set(base)]
enum MyBaseSet {
    Early,
    Late,
}

app
    // 默认情况下，这最终会进入 CoreSet::Update
    .add_system(no_explicit_base_set)
    // 必须使用 .in_base_set 而不是 .in_set 以示明确
    // 这是一个高影响的决定！
    .add_system(post_update.in_base_set(CoreSet::PostUpdate))
    // 看，这有效！
    .add_system(custom_base_set.in_base_set(MyBaseSet::Early))
    // 将你的 base set 相对于 CoreSet 排序是明智的
    .configure_set(MyBaseSet::Early.before(CoreSet::Update))
    .configure_set(MyBaseSet::Late.after(CoreSet::Update));
```

让我给你讲一个故事，背景是一个没有 **Base Set** 的世界：

1. 一个新用户将 `make_player_run` 系统添加到他们的应用中。
2. 有时这个系统在输入处理之前运行，导致随机丢失输入。有时它在渲染之后运行，导致奇怪的闪烁。
3. 经过大量的挫折，用户发现这是由于"系统执行顺序歧义"。
4. 用户运行一个专门的检测工具，深入引擎的源代码，找出他们的系统相对于引擎的 system set 应该以什么顺序运行，然后继续他们的快乐之路，为每个新系统都这样做一遍。
5. Bevy（或第三方插件之一）更新，再次打破了我们可怜的用户的系统排序。

这清楚地说明的问题是，_大多数_游戏系统不需要知道或关心"内部系统"。

我们在实践中发现，有三类系统：游戏逻辑（绝大多数终端用户系统）、需要在游戏逻辑之前发生的事情（如事件清理和输入处理）以及需要在游戏逻辑之后发生的事情（如渲染和音频）。

通过 **Base Set** 对调度进行广泛的排序，Bevy 应用可以有良好的默认行为和清晰的高层结构，而不会影响高级用户所渴望的调度灵活性和明确性。让我们知道这对你来说效果如何！

### 改进的系统歧义检测 [#](https://bevy.org/news/bevy-0-10/#improved-system-ambiguity-detection)

当多个系统以冲突的方式访问 ECS 资源，但它们之间没有排序约束时，我们称之为"歧义"。如果你的 [`App`](https://docs.rs/bevy/0.10.0/bevy/app/struct.App.html) 存在歧义，这可能导致 Bug。我们显著改进了歧义报告，可以在新的 [`ScheduleBuildSettings`](https://docs.rs/bevy/0.10.0/bevy/ecs/schedule/struct.ScheduleBuildSettings.html) 中进行配置。查看文档了解更多信息。如果你还没有在你的应用上试过这个：你应该看看！

### 单线程执行 [#](https://bevy.org/news/bevy-0-10/#single-threaded-execution)

现在你可以通过 [`SingleThreadedExecutor`](https://docs.rs/bevy/0.10.0/bevy/ecs/schedule/struct.SingleThreadedExecutor.html) 轻松地将 [`Schedule`](https://docs.rs/bevy/0.10.0/bevy/ecs/schedule/struct.Schedule.html) 切换为单线程评估，适用于不需要或不需要并行性的用户。

```rust
schedule.set_executor_kind(ExecutorKind::SingleThreaded);
```

## 级联阴影映射 [#](https://bevy.org/news/bevy-0-10/#cascaded-shadow-maps)

作者：@danchia, Rob Swain (@superdump)

Bevy 使用"阴影映射"来为光源/物体投射阴影。以前的 Bevy 版本对方向光源使用了一种简单但有限的阴影映射实现。对于一个给定的光源，你需要定义阴影映射的分辨率_以及_一个手动的"视图投影"，来确定阴影的投射方式。这有许多缺点：

- 阴影映射的分辨率是固定的。你必须在"覆盖大范围，但分辨率较低"和"覆盖较小范围，但分辨率较高"之间做出选择。
- 分辨率不适应相机位置。阴影在一个位置可能看起来很棒，但在另一个位置却很糟糕。
- "阴影投影"必须手动定义。这使得配置阴影以适应给定场景变得困难且不易上手。

**Bevy 0.10** 添加了"级联阴影映射"，它将相机的视锥体分解为一系列可配置的"级联"，每个级联都有自己的阴影映射。这使得靠近相机的级联中的阴影高度精细，同时允许远离相机的阴影覆盖更广的区域但细节较少。由于它使用相机的视锥体来定义阴影投影，阴影质量随着相机在场景中的移动而保持一致。这也意味着用户不再需要手动配置阴影投影了。它们是自动计算的！

注意附近的阴影是如何高度精细的，而远处的阴影随着距离变远而变得不太精细（这不太重要，因为它们很远）。

虽然阴影级联解决了重要问题，但也引入了新的问题。你应该使用多少个级联？阴影应该出现的最小和最大距离是多少？级联之间应该有多少重叠？一定要仔细调整这些参数以适应你的场景。

## 环境映射光照 [#](https://bevy.org/news/bevy-0-10/#environment-map-lighting)

作者：@JMS55

环境映射是一种流行且计算廉价的方式，可以显著提升场景照明的质量。它使用立方体贴图纹理来提供 360 度的"来自所有方向"的照明。这对于反射表面尤其明显，但它适用于所有被照明的材质。

这是没有环境映射光照时 PBR 材质的样子：

![环境映射之前](https://bevy.org/news/bevy-0-10/env_map_before.png)

这是有环境映射光照时 PBR 材质的样子：

![环境映射之后](https://bevy.org/news/bevy-0-10/env_map_after.png)

对于需要恒定照明的场景（尤其是室外场景），环境映射是一个很好的解决方案。而且由于环境映射是任意图像，艺术家对场景照明的特性有很大的控制权。

## 深度和法线预处理 [#](https://bevy.org/news/bevy-0-10/#depth-and-normal-prepass)

作者：@icesentry, Rob Swain (@superdump), @robtfm, @JMS55

这个效果使用预处理的深度来找到地面和力场之间的交叉点

Bevy 现在能够运行深度和/或法线预处理。这意味着深度和法线纹理将在主通道之前运行的渲染通道中生成，因此可以在主通道期间使用。这启用了各种特殊效果，如屏幕空间环境光遮蔽、时间抗锯齿等。这些目前正在开发中，应该会在[Bevy 的下一个版本中可用](https://bevy.org/news/bevy-0-10/#what-s-next)。

![边缘检测](https://bevy.org/news/bevy-0-10/edge_detection.png)

在右边的图像中，绿线是在法线纹理中检测到的边缘，蓝线是在深度纹理中检测到的边缘

![边缘检测预处理](https://bevy.org/news/bevy-0-10/edge_detection_prepass.png)

预处理生成的深度和法线纹理

使用预处理本质上意味着将所有内容渲染两次。预处理本身要快得多，因为它比主通道做的工作少得多。预处理的结果可以用来减少主通道中的过度绘制，但如果你的场景没有过度绘制的问题，启用预处理会对性能产生负面影响。有很多方法可以改善这一点，我们将继续朝着这个目标努力。与任何与性能相关的事情一样，确保为你的用例测量一下，看看它是否有帮助。

当处理需要深度或法线纹理的特殊效果时，预处理仍然非常有用，所以如果你想使用它，只需在你的相机上添加 `DepthPrepass` 或 `NormalPrepass` 组件。

## 使用预处理着色器的阴影映射 [#](https://bevy.org/news/bevy-0-10/#shadow-mapping-using-prepass-shaders)

作者：@geieredgar

以前，用于阴影映射的着色器是硬编码的，不了解材质，只知道网格。现在在 **Bevy 0.10** 中，`Material` 的深度预处理着色器被用于阴影映射。这意味着用于 `Material` 的阴影映射的着色器现在是可自定义的！

作为额外的好处，在阴影映射期间 `Material` 信息的可用性意味着我们可以立即启用阿尔法遮罩阴影，使植被能够根据其纹理中的阿尔法值投射阴影，而不仅仅是基于几何体。

![阿尔法遮罩阴影](https://bevy.org/news/bevy-0-10/alpha_mask_shadows.png)

[NVIDIA ORCA Emerald Square 场景](https://developer.nvidia.com/orca/nvidia-emerald-square)（[CC BY-NC-SA 3.0](https://creativecommons.org/licenses/by-nc-sa/3.0/)）启用了阿尔法遮罩阴影支持

## 平滑的骨骼动画过渡 [#](https://bevy.org/news/bevy-0-10/#smooth-skeletal-animation-transitions)

作者：@smessmer

你现在可以在两个（或更多）骨骼动画之间平滑过渡了！

角色模型和动画是来自 Mixamo 的免版税资产。

通过在 [`AnimationPlayer`](https://docs.rs/bevy/0.10.0/bevy/animation/struct.AnimationPlayer.html) 组件上新增的 [`play_with_transition`](https://docs.rs/bevy/0.10/bevy/animation/struct.AnimationPlayer.html#method.play_with_transition) 方法，你现在可以指定一个过渡持续时间，在此期间新动画将与当前播放的动画线性混合，当前动画的权重将在该持续时间内逐渐减少，直到达到 `0.0`。

```rust
#[derive(Component, Default)]
struct ActionTimer(Timer);

#[derive(Component)]
struct Animations {
    run: Handle<AnimationClip>,
    attack: Handle<AnimationClip>,
}

fn run_or_attack(
    mut query: Query<(&mut AnimationPlayer, &mut ActionTimer, &Animations)>,
    keyboard_input: Res<Input<KeyCode>>,
    animation_clips: Res<Assets<AnimationClip>>,
    time: Res<Time>,
) {
    for (mut animation_player, mut timer, animations) in query.iter_mut() {
        // 按下空格键时触发攻击动画
        if keyboard_input.just_pressed(KeyCode::Space) {
            let clip = animation_clips.get(&animations.attack).unwrap();
            // 设置一个定时器，用于何时重新开始跑步动画
            timer.0 = Timer::new(
                Duration::from_secs_f32(clip.duration() - 0.5),
                TimerMode::Once,
            );
            // 将在半秒内过渡到攻击动画
            animation_player
                .play_with_transition(animations.attack.clone(), Duration::from_secs_f32(0.5));
        }
        if timer.0.tick(time.delta()).just_finished() {
            // 攻击动画结束后，重新开始跑步动画
            animation_player
                .play_with_transition(animations.run.clone(), Duration::from_secs_f32(0.5))
                .repeat();
        }
    }
}
```

## 改进的 Android 支持 [#](https://bevy.org/news/bevy-0-10/#improved-android-support)

作者：@mockersf, @slyedoc

![运行 Bevy 的 Android 模拟器](https://bevy.org/news/bevy-0-10/android%20emulator.png)

Bevy 现在在更多设备上的 Android 上可以开箱即用。这是通过等待 [`Resumed`](https://docs.rs/winit/0.28/winit/event/enum.Event.html#variant.Resumed) 事件来创建窗口，而不是在启动时创建，以匹配 Android 上的 [`onResume()`](https://developer.android.com/guide/components/activities/activity-lifecycle#onresume) 回调。

为了遵循关于 [`Suspended`](https://docs.rs/winit/0.28/winit/event/enum.Event.html#variant.Suspended) 事件的建议，Bevy 现在将在收到该事件时退出。这是一个临时解决方案，直到 Bevy 能够在恢复时重新创建渲染资源。

请在你的设备上测试，并报告成功情况或你遇到的问题！在某些带有软件按钮的设备上，触摸位置存在已知问题，因为 winit 没有暴露（[尚未](https://github.com/rust-windowing/winit/issues/2308)）嵌入区域大小，只有内部大小。

由于这使 Bevy 更接近完全支持 Android，不再需要为 Android 和 iOS 分别提供示例。它们已被重新组合为一个["mobile" 示例](https://github.com/bevyengine/bevy/tree/v0.10.0/examples/mobile)，并且说明已更新（[针对 Android](https://github.com/bevyengine/bevy/tree/v0.10.0/examples#android) 和[针对 iOS](https://github.com/bevyengine/bevy/tree/v0.10.0/examples#ios)）。

这是在 iOS 上运行相同示例的样子！

![运行 Bevy 的 iOS 模拟器](https://bevy.org/news/bevy-0-10/ios%20emulator.png)

## 改造的 Bloom [#](https://bevy.org/news/bevy-0-10/#revamped-bloom)

作者：@StarLederer, @JMS55

Bloom 经历了一些重大变化，现在看起来更好、更易控制，而且视觉伪影更少。结合新的色调映射选项，自上一版本以来，bloom 得到了很大改进！

1. 在 Bevy 0.9 中，bloom 看起来是这样的。
2. 将色调映射器切换到像 `AcesFitted` 这样的选项已经是一个很大的改进。
3. 在 Bevy 0.10 中，bloom 现在看起来是这样的。它更加可控，不那么突兀。
4. 要使 bloom 更强，与其提高 `BloomSettings` 的强度，不如将每个立方体的 `emissive` 值加倍。
5. 最后，如果你想要类似于旧算法的更极端的 bloom，你可以将 `BloomSettings::composite_mode` 从 `BloomCompositeMode::EnergyConserving` 改为 `BloomCompositeMode::Additive`。
6. 使用新的 `bloom_3d`（和 `bloom_2d`）示例，在交互式游乐场中探索新的 bloom 设置。

1

![](https://bevy.org/news/bevy-0-10/old_bloom.png)

2

![](https://bevy.org/news/bevy-0-10/old_bloom_aces.png)

3

![](https://bevy.org/news/bevy-0-10/new_bloom.png)

4

![](https://bevy.org/news/bevy-0-10/new_bloom_double_emission.png)

5

![](https://bevy.org/news/bevy-0-10/new_bloom_additive.png)

6

![](https://bevy.org/news/bevy-0-10/bloom_example.png)

## 距离雾与大气雾 [#](https://bevy.org/news/bevy-0-10/#distance-and-atmospheric-fog)

作者：Marco Buono (@coreh)

Bevy 现在可以渲染距离雾和大气雾效果，通过使物体距离视点越远显得越暗，为你的场景带来更强的_深度感_和_氛围感_。

![新的雾效示例展示了不同的雾效模式和参数。](https://bevy.org/news/bevy-0-10/fog.png)

雾效可以通过新的 [`FogSettings`](https://docs.rs/bevy/0.10.0/bevy/pbr/struct.FogSettings.html) 组件对每个相机进行控制。我们特别花了很多精力暴露了几个可调节的参数，让你对雾的外观有完全的艺术控制，包括通过控制雾颜色的阿尔法通道来淡入和淡出雾的能力。

```rust
commands.spawn((
    Camera3dBundle::default(),
    FogSettings {
        color: Color::rgba(0.1, 0.2, 0.4, 1.0),
        falloff: FogFalloff::Linear { start: 50.0, end: 100.0 },
    },
));
```

雾_具体如何_随距离变化的表现在通过 [`FogFalloff`](https://docs.rs/bevy/0.10.0/bevy/pbr/enum.FogFalloff.html) 枚举控制。从固定功能 OpenGL 1.x / DirectX 7 时代以来所有"传统"雾衰减模式都受到支持：

`FogFalloff::Linear` 在 `start` 和 `end` 参数之间以线性方式从 0 增加到 1。（此示例分别使用 0.8 和 2.2 的值。）

11023distancefog intensitystartend

`FogFalloff::Exponential` 根据（反）指数公式增长，由 `density` 参数控制。

11023density = 2density = 1density = 0.5distancefog intensity

`FogFalloff::ExponentialSquared` 根据略微修改的（反）指数平方公式增长，也由 `density` 参数控制。

11023density = 2density = 1density = 0.5distancefog intensity

此外，还提供更复杂的 `FogFalloff::Atmospheric` 模式，通过分别考虑光的 `extinction`（消光）和 `inscattering`（内散射）来提供_更物理精确_的结果。

所有雾模式都通过 `directional_light_color` 和 `directional_light_exponent` 参数支持 [`DirectionalLight`](https://docs.rs/bevy/0.10.0/bevy/pbr/struct.DirectionalLight.html) 的影响，模拟在阳光明媚的室外环境中看到的光线散射效果。

![新的 atmospheric_fog 示例展示了带有大气雾和方向光影响的地形。](https://bevy.org/news/bevy-0-10/atmospheric-fog.png)

由于直接"手动"控制非线性雾衰减参数可能难以正确调优，一些基于[气象能见度](https://en.wikipedia.org/wiki/Visibility)的辅助函数可用，例如 [`FogFalloff::from_visibility()`](https://docs.rs/bevy/0.10.0/bevy/pbr/enum.FogFalloff.html#method.from_visibility)：

```rust
FogSettings {
    // 物体在最多 15 个单位内保持可见（>= 5% 对比度）
    falloff: FogFalloff::from_visibility(15.0),
    ..default()
}
```

雾效是在 PBR 片段着色器上以"前向渲染风格"应用的，而不是作为后处理效果，这使其能够正确处理半透明网格。

大气雾的实现主要基于 Inigo Quilez（Shadertoy 联合创始人、计算机图形学传奇人物）的[这篇精彩文章](https://iquilezles.org/articles/fog/)。_感谢这篇精彩的撰写和灵感！_

## StandardMaterial 混合模式 [#](https://bevy.org/news/bevy-0-10/#standardmaterial-blend-modes)

作者：Marco Buono (@coreh)

[`AlphaMode`](https://docs.rs/bevy/0.10.0/bevy/pbr/enum.AlphaMode.html) 枚举在 **Bevy 0.10** 中得到了扩展，为 [`StandardMaterial`](https://docs.rs/bevy/0.10.0/bevy/pbr/struct.StandardMaterial.html) 带来了_加法混合和乘法混合_的支持。这两种混合模式是"经典"（非基于物理的）计算机图形学工具箱中的基本工具，通常用于实现各种效果。

_展示使用混合模式创建彩色玻璃和火焰效果的演示。_（[源代码](https://github.com/coreh/bevy-demo-ruins)）

此外，通过专用的阿尔法模式，增加了对具有[预乘阿尔法](https://en.wikipedia.org/wiki/Alpha_compositing#Straight_versus_premultiplied)的半透明纹理的支持。

以下是新模式的高层概述：

- [`AlphaMode::Add`](https://docs.rs/bevy/0.10.0/bevy/pbr/enum.AlphaMode.html#variant.Add) — 以加法方式将片段的颜色与它们后面的颜色结合起来（即像光一样），产生**更亮**的结果。适用于火、全息图、鬼魂、激光和其他能量束等效果。在图形软件中也称为_Linear Dodge_。
- [`AlphaMode::Multiply`](https://docs.rs/bevy/0.10.0/bevy/pbr/enum.AlphaMode.html#variant.Multiply) — 以乘法方式将片段的颜色与它们后面的颜色结合起来（即像颜料一样），产生**更暗**的结果。适用于近似部分光透射的效果，如彩色玻璃、窗膜和某些有色液体。
- [`AlphaMode::Premultiplied`](https://docs.rs/bevy/0.10.0/bevy/pbr/enum.AlphaMode.html#variant.Premultiplied) — 行为与 [`AlphaMode::Blend`](https://docs.rs/bevy/0.10.0/bevy/pbr/enum.AlphaMode.html#variant.Blend) 非常相似，但假定颜色通道具有**预乘阿尔法**。可用于避免使用普通阿尔法混合纹理时可能出现的变色"轮廓"伪影，或者利用以下事实巧妙地创建在单个纹理中结合了加法和常规阿尔法混合的材质：对于其他恒定的 RGB 值，`Premultiplied` 在阿尔法值接近 1.0 时更类似 `Blend`，在阿尔法值接近 0.0 时更类似 `Add`。

![新的 blend_modes 示例。](https://bevy.org/news/bevy-0-10/blend-modes.png)

**注意：** 使用新混合模式的网格在现有的 `Transparent3d` 渲染阶段绘制，因此来自 `AlphaMode::Blend` 的_相同 z 排序考虑因素/限制_适用。

## 更多色调映射选择 [#](https://bevy.org/news/bevy-0-10/#more-tonemapping-choices)

作者：@DGriffin91, @JMS55

色调映射是使用"显示渲染变换"（DRT）将原始高动态范围（HDR）信息转换为实际"屏幕颜色"的过程。在以前的 Bevy 版本中，你只有两个色调映射选项：Reinhard Luminance 或者没有。在 **Bevy 0.10** 中，我们添加了大量的选择！

### 无色调映射 [#](https://bevy.org/news/bevy-0-10/#no-tonemapping)

通常不推荐这样做，因为 HDR 光照不适合直接用作颜色。

![无色调映射](https://bevy.org/news/bevy-0-10/tm_none.png)

### Reinhard [#](https://bevy.org/news/bevy-0-10/#reinhard)

一种简单的方法，适应场景中的颜色：`r = color / (1.0 + color)`。大量的色相偏移，亮色不会自然地降低饱和度。原色和间色完全不降低饱和度。

![reinhard](https://bevy.org/news/bevy-0-10/tm_reinhard.png)

### Reinhard Luminance [#](https://bevy.org/news/bevy-0-10/#reinhard-luminance)

一种流行的类似于普通 Reinhard 的方法，结合了亮度。它适应场景中的光量。这就是我们在以前的 Bevy 版本中使用的。它仍然是我们默认的算法，但这将来可能会改变。色相会发生偏移。亮色在整个光谱上几乎不降低饱和度。

![reinhard luminance](https://bevy.org/news/bevy-0-10/tm_reinhard_luminance.png)

### ACES Fitted [#](https://bevy.org/news/bevy-0-10/#aces-fitted)

一种在电影和行业中极其流行的算法（例如：ACES 是 Unreal 的默认色调映射算法）。当人们说"电影感"时，这通常就是他们的意思。

不是中性的，有一种非常特定的审美，有意且戏剧性的色相偏移。亮绿色和红色变成橙色。亮蓝色变成品红色。对比度显著增加。亮色在整个光谱上降低饱和度。

![aces](https://bevy.org/news/bevy-0-10/tm_aces.png)

### AgX [#](https://bevy.org/news/bevy-0-10/#agx)

非常中性。与其他变换相比，图像有些去饱和。几乎没有色相偏移。微妙的[Abney 偏移](https://en.wikipedia.org/wiki/Abney_effect)。[由 Troy Sobotka 创建](https://github.com/sobotka/AgX)

![agx](https://bevy.org/news/bevy-0-10/tm_agx.png)

### Somewhat Boring Display Transform [#](https://bevy.org/news/bevy-0-10/#somewhat-boring-display-transform)

在暗部和中间调中色相偏移很小，但亮部偏移很大。亮色在整个光谱上降低饱和度。介于 Reinhard 和 Reinhard Luminance 之间。概念上类似于 reinhard-jodie。设计为一个折衷方案，如果你想要例如在低光下较好的肤色，但又不能承受重新制作 VFX 以在没有色相偏移的情况下看起来不错。由 Tomasz Stachowiak 创建。

![SomewhatBoringDisplayTransform](https://bevy.org/news/bevy-0-10/tm_sbdt.png)

### TonyMcMapface [#](https://bevy.org/news/bevy-0-10/#tonymcmapface)

非常中性。微妙但有意为之的色相偏移。亮色在整个光谱上降低饱和度。

作者的原话：Tony 是一种为实时应用（如游戏）设计的显示变换。它故意做得平淡，不增加对比度或饱和度，尽可能保持接近输入刺激（在不需要压缩的地方）。输入刺激的亮度等效值被压缩。非线性类似于 Reinhard。颜色色相在压缩过程中被保留，除了有意的 [Bezold–Brücke 偏移](https://en.wikipedia.org/wiki/Bezold%E2%80%93Br%C3%BCcke_shift)。为了避免条带化，采用了选择性去饱和，并注意避免 [Abney 效应](https://en.wikipedia.org/wiki/Abney_effect)。[由 Tomasz Stachowiak 创建](https://github.com/h3r2tic/tony-mc-mapface)

![TonyMcMapface](https://bevy.org/news/bevy-0-10/tm_tonymcmapface.png)

### Blender Filmic [#](https://bevy.org/news/bevy-0-10/#blender-filmic)

Blender 默认的 Filmic 显示变换。有些中性。色相偏移。亮色在整个光谱上降低饱和度。

![Blender Filmic](https://bevy.org/news/bevy-0-10/tm_blender_filmic.png)

## 色彩分级控制 [#](https://bevy.org/news/bevy-0-10/#color-grading-control)

作者：@DGriffin91

我们增加了一些对色彩分级参数的基本控制，如曝光、伽马值、"色调映射前饱和度"和"色调映射后饱和度"。这些可以通过新的 [`ColorGrading`](https://docs.rs/bevy/0.10.0/bevy/render/view/struct.ColorGrading.html) 组件对每个相机进行配置。

### 0.5 曝光 [#](https://bevy.org/news/bevy-0-10/#0-5-exposure)

![0.5 曝光](https://bevy.org/news/bevy-0-10/exposure_005.png)

### 2.25 曝光 [#](https://bevy.org/news/bevy-0-10/#2-25-exposure)

![2.25 曝光](https://bevy.org/news/bevy-0-10/exposure_225.png)

## 并行管线化渲染 [#](https://bevy.org/news/bevy-0-10/#parallel-pipelined-rendering)

作者：@hymm, @james7132

![管线化渲染跟踪](https://bevy.org/news/bevy-0-10/pipelined-rendering-trace.png)

在多线程平台上，**Bevy 0.10** 现在通过并行运行模拟和渲染而显著加快。渲染器在 [Bevy 0.6](https://bevy.org/news/bevy-0-6/#pipelined-rendering-extract-prepare-queue-render) 中已经重新架构以启用此功能，但实际并行运行的最后步骤直到现在才完成。这有一些棘手的工作需要解决。渲染世界有一个必须在主线程上运行的系统，但任务池只能在该世界的线程上运行。因此，当我们将渲染世界发送到另一个线程时，我们需要仍然容纳在主线程上运行的渲染系统。为了实现这一点，我们增加了在世界线程之外还将任务生成到主线程的能力。

![Many Foxes 帧时间直方图](https://bevy.org/news/bevy-0-10/pipelined-rendering-histogram.png)

在测试不同的 Bevy 示例时，性能提升通常在 10% 到 30% 的范围内。从上方的直方图可以看出，"many foxes"压力测试的平均帧时间比以前快了 1.8ms。

要使用管线化渲染，你只需要添加 [`PipelinedRenderingPlugin`](https://docs.rs/bevy/0.10.0/bevy/render/pipelined_rendering/struct.PipelinedRenderingPlugin.html)。如果你正在使用 [`DefaultPlugins`](https://docs.rs/bevy/0.10.0/bevy/struct.DefaultPlugins.html)，那么除了 wasm 之外的所有平台上都会自动为你添加。Bevy 目前不支持 wasm 上的多线程，而此功能需要多线程才能工作。如果你没有使用 [`DefaultPlugins`](https://docs.rs/bevy/0.10.0/bevy/struct.DefaultPlugins.html)，你可以手动添加插件。

## 窗口作为实体 [#](https://bevy.org/news/bevy-0-10/#windows-as-entities)

作者：@aceeri, @Weibye, @cart

在以前的 Bevy 版本中，[`Window`](https://docs.rs/bevy/0.10.0/bevy/window/struct.Window.html) 被表示为 ECS 资源（包含在 `Windows` 资源中）。在 **Bevy 0.10** 中，[`Window`](https://docs.rs/bevy/0.10.0/bevy/window/struct.Window.html) 现在是一个组件（因此窗口被表示为实体）。

这实现了几个目标：

- 打开了在 Bevy 的场景系统中表示 Window 的大门
- 将 `Window` 暴露给 Bevy 强大的 ECS 查询
- 提供了细粒度的每窗口变更检测
- 提高了创建、使用和关闭窗口的可读性/可发现性
- 更改窗口属性的方式在初始化和修改时相同。不再有 `WindowDescriptor` 的麻烦！
- 允许 Bevy 开发者和用户轻松地将新的组件数据附加到窗口

```rust
fn create_window(mut commands: Commands) {
    commands.spawn(Window {
        title: "我的窗口 :D".to_string(),
        ..default()
    });
}

fn modify_windows(mut windows: Query<&mut Window>) {
    for window in &mut windows {
        window.title = "我更改过的窗口！ :D".to_string();
    }
}

fn close_windows(mut commands: Commands, windows: Query<Entity, With<Window>>) {
    for entity in &windows {
        commands.entity(entity).despawn();
    }
}
```

## 渲染器优化 [#](https://bevy.org/news/bevy-0-10/#renderer-optimizations)

作者：@danchia, Rob Swain (@superdump), james7132, @kurtkuehnert, @robfm

Bevy 的渲染器已经到了优化的成熟时机。所以我们优化了它！

在 Bevy 中渲染任何东西时最大的瓶颈是最终的渲染阶段，在这个阶段我们收集渲染世界中的所有数据以向 GPU 发出绘制调用。这里的核心循环非常热，任何额外的开销都是明显的。在 **Bevy 0.10** 中，我们对这个问题投入了大量的精力，从各个角度进行了攻击。总的来说，以下优化应该使渲染阶段比 0.9 中**快 2-3 倍**：

- 在 [@danchia 的 #7639](https://github.com/bevyengine/bevy/pull/7639) 中，我们发现即使是禁用的日志记录也会对热循环产生很大影响，为我们带来了 20-50% 的阶段加速。
- 在 [@james7132 的 #6944](https://github.com/bevyengine/bevy/pull/6944) 中，我们缩小了该阶段涉及的核心数据结构，减少了内存访问，为我们带来了 9% 的加速。
- 在 [@james7132 的 #6885](https://github.com/bevyengine/bevy/pull/6885) 中，我们重新架构了 `PhaseItem` 和 `RenderCommand` 基础设施，以在从 `World` 获取组件数据时合并常见操作，为我们带来了 7% 的加速。
- 在 [@james7132 的 #7053](https://github.com/bevyengine/bevy/pull/7053) 中，我们改变了 `TrackedRenderPass` 的分配模式，以最小化这些循环中的分支，带来了 6% 的加速。
- 在 [@james7132 的 #7084](https://github.com/bevyengine/bevy/pull/7084) 中，我们改变了从 World 获取资源的方式，以最小化该阶段中原子操作的使用，带来了 2% 的加速。
- 在 [@kurtkuehnert 的 #6988](https://github.com/bevyengine/bevy/pull/6988) 中，我们将内部资源 ID 改为使用原子递增计数器而不是 UUID，降低了该阶段中某些分支的比较成本。

另一项正在进行的开发是使渲染阶段能够在多个线程上正确并行化命令编码。继 [@james7132 的 #7248](https://github.com/bevyengine/bevy/pull/7248) 之后，我们现在支持将外部创建的 `CommandBuffer` 导入到渲染图中，这应该允许用户并行编码 GPU 命令并将其导入渲染图。这目前被 wgpu 阻塞，因为 wgpu 在编码渲染通道时会锁定 GPU 设备，但一旦这个问题被解决，我们就应该能够支持并行命令编码。

类似地，我们已经在渲染管线的其他阶段为实现更高的并行性迈出了步伐。`PipelineCache` 是一个几乎所有 Queue 阶段系统都需要可变访问的资源，但也很少需要写入。在 [#7205](https://github.com/bevyengine/bevy/pull/7205) 中，@danchia 将其改为使用内部可变性，以允许这些系统并行化。这还不能完全让该阶段中的每个系统并行化，因为仍然存在一些常见的阻塞因素，但它应该允许不冲突的渲染阶段同时排队命令。

优化不仅仅是关于 CPU 时间！我们还改进了内存使用、编译时间和 GPU 性能。

- 我们还通过 @james7132 将 `ComputedVisibility` 的内存使用减少了 50%。这是通过将内部存储替换为一组位标志而不是多个布尔值来实现的。
- @robfm 还使用了类型擦除作为 [rustc 性能回归](https://github.com/rust-lang/rust/issues/99188) 的变通方法，以确保渲染相关的 crate 具有更好的编译时间，其中一些 crate 的编译**速度提高了 60%**！详细信息见 [#5950](https://github.com/bevyengine/bevy/pull/5950)。
- 在 [#7069](https://github.com/bevyengine/bevy/pull/7069) 中，Rob Swain (@superdump) 减少了 GPU 上使用的活动寄存器数量，以防止寄存器溢出，显著提高了 GPU 端的性能。

最后，我们在特定使用场景上做了一些改进：

- 在 [#6833](https://github.com/bevyengine/bevy/pull/6833) 中，@james7132 通过省略不必要的缓冲区复制，将网格蒙皮的骨骼提取速度提高了 40-50%。
- 在 [#7311](https://github.com/bevyengine/bevy/pull/7311) 中，@james7132 通过将公共计算从热循环中提取出来，将 UI 提取速度提高了 33%。

## 并行化的变换传播和动画运动学 [#](https://bevy.org/news/bevy-0-10/#parallelized-transform-propagation-and-animation-kinematics)

作者：@james7132

变换传播是任何游戏引擎的核心系统之一。如果你移动一个父实体，你期望它的子节点在世界空间中随之移动。Bevy 的变换传播系统恰好是多个系统最大的瓶颈之一：渲染、UI、物理、动画等都必须等到它完成。变换传播必须快速以避免阻塞所有这些系统。在 **Bevy 0.9** 及之前，变换传播一直是单线程的，并且始终需要完整的层级遍历。随着世界变得越来越大，在这个关键瓶颈上花费的时间也越来越多。在 **Bevy 0.10** 中，变换传播利用良好形成的层级的结构来完全在多线程上运行。完整的性能提升完全取决于层级的结构方式和可用的 CPU 核心数量。在我们的测试中，这使我们在 `many_foxes` 基准测试中的变换传播速度**提高了 4 倍**。

如果变换传播可以并行化，那么动画的前向运动学也可以。我们利用相同的有保证的良好形成层级结构来完全并行化播放骨骼动画。我们还启用了一个基本的实体路径缓存查找，以减少系统正在进行的额外查找。总的来说，我们能够在相同的 `many_foxes` 基准测试中使动画播放器系统**快 10 倍**。

结合此版本中的所有其他优化，我们在 `many_foxes` 基准测试中的测试从约 10ms 每帧（约 100 FPS）加速到约 2.3ms 每帧（约 434 FPS），接近 5 倍的加速！

## ECS 优化 [#](https://bevy.org/news/bevy-0-10/#ecs-optimizations)

作者：@james7132, @JoJoJet

ECS 是整个引擎的基础，因此消除 ECS 中的开销会带来引擎级的加速。在 **Bevy 0.10** 中，我们发现了相当多的领域，能够大幅减少开销并提高整个引擎的 CPU 利用率。

在 [#6547](https://github.com/bevyengine/bevy/pull/6547) 中，我们在使用 `Query::for_each` 及其并行变体时启用了[自动向量化](https://en.wikipedia.org/wiki/Automatic_vectorization)。根据引擎编译的目标架构，这可以使查询迭代时间加快 50-87.5%。在 0.11 中，我们可能会将此优化扩展到所有基于 `Iterator::fold` 的迭代器组合器，例如 `Iterator::count`。更多细节请参见[此 PR](https://github.com/bevyengine/bevy/pull/6773)。

在 [#6681](https://github.com/bevyengine/bevy/pull/6681) 中，通过紧密打包实体位置元数据并避免额外的内存查找，我们显著减少了通过 `Query::get` 进行随机查询查找时的开销，在 `Query::get` 和 `World::get` 的开销中减少了高达 43%。

在 [#6800](https://github.com/bevyengine/bevy/pull/6800) 和 [#6902](https://github.com/bevyengine/bevy/pull/6902) 中，我们发现 rustc 可以优化掉跨函数边界的编译时常量分支，将分支从运行时移到编译时，这导致使用 `EntityRef::get`、`EntityMut::insert`、`EntityMut::remove` 及其变体时的开销减少了高达 50%。

在 [#6391](https://github.com/bevyengine/bevy/pull/6391) 中，我们重新设计了 `CommandQueue` 的内部结构以使其对 CPU 缓存更友好，这在编码和应用命令时显示了高达 37% 的加速。

## `SystemParam` 改进 [#](https://bevy.org/news/bevy-0-10/#systemparam-improvements)

作者：@JoJoJet

Bevy ECS 的核心是 `SystemParam`：这些类型，如 `Query` 和 `Res`，决定了系统能做什么和不能做什么。以前，手动创建一个需要实现一组四个不可分离的 trait。在 **Bevy 0.10** 中，我们[使用泛型关联类型](https://github.com/bevyengine/bevy/pull/6865)[将其减少为只有两个 trait](https://github.com/bevyengine/bevy/pull/6919)：`SystemParam` 和 `ReadOnlySystemParam`。

此外，`#[derive(SystemParam)]` 宏也获得了一系列可用的改进：

- **更灵活**：你不再被迫声明你不需要的生命周期。现在允许元组结构体，且 const 泛型不会破坏东西。
- **可封装**：一个长期存在的 Bug 已经被修复，该 Bug 泄露了私有字段的类型。现在，`SystemParam` 可以正确地封装私有的世界数据。
- **无限制**：16 字段的限制已被解除，所以你可以让你的参数像你想要的那样复杂。这对生成的代码最有用。

## 延迟的世界突变 [#](https://bevy.org/news/bevy-0-10/#deferred-world-mutations)

作者：@JoJoJet

你可能知道，当你发送 `Command` 时，它不会立即改变世界。命令存储在系统中，稍后在调度中应用。以这种方式延迟突变有几个好处：

- **最小化世界访问**：与可变查询（和资源）不同，延迟突变没有数据访问冲突，这为使用此模式的系统提供了更好的可并行化性。
- **顺序无关性**：在执行幂等操作（如设置全局标志）时，延迟突变允许你不用担心系统执行顺序。
- **结构性突变**：延迟突变能够以 `Query` 和 `ResMut` 不能的方式改变世界的结构，例如添加组件或生成和销毁实体。

**Bevy 0.10** 通过 `Deferred` 系统参数为此模式添加了一等支持，它接受 [`SystemBuffer`](https://docs.rs/bevy/0.10.0/bevy/ecs/system/trait.SystemBuffer.html) trait 实现。这让你可以创建具有自定义延迟突变行为的系统，同时跳过与 `Commands` 相关的开销！

```rust
/// 延迟发送事件，但可以与其他事件写入者并行运行。
pub struct EventBuffer<E>(Vec<E>);

// `SystemBuffer` trait 控制延迟突变如何应用于世界。
impl<E> SystemBuffer for EventBuffer<E> { ... }

fn my_system(mut events: Deferred<EventBuffer<MyEvent>>) {
    // 排队一个事件，在命令应用时发送。
    events.0.push(MyEvent);
}
```

请注意，此功能应谨慎使用——尽管有潜在的性能优势，不当使用实际上会_降低_性能。每当你进行优化时，确保检查它是否真的加快了速度！

## Ref<T> 查询 [#](https://bevy.org/news/bevy-0-10/#ref-t-queries)

作者：@Guvante, @JoJoJet

自 Bevy 0.1 以来，`Mut<T>` 一直被用于启用变更检测（以及相关类型，如 `ResMut<T>`）。它是一个简单的包装类型，提供对组件的可变访问以及其变更 tick 元数据，在值被修改时自动标记变更。

在 **Bevy 0.10** 中，变更检测家族新增了 `Ref<T>`，即 `Mut<T>` 的不可变变体。像它的可变兄弟一样，它允许你对当前系统外部的变更做出反应。

```rust
use bevy::prelude::*;

fn inspect_changes_system<T: Component + Debug>(q: Query<Ref<T>>) {
    // 遍历类型 `T` 的每个组件并记录其变更状态。
    for val in &q {
        if val.is_changed() {
            println!("值 `{val:?}` 最后在 tick {} 发生变更。", val.last_changed());
        } else {
            println!("值 `{val:?}` 未变更。");
        }
    }
}
```

我们也在逐步弃用 `ChangeTrackers<T>`，这是检查组件变更 tick 的旧方法。此类型将在 Bevy 的下一个版本中移除。

## 三次曲线 [#](https://bevy.org/news/bevy-0-10/#cubic-curves)

作者：@aevyrie

本视频展示了四种三次曲线使用贝塞尔缓动进行平滑动画。曲线本身是白色的，绿色是速度，红色是加速度，蓝色是确定曲线形状的控制点。

为准备 UI 动画和手动调整的动画曲线，三次曲线已添加到 `bevy_math` 中。该实现直接提供了多种曲线，在各种应用中很有用：

- `Bezier`：用户绘制的样条曲线，以及用于 UI 的三次贝塞尔动画缓动——提供了辅助方法，用于如上述视频所示的三次动画缓动。
- `Hermite`：在知道位置和速度的两个时间点之间的平滑插值，如网络预测。
- `Cardinal`：在任意数量的控制点之间轻松插值，自动计算切线；Catmull-Rom 是 Cardinal 样条的一种类型。
- `B-Spline`：加速度连续的运动，特别适用于相机路径，其中速度的平滑变化（加速度）对于防止剧烈抖动很重要。

`CubicGenerator` trait 是公开的，允许你定义自己的自定义样条来生成 `CubicCurve`！

### 性能 [#](https://bevy.org/news/bevy-0-10/#performance)

`CubicCurve` 的位置、速度和加速度可以在任何点求值。无论使用哪种三次曲线，这些求值的性能开销都相同。在现代 CPU 上，这些求值需要 1-2 ns，而动画缓动——这是一个迭代过程——需要 15-20 ns。

## AccessKit 集成到 `bevy_ui` [#](https://bevy.org/news/bevy-0-10/#accesskit-integration-into-bevy-ui)

作者：@ndarilek

游戏是给每个人玩的：它们构建的方式应该反映这一点。无障碍游戏很少见，适当的支持通常是事后的想法，无论是在引擎层面还是在游戏层面。通过在构建 UI 解决方案时考虑到无障碍性，我们希望解决这个问题。

Bevy 已经[加入了 `egui`](https://github.com/emilk/egui/pull/2294)，借助出色的 [AccessKit](https://github.com/AccessKit/accesskit) crate，迈出了跨平台默认无障碍的第一步。据我们所知，这使得 Bevy 成为第一个拥有第一方无障碍支持的通用游戏引擎。

我们已经将 Bevy 的 UI 层级和文本元素暴露给屏幕阅读器和其他辅助设备，由默认启用的新 `bevy_a11y` crate 管理。这最终由新的 [`AccessibilityNode`](https://docs.rs/bevy/0.10.0/bevy/a11y/struct.AccessibilityNode.html) 组件驱动，它与现有的层级结构结合，直接将此信息暴露给 AccessKit，以及 [`Focus`](https://docs.rs/bevy/0.10.0/bevy/a11y/struct.Focus.html) 资源，后者存储拥有键盘焦点的实体。

这里还有很多工作要做：将焦点系统与[手柄驱动的 UI 控件](https://github.com/bevyengine/rfcs/pull/41)解决方案集成，清理数据模型以[确保"默认无障碍"成为现实](https://github.com/bevyengine/bevy/issues/7862)，以及添加对 AccessKit 中剩余功能的支持。

特别感谢 AccessKit 的主要作者 `@mwcampbell` 审阅我们的集成工作，并与我们合作减少了上游的依赖数量，[显著改善了编译时间和最终可执行文件大小](https://github.com/bevyengine/bevy/pull/6874#issuecomment-1440978453)。这在 Linux 上[仍然是一个严峻的挑战](https://github.com/bevyengine/bevy/pull/6874#issuecomment-1432144117)，因此 `accesskit_unix` 功能标志目前[默认禁用](https://github.com/bevyengine/bevy/pull/6874#issuecomment-1433896811)。

## 空间音频 [#](https://bevy.org/news/bevy-0-10/#spatial-audio)

作者：@mockersf, @DGriffin91, @harudagondi, @alice-i-cecile

Bevy 用于音频的库 [`rodio`](https://crates.io/crates/rodio) 包含对空间音频的支持。Bevy 0.10 暴露了基本的空间音频。仍然有一些注意事项，比如没有 HRTF，也没有对 `Emitter` 和 `Listener` 组件的一等支持。

有趣的是，在此特定功能的开发过程中，`@harudagondi` 发现了一个 [Bug](https://github.com/RustAudio/rodio/issues/444)，即在调试或发布模式下运行应用时音频通道会反转。这原来是 `rodio` 的问题，也影响了以前版本的 Bevy。感谢 `@dis-da-moe`，该 Bug 已在[上游修复](https://github.com/RustAudio/rodio/pull/455)。有关音频编程古怪之处和性能问题的有趣细节，请参见链接的 PR。

你的游戏现在可以拥有空间音频了！克隆 `bevy` 仓库并在命令行中运行 `cargo run --example spatial_audio_3d --release` 以查看 Bevy 中 3D 空间音频的展示。

## 自定义音频源 [#](https://bevy.org/news/bevy-0-10/#custom-audio-sources)

作者：@dis-da-moe

Bevy 通过 [`Decodable`](https://docs.rs/bevy_audio/latest/bevy_audio/trait.Decodable.html) trait 支持自定义音频源，但注册到 Bevy app 的方式非常样板化且文档稀少。在 **Bevy 0.10** 中，为 `App` 添加了一个新的扩展 trait，并且 [`Decodable`](https://docs.rs/bevy_audio/latest/bevy_audio/trait.Decodable.html) 的文档得到了极大改善。

因此，不再需要这样做：

```rust
struct MyCustomAudioSource { /* ... */ }

app.add_asset::<MyCustomAudioSource>()
    .init_resource::<Audio<MyCustomAudioSource>>()
    .init_resource::<AudioOutput<MyCustomAudioSource>>()
    .add_system(play_queued_audio_system::<MyCustomAudioSource>.in_base_set(CoreSet::PostUpdate))
```

你只需要这样做：

```rust
app.add_audio_source::<MyCustomAudioSource>()
```

整洁多了！

## ShaderDef 值 [#](https://bevy.org/news/bevy-0-10/#shaderdef-values)

作者：@mockersf

Bevy 的着色器处理器现在支持带值的 ShaderDef，使用新的 [`ShaderDefVal`](https://docs.rs/bevy/0.10.0/bevy/render/render_resource/enum.ShaderDefVal.html)。这允许开发者将常数值传递到他们的着色器中：

```rust
let shader_defs = vec![
    ShaderDefVal::Int("MAX_DIRECTIONAL_LIGHTS".to_string(), 10),
];
```

这些可以在 `#if` 语句中使用，根据值选择性地启用着色器代码：

```rust
#if MAX_DIRECTIONAL_LIGHTS >= 10
let color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
#else
let color = vec4<f32>(0.0, 1.0, 0.0, 1.0);
#endif
```

ShaderDef 值可以被内联到着色器中：

```rust
for (var i: u32 = 0u; i < #{MAX_DIRECTIONAL_LIGHTS}; i = i + 1u) {
}
```

它们也可以在着色器中内联定义：

```rust
#define MAX_DIRECTIONAL_LIGHTS 10
```

在着色器中定义的 ShaderDef 会覆盖从 Bevy 传入的值。

## 着色器中的 `#else ifdef` 链 [#](https://bevy.org/news/bevy-0-10/#else-ifdef-chains-in-shaders)

作者：@torsteingrindvik

Bevy 的着色器处理器现在也支持像这样的 `#else ifdef` 链：

```rust
#ifdef FOO
// foo 代码
#else ifdef BAR
// bar 代码
#else ifdef BAZ
// baz 代码
#else
// 回退代码
#endif
```

## 新的着色器导入：Global 和 View [#](https://bevy.org/news/bevy-0-10/#new-shader-imports-global-and-view)

作者：@torsteingrindvik

`Global` 和 `View` 结构体现在可以在着色器中使用 `#import bevy_render::globals` 和 `#import bevy_render::view` 导入。Bevy 的内部着色器现在使用这些导入（省去了大量冗余）。以前，你要么需要在每个着色器中重新定义，要么导入更大的 `bevy_pbr::mesh_view_types`（这并不总是需要的）。

以前需要这样：

```rust
struct View {
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    projection: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
    world_position: vec3<f32>,
    // viewport(x_origin, y_origin, width, height)
    viewport: vec4<f32>,
};
```

现在你只需这样做！

```rust
#import bevy_render::view
```

## 并行查询迭代的自适应批处理 [#](https://bevy.org/news/bevy-0-10/#adaptive-batching-for-parallel-query-iteration)

作者：@james7132

`Query::par_for_each` 一直是每个人在查询变得太大而无法单线程运行时使用的工具。屏幕上跑着 100,000 个实体？没问题，`Query::par_for_each` 将其分成更小的批次，并将工作负载分配到多个线程上。然而，在 **Bevy 0.9** 及之前，`Query::par_for_each` 要求调用者提供一个批次大小，以帮助调整这些批次以获得最佳性能。这个相当不透明的调节旋钮往往导致用户随意选择一个值然后用下去，或者根据他们的开发机器微调这个值。不幸的是，最有效的值取决于运行时环境（即玩家的计算机有多少个逻辑核心）和 ECS World 的状态（即匹配了多少个实体？）。最终，大多数 API 用户只是选择一个固定值并接受结果，无论好坏。

```rust
// 0.9
const QUERY_BATCH_SIZE: usize = 32;

query.par_for_each(QUERY_BATCH_SIZE, |mut component| {
   // ...
});
```

在 0.10 中，你不再需要提供批次大小了！如果你使用 [`Query::par_iter`](https://docs.rs/bevy/0.10.0/bevy/ecs/system/struct.Query.html#method.par_iter)，Bevy 将自动评估 World 和任务池的状态，并[使用启发式算法](https://github.com/bevyengine/bevy/blob/43ea6f239deefd7a497da6ef581a05a63a278605/crates/bevy_ecs/src/query/par_iter.rs#L24)选择一个批次大小，以确保足够的并行性，而不会产生太多开销。这使得并行查询像普通的单线程查询一样易于使用！虽然这对大多数典型用例很好，但这些启发式算法可能不适用于每个工作负载，因此我们为那些需要更精细控制工作负载分布的人提供了一个逃生舱口。在未来，我们可能会进一步调整底层的启发式算法，以试图让默认值在这些工作负载中更接近最优。

```rust
// 0.10
query.par_iter().for_each(|component| {
   // ...
});

// 从单线程 for_each 转换非常容易。只需将 iter 改为 par_iter 即可！
query.iter().for_each(|component| {
   // ...
});
```

你也可以使用 [`BatchingStrategy`](https://docs.rs/bevy/0.10.0/bevy/ecs/query/struct.BatchingStrategy.html) 来更精细地控制批处理：

```rust
query
    .par_iter_mut()
    // 以 100 为一批运行
    .batching_strategy(BatchingStrategy::fixed(100))
    .for_each(|mut component| { /* ... */ });
```

更多信息请参见 [`BatchingStrategy`](https://docs.rs/bevy/0.10.0/bevy/ecs/query/struct.BatchingStrategy.html) 文档。

## `UnsafeWorldCell` 和 `UnsafeEntityCell` [#](https://bevy.org/news/bevy-0-10/#unsafeworldcell-and-unsafeentitycell)

作者：@jakobhellermann, @BoxyUwU 和 @JoJoJet

`UnsafeWorldCell` 和 `UnsafeEntityCell` 允许通过 unsafe 代码对世界部分进行共享可变访问。它的目的类似于 `UnsafeCell`，允许人们构建内部可变性抽象，如 `Cell`、`Mutex`、`Channel` 等。在 Bevy 中，`UnsafeWorldCell` 将用于支持调度器和系统参数实现，因为它们是 `World` 的内部可变性抽象，它目前也用于实现 `WorldCell`。我们计划使用 `UnsafeEntityCell` 来实现 `EntityRef`/`EntityMut` 的版本，这些版本只访问实体上的组件，而不是整个世界。

这些抽象是在 [#6404](https://github.com/bevyengine/bevy/pull/6404)、[#7381](https://github.com/bevyengine/bevy/pull/7381) 和 [#7568](https://github.com/bevyengine/bevy/pull/7568) 中引入的。

## 圆柱体形状 [#](https://bevy.org/news/bevy-0-10/#cylinder-shape)

作者：@JayPavlina, @rparrett, @davidhof

圆柱体形状原语已加入我们内置形状的动物园！

![基本形状](https://bevy.org/news/bevy-0-10/primitive_shapes.png)

## 可细分的平面形状 [#](https://bevy.org/news/bevy-0-10/#subdividable-plane-shape)

作者：@woodroww

Bevy 的 [`Plane`](https://docs.rs/bevy/0.10.0/bevy/prelude/shape/struct.Plane.html) 形状现在可以被任意次细分。

![平面](https://bevy.org/news/bevy-0-10/plane.png)

## 相机输出模式 [#](https://bevy.org/news/bevy-0-10/#camera-output-modes)

作者：@cart, @robtfm

[Bevy 0.9 中新增的](https://bevy.org/news/bevy-0-9/#hdr-post-processing-tonemapping-and-bloom)[相机驱动的](https://bevy.org/news/bevy-0-8/#camera-driven-rendering)后处理功能提供了对场景中多个相机的后处理的直观控制，但[有一些边缘情况](https://github.com/bevyengine/bevy/pull/7490)并不_完全_适合硬编码的相机输出模型。而且存在一些 Bug 和限制，与双缓冲目标纹理源真理在相机间不一致，以及 MSAA 的采样纹理在某些情况下不包含应有的内容有关。

**Bevy 0.10** 向 [`Camera`](https://docs.rs/bevy/0.10.0/bevy/render/camera/struct.Camera.html) 添加了一个 [`CameraOutputMode`](https://docs.rs/bevy/0.10.0/bevy/render/camera/enum.CameraOutputMode.html) 字段，使 Bevy 应用开发者能够手动配置 [`Camera`](https://docs.rs/bevy/0.10.0/bevy/render/camera/struct.Camera.html) 的渲染结果应该（以及是否）写入最终输出纹理：

```rust
// 配置相机写入最终输出纹理
camera.output_mode = CameraOutputMode::Write {
    // 不与输出纹理的当前状态混合
    blend_state: None,
    // 清除输出纹理
    color_attachment_load_op: LoadOp::Clear(Default::default()),
};

// 配置相机跳过写入最终输出纹理
// 当有多个相机时，这可以节省一个通道，并且对一些后处理场景也很有用
camera.output_mode = CameraOutputMode::Skip;
```

_大多数_单相机和多相机设置完全不需要触碰这个设置。但如果你需要它，它等着你！

MSAA 需要一个额外的中间"多重采样"纹理，它会解析为"实际"的非采样纹理。在某些渲染到同一纹理的多相机设置边缘情况下，基于 MSAA 是启用还是禁用，这可能会产生奇怪/不一致的结果。我们添加了一个新的 `Camera::msaa_writeback` `bool` 字段，当启用时（如果之前的相机在某一帧已经渲染到目标），它会将非采样纹理的当前状态写入中间 MSAA 纹理。这确保了无论 MSAA 配置如何，状态都是一致的。默认为 true，所以你只需要在多相机设置且你_不_想要 MSAA 回写时才需要考虑这个。

## 可配置的可见性组件 [#](https://bevy.org/news/bevy-0-10/#configurable-visibility-component)

作者：@ickk

[`Visibility`](https://docs.rs/bevy/0.10.0/bevy/render/view/enum.Visibility.html) 组件控制 [`Entity`](https://docs.rs/bevy/0.10.0/bevy/ecs/entity/index.html) 是否应该被渲染。**Bevy 0.10** 重新设计了类型定义：不再是单一的 `is_visible: bool` 字段，我们现在使用一个枚举，包含额外的模式：

```rust
pub enum Visibility {
  Hidden,    // 无条件隐藏
  Visible,   // 无条件可见
  Inherited, // 从父级继承可见性
}
```

更容易理解了！在以前的 Bevy 版本中，"继承可见性"和"隐藏"基本上是仅有的两个选项。现在实体可以选择可见，即使其父级是隐藏的！

## `AsBindGroup` 存储缓冲区 [#](https://bevy.org/news/bevy-0-10/#asbindgroup-storage-buffers)

作者：@IceSentry, @AndrewB330

[`AsBindGroup`](https://docs.rs/bevy/0.10.0/bevy/render/render_resource/trait.AsBindGroup.html) 是一个有用的 Bevy trait，它[使得将数据传递到着色器变得非常容易](https://bevy.org/news/bevy-0-8/#new-material-system)。

**Bevy 0.10** 扩展了这一点，增加了对"存储缓冲区绑定"的支持，这在传入大量/无界数据块时非常有用：

```rust
#[derive(AsBindGroup)]
struct CoolMaterial {
    #[uniform(0)]
    color: Color,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Handle<Image>,
    #[storage(3)]
    values: Vec<f32>,
    #[storage(4, read_only, buffer)]
    buffer: Buffer,
}
```

## `ExtractComponent` 派生 [#](https://bevy.org/news/bevy-0-10/#extractcomponent-derive)

作者：@torsteingrindvik

为了将组件数据从"主应用"传递到"渲染应用"以进行[管线化渲染](https://bevy.org/news/bevy-0-10/#parallel-pipelined-rendering)，我们运行一个"提取步骤"。[`ExtractComponent`](https://docs.rs/bevy/0.10.0/bevy/render/extract_component/trait.ExtractComponent.html) trait 用于复制数据。在以前的 Bevy 版本中，你必须手动实现它，但现在你可以派生它了！

```rust
#[derive(Component, Clone, ExtractComponent)]
pub struct Car {
    pub wheels: usize,
}
```

这会展开为：

```rust
impl ExtractComponent for Car
{
    type Query = &'static Self;
    type Filter = ();
    type Out = Self;
    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(item.clone())
    }
}
```

它也支持过滤器！

```rust
#[derive(Component, Clone, ExtractComponent)]
#[extract_component_filter(With<Fuel>)]
pub struct Car {
    pub wheels: usize,
}
```

## 升级 wgpu 到 0.15 [#](https://bevy.org/news/bevy-0-10/#upgraded-wgpu-to-0-15)

作者：@Elabajaba

**Bevy 0.10** 现在使用了最新最好的 [`wgpu`](https://github.com/gfx-rs/wgpu)（我们选择的底层图形层）。除了[许多不错的 API 改进和 Bug 修复](https://github.com/gfx-rs/wgpu/releases/tag/v0.15.0)之外，`wgpu` 现在还对 DX12 使用 DXC 着色器编译器，这更快、更少 Bug，并允许新功能。

## 默认启用 OpenGL 后端 [#](https://bevy.org/news/bevy-0-10/#enabled-opengl-backend-by-default)

作者：@wangling12

Bevy 已经支持了 `wgpu` 的 OpenGL 后端一段时间了，但它是可选加入的。这导致 Bevy 在一些不支持现代 API（如 Vulkan）的机器上无法启动。在 **Bevy 0.10** 中，OpenGL 后端默认启用，这意味着如果没有其他 API 可用，机器将自动回退到 OpenGL。

## 暴露的非均匀索引支持（无绑定） [#](https://bevy.org/news/bevy-0-10/#exposed-non-uniform-indexing-support-bindless)

作者：@cryscan

**Bevy 0.10** 连接了对纹理和存储缓冲区的非均匀索引的初始支持。这是迈向现代["无绑定/GPU 驱动渲染"](https://vkguide.dev/docs/gpudriven/gpu_driven_engines/)的重要一步，可以在支持的平台上解锁显著的性能提升。请注意，这只是使该功能对渲染插件开发者可用。Bevy 的核心渲染功能并未（尚未）使用无绑定方法。

我们添加了一个[新示例](https://github.com/bevyengine/bevy/blob/v0.10.0/examples/shader/texture_binding_array.rs)来说明如何使用此功能：

![纹理绑定数组](https://bevy.org/news/bevy-0-10/texture_binding_array.png)

## 手柄 API 改进 [#](https://bevy.org/news/bevy-0-10/#gamepad-api-improvements)

作者：@DevinLeamy

[`GamepadEventRaw`](https://docs.rs/bevy/0.9.0/bevy/input/gamepad/struct.GamepadEventRaw.html) 类型已被移除，取而代之的是独立的 [`GamepadConnectionEvent`](https://docs.rs/bevy/0.10.0/bevy/input/gamepad/struct.GamepadConnectionEvent.html)、[`GamepadAxisChangedEvent`](https://docs.rs/bevy/0.10.0/bevy/input/gamepad/struct.GamepadAxisChangedEvent.html) 和 [`GamepadButtonChangedEvent`](https://docs.rs/bevy/0.10.0/bevy/input/gamepad/struct.GamepadButtonChangedEvent.html)，并且内部实现已经重新设计以适应这一点。

这允许更简单、更细粒度的事件访问，而无需过滤通用的 [`GamepadEvent`](https://docs.rs/bevy/0.10.0/bevy/input/gamepad/enum.GamepadEvent.html) 类型。很好！

```rust
fn system(mut events: EventReader<GamepadConnectionEvent>)
    for event in events.iter() {
    }
}
```

## 输入法编辑器（IME）支持 [#](https://bevy.org/news/bevy-0-10/#input-method-editor-ime-support)

作者：@mockersf

[`Window`](https://docs.rs/bevy/0.10.0/bevy/window/struct.Window.html) 现在可以使用 `ime_enabled` 和 `ime_position` 配置 IME 支持，这启用了"死键"的使用，增加了对法语、拼音等的支持：

## 反射路径：枚举和元组 [#](https://bevy.org/news/bevy-0-10/#reflection-paths-enums-and-tuples)

作者：@MrGVSV

Bevy 的"反射路径"允许使用简单（且动态）的字符串语法来导航 Rust 值。**Bevy 0.10** 通过为反射路径中的元组和枚举添加支持来扩展该系统：

```rust
#[derive(Reflect)]
struct MyStruct {
  data: Data,
  some_tuple: (u32, u32),
}

#[derive(Reflect)]
enum Data {
  Foo(u32, u32),
  Bar(bool)
}

let x = MyStruct {
  data: Data::Foo(123),
  some_tuple: (10, 20),
};

assert_eq!(*x.path::<u32>("data.1").unwrap(), 123);
assert_eq!(*x.path::<u32>("some_tuple.0").unwrap(), 10);
```

## 预解析的反射路径 [#](https://bevy.org/news/bevy-0-10/#pre-parsed-reflection-paths)

作者：@MrGVSV, @james7132

反射路径启用了许多有趣的动态编辑器场景，但它们确实有一个缺点：调用 `path()` 每次都需要解析字符串。为了解决这个问题，我们添加了 [`ParsedPath`](https://docs.rs/bevy/0.10.0/bevy/reflect/struct.ParsedPath.html)，它可以预先解析路径，然后在每次访问时重用这些结果：

```rust
let parsed_path = ParsedPath::parse("foo.bar[0]").unwrap();
let element = parsed_path.element::<usize>(&some_value);
```

更适合重复访问，例如每帧进行相同的查找！

## `ReflectFromReflect` [#](https://bevy.org/news/bevy-0-10/#reflectfromreflect)

作者：@MrGVSV

当使用 Bevy 的 Rust 反射系统时，我们有时会遇到这样的情况：我们有一个表示某种类型 `MyType` 的"动态反射值"（尽管在底层，它不完全是那个类型）。当我们调用 `Reflect::clone_value`、使用反射反序列化器或自己创建动态值时，就会出现这样的情况。不幸的是，我们不能直接调用 `MyType::from_reflect`，因为我们在运行时不知道具体的 `MyType`。

[`ReflectFromReflect`](https://docs.rs/bevy/0.10.0/bevy/reflect/struct.ReflectFromReflect.html) 是 [`TypeRegistry`](https://docs.rs/bevy/0.10.0/bevy/reflect/struct.TypeRegistry.html) 中一个新的"类型数据"结构体，它允许在没有任何对给定类型的具体引用的情况下进行 `FromReflect` trait 操作。非常酷！

```rust
#[derive(Reflect, FromReflect)]
#[reflect(FromReflect)] // <- 注册 `ReflectFromReflect`
struct MyStruct(String);

let type_id = TypeId::of::<MyStruct>();

// 注册我们的类型
let mut registry = TypeRegistry::default();
registry.register::<MyStruct>();

// 创建一个具体的实例
let my_struct = MyStruct("Hello world".to_string());

// `Reflect::clone_value` 将为元组结构体类型生成一个 `DynamicTupleStruct`
// 注意，这_不是_一个 MyStruct 实例
let dynamic_value: Box<dyn Reflect> = my_struct.clone_value();

// 从 registry 获取 `ReflectFromReflect` 类型数据
let rfr: &ReflectFromReflect = registry
  .get_type_data::<ReflectFromReflect>(type_id)
  .unwrap();

// 在我们的动态值上调用 `FromReflect::from_reflect`
let concrete_value: Box<dyn Reflect> = rfr.from_reflect(&dynamic_value);
assert!(concrete_value.is::<MyStruct>());
```

## 其他反射改进 [#](https://bevy.org/news/bevy-0-10/#other-reflection-improvements)

作者：@james7132, @soqb, @cBournhonesque, @SkiFire13

- [`Reflect`](https://docs.rs/bevy/0.10.0/bevy/reflect/trait.Reflect.html) 现在已为 [`std::collections::VecDeque`](https://doc.rust-lang.org/std/collections/vec_deque/struct.VecDeque.html) 实现
- 反射后的 [`List`](https://docs.rs/bevy/0.10.0/bevy/reflect/trait.List.html) 类型现在有 `insert` 和 `remove` 操作
- 反射后的 [`Map`](https://docs.rs/bevy/0.10.0/bevy/reflect/trait.Map.html) 类型现在有 `remove` 操作
- 反射后的泛型类型现在如果泛型也实现了 Reflect，则会自动实现 [`Reflect`](https://docs.rs/bevy/0.10.0/bevy/reflect/trait.Reflect.html)。不再需要手动添加 `T: Reflect` 约束！
- 组件反射现在使用 [`EntityRef`](https://docs.rs/bevy/0.10.0/bevy/ecs/world/struct.EntityRef.html) / [`EntityMut`](https://docs.rs/bevy/0.10.0/bevy/ecs/world/struct.EntityMut.html) 而不是同时使用 [`World`](https://docs.rs/bevy/0.10.0/bevy/ecs/world/struct.World.html) 和 [`Entity`](https://docs.rs/bevy/0.10.0/bevy/ecs/entity/index.html)，这使其可以在更多场景中使用
- 反射反序列化器现在在某些场景中避免不必要的克隆字符串！

## 升级 Taffy 到 0.3 [#](https://bevy.org/news/bevy-0-10/#upgraded-taffy-to-0-3)

作者：@ickshonpe, @rparret

[Taffy](https://crates.io/crates/taffy) 是我们用来计算 `bevy_ui` 布局的库。Taffy 0.2 显著提升了嵌套 UI 的性能（我们的 `many_buttons` 示例现在快了 8%，更深嵌套的 UI 应该能看到更大的提升！）。它还带来了对 [gap](https://developer.mozilla.org/en-US/docs/Web/CSS/gap) 属性的支持，这使得创建具有均匀间隔项目的 UI 更加容易。Taffy 0.3 添加了一些不错的 API 调整（还有一个网格布局功能，我们目前禁用了它，因为它仍然需要一些集成工作）。

## 相对鼠标位置 [#](https://bevy.org/news/bevy-0-10/#relative-cursor-position)

作者：@Pietrek14

我们添加了一个新的 [`RelativeCursorPosition`](https://docs.rs/bevy/0.10.0/bevy/ui/struct.RelativeCursorPosition.html) UI 组件，当添加到 UI 实体时，它会跟踪鼠标相对于节点的位置。`Some((0, 0))` 表示节点的左上角，`Some((1,1))` 表示节点的右下角，`None` 表示鼠标"在节点外部"。

```rust
commands.spawn((
    NodeBundle::default(),
    RelativeCursorPosition::default(),
));
```

## Const Bevy UI 默认值 [#](https://bevy.org/news/bevy-0-10/#const-bevy-ui-defaults)

作者：@james-j-obrien

Bevy 大量使用 [`Default`](https://doc.rust-lang.org/std/default/trait.Default.html) trait 来简化类型的构造。Bevy UI 类型通常实现 [`Default`](https://doc.rust-lang.org/std/default/trait.Default.html)。然而，它有一个缺点（这对 Rust 来说是根本性的）：[`Default`](https://doc.rust-lang.org/std/default/trait.Default.html) 不能在 `const` 上下文中使用（[目前还不能！](https://blog.rust-lang.org/inside-rust/2022/07/27/keyword-generics.html)）。为了允许 UI 布局配置被定义为常量，我们为大多数 Bevy UI 类型添加了 `DEFAULT` 关联常量。例如，你可以使用 `Style::DEFAULT` 来定义一个 const 样式：

```rust
const COOL_STYLE: Style = Style {
    size: Size::width(Val::Px(200.0)),
    border: UiRect::all(Val::Px(2.0)),
    ..Style::DEFAULT
};
```

## 迭代世界的实体 [#](https://bevy.org/news/bevy-0-10/#iterating-through-a-world-s-entities)

作者：@james7132

在 **Bevy 0.9** 中，`World::iter_entities` 允许用户以 `Entity` 的形式获得一个迭代器，遍历 `World` 中的所有实体。在 **Bevy 0.10** 中，这已被更改为 `EntityRef` 的迭代器，它提供对所有实体组件的完全只读访问，而不仅仅是获取其 ID。它的新实现也应该比手动获取 `EntityRef` 快得多（但请注意，如果你知道要查找的确切组件，`Query` 仍然会更快）。这使用户可以自由地从 World 中任意读取任何实体数据，并可能在脚本语言集成和反射密集型工作流中看到用途。

```rust
// Bevy 0.9
for entity in world.iter_entities() {
   if let Some(entity_ref) = world.get_entity(entity) {
      if let Some(component) = entity_ref.get::<MyComponent>() {
         ...
      }
   }
}

// Bevy 0.10
for entity_ref in world.iter_entities() {
   if let Some(component) = entity_ref.get::<MyComponent>() {
      ...
   }
}
```

在未来，我们可能会有 `World::iter_entities_mut` 来暴露此功能，但提供对 `World` 中所有实体的任意可变访问。由于返回 `EntityMut` 的迭代器存在潜在的安全问题，我们暂时避免实现这一点。更多细节请参见此 [GitHub issue](https://github.com/bevyengine/bevy/issues/5504)。

## LCH 色彩空间 [#](https://bevy.org/news/bevy-0-10/#lch-color-space)

作者：@ldubos

Bevy 的 [`Color`](https://docs.rs/bevy/0.10.0/bevy/render/color/enum.Color.html) 类型现在支持 LCH 色彩空间（Lightness 亮度、Chroma 彩度、Hue 色相）。LCH 有很多支持理由，包括它比 sRGB 能访问大约多 50% 的颜色。查看[这篇文章](https://lea.verou.me/2020/04/lch-colors-in-css-what-why-and-how/)了解更多信息。

```rust
Color::Lcha {
    lightness: 1.0,
    chroma: 0.5,
    hue: 200.0,
    alpha: 1.0,
}
```

## 优化的 `Color::hex` 性能 [#](https://bevy.org/news/bevy-0-10/#optimized-color-hex-performance)

作者：@wyhaya

[`Color::hex`](https://docs.rs/bevy/0.10.0/bevy/render/color/enum.Color.html#method.hex) 现在是一个 `const` 函数，将 `hex` 的运行时间从约 14ns 降低到约 4ns！

## 拆分的 `CorePlugin` [#](https://bevy.org/news/bevy-0-10/#split-up-coreplugin)

作者：@targrub

`CorePlugin` 历史上一直有点像一个"厨房水槽插件"。"核心"的东西无处可去就最终到了这里。这不是一个好的组织策略，所以我们将其拆分为独立的模块：[`TaskPoolPlugin`](https://docs.rs/bevy/0.10.0/bevy/core/struct.TaskPoolPlugin.html)、[`TypeRegistrationPlugin`](https://docs.rs/bevy/0.10.0/bevy/core/struct.TypeRegistrationPlugin.html) 和 [`FrameCountPlugin`](https://docs.rs/bevy/0.10.0/bevy/core/struct.FrameCountPlugin.html)。

## `EntityCommand` [#](https://bevy.org/news/bevy-0-10/#entitycommands)

作者：@JoJoJet

[`Commands`](https://docs.rs/bevy/0.10.0/bevy/ecs/system/struct.Commands.html) 是"延迟的 ECS"操作。它们使开发者能够定义在并行系统运行完成后应用的自定义 ECS 操作。许多 [`Commands`](https://docs.rs/bevy/0.10.0/bevy/ecs/system/struct.Commands.html) 作用于单个实体，但这种模式有些笨拙：

```rust
struct MyCustomCommand(Entity);

impl Command for MyCustomCommand {
    fn write(self, world: &mut World) {
        // 对 self.0 的实体进行操作
    }
}

let id = commands.spawn(SpriteBundle::default()).id();
commands.add(MyCustomCommand(id));
```

为了解决这个问题，在 **Bevy 0.10** 中，我们添加了 [`EntityCommand`](https://docs.rs/bevy/0.10.0/bevy/ecs/system/trait.EntityCommand.html) trait。这使得命令可以符合人体工程学地应用于生成的实体：

```rust
struct MyCustomCommand;

impl EntityCommand for MyCustomCommand {
    fn write(self, id: Entity, world: &mut World) {
        // 对给定的实体 ID 进行操作
    }
}

commands.spawn(SpriteBundle::default()).add(MyCustomCommand);
```

## 像素完美示例 [#](https://bevy.org/news/bevy-0-10/#pixel-perfect-example)

作者：@Ian-Yy

我们现在有一个新的["像素完美"示例](https://github.com/bevyengine/bevy/blob/v0.10.0/examples/2d/pixel_perfect.rs)，演示了如何设置像素完美的精灵。它使用了一个可爱的新的 Bevy 标志精灵！

![像素完美](https://bevy.org/news/bevy-0-10/pixel_perfect.png)

## UI 文本布局示例 [#](https://bevy.org/news/bevy-0-10/#ui-text-layout-example)

作者：@ickshonpe

我们添加了一个漂亮的["文本布局"示例](https://github.com/bevyengine/bevy/blob/v0.10.0/examples/ui/text_layout.rs)，演示了各种 Bevy UI 文本布局设置：

![文本布局](https://bevy.org/news/bevy-0-10/text_layout.png)

## CI 改进 [#](https://bevy.org/news/bevy-0-10/#ci-improvements)

作者：@mockersf

在 Bevy 领域，我们非常重视 CI，并且我们一直在寻找让我们的生活更美好的新方法。这个周期我们做了许多不错的改进：

- 我们现在为 `bevy` crate 设置了 MSRV（最低支持的 Rust 版本），并且有一个 CI 任务来检查 MSRV
- CI 现在会向新贡献者发送友好的欢迎信息！
- 当一个 PR 被标记为破坏性变更但没有迁移指南时，CI 现在会要求提供一个迁移指南

## 第一个领域专家发布 [#](https://bevy.org/news/bevy-0-10/#the-first-subject-matter-expert-release)

这是我们的第一次使用新的[领域专家（SME）系统](https://bevy.org/news/scaling-bevy-development/)的发布。我们合并了大量变化，这_还是_在我们的项目负责人 `@cart` 离开大约一个月去过圣诞节和滑雪假期的情况下完成的。我们保持了高质量标准，并构建了令人惊叹的东西。可以说，未来是光明的（且可持续的）！敬请期待在更多领域任命更多的 SME。

## 下一步是什么？ [#](https://bevy.org/news/bevy-0-10/#what-s-next)

- **资源系统演进**：我们在[Bevy 资源系统的下一个迭代](https://github.com/bevyengine/bevy/discussions/3972)上取得了良好进展，这将增加预处理资源的能力，并提高资源系统的灵活性和可用性。
- **启动 Bevy 编辑器的努力**：我们准备开始将重心转向构建 Bevy 编辑器！我们已经开始[收集需求](https://github.com/bevyengine/bevy/discussions/7100)，并希望在 **Bevy 0.11** 周期内启动初始设计阶段。
- **时间抗锯齿（TAA）**：我们已经基本实现了 TAA，它使用运动向量和时间来产生一种非常流行的屏幕空间抗锯齿效果。
- **屏幕空间环境光遮蔽（SSAO）**：这是一种流行、相对廉价的照明技术，可以使场景看起来更自然。它建立在深度预处理工作的基础上。
- **自动化渲染批处理和实例化**：通过组合几何体或使用实例化来自动减少绘制调用。这将使 Bevy 能够渲染数十万个对象而不会崩溃。我们技术上已经支持这个，但必须在我们的标准管线之外手动实现。这将在我们的内置渲染管线中"免费"带来批处理和实例化的好处。
- **一次性系统**：通过命令以[推送方式运行任意系统](https://github.com/bevyengine/bevy/issues/2192)，并将其存储为回调组件，实现超灵活的行为定制。
- **更好的插件**：用于[使第三方插件适应你的应用独特架构](https://github.com/bevyengine/bevy/issues/2160)、消除[初始化中的顺序依赖性](https://github.com/bevyengine/bevy/issues/1255)以及定义它们之间的[依赖关系](https://github.com/bevyengine/bevy/issues/69)的更清晰、更标准化的工具。
- **将 `!Send` 数据从 `World` 中拉出**：将非线程安全的数据存储在旨在跨线程发送的结构中，给我们带来了无尽的头痛。我们计划将这些数据拉入 `App` 中，解决一等公民[多世界](https://github.com/bevyengine/rfcs/pull/43)设计的一个主要障碍。
- **时间戳窗口和输入事件**：正如 [#5984](https://github.com/bevyengine/bevy/issues/5984) 中所讨论的，跟踪输入事件的确切时序对于确保事件排序和时序可以被精确重建至关重要。
- **可选择退出的变更检测**：通过[在编译或运行时关闭变更检测](https://github.com/bevyengine/bevy/issues/4882)来提高小组件的性能。
- **全面的动画组合**：支持非过渡性动画组合（即动画的任意加权混合）。更完整的信息请参见 [RFC](https://github.com/bevyengine/rfcs/pull/51)。

查看 [**Bevy 0.11 Milestone**](https://github.com/bevyengine/bevy/milestone/11) 获取正在考虑用于 **Bevy 0.11** 的最新工作列表。
