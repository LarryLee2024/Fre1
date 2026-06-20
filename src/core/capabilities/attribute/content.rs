//! 内容到 AttributeRegistry 的桥接系统。
//!
//! 在内容加载完成后，遍历 LoadedAttributeDefs 并将所有 AttributeDefinition
//! 注册到 AttributeRegistry Resource 中。
//!
//! 此模块位于 core/attribute 以保持注册逻辑靠近 AttributeRegistry 实现，
//! 职责是连接内容加载管线（Content Plugin）与运行时属性注册表（AttributeRegistry）。

use bevy::prelude::*;

use crate::content::LoadedAttributeDefs;
use crate::core::capabilities::attribute::mechanism::AttributeRegistry;

/// 从 LoadedAttributeDefs 读取属性定义并注册到 AttributeRegistry。
///
/// 在内容加载完成后由 Update 调度中的条件触发执行。
/// 使用 `std::mem::take` 消耗 defs，防止系统重跑时重复注册。
pub(crate) fn register_attributes_from_content(
    mut registry: ResMut<AttributeRegistry>,
    mut loaded_attributes: ResMut<LoadedAttributeDefs>,
) {
    let defs = std::mem::take(&mut loaded_attributes.defs);
    let _errors = std::mem::take(&mut loaded_attributes.errors);

    if defs.is_empty() {
        return;
    }

    info!(target: "attribute", 
        event = "attribute.registration.start",
        count = defs.len(),
        "[Attribute] 开始注册 {} 个属性定义",
        defs.len()
    );

    let mut success_count = 0u32;
    let mut error_count = 0u32;

    for def in defs {
        let id = def.id.clone();
        match registry.register(def) {
            Ok(()) => {
                trace!(target: "attribute", 
                    event = "attribute.registration.item_ok",
                    attribute_id = %id,
                    "[Attribute] 属性 '{}' 注册成功",
                    id
                );
                success_count += 1;
            }
            Err(e) => {
                warn!(target: "attribute", 
                    event = "attribute.registration.item_error",
                    attribute_id = %id,
                    error = %e,
                    "[Attribute] 属性 '{}' 注册失败：{}",
                    id,
                    e
                );
                error_count += 1;
            }
        }
    }

    info!(target: "attribute", 
        event = "attribute.registration.complete",
        success = success_count,
        failed = error_count,
        "[Attribute] 属性注册完成：{} 成功，{} 失败",
        success_count,
        error_count
    );
}
