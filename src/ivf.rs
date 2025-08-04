use bitstream_io::{ByteRead, ByteReader, LittleEndian};
use std::io;

#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    pub w: u16,
    pub h: u16,
    pub timebase_num: u32,
    pub timebase_den: u32,
}

pub fn read_header(r: &mut dyn io::Read) -> io::Result<Header> {
    let mut br = ByteReader::endian(r, LittleEndian);

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

    br.skip(8)?;

    Ok(Header {
        w,
        h,
        timebase_num,
        timebase_den,
    })
}

pub struct Packet {
    pub data: Vec<u8>,
    pub pts: u64,
}

pub fn read_packet(r: &mut dyn io::Read) -> io::Result<Packet> {
    let mut br = ByteReader::endian(r, LittleEndian);

    let len = br.read::<u32>()?;
    let pts = br.read::<u64>()?;
    let mut buf = vec![0u8; len as usize];

    br.read_bytes(&mut buf)?;

    Ok(Packet { data: buf, pts })
}
