use super::{Demuxer, Error, Packet};
use bitstream_io::{ByteReader, LittleEndian};
use std::io::{self, Read, Seek};

pub struct Mp4Demuxer<R: Read + Send> {
    reader: ByteReader<R, LittleEndian>,
}

impl<R: Read + Seek + Send> Mp4Demuxer<R> {
    pub fn new(reader: R) -> io::Result<Self> {
        let mut reader = ByteReader::endian(reader, LittleEndian);
        Ok(Self { reader })
    }
}

impl<R: Read + Seek + Send> Demuxer for Mp4Demuxer<R> {
    fn width(&self) -> u16 {
        todo!()
    }

    fn height(&self) -> u16 {
        todo!()
    }

    fn timebase(&self) -> (u32, u32) {
        todo!()
    }

    fn read_packet(&mut self) -> Result<Packet, Error> {
        todo!()
    }

    fn reset(&mut self) -> Result<(), Error> {
        todo!()
    }
}
