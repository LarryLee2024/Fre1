# Bevy 版本总结

## 内容编写标准

| 标准项 | 要求 |
| --- | --- |
| **术语使用** | 保留英文技术术语（如 ECS、Query、Transform、GLTF、PBR 等），中文描述核心概念 |
| **描述深度** | 每个特性需包含：定义/概念、解决的问题、设计理念、核心组件、使用方式、API 示例 |
| **代码示例** | 原文中的代码示例是核心内容，必须在总结中体现。提取关键代码片段，展示 API 用法、组件结构、系统签名等。代码使用反引号格式，每行独立 |
| **分类粒度** | 大类 → 细类 → 子项，三级分类，覆盖引擎所有核心模块 |
| **应用建议** | 具体到 SRPG 场景，说明如何用于战棋开发，而非泛泛而谈 |
| **版本对比** | 标注相对于前一版本的新增/变更/废弃，便于追踪演进 |
| **符号规范** | ✅ 新增、⚠️ 变更、❌ 废弃、⭐️-⭐️⭐️⭐️ 重要程度（⭐️低/⭐️⭐️中/⭐️⭐️⭐️高） |

---

## Bevy 0.6 — 全新渲染器与ECS人体工程学革命

| 大类 | 项目 | 详细描述 | SRPG项目应用建议 |
| --- | --- | --- | --- |
| **渲染系统** | ⭐️⭐️⭐️ 全新渲染器架构 | ✅ **现代渲染器重写**：<br>**解决问题**：旧渲染器复杂、性能差、难以扩展<br>**设计理念**：从行业经验汲取灵感（Bungie管线化Destiny渲染器），与Rust图形生态协作<br>**架构优势**：<br>- 更高并行度<br>- 更少每实体计算<br>- 更高效CPU→GPU数据流<br>- 模块化核心管线（2D/3D）<br>**协作开发者**：@aclysma(rafx)、@cwfitzgerald(rend3) | **战棋渲染**：<br>- 性能提升：支持更多单位<br>- 模块化：易于扩展特效<br>- 多视图：分屏/小地图<br>- 阴影：提升真实感 |
| **渲染系统** | ⭐️⭐️⭐️ 管线化渲染 | ✅ **Extract→Prepare→Queue→Render**：<br>**解决问题**：应用和渲染串行执行，无法并行<br>**设计理念**：管线化分离应用逻辑和渲染逻辑<br>**管线阶段**：<br>- **Extract**：从主世界同步数据到渲染世界（只读最少量数据）<br>- **Prepare**：写入GPU缓冲区、纹理、绑定组<br>- **Queue**：排队渲染作业<br>- **Render**：运行渲染图生成GPU命令<br>**优势**：<br>- 应用逻辑和渲染逻辑可并行<br>- 数据流清晰<br>**状态**：当前尚未并行（需修复非Send资源问题） | **战棋性能**：<br>- 应用逻辑：AI、战斗计算<br>- 渲染逻辑：精灵、阴影<br>- 双核并行：提升帧率<br>- 未来：完整管线化 |
| **渲染系统** | ⭐️⭐️⭐️ 渲染图与子图 | ✅ **Render Graph + Sub Graph**：<br>**解决问题**：渲染逻辑难以复用和扩展<br>**设计理念**：模块化有向无环图（DAG）<br>**核心概念**：<br>- **图节点**：构建GPU命令<br>- **子图**：命名空间化渲染图（如"2D"、"3D"）<br>- **任意输入运行**：从任意节点执行子图<br>**应用场景**：<br>- 分屏渲染<br>- 镜子效果<br>- 渲染到纹理<br>- 阴影贴图 | **战棋多视图**：<br>- 主视图：战棋场景<br>- 小地图：缩略视图<br>- 角色详情：特写视图<br>- 阴影：方向/点光源 |
| **渲染系统** | ⭐️⭐️⭐️ 材质系统 | ✅ **Material trait + MaterialPlugin**：<br>**解决问题**：自定义着色器需要大量样板代码<br>**设计理念**：简化自定义材质流程<br>**使用方式**：<br>`impl Material for CustomMaterial { ... }`<br>`app.add_plugin(MaterialPlugin::<CustomMaterial>::default())`<br>**可选实现**：<br>- `fragment_shader()`：自定义片段着色器<br>- `bind_group_layout()`：绑定组布局<br>- `bind_group()`：绑定组<br>**SpecializedMaterial**：<br>- 每实体键特化着色器<br>- 内置StandardMaterial使用此特性切换光照接收 | **战棋材质**：<br>- 地形材质：森林/沙漠/雪地<br>- 单位材质：PBR+自定义效果<br>- 特效材质：发光、半透明<br>- 物理材质：金属/非金属 |
| **渲染系统** | ⭐️⭐️⭐️ 方向光与点光源阴影 | ✅ **DirectionalLight阴影**：<br>- 太阳般无限远光源阴影<br>- 启用：`DirectionalLight::shadows_enabled = true`<br>✅ **PointLight阴影**：<br>- 全向阴影<br>- 启用：`PointLight::shadows_enabled = true`<br>✅ **阴影控制组件**：<br>- `NotShadowCaster`：不投射阴影<br>- `NotShadowReceiver`：不接收阴影<br>**代码示例**：<br>`commands.entity(entity).insert(NotShadowCaster);` | **战棋阴影**：<br>- 太阳阴影：室外场景<br>- 点光源：火把、魔法灯<br>- 精灵阴影：2D单位<br>- 动态控制：隐身单位无阴影 |
| **渲染系统** | ⭐️⭐️ 分簇前向渲染 | ✅ **Clustered Forward Rendering**：<br>**解决问题**：多灯光性能急剧下降<br>**设计理念**：视锥体划分为3D网格（簇），每个簇分配灯光<br>**实现**：<br>- 视锥体空间划分<br>- 灯光按影响范围分配<br>- 片段只计算所属簇的灯光<br>**限制**：最多256个灯光（WebGL2 uniform缓冲区限制）<br>**未来**：存储缓冲区支持更多灯光 | **战棋灯光**：<br>- 多光源场景：火把、魔法<br>- 性能：100+灯光可行<br>- 视觉效果：丰富的光照 |
| **渲染系统** | ⭐️⭐️⭐️ 精灵批处理 | ✅ **Sprite Batching**：<br>**解决问题**：每精灵绘制调用，性能差<br>**实现**：<br>- 按纹理在z层级内批处理<br>- 跨z层级机会性批处理<br>**性能对比**：<br>- 旧版：~8,000精灵@60fps<br>- 新版：~100,000精灵@60fps<br>- 提升：**12.5倍** | **战棋2D性能**：<br>- 大量精灵：10,000+单位<br>- 地图瓦片：大量小精灵<br>- 特效粒子：数千粒子<br>- 流畅60fps |
| **渲染系统** | ⭐️⭐️ 视锥体裁剪 | ✅ **Frustum Culling**：<br>**解决问题**：绘制不可见对象浪费性能<br>**实现**：<br>- 3D对象自动裁剪<br>- 轴对齐包围盒（AABB）<br>- 排除视锥体外对象<br>**性能提升**：<br>- 大型场景显著<br>- 避免绘制不可见物体 | **战棋场景**：<br>- 大地图：只渲染可见区域<br>- 性能优化：避免无效绘制<br>- 视角控制：相机裁剪 |
| **渲染系统** | ⭐️⭐️ WebGL2原生支持 | ✅ **内置WebGL2后端**：<br>**解决问题**：第三方bevy_webgl2插件<br>**实现**：wgpu原生WebGL2后端<br>**部署方式**：<br>`cargo build --target wasm32-unknown-unknown`<br>`wasm-bindgen --out-dir OUTPUT_DIR --target web TARGET_DIR`<br>**限制**：无存储缓冲区、无计算着色器 | **战棋Web版**：<br>- 完整渲染支持<br>- 跨平台一致<br>- 浏览器直接运行 |
| **渲染系统** | ⭐️⭐️ WGSL着色器 | ✅ **WGSL着色器语言**：<br>**解决问题**：GLSL依赖复杂<br>**设计理念**：WebGPU新着色器语言<br>**特点**：<br>- 跨平台<br>- Bevy官方推荐<br>**代码示例**：<br>`[[stage(vertex)]] fn vertex(vertex: Vertex) -> VertexOutput { ... }` | **战棋着色器**：<br>- 着色器开发<br>- WGSL学习<br>- 更简洁语法 |
| **渲染系统** | ⭐️⭐️ 着色器预处理器 | ✅ **#import、#ifdef、#endif**：<br>**解决问题**：着色器代码无法复用<br>**支持指令**：<br>- `#import "path"`：资源路径导入<br>- `#import bevy_pbr::mesh_view_bind_group`：插件导入<br>- `#ifdef FOO`、`#ifndef FOO`、`#else`、`#endif`：条件编译<br>**应用场景**：<br>- 复杂着色器模块化<br>- 条件特性启用 | **战棋着色器**：<br>- 模块化：公共函数库<br>- 条件编译：平台适配<br>- 代码复用：减少重复 |
| **渲染系统** | ⭐️⭐️ 管线特化 | ✅ **SpecializedPipeline**：<br>**解决问题**：着色器多排列需要多管线<br>**设计理念**：按键特化管线<br>**实现**：<br>`impl SpecializedPipeline for MyPipeline`<br>`type Key = MyPipelineKey;`<br>`fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor`<br>**特性**：<br>- 自动键缓存<br>- 热重载 | **战棋管线**：<br>- 多材质：特化不同管线<br>- 动态特性：运行时切换<br>- 性能：缓存优化 |
| **渲染系统** | ⭐️⭐️ 材质Alpha混合 | ✅ **AlphaMode配置**：<br>**解决问题**：透明度控制不足<br>**模式**：<br>- `AlphaMode::Opaque`：不透明<br>- `AlphaMode::Mask(f32)`：遮罩<br>- `AlphaMode::Blend`：混合<br>**GLTF支持**：自动设置 | **战棋透明**：<br>- 半透明UI<br>- 玻璃效果<br>- 水面反射 |
| **渲染系统** | ⭐️⭐️ 球形区域光 | ✅ **PointLight半径**：<br>**解决问题**：点光源大小无法控制<br>**API**：<br>`PointLight { radius: 1.0, ... }`<br>**效果**：光源大小影响衰减 | **战棋灯光**：<br>- 火把大小<br>- 魔法范围<br>- 视觉效果 |
| **渲染系统** | ⭐️⭐️ 无限反向Z投影 | ✅ **无限反向Z透视投影**：<br>**解决问题**：深度精度不足<br>**实现**：右手系无限反向Z<br>**优势**：近处精度更高 | **战棋3D**：<br>- 深度精度<br>- 更好视觉效果 |
| **渲染系统** | ⭐️⭐️ 计算着色器 | ✅ **Compute Shader支持**：<br>**解决问题**：GPU通用计算无法使用<br>**示例**：康威生命游戏<br>**应用场景**：<br>- 粒子模拟<br>- 物理计算<br>- AI推理 | **战棋计算**：<br>- 粒子特效<br>- 物理模拟<br>- GPU加速AI |
| **渲染系统** | ⭐️⭐️ 精灵人体工程学 | ✅ **简化Sprite创建**：<br>**解决问题**：旧版需要材质管理<br>**新版API**：<br>`commands.spawn_bundle(SpriteBundle { texture: asset_server.load("player.png"), ..Default::default() });`<br>**变更**：<br>- 纹理直接组件<br>- 颜色在Sprite组件设置<br>**旧版对比**：需要`materials.add(handle.into())` | **战棋精灵**：<br>- 简化创建<br>- 更少样板代码<br>- 更易维护 |
| **渲染系统** | ⭐️ UV球体网格 | ✅ **UVSphere**：<br>`Mesh::from(UVSphere { radius: 1.0, sectors: 16, stacks: 32 })` | **战棋3D**：<br>- 球体单位<br>- 星球地形 |
| **渲染系统** | ⭐️ 平面法线计算 | ✅ **compute_flat_normals()**：<br>- 导入GLTF自动计算<br>- 符合GLTF规范 | **战棋模型**：<br>- 自动法线<br>- 修复导入问题 |
| **渲染系统** | ⭐️⭐️ 更快GLTF加载 | ✅ **性能优化**：<br>- 节点加载：40秒→0.2秒<br>- 纹理异步加载：减半加载时间<br>**开发者**：@DJMcNab、@mockersf | **战棋资源**：<br>- 大地图快速加载<br>- 减少等待时间 |
| **ECS核心** | ⭐️⭐️⭐️ 不再需要.system() | ⚠️ **移除.system()调用**：<br>**解决问题**：旧版必须调用.system()<br>**旧版**：`app.add_system(gravity.system())`<br>**新版**：`app.add_system(gravity)`<br>**实现**：@DJMcNab找到解决方案 | **战棋代码**：<br>- 更简洁<br>- 更少样板<br>- 更易读 |
| **ECS核心** | ⭐️⭐️⭐️ 新Component Trait | ⚠️ **显式Component实现**：<br>**解决问题**：旧版自动impl导致混淆<br>**变更**：<br>- 不再自动实现<br>需派生或手动实现：<br>`#[derive(Component)]`<br>`impl Component for T { type Storage = TableStorage; }`<br>**原因**：<br>- 避免非组件类型意外添加<br>- 优化：关联类型自定义<br>- 文档化：自我说明<br>- 组织性：配置在类型旁<br>**Storage类型**：<br>- `TableStorage`：默认，快速迭代<br>- `SparseSetStorage`：快速增删 | **战棋组件**：<br>- 显式标记<br>- 存储优化<br>- 避免错误<br>- 更清晰意图 |
| **ECS核心** | ⭐️⭐️ 可变查询iter() | ✅ **Query迭代改进**：<br>**解决问题**：可变查询必须用iter_mut<br>**新版**：<br>`for player in players.iter() { }` // 不可变<br>`for mut player in players.iter_mut() { }` // 可变<br>**旧版**：需要QuerySet避免冲突 | **战棋查询**：<br>- 灵活访问<br>- 更少样板<br>- 更安全 |
| **ECS核心** | ⭐️⭐️ SystemState | ✅ **直接World访问**：<br>**解决问题**：系统参数只能在系统中使用<br>**API**：<br>`let mut state: SystemState<(Res<A>, Query<&B>)> = SystemState::new(&mut world);`<br>`let (a, query) = state.get(&world);`<br>**优势**：<br>- 可变访问多个不相交资源<br>- 缓存优化<br>- 消除WorldCell需求 | **战棋底层**：<br>- World直接操作<br>- 资源访问<br>- 性能关键路径 |
| **ECS核心** | ⭐️⭐️ 子应用 | ✅ **Sub Apps**：<br>**解决问题**：主应用和渲染应用分离<br>**实现**：<br>`app.add_sub_app(RenderApp, render_app, |app_world, render_app| { ... });`<br>**访问**：<br>`app.sub_app_mut(RenderApp).add_system(system);` | **战棋架构**：<br>- 渲染隔离<br>- 多应用<br>- 模块化 |
| **ECS核心** | ⭐️⭐️ Query::iter_combinations | ✅ **组合迭代**：<br>**解决问题**：N实体组合难以迭代<br>**API**：<br>`for [p1, p2] in query.iter_combinations() { }`<br>**时间复杂度**：O(N^K)，指数增长<br>**应用**：碰撞检测、引力模拟 | **战棋碰撞**：<br>- 单位碰撞检测<br>- 范围检查<br>- 交互系统 |
| **ECS核心** | ⭐️ 优化系统命令 | ✅ **Commands性能**：<br>**改进**：存储和重用优化<br>**基准测试**：实体生成显著提升 | **战棋命令**：<br>- 更快实体操作<br>- 批量生成 |
| **ECS核心** | ⭐️ 健全性改进 | ✅ **Soundness修复**：<br>- unsafe代码移除<br>- Query改进<br>- 存储修复<br>**目标**：减少unsafe | **战棋稳定**：<br>- 更安全<br>- 更可靠 |
| **ECS核心** | ⭐️ 层级便利函数 | ✅ **层级操作**：<br>`commands.entity(e).despawn_descendants();`<br>`commands.entity(parent).remove_children(&[child1, child2]);` | **战棋层级**：<br>- 清理子实体<br>- 重组层级 |
| **UI系统** | ⭐️ Overflow::Hidden | ✅ **溢出隐藏**：<br>- Flexbox Overflow::Hidden<br>- 裁剪子内容<br>- 滚动列表基础 | **战棋UI**：<br>- 滚动列表<br>- 内容裁剪 |
| **UI系统** | ⭐️ Text2D变换 | ✅ **Text2d变换**：<br>- Transform组件支持<br>- 任意变换<br>**建议**：优先用字体大小调整 | **战棋文本**：<br>- 动态文本<br>- 世界空间文字 |
| **UI系统** | ⭐️ 窗口透明 | ✅ **Window透明**：<br>- 无背景窗口<br>- 类小部件应用 | **战棋UI**：<br>- 透明面板<br>- 覆盖层 |
| **变换系统** | ⭐️ 方向向量 | ✅ **Transform方向**：<br>- `left()`/`right()`<br>- `up()`/`down()`<br>- `forward()`/`back()` | **战棋方向**：<br>- 单位朝向<br>- 移动方向 |
| **变换系统** | ⭐️ 构建器方法 | ✅ **Transform构建器**：<br>`Transform::from_xyz(0.0, 0.0, 10.0).with_scale(Vec3::splat(2.0))` | **战棋变换**：<br>- 链式调用<br>- 简化创建 |
| **输入系统** | ⭐️ Gamepads资源 | ✅ **Gamepads资源**：<br>**API**：<br>`fn system(gamepads: Res<Gamepads>) { for gamepad in gamepads.iter() { } }` | **战棋手柄**：<br>- 手柄检测<br>- 多手柄支持 |
| **输入系统** | ⭐️ any_pressed | ✅ **Input::any_pressed()**：<br>`if input.any_pressed([KeyCode::LShift, KeyCode::RShift]) { }` | **战棋输入**：<br>- 多键检测<br>- 便捷API |
| **性能分析** | ⭐️ Tracy后端 | ✅ **trace_tracy feature**：<br>- Tracy性能分析器<br>- 可视化性能 | **战棋优化**：<br>- 性能分析<br>- 瓶颈定位 |
| **性能分析** | ⭐️ 更多Spans | ✅ **渲染应用tracing**：<br>- 帧span<br>- Schedule span<br>- 渲染图span<br>- 系统执行span | **战棋调试**：<br>- 性能追踪<br>- 详细分析 |
| **反射系统** | ⭐️ FromReflect | ✅ **FromReflect trait**：<br>`#[derive(Reflect, FromReflect)]`<br>**用途**：反射集合类型、往返转换 | **战棋反射**：<br>- 序列化<br>- 配置系统 |
| **Rust 2021** | ⭐️ Rust 2021 | ⚠️ **更新到Rust 2021**：<br>- 新Cargo功能解析器<br>- edition = "2021"<br>**注意**：虚拟工作区需手动`resolver = "2"` | **战棋编译**：<br>- 更新Cargo.toml<br>- 新特性支持 |
| **错误处理** | ⭐️ 错误码系统 | ✅ **Bevy错误码**：<br>- 类似rustc错误码<br>- 可搜索讨论<br>- 自动生成页面 | **战棋调试**：<br>- 错误诊断<br>- 社区支持 |
| **组织变更** | ⭐️ 更多合并者 | ✅ **PR合并者扩展**：<br>- @mockersf：无争议变更<br>- @alice-i-cecile：无争议文档 | **社区贡献**：<br>- 更快合并<br>- 更活跃 |
| **组织变更** | ⭐️ 双MIT/Apache-2.0 | ✅ **双许可**：<br>- MIT或Apache-2.0<br>- 更灵活<br>- 专利保护 | **战棋发布**：<br>- 许可灵活<br>- 企业友好 |

