---
id: 09-planning.map-content-pipeline-plan
title: Map 内容管线架构补充规划
status: completed
owner: architect
created: 2026-06-22
updated: 2026-06-22 (Phase 1.1 ADR-065 completed)
tags:
  - planning
  - map
  - content-pipeline
  - tiled
  - rendering
---

# Map 内容管线架构补充规划

> 基于 `docs/99-history/ai_ignore_this_dir/16地图.md` + `16tiled.md` 历史经验 + 项目现状评估

---

## 1. 现状评估

### 已存在（不改动）

| 模块 | 路径 | 状态 |
|------|------|------|
| Tactical Domain | `src/core/domains/tactical/` | ✅ GridPos, GridMap, TileData(packed), 寻路, 移动规则 |
| Terrain Domain | `src/core/domains/terrain/` | ✅ SurfaceType, TileProperties, Hazard, 地形效果 |
| ADR-022 | `docs/01-architecture/20-tactical-combat/ADR-022-grid-terrain-faction.md` | ✅ 网格/地形/阵营架构 |
| Tactical 领域规则 | `docs/02-domain/domains/tactical_domain.md` | ✅ stable |
| Terrain 领域规则 | `docs/02-domain/domains/terrain_domain.md` | ✅ stable |
| Tactical 数据 Schema | `docs/04-data/domains/tactical_schema.md` | ✅ 含 GridMap/TileData |
| Terrain 数据 Schema | `docs/04-data/domains/terrain_schema.md` | ✅ 含 SurfaceType/Hazard |
| Content Platform 架构 | `docs/03-content/` | ✅ L0-L3 完成，L4 已预留 MapDef 概念 |

### 缺口（需要补充）

| # | 缺口 | 说明 | 紧急性 |
|---|------|------|--------|
| 1 | **Map 内容管线架构 ADR** | Tiled 定位、TMX→MapAsset 管线设计、Object Layer/Property 一等公民、Tile 不承载 Gameplay 数值 | 🔴 高 |
| 2 | **MapDef 内容定义** | `docs/03-content/definitions/` 缺少 L4 World 层 MapDef 类型定义 | 🔴 高 |
| 3 | **MapAsset 运行格式** | TMX 转换后的运行时地图数据格式（terrain grid + objects + regions + navigation mask） | 🔴 高 |
| 4 | **TerrainDef 内容注册** | 已有 terrain RON 配置（旧原型），但当前 `src/content/` pipeline 未加载地形 | 🟡 中 |
| 5 | **TMX Importer** | Tiled 文件 → MapAsset 的构建时转换工具 | 🟡 中 |
| 6 | **MapRenderer 架构** | 替代当前 test_battle 占位彩块渲染，支持 Tile Sprite/AOE 高亮/移动范围 | 🟡 中 |
| 7 | **宪法条款** | 缺少 Tiled/Tile-Gameplay 分离/地图内容管线条款 | 🟡 中 |
| 8 | **地图渲染表现文档** | `docs/06-ui/` 缺少地图渲染相关设计 | 🟢 低 |

### 架构原则确认（来自历史文档 + 用户决策）

```
地图编辑器:   Tiled（外部，不自行开发）
编辑格式:     TMX（Tiled 原生格式）
运行时格式:   MapAsset（RON/JSON，Importer 生成）
地图渲染:     自研 MapRenderer（不使用 bevy_ecs_tilemap）
地图逻辑:     100% 自研（已存在 tactical/terrain domain）
寻路/AOE:    100% 自研（已存在 tactical domain）
Tile 数值:   Tile 只存 TerrainId，不承载 Gameplay 数值（来自 Config）
Object Layer: 一等公民，对象有稳定 GUID
地图验证:     Importer 阶段完成验证
高度:         TileData 已预留在 u8（无需额外改动）
Importer:    核心资产——编辑器可换，MapAsset 不可变
```

---

## 2. 行动方案

### Phase 1 — 架构决策（当前 sprint）

| 步骤 | 负责 Agent | 产出 | 依赖 | 估计文件 |
|------|-----------|------|------|---------|
| 1.1 | @architect | ADR: Map 内容管线架构（Tiled 定位/TMX→MapAsset/Object Layer/Propertiy 策略/Tile-Config 分离） | 无 | 1 | ✅ Done |
| 1.2 | @content-architect | MapDef 内容定义 + TerrainDef 内容注册方案 | ADR 完成后 | 2-3 | ✅ Done |
| 1.3 | @data-architect | MapAsset 数据 Schema + TMX → MapAsset 映射设计 | ADR 完成后 | 2 | ✅ Done |
| 1.4 | @presentation-architect | MapRenderer 架构设计（渲染层/高亮层/坐标转换） | ADR 完成后 | 2 | ✅ Done |

### Phase 1 进展
- 1.1 ✅ ADR-065 Map 内容管线架构
- 1.2 ✅ MapDef + TerrainDef 内容定义（docs/03-content/）
- 1.3 ✅ MapAsset Schema + Importer Schema（docs/04-data/）
- 1.4 ✅ MapRenderer 架构设计（docs/06-ui/）

