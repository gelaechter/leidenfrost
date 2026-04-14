use std::collections::{HashMap, HashSet};

use iced::{
    Alignment::Center,
    Border, Color, Element, Font,
    Length::{Fill, FillPortion, Shrink},
    Padding, Task, Theme, font,
    widget::{
        self,
        button::{self, Status},
        column,
        pane_grid::{self, Axis, Configuration},
        row, sensor, space,
        text::{self, Wrapping},
    },
};
use iced_fonts::lucide;
use url::Url;
use uuid::Uuid;

use crate::{
    backend::{
        api::{
            endpoint_api::{ApiContract, UserPasswordAuth},
            jellyfin::api::JellyfinApi,
        },
        data_view::{RelatedArtist, RelatedGenre, TrackView},
        db::models::Track,
    },
    ui::{
        components::{
            image::{self, Image},
            utils::{IntoLink, IntoLinks},
        },
        router::Route,
        util::format_duration,
    },
};

/// The tracks route
pub struct Tracks {
    /// The tracks as displayed in the table
    tracks: Vec<TrackView>,
    /// Tracks have been selected in the table
    selected_tracks: HashSet<String>,
    /// Tracks that are currently visible in the table
    track_visibility: HashMap<String, bool>,
    table_header: pane_grid::State<TableColumns>,
    header_width: HashMap<TableColumns, f32>,
    images: HashMap<Uuid, Image>,
}

impl Default for Tracks {
    fn default() -> Self {
        Self {
            tracks: Default::default(),
            selected_tracks: Default::default(),
            track_visibility: Default::default(),
            // TODO: Replace with default and dynamically add the other columns
            table_header: {
                pane_grid::State::with_configuration(Configuration::Split {
                    axis: Axis::Vertical,
                    ratio: 0.2,
                    a: Box::new(Configuration::Split {
                        axis: Axis::Vertical,
                        ratio: 0.5,
                        a: Box::new(Configuration::Pane(TableColumns::Number)),
                        b: Box::new(Configuration::Pane(TableColumns::CombinedTitle)),
                    }),
                    b: Box::new(Configuration::Split {
                        axis: Axis::Vertical,
                        ratio: 0.5,
                        a: Box::new(Configuration::Pane(TableColumns::Duration)),
                        b: Box::new(Configuration::Split {
                            axis: Axis::Vertical,
                            ratio: 0.5,
                            a: Box::new(Configuration::Pane(TableColumns::Album)),
                            b: Box::new(Configuration::Pane(TableColumns::Genre)),
                        }),
                    }),
                })
            },
            images: Default::default(),
            header_width: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TableColumns {
    Number,
    CombinedTitle,
    Duration,
    Album,
    Genre,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Initial track fetching
    FetchTracks,
    /// The tracks have been fetched
    TracksFetched(Vec<TrackView>),
    /// A track has scrolled into view
    TrackShown(String),
    /// A track has left the view
    TrackHidden(String),
    /// A track row has been clicked
    TrackClicked(String),
    ColumnResized(TableColumns, f32),
    /// A table column has been resized
    PaneResized(pane_grid::ResizeEvent),
    /// A table column has been reordered
    PaneReordered(pane_grid::DragEvent),
    /// A driver for images
    ImageDriver(Uuid, image::Message),
    /// The route has been changed (e.g. through clicking an album)
    ChangeRoute(Route),
    Play,
}

/// The height one cell of the track table has
const CELL_HEIGHT: u32 = 64;
/// How many items should be anticipated by the sensor
const ANTICIPATED_CELLS: u32 = 0;

impl Tracks {
    pub fn view(&self) -> Element<'_, Message> {
        let play_button = widget::button(widget::container(lucide::play().size(20)).center(48))
            .height(42)
            .width(42)
            .style(|theme, status| {
                let mut style = button::primary(theme, status);
                style.border = Border::default().rounded(50);
                style
            })
            .on_press(Message::Play);

        let tracks = column![
            widget::space().height(8),
            widget::container(row![
                widget::space().width(12),
                play_button,
                widget::space().width(12),
                widget::container(widget::text("Tracks").size(36).font(Font {
                    weight: font::Weight::Bold,
                    ..Font::DEFAULT
                }))
                .align_y(Center)
                .height(Fill)
            ])
            .height(48),
            widget::space().height(16),
            widget::rule::horizontal(1),
            self.table_header(),
            widget::rule::horizontal(1),
            self.sliding_window()
        ];

        widget::sensor(tracks)
            .on_show(|_| Message::FetchTracks)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchTracks => {
                // Only fetch first time
                if !self.tracks.is_empty() {
                    return Task::none();
                }

                Task::perform(
                    async {
                        // TODO: Replace with global state
                        let jf = JellyfinApi::auth_user_password(
                            Url::parse("http://***REMOVED***").unwrap(),
                            "***REMOVED***".to_string(),
                            "***REMOVED***".to_string(),
                        )
                        .await;

                        jf.get_tracks().await.unwrap()
                    },
                    Message::TracksFetched,
                )
            }
            Message::TracksFetched(tracks) => {
                // Create images from tracks
                for v in tracks.iter().filter(|v| v.track.image_url.is_some()) {
                    let url = v.track.image_url.clone().unwrap();
                    let (id, image) = Image::new(url.into(), v.track.image_blur_hash.clone());
                    self.images.insert(id, image);
                }
                // Insert tracks
                self.tracks = tracks;

                Task::none()
            }
            Message::ImageDriver(url, m) => {
                if let Some(image) = self.images.get_mut(&url) {
                    image.update(m).map(move |m| Message::ImageDriver(url, m))
                } else {
                    Task::none()
                }
            }
            // Only bubbles up
            Message::ChangeRoute(_) => Task::none(),
            Message::TrackShown(id) => {
                self.track_visibility.insert(id, true);
                Task::none()
            }
            Message::TrackHidden(id) => {
                self.track_visibility.insert(id, false);
                Task::none()
            }
            Message::TrackClicked(id) => {
                // TODO: support ctrl/shift click for multiselect
                self.selected_tracks.clear();
                self.selected_tracks.insert(id);
                Task::none()
            }
            Message::ColumnResized(column, width) => {
                self.header_width.insert(column, width);
                Task::none()
            }
            Message::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                self.table_header.resize(split, ratio);
                Task::none()
            }
            Message::PaneReordered(pane_grid::DragEvent::Dropped { pane, target }) => {
                self.table_header.drop(pane, target);
                Task::none()
            }
            Message::PaneReordered(_) => Task::none(),
            Message::Play => todo!(),
        }
    }

