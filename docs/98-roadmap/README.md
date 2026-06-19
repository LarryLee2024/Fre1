---
id: 98-roadmap.README
title: Roadmap
status: stable
owner: architect
created: 2026-06-14
updated: 2026-06-16
tags:
  - roadmap
---

# Roadmap

项目路线图。

## 迁移路线

当前项目已从扁平结构迁移到七层架构，详见 `docs/01-architecture/README.md`。

## 当前阶段

项目处于 v0.1.0 开发阶段，已完成：
- 七层架构搭建与迁移
- 战斗 FSM 与 Effect Pipeline
- 技能/Buff/装备/背包系统
- AI 行为系统
- 地图与寻路
- 回合制状态机
- 调试面板
- MOD 支持基础框架
- 国际化（Fluent）基础框架
- 数据驱动配置（RON）

## 后续规划

- 效果管线全链路重构（ADR-029~035）
- UI 正式化（从 egui 调试面板到正式 UI）
- 网络对战基础
- 音频系统集成
- 存档/回放系统完善
