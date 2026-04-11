use iced::{
    ContentFit, Element,
    Length::Fill,
    Size, Task,
    debug::time_with,
    task::{self},
    widget::{
        self, grid,
        image::{self},
        scrollable,
    },
};
use iced_fonts::lucide;
use serde::Deserialize;
use std::{
    collections::HashMap,
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
    ImageRendered(Size),
    /// The image was resized
    ImageResized(Size),
    /// Message that the image has been downloaded and allocated
    ImageDownloaded(Result<image::Handle, Error>),
}

impl Image {
    pub fn uuid(url: &Url) -> Uuid {
        Uuid::new_v3(&Uuid::NAMESPACE_URL, url.as_str().as_bytes())
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
                .content_fit(ContentFit::Contain)
                .into()
        } else {
            // No blurhash / image
            widget::container(lucide::image_off()).center(64).into()
        };

        let sensor = widget::sensor(image)
            .on_resize(Message::ImageResized)
            .on_show(Message::ImageRendered)
            .on_hide(Message::ImageHidden);

        widget::container(sensor).center(64).into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ImageHidden => {
                if let Some(download_task) = &self._download_task.take() {
                    download_task.abort();
                };
                Task::none()
            }
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

                match &self.blurhash {
                    // Either just download
                    None => self.download_task(),
                    // Or download and decode blurhash
                    Some(_) => Task::batch([self.blurhash_task(), self.download_task()]),
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
            Some(Endpoint::Jellyfin) => todo!(),
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

#[allow(dead_code)]
pub fn test_images_concurrency() {
    #[derive(Clone)]
    enum Message {
        ImageDriver(Uuid, super::image::Message),
    }

    impl std::fmt::Debug for Message {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::ImageDriver(arg0, arg1) => f
                    .debug_tuple("ImageDriver")
                    .field(&arg0.to_string().chars().take(5).collect::<String>())
                    .field(arg1)
                    .finish(),
            }
        }
    }

    struct State {
        images: HashMap<Uuid, Image>,
    }

