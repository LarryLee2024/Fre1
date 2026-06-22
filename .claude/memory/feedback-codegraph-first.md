---
name: feedback-codegraph-first
description: 查代码优先使用 CodeGraph 而非直接读文件或 grep
metadata:
  type: feedback
---

# 查代码优先用 CodeGraph

CodeGraph 一次调用返回源码 + 调用链 + 依赖关系，比 grep+read 循环快 10 倍。

**How to apply**: 无论理解流程、找定义、查调用者，都先调 `codegraph_explore` 或 `codegraph_node`。仅在 CodeGraph 未覆盖时用 Read 或 bash grep。
