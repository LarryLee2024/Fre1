# Bevy 0.19

## 发布于 2026 年 6 月 19 日，Bevy 贡献者

![Fields of Aaru：一款设定在古埃及神秘来世的 cozy 生活模拟游戏。使用 Bevy 制作！](https://bevy.org/news/bevy-0-19/fields_of_aaru.jpg)

[Fields of Aaru：一款设定在古埃及神秘来世的 cozy 生活模拟游戏。使用 Bevy 制作！](https://store.steampowered.com/app/4410710/Fields_of_Aaru/)

感谢 **261** 位贡献者、**1185** 个拉取请求、社区审阅者以及我们的[**慷慨捐赠者**](https://bevy.org/donate)，我们很高兴在 [crates.io](https://crates.io/crates/bevy) 上发布 **Bevy 0.19**！

对于那些还不了解的人，Bevy 是一个用 Rust 构建的、令人耳目一新的简单数据驱动游戏引擎。您可以查看我们的[快速入门指南](https://bevy.org/learn/quick-start)立即试用。它永久免费且开源！您可以在 GitHub 上获取完整[源代码](https://github.com/bevyengine/bevy)。查看 [Bevy Assets](https://bevy.org/assets) 获取社区开发的插件、游戏和学习资源集合。

要将现有的 Bevy App 或 Plugin 更新到 **Bevy 0.19**，请查看我们的[0.18 到 0.19 迁移指南](https://bevy.org/learn/migration-guides/0-18-to-0-19/)。

自几个月前上一个版本发布以来，我们添加了大量新功能、错误修复和生活质量改进，以下是其中一些亮点：

- **下一代场景（Next Generation Scenes）**：我们全新、大幅改进的 Bevy 场景系统终于上线了！通过 `bsn!` 宏在代码中以新的 BSN（Bevy Scene Notation）格式（或在未来版本中以资源文件形式）符合人体工学地定义场景。场景可组合、可修补、可感知依赖。不再需要手动拉入生成某物所需的所有 ECS 和资源依赖！
- **更快渲染更大场景**：我们将更多工作移到了 GPU，并在多个方面优化了渲染器。Bevy 可以更快地绘制更多内容！
- **接触阴影（Contact Shadows）**：阴影质量对游戏"精致度"影响巨大。Bevy 0.19 加入了接触阴影，在不增加完整光线追踪成本的情况下显著改善了阴影细节。
- **更多 Feathers Widget**：Bevy 自带的"编辑器工具" widget 集合新增了大量 widget。同时已移植到 BSN，使用起来更加愉快！
- **文本输入（Text Input）**：Bevy UI **终于**通过新的 `EditableText` 组件获得了上游对文本输入的支持。
- **更丰富的文本（Richer Text）**：Bevy 现在有更灵活的字体选择，支持"字体族"和可变字体属性等高级功能。
- **应用设置（App Settings）**：我们添加了一个官方的"应用设置"框架，可以从文件加载和保存设置，并将其暴露为 ECS 资源。
- **后处理效果（Post Processing Effects）**：我们添加了内置的"暗角"和"镜头畸变"后处理效果。
- **改进的蒙皮网格剔除**：蒙皮网格现在在进行剔除时可以考虑其动画信息。

## 下一代场景（Next Generation Scenes）[#](https://bevy.org/news/bevy-0-19/#next-generation-scenes)

作者：[carrt](https://github.com/cart)

PR：[#23413](https://github.com/bevyengine/bevy/pull/23413), [#23880](https://github.com/bevyengine/bevy/pull/23880), [#23808](https://github.com/bevyengine/bevy/pull/23808), [#23905](https://github.com/bevyengine/bevy/pull/23905), [#24008](https://github.com/bevyengine/bevy/pull/24008)

![bsn](https://bevy.org/news/bevy-0-19/bsn.png)

**Bevy 0.19** 引入了我们全新、大幅改进的 Bevy 场景系统。我们已经为此工作了很长时间（已经好几年了！），我们很高兴终于将其交到 Bevy 开发者手中。它使得在代码中（以及最终在即将推出的 Bevy Editor 生成的资源中）定义场景变得更加优雅。它还将用于**构建**即将推出的 Bevy Editor！

### BSN（Bevy Scene Notation）[#](https://bevy.org/news/bevy-0-19/#bsn-bevy-scene-notation)

BSN 是一种符合人体工学的、类似 Rust 的场景语法，可以通过 [`bsn!`](https://docs.rs/bevy_scene/0.19.0-rc.3/bevy_scene/macro.bsn.html) 宏在 Rust 代码中定义，也可以在 `.bsn` 资源文件中定义。如果你曾经被在 Bevy 中生成复杂实体集合时的冗长和复杂性所困扰，你可能会喜欢 BSN 提供的一切。BSN 可用于生成 ECS 中的任何内容。这使所有场景受益，但值得一提的是，这使 Bevy UI 代码的读写变得显著更容易。

一些快速说明：虽然 **Bevy 0.19** 在技术上支持场景资源文件，但我们尚未提供官方的 `.bsn` 资源加载器。本版本侧重于代码驱动的工作流，我们计划在未来的版本中推出资源驱动的工作流。此外，BSN 刚刚出炉，可能还需要几个版本来完善体验。目前它已经非常有用，但请预料到一些粗糙的边缘和缺失的功能。

在 Rust 中，[`bsn!`](https://docs.rs/bevy_scene/0.19.0-rc.3/bevy_scene/macro.bsn.html) 表达式本质上是一个要添加到实体上的组件列表：

```rust
bsn! {
    Player {
        score: 0
    }
    Team::Blue
}
```

到目前为止，这看起来和行为很像 Bevy 现有的 [`Bundle`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/bundle/trait.Bundle.html)（它**只是**一组组件的集合）。但 BSN 拥有大量额外的超能力！

**点击这里查看 BSN 提供的一切！**

### 可选字段（Optional Fields）[#](https://bevy.org/news/bevy-0-19/#optional-fields)

在 BSN 中，你不需要指定每个字段，也不需要写 `..Default::default()`。你只需要设置你关心的字段，其余的将使用其默认值：

```rust
#[derive(Component, Default, Clone)]
struct Player {
    score: usize,
    coins: usize,
}

bsn! {
    Player {
        score: 0
    }
}
```

如果你希望所有字段都使用默认值，也可以只指定类型名称：

```rust
bsn! {
    Player
}
```

字段值可以通过 `{}` 语法使用任意 Rust 表达式：

```rust
bsn! {
    Player { score: {current_points + 10} }
}
```

### BSN 关系（BSN Relationships）[#](https://bevy.org/news/bevy-0-19/#bsn-relationships)

BSN 对 ECS 关系有一流支持。你可以内联生成相关实体（如子实体）：

```rust
bsn! {
    Player
    Children [
        Sword,
        Shield,
    ]
}
```

这也适用于自定义关系：

```rust
bsn! {
    Player
    Inventory [
        Apple,
        Potion,
    ]
}
```

### 场景函数（Scene Functions）[#](https://bevy.org/news/bevy-0-19/#scene-functions)

[`bsn!`](https://docs.rs/bevy_scene/0.19.0-rc.3/bevy_scene/macro.bsn.html) 返回一个实现了 [`Scene`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.Scene.html) trait 的类型，这意味着你可以像这样定义可复用的 BSN 函数：

```rust
fn player() -> impl Scene {
    bsn! {
        Player
        Children [ Sword, Shield ]
    }
}
```

这些函数可以接受并使用参数：

```rust
fn player(name: &str) -> impl Scene {
    bsn! {
        Name(name)
        Player
    }
}
```

### 场景是可组合的补丁（Scenes are Composable Patches）[#](https://bevy.org/news/bevy-0-19/#scenes-are-composable-patches)

BSN 表达式是一个"补丁"；它不会写入其定义的每个类型的"完整"实例。这意味着你可以将场景层层叠加：

```rust
fn button() -> impl Scene {
    bsn! {
        Button
        Node { width: px(100) }
    }
}

fn my_button() -> impl Scene {
    bsn! {
        button()
        Node { height: px(100) }
    }
}
```

`my_button` 将生成一个带有 `Node { width: px(100), height: px(100) }` 组件的实体。场景中的组件会被初始化为其默认值，每多一层场景都会在其之上写入字段值。

### 场景资源与缓存（Scene Assets and Caching）[#](https://bevy.org/news/bevy-0-19/#scene-assets-and-caching)

虽然 **Bevy 0.19** 没有附带官方的 `.bsn` 资源加载器，但它在功能上**已经**支持场景资源依赖。我们只是还没有包含任何内置加载器：

```rust
commands.queue_spawn_scene(bsn! {
    :"player.bsn"
    Transform {
        translation: Vec3 { x: 10. }
    }
})
```

这样（如果有 `.bsn` 资源加载器的话）将生成一个包含 `"player.bsn"` 场景资源的场景，并将"x 位置"修补为 `10`。BSN 是感知依赖的：如果你使用 `queue_spawn_scene` 而不是 `spawn_scene`，它会等待所有依赖加载完成后才生成场景。`spawn_scene` 会始终尝试立即生成场景……如果它的场景依赖尚未加载，则会失败。

另外请注意 `:`，这是"缓存"语法。首次加载时，它将解析 `"player.bsn"` 场景并缓存结果以供复用。这使得生成多个场景实例更加高效，因为它只需要解析缓存在场景"之上"叠加的内容。

我们正在[开发](https://github.com/bevyengine/bevy/pull/23576)官方的 `.bsn` 资源加载器，同时计划将 Bevy 的 glTF 加载器移植到新的场景系统（这样你就可以依赖 `"my_scene.gltf"`，就像使用 `my_scene.bsn` 文件一样）。`bsn!` 宏和生成系统已经支持场景资源，所以如果你喜欢冒险，可以在等待我们的官方加载器时尝试实现自己的 Bevy 场景资源格式！

### 场景列表（Scene Lists）[#](https://bevy.org/news/bevy-0-19/#scene-lists)

[`bsn!`](https://docs.rs/bevy_scene/0.19.0-rc.3/bevy_scene/macro.bsn.html) / [`Scene`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.Scene.html) 对应单个实体。[`bsn_list!`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/macro.bsn_list.html) / [`SceneList`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.SceneList.html) 是相同的概念，但应用于实体列表：

```rust
fn players() -> impl SceneList {
    bsn_list! [
        (#Player1 Team::Blue),
        (#Player2 Team::Red),
    ]
}
```

[`bsn_list!`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/macro.bsn_list.html) 中的实体以逗号分隔，用来视觉上标识实体边界的括号是可选的：

```rust
fn players() -> impl SceneList {
    bsn_list! [
        #Player1 Team::Blue,
        #Player2 Team::Red,
    ]
}
```

上面看到的"BSN 关系语法"（例如：`Children []`）使用了 [`SceneList`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.SceneList.html)。这意味着你可以将场景列表作为参数传递给场景：

```rust
fn widget(children: impl SceneList) -> impl Scene {
    bsn! {
        Widget
        Children [ {children} ]
    }
}
```

### 观察事件（Observing Events）[#](https://bevy.org/news/bevy-0-19/#observing-events)

[`bsn!`](https://docs.rs/bevy_scene/0.19.0-rc.3/bevy_scene/macro.bsn.html) 实体可以轻松观察事件，使得在场景中嵌入"回调风格"的行为变得简单：

```rust
fn button() -> impl Scene {
    bsn! {
        Node { width: px(100), height: px(50) }
        on(|press: On<Pointer<Press>>| {
            info!("button pressed!")
        })
    }
}
```

### 模板（Templates）[#](https://bevy.org/news/bevy-0-19/#templates)

BSN 表达式实际上为组件定义的是"模板"而非实际的组件本身。[`Template`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.Template.html) 本质上是一个类型的精巧构造器，它产生一个输出类型（如 Component）。关键是，[`Template`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.Template.html) 可以访问 [`World`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/struct.World.html)、当前实体和"场景生成上下文"。这使得强大的行为成为可能，例如从给定资源路径加载资源并生成资源句柄（如 `Handle<Image>`）。

通过 Bundle 生成的"旧"方法需要将每个 ECS 依赖都传入 Bundle 函数，并手动使用该依赖来产生最终值：

```rust
fn player(asset_server: &AssetServer) -> impl Bundle {
    (
        Player {
            score: 10,
            ..Default::default()
        },
        children! [
            Sprite {
                image: asset_server.load("player.png"),
                ..Default::default()
            }
        ]
    )
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(player(&asset_server))
}
```

在生成具有许多依赖的复杂深层嵌套场景时，这变得**非常**糟糕。

BSN 让这一切变得容易得多：

```rust
fn player() -> impl Scene {
    bsn! {
        Player { score: 10 }
        Children [
            Sprite { image: "player.png" }
        ]
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_scene(player());
}
```

生成场景不再需要知道它内部需要的每一个小依赖，诸如通过路径加载和分配资源等常见操作也变得简单！

这确实意味着 BSN 要求类型拥有 [`Template`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.Template.html)。这是通过 [`FromTemplate`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.FromTemplate.html) trait 实现的，它告诉 BSN 对于给定的 [`Component`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/component/trait.Component.html) 应该使用什么 [`Template`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.Template.html) 类型。[`FromTemplate`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.FromTemplate.html) 可以派生，这也会为你的类型生成一个 [`Template`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.Template.html) 类型。幸运的是，大多数类型**不需要**手动派生或实现 [`FromTemplate`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.FromTemplate.html)。这是因为 [`FromTemplate`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.FromTemplate.html) 和 [`Template`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.Template.html) 会自动为每个实现了 `Default` 和 `Clone` 的类型实现。这些类型是"自己的模板"，只是被"透传"。只有当你需要模板功能时（例如上面的 [`Sprite`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.Sprite.html) 用例，它使用 `Handle<Image>` 模板来接受 `"player.png"`），才需要派生 [`FromTemplate`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.FromTemplate.html)。

### 内联资源模板（Inline Asset Templates）[#](https://bevy.org/news/bevy-0-19/#inline-asset-templates)

BSN 通过 `asset_value` 模板内置了对"内联资源"的支持：

```rust
fn cube() -> impl Scene {
    bsn! {
        Mesh3d(asset_value(Cuboid::new(1., 1., 1.)))
    }
}
```

对比一下以前需要做的事情！

```rust
fn setup(meshes: Res<Assets<Mesh>>) -> impl Bundle {
    let handle = meshes.add(Cuboid::new(1., 1., 1.));
    Mesh3d(handle)
}
```

### 实体引用语法（Entity Reference Syntax）[#](https://bevy.org/news/bevy-0-19/#entity-reference-syntax)

BSN 有特殊的"实体引用语法"来定义实体的 [`Name`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/name/struct.Name.html) 组件：

```rust
bsn! {
    #FirstPlayer
    Player
}
```

这基本上等同于：

```rust
bsn! {
    Name("FirstPlayer")
    Player
}
```

然而"实体引用语法"还允许在场景中的其他地方引用该实体：

```rust
#[derive(Component, FromTemplate)]
struct Reference(Entity);

bsn! {
    #Root
    Children [
        Reference(#Root)
    ]
}
```

你可以访问给定 `bsn! {}` 作用域中定义的**任何**实体引用，在该作用域的任何位置：

```rust
bsn! {
    References {
        child: #Child,
        grandchild: #Grandchild,
    }
    Children [
        #Child Children [
            #Grandchild
        ]
    ]
}
```

在 [`bsn_list!`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/macro.bsn_list.html) 的上下文中，这使得定义图结构成为可能：

```rust
bsn_list! [
    (#A PointsTo(#B)),
    (#B PointsTo(#A)),
]
```

### 隐式 Into（Implicit Into）[#](https://bevy.org/news/bevy-0-19/#implicit-into)

"字段位置"上的大多数值支持"隐式 `.into()`"。这意味着可以转换为其他类型的类型通常可以跳过手动转换：

```rust
#[derive(Component, Default, Clone)]
struct Foo(String);

bsn! {
    Foo("hello")
}
```

这是因为 `"hello"` 是一个 `&str`，它实现了 `Into<String>`。这在定义 Bevy UI 值时尤其方便：

```rust
// 原始 Rust
Node {
    border: UiRect::all(Val::Px(2.0)),
    ..Default::default()
}

// BSN
Node { border: px(2) }
```

`px(2)` 只是一个生成 `Val::Px(2.0)` 的函数，而 `UiRect` 有一个针对 `Val` 的 `Into` 实现，会生成 `UiRect::all`（将值写入所有四个边框"侧"）。这里的人体工学可与 CSS 等媲美，但它是完全静态类型的，并且源于普通的 Rust trait 转换（这些不是特例/硬编码的）。这意味着你可以构建自己的转换！

### 场景组件（Scene Components）[#](https://bevy.org/news/bevy-0-19/#scene-components)

定义一个像 `Player` 这样的组件——具有依赖于某个更大"场景"的复杂行为——然后问出诸如"我如何一起生成这些东西？"和"我如何编写可以安全地假设整个场景存在的代码？"这类问题，几乎已经成为 Bevy 开发者的成人礼。Bevy 开发者也通过各种创意方式解决了这些问题，但从未有一个简单的推荐/惯用上游解决方案。

BSN 通过 [`SceneComponent`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.SceneComponent.html) 派生宏，使得将 [`Scene`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.Scene.html) 与 [`Component`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/component/trait.Component.html) 关联起来成为可能，从而解决了这个问题：

```rust
#[derive(SceneComponent, Default, Clone)]
struct Player {
    score: usize
}

impl Player {
    fn scene() -> impl Scene {
        bsn! {
            Transform { translation: Vec3 { x: 10. } }
            Children [
                LeftHand,
                RightHand,
            ]
        }
    }
}
```

场景组件可以这样生成：

```rust
world.spawn_scene(bsn! {
    @Player { score: 10 }
})
```

场景组件必须这样生成（作为"场景组件"），如果直接通过 `world.spawn(Player::default())` 等方式生成，则会记录错误日志。关键是，这提供了保证：如果 `Player` 组件存在，那么在生成时完整场景也一定存在。作为开发者，这意味着你可以编写查询 `Player` 的代码，并假设它同时拥有 `LeftHand` 和 `RightHand` 子实体（前提是它们生成后没有被移除）。这是 Bevy 数据模型中一个长期缺失的重要拼图！

场景组件还可以定义"属性（props）"，这些属性会传入场景函数并可影响 BSN 输出：

```rust
#[derive(SceneComponent, Default, Clone)]
#[scene(PlayerProps)]
struct Player {
    score: usize,
}

#[derive(Default)]
struct PlayerProps {
    alignment: Alignment
}

impl Player {
    fn scene(props: PlayerProps) -> impl Scene {
        let alignment: Box<dyn Scene> = match props.alignment {
            Alignment::Good => Box::new(bsn! { AngelWings }),
            Alignment::Evil => Box::new(bsn! { DevilHorns }),
        };
        bsn! {
            #Player
            alignment
            Items [ Sword, Shield ]
        }
    }
}

bsn! {
    @Player {
        // this is a "prop"
        @alignment: Alignment::Good,
        // this is a normal field
        score: 10,
    }
}
```

"Props" 首先被求值（在组件字段补丁之前）。逻辑上，它们是立即/就地求值的，并且 SceneComponent 的场景会立即应用到当前场景。这意味着它们产生的场景可以被修补。这**也**意味着你不能修补"props"，因为它们在后续的场景中不存在。

[`SceneComponent`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.SceneComponent.html) 派生宏还支持场景资源的简写形式：

```rust
#[derive(SceneComponent, Default, Clone)]
#[scene("player.bsn")]
struct Player {
    score: usize
}
```

再次提醒，**Bevy 0.19** 没有附带 `.bsn` 资源加载器。我们正在开发中！

[`SceneComponent`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.SceneComponent.html) 派生宏默认查找 `Player::scene` 函数，但你也可以指定自定义函数：

```rust
#[derive(SceneComponent, Default, Clone)]
#[scene(player)]
struct Player {
    score: usize
}

fn player() -> impl Scene {
    bsn! { Player }
}
```

### 场景生成系统（Scene Spawning Systems）[#](https://bevy.org/news/bevy-0-19/#scene-spawning-systems)

**Bevy 0.19** 附带了一个辅助工具，可以轻松生成场景函数。这是一个**完全自包含**的 Bevy app，用于生成 2D 场景：

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, level.spawn())
        .run();
}

fn level() -> impl SceneList {
    bsn_list![
        Camera2d,
        Sprite { image: "player.png" }
    ]
}
```

`.spawn()` 会将任何返回 [`Scene`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.Scene.html) 或 [`SceneList`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.SceneList.html) 的函数转换为一个生成该场景的 System。

## 更快渲染大场景（Render Big Scenes Faster!）[#](https://bevy.org/news/bevy-0-19/#render-big-scenes-faster)

作者：[pcwalton](https://github.com/pcwalton)、[aevyrie](https://github.com/aevyrie)、[tychedelia](https://github.com/tychedelia)

PR：[#23242](https://github.com/bevyengine/bevy/pull/23242), [#23481](https://github.com/bevyengine/bevy/pull/23481), [#23711](https://github.com/bevyengine/bevy/pull/23711), [#23036](https://github.com/bevyengine/bevy/pull/23036), [#23211](https://github.com/bevyengine/bevy/pull/23211), [#23023](https://github.com/bevyengine/bevy/pull/23023), [#22966](https://github.com/bevyengine/bevy/pull/22966), [#22874](https://github.com/bevyengine/bevy/pull/22874), [#22988](https://github.com/bevyengine/bevy/pull/22988), [#23106](https://github.com/bevyengine/bevy/pull/23106), [#23115](https://github.com/bevyengine/bevy/pull/23115), [#23530](https://github.com/bevyengine/bevy/pull/23530), [#22813](https://github.com/bevyengine/bevy/pull/22813), [#22297](https://github.com/bevyengine/bevy/pull/22297), [#23185](https://github.com/bevyengine/bevy/pull/23185), [#23297](https://github.com/bevyengine/bevy/pull/23297), [#23103](https://github.com/bevyengine/bevy/pull/23103), [#22846](https://github.com/bevyengine/bevy/pull/22846)

在 **Bevy 0.19** 中，我们继续专注于让大规模场景快速渲染。让我们先看看一些基准测试！

在 **Bevy 0.18** 中，一台搭载移动版 Nvidia RTX 4090 的笔记本电脑在 `many_cubes` 示例中可处理 160 万个立方体网格实体（使用 Bevy 的 PBR StandardMaterial），启用剔除、11.6 万个实体可见，每帧约 49.47ms（21 FPS）。在 **Bevy 0.19** 中，渲染同样的立方体只需 18.77ms（53 FPS）！禁用剔除时（即所有 160 万个立方体都被渲染），同样的示例从 93.1ms 降到了 41.2ms！！

我们新的 [`bevy_city`](https://github.com/bevyengine/bevy/tree/main/examples/large_scenes/bevy_city) 示例默认生成一个包含 55000 个渲染实体的城市：

![bevy_city](https://bevy.org/news/bevy-0-19/bevy_city.jpg)

||Static|Moving|
|---|---|---|
|Bevy 0.18|19.3 ms|22.1 ms|
|Bevy 0.19|11.8 ms|16.2 ms|

这些成果来自于**众多**改进。可以概括为"我们将更多工作移到 GPU，更多批量渲染，更多并行化，以及减少内存访问"。

点击这里查看每个优化的详细说明！

- **在 GPU 上解包多可绘制批次集**：为了使用 GPU 驱动的多间接绘制（multi-draw-indirect）方法进行渲染，Bevy 需要将网格实例分组为"批次集"。过去我们在 CPU 上做这个准备工作，但我们通过在 GPU 上执行"bin 解包"步骤，将大部分工作移到了 GPU。在保持硬件兼容性**并且**使更新低成本的情况下完成这工作很有挑战性！在绘制一百万个立方体时，这为我们节省了近 1.5ms！
- **批量深度预渲染**：对于不需要材质数据或仅写入法线/运动向量的预渲染，我们现在将它们批处理在一起，这可以节省相当可观的绘制调用次数！
- **稀疏网格 uniform 缓冲上传**：Bevy 现在跟踪哪些网格 uniform 发生了变化，并仅将这些更改上传到 GPU（前提是超过一定的大小阈值）。对于具有许多变换未变化的网格的场景，这可以带来巨大的收益！
- **灯光、光照探针和贴花的 GPU 聚类**：Bevy 现在在 GPU 上进行灯光聚类。在我们的 `many_lights` 基准测试中，这使灯光聚类性能提高了约 20 倍！
- **增加的系统并行度**：更多渲染系统并行运行。这显然更快！
- **可见性范围在 GPU 上检查**：我们已经将这些 LOD 检查移到了 GPU。
- **批量变形目标**：带有变形目标的网格现在可以在支持存储缓冲区的平台上批量渲染。在我们的 `many_morph_targets` 示例中，这带来了近 2 倍的加速！
- **`NoCpuCulling` 优化**：带有 `NoCpuCulling` 标记的网格现在完全被 CPU 可见性系统忽略，这为带有该标记组件的网格节省了大量工作。
- **减少"上一帧变换"拷贝**：Bevy 的渲染器需要上一帧的变换数据。我们现在仅在变换被修改时才写入上一帧的变换，节省了宝贵的时间。
- **网格集合使用共享内存直接更新 GPU 数据**：这消除了一个缓慢的顺序瓶颈，节省了宝贵的毫秒数！
- **使用变更列表替代 tick**：我们使 specializetion 和队列系统只处理每帧需要的实体，大幅减少了需要做的工作量！
- **更智能的聚类启发式使用**：我们现在使用上一帧的聚类统计来进行聚类，这效果**更好**，同时也让我们避免了昂贵的内存扫描。
- **更简单的 GPU 内存拷贝**：Bevy 使用 `encase` 库来按照 WGSL 预期的布局准备 GPU 缓冲区数据。实际上，这并不总是必要的，并引入了不必要的开销。我们在灯光聚类时改用更便宜的 direct memory copy，以节省一点时间。
- **并行网格收集**：我们已将网格收集的"聚合"步骤并行化，在 `bevymark_3d` 中渲染 20 万个移动网格时节省了约 4ms。
- **优化的脏变换树标记**：我们现在使用缓冲通道使并行工作线程将脏位从叶子传播到根。这使得具有许多静态对象的场景更快！具有许多动态对象的场景也快了一点！
- **优化的实体移除**：我们现在在移除实体时从列表末尾而不是开头扫描，因为较新的实体比较旧的实体更可能是动态的。

## Solari 改进（Solari Improvements）[#](https://bevy.org/news/bevy-0-19/#solari-improvements)

作者：[JMS55](https://github.com/JMS55)、[dylansechet](https://github.com/dylansechet)

PR：[#22348](https://github.com/bevyengine/bevy/pull/22348), [#22459](https://github.com/bevyengine/bevy/pull/22459), [#22468](https://github.com/bevyengine/bevy/pull/22468), [#22618](https://github.com/bevyengine/bevy/pull/22618), [#22671](https://github.com/bevyengine/bevy/pull/22671), [#23442](https://github.com/bevyengine/bevy/pull/23442), [#23809](https://github.com/bevyengine/bevy/pull/23809), [#23813](https://github.com/bevyengine/bevy/pull/23813), [#23898](https://github.com/bevyengine/bevy/pull/23898), [#23948](https://github.com/bevyengine/bevy/pull/23948), [#23968](https://github.com/bevyengine/bevy/pull/23968)

![solari](https://bevy.org/news/bevy-0-19/solari.jpg)

Solari —— Bevy 的实时路径追踪渲染器 —— 获得了多项改进：镜面和非金属材质的修复、性能提升，以及大幅提高的时域稳定性。

更多详情，请阅读 [JMS55 的博客文章](https://jms55.github.io/posts/2026-04-12-solari-bevy-0-19)。

## 更多 Feathers Widget [#](https://bevy.org/news/bevy-0-19/#more-feathers-widgets)

作者：[viridia](https://github.com/viridia)、[jordanhalase](https://github.com/jordanhalase)

PR：[#23645](https://github.com/bevyengine/bevy/pull/23645), [#23707](https://github.com/bevyengine/bevy/pull/23707), [#23788](https://github.com/bevyengine/bevy/pull/23788), [#23787](https://github.com/bevyengine/bevy/pull/23787), [#23804](https://github.com/bevyengine/bevy/pull/23804), [#23817](https://github.com/bevyengine/bevy/pull/23817), [#23842](https://github.com/bevyengine/bevy/pull/23842), [#23744](https://github.com/bevyengine/bevy/pull/23744), [#23820](https://github.com/bevyengine/bevy/pull/23820), [#23830](https://github.com/bevyengine/bevy/pull/23830), [#23869](https://github.com/bevyengine/bevy/pull/23869), [#23883](https://github.com/bevyengine/bevy/pull/23883), [#23890](https://github.com/bevyengine/bevy/pull/23890), [#23993](https://github.com/bevyengine/bevy/pull/23993), [#24092](https://github.com/bevyengine/bevy/pull/24092)

![feathers widgets](https://bevy.org/news/bevy-0-19/feathers.jpg)

Bevy Feathers —— 我们的自认为编辑器设计的 UI widget 集合 —— 在本周期新增了几个 widget：

- 文本输入（Text input）
- 数字输入（Number input）
- 下拉菜单按钮和菜单分隔线
- 开闭切换（chevron 展开/折叠）
- 图标和标签（显示原语）
- 窗格、子窗格和组（编辑器的装饰性框架）
- 列表视图（List view）
- 滚动条（Scrollbar）

我们还改进了现有 widget！如需完整用法和交互式演示，请尝试 [`feathers_gallery`](https://github.com/bevyengine/bevy/blob/v0.19.0/examples/ui/widgets/feathers_gallery.rs) 示例。

### Feathers + BSN = ❤️ [#](https://bevy.org/news/bevy-0-19/#feathers-bsn-red-heart)

Feathers 的 widget 已经迁移到 BSN —— Bevy 的下一代场景系统。BSN 是比旧的生成函数方法更好的 widget 基础：它减少了样板代码，允许你将 widget 组合在一起，使用 SceneComponent props 参数化 widget，引用字体/图片资源，并在同一个声明中注册观察者。

```rust
// Before: label children passed as a generic argument, observer wired separately
commands.spawn(checkbox_bundle(
    MyCheckbox,
    children![(Text::new("Enable shadows"), ThemedText)],
)).observe(|change: On<ValueChange<bool>>, mut config: ResMut<ShadowConfig>| {
    config.enabled = change.value;
});

// After: caption, extra components, and observer all defined in one call
bsn! {
    @FeathersCheckbox {
        @caption: bsn! { Text("Enable shadows") ThemedText }
    }
    MyCheckbox
    on(|change: On<ValueChange<bool>>, mut config: ResMut<ShadowConfig>| {
        config.enabled = change.value;
    })
}
```

将来，`bsn!` 宏中使用的相同 BSN 语法将可移植到 `.bsn` 文件中，让开发者可以在定义 UI 时选择并快速切换代码优先和资源驱动的工作流。

## 文本输入（Text Input）[#](https://bevy.org/news/bevy-0-19/#text-input)

作者：[ickshonpe](https://github.com/ickshonpe)、[Zeophlite](https://github.com/Zeophlite)、[alice-i-cecile](https://github.com/alice-i-cecile)、[chronicl](https://github.com/chronicl)

PR：[#19106](https://github.com/bevyengine/bevy/pull/19106), [#23282](https://github.com/bevyengine/bevy/pull/23282), [#23455](https://github.com/bevyengine/bevy/pull/23455), [#23475](https://github.com/bevyengine/bevy/pull/23475), [#23479](https://github.com/bevyengine/bevy/pull/23479), [#23496](https://github.com/bevyengine/bevy/pull/23496), [#23679](https://github.com/bevyengine/bevy/pull/23679), [#23704](https://github.com/bevyengine/bevy/pull/23704), [#23841](https://github.com/bevyengine/bevy/pull/23841), [#23947](https://github.com/bevyengine/bevy/pull/23947), [#23960](https://github.com/bevyengine/bevy/pull/23960), [#23969](https://github.com/bevyengine/bevy/pull/23969), [#24023](https://github.com/bevyengine/bevy/pull/24023), [#24028](https://github.com/bevyengine/bevy/pull/24028), [#24032](https://github.com/bevyengine/bevy/pull/24032)

在 **Bevy 0.19** 中，我们以 [`EditableText`](https://docs.rs/bevy/0.19.0-rc.3/bevy/text/struct.EditableText.html) 组件的形式加入了文本输入的基本支持。生成带有此组件的实体将创建一个简单的无样式可编辑文本矩形。我们的初始文本输入支持：

- **打字**：按键盘按键，显示文本（厉害吧！）
- **光标导航**：方向键、Home/End 和词级快捷键（Ctrl/Alt+方向键）
- **选择**：Shift+方向键按字符或词扩展；指针点击并拖动
- **多点点击**：双击选择一个词，三击选择整行
- **退格和删除**：支持单个字符和整个词
- **剪贴板**：启用 `system_clipboard` 功能时使用系统剪贴板，否则使用应用内缓冲区
- **Unicode 感知的导航和编辑**：1 字节/字符 != 1 个字符
- **双向文本**：支持从左到右和从右到左的文字
- **IME（输入法）支持**：支持中文、日文、韩文等组合输入文字
- **多行支持**：换行、软换行和垂直滚动
- **水平滚动**：当内容超出可见宽度时
- **逐字符输入过滤**：通过 `EditableTextFilter` 实现
- **聚焦时可选全选**：通过 `SelectAllOnFocus` 组件实现
- **最大字符限制**：通过 `EditableText::max_characters` 实现

许多重要功能目前尚未实现（占位文本、撤销重做、密码掩码……）。我们已小心地暴露和记录了内部实现，方便你在自己的项目中实现这些功能，但我们希望继续扩展基础 widget 的功能。欢迎提交 PR！

要查看实际用法，请查看我们新的 [`text_input.rs`](https://github.com/bevyengine/bevy/blob/v0.19.0/examples/ui/text/text_input.rs) 示例。

## 接触阴影（Contact Shadows）[#](https://bevy.org/news/bevy-0-19/#contact-shadows)

作者：[aevyrie](https://github.com/aevyrie)

PR：[#22382](https://github.com/bevyengine/bevy/pull/22382)

拖动此图片进行比较

![接触阴影关闭](https://bevy.org/news/bevy-0-19/no_contact_shadows.jpg)![接触阴影开启](https://bevy.org/news/bevy-0-19/contact_shadows.jpg)

Bevy 0.19 引入了**接触阴影**，帮助阴影捕捉物体的细节并正确地附着到附近的表面上。

以前，Bevy 的阴影（Solari 之外）完全使用（级联）[阴影映射](https://en.wikipedia.org/wiki/Shadow_mapping)来渲染。阴影映射是一种可靠的经典技术，其工作原理是从光源视角观察场景中的物体，创建深度图，然后用它来确定哪些物体处于阴影中。不幸的是，这项技术从根本上受限于所创建的阴影贴图纹理的分辨率，并且只有在投射阴影的物体与其投射阴影的表面之间的距离相对较大时才能产生良好的效果。

在近距离下，根据你设置的[深度偏移](https://renderdiagrams.org/2024/12/18/shadowmap-bias/)，你得到的要么是彼得潘现象（物体因阴影脱离而似乎漂浮在地面上方），要么是阴影痤疮（阴影以不真实的方式自相交）。提高阴影贴图的分辨率会改变"近"的含义，但内存成本高得令人望而却步。单靠阴影贴图根本无法获得良好的短距离阴影。你需要一个互补的解决方案。

[接触阴影](https://www.bendstudio.com/blog/inside-bend-screen-space-shadows/)弥补了这一空白。其核心思想是执行一个短距离（因此成本可承受）的屏幕空间[光线投射](https://en.wikipedia.org/wiki/Ray_casting)，从表面向光源追踪一条线，检查附近是否有遮挡物体。

效果令人惊叹：阴影正确地**贴合**表面，以赋予物体和角色生命力的方式强调微妙的曲线。

接触阴影目前支持定向光、点光源和聚光灯。它们按光源单独切换，渲染接触阴影的成本与屏幕上被启用了接触阴影的光源照亮的像素数量乘以该类光源数量成正比。要为光源启用接触阴影，请将灯光组件上的 `contact_shadows_enabled` 字段设置为 `true`，并将 [`ContactShadows`](https://docs.rs/bevy/0.19.0-rc.3/bevy/pbr/struct.ContactShadows.html) 组件添加到摄像机。调整该组件上的值可以控制整个场景中接触阴影的计算方式。

## 基于物理的屏幕空间反射（Physically Based Screen Space Reflections）[#](https://bevy.org/news/bevy-0-19/#physically-based-screen-space-reflections)

作者：[aevyrie](https://github.com/aevyrie)

PR：[#22379](https://github.com/bevyengine/bevy/pull/22379)

Bevy 的屏幕空间反射现在使用"基于物理"的算法，大幅提升了反射质量！

![physical_reflections](https://bevy.org/news/bevy-0-19/physical_reflections.png)

## 矩形区域光（Rectangular Area Lights）[#](https://bevy.org/news/bevy-0-19/#rectangular-area-lights)

作者：[dylansechet](https://github.com/dylansechet)

PR：[#23288](https://github.com/bevyengine/bevy/pull/23288)

![rectangular area lights](https://bevy.org/news/bevy-0-19/rect_area_lights.png)

Bevy 的照明工具集新增了一个成员：矩形区域光！

该实现使用了[线性变换余弦](https://eheitzresearch.wordpress.com/415-2/)，这是实时区域光的标准方法，也应有助于在不久的将来使我们的球形区域光更加精确。

矩形区域光目前不投射阴影，也不支持各向异性材质。

你需要启用 `area_light_luts` cargo feature 才能使用它。

查看[新示例](https://github.com/bevyengine/bevy/blob/v0.19.0/examples/3d/rect_light.rs)看看它们的效果。

## 更丰富的文本（Richer Text）[#](https://bevy.org/news/bevy-0-19/#richer-text)

作者：[ickshonpe](https://github.com/ickshonpe)、[alice-i-cecile](https://github.com/alice-i-cecile)、[gregcsokas](https://github.com/gregcsokas)

PR：[#22156](https://github.com/bevyengine/bevy/pull/22156), [#22396](https://github.com/bevyengine/bevy/pull/22396), [#22614](https://github.com/bevyengine/bevy/pull/22614), [#22879](https://github.com/bevyengine/bevy/pull/22879), [#23380](https://github.com/bevyengine/bevy/pull/23380)

Bevy 的文本系统历来功能稀疏：通过资源句柄选择字体，设置像素大小，完事。想要粗体？加载单独的粗体字体资源。想要斜体？再加载一个资源。想要用户的系统等宽字体？没门。想要随视口缩放的文本？自己实现。

现在都不再是问题了。

### 更好的字体选择（Better font selection）[#](https://bevy.org/news/bevy-0-19/#better-font-selection)

![generic fonts](https://bevy.org/news/bevy-0-19/generic_fonts.jpg)

[`FontSource`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/enum.FontSource.html) 现在提供了三种指定字体的方式：

```rust
// 字体句柄
FontSource::Handle(asset_server.load("fonts/FiraMono.ttf"))

// 字体族名称
FontSource::Family("FiraMono".into())

// 语义类别
FontSource::Monospace
```

通用变体 —— `Serif`、`SansSerif`、`Cursive`、`Fantasy`、`Monospace`，以及几个 UI 专用变体（`SystemUi`、`Emoji`、`Math` 等）—— 解析为可配置的默认值。通过 `FontCx` 覆盖它们：

```rust
fn configure_fonts(mut font_cx: ResMut<FontCx>) {
    font_cx.set_serif_family("Merriweather");
    font_cx.set_monospace_family("JetBrains Mono");
}
```

对于希望尊重用户字体偏好而不硬编码资源路径的编辑器工具和非游戏应用来说，这尤为有用。

在之前的版本中，系统字体已经可以通过后端资源加载，但 `FontSource::Family` 是一种更简洁、更强大的加载方式。启用 `bevy/system_font_discovery` 功能可使已安装的系统字体按名称可用；不启用时，`FontSource::Family("...")` 只会查找显式加载为 Bevy 资源的字体。

### 可变字体属性（Variable font properties）[#](https://bevy.org/news/bevy-0-19/#variable-font-properties)

![variable font properties](https://bevy.org/news/bevy-0-19/variable_font_properties.jpg)

[`TextFont`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.TextFont.html) 新增了 `weight`、`width` 和 `style` 字段。选择一个可变字体，告别为每种字型变体准备单独的资源：

```rust
TextFont {
    font: FontSource::SansSerif,
    weight: FontWeight::BOLD,
    style: FontStyle::Italic,
    width: FontWidth::CONDENSED,
    ..default()
}
```

### 响应式字体大小（Responsive font sizing）[#](https://bevy.org/news/bevy-0-19/#responsive-font-sizing)

`font_size` 现在是一个 [`FontSize`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/enum.FontSize.html) 枚举，而不是单纯的 `f32`：

```rust
TextFont::from_font_size(FontSize::Px(24.0))   // fixed pixels — unchanged behavior
TextFont::from_font_size(FontSize::Vh(5.0))    // 5% of viewport height
TextFont::from_font_size(FontSize::Rem(1.5))   // relative to the RemSize resource
```

完整的变体集合与 CSS 相对应：`Px`、`Vw`、`Vh`、`VMin`、`VMax` 和 `Rem`。`Rem` 值随 `RemSize` 资源缩放，让你可以通过一个旋钮同时调整所有相对文本的大小。

### 字母间距（Letter spacing）[#](https://bevy.org/news/bevy-0-19/#letter-spacing)

一个新的 [`LetterSpacing`](https://docs.rs/bevy/0.19.0-rc.3/bevy/text/enum.LetterSpacing.html) 组件控制字符之间的间距：

```rust
commands.spawn((
    Text::new("SPACED OUT"),
    LetterSpacing::Px(4.0),
));
```

虽然所有这些功能在 [`cosmic_text`](https://github.com/pop-os/cosmic-text) 中也是可能实现的，但我们选择在本周期迁移到 [`parley`](https://github.com/linebender/parley)。两者都是可靠、现代的选择，但我们发现 `parley` 的文档明显更好，使用起来也更顺手一些。

## 应用设置（App Settings）[#](https://bevy.org/news/bevy-0-19/#app-settings)

作者：[viridia](https://github.com/viridia)、[mpowell90](https://github.com/mpowell90)

PR：[#22891](https://github.com/bevyengine/bevy/pull/22891), [#23034](https://github.com/bevyengine/bevy/pull/23034), [#23719](https://github.com/bevyengine/bevy/pull/23719), [#23812](https://github.com/bevyengine/bevy/pull/23812)

Bevy 现在有了一个内置的通用"应用设置"系统，Bevy 应用可以使用它来加载和保存任意设置，例如：

- 图形选项
- 面板布局和工具偏好
- 音乐和音效音量控制
- 窗口位置和大小
- "不再显示此对话框"

值得注意的是，Bevy Editor 需要一个设置系统来管理布局偏好、工具配置以及所有应在会话之间持久化的内容。由于 Bevy Editor 本身是作为 Bevy 应用构建的，它可以使用这个新的设置系统！

设置组是普通的 Rust 结构体，派生 [`Resource`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.Resource.html)、[`SettingsGroup`](https://docs.rs/bevy/0.19.0-rc.3/bevy/settings/trait.SettingsGroup.html) 和 [`Reflect`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.Reflect.html)：

```rust
#[derive(Resource, SettingsGroup, Reflect, Default)]
#[reflect(Resource, SettingsGroup, Default)]
struct AudioSettings {
    music_volume: f32,
    sfx_volume: f32,
}
```

使用唯一的[反向域名](https://en.wikipedia.org/wiki/Reverse_domain_name_notation)应用名称添加 [`SettingsPlugin`](https://docs.rs/bevy/0.19.0-rc.3/bevy/settings/struct.SettingsPlugin.html) 会自动在启动时加载你的设置组并将其作为资源插入：

```rust
app.add_plugins(SettingsPlugin::new("com.example.mygame"));
```

然后你可以像读取任何其他资源一样读取它们：

```rust
fn adjust_volume(audio: Res<AudioSettings>, mut music: ResMut<AudioSink>) {
    music.set_volume(audio.music_volume);
}
```

设置可以通过 [`SaveSettingsDeferred`](https://docs.rs/bevy/0.19.0-rc.3/bevy/settings/struct.SaveSettingsDeferred.html) 或 [`SaveSettingsSync`](https://docs.rs/bevy/0.19.0-rc.3/bevy/settings/enum.SaveSettingsSync.html) 命令保存。

完整示例请查看 [`settings.rs`](https://github.com/bevyengine/bevy/blob/v0.19.0/examples/app/settings.rs)。

---

特别感谢 Andhrimnir（@tecbeast42）将在 `crates.io` 上的 `bevy-settings` crate 名称所有权转让给 Bevy。我们构建了全新的 settings crate，但我们重新使用了 `bevy-settings` 这个名称，因为它最合适。

## 更多后处理效果（More Post-Processing Effects）[#](https://bevy.org/news/bevy-0-19/#more-post-processing-effects)

作者：[Breakdown-Dog](https://github.com/Breakdown-Dog)

PR：[#22564](https://github.com/bevyengine/bevy/pull/22564), [#23110](https://github.com/bevyengine/bevy/pull/23110)

本周期新增了两个后处理效果，都是让你的摄像机呈现出更电影化或风格化外观的经典工具。

### 暗角（Vignette）[#](https://bevy.org/news/bevy-0-19/#vignette)

拖动此图片进行比较

![无暗角](https://bevy.org/news/bevy-0-19/post_processing_base.jpg)![有暗角](https://bevy.org/news/bevy-0-19/post_processing_vignette.jpg)

暗角效果会降低画面边缘的亮度，将观众的注意力吸引到中心。

```rust
commands.spawn((
    Camera3d::default(),
    Vignette {
        intensity: 1.0,
        radius: 0.75,
        smoothness: 5.0,
        roundness: 1.0,
        center: Vec2::new(0.5, 0.5),
        edge_compensation: 1.0,
        color: Color::BLACK,
    },
));
```

### 镜头畸变（Lens Distortion）[#](https://bevy.org/news/bevy-0-19/#lens-distortion)

拖动此图片进行比较

![无畸变](https://bevy.org/news/bevy-0-19/post_processing_base.jpg)![桶形畸变](https://bevy.org/news/bevy-0-19/post_processing_barrel_distortion.jpg)

拖动此图片进行比较

![无畸变](https://bevy.org/news/bevy-0-19/post_processing_base.jpg)![枕形畸变](https://bevy.org/news/bevy-0-19/post_processing_pincushion_distortion.jpg)

镜头畸变在空间上扭曲图像。正的 `intensity` 将边缘向外推（桶形畸变），负的则向内拉（枕形畸变）。

```rust
commands.spawn((
    Camera3d::default(),
    LensDistortion {
        intensity: 0.5,
        scale: 1.0,
        multiplier: Vec2::ONE,
        center: Vec2::splat(0.5),
        edge_curvature: 0.0,
    },
));
```

## 渲染恢复（Render Recovery）[#](https://bevy.org/news/bevy-0-19/#render-recovery)

作者：[atlv24](https://github.com/atlv24)、[kfc35](https://github.com/kfc35)

PR：[#22761](https://github.com/bevyengine/bevy/pull/22761), [#23350](https://github.com/bevyengine/bevy/pull/23350), [#23349](https://github.com/bevyengine/bevy/pull/23349), [#23433](https://github.com/bevyengine/bevy/pull/23433), [#23458](https://github.com/bevyengine/bevy/pull/23458), [#23444](https://github.com/bevyengine/bevy/pull/23444), [#23459](https://github.com/bevyengine/bevy/pull/23459), [#23461](https://github.com/bevyengine/bevy/pull/23461), [#23463](https://github.com/bevyengine/bevy/pull/23463), [#22714](https://github.com/bevyengine/bevy/pull/22714), [#22759](https://github.com/bevyengine/bevy/pull/22759), [#16481](https://github.com/bevyengine/bevy/pull/16481), [#24131](https://github.com/bevyengine/bevy/pull/24131)

GPU 错误以前没有恢复路径 —— 驱动崩溃、内存不足或设备丢失会导致应用静默挂起或崩溃。这在长时间运行的应用（如艺术装置）或频繁出错的设备（如 VR 头显）上尤其令人沮丧。Bevy 现在将这些错误作为类型化错误呈现，让你决定如何处理每个错误：

```rust
use bevy::render::error_handler::{ErrorType, RenderErrorHandler, RenderErrorPolicy};

app.insert_resource(RenderErrorHandler(
    |error, main_world, render_world| match error.ty {
        ErrorType::DeviceLost => RenderErrorPolicy::Recover(default()),
        ErrorType::OutOfMemory => RenderErrorPolicy::StopRendering,
        ErrorType::Validation => RenderErrorPolicy::Ignore,
        ErrorType::Internal => panic!(),
    },
));
```

`DeviceLost` 是大多数游戏会想要处理的情况：它涵盖 GPU 驱动崩溃、热关机和硬件物理断开连接。`RenderErrorPolicy::Recover` 会重新初始化渲染器并保持应用运行。`StopRendering` 停止渲染但保持应用的其余部分存活 —— 如果你想在退出前显示错误屏幕或保存状态，这很有用。`Ignore` 静默忽略错误，这是验证错误的现有行为。`Internal` 错误表示 bug，继续 panic 是合适的。

请务必在游戏中仔细测试错误恢复；我们观察到在重复失败期间（例如由内存不足问题引起的）出现硬件相关的闪烁情况，这对光敏性癫痫患者构成严重访问风险。虽然我们希望在后续版本中彻底解决这个问题，但目前我们选择了保守的默认策略。如果你没有配置 [`RenderErrorHandler`](https://docs.rs/bevy/0.19.0-rc.3/bevy/render/error_handler/struct.RenderErrorHandler.html)，行为与之前相似但不完全相同：Vulkan 验证错误被忽略，其他所有错误发送 `AppExit` 事件以优雅关闭。

## 渲染图作为 System（Render Graph as Systems）[#](https://bevy.org/news/bevy-0-19/#render-graph-as-systems)

作者：[tychedelia](https://github.com/tychedelia)

PR：[#22144](https://github.com/bevyengine/bevy/pull/22144)

Bevy 的 `RenderGraph` 架构已被 ECS Schedule 取代。渲染通道现在是普通的 System，运行在诸如 [`Core3d`](https://docs.rs/bevy/0.19.0-rc.3/bevy/core_pipeline/struct.Core3d.html)、[`Core2d`](https://docs.rs/bevy/0.19.0-rc.3/bevy/core_pipeline/struct.Core2d.html) 等 Schedule 中，在渲染世界上执行。

旧的渲染图最初是在 Bevy 的 ECS 还不够成熟时设计的。为了添加自定义渲染功能，我们要求用户实现 trait `Node`、派生 `RenderLabel`，并使用专门的 API 来对渲染工作进行排序。这需要大量样板代码！

点击这里看看以前的样子！

```rust
pub struct MyCustomRenderNode;

impl Node for MyCustomRenderNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let res_a = world.resource::<Res<A>>();
        let encoder = render_context.command_encoder();

        // do some rendering things

        Ok(())
    }
}

#[derive(RenderLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub struct MyCustomRenderNodeLabel;

pub struct MyRenderPlugin;

impl Plugin for MyRenderPlugin {
    fn build(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_render_graph_node::<ViewNodeRunner<MyCustomNode>>(
                Core3d,
                MyCustomRenderNodeLabel
            )
            .add_render_graph_edge(
                Core3d,
                Node3d::MainPass,
                MyCustomRenderNodeLabel
            );
    }
}
```

随着 Bevy ECS 的发展，[`Schedule`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/struct.Schedule.html) 已经能够表达"渲染图"模式。直接使用 ECS 让渲染更好地利用熟悉的 Bevy 模式，使上述代码可以表达得更加简洁：

```rust
fn my_custom_render_system(mut ctx: RenderContext, res_a: Res<A>) {
    let encoder = ctx.command_encoder();
    // do some rendering things 
}

pub struct MyRenderPlugin;

impl Plugin for MyRenderPlugin {
    fn build(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(Core3d, my_custom_render_system.after(Core3dSystems::MainPass));
    }
}
```

将来，将渲染工作表达为 System 将使我们能够利用 ECS 探索性能优化。例如，未来支持只读 Schedule 的工作可以通过强制 Schedule 不修改 World 来帮助并行化命令编码。我们很兴奋能继续改进 Bevy 自定义渲染的体验！

## 改进的蒙皮网格剔除（Improved Skinned Mesh Culling）[#](https://bevy.org/news/bevy-0-19/#improved-skinned-mesh-culling)

作者：[greeble-dev](https://github.com/greeble-dev)

PR：[#21837](https://github.com/bevyengine/bevy/pull/21837)

在早期 Bevy 版本中，动画角色和生物有时会在动画中途消失。这是因为 Bevy 使用骨架的静止位置来决定哪些网格在屏幕上，而不是它们实际的动画姿势。一个角色举起手臂时，那些手臂可能实际上超出了 Bevy 用于剔除的包围盒。

蒙皮网格现在每帧从实际关节位置计算其包围体，修复了像 [#4971](https://github.com/bevyengine/bevy/issues/4971) 中报告的网格消失问题。如果你从 glTF 加载蒙皮网格，这是自动完成的 —— 不需要任何更改。

对于手工制作的蒙皮网格，请调用 `Mesh::generate_skinned_mesh_bounds` 并向实体添加 `DynamicSkinnedMeshBounds`：

```rust
let mut mesh: Mesh = ...;
mesh.generate_skinned_mesh_bounds()?;

entity.insert((
    Mesh3d(meshes.add(mesh)),
    DynamicSkinnedMeshBounds,
));
```

## 视差校正立方体贴图（Parallax Corrected Cubemaps）[#](https://bevy.org/news/bevy-0-19/#parallax-corrected-cubemaps)

作者：[pcwalton](https://github.com/pcwalton)

PR：[#22582](https://github.com/bevyengine/bevy/pull/22582)

拖动此图片进行比较

![校正关闭](https://bevy.org/news/bevy-0-19/parallax_correction_off.jpg)![校正开启](https://bevy.org/news/bevy-0-19/parallax_correction_on.jpg)

Bevy 以前渲染立方体贴图反射时就像环境无限远一样。对于室外场景这通常没问题，但对于室内场景和密集环境，结果看起来不真实 —— 反射与观察者周围的几何体不匹配。

标准的修复方法是视差校正：每个反射探针获得自己的包围盒，对该包围盒进行光线追踪确定立方体贴图的正确采样方向。Bevy 现在自动对光照探针应用此技术，使用探针的影响包围盒作为校正体积。这对于捕捉矩形房间内部的立方体贴图是一个合理的默认值，与 Blender 的方法一致。

视差校正默认启用。要在特定探针上禁用它，添加 `ParallaxCorrection::None`：

```rust
commands.spawn((
    LightProbe,
    EnvironmentMapLight { .. },
    ParallaxCorrection::None,
));
```

一个新的 `pccm` 示例演示了该效果，视差校正可在运行时切换。

## 部分无绑定/减少绑定组开销（Partial Bindless / Reduced Bind Group Overhead）[#](https://bevy.org/news/bevy-0-19/#partial-bindless-reduced-bind-group-overhead)

作者：[holg](https://github.com/holg)

PR：[#23436](https://github.com/bevyengine/bevy/pull/23436)

无绑定渲染是现代引擎高效处理具有许多不同材质的场景的方式：着色器索引到共享的纹理和缓冲池，而不是在每个绘制调用中重新绑定它们。

WGPU 对 Metal（Apple 的 GPU API）的后端具有部分无绑定支持。它目前只允许纹理绑定数组，不允许缓冲绑定数组。

历史上，Bevy 要求同时支持这两个功能才会使用无绑定渲染，这完全排除了 Metal，即使对于从未使用缓冲数组的材质也是如此。

包括 [`StandardMaterial`](https://docs.rs/bevy/0.19.0-rc.3/bevy/pbr/struct.StandardMaterial.html) 在内的大多数材质都不需要缓冲数组支持。为了确保这些材质走快速路径，Bevy 现在检查每种材质的实际需求。如果你只需要纹理数组，你的材质可以在 Bevy 的桌面平台上高效渲染。如果你使用 `#[uniform(..., binding_array(...))]`，则预期在 Metal 上性能会下降。

我们还在此过程中修复了两个重要的正确性 bug：

1. 我们发现采样器限制检查测试了错误的指标：`max_samplers_per_shader_stage` 计数的是绑定槽位，但相关的限制是 `max_binding_array_sampler_elements_per_shader_stage`，即数组元素计数（这种不匹配可能错误地禁用无绑定渲染）。
2. Bevy 现在还会跳过为材质不使用的资源类型创建绑定数组槽位，从而保持在 Metal 严格的 31 参数缓冲槽位限制内，并减少所有平台的开销。

在 Bistro Exterior（698 种材质）上进行的基准测试显示，在许多硬件配置上帧时间显著改善（有时内存也有所改善）：

|GPU|Frame Time Speedup|Memory|
|---|---|---|
|Apple M2 Max (Metal)|+15%|−57 MB RAM|
|NVIDIA 5060 Ti|+46%|Same|
|AMD Vega 8 / Ryzen 4800U|Same|−88 MB VRAM|
|Intel i360P|+14%|Same|
|Intel Iris XE|Same|Same|

[Bistro](https://developer.nvidia.com/orca/amazon-lumberyard-bistro) 是一个要求苛刻、相当逼真的场景。虽然无绑定限制仍然令人沮丧（尤其是在 Mac 上无法选择 Vulkan），但看到这些性能提升、知道 Bevy 本身不再人为地拖累用户，这让人很高兴。

## 诊断覆盖层（Diagnostics Overlay）[#](https://bevy.org/news/bevy-0-19/#diagnostics-overlay)

作者：[hukasu](https://github.com/hukasu)、[cart](https://github.com/cart)

PR：[#22486](https://github.com/bevyengine/bevy/pull/22486)

![overlay](https://bevy.org/news/bevy-0-19/overlay.jpg)

Bevy 的诊断信息一直很容易输出到终端，但在游戏中显示它们意味着要自己搭建 UI。[`DiagnosticsOverlayPlugin`](https://docs.rs/bevy/0.19.0-rc.3/bevy/dev_tools/diagnostics_overlay/struct.DiagnosticsOverlayPlugin.html) 为此添加了一个内置覆盖层，为常见情况提供了预设：

```rust
commands.spawn(DiagnosticsOverlay::fps());
commands.spawn(DiagnosticsOverlay::mesh_and_standard_material());
```

你也可以从任意 [`DiagnosticPath`](https://docs.rs/bevy/0.19.0-rc.3/bevy/diagnostic/struct.DiagnosticPath.html) 列表构建自定义覆盖层：

```rust
commands.spawn(DiagnosticsOverlay::new("Diagnostics", vec![
    MyDiagnostics::COUNTER.into()
]));
```

默认情况下，覆盖层显示平滑移动平均值。你可以通过 [`DiagnosticsOverlayStatistic`](https://docs.rs/bevy/0.19.0-rc.3/bevy/dev_tools/diagnostics_overlay/enum.DiagnosticsOverlayStatistic.html) 切换到最新值或原始移动平均值，并通过 [`DiagnosticsOverlayItem::precision`](https://docs.rs/bevy/0.19.0-rc.3/bevy/dev_tools/diagnostics_overlay/struct.DiagnosticsOverlayItem.html#structfield.precision) 配置浮点数精度：

```rust
commands.spawn(DiagnosticsOverlay::new("Diagnostics", vec![DiagnosticsOverlayItem {
    path: MyDiagnostics::COUNTER,
    statistic: DiagnosticsOverlayStatistic::Value,
    precision: 4,
}]));
```

## 连续 Query 访问（Contiguous Query Access）[#](https://bevy.org/news/bevy-0-19/#contiguous-query-access)

作者：[Jenya705](https://github.com/Jenya705)

PR：[#21984](https://github.com/bevyengine/bevy/pull/21984), [#24181](https://github.com/bevyengine/bevy/pull/24181)

[SIMD](https://en.wikipedia.org/wiki/Single_instruction,_multiple_data) 是性能优化的关键工具，但在 Bevy 中使用它一直比应有的更难。Bevy 中的表组件已经在内存中平坦排列 —— 所有 [`Transform`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.Transform.html) 组件作为值存储在连续表中，这正是 SIMD 所需要的。只是 [`Query`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/struct.Query.html) 迭代器没有暴露这种结构：它一次给你一个实体的组件，编译器无法知道底层数据是一个连续数组。

[`contiguous_iter`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.QueryState.html#method.contiguous_iter) 和 [`contiguous_iter_mut`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.QueryState.html#method.contiguous_iter_mut) 一次将整个表切片交给你。LLVM 可以看到连续数组并自动向量化 —— 或者你也可以自己使用显式 SIMD。

On a bulk `position += velocity` update over 10,000 entities, this gives some serious speedups:

|Method|Time|Time (AVX2)|
|---|---|---|
|Normal iteration|5.58 µs|5.51 µs|
|Contiguous iteration|4.88 µs|1.87 µs|
|Contiguous, no change detection|4.40 µs|1.58 µs|

如果你的项目有 CPU 密集型工作负载（物理引擎是一个典型例子），你应该立即尝试这个功能。

```rust
fn apply_health_decay(mut query: Query<(&mut Health, &HealthDecay)>) {
    for (mut health, decay) in query.contiguous_iter_mut().unwrap() {
        for (h, d) in health.iter_mut().zip(decay) {
            h.0 *= d.0;
        }
    }
}
```

[`contiguous_iter`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.QueryState.html#method.contiguous_iter) 方法族只在 query 是密集的情况下返回 `Ok`。这意味着：

- 所有获取的组件必须使用默认的"table"存储策略。
- query 过滤器不能干扰返回的 query 数据。像 `With<T>` 和 `Without<T>` 这样的"原型过滤器"没问题；`Changed<T>` 和 `Added<T>` 则不行，因为它们需要逐实体检查，这使得无法返回原始表切片。

由于这些条件是 query 类型的固定属性，除非你编写泛型代码或处理动态组件，否则在这里 `unwrap` 是安全的。

你可能已经注意到上面的表有**三**行。虽然变更检测是一个普遍有用的功能，但它确实带来了可测量的性能开销。默认情况下，[`contiguous_iter_mut`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.QueryState.html#method.contiguous_iter_mut) 返回 `ContiguousMut<T>`。就像普通的 `Mut<T>`，它在解引用时自动触发变更检测。如果你不关心这个，`bypass_change_detection()` 直接给你原始的 `&mut [T]`，实现更快的访问。嗖！

## 延迟命令（Delayed Commands）[#](https://bevy.org/news/bevy-0-19/#delayed-commands)

作者：[Runi-c](https://github.com/Runi-c)

PR：[#23090](https://github.com/bevyengine/bevy/pull/23090)

安排在未来的某个时间点执行某些操作，是游戏开发中从游戏逻辑到音频提示再到 VFX 的常见且有用的工具。

虽然以前可以通过精心使用计时器来实现这一点，但要把细节做对出奇地棘手，而天真的解决方案样板代码又很重。

现在，你可以简单地延迟任意命令稍后执行。

```rust
fn delayed_spawn(mut commands: Commands) {
    commands.delayed().secs(1.0).spawn(DummyComponent);
}

fn delayed_spawn_then_insert(mut commands: Commands) {
    let mut delayed = commands.delayed();
    let entity = delayed.secs(0.5).spawn_empty().id();
    delayed.secs(1.5).entity(entity).insert(DummyComponent);
}
```

请注意，这还没有内置的官方取消机制。我们建议在命令中嵌入发起方的 [`Entity`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/entity/struct.Entity.html)，以便在该实体死亡或被销毁时取消操作。

## 文本 Gizmo（Text Gizmos）[#](https://bevy.org/news/bevy-0-19/#text-gizmos)

作者：[ickshonpe](https://github.com/ickshonpe)、[nuts-rice](https://github.com/nuts-rice)

PR：[#22732](https://github.com/bevyengine/bevy/pull/22732), [#23120](https://github.com/bevyengine/bevy/pull/23120)

![text gizmos](https://bevy.org/news/bevy-0-19/text_gizmos.jpg)

有时候你只是想调试时在某个东西上贴个标签。文本 Gizmo 正是为此而生：一种零设置的方式，使用内置描边字体在场景中的任何位置绘制世界空间文本。

与 Bevy 的 [`Text2D`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.Text2d.html)（适用于伤害数字、铭牌和游戏内标签）不同，文本 Gizmo **严格**用于开发工具和调试。字体是固定的，只支持 ASCII。

使用 [`Gizmos::text`] 和 `text_2d` 快速绘制文本：

```rust
fn draw_text(mut gizmos: Gizmos) {
    gizmos.text_2d(
        Isometry2d::IDENTITY, // Position and rotation of the text in world-space
        "Hello Bevy",         // Only supports ASCII text
        40.0,                 // Font size in screen-space pixels
        Vec2::ZERO,           // Anchor point, zero is centered
        Color::WHITE,         // Color of the text
    );
}
```

如果你想分别为每个字符段着色，可以使用 `text_sections` 和 `text_sections_2d`。

## 可取消的 Web 任务（Cancellable Web Tasks）[#](https://bevy.org/news/bevy-0-19/#cancellable-web-tasks)

作者：[NthTensor](https://github.com/NthTensor)、[Gingeh](https://github.com/Gingeh)

PR：[#21795](https://github.com/bevyengine/bevy/pull/21795)

当 Bevy 中的 [`Task`](https://docs.rs/bevy/0.19.0-rc.3/bevy/tasks/struct.Task.html) 被丢弃时，它应该被取消，在下一个 yield 点停止底层工作。在 Web 上，这从未生效。`wasm_bindgen_futures::spawn_local` 将你的 future 直接交给 JS 事件循环，没有拿回句柄，所以 Bevy 的任务包装器只是一个没有取消能力的收据。

结果，在原生桌面和移动平台上正确管理任务生命周期的代码，在 Web 上会静默泄漏工作。

```rust
fn update_background_task(mut task_handle: ResMut<CurrentTask>) {
    // Replaces the old task. On native: the old task is dropped and cancelled.
    // On web: the old task kept running. Oops.
    task_handle.0 = AsyncComputeTaskPool::get().spawn(async { do_work().await });
}
```

修复这个问题需要一种新的 WASM 执行器方法。[`web-task`](https://crates.io/crates/web-task) crate 由我们的 `@NthTensor` 构建，在 JS 事件循环之上实现了协作式取消：生成的任务在每个 yield 点检查中止标志。Bevy 现在在 WASM 上使用它，因此 [`Task`](https://docs.rs/bevy/0.19.0-rc.3/bevy/tasks/struct.Task.html) 的 drop 语义终于在所有平台上一致了。

## 资源保存（Asset Saving）[#](https://bevy.org/news/bevy-0-19/#asset-saving)

作者：[andriyDev](https://github.com/andriyDev)

PR：[#22622](https://github.com/bevyengine/bevy/pull/22622)

Bevy 自 0.12 版本以来就有了 [`AssetSaver`](https://docs.rs/bevy/0.19.0-rc.3/bevy/asset/saver/trait.AssetSaver.html) trait。然而，它只打算在资源处理管道内部使用，而不是在运行时保存资源。这留下了一个令人沮丧的缺口：如果你想保存程序生成的网格、烘焙的光照贴图或编辑器内工作流的输出，没有支持的途径。

现在有了。`save_using_saver` 允许你使用选择的 `AssetSaver` 实现将任何资源保存到磁盘。

### 1. 构建 `SavedAsset` [#](https://bevy.org/news/bevy-0-19/#1-building-the-savedasset)

对于没有子资源的简单资源，使用 `SavedAsset::from_asset`：

```rust
let main_asset = InlinedBook {
    lines: vec!["Save me!".to_string(), "Please!".to_string()],
};
let saved_asset = SavedAsset::from_asset(&main_asset);
```

对于引用其他资源（子资源）的资源，使用 `SavedAssetBuilder`：

```rust
let asset_path: AssetPath<'static> = "my/file/path.whatever".into();
let mut builder = SavedAssetBuilder::new(asset_server.clone(), asset_path.clone());

let subasset_1 = Line("howdy".into());
let subasset_2 = Line("goodbye".into());
let handle_1 = builder.add_labeled_asset_with_new_handle(
    "TheFirstLabel", SavedAsset::from_asset(&subasset_1));
let handle_2 = builder.add_labeled_asset_with_new_handle(
    "AnotherOne", SavedAsset::from_asset(&subasset_2));

let main_asset = Book {
    lines: vec![handle_1, handle_2],
};
let saved_asset = builder.build(&main_asset);
```

`SavedAsset` 借用（borrow）而非拥有（own）其资源。这意味着你可以在同一个异步块中构建和保存 —— 无需先转移所有权。

### 2. 调用 `save_using_saver` [#](https://bevy.org/news/bevy-0-19/#2-calling-save-using-saver)

```rust
save_using_saver(
    asset_server.clone(),
    &MyAssetSaver::default(),
    &asset_path,
    saved_asset,
    &MySettings::default(),
).await.unwrap();
```

`save_using_saver` 是异步的。通常，你会想用 `IoTaskPool::get().spawn(...)` 来 spawn 它。你还需要为 `MyAssetSaver` 实现 `AssetSaver` 来定义序列化格式。

## 资源作为组件（Resources as Components）[#](https://bevy.org/news/bevy-0-19/#resources-as-components)

作者：[Trashtalk217](https://github.com/Trashtalk217)、[cart](https://github.com/cart)、[SpecificProtagonist](https://github.com/SpecificProtagonist)

PR：[#20934](https://github.com/bevyengine/bevy/pull/20934), [#22910](https://github.com/bevyengine/bevy/pull/22910), [#22911](https://github.com/bevyengine/bevy/pull/22911), [#22919](https://github.com/bevyengine/bevy/pull/22919), [#22930](https://github.com/bevyengine/bevy/pull/22930), [#23616](https://github.com/bevyengine/bevy/pull/23616), [#23716](https://github.com/bevyengine/bevy/pull/23716), [#24077](https://github.com/bevyengine/bevy/pull/24077), [#24164](https://github.com/bevyengine/bevy/pull/24164)

资源和组件在 Bevy 的 ECS 中一直是独立的概念。虽然简单的 `Res<Time>` 语法糖很好，但唯一的真正区别是基数（cardinality）—— 资源是一种最多同时存在一个的组件。

这种分离一直是摩擦的来源。我们许多针对组件的工具（如 hooks、observer 和 relation）对资源根本不可用，并且引擎携带了大量重复的内部机制来保持两个机制同步。

在 **Bevy 0.19** 中，资源现在作为组件存储在单例实体上，统一了我们的内部机制并为资源赋予了更多能力。现在你可以：

- 通过假设实体+组件是你需要关注的唯一数据形式，简化网络和开发工具代码
- 在资源 and 组件上进行 Query，支持灵活的使用模式
- 添加指向和来自资源实体的关系
- 向资源实体添加额外的组件
- 向资源类型添加生命周期观察者
- 向资源添加自己的 hook
- 将资源标记为不可变

## 远程实体预留（Remote Entity Reservation）[#](https://bevy.org/news/bevy-0-19/#remote-entity-reservation)

作者：[ElliottjPierce](https://github.com/ElliottjPierce)、[alice-i-cecile](https://github.com/alice-i-cecile)、[cart](https://github.com/cart)

PR：[#18670](https://github.com/bevyengine/bevy/pull/18670), [#22658](https://github.com/bevyengine/bevy/pull/22658)

Bevy 历来需要 [`World`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/struct.World.html) 引用来分配实体 ID。这在大多数场景下可行，但意味着如果你想并行执行初始化实体的工作，你需要阻塞应用的执行！这对我们即将进行的"资产即实体"等工作来说是个问题，它需要在应用继续运行的同时在后台准备实体内容。

**Bevy 0.19** 引入了一种新的实体分配策略，使得无需 [`World`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/struct.World.html) 引用即可从任何线程预留实体 ID，**同时**不牺牲性能。这涉及将[实体生命周期](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/entity/index.html#entity-life-cycle)分为五个阶段：未分配、已分配、已生成、已销毁和已释放。

## 交互式 Transform Gizmo（Interactive Transform Gizmo）[#](https://bevy.org/news/bevy-0-19/#interactive-transform-gizmo)

作者：[jbuehler23](https://github.com/jbuehler23)、[aevyrie](https://github.com/aevyrie)

PR：[#23435](https://github.com/bevyengine/bevy/pull/23435)

Transform Gizmo —— 用于在 3D 视口中平移、旋转和缩放对象的点击拖拽手柄 —— 是任何人构建关卡编辑器时最先想到的功能之一。Bevy 现在内置了一个，供你现在使用和我们未来使用。

添加 [`TransformGizmoPlugin`](https://docs.rs/bevy/0.19.0-rc.3/bevy/gizmos/prelude/struct.TransformGizmoPlugin.html)，用 [`TransformGizmoCamera`](https://docs.rs/bevy/0.19.0-rc.3/bevy/gizmos/prelude/struct.TransformGizmoCamera.html) 标记摄像机，用 [`TransformGizmoFocus`](https://docs.rs/bevy/0.19.0-rc.3/bevy/gizmos/prelude/struct.TransformGizmoFocus.html) 标记实体：

```rust
app.add_plugins(TransformGizmoPlugin);

commands.spawn((Camera3d::default(), TransformGizmoCamera));
commands.spawn((Mesh3d(mesh), TransformGizmoFocus));
```

该插件故意不与用户输入连接。这使 Gizmo 对已有输入处理偏好的编辑器作者保持可组合性。灵敏度、吸附和屏幕空间缩放都可以通过 [`TransformGizmoSettings`](https://docs.rs/bevy/0.19.0-rc.3/bevy/gizmos/prelude/struct.TransformGizmoSettings.html) 配置，而模式通过 [`TransformGizmoMode`](https://docs.rs/bevy/0.19.0-rc.3/bevy/gizmos/prelude/enum.TransformGizmoMode.html) 资源控制。

这个 widget 的许多数学和实现策略来自 [`bevy_transform_gizmo`](https://github.com/fslabs/bevy_transform_gizmo) crate。再次感谢 Foresight Spatial Labs 慷慨的开源贡献！

## 无限网格（Infinite Grid）[#](https://bevy.org/news/bevy-0-19/#infinite-grid)

作者：[IceSentry](https://github.com/IceSentry)

PR：[#23482](https://github.com/bevyengine/bevy/pull/23482)

![infinite grid](https://bevy.org/news/bevy-0-19/infinite_grid.jpg)

透明的地面网格是 3D 编辑器工具的标配：它标记主轴、为场景定向，并使比例立即清晰可辨。

简单地画线效果不好：网格必须有尽头，而延伸到地平线的线条无论延伸多远都会产生锯齿伪影和摩尔纹。

我们的实现将网格渲染为全屏着色器：从摄像机视角在屏幕空间中逐像素计算网格，并随距离淡出以消除地平线上的锯齿。

To add an infinite grid to your app, register [`InfiniteGridPlugin`](https://docs.rs/bevy/0.19.0-rc.3/bevy/dev_tools/infinite_grid/struct.InfiniteGridPlugin.html) and spawn the [`InfiniteGrid`](https://docs.rs/bevy/0.19.0-rc.3/bevy/dev_tools/infinite_grid/struct.InfiniteGrid.html) component:

```rust
use bevy::dev_tools::infinite_grid::{InfiniteGrid, InfiniteGridPlugin};
use bevy::prelude::*;

App::new()
    .add_plugins((DefaultPlugins, InfiniteGridPlugin))
    .add_systems(Startup, setup)
    .run();

fn setup(mut commands: Commands) {
    commands.spawn(InfiniteGrid);
}
```

网格外观 —— 颜色、淡出距离、线条缩放 —— 由 [`InfiniteGridSettings`](https://docs.rs/bevy/0.19.0-rc.3/bevy/dev_tools/infinite_grid/struct.InfiniteGridSettings.html) 控制，可以放置在网格实体或特定摄像机上以按视图覆盖。你可以在新的 [`infinite_grid.rs`](https://github.com/bevyengine/bevy/blob/v0.19.0/examples/dev_tools/infinite_grid.rs) 示例中查看其工作方式。

这是 [`bevy_infinite_grid` crate](https://github.com/fslabs/bevy_infinite_grid) 的上游化版本，由 Foresight Spatial Labs 创建和维护 —— 感谢他们构建并慷慨地将其贡献给 Bevy！

## 白炉测试（White Furnace Test）[#](https://bevy.org/news/bevy-0-19/#white-furnace-test)

作者：[dylansechet](https://github.com/dylansechet)

PR：[#23194](https://github.com/bevyengine/bevy/pull/23194), [#23203](https://github.com/bevyengine/bevy/pull/23203)

拖动此图片进行比较

![之前](https://bevy.org/news/bevy-0-19/white_furnace_before.jpg)![之后](https://bevy.org/news/bevy-0-19/white_furnace_after.jpg)

[白炉测试](https://lousodrome.net/blog/light/2023/10/21/the-white-furnace-test/) 是基于物理渲染器的经典完整性检查。将一个完全反射的物体放入均匀的白色环境中，无论金属度和粗糙度如何，它都应该与背景无法区分。任何仍然可见的物体都表明着色器在制造或吸收不该有的能量。

Bevy 以前无法通过这个测试，意味着我们的着色器计算有问题。两个 bug 导致了这个失败：

- 当使用 [`GeneratedEnvironmentMapLight`](https://docs.rs/bevy/0.19.0-rc.3/bevy/light/struct.GeneratedEnvironmentMapLight.html) 时，某些表面方向出现接缝。
- 部分金属材质吸收了能量，看起来比应有的颜色更深。

修复这些问题后，Bevy 通过了测试。这意味着你的材质在基于图像的照明下行为将更加正确。

一张灰色的图像从未如此令人兴奋！

## Observer 运行条件（Observer Run Conditions）[#](https://bevy.org/news/bevy-0-19/#observer-run-conditions)

作者：[jonas-meyer](https://github.com/jonas-meyer)

PR：[#22602](https://github.com/bevyengine/bevy/pull/22602)

运行条件是一种方便、可复用的模式，用于在满足某些条件时跳过 System。以前，运行条件只适用于普通 System。Observer 不能使用它们。

现在，它们可以了！

```rust
#[derive(Resource)]
struct GamePaused(bool);

// Observer only runs when game is not paused
app.add_observer(
    on_damage.run_if(|paused: Res<GamePaused>| !paused.0)
);

// Multiple conditions can be chained
app.add_observer(
    on_damage
        .run_if(|paused: Res<GamePaused>| !paused.0)
        .run_if(resource_exists::<Player>)
);
```

这适用于 `add_observer`、实体 `.observe()` 以及 `Observer` 构建器模式。

## 序列化和反序列化资源句柄（Serializing and Deserializing Asset Handles）[#](https://bevy.org/news/bevy-0-19/#serializing-and-deserializing-asset-handles)

作者：[andriyDev](https://github.com/andriyDev)

PR：[#23329](https://github.com/bevyengine/bevy/pull/23329)

资源句柄现在可以在序列化和反序列化期间成功地进行往返转换。这对世界资源尤其重要 —— 通过 [`DynamicWorld::serialize`] 写入的序列化格式，以前称为场景。

这不仅仅是添加一些派生宏的问题，因为句柄不是原始数据：它们是指向实际已加载资源的指针。因此，没有明确的方法来持久化或重建一个句柄。新的 `HandleSerializeProcessor` 和 `HandleDeserializeProcessor` 通过在序列化时存储句柄的标识信息（其资源路径），然后在反序列化时从该路径重新加载资源来解决这个问题。如果你在自己的序列化管道中需要相同的行为，将它们传递给 `TypedReflectSerializer::with_processor` 和 `TypedReflectDeserializer::with_processor`。

### 注意事项（Caveat）[#](https://bevy.org/news/bevy-0-19/#caveat)

要使此功能生效，你的资源需要正确实现反射。如果你的资源看起来像：

```rust
#[derive(Asset, TypePath)]
struct MyAsset {
    ...
}
```

Change it to:

```rust
#[derive(Asset, Reflect)]
#[reflect(Asset)]
struct MyAsset {
    ...
}
```

## 自引用关系（Self-Referential Relationships）[#](https://bevy.org/news/bevy-0-19/#self-referential-relationships)

作者：[mrchantey](https://github.com/mrchantey)

PR：[#22269](https://github.com/bevyengine/bevy/pull/22269)

默认情况下，Bevy 会拒绝指向自身所在实体的关系组件。如果你插入一个，Bevy 会记录警告并将其移除。这个默认设置是有充分理由的：像 [`ChildOf`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/hierarchy/struct.ChildOf.html) 这样的结构关系形成 Bevy 递归遍历的层级结构 —— 自引用的 [`ChildOf`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/hierarchy/struct.ChildOf.html) 会产生无限循环。

但许多关系是纯语义的。`Likes(self)`、`EmployedBy(self)`、`Healing(self)` —— 这些不暗示任何遍历，自引用是完全有效的。你现在可以通过 `allow_self_referential` 选择启用：

```rust
#[derive(Component)]
#[relationship(relationship_target = PeopleILike, allow_self_referential)]
pub struct LikedBy(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = LikedBy)]
pub struct PeopleILike(Vec<Entity>);
```

设置了该属性后，插入自引用关系将被接受而不会发出警告。

## 无障碍标签组件（Accessible Label Component）[#](https://bevy.org/news/bevy-0-19/#accessible-label-component)

作者：[viridia](https://github.com/viridia)

PR：[#24308](https://github.com/bevyengine/bevy/pull/24308)

[`AccessibleLabel`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.AccessibleLabel.html) 组件允许无障碍 `label` 属性与其他无障碍属性分开指定。

在大多数应用中，`label` 属性来自应用代码而非库代码。然而，`accesskit` 的设计要求所有无障碍属性存储在一个包含在 [`AccessibilityNode`](https://docs.rs/bevy/0.19.0-rc.3/bevy/a11y/struct.AccessibilityNode.html) 组件中的大型数据结构中。这与 BSN 和其他生成复杂层级结构的方法产生了可用性冲突，在这些方法中，组合多个组件是行为复用的主要手段。

通过将标签放在自己的组件中，它可以在 BSN 模板中作为 mixin 使用，允许标签由 widget 用户而不是 widget 作者添加。

在内部，这使用组件 hook 将 [`AccessibilityNode`](https://docs.rs/bevy/0.19.0-rc.3/bevy/a11y/struct.AccessibilityNode.html) 属性与 [`AccessibleLabel`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.AccessibleLabel.html) 组件的载荷同步，满足 `accesskit` 的需求。

## 接下来是什么？（What's Next?）[#](https://bevy.org/news/bevy-0-19/#what-s-next)

无论我们添加多少功能，社区总是会要求**更多**。不幸的是，游戏引擎永远不会**完成**。

应大家的要求，让我们深入时间的迷雾，看看 Bevy 还有哪些正在进行中的功能！像往常一样，其中许多功能是"Bevy 场景编辑器的重要组成部分"，即使它们不是"编辑器本身"。这使我们能够增量地交付有用的零碎部分，并在整合过程中不断完善它们。

- **`.bsn` 场景格式：** 打赌你没想到这个。将 BSN 风格的资源文件实际加载和保存到磁盘仍然是最高优先级。`.bsn` 是我们传说中的编辑器将要实际编辑的东西，允许你创建和组合角色、游戏对象和关卡。
- **统一的 2D 和 3D 渲染内部：** Bevy 的 2D 渲染足够快，但在性能和功能方面已开始落后于我们的 3D 渲染。我们希望统一其内部架构以避免重复工作，同时保持高级 `Sprite` API 完全不变。
- **实体检查器：** 在你的游戏中以及在最终的编辑器中检查实体树。框架已经原型化，widget 的开发也在持续推进。现在是时候将它们整合在一起了！
- **资产即实体：** Bevy 的资源系统是一个定制的代码块，有着自己独特的惯用法和复杂的 API 表面。我们准备将其移入 ECS 本身，使引擎内部和最终用户都能利用他们在其他各处已经使用的强大功能。
- **WESL 着色器语言：** WGSL 是一种够用的着色器语言，但它缺少一些重要的细节。Bevy 一直在与一个跨项目小组合作，以 [WESL](https://github.com/webgpu-tools/wesl-spec) 的形式扩展它。我们[支持 WESL 已超过一年](https://github.com/bevyengine/bevy/pull/17953)，但我们计划将现有的内部着色器移植到 WESL，并将其作为 Bevy 的首选着色器语言。
- **更完整的 Bevy 书籍：** 希望 Bevy Book 更长？我们也是！我们已经大幅扩展了它，涵盖了更广泛的主题并更加深入，希望在 0.20 开发周期内尽快发布已完成的内容。随着引擎更多部分达到"足够稳定"的状态，期待源源不断的新章节。