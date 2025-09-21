#![doc=include_str!("../README.md")]

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