---

## Bevy 0.5 — PBR渲染与ECS V2革命

| 大类 | 项目 | 详细描述 | SRPG项目应用建议 |
| --- | --- | --- | --- |
| **渲染系统** | ⭐️⭐️⭐️ PBR渲染 | ✅ **基于物理的渲染（Physically Based Rendering）**：<br>**解决问题**：旧版渲染缺乏真实感，无法满足3D游戏的视觉需求<br>**设计理念**：基于物理的光照和材质模型，模拟真实世界的光线行为<br>**StandardMaterial 属性**：<br>- `base_color`：基础颜色（反照率）<br>- `roughness`：表面粗糙度（0.0光滑~1.0粗糙）<br>- `metallic`：金属度（0.0非金属~1.0金属）<br>- `emissive`：自发光颜色<br>- `reflection`：反射强度<br>**纹理支持**：<br>- `base_color`：漫反射贴图<br>- `normal_map`：法线贴图（增加表面细节）<br>- `metallic_roughness`：金属度/粗糙度贴图<br>- `emissive`：自发光贴图<br>- `occlusion`：环境光遮蔽贴图<br>**技术来源**：Filament（Google）、Unreal Engine、Disney | **战棋3D渲染**：<br>- 金属材质：武器、盔甲（高metallic）<br>- 非金属材质：皮肤、布料（低metallic）<br>- 粗糙度控制：抛光金属vs磨砂表面<br>- 自发光：魔法特效、UI指示器<br>- 法线贴图：地形细节、模型细节<br>**工作流**：美术在Blender中设置PBR材质，导入Bevy直接使用 |
| **渲染系统** | ⭐️⭐️ 渲染层 | ✅ **RenderLayer**：<br>**解决问题**：相机无法控制可见内容，调试时需要隐藏特定元素<br>**设计理念**：分层渲染，类似Photoshop图层概念<br>**API**：<br>- `RenderLayers::layer(0)`：添加到第0层<br>- 相机组件：指定可见层<br>- 多层组合：`RenderLayers::from_bits()` | **战棋分层**：<br>- Layer 0：地形层（地面、墙壁）<br>- Layer 1：单位层（角色、怪物）<br>- Layer 2：特效层（攻击特效、状态效果）<br>- Layer 3：UI层（血条、名称）<br>- 调试：临时隐藏某层<br>- 小地图：单独相机渲染缩略图 |
| **渲染系统** | ⭐️ 精灵翻转 | ✅ **flip_x/flip_y**：<br>**解决问题**：角色朝向需要镜像纹理，浪费内存存储对称精灵<br>**设计理念**：GPU级别翻转，零内存开销<br>**API**：<br>`Sprite { flip_x: true, flip_y: false }` | **战棋精灵**：<br>- 角色朝向：左右翻转复用纹理<br>- 动画复用：一套动画支持左右朝向<br>- 节省内存：减少50%纹理资源 |
| **渲染系统** | ⭐️ 色彩空间 | ⚠️ **Color 枚举**：<br>**解决问题**：旧版统一转换为线性sRGB，导致精度问题和颜色失真<br>**设计理念**：保留原始色彩空间，仅在GPU转换<br>**枚举类型**：<br>- `Rgba`：sRGB颜色<br>- `RgbaLinear`：线性RGB<br>- `Hsla`：色相/饱和度/亮度<br>**代码示例**：<br>`Color::rgba(1.0, 0.5, 0.0, 1.0)`<br>`Color::hsl(120.0, 0.8, 0.5)` | **战棋颜色**：<br>- HSL调色：方便美术调整<br>- 精确颜色：避免精度问题<br>- 状态效果：不同色彩空间混合 |
| **渲染系统** | ⭐️ 线框模式 | ✅ **WireframePlugin**：<br>**解决问题**：开发阶段需要查看模型网格结构<br>**使用方式**：<br>- 全局启用：`WireframePlugin`<br>- 按实体：`Wireframe` 组件<br>`commands.spawn_bundle(PbrBundle { ... }).insert(Wireframe);` | **战棋调试**：<br>- 模型调试：检查网格拓扑<br>- 占位符：开发初期无美术资源<br>- 性能分析：线框渲染更快 |
| **渲染系统** | ⭐️ 3D正交相机 | ✅ **正交相机3D**：<br>**解决问题**：等距视角游戏需要正交投影<br>**应用场景**：<br>- 等距战棋：经典视角<br>- CAD应用<br>- 建筑可视化 | **战棋视角**：<br>- 等距战棋：经典视角<br>- 2.5D效果：无透视变形<br>- 策略游戏：全局视野 |
| **渲染系统** | ⭐️ 正交相机缩放 | ✅ **ScalingMode**：<br>**解决问题**：旧版只有一种窗口缩放模式，无法适配不同游戏类型<br>**模式选项**：<br>- 窗口缩放（默认）<br>- 任意缩放因子<br>- 水平/垂直缩放<br>- `OrthographicProjection::scale` | **战棋相机**：<br>- 缩放控制：放大查看细节<br>- 分辨率适配：不同屏幕<br>- 战术视角：全局/局部切换 |
| **渲染系统** | ⭐️ 灵活相机绑定 | ✅ **RenderResourceBindings**：<br>**解决问题**：旧版硬编码相机数据，自定义着色器无法访问<br>**设计理念**：统一的资源绑定系统<br>**GLSL绑定**：<br>`layout(set = 0, binding = 0) uniform CameraViewProj { mat4 ViewProj; };`<br>`layout(set = 0, binding = 1) uniform CameraPosition { vec3 CameraPos; };` | **战棋着色器**：<br>- 相机位置：雾效、光照计算<br>- 相机方向：边缘检测、轮廓线<br>- 自定义数据：投影矩阵 |
| **渲染系统** | ⭐️ WGPU配置 | ✅ **WgpuOptions**：<br>**解决问题**：不同平台GPU能力不同，需要动态适配<br>**配置项**：<br>- 特性：`WgpuFeatures`<br>- 限制：`WgpuLimits`<br>`app.insert_resource(WgpuOptions { features: WgpuFeatures { features: vec![WgpuFeature::NonFillPolygonMode] }, .. })` | **战棋渲染**：<br>- 平台适配：移动端降低质量<br>- 特性开关：启用/禁用高级特性<br>- 性能调优：平衡质量和速度 |
| **ECS核心** | ⭐️⭐️⭐️ 混合组件存储 | ✅ **Table + Sparse Set**：<br>**解决问题**：两种存储范式各有优劣，无法兼得<br>**设计理念**：混合存储，按需选择<br>**Table（原型存储）**：<br>- 默认存储<br>- 优点：快速迭代（缓存友好）<br>- 缺点：添加/移除组件昂贵（需复制到新表）<br>- 适用：静态组件（如Position、Velocity）<br>**Sparse Set（稀疏集）**：<br>- 可选启用<br>- 优点：添加/移除组件O(1)<br>- 缺点：迭代较慢（随机访问）<br>- 适用：频繁变化组件（如Buff、状态）<br>**配置方式**：<br>`app.register_component(ComponentDescriptor::new::<T>(StorageType::SparseSet))`<br>**性能对比**：<br>- 添加/移除：Sparse Set快10x<br>- 迭代：Table快2x | **战棋性能**：<br>- Table组件：Position、Velocity、Health（静态）<br>- Sparse Set组件：Buff、状态效果、临时标记<br>- 战斗系统：频繁添加/移除Buff用Sparse Set<br>- 移动系统：位置更新用Table<br>**设计原则**：默认Table，需要频繁变化时显式配置Sparse Set |
| **ECS核心** | ⭐️⭐️⭐️ 有状态查询 | ✅ **查询缓存**：<br>**解决问题**：朴素ECS每次查询都需遍历所有原型，性能随原型数量增加而下降<br>**设计理念**：缓存匹配结果，增量构建状态<br>**优化点**：<br>- 缓存原型匹配结果<br>- 缓存获取/过滤状态（TypeId哈希只做一次）<br>- 增量构建（新原型只处理自身）<br>**代码示例**：<br>`let mut query = world.query::<(&A, &mut B)>();`<br>`for (a, mut b) in query.iter_mut(&mut world)`<br>**性能提升**：<br>- 碎片化迭代：显著提升<br>- 大量原型：稳定性能 | **战棋查询**：<br>- 复杂查询：缓存加速<br>- 碎片化：真实游戏场景<br>- 大量原型：不同单位类型<br>- 稳定帧率：不受实体数量影响 |
| **ECS核心** | ⭐️⭐️ for_each迭代器 | ✅ **Query::for_each**：<br>**解决问题**：标准迭代器有边界检查开销<br>**设计理念**：无边界检查的快速迭代<br>**性能提升**：<br>- 碎片化迭代：1.5-3x<br>- 非碎片化：1.2x<br>**代码示例**：<br>`query.for_each_mut(|(a, mut b)| { });` | **战棋性能**：<br>- 热点系统：伤害计算、AI决策<br>- 批量处理：100+单位<br>- 选择使用：需要灵活性用iter，需要性能用for_each |
| **ECS核心** | ⭐️⭐️⭐️ 并行执行器V2 | ✅ **System Labels**：<br>**解决问题**：旧版只能通过Stage控制顺序，粒度粗<br>**设计理念**：细粒度依赖声明<br>**API**：<br>- 字符串标签：`.label("velocity")`<br>- 依赖：`.after("velocity")`、`.before("movement")`<br>- 自定义标签：`#[derive(SystemLabel)]`<br>✅ **System Sets**：<br>**解决问题**：多个系统需要相同配置<br>**设计理念**：批量配置，减少样板代码<br>**API**：<br>`SystemSet::new().label(Physics).with_system(a.system()).with_system(b.system())`<br>✅ **运行条件改进**：<br>**解决问题**：旧版运行条件无法复用和组合<br>**新特性**：<br>- 可标记：`.label("every_other_time")`<br>- 可复用：多个系统共享同一条件<br>- 管道化：条件结果可组合<br>✅ **歧义检测**：<br>**解决问题**：系统顺序不确定导致bug<br>**使用方式**：<br>`app.insert_resource(ReportExecutionOrderAmbiguities)` | **战棋调度**：<br>- 物理系统：统一标签Physics<br>- 回合逻辑：精确执行顺序<br>- 条件运行：只在玩家回合运行<br>- 检测冲突：避免竞态条件<br>**设计原则**：显式优于隐式，可预测的执行顺序 |
| **ECS核心** | ⭐️⭐️⭐️ 可靠变更检测 | ⚠️ **跨帧检测**：<br>**解决问题**：旧版只检测本帧变更，条件系统可能丢失变更<br>**设计理念**："世界tick"设计，追踪自上次运行以来的变更<br>**旧版问题**：<br>- 系统未运行时丢失变更<br>- 依赖运行顺序<br>**新版优势**：<br>- 无论排序/阶段/条件<br>- 始终检测到变更<br>**代码示例**：<br>`fn system(query: Query<Entity, Changed<A>>)` | **战棋状态**：<br>- 变更追踪：可靠<br>- 条件系统：不丢失变更<br>- 回合检测：稳定<br>- 状态效果：持续追踪 |
| **ECS核心** | ⭐️⭐️⭐️ States V2 | ⚠️ **基于栈的状态机**：<br>**解决问题**：旧版StateStage减少并行度，生命周期不总是按预期工作<br>**设计理念**：基于栈的状态机，与SystemSet/RunCriteria集成<br>**栈操作**：<br>- `push`：推入新状态（保留之前状态）<br>- `pop`：恢复上一状态<br>- `set`：覆盖当前状态<br>- `replace`：替换整个栈<br>**生命周期**：<br>- `on_enter`：进入状态<br>- `on_update`：更新状态<br>- `on_exit`：退出状态<br>**代码示例**：<br>`app.add_state(AppState::Menu)`<br>`SystemSet::on_enter(AppState::Menu).with_system(setup_menu.system())` | **战棋状态**：<br>- 游戏状态：Loading→Menu→InGame→Victory/Defeat<br>- 回合栈：暂停/恢复（push/pop）<br>- 弹窗：push新状态，pop返回<br>- 同帧转换：状态A→B→C同帧完成 |
| **资源系统** | ⭐️⭐️ 顶级GLTF资源 | ✅ **Gltf 资源**：<br>**解决问题**：旧版场景/网格/纹理只作为子资源加载，难以访问<br>**设计理念**：统一的GLTF资源入口<br>**API**：<br>`let gltf = gltfs.get(&handle).unwrap();`<br>`let material = gltf.named_materials.get("MetalPartsMat");` | **战棋GLTF**：<br>- 材质访问：动态修改材质<br>- 网格遍历：检查模型结构<br>- 场景探索：运行时查询资源 |
| **场景系统** | ⭐️⭐️ 场景实例迭代 | ✅ **iter_instance_entities**：<br>**解决问题**：加载后无法遍历场景实体进行后处理<br>**API**：<br>`if let Some(entity_iter) = scene_spawner.iter_instance_entities(instance_id)` | **战棋场景**：<br>- 加载后处理：添加组件<br>- 实体遍历：初始化状态<br>- 动态修改：运行时调整 |
| **UI系统** | ⭐️⭐️ 富文本 | ✅ **TextSection**：<br>**解决问题**：旧版文本样式单一，无法混合格式<br>**设计理念**：多段落文本，每段独立样式<br>**API**：<br>`Text { sections: vec![TextSection { value: "FPS: ", style: TextStyle { color: Color::WHITE } }, TextSection { value: "60", style: TextStyle { color: Color::GOLD } }] }` | **战棋UI**：<br>- 混合样式：标签白色，数值金色<br>- 数值高亮：伤害数字红色<br>- 多语言支持：不同字体 |
| **UI系统** | ⭐️ HIDPI文本 | ✅ **缩放因子渲染**：<br>**解决问题**：文本在高DPI屏幕模糊<br>**解决方案**：根据显示器缩放因子渲染 | **战棋显示**：<br>- 高DPI支持：Retina清晰<br>- 清晰文本：所有分辨率 |
| **UI系统** | ⭐️ 2D世界空间文本 | ✅ **Text2dBundle**：<br>**解决问题**：无法在2D世界空间渲染文本<br>**应用场景**：玩家头顶名字、伤害数字 | **战棋标签**：<br>- 单位名称<br>- 血量显示<br>- 状态图标 |
| **输入系统** | ⭐️ 世界到屏幕转换 | ✅ **Camera::world_to_screen**：<br>**解决问题**：UI元素无法跟随世界坐标<br>**API**：<br>`camera.world_to_screen(window, world_position)` | **战棋UI**：<br>- 单位血条：跟随单位<br>- 伤害数字：弹出效果<br>- 选择框：高亮选中单位 |
| **事件系统** | ⭐️⭐️ 事件人体工学 | ⚠️ **EventReader/EventWriter**：<br>**解决问题**：旧版事件消费需要Local和Res，样板代码多<br>**新API**：<br>- 旧版：`Local<EventReader<SomeEvent>>`<br>- 新版：`EventReader<SomeEvent>`<br>- EventWriter：`writer.send(event)` | **战棋事件**：<br>- 更简洁API<br>- 事件发送/接收<br>- 回合事件：UnitMoved、UnitAttacked |
| **其他改进** | ⭐️⭐️ EntityRef/EntityMut | ⚠️ **构建器模式**：<br>**解决问题**：旧版每个操作都需要entity id查找<br>**新API**：<br>`world.spawn().insert(A).insert_bundle((B, C)).id()`<br>`world.entity_mut(entity).insert(D)` | **战棋实体**：<br>- 链式操作：更少样板代码<br>- 更安全：构建时验证<br>- Commands同步更新 |
| **其他改进** | ⭐️ Query::single | ✅ **单实体查询**：<br>**解决问题**：查询唯一实体需要unwrap<br>**API**：<br>`if let Ok(player) = query.single()` | **战棋查询**：<br>- 唯一单位：玩家角色<br>- 单一资源：当前回合<br>- 安全访问：避免panic |
| **其他改进** | ⭐️ 可选资源查询 | ✅ **Option<Res<T>>**：<br>**解决问题**：无法检查资源是否存在<br>**API**：<br>`fn system(a: Option<Res<A>>)` | **战棋资源**：<br>- 可选配置<br>- 可选状态<br>- 条件逻辑 |
| **其他改进** | ⭐️ 新Bundle命名 | ⚠️ **XBundle**：<br>**解决问题**：XComponents命名混淆Bundle和Component<br>**迁移**：SpriteComponents→SpriteBundle | **战棋迁移**：<br>- 更新命名<br>- 更清晰意图 |
| **其他改进** | ⭐️ World元数据 | ✅ **Archetypes、Components、Bundles、Entities**：<br>**解决问题**：无法从系统访问内部ECS元数据<br>**API**：<br>`fn system(archetypes: &Archetypes, components: &Components)` | **战棋调试**：<br>- 内省ECS<br>- 调试工具<br>- 运行时检查 |
| **其他改进** | ⭐️ 可配置SystemParams | ✅ **Local<T> 配置**：<br>**解决问题**：Local初始值无法配置<br>**API**：<br>`foo.system().config(|c| c.0 = Some(10))` | **战棋系统**：<br>- 初始值配置<br>- 更灵活 |
| **其他改进** | ⭐️ 脚本支持准备 | ✅ **组件与类型解耦**：<br>**解决问题**：组件绑定Rust类型，无法集成脚本语言<br>**设计理念**：blob存储，内存布局元数据<br>**好处**：Python/Lua数据类型可存储 | **战棋扩展**：<br>- 脚本语言集成<br>- 热重载脚本<br>- 非Rust逻辑 |
| **其他改进** | ⭐️ WorldCell | ✅ **安全多资源访问**：<br>**解决问题**：直接World访问需要unsafe<br>**API**：<br>`let world_cell = world.cell();`<br>`let a = world_cell.get_resource_mut::<i32>();` | **战棋World**：<br>- 多资源修改<br>- 安全保证<br>- 运行时检查 |
| **其他改进** | ⭐️ 资源作用域 | ✅ **resource_scope**：<br>**解决问题**：需要同时持有可变world和资源引用<br>**API**：<br>`world.resource_scope(|world, mut a: Mut<A>| {})` | **战棋渲染**：<br>- 安全资源访问<br>- 避免unsafe |
| **其他改进** | ⭐️ 计时器改进 | ⚠️ **Duration**：<br>**解决问题**：f32秒精度不足<br>**新API**：<br>`timer.tick(time.delta()).just_finished()` | **战棋计时**：<br>- 更精确<br>- 简化API |
| **其他改进** | ⭐️ 资源改进 | ✅ **错误处理**：<br>**解决问题**：加载失败panic<br>- 不再panic<br>- 路径不区分大小写<br>- 类型安全性提升 | **战棋开发**：<br>- 更稳定<br>- 更宽容 |
| **其他改进** | ⭐️ 窗口约束 | ✅ **WindowResizeConstraints**：<br>**解决问题**：窗口可被调整到过小<br>**API**：<br>`WindowResizeConstraints { min_height: 200.0, max_height: 800.0 }` | **战棋窗口**：<br>- 最小尺寸<br>- 防止过小 |
| **其他改进** | ⭐️ !Send任务 | ✅ **spawn_local**：<br>**解决问题**：非Send数据无法在线程间传递<br>**API**：<br>`scope.spawn_local(async { println!("local task"); });` | **战棋任务**：<br>- 非Send数据<br>- 特殊场景 |
| **GLTF** | ⭐️ GLTF PBR纹理 | ✅ **法线贴图、metallic/roughness、遮挡、自发光** | **战棋模型**：<br>- 更真实材质<br>- PBR工作流 |
| **其他改进** | ⭐️ 歧义集 | ✅ **in_ambiguity_set**：<br>**解决问题**：有意的歧义产生警告<br>**API**：<br>`a.system().in_ambiguity_set("foo")` | **战棋代码**：<br>- 有意的歧义<br>- 减少噪音 |

