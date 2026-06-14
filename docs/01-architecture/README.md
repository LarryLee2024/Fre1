---
id: 01-architecture.README
title: Architecture
status: stable
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
---

# Architecture

Version: 4.0

本文件优先级高于任何代码实现。

当代码与本文件冲突时：
优先认为代码错误，而不是文档错误。

---

# Architecture Folder Index

> 本文件是架构规范的最高优先级文档。`docs/01-architecture/` 目录下 36 个子文件提供各领域的详细设计，以下索引列出所有子文件及其核心内容摘要。

| 文件 | 核心内容 | 状态 |
|------|---------|------|
| `app-bootstrap.md` | App 层装配器设计：AppState 状态机、启动/关闭序列、SystemSet 排序、Schedule 组织、EffectPipelineSchedule、BattleEntity 标记清理 | 提议 |
| `asset-organization.md` | 美术资产三树分离（assets/content/src）、Content Packs 组织、角色/地图/音频规范、外包工作流（Git LFS + 自动校验） | 提议 |
| `asset_lifecycle_rules.md` | Handle 类型选择（Strong/Weak）、分阶段卸载（每帧≤4MB）、SafeAssetRef 封装、内存预算（战斗场景 256MB）、延迟卸载队列、Fallback 资源 | 提议 |
| `asset_namespace_design.md` | `base:category/name` 命名空间前缀、MOD 隔离、结构化 AssetId + AssetKey(u64) 预计算哈希、Resolution Chain 解析优先级链 | 提议 |
| `battle_fsm_design.md` | 战斗 FSM（PreBattle→RoundStart→PlayerPhase→EnemyPhase→TurnEnd→VictoryCheck→RoundEnd→PostBattle）、Guard/Action/Effect 三段式、GuardContext 预计算、回合内调度时序（Buff Tick→Trigger→ExecutionStack→Action→Settlement） | 提议 |
| `collaboration-model.md` | AI 6-Agent 协作流程、Handoff 协议、外包美术团队分支策略、CI 分级（PR/Merge/Nightly）、反仓库膨胀策略 | 提议 |
| `command_bus_design.md` | GameCommand 可序列化枚举（禁止 Box<dyn Command>）、Memento 模式撤销、Cursor-based 执行、ActionQueue 效果链式执行、校验/执行分离 | 提议 |
| `component_design_rules.md` | 四位一体组件分类（Marker/Data/Status/Definition）、Hook 跨组件安全、变更检测防御性编程、8 字段拆分阈值、序列化 version 字段 | 提议 |
| `config_system_design.md` | 四层配置（Engine/GameRules/UserSettings/DebugSwitch）、反上帝配置拆分、防抖写入、战斗锁（Battle Lock）、MOD Patch 语义深度合并 | 提议 |
| `content-pipeline.md` | Rule/Content 分离、RON→XxxDef→XxxData→Registry 数据流、异步 AssetLoader+AssetEvent 响应式管线、LoadingProgress 屏障、统一 Registry 模式 | 提议 |
| `content_data_format.md` | RON 配置契约（id/version/tags 必填）、命名空间强制 ID 格式、两阶段加载（Raw→Baked）、CurveTable 数组值映射、字符串 ID 驻留 | 提议 |
| `content_migration_design.md` | 内容格式迁移链（V1→V2→V3）、禁止版本子目录（扁平化）、强类型迁移函数、AssetLoader 内执行迁移、Core 失败阻止启动/MOD 失败降级 | 提议 |
| `determinism_rules.md` | ChaCha8Rng 确定性 PRNG、预生成随机流 Component、整数精度（禁止 f32）、LogicalId 确定性排序、状态哈希（TurnEnd 计算）、浮点污染隔离墙 | 提议 |
| `error-architecture.md` | 三层错误模型（Domain/Infrastructure/Shared）、失败分类学（RuleFailure/DomainError/InfrastructureError/Bug）、GameErrorEvent 系统、Port/Adapter 隔离 | 提议 |
| `events_audit_design.md` | 独立 Struct 事件 + Auditable Trait、EventOrd 同 Tick 排序、EventWhitelist 白名单（TypeId O(1)）、双轨制日志（Command vs Audit）、流式写入 | 提议 |
| `feature_flag_design.md` | 7 个 Feature Flag（replay/debug_ui/cheat/modding/network/telemetry/profiler）、PluginGroup 模式、Core 层零 cfg、CI 全组合测试 | 提议 |
| `i18n_design.md` | Fluent (.ftl) 国际化、永久唯一 ID Key（禁止语义化名称）、LocalizedText 组件、字体回退链、MOD 本地化隔离、性能缓存策略 | 提议 |
| `ids_design.md` | Strong ID newtype（禁止裸 Entity/String 跨模块）、define_id! 宏、Display 含类型前缀、u64 哈希缓存、ID 生命周期不可变不复用 | 提议 |
| `infrastructure-design.md` | 20 个 Infrastructure 模块（P0-P3 优先级）、模块依赖图、可替换性原则、存档 JSON→二进制预留、热重载边界、跨平台适配 | 提议 |
| `layer-contracts.md` | 七层边界定义、三问判断法、Core 禁止依赖外部层、Shared 零依赖、三个垃圾桶警告、层间依赖自动化检测、架构例外审批流程 | 提议 |
| `logging_design.md` | 日志是领域事件消费者（非主动调用）、五级日志体系、LogObserver 统一输出、结构化字段、Feature Gate 编译控制、遥测+数据脱敏 | 提议 |
| `migration-roadmap.md` | 7 Phase 迁移计划（Shared→Core→Infra→Content→App→Modding→Tools）、Git Tag 回滚、技术债务清理窗口期、Phase 4 功能验证清单 | 提议 |
| `modding-design.md` | MOD API 稳定接口、MOD 生命周期（发现→解析→加载→校验→合并）、分级权限策略（Official/Community/Test）、沙箱约束、WASM 预留接口 | 提议 |
| `pathfinding_design.md` | PathFinder trait 抽象（BFS/A*）、RangeCalculator 统一接口、UnitBlocker 阻挡规则、缓存软失效策略、大地图分帧计算、并行计算支持 | 提议 |
| `performance_budget.md` | 60fps 帧预算（16.67ms）、模块级性能预算、变更检测优化（Changed 过滤器）、Reflect 限制、缓存规范（必须定义失效条件） | 提议 |
| `plugin-design.md` | Plugin 按七层组织、单责任原则、Plugin 声明式注册（build() 不执行业务逻辑）、命名规范、错误处理约定 | 提议 |
| `plugin_contract_rules.md` | 显式依赖声明（禁止隐式依赖）、公共 API 最小化、分层禁令、Plugin 初始化顺序、错误处理契约 | 提议 |
| `project-structure.md` | 三棵树分离（src/content/assets）、完整源码树+资产树+内容树、MOD 目录结构、外包目录权限、配置模板规范 | 提议 |
| `save_migration_rules.md` | 存档 SemVer 版本号、向前兼容（新增字段=默认值）、三步删除原则、只保存 Instance 不保存 Definition、纯函数迁移 | 提议 |
| `schedules_design.md` | 自定义 Schedule（Input/Logic/Presentation）、SystemSet 层级排序、状态门控调度、并行 vs 顺序策略、`.chain()` 性能陷阱 | 提议 |
| `skill-buff-abstraction.md` | Effect Executor 统一抽象（500 技能收敛为 20-30 个 Executor）、Skill/Buff/Effect 三层数据模型、ExecutionStack（LIFO 响应栈）、TriggerRegistry 统一分发、SRPG-GAS 对齐、Effect 一级领域化 | 提议 |
| `system_design_rules.md` | 参数上限 8 个、读写分离（Query 参数只读 vs 可变）、禁止 System 间直接调用、命名规范、Exclusive System 使用场景 | 提议 |
| `testing_architecture.md` | 五层测试金字塔（单元→领域集成→系统集成→E2E→Testbeds）、Battle Replay 测试、failure_snapshots、TestCharacterBuilder | 提议 |
| `tools_architecture.md` | Tools 层独立二进制（永不发布）、data_validator/content_linter/balance_checker、headless CI 模式、按需实现原则 | 提议 |
| `ui_domain_boundary_rules.md` | 单向数据流（CQRS）、UI 只读 ViewModel + 只输出 UiCommand、CommandHandler 写入收口、ViewModel 更新机制、Core 层无 UI 知觉 | 提议 |
| `validation_rules.md` | 校验检查点（回合结束/战斗结束/状态转换）、全局不变量（HP≥0/Buff 有来源/Modifier 有 Source）、违规处理分级策略 | 提议 |

---

# 七层架构（v4 新增）

> 来源：`docs/其他/30.md` 架构提炼、最佳实践综合

项目源码按七层组织。每层有严格定义的职责边界和依赖规则。

- App 是唯一允许全局视野的层，禁止包含业务逻辑、禁止直接创建 Entity
- Core 只依赖 Shared（叶子节点），禁止依赖任何其他层
- Shared 零外部依赖，禁止包含任何业务逻辑
- Infrastructure 可替换实现，通过共享事件与 Core 交互
- Content 只做"加载→校验→注册"，禁止包含游戏规则
- Modding 通过稳定 API 扩展，禁止绕过 Effect Pipeline
- Tools 永不进入发布构建
- 层间依赖必须自动化检测（dependcheck.rs + CI 门禁）

详见 `docs/01-architecture/layer-contracts.md`。

## 七层总览

```
Layer 1: App          组装整个游戏           → 只注册，不含逻辑
Layer 2: Core         游戏规则（纯领域逻辑）    → 只依赖 Shared
Layer 3: Shared       基础能力（通用工具）      → 零外部依赖
Layer 4: Infra        技术实现               → 依赖 Core + Shared
Layer 5: Content      内容桥接（配置 → 规则）  → 依赖 Core + Infra + Shared
Layer 6: Modding      MOD 支持               → 依赖 Core + Shared + Infra + Content
Layer 7: Tools        开发工具               → 开发期间专用，永不发布

跨层：
  UI   → 只读 ViewModel，只输出 UiCommand
  Debug → 只读业务数据
```

## 三问判断法

每个文件/模块归属于哪层，用三个问题判断：

