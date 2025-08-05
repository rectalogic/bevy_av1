use bevy::{prelude::*, tasks::Task};

use crate::video_source::VideoFrame;

#[derive(Component)]
pub struct VideoSink {
    image: Handle<Image>,
    rx: async_channel::Receiver<VideoFrame>,
    task: Task<Result<()>>,
}

impl VideoSink {
    /// Create a new video sink.
    pub(crate) fn new(
        image: Handle<Image>,
        rx: async_channel::Receiver<VideoFrame>,
        task: Task<Result<()>>,
    ) -> Self {
        Self { image, rx, task }
    }

    pub fn image(&self) -> &Handle<Image> {
        &self.image
    }
}

// XXX add and implement VideoSinkPlayback trait with pause() etc.
