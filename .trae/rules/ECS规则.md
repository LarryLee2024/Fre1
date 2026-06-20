---
alwaysApply: false
description: 
---
# Bevy 0.19+ SRPG 架构宪法 v5.2 · ECS篇
> 定位：Bevy ECS 最佳实践与编码规则，负责所有引擎层代码规范
> 适用场景：编写系统、组件、通信逻辑、调度配置、状态管理

## 效力说明
1. 本篇约束所有ECS相关代码的编写方式，优先级高于通用Rust编码规范
2. 专为Bevy 0.19及以上版本设计，贴合官方原生能力与演进方向
3. 条款强制等级与全文一致，违反条款的代码视为不合格输出

---

## 一、ECS核心概念
### 1.1 Entity
- 🟩 Entity只是纯ID，**绝对禁止**在Entity上调用方法或当作面向对象实例使用
- 允许将Entity作为ID参数传递，禁止承载任何行为
- 🟥 **Domain层禁止裸Entity**：Domain/Application层禁止裸`Entity`、`u64`、`usize`作为业务对象标识
  - 只允许使用显式命名的强类型ID（如`UnitId`、`AbilityId`）
  - Entity ↔ 业务ID映射集中在Infrastructure层（`integration/`模块）
  - 详见 `docs/04-data/foundation/id-taxonomy.md`

### 1.2 数据与行为分离
- 🟥 Component只能存储纯数据状态，绝对禁止包含任何逻辑
- 🟥 System只能包含纯逻辑，绝对禁止存储任何状态
- 规则本质：ECS的核心优势就是数据行为分离带来的并行性与可维护性

### 1.3 Tag Component原则
- 🟩 实体持久状态优先使用空Tag Component实现
  - 合法：`Dead`、`Frozen`、`Stunned`
  - 非法：`is_dead: bool`、`is_frozen: bool`
- 配置字段、临时计算值、非实体级状态允许使用bool类型
  - 合法：`is_unique: bool`（配置属性）、`is_critical: bool`（结算临时值）

### 1.4 从属关系
- 🟨 实体间从属关系（Buff归属单位、背包归属角色）优先使用Bevy官方`Relationship`机制
- 禁止手动维护父子实体ID字段

---

## 二、四级通信机制
### 2.1 Hook = 组件生命周期固有行为
- 🟩 组件添加/移除的轻量副作用，通过`#[component(on_add=..., on_remove=...)]`声明
- 仅用于简单状态联动，禁止承载复杂业务逻辑
- 示例：`#[component(on_add=remove_moveable)] struct Dead;`

### 2.2 Trigger = Feature内事件链载体
- 🟩 同一Feature内的多段响应逻辑、战斗事件链，必须使用`commands.trigger()`实现
- 典型场景：伤害触发护盾、吸血、反击等连锁效果
- 规则本质：比全局Message轻量，天然绑定实体，适合构建线性事件链

### 2.3 Observer = 跨领域通信首选机制
- 🟩 跨Feature、跨Domain的事件响应，必须使用`On<T>` Observer实现
- 🟩 Observer支持`run_if()`条件守卫，替代手动if判断
- 🟥 禁止使用`EventWriter<T>` / `EventReader<T>`（废弃模式）
- 典型场景：伤害→护盾→吸血→死亡判定（跨Ability/Effect/Combat领域）

### 2.4 Delayed Commands = 延迟效果首选
- 🟩 一次性延迟效果使用`Delayed<T>`或`FreDelayed<T>`实现
- 🟩 循环延迟使用Observer自注册模式（handler末尾重新trigger）
- 典型场景：DOT Tick、Buff过期、技能冷却
- Timer仅用于基础设施周期性任务（热重载/审计）或可暂停Buff

### 2.5 Message = 跨Feature全局广播（备选）
- 🟩 当Observer不适用时的全局事件备选方案
- 执行方式：`trigger(T)` + `On<T>` Observer（非旧EventWriter/EventReader）

### 2.6 通用原则
- 🟥 禁止将同一模块内的普通逻辑全部事件化，禁止滥用事件/Trigger模拟函数调用
- 🟩 领域事件是唯一业务事实源，日志、回放、UI、成就均为下游消费者
- 🟩 所有领域事件必须纳入白名单管理，禁止为临时副作用随意新增事件

