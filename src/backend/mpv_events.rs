use futures::Stream;
use futures::{StreamExt, task::AtomicWaker};
use libmpv2::{EndFileReason, Format, GetData, LogLevel, Mpv};
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

#[derive(Debug, Clone)]
pub enum OwnedMpvEvent {
    Shutdown,
    LogMessage {
        prefix: String,
        level: String,
        text: String,
        log_level: LogLevel,
    },
    GetPropertyReply {
        
        name: String,
        value: MpvPropertyValue,
        reply_userdata: u64,
    },
    SetPropertyReply(u64),
    CommandReply(u64),
    StartFile,
    EndFile(EndFileReason),
    FileLoaded,
    ClientMessage(Vec<String>),
    VideoReconfig,
    AudioReconfig,
    Seek,
    PlaybackRestart,
    PropertyChange {
        name: String,
        value: MpvPropertyValue,
        reply_userdata: u64,
    },
    QueueOverflow,
    // Deprecated events are rare, so we can skip or handle minimally
    Deprecated,
}

#[derive(Debug, Clone)]
pub enum MpvPropertyValue {
    None,
    String(String),
    Boolean(bool),
    Flag(bool),
    I64(i64),
    F64(f64),
}

impl From<libmpv2::events::Event<'_>> for OwnedMpvEvent {
    /// Convert Event<'a> -> OwnedMpvEvent
    fn from(event: libmpv2::events::Event<'_>) -> Self {
        match event {
            libmpv2::events::Event::Shutdown => OwnedMpvEvent::Shutdown,
            libmpv2::events::Event::LogMessage {
                prefix,
                level,
                text,
                log_level,
            } => OwnedMpvEvent::LogMessage {
                prefix: prefix.to_string(),
                level: level.to_string(),
                text: text.to_string(),
                log_level,
            },
            libmpv2::events::Event::GetPropertyReply {
                name,
                result,
                reply_userdata,
            } => OwnedMpvEvent::GetPropertyReply {
                name: name.to_string(),
                value: MpvPropertyValue::from(result),
                reply_userdata,
            },
            libmpv2::events::Event::SetPropertyReply(id) => OwnedMpvEvent::SetPropertyReply(id),
            libmpv2::events::Event::CommandReply(id) => OwnedMpvEvent::CommandReply(id),
            libmpv2::events::Event::StartFile => OwnedMpvEvent::StartFile,
            libmpv2::events::Event::EndFile(reason) => OwnedMpvEvent::EndFile(reason),
            libmpv2::events::Event::FileLoaded => OwnedMpvEvent::FileLoaded,
            libmpv2::events::Event::ClientMessage(args) => {
                OwnedMpvEvent::ClientMessage(args.into_iter().map(|s| s.to_string()).collect())
            }
            libmpv2::events::Event::VideoReconfig => OwnedMpvEvent::VideoReconfig,
            libmpv2::events::Event::AudioReconfig => OwnedMpvEvent::AudioReconfig,
            libmpv2::events::Event::Seek => OwnedMpvEvent::Seek,
            libmpv2::events::Event::PlaybackRestart => OwnedMpvEvent::PlaybackRestart,
            libmpv2::events::Event::PropertyChange {
                name,
                change,
                reply_userdata,
            } => OwnedMpvEvent::PropertyChange {
                name: name.to_string(),
                value: MpvPropertyValue::from(change),
                reply_userdata,
            },
            libmpv2::events::Event::QueueOverflow => OwnedMpvEvent::QueueOverflow,
            libmpv2::events::Event::Deprecated(_) => OwnedMpvEvent::Deprecated,
        }
    }
}

impl From<libmpv2::events::PropertyData<'_>> for MpvPropertyValue {
    fn from(data: libmpv2::events::PropertyData<'_>) -> Self {
        match data {
            libmpv2::events::PropertyData::Str(s) => MpvPropertyValue::String(s.to_string()),
            libmpv2::events::PropertyData::OsdStr(s) => MpvPropertyValue::String(s.to_string()),
            libmpv2::events::PropertyData::Double(f) => MpvPropertyValue::F64(f),
            libmpv2::events::PropertyData::Int64(i) => MpvPropertyValue::I64(i),
            libmpv2::events::PropertyData::Flag(b) => MpvPropertyValue::Flag(b),
        }
    }
}

pub struct MpvEventStream {
    mpv: Mpv,
    waker: Arc<AtomicWaker>,
}

impl MpvEventStream {
    pub fn new(mut mpv: Mpv) -> Self {
        let waker = Arc::new(AtomicWaker::new());
        let waker_clone = waker.clone();

        mpv.set_wakeup_callback(move || {
            println!("Callback");
            waker_clone.wake();
        });

        Self { mpv, waker }
    }
}

impl Stream for MpvEventStream {
    type Item = Result<OwnedMpvEvent, libmpv2::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        println!("Poll called!");
        self.waker.register(cx.waker());

        let event = self.mpv.wait_event(f64::MIN_POSITIVE);
        dbg!(&event);

        if let Some(ev) = event {
            println!("Ready!");
            return Poll::Ready(Some(ev.map(OwnedMpvEvent::from)));
        }

        println!("Pending!");
        Poll::Pending
    }
}

/// TODO: This test DOES NOT WORK
#[tokio::test]
async fn test_event_stream() {
    // Create MPV
    let mpv = Mpv::with_initializer(|init| {
        init.set_option("volume", "0")?;
        init.set_option("vid", "no")?;
        Ok(())
    })
    .unwrap();

    // Load some file
    mpv.command("loadfile", &["/mnt/NAS/Samuel/Music/flac/Aaron Cherof/Minecraft_ Trails & Tales_ Original Game Soundtrack/04 - Crescent Dunes.flac"])
        .unwrap();

    // Observe time-pos (playback progress)
    mpv.observe_property("time-pos", Format::Double, 0).unwrap();

    let mut stream = MpvEventStream::new(mpv);

    // Continually check stream
    for i in 0..1000 {
        println!("Awaiting {i}");
        let next = stream.next().await;
        dbg!(next);
    }
}