---

## Bevy 0.4 — ECS进化与开发者体验提升

| 大类 | 项目 | 详细描述 | SRPG项目应用建议 |
| --- | --- | --- | --- |
| **ECS核心** | ⭐️⭐️⭐️ 灵活ECS参数 | ⚠️ **SystemParam trait**：<br>**解决问题**：旧版强制参数顺序（Commands > Resources > Queries），新手易犯错<br>**设计理念**：参数顺序无关，编译期推断依赖<br>**旧版问题**：<br>`fn invalid_system(query: Query<&Transform>, mut commands: Commands, time: Res<Time>)` // 编译失败<br>**新版优势**：<br>- 任意顺序：`fn system(query: Query<&Transform>, commands: &mut Commands, time: Res<Time>)`<br>- 编译时间减少~25%<br>- 易于添加新参数：实现`SystemParam` trait<br>⚠️ **Commands变更**：<br>- 旧版：`mut commands: Commands`<br>- 新版：`commands: &mut Commands` | **战棋系统**：<br>- 参数顺序自由<br>- 编译更快<br>- 扩展性强：自定义系统参数 |
| **ECS核心** | ⭐️⭐️⭐️ 简化查询过滤器 | ⚠️ **过滤器与组件分离**：<br>**解决问题**：旧版过滤器与组件混杂，难以理解<br>**旧版**：`Query<With<A, Without<B, (&Transform, Changed<Velocity>)>>>`<br>**新版**：`Query<(&Transform, &Velocity), (With<A>, Without<B>, Changed<Velocity>)>`<br>✅ **类型别名**：<br>`type ChangedVelocity = (With<A>, Without<B>, Changed<Velocity>);` | **战棋查询**：<br>- 清晰过滤器<br>- 可复用类型别名<br>- 更好的可读性 |
| **ECS核心** | ⭐️⭐️⭐️ 系统输入输出与链式调用 | ✅ **系统输入输出**：<br>**解决问题**：系统间无法传递数据<br>**设计理念**：函数式管道<br>**API**：<br>- `System<In = (), Out = ()>`：默认<br>- `System<In = usize, Out = f32>`：自定义<br>✅ **链式调用**：<br>`result_system.system().chain(error_handler.system())`<br>✅ **错误处理**：<br>`fn system() -> Result<()>` | **战棋系统**：<br>- 错误处理：查询失败优雅降级<br>- 系统链：攻击→结算→UI<br>- 更强的组合性 |
| **ECS核心** | ⭐️⭐️⭐️ Schedule V2 | ✅ **Stage Trait**：<br>**解决问题**：Stage是具体类型，无法自定义<br>**设计理念**：trait化，可扩展<br>✅ **SystemStage**：<br>- 并行：`SystemStage::parallel()`<br>- 串行：`SystemStage::serial()`<br>- 自定义执行器：`SystemStage::new(MyCustomExecutor)`<br>✅ **Schedule嵌套**：<br>**解决问题**：无法组合多个Schedule<br>**设计理念**：Schedule实现Stage trait，可嵌套<br>✅ **运行条件**：<br>- `with_run_criteria(criteria.system())`<br>- `RunOnce::default()`：只运行一次<br>✅ **固定时间步长**：<br>`FixedTimestep::step(0.4)`：每0.4秒运行一次<br>✅ **类型化Stage构建器**：<br>`stage(MY_CUSTOM_STAGE, |stage: &mut MyCustomStage|)` | **战棋调度**：<br>- 回合流程：并行处理多个单位<br>- 固定时间步长：物理模拟、AI决策<br>- 条件运行：只在玩家回合运行<br>- 自定义阶段：战斗、动画、AI独立阶段 |
| **ECS核心** | ⚠️ 弃用For-Each系统 | ❌ **移除For-Each系统**：<br>**解决问题**：两种系统定义方式造成混乱<br>**移除原因**：<br>- 限制多：无法过滤、无法多查询<br>- 不一致：两种定义方式<br>- 陷阱多：`&mut T`不能工作<br>- 编译时间：节省~5秒<br>- 实现复杂：需要复杂宏<br>**迁移**：统一使用查询系统 | **战棋迁移**：<br>- 统一使用查询系统<br>- 更强的功能<br>- 更好的性能 |
| **应用状态** | ⭐️⭐️⭐️ States系统 | ✅ **应用状态**：<br>**解决问题**：无法根据应用状态启用/禁用系统<br>**设计理念**：状态机模式<br>**API**：<br>- `enum AppState { Loading, Menu, InGame }`<br>- `app.add_resource(State::new(AppState::Loading))`<br>- `StateStage::<AppState>`<br>✅ **生命周期事件**：<br>- `on_state_enter`：进入状态<br>- `on_state_update`：更新状态<br>- `on_state_exit`：退出状态<br>**代码示例**：<br>`app.on_state_enter(STAGE, AppState::Menu, setup_menu.system())`<br>✅ **状态变更**：<br>`state.set_next(AppState::InGame).unwrap()` | **战棋状态**：<br>- 游戏状态：Loading→Menu→InGame→Victory/Defeat<br>- 回合状态：PlayerTurn→EnemyTurn<br>- 状态管理：进入/退出时清理资源<br>- 条件系统：只在特定状态运行 |
| **渲染系统** | ⭐️⭐️⭐️ WebGL2支持 | ✅ **WebGL2渲染后端**：<br>**解决问题**：Web平台无渲染支持<br>**实现**：`bevy_webgl2`插件<br>**能力**：2D/3D精灵和模型渲染 | **战棋Web版**：<br>- 完整渲染支持<br>- 跨平台一致性 |
| **渲染系统** | ⭐️⭐️⭐️ 渲染器优化 | ✅ **增量更新**：<br>**解决问题**：旧版每帧全量更新，性能差<br>**优化点**：<br>- RenderResourceNode、Sprites、Transforms变更检测<br>- 仅在资源变化时同步GPU数据<br>- 共享资源RenderResourceBindings<br>- Mesh提供系统仅在需要时更新<br>- 缓存未匹配的渲染资源绑定结果<br>✅ **SharedBuffers抽象**：<br>**解决问题**：文本渲染极慢<br>**原因**：占位实现，未真正共享<br>**解决**：实现真正的共享缓冲区<br>✅ **邮箱垂直同步**：<br>**解决问题**：输入延迟<br>**解决方案**：默认使用wgpu邮箱垂直同步 | **战棋性能**：<br>- 大量精灵：10,000+高效渲染<br>- 文本渲染：UI性能提升<br>- 输入延迟：操作响应更快<br>- 帧率稳定：增量更新减少开销 |
| **渲染系统** | ⭐️ 着色器热重载 | ✅ **运行时着色器更新**：<br>**解决问题**：着色器修改需重启<br>**能力**：无需重启，即时反馈 | **战棋开发**：<br>- 着色器调试<br>- 特效开发<br>- 美术工作流 |
| **渲染系统** | ⭐️ 3D纹理资源 | ✅ **3D纹理支持**：<br>**解决问题**：无法加载3D纹理<br>**API**：`array_texture.rs`示例 | **战棋3D**：<br>- 体素地形<br>- 粒子纹理<br>- 多层纹理混合 |
| **渲染系统** | ⭐️ 文本布局改进 | ⚠️ **glyph_brush_layout**：<br>**解决问题**：自定义布局有bug<br>**改进**：修复布局错误，新增选项 | **战棋UI**：<br>- 文本渲染正确<br>- 多语言支持 |
| **渲染系统** | ⭐️ HIDPI支持 | ✅ **高像素密度显示器**：<br>**解决问题**：高DPI屏幕模糊<br>**解决方案**：考虑像素密度创建窗口 | **战棋显示**：<br>- Retina清晰<br>- 高DPI支持<br>- UI自动缩放 |
| **场景系统** | ⭐️⭐️ 场景子节点 | ✅ **场景作为子节点**：<br>**解决问题**：场景实例无法变换<br>**API**：<br>`parent.spawn_scene(handle)` | **战棋实例**：<br>- 多个相同单位<br>- 地图复用<br>- 特效实例 |
| **资源系统** | ⭐️⭐️⭐️ 反射系统 | ⚠️ **bevy_reflect**：<br>**解决问题**：旧版bevy_property功能有限<br>**设计理念**：通用Rust反射crate<br>**核心功能**：<br>- 字段访问：`get_field::<T>("name")`<br>- 路径查询：`get_path::<T>("b[0].value")`<br>- 字段遍历：`iter_fields()`<br>- Serde序列化：自动实现<br>- Trait反射：`#[reflect(DoThing)]`<br>**代码示例**：<br>`#[derive(Reflect)] struct Foo { a: u32, b: Vec<Bar> }`<br>`foo.get_field::<u32>("a")` | **战棋配置**：<br>- 单位属性：动态访问和修改<br>- 序列化：自动保存/加载<br>- 编辑器：运行时检查组件<br>- 动画：属性动画系统基础 |
| **平台支持** | ⭐️⭐️ 跨平台主函数 | ✅ **#[bevy_main]宏**：<br>**解决问题**：Android/iOS需要样板代码<br>**API**：<br>`#[bevy_main] fn main() { App::build().run(); }` | **战棋跨平台**：<br>- 一份代码多平台运行<br>- 减少平台特定代码 |
| **平台支持** | ⭐️ Apple Silicon支持 | ✅ **Apple Silicon**：<br>**解决问题**：M1/M2芯片不支持<br>**解决方案**：更新winit、coreaudio-sys依赖 | **战棋Mac**：<br>- M1/M2支持<br>- 原生性能 |
| **编译优化** | ⭐️⭐️⭐️ 动态链接 | ✅ **动态链接**：<br>**解决问题**：迭代编译慢<br>**解决方案**：强制动态链接<br>**使用方式**：<br>`cargo run --features bevy/dynamic`<br>⚠️ **注意**：发布时禁用 | **战棋开发**：<br>- 迭代编译：<1秒<br>- 快速原型<br>- 开发效率大幅提升 |
| **开发工具** | ⭐️⭐️ 日志和性能分析 | ✅ **LogPlugin**：<br>**解决问题**：无内置日志<br>**基于**：tracing crate<br>**日志宏**：<br>- `trace!()`、`debug!()`、`info!()`、`warn!()`、`error!()`<br>✅ **性能分析**：<br>- `trace` feature：ECS系统tracing spans<br>- `trace_chrome`：Chrome tracing格式<br>- `chrome://tracing`查看 | **战棋调试**：<br>- 日志记录<br>- 性能分析<br>- 跨平台日志<br>- 问题诊断 |
| **其他改进** | ⭐️ 定时器改进 | ✅ **Timer增强**：<br>**改进点**：暂停、字段访问<br>⚠️ **变更**：不再默认跳动<br>**代码示例**：<br>`Timer::from_seconds(5.0, false)` | **战棋计时**：<br>- 回合计时器<br>- 技能冷却<br>- 动画播放 |
| **其他改进** | ⭐️ 任务系统改进 | ⚠️ **性能提升**：<br>- breakout.rs提升~20%<br>- 单任务立即执行<br>- 修复死锁 | **战棋性能**：<br>- 单任务无开销<br>- 多任务高效并行 |
| **GLTF** | ⭐️ GLTF改进 | ✅ **相机导入**：<br>- 图像像素格式转换<br>- 默认材质加载<br>- 层级修复 | **战棋3D**：<br>- 相机设置<br>- 材质支持<br>- 正确层级 |