---

## 三、ECS执行模型
### 3.1 数据流原则
- 🟥 绝对禁止模拟面向对象调用（如`player.attack(enemy)`）
- 正确方式：创建命令组件，由对应系统统一处理

### 3.2 组件依赖
- 🟩 组件依赖必须通过`#[require(Component)]`属性声明
- 🟥 绝对禁止手动检查并补全缺失组件

### 3.3 状态变更检测
- 🟩 组件生命周期变化优先使用原生`Added`、`Changed`、`Removed`过滤器
- 🟥 绝对禁止手写bool脏标记检测组件变化
- 业务状态机（如TurnPhase、BattleState）允许使用显式状态组件自行管理流转

### 3.4 Resource规范
- 🟩 Resource只能存储真正的全局唯一状态
- 🟥 绝对禁止将Resource当作全局变量仓库使用
- 🟨 优先使用Singleton Entity Component替代Res&lt;T&gt;（利用Observer/Hook能力）

### 3.5 性能优化
- 🟥 高频逻辑中禁止使用Observer造成事件风暴
- 每帧执行10次以上的逻辑必须直接使用System处理
- 🟩 优先使用`Changed`过滤器减少不必要计算
- 🟨 批量同构查询优先使用`contiguous_iter()`提升缓存命中率

### 3.6 Reflect 全覆盖
- 🟩 所有Component/Event/Resource类型必须 derive Reflect
- 格式：`#[derive(Component, Reflect)]` + `#[reflect(Component)]`
- 🟥 禁止新增无Reflect的Component/Event/Resource类型

### 3.7 BSN 使用范围
- 🟩 BSN用于描述实体结构（组件组合），System负责行为
- 🟩 UI层（app/scenes/）默认使用`bsn! {}`
- 🟩 核心玩法层使用`spawn_*()`工厂函数（内部自由选择BSN或传统spawn）
- 🟥 禁止在BSN中描述业务逻辑或引用System/Observer
- 🟥 禁止Core层直接import `bsn!`宏

---

## 四、状态与调度规范
### 4.1 状态管理
- 🟩 全局流程状态优先使用`States`、`SubStates`、`ComputedStates`实现
- 禁止手动维护全局状态枚举与切换逻辑

### 4.2 运行条件
- 🟨 系统执行前置判断优先使用`run_if()`条件表达
- 避免在系统内部写大段if判断是否执行

### 4.3 Schedule权责划分
- 🟩 `PreUpdate`：输入处理、命令执行、状态同步
- 🟩 `Update`：核心业务逻辑、规则结算
- 🟩 `PostUpdate`：事件响应、表现层更新、UI刷新
- 🟥 禁止跨Schedule乱放系统，禁止在错误阶段执行业务逻辑

### 4.4 生命周期
- 🟩 `OnEnter`/`OnExit`系统必须保持轻量
- 🟩 重型初始化必须拆分成多个加载阶段
- 🟥 状态切换时禁止隐藏副作用

---

## 五、Bevy专项工程检查
- 🟩 **Commands滥用监控**：禁止每帧高频生成/销毁大量实体
- 🟩 **事件洪泛监控**：禁止单帧触发大量无意义事件
- 🟩 **递归防护**：事件链必须设置递归深度限制，防止循环触发
- 🟩 **Archetype监控**：关注组件组合导致的Archetype数量膨胀，禁止频繁insert/remove造成抖动
- 🟥 绝对禁止凭直觉做性能优化，所有优化必须基于Profile数据
- 🟩 正确性优先于性能，可读性优先于性能，热点代码才优化

---

## 核心速查（10条）
1. Entity只是ID，数据行为严格分离
2. Domain层禁止裸Entity/u64，只用强类型ID
3. 实体状态用Tag，配置临时值可用bool
4. 四级通信：Hook生命周期、Trigger事件链、Observer局部响应、Message跨域广播
5. 组件依赖用`#[require]`，禁止手动补全
6. 组件变化用原生过滤器，禁止手写脏标记
7. 状态机用官方States体系，不手动实现
8. 三级Schedule权责清晰，不跨阶段乱放系统
9. 高频逻辑不用Observer，优先System处理
10. 事件链设防递归深度，避免循环触发

---

