# Bevy Reflect 深度解析 — 类型之镜

> 如果你写过 `#[derive(Reflect)]` 但不确定它在 Bevy 里到底是干什么的——你并不孤独。Reflect 是 Bevy 的一个"隐形基础设施"：大部分时间你碰不到它，但编辑器、存档系统、调试面板、UI 数据绑定全都悄悄依赖它。本文从 Fre 50 万行代码的实际使用出发，解释 Reflect 是什么、为什么重要、怎么正确使用。

---

## 1. Reflect 是什么？

Bevy Reflect 提供的是**运行时的类型信息**。Rust 是一门编译期类型系统极其强大的语言，但有一个天然短板：编译完成后，所有类型信息都消失了。你不能在运行时问"这个 struct 有哪些字段？"、"这个 enum 的当前变体是什么？"、"把这个 struct 序列化成 JSON"。

Reflect 就是来补这个短板的。它通过 derive 宏，在编译期为每个类型生成一张"类型元数据表"，运行时可以通过这张表来做：

- 遍历 struct 的所有字段（名字、类型、值）
- 动态创建/修改类型的实例
- 动态调用方法（有限支持）
- 反射驱动的序列化与反序列化

打个比方：**Reflect 是 Rust 类型系统的"镜子"——运行时还能看到自己长什么样。**

---

## 2. 为什么 Fre 需要 Reflect？

一个 50 万行的 SRPG 项目，如果没有 Reflect，下面这些事会变得极其困难：

### 2.1 存档系统（Save）

存档就是把游戏内所有需要持久化的数据序列化成文件。没有 Reflect，你必须在每个 struct 上手写序列化/反序列化代码：

```rust
// ❌ 没有 Reflect：每新增一个字段都必须更新序列化代码
impl SaveGame {
    fn serialize(&self) -> Vec<u8> { /* 几百行模式相同的代码 */ }
    fn deserialize(data: &[u8]) -> Self { /* 同样几百行 */ }
}

// ✅ 有 Reflect + bevy_reflect_serde：derive + register_type 就够了
#[derive(Reflect, Serialize, Deserialize)]
struct SaveGame { /* ... */ }
```

### 2.2 回放系统（Replay）

Replay 录制的是命令，但命令的参数五花八门：Entity、坐标、ID、枚举……有了 Reflect，可以在不知道具体类型的情况下统一处理序列化。

### 2.3 调试工具/编辑器

Bevy Editor 和 `bevy-inspector-egui` 依赖 Reflect 来生成属性面板。一个没有 Reflect 的类型，在编辑器里就是一个黑洞——看不到字段、改不了值。

### 2.4 UI 数据绑定

用 Dirty 机制实现 ViewModel → Widget 的自动刷新时，Reflect 提供的类型注册是基础设施。

### 2.5 运行时审计

某些调试场景下，你想遍历 World 中所有持有某类 Component 的 Entity——Reflect 帮你做什么类型是动态的。

---

## 3. 宪法的 Reflect 规则

Fre 项目对 Reflect 的使用有两层约束：

### 规则一：所有 Component/Event/Resource 必须 derive Reflect

来自宪法 §18（Behvior 0.19 升级变更）：

> 🟩 **所有 Component/Event/Resource 类型必须 derive Reflect**

这条规则是 Bevy 0.19 迁移时新增的。之前的代码可能只在需要的地方加 Reflect，迁移后变成了**硬性要求**。原因很简单：

- Save 系统需要对任意 Entity 序列化它携带的所有 Component——这要求所有 Component 都是 Reflect
- Replay、Debug、Editor 都有类似需求
- "以后再加"等于"永远不加"——先全部加上，后续裁剪比补缺容易

具体做法：

```rust
// Component 必须
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

// Resource 必须
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct GameState {
    pub phase: GamePhase,
}
```

