use crate::{
    PlaybackMode,
    decodable::{Decodable, Decoder},
    video::VideoPlayer,
    video_sink::{DrainVideoSink, VideoFrameUpdated, VideoSink},
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
    for (entity, player) in &query_nonplaying {
        let Some(video_source) = video_sources.get(&player.source) else {
            continue;
        };
        let mut decoder = video_source.decoder();
        let timebase = decoder.timebase();
        let width = decoder.width();
        let height = decoder.height();
        let image = Image::new_uninit(
            Extent3d {
                width,
                height,
                ..default()
            },
            TextureDimension::D2,
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        );
        let loop_ = matches!(player.mode, PlaybackMode::Loop);
        let (tx, rx) = async_channel::bounded(1); //XXX make configurable?
        let task = ComputeTaskPool::get().spawn(async move { decoder.decode(tx, loop_).await });
        let sink = VideoSink::new(images.add(image), timebase, width, height, rx, task);
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
    mut query_playing: Query<(
        Entity,
        &mut VideoSink,
        &VideoPlayer<Source>,
        Option<&DrainVideoSink>,
    )>,
    mut images: ResMut<Assets<Image>>,
    time: Res<Time>,
    mut commands: Commands,
    mut video_frame_events: EventWriter<VideoFrameUpdated>,
) {
    for (entity, mut sink, player, drain) in &mut query_playing {
        match sink.next_frame(time.elapsed()) {
            None => {
                // If draining and no more frames, tear down
                if drain.is_some() {
                    match player.mode {
                        PlaybackMode::Remove => {
                            commands
                                .entity(entity)
                                .remove::<(DrainVideoSink, VideoSink, VideoPlayer<Source>)>();
                        }
                        PlaybackMode::Despawn => {
                            commands.entity(entity).despawn();
                        }
                        PlaybackMode::Loop => {
                            commands.entity(entity).remove::<DrainVideoSink>();
                        }
                    }
                    continue;
                }
            }
            Some(frame) => {
                if let Some(image) = images.get_mut(sink.image()) {
                    *image = frame.image;
                    video_frame_events.write(VideoFrameUpdated(sink.image().id()));
                }
            }
        }
    }
}
