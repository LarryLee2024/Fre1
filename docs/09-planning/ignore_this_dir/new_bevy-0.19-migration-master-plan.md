# Bevy 0.19 激进重构总纲 v3.0

> 本计划基于项目实际代码扫描数据制定。策略：**激进重构，全面采用 0.19 新特性，不计代价。**
> 迁移知识库：`docs/03-technical/bevy-0.19-migration/`

## 1. 项目现状快照

### 1.1 代码规模

- 总文件：500+ .rs
- Bevy API 依赖分布：
  - Observer/Trigger：218行/91文件（核心通信模式）
  - Res/ResMut：176行/42文件（资源访问）
  - Plugin：478行/85文件（模块骨架）
  - Component：178行/59文件
  - Query：99行/37文件
  - Timer：11行/3文件（极少）
  - EventReader：0行（未使用）
  - Bundle：0行（未使用）
  - UI（TextFont/Node）：0行（未使用）
  - commands.spawn：36行/13文件

### 1.2 架构特征

- 完全基于 Observer 模式进行事件通信（非传统 EventReader/EventWriter）
- 15 Capabilities + 15 Domains + Infra 层
- 每个 Plugin 都通过 `.add_observer()` 注册事件处理
- Effect/Modifier 管线是核心红线

### 1.3 迁移风险评估

| 风险域 | 影响面 | 严重度 | 激进策略 |
|--------|--------|--------|---------|
| Observer API 变更 | 91文件/218行 | 🔴 高 | 全量迁移 + run_if 重构 |
| Plugin trait 变更 | 85文件/478行 | 🔴 高 | 全量重写 Plugin 注册 |
| Res/ResMut API 变更 | 42文件/176行 | 🟡 中 | 评估 Resource Hook 化 |
| Component derive 变更 | 59文件/178行 | 🟡 中 | 全量补 Reflect + SceneComponent |
| commands.spawn | 13文件/36行 | 🟡 中 | 全量迁移到 spawn_scene + bsn! |
| bevy-inspector-egui | 1处 | 🟡 中 | 直接删除，用 DiagnosticsOverlay 替代 |
| Timer | 3文件/11行 | 🟢 低 | 全量迁移到 Delayed Commands |

## 2. 迁移策略：两阶段激进重构

### Phase 1：全面升级 + 全特性采用（目标：一步到位）

不是"先兼容再升级"，而是**直接升级到 0.19 并同时采用所有新特性**：

1. **Cargo.toml 升级** → bevy 0.19
2. **编译修复** → 不只是修编译错误，同时重构
3. **Observer 全量 run_if 化** → 91 个文件一次性迁移
4. **Delayed Commands 全面替代 Timer** → 所有延迟逻辑重写
5. **spawn → spawn_scene + bsn!** → 所有实体生成重写
6. **Resource Hook 化** → 关键 Resource 添加 Observer/Hook
7. **Reflect 全量补齐** → 所有资产/配置类型补 Reflect
8. **DiagnosticsOverlay** → 替代 bevy-inspector-egui
9. **User Settings** → 立即引入
10. **Contiguous Query** → 批量运算场景立即使用

### Phase 2：深度架构重构（目标：架构对齐 0.19 范式）

Phase 1 完成后，趁热打铁做架构级重构：

1. **Relationship 替代 Entity 字段** → CasterOf/OwnerOf/SummonedBy 等
2. **SceneComponent 化** → 关键实体预制体化
3. **Observer 链路优化** → 消除 Observer 地狱
4. **批处理思维重构** → Attribute/Buff/Effect 系统批量化

## 3. 宪法级约束（仅保留不可违反的）

1. **架构守界**：Effect/Modifier 管线不可绕过，模块边界不可突破
2. **Replay 兼容**：所有重构后 Replay 必须仍然确定性可重放
3. **Save 兼容**：存档格式变更必须有迁移路径

**删除的约束**（激进模式下不再适用）：
- ~~功能等价~~ — 允许行为变更（只要更优）
- ~~渐进升级~~ — 一步到位
- ~~Simplicity First~~ — 追求最佳架构而非最简方案
- ~~Surgical Changes~~ — 允许大范围重构

## 4. 排除清单（仅排除 3D 无关项）

以下 0.19 特性与 2D SRPG 无关，直接排除：

- ❌ Solari（光线追踪）
- ❌ Skinned Mesh Culling（3D 骨骼网格）
- ❌ Partial Bindless（3D 材质优化）
- ❌ Parallax Corrected Cubemaps（3D 环境反射）
- ❌ White Furnace Test（PBR 着色器）

**保留的项**（激进模式不再排除）：
- ✅ Render Graph as Systems — 架构信号：Trait → System 收敛，学习其设计思想
- ✅ Vignette — 战斗受伤特效可用
- ✅ Lens Distortion — 战术技能特效可用
- ✅ Infinite Grid — 调试时地图网格可视化
- ✅ Transform Gizmo — 调试时实体位置可视化
- ✅ Feathers Widget — 未来编辑器 UI 基础

## 5. 全特性采用矩阵

