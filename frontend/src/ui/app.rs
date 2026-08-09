//! The App is the top level model in leidenfrost

use backend::player::{Player, PlayerError};
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
    ICMsg,
    player::{self, GenericPlayer, PlayerMsg},
    playerbar::{self, PlayerBar, PlayerBarMsg},
    queue::{self, Queue},
    router::{self, Router, RouterMsg},
    sidebar::{self, Sidebar, SidebarMsg},
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

impl App {
    pub fn new() -> Self {
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

#[derive(Debug, Clone)]
pub enum Message {
    PaneResized(pane_grid::ResizeEvent),
    Sidebar(SidebarMsg),
    Queue(Box<queue::Message>),
    Router(RouterMsg),
    PlayerBar(PlayerBarMsg),
    KeyboardEvent(keyboard::Event),
    Player(PlayerMsg),
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
            Message::Sidebar(sidebar_message) => match sidebar_message {
                ICMsg::Cmd(cmd) => self.sidebar.update(cmd).map(Message::Sidebar),
                ICMsg::Out(out) => match out {
                    // Pass update router upwards
                    sidebar::Out::UpdateRoute(ref route) => Task::batch([self
                        .router
                        .update(router::Cmd::ChangeRoute(route.clone()))
                        .map(Message::Router)]),
                },
            },
            Message::Queue(queue_message) => self
                .queue
                .update(*queue_message)
                .map(|m| Message::Queue(Box::new(m))),
            Message::Router(message) => match message {
                ICMsg::Cmd(cmd) => self.router.update(cmd).map(Message::Router),
                ICMsg::Out(out) => match out {
                    router::Out::ChangeRoute(route) => {
                        // The sidebar needs to be notified so it can display the current route
                        self.sidebar
                            .update(sidebar::Cmd::RouteChanged(route))
                            .map(Message::Sidebar)
                    }
                    router::Out::TracksMsg(out) => {
                        // Tracks has emitted an out message
                        match out {
                            // It wants to play a track
                            router::tracks::Out::PlayTrack { tracks, index } => {
                                dbg!("Playtrack triggered");
                                Task::batch([
                                    self.player // Play all
                                        .play_all(tracks)
                                        .map(Message::Player),
                                    self.player // Then set the index
                                        .play_index(index)
                                        .map(Message::Player),
                                ])
                            }
                        }
                    }
                },
            },
            // Player bar wiring
            Message::PlayerBar(message) => match message {
                ICMsg::Cmd(cmd) => self.player_bar.update(cmd).map(Message::PlayerBar),
                ICMsg::Out(out) => {
                    type Bar = playerbar::Out;
                    type Player = player::Cmd;

                    fn err_to_task(res: Result<(), PlayerError>) {}

                    match out {
                        // Forward messages for the player
                        Bar::Pause(p) => self.player.pause(p).into(),
                        Bar::Seek(d) => self.player.seek(d).into(),
                        Bar::Next => self.player.next().into(),
                        Bar::Previous => self.player.previous().into(),
                        Bar::Stop => self.player.stop().into(),
                        Bar::Shuffle(s) => self.player.set_shuffle(s).into(),
                        Bar::Repeat(r) => self.player.change_repeat_mode(r).into(),
                        Bar::PlayRandom => Task::done(todo!("TODO:")),
                        Bar::Volume(v) => self.player.volume(v).into(),
                        // Forward change route (from clicking links)
                        Bar::ChangeRoute(route) => self
                            .router
                            .update(router::Cmd::ChangeRoute(route))
                            .map(Message::Router),
                    }
                }
            },
            Message::KeyboardEvent(event) => match event {
                keyboard::Event::KeyPressed {
                    key: Key::Named(Named::F11),
                    ..
                } => Task::none(),
                _ => Task::none(),
            },
            Message::Player(message) => {
                match message {
                    ICMsg::Cmd(cmd) => self.player.update(cmd).map(Message::Player),
                    ICMsg::Out(out) => match out {
                        player::Out::Event(player_event) => self
                            .player_bar
                            .update(playerbar::Cmd::Event(player_event))
                            .map(Message::PlayerBar),
                        // Handle queue updates that should reflect the actual player queue
                        player::Out::Queue(queue_event) => match *queue_event {
                            player::QueueEvent::Append(track_view) => self
                                .queue
                                .update(queue::Message::Append(track_view))
                                .map(|m| Message::Queue(Box::new(m))),
                        },
                    },
                    // TODO: actually handle errors
                    ICMsg::Err(e) => todo!(),
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            keyboard::listen().map(Message::KeyboardEvent),
            GenericPlayer::subscription().map(Message::Player),
        ])
    }

    pub fn theme(&self) -> Option<Theme> {
        self.router.settings.theme.clone()
    }
}
