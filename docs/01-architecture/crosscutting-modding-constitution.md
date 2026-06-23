---
id: CROSSCUT-MODDING-CONSTITUTION
title: 横切关注点与Modding宪法
status: accepted
stability: stable
layer: architecture
related:
  - ai-constitution-complete.md
  - architecture-constitution.md
tags:
  - crosscutting
  - modding
  - infrastructure
---

> **原文来源**：`ai-constitution-complete.md` 第四编（L700-L713）、第五编（L716-L727）
>
> **锚定总宪法**：第四编、第五编

## 第四编 Modding 能力体系
Modding 不是独立层级，而是贯穿多层的扩展能力，按职责拆分到对应层级，保证边界可控。

| 模块 | 归属 | 职责 |
|------|------|------|
| `core/mod_api/` | Core 层 | 对外暴露的稳定 Mod 接口，Facade + Gateway 模式，唯一合法的核心规则访问入口 |
| `content/mod_support/` | Content 层 | Mod 内容加载、数据覆盖、冲突处理、注册逻辑 |
| `infrastructure/mod_loader/` | Infrastructure 层 | Mod 文件扫描、沙箱隔离、版本校验、依赖管理 |

### 约束
- 🟥 Mod 禁止绕过稳定 API 直接访问 Core 内部实现
- 🟩 Mod 能力与原生内容走同一套 Registry 与执行管线
- 🟩 API 分级：稳定 API / 实验性 API / 内部 API（Mod 禁止调用）

---

## 第五编 横切关注点治理
不单独设立 `crosscutting/` 目录，采用「**抽象定义在 Shared，具体实现在 Infrastructure**」的模式，既保证复用性，又不破坏单向依赖链。

| 横切能力 | 抽象归属 | 实现归属 | 说明 |
|----------|----------|----------|------|
| 日志 Logging | Shared/logging_trait | Infrastructure/logging | Core 仅依赖 Trait，不依赖具体日志框架 |
| 指标 Metrics | Shared/metrics_trait | Infrastructure/analytics | 性能指标、业务指标埋点 |
| 审计 Audit | Shared/audit_trait | Infrastructure/audit | 关键操作审计轨迹 |
| 遥测 Telemetry | - | Infrastructure/analytics | 纯技术层能力，Core 无感知 |
| 事务 Transaction | Shared/transaction_trait | Infrastructure/transaction | 战斗结算原子性保障 |
| 安全 Security | - | Infrastructure/security | 加密、反作弊等纯技术能力 |
