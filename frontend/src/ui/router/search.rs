use iced::{
    Element, Task,
    widget::{self},
};

use backend::api::{
    endpoint_api::{EndpointManager, Pagination, SearchParams, SearchResult},
    jellyfin::errors::ApiError,
};

use crate::ui::{app::AppState, components::{
    search_table::{SearchRow, SearchTable, SearchTableMsg},
    style::header_text,
}};

/// The searchs route
#[derive(Default)]
pub struct SearchPage {
    /// The searchs as displayed in the table
    search_table: SearchTable,
    query: String,
    /// An error display
    error: Option<ApiError>,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// The search term has changed
    SearchChanged(String),
    /// The search results have been fetched
    SearchResult(Vec<SearchResult>),
    /// A driver for the table
    TableDriver(Box<SearchTableMsg>),
}

pub type SearchMsg = Message;

impl SearchPage {
    pub fn view<'a>(&'a self, app_state: &'a AppState) -> Element<'a, Message> {
        let header = header_text("Search");

        let search_bar =
            widget::text_input("search anything", &self.query).on_input(Message::SearchChanged);

        widget::column([
            header.into(),
            search_bar.into(),
            self.search_table
                .view(app_state)
                .map(|c| Message::TableDriver(Box::new(c))),
        ])
        .into()
    }

    pub fn update(&mut self, message: Message, app_state: &mut AppState) -> Task<Message> {
        match message {
            Message::SearchChanged(query) => {
                self.query.clone_from(&query);

                Task::perform(
                    async {
                        let endpoint = EndpointManager::get_active_endpoint().await.unwrap();

                        endpoint
                            .search(
                                query,
                                SearchParams {
                                    pagination: Some(Pagination {
                                        start_page: 0,
                                        limit: 100,
                                    }),
                                },
                            )
                            // TODO: error handling
                            .await
                            .unwrap()
                    },
                    Message::SearchResult,
                )
            }
            Message::SearchResult(searchs) => {
                // After fetching convert the searches into SearchRows
                let rows: Vec<SearchRow> = searchs.into_iter().map(SearchRow::from).collect();
                self.search_table.clear();
                self.search_table.extend(rows);
                Task::none()
            }
            Message::TableDriver(m) => self
                .search_table
                .update(*m, app_state)
                .map(|c| Message::TableDriver(Box::new(c))),
        }
    }
}
