use bevy::prelude::*;

use crate::video_source::{Decodable, VideoSource};

//XXX add PlaybackSettings to control mode, paused etc.?

#[derive(Component, Clone)]
pub struct VideoPlayer<Source = VideoSource>(pub Handle<Source>)
where
    Source: Asset + Decodable;

impl VideoPlayer<VideoSource> {
    /// Creates a new [`VideoPlayer`] with the given [`Handle<VideoSource>`].
    ///
    /// For convenience reasons, this hard-codes the [`VideoSource`] type. If you want to
    /// initialize an [`VideoPlayer`] with a different type, just initialize it directly using normal
    /// tuple struct syntax.
    pub fn new(source: Handle<VideoSource>) -> Self {
        Self(source)
    }
}
