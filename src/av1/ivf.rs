use super::{Demuxer, Error, Packet};
use bitstream_io::{ByteRead, ByteReader, LittleEndian};
use std::io::{self, Read, Seek, SeekFrom};

pub const HEADER_SIZE: u64 = 32;

pub struct IvfDemuxer<R: Read + Send> {
    reader: ByteReader<R, LittleEndian>,
    header: Header,
}

#[derive(Debug, PartialEq, Eq)]
struct Header {
    pub w: u16,
    pub h: u16,
    pub frame_count: u32,
    pub timebase_num: u32,
    pub timebase_den: u32,
}

impl<R: Read + Seek + Send> IvfDemuxer<R> {
    pub fn new(reader: R) -> io::Result<Self> {
        let mut reader = ByteReader::endian(reader, LittleEndian);
        let header = Self::read_header(&mut reader)?;
        Ok(Self { reader, header })
    }

    fn read_header(br: &mut ByteReader<R, LittleEndian>) -> io::Result<Header> {
        const TAG: &[u8] = b"DKIF";
        const CODEC: &[u8] = b"AV01";
        let mut signature = [0u8; 4];

        br.read_bytes(&mut signature)?;
        if signature != TAG {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid IVF tag",
            ));
        }
        br.skip(4)?;
        br.read_bytes(&mut signature)?;
        if signature != CODEC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "IVF does not contain AV1",
            ));
        }

        let w = br.read::<u16>()?;
        let h = br.read::<u16>()?;

        let timebase_den = br.read::<u32>()?;
        let timebase_num = br.read::<u32>()?;
        let frames = br.read::<u32>()?;
        br.skip(4)?;

        Ok(Header {
            w,
            h,
            frame_count: frames,
            timebase_num,
            timebase_den,
        })
    }
}

impl<R: Read + Seek + Send> Demuxer for IvfDemuxer<R> {
    fn width(&self) -> u16 {
        self.header.w
    }

    fn height(&self) -> u16 {
        self.header.h
    }

    fn timebase(&self) -> (u32, u32) {
        (self.header.timebase_num, self.header.timebase_den)
    }

    fn read_packet(&mut self) -> Result<Packet, Error> {
        let len = self.reader.read::<u32>().map_err(Error::Demuxer)?;
        let pts = self.reader.read::<u64>().map_err(Error::Demuxer)?;
        let mut buf = vec![0u8; len as usize];
        self.reader.read_bytes(&mut buf).map_err(Error::Demuxer)?;

        Ok(Packet { data: buf, pts })
    }

    fn reset(&mut self) -> Result<(), Error> {
        self.reader
            .reader()
            .seek(SeekFrom::Start(HEADER_SIZE))
            .map_err(Error::Demuxer)?;
        Ok(())
    }
}
