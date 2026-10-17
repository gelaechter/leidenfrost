use std::{
    collections::HashMap,
    fmt, io,
    sync::{Arc, LazyLock, RwLock},
    time::Duration,
};

use bytes::Bytes;
use iced::{
    Alignment::Center,
    Border, Color, ContentFit, Element, Length, Size, Task,
    border::{self, Radius},
    task,
    widget::{self, container, image},
};
use url::Url;
use uuid::Uuid;

use crate::ui::components::icons;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

/// Everything which is shared between all instances of the same image URL
#[derive(Debug, Clone, Default)]
pub struct ImageState {
    /// Handle of any running image download task
    pub download_task: Option<task::Handle>,
    /// Handle of any running blurhash decoding task
    pub blurhash_task: Option<task::Handle>,
    /// How many image instances are currently visible on screen
    pub visible: usize,
    /// The currently allocated image data (if any)
    pub content: Option<Content>,
}

impl ImageState {
    /// Sets a new download task handle, aborting any previous download
    pub(super) fn set_download_task(&mut self, handle: task::Handle) {
        // Abort previous download (if any)
        if let Some(previous) = self.download_task.take() {
            previous.abort();
        }

        self.download_task = Some(handle);
    }

    /// Sets a new blurhash task handle, aborting any previous decoding
    pub(super) fn set_blurhash_task(&mut self, handle: task::Handle) {
        // Abort previous task (if any)
        if let Some(previous) = self.blurhash_task.take() {
            previous.abort();
        }

        self.blurhash_task = Some(handle);
    }

    /// The download task has concluded: drop its handle and
    /// abort any still running blurhash decoding
    pub(super) fn conclude_download(&mut self) {
        self.download_task = None;

        // The download has concluded so abort blurhash decoding
        if let Some(handle) = self.blurhash_task.take() {
            handle.abort();
        }
    }

    /// The blurhash task has concluded: drop its handle
    pub(super) fn conclude_blurhash(&mut self) {
        self.blurhash_task = None;
    }
}

/// The content which is currently present for an image
#[derive(Debug, Clone)]
pub enum Content {
    /// A decoded blurhash placeholder
    Blurhash(image::Handle),
    /// The full image
    Full(image::Handle),
    /// Loading the image failed
    Error,
}

/// The store holding the state of all images, keyed by their UUID.
///
/// It can be accessed using [`Store::global`]
#[derive(Default)]
struct Store {
    states: RwLock<HashMap<Uuid, ImageState>>,
}

impl Store {
    /// The global image store
    pub fn global() -> &'static Self {
        static STORE: LazyLock<Store> = LazyLock::new(Store::default);
        &STORE
    }

    /// Returns a snapshot of the state of the image.
    ///
    /// Unlike before, this is a read-only lookup — it neither takes the write
    /// lock nor creates an empty entry for images which never did anything.
    pub fn get(&self, uuid: Uuid) -> ImageState {
        self.states
            .read()
            .expect("Image store poisoned")
            .get(&uuid)
            .cloned()
            .unwrap_or_default()
    }

    /// Mutates the state of the image, creating the entry if necessary
    pub fn mutate<R>(&self, uuid: Uuid, f: impl FnOnce(&mut ImageState) -> R) -> R {
        let mut states = self.states.write().expect("Image store poisoned");
        f(states.entry(uuid).or_default())
    }

    /// Stores new content for the image
    pub fn insert(&self, uuid: Uuid, content: Content) {
        self.mutate(uuid, |state| state.content = Some(content));
    }

    /// Registers that another instance of the image became visible
    pub fn show(&self, uuid: Uuid) {
        self.mutate(uuid, |state| state.visible += 1);
    }

    /// Registers that an instance of the image was hidden.
    ///
    /// If the image is hidden before it was fully downloaded, the download is
    /// aborted, but only once the last visible instance is gone, so that
    /// multiple instances of the same URL don't kill each other's download.
    pub fn hide(&self, uuid: Uuid) {
        self.mutate(uuid, |state| {
            state.visible = state.visible.saturating_sub(1);

            if state.visible == 0
                && let Some(handle) = state.download_task.take()
            {
                handle.abort();
            }
        });
    }

    /// Spawns the download of the image at the given URL, replacing
    /// any previously running download for the same image
    pub fn download_task(
        &self,
        uuid: Uuid,
        url: Url,
        measured_size: Size,
        resolution_strategy: Option<fn(Url, Size) -> Url>,
    ) -> Task<ImgMsg> {
        let (task, handle) = Task::future(download(url, measured_size, resolution_strategy))
            .and_then(|bytes| {
                // After downloading directly allocate the image so we only start
                // overwriting the Blurhashes once the image is fully allocated.
                // This way we get a guarantee that the image is actually drawn once
                // we replace the Blurhash, instead of being deferred to the renderer.
                //
                // > [`image::allocate`]:
                // > When you obtain an Allocation explicitly, you get the guarantee that
                // > using a Handle will draw the corresponding image immediately in the
                // > next frame.
                image::allocate(image::Handle::from_bytes(bytes))
                    .map_err(|_| Error::ImageDecodingFailed)
            })
            .and_then(|a| Task::done(Ok(a.handle().clone())))
            .map(ImgMsg::ImageDownloaded)
            .abortable();

        self.mutate(uuid, |state| {
            state.set_download_task(handle.abort_on_drop());
        });

        task
    }

    /// Spawns the decoding of the image's blurhash placeholder, replacing
    /// any previously running decoding for the same image
    pub fn blurhash_task(&self, uuid: Uuid, blurhash: String, measured_size: Size) -> Task<ImgMsg> {
        let (task, handle) = Task::future(decode_blurhash(blurhash, measured_size))
            .and_then(|rgba| {
                let Rgba {
                    width,
                    height,
                    pixels,
                } = rgba;

                Task::done(Ok(
                    // Unlike the download task this time we only create the handle
                    // and defer the allocation to the renderer. This way we can
                    // decode all blurhashes as fast as possible without blocking
                    // ourselves by also allocating them.
                    image::Handle::from_rgba(width, height, pixels),
                ))
            })
            .map(ImgMsg::BlurhashDecoded)
            .abortable();

        self.mutate(uuid, |state| {
            state.set_blurhash_task(handle.abort_on_drop());
        });

        task
    }
}

