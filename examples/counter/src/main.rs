use color_eyre::{
    eyre::WrapErr,
    eyre::{eyre, Report},
    Section,
};
use emg_bind::{
    better_any::TidAble,
    common::mouse::CLICK,
    common::px,
    element::*,
    emg_msg, gtree,
    layout::styles::{fill, hsl, w},
    state::use_state,
    Error, Sandbox, Settings,
};
use std::{cell::Cell, rc::Rc};
use tracing::{debug, debug_span, info, instrument};
use tracing_subscriber::EnvFilter;
// use tracing_error::InstrumentResult;
fn tracing_init() -> Result<(), Report> {
    // use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;

    let out_layer =
    //  tracing_subscriber::fmt::layer()
    tracing_tree::HierarchicalLayer::new(2) .with_indent_lines(true)
    .with_indent_amount(4)
        .with_targets(true)
        .with_filter(tracing_subscriber::filter::dynamic_filter_fn(|metadata,cx| {




            // if metadata.level() <= &tracing::Level::DEBUG{
            //     // If this *is* "interesting_span", make sure to enable it.
            //     if metadata.is_span() && metadata.name() == "LayoutOverride" {
            //         return true;
            //     }

            //     // Otherwise, are we in an interesting span?
            //     if let Some(current_span) = cx.lookup_current()  {
            //         return current_span.name() == "LayoutOverride";
            //     }
            // }
            // return false;



                 !metadata.target().contains("anchors")
                && !metadata.target().contains("emg_layout")
                && !metadata.target().contains("emg_state")
                && !metadata.target().contains("cassowary")
                && !metadata.target().contains("wgpu")
                && metadata.level() <= &tracing::Level::INFO
            // && !metadata.target().contains("winit event")
            // && !metadata.fields().field("event").map(|x|x.to_string())
            // && !metadata.target().contains("winit event: DeviceEvent")
        }));

    // let layout_override_layer = tracing_tree::HierarchicalLayer::new(2)
    //     .with_indent_lines(true)
    //     .with_indent_amount(4)
    //     .with_targets(true)
    //     .with_filter(EnvFilter::new("[LayoutOverride]=debug"));

    tracing_subscriber::registry()
        // .with(layout_override_layer)
        .with(out_layer)
        .init();

    // tracing_subscriber::Registry::default().with(tracing_tree::HierarchicalLayer::new(2));
    color_eyre::install()
}

// pub fn main() -> emg_bind::Result {
#[instrument]
pub fn main() -> Result<(), Report> {
    // pub fn main() -> Result<(), Error> {
    // #[cfg(debug_assertions)]
    tracing_init()?;
    Counter::run(Settings::default()).wrap_err("saw a downstream error")
}

#[derive(Default)]
struct Counter {
    value: i32,
}

#[emg_msg]
#[derive(Debug, Copy, Clone, PartialEq)]
enum Message {
    Empty,
    IncrementPressed,
    DecrementPressed,
}

impl Sandbox for Counter {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::IncrementPressed => {
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
            Message::Empty => {
                info!("update ---- got Message::Empty");
            }
        }
    }

    fn tree_build(
        &self,
        // orders: impl Orders<Self::Message> + 'static,
    ) -> GTreeBuilderElement<Self::Message> {
        let n = Rc::new(Cell::new(100));
        let ww = use_state(w(px(100)));
        let ff = use_state(fill(hsl(150, 100, 100)));
        gtree! {
            @=debug_layer
            Layer [
                On:CLICK  ||{
                    let _span = debug_span!("LayoutOverride", "click cb")
                            .entered();
                    debug!(" on [debug_layer]----click cb ----");
                    info!(" on [debug_layer]----click cb ----");
                },
                @=a1 @E=[
                        origin_x(pc(0)),align_x(pc(0)),
                        w(px(100)),h(px(100)),
                        ff,
                        b_width(px(5)),
                        b_color(rgb(1,0,0))
                    ]
                Layer [
                    @=a2 @E=[
                        origin_x(px( 0)),align_x(px(200)),
                        origin_y(px(0)),align_y(px(200)),
                        w(px(20)),h(px(20)),
                        fill(rgba(1, 0.5, 0, 1))
                    ]
                    Layer [
                        On:CLICK  move||{
                            let _span = debug_span!("LayoutOverride", "click cb")
                            .entered();
                            debug!(" on [a2] ----click cb ----");
                            info!(" on [a2] ----click cb ----");
                            let nn =n.get()+4;
                            n.set(nn);
                            ww.set(w(px(nn)));
                            ff.set(fill(hsl(nn as f64/100.*360.%360., 50, 50)));


                        },
                    ],
                    @=a3 @E=[
                        origin_x(px( 0)),align_x(px(300)),
                        origin_y(px(0)),align_y(px(300)),
                        w(px(30)),h(px(30)),
                        fill(rgba(1, 1, 0, 1)),
                        b_width(px(1)),
                        b_color(rgb(1,0,0))
                    ]
                    Layer [],
                    @=a4 @E=[
                        origin_x(px( 0)),align_x(px(400)),
                        origin_y(px(0)),align_y(px(400)),
                        // ww,
                        w(px(40)),h(px(40)),
                        fill(rgba(1, 1, 0, 1)),
                        b_width(px(7)),
                        b_color(rgb(1,0,1))
                    ]
                    Layer []
                ]
            ]
        }
    }

    fn root_id(&self) -> &str {
        "debug_layer"
    }

    // #[instrument(skip(self, g))]
    // fn ctx(
    //         &self,
    //         g: &GraphType<Self::Message>,
    //     ) -> StateAnchor<runtime::PaintCtx<renderer::SceneCtx> > {
    //         let ctx =StateAnchor::constant( runtime::PaintCtx::<renderer::SceneCtx>::default());
    //         g.get_node_item_use_ix(&IdStr::new_inline("debug_layer"))
    //         .unwrap()
    //         .build_ctx_sa(&EPath::<IdStr>::new(vector![edge_index_no_source("debug_layer")]),&ctx)

    // }

    // #[instrument(skip(self, g), ret)]
    // fn view(&self, g: &GraphType<Self::Message>) -> GelType<Self::Message> {
    //     g.get_node_item_use_ix(&IdStr::new_inline("debug_layer"))
    //         .unwrap()
    //         .get_view_gelement_sa(&EPath::<IdStr>::new(vector![edge_index_no_source("debug_layer")]))
    //         .get()
    // }
}
