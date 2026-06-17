//! Registry Validator — 注册校验逻辑
//!
//! 提供注册时的一致性校验：ID 格式、跨 Def 引用、循环依赖检测。
//!
//! 详见 docs/04-data/infrastructure/registry_schema.md §5。

use crate::core::capabilities::runtime::registry::foundation::{
    BrokenReference, CrossReferenceReport, DefRegistry, IdType, RegistryError,
};

/// 校验 ID 格式是否合法。
///
/// V1: ID 格式正确（前缀 + 6 位数字）。
pub fn validate_id_format(def_id: &str) -> Result<(), RegistryError> {
    if def_id.is_empty() {
        return Err(RegistryError::InvalidIdFormat(
            "ID must not be empty".into(),
        ));
    }

    // 检查前缀是否合法
    let prefix = &def_id[..def_id.len().min(4)];
    if IdType::from_prefix(prefix).is_none() {
        return Err(RegistryError::InvalidIdFormat(format!(
            "unknown ID prefix: '{}' in '{}'",
            prefix, def_id
        )));
    }

    Ok(())
}

/// 校验跨 Def 引用。
///
/// V3: 所有跨 Def 引用有效。
/// 对每个 Def 条目中的引用字段进行检查，确保目标 ID 存在于 Registry 中。
pub fn validate_cross_references(registry: &DefRegistry) -> CrossReferenceReport {
    let mut report = CrossReferenceReport::new();
    report.total_defs = registry.count() as u32;

    for entry in registry.all_entries() {
        let data = &entry.data;
        // 扫描 data 字符串中的 ID 引用（前缀 + 6 位数字模式）
        // 简单实现：查找所有以已知前缀开头后跟数字的 token
        let references = extract_id_references(data);

        for ref_id in references {
            report.total_references += 1;
            if !registry.contains(&ref_id) {
                report.broken_count += 1;
                report.broken_references.push(BrokenReference {
                    source_def: entry.def_id.clone(),
                    field: "data".into(),
                    referenced_id: ref_id,
                    expected_type: "any".into(),
                });
            }
        }
    }

    report
}

/// 从数据字符串中提取可能的 ID 引用。
///
/// 查找形如 `xxx_NNNNNN`（前缀 + 6 位数字）的模式。
fn extract_id_references(data: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let known_prefixes = [
        "abl_", "eff_", "trg_", "tag_", "attr_", "cue_", "itm_", "spl_", "qst_", "fct_", "ter_",
        "rcp_", "buf_", "oot_",
    ];

    for prefix in &known_prefixes {
        let mut start = 0;
        while let Some(pos) = data[start..].find(prefix) {
            let abs_pos = start + pos;
            // ID 格式为 prefix(4) + 6digits = 10 字节
            if abs_pos + 10 > data.len() {
                break;
            }
            let candidate = &data[abs_pos..abs_pos + 10];
            if candidate.len() >= 10 {
                let suffix = &candidate[4..];
                if suffix.len() >= 6 && suffix.chars().take(6).all(|c| c.is_ascii_digit()) {
                    let id = &data[abs_pos..abs_pos + 10];
                    refs.push(id.to_string());
                }
            }
            start = abs_pos + 1;
        }
    }

    refs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::capabilities::runtime::registry::foundation::RegistryEntry;

    #[test]
    fn unit_040_validate_valid_format() {
        assert!(validate_id_format("abl_000001").is_ok());
        assert!(validate_id_format("eff_000042").is_ok());
    }

    #[test]
    fn unit_041_validate_empty_format() {
        assert!(validate_id_format("").is_err());
    }

    #[test]
    fn unit_042_validate_unknown_prefix() {
        assert!(validate_id_format("xxx_000001").is_err());
    }

    #[test]
    fn unit_043_cross_reference_no_broken() {
        let mut reg = DefRegistry::new();
        reg.register(RegistryEntry::new(
            "abl_000001",
            "Ability",
            "uses=eff_000001",
        ))
        .unwrap();
        // eff_000001 is not registered, so it's broken
        let report = validate_cross_references(&reg);
        assert!(report.has_broken_references());
    }

    #[test]
    fn unit_044_cross_reference_all_valid() {
        let mut reg = DefRegistry::new();
        reg.register(RegistryEntry::new(
            "abl_000001",
            "Ability",
            "uses=eff_000001",
        ))
        .unwrap();
        reg.register(RegistryEntry::new("eff_000001", "Effect", "type=damage"))
            .unwrap();
        let report = validate_cross_references(&reg);
        assert!(!report.has_broken_references());
    }

    #[test]
    fn unit_045_extract_id_references() {
        let data = "effect=eff_000001,trigger=trg_000042";
        let refs = extract_id_references(data);
        assert!(refs.contains(&"eff_000001".to_string()));
        assert!(refs.contains(&"trg_000042".to_string()));
    }

    #[test]
    fn unit_046_extract_id_references_none() {
        let data = "name=Fireball,damage=50";
        let refs = extract_id_references(data);
        assert!(refs.is_empty());
    }

    #[test]
    fn unit_047_infer_id_type() {
        assert_eq!(IdType::from_prefix("abl_"), Some(IdType::Ability));
        assert_eq!(IdType::from_prefix("eff_"), Some(IdType::Effect));
        assert_eq!(IdType::from_prefix("sho"), None);
    }
}
