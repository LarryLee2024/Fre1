# Bevy 0.19 迁移总览与策略

> 本文档为 Bevy 0.19 → 0.19 迁移的最高层导航文件，定义迁移策略、优先级与风险。
> 所有子文档均以本文档为纲，不得违反本文档所定阶段约束。

---

## 1. 迁移背景

### 1.1 为什么要升级

Bevy 0.19 带来了多项对 ECS 架构和开发体验有深远影响的变更。对于我们这个大型 SRPG 项目而言，以下几个方向尤为关键：

- **Delayed Commands**：直接替代项目中大量 Timer 样板代码，对 Ability/Effect/Buff/Animation 系统价值极大
- **Observer Run Conditions**：将散布在各系统中的状态守卫（`if battle_state != Running`）统一为声明式条件
- **Resources as Components**：ECS 模型统一化，Resource 本质上是 Singleton Entity 的 Component
- **BSN（Bevy Scene Notation）**：声明式 UI 与场景定义，为未来编辑器时代铺路
- **User Settings**：用户偏好持久化的标准化方案

### 1.2 项目特点

| 维度 | 描述 |
|------|------|
| 项目类型 | 回合制战棋（SRPG），2D 渲染 |
| 架构风格 | DDD 领域驱动设计 + GAS-Lite 能力系统 + ECS |
| 协作模式 | 重度 AI 协作，7 个专用 Agent |
| 领域插件 | Ability / Effect / Buff / Turn / Character / Faction / Terrain / AI / Animation 等 |
| 编辑器 | 暂不涉及，未来规划 |
| 渲染 | 2D 战棋，非 3D 渲染项目 |

### 1.3 迁移原则

1. **功能等价优先**：迁移后功能必须与 0.19 完全一致，不允许功能退化
2. **渐进式升级**：分三阶段推进，每阶段有明确的准入/准出条件
3. **架构守界**：不因迁移引入违反项目架构规范的代码
4. **AI 可追溯**：所有迁移变更必须可追溯到本文档的决策依据

---

## 2. 迁移策略：三阶段

### 第一阶段：纯兼容迁移

> **目标**：0.19 → 0.19，功能完全不变
> **原则**：最小变更集，仅做 API 兼容修复

#### 步骤

```
1. 修改 Cargo.toml 中 bevy 依赖版本 → 0.19
2. cargo check — 识别所有编译错误
3. cargo fix — 自动修复可自动修复的 API 变更
4. 手动修复剩余编译错误（仅 API 兼容修复）
5. cargo clippy — 消除所有警告
6. cargo nextest run — 确保所有测试通过
7. 手动冒烟测试 — 确保游戏功能正常
```

#### 准入条件

- 当前 0.19 版本所有测试通过
- 无未合并的功能分支

#### 准出条件

- `cargo check` 零错误
- `cargo clippy` 零警告
- `cargo nextest run` 全部通过
- 手动冒烟测试通过

#### 红线：禁止事项

| 禁止 | 原因 |
|------|------|
| 引入 BSN | 属于第二/三阶段范畴 |
| 引入 SceneComponent | 属于第三阶段范畴 |
| RenderGraph 重构 | 2D 项目无收益，风险高 |
| Feathers UI | 属于第三阶段范畴 |
| 任何功能变更 | 本阶段只做兼容，不做功能 |

#### 允许事项

- API 签名变更的适配（如函数参数顺序调整）
- 废弃 API 替换为新 API
- Trait 方法签名变更的适配
- 类型重命名的适配

---

### 第二阶段：引入高价值新特性

> **目标**：选择性引入对项目架构价值最高的 0.19 新特性
> **原则**：每个特性独立引入，逐个验证

#### 特性清单与引入顺序

| 顺序 | 特性 | 价值点 | 影响范围 |
|------|------|--------|----------|
| 1 | Delayed Commands | 替代 Timer 样板代码 | Ability / Effect / Buff / Animation / Turn |
| 2 | Observer Run Conditions | 简化状态守卫 | 所有 Observer 系统 |
| 3 | Diagnostics Overlay | 开发调试效率 | 开发环境全局 |
| 4 | User Settings | 用户偏好持久化 | Settings 模块 |

#### 每个特性的引入流程

```
1. 阅读对应子文档（docs/03-technical/bevy-0.19-migration/0X-*.md）
2. 评估对现有代码的影响范围
3. 编写迁移计划（记录在 docs/09-planning/）
4. 逐模块引入，每模块引入后立即测试
5. 更新相关领域文档
```

#### 准出条件

- 所有四个特性均已引入并通过测试
- 无功能回归
- 代码审查通过

---

### 第三阶段：逐步探索

> **目标**：在稳定基础上探索更多 0.19 特性
> **原则**：小范围试点，验证后再推广

#### 探索项目