**Core 问题**：如果明天把 Bevy 删了，换成 Godot/Unity/UE/服务器模拟器，这个逻辑还存在吗？
→ 存在 → `core/`

**Infrastructure 问题**：如果游戏规则不变，能不能换一种实现方式？
→ 能 → `infrastructure/`

**Shared 问题**：这个东西既不是游戏规则，也不是技术实现，而是所有模块都会用到的基础工具吗？
→ 是 → `shared/`

**一句话总结**：Core = 为什么（业务规则），Infrastructure = 怎么做（技术实现），Shared = 通用工具（基础能力）。

## 层间依赖规则

```
App      → 任意层           ✅（仅注册，不含逻辑）
Core     → Shared           ✅（唯一允许的外部依赖）
Shared   → 无               ✅（叶子节点，零外部依赖）
Infra    → Core, Shared     ✅
Content  → Core, Infra, Shared  ✅
UI       → ViewModel only   ✅
Debug    → Core（只读）      ✅
Modding  → Core, Shared, Infra, Content  ✅
Tools    → Core, Shared      ✅

严格禁止：
Core → Infra              🟥
Core → Content            🟥
Core → UI                 🟥
Core → Modding            🟥
Shared → Core             🟥
Shared → Infra            🟥
Shared → UI                🟥
```

## Content 层核心区分

> Skill 是 Core，Fireball 是 Content。

- **Core** 回答"怎么做"：技能规则引擎怎么跑、Buff 怎么结算、装备怎么穿脱
- **Content** 回答"是什么"：火球术数值、剧毒 Buff 持续回合、铁剑属性加成

🟥 **新增内容 = 新增 RON 文件，绝对禁止修改 Rust 代码。**

## 错误架构

错误分三层：

1. **领域错误** → 放领域内部（`core/skill/domain/skill_error.rs`）
2. **基础设施错误** → 放基础设施内部（`infrastructure/persistence/save/save_error.rs`）
3. **共享错误工具** → 放 `shared/error/`（`GameResult<T>`、错误转换 trait）

🟥 **绝对禁止**：全局统一 `AppError` 大枚举、`anyhow::Error`、`Box<dyn Error>` 作为业务层返回类型。

- 领域错误（SkillError/BattleError 等）放在各 `core/xxx/domain/` 内部
- 基础设施错误（SaveError/AssetError 等）放在各 `infrastructure/xxx/` 内部
- 共享错误工具（ErrorContext/LogIfError）放在 `shared/error/`
- 🟥 禁止全局 `AppError` 大枚举、`anyhow::Error`、`Box<dyn Error>`
- 失败分四类：RuleFailure（业务正常不满足）、DomainError、InfrastructureError、Bug
- GameErrorEvent 事件通道统一上报，禁止 System 内部吞没错误

详见 `docs/01-architecture/error-architecture.md`。

## 项目根目录三级分离

```
project/
├── src/       → Rust 源码（游戏逻辑）
├── assets/    → 运行时资源（美术音频）
├── content/   → 游戏数据（RON 配置）     ← 关键新增
├── mods/      → MOD 扩展
├── tools/     → 开发工具
├── scripts/   → 自动化脚本
├── tests/     → 集成测试
├── benchmarks/ → 性能基准
├── docs/      → 文档
└── build/     → 构建输出
```

🟥 **绝对禁止**：将配置数据、美术资源、开发脚本混入同一目录。

- 三棵树分离：`src/`（Rust 源码）、`content/`（RON 配置）、`assets/`（美术音频）
- 源码树按七层组织：app/ → core/ → shared/ → infrastructure/ → content/ → modding/ → debug/
- Core 内部按业务领域拆分（battle/skill/buff/character 等），禁止 components.rs/utils.rs 巨文件
- 内容树（content/）完整结构：skills/buffs/characters/equipments/items/stages/terrains/ai_behaviors/formulas/curves/
- 资产树（assets/）按类型组织：art/（角色/地图/UI/粒子/着色器）、audio/（sfx/bgm）、fonts/

详见 `docs/01-architecture/project-structure.md`。

## 迁移路线

当前项目需要从扁平结构迁移到七层架构。- 7 Phase 迁移路径：准备 → Shared → Core 重组 → Infra 扩展 → Content 层 → App 层 → Modding → Tools
- 每阶段必须保持编译通过 + 测试通过 + Git Tag 回滚点
- Phase 1（Shared）：迁移强类型 ID + 错误工具到 shared/
- Phase 2（Core）：业务模块从顶层迁入 src/core/，错误迁移到各领域 domain/
- Phase 4（Content）：配置数据从 assets/ 迁移到 content/，影响最大需灰度迁移
- 技术债务清理窗口期：每 Phase 完成后预留 1 天清理 deprecated 重导出

详见 `docs/01-architecture/migration-roadmap.md`。

---

# 强制等级

所有规则分为五个强制等级：

- 🟥 **绝对禁止**：任何情况下都不允许出现，不可豁免
- 🟩 **必须遵守**：无例外强制执行，除非获得明确豁免
- 🟨 **优先选择**：除非有明确且可验证的技术理由，否则必须采用
- 🟦 **最佳实践**：推荐但非强制，无技术理由时优先采用
- ⚠️ **警觉阈值**：达到阈值时必须主动提出重构建议

---

# 宪法豁免机制

违反本文件规则的代码必须标注 `[宪法豁免]` 并说明理由。

豁免代码每 3 个月必须重新评估。

豁免格式：

```
// [宪法豁免] 违反条款：[条款编号]
// 理由：[具体技术理由]
// 有效期：[日期]，下次架构复盘时重新评估
```

---

# 项目目标

项目类型：

SRPG（战棋RPG）

技术栈：

- Rust
- Bevy 0.18+
- ECS
- Data Driven

核心原则（优先级从高到低）：

1. 🟥 Feature First
2. 🟥 Definition / Instance 分离
3. 🟥 Rule / Content 分离
4. 🟥 Logic / Presentation 分离
5. 🟩 Hook = 固有行为，Observer = 局部响应，Message = 跨Feature广播
6. 🟩 Trait + Modifier 统一扩展体系
7. 🟩 数据驱动优于硬编码
8. 🟩 组合优于继承
9. 🟩 只解决当前复杂度
10. 🟩 官方能力优先

---

# Feature 划分

## 七层架构下的目录组织

> 详见 `docs/01-architecture/project-structure.md` 和 `docs/01-architecture/layer-contracts.md`

源码按七层组织：

```
src/
├── app/              # Layer 1: 游戏启动与装配
├── core/             # Layer 2: 游戏规则（纯领域逻辑）
├── shared/           # Layer 3: 基础能力（通用工具）
├── infrastructure/   # Layer 4: 技术实现
├── content/          # Layer 5: 内容桥接（配置 → 规则）
├── modding/          # Layer 6: MOD 支持
├── ui/               # 表现层
└── debug/            # 调试工具
```

## Core 内部模块划分

允许：

```
core/
battle/
character/
buff/
skill/
equipment/
inventory/
map/
turn/
ai/
movement/
```

禁止：

```
components.rs
systems.rs
events.rs
utils.rs
```

作为顶层业务模块。

发现以上文件作为业务模块时：

必须停止。

必须输出：

```
ARCHITECTURE VIOLATION: 发现技术分层模块 [文件名]，违反 Feature First 原则。
```

> **[宪法豁免]** `core/` 路径名不符合宪法 3.0.5 "路径表达业务含义"的要求（宪法禁止 `base/`、`core/` 等纯技术命名）。
> 豁免理由：`core/` 承载属性系统、标签系统、效果管线、修饰规则等跨模块基础设施，属于项目铁律明确规定的模块划分，重命名将破坏与代码的对应关系。
> 有效期：长期，每次架构复盘时重新评估。

---

# 模块边界

## Battle

负责：

- 战斗效果管线（generate → modify → execute）
- 伤害计算与修饰
- 战斗记录（BattleRecord + DamageBreakdown）
- 回合内行动执行

禁止：

- 修改背包
- 修改任务
- 修改UI
- 直接修改角色属性

必须通过：

```
DamageApplied Message
HealApplied Message
CharacterDied Message
```

通知其他模块。

## Character

负责：

- 单位生成（spawn from template）
- 单位组件（Unit, Faction, GridPosition, Dead, UnitName, UnitId）
- Trait 扩展体系（TraitCollection, TraitTrigger, TraitEffect）
- 移动动画

Character 组成：

```
Race + Job + Stats + Equipment + Traits + Buffs
```

允许：

- 通过 Trait + Modifier 组合能力
- 通过 UnitTemplate 生成运行时实例
- 通过 RegistryLoader 加载配置

禁止：

- 直接计算伤害
- 直接操作 Buff
- 🟥 把 Entity 当对象使用
- 直接修改 UnitTemplate
- 🟥 在运行时创建新的 Definition

必须通过：

```
TraitTrigger::OnAttack → battle/pipeline 触发
TraitEffect::ApplyBuff → buff/apply 执行
```

## Race

种族 = Trait 集合 + Modifier 集合

允许：

- 提供 Trait + Modifier 集合

禁止：

- 直接实现逻辑
- 硬编码种族效果

## Job

职业 = 成长率表 + 技能池 + Trait 集合

升级：

修改成长

禁止：

- 修改 Definition
- 硬编码职业效果

## Buff

负责：

- Buff 定义与注册（BuffData, BuffDef, BuffRegistry）
- Buff 实例管理（BuffInstance, ActiveBuffs）
- Buff 穿戴/移除（修改 Attributes + Tags）
- 持续效果结算（DoT/HoT/晕眩/tick）

Buff = 临时 Trait + 临时 Modifier

持续时间 = Turn

必须记录：

- Stack 数
- Source 来源
- Remaining Turns

允许：

- Stack 叠加（有上限）
- Tick 结算（每回合）

禁止：

- 直接扣血
- 直接修改 UI
- 🟥 Buff 永不过期
- 🟥 Buff 无来源

必须：

- 回合结束检查过期
- 过期自动移除 + 重建 Trait
- 通过 DotApplied / HotApplied / StunApplied Message 通知

## Skill

负责：

- 技能定义与注册（SkillData, SkillDef, SkillRegistry）
- 技能槽位与冷却（SkillSlots, SkillCooldowns）
- 技能效果预览

Skill = SkillDef（配置）+ SkillCooldowns（运行时）

