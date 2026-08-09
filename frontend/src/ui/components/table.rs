use core::panic;
use std::{
    clone::Clone,
    collections::HashSet,
    time::{Duration, Instant},
};

use iced::{
    Border, Color, Element,
    Length::{self, Fill, FillPortion, Fixed, Shrink},
    Padding, Point, Size, Task, Theme,
    advanced::graphics::futures::MaybeSend,
    alignment::{self, Vertical},
    mouse::Interaction,
    widget::{
        self,
        button::{self, Status, Style},
        row,
    },
};

/// A table component
pub struct Table<T, M> {
    /// The table rows
    ///
    /// These act as state, meaning every cell in a row shares one state
    rows: Vec<Row<T>>,
    /// The table columns
    ///
    /// These dictate view, they define multiple ways to draw the state (the
    /// rows)
    columns: Vec<Column<T, M>>,
    /// Contains the indices of visible chunks
    visible_chunks: HashSet<usize>,
    /// The current mouse position (we need this for resizing the columns)
    mouse_position: Point,
    /// Information about a resize process
    resize_info: Option<ResizeInfo>,
    double_click_callback: Option<fn(T)>,
}

pub struct ResizeInfo {
    /// Point where something started to get dragged
    drag_point: Point,
    /// A column that is at the moment being resized
    resizing_col: usize,
    /// The width of a column before resizing
    size_before: f32,
}

// Deriving default requires T and M implementing Default as well
impl<T, M> Default for Table<T, M> {
    fn default() -> Self {
        Self {
            rows: Vec::default(),
            columns: Vec::default(),
            mouse_position: Point::default(),
            resize_info: Option::default(),
            visible_chunks: HashSet::default(),
            double_click_callback: None,
        }
    }
}

impl<T, M> Table<T, M> {
    /// Grants a view into the data the table contains
    pub fn rows(&self) -> &Vec<Row<T>> {
        &self.rows
    }

    /// Grants a mutable view into the data the table contains
    pub fn rows_mut(&mut self) -> &mut Vec<Row<T>> {
        &mut self.rows
    }

    /// Clears the table data
    pub fn clear(&mut self) {
        self.rows.clear();
    }

    /// Adds a piece of data, hence a new row, to the table
    pub fn push(&mut self, data: T) {
        self.rows.push(Row::new(data));
    }

    /// Extends the table data
    pub fn extend(&mut self, rows: Vec<T>) {
        let rows: Vec<Row<T>> = rows.into_iter().map(Row::new).collect();
        self.rows.extend(rows);
    }

    /// Adds an additional column to the table
    pub fn add_column(mut self, column: Column<T, M>) -> Self {
        self.columns.push(column);
        self
    }

    /// Adds a double click handler to the table
    pub fn on_double_click_row(mut self, on_click: fn(item: T)) -> Self {
        self.double_click_callback = Some(on_click);
        self
    }
}

pub struct Row<T> {
    item: T,
    /// Is the row currently visible?
    visible: bool,
    /// Is the row currently selected?
    selected: bool,
    /// Tracking when a row was last clicked to register double clicks
    ///
    /// TODO: Consider stealing™ the `decorator/double_click` widget from
    /// halloy: <https://github.com/squidowl/halloy/blob/c3f2e4a30a1ac787342495640eb5de671de6d695/src/widget/double_click.rs>
    clicked_at: Option<Instant>,
}

impl<T> Row<T> {
    pub fn new(item: T) -> Self {
        Row {
            item,
            visible: false,
            selected: false,
            clicked_at: None,
        }
    }
}

/// A function that produces the header of a column
/// while this header can produce an arbitrary Message `M`, it will not be
/// driven or reacted to.
type ColumnHeader<M> = Box<dyn Fn() -> Element<'static, M> + 'static>;

/// A function that presents the row state in a column
type ColumnView<T, M> = Box<dyn for<'a> Fn(&'a T) -> Element<'a, M> + 'static>;

/// A function allowing us to update the row state in reaction to a column
/// view emitted message
type ColumnUpdate<T, M> = Box<dyn Fn(&mut T, M) -> Task<M> + 'static>;