| 项目 | 时机 | 说明 |
|------|------|------|
| UI 层试点 BSN | 第二阶段完成后 | 仅 UI 层，不碰核心玩法层 |
| contiguous_iter 研究 | 遇到性能瓶颈时 | 需要先做 profiling 确认瓶颈 |
| SceneComponent | 编辑器时代 | 等项目进入编辑器开发阶段 |
| BSN Asset | 编辑器时代 | 与 SceneComponent 同期 |
| Transform Gizmo | 编辑器时代 | 编辑器必需工具 |

---

## 3. 特性优先级矩阵

### S 级 — 立即采用（第二阶段引入）

| 特性 | 评分 | 核心理由 |
|------|------|----------|
| Delayed Commands | ⭐⭐⭐⭐⭐ | 替代 Timer 样板代码，对 Ability/Effect/Buff/Animation 价值极大。当前项目中大量使用 Timer 实现延迟效果、Buff 持续时间、动画帧间隔等，Delayed Commands 可以用声明式方式替代这些命令式样板代码，大幅提升可读性和可维护性 |
| Observer Run Conditions | ⭐⭐⭐⭐⭐ | 简化大量 `if battle_state != Running` 守卫。项目中 Observer 广泛用于事件驱动（伤害触发、Buff 应用、回合切换等），Run Conditions 将散布在各系统入口的 if 守卫统一为声明式条件，减少遗漏和冗余 |
| Diagnostics Overlay | ⭐⭐⭐⭐⭐ | 开发调试神器。SRPG 项目状态复杂（战斗状态、回合状态、角色状态、AI 状态等），Diagnostics Overlay 可以实时可视化关键状态，大幅缩短调试时间 |
| User Settings | ⭐⭐⭐⭐⭐ | 用户偏好持久化标准化。当前项目缺少统一的设置持久化方案，User Settings 提供了开箱即用的解决方案，覆盖音量、画质、键位等常见需求 |

### A 级 — 逐步采用（第二/三阶段引入）

| 特性 | 评分 | 核心理由 |
|------|------|----------|
| Resources as Components | ⭐⭐⭐⭐ | 理解新模型：Resource = Singleton Entity 的 Component。这是 ECS 模型的统一化方向，需要理解但不必立即迁移。当前项目的 Resource 使用方式可以保持，新代码优先考虑新模型 |
| BSN（仅 UI） | ⭐⭐⭐⭐ | 声明式 UI 定义，提升 UI 代码可读性。**严格限制**：仅用于 UI 层，不碰核心玩法层。核心玩法层继续使用命令式 ECS |

### B 级 — 未来再说（第三阶段或更晚）

| 特性 | 评分 | 核心理由 |
|------|------|----------|
| Contiguous Query | ⭐⭐⭐ | 连续内存迭代优化，对大规模实体查询有性能收益。但需要先做 profiling 确认瓶颈所在，不要过早优化 |
| SceneComponent | ⭐⭐⭐ | 场景即组件，编辑器时代的核心特性。当前项目无编辑器需求，暂不引入 |

### C 级 — 直接忽略（与本项目无关）

| 特性 | 忽略理由 |
|------|----------|
| Solari | 3D 渲染相关，2D 战棋项目不需要 |
| Render Graph as Systems | 2D 项目无自定义渲染管线需求 |
| Skinned Mesh | 3D 骨骼动画，2D 战棋不需要 |
| Lens Distortion | 3D 后处理效果，2D 战棋不需要 |
| Vignette | 3D 后处理效果，2D 战棋不需要 |
| Infinite Grid | 3D 编辑器辅助，2D 战棋不需要 |
| Transform Gizmo | 编辑器工具，当前阶段不需要 |
| Parallax Cubemap | 3D 环境映射，2D 战棋不需要 |

---

## 4. 迁移风险评估

### 4.1 风险评分

| 维度 | 评分 | 说明 |
|------|------|------|
| 架构价值 | 9/10 | Delayed Commands + Observer Run Conditions 直接提升核心系统架构质量 |
| 实际收益 | 8/10 | Timer 样板消除、状态守卫简化、调试效率提升，均为高确定性收益 |
| 迁移风险 | 4/10 | 第一阶段纯兼容迁移风险可控；第二阶段逐特性引入，每特性独立验证 |

### 4.2 主要风险点

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| API 破坏性变更超出预期 | 中 | 高 | 第一阶段先 `cargo check` 全面评估，再决定是否继续 |
| Delayed Commands 与现有 Timer 系统冲突 | 低 | 中 | 逐模块替换，保留 Timer 作为降级方案 |
| Observer Run Conditions 改变事件分发行为 | 低 | 高 | 引入前写回归测试，确保事件分发行为不变 |
| BSN 引入导致 UI 层架构漂移 | 中 | 中 | 严格限制 BSN 仅用于 UI 层，核心玩法层禁止使用 |
| AI 协作上下文丢失 | 中 | 中 | 迁移文档必须详实，所有变更可追溯到本文档 |

