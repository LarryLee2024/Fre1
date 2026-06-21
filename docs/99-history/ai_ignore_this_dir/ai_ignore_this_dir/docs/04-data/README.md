---
id: 04-data.README
title: Data
status: stable
owner: feature-developer
created: 2026-06-14
updated: 2026-06-16
tags:
  - data
---

# Data

数据与配置文档 — 内容格式、配置系统、资产管理。

## 文档列表

| 文件 | 主题 |
|------|------|
| `asset-lifecycle-rules.md` | 资源生命周期、Handle 类型、内存预算 |
| `asset-organization-rules.md` | 三树分离、命名空间 |
| `config-system-rules.md` | 运行时配置、热重载 |
| `content-migration-rules.md` | 版本兼容、字段兼容 |
| `content-system-rules.md` | RON 加载、Registry、Definition 不可变 |
| `feature-flag-rules.md` | Feature Flag、灰度发布 |
| `validation-rules.md` | 数据完整性、配置校验 |

## 子目录

| 目录 | 说明 |
|------|------|
| `bo3/` | 参考数据（Bo3 数据模型参考） |
| `ll/` | 铃兰之剑参考数据（属性/Modifier/技能等 Schema 参考） |
| `rule/` | 规则参考数据 |

共 7 个数据规则文件 + 3 个参考数据子目录，覆盖资源生命周期、内容管线、配置系统、数据校验等数据治理领域。
