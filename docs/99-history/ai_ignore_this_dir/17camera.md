如果你的目标是 **Bevy 0.19 + SRPG（铃兰之剑/FFT/Tactics Ogre）+ 50万行项目**，那么 Camera 千万不要一开始就写成一个简单系统：

```rust
commands.spawn(Camera2d);
```

因为后面一定会长成：

```text
地图浏览
镜头跟随
技能演出
对话聚焦
边缘滚屏
键盘移动
鼠标拖拽
缩放
镜头限制
战斗回放
震屏
过场动画
```

最后 Camera 会变成一个独立子系统。

---

# 大型项目推荐架构

不要放：

```text
src/map/
src/ui/
```

里面。

应该单独领域：

```text
src/
├── domains/
│   ├── camera/
│   │   ├── components.rs
│   │   ├── resources.rs
│   │   ├── events.rs
│   │   ├── systems/
│   │   ├── plugin.rs
│   │   └── mod.rs
```

因为 Camera 实际是全局系统。

---

# Camera Domain

## Components

```rust
#[derive(Component)]
pub struct MainCamera;
```

---

```rust
#[derive(Component)]
pub struct CameraFollow {
    pub target: Entity,
}
```

---

```rust
#[derive(Component)]
pub struct CameraBounds {
    pub min: Vec2,
    pub max: Vec2,
}
```

---

# Resource

```rust
#[derive(Resource)]
pub struct CameraState {
    pub zoom: f32,
    pub mode: CameraMode,
}
```

---

```rust
pub enum CameraMode {
    Free,
    FollowUnit(Entity),
    Cinematic,
}
```

---

# Event驱动

不要：

```rust
camera.translation = ...
```

到处乱改。

---

定义：

```rust
pub struct FocusUnitEvent {
    pub unit: Entity,
}
```

---

```rust
pub struct ShakeCameraEvent {
    pub strength: f32,
}
```

---

```rust
pub struct MoveCameraToEvent {
    pub world_pos: Vec2,
}
```

---

然后 Camera Domain 消费。

---

# SRPG 推荐 Camera 模式

## 1. Free Camera

玩家拖地图。

```text
WASD
方向键
鼠标中键拖拽
边缘滚屏
```

---

## 2. Follow Camera

例如：

```text
角色移动
```

镜头自动跟随。

---

## 3. Focus Camera

例如：

```text
施法
```

发送：

```rust
FocusUnitEvent
```

镜头移动过去。

---

## 4. Cinematic Camera

例如：

```text
Boss登场
剧情
大招演出
```

进入：

```rust
CameraMode::Cinematic
```

暂停玩家输入。

---

# Zoom

不要直接改 Camera。

统一：

```rust
CameraZoomRequest {
    delta: f32
}
```

---

Camera Domain：

```rust
projection.scale = target_zoom;
```

---

推荐：

```text
0.5x
1x
2x
3x
```

分档。

不要无限滚轮。

---

# Map Bounds

必须有。

否则玩家能看到地图外。

```rust
pub struct CameraBounds {
    min: Vec2,
    max: Vec2,
}
```

每帧：

```rust
camera_pos.x =
    camera_pos.x.clamp(min.x, max.x);
```

---

# Camera Shake

单独组件：

```rust
#[derive(Component)]
pub struct CameraShake {
    timer: Timer,
    strength: f32,
}
```

---

事件：

```rust
ShakeCameraEvent
```

---

不要：

```rust
camera.translation += random();
```

散落各处。

---

# 多层级设计

我推荐：

```text
L0
Camera Data

L1
Camera Commands

L2
Camera Behaviors

L3
Camera Effects

L4
Gameplay Integration
```

---

例如：

```text
L0
CameraState

L1
MoveCameraToEvent

L2
FollowSystem

L3
ShakeSystem

L4
CombatFocusSystem
DialogueFocusSystem
```

---

# 对你的项目最重要的一条

