#![feature(fn_traits)]
#![feature(iter_intersperse)]
//for trait Program
#![feature(specialization)]
#![feature(let_chains)]
// #![feature(trivial_bounds)] // for emg_state::StateAnchor<emg_common::nalgebra::Translation<f64, 3>>: emg_shaping::ShapingWhoNoWarper;
// ────────────────────────────────────────────────────────────────────────────────
pub mod bus;
pub mod clipboard;
pub mod command;
pub mod drag;
pub mod event;
pub mod features;
pub mod future_runtime;
pub mod keyboard;
pub mod mouse;
pub mod paint_ctx;
pub mod program;
pub mod renderer;
pub mod system;
pub mod touch;
pub mod widget;
pub mod window;
// ────────────────────────────────────────────────────────────────────────────────
#[cfg(feature = "debug")]
#[path = "debug/basic.rs"]
mod debug;
#[cfg(not(feature = "debug"))]
#[path = "debug/null.rs"]
mod debug;

// ────────────────────────────────────────────────────────────────────────────────
pub use clipboard::Clipboard;
pub use command::Command;
pub use debug::Debug;
pub use emg_common::time;
pub use emg_common::Affine;
pub use emg_common::Pos;
pub use emg_futures::{executor, futures};
pub use event::Event;
pub use event::EventWithFlagType;
pub use executor::Executor;
pub use future_runtime::FutureRuntime;
// pub use hasher::Hasher;
// pub use layout::Layout;
// pub use overlay::Overlay;
pub use paint_ctx::{PaintCtx, WidgetState};
pub use program::Program;
// pub use renderer::*;
// pub use shell::Shell;
// pub use theme::Theme;
// pub use user_interface::UserInterface;
pub use bus::Bus;
pub use widget::Widget;

// ────────────────────────────────────────────────────────────────────────────────
use emg_state::use_state;
use event::{EventIdentify, MultiLevelIdentify, MultiLevelIdentifyWithSwitch};
use static_init::dynamic;

#[dynamic]
pub static G_POS: emg_state::StateVar<Option<Pos<f64>>> = use_state(|| None);

#[dynamic]
pub static EVENT_HOVER_CHECK: MultiLevelIdentify = {
    let m_click: EventIdentify = mouse::GENERAL_CLICK.into();
    let m_cursor: EventIdentify = mouse::CURSOR.into();
    let m_ws: EventIdentify = mouse::WHEEL_SCROLLED.into();
    let touch: EventIdentify = touch::EventFlag::empty().into(); //all
    let dnd: EventIdentify = drag::EventFlag::empty().into(); //all
    m_click | m_cursor | m_ws | touch | dnd
};

#[dynamic]
pub static EVENT_DEBOUNCE: MultiLevelIdentify = {
    let mouse_e: EventIdentify = mouse::CURSOR_MOVED.into();
    let drag_s: EventIdentify = drag::EventFlag::DRAG_START.into();
    let drag: EventIdentify = drag::EventFlag::DRAG.into();
    // let drag_e: EventIdentify = drag::EventFlag::empty().into();

    mouse_e | drag_s | drag
};

//collision down ,if collision,choice right
//如果满足 if_cb_contains , 并且event 含有 left,right 两者,即冲突, 选择 left,remove right
#[dynamic]
pub static COLLISION_DOWN: Vec<(MultiLevelIdentify, MultiLevelIdentify, MultiLevelIdentify)> = {
    let if_cb_contains: EventIdentify = drag::EventFlag::DRAG.into();
    let drag_e: EventIdentify = drag::EventFlag::DRAG_END.into(); //use
    let mouse_e: EventIdentify = mouse::RELEASED.into(); //remove
                                                         // let drag_e: EventIdentify = drag::EventFlag::empty().into();

    vec![(
        MultiLevelIdentify::new(if_cb_contains),
        MultiLevelIdentify::new(drag_e),  //true choose
        MultiLevelIdentify::new(mouse_e), //false choose
    )]
};

#[dynamic]
pub static mut EVENT_LONG_STATE_INIT: MultiLevelIdentifyWithSwitch = {
    let draging: EventIdentify = drag::EventFlag::DRAG.into();

    MultiLevelIdentifyWithSwitch::new(draging, false)
};

// thread_local! {
//     static GLOBAL_PENETRATE_EVENTS:  HashMap<
//     EventIdentify,
//     bool,
//     BuildHasherDefault<CustomHasher>,
// > = HashMap::default();
// }

#[dynamic]
pub static mut GLOBAL_PENETRATE_EVENTS: MultiLevelIdentifyWithSwitch = {
    let d: EventIdentify = drag::DRAG.into();
    let de: EventIdentify = drag::DRAG_END.into();
    let cu: EventIdentify = mouse::CURSOR.into();
    let gc: EventIdentify = mouse::GENERAL_CLICK.into();
    // MultiLevelIdentifyWithSwitch::new(d, false) | de | cu | gc
    MultiLevelIdentifyWithSwitch::new(d, false) | de | cu | (gc, false)
};
// ────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::{drag, event::EventIdentify, mouse, EVENT_DEBOUNCE, EVENT_HOVER_CHECK};

    #[test]
    fn dyn_static() {
        let cm = mouse::CURSOR_MOVED.into();
        let drag_start: EventIdentify = drag::EventFlag::DRAG_START.into();

        assert!(EVENT_DEBOUNCE.involve(&cm));

        // println!("{:?}   {:?}", EVENT_DEBOUNCE, drag_start);
        assert!(EVENT_DEBOUNCE.involve(&drag_start));

        assert!(EVENT_HOVER_CHECK.involve(&mouse::CLICK.into()));
        // assert!(EVENT_HOVER_CHECK.involve(&mouse::CLICK.into()));
    }
}