---

## Bevy 0.3 — 平台扩展与ECS性能飞跃

| 大类 | 项目 | 详细描述 | SRPG项目应用建议 |
| --- | --- | --- | --- |
| **平台支持** | ⭐️ Android支持 | ✅ **初始Android支持**：<br>**解决问题**：Android平台无支持<br>**实现**：重写bevy-glsl-to-spirv、AssetManager集成<br>**底层依赖**：android-ndk-rs、cargo-apk、Cpal音频 | **战棋移动端**：<br>- Android版战棋<br>- 触摸操作 |
| **平台支持** | ⭐️ iOS支持 | ✅ **初始iOS支持**：<br>**解决问题**：iOS平台无支持<br>**实现**：XCode项目、shaderc运行时编译<br>⚠️ **已知问题**：音频不完全工作 | **战棋移动端**：<br>- iOS版战棋<br>- 统一触摸操作 |
| **平台支持** | ⭐️ WASM资源加载 | ✅ **Web资源加载**：<br>**解决问题**：WASM无法加载资源<br>**实现**：fetch() HTTP请求<br>⚠️ **尚未支持**：渲染、多线程、声音 | **战棋Web版**：<br>- 资源异步加载<br>- 后续完善渲染 |
| **输入系统** | ⭐️⭐️ 触摸输入 | ✅ **Touch支持**：<br>**解决问题**：移动端无触摸支持<br>**API**：<br>- `Touches`资源：当前触摸状态<br>- `iter()`：遍历活跃触摸<br>- `iter_just_pressed()`：刚按下<br>- `iter_just_released()`：刚释放<br>**代码示例**：<br>`fn touch_system(touches: Res<Touches>)`<br>`  for touch in touches.iter()` | **战棋触摸操作**：<br>- 点击选择单位<br>- 长按显示信息<br>- 滑动移动地图<br>- 多点触控缩放<br>- 触摸优先级：UI>地图>单位 |
| **资源系统** | ⭐️⭐️ Handle引用计数 | ⚠️ **自动资源释放**：<br>**解决问题**：资源需要手动管理生命周期<br>**设计理念**：引用计数，归零自动释放<br>**代码示例**：<br>`let handle = asset_server.load("sprite.png");`<br>`let second_handle = handle.clone();`<br>`commands.spawn(SpriteComponents { material: materials.add(handle.into()) });`<br>`commands.despawn(sprite_entity);` // 资源自动释放 | **战棋资源管理**：<br>- 单位精灵自动释放<br>- 避免内存泄漏<br>- 简化生命周期管理 |
| **资源系统** | ⭐️ 多资源加载 | ✅ **AssetLoader多资源支持**：<br>**解决问题**：单个加载器只能生成单个资源<br>**改进**：支持GLTF加载多个网格、纹理、场景 | **战棋GLTF加载**：<br>- 完整3D场景<br>- 多个网格和纹理 |
| **资源系统** | ⭐️ 子资源加载 | ✅ **子资源引用**：<br>**解决问题**：无法加载资源文件中的特定子资源<br>**语法**：`"file.gltf#Mesh0/Primitive0"` | **战棋精确加载**：<br>- 只加载特定模型/纹理<br>- 减少内存占用 |
| **资源系统** | ⭐️ AssetIo Trait | ✅ **存储抽象层**：<br>**解决问题**：资源加载与存储耦合<br>**设计理念**：trait抽象，平台适配<br>- 桌面：文件系统<br>- Android：AssetManager<br>- Web：fetch() | **战棋跨平台**：<br>- 统一资源API<br>- 平台差异透明 |
| **资源系统** | ⚠️ 移除load_sync | ❌ **移除同步加载**：<br>**解决问题**：WASM不友好、阻塞游戏执行<br>**替代**：使用load()异步加载 | **战棋异步加载**：<br>- 不阻塞主线程<br>- 加载界面<br>- 避免卡顿 |
| **渲染系统** | ⭐️⭐️⭐️ GLTF场景加载 | ⚠️ **完整GLTF支持**：<br>**解决问题**：旧版只能加载第一个网格和纹理<br>**改进**：作为Bevy Scene加载所有内容<br>**代码示例**：<br>`fn load_gltf_system(mut commands: Commands, asset_server: Res<AssetServer>)`<br>`  let scene_handle = asset_server.load("models/FlightHelmet/FlightHelmet.gltf");`<br>`  commands.spawn_scene(scene_handle);` | **战棋3D模型**：<br>- 完整3D角色<br>- 多个网格和纹理<br>- 场景层级 |
| **渲染系统** | ⭐️⭐️ 灵活顶点属性 | ✅ **自定义顶点属性**：<br>**解决问题**：旧版固定position/normal/uv<br>**改进**：任意顶点属性<br>- 顶点颜色<br>- 骨骼权重 | **战棋顶点数据**：<br>- 顶点颜色：单位着色<br>- 骨骼权重：动画系统<br>- 自定义数据：特效参数 |
| **渲染系统** | ⭐️ 索引缓冲区特化 | ⚠️ **索引精度可配置**：<br>**解决问题**：旧版固定u16，大模型溢出<br>**改进**：可配置，默认u32 | **战棋大模型**：<br>- 复杂地形<br>- 大规模场景 |
| **变换系统** | ⭐️⭐️⭐️ Transform再次重写 | ⚠️ **相似变换（Similarity）**：<br>**解决问题**：4x4矩阵累积误差<br>**旧版问题**：<br>- 矩阵累积误差<br>- API繁琐<br>**新版优势**：<br>- translation + rotation + scale<br>- 无误差累积<br>- 直接字段访问<br>**代码示例**：<br>`fn system(mut transform: Mut<Transform>)`<br>`  transform.translation += Vec3::new(1.0, 0.0, 0.0);`<br>`  transform.rotation *= Quat::from_rotation_y(PI);`<br>`  transform.scale *= 2.0;` | **战棋变换**：<br>- 单位位置/旋转/缩放<br>- 无误差累积：长时间稳定<br>- API简洁 |
| **输入系统** | ⭐️ 手柄设置 | ✅ **GamepadSettings**：<br>**解决问题**：手柄输入无法自定义<br>**API**：<br>`gamepad_settings.axis_settings.insert(GamepadAxis(...), AxisSettings { positive_high: 0.8, .. })` | **战棋手柄适配**：<br>- 摇杆死区<br>- 灵敏度调整<br>- 按钮映射<br>- 多手柄支持 |
| **插件架构** | ⭐️⭐️ PluginGroups | ✅ **插件组**：<br>**解决问题**：add_default_plugins()过于静态<br>**API**：<br>- `add_plugins(DefaultPlugins)`<br>- `group.disable::<RenderPlugin>()`<br>- 自定义PluginGroup | **战棋插件管理**：<br>- 禁用不需要的插件<br>- 创建战棋专用插件组<br>- 模块化配置 |
| **窗口系统** | ⭐️ 动态窗口设置 | ✅ **运行时窗口属性**：<br>**解决问题**：窗口设置只能启动时配置<br>**API**：<br>`window.set_title(format!("Seconds: {}", time.seconds_since_startup))` | **战棋窗口管理**：<br>- 动态标题<br>- 窗口大小适配<br>- 全屏切换 |
| **ECS性能** | ⭐️⭐️⭐️ 100%无锁ECS | ✅ **完全无锁并行**：<br>**解决问题**：Query访问有原子锁<br>**优化**：<br>- 移除Query原子锁<br>- 移除原型安全检查<br>- 重写QueryIter<br>⚠️ **冲突查询**：<br>- 默认禁止<br>- QuerySet允许安全冲突<br>**代码示例**：<br>`fn system(mut queries: QuerySet<(Query<&mut A>, Query<(&mut A, &B)>)>)` | **战棋性能**：<br>- 大量单位查询更快<br>- 战斗计算提升<br>- AI决策流畅<br>- 帧率提升 |
| **ECS性能** | ⭐️ 线程本地资源 | ✅ **Thread Local Resources**：<br>**解决问题**：不能在线程间传递的资源<br>**API**：<br>`app.add_thread_local_resource(MyResource);` | **战棋底层访问**：<br>- 窗口管理<br>- 输入处理<br>- 音频播放<br>- 避免数据竞争 |
| **ECS API** | ⚠️ Query API变更 | ⚠️ **更清晰的API**：<br>**解决问题**：API命名不清晰<br>**变更**：<br>- `query.get::<T>(entity)` → `query.get_component::<T>(entity)`<br>- `query.get(entity)`：完整查询结果<br>- 独立iter/iter_mut | **战棋查询**：<br>- 更清晰命名<br>- 只读/可变分离<br>- 提升可读性 |