pub struct Column<T, M> {
    /// The header for the column
    /// While this can be any element it cannot mutate state
    /// this is somewhat a given since an element is just a view
    header: ColumnHeader<M>,
    /// A function that presents the row state in a column
    view: ColumnView<T, M>,
    /// A function allowing us to update the row state in reaction to a column
    /// view emitted message
    update: ColumnUpdate<T, M>,
    /// How this columns contents are aligned on the x axis
    align_x: alignment::Horizontal,
    /// How this columns contents are aligned on the y axis
    align_y: alignment::Vertical,
    /// The target length of this column which we use for sizing
    width: Length,
    /// The minimal absolute width for this column
    min_width: f32,
    /// Marker for if the
    relative_sized: bool,
    /// The current measured width of the column
    measured_width: f32,
    /// If the cursor is currently on this columns grab area
    on_grab_area: bool,
}

impl<T, M> Column<T, M> {
    /// Creates a new column with a header and a data-view
    pub fn new(
        header: impl Fn() -> Element<'static, M> + 'static,
        view: impl for<'a> Fn(&'a T) -> Element<'a, M> + 'static,
    ) -> Column<T, M> {
        Column {
            header: Box::new(header),
            view: Box::new(view),
            update: Box::new(|_t, _m| Task::none()),
            align_x: alignment::Horizontal::Left,
            align_y: alignment::Vertical::Center,
            width: FillPortion(1),
            measured_width: 0.0,
            on_grab_area: false,
            relative_sized: false,
            min_width: 64.0,
        }
    }

    /// Sets the update method for this column if it's stateful
    pub fn update(mut self, update: impl Fn(&mut T, M) -> Task<M> + 'static) -> Column<T, M> {
        self.update = Box::new(update);
        self
    }

    /// Sets the x-axis alignment of this column
    pub fn align_x(mut self, alignment: alignment::Horizontal) -> Self {
        self.align_x = alignment;
        self
    }

    /// Sets the y-axis alignment of this column
    pub fn align_y(mut self, alignment: alignment::Vertical) -> Self {
        self.align_y = alignment;
        self
    }

    /// Sets initial width of this column
    pub fn intial_width(mut self, width: Length) -> Self {
        self.relative_sized = match width {
            FillPortion(_) => true,
            Fixed(_) => false,
            _ => panic!("Columns can only be fixed or by portion"),
        };
        self.width = width;
        self
    }

    /// Sets the minimal absolute width of this column
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }
}

#[derive(Debug, Clone)]
pub enum Cmd<M: Clone> {
    /// Row visibility
    RowShown(usize),
    RowHidden(usize),
    /// Chunk visibility
    ChunkShown(usize),
    ChunkHidden(usize),
    /// Driver for column content
    ColumnDriver(usize, usize, M),
    /// Mouse events
    MouseMoved(Point),
    MousePressed,
    MouseReleased,
    /// Message meant to consume the header element view
    None,
    /// Notification on column size change
    MeasureColumn(usize, Size),
    RowClicked(usize),
}

#[derive(Debug, Clone)]
pub enum Out<T: Clone> {
    /// A row has been double clicked
    DoubleClicked(T),
}

/// The height one row of the track table has
///
/// This has to be consistent because of virtualization of the rows
const ROW_HEIGHT: u32 = 64 + 8;
/// How many items should be anticipated by the sensor
const ANTICIPATED_ROWS: u32 = 2;
/// The amount of rows per chunk
const CHUNK_SIZE: usize = 100;

/// A table message defined by
/// - T: The table data type
/// - M: The cell message
pub type TableMsg<M> = Cmd<M>;