不要让：

```text
Combat
Dialogue
Quest
Map
UI
```

直接修改 Camera。

全部通过：

```rust
MoveCameraToEvent
FocusUnitEvent
ShakeCameraEvent
```

驱动。

即：

```text
Gameplay
    ↓ Event
Camera Domain
    ↓
Camera Entity
```

这样以后加：

```text
过场动画
回放系统
录像系统
多人同步
```

都不会推翻 Camera 架构。

对于你这种目标接近《铃兰之剑》《博德之门3》的大型战棋项目，我会把 Camera 当成一个独立 Domain，而不是 Map 的附属功能。这样后期扩展成本最低。
有，而且这些往往是很多人做到中后期才踩坑的内容。下面不重复前面说的 Follow/Zoom/Shake/Event，而是讲 **大型 SRPG Camera 架构里更深层的设计点**。

---

# 1. Camera 不要直接存 Entity

很多人会写：

```rust
pub enum CameraMode {
    Follow(Entity),
}
```

前期没问题。

后期：

```text
单位死亡
单位被移除
单位切地图
单位被替换
录像回放
网络同步
```

容易出问题。

更稳的是：

```rust
pub enum CameraTarget {
    Unit(UnitId),
    Tile(TilePos),
    WorldPos(Vec2),
}
```

Camera 只认领域 ID。

然后 Camera Domain 自己 Query。

这是你现在 DDD 架构一直在强调的：

```text
Entity 是 ECS实现细节
UnitId 才是领域身份
```

Camera 也应该遵守。

---

# 2. Camera 不要直接操作 Transform

很多项目最终会出现：

```rust
camera_transform.translation = ...
```

几十个系统都在改。

应该拆成：

```rust
CurrentCameraPose
TargetCameraPose
```

例如：

```rust
pub struct CameraPose {
    pub position: Vec2,
    pub zoom: f32,
    pub rotation: f32,
}
```

系统流程：

```text
Gameplay
 ↓
TargetCameraPose
 ↓
Interpolation
 ↓
Transform
```

最终：

```text
Transform
```

只是渲染结果。

---

# 3. Camera 应该支持优先级栈（非常重要）

后期一定会出现：

```text
角色移动
↓
技能释放
↓
Boss登场
↓
剧情插入
```

谁控制镜头？

很多项目：

```rust
CameraMode::Follow
CameraMode::Cinematic
CameraMode::Focus
```

互相覆盖。

最后变成屎山。

---

推荐：

```rust
CameraRequest {
    priority: u32,
}
```

例如：

```text
普通跟随
priority = 10

技能演出
priority = 100

剧情
priority = 1000
```

Camera 永远执行最高优先级。

---

# 4. Camera 应该支持 Source Token

例如：

```text
剧情系统请求镜头
↓
镜头切换
↓
剧情结束
```

如果直接：

```rust
SetCameraMode(...)
```

恢复很麻烦。

推荐：

```rust
CameraToken
```

例如：

```rust
let token = camera.push_request(...);

camera.pop_request(token);
```

类似：

```text
UI Modal
Audio Bus
Input Capture
```

的管理方式。

大型项目很好用。

---

# 5. Camera 不应该知道 Combat

例如：

```rust
FocusAttackTargetEvent
```

Camera 不应该订阅。

Camera 不应该出现：

```text
Attack
Skill
Dialogue
Quest
```

这些词。

应该统一：

```rust
CameraFocusRequest
CameraMoveRequest
CameraLookAtRequest
```

Camera 永远是通用系统。

---

# 6. Camera 边界要支持动态更新

很多人：

```rust
MapLoaded
↓
设置边界
```

结束。

实际上后期会出现：

```text
地图扩张
战场变化
多区域切换
地下城层切换
```

推荐：

```rust
CameraConstraint
```

组件化。

例如：

```rust
MapConstraint
CutsceneConstraint
ArenaConstraint
```

