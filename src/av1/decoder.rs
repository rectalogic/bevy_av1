use async_trait::async_trait;
use std::{io::Read, time::Duration};

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bitstream_io::{ByteReader, LittleEndian};

use crate::{av1::ivf, video_source::VideoFrame};

// Based on https://github.com/rust-av/dav1d-rs/blob/master/tools/src/main.rs

pub struct Decoder<R: Read + Send> {
    decoder: dav1d::Decoder,
    demuxer: ivf::Demuxer<R>,
}

impl<R: Read + Send> Decoder<R> {
    pub fn new(reader: R) -> Result<Self> {
        Ok(Self {
            decoder: dav1d::Decoder::new()?, //XXX configure # threads?
            demuxer: ivf::Demuxer::new(ByteReader::endian(reader, LittleEndian))?,
        })
    }

    pub async fn decode(&mut self, tx: async_channel::Sender<VideoFrame>) -> Result<()> {
        while let Ok(packet) = self.demuxer.read_packet() {
            // Send packet to the decoder
            match self
                .decoder
                .send_data(packet.data, None, Some(packet.pts as i64), None)
            {
                Err(e) if e.is_again() => {
                    // If the decoder did not consume all data, output all
                    // pending pictures and send pending data to the decoder
                    // until it is all used up.
                    loop {
                        self.handle_pending_pictures(&tx, false).await?;

                        match self.decoder.send_pending_data() {
                            Err(e) if e.is_again() => continue,
                            Err(e) => return Err(e.into()),
                            _ => break,
                        }
                    }
                }
                Err(e) => return Err(e.into()),
                _ => (),
            }

            // Handle all pending pictures before sending the next data.
            self.handle_pending_pictures(&tx, false).await?;
        }

        // Handle all pending pictures that were not output yet.
        self.handle_pending_pictures(&tx, true).await?;

        Ok(())
    }

    async fn handle_pending_pictures(
        &mut self,
        tx: &async_channel::Sender<VideoFrame>,
        drain: bool,
    ) -> Result<()> {
        loop {
            match self.decoder.get_picture() {
                Ok(p) => {
                    let pts = p.timestamp().unwrap();
                    let timebase = self.demuxer.timebase();
                    let timebase = timebase.1 as f64 / timebase.0 as f64;
                    let pts = Duration::from_secs_f64(pts as f64 * timebase);
                    let frame = VideoFrame {
                        image: Image::new(
                            Extent3d {
                                width: p.width(),
                                height: p.height(),
                                ..default()
                            },
                            TextureDimension::D2,
                            p.plane(dav1d::PlanarImageComponent::Y).to_vec(),
                            TextureFormat::R8Unorm,
                            RenderAssetUsages::default(),
                        ),
                        pts,
                    };
                    tx.send(frame).await?; // XXX handle SendError gracefully, just stop decoding?
                }
                // Need to send more data to the decoder before it can decode new pictures
                Err(e) if e.is_again() => return Ok(()),
                Err(e) => {
                    return Err(e.into());
                }
            }

            if !drain {
                break;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl<R: Read + Send> crate::video_source::Decoder for Decoder<R> {
    fn width(&self) -> u32 {
        self.demuxer.width() as u32
    }

    fn height(&self) -> u32 {
        self.demuxer.height() as u32
    }

    async fn decode(&mut self, tx: async_channel::Sender<VideoFrame>) -> Result<()> {
        Decoder::decode(self, tx).await
    }
}