允许：

- 数据驱动配置技能
- 技能效果预览

禁止：

- 直接执行伤害
- 直接施加 Buff
- 硬编码技能效果
- 跳过冷却检查

必须：

- 冷却每回合递减
- 技能范围由 effective_skill_range 计算
- 通过 CombatIntent → battle/pipeline 执行
- 通过 EffectDef → EffectHandler 分发

## Equipment

负责：

- 装备定义与注册（EquipmentDef, EquipmentRegistry）
- 装备实例与槽位（EquipmentInstance, EquipmentSlots）
- 穿脱逻辑
- 装备需求检查

装备本质 = Modifier 集合 + Trait 集合

穿脱流程：

```
检查需求 → 穿戴 → 应用效果 → 重建 Trait
```

允许：

- 通过 EquipmentDef 配置装备
- 通过 EquipmentSlot 管理槽位

禁止：

- 直接修改角色基础属性
- 跳过需求检查
- 🟥 跳过 Trait 重建

必须：

- 穿脱后重建 TraitCollection
- 记录装备实例 ID
- 通过 ModifierSource::Equipment → Attribute Modifier Stack
- 通过 TraitSource::Equipment → TraitCollection
- 通过 EquipItem / UnequipItem Message
- 通过 ItemEquipped / ItemUnequipped Message

## Inventory

负责：

- 物品定义与注册（ItemDef, ItemRegistry）
- 物品实例与堆叠（ItemInstance, ItemStack）
- 容器管理（Container）
- 战斗背包（BattleBag）
- 物品转移与使用

Container = Slot + Stack + Weight

允许：

- ItemDef / ItemInstance 分离
- Container 间转移物品

禁止：

- 直接修改角色属性
- 直接施加 Buff
- 直接修改 Stack 数量绕过转移逻辑
- Stack.count = 0 时不清理

必须：

- 转移校验容量
- 空 Stack 自动清理
- ItemDef 引用校验
- 通过 UseItem / TransferItem Message

## Encumbrance

总重量超过限制：

移动力下降

禁止：

- 直接禁止行动

## Map

负责：

- 地形数据（TerrainGrid）
- 单位占位（OccupancyGrid）
- 寻路（BFS + 地形消耗）
- 坐标转换（GameMap）

数据分离：

```
TerrainGrid  → 地形唯一真相源
OccupancyGrid → 单位占用独立存在
GameMap       → 坐标转换
```

允许：

- Tile 按需设为 Entity
- Chunk 按需引入

禁止：

- 直接移动角色
- 直接修改角色属性
- 🟨 Tile 默认作为 Entity
- 在 Unit Component 上存储地形信息
- 🟥 地图逻辑依赖渲染层
- 寻路数据硬编码

必须：

- 🟥 地图数据与渲染分离
- 🟥 寻路数据运行时生成
- OccupancyGrid 与 TerrainGrid 独立更新

## Pathfinding

BFS 寻路

允许：

- TerrainCostCalculator trait 扩展
- 标签解析计算器（SWIMMING > FLYING > MOUNTED > ground）

禁止：

- 硬编码移动成本
- 寻路时直接查询 Entity

必须：

- find_reachable_tiles 返回可达范围
- reconstruct_path 返回路径

## Turn

负责：

- 主状态（AppState）
- 回合阶段（TurnPhase SubState）
- 行动队列（TurnOrder）
- 回合 Message

回合流程：

```
AppState（MainMenu / InGame / GameOver）
↓
TurnPhase（SubState，仅 InGame 激活）
↓
TurnOrder（Initiative 降序行动队列）
```

允许：

- 队列耗尽自动进入 TurnEnd
- ForceEndTurn 强制结束

禁止：

- 执行战斗逻辑
- 修改角色状态
- 🟥 状态机处理业务细节
- 🟥 OnEnter / OnExit 包含重逻辑

必须：

- 回合结束重置所有单位 acted
- NeedsResolve 防止重复结算
- TurnStarted / TurnEnded Message 广播
- 通过 NextState\<TurnPhase\> 驱动阶段转换

## AI

负责：

- AI 行为定义与注册（AiBehavior, AiBehaviorRegistry）
- 策略选择（TargetSelector, MoveSelector, SkillSelector）
- AI 决策系统

AI 决策 = AiBehavior（数据驱动配置）+ AiStrategyRegistry（trait 分发）

允许：

- 新增策略只需实现 trait 并注册
- RON 配置不同行为模式

禁止：

- 🟥 独立执行攻击逻辑
- 🟥 独立计算伤害
- 硬编码 AI 逻辑
- enum + match 分发策略

必须：

- strategy_name 与 RON 配置对应
- 未知策略回退默认
- 🟥 AI 和玩家共用 Effect Pipeline
- 🟥 CombatIntent 是唯一攻击意图通道

发现 AI 模块包含独立伤害计算时：

必须停止。

必须输出：

```
ARCHITECTURE VIOLATION: AI 模块包含独立伤害计算，违反"AI 与玩家共享 Effect Pipeline"原则。
```

## UI

负责：

- 用户输入处理（UiCommand）
- 命令分发（command_handler）
- ViewModel 层
- 面板与组件展示

UI = ViewModel + UiCommand + UiTheme

允许：

- UI 只读 ViewModel
- UI 发出 UiCommand Message
- 主题统一样式

禁止：

- 🟥 保存业务真相
- 🟥 直接操作 ECS 组件修改业务状态
- 🟥 直接查询 ECS World 获取业务数据
- 🟥 业务逻辑直接操作 UI
- 🟥 UI 绕过 ViewModel 直接 Query 游戏组件

必须：

- 🟥 UI → Logic 只走 UiCommand
- 🟥 Logic → UI 只走 ViewModel
- 模态面板标记 BlocksGameInput
- ViewModel 变化时刷新（非每帧轮询）
- 🟩 输入焦点全局统一管理
- 🟩 拖拽功能全局统一实现
- 🟩 复杂 UI 支持调试与可视化检查

发现 UI 代码直接修改 Attributes / ActiveBuffs / EquipmentSlots 时：

必须停止。

必须输出：

```
ARCHITECTURE VIOLATION: UI 直接修改业务状态，违反 Logic/Presentation 分离原则。
```

## Core

负责：

- 属性系统（Attributes, AttributeKind, ModifierSource）
- 标签系统（GameplayTag 位掩码）
- 效果管线（EffectDef, PendingEffect, EffectHandler）
- 修饰规则（ModifierRuleRegistry）
- 注册表加载（RegistryLoader）
- 场景快照（Snapshot）

禁止：

- 🟥 依赖任何业务模块
- 🟥 包含业务逻辑

发现 core 模块 use 了 character/battle/buff 等业务模块时：

必须停止。

必须输出：

```
ARCHITECTURE VIOLATION: core 模块依赖业务模块，违反"核心层无外部依赖"原则。
```

## Debug

负责：

- 调试面板（bevy_egui）
- DebugPanelState 管理
- 可观测性（BattleRecord, DamageBreakdown）

禁止：

- 影响生产逻辑
- 修改业务状态

---

# 模块化规范

## 接口最小化

🟩 模块只暴露必要的公共接口，所有内部实现必须私有。

## 边界优先于目录

🟩 模块边界的清晰度比目录结构更重要。目录能调整，耦合难清理。

## 跨模块通信规范

🟥 跨模块禁止直接访问内部组件或状态。

跨模块只能通过 Message、Observer、Command 三种方式通信。

## Plugin 拆分原则

🟩 Plugin 职责过大时必须拆分，按业务领域拆分而非按代码数量拆分。

## 通用代码规范

🟨 优先不创建通用顶层目录。

如确需创建 `common/`，只能存放与业务无关的纯工具代码。

🟥 禁止在 `common/` 中放入任何业务逻辑。

---

# App 层与启动装配

- App 层是游戏的装配器，唯一职责是组装整个游戏，禁止包含任何业务逻辑
- AppState 状态机：MainMenu → LevelSelect → InGame → GameOver，TurnPhase 为 InGame 的 SubState
- 启动序列：DefaultPlugins → EguiPlugin → AppPlugin（统一入口），按 Shared → Infra → Core → Content → Logic → UI → Debug 顺序注册
- SystemSet 排序：InputSet → CommandSet → LogicSet → EffectSet → ViewModelSet → UISet
- `.chain()` 是 Bevy 并行杀手，禁止使用；效果管线用 EffectPipelineSchedule + run_schedule() 替代
- 关闭时使用 BattleEntity 标记组件 + desppawn_recursive() 统一清理，禁止逐个 despawn

详见 `docs/01-architecture/app-bootstrap.md` 和 `docs/01-architecture/plugin-design.md`。

---

# 插件注册顺序

必须按以下顺序注册：

```
1. 核心层：EffectPlugin, ModifierRulePlugin, AttributeDefPlugin, TagDefPlugin
2. 数据层：SkillPlugin, BuffPlugin, AiBehaviorPlugin, EquipmentPlugin, InventoryPlugin
3. 逻辑层：AssetsPlugin, TurnPlugin, MapPlugin, CharacterPlugin, BattlePlugin, AiPlugin
4. 表现层：UiPlugin, InputPlugin, DebugPlugin
```

禁止：

- 表现层插件在数据层之前注册
- 逻辑层插件在核心层之前注册

发现注册顺序错误时：

必须输出：

```
ARCHITECTURE VIOLATION: 插件注册顺序错误，[插件名] 不应在 [层级] 之前注册。
```

- Plugin 必须显式声明依赖，禁止隐式依赖其他 Plugin 注册的 Resource
- Plugin 公共 API 最小化，禁止暴露内部实现
- Plugin build() 只做声明式注册，不执行业务逻辑
- 每个 Plugin 的错误类型独立，禁止跨 Plugin 共享错误枚举

详见 `docs/01-architecture/plugin_contract_rules.md`。

---

# Definition / Instance 分离

🟥 不可豁免。

Definition：

不可变配置

例如：

```
BuffData / BuffDef
SkillData / SkillDef
EquipmentDef
ItemDef
UnitTemplate
AiBehavior
TerrainDef
```

Instance：

运行时状态

例如：

```
BuffInstance / ActiveBuffs
SkillSlots / SkillCooldowns
EquipmentInstance / EquipmentSlots
ItemInstance / ItemStack
Unit + Attributes + ActiveBuffs
```

