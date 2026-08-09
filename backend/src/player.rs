pub mod mpv_player;

use tokio::sync::broadcast;

use crate::data_view::TrackView;

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

/// Events emitted by a player to
#[derive(Clone, Debug)]
pub enum PlayerEvent {
    /// The player instance has shut down
    Shutdown,
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
    /// An error has occured
    Error(PlayerError),
}

pub type Result<T> = std::result::Result<T, PlayerError>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum PlayerError {
    #[error("unknown player error")]
    Unknown,
    #[error("The TrackViews StreamURL was None: {0:?}")]
    NoStream(Box<TrackView>),
}

pub trait Player: Sized {
    /// Constructs a new player with a callback for events.
    fn new(on_event: impl Fn(PlayerEvent) + Send + 'static) -> Result<Self>;
    /// (Un)pauses the player
    fn pause(&self, paused: bool) -> Result<()>;
    /// Seek to a specific time in seconds
    fn seek(&self, position: f64) -> Result<()>;
    /// Stop playback and clear playlist.
    fn stop(&self) -> Result<()>;
    /// Plays a track, should return an `Out::Queue` task
    fn play(&self, track: TrackView) -> Result<()>;
    /// Adds multiple tracks and plays the first
    fn play_all(&self, tracks: Vec<TrackView>) -> Result<()>;
    /// Plays a certain index in the queue
    fn play_index(&self, index: usize) -> Result<()>;
    fn append(&self, track: TrackView) -> Result<()>;
    fn append_all(&self, tracks: Vec<TrackView>) -> Result<()>;
    /// Remove a track from the queue
    fn queue_remove(&self, index: usize) -> Result<()>;
    /// Move a track in the queue
    ///
    /// Moves a target track in the queue before the position of another one
    fn queue_move(&self, target: usize, position: usize) -> Result<()>;
    fn set_shuffle(&self, shuffle: bool) -> Result<()>;
    /// Changes the repeat mode of the player
    fn change_repeat_mode(&self, mode: RepeatMode) -> Result<()>;
    /// Plays the next song
    fn next(&self) -> Result<()>;
    /// Plays the previous song
    fn previous(&self) -> Result<()>;
    /// Sets the volume (0-100)
    fn volume(&self, percentage: u32) -> Result<()>;
}
