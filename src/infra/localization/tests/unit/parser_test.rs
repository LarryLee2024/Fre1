//! 本地化解析的单元测试。
//!
//! parse_ftl 的测试：基本解析、空输入、注释、变量、伪区域设置。

use crate::infra::localization::io::parse_ftl;

#[test]
fn test_parse_basic_ftl() {
    let content = r#"
### Sample file
-core-yes = Yes
-core-no = No

-ability-abl_000042-name = Fireball
    .desc = Deals {$damage} fire damage
"#;
    let map = parse_ftl(content);
    assert_eq!(map.get("core.yes").unwrap().template, "Yes");
    assert!(map.get("core.yes").unwrap().variables.is_empty());
    assert_eq!(
        map.get("ability.abl_000042.name").unwrap().template,
        "Fireball"
    );
    assert_eq!(
        map.get("ability.abl_000042.name.desc").unwrap().template,
        "Deals {$damage} fire damage"
    );
    assert_eq!(
        map.get("ability.abl_000042.name.desc").unwrap().variables,
        vec!["damage"]
    );
}

#[test]
fn test_parse_empty_content() {
    let map = parse_ftl("");
    assert!(map.is_empty());
}

#[test]
fn test_parse_comments_only() {
    let map = parse_ftl("### Just a comment\n### Another comment\n");
    assert!(map.is_empty());
}

#[test]
fn test_parse_multiple_variables() {
    let content = r#"
-battle-damage_dealt = {$source} dealt {$damage} damage to {$target}
"#;
    let map = parse_ftl(content);
    let pattern = map.get("battle.damage_dealt").unwrap();
    assert_eq!(pattern.variables, vec!["source", "damage", "target"]);
}

#[test]
fn test_parse_zz_zz_fake_locale() {
    let content = r#"
-core-yes = [Yes]
-core-no = [No]
"#;
    let map = parse_ftl(content);
    assert_eq!(map.get("core.yes").unwrap().template, "[Yes]");
    assert_eq!(map.get("core.no").unwrap().template, "[No]");
}
