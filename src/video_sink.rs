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
    last_frame: Option<VideoFrame>,
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
            last_frame: None,
            start_timestamp: None,
        }
    }

    pub(crate) fn poll_task(&mut self) -> Option<Result<()>> {
        block_on(future::poll_once(&mut self.task))
    }

    pub(crate) fn next_frame(&mut self, current_time: Duration) -> Option<VideoFrame> {
        let start_timestamp = self.start_timestamp.get_or_insert(current_time);
        let elapsed = current_time - *start_timestamp;
        let timing_tolerance = self.frame_duration / 2;

        // First, check if we have a stored frame that's now appropriate
        if let Some(ref frame) = self.last_frame {
            if frame.timestamp <= elapsed + timing_tolerance
                && frame.timestamp + timing_tolerance >= elapsed
            {
                // This stored frame is now appropriate to display
                return self.last_frame.take();
            } else if frame.timestamp + timing_tolerance < elapsed {
                // Stored frame is now too old, discard it
                self.last_frame = None;
            } else {
                // Stored frame is still in the future, wait for it
                return None;
            }
        }

        // Process new frames from the channel
        while let Ok(frame) = self.rx.try_recv() {
            // Frame is too old - skip it
            if frame.timestamp + timing_tolerance < elapsed {
                continue;
            }

            // Frame is current - display it now
            if frame.timestamp <= elapsed + timing_tolerance {
                return Some(frame);
            }

            // Frame is in the future - store it for later
            if self.last_frame.is_none()
                || frame.timestamp < self.last_frame.as_ref().unwrap().timestamp
            {
                self.last_frame = Some(frame);
            }
        }

        // If we still have a stored frame and no current frame was found,
        // check if the stored frame is close enough to display
        if let Some(frame) = &self.last_frame {
            if frame.timestamp <= elapsed + self.frame_duration {
                return self.last_frame.take();
            }
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
