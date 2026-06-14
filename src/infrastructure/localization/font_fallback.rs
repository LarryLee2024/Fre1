//! FontFallbackChain：多语言字体回退链

use bevy::prelude::*;
use std::collections::HashMap;

use super::locale::Locale;

/// 字体回退链配置 Resource
#[derive(Resource, Debug)]
pub struct FontFallbackChain {
    /// 按语言优先级排列的字体列表
    chains: HashMap<Locale, Vec<Handle<Font>>>,
}

impl Default for FontFallbackChain {
    fn default() -> Self {
        Self {
            chains: HashMap::new(),
        }
    }
}

impl FontFallbackChain {
    /// 为指定语言获取字体列表（含回退）
    pub fn get_fonts(&self, locale: &Locale) -> Vec<Handle<Font>> {
        self.chains
            .get(locale)
            .or_else(|| self.chains.get(&Locale::EnUs))
            .cloned()
            .unwrap_or_default()
    }

    /// 注册语言的字体链
    pub fn register(&mut self, locale: Locale, fonts: Vec<Handle<Font>>) {
        self.chains.insert(locale, fonts);
    }

    /// 加载字体回退链（从 assets/fonts/ 目录）
    pub fn load_defaults(asset_server: &AssetServer) -> Self {
        let mut chain = Self::default();

        // 中文：使用 Arial Unicode（项目自带）
        let zh_fonts = vec![asset_server.load("fonts/Arial Unicode.ttf")];
        chain.register(Locale::ZhCn, zh_fonts);

        // 英文：使用 Arial Unicode
        let en_fonts = vec![asset_server.load("fonts/Arial Unicode.ttf")];
        chain.register(Locale::EnUs, en_fonts);

        chain
    }
}
