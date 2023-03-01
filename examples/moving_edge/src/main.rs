#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
// ─────────────────────────────────────────────────────────────────────────────

use color_eyre::{eyre::Report, eyre::Result, eyre::WrapErr};
use emg_bind::{
    element::*,
    emg::{edge_index, Direction::Incoming},
    emg_msg,
    emg_msg_macro_prelude::*,
    graph_edit::*,
    runtime::OrdersContainer,
    Sandbox, Settings,
};
use std::{cell::Cell, rc::Rc};
use tracing::{debug_span, instrument};
fn tracing_init() -> Result<(), Report> {
    // use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    let error_layer =
        tracing_subscriber::fmt::layer().with_filter(tracing::metadata::LevelFilter::ERROR);

    // #[cfg(not(feature = "debug"))]
    // let out_layer = tracing_tree::HierarchicalLayer::new(2)
    //     .with_indent_lines(true)
    //     .with_indent_amount(4)
    //     .with_targets(true)
    //     .with_filter(tracing_subscriber::filter::dynamic_filter_fn(
    //         |metadata, _cx| {
    //             tracing::debug!(target: "tracing", "metadata.level() = {:?}, metadata.is_span() = {:?}, metadata.name() = {:?}", metadata.level(), metadata.is_span(), metadata.name());
    //             // if metadata.level() <= &tracing::Level::DEBUG{
    //             //     // If this *is* "interesting_span", make sure to enable it.
    //             //     if metadata.is_span() && metadata.name() == "LayoutOverride" {
    //             //         return true;
    //             //     }

    //             //     // Otherwise, are we in an interesting span?
    //             //     if let Some(current_span) = cx.lookup_current()  {
    //             //         return current_span.name() == "LayoutOverride";
    //             //     }
    //             // }
    //             // ─────────────────────────────────────────────────────

    //             // #[cfg(feature = "debug")]
    //             // return false;

    //             !metadata.target().contains("anchors")
    //                 && !metadata.target().contains("emg_layout")
    //                 && !metadata.target().contains("emg_state")
    //                 && !metadata.target().contains("cassowary")
    //                 && !metadata.target().contains("wgpu")
    //                 && metadata.level() <= &tracing::Level::INFO // global tracing level
    //             // && !metadata.target().contains("winit event")
    //             // && !metadata.fields().field("event").map(|x|x.to_string())
    //             // && !metadata.target().contains("winit event: DeviceEvent")
    //         },
    //     ));

    // #[cfg(feature = "debug")]
    // let layout_override_layer = tracing_tree::HierarchicalLayer::new(2)
    //     .with_indent_lines(true)
    //     .with_indent_amount(4)
    //     .with_targets(true)
    //     .with_filter(EnvFilter::new("[LayoutOverride]=debug"));

    // #[cfg(feature = "debug")]
    // let event_matching_layer = tracing_tree::HierarchicalLayer::new(2)
    //     .with_indent_lines(true)
    //     .with_indent_amount(4)
    //     .with_targets(true)
    //     .with_filter(EnvFilter::new("[event_matching]=debug"));

    // #[cfg(feature = "debug")]
    // let touch_layer = tracing_tree::HierarchicalLayer::new(2)
    //     .with_indent_lines(true)
    //     .with_indent_amount(4)
    //     .with_targets(true)
    //     .with_filter(EnvFilter::new("[Touch]=debug"));

    //NOTE emg_layout
    #[cfg(feature = "debug")]
    let emg_layout_layer = tracing_tree::HierarchicalLayer::new(2)
        .with_indent_lines(true)
        .with_indent_amount(4)
        .with_targets(true)
        .with_filter(tracing_subscriber::EnvFilter::new(
            // "emg_layout=debug,emg_layout[build inherited cassowary_generals_map],emg_layout[LayoutOverride]=error",
            // "[GElement-shaping]=debug",
            // "error,[sa gel in map clone]=debug",
            "error",
            // "error",
        ));
    // ─────────────────────────────────────────────────────────────────────────────

    tracing_subscriber::registry()
        // .with(layout_override_layer)
        // .with(event_matching_layer)
        // .with(touch_layer)
        .with(error_layer)
        // .with(emg_layout_layer)
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
        orders: &OrdersContainer<Self::Message>,
        message: Self::Message,
    ) {
        match message {
            Message::IncrementPressed => {
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
            Message::Empty => graph
                .edit(edge_index("a", "b"))
                .moving(Incoming, "m")
                .or_else(|editor, e| match e {
                    ElementError::GraphError(ee) => match ee {
                        emg_bind::emg::Error::CanNotGetEdge => editor
                            .edit(edge_index("m", "b"))
                            .moving(Incoming, "a")
                            .into_result(),

                        _ => todo!(),
                    },
                })
                .unwrap(),
        }
    }

    fn tree_build(&self, orders: Self::Orders) -> GTreeBuilderElement<Self::Message> {
        use emg_bind::gtree_macro_prelude::*;

        let n = Rc::new(Cell::new(100));
        let ww = use_state(|| w(px(100)));
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
                        let _span = debug_span!("Moving", "on [x] click, moving a->b to m->b")
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


                            @E=[
                                w(px(50)),h(px(50)),
                            ]
                            @="b-check" Checkbox::new(false,"b-abcd",|_|{})=>[
                            ],

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

    fn root_eix(&self) -> &str {
        "root"
    }

    // #[instrument(skip(self, g), ret)]
    // fn view(&self, g: &GraphType<Self::Message>) -> GelType<Self::Message> {
    //     g.get_node_item_use_ix(&IdStr::new_inline("debug_layer"))
    //         .unwrap()
    //         .get_view_gelement_sa(&EPath::new(vector![edge_index_no_source("debug_layer")]))
    //         .get()
    // }
}
