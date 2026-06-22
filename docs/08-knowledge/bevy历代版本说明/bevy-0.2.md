# Bevy 0.2

## 发布于 2020 年 9 月 19 日，作者：Carter Anderson ( ![一个猫耳人物挥舞触手的剪影，或称 Octocat：GitHub 的吉祥物和标志](https://bevy.org/assets/github_grey.svg) [@cart](https://www.github.com/cart) ![一个圆角矩形内指向右的三角形；Youtube 的标志](https://bevy.org/assets/youtube_grey.svg) [cartdev](https://www.youtube.com/cartdev) )

![Matching Squares by @TheNeikos](https://bevy.org/news/bevy-0-2/matching_squares.png)

[Matching Squares by @TheNeikos](https://github.com/TheNeikos/bevy_squares)

在 Bevy 首次发布一个月后，感谢 **87** 位贡献者、**174** 个 pull request，以及我们[慷慨的赞助者](https://github.com/sponsors/cart)，我很高兴地宣布 **Bevy 0.2** 已在 [crates.io](https://crates.io/crates/bevy) 上发布！

如果你还不了解 Bevy，它是一款基于 Rust 构建的、简洁的数据驱动游戏引擎。你可以查看[快速入门指南](https://bevy.org/learn/quick-start/introduction/)来开始使用。Bevy 将永久免费且开源！你可以在 GitHub 上获取完整的[源代码](https://github.com/bevyengine/bevy)。

以下是本次发布的一些亮点：

## 异步任务系统 [#](https://bevy.org/news/bevy-0-2/#async-task-system)

作者：@lachlansneff 和 @aclysma

Bevy 在引擎内部广泛使用多线程：ECS 调度、资源加载、渲染等。在本次发布之前，几乎所有这些任务都使用 [Rayon](https://github.com/rayon-rs/rayon)。Rayon 之所以好用，是因为通常只需调用 `some_list.par_iter().for_each(|x| do_something(x))` 即可。Rayon 会自动将 `for_each` 拆分为任务，并在尽可能多的核心上运行它们。如果你想轻松地对代码进行并行化，Rayon 是一个很好的选择，但它的缺点是相当消耗 CPU。

Bevy（以及许多其他使用 rayon 的 rust 游戏引擎和 ECS 框架）收到了反馈，称它们过度消耗 CPU，或者 CPU 使用率与"实际"完成的工作不成比例。

我们决定通过构建一个自定义的、对异步友好的任务系统来解决这个问题，该系统支持创建针对特定上下文的任务池。例如，你可以为计算、IO、网络等分别设置不同的池。这还使我们能够根据工作类型和/或优先级灵活地进行负载均衡。CPU 使用率方面的提升非常显著：

### 总合并 CPU 使用百分比 - 8 核机器（越低越好）[#](https://bevy.org/news/bevy-0-2/#total-combined-percent-cpu-usage-8-core-machine-smaller-is-better)

![线程 CPU 使用率 8 核](https://bevy.org/news/bevy-0-2/bevy_tasks_1.svg)

### 总合并 CPU 使用百分比 - 32 核机器（越低越好）[#](https://bevy.org/news/bevy-0-2/#total-combined-percent-cpu-usage-32-core-machine-smaller-is-better)

![线程 CPU 使用率 32 核](https://bevy.org/news/bevy-0-2/bevy_tasks_2.svg)

## 初始 Web 平台支持 [#](https://bevy.org/news/bevy-0-2/#initial-web-platform-support)

作者：@smokku

（部分功能）Bevy 现在可以通过 WebAssembly/WASM 在 Web 上运行了！具体来说，Bevy 应用程序可以运行 Bevy ECS 调度、响应输入事件、创建空白画布（使用 winit），以及其他一些功能。这是一个巨大的第一步，但需要指出的是，仍然有一些缺失的部分，例如 2D/3D 渲染、多线程和声音。

这些限制并没有阻止 @mrk-its 构建第一款 WASM Bevy 游戏！

### [bevy-robbo](https://github.com/mrk-its/bevy-robbo)（[可在此处试玩](https://mrk.sed.pl/bevy-robbo/ascii/)）[#](https://bevy.org/news/bevy-0-2/#bevy-robbo-playable-here)

![bevy-robbo](https://bevy.org/news/bevy-0-2/bevy-robbo.png)

他们使用 Bevy 处理游戏逻辑，并巧妙地绕过了渲染限制，将 ASCII 艺术游戏状态从[这个 Bevy 系统](https://github.com/mrk-its/bevy-robbo/blob/ascii/src/systems/js_render.rs)传递到[这个 JavaScript 函数](https://github.com/mrk-its/bevy-robbo/blob/ascii/wasm/render.js)。

你可以通过[按照此处的说明](https://github.com/bevyengine/bevy/tree/v0.2.0/examples#wasm)来试用一些 Bevy WASM 示例。

## 并行查询 [#](https://bevy.org/news/bevy-0-2/#parallel-queries)

作者：@GrantMoyer

Bevy ECS 查询是一种从实体组件系统（Entity Component System）中检索数据的灵活方式。*使用*查询的系统已经并行运行，但在此变更之前，查询本身无法被*并行迭代*。**Bevy 0.2** 新增了轻松并行迭代查询的能力：

```rs
fn system(pool: Res<ComputeTaskPool>, mut query: Query<&mut Transform>) {
    query.iter().par_iter(32).for_each(&pool, |mut transform| {
      transform.translate(Vec3::new(1.0, 0.0, 0.0));
    });
}
```

这提供了一个类似于 Rayon 的优雅函数式 API，运行在新的 `bevy_tasks` 系统之上。它将查询拆分为 32 个"批次"，并将每个批次作为 bevy 任务系统中的一个不同任务运行。

## Transform 系统重写 [#](https://bevy.org/news/bevy-0-2/#transform-system-rewrite)

作者：@MarekLg

```rs
// 旧版
fn system(translation: &Translation, rotation: &Rotation, scale: &Scale) {
  println!("{} {} {}", translation.0, rotation.0, scale.0);
}

// 新版
fn system(transform: &Transform) {
  println!("{} {} {}", transform.translation(), transform.rotation(), transform.scale());
}
```

Bevy 旧的 Transform 系统使用独立的 `Translation`、`Rotation` 和 `Scale` 组件作为"数据源"。用户在系统中修改这些组件，之后它们会同步到 `LocalTransform` 组件，而 `LocalTransform` 又会同步到全局的 `Transform` 组件，同时考虑层级关系。这样做的好处有几个：

- 检索像 `Translation` 这样的单独组件时稍微更节省缓存（因为需要访问的数据更少）
- 理论上更利于并行。只访问 `Translation` 的系统不会阻塞访问 `Rotation` 的系统。

然而，这种方法也有一些相当严重的缺点：

- "单独组件"是数据源，因此当用户系统运行时 `LocalTransform` 已经过时了。如果需要最新的"完整变换"，必须手动访问所有三个组件来构建。
- 非常难以理解。用户需要考虑 5 个组件，而且它们之间的交互方式各不相同。
- 将 Transform 设置为特定的矩阵值（例如：`Mat4::look_at()`）极其繁琐，并且除非用户显式禁用组件同步，否则该值会立即被覆盖。

考虑到这些问题，我们决定转向使用单个统一的 local-to-parent `Transform` 组件作为数据源，以及一个计算得出的 `GlobalTransform` 组件用于世界空间变换。我们认为这个 API 会更容易使用和理解。[Unity 也在考虑为其 ECS 进行类似的 Transform 重构](https://gist.github.com/joeante/79d25ec3a0e86436e53eb74f3ac82c0c)，并且在这个主题上发生了大量的讨论，可以参见这个 [Amethyst 论坛帖子](https://community.amethyst.rs/t/legion-transform-design-discussion)。

## 摇杆/手柄输入 [#](https://bevy.org/news/bevy-0-2/#joystick-gamepad-input)

作者：@simpuid

得益于 [gilrs](https://gitlab.com/gilrs-project/gilrs) 库，Bevy Input 插件现在可以跨平台支持大多数控制器了！

```rs
fn button_system(gamepads: Res<Vec<Gamepad>>, button_input: Res<Input<GamepadButton>>) {
    for gamepad in gamepads.iter() {
        if button_input.just_pressed(GamepadButton(*gamepad, GamepadButtonType::RightTrigger)) {
            println!("Pressed right trigger!");
        }
    }
}
```

## Bevy ECS 性能改进 [#](https://bevy.org/news/bevy-0-2/#bevy-ecs-performance-improvements)

作者：@cart

### 世代实体 ID [#](https://bevy.org/news/bevy-0-2/#generational-entity-ids)

我们将 Entity ID 从随机 UUID 改为递增的世代索引。随机 UUID 很好，因为它们可以在任何地方创建，在不同游戏运行之间是唯一的，并且可以安全地持久化到文件或跨网络复用。我非常希望能够让它们正常工作，但最终它们相对于其他替代方案来说太慢了。随机性带来了可衡量的开销，并且实体位置必须使用哈希映射（hash map）来查找。

通过转向世代索引（我们使用 hecs 的实现），我们可以直接将 entity id 用作数组索引，这使得实体位置查找变得极快。

### 只读查询 [#](https://bevy.org/news/bevy-0-2/#read-only-queries)

我为不会修改任何内容的查询实现了"只读"trait。这允许我们保证一个查询不会修改任何内容。

### 从 World API 中移除锁 [#](https://bevy.org/news/bevy-0-2/#removed-locking-from-world-apis)

这给我们带来了非常好的速度提升。由于新的"只读查询"以及将 World 变更 API 改为可变 World 借用的组合，我们可以安全地做到这一点。

这尚未在系统中的 `Queries` 上启用，因为一个系统可能拥有多个 `Queries`，而这些查询可能以不保证可变访问唯一性的方式被同时访问。我认为这是一个可以解决的问题，但还需要更多的工作。幸运的是，"for-each"系统没有碰撞风险，所以我们现在在这些系统中使用无锁查询。

### 直接组件查找（单位：纳秒，越低越好）[#](https://bevy.org/news/bevy-0-2/#direct-component-lookup-in-nanoseconds-smaller-is-better)

作为这些优化的结果，直接组件查找比以前*快得多*了：

![get_component 图表](https://bevy.org/news/bevy-0-2/get_component.svg)

请注意，此基准测试使用的是 `world.get::<T>(entity)`。`query.get::<T>(entity)` 的结果应该与 `hecs` 的结果相似，因为它仍然使用锁。最终，我希望我们也能从系统查询中移除锁。
