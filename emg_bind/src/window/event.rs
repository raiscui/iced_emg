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

pub struct WindowEventRecipe {
    // sender: UnboundedSender<(Event, event::Status)>,
// sender: Sender<(Event, event::Status)>,
// receiver: UnboundedReceiver<(Event, event::Status)>,
// receiver: Receiver<(Event, event::Status)>,
// closure: Box<Closure<dyn Fn()>>,
// _publish: Rc<dyn Fn((Event, event::Status))>,
// boxed_local: BoxStream<(Event, event::Status)>,
}

impl Default for WindowEventRecipe {
    fn default() -> Self {
        // let (sender, receiver) = futures::channel::mpsc::channel(10);

        // let (sender, receiver) = futures::channel::mpsc::unbounded();

        // Self::new(sender, receiver)
        Self {}
    }
}

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

            //     // b────────────────────────────────────────────────────────────────────────────────

            //     // if let Err(error) = sender2.unbounded_send(()) {
            //     //     warn!("Error sending event to subscription: {:?}", error);
            //     // }
            //     // ────────────────────────────────────────────────────────────────────────────────

            let _droppable = idle_timeout2.borrow_mut().take();
            // }));
        }) as Box<dyn FnMut()>));
        // ─────────────────────────────────────────────────────────────────

        let mut idle_opt = IdleRequestOptions::new();
        idle_opt.timeout(99);

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

            // let window_am = Rc::new(web_sys::window().unwrap());
            // let am = Closure::wrap(Box::new(move || {
            //     let _g = trace_span!("animation").entered();
            //     trace!("in animation..........");
            // }) as Box<dyn Fn()>);
            // window_am.request_animation_frame(am.as_ref().unchecked_ref());
            // am.forget();
        });
        // ─────────────────────────────────────────────────────────────────

        // ────────────────────────────────────────────────────────────────────────────────

        // let on_resize = Box::new(move || {
        //     trace!("on_resize");

        //     if false {
        //         // if *running.borrow() {
        //         let _gg = trace_span!("window on resize -> is running").entered();
        //         trace!("resize disable");
        //     } else {
        //         let _gg = trace_span!("window on resize -> running false").entered();

        //         *running.borrow_mut() = true;
        //         let running_clone = running.clone();

        //         let sender2 = sender.clone();

        //         // with_animation_frame(move || {
        //         // let f = Closure::wrap(Box::new(move || {
        //         let _ggg = trace_span!("window on resize -> animation_frame").entered();
        //         let window2 = window1.clone();

        //         // • • • • •
        //         if let Err(error) = sender2.unbounded_send((
        //             // if let Err(error) = sender.try_send((
        //             Event::Resized {
        //                 width: 11.0,
        //                 height: 11.0,
        //             },
        //             event::Status::Ignored,
        //         )) {
        //             warn!("Error sending event to subscription: {:?}", error);
        //         }
        //         // • • • • •

        //         // let timeout_send = Closure::wrap(Box::new(move || {
        //         //     // let window = web_sys::window().unwrap();
        //         //     let w: JsValue = (window2).inner_width().unwrap();
        //         //     let h: JsValue = (window2).inner_height().unwrap();
        //         //     trace!("with_animation_frame-> on_resize: w:{:?} , h:{:?}", &w, &h);

        //         //     if let Err(error) = sender2.unbounded_send(
        //         // (
        //         //         // if let Err(error) = sender.try_send((
        //         //         Event::Resized {
        //         //             width: w.as_f64().unwrap(),
        //         //             height: h.as_f64().unwrap(),
        //         //         },
        //         //         event::Status::Ignored,
        //         //     )
        //         //) {
        //         //         warn!("Error sending event to subscription: {:?}", error);
        //         //     }

        //         //     *running_clone.borrow_mut() = false;
        //         // }) as Box<dyn FnMut()>);
        //         // window1
        //         //     .set_timeout_with_callback_and_timeout_and_arguments_0(
        //         //         timeout_send.as_ref().unchecked_ref(),
        //         //         1000,
        //         //     )
        //         //     .expect("..... ");

        //         // timeout_send.forget();
        //         // • • • • •

        //         // if let Err(error) = sender2.unbounded_send((
        //         //     // if let Err(error) = sender.try_send((
        //         //     Event::Resized {
        //         //         width: w.as_f64().unwrap(),
        //         //         height: h.as_f64().unwrap(),
        //         //     },
        //         //     event::Status::Ignored,
        //         // )) {
        //         //     warn!("Error sending event to subscription: {:?}", error);
        //         // }

        //         // *running_clone.borrow_mut() = false;

        //         /*
        //         }) as Box<dyn FnMut()>);
        //         */
        //         // });

        //         // request_animation_frame(&f);
        //     }
        // });
        let event_send_closure = Closure::wrap(on_resize as Box<dyn FnMut()>);

        window.set_onresize(Some(event_send_closure.as_ref().unchecked_ref()));
        event_send_closure.forget();

        receiver
            // .map(move |_x| {
            //     let _g = trace_span!("window on resize -> got receiver ").entered();
            //     trace!("==r :map: window on resize");
            //     // let window = web_sys::window().unwrap();
            //     let w: JsValue = (window1).inner_width().unwrap();
            //     let h: JsValue = (window1).inner_height().unwrap();
            //     (
            //         // if let Err(error) = sender.try_send((
            //         Event::Resized {
            //             width: w.as_f64().unwrap(),
            //             height: h.as_f64().unwrap(),
            //         },
            //         event::Status::Ignored,
            //     )
            // })
            .boxed_local()
    }
}

// fn with_animation_frame<F>(mut f: F)
// where
//     F: 'static + FnMut(),
// {
//     let g = Rc::new(RefCell::new(None));
//     let h = g.clone();

//     let f = Closure::wrap(Box::new(move || {
//         *g.borrow_mut() = None;
//         f();
//     }) as Box<dyn FnMut()>);
//     request_animation_frame(&f);

//     *h.borrow_mut() = Some(f);
// }

// fn request_animation_frame(f: &Closure<dyn FnMut()>) {
//     web_sys::window()
//         .expect_throw("should have a window")
//         .request_animation_frame(f.as_ref().unchecked_ref())
//         .expect_throw("should register `requestAnimationFrame` OK");
// }

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
        let mut hasher = iced_web::Hasher::default();
        cc.hash(&mut hasher);
        console_log!("Hash is {:x}!", hasher.finish());
        let cc2 = WindowEventRecipe::default();
        let mut hasher2 = iced_web::Hasher::default();
        cc2.hash(&mut hasher2);
        console_log!("Hash is {:x}!", hasher2.finish());
    }
}
//TODO test hash
