use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use iced::{
    Color, Element, Event,
    Length::{Fill, Shrink},
    Padding, Point, Subscription, Task, Theme, alignment, event,
    mouse::{self, Event},
    widget::{
        self,
        button::{self, Status, Style},
        container::{self},
    },
};
use itertools::Itertools;
use uuid::Uuid;

#[repr(transparent)]
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
struct RowId(Uuid);

/// A dynamic table component
pub struct Table<'a, T, Message> {
    /// Whatever we are showing
    rows: HashMap<RowId, T>,
    /// Rows have been selected in the table
    selected_rows: HashSet<RowId>,
    /// Rows that are currently visible in the table
    row_visibility: HashMap<RowId, bool>,
    /// The columns identified each by a ColumnId
    columns: Vec<Column<'a, T, Message>>,
    /// A split that the user is currently hovering over
    hovered_split: Option<(usize, Point)>,
    /// A split that the user is currently grabbing
    grabbed_split: Option<(usize, Point)>,
}

impl<'a, T, Message> Default for Table<'a, T, Message> {
    fn default() -> Self {
        Self {
            rows: Default::default(),
            selected_rows: Default::default(),
            row_visibility: Default::default(),
            columns: Default::default(),
            hovered_split: Default::default(),
            grabbed_split: Default::default(),
        }
    }
}

impl<'a, T, Message> Table<'a, T, Message> {
    pub fn rows(mut self, rows: Vec<T>) -> Self {
        let rows = rows.into_iter().map(|t| (RowId(Uuid::new_v4()), t));
        self.rows.extend(rows);
        self
    }

    pub fn column(mut self, column: Column<'a, T, Message>) -> Self {
        self.columns.push(column);
        self
    }
}

pub struct Column<'a, T, Message> {
    /// A function that returns the content of the header
    header: Box<dyn Fn() -> Element<'a, Message> + 'a>,
    /// A function that returns the content of a column for the given data
    view: Box<dyn Fn(&T) -> Element<'a, Message> + 'a>,
    /// How this columns contents are aligned on the x axis
    align_x: alignment::Horizontal,
    /// How this columns contents are aligned on the y axis
    align_y: alignment::Vertical,
    /// The preferred relative width of this column ([`iced::Length::FillPortion`])
    pref_rel_width: u32,
    /// The minimum absolute width of this column
    min_abs_width: f32,
    /// The current absolute width of this column
    width: f32,
}

