//! Example implementing a custom video source [`Asset`].
//! The [`Decoder`] just synthesizes NTSC TV static video frames.

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_av1::{
    AddVideoSource, Decodable, Decoder, PlaybackMode, VideoFrame, VideoPlayer, VideoPlugin,
    VideoSink,
};
use rand::{SeedableRng, rngs::SmallRng, seq::IndexedRandom};
use std::time::Duration;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, VideoPlugin))
        .add_video_source::<CustomVideoSource>()
        .add_systems(Startup, setup);

    app.run();
}

#[derive(Asset, Debug, Clone, Reflect)]
struct CustomVideoSource {
    width: u32,
    height: u32,
}

impl CustomVideoSource {
    fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl Decodable for CustomVideoSource {
    type Decoder = CustomDecoder;

    fn decoder(&self) -> Self::Decoder {
        CustomDecoder {
            width: self.width,
            height: self.height,
        }
    }
}

struct CustomDecoder {
    width: u32,
    height: u32,
}

impl Decoder for CustomDecoder {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn timebase(&self) -> (u32, u32) {
        // 23.976 NTSC framerate
        (125, 2997)
    }

    async fn decode(
        &mut self,
        tx: bevy_av1::Sender<bevy_av1::VideoFrame>,
        loop_: bool,
    ) -> Result<()> {
        let timebase = self.timebase();
        let frame_duration = timebase.0 as f32 / timebase.1 as f32;

        const NTSC_BLACK_PIXEL: [u8; 4] = [16, 16, 16, 255];
        const NTSC_WHITE_PIXEL: [u8; 4] = [235, 235, 235, 255];
        const PIXEL_OPTIONS: [[u8; 4]; 2] = [NTSC_BLACK_PIXEL, NTSC_WHITE_PIXEL];
        let mut rng = SmallRng::seed_from_u64(1);

        // We generate random "static".
        // Hardcode a duration.
        const DURATION: u32 = 90;
        let mut count = 0u32;
        loop {
            let pixels: Vec<u8> = (0..self.width() * self.height())
                .flat_map(|_| *PIXEL_OPTIONS.choose(&mut rng).unwrap())
                .collect();
            let image = Image::new(
                Extent3d {
                    width: self.width(),
                    height: self.height(),
                    ..default()
                },
                TextureDimension::D2,
                pixels,
                TextureFormat::Bgra8UnormSrgb,
                RenderAssetUsages::default(),
            );

            let frame = VideoFrame {
                image,
                timestamp: Duration::from_secs_f32((count as f32) * frame_duration),
            };
            tx.send(frame).await?;

            count += 1;
            if count >= DURATION {
                if loop_ {
                    count = 0;
                } else {
                    break;
                }
            }
        }
        Ok(())
    }
}

fn setup(mut commands: Commands, mut custom_sources: ResMut<Assets<CustomVideoSource>>) {
    commands
        .spawn(VideoPlayer {
            source: custom_sources.add(CustomVideoSource::new(640, 480)),
            mode: PlaybackMode::Loop,
        })
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
