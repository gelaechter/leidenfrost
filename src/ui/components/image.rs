use iced::{
    ContentFit, Element,
    Length::Fill,
    Size, Task,
    task::{self},
    widget::{
        self,
        image::{self, Allocation},
    },
};
use iced_fonts::lucide;
use sea_orm::UuidColumn;
use serde::Deserialize;
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
    // An image resizer if the endpoint supports auto sizing
    image_resizer: Option<Endpoint>,
    /// The currently allocated image data (if any)
    _allocation: Option<Allocation>,
    // Any running download task
    _download_task: Option<task::Handle>,
    // Any running blurhash decoding task
    _blurhash_task: Option<task::Handle>,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// The image has become visible
    ImageRendered(Size),
    /// The image was resized
    ImageResized(Size),
    /// Message that the image has been downloaded and allocated
    ImageDownloaded(Result<Allocation, Error>),
}

impl Image {
    pub fn uuid(url: &Url) -> Uuid {
        Uuid::new_v3(&Uuid::NAMESPACE_URL, url.as_str().as_bytes())
    }

    pub fn new(url: Url, blurhash: Option<String>) -> (Uuid, Self) {
        (
            Self::uuid(&url),
            Image {
                url,
                blurhash,
                image_resizer: None,
                size: Size::default(),
                _allocation: None,
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
                _allocation: None,
                _download_task: None,
                _blurhash_task: None,
            },
        )
    }

    pub fn view(&self) -> Element<'_, Message> {
        let image: Element<'_, Message> = if let Some(alloc) = &self._allocation {
            // Either the blurhash or the image have been loaded
            widget::image(alloc.handle())
                .expand(true)
                .content_fit(ContentFit::Contain)
                .into()
        } else {
            // No blurhash / image
            lucide::image_off().into()
        };

        let sensor = widget::sensor(image)
            .on_resize(Message::ImageResized)
            .on_show(Message::ImageRendered);

        widget::container(sensor).center(64).into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ImageResized(size) => {
                // If autosizing is active and the new resolution has greater dimensions
                // then redownload the new resolution
                if self.image_resizer.is_some()
                    && size.width > self.size.width as f32
                    && size.height > self.size.height as f32
                {
                    Task::done(Message::ImageRendered(size))
                } else {
                    Task::none()
                }
            }
            // Once the image is shown / size is changed
            Message::ImageRendered(size) => {
                let size = Size {
                    width: size.width.round() as u32,
                    height: size.height.round() as u32,
                };

                self.size = size;

                Task::batch(match &self.blurhash {
                    // Either just download
                    None => vec![self.download_task()],
                    // Or download and decode blurhash
                    Some(_) => vec![self.download_task(), self.blurhash_task()],
                })
            }
            Message::ImageDownloaded(allocation) => {
                if let Ok(allocation) = allocation {
                    self.size = allocation.size();
                    self._allocation = Some(allocation);
                    // Download has concluded so abort blurhash decoding
                    if let Some(blurhash_task) = &self._blurhash_task {
                        blurhash_task.abort();
                        self._blurhash_task = None;
                    }
                }

                Task::none()
            }
        }
    }

    fn blurhash_task(&mut self) -> Task<Message> {
        let Self {
            size,
            blurhash,
            _blurhash_task,
            ..
        } = self;

        // Abort previous task (if any)
        if let Some(blurhash_handle) = &self._blurhash_task {
            blurhash_handle.abort();
            self._blurhash_task = None;
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

            image::allocate(image::Handle::from_rgba(width, height, pixels))
                .map_err(|_| Error::ImageDecodingFailed)
        })
        .map(Message::ImageDownloaded)
        .abortable();

        // Set blurhash handle and return value
        self._blurhash_task = Some(blurhash_handle);
        blurhash_task
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
        if let Some(download_handle) = _download_task {
            download_handle.abort();
            *_download_task = None;
        }

        // Start new download
        let (download_task, download_handle) =
            Task::future(Self::download(url.clone(), *size, image_resizer.clone()))
                // After downloading decode image
                .and_then(|bytes| {
                    image::allocate(image::Handle::from_bytes(bytes))
                        .map_err(|_| Error::ImageDecodingFailed)
                })
                .map(Message::ImageDownloaded)
                .abortable();

        // Set download handle and return value
        *_download_task = Some(download_handle);
        download_task
    }

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

    /// If the image resizer:
    /// ```rust,no_exec
    /// pub struct Image<Resizer: ImageSize = NoAutoSize>
    /// ```
    /// has the default value then width and height are ignored
    pub async fn download(
        url: Url,
        size: Size<u32>,
        resizer: Option<Endpoint>,
    ) -> Result<Bytes, Error> {
        let request = CLIENT.get(url);

        // Choose
        let request = match resizer {
            Some(Endpoint::Jellyfin) => todo!(),
            None => request,
        };

        // Resizer::set_image_resolution is an identity function for NoAutoSize
        let bytes = request.send().await?.error_for_status()?.bytes().await?;

        Ok(Bytes(bytes))
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
