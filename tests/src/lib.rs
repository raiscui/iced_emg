#[cfg(test)]
mod test {

    use wasm_bindgen_test::wasm_bindgen_test;

    use emg_bind::GTreeBuilderFn;
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
        time::Duration,
    };

    use emg_animation::{interrupt, opacity, style, to};
    use emg_bind::{
        better_any::{impl_tid, tid, type_id, Tid, TidAble, TidExt},
        button, edge_index_no_source, emg_msg,
        event::Event,
        subscription, Application, Button, Checkbox, Command, Element, GTreeBuilderElement,
        GraphMethods, GraphType, GraphView, Orders, Subscription, Text, Tick,
    };
    use emg_core::{into_vector, vector};
    use emg_core::{parent, TypeCheck, TypeCheckObjectSafe};
    use emg_layout::{
        add_values::origin_x,
        anima,
        animation::{global_clock, AnimationE},
        styles::{pc, px, width, CssWidth},
        EmgEdgeItem,
    };
    use emg_state::{topo, CloneStateVar, Dict, StateAnchor};

    use emg_state::{use_state, StateVar};

    use iced::{Align, Column, Error, Settings};
    extern crate gtree;

    use gtree::gtree;
    use seed_styles::w;
    use tracing::{debug, debug_span, trace, warn};
    use tracing::{info, trace_span};
    use web_sys::Performance;

    #[emg_msg]
    #[derive(Debug, Copy, Clone, PartialEq)]
    enum Message {
        IncrementPressed,
        DecrementPressed,
    }
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    // #[test]
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
                    Layer [Checkbox::new(false,"abcd",|_|Message::IncrementPressed)=>[
                        Checkbox::new(false,"222",|_|Message::IncrementPressed)=>[
                            Text::new(format!("checkbox-text")),
                        ],
                        node_ref("temp")
                    ]],
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

                                an.interrupt(vector![
                                    to(into_vector![width(px(50))]),
                                    to(into_vector![width(pc(100))]),
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

        let vs = p.now();
        emg_graph.borrow().view("a");
        let ve = p.now() - vs;
        warn!("view 1:{}", ve);

        let vs = p.now();
        emg_graph.borrow().view("a");
        let ve = p.now() - vs;
        warn!("view 2:{}", ve);

        let mut tot = 0f64;

        an2.interrupt(vector![
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
            to(into_vector![width(px(50))]),
            to(into_vector![width(pc(10110))]),
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
