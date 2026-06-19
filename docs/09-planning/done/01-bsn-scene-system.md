# BSN (Bevy Scene Notation) 场景系统

## 1. 概述

BSN 是 Bevy 0.19 最大的新特性，是一种类 Rust 的场景语法，可通过 `bsn!` 宏在 Rust 代码中定义，也可在 `.bsn` 资产文件中使用。它让复杂实体集合的生成变得简洁。

BSN 的核心定位是**下一代 Spawn DSL**——不是 ECS 核心，而是构建在 ECS 之上的声明式实体生成语法。它将"实体长什么样"与"实体怎么生成"分离，使场景定义更接近设计意图。

### 核心价值

| 维度 | 传统 spawn | BSN |
|------|-----------|-----|
| 语法风格 | 命令式（过程代码） | 声明式（结构描述） |
| 可组合性 | 需手动组合 Bundle | 场景天然可层叠组合 |
| 可复用性 | 函数封装 | Scene Function + SceneComponent |
| 资产引用 | 显式 `asset_server.load()` | Template 自动解析 |
| UI 生成 | 冗长的 NodeBundle | 简洁的嵌套语法 |

## 2. 核心概念

### 2.1 bsn! 宏基础

`bsn!` 是 BSN 的核心入口，语法类似结构体初始化，但更简洁：

```rust
// 基本语法：指定组件类型和字段
bsn! { ComponentType { field: value } }

// 可选字段：不需要指定每个字段，未指定的使用默认值
bsn! { Node { width: px(200) } }  // height 等字段自动 default

// 仅类型名：等同于所有字段默认
bsn! { Player }

// 表达式值：用花括号包裹任意 Rust 表达式
bsn! { Player { score: {current_points + 10} } }
```

**关键点**：
- `bsn!` 内部的字段语法不是 Rust 结构体字面量，而是 BSN 自己的 DSL
- 未指定的字段自动使用 `Default::default()`，无需手动写 `..default()`
- 表达式值用 `{}` 包裹，与 BSN 的 `{}` 结构语法区分

### 2.2 BSN Relationships

BSN 原生支持 Bevy 的关系系统，用 `[]` 表示子实体列表：

```rust
// Children 语法：标准父子关系
bsn! {
    Player,
    Children [
        Sword,
        Shield
    ]
}

// 自定义关系：不限于 Children，任何 Relationship 都可以
bsn! {
    Player,
    Inventory [
        Apple,
        Potion
    ]
}
```

**与 `children!` 宏的区别**：
- `children!` 是 0.18 的宏，生成 `Vec<Entity>`，只能用于 `Children` 关系
- BSN `[]` 语法更通用，支持任意 Relationship 类型
- BSN `[]` 内部直接写组件/场景，不需要 `commands.spawn()` 包裹

### 2.3 Scene Functions

Scene Function 是可复用的场景生成函数，返回 `impl Scene`：

```rust
// 无参数场景函数
fn player() -> impl Scene {
    bsn! {
        Player,
        Children [ Sword, Shield ]
    }
}

// 带参数场景函数
fn player(name: &str) -> impl Scene {
    bsn! {
        Name(name),
        Player
    }
}

// 使用
fn setup(mut commands: Commands) {
    commands.spawn_scene(player());
    commands.spawn_scene(player("Hero"));
}
```

**设计要点**：
- 返回类型是 `impl Scene`，不是具体类型
- 参数可以是任意 Rust 类型，在 `bsn!` 内直接使用
- Scene Function 是零成本抽象，编译期展开

### 2.4 Scenes are Composable Patches

场景是**可组合的补丁（Patch）**，不是完整实例。这是 BSN 最强大的特性之一：

```rust
// 基础场景函数
fn button() -> impl Scene {
    bsn! {
        Node { width: px(200), height: px(50) },
        Button
    }
}

// 层叠场景：在基础场景上覆盖字段
fn my_button() -> impl Scene {
    bsn! {
        button(),           // 基础场景
        Node { height: px(100) }  // 覆盖 height，width 保持 200
    }
}
```

**补丁语义**：
- 多个场景按顺序层叠
- 后续场景的字段覆盖前面场景的默认值
- 未指定的字段保留前一个场景的值
- 这使得"基础模板 + 变体"模式非常自然

### 2.5 Scene Assets and Caching

