---
id: 01-architecture.asset-organization
title: Asset Organization
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
---

# Asset Organization — 美术资产架构与外包流程

Version: 1.0
Status: Proposed

本文档定义 SRPG 项目的美术资产组织架构，支持外包美术团队协作。

> **优化来源**: `docs/其他/38.md`

交叉引用：
- `docs/AI开发宪法完整版.md` — AI 开发宪法（最高约束力），本文档对应条款：1.1.2（定义与实例分离）、1.1.3（规则与内容分离）、12.1.4（统一 Asset Pipeline）、14.0.4（资源分类）

核心目标：让美术团队可以独立工作，不依赖程序团队即可提交资源。

---

## 核心原则

### 三树分离

> **优化来源**: `docs/其他/38.md` — 从根源规避"资源/配置/代码混放"的行业常见问题，完美支撑"美术独立工作、不依赖程序"的核心目标。

```
assets/      → 二进制美术资源（Sprite、Audio、Shader、Font...）
content/     → 游戏配置数据（RON 文件）
src/         → Rust 游戏逻辑
```

🟥 **绝对禁止**：把配置数据和美术资源混在同一目录。

### Content Packs（内容包）

> **优化来源**: `docs/其他/38.md` — 资源按"功能包"而非"文件类型"组织，一个技能的所有相关资源物理上相邻，降低认知负荷和误删风险。

> **⚠️ 宪法合规说明**：Content Packs 将二进制美术资源（icon.png）与配置数据（config.ron）放在同一目录下，与本文档"三树分离"原则（assets/ → 二进制、content/ → 配置）存在张力。此设计是**开发期便捷性与架构规范性的权衡**：美术团队需要在一个目录中看到技能的全部资源。运行时通过 Asset Pipeline 的路径映射保持逻辑分离（见"与命名空间协议的映射"一节）。若严格遵循宪法 §1.1.2（定义与实例分离），应将 config.ron 独立到 `content/` 目录，美术资源保留在 `assets/` 目录。

资源按**功能单元**组织，而非按文件类型：

```text
content_packs/
├── base/                          # 基础游戏内容包
│   ├── skills/
│   │   ├── fireball/              # 一个技能 = 一个目录
│   │   │   ├── icon.png           # 技能图标
│   │   │   ├── config.ron         # 技能配置（伤害、范围、CD）
│   │   │   └── vfx/              # 技能特效
│   │   │       └── fireball_effect.png
│   │   └── heal/
│   │       ├── icon.png
│   │       └── config.ron
│   ├── buffs/
│   │   └── flame_aura/
│   │       ├── icon.png
│   │       └── config.ron
│   └── items/
│       └── health_potion/
│           ├── icon.png
│           └── config.ron
├── _shared/                       # 运行时共享资源（被多个功能包引用）
│   ├── common_icons/              # 通用图标
│   └── shared_audio/              # 通用音效
├── _templates/                    # 编辑器/工具链模板（不打包进发布版）
│   ├── new_skill/                 # 新技能模板
│   └── new_buff/                  # 新 Buff 模板
├── _wip/                          # Work In Progress - 不会被 AssetServer 加载
│   └── new_skill_test/
└── _deprecated/                   # 已废弃 - 保留用于回溯，但不参与构建
    └── old_fireball_v1/
```

**关键规则**：
- `_shared/`：运行时共享资源，发布时打包，可被多个功能包引用
- `_templates/`：编辑器模板，发布时不打包，防止"体积爆炸"
- `_wip/`：实验性资源安全区，AssetLoader 和 CI 脚本显式忽略
- `_deprecated/`：废弃资源保留区，参与 Git 历史但不参与构建

### 与命名空间协议的映射

目录结构直接对应 `AssetId { namespace, category, name }`：

```text
content_packs/base/skills/fireball/icon.png
  ↓ 映射为
AssetId { namespace: "base", category: "skills", name: "fireball/icon" }
```

代码中永远不要硬编码 `content_packs/` 前缀，必须通过 AssetResolver 或别名访问。

### 外包友好原则

1. **一个角色 = 一个目录**：所有角色相关资源在同一个目录
2. **命名规范统一**：所有文件名使用 `snake_case`
3. **版本独立**：美术可以独立提交，不影响代码构建
4. **格式规范**：使用标准格式，便于工具链处理

---

## 完整资产目录

详见 `docs/01-architecture/00-overview/project-structure.md` 中"二、assets/ — 运行时资源树"部分。

本节重点描述外包工作流和美术规范。

---

## 角色资源规范

### 每个角色的标准目录

