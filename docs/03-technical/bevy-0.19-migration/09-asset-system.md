# 资产系统增强

> Bevy 0.19 迁移系列 — 第 9 篇
> 本文档梳理 0.19 中资产系统（Asset System）的三项核心增强，并评估其对 SRPG 项目的迁移影响。

---

## 1. Asset Saving — 运行时资产保存

### 1.1 新特性概述

Bevy 0.19 新增 `save_using_saver`，允许在运行时将资产保存到磁盘。此前 `AssetSaver` 只能在资产处理管线（Asset Processor）中使用，无法在游戏运行时动态保存资产。

这一改动打通了"加载 → 修改 → 保存"的完整闭环，为编辑器和工作流工具提供了底层支撑。

### 1.2 构建 SavedAsset

`SavedAsset` 是对待保存资产的封装，它借用而非拥有资产数据，因此可以在同一个 async block 中完成构建和保存。

**简单资产：**

```rust
let main_asset = InlinedBook {
    lines: vec!["Save me!".to_string(), "Please!".to_string()],
};
let saved_asset = SavedAsset::from_asset(&main_asset);
```

**含子资产的资产：**

当资产内部引用了其他资产（通过 Handle），需要使用 `SavedAssetBuilder` 来声明子资产关系，确保保存时子资产也被正确写入。

```rust
let asset_path: AssetPath<'static> = "my/file/path.whatever".into();
let mut builder = SavedAssetBuilder::new(asset_server.clone(), asset_path.clone());

let subasset_1 = Line("howdy".into());
let subasset_2 = Line("goodbye".into());
let handle_1 = builder.add_labeled_asset_with_new_handle(
    "TheFirstLabel",
    SavedAsset::from_asset(&subasset_1),
);
let handle_2 = builder.add_labeled_asset_with_new_handle(
    "AnotherOne",
    SavedAsset::from_asset(&subasset_2),
);

let main_asset = Book {
    lines: vec![handle_1, handle_2],
};
let saved_asset = builder.build(&main_asset);
```

关键要点：
- `add_labeled_asset_with_new_handle` 会为子资产分配一个新的 Handle，同时记录标签与资产的映射
- `builder.build(&main_asset)` 完成构建，返回包含主资产和所有子资产信息的 `SavedAsset`
- 子资产的 Handle 在构建过程中产生，主资产通过这些 Handle 引用子资产

### 1.3 调用 save_using_saver

```rust
save_using_saver(
    asset_server.clone(),
    &MyAssetSaver::default(),
    &asset_path,
    saved_asset,
    &MySettings::default(),
)
.await
.unwrap();
```

注意事项：
- `save_using_saver` 是异步操作，返回 `impl Future<Output = Result<()>>`
- 通常在 `IoTaskPool::get().spawn(...)` 中启动，避免阻塞主线程
- `asset_server` 需要克隆（`Clone` 开销很低，内部为 `Arc`）
- 第二个参数是实现了 `AssetSaver` trait 的 saver 实例
- 最后一个参数是 saver 对应的 Settings 类型

### 1.4 对项目的价值

| 场景 | 说明 |
|------|------|
| 地图编辑器 | 保存编辑后的地图数据（Tile、地形、部署点等） |
| 技能编辑器 | 保存自定义技能配置（效果链、条件、修饰器） |
| 战斗回放 | 保存回放数据到磁盘，支持回放加载与分享 |
| Mod 工具 | 社区用户保存 Mod 内容，实现自定义扩展 |
| 配置热更新 | 运行时修改配置后持久化，下次启动自动加载 |

**当前阶段评估：** 项目尚处于核心战斗逻辑开发期，暂不需要运行时保存。但这是重要的基础设施，了解其存在有助于在架构层面预留扩展空间。

---

## 2. Handle 序列化

### 2.1 新特性概述

Asset `Handle` 现在可以正确地序列化和反序列化。此前 Handle 的序列化行为未定义，无法可靠地将含 Handle 的数据结构序列化后恢复。