    fn new() -> State {
        let dbg_tracks = [
            (
                "http://***REMOVED***/Items/75b12630de8c17f36d3d740f13b8d977/Images/Primary",
                "VEH2vTMKEj}@k?9HM^w^cF%0Mwo#$eT0jHs6s=g4Z}kE",
            ),
            (
                "http://***REMOVED***/Items/40cdc0e32fce006fd1a35f8279d9f442/Images/Primary",
                "ecIgi-KiM|Rkb[~XELRPj?kU0hMdv~tOofIXxBt7s+NI%McCo|rYRj",
            ),
            (
                "http://***REMOVED***/Items/0699ffc70b41d571beede28099656fa0/Images/Primary",
                "e35OTvj[M{RjWB4nj[t7oft7xuofxuj[ofM{ayM{WBRj%NWBWBt7of",
            ),
            (
                "http://***REMOVED***/Items/01adfc080d38fb8cb258335f17e992a0/Images/Primary",
                "e:NAPIxuS2t7kC_4t7t7s;ofROW;ozjZWBo#WVWAWAWBt8t7bba#t7",
            ),
            (
                "http://***REMOVED***/Items/720e4b7ecb191681538f86f2d718ce4d/Images/Primary",
                "eyM%f%tRX8V@fi_4tRo#jFofRObct7jZV[f,WVe.aeWAs;t7bbjtog",
            ),
            (
                "http://***REMOVED***/Items/e1cc37297a4d8a47c00941de91687e4f/Images/Primary",
                "ec9awOs+NJNNjYkvRioct7ayIpWEt3oHWBngo#WEWCt7oboea#WCay",
            ),
            (
                "http://***REMOVED***/Items/11af0c7de96efb2f4129049048e27033/Images/Primary",
                "eCBVq-Rk0JIU^+r=X6ELNa$*hft7?I%2R5$QM|tlSfVY[CWBtRxaVs",
            ),
            (
                "http://***REMOVED***/Items/932f0cfa13c58f6d4a435c9ae7b47c18/Images/Primary",
                "euM6efxaIVs:Sh%2juRkf8oe0MR*tRWXn$jFayxZj@WBtQj[n$jsf8",
            ),
            (
                "http://***REMOVED***/Items/b03ecad437f736d1b9856dc9a4359099/Images/Primary",
                "eQE{kcNG9DaMWq%iM{k6RjaL4.jc-qbIt8s.ogo#ocM{%Mt8RkWAk9",
            ),
            (
                "http://***REMOVED***/Items/ee023e18ef9c85d1ffe4de55a8e23499/Images/Primary",
                "epMQSHjd_4-.t6xvR%WBj?RjM|offgIoRk-;axRjRkRiWBR*Rjj[j]",
            ),
            (
                "http://***REMOVED***/Items/721bb002b738b47ed7036e06415c1908/Images/Primary",
                "eOM%_+xu?d%3IoM#f+${V@Ioo*ofIUV@%K?at6E2jrs;xtoJs+oeM|",
            ),
            (
                "http://***REMOVED***/Items/d0298958424f6bdbde47b70c1e3e69a5/Images/Primary",
                "VNLXGDuLyL-=qfkpJ%$+#ZJ*Ugv6,q$dIu=~;gv#tRxV",
            ),
            (
                "http://***REMOVED***/Items/dd5fb74731cd15bdcdf41858e7177e06/Images/Primary",
                "elEM:.M{D%oeoc_NM{M{oef5jERPt7ayWBNGWBoeWBofR-j@WAfkt7",
            ),
            (
                "http://***REMOVED***/Items/c0898c952ca472d485de3423e55a9c29/Images/Primary",
                "eI9*lNoz4UWUMy?ukBM{kCMe8_WB.RofV@4TWB%gay%fE0ayozay%M",
            ),
            (
                "http://***REMOVED***/Items/0910087a764df56e462e6b7ebb93d9bc/Images/Primary",
                "ehKo19IAxvg3%K%jozt5o#js?coeV_V{WCnnjaR*V[WA-jWZj]s.fR",
            ),
            (
                "http://***REMOVED***/Items/74af3d579f8565c0b03bc2568e472a48/Images/Primary",
                "e87xtck=9GTOT2Q.I7xVxvxrMHI8%LoGVV^$x^k[D$IA4oR:-;RM%i",
            ),
            (
                "http://***REMOVED***/Items/8afd9e0681d0a3734ade4b2a48e8b258/Images/Primary",
                "VMCidWM|0zxaVuMys:bboKsm9aR.$*xZxtTJWVw]R-S4",
            ),
            (
                "http://***REMOVED***/Items/638527e692ed532473360db73b8276a7/Images/Primary",
                "eEHxf;L3vd]+9a?d0?%Ln3Ic*{,p${K3%N+[}%Vg%houDl.5non6xs",
            ),
            (
                "http://***REMOVED***/Items/12ec0d7ce427e8ee59680eda1045404c/Images/Primary",
                "eSIg$T^kr?z;Io=K^k9FZ$Wpt7D%M{Iox]0|o}M{9tV@yXS~xuo}Rj",
            ),
            (
                "http://***REMOVED***/Items/0e3ac82ff03b6bbec7692eae5dc54ae8/Images/Primary",
                "ePHKhpn4E,tnRlyGMdt4OGIUIuMxV?ayV=%NbFxZ%3IqtQRko#bvRQ",
            ),
            (
                "http://***REMOVED***/Items/5189b7f3e73bc3cc6c1e40d7207ea9e5/Images/Primary",
                "eQDliv0g$~Iq?F9wxtVtt7M|xaNJt6RQbbxuR+oeodoexuWXM|t7R*",
            ),
            (
                "http://***REMOVED***/Items/292a48d4cf8ee0bf84d3b1c768f528b0/Images/Primary",
                "eSKc-G~C=e%0wH^+%2IoRjRj56-BV?Vr$y^hniIVROj[vzaxW;NGXS",
            ),
            (
                "http://***REMOVED***/Items/349a87fa3bd559e0b3c0f9f901deece7/Images/Primary",
                "e9EJ*-J-1b-BEL#.%2soiwni$*=eJkIoRjX-N^NGJ*n4OXozWoM{ni",
            ),
            (
                "http://***REMOVED***/Items/aca7020c9ae72faad872f51128521f9f/Images/Primary",
                "e#HeOWM|S4WVae-oWBShbGae~pWCR*kCRkodR+s:WWay%gj[V[fQay",
            ),
            (
                "http://***REMOVED***/Items/69d1075c8165e515a0879d368c25e42b/Images/Primary",
                "VRFiJmn4ROx^NeOaXTM{xtxu0KtRV@aeoJ-TaJn$I;NG",
            ),
            (
                "http://***REMOVED***/Items/5ba7af4ef9c63d8abd4b2caade7e84dc/Images/Primary",
                "eOO4Ynt8BB%hR*u6%Nx^tSt7FyM{Q-Rja0yYozi_j[n*g4kWs.niX9",
            ),
            (
                "http://***REMOVED***/Items/862329cdebe97304f187454ba675a41f/Images/Primary",
                "edJwBhkC3fkC+hKJayeojZbuJ5ayxGjZV@pHj[VtayoLS0a{r]aySx",
            ),
            (
                "http://***REMOVED***/Items/55f2243d4a13dba5fb36e23c4ca165e1/Images/Primary",
                "eSI#4ZS$3?vVGZ-qr;Orobs90?S~,[S_+Ko{S#SxXRS_RortwHoInh",
            ),
            (
                "http://***REMOVED***/Items/ec0f197a62f4943a2bc71ea509658984/Images/Primary",
                "VNBhE4Q-McW9E1LNVXR4xas:%OVqS2S0jEITRPtSt7jG",
            ),
            (
                "http://***REMOVED***/Items/1054ff638f740aea2e74b788d50cfa51/Images/Primary",
                "eyH{1,b}t8s.bI~pInNHbcWC^%xYRkofWB-:xujFs,WCxtt6j?WVae",
            ),
            (
                "http://***REMOVED***/Items/a4e251a2402ce84950973117bf14e1bb/Images/Primary",
                "eeN,uixu_3bF~qaLr]f6S1%M?bo2bukVDjt7X7x[oyRj%gX6IUt7ae",
            ),
            (
                "http://***REMOVED***/Items/b4747e06ea915d73600a6241bf7017d2/Images/Primary",
                "eNRA3k%3=k-EEu$,oMo3axj[$VjcJeWmwk-EaxW.j[jGxaoMNGWUs:",
            ),
            (
                "http://***REMOVED***/Items/c5c90328ee1b8a6164c373edc3d01f49/Images/Primary",
                "eLCsjuxu0JxutS_3M{4nt7xuRiM{Rit7t7MxWBt7ofo#ozogf6WBj]",
            ),
            (
                "http://***REMOVED***/Items/37724c694f3e17f2bd8463ebb0b73942/Images/Primary",
                "emJizC=^xYo0NI~7t6S4a#R+-moeS4azazbakCoLj[azofWqR,jss.",
            ),
            (
                "http://***REMOVED***/Items/fa3d12d55954e95a135521edb14522c6/Images/Primary",
                "eA9$,%~BK5S6RQ~B^*-pa$E2BUx]-Uf5X85RI=X8oe$%9u9uM|%1%1",
            ),
            (
                "http://***REMOVED***/Items/ea65abc1a45063b1d67b6b990d0f025b/Images/Primary",
                "eDAIyncF|E#8=Q%PxHx8sp$-=#sl=Os?%3-qof$mn+N?=Zo2%5t8W;",
            ),
            (
                "http://***REMOVED***/Items/755d43a0cfae207f3d1540c508a0da2d/Images/Primary",
                "ekMsKjV@DQW.e;^nt7jJxaRjDjae%zRjoLg2WnRij@axtQWCWUWAof",
            ),
            (
                "http://***REMOVED***/Items/1213fd024dd482973e5ca72677515763/Images/Primary",
                "eFAJdYk=Kk%2Im7%o2xbXT#Q#4RoZ|aJE%EIs;aHRh$,RMa_jXacT1",
            ),
            (
                "http://***REMOVED***/Items/ce74f0dec44f07d21afea999111213ca/Images/Primary",
                "eJJ@OYEMv|5U58_Nv%D*xvsV0x9axs?GIoE4jaxaIoRiIU%2Ior=xu",
            ),
            (
                "http://***REMOVED***/Items/732577560dde4ecb74cfc71f9daa106a/Images/Primary",
                "eA9$8[=wRk=c-UKMxZ-U-Aw^{|,=%0xZxFKNW.nixFxZ;#$h$NnPr?",
            ),
            (
                "http://***REMOVED***/Items/2f853f24ce212ce4e9712dbf41f8701d/Images/Primary",
                "evJbR3x]I8xa9GD$WVxbV@t78^RjWZX4ozVsj[R*WCoeRjbFx]oNs:",
            ),
            (
                "http://***REMOVED***/Items/48e4ab1d5f01ec893e8d5a040879667b/Images/Primary",
                "ejKTrUbI$ys,%M.mt6WqkCt7R.W=S$ozIV%0WC%1j@RPOYoeW;bHax",
            ),
            (
                "http://***REMOVED***/Items/13ff61bc9e17f2d484c8293ed98a81f2/Images/Primary",
                "e4H[Ee}EDT#Doc]no0smWDWC9KnR%ctOoL+|jboeocazo_oeaNRSkA",
            ),
            (
                "http://***REMOVED***/Items/8f03fa6d6a303bc3bbc938954c8fe767/Images/Primary",
                "eSD]eK%14XRjjqROWBa$o3%K05WX?Xoej[x^WrWTk8DlITRjW?t7R.",
            ),
            (
                "http://***REMOVED***/Items/76d8b284b4feeaa6d454109f105a6f64/Images/Primary",
                "egM*5Gxu-p%Mxu-pj[WCfkof~pofR%t6M|?aofjYs:M{WBfQR+WC%1",
            ),
            (
                "http://***REMOVED***/Items/fb88d2d649fe52c93c8d4e1a225daf1a/Images/Primary",
                "eUJP#m;5+^Q:R6~BnPxpV[R7%yM{Ip%1nl-pjERjnii{%Mj;VsjFn*",
            ),
            (
                "http://***REMOVED***/Items/510b679eb51078deb6a4fd21d02f2485/Images/Primary",
                "eHKecJIoln-:T_T[%Lt,WCv$%Mn~kCogaJ=zI:ITwdJ6qEo}eTnixH",
            ),
            (
                "http://***REMOVED***/Items/50b46cf748991bd1d8bcad4e11cd3b0e/Images/Primary",
                "eXI6+z-=5xOcET.9xcRRIpRj5xWGwFWTxUx^NXNfxct7XEa^WmS7s,",
            ),
            (
                "http://***REMOVED***/Items/7301915c212374df202d88bea8f4c836/Images/Primary",
                "eJD+r#=~?K%h-:tRs:SdIoW9s.W,SJbbM|-poMepR%s:X9WCW:Rjoc",
            ),
            (
                "http://***REMOVED***/Items/986f08731272a1d6143bc04ca195385c/Images/Primary",
                "e9A,zkM{~qxu?bt7WBofWBj[_3ayM{RjM{j[ayt7j[of-;ofRjj[of",
            ),
            (
                "http://***REMOVED***/Items/5930717dcf796973d157414dacc3de5e/Images/Primary",
                "etIN:KD+9GxvWB~UE2IVtRof%MWDV?SioeM|oLs.WBofV?W=bcsloM",
            ),
            (
                "http://***REMOVED***/Items/76f7cc8b4353bf837b9273e32b3d4d11/Images/Primary",
                "VeE0sH$*ACRje.$jW:NbsUoe1HI:$%t7f+EgsAxGN]WB",
            ),
            (
                "http://***REMOVED***/Items/b90962d640169eff7e9390af1a392dda/Images/Primary",
                "ePC%:5WF8{M|Me?]ofIVRjRP%3jaIUoff+rtbHMxa}x]MxoeR%j@xu",
            ),
            (
                "http://***REMOVED***/Items/d4ab130f1825c5eeac8ff0fb605bf194/Images/Primary",
                "eHJY*R-95bnU9+M-NwNIoH%I19n-jvNLxD=xjEWUavM~D?WGxqt2tQ",
            ),
            (
                "http://***REMOVED***/Items/88c937e3077e67076da8b8ac6e8e99d0/Images/Primary",
                "eH8g]Zt79DWBIVR+j[j[ays:DgWBp0az%MxtWVWBt7RjIVof-:ofIo",
            ),
            (
                "http://***REMOVED***/Items/13dbc96682cd067ffeaba1c5ac793168/Images/Primary",
                "eYMtjoD$%M-;of~WNGV[V@M{~pxvozNaM{E1xux[NGNGs9R*t6aeba",
            ),
            (
                "http://***REMOVED***/Items/d0de7a9bf3f22e59542afe827e8205d3/Images/Primary",
                "eCBxHA7KwJ$kE1-CxGS2j[WB0errofR*xuVYNGofn%t7Tx$*R*Sza0",
            ),
            (
                "http://***REMOVED***/Items/eba6905dd9626b7ec651b96e94c7eb88/Images/Primary",
                "eMO|kS$$IqNH%001D*e?MxxuIqRj%1IUWq%fxZogkDbIRjxuxutRWo",
            ),
            (
                "http://***REMOVED***/Items/a02f2dbb8fd5c8b1d98a4e2a55cd35a1/Images/Primary",
                "eeE3oes%NZj]RO-tt9oha$WUIhj=xZbcjwD,jxs.e:kCRhWBR$jdai",
            ),
            (
                "http://***REMOVED***/Items/4a47c3b8850e00b218db95e907c3c18b/Images/Primary",
                "eBAAaI~nouxtWB?X?Z%Lxtt6R-j[ofoet6Rna#ocobs.IUj[s.jFbF",
            ),
            (
                "http://***REMOVED***/Items/9555ba055b7c45838ac97d9f425b9bb3/Images/Primary",
                "eKE3F+IUM_xuIUR#xuRjoft7xuRjjvaya#RkoeRkWBoM00t7ofWBt7",
            ),
            (
                "http://***REMOVED***/Items/4b59ca2e2b620501c90dfeeb93d20d7b/Images/Primary",
                "enN0PS}7r=K5ax-?xZW.WVWBl;Avn%wJaytis9WCS5j[M_oKofn%bI",
            ),
            (
                "http://***REMOVED***/Items/e803eb575f7dd7739bfdfd4ba0b1ef38/Images/Primary",
                "eXCGGt%Nt5xvIo$]jXNGNFRoD$M{IUIUog%QkEV@xHoItKX5s:s,nl",
            ),
            (
                "http://***REMOVED***/Items/abefbd82acf6a1ea6d946f33755f88d6/Images/Primary",
                "eEB.s8O?Io=LtR0Le.xFNGNG?^$%icJ%kBMJS2kBniWByEofRkoMS~",
            ),
            (
                "http://***REMOVED***/Items/8646916206f702248835d8108c6f6cfc/Images/Primary",
                "eeN9#R?^PT-=ODPl-p-Vt0RR?7tK-Vt7V[#JtRtcV@NapaV_r?RoNL",
            ),
            (
                "http://***REMOVED***/Items/3f332bccc7be7069cbca36710fb16842/Images/Primary",
                "eML=tjF}.StmxC$S9vbwxDs+?uE4Q-k9Ne.80hR*xtniV]WB5ANeR+",
            ),
            (
                "http://***REMOVED***/Items/b5e5d04cfcd591d1dccacffa43c62013/Images/Primary",
                "eYIrHbtSRjxuR+~ptRWBj[a}IUWBofayj[?bM|oft7WB%MM|WBj@WB",
            ),
            (
                "http://***REMOVED***/Items/cc768cae4b73d3720a5c2161493effd9/Images/Primary",
                "eB8oMS}GK36MAW]:-BX7JRNaS#kC$j$PayN[W:$P,toLEzE|ay$Q,@",
            ),
            (
                "http://***REMOVED***/Items/7bbcd24d64ecdcaa7b0779bdc34bcb9a/Images/Primary",
                "eRDvTNxZITj?V?-.t3j?t6ad00ad-;oeofM{WBt6j[WX-=ogMyWCR-",
            ),
            (
                "http://***REMOVED***/Items/f444c08e22d9dcfdae47de664714b4a7/Images/Primary",
                "eVFhx7~p%Moyj[x@xuRjM{V[x[ofM|RjaysCofflj]ofozj[WAjut6",
            ),
            (
                "http://***REMOVED***/Items/9585ecd35a271b2317762c96ef53d3cd/Images/Primary",
                "eA9aNsIUDjx[IAy?t89ZSesC?bRjIojZM{-UoJt6WCS14.x[ozRjxb",
            ),
            (
                "http://***REMOVED***/Items/0157b692d6c94a5dd3028a1f91c78f2c/Images/Primary",
                "e65hx-xv4TWB%goio#x[ofRP8wWB.8ofRP.7ayDjWU%Mx^kCV@oeM{",
            ),
            (
                "http://***REMOVED***/Items/9b19ce45ed50768d290e91bf4a16ec45/Images/Primary",
                "eBAAH-%20LS4wy^+xFNHW=V?RijG%2s:Rks:s:oMaLs.wJoMo#n$rr",
            ),
            (
                "http://***REMOVED***/Items/84cdcaebf1db13f19a69d663f24ad287/Images/Primary",
                "eoHnEUt70KRjo|%2ofNGayW;0LWB-;ofniM{WBtQbHsoXSj[nin%S2",
            ),
            (
                "http://***REMOVED***/Items/50c9cbfe7978c941295d6f7e67ab1e85/Images/Primary",
                "e$Gb#SofWCofj[00WBj[WBay%Lj[j[j[azt7j[WBfRj[NGayayfQfQ",
            ),
            (
                "http://***REMOVED***/Items/06ede8d7e6058f687f53c1cf9f7b3afa/Images/Primary",
                "epAfRPozHXV@yDkWkWR5V@tlayV@ayV@f6tRozayozf6R5ayozayoz",
            ),
            (
                "http://***REMOVED***/Items/765581058a6b625823a7660bddf13df2/Images/Primary",
                "eYSY{q?bt7-;ay%Mofofayj[ofayofj[WB~qj[WBofay-;j[WBofj[",
            ),
            (
                "http://***REMOVED***/Items/51f57e62393dcc40938117d8a827e57f/Images/Primary",
                "ecFg?;-o}r-n^ja{xGWVxGazMxM|ShniNGR+ayR+j[n%xtS4WBj[oL",
            ),
            (
                "http://***REMOVED***/Items/457003740d4c848d9f710f93ac97ec04/Images/Primary",
                "eUA^taMwM{xuRP.ARORit7V@D+j[adfloIIUtRjYWCj?%MtRayV@bH",
            ),
            (
                "http://***REMOVED***/Items/06f409fd7dd12bec423a9ba44c52ae8a/Images/Primary",
                "e,97S}WFa$sqjGtDayj]jcjZoiawn$fSjZohb0sAb0a$bKbJs;WEa~",
            ),
            (
                "http://***REMOVED***/Items/2660b2451af2bbb6ae1515a445713a43/Images/Primary",
                "eVRW0bIUof~qxut7M{t7xuj[M{WBofWBWB-;Rjoft7t7%g%Mt7WBRj",
            ),
            (
                "http://***REMOVED***/Items/095d4ac239012e4b27c0c39bd2d62f4d/Images/Primary",
                "eMBz99oIDja#-:W@j[s;WBay01WBxuozIUxvWCM{t7s:%MogafRj-q",
            ),
            (
                "http://***REMOVED***/Items/97c1ed641593711526f2efdf11e79f3e/Images/Primary",
                "dTRo~gof_NIU?bofM{xvt7M{IUofRjt7%Mj[WARjWCoL",
            ),
            (
                "http://***REMOVED***/Items/056cc4aa2a962a30c6caa28d9c3c6ff5/Images/Primary",
                "exMis-%2yBxuV_#FoLs;sCoL0*f8RRRls,pbWXbukCjsNFW.r?o0WV",
            ),
            (
                "http://***REMOVED***/Items/b5201171d8d1b297d25f0cefb9c0f524/Images/Primary",
                "eM9%#n-=S%NeRk*0%goMRjjE?I%MWnRjWB%MxaWBRjWBOsSeV[i{jc",
            ),
            (
                "http://***REMOVED***/Items/e391023fe34a7471dc15b503eb0c8a40/Images/Primary",
                "eD9?zxcaAB$2m,}]T1E}-9wIRORjkB%1tRNZslxGozXmNasl$%xvXm",
            ),
            (
                "http://***REMOVED***/Items/f87818f914f07157b6aa2594baeb315f/Images/Primary",
                "eDEoAN560x?HI94.Xns:iwS54mxto#NbbFXSMw-p%3of9u%2aeNGxu",
            ),
            (
                "http://***REMOVED***/Items/f8fcd6acd50ce73b9e57661c88dd4ed4/Images/Primary",
                "VNJ$]X^v;B}n%Eret8-hwbakxbW.ELaL%1$+w4$z-$W=",
            ),
            (
                "http://***REMOVED***/Items/2463b7acfa0197ab1cddef28445d9d68/Images/Primary",
                "eLIsw9x^%e?vM|M#R6yDkq9Z0extShNGM{HBShrDMe%M}YRQIUn4Rl",
            ),
            (
                "http://***REMOVED***/Items/7c36cb896fe2cf849e75b9f30cbf30ce/Images/Primary",
                "eMPjAPxax[%fNa?a9FWqtRW;9Z%fRQMyWC_MWVs:niV@R+-;s:i_n$",
            ),
            (
                "http://***REMOVED***/Items/e32c13c0fedd76dc2ad4760c03018eca/Images/Primary",
                "e55XGSsmENS45T=wsnNcS4I=5TfkxDWC=w0%R,-9xE=vNdR+xYxFWV",
            ),
            (
                "http://***REMOVED***/Items/3e231331dd32cd64f66218ad2b45fb95/Images/Primary",
                "e99a22-;0K4nw^DiITX9xv?H0KM{?b-;IU%g-;-VD%9ZS3WBMxM{x]",
            ),
            (
                "http://***REMOVED***/Items/b1cea58b4e2cc4b56b6c2678ab4d422d/Images/Primary",
                "eUFFy;IVozt7IU00-;t7RjoLWBofaya|oft7IUWBofRjM{t7j[WBof",
            ),
            (
                "http://***REMOVED***/Items/3feece1957c917fc833843cb7a2ae41e/Images/Primary",
                "eGE1K@%M%2=|v~0|IUVYI:WBZ$xu-pNaxuIoaK#mw0$PpwX8NGXSNG",
            ),
            (
                "http://***REMOVED***/Items/41f028168e91ec428103c5c3edec40bb/Images/Primary",
                "eBA^8ZtQ14%KIp7#t6=^Sh%1NFoenOWBtl]~f69Hn$tl5Zfjv}WCtP",
            ),
            (
                "http://***REMOVED***/Items/139752d2963a2fe6a47b6d61b23bf212/Images/Primary",
                "eQRKm|?HEe?H9F~WWBWUs:WB1GS1R6RQXm^+WBRPs:Sg4.R*tkW.ix",
            ),
            (
                "http://***REMOVED***/Items/651af94f0522b7a5a5cc21602ece3d21/Images/Primary",
                "eaLMr0LMBAsDKKuiSiTKOXw^?Grqg0J+oN-pjFjGo3w|nnjbS$xtNa",
            ),
            (
                "http://***REMOVED***/Items/3b9ba8b7e7d81e9ca129c11cd3b1f734/Images/Primary",
                "e76th0oh0cf#v%0gaf=_WBTJ$yt8NyRlrq%eR+M|%3R*0Jj[?IRjkX",
            ),
            (
                "http://***REMOVED***/Items/c103379495a0e26012a28a2529397c0c/Images/Primary",
                "eIEVHI|nEMEn4;97Rms.%0-m56TKsprqxZpdM{skbcENS%E2NHo}k9",
            ),
            (
                "http://***REMOVED***/Items/c57a9cc5f1eda7cc01f1e5b71b498aab/Images/Primary",
                "eUB3mLaeNGj[WV00oft7f6of_3R*V@jtay9Ft7t7fkof%MRjRjWBWB",
            ),
        ];

        State {
            images: dbg_tracks
                .into_iter()
                .map(|(url, blurhash)| {
                    Image::new(Url::parse(url).unwrap(), Some(blurhash.to_owned()))
                })
                .collect(),
        }
    }

    fn view(state: &State) -> Element<'_, Message> {
        scrollable(
            grid(
                state
                    .images
                    .iter()
                    .map(|i| i.1.view().map(|m| Message::ImageDriver(*i.0, m))),
            )
            .columns(7),
        )
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn update(state: &mut State, message: Message) -> Task<Message> {
        match message {
            Message::ImageDriver(uuid, message) => {
                if let Some(image) = state.images.get_mut(&uuid) {
                    time_with("image_driver", || {
                        image
                            .update(message)
                            .map(move |m| Message::ImageDriver(uuid, m))
                    })
                } else {
                    Task::none()
                }
            }
        }
    }

    iced::application(new, update, view).run().unwrap()
}
