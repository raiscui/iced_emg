#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

// ─────────────────────────────────────────────────────────────────────────────

use std::{rc::Rc, sync::Arc};

use tracing_error::ErrorLayer;
use tracing_subscriber::prelude::*;
// ─────────────────────────────────────────────────────────────────────────────

use color_eyre::{eyre::Report, eyre::Result, eyre::WrapErr};
use emg_bind::{
    element::*,
    emg::{edge_index_no_source, EdgeIndex},
    emg_msg,
    emg_msg_macro_prelude::*,
    graph_edit::*,
    runtime::{drag::DRAG, Affine, OrdersContainer, Pos},
    state::{use_state_voa, StateVar},
    trait_prelude::*,
    Sandbox, Settings,
};
use tracing::{debug_span, info, instrument, warn};

fn tracing_init() -> Result<()> {
    // use tracing_error::ErrorLayer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::prelude::*;

    let filter_layer = tracing_tree::HierarchicalLayer::new(2)
        .with_indent_lines(true)
        .with_indent_amount(4)
        .with_targets(true)
        .with_filter(tracing_subscriber::filter::dynamic_filter_fn(
            |metadata, cx| {
                let skip_target = ["emg_state", "underlay", "to_layout_override"];
                for t in skip_target {
                    if metadata.target().contains(t) {
                        return false;
                    }
                }

                let skip_span = ["xxx"];
                for t in skip_span {
                    if metadata.name().contains(t) {
                        return false;
                    }
                }

                let skip_fields = ["native_events"];
                // let skip_fields = ["window_event"];

                for x in metadata.fields() {
                    let f_str = format!("{}", x);
                    if skip_fields.contains(&f_str.as_str()) {
                        return false;
                    }
                }

                // let keep_target = ["emg_element"];
                // if !keep_target.iter().any(|t| metadata.target().starts_with(t)) {
                //     return false;
                // }

                // let keep_span = ["event_matching"];
                // if metadata.is_span() && keep_span.contains(&metadata.name()) {
                //     return true;
                // }

                true
            },
        ))
        .with_filter(tracing_subscriber::EnvFilter::new(
            // "shaping=warn,[DRAG]=debug,[CLICK]=debug,winit_event=debug,[event_matching]=debug,[LayoutOverride]=debug",
            // "shaping=warn,[DRAG]=debug,[event_matching_filter]=debug",
            // "[event_matching]=debug,[event_matching_filter]=debug",
            "video-player=debug,run-loop=debug,RenderLoopCommand=debug,",
        ))
        .with_filter(tracing_subscriber::filter::dynamic_filter_fn(
            |metadata, cx| {
                // let keep_target = ["emg_element"];
                // if !keep_target.iter().any(|t| metadata.target().starts_with(t)) {
                //     return false;
                // }

                let keep_span = [];
                if metadata.is_span() && keep_span.contains(&metadata.name()) {
                    return true;
                }

                keep_span.is_empty()
            },
        ));

    // ─────────────────────────────────────────────────────────────────────────────
    #[cfg(feature = "tracy")]
    let tracy_layer = emg_tracy::tracing_tracy::TracyLayer::new().with_filter(
        tracing_subscriber::EnvFilter::new(
            // "shaping=warn,[DRAG]=debug,[CLICK]=debug,winit_event=debug,[event_matching]=debug,[LayoutOverride]=debug",
            // "shaping=warn,[DRAG]=debug,[event_matching_filter]=debug",
            // "[event_matching]=debug,[event_matching_filter]=debug",
            "loop-tracy=debug",
        ),
    );
    // ─────────────────────────────────────────────────────────────────────

    let ts = tracing_subscriber::registry()
        // .with(layout_override_layer)
        // .with(event_matching_layer)
        // .with(touch_layer)
        // .with(tracing_subscriber::fmt::layer().with_filter(tracing::metadata::LevelFilter::ERROR))
        .with(filter_layer)
        // .with(out_layer)
        .with(ErrorLayer::default());
    #[cfg(feature = "tracy")]
    let ts = ts.with(tracy_layer);
    // ─────────────────────────────────────────────────────────────

    ts.init();

    // ─────────────────────────────────────────────────────────────────────────────

    color_eyre::install()
}

// pub fn main() -> emg_bind::Result {
// #[instrument]

pub fn main() -> Result<()> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    // pub fn main() -> Result<(), Error> {
    #[cfg(debug_assertions)]
    tracing_init()?;
    App::run(Settings {
        vsync: false,
        ..Settings::default()
    })
    .wrap_err("saw a downstream error")
}

#[derive(Default)]
struct App {
    value: i32,
}

#[emg_msg]
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(dead_code)]
enum Message {
    Ignored,
    Empty,
    IncrementPressed,
    DecrementPressed,
}

impl Sandbox for App {
    type Message = Message;

