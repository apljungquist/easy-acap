//! Idiomatic and easy-to-use API for building ACAP applications.
//!
//! > Make common things easy, rare things possible.
//! > But not if they are silly.
//!
//! The entry point to this API is the [`Runtime`] type.
//!
//! # Examples
//!
//! ## Subscribe to virtual input events on port 0
//!
//! ```no_run
//! # use easy_acap::{Runtime, zoo::device_io_events::VirtualInput};
//! # async fn subscribe_to_virtual_input_0() {
//! let mut runtime = Runtime::new();
//! let mut subscriber = runtime.try_new_subscriber::<VirtualInput<0>>().unwrap();
//! let VirtualInput { timestamp, active } = subscriber.recv().await.unwrap();
//! println!("timestamp: {timestamp:?}, active: {active:?}");
//! # }
//! ```
//!
//! ## Publish a custom event
//!
//! ```no_run
//! # async fn send_and_receive_custom_event() {
//! # use std::time::{Duration, SystemTime};
//! #
//! # use anyhow::Context;
//! # use easy_acap::{
//! #     Event, EventBuilder, KeyValuePair, KeyValueSetBuilder, PublishableStructuredEvent, Runtime,
//! #     StructuredEvent, SubscribableStructuredEvent,
//! # };
//! #
//! # #[derive(Clone, Debug, Eq, PartialEq)]
//! struct Greeting {
//!     what: String,
//!     when: SystemTime,
//! }
//!
//! impl StructuredEvent for Greeting {
//!     fn topic() -> KeyValueSetBuilder {
//!         KeyValueSetBuilder::default()
//!             .insert(
//!                 KeyValuePair::new("topic0")
//!                     .namespace("tnsaxis")
//!                     .value("CameraApplicationPlatform"),
//!             )
//!             .insert(
//!                 KeyValuePair::new("topic1")
//!                     .namespace("tnsaxis")
//!                     .value("SendAndReceiveEvents"),
//!             )
//!     }
//! }
//!
//! impl PublishableStructuredEvent for Greeting {
//!     fn schema() -> KeyValueSetBuilder {
//!         KeyValueSetBuilder::default().insert(KeyValuePair::new("Greeting"))
//!     }
//!
//!     fn into_event(self) -> EventBuilder {
//!         let Self { what, when } = self;
//!         EventBuilder::default()
//!             .timestamp(when)
//!             .insert(KeyValuePair::new("Greeting").value(what.as_str()))
//!     }
//! }
//!
//! # impl SubscribableStructuredEvent for Greeting {
//! #     fn try_from(event: Event) -> anyhow::Result<Self> {
//! #         Ok(Self {
//! #             what: event
//! #                 .try_get_string("Greeting", None)?
//! #                 .context("Greeting was null")?,
//! #             when: event.timestamp(),
//! #         })
//! #     }
//! # }
//!
//! let mut runtime = Runtime::new();
//! let mut subscriber = runtime.try_new_subscriber::<Greeting>().unwrap();
//! let publisher = runtime.try_new_publisher::<Greeting>().await.unwrap();
//!
//! let expected = Greeting {
//!     what: "Hello, world!".to_string(),
//!     when: SystemTime::UNIX_EPOCH + Duration::from_secs(86400 + 7200 + 180 + 45),
//! };
//! publisher.publish(expected.clone()).unwrap();
//! # let actual = subscriber.recv().await.unwrap();
//! # assert_eq!(expected, actual);
//! # }
//! ```

use std::{
    sync::{Arc, Weak},
    thread,
    thread::JoinHandle,
};

pub use axevent::flex::Error;
pub use common::{Event, EventBuilder, KeyValuePair, KeyValueSetBuilder, StructuredEvent};
use log::debug;
pub use publish::{PublishableStructuredEvent, Publisher};
pub use subscribe::{SubscribableStructuredEvent, Subscriber};

mod common;
mod publish;
mod subscribe;
mod utils;
pub mod zoo;

/// The entry point for most APIs.
pub struct Runtime {
    main_loop: Arc<glib::MainLoop>,
    join_handle: JoinHandle<()>,
    event_handler: Weak<axevent::flex::Handler>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        let main_loop = Arc::new(glib::MainLoop::new(None, false));
        let join_handle = thread::spawn({
            let main_loop = Arc::clone(&main_loop);
            move || {
                debug!("Starting main loop");
                main_loop.run();
                debug!("Main loop stopped");
            }
        });
        Self {
            main_loop,
            join_handle,
            event_handler: Weak::new(),
        }
    }

    fn event_handler(&mut self) -> Arc<axevent::flex::Handler> {
        match self.event_handler.upgrade() {
            None => {
                let handler = Arc::new(axevent::flex::Handler::new());
                self.event_handler = Arc::downgrade(&handler);
                handler
            }
            Some(handler) => handler,
        }
    }

    pub async fn try_new_publisher<T>(&mut self) -> Result<Publisher<T>, Error>
    where
        T: PublishableStructuredEvent + Send + 'static,
    {
        let handler = self.event_handler();
        Publisher::try_new(handler).await
    }

    pub fn try_new_subscriber<T>(&mut self) -> Result<Subscriber<T>, Error>
    where
        T: SubscribableStructuredEvent + Send + 'static,
    {
        let handler = self.event_handler();
        Subscriber::try_new(handler)
    }

    pub fn quit_and_join(self) -> thread::Result<()> {
        debug!("Quitting main loop...");
        self.main_loop.quit();
        debug!("Waiting for main loop...");
        self.join_handle.join()
    }
}
