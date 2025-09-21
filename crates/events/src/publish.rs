use std::{marker::PhantomData, sync::Arc};

use log::{debug, error};
use tokio::sync::mpsc::error::TrySendError;

use crate::{
    common::{EventBuilder, KeyValueSetBuilder, StructuredEvent},
    Error,
};

/// Provides the information needed to publish an event.
pub trait PublishableStructuredEvent: StructuredEvent {
    fn schema() -> KeyValueSetBuilder;

    fn into_event(self) -> EventBuilder;
}

/// A handle for an active declaration used for sending messages.
pub struct Publisher<T> {
    handler: Arc<axevent::flex::Handler>,
    id: axevent::flex::Declaration,
    _phantom: PhantomData<T>,
}

impl<T> Publisher<T>
where
    T: PublishableStructuredEvent + Send + 'static,
{
    pub(crate) async fn try_new(handler: Arc<axevent::flex::Handler>) -> Result<Self, Error> {
        let kvs = T::topic().union(T::schema());
        let stateless = kvs.is_stateless();
        let kvs = kvs.build()?;
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let mut droppable_handler = Some(Arc::clone(&handler));
        handler.declare(
            &kvs,
            stateless,
            Some(move |id| {
                let Some(handler) = droppable_handler.take() else {
                    error!("Declaration complete called more than once");
                    return;
                };
                match tx.try_send(Self {
                    handler,
                    id,
                    _phantom: PhantomData,
                }) {
                    Ok(()) => debug!("Publisher sent"),
                    Err(TrySendError::Closed(_)) => {
                        debug!("Publisher not sent because channel is closed")
                    }
                    Err(TrySendError::Full(_)) => unreachable!(),
                }
            }),
        )?;
        Ok(rx.recv().await.unwrap())
    }

    pub fn publish(&self, event: T) -> Result<(), Error> {
        let event = event.into_event().build()?.into_inner();
        self.handler.send_event(event, &self.id)
    }
}

impl<T> Drop for Publisher<T> {
    fn drop(&mut self) {
        if let Err(e) = self.handler.undeclare(&self.id) {
            error!("Failed to undeclare: {e}");
        }
    }
}
