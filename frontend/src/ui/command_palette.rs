use backend::api::endpoint_api::{EndpointManager, Pagination, SearchParams, SearchResult};
use iced::{
    Background, Element, Length, Task,
    widget::{self, Id, button, container},
};
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::ui::{
    Receiver,
    command_palette::Message::ChangeContent,
    components::{
        icons,
        search_table::{self, SearchCellMsg, SearchRow, SearchTable},
        table::TableMsg,
    },
    router::{Route, Router, RouterMsg},
};

pub struct CommandPalette {
    visible: bool,
    command: String,
    result: PaletteResults,
    search_result_table: SearchTable,
}

#[derive(Debug, Clone)]
pub enum Message {
    SearchTableDriver(TableMsg<SearchCellMsg>),
    ChangeContent(String),
    Result(PaletteResults),
    Toggle,
    SetActive(bool),
    ChangeRoute(Route),
}

impl Default for CommandPalette {
    fn default() -> Self {
        let search_table = SearchTable::default().add_column(search_table::combined_display());

        Self {
            visible: Default::default(),
            command: String::default(),
            result: PaletteResults::default(),
            search_result_table: search_table,
        }
    }
}

impl CommandPalette {
    /// An Id identifying the pallete input field
    /// we use this to focus it when it becomes visible
    const INPUT_ID: Id = Id::new("palette_input");

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::ChangeContent(new_content) => {
                // Update command
                self.command = new_content;

                // Parse command
                match &self.command {
                    // No input (shows legend)
                    s if s.is_empty() => Task::done(Message::Result(PaletteResults::Empty)),
                    // ?: Search everything
                    s if s.starts_with('?') => {
                        let search_term = s[1..].to_owned();
                        search_everything(search_term)
                    }
                    // @: Go To Page
                    s if s.starts_with('@') => {
                        let page_search = s[1..].to_owned();
                        search_routes(page_search)
                    }
                    // No results
                    _ => Task::done(Message::Result(PaletteResults::NoResults)),
                }
            }
            Message::Result(palette_result) => {
                if let PaletteResults::SearchResults(ref search_results) = palette_result {
                    self.search_result_table.clear();
                    self.search_result_table
                        .extend(search_results.iter().cloned().map(SearchRow::from));
                }
                self.result = palette_result;
                Task::none()
            }
            Message::Toggle => {
                self.visible = !self.visible;
                if self.visible {
                    widget::operation::focus(Self::INPUT_ID)
                } else {
                    Task::none()
                }
            }
            Message::SearchTableDriver(message) => self
                .search_result_table
                .update(message)
                .map(Message::SearchTableDriver),
            Message::ChangeRoute(route) => {
                Router::send(RouterMsg::ChangeRoute(route));
                Task::none()
            }
            Message::SetActive(active) => {
                self.visible = active;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if !self.visible {
            return widget::void().into();
        }

        // Outer full size container
        widget::mouse_area(
            widget::center(
                // Actual palette
                widget::container(widget::column([
                    // Command input
                    widget::text_input("Enter command...", self.command.clone())
                        .on_input(ChangeContent)
                        .id(Self::INPUT_ID)
                        .into(),
                    widget::space().height(8).into(),
                    // Results
                    widget::scrollable(self.view_results()).into(),
                ]))
                .style(container::bordered_box)
                .height(Length::Fit.max(400))
                .width(500),
            )
            .align_top(Length::Fill),
        )
        .on_press(Message::SetActive(false))
        .into()
    }

    fn view_results(&self) -> Element<'_, Message> {
        let results: Element<'_, Message> = match &self.result {
            PaletteResults::Empty => widget::text(
                "?: Search everything\n\
            @: Search pages",
            )
            .into(),
            // Search results
            PaletteResults::SearchResults(_) => self
                .search_result_table
                .view()
                .map(Message::SearchTableDriver),
            // Page results
            PaletteResults::Routes(items) => widget::column(items.iter().map(|route| {
                let (icon, target_route) = match route {
                    SearchableRoute::Home => (icons::home(), Route::Home),
                    SearchableRoute::Favorites => (icons::heart(), Route::Favorites),
                    SearchableRoute::Albums => (icons::disc(), Route::Albums),
                    SearchableRoute::Tracks => (icons::music(), Route::Tracks),
                    SearchableRoute::Artists => (icons::user(), Route::Artists),
                    SearchableRoute::Genres => (icons::tag(), Route::Genres),
                    SearchableRoute::Settings => (icons::settings(), Route::Settings),
                };

                widget::button(
                    widget::row([icon.into(), widget::text(route.to_string()).into()]).spacing(8),
                )
                .on_press(Message::ChangeRoute(target_route))
                .style(move |theme: &iced::Theme, status| {
                    let palette = theme.palette();
                    button::Style {
                        // Make background gray if hovered over
                        background: if matches!(status, button::Status::Hovered) {
                            Some(Background::Color(palette.background.weak.color))
                        } else {
                            None
                        },
                        ..Default::default()
                    }
                })
                .width(Length::Fill)
                .into()
            }))
            .into(),
            PaletteResults::NoResults => widget::text("No results").into(),
        };
        widget::container(results).width(Length::Fill).into()
    }
}

/// Searches
fn search_routes(route_search: String) -> Task<Message> {
    let query = route_search.to_lowercase();
    let routes = SearchableRoute::iter()
        .filter(|route| route.to_string().to_lowercase().contains(&query))
        .collect();

    Task::done(Message::Result(PaletteResults::Routes(routes)))
}

fn search_everything(search_term: String) -> Task<Message> {
    Task::future(async move {
        let endpoint = EndpointManager::get_active_endpoint().await.unwrap();

        endpoint
            .search(
                search_term,
                SearchParams {
                    pagination: Some(Pagination {
                        start_page: 0,
                        limit: 20,
                    }),
                },
            )
            .await
            .unwrap()
    })
    .then(|search_results| {
        Task::done(Message::Result(PaletteResults::SearchResults(
            search_results,
        )))
    })
}

#[derive(Debug, Clone, Display, EnumIter)]
pub enum SearchableRoute {
    Home,
    Favorites,
    Albums,
    Tracks,
    Artists,
    Genres,
    Settings,
}

#[derive(Debug, Clone, Default)]
pub enum PaletteResults {
    #[default]
    Empty,
    SearchResults(Vec<SearchResult>),
    Routes(Vec<SearchableRoute>),
    NoResults,
}
