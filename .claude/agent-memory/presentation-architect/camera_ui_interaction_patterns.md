---
name: camera-ui-interaction-patterns
description: Camera-UI 交互模式已产出完整设计文档 (docs/06-ui/04-data-flow/camera-ui-interaction.md)
metadata:
  type: reference
---

Camera-UI 交互设计要点（2026-06-21 产出）：

**核心原则**：
- UI → Camera 通过直接 trigger CameraRequest（不经过 UiCommand，Camera 是 Infra）
- Camera → UI 通过 CameraQuery 只读查询 + CameraStateVm 直接写入（唯一 Camera→UI 通道）
- 坐标转换（world→screen）发生在 Overlay/Widget 系统（渲染时），不在 Projection
- CameraInputBlock Resource 使用计数堆叠实现输入阻塞（UI 写，Camera 读）

**7 种交互场景覆盖**：
1. Tooltip 定位 - TooltipVm.world_position + CameraQuery::world_to_screen
2. 输入阻塞 - CameraInputBlock (block_count 堆叠)
3. Minimap 导航 - map_size 比例计算（不需 CameraQuery）
4. 单位聚焦 - UI 协调 UiCommand + CameraRequest 同时触发
5. 战斗日志跳转 - CameraRequest::Focus (一次性动画)
6. 镜头状态显示 - CameraStateVm (Camera 直接写入)
7. z-order - Bevy 默认行为，无需处理

**架构例外记录**：
- CameraStateVm 由 Camera system 直接写入，不经过 Projection（因 Camera 状态帧相关）
  → 这是 UI 架构中唯一一个 "直接写入" 通道，已显式记录在文档中

**新增的架构规则 (8 条)**：
CAM-UI-01 到 CAM-UI-08，详见 `camera-ui-interaction.md §5.1`

**需要修改的文件 (12 个)**：详见 `camera-ui-interaction.md §4.2`