impl<'a, T, Message> Column<'a, T, Message> {
    pub fn new(
        header: impl Fn() -> Element<'a, Message> + 'a,
        view: impl Fn(&T) -> Element<'a, Message> + 'a,
    ) -> Column<'a, T, Message> {
        Column {
            header: Box::new(header),
            view: Box::new(view),
            align_x: alignment::Horizontal::Left,
            align_y: alignment::Vertical::Center,
            pref_rel_width: 1,
            min_abs_width: 64.0,
            width: 300.0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message<M>
where
    M: Clone + std::fmt::Debug,
{
    RowShown(RowId),
    RowHidden(RowId),
    ColumnResized(usize, f32),
    HeaderPress(usize),
    HeaderRelease,
    HeaderMove(Point),
    EventOccurred(Event),
}

/// The height one cell of the track table has
const CELL_HEIGHT: u32 = 64;
/// How many items should be anticipated by the sensor
const ANTICIPATED_CELLS: u32 = 0;

impl

impl<'a, T, M> Table<'a, T, M>
where
    M: Clone + std::fmt::Debug + 'a,
{
    pub fn view(&self) -> Element<'_, Message<M>> {
        widget::column![
            widget::rule::horizontal(1),
            self.table_header(),
            widget::rule::horizontal(1),
            self.sliding_window()
        ]
        .into()
    }

    pub fn update(&mut self, message: Message<M>) -> Task<Message<M>> {
        match message {
            Message::RowShown(row_id) => {
                self.row_visibility.insert(row_id, true);
                Task::none()
            }
            Message::RowHidden(row_id) => {
                self.row_visibility.insert(row_id, false);
                Task::none()
            }
            // Update column size
            Message::ColumnResized(index, width) => {
                // self.columns.get_mut(index).unwrap().width = width;
                Task::none()
            }
            Message::HeaderPress(idx) => {
                println!("Press");
                self.grabbed_split = self.hovered_split;
                Task::none()
            }
            Message::HeaderRelease => {
                self.grabbed_split = None;
                Task::none()
            }
            Message::HeaderMove(point) => {
                self.hovered_split = self.on_split(point).map(|i| (i, point));
                println!("{}", self.hovered_split.is_some());
                if let Some((index, point2)) = self.grabbed_split {
                    self.grabbed_split = self.grabbed_split.map(|(i, _p)| (i, point));
                    self.columns[index].width += point2.x - point.x;
                    println!("Updating {index} to {}", self.columns[index].width);
                    // A split was grabbed so we update the width of the item before it
                }
                Task::none()
            }
            Message::ColMessage(_) => Task::none(),
            Message::EventOccurred(Event::Mouse(e)) => match e {
                mouse::Event::CursorEntered => todo!(),
                mouse::Event::CursorLeft => todo!(),
                mouse::Event::CursorMoved { position } => todo!(),
                mouse::Event::ButtonPressed(button) => todo!(),
                mouse::Event::ButtonReleased(button) => todo!(),
                mouse::Event::WheelScrolled { delta } => todo!(),
            },
            Message::EventOccurred(_) => Task::none(),
        }
    }

    fn subscription(&self) -> Subscription<Message<M>> {
        event::listen().map(Message::EventOccurred)
    }

    pub fn on_split(&self, point: Point) -> Option<usize> {
        let mut split_start = 0.0;
        let x = point.x;
        self.columns.iter().position(|col| {
            split_start += col.width;
            let split_end: f32 = split_start + 4.0;
            split_start <= x && x <= split_end
        })
    }

    pub fn table_header(&self) -> Element<'_, Message<M>> {
        let mut count = 0;
        let headers = self
            .columns
            .iter()
            .enumerate()
            .map(|(idx, col)| {
                // Header container
                widget::container((col.header)().map(Message::ColMessage))
                    .padding(Padding::new(0.0).horizontal(8))
                    .style(container::bordered_box)
                    .height(Fill)
                    .width(col.width)
                    .align_x(col.align_x)
                    .align_y(col.align_y)
                    .into()
            })
            .intersperse_with(move || {
                // Just a button that shows the grab area for resizing
                widget::button(widget::space())
                    .style(|theme: &Theme, status| {
                        Style::default().with_background(match status {
                            Status::Pressed | Status::Hovered => {
                                theme.palette().background.weaker.color
                            }
                            _ => Color::TRANSPARENT,
                        })
                    })
                    .width(4)
                    .height(Fill)
                    .into()
            });

        widget::container(widget::row(headers))
            .width(Fill)
            .height(40)
            .into()
    }

    pub fn sliding_window(&self) -> Element<'_, Message<M>> {
        let rows = self.rows.iter().map(|(id, item)| {
            let content = match self.row_visibility.get(id) {
                // Produces a table segment if the chunk is visible the chunk
                Some(visible) if *visible => self.item_row(item),
                // Produce a cheap placeholder otherwise
                _ => widget::space().width(Fill).height(CELL_HEIGHT).into(),
            };

            widget::sensor(content)
                .anticipate(ANTICIPATED_CELLS * CELL_HEIGHT) // Anticipate cells
                .delay(Duration::from_millis(10))
                .on_show(|_| Message::RowShown(*id))
                .on_hide(Message::RowHidden(*id))
                .into()
        });

        widget::scrollable(widget::column(rows))
            .height(Shrink)
            .into()
    }

    /// One row representing the
    pub fn item_row(&self, item: &T) -> Element<'_, Message<M>> {
        let views = self.columns.iter().map(|col| {
            widget::container((col.view)(item).map(Message::ColMessage))
                .width(col.width)
                .clip(true)
                .height(CELL_HEIGHT)
                .align_x(col.align_x)
                .align_y(col.align_y)
                .into()
        });

        // let selected = self.selected_rows.contains(&view.track.id);
        let selected = false; // TODO: see above
        widget::button(widget::row(views).width(Fill))
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
            // .on_press(Message::TrackClicked(view.track.id.clone())) TODO:
            .width(Fill)
            .into()
    }
}
