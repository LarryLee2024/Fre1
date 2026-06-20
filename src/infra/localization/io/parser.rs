//! FTL 解析器 — 纯函数，零 ECS 依赖
//!
//! 解析 .ftl 内容为扁平的 key -> Pattern 映射。
//!
//! 详见 `docs/03-technical/localization-design.md` §2 + 附录 B

use std::collections::HashMap;

use regex::Regex;

use crate::infra::localization::foundation::Pattern;

/// 解析 .ftl 内容为扁平 key -> Pattern 映射
///
/// 支持:
/// - Message ID: `-xxx-yyy = value`
/// - Attribute: `.desc = value`（需跟在 message ID 之后）
/// - 变量提取: `{$var}`
/// - 注释: `###` 开头的行
pub fn parse_ftl(content: &str) -> HashMap<String, Pattern> {
    let mut result = HashMap::new();
    // 正则对象仅在本函数内部创建，避免全局 lazy_static
    let id_re = Regex::new(r"^-([a-zA-Z0-9_-]+)\s*=\s*(.*)$").unwrap();
    let attr_re = Regex::new(r"^\s+\.([a-zA-Z0-9_-]+)\s*=\s*(.*)$").unwrap();
    let var_re = Regex::new(r"\{\$([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();

    let mut current_id: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // 跳过注释行（以 ### 开头）和空行
        if trimmed.starts_with("###") || trimmed.is_empty() {
            continue;
        }

        // Message ID: -xxx-yyy = value
        if let Some(caps) = id_re.captures(trimmed) {
            let raw_id = caps.get(1).unwrap().as_str();
            let value = caps.get(2).unwrap().as_str().trim();
            let key = raw_id.replace('-', ".");

            let vars: Vec<String> = var_re
                .captures_iter(value)
                .map(|c| c[1].to_string())
                .collect();

            result.insert(
                key.clone(),
                Pattern {
                    template: value.to_string(),
                    variables: vars,
                },
            );

            current_id = Some(key);
        }
        // Attribute: .xxx = value
        else if let Some(caps) = attr_re.captures(line)
            && let Some(ref base_key) = current_id
        {
            let attr_name = caps.get(1).unwrap().as_str();
            let value = caps.get(2).unwrap().as_str().trim();
            let key = format!("{}.{}", base_key, attr_name);

            let vars: Vec<String> = var_re
                .captures_iter(value)
                .map(|c| c[1].to_string())
                .collect();

            result.insert(
                key,
                Pattern {
                    template: value.to_string(),
                    variables: vars,
                },
            );
        }
    }

    result
}
