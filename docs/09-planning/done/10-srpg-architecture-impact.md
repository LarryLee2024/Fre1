# SRPG 架构影响与迁移指南

> 本文档是 Bevy 0.19 迁移知识体系的总结篇，综合前面 00-09 号文档的结论，提炼出对项目架构的长期影响。
>
> 面向读者：架构师、功能开发者、数据架构师
>
> 前置阅读：`00-migration-overview.md` 至 `09-asset-system.md`

---

## 1. 核心结论

**0.19 真正值得关注的不是 BSN，而是 Bevy 官方正在把 ECS 从"组件存储系统"升级为"关系 + 生命周期 + 观察器驱动的数据模型"。这会直接影响未来 0.20~0.22 的架构方向。**

BSN 是最显眼的新特性，但对 SRPG 项目而言，它目前只是 UI 层的便利工具。真正改变游戏规则的是以下三个底层变化：

1. **Resource → Singleton Entity**：Resource 不再是全局变量，而是世界中的特殊实体，可以拥有 Observer、Hook、Relationship
2. **Observer + RunCondition + DelayedCommand**：三者组合直接改善 Ability/Effect/Buff/Turn 四大核心领域的实现质量
3. **Bundle → Scene → BSN**：这是未来两三个版本大概率成为主流的方向，但现在不应全项目 BSN 化

这三个变化共同指向一个趋势：**Bevy 正在让 ECS 从"被动查询"模型走向"主动响应"模型**。这对 SRPG 这种事件密集、状态变化频繁的领域来说，是根本性的架构利好。

---

## 2. 三个最值得关注的架构变化

### 2.1 Resource → Singleton Entity

**思维模型变化**：Resource 不再是全局变量，而是世界中的特殊实体。

在 0.18 中，Resource 是独立于 Entity 之外的存储，无法挂载 Observer、无法建立 Relationship、无法被 Hook 监听。在 0.19 中，Resource 逐步向 Singleton Entity 演化——它仍然是全局唯一的，但获得了实体的能力。

**对 DDD 的影响**：

- `BattleState` / `TurnState` / `InputState` 等 Domain Resource 可以拥有 Observer，实现"状态变化自动触发行为"
- 可以建立 Relationship，比如 `BattleState` → `CurrentTurn` → `ActiveCharacter`
- Hook 可以监听 Resource 的插入/替换/移除，替代手动 `Changed<T>` 检测

**行动**：理解新模型，代码不需要改动。在新增 Resource 时，考虑它是否需要 Observer/Hook/Relationship 能力；如果需要，设计时预留。

**参考**：`04-resources-as-components.md`

### 2.2 Observer + RunCondition + DelayedCommand

这三者组合会直接改善 Ability/Effect/Buff/Turn 四大核心领域的实现质量。

**Observer 越来越重要**，正在取代部分 EventReader 模式：
- 0.18：`EventWriter<CastAbilityEvent>` → `EventReader<CastAbilityEvent>` → 处理逻辑
- 0.19：`trigger(CastAbility)` → `observe(|trigger: On<CastAbility>| { ... })` → 自动分发

Observer 的优势在于：
- 自动注册，无需在 Schedule 中手动添加 System
- 可以绑定到 Entity 上，实现"谁关心谁监听"
- 与 RunCondition 组合，实现条件触发

**DelayedCommand 改变效果生命周期管理**：
- 0.18：Timer + 状态标记 + 清理 System
- 0.19：`commands.entity(e).insert(Delayed::new(2.secs(), RemoveBuff))` → 自动执行

**行动**：立即采用。Observer + RunCondition + DelayedCommand 是 0.19 中投资回报率最高的特性组合。

**参考**：`02-observer-enhancements.md`、`03-delayed-commands.md`

### 2.3 Bundle → Scene → BSN

**这是未来两三个版本大概率成为主流的方向，但现在不要全项目 BSN 化。**

BSN 的本质是声明式实体组合——用代码描述"实体长什么样"，而不是用命令式代码"一步步构建实体"。这与游戏开发中"预制体"的概念天然契合。

但 BSN 目前的问题：
- 0.19 中 BSN 仍处于早期阶段，API 可能变动
- 核心玩法层（Ability/Effect/Buff/Turn）的行为逻辑不适合声明式描述
- AI 代码生成对 BSN 的支持尚不成熟

