use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::subscription::{EventStream, Recipe};
use crate::Hasher;
use crate::{
    event::{self, Event},
    window,
};
use iced::{futures::FutureExt, Executor};
// use iced::futures::stream::futures_unordered;
use iced_futures::futures::future;
use iced_futures::futures::StreamExt;
use iced_futures::BoxStream;
use tracing::{trace, trace_span};

pub struct Events<Message> {
    pub(super) f: fn(Event, event::Status) -> Option<Message>,
}

impl<Message> Recipe<Hasher, (Event, event::Status)> for Events<Message>
where
    Message: 'static + Send,
{
    type Output = Message;

    fn hash(&self, state: &mut Hasher) {
        use std::hash::Hash;

        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);
        self.f.hash(state);
    }

    fn stream(self: Box<Self>, event_stream: EventStream) -> BoxStream<Self::Output> {
        event_stream
            .filter_map(move |(event, status)| future::ready((self.f)(event, status)))
            .boxed_local()
    }
}
