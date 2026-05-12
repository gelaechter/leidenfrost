//! Contains the logic for the player that actually plays the audio
//! At the moment this uses MPV as the backend as it I deem it highly reliable
//! and feature-complete

use std::sync::LazyLock;

use iced::Subscription;
use iced::futures::Stream;
use libmpv2::{Format, Mpv};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use url::Url;

use crate::{
    backend::mpv_events::MpvEvent,
    ui::mpv::{
        command::{
            LOADFILE, PLAYLIST_MOVE, PLAYLIST_NEXT, PLAYLIST_PREV, PLAYLIST_REMOVE, SEEK, STOP,
        },
        property::{DURATION, LOOP_FILE, LOOP_PLAYLIST, PAUSE, SHUFFLE, TIME_POS},
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

pub struct Player {
    /// the Mpv instance
    mpv: Mpv,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            mpv: initialize_mpv(),
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

    // Start a thread that continually checks the event queue
    std::thread::spawn(move || {
        loop {
            let event = event_context.wait_event(f64::MAX);
            if let Some(Ok(event)) = event {
                EVENT_CHANNEL.send(event.into());
            }
        }
    });

    // Reset state
    mpv.set_property(LOOP_FILE, "no");
    mpv.set_property(LOOP_PLAYLIST, "no");

    mpv
}

#[derive(Clone, Debug)]
pub enum Message {
    /// (Un)pauses the player
    Pause(bool),
    /// Seek to a specific time in seconds
    Seek(f64),
    /// Stop playback and clear playlist.
    Stop,
    /// Clears the queue and plays just that track
    Play(Url),
    PlayAll(Vec<Url>),
    /// Adds some tracks to the queue
    Append(Url),
    AppendAll(Vec<Url>),
    /// Remove a track from the queue
    QueueRemove(usize),
    /// Move a track in the queue
    ///
    /// Moves a target track in the queue before the position of another one
    QueueMove {
        target: usize,
        position: usize,
    },
    Shuffle(bool),
    /// Changes the repeat mode of the player
    ChangeRepeatMode(RepeatMode),
    /// Plays the next song
    Next,
    /// Plays the previous song
    Previous,
    /// An event occurred
    Event(MpvEvent),
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
}

// TODO: Since player doesn't implement view we technically don't need to use ELM
// We could instead just have a function for each message which might declutter things.
impl Player {
    pub fn update(&mut self, message: Message) {
        let res = match message {
            Message::Pause(pause) => self.mpv.set_property(PAUSE, pause),
            Message::Seek(seconds) => self.mpv.command(SEEK, &[&seconds.to_string(), "absolute"]),
            Message::Play(url) => self.mpv.command(LOADFILE, &[url.as_str()]),
            Message::PlayAll(urls) => {
                // Play the first append the others
                for (index, ele) in urls.into_iter().enumerate() {
                    if index == 0 {
                        self.update(Message::Play(ele));
                    } else {
                        self.update(Message::Append(ele));
                    }
                }
                Ok(())
            }
            Message::Append(url) => self.mpv.command(LOADFILE, &["append", url.as_str()]),
            Message::AppendAll(urls) => {
                for url in urls {
                    self.update(Message::Append(url));
                }
                Ok(())
            }
            Message::Stop => self.mpv.command(STOP, &[]),
            Message::QueueRemove(usize) => self.mpv.command(PLAYLIST_REMOVE, &[&usize.to_string()]),
            Message::Next => self.mpv.command(PLAYLIST_NEXT, &[]),
            Message::Previous => self.mpv.command(PLAYLIST_PREV, &[]),
            Message::Event(event) => Ok(()),
            Message::QueueMove {
                target: before,
                position: after,
            } => self
                .mpv
                .command(PLAYLIST_MOVE, &[&before.to_string(), &after.to_string()]),
            Message::ChangeRepeatMode(mode) => match mode {
                RepeatMode::None => {
                    self.mpv.set_property(LOOP_FILE, "no");
                    self.mpv.set_property(LOOP_PLAYLIST, "no")
                }
                RepeatMode::Song => {
                    self.mpv.set_property(LOOP_FILE, "inf");
                    self.mpv.set_property(LOOP_PLAYLIST, "no")
                }
                RepeatMode::Queue => {
                    self.mpv.set_property(LOOP_FILE, "no");
                    self.mpv.set_property(LOOP_PLAYLIST, "inf")
                }
            },
            Message::Shuffle(shuffle) => self.mpv.set_property(SHUFFLE, shuffle),
        };
    }

    pub fn subscription() -> Subscription<Message> {
        fn subscribe_mpv_events() -> impl Stream<Item = MpvEvent> {
            use tokio_stream::StreamExt;

            let stream = EVENT_CHANNEL.subscribe();
            BroadcastStream::new(stream).filter_map(Result::ok)
        }

        Subscription::run(subscribe_mpv_events).map(Message::Event)
    }
}