禁止：

- 🟥 修改 Definition 中的任何字段
- 在 Instance 中硬编码配置数据

发现 `BuffData.xxx = ...` 或 `SkillData.xxx = ...` 赋值时：

必须停止。

必须输出：

```
ARCHITECTURE VIOLATION: 运行时修改 Definition 数据，违反 Definition/Instance 分离原则。
```

---

# Rule / Content 分离

🟥 不可豁免。

代码：

负责规则

配置：

负责内容

新增职业：

允许：

- 新增 RON 配置文件

禁止：

- 🟥 修改伤害计算代码

新增技能：

允许：

- 新增 RON 配置文件

禁止：

- 🟥 修改 Effect Pipeline 代码

新增装备：

允许：

- 新增 RON 配置文件

禁止：

- 🟥 修改 Modifier 规则代码

发现为了新增内容而修改核心规则代码时：

必须停止。

必须输出：

```
ARCHITECTURE VIOLATION: 新增内容修改了规则代码 [文件名]，违反 Rule/Content 分离原则。
应通过 RON 配置实现，而非修改代码。
```

---

# Logic / Presentation 分离

🟥 不可豁免。

Logic：

- 伤害计算
- Buff 施加与结算
- 属性修饰
- 回合管理
- AI 决策

Presentation：

- 动画
- 音效
- UI 面板
- 战斗飘字
- 调试面板

禁止：

- 🟥 `apply_damage()` 播放动画
- 🟥 `add_buff()` 刷新 UI
- 🟥 `execute_effects()` 播放音效

必须通过：

```
DamageApplied Message → combat_vfx_handler 播放飘字
CharacterDied Message → Observer 播放死亡动画
BuffApplied Message → UI 刷新 Buff 列表
```

发现业务函数包含 UI/动画/音效调用时：

必须停止。

必须输出：

```
ARCHITECTURE VIOLATION: 业务逻辑 [函数名] 包含表现层调用，违反 Logic/Presentation 分离原则。
```

---

# 只解决当前复杂度

🟥 不可豁免。

禁止：

- 🟥 为未来可能出现但未明确的需求提前设计架构
- 🟥 过度抽象增加当前维护成本

规则本质：

预测未来需求的准确率极低，过度设计是最大的技术债务。

---

# 官方能力优先

🟩 必须遵守。

- 🟩 优先使用 Bevy 原生提供的所有功能
- 🟩 优先使用社区成熟插件而非自研
- 🟥 禁止重复实现 Bevy 已有的基础设施

---

# ECS

## Entity

🟥 Entity 仅为 ID。

禁止：

- 🟥 EntityManager OOP 风格封装
- 🟥 在 Entity 上调用任何方法或将其当作面向对象实例使用
- 🟥 把 Entity 当对象使用

允许：

- 将 Entity 作为纯 ID 参数传递

## Component

🟥 Component 只能存储纯数据状态。

禁止：

- 🟥 Component 包含任何逻辑
- Component impl 包含超过 3 个方法
- Component 方法修改其他 Component

## System

🟩 System 只能包含纯逻辑，禁止存储任何状态。

## Hook

🟩 用于组件固有行为

Component 的添加/移除时的固有副作用必须通过 `#[component(on_add=..., on_remove=...)]` 属性声明。

例如：

```
#[component(on_add=remove_moveable)] struct Dead;
```

规则本质：组件的副作用应该与组件定义绑定，而不是分散在各个系统中。

## Observer

🟩 用于同一 Feature 内的局部响应

例如：

```
死亡动画
任务更新
UI 刷新
```

## Message

🟩 用于跨 Feature 广播

例如：

```
DamageApplied
CharacterDied
EquipItem
TurnEnded
UiCommand
```

当前 Message 注册表：

| Message | 发送方 | 接收方 |
|---------|--------|--------|
| UiCommand | input | command_handler |
| DamageApplied | battle/execute | ui/combat_vfx, ui/combat_log, battle/record |
| HealApplied | battle/execute | ui/combat_log, battle/record |
| CharacterDied | battle/execute | battle/events, ui/combat_log, battle/record |
| StunApplied | buff/resolve | ui/combat_log, battle/record |
| DotApplied | buff/resolve | ui/combat_log, battle/record |
| HotApplied | buff/resolve | ui/combat_log, battle/record |
| EquipItem | ui | equipment/equip |
| UnequipItem | ui | equipment/equip |
| ItemEquipped | equipment/equip | ui/combat_log |
| ItemUnequipped | equipment/equip | ui/combat_log |
| UseItem | ui | inventory/use_item |
| TransferItem | ui | inventory/transfer |
| TurnStarted | turn | battle/record |
| TurnEnded | turn | battle/record |
| ForceEndTurn | ui/command_handler | turn |

## Required Components

🟩 组件依赖必须通过 `#[require(Component)]` 属性声明。

禁止：

- 🟥 手动检查并补全缺失的组件

必须通过：

```
#[require(Attributes, ActiveBuffs, SkillSlots, EquipmentSlots)]
struct Unit;
```

## 状态变更检测

🟩 必须优先使用 Bevy 原生的 `Added`、`Changed`、`Removed` 过滤器。

禁止：

- 🟥 手写状态标记字段检测变更

规则本质：Bevy 的变更检测经过高度优化，比手写实现更高效。

## Tag Component

🟩 所有布尔状态必须使用空 Tag Component 实现。

允许：

```
Dead
Frozen
Stunned
Selected
Acted
```

禁止：

```
is_dead: bool
is_frozen: bool
is_stunned: bool
is_selected: bool
has_acted: bool
```

发现 `is_xxx: bool` 字段时：

必须输出：

```
ARCHITECTURE WARNING: 发现 bool 字段 [字段名]，建议使用 Tag Component 替代。
```

规则本质：Tag Component 可以被 Bevy 优化为位掩码，查询性能远超布尔字段。

## Resource

🟩 Resource 只能存储真正的全局唯一状态。

允许：

```
SkillRegistry
BuffRegistry
EquipmentRegistry
TurnOrder
GameMap
TerrainGrid
OccupancyGrid
```

禁止：

- 🟥 将 Resource 当作全局变量仓库使用
- 在 Resource 中存储可变业务状态

## 跨模块通信

🟩 Hook = 固有行为（组件添加/移除时）
🟩 Observer = 局部响应（同一 Feature 内状态变化时）
🟩 Message = 跨 Feature 广播

允许：

- 🟩 模块内部优先函数调用

禁止：

- 🟥 将同一模块内的所有逻辑都事件化
- 🟥 滥用事件系统模拟函数调用
- 🟥 跨模块访问内部细节
- 🟥 高频逻辑走 Observer 风暴

必须：

- 跨模块通过 Message / Observer / Command 通信
- 模块只暴露公共接口

## ECS 执行模型

🟩 ECS 是数据流，不是调用链。

禁止：

- 🟥 模拟面向对象的调用方式，如 `player.attack(enemy)`

正确方式：

创建 `CombatIntent` 组件，由 `AttackSystem` 统一处理。

## 性能优化

🟥 在高频逻辑中禁止使用 Observer 造成风暴。

每帧执行 10 次以上的逻辑必须直接使用 System 处理。

---

# 组件设计规范

- 四位一体组件分类：Marker（零数据标记）、Data（纯数据容器）、Status（临时状态）、Definition（静态配置）
- Marker Component 绝对禁止携带字段，仅用于 With/Without 查询过滤
- Data Component 允许纯读取视图方法（如 is_alive()），禁止修改自身状态的方法
- Status Component 必须在状态退出时自动清理，禁止残留
- Definition Component 命名以 Def/Config 结尾，运行时绝对禁止 &mut 引用
- 8 字段拆分阈值：超过 8 个字段且分属不同关注点时必须拆分
- Hook 中禁止跨组件修改（get_mut），必须通过 Observer/Event 协调
- 变更检测防御性编程：修改前先判断值是否真正改变，避免无效 Changed 触发

详见 `docs/01-architecture/component_design_rules.md`。

# System 编写铁律

- 单系统 Query/Resource 参数上限 8 个，超过必须拆分（编译时间呈指数增长）
- 读写分离：只需读取时用 &T，修改前先判断值是否改变
- 禁止 System 间直接调用（破坏并行），跨模块必须通过 Message/Observer/Command
- 命名规范：动词短语描述意图（如 update_health_bar、calculate_damage）
- Exclusive System 仅用于需要 &mut World 的场景（如 Command Bus），普通 System 禁止使用

详见 `docs/01-architecture/system_design_rules.md`。

---

# Effect Pipeline

战斗效果必须走三步管线：

```
Generate（生成效果）
↓
Modify（修饰规则）
↓
Execute（执行效果）
```

允许：

- ModifierRule 标签匹配修饰
- ModifierEntry 记录每步修饰

禁止：

- 🟥 跳过管线直接执行
- 🟥 跳过 Modify 阶段
- 🟥 在 generate 中直接扣血
- 在 execute 中重新计算修饰

必须：

- 伤害下限 ≥ 1
- 治疗下限 ≥ 0
- 所有修饰记录写入 BattleRecord

扩展点：

- EffectHandler trait：新增效果类型
- ModifierCalculator trait：新增修饰规则
- TraitTrigger 枚举：新增触发时机

新增效果类型时：

允许：

- 实现 EffectHandler trait
- 注册到 EffectHandlerRegistry

禁止：

- 修改管线调度代码
- 修改 generate/modify/execute 的执行顺序

---

# Damage Pipeline

伤害顺序：

```
Hit Check
↓
Dodge / Block
↓
Critical
↓
Shield Absorb
↓
Resistance
↓
ModifierRule
↓
Final Damage
↓
HP Change
```

禁止：

- 🟥 跳过流程
- 🟥 绕过管线直接扣 HP

---

# Death

HP ≤ 0 不等于死亡。

必须：

1. 添加 Dead Tag Component
2. 由 Hook 处理固有行为（移除移动组件等）
3. 由 Observer 处理局部响应（播放动画、刷新 UI）
4. 由 Message 广播跨模块通知（CharacterDied）

禁止：

- 🟥 直接删除 Entity
- 🟥 跳过 Dead Tag 直接处理死亡逻辑
- 在 HP 变化时内联死亡处理

