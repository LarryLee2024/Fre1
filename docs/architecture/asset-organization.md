# Asset Organization — 美术资产架构与外包流程

Version: 1.0
Status: Proposed

本文档定义 SRPG 项目的美术资产组织架构，支持外包美术团队协作。

核心目标：让美术团队可以独立工作，不依赖程序团队即可提交资源。

---

## 核心原则

### 三树分离

```
assets/      → 二进制美术资源（Sprite、Audio、Shader、Font...）
content/     → 游戏配置数据（RON 文件）
src/         → Rust 游戏逻辑
```

🟥 **绝对禁止**：把配置数据和美术资源混在同一目录。

### 外包友好原则

1. **一个角色 = 一个目录**：所有角色相关资源在同一个目录
2. **命名规范统一**：所有文件名使用 `snake_case`
3. **版本独立**：美术可以独立提交，不影响代码构建
4. **格式规范**：使用标准格式，便于工具链处理

---

## 完整资产目录

详见 `docs/architecture/project-structure.md` 中"二、assets/ — 运行时资源树"部分。

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

| 类别 | 格式 | 示例 |
|------|------|------|
| 精灵图 | `{action}.png` | `attack.png`, `move.png` |
| 动画数据 | `{action}.anim` | `attack.anim` |
| 头像 | `{state}.png` | `normal.png`, `damaged.png` |
| 特效 | `{effect_name}.png` | `aura.png` |

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

```python
# scripts/asset_pipeline/validate_assets.py 校验内容
1. 文件格式合规（PNG、OGG、WAV）
2. 图片尺寸合规（2的幂次方、最大尺寸限制）
3. 文件大小合规（不超过阈值）
4. 命名规范合规（snake_case）
5. 目录结构合规（按角色/类型组织）
6. 无多余文件（临时文件、PSD 等）
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

- 🟥 把游戏配置数据放在 `assets/` 目录
- 🟥 把 Rust 代码放在 `assets/` 目录
- 🟥 把开发脚本放在 `assets/` 目录
- 🟥 美术团队修改 `src/` 或 `content/` 目录
- 🟥 使用非标准格式（如 PSD、AI 放入 assets）
- 🟩 使用 Git LFS 管理所有二进制资源
- 🟩 使用自动化脚本校验所有资源提交