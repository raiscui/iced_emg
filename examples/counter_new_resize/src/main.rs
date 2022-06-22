// #![feature(generic_associated_types)]
use anchors::dict_k_into;
use emg_animation::{interrupt, opacity, style, to};

use emg_bind::{
    better_any::{impl_tid, tid, type_id, Tid, TidAble, TidExt},
    button, edge_index_no_source, emg_msg,
    event::Event,
    g_node::node_item_rc_sv::{GelType, GraphType},
    Application, Button, Checkbox, Command, Element, GElement, GTreeBuilderElement, GraphMethods,
    GraphView, Orders, Subscription, Text,
};

use emg_core::{into_vector, parent, vector, IdStr, TypeCheckObjectSafe};
use emg_layout::{
    add_values::origin_x,
    anima,
    animation::AnimationE,
    global_clock,
    styles::{pc, px, width, CssWidth},
    EPath,
};
use emg_refresh::RefreshUse;

use emg_state::{topo, CloneStateVar, Dict};
use emg_state::{CloneStateAnchor, StateAnchor};
use std::convert::TryFrom;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

use emg_state::{dict, use_state, StateVar};

use iced::{Align, Column, Error, Settings};
extern crate gtree;

use gtree::gtree;

use seed_styles::w;
use tracing::{debug, debug_span, trace, warn};
use tracing::{info, trace_span};
fn setup_tracing() {
    // #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    #[cfg(debug_assertions)]
    {
        let mut config = tracing_wasm::WASMLayerConfigBuilder::default();
        config.set_max_level(tracing::Level::DEBUG);
        config.set_console_config(tracing_wasm::ConsoleConfig::ReportWithConsoleColor);
        // config.set_console_config(tracing_wasm::ConsoleConfig::NoReporting);

        tracing_wasm::set_as_global_default_with_config(config.build());
    }
    // #[cfg(not(debug_assertions))]
    // {
    //     let mut config = tracing_wasm::WASMLayerConfigBuilder::default();
    //     config.set_max_level(tracing::Level::DEBUG);
    //     config.set_console_config(tracing_wasm::ConsoleConfig::NoReporting);

    //     tracing_wasm::set_as_global_default_with_config(config.build());
    // }
}

pub fn main() -> iced::Result {
    // GraphType::init();
    // GraphType::get_mut_graph_with(|g| {
    //     g.insert_node(
    //         1,
    //         Rc::new(GContainer(Layer::new().push(
    //             Layer::new().push(Text::new("bbbbbbbbbbbbbbbbbbbbb..")),
    //         ))),
    //     )
    // });

    setup_tracing();
    Counter::run(Settings::default())
}

#[derive(Debug)]
struct Counter {
    value: i32,
    increment_button: button::State,
    decrement_button: button::State,
    ddd: StateVar<i32>,
    dt: StateVar<Duration>,
    dt2: Cell<Duration>,
    // an: emg_animation::State<Message>,
    // ffxx: StateVar<CssWidth>,
}
impl Counter {
    // pub fn emg_setup<'a>(
    //     g: &mut GraphType<'a, Message>,
    //     root: GTreeBuilderElement<'a, Message>,
    // ) {
    //     handle_root(g, root);

    //     // GraphType::<Message>::init();
    //     // GraphType::<Message>::get_mut_graph_with(|g| {
    //     //     handle_root(g, root);
    //     //     log::info!("{:#?}", g);
    //     // });
    // }
    // pub fn tree_build(&self) {
}
#[emg_msg]
#[derive(Debug, Copy, Clone, PartialEq)]
enum Message {
    IncrementPressed,
    DecrementPressed,
    // None,
    Event(Event),
    X,
    Y,
}
// impl<'a> MessageTid<'a> for Message {}

// #[derive(Debug, Copy, Clone, PartialEq, Eq)]
// enum Message2 {
//     IncrementPressed,
//     DecrementPressed,
//     None,
//     X,
//     Y,
// }

//@ no need new component
struct ComponentStatic {
    nn: i32,
}
impl ComponentStatic {
    pub fn tree_build<T>(
        this: Rc<RefCell<T>>,
        orders: impl Orders<Message> + 'static,
    ) -> GTreeBuilderElement<Message> {
        gtree! {
            @=componentX
            Layer [Text::new(format!("component static")) ]
        }
    }
}
//@ App
impl Application for Counter {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();
    // type GTreeBuilderElement<'a> = GTreeBuilderElement<'a, Message>;

