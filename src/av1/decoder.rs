use std::{
    io::{Read, Seek},
    time::Duration,
};
use yuv::{
    YuvGrayImage, YuvPlanarImage, YuvRange, YuvStandardMatrix, yuv400_to_bgra, yuv420_to_bgra,
    yuv422_to_bgra, yuv444_to_bgra,
};

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use std::result::Result;

use crate::{av1, video_source::VideoFrame};

// Based on https://github.com/rust-av/dav1d-rs/blob/master/tools/src/main.rs

pub struct Decoder<R: Read + Seek + Send> {
    decoder: dav1d::Decoder,
    demuxer: av1::ivf::Demuxer<R>,
}

impl<R: Read + Seek + Send> Decoder<R> {
    pub fn new(reader: R) -> Result<Self, av1::Error> {
        let mut settings = dav1d::Settings::new();
        settings.set_n_threads(1);
        Ok(Self {
            decoder: dav1d::Decoder::with_settings(&settings).map_err(av1::Error::Decoder)?,
            demuxer: av1::ivf::Demuxer::new(reader).map_err(av1::Error::Demuxer)?,
        })
    }

    pub async fn decode(
        &mut self,
        tx: async_channel::Sender<VideoFrame>,
        loop_: bool,
    ) -> Result<(), av1::Error> {
        loop {
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
                                Err(e) => return Err(av1::Error::Decoder(e)),
                                _ => break,
                            }
                        }
                    }
                    Err(e) => return Err(av1::Error::Decoder(e)),
                    _ => (),
                }

                // Handle all pending pictures before sending the next data.
                self.handle_pending_pictures(&tx, false).await?;
            }

            // Handle all pending pictures that were not output yet.
            self.handle_pending_pictures(&tx, true).await?;

            if loop_ {
                self.demuxer.reset().map_err(av1::Error::Demuxer)?;
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
    ) -> Result<(), av1::Error> {
        loop {
            match self.decoder.get_picture() {
                Ok(p) => {
                    let pts = p.timestamp().unwrap();
                    let timebase = self.demuxer.timebase();
                    let timebase = timebase.0 as f64 / timebase.1 as f64;
                    let pts = Duration::from_secs_f64(pts as f64 * timebase);
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
                    };
                    tx.send(frame)
                        .await
                        .map_err(|_| av1::Error::ChannelClosed)?;
                }
                // Need to send more data to the decoder before it can decode new pictures
                Err(e) if e.is_again() => return Ok(()),
                Err(e) => {
                    return Err(av1::Error::Decoder(e));
                }
            }

            if !drain {
                break;
            }
        }
        Ok(())
    }

    fn yuv_to_bgr(&self, p: &dav1d::Picture) -> Result<Vec<u8>, av1::Error> {
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
                yuv400_to_bgra(&yuv_data, &mut bgra_data, p.width() * 4, range, matrix)
                    .map_err(av1::Error::Conversion)?
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
                        yuv420_to_bgra(&yuv_data, &mut bgra_data, p.width() * 4, range, matrix)
                            .map_err(av1::Error::Conversion)?
                    }
                    dav1d::PixelLayout::I422 => {
                        yuv422_to_bgra(&yuv_data, &mut bgra_data, p.width() * 4, range, matrix)
                            .map_err(av1::Error::Conversion)?
                    }
                    dav1d::PixelLayout::I444 => {
                        yuv444_to_bgra(&yuv_data, &mut bgra_data, p.width() * 4, range, matrix)
                            .map_err(av1::Error::Conversion)?
                    }
                    dav1d::PixelLayout::I400 => {}
                }
            }
        };
        Ok(bgra_data)
    }
}

impl<R: Read + Seek + Send> crate::video_source::Decoder for Decoder<R> {
    fn width(&self) -> u32 {
        self.demuxer.width() as u32
    }

    fn height(&self) -> u32 {
        self.demuxer.height() as u32
    }

    fn timebase(&self) -> (u32, u32) {
        self.demuxer.timebase()
    }

    async fn decode(
        &mut self,
        tx: async_channel::Sender<VideoFrame>,
        loop_: bool,
    ) -> Result<(), BevyError> {
        match Decoder::decode(self, tx, loop_).await {
            Err(av1::Error::ChannelClosed) => Ok(()),
            Err(e) => Err(e.into()),
            Ok(_) => Ok(()),
        }
    }
}
