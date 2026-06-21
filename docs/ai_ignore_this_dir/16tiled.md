这是一个非常关键的问题。

因为对于大型项目来说：

> **Tiled 不是地图文件格式。**
>
> **Tiled 本质上是“关卡内容编辑器（Content Authoring Tool）”。**

如果你未来要做《铃兰之剑》《FFT》《火纹》《博德3》这种级别的项目，那么你在设计 Bevy 架构之前，必须先知道：

```text
Tiled 能生产什么数据？
Tiled 不能生产什么数据？
```

否则后面一定会出现：

```text
地图系统推翻
Importer重写
Content重构
```

---

# 一、Tiled 的核心能力模型

Tiled 本质上提供：

```text
地图
├── Tile
├── Layer
├── Object
├── Property
├── Tileset
└── Template
```

实际上：

```text
地图 = 图层树 + 对象树 + 属性系统
```

理解这一点最重要。

---

# 二、Tile Layer（瓦片层）

这是最常见功能。

例如：

```text
Ground
Road
Grass
Water
Mountain
```

每个格子：

```text
(x,y) -> TileID
```

本质：

```rust
Vec<TileId>
```

---

支持：

### Rect Brush

矩形刷

```text
■■■■
■■■■
■■■■
```

---

### Stamp Brush

图章刷

例如：

```text
树林
房子
石头群
```

一次刷出来。

---

### Terrain Brush

地形刷。

例如：

```text
草地
草地
草地

↓↓↓

草地-沙地边缘
```

自动拼接。

类似：

```text
RPG Maker
Unity Rule Tile
```

---

### Random Brush

随机刷。

例如：

```text
草1
草2
草3
```

随机出现。

---

### Wang Tile

极其重要。

自动道路：

```text
道路
河流
城墙
围栏
```

自动连接。

---

# 三、Object Layer

很多人低估它。

实际上：

> Object Layer 才是大型项目最重要功能。

---

对象支持：

### Point

点

例如：

```text
出生点
NPC点
摄像机点
```

---

### Rectangle

矩形区域

例如：

```text
触发区域
战斗区域
剧情区域
```

---

### Polygon

多边形

例如：

```text
视野区域
Boss区域
```

---

### Polyline

折线

例如：

```text
巡逻路径
移动路径
```

---

### Ellipse

圆形区域

例如：

```text
警戒范围
AOE范围
```

---

### Tile Object

地图对象

例如：

```text
宝箱
门
机关
```

---

# 四、Properties（最重要）

这是大型项目核心。

任何东西都能挂属性。

---

地图属性

```text
Map
├─ biome=forest
├─ chapter=3
└─ weather=rain
```

---

Layer属性

```text
Layer
├─ collision=true
├─ move_cost=2
```

---

Object属性

```text
Chest
├─ item_id=1001
├─ gold=500
```

---

Tile属性

```text
Grass
├─ move_cost=1
├─ cover=10
```

---

# 属性类型

支持：

```text
bool
int
float
string
color
file
object
class
enum
```

---

# 五、Custom Class

这是新版 Tiled 的神级功能。

例如：

定义：

```text
UnitSpawn
```

包含：

```text
unit_id
level
team
ai_type
```

---

之后：

```text
SpawnPointA
SpawnPointB
SpawnPointC
```

全部自动拥有。

---

类似：

```rust
struct UnitSpawn {
    unit_id: String,
    level: i32,
    team: Team,
}
```

---

# 六、Tileset

瓦片集。

支持：

### 单图集

```text
terrain.png
```

---

### Collection

每张图单独。

```text
tree.png
rock.png
chest.png
```

---

# Tile Metadata

极其重要。

例如：

```text
Grass
```

挂：

```text
move_cost=1
```

---

```text
Forest
```

挂：

```text
cover=30
```

---

```text
Water
```

挂：

```text
swimmable=true
```

---

# 七、Tile Animation

支持动画瓦片。

例如：

```text
水
火
岩浆
瀑布
```

---

```text
frame1
frame2
frame3
frame4
```

循环。

---

# 八、Layer Group

图层分组。

例如：

