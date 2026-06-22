# Bevy 0.17

## Posted on September 30, 2025 by Bevy Contributors

![A factory from Exofactory, an in-development factory builder made with Bevy](https://bevy.org/news/bevy-0-17/cover.jpg)

[A factory from Exofactory, an in-development factory builder made with Bevy](https://store.steampowered.com/app/3615720/Exofactory/)

感谢 **278** 位贡献者、**1311** 个 pull request、社区审阅者以及我们[**慷慨的捐赠者**](https://bevy.org/donate)，我们很高兴地宣布 **Bevy 0.17** 已在 [crates.io](https://crates.io/crates/bevy) 上发布！

如果你还不了解，Bevy 是一个基于 Rust 构建的、令人耳目一新的简洁数据驱动游戏引擎。你可以查看我们的[快速入门指南](https://bevy.org/learn/quick-start)来立即试用。它是免费且永远开源的！你可以在 GitHub 上获取完整的[源代码](https://github.com/bevyengine/bevy)。查看 [Bevy Assets](https://bevy.org/assets) 获取社区开发的插件、游戏和学习资源合集。

要将现有的 Bevy App 或 Plugin 更新到 **Bevy 0.17**，请查看我们的 [0.16 到 0.17 迁移指南](https://bevy.org/learn/migration-guides/0-16-to-0-17/)。

自上次发布以来的几个月里，我们添加了_大量_新功能、Bug 修复和易用性改进，以下是一些亮点：

- **Bevy Solari - 光线追踪光照（实验性）：** Bevy 现在有正在开发中的支持，可实现出色的、物理上逼真的实时光照。虽然存在许多限制，但效果确实令人惊叹。
- **改进的观察者/事件系统：** 观察者（Observers）一直非常受欢迎，为用户提供了以极少的样板代码响应事件的灵活方式。我们已经清理了 Observer 和 Event API，使它们更加灵活，并改进了文档！
- **无头 Bevy UI Widget（实验性）：** 一个全新的正在开发中的无头 UI widget 库，提供构建在上面的基础 widget 功能。
- **Bevy Feathers - 工具 Widget（实验性）：** 一套基于我们的无头 widget 构建的、面向工具的专用 widget 集合。我们仍在构建中，但你现在就可以试用了！
- **Rust 热补丁：** 厌倦了原型设计时等待 Rust 重新编译？Bevy 现在集成了 Dioxus 的 `subsecond`，允许你选择热重载 Rust 代码而无需重启程序。目前仅限于 Bevy ECS 系统，且有一些限制。
- **光照纹理：** 你现在可以使用纹理来艺术性地调制光照强度。
- **DLSS：** 在 Nvidia RTX GPU 上，Bevy 现在支持深度学习超级采样（DLSS）进行抗锯齿和上采样。
- **瓦片地图分块渲染：** 一种新的高性能瓦片地图分块渲染方式...这是构建 Bevy 内置瓦片地图系统的第一步。
- **Web 资产：** Bevy 的资产系统现在支持从 `http` 和 `https` URL 加载资产。
- **Reflect 自动注册：** 在反射类型时，你不再需要在应用中手动注册它们。
- **帧时间图表：** 一个新的内置 widget，用于在运行中的 Bevy 应用中调试帧时间。
- **UI 渐变：** Bevy UI 现在支持背景和边框渐变。
- **光线行进大气：** Bevy 的程序化大气现在有一个光线行进模式，提供更准确的光照。
- **虚拟几何 BVH 剔除：** Bevy 的虚拟几何系统由于 BVH 剔除而变得更加快速。

## Bevy Solari：光线追踪光照（实验性）[#](https://bevy.org/news/bevy-0-17/#bevy-solari-raytraced-lighting-experimental)

Authors:[@JMS55](https://github.com/JMS55), [@SparkyPotato](https://github.com/SparkyPotato)

PRs:[#19058](https://github.com/bevyengine/bevy/pull/19058), [#19620](https://github.com/bevyengine/bevy/pull/19620), [#19790](https://github.com/bevyengine/bevy/pull/19790), [#20020](https://github.com/bevyengine/bevy/pull/20020), [#20113](https://github.com/bevyengine/bevy/pull/20113), [#20156](https://github.com/bevyengine/bevy/pull/20156), [#20213](https://github.com/bevyengine/bevy/pull/20213), [#20242](https://github.com/bevyengine/bevy/pull/20242), [#20259](https://github.com/bevyengine/bevy/pull/20259), [#20406](https://github.com/bevyengine/bevy/pull/20406), [#20457](https://github.com/bevyengine/bevy/pull/20457), [#20580](https://github.com/bevyengine/bevy/pull/20580), [#20596](https://github.com/bevyengine/bevy/pull/20596), [#20622](https://github.com/bevyengine/bevy/pull/20622), [#20658](https://github.com/bevyengine/bevy/pull/20658), [#20659](https://github.com/bevyengine/bevy/pull/20659), [#20980](https://github.com/bevyengine/bevy/pull/20980)

通过新的 `bevy_solari` crate，我们正在迈出实现实时光线追踪光照的第一步。

作为背景知识，视频游戏中的光照可以分为两部分：直接光照和间接光照。

直接光照是从光源发出、从一个表面反弹后到达相机的光。间接光照则是从不同表面多次反弹后才到达相机的光。间接光照通常也被称为全局光照。

在 Bevy 中，直接光照来自解析光源组件（`DirectionalLight`、`PointLight`、`SpotLight`）和阴影贴图。间接光照来自硬编码的 `AmbientLight`、烘焙光照组件（`EnvironmentMapLight`、`IrradianceVolume`、`Lightmap`）和屏幕空间计算（`ScreenSpaceAmbientOcclusion`、`ScreenSpaceReflections`、`specular_transmission`、`diffuse_transmission`）。

这些方法的问题在于它们都有很大的缺点：

- 发光网格不会向其他物体投射光线，无论是直接还是间接。
- 阴影贴图渲染非常昂贵且消耗大量内存，因此你只能使用少量投射阴影的光源。在大型场景中获得良好的阴影质量可能很困难。
- 烘焙光照不会在物体和光源移动时实时更新，且需要烘焙时间，减慢游戏制作速度。
- 屏幕空间方法质量较低，且无法捕获屏幕外的几何体和光线。

Bevy Solari 旨在成为 Bevy 的完全替代高端光照解决方案，使用 GPU 加速光线追踪来解决上述所有问题。发光网格会正确投射光和阴影，你可以拥有数百个投射阴影的光源，质量更好，不需要烘焙时间，并且支持_完全_动态的场景！

### 试用 [#](https://bevy.org/news/bevy-0-17/#try-it-out)

虽然 Bevy 0.17 添加了 bevy_solari crate，但它尚未达到生产可用状态。

但是，请随意运行 solari 示例来查看我们取得的进展。你可以尝试两种不同的模式：

1. 使用路径追踪的非实时"参考"模式：`cargo run --release --example solari --features bevy_solari -- --pathtracer`。
2. 使用多种技术组合的实时模式，目前仅支持漫反射材质：`cargo run --release --example solari --features bevy_solari`。

此外，如果你有 NVIDIA GPU，你可以在实时模式下启用 DLSS 光线重建以获得降噪（Bevy Solari 目前不附带替代降噪器）、更低的渲染时间和抗锯齿的组合：`cargo run --release --example solari --features bevy_solari,dlss`。

### 工作原理 [#](https://bevy.org/news/bevy-0-17/#how-it-works)

![noisy direct illumination](https://bevy.org/news/bevy-0-17/noisy_di.jpg)

![noisy global illumination](https://bevy.org/news/bevy-0-17/noisy_gi.jpg)

![composited / denoised / anti-aliased](https://bevy.org/news/bevy-0-17/denoised_full.jpg)

我们当前的实现使用光线追踪的直接和间接光照（也称为全局照明），通过 ReSTIR DI/GI 采样，并使用世界空间辐照度缓存来提高 GI 质量。像所有光线追踪技术一样，这会产生对实时应用来说过于嘈杂的结果。为了解决这个问题，你需要添加降噪步骤，目前通过 DLSS 光线重建处理，尽管我们将来也乐意添加对其他方法的支持。

如果你对所有这些如何工作的技术细节感兴趣：请阅读 [@JMS55 的博客文章](https://jms55.github.io/posts/2025-09-20-solari-bevy-0-17)了解帧分解！

期待 Bevy Solari 在未来版本中的更多工作！

特别感谢 `@Vecvec` 为 wgpu 添加光线追踪支持。

## 事件/观察者重构 [#](https://bevy.org/news/bevy-0-17/#event-observer-overhaul)

Authors:[@cart](https://github.com/cart), [@Jondolf](https://github.com/Jondolf), [@alice-i-cecile](https://github.com/alice-i-cecile), [@hukasu](https://github.com/hukasu), [@oscar-benderstone](https://github.com/oscar-benderstone), [@Zeophlite](https://github.com/Zeophlite), [@gwafotapa](https://github.com/gwafotapa)

PRs:[#20731](https://github.com/bevyengine/bevy/pull/20731), [#19596](https://github.com/bevyengine/bevy/pull/19596), [#19663](https://github.com/bevyengine/bevy/pull/19663), [#19611](https://github.com/bevyengine/bevy/pull/19611), [#19935](https://github.com/bevyengine/bevy/pull/19935), [#20274](https://github.com/bevyengine/bevy/pull/20274)

Bevy 的 Observer API 在几个版本前落地，迅速成为我们最受欢迎的功能之一。在 **Bevy 0.17** 中，我们重新架构并优化了 Event 和 Observer API，使其更清晰、更易用、性能更好。我们计划在不久的将来推出 Bevy 的[下一代 Scene / UI 系统](https://github.com/bevyengine/bevy/pull/20158/)，而观察者是关键部分！我们希望确保它们在 Bevy 下一阶段的开发中处于更好的状态。旧 API 存在一些问题：

1. **概念名称令人困惑且模糊**：事件可以被观察者"观察"，在 `Events` 集合中"缓冲"，或者两者兼有。了解如何产生或消费给定的 [`Event`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Event.html) 需要太多隐含的上下文，而且容易出错。
2. **API 不够"静态"**：因为给定的 [`Event`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Event.html) 类型可以被_任何上下文_使用和产生，我们必须为_每个事件类型_提供对_每个可能 API_ 的访问。这使得 API 容易被误用。不应该能够在没有实体的情况下触发"实体事件"！
3. **API 做了太多工作**：因为事件可以在任何上下文中产生和使用，这意味着它们都经过了每个可能上下文的代码分支。这带来了不必要的开销。还导致了大量不必要的代码生成！

在 **Bevy 0.17** 中，我们在没有根本改变 API 形态的情况下解决了这些问题。迁移通常应该非常直接。

### 重新架构 [#](https://bevy.org/news/bevy-0-17/#the-rearchitecture)

`Event` trait 已经被重新定位/重新聚焦，以增加灵活性，使 API 更静态，并移除专门化的冗余：

```rust
// Old: Bevy 0.16
trait Event {
    // this embedded configuration specific to "propagating entity events" in all events!
    type Traversal: Traversal<Self>;
    const AUTO_PROPAGATE: bool = false;
    fn register_component_id(world: &mut World);
    fn component_id(world: &World) -> Option<ComponentId>;
}

// New: Bevy 0.17
trait Event {
    type Trigger<'a>: Trigger<Self>;
}
```

每个 [`Event`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Event.html) 现在都有一个关联的 [`Trigger`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Trigger.html) 实现。[`Trigger`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Trigger.html) trait 定义了该事件的 `world.trigger()` 行为。[`Trigger`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Trigger.html) 定义了哪些观察者将运行、它们的运行顺序以及传递给它们的数据。

通过在类型系统中表示这一点，我们可以静态地将行为和数据约束到_特定_类型的事件，使得不可能"误用" [`Event`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Event.html)。Bevy 所有现有的"风味"事件都已移植到新的 [`Event`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Event.html) / [`Trigger`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Trigger.html) 系统。

### `Event` trait [#](https://bevy.org/news/bevy-0-17/#the-event-trait)

乍一看，默认的 [`Event`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Event.html) derive 和用法没有太多变化。只是一些更短/更清晰的命名。旧 API 看起来像这样：

```rust
#[derive(Event)]
struct GameOver {
    score: u32,
}

world.add_observer(|trigger: Trigger<GameOver>| {
    info!("Game over! You scored {} points", trigger.score);
});

world.trigger(GameOver { score: 100 });
```

在 **Bevy 0.17** 中，定义观察者只有轻微变化：

```rust
world.add_observer(|game_over: On<GameOver>| {
    info!("Game over! You scored {} points", game_over.score);
});
```

`Trigger` 现在是 `On`。`On` 鼓励开发者将此参数_视为事件本身_。这也反映在新的命名约定中，我们按 `Event` 命名变量（例如 `game_over`）而不是按 `Trigger`（例如 `trigger`）。

但在内部有些不同！[`Event`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Event.html) derive 默认为"无目标"/"全局"，通过将 `Event::Trigger` 设置为 [`GlobalTrigger`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/struct.GlobalTrigger.html)。当它被触发时，只有"无目标"的顶级观察者会运行，并且_没有其他方式_在不同上下文中触发它（例如，具有 [`GlobalTrigger`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/struct.GlobalTrigger.html) 的事件无法定位实体！）。

### `EntityEvent` trait [#](https://bevy.org/news/bevy-0-17/#the-entityevent-trait)

在 Bevy 的早期版本中，_任何_事件都可以选择性地针对实体触发。看起来像这样：

```rust
#[derive(Event)]
struct Click;

world.trigger_targets(Click, entity);
```

在 **Bevy 0.17** 中，如果你希望 [`Event`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Event.html) 针对 [`Entity`](https://docs.rs/bevy/0.17.1/bevy/ecs/entity/struct.Entity.html)（从而触发任何监视该特定实体的观察者），你需要 derive [`EntityEvent`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.EntityEvent.html)：

```rust
#[derive(EntityEvent)]
struct Click {
    entity: Entity,
}

world.trigger(Click { entity });
```

注意 `Click` 现在将目标实体作为 [`Event`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Event.html) 上的一个_字段_，并且它现在使用与其他事件相同的 `world.trigger()` API。`world.trigger_targets` 不复存在...每个事件都使用相同的 API 触发！

```rust
// This observer will run for _all_ Click events targeting any entity
world.add_observer(|mut click: On<Click>| {});

/// This observer will only run for Click events triggered for `some_entity`
world.entity_mut(some_entity).observe(|mut click: On<Click>| {});
```

[`EntityEvent`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.EntityEvent.html) 是一个具有 [`EntityTrigger`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/struct.EntityTrigger.html) 的 [`Event`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Event.html)，除了任何全局观察者外，还会触发实体特定的观察者。

Derive [`EntityEvent`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.EntityEvent.html) 默认会将 `EntityEvent::event_target` 设置为名为 `entity` 的字段。在某些情况下（例如具有多个实体字段的事件），使用更具描述性的名称可能更合理。你可以使用 `#[event_target]` 字段属性设置目标：

```rust
#[derive(EntityEvent)]
struct Attack {
    // 这将触发 `attacker` 观察者
    #[event_target]
    attacker: Entity,
    attack_target: Entity,
}
```

### EntityEvent 传播 [#](https://bevy.org/news/bevy-0-17/#entityevent-propagation)

[`EntityEvent`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.EntityEvent.html) 默认不"传播"（并且默认_静态地_无法访问控制传播的 API）。可以使用 `propagate` 属性启用传播（默认使用 [`ChildOf`](https://docs.rs/bevy/0.17.1/bevy/ecs/hierarchy/struct.ChildOf.html) 关系来"将事件向上冒泡到层级结构"）：

```rust
#[derive(EntityEvent)]
#[entity_event(propagate)]
struct Click {
    entity: Entity
}
```

这将把 [`Event`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Event.html) 的 [`Trigger`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Trigger.html) 设置为 [`PropagateEntityTrigger`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/struct.PropagateEntityTrigger.html)。

这启用了对"传播"功能的访问，如下所示：

```rust
world.add_observer(|mut click: On<Click>| {
    if SOME_CONDITION {
        // 阻止事件"向上冒泡"
        click.propagate(false);
    }
});
```

Bevy 的 `Pointer` 事件一直跟踪"实体事件"的"原始目标"。这很方便！我们已为每个具有 [`PropagateEntityTrigger`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/struct.PropagateEntityTrigger.html) 的 [`EntityEvent`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.EntityEvent.html) 启用了此功能：只需调用 `On::original_event_target`。

### 组件生命周期事件 [#](https://bevy.org/news/bevy-0-17/#component-lifecycle-events)

在过去的版本中，生命周期事件的观察者 API 看起来像这样：

```rust
app.add_observer(|trigger: Trigger<OnAdd, Player>| {
    info!("Added player {}", trigger.entity());
});
```

我们将这些移植到了新系统，并重命名以匹配我们的新命名方案（例如 `OnAdd` 现在是 [`Add`](https://docs.rs/bevy/0.17.1/bevy/ecs/lifecycle/struct.Add.html)）。它们现在看起来像这样：

```rust
app.add_observer(|add: On<Add, Player>| {
    info!("Added player {}", add.entity);
});
```

组件生命周期事件是 [`EntityEvent`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.EntityEvent.html)（因此将目标实体存储为字段）。它们使用 [`EntityComponentsTrigger`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/struct.EntityComponentsTrigger.html)，允许它们针对实体上的特定组件触发。得益于新的 [`Trigger`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Trigger.html) 系统，这也更加高效，因为我们可以在每次执行时传递大型/分配的上下文，而不是克隆它！

### 自定义事件触发器 [#](https://bevy.org/news/bevy-0-17/#custom-event-triggers)

新的 [`Trigger`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Trigger.html) trait 还使开发者能够实现_自己的_专用 [`Event`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Event.html) [`Trigger`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Trigger.html) 逻辑。

[`Event`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Event.html) derive 可以像这样指定自定义 [`Trigger`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Trigger.html)：

```rust
#[derive(Event)]
#[event(trigger = CoolTrigger)
struct Jump;
```

但通常这没有必要...Bevy 内置的默认触发器几乎总是你想要的。

### 事件 vs 消息 [#](https://bevy.org/news/bevy-0-17/#events-vs-messages)

在 Bevy 的早期版本中，[`Event`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Event.html) trait 同时用于"可观察事件"（通过 `Observer` 处理）和"缓冲事件"（通过 `EventReader` 处理）。这在某种程度上是有道理的，因为两个概念都可以被视为"事件"。但它们在功能上也是根本_非常_不同的东西（完整原因请参见 [此 PR 描述](https://github.com/bevyengine/bevy/pull/20731)）。

在 **Bevy 0.17** 中，[`Event`](https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Event.html) 现在_专门_是"被触发"和"被观察"概念的名称/trait。[`Message`](https://docs.rs/bevy/0.17.1/bevy/ecs/message/trait.Message.html) 是"被缓冲"的东西的名称/trait：它通过 [`MessageWriter`](https://docs.rs/bevy/0.17.1/bevy/ecs/message/struct.MessageWriter.html)"写入"，通过 [`MessageReader`](https://docs.rs/bevy/0.17.1/bevy/ecs/message/struct.MessageReader.html)"读取"。

仍然可以通过实现_两个 trait_ 来支持两种上下文，但我们预计这将比只选择一种少得多。

## Bevy Feathers：工具 Widget（实验性）[#](https://bevy.org/news/bevy-0-17/#bevy-feathers-widgets-for-tooling-experimental)

Authors:[@viridia](https://github.com/viridia), [@Atlas16A](https://github.com/Atlas16A), [@ickshonpe](https://github.com/ickshonpe), [@amedoeyes](https://github.com/amedoeyes)

PRs:[#19730](https://github.com/bevyengine/bevy/pull/19730), [#19900](https://github.com/bevyengine/bevy/pull/19900), [#19928](https://github.com/bevyengine/bevy/pull/19928), [#20237](https://github.com/bevyengine/bevy/pull/20237), [#20169](https://github.com/bevyengine/bevy/pull/20169), [#20422](https://github.com/bevyengine/bevy/pull/20422), [#20350](https://github.com/bevyengine/bevy/pull/20350), [#20548](https://github.com/bevyengine/bevy/pull/20548), [#20969](https://github.com/bevyengine/bevy/pull/20969), [#21247](https://github.com/bevyengine/bevy/pull/21247)

![feathers widgets](https://bevy.org/news/bevy-0-17/feathers.jpg)

为了让 Bevy 引擎开发者和第三方工具创作者更容易制作舒适的、视觉上协调的工具，我们很高兴地推出"Feathers"——一套全面的 Bevy UI widget 集合。Feathers 旨在成为 Bevy 的"开发者工具"widget 集合，它将用于构建即将推出的 [Bevy Editor](https://bevy.org/news/bevys-fifth-birthday/#bevy-editor-design)。它具有实用的外观和感觉，以及为编辑器和图形工具定制的功能集。它构建在 Bevy 新的通用"无头"widget 集合 `bevy_ui_widgets`（如下所述）之上。Feathers_可以_在游戏使用，但这不是它的主要用例。

Feathers 目前提供：

- 设计为匹配计划中的 Bevy Editor 外观和感觉的标准 widget。
- 可用于构建自定义编辑器、检查器和实用程序界面的组件，使其感觉与其他 Bevy 工具一致。
- 基本 UI 元素，包括按钮、滑块、复选框、菜单按钮等。
- 用于组织和构建 UI 元素的布局容器。
- 初始（简单/基础）主题支持，确保应用程序之间一致且可配置的视觉样式。这不是"最终的" Bevy UI 主题系统，但它提供了一些基础功能。
- 具有内置屏幕阅读器和辅助技术支持的无障碍功能。
- 在悬停在 widget 上时适当改变的交互式光标行为。
- 适用于触摸屏文本输入的虚拟键盘。

Feathers 仍处于早期开发阶段。它目前隐藏在 `experimental_bevy_feathers` feature flag 后面。Feathers 仍然不完整，可能会以各种方式变化：

- 我们将在 BSN（Bevy 的[下一代 Scene/UI 系统](https://github.com/bevyengine/bevy/pull/20158/)）落地时将 Feathers 移植到 BSN（目标版本为 **Bevy 0.18**）。
- `observe` API 是临时的：我们希望用通用的、[基于关系的解决方案](https://github.com/bevyengine/bevy/issues/17607)来替换它们。
- 我们仍在努力完善一些 UX 问题。
- 缺少一些 widget 和功能。特别是"文本输入"widget [仍在开发中](https://github.com/bevyengine/bevy/issues/20885)。

如果你想尝试为 Bevy 构建工具，请启用它并对 `feathers` 进行试飞！让我们知道你遇到的问题，并欢迎贡献缺失的 widget 和修复 bug。一些人已经开始在上面构建工具，例如 [Rerecast Editor](https://github.com/janhohenheim/rerecast)（一个导航网格编辑器）：

![rerecast editor](https://bevy.org/news/bevy-0-17/rerecast.jpg)

如果你迫不及待地想在你的游戏中使用 `bevy_ui` widget，我们建议将 Feathers 代码复制到你的项目中并开始修改！Feathers 可以作为理解如何在 Bevy UI 中构建和设置 widget 主题的有用基础。它还说明了如何使用我们新的"无头"widget 集合 `bevy_ui_widgets`。

## 实时过滤环境贴图 [#](https://bevy.org/news/bevy-0-17/#realtime-filtered-environment-maps)

Authors:[@mate-h](https://github.com/mate-h)

PRs:[#19076](https://github.com/bevyengine/bevy/pull/19076), [#20529](https://github.com/bevyengine/bevy/pull/20529)

![atmosphere reflections](https://bevy.org/news/bevy-0-17/atmosphere_reflections.jpg)

环境贴图需要经过处理才能支持超出简单天空盒的用途，例如不同粗糙度级别的反射和环境光贡献。这个过程称为过滤，可以提前完成（预过滤），也可以实时完成，但质量会降低。

Bevy 已经支持预过滤，但并非总是可以预过滤：有时你的环境贴图在运行时才可用。通常来自实时反射探针，但你也可能使用程序化天空盒。

现在，Bevy 同时支持两种过滤模式！将 `GeneratedEnvironmentMapLight` 添加到 `Camera` 实体，让你可以将任何环境贴图与 Bevy 的渲染器一起使用，并享受预过滤的所有好处而无需资产处理。

我们确保它能与我们内置的大气着色器配合使用。为此，请将新组件 `AtmosphereEnvironmentMapLight` 添加到相机实体。

这是一个完全动态的每视图效果：不需要预烘焙的环境贴图。但请注意，目前还不支持光探针。

## 无头 Bevy UI Widget（实验性）[#](https://bevy.org/news/bevy-0-17/#headless-bevy-ui-widgets-experimental)

Authors:[@viridia](https://github.com/viridia), [@ickshonpe](https://github.com/ickshonpe), [@alice-i-cecile](https://github.com/alice-i-cecile)

PRs:[#19366](https://github.com/bevyengine/bevy/pull/19366), [#19584](https://github.com/bevyengine/bevy/pull/19584), [#19665](https://github.com/bevyengine/bevy/pull/19665), [#19778](https://github.com/bevyengine/bevy/pull/19778), [#19803](https://github.com/bevyengine/bevy/pull/19803), [#20032](https://github.com/bevyengine/bevy/pull/20032), [#20036](https://github.com/bevyengine/bevy/pull/20036), [#20086](https://github.com/bevyengine/bevy/pull/20086), [#20944](https://github.com/bevyengine/bevy/pull/20944)

Bevy 的 `Button` 和 `Interaction` 组件已经存在很长时间了。不幸的是这些组件有一些缺点，例如它们不使用新的 `bevy_picking` 框架，或者它们实际上只对创建按钮有用，而不适用于其他类型的 widget 如滑块。

在 Web 开发领域，"无头"widget 库如 [headlessui](https://headlessui.com/) 和 [reakit](https://reakit.io/) 已经流行起来。这些提供无样式的标准化 widget，实现所有正确的交互和行为逻辑：事件、状态管理、无障碍性等。无头 widget 提供核心行为，而游戏开发者负责提供 widget 的视觉样式和动画，使其符合游戏的整体风格。

**Bevy 0.17** 引入了一组无头 widget。这些是可以添加到任何 UI 节点以获得类似 widget 行为的组件。标准 widget 集合包括：

- `Button`：在点击时发出激活事件。
- `Slider`：让你在给定范围内编辑 `f32` 值。
- `Scrollbar`：让你滚动 UI 节点的内容。
- `Checkbox`：提供可切换的状态。
- `RadioButton` 和 `RadioGroup`：从一组项目中选择一个项目。

以下是一个重度样式的示例，展示它们可以实现的效果：

虽然这些 widget 现在可以在游戏和应用中使用，但它们仍然是**实验性的**。我们仍在完善一些方面的开发者体验并填补空白。随着我们继续迭代和改进，预计会有破坏性更改！

我们和你一样期待第一方 widget。我们决定现在发布它们以便人们可以试用：你的反馈至关重要！

如果你想尝试我们实验性的无头 widget，请启用 `experimental_bevy_ui_widgets` feature。

### Widget 交互组件 [#](https://bevy.org/news/bevy-0-17/#widget-interaction-components)

标准 widget 使用额外的组件来驱动行为：

- `InteractionDisabled`：一个布尔组件，用于指示组件应被"灰显"且不可交互。
- `Hovered`：一个简单的布尔组件，允许检测 widget 是否被悬停。
- `Checked`：一个布尔组件，存储复选框或单选按钮的选中状态。
- `Pressed`：一个布尔组件，由类按钮 widget 使用。在按钮被按住时为 true。

这些"布尔组件"可以使用 Bevy 内置的组件变更检测来跟踪。

### Widget 事件 [#](https://bevy.org/news/bevy-0-17/#widget-events)

无头 widget 使用 Bevy 的事件/观察者系统，就像任何其他组件一样！

- `Activate`：在 widget 被激活时触发的事件（例如按钮被按下、单选按钮被选中等）。
- `ValueChange`：在值被更改时触发的事件（例如滑块被更改、复选框被勾选等）。

这些事件可以使用观察者来处理：

```rust
commands.spawn((
    Button,
    observe(|activate: On<Activate>| {
        info!("button clicked!");
    })
));
```

## 光照纹理 [#](https://bevy.org/news/bevy-0-17/#light-textures)

Authors:[@robtfm](https://github.com/robtfm)

PRs:[#18031](https://github.com/bevyengine/bevy/pull/18031)

新组件 `PointLightTexture`、`SpotLightTexture` 和 `DirectionalLightTexture` 允许为灯光指定光照纹理，也通常称为"光照饼干"。它们可以调制投射到表面上的光的强度以实现各种艺术效果。请参见 [`light_textures`](https://github.com/bevyengine/bevy/blob/release-0.17.0/examples/3d/light_textures.rs) 示例了解用法。

## 在运行中的应用中热补丁系统 [#](https://bevy.org/news/bevy-0-17/#hot-patching-systems-in-a-running-app)

Authors:[@mockersf](https://github.com/mockersf), [@janhohenheim](https://github.com/janhohenheim)

PRs:[#19309](https://github.com/bevyengine/bevy/pull/19309)

Bevy 现在通过 [subsecond](https://crates.io/crates/subsecond) 和 Dioxus 项目的 [`dx`](https://crates.io/crates/dioxus-cli) 命令行工具支持系统热补丁。

当启用 cargo feature `hotpatching` 时，每个系统现在都可以在执行期间被修改，更改会立即在你的游戏中可见。

`dx` 是 Dioxus CLI。要安装它，请运行 `cargo install dioxus-cli@0.7.0-rc.0 --locked`

然后运行 `BEVY_ASSET_ROOT="." dx serve --hot-patch --features "bevy/hotpatching"` 在你的项目中测试它。你也可以使用 Bevy 的 [`hotpatching_systems.rs`](https://github.com/bevyengine/bevy/blob/release-0.17.0/examples/ecs/hotpatching_systems.rs) 示例来尝试。

这只是第一步。已知限制：

- 仅适用于二进制 crate。Dioxus 计划扩展这方面的支持。
- 不支持 Wasm。Dioxus 支持这个，但 Bevy 端需要一些工作。
- 如果系统参数更改，它不会被热重载。这是我们需要在 Bevy 端解决的问题。
- 它可能对 rust/linker 配置敏感。不过 Dioxus 在这方面已经做得很好了！

我们计划进一步扩展支持，包括使即将推出的 [`bsn!` 宏](https://github.com/bevyengine/bevy/pull/20158/)可热重载（查看[此视频](https://bevy.org/news/bevys-fifth-birthday/#bevy-hot-reloading)了解实际效果！）。

## 深度学习超级采样 (DLSS) [#](https://bevy.org/news/bevy-0-17/#deep-learning-super-sampling-dlss)

Authors:[@JMS55](https://github.com/JMS55), [@cart](https://github.com/cart)

PRs:[#19864](https://github.com/bevyengine/bevy/pull/19864), [#19817](https://github.com/bevyengine/bevy/pull/19817), [#20565](https://github.com/bevyengine/bevy/pull/20565)

拖动此图像进行对比

![No AA](https://bevy.org/news/bevy-0-17/no_aa.jpg)![DLSS](https://bevy.org/news/bevy-0-17/dlss.jpg)

对于拥有 NVIDIA RTX GPU 的用户，Bevy 现在提供另一种抗锯齿形式：DLSS。

通过运行 Bevy 的 anti_aliasing 示例来试用：`cargo run --example anti_aliasing --features dlss --release`（在 [https://github.com/bevyengine/dlss_wgpu](https://github.com/bevyengine/dlss_wgpu) 完成设置后）。

此外，我们将 [https://github.com/bevyengine/dlss_wgpu](https://github.com/bevyengine/dlss_wgpu) 开源为独立 crate，以帮助其他基于 wgpu 的渲染器集成 DLSS。

与 Bevy 内置的 TAA 相比，DLSS：

- 产生更高质量和更稳定的图像
- 除了抗锯齿外还支持上采样，导致更便宜的渲染时间，特别是与 GPU 密集型功能如 Bevy Solari 一起使用时
- 需要 NVIDIA RTX GPU
- 目前需要通过 Windows/Linux 上的 Vulkan 后端运行（不支持 DirectX、macOS、Web 或移动端）

请注意，DLSS 集成在本版本中预计存在一些与某些渲染效果不遵循上采样设置以及透明度或相机曝光可能存在问题相关的 bug。请报告遇到的任何 bug。

其他时间上采样器如 AMD 的 FidelityFX™ Super Resolution (FSR)、Intel 的 Xe Super Sampling XeSS (XeSS) 和 Apple 的 MTLFXTemporalScaler 在本版本中未集成。但它们都使用类似的 API，在未来版本中集成不会是挑战。

目前没有计划支持其他交换链相关功能，如帧插值/外推、延迟减少或动态分辨率缩放。

特别感谢 @cwfitzgerald 帮助处理 [`wgpu`](https://github.com/gfx-rs/wgpu) 后端互操作 API。

## 瓦片地图分块渲染 [#](https://bevy.org/news/bevy-0-17/#tilemap-chunk-rendering)

Authors:[@ConnerPetzold](https://github.com/ConnerPetzold), [@grind086](https://github.com/grind086), [@IceSentry](https://github.com/IceSentry)

PRs:[#18866](https://github.com/bevyengine/bevy/pull/18866)

![tilemap](https://bevy.org/news/bevy-0-17/tilemap.jpg)

[Tilemap Credit: Cup Nooble's Sprout Lands](https://cupnooble.itch.io/sprout-lands-asset-pack)

已添加一种高性能的瓦片地图分块渲染方式作为 Bevy 瓦片地图支持的第一个构建块（未来版本将有更多！）。你可以通过向 `TilemapChunk` 组件提供瓦片集纹理并向 `TilemapChunkTileData` 提供瓦片数据来渲染分块。对于每个瓦片，`TileData` 允许你指定瓦片集中的索引、可见性和颜色着色。

```rust
let chunk_size = UVec2::new(16, 16);
commands.spawn((
    TilemapChunk {
        tileset: assets.load("tileset.png"),
        // the dimensions of the chunk (in tiles)
        chunk_size,
        // the size to render each tile (in pixels) 
        tile_display_size: UVec2::new(32, 32),
    },
    // Fill in random tile data for this chunk using the first five tiles in the set
    TilemapChunkTileData((0..chunk_size.element_product())
        .map(|_| Some(rng.gen_range(0..5)))
        .collect()
    ),
));
```

## `ViewportNode` [#](https://bevy.org/news/bevy-0-17/#viewportnode)

Authors:[@chompaa](https://github.com/chompaa), [@ickshonpe](https://github.com/ickshonpe)

PRs:[#17253](https://github.com/bevyengine/bevy/pull/17253)

Bevy UI 现在有一个 `ViewportNode` 组件，让你可以将相机输出直接渲染到 UI 节点：

```rust
commands.spawn(ViewportNode::new(camera));
```

这里引用的 `camera` 需要其目标为 `RenderTarget::Image`。更多用法详情请参见新的 [`viewport_node`](https://github.com/bevyengine/bevy/blob/release-0.17.0/examples/ui/viewport_node.rs) 示例。

此外，如果启用了 `bevy_ui_picking_backend` feature，你可以使用渲染目标进行"选取"。也就是说，你可以通过 viewport node 使用**任何**选取后端。

## 光线行进大气/太空视角 [#](https://bevy.org/news/bevy-0-17/#raymarched-atmosphere-space-views)

Authors:[@mate-h](https://github.com/mate-h)

PRs:[#20766](https://github.com/bevyengine/bevy/pull/20766)

Bevy 的程序化大气现在支持光线行进渲染路径，解锁了从大气上方观看的准确视角。这意味着 **Bevy 0.17** 现在有两种大气渲染模式可供选择：

- [`AtmosphereMode::Raymarched`](https://docs.rs/bevy/0.17.1/bevy/pbr/enum.AtmosphereMode.html#variant.Raymarched)
    - 适合电影镜头、从太空看到的行星和"飞行模拟器"类场景
    - 更准确的光照，但更慢
    - 通过大气更锐利的阴影
- [`AtmosphereMode::LookupTexture`](https://docs.rs/bevy/0.17.1/bevy/pbr/enum.AtmosphereMode.html#variant.LookupTexture)
    - 这是默认模式
    - 非常适合地面级别和广阔的户外场景
    - 在远距离光照不太准确，但更快
    - 通过大气更柔和的阴影

要使用它，将 [`Atmosphere`](https://docs.rs/bevy/0.17.1/bevy/pbr/struct.Atmosphere.html) 组件添加到你的 [`Camera`](https://docs.rs/bevy/0.17.1/bevy/camera/struct.Camera.html) 并在相机的 [`AtmosphereSettings`](https://docs.rs/bevy/0.17.1/bevy/pbr/struct.AtmosphereSettings.html) 上设置渲染方法：

```rust
commands.spawn((
    Camera3d::default(),
    Atmosphere::default(),
    AtmosphereSettings { 
      rendering_method: AtmosphereMode::Raymarched, 
      ..default() 
    }
));
```

你还可以调整 `AtmosphereSettings::sky_max_samples` 来配置光线行进大气时的最大步数。较低的数字更快但不太准确。较高的数字更慢但更准确。

请参见更新后的 [`atmosphere` 示例](https://github.com/bevyengine/bevy/blob/release-0.17.0/examples/3d/atmosphere.rs)获取可工作的参考。

## 程序化太阳盘 [#](https://bevy.org/news/bevy-0-17/#procedural-sun-disk)

Authors:[@defuz](https://github.com/defuz)

PRs:[#20434](https://github.com/bevyengine/bevy/pull/20434)

任何优秀的[程序化大气](https://bevy.org/news/bevy-0-16/#procedural-atmospheric-scattering)都值得一个程序化太阳来照亮它。为此，请将 [`SunDisk`](https://docs.rs/bevy/0.17.1/bevy/light/struct.SunDisk.html) 组件添加到你的 [`DirectionalLight`](https://docs.rs/bevy/0.17.1/bevy/light/struct.DirectionalLight.html) 实体。太阳将随你的光源移动，与你实现的任何定位或移动逻辑良好配合。

你可以设置太阳盘的 `angular_size` 和 `intensity`，改变太阳的大小和亮度。我们包含了一个方便的 `SunDisk::EARTH` 常量，省去你进行棘手的实验性三角学计算。

如果你曾经在现实生活中直视太阳（不推荐），你也会熟悉一种扩散的光晕效果，蔓延到附近的天空。这种效果被称为"泛光"（bloom），通过将 [`Bloom`](https://docs.rs/bevy/0.17.1/bevy/post_process/bloom/struct.Bloom.html) 组件添加到你的相机实体来启用。

## Web 资产 [#](https://bevy.org/news/bevy-0-17/#web-assets)

Authors:[@johanhelsing](https://github.com/johanhelsing), [@mrchantey](https://github.com/mrchantey), [@jf908](https://github.com/jf908), [@atlv24](https://github.com/atlv24)

PRs:[#20628](https://github.com/bevyengine/bevy/pull/20628)

Bevy 现在支持通过 http 和 https 从网络下载资产。使用新的 `http` 和 `https` feature 来启用 `http://` 和 `https://` URL 作为资产路径。此功能在原生平台上由 [`ureq`](https://github.com/algesten/ureq) crate 驱动，在 wasm 上由 fetch API 驱动。

```rust
let image = asset_server.load("https://example.com/image.png");
commands.spawn(Sprite::from_image(image));
```

安全提示：如果使用 web 资产，请注意 URL 的来源！如果你允许任意 URL 进入资产服务器，攻击者可能利用它来触发资产加载器中的漏洞，或者通过下载巨大的文件进行 DoS 攻击。我们目前没有发现此类漏洞，但请小心！

默认情况下这些资产不会保存在任何地方，但你可以启用 `web_asset_cache` feature 来在文件系统上缓存资产。

实现已经发生了很大变化，但此功能最初源于 [`bevy_web_asset`](https://github.com/johanhelsing/bevy_web_asset) crate 的上游化。特别感谢 @johanhelsing 和 bevy_web_asset 的贡献者！

## Reflect 自动注册 [#](https://bevy.org/news/bevy-0-17/#reflect-auto-registration)

Authors:[@eugineerd](https://github.com/eugineerd)

PRs:[#15030](https://github.com/bevyengine/bevy/pull/15030)

Derive [`Reflect`](https://docs.rs/bevy/0.17.1/bevy/prelude/trait.Reflect.html) 会将类型加入 **Bevy** 的运行时反射基础设施，用于驱动运行时组件检查和序列化等系统：

```rust
#[derive(Reflect)]
pub struct Foo {
  a: usize,
}
```

在以前的 Bevy 版本中，任何派生了 [`Reflect`](https://docs.rs/bevy/0.17.1/bevy/prelude/trait.Reflect.html) 的顶层类型都必须使用 [`register_type`](https://docs.rs/bevy/0.17.1/bevy/prelude/struct.App.html#method.register_type) 手动注册：

```rust
// 这将使 Foo 对 Bevy 可见
app.register_type::<Foo>()
```

在 **Bevy 0.17** 中，所有 `#[derive(Reflect)]` 的类型现在会自动注册！这大大减少了使用 Bevy 反射功能所需的样板代码，随着我们构建 Bevy 的新场景系统、实体检查器和可视化编辑器，这将变得越来越重要。

请注意，泛型类型仍需要手动注册，因为这些类型在 [`Reflect`](https://docs.rs/bevy/0.17.1/bevy/prelude/trait.Reflect.html) 被 derive 时（还）不存在：

```rust
app.register_type::<Container<Item>>()
```

在不需要自动注册的情况下，可以通过向类型添加 `#[reflect(no_auto_register)]` 属性来选择退出。

### 支持不支持的平台 [#](https://bevy.org/news/bevy-0-17/#supporting-unsupported-platforms)

此功能依赖 [`inventory`](https://github.com/dtolnay/inventory) crate 在编译时收集所有类型注册。这在 Bevy 最流行的平台上得到支持：Windows、macOS、Linux、iOS、Android 和 WebAssembly。然而，一些小众平台不被 [`inventory`](https://github.com/dtolnay/inventory) 支持，虽然最好能在上游支持任何不支持的平台，但有时可能无法做到。因此，有[不同的实现](https://github.com/bevyengine/bevy/tree/release-0.17.0/examples/reflection/auto_register_static)可以在所有平台上使用。

## 虚拟几何 BVH 剔除 [#](https://bevy.org/news/bevy-0-17/#virtual-geometry-bvh-culling)

Authors:[@SparkyPotato](https://github.com/SparkyPotato), [@atlv24](https://github.com/atlv24)

PRs:[#19318](https://github.com/bevyengine/bevy/pull/19318)

![lots of dragons being rendered](https://bevy.org/news/bevy-0-17/mesh_bvh.jpg)

Bevy 的[虚拟几何](https://bevy.org/news/bevy-0-14/#virtual-geometry-experimental)已经通过基于 BVH 的剔除得到了大幅优化，使渲染成本几乎与场景几何体无关。

这些更改还取消了之前限制世界为 2^24 个簇（约 40 亿三角形）的集群限制。现在对场景大小_没有_硬编码限制。实际上你将仅受资产 VRAM 使用量（因为资产流式加载尚未实现）和总实例数量（同样由于我们正在改进的临时限制）的限制。

上方截图中有 130,000 条龙在场景中，每条约 870,000 个三角形，导致场景中总计超过 _1150 亿_个三角形。

具体到 GPU 成本，上述场景在 4070 上约 3.5 ms 渲染，其中约 3.1 ms 花在几何渲染上，约 0.4 ms 花在材质评估上。当实例数量增加到超过 100 万（几乎 _9000 亿个三角形_！）时，总计增加到约 4.5 ms，其中约 4.1 ms 在几何渲染上，材质评估保持在约 0.4 ms 不变。这是 GPU 时间增加 30%，而场景复杂度几乎增加了 8 倍。

与 **Bevy 0.16** 在一个只有 1,300 个实例的更小场景上的 GPU 时间相比，之前完整的渲染需要 2.2 ms，而现在在 **Bevy 0.17** 中只需 1.3 ms。

## 帧时间图表 [#](https://bevy.org/news/bevy-0-17/#frame-time-graph)

Authors:[@IceSentry](https://github.com/IceSentry), [@Zeophlite](https://github.com/Zeophlite)

PRs:[#12561](https://github.com/bevyengine/bevy/pull/12561), [#19277](https://github.com/bevyengine/bevy/pull/19277)

在衡量游戏性能时，仅看到一个数字通常不够。看到显示历史记录的图表更容易理解性能。**Bevy 0.17** 引入了新的可视化"帧时间图表"来解决这个问题！

要显示帧时间图表，启用 `bevy_dev_tools` cargo feature 并添加 `FpsOverlayPlugin`：

这显示的是"帧时间"而非"每秒帧数"，因此更长的帧时间会导致更大更宽的条形。颜色也随帧时间变化。红色在或低于最小目标 fps，绿色在或高于目标最大帧率。介于这两个值之间的任何内容将根据帧时间在绿色和红色之间插值。

该算法深受 [Adam Sawicki 关于帧时间可视化的文章](https://asawicki.info/news_1758_an_idea_for_visualization_of_frame_times) 启发。

## `Text2d` 阴影 [#](https://bevy.org/news/bevy-0-17/#text2d-drop-shadows)

Authors:[@ickshonpe](https://github.com/ickshonpe)

PRs:[#20463](https://github.com/bevyengine/bevy/pull/20463)

![text2d shadow](https://bevy.org/news/bevy-0-17/text2d_shadow.jpg)

`Text2d` 是一个简单的世界空间文本 API：非常适合伤害数字和简单标签。与它的 UI 兄弟 `Text` 不同，它不支持阴影，所以在 **Bevy 0.17** 中我们为 `Text2d` 添加了阴影支持。将 `Text2dShadow` 组件添加到 `Text2d` 实体即可在其文本下方绘制阴影效果。

## 文本背景颜色 [#](https://bevy.org/news/bevy-0-17/#text-background-colors)

Authors:[@ickshonpe](https://github.com/ickshonpe)

PRs:[#18892](https://github.com/bevyengine/bevy/pull/18892), [#20464](https://github.com/bevyengine/bevy/pull/20464)

![text2d background](https://bevy.org/news/bevy-0-17/text2d_background.jpg)

Bevy 中的文本现在支持背景颜色。在 UI `Text` 或 `TextSpan` 实体上插入 `TextBackgroundColor` 组件即可为其文本段落设置背景颜色。`TextBackgroundColor` 提供了设置_每个_"文本段"颜色的能力，而标准的 `BackgroundColor` 应用于 `Text` 节点中的_所有_段，还包括填充所占的空间。

`TextBackgroundColor` 也适用于 `Text2d`：非常适合世界空间工具提示！

## UI 渐变 [#](https://bevy.org/news/bevy-0-17/#ui-gradients)

Authors:[@Ickshonpe](https://github.com/Ickshonpe)

PRs:[#18139](https://github.com/bevyengine/bevy/pull/18139), [#19330](https://github.com/bevyengine/bevy/pull/19330), [#19992](https://github.com/bevyengine/bevy/pull/19992)

![ui gradients](https://bevy.org/news/bevy-0-17/ui_gradients.jpg)

Bevy 现在支持显示在两种或多种颜色之间平滑过渡的渐变的 UI 节点。

你现在可以将 `BackgroundGradient` 组件添加到 `Node` 以将其背景设置为渐变。如果你还设置了 `BackgroundColor`，则先绘制背景颜色，然后在其上绘制渐变。你还可以使用 `BorderGradient` 组件使边框使用渐变。

这两个组件都包装了 `Gradient` 枚举类型，它有三个变体：`Linear`、`Conic` 和 `Radial`。

每种渐变类型由该渐变的几何属性、颜色停止列表和用于插值的颜色空间组成（Bevy 默认使用 `InterpolationColorSpace::Oklab`）。

```rust
commands.spawn((
    Node { width: px(20), height: px(20) },
    BackgroundGradient::from(LinearGradient {
        angle: 4.,
        stops: vec![
            ColorStop::new(Color::WHITE, percent(15)),
            ColorStop::new(Color::BLACK, percent(85)),
        ],
        ..default()
    })
))
```

## 每边 UI 边框颜色 [#](https://bevy.org/news/bevy-0-17/#per-side-ui-border-colors)

Authors:[@robtfm](https://github.com/robtfm)

PRs:[#18682](https://github.com/bevyengine/bevy/pull/18682)

![ui border colors](https://bevy.org/news/bevy-0-17/ui_border_colors.jpg)

`bevy_ui` 现在支持 UI 节点每边使用不同的边框颜色，通过 [`BorderColor`](https://docs.rs/bevy/0.17.1/bevy/prelude/struct.BorderColor.html) 组件控制。此功能借鉴自 CSS，通常用于模拟具有深度的按钮，但我们期待看到你的创意设计。

## 专用 UI Transform [#](https://bevy.org/news/bevy-0-17/#specialized-ui-transform)

Authors:[@Ickshonpe](https://github.com/Ickshonpe)

PRs:[#16615](https://github.com/bevyengine/bevy/pull/16615)

在 Bevy UI 中，[`Transform`](https://docs.rs/bevy/0.17.1/bevy/prelude/struct.Transform.html) 和 `GlobalTransform` 已被 [`UiTransform`](https://docs.rs/bevy/0.17.1/bevy/ui/ui_transform/struct.UiTransform.html) 和 `UiGlobalTransform` 替换。[`UiTransform`](https://docs.rs/bevy/0.17.1/bevy/ui/ui_transform/struct.UiTransform.html) 是一个专门化的 2D UI transform，更有效地映射到 UI 空间，大幅改进了我们的内部实现，并消除了冗余的、不必要的、通常昂贵的工作（例如在 Bevy UI 布局算法_之外_还进行完整的层级 [`Transform`](https://docs.rs/bevy/0.17.1/bevy/prelude/struct.Transform.html) 传播）。

## 数据驱动材质 [#](https://bevy.org/news/bevy-0-17/#data-driven-materials)

Authors:[@tychedelia](https://github.com/tychedelia)

PRs:[#19667](https://github.com/bevyengine/bevy/pull/19667)

Bevy 的材质系统历史上依赖 `Material` 和 `AsBindGroup` trait 来提供一种类型安全的方式来定义传递给渲染材质的着色器的数据。虽然这种方法有许多优势，但渲染器的近期改进，如 Bevy `0.16` 中的 GPU 驱动渲染，使得 3D 渲染器更加孤立和不够模块化。此外，`Material` 和 `Material2d` 之间的类型级分离意味着为 3D 实现的每个功能都需要一个大部分复制粘贴的 2D 版本，导致 2D 渲染器在功能方面落后。

在 Bevy `0.17` 中，我们已开始将渲染器的中层和底层 API 重构为_数据驱动_的。更具体地说，我们从渲染世界中的每个渲染系统中移除了 `M: Material` 约束。渲染器现在不再通过类型静态描述材质，而是理解可以运行时修改的纯数据形式的材质。因此，现在可以实现不依赖 `Material` trait 的自定义材质，例如在新的[手动材质示例](https://github.com/bevyengine/bevy/blob/8b36cca28c4ea00425e1414fd88c8b82297e2b96/examples/3d/manual_material.rs)中。虽然这个 API 还不完全符合人体工程学，但它代表了将渲染器从特定的高级材质 API 解耦的第一步。

重要的是，对于使用 `Material` trait 的用户，没有任何变化。我们的 `AsBindGroup` 驱动的 API 现在只是渲染器的一种可能消费者。但采用更动态的、数据优先的方法为渲染器创造了我们希望在 `0.18` 及以后探索的许多机会，包括：

- 统一 2D 和 3D 渲染实现。虽然我们将继续提供有利于构建 2D 游戏用户的有主见的 2D API，但我们希望 3D 渲染器的每个新渲染改进都至少可能对 2D 用户可用。
- 为未来的材质编辑器探索新的材质表示。虽然类型安全对于编写代码很棒，但它为能够在 UI 中动态编辑材质（如着色器图）或从序列化格式在运行时加载材质带来了真正的问题。
- 将更多中层渲染 API 模块化，允许编写高级渲染代码的用户访问渲染基础设施的复杂部分，如网格和绑定组分配、GPU 预处理、保留渲染缓存和自定义绘制函数。

有了这个基础，我们正在积极发展渲染器以拥抱定义 Bevy ECS 的灵活性和可组合性。如果你想帮助我们探索 ECS 驱动渲染的可能性，请加入我们的 [Discord](https://discord.gg/bevy) 或 [GitHub Discussions](https://github.com/bevyengine/bevy/discussions)！

## 实体生成 Tick [#](https://bevy.org/news/bevy-0-17/#entity-spawn-ticks)

Authors:[@urben1680](https://github.com/urben1680), [@specificprotagonist](https://github.com/specificprotagonist)

PRs:[#19047](https://github.com/bevyengine/bevy/pull/19047), [#19350](https://github.com/bevyengine/bevy/pull/19350)

在 Bevy 的早期版本中，跟踪哪些实体自上次系统运行以来被生成只能通过编写自己的逻辑间接完成。

新的 `SpawnDetails` 查询数据和 `Spawned` 查询过滤器使你无需任何标记组件即可找到最近生成的实体。

### `SpawnDetails` [#](https://bevy.org/news/bevy-0-17/#spawndetails)

当你想要获取实体的生成信息时在查询中使用：

```rs
fn print_spawn_details(query: Query<(Entity, SpawnDetails)>) {
    for (entity, spawn_details) in &query {
        if spawn_details.is_spawned() {
            print!(
                "new entity {entity:?} spawned at {:?}",
                spawn_details.spawn_tick()
            );
            // if the `track_location` cargo feature is activated, this contains the source
            // code location where this entity was spawned. This has a runtime cost, so only
            // use it for debugging!
            match spawn_details.spawned_by().into_option() {
                Some(location) => println!(" by {location:?}"),
                None => println!()
            }    
        }
    }
}
```

### `Spawned` [#](https://bevy.org/news/bevy-0-17/#spawned)

如果你只对在上次系统运行后生成的实体感兴趣，请在查询中使用此过滤器：

```rust
fn system(query: Query<Entity, Spawned>) {
    for entity in &query { /* entity spawned */ }
}
```

请注意，与 `Added` 和 `Changed` 过滤器一样，这是一个"非原型过滤器"，意味着它需要扫描每个匹配查询的实体，包括那些自上次运行以来未被生成的实体。因此，上述系统的性能大致与此系统相同：

```rust
fn system(query: Query<(Entity, SpawnDetails)>) {
    for (entity, spawned) in &query {
        if spawned.is_spawned() { /* entity spawned */ }
    }
}
```

### Getter 方法 [#](https://bevy.org/news/bevy-0-17/#getter-methods)

你还可以在 `EntityWorldMut` 和 `EntityCommands` 上使用辅助方法：

```rust
world.entity(entity).spawn_tick()
```

## Key 的 ButtonInput [#](https://bevy.org/news/bevy-0-17/#buttoninput-for-key)

Authors:[@kristoff3r](https://github.com/kristoff3r)

PRs:[#19684](https://github.com/bevyengine/bevy/pull/19684)

Bevy 现在有一个 `ButtonInput<Key>` 资源，类似于现有的 `ButtonInput<KeyCode>` 资源。

`KeyCode` 和 `Key` 之间的区别在于前者指的是 US 键盘上按键的位置，与实际使用的布局无关，而 `Key` 给出你输入的实际字母或符号。在大多数情况下你仍然想要使用 `KeyCode`，但在某些情况下使用 `Key` 更合理，例如使用 '+'/'-' 进行缩放时。

## `Val` 辅助函数 [#](https://bevy.org/news/bevy-0-17/#val-helper-functions)

Authors:[@Ickshonpe](https://github.com/Ickshonpe), [@TheBlckbird](https://github.com/TheBlckbird)

PRs:[#20518](https://github.com/bevyengine/bevy/pull/20518), [#20551](https://github.com/bevyengine/bevy/pull/20551), [#20937](https://github.com/bevyengine/bevy/pull/20937)

为了使 `Val` 更容易构造，添加了以下辅助函数：`px`、`percent`、`vw`、`vh`、`vmin` 和 `vmax`：

```rust
// Using Val::Px directly:
Node {
    width: Val::Px(200.),
    ..default()
}
// Using the px() helper:
Node {
    width: px(200),
    ..default()
}
```

每个函数接受任何整数类型并返回由其对应 `Val` 变体包装的值。还有一个 `auto` 辅助函数映射到 `Val::Auto`。

此版本还包括一个用于从 `Val` 构造 `UiRect` 的流畅接口：

```rust
Node {
    border: px(2).all(), 
    padding: percent(20).horizontal().with_top(px(10.)),
    margin: vw(10).left(),
    ..default()
}
```

可用函数有 `left`、`right`、`top`、`bottom`、`all`、`horizontal` 和 `vertical`。每个函数在 `self` 上调用相应的 `UiRect` 构造函数，即 `fn left(self) -> UiRect { UiRect::left(self) }`。

## glTF Forward 语义配置 [#](https://bevy.org/news/bevy-0-17/#gltf-forward-semantics-configuration)

Authors:[@janhohenheim](https://github.com/janhohenheim)

PRs:[#19633](https://github.com/bevyengine/bevy/pull/19633), [#19685](https://github.com/bevyengine/bevy/pull/19685), [#19816](https://github.com/bevyengine/bevy/pull/19816), [#20131](https://github.com/bevyengine/bevy/pull/20131), [#20122](https://github.com/bevyengine/bevy/pull/20122)

_注意：这是一个具有[已知问题](https://github.com/bevyengine/bevy/issues/20621)的实验性功能。行为可能在未来的版本中发生变化。_

Bevy 对所有具有 `Transform` 的世界空间实体使用以下坐标系：

- forward: -Z
- up: Y
- right: X

但 glTF 更复杂一些。glTF 场景中的模型使用以下坐标系：

- forward: Z
- up: Y
- right: -X

但 glTF 场景中的相机和灯光使用以下坐标系：

- forward: -Z
- up: Y
- right: X

如你所见，这与 Bevy 假设世界中所有事物使用相同坐标系相冲突。过去，我们只使用相机/灯光坐标系导入所有 glTF，因为它已经与 Bevy 对齐。换句话说，glTF 导入器简单地假设 glTF 模型使用 -Z 作为其 forward 方向，即使它们使用的是 +Z。

但那意味着在 Bevy 端，glTF 模型的 `Transform::forward()` 实际上从模型的角度来看是指向后面的，这违反直觉且在跨不同美术管道工作时非常烦人。

为了解决这个问题，用户现在可以更改导入行为以优先考虑模型的正确 `Transform::forward()` 语义。缺点是在 glTF 中具有全局身份变换的 glTF 相机和灯光现在在 Bevy 中将看向 +Z 而不是 -Z。这在许多情况下应该不是问题，因为整个场景被旋转，所以屏幕上的最终结果将以完全相同的方式渲染。

要全局选择有利于 glTF 模型而非 glTF 相机的行为，你可以设置 `GltfPlugin::use_model_forward_direction`：

```rust
App::new()
    .add_plugins(DefaultPlugins.set(GltfPlugin {
        use_model_forward_direction: true,
        ..default()
    }))
    .run();
```

你也可以在每个资产级别控制此设置：

```rust
let handle = asset_server.load_with_settings(
    "fox.gltf#Scene0",
    |settings: &mut GltfLoaderSettings| {
        settings.use_model_forward_direction = Some(true);
    },
);
```

将以上设置为 `None` 将回退到从 `GltfPlugin::use_model_forward_direction` 获取的全局设置。

## `RenderStartup` 调度 [#](https://bevy.org/news/bevy-0-17/#renderstartup-schedule)

Authors:[@IceSentry](https://github.com/IceSentry), [@andriyDev](https://github.com/andriyDev)

PRs:[#19841](https://github.com/bevyengine/bevy/pull/19841), [#19885](https://github.com/bevyengine/bevy/pull/19885), [#19886](https://github.com/bevyengine/bevy/pull/19886), [#19897](https://github.com/bevyengine/bevy/pull/19897), [#19898](https://github.com/bevyengine/bevy/pull/19898), [#19901](https://github.com/bevyengine/bevy/pull/19901), [#19912](https://github.com/bevyengine/bevy/pull/19912), [#19926](https://github.com/bevyengine/bevy/pull/19926), [#19999](https://github.com/bevyengine/bevy/pull/19999), [#20002](https://github.com/bevyengine/bevy/pull/20002), [#20024](https://github.com/bevyengine/bevy/pull/20024), [#20124](https://github.com/bevyengine/bevy/pull/20124), [#20147](https://github.com/bevyengine/bevy/pull/20147), [#20184](https://github.com/bevyengine/bevy/pull/20184), [#20194](https://github.com/bevyengine/bevy/pull/20194), [#20195](https://github.com/bevyengine/bevy/pull/20195), [#20208](https://github.com/bevyengine/bevy/pull/20208), [#20209](https://github.com/bevyengine/bevy/pull/20209), [#20210](https://github.com/bevyengine/bevy/pull/20210)

在 Bevy 的早期版本中，渲染 `Plugin` 代码必须与其他 `Plugin` 代码有所不同，因为渲染器的初始化方式。通常渲染器资源和系统必须在 `Plugin::finish` 中添加，与典型位置 `Plugin::build` 分开。`Plugin::finish` 产生正确顺序的事实有些任意/偶然。

作为解决此问题的一步，**Bevy 0.17** 引入了 `RenderStartup` 调度并将许多渲染器资源移植到在 `RenderStartup` 中使用系统初始化。这使渲染器初始化更加结构化，允许渲染器插件初始化"正常地"在 `Plugin::build` 中定义。它还允许渲染器初始化代码受益于 Bevy ECS 调度器，包括自动并行化和系统排序。

在以前的版本中，初始化渲染器资源看起来像这样：

```rust
impl Plugin for MyRenderingPlugin {
    fn build(&self, app: &mut App) {
        // Do nothing??
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<MyRenderResource>();
    }
}

#[derive(Resource)]
pub struct MyRenderResource(/* ... */);

impl FromWorld for MyRenderResource {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        MyRenderResource(/* ... */)
    }
}
```

在 **Bevy 0.17** 中，现在可以这样写：

```rust
impl Plugin for MyRenderingPlugin {
    fn build(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(RenderStartup, init_my_resource);
    }
}

#[derive(Resource)]
pub struct MyRenderResource(/* ... */);

fn init_my_resource(mut commands: Commands, render_device: Res<RenderDevice>) {
    commands.insert_resource(MyRenderResource(/* ... */));
}
```

我们强烈鼓励渲染器开发者将自己的渲染资源移植到这种新方式！

## 组件传播 [#](https://bevy.org/news/bevy-0-17/#component-propagation)

Authors:[@robtfm](https://github.com/robtfm)

PRs:[#17575](https://github.com/bevyengine/bevy/pull/17575)

在处理大型游戏对象层级结构时，协调整个树的状态可能令人沮丧。Bevy 在内部处理变换和可见性时使用这种模式，但用户每次想要使用类似模式时都不得不重新发明轮子。

虽然这种痛苦在处理 [`RenderLayers`](https://docs.rs/bevy/0.17.1/bevy/camera/visibility/struct.RenderLayers.html) 时最为严重，但这种模式更广泛地有用，并且已经以 [`HierarchyPropagatePlugin`](https://docs.rs/bevy/0.17.1/bevy/app/struct.HierarchyPropagatePlugin.html) 的形式暴露给最终用户。一些示例用例：

- 同步预览建筑"幽灵"版本的颜色和 alpha 值
- 确保模型的所有部分都在相同的渲染层上
- 传播字体样式

此插件有三个泛型：

- `C: Component`：应传播的组件类型
- `F: QueryFilter = ()`：如果设置，只有匹配此过滤器的实体会受到传播影响
- `R: Relationship = ChildOf`：要向下传播的树状关系类型

此插件的每个副本将沿所有匹配类型 `F` 的查询过滤器的实体向下传播类型 `C` 的组件。通过为此插件启用 `C`，你可以添加 [`Propagate<C>`](https://docs.rs/bevy/0.17.1/bevy/app/struct.Propagate.html) 组件来向所有子级添加新组件，添加 [`PropagateStop<C>`](https://docs.rs/bevy/0.17.1/bevy/app/struct.PropagateStop.html) 组件来停止传播，或甚至使用 [`PropagateOver<C>`](https://docs.rs/bevy/0.17.1/bevy/app/struct.PropagateOver.html) 在传播期间跳过此实体。

这是一个非常通用的工具：请让我们知道你用它做什么，我们可以继续在文档中添加示例！

## 无限子级 [#](https://bevy.org/news/bevy-0-17/#infinite-children)

Authors:[@CorvusPrudens](https://github.com/CorvusPrudens)

PRs:[#18865](https://github.com/bevyengine/bevy/pull/18865)

`children!` 宏是在 Bevy 代码中与父级一起生成子级的便捷方式。在 **Bevy 0.16** 中引入时，由于任意限制（Rust：请[支持可变泛型！](https://blog.rust-lang.org/inside-rust/2025/09/11/program-management-update-2025-08/#variadic-generics)），且未实现必要的变通方法，这限制为 12 个子级。在处理大型 UI 层级结构时，这可能是一个真正的麻烦，迫使用户诉诸于丑陋的变通方法。

我们重写了宏并取消了这个不公正的限制。你现在只受 Rust 递归限制的约束：一次约 1400 个子级。欢呼吧！如果你正在手动在一个宏调用中生成超过 1400 个子级，你应该重新考虑你的策略（例如使用 `SpawnIter` 或 `SpawnWith`）。

我们对 `related!` 宏进行了相同的更改，允许你在一次调用中生成大量关联实体。

## 将 Bevy 的公共 API 与 Bevy Render 解耦 [#](https://bevy.org/news/bevy-0-17/#decoupling-bevy-s-public-api-from-bevy-render)

Authors:[@atlv24](https://github.com/atlv24), [@Ickshonpe](https://github.com/Ickshonpe), [@zeophlite](https://github.com/zeophlite)

PRs:[#20485](https://github.com/bevyengine/bevy/pull/20485), [#20330](https://github.com/bevyengine/bevy/pull/20330), [#18703](https://github.com/bevyengine/bevy/pull/18703), [#20587](https://github.com/bevyengine/bevy/pull/20587), [#20502](https://github.com/bevyengine/bevy/pull/20502), [#19997](https://github.com/bevyengine/bevy/pull/19997), [#19991](https://github.com/bevyengine/bevy/pull/19991), [#20000](https://github.com/bevyengine/bevy/pull/20000), [#19949](https://github.com/bevyengine/bevy/pull/19949), [#19943](https://github.com/bevyengine/bevy/pull/19943), [#19953](https://github.com/bevyengine/bevy/pull/19953), [#20498](https://github.com/bevyengine/bevy/pull/20498), [#20496](https://github.com/bevyengine/bevy/pull/20496), [#20493](https://github.com/bevyengine/bevy/pull/20493), [#20492](https://github.com/bevyengine/bevy/pull/20492), [#20491](https://github.com/bevyengine/bevy/pull/20491), [#20488](https://github.com/bevyengine/bevy/pull/20488), [#20487](https://github.com/bevyengine/bevy/pull/20487), [#20486](https://github.com/bevyengine/bevy/pull/20486), [#20483](https://github.com/bevyengine/bevy/pull/20483), [#20480](https://github.com/bevyengine/bevy/pull/20480), [#20479](https://github.com/bevyengine/bevy/pull/20479), [#20478](https://github.com/bevyengine/bevy/pull/20478), [#20477](https://github.com/bevyengine/bevy/pull/20477), [#20473](https://github.com/bevyengine/bevy/pull/20473), [#20472](https://github.com/bevyengine/bevy/pull/20472), [#20471](https://github.com/bevyengine/bevy/pull/20471), [#20470](https://github.com/bevyengine/bevy/pull/20470), [#20392](https://github.com/bevyengine/bevy/pull/20392), [#20390](https://github.com/bevyengine/bevy/pull/20390), [#20388](https://github.com/bevyengine/bevy/pull/20388), [#20345](https://github.com/bevyengine/bevy/pull/20345), [#20344](https://github.com/bevyengine/bevy/pull/20344), [#20051](https://github.com/bevyengine/bevy/pull/20051), [#19985](https://github.com/bevyengine/bevy/pull/19985), [... (line truncated to 2000 chars)

在 **Bevy 0.17** 中，我们已将大部分面向用户的渲染器 API 从 `bevy_render`（Bevy 的默认内置渲染器，使用 [`wgpu`](https://github.com/gfx-rs/wgpu)）解耦。现在可以使用相机、灯光、着色器、图像、网格、精灵、文本、UI、选取、动画和场景而不依赖 `bevy_render`。

有了这些更改，第三方自定义渲染器现在可以作为渲染 Bevy 场景的直接替换，而无需引入 `bevy_render`。

这对于减少编译时间也非常重要，特别是对于第三方 crate：crate 作者现在可以更细粒度地依赖他们需要的特定 crate。如果他们不需要访问渲染器内部，他们就不需要等待它们开始编译！这增加了并行编译的潜力。

此外，具有最小依赖的"仅着色器库"crate 现在由于新的独立 `bevy_shader` crate 而成为可能。

## System Set 的一致命名约定 [#](https://bevy.org/news/bevy-0-17/#consistent-naming-conventions-for-system-sets)

Authors:[@Jondolf](https://github.com/Jondolf)

PRs:[#18900](https://github.com/bevyengine/bevy/pull/18900)

Bevy 及其生态系统内 `SystemSet` 类型的名称历史上一直非常不一致。System set 名称的示例包括 `AccessibilitySystem`、`PickSet`、`StateTransitionSteps` 和 `Animation`。

命名约定如此不一致可能使用户更难为自己的类型选择名称、在 docs.rs 上搜索 system set，甚至辨别哪些类型_是_ system set。

为了控制不一致性并帮助统一生态系统，**Bevy 0.17** 已将其大多数 system set 重命名为遵循一致的 `*Systems` 命名约定。正如你通过这个非常不完整的重命名列表所看到的，我们的命名到处都是：

- `GizmoRenderSystem` → `GizmoRenderSystems`
- `PickSet` → `PickingSystems`
- `Animation` → `AnimationSystems`
- `Update2dText` → `Text2dUpdateSystems`

选择 `Systems` 后缀而不是另一个流行的后缀 `Set`，是因为 `Systems` 更清楚地传达它是一个系统的集合，并且与其他 set 类型命名冲突的风险更低。

为了保持一致，我们建议生态系统 crate 和用户在适用时也采用 `*Systems` 命名约定用于他们的 system set。

## 下一步是什么？[#](https://bevy.org/news/bevy-0-17/#what-s-next)

上述功能可能很棒，但 Bevy 还有什么正在进行中？深入时间的迷雾中（当你的团队几乎全是志愿者时，预测_格外_困难！），我们可以看到一些令人兴奋的工作正在成型：

- **BSN：Bevy 的下一代 Scene / UI 系统：** 我们目前有一个[工作原型](https://github.com/bevyengine/bevy/pull/20158)，用于大幅改进的统一 Scene / UI 系统。我们计划在 **Bevy 0.18** 中发布新的 `bsn!` 宏和 `.bsn` 资产格式。
- **成熟的 UI 框架：** `feathers` 是一个很好的开始，但它才刚刚孵化。我们期待通过将其移植到 BSN、[添加更多 widget](https://github.com/bevyengine/bevy/issues/19236)、巩固主题和屏幕阅读器支持等功能，以及使你更容易构建项目特定的设计系统来改进人体工程学，使你能够创建一致样式、低样板的 UI。
- **第一方实体检查器：** [实体检查器](https://github.com/bevyengine/bevy/pull/20189)是一个极其有价值的调试工具，既作为临时开发工具也作为编辑器的关键元素。这将使用 `feathers` 构建，使我们能够完善其美学和功能，为更广泛的开发者工具做准备。
- **`firewheel` 音频：** [`firewheel`](https://github.com/BillyDM/firewheel) 团队一直在努力工作，为 Rust 创建生产级音频解决方案。我们对通过 [`bevy_seedling`](https://github.com/corvusprudens/bevy_seedling) 的草案集成感到鼓舞，并热衷于改进 Bevy 的第一方音频质量。
- **改进的示例：** 我们正在寻找扩展我们的 Bevy 使用示例，以确保涵盖更多真实世界场景，并最终为我们的示例添加生产级资产，以展示 Bevy 在展示才华横溢的艺术家作品时的表现。