BSN 支持从 `.bsn` 资产文件加载场景，并提供缓存机制：

```rust
// queue_spawn_scene：排队加载，适合 Startup
app.add_systems(Startup, |mut commands: Commands, asset_server: Res<AssetServer>| {
    commands.queue_spawn_scene(asset_server.load("player.bsn"));
});

// spawn_scene：立即生成，适合已加载的场景
commands.spawn_scene(my_scene());

// 缓存语法 `:`：引用 .bsn 文件并覆盖部分字段
bsn! {
    :"player.bsn",          // 加载 player.bsn 作为基础
    Transform { ... }       // 覆盖 Transform
}
```

**重要限制**：
- **0.19 不附带官方 `.bsn` 资产加载器**，需要社区或自行实现
- 缓存语法 `:` 依赖资产加载器，目前仅代码驱动工作流可用
- `queue_spawn_scene` vs `spawn_scene`：前者异步等待资产加载，后者同步生成

### 2.6 Scene Lists

`bsn_list!` / `SceneList` 用于生成多个独立实体：

```rust
// bsn_list!：一次生成多个实体
bsn_list! [
    Player,
    Enemy,
    Npc
]

// 实体引用语法 #Name：在列表内引用其他实体
bsn_list! [
    #Hero,
    #Villain,
    Duel(Hillside, #Hero, #Villain)  // 引用上面定义的实体
]
```

**与 `bsn!` 的区别**：
- `bsn!` 生成单个实体（可以有 Children）
- `bsn_list!` 生成多个平级实体
- `#Name` 语法在 `bsn_list!` 中用于跨实体引用

### 2.7 Observing Events in BSN

BSN 支持内联事件观察者，用 `on()` 语法：

```rust
bsn! {
    Button,
    on(|press: On<Pointer<Press>>| {
        info!("pressed!");
    })
}
```

**适用场景**：
- UI 交互（点击、悬停）
- 简单的事件响应
- 不适合复杂业务逻辑（应放在 System 中）

**设计原则**：
- `on()` 适合"结构相关"的简单响应
- 业务逻辑仍应放在 System 中，通过事件/资源通信
- 不要让 BSN 场景变成"逻辑脚本"

### 2.8 Templates

Template 是组件的"高级构造器"，可访问 World、当前实体、场景生成上下文：

```rust
// FromTemplate trait：大多数实现 Default+Clone 的类型自动获得
// 无需手动实现

// 需要手动 derive FromTemplate 的场景：
// 如 Sprite 的 Handle<Image> 模板接受 "player.png" 字符串
#[derive(FromTemplate)]
struct SpriteTemplate {
    image: String,  // BSN 中写 "player.png"，Template 自动转为 Handle<Image>
}

// 使用
bsn! { Sprite { image: "player.png" } }
// Template 将 "player.png" 转为 Handle<Image>
```

**自动 vs 手动 FromTemplate**：
- `Default + Clone` 类型：自动获得 `FromTemplate`，无需任何代码
- 需要资产加载或类型转换：手动 `#[derive(FromTemplate)]` 并实现转换逻辑
- Template 是 BSN 资产引用的底层机制

### 2.9 Inline Asset Templates

`asset_value()` 语法用于内联创建资产值：

```rust
bsn! {
    Mesh3d(asset_value(Cuboid::new(1., 1., 1.))),
    MeshMaterial3d(asset_value(StandardMaterial {
        base_color: Color::srgb(0.8, 0.2, 0.2),
        ..default()
    }))
}
```

**适用场景**：
- 3D 网格和材质的内联创建
- 不需要单独声明 Asset 变量
- 注意：本项目是 2D 战棋，此特性主要用于 3D 场景

### 2.10 Entity Reference Syntax

`#Name` 语法用于定义和引用实体：

```rust
// 定义命名实体
bsn! {
    #Root,
    Children [
        Reference(#Root)  // 引用 Root 实体
    ]
}

// 图结构：双向引用
bsn_list! [
    (#A PointsTo(#B)),
    (#B PointsTo(#A))
]
```

**语义**：
- `#Name` 在首次出现时定义实体，后续出现时引用同一实体
- 支持图结构（非树结构），突破 Children 的树形限制
- 实体引用在场景生成时解析为实际的 `Entity` ID

### 2.11 Implicit Into

字段位置支持隐式 `.into()` 转换，减少显式类型标注：

