use std::time::Duration;

use bevy::prelude::*;

/// A frame of video.
#[derive(Debug, Clone)]
pub struct VideoFrame {
    /// The video frame image.
    pub image: Image,
    /// The presentation timestamp of this frame.
    pub timestamp: Duration,
}

/// A type implementing this trait can decode frames of video.
///
pub trait Decoder: Send {
    /// The width of a video frame.
    fn width(&self) -> u32;
    /// The height of a video frame.
    fn height(&self) -> u32;
    /// The timebase of the decoded video `(numerator, denominator)`.
    /// For example, 30fps video could be `(1, 30)`.
    /// 23.976fps NTSC could be `(125, 2997)`.
    fn timebase(&self) -> (u32, u32);
    /// Asynchronously decode frames of video and send them through channel `tx`.
    /// If `loop_` is `true`, this function does not return unless there is an error.
    fn decode(
        &mut self,
        tx: async_channel::Sender<VideoFrame>,
        loop_: bool,
    ) -> impl Future<Output = Result<()>> + Send;
}

/// A type implementing this trait can be converted to a [`Self::Decoder`] type.
///
/// It must be [`Send`] and [`Sync`] in order to be registered.
/// Types that implement this trait usually contain raw video data that can be decoded into a series of [`crate::VideoFrame`].
/// This trait is implemented for [`VideoSource`].
pub trait Decodable: Send + Sync + 'static {
    /// The type of the decoder of the video frames.
    type Decoder: Decoder + Send;

    /// Build and return a [`Self::Decoder`] of the implementing type
    fn decoder(&self) -> Self::Decoder;
}
