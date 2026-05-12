pub mod app;
pub mod components;
pub mod mpris_server;
pub mod mpv;
pub mod playerbar;
pub mod queue;
pub mod router;
pub mod settings;
pub mod sidebar;

/// This message carries data meant for another component
/// it has to be translated at a pivot point into an IccRecv
pub struct IccSend<Msg>(Msg);
pub struct IccRecv<Msg>(Msg);

impl<Msg> From<IccSend<Msg>> for IccRecv<Msg> {
    fn from(value: IccSend<Msg>) -> Self {
        Self(value.0)
    }
}
