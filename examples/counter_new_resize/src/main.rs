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
        config.set_max_level(tracing::Level::WARN);
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
        let b = a.watch();
        // let sa_text =
        //     use_state(GElement::from(Text::new(format!("temp----------"))));
        let saw = origin_x(pc(50));
        debug!("saw:=========={} ", saw.type_name());
        let saw = w(w(pc(50)));
        debug!("saw:=========={} ", saw.type_name());

        let f = RefCell::new(1111111);

        let that = this.clone();
        let that2 = this.clone();
        let that3 = this.clone();
        let an: AnimationE<Message> = anima![width(px(80))];
        let orders2 = orders.clone();
        let orders3 = orders.clone();

        let dyn_v = use_state(22222222);
        let dyn_v2 = dyn_v.clone();

        let dyn_tree = use_state(dict_k_into! {
            // "1".to_string()=>GTreeBuilderElement::Layer("1".to_string(),vec![],vec![])
            //TODO ”固化“ / ”永久化“ 标签文字 定义
            "aa1" => gtree!{
                @=aa1
                Layer [
                    Text::new(format!("aa1***********8"))=>[
                        RefreshUse dyn_v
                    ],


                ]
            },
            "aa2" => gtree!{
                @=aa2
                Layer [
                    Text::new(format!("aa2***********9"))=>[

                        // RefreshUse dyn_v
                    ]
                ]
            }
        });

        let dyn_tree2 = dyn_tree.clone();

        // GTreeBuilderElement::Layer("".to_string(), vec![], vec![])

        // // let component_static = ComponentStatic::tree_build(that, orders);
        // // ───────────────────s──────────────────────────────────────────────
        // let xxx = 100i32;
        let aw = a.watch();

        gtree! {
            @=a
            Layer [


                 @=b @E=[w(w(pc(50))),h(pc(50)),origin_x(pc(50)),align_x(pc(50))]
                 Layer [
                    @=modstatic @Mod ComponentStatic::tree_build(that3.clone(), orders3.clone()),
                    @=c @E=[w(px(150)),h(px(50)),origin_x(pc(50)),origin_y(pc(50)),align_x(pc(50)),align_y(pc(50))]
                    Layer [
                        @=clickbox1
                        Checkbox::new(false,"abcd",|_|Message::IncrementPressed)=>[
                            // @=clickbox2
                            // Checkbox::new(false,"222",|_|Message::IncrementPressed)=>[
                            //     @=textinclickbox
                            //     Text::new(format!("checkbox-text")),
                            // ],
                            aw => |p:&Rc<GElement<Message>>,num|{
                                warn!("run in sa map builder");
                                if let Some(p_checkbox) = p.as_generic().and_then(|dyn_gel|dyn_gel.downcast_ref::<Checkbox<Message>>()){
                                    warn!("downcast_ref::<Checkbox<Message>> ok");

                                    Rc::new(p_checkbox.clone().with_label( num.to_string().into()).into())
                                }else{
                                    warn!("downcast_ref::<Checkbox<Message>> false");
                                    p.clone()

                                }
                                // let mut new_p_layer = (**p_layer).clone();
                                // new_p_layer.refresh_use(num);//if layer, push in layer
                                // Rc::new(new_p_layer)
                            } => [ ],
                            // node_ref("temp"),
                            //xx


                        ]
                    ],
                    // sa_text,
                    // @=mod1 @E=[w(w(pc(50))),h(pc(50)),origin_y(pc(50)),align_y(pc(60))]
                    // @Mod dyn_tree.into(),

                    @=temp @E=[w(px(150)),h(px(150)),origin_x(pc(50)),origin_y(pc(0)),align_x(pc(50)),align_y(pc(50))]
                    Text::new(format!("temp34567845678345678"))=>[
                        // Text::new(format!("overlay----------"))

                    ],

                    @=lay_refresh
                    Layer [RefreshUse GElement::from( Text::new(format!("ee up")))],

                    @=an @E=[w(px(150)),origin_x(pc(50)),origin_y(pc(0)),align_x(pc(50)),align_y(pc(100))]
                    Text::new(format!("in quote.. {}", "b")) => [
                        // xxx,

                        RefreshUse ||{100},
                        // RefreshUse  move||{
                        //     that3.borrow().an.get_position(0)
                        // },
                        // RefreshUse  move||{that.borrow().ddd}
                    ],
                    // @E=[w(px(150)),origin_x(pc(50)),align_x(pc(50))]
                    // Text::new(format!("in quote.. {}", "b")) => [
                    //     RefreshUse ||{100},
                    //     RefreshUse  move||{
                    //         a.set_with(|x|x+1);
                    //         a.get()
                    //     },
                    //     // RefreshUse  move||{that.borrow().ddd}
                    // ],
                    @=t11 @E=[w(px(150)),origin_x(pc(100)),align_x(pc(100))]
                    Text::new(format!("in quote.. {}", "b")) => [
                        RefreshUse ||100,
                        RefreshUse this.borrow().ddd,
                        // RefreshUse a.watch(),

                        // RefreshUse  move||{that.borrow().ddd}
                    ],
                    @=t12 @E=[w(px(150)),origin_x(pc(0)),align_x(pc(0))]
                    Text::new(format!("dt.. {}", "b")) => [
                        RefreshUse move||{that.borrow().dt.get_with(Duration:: subsec_millis)}
                        // RefreshUse  move||{that.borrow().ddd}
                    ],
                    @=t13 @E=[w(px(250)),origin_x(pc(0)),align_y(pc(140))]
                    Text::new(format!("dt.. {}", "b")) => [
                        RefreshUse move||{that2.borrow().dt2.get(). subsec_millis()}
                        // RefreshUse  move||{that.borrow().ddd}
                    ],
                    @=e @E=[w(pc(100)),h(px(40)),css(background_color("red")),origin_x(pc(50)),align_y(pc(70))]
                    Layer [
                        @=eb @E=[w(px(150)),h(px(30)),origin_x(pc(60)),align_y(pc(250))]
                        Button::new(Text::new(format!("2 button in quote..{}", "e"))) => [
                            On:click move||{
                                // a.set((*a.get()).clone()+1);
                                // a.set(a.get()+1);
                                trace!("bbbbbbbbbbbbb");

                                a.set_with(|v|v+1);

                                let new_dom: GTreeBuilderElement<Message> = gtree! {
                                    // @=aa3
                                    Layer [
                                        Text::new(format!("aa3***********3 "))=>[

                                            // RefreshUse a
                                        ]
                                    ]
                                };

                                let xx = a.get().to_string() + "xx";

                                dyn_tree2.set_with_once(|dict| {
                                    debug!("dyn_tree2.set_with_once:{}",&xx);

                                    let mut  d = dict.clone();

                                    for _ in 0..300 {
                                    a.set(a.get()+1);

                                        d.insert(IdStr::new(a.get().to_string()),new_dom.clone());


                                    }
                                    d


                                });

                                orders2.schedule_render()

                                // this.borrow_mut().ddd +=1;
                            },
                            // On:dblclick move||{
                            //     // a.set((*a.get()).clone()+1);
                            //     // a.set(a.get()+1);
                            //     trace!("ccccccccccccc");
                            //     a.set_with(|v|v+1);
                            //     // this.borrow_mut().ddd +=1;
                            //     Message::None
                            // }
                        ],
                        // @=b2 @E=[an.clone(),h(px(30)),origin_x(pc(60)),align_y(pc(300))]
                        @=b2 @E=[an.clone(),h(parent!(CssWidth)+px(30)),origin_x(pc(60)),align_y(pc(300))]
                        Button::new(Text::new(format!("2 button in quote..{}", "e"))) => [
                            On:click move |_root, vdom, _event| {

                                an.interrupt([
                                    to![width(px(50))],
                                    to![width(pc(100))],
                                ]);

                                            a.set(a.get()+1);
                                            dyn_v2.set_with(|v|v+1);


                                            // let d = dyn_tree2.get();
                                            // debug!("get dict");

                                            // dyn_tree2.set(d);
                                            // let new_dom: GTreeBuilderElement<Message> = gtree! {
                                            //     // @=aa3
                                            //     Layer [
                                            //         Text::new(format!("aa3***********3 "))=>[

                                            //             RefreshUse a
                                            //         ]
                                            //     ]
                                            // };

                                            // let xx = a.get().to_string() + "xx";

                                            // dyn_tree2.set_with_once(|dict| {
                                            //     debug!("dyn_tree2.set_with_once:{}",&xx);

                                            //     let mut  d = dict.clone();

                                            //     for _ in 0..300 {
                                            //     a.set(a.get()+1);

                                            //         d.insert(a.get().to_string(),new_dom.clone());


                                            //     }
                                            //     d


                                            // });
                                            debug!("set dict");


                                            debug!("will render");
                                            orders.schedule_render()
                                        // orders.schedule_render_then("am",
                                        //     |tick| {
                                        //         Message::Event(Event::OnAnimationFrame(tick))
                                        //     }
                                        // )

                                        // orders.publish(Message::X);

                                        // orders.after_next_render(|tick| {
                                            // Message::Event(Event::OnAnimationFrame(tick))
                                        // });
                                        // let o = orders.clone();
                                        // vdom.schedule_render_with_orders(o);
                                            //  vdom.schedule_render();
                                        // Some(Message::None)
                                    //    Some( Message::Y)
                                        // None
                                            }
                        ]
                    ],
                ]
            ]
        }
    }
}
