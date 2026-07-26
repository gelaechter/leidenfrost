//! Contains the logic for the player that actually plays the audio
//! At the moment this uses MPV as the backend as it I deem it highly reliable
//! and feature-complete

use std::sync::LazyLock;

use iced::futures::Stream;
use iced::{Subscription, Task};
use libmpv2::{Format, Mpv};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use backend::{
    data_view::TrackView,
    db::models::OrmUrl,
    mpv_data::{MpvEvent, MpvValue},
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

#[derive(Default, Clone, Debug)]
pub enum RepeatMode {
    #[default]
    /// Do not repeat at all
    None,
    /// Repeat the current song
    Song,
    /// Repeat the queue
    Queue,
}

#[derive(Default)]
pub enum PlayerState {
    #[default]
    Stopped,
    Playing,
    Paused,
}

pub struct MpvPlayer {
    /// the Mpv instance
    mpv: Mpv,
    /// For some god-forsaken reason mpv stores the
    /// repeat state as strings ("no" and "inf")
    loop_file: String,
    loop_playlist: String,
}

impl Default for MpvPlayer {
    fn default() -> Self {
        Self {
            mpv: initialize_mpv(),
            loop_file: String::from("no"),
            loop_playlist: String::from("no"),
        }
    }
}

/// Mpv properties (<https://mpv.io/manual/master/#properties>) \
/// These can be inspected in the MPV GUI using `g-r`
pub mod property {
    /// <https://mpv.io/manual/master/#options-volume>
    pub const VOLUME: &str = "volume";
    /// <https://mpv.io/manual/master/#command-interface-time-pos>
    pub const TIME_POS: &str = "time-pos";
    /// If the player is currently paused
    pub const PAUSE: &str = "pause";
    /// If shuffle is active
    pub const SHUFFLE: &str = "shuffle";
    /// <https://mpv.io/manual/master/#options-loop>
    pub const LOOP_FILE: &str = "loop-file";
    /// <https://mpv.io/manual/master/#options-loop-playlist>
    pub const LOOP_PLAYLIST: &str = "loop-playlist";
    /// <https://mpv.io/manual/master/#command-interface-duration>
    pub const DURATION: &str = "duration";
}

/// A channel over which Mpv Events can travel
/// from the Mpv thread to the iced subscription
static EVENT_CHANNEL: LazyLock<broadcast::Sender<MpvEvent>> =
    LazyLock::new(|| broadcast::channel(100).0);

fn initialize_mpv() -> Mpv {
    let mpv = Mpv::with_initializer(|init| {
        init.set_option("vid", "no")?;
        Ok(())
    })
    .unwrap();

    // Since [`Mpv::wait_event`] takes `&mut self` we
    // create a second Mpv client just for receiving events
    let mut event_context = mpv.create_client(Some("EventContext")).unwrap();

    // Subscribe to all the different properties
    event_context
        .observe_property(TIME_POS, Format::Double, 0)
        .unwrap();
    event_context
        .observe_property(PAUSE, Format::Flag, 1)
        .unwrap();
    event_context
        .observe_property(SHUFFLE, Format::Flag, 2)
        .unwrap();
    event_context
        .observe_property(LOOP_FILE, Format::String, 3)
        .unwrap();
    event_context
        .observe_property(LOOP_PLAYLIST, Format::String, 4)
        .unwrap();
    event_context
        .observe_property(DURATION, Format::Double, 5)
        .unwrap();
    event_context
        .observe_property(VOLUME, Format::Int64, 6)
        .unwrap();

    // Start a thread that continually checks the event queue
    std::thread::spawn(move || {
        loop {
            let event = event_context.wait_event(f64::MAX);
            if let Some(Ok(event)) = event {
                let send_res = EVENT_CHANNEL.send(event.into());
                if send_res.is_err() {
                    log::warn!(
                        "A player event has been emitted before the UI was ready\n\
                        to receive it. Has something gotten out of order?",
                    );
                };
            }
        }
    });

    // Reset repeat state
    mpv.set_property(LOOP_FILE, "no");
    mpv.set_property(LOOP_PLAYLIST, "no");

    mpv
}

pub trait Player {
    /// (Un)pauses the player
    fn pause(&self, paused: bool) -> Task<PlayerMsg>;
    /// Seek to a specific time in seconds
    fn seek(&self, position: f64) -> Task<PlayerMsg>;
    /// Stop playback and clear playlist.
    fn stop(&self) -> Task<PlayerMsg>;
    /// Plays a track, should return an `Out::Queue` task
    fn play(&self, track: TrackView) -> Task<PlayerMsg>;
    /// Adds multiple tracks and plays the first
    fn play_all(&self, tracks: Vec<TrackView>) -> Task<PlayerMsg>;
    /// Plays a certain index in the queue
    fn play_index(&self, index: usize) -> Task<PlayerMsg>;
    fn append(&self, track: TrackView) -> Task<PlayerMsg>;
    fn append_all(&self, tracks: Vec<TrackView>) -> Task<PlayerMsg>;
    /// Remove a track from the queue
    fn queue_remove(&self, index: usize) -> Task<PlayerMsg>;
    /// Move a track in the queue
    ///
    /// Moves a target track in the queue before the position of another one
    fn queue_move(&self, target: usize, position: usize) -> Task<PlayerMsg>;
    fn set_shuffle(&self, shuffle: bool) -> Task<PlayerMsg>;
    /// Changes the repeat mode of the player
    fn change_repeat_mode(&self, mode: RepeatMode) -> Task<PlayerMsg>;
    /// Plays the next song
    fn next(&self) -> Task<PlayerMsg>;
    /// Plays the previous song
    fn previous(&self) -> Task<PlayerMsg>;
    /// Sets the volume (0-100)
    fn volume(&self, percentage: u32) -> Task<PlayerMsg>;
}

impl Player for MpvPlayer {
    fn pause(&self, paused: bool) -> Task<PlayerMsg> {
        self.mpv
            .set_property(PAUSE, paused)
            .map_or_else(task_from_error, |_| Task::none())
    }

    fn seek(&self, pos_in_secs: f64) -> Task<PlayerMsg> {
        self.mpv
            .command(SEEK, &[&pos_in_secs.to_string(), "absolute"])
            .map_or_else(task_from_error, |_| Task::none())
    }

    fn stop(&self) -> Task<PlayerMsg> {
        self.mpv
            .command(STOP, &[])
            .map_or_else(task_from_error, |_| Task::none())
    }

    fn play(&self, view: TrackView) -> Task<PlayerMsg> {
        let url = match track_url(view) {
            Ok(url) => url,
            Err(e) => return task_from_error(e),
        };

        self.mpv
            .command(LOADFILE, &[url.as_str()])
            .map_or_else(task_from_error, |_| Task::none())
    }

    fn play_all(&self, views: Vec<TrackView>) -> Task<PlayerMsg> {
        // Try to play all and return errors for all the unsuccessful queue adds
        // TODO: This iterative approach might get real slow if we have HUGE queues
        let tasks = views.into_iter().enumerate().map(|(idx, view)| {
            // Play the first append the others
            if idx == 0 {
                self.play(view)
            } else {
                self.append(view)
            }
        });

        Task::batch(tasks)
    }

    fn play_index(&self, index: usize) -> Task<PlayerMsg> {
        self.mpv
            .command(PLAYLIST_PLAY_INDEX, &[&index.to_string()])
            .map_or_else(task_from_error, |_| Task::none())
    }

    fn append(&self, view: TrackView) -> Task<PlayerMsg> {
        let url = match track_url(view) {
            Ok(url) => url,
            Err(e) => return task_from_error(e),
        };

        self.mpv
            .command(LOADFILE, &["append", url.as_str()])
            .map_or_else(task_from_error, |_| Task::none())
    }

    fn append_all(&self, tracks: Vec<TrackView>) -> Task<PlayerMsg> {
        let tasks = tracks.into_iter().map(|view| self.append(view));

        Task::batch(tasks)
    }

    fn queue_remove(&self, index: usize) -> Task<PlayerMsg> {
        self.mpv
            .command(PLAYLIST_REMOVE, &[&index.to_string()])
            .map_or_else(task_from_error, |_| Task::none())
    }

    fn queue_move(&self, target: usize, position: usize) -> Task<PlayerMsg> {
        self.mpv
            .command(PLAYLIST_MOVE, &[&target.to_string(), &position.to_string()])
            .map_or_else(task_from_error, |_| Task::none())
    }

    fn set_shuffle(&self, shuffle: bool) -> Task<PlayerMsg> {
        self.mpv
            .set_property(SHUFFLE, shuffle)
            .map_or_else(task_from_error, |_| Task::none())
    }

    fn change_repeat_mode(&self, mode: RepeatMode) -> Task<PlayerMsg> {
        let (loop_file, loop_playlist) = match mode {
            RepeatMode::None => ("no", "no"),
            RepeatMode::Song => ("inf", "no"),
            RepeatMode::Queue => ("no", "inf"),
        };

        let file_res = self.mpv.set_property(LOOP_FILE, loop_file);
        let playlist_res = self.mpv.set_property(LOOP_PLAYLIST, loop_playlist);

        Task::batch([
            file_res.map_or_else(task_from_error, |_| Task::none()),
            playlist_res.map_or_else(task_from_error, |_| Task::none()),
        ])
    }

    fn next(&self) -> Task<PlayerMsg> {
        self.mpv
            .command(PLAYLIST_NEXT, &[])
            .map_or_else(task_from_error, |_| Task::none())
    }

    fn previous(&self) -> Task<PlayerMsg> {
        self.mpv
            .command(PLAYLIST_PREV, &[])
            .map_or_else(task_from_error, |_| Task::none())
    }

    fn volume(&self, percentage: u32) -> Task<PlayerMsg> {
        self.mpv
            .set_property(VOLUME, i64::from(percentage))
            .map_or_else(task_from_error, |_| Task::none())
    }
}

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
pub enum PlayerEvent {
    /// Shuffle has been (de)actived
    Shuffle(bool),
    /// The repeat mode has changed
    Repeat(RepeatMode),
    ///
    Pause(bool),
    /// The playback position in seconds
    PlaybackPos(f64),
    /// The playing items duration has changed
    ///
    /// This might seem obsolete since the application should know the duration
    /// through the queue and the currently playing [`TrackView`], but we treat
    /// the player as our source of truth, in case the Metadata and actual file
    /// don't match.
    Duration(f64),
    /// The volume (0 to 100) has changed
    Volume(u32),
}

/// TODO: Consider extracing these directly into Out
#[derive(Clone, Debug)]
pub enum QueueEvent {
    /// A track has been appended
    Append(TrackView),
}

pub mod command {
    /// <https://mpv.io/manual/master/#command-interface-seek-%3Ctarget%3E-[%3Cflags%3E]>
    pub const SEEK: &str = "seek";
    /// <https://mpv.io/manual/master/#command-interface-[%3Coptions%3E]]]>
    pub const LOADFILE: &str = "loadfile";
    /// <https://mpv.io/manual/master/#command-interface-stop-[%3Cflags%3E]>
    pub const STOP: &str = "stop";
    /// <https://mpv.io/manual/master/#command-interface-playlist-remove>
    pub const PLAYLIST_REMOVE: &str = "playlist-remove";
    /// <https://mpv.io/manual/master/#command-interface-playlist-next>
    pub const PLAYLIST_NEXT: &str = "playlist-next";
    /// <https://mpv.io/manual/master/#command-interface-playlist-prev>
    pub const PLAYLIST_PREV: &str = "playlist-prev";
    /// <https://mpv.io/manual/master/#command-interface-playlist-move>
    pub const PLAYLIST_MOVE: &str = "playlist-move";
    /// <https://mpv.io/manual/master/#command-interface-playlist-play-index[preserve-options]>
    pub const PLAYLIST_PLAY_INDEX: &str = "playlist-play-index";
}

pub type PlayerMsg = ICMsg<Cmd, Out, PlayerError>;

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

    /// MPV uses two variables and Strings to determine repeat mode
    ///
    /// See <https://mpv.io/manual/master/#options-loop>
    fn determine_repeat_mode(&self) -> RepeatMode {
        match (self.loop_file.as_str(), self.loop_playlist.as_str()) {
            ("inf", _) => RepeatMode::Song,
            ("no", "inf") => RepeatMode::Queue,
            ("no", "no") | (_, _) => RepeatMode::None,
        }
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

#[derive(Debug, Clone, thiserror::Error)]
pub enum PlayerError {
    #[error("unknown player error")]
    Unknown,
    #[error("The TrackViews StreamURL was None: {0:?}")]
    NoStream(TrackView),
}

impl From<libmpv2::Error> for PlayerError {
    fn from(value: libmpv2::Error) -> Self {
        // TODO: parse the actual mpv error
        Self::Unknown
    }
}

pub fn task_from_error(err: impl Into<PlayerError>) -> Task<PlayerMsg> {
    Task::done(err.into().err_msg())
}

/// Returns Ok(OrmUrl) if the TrackView has a StreamUrl,
/// Err(PlayerError::NoStream(TrackView)) otherwise
pub fn track_url(view: TrackView) -> Result<OrmUrl, PlayerError> {
    if let Some(url) = view.track.stream_url {
        Ok(url)
    } else {
        Err(PlayerError::NoStream(view))
    }
}
