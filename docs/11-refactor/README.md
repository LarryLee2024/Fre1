---
id: 11-refactor.README
title: Refactor
status: stable
owner: refactor-guardian
created: 2026-06-14
updated: 2026-06-16
tags:
  - refactor
---

# Refactor

技术债扫描记录。

## 说明

由 @refactor-guardian 定期扫描代码库生成的技术债清单，使用 `Debt-XXX` 格式记录。

扫描目标：
- 死代码（未使用的 Components/Systems/Events/Resources）
- 重复代码（重复逻辑、重复 Modifier/Buff）
- 结构退化（超大文件、模块边界违规）
- Bevy SRPG 特有债务（Pipeline 绕过、ECS 反模式、Observer 风暴）

> 待首次技术债扫描后填充具体条目。