注意两点：
1. 光 `#[derive(Reflect)]` 不够，还要加 `#[reflect(Component)]` 或 `#[reflect(Resource)]`——这确保类型被注册到 Bevy 的 Reflect 注册表时，系统知道它扮演什么 ECS 角色
2. `#[reflect(Component)]` 需要 struct 本身已经有 `#[derive(Component)]`；`#[reflect(Resource)]` 需要 struct 已经有 `#[derive(Resource)]`

### 规则二：绝对禁止在高频计算中使用 Reflect

来自宪法 §17：

> 🟩 🟥 **绝对禁止在高频计算中使用 Reflect**

这条反过来说就是：Reflect 是给**非性能关键路径**用的。为什么？

看一下 Reflect 访问字段的内部实现：

```rust
// 通过 Reflect 读取某个字段的值
let field = my_struct.field("name").unwrap();
// 这背后是：字符串查找 → 类型分发 → Any 转换 → 值访问
// 比直接 my_struct.name 慢 100~1000 倍
```

所以在战斗计算、AI 决策、寻路算法、Modifier 聚合、效果执行这些每帧跑几百上千次的地方，禁止使用 Reflect。直接使用 Rust 的原生字段访问。

### 规则三：Reflect Boundary（编码规范 §12）

```
Reflect 仅属于工具链层，不属于游戏逻辑层。

允许用于：
  * 编辑器
  * Inspector
  * 配置工具
  * 调试面板

绝对禁止在：
  * 战斗计算
  * AI 决策
  * 寻路算法
  * 属性计算
  * Buff 结算
  * 回合管理
```

本质上，Reflect 是"给开发者用的"，不是"给游戏逻辑用的"。游戏逻辑用具体的 Rust 类型，开发工具用 Reflect。

---

## 4. Fre 中的 Reflect 三件套

在代码层面，Reflect 使用有三个层次，层层递进：

### Layer 1：类型声明——`#[derive(Reflect)]`

这是最基础的一层。每个需要被 Reflect 感知的类型，在定义时加上 derive：

```rust
use bevy::prelude::Reflect;

// 一个值类型
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct ModifierOp(pub f32);

// 一个 ECS Component
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct AttributeContainer {
    pub attributes: HashMap<AttributeId, AttributeValue>,
    pub derived_cache: HashMap<AttributeId, f32>,
}

// 一个 Resource
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct Theme {
    pub name: &'static str,
    pub colors: UiColors,
    pub spacing: UiSpacing,
    pub typography: UiTypography,
}

// 一个 Event
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct BattleStarted {
    pub formation_id: FormationId,
}
```

### Layer 2：`#[reflect(...)]` 属性宏

`#[reflect()]` 告诉 Bevy 这个类型扮演什么 ECS 角色，或者实现什么特性：

| 属性 | 含义 |
|------|------|
| `#[reflect(Component)]` | 这是一个 Component，注册到 ComponentRegistry |
| `#[reflect(Resource)]` | 这是一个 Resource，注册到 ResourceRegistry |
| `#[reflect(Hash, PartialEq)]` | 实现了 Hash + PartialEq（用于集合类） |
| `#[reflect(Clone)]` | 实现了 Clone |
| `#[reflect(ignore)]` | 跳过这个字段，不参与反射 |

使用 `#[reflect(ignore)]` 的场景：

```rust
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct CraftingComponent {
    pub recipe_id: RecipeId,
    pub progress: f32,
    #[reflect(ignore)]
    pub _marker: PhantomData<()>,  // PhantomData 不需要反射
}
```

项目中 `#[reflect(ignore)]` 通常用在三个地方：
- `PhantomData` 字段（如 `InstanceId<T>` 中的泛型标记）
- `ComponentLink` 或 `Entity` 类型的外部引用（反射序列化时需特殊处理）
- 运行时特有的非序列化状态（如连接池、文件句柄）

### Layer 3：`app.register_type::<T>()`——类型注册