### 4.3 结论

**建议立即升级，但不要全面 BSN 化。**

- 第一阶段纯兼容迁移风险低、收益确定，应尽快完成
- 第二阶段四个 S 级特性均为高价值、低风险，应优先引入
- BSN 和 SceneComponent 等编辑器相关特性留到第三阶段或更晚

---

## 5. 三个隐藏架构信号

### 信号 1：ECS 正在向"数据批处理"方向发展

`contiguous_iter` 和 Contiguous Query 的引入表明 Bevy 正在强化数据局部性（Data Locality）优化。这意味着：

- **未来 ECS 查询会更关注内存布局**：Archetype 的内存连续性将成为性能关键
- **Component 设计需要考虑布局亲和性**：经常一起查询的 Component 应放在同一 Archetype
- **对 SRPG 项目的启示**：当战场上同时存在数百个单位时，Contiguous Query 可能带来显著性能提升，但需要先做 profiling 确认

### 信号 2：Observer 正在成为一级公民

Observer Run Conditions + Observer 增强 表明 Bevy 正在将事件驱动从"辅助模式"提升为"一等公民"。这意味着：

- **事件驱动架构将越来越主流**：Observer 不仅是事件监听器，而是带有条件守卫、优先级、生命周期管理的完整事件处理单元
- **对 SRPG 项目的启示**：项目中的 Ability 触发、Buff 应用、伤害计算等事件驱动逻辑，应逐步从 System 轮询迁移到 Observer 声明式模式
- **架构影响**：GAS-Lite 能力系统中的 Effect 触发链路，可以用 Observer Run Conditions 重构为更声明式的风格

### 信号 3：Bevy 正在朝"编辑器平台"演化

BSN + Feathers + Gizmo + Settings 的组合表明 Bevy 的长期目标是成为一个编辑器驱动的游戏开发平台。这意味着：

- **场景定义将走向声明式**：BSN 让场景可以用代码定义，SceneComponent 让场景可以作为组件嵌入
- **用户设置将成为基础设施**：User Settings 不是简单的 KV 存储，而是与 ECS 深度集成的偏好系统
- **对 SRPG 项目的启示**：当前阶段不需要编辑器，但架构设计应预留编辑器集成点。特别是：
  - 关卡数据应保持 Asset 化，便于未来编辑器读写
  - 角色配置应保持 Definition 态与 Runtime 态分离，便于未来编辑器编辑
  - UI 布局可以提前用 BSN 试点，为未来编辑器 UI 编辑铺路

---

## 6. 文档索引

本目录下所有迁移文档的链接与简要说明：

| 文档 | 说明 |
|------|------|
| [01-bsn-scene-system.md](./01-bsn-scene-system.md) | BSN 场景系统 — Bevy Scene Notation 的语法、语义、与现有场景系统的关系，以及 SRPG 项目中的适用范围评估 |
| [02-observer-enhancements.md](./02-observer-enhancements.md) | Observer 增强与关系系统 — Observer Run Conditions、Observer 生命周期管理、与 GAS-Lite 事件链路的集成方案 |
| [03-delayed-commands.md](./03-delayed-commands.md) | 延迟命令 — Delayed Commands 的 API、与 Timer 的对比、在 Ability/Effect/Buff/Animation 中的替代方案 |
| [04-resources-as-components.md](./04-resources-as-components.md) | Resource 统一为 Component — Resources as Components 的设计哲学、迁移策略、对现有 Resource 使用模式的影响 |
| [05-contiguous-query.md](./05-contiguous-query.md) | 连续查询与性能优化 — Contiguous Query 的原理、适用场景、SRPG 大规模单位查询的性能评估 |
| [06-user-settings.md](./06-user-settings.md) | 用户设置系统 — User Settings 的 API、与现有设置模块的集成、偏好持久化方案 |
| [07-text-and-ui.md](./07-text-and-ui.md) | 文本系统增强与 Feathers UI — Text 系统变更、Feathers UI 框架、BSN 在 UI 层的试点方案 |
| [08-rendering-and-devtools.md](./08-rendering-and-devtools.md) | 渲染变更与开发工具 — 2D 渲染管线变更、Diagnostics Overlay、开发调试工具集成 |
| [09-asset-system.md](./09-asset-system.md) | 资产系统增强 — Asset 系统变更、对 Definition 态资产的影响、资产加载与热重载 |
| [10-srpg-architecture-impact.md](./10-srpg-architecture-impact.md) | SRPG 架构影响与迁移指南 — 综合评估 0.19 对 SRPG 项目架构的影响，提供模块级迁移检查清单 |

---

> **维护说明**：本文档由 @architect 维护，任何阶段策略变更需经 @architect 审查。
> **最后更新**：2026-06-18