---

## Bevy 0.2 — 性能优化与平台扩展

| 大类 | 项目 | 详细描述 | SRPG项目应用建议 |
| --- | --- | --- | --- |
| **任务系统** | ⭐️⭐️ 异步任务系统 | ✅ **自定义任务池**：<br>**解决问题**：Rayon CPU使用率过高<br>**设计理念**：针对游戏引擎优化的任务池<br>- 按上下文分池：计算、IO、网络<br>- 灵活负载均衡<br>**性能对比**：<br>- 8核机器：显著降低<br>- 32核机器：显著降低 | **战棋任务管理**：<br>- 计算池：AI决策、伤害计算<br>- IO池：资源加载、存档<br>- 网络池：联机同步<br>- 避免CPU过消耗 |
| **任务系统** | ⭐️⭐️ 并行查询 | ✅ **Query并行迭代**：<br>**解决问题**：查询迭代无法利用多核<br>**API**：<br>`query.iter().par_iter(32).for_each(&pool, |mut transform| { transform.translate(Vec3::new(1.0, 0.0, 0.0)); });` | **战棋并行计算**：<br>- 100+单位位置更新<br>- 移动/攻击范围计算<br>- AI决策并行<br>- 提升帧率 |
| **变换系统** | ⭐️⭐️⭐️ Transform重写 | ⚠️ **统一Transform组件**：<br>**解决问题**：分离的Translation/Rotation/Scale导致混乱<br>**旧版问题**：<br>- LocalTransform在用户系统运行时已过时<br>- 需手动访问3个组件<br>**新版优势**：<br>- 单个Transform作为数据源<br>- GlobalTransform用于世界空间<br>**代码对比**：<br>`// 旧版`<br>`fn system(translation: &Translation, rotation: &Rotation, scale: &Scale)`<br>`// 新版`<br>`fn system(transform: &Transform)` | **战棋变换操作**：<br>- 直接修改transform<br>- 简化代码<br>- 提升可维护性 |
| **输入系统** | ⭐️⭐️ 摇杆/手柄输入 | ✅ **Gamepad支持**：<br>**解决问题**：无手柄支持<br>**基于**：gilrs库<br>**API**：<br>- `Gamepad`：手柄标识<br>- `GamepadButton`：按钮<br>- `GamepadButtonType`：按钮类型<br>**代码示例**：<br>`fn button_system(gamepads: Res<Vec<Gamepad>>, button_input: Res<Input<GamepadButton>>)`<br>`  if button_input.just_pressed(GamepadButton(*gamepad, GamepadButtonType::RightTrigger))` | **战棋手柄支持**：<br>- 手柄操作：移动光标、选择单位<br>- 按钮映射：A确认、B取消<br>- 摇杆控制：地图滚动<br>- 支持主机平台 |
| **ECS性能** | ⭐️⭐️⭐️ 世代实体ID | ⚠️ **Entity ID优化**：<br>**解决问题**：UUID随机性带来开销<br>**旧版问题**：<br>- 随机UUID<br>- 实体位置使用HashMap查找<br>**新版优势**：<br>- 递增世代索引<br>- 直接用作数组索引<br>- O(1)查找 | **战棋实体管理**：<br>- 100+单位快速查找<br>- 碰撞检测<br>- 网络同步<br>- 提升整体性能 |
| **ECS性能** | ⭐️ 只读查询 | ✅ **Read-Only Query**：<br>**解决问题**：无法保证查询不修改<br>**实现**：ReadOnly trait<br>**好处**：编译期保证，更安全的并行 | **战棋查询优化**：<br>- 只读查询<br>- 避免借用冲突<br>- 提升并行度 |
| **ECS性能** | ⭐️⭐️ 无锁World API | ⚠️ **移除World锁**：<br>**解决问题**：World访问有锁<br>**实现**：只读查询+可变借用<br>⚠️ **限制**：系统中Queries仍用锁 | **战棋性能**：<br>- 直接组件查找更快<br>- 频繁属性访问<br>- 战斗计算提升 |
| **平台支持** | ⭐️⭐️ Web平台（WASM） | ✅ **初始Web支持**：<br>**解决问题**：Web平台无支持<br>**已支持**：ECS调度、输入、空白画布<br>⚠️ **尚未支持**：渲染、多线程、声音<br>**首个WASM游戏**：bevy-robbo | **战棋Web版**：<br>- 浏览器运行<br>- 轻量级演示<br>- 后续完善渲染和音效 |

