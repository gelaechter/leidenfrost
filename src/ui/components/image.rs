use iced::{
    ContentFit, Element,
    Length::Fill,
    Size, Task,
    task::{self},
    widget::{
        self, container,
        image::{self},
    },
};
use iced_fonts::lucide;
use sea_orm::ExprTrait;
use serde::Deserialize;
use serde_json::json;
use std::{
    fmt,
    hash::Hash,
    io,
    sync::{Arc, LazyLock},
};
use url::Url;
use uuid::Uuid;

use crate::backend::api::endpoint_api::Endpoint;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

/// An image that will automatically try to fetch the correct image size from the API
/// this is done through the ImageSize trait of the API.
/// An ImageSize implementor can be passed to the Image as a generic
#[derive(Debug, Clone)]
pub struct Image {
    url: Url,
    /// The last max size of the image
    size: Size<u32>,
    /// The images blur hash (if any)
    blurhash: Option<String>,
    // TODO: store a proper state with error handling
    /// An image resizer if the endpoint supports auto sizing
    image_resizer: Option<Endpoint>,
    /// The currently allocated image data (if any)
    status: Option<image::Handle>,
    /// Any running download task
    _download_task: Option<task::Handle>,
    /// Any running blurhash decoding task
    _blurhash_task: Option<task::Handle>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ImageHidden,
    /// The image has become visible
    ImageShown(Size),
    /// The image was resized
    ImageResized(Size),
    /// Message that the image has been downloaded and allocated
    ImageDownloaded(Result<image::Handle, Error>),
}

impl Image {
    pub fn uuid(url: &Url) -> Uuid {
        Uuid::new_v3(
            &Uuid::NAMESPACE_URL,
            // TODO: Only use base-path / disregard any queries
            url.as_str().as_bytes(),
        )
    }

    /// Creates a new image
    pub fn new(url: Url, blurhash: Option<String>) -> (Uuid, Self) {
        (
            Self::uuid(&url),
            Image {
                url,
                blurhash,
                image_resizer: None,
                size: Size::default(),
                status: None,
                _download_task: None,
                _blurhash_task: None,
            },
        )
    }

    pub fn new_autosized(url: Url, blurhash: Option<String>, resizer: Endpoint) -> (Uuid, Self) {
        (
            Self::uuid(&url),
            Image {
                url,
                blurhash,
                size: Size::default(),
                image_resizer: Some(resizer),
                status: None,
                _download_task: None,
                _blurhash_task: None,
            },
        )
    }

    pub fn view(&self) -> Element<'_, Message> {
        let image: Element<'_, Message> = if let Some(handle) = &self.status {
            // Either the blurhash or the image have been loaded
            widget::image(handle)
                // Use width an height instead of expand to enforce full size and prevent resizing
                .width(Fill)
                .height(Fill)
                .content_fit(ContentFit::Cover)
                .border_radius(8)
                .into()
        } else {
            // No blurhash / image
            // TODO: replace this with a loading symbol if we are currently loading the image
            lucide::image_off().size(16).into()
        };

        let sensor = widget::sensor(image)
            .on_resize(Message::ImageResized)
            .on_show(Message::ImageShown)
            .on_hide(Message::ImageHidden);

        widget::container(sensor)
            .style(container::rounded_box)
            .clip(true)
            .center(64)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Once the image is shown / size is changed
            Message::ImageShown(size) => {
                let size = Size {
                    width: size.width.round() as u32,
                    height: size.height.round() as u32,
                };

                self.size = size;

                // Abort if we're already working
                if self._download_task.is_some() || self.status.is_some() {
                    return Task::none()
                };

                match &self.blurhash {
                    // Either just download
                    None => self.download_task(),
                    // Or download and decode blurhash
                    Some(_) => Task::batch([self.blurhash_task(), self.download_task()]),
                }
            }
            Message::ImageHidden => {
                // if let Some(download_task) = &self._download_task.take() {
                //     download_task.abort();
                // };
                Task::none()
            }
            Message::ImageResized(size) => {
                // If autosizing is active and the new resolution has greater dimensions
                // then redownload the new resolution
                if self.image_resizer.is_some()
                    && size.width > self.size.width as f32
                    && size.height > self.size.height as f32
                {
                    Task::done(Message::ImageShown(size))
                } else {
                    Task::none()
                }
            }
            Message::ImageDownloaded(handle) => {
                if let Ok(handle) = handle {
                    self.status = Some(handle);
                    // Download has concluded so abort blurhash decoding
                    if let Some(blurhash_task) = &self._blurhash_task.take() {
                        blurhash_task.abort();
                    }
                }
                // TODO: Error handling

                Task::none()
            }
        }
    }

    /// Asynchronously downloads the image from the url
    /// automatically downloads a sized version based on the resizer
    pub async fn download(
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

        // Resizer::set_image_resolution is an identity function for NoAutoSize
        let bytes = request.send().await?.error_for_status()?.bytes().await?;

        Ok(Bytes(bytes))
    }

    /// Asynchronously decodes the blurhash for the given size
    pub async fn blurhash(blurhash: String, size: Size<u32>) -> Result<Rgba, Error> {
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

    pub fn download_task(&mut self) -> Task<Message> {
        let Self {
            url,
            size,
            image_resizer,
            _download_task,
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
                    // blurhashes once the image is fully allocated, this way we get a guarantee that the
                    // image is actually drawn once we replace the blurhash instead of being deferred to the
                    // renderer
                    //
                    // > When you obtain an Allocation explicitly, you get the guarantee that using a Handle
                    // > will draw the corresponding image immediately in the next frame.
                    image::allocate(image::Handle::from_bytes(bytes))
                        .map_err(|_| Error::ImageDecodingFailed)
                })
                .and_then(|a| Task::done(Ok(a.handle().clone())))
                .map(Message::ImageDownloaded)
                .abortable();

        // Set download handle and return value
        *_download_task = Some(download_handle.abort_on_drop());
        download_task
    }

    fn blurhash_task(&mut self) -> Task<Message> {
        let Self {
            size,
            blurhash,
            _blurhash_task,
            ..
        } = self;

        // Abort previous task (if any)
        if let Some(blurhash_handle) = _blurhash_task.take() {
            blurhash_handle.abort();
        }

        // Decode blurhash
        let (blurhash_task, blurhash_handle) = Task::future(Self::blurhash(
            blurhash
                .clone()
                .expect("The blurhash_task function should only be called when blurhash is Some")
                .to_string(),
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
        .map(Message::ImageDownloaded)
        .abortable();

        // Set blurhash handle and return value
        self._blurhash_task = Some(blurhash_handle.abort_on_drop());
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
