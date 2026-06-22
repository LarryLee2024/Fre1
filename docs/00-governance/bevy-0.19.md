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

### Optional Fields [#](https://bevy.org/news/bevy-0-19/#optional-fields)

In BSN, you don't need to specify every field, or use `..Default::default()`. You only need to set the fields you care about, and the rest will have their default values:

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

You can also just specify the type name if you want all the fields to take on their default values:

```rust
bsn! {
    Player
}
```

Field values can be arbitrary Rust expressions via `{}` syntax:

```rust
bsn! {
    Player { score: {current_points + 10} }
}
```

### BSN Relationships [#](https://bevy.org/news/bevy-0-19/#bsn-relationships)

BSN has first-class support for ECS Relationships. You can spawn related entities (such as children) inline:

```rust
bsn! {
    Player
    Children [
        Sword,
        Shield,
    ]
}
```

This also works for custom relationships:

```rust
bsn! {
    Player
    Inventory [
        Apple,
        Potion,
    ]
}
```

### Scene Functions [#](https://bevy.org/news/bevy-0-19/#scene-functions)

[`bsn!`](https://docs.rs/bevy_scene/0.19.0-rc.3/bevy_scene/macro.bsn.html) returns a type that implements the [`Scene`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.Scene.html) trait, meaning you can define reusable BSN functions like this:

```rust
fn player() -> impl Scene {
    bsn! {
        Player
        Children [ Sword, Shield ]
    }
}
```

These can accept and use parameters:

```rust
fn player(name: &str) -> impl Scene {
    bsn! {
        Name(name)
        Player
    }
}
```

### Scenes are Composable Patches [#](https://bevy.org/news/bevy-0-19/#scenes-are-composable-patches)

A BSN expression is a "patch"; it does not write a "full" instance of every type it defines. This means you can layer scenes on top of each other:

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

`my_button` will spawn with a `Node { width: px(100), height: px(100) }` component. Components in scenes are initialized to their defaults, and each additional scene layer writes its fields on top of those defaults.

### Scene Assets and Caching [#](https://bevy.org/news/bevy-0-19/#scene-assets-and-caching)

While **Bevy 0.19** doesn't ship with an official `.bsn` asset loader, it _does_ already functionally support scene asset dependencies. We just don't yet include any built-in loaders for them:

```rust
commands.queue_spawn_scene(bsn! {
    :"player.bsn"
    Transform {
        translation: Vec3 { x: 10. }
    }
})
```

This (if there was a `.bsn` asset loader) would spawn a scene that includes the `"player.bsn"` scene asset and patches the "x position" to be `10`. BSN is dependency-aware: if you use `queue_spawn_scene` instead of `spawn_scene`, it will wait to spawn the scene until all dependencies have loaded. `spawn_scene` will always try to spawn the scene immediately ... if it has scene dependencies that aren't loaded yet it will fail.

Also note the `:`, which is "caching" syntax. When first loaded, this will resolve the `"player.bsn"` scene and cache the results for reuse. This makes spawning multiple instances of the scene much cheaper, as it only needs to resolve whatever is layered "on top" of the cached scene.

We're [working](https://github.com/bevyengine/bevy/pull/23576) on an official `.bsn` asset loader, and we also plan on porting Bevy's glTF loader to the new scene system (so you can depend on `"my_scene.gltf"` just like you would a `my_scene.bsn` file). The `bsn!` macro and spawning system already supports scene assets, so if you're feeling adventurous you can try implementing your own Bevy scene asset format while you wait for ours!

### Scene Lists [#](https://bevy.org/news/bevy-0-19/#scene-lists)

[`bsn!`](https://docs.rs/bevy_scene/0.19.0-rc.3/bevy_scene/macro.bsn.html) / [`Scene`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.Scene.html) corresponds to a single entity. [`bsn_list!`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/macro.bsn_list.html) / [`SceneList`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.SceneList.html) is the same idea, but applied to lists of entities:

```rust
fn players() -> impl SceneList {
    bsn_list! [
        (#Player1 Team::Blue),
        (#Player2 Team::Red),
    ]
}
```

Entities in a [`bsn_list!`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/macro.bsn_list.html) are comma separated, and the parentheses to visually indicate entity boundaries are optional:

```rust
fn players() -> impl SceneList {
    bsn_list! [
        #Player1 Team::Blue,
        #Player2 Team::Red,
    ]
}
```

The "BSN relationship syntax" seen above (ex: `Children []`) uses [`SceneList`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.SceneList.html). This means you can pass scene lists as arguments to your scenes:

```rust
fn widget(children: impl SceneList) -> impl Scene {
    bsn! {
        Widget
        Children [ {children} ]
    }
}
```

### Observing Events [#](https://bevy.org/news/bevy-0-19/#observing-events)

[`bsn!`](https://docs.rs/bevy_scene/0.19.0-rc.3/bevy_scene/macro.bsn.html) entities can easily observe events, making it easy to embed "callback-style" behaviors in your scenes:

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

### Templates [#](https://bevy.org/news/bevy-0-19/#templates)

A BSN expression actually defines "templates" for components rather than the actual components themselves. A [`Template`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.Template.html) is essentially a fancy constructor for a type, which produces an output type (such as a Component). Critically, [`Template`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.Template.html) has access to the [`World`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/struct.World.html), the current entity, and the "scene spawn context". This enables powerful behaviors, such as loading assets from a given asset path and producing asset handles (ex: `Handle<Image>`).

The "old" approach to spawning via bundles required passing in every ECS dependency into a bundle function and manually using that dependency to produce the final value:

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

This gets _quite_ nasty when spawning complex deeply nested scenes with many dependencies.

BSN makes this all much easier:

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

Spawning a scene no longer requires knowing every little dependency it requires internally, and common actions like loading and assigning assets via their paths is simple!

This does mean that BSN requires types to have a [`Template`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.Template.html). This is accomplished via the [`FromTemplate`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.FromTemplate.html) trait, which tells BSN what [`Template`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.Template.html) type it should use for a given [`Component`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/component/trait.Component.html). [`FromTemplate`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.FromTemplate.html) can be derived, which will also generate a [`Template`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.Template.html) type for your type. Fortunately, most types _do not_ need to derive or implement [`FromTemplate`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.FromTemplate.html) manually. This is because [`FromTemplate`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.FromTemplate.html) and [`Template`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.Template.html) is automatically implemented for every type that implements `Default` and `Clone`. These types are "templates of themselves" and are just "passed through". You only need to derive [`FromTemplate`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.FromTemplate.html) if you need template features (such as the [`Sprite`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.Sprite.html) use case above, which uses a `Handle<Image>` template to accept `"player.png"`).

### Inline Asset Templates [#](https://bevy.org/news/bevy-0-19/#inline-asset-templates)

BSN ships with support for "inline assets" via the `asset_value` template:

```rust
fn cube() -> impl Scene {
    bsn! {
        Mesh3d(asset_value(Cuboid::new(1., 1., 1.)))
    }
}
```

Compare that to what was necessary before!

```rust
fn setup(meshes: Res<Assets<Mesh>>) -> impl Bundle {
    let handle = meshes.add(Cuboid::new(1., 1., 1.));
    Mesh3d(handle)
}
```

### Entity Reference Syntax [#](https://bevy.org/news/bevy-0-19/#entity-reference-syntax)

BSN has special "entity reference syntax" to define an Entity's [`Name`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/name/struct.Name.html) component:

```rust
bsn! {
    #FirstPlayer
    Player
}
```

This is essentially the same as:

```rust
bsn! {
    Name("FirstPlayer")
    Player
}
```

However "entity reference syntax" also enables referencing that entity elsewhere in the scene:

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

You can access _any_ entity reference defined in a given `bsn! {}` scope anywhere else in that scope:

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

In the context of [`bsn_list!`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/macro.bsn_list.html), this enables defining graph structures:

```rust
bsn_list! [
    (#A PointsTo(#B)),
    (#B PointsTo(#A)),
]
```

### Implicit Into [#](https://bevy.org/news/bevy-0-19/#implicit-into)

Most values in "field position" support "implicit `.into()`". This means types that can convert into other types can generally skip manual conversion:

```rust
#[derive(Component, Default, Clone)]
struct Foo(String);

bsn! {
    Foo("hello")
}
```

This works because `"hello"` is a `&str`, which has an `Into<String>` implementation. This is especially nice in the context of defining Bevy UI values:

```rust
// Raw Rust
Node {
    border: UiRect::all(Val::Px(2.0)),
    ..Default::default()
}

// BSN
Node { border: px(2) }
```

`px(2)` is just a function that produces a `Val::Px(2.0)`, and `UiRect` has an `Into` impl for `Val`, which produces `UiRect::all` (writes the value to all four border "sides"). The ergonomics here are competitive with things like CSS, but it is fully statically typed and derived from normal Rust trait conversions (these aren't special cased / hard-coded). This means you can build your own!

### Scene Components [#](https://bevy.org/news/bevy-0-19/#scene-components)

It has almost been a Bevy developer rite of passage to define something like a `Player` component, which has complex behaviors that rely on some larger "scene", and then ask questions like "how do I spawn this all together?" and "how do I write code that can safely assume the whole scene is present?". Bevy developers have solved these problems in a variety of creative ways, but there has never been an easy recommended / idiomatic upstream solution.

BSN solves this problem by making it possible to associate a [`Scene`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.Scene.html) with a [`Component`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/component/trait.Component.html) via the [`SceneComponent`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.SceneComponent.html) derive:

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

Scene components can then be spawned like this:

```rust
world.spawn_scene(bsn! {
    @Player { score: 10 }
})
```

Scene Components must be spawned this way (as a "scene component"), and will log errors if they are spawned directly (ex: via `world.spawn(Player::default())`). Critically, this provides the guarantee that if the `Player` component is present, the full scene will also be present at spawn time. As a developer this means you can write code that queries for `Player` and assume that it will have both a `LeftHand` and a `RightHand` child (provided they haven't been removed since being spawned). This was a major missing piece in the Bevy data model!

Scene Components can also define "props" which are passed into the scene function and can inform BSN outputs:

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

"Props" are evaluated first (before component field patches). Logically, they are evaluated immediately / in-place and the SceneComponent's scene is immediately applied to the current scene. This means the scene they produce can be patched. This _also_ means that you cannot patch "props", as they do not exist later in the scene.

The [`SceneComponent`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.SceneComponent.html) derive also supports shorthand for scene assets:

```rust
#[derive(SceneComponent, Default, Clone)]
#[scene("player.bsn")]
struct Player {
    score: usize
}
```

Again, note that **Bevy 0.19** does not ship with a `.bsn` asset loader. We're working on it!

The [`SceneComponent`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.SceneComponent.html) derive looks for the `Player::scene` function by default, but you can specify a custom function too:

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

### Scene Spawning Systems [#](https://bevy.org/news/bevy-0-19/#scene-spawning-systems)

**Bevy 0.19** ships with a helper to easily spawn scene functions. This is a _fully self-contained_ Bevy app that spawns a 2D scene:

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

`.spawn()` will turn any function that returns a [`Scene`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.Scene.html) or a [`SceneList`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.SceneList.html) into a system that spawns that scene.

## Render Big Scenes Faster! [#](https://bevy.org/news/bevy-0-19/#render-big-scenes-faster)

Authors:[@pcwalton](https://github.com/pcwalton), [@aevyrie](https://github.com/aevyrie), [@tychedelia](https://github.com/tychedelia)

PRs:[#23242](https://github.com/bevyengine/bevy/pull/23242), [#23481](https://github.com/bevyengine/bevy/pull/23481), [#23711](https://github.com/bevyengine/bevy/pull/23711), [#23036](https://github.com/bevyengine/bevy/pull/23036), [#23211](https://github.com/bevyengine/bevy/pull/23211), [#23023](https://github.com/bevyengine/bevy/pull/23023), [#22966](https://github.com/bevyengine/bevy/pull/22966), [#22874](https://github.com/bevyengine/bevy/pull/22874), [#22988](https://github.com/bevyengine/bevy/pull/22988), [#23106](https://github.com/bevyengine/bevy/pull/23106), [#23115](https://github.com/bevyengine/bevy/pull/23115), [#23530](https://github.com/bevyengine/bevy/pull/23530), [#22813](https://github.com/bevyengine/bevy/pull/22813), [#22297](https://github.com/bevyengine/bevy/pull/22297), [#23185](https://github.com/bevyengine/bevy/pull/23185), [#23297](https://github.com/bevyengine/bevy/pull/23297), [#23103](https://github.com/bevyengine/bevy/pull/23103), [#22846](https://github.com/bevyengine/bevy/pull/22846)

In **Bevy 0.19** we continued our focus on making large scale scenes render quickly. Lets first look at some benchmarks!

In **Bevy 0.18**, a laptop with a mobile Nvidia RTX 4090 could handle our `many_cubes` example with 1.6 million cube mesh entities (with Bevy's PBR StandardMaterial), culling enabled, and 116,000 entities in view at about 49.47ms per frame (21 FPS). In **Bevy 0.19**, it renders those cubes at 18.77ms (53 FPS)! With culling disabled (meaning all 1.6 million cubes are rendered), the same example went from 93.1ms to 41.2ms!!

Our new [`bevy_city`](https://github.com/bevyengine/bevy/tree/main/examples/large_scenes/bevy_city) example defaults to generating a city with 55000 rendered entities:

![bevy_city](https://bevy.org/news/bevy-0-19/bevy_city.jpg)

||Static|Moving|
|---|---|---|
|Bevy 0.18|19.3 ms|22.1 ms|
|Bevy 0.19|11.8 ms|16.2 ms|

We accomplished these wins across _many_ changes. They can be summarized as "we moved more things to the GPU, more batched rendering, parallelized more things, and reduced memory accesses".

Click here for a breakdown of each optimization!

- **Unpack multi-drawable batch sets on the GPU**: In order to render using the GPU-driven multi-draw-indirect approach, Bevy needs to group mesh instances into "batch sets". Historically we did this preparatory work on the CPU, but we've managed to move most of this work over to the GPU by doing a "bin unpacking" step on the GPU. Doing this while maintaining hardware compatibility _and_ making updates cheap was challenging! When drawing a million cubes, this managed to save us almost 1.5ms!
- **Batched depth-only prepasses**: For prepasses that don't need material data or just write normals / motion vectors, we now batch them together, this can save a considerable number of draw calls!
- **Sparse mesh uniform buffer uploads**: Bevy now tracks which mesh uniforms have changed and only uploads those changes to the GPU (provided they cross a certain size threshold). For scenes with many meshes whose transforms haven't changed, this can be a huge win!
- **GPU clustering for lights, light probes, and decals**: Bevy now clusters lights on the GPU. On our `many_lights` benchmark, this improved light clustering performance by about 20x!
- **Increased system parallelism**: More rendering systems run in parallel. This is obviously faster!
- **Visibility ranges are checked on the GPU**: We've moved these LOD checks to the GPU.
- **Batched morph targets**: Meshes with morph targets can now be rendered in batches on platforms that support storage buffers. In our `many_morph_targets` example, this resulted in an almost 2x speedup!
- **`NoCpuCulling` optimizations**: Meshes with `NoCpuCulling` are now ignored by the CPU visibility systems entirely, which saves a significant amount of work for meshes with that marker component.
- **Reduced "previous transform" copies**: Bevy's renderer needs the previous frame's transforms. We now only write the previous frame's transform when the transform has been mutated, saving valuable time.
- **Mesh collection updates GPU data directly using shared memory**: This removed a slow sequential bottleneck and saved valuable milliseconds!
- **Use change lists instead of ticks**: We've made our specialization and queuing systems only process the entities they need each frame, significantly cutting down on the work being done!
- **Smarter clustering heuristic usage**: We now cluster using last frame's clustering statistics, which are _better_ and also allow us to void expensive memory scans.
- **Simpler GPU memory copies**: Bevy uses the `encase` library to prepare data for GPU buffers in the layout WGSL expects. In practice, this isn't always needed and incurs unnecessary overhead. We've swapped to cheaper direct memory copies when clustering lights to shave of a bit of time.
- **Parallel mesh collection**: We've parallelized the "gather" step of our mesh collection, which saved ~4ms when rendering 200,000 moving meshes in `bevymark_3d`
- **Optimized dirty transform tree marking**: We now use buffered channels to make parallel concurrent workers propagate dirty bits from leaves to roots. This makes scenes with many static objects much faster! Scenes with many dynamic objects are also a bit faster!
- **Optimized entity removal**: We now scan from the end of the entity list when removing instead of the front, as newer entities are more likely to be dynamic than older entities.

## Solari Improvements [#](https://bevy.org/news/bevy-0-19/#solari-improvements)

Authors:[@JMS55](https://github.com/JMS55), [@dylansechet](https://github.com/dylansechet)

PRs:[#22348](https://github.com/bevyengine/bevy/pull/22348), [#22459](https://github.com/bevyengine/bevy/pull/22459), [#22468](https://github.com/bevyengine/bevy/pull/22468), [#22618](https://github.com/bevyengine/bevy/pull/22618), [#22671](https://github.com/bevyengine/bevy/pull/22671), [#23442](https://github.com/bevyengine/bevy/pull/23442), [#23809](https://github.com/bevyengine/bevy/pull/23809), [#23813](https://github.com/bevyengine/bevy/pull/23813), [#23898](https://github.com/bevyengine/bevy/pull/23898), [#23948](https://github.com/bevyengine/bevy/pull/23948), [#23968](https://github.com/bevyengine/bevy/pull/23968)

![solari](https://bevy.org/news/bevy-0-19/solari.jpg)

Solari, Bevy's realtime pathtraced renderer, has gained several improvements and fixes for mirrors and non-metallic materials, performance improvements, and greatly increased temporal stability.

For more details, read [JMS55's blog post](https://jms55.github.io/posts/2026-04-12-solari-bevy-0-19).

## More Feathers Widgets [#](https://bevy.org/news/bevy-0-19/#more-feathers-widgets)

Authors:[@viridia](https://github.com/viridia), [@jordanhalase](https://github.com/jordanhalase)

PRs:[#23645](https://github.com/bevyengine/bevy/pull/23645), [#23707](https://github.com/bevyengine/bevy/pull/23707), [#23788](https://github.com/bevyengine/bevy/pull/23788), [#23787](https://github.com/bevyengine/bevy/pull/23787), [#23804](https://github.com/bevyengine/bevy/pull/23804), [#23817](https://github.com/bevyengine/bevy/pull/23817), [#23842](https://github.com/bevyengine/bevy/pull/23842), [#23744](https://github.com/bevyengine/bevy/pull/23744), [#23820](https://github.com/bevyengine/bevy/pull/23820), [#23830](https://github.com/bevyengine/bevy/pull/23830), [#23869](https://github.com/bevyengine/bevy/pull/23869), [#23883](https://github.com/bevyengine/bevy/pull/23883), [#23890](https://github.com/bevyengine/bevy/pull/23890), [#23993](https://github.com/bevyengine/bevy/pull/23993), [#24092](https://github.com/bevyengine/bevy/pull/24092)

![feathers widgets](https://bevy.org/news/bevy-0-19/feathers.jpg)

Bevy Feathers, our opinionated UI widget collection designed with the Bevy editor in mind, has added several new widgets this cycle:

- Text input
- Number input
- Dropdown menu button and menu divider
- Disclosure toggle (chevron expand/collapse)
- Icon and label (display primitives)
- Pane, subpane, and group (decorative frames for editors)
- List view
- Scrollbar

We've improved the existing widgets! For full usage and an interactive demo, try out the [`feathers_gallery`](https://github.com/bevyengine/bevy/blob/v0.19.0/examples/ui/widgets/feathers_gallery.rs) example.

### Feathers + BSN = ❤️ [#](https://bevy.org/news/bevy-0-19/#feathers-bsn-red-heart)

The Feathers widgets have migrated to BSN, Bevy's next-generation scene system. BSN is a better foundation for widgets than the old spawn-function approach: it reduces boilerplate, lets you compose widgets together, parameterize widgets with SceneComponent props, reference font/image assets, and register observers in the same declaration.

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

In the future, the same BSN syntax used in the `bsn!` macro will be portable to `.bsn` files, letting devs choose and rapidly swap between code-first and asset-driven workflows when defining UI.

## Text Input [#](https://bevy.org/news/bevy-0-19/#text-input)

Authors:[@ickshonpe](https://github.com/ickshonpe), [@Zeophlite](https://github.com/Zeophlite), [@alice-i-cecile](https://github.com/alice-i-cecile), [@chronicl](https://github.com/chronicl)

PRs:[#19106](https://github.com/bevyengine/bevy/pull/19106), [#23282](https://github.com/bevyengine/bevy/pull/23282), [#23455](https://github.com/bevyengine/bevy/pull/23455), [#23475](https://github.com/bevyengine/bevy/pull/23475), [#23479](https://github.com/bevyengine/bevy/pull/23479), [#23496](https://github.com/bevyengine/bevy/pull/23496), [#23679](https://github.com/bevyengine/bevy/pull/23679), [#23704](https://github.com/bevyengine/bevy/pull/23704), [#23841](https://github.com/bevyengine/bevy/pull/23841), [#23947](https://github.com/bevyengine/bevy/pull/23947), [#23960](https://github.com/bevyengine/bevy/pull/23960), [#23969](https://github.com/bevyengine/bevy/pull/23969), [#24023](https://github.com/bevyengine/bevy/pull/24023), [#24028](https://github.com/bevyengine/bevy/pull/24028), [#24032](https://github.com/bevyengine/bevy/pull/24032)

In **Bevy 0.19**, we've added basic support for text entry, in the form of the [`EditableText`](https://docs.rs/bevy/0.19.0-rc.3/bevy/text/struct.EditableText.html) component. Spawning an entity with this component will create a simple unstyled rectangle of editable text. Our initial text entry supports:

- **Typing**: Press keys on your keyboard, get text (wow!)
- **Cursor navigation**: arrow keys, Home/End, and word-level shortcuts (Ctrl/Alt+arrow)
- **Selection**: Shift+arrow extends by character or word; click and drag with the pointer
- **Multi-click**: double-click to select a word, triple-click to select the whole line
- **Backspace and Delete**: Both for single characters and words
- **Clipboard**: uses the OS clipboard with the `system_clipboard` feature enabled, or an in-app buffer without it
- **Unicode-aware navigation and editing**: 1 byte/char != 1 character
- **Bidirectional text**: allows both left-to-right and right-to-left scripts
- **IME (Input Method Editor) support**: for CJK and other composing scripts
- **Multiline support**: newlines, soft-wrapping, and vertical scrolling
- **Horizontal scrolling**: when content exceeds the visible width
- **Per-character input filtering**: via `EditableTextFilter`
- **Optional select-all on focus**: via the `SelectAllOnFocus` component
- **Max character limits**: via `EditableText::max_characters`

Many important features are currently unimplemented (placeholder text, undo-redo, password masking...). While we've been careful to expose and document the internals so that you can readily implement these features in your own projects, we would like to continue to expand the functionality of the base widget. Please consider making a PR!

To see how to use it in practice, check out our new [`text_input.rs`](https://github.com/bevyengine/bevy/blob/v0.19.0/examples/ui/text/text_input.rs) example.

## Contact Shadows [#](https://bevy.org/news/bevy-0-19/#contact-shadows)

Authors:[@aevyrie](https://github.com/aevyrie)

PRs:[#22382](https://github.com/bevyengine/bevy/pull/22382)

Drag this image to compare

![Contact Shadows Off](https://bevy.org/news/bevy-0-19/no_contact_shadows.jpg)![Contact Shadows On](https://bevy.org/news/bevy-0-19/contact_shadows.jpg)

Bevy 0.19 introduces **contact shadows**, which help shadows capture the details of objects and attach properly to nearby surfaces.

Previously, Bevy's shadows (outside of Solari) were rendered entirely using (cascaded) [shadow maps](https://en.wikipedia.org/wiki/Shadow_mapping). Shadow mapping is a solid, standard technique that works by looking at objects in the scene from the perspective of the light, creating a depth map, then using that to determine which objects should be in shadow. Unfortunately, this technique is fundamentally limited by the resolution of the shadow map textures created, and it only produces good results when the distance between the shadow casting object and the surface it is casting a shadow on is relatively large.

Up close, you either get peter-panning (where the object seems to float above the ground due to a disconnected shadow), or shadow acne (where the shadows self-intersect in unrealistic ways), depending on what your [depth bias](https://renderdiagrams.org/2024/12/18/shadowmap-bias/) is set to. Increasing the resolution of the shadow maps changes what "close" means, but the memory cost is prohibitive. You simply cannot get good short-range shadows with shadow maps alone. You need a complementary solution.

[Contact shadows](https://www.bendstudio.com/blog/inside-bend-screen-space-shadows/) fill that gap. The core idea here is to perform a short-range (and thus affordable) screen-space [raycast](https://en.wikipedia.org/wiki/Ray_casting), tracing a line from surfaces towards lights, checking for nearby occluding objects.

The results are striking: shadows _cling_ to surfaces properly, emphasizing subtle curves in a way that brings objects and characters to life.

Contact shadows are currently supported for directional, point and spot lights. They are toggled per light, and the cost of rendering contact shadows scales with the number of pixels on screen lit by lights with contact shadows enabled, multiplied by the number of such lights. To enable contact shadows for a light, set the `contact_shadows_enabled` field on your light components to `true`, and add the [`ContactShadows`](https://docs.rs/bevy/0.19.0-rc.3/bevy/pbr/struct.ContactShadows.html) component to your camera. Tuning values on that component controls how contact shadows are computed across the scene.

## Physically Based Screen Space Reflections [#](https://bevy.org/news/bevy-0-19/#physically-based-screen-space-reflections)

Authors:[@aevyrie](https://github.com/aevyrie)

PRs:[#22379](https://github.com/bevyengine/bevy/pull/22379)

Bevy's screen space reflections now use a "physically based" algorithm, which improves the quality of our reflections significantly!

![physical_reflections](https://bevy.org/news/bevy-0-19/physical_reflections.png)

## Rectangular Area Lights [#](https://bevy.org/news/bevy-0-19/#rectangular-area-lights)

Authors:[@dylansechet](https://github.com/dylansechet)

PRs:[#23288](https://github.com/bevyengine/bevy/pull/23288)

![rectangular area lights](https://bevy.org/news/bevy-0-19/rect_area_lights.png)

Bevy's lighting toolkit just got a new addition: rectangular area lights!

The implementation uses [Linearly Transformed Cosines](https://eheitzresearch.wordpress.com/415-2/), which is the standard method for real-time area lights and should also help make our spherical area lights more accurate in the near future.

Rectangular lights currently don't cast shadows or have support for anisotropic materials.

You need to enable the `area_light_luts` cargo feature to use it.

Check out [the new example](https://github.com/bevyengine/bevy/blob/v0.19.0/examples/3d/rect_light.rs) to see them in action.

## Richer text [#](https://bevy.org/news/bevy-0-19/#richer-text)

Authors:[@ickshonpe](https://github.com/ickshonpe), [@alice-i-cecile](https://github.com/alice-i-cecile), [@gregcsokas](https://github.com/gregcsokas)

PRs:[#22156](https://github.com/bevyengine/bevy/pull/22156), [#22396](https://github.com/bevyengine/bevy/pull/22396), [#22614](https://github.com/bevyengine/bevy/pull/22614), [#22879](https://github.com/bevyengine/bevy/pull/22879), [#23380](https://github.com/bevyengine/bevy/pull/23380)

Bevy's text system has historically been sparse: pick a font by asset handle, set a size in pixels, done. Want bold? Load a separate bold font asset. Want italic? Another asset. Want the user's system monospace? No luck. Want text that scales with the viewport? Roll it yourself.

Not anymore.

### Better font selection [#](https://bevy.org/news/bevy-0-19/#better-font-selection)

![generic fonts](https://bevy.org/news/bevy-0-19/generic_fonts.jpg)

[`FontSource`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/enum.FontSource.html) now offers three ways to identify a font:

```rust
// Asset handle
FontSource::Handle(asset_server.load("fonts/FiraMono.ttf"))

// Family name
FontSource::Family("FiraMono".into())

// Semantic category
FontSource::Monospace
```

The generic variants — `Serif`, `SansSerif`, `Cursive`, `Fantasy`, `Monospace`, and several UI-specific ones (`SystemUi`, `Emoji`, `Math`, and others) — resolve to configurable defaults. Override them via `FontCx`:

```rust
fn configure_fonts(mut font_cx: ResMut<FontCx>) {
    font_cx.set_serif_family("Merriweather");
    font_cx.set_monospace_family("JetBrains Mono");
}
```

Editor tooling and non-game applications that want to respect the user's font preferences without hardcoding an asset path will find this particularly useful.

System fonts were already loadable via the backend resource in previous releases, but `FontSource::Family` is a cleaner, more powerful way to load them. Enable the `bevy/system_font_discovery` feature to make installed system fonts available by name; without it, `FontSource::Family("...")` will only find fonts explicitly loaded as Bevy assets.

### Variable font properties [#](https://bevy.org/news/bevy-0-19/#variable-font-properties)

![variable font properties](https://bevy.org/news/bevy-0-19/variable_font_properties.jpg)

[`TextFont`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.TextFont.html) has gained the `weight`, `width`, and `style` fields. Pick a variable font, and say goodbye to separate assets for every variant of a typeface:

```rust
TextFont {
    font: FontSource::SansSerif,
    weight: FontWeight::BOLD,
    style: FontStyle::Italic,
    width: FontWidth::CONDENSED,
    ..default()
}
```

### Responsive font sizing [#](https://bevy.org/news/bevy-0-19/#responsive-font-sizing)

`font_size` is now a [`FontSize`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/enum.FontSize.html) enum rather than a bare `f32`:

```rust
TextFont::from_font_size(FontSize::Px(24.0))   // fixed pixels — unchanged behavior
TextFont::from_font_size(FontSize::Vh(5.0))    // 5% of viewport height
TextFont::from_font_size(FontSize::Rem(1.5))   // relative to the RemSize resource
```

The full set of variants mirrors CSS: `Px`, `Vw`, `Vh`, `VMin`, `VMax`, and `Rem`. `Rem` values scale with the `RemSize` resource, giving you a single knob to resize all relative text at once.

### Letter spacing [#](https://bevy.org/news/bevy-0-19/#letter-spacing)

A new [`LetterSpacing`](https://docs.rs/bevy/0.19.0-rc.3/bevy/text/enum.LetterSpacing.html) component controls the spacing between characters:

```rust
commands.spawn((
    Text::new("SPACED OUT"),
    LetterSpacing::Px(4.0),
));
```

While all of these features would have been possible in [`cosmic_text`](https://github.com/pop-os/cosmic-text), we've chosen to migrate to [`parley`](https://github.com/linebender/parley) during this cycle. Both are solid, modern choices, but we found `parley` had meaningfully better documentation and was somewhat nicer to use.

## App Settings [#](https://bevy.org/news/bevy-0-19/#app-settings)

Authors:[@viridia](https://github.com/viridia), [@mpowell90](https://github.com/mpowell90)

PRs:[#22891](https://github.com/bevyengine/bevy/pull/22891), [#23034](https://github.com/bevyengine/bevy/pull/23034), [#23719](https://github.com/bevyengine/bevy/pull/23719), [#23812](https://github.com/bevyengine/bevy/pull/23812)

Bevy now has a built in general-purpose "app settings" system, which Bevy apps can use to load and save arbitrary settings such as:

- Graphics options
- Panel layouts and tool preferences
- Music and sound volume controls
- Window position and size
- "Don't show this dialog again"

Notably, the Bevy Editor needs a settings system for layout preferences, tool configuration, and everything else that should persist between sessions. Because the Bevy Editor is being built _as_ a Bevy app, it can make use of this new settings system!

Settings groups are plain Rust structs that derive [`Resource`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/trait.Resource.html), [`SettingsGroup`](https://docs.rs/bevy/0.19.0-rc.3/bevy/settings/trait.SettingsGroup.html), and [`Reflect`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/trait.Reflect.html):

```rust
#[derive(Resource, SettingsGroup, Reflect, Default)]
#[reflect(Resource, SettingsGroup, Default)]
struct AudioSettings {
    music_volume: f32,
    sfx_volume: f32,
}
```

Adding [`SettingsPlugin`](https://docs.rs/bevy/0.19.0-rc.3/bevy/settings/struct.SettingsPlugin.html) with a unique [reverse-domain](https://en.wikipedia.org/wiki/Reverse_domain_name_notation) app name will automatically load your settings groups on startup and insert them as resources:

```rust
app.add_plugins(SettingsPlugin::new("com.example.mygame"));
```

You can then read them like any other resource:

```rust
fn adjust_volume(audio: Res<AudioSettings>, mut music: ResMut<AudioSink>) {
    music.set_volume(audio.music_volume);
}
```

Settings can then be saved via the [`SaveSettingsDeferred`](https://docs.rs/bevy/0.19.0-rc.3/bevy/settings/struct.SaveSettingsDeferred.html) or [`SaveSettingsSync`](https://docs.rs/bevy/0.19.0-rc.3/bevy/settings/enum.SaveSettingsSync.html) command.

See the [`settings.rs`](https://github.com/bevyengine/bevy/blob/v0.19.0/examples/app/settings.rs) example for a complete walkthrough.

---

A special thanks to Andhrimnir (@tecbeast42) for giving Bevy ownership of the `bevy-settings` crate name on `crates.io`. We built our own brand new settings crate, but we're re-using the `bevy-settings` crate name because it fits the best.

## More Post-Processing Effects [#](https://bevy.org/news/bevy-0-19/#more-post-processing-effects)

Authors:[@Breakdown-Dog](https://github.com/Breakdown-Dog)

PRs:[#22564](https://github.com/bevyengine/bevy/pull/22564), [#23110](https://github.com/bevyengine/bevy/pull/23110)

Two new post-processing effects were added in this cycle, both classic tools for giving your camera a more cinematic or stylized look.

### Vignette [#](https://bevy.org/news/bevy-0-19/#vignette)

Drag this image to compare

![Without Vignette](https://bevy.org/news/bevy-0-19/post_processing_base.jpg)![With Vignette](https://bevy.org/news/bevy-0-19/post_processing_vignette.jpg)

Vignette reduces image brightness towards the periphery of the frame, drawing the viewer's eye to the center.

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

### Lens Distortion [#](https://bevy.org/news/bevy-0-19/#lens-distortion)

Drag this image to compare

![No Distortion](https://bevy.org/news/bevy-0-19/post_processing_base.jpg)![Barrel Distortion](https://bevy.org/news/bevy-0-19/post_processing_barrel_distortion.jpg)

Drag this image to compare

![No Distortion](https://bevy.org/news/bevy-0-19/post_processing_base.jpg)![Pincushion Distortion](https://bevy.org/news/bevy-0-19/post_processing_pincushion_distortion.jpg)

Lens distortion warps the image spatially. Positive `intensity` pushes the edges outward (barrel distortion), negative pulls them inward (pincushion distortion).

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

## Render Recovery [#](https://bevy.org/news/bevy-0-19/#render-recovery)

Authors:[@atlv24](https://github.com/atlv24), [@kfc35](https://github.com/kfc35)

PRs:[#22761](https://github.com/bevyengine/bevy/pull/22761), [#23350](https://github.com/bevyengine/bevy/pull/23350), [#23349](https://github.com/bevyengine/bevy/pull/23349), [#23433](https://github.com/bevyengine/bevy/pull/23433), [#23458](https://github.com/bevyengine/bevy/pull/23458), [#23444](https://github.com/bevyengine/bevy/pull/23444), [#23459](https://github.com/bevyengine/bevy/pull/23459), [#23461](https://github.com/bevyengine/bevy/pull/23461), [#23463](https://github.com/bevyengine/bevy/pull/23463), [#22714](https://github.com/bevyengine/bevy/pull/22714), [#22759](https://github.com/bevyengine/bevy/pull/22759), [#16481](https://github.com/bevyengine/bevy/pull/16481), [#24131](https://github.com/bevyengine/bevy/pull/24131)

GPU errors previously had no recovery path — a driver crash, an out-of-memory condition, or a device loss would silently hang or crash the app. This was particularly frustrating in long-lived applications (like art installations) or on devices with frequent failures, such as VR headsets. Bevy now surfaces these as typed errors and lets you decide what to do with each one:

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

`DeviceLost` is the case most games will want to handle: it covers GPU driver crashes, thermal shutdowns, and hardware being physically disconnected. `RenderErrorPolicy::Recover` reinitializes the renderer and keeps the app running. `StopRendering` halts rendering but leaves the rest of the app alive — useful if you want to show an error screen or save state before exiting. `Ignore` silently swallows the error, which is the existing behavior for validation errors. Panicking remains appropriate for `Internal` errors, which indicate bugs.

Be sure to test your error recovery carefully in your games; we've seen hardware-specific cases of flickering during repeated failures (as might be caused by an out-of-memory problem), which are a serious accessibility risk for people with photosensitive epilepsy. While we're looking to solve that problem for good in later releases, we've currently opted for a conservative default. If you don't configure a [`RenderErrorHandler`](https://docs.rs/bevy/0.19.0-rc.3/bevy/render/error_handler/struct.RenderErrorHandler.html), behavior is similar to but not identical to before: Vulkan validation errors are ignored, everything else sends an `AppExit` event to gracefully shut down.

## Render Graph as Systems [#](https://bevy.org/news/bevy-0-19/#render-graph-as-systems)

Authors:[@tychedelia](https://github.com/tychedelia)

PRs:[#22144](https://github.com/bevyengine/bevy/pull/22144)

Bevy's `RenderGraph` architecture has been replaced with ECS schedules. Render passes are now regular systems that run in schedules such as [`Core3d`](https://docs.rs/bevy/0.19.0-rc.3/bevy/core_pipeline/struct.Core3d.html), [`Core2d`](https://docs.rs/bevy/0.19.0-rc.3/bevy/core_pipeline/struct.Core2d.html), which are executed on the render world.

The old render graph was originally designed when Bevy's ECS was less mature. In order to add custom rendering functionality, we required users to implement a trait `Node`, derive a `RenderLabel`, and use a targeted API for ordering this rendering work relative to other tasks. This required a lot of boilerplate!

Click here to see what it used to look like!

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

As Bevy ECS has evolved, [`Schedule`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/struct.Schedule.html) has become capable of expressing the "render graph" pattern. Using the ECS directly lets rendering better leverage familiar Bevy patterns, allowing the above to be expressed much more succinctly:

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

In the future, expressing rendering work as systems will allow us to explore performance optimizations that take advantage of the ECS. For example, future work to support read-only schedules could help parallelizing command encoding by enforcing that a schedule does not mutate the world. We are excited to continue to improve the experience of custom rendering inside Bevy!

## Improved Skinned Mesh Culling [#](https://bevy.org/news/bevy-0-19/#improved-skinned-mesh-culling)

Authors:[@greeble-dev](https://github.com/greeble-dev)

PRs:[#21837](https://github.com/bevyengine/bevy/pull/21837)

In earlier Bevy versions, animated characters and creatures would sometimes vanish mid-animation. This happened because Bevy used the skeleton's resting position to decide which meshes were on-screen, rather than their actual animated pose. A character raising their arms could have those arms literally outside the bounding box Bevy used for culling.

Skinned meshes now compute their bounds from actual joint positions each frame, fixing disappearing meshes like those reported in [#4971](https://github.com/bevyengine/bevy/issues/4971). If you load skinned meshes from glTFs, this is automatic — no changes needed.

For hand-crafted skinned meshes, call `Mesh::generate_skinned_mesh_bounds` and add `DynamicSkinnedMeshBounds` to the entity:

```rust
let mut mesh: Mesh = ...;
mesh.generate_skinned_mesh_bounds()?;

entity.insert((
    Mesh3d(meshes.add(mesh)),
    DynamicSkinnedMeshBounds,
));
```

## Parallax Corrected Cubemaps [#](https://bevy.org/news/bevy-0-19/#parallax-corrected-cubemaps)

Authors:[@pcwalton](https://github.com/pcwalton)

PRs:[#22582](https://github.com/bevyengine/bevy/pull/22582)

Drag this image to compare

![Correction Off](https://bevy.org/news/bevy-0-19/parallax_correction_off.jpg)![Correction On](https://bevy.org/news/bevy-0-19/parallax_correction_on.jpg)

Bevy previously rendered cubemap reflections as though the environment were infinitely far away. For outdoor scenes this was often fine, but for indoor scenes and dense environments the result looked wrong — reflections didn't line up with the actual geometry around the viewer.

The standard fix is parallax correction: each reflection probe gets its own bounding box, and a raytrace against that box determines the correct sampling direction for the cubemap. Bevy now applies this automatically for light probes, using the probe's influence bounding box as the correction volume. This is a reasonable default for a cubemap capturing a rectangular room interior, and matches Blender's approach.

Parallax correction is enabled by default. To opt out on a specific probe, add `ParallaxCorrection::None`:

```rust
commands.spawn((
    LightProbe,
    EnvironmentMapLight { .. },
    ParallaxCorrection::None,
));
```

A new `pccm` example demonstrates the effect, with parallax correction toggleable at runtime.

## Partial Bindless / Reduced Bind Group Overhead [#](https://bevy.org/news/bevy-0-19/#partial-bindless-reduced-bind-group-overhead)

Authors:[@holg](https://github.com/holg)

PRs:[#23436](https://github.com/bevyengine/bevy/pull/23436)

Bindless rendering is how modern engines handle scenes with many different materials efficiently: shaders index into shared pools of textures and buffers rather than rebinding them each draw call.

WGPU's backend for Metal (Apple's GPU API) has partial bindless support. It currently only permits texture binding arrays but not buffer binding arrays.

Historically, Bevy required support for both features before it would use bindless, which excluded Metal entirely, even for materials that never use buffer arrays.

Most materials, including [`StandardMaterial`](https://docs.rs/bevy/0.19.0-rc.3/bevy/pbr/struct.StandardMaterial.html), do not need buffer array support. To ensure those materials take the fast path, Bevy now checks the actual needs of each material. If you only need texture arrays, your material can be rendered efficiently across Bevy's desktop platforms. If you use `#[uniform(..., binding_array(...))]`, expect performance degradation on Metal.

We've also fixed two important correctness bugs in the process:

1. We discovered that the sampler limit check was testing the wrong metric: `max_samplers_per_shader_stage` counts binding slots, but the relevant limit is `max_binding_array_sampler_elements_per_shader_stage`, the array element count (a mismatch that could incorrectly disable bindless).
2. Bevy now also skips creating binding array slots for resource types a material doesn't use, staying within Metal's hard 31 argument buffer slot limit and reducing overhead on all platforms.

Benchmarked on Bistro Exterior (698 materials) we saw significant frame time improvements (and sometimes memory improvements) across many hardware configurations:

|GPU|Frame Time Speedup|Memory|
|---|---|---|
|Apple M2 Max (Metal)|+15%|−57 MB RAM|
|NVIDIA 5060 Ti|+46%|Same|
|AMD Vega 8 / Ryzen 4800U|Same|−88 MB VRAM|
|Intel i360P|+14%|Same|
|Intel Iris XE|Same|Same|

[Bistro](https://developer.nvidia.com/orca/amazon-lumberyard-bistro) is a demanding, fairly realistic scene. While bindless limitations remain frustrating, especially on Mac where Vulkan isn't an option, it's lovely to see those performance gains, and to know that Bevy itself is no longer artificially holding our users back.

## Diagnostics Overlay [#](https://bevy.org/news/bevy-0-19/#diagnostics-overlay)

Authors:[@hukasu](https://github.com/hukasu), [@cart](https://github.com/cart)

PRs:[#22486](https://github.com/bevyengine/bevy/pull/22486)

![overlay](https://bevy.org/news/bevy-0-19/overlay.jpg)

Bevy's diagnostics have always been easy to dump to the terminal, but displaying them in-game meant wiring up your own UI. [`DiagnosticsOverlayPlugin`](https://docs.rs/bevy/0.19.0-rc.3/bevy/dev_tools/diagnostics_overlay/struct.DiagnosticsOverlayPlugin.html) adds a built-in overlay for this, with presets for common cases:

```rust
commands.spawn(DiagnosticsOverlay::fps());
commands.spawn(DiagnosticsOverlay::mesh_and_standard_material());
```

You can also build a custom overlay from any [`DiagnosticPath`](https://docs.rs/bevy/0.19.0-rc.3/bevy/diagnostic/struct.DiagnosticPath.html) list:

```rust
commands.spawn(DiagnosticsOverlay::new("Diagnostics", vec![
    MyDiagnostics::COUNTER.into()
]));
```

By default the overlay shows the smoothed moving average. You can switch to the latest value or the raw moving average via [`DiagnosticsOverlayStatistic`](https://docs.rs/bevy/0.19.0-rc.3/bevy/dev_tools/diagnostics_overlay/enum.DiagnosticsOverlayStatistic.html), and configure floating-point precision with [`DiagnosticsOverlayItem::precision`](https://docs.rs/bevy/0.19.0-rc.3/bevy/dev_tools/diagnostics_overlay/struct.DiagnosticsOverlayItem.html#structfield.precision):

```rust
commands.spawn(DiagnosticsOverlay::new("Diagnostics", vec![DiagnosticsOverlayItem {
    path: MyDiagnostics::COUNTER,
    statistic: DiagnosticsOverlayStatistic::Value,
    precision: 4,
}]));
```

## Contiguous Query Access [#](https://bevy.org/news/bevy-0-19/#contiguous-query-access)

Authors:[@Jenya705](https://github.com/Jenya705)

PRs:[#21984](https://github.com/bevyengine/bevy/pull/21984), [#24181](https://github.com/bevyengine/bevy/pull/24181)

[SIMD](https://en.wikipedia.org/wiki/Single_instruction,_multiple_data) is a critical tool for performance optimization, but using it in Bevy has always been harder than it needed to be. Table components in Bevy are already laid out flat in memory — all [`Transform`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.Transform.html) components are stored as values in a contiguous table, exactly what SIMD wants. The [`Query`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/struct.Query.html) iterator just wasn't exposing that structure: it handed you one entity's component at a time, and the compiler had no way to know the underlying data was a contiguous array.

[`contiguous_iter`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.QueryState.html#method.contiguous_iter) and [`contiguous_iter_mut`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.QueryState.html#method.contiguous_iter_mut) hand you the whole table slice at once. LLVM can see the contiguous array and auto-vectorize — or you can reach for explicit SIMD yourself.

On a bulk `position += velocity` update over 10,000 entities, this gives some serious speedups:

|Method|Time|Time (AVX2)|
|---|---|---|
|Normal iteration|5.58 µs|5.51 µs|
|Contiguous iteration|4.88 µs|1.87 µs|
|Contiguous, no change detection|4.40 µs|1.58 µs|

If your project has CPU-heavy workloads (physics engines are a prime example), you should try this out immediately.

```rust
fn apply_health_decay(mut query: Query<(&mut Health, &HealthDecay)>) {
    for (mut health, decay) in query.contiguous_iter_mut().unwrap() {
        for (h, d) in health.iter_mut().zip(decay) {
            h.0 *= d.0;
        }
    }
}
```

The [`contiguous_iter`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.QueryState.html#method.contiguous_iter) family of methods only returns `Ok` if the query is dense. That means:

- All of the fetched components must use the default "table" storage strategy.
- The query filters cannot disrupt the returned query data. "Archetypal filters" like `With<T>` and `Without<T>` are fine; `Changed<T>` and `Added<T>` are not, since they require a per-entity check that makes it impossible to return raw table slices.

Because these conditions are fixed properties of the query type, you're safe to unwrap here unless you are writing generic code, or working with dynamic components.

You may have noticed that the table above had _three_ rows. While change detection is a generally useful feature, it does incur measurable performance overhead. By default, [`contiguous_iter_mut`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.QueryState.html#method.contiguous_iter_mut) returns `ContiguousMut<T>`. Just like the ordinary `Mut<T>`, it triggers change detection automatically on dereference. If you don't care about that, `bypass_change_detection()` gives you the raw `&mut [T]` directly for even faster access. Vroom!

## Delayed Commands [#](https://bevy.org/news/bevy-0-19/#delayed-commands)

Authors:[@Runi-c](https://github.com/Runi-c)

PRs:[#23090](https://github.com/bevyengine/bevy/pull/23090)

Scheduling things to happen some time in the future is a common and useful tool in game development for everything from gameplay logic to audio cues to VFX.

While this was previously possible through careful use of timers, getting the details right was surprisingly tricky and naive solutions were heavy on boilerplate.

Now, you can simply delay arbitrary commands to be executed later.

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

Note that this does not have a built-in, blessed cancellation mechanism yet. We recommend embedding the originating [`Entity`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/entity/struct.Entity.html) into the command if you want to cancel the action if that entity dies or is despawned.

## Text Gizmos [#](https://bevy.org/news/bevy-0-19/#text-gizmos)

Authors:[@ickshonpe](https://github.com/ickshonpe), [@nuts-rice](https://github.com/nuts-rice)

PRs:[#22732](https://github.com/bevyengine/bevy/pull/22732), [#23120](https://github.com/bevyengine/bevy/pull/23120)

![text gizmos](https://bevy.org/news/bevy-0-19/text_gizmos.jpg)

Sometimes you just want to slap a label on something while debugging. Text gizmos are for exactly that: a zero-setup way to draw world-space text anywhere in your scene using a built-in stroke font.

Unlike Bevy's [`Text2D`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.Text2d.html) — the right choice for damage numbers, nameplates, and in-game labels — text gizmos are _strictly_ for dev tools and debugging. The font is fixed and only supports ASCII.

Use [`Gizmos::text`] and `text_2d` to quickly draw text:

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

If you want to color each section of characters separately, reach for `text_sections` and `text_sections_2d`.

## Cancellable Web Tasks [#](https://bevy.org/news/bevy-0-19/#cancellable-web-tasks)

Authors:[@NthTensor](https://github.com/NthTensor), [@Gingeh](https://github.com/Gingeh)

PRs:[#21795](https://github.com/bevyengine/bevy/pull/21795)

When a [`Task`](https://docs.rs/bevy/0.19.0-rc.3/bevy/tasks/struct.Task.html) in Bevy is dropped, it's supposed to be cancelled, stopping the underlying work at the next yield point. On web, this never worked. `wasm_bindgen_futures::spawn_local` hands your future directly to the JS event loop with no handle to take it back, so Bevy's task wrapper was just a receipt with no power to cancel.

As a result, code that correctly managed task lifetimes on native desktop and mobile platforms silently leaked work on web.

```rust
fn update_background_task(mut task_handle: ResMut<CurrentTask>) {
    // Replaces the old task. On native: the old task is dropped and cancelled.
    // On web: the old task kept running. Oops.
    task_handle.0 = AsyncComputeTaskPool::get().spawn(async { do_work().await });
}
```

Fixing this required a new approach to the WASM executor. The [`web-task`](https://crates.io/crates/web-task) crate, built by our very own `@NthTensor`, builds cooperative cancellation on top of the JS event loop: spawned tasks check an abort flag at every yield point. Bevy now uses it on WASM, so [`Task`](https://docs.rs/bevy/0.19.0-rc.3/bevy/tasks/struct.Task.html) drop semantics are finally identical on all platforms.

## Asset Saving [#](https://bevy.org/news/bevy-0-19/#asset-saving)

Authors:[@andriyDev](https://github.com/andriyDev)

PRs:[#22622](https://github.com/bevyengine/bevy/pull/22622)

Bevy has had an [`AssetSaver`](https://docs.rs/bevy/0.19.0-rc.3/bevy/asset/saver/trait.AssetSaver.html) trait since 0.12. However, it was only ever intended for use inside asset processing pipelines, not for saving assets at runtime. This left a frustrating gap: if you wanted to save a procedurally generated mesh, a baked lightmap, or the output of an in-editor workflow, there was no supported path to do it.

Now there is. `save_using_saver` lets you save any asset to disk using an `AssetSaver` implementation of your choice.

### 1. Building the `SavedAsset` [#](https://bevy.org/news/bevy-0-19/#1-building-the-savedasset)

For simple assets with no sub-assets, use `SavedAsset::from_asset`:

```rust
let main_asset = InlinedBook {
    lines: vec!["Save me!".to_string(), "Please!".to_string()],
};
let saved_asset = SavedAsset::from_asset(&main_asset);
```

For assets that reference other assets (sub-assets), use `SavedAssetBuilder`:

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

`SavedAsset` borrows rather than owns its assets. That means you can build and save in the same async block — no need to transfer ownership first.

### 2. Calling `save_using_saver` [#](https://bevy.org/news/bevy-0-19/#2-calling-save-using-saver)

```rust
save_using_saver(
    asset_server.clone(),
    &MyAssetSaver::default(),
    &asset_path,
    saved_asset,
    &MySettings::default(),
).await.unwrap();
```

`save_using_saver` is async. Generally, you'll want to spawn it with `IoTaskPool::get().spawn(...)`. You'll also need to implement `AssetSaver` for `MyAssetSaver` to define the serialization format.

## Resources as Components [#](https://bevy.org/news/bevy-0-19/#resources-as-components)

Authors:[@Trashtalk217](https://github.com/Trashtalk217), [@cart](https://github.com/cart), [@SpecificProtagonist](https://github.com/SpecificProtagonist)

PRs:[#20934](https://github.com/bevyengine/bevy/pull/20934), [#22910](https://github.com/bevyengine/bevy/pull/22910), [#22911](https://github.com/bevyengine/bevy/pull/22911), [#22919](https://github.com/bevyengine/bevy/pull/22919), [#22930](https://github.com/bevyengine/bevy/pull/22930), [#23616](https://github.com/bevyengine/bevy/pull/23616), [#23716](https://github.com/bevyengine/bevy/pull/23716), [#24077](https://github.com/bevyengine/bevy/pull/24077), [#24164](https://github.com/bevyengine/bevy/pull/24164)

Resources and components have always been separate concepts in Bevy's ECS. While the simple `Res<Time>` sugar is nice, the only real distinction is cardinality — a resource is a component of which at most one exists at any time.

That separation has been a persistent source of friction. Many of our tools for components (like hooks, observers, and relations) simply weren't available for resources, and the engine carried a significant amount of duplicated internal machinery to keep the two mechanisms in sync.

In **Bevy 0.19**, resources are now stored as components on singleton entities, unifying our internals and giving resources more capabilities. You can now:

- Simplify networking and dev-tools code by assuming that entities + components are the only form of data you need to worry about
- Query over both resources and components to support flexible usage patterns
- Add relationships pointing to and from resource entities
- Add additional components to your resource entities
- Add lifecycle observers to your resource types
- Add your own hooks to resources
- Mark resources as immutable

## Remote Entity Reservation [#](https://bevy.org/news/bevy-0-19/#remote-entity-reservation)

Authors:[@ElliottjPierce](https://github.com/ElliottjPierce), [@alice-i-cecile](https://github.com/alice-i-cecile), [@cart](https://github.com/cart)

PRs:[#18670](https://github.com/bevyengine/bevy/pull/18670), [#22658](https://github.com/bevyengine/bevy/pull/22658)

Bevy has historically required a [`World`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/struct.World.html) reference to allocate entity IDs. This works in most scenarios, but it means that if you want to do work in parallel that initializes entities, you need to block your app's execution! This is problematic for things like our upcoming "assets as entities" work, which will involve preparing entity contents in the background while the app continues to run.

**Bevy 0.19** introduces a new entity allocation strategy that enables reserving entity IDs from any thread without a [`World`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/prelude/struct.World.html) reference _and_ without compromising on performance. This involved splitting the [entity lifecycle](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/entity/index.html#entity-life-cycle) into five stages: unallocated, allocated, spawned, despawned, and freed.

## Interactive Transform Gizmo [#](https://bevy.org/news/bevy-0-19/#interactive-transform-gizmo)

Authors:[@jbuehler23](https://github.com/jbuehler23), [@aevyrie](https://github.com/aevyrie)

PRs:[#23435](https://github.com/bevyengine/bevy/pull/23435)

A transform gizmo — the click-and-drag handles for translating, rotating, and scaling objects in a 3D viewport — is one of the first things anyone reaches for when building a level editor. Bevy now has one built in, for your use today and our own use in the future.

Add [`TransformGizmoPlugin`](https://docs.rs/bevy/0.19.0-rc.3/bevy/gizmos/prelude/struct.TransformGizmoPlugin.html), mark a camera with [`TransformGizmoCamera`](https://docs.rs/bevy/0.19.0-rc.3/bevy/gizmos/prelude/struct.TransformGizmoCamera.html), and tag entities with [`TransformGizmoFocus`](https://docs.rs/bevy/0.19.0-rc.3/bevy/gizmos/prelude/struct.TransformGizmoFocus.html):

```rust
app.add_plugins(TransformGizmoPlugin);

commands.spawn((Camera3d::default(), TransformGizmoCamera));
commands.spawn((Mesh3d(mesh), TransformGizmoFocus));
```

The plugin is deliberately not connected to user input. This keeps the gizmo composable for editor authors who already have opinions about input handling. Sensitivity, snapping, and screen-space scaling are all configurable via [`TransformGizmoSettings`](https://docs.rs/bevy/0.19.0-rc.3/bevy/gizmos/prelude/struct.TransformGizmoSettings.html), while modes are controlled via the [`TransformGizmoMode`](https://docs.rs/bevy/0.19.0-rc.3/bevy/gizmos/prelude/enum.TransformGizmoMode.html) resource.

Much of the math and implementation strategy for this widget comes from the [`bevy_transform_gizmo`](https://github.com/fslabs/bevy_transform_gizmo) crate. Thanks again to Foresight Spatial Labs for their generous open source contributions!

## Infinite Grid [#](https://bevy.org/news/bevy-0-19/#infinite-grid)

Authors:[@IceSentry](https://github.com/IceSentry)

PRs:[#23482](https://github.com/bevyengine/bevy/pull/23482)

![infinite grid](https://bevy.org/news/bevy-0-19/infinite_grid.jpg)

A transparent ground-plane grid is a staple of 3D editor tooling: it marks the major axes, orients the scene, and makes scale immediately legible.

Simply drawing lines doesn't work well: the mesh has to end somewhere, and the lines that reach toward the horizon create aliasing artifacts and Moiré patterns no matter how far you extend it.

Our implementation renders the grid as a fullscreen shader: the grid is computed per-pixel in screen space from the camera's perspective, and fades out with distance to eliminate aliasing at the horizon.

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

Grid appearance — colors, fade distance, line scale — is controlled by [`InfiniteGridSettings`](https://docs.rs/bevy/0.19.0-rc.3/bevy/dev_tools/infinite_grid/struct.InfiniteGridSettings.html), which can be placed on the grid entity or on a specific camera to override it per-view. You can see how this works in the new [`infinite_grid.rs`](https://github.com/bevyengine/bevy/blob/v0.19.0/examples/dev_tools/infinite_grid.rs) example.

This is an upstreamed version of the [`bevy_infinite_grid` crate](https://github.com/fslabs/bevy_infinite_grid), created and maintained by Foresight Spatial Labs — thank you for building it and generously contributing it to Bevy!

## White Furnace Test [#](https://bevy.org/news/bevy-0-19/#white-furnace-test)

Authors:[@dylansechet](https://github.com/dylansechet)

PRs:[#23194](https://github.com/bevyengine/bevy/pull/23194), [#23203](https://github.com/bevyengine/bevy/pull/23203)

Drag this image to compare

![Before](https://bevy.org/news/bevy-0-19/white_furnace_before.jpg)![After](https://bevy.org/news/bevy-0-19/white_furnace_after.jpg)

The [white furnace test](https://lousodrome.net/blog/light/2023/10/21/the-white-furnace-test/) is a classic sanity check for physically-based renderers. Place a perfectly reflective object inside a uniform white environment, and it should be indistinguishable from the background, no matter how metallic and rough. Any object that remains visible is a sign that the shader is creating or absorbing energy it shouldn't.

Bevy used to fail this test, meaning something was wrong with our shader math. Two bugs were responsible:

- Seams were visible when using [`GeneratedEnvironmentMapLight`](https://docs.rs/bevy/0.19.0-rc.3/bevy/light/struct.GeneratedEnvironmentMapLight.html) for certain surface orientations.
- Partially metallic materials absorbed energy, appearing darker than they should be.

After fixing those, Bevy passes the test. That means your materials will behave more correctly under image-based lighting.

A gray image has never been so exciting!

## Observer Run Conditions [#](https://bevy.org/news/bevy-0-19/#observer-run-conditions)

Authors:[@jonas-meyer](https://github.com/jonas-meyer)

PRs:[#22602](https://github.com/bevyengine/bevy/pull/22602)

Run conditions are a convenient, reusable pattern for skipping systems when certain conditions are met. Previously, run conditions only worked for ordinary systems. Observers couldn't use them.

Now, they can!

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

This works with `add_observer`, entity `.observe()`, and the `Observer` builder pattern.

## Serializing and Deserializing Asset Handles [#](https://bevy.org/news/bevy-0-19/#serializing-and-deserializing-asset-handles)

Authors:[@andriyDev](https://github.com/andriyDev)

PRs:[#23329](https://github.com/bevyengine/bevy/pull/23329)

Asset handles can now be round-tripped successfully during serialization and deserialization. This is particularly important for world assets — the serialization format written through [`DynamicWorld::serialize`], previously called scenes.

This wasn't a matter of just slapping on some derives, because handles aren't raw data: they're a pointer to the actual loaded asset. As a result, there was no clear way to either persist or reconstruct one. The new `HandleSerializeProcessor` and `HandleDeserializeProcessor` solve this by storing a handle's identifying information (its asset path) on serialization, then reloading the asset from that path on deserialization. Pass them to `TypedReflectSerializer::with_processor` and `TypedReflectDeserializer::with_processor` if you need the same behavior in your own serialization pipelines.

### Caveat [#](https://bevy.org/news/bevy-0-19/#caveat)

For this to work, your assets need to be correctly reflected. If your asset looks like:

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

## Self-Referential Relationships [#](https://bevy.org/news/bevy-0-19/#self-referential-relationships)

Authors:[@mrchantey](https://github.com/mrchantey)

PRs:[#22269](https://github.com/bevyengine/bevy/pull/22269)

By default, Bevy rejects relationship components that point to the entity they live on. If you insert one, Bevy will log a warning and remove it. This default exists for good reason: structural relationships like [`ChildOf`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/hierarchy/struct.ChildOf.html) form hierarchies that Bevy traverses recursively — a self-referential [`ChildOf`](https://docs.rs/bevy/0.19.0-rc.3/bevy/ecs/hierarchy/struct.ChildOf.html) would produce an infinite loop.

But many relationships are purely semantic. `Likes(self)`, `EmployedBy(self)`, `Healing(self)` — these don't imply any traversal, and self-reference is perfectly valid. You can now opt in with `allow_self_referential`:

```rust
#[derive(Component)]
#[relationship(relationship_target = PeopleILike, allow_self_referential)]
pub struct LikedBy(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = LikedBy)]
pub struct PeopleILike(Vec<Entity>);
```

With the attribute set, inserting a self-referential relationship is accepted without warning.

## Accessible Label Component [#](https://bevy.org/news/bevy-0-19/#accessible-label-component)

Authors:[@viridia](https://github.com/viridia)

PRs:[#24308](https://github.com/bevyengine/bevy/pull/24308)

The [`AccessibleLabel`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.AccessibleLabel.html) component allows the a11y `label` property to be specified separately from other a11y properties.

In most apps, the `label` property comes from application code rather than library code. However, the design of `accesskit` requires that all a11y properties be stored in a single large data structure contained in the [`AccessibilityNode`](https://docs.rs/bevy/0.19.0-rc.3/bevy/a11y/struct.AccessibilityNode.html) component. This creates a usability conflict with BSN and other methods of spawning complex hierarchies, where composing multiple components is the primary means of behavioral reuse.

By putting the label in its own component, it can be used as a mixin within BSN templates, allowing the label to be added by the widget user rather than the widget author.

Internally, this uses component hooks to sync the [`AccessibilityNode`](https://docs.rs/bevy/0.19.0-rc.3/bevy/a11y/struct.AccessibilityNode.html) properties with the payload of the [`AccessibleLabel`](https://docs.rs/bevy/0.19.0-rc.3/bevy/prelude/struct.AccessibleLabel.html) component, satisfying the needs of `accesskit`.

## What's Next? [#](https://bevy.org/news/bevy-0-19/#what-s-next)

No matter how many features we add, the flock will always demand _more_. Game engines, unfortunately, are never _done_.

Returning by popular demand, let us peer deep into the mists of time, and see what other features Bevy has in flight! Like usual, many of these features are "essential components of a Bevy scene editor", even if they are not "the editor itself". That allows us to ship useful bits and pieces incrementally, and polish them while we put it all together.

- **`.bsn` scene format:** Bet you didn't see that one coming. Actually loading and saving BSN-flavored asset files to disk remains a top priority. `.bsn` is the thing that our fabled editor will actually edit, allowing you to create and compose characters, game objects, and levels.
- **Unified 2D and 3D rendering internals:** Bevy's 2D rendering is plenty fast, but it's started to lag behind our 3D rendering in terms of both performance and features. We're hoping to unify its internal architecture to avoid duplicating work, keeping the high-level `Sprite` API completely untouched.
- **Entity inspector:** Examine trees of entities, inside your game, and inside of the eventual editor. The framework has been prototyped and widgets continue apace. Now it's time to put it all together!
- **Assets-as-entities:** Bevy's asset system is a bespoke ball of code, with its own unique idioms and complex API surface. We're ready to move this into the ECS itself, allowing engine internals and end users to leverage the powerful features they already use everywhere else.
- **WESL shader language:** WGSL is an adequate shader language, but it's missing some important niceties. Bevy has been working together with a cross-project group to extend it, in the form of [WESL](https://github.com/webgpu-tools/wesl-spec). We've [supported WESL for more than a year](https://github.com/bevyengine/bevy/pull/17953), but we're planning to port our existing internal shaders to use WESL, and endorse it as the shader language of choice for Bevy.
- **A much more complete Bevy book:** Wish the Bevy Book was longer? We do too! We've substantially extended it, covering a much wider range of topics in more depth, and are hoping to release what we have soon, during the 0.20 development cycle. Expect a steady stream of new chapters as more of the engine reaches a "stable enough" status.