//! Events from topics starting with `Device/IO/`.
use std::time::SystemTime;

/// Structured representation of a virtual input event
pub struct VirtualInput<const N: u8> {
    pub timestamp: SystemTime,
    pub active: bool,
}

#[cfg(feature = "target")]
mod target {
    use axevent::flex::{Event, KeyValueSet};

    use super::*;
    use crate::Subscribable;

    impl<const N: u8> Subscribable for VirtualInput<N> {
        type Error = anyhow::Error;

        fn topic() -> Result<KeyValueSet, axevent::flex::Error> {
            let mut kvs = KeyValueSet::new();
            kvs.add_key_value(c"topic0", Some(c"tns1"), Some(c"Device"))?
                .add_key_value(c"topic1", Some(c"tnsaxis"), Some(c"IO"))?
                .add_key_value(c"topic2", Some(c"tnsaxis"), Some(c"VirtualInput"))?
                .add_key_value(c"port", None, Some(i32::from(N)))?;
            Ok(kvs)
        }

        fn try_parse(event: Event) -> anyhow::Result<Self> {
            let timestamp = event.time_stamp2();
            let timestamp = axevent::ergo::system_time(timestamp);
            let kvs = event.key_value_set();
            let active = kvs.get_boolean(c"active", None)?;
            Ok(Self { timestamp, active })
        }
    }
}