光 derive 是不够的。Bevy 需要一个显式的注册步骤，把类型信息挂载到全局 ReflectRegistry：

```rust
impl Plugin for MyWidgetPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<MyWidgetState>()
            .register_type::<MyWidgetAction>();
    }
}
```

每个 Plugin 对自己的 Component/Event/Resource 负注册责任。Bevy 推荐"就近注册"：在定义类型的 Plugin 中注册。

Fre 为了减少样板代码，提供了一个批量注册宏：

```rust
// 自动展开为：app.register_type::<Experience>()
//                    .register_type::<ClassLevels>()
//                    .register_type::<TalentTree>()
register_domain_types!(app, [
    Experience,
    ClassLevels,
    TalentTree,
    SubclassChoice,
    ProgressionMarker,
]);
```

---

## 5. Reflect 在 Save 系统中的角色

存档系统是 Reflect 最重要的消费者之一。过程大致如下：

```
保存时：
  获取 Entity 的所有 Component
  → 通过 Reflect 遍历每个 Component 的字段
  → 用 bevy_reflect_serde::Serialization 序列化
  → 写入文件

加载时：
  从文件读取序列化数据
  → 用 bevy_reflect_serde::Deserialization 反序列化
  → 通过 Reflect 动态创建 Component 实例
  → 附加到 Entity
```

这里 Reflect 不可或缺的原因：**存档时你不知道运行时要存哪些 Component。** 不同 Entity 携带的 Component 集合完全不同——一个战斗中 Entity 有 `Health`、`AttributeContainer`、`AbilitySet`；一个剧情 Entity 有 `DialogueComponent`、`StoryFlag`。没有 Reflect 的运行时类型信息，无法统一处理这些异构类型。

保存时只使用 Reflect 的序列化能力（读取字段值），加载时使用 Reflect 的实例化能力（按类型名创建实例）。核心游戏逻辑全程不碰 Reflect。

---

## 6. Reflect 在 ID 系统中的角色

Fre 的强类型 ID 系统（`define_string_id!` 和 `define_numeric_id!` 宏）自动为每个 ID 类型 derive Reflect：

```rust
// define_string_id! 展开后自动包含：
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
#[reflect(Hash, PartialEq)]
pub struct AbilityId(pub String);
```

这意味着所有 ID 类型——`AbilityId`、`EffectId`、`TagId`、`BuffId`、`ItemId`……——都可以被 Reflect 感知。Save 系统序列化它们时，能正确处理。

---

## 7. Reflect 不干什么

理解了 Reflect 能做什么后，理解它**不做什么**同样重要：

| 误解 | 真相 |
|------|------|
| "Reflect 可以替代 Serde" | 不能。`bevy_reflect_serde` 是 Reflect → Serde 的桥梁，不是替代。两者可以配合使用 |
| "Reflect 是运行时 GC" | 不是。Reflect 不管理内存生命周期，它只提供类型信息 |
| "所有类型都要 Reflect" | 不是。只有 Component/Event/Resource 需要。纯内部计算类型（如 `DamageFormula`、`PathNode`）不需要 |
| "Reflect 与静态类型矛盾" | 不矛盾。Reflect 是 Rust 静态类型的补充——编译期用静态类型保证正确性，运行时用 Reflect 处理异构数据 |
| "Reflect 很重，应该避免" | 要分场合：非热路径（Save/Editor）上 Reflect 的开销完全可以接受；热路径（战斗循环）上才需要避免 |

---

## 8. 整体脉络图

```
                             工具层（开发/调试）
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
              bevy-inspector    CLI 调试工具    运行时审计
                    │               │               │
                    └───────────────┼───────────────┘
                                    │
                            Reflect Registry
                           （运行时类型信息）
                           │    ↑    ↑    ↑
                           │    │    │    │
                    ┌──────┘    │    │    └──────────┐
                    ▼           │    ▼                ▼
              Save 系统     ────┘  Replay 系统    bevy_editor
          (序列化 Component)        (序列化命令)    (属性面板)
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
              #[derive(Reflect)]  #[reflect(Comp)]  register_type
              每个 Component/    类型角色标记       Plugin 中显式注册
              Event/Resource
```