/// Asynchronously downloads the image from the URL,
/// automatically downloading a sized version based on the resolution strategy
async fn download(
    url: Url,
    measured_size: Size,
    resolution_strategy: Option<fn(Url, Size) -> Url>,
) -> Result<Bytes, Error> {
    // Change the image URL to be sized if there is a resolution strategy available
    let url = if let Some(resize_strat) = resolution_strategy {
        resize_strat(url, measured_size)
    } else {
        url
    };

    let request = CLIENT.get(url);
    let bytes = request.send().await?.error_for_status()?.bytes().await?;

    Ok(bytes)
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
            pixels: pixels.into(),
        })
    })
    .await?
}

/// A network image which automatically fetches the image data from a given URL
///
/// This struct only holds the *configuration* of the image. All state — the
/// downloaded content, running tasks and visibility — is stored per-URL in
/// the global [`Store`] and shared between every instance of the same image.
#[derive(Debug, Clone)]
pub struct Image {
    /// The Url of this image
    url: Url,
    /// The [`Uuid`] of this image, derived from its URL.
    uuid: Uuid,
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
    /// Duration to wait before we start decoding the blurhash (if any)
    blurhash_debounce: Option<Duration>,
    /// Duration to wait before we start to download the actual image
    ///
    /// This is relevant e.g. on tables where scrolling over items will show a
    /// huge amount of images, hence normally triggering a huge amount of
    /// downloads
    download_debounce: Option<Duration>,
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
    /// Whether *this instance* is currently on screen
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

impl Image {
    /// Creates a new image
    pub fn new(url: impl Into<Url>) -> Self {
        let url = url.into();
        // We derive the Uuid from the url
        //
        // TODO: Consider ignoring parameters like size in the url so that the same
        // Image is share (greatest size wins); This would constitute a rewrite of the
        // store where we'd have to store the resolution of the content as well
        let uuid = Uuid::new_v3(&Uuid::NAMESPACE_URL, url.as_str().as_bytes());

        Image {
            uuid,
            url,
            blurhash: None,
            resolution_strategy: None,
            measured_size: Size {
                width: 16.0,
                height: 16.0,
            },
            download_debounce: None,
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
    /// [`Image::resolution_strategy`]
    pub fn responsive_resolution(mut self, resp_res: bool) -> Self {
        self.responsive_resolution = resp_res;
        self
    }

    /// Decodes the blurhash (if any) beforehand;
    /// This is a blocking operation and should be treated as such
    #[deprecated = "let the blurhash calculate dynamically"]
    pub fn pre_decode_blurhash(self, width: u32, height: u32) -> Self {
        // Do nothing if we already have content stored
        if self.get_state().content.is_some() {
            return self;
        }

        if let Some(blurhash) = &self.blurhash
            && let Ok(pixels) = blurhash::decode(blurhash, width, height, 1.0)
        {
            let handle = image::Handle::from_rgba(width, height, pixels);
            Store::global().insert(self.uuid, Content::Blurhash(handle));
        }
        self
    }

    /// Debounces the fetching of the image
    pub fn debounce(mut self, duration: Duration) -> Self {
        self.download_debounce = Some(duration);
        self
    }

    /// Debounces the rendering of the blurhash
    pub fn debounce_blurhash(mut self, duration: Duration) -> Self {
        self.blurhash_debounce = Some(duration);
        self
    }

    /// A snapshot of the state shared between all instances of this image
    pub fn get_state(&self) -> ImageState {
        Store::global().get(self.uuid)
    }

    /// Mutates the state shared between all instances of this image
    pub fn with_state_mut<R>(&self, f: impl FnOnce(&mut ImageState) -> R) -> R {
        Store::global().mutate(self.uuid, f)
    }
}

impl Image {
    pub fn view(&self) -> Element<'_, ImgMsg> {
        let image: Element<'_, ImgMsg> = match self.get_state().content {
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
        let store = Store::global();
        let uuid = self.uuid;

        match message {
            // Once the image is shown / size is changed
            ImgMsg::Shown(size) => {
                if !self.visible {
                    self.visible = true;
                    store.show(uuid);
                }

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
                let state = store.get(uuid);

                // Start rendering a blurhash if there currently is no content
                let blurhash_timer = if self.blurhash.is_some() && state.content.is_none() {
                    debounce_timer(self.blurhash_debounce, ImgMsg::DebounceBlurhash(size))
                } else {
                    Task::none()
                };

                // Start downloading the image if there is currently no image
                let render_timer = if matches!(state.content, None | Some(Content::Blurhash(_))) {
                    debounce_timer(self.download_debounce, ImgMsg::DebounceImage(size))
                } else {
                    Task::none()
                };

                Task::batch([blurhash_timer, render_timer])
            }
            // This runs AFTER the debounce
            ImgMsg::DebounceBlurhash(_size) => {
                // If the image is still visible and ...
                if self.visible && store.get(uuid).content.is_none() {
                    return self.blurhash_task();
                }

                Task::none()
            }
            // This runs AFTER the debounce
            ImgMsg::DebounceImage(_size) => {
                // If the image is still visible and ...
                if self.visible
                    && matches!(store.get(uuid).content, None | Some(Content::Blurhash(_)))
                {
                    return self.download_task();
                }

                Task::none()
            }
            ImgMsg::Hidden => {
                self.visible = false;
                // Notify the store that this instance was hidden
                //
                store.hide(uuid);

                Task::none()
            }
            ImgMsg::Resized(size) => {
                // If auto sizing is active and the new resolution has greater dimensions than
                // the one we last fetched at
                if self.responsive_resolution
                    && self.resolution_strategy.is_some()
                    && size.width > self.measured_size.width
                    && size.height > self.measured_size.height
                {
                    // Then redownload / rerender
                    self.measured_size = size;

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
            ImgMsg::BlurhashDecoded(result) => {
                match result {
                    Ok(handle) => {
                        store.insert(uuid, Content::Blurhash(handle));
                    }
                    // A broken blurhash must not poison the image itself;
                    // the real image may still download just fine
                    Err(error) => log::warn!("Failed to decode blurhash: {error:?}"),
                }

                store.mutate(uuid, ImageState::conclude_blurhash);
                Task::none()
            }
            ImgMsg::ImageDownloaded(result) => {
                match result {
                    Ok(handle) => {
                        store.insert(uuid, Content::Full(handle));
                    }
                    Err(_error) => {
                        // Only show the error placeholder if there is
                        // nothing else to display
                        store.mutate(uuid, |state| {
                            if state.content.is_none() {
                                state.content = Some(Content::Error);
                            }
                        });
                    }
                }

                store.mutate(uuid, ImageState::conclude_download);
                Task::none()
            }
        }
    }

    /// Spawns a download of this image through the global [`Store`]
    fn download_task(&self) -> Task<ImgMsg> {
        Store::global().download_task(
            self.uuid,
            self.url.clone(),
            self.measured_size,
            self.resolution_strategy,
        )
    }

    /// Spawns a blurhash decoding of this image through the global [`Store`]
    fn blurhash_task(&self) -> Task<ImgMsg> {
        Store::global().blurhash_task(
            self.uuid,
            self.blurhash
                .clone()
                .expect("The blurhash_task function should only be called when blurhash is Some"),
            self.measured_size,
        )
    }
}

/// If an Image is dropped it can no longer be displayed hence no sensor will
/// emit hidden, meaning we should decrease the visibility counter ourselves
impl Drop for Image {
    fn drop(&mut self) {
        Store::global().hide(self.uuid);
    }
}

/// A task which emits `msg` after the given `debounce` duration
/// (or immediately, if debounce is [`None`])
fn debounce_timer(debounce: Option<Duration>, msg: ImgMsg) -> Task<ImgMsg> {
    match debounce {
        Some(debounce) => Task::future(async move {
            tokio::time::sleep(debounce).await;
            Some(msg)
        })
        .and_then(Task::done),
        None => Task::done(msg),
    }
}

#[derive(Clone)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Bytes,
}

// We don't want to log all the pixels
#[allow(clippy::missing_fields_in_debug)]
impl fmt::Debug for Rgba {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Rgba")
            .field("width", &self.width)
            .field("height", &self.height)
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
