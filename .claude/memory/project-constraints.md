---
name: project-constraints
description: 技术选型约束和禁止事项
metadata:
  type: project
---

# 项目技术约束

## 禁止

- 禁止全局 `AppError` 枚举，必须用 thiserror 分领域定义
- 禁止 `#[cfg(test)] mod tests` 内联测试
- 禁止绕过 Effect/Modifier 管线直接修改战斗数值
- 禁止修改 Definition（配置）数据，Def 全局不可变
- 禁止 Core 层依赖 Infra 层
- 禁止 Domain 间直接引用——只通过 Event 通信

## 强制

- 所有用户可见文本必须用 `LocalizationKey`，禁止硬编码
- 所有跨域测试放在 `tests/{unit,invariant,integration,fixtures}/` 子目录
- 测试用 `cargo nextest run`
- 核心战斗逻辑必须可确定性重放
