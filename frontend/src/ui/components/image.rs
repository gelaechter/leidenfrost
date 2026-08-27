use iced::{
    Alignment::Center, Border, Color, ContentFit, Element, Length::{self}, Size, Task, advanced::Widget, border::{self, Radius}, task::{self}, widget::{
        self, container,
        image::{self},
    },
};
use std::{
    collections::HashMap,
    fmt, io,
    sync::{Arc, LazyLock},
    time::Duration,
};
use url::Url;
use uuid::Uuid;

use crate::ui::components::icons;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[derive(Default)]
pub struct Manager {
    images: HashMap<Uuid, Image>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ImageDriver(Uuid, ImgMsg),
}

impl Manager {
    fn uuid(url: &Url) -> Uuid {
        Uuid::new_v3(
            &Uuid::NAMESPACE_URL,
            // TODO: Only use base-path / disregard any queries
            url.as_str().as_bytes(),
        )
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<I>(&mut self, images: I)
    where
        I: IntoIterator<Item = Image>,
    {
        self.images.extend(
            images
                .into_iter()
                .map(|image| (Self::uuid(&image.url), image)),
        );
    }

    pub fn view<'a>(&self, url: impl Into<&'a Url>) -> Option<Element<'_, Message>> {
        let uuid = Self::uuid(url.into());
        self.images
            .get(&uuid)
            .map(move |image| image.view().map(move |m| Message::ImageDriver(uuid, m)))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ImageDriver(uuid, image_message) => {
                if let Some(image) = self.images.get_mut(&uuid) {
                    image
                        .update(image_message)
                        .map(move |m| Message::ImageDriver(uuid, m))
                } else {
                    log::warn!(
                        "Received message {image_message:?} in event loop for non-existing image {uuid:?}"
                    );
                    Task::none()
                }
            }
        }
    }
}

/// A network image which automatically fetches the image data from a given URL
#[derive(Debug, Clone)]
pub struct Image {
    /// The Url of this image
    /// TODO: Add a global Handle cache for Urls that are the same.
    url: Url,
    /// The height of the image.
    ///
    /// This specifies the height of the container around the image. The image
    /// will expand to fill that container.
    height: Option<Length>,
    /// The width of the image.
    ///
    /// This specifies the width of the _container around the image_. The image
    /// will expand to fill that container.
    width: Option<Length>,
    border_radius: border::Radius,
    /// The size of the image container
    measured_size: Size,
    /// Duration to wait before registering image visibility
    image_debounce: Option<Duration>,
    blurhash_debounce: Option<Duration>,
    /// The images blur hash (if any)
    blurhash: Option<String>,
    /// This function gives the image the ability to modify the given
    /// url with size parameters, based on its own measured dimensions
    ///
    /// This
    resolution_strategy: Option<fn(Url, Size) -> Url>,
    /// Automatically refetch the image if the image size changes?
    ///
    /// This does nothing if there is no sizing strategy
    responsive_resolution: bool,
    /// The currently allocated image data (if any)
    status: Option<Content>,
    /// Any running image download task
    download_task: Option<task::Handle>,
    /// Any running blurhash decoding task
    blurhash_task: Option<task::Handle>,
    visible: bool,
}

#[derive(Debug, Clone)]
pub enum ImgMsg {
    Hidden,
    /// The image has become visible
    Shown(Size),
    /// The blurhash should be rendered
    DebounceBlurhash(Size),
    /// The image should be rendered
    DebounceImage(Size),
    /// The image was resized
    Resized(Size),
    /// The blurhash has been decoded
    BlurhashDecoded(Result<image::Handle, Error>),
    /// The image has been downloaded and allocated
    ImageDownloaded(Result<image::Handle, Error>),
}

#[derive(Debug, Clone)]
pub enum Content {
    Blurhash(image::Handle),
    Full(image::Handle),
    Error,
}

impl Image {
    /// Creates a new image
    pub fn new(url: impl Into<Url>) -> Self {
        Image {
            url: url.into(),
            blurhash: None,
            resolution_strategy: None,
            measured_size: Size {
                width: 16.0,
                height: 16.0,
            },
            status: None,
            image_debounce: None,
            download_task: None,
            blurhash_task: None,
            height: None,
            width: None,
            border_radius: Radius::default(),
            responsive_resolution: false,
            blurhash_debounce: None,
            visible: false,
        }
    }

