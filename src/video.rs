use bevy::prelude::*;

use crate::{decodable::Decodable, video_source::VideoSource};

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

#[derive(Debug, Copy, Clone)]
pub enum PlaybackMode {
    /// Repeat the video forever.
    Loop,
    /// Despawn the entity and its children when the video finishes playing.
    Despawn,
    /// Remove the video components from the entity, when the video finishes playing.
    Remove,
}