最终合并计算。

而不是：

```rust
resource.bounds
```

一个全局变量。

---

# 7. Camera 与输入彻底解耦

不要：

```rust
mouse_wheel
↓
camera zoom
```

直接写。

推荐：

```rust
Input
↓
CameraCommand
↓
Camera
```

例如：

```rust
CameraCommand::ZoomIn
CameraCommand::MoveLeft
CameraCommand::Pan
```

以后：

```text
键盘
手柄
触屏
AI演示
录像回放
```

全部复用。

---

# 8. Camera 要有逻辑坐标系

不要让 Gameplay 知道：

```rust
Vec3
Transform
Projection
```

---

Gameplay：

```rust
TilePos
WorldPos
UnitId
```

---

Camera：

```rust
screen_to_world()
world_to_screen()
```

统一封装。

这样未来：

```text
正交
透视
伪3D
真3D
```

Gameplay 不变。

---

# 9. Camera 应支持 Script Driver

这个很多独立开发者不会提前设计。

例如：

```text
Boss出现
↓
镜头移动
↓
停1秒
↓
聚焦Boss
↓
返回玩家
```

不要：

```rust
Timer
State
if else
```

硬编码。

---

推荐：

```rust
CameraSequence
```

例如：

```rust
MoveTo(...)
Wait(...)
Focus(...)
Return(...)
```

未来剧情系统直接调用。

---

# 10. Camera 最终应该成为服务

很多项目最后会形成：

```text
Map Camera
Combat Camera
Dialogue Camera
```

三套。

维护灾难。

---

推荐：

```text
Camera Domain
```

唯一镜头服务。

提供：

```rust
request_focus(...)
request_move(...)
request_zoom(...)
request_sequence(...)
```

所有系统调用它。

---

# 如果是你的项目

我会在 ADR 里直接加一条：

```text
ADR-CAMERA-001

Camera Domain 不允许依赖任何 Gameplay Domain。

Camera 是全局服务层。

Combat / Dialogue / Quest / AI
只能发送 CameraRequest。

禁止直接修改 Camera Entity。
```

这一条看似简单，但能保证你未来几十万行代码里，Camera 不会变成到处都能碰的“上帝对象”。这比 Follow、Zoom、Shake 本身更重要。
还有，而且下面这些属于 **做到 20~50 万行代码以后才会意识到的重要设计**。前面讲的是 Camera 本身，这次讲的是 **Camera 在整个游戏架构中的定位**。

---

# 1. Camera 应该支持 Replay（录像系统）

很多项目 Camera 是：

```text
当前状态
↓
立即计算
↓
显示
```

这样录像系统很难做。

---

推荐：

```rust
CameraCommand
```

全部可记录：

```rust
Focus(UnitId)
MoveTo(WorldPos)
ZoomTo(1.5)
```

录像记录：

```text
Frame 100
Focus(Unit42)

Frame 120
ZoomTo(2.0)

Frame 150
MoveTo(...)
```

---

回放：

```text
Replay
↓
重放 CameraCommand
↓
恢复镜头
```

未来：

```text
战斗录像
BUG复现
自动测试
```

全部受益。

---

# 2. Camera 应该支持 Observer 模式

很多人认为：

```text
玩家视角
```

只有一个。

实际上后期可能有：

```text
主镜头
小地图
战术视图
观战模式
回放模式
AI调试模式
```

---

不要：

```rust
Resource<CameraState>
```

唯一实例。

---

推荐：

```rust
CameraId
```

```rust
Main
Minimap
Replay
Debug
```

即使现在只用一个。

未来扩展成本极低。

---

# 3. Camera 不应该拥有地图知识

不要：

```rust
camera.rs
```

出现：

```rust
MapSize
MapGrid
TileMap
```

---

应该：

```rust
CameraConstraintProvider
```

提供：

```rust
min_x
max_x
min_y
max_y
```

---

Camera 不知道：

