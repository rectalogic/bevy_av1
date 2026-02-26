pub mod decoder;
pub mod ivf;
pub mod mp4;
use std::{
    fmt::Display,
    io::{Read, Seek},
};

use bevy::ecs::error::BevyError;
pub use decoder::Decoder;

struct Packet {
    pub data: Vec<u8>,
    pub pts: u64,
    pub duration: u32,
}

trait Demuxer {
    fn width(&self) -> u16;
    fn height(&self) -> u16;
    fn timescale(&self) -> u32;
    fn read_packet(&mut self) -> Result<Packet, Error>;
    fn reset(&mut self) -> Result<(), Error>;
}

#[allow(clippy::large_enum_variant)]
pub enum Demuxers<R: Read + Seek + Send> {
    Ivf(ivf::IvfDemuxer<R>),
    Mp4(mp4::Mp4Demuxer<R>),
}

impl<R: Read + Seek + Send> Demuxer for Demuxers<R> {
    fn width(&self) -> u16 {
        match self {
            Demuxers::Ivf(ivf_demuxer) => ivf_demuxer.width(),
            Demuxers::Mp4(mp4_demuxer) => mp4_demuxer.width(),
        }
    }
    fn height(&self) -> u16 {
        match self {
            Demuxers::Ivf(ivf_demuxer) => ivf_demuxer.height(),
            Demuxers::Mp4(mp4_demuxer) => mp4_demuxer.height(),
        }
    }
    fn timescale(&self) -> u32 {
        match self {
            Demuxers::Ivf(ivf_demuxer) => ivf_demuxer.timescale(),
            Demuxers::Mp4(mp4_demuxer) => mp4_demuxer.timescale(),
        }
    }
    fn read_packet(&mut self) -> Result<Packet, Error> {
        match self {
            Demuxers::Ivf(ivf_demuxer) => ivf_demuxer.read_packet(),
            Demuxers::Mp4(mp4_demuxer) => mp4_demuxer.read_packet(),
        }
    }
    fn reset(&mut self) -> Result<(), Error> {
        match self {
            Demuxers::Ivf(ivf_demuxer) => ivf_demuxer.reset(),
            Demuxers::Mp4(mp4_demuxer) => mp4_demuxer.reset(),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    DemuxerIO(std::io::Error),
    Demuxer(BevyError),
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
