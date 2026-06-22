# Bevy 0.5

## 发布于 2021 年 4 月 6 日，作者：Carter Anderson ( ![一个猫耳人物挥舞触手的剪影，或称 Octocat：GitHub 的吉祥物和标志](https://bevy.org/assets/github_grey.svg) [@cart](https://www.github.com/cart) ![一个圆角矩形内指向右的三角形；Youtube 的标志](https://bevy.org/assets/youtube_grey.svg) [cartdev](https://www.youtube.com/cartdev) )

![Ante 截图：由 @TheNeikos 使用 Bevy 开发的体素建造游戏](https://bevy.org/news/bevy-0-5/ante.png)

Ante 截图：由 @TheNeikos 使用 Bevy 开发的体素建造游戏

感谢 **88** 位贡献者、**283** 个 pull request，以及我们[慷慨的赞助者](https://github.com/sponsors/cart)，我很高兴地宣布 **Bevy 0.5** 已在 [crates.io](https://crates.io/crates/bevy) 上发布！

如果你还不了解 Bevy，它是一款基于 Rust 构建的、简洁的数据驱动游戏引擎。你可以查看[快速入门指南](https://bevy.org/learn/quick-start/introduction)来开始使用。Bevy 将永久免费且开源！你可以在 GitHub 上获取完整的[源代码](https://github.com/bevyengine/bevy)。查看 [Awesome Bevy](https://github.com/bevyengine/awesome-bevy) 获取社区开发的插件、游戏和学习资源列表。

**Bevy 0.5** 比过去几个版本要大得多（也花了更长时间），因为我们做了许多基础性的变更。如果你打算将你的 App 或 Plugin 更新到 **Bevy 0.5**，请查看我们的 [0.4 到 0.5 迁移指南](https://bevy.org/learn/migration-guides/0.4-0.5/)。

以下是本次发布的一些亮点：

## 基于物理的渲染（PBR）[#](https://bevy.org/news/bevy-0-5/#physically-based-rendering-pbr)

作者：@StarArawn、@mtsr、@mockersf、@IngmarBitter、@Josh015、@norgate、@cart

Bevy 现在在渲染时使用 PBR 着色器。PBR 是一种半标准的渲染方法，它试图使用真实世界"基于物理"的光照和材质属性的近似值。我们主要使用 [Filament](https://github.com/google/filament/) PBR 实现中的技术，但也结合了 [Unreal](https://www.unrealengine.com/en-US/blog/physically-based-shading-on-mobile) 和 [Disney](https://google.github.io/filament/Filament.html#citation-burley12) 的一些理念。

Bevy 的 [`StandardMaterial`](https://docs.rs/bevy/0.5.0/bevy/pbr/prelude/struct.StandardMaterial.html) 现在具有 `base_color`、`roughness`、`metallic`、`reflection` 和 `emissive` 属性。它还支持 `base_color`、`normal_map`、`metallic_roughness`、`emissive` 和 `occlusion` 属性的纹理。

新的 PBR 示例有助于可视化这些新的材质属性：

![pbr](https://bevy.org/news/bevy-0-5/pbr.png)

## GLTF 改进 [#](https://bevy.org/news/bevy-0-5/#gltf-improvements)

### PBR 纹理 [#](https://bevy.org/news/bevy-0-5/#pbr-textures)

作者：@mtsr、@mockersf

GLTF 加载器现在支持法线贴图、metallic/roughness、遮挡和自发光纹理。我们的"flight helmet" gltf 示例利用了新的 PBR 纹理支持，效果看起来好多了：

### 顶级 GLTF 资源 [#](https://bevy.org/news/bevy-0-5/#top-level-gltf-asset)

作者：@mockersf

以前很难与 GLTF 资源交互，因为场景/网格/纹理/材质只作为"子资源"加载。感谢新的顶级 [`Gltf`](https://docs.rs/bevy/0.5.0/bevy/gltf/struct.Gltf.html) 资源类型，现在可以浏览 GLTF 资源的内容了：

```rust
// 在启动时加载 GLTF 资源
fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let handle = assets.load("flight_helmet.gltf");
    commands.insert_resource(handle);
}

// 在之后的某个时间点访问 GLTF 资源
fn system(handle: Res<Handle<Gltf>>, gltfs: Res<Assets<Gltf>>, materials: Res<Assets<StandardMaterial>>) {
    let gltf = gltfs.get(&handle).unwrap();
    let material_handle = gltf.named_materials.get("MetalPartsMat").unwrap();
    let material = materials.get(material_handle).unwrap();
}
```

## Bevy ECS V2 [#](https://bevy.org/news/bevy-0-5/#bevy-ecs-v2)

本次发布标志着 Bevy ECS 的一个巨大进步。它对 Bevy App 的组成方式和性能有重大影响：

- **[ECS 核心全面重写：](https://bevy.org/news/bevy-0-5/#ecs-core-rewrite)**
    - 全面大幅提升性能
    - "混合"组件存储
    - 用于更快原型变更的"原型图"
    - 跨运行缓存结果的有状态查询
- **[全新的并行系统执行器：](https://bevy.org/news/bevy-0-5/#new-parallel-system-executor)**
    - 支持显式系统排序
    - System Labels
    - System Sets
    - 改进的系统"运行条件"
    - 增加系统并行度
- **["可靠的"变更检测：](https://bevy.org/news/bevy-0-5/#reliable-change-detection)**
    - 系统现在总是能检测到组件变更，即使跨越帧
- **[状态系统重写：](https://bevy.org/news/bevy-0-5/#states-v2)**
    - 更自然的"基于栈的状态机"模型
    - 直接与新调度器集成
    - 改进的"状态生命周期"事件

请继续阅读了解详情！

## ECS 核心重写 [#](https://bevy.org/news/bevy-0-5/#ecs-core-rewrite)

作者：@cart

在此之前，Bevy 使用了 [hecs](https://github.com/Ralith/hecs) 的一个大量 fork 版本作为 ECS 核心。自从 Bevy 首次发布以来，我们对 Bevy 的 ECS 需求有了很多了解。我们还与其他 ECS 项目负责人合作，如 [Sander Mertens](https://github.com/SanderMertens)（[flecs](https://github.com/SanderMertens/flecs) 首席开发者）和 [Gijs-Jan Roelofs](https://github.com/gjroelofs)（Xenonauts ECS 框架开发者）。作为"ECS 社区"，我们开始聚焦于 ECS 的未来发展方向。

Bevy ECS V2 是我们迈向那个未来的第一步。这也意味着 Bevy ECS 不再是"hecs fork"。我们将独立前行！

### 组件存储（问题）[#](https://bevy.org/news/bevy-0-5/#component-storage-the-problem)

多年来，两种 ECS 存储范式获得了广泛认可：

- **基于原型的 ECS（Archetypal ECS）**：
    - 将组件存储在具有静态 schema 的"表"中。每个"列"存储给定类型的组件。每个"行"是一个实体。
    - 每个"原型"有自己的表。添加/移除实体的组件会更改原型。
    - 由于其缓存友好的数据布局，支持超快的查询迭代
    - 代价是实体组件的添加/移除操作更昂贵，因为所有组件都需要复制到新原型的"表"中
    - 对并行友好：实体一次只能存在于一个原型中，因此访问相同组件但不同原型的系统可以并行运行
    - 框架：旧版 Bevy ECS、hecs、legion、flecs、Unity DOTS
- **稀疏集 ECS（Sparse Set ECS）**：
    - 将相同类型的组件存储在密集打包的数组中，通过密集打包的无符号整数（entity id）进行稀疏索引
    - 查询迭代比基于原型的 ECS 慢（默认），因为每个实体的组件可能在稀疏集中的任何位置。这种"随机访问"模式对缓存不友好。此外，还有一层额外的间接寻址，因为你必须首先将 entity id 映射到组件数组中的索引。
    - 添加/移除组件是廉价的常数时间操作
    - "组件包（Component Packs）"用于根据具体情况优化迭代性能（但包之间会冲突）
    - 对并行不太友好：系统需要锁定整个组件存储（粒度不够）或单个实体（开销大）
    - 框架：Shipyard、EnTT

开发者选择 ECS 框架时面临着一个艰难的选择。选择一个"到处快速迭代"的"基于原型"的框架，但无法廉价地添加/移除组件；或者选择一个可以廉价添加/移除组件的"稀疏集"框架，但迭代性能较慢或需要手动（且冲突的）包优化。

### 混合组件存储（解决方案）[#](https://bevy.org/news/bevy-0-5/#hybrid-component-storage-the-solution)

在 Bevy ECS V2 中，我们两全其美。它现在_同时_具有上述两种组件存储类型（如果需要，以后还可以添加更多）：

- **表（Tables）**（即其他框架中的"基于原型"存储）
    - 默认存储。如果你不配置任何内容，这就是你得到的
    - 默认快速迭代
    - 较慢的添加/移除操作
- **稀疏集（Sparse Sets）**
    - 可选启用
    - 较慢的迭代
    - 较快的添加/移除操作

这些存储类型完美互补。默认情况下查询迭代很快。如果开发者知道他们想要高频率地添加/移除组件，他们可以将存储设置为"稀疏集"：

```rust
app.register_component(
    ComponentDescriptor::new::<MyComponent>(StorageType::SparseSet)
);
```

#### 组件添加/移除基准测试（单位：毫秒，越低越好）[#](https://bevy.org/news/bevy-0-5/#component-add-remove-benchmark-in-milliseconds-less-is-better)

此基准测试展示了从一个具有 5 个其他 4x4 矩阵组件的实体上添加和移除单个 4x4 矩阵组件 10,000 次。"其他"组件的加入是为了帮助说明"表存储"（由 Bevy 0.4、Bevy 0.5（Table）和 Legion 使用）的成本，因为它需要将"其他"组件移动到新表中。

![组件添加/移除](https://bevy.org/news/bevy-0-5/add_remove_big.svg)

你可能注意到 **Bevy 0.5（Table）** 也比 **Bevy 0.4** 快得多，尽管它们都使用"表存储"。这在很大程度上是新的[原型图](https://github.com/bevyengine/bevy/pull/1525)的结果，它显著降低了原型变更的成本。

### 有状态查询和系统参数 [#](https://bevy.org/news/bevy-0-5/#stateful-queries-and-system-parameters)

[`World`](https://docs.rs/bevy/0.5.0/bevy/ecs/world/struct.World.html) 查询（和其他系统参数）现在是有状态的。这允许我们：

1. 缓存原型（和表）匹配结果
    - 这解决了（朴素的）基于原型 ECS 的另一个问题：随着原型数量增加（且出现碎片化），查询性能会变差。
2. 缓存查询获取和过滤状态
    - 获取/过滤操作中昂贵的部分（例如对 TypeId 进行哈希以找到 ComponentId）现在只在查询首次构造时发生一次
3. 增量构建状态
    - 当添加新原型时，我们只处理新原型（不需要为旧原型重建状态）

因此，直接的 [`World`](https://docs.rs/bevy/0.5.0/bevy/ecs/world/struct.World.html) 查询 API 现在看起来像这样：

```rust
let mut query = world.query::<(&A, &mut B)>();
for (a, mut b) in query.iter_mut(&mut world) {
}
```

然而，对于系统来说这是一个非破坏性变更。查询状态管理由相关的 [`SystemParam`](https://docs.rs/bevy/0.5.0/bevy/ecs/system/derive.SystemParam.html) 在内部完成。

由于新的 `Query` 系统，我们取得了一些相当显著的性能提升。

#### "稀疏"碎片化迭代器基准测试（单位：纳秒，越低越好）[#](https://bevy.org/news/bevy-0-5/#sparse-fragmented-iterator-benchmark-in-nanoseconds-less-is-better)

此基准测试运行一个在单个原型中匹配 5 个实体且_不匹配_ 100 个其他原型的查询。这是对游戏中"真实世界"查询的合理测试，游戏中通常有许多不同的实体"类型"，其中大多数_不匹配_给定查询。此测试全面使用"表存储"。

![sparse_frag_iter](https://bevy.org/news/bevy-0-5/sparse_frag_iter.svg)

**Bevy 0.5** 在此类情况下取得了巨大改进，这要归功于新的"有状态查询"。**Bevy 0.4** 每次运行迭代器时都需要检查每个原型，而 **Bevy 0.5** 将此成本摊销为零。

#### 碎片化迭代器基准测试（单位：毫秒，越低越好）[#](https://bevy.org/news/bevy-0-5/#fragmented-iterator-benchmark-in-milliseconds-less-is-better)

这是 [ecs_bench_suite](https://github.com/rust-gamedev/ecs_bench_suite) 的 `frag_iter` 基准测试。它在 27 个原型上运行查询，每个原型有 20 个实体。但与"稀疏碎片化迭代器基准测试"不同的是，这里没有"不匹配"的原型。此测试全面使用"表存储"。

![frag_iter](https://bevy.org/news/bevy-0-5/frag_iter.svg)

与上一个基准测试相比，这里的增益较小，因为没有不匹配的原型。然而 **Bevy 0.5** 由于更好的迭代器/查询实现、将匹配原型的成本摊销为零以及 for_each 迭代器，仍然获得了不错的提升。

### 超快的 "for_each" 查询迭代器 [#](https://bevy.org/news/bevy-0-5/#uber-fast-for-each-query-iterators)

开发者现在可以选择使用快速的 [`Query::for_each`](https://docs.rs/bevy/0.5.0/bevy/ecs/system/struct.Query.html#method.for_each) 迭代器，它在"碎片化迭代"中提供约 1.5-3 倍的迭代速度提升，在非碎片化迭代中提供约 1.2 倍的轻微提升。

```rust
fn system(query: Query<(&A, &mut B)>) {
    // 你现在可以选择这样做来获得速度提升
    query.for_each_mut(|(a, mut b)| {
    });

    // 但普通迭代器仍然可用
    for (a, mut b) in query.iter_mut() {
    }
}
```

我们将继续鼓励使用"普通"迭代器，因为它们更灵活且更"符合 Rust 习惯"。但当需要额外的"爆发力"时，`for_each` 会在那里……等着你 :)

## 新的并行系统执行器 [#](https://bevy.org/news/bevy-0-5/#new-parallel-system-executor)

作者：@Ratysz

Bevy 旧的并行执行器有一些根本性的限制：

1. 显式定义系统顺序的唯一方法是创建新的阶段（stages）。这既冗长又阻碍了并行（因为阶段按顺序"逐个"运行）。我们注意到系统排序是一个常见需求，而阶段根本不够用。
2. 当系统访问冲突资源时，它们有"隐式"排序。这些排序难以理解。
3. "隐式排序"产生的执行策略通常会留下很多并行潜力。

幸运的是，@Ratysz 一直在该领域进行大量[研究](https://ratysz.github.io/article/scheduling-1/)（[项目](https://github.com/Ratysz/yaks/)），并自愿贡献一个新的执行器。新的执行器解决了上述所有问题，还添加了许多新的可用性改进。"排序"规则现在非常简单：

1. 系统默认并行运行
2. 定义了显式排序的系统将遵守这些排序

### 显式系统依赖和 System Labels [#](https://bevy.org/news/bevy-0-5/#explicit-system-dependencies-and-system-labels)

作者：@Ratysz、@TheRawMeatball

系统现在可以被分配一个或多个 [`SystemLabels`](https://docs.rs/bevy/0.5.0/bevy/ecs/schedule/trait.SystemLabel.html)。其他系统（在同一阶段内）可以引用这些标签，以在具有该标签的系统之前或之后运行：

```rust
app
    .add_system(update_velocity.system().label("velocity"))
    // "movement" 系统将在 "update_velocity" 之后运行
    .add_system(movement.system().after("velocity"))
```

这会产生等效的排序，但它使用的是 `before()`。

```rust
app
    // "update_velocity" 系统将在 "movement" 之前运行
    .add_system(update_velocity.system().before("movement"))
    .add_system(movement.system().label("movement"));
```

任何实现了 [`SystemLabel`](https://docs.rs/bevy/0.5.0/bevy/ecs/schedule/trait.SystemLabel.html) trait 的类型都可以使用。在大多数情况下，我们建议定义自定义类型并为它们派生 [`SystemLabel`](https://docs.rs/bevy/0.5.0/bevy/ecs/schedule/trait.SystemLabel.html)。这可以防止拼写错误，允许封装（需要时），并允许 IDE 自动补全标签：

```rust
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum PhysicsSystem {
    UpdateVelocity,
    Movement,
}

app
    .add_system(update_velocity.system().label(PhysicsSystem::UpdateVelocity))
    .add_system(movement.system()
        .label(PhysicsSystem::Movement)
        .after(PhysicsSystem::UpdateVelocity)
    );
```

### 多对多 System Labels [#](https://bevy.org/news/bevy-0-5/#many-to-many-system-labels)

多对多标签是一个强大的概念，它使得轻松依赖多个产生给定行为/结果的系统变得容易。例如，如果你有一个需要在所有"物理"更新完成后运行的系统（参见上面的示例），你可以用同一个 `Physics` 标签标记所有"物理系统"：

```rust
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub struct Physics;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum PhysicsSystem {
    UpdateVelocity,
    Movement,
}

app
    .add_system(update_velocity.system()
        .label(PhysicsSystem::UpdateVelocity)
        .label(Physics)
    )
    .add_system(movement.system()
        .label(PhysicsSystem::Movement)
        .label(Physics)
        .after(PhysicsSystem::UpdateVelocity)
    )
    .add_system(runs_after_physics.system().after(Physics));
```

Bevy 插件作者应该在其公共 API 中这样导出标签，以使用户能够在插件提供的逻辑之前/之后插入系统。

### System Sets [#](https://bevy.org/news/bevy-0-5/#system-sets)

[`SystemSets`](https://docs.rs/bevy/0.5.0/bevy/ecs/schedule/struct.SystemSet.html) 是一种将相同配置应用于一组系统的新方式，显著减少了样板代码。上面的"物理"示例可以改写为：

```rust
app
    .add_system_set(SystemSet::new()
        // 此标签将被添加到集合中的所有系统
        .label(Physics)
        .with_system(update_velocity.system().label(PhysicsSystem::UpdateVelocity))
        .with_system(movement.system()
            .label(PhysicsSystem::Movement)
            .after(PhysicsSystem::UpdateVelocity)
        )
    )
```

SystemSets 也可以使用 `before(Label)` 和 `after(Label)` 来在给定标签之前/之后运行集合中的所有系统。

这对于需要以相同 [`RunCriteria`](https://docs.rs/bevy/0.5.0/bevy/ecs/schedule/struct.RunCriteria.html) 运行的系统组也非常有用。

```rust
app
    // 此集合中的所有系统每两秒运行一次
    .add_system_set(SystemSet::new()
        .with_run_criteria(FixedTimestep::step(2.0))
        .with_system(foo.system())
        .with_system(bar.system())
    )
```

### 改进的运行条件 [#](https://bevy.org/news/bevy-0-5/#improved-run-criteria)

运行条件现在与系统解耦，并将在可能时被复用。例如，上面示例中的 FixedTimestep 条件每次阶段运行时只会运行一次。执行器将为 `foo` 和 `bar` 系统复用该条件的结果。

运行条件现在也可以被标记和被其他系统引用：

```rust
fn every_other_time(mut has_ran: Local<bool>) -> ShouldRun {
    *has_ran = !*has_ran;
    if *has_ran {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

app.add_stage(SystemStage::parallel()
   .with_system_run_criteria(every_other_time.system().label("every_other_time")))
   .add_system(foo.system().with_run_criteria("every_other_time"))
```

运行条件的结果也可以"管道化"到其他条件中，这实现了有趣的组合行为：

```rust
fn once_in_a_blue_moon(In(input): In<ShouldRun>, moon: Res<Moon>) -> ShouldRun {
    if moon.is_blue() {
        input
    } else {
        ShouldRun::No
    }
}

app
    .add_system(foo.with_run_criteria(
        "every_other_time".pipe(once_in_a_blue_moon.system())
    )
```

### 歧义检测和解决 [#](https://bevy.org/news/bevy-0-5/#ambiguity-detection-and-resolution)

虽然新的执行器现在更容易理解，但它确实引入了一类新的错误："系统顺序歧义"。当两个系统与相同的数据交互，但没有定义显式排序时，它们产生的输出是不确定的（通常不是作者的意图）。

考虑以下应用：

```rust
fn increment_counter(mut counter: ResMut<usize>) {
    *counter += 1;
}

fn print_every_other_time(counter: Res<usize>) {
    if *counter % 2 == 0 {
        println!("ran");
    }
}

app
    .add_system(increment_counter.system())
    .add_system(print_every_other_time.system())
```

作者显然打算 `print_every_other_time` 每隔一次更新运行。然而，由于这些系统没有定义排序，它们每次更新可能以不同的顺序运行，导致在两次更新过程中什么都没有打印：

```txt
UPDATE
- increment_counter (counter now equals 1)
- print_every_other_time (nothing printed)
UPDATE
- print_every_other_time (nothing printed)
- increment_counter (counter now equals 2)
```

旧的执行器会隐式地强制 `increment_counter` 先运行，因为它与 `print_every_other_time` 冲突且它先被插入。但新的执行器要求你在这里显式说明（我们相信这是件好事）。

为了帮助检测这类错误，我们构建了一个可选启用的工具来检测这些歧义并记录它们：

```rust
// 将此资源添加到你的 App 以启用歧义检测
app.insert_resource(ReportExecutionOrderAmbiguities)
```

然后当我们运行 App 时，会在终端中看到以下消息：

```txt
Execution order ambiguities detected, you might want to add an explicit dependency relation between some of these systems:
 * Parallel systems:
 -- "&app::increment_counter" and "&app::print_every_other_time"
    conflicts: ["usize"]
```

歧义检测器发现了一个冲突，并提到添加显式依赖关系将解决此冲突：

```rust
app
    .add_system(increment_counter.system().label("increment"))
    .add_system(print_every_other_time.system().after("increment"))
```

在某些情况下歧义_不是_ bug，例如对无序集合（如 `Assets`）的操作。这就是为什么我们默认不启用检测器。你可以自由地忽略这些歧义，但如果你想在检测器中抑制消息（而不定义依赖关系），你可以将你的系统添加到"歧义集"中：

```rust
app
    .add_system(a.system().in_ambiguity_set("foo"))
    .add_system(b.system().in_ambiguity_set("foo"))
```

我想强调这完全是可选的。Bevy 代码应该具有人体工学性且"有趣"。如果到处添加歧义集不合你意，就别担心！

我们也在积极寻求关于新执行器的反馈。我们相信新的实现更容易理解，并鼓励自文档化的代码。改进的并行性也很不错！但我们希望听到用户的反馈（无论是从零开始的新用户还是将代码库迁移到新执行器的老用户）。这个领域关乎设计权衡，反馈将帮助我们确保做出了正确的决策。

## 可靠的变更检测 [#](https://bevy.org/news/bevy-0-5/#reliable-change-detection)

作者：@Davier、@bjorn3、@alice-i-cecile、@cart

全局变更检测——在任何 ECS 组件或资源的 Changed/Added 状态上运行查询的能力——刚刚获得了重大可用性提升：现在可以跨帧/更新检测变更：

```rust
// 这仍然是我们都知道和喜爱的同一个变更检测 API，
// 唯一的区别是它在每种情况下都"开箱即用"。
fn system(query: Query<Entity, Changed<A>>) {
    // 迭代所有自上次运行此系统以来
    // A 组件已更改的实体
    for e in query.iter() {
    }
}
```

全局变更检测已经是一个让 Bevy 与其他 ECS 框架区分开来的功能，但现在它完全"傻瓜式"。无论系统排序、阶段成员资格还是系统运行条件如何，它都能按预期工作。

旧的行为是"系统检测在本帧中在它们之前运行的系统中发生的变更"。这是因为我们使用 `bool` 来跟踪每个组件/资源何时被添加/修改。此标志在每帧结束时为每个组件清除。因此，用户必须非常小心操作顺序，使用"系统运行条件"等功能可能导致在给定更新中系统未运行时丢失变更。

我们现在使用巧妙的"世界 tick"设计，允许系统检测自上次运行以来_任何_时间点发生的变更。

## States V2 [#](https://bevy.org/news/bevy-0-5/#states-v2)

作者：@TheRawMeatball

[上次 Bevy 发布](https://bevy.org/news/bevy-0-4)添加了 States，使开发者能够根据 `State<T>` 资源的值运行一组 ECS 系统。系统可以根据"状态生命周期事件"运行，如 on_enter、on_update 和 on_exit。States 使得在 Bevy ECS 中编码单独的"加载屏幕"和"游戏中"逻辑变得更容易。

旧的实现基本可以工作，但它有一些怪癖和限制。首先，它需要添加新的 `StateStage`，这减少了并行度，增加了样板代码，并在不需要的地方强制排序。此外，一些生命周期事件并不总是按预期工作。

新的 [`State`](https://docs.rs/bevy/0.5.0/bevy/ecs/schedule/struct.State.html) 实现建立在新的并行执行器的 SystemSet 和 RunCriteria 功能之上，提供了一个更自然、灵活和并行的 API，它建立在现有概念之上，而不是创建新概念：

```rust
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Menu,
    InGame,
}

fn main() {
    App::build()
        .add_state(AppState::Menu)
        .add_system_set(SystemSet::on_enter(AppState::Menu).with_system(setup_menu.system()))
        .add_system_set(SystemSet::on_update(AppState::Menu).with_system(menu_logic.system()))
        .add_system_set(SystemSet::on_exit(AppState::Menu).with_system(cleanup_menu.system()))
        .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup_game.system()))
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(game_logic.system())
                .with_system(more_game_logic.system())
        )
        .run();
}
```

States 现在使用"基于栈的状态机"模型。这为状态转换提供了多种选择：

```rust
fn system(mut state: ResMut<State<AppState>>) {
    // 排队一个状态变更，将新状态推入
    // 栈中（保留之前的状态）
    state.push(AppState::InGame).unwrap();

    // 排队一个状态变更，移除栈上的
    // 当前状态并恢复到之前的状态
    state.pop().unwrap();

    // 排队一个状态变更，覆盖栈
    // "顶部"的当前状态
    state.set(AppState::InGame).unwrap();

    // 排队一个状态变更，替换整个状态栈
    state.replace(AppState::InGame).unwrap();
}
```

与旧的实现一样，状态变更在同一帧内应用。这意味着可以从状态 `A->B->C` 过渡并运行相关的状态生命周期事件而不会跳帧。这建立在"循环运行条件"之上，我们也用它来实现"固定时间步长"（你也可以用自己的运行条件逻辑使用它）。

## 事件人体工学 [#](https://bevy.org/news/bevy-0-5/#event-ergonomics)

作者：@TheRawMeatball

事件现在有了一流的简写语法，更易于消费：

```rust
// 旧的 Bevy 0.4 语法
fn system(mut reader: Local<EventReader<SomeEvent>>, events: Res<Events<SomeEvent>>) {
    for event in reader.iter(&events) {
    }
}

// 新的 Bevy 0.5 语法
fn system(mut reader: EventReader<SomeEvent>) {
    for event in reader.iter() {
    }
}
```

现在还有一个对称的 `EventWriter` API：

```rust
fn system(mut writer: EventWriter<SomeEvent>) {
    writer.send(SomeEvent { ... })
}
```

旧的"手动"方式仍然可以通过 `ManualEventReader` 使用：

```rust
fn system(mut reader: Local<ManualEventReader<SomeEvent>>, events: Res<Events<SomeEvent>>) {
    for event in reader.iter(&events) {
    }
}
```

## 富文本 [#](https://bevy.org/news/bevy-0-5/#rich-text)

作者：@tigregalis

文本现在可以有"段落"，每个段落都有自己的样式/格式。这使得文本更加灵活，同时仍然遵守文本布局规则：

![rich_text](https://bevy.org/news/bevy-0-5/rich_text.png)

这是通过新的"文本段落"API 实现的：

```rust
commands
    .spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "FPS: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("FiraSans-Bold.ttf"),
                        font_size: 90.0,
                        color: Color::WHITE,
                    },
                },
                TextSection {
                    value: "60.03".to_string(),
                    style: TextStyle {
                        font: asset_server.load("FiraMono-Medium.ttf"),
                        font_size: 90.0,
                        color: Color::GOLD,
                    },
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    })
```

## 高 DPI 文本 [#](https://bevy.org/news/bevy-0-5/#hidpi-text)

作者：@blunted2night

文本现在根据当前显示器的缩放因子进行渲染。这在任何分辨率下都能提供清晰、锐利的文本。

![hidpi_text](https://bevy.org/news/bevy-0-5/hidpi_text.png)

## 在 2D 世界空间中渲染文本 [#](https://bevy.org/news/bevy-0-5/#render-text-in-2d-world-space)

作者：@CleanCut、@blunted2night

文本现在可以使用新的 `Text2dBundle` 生成到 2D 场景中。这使得实现"在玩家头顶绘制名字"等功能变得更容易。

## 世界坐标到屏幕坐标的转换 [#](https://bevy.org/news/bevy-0-5/#world-to-screen-coordinate-conversions)

作者：@aevyrie

现在可以使用新的 `Camera::world_to_screen()` 函数将世界坐标转换为给定相机的屏幕坐标。以下是此功能用于在移动的 3D 对象上定位 UI 元素的示例。

## 3D 正交相机 [#](https://bevy.org/news/bevy-0-5/#3d-orthographic-camera)

作者：@jamadazi

正交相机现在可以在 3D 中使用！这对于 CAD 应用和等距游戏等场景很有用。

![ortho_3d](https://bevy.org/news/bevy-0-5/ortho_3d.png)

## 正交相机缩放模式 [#](https://bevy.org/news/bevy-0-5/#orthographic-camera-scaling-modes)

作者：@jamadazi

在 **Bevy 0.5** 之前，Bevy 的正交相机只有一种模式："窗口缩放"。它会根据窗口的垂直和水平大小调整投影。这适用于某些类型的游戏，但其他游戏需要任意的窗口无关缩放因子，或由水平或垂直窗口大小定义的缩放因子。

**Bevy 0.5** 为 `OrthographicCamera` 添加了新的 `ScalingMode` 选项，使开发者能够自定义投影的计算方式。

它还添加了使用 `OrthographicProjection::scale` 来"缩放"相机的能力。

## 灵活的相机绑定 [#](https://bevy.org/news/bevy-0-5/#flexible-camera-bindings)

作者：@cart

Bevy 以前为每个 RenderGraph PassNode "硬编码"相机绑定。当只有一种绑定类型（组合的 `ViewProj` 矩阵）时这可以工作，但许多着色器需要其他相机属性，如世界空间位置。

在 Bevy 0.5 中，我们移除了这个"hack"，转而使用在其他地方使用的 `RenderResourceBindings` 系统。这使着色器能够绑定任意相机数据（使用任何 set 或 binding index），并且只拉取它们需要的数据。

新的 PBR 着色器利用了这一特性，但自定义着色器也可以使用它。

```glsl
layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 0, binding = 1) uniform CameraPosition {
    vec3 CameraPos;
};
```

## 渲染层 [#](https://bevy.org/news/bevy-0-5/#render-layers)

作者：@schell

有时你不希望相机绘制场景中的所有内容，或者你想临时隐藏场景中的一组事物。**Bevy 0.5** 添加了 `RenderLayer` 系统，使开发者能够通过添加 [`RenderLayers`](https://docs.rs/bevy/0.5.0/bevy/render/camera/struct.RenderLayers.html) 组件将实体添加到层中。

相机也可以有 [`RenderLayers`](https://docs.rs/bevy/0.5.0/bevy/render/camera/struct.RenderLayers.html) 组件，这决定了它们可以看到哪些层。

```rust
// 在第 0 层生成一个精灵
commands
    .spawn_bundle(SpriteBundle {
        material: materials.add(Color::rgb(1.0, 0.5, 0.5).into()),
        transform: Transform::from_xyz(0.0, -50.0, 1.0),
        sprite: Sprite::new(Vec2::new(30.0, 30.0)),
    })
    .insert(RenderLayers::layer(0));
// 在第 1 层生成一个精灵
commands
    .spawn_bundle(SpriteBundle {
        material: materials.add(Color::rgb(1.0, 0.5, 0.5).into()),
        transform: Transform::from_xyz(0.0, -50.0, 1.0),
        sprite: Sprite::new(Vec2::new(30.0, 30.0)),
    })
    .insert(RenderLayers::layer(1));
// 生成一个只绘制第 1 层精灵的相机
commands
    .spawn_bundle(OrthographicCameraBundle::new_2d());
    .insert(RenderLayers::layer(1));
```

## 精灵翻转 [#](https://bevy.org/news/bevy-0-5/#sprite-flipping)

作者：@zicklag

精灵现在可以轻松（且高效地）沿 x 或 y 轴翻转：

![sprite_flipping](https://bevy.org/news/bevy-0-5/sprite_flipping.png)

```rust
commands.spawn_bundle(SpriteBundle {
    material: material.clone(),
    transform: Transform::from_xyz(150.0, 0.0, 0.0),
    ..Default::default()
});
commands.spawn_bundle(SpriteBundle {
    material,
    transform: Transform::from_xyz(-150.0, 0.0, 0.0),
    sprite: Sprite {
        // 将 logo 翻转到左侧
        flip_x: true,
        // 不要上下翻转（默认值）
        flip_y: false,
        ..Default::default()
    },
    ..Default::default()
});
```

## 色彩空间 [#](https://bevy.org/news/bevy-0-5/#color-spaces)

作者：@mockersf

[`Color`](https://docs.rs/bevy/0.5.0/bevy/render/color/enum.Color.html) 现在在内部表示为枚举，这使得无损（且正确）的颜色表示成为可能。相比之前的实现，这是一个重大改进，之前的实现内部将所有颜色转换为线性 sRGB（这可能导致精度问题）。颜色现在只在发送到 GPU 时才转换为线性 sRGB。我们还借此机会修复了一些在错误色彩空间中定义的不正确颜色常量。

```rust
pub enum Color {
    /// sRGBA 颜色
    Rgba {
        /// 红色分量。[0.0, 1.0]
        red: f32,
        /// 绿色分量。[0.0, 1.0]
        green: f32,
        /// 蓝色分量。[0.0, 1.0]
        blue: f32,
        /// Alpha 分量。[0.0, 1.0]
        alpha: f32,
    },
    /// 线性 sRGB 色彩空间中的 RGBA 颜色（通常口语化地称为"线性"、"RGB"或"线性 RGB"）。
    RgbaLinear {
        /// 红色分量。[0.0, 1.0]
        red: f32,
        /// 绿色分量。[0.0, 1.0]
        green: f32,
        /// 蓝色分量。[0.0, 1.0]
        blue: f32,
        /// Alpha 分量。[0.0, 1.0]
        alpha: f32,
    },
    /// 带 alpha 通道的 HSL（色相、饱和度、亮度）颜色
    Hsla {
        /// 色相分量。[0.0, 360.0]
        hue: f32,
        /// 饱和度分量。[0.0, 1.0]
        saturation: f32,
        /// 亮度分量。[0.0, 1.0]
        lightness: f32,
        /// Alpha 分量。[0.0, 1.0]
        alpha: f32,
    },
}
```

## 线框模式 [#](https://bevy.org/news/bevy-0-5/#wireframes)

作者：@Neo-Zhixing

Bevy 现在可以使用可选的 `WireframePlugin` 绘制线框

![wireframe](https://bevy.org/news/bevy-0-5/wireframe.png)

这些可以全局启用，也可以通过添加新的 `Wireframe` 组件按实体启用。

## 简单 3D 游戏示例：Alien Cake Addict [#](https://bevy.org/news/bevy-0-5/#simple-3d-game-example-alien-cake-addict)

作者：@mockersf

此示例作为在 Bevy 中构建 3D 游戏的快速入门。它展示了如何生成场景、响应输入、实现游戏逻辑和处理状态转换。尽可能多地收集蛋糕！

![alien_cake_addict](https://bevy.org/news/bevy-0-5/alien_cake_addict.png)

## 计时器改进 [#](https://bevy.org/news/bevy-0-5/#timer-improvements)

作者：@kokounet

[`Timer`](https://docs.rs/bevy/0.5.0/bevy/core/struct.Timer.html) 结构体现在内部使用 [`Duration`](https://docs.rs/bevy/0.5.0/bevy/utils/struct.Duration.html) 而不是使用秒的 `f32` 表示。这既提高了精度，也使 API 看起来更友好。

```rust
fn system(mut timer: ResMut<Timer>, time: Res<Time>) {
    if timer.tick(time.delta()).just_finished() {
        println!("timer just finished");
    }
}
```

## 资源改进 [#](https://bevy.org/news/bevy-0-5/#assets-improvements)

作者：@willcrichton、@zicklag、@mockersf、@Archina

Bevy 的资源系统在本次发布中有几个小改进：

- Bevy 在加载资源时不再因错误而 panic
- 带有多个点的资源路径现在被正确处理
- 改进了资产加载器产生的"标记资源"的类型安全性
- 资源路径加载现在不区分大小写

## WGPU 配置选项 [#](https://bevy.org/news/bevy-0-5/#wgpu-configuration-options)

作者：@Neo-Zhixing

现在可以通过在 `WgpuOptions` 资源中设置来启用/禁用 wgpu 功能（如 `WgpuFeature::PushConstants` 和 `WgpuFeature::NonFillPolygonMode`）：

```rust
app
    .insert_resource(WgpuOptions {
        features: WgpuFeatures {
            features: vec![WgpuFeature::NonFillPolygonMode],
        },
        ..Default::default()
    })

```

Wgpu 限制（如 `WgpuLimits::max_bind_groups`）现在也可以在 `WgpuOptions` 资源中配置。

## 场景实例实体迭代 [#](https://bevy.org/news/bevy-0-5/#scene-instance-entity-iteration)

作者：@mockersf

现在可以迭代生成的场景实例中的所有实体。这使得在场景加载后对其进行后处理成为可能。

```rust
struct MySceneInstance(InstanceId);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    // 生成一个场景并保留其 `instance_id`
    let instance_id = scene_spawner.spawn(asset_server.load("model.gltf#Scene0"));
    commands.insert_resource(MySceneInstance(instance_id));
}

fn print_scene_entities(
    scene_spawner: Res<SceneSpawner>,
    scene_instance: Res<MySceneInstance>,
) {
    if let Some(entity_iter) = scene_spawner.iter_instance_entities(scene_instance.0) {
        for entity in entity_iter {
            println!("Found scene entity {:?}", entity);
        }
    }
}
```

## 窗口调整大小约束 [#](https://bevy.org/news/bevy-0-5/#window-resize-constraints)

作者：@digital7-code

窗口现在可以有"调整大小约束"。窗口不能被调整到超出这些约束

```rust
app
    .insert_resource(WindowDescriptor {
        resize_constraints: WindowResizeConstraints {
            min_height: 200.0,
            max_height: 800.0,
            ..Default::default()
        },
        ..Default::default()
    })
```

## !Send 任务 [#](https://bevy.org/news/bevy-0-5/#send-tasks)

作者：@alec-deason

Bevy 的异步任务系统现在支持 `!Send` 任务。某些任务不能被发送/在其他线程上运行（例如即将推出的 Distill 资源插件创建的任务）。"线程本地"任务现在可以像这样在 Bevy `TaskPools` 中生成：

```rust
let pool = TaskPool::default();
pool.scope(|scope| {
    scope.spawn_local(async move {
        println!("I am a local task");
    });
});
```

## 更多 ECS V2 变更 [#](https://bevy.org/news/bevy-0-5/#more-ecs-v2-changes)

### EntityRef / EntityMut [#](https://bevy.org/news/bevy-0-5/#entityref-entitymut)

作者：@cart

**Bevy 0.4** 中的 World 实体操作要求用户为每个操作传入一个 `entity` id：

```rust
let entity = world.spawn((A, )); // 创建一个带有 A 的新实体
world.get::<A>(entity);
world.insert(entity, (B, C));
world.insert_one(entity, D);
```

这意味着每个操作都需要查找实体位置/验证其有效性。初始生成操作还需要一个 Bundle 作为输入。当不需要组件（或只需要一个组件）时，这可能很笨拙。

这些操作已被 `EntityRef` 和 `EntityMut` 替换，它们是围绕 world 的"构建器风格"包装器，在单个、预验证的实体上提供读和读/写操作：

```rust
// spawn 现在不接受输入并返回 EntityMut
let entity = world.spawn()
    .insert(A) // 向实体插入单个组件
    .insert_bundle((B, C)) // 向实体插入一组组件
    .id() // id 返回 Entity id

// 返回 EntityMut（如果实体不存在则 panic）
world.entity_mut(entity)
    .insert(D)
    .insert_bundle(SomeBundle::default());

// `get_X` 变体返回 Option，以便你想检查存在性而不是 panic
world.get_entity_mut(entity)
    .unwrap()
    .insert(E);

if let Some(entity_ref) = world.get_entity(entity) {
    let d = entity_ref.get::<D>().unwrap();
}
```

`Commands` 也已更新为使用这种新模式

```rust
let entity = commands.spawn()
    .insert(A)
    .insert_bundle((B, C))
    .insert_bundle(SomeBundle::default())
    .id();
```

`Commands` 仍然支持使用 Bundle 生成，这应该使从 **Bevy 0.4** 迁移变得更容易。它还在某些情况下减少了样板代码：

```rust
commands.spawn_bundle(SomeBundle::default());
```

请注意，这些 Command 方法使用了"类型状态"模式，这意味着这种链式风格不再可能：

```rust
// 生成两个实体，每个都带有 SomeBundle 中的组件和 A 组件
// 在 Bevy 0.4 中有效，但在 Bevy 0.5 中无效
commands
    .spawn(SomeBundle::default())
    .insert(A)
    .spawn(SomeBundle::default())
    .insert(A);
```

相反，你应该这样做：

```rust
commands
    .spawn_bundle(SomeBundle::default())
    .insert(A);
commands
    .spawn_bundle(SomeBundle::default())
    .insert(A);
```

这使我们能够使"实体 id 检索"等操作不会失败，并为未来的 API 改进打开了大门。

### Query::single [#](https://bevy.org/news/bevy-0-5/#query-single)

作者：@TheRawMeatball

查询现在有 [`Query::single`](https://docs.rs/bevy/0.5.0/bevy/ecs/system/struct.Query.html#method.single) 和 [`Query::single_mut`](https://docs.rs/bevy/0.5.0/bevy/ecs/system/struct.Query.html#method.single_mut) 方法，如果_恰好_有一个匹配的实体，则返回单个查询结果：

```rust
fn system(query: Query<&Player>) {
    // 只有当恰好有一个 Player 时才返回 Ok
    if let Ok(player) = query.single() {
    }
}
```

### 移除 ChangedRes [#](https://bevy.org/news/bevy-0-5/#removed-changedres)

作者：@TheRawMeatball

我们移除了 `ChangedRes<A>`，改为使用以下方式：

```rust
fn system(a: Res<A>) {
    if a.is_changed() {
        // 做一些事情
    }
}
```

### 可选资源查询 [#](https://bevy.org/news/bevy-0-5/#optional-resource-queries)

作者：@jamadazi

现在系统可以通过 `Option` 查询来检查 Resource 是否存在：

```rust
fn system(a: Option<Res<A>>) {
    if let Some(a) = a {
        // 做一些事情
    }
}
```

### 新的 Bundle 命名约定 [#](https://bevy.org/news/bevy-0-5/#new-bundle-naming-convention)

组件 Bundle 以前使用 `XComponents` 命名约定（例如：`SpriteComponents`、`TextComponents` 等）。我们决定转向 `XBundle` 命名约定（例如：`SpriteBundle`、`TextBundle` 等），以更明确地说明这些类型是什么，并帮助防止新用户混淆 Bundles 和 Components。

### World 元数据改进 [#](https://bevy.org/news/bevy-0-5/#world-metadata-improvements)

作者：@cart

`World` 现在有可查询的 `Components`、`Archetypes`、`Bundles` 和 `Entities` 集合：

```rust
// 你可以像访问任何其他 SystemParam 一样从普通系统访问这些新集合
fn system(archetypes: &Archetypes, components: &Components, bundles: &Bundles, entities: &Entities) {
}
```

这使开发者能够从系统中访问内部 ECS 元数据。

### 可配置的 SystemParams [#](https://bevy.org/news/bevy-0-5/#configurable-systemparams)

作者：@cart、@DJMcNab

用户现在可以为系统参数提供一些初始配置/值（在可能的情况下）。大多数 SystemParam 没有配置（配置类型为 `()`），但 `Local<T>` 参数现在支持用户提供的参数：

```rust
fn foo(value: Local<usize>) {    
}

app.add_system(foo.system().config(|c| c.0 = Some(10)));
```

### 为脚本支持做准备 [#](https://bevy.org/news/bevy-0-5/#preparation-for-scripting-support)

作者：@cart

Bevy ECS 组件现在与 Rust 类型解耦。新的 `Components` 集合存储内存布局和析构函数等元数据。组件也不再需要 Rust TypeIds。

新的组件元数据可以随时使用 `world.register_component()` 添加。

所有组件存储类型（目前是 Table 和 Sparse Set）都是"blob 存储"。它们可以存储具有给定内存布局的任何值。这使得来自其他来源（例如：Python 数据类型）的数据可以与 Rust 数据类型以相同的方式存储和访问。

我们还没有完全启用脚本支持（[并且可能永远不会官方支持非 Rust 脚本](https://discord.com/channels/691052431525675048/692648082499829760/817178225791729716)），但这是迈向社区支持脚本语言的重要一步。

### 将 Resources 合并到 World 中 [#](https://bevy.org/news/bevy-0-5/#merged-resources-into-world)

作者：@cart

Resources 现在只是一种特殊类型的 Component。这允许我们通过复用现有的 Bevy ECS 内部实现来保持代码体积小。它还使我们能够优化并行执行器的访问控制，并且应该使未来的脚本语言集成变得更容易。

```rust
world.insert_resource(1);
world.insert_resource(2.0);
let a = world.get_resource::<i32>().unwrap();
let mut b = world.get_resource_mut::<f64>().unwrap();
*b = 3.0;

// 在系统中资源仍然以相同方式访问
fn system(foo: Res<f64>, bar: ResMut<i32>) {
}
```

_但是_这个合并对直接与 `World` 交互的人造成了问题。如果你需要同时对多个资源进行可变访问怎么办？`world.get_resource_mut()` 可变借用 World，这阻止了多个可变访问！我们用 `WorldCell` 解决了这个问题。

### WorldCell [#](https://bevy.org/news/bevy-0-5/#worldcell)

作者：@cart

WorldCell 将系统使用的"访问控制"概念应用于直接 world 访问：

```rust
let world_cell = world.cell();
let a = world_cell.get_resource_mut::<i32>().unwrap();
let b = world_cell.get_resource_mut::<f64>().unwrap();
```

这添加了廉价的运行时检查以确保 world 访问不会相互冲突。

我们将此设为单独的 API，以使用户能够决定他们想要的权衡。直接 World 访问具有更严格的生命周期，但它更高效且在编译时进行访问控制。`WorldCell` 具有更宽松的生命周期，但因此会产生_小的_运行时开销。

该 API 目前仅限于资源访问，但未来将扩展到查询/实体组件访问。

### 资源作用域 [#](https://bevy.org/news/bevy-0-5/#resource-scopes)

作者：@cart

WorldCell 尚不支持组件查询，即使支持，有时也会有正当理由需要同时持有可变 world 引用_和_可变资源引用（例如：bevy_render 和 bevy_scene 都需要这个）。在这些情况下，我们总是可以降级使用不安全的 `world.get_resource_unchecked_mut()`，但这并不理想！

相反，开发者可以使用"资源作用域"

```rust
world.resource_scope(|world: &mut World, mut a: Mut<A>| {
})
```

这会临时从 `World` 中移除 `A` 资源，为两者提供可变指针，并在完成后将 A 重新添加到 World。感谢迁移到 ComponentIds/sparse sets，这是一个廉价的操作。

如果需要多个资源，作用域可以嵌套。如果这种模式变得普遍且样板代码变得难以忍受，我们也可以考虑向 API 添加"资源元组"。

### 查询冲突使用 ComponentId 而不是 ArchetypeComponentId [#](https://bevy.org/news/bevy-0-5/#query-conflicts-use-componentid-instead-of-archetypecomponentid)

作者：@cart

出于安全原因，系统不能包含相互冲突的查询而不将它们包装在 `QuerySet` 中。在 **Bevy 0.4** 中，我们使用 `ArchetypeComponentIds` 来确定冲突。这很好，因为它可以考虑过滤器：

```rust
// 这些查询由于其过滤器永远不会冲突
fn filter_system(a: Query<&mut A, With<B>>, b: Query<&mut B, Without<B>>) {
}
```

但它也有一个明显的缺点：

```rust
// 这些查询_直到_一个具有 A、B 和 C 的实体被生成才会冲突
fn maybe_conflicts_system(a: Query<(&mut A, &C)>, b: Query<(&mut A, &B)>) {
}
```

上面的系统如果生成了一个具有 A、B 和 C 的实体，将在运行时 panic。这使得很难相信你的游戏逻辑会在不崩溃的情况下运行。

在 **Bevy 0.5** 中，我们切换到使用 `ComponentId` 而不是 `ArchetypeComponentId`。这_确实_更受约束。`maybe_conflicts_system` 现在总是会失败，但它会在启动时一致地失败。

朴素地，它也会_禁止_ `filter_system`，这将是对可用性的重大降级。Bevy 有许多内部系统依赖于不相交的查询，我们预计这在用户空间中会是一种常见模式。为了解决这个问题，我们添加了一个新的内部 `FilteredAccess<T>` 类型，它包装 `Access<T>` 并添加了 with/without 过滤器。如果两个 `FilteredAccess` 的 with/without 值证明它们是不相交的，它们将不再冲突。

这意味着 `filter_system` 在 **Bevy 0.5** 中仍然完全有效。我们获得了旧实现的大部分好处，但在应用启动时强制执行一致且可预测的规则。

## Bevy 的下一步是什么？[#](https://bevy.org/news/bevy-0-5/#what-s-next-for-bevy)

我们仍然有很长的路要走，但 Bevy 开发者社区正在快速成长，我们已经对未来看到了大计划。期待在以下领域很快取得进展：

- "流水线化"渲染和其他渲染器优化
- Bevy UI 重新设计
- 动画：组件动画和 3D 骨骼动画
- ECS：关系/索引、异步系统、原型不变量、"无阶段"系统调度
- 3D 光照功能：阴影、更多光照类型
- 更多 Bevy Scene 功能和可用性改进

我们还计划在确定最终 Bevy UI 设计后立即开始构建 Bevy 编辑器。