---

# 战斗事件链

🟩 复杂战斗交互必须使用 EntityEvent 机制。

例如：

```
装备护盾 → 护盾吸收伤害 → 角色受伤 → 触发被动技能
```

---

# 属性系统

## Primary / Derived 分离

🟥 不可豁免。

三层架构：

```
Core Stat（8维）
↓
Derived Stat（战斗组 + 辅助组）
↓
Vital Resource（HP / MP / Stamina）
```

Primary Stat：

```
Might, Agility, Vitality, Intelligence, Luck, Resistance, Dexterity, Willpower
```

Derived Stat：

```
MaxHp = 5 + Vitality * 5
MaxMp = Intelligence * 5
Attack = Might * 2
Defense = Vitality
MagicAttack = Intelligence * 2
Initiative = Agility * 2
MoveRange = Agility / 2
```

禁止：

- 🟥 直接修改 Derived Stat
- 🟥 直接修改最终属性值
- 缓存 Derived Stat（必须实时计算）
- Derived Stat 缓存无失效条件
- 到处硬编码属性计算

必须：

- 🟥 所有属性修改走 Modifier 管线（添加/移除 Modifier）
- 🟥 最终属性值必须只有一个统一的计算来源
- 🟩 属性公式集中管理
- 🟩 配置型属性数据必须保持不可变

## Modifier Stack

Modifier = ModifierType（Flat / Percent / Override）+ ModifierSource（Race / Job / Equipment / Buff / Trait）

所有属性修改必须通过 ModifierSource：

```
ModifierSource::Base           # 种族/职业基础值
ModifierSource::Buff(id)       # Buff 来源
ModifierSource::Equipment(slot) # 装备来源
ModifierSource::Trait(name)    # Trait 来源
ModifierSource::Temporary(name) # 临时修饰
```

允许：

- 叠加多个 Modifier
- 按优先级排序

禁止：

- 🟥 直接修改 base 属性值
- 🟥 绕过 Modifier Stack 修改最终属性

必须：

- Modifier 有 Source 标记
- Modifier 有过期条件

发现 `attributes.base[xxx] = ...` 或 `attributes.current_hp -= ...` 不走管线时：

必须停止。

必须输出：

```
ARCHITECTURE VIOLATION: 直接修改属性绕过 Modifier Stack，违反统一修饰管线原则。
```

---

# Trait + Modifier 统一扩展

🟩 不可豁免。

所有能力来源统一通过 Trait + Modifier 体系：

```
Race → Trait
Job → Trait
Talent → Trait
Equipment → Trait
Buff → Trait
```

| 来源 | TraitSource | 示例 |
|------|-------------|------|
| 种族 | Race | 飞行（忽略地形消耗） |
| 职业 | Class | 战士（近战加成） |
| 装备 | Equipment | 火焰武器（OnAttack 施加燃烧） |
| Buff | Buff | 狂暴（OnTurnStart 增加攻击） |
| 天赋 | Talent | 龙裔（OnKill 恢复HP） |

允许：

- 新增 TraitSource 枚举变体
- 新增 TraitTrigger 触发时机
- 新增 TraitEffect 效果类型
- TraitTrigger 触发效果
- TraitEffectHandler 分发效果

禁止：

- 🟥 为不同能力来源创建独立的扩展机制
- 🟥 绕过 TraitCollection 直接查询能力
- 🟥 硬编码 Trait 效果

必须：

- Trait 变化时重建 TraitCollection
- 跨模块 Trait 效果走 Message

---

# ModifierRule

修饰规则 = 标签匹配 + Calculator trait 分发

匹配条件：

```
source_tag（攻击方技能标签）
AND
target_tag（目标标签集合）
```

允许：

- 多规则链式叠加
- 自定义 Calculator 注册

禁止：

- 🟥 match 分发效果类型
- 🟥 绕过 Calculator 直接计算

必须：

- 伤害 ≥ 1
- 治疗 ≥ 0
- 每步修饰记录 ModifierEntry

---

# 数据驱动

## 注册表

所有配置通过 RON 文件加载。

### 配置数据三级分离（v4 新增）

```
content/*.ron      → 游戏内容（策划可编辑，热重载）
assets/*.ron       → 引擎配置（定义、标签）     ← 仅含引擎必需的配置
src/content/       → 内容桥接代码（RON → Registry）
```

🟥 **游戏内容配置（技能、Buff、关卡等）必须放在 `content/` 目录，不得放在 `assets/` 目录。**

🟥 **`assets/definitions/` 和 `assets/rules/` 中的引擎配置是过渡产物，最终应迁移到 `content/`。**

- RON → XxxDef（TagName 字符串）→ XxxData（GameplayTag 位掩码）→ XxxRegistry（全局注册表）
- 异步 AssetLoader + AssetEvent 响应式管线（禁止同步 load 后立即 .into()）
- LoadingProgress 屏障机制（禁止 OnEnter 时立即 validate_all_references）
- 统一 Registry 模式（所有内容类型共享加载/查询/校验接口）
- MOD 内容通过相同接口注册，后加载覆盖先加载

详见 `docs/01-architecture/content-pipeline.md`。

### 当前注册表路径（过渡期）与目标路径

> 当前 RON 配置数据存放在 `assets/` 目录下（过渡期），最终应迁移到 `content/` 目录（参见 `docs/01-architecture/migration-roadmap.md` Phase 4）。`assets/definitions/` 和 `assets/rules/` 中的引擎配置是过渡产物。

**当前路径（assets/，过渡期）**：

```
assets/units/*.ron       → UnitTemplateRegistry
assets/skills/*.ron      → SkillRegistry
assets/buffs/*.ron       → BuffRegistry
assets/equipment/*.ron   → EquipmentRegistry
assets/items/*.ron       → ItemRegistry
assets/terrains/*.ron    → TerrainRegistry
assets/maps/*.ron        → LevelRegistry
assets/ai/*.ron          → AiBehaviorRegistry
assets/modifier_rules/*.ron → ModifierRuleRegistry
assets/traits/*.ron      → TraitRegistry
```

**目标路径（content/，迁移后）**：

```
content/characters/*.ron    → UnitTemplateRegistry
content/skills/*.ron        → SkillRegistry
content/buffs/*.ron         → BuffRegistry
content/equipments/*.ron    → EquipmentRegistry
content/items/*.ron         → ItemRegistry
content/terrains/*.ron      → TerrainRegistry
content/stages/*.ron        → LevelRegistry
content/ai_behaviors/*.ron  → AiBehaviorRegistry
content/formulas/*.ron      → ModifierRuleRegistry
content/classes/*.ron       → TraitRegistry
```

## 双类型模式

每个领域两种类型：

- `XxxDef`：RON 反序列化用，使用 TagName 字符串
- `XxxData`：运行时用，使用 GameplayTag 位掩码

必须实现：

```
impl From<XxxDef> for XxxData
```

禁止：

- 运行时使用字符串查询标签
- RON 文件中使用位掩码

## 配置原则

🟩 配置定义内容，代码解释配置。

允许：

- 新增内容修改 RON 配置
- 配置热重载

禁止：

- 🟥 新增内容修改逻辑代码
- 配置结构频繁变更

必须：

- 🟩 配置引用关系自动校验
- 🟩 配置兼容性优先于配置优雅
- 🟩 配置型数据尽量不可变
- 🟩 配置结构的稳定性优先于配置格式的优雅性
- 🟩 内容生产效率优先于配置格式的争论

---

# 回合状态机

## 状态层次

```
AppState
├── MainMenu
├── InGame
│   └── TurnPhase (SubState)
│       ├── SelectUnit
│       ├── MoveUnit
│       ├── ActionMenu
│       ├── SelectTarget
│       ├── ExecuteAction
│       ├── WaitAction
│       └── TurnEnd
└── GameOver
```

## 阶段转换

必须通过 `NextState<TurnPhase>` 驱动。

禁止：

- 手动设置 TurnPhase 而不经过 NextState
- 在 OnEnter 中执行跨阶段跳转

## AI 与玩家共享流程

🟥 AI 必须设置 CombatIntent 后进入统一 Effect Pipeline。

禁止：

- 🟥 AI 独立计算伤害
- 🟥 AI 绕过 Effect Pipeline 直接扣血

---

# UI 架构

## 三层分离

```
UiCommand（意图层）→ ViewModel（状态层）→ Panel/Widget（展示层）
```

正式 UI：

```
bevy_ui
```

开发工具：

```
bevy_egui
```

Inspector：

必须保留

## 统一管理

🟩 输入焦点必须全局统一管理。
🟩 拖拽功能必须全局统一实现。

## 调试支持

🟩 所有复杂 UI 必须支持调试与可视化检查。

---

# AI 架构

## 数据驱动

AI 行为从 RON 加载：

```
AiBehavior
├── target_strategy
├── move_strategy
├── skill_strategy
└── skill_priority
```

## 策略扩展

```
TargetSelector trait
MoveSelector trait
SkillSelector trait
```

允许：

- 新增策略实现
- 注册到 AiStrategyRegistry

禁止：

- 🟥 在 decision.rs 中硬编码策略逻辑

---

# Save

存档：

保存 Instance

禁止：

- 🟥 保存 Definition

必须：

- Instance 引用 Definition ID
- 加载时从 Registry 恢复 Definition

---

# Battle Record

战斗记录：

结构化记录所有战斗事件

允许：

- 用于 Replay / 调试 / AI 分析

必须：

- 记录每步修饰详情（ModifierEntry）
- 记录伤害来源和目标
- 记录技能 ID 和效果类型

---

# Reflect

🟩 Reflect 只能用于工具链支持，绝对禁止用于核心运行时逻辑。

仅用于：

- 编辑器
- 调试器
- 配置检查
- 展示、检查、编辑数据

禁止：

- 🟥 战斗计算依赖 Reflect
- 🟥 高频逻辑使用 Reflect 查询
- 🟥 Reflect 用于计算

必须：

- 🟩 需要编辑器支持的数据类型必须实现 Reflect
- 🟩 所有 Reflect 注册必须集中在一个模块中
- 🟩 类型的文档注释必须优先作为编辑器的说明来源

发现战斗代码中 `reflect_*` 调用时：

必须输出：

