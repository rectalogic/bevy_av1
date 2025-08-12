use bevy::prelude::*;

use crate::{decodable::Decodable, video_source::VideoSource};

/// A component for playing a video.
///
/// Insert this component onto an entity to trigger a video source to begin playing.
///
/// If the handle refers to an unavailable asset (such as if it has not finished loading yet),
/// the video will not begin playing immediately. The video will play when the asset is ready.
///
/// When Bevy begins the video playback, a [`VideoSink`][crate::VideoSink] component will be
/// added to the entity. You can use that component to access the video dimensions and texture image.
///
#[derive(Component, Clone)]
pub struct VideoPlayer<Source = VideoSource>
where
    Source: Asset + Decodable,
{
    pub source: Handle<Source>,
    pub mode: PlaybackMode,
}

impl VideoPlayer<VideoSource> {
    /// Creates a new [`VideoPlayer`] with the given [`Handle<VideoSource>`].
    ///
    /// For convenience reasons, this hard-codes the [`VideoSource`] type. If you want to
    /// initialize an [`VideoPlayer`] with a different type, just initialize it directly using normal
    /// struct syntax.
    pub fn new(source: Handle<VideoSource>, mode: PlaybackMode) -> Self {
        Self { source, mode }
    }
}

/// The way Bevy manages the video playback.
#[derive(Debug, Copy, Clone)]
pub enum PlaybackMode {
    /// Repeat the video forever.
    Loop,
    /// Despawn the entity and its children when the video finishes playing.
    Despawn,
    /// Remove the video components from the entity, when the video finishes playing.
    Remove,
}
