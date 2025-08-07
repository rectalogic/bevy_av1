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
    .add_systems(Startup, setup)
    .add_systems(Update, update);

    app.run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((PointLight::default(), Transform::from_xyz(2.0, 2.0, 3.0)));
    commands.spawn((Camera3d::default(), Transform::from_xyz(0.0, 0.0, 2.0)));
    commands
        .spawn((
            VideoPlayer::new(asset_server.load("av1/cosmos-laundromat.ivf")),
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial::default())),
        ))
        .observe(
            |trigger: Trigger<OnAdd, VideoSink>,
             mut sinks: Query<(
                &VideoSink,
                &MeshMaterial3d<StandardMaterial>,
                &mut Transform,
            )>,
             mut materials: ResMut<Assets<StandardMaterial>>| {
                let entity = trigger.target();
                if let Ok((sink, mesh_material, mut transform)) = sinks.get_mut(entity) {
                    if let Some(material) = materials.get_mut(&mesh_material.0) {
                        material.base_color_texture = Some(sink.image().clone());
                        let aspect = sink.width() as f32 / sink.height() as f32;
                        if aspect > 1.0 {
                            transform.scale = Vec3::new(aspect, 1.0, 1.0);
                        } else {
                            transform.scale = Vec3::new(1.0, aspect, 1.0);
                        }
                    }
                }
            },
        );
}

fn update(
    mut videos: Query<(&mut Transform, &MeshMaterial3d<StandardMaterial>), With<VideoPlayer>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for (mut transform, mesh_material) in videos.iter_mut() {
        transform.rotate_x(time.delta_secs() * 0.8);
        // Workaround https://github.com/bevyengine/bevy/issues/20269
        materials.get_mut(&mesh_material.0);
    }
}