```
ARCHITECTURE WARNING: 战斗代码使用 Reflect，违反"Reflect 不参与高频计算"原则。
```

---

# Logging

统一：

```
tracing
```

禁止：

```
println!
dbg!
```

日志必须：

- 🟩 结构化
- 🟩 记录状态变化，不记录函数进入退出
- 🟩 Error 包含完整上下文

禁止：

- 🟥 记录每帧 Info/Debug 级别日志，仅允许 Error 级别日志出现在每帧系统中
- 循环内日志
- 🟥 通过堆砌日志进行调试

优先：

- 🟩 使用 Inspector、Replay、Debug Panel 进行调试

---

# Testing

🟩 所有功能必须优先编写测试，其次才是手工验证。

允许：

- Unit Test（验证规则：伤害、Buff、属性、寻路）
- Integration Test（验证 Feature：装备、背包、战斗、升级）
- Scenario Test（验证流程：战斗回合、技能释放、胜负结算）
- 🟩 Battle Replay（验证状态流，复杂 SRPG 逻辑优先使用 Battle Replay 而非 BDD）

禁止：

- 🟥 通过修改业务逻辑让测试通过
- 🟥 通过修改测试适配错误逻辑
- 🟥 删除测试来消除失败

必须：

- 🟩 发现 Bug 后必须先编写重现测试，再修复 Bug
- 🟩 所有战斗相关 Bug 必须通过 Battle Replay 重现并转化为永久测试用例
- 🟩 测试必须覆盖所有核心规则，不追求表面的覆盖率数字
- 🟩 所有修复的 Bug 最终都必须沉淀为测试资产

发现测试与逻辑冲突时：

必须停止。

必须输出：

```
POSSIBLE TEST BUG: [描述冲突]
```

或

```
POSSIBLE LOGIC BUG: [描述冲突]
```

等待确认。

---

# 代码组织

## 文件

- 🟩 一个文件一个主题
- 🟩 优先按业务拆文件，不按代码类型拆文件
- 🟥 禁止创建 `systems.rs`、`components.rs` 巨文件
- 🟥 禁止创建 `utils.rs` 垃圾桶文件
- ⚠️ 单个文件超过 500 行时必须主动提出拆分建议
- ⚠️ 单个文件超过 1000 行时强制拆分

## 函数

- 🟩 一个函数一个主要职责
- 🟩 函数名描述意图，不描述过程
- 🟩 优先 Early Return
- ⚠️ 超过 3 层嵌套必须重构
- ⚠️ 超过 100 行开始警觉
- 🟩 重复三次以上再抽象
- 🟩 可读性优先于复用性

## Trait

- 🟩 Trait 表示能力，不表示分类
- 🟩 Trait 用于扩展点，不用于模拟继承树
- 🟩 Trait 只解决明确的变化点
- 🟩 重复出现三次以上再抽象 Trait
- 🟥 禁止为了"代码优雅"而创建无实际价值的 Trait

禁止：

```
trait Character { ... }
trait Monster { ... }
trait Boss { ... }
```

允许：

```
trait DamageSource { ... }
trait Healable { ... }
trait TargetSelector { ... }
```

## 私有函数规范

🟦 优先使用 Rust 私有可见性表达边界，不使用 `_` 前缀命名私有函数。

仅在模块内有明确区分价值时才使用 `_` 前缀。

---

# 资源与内容生产

🟩 统一 Settings 体系管理所有游戏设置。

必须：

- 🟩 所有资源加载必须可追踪
- 🟩 所有资源的生命周期必须显式管理
- 🟩 字体、音频、配置等所有资源必须分类统一管理
- 🟩 高频修改的资源必须优先支持热重载

编辑器地位：

🟩 编辑器是正式产品的一部分，不是开发工具。

核心资产：

🟩 内容生产能力决定项目上限。
🟩 工具链是长期项目的核心资产。

---

# 生命周期

## 状态切换

🟩 `OnEnter` 和 `OnExit` 系统必须保持轻量。

## 初始化拆分

🟩 重型初始化逻辑必须拆分成多个加载阶段。

## 副作用透明

🟥 状态切换时绝对禁止隐藏副作用。

## 状态机职责

🟥 状态机只负责流程控制，绝对禁止包含业务细节。

## 初始化可恢复

🟩 初始化过程必须可追踪、可恢复、可中断。

---

# 长期维护

- 🟩 代码首先是写给人看的，其次才是写给机器执行的
- 🟩 明确优于聪明
- 🟩 简单优于优雅
- 🟩 稳定优于炫技
- 🟩 删除无用代码通常比写新代码更有价值
- 🟩 社区维护的成本通常低于自维护成本
- 🟩 每引入一个自研系统，必须评估未来五年的维护成本
- 🟩 架构必须每 3 个月进行一次复盘和调整，重点清理过度设计和无用代码
- 🟩 工具链与内容生产能力最终决定项目成败

---

# 性能

原则：

- 🟥 先正确，再优化
- 🟥 先 Profile，再优化
- 🟥 性能问题必须测量
- 🟩 禁止为了性能牺牲代码可读性，除非有明确的 Profile 数据证明该部分是性能瓶颈

优先：

- 🟩 Changed 过滤优于全量扫描
- 🟩 Feature 裁剪优于无脑开启全部功能

禁止：

- 🟥 Reflect 参与高频计算
- 🟥 缓存不定义失效条件
- 🟥 未 Profile 就全局重构
- 🟥 凭直觉进行性能优化

必须：

- 🟩 优先优化热点代码，禁止为了性能进行全局重构

原则：

🟩 大多数独立游戏死于复杂度，而非性能。

---

# Strong ID 系统

- 🟥 跨模块传递 ID 只使用 Strong ID 类型（UnitId/SkillId/BuffId 等），禁止裸 Entity 或 String
- ID 使用 newtype 模式，Display 格式包含类型前缀（如 `Unit(warrior_001)`）
- ID 创建后不可修改，已删除实体的 ID 永不复用
- 存档文件使用 Strong ID 的字符串表示（禁止 u64 数值型内部表示）
- 高频 ID 推荐预计算 u64 哈希缓存（OnceLock），避免 HashMap 重复计算
- define_id! 宏一键生成 newtype + 所有必需 trait（零手写样板）

详见 `docs/01-architecture/ids_design.md`。

---

# 确定性执行规范

- 🟥 所有战斗随机性必须通过确定性 PRNG 管线管理，禁止 rand::thread_rng() 或 SmallRng::from_entropy()
- 🟥 必须使用 ChaCha8Rng（Send + Sync + 跨平台确定性），禁止 xorshift64* 等非标准 PRNG
- 预生成随机流 Component：战斗前为每个 Unit 生成 Vec<u32> 随机流，System 只需 Res<RandomStream>（只读锁）
- 🟥 核心战斗数值禁止使用浮点数（i32/u32），百分比用整数表示，公式先乘后除避免精度丢失
- 中间变量必须提升为 i64 防溢出，计算完成后再截断回 i32
- 🟥 禁止使用 Bevy Entity ID 排序，必须引入确定性 LogicalId（战斗开始时按加载顺序分配）
- 状态哈希：每个 TurnEnd 计算确定性哈希，按 LogicalId 排序后遍历关键状态字段
- 浮点污染隔离墙：战斗逻辑 System 代码禁止出现 f32/f64，ViewModelSet 允许转换为浮点用于 UI 渲染但禁止回写

详见 `docs/01-architecture/determinism_rules.md`。

---

# 性能预算

- 60fps 帧预算 16.67ms，每帧逻辑计算 ≤ 8ms，渲染 ≤ 8ms
- 模块级预算：战斗结算 ≤ 5ms、寻路 ≤ 2ms/次、属性计算 ≤ 0.5ms/次
- 变更检测优化：修改前先判断值是否改变，避免无效 Changed 触发下游 System 每帧执行
- 缓存规范：所有缓存必须定义失效条件，缓存永远不是事实源
- Feature 裁剪优于无脑开启全部功能

详见 `docs/01-architecture/performance_budget.md`。

---

# AI 约束

## 优先级

AI 修改代码时优先级：

```
1. Architecture（本文件）
2. Domain Rules（docs/02-domain/*.md）
3. Test Spec（docs/testing/*.md）
4. Existing Code
```

禁止：

- 🟥 为了通过测试修改业务规则
- 🟥 为了通过业务规则删除测试
- 🟥 违反本文件中的任何禁止项

必须说明：

- 为什么修改
- 影响范围
- 风险

发现修改违反优先级时：

必须停止。

必须输出：

```
PRIORITY VIOLATION: 修改违反了优先级规则。[描述冲突]
Architecture > Domain Rules > Test Spec > Existing Code
```

## AI 反模式黑名单

AI 生成代码前必须对照检查，以下 10 条任何违反必须立即重写：

1. 🟥 把 Entity 当对象：`player.attack(enemy)`
2. 🟥 把 Resource 当全局变量仓库
3. 🟥 创建 `systems.rs` / `components.rs` 巨文件
4. 🟥 滥用事件系统模拟函数调用
5. 🟥 业务逻辑直接操作 UI 组件
6. 🟥 直接修改最终属性值
7. 🟥 为单个实现创建 Trait
8. 🟥 提前为未来需求过度设计
9. 🟥 手写状态变更检测代替 `Added/Changed`
10. 🟥 在每帧系统中打印 Info/Debug 日志

## AI 代码自检清单

AI 生成任何代码后，必须自动完成以下检查并标注结果：

```rust
// ================================================
// Bevy SRPG Architecture v3.0 自检结果
// ================================================
// ✅ 按业务拆分模块：是
// ✅ 配置与运行时分离：是
// ✅ 逻辑与表现分离：是
// ✅ 未使用继承：是
// ✅ 未直接操作UI：是
// ✅ 未直接修改最终属性：是
// ================================================
// ❌ 违反条款：[条款编号]（[描述]）
// [宪法豁免] 理由：[具体技术理由]
// 有效期：[日期]，下次架构复盘时重新评估
// ================================================
```

---

# 可观测性

必须：

- 🟩 关键系统支持单步执行与状态回溯
- 🟩 DamageBreakdown 记录 generate→modify→execute 全链路
- 🟩 系统执行顺序可观察
- 🟩 复杂系统拥有可视化观察窗口
- 🟩 优先使用 Inspector、Replay、Debug Panel 进行调试

