//! The App is the top level model in leidenfrost

use iced::{
    Element,
    Length::Fill,
    Subscription, Task, Theme,
    keyboard::{self, Key, key::Named},
    widget::{
        column, container,
        pane_grid::{Axis, Configuration, ResizeEvent},
    },
};

use iced::widget::pane_grid;

use crate::ui::{
    ReceiverContainer,
    player::{GenericPlayer, PlayerMsg},
    playerbar::{PlayerBar, PlayerBarMsg},
    queue::{self, Queue, QueueMsg},
    router::{Router, RouterMsg, settings::SettingsMsg},
    sidebar::{Sidebar, SidebarMsg},
};

/// App is the top level model in this application
pub struct App {
    /// These panes are the main layout of the app. They show:
    /// - the sidebar (containing navigation)
    /// - the main window (containing the router/routes)
    /// - the queue (containing the next queued items)
    panes: pane_grid::State<Pane>,
    sidebar: Sidebar,
    router: Router,
    queue: Queue,
    player: GenericPlayer,
    player_bar: PlayerBar,
}

impl Default for App {
    fn default() -> Self {
        let sidebar = Sidebar::default();
        let router = Router::default();
        let queue = Queue::default();
        let player = GenericPlayer::default();
        let player_bar = PlayerBar::default();

        // Creates a new pane state and immediately splits it
        let panes = pane_grid::State::<Pane>::with_configuration(Configuration::Split {
            axis: Axis::Vertical,
            ratio: 0.8,
            a: Box::new(Configuration::Split {
                axis: Axis::Vertical,
                ratio: 0.2,
                a: Box::new(Configuration::Pane(Pane::Sidebar)),
                b: Box::new(Configuration::Pane(Pane::Router)),
            }),
            b: Box::new(Configuration::Pane(Pane::Queue)),
        });

        Self {
            panes,
            sidebar,
            router,
            queue,
            player,
            player_bar,
        }
    }
}

pub enum Pane {
    /// The sidebar pane on the left
    Sidebar,
    /// The queue on the right
    Queue,
    /// The main window in the middle
    Router,
}

/// The top level [`Message`] works as a pivot for messages received by
/// [`super::Receiver`]. Hence it must hold Variants for each message type which
/// has a receiver.
///
/// The Receivers message type is expected to implement `Into::<Message>::into`.
#[derive(Debug, Clone)]
pub enum Message {
    PaneResized(pane_grid::ResizeEvent),
    Sidebar(SidebarMsg),
    Queue(Box<queue::QueueMsg>),
    Router(RouterMsg),
    KeyboardEvent(keyboard::Event),
    Player(Box<PlayerMsg>),
    PlayerBar(PlayerBarMsg),
    Settings(SettingsMsg),
}

impl From<PlayerMsg> for Message {
    fn from(value: PlayerMsg) -> Self {
        Self::Player(Box::new(value))
    }
}

impl From<PlayerBarMsg> for Message {
    fn from(value: PlayerBarMsg) -> Self {
        Self::PlayerBar(value)
    }
}

impl From<SettingsMsg> for Message {
    fn from(value: SettingsMsg) -> Self {
        Self::Settings(value)
    }
}

impl From<RouterMsg> for Message {
    fn from(value: RouterMsg) -> Self {
        Self::Router(value)
    }
}

impl From<SidebarMsg> for Message {
    fn from(value: SidebarMsg) -> Self {
        Self::Sidebar(value)
    }
}

impl From<QueueMsg> for Message {
    fn from(value: QueueMsg) -> Self {
        Self::Queue(Box::new(value))
    }
}

/// The main layout of the application
impl App {
    pub fn view(&self) -> Element<'_, Message> {
        // Enclosing container
        container(column![
            // Pane grid consisting of sidebar, main window (router) and queue
            pane_grid(&self.panes, |_pane, state, _is_maximized| {
                pane_grid::Content::new(match state {
                    Pane::Sidebar => self.sidebar.view().map(Message::Sidebar),
                    Pane::Queue => self.queue.view().map(|m| Message::Queue(Box::new(m))),
                    // The router view s
                    Pane::Router => self.router.view().map(Message::Router),
                })
            })
            .on_resize(10, Message::PaneResized),
            // The player bar
            self.player_bar.view().map(Message::PlayerBar)
        ])
        .height(Fill)
        .width(Fill)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PaneResized(ResizeEvent { split, ratio }) => {
                self.panes.resize(split, ratio);
                Task::none()
            }
            Message::Sidebar(sidebar_message) => {
                self.sidebar.update(sidebar_message).map(Message::Sidebar)
            }
            Message::Queue(queue_message) => self
                .queue
                .update(*queue_message)
                .map(|m| Message::Queue(Box::new(m))),
            Message::Router(message) => self.router.update(message).map(Message::Router),
            Message::PlayerBar(message) => self.player_bar.update(message).map(Message::PlayerBar),
            Message::KeyboardEvent(event) => match event {
                keyboard::Event::KeyPressed {
                    key: Key::Named(Named::F11),
                    ..
                } => Task::none(),
                _ => Task::none(),
            },
            Message::Player(message) => {
                self.player.update(*message);
                Task::none()
            }
            Message::Settings(cmd) => self.router.settings.update(cmd).map(Message::Settings),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Register all the event receivers
        let wa = inventory::iter::<ReceiverContainer>;
        let mut subscriptions: Vec<Subscription<Message>> =
            wa.into_iter().map(|s| (s.0)()).collect();

        subscriptions.extend([
            // Additional subscriptions
            keyboard::listen().map(Message::KeyboardEvent),
            GenericPlayer::subscription().map(|m| Message::Player(Box::new(m))),
        ]);

        Subscription::batch(subscriptions)
    }

    pub fn theme(&self) -> Option<Theme> {
        self.router.settings.theme.clone()
    }
}
