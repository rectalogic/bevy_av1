use bevy::prelude::*;

use crate::{
    systems::play_video_system,
    video_source::{AddVideoSource, Decodable, VideoLoader, VideoSource},
};

mod av1;
mod systems;
mod video;
mod video_sink;
mod video_source;

pub struct VideoPlugin;

impl Plugin for VideoPlugin {
    fn build(&self, app: &mut App) {
        app.add_video_source::<VideoSource>()
            .init_asset_loader::<VideoLoader>();
    }
}

impl AddVideoSource for App {
    fn add_video_source<T>(&mut self) -> &mut Self
    where
        T: Decodable + Asset,
    {
        self.init_asset::<T>()
            .add_systems(PostUpdate, play_video_system::<T>);
        self
    }
}
