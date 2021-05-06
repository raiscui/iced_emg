//! Listen to external events in your application.
use crate::Hasher;
use crate::{
    event::{self, Event},
    window::WindowEventRecipe,
};
use iced_futures::BoxStream;

/// A request to listen to external events.
///
/// Besides performing async actions on demand with [`Command`], most
/// applications also need to listen to external events passively.
///
/// A [`Subscription`] is normally provided to some runtime, like a [`Command`],
/// and it will generate events as long as the user keeps requesting it.
///
/// For instance, you can use a [`Subscription`] to listen to a `WebSocket`
/// connection, keyboard presses, mouse events, time ticks, etc.
///
/// [`Command`]: crate::Command
pub type Subscription<T> = iced_futures::Subscription<Hasher, (Event, event::Status), T>;

/// A stream of runtime events.
///
/// It is the input of a [`Subscription`] in the native runtime.
pub type EventStream = BoxStream<(Event, event::Status)>;

/// A native [`Subscription`] tracker.
pub type Tracker = iced_futures::subscription::Tracker<Hasher, (Event, event::Status)>;

pub use iced_futures::subscription::Recipe;

// mod events;

/// Returns a [`Subscription`] to all the runtime events.
///
/// This subscription will notify your application of any [`Event`] that was
/// not captured by any widget.
#[must_use]
pub fn events() -> Subscription<Event> {
    Subscription::from_recipe(WindowEventRecipe::default())
        .filter_map(|(e, status)| match status {
            event::Status::Ignored => Some(e),
            event::Status::Captured => None,
        })
        .map(Event::Window)
}

#[allow(clippy::module_name_repetitions)]
pub trait SubscriptionFilterMap {
    type A;
    type H;
    type E;
    fn filter_map<B>(
        self,
        f: fn(Self::A) -> Option<B>,
    ) -> iced_futures::subscription::Subscription<Self::H, Self::E, B>
    where
        Self::H: 'static + std::hash::Hasher,
        Self::A: 'static,
        Self::E: 'static,
        B: 'static;
}
impl<H, E, A> SubscriptionFilterMap for iced_futures::subscription::Subscription<H, E, A> {
    type A = A;
    type H = H;
    type E = E;

    fn filter_map<B>(
        self,
        f: fn(Self::A) -> Option<B>,
    ) -> iced_futures::Subscription<Self::H, Self::E, B>
    where
        Self::H: 'static + std::hash::Hasher,
        Self::A: 'static,
        Self::E: 'static,
        B: 'static,
    {
        iced_futures::subscription::Subscription::new(
            self.recipes()
                .drain(..)
                .map(|recipe| {
                    Box::new(FilterMap::new(recipe, f)) as Box<dyn Recipe<H, E, Output = B>>
                })
                .collect(),
        )
    }
}
struct FilterMap<Hasher, Event, A, B> {
    recipe: Box<dyn Recipe<Hasher, Event, Output = A>>,
    mapper: fn(A) -> Option<B>,
}

impl<Hasher, Event, A, B> FilterMap<Hasher, Event, A, B> {
    fn new(recipe: Box<dyn Recipe<Hasher, Event, Output = A>>, mapper: fn(A) -> Option<B>) -> Self {
        Self { recipe, mapper }
    }
}

impl<H, E, A, B> Recipe<H, E> for FilterMap<H, E, A, B>
where
    A: 'static,
    B: 'static,
    H: std::hash::Hasher,
{
    type Output = B;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;
        self.recipe.hash(state);
        self.mapper.hash(state);
    }

    fn stream(self: Box<Self>, input: BoxStream<E>) -> BoxStream<Self::Output> {
        use iced::futures::StreamExt;

        let mapper = self.mapper;

        Box::pin(
            self.recipe
                .stream(input)
                .filter_map(move |a| iced::futures::future::ready((mapper)(a))),
        )
    }
}

/// Returns a [`Subscription`] that filters all the runtime events with the
/// provided function, producing messages accordingly.
///
/// This subscription will call the provided function for every [`Event`]
/// handled by the runtime. If the function:
///
/// - Returns `None`, the [`Event`] will be discarded.
/// - Returns `Some` message, the `Message` will be produced.
pub fn events_with<Message>(
    f: fn((Event, event::Status)) -> Option<Message>,
) -> Subscription<Message>
where
    Message: 'static + Send,
{
    Subscription::from_recipe(WindowEventRecipe::default())
        .map(|(e, s)| (event::Event::Window(e), s))
        .filter_map(f)
}