impl<T, M> Table<T, M>
where
    T: std::fmt::Debug + Clone + MaybeSend + 'static,
    M: std::fmt::Debug + Clone + MaybeSend + 'static,
{
    pub fn view(&self) -> Element<'_, TableMsg<M>> {
        let content = widget::column![
            widget::rule::horizontal(1),
            self.table_header(),
            widget::rule::horizontal(1),
            self.sliding_window()
        ];

        let mouse_area = widget::mouse_area(content)
            .on_move(Cmd::MouseMoved)
            .on_press(Cmd::MousePressed)
            .on_release(Cmd::MouseReleased)
            .on_exit(Cmd::MouseReleased);

        let mouse_area = if self.columns.iter().any(|c| c.on_grab_area) {
            mouse_area.interaction(Interaction::ResizingHorizontally)
        } else {
            mouse_area
        };

        mouse_area.into()
    }

    pub fn update(&mut self, message: impl Into<TableMsg<M>>) -> Task<TableMsg<M>> {
        match message.into() {
            Cmd::RowShown(row_idx) => {
                self.rows[row_idx].visible = true;
                Task::none()
            }
            Cmd::RowHidden(row_idx) => {
                self.rows[row_idx].visible = false;
                Task::none()
            }
            Cmd::ChunkShown(chunk_idx) => {
                self.visible_chunks.insert(chunk_idx);
                Task::none()
            }
            Cmd::ChunkHidden(chunk_idx) => {
                self.visible_chunks.remove(&chunk_idx);
                Task::none()
            }
            Cmd::ColumnDriver(row_idx, col_id, message) => {
                let row = &mut self.rows[row_idx];
                let col = &self.columns[col_id];

                (col.update)(&mut row.item, message)
                    .map(move |m| Cmd::ColumnDriver(row_idx, col_id, m))
            }
            Cmd::None => Task::none(),
            Cmd::MouseMoved(position) => {
                self.mouse_position = position;

                // Check if the mouse is on any resize points
                let mut end = 0.0;
                for col in &mut self.columns {
                    end += col.measured_width;
                    col.on_grab_area = end - 6.0 <= position.x && position.x <= end + 4.0;
                }

                // Resize column if needed
                if let Some(ResizeInfo {
                    drag_point,
                    resizing_col,
                    size_before,
                }) = self.resize_info
                {
                    let target_size = size_before - (drag_point.x - self.mouse_position.x);
                    self.columns[resizing_col].width = Fixed(target_size);
                }

                Task::none()
            }
            Cmd::MousePressed => {
                // Check if the cursor is on a resizer
                let Some(resizing_col) = self.columns.iter().position(|c| c.on_grab_area) else {
                    return Task::none();
                };

                self.resize_info = Some(ResizeInfo {
                    drag_point: self.mouse_position,
                    resizing_col,
                    size_before: self.columns[resizing_col].measured_width,
                });

                // Fix all columns before the one being resized
                // This allows us to resize without side effects
                // (No previous column changes the position of the current one)
                for col in self.columns.iter_mut().take(resizing_col + 1) {
                    col.width = Fixed(col.measured_width);
                }

                // Let the last column fill the rest of the space left when resizing the
                // second-to-last column
                if resizing_col == self.columns.len() - 2
                    && let Some(col) = self.columns.last_mut()
                {
                    // Go on; show me a screen with a width larger than 65536 pixels
                    col.width = FillPortion(col.measured_width.round() as u16);
                }
                Task::none()
            }
            Cmd::MouseReleased => {
                self.resize_info = None;

                // Return relative columns to a portioned layout
                for col in &mut self.columns {
                    if col.relative_sized {
                        col.width = FillPortion(col.measured_width.round() as u16);
                    } else {
                        col.width = Fixed(col.measured_width);
                    }
                }

                Task::none()
            }
            Cmd::MeasureColumn(idx, Size { width, .. }) => {
                let column = &mut self.columns[idx];

                if width >= column.min_width {
                    column.measured_width = width;
                    Task::none()
                } else {
                    // Abort resizing if any column goes below the minimum
                    // TODO: This fully disengages the resizing process; If possible this should
                    // instead disallow any further shrinking but allow growth
                    Task::done(Cmd::MouseReleased)
                }
            }
            Cmd::RowClicked(idx) => {
                let row = &mut self.rows[idx];
                row.selected = true;

                // Check if the row was clicked less than 200ms ago
                if let Some(instant) = row.clicked_at
                    && Instant::now().duration_since(instant) <= Duration::from_millis(200)
                    // Is a callback registered?
                    && let Some(double_click_callback) = self.double_click_callback
                {
                    log::debug!("Calling double click callback");
                    double_click_callback(row.item.clone());
                }
                row.clicked_at = Some(Instant::now());

                // Deselect all other rows
                for (index, row) in self.rows.iter_mut().enumerate() {
                    row.selected = index == idx;
                }

                Task::none()
            }
        }
    }

    /// Calculates the x positions of the end of the columns
    pub fn column_split(&self) -> Vec<f32> {
        self.columns
            .iter()
            .scan(0.0, |counter, col| {
                *counter += col.measured_width;
                Some(*counter)
            })
            .collect()
    }

    pub fn table_header(&self) -> Element<'_, Cmd<M>> {
        let last = self.columns.len() - 1;
        let headers = self.columns.iter().enumerate().map(|(idx, col)| {
            // The header
            let header = widget::container((col.header)().map(|_m| Cmd::None))
                .padding(Padding::ZERO.horizontal(8))
                .height(Fill)
                .width(Fill)
                .align_x(col.align_x)
                .align_y(Vertical::Center);

            // All the headers except the last get a grab button
            let content: Element<'_, Cmd<M>> = if idx < last {
                // Grab button for resizing
                let button = Self::grab_button(col);
                row![header, button].width(Fill).into()
            } else {
                // Just the header for the last one
                header.into()
            };

            // Measure the header width
            widget::sensor(widget::container(content).width(col.width))
                .on_show(move |s| Cmd::MeasureColumn(idx, s))
                .on_resize(move |s| Cmd::MeasureColumn(idx, s))
                .into()
        });

        widget::container(widget::row(headers))
            .width(Fill)
            .height(40)
            .into()
    }

    /// The button at the right end of a column that allows resizing it
    pub fn grab_button(column: &Column<T, M>) -> widget::Container<'_, Cmd<M>> {
        widget::container(
            widget::button(widget::space())
                .style(|theme: &Theme, _status| Style {
                    background: Some(
                        if column.on_grab_area {
                            theme.palette().primary.strong.color
                        } else {
                            theme.palette().background.strong.color
                        }
                        .into(),
                    ),
                    border: Border::default().rounded(90),
                    ..Default::default()
                })
                .width(4)
                .height(40),
        )
        .padding(Padding::ZERO.vertical(6))
    }

    /// The sliding window splits visibility into two parts:
    /// 1. chunks containing [`CHUNK_SIZE`] rows
    /// 2. the rows themselves
    ///
    /// By dividing visibility we can load a huge amount of rows
    /// (I tested this with up to `800_000` rows)
    ///
    /// TODO: refactor this (it reads like shit)
    pub fn sliding_window(&self) -> Element<'_, Cmd<M>> {
        // Split the rows into chunks of size CHUNK_ROWS
        let rows = self
            .rows
            .chunks(CHUNK_SIZE)
            .enumerate()
            .map(|(chunk_idx, chunk)| {
                // Check if the chunk is visible
                let chunk: Element<'_, Cmd<M>> = if self.visible_chunks.contains(&chunk_idx) {
                    widget::column(chunk.iter().enumerate().map(|(row_idx, row)| {
                        // Calculate proper row index with
                        let row_idx = row_idx + chunk_idx * CHUNK_SIZE;
                        let row = if row.visible {
                            // Row is visible so produce the row
                            self.item_row(row_idx, row)
                        } else {
                            // Row is invisible so produce a placeholder
                            widget::space().width(Fill).height(ROW_HEIGHT).into()
                        };

                        // Wrap the row with a sensor to watch if it's visible
                        widget::sensor(row)
                            .anticipate(ANTICIPATED_ROWS * ROW_HEIGHT)
                            .on_show(move |_| Cmd::RowShown(row_idx))
                            .on_hide(Cmd::RowHidden(row_idx))
                            .into()
                    }))
                    .into()
                } else {
                    widget::space()
                        .width(Fill)
                        .height(ROW_HEIGHT * CHUNK_SIZE as u32)
                        .into()
                };

                // Wrap the chunk with a sensor to watch if it's visible
                widget::sensor(chunk)
                    .anticipate(ANTICIPATED_ROWS * ROW_HEIGHT)
                    .on_show(move |_| Cmd::ChunkShown(chunk_idx))
                    .on_hide(Cmd::ChunkHidden(chunk_idx))
                    .into()
            });

        widget::scrollable(widget::column(rows))
            .auto_scroll(true)
            .height(Shrink)
            .into()
    }

    /// A row showing all the columns for an item T
    pub fn item_row<'a>(&self, row_idx: usize, row: &'a Row<T>) -> Element<'a, Cmd<M>> {
        // The columns
        let columns = self.columns.iter().enumerate().map(|(col_idx, col)| {
            widget::container(
                (col.view)(&row.item).map(move |m| Cmd::ColumnDriver(row_idx, col_idx, m)),
            )
            .width(col.measured_width - 2.0)
            .height(ROW_HEIGHT)
            .padding(Padding::new(8.0))
            .clip(true)
            .align_x(col.align_x)
            .align_y(col.align_y)
            .into()
        });

        // Row button allowing selection
        widget::button(widget::row(columns).width(Fill))
            .style({
                let selected = row.selected;
                move |theme: &Theme, status| {
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
                }
            })
            .height(ROW_HEIGHT)
            .padding(Padding::new(0.0))
            .on_press(Cmd::RowClicked(row_idx))
            .width(Fill)
            .into()
    }
}
