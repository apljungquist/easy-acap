use std::time::SystemTime;

use anyhow::Context;

use crate::{
    common::{Event, KeyValuePair, KeyValueSetBuilder, StructuredEvent},
    subscribe::SubscribableStructuredEvent,
};

/// Structured representation of a virtual input event
pub struct VirtualInput<const N: u8> {
    pub timestamp: SystemTime,
    pub active: bool,
}

impl<const N: u8> StructuredEvent for VirtualInput<N> {
    fn topic() -> KeyValueSetBuilder {
        KeyValueSetBuilder::default()
            .insert(
                KeyValuePair::new("topic0")
                    .namespace("tns1")
                    .value("Device"),
            )
            .insert(KeyValuePair::new("topic1").namespace("tnsaxis").value("IO"))
            .insert(
                KeyValuePair::new("topic2")
                    .namespace("tnsaxis")
                    .value("VirtualInput"),
            )
            .insert(KeyValuePair::new("port").value(N))
    }
}

impl<const N: u8> SubscribableStructuredEvent for VirtualInput<N> {
    fn try_from(event: Event) -> anyhow::Result<Self> {
        let timestamp = event.timestamp();
        let active = event
            .try_get_boolean("active", None)?
            .context("active was none")?;
        Ok(Self { timestamp, active })
    }
}
