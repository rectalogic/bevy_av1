use std::{fs::File, io::BufReader};

use bevy::prelude::*;
use dav1d::Picture;

use crate::ivf;

fn handle_pending_pictures(dec: &mut dav1d::Decoder) -> Option<Picture> {
    match dec.get_picture() {
        Ok(p) => Some(p),
        // Need to send more data to the decoder before it can decode new pictures
        Err(e) if e.is_again() => None,
        Err(e) => {
            panic!("Error getting decoded pictures: {}", e);
        }
    }
}

pub fn decode() -> Result<Picture> {
    let file = File::open("Johnny_1280x720.ivf")?;
    let mut r = BufReader::new(file);
    let header = ivf::read_header(&mut r)?;
    println!("{header:?}");

    let mut dec = dav1d::Decoder::new().expect("failed to create decoder instance");

    while let Ok(packet) = ivf::read_packet(&mut r) {
        println!("Packet {}", packet.pts);

        // Send packet to the decoder
        match dec.send_data(packet.data, None, None, None) {
            Err(e) if e.is_again() => {
                // If the decoder did not consume all data, output all
                // pending pictures and send pending data to the decoder
                // until it is all used up.
                loop {
                    if let Some(p) = handle_pending_pictures(&mut dec) {
                        return Ok(p);
                    }

                    match dec.send_pending_data() {
                        Err(e) if e.is_again() => continue,
                        Err(e) => {
                            panic!("Error sending pending data to the decoder: {}", e);
                        }
                        _ => break,
                    }
                }
            }
            Err(e) => {
                panic!("Error sending data to the decoder: {}", e);
            }
            _ => (),
        }

        // Handle all pending pictures before sending the next data.
        if let Some(p) = handle_pending_pictures(&mut dec) {
            return Ok(p);
        }
    }

    // Handle all pending pictures that were not output yet.
    if let Some(p) = handle_pending_pictures(&mut dec) {
        return Ok(p);
    }

    Err("nothing decoded".into())
}
