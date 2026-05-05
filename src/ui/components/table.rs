use iced::{
    Color, Element,
    Length::{self, Fill, FillPortion, Fixed, Shrink},
    Point, Size, Subscription, Task, Theme,
    advanced::graphics::futures::MaybeSend,
    alignment,
    mouse::Interaction,
    widget::{
        self,
        button::{self, Status, Style},
        row,
    },
};
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
    /// The current mouse position
    mouse_position: Point,
    /// Information about a resize process
    resize_info: Option<ResizeInfo>,
}

pub struct ResizeInfo {
    /// Point where something started to get dragged
    drag_point: Point,
    /// A column that is at the moment being resized
    resizing_col: usize,
    /// The width of a column before resizing
    size_before: f32,
}

// Deriving default doesn't work
impl<T, M> Default for Table<T, M> {
    fn default() -> Self {
        Self {
            rows: Default::default(),
            columns: Default::default(),
            mouse_position: Default::default(),
            resize_info: Default::default(),
        }
    }
}

impl<T, M> Table<T, M> {
    /// Grants a view into the data the table contains
    pub fn data(&self) -> Vec<&T> {
        self.rows.iter().map(|r| &r.item).collect()
    }

    /// Grants a mutable view into the data the table contains
    pub fn data_mut(&mut self) -> Vec<&mut T> {
        self.rows.iter_mut().map(|r| &mut r.item).collect()
    }

    /// Extends the table data
    pub fn extend_rows(&mut self, rows: Vec<T>) {
        let rows: Vec<Row<T>> = rows.into_iter().map(Row::new).collect();
        self.rows.extend(rows);
    }

    /// Adds an additional column to the table
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
            id: ColId(Uuid::new_v4()),
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
pub enum Message<M: Clone> {
    /// Row visibility
    RowShown(usize),
    RowHidden(usize),
    /// Driver for column content
    ColumnDriver(RowId, ColId, M),
    /// Mouse events
    MouseMoved(Point),
    MousePressed,
    MouseReleased,
    /// Message meant to consume the header element view
    None,
    /// Notification on column size change
    MeasureColumn(usize, Size),
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
        let content = widget::column![
            widget::rule::horizontal(1),
            self.table_header(),
            widget::rule::horizontal(1),
            self.sliding_window()
        ];

        let mouse_area = widget::mouse_area(content)
            .on_move(Message::MouseMoved)
            .on_press(Message::MousePressed)
            .on_release(Message::MouseReleased);

        let mouse_area = if self.columns.iter().any(|c| c.on_grab_area) {
            mouse_area.interaction(Interaction::ResizingHorizontally)
        } else {
            mouse_area
        };

        mouse_area.into()
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
            Message::ColumnDriver(row_id, col_id, message) => {
                // TODO: atrocious approach for indexing
                // Either use hashmaps or just use the actual indices
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
            Message::MouseMoved(position) => {
                self.mouse_position = position;

                // Check if the mouse is on any resize points
                let mut end = 0.0;
                for col in &mut self.columns {
                    end += col.measured_width;
                    col.on_grab_area = end - 8.0 <= position.x && position.x <= end + 4.0;
                }

                // Resize column if needed
                if let Some(ResizeInfo {
                    drag_point,
                    resizing_col,
                    size_before,
                }) = self.resize_info
                {
                    let col = &mut self.columns[resizing_col];
                    let target_size = size_before - (drag_point.x - self.mouse_position.x);
                    // Restrain to not go below minimum width
                    let target_size = target_size.max(col.min_width);

                    col.width = Fixed(target_size);
                }

                Task::none()
            }
            Message::MousePressed => {
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
                for col in self.columns.iter_mut().take(resizing_col + 1) {
                    col.width = Fixed(col.measured_width)
                }

                // Fill last column
                if let Some(col) = self.columns.last_mut() {
                    col.width = FillPortion(col.measured_width.round() as u16)
                }
                Task::none()
            }
            Message::MouseReleased => {
                self.resize_info = None;

                // Return relative columns to a portioned layout
                for col in &mut self.columns {
                    if col.relative_sized {
                        col.width = FillPortion(col.measured_width.round() as u16)
                    } else {
                        col.width = Fixed(col.measured_width)
                    }
                }

                Task::none()
            }
            Message::MeasureColumn(idx, size) => {
                self.columns[idx].measured_width = size.width;
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

    pub fn table_header(&self) -> Element<'_, Message<M>> {
        let last = self.columns.len() - 1;
        let headers = self.columns.iter().enumerate().map(|(idx, col)| {
            // All the headers except the last get a grab button
            let content: Element<'_, Message<M>> = if idx < last {
                // Grab button for resizing
                let button = widget::button(widget::space())
                    .style(|theme: &Theme, _status| {
                        Style::default().with_background(if col.on_grab_area {
                            theme.palette().primary.strong.color
                        } else {
                            theme.palette().background.strong.color
                        })
                    })
                    .width(2)
                    .height(Fill);

                row![
                    (col.header)().map(|_m| Message::None),
                    widget::space().width(Fill),
                    button
                ]
                .into()
            } else {
                (col.header)().map(|_m| Message::None)
            };

            // Header container
            let container = widget::container(content)
                .height(Fill)
                .width(col.width)
                .align_x(col.align_x)
                .align_y(col.align_y);

            // Measure the header width
            widget::sensor(container)
                .on_show(move |s| Message::MeasureColumn(idx, s))
                .on_resize(move |s| Message::MeasureColumn(idx, s))
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
                .anticipate(ANTICIPATED_CELLS * ROW_HEIGHT)
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
            .width(col.measured_width)
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
