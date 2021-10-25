// use std::{path::PathBuf, rc::Rc};

use std::{cell::RefCell, rc::Rc};

use crate::{event, subscription::Recipe, Hasher};

use iced::futures::{self, StreamExt};
use iced_futures::BoxStream;
use tracing::{trace, trace_span, warn};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::IdleRequestOptions;

/// A window-related event.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Event {
    /// A window was resized.
    Resized {
        /// The new width of the window (in units)
        width: f64,

        /// The new height of the window (in units)
        height: f64,
    },
    // /// A window was focused.
    // Focused,

    // /// A window was unfocused.
    // Unfocused,

    // /// A file is being hovered over the window.
    // ///
    // /// When the user hovers multiple files at once, this event will be emitted
    // /// for each file separately.
    // FileHovered(PathBuf),

    // /// A file has beend dropped into the window.
    // ///
    // /// When the user drops multiple files at once, this event will be emitted
    // /// for each file separately.
    // FileDropped(PathBuf),

    // /// A file was hovered, but has exited the window.
    // ///
    // /// There will be a single `FilesHoveredLeft` event triggered even if
    // /// multiple files were hovered.
    // FilesHoveredLeft,
}

#[derive(Default)]
pub struct WindowEventRecipe {}

// impl Default for WindowEventRecipe {
//     fn default() -> Self {
//         Self {}
//     }
// }

//TODO: 泛型 like download.rs
impl Recipe<Hasher, (crate::event::Event, event::Status)> for WindowEventRecipe {
    type Output = (Event, event::Status);

    fn hash(&self, state: &mut Hasher) {
        use std::hash::Hash;

        // struct Marker;
        // std::any::TypeId::of::<Marker>().hash(state);

        std::any::TypeId::of::<Self>().hash(state);

        // self.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _event_stream: BoxStream<(crate::event::Event, event::Status)>,
    ) -> BoxStream<Self::Output> {
        // let (mut sender, receiver) = futures::channel::mpsc::channel(1);
        let _g = trace_span!("on window event stream").entered();
        let (sender, receiver) = futures::channel::mpsc::unbounded();

        trace!("new WindowEventRecipe");

        let window = Rc::new(web_sys::window().unwrap());
        let window1 = window.clone();

        let idle_timeout: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));

        // ─────────────────────────────────────────────────────────────────
        // let sender2 = sender.clone();
        let idle_timeout2 = idle_timeout.clone();
        let window2 = window1.clone();

        // ─────────────────────────────────────────────────────────────────
        let idle_cb = Rc::new(RefCell::new(None));

        *idle_cb.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            // *timeout2.borrow_mut() = Some(Timeout::new(66, move || {
            let _g = trace_span!("window on resize -> idle calling").entered();
            // a ────────────────────────────────────────────────────────────────────────────────

            let w: JsValue = (window2).inner_width().unwrap();
            let h: JsValue = (window2).inner_height().unwrap();
            if let Err(error) = sender.unbounded_send((
                // if let Err(error) = sender.try_send((
                Event::Resized {
                    width: w.as_f64().unwrap(),
                    height: h.as_f64().unwrap(),
                },
                event::Status::Ignored,
            )) {
                warn!("Error sending event to subscription: {:?}", error);
            }

            let _droppable = idle_timeout2.borrow_mut().take();
            // }));
        }) as Box<dyn FnMut()>));
        // ─────────────────────────────────────────────────────────────────

        let idle_opt = IdleRequestOptions::new();
        // idle_opt.timeout(66);

        let on_resize = Box::new(move || {
            let _g = trace_span!("window on resize -> event").entered();
            trace!("==r on window resize event call");

            if idle_timeout.borrow().is_some() {
                trace!("==r idle fn working, should not running again");
                return;
            }

            *idle_timeout.borrow_mut() = Some(
                window1
                    .request_idle_callback_with_options(
                        idle_cb.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
                        &idle_opt,
                    )
                    .expect("idle callback init failed"),
            );
            // ─────────────────────────────────────────────────────────────────
        });

        let event_send_closure = Closure::wrap(on_resize as Box<dyn FnMut()>);

        window.set_onresize(Some(event_send_closure.as_ref().unchecked_ref()));
        event_send_closure.forget();

        receiver.boxed_local()
    }
}

#[cfg(test)]
mod test {

    use std::hash::Hasher;

    use iced_futures::subscription::Recipe;
    use wasm_bindgen_test::{console_log, wasm_bindgen_test};

    use super::WindowEventRecipe;

    #[wasm_bindgen_test]
    // #[test]
    fn test() {
        let cc = WindowEventRecipe::default();
        let mut hasher = crate::Hasher::default();
        cc.hash(&mut hasher);
        console_log!("Hash is {:x}!", hasher.finish());
        let cc2 = WindowEventRecipe::default();
        let mut hasher2 = crate::Hasher::default();
        cc2.hash(&mut hasher2);
        console_log!("Hash is {:x}!", hasher2.finish());
    }
}
//TODO test hash