    pub fn table_header(&self) -> Element<'_, Message> {
        let pane_grid = widget::pane_grid(&self.table_header, |pane, state, is_maximized| {
            pane_grid::Content::new({
                let container = match state {
                    TableColumns::Number => widget::container("#").center_x(Fill),
                    TableColumns::CombinedTitle => widget::container("TITLE"),
                    TableColumns::Duration => {
                        widget::container(lucide::clock_three()).center_x(Fill)
                    }
                    TableColumns::Album => widget::container("ALBUM"),
                    TableColumns::Genre => widget::container("GENRE"),
                }
                .center_y(Fill);

                sensor(container)
                    .on_show(|size| Message::ColumnResized(state.clone(), size.width))
                    .on_resize(|size| Message::ColumnResized(state.clone(), size.width))
            })
        })
        .on_resize(8, Message::PaneResized)
        .on_drag(Message::PaneReordered);

        widget::container(pane_grid).width(Fill).height(40).into()
    }

    pub fn combined_title(&self, v: &TrackView) -> Element<'_, Message> {
        let TrackView {
            track: Track {
                image_url, title, ..
            },
            artists,
            ..
        } = v;

        let image = if let Some(image_id) = image_url.as_ref().map(|u| Image::uuid(u))
            && let Some(image) = self.images.get(&image_id)
        {
            // Show an image if it's ready
            image.view().map(move |m| Message::ImageDriver(image_id, m))
        } else {
            // Or show a placeholder
            lucide::disc_album().size(18).into()
        };

        let title = widget::text(title.clone().unwrap_or("No title".to_owned()))
            .wrapping(Wrapping::None)
            .ellipsis(text::Ellipsis::End);

        let artist_text = artists
            .clone()
            .into_links(
                |RelatedArtist { id, name }| {
                    (
                        name.unwrap_or("Unknown artist".to_owned()),
                        Route::Artist(id.to_string()),
                    )
                },
                Message::ChangeRoute,
            )
            .wrapping(Wrapping::None)
            .ellipsis(text::Ellipsis::End)
            .size(14);

        row![
            widget::container(image).center(64),
            widget::space().width(8),
            widget::container(column![title, widget::space().width(2), artist_text])
                .center_y(64)
                .clip(true)
        ]
        .into()
    }

    pub fn sliding_window(&self) -> Element<'_, Message> {
        let tracks = self.tracks.iter().enumerate().map(|(index, view)| {
            let content = match self.track_visibility.get(&view.track.id) {
                // Produces a table segment if the chunk is visible the chunk
                Some(visible) if *visible => self.track_row(index, view),
                // Produce a cheap placeholder otherwise
                _ => space().width(Fill).height(CELL_HEIGHT).into(),
            };

            widget::sensor(content)
                .anticipate(ANTICIPATED_CELLS * CELL_HEIGHT) // Anticipate cells
                .on_show(|_| Message::TrackShown(view.track.id.clone()))
                .on_hide(Message::TrackHidden(view.track.id.clone()))
                .into()
        });

        widget::scrollable(column(tracks)).height(Shrink).into()
    }

    /// One row representing the
    pub fn track_row(&self, index: usize, view: &TrackView) -> Element<'_, Message> {
        // Index column
        let index_col = widget::container(widget::text(index as u32))
            .clip(true)
            .height(CELL_HEIGHT)
            .width(*self.header_width.get(&TableColumns::Number).unwrap_or(&0.0))
            .align_x(Center)
            .align_y(Center);

        // Cimbine title column
        let title_col = widget::container(self.combined_title(view))
            .clip(true)
            .height(CELL_HEIGHT)
            .width(
                *self
                    .header_width
                    .get(&TableColumns::CombinedTitle)
                    .unwrap_or(&0.0),
            )
            .align_y(Center);

        // Duration column
        let duration_col = widget::container(
            widget::text(format_duration(
                view.track.duration.unwrap_or_default() as f64
            ))
            .wrapping(Wrapping::None),
        )
        .clip(true)
        .height(CELL_HEIGHT)
        .width(
            *self
                .header_width
                .get(&TableColumns::Duration)
                .unwrap_or(&0.0),
        )
        .align_x(Center)
        .align_y(Center);

        // Album column
        let album_col = {
            let album_name = view
                .album_name
                .clone()
                .unwrap_or("Unknown album".to_owned());

            let album_name = album_name
                .link(
                    Route::Album(view.track.album_id.clone()),
                    Message::ChangeRoute,
                )
                .wrapping(Wrapping::None)
                .ellipsis(text::Ellipsis::End);

            widget::container(album_name)
                .clip(true)
                .height(CELL_HEIGHT)
                .width(*self.header_width.get(&TableColumns::Album).unwrap_or(&0.0))
                .align_y(Center)
        };

        // Genre column
        let genre_col = {
            let links = view
                .genres
                .clone()
                .into_links(
                    |RelatedGenre { id, name }| {
                        (name.unwrap_or("Unknown genre".to_owned()), Route::Genre(id))
                    },
                    Message::ChangeRoute,
                )
                .wrapping(Wrapping::None)
                .ellipsis(text::Ellipsis::End)
                .width(Fill);

            widget::container(links)
                .clip(true)
                .height(CELL_HEIGHT)
                .width(FillPortion(2))
                .align_y(Center)
        };

        let selected = self.selected_tracks.contains(&view.track.id);
        widget::button(row![index_col, title_col, duration_col, album_col, genre_col].width(Fill))
            .style(move |theme: &Theme, status| {
                button::Style::default().with_background(match selected {
                    // Selected
                    true => theme.palette().background.neutral.color,
                    // Hovered
                    false if matches!(status, Status::Hovered) => {
                        theme.palette().background.weak.color
                    }
                    // Neither
                    false => Color::TRANSPARENT,
                })
            })
            .padding(Padding::new(0.0).vertical(2))
            .on_press(Message::TrackClicked(view.track.id.clone()))
            .width(Fill)
            .into()
    }
}
