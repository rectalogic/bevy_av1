use crate::decodable::Decodable;
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};
use std::sync::Arc;

#[derive(Asset, Debug, Clone, TypePath)]
pub struct VideoSource {
    pub bytes: Arc<[u8]>,
}

impl AsRef<[u8]> for VideoSource {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

/// Loads files as [`VideoSource`] [`Assets`](bevy_asset::Assets)
///
/// This asset loader supports the AV1 video codec in an IVF container.
#[derive(Default)]
pub struct VideoLoader;

impl AssetLoader for VideoLoader {
    type Asset = VideoSource;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<VideoSource, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(VideoSource {
            bytes: bytes.into(),
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ivf"]
    }
}

/// A trait that allows adding a custom video source to the object.
/// This is implemented for [`App`][bevy_app::App] to allow registering custom [`Decodable`] types.
pub trait AddVideoSource {
    /// Registers a video source.
    /// The type must implement [`Decodable`],
    /// so that it can be converted to a [`Decoder`] type,
    /// and [`Asset`], so that it can be registered as an asset.
    /// To use this method on [`App`][bevy_app::App],
    /// the [video][super::VideoPlugin] and [asset][bevy_asset::AssetPlugin] plugins must be added first.
    fn add_video_source<T>(&mut self) -> &mut Self
    where
        T: Decodable + Asset;
}