    /// Sets the preferred height of the image
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = Some(height.into());
        self
    }

    /// Sets the preferred width of the image
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Sets the border radius of the image
    pub fn border_radius(mut self, radius: impl Into<border::Radius>) -> Self {
        self.border_radius = radius.into();
        self
    }

    /// Adds an optional blurhash to the image
    pub fn blurhash_maybe(mut self, blurhash: Option<String>) -> Self {
        self.blurhash = blurhash;
        self
    }

    /// Sets the URL parametrization strategy so the image can fetch
    /// the proper resolution of image and doesn't unneccesarily fetch
    /// the fully sized image.
    ///
    /// E.g. `("xx://server/image", Size {32, 32}) ->
    /// "xx://server/image?width=32&height=32"`
    pub fn resolution_strategy(mut self, strategy: fn(Url, Size) -> Url) -> Self {
        self.resolution_strategy = Some(strategy);
        self
    }

    /// Makes the image respond to resolution changes by refetching the image
    /// in a higher resolution if a sizing strategy has been specified through
    /// [`Image::sizing_strategy`]
    pub fn responsive_resolution(mut self, resp_res: bool) -> Self {
        self.responsive_resolution = resp_res;
        self
    }

    /// Decodes the blurhash (if any) beforehand;
    /// This is a blocking operation and should be treated as such
    #[deprecated = "let the blurhash calculate dynamically"]
    pub fn pre_decode_blurhash(mut self, width: u32, height: u32) -> Self {
        if let Some(blurhash) = &self.blurhash
            && let Ok(pixels) = blurhash::decode(blurhash, width, height, 1.0)
        {
            let handle = image::Handle::from_rgba(width, height, pixels);
            self.status = Some(Content::Blurhash(handle));
        }
        self
    }

    /// Debounces the fetching of the image
    pub fn debounce(mut self, duration: Duration) -> Self {
        self.image_debounce = Some(duration);
        self
    }

    /// Debounces the rendering of the blurhash
    pub fn debounce_blurhash(mut self, duration: Duration) -> Self {
        self.blurhash_debounce = Some(duration);
        self
    }
}

