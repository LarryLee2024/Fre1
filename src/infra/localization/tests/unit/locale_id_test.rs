//! LocaleId enum tests — string conversion, display, equality.

use crate::infra::localization::foundation::LocaleId;

#[test]
fn test_locale_id_as_str() {
    assert_eq!(LocaleId::EnUS.as_str(), "en-US");
    assert_eq!(LocaleId::ZhCN.as_str(), "zh-CN");
    assert_eq!(LocaleId::JaJP.as_str(), "ja-JP");
    assert_eq!(LocaleId::ZzZZ.as_str(), "zz-ZZ");
}

#[test]
fn test_locale_id_display() {
    assert_eq!(format!("{}", LocaleId::EnUS), "en-US");
    assert_eq!(format!("{}", LocaleId::ZhCN), "zh-CN");
}

#[test]
fn test_locale_id_try_from_str() {
    assert_eq!(LocaleId::try_from("en-US").unwrap(), LocaleId::EnUS);
    assert_eq!(LocaleId::try_from("zh-CN").unwrap(), LocaleId::ZhCN);
    assert!(LocaleId::try_from("fr-FR").is_err());
}

#[test]
fn test_locale_id_equality() {
    assert_eq!(LocaleId::EnUS, LocaleId::EnUS);
    assert_ne!(LocaleId::EnUS, LocaleId::ZhCN);
}

#[test]
fn test_locale_id_clone() {
    let a = LocaleId::EnUS;
    let b = a.clone();
    assert_eq!(a, b);
}
