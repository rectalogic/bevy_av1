use super::{Demuxer, Error, Packet};
use bevy::prelude::*;
use mp4::TrackType;
use std::io::{Read, Seek, SeekFrom};

pub struct Mp4Demuxer<R: Read + Seek + Send> {
    reader: mp4::Mp4Reader<R>,
    track_id: u32,
    width: u16,
    height: u16,
    current_sample: u32,
    timescale: u32,
}

impl<R: Read + Seek + Send> Mp4Demuxer<R> {
    pub fn new(mut reader: R) -> Result<Self, BevyError> {
        let old_pos = reader.stream_position()?;
        let len = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(old_pos))?;
        let mp4 = mp4::Mp4Reader::read_header(reader, len)?;
        //XXX validate contains AV1 video
        if let Some((&track_id, track)) = mp4
            .tracks()
            .iter()
            .find(|&(_, track)| matches!(track.track_type(), Ok(TrackType::Video)))
        {
            Ok(Self {
                width: track.width(),
                height: track.height(),
                timescale: track.timescale(),
                reader: mp4,
                current_sample: 1,
                track_id,
            })
        } else {
            Err("No video track found".into())
        }
    }
}

impl<R: Read + Seek + Send> Demuxer for Mp4Demuxer<R> {
    fn width(&self) -> u16 {
        self.width
    }

    fn height(&self) -> u16 {
        self.height
    }

    fn timescale(&self) -> u32 {
        self.timescale
    }

    //XXX change to distinguish error from EOF/loop
    fn read_packet(&mut self) -> Result<Packet, Error> {
        if let Some(sample) = self
            .reader
            .read_sample(self.track_id, self.current_sample)
            .map_err(|e| Error::Demuxer(e.into()))?
        {
            self.current_sample += 1;
            Ok(Packet {
                data: sample.bytes.into(),
                pts: sample.start_time,
                duration: sample.duration,
            })
        } else {
            Err(Error::Demuxer("EOF".into()))
        }
    }

    fn reset(&mut self) -> Result<(), Error> {
        self.current_sample = 1;
        Ok(())
    }
}