---

## 9. 当前 Reflect 覆盖状态

Fre 项目对 Reflect 的覆盖情况：

| 范畴 | 覆盖度 | 说明 |
|------|--------|------|
| **Core Capabilities Components** | ✅ 全覆盖 | effect/modifier/tag/cue/trigger/aggregator/attribute 全部 Component 已 derive Reflect |
| **Domain Components** | ✅ 全覆盖 | 15 个业务域的 Component 已全部 Reflect |
| **Domain Events** | ✅ 全覆盖 | 跨领域事件（BattleStarted, TurnPhaseChanged 等）已 Reflect |
| **Infra Resources** | ✅ 全覆盖 | Save/Replay/Localization/Logging 等 Resource 已 Reflect |
| **UI Components/Resources** | ✅ 全覆盖 | Theme、所有 Primitives/Widgets/Screens 状态组件已 Reflect |
| **InstanceId<T>** | ✅ 支持 | PhantomData 已 `#[reflect(ignore)]` |
| **ID 类型** | ✅ 全覆盖 | 所有 `define_string_id!` / `define_numeric_id!` 类型自动 Reflect |
| **高频计算禁止** | ✅ 合规 | 战斗计算/AI/寻路等热路径无 Reflect 调用 |
| **register_type 注册** | ⚠️ 大部分完成 | 各 Plugin 已注册自己管辖的类型，但需全量审计确认无遗漏 |
| **reflect(Component/Resource) 属性** | ⚠️ 大部分完整 | 已知某些早期 Component 遗漏了 `#[reflect(Component)]` |

---

## 10. 常见问题

### Q: 什么时候需要加 `#[reflect(Resource)]` 而不是 `#[reflect(Component)]`？
前者给 ECS Resource 用，后者给 ECS Component 用。Resource 是全局唯一的单例，Component 是 Entity 绑定的数据。加错会导致 Bevy 注册失败（Rust 编译器会报错）。

### Q: 为什么 struct 必须同时 derive `Component` 和 `Reflect`？
`Component` 是 ECS 标记（这个类型可以挂到 Entity 上），`Reflect` 是运行时类型信息（这个类型允许反射访问）。两者独立正交——一个 Component 可以没有 Reflect（不推荐），一个 Reflect 类型可以不是 Component。

### Q: 非 Component/Event/Resource 的类型需要 Reflect 吗？
不需要。像 `DamageFormula`、`PathfindingResult`、`AIDecision` 这类纯计算中间值，不进入 ECS 的 Component 生命周期，也不需要序列化，不需要 Reflect。

### Q: 怎么检查某个类型是否已注册 Reflect？
在 Bevy 运行的任意时刻：

```rust
fn debug_system(world: &mut World) {
    let registry = world.resource::<AppTypeRegistry>();
    let reg = registry.read();
    let registered = reg.iter().count();
    info!("已注册 {} 个类型", registered);
}
```

### Q: `#[reflect(Component)]` 和 `register_type::<T>()` 都做同样的事吗？
不是。`#[reflect(Component)]` 是**编译期属性**，告诉 Bevy 的 Reflexive derive 类型可以充当 Component。`app.register_type::<T>()` 是**运行时注册**，把类型信息挂载到全局注册表。两个步骤都要做。

---

*本文覆盖了 Reflect 从宪法规则到代码实现的完整链路。Bevy 官方 Reflect 文档见 `docs/00-governance/bevy-examples-catalog.md` §Reflection 节。宪法规则详见 `docs/00-governance/ai-constitution-complete.md` §17 和 §18，编码规范见 `docs/00-governance/coding-rules.md` §12。*
