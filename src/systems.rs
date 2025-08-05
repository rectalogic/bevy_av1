use std::io::Cursor;

use crate::{
    video::VideoPlayer,
    video_sink::VideoSink,
    video_source::{Decodable, Decoder},
};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    tasks::ComputeTaskPool,
};

pub fn play_video_system<Source: Asset + Decodable>(
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
        let (tx, rx) = async_channel::bounded(2); //XXX what size channel? compare bevy framerate to decoder framerate?
        let task = ComputeTaskPool::get().spawn(async move { decoder.decode(tx).await });
        let image_handle = images.add(image);
        let sink = VideoSink::new(image_handle, rx, task);
        commands.entity(entity).insert(sink);
    }
}

//XXX add system to handle VideoSinks - poll the sink rx and copy frame into images
