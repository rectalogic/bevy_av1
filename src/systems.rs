use crate::{
    video::VideoPlayer,
    video_sink::{DrainVideoSink, VideoSink},
    video_source::{Decodable, Decoder},
};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    tasks::ComputeTaskPool,
};

pub fn play_videos<Source: Asset + Decodable>(
    query_nonplaying: Query<(Entity, &VideoPlayer<Source>), Without<VideoSink>>,
    video_sources: Res<Assets<Source>>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    for (entity, source_handle) in &query_nonplaying {
        let Some(video_source) = video_sources.get(&source_handle.0) else {
            continue;
        };
        let mut decoder = video_source.decoder();
        let timebase = decoder.timebase();
        let image = Image::new_uninit(
            Extent3d {
                width: decoder.width(),
                height: decoder.height(),
                ..default()
            },
            TextureDimension::D2,
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        );
        let (tx, rx) = async_channel::bounded(2); //XXX make configurable?
        let task = ComputeTaskPool::get().spawn(async move { decoder.decode(tx).await });
        let sink = VideoSink::new(images.add(image), timebase, rx, task);
        commands.entity(entity).insert(sink);
    }
}

pub fn poll_video_sinks(
    mut query_playing: Query<(Entity, &mut VideoSink), Without<DrainVideoSink>>,
    mut commands: Commands,
) {
    for (entity, mut sink) in &mut query_playing {
        if let Some(result) = sink.poll_task() {
            if let Err(err) = result {
                warn!("Video decoding failed: {err}");
            }
            commands.entity(entity).insert(DrainVideoSink);
        }
    }
}

pub fn render_video_sinks<Source: Asset + Decodable>(
    mut query_playing: Query<(Entity, &mut VideoSink, Option<&DrainVideoSink>)>,
    mut images: ResMut<Assets<Image>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut sink, drain) in &mut query_playing {
        match sink.next_frame(time.elapsed()) {
            None => {
                // If draining and no more frames, tear down
                // XXX need a marker frame for final frame (or an enum containing frame), for proper draining and to handle looping
                if drain.is_some() {
                    commands
                        .entity(entity)
                        .remove::<(DrainVideoSink, VideoSink, VideoPlayer<Source>)>();
                    continue;
                }
            }
            Some(frame) => {
                if let Some(image) = images.get_mut(sink.image()) {
                    *image = frame.image;
                }
            }
        }
    }
}
