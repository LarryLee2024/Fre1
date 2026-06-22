---
name: reference-bevy-source
description: Bevy 引擎源码和示例的本地路径
metadata:
  type: reference
---

# Bevy 源码与示例路径

- **源码**：`/Users/lf380/Code/Github/bevy/crates`
- **示例**：`/Users/lf380/Code/Github/bevy/examples`

使用场景：需要查阅 Bevy API 实现、示例用法、或验证 BSN/Feathers 等新功能如何使用时，优先从本地直接读取源码和示例，而非依赖网络搜索或记忆。

**Why**: 本地源码保证与项目所用版本 (Bevy 0.19) 完全一致，且不依赖网络。

**How to apply**: 遇到 Bevy API 行为不明确、需要了解新功能用法、或调试引擎行为时，先查本地 `crates/` 和 `examples/`，再查网络文档。
