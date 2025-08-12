use bevy::prelude::*;

use crate::{
    systems::{play_videos, poll_video_sinks, render_video_sinks},
    video_source::{AddVideoSource, VideoLoader},
};

mod av1;
mod systems;
mod video;
mod video_sink;
mod video_source;
pub use crate::{
    video::{PlaybackMode, VideoPlayer},
    video_sink::VideoSink,
    video_source::{Decodable, Decoder, VideoFrame, VideoSource},
};
#[doc(no_inline)]
pub use async_channel::Sender;

pub struct VideoPlugin;

impl Plugin for VideoPlugin {
    fn build(&self, app: &mut App) {
        app.add_video_source::<VideoSource>()
            .init_asset_loader::<VideoLoader>()
            .add_systems(Update, poll_video_sinks);
    }
}

impl AddVideoSource for App {
    fn add_video_source<T>(&mut self) -> &mut Self
    where
        T: Decodable + Asset,
    {
        self.init_asset::<T>()
            .add_systems(Update, (play_videos::<T>, render_video_sinks::<T>));
        self
    }
}