    fn update(
        &mut self,
        // graph: Self::GraphEditor,
        graph: GraphEditor<Self::Message>,
        // orders: &Self::Orders,
        _orders: &OrdersContainer<Self::Message>,
        message: Self::Message,
    ) {
        match message {
            Message::Ignored => (),
            Message::IncrementPressed => {
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
            Message::Empty => {
                use crate::gtree_macro_prelude::*;

                // let width = use_state(|| w(px(50)));
                let ax = use_state(|| align_x(pc(50)));
                let ay = use_state(|| align_y(pc(50)));

                let x = Affine::<f32>::default();

                let pos = Pos::<f32>::default();
                let xx = x * pos;

                // ax.set_with(|x| align_x(pc(3)) + x );
                // width.set_with(|x| match x {
                //     CssWidth::Auto => todo!(),
                //     CssWidth::Length(ll) => (px(10) + ll).into(),
                //     CssWidth::Initial => todo!(),
                //     CssWidth::Inherit => todo!(),
                //     CssWidth::StringValue(_) => todo!(),
                //     CssWidth::Gs(_) => todo!(),
                // });

                let builder = {
                    gtree! {

                            @E=[
                                w(px(50)),
                                // width,
                                h(px(50)),
                                ax,
                                ay,
                                fill(rgb(1,0,0))
                            ]
                            @="b-check" Checkbox::new(false,"b-abcd",|_|{
                                println!("b checkbox");
                            })=>[
                                @="b_click2" On:DRAG  move|ev|{
                                    use owo_colors::OwoColorize;

                                    let _span = debug_span!("DRAG", "{} -> ev:{:?}","on [b-check] drag".on_black().red(),ev)
                                            .entered();
                                        let drag_offset = ev.get_drag_offset();
                                        let offset_trans = drag_offset * Pos::default();


                                        ax.set_with(|v| v + offset_trans);
                                        ay.set_with(|v| v + offset_trans);


                                },
                            ]


                    }
                };

                graph.edit(builder).insert("b").unwrap();
                // insta::assert_display_snapshot!("graph", graph.graph());
            }
        }
    }

    fn tree_build(
        &self,
        orders: OrdersContainer<Self::Message>,
    ) -> GTreeBuilderElement<Self::Message> {
        use emg_bind::gtree_macro_prelude::*;

        let fill_var = use_state(|| fill(hsl(150, 100, 30)));

        let ax = use_state(|| align_x(pc(50)));
        let ay = use_state(|| align_y(pc(50)));
        let width = use_state(|| w(px(50)));
        let bus = orders.bus();

        // let video_el = Video::new(
        //     "video-player",
        //     "file:///Users/cuiluming/Downloads/sintel_trailer-1080p.mp4",
        //     false,
        // );
        // let vp = video_el.player().clone();

        let pause_voa = use_state_voa(|| false);

        let vp_node: StateVar<Rc<GElement<Message>>> = use_state(|| {
            Rc::new(
                (Rc::new((VideoController::Pause, false)) as Rc<dyn EqShaping<GElement<Message>>>)
                    .into(),
            )
        });

        gtree! {
            @="root" Layer [
                @E=[

                origin_x(px(0)),
                origin_y(px(0)),
                // align_x(px(0)),
                // align_y(px(0)),
                w(pc(40)),h(pc(40)),
                b_width(px(2)),
                b_color(rgb(0,0,1)),
                fill(rgba(0, 0, 1, 1))
                ]
                @="y" Layer [
                    // node_ref("b")

                    // @E=[
                    //             w(px(50)),
                    //             // width,
                    //             h(px(50)),
                    //             ax,
                    //             ay,
                    //             fill(rgb(1,0,0))


                    //         ]
                    //         @="b-check" Checkbox::new(false,"b-abcd",move|_|{
                    //             println!("b checkbox");
                    //         })=>[
                    //             @="b_click2" On:DRAG  move|ev|{
                    //                 use nu_ansi_term::Color::Red;
                    //                 let _span = debug_span!("DRAG", "{} -> ev:{:?}",Red.paint("on [b-check] drag"),ev)
                    //                         .entered();
                    //                     let drag_offset = ev.get_drag_offset();
                    //                     let offset_trans = drag_offset * Pos::default();


                    //                     ax.set_with(|v| v + offset_trans);
                    //                     ay.set_with(|v| v + offset_trans);


                    //             },
                    //         ]

                ],
                // ─────────────────────────────────────────────

                @E=[

                        origin_x(pc(0)),align_x(pc(50)),
                        origin_y(pc(0)),align_y(pc(0)),
                        w(pc(40)),h(pc(40)),
                        fill_var,
                        b_width(px(2)),
                        b_color(rgb(1,0,0))
                    ]
                @="x" Layer [

                    @="x_click" On:CLICK  move||{
                        let _span = debug_span!("CLICK", "on [x] click, moving a->b to m->b")
                                .entered();

                                pause_voa.set(!pause_voa.get_out_val());


                                // vp_node.set(
                                //     Rc::new(
                                //         (Rc::new((VideoController::Pause, true)) as Rc<dyn EqShaping<GElement<Message>>>)
                                //             .into(),
                                //     )
                                // )

                                // vp.set_source_paused(true);
                        Message::Empty
                    },

                    @E=[
                        origin_x(px(0)),
                        origin_y(px(0)),
                        align_x(px(20)),
                        align_y(px(0)),
                        w(pc(50)),
                        h(pc(25)),
                        fill(rgba(1, 0.5, 0, 1))
                    ]
                    @="a" Layer [

                        @E=[
                            origin_x(pc(50)),
                            origin_y(pc(50)),
                            align_x(pc(50)),
                            align_y(pc(50)),
                            w(pc(50)),
                            h(pc(75)),
                            fill(rgba(1, 0, 0, 1))
                        ]
                        @="b" Layer [


                            //   builder



                        ],
                    ],
                    @E=[
                        origin_x(px(0)),
                        origin_y(pc(50)),
                        align_x(px(20)),
                        align_y(pc(50)),
                        w(pc(50)),
                        h(pc(25)),
                        fill(rgba(0, 0.5, 0, 1))
                    ]
                    @="m" Layer [
                        // b will move here ─────────────────────────────

                    ],
                    @E=[
                        origin_x(px(0)),
                        origin_y(pc(100)),
                        align_x(px(20)),
                        align_y(pc(100)),
                        w(pc(50)),
                        h(pc(25)),
                        fill(rgba(0, 0, 0.5, 1))
                    ]
                    @="w" Layer [

                        // @E=[
                        //     w(px(50)),
                        //     // width,
                        //     h(px(50)),
                        //     ax,
                        //     ay,
                        //     fill(rgb(1,0,0))
                        // ]
                        // @="b-check2" Checkbox::new(false,"b-abcd2",|_|{
                        //     println!("b2 checkbox");
                        // })=>[

                        //     // Checkbox::new(false,"b-abcd22",|_|{
                        //     //     //FIXME ,find why not call this
                        //     //     println!("b22 checkbox");
                        //     // })
                        // ]


                        // @E=[
                        //     // origin_x(pc(100)),
                        //     // origin_y(pc(100)),
                        //     // align_x(pc(100)),
                        //     // align_y(pc(50)),
                        //     w(pc(100)),
                        //     h(pc(100)),
                        //     // fill(rgba(0, 1, 0, 1))
                        // ]
                        // Video::new(
                        //     "video-player",
                        //     "file:///Users/cuiluming/Downloads/sintel_trailer-1080p.mp4",
                        //     false,
                        // )
                        // .with_setup(&(VideoController::Pause,pause_voa)) =>[
                        //     // GElement::SaNode_( vp_node.watch())
                        // ],
                        // @E=[
                        //     // origin_x(pc(100)),
                        //     // origin_y(pc(100)),
                        //     align_x(pc(20)),
                        //     align_y(pc(20)),
                        //     w(pc(30)),
                        //     h(pc(30)),
                        //     // fill(rgba(0, 1, 0, 1))
                        // ]
                        // Video::new(
                        //     "video-player",
                        //     "file:///Users/cuiluming/Downloads/sintel_trailer-1080p.mp4",
                        //     false,
                        // ).with_setup(&(VideoController::Pause,pause_voa)),
                        // @E=[
                        //     // origin_x(pc(30)),
                        //     // origin_y(pc(30)),
                        //     align_x(pc(40)),
                        //     align_y(pc(40)),
                        //     w(pc(30)),
                        //     h(pc(30)),
                        //     // fill(rgba(0, 1, 0, 1))
                        // ]
                        // Video::new(
                        //     "video-player",
                        //     "file:///Users/cuiluming/Downloads/sintel_trailer-1080p.mp4",
                        //     false,
                        // ).with_setup(&(VideoController::Pause,pause_voa)),
                        // @E=[
                        //     // origin_x(pc(30)),
                        //     // origin_y(pc(30)),
                        //     align_x(pc(60)),
                        //     align_y(pc(60)),
                        //     w(pc(30)),
                        //     h(pc(30)),
                        //     // fill(rgba(0, 1, 0, 1))
                        // ]
                        // Video::new(
                        //     "video-player",
                        //     "file:///Users/cuiluming/Downloads/sintel_trailer-1080p.mp4",
                        //     false,
                        // ).with_setup(&(VideoController::Pause,pause_voa)),
                        // @E=[
                        //     // origin_x(pc(30)),
                        //     // origin_y(pc(30)),
                        //     align_x(pc(80)),
                        //     align_y(pc(80)),
                        //     w(pc(30)),
                        //     h(pc(30)),
                        //     // fill(rgba(0, 1, 0, 1))
                        // ]
                        // Video::new(
                        //     "video-player",
                        //     "file:///Users/cuiluming/Downloads/sintel_trailer-1080p.mp4",
                        //     false,
                        // ).with_setup(&(VideoController::Pause,pause_voa)),
                        // @E=[
                        //     // origin_x(pc(30)),
                        //     // origin_y(pc(30)),
                        //     align_x(pc(100)),
                        //     align_y(pc(100)),
                        //     w(pc(30)),
                        //     h(pc(30)),
                        //     // fill(rgba(0, 1, 0, 1))
                        // ]
                        // Video::new(
                        //     "video-player",
                        //     "file:///Users/cuiluming/Downloads/sintel_trailer-1080p.mp4",
                        //     false,
                        // ).with_setup(&(VideoController::Pause,pause_voa)),
                        // @E=[
                        //     // origin_x(pc(30)),
                        //     // origin_y(pc(30)),
                        //     align_x(pc(120)),
                        //     align_y(pc(120)),
                        //     w(pc(30)),
                        //     h(pc(30)),
                        //     // fill(rgba(0, 1, 0, 1))
                        // ]
                        // Video::new(
                        //     "video-player",
                        //     "file:///Users/cuiluming/Downloads/sintel_trailer-1080p.mp4",
                        //     false,
                        // ).with_setup(&(VideoController::Pause,pause_voa)),
                        // @E=[
                        //     // origin_x(pc(30)),
                        //     // origin_y(pc(30)),
                        //     align_x(pc(140)),
                        //     align_y(pc(140)),
                        //     w(pc(30)),
                        //     h(pc(30)),
                        //     // fill(rgba(0, 1, 0, 1))
                        // ]
                        // Video::new(
                        //     "video-player",
                        //     "file:///Users/cuiluming/Downloads/sintel_trailer-1080p.mp4",
                        //     false,
                        // ).with_setup(&(VideoController::Pause,pause_voa)),
                        // ─────────────────────────────

                        // ─────────────────────────────

                        // ─────────────────────────────


                        // @E=[
                        //     // origin_x(pc(30)),
                        //     // origin_y(pc(30)),
                        //     align_x(pc(160)),
                        //     align_y(pc(160)),
                        //     w(pc(30)),
                        //     h(pc(30)),
                        //     // fill(rgba(0, 1, 0, 1))
                        // ]
                        // Video::new(
                        //     "video-player",
                        //     "file:///Users/cuiluming/Downloads/sintel_trailer-1080p.mp4",
                        //     false,
                        // ).with_setup(&(VideoController::Pause,pause_voa)),
                        // @E=[
                        //     // origin_x(pc(30)),
                        //     // origin_y(pc(30)),
                        //     align_x(pc(180)),
                        //     align_y(pc(180)),
                        //     w(pc(30)),
                        //     h(pc(30)),
                        //     // fill(rgba(0, 1, 0, 1))
                        // ]
                        // Video::new(
                        //     "video-player",
                        //     "file:///Users/cuiluming/Downloads/sintel_trailer-1080p.mp4",
                        //     false,
                        // ).with_setup(&(VideoController::Pause,pause_voa)),
                        // @E=[
                        //     // origin_x(pc(30)),
                        //     // origin_y(pc(30)),
                        //     align_x(pc(200)),
                        //     align_y(pc(200)),
                        //     w(pc(30)),
                        //     h(pc(30)),
                        //     // fill(rgba(0, 1, 0, 1))
                        // ]
                        // Video::new(
                        //     "video-player",
                        //     "file:///Users/cuiluming/Downloads/sintel_trailer-1080p.mp4",
                        //     false,
                        // ).with_setup(&(VideoController::Pause,pause_voa)),





                        // @E=[
                        //     origin_x(pc(100)),
                        //     origin_y(pc(50)),
                        //     align_x(pc(100)),
                        //     align_y(pc(50)),
                        //     w(pc(50)),
                        //     h(pc(15)),
                        //     fill(rgba(0, 1, 0, 1))
                        // ]
                        // @="ref_x_click" node_ref("x_click")
                    ],
                ],

            ]
        }
    }

    fn root_eix(&self) -> EdgeIndex {
        edge_index_no_source("root")
    }

    // #[instrument(skip(self, g), ret)]
    // fn view(&self, g: &GraphType<Self::Message>) -> GelType<Self::Message> {
    //     g.get_node_item_use_ix(&IdStr::new_inline("debug_layer"))
    //         .unwrap()
    //         .get_view_gelement_sa(&EPath::new(vector![edge_index_no_source("debug_layer")]))
    //         .get()
    // }
}