### Phase 2 进展
- 2.1 ✅ TMX Importer 工具（src/infra/map/importer.rs）
- 2.2 ✅ TerrainDef 内容管线注册（src/content/ + assets/config/terrains/）
- 2.3 ✅ MapRenderer V1（src/infra/map/renderer/）
- 2.4 ✅ 地图基建接入 app_plugin（MapPlugin 注册）

### Phase 3 进展
- 3.1 ✅ 宪法已更新（P0 第 10-12 条 + §2.5 map 模块 + §21 第 25-29 条）
- 3.2 ✅ 架构 README/内容索引/数据索引 已在 Phase 1 各 Agent 完成时同步更新

### Phase 4 — 扩展（有具体需求时）

| 功能 | 触发条件 |
|------|---------|
| 导航数据烘焙（Importer 阶段生成 movement_mask/region） | 地图数量 > 5 或性能需求 |
| Fog of War 系统 | 战术需求明确 |
| 地图 Region/Zone 系统 | AI/Trigger 系统需要 |
| 动态地图（可破坏地形/地面效果） | 技能/法术系统需要 |
| World Map（世界地图切换） | 叙事系统需要 |

---

## 3. 不做的范围（Phase 4 再考虑）

- 自研地图编辑器（永远不做——Tiled 已足够）
- bevy_ecs_tilemap 集成（渲染自研）
- 运行时 TMX 加载（Importer 阶段转换）
- 无限地图/Chunk Streaming（SRPG 不需要）
- 多地图世界管理（World Map）
- 地图动态编辑（运行时改地形）

---

## 4. 与现有架构的关系

```
Tiled (TMX)
  │
  ▼  [Importer — 构建时]
MapAsset (RON)
  │
  ▼  [Content Pipeline — 启动时]
  ├──→ TerrainRegistry (地形定义)
  ├──→ MapRegistry (地图 Asset)
  │
  ▼  [OnEnter(Combat/TacticalMap)]
MapLoader → GridMap + TileEntityMap + MapRenderer
  │
  ├──→ Tactical Domain (寻路/移动/网格查询)
  ├──→ Terrain Domain (地形效果/通行性)
  └──→ MapRenderer (渲染)

已有模块（不改动）
  src/core/domains/tactical/     ← GridMap, GridPos, TileData, 寻路
  src/core/domains/terrain/      ← TerrainType, TileProperties, Hazard

新增模块
  src/infra/map/importer/        ← TMX → MapAsset 转换
  src/infra/map/asset.rs         ← MapAsset 类型定义
  src/infra/map/loader.rs        ← MapAsset → GridMap 加载
  src/infra/map/renderer/        ← MapRenderer（Tile Sprite 渲染）
  src/content/ (terrain bucket)  ← TerrainDef RON 加载
```

---

## 5. 需要更新的文档清单

| 文档 | 改动 | 时机 |
|------|------|------|
| `docs/01-architecture/README.md` | 新增 ADR 索引、Infra 模块表新增 map/ | Phase 1.1 |
| `docs/01-architecture/40-cross-cutting/ADR-065-map-pipeline.md` | 新建 | Phase 1.1 |
| `docs/03-content/definitions/README.md` | 新增 L4 World 索引 | Phase 1.2 |
| `docs/03-content/definitions/world/map-def.md` | 新建 MapDef | Phase 1.2 |
| `docs/03-content/definitions/vocabulary/terrain-def.md` | 新建 TerrainDef | Phase 1.2 |
| `docs/04-data/README.md` | 新增 MapAsset Schema 索引 | Phase 1.3 |
| `docs/04-data/infrastructure/map-asset-schema.md` | 新建 | Phase 1.3 |
| `docs/04-data/infrastructure/map-importer-schema.md` | 新建 | Phase 1.3 |
| `docs/06-ui/README.md` | 新增地图渲染相关索引 | Phase 1.4 |
| `docs/06-ui/04-data-flow/map-rendering.md` | 新建 MapRenderer 设计 | Phase 1.4 |
| `docs/00-governance/ai-constitution-complete.md` | 补充 Tiled/Tile-Config 分离条款 | Phase 3 |

---

## 6. 关键决策点

| 决策 | 推荐 | 理由 |
|------|------|------|
| Importer 时机 | 构建时（build.rs 或独立工具） | 运行时不需要 TMX 解析，MapAsset 是最终数据 |
| MapAsset 格式 | RON（与项目一致） | 可版本化、可 diff、可 Review |
| Tile→Config 映射 | Tile 存 TerrainId+高度+标记，数值查 Config | 符合 Definition/Instance 分离宪法原则 |
| Object 处理 | 转为 MapObject 清单（含 GUID/Class/Property） | 运行时生成 ECS Entity，Object 定义不可变 |
| 高度系统 | TileData 已预留 height: u8，Importer 填充 | 历史文档强烈建议预留 |
| Spawn 数据 | 地图存 spawn_group_id，不存具体单位 | 平衡性调整不改地图 |
| Map 与 Scenes | 一个 map_id 对应一个 TacticalMap/Combat 场景 | ADR-050 场景架构对接 |
