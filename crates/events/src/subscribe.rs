use std::{mem, sync::Arc};

use log::{error, warn};
use tokio::sync::mpsc;

use crate::{
    common::{Event, StructuredEvent},
    Error,
};

/// Provides the information needed to subscribe to an event.
pub trait SubscribableStructuredEvent: StructuredEvent + Sized {
    fn try_from(event: Event) -> anyhow::Result<Self>;
}

/// A handle for an active subscription used to receive messages.
pub struct Subscriber<T> {
    handler: Arc<axevent::flex::Handler>,
    id: axevent::flex::Subscription,
    rx: mpsc::Receiver<T>,
}

impl<T> Subscriber<T>
where
    T: SubscribableStructuredEvent + Send + 'static,
{
    pub(crate) fn try_new(handler: Arc<axevent::flex::Handler>) -> Result<Self, Error>
    where
        T: SubscribableStructuredEvent,
    {
        let topic = T::topic();
        let (tx, rx) = mpsc::channel(10);
        let mut maybe_tx = Some(tx);
        let mut callback = move |event| {
            let Some(tx) = &maybe_tx else {
                return;
            };
            let event = match T::try_from(event) {
                Ok(event) => event,
                Err(e) => {
                    warn!("Failed to convert event: {e:?}");
                    return;
                }
            };
            if tx.try_send(event).is_err() {
                drop(mem::take(&mut maybe_tx));
            }
        };
        let id = handler.subscribe(topic.build()?, move |_, evt| {
            callback(Event::from_event(evt));
        })?;
        Ok(Self { handler, id, rx })
    }

    pub async fn recv(&mut self) -> Option<T> {
        self.rx.recv().await
    }
}

impl<T> Drop for Subscriber<T> {
    fn drop(&mut self) {
        if let Err(e) = self.handler.unsubscribe(&self.id) {
            error!("Failed to unsubscribe: {e}");
        }
    }
}
