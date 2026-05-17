pub mod app;
pub mod components;
pub mod mpris_server;
pub mod player;
pub mod playerbar;
pub mod queue;
pub mod router;
pub mod sidebar;
pub mod settings;

use std::{
    clone::Clone,
    fmt::{self, Debug},
};

use iced::Task;

// TODO: Do some thinking if this is not completely over-engineered
/// This is an inter-component-message consisting of two variants:
/// - **Command** messages that tell the component what to do
/// - **Out** messages that are meant for an upper component
#[derive(Clone, Debug)]
pub enum ICMsg<Cmd, Out>
where
    Cmd: Clone + fmt::Debug,
    Out: Clone + fmt::Debug,
{
    Cmd(Cmd),
    Out(Out),
}

impl<Cmd, Out> From<Cmd> for ICMsg<Cmd, Out>
where
    Cmd: Clone + Debug,
    Out: Clone + Debug,
{
    /// Since [`Message::Cmd`] is the default variant, this method provides
    /// coercion from any type into that.
    ///
    /// [`Message::Out`] variants need to be created manually \
    /// See [`ToOutMsg::out`]
    fn from(value: Cmd) -> Self {
        ICMsg::Cmd(value)
    }
}

trait ToOutMsg<Cmd, Out>
where
    Cmd: Clone + Debug,
    Out: Clone + Debug,
{
    fn out_msg(self) -> ICMsg<Cmd, Out>;
}

impl<Cmd, Out> ToOutMsg<Cmd, Out> for Out
where
    Cmd: Clone + Debug,
    Out: Clone + Debug,
{
    fn out_msg(self) -> ICMsg<Cmd, Out> {
        ICMsg::Out(self)
    }
}

trait ToCmdMsg<Cmd, Out>
where
    Cmd: Clone + Debug,
    Out: Clone + Debug,
{
    fn cmd_msg(self) -> ICMsg<Cmd, Out>;
}

impl<Cmd, Out> ToCmdMsg<Cmd, Out> for Cmd
where
    Cmd: Clone + Debug,
    Out: Clone + Debug,
{
    fn cmd_msg(self) -> ICMsg<Cmd, Out> {
        ICMsg::Cmd(self)
    }
}

impl<Cmd, Out> ICMsg<Cmd, Out>
where
    Cmd: Clone + Debug,
    Out: Clone + Debug,
{
    /// Processes only [`Message::Cmd`] variants
    /// [`Message::Out`]s produce a Task::none()
    pub fn cmd<T, F>(self, f: F) -> Task<Self>
    where
        T: Into<Task<Self>>,
        F: FnOnce(Cmd) -> T,
    {
        match self {
            ICMsg::Cmd(cmd) => f(cmd).into(),
            ICMsg::Out(_) => Task::none(),
        }
    }

    /// Processes only [`Message::Cmd`] variants
    /// [`Message::Out`]s produce  the default
    pub fn cmd_or<F, U>(self, default: U, f: F) -> U
    where
        F: FnOnce(Cmd) -> U,
    {
        match self {
            ICMsg::Cmd(cmd) => f(cmd),
            ICMsg::Out(_) => default,
        }
    }

    /// Processes only [`Message::Cmd`] variants
    /// [`Message::Out`]s compute the default function
    pub fn cmd_or_else<U, D, F>(self, default: D, f: F) -> U
    where
        D: FnOnce(Out) -> U,
        F: FnOnce(Cmd) -> U,
    {
        match self {
            ICMsg::Cmd(cmd) => f(cmd),
            ICMsg::Out(out) => default(out),
        }
    }

    /// Processes only [`Message::Out`] variants
    /// [`Message::Cmd`]s produce a Task::none()
    pub fn out<T, F>(self, f: F) -> Task<Self>
    where
        T: Into<Task<Self>>,
        F: FnOnce(Out) -> T,
    {
        match self {
            ICMsg::Cmd(_) => Task::none(),
            ICMsg::Out(out) => f(out).into(),
        }
    }

    /// Processes only [`Message::Out`] variants
    /// [`Message::Out`]s produce the default
    pub fn out_or<U, F>(self, default: U, f: F) -> U
    where
        F: FnOnce(Out) -> U,
    {
        match self {
            ICMsg::Cmd(_) => default,
            ICMsg::Out(out) => f(out),
        }
    }

    /// Processes only [`Message::Out`] variants
    /// [`Message::Cmd`]s compute the default function
    pub fn out_or_else<U, D, F>(self, default: D, f: F) -> U
    where
        D: FnOnce(Cmd) -> U,
        F: FnOnce(Out) -> U,
    {
        match self {
            ICMsg::Cmd(cmd) => default(cmd),
            ICMsg::Out(out) => f(out),
        }
    }
}
