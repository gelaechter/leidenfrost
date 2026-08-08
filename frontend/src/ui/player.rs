//! Contains the logic for the player that actually plays the audio
//! At the moment this uses MPV as the backend as it I deem it highly reliable
//! and feature-complete

use std::sync::LazyLock;

use iced::{Subscription, Task};
use iced::{advanced::graphics::futures::backend::default, futures::Stream};
use libmpv2::{Format, Mpv};
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::BroadcastStream;

use backend::{
    data_view::TrackView,
    db::models::OrmUrl,
    mpv_data::{MpvEvent, MpvValue},
    player::{Player, PlayerEvent, mpv_player::MpvPlayer},
};

use crate::ui::{
    ICMsg, ToErrMsg, ToOutMsg,
    player::{
        command::{
            LOADFILE, PLAYLIST_MOVE, PLAYLIST_NEXT, PLAYLIST_PLAY_INDEX, PLAYLIST_PREV,
            PLAYLIST_REMOVE, SEEK, STOP,
        },
        property::{DURATION, LOOP_FILE, LOOP_PLAYLIST, PAUSE, SHUFFLE, TIME_POS, VOLUME},
    },
};

#[derive(Clone, Debug)]
pub enum Cmd {
    /// An Event received from the MPV player
    /// this needs to be converted to an [`Out`] for public consumption
    Event(MpvEvent),
}

#[derive(Clone, Debug)]
pub enum Out {
    Event(PlayerEvent),
    Queue(Box<QueueEvent>),
}

/// TODO: Consider extracing these directly into Out
#[derive(Clone, Debug)]
pub enum QueueEvent {
    /// A track has been appended
    Append(TrackView),
}

pub type PlayerMsg = ICMsg<Cmd, Out, PlayerError>;

pub struct GenericPlayer {
    event_receiver: broadcast::Receiver<PlayerEvent>,
    player_impl: PlayerImpl,
}

#[derive(Default)]
pub enum PlayerImpl {
    #[default]
    MpvPlayer(MpvPlayer),
}

impl Default for GenericPlayer {
    fn default() -> Self {
        let (player, event_receiver) = MpvPlayer::new().expect("Error during player construction");
        let player_impl = PlayerImpl::MpvPlayer(player);

        Self {
            event_receiver,
            player_impl,
        }
    }
}

// TODO: Since player doesn't implement view we technically don't need to use
// ELM We could instead just have a function for each message which might
// declutter things.
impl MpvPlayer {
    pub fn update(&mut self, message: impl Into<PlayerMsg>) -> Task<PlayerMsg> {
        // Only commands need to be handled
        let icmsg = message.into();
        icmsg.cmd(|c| {
            match c {
                // Handle the events emitted from [`Self::subscription`]
                Cmd::Event(event) => self.handle_event(event),
            }
        })
    }

    /// Handles the events emitted from [`Self::subscription`]
    fn handle_event(&mut self, mpv_event: MpvEvent) -> Task<PlayerMsg> {
        let MpvEvent::PropertyChange { name, value, .. } = mpv_event else {
            // Events beyond [`MpvEvent::PropertyChange`] are currently not handled
            return Task::none();
        };

        // Convert the raw mpv events to proper [`Out`] commands for easy consumption
        let event = match (name.as_str(), value) {
            (SHUFFLE, MpvValue::Bool(s)) => PlayerEvent::Shuffle(s),
            (LOOP_FILE, MpvValue::String(val)) => {
                self.loop_file = val;
                PlayerEvent::Repeat(self.determine_repeat_mode())
            }
            (LOOP_PLAYLIST, MpvValue::String(val)) => {
                self.loop_playlist = val;
                PlayerEvent::Repeat(self.determine_repeat_mode())
            }
            (PAUSE, MpvValue::Bool(p)) => PlayerEvent::Pause(p),
            (TIME_POS, MpvValue::F64(p)) => PlayerEvent::PlaybackPos(p),
            (DURATION, MpvValue::F64(d)) => PlayerEvent::Duration(d),
            (VOLUME, MpvValue::I64(d)) => PlayerEvent::Volume(u32::try_from(d).unwrap()),
            _ => return Task::none(),
        };

        // Dispatch task so this can be dealt with at some upper layer
        Task::done(Out::Event(event).out_msg())
    }

    /// Subscribes to the Mpv threads
    pub fn subscription() -> Subscription<ICMsg<Cmd, Out, PlayerError>> {
        fn subscribe_mpv_events() -> impl Stream<Item = MpvEvent> {
            use tokio_stream::StreamExt;

            let stream = EVENT_CHANNEL.subscribe();
            BroadcastStream::new(stream).filter_map(Result::ok)
        }

        Subscription::run(subscribe_mpv_events).filter_map(|m| {
            if let &MpvEvent::PropertyChange { .. } = &m {
                Some(Cmd::Event(m).into())
            } else {
                None
            }
        })
    }
}

pub fn task_from_error(err: impl Into<PlayerError>) -> Task<PlayerMsg> {
    Task::done(err.into().err_msg())
}
