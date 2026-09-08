//! Events from topics starting with `AudioClassification/`.
use std::{
    convert::Infallible,
    fmt::{Display, Formatter},
    str::FromStr,
};

use anyhow::Context;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Topic1 {
    GlassBreak,
    Screaming,
    Shout,
    /// New in 12.5
    Speech,
    Other(String),
}

impl Display for Topic1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GlassBreak => "GlassBreak".fmt(f),
            Self::Screaming => "Screaming".fmt(f),
            Self::Shout => "Shout".fmt(f),
            Self::Speech => "Speech".fmt(f),
            Self::Other(s) => s.fmt(f),
        }
    }
}

impl FromStr for Topic1 {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GlassBreak" => Ok(Self::GlassBreak),
            "Screaming" => Ok(Self::Screaming),
            "Shout" => Ok(Self::Shout),
            "Speech" => Ok(Self::Speech),
            s => Ok(Self::Other(s.to_string())),
        }
    }
}

#[derive(Debug)]
pub struct AudioSource {
    pub device: u32,
    pub input: u32,
}

impl FromStr for AudioSource {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (device, input) = s
            .strip_prefix("AudioDevice")
            .context("Expected value to start with AudioDevice")?
            .split_once("Input")
            .context("Expected value to contain Input")?;
        let device = device
            .parse::<u32>()
            .context("Expected device to be a number")?;
        let input = input
            .parse::<u32>()
            .context("Expected input to be a number")?;
        Ok(Self { device, input })
    }
}

/// This is a stateful event.
/// [`AudioClassificationEvent::detected`] will be true while it is ongoing.
#[derive(Debug)]
pub struct AudioClassificationEvent {
    pub topic1: Topic1,
    pub detected: bool,
    pub audio_source: AudioSource,
}

#[cfg(feature = "target")]
mod target {
    use ::axevent::flex::{Event, KeyValueSet};

    pub use super::*;
    use crate::Subscribable;

    impl Subscribable for AudioClassificationEvent {
        type Error = anyhow::Error;

        fn topic() -> Result<KeyValueSet, ::axevent::flex::Error> {
            let mut kvs = KeyValueSet::new();
            kvs.add_key_value(c"topic0", Some(c"tnsaxis"), Some(c"AudioClassification"))?;

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
