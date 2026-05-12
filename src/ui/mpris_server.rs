use std::sync::Arc;

use iced::Task;
use mpris_server::{
    LoopStatus, Metadata, PlaybackRate, PlaybackStatus, PlayerInterface, RootInterface, Time,
    TrackId, Volume, zbus::Result, zbus::fdo,
};

use crate::backend::mpv_events::MpvEvent;

#[derive(Default)]
struct MprisServer {
    mpris_server: Option<Arc<MprisPlayer>>
}

pub enum CmdMsg {
    /// Requests that the player is being built
    PlayerEvent(MpvEvent),
}

pub enum OutMsg {
    /// Requests that the player is being built
    PlayerEvent(MpvEvent),
}

impl MprisServer {
    pub fn update(&mut self, message: CmdMsg) -> Task<OutMsg> {
        match message {
            CmdMsg::PlayerEvent(mpv_event) => todo!(),
        }
    }
}

struct MprisPlayer {}

impl RootInterface for MprisPlayer {
    // Raising to foreground
    async fn can_raise(&self) -> fdo::Result<bool> {
         Ok(false) // TODO:
    }

    async fn raise(&self) -> fdo::Result<()> {
         Ok(()) // TODO:
    }

    // Quitting
    async fn can_quit(&self) -> fdo::Result<bool> {
         Ok(true) // TODO:
    }

    async fn quit(&self) -> fdo::Result<()> {
         todo!() // TODO:
    }

    // Fullscreen
    async fn can_set_fullscreen(&self) -> fdo::Result<bool> {
         Ok(false) // TODO:
    }

    async fn set_fullscreen(
        &self,
        fullscreen: bool,
    ) -> Result<()> {
         Ok(()) // TODO:
    }

     // Is fullscreen?
    async fn fullscreen(&self) -> fdo::Result<bool> {
        Ok(false) // TODO:
    }

    async fn has_track_list(&self) -> fdo::Result<bool> {
         todo!()
    }

    async fn identity(&self) -> fdo::Result<String> {
         todo!()
    }

    async fn desktop_entry(&self) -> fdo::Result<String> {
         todo!()
    }

    async fn supported_uri_schemes(
        &self,
    ) -> fdo::Result<Vec<String>> {
         todo!()
    }

    async fn supported_mime_types(&self) -> fdo::Result<Vec<String>> {
         todo!()
    }
}

impl PlayerInterface for MprisPlayer {
    async fn next(&self) -> fdo::Result<()> {
         todo!()
    }

    async fn previous(&self) -> fdo::Result<()> {
         todo!()
    }

    async fn pause(&self) -> fdo::Result<()> {
         todo!()
    }

    async fn play_pause(&self) -> fdo::Result<()> {
         todo!()
    }

    async fn stop(&self) -> fdo::Result<()> {
         todo!()
    }

    async fn play(&self) -> fdo::Result<()> {
         todo!()
    }

    async fn seek(&self, offset: Time) -> fdo::Result<()> {
         todo!()
    }

    async fn set_position(
        &self,
        track_id: TrackId,
        position: Time,
    ) -> fdo::Result<()> {
         todo!()
    }

    async fn open_uri(&self, uri: String) -> fdo::Result<()> {
         todo!()
    }

    async fn playback_status(&self) -> fdo::Result<PlaybackStatus> {
         todo!()
    }

    async fn loop_status(&self) -> fdo::Result<LoopStatus> {
         todo!()
    }

    async fn set_loop_status(
        &self,
        loop_status: LoopStatus,
    ) -> Result<()> {
         todo!()
    }

    async fn rate(&self) -> fdo::Result<PlaybackRate> {
         todo!()
    }

    async fn set_rate(&self, rate: PlaybackRate) -> Result<()> {
         todo!()
    }

    async fn shuffle(&self) -> fdo::Result<bool> {
         todo!()
    }

    async fn set_shuffle(&self, shuffle: bool) -> Result<()> {
         todo!()
    }

    async fn metadata(&self) -> fdo::Result<Metadata> {
         todo!()
    }

    async fn volume(&self) -> fdo::Result<Volume> {
         todo!()
    }

    async fn set_volume(&self, volume: Volume) -> Result<()> {
         todo!()
    }

    async fn position(&self) -> fdo::Result<Time> {
         todo!()
    }

    async fn minimum_rate(&self) -> fdo::Result<PlaybackRate> {
         todo!()
    }

    async fn maximum_rate(&self) -> fdo::Result<PlaybackRate> {
         todo!()
    }

    async fn can_go_next(&self) -> fdo::Result<bool> {
         todo!()
    }

    async fn can_go_previous(&self) -> fdo::Result<bool> {
         todo!()
    }

    async fn can_play(&self) -> fdo::Result<bool> {
         todo!()
    }

    async fn can_pause(&self) -> fdo::Result<bool> {
         todo!()
    }

    async fn can_seek(&self) -> fdo::Result<bool> {
         todo!()
    }

    async fn can_control(&self) -> fdo::Result<bool> {
         todo!()
    }
}
