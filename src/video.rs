use bevy::prelude::*;

use crate::video_source::VideoSource;

//XXX add PlaybackSettings to control mode, paused etc.?

#[derive(Component, Clone)]
pub struct VideoPlayer<Source = VideoSource>(pub Handle<Source>)
where
    Source: Asset; //XXX if we add Decodable trait: + Decodable;
