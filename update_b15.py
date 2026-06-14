#!/usr/bin/env python3
import re

CATALOG = "/Users/lf380/Code/Bevy/a1/bevy_examples_catalog.md"

DATA = [
    (
        141,
        "运行条件",
        [
            "⭐⭐ run_conditions.rs — Run Condition 控制 System 执行时机，短路组合器与参数化工厂函数",
            "- **.run_if() 基本用法**：<br>  - `.run_if(condition)` 附加运行条件，返回 false 时 System 不执行；<br>  - 条件可以是函数、闭包或内置组合器；<br>  - ⚠️ 条件不满足时整个系统不调度，比系统内 if 判断更高效；",
            "- **内置条件(resource_exists/not)**：<br>  - `resource_exists::<InputCounter>()` 检查资源是否存在——资源未初始化时系统安全跳过；<br>  - `not(condition)` 反转运行条件，如 `not(time_passed(2.5))` 表示「尚未超过 2.5 秒」；<br>  - 🔧 resource_exists 常用于确保 Resource 已 init 再访问，避免 Res 不存在时 panic；",
            "- **组合器 .or()/.and() 短路求值**：<br>  - `.or()` 第一个 false 才评估第二个，类似 `||`；<br>  - `.and()` 第一个 true 才评估第二个，类似 `&&`；<br>  - ⚠️ 顺序影响性能：高频/快速失败条件放前面，利用短路避免不必要的计算；",
            "- **自定义条件函数**：<br>  - `fn has_user_input(...) -> bool` 只读 SystemParam 返回 bool；<br>  - 所有参数必须只读（Local 除外），如 `Res<ButtonInput<KeyCode>>`；<br>  - 🔧 自定义函数比闭包更利于复用和调试（有函数名可追踪）；",
            "- **闭包条件 + 参数化工厂**：<br>  - `|counter: Res<T>| counter.is_changed() && !counter.is_added()` 内联闭包；<br>  - `fn time_passed(t: f32) -> impl FnMut(Local<f32>, Res<Time>) -> bool` 返回闭包的工厂函数；<br>  - 🔧 工厂函数实现「同逻辑不同参数」：time_passed(2.0) 和 time_passed(2.5) 复用同一代码；",
            "- **is_changed + is_added 过滤模式**：<br>  - `counter.is_changed() && !counter.is_added()` 过滤初始帧，仅在值真正变化时触发；<br>  - 🔧 此模式适用于「首次插入不触发，后续修改才触发」的场景（如 UI 刷新）；",
        ],
        [
            "resource_exists::<T>() / not() 内置条件；",
            ".or() / .and() 短路组合器；",
            "自定义 fn() -> bool 条件函数；",
            "闭包条件与参数化工厂函数；",
            "is_changed() && !is_added() 惯用过滤模式；",
        ],
        [
            "ecs/error_handling.rs(138)；",
            "ecs/fixed_timestep.rs(140)。",
        ],
    ),
    (
        142,
        "一次性系统",
        [
            "⭐⭐ one_shot_systems.rs — 一次性系统（One-Shot Systems）：按需触发的回调式系统执行",
            "- **commands.register_system() 注册**：<br>  - `commands.register_system(system_a)` 返回 `SystemId`，存储在组件中供后续触发；<br>  - SystemId 作为实体组件持有：`Callback(SystemId)` 模式；<br>  - 🔧 注册发生在 Startup 系统中，SystemId 在整个应用生命周期有效；",
            "- **World::run_system_once() 直接执行**：<br>  - `world.run_system_once(system_b)` 在 World 上直接执行一次系统；<br>  - ⚠️ 需要 `&mut World`，只能在 Exclusive 系统或非系统代码中使用；<br>  - 🔧 适合测试/初始化场景：不需要注册到调度即可执行；",
            "- **commands.run_system(id) 触发执行**：<br>  - `commands.run_system(callback.0)` 通过命令缓冲在帧末执行已注册的系统；<br>  - ⚠️ 不是立即执行——命令缓冲机制保证系统在当前帧安全完成后再运行；<br>  - 🧩 配合 Triggered 标记组件实现「条件触发」模式：insert(Triggered) → 下一帧 evaluate_callbacks 运行；",
            "- **回调模式设计**：<br>  - Triggered 标记组件标记「需要执行」的实体；<br>  - evaluate_callbacks 查询所有 (Callback, Triggered) 实体，运行系统后 remove::<Triggered>()；<br>  - 🔧 此模式将「何时触发」与「触发什么」解耦——trigger_system 控制时机，Callback 持有逻辑；",
            "- **应用场景**：<br>  - 🧩 UI 事件处理：点击按钮触发一次性系统更新数据；<br>  - 🧩 延迟初始化：实体创建后按需执行配置系统；<br>  - 🔧 减少空闲系统开销——不触发的系统零成本；",
        ],
        [
            "commands.register_system() → SystemId；",
            "World::run_system_once() 直接执行；",
            "commands.run_system(id) 命令缓冲触发；",
            "RunSystemOnce trait。",
        ],
        [
            "ecs/ecs_guide.rs(124)；",
            "ecs/error_handling.rs(138)。",
        ],
    ),
    (
        143,
        "系统管道",
        [
            "⭐⭐ system_piping.rs — 系统管道（System Piping）：将输出传入下一个系统，构建数据流管线",
            "- **.pipe() 管道连接**：<br>  - `parse_message_system.pipe(handler_system)` 将第一个系统的输出作为第二个系统的输入；<br>  - handler_system 使用 `In(result): In<Result<usize, ParseIntError>>` 接收管道输入；<br>  - 🔧 .pipe() 是构建「解析→处理→输出」管线的核心操作符；",
            '- **.map() 转换输出**：<br>  - `data_pipe_system.map(|out| info!("{out}"))` 将系统输出传给闭包处理；<br>  - `parse_message_system.map(drop)` 丢弃输出（不关心结果时使用）；<br>  - 🔧 .map() 适合轻量级后处理：日志打印、错误忽略、格式转换；',
            '- **Result 管道的错误处理**：<br>  - 系统返回 `Result<T, E>` → .pipe() 传递整个 Result 给下游；<br>  - `warning_pipe_system.map(|out| { if let Err(err) = out { error!("{err}"); } })` 处理错误分支；<br>  - ⚠️ 管道中的错误不会自动传播——需要显式 match/inspect_err 处理；',
            "- **系统输出类型**：<br>  - 任何系统都可以有输出：`fn data_pipe_system(msg: Res<Message>) -> String` 返回 String；<br>  - 🔧 输出类型必须是 `SystemOutput`（Send + 'static），不支持引用返回；",
            "- **组合模式**：<br>  - 多条管道可并行注册到同一 Schedule：(pipe_a, pipe_b, pipe_c)；<br>  - 🧩 管道 vs 独立系统：管道强调数据流动关系，独立系统强调并行独立性；<br>  - 🔧 管道系统适合「输入→转换→输出」的线性数据流场景；",
        ],
        [
            ".pipe() 连接两个系统形成管道；",
            ".map() 转换系统输出；",
            "In<T> 管道输入参数类型；",
            "系统返回 Result<T, E> 的错误处理。",
        ],
        [
            "ecs/error_handling.rs(138)；",
            "ecs/run_conditions.rs(141)。",
        ],
    ),
    (
        144,
        "自定义调度",
        [
            "⭐ custom_schedule.rs — 自定义 Schedule 注册到 Main 调度链，控制执行顺序和线程模型",
            "- **ScheduleLabel 派生**：<br>  - `#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]` 定义调度标签；<br>  - 标签是类型级标识符，用于注册和查找调度：`app.add_schedule(schedule)`；<br>  - 🔧 ScheduleLabel 必须实现 Hash + Eq 以支持 HashMap 存储；",
            "- **Schedule 创建与配置**：<br>  - `Schedule::new(SingleThreadedUpdate)` 创建新调度；<br>  - `schedule.set_executor_kind(ExecutorKind::SingleThreaded)` 设置单线程执行器；<br>  - ⚠️ 单线程执行器仅用于演示——生产环境通常用默认的并行执行器；",
            "- **MainScheduleOrder 注册**：<br>  - `main_schedule_order.insert_after(Update, SingleThreadedUpdate)` 在 Update 之后运行自定义调度；<br>  - ⚠️ 必须在 main() 中直接修改 MainScheduleOrder，不能在系统中修改（因为调度正在运行）；<br>  - 🔧 insert_after/insert_before 控制调度在 Main 链中的位置；",
            "- **自定义 Startup 调度**：<br>  - `main_schedule_order.insert_startup_after(PreStartup, CustomStartup)` 在 PreStartup 之后插入自定义启动调度；<br>  - 🔧 自定义 Startup 调度适合按阶段分离初始化逻辑；",
            "- **Main 调度执行顺序**：<br>  - PreStartup → CustomStartup → Startup → First → Update → SingleThreadedUpdate → Last；<br>  - 🧩 调度顺序决定系统执行的全局时序——自定义调度是扩展执行管线的关键机制；",
        ],
        [
            "ScheduleLabel 派生宏；",
            "Schedule::new() + set_executor_kind()；",
            "MainScheduleOrder::insert_after() / insert_startup_after()；",
            "ExecutorKind::SingleThreaded 单线程执行器。",
        ],
        [
            "ecs/ecs_guide.rs(124)；",
            "ecs/fixed_timestep.rs(140)。",
        ],
    ),
    (
        145,
        "自定义查询参数",
        [
            "⭐⭐ custom_query_param.rs — 自定义 QueryData/QueryFilter 派生宏，构建可复用的命名查询类型",
            "- **#[derive(QueryData)] 派生**：<br>  - 定义命名结构体替代元组查询：`struct ReadOnlyCustomQuery { entity, a, b, nested }`；<br>  - 避免 `q.0, q.1` 的元组访问模式，提升可读性；<br>  - 🔧 命名字段增删不影响迭代代码——比元组查询更易维护；",
            "- **嵌套查询与泛型**：<br>  - `nested: NestedQuery` 在查询中嵌套另一个 QueryData 结构体；<br>  - `GenericQuery<T, P>` 支持泛型参数，实现查询类型复用；<br>  - 🧩 组合模式：ReadOnlyCustomQuery<ComponentC, ComponentD> 组装出具体查询；",
            "- **#[query_data(mutable)] 可变查询**：<br>  - 添加 `mutable` 属性生成可变版本和只读版本（自动加 ReadOnly 后缀）；<br>  - 可变版本使用 `&'static mut`，只读版本使用 `&'static`；<br>  - ⚠️ mutable 查询在迭代时独占组件访问，注意系统参数冲突；",
            "- **#[derive(QueryFilter)] 自定义过滤器**：<br>  - `struct CustomQueryFilter { _c: With<C>, _or: Or<(Added<C>, Without<Z>)> }` 组合过滤条件；<br>  - 支持泛型过滤器参数：`_generic_tuple: (With<T>, With<P>)`；<br>  - 🔧 过滤器与查询分离定义，可在多个查询间复用同一过滤逻辑；",
            "- **EmptyQuery 遍历所有实体**：<br>  - `struct EmptyQuery { empty: () }` 匹配所有实体（无过滤条件）；<br>  - 🔧 用于需要遍历全部实体的全局操作（如调试打印）；",
            "- **优势总结**：<br>  - 突破元组查询 15 个组件的限制；<br>  - 命名字段便于 IDE 自动补全和重构；<br>  - 组合模式实现查询类型的模块化复用；",
        ],
        [
            "#[derive(QueryData)] / #[derive(QueryFilter)] 派生宏；",
            "#[query_data(mutable)] 可变/只读双版本；",
            "嵌套 QueryData 和泛型参数；",
            "CustomQueryItem / CustomQueryReadOnlyItem 类型。",
        ],
        [
            "ecs/ecs_guide.rs(124)；",
            "ecs/system_param.rs(129)。",
        ],
    ),
    (
        146,
        "并行查询",
        [
            "⭐ parallel_query.rs — 并行迭代器 par_iter_mut() 利用 ComputeTaskPool 多线程处理大量实体",
            "- **par_iter_mut() 并行迭代**：<br>  - `sprites.par_iter_mut().for_each(|(mut transform, velocity)| { ... })` 并行修改每个实体；<br>  - ⚠️ 仅当实体数量大且操作开销高时才有性能收益——128 个简单加法可能更慢；<br>  - 🔧 并行迭代自动将实体分批到 ComputeTaskPool 的多个线程；",
            "- **BatchingStrategy 批次控制**：<br>  - `.batching_strategy(BatchingStrategy::fixed(32))` 设置每批 32 个实体；<br>  - 默认批次大小由 Bevy 根据实体数和线程数自动计算；<br>  - ⚠️ 批次太小→线程调度开销大；批次太大→负载不均衡；<br>  - 🔧 简单操作（如向量取反）用小批次减少并行开销；",
            "- **并行安全性**：<br>  - par_iter_mut 要求每个实体的数据访问互不重叠（ECS 天然保证）；<br>  - 不同实体的 Transform 和 Velocity 可安全并行修改；<br>  - ⚠️ 同一系统内不能同时 par_iter_mut 两个有交集的 Query；",
            "- **性能考量**：<br>  - 🔬 并行迭代有固定开销（任务分发+合并），小数据集得不偿失；<br>  - 🔬 实体数量 > 1000 且操作为复杂计算时收益明显；<br>  - 🔧 先用普通 iter 测试，确认瓶颈后再切换 par_iter；",
            "- **ChaCha8Rng 确定性随机**：<br>  - `ChaCha8Rng::seed_from_u64(seed)` 确定性随机种子保证测试可复现；<br>  - 🔧 发布版可用系统熵替换固定种子；",
        ],
        [
            "Query::par_iter_mut() 并行迭代；",
            "BatchingStrategy::fixed(N) 批次大小控制；",
            "ComputeTaskPool 线程池；",
            "ChaCha8Rng 确定性随机。",
        ],
        [
            "ecs/ecs_guide.rs(124)；",
            "ecs/iter_combinations.rs(147)。",
        ],
    ),
    (
        147,
        "组合遍历",
        [
            "⭐⭐ iter_combinations.rs — iter_combinations_mut() 遍历实体两两组合，实现 N-body 引力交互",
            "- **iter_combinations_mut() 核心**：<br>  - `query.iter_combinations_mut()` 返回所有可能的两两组合 `[(Mass, &GlobalTransform, &mut Acceleration); 2]`；<br>  - `while let Some([pair1, pair2]) = iter.fetch_next()` 逐对获取组合；<br>  - 🔧 与嵌套 for 循环不同：组合遍历确保每对只访问一次，避免重复计算；",
            "- **N-body 引力模拟**：<br>  - `GRAVITY_CONSTANT / distance_sq` 计算引力强度（平方反比定律）；<br>  - `acc1 += force_unit_mass * m2; acc2 -= force_unit_mass * m1` 牛顿第三定律对称施力；<br>  - 🧩 力的对称性：A 对 B 的力 = -(B 对 A 的力)，一次计算同时更新两个加速度；",
            "- **Verlet 积分**：<br>  - `new_pos = 2 * pos - last_pos + acc * dt²` 二阶精度时间积分；<br>  - 比欧拉积分更稳定：不需要存储速度，用位置差分隐式表示速度；<br>  - 🔧 Verlet 积分适合约束系统和粒子模拟，能量守恒性优于显式欧拉；",
            "- **FixedUpdate + Update 分离**：<br>  - `interact_bodies` 和 `integrate` 在 FixedUpdate 中执行（确定性物理步长）；<br>  - `look_at_star` 在 Update 中执行（相机跟随每帧平滑）；<br>  - 🔧 物理模拟必须在 FixedUpdate 保证固定时间步，渲染插值在 Update；",
            "- **Mass 驱动缩放**：<br>  - `Transform::from_scale(Vec3::splat(radius))` 用质量相关半径缩放球体；<br>  - 🔧 视觉大小与物理质量关联——直观表达质量差异；",
        ],
        [
            "Query::iter_combinations_mut() 两两组合遍历；",
            "fetch_next() 逐对获取组合；",
            "Verlet 积分时间步进算法；",
            "FixedUpdate 固定步长调度。",
        ],
        [
            "ecs/parallel_query.rs(146)；",
            "ecs/fixed_timestep.rs(140)。",
        ],
    ),
    (
        148,
        "非确定性顺序",
        [
            "⭐ nondeterministic_system_order.rs — 系统执行顺序歧义检测与 .after()/.ambiguous_with() 解决方案",
            "- **执行顺序歧义问题**：<br>  - Bevy 默认并行执行系统，无依赖的系统执行顺序不确定；<br>  - 两个系统修改同一资源时，执行顺序不同导致结果不同（非确定性 Bug）；<br>  - ⚠️ 读写冲突（reads_a vs writes_a）是最常见的歧义类型；",
            "- **ScheduleBuildSettings 歧义检测**：<br>  - `schedule.set_build_settings(ScheduleBuildSettings { ambiguity_detection: LogLevel::Warn, .. })` 启用检测；<br>  - ⚠️ 仅检测直接配置的 Schedule——子调度不继承检测设置；<br>  - 🔧 DefaultPlugins 内部也可能有歧义，需向插件维护者报告；",
            "- **.after() 显式排序**：<br>  - `doubles_b.after(adds_one_to_b)` 强制 doubles 在 adds 之后执行；<br>  - 传递性：A before B before C → A before C；<br>  - 🔧 .after() 是解决歧义的首选方式——明确表达依赖关系；",
            "- **.ambiguous_with() 静默误报**：<br>  - `reads_a_and_b.ambiguous_with(adds_one_to_b)` 声明「已知歧义但无害」；<br>  - ⚠️ 仅用于明确的误报：两个系统的读写冲突不影响正确性；<br>  - 🔧 必须附注释说明为何安全，否则后续维护者可能忽略真正的歧义；",
            "- **数据访问由类型决定**：<br>  - Bevy 根据系统参数类型推断读写访问（Res<T>=读，ResMut<T>=写）；<br>  - 🔧 SystemParam/WorldQuery 的 trait 实现必须声明访问模式；",
        ],
        [
            "ScheduleBuildSettings { ambiguity_detection } 歧义检测；",
            ".after() 显式排序解决歧义；",
            ".ambiguous_with() 静默已知无害歧义；",
            "LogLevel::Warn/Info/Error 检测级别。",
        ],
        [
            "ecs/ecs_guide.rs(124)；",
            "ecs/system_stepping.rs(153)。",
        ],
    ),
    (
        149,
        "实体禁用",
        [
            "⭐⭐ entity_disabling.rs — Disabled 组件禁用实体：跳过所有 Query 但不删除实体",
            "- **Disabled 组件机制**：<br>  - 插入 `Disabled` 组件后，实体被所有默认 Query 过滤器自动排除；<br>  - ⚠️ 不是 Visibility::Hidden——Disabled 实体完全跳过 ECS 处理（系统看不到它）；<br>  - 🔧 适合「休眠」场景：离屏实体、暂停的网络实体、临时禁用的 NPC；",
            "- **默认过滤器行为**：<br>  - Bevy 内部使用「默认查询过滤器」跳过 Disabled 实体；<br>  - 所有常规 Query 自动排除 Disabled 实体，无需手动 With/Without；<br>  - 🔧 过滤在调度层实现，比系统内 if 判断更高效（零运行时开销）；",
            "- **绕过过滤器**：<br>  - `Query<Entity, With<Disabled>>` 显式包含 Disabled 实体；<br>  - `Query<(Entity, Has<Disabled>)>` 检查实体是否被禁用；<br>  - `Query<&A, Or<(With<Disabled>, With<B>)>>` 组合条件绕过；<br>  - ⚠️ 仅在需要操作 Disabled 实体时才绕过——常规系统不应接触禁用实体；",
            "- **insert/remove 启用/禁用**：<br>  - `commands.entity(e).insert(Disabled)` 禁用实体；<br>  - `commands.entity(e).remove::<Disabled>()` 重新启用实体；<br>  - 🔧 插入/移除 Disabled 不影响实体的其他组件和子实体关系；",
            "- **与 Visibility 的区别**：<br>  - Visibility::Hidden：实体仍在 Query 中可见，只是不渲染；<br>  - Disabled：实体完全从 Query 中消失，系统无法访问；<br>  - ⚠️ Disabled 不递归到子实体——子实体仍可被 Query 匹配；",
        ],
        [
            "Disabled 组件（bevy::ecs::entity_disabling）；",
            "默认查询过滤器自动排除 Disabled；",
            "With<Disabled> / Has<Disabled> 绕过过滤。",
        ],
        [
            "ecs/ecs_guide.rs(124)；",
            "ecs/hierarchy.rs(126)。",
        ],
    ),
    (
        150,
        "实体关系",
        [
            "⭐⭐ relationships.rs — 自定义实体关系（Relationship/RelationshipTarget）实现复杂图结构",
            "- **#[relationship] 派生**：<br>  - `#[relationship(relationship_target = TargetedBy)] struct Targeting(Entity)` 定义关系源；<br>  - 关系源是「真相来源」——直接修改 Targeting 组件即改变关系；<br>  - 🔧 Targeting(Entity) 持有目标实体 ID，类似 ChildOf 的自定义版本；",
            "- **#[relationship_target] 派生**：<br>  - `#[relationship_target(relationship = Targeting)] struct TargetedBy(Vec<Entity>)` 反向索引；<br>  - TargetedBy 由 Bevy 自动维护——插入/移除 Targeting 时自动更新；<br>  - ⚠️ 不要直接修改 TargetedBy——它是缓存数据，通过关系源操作间接更新；",
            '- **with_related / with_related_entities**：<br>  - `.with_related::<Targeting>(Name::new("James"))` spawn 时建立关系；<br>  - `.with_related_entities::<Targeting>(|spawner| { spawner.spawn(...); })` 闭包内 spawn 的实体自动建立关系；<br>  - 🔧 比手动 insert Targeting 更符合人体工程学；',
            "- **关系变更**：<br>  - `commands.entity(devon).insert(Targeting(alice))` 替换关系——自动更新 TargetedBy；<br>  - `commands.entity(charlie).remove::<Targeting>()` 移除关系——断开连接；<br>  - ⚠️ 关系组件不可变（不能 get_mut 修改），只能 insert 替换或 remove 移除；",
            "- **关系遍历与环检测**：<br>  - `query.iter_ancestors(entity)` 沿关系链向上遍历祖先；<br>  - `EntityHashSet` 记录已访问实体检测循环——环会导致无限遍历；<br>  - 🧩 战棋游戏中可建模「瞄准→被瞄准」「治疗→被治疗」等双向关系；",
        ],
        [
            "#[relationship] / #[relationship_target] 派生宏；",
            "with_related / with_related_entities 辅助方法；",
            "iter_ancestors() 关系链遍历；",
            "Relationship / RelationshipTarget trait。",
        ],
        [
            "ecs/hierarchy.rs(126)；",
            "ecs/component_hooks.rs(134)。",
        ],
    ),
    (
        151,
        "状态作用域",
        [
            "⭐⭐ state_scoped.rs — DespawnOnExit/DespawnOnEnter 在状态切换时自动销毁实体",
            "- **DespawnOnExit 组件**：<br>  - `DespawnOnExit(GameState::A)` 标记实体在离开状态 A 时自动销毁；<br>  - 离开 A → 进入 B 时，所有 DespawnOnExit(A) 实体被 despawn；<br>  - 🔧 适合状态专属 UI/场景实体的自动清理——无需手动 cleanup 系统；",
            "- **DespawnOnEnter 组件**：<br>  - `DespawnOnEnter(GameState::A)` 标记实体在进入状态 A 时自动销毁；<br>  - 进入 A → 离开 B 时，所有 DespawnOnEnter(B) 实体被 despawn；<br>  - 🔧 适合「状态过渡期间的临时提示」：显示后在下一次进入时消失；",
            "- **子实体继承**：<br>  - `children![DespawnOnExit(GameState::A)]` 子实体也可标记相同的作用域；<br>  - ⚠️ 父实体被 despawn 时子实体自然也被删除——但复杂层级中显式标记更安全；<br>  - 🔧 状态作用域标记可以同时在父和子上存在，Bevy 处理重复 despawn 不报错；",
            "- **与 OnExit/OnEnter 系统对比**：<br>  - OnExit(A) 系统需要手动 `commands.entity(e).despawn()` 清理；<br>  - DespawnOnExit(A) 声明式自动清理——减少样板代码；<br>  - 🧩 两种方式可组合：DespawnOnExit 处理 UI 实体，OnExit 系统处理资源清理；",
            "- **定时器驱动状态切换**：<br>  - `Timer::from_seconds(1.0, TimerMode::Repeating)` 每秒切换状态；<br>  - `NextState::Pending(state)` 设置下一状态——比 set() 更安全（延迟生效）；<br>  - 🔧 Pending 确保状态切换在当前帧所有系统完成后生效；",
        ],
        [
            "DespawnOnExit<S>(state) 组件；",
            "DespawnOnEnter<S>(state) 组件；",
            "子实体继承 DespawnOnExit/Enter；",
            "NextState::Pending() 延迟状态切换。",
        ],
        [
            "ecs/ecs_guide.rs(124)；",
            "state/states.rs(154)。",
        ],
    ),
    (
        152,
        "动态组件",
        [
            "⚠️ dynamic.rs — 运行时动态创建组件和实体（unsafe，底层 API）",
            "- **ComponentDescriptor 动态注册**：<br>  - `ComponentDescriptor::new_with_layout(name, StorageType::Table, layout, None, true, CloneBehavior, None)` 动态创建组件；<br>  - Layout::array::<u64>(size) 按 u64 数组大小分配内存；<br>  - ⚠️ unsafe 代码——必须保证 Layout 和数据布局匹配；",
            "- **world.register_component_with_descriptor()**：<br>  - 将动态组件注册到 World，返回 ComponentId；<br>  - ComponentId 是运行时标识符（区别于类型级的 ComponentId）；<br>  - 🔧 用于 MOD 系统、脚本语言绑定等需要运行时定义组件的场景；",
            "- **entity.insert_by_ids() 批量插入**：<br>  - `entity.insert_by_ids(&component_ids, owning_ptrs)` 按 ComponentId 批量插入数据；<br>  - OwningPtr 是裸指针包装——数据必须按组件描述的 Layout 布局；<br>  - ⚠️ 指针必须指向有效内存，生命周期由调用方保证；",
            "- **QueryBuilder 动态查询**：<br>  - `QueryBuilder::<FilteredEntityMut>::new(&mut world)` 构建动态查询；<br>  - `builder.ref_id(id)` / `builder.mut_id(id)` 按 ComponentId 添加读/写访问；<br>  - `builder.with_id(id)` / `builder.optional(|b| ...)` / `builder.or(|b| ...)` 组合过滤；<br>  - 🔧 QueryBuilder 是运行时构建查询的唯一方式——静态查询在编译期确定；",
            "- **get_by_id / get_mut_by_id**：<br>  - `filtered_entity.get_by_id(id)` 按 ComponentId 读取组件数据（返回 OwnedPtr）；<br>  - 不可变组件调用 get_mut_by_id 会返回 Err——运行时不可变保证；<br>  - 🔧 与 immutable_components.rs(135) 配合展示动态不可变组件的完整生命周期；",
        ],
        [
            "ComponentDescriptor::new_with_layout() 动态组件注册；",
            "insert_by_ids() / get_by_id() 按 ComponentId 操作；",
            "QueryBuilder 动态查询构建；",
            "FilteredEntityMut 过滤实体可变访问。",
        ],
        [
            "ecs/immutable_components.rs(135)；",
            "ecs/ecs_guide.rs(124)。",
        ],
    ),
    (
        153,
        "步进调试",
        [
            "⭐ system_stepping.rs — Stepping 资源逐步调试 System 执行，支持断点和跳过",
            "- **Stepping 基本配置**：<br>  - `Stepping::new()` 创建步进调试资源；<br>  - `stepping.add_schedule(Update)` 将 Update 调度加入步进控制；<br>  - `stepping.enable()` 启用步进模式——未步进的调度系统照常运行；<br>  - ⚠️ 需要 `bevy_debug_stepping` feature 才能使用；",
            "- **step_frame() 单步执行**：<br>  - `stepping.step_frame()` 标记下一帧只执行一个系统；<br>  - 执行完一个系统后自动暂停——PreUpdate 系统不受影响（未加入 Stepping）；<br>  - 🔧 调试复杂交互时精确定位问题系统——避免全量执行的噪声；",
            "- **continue_frame() 继续执行**：<br>  - `stepping.continue_frame()` 从当前位置执行到帧末尾或下一个断点；<br>  - 🔧 比 step_frame 更实用：跳过无问题的系统，停在可疑系统处；",
            "- **always_run / never_run 覆盖**：<br>  - `stepping.always_run(Update, system_two)` 强制某系统每帧都执行（无视步进）；<br>  - `stepping.never_run(Update, system_two)` 跳过某系统（即使 step 到它）；<br>  - 🔧 always_run 适合「背景系统」：日志、性能监控等不应被步进中断；",
            "- **set_breakpoint / clear_breakpoint 断点**：<br>  - `stepping.set_breakpoint(Update, system_two)` 在指定系统处暂停；<br>  - continue → 执行到断点处暂停 → step 执行断点系统 → continue 继续到下一断点；<br>  - 🔧 断点 + chain 依赖：若 system_one → system_two → system_three，断点 system_two 会跳过 system_one；",
        ],
        [
            "Stepping 资源（bevy_debug_stepping feature）；",
            "step_frame() / continue_frame() 步进控制；",
            "always_run() / never_run() 系统覆盖；",
            "set_breakpoint() / clear_breakpoint() 断点。",
        ],
        [
            "ecs/nondeterministic_system_order.rs(148)；",
            "ecs/ecs_guide.rs(124)。",
        ],
    ),
    (
        154,
        "状态管理",
        [
            "⭐⭐⭐ states.rs — States 枚举驱动高级别应用控制流，OnEnter/OnExit/setup+cleanup 模式",
            "- **States 派生与 init_state**：<br>  - `#[derive(States)] enum AppState { Menu, InGame }` 定义状态枚举；<br>  - `app.init_state::<AppState>()` 初始化状态资源（等价于 insert_state 默认值）；<br>  - 🔧 States 必须实现 Clone + Copy + Eq + Hash + Default + 'static；",
            "- **OnEnter/OnExit 调度**：<br>  - `add_systems(OnEnter(AppState::Menu), setup_menu)` 进入状态时运行；<br>  - `add_systems(OnExit(AppState::Menu), cleanup_menu)` 离开状态时运行；<br>  - ⚠️ Exit 在 Enter 之前执行——先清理旧状态再建立新状态；",
            "- **in_state() 条件运行**：<br>  - `.run_if(in_state(AppState::Menu))` 仅在 Menu 状态下运行 Update 系统；<br>  - 🔧 比在系统内 `if *state.get() == Menu` 更高效——条件不满足时系统不调度；",
            "- **NextState 状态切换**：<br>  - `next_state.set(AppState::InGame)` 立即设置下一状态；<br>  - ⚠️ set() 在当前帧内立即生效——可能影响同帧其他系统；<br>  - 🔧 配合 setup_menu 返回 Entity ID + MenuData Resource 实现 cleanup 精准销毁；",
            "- **setup + cleanup 模式**：<br>  - OnEnter 创建实体并记录 Entity ID 到 Resource（MenuData）；<br>  - OnExit 用 Resource 中的 ID 精准 despawn——避免全量清理的性能浪费；<br>  - 🧩 此模式是 Bevy 状态管理的核心范式：setup → 运行 → cleanup；",
        ],
        [
            "#[derive(States)] 状态枚举定义；",
            "OnEnter / OnExit 调度注册；",
            "in_state() 条件运行系统；",
            "NextState::set() 状态切换。",
        ],
        [
            "ecs/ecs_guide.rs(124)；",
            "ecs/generic_system.rs(127)。",
        ],
    ),
    (
        155,
        "子状态",
        [
            "⭐⭐ sub_states.rs — SubStates 从属状态：仅在父状态激活时存在的子状态",
            "- **SubStates 派生**：<br>  - `#[derive(SubStates)] #[source(AppState = AppState::InGame)]` 声明子状态依赖；<br>  - IsPaused 仅在 AppState::InGame 时存在——离开 InGame 时 IsPaused 资源自动移除；<br>  - 🔧 子状态简化复杂状态组合：暂停/加速/教程等只在游戏中的状态；",
            "- **add_sub_state 注册**：<br>  - `app.add_sub_state::<IsPaused>()` 注册子状态（在 init_state 之后）；<br>  - ⚠️ 子状态的默认值在父状态激活时自动插入；",
            "- **#[states(scoped_entities)]**：<br>  - 标记子状态使用作用域实体——OnEnter/OnExit 时自动管理实体生命周期；<br>  - 🔧 配合 DespawnOnExit(IsPaused::Paused) 实现暂停界面自动清理；",
            "- **子状态条件系统**：<br>  - `.run_if(in_state(IsPaused::Running))` 仅在未暂停时运行游戏逻辑；<br>  - `.run_if(in_state(AppState::InGame))` 暂停切换按钮在任何 InGame 子状态都可运行；<br>  - 🧩 父状态条件覆盖所有子状态——子状态条件仅在父状态激活时有意义；",
            "- **OnEnter 子状态 UI**：<br>  - `add_systems(OnEnter(IsPaused::Paused), setup_paused_screen)` 暂停时显示 UI；<br>  - DespawnOnExit(IsPaused::Paused) 自动在恢复时销毁暂停界面；<br>  - 🔧 子状态的 OnEnter/OnExit 仅在父状态范围内触发——离开 InGame 不会触发 IsPaused 的 Exit；",
        ],
        [
            "#[derive(SubStates)] + #[source(AppState = X)] 子状态定义；",
            "add_sub_state::<T>() 注册；",
            "子状态在父状态激活时存在，离开时自动移除；",
            "#[states(scoped_entities)] 作用域实体。",
        ],
        [
            "state/states.rs(154)；",
            "state/computed_states.rs(156)。",
        ],
    ),
    (
        156,
        "计算状态",
        [
            "⭐⭐⭐ computed_states.rs — ComputedStates 从源状态派生的自动计算状态，支持多源组合",
            "- **ComputedStates trait 实现**：<br>  - `impl ComputedStates for InGame { type SourceStates = AppState; fn compute(sources) -> Option<Self> }`；<br>  - compute() 返回 Some 时状态存在，None 时状态不存在；<br>  - 🔧 ZST 标记状态（如 InGame/TurboMode）用 struct 而非 enum——仅表示存在/不存在；",
            "- **枚举计算状态**：<br>  - `enum IsPaused { NotPaused, Paused }` 有多个值的计算状态用 enum；<br>  - compute() 映射源状态到子状态：InGame{paused:true} → IsPaused::Paused；<br>  - 🧩 ZST vs enum：无变化用 struct，有多种模式用 enum；",
            "- **多源组合派生**：<br>  - `type SourceStates = (TutorialState, InGame, Option<IsPaused>)` 多源元组；<br>  - `Option<T>` 包裹可选源状态——源不存在时 compute 仍被调用；<br>  - 🔧 派生其他 ComputedState 避免逻辑重复——但禁止循环依赖（编译错误）；",
            "- **add_computed_state 注册**：<br>  - `app.add_computed_state::<InGame>()` 注册计算状态；<br>  - 计算状态不需要手动 set——由源状态变化自动触发重算；<br>  - ⚠️ 源状态变化 → 所有依赖的 ComputedState 重新 compute()；",
            "- **DespawnOnExit 与计算状态**：<br>  - `DespawnOnExit(InGame)` 当 InGame 不存在时销毁实体；<br>  - `DespawnOnExit(TurboMode)` TurboMode 不存在时销毁 turbo 文字；<br>  - 🔧 计算状态的 OnEnter/OnExit 在 compute 结果变化时触发——与普通状态行为一致；",
            "- **设计原则**：<br>  - 🔧 将复杂 AppState 的多种组合拆分为独立计算状态——每个系统只关心自己关注的状态；<br>  - 🧩 运行时检查 `Option<Res<State<TurboMode>>>` 获取计算状态值——不存在时为 None；",
        ],
        [
            "impl ComputedStates for T 派生实现；",
            "type SourceStates 源状态类型声明；",
            "compute() -> Option<Self> 计算函数；",
            "add_computed_state::<T>() 注册；",
            "Option<T> 可选源状态。",
        ],
        [
            "state/states.rs(154)；",
            "state/sub_states.rs(155)。",
        ],
    ),
    (
        157,
        "自定义转换",
        [
            "⭐ custom_transitions.rs — 自定义 OnReenter/OnReexit 转换：响应同一状态的重复进入/离开",
            "- **问题背景**：<br>  - 默认 OnEnter/OnExit 不触发恒等转换（InGame→InGame 被忽略）；<br>  - 实际需求：重新进入同一状态时重新初始化（如重启关卡）；",
            "- **OnReenter/OnReexit 自定义调度**：<br>  - `OnReenter(S)` 自定义调度标签，在恒等转换时触发；<br>  - `OnReexit(S)` 在恒等转换离开时触发；<br>  - 🔧 与 OnEnter/OnExit 语义一致但额外响应重复进入；",
            "- **IdentityTransitionsPlugin**：<br>  - `IdentityTransitionsPlugin::<AppState>::default()` 插件封装自定义转换逻辑；<br>  - 在 StateTransition 调度中注册 `last_transition::<S>.pipe(run_reenter::<S>)`；<br>  - 🔧 插件模式封装复杂逻辑——使用者只需 `.add_plugins(IdentityTransitionsPlugin::<T>::default())`；",
            "- **StateTransitionEvent 内部机制**：<br>  - `last_transition::<S>` 获取最近一次转换事件 `Option<StateTransitionEvent<S>>`；<br>  - transition.entered / transition.exited 标识进入和离开的状态；<br>  - 🔧 恒等转换：entered == exited 时默认忽略，自定义逻辑可检测此情况；",
            "- **EnterSchedules / ExitSchedules 集成**：<br>  - `.in_set(EnterSchedules::<S>::default())` 将自定义系统插入转换调度链；<br>  - ExitSchedules 从叶状态到根状态执行，EnterSchedules 从根到叶；<br>  - 🧩 与 ComputedStates 的转换传播兼容——父状态恒等转换会传播到子状态；",
            "- **实际应用**：<br>  - `next_state.set(AppState::InGame)` 重复设置当前状态触发恒等转换；<br>  - setup_game → teardown_game → setup_game 完整重启流程；<br>  - 🔧 比「先切换到 Menu 再切回 InGame」更简洁——无需中间状态；",
        ],
        [
            "OnReenter<S> / OnReexit<S> 自定义调度标签；",
            "StateTransitionEvent 转换事件；",
            "EnterSchedules / ExitSchedules 调度集；",
            "IdentityTransitionsPlugin 插件封装。",
        ],
        [
            "state/states.rs(154)；",
            "state/computed_states.rs(156)。",
        ],
    ),
]


