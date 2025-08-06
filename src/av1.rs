pub mod decoder;
pub mod ivf;
use std::fmt::Display;

pub use decoder::Decoder;

#[derive(Debug)]
pub enum Error {
    Demuxer(std::io::Error),
    ChannelClosed,
    Decoder(dav1d::Error),
    Conversion(yuv::YuvError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
