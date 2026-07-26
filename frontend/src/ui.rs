pub mod app;
pub mod components;
pub mod mpris_server;
pub mod player;
pub mod playerbar;
pub mod queue;
pub mod router;
pub mod sidebar;

use std::{
    clone::Clone,
    convert::Infallible,
    fmt::{self, Debug},
};

use iced::{Task, message::MaybeDebug};

// TODO: Do some thinking if this is not completely over-engineered
/// This is an inter-component-message consisting of two variants:
/// - **Command** messages that tell the component what to do
/// - **Out** messages that are meant for an upper component
#[derive(Clone, Debug)]
pub enum ICMsg<Cmd, Out, Err = Infallible>
where
    Cmd: Clone + fmt::Debug,
    Out: Clone + fmt::Debug,
    Err: std::error::Error,
{
    Cmd(Cmd),
    Out(Out),
    Err(Err),
}

impl<Cmd, Out, Err> From<Cmd> for ICMsg<Cmd, Out, Err>
where
    Cmd: Clone + Debug,
    Out: Clone + Debug,
    Err: std::error::Error,
{
    /// [`ICMsg::Cmd`] is the default variant because our components always
    /// expect to receive their instructions via Cmd. Out/Error variants are
    /// meant for consumption by another component meaning they will be at some
    /// point converted to a Cmd as well.
    ///
    /// This method provides coercion from any type into that.
    ///
    /// The [`ICMsg::Cmd`] and [`ICMsg::Err`] variants need to be created
    /// manually \ See [`ToOutMsg::out_msg`] and [`ToErrMsg::err_msg`]
    fn from(value: Cmd) -> Self {
        ICMsg::Cmd(value)
    }
}

trait ToOutMsg<Cmd, Out, Err>
where
    Cmd: Clone + Debug,
    Out: Clone + Debug,
    Err: std::error::Error,
{
    fn out_msg(self) -> ICMsg<Cmd, Out, Err>;
}

impl<Cmd, Out, Err> ToOutMsg<Cmd, Out, Err> for Out
where
    Cmd: Clone + Debug,
    Out: Clone + Debug,
    Err: std::error::Error,
{
    fn out_msg(self) -> ICMsg<Cmd, Out, Err> {
        ICMsg::Out(self)
    }
}

trait ToCmdMsg<Cmd, Out, Err>
where
    Cmd: Clone + Debug,
    Out: Clone + Debug,
    Err: std::error::Error,
{
    fn cmd_msg(self) -> ICMsg<Cmd, Out, Err>;
}

impl<Cmd, Out, Err> ToCmdMsg<Cmd, Out, Err> for Cmd
where
    Cmd: Clone + Debug,
    Out: Clone + Debug,
    Err: std::error::Error,
{
    fn cmd_msg(self) -> ICMsg<Cmd, Out, Err> {
        ICMsg::Cmd(self)
    }
}

/// Helper trait which allows wrapping arbitrary values in [`ICMsg::Err`]
trait ToErrMsg<Cmd, Out, Err>
where
    Cmd: Clone + Debug,
    Out: Clone + Debug,
    Err: std::error::Error,
{
    fn err_msg(self) -> ICMsg<Cmd, Out, Err>;
}

impl<Cmd, Out, Err> ToErrMsg<Cmd, Out, Err> for Err
where
    Cmd: Clone + Debug,
    Out: Clone + Debug,
    Err: std::error::Error,
{
    /// Helper function which wraps arbitrary values in [`ICMsg::Err`]
    fn err_msg(self) -> ICMsg<Cmd, Out, Err> {
        ICMsg::Err(self)
    }
}

impl<Cmd, Out, Err> ICMsg<Cmd, Out, Err>
where
    Cmd: Clone + Debug,
    Out: Clone + Debug,
    Err: std::error::Error,
{
    /// Processes only [`ICMsg::Cmd`] variants
    /// Everything else (Out/Err) results in [`Task::none`]
    pub fn cmd<T, F>(self, f: F) -> Task<Self>
    where
        T: Into<Task<Self>>,
        F: FnOnce(Cmd) -> T,
    {
        match self {
            ICMsg::Cmd(cmd) => f(cmd).into(),
            _ => Task::none(),
        }
    }
}
