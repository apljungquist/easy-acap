_Idiomatic and easy-to-use API for building ACAP applications._

> Make common things easy, rare things possible.
> But not if they are silly.

The entry point to this API is the [`Runtime`] type.

# Examples

## Subscribe to virtual input events on port 0

```no_run
# use easy_acap::{Runtime, zoo::device_io_events::VirtualInput};
# async fn subscribe_to_virtual_input_0() {
let mut runtime = Runtime::new();
let mut subscriber = runtime.try_new_subscriber::<VirtualInput<0>>().unwrap();
let VirtualInput { timestamp, active } = subscriber.recv().await.unwrap();
println!("timestamp: {timestamp:?}, active: {active:?}");
# }
```

## Publish a custom event

```no_run
# async fn send_and_receive_custom_event() {
# use std::time::{Duration, SystemTime};
#
# use anyhow::Context;
# use easy_acap::{
#     Event, EventBuilder, KeyValuePair, KeyValueSetBuilder, PublishableStructuredEvent, Runtime,
#     StructuredEvent, SubscribableStructuredEvent,
# };
#
# #[derive(Clone, Debug, Eq, PartialEq)]
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

# impl SubscribableStructuredEvent for Greeting {
#     fn try_from(event: Event) -> anyhow::Result<Self> {
#         Ok(Self {
#             what: event
#                 .try_get_string("Greeting", None)?
#                 .context("Greeting was null")?,
#             when: event.timestamp(),
#         })
#     }
# }

let mut runtime = Runtime::new();
let mut subscriber = runtime.try_new_subscriber::<Greeting>().unwrap();
let publisher = runtime.try_new_publisher::<Greeting>().await.unwrap();

let expected = Greeting {
    what: "Hello, world!".to_string(),
    when: SystemTime::UNIX_EPOCH + Duration::from_secs(86400 + 7200 + 180 + 45),
};
publisher.publish(expected.clone()).unwrap();
# let actual = subscriber.recv().await.unwrap();
# assert_eq!(expected, actual);
# }
```