```text
Map
├─ Terrain
│   ├─ Ground
│   ├─ Water
│   └─ Cliff
│
├─ Decoration
│   ├─ Tree
│   └─ Grass
│
└─ Gameplay
    ├─ Spawn
    └─ Trigger
```

---

大型项目必用。

---

# 九、Infinite Map

无限地图。

支持：

```text
Chunk
Chunk
Chunk
```

动态扩展。

---

不过：

SRPG基本不用。

---

# 十、Template

模板系统。

例如：

```text
ChestTemplate
```

定义：

```text
loot_table
gold
respawn
```

---

以后：

```text
ChestA
ChestB
ChestC
```

继承。

---

类似：

```rust
Prefab
```

---

# 十一、World

极其重要。

很多人不知道。

---

支持：

```text
world.world
```

管理：

```text
forest.tmx
town.tmx
castle.tmx
```

---

形成：

```text
大世界
```

---

类似：

```text
Pokemon
Chrono Trigger
Golden Sun
```

地图切换。

---

# 十二、Terrain System

支持：

```text
Grass
Sand
Rock
Water
```

定义规则。

自动连接。

---

# 十三、Wang Set

Terrain 的升级版。

支持：

```text
道路
河流
墙壁
海岸线
```

自动生成边缘。

---

这是 Tiled 最复杂系统之一。

---

# 十四、脚本扩展（JavaScript）

支持：

```javascript
exportMap()
validate()
generateObjects()
```

---

可以：

```text
检查地图规范
自动生成数据
批量修改
```

---

大型项目非常有价值。

---

# 十五、导出格式

支持：

```text
TMX(XML)
JSON
Lua
```

---

以及：

```text
自定义导出
```

---

# 十六、真正适合你的功能

如果目标是：

```text
50万行代码
SRPG
DDD
Bevy
```

真正重要的是：

```text
★★★★★ Object Layer
★★★★★ Properties
★★★★★ Custom Class
★★★★★ Tile Metadata
★★★★★ Template

★★★★ Layer Group
★★★★ Tileset Metadata

★★★ Wang Set
★★★ Terrain

★★ Tile Animation

★ Infinite Map
★ World
```

---

# 最后给你的架构建议

不要把 Tiled 当成：

```text
地图编辑器
```

而要当成：

```text
内容生产工具
```

未来你的 Importer 应该导入：

```text
Tile Layer
Object Layer
Property
Class
Template
Tile Metadata
```

而不仅仅是：

```text
TileID
```

很多 Bevy 项目最后推倒重来，就是因为最开始只导入了：

```text
TileLayer -> Vec<Tile>
```

等做到剧情、任务、宝箱、刷怪、触发器、AI 巡逻时，才发现 **Tiled 真正有价值的数据（Object + Property + Class）根本没进入游戏架构。**

对于你这种长期目标项目，我会把 **Object Layer + Class + Property** 视为一等公民，甚至比 Tile Layer 本身还重要。
有，而且是很多大型项目后期才踩到的坑。上面你已经覆盖了 Tiled 的主要功能，但如果你的目标真的是 **50万行以上 SRPG**，下面这些属于「架构级影响」，很多教程根本不会讲。

---

# 1. Tile 不要承载 Gameplay

很多新人会这样：

```text
Grass Tile
├─ move_cost=1
├─ cover=10
├─ defense=5
```

然后游戏逻辑直接读 Tile 属性。

看起来合理。

后期一定出问题。

---

例如：

```text
普通草地
被火烧的草地
下雨后的草地
剧情中的草地
```

实际上已经变成：

```text
Grass
BurningGrass
WetGrass
StoryGrass
```

指数爆炸。

---

大型项目做法：

```rust
TerrainDef
```

来自配置。

Tiled只存：

```text
terrain_id=grass
```

而不是：

```text
move_cost=1
cover=10
...
```

---

Importer：

```text
grass
 ↓
TerrainId(Grass)
 ↓
Config Database
```

---

# 2. 不要把 Object 当 Entity

很多人会：

```text
Chest Object
 ↓
直接生成 ECS Entity
```

看似方便。

---

后期：

```text
存档
剧情
重置
回放
联机
```

全部崩。

---

正确做法：

```rust
MapObjectId
```

例如：

```text
chest_001
door_002
npc_003
```

---

地图只提供：

```text
Object Definition
```

