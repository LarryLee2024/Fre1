---
id: 00-governance.project-constitution
title: SRPG 项目总宪法（总纲）
version: 5.3
status: accepted
stability: stable
layer: governance
related:
  - architecture-constitution.md
  - crosscutting-modding-constitution.md
  - ecs-constitution.md
  - srpg-systems-constitution.md
  - ui-constitution.md
  - data-save-constitution.md
  - observability-constitution.md
  - performance-constitution.md
  - coding-constitution.md
  - quality-maintenance-constitution.md
  - architecture-governance.md
  - ai-redline-constitution.md
  - test-constitution.md
  - resource-state-constitution.md
tags:
  - governance
  - constitution
  - architecture
  - bevy
  - srpg
---

# Bevy 0.19+ SRPG 项目总宪法 v5.3（总纲）

> 效力说明：本宪法对项目所有架构设计、代码编写、AI生成内容具有最高约束力，优先级高于任何通用编程规范、语言习惯或AI默认输出。条款编号永久固定，违反条款即视为不合格输出。
>
> 本文件为总纲，仅包含总则、P0铁则、架构总览与各编引用指针。各编完整内容见对应子宪法文件。

本版本在 v4.1 基础上完成核心架构对齐：将 Core 层内部结构从「5级纵向分层（L0~L4）」升级为「**Capabilities 能力层 + Domains 业务域 + Mod API**」双轴结构，与项目架构设计文档完全对齐。

---

## 第一编 总则

### 1.1 适用范围
- 引擎：Bevy 0.19 及以上版本
- 品类：单机战棋 SRPG
- 规模：50万行+代码量级
- 模式：长期连载式内容迭代
- 核心特性：Replay First、Data Driven、GAS-Lite
- 扩展目标：自动化测试、战斗模拟器、服务器模拟、Mod 生态

### 1.2 架构优先级
所有架构与代码决策严格遵循以下优先级，禁止倒置：
```
正确性 > 可维护性 > 可扩展性 > 开发效率 > 性能
```
性能优化必须基于 Profiling 实证数据，禁止凭体感提前优化。

### 1.3 强制等级说明
| 标记 | 等级 | 说明 |
|------|------|------|
| 🟥 | 绝对禁止 | 任何情况下都不允许出现，不可豁免 |
| 🟩 | 必须遵守 | 无例外强制执行，除非获得明确豁免 |
| 🟨 | 优先选择 | 除非有明确且可验证的技术理由，否则必须采用 |
| 🟦 | 最佳实践 | 推荐但非强制，无技术理由时优先采用 |
| ⚠️ | 警觉阈值 | 达到阈值时必须主动提出重构建议 |

### 1.4 豁免规则
- 所有违反宪法的代码必须标注 `[宪法豁免]` 并说明理由、有效期、审批人
- 豁免代码每3个月必须重新评估
- 性能类豁免必须附带 Profiling 实证数据与 ADR 架构决策记录

### 1.5 P0 级顶层铁则
以下原则具有最高优先级，所有层级与模块必须无条件遵守。
1. **Feature First**：永远按业务领域组织代码，禁止按技术类型拆分全局目录
2. **Data Driven First**：新增内容优先通过配置数据实现，禁止硬编码业务内容
3. **Replay First**：所有核心战斗逻辑必须可确定性重放，禁止不可控随机源
4. **Logic / Presentation Separation**：业务逻辑与视觉表现彻底解耦，禁止混写
5. **Composition Over Inheritance**：所有差异化通过原子能力组合实现，禁止继承式设计
6. **Capabilities/Domains 双轴架构原则**：Core 层采用「纵向 Capabilities 通用机制复用 + 横向 Domains 业务内聚」双轴结构，禁止单维度无限分层
7. **Localization First**：所有用户可见文本禁止直接进入 Rust 代码，必须通过 LocalizationKey 引用；Def 只存 name_key/desc_key 等 Key，不存任何自然语言文本；Replay/Event/BattleLog 只存 Key + 参数，不存最终翻译文本；存档禁止保存翻译结果，只存 ID/Key
8. **五层能力架构（Type→Tag→Query→Rule→Content）**：游戏逻辑遵循五层分工——Type System 管规则（强类型参与计算）、Tag System 管语义（描述性标签不影响规则）、Query System 管筛选（统一查询入口）、Rule System 管逻辑（数据驱动规则）、Content System 管配置（RON/YAML/Mod）。参与规则计算的东西必须强类型（Type），用于筛选分类内容驱动的东西用 Tag
9. **Camera Event 驱动（ADR-064）**：所有外部镜头操作必须通过 `commands.trigger(CameraRequest::...)` 事件驱动，禁止直接修改 Camera Entity 的 Transform/Projection
10. **Map 内容管线三层分离（ADR-065）**：地图内容遵循 Tiled(TMX) → Importer(构建时) → MapAsset(RON) 三层管线
11. **Tile-Config 分离（ADR-065）**：TileEntry 只存 terrain_id，所有 Gameplay 数值归 TerrainDef Config Registry
12. **Object-Entity 分离（ADR-065）**：Map Object 是定义（不可变），运行时由 Domain System 决定是否/何时实例化为 ECS Entity

