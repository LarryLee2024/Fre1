# 介绍 Bevy 0.1

## 发布于 2020 年 8 月 10 日，作者 Carter Anderson ( ![GitHub 图标，一个带猫耳挥舞触手的剪影](https://bevy.org/assets/github_grey.svg) [@cart](https://www.github.com/cart) ![YouTube 图标，一个带圆角矩形的右指三角形](https://bevy.org/assets/youtube_grey.svg) [cartdev](https://www.youtube.com/cartdev) )

经过数月的努力，我无比激动地终于宣布 **Bevy Engine**！

Bevy 是一个用 Rust 构建的、令人耳目一新的简单数据驱动游戏引擎和应用框架。它是[免费且开源](https://github.com/bevyengine/bevy)的，永远！

它有以下设计目标：

- **能力完备**：提供完整的 2D 和 3D 功能集
- **简单易用**：新手容易上手，同时为高级用户提供无限灵活性
- **数据驱动**：使用实体组件系统（ECS）范式的数据导向架构
- **模块化**：只使用你需要的，替换你不喜欢的
- **高性能**：应用逻辑应快速运行，并尽可能并行执行
- **高效产出**：变更应快速编译……等待可不好玩

Bevy 有一些我认为使其区别于其他引擎的特性：

- **Bevy ECS**：一个自定义的实体组件系统，具有无与伦比的易用性和极速性能
- **渲染图（Render Graph）**：使用渲染图节点轻松构建自己的多线程渲染管线
- **Bevy UI**：一个专门为 Bevy 构建的自定义 ECS 驱动 UI 框架
- **高效的编译时间**：使用"快速编译"配置，预计变更在 ~0.8-3.0 秒内编译完成

它还拥有大多数人对现代通用引擎所期望的许多功能：

- **跨平台**：Windows、MacOS 和 Linux（计划支持移动端和 Web）
- **3D**：灯光、网格、纹理、MSAA 和 GLTF 加载
- **精灵**：将单个图像渲染为精灵，从精灵表中渲染，以及动态生成新的精灵表
- **资源**：一个可扩展的、事件驱动的资源系统，在后台线程中异步加载资源
- **场景**：将 ECS World 保存为人类可读的场景文件，并将场景文件加载到 ECS World 中
- **插件**：所有引擎和应用功能都实现为模块化插件
- **音效**：将音频文件作为资源加载，并在系统中播放
- **多渲染后端**：Vulkan、DirectX 12 和 Metal（得益于 [wgpu](https://github.com/gfx-rs/wgpu)，更多后端正在路上）
- **数据驱动着色器**：轻松将 ECS 组件直接绑定到着色器 uniform
- **热资源重载**：在运行时自动重新加载对资源的更改，无需重新编译或重启
- **事件**：在 ECS 系统中高效地消费和生产事件
- **属性**：使用属性名的字符串形式动态获取和设置组件字段
- **层级变换**：在实体之间创建父子关系，沿着层级传递变换（Transform）

话虽如此，Bevy 仍处于非常早期的阶段。我认为它处于"原型"阶段：功能尚不完善，API 将会变化，文档也很稀疏。**除非你愿意面对各种空白和不稳定性，我目前还不建议在正式项目中使用 Bevy。**

希望此时你要么（1）对 Bevy 感到兴奋，要么（2）已经不再阅读了。如果你现在就想上手，[快速入门指南](https://bevy.org/learn/quick-start/introduction/)是最好的起点。你也可以继续阅读，了解 Bevy 的当前状态以及我们想要将它带向何方。

**给读者的小提示**：在本文中，你会看到像这样的文本：[`Texture`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.Texture.html)

这种格式表示该文本是一个 Rust 类型，链接到 API 文档。我鼓励你点击任何让你感兴趣的内容！

## Bevy Apps [#](https://bevy.org/news/introducing-bevy/#bevy-apps)

首先，让我们看看一个 Bevy App 实际长什么样。最简单的 App 如下所示：

```rs
use bevy::prelude::*;

fn main() {
    App::build().run();
}
```

就是这样！这个 App 没有引入任何功能，实际上什么都不做。运行程序只会立即终止。我们可以通过这样做让它更有趣一点：

```rs
fn main() {
    App::build()
        .add_default_plugins()
        .run();
}
```

[`AddDefaultPlugins::add_default_plugins`](https://docs.rs/bevy/0.1.0/bevy/trait.AddDefaultPlugins.html#tymethod.add_default_plugins) 添加了你可能期望的游戏引擎的所有功能：2D / 3D 渲染器、资源加载、UI 系统、窗口、输入等。

你也可以像这样手动注册默认的 [`Plugins`](https://docs.rs/bevy/0.1.0/bevy/prelude/trait.Plugin.html)：

```rs
fn main() {
    App::build()
        .add_plugin(CorePlugin::default())
        .add_plugin(InputPlugin::default())
        .add_plugin(WindowPlugin::default())
        .add_plugin(RenderPlugin::default())
        .add_plugin(UiPlugin::default())
        /* 此处省略更多插件…… */
        .run();
}
```

当然，你也可以创建自己的插件。事实上，所有引擎和游戏逻辑都是使用插件构建的。希望你现在理解我们所说的模块化是什么意思：你可以根据项目的独特需求自由添加/删除插件。不过我预计大多数人会为了简洁起见（至少在一开始）坚持使用 [`AddDefaultPlugins::add_default_plugins`](https://docs.rs/bevy/0.1.0/bevy/trait.AddDefaultPlugins.html#tymethod.add_default_plugins)。

## Bevy ECS [#](https://bevy.org/news/introducing-bevy/#bevy-ecs)

所有 Bevy 引擎和游戏逻辑都构建在一个自定义的[实体组件系统](https://en.wikipedia.org/wiki/Entity_component_system)（简称 ECS）之上。实体组件系统是一种将数据分解为组件的软件范式。实体是分配给组件组的唯一 ID。例如，一个实体可能拥有 `Position` 和 `Velocity` 组件，而另一个实体可能拥有 `Position` 和 `UI` 组件。系统是在特定组件类型集上运行的逻辑。你可能有一个 `movement` 系统，它在所有拥有 `Position` 和 `Velocity` 组件的实体上运行。

ECS 模式通过强制你将应用数据和逻辑分解为其核心组件，从而鼓励简洁、解耦的设计。

与其他需要复杂生命周期、trait、构建器模式或宏的 Rust ECS 实现不同，Bevy ECS 为所有这些概念使用普通的 Rust 数据类型：

- **组件**：普通的 Rust 结构体
- **系统**：普通的 Rust 函数
- **实体**：包含唯一整数的类型

已经有大量[优秀的介绍](https://www.youtube.com/watch?v=2rW7ALyHaas)关于 ECS 范式，所以我将"熟悉 ECS"作为练习留给读者，直接跳到 Bevy ECS 的特别之处：

### 人体工程学 [#](https://bevy.org/news/introducing-bevy/#ergonomics)

我要在这里做一个大胆（且无法验证）的宣称：Bevy ECS 是*现存*最符合人体工程学的 ECS：

```rs
use bevy::prelude::*;

struct Velocity(f32);
struct Position(f32);

// 该系统生成带有 Position 和 Velocity 组件的实体
fn setup(mut commands: Commands) {
    commands
        .spawn((Position(0.0), Velocity(1.0),))
        .spawn((Position(1.0), Velocity(2.0),));
}

// 该系统在每个带有 Position 和 Velocity 组件的实体上运行
fn movement(mut position: Mut<Position>, velocity: &Velocity) {
    position.0 += velocity.0;
}

// 应用入口点。希望你能从上面的例子认出它！
fn main() {
    App::build()
        .add_default_plugins()
        .add_startup_system(setup.system())
        .add_system(movement.system())
        .run();
}
```

这是一个完整的自包含 Bevy 应用，具有自动并行系统调度和全局变更检测。在我看来，你[找不到](https://github.com/amethyst/specs/blob/master/examples/basic.rs)任何 ECS 有更好的[清晰度](https://github.com/leudz/shipyard/blob/master/bunny_demo/src/systems.rs)或人体工程学。开发游戏（和引擎）需要编写大量的系统，所以我大力投入，使 ECS 代码既易于编写*又*易于阅读。

### 性能 [#](https://bevy.org/news/introducing-bevy/#performance)

ECS 范式如此受欢迎的原因之一是它有可能使游戏逻辑*超级*快，主要有以下两个原因：

1. **迭代速度**：组件被紧密打包以优化缓存局部性，这使得迭代它们非常快
2. **并行性**：系统声明读/写依赖关系，从而实现自动且高效的无锁并行调度

Bevy ECS 在这两方面都做得尽可能好。根据流行的 `ecs_bench` 基准测试，Bevy ECS 以相当大的优势成为最快的 Rust ECS：

#### 系统迭代（纳秒，越少越好） [#](https://bevy.org/news/introducing-bevy/#system-iteration-in-nanoseconds-less-is-better)

![ecs 迭代性能图](https://bevy.org/news/introducing-bevy/ecs_iter.svg)

#### World 设置（纳秒，越少越好） [#](https://bevy.org/news/introducing-bevy/#world-setup-in-nanoseconds-less-is-better)

![ecs 迭代性能图](https://bevy.org/news/introducing-bevy/ecs_build.svg)

请注意，`ecs_bench` 是一个单线程基准测试，因此它不能说明这些框架的多线程能力。和往常一样，请注意 `ecs_bench` 是一个微基准测试，并不能说明复杂游戏的性能。ECS 性能领域有很多细微差别，上述每个 ECS 实现在不同工作负载下的表现都会不同。

我已将我的 `ecs_bench` 版本推送到了[这里](https://github.com/cart/ecs_bench)，如果有人想验证我的方法。在合理的时间范围内，如果有人报告问题，或者我的结果（平均而言）不可复现，我会在此发布更新。

### 功能 [#](https://bevy.org/news/introducing-bevy/#features)

现在你可能会想"好的 @cart，所以 Bevy ECS 有出色的性能和人体工程学，但*肯定*这意味着你不得不在功能上妥协！"

......**不**，Bevy 都能满足你：

#### For Each 系统 [#](https://bevy.org/news/introducing-bevy/#for-each-systems)

```rs
// "for each 系统"对每个包含给定组件的实体运行一次
fn system(position: Mut<Position>, velocity: &Velocity) {
    // 做某事
}
```

#### 查询系统 [#](https://bevy.org/news/introducing-bevy/#query-systems)

```rs
// 这个"查询系统"与上面的系统相同，但让你控制迭代
fn system(mut query: Query<(&Position, &mut Velocity)>) {
    for (position, mut velocity) in &mut query.iter() {
        // 做某事
    }
}
```

#### 变更检测 [#](https://bevy.org/news/introducing-bevy/#change-detection)

```rs
// Added<T> 查询仅在给定组件被添加时运行
fn system(mut query: Query<Added<Position>>) {
    for position in &mut query.iter() {
        // 做某事
    }
}

// Mutated<T> 查询仅在给定组件被修改时运行
fn system(mut query: Query<Mutated<Position>>) {
    for position in &mut query.iter() {
        // 做某事
    }
}

// Changed<T> 查询仅在给定组件被添加或修改时运行
fn system(mut query: Query<Changed<Position>>) {
    for position in &mut query.iter() {
        // 做某事
    }
}

// query.removed<T>() 将遍历本次更新中组件 T 被移除的每个实体
fn system(mut query: Query<&Position>>) {
    for entity in query.removed::<Velocity>() {
        // 做某事
    }
}
```

#### 多查询 [#](https://bevy.org/news/introducing-bevy/#multiple-queries)

```rs
fn system(mut wall_query: Query<&Wall>, mut player_query: Query<&Player>) {
    for player in &mut player_query.iter() {
        for wall in &mut wall_query.iter() {
            if player.collides_with(wall) {
                println!("哎哟");
            }
        }
    }
}
```

#### 实体查询和直接组件访问 [#](https://bevy.org/news/introducing-bevy/#entity-queries-and-direct-component-access)

```rs
fn system(mut entity_query: Query<Entity>, mut player_query: Query<&Player>) {
    for entity in &mut entity_query.iter() {
       if let Some(player) = player_query.get::<Player>(entity) {
           // 当前实体有一个 player 组件
       }
    }
}
```

#### 资源 [#](https://bevy.org/news/introducing-bevy/#resources)

```rs
// Res 和 ResMut 访问全局资源
fn system(time: Res<Time>, score: ResMut<Score>) {
    // 做某事
}

// 你可以在任何系统类型中使用资源
fn system(time: Res<Time>, mut query: Query<&Position>) {
    // 做某事
}
fn system(time: Res<Time>, &Position) {
    // 做某事
}
```

#### "本地"系统资源 [#](https://bevy.org/news/introducing-bevy/#local-system-resources)

```rs
// Local<T> 资源是每个系统唯一的。同一系统的两个实例将各自拥有自己的资源。
// 本地资源会自动初始化为其默认值。
fn system(state: Local<State>, &Position) {
    // 做某事
}
```

#### 空系统 [#](https://bevy.org/news/introducing-bevy/#empty-systems)

```rs
// 给超极简主义者
fn system() {
    // 做某事
}
```

#### With/Without 过滤器 [#](https://bevy.org/news/introducing-bevy/#with-without-filters)

```rs
// 仅在带有（With）或不带有（Without）给定组件的实体上运行
fn system(mut query: Query<Without<Parent, &Position>>) {
    for position in &mut query.iter() {
        // 做某事
    }
}
```

#### 线程本地系统 [#](https://bevy.org/news/introducing-bevy/#thread-local-systems)

```rs
// 必须在主线程上运行并具有 World 和 Resources 独占访问权限的系统
fn system(world: &mut World, resources: &mut Resources) {
    // 做某事
}
```

#### 阶段 [#](https://bevy.org/news/introducing-bevy/#stages)

```rs
// 调度器提供阶段（Stage）作为按顺序运行系统组的方式
fn main() {
    App::build()
        // 将系统添加到默认阶段："update"
        .add_system(movement.system())
        // 在 "update" 之后创建一个新阶段
        .add_stage_after("update", "do_things")
        .add_system_to_stage("do_things", something.system())
}
```

#### 命令 [#](https://bevy.org/news/introducing-bevy/#commands)

```rs
// 使用 Commands 将 World 和 Resource 更改排队，这些更改将在当前阶段结束时应用
fn system(mut commands: Commands) {
    commands.spawn((Position(0.0), Velocity(1.0)));
}

// Commands 也可以与其他类型一起使用
fn system(mut commands: Commands, time: Res<Time>, mut query: Query<&Position>) {
    // 做某事
}
```

### 函数系统是如何工作的？ [#](https://bevy.org/news/introducing-bevy/#how-do-function-systems-work)

能够直接将 Rust 函数用作系统可能感觉像是魔法，但我保证它不是！你可能已经注意到我们在 App 中注册系统时是这样做的：

```rs
fn some_system() { }

fn main() {
    App::build()
        .add_system(some_system.system())
        .run();
}
```

`.system()` 调用获取 `some_system` 函数指针并将其转换为 `Box<dyn System>`。这之所以有效，是因为我们为所有匹配特定函数签名集的函数实现了 [`IntoQuerySystem`](https://docs.rs/bevy/0.1.0/bevy/prelude/trait.IntoQuerySystem.html) trait。

### 良好基础 [#](https://bevy.org/news/introducing-bevy/#good-bones)

Bevy ECS 实际上使用了大量分支版本的极简主义 [Hecs ECS](https://github.com/Ralith/hecs)。Hecs 是一个高效的单线程原型型（archetypal）ECS。它提供了核心的 [`World`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.World.html)、[`Archetype`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.World.html#method.archetypes) 和内部 [`Query`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.Query.html) 数据结构。Bevy ECS 在此基础上增加了以下内容：

- **函数系统**：Hecs 实际上根本没有"系统"的概念。你直接在 World 上运行查询。Bevy 增加了使用普通 Rust 函数定义可移植、可调度系统的能力。
- **资源**：Hecs 没有唯一/全局数据的概念。在构建游戏时，这通常是需要的。Bevy 增加了 `Resource` 集合和资源查询。
- **并行调度器**：Hecs 是单线程的，但它被设计为允许在其之上构建并行调度器。Bevy ECS 添加了一个自定义的依赖感知调度器，它建立在上文提到的"函数系统"之上。
- **优化**：Hecs 已经足够快了，但通过修改其一些内部数据访问模式，我们能够显著提高性能。这将其从"足够快"提升到了"最快的"（请参阅上面的基准测试，将 Bevy ECS 与原始 Hecs 进行比较）。
- **查询包装器**：Bevy ECS 导出的 `Query` 实际上是对 Hecs 查询的包装器。它在多线程上下文中提供对 `World` 的安全、有作用域的访问，并改善了迭代的人体工程学。
- **变更检测**：自动（且高效地）跟踪组件的添加/移除/更新操作，并在查询接口中暴露它们。
- **稳定的实体 ID**：几乎所有的 ECS（包括 Hecs）都使用不稳定的实体 ID，不能用于序列化（场景/保存文件）或网络。在 Bevy ECS 中，实体 ID 是全局唯一且稳定的。你可以在任何上下文中使用它们！

在不久的将来，我将在 Hecs git 仓库上提交一个 issue，提议向上游贡献 Bevy ECS 中他们想要的任何更改。我感觉他们不会想要像函数系统和并行调度这样的"高级"东西，但我们拭目以待！

## Bevy UI [#](https://bevy.org/news/introducing-bevy/#bevy-ui)

![bevy ui](https://bevy.org/news/introducing-bevy/bevy_ui.png)

Bevy 有一个自定义但熟悉的 UI 系统，基于"flex box"模型。嗯……半自定义，但稍后会详细介绍。最初，我认真考虑过使用 Rust 生态系统中众多优秀的[预制](https://github.com/linebender/druid) UI 解决方案中的[一个](https://github.com/hecrj/iced)。但这些框架在某种程度上都感觉与 Bevy 核心的数据驱动 ECS 方法"分离"。如果我们采用像 [Druid](https://github.com/linebender/druid) 这样的框架（在设计方面属于同类最佳），然后将其硬塞进 Bevy 的数据/事件模型中，那将会*损害* Druid 的设计，而且 Bevy+Druid 最终会比仅仅将 Druid 作为独立框架使用更缺乏吸引力。

我决定 Bevy 唯一有可能带来引人注目成果的方式，就是完全拥抱 Bevy 自己的做事方式。

Bevy UI 直接使用 Bevy 核心现有的 ECS、层级、变换、事件、资源和场景系统。正因为如此，Bevy UI 自动获得了诸如 UI 场景文件热重载、异步纹理加载和变更检测等功能。共享架构意味着对这些系统中任何一个的改进都会直接反馈到 Bevy UI。我尚未确信这种方法会产生最好的 UI 框架，但我*确实*相信在 Bevy App 的上下文中，它将产生最好的 UI 框架。

我们仍处于实验阶段，我预计有些事情会发生变化，但我们目前找到的模式非常有前景。还请记住，目前编写 Bevy UI 的最佳方式是通过代码，但我们正在设计一种新的场景文件格式，这应该会使基于声明式文件的 UI 组合比现在更加方便。

### 构建块 [#](https://bevy.org/news/introducing-bevy/#building-blocks)

在 Bevy 中，UI 元素只是一个带有 [`Node`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.Node.html) 组件的 ECS 实体。Node 是具有宽度和高度的矩形，并使用在 Bevy 其他地方使用的相同 [`Transform`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.Transform.html) 组件来定位。[`Style`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.Style.html) 组件用于确定 Node 如何被渲染、调整大小和定位。

添加一个新节点（包含所有必需的组件）最简单的方法是：

```rs
commands.spawn(NodeComponents::default())
```

[`NodeComponents`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.NodeComponents.html) 是一个"组件 bundle"，Bevy 使用它来更容易地生成各种"类型"的实体。

### 布局 [#](https://bevy.org/news/introducing-bevy/#layout)

对于布局，Bevy 使用了一个出色的纯 Rust flexbox 实现，叫做 [Stretch](https://github.com/vislyhq/stretch)。Stretch 提供了根据 flexbox 规范在 2D 空间中定位矩形的算法。Bevy 在上述的 [`Style`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.Style.html) 组件中暴露了 flex 属性，并渲染 Stretch 输出的位置和大小的矩形。Bevy 使用自己的 z 层叠算法来"堆叠"元素，但这基本上和 HTML/CSS 使用的是一样的。

### 相对定位 [#](https://bevy.org/news/introducing-bevy/#relative-positioning)

节点默认是相对于彼此定位的：

![相对定位](https://bevy.org/news/introducing-bevy/relative_position.png)

```rs
commands
    .spawn(NodeComponents {
        style: Style {
            size: Size::new(Val::Px(100.0), Val::Px(100.0)),
            ..Default::default()
        },
        material: materials.add(Color::rgb(0.08, 0.08, 1.0).into()),
        ..Default::default()
    })
    .spawn(NodeComponents {
        style: Style {
            size: Size::new(Val::Percent(40.0), Val::Percent(40.0)),
            ..Default::default()
        },
        material: materials.add(Color::rgb(1.0, 0.08, 0.08).into()),
        ..Default::default()
    });
```

### 绝对定位 [#](https://bevy.org/news/introducing-bevy/#absolute-positioning)

你可以像这样相对于其父节点的角"绝对"定位一个 Node：

![绝对定位](https://bevy.org/news/introducing-bevy/absolute_positioning.png)

```rs
commands
    .spawn(NodeComponents {
        style: Style {
            size: Size::new(Val::Percent(40.0), Val::Percent(40.0)),
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                ..Default::default()
            },
            ..Default::default()
        },
        material: materials.add(Color::rgb(0.08, 0.08, 1.0).into()),
        ..Default::default()
    });
```

### 父子层级 [#](https://bevy.org/news/introducing-bevy/#parenting)

就像任何其他实体一样，Node 可以有子节点。子节点相对于其父节点定位和缩放。默认情况下，子节点总是显示在其父节点前面。

![ui 父子层级](https://bevy.org/news/introducing-bevy/ui_parenting.png)

```rs
commands
    .spawn(NodeComponents {
        style: Style {
            size: Size::new(Val::Percent(60.0), Val::Percent(60.0)),
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                ..Default::default()
            },
            ..Default::default()
        },
        material: materials.add(Color::rgb(0.08, 0.08, 1.0).into()),
        ..Default::default()
    })
    .with_children(|parent| {
        parent
            .spawn(NodeComponents {
                style: Style {
                    size: Size::new(Val::Px(50.0), Val::Px(50.0)),
                    ..Default::default()
                },
                material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
                ..Default::default()
            });
    });
```

### Flexbox [#](https://bevy.org/news/introducing-bevy/#flexbox)

我不会在这里介绍 flexbox 的工作原理，但你可以使用在 Web 上下文中使用的所有相同的"flex"属性。下面是一个例子，展示如何在父节点内垂直和水平居中两个 Node：

![flex](https://bevy.org/news/introducing-bevy/flex.png)

```rs
commands
    .spawn(NodeComponents {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        material: materials.add(Color::rgb(0.04, 0.04, 0.04).into()),
        ..Default::default()
    })
    .with_children(|parent| {
        parent
            .spawn(NodeComponents {
                style: Style {
                    size: Size::new(Val::Px(80.0), Val::Px(80.0)),
                    ..Default::default()
                },
                material: materials.add(Color::rgb(0.08, 0.08, 1.0).into()),
                ..Default::default()
            })
            .spawn(NodeComponents {
                style: Style {
                    size: Size::new(Val::Px(80.0), Val::Px(80.0)),
                    ..Default::default()
                },
                material: materials.add(Color::rgb(1.0, 0.08, 0.08).into()),
                ..Default::default()
            });
    });
```

### 文本和图像 [#](https://bevy.org/news/introducing-bevy/#text-and-images)

Node 也可以有文本和图像组件，这会影响节点的推断大小。

![文本和图像](https://bevy.org/news/introducing-bevy/text_and_image.png)

```rs
commands
    .spawn(TextComponents {
        text: Text {
            value: "Hello from Bevy UI!".to_string(),
            font: asset_server.load("FiraSans-Bold.ttf").unwrap(),
            style: TextStyle {
                font_size: 25.0,
                color: Color::WHITE,
            },
        },
        ..Default::default()
    })
    .spawn(ImageComponents {
        style: Style {
            size: Size::new(Val::Px(200.0), Val::Auto),
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                ..Default::default()
            },
            ..Default::default()
        },
        material: materials.add(asset_server.load("bevy_logo.png").unwrap().into()),
        ..Default::default()
    });
```

### 交互事件 [#](https://bevy.org/news/introducing-bevy/#interaction-events)

带有 [`Interaction`](https://docs.rs/bevy/0.1.0/bevy/prelude/enum.Interaction.html) 组件的 Node 将跟踪交互状态。你可以像这样轻松构建按钮等小部件：

例如，这是一个仅在 Interaction 状态发生变化的 Button 上运行的系统：

```rs
fn system(_button: &Button, interaction: Mutated<Interaction>) {
    match *interaction {
        Interaction::Clicked => println!("点击"),
        Interaction::Hovered => println!("悬停"),
        Interaction::None => {},
    }
}
```

## 2D 功能 [#](https://bevy.org/news/introducing-bevy/#2d-features)

### [精灵](https://github.com/bevyengine/bevy/blob/1d68094f59b01e14f44ed7db8907dbd011b59973/examples/2d/sprite.rs) [#](https://bevy.org/news/introducing-bevy/#sprites)

你可以直接使用任何 [`Texture`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.Texture.html) 资源作为精灵：

![精灵](https://bevy.org/news/introducing-bevy/sprite.png)

```rs
let texture = asset_server.load("icon.png").unwrap();
commands.spawn(SpriteComponents {
    material: materials.add(texture.into()),
    ..Default::default()
});
```

### [精灵表](https://github.com/bevyengine/bevy/blob/1d68094f59b01e14f44ed7db8907dbd011b59973/examples/2d/sprite_sheet.rs) [#](https://bevy.org/news/introducing-bevy/#sprite-sheets)

精灵表（也称为纹理图集）可用于动画、瓦片集，或用于优化精灵渲染。

```rs
let texture_atlas = TextureAtlas::from_grid(texture_handle, texture.size, 7, 1);
let texture_atlas_handle = texture_atlases.add(texture_atlas);
commands
    .spawn(SpriteSheetComponents {
        texture_atlas: texture_atlas_handle,
        sprite: TextureAtlasSprite::new(0),
        ..Default::default()
    });
```

### [动态纹理图集生成](https://github.com/bevyengine/bevy/blob/1d68094f59b01e14f44ed7db8907dbd011b59973/examples/2d/texture_atlas.rs) [#](https://bevy.org/news/introducing-bevy/#dynamic-texture-atlas-generation)

精灵通常作为单独的文件产生。Bevy 可以动态地将它们合并到一个精灵表中！

![动态纹理图集](https://bevy.org/news/introducing-bevy/dynamic_texture_atlas.png)

```rs
for sprite_handle in sprite_handles.iter() {
    let texture = textures.get(&handle).unwrap();
    texture_atlas_builder.add_texture(handle, &texture);
}

let texture_atlas = texture_atlas_builder.finish(&mut textures).unwrap();
```

## 3D 功能 [#](https://bevy.org/news/introducing-bevy/#3d-features)

### [GLTF 模型加载](https://github.com/bevyengine/bevy/blob/1d68094f59b01e14f44ed7db8907dbd011b59973/examples/3d/load_model.rs) [#](https://bevy.org/news/introducing-bevy/#gltf-model-loading)

加载 GLTF 文件作为网格资源

![船只渲染](https://bevy.org/news/introducing-bevy/boat.png)

```rs
commands
    .spawn(PbrComponents {
        // 加载模型
        mesh: asset_server.load("boat.gltf").unwrap(),
        // 为模型创建材质
        material: materials.add(asset_server.load("boat.png").into()),
        ..Default::default()
    })
```

注意：在不久的将来，我们将添加将 GLTF 文件作为场景（Scene）而非网格加载的支持。

### [基于深度的绘制顺序](https://github.com/bevyengine/bevy/blob/1d68094f59b01e14f44ed7db8907dbd011b59973/examples/3d/texture.rs) [#](https://bevy.org/news/introducing-bevy/#depth-based-draw-order)

不透明材质使用从前往后的绘制以实现快速"早期片元丢弃"，透明材质使用从后往前的绘制以实现正确的透明效果。

![alpha](https://bevy.org/news/introducing-bevy/alpha.png)

### [父子层级](https://github.com/bevyengine/bevy/blob/1d68094f59b01e14f44ed7db8907dbd011b59973/examples/3d/parenting.rs) [#](https://bevy.org/news/introducing-bevy/#parenting-1)

父级变换会传递到其后代。

```rs
commands
    .spawn(PbrComponents {
        mesh: cube_handle,
        ..Default::default()
    }).with_children(|parent| {
        parent.spawn(PbrComponents {
            mesh: cube_handle,
            translation: Translation::new(0.0, 2.0, 0.0),
            ..Default::default()
        });
    })
```

### [MSAA](https://github.com/bevyengine/bevy/blob/1d68094f59b01e14f44ed7db8907dbd011b59973/examples/3d/msaa.rs) [#](https://bevy.org/news/introducing-bevy/#msaa)

通过使用多重采样抗锯齿（MSAA）获得漂亮的平滑边缘。

![msaa 关闭](https://bevy.org/news/introducing-bevy/msaa_off.png) ![msaa 开启](https://bevy.org/news/introducing-bevy/msaa_on.png)

```rs
app.add_resource(Msaa { samples: 8 })
```

## 场景 [#](https://bevy.org/news/introducing-bevy/#scenes)

场景是一种预先组合你的游戏/应用各部分的方式。在 Bevy 中，场景仅仅是实体和组件的集合。一个场景可以被"生成"到一个 [`World`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.World.html) 中任意次数。"生成"会将场景的实体和组件复制到给定的 [`World`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.World.html) 中。

场景也可以保存到"场景文件"或从"场景文件"加载。未来"Bevy 编辑器"的主要目标之一将是使可视化地组合场景文件变得容易。

### 文件格式 [#](https://bevy.org/news/introducing-bevy/#file-format)

场景文件保存和加载为实体和组件的扁平列表：

```json
[
  (
    entity: 328997855,
    components: [
      {
        "type": "Position",
        "map": { "x": 3.0, "y": 4.0 },
      },
    ],
  ),
  (
    entity: 404566393,
    components: [
      {
        "type": "Position",
        "map": { "x": 1.0, "y": 2.0 },
      },
      {
        "type": "Name",
        "map": { "value": "Carter" },
      },
    ],
  ),
]
```

分配给 `entity` 字段的数字是实体的 ID，这些 ID 是完全可选的。如果没有提供实体 ID，则在加载场景时会随机生成一个。我们[计划在未来改进这种格式](https://gist.github.com/cart/3e77d6537e1a0979a69de5c6749b6bcb)，使其更符合人体工程学，缩进实体层级，并支持嵌套场景。

### 加载和实例化 [#](https://bevy.org/news/introducing-bevy/#loading-and-instancing)

场景可以通过 [`SceneSpawner`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.SceneSpawner.html) 资源添加到 [`World`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.World.html) 中。生成可以使用 [`SceneSpawner::load`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.SceneSpawner.html#method.load) 或 [`SceneSpawner::instance`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.SceneSpawner.html#method.instance)。"加载"场景会保留其中的实体 ID。这对于保存文件之类的情况很有用，因为你希望实体 ID 保持不变，并且更改将应用于已经存在于世界中的实体之上。"实例化"则使用全新的 ID 将实体添加到 [`World`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.World.html)，允许多个场景"实例"存在于同一个 [`World`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.World.html) 中。

```rs
fn load_scene_system(asset_server: Res<AssetServer>, mut scene_spawner: ResMut<SceneSpawner>) {
    // 场景像任何其他资源一样被加载。
    let scene: Handle<Scene> = asset_server.load("my_scene.scn").unwrap();
    // 生成场景，保留实体 ID
    scene_spawner.load(scene);
    // 生成场景，使用新的实体 ID
    scene_spawner.instance(scene);
}
```

### 将 ECS World 保存到场景 [#](https://bevy.org/news/introducing-bevy/#saving-ecs-worlds-to-scenes)

任何 ECS [`World`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.World.html) 都可以像这样转换为场景：

```rs
let scene = Scene::from_world(&world, &component_type_registry);
```

然后你可以像这样将场景转换为 RON 格式的字符串：

```rs
let ron_string = scene.serialize_ron(&property_type_registry)?;
```

### 热场景重载 [#](https://bevy.org/news/introducing-bevy/#hot-scene-reloading)

对场景文件的更改可以在运行时自动应用到已生成的场景中。这允许即时反馈，无需重启或重新编译。

注意，上面的视频没有加速。场景更改实际上是瞬间应用的。

### 这是如何工作的？ [#](https://bevy.org/news/introducing-bevy/#how-does-this-work)

场景建立在 Bevy 的属性和资源系统之上。组件如果派生了 `Properties` trait，就可以在场景中使用。属性（Properties）使得场景的序列化、反序列化和在运行时打补丁成为可能。查看下一节了解更多细节。

## 属性 [#](https://bevy.org/news/introducing-bevy/#properties)

简而言之，Bevy 的属性系统为 Rust 这个出了名静态的语言增加了动态性。使用字段名的字符串形式来获取或设置结构体的字段，或者在你没有静态类型引用时与结构体交互，通常非常有用。语言通常通过"反射"功能来覆盖这些情况，但不幸的是 Rust 目前没有这种类型的反射。我构建了 `bevy_property` crate 来在 Rust 中提供一部分有用的"类反射"功能。以下是一个快速的表面介绍：

```rs
#[derive(Properties)]
pub struct Counter {
    count: u32,
}

let mut counter = Counter { count: 1 };

// 你可以像这样设置属性值。类型必须完全匹配，否则会失败。
counter.set_prop_val::<u32>("count", 2);
assert_eq!(counter.count, 2);
assert_eq!(counter.prop_val::<u32>("count").unwrap(), 2);

// 你也可以动态设置属性。set_prop 接受任何实现了 Property trait 的类型，
// 但属性类型必须与字段类型匹配，否则此操作将失败。
let new_count: u32 = 3;
counter.set_prop("count", &new_count);
assert_eq!(counter.count, 3);

// DynamicProperties 也实现了 Properties trait，但它对字段名称或类型没有限制
let mut patch = DynamicProperties::map();
patch.set_prop_val::<usize>("count", 4);

// 你可以将属性"应用"到其他属性之上。这只会设置具有相同名称和类型的属性。
// 你可以用它来用新值"修补"你的属性。
counter.apply(&patch);
assert_eq!(counter.count, 4);

// 实现了 Properties 的类型可以转换为 DynamicProperties
let dynamic_thing: DynamicProperties = counter.to_dynamic();
```

属性是使 Bevy 的场景系统如此好用的原因。我还计划在即将推出的 Bevy 编辑器中使用它们来实现撤销/重做、在运行时查看和编辑组件属性，以及属性动画工具等功能。

实现了 Properties 的类型可以使用 [serde](https://serde.rs/) 进行序列化，而 `DynamicProperties` 可以使用 serde 进行反序列化。结合 `Properties` 的修补功能，这意味着任何派生了 `Properties` 的类型都可以进行往返序列化和反序列化。

要派生 `Properties`，结构体中的每个字段都必须实现 `Property` trait。这对于大多数核心 Rust 和 Bevy 类型已经实现了，所以你只需要为自定义类型实现 `Property`（你也可以派生 `Property`）。

我感觉 `bevy_property` crate 在非 Bevy 上下文中也会有用，所以我将在不久的将来将其发布到 crates.io。

## 事件 [#](https://bevy.org/news/introducing-bevy/#events)

Bevy 使用一个双缓冲事件系统，实现了高效的事件生产和消费，且事件消费者零分配。这是一个完整的 Bevy 应用，它生产和消费自定义事件：

```rs
fn main() {
    App::build()
        .add_event::<MyEvent>()
        .add_system(event_producer.system())
        .add_system(event_consumer.system())
        .run();
}

struct MyEvent {
    message: String,
}

fn event_producer(mut my_events: ResMut<Events<MyEvent>>) {
    my_events.send(MyEvent { message: "Hello".to_string() });
}

#[derive(Default)]
struct State {
    reader: EventReader<MyEvent>, 
}

fn event_consumer(mut state: Local<State>, my_events: Res<Events<MyEvent>>) {
    for event in state.reader.iter(&my_events) {
        println!("收到消息: {}", event.message);
    }
}
```

`app.add_event::<MyEvent>()` 为 MyEvent 添加了一个新的 [`Events`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.Events.html) 资源，以及一个每次更新时交换 `Events<MyEvent>` 缓冲区的系统。[`EventReaders`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.EventReader.html) 的创建成本非常低。它们本质上只是一个跟踪最后读取事件的数组索引。

Bevy 在窗口调整大小、资源和输入等功能中使用了事件。这种在分配和 CPU 效率之间的权衡是，每个系统只有一次机会接收事件，否则将在下一次更新中丢失。我相信对于循环运行的应用（例如游戏）来说，这是正确的权衡。

## 资源 [#](https://bevy.org/news/introducing-bevy/#assets)

Bevy 的 [`Assets`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.Assets.html) 只是类型化的数据，可以使用资源 [`Handles`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.Handle.html) 来引用。例如，3D 网格、纹理、字体、材质、场景和音效都是资源。`Assets<T>` 是一个类型 `T` 的资源的通用集合。通常的资源使用方式如下：

### 资源创建 [#](https://bevy.org/news/introducing-bevy/#asset-creation)

```rs
fn create_texture_system(mut textures: ResMut<Assets<Texture>>) {
    // 创建一个新的 Texture 资源并返回一个句柄，该句柄随后可用于检索实际资源
    let texture_handle: Handle<Texture> = textures.add(Texture::default());
}
```

### 资源访问 [#](https://bevy.org/news/introducing-bevy/#asset-access)

```rs
fn read_texture_system(textures: Res<Assets<Texture>>, texture_handle: &Handle<Texture>) {
    // 使用当前实体的 Handle<Texture> 组件检索一个 Texture
    let texture: &Texture = textures.get(texture_handle).unwrap();
}
```

### 资源事件 [#](https://bevy.org/news/introducing-bevy/#asset-events)

`Assets<T>` 集合基本上就是一个从 `Handle<T>` 到 `T` 的映射，它会记录创建、修改和移除的 [`Events`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.Events.html)。这些事件也可以作为系统资源消费，就像任何其他 [`Events`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.Events.html) 一样：

```rs
fn system(mut state: Local<State>, texture_events: Res<Events<AssetEvent>>) {
    for event in state.reader.iter(&texture_events) {
        if let AssetEvent::Created { handle } = event {
            /* 对创建的资源进行处理 */
        }
    }
}
```

### 资源服务器 [#](https://bevy.org/news/introducing-bevy/#asset-server)

`Assets<T>` 集合对文件系统或多线程一无所知。这是 [`AssetServer`](https://docs.rs/bevy/0.1.0/bevy/prelude/struct.AssetServer.html) 资源的职责：

```rs
fn system(mut commands: Commands, asset_server: Res<AssetServer>, mut textures: ResMut<Assets<Texture>>) {
    // 这将异步开始并行加载 "texture.png"
    let texture_handle: Handle<Texture> = asset_server.load("texture.png").unwrap();

    // 纹理可能尚未加载完成，但你仍然可以立即将句柄作为组件添加。
    // 只要可能，内部的 Bevy 系统会等待资源准备好后再使用它们：
    let entity = commands.spawn((texture_handle,));

    // 你也可以通过将整个文件夹添加为 "asset folder" 来异步加载整个文件夹（递归地）
    asset_server.load_asset_folder("assets").unwrap();

    // 你可以像这样获取任何资源（无论是正在加载还是已加载）的句柄：
    let music_handle: Handle<AudioSource> = asset_server.get_handle("assets/music.mp3").unwrap(); 

    // 当资源加载完成时，它们会自动添加到相应的 Assets<T> 集合中
    // 你可以像这样检查资源是否就绪：
    if let Some(texture) = textures.get(&texture_handle) {
        // 对纹理进行处理
    }

    // 有时你需要立即访问资源。你可以使用 "load_sync" 方法阻塞当前系统，
    // 直到资源加载完成并立即更新 Assets<T>
    let cool_sprite: &Texture =  asset_server.load_sync(&mut textures, "assets/cool_sprite.png").unwrap();
}
```

### 热重载 [#](https://bevy.org/news/introducing-bevy/#hot-reloading)

你可以通过调用以下代码启用资源变更检测：

```rs
asset_server.watch_for_changes().unwrap();
```

这将在资源文件发生变化时加载其新版本。

### 添加新的资源类型 [#](https://bevy.org/news/introducing-bevy/#adding-new-asset-types)

要添加新的资源类型，请实现 [`AssetLoader`](https://docs.rs/bevy_asset/0.1.0/bevy_asset/trait.AssetLoader.html) trait。这告诉 Bevy 要查找哪些文件格式以及如何将文件字节转换为给定的资源类型。

一旦你为 `MyAssetLoader` 实现了 `AssetLoader<MyAsset>`，你可以像这样注册新的加载器：

```rs
app.add_asset_loader::<MyAsset, MyAssetLoader>();
```

然后你就可以访问 `Assets<MyAsset>` 资源，监听变更事件，并调用 `asset_server.load("something.my_asset")`。

## 音效 [#](https://bevy.org/news/introducing-bevy/#sound)

目前你可以像这样加载和播放音效：

```rs
fn system(asset_server: Res<AssetServer>, audio_output: Res<AudioOutput>) {
    let music: Handle<AudioSource> = asset_server.load("music.mp3").unwrap(); 

    // 这将在音乐加载完成后立即异步播放
    audio_output.play(music);

    // 如果你已经有 AudioSource 引用，你可以像这样立即播放：
    audio_output.play_source(audio_source);
}
```

我们计划在未来扩展音频系统，提供更多的控制和功能。

## 渲染图 [#](https://bevy.org/news/introducing-bevy/#render-graph)

所有渲染逻辑都构建在 Bevy 的 [`RenderGraph`](https://docs.rs/bevy_render/0.1.0/bevy_render/render_graph/struct.RenderGraph.html) 之上。渲染图是一种编码渲染逻辑原子单元的方式。例如，你可能为 2D 通道、UI 通道、相机、纹理拷贝、交换链等创建图节点。将一个节点连接到另一个节点表示它们之间存在某种依赖关系。通过以这种方式编码渲染逻辑，Bevy 渲染器能够分析依赖关系并并行渲染图。它还具有鼓励开发者编写模块化渲染逻辑的好处。

Bevy 默认包含多个节点：`CameraNode`、`PassNode`、`RenderResourcesNode`、`SharedBuffersNode`、`TextureCopyNode`、`WindowSwapChainNode` 和 `WindowTextureNode`。它还提供了用于 2D 渲染、3D 渲染和 UI 渲染的子图。但欢迎你创建自己的节点、自己的图，或扩展包含的图！

### [数据驱动着色器](https://github.com/bevyengine/bevy/blob/1d68094f59b01e14f44ed7db8907dbd011b59973/examples/shader/shader_custom_material.rs) [#](https://bevy.org/news/introducing-bevy/#data-driven-shaders)

组件和资源可以派生 [`RenderResources`](https://docs.rs/bevy_render/0.1.0/bevy_render/renderer/trait.RenderResources.html) trait，这使得它们可以直接复制到 GPU 资源并用作着色器 uniform。

将 uniform 绑定到自定义着色器实际上就像在你的组件或资源上派生 [`RenderResources`](https://docs.rs/bevy_render/0.1.0/bevy_render/renderer/trait.RenderResources.html) 一样简单：

```rs
#[derive(RenderResources, Default)]
struct MyMaterial {
    pub color: Color,
}
```

然后在渲染图中添加一个新的 RenderResourceNode：

```rs
// 创建新节点
render_graph.add_system_node("my_material", RenderResourcesNode::<MyMaterial>::new(true));

// 将新节点连接到"主通道节点"
render_graph.add_node_edge("my_material", base::node::MAIN_PASS).unwrap();
```

此后，MyMaterial 组件将自动复制到 GPU 缓冲区。着色器可以像这样引用实体的 MyMaterial：

```c
layout(set = 1, binding = 1) uniform MyMaterial_color {
    vec4 color;
};
```

我认为[完全自包含的自定义着色器示例](https://github.com/bevyengine/bevy/blob/1d68094f59b01e14f44ed7db8907dbd011b59973/examples/shader/shader_custom_material.rs)的简洁性不言自明。

### [着色器定义](https://github.com/bevyengine/bevy/blob/1d68094f59b01e14f44ed7db8907dbd011b59973/examples/shader/shader_defs.rs) [#](https://bevy.org/news/introducing-bevy/#shader-defs)

组件和资源还可以添加"着色器定义"来按实体选择性地启用着色器代码：

```rs
#[derive(RenderResources, ShaderDefs, Default)]
struct MyMaterial {
    pub color: Color,
    #[render_resource(ignore)]
    #[shader_def]
    pub always_blue: bool,
}
```

然后在你的片元着色器中，你可以这样做：

```c
void main() {
    o_Target = color;
# ifdef MYMATERIAL_ALWAYS_BLUE
    o_Target = vec4(0.0, 0.0, 1.0, 1.0);
# endif
}
```

任何带有 `MyMaterial` 组件且 `always_blue: true` 的实体都将被渲染为蓝色。如果 `always_blue` 为 false，则将使用 `color` 渲染。

我们目前使用这个功能来切换"无光照"渲染和可选纹理，但我预计它在各种上下文中都会有用。

### [着色器布局反射](https://github.com/bevyengine/bevy/blob/1d68094f59b01e14f44ed7db8907dbd011b59973/examples/shader/shader_custom_material.rs) [#](https://bevy.org/news/introducing-bevy/#shader-layout-reflection)

Bevy 可以从 SpirV 着色器（以及通过编译为 SpirV 的 GLSL 着色器）自动反射着色器数据布局。这意味着创建自定义着色器就像这样简单：

```rs
let shader_stages = ShaderStages {
    vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
    fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
};
let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(shader_stages));
```

## 高效的编译时间 [#](https://bevy.org/news/introducing-bevy/#productive-compile-times)

我设计 Bevy 的主要目标之一是"高效产出"。游戏开发是一个极度迭代和实验性的过程，充满了小改动。如果每次改动都需要大量时间来测试，那么开发就变成了一种煎熬。以下是我个人对迭代改动的"可接受程度"评分：

- **0-1 秒**：理想
- **1-3 秒**：不错
- **3-5 秒**：烦人
- **5-10 秒**：痛苦但如果你下定决心还能用
- **10 秒以上**：完全不可用

注意，这些是"迭代编译时间"，而不是"全新编译时间"。全新编译只需要发生一次，而迭代编译则不断发生。就生产效率而言，我对"全新编译"指标关注得少得多，尽管出于其他原因，保持全新编译时间较低仍然很重要。

当今最流行的 Rust 引擎之一需要*超过 30 秒*来编译在简单示例中插入的一个换行。这绝对是没有效率的，使得真正的游戏开发几乎不可能。

目前，使用"快速编译"配置，对 Bevy 示例的更改可以在 ~0.8-3 秒内编译完成，具体取决于你的计算机规格、配置和操作系统选择（稍后会详细介绍）。当然，在这方面总有改进空间，但 Bevy 目前落入了我的"可用性最佳点"。

"Rust 编译慢"这个梗的存在，很大程度上是因为许多 Rust 项目没有充分考虑某些代码模式对编译时间性能的影响。Rust 代码编译慢通常有三个原因：

- **泛型单态化**：将泛型代码转换为非泛型副本的编译步骤。随着单态化代码量的增加，编译时间会增加。为了保持低成本，你应该要么完全避免泛型，要么保持泛型代码"小"且浅。
- **链接时间**：链接代码所需的时间。这里重要的是保持代码量和依赖数量较低。
- **LLVM**：Rust 向 LLVM 抛出了大量的 IR 代码并期望它优化。这需要时间。此外，LLVM 针对"运行时快速代码"的优化程度高于"快速代码生成"。

LLVM 部分不在我们的控制范围内（目前）。保持泛型使用量低且浅并不是一个特别困难的问题，前提是你从一开始就采用这种思维。另一方面，链接时间是迭代编译时间的一个持续且非常真实的"敌人"。每次迭代编译都会发生链接。向你的项目添加任何代码都会增加链接时间。向你的项目添加任何依赖都会增加链接时间。

由于多种原因，情况对我们不利：

- **游戏引擎领域**
    - 游戏引擎天生涉及大量领域（因此涉及大量依赖）
    - 游戏引擎是"庞大的"……它们需要大量代码
- **Rust 的设计选择**
    - 依赖默认是静态链接的，这意味着每个新依赖都会增加链接时间
    - Rust 的默认链接器相当慢
    - Cargo 使得引入依赖非常容易。一个看似小巧简单的 crate 可能实际上有一个巨大的依赖树

解决这个问题的一个方法是不惜一切代价避免依赖，并编写尽可能少的代码。[Macroquad](https://github.com/not-fl3/macroquad) 项目就是一个很好的例子。他们对代码采用极简主义方法，避免任何不符合严格编译时间要求的依赖。结果，我认为公平地说，他们是编译最快的（同时仍然可用的）Rust 游戏引擎，无论是全新编译还是迭代编译。然而，他们的方法是以回避依赖为代价的。

Bevy 采取了一种稍微更务实的方法。首先，愿意引入依赖对 Rust 生态系统是有好处的。我不想忽视已经完成的所有出色工作，特别是像 [winit](https://github.com/rust-windowing/winit) 和 [wgpu](https://github.com/gfx-rs/wgpu-rs) 这样的项目。但我们仍然努力保持依赖树尽可能小。任何将 Bevy 带出"理想到不错"迭代编译时间范围的依赖都必须被缩减或移除。结合"快速编译"配置，这带来了不错的编译时间。

### "快速编译"配置 [#](https://bevy.org/news/introducing-bevy/#the-fast-compiles-configuration)

"快速编译"配置是我们如何在仍然引入依赖的情况下实现可用迭代编译时间的方式。它由三部分组成：

- **LLD 链接器**：LLD 在链接方面比默认的 Rust 链接器*快得多*。这是最大的优势。
- **Nightly Rust 编译器**：可以访问最新的性能改进和"不稳定"优化。请注意，如果这是你的要求，Bevy 仍然可以在稳定版 Rust 上编译。
- **泛型共享**：允许 crate 共享单态化的泛型代码，而不是复制它。在某些情况下，这允许我们"预编译"泛型代码，使其不影响迭代编译。

要启用快速编译，请安装 nightly Rust 编译器和 LLD。然后将[此文件](https://github.com/bevyengine/bevy/blob/1d68094f59b01e14f44ed7db8907dbd011b59973/.cargo/config_fast_builds)复制到 `YOUR_WORKSPACE/.cargo/config`。

### 当前限制和未来改进 [#](https://bevy.org/news/introducing-bevy/#current-limitations-and-future-improvements)

虽然 Bevy 目前按照我的标准是"高效的"，但还不是一切都美好。首先，MacOS 没有最新版本的 LLD 链接器，因此在该平台上的迭代编译*慢得多*。此外，LLD 在 Windows 上比在 Linux 上*稍慢*。在我的机器上，Windows 上大约是 ~1.5-3.0 秒，而 Linux 上是 ~0.8-3.0 秒。

#### 动态链接来救援 [#](https://bevy.org/news/introducing-bevy/#dynamic-linking-to-the-rescue)

减少链接时间的一个简单方法是改用动态链接。在我的 2013 年 MacBook Pro（运行 MacOS，没有 LLD）上，我通过动态链接应用插件，将 Bevy 的迭代编译时间从约 6 秒降到了约 0.6 秒。Bevy 实际上已经支持动态 App 插件，但新的 Bevy ECS 目前不支持动态链接，因为它依赖 TypeId（与动态链接不兼容）。幸运的是，我已经在其他项目中解决了 TypeId 问题，所以我们很快就能重新添加这个功能。

#### Cranelift Rustc 后端 [#](https://bevy.org/news/introducing-bevy/#cranelift-rustc-backend)

Cranelift 是一个备选的编译器后端，专为快速编译而优化。[rustc cranelift 后端](https://github.com/bjorn3/rustc_codegen_cranelift)正在迅速接近可用状态。我希望它最终能给我们带来不错的提升。

## 示例游戏：Breakout [#](https://bevy.org/news/introducing-bevy/#example-game-breakout)

如果你好奇真正的 Bevy 游戏代码长什么样，请查看 [breakout 示例](https://github.com/bevyengine/bevy/blob/1d68094f59b01e14f44ed7db8907dbd011b59973/examples/game/breakout.rs)。请原谅我略显粗糙的碰撞检测代码 :)

## 为什么要构建 Bevy？ [#](https://bevy.org/news/introducing-bevy/#why-build-bevy)

市面上已经有大量优秀的引擎了……为什么还要再做一个？特别是当 Rust 生态系统中已经有这么多的时候？

首先介绍一下我自己：我决定在多年为其他引擎（例如 Godot）贡献代码之后构建 Bevy。我花了四年多的时间[用 Godot 构建了一个游戏](https://www.youtube.com/c/cartdev)，我也有 Unity、Unreal 以及许多其他框架（如 SDL 和 Three.js）的经验。我过去使用 Rust、Go、HTML5 和 Java 构建过多个自定义引擎。我也使用过或密切关注过 Rust 游戏开发生态系统中的大多数现有参与者。我最近辞去了微软高级软件工程师的工作，我的这段经历深刻影响了我对软件及其应有形态的看法。

这些经历让我渴望从一个游戏引擎中得到以下东西：

- **免费且开源**：它必须是免费且开源的，*没有任何附加条件*。游戏是我们文化的重要组成部分，人类正在投入*数百万*小时进行游戏开发。为什么我们（作为游戏开发者/引擎开发者）还要继续建设那些从我们的销售中抽成并拒绝让我们了解日常使用的技术的闭源垄断生态系统？作为一个社区，我相信我们可以做得更好。这个标准排除了 Unreal 和 Unity，尽管它们拥有庞大的功能集。
- **高效产出**：它需要有快速的构建/运行/测试循环，这意味着要么使用脚本语言，要么使用原生语言且编译速度快。但脚本语言会引入运行时开销、认知负荷，以及在我和实际引擎之间的隔阂，所以我更倾向于编译速度快的原生语言。遗憾的是，编译时间是 Rust 生态系统中的一个巨大问题，许多 Rust 引擎的迭代编译时间过长。幸运的是，像 Macroquad 和 coffee 这样的 Rust 游戏引擎证明了高效的迭代编译时间是可能的。
- **全栈透明（Turtles All The Way Down）**：理想情况下，引擎和游戏使用同一种语言编写。能够在你的游戏中对一个符号执行 IDE 的"转到定义"命令，然后直接跳转到引擎源码，这是一个极其强大的概念。你也不需要担心繁重的语言转换层或有损抽象。如果一个引擎的社区使用与引擎相同的语言构建游戏，他们更有可能（也有能力）回馈引擎。
- **简单**：它需要易于用于常见任务，但同时不能对你隐藏细节。许多引擎要么"易于使用但过于高层"，要么"非常底层但难以执行常见任务"。此外，Rust 中的许多引擎充斥着生命周期和泛型。两者当然都是强大的工具，但它们也会引入认知负荷并降低人体工程学。泛型如果不小心，还会对编译时间产生巨大影响。
- **编辑器**：它需要有一个（可选的）图形编辑器。场景创建是游戏开发的一大部分，在许多情况下，可视化编辑器胜于代码。作为额外的好处，编辑器应该构建在*引擎内部*。Godot 使用了这种方法，这*非常明智*。这样做可以[吃自己的狗粮](https://en.wikipedia.org/wiki/Eating_your_own_dog_food)，使用引擎自己的 UI 系统并创造正反馈循环。对编辑器的改进也常常是对核心引擎的改进。它还确保你的引擎足够灵活，可以构建工具（而不仅仅是游戏）。我个人认为在另一个技术栈中构建引擎的编辑器是一种错失的机会（例如：Web、QT、原生控件）。
- **数据驱动**：它需要是数据驱动/数据导向/数据优先的。ECS 是实现这一点的一种常见方式，但绝对不是唯一的方式。这些范式可以使你的游戏更快（缓存友好，更容易并行化），但它们也使游戏状态序列化和同步等常见任务变得出奇地简单直接。

市面上没有一款引擎能*完全*符合我的要求。而让它们满足我的要求所需的变化要么范围太大，要么不可能做到（闭源），要么不受欢迎（我想要的东西不是开发者或客户想要的）。最重要的是，制作新引擎很有趣！

Bevy 并不是要击败其他开源游戏引擎。我们应尽可能地进行协作，构建共同的基础。如果你是一个开源游戏引擎开发者，并且你认为 Bevy 的某个组件可以让你的引擎更好，或者你引擎的某个组件可以让 Bevy 更好，或者两者兼有，请联系我们！Bevy 已经从 Rust 游戏开发生态系统的努力中受益匪浅，我们很乐意以任何方式回馈。

## 接下来是什么？ [#](https://bevy.org/news/introducing-bevy/#what-s-next)

我为 Bevy 在相对较短的时间内取得的进展感到自豪，但仍有很多工作要做。以下将是我们未来几个月的重点领域：

### 基于物理的渲染（PBR） [#](https://bevy.org/news/introducing-bevy/#physically-based-rendering-pbr)

Bevy 当前的 3D 渲染器极其简陋。由于我主要制作 3D 游戏，改进 3D 渲染器是我的一个优先事项。我们将添加 PBR 着色器、阴影、更多光照选项、骨骼动画、改进的 GLTF 导入、环境光遮蔽（实现方式待定），以及可能一堆其他的东西。

### 编辑器 [#](https://bevy.org/news/introducing-bevy/#editor)

Bevy 的架构从一开始就考虑到了可视化编辑器。场景和属性系统是专门为使游戏<->编辑器的数据流更加顺畅而构建的。编辑器将作为一个 Bevy App 构建，并利用现有的 Bevy UI、Bevy ECS、场景和属性功能。我喜欢"在引擎中构建编辑器"的方法，因为对编辑器的改进往往也是对引擎的改进，反之亦然。此外，这确保了 Bevy 能够构建非游戏应用和工具。

### 平台支持：Android、iOS、Web [#](https://bevy.org/news/introducing-bevy/#platform-support-android-ios-web)

在底层，Bevy 使用 [winit](https://github.com/rust-windowing/winit)（用于跨平台窗口和输入）和 [wgpu](https://github.com/gfx-rs/wgpu-rs)（用于跨平台渲染）。这些项目各自对上述平台有不同程度的支持。总的来说，Bevy 被设计为平台无关的，因此只需一点点工作就应该能支持上述平台。

### 渲染批处理和实例化 [#](https://bevy.org/news/introducing-bevy/#render-batching-and-instancing)

目前 Bevy 在大多数用例中渲染速度足够快，但在渲染大量对象（数万个）时，它还不能完全胜任。要实现这一点，我们需要实现批处理/实例化。这些概念可以有多种定义方式，但总体思路是，我们将尽可能多的几何体和数据分组到最少数量的绘制调用中，同时尽可能减少 GPU 状态变化。我希望 Bevy 的数据驱动着色器方法能使实例化实现变得简单且可扩展。

### Canvas [#](https://bevy.org/news/introducing-bevy/#canvas)

目前绘制 UI 和 2D 场景的唯一方式是通过精灵和矩形。Bevy 需要一个即时模式的绘图 API，能够绘制抗锯齿曲线和形状。这可以用于在 Bevy UI 中绘制圆角、编辑器中的性能图等。我们很可能会集成像 [pathfinder](https://github.com/servo/pathfinder) 或 [lyon](https://github.com/nical/lyon) 这样的项目来实现这一点。

### 动画 [#](https://bevy.org/news/introducing-bevy/#animation)

动画渗透到游戏开发的几乎每一个方面。首先，我想添加一个通用的代码优先动画系统。然后在此基础上，我们将添加一个基于属性的时间轴系统，可以保存到配置文件，并在 Bevy 编辑器中可视化/编辑。

### 更好的场景格式 [#](https://bevy.org/news/introducing-bevy/#nicer-scene-format)

当前的场景格式是可用的，但因为它是一个扁平的、无序的实体列表，对于手动场景组合来说还不够理想。我还想添加嵌套场景。最终，我希望场景格式[看起来像这样](https://gist.github.com/cart/3e77d6537e1a0979a69de5c6749b6bcb)。

### 动态插件加载 [#](https://bevy.org/news/introducing-bevy/#dynamic-plugin-loading)

为了减轻编译和链接插件的成本，并使热代码重载成为可能，我们将提供动态加载 App 插件的选项。Bevy 实际上已经支持这个功能，但有一个障碍：Rust 的 `TypeId`。TypeId 在不同的二进制文件之间是不稳定的，这意味着宿主二进制文件中的 `TypeId::of::<T>()` 不会匹配动态加载的二进制文件中的 `TypeId::of::<T>()`。Bevy ECS 使用 TypeId，这意味着动态加载的 ECS 类型将无法正确运行。在过去，Bevy 使用了一个自定义分支的 Legion ECS（我们在那里修复了 TypeId 问题）。但自从迁移到 Bevy ECS 后，这个问题重新出现了。解决方法是将我们在 Legion 中使用的相同方法应用于 Bevy ECS。

### 物理引擎 [#](https://bevy.org/news/introducing-bevy/#physics)

许多游戏需要碰撞检测和物理引擎。我计划构建一个可插拔的物理接口，以 [nphysics / ncollide](https://nphysics.org/) 作为第一个后端。

### 打磨 [#](https://bevy.org/news/introducing-bevy/#polish)

还有很多方面需要更多设计工作或功能。例如，我认为核心渲染图处于相当好的状态，但中层次和高层次的渲染 API 需要更多时间和实验。我还想重新思考材质是如何组合的。如果你对我们关注的所有改进感兴趣，请查看 [Bevy 的 GitHub 问题跟踪器](https://github.com/bevyengine/bevy/issues)。

### 文档 [#](https://bevy.org/news/introducing-bevy/#documentation)

Bevy 的 API 仍然相当不稳定，所以文档也不完善。[Bevy 手册](https://bevy.org/learn/book/welcome/)只涵盖了引擎中相对成熟的领域，而 [Rust API 文档](https://docs.rs/bevy)也有大量空白。总的来说，我赞同"文档与稳定性成比例"的理念。随着功能的稳定和设计模式的出现，我们将在这两个领域加大投入。

## 加入 Bevy！ [#](https://bevy.org/news/introducing-bevy/#join-the-bevy)

如果这些内容中有任何让你感兴趣的，我鼓励你去看看 [GitHub 上的 Bevy](https://github.com/bevyengine/bevy)，阅读[快速入门指南](https://bevy.org/learn/quick-start/introduction)，并[加入 Bevy 社区](https://bevy.org/community/)。目前 Bevy 完全由志愿者构建，所以如果你想帮助我们构建下一个伟大的游戏引擎，请[联系我们](https://discord.com/invite/gMUk5Ph)！我们需要一切能得到的帮助，特别是如果你是：

- **软件开发者**：请查看 Bevy 手册中的[贡献代码](https://bevy.org/learn/quick-start/contributing/code)部分。
- **技术写作者**：请查看 Bevy 手册中的[贡献文档](https://bevy.org/learn/quick-start/contributing/docs)部分。

我希望 Bevy 能够成为一个充满活力的开发者社区……这实际上也是我选择这个名字的原因！Bevy（一群鸟）正像我们这群游戏开发者。加入 Bevy 吧！
