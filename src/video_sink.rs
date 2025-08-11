use std::time::Duration;

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
    width: u32,
    height: u32,
    frame_duration: Duration,
    buffered_frame: Option<VideoFrame>,
    start_timestamp: Option<Duration>,
}

impl VideoSink {
    /// Create a new video sink.
    pub(crate) fn new(
        image: Handle<Image>,
        timebase: (u32, u32),
        width: u32,
        height: u32,
        rx: async_channel::Receiver<VideoFrame>,
        task: Task<Result<()>>,
    ) -> Self {
        Self {
            image,
            frame_duration: Duration::from_secs_f64(timebase.0 as f64 / timebase.1 as f64),
            rx,
            task,
            width,
            height,
            buffered_frame: None,
            start_timestamp: None,
        }
    }

    pub(crate) fn poll_task(&mut self) -> Option<Result<()>> {
        block_on(future::poll_once(&mut self.task))
    }

    fn fetch_frame(&mut self) -> Option<VideoFrame> {
        if self.buffered_frame.is_some() {
            return self.buffered_frame.take();
        }
        self.rx.try_recv().ok()
    }

    pub(crate) fn next_frame(&mut self, current_time: Duration) -> Option<VideoFrame> {
        let start_timestamp = self.start_timestamp.get_or_insert(current_time);
        let elapsed = current_time - *start_timestamp;
        while let Some(frame) = self.fetch_frame() {
            // Frame in the future
            if frame.timestamp > elapsed + self.frame_duration {
                self.buffered_frame = Some(frame);
                return None;
            }
            // Frame too old, discard
            else if frame.timestamp + self.frame_duration < elapsed {
                continue;
            }
            // Frame is current
            return Some(frame);
        }
        None
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn image(&self) -> &Handle<Image> {
        &self.image
    }
}

// XXX add and implement VideoSinkPlayback trait with pause() etc.
