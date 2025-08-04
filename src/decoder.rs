use std::io::{BufReader, Read};

use bevy::prelude::*;
use dav1d::Picture;

use crate::ivf;

// Based on https://github.com/rust-av/dav1d-rs/blob/master/tools/src/main.rs

pub struct Av1Decoder<R: Read> {
    decoder: dav1d::Decoder,
    tx: crossbeam_channel::Sender<Picture>,
    reader: BufReader<R>,
}

impl<R: Read> Av1Decoder<R> {
    pub fn new(reader: R) -> Result<(Self, crossbeam_channel::Receiver<Picture>)> {
        let (tx, rx) = crossbeam_channel::bounded(2);
        Ok((
            Self {
                decoder: dav1d::Decoder::new()?,
                tx,
                reader: BufReader::new(reader),
            },
            rx,
        ))
    }

    pub fn play(&mut self) -> Result<()> {
        let header = ivf::read_header(&mut self.reader)?;
        println!("{header:?}");

        while let Ok(packet) = ivf::read_packet(&mut self.reader) {
            println!("Packet {}", packet.pts);

            // Send packet to the decoder
            match self.decoder.send_data(packet.data, None, None, None) {
                Err(e) if e.is_again() => {
                    // If the decoder did not consume all data, output all
                    // pending pictures and send pending data to the decoder
                    // until it is all used up.
                    loop {
                        self.handle_pending_pictures(false)?;

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
            self.handle_pending_pictures(false)?;
        }

        // Handle all pending pictures that were not output yet.
        self.handle_pending_pictures(true)?;

        Ok(())
    }

    fn handle_pending_pictures(&mut self, drain: bool) -> Result<()> {
        loop {
            match self.decoder.get_picture() {
                Ok(p) => self.tx.send(p)?, // XXX handle SendError gracefully, just stop decoding?
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
