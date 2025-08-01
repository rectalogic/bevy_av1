use bitstream_io::{ByteRead, ByteReader, LittleEndian};
use std::io;

#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    pub tag: [u8; 4],
    pub w: u16,
    pub h: u16,
    pub timebase_num: u32,
    pub timebase_den: u32,
}

pub fn read_header(r: &mut dyn io::Read) -> io::Result<Header> {
    let mut br = ByteReader::endian(r, LittleEndian);

    let mut signature = [0u8; 4];
    let mut tag = [0u8; 4];

    br.read_bytes(&mut signature)?;
    let v0 = br.read::<u16>()?;
    let v1 = br.read::<u16>()?;
    br.read_bytes(&mut tag)?;

    let w = br.read::<u16>()?;
    let h = br.read::<u16>()?;

    let timebase_den = br.read::<u32>()?;
    let timebase_num = br.read::<u32>()?;

    let _ = br.read::<u32>()?;
    let _ = br.read::<u32>()?;

    Ok(Header {
        tag,
        w,
        h,
        timebase_num,
        timebase_den,
    })
}

pub struct Packet {
    pub data: Box<[u8]>,
    pub pts: u64,
}

pub fn read_packet(r: &mut dyn io::Read) -> io::Result<Packet> {
    let mut br = ByteReader::endian(r, LittleEndian);

    let len = br.read::<u32>()?;
    let pts = br.read::<u64>()?;
    let mut buf = vec![0u8; len as usize];

    br.read_bytes(&mut buf)?;

    Ok(Packet {
        data: buf.into_boxed_slice(),
        pts,
    })
}
