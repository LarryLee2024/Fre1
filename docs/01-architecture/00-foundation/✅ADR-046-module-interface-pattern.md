---
id: 01-architecture.00-foundation.ADR-046
title: "ADR-046: 模块接口模式统一（消除 api.rs，统一到 integration/）"
status: Accepted
owner: architect
created: 2026-06-18
updated: 2026-06-18
tags:
  - architecture
  - module-interface
  - acl
  - integration
  - facade
---

# ADR-046: 模块接口模式统一（消除 api.rs，统一到 integration/）

## 状态

**Accepted**

## 背景

项目存在三种模块接口模式混用：

| 模式 | 位置 | 数量 |
|------|------|------|
| mod.rs 路由 | 全项目 | ~80+ |
| integration/ (ACL) | combat, tactical | 2 |
| api.rs | combat, replay | 2 |

三种模式并存导致：

1. **AI 找不到入口** — 50万行后，AI 不知道该找 api.rs 还是 integration/
2. **开发者困惑** — 新增跨域访问时不知道该用哪个模式
3. **审查标准不统一** — code-reviewer 无法统一判断接口合规性
4. **api.rs 语义模糊** — 可能表示公开接口、Facade、Service、Application Service、SDK、RPC、HTTP

### 参考依据

`docs/其他/2.md` — 基于项目 DDD + ACL + 14 领域 + 50万行的实践分析，明确推荐：

> mod.rs = 模块组织者
> plugin.rs = Bevy 注册者
> integration/ = 领域门面（唯一对外入口）
> api.rs = 删除

## 引用的领域规则

- `docs/01-architecture/README.md` §6.2 — Business Domain 标准结构
- `docs/00-governance/ai-constitution-complete.md` §9 — 封装原则
- `docs/00-governance/coding-rules.md` — 最小可见性原则

## 决策

### 三层角色定义

```
┌─────────────────────────────────────────────────────────────┐
│  mod.rs — 模块组织者                                         │
│  职责: 纯 submodule 声明 + barrel re-export                   │
│  规则: 零逻辑、零函数、零 impl                                │
├─────────────────────────────────────────────────────────────┤
│  integration/ — 领域唯一对外入口                              │
│  职责: 跨域访问 ACL / Facade / Gateway                       │
│  规则: facade.rs + types.rs + system_param.rs 三件套          │
│  适用: 所有需要被其他域访问的 domain                          │
├─────────────────────────────────────────────────────────────┤
│  plugin.rs — Bevy 注册者                                     │
│  职责: Plugin 实现 + System/Resource 注册                     │
│  规则: 不含业务逻辑                                          │
└─────────────────────────────────────────────────────────────┘
```

### 核心规则

1. **mod.rs 零逻辑** — 只允许 `mod` 声明、`use` 重导出、`#[cfg(test)]` 声明、文档注释
2. **integration/ 是唯一跨域入口** — 跨域访问只能通过 integration/
3. **禁止新增 api.rs** — 已有的 api.rs 迁移到 integration/ 或合并到 mod.rs
4. **禁止直接访问其他域的内部 model/service/component**

### integration/ 标准结构

```
domain/
├── integration/
│   ├── mod.rs              # 声明子模块
│   └── <capability>/
│       ├── mod.rs          # re-export facade + types + system_param
│       ├── facade.rs       # 业务语义函数（唯一访问外部类型的地方）
│       ├── types.rs        # View Types（newtype 替代裸类型）
│       └── system_param.rs # Bevy SystemParam（封装查询依赖）
├── plugin.rs
└── mod.rs
```

### 决策矩阵

| 场景 | 用什么 | 为什么 |
|------|--------|--------|
| 模块内部组织 | mod.rs 路由 | 不需要对外暴露 |
| 域间只读查询 | integration/ | 统一入口，语义明确 |
| Domain → Capabilities | integration/ | ACL 屏蔽内部类型 |
| Domain → Domain | integration/ + Event | 禁止直接数据引用 |
| Infra → Core | integration/ | 通过 facade 访问 |

## 禁止

- 🟥 禁止新增 `api.rs` 文件
- 🟥 禁止 mod.rs 包含函数定义、impl 块、struct/enum/trait 定义
- 🟥 禁止跨域访问绕过 integration/
- 🟥 禁止 Systems 直接 import 其他域的组件类型

## 后果

### 正面
- 接口模式统一，AI 和开发者都能快速定位入口
- 审查标准统一，code-reviewer 可以明确判断
- 域边界更清晰，耦合更可控

### 负面
- 已有 api.rs 需要迁移（combat, replay — 2 个文件）
- 部分域可能暂时没有 integration/（按需创建，不提前建空目录）

## 参考

- `docs/其他/2.md` — 模块接口模式分析
- `docs/01-architecture/00-foundation/ADR-045-module-visibility-strategy.md` — 可见性策略（ADR-045）

## 后续更新

### D2-4: Integration 层覆盖全 13 个 Domain 确认

本 ADR 发布后，integration 层已在全部 13 个需要跨域访问的业务 Domain 中落地，证实了 Query Facade 模式的有效性：

| Domain | Integration Facade | 模式 |
|--------|-------------------|------|
| combat | `combat/integration/ability/`, `combat/integration/aggregator/`, `combat/integration/effect/` | 每 Capability 子模块 |
| spell | `spell/integration/facade.rs` | 单 facade.rs (Read + Write) |
| inventory | `inventory/integration/facade.rs` | 单 facade.rs (Read + Write) |
| reaction | `reaction/integration/facade.rs` | 单 facade.rs |
| economy | `economy/integration/facade.rs` | 单 facade.rs |
| progression | `progression/integration/facade.rs` | 单 facade.rs |
| faction | `faction/integration/facade.rs` | 单 facade.rs |
| party | `party/integration/facade.rs` | 单 facade.rs |
| narrative | `narrative/integration/facade.rs` | 单 facade.rs |
| summon | `summon/integration/facade.rs` | 单 facade.rs |
| terrain | `terrain/integration/facade.rs` | 单 facade.rs |
| quest | `quest/integration/facade.rs` | 单 facade.rs |
| crafting | `crafting/integration/facade.rs` | 单 facade.rs |
| camp_rest | `camp_rest/integration/facade.rs` | 单 facade.rs |

**结论**：Query Facade 模式（facade.rs + types.rs + system_param.rs 三件套）作为 integration/ 标准结构已在全项目范围内验证可行。combat 域因需要对接多个 Capabilities 而采用每 Capability 子模块结构，其余 Domain 均使用单 facade.rs 模式。详见 `docs/02-domain/factories.md` §4.2。