0.19 通过引入 `HandleSerializeProcessor` 和 `HandleDeserializeProcessor` 解决了这个问题，使 Handle 能够在 Reflect 序列化管线中正确流转。

### 2.2 工作原理

**序列化方向（Handle → 数据）：**

`HandleSerializeProcessor` 在序列化时将 Handle 转换为其对应的资产路径（`AssetPath`），存储路径字符串而非 Handle 的内部 ID。

**反序列化方向（数据 → Handle）：**

`HandleDeserializeProcessor` 在反序列化时从路径字符串重新加载资产，生成新的 Handle。如果资产尚未加载，会触发异步加载。

这种设计保证了：
- 序列化结果是自描述的（包含路径信息，而非不可解读的内部 ID）
- 反序列化后 Handle 指向正确的资产实例
- 跨进程、跨会话的序列化兼容性

### 2.3 使用方式

Handle 序列化集成在 Reflect 序列化管线中，通过 `with_processor` 传入对应的 Processor：

```rust
// 序列化（Handle → 路径字符串）
let serializer = TypedReflectSerializer::with_processor(
    &value,
    &registry,
    &HandleSerializeProcessor,
);

// 反序列化（路径字符串 → Handle）
let deserializer = TypedReflectDeserializer::with_processor(
    &mut registry,
    &HandleDeserializeProcessor,
);
```

注意：
- 序列化时使用 `HandleSerializeProcessor`（不可变引用）
- 反序列化时使用 `HandleDeserializeProcessor`（需要可变引用，因为它会注册新加载的 Handle）
- 两者配合使用，确保序列化/反序列化的往返一致性

### 2.4 对项目的价值

| 场景 | 说明 |
|------|------|
| Config 资产引用 | `CharacterConfig` / `SkillConfig` / `BuffConfig` 中直接引用 `Handle<Image>` 等资产，序列化后可正确恢复 |
| 数据驱动架构 | 配置数据中嵌入 Handle 引用不再需要额外的间接层或手动映射 |
| Save/Replay 兼容性 | 存档和回放数据中若包含资产引用，可正确序列化与恢复 |
| 编辑器支持 | 编辑器中修改含 Handle 的配置后，保存/加载行为一致 |

### 2.5 前提条件

Handle 序列化依赖资产类型正确实现 `Reflect`。仅 derive `TypePath` 是不够的：

```rust
// 不够 — 无法参与 Handle 序列化
#[derive(Asset, TypePath)]
struct MyAsset {
    // ...
}

// 需要 — 同时 derive Reflect 并注册
#[derive(Asset, Reflect)]
#[reflect(Asset)]
struct MyAsset {
    // ...
}
```

在 app 构建时还需要注册：

```rust
app.register_asset_reflect::<MyAsset>();
```

**对项目现有资产的影响：** 需要审查当前所有资产类型的 derive 情况，确保未来需要序列化的资产已正确实现 Reflect。

---

## 3. Cancellable Web Tasks

### 3.1 新特性概述

在 WASM 平台上，`Task` 的 drop 现在能正确取消任务。此前在 WASM 目标中，`wasm_bindgen_futures::spawn_local` 启动的任务无法被取消——即使 drop 了 Task 句柄，任务仍会继续执行到完成。

### 3.2 影响

| 方面 | 说明 |
|------|------|
| 跨平台一致性 | Task 生命周期行为在 native 和 WASM 平台上统一 |
| 资源管理 | WASM 平台上取消长时间运行的异步任务不再泄漏 |
| 对项目影响 | 较小 — 项目当前无 WASM 目标，但若未来考虑 Web 部署则直接受益 |

---

## 4. 对项目的迁移建议

### 4.1 立即可做（低成本高收益）

**为新增资产类型确保 derive Reflect：**

审查并确保所有资产类型同时 derive `Reflect` 并添加 `#[reflect(Asset)]`，而非仅 derive `TypePath`。这是最小成本的前置准备。

重点关注的 Config 资产：
- `AbilityConfig`
- `BuffConfig`
- `EffectConfig`
- `CharacterConfig`

