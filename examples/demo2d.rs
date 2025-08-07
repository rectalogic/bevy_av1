use bevy::{prelude::*, window::WindowResolution};
use bevy_av1::{VideoPlayer, VideoPlugin, VideoSink};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(1718.0, 720.0),
                ..default()
            }),
            ..default()
        }),
        VideoPlugin,
    ))
    .add_systems(Startup, setup);

    app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(VideoPlayer::new(
            asset_server.load("av1/cosmos-laundromat.ivf"),
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
    commands.spawn(Camera2d);
}