| 0.19 特性 | 采用策略 | 影响范围 | 执行批次 |
|-----------|---------|---------|---------|
| Delayed Commands | **全面替代 Timer** | 所有延迟逻辑 | Phase 1-Batch1 |
| Observer Run Conditions | **全量迁移 if 守卫** | 91 文件 | Phase 1-Batch1 |
| BSN (bsn!) | **全面替代 spawn** | 13 文件/36行 | Phase 1-Batch2 |
| SceneComponent | **关键实体预制体化** | Character/Ability/Buff | Phase 2 |
| Resources as Components | **Hook + Observer 化** | BattleState/TurnState | Phase 1-Batch2 |
| Contiguous Query | **批量运算立即使用** | Attribute/Buff/Effect | Phase 1-Batch3 |
| User Settings | **立即引入** | Audio/Video/Gameplay | Phase 1-Batch2 |
| DiagnosticsOverlay | **替代 inspector-egui** | 开发调试 | Phase 1-Batch1 |
| FontSource/FontSize | **全面采用** | 未来 UI 代码 | Phase 1-Batch3 |
| EditableText | **立即引入** | 角色命名/搜索 | Phase 1-Batch3 |
| Asset Saving | **立即引入** | 地图/技能编辑 | Phase 2 |
| Handle Serialization | **立即采用** | Config 系统 | Phase 1-Batch2 |
| Self-Referential Relationships | **立即采用** | CasterOf/Healing | Phase 2 |
| Text Gizmos | **调试时使用** | 开发调试 | Phase 1-Batch1 |
| Vignette | **战斗特效** | 受伤反馈 | Phase 2 |
| Render Recovery | **长期运行保障** | 生产环境 | Phase 2 |

## 6. 迁移知识库索引

| 文档 | 位置 | 说明 |
|------|------|------|
| 迁移总览 | `docs/03-technical/bevy-0.19-migration/00-migration-overview.md` | 策略与优先级 |
| BSN | `01-bsn-scene-system.md` | 场景系统（Phase 1 全面采用） |
| Observer | `02-observer-enhancements.md` | Observer 增强（Phase 1 全面迁移） |
| Delayed Commands | `03-delayed-commands.md` | 延迟命令（Phase 1 全面替代 Timer） |
| Resources | `04-resources-as-components.md` | Resource 统一（Phase 1 Hook 化） |
| Contiguous Query | `05-contiguous-query.md` | 性能优化（Phase 1 批量场景） |
| User Settings | `06-user-settings.md` | 用户设置（Phase 1 立即引入） |
| Text/UI | `07-text-and-ui.md` | 文本系统（Phase 1 全面采用） |
| 渲染/DevTools | `08-rendering-and-devtools.md` | DevTools + 特效（Phase 1-2） |
| Asset | `09-asset-system.md` | 资产系统（Phase 1 Reflect + Phase 2 Saving） |
| SRPG 架构影响 | `10-srpg-architecture-impact.md` | 架构级决策 |

## 7. 子计划文件

| 文件 | 说明 |
|------|------|
| `new_bevy-0.19-phase1-aggressive.md` | Phase 1 全面升级 + 全特性采用 |
| `new_bevy-0.19-phase2-deep-refactor.md` | Phase 2 深度架构重构 |
| `new_bevy-0.19-module-checklist.md` | 逐模块迁移检查清单 |

## 8. 决策记录

### DR-001：BSN 全面替代 spawn

- **决策**：所有 `commands.spawn()` 迁移到 `commands.spawn_scene(bsn!{...})`
- **理由**：BSN 是 Bevy 未来方向，早迁移早受益
- **风险**：大量文件同时修改
- **缓解**：按模块分批，每批 nextest 验证

### DR-002：Observer 全量 run_if 化

- **决策**：91 个 Observer 文件一次性迁移 if 守卫到 run_if
- **理由**：渐进迁移反而增加维护成本，不如一步到位
- **风险**：行为变更
- **缓解**：提取共享条件函数到 conditions.rs

### DR-003：Timer 全量迁移到 Delayed Commands

- **决策**：所有 Timer 迁移到 Delayed Commands，包括循环 Timer
- **理由**：统一延迟机制，减少概念数量
- **风险**：循环 Timer 需要重新注册 Delayed Command
- **缓解**：封装 DelayedCommandLoop 工具

### DR-004：删除 bevy-inspector-egui

- **决策**：直接删除 bevy-inspector-egui，用 DiagnosticsOverlay + 自定义 Debug 系统替代
- **理由**：第三方依赖是升级阻塞点，0.19 原生工具更好
- **风险**：失去 inspector UI
- **缓解**：DiagnosticsOverlay + Text Gizmo + 自定义 Debug 系统

### DR-005：Relationship 替代 Entity 字段

- **决策**：所有跨实体引用的 Entity 字段迁移为 Relationship
- **理由**：Relationship 是 ECS 原生关系机制，比裸 Entity 字段更安全
- **风险**：大量文件修改 + Save/Replay 兼容性
- **缓解**：Phase 2 执行，先确保 Phase 1 稳定

### DR-006：Reflect 全量补齐

- **决策**：所有资产/配置/组件类型补齐 Reflect derive
- **理由**：Reflect 是 0.19 生态基础能力，缺 Reflect = 缺未来兼容性
- **风险**：大量 derive 宏添加
- **缓解**：机械性修改，风险低

### DR-007：Contiguous Query 立即使用

- **决策**：所有批量运算场景立即使用 contiguous_iter
- **理由**：AVX2 下 3.5x 性能提升，没有理由不用
- **风险**：代码可读性下降
- **缓解**：封装迭代器工具函数
