use emg_bind::{
    better_any::TidAble,
    common::{vector, IdStr},
    element::*,
    emg::edge_index_no_source,
    emg_msg, gtree,
    layout::EPath,
    state::CloneStateAnchor,
    Sandbox, Settings,
};
use tracing::instrument;
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
                && !metadata.target().contains("winit event")
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
        }
    }

    fn tree_build(
        &self,
        // orders: impl Orders<Self::Message> + 'static,
    ) -> GTreeBuilderElement<Self::Message> {
        gtree! {
            @=debug_layer
            Layer [
                @=a1 @E=[
                        origin_x(pc(50)),align_x(pc(50)),
                        w(pc(50)),h(pc(50)),
                    ]
                Layer [
                    @=a2 @E=[
                        w(pc(50)),h(pc(50)),
                    ]
                    Layer []
                ]
            ]
        }
    }

    #[instrument(skip(self, g), ret)]
    fn view(&self, g: &GraphType<Self::Message>) -> GelType<Self::Message> {
        g.get_node_item_use_ix(&IdStr::new_inline("debug_layer"))
            .unwrap()
            .get_view_gelement_sa(&EPath::<IdStr>::new(vector![edge_index_no_source("debug_layer")]))
            .get()
    }
}
