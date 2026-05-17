use iced::{
    ContentFit, Element,
    Length::{self, Fill},
    Size, Task,
    task::{self},
    widget::{
        self,
        image::{self},
    },
};
use iced_fonts::lucide;
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::HashMap,
    fmt,
    hash::Hash,
    io,
    sync::{Arc, LazyLock},
    time::Duration,
};
use url::Url;
use uuid::Uuid;

use crate::backend::api::endpoint_api::Endpoint;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[derive(Default)]
pub struct Manager {
    images: HashMap<Uuid, Image>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ImageDriver(Uuid, IMessage),
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

/// An image that will automatically try to fetch the correct image size from
/// the API this is done through the ImageSize trait of the API.
/// An ImageSize implementer can be passed to the Image as a generic
#[derive(Debug, Clone)]
pub struct Image {
    /// The Url of this image
    /// TODO: Add a global Handle cache for Urls that are the same.
    url: Url,
    /// TODO:
    /// The preferred height of the image
    height: Option<Length>,
    /// TODO:
    /// The preferred width of the image
    width: Option<Length>,
    /// TODO:
    /// An aspect ratio determining width/height
    aspect_ratio: Option<f32>,
    /// The size of the image container
    container_size: Size<u32>,
    /// Duration to wait before registering image visibility
    debounce: Option<Duration>,
    /// The images blur hash (if any)
    blurhash: Option<String>,
    image_resizer: Option<Endpoint>,
    /// The currently allocated image data (if any)
    status: Option<Content>,
    /// Any running download task
    download_task: Option<task::Handle>,
    /// Any running blurhash decoding task
    blurhash_task: Option<task::Handle>,
}

#[derive(Debug, Clone)]
pub enum IMessage {
    Hidden,
    /// The image has become visible
    Shown(Size),
    /// The image was resized
    Resized(Size),
    /// Message that the image has been downloaded and allocated
    BlurhashDecoded(Result<image::Handle, Error>),
    /// Message that the image has been downloaded and allocated
    Downloaded(Result<image::Handle, Error>),
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
            image_resizer: None,
            container_size: Size {
                width: 16,
                height: 16,
            },
            status: None,
            debounce: None,
            download_task: None,
            blurhash_task: None,
            height: None,
            width: None,
            aspect_ratio: None,
        }
    }

    /// Adds an optional blurhash to the image
    pub fn blurhash_maybe(mut self, blurhash: Option<String>) -> Self {
        self.blurhash = blurhash;
        self
    }

    /// Activates automatic sizing
    pub fn autosized(mut self, endpoint: Endpoint) -> Self {
        self.image_resizer = Some(endpoint);
        self
    }

    /// Decodes the blurhash (if any) beforehand
    /// This is a blocking operation and should be treated as such
    pub fn pre_decode_blurhash(mut self, width: u32, height: u32) -> Self {
        if let Some(blurhash) = &self.blurhash
            && let Ok(pixels) = blurhash::decode(blurhash, width, height, 1.0)
        {
            let handle = image::Handle::from_rgba(width, height, pixels);
            self.status = Some(Content::Blurhash(handle))
        }
        self
    }

    /// Debounces the visibility of the image
    ///
    /// This can be used together with [`Image::pre_decode_blurhash`] to
    /// immediately show a blurhash but only start fetching the actual image
    /// once it has been visible for a certain amount of time.
    pub fn debounce(mut self, duration: Duration) -> Self {
        self.debounce = Some(duration);
        self
    }
}

