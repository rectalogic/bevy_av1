//! Example showing a two videos, each rendered to two materials -
//! a 2D [`ColorMaterial`] applied to a 2D PiP (picture-in-picture) mesh,
//! and a 3D [`StandardMaterial`] applied to a 3D mesh.

use bevy::prelude::*;
use bevy_av1::{
    PlaybackMode, VideoPlayer, VideoPlugin, VideoSink, VideoTargetApp, VideoTargetAssets,
};

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, VideoPlugin))
        .init_video_target_asset::<StandardMaterial>()
        .init_video_target_asset::<ColorMaterial>()
        .add_systems(Startup, setup)
        .add_systems(Update, update);

    app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((PointLight::default(), Transform::from_xyz(2.0, 2.0, 3.0)));
    commands.spawn((
        Camera {
            order: 0,
            ..default()
        },
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 3.5),
    ));
    commands.spawn((
        Camera {
            order: 1,
            ..default()
        },
        Camera2d,
    ));

    commands
        .spawn(VideoPlayer::new(
            asset_server.load("av1/tears-of-steel.ivf"),
            PlaybackMode::Loop,
        ))
        .observe(
            |trigger: Trigger<OnAdd, VideoSink>,
             commands: Commands,
             mut sinks: Query<&VideoSink>,
             meshes: ResMut<Assets<Mesh>>,
             standard_materials: ResMut<Assets<StandardMaterial>>,
             color_materials: ResMut<Assets<ColorMaterial>>,
             standard_material_video_targets: ResMut<VideoTargetAssets<StandardMaterial>>,
             color_material_video_targets: ResMut<VideoTargetAssets<ColorMaterial>>| {
                let entity = trigger.target();
                if let Ok(sink) = sinks.get_mut(entity) {
                    spawn_video_targets(
                        sink,
                        commands,
                        meshes,
                        standard_materials,
                        color_materials,
                        standard_material_video_targets,
                        color_material_video_targets,
                        -1.5,
                        300.0,
                    );
                }
            },
        );

    commands
        .spawn(VideoPlayer::new(
            asset_server.load("av1/cosmos-laundromat.ivf"),
            PlaybackMode::Remove,
        ))
        .observe(
            |trigger: Trigger<OnAdd, VideoSink>,
             commands: Commands,
             mut sinks: Query<&VideoSink>,
             meshes: ResMut<Assets<Mesh>>,
             standard_materials: ResMut<Assets<StandardMaterial>>,
             color_materials: ResMut<Assets<ColorMaterial>>,
             standard_material_video_targets: ResMut<VideoTargetAssets<StandardMaterial>>,
             color_material_video_targets: ResMut<VideoTargetAssets<ColorMaterial>>| {
                let entity = trigger.target();
                if let Ok(sink) = sinks.get_mut(entity) {
                    spawn_video_targets(
                        sink,
                        commands,
                        meshes,
                        standard_materials,
                        color_materials,
                        standard_material_video_targets,
                        color_material_video_targets,
                        1.5,
                        -300.0,
                    );
                }
            },
        );
}

#[allow(clippy::too_many_arguments)]
fn spawn_video_targets(
    sink: &VideoSink,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut standard_material_video_targets: ResMut<VideoTargetAssets<StandardMaterial>>,
    mut color_material_video_targets: ResMut<VideoTargetAssets<ColorMaterial>>,
    offset3d: f32,
    offset2d: f32,
) {
    let standard_material = standard_materials.add(StandardMaterial {
        base_color_texture: Some(sink.image().clone()),
        ..default()
    });
    standard_material_video_targets.add_target(sink, &standard_material);
    let aspect = sink.width() as f32 / sink.height() as f32;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(aspect.max(1.0), aspect.min(1.0), 1.0))),
        MeshMaterial3d(standard_material),
        Transform::from_xyz(offset3d, -offset3d / 2.0, 0.0),
    ));

    let color_material = color_materials.add(ColorMaterial {
        texture: Some(sink.image().clone()),
        ..default()
    });
    color_material_video_targets.add_target(sink, &color_material);
    commands.spawn((
        MeshMaterial2d(color_material),
        Mesh2d(meshes.add(Rectangle::new(sink.width() as f32, sink.height() as f32))),
        Transform::from_xyz(offset2d, offset2d / 2.0, 0.0).with_scale(Vec3::splat(0.3)),
    ));
}

fn update(
    mut transforms: Query<&mut Transform, With<MeshMaterial3d<StandardMaterial>>>,
    time: Res<Time>,
) {
    for mut transform in transforms.iter_mut() {
        transform.rotate_x(time.delta_secs() * 0.8);
    }
}