impl Image {
    pub fn view(&self) -> Element<'_, ImgMsg> {
        let image: Element<'_, ImgMsg> = match &self.status {
            Some(Content::Blurhash(handle) | Content::Full(handle)) => {
                // Either the blurhash or the image have been loaded
                widget::image(handle)
                    .expand(true)
                    .content_fit(ContentFit::Contain)
                    .border_radius(self.border_radius)
                    .into()
            }
            Some(Content::Error) => widget::responsive(|s| {
                let size = s.ratio(1.0).width;
                widget::center(
                    icons::image_off()
                        .size(size - 16.0)
                        .color(Color::BLACK.scale_alpha(0.5)),
                )
            })
            .into(),
            None => widget::responsive(|s| {
                let size = s.ratio(1.0);
                widget::space().height(size.height).width(size.width)
            })
            .into(),
        };

        // Sensor that checks width/height the image is trying to occupy
        let sensor = widget::sensor(image)
            // key images so that if a reflow is triggered where images are replaced with others the
            // events then re-emit
            .key(self.url.to_string())
            .on_resize(ImgMsg::Resized)
            .on_show(ImgMsg::Shown)
            .on_hide(ImgMsg::Hidden);

        let mut container = widget::container(sensor)
            .align_x(Center)
            .align_y(Center)
            .style(|_| {
                container::background(Color::BLACK.scale_alpha(0.1))
                    .border(Border::default().rounded(self.border_radius))
            });

        // Apply optional width/height
        if let Some(width) = self.width {
            container = container.width(width);
        }
        if let Some(height) = self.height {
            container = container.height(height);
        }

        container.into()
    }

    pub fn update(&mut self, message: ImgMsg) -> Task<ImgMsg> {
        match message {
            // Once the image is shown / size is changed
            ImgMsg::Shown(size) => {
                self.visible = true;
                if size.width < 1.0 || size.height < 1.0 {
                    return Task::none();
                }

                if size.width < 16.0 || size.height < 16.0 {
                    log::warn!(
                        "Tiny image shown; Are you still layouting?\n\
                        {size:?}\n\
                        {}",
                        self.url
                    );
                }
                self.measured_size = size;

                let Image {
                    image_debounce,
                    blurhash_debounce,
                    blurhash,
                    ..
                } = self.clone();

                // Only render the blurhash if one is set
                let blurhash_timer = if blurhash.is_some() && self.status.is_none() {
                    // Delay the blurhash if a delay is set
                    if let Some(debounce) = blurhash_debounce {
                        Task::future(async move {
                            tokio::time::sleep(debounce).await;
                            Some(ImgMsg::DebounceBlurhash(size))
                        })
                        .and_then(Task::done)
                    } else {
                        Task::done(ImgMsg::DebounceBlurhash(size))
                    }
                } else {
                    Task::none()
                };

                // Delay the image if a delay is set
                let render_timer = if matches!(self.status, None | Some(Content::Blurhash(_))) {
                    if let Some(debounce) = image_debounce {
                        Task::future(async move {
                            tokio::time::sleep(debounce).await;
                            Some(ImgMsg::DebounceImage(size))
                        })
                        .and_then(Task::done)
                    } else {
                        Task::done(ImgMsg::DebounceImage(size))
                    }
                } else {
                    Task::none()
                };

                Task::batch([blurhash_timer, render_timer])
            }
            ImgMsg::DebounceBlurhash(size) => {
                if self.visible && self.status.is_none() {
                    return self.blurhash_task();
                }

                Task::none()
            }
            ImgMsg::DebounceImage(size) => {
                if self.visible && matches!(self.status, None | Some(Content::Blurhash(_))) {
                    return self.download_task();
                }

                Task::none()
            }
            ImgMsg::Hidden => {
                // stop tracking time once the image is hidden
                self.visible = false;

                // If the image is hidden before it is fully downloaded
                // the download may be aborted
                if let Some(download_task) = &self.download_task.take() {
                    download_task.abort();
                }
                Task::none()
            }
            ImgMsg::Resized(size) => {
                // If auto sizing is active and the new resolution has greater dimensions
                if self.responsive_resolution
                    && self.resolution_strategy.is_some()
                    && size.width > self.measured_size.width
                    && size.height > self.measured_size.height
                {
                    // Then redownload / rerender
                    match &self.blurhash {
                        // Either just download
                        None => self.download_task(),
                        // Or download and decode blurhash
                        Some(_) => Task::batch([self.blurhash_task(), self.download_task()]),
                    }
                } else {
                    Task::none()
                }
            }
            ImgMsg::BlurhashDecoded(handle) => {
                if let Ok(handle) = handle {
                    self.status = Some(Content::Blurhash(handle));
                } else {
                    self.status = Some(Content::Error);
                }
                self.blurhash_task.take();
                Task::none()
            }
            ImgMsg::ImageDownloaded(handle) => {
                if let Ok(handle) = handle {
                    self.status = Some(Content::Full(handle));
                    // Download has concluded so abort blurhash decoding
                    if let Some(blurhash_task) = &self.blurhash_task.take() {
                        blurhash_task.abort();
                    }
                } else if self.status.is_none() {
                    self.status = Some(Content::Error);
                }
                self.download_task.take();
                Task::none()
            }
        }
    }

    /// Asynchronously downloads the image from the URL
    /// automatically downloads a sized version based on the resizer
    async fn download(
        url: Url,
        measured_size: Size,
        sizing_strategy: Option<fn(Url, Size) -> Url>,
    ) -> Result<Bytes, Error> {
        // Change the image URL to be sized if there is a sizing strategy available
        let url = if let Some(resize_strat) = sizing_strategy {
            resize_strat(url, measured_size)
        } else {
            url
        };

        let request = CLIENT.get(url);
        let bytes = request.send().await?.error_for_status()?.bytes().await?;

        Ok(Bytes(bytes))
    }

    /// Asynchronously decodes the blurhash for the given size
    #[allow(clippy::cast_possible_truncation)] // the resol are relatively small
    #[allow(clippy::cast_sign_loss)] // And they shouldn't be negative either
    async fn decode_blurhash(blurhash: String, size: Size) -> Result<Rgba, Error> {
        let width = size.width.round() as u32;
        let height = size.height.round() as u32;

        // Decode the blurhash (this is a blocking operation)
        tokio::task::spawn_blocking(move || {
            let pixels = blurhash::decode(&blurhash, width, height, 1.0)?;

            Ok(Rgba {
                width,
                height,
                pixels: Bytes(pixels.into()),
            })
        })
        .await?
    }

    fn download_task(&mut self) -> Task<ImgMsg> {
        let Self {
            url,
            measured_size,
            resolution_strategy: sizing_strategy,
            download_task,
            ..
        } = self;

        // Abort previous download (if any)
        if let Some(download_handle) = download_task.take() {
            download_handle.abort();
        }

        // Start new download
        let (task, download_handle) = Task::future(Self::download(
            url.clone(),
            *measured_size,
            *sizing_strategy,
        ))
        .and_then(|bytes| {
            // After downloading directly allocate the image so we only start overwriting the
            // Blurhashes once the image is fully allocated. This way we get a guarantee that the
            // image is actually drawn once we replace the Blurhash, instead of being deferred to the
            // renderer.
            //
            // > [`image::allocate`]:
            // > When you obtain an Allocation explicitly, you get the guarantee that using a Handle
            // > will draw the corresponding image immediately in the next frame.
            image::allocate(image::Handle::from_bytes(bytes)).map_err(|_| Error::ImageDecodingFailed)
        })
        .and_then(|a| Task::done(Ok(a.handle().clone())))
        .map(ImgMsg::ImageDownloaded)
        .abortable();

        // Set download handle and return value
        *download_task = Some(download_handle.abort_on_drop());
        task
    }

    fn blurhash_task(&mut self) -> Task<ImgMsg> {
        let Self {
            measured_size,
            blurhash,
            blurhash_task,
            ..
        } = self;

        // Abort previous task (if any)
        if let Some(blurhash_handle) = blurhash_task.take() {
            blurhash_handle.abort();
        }

        // Decode blurhash
        let (blurhash_task, blurhash_handle) = Task::future(Self::decode_blurhash(
            blurhash
                .clone()
                .expect("The blurhash_task function should only be called when blurhash is Some")
                .clone(),
            *measured_size,
        ))
        // Then allocate image
        .and_then(|rgba| {
            let Rgba {
                width,
                height,
                pixels,
            } = rgba;

            Task::done(Ok(
                // Unlike the download task this time we only create the handle and defer
                // the allocation to the renderer. This way we can decode all blurhashes as
                // fast as possible without blocking ourselves by also allocating them.
                image::Handle::from_rgba(width, height, pixels),
            ))
        })
        .map(ImgMsg::BlurhashDecoded)
        .abortable();

        // Set blurhash handle and return value
        self.blurhash_task = Some(blurhash_handle.abort_on_drop());
        blurhash_task
    }
}

#[derive(Clone)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Bytes,
}

// We don't want to log all the bytes
#[allow(clippy::missing_fields_in_debug)]
impl fmt::Debug for Rgba {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Rgba")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

#[derive(Clone)]
pub struct Bytes(bytes::Bytes);

impl Bytes {
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl From<Bytes> for bytes::Bytes {
    fn from(value: Bytes) -> Self {
        value.0
    }
}

impl fmt::Debug for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Compressed")
            .field("bytes", &self.0.len())
            .finish()
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Error {
    RequestFailed(Arc<reqwest::Error>),
    IOFailed(Arc<io::Error>),
    JoinFailed(Arc<tokio::task::JoinError>),
    ImageDecodingFailed,
    BlurhashDecodingFailed(Arc<blurhash::Error>),
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::RequestFailed(Arc::new(error))
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IOFailed(Arc::new(error))
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(error: tokio::task::JoinError) -> Self {
        Self::JoinFailed(Arc::new(error))
    }
}

impl From<blurhash::Error> for Error {
    fn from(error: blurhash::Error) -> Self {
        Self::BlurhashDecodingFailed(Arc::new(error))
    }
}
