//! The App is the top level model in leidenfrost

use iced::{
    Color, Element,
    Length::Fill,
    Subscription, Task,
    keyboard::{self, Key, key::Named},
    widget::{
        column, container,
        pane_grid::{Axis, Configuration, ResizeEvent},
    },
};

use iced::widget::pane_grid;
use url::Url;

use crate::ui::{
    ICMsg,
    player::{self, Player, PlayerMsg},
    playerbar::{self, PlayerBar, PlayerBarMsg},
    queue::{self, Queue},
    router::{self, Router},
    settings::{self, Settings},
    sidebar::{self, Sidebar, SidebarMsg},
};

/// App is the top level model in this application
pub struct App {
    panes: pane_grid::State<Pane>,
    settings: Settings,
    sidebar: Sidebar,
    router: Router,
    queue: Queue,
    player: Player,
    player_bar: PlayerBar,
}

impl Default for App {
    fn default() -> Self {
        let sidebar = Sidebar::default();
        let router = Router::default();
        let queue = Queue::default();
        let player = Player::default();
        let player_bar = PlayerBar::default();
        let settings = Settings::default();

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
            settings,
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
    Router(router::Message),
    PlayerBar(PlayerBarMsg),
    KeyboardEvent(keyboard::Event),
    Player(PlayerMsg),
    Settings(settings::Message),
}

/// The main layout of the application
impl App {
    pub fn view(&self) -> Element<'_, Message> {
        // Enclosing container
        container(column![
            // Pane grid consisting of sidebar, main window (router) and queue
            pane_grid(&self.panes, |pane, state, is_maximized| {
                pane_grid::Content::new(match state {
                    Pane::Sidebar => self.sidebar.view().map(Message::Sidebar),
                    Pane::Queue => self.queue.view().map(|m| Message::Queue(Box::new(m))),
                    Pane::Router => self.router.view().map(Message::Router),
                })
            })
            .on_resize(10, Message::PaneResized),
            // The player bar
            self.player_bar
                .view()
                .map(Message::PlayerBar)
                .explain(Color::BLACK)
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
                        .update(router::Message::ChangeRoute(route.clone()))
                        .map(Message::Router)]),
                },
            },
            Message::Queue(queue_message) => self
                .queue
                .update(*queue_message)
                .map(|m| Message::Queue(Box::new(m))),
            Message::Router(router_message) => {
                self.router.update(router_message).map(Message::Router)
            }
            // Player bar wiring
            Message::PlayerBar(message) => match message {
                ICMsg::Cmd(cmd) => self.player_bar.update(cmd).map(Message::PlayerBar),
                ICMsg::Out(out) => {
                    type Bar = playerbar::Out;
                    type Player = player::Cmd;

                    match out {
                        // Forward messages for the player
                        Bar::Pause(p) => self.player.update(Player::Pause(p)).map(Message::Player),
                        Bar::Seek(d) => self.player.update(Player::Seek(d)).map(Message::Player),
                        Bar::Next => self.player.update(Player::Next).map(Message::Player),
                        Bar::Previous => self.player.update(Player::Previous).map(Message::Player),
                        Bar::Stop => self.player.update(Player::Stop).map(Message::Player),
                        Bar::Shuffle(s) => {
                            self.player.update(Player::Shuffle(s)).map(Message::Player)
                        }
                        Bar::Repeat(r) => self
                            .player
                            .update(Player::ChangeRepeatMode(r))
                            .map(Message::Player),
                        Bar::PlayRandom => {
                            // Currently used for testing
                            let url = Url::parse(
                                "file:///mnt/NAS/Samuel/Music/flac/Savant/Alchemist 2/07 - Hungry Eyes.flac",
                            )
                            .unwrap();
                            self.player.update(Player::Play(url)).map(Message::Player)
                        }
                        Bar::Volume(v) => {
                            self.player.update(Player::Volume(v)).map(Message::Player)
                        }
                        // Forward change route (from clicking links)
                        Bar::ChangeRoute(route) => self
                            .router
                            .update(router::Message::ChangeRoute(route))
                            .map(Message::Router),
                    }
                }
            },
            Message::KeyboardEvent(event) => match event {
                keyboard::Event::KeyPressed {
                    key: Key::Named(Named::F11),
                    ..
                } => {
                    self.settings.debug_overlay = !self.settings.debug_overlay;
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::Player(message) => {
                dbg!(&message);
                match message {
                    ICMsg::Cmd(cmd) => self.player.update(cmd).map(Message::Player),
                    ICMsg::Out(out) => {
                        match out {
                            player::Out::Event(player_event) => self
                                .player_bar
                                .update(playerbar::Cmd::Event(player_event))
                                .map(Message::PlayerBar),
                            // TODO: actually handle errors
                            player::Out::Error(player_error) => Task::none(),
                        }
                    }
                }
            }
            Message::Settings(message) => {
                self.settings.update(message);
                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            keyboard::listen().map(Message::KeyboardEvent),
            Player::subscription().map(Message::Player),
        ])
    }
}
