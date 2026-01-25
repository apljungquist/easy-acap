use std::{convert::Infallible, str::FromStr};

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ClassType {
    Human,
    Other(String),
}

impl ClassType {
    pub fn into_string(self) -> String {
        match self {
            Self::Human => "Human".to_string(),
            Self::Other(s) => s,
        }
    }
}

impl FromStr for ClassType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Human" => ClassType::Human,
            s => ClassType::Other(s.to_string()),
        })
    }
}

pub struct MovementInAreaEvent {
    pub active: bool,
}

pub struct LineCrossing {
    pub channel_id: i32,
    pub track_id: i32,
    pub azimuth: f64,
    pub speed: f64,
    pub bearing: f64,
    pub range: f64,
    pub classification: ClassType,
    pub profile_id: i32,
}

#[cfg(feature = "target")]
mod target {
    use axevent::flex::{Error, Event, KeyValueSet};

    use super::*;
    use crate::Subscribable;

    impl Subscribable for MovementInAreaEvent {
        type Error = anyhow::Error;

        fn topic() -> Result<KeyValueSet, Error> {
            let mut kvs = KeyValueSet::new();
            kvs.add_key_value(c"topic0", Some(c"tnsaxis"), Some(c"RadarSource"))?
                .add_key_value(c"topic1", Some(c"tnsaxis"), Some(c"MotionAlarm"))?
                .add_key_value(c"topic2", Some(c"tnsaxis"), Some(c"Channel1ProfileANY"))?;
            Ok(kvs)
        }

        fn try_parse(event: Event) -> Result<Self, Self::Error> {
            let kvs = event.key_value_set();
            let active = kvs.get_boolean(c"active", None)?;
            Ok(Self { active })
        }
    }
    impl Subscribable for LineCrossing {
        type Error = anyhow::Error;

        fn topic() -> Result<KeyValueSet, Error> {
            let mut kvs = KeyValueSet::new();
            kvs.add_key_value(c"topic0", Some(c"tnsaxis"), Some(c"RadarSource"))?
                .add_key_value(c"topic1", Some(c"tnsaxis"), Some(c"MotionAlarm"))?
                .add_key_value(c"topic3", Some(c"tnsaxis"), Some(c"SingleCrossLineCrossed"))?;
            Ok(kvs)
        }

        fn try_parse(event: Event) -> Result<Self, Self::Error> {
            let kvs = event.key_value_set();
            let channel_id = kvs.get_integer(c"channel_id", None)?.try_into()?;
            let track_id = kvs.get_integer(c"track_id", None)?.try_into()?;
            let azimuth = kvs.get_integer(c"azimuth", None)?.try_into()?;
            let speed = kvs.get_integer(c"speed", None)?.try_into()?;
            let bearing = kvs.get_integer(c"bearing", None)?.try_into()?;
            let range = kvs.get_integer(c"range", None)?.try_into()?;
            let classification = kvs
                .get_string(c"classification", None)?
                .as_c_str()
                .to_string_lossy()
                .parse()?;
            let profile_id = kvs.get_integer(c"profile_id", None)?.try_into()?;
            Ok(Self {
                channel_id,
                track_id,
                azimuth,
                speed,
                bearing,
                range,
                classification,
                profile_id,
            })
        }
    }
}
