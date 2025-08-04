use bevy::{asset::AsyncReadExt, tasks::futures_lite::AsyncRead};
use bitstream_io::{ByteRead, ByteReader, LittleEndian};
use std::io;

#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    pub w: u16,
    pub h: u16,
    pub timebase_num: u32,
    pub timebase_den: u32,
}

pub async fn read_header<R: AsyncRead + Unpin>(r: &mut R) -> io::Result<Header> {
    let mut header = [0u8; 44];
    r.read_exact(&mut header).await?;
    let mut br = ByteReader::endian(header.as_ref(), LittleEndian);

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

pub async fn read_packet<R: AsyncRead + Unpin>(r: &mut R) -> io::Result<Packet> {
    let mut header = [0u8; 12];
    r.read_exact(&mut header).await?;
    let mut br = ByteReader::endian(header.as_ref(), LittleEndian);

    let len = br.read::<u32>()?;
    let pts = br.read::<u64>()?;
    let mut buf = vec![0u8; len as usize];
    r.read_exact(&mut buf).await?;

    Ok(Packet { data: buf, pts })
}
