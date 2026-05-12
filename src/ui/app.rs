use iced::{
    Color, Element, Length::Fill, Subscription, Task, keyboard::{self, Key, key::Named}, widget::{
        column, container,
        pane_grid::{Axis, Configuration, ResizeEvent},
    }
};

use iced::widget::pane_grid;
use url::Url;

use crate::ui::{
    mpv::{self, Player},
    playerbar::{self, PlayerBar},
    queue::{self, Queue},
    router::{self, Router},
    settings::{self, Settings},
    sidebar::{self, Sidebar},
};

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
    Sidebar(sidebar::Message),
    Queue(Box<queue::Message>),
    Router(router::Message),
    PlayerBar(playerbar::Message),
    KeyboardEvent(keyboard::Event),
    PlayerDriver(mpv::Message),
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
            self.player_bar.view().map(Message::PlayerBar).explain(Color::BLACK)
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
                // Pass update router upwards
                sidebar::Message::UpdateRoute(ref route) => Task::batch([
                    self.router
                        .update(router::Message::ChangeRoute(route.clone()))
                        .map(Message::Router),
                    self.sidebar.update(sidebar_message).map(Message::Sidebar),
                ]),
                // Anything else is just sidebar state
                _ => self.sidebar.update(sidebar_message).map(Message::Sidebar),
            },
            Message::Queue(queue_message) => self
                .queue
                .update(*queue_message)
                .map(|m| Message::Queue(Box::new(m))),
            Message::Router(router_message) => {
                self.router.update(router_message).map(Message::Router)
            }
            // Player bar wiring
            Message::PlayerBar(bar_message) => {
                type BMsg = playerbar::Message;
                type PMsg = mpv::Message;

                // Requests coming from the player bar to the player
                match bar_message {
                    BMsg::Pause(p) => self.player.update(PMsg::Pause(p)),
                    BMsg::FinishSeek(d) => {
                        self.player.update(PMsg::Seek(d));
                        // The player bar also needs this
                        self.player_bar.update(bar_message);
                    }
                    BMsg::Next => self.player.update(PMsg::Next),
                    BMsg::Previous => self.player.update(PMsg::Previous),
                    BMsg::Stop => self.player.update(PMsg::Stop),
                    BMsg::Shuffle(s) => self.player.update(PMsg::Shuffle(s)),
                    BMsg::Repeat(r) => self.player.update(PMsg::ChangeRepeatMode(r)),
                    BMsg::PlayRandom => {
                        let url = Url::parse(
                            "file:///mnt/NAS/Samuel/Music/flac/Savant/Alchemist 2/07 - Hungry Eyes.flac",
                        )
                        .unwrap();
                        self.player.update(PMsg::Play(url));
                    }
                    _ => self.player_bar.update(bar_message),
                }
                Task::none()
            }
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
            Message::PlayerDriver(message) => {
                if let mpv::Message::Event(event) = &message {
                    self.player_bar
                        .update(playerbar::Message::Event(event.clone()));
                }
                self.player.update(message);
                Task::none()
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
            Player::subscription().map(Message::PlayerDriver),
        ])
    }
}
