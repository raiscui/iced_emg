#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
// ─────────────────────────────────────────────────────────────────────────────

use color_eyre::{eyre::Report, eyre::Result, eyre::WrapErr};
use emg_bind::{
    element::*,
    emg::{edge_index, edge_index_no_source, Direction::Incoming, EdgeIndex},
    emg_msg,
    emg_msg_macro_prelude::*,
    graph_edit::*,
    runtime::OrdersContainer,
    Sandbox, Settings,
};
use tracing::{debug_span, info, instrument, warn};
fn tracing_init() -> Result<(), Report> {
    // use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    let error_layer =
        tracing_subscriber::fmt::layer().with_filter(tracing::metadata::LevelFilter::ERROR);

    let filter_layer = tracing_tree::HierarchicalLayer::new(2)
        .with_indent_lines(true)
        .with_indent_amount(4)
        .with_targets(true)
        .with_filter(tracing_subscriber::EnvFilter::new(
            // "emg_layout=debug,emg_layout[build inherited cassowary_generals_map],emg_layout[LayoutOverride]=error",
            // "[GElement-shaping]=debug",
            // "error,[sa gel in map clone]=debug",
            // "[event_matching]=debug,[editor]=debug,[checkbox]=debug",
            "[event_matching]=debug",
            // "error",
        ))
        .with_filter(tracing_subscriber::filter::dynamic_filter_fn(
            |metadata, cx| {
                let skip_target = ["emg_state", "winit event"];
                for t in skip_target {
                    if metadata.target().contains(t) {
                        return false;
                    }
                }

                let keep_target = ["emg_element"];
                if !keep_target.iter().any(|t| metadata.target().starts_with(t)) {
                    return false;
                }

                let keep_span = ["event_matching"];
                if metadata.is_span() && keep_span.contains(&metadata.name()) {
                    return true;
                }

                // if let Some(current_span) = cx.lookup_current() {
                //     return keep_span.contains(&current_span.name());
                // }

                false
            },
        ));

    // ─────────────────────────────────────────────────────────────────────────────

    tracing_subscriber::registry()
        // .with(layout_override_layer)
        // .with(event_matching_layer)
        // .with(touch_layer)
        .with(error_layer)
        .with(filter_layer)
        // .with(out_layer)
        .init();

    // ─────────────────────────────────────────────────────────────────────────────

    color_eyre::install()
}

// pub fn main() -> emg_bind::Result {
// #[instrument]

pub fn main() -> Result<(), Report> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    // pub fn main() -> Result<(), Error> {
    #[cfg(debug_assertions)]
    tracing_init()?;
    App::run(Settings::default()).wrap_err("saw a downstream error")
}

#[derive(Default)]
struct App {
    value: i32,
}

#[emg_msg]
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(dead_code)]
enum Message {
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
            Message::IncrementPressed => {
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
            Message::Empty => {
                use crate::gtree_macro_prelude::*;

                let builder = {
                    gtree! {

                            @E=[
                                w(px(50)),h(px(50)),
                            ]
                            @="b-check" Checkbox::new(false,"b-abcd",|_|{
                                println!("b checkbox");
                            })


                    }
                };

                graph.edit(builder).insert("b").unwrap();
                // insta::assert_display_snapshot!("graph", graph.graph());
            }
        }
    }

    fn tree_build(&self, _orders: Self::Orders) -> GTreeBuilderElement<Self::Message> {
        use emg_bind::gtree_macro_prelude::*;

        let fill_var = use_state(|| fill(hsl(150, 100, 30)));
        gtree! {
            @="root" Layer [
                @E=[

                origin_x(px(0)),
                origin_y(px(0)),
                align_x(px(0)),
                align_y(px(0)),
                w(pc(40)),h(pc(40)),
                b_width(px(2)),
                b_color(rgb(0,0,1)),
                fill(rgba(0, 0, 1, 1))
                ]
                @="y" Layer [
                    // node_ref("b")
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

                    @="x_click" On:CLICK  ||{
                        let _span = debug_span!("CLICK", "on [x] click, moving a->b to m->b")
                                .entered();

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


                            // @="fb"  builder

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
                        //     origin_x(pc(100)),
                        //     origin_y(pc(50)),
                        //     align_x(pc(100)),
                        //     align_y(pc(50)),
                        //     w(pc(50)),
                        //     h(pc(15)),
                        //     fill(rgba(0, 1, 0, 1))
                        // ]
                        @="ref_x_click" node_ref("x_click")
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
