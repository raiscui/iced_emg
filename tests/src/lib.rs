/*
 * @Author: Rais
 * @Date: 2022-05-23 16:41:57
 * @LastEditTime: 2022-05-24 18:27:37
 * @LastEditors: Rais
 * @Description: 
 */
#[cfg(test)]
mod wasm_test {


    use wasm_bindgen::JsCast;

    use wasm_bindgen_test::wasm_bindgen_test;
    use emg_bind::{GTreeBuilderFn, futures, Bus};
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
        time::Duration,
    };

    use emg_animation::{interrupt_og, opacity_og, style_og, to_og, to};
    use emg_bind::{
        better_any::{impl_tid, tid, type_id, Tid, TidAble, TidExt},
        button, edge_index_no_source, emg_msg,
        event::Event,
        subscription, Application, Button, Checkbox, Command, Element, GTreeBuilderElement,
        GraphMethods, GraphType, GraphView, Orders, Subscription, Text, Tick,
    };
    use emg_core::{into_vector, vector, into_smvec};
    use emg_core::{parent, TypeCheck, TypeCheckObjectSafe};
    use emg_layout::{
        global_clock,
        add_values::origin_x,
        anima,
        animation::{ AnimationE},
        styles::{pc, px, width, CssWidth},
        EmgEdgeItem,
    };
    use emg_state::{topo, CloneStateVar, Dict, StateAnchor};

    use emg_state::{use_state, StateVar};

    use iced::{Align, Column, Error, Settings};
    extern crate gtree;

    use gtree::gtree;
    use seed_styles::{w, GlobalStyleSV};
    use tracing::{debug, debug_span, trace, warn};
    use tracing::{info, trace_span};
    use web_sys::Performance;

    use rustc_hash::FxHashMap ;
    

    use dodrio::{bumpalo::Bump,
        Attribute, CachedSet, ElementNode, Node, NodeKind, Render, RenderContext, TextNode, Vdom,
    };


    fn window() -> web_sys::Window {
        web_sys::window().expect("no global `window` exists")

    }
    
    fn document() -> web_sys::Document {
        window()
            .document()
            .expect("should have a document on window")
    }
    fn create_element(tag: &str) -> web_sys::Element {
        document()
            .create_element(tag)
            .expect("should create element OK")
    }

    pub struct RenderFn<F>(F)
    where
        F: for<'a> Fn(&mut RenderContext<'a>) -> Node<'a>;

    impl<'a, F> Render<'a> for RenderFn<F>
    where
        F: for<'b> Fn(&mut RenderContext<'b>) -> Node<'b>,
    {
        fn render(&self, cx: &mut RenderContext<'a>) -> Node<'a> {
            (self.0)(cx)
        }
    }
    fn render2text<R: for<'a> Render<'a>>( r: &R){
        let cached_set = &RefCell::new(CachedSet::default());
        let bump = &Bump::new();
        let templates = &mut FxHashMap::default();
        let cx = &mut RenderContext::new(bump, cached_set, templates);
        let node = r.render(cx);
        warn!("node = {:#?}", node);

    }


    fn assert_rendered<R: for<'a> Render<'a>>(container: &web_sys::Element, r: &R) {
    
        let cached_set = &RefCell::new(CachedSet::default());
        let bump = &Bump::new();
        let templates = &mut FxHashMap::default();
        let cx = &mut RenderContext::new(bump, cached_set, templates);
        let node = r.render(cx);
        let child = container
            .first_child()
            .expect("container does not have anything rendered into it?");
    
        let cached_set = cached_set.borrow();
        check_node(&cached_set, &child, &node);
    
        fn stringify_actual_node(n: &web_sys::Node) -> String {
            if let Some(el) = n.dyn_ref::<web_sys::Element>() {
                el.outer_html()
            } else {
                format!("#text({:?})", n.text_content())
            }
        }
    
        fn check_node(cached_set: &CachedSet, actual: &web_sys::Node, expected: &Node) {
            debug!("check_node:");
            debug!("    actual = {}", stringify_actual_node(&actual));
            debug!("    expected = {:#?}", expected);
            match expected.kind {
                NodeKind::Text(TextNode { text }) => {
                    assert_eq!(
                        actual.node_name().to_uppercase(),
                        "#TEXT",
                        "actual.node_name() == #TEXT"
                    );
                    assert_eq!(
                        actual.text_content().unwrap_or_default(),
                        text,
                        "actual.text_content() == expected.text()"
                    );
                }
                NodeKind::Element(&ElementNode {
                    tag_name,
                    attributes,
                    children,
                    namespace,
                    ..
                }) => {
                    assert_eq!(
                        actual.node_name().to_uppercase(),
                        tag_name.to_uppercase(),
                        "actual.node_name() == expected.tag_name()"
                    );
                    let actual = actual
                        .dyn_ref::<web_sys::Element>()
                        .expect("`actual` should be an `Element`");
                    check_attributes(actual.attributes(), attributes);
                    check_children(cached_set, actual.child_nodes(), children);
                    if let Some(namespace) = namespace {
                        assert_eq!(actual.namespace_uri(), Some(namespace.into()))
                    }
                }
                NodeKind::Cached(ref c) => {
                    warn!("=== cached node");
                    let (expected, _template) = cached_set.get(c.id);
                    check_node(cached_set, actual, &expected);
                }
            }
        }
    
        fn check_attributes(actual: web_sys::NamedNodeMap, expected: &[Attribute]) {

            let mut actual_attr_names = vec![];
            let mut actual_skips = 0;

            for n in 0..actual.length() {
                let a = actual.item(n).unwrap();
                if a.name() == "dodrio-a-click" || a.name() == "dodrio-b-click" {
                    actual_skips +=1;

                }else{
                actual_attr_names.push((a.name().clone(),a.value().clone()));

                }
            }


            assert_eq!(
                actual.length()-actual_skips,
                expected.len() as u32,
                "actual's number of attributes == expected's number of attributes, \n actual: {:?} \n expected: {:?}",&actual_attr_names,&expected
            );
            for attr in expected {
                let actual_attr = actual
                    .get_named_item(attr.name())
                    .expect(&format!("should have attribute \"{}\"", attr.name()));
                if attr.name() != "dodrio-a-click" && attr.name() != "dodrio-b-click" {
                    assert_eq!(
                        actual_attr.value(),
                        attr.value(),
                        "actual attr value == expected attr value for attr \"{}\"",
                        attr.name()
                    );
                }
            }
        }
    
        fn check_children(cached_set: &CachedSet, actual: web_sys::NodeList, expected: &[Node]) {
            assert_eq!(
                actual.length(),
                expected.len() as u32,
                "actual children length == expected children length"
            );
            for (i, child) in expected.iter().enumerate() {
                let actual_child = actual.item(i as u32).unwrap();
                check_node(cached_set, &actual_child, child);
            }
        }
    }




    #[emg_msg]
    #[derive(Debug, Copy, Clone, PartialEq)]
    enum Message {
        IncrementPressed,
        DecrementPressed,
    }
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    use easybench_wasm::{bench, bench_env,bench_limit};

    fn view()->Element<Message> {
        console_error_panic_hook::set_once();
        let emg_graph = Rc::new(RefCell::new(GraphType::<Message>::default()));
        let an: AnimationE<Message> = anima![width(px(80))];
        let a = use_state(9999);

        let root: GTreeBuilderElement<Message> = gtree! {
            @=a
            Layer [
                 @=b @E=[w(w(pc(50))),h(pc(50)),origin_x(pc(50)),align_x(pc(50))]
                 Layer [
                    @=c @E=[w(px(150)),h(px(50)),origin_x(pc(50)),origin_y(pc(50)),align_x(pc(50)),align_y(pc(50))]
                    Layer [
                        node_ref("b"),

                        Checkbox::new(false,"abcd",|_|Message::IncrementPressed)=>[
                            Checkbox::new(false,"222",|_|Message::IncrementPressed)=>[
                                Text::new(format!("checkbox-text")),
                            ],
                        ]
                    ],
                    @=temp @E=[w(px(150)),h(px(150)),origin_x(pc(50)),origin_y(pc(0)),align_x(pc(50)),align_y(pc(50))]
                    Text::new(format!("temp----------")),

                    Layer [RefreshUse GElement::from( Text::new(format!("ee up")))],

                    @=an @E=[w(px(150)),origin_x(pc(50)),origin_y(pc(0)),align_x(pc(50)),align_y(pc(100))]
                    Text::new(format!("in quote.. {}", "b")) => [
                        RefreshUse ||{GElement::from( Text::new(format!("ee up")))},

                    ],

                    @E=[w(px(150)),origin_x(pc(100)),align_x(pc(100))]
                    Text::new(format!("in quote.. {}", "b")) => [
                        RefreshUse ||{100},
                    ],
                    @E=[w(px(150)),origin_x(pc(0)),align_x(pc(0))]
                    Text::new(format!("dt.. {}", "b")) => [
                    ],
                    @E=[w(px(250)),origin_x(pc(0)),align_y(pc(140))]
                    Text::new(format!("dt.. {}", "b")) => [
                    ],
                    @=e @E=[w(pc(100)),h(px(40)),css(background_color("red")),origin_x(pc(50)),align_y(pc(70))]
                    Layer [
                        @=eb @E=[w(px(150)),h(px(30)),origin_x(pc(60)),align_y(pc(250))]
                        Button::new(Text::new(format!("2 button in quote..{}", "e"))) => [
                            On:click move||{

                                trace!("bbbbbbbbbbbbb");

                                a.set_with(|v|v+1);
                                Option::<Message>::None

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
                        @=b2 @E=[an.clone(),h(parent!(CssWidth)+px(30)),origin_x(pc(60)),align_y(pc(300))]
                        Button::new(Text::new(format!("2 button in quote..{}", "e"))) => [
                            On:click move |_root, vdom, _event| {

                                an.interrupt([
                                    to![width(px(50))],
                                    to![width(pc(100))],
                                ]);

                                            a.set(a.get()+1);

                                            debug!("will render");

                                        Option::<Message>::None
                                }
                        ]
                    ],
                ]
            ]
        };

        // ─────────────────────────────────────────────────────────────────

        emg_graph.handle_root_in_topo(&root);
        let root_elm = emg_graph.borrow().view("a");
        root_elm

        // let _vdom = Vdom::new(&container, root_elm_render_fn.clone());
    }

    #[wasm_bindgen_test]
    fn benchmark(){
        use web_sys::console;

        console::log_1(
            &format!(
                "view: {}",
                bench_limit(10.,|| {
                    let _f = view();
                })
            )
            .into(),
        );

    }

    #[wasm_bindgen_test]
    fn test2() {
        
        console_error_panic_hook::set_once();
        // ─────────────────────────────────────────────────────────────────

        let mut config = tracing_wasm::WASMLayerConfigBuilder::default();
        config.set_max_level(tracing::Level::WARN);
        config.set_console_config(tracing_wasm::ConsoleConfig::ReportWithConsoleColor);
        // config.set_console_config(tracing_wasm::ConsoleConfig::NoReporting);

        tracing_wasm::set_as_global_default_with_config(config.build());
        // ────────────────────────────────────────────────────────────────────────────────
        let (sender, _receiver) = futures::channel::mpsc::unbounded();
        let bus = Bus::new(sender);
        let css = GlobalStyleSV::default_topo();

       
        let root_elm = view();
        let root_elm_render_fn = Rc::new(RenderFn(move |cx|root_elm.node(&cx.bump,&bus,&css)));
        // let _vdom = Vdom::new(&container, root_elm_render_fn.clone());
        render2text(&root_elm_render_fn);


    }


    #[wasm_bindgen_test]
    fn test1() {
        console_error_panic_hook::set_once();
        #[cfg(debug_assertions)]
        {
            let mut config = tracing_wasm::WASMLayerConfigBuilder::default();
            config.set_max_level(tracing::Level::DEBUG);
            config.set_console_config(tracing_wasm::ConsoleConfig::ReportWithConsoleColor);
            // config.set_console_config(tracing_wasm::ConsoleConfig::NoReporting);

            tracing_wasm::set_as_global_default_with_config(config.build());
        }
        #[cfg(not(debug_assertions))]
        {
            let mut config = tracing_wasm::WASMLayerConfigBuilder::default();
            config.set_max_level(tracing::Level::WARN);
            config.set_console_config(tracing_wasm::ConsoleConfig::ReportWithConsoleColor);
            // config.set_console_config(tracing_wasm::ConsoleConfig::NoReporting);

            tracing_wasm::set_as_global_default_with_config(config.build());
        }

        let emg_graph =Rc::new(RefCell::new( GraphType::<Message>::default()));
        let an: AnimationE<Message> = anima![width(px(80))];
        let an2 = an.clone();
        let a = use_state(9999);        
        
        let p = web_sys::window().unwrap().performance().unwrap();

        let treetime = p.now();

        let root: GTreeBuilderElement< Message> = 
        
        // gtree! {
        //     @=a
        //     Layer [
        //         Button::new(Text::new(format!("2 button in quote..{}", "e"))) => []
        //         ]
        // };
        // gtree! {
        //     @=a
        //     Layer [
        //             Checkbox::new(false,"abcd",|_|Message::IncrementPressed)=>[
        //                 Text::new(format!("2 button in quote..{}", "e"))
        //             ]
        //         ]
        // };
         gtree! {
            @=a
            Layer [
                 @=b @E=[w(w(pc(50))),h(pc(50)),origin_x(pc(50)),align_x(pc(50))]
                 Layer [
                    @=c @E=[w(px(150)),h(px(50)),origin_x(pc(50)),origin_y(pc(50)),align_x(pc(50)),align_y(pc(50))]
                    Layer [
                        node_ref("b"),

                        Checkbox::new(false,"abcd",|_|Message::IncrementPressed)=>[
                            Checkbox::new(false,"222",|_|Message::IncrementPressed)=>[
                                Text::new(format!("checkbox-text")),
                            ],
                        ]
                    ],
                    @=temp @E=[w(px(150)),h(px(150)),origin_x(pc(50)),origin_y(pc(0)),align_x(pc(50)),align_y(pc(50))]
                    Text::new(format!("temp----------")),

                    Layer [RefreshUse GElement::from( Text::new(format!("ee up")))],

                    @=an @E=[w(px(150)),origin_x(pc(50)),origin_y(pc(0)),align_x(pc(50)),align_y(pc(100))]
                    Text::new(format!("in quote.. {}", "b")) => [
                        // xxx,
                        RefreshUse ||{GElement::from( Text::new(format!("ee up")))},
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
                    @E=[w(px(150)),origin_x(pc(100)),align_x(pc(100))]
                    Text::new(format!("in quote.. {}", "b")) => [
                        RefreshUse ||{100},
                        // RefreshUse  move||{that.borrow().ddd}
                    ],
                    @E=[w(px(150)),origin_x(pc(0)),align_x(pc(0))]
                    Text::new(format!("dt.. {}", "b")) => [
                        // RefreshUse  move||{that.borrow().ddd}
                    ],
                    @E=[w(px(250)),origin_x(pc(0)),align_y(pc(140))]
                    Text::new(format!("dt.. {}", "b")) => [
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
                                Option::<Message>::None

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

                                            debug!("will render");
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
                                        Option::<Message>::None
                                            }
                        ]
                    ],
                ]
            ]
        };

        let treebuildtime = p.now() - treetime;
        warn!("build GTreeBuilderElement:{}", treebuildtime);

        let handle_root_in_topo_start = p.now();
        emg_graph.handle_root_in_topo(&root);
        let handle_root_in_topo_time = p.now() - handle_root_in_topo_start;
        warn!("emg_graph.handle_root_in_topo:{}", handle_root_in_topo_time);
// ────────────────────────────────────────────────────────────────────────────────


        let (sender, _receiver) = futures::channel::mpsc::unbounded();
        let css = GlobalStyleSV::default_topo();
        let bus = Bus::new(sender);
        let container = create_element("div");

// ────────────────────────────────────────────────────────────────────────────────

        let vs = p.now();

        let root_elm =emg_graph.borrow()
        .view("a");
        
        let root_elm_render_fn = Rc::new(RenderFn(move |cx|root_elm.node(&cx.bump,&bus,&css)));

        let _vdom = Vdom::new(&container, root_elm_render_fn.clone());

        let ve = p.now() - vs;
        warn!("view 1:{}", ve);
        assert_rendered(&container, &root_elm_render_fn);


        let vs = p.now();
        emg_graph.borrow().view("a");
        let ve = p.now() - vs;
        warn!("view 2:{}", ve);

        let mut tot = 0f64;

        an2.interrupt([
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
            to![width(px(50))],
            to![width(pc(10110))],
        ]);

        let t1 = p.now();


        for i in 0..10000 {

            emg_graph.borrow().view("a");

       
        }
        let t2 = p.now();

        tot += t2 - t1;

        warn!("tut:{}", tot);//990

        warn!("dt:{}", tot / 10000.);
    }
}

