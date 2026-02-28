use crate::{av1, decodable::Decodable};
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};
use std::io::Cursor;
use std::sync::Arc;

#[derive(Debug, Clone, Reflect)]
enum Format {
    Ivf,
    Mp4,
}

/// A source of AV1 video data in an IVF or MP4 container.
#[derive(Asset, Debug, Clone, Reflect)]
pub struct VideoSource {
    pub bytes: Arc<[u8]>,
    format: Format,
}

impl AsRef<[u8]> for VideoSource {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl Decodable for VideoSource {
    type Decoder = av1::Decoder<Cursor<VideoSource>>;

    fn decoder(&self) -> Result<Self::Decoder, BevyError> {
        let demuxer = match self.format {
            Format::Ivf => {
                av1::Demuxers::Ivf(av1::ivf::IvfDemuxer::new(Cursor::new(self.clone()))?)
            }
            Format::Mp4 => {
                av1::Demuxers::Mp4(av1::mp4::Mp4Demuxer::new(Cursor::new(self.clone()))?)
            }
        };
        Self::Decoder::new(demuxer)
    }
}

/// Loads IVF/MP4 files as [`VideoSource`] [`Assets`]
///
/// This asset loader supports the AV1 video codec in an IVF or MP4 container.
#[derive(Default, TypePath)]
pub struct VideoLoader;

impl AssetLoader for VideoLoader {
    type Asset = VideoSource;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(Self::Asset {
            bytes: bytes.into(),
            format: if load_context.path().get_full_extension().as_deref() == Some("mp4") {
                Format::Mp4
            } else {
                Format::Ivf
            },
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ivf", "mp4"]
    }
}

/// A trait that allows adding a custom video source to the object.
/// This is implemented for [`App`] to allow registering custom [`Decodable`] types.
pub trait AddVideoSource {
    /// Registers a video source.
    /// The type must implement [`super::Decodable`],
    /// so that it can be converted to a [`super::Decoder`] type,
    /// and [`Asset`], so that it can be registered as an asset.
    /// To use this method on [`App`],
    /// the [video][super::VideoPlugin] and [asset][AssetPlugin] plugins must be added first.
    fn add_video_source<T>(&mut self) -> &mut Self
    where
        T: Decodable + Asset;
}
