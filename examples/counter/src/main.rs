use std::{rc::Rc, cell::Cell};

use emg_bind::{
    better_any::TidAble,
    common::px,
    element::*,
    emg_msg, gtree,
    layout::{ styles::{w, fill, hsl}},
    state::{ use_state},
    Sandbox, Settings
};

use tracing::{instrument, info};
fn tracing_init() {
    use tracing_subscriber::prelude::*;

    let out_layer = 
    // tracing_subscriber::fmt::layer()
    tracing_tree::HierarchicalLayer::new(2) .with_indent_lines(true)
    // .with_targets(true)
        .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
            // println!("metadata: {:?}",&metadata.fields().field("event").map(|x|x.to_string()));
            !metadata.target().contains("emg_layout")
                && !metadata.target().contains("anchors")
                && !metadata.target().contains("emg_state")
                && !metadata.target().contains("cassowary")
                // && !metadata.target().contains("winit event")
                // && !metadata.fields().field("event").map(|x|x.to_string())
                // && !metadata.target().contains("winit event: DeviceEvent")

        }))
        .with_filter(tracing_subscriber::filter::LevelFilter::DEBUG);

    // let def_subscriber = tracing_subscriber::fmt()
    //     // .with_test_writer()
    //     // .with_max_level(tracing::Level::TRACE)
    //     // .with_env_filter("emg_layout=error")
    //     .with_span_events(
    //         tracing_subscriber::fmt::format::FmtSpan::ACTIVE, //         | tracing_subscriber::fmt::format::FmtSpan::ENTER
    //                                                           //         | tracing_subscriber::fmt::format::FmtSpan::CLOSE,
    //     )
    //     .without_time()
    //     .finish();

    tracing_subscriber::registry()
        .with( out_layer )
        .init();

    // tracing_subscriber::Registry::default().with(tracing_tree::HierarchicalLayer::new(2));
}

pub fn main() -> emg_bind::Result {
    #[cfg(debug_assertions)]
    tracing_init();
    
    Counter::run(Settings::default())
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
            },
        }
    }

    fn tree_build(
        &self,
        // orders: impl Orders<Self::Message> + 'static,
    ) -> GTreeBuilderElement<Self::Message> {
        let  n = Rc::new(Cell::new(100));
        let ww = use_state(w(px(100)));
        let ff  = use_state(fill(hsl(150, 100, 100)) );
        gtree! {
            @=debug_layer
            Layer [
                On:click  ||{
                    info!(" on [debug_layer]----click cb ----");
                },
                @=a1 @E=[
                        origin_x(pc(50)),align_x(pc(50)),
                        w(pc(50)),h(pc(50)),
                        ff,
                        b_width(px(5)),
                        b_color(rgb(1,0,0))
                    ]
                Layer [
                    @=a2 @E=[
                        origin_x(pc( 10)),align_x(pc(100)),
                        w(px(100)),h(px(100)),
                        fill(rgba(1, 0.5, 0, 1))
                    ]
                    Layer [
                        On:click  move||{
                            info!(" on [a2] ----click cb ----");
                            // let nn =n.get()+10;
                            // n.set(nn);
                            // ww.set(w(px(nn)));
                            // ff.set(fill(hsl(nn/100*360%360, 100, 100)));


                        },
                    ],
                    @=a3 @E=[
                        origin_x(pc( 10)),align_x(pc(100)),
                        origin_y(px(-50)),
                        w(px(100)),h(px(100)),
                        fill(rgba(1, 1, 0, 1)),
                        b_width(px(1)),
                        b_color(rgb(1,0,0))
                    ]
                    Layer [],
                    @=a4 @E=[
                        origin_x(pc( 10)),align_x(pc(100)),
                        origin_y(px(-60)),
                        ww,h(px(100)),
                        fill(rgba(1, 1, 0, 1)),
                        b_width(px(7)),
                        b_color(rgb(1,0,1))
                    ]
                    Layer []
                ]
            ]
        }




    }

    fn root_id(&self)->&str {
        "debug_layer"
    }

    // #[instrument(skip(self, g))]
    // fn ctx(
    //         &self,
    //         g: &GraphType<Self::Message>,
    //     ) -> StateAnchor<runtime::PaintCtx<renderer::RenderCtx> > {
    //         let ctx =StateAnchor::constant( runtime::PaintCtx::<renderer::RenderCtx>::default());
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
