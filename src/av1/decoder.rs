use async_channel::SendError;
use std::{
    error::Error,
    io::{Read, Seek},
    time::Duration,
};
use yuv::{
    YuvGrayImage, YuvPlanarImage, YuvRange, YuvStandardMatrix, yuv400_to_bgra, yuv420_to_bgra,
    yuv422_to_bgra, yuv444_to_bgra,
};

use super::{Demuxer, Demuxers};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use std::result::Result;

use crate::decodable::VideoFrame;

// Based on https://github.com/rust-av/dav1d-rs/blob/master/tools/src/main.rs

pub struct Decoder<R: Read + Seek + Send> {
    decoder: dav1d::Decoder,
    demuxer: Demuxers<R>,
}

impl<R: Read + Seek + Send> Decoder<R> {
    pub fn new(demuxer: Demuxers<R>) -> Result<Self, BevyError> {
        let mut settings = dav1d::Settings::new();
        settings.set_n_threads(1);
        Ok(Self {
            decoder: dav1d::Decoder::with_settings(&settings)?,
            demuxer,
        })
    }

    async fn decode(
        &mut self,
        tx: async_channel::Sender<VideoFrame>,
        loop_: bool,
    ) -> Result<(), DecodeError> {
        loop {
            while let Some(packet) = self.demuxer.read_packet()? {
                // Send packet to the decoder
                match self.decoder.send_data(
                    packet.data,
                    None,
                    Some(packet.pts as i64),
                    Some(packet.duration as i64),
                ) {
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

            if loop_ {
                self.demuxer.reset()?;
            } else {
                break;
            }
        }
        Ok(())
    }

    async fn handle_pending_pictures(
        &mut self,
        tx: &async_channel::Sender<VideoFrame>,
        drain: bool,
    ) -> Result<(), DecodeError> {
        loop {
            match self.decoder.get_picture() {
                Ok(p) => {
                    let timescale = self.demuxer.timescale() as f64;
                    let pts = Duration::from_secs_f64(p.timestamp().unwrap() as f64 / timescale);
                    let duration = Duration::from_secs_f64(p.duration() as f64 / timescale);
                    let frame = VideoFrame {
                        image: Image::new(
                            Extent3d {
                                width: p.width(),
                                height: p.height(),
                                ..default()
                            },
                            TextureDimension::D2,
                            self.yuv_to_bgr(&p)?,
                            TextureFormat::Bgra8UnormSrgb, //XXX Bgra8Unorm or Bgra8UnormSrgb
                            RenderAssetUsages::default(),
                        ),
                        timestamp: pts,
                        duration,
                    };
                    tx.send(frame).await?;
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

    fn yuv_to_bgr(&self, p: &dav1d::Picture) -> Result<Vec<u8>, BevyError> {
        assert!(p.bit_depth() == 8, "AV1 bit depth must be 8");
        let range = match p.color_range() {
            dav1d::pixel::YUVRange::Limited => YuvRange::Limited,
            dav1d::pixel::YUVRange::Full => YuvRange::Full,
        };
        let matrix = match p.matrix_coefficients() {
            dav1d::pixel::MatrixCoefficients::BT709 => YuvStandardMatrix::Bt709,
            dav1d::pixel::MatrixCoefficients::BT470BG
            | dav1d::pixel::MatrixCoefficients::ST170M => YuvStandardMatrix::Bt601,
            dav1d::pixel::MatrixCoefficients::ST240M => YuvStandardMatrix::Smpte240,
            dav1d::pixel::MatrixCoefficients::BT2020NonConstantLuminance
            | dav1d::pixel::MatrixCoefficients::BT2020ConstantLuminance => {
                YuvStandardMatrix::Bt2020
            }
            _ => YuvStandardMatrix::Bt601,
        };
        let mut bgra_data = vec![0; (p.width() * p.height() * 4) as usize];
        match p.pixel_layout() {
            dav1d::PixelLayout::I400 => {
                let yuv_data = YuvGrayImage {
                    y_plane: &p.plane(dav1d::PlanarImageComponent::Y),
                    y_stride: p.stride(dav1d::PlanarImageComponent::Y),
                    width: p.width(),
                    height: p.height(),
                };
                yuv400_to_bgra(&yuv_data, &mut bgra_data, p.width() * 4, range, matrix)?
            }
            layout => {
                let yuv_data = YuvPlanarImage {
                    y_plane: &p.plane(dav1d::PlanarImageComponent::Y),
                    y_stride: p.stride(dav1d::PlanarImageComponent::Y),
                    u_plane: &p.plane(dav1d::PlanarImageComponent::U),
                    u_stride: p.stride(dav1d::PlanarImageComponent::U),
                    v_plane: &p.plane(dav1d::PlanarImageComponent::V),
                    v_stride: p.stride(dav1d::PlanarImageComponent::V),
                    width: p.width(),
                    height: p.height(),
                };
                match layout {
                    dav1d::PixelLayout::I420 => {
                        yuv420_to_bgra(&yuv_data, &mut bgra_data, p.width() * 4, range, matrix)?
                    }
                    dav1d::PixelLayout::I422 => {
                        yuv422_to_bgra(&yuv_data, &mut bgra_data, p.width() * 4, range, matrix)?
                    }
                    dav1d::PixelLayout::I444 => {
                        yuv444_to_bgra(&yuv_data, &mut bgra_data, p.width() * 4, range, matrix)?
                    }
                    dav1d::PixelLayout::I400 => {}
                }
            }
        };
        Ok(bgra_data)
    }
}

impl<R: Read + Seek + Send> crate::decodable::Decoder for Decoder<R> {
    fn width(&self) -> u32 {
        self.demuxer.width() as u32
    }

    fn height(&self) -> u32 {
        self.demuxer.height() as u32
    }

    fn timescale(&self) -> u32 {
        self.demuxer.timescale()
    }

    async fn decode(
        &mut self,
        tx: async_channel::Sender<VideoFrame>,
        loop_: bool,
    ) -> Result<(), BevyError> {
        match Decoder::decode(self, tx, loop_).await {
            Err(DecodeError::SendError) => Ok(()),
            Err(DecodeError::BevyError(e)) => Err(e),
            Ok(_) => Ok(()),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum DecodeError {
    BevyError(BevyError),
    SendError,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for DecodeError {}

impl From<BevyError> for DecodeError {
    fn from(error: BevyError) -> Self {
        DecodeError::BevyError(error)
    }
}

impl From<SendError<VideoFrame>> for DecodeError {
    fn from(_: SendError<VideoFrame>) -> Self {
        DecodeError::SendError
    }
}

impl From<dav1d::Error> for DecodeError {
    fn from(error: dav1d::Error) -> Self {
        DecodeError::BevyError(error.into())
    }
}