```rust
// &str 自动 Into<String>
bsn! { Foo("hello") }

// UI 场景：Val 自动 Into UiRect
bsn! { Node { border: px(2) } }
// 等价于 border: UiRect::all(Val::Px(2.0))

// 数字类型转换
bsn! { Transform { translation: (1.0, 2.0, 3.0) } }
```

**设计意图**：
- BSN 的目标是"写起来像配置文件"
- 隐式 Into 减少样板代码
- 但不牺牲类型安全——转换必须通过 `Into` trait

### 2.12 Scene Components

SceneComponent 是 BSN 的高级特性，将 Scene 关联到 Component：

```rust
// SceneComponent derive：将 Scene 关联到 Component
#[derive(SceneComponent)]
#[scene("player.bsn")]  // 场景资产简写
struct Player {
    score: u32,
}

// @SceneComponent 语法：生成时自动附带完整场景
bsn! { @Player { score: 10 } }
// 等价于：spawn Player 组件 + "player.bsn" 定义的场景

// Props 机制：传递参数到场景函数
#[derive(SceneComponent)]
#[scene(PlayerProps)]  // 将 PlayerProps 传递给场景函数
struct Player {
    score: u32,
}
```

**保证**：
- 如果 `Player` 组件存在，完整场景也一定存在
- 必须通过 `spawn_scene` 方式生成，直接 `spawn` 会报错
- SceneComponent = Component + Scene 的绑定契约

**当前评估**：
- SceneComponent 更适合编辑器时代（可视化关联场景与组件）
- 代码驱动开发中，Scene Function 已足够

### 2.13 Scene Spawning Systems

`.spawn()` 方法将场景函数转为 Startup 系统：

```rust
fn level() -> impl Scene {
    bsn! { /* ... */ }
}

// 方式一：转为 Startup 系统
app.add_systems(Startup, level.spawn());

// 方式二：在 System 中手动调用
fn setup(mut commands: Commands) {
    commands.spawn_scene(level());
}
```

**两种方式的区别**：
- `.spawn()` 更简洁，适合纯声明式场景
- `commands.spawn_scene()` 更灵活，适合需要运行时逻辑的场景
- 本项目建议使用 `commands.spawn_scene()`，保持显式控制

## 3. 对本项目的影响评估

### 3.1 立即收益场景

| 场景 | 收益 | 说明 |
|------|------|------|
| UI 层 | **高** | `bsn! { Node, Children [ Text("Attack") ] }` 比原 NodeBundle 写法简洁得多 |
| AI 代码生成 | **中** | BSN 语法树更简单，AI 生成 UI 成功率更高 |
| 场景描述 | **中** | 剧情场景、地图装饰、NPC 预制体的声明式定义 |

**UI 层收益详解**：

当前 0.18 的 UI 生成代码冗长且嵌套深：
```rust
// 0.18：一个简单按钮需要 10+ 行
commands.spawn((
    Node { width: Val::Px(200.0), height: Val::Px(50.0), ..default() },
    Button,
    children![(
        Text::new("Attack"),
        TextFont { font_size: 24.0, ..default() },
    )],
));
```

BSN 将其压缩到 4 行，且结构更清晰：
```rust
// 0.19 BSN：同样功能，4 行
commands.spawn_scene(bsn! {
    Node { width: px(200), height: px(50) }
    Button
    Children [ Text("Attack") ]
});
```

### 3.2 暂不采用的场景

| 场景 | 原因 |
|------|------|
| 核心玩法层（Character/Ability/Buff/Effect） | 现有 `commands.spawn()` 写法足够，重构收益低，风险高 |
| SceneComponent | 等编辑器时代再评估，当前代码驱动不需要 |
| .bsn 资产文件 | 0.19 无官方加载器，等生态成熟 |
| on() 事件观察者 | 业务逻辑应放在 System 中，不在场景定义中 |

**核心玩法层不采用 BSN 的理由**：
1. Character/Ability/Buff/Effect 的生成逻辑涉及大量运行时计算（属性推导、条件判断）
2. 这些实体的组件组合由 Domain 规则决定，不适合声明式定义
3. 重构成本高、收益低——现有代码可读性已足够
4. 违反"BSN 负责结构，System 负责行为，Domain 负责规则"的分层原则

### 3.3 风险警示