运行时生成：

```rust
Game Entity
```

---

永远分离。

---

# 3. Object GUID

这是大型项目必须补的。

Tiled本身对象ID：

```text
1
2
3
4
```

不稳定。

删除一个对象：

```text
1
2
4
5
```

可能变化。

---

因此：

每个重要对象：

```text
guid=
3f4a90d7
```

---

或者：

```text
spawn_goblin_001
```

---

未来：

```text
存档
任务
剧情
```

全部依赖它。

---

# 4. Layer 不等于系统

很多人设计：

```text
Collision Layer
```

然后：

```text
有这个Layer
=
有碰撞
```

---

后期：

```text
悬崖
水
岩浆
毒气
飞行
```

开始爆炸。

---

推荐：

Layer负责：

```text
组织
编辑
显示
```

---

Gameplay靠：

```text
Property
Class
Config
```

驱动。

---

# 5. 地图验证器（非常重要）

大型项目必须有：

```text
Map Validator
```

---

启动Importer时：

自动检查：

```text
是否存在重复GUID

是否存在无效Terrain

是否存在无效UnitID

是否存在非法Trigger

是否存在断链事件
```

---

否则：

```text
运行两小时
发现地图配置错
```

很痛苦。

---

# 6. Import阶段生成导航数据

很多人：

```text
运行时
A*
```

计算。

---

大型项目：

Importer阶段直接生成：

```rust
MapAsset {
    terrain,
    movement_mask,
    height_map,
}
```

---

甚至：

```text
Region
Zone
Portal
```

提前烘焙。

---

运行时更快。

---

# 7. Height（高度）提前规划

SRPG迟早会碰：

```text
楼梯
高台
屋顶
桥
悬崖
```

---

即使第一版不用。

Importer也要预留：

```rust
height: i16
```

---

否则未来重构地图格式。

---

# 8. Region系统

这是很多SRPG核心。

例如：

```text
Boss Area

Village Area

Castle Area

Bridge Area
```

---

不要用：

```text
100个Trigger
```

实现。

---

Importer阶段：

```rust
RegionId
```

生成。

---

AI和剧情直接查询：

```rust
region.contains(pos)
```

---

# 9. Spawn不要直接写单位

不要：

```text
SpawnPoint
├─ character=knight
```

---

推荐：

```text
spawn_group=enemy_wave_1
```

---

然后：

```rust
EncounterConfig
```

决定刷什么。

---

否则平衡策划会天天改地图。

---

# 10. Trigger不要写逻辑

很多人：

```text
Trigger
├─ script=spawn_boss
```

---

后期：

```text
500个Trigger
```

全是脚本字符串。

地狱。

---

推荐：

```text
event_id=boss_intro
```

---

然后：

```rust
EventDatabase
```

统一管理。

---

# 11. Tiled不要成为真相源

这是最大的坑。

---

错误：

```text
Tiled
=
游戏数据
```

---

正确：

```text
Config
+
Tiled
+
Content Pipeline
=
游戏数据
```

---

例如：

Tiled：

```text
unit_id=orc_warrior
```

---

真正属性：

```rust
orc_warrior
```

来自：

```text
characters/orc_warrior.ron
```

---

不要把数值写进地图。

---

# 12. Importer才是核心资产

很多团队以为：

```text
Tiled最重要
```

其实不是。

---

真正值钱的是：

```text
TMX
 ↓
Importer
 ↓
MapAsset
```

这一层。

---

未来你可能：

```text
Tiled
换
LDtk
换
自研编辑器
```

---

只要：

```rust
MapAsset
```

不变。

全项目不动。

---

# 对于你的项目，我还会额外增加一个 ADR

```text
ADR-Map-Content-001

Tiled仅负责内容编辑。

禁止：
- Tile直接承载游戏数值
- Object直接映射ECS Entity
- Trigger直接执行脚本

要求：
- 所有Gameplay数据来自Config
- 所有地图对象拥有稳定GUID
- 所有地图经过Importer验证
- Runtime只读取MapAsset
```

这几个点的价值，实际上比「TMX还是JSON」「用不用bevy_ecs_tilemap」还高，因为它们决定的是 **未来十万行之后地图系统会不会崩掉**。
