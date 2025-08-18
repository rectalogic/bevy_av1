/*!
Video support for the game engine Bevy.
Supports decoding [AV1](https://aomedia.org/av1-features/) video in an
[IVF](https://wiki.multimedia.cx/index.php/Duck_IVF) container.

Extensible to other formats by implementing [`Decodable`] + [`Asset`].

# Usage

Add [`VideoPlugin`] to your Bevy [`App`], then load a [`VideoSource`] asset and
spawn a [`VideoPlayer`] component.

```rust
# use bevy::prelude::*;
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(VideoPlayer::new(
            asset_server.load("av1/cosmos-laundromat.ivf"),
            PlaybackMode::Remove,
        ))
        .observe(
            |trigger: Trigger<OnAdd, VideoSink>,
             sinks: Query<&VideoSink>,
             mut commands: Commands| {
                let entity = trigger.target();
                if let Ok(sink) = sinks.get(entity) {
                    commands
                        .entity(entity)
                        .insert(Sprite::from_image(sink.image().clone()));
                }
            },
        );
}
```
 */

use bevy::prelude::*;

mod av1;
mod decodable;
mod systems;
mod video;
mod video_sink;
mod video_source;
pub use crate::{
    decodable::{Decodable, Decoder, VideoFrame},
    video::{PlaybackMode, VideoPlayer},
    video_sink::{VideoSink, VideoTargetAssets},
    video_source::{AddVideoSource, VideoSource},
};
use crate::{
    systems::{play_videos, poll_video_sinks, render_video_sinks},
    video_sink::VideoFrameUpdated,
    video_source::VideoLoader,
};
#[doc(no_inline)]
pub use async_channel::Sender;

/// Adds support for video playback to a Bevy Application
///
/// Insert an [`VideoPlayer`] onto your entities to play video.
pub struct VideoPlugin;

impl Plugin for VideoPlugin {
    fn build(&self, app: &mut App) {
        app.add_video_source::<VideoSource>()
            .init_asset_loader::<VideoLoader>()
            .add_event::<VideoFrameUpdated>()
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

/// Adds video-related methods to [`App`].
pub trait VideoTargetApp {
    /// Registers a target [`Asset`] type.
    /// This is an asset that uses the [`Image`] asset from [`VideoSink`].
    /// This must be called before using [`VideoTargetAssets::add_target`]
    fn init_video_target_asset<A: Asset>(&mut self) -> &mut Self;
}

impl VideoTargetApp for App {
    fn init_video_target_asset<A: Asset>(&mut self) -> &mut Self {
        self.init_resource::<VideoTargetAssets<A>>().add_systems(
            PostUpdate,
            (
                VideoTargetAssets::<A>::update_target_assets,
                VideoTargetAssets::<A>::remove_unused_image_target_assets,
                VideoTargetAssets::<A>::remove_unused_target_assets,
            ),
        );
        self
    }
}