---

## 架构总览

```
src/
├── main.rs                   # 程序入口
├── lib.rs                    # 库根
│
│                           ┌─ DDD 纵向三层 ─┐
├── shared/                   # L0：底层原子层
├── core/                     # L1：领域规则层（Capabilities + Domains 双轴）
├── infra/                    # L2：技术实现层
│                           └─────────────────┘
│                           ┌─ 横切四层 ─┐
├── app/                      # 横切1：启动装配层
├── content/                  # 横切2：内容桥接层
├── tools/                    # 横切3：开发工具层
└── modding/                  # 横切4：Mod 扩展层
                            └─────────────┘
```

### 依赖方向总规则

```
# DDD 纵向三层：L0 Shared ← L1 Core ← L2 Infrastructure
Shared → Core → Infrastructure

# 横切层与纵向层的关系
App     → 知道所有层（仅装配，不含业务逻辑）
Content → Core + Infrastructure（只做加载/校验/注册）
Tools   → Core + Shared（开发期专用）
Modding → Core/mod_api（唯一对外暴露的核心接口）
```

🟥 严格禁止反向依赖：Core 不得依赖 Infrastructure/Content/UI，Shared 不得依赖任何业务层。

### Core 层双轴结构

```
src/core/
├── capabilities/               # 纵向：15个核心能力领域（通用机制骨架）
├── domains/                    # 横向：15个业务子系统（承载全部玩法复杂度）
└── mod_api/                    # Mod 稳定 API（Facade + Gateway 模式）
```

**一句话总结：Capabilities 管「机制」，Domains 管「规则」。**

---

## 各编引用指针

| 编章 | 标题 | 完整内容位置 |
|------|------|-------------|
| 第二编 | 纵向三层 + 横向四层架构体系 | [01-architecture/architecture-constitution.md](../01-architecture/architecture-constitution.md) |
| 第三编 | Core 层双轴架构 | [01-architecture/architecture-constitution.md](../01-architecture/architecture-constitution.md) |
| 第四编 | Modding 能力体系 | [01-architecture/crosscutting-modding-constitution.md](../01-architecture/crosscutting-modding-constitution.md) |
| 第五编 | 横切关注点治理 | [01-architecture/crosscutting-modding-constitution.md](../01-architecture/crosscutting-modding-constitution.md) |
| 第六编 | ECS 宪法 | [02-domain/ecs-constitution.md](../02-domain/ecs-constitution.md) |
| 第七编 | 模块化与 Plugin 边界宪法 | [01-architecture/architecture-constitution.md](../01-architecture/architecture-constitution.md) |
| 第八编 | SRPG 核心系统专项宪法 | [02-domain/srpg-systems-constitution.md](../02-domain/srpg-systems-constitution.md) |
| 第九编 | UI 系统宪法 | [06-ui/ui-constitution.md](../06-ui/ui-constitution.md) |
| 第十编 | 数据驱动与存档宪法 | [04-data/data-save-constitution.md](../04-data/data-save-constitution.md) |
| 第十一编 | 可观测性宪法 | [00-governance/observability-constitution.md](./observability-constitution.md) |
| 第十二编 | 测试与确定性宪法 | [05-testing/test-constitution.md](../05-testing/test-constitution.md) |
| 第十三编 | 资源与内容生产宪法 | [01-architecture/resource-state-constitution.md](../01-architecture/resource-state-constitution.md) |
| 第十四编 | 性能宪法 | [00-governance/performance-constitution.md](./performance-constitution.md) |
| 第十五编 | 生命周期与状态机宪法 | [01-architecture/resource-state-constitution.md](../01-architecture/resource-state-constitution.md) |
| 第十六编 | 代码组织与编写规范 | [00-governance/coding-constitution.md](./coding-constitution.md) |
| 第十七编 | 长期维护与运营宪法 | [00-governance/quality-maintenance-constitution.md](./quality-maintenance-constitution.md) |
| 第十八编 | 工程质量与技术债治理 | [00-governance/quality-maintenance-constitution.md](./quality-maintenance-constitution.md) |
| 第十九编 | 架构治理与演进 | [00-governance/architecture-governance.md](./architecture-governance.md) |
| 第二十编 | AI 专属执行规范 | [00-governance/ai-redline-constitution.md](./ai-redline-constitution.md) |
| 第二十一编 | 红线禁止事项总览 | [00-governance/ai-redline-constitution.md](./ai-redline-constitution.md) |
| 第二十二编 | Localization 专项规则 | [00-governance/ai-redline-constitution.md](./ai-redline-constitution.md) |

---

## 附则

### 修订说明
- 本宪法版本：v5.3（Bevy 0.19+）
- 发布日期：2026-06-20
- 本文件为总纲版，各编完整内容见对应子宪法文件
- 修订周期：每半年根据 Bevy 版本更新和项目实践进行一次修订
- 效力期限：永久有效，除非发布新版本宪法明确替代

### 文档治理规范
文档分级、Rule ID、YAML frontmatter 等元规范详见 [doc-governance/](./doc-governance/doc-governance-overview.md)
