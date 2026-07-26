use libmpv2::{EndFileReason, LogLevel};

/// An owned variant of [`libmpv2::events::Event`]
#[derive(Debug, Clone)]
pub enum MpvEvent {
    Shutdown,
    LogMessage {
        prefix: String,
        level: String,
        text: String,
        log_level: LogLevel,
    },
    GetPropertyReply {
        name: String,
        value: MpvValue,
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
        value: MpvValue,
        reply_userdata: u64,
    },
    QueueOverflow,
    Deprecated,
}


/// An owned version of [`libmpv2::events::PropertyData`]
#[derive(Debug, Clone, Default)]
pub enum MpvValue {
    #[default]
    None,
    String(String),
    Bool(bool),
    I64(i64),
    F64(f64),
}

impl From<libmpv2::events::Event<'_>> for MpvEvent {
    /// Convert Event<'a> -> `MpvEvent`
    fn from(event: libmpv2::events::Event<'_>) -> Self {
        match event {
            libmpv2::events::Event::Shutdown => MpvEvent::Shutdown,
            libmpv2::events::Event::LogMessage {
                prefix,
                level,
                text,
                log_level,
            } => MpvEvent::LogMessage {
                prefix: prefix.to_string(),
                level: level.to_string(),
                text: text.to_string(),
                log_level,
            },
            libmpv2::events::Event::GetPropertyReply {
                name,
                result,
                reply_userdata,
            } => MpvEvent::GetPropertyReply {
                name: name.to_string(),
                value: MpvValue::from(result),
                reply_userdata,
            },
            libmpv2::events::Event::SetPropertyReply(id) => MpvEvent::SetPropertyReply(id),
            libmpv2::events::Event::CommandReply(id) => MpvEvent::CommandReply(id),
            libmpv2::events::Event::StartFile => MpvEvent::StartFile,
            libmpv2::events::Event::EndFile(reason) => MpvEvent::EndFile(reason),
            libmpv2::events::Event::FileLoaded => MpvEvent::FileLoaded,
            libmpv2::events::Event::ClientMessage(args) => {
                MpvEvent::ClientMessage(args.into_iter().map(ToString::to_string).collect())
            }
            libmpv2::events::Event::VideoReconfig => MpvEvent::VideoReconfig,
            libmpv2::events::Event::AudioReconfig => MpvEvent::AudioReconfig,
            libmpv2::events::Event::Seek => MpvEvent::Seek,
            libmpv2::events::Event::PlaybackRestart => MpvEvent::PlaybackRestart,
            libmpv2::events::Event::PropertyChange {
                name,
                change,
                reply_userdata,
            } => MpvEvent::PropertyChange {
                name: name.to_string(),
                value: MpvValue::from(change),
                reply_userdata,
            },
            libmpv2::events::Event::QueueOverflow => MpvEvent::QueueOverflow,
            libmpv2::events::Event::Deprecated(_) => MpvEvent::Deprecated,
        }
    }
}

impl From<libmpv2::events::PropertyData<'_>> for MpvValue {
    fn from(data: libmpv2::events::PropertyData<'_>) -> Self {
        match data {
            libmpv2::events::PropertyData::Str(s) | libmpv2::events::PropertyData::OsdStr(s) => {
                MpvValue::String(s.to_string())
            }
            libmpv2::events::PropertyData::Double(f) => MpvValue::F64(f),
            libmpv2::events::PropertyData::Int64(i) => MpvValue::I64(i),
            libmpv2::events::PropertyData::Flag(b) => MpvValue::Bool(b),
        }
    }
}