调试面板：

```
F1  → Battle Debugger
F2  → Buff Viewer
F3  → Gizmos Overlay
F4  → Damage & Attribute Viewer (Tab 切换)
F5  → Turn Queue Viewer
F6  → Pause
F7  → Step
F12 → World Inspector
```

---

# 日志系统设计

- 🟥 日志是领域事件的消费者，非业务代码主动调用的功能
- 统一使用 tracing 库，禁止 println!/dbg!/log crate
- 五级日志：TRACE（极细粒度）、DEBUG（开发调试）、INFO（核心业务事件）、WARN（可恢复异常）、ERROR（不应发生的异常）
- 🟥 所有 INFO 级别核心业务事件必须通过触发领域事件生成日志，禁止业务代码直接调用 info!
- 日志 Observer 统一放在基础设施层，不侵入业务模块
- 每帧系统中仅允许 ERROR 级别日志
- 结构化字段：日志必须使用结构化字段，禁止字符串拼接
- Feature Gate 控制：Trace/Debug 在 Release 编译移除
- 遥测可选接入 OpenTelemetry，数据脱敏（PII/密钥/IP）

详见 `docs/01-architecture/logging_design.md`。

---

# 领域事件与审计系统

- 🟥 废除 DomainEvent 大枚举，每个事件必须是独立的 Struct 并独立注册为 Bevy Message
- 所有可审计事件实现 Auditable Trait，审计系统通过 Trait 统一收集（无硬编码 Reader）
- EventOrd 确定性排序：同 Tick 内事件按显式排序键排列（禁止时钟时间戳）
- EventWhitelist 白名单：使用 TypeId 实现零分配 O(1) 查询
- 🟥 双轨制日志：Command Stream（输入流，用于确定性回放）vs Audit Trail（审计流，用于调试/统计）
- 🟥 审计轨迹不存储在内存中整个战斗期间的数据，每 1000 条分块流式写入磁盘
- 🟥 audit feature 禁用时审计代码完全不编译（零运行时开销）
- GameplayCue 模式：效果执行后必须发出 DomainEvent，表现层通过事件独立响应

详见 `docs/01-architecture/events_audit_design.md`。

---

# 命令总线架构

- 🟥 所有玩家/AI 操作封装为 Command 对象，禁止直接修改游戏状态
- GameCommand 可序列化枚举（禁止 Box<dyn Command>），支持存档/Replay/网络同步
- 校验层只读、执行层只写，执行阶段不重复校验
- Memento 模式撤销（状态快照），禁止手写 undo 逻辑
- Cursor-based 执行：游标指针追踪队列中的执行位置，支持断点恢复
- ActionQueue 效果链式执行（伤害→Buff→死亡检查→反击），确保顺序确定性
- Command Bus 使用 Exclusive System（需要 &mut World），回合制 SRPG 中合理

详见 `docs/01-architecture/command_bus_design.md`。

---

# 配置系统设计

- 四层配置：EngineConfig（重启生效）→ GameRulesConfig（热重载）→ UserSettings（用户偏好）→ DebugSwitches（运行时内存）
- 反上帝配置：禁止将所有游戏规则塞入单一 Resource，必须按领域拆分为细粒度独立 Resource
- 防抖写入：UI 滑块拖动时只修改内存，停止操作 200ms 后异步写盘
- 战斗锁（Battle Lock）：AppState::InGame 期间禁止 ALL 配置热重载
- MOD 配置使用 Patch 语义（深度合并），禁止 Override（整体替换）
- 加载失败分级：EngineConfig 失败使用默认值+ERROR 日志，禁止 panic

详见 `docs/01-architecture/config_system_design.md`。

---

# 内容数据格式

- RON 配置文件契约：id（命名空间格式）、version（必填）、tags（分类标签）为通用基础字段
- 命名空间强制 ID 格式：`namespace:type:name`（如 `core:skill:fireball`），MOD 生态的基石
- 两阶段加载：Stage 1 Raw Asset（AssetLoader 反序列化）→ Stage 2 Baked Asset（Baker System 校验+转换）
- 字符串 ID 驻留：加载期将字符串 ID 转换为 u64，运行时 HashMap 热路径性能提升 10 倍以上
- 引用机制：通过字符串 ID 引用，加载时解析为 Strong ID，禁止文件路径引用
- MOD 冲突解决：覆盖优先级链（Base → DLC → MOD，后加载覆盖先加载）

详见 `docs/01-architecture/content_data_format.md`。

---

# 全局校验与合法性守卫

- 校验检查点：回合结束、战斗结束、状态转换、关卡加载后、存档加载后、MOD 加载后
- 全局不变量：HP ≥ 0、Buff 有来源、Modifier 有 Source、属性修改走 Modifier 管线
- 违规处理分级：可恢复→修正到合法值+WARN，不可恢复→拒绝变更+ERROR，数据损坏→PANIC+崩溃报告
- 校验执行模式：系统执行 → 状态变更 → 到达检查点 → 执行全局校验 → 通过/修正/拒绝

详见 `docs/01-architecture/validation_rules.md`。

---

# 存档迁移与版本兼容

- 存档使用 SemVer 版本号（MAJOR.MINOR.PATCH），每次格式变更递增
- 🟥 只保存 Instance 数据，禁止保存 Definition；Instance 通过 Definition ID 引用恢复
- 向前兼容：新增字段必须有默认值，删除字段必须提供迁移脚本
- 三步删除原则：标记废弃 → 迁移脚本支持 → 大版本删除
- 降级加载：支持读取旧版本存档（自动执行迁移链），禁止读取新版本存档（向前不兼容）

详见 `docs/01-architecture/save_migration_rules.md`。

---

# 内容格式迁移

- 🟥 技能格式一定会变，必须提前建立迁移系统
- 禁止使用版本子目录（v1/v2/v3），目录保持扁平化，版本信息只在 RON 文件内部
- 迁移函数必须是纯函数（不依赖外部状态），每个函数只处理一个版本跳跃
- 迁移在 AssetLoader::load() 内部执行（禁止独立预处理 pass）
- Core 内容迁移失败必须阻止游戏启动，MOD 内容迁移失败降级为警告+跳过
- Minior 版本变更（新增可选字段）使用 #[serde(default)] 自动兼容，不需要迁移脚本

详见 `docs/01-architecture/content_migration_design.md`。

---

# 资源生命周期管理

- Handle 类型选择：Strong Handle（战斗中使用的资源）vs Weak Handle（调试/可选资源）
- 分阶段卸载：每帧卸载资源总大小不超过 4MB，大资源优先→中资源其次→小资源最后
- SafeAssetRef 封装：缓存有效性状态，避免重复查询 AssetServer
- 内存预算：主菜单 64MB、战斗场景 256MB（PC 8GB+ RAM），低端设备差异化适配
- 技能特效按需加载：战斗开始时预加载本回合可能使用的特效，禁止预加载所有 100+ 技能特效
- 延迟卸载队列：禁止同步调用 unload_unused()，分帧执行避免帧尖峰
- 场景切换清理：使用 BattleEntity 标记组件 + desppawn_recursive() 统一清理

详见 `docs/01-architecture/asset_lifecycle_rules.md`。

---

# 资源命名空间

