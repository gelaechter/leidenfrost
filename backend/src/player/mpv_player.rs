use libmpv2::{Format, Mpv, events::PropertyData};

use crate::{
    data_view::TrackView,
    db::models::{OrmUrl, Track},
    player::{
        self, Player, PlayerError, PlayerEvent, RepeatMode,
        mpv_player::{command::*, property::*},
    },
};

/// Mpv commands that do things
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

pub struct MpvPlayer(Mpv);

type Result<T> = player::Result<T>;

impl Player for MpvPlayer {
    fn new(on_event: impl Fn(PlayerEvent) + Send + 'static) -> Result<Self> {
        let mpv = Mpv::with_initializer(|init| {
            init.set_option("vid", "no")?;
            Ok(())
        })
        .unwrap();

        // Since [`Mpv::wait_event`] takes `&mut self` we
        // create a second Mpv client just for receiving events
        let event_context = mpv.create_client(Some("EventContext")).unwrap();

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
        // it then converts the mpv events into player events and uses the callback
        std::thread::spawn(move || {
            // For some god-forsaken reason mpv stores the
            // repeat state as strings ("no" and "inf")
            // See <https://mpv.io/manual/master/#options-loop>
            let mut loop_file = String::from("no");
            let mut loop_playlist = String::from("no");

            fn determine_repeat_mode(loop_file: &str, loop_playlist: &str) -> RepeatMode {
                match (loop_file, loop_playlist) {
                    ("inf", _) => RepeatMode::Song,
                    ("no", "inf") => RepeatMode::Queue,
                    ("no", "no") | (_, _) => RepeatMode::None,
                }
            }

            loop {
                let event = event_context.wait_event(f64::MAX);
                if let Some(Ok(event)) = event {
                    // Translate mpv events to `PlayerEvent`s
                    let player_event = match event {
                        libmpv2::events::Event::Shutdown => PlayerEvent::Shutdown,
                        libmpv2::events::Event::PropertyChange { name, change, .. } => {
                            match (name, change) {
                                (SHUFFLE, PropertyData::Flag(b)) => PlayerEvent::Shuffle(b),
                                // The loops need special handling
                                (LOOP_FILE, PropertyData::OsdStr(val)) => {
                                    loop_file = val.to_owned();
                                    PlayerEvent::Repeat(determine_repeat_mode(
                                        &loop_file,
                                        &loop_playlist,
                                    ))
                                }
                                (LOOP_PLAYLIST, PropertyData::OsdStr(val)) => {
                                    loop_playlist = val.to_owned();
                                    PlayerEvent::Repeat(determine_repeat_mode(
                                        &loop_file,
                                        &loop_playlist,
                                    ))
                                }
                                (PAUSE, PropertyData::Flag(p)) => PlayerEvent::Pause(p),
                                (TIME_POS, PropertyData::Double(p)) => PlayerEvent::PlaybackPos(p),
                                (DURATION, PropertyData::Double(d)) => PlayerEvent::Duration(d),
                                (VOLUME, PropertyData::Int64(d)) => {
                                    PlayerEvent::Volume(u32::try_from(d).unwrap())
                                }
                                _ => continue,
                            }
                        }
                        _ => continue,
                    };

                    on_event(player_event);
                }
            }
        });

        // Reset repeat state
        mpv.set_property(LOOP_FILE, "no")?;
        mpv.set_property(LOOP_PLAYLIST, "no")?;

        let mpv_player = Self(mpv);

        Ok(mpv_player)
    }

    fn pause(&self, paused: bool) -> Result<()> {
        self.0.set_property(PAUSE, paused).map_err(Into::into)
    }

    fn seek(&self, pos_in_secs: f64) -> Result<()> {
        self.0
            .command(SEEK, &[&pos_in_secs.to_string(), "absolute"])
            .map_err(Into::into)
    }

    fn stop(&self) -> Result<()> {
        self.0.command(STOP, &[]).map_err(Into::into)
    }

    fn play(&self, view: TrackView) -> Result<()> {
        let url = try_track_url(view)?;

        self.0
            .command(LOADFILE, &[url.as_str()])
            .map_err(Into::into)
    }

    fn play_all(&self, views: Vec<TrackView>) -> Result<()> {
        // Try to play all and return errors for all the unsuccessful queue adds
        // TODO: This iterative approach might get real slow if we have HUGE queues
        for (idx, view) in views.into_iter().enumerate() {
            // Play the first append the others
            if idx == 0 {
                self.play(view)?;
            } else {
                self.append(view)?;
            }
        }

        Ok(())
    }

    fn play_index(&self, index: usize) -> Result<()> {
        self.0
            .command(PLAYLIST_PLAY_INDEX, &[&index.to_string()])
            .map_err(Into::into)
    }

    fn append(&self, view: TrackView) -> Result<()> {
        let url = try_track_url(view)?;

        self.0
            .command(LOADFILE, &["append", url.as_str()])
            .map_err(Into::into)
    }

    fn append_all(&self, tracks: Vec<TrackView>) -> Result<()> {
        for track in tracks {
            self.append(track)?;
        }

        Ok(())
    }

    fn queue_remove(&self, index: usize) -> Result<()> {
        self.0
            .command(PLAYLIST_REMOVE, &[&index.to_string()])
            .map_err(Into::into)
    }

    fn queue_move(&self, target: usize, position: usize) -> Result<()> {
        self.0
            .command(PLAYLIST_MOVE, &[&target.to_string(), &position.to_string()])
            .map_err(Into::into)
    }

    fn set_shuffle(&self, shuffle: bool) -> Result<()> {
        self.0.set_property(SHUFFLE, shuffle).map_err(Into::into)
    }

    fn change_repeat_mode(&self, mode: RepeatMode) -> Result<()> {
        let (loop_file, loop_playlist) = match mode {
            RepeatMode::None => ("no", "no"),
            RepeatMode::Song => ("inf", "no"),
            RepeatMode::Queue => ("no", "inf"),
        };

        self.0.set_property(LOOP_FILE, loop_file)?;
        self.0.set_property(LOOP_PLAYLIST, loop_playlist)?;

        Ok(())
    }

    fn next(&self) -> Result<()> {
        self.0.command(PLAYLIST_NEXT, &[]).map_err(Into::into)
    }

    fn previous(&self) -> Result<()> {
        self.0.command(PLAYLIST_PREV, &[]).map_err(Into::into)
    }

    fn volume(&self, percentage: u32) -> Result<()> {
        self.0
            .set_property(VOLUME, i64::from(percentage))
            .map_err(Into::into)
    }
}

impl From<libmpv2::Error> for PlayerError {
    fn from(value: libmpv2::Error) -> Self {
        // TODO: parse the actual mpv error
        Self::Unknown
    }
}

fn try_track_url(track_view: TrackView) -> Result<OrmUrl> {
    if let TrackView {
        track: Track {
            stream_url: Some(url),
            ..
        },
        ..
    } = track_view
    {
        Ok(url)
    } else {
        Err(PlayerError::NoStream(Box::new(track_view)))
    }
}