---

## Bevy 0.1 — 首次发布：开创性的 ECS 游戏引擎

| 大类 | 项目 | 详细描述 | SRPG项目应用建议 |
| --- | --- | --- | --- |
| **核心架构** | ⭐️⭐️⭐️ ECS架构体系 | ✅ **实体组件系统（Entity Component System）**：<br>**设计理念**：数据驱动架构，将数据和逻辑分离<br>**核心概念**：<br>- **实体（Entity）**：唯一ID，组件容器<br>- **组件（Component）**：普通Rust结构体，存储数据<br>- **系统（System）**：普通Rust函数，处理逻辑<br>**架构优势**：<br>- 数据与逻辑分离：强制解耦<br>- 缓存友好：Struct of Arrays布局<br>- 天然并行：系统声明依赖后自动调度<br>**对比其他ECS**：<br>- Specs：复杂生命周期和宏<br>- Shipyard：更多抽象层<br>- Bevy：纯Rust数据类型，零额外学习成本 | **战棋实体建模**：<br>- 角色实体：Unit、Position、Health、ActionPoints<br>- 地图实体：Tile、Terrain、Occupant<br>- 技能实体：Skill、Cooldown、Range<br>**系统设计**：<br>- movement_system：处理单位移动<br>- combat_system：计算伤害和状态<br>- ai_system：敌方单位决策<br>**架构价值**：<br>- 数据驱动：支持Replay/Save<br>- 并行处理：100+单位高效计算<br>- 易于扩展：添加新组件不影响现有代码 |
| **核心架构** | ⭐️⭐️⭐️ 函数系统机制 | ✅ **函数系统（Function Systems）**：<br>**设计理念**：直接使用Rust函数作为系统，无需实现trait或宏<br>**实现**：<br>- `.system()`方法将函数指针转换为`Box<dyn System>`<br>- 编译期通过`IntoQuerySystem` trait推断依赖<br>**系统签名**：<br>- 无参系统：`fn system()`<br>- 查询系统：`fn system(query: Query<(&A, &mut B)>)`<br>- 资源系统：`fn system(time: Res<Time>, score: ResMut<Score>)`<br>- 混合系统：同时访问查询和资源 | **战棋系统实现**：<br>- `fn move_unit(mut query: Query<(&mut Position, &MovementSpeed)>, input: Res<Input<KeyCode>>)`<br>- `fn apply_damage(mut query: Query<(&mut Health, &DamageReceiver)>)`<br>- `fn update_turn(timer: Res<TurnTimer>, mut state: ResMut<GameState>)`<br>**设计价值**：<br>- 零学习成本：会写Rust函数就会写系统<br>- 自动依赖推断：无需手动声明<br>- 强类型：编译期检查 |
| **核心架构** | ⭐️⭐️⭐️ 查询系统 | ✅ **Query<T>查询**：<br>**设计理念**：精确控制迭代范围和访问权限<br>**访问模式**：<br>- `&T`：只读<br>- `&mut T`：可变<br>- `Query<Entity>`：获取实体ID<br>**过滤器**：<br>- `With<T>`/`Without<T>`：包含/不包含组件<br>- `Added<T>`/`Mutated<T>`/`Changed<T>`：变更检测<br>**代码示例**：<br>`fn system(mut query: Query<(&mut Position, &Velocity)>)`<br>`  for (pos, vel) in query.iter_mut()` | **战棋查询**：<br>- 查询所有可移动单位：`Query<(&Unit, &mut Position), With<ActionPoints>>`<br>- 查询攻击范围内敌人：`Query<&Unit, Without<Ally>>`<br>- 检测新回合：`Query<Added<TurnStarted>>`<br>**设计价值**：<br>- 精确访问：只读/可变明确<br>- 过滤器：灵活的查询条件<br>- 变更检测：响应式更新 |
| **核心架构** | ⭐️⭐️ 阶段系统 | ✅ **阶段（Stages）**：<br>**设计理念**：按顺序运行系统组<br>**默认阶段**："update"<br>**自定义阶段**：<br>`add_stage_after("update", "do_things")` | **战棋回合流程**：<br>- input_stage：处理玩家输入<br>- movement_stage：执行移动逻辑<br>- combat_stage：结算战斗<br>- ai_stage：敌方AI决策 |
| **核心架构** | ⭐️⭐️ 命令系统 | ✅ **Commands**：<br>**设计理念**：延迟执行，避免系统间直接借用冲突<br>**常用命令**：<br>- `commands.spawn((ComponentA, ComponentB))`<br>- `commands.despawn(entity)`<br>- `commands.insert_resource(resource)` | **战棋实体操作**：<br>- 生成新单位<br>- 移除死亡单位<br>- 状态变更 |
| **渲染系统** | ⭐️⭐️⭐️ 渲染图架构 | ✅ **渲染图（Render Graph）**：<br>**设计理念**：节点化渲染逻辑，支持多线程<br>**内置节点**：<br>- CameraNode：相机处理<br>- PassNode：渲染通道<br>- RenderResourcesNode：资源绑定<br>- SharedBuffersNode：共享缓冲区<br>- TextureCopyNode：纹理拷贝<br>- WindowSwapChainNode：交换链 | **战棋渲染管线**：<br>- 地形层：瓦片渲染<br>- 单位层：精灵/模型渲染<br>- 高亮层：移动范围、攻击范围<br>- UI层：界面元素<br>**设计价值**：<br>- 模块化：可自定义节点<br>- 多线程：并行渲染<br>- 可扩展：添加新渲染通道 |
| **渲染系统** | ⭐️⭐️⭐️ 数据驱动着色器 | ✅ **着色器绑定**：<br>**设计理念**：组件直接绑定GPU uniform<br>**实现**：<br>- 组件派生`RenderResources`<br>- 支持GLSL着色器<br>**着色器定义**：<br>- `ShaderDefs`派生宏<br>- `#[shader_def]`属性<br>- 条件编译：`#ifdef`<br>**布局反射**：<br>- 从SpirV自动反射数据布局 | **战棋自定义着色器**：<br>- 移动范围高亮：蓝色半透明<br>- 攻击范围高亮：红色半透明<br>- 选中单位：边框闪烁<br>- 地形效果：森林、山脉、水域<br>**设计价值**：<br>- 数据驱动：组件决定着色器行为<br>- 条件编译：按需启用功能<br>- 自动反射：简化绑定 |
| **渲染系统** | ⭐️⭐️⭐️ 3D渲染特性 | ✅ **GLTF模型加载**：<br>- 加载GLTF文件作为网格资源<br>✅ **绘制顺序**：<br>- 不透明：前→后（Early-Z优化）<br>- 透明：后→前（正确混合）<br>✅ **父子层级**：<br>- 父级变换传递到后代<br>- `with_children`构建层级树<br>✅ **MSAA抗锯齿**：<br>- 多重采样抗锯齿<br>- 可配置采样数 | **战棋3D扩展**：<br>- 3D单位模型：角色、怪物<br>- 3D地形：山丘、建筑<br>- 父子层级：武器挂载点<br>- 抗锯齿：提升视觉质量 |
| **渲染系统** | ⭐️⭐️ 2D精灵系统 | ✅ **精灵渲染**：<br>- 单个图像渲染为精灵<br>- `SpriteComponents`组件<br>✅ **精灵表（Texture Atlas）**：<br>- `TextureAtlas::from_grid`从网格生成<br>- `SpriteSheetComponents`组件<br>✅ **动态图集生成**：<br>- `TextureAtlasBuilder`运行时合并 | **战棋2D渲染**：<br>- 单位精灵：每个角色独立纹理<br>- 地图瓦片：精灵表存储<br>- 动画帧：攻击、死亡动画 |
| **UI系统** | ⭐️⭐️⭐️ Bevy UI架构 | ✅ **ECS驱动UI**：<br>**设计理念**：直接使用核心ECS/层级/变换/事件/资源系统<br>**构建块**：<br>- `Node`：矩形区域<br>- `Transform`：定位组件<br>- `Style`：Flex属性控制<br>**组件Bundle**：<br>- `NodeComponents`、`TextComponents`、`ImageComponents` | **战棋UI框架**：<br>- 回合信息面板<br>- 单位状态面板<br>- 技能面板<br>**设计价值**：<br>- ECS驱动：与游戏逻辑一致<br>- 复用核心系统：热重载、异步加载<br>- 统一架构：UI和游戏共享系统 |
| **UI系统** | ⭐️⭐️ Flexbox布局 | ✅ **布局引擎**：<br>- 基于Stretch库的纯Rust Flexbox实现<br>**定位方式**：<br>- 相对定位：默认相对于父节点<br>- 绝对定位：`PositionType::Absolute`<br>- 父子层级：子节点相对于父节点缩放<br>**Flex属性**：<br>- `justify_content`：主轴对齐<br>- `align_items`：交叉轴对齐<br>- `flex_direction`：主轴方向 | **战棋UI布局**：<br>- 主菜单：居中对齐<br>- 战斗界面：顶部回合信息，左侧单位列表，底部技能栏<br>- 弹窗：居中显示<br>- 适配不同分辨率 |
| **UI系统** | ⭐️ 交互事件 | ✅ **Interaction组件**：<br>- `Interaction::Clicked`：已点击<br>- `Interaction::Hovered`：悬停<br>- `Interaction::None`：无交互<br>✅ **变更检测**：<br>- `Mutated<Interaction>`：状态变化时触发 | **战棋按钮交互**：<br>- 攻击按钮：点击后进入攻击模式<br>- 移动按钮：点击后显示移动范围<br>- 待机按钮：结束当前单位行动 |
| **场景系统** | ⭐️⭐️⭐️ 场景概念 | ✅ **场景（Scene）**：<br>**设计理念**：实体和组件的集合，可多次生成<br>**生成方式**：<br>- `load`：保留实体ID（用于存档）<br>- `instance`：使用新ID（用于多实例）<br>**场景文件**：<br>- RON（Rusty Object Notation）格式<br>- 扁平实体列表<br>- 实体ID可选 | **战棋配置存储**：<br>- 地图配置：地形、单位初始位置<br>- 单位数据：属性、技能、装备<br>- 关卡配置：胜利条件、敌人配置<br>**设计价值**：<br>- 可序列化：支持存档/读档<br>- 可热重载：快速迭代<br>- 可复用：同一场景多实例 |
| **场景系统** | ⭐️⭐️ 场景热重载 | ✅ **热重载机制**：<br>**设计理念**：运行时自动应用场景文件变更<br>**工作原理**：基于属性系统的序列化/反序列化 | **战棋调试**：<br>- 修改地图文件后实时看到效果<br>- 调整单位属性后立即测试<br>- 快速迭代关卡设计 |
| **场景系统** | ⭐️⭐️ 场景序列化 | ✅ **保存/加载**：<br>- 保存：`Scene::from_world(&world, &registry)`<br>- 序列化：`scene.serialize_ron(&registry)`<br>- 加载：`asset_server.load("scene.scn")`<br>- 生成：`scene_spawner.load(handle)` / `scene_spawner.instance(handle)` | **战棋存档系统**：<br>- 保存当前游戏状态<br>- 读档恢复游戏状态<br>- 多存档槽位管理 |
| **属性系统** | ⭐️⭐️ 反射机制 | ✅ **bevy_property crate**：<br>**设计理念**：为Rust增加动态反射能力<br>**派生宏**：`#[derive(Properties)]`<br>**核心方法**：<br>- `set_prop_val::<T>("field", value)`：类型安全设置<br>- `prop_val::<T>("field")`：获取属性值<br>- `apply(&patch)`：应用属性补丁 | **战棋配置系统**：<br>- 单位属性动态访问<br>- 配置补丁：用新值修补<br>- 技能效果：动态修改属性 |
| **事件系统** | ⭐️⭐️ 双缓冲事件 | ✅ **事件机制**：<br>**设计理念**：高效生产/消费，零分配消费<br>**实现**：双缓冲EventBuffer<br>**使用方式**：<br>- 注册：`app.add_event::<MyEvent>()`<br>- 发送：`ResMut<Events<MyEvent>>`<br>- 消费：`EventReader<MyEvent>`<br>**限制**：每个系统只有一次机会接收事件 | **战棋事件**：<br>- UnitMoved：单位移动事件<br>- UnitAttacked：单位攻击事件<br>- UnitDied：单位死亡事件<br>- TurnEnded：回合结束事件 |
| **资源系统** | ⭐️⭐️ Assets集合 | ✅ **Assets<T>**：<br>**设计理念**：类型化资源集合<br>**Handle<T>**：引用机制<br>**资源事件**：<br>- `AssetEvent::Created`/`Modified`/`Removed` | **战棋资源管理**：<br>- 地图数据：`Assets<MapData>`<br>- 单位配置：`Assets<UnitConfig>`<br>- 技能配置：`Assets<SkillConfig>` |
| **资源系统** | ⭐️⭐️ 异步加载 | ✅ **AssetServer**：<br>**设计理念**：异步并行加载<br>**API**：<br>- `load("file.png")`返回Handle<br>- Handle可立即使用<br>- `load_asset_folder()`递归加载 | **战棋资源加载**：<br>- 地图数据异步加载<br>- 单位精灵/模型异步加载<br>- 启动时预加载常用资源 |
| **资源系统** | ⭐️ 热重载 | ✅ **热重载机制**：<br>- `watch_for_changes()`启用资源变更检测<br>✅ **扩展性**：<br>- 实现`AssetLoader` trait添加新资源类型 | **战棋开发**：<br>- 修改单位配置后自动更新<br>- 调整地图数据后实时预览 |
| **音效系统** | ⭐️ 音频播放 | ✅ **基础音频**：<br>- 加载音频文件（如MP3）<br>✅ **AudioOutput**：<br>- `play(handle)`：异步播放<br>- `play_source(source)`：立即播放 | **战棋音效**：<br>- 攻击音效<br>- 移动音效<br>- 背景音乐 |
| **插件架构** | ⭐️⭐️⭐️ 模块化设计 | ✅ **Plugin trait**：<br>**设计理念**：所有功能实现为Plugin<br>**API**：<br>- `build(&mut App)`方法注册系统和资源<br>**默认插件**：<br>- CorePlugin、InputPlugin、WindowPlugin<br>- RenderPlugin、UiPlugin<br>- `add_default_plugins()`添加所有 | **战棋模块化**：<br>- CombatPlugin：战斗系统<br>- MovementPlugin：移动系统<br>- AiPlugin：AI系统<br>- UiPlugin：UI系统<br>- 可独立开发、测试、替换 |
| **编译优化** | ⭐️⭐️ 快速编译配置 | ✅ **编译时间目标**：<br>- 0-1秒：理想<br>- 1-3秒：不错<br>- 3-5秒：烦人<br>✅ **配置三要素**：<br>- LLD链接器：比默认快很多<br>- Nightly Rust编译器<br>- 泛型共享：预编译泛型代码<br>✅ **迭代编译时间**：<br>- Linux：~0.8-3.0秒<br>- Windows：~1.5-3.0秒 | **开发效率**：<br>- 快速迭代<br>- 多平台开发<br>- 原型开发 |
| **跨平台** | ⭐️⭐️ 平台支持 | ✅ **当前支持**：<br>- Windows、MacOS、Linux<br>✅ **底层依赖**：<br>- winit：跨平台窗口和输入<br>- wgpu：跨平台渲染（Vulkan/DX12/Metal）<br>⚠️ **未来计划**：<br>- Android、iOS、Web | **战棋发布**：<br>- PC平台首发<br>- 移动端后续支持<br>- Web版本可能性 |
| **未来规划** | ⭐️ 后续版本预告 | ⚠️ **PBR渲染**：物理基础渲染着色器、阴影<br>⚠️ **编辑器**：在引擎内部构建<br>⚠️ **动画系统**：代码优先动画<br>⚠️ **物理引擎**：可插拔物理接口<br>⚠️ **其他**：Canvas绘图API、更好的场景格式 | **战棋扩展方向**：<br>- 3D战棋的PBR渲染<br>- 可视化地图编辑器<br>- 单位骨骼动画<br>- 碰撞检测和物理效果 |