| 风险 | 等级 | 缓解措施 |
|------|------|----------|
| 业务逻辑泄漏到场景定义 | **高** | 代码审查强制检查 BSN 场景中不包含逻辑 |
| 全项目 BSN 化冲动 | **高** | 明确边界：仅 UI 层使用，核心玩法层禁止 |
| .bsn 加载器缺失 | **中** | 代码驱动先行，不依赖资产文件 |
| FromTemplate 自动派生的隐式行为 | **低** | 文档化自动派生规则，手动派生需注释说明 |
| SceneComponent 误用 | **中** | 当前不采用，等编辑器时代再评估 |

**红线**：
- **不要让业务逻辑进入场景定义**（BSN 负责结构，System 负责行为，Domain 负责规则）
- **不要全项目 BSN 化**，尤其不要重构 Character/Ability/Buff/Effect 为 BSN
- **Bundle → Scene → BSN 是未来路线，但不是现在**

## 4. 迁移步骤

### 4.1 第一阶段：不引入 BSN

**目标**：纯兼容迁移到 0.19，所有 spawn 代码保持原样。

**具体操作**：
- 所有 `commands.spawn()` 调用保持不变
- Bundle 写法不变，0.19 仍支持传统 spawn
- 关注 0.19 中 Bundle/Children API 的废弃通知，记录但不立即行动
- 验证现有 spawn 代码在 0.19 下正常工作

**完成标志**：项目在 0.19 下编译通过，所有测试通过，无 BSN 相关代码。

### 4.2 第二阶段：UI 层试点

**目标**：新增 UI 代码使用 `bsn!` 写法，旧 UI 代码不重构。

**具体操作**：
- 新增的 UI 组件/界面使用 `bsn!` 宏生成
- 旧 UI 代码保持原样，不主动重构
- 建立 UI 场景函数库（如 `fn action_button() -> impl Scene`）
- 验证 BSN UI 与传统 spawn UI 的互操作性

**试点范围**：
- 战斗 UI（技能按钮、状态栏）
- 菜单 UI（主菜单、设置菜单）
- HUD（回合指示器、小地图）

**不涉及**：
- Character/Ability/Buff/Effect 的生成
- 地图/地形实体
- 任何非 UI 的 spawn 代码

**完成标志**：新增 UI 代码 100% 使用 BSN，旧 UI 代码不变，项目编译测试通过。

### 4.3 未来：场景描述

**目标**：剧情场景、地图装饰、NPC 预制体逐步 BSN。

**前置条件**：
- 官方 `.bsn` 资产加载器发布
- UI 层 BSN 试点稳定运行
- 编辑器需求明确

**可能的方向**：
- 剧情场景定义（角色站位、对话触发点、特效位置）
- 地图装饰预制体（树木、建筑、特效）
- NPC 预制体（商人、NPC 对话触发器）
- 关卡配置（敌人配置、掉落表、胜利条件）

**不在当前计划中**：SceneComponent、.bsn 资产文件、on() 事件观察者。

## 5. 代码示例对比

### 5.1 UI 生成对比

**旧写法（0.18）**：

```rust
commands.spawn((
    Node {
        width: Val::Px(200.0),
        height: Val::Px(50.0),
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    },
    Button,
    children![(
        Text::new("Attack"),
        TextFont { font_size: 24.0, ..default() },
    )],
));
```

**新写法（0.19 BSN）**：

```rust
commands.spawn_scene(bsn! {
    Node { width: px(200), height: px(50), border: px(2) }
    Button
    Children [
        Text("Attack")
    ]
});
```

**差异分析**：
- 行数：10+ 行 → 4 行
- `Val::Px(200.0)` → `px(200)`：BSN 提供便捷函数
- `UiRect::all(Val::Px(2.0))` → `px(2)`：Implicit Into 自动转换
- `..default()` 不再需要：BSN 自动填充默认值
- `children![]` → `Children []`：BSN 原生关系语法
- `Text::new("Attack")` → `Text("Attack")`：Implicit Into

### 5.2 带 Asset 的场景对比

**旧写法（0.18）**：

```rust
fn player(asset_server: &AssetServer) -> impl Bundle {
    (
        Player { score: 10, ..Default::default() },
        children![Sprite { image: asset_server.load("player.png"), ..Default::default() }]
    )
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(player(&asset_server));
}
```

**新写法（0.19 BSN）**：

