use std::time::SystemTime;

use anyhow::Context;
use easy_acap::{
    zoo::device_io_events::VirtualInput, Event, EventBuilder, KeyValuePair, KeyValueSetBuilder,
    PublishableStructuredEvent, Runtime, StructuredEvent, SubscribableStructuredEvent,
};
use log::info;

#[tokio::test]
async fn subscribe_to_virtual_input_0() {
    let mut runtime = Runtime::new();
    let mut subscriber = runtime.try_new_subscriber::<VirtualInput<0>>().unwrap();
    let VirtualInput { timestamp, active } = subscriber.recv().await.unwrap();
    println!("timestamp: {timestamp:?}, active: {active:?}");
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Greeting {
    what: String,
    when: SystemTime,
}

impl StructuredEvent for Greeting {
    fn topic() -> KeyValueSetBuilder {
        KeyValueSetBuilder::default()
            .insert(
                KeyValuePair::new("topic0")
                    .namespace("tnsaxis")
                    .value("CameraApplicationPlatform"),
            )
            .insert(
                KeyValuePair::new("topic1")
                    .namespace("tnsaxis")
                    .value("SendAndReceiveEvents"),
            )
    }
}

impl PublishableStructuredEvent for Greeting {
    fn schema() -> KeyValueSetBuilder {
        KeyValueSetBuilder::default().insert(KeyValuePair::new("Greeting"))
    }

    fn into_event(self) -> EventBuilder {
        let Self { what, when } = self;
        EventBuilder::default()
            .timestamp(when)
            .insert(KeyValuePair::new("Greeting").value(what.as_str()))
    }
}

impl SubscribableStructuredEvent for Greeting {
    fn try_from(event: Event) -> anyhow::Result<Self> {
        Ok(Self {
            what: event
                .try_get_string("Greeting", None)?
                .context("Greeting was null")?,
            when: event.timestamp(),
        })
    }
}

#[tokio::main]
async fn main() {
    let mut runtime = Runtime::new();
    let mut subscriber = runtime.try_new_subscriber::<Greeting>().unwrap();
    let publisher = runtime.try_new_publisher::<Greeting>().await.unwrap();

    for greeting in ["tjena", "tjabba", "hallå"].into_iter().cycle() {
        publisher
            .publish(Greeting {
                what: greeting.to_string(),
                when: SystemTime::now(),
            })
            .unwrap();
        let received = subscriber.recv().await.unwrap();
        info!("Received: {received:?}");
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use easy_acap::Runtime;

    use super::*;

    #[tokio::test]
    async fn sent_event_is_received_promptly() {
        let mut runtime = Runtime::new();
        let mut subscriber = runtime.try_new_subscriber::<Greeting>().unwrap();
        let publisher = runtime.try_new_publisher::<Greeting>().await.unwrap();

        let expected = Greeting {
            what: "Hello, world!".to_string(),
            when: SystemTime::UNIX_EPOCH + Duration::from_secs(86400 + 7200 + 180 + 45),
        };
        publisher.publish(expected.clone()).unwrap();
        let actual = subscriber.recv().await.unwrap();
        assert_eq!(expected, actual);
    }
}
