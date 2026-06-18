//! 通用 RON AssetLoader — 将 .ron 文件加载为 Bevy Asset
//!
//! 通过 Bevy 的 AssetLoader trait 实现，支持热重载。
//! 详见 ADR-047 §3

use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use std::marker::PhantomData;

use super::errors::ConfigError;

/// 通用 RON 资源加载器。
///
/// 可加载任何实现了 `serde::de::DeserializeOwned + Asset + TypePath` 的类型。
///
/// # 使用方式
///
/// ```rust,ignore
/// app.register_asset_loader(RonAssetLoader::<SpellDef>::new());
/// ```
#[derive(TypePath)]
pub struct RonAssetLoader<T> {
    _phantom: PhantomData<T>,
}

impl<T> RonAssetLoader<T> {
    /// 创建新的加载器实例。
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for RonAssetLoader<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> AssetLoader for RonAssetLoader<T>
where
    T: serde::de::DeserializeOwned + Asset + TypePath,
{
    type Asset = T;
    type Settings = ();
    type Error = ConfigError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|e| ConfigError::FileReadError {
                path: Default::default(),
                reason: e.to_string(),
            })?;

        let data: T = ron::de::from_bytes(&bytes).map_err(|e| ConfigError::DeserializeError {
            path: Default::default(),
            detail: e.to_string(),
        })?;

        Ok(data)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}