impl Image {
    pub fn view<'a>(&'a self) -> Element<'a, IMessage> {
        let image: Element<'_, IMessage> = match &self.status {
            Some(Content::Blurhash(handle)) | Some(Content::Full(handle)) => {
                // Either the blurhash or the image have been loaded
                widget::image(handle)
                    // Use width an height instead of expand to enforce full size and prevent
                    // resizing
                    .width(Fill)
                    .height(Fill)
                    .content_fit(ContentFit::Cover)
                    .border_radius(8)
                    .into()
            }
            Some(Content::Error) => lucide::image_off().size(16).into(),
            None => lucide::image_down().into(),
        };

        let mut sensor = widget::sensor(image)
            .on_resize(IMessage::Resized)
            .on_show(IMessage::Shown)
            .on_hide(IMessage::Hidden);

        // Set an optional delay to debounce
        if let Some(delay) = self.debounce {
            sensor = sensor.delay(delay)
        }

        sensor.into()
        // widget::container(sensor)
        //     .style(container::rounded_box)
        //     .clip(true)
        //     .center(64)
        //     .into()
    }

    pub fn update(&mut self, message: IMessage) -> Task<IMessage> {
        match message {
            // Once the image is shown / size is changed
            IMessage::Shown(size) => {
                // Determine initial size
                let size = Size {
                    width: size.width.round() as u32,
                    height: size.height.round() as u32,
                };
                if size.width < 1 || size.height < 1 {
                    return Task::none();
                }

                match self.status {
                    // No image and Blurhash exists
                    None if self.blurhash.is_some() => {
                        Task::batch([self.blurhash_task(), self.download_task()])
                    }
                    // No image and no blurhash
                    None => self.download_task(),
                    // Blurhash rendered
                    Some(Content::Blurhash(_)) => self.download_task(),
                    // Do nothing if image already loaded or errored
                    Some(Content::Full(_) | Content::Error) => Task::none(),
                }
            }
            IMessage::Hidden => {
                if let Some(download_task) = &self.download_task.take() {
                    download_task.abort();
                };
                Task::none()
            }
            IMessage::Resized(size) => {
                // If auto sizing is active and the new resolution has greater dimensions
                // then redownload the new resolution
                if self.image_resizer.is_some()
                    && size.width > self.container_size.width as f32
                    && size.height > self.container_size.height as f32
                {
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
            IMessage::BlurhashDecoded(handle) => {
                if let Ok(handle) = handle {
                    self.status = Some(Content::Blurhash(handle));
                } else {
                    self.status = Some(Content::Error);
                }
                self.blurhash_task.take();
                Task::none()
            }
            IMessage::Downloaded(handle) => {
                if let Ok(handle) = handle {
                    self.status = Some(Content::Full(handle));
                    // Download has concluded so abort blurhash decoding
                    if let Some(blurhash_task) = &self.blurhash_task.take() {
                        blurhash_task.abort();
                    }
                } else {
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
        size: Size<u32>,
        resizer: Option<Endpoint>,
    ) -> Result<Bytes, Error> {
        let request = CLIENT.get(url);

        // Choose
        let request = match resizer {
            Some(Endpoint::Jellyfin) => request.query(&json!({
                "width": size.width,
                "height": size.height,
            })),
            None => request,
        };

        let bytes = request.send().await?.error_for_status()?.bytes().await?;

        Ok(Bytes(bytes))
    }

    /// Asynchronously decodes the blurhash for the given size
    async fn decode_blurhash(blurhash: String, size: Size<u32>) -> Result<Rgba, Error> {
        tokio::task::spawn_blocking(move || {
            let pixels = blurhash::decode(&blurhash, size.width, size.height, 1.0)?;

            Ok(Rgba {
                width: size.width,
                height: size.height,
                pixels: Bytes(pixels.into()),
            })
        })
        .await?
    }

    fn download_task(&mut self) -> Task<IMessage> {
        let Self {
            url,
            container_size: size,
            image_resizer,
            download_task: _download_task,
            ..
        } = self;

        // Abort previous download (if any)
        if let Some(download_handle) = _download_task.take() {
            download_handle.abort();
        }

        // Start new download
        let (download_task, download_handle) =
            Task::future(Self::download(url.clone(), *size, image_resizer.clone()))
                .and_then(|bytes| {
                    // After downloading directly allocate the image so we only start overwriting the
                    // Blurhashes once the image is fully allocated. This way we get a guarantee that the
                    // image is actually drawn once we replace the Blurhash instead of being deferred to the
                    // renderer.
                    //
                    // > When you obtain an Allocation explicitly, you get the guarantee that using a Handle
                    // > will draw the corresponding image immediately in the next frame.
                    image::allocate(image::Handle::from_bytes(bytes))
                        .map_err(|_| Error::ImageDecodingFailed)
                })
                .and_then(|a| Task::done(Ok(a.handle().clone())))
                .map(IMessage::Downloaded)
                .abortable();

        // Set download handle and return value
        *_download_task = Some(download_handle.abort_on_drop());
        download_task
    }

    fn blurhash_task(&mut self) -> Task<IMessage> {
        let Self {
            container_size: size,
            blurhash,
            blurhash_task: _blurhash_task,
            ..
        } = self;

        // Abort previous task (if any)
        if let Some(blurhash_handle) = _blurhash_task.take() {
            blurhash_handle.abort();
        }

        // Decode blurhash
        let (blurhash_task, blurhash_handle) = Task::future(Self::decode_blurhash(
            blurhash
                .clone()
                .expect("The blurhash_task function should only be called when blurhash is Some")
                .clone(),
            *size,
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
        .map(IMessage::BlurhashDecoded)
        .abortable();

        // Set blurhash handle and return value
        self.blurhash_task = Some(blurhash_handle.abort_on_drop());
        blurhash_task
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct Id(u32);

#[derive(Debug, Clone)]
pub struct Blurhash {
    pub rgba: Rgba,
}

#[derive(Clone)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Bytes,
}

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
