use std::time::Duration;

use bevy::{
    prelude::*,
    tasks::{Task, block_on, futures_lite::future},
};

use crate::decodable::VideoFrame;

#[derive(Component)]
pub struct DrainVideoSink;

/// Bevy inserts this component onto your entities when it begins playing a video source.
/// Use [`VideoPlayer`][crate::VideoPlayer] to trigger that to happen.
///
/// You can use this component to access the texture that renders video frames.
///
/// If this component is removed from an entity, and a [`VideoSource`][crate::VideoSource] is
/// attached to that entity, that [`VideoSource`][crate::VideoSource] will start playing. If
/// that source is unchanged, that translates to the video restarting.
#[derive(Component)]
pub struct VideoSink {
    image: Handle<Image>,
    rx: async_channel::Receiver<VideoFrame>,
    task: Task<Result<()>>,
    width: u32,
    height: u32,
    frame_duration: Duration,
    buffered_frame: Option<VideoFrame>,
    start_timestamp: Option<Duration>,
}

impl VideoSink {
    /// Create a new video sink.
    pub(crate) fn new(
        image: Handle<Image>,
        timebase: (u32, u32),
        width: u32,
        height: u32,
        rx: async_channel::Receiver<VideoFrame>,
        task: Task<Result<()>>,
    ) -> Self {
        Self {
            image,
            frame_duration: Duration::from_secs_f64(timebase.0 as f64 / timebase.1 as f64),
            rx,
            task,
            width,
            height,
            buffered_frame: None,
            start_timestamp: None,
        }
    }

    pub(crate) fn poll_task(&mut self) -> Option<Result<()>> {
        block_on(future::poll_once(&mut self.task))
    }

    fn fetch_frame(&mut self) -> Option<VideoFrame> {
        let frame = match self.buffered_frame.take() {
            Some(frame) => frame,
            None => self.rx.try_recv().ok()?,
        };
        // Support looping
        if frame.timestamp == Duration::ZERO {
            self.start_timestamp = None;
        }
        Some(frame)
    }

    pub(crate) fn next_frame(&mut self, current_time: Duration) -> Option<VideoFrame> {
        while let Some(frame) = self.fetch_frame() {
            let start_timestamp = self.start_timestamp.get_or_insert(current_time);
            let elapsed = current_time - *start_timestamp;

            // Frame in the future
            if frame.timestamp > elapsed + self.frame_duration {
                self.buffered_frame = Some(frame);
                return None;
            }
            // Frame too old, discard
            else if frame.timestamp + self.frame_duration < elapsed {
                continue;
            }
            // Frame is current
            return Some(frame);
        }
        None
    }

    /// Width of a video frame.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height of a video frame.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The texture handle that video frames are rendered into.
    /// Apply this to a material or sprite to make the video visible.
    pub fn image(&self) -> &Handle<Image> {
        &self.image
    }
}

#[derive(Event, Debug)]
pub struct VideoFrameUpdated(pub AssetId<Image>);

/// Stores target [`AssetId`]s of assets that have a dependency on the video [`Image`] asset.
///
/// e.g. if you store the [`VideoSink::image`] in `StandardMaterial::base_color_texture`
/// you can [`VideoTargetAssets::add_target`] to ensure the material is updated when the image updates.
#[derive(Resource)]
pub struct VideoTargetAssets<A: Asset>(Vec<TargetAsset<A>>);

impl<A: Asset> Default for VideoTargetAssets<A> {
    fn default() -> Self {
        Self(Vec::default())
    }
}

#[derive(Debug, Default)]
pub struct TargetAsset<A: Asset> {
    pub image_id: AssetId<Image>,
    pub target_asset_id: AssetId<A>,
}

impl<A: Asset> VideoTargetAssets<A> {
    /// Add a target asset that has a dependency on the video image.
    /// This ensures the asset is updated whenever the video image changes.
    pub fn add_target(&mut self, sink: &VideoSink, target_asset_id: impl Into<AssetId<A>>) {
        self.0.push(TargetAsset {
            image_id: sink.image().id(),
            target_asset_id: target_asset_id.into(),
        })
    }

    pub(crate) fn update_target_assets(
        video_target_assets: Res<Self>,
        mut target_assets: ResMut<Assets<A>>,
        mut video_frame_events: EventReader<VideoFrameUpdated>,
    ) {
        if video_target_assets.0.is_empty() {
            return;
        }
        for event in video_frame_events.read() {
            video_target_assets.0.iter().for_each(|a| {
                if a.image_id == event.0 {
                    target_assets.get_mut(a.target_asset_id);
                }
            });
        }
    }

    pub(crate) fn remove_unused_image_target_assets(
        mut video_target_assets: ResMut<Self>,
        mut image_events: EventReader<AssetEvent<Image>>,
    ) {
        for event in image_events.read() {
            if let AssetEvent::Unused { id: image_id } = event {
                video_target_assets.0.retain(|e| &e.image_id != image_id);
            }
        }
    }

    pub(crate) fn remove_unused_target_assets(
        mut video_target_assets: ResMut<Self>,
        mut target_asset_events: EventReader<AssetEvent<A>>,
    ) {
        for event in target_asset_events.read() {
            if let AssetEvent::Unused { id: asset_id } = event {
                video_target_assets
                    .0
                    .retain(|e| &e.target_asset_id != asset_id);
            }
        }
    }
}
