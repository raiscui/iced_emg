use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

// use emg_animation::{interrupt, opacity, style, to};

// use emg_bind::{
//     better_any::{impl_tid, tid, type_id, Tid, TidExt},
//     button, edge_index_no_source,
//     event::Event,
//      Button, Checkbox, Element, GElement, GraphMethods,
//     GraphView, Subscription, Text,
// };

// use emg_common::{into_vector, parent, IdStr, TypeCheckObjectSafe};
// use emg_bind::layout::{
//     add_values::origin_x,
//     css, global_clock,
//     styles::{pc, CssWidth},
// };

use emg_bind::{
    better_any::TidAble,
    common::{px, vector, IdStr},
    emg::edge_index_no_source,
    emg_msg, gtree,
    layout::{anima, global_clock, styles::width, AnimationE, EPath},
    runtime::{Application, Command},
    state::{use_state, CloneStateAnchor, StateVar},
    topo,
    widget::*,
    GelType, GraphType, Settings,
};
// use seed_styles::w;
use tracing::warn;
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
    #[cfg(not(debug_assertions))]
    {
        // let mut config = tracing_wasm::WASMLayerConfigBuilder::default();
        // config.set_max_level(tracing::Level::WARN);
        // tracing_wasm::set_as_global_default_with_config(config.build());
    }
}

pub fn main() -> emg_bind::Result {
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
    // increment_button: button::State,
    // decrement_button: button::State,
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
    // Event(Event),
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
    type Executor = emg_bind::executor::Default;
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
                // increment_button: Default::default(),
                // decrement_button: Default::default(),
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
            } // Message::Event(e) => {
              //     let _g = trace_span!("sys env").entered();
              //     trace!("sys event: {:?}", &e);
              //     match e {
              //         Event::Window(we) => {
              //             match we {
              //                 emg_bind::window::Event::Resized {
              //                     /// The new width of the window (in units)
              //                     width,
              //                     /// The new height of the window (in units)
              //                     height,
              //                 } => {
              //                     self.ddd.set(width as i32);
              //                     graph.edge_item_set_size(
              //                         &edge_index_no_source("a"),
              //                         px(width),
              //                         px(height),
              //                     );
              //                 }
              //             }
              //         }
              //         Event::OnAnimationFrame(new) => {
              //             // warn!("message: OnAnimationFrame");

              //             // let _g = debug_span!("sys-env:on animation").entered();
              //             // let dt = self.dt.get_with(|old| new.0 - (*old));
              //             // debug!("update->on animation: {:?}", &dt);
              //             // self.dt.set(new.0);
              //             // self.dt2.set(dt);

              //             // // self.an.update_animation();

              //             // // emg_animation::update(new, &mut self.an);

              //             // // self.ffxx.set(emg_layout::styles::w(pc(self
              //             // //     .an
              //             // //     .get_position(0)
              //             // //     * 100.)));

              //             // orders.after_next_render("am", |tick| {
              //             //     Message::Event(Event::OnAnimationFrame(tick))
              //             // });
              //         }
              //     }
              // }
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

    // fn subscription(&self) -> Subscription<Self::Message> {
    //     trace!("init subscription");
    //     // subscription::events().map(Message::Event)
    //     Subscription::none()
    // }

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

                @=a1 @E=[
                    {md==22},
                    w(pc(100)),h(pc(100)),

                ]
                Layer [

                    @=a2 @E=[
                        {md==30},

                        w(pc(100)),h(pc(100)),

                    ]
                    Layer [
                        @=b @E=[
                            {md==120,my_other_gap==28},
                            {"nn":{
                                    // width==0,
                                    // height==29,
                                    // top==0,
                                    // left==0,
                                    // bottom==20,
                                    // right==99,
                                }
                            },
                            {
                                @h (#b1)(#b2) chain-height chain-width(250)
                            },
                        // w(pc(50)),
                        // h(pc(50)),
                        css(bg_color(hsl(333,70,20)))
                        ]
                        Layer [

                            @E=[
                                {md==10},
                                w(pc(90)),h(pc(90)),
                                css(bg_color(hsl(00,70,60)))
                            ]
                            @=b0
                            Layer[],

                            @=b1 @E=[
                                w(px(50)),
                                h(px(50)),
                            //origin_x(pc(50)),origin_y(pc(0)),align_x(pc(50)),align_y(pc(50)),
                            css(bg_color(hsl(11,70,70)))]
                            Layer[],

                            @=b2 @E=[
                                w(pc(50)),
                                h(pc(50)),
                            //origin_x(pc(50)),origin_y(pc(0)),align_x(pc(50)),align_y(pc(50)),
                            css(bg_color(hsl(33,70,70)))]
                            Layer[],

                            // @=b3 @E=[w(px(50)),h(px(50)),
                            // //origin_x(pc(50)),origin_y(pc(0)),align_x(pc(50)),align_y(pc(50)),
                            // css(bg_color(hsl(55,70,70)))]
                            // Layer[],

                            // @=b4 @E=[w(px(50)),h(px(50)),
                            // //origin_x(pc(50)),origin_y(pc(0)),align_x(pc(50)),align_y(pc(50)),
                            // css(bg_color(hsl(77,70,70)))]
                            // Layer[],

                            // // // @=taa @E=[w(px(150)),h(px(150)),origin_x(pc(50)),origin_y(pc(0)),align_x(pc(50)),align_y(pc(50))]
                            // @=b5 @E=[w(pc(50)),h(pc(50)),
                            // //origin_x(pc(50)),origin_y(pc(0)),align_x(pc(50)),align_y(pc(50)),
                            // css(bg_color(hsl(55,70,30)))]
                            // Layer[],
                        ],
                    ],
                ],






                // @=modstatic @Mod ComponentStatic::tree_build(this.clone(), orders.clone()),


                // node_ref("a"),


                // @=b2 @E=[
                //     an.clone(),
                // // w(px(100)),
                // h(parent!(CssWidth)*0.3),origin_x(pc(50)),align_y(pc(50))]
                // Button::new(Text::new(format!("2 button in quote..{}", "e"))) => [
                //     On:click move |_root, _vdom, _event| {

                //         an.interrupt([
                //             to![width(px(50))],
                //             to![width(pc(100))],
                //         ]);

                //                     a.set(a.get()+1);


                //                     orders.schedule_render()

                //                     }
                // ]




            ]
        }
    }
}