- `base:category/name` 格式命名空间前缀：base（基础游戏）、mod_xxx（MOD 内容）
- 结构化 AssetId（namespace + category + name），AssetKey(u64) 预计算哈希用于热路径
- Resolution Chain 解析优先级：User Override > Active Mods (Priority) > Base Game
- MOD 覆盖规则：视觉/表现层开放覆盖，逻辑/规则层限制覆盖（如 formulas/* 禁止覆盖）
- 向后兼容：无冒号前缀的 ID 自动添加 base: 前缀
- 命名空间冲突检测：MOD 加载阶段检测重复命名空间和保留名冲突

详见 `docs/01-architecture/asset_namespace_design.md`。

---

# 战斗有限状态机

- 战斗 FSM 管理宏观流程：PreBattle → RoundStart → PlayerPhase → EnemyPhase → TurnEnd → VictoryCheck → RoundEnd → PostBattle
- TurnPhase（SubState）在 PlayerPhase/EnemyPhase 内激活，管理回合内微观阶段
- Guard/Action/Effect 三段式：Guard（纯函数只读）→ Action（同步变更）→ Effect（事件发射）
- GuardContext 预计算：Phase 进入时批量收集所有查询数据，避免每帧 ECS 随机访问
- 一帧延迟反模式：FSM 转换存在一帧延迟，Event→FSM 反馈必须通过下一帧 Guard 重新评估
- 不变量：PreBattle 必须先于所有战斗阶段、终态后不可逆、PlayerPhase 和 EnemyPhase 互斥

详见 `docs/01-architecture/battle_fsm_design.md`。

---

# 技能/Buff/Effect 统一抽象

- 反模式警告：一个技能=一个函数，做到 500 技能彻底爆炸
- 核心设计：500 技能 + 1000 Buff 收敛为 20~30 个 Effect Executor，新增内容只改配置不改代码
- Effect Pipeline 三步：Generate（生成效果）→ Modify（修饰规则）→ Execute（执行效果）
- Skill = SkillDef（配置）+ SkillCooldowns（运行时），通过 CombatIntent → Effect Pipeline 执行
- Buff = 临时 Trait + 临时 Modifier，持续时间按 Turn 计算
- ActionQueue 效果执行队列：确保伤害→Buff→死亡→反击的链式执行顺序确定性

详见 `docs/01-architecture/skill-buff-abstraction.md`。

---

# 寻路与范围计算

- PathFinder trait 抽象：支持 BFS（默认）、A*、Dijkstra 算法替换
- RangeCalculator 统一接口：移动范围、攻击范围、技能范围共享相同的参数和返回结构
- UnitBlocker 阻挡规则：FriendlyBlocker、EnemyBlocker、AllBlocker、NoBlocker
- 缓存软失效策略：120 帧未使用的缓存条目自动标记为无效
- 大地图兜底策略：31-50 地图每帧最多 8 个路径请求，>50 地图每帧 4 个请求
- 并行计算支持：多单位同时计算范围时可用 rayon 并行加速

详见 `docs/01-architecture/pathfinding_design.md`。

---

# 国际化系统

- 🟥 绝对禁止依赖第三方 bevy_fluent 插件，基于 fluent-rs 自封装 Bevy 适配层
- Content 数据只存储本地化 KEY，从不存储文本本身（key: "skill.s_1001.name"）
- 🟥 Key 使用永久唯一 ID（禁止语义化名称如 skill.fireball.name），一旦创建不允许修改
- LocalizedText 组件：标记 UI 元素需要本地化，仅在语言切换或组件刚添加时解析
- MOD 本地化隔离：MOD 自带 localization/ 目录，Key 使用 mod.{mod_id}. 前缀
- 字体回退链：多语言字体按优先级排列，Cosmic Text 支持 CJK 混排
- Fluent 术语（Term）全局复用：核心属性名定义为 Term，所有引用自动同步

详见 `docs/01-architecture/i18n_design.md`。

---

# MOD 支持架构

- MOD API 稳定公开接口：register_skill/buff/equipment/character/stage，MOD 作者唯一需要了解的部分
- MOD 沙箱约束：禁止直接访问 World/Commands/Entity，禁止绕过 Effect Pipeline
- 分级权限策略：Level 0（官方，可扩展战斗逻辑）、Level 1（社区，仅内容扩展）、Level 2（测试，全权限）
- 内容合并：基础内容先加载，MOD 按 priority 顺序加载，后加载覆盖先加载
- 核心内容不可覆盖（白名单机制）：基础职业定义、元素交互规则、回合状态机等
- WASM 沙箱预留接口（Phase 1 只定义 trait，不引入 WASM 运行时依赖）

详见 `docs/01-architecture/modding-design.md`。

---

# 开发工具链

- 🟥 Tools 是开发者工具链，永不进入发布构建
- 按需实现原则：仅在内容量增长到人工审查不可行时启动
- data_validator：自动检查 SkillId 重复、Buff 不存在、Quest 引用链断裂等
- content_linter：检查 RON 格式合规、命名规范、标签一致性
- balance_checker：分析数值平衡性、伤害分布、属性成长曲线
- 所有 Tools 必须支持 headless 模式用于 CI

详见 `docs/01-architecture/tools_architecture.md`。

---

# 测试体系架构

- 五层测试金字塔：单元测试 → 领域集成测试 → 系统集成测试 → E2E 回归测试 → Testbeds 调试沙盒
- Battle Replay 测试：复杂 SRPG 逻辑优先使用 Battle Replay 而非 BDD
- failure_snapshots：战斗 Bug 必须通过 Battle Replay 重现并转化为永久测试用例
- TestCharacterBuilder / TestBattleBuilder：Builder 模式构建测试夹具
- 确定性优先：同状态+同输入+同种子=同结果，禁止系统时间依赖
- Golden Test：战斗回放、配置导出用金文件对比

详见 `docs/01-architecture/testing_architecture.md`。

---

# UI-领域交互边界

- 🟥 CQRS 是灵魂：读路径（Query ViewModel）无副作用，写路径（UiCommand → CommandHandler）收口
- 单向数据流：Core → UI 通过 ViewModel（只读 Resource），UI → Core 通过 UiCommand Message
- UI 只读 ViewModel + 只输出 UiCommand，禁止直接查询 ECS 组件修改业务状态
- CommandHandler 写入收口：所有 UI 写操作必须经过 CommandHandler
- ViewModel 更新机制：ViewModel 变化时刷新（非每帧轮询），使用 Change Detection
- Core 层无 UI 知觉：Core 代码禁止 import 任何 UI 模块类型

详见 `docs/01-architecture/ui_domain_boundary_rules.md`。

---

# 停止条件

发现以下情况，必须停止并报告：

1. 🟥 属性修改绕过 Modifier 管线
2. 🟥 死亡处理跳过 Dead Tag
3. 🟥 效果执行跳过 Pipeline
4. 🟥 UI 直接修改游戏状态
5. 🟥 跨模块直接访问内部实现
6. 🟥 新增硬编码替代数据驱动
7. 🟥 Definition 被运行时修改
8. 🟥 Buff 无来源或永不过期
9. 🟥 穿脱装备跳过 Trait 重建
10. 🟥 存档保存 Definition 数据
11. 🟥 Core 层依赖 Infrastructure 或 UI 层（v4 新增）
12. 🟥 Shared 层依赖任何其他层（v4 新增）
13. 🟥 新增内容修改 Rust 代码（v4 新增）
14. 🟥 领域错误放在 shared/ 或 infrastructure/（v4 新增）
15. 🟥 shared/ 出现 xxx_utils 垃圾桶模块（v4 新增）

---

# 架构文档索引

> 以下文档是本文件的详细补充，共同构成完整的架构体系。

| 文档 | 内容 |
|------|------|
| `architecture/project-structure.md` | 完整项目目录结构（源码树 + 资产树 + 内容树） |
| `architecture/layer-contracts.md` | 七层架构边界定义、依赖规则、禁止区域 |
| `architecture/error-architecture.md` | 错误体系设计（领域错误 / 基础设施错误 / 共享工具） |
| `architecture/content-pipeline.md` | 内容管线（Rule/Content 分离、数据驱动架构） |
| `architecture/modding-design.md` | MOD 支持架构设计 |
| `architecture/asset-organization.md` | 美术资产组织与外包工作流 |
| `architecture/collaboration-model.md` | AI 协作、多人协作、外包美术模型 |
| `architecture/migration-roadmap.md` | 从当前架构到目标架构的分阶段迁移计划 |
| `architecture/infrastructure-design.md` | 基础设施层深度设计（日志/存档/回放/热重载等 20 个模块） |
| `architecture/app-bootstrap.md` | App 层与游戏启动装配设计 |
| `architecture/plugin-design.md` | Plugin 组织方式与注册顺序设计 |
| `architecture/skill-buff-abstraction.md` | 技能/Buff/Effect 统一数据驱动抽象模型 |
| `architecture/component_design_rules.md` | Bevy Component 设计规范（标记/数据/状态组件三位一体） |
| `architecture/system_design_rules.md` | Bevy System 编写铁律（粒度/参数边界/读写分离） |
| `architecture/plugin_contract_rules.md` | Plugin 边界与依赖契约（显式依赖/公共API/分层禁令） |
| `architecture/ids_design.md` | 强类型 ID 系统架构（newtype/分配策略/生命周期） |
| `architecture/events_audit_design.md` | 领域事件 + Audit 审计系统架构 |
| `architecture/content_data_format.md` | Content 数据格式规范（RON 配置契约） |
| `architecture/command_bus_design.md` | 命令总线架构（输入→验证→执行抽象层） |
| `architecture/determinism_rules.md` | 确定性执行规范（PRNG/精度/迭代排序/状态哈希） |
| `architecture/battle_fsm_design.md` | 战斗有限状态机设计（阶段/转换/扩展点） |
| `architecture/pathfinding_design.md` | 寻路与范围计算架构（A* 抽象/缓存/性能预算） |
| `architecture/schedules_design.md` | Bevy Schedule 与 SystemSet 组织架构 |
| `architecture/asset_lifecycle_rules.md` | 资源生命周期管理（Handle/加载卸载/热重载同步） |
| `architecture/config_system_design.md` | 配置系统设计（四层配置/平衡参数/热重载） |
| `architecture/performance_budget.md` | 性能预算与优化基线（60FPS/模块预算/门禁） |
| `architecture/validation_rules.md` | 全局数值校验与合法性守卫 |
| `architecture/testing_architecture.md` | 完整测试体系架构（五层测试/Testbeds/CI） |
| `architecture/ui_domain_boundary_rules.md` | UI-领域交互边界（只读/单向/分离契约） |
| `architecture/logging_design.md` | 日志系统设计（tracing/五级日志/结构化字段） |
| `architecture/save_migration_rules.md` | 存档格式迁移与版本兼容策略 |
| `architecture/i18n_design.md` | 国际化系统架构（Fluent/.ftl/Key驱动/LocalizedText/MOD翻译） |
| `architecture/tools_architecture.md` | 开发工具链架构（Data Validator/Content Linter/Balance Checker） |
| `architecture/content_migration_design.md` | 内容格式迁移设计（Content Format V1→V2→V3 迁移链） |
| `architecture/asset_namespace_design.md` | 资源命名空间设计（base:/mod_x: 前缀防止 MOD 冲突） |
| `architecture/feature_flag_design.md` | Feature Flag 架构（replay/debug_ui/cheat/modding 条件编译） |

---

# 版本修订说明

**v4.2**（当前版本）：
- 来源：`docs/其他/32.国际化.md` + `docs/其他/33遗漏2.md`
- 新增国际化系统架构（Fluent + Key驱动内容 + MOD翻译）
- 新增开发工具链架构（Data Validator + Content Linter）
- 新增内容格式迁移设计（Content Migration Chain）
- 新增资源命名空间设计（base:/mod_x: 前缀）
- 新增 Feature Flag 架构（8 个条件编译开关）

**v4.1**：
- 来源：`docs/其他/31遗漏.md` 全面补充
- 新增 19 篇架构文档，覆盖：ECS 代码层规范（Component/System/Plugin）、核心数据架构（IDs/Events/Audit/Content Format/Command Bus）、SRPG 核心机制（Determinism/Battle FSM/Pathfinding/Schedules）、工程质量（Asset Lifecycle/Config/Performance/Validation）、测试体系（Testing Architecture）、边界守则（UI-Domain/Logging/Save Migration）

**v4.0**：
1. 新增七层架构（App/Core/Shared/Infra/Content/Modding/Tools）
2. 新增三问判断法（Core/Infra/Shared 归层标准）
3. 新增 Content 层核心区分（Skill 是 Core，Fireball 是 Content）
4. 新增错误架构三层模型（领域/基础设施/共享）
5. 新增项目根目录三级分离（src/assets/content）
6. 新增层间依赖规则和禁止区域
7. 新增 Infrastructure 层深度设计（20 个模块）
8. 新增 App 启动装配与 Plugin 组织设计
9. 新增技能/Buff/Effect 统一抽象模型（10 层补充系统）
10. 来源：`docs/其他/30.md` + `docs/其他/27技能buf抽象.md`

**v3.0**：
原有架构规范，包含 ECS、属性系统、Effect Pipeline、UI 架构、AI 架构等完整规则。
