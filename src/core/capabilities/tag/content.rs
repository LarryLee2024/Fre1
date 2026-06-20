//! 内容到 TagHierarchy 的桥接系统。
//!
//! 在内容加载完成后，遍历 LoadedTagDefs 并将所有 TagDefinition
//! 注册到 TagHierarchy Resource 中。
//!
//! 此模块位于 core/tag 以保持注册逻辑靠近 TagHierarchy 实现，
//! 职责是连接内容加载管线（Content Plugin）与运行时标签层级（TagHierarchy）。

use bevy::prelude::*;

use crate::content::LoadedTagDefs;
use crate::core::capabilities::tag::mechanism::TagHierarchy;

/// 从 LoadedTagDefs 读取标签定义并注册到 TagHierarchy。
///
/// 在内容加载完成后由 Update 调度中的条件触发执行。
/// 使用 `std::mem::take` 消耗 defs 和 errors，防止系统重跑时重复注册。
pub(crate) fn register_tags_from_content(
    mut hierarchy: ResMut<TagHierarchy>,
    mut loaded_tags: ResMut<LoadedTagDefs>,
    mut commands: Commands,
) {
    let defs = std::mem::take(&mut loaded_tags.defs);
    let _errors = std::mem::take(&mut loaded_tags.errors);

    if defs.is_empty() {
        return;
    }

    info!(target: "tag", 
        event = "tag.registration.start",
        count = defs.len(),
        "[Tag] 开始注册 {} 个标签定义",
        defs.len()
    );

    let mut success_count = 0u32;
    let mut error_count = 0u32;

    for def in defs {
        let id = def.id.clone();
        match hierarchy.register(def, &mut commands) {
            Ok(()) => {
                trace!(target: "tag", 
                    event = "tag.registration.item_ok",
                    tag_id = %id,
                    "[Tag] 标签 '{}' 注册成功",
                    id
                );
                success_count += 1;
            }
            Err(e) => {
                warn!(target: "tag", 
                    event = "tag.registration.item_error",
                    tag_id = %id,
                    error = %e,
                    "[Tag] 标签 '{}' 注册失败：{}",
                    id,
                    e
                );
                error_count += 1;
            }
        }
    }

    info!(target: "tag", 
        event = "tag.registration.complete",
        success = success_count,
        failed = error_count,
        "[Tag] 标签注册完成：{} 成功，{} 失败",
        success_count,
        error_count
    );
}
