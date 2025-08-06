use bevy::{
    prelude::*,
    tasks::{Task, block_on, futures_lite::future},
};

use crate::video_source::VideoFrame;

#[derive(Component)]
pub struct DrainVideoSink;

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

    pub(crate) fn poll_task(&mut self) -> Option<Result<()>> {
        block_on(future::poll_once(&mut self.task))
    }

    pub(crate) fn poll_frame(&mut self) -> Option<VideoFrame> {
        self.rx.try_recv().ok()
    }

    pub fn image(&self) -> &Handle<Image> {
        &self.image
    }
}

// XXX add and implement VideoSinkPlayback trait with pause() etc.
