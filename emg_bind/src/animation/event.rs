/*
 * @Author: Rais
 * @Date: 2021-05-10 18:06:08
 * @LastEditTime: 2021-05-12 13:00:40
 * @LastEditors: Rais
 * @Description:
 */

// use std::{path::PathBuf, rc::Rc};

use std::{
    cell::{Cell, RefCell},
    convert::FloatToInt,
    rc::Rc,
    time::Duration,
};

use crate::{event, subscription::Recipe, Hasher};

use emg_animation::{Msg, Tick};
use iced::futures::{self, StreamExt};
use iced_futures::BoxStream;
use tracing::{debug_span, error, trace, trace_span, warn};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::IdleRequestOptions;

pub struct RafEventRecipe {
    // sender: UnboundedSender<(Event, event::Status)>,
// sender: Sender<(Event, event::Status)>,
// receiver: UnboundedReceiver<(Event, event::Status)>,
// receiver: Receiver<(Event, event::Status)>,
// closure: Box<Closure<dyn Fn()>>,
// _publish: Rc<dyn Fn((Event, event::Status))>,
// boxed_local: BoxStream<(Event, event::Status)>,
}

impl Default for RafEventRecipe {
    fn default() -> Self {
        // let (sender, receiver) = futures::channel::mpsc::channel(10);

        // let (sender, receiver) = futures::channel::mpsc::unbounded();

        // Self::new(sender, receiver)
        Self {}
    }
}

pub type AmClosure = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;

//TODO: 泛型 like download.rs
impl Recipe<Hasher, (crate::event::Event, event::Status)> for RafEventRecipe {
    type Output = (Msg, event::Status);

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
        let _g = trace_span!("on AnimationFrameEventRecipe event stream").entered();
        let (sender, receiver) = futures::channel::mpsc::unbounded();

        trace!("new AnimationFrameEventRecipe");

        let window = Rc::new(web_sys::window().unwrap());
        let window1 = window.clone();

        let am_running: AmClosure = Rc::new(RefCell::new(None));

        // ─────────────────────────────────────────────────────────────────
        // let sender2 = sender.clone();
        let am_running2 = am_running.clone();

        // ─────────────────────────────────────────────────────────────────

        *am_running.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
            let _g = debug_span!("in animation frame").entered();
            // let ff = Tick(Duration::from_micros(unsafe {
            // (timestamp * 1000.).trunc().to_int_unchecked::<u64>()
            // }));

            // ─────────────────────────────────────────────────────────────────

            trace!("{:?}", &timestamp);
            if let Err(error) = sender.unbounded_send((
                // Tick(Duration::from_millis(11)),
                Tick(Duration::from_micros(unsafe {
                    (timestamp * 1000.).trunc().to_int_unchecked::<u64>()
                })),
                event::Status::Ignored,
            )) {
                error!("Error sending event to subscription: {:?}", error);
            }

            // ─────────────────────────────────────────────────────────────────

            window1
                .request_animation_frame(
                    am_running2
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unchecked_ref(),
                )
                .expect("should register `requestAnimationFrame` OK");
        }) as Box<dyn FnMut(f64)>));
        // ─────────────────────────────────────────────────────────────────

        window
            .request_animation_frame(
                am_running
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            )
            .expect("should register `requestAnimationFrame` OK");

        receiver.boxed_local()
    }
}

#[cfg(test)]
mod test {

    use std::hash::Hasher;

    use iced_futures::subscription::Recipe;
    use wasm_bindgen_test::{console_log, wasm_bindgen_test};

    use super::RafEventRecipe;

    #[wasm_bindgen_test]
    // #[test]
    fn test() {
        let cc = RafEventRecipe::default();
        let mut hasher = iced_web::Hasher::default();
        cc.hash(&mut hasher);
        console_log!("Hash is {:x}!", hasher.finish());
        let cc2 = RafEventRecipe::default();
        let mut hasher2 = iced_web::Hasher::default();
        cc2.hash(&mut hasher2);
        console_log!("Hash is {:x}!", hasher2.finish());
    }
}
//TODO test hash
