use std::{
    convert::Infallible,
    fmt::{Display, Formatter},
    str::FromStr,
};

use chrono::{DateTime, FixedOffset};

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub enum ClassType {
    Other(String),
}

impl Display for ClassType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassType::Other(s) => s.fmt(f),
        }
    }
}

impl FromStr for ClassType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ClassType::Other(s.to_string()))
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AnyScenarioEvent {
    pub active: bool,
    pub class_types: Vec<ClassType>,
    pub trigger_time: Option<DateTime<FixedOffset>>,
}

impl AnyScenarioEvent {
    pub fn try_new(
        active: bool,
        class_type: &str,
        trigger_time: &str,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            active,
            class_types: parse_class_types(class_type),
            trigger_time: parse_trigger_time(trigger_time)?,
        })
    }
}

fn parse_class_types(s: &str) -> Vec<ClassType> {
    match s {
        "" => Vec::new(),
        s => s
            .split(",")
            .map(|ct| match ClassType::from_str(ct) {
                Ok(ct) => ct,
            })
            .collect(),
    }
}

fn parse_trigger_time(s: &str) -> Result<Option<DateTime<FixedOffset>>, chrono::ParseError> {
    match s {
        "" => Ok(None),
        s => Some(s.parse()).transpose(),
    }
}

#[cfg(feature = "target")]
mod target {
    use axevent::flex::{Error, Event, KeyValueSet};

    use super::*;
    use crate::Subscribable;
    impl Subscribable for AnyScenarioEvent {
        type Error = anyhow::Error;

        fn topic() -> Result<KeyValueSet, Error> {
            let mut kvs = KeyValueSet::new();
            kvs.add_key_value(
                c"topic0",
                Some(c"tnsaxis"),
                Some(c"CameraApplicationPlatform"),
            )?
            .add_key_value(c"topic1", Some(c"tnsaxis"), Some(c"ObjectAnalytics"))?
            .add_key_value(
                c"topic2",
                Some(c"tnsaxis"),
                Some(c"Device1ScenarioANY"),
            )?;
            Ok(kvs)
        }

        fn try_parse(event: Event) -> Result<Self, Self::Error> {
            let kvs = event.key_value_set();
            let class_types = parse_class_types(
                kvs.get_string(c"classTypes", None)?
                    .as_c_str()
                    .to_string_lossy()
                    .as_ref(),
            );
            let active = kvs.get_boolean(c"active", None)?;
            let trigger_time = parse_trigger_time(
                kvs.get_string(c"triggerTime", None)?
                    .as_c_str()
                    .to_string_lossy()
                    .as_ref(),
            )?;
            Ok(Self {
                active,
                class_types,
                trigger_time,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;
    #[test]
    fn empty_string_parses_to_empty_list() {
        let AnyScenarioEvent { class_types, .. } =
            AnyScenarioEvent::try_new(false, "", "").unwrap();
        assert_eq!(Vec::<ClassType>::new(), class_types);
    }

    #[test]
    fn any_scenario_preserves_utc_offset() {
        let expected = "2020-02-02T12:34:56+10:30";
        let event = AnyScenarioEvent::try_new(false, "", expected).unwrap();
        let text = serde_json::to_string(&event).unwrap();
        let v: Value = serde_json::from_str(&text).unwrap();
        let actual = v.pointer("/trigger_time").unwrap().as_str().unwrap();
        assert_eq!(actual, expected);
    }
}
