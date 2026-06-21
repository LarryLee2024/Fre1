use crate::shared::path::*;
use std::path::Path;

// ---------------------------------------------------------------------------
// 辅助：在测试范围内安全地设置/移除环境变量
// ---------------------------------------------------------------------------

/// 在作用域内设置环境变量，离开时自动恢复。
struct EnvGuard<'a> {
    key: &'a str,
    previous: Option<String>,
}

impl<'a> EnvGuard<'a> {
    fn set(key: &'a str, value: &str) -> Self {
        let previous = std::env::var(key).ok();
        // safety: 测试是单线程运行，不存在数据竞争
        unsafe { std::env::set_var(key, value) }
        Self { key, previous }
    }
}

impl Drop for EnvGuard<'_> {
    fn drop(&mut self) {
        match &self.previous {
            Some(val) => {
                // safety: 测试是单线程运行，不存在数据竞争
                unsafe { std::env::set_var(self.key, val) }
            }
            None => {
                // safety: 测试是单线程运行，不存在数据竞争
                unsafe { std::env::remove_var(self.key) }
            }
        }
    }
}

/// 确保环境变量不存在于作用域内。
struct EnvRemoveGuard<'a> {
    key: &'a str,
    previous: Option<String>,
}

impl<'a> EnvRemoveGuard<'a> {
    fn remove(key: &'a str) -> Self {
        let previous = std::env::var(key).ok();
        // safety: 测试是单线程运行，不存在数据竞争
        unsafe { std::env::remove_var(key) }
        Self { key, previous }
    }
}

impl Drop for EnvRemoveGuard<'_> {
    fn drop(&mut self) {
        if let Some(val) = &self.previous {
            // safety: 测试是单线程运行，不存在数据竞争
            unsafe { std::env::set_var(self.key, val) }
        }
    }
}

// ---------------------------------------------------------------------------
// ProjectDirs tests
// ---------------------------------------------------------------------------

#[test]
fn project_dirs_config_dir_ends_with_project_name() {
    let _guard = EnvRemoveGuard::remove("FRE_CONFIG_DIR");
    let dirs = ProjectDirs::new("mygame");
    let config = dirs.config_dir();
    assert!(
        config.ends_with("mygame"),
        "config_dir should end with project name, got: {:?}",
        config
    );
}

#[test]
fn project_dirs_data_dir_ends_with_project_name() {
    let _guard = EnvRemoveGuard::remove("FRE_DATA_DIR");
    let dirs = ProjectDirs::new("mygame");
    let data = dirs.data_dir();
    assert!(
        data.ends_with("mygame"),
        "data_dir should end with project name, got: {:?}",
        data
    );
}

#[test]
fn project_dirs_env_override_config_dir() {
    let _guard = EnvGuard::set("FRE_CONFIG_DIR", "/custom/config/path");
    let dirs = ProjectDirs::new("fre");
    let result = dirs.config_dir();
    assert_eq!(result, Path::new("/custom/config/path"));
}

#[test]
fn project_dirs_env_override_data_dir() {
    let _guard = EnvGuard::set("FRE_DATA_DIR", "/custom/data/path");
    let dirs = ProjectDirs::new("fre");
    let result = dirs.data_dir();
    assert_eq!(result, Path::new("/custom/data/path"));
}

#[test]
fn project_dirs_project_name_accessor() {
    let dirs = ProjectDirs::new("test_project");
    assert_eq!(dirs.project_name(), "test_project");
}

#[test]
fn project_dirs_clone_equality() {
    let a = ProjectDirs::new("game");
    let b = a.clone();
    assert_eq!(a.config_dir(), b.config_dir());
    assert_eq!(a.data_dir(), b.data_dir());
}

// ---------------------------------------------------------------------------
// ensure_dir tests
// ---------------------------------------------------------------------------

#[test]
fn ensure_dir_creates_directory() {
    let temp_base = std::env::temp_dir().join("fre_test_ensure_dir_create");
    let test_dir = temp_base.join("nested").join("sub").join("dir");

    // 清理旧数据
    let _ = std::fs::remove_dir_all(&temp_base);

    assert!(!test_dir.exists(), "test directory should not exist yet");
    ensure_dir(&test_dir).unwrap();
    assert!(
        test_dir.exists(),
        "test directory should exist after ensure_dir"
    );
    assert!(test_dir.is_dir(), "path should be a directory");

    // 清理
    let _ = std::fs::remove_dir_all(&temp_base);
}