```text
地图是TMX
地图是LDtk
地图是Hex
地图是方格
```

统统不知道。

---

# 4. Camera 与渲染层分离

很多人：

```rust
OrthographicProjection
```

写得到处都是。

---

推荐：

```rust
CameraView
```

```rust
position
zoom
rotation
```

---

最后一层：

```rust
CameraView
↓
BevyCameraAdapter
↓
OrthographicProjection
```

这样未来：

```text
2D
2.5D
3D
```

切换成本低。

---

# 5. Camera Rotation 提前预留

SRPG 初期：

```text
永远不旋转
```

很正常。

但后期：

```text
战斗演出
剧情
地图预览
```

经常需要。

---

即使暂时不用：

```rust
CameraPose {
    position,
    zoom,
    rotation,
}
```

也建议保留。

---

否则未来：

```rust
CameraPose
```

全项目重构。

---

# 6. Camera 应支持 Query API

不要让外部：

```rust
Query<&Transform, With<MainCamera>>
```

到处查。

---

统一：

```rust
CameraService
```

例如：

```rust
camera.world_to_screen(...)
camera.screen_to_world(...)
camera.visible_rect(...)
```

---

未来：

```text
多镜头
缩放
旋转
```

实现变化。

调用方不变。

---

# 7. Camera 可见区域缓存

很多大型项目后期会有：

```text
Fog
AOI
LOD
动态加载
```

---

不要每个系统：

```rust
camera.visible_rect()
```

自己算。

---

推荐：

```rust
VisibleRegion
```

Resource。

每帧算一次：

```rust
VisibleRegion
```

所有系统共享。

---

例如：

```text
Map Streaming
Fog
AI
Particle
```

都能用。

---

# 8. Camera 应支持 Temporal Lock

例如：

```text
玩家拖镜头
↓
技能演出开始
↓
玩家继续拖
```

怎么办？

---

推荐：

```rust
CameraInputLock
```

例如：

```rust
Unlocked
GameplayLocked
CinematicLocked
```

---

不要：

```rust
if cutscene {
    return;
}
```

散落系统。

---

# 9. Camera 应支持 Predictive Focus

战棋特别有用。

例如：

```text
鼠标悬停技能
↓
预览攻击区域
↓
镜头轻微移动
```

---

或者：

```text
角色即将移动到远处
↓
提前平滑跟随
```

---

不要等移动完成：

```text
啪
镜头瞬移
```

体验差很多。

---

# 10. Camera 调试接口

大型项目必备。

我一般会做：

```rust
CameraDebugInfo
```

显示：

```text
Current Mode
Target Mode
Current Zoom
Target Zoom
Current Position
Target Position
Priority Owner
Current Sequence
```

---

否则后期出现：

```text
为什么镜头不动？
为什么镜头乱跳？
为什么技能演出结束没恢复？
```

非常难查。

---

# 最后一个真正容易被忽略的点

## Camera 应该是 State Machine，而不是一堆系统

很多项目最后：

```text
follow_system
zoom_system
shake_system
cutscene_system
dialogue_system
...
```

二十几个系统互相抢控制权。

---

我更推荐：

```rust
CameraController
```

内部维护：

```rust
Idle
FreeMove
Follow
Focus
Sequence
Cinematic
```

状态机。

---

然后：

```text
Input
Gameplay
Dialogue
Quest
```

都只是提交 Request。

真正决定：

```text
当前镜头行为
```

的是 CameraController。

这是大型项目里 Camera 从「功能集合」进化成「子系统」的关键一步。

如果让我给你的项目补最后一条 ADR，我会写：

```text
ADR-CAMERA-002

Camera 是一个独立状态机系统，
而不是若干直接修改 Transform 的系统集合。

所有镜头行为必须通过 CameraController 仲裁。
```

这一条往往决定项目后期镜头代码是 3000 行还能维护，还是变成 3 万行谁也不敢动的模块。
