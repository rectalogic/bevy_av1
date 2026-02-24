pub mod decoder;
pub mod ivf;
pub mod mp4;
use std::{
    fmt::Display,
    io::{Read, Seek},
};

pub use decoder::Decoder;

struct Packet {
    pub data: Vec<u8>,
    pub pts: u64,
}

trait Demuxer {
    fn width(&self) -> u16;
    fn height(&self) -> u16;
    fn timebase(&self) -> (u32, u32);
    fn read_packet(&mut self) -> Result<Packet, Error>;
    fn reset(&mut self) -> Result<(), Error>;
}

pub enum Demuxers<R: Read + Send> {
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
    fn timebase(&self) -> (u32, u32) {
        match self {
            Demuxers::Ivf(ivf_demuxer) => ivf_demuxer.timebase(),
            Demuxers::Mp4(mp4_demuxer) => mp4_demuxer.timebase(),
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