#[test]
fn ensure_dir_is_idempotent() {
    let temp_base = std::env::temp_dir().join("fre_test_ensure_idempotent");
    let test_dir = temp_base.join("subdir");

    let _ = std::fs::remove_dir_all(&temp_base);

    // 第一次创建
    ensure_dir(&test_dir).unwrap();
    assert!(test_dir.exists());

    // 第二次调用不应报错
    ensure_dir(&test_dir).unwrap();
    assert!(test_dir.exists());

    let _ = std::fs::remove_dir_all(&temp_base);
}

#[test]
fn ensure_dir_returns_ok_for_existing_dir() {
    let temp_base = std::env::temp_dir().join("fre_test_ensure_existing");
    let test_dir = temp_base.join("existing");

    let _ = std::fs::remove_dir_all(&temp_base);
    std::fs::create_dir_all(&test_dir).unwrap();
    assert!(test_dir.exists());

    // 对已存在的目录调用 ensure_dir 应返回 Ok
    let result = ensure_dir(&test_dir);
    assert!(result.is_ok());

    let _ = std::fs::remove_dir_all(&temp_base);
}

// ---------------------------------------------------------------------------
// asset_path tests
// ---------------------------------------------------------------------------

#[test]
fn asset_path_default_uses_assets_dir() {
    let _guard = EnvRemoveGuard::remove("FRE_ASSETS_DIR");
    let path = asset_path("textures/icon.png");
    #[cfg(target_os = "windows")]
    assert!(path.ends_with("assets\\textures\\icon.png"));
    #[cfg(not(target_os = "windows"))]
    assert!(path.ends_with("assets/textures/icon.png"));
}

#[test]
fn asset_path_env_override() {
    let _guard = EnvGuard::set("FRE_ASSETS_DIR", "/my/custom/assets");
    let path = asset_path("ui/button.png");
    assert_eq!(path, Path::new("/my/custom/assets/ui/button.png"));
}

#[test]
fn asset_path_empty_relative() {
    let _guard = EnvGuard::set("FRE_ASSETS_DIR", "/test/assets");
    let path = asset_path("");
    assert_eq!(path, Path::new("/test/assets"));
}

#[test]
fn asset_path_traverses_correctly() {
    let _guard = EnvGuard::set("FRE_ASSETS_DIR", "/base");
    let path = asset_path("../config/settings.ron");
    assert_eq!(path, Path::new("/base/../config/settings.ron"));
}

// ---------------------------------------------------------------------------
// config_path tests
// ---------------------------------------------------------------------------

#[test]
fn config_path_default_uses_fre_config_dir() {
    let _guard = EnvRemoveGuard::remove("FRE_CONFIG_DIR");
    let path = config_path("settings.ron");
    assert!(
        path.ends_with("settings.ron"),
        "config_path should end with the relative path, got: {:?}",
        path
    );
    // 默认回退路径是 config/fre/<relative>
    assert!(path.ends_with("fre/settings.ron") || path.ends_with("fre\\settings.ron"));
}

#[test]
fn config_path_env_override() {
    let _guard = EnvGuard::set("FRE_CONFIG_DIR", "/custom/fre");
    let path = config_path("keybindings.ron");
    assert_eq!(path, Path::new("/custom/fre/keybindings.ron"));
}

#[test]
fn config_path_empty_relative() {
    let _guard = EnvGuard::set("FRE_CONFIG_DIR", "/cfg");
    let path = config_path("");
    assert_eq!(path, Path::new("/cfg"));
}

// ---------------------------------------------------------------------------
// Edge case tests
// ---------------------------------------------------------------------------

#[test]
fn project_dirs_with_empty_name() {
    let _guard_config = EnvRemoveGuard::remove("FRE_CONFIG_DIR");
    let _guard_data = EnvRemoveGuard::remove("FRE_DATA_DIR");
    let dirs = ProjectDirs::new("");
    let config = dirs.config_dir();
    // 空项目名时，路径末尾不应有多余的子目录段
    let config_filename = config
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    // 当 project_name 为空时，最后一节应该是 .config 或 config
    assert!(!config_filename.is_empty());
}

#[test]
fn ensure_dir_with_current_dir() {
    let temp_base = std::env::temp_dir().join("fre_test_ensure_current");
    let _ = std::fs::remove_dir_all(&temp_base);
    std::fs::create_dir_all(&temp_base).unwrap();
    let original = std::env::current_dir().unwrap();
    assert!(std::env::set_current_dir(&temp_base).is_ok());

    let result = ensure_dir(".");
    assert!(result.is_ok());

    let _ = std::env::set_current_dir(original);
    let _ = std::fs::remove_dir_all(&temp_base);
}