**行动**：停止大量发明 Bundle，改用 `spawn_hero()` / `hero_scene()` 工厂模式。工厂函数内部可以用 BSN，也可以用传统 spawn，对外接口统一。这样未来迁移到 BSN 时，只需改工厂函数内部实现。

**参考**：`01-bsn-scene-system.md`

---

## 3. 各领域影响评估

### 3.1 Ability 领域

**当前模式**：
```
CastAbilityEvent → ability_system → ApplyDamageEvent → damage_system
```

**未来模式**：
```
CastAbility → Observer → DamageApplied → Observer → DeathTriggered
```

**变化要点**：
- Observer 链替代 Event 链，更接近 GAS（Gameplay Ability System）设计思路
- Ability 的"效果链"可以用 Observer 自动分发，无需手动在 Schedule 中编排
- RunCondition 可以控制"只在战斗中生效"、"只在特定阵营生效"

**建议**：新增 Ability 行为优先用 Observer 注册。现有 Ability 代码不强制迁移，但新代码应采用新模式。

**风险**：Observer 链式触发可能导致"Observer 地狱"——A 触发 B，B 触发 C，C 又触发 A。缓解措施见第 6 节。

### 3.2 Effect 领域

**Delayed Commands 替代大量 Timer 样板代码**。

当前 Effect 实现中，大量代码是 Timer 管理：
```rust
// 0.18 样板代码
fn effect_tick_system(
    mut query: Query<&mut EffectTimer>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for mut timer in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            // 执行效果
        }
    }
}
```

0.19 可以简化为：
```rust
// 0.19 延迟命令
commands.entity(target).insert(Delayed::new(
    2.secs(),
    ApplyDamage { amount: 10 },
));
```

**分层策略**：
- **短生命周期效果**（死亡动画、技能冷却、简单 DOT）：用 Delayed Commands
- **长生命周期/可取消效果**（持续 Buff、可驱散效果）：仍用 Timer + 状态标记

**建议**：死亡动画/技能冷却/简单 DOT 立即迁移到 Delayed Commands。

**参考**：`03-delayed-commands.md`

### 3.3 Buff 领域

**Delayed Commands 改变 Buff 架构**：短生命周期 Buff 不再需要 Timer。

**Observer RunCondition 简化 Buff 触发条件**：
- 0.18：`if battle_state.is_in_battle() && buff.should_trigger()`
- 0.19：Observer 的 RunCondition 自动处理

**Relationship 可以表达 CasterOf/OwnerOf/SummonedBy**：
- 当前：Buff 组件上存储 `caster: Entity`
- 未来：`Relationship<CasterOf>` 让 Bevy 自动维护关系图
- 优势：查询"某个施法者的所有 Buff"变得简单高效

**建议**：新增 Buff 优先用 Delayed Commands + Observer。Relationship 在 0.19 中观察，0.20 再考虑采用。

### 3.4 Turn 领域

**Observer RunCondition**：`TurnStarted` Observer 仅在 `BattleState` 存在时触发。

```rust
// 0.19
app.observe(turn_started_handler)
    .run_if(resource_exists::<BattleState>);
```

**Delayed Commands**：回合间延迟效果（如"下回合开始时恢复 10 HP"）。

**Resource Hook**：`TurnState` 初始化/清理——Hook 可以在 Resource 插入/替换时自动执行，替代 `OnEnter` 状态逻辑。

**建议**：TurnSystem 中的 `if` 守卫逐步迁移到 `run_if`。这是低风险、高可读性的改进。

### 3.5 Character 领域

**Resource Hook**：`CharacterState` 变化监听——当角色状态从 `Alive` 变为 `Dead` 时，Hook 自动触发死亡流程。

**Relationship**：`OwnerOf` / `SummonedBy`——召唤物与召唤者的关系可以由 Bevy 自动维护。

**Contiguous Query**（未来）：批量属性运算——当需要计算"所有友方角色的平均攻击力"时，contiguous_iter 可以显著提升性能。

**建议**：组件设计保持细粒度（Health / Mana / Level / Faction 拆开），这符合 Bevy 的 ECS 哲学，也为未来的 contiguous_iter 优化做准备。

### 3.6 Map 领域

