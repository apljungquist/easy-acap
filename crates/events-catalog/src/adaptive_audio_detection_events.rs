use std::{
    convert::Infallible,
    fmt::{Display, Formatter},
    str::FromStr,
};

use crate::audio_classification_events::AudioSource;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Topic1 {
    Detected,
    Other(String),
}

impl Display for Topic1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Detected => "Detected".fmt(f),
            Self::Other(s) => s.fmt(f),
        }
    }
}

impl FromStr for Topic1 {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Detected" => Ok(Self::Detected),
            s => Ok(Self::Other(s.to_string())),
        }
    }
}

#[derive(Debug)]
pub struct AdaptiveAudioDetectionEvent {
    pub topic1: Topic1,
    pub detected: bool,
    pub audio_source: AudioSource,
}

#[cfg(feature = "target")]
mod target {
    use ::axevent::flex::{Event, KeyValueSet};

    pub use super::*;
    use crate::Subscribable;

    impl Subscribable for AdaptiveAudioDetectionEvent {
        type Error = anyhow::Error;

        fn topic() -> Result<KeyValueSet, ::axevent::flex::Error> {
            let mut kvs = KeyValueSet::new();
            kvs.add_key_value(c"topic0", Some(c"tnsaxis"), Some(c"AdaptiveAudioDetection"))?;

            Ok(kvs)
        }

        fn try_parse(event: Event) -> Result<Self, Self::Error> {
            let kvs = event.key_value_set();
            let topic1 = kvs
                .get_string(c"topic1", Some(c"tnsaxis"))?
                .as_c_str()
                .to_string_lossy()
                .as_ref()
                .parse()?;
            let audio_source = kvs
                .get_string(c"AudioSource", None)?
                .as_c_str()
                .to_string_lossy()
                .as_ref()
                .parse()?;
            let detected = kvs.get_boolean(c"Detected", None)?;
            Ok(Self {
                topic1,
                detected,
                audio_source,
            })
        }
    }
}