```rust
// 迁移前
#[derive(Asset, TypePath)]
struct AbilityConfig { /* ... */ }

// 迁移后
#[derive(Asset, Reflect)]
#[reflect(Asset)]
struct AbilityConfig { /* ... */ }
```

同时在 app 构建中注册：

```rust
app.register_asset_reflect::<AbilityConfig>()
    .register_asset_reflect::<BuffConfig>()
    .register_asset_reflect::<EffectConfig>()
    .register_asset_reflect::<CharacterConfig>();
```

### 4.2 未来采用（按需启用）

| 功能 | 启用时机 | 说明 |
|------|----------|------|
| Asset Saving | 编辑器阶段 | 地图编辑器、技能编辑器需要运行时保存时引入 |
| Handle 序列化 | 需要序列化含 Handle 的配置时 | 当 Save/Replay 系统需要持久化含资产引用的数据时启用 |

### 4.3 暂不需要

| 功能 | 原因 |
|------|------|
| Web Task 取消 | 项目暂无 WASM 目标，无直接影响 |

---

## 5. Reflect 地位上升的信号

### 5.1 趋势

Bevy 0.19 中多个新功能依赖 Reflect：

| 功能 | 对 Reflect 的依赖 |
|------|-------------------|
| AssetSaver Settings | Settings 需要通过 Reflect 进行序列化 |
| Handle 序列化 | 通过 Reflect Processor 实现 Handle ↔ 路径的转换 |
| Scene 系统 | Scene 的序列化/反序列化依赖 Reflect |
| Asset Saving | SavedAsset 构建依赖 Reflect 类型信息 |
| BSN 场景 | 场景节点和属性的运行时操作依赖 Reflect |

这一趋势表明：**Reflect 正在从"可选的反射工具"演变为"Bevy 生态的基础能力"**。未实现 Reflect 的类型将在越来越多的场景中被排除在外。

### 5.2 对项目的影响

领域核心对象建议尽量 derive Reflect：

```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
struct Buff {
    // ...
}
```

尤其需要关注的类型：

| 类型 | 当前状态建议 | 理由 |
|------|-------------|------|
| `AbilityConfig` | 必须 derive Reflect + `#[reflect(Asset)]` | 资产类型，未来编辑器和序列化的核心 |
| `BuffConfig` | 必须 derive Reflect + `#[reflect(Asset)]` | 同上 |
| `EffectConfig` | 必须 derive Reflect + `#[reflect(Asset)]` | 同上 |
| `CharacterConfig` | 必须 derive Reflect + `#[reflect(Asset)]` | 同上 |
| 领域核心组件 | 逐步补上 Reflect + `#[reflect(Component)]` | 未来接入编辑器、Scene 系统时避免技术债 |

### 5.3 建议策略

1. **新增资产类型：必须** derive `Reflect` + `#[reflect(Asset)]`
2. **新增配置类型：建议** derive `Reflect`，为未来序列化需求预留
3. **领域核心组件：逐步** 补上 `Reflect` + `#[reflect(Component)]`，可在重构窗口期集中处理

> **风险提示：** 如果现在不补 Reflect，未来接入编辑器时需要大量补 derive 和注册代码，且可能涉及序列化格式的兼容性问题。早期补齐的成本远低于后期补救。

---

## 6. 注意事项

1. **Asset Saving 是异步操作**：必须在 `IoTaskPool` 中执行，不能在主线程同步等待，否则会阻塞帧循环
2. **Handle 序列化需要正确的 Reflect 实现**：仅 `TypePath` 不够，必须 `Reflect` + `#[reflect(Asset)]` + `register_asset_reflect`
3. **SavedAsset 借用而非拥有资产**：可以在同一个 async block 中构建 `SavedAsset` 并调用 `save_using_saver`，无需额外的所有权转移
4. **HandleDeserializeProcessor 需要可变引用**：反序列化时会注册新 Handle，调用时需 `&mut registry`
5. **未来演进**：Bevy 可能继续增强资产系统（如 `.bsn` 资产加载器、资产热重载改进等），保持关注以减少后续迁移成本
