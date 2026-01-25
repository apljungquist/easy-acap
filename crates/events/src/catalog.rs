//! A collection of common events ready to be subscribed to.

pub mod adaptive_audio_detection_events;
pub mod audio_classification_events;
pub mod device_io_events;
pub mod object_analytics_events;
pub mod radar_source_events;

#[cfg(feature = "target")]
pub trait Subscribable: Sized {
    type Error;
    fn topic() -> Result<axevent::flex::KeyValueSet, axevent::flex::Error>;
    fn try_parse(event: axevent::flex::Event) -> Result<Self, Self::Error>;
}
