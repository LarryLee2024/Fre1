---
id: 08-decisions.README
title: Architecture Decision Records
status: stable
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - adr
---

# Architecture Decision Records

架构决策记录。

| 文件 | 主题 |
|------|------|
| `ADR-001-migration-plan.md` | 迁移总计划 |
| `ADR-002-技术债修复方案.md` | 技术债治理策略 |
| `ADR-003-分层契约与依赖迁移.md` | 七层架构落地 |
| `ADR-004-内容与数据迁移方案.md` | 配置数据迁移 |
| `ADR-005-插件与通信迁移方案.md` | 插件体系与通信 |
| `ADR-006-验证与测试迁移方案.md` | 测试体系迁移 |
| `ADR-007-目录结构迁移映射.md` | 源码/资产/内容目录 |
| `ADR-008-核心机制与工程质量迁移.md` | 核心机制与质量门禁 |
| `ADR-009-迁移合规修正与架构决策.md` | 迁移合规修正 |
| `ADR-010-测试迁移与重整方案.md` | 测试体系迁移与重整 |
| `ADR-011-错误模块实施.md` | 三层错误体系落地（先做） |
| `ADR-012-日志模块与统一事件目录.md` | 日志模块修复 + shared/event/ 统一事件管理（后做） |
| `ADR-013-技能数据模型与配置规范.md` | SkillDef RON 格式、双类型模式、版本管理 |
| `ADR-014-技能释放管线设计.md` | 五阶段释放管线、验证逻辑、冷却管理、Effect Pipeline 衔接 |
| `ADR-015-技能标签与分类体系.md` | GameplayTag 在技能分类、修饰匹配、AI 决策中的驱动作用 |
| `ADR-016-技能系统扩展点设计.md` | 新增效果/条件/目标类型的扩展机制 |
| `ADR-017-国际化架构决策.md` | Fluent (.ftl) 技术选型、Key 永久 ID、LocalizedText 组件、语言回退链 |
| `ADR-018-国际化迁移方案.md` | 35个RON配置+21处UI代码的分阶段渐进迁移、永久ID分配表 |
| `ADR-020-Buff数据模型与配置规范.md` | BuffDef/BuffData 双类型模式、RON 格式契约、版本管理、扁平字段迁移策略 |
| `ADR-021-Buff生命周期与持续策略.md` | DurationPolicy 7种持续策略、StackPolicy 3种叠层策略、tick 生命周期规范、apply_buff 扩展 |
| `ADR-022-Buff触发系统与事件架构.md` | TriggerRegistry、TriggerHandler trait、TriggerContext、Effect Pipeline 衔接、分阶段迁移 |
| `ADR-023-标签系统架构重整.md` | 标签系统架构重整：三重数据源消除、TagCategory 扩展、label() 废弃、register_defaults() 删除、u64 耗尽预案、统一 rebuild_tags() |
| `ADR-024-标签系统迁移方案.md` | 标签系统分阶段迁移计划（4 Phase）：RON 扩展 → API 迁移 → label() 全域替换 → 清理与验证 |
| `ADR-025-七领域模块化架构设计.md` | 七领域(Tag/Modifier/Buff/Effect/Targeting/Ability/Trigger)独立模块化：目录结构、Plugin 注册顺序(DAG)、跨模块通信、数据流方向、EffectDef 所有权迁移 |
| `ADR-026-SRPG-Lite-GAS-架构对齐.md` | SRPG Lite-GAS 冻结架构对齐：10+3模块(10业务+3基建)、删除独立Buff、新增Execution/Cue/Stacking/Attribute、GAS执行链时序冻结、12条Forbidden |
| `ADR-027-业务模块执行计划结果.md` | 业务模块执行计划与结果 |
| `ADR-028-logging-error-architecture-review.md` | 日志与错误架构审查 |
| `ADR-029-数据模型完全重构总纲.md` | 基于铃兰之剑数据模型的完全重构总纲：5阶段策略、13领域全覆盖、执行原则 |
| `ADR-030-ID系统与Registry基础设施重构.md` | define_id! 宏、统一Registry trait、22个Registry强类型化、删除重复ID |
| `ADR-031-核心属性与标签系统重构.md` | AttributeKind 24→11变体、TagCategory 10→4类、控制层级3级模型 |
| `ADR-032-Effect管线全链路重构.md` | 7阶段Pipeline、删除完整Buff模块、Execution trait全量替代旧路径 |
| `ADR-033-Ability与Trigger系统重构.md` | Skill*→Ability*全量重命名、5阶段释放管线、Trigger 5大类20+事件 |
| `ADR-034-Cue与Replay与I18n系统实现.md` | 12种Cue全覆盖、Replay录制/播放/验证、FTL国际化体系 |
| `ADR-035-模块清理与迁移执行计划.md` | 文件级删除/新建清单、6阶段门禁检查表、RON 41文件迁移总清单 |