def build_entry_line(seq, category, intent_lines, knowledge, examples):
    intent = "<br>".join(intent_lines)
    knowledge_str = "<br>".join(f"- {k}" for k in knowledge)
    examples_str = "<br>".join(f"- {e}" for e in examples)
    return (
        f"| {seq} | {category} | `{intent_lines[0].split('—')[0].strip().split(' ')[-1]}` "
        f"| {intent} | 知：<br>{knowledge_str}<br>例：<br>{examples_str} |"
    )


def main():
    with open(CATALOG, "r", encoding="utf-8") as f:
        content = f.read()

    lines = content.split("\n")

    start_idx = None
    for i, line in enumerate(lines):
        if re.match(r"\|\s*141\s*\|", line):
            start_idx = i
            break

    if start_idx is None:
        print("ERROR: Could not find entry 141")
        return

    end_idx = None
    for i in range(start_idx, len(lines)):
        if re.match(r"\|\s*158\s*\|", lines[i]):
            end_idx = i + 1
            break

    if end_idx is None:
        print("ERROR: Could not find end of entries 141-156")
        return

    print(f"Old block: lines {start_idx + 1} to {end_idx + 1}")

    new_entries = []
    for seq, cat, intent, knowledge, examples in DATA:
        new_entries.append(build_entry_line(seq, cat, intent, knowledge, examples))

    header_lines = lines[start_idx - 2 : start_idx]
    replacement = header_lines + new_entries + ["", "---"]

    before = lines[: start_idx - 2]
    after = lines[end_idx:]

    new_lines = before + replacement + after

    renumber_from = len(before) + len(replacement)
    old_entry_count = 18
    new_entry_count = len(new_entries)
    shift = new_entry_count - old_entry_count

    renumbered = []
    for i, line in enumerate(new_lines):
        m = re.match(r"^(\|\s*)(\d+)(\s*\|)", line)
        if m and i >= renumber_from:
            old_num = int(m.group(2))
            new_num = old_num + shift
            line = f"{m.group(1)}{new_num}{m.group(3)}{line[m.end() :]}"
        renumbered.append(line)

    with open(CATALOG, "w", encoding="utf-8") as f:
        f.write("\n".join(renumbered))

    print(
        f"Done! {len(new_entries)} entries (141-157), downstream renumbered {shift:+d}"
    )


if __name__ == "__main__":
    main()