```
assets/art/characters/{character_name}/
├── sprite/           # 精灵图
│   ├── idle.png       # 待机动画
│   ├── move.png       # 移动动画
│   ├── attack.png     # 攻击动画
│   ├── skill.png      # 技能动画
│   ├── hurt.png       # 受伤动画
│   └── dead.png       # 死亡动画
├── animation/        # 动画数据
│   ├── idle.anim      # Bevy 动画配置
│   └── attack.anim
├── portrait/         # 头像
│   ├── normal.png     # 常规头像
│   └── damaged.png    # 受伤头像
├── avatar/           # 缩略图
│   └── small.png      # UI 缩略图
└── vfx/              # 角色专属特效
    └── aura.png       # 光环特效
```

### 命名规范

> **优化来源**: `docs/其他/38.md` — snake_case、单数名词、禁止空格/特殊字符是自动化脚本、CI/CD 验证、跨平台兼容性的生命线。

| 类别 | 格式 | 示例 |
|------|------|------|
| 精灵图 | `{action}.png` | `attack.png`, `move.png` |
| 动画数据 | `{action}.anim` | `attack.anim` |
| 头像 | `{state}.png` | `normal.png`, `damaged.png` |
| 特效 | `{effect_name}.png` | `aura.png` |

**通用命名规则**：

| 规则 | ✅ 正确 | ❌ 错误 |
|------|---------|---------|
| snake_case | `fire_ball.png` | `FireBall.png`、`fire-ball.png` |
| 单数名词 | `skill/` | `skills/`（目录用单数） |
| 无空格 | `hero_portrait.png` | `hero portrait.png` |
| 无特殊字符 | `ice_sword.png` | `ice-sword!.png` |
| 伴随文件同名 | `fireball/icon.png` + `fireball/config.ron` | `fireball/icon.png` + `fireball_config.ron` |

**元数据伴随文件规范**：
- 伴随文件必须同名同目录
- ✅ `skills/fireball/icon.png` + `skills/fireball/config.ron`
- ❌ `skills/fireball/icon.png` + `configs/skills/fireball.ron`（物理分离）

---

## 地图资源规范

### 战棋地图标准结构

```
assets/art/maps/battle_maps/{map_name}/
├── terrain.png           # 地形瓦片图集
├── objects.png            # 地图物件图集
├── background.png         # 背景图
├── foreground.png         # 前景装饰
└── minimap.png            # 小地图缩略图
```

### 地图数据与地图美术分离

```
assets/art/maps/          # 美术资源（Sprite、图集）
content/stages/           # 游戏数据（关卡配置、敌人配置、胜负条件）
```

🟥 **禁止** 把敌人配置放在美术目录中。

---

## 音频资源规范

### 音频文件格式

| 类别 | 格式 | 说明 |
|------|------|------|
| BGM | `.ogg` | 循环背景音乐 |
| SFX | `.wav` | 短音效 |
| Voice | `.ogg` | 语音（如有） |
| Ambience | `.ogg` | 环境音 |

### 音频命名规范

```
sfx_{category}_{description}_{variant}.wav

示例：
sfx_battle_hit_normal_01.wav
sfx_battle_hit_critical_01.wav
sfx_skill_fireball_cast_01.wav
sfx_skill_fireball_impact_01.wav
sfx_ui_button_click_01.wav
sfx_ui_menu_open_01.wav
sfx_item_equip_01.wav
sfx_buff_apply_poison_01.wav
```

---

## 外包美术工作流

### Git LFS 配置

```gitattributes
# 美术资源使用 Git LFS
*.png filter=lfs diff=lfs merge=lfs -text
*.jpg filter=lfs diff=lfs merge=lfs -text
*.wav filter=lfs diff=lfs merge=lfs -text
*.ogg filter=lfs diff=lfs merge=lfs -text
*.mp3 filter=lfs diff=lfs merge=lfs -text
*.ttf filter=lfs diff=lfs merge=lfs -text
*.otf filter=lfs diff=lfs merge=lfs -text
```

### 美术提交规范

```
外包美术只能修改以下目录：
- assets/art/
- assets/audio/
- assets/ui/
- assets/fonts/

绝对禁止修改：
- src/
- content/
- assets/definitions/
- assets/rules/
- assets/maps/（地图数据，非地图图片）
```

### 美术团队分支策略

```
main                        # 程序主分支
├── art/{artist_name}       # 各美术人员的分支
│   ├── art/alice/          # Alice 的分支
│   ├── art/bob/            # Bob 的分支
│   └── art/carol/          # Carol 的分支
```

### 美术资源审批流程

```
美术提交 PR
    ↓
scripts/asset_pipeline/validate_assets.py  # 自动校验
    ↓
格式检查通过？
    ├── 否 → 通知美术修改
    └── 是 → 代码审查
              ↓
         合并到 main
```

