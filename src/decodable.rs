use std::io::Cursor;
use std::time::Duration;

use bevy::prelude::*;

use crate::{av1, video_source::VideoSource};

#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub image: Image,
    pub timestamp: Duration,
}

pub trait Decoder: Send {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn timebase(&self) -> (u32, u32);
    fn decode(
        &mut self,
        tx: async_channel::Sender<VideoFrame>,
        loop_: bool,
    ) -> impl Future<Output = Result<()>> + Send;
}

pub trait Decodable: Send + Sync + 'static {
    /// The type of the decoder of the video frames.
    type Decoder: Decoder + Send;

    /// Build and return a [`Self::Decoder`] of the implementing type
    fn decoder(&self) -> Self::Decoder;
}

impl Decodable for VideoSource {
    type Decoder = av1::Decoder<Cursor<VideoSource>>;

    fn decoder(&self) -> Self::Decoder {
        av1::Decoder::new(Cursor::new(self.clone())).unwrap()
    }
}
