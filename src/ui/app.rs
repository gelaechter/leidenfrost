use iced::{
    Element,
    Length::Fill,
    Subscription, Task,
    keyboard::{self, Key, key::Named},
    message::MaybeClone,
    widget::{
        column, container,
        pane_grid::{Axis, Configuration, ResizeEvent},
    },
};

use iced::widget::pane_grid;

use crate::{
    backend::api::endpoint_api::Endpoint,
    ui::{
        player::{Player, PlayerMessage},
        queue::{Queue, QueueMessage},
        router::{self, Router},
        sidebar::{Message, Sidebar},
    },
};

pub struct App {
    panes: pane_grid::State<Pane>,
    settings: Settings,
    sidebar: Sidebar,
    router: Router,
    queue: Queue,
    player_bar: Player,
}

#[derive(Default)]
pub struct Settings {
    selected_apis: Vec<Endpoint>,
    debug_overlay: bool,
}

impl Default for App {
    fn default() -> Self {
        let sidebar = Sidebar::default();
        let router = Router::default();
        let queue = Queue::default();
        let player_bar = Player::default();
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
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),
    Sidebar(Message),
    Queue(QueueMessage),
    Router(router::Message),
    PlayerBar(PlayerMessage),
    KeyboardEvent(keyboard::Event),
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
                    Pane::Queue => self.queue.view().map(Message::Queue),
                    Pane::Router => self.router.view().map(Message::Router),
                })
            })
            .on_drag(Message::PaneDragged)
            .on_resize(10, Message::PaneResized),
            // the player bar
            self.player_bar.view().map(Message::PlayerBar)
        ])
        .height(Fill)
        .width(Fill)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PaneDragged(drag_event) => todo!(),
            Message::PaneResized(ResizeEvent { split, ratio }) => {
                self.panes.resize(split, ratio);
                Task::none()
            }
            Message::Sidebar(sidebar_message) => {
                // Currently irrefutable
                self.sidebar.update(sidebar_message.clone());
                if let Message::UpdateRoute(route) = sidebar_message {
                    self.router
                        .update(router::Message::ChangeRoute(route))
                        .map(Message::Router)
                } else {
                    Task::none()
                }
            }
            Message::Queue(queue_message) => {
                self.queue.update(queue_message);
                Task::none()
            }
            Message::Router(router_message) => {
                self.router.update(router_message).map(Message::Router)
            }
            Message::PlayerBar(player_message) => {
                self.player_bar.update(player_message);
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
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().map(Message::KeyboardEvent)
    }
}
