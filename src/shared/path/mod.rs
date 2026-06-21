//! 路径工具
//!
//! 零业务语义、零框架依赖的纯文件系统路径操作工具。
//! 基于 std::path 和 std::env 实现，无第三方 crate 依赖。
//!
//! # 设计原则
//!
//! - 平台感知：Unix 上遵循 XDG Base Directory，Windows 上使用已知文件夹
//! - 环境变量可覆写：`FRE_CONFIG_DIR`、`FRE_DATA_DIR`、`FRE_ASSETS_DIR`
//! - 纯函数设计：无全局可变状态，所有函数纯净且可测试
//!
//! # 核心类型
//!
//! | 类型 | 用途 |
//! |------|------|
//! | [`ProjectDirs`] | 项目标准目录定位器 |
//!
//! # 核心函数
//!
//! | 函数 | 用途 |
//! |------|------|
//! | [`ensure_dir`] | 创建目录（如不存在） |
//! | [`asset_path`] | 解析资产文件路径 |
//! | [`config_path`] | 解析配置文件路径 |

use std::path::{Path, PathBuf};

/// 项目标准目录定位器。
///
/// 遵循 XDG Base Directory 规范（Unix）确定配置文件和数据文件的标准路径。
/// 可通过环境变量 `FRE_CONFIG_DIR` 和 `FRE_DATA_DIR` 覆写。
///
/// # 环境变量覆写
///
/// | 环境变量 | 影响方法 | 说明 |
/// |----------|----------|------|
/// | `FRE_CONFIG_DIR` | [`config_dir()`] | 完全取代配置目录计算逻辑 |
/// | `FRE_DATA_DIR` | [`data_dir()`] | 完全取代数据目录计算逻辑 |
///
/// [`config_dir()`]: Self::config_dir
/// [`data_dir()`]: Self::data_dir
///
/// # 示例
///
/// ```ignore
/// let dirs = ProjectDirs::new("fre");
/// let config = dirs.config_dir();
/// let data = dirs.data_dir();
/// ```
#[derive(Debug, Clone)]
pub struct ProjectDirs {
    /// 项目名称（用于标准路径中的子目录名）
    project_name: String,
}

impl ProjectDirs {
    /// 创建项目标准目录定位器。
    ///
    /// `project_name` 将作为子目录名追加到标准路径末尾。
    /// 例如 `ProjectDirs::new("fre")` 会生成 `~/.config/fre` 这样的路径。
    pub fn new(project_name: impl Into<String>) -> Self {
        Self {
            project_name: project_name.into(),
        }
    }

    /// 返回项目配置目录。
    ///
    /// 优先级（从高到低）：
    /// 1. `FRE_CONFIG_DIR` 环境变量
    /// 2. Unix: `$XDG_CONFIG_HOME/<project>` 或 `$HOME/.config/<project>`
    /// 3. Windows: `%APPDATA%/<project>`
    /// 4. 回退: `./config/<project>`
    pub fn config_dir(&self) -> PathBuf {
        // 最高优先级：环境变量显式指定
        if let Ok(dir) = std::env::var("FRE_CONFIG_DIR") {
            return PathBuf::from(dir);
        }

        // Unix: XDG_CONFIG_HOME
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            return PathBuf::from(xdg).join(&self.project_name);
        }

        // Unix: $HOME/.config
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(".config").join(&self.project_name);
        }

        // Windows: %APPDATA%
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join(&self.project_name);
        }

        // 最终回退：当前工作目录下的 config/<project>
        PathBuf::from("config").join(&self.project_name)
    }

    /// 返回项目数据目录。
    ///
    /// 优先级（从高到低）：
    /// 1. `FRE_DATA_DIR` 环境变量
    /// 2. Unix: `$XDG_DATA_HOME/<project>` 或 `$HOME/.local/share/<project>`
    /// 3. Windows: `%LOCALAPPDATA%/<project>`
    /// 4. 回退: `./data/<project>`
    pub fn data_dir(&self) -> PathBuf {
        // 最高优先级：环境变量显式指定
        if let Ok(dir) = std::env::var("FRE_DATA_DIR") {
            return PathBuf::from(dir);
        }

        // Unix: XDG_DATA_HOME
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            return PathBuf::from(xdg).join(&self.project_name);
        }

        // Unix: $HOME/.local/share
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join(".local")
                .join("share")
                .join(&self.project_name);
        }

        // Windows: %LOCALAPPDATA%
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            return PathBuf::from(localappdata).join(&self.project_name);
        }

        // 最终回退：当前工作目录下的 data/<project>
        PathBuf::from("data").join(&self.project_name)
    }

    /// 返回项目名称。
    pub fn project_name(&self) -> &str {
        &self.project_name
    }
}

/// 创建目录及其所有父目录（如果不存在）。
///
/// 内部调用 `std::fs::create_dir_all`。如果目录已存在，返回 `Ok(())`（幂等）。
///
/// # 错误
///
/// 返回 `std::io::Error`，可能原因包括：
/// - 权限不足
/// - 路径包含无效字符
/// - 路径中某部分不是目录
///
/// # 示例
///
/// ```ignore
/// ensure_dir("/tmp/my_app/data")?;
/// ```
pub fn ensure_dir(path: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(path.as_ref())
}

/// 返回相对于资产目录的路径。
///
/// 资产目录基准路径通过 `FRE_ASSETS_DIR` 环境变量配置，
/// 未设置时默认为当前工作目录下的 `assets/`。
///
/// 该路径不会验证文件是否存在——它只是路径拼接工具。
///
/// # 示例
///
/// ```ignore
/// let path = asset_path("textures/icon.png");
/// assert!(path.ends_with("assets/textures/icon.png"));
/// ```
pub fn asset_path(relative: &str) -> PathBuf {
    let assets_root = std::env::var("FRE_ASSETS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("assets"));
    assets_root.join(relative)
}

/// 返回相对于项目配置目录的路径。
///
/// 使用 `ProjectDirs::new("fre")` 的配置目录作为基准路径。
/// 等同于 `ProjectDirs::new("fre").config_dir().join(relative)`。
///
/// 该路径不会验证文件是否存在——它只是路径拼接工具。
///
/// # 示例
///
/// ```ignore
/// let path = config_path("settings.ron");
/// assert!(path.ends_with("fre/settings.ron"));
/// ```
pub fn config_path(relative: &str) -> PathBuf {
    ProjectDirs::new("fre").config_dir().join(relative)
}

#[cfg(test)]
mod tests;