**影响较小**。Map 领域主要是静态数据和空间查询，0.19 的特性对它没有直接影响。

**User Settings**：地图显示设置（如网格线开关、迷雾显示模式）可以用 User Settings 系统管理。

**建议**：暂无特殊迁移需求。地图显示设置可纳入 User Settings 统一管理。

**参考**：`06-user-settings.md`

---

## 4. 架构设计原则更新

### 4.1 新增原则

1. **Observer 只能跨领域通信，领域内部仍然走 System**
   - Observer 的价值在于解耦领域间的依赖
   - 领域内部用 System 调用更直接、更可测试
   - 这条规则防止"Observer 地狱"

2. **短生命周期效果用 Delayed Commands，长生命周期/可取消效果用 Timer**
   - Delayed Commands 不可取消，适合"一定会发生"的效果
   - Timer 可以暂停/取消，适合"可能被打断"的效果
   - 不要一刀切，按生命周期和可取消性选择

3. **新增资产类型必须 derive Reflect（而非仅 TypePath）**
   - 0.19 加强了 Reflect 的地位，Scene 序列化、编辑器支持都依赖 Reflect
   - 仅 TypePath 不够，未来会还债

4. **停止大量发明 Bundle，改用工厂函数**
   - `spawn_hero()` / `spawn_enemy()` 比 `HeroBundle` 更灵活
   - 工厂函数内部可以自由选择 BSN 或传统 spawn
   - 未来迁移时只改工厂函数内部

5. **扩展点设计用 System/Schedule/Observer，不用 Trait 框架**
   - Bevy 的扩展点是 System 和 Schedule，不是 Trait
   - Observer 是新的扩展点机制
   - 不要引入 Bevy 之外的抽象框架

6. **BSN 负责结构，System 负责行为，Domain 负责规则**
   - BSN 描述"实体长什么样"（组件组合）
   - System 描述"实体做什么"（行为逻辑）
   - Domain 描述"规则是什么"（业务约束）
   - 三者各司其职，不要混淆

### 4.2 保持不变的原则

1. **Feature First** — 按业务领域组织代码，不按技术层组织
2. **Capabilities/Domains 双轴** — Capabilities 管机制（ECS 基础设施），Domains 管业务（SRPG 规则）
3. **四级通信** — Hook / Trigger / Observer / Message，各有适用场景
4. **数据驱动** — 玩法规则下沉 `Domain/rules/`，数值配置归 `content/`
5. **测试跟领域走** — 领域内聚四层测试（单元 → 集成 → 场景 → 回放）

### 4.3 需要更新的原则

1. **通信机制**：Observer 地位提升，从"可选"变为"推荐"
   - 旧：Observer 是 EventReader 的替代选项
   - 新：Observer 是跨领域通信的首选机制，EventReader 退居领域内部

2. **Resource 理解**：从"全局变量"变为"Singleton Entity"
   - 旧：Resource 是全局可变状态
   - 新：Resource 是拥有 Observer/Hook/Relationship 能力的特殊实体

3. **Bundle 模式**：从"推荐 Bundle"变为"推荐工厂函数/Scene"
   - 旧：用 Bundle 组织组件组合
   - 新：用工厂函数封装组件组合，内部可用 BSN Scene

---

## 5. 迁移时间线

### 5.1 第一阶段：纯兼容迁移（1-2 周）

目标：项目在 0.19 上编译通过、测试全绿，不引入任何新特性。

- [ ] 更新 `Cargo.toml`：`bevy = "0.19"`
- [ ] 修复所有编译错误（API 更名、签名变更等）
- [ ] 全局搜索替换 `font_size: f32` → `FontSize::Px(f32)`（如有）
- [ ] 确保测试全绿：`cargo nextest run`
- [ ] 不引入任何新特性，保持代码行为一致

### 5.2 第二阶段：引入 S 级特性（2-4 周）

目标：采用 0.19 中投资回报率最高的特性，改善核心领域代码质量。

- [ ] **Delayed Commands**：替代死亡动画/技能冷却/简单 DOT 的 Timer
  - 识别所有"一次性定时效果"，迁移到 `Delayed::new()`
  - 保留可取消效果的 Timer 实现
