//! Example showing rendering video in a 2D [`Sprite`] component.

use bevy::{asset::io::web::WebAssetPlugin, prelude::*, window::WindowResolution};
use bevy_av1::{PlaybackMode, VideoPlayer, VideoPlugin, VideoSink};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(1718, 720),
                    ..default()
                }),
                ..default()
            })
            .set(WebAssetPlugin {
                silence_startup_warning: true,
            }),
        VideoPlugin,
    ))
    .add_systems(Startup, setup);

    app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Play a web based video via WebAssetPlugin
    commands
        .spawn(VideoPlayer::new(
            asset_server.load("https://github.com/rectalogic/bevy_av1/raw/refs/heads/main/assets/av1/cosmos-laundromat.ivf"),
            PlaybackMode::Remove,
        ))
        .observe(
            |add: On<Add, VideoSink>, sinks: Query<&VideoSink>, mut commands: Commands| {
                let entity = add.entity;
                if let Ok(sink) = sinks.get(entity) {
                    commands
                        .entity(entity)
                        .insert(Sprite::from_image(sink.image().clone()));
                }
            },
        );
    commands.spawn(Camera2d);
}
