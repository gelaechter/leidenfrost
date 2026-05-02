use std::time::Duration;

use iced::{
    Color, Element, Event,
    Length::{Fill, Shrink},
    Padding, Point, Subscription, Task, Theme,
    advanced::graphics::futures::MaybeSend,
    alignment, event,
    mouse::{self},
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
struct ColId(Uuid);

#[repr(transparent)]
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
struct RowId(Uuid);

/// A table component
pub struct Table<T, M> {
    /// The table rows
    /// These act as state, meaning every cell in a row shares one state
    rows: Vec<Row<T>>,
    /// The table columns
    /// These dictate view, they define multiple ways to draw the state (the rows)
    columns: Vec<Column<T, M>>,
    /// A split that the user is currently hovering over
    hovered_split: Option<(usize, Point)>,
    /// A split that the user is currently grabbing
    grabbed_split: Option<(usize, Point)>,
}

// Deriving default doesn't work
impl<T, M> Default for Table<T, M> {
    fn default() -> Self {
        Self {
            rows: Default::default(),
            columns: Default::default(),
            hovered_split: Default::default(),
            grabbed_split: Default::default(),
        }
    }
}

impl<T, M> Table<T, M> {
    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn extend_rows(&mut self, rows: Vec<T>) {
        let rows: Vec<Row<T>> = rows.into_iter().map(Row::new).collect();
        self.rows.extend(rows);
    }

    pub fn column(mut self, column: Column<T, M>) -> Self {
        self.columns.push(column);
        self
    }
}

pub struct Row<T> {
    id: RowId,
    item: T,
    visible: bool,
    selected: bool,
}

impl<T> Row<T> {
    pub fn new(item: T) -> Self {
        Row {
            item,
            visible: false,
            selected: false,
            id: RowId(Uuid::new_v4()),
        }
    }
}

pub struct Column<T, M> {
    id: ColId,
    /// The header for the column
    /// While this can be any element it cannot mutate state
    /// this is somewhat a given since an element is just a view
    header: Box<dyn for<'a> Fn() -> Element<'static, M> + 'static>,
    /// A function that returns the content of a column for the given data
    view: Box<dyn for<'a> Fn(&'a T) -> Element<'a, M> + 'static>,
    update: Box<dyn Fn(&mut T, M) -> Task<M> + 'static>,
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

impl<T, M> Column<T, M> {
    pub fn new(
        header: impl Fn() -> Element<'static, M> + 'static,
        view: impl for<'a> Fn(&'a T) -> Element<'a, M> + 'static,
    ) -> Column<T, M> {
        Column {
            id: ColId(Uuid::new_v4()),
            header: Box::new(header),
            view: Box::new(view),
            update: Box::new(|_t, _m| Task::none()),
            align_x: alignment::Horizontal::Left,
            align_y: alignment::Vertical::Center,
            pref_rel_width: 1,
            min_abs_width: 64.0,
            width: 300.0,
        }
    }

    pub fn update(mut self, update: impl Fn(&mut T, M) -> Task<M> + 'static) -> Column<T, M> {
        self.update = Box::new(update);
        self
    }

    pub fn align_x(mut self, alignment: alignment::Horizontal) -> Self {
        self.align_x = alignment;
        self
    }

    pub fn align_y(mut self, alignment: alignment::Vertical) -> Self {
        self.align_y = alignment;
        self
    }
}

#[derive(Debug, Clone)]
pub enum Message<M: Clone> {
    RowShown(usize),
    RowHidden(usize),
    ColumnResized(usize, f32),
    HeaderPress(usize),
    HeaderRelease,
    HeaderMove(Point),
    EventOccurred(Event),
    ColumnDriver(RowId, ColId, M),
    /// None message meant to consume the header element view
    None,
}

/// The height one cell of the track table has
const ROW_HEIGHT: u32 = 64;
/// How many items should be anticipated by the sensor
const ANTICIPATED_CELLS: u32 = 2;

impl<T, M> Table<T, M>
where
    T: std::fmt::Debug + Clone + 'static,
    M: std::fmt::Debug + Clone + MaybeSend + 'static,
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
            Message::RowShown(row_idx) => {
                self.rows[row_idx].visible = true;
                Task::none()
            }
            Message::RowHidden(row_idx) => {
                self.rows[row_idx].visible = false;
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
            Message::EventOccurred(Event::Mouse(e)) => match e {
                mouse::Event::CursorEntered => todo!(),
                mouse::Event::CursorLeft => todo!(),
                mouse::Event::CursorMoved { position } => todo!(),
                mouse::Event::ButtonPressed(button) => todo!(),
                mouse::Event::ButtonReleased(button) => todo!(),
                mouse::Event::WheelScrolled { delta } => todo!(),
            },
            Message::EventOccurred(_) => Task::none(),

            Message::ColumnDriver(row_id, col_id, message) => {
                let row = self.rows.iter_mut().find(|r| r.id == row_id);
                let col = self.columns.iter_mut().find(|c| c.id == col_id);

                if let Some(row) = row
                    && let Some(col) = col
                {
                    (col.update)(&mut row.item, message)
                        .map(move |m| Message::ColumnDriver(row_id, col_id, m))
                } else {
                    Task::none()
                }
            }
            Message::None => Task::none(),
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
        let headers = self
            .columns
            .iter()
            .enumerate()
            .map(|(idx, col)| {
                // Header container
                widget::container((col.header)().map(|_m| Message::None))
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
        // Split the rows into chunks of size CHUNK_ROWS
        let rows = self.rows.iter().enumerate().map(|(row_idx, row)| {
            // Check if the chunk is visible
            let content: Element<'_, Message<M>> = match row.visible {
                // Produce a table segment if the chunk is visible
                true => self.item_row(row),
                // Produce a cheap placeholder otherwise
                false => widget::space().width(Fill).height(ROW_HEIGHT).into(),
            };

            // Wrap the chunk with a sensor to watch if it's visible
            let content = widget::sensor(content)
                .key(row_idx)
                .anticipate(ANTICIPATED_CELLS * ROW_HEIGHT) // Anticipate cells
                // .delay(Duration::from_millis(1))
                .on_show(move |_| Message::RowShown(row_idx))
                .on_hide(Message::RowHidden(row_idx))
                .into();

            (row_idx, content)
        });

        widget::scrollable(widget::keyed_column(rows))
            .height(Shrink)
            .into()
    }

    /// A row showing all the columns for an item T
    pub fn item_row<'a>(&self, row: &'a Row<T>) -> Element<'a, Message<M>> {
        let columns = self.columns.iter().map(move |col| {
            widget::container((col.view)(&row.item).map({
                let row_id = row.id;
                let col_id = col.id;
                move |m| Message::ColumnDriver(row_id, col_id, m)
            }))
            .width(col.width)
            .clip(true)
            .height(ROW_HEIGHT)
            .align_x(col.align_x)
            .align_y(col.align_y)
            .into()
        });

        widget::button(widget::row(columns).width(Fill))
            .style({
                let selected = row.selected;
                move |theme: &Theme, status| {
                    button::Style::default().with_background(match selected {
                        // Selected
                        true => theme.palette().background.neutral.color,
                        // Hovered
                        false if matches!(status, Status::Hovered) => {
                            theme.palette().primary.weak.color
                        }
                        // Neither
                        false => Color::TRANSPARENT,
                    })
                }
            })
            .height(ROW_HEIGHT)
            // .padding(Padding::new(0.0).vertical(2))
            // .on_press(Message::TrackClicked(view.track.id.clone()))
            .width(Fill)
            .into()
    }
}