- [ ] **Observer RunCondition**：迁移 Ability/Buff/Turn 领域的 `if` 守卫
  - `if battle_state.is_in_battle()` → `run_if(resource_exists::<BattleState>)`
  - `if turn_state.is_player_turn()` → `run_if(resource_equals::<TurnState>(TurnState::Player))`
- [ ] **DiagnosticsOverlay**：添加 Debug Feature Flag
  - 仅在 Debug 模式下启用
- [ ] **User Settings**：定义 `AudioSettings` / `VideoSettings` / `GameplaySettings`
  - 用 0.19 的 User Settings 系统管理

### 5.3 第三阶段：UI 层试点 BSN（4-8 周）

目标：在 UI 层验证 BSN 的可行性，不触碰核心玩法层。

- [ ] 新增 UI 代码使用 `bsn!` 写法
- [ ] 旧 UI 代码不重构，保持稳定
- [ ] 评估 BSN 对 AI 代码生成的提升（可读性、一致性）
- [ ] 评估 BSN 对热重载的支持情况

**红线**：核心玩法层（Ability/Effect/Buff/Turn/Character）禁止 BSN。

### 5.4 第四阶段：性能优化（按需）

目标：基于性能分析数据，针对性使用 0.19 性能特性。

- [ ] 性能分析发现热点（使用 `tracy` / `DiagnosticsOverlay`）
- [ ] 热点处使用 `contiguous_iter`（同构组件批量查询）
- [ ] 高频只读数据 `bypass_change_detection`
- [ ] 评估 `Relationship` 对查询性能的影响

### 5.5 第五阶段：编辑器准备（未来）

目标：为未来的关卡编辑器/角色编辑器做准备。

- [ ] `SceneComponent` 评估：是否适合作为编辑器数据格式
- [ ] BSN Asset Loader：是否支持 `.bsn` 文件加载
- [ ] Feathers Widget：是否满足编辑器 UI 需求
- [ ] Transform Gizmo / Infinite Grid：编辑器必备工具

---

## 6. 风险与缓解

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| Observer 地狱（链式触发） | 中 | 高 | 规范：Observer 只跨领域，领域内走 System；单个 Observer 最多触发一个下游 Observer |
| Delayed Command 不可取消 | 高 | 中 | 封装 `DelayedBattleCommand`，内嵌 `BattleId` 校验；战斗结束时批量清理 |
| BSN 过早采用 | 中 | 高 | 红线：核心玩法层禁止 BSN；UI 层试点后再评估 |
| `font_size` 破坏性变更 | 高 | 低 | 全局搜索替换 `f32` → `FontSize::Px`；编译器会报错，不会遗漏 |
| Reflect 补债 | 低 | 中 | 新增资产/配置类型必须 `derive Reflect`；存量逐步补齐 |
| Relationship API 不稳定 | 中 | 中 | 0.19 观察，0.20 再采用；当前继续用 `Entity` 字段表达关系 |
| 工厂函数与 Bundle 混用 | 中 | 低 | 新代码统一用工厂函数；旧 Bundle 不强制迁移，自然淘汰 |
| RunCondition 复杂度膨胀 | 低 | 中 | RunCondition 保持简单谓词，不嵌套超过 2 层 |

---

## 7. 长期架构信号（3-5 年视角）

### 7.1 ECS 正在向"数据批处理"方向发展

`contiguous_iter` 是信号。当前 SRPG 项目中，批量属性运算（如"所有友方角色攻击力 +10%"）需要遍历所有匹配实体。未来 `contiguous_iter` 可以让同构组件在内存中连续排列，大幅提升缓存命中率。

**启示**：组件设计保持细粒度、同构化。不要把 `Health` + `Mana` + `Level` 合并成 `CharacterStats`，这会阻碍批处理优化。

### 7.2 Observer 正在成为一级公民

事件驱动越来越强。从 0.14 引入 Observer，到 0.19 加强 RunCondition，Bevy 正在把"事件响应"从"手动编排"变为"自动分发"。

**启示**：未来 Ability/Effect/Buff 的 `Entity` 字段逐步迁移为 `Relationship`。`caster: Entity` → `Relationship<CasterOf>`，让 Bevy 自动维护级联删除和关系查询。

### 7.3 Bevy 正在朝"编辑器平台"演化