### 自动校验内容

> **优化来源**: `docs/其他/38.md` — 增强校验维度，包括 2 的幂次方、Alpha 通道完整性、资源引用完整性。

```python
# scripts/asset_pipeline/validate_assets.py 校验内容
1. 文件格式合规（PNG、OGG、WAV）
2. 图片尺寸合规（2的幂次方、最大尺寸限制 4096x4096）
3. 文件大小合规（单文件不超过 5MB）
4. 命名规范合规（snake_case、单数名词、无空格/特殊字符）
5. 目录结构合规（按角色/类型组织）
6. 无多余文件（临时文件、PSD 等）
7. Alpha 通道完整性（Sprite 必须使用 Alpha 通道）
8. 色彩空间合规（sRGB）
```

### 资源引用完整性校验

> **优化来源**: `docs/其他/38.md` — 自动化保障资源引用完整性，防止"孤立资源"和"断链配置"。

```python
# scripts/asset_pipeline/validate_references.py
1. 扫描所有 content_packs/**/*.ron，提取引用的 AssetId
2. 反向验证每个 AssetId 是否在文件系统中存在
3. 扫描所有资源文件，检查是否被至少一个 .ron 引用（排除 _shared）
4. 在 Pre-commit Hook 或 CI 中强制执行，不合格则阻止提交
```

### 校验失败反馈机制

校验失败时，报错信息必须包含：`文件路径 + 违规原因 + 修正建议`

```text
❌ VALIDATION FAILED: assets/art/characters/knight/sprite/attack.PNG
   原因: 文件名包含大写字母
   建议: 重命名为 attack.png（snake_case，全小写）
```

---

## 资源管线架构

### 构建管线

```
原始资源（PNG、WAV、...）
    ↓  [Asset Pipeline]
优化资源（压缩、打包、图集生成）
    ↓  [Bevy AssetServer]
运行时加载
```

### 资源管线脚本

```
scripts/asset_pipeline/
├── validate_assets.py     # 资源校验
├── sprite_pipeline.py     # Sprite 管线（裁剪、图集生成）
├── audio_pipeline.py      # 音频管线（格式转换、音量标准化）
├── localization.py        # 本地化管线
└── data_generation.py     # 数据生成（从 Excel 生成 RON）
```

---

## 多语言资源架构

### 本地化文件组织

```
assets/localization/
├── en/                    # 英文
│   ├── ui.ron             # 界面文本
│   ├── skills.ron         # 技能描述
│   ├── buffs.ron          # Buff 描述
│   ├── items.ron          # 物品描述
│   ├── dialogue.ron       # 对话文本
│   ├── quest.ron          # 任务描述
│   └── tutorial.ron       # 教程文本
├── zh_cn/                 # 简体中文
│   └── ...
├── zh_tw/                 # 繁体中文
│   └── ...
├── ja/                    # 日文
│   └── ...
└── ko/                    # 韩文
    └── ...
```

### 本地化数据格式

```ron
// assets/localization/zh_cn/skills.ron
{
    "fireball": "火球术",
    "heal": "治疗",
    "bash": "猛击",
    // ...
}
```

---

## 皮肤/主题架构

### UI 主题支持

```
assets/ui/themes/
├── default/               # 默认主题
│   ├── colors.ron         # 颜色定义
│   ├── sizes.ron          # 尺寸定义
│   └── styles.ron         # 样式定义
├── dark/                   # 暗色主题
└── classic/                # 经典主题
```

---

## 禁止事项

> **优化来源**: `docs/其他/38.md`

- 🟥 把游戏配置数据放在 `assets/` 目录
- 🟥 把 Rust 代码放在 `assets/` 目录
- 🟥 把开发脚本放在 `assets/` 目录
- 🟥 美术团队修改 `src/` 或 `content/` 目录
- 🟥 使用非标准格式（如 PSD、AI 放入 assets）
- 🟥 在代码中硬编码 `content_packs/` 前缀
- 🟩 使用 Git LFS 管理所有二进制资源
- 🟩 使用自动化脚本校验所有资源提交
- 🟩 新资源必须先在 `_wip/` 中测试，通过后再移入正式目录

---

## 新资源添加 Checklist

> **优化来源**: `docs/其他/38.md`

- [ ] 是否放入正确的 content_pack？
- [ ] 命名是否符合 snake_case + 单数？
- [ ] 是否有伴随的 .ron 配置？
- [ ] 是否在 `_wip/` 中测试过再移入正式目录？
- [ ] 是否通过了 Data Validator？
- [ ] 文件大小是否在限制范围内？
- [ ] 图片尺寸是否为 2 的幂次方？
- [ ] 是否使用了 Alpha 通道（Sprite 类）？