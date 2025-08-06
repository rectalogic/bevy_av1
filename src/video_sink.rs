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
    frame_duration: Duration,
    start_timestamp: Option<Duration>,
}

impl VideoSink {
    /// Create a new video sink.
    pub(crate) fn new(
        image: Handle<Image>,
        timebase: (u32, u32),
        rx: async_channel::Receiver<VideoFrame>,
        task: Task<Result<()>>,
    ) -> Self {
        Self {
            image,
            frame_duration: Duration::from_secs_f64(timebase.0 as f64 / timebase.1 as f64),
            rx,
            task,
            start_timestamp: None,
        }
    }

    pub(crate) fn poll_task(&mut self) -> Option<Result<()>> {
        block_on(future::poll_once(&mut self.task))
    }

    pub(crate) fn next_frame(&mut self, current_time: Duration) -> Option<VideoFrame> {
        let mut last_frame = None;
        loop {
            match self.rx.try_recv().ok() {
                None => return last_frame,
                Some(frame) => {
                    let start_timestamp = self.start_timestamp.get_or_insert(current_time);
                    let elapsed = current_time - *start_timestamp;
                    if frame.timestamp + self.frame_duration < elapsed {
                        last_frame = Some(frame);
                        continue;
                    }
                    return Some(frame);
                }
            }
        }
    }

    pub fn image(&self) -> &Handle<Image> {
        &self.image
    }
}

// XXX add and implement VideoSinkPlayback trait with pause() etc.