BSN + Feathers + Gizmo + Settings，这些特性都在为 Bevy Editor 铺路。

**启示**：未来做编辑器时尽量等待官方方案，不要自己造轮子。当前阶段专注核心玩法，编辑器相关特性只做技术预研。

### 7.4 生命周期成为正式设计维度

Spawn → Observe → Cleanup 模式正在成为 Bevy 的惯用模式。`Delayed Commands` 是"延迟 Cleanup"，`Observer` 是"自动 Observe"，`Hook` 是"初始化 Spawn"。

**启示**：对 Buff/Effect/Ability 特别重要。每个效果都应该明确回答：何时 Spawn？何时 Observe？何时 Cleanup？这比"何时创建？何时销毁？"更精确。

### 7.5 官方正在削弱全局状态机依赖

不要让 `BattleState` 控制全世界。0.19 的 RunCondition 允许更细粒度的条件控制，不需要一个全局状态机来决定"哪些 System 应该运行"。

**启示**：局部状态化。用 `InBattle` 组件标记"正在战斗的实体"，用 `BattleRoot` 实体聚合战斗相关的所有实体。`BattleState` Resource 退化为"战斗元数据"，不再是"战斗控制器"。

### 7.6 声明式趋势

BSN 本质是声明式——"我要什么"而非"我怎么做"。这个趋势不仅限于 UI，未来可能扩展到更多领域。

**启示**：未来设计 API 时考虑 Builder/Config/Descriptor 风格。比如 `AbilityDescriptor` 描述"能力长什么样"，System 负责执行。这比"能力自己执行自己"更符合 ECS 哲学。

---

## 8. 决策记录模板

对于每个 0.19 新特性的采用/不采用决策，建议记录如下信息。这对长期项目特别重要——半年后 AI 和你自己都会知道为什么当初没选 BSN，什么时候再评估。

```markdown
### [特性名]
- 决策：采用 / 暂不采用 / 直接忽略
- 理由：...
- 重新评估时间：...
- 依赖条件：...
```

### 已有决策记录

#### Delayed Commands
- 决策：采用
- 理由：直接替代 Timer 样板代码，投资回报率最高
- 重新评估时间：0.20 发布时
- 依赖条件：无

#### Observer RunCondition
- 决策：采用
- 理由：替代 if 守卫，提升可读性和可维护性
- 重新评估时间：0.20 发布时
- 依赖条件：Observer 地狱风险可控

#### BSN
- 决策：暂不采用（核心玩法层）；试点（UI 层）
- 理由：API 不稳定，核心玩法层行为逻辑不适合声明式描述
- 重新评估时间：0.20 发布时
- 依赖条件：BSN API 稳定 + AI 代码生成支持

#### Relationship
- 决策：暂不采用
- 理由：0.19 中 API 可能变动，当前 Entity 字段方案够用
- 重新评估时间：0.20 发布时
- 依赖条件：API 稳定 + 级联删除行为明确

#### Contiguous Query
- 决策：暂不采用
- 理由：当前项目规模不需要，性能热点未出现
- 重新评估时间：角色数量超过 500 时
- 依赖条件：性能分析发现热点

#### Resource Hook
- 决策：观察
- 理由：理解新模型，代码不改动，新增 Resource 时考虑
- 重新评估时间：0.20 发布时
- 依赖条件：Resource Hook API 稳定

#### DiagnosticsOverlay
- 决策：采用
- 理由：Debug 模式下的性能诊断工具，零运行时开销
- 重新评估时间：0.20 发布时
- 依赖条件：仅 Debug 模式启用

#### User Settings
- 决策：采用
- 理由：统一设置管理，减少样板代码
- 重新评估时间：0.20 发布时
- 依赖条件：设置项数量超过 5 个时启用

#### font_size 变更
- 决策：采用（必须）
- 理由：破坏性变更，不迁移无法编译
- 重新评估时间：N/A
- 依赖条件：无

#### Reflect 补债
- 决策：渐进采用
- 理由：新增类型必须 derive Reflect，存量逐步补齐
- 重新评估时间：每次新增资产类型时
- 依赖条件：无

---

> **文档版本**：v1.0
> **最后更新**：2026-06-18
> **维护者**：@architect
> **关联文档**：`00-migration-overview.md` 至 `09-asset-system.md`