    #[topo::nested]
    fn new(_flags: (), orders: &impl Orders<Message>) -> (Self, Command<Message>) {
        orders.observe_root_size(|_w, _h| {
            // warn!("width:{} height:{}", &w, &h);
        });
        // ────────────────────────────────────────────────────────────────────────────────

        (
            Self {
                value: 1,
                increment_button: Default::default(),
                decrement_button: Default::default(),
                ddd: use_state(1),
                dt: global_clock(),
                dt2: Cell::new(Duration::ZERO),
                // an: style(vector![opacity(1.)]),
            },
            Command::none(),
        )
    }
    fn title(&self) -> String {
        String::from("Counter1 - Iced")
    }
    fn update(
        &mut self,
        graph: &mut GraphType<Message>,
        orders: &impl Orders<Message>,
        message: Message,
    ) -> Command<Message> {
        match message {
            Message::X => {
                // warn!("message: X");
            }
            Message::Y => {
                // warn!("message: Y");
            }
            Message::IncrementPressed => {
                warn!("get IncrementPressed");
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
            Message::Event(e) => {
                let _g = trace_span!("sys env").entered();
                trace!("sys event: {:?}", &e);
                match e {
                    Event::Window(we) => {
                        match we {
                            emg_bind::window::Event::Resized {
                                /// The new width of the window (in units)
                                width,
                                /// The new height of the window (in units)
                                height,
                            } => {
                                self.ddd.set(width as i32);
                                graph.edge_item_set_size(
                                    &edge_index_no_source("a"),
                                    px(width),
                                    px(height),
                                );
                            }
                        }
                    }
                    Event::OnAnimationFrame(new) => {
                        // warn!("message: OnAnimationFrame");

                        // let _g = debug_span!("sys-env:on animation").entered();
                        // let dt = self.dt.get_with(|old| new.0 - (*old));
                        // debug!("update->on animation: {:?}", &dt);
                        // self.dt.set(new.0);
                        // self.dt2.set(dt);

                        // // self.an.update_animation();

                        // // emg_animation::update(new, &mut self.an);

                        // // self.ffxx.set(emg_layout::styles::w(pc(self
                        // //     .an
                        // //     .get_position(0)
                        // //     * 100.)));

                        // orders.after_next_render("am", |tick| {
                        //     Message::Event(Event::OnAnimationFrame(tick))
                        // });
                    }
                }
            }
        };
        Command::none()
    }
    fn view(&self, g: &GraphType<Message>) -> GelType<Message> {
        // fn view(&mut self) -> Element<Message> {
        //g.upgrade().map(|xg:Rc<RefCell<GraphType<'a, Message>>>| (&**xg.borrow()).view("a")).unwrap()
        // trace!("view graph::\n {:#?}", g);
        // g.view("a")
        g.get_node_item_use_ix(&IdStr::new_inline("a"))
            .unwrap()
            .get_view_gelement_sa(&EPath::<IdStr>::new(vector![edge_index_no_source("a")]))
            .get()

        // ─────────────────────────────────────────────────────────────────

        // Column::new()
        //     .padding(20)
        //     .align_items(Align::Center)
        //     .push(
        //         Button::new(Text::new("Increment"))
        //             .on_press(Message::IncrementPressed),
        //     )
        //     .push(Text::new(self.value.to_string()).size(50))
        //     .push(
        //         Button::new(Text::new("Decrement"))
        //             .on_press(Message::DecrementPressed),
        //     )
        //     .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        trace!("init subscription");
        // subscription::events().map(Message::Event)
        Subscription::none()
    }

    fn tree_build(
        this: Rc<RefCell<Self>>,
        orders: impl Orders<Message> + 'static,
    ) -> GTreeBuilderElement<Message> {
        let a = use_state(19999);
        let b = use_state(333i32);
        let bw = b.watch();

        let aw = a.watch();

        let an: AnimationE<Message> = anima![width(px(80))];

        gtree! {
            @=a
            Layer [
                    // @=modstatic @Mod ComponentStatic::tree_build(this.clone(), orders.clone()),

                    @=taa @E=[w(px(150)),h(px(150)),origin_x(pc(50)),origin_y(pc(0)),align_x(pc(50)),align_y(pc(50))]
                    Checkbox::new(false,"abcd",|_|Message::IncrementPressed)=>[
                    // Text::new(format!("temp34567845678345678"))=>[
                        // aw.clone() => |p:&Rc<GElement<Message>>,num|{
                        //     warn!("run in sa map builder");
                        //     // if let Some(p_text) = p.as_text(){
                        //     //     warn!("downcast_ref to text ok");

                        //     //     Rc::new(p_text.clone().with_content( num.to_string()).into())
                        //     if let Some(p_checkbox) = p.as_generic().and_then(|dyn_gel|dyn_gel.downcast_ref::<Checkbox<Message>>()){
                        //         warn!("downcast_ref::<Checkbox<Message>> ok");

                        //         Rc::new(p_checkbox.clone().with_label( num.to_string().into()).into())
                        //     }else{
                        //         warn!("downcast_ref::<Checkbox<Message>> false");
                        //         p.clone()

                        //     }

                        // } ,
                        bw
                    ],

                @=b2 @E=[
                    an.clone(),
                // w(px(100)),
                h(parent!(CssWidth)+px(30)),origin_x(pc(60)),align_y(pc(70))]
                Button::new(Text::new(format!("2 button in quote..{}", "e"))) => [
                    On:click move |_root, _vdom, _event| {

                        an.interrupt([
                            to![width(px(50))],
                            to![width(pc(100))],
                        ]);

                                    a.set(a.get()+1);


                                    orders.schedule_render()

                                    }
                ]




            ]
        }
    }
}
