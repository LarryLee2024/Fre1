---
name: project-architecture
description: DDD 三层+横切四层，能力系统三层，15+15 领域划分
metadata:
  type: project
---

# 项目架构不变式

这些是长期稳定的事实，不从代码推导。

## 总体架构

- **DDD 纵向三层**: Shared(L0) → Core(L1) → Infra(L2)
- **横切四层**: App, Content, Tools, Modding
- **依赖方向严格单向**: Shared ← Core ← Infra，禁止反向

## 能力系统（Capabilities）

15 个通用机制，每个三层内聚结构：
- Foundation（纯类型定义）
- Mechanism（核心逻辑 + ECS System）
- Plugin（唯一对外入口）

## 业务领域（Domains）

15 个业务子系统，每个标准结构：
- rules/（纯函数规则，零 ECS）
- systems/（ECS Systems）
- integration/（Facade 模式，跨 Capability 访问）
- plugin.rs（唯一入口）

## 通信

- 四级通信：Hook > Trigger > Observer > Message
- 禁止 `EventWriter/EventReader`，统一用 `trigger(T)` + `On<T>` Observer
- Domain 间禁止直接引用，只通过 Event

## 数据

- 四层分离：Definition / Spec / Instance / Persistence
- Effect Pipeline 是战斗数值变更唯一入口
- Modifier 不拥有业务逻辑