```rust
fn player() -> impl Scene {
    bsn! {
        Player { score: 10 },
        Children [ Sprite { image: "player.png" } ]
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_scene(player());
}
```

**差异分析**：
- 不再需要 `Res<AssetServer>` 参数：Template 自动处理资产加载
- `"player.png"` 字符串自动转为 `Handle<Image>`：FromTemplate 机制
- `..Default::default()` 不再需要
- Scene Function 签名更简洁：无需传递 AssetServer

### 5.3 可组合 UI 对比

**旧写法（0.18）**：需要手动组合 Bundle

```rust
fn action_button(label: &str) -> impl Bundle {
    (
        Node {
            width: Val::Px(200.0),
            height: Val::Px(50.0),
            ..default()
        },
        Button,
        children![(
            Text::new(label.to_string()),
            TextFont { font_size: 24.0, ..default() },
        )],
    )
}

fn wide_button(label: &str) -> impl Bundle {
    (
        Node {
            width: Val::Px(400.0),  // 只改了 width
            height: Val::Px(50.0),
            ..default()
        },
        Button,
        children![(
            Text::new(label.to_string()),
            TextFont { font_size: 24.0, ..default() },
        )],
    )
}
```

**新写法（0.19 BSN）**：场景补丁天然可组合

```rust
fn action_button() -> impl Scene {
    bsn! {
        Node { width: px(200), height: px(50) }
        Button
    }
}

fn wide_button() -> impl Scene {
    bsn! {
        action_button(),              // 复用基础场景
        Node { width: px(400) }       // 只覆盖 width
    }
}
```

**差异分析**：
- 旧写法需要复制整个 Bundle 定义，只改一个字段
- BSN 补丁模式：基础场景 + 覆盖字段，零重复
- 修改 `action_button` 的样式时，`wide_button` 自动继承

## 6. 注意事项

### 6.1 当前限制

| 限制 | 说明 | 影响 |
|------|------|------|
| 无官方 .bsn 加载器 | 0.19 不附带 `.bsn` 资产加载器 | 代码驱动工作流先行，不依赖资产文件 |
| FromTemplate 自动派生 | `Default + Clone` 类型自动获得，但需要模板功能时需手动 derive | 需文档化哪些类型需要手动 derive |
| SceneComponent 必须用 spawn_scene | 直接 `spawn` 会报错 | 代码审查需检查生成方式 |
| BSN 不是 ECS 核心 | BSN 是 Spawn DSL，不影响 ECS 架构 | 不改变现有 System/Query/Command 模式 |

### 6.2 最佳实践

1. **BSN 负责结构，System 负责行为，Domain 负责规则**
   - BSN 场景只定义"实体长什么样"
   - System 处理"实体怎么行为"
   - Domain 规则决定"实体应该是什么"

2. **渐进式采用，不强制迁移**
   - 新代码用 BSN，旧代码保持原样
   - 不为"统一风格"而重构已有代码

3. **Scene Function 优于内联 bsn!**
   - 可复用的场景提取为 Scene Function
   - 一次性场景可以内联 `bsn!`

4. **避免在 BSN 中写逻辑**
   - `on()` 只用于简单 UI 反馈
   - 业务逻辑放在 System 中
   - 条件生成放在 System 中，不在 Scene Function 中

5. **UI 层是 BSN 的最佳切入点**
   - UI 组件天然是声明式的
   - UI 组件组合频繁，补丁模式收益大
   - UI 代码量大，BSN 的简洁性收益明显

### 6.3 与项目架构的关系

```
┌─────────────────────────────────────────────┐
│                  UI 层                       │
│  BSN 试点区域：新增 UI 使用 bsn! 写法       │
│  Scene Function 库：可复用 UI 组件           │
├─────────────────────────────────────────────┤
│              核心玩法层                       │
│  禁止 BSN：Character/Ability/Buff/Effect    │
│  保持 commands.spawn() 写法                  │
├─────────────────────────────────────────────┤
│              基础设施层                       │
│  不涉及：System/Query/Command 不变           │
│  BSN 不改变 ECS 架构                         │
└─────────────────────────────────────────────┘
```

BSN 是 UI 层的工具，不是架构变革。它不改变项目的 ECS 架构、领域模型、数据流。采用 BSN 是"在 UI 层用更好的工具"，不是"改变项目架构"。
