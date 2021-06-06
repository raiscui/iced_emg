/*
 * @Author: Rais
 * @Date: 2021-05-28 11:50:10
 * @LastEditTime: 2021-06-06 09:59:41
 * @LastEditors: Rais
 * @Description:
 */
/*
 * @Author: Rais
 * @Date: 2021-05-27 12:42:24
 * @LastEditTime: 2021-05-28 11:37:46
 * @LastEditors: Rais
 * @Description:
 */
// mod define;
// mod func;
use std::{cell::RefCell, rc::Rc, time::Duration};

use emg_state::{
    state_store, topo, use_state, CloneStateAnchor, CloneStateVar, GStateStore, StateAnchor,
    StateMultiAnchor, StateVar,
};
use im::{vector, Vector};

use emg_animation::{
    extract_initial_wait,
    models::{map_to_motion, resolve_steps, Motion, Property, Step, StepTimeVector},
    props::warn_for_double_listed_properties,
    set_default_interpolation, Timing,
};

use crate::{DictPathEiNodeSA, EmgEdgeItem, Layout};

// ────────────────────────────────────────────────────────────────────────────────
type SAPropsMessageSteps<Message> =
    StateAnchor<(Vector<Property>, Vector<Message>, Vector<Step<Message>>)>;
// ────────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Copy, Clone)]
struct AnimationInside<Message>
where
    Message: Clone + std::fmt::Debug + 'static,
{
    pub(crate) steps: StateVar<Vector<Step<Message>>>,
    pub(crate) interruption: StateVar<StepTimeVector<Message>>,
    pub(crate) props: StateVar<Vector<Property>>,
}

impl<Message> AnimationInside<Message>
where
    Message: Clone + std::fmt::Debug + 'static,
{
    #[topo::nested]
    fn new_in_topo(props: Vector<Property>) -> Self {
        Self {
            steps: use_state(vector![]),
            interruption: use_state(vector![]),
            props: use_state(props.into_iter().map(set_default_interpolation).collect()),
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct AnimationEdge<Ix, Message>
where
    Message: Clone + std::fmt::Debug + 'static + PartialEq,
    Ix: Clone + std::hash::Hash + Eq + Default + Ord + 'static,
{
    inside: AnimationInside<Message>,
    timing: StateAnchor<Timing>,
    pub(crate) running: StateAnchor<bool>,
    store: Rc<RefCell<GStateStore>>,
    edge_nodes: DictPathEiNodeSA<Ix>,
    layout: Layout,
    queued_interruptions: StateAnchor<StepTimeVector<Message>>,
    revised_steps: StateAnchor<Vector<Step<Message>>>,
    revised_props: StateAnchor<Vector<Property>>,
    send_messages: StateAnchor<Vector<Message>>,
    // timing_ob: StateAnchor<()>,
    // processed_interruptions: StateAnchor<(StepTimeVector<Message>, StepTimeVector<Message>)>,
    // revised: SAPropsMessageSteps<Message>,
}

impl<Ix, Message> std::fmt::Debug for AnimationEdge<Ix, Message>
where
    Message: Clone + std::fmt::Debug + 'static + PartialEq,
    Ix: Clone + std::hash::Hash + Eq + Default + Ord + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnimationEdge")
            .field("inside", &self.inside)
            .field("timing", &self.timing)
            .field("running", &self.running)
            .field("queued_interruptions", &self.queued_interruptions)
            .field("revised_steps", &self.revised_steps)
            .field("revised_props", &self.revised_props)
            .field("send_messages", &self.send_messages)
            .finish()
    }
}

impl<Ix, Message> AnimationEdge<Ix, Message>
where
    Message: Clone + std::fmt::Debug + 'static + PartialEq,
    Ix: Clone + std::hash::Hash + Eq + Default + Ord + 'static,
{
    fn set_timer(sv_now: StateVar<Duration>) -> StateAnchor<Timing> {
        // let mut timing = Timing::default();
        sv_now
            .watch()
            // .map(move |now:&Duration|{
            //     if timing.current() != *now{
            //         let dt_tmp = now.saturating_sub(timing.current());
            //         let dt = {
            //             if timing.current() == Duration::ZERO || dt_tmp.as_millis() > 34 {
            //                 Duration::from_micros(16666)
            //             } else {
            //                 dt_tmp
            //             }
            //         };
            //         timing.set_current(*now);
            //         timing.set_dt(dt);
            //     }
            //     timing
            // })
            .map_mut(
                Timing::default(),
                move |timing: &mut Timing, now: &Duration| {
                    let current = timing.current_mut();
                    if now == current {
                        println!("same time");
                        return false;
                    }
                    // • • • • •

                    let dt_tmp = now.saturating_sub(*current);
                    let dt = {
                        if *current == Duration::ZERO || dt_tmp.as_millis() > 34 {
                            Duration::from_micros(16666)
                        } else {
                            dt_tmp
                        }
                    };
                    // *current = *now;
                    timing.set_dt(dt);
                    timing.set_current(*now);
                    // println!("new time:{:?}", &timing);
                    true
                },
            )
    }

    #[topo::nested]
    pub fn new_in_topo(
        props: Vector<Property>,
        edge: EmgEdgeItem<Ix>,
        sv_now: StateVar<Duration>,
    ) -> Self
    where
        Message: Clone + std::fmt::Debug + 'static + PartialEq,
        Ix: Clone + std::hash::Hash + Eq + Default + Ord + 'static,
    {
        warn_for_double_listed_properties(&props);

        let sv_inside: AnimationInside<Message> = AnimationInside::<Message>::new_in_topo(props);
        let sa_timing = Self::set_timer(sv_now);
        // let sa_timing_real = Self::set_timer(sv_now);
        // let sa_timing = {
        //     let mut saved_current_time = sv_now.get();

        //     sa_timing_real.cutoff(move |timing: &Timing| {
        //         let current = timing.current();
        //         if current == saved_current_time {
        //             false
        //         } else {
        //             saved_current_time = current;
        //             true
        //         }
        //     })
        // };

        let AnimationInside {
            interruption: interruption_og,
            steps: steps_og,
            props: props_og,
        } = sv_inside;

        let interruption_cut = {
            let mut ct = sv_now.get();
            (&sa_timing, &interruption_og.watch())
                .map(|t, i| (*t, i.clone()))
                .cutoff(move |(timing, _)| {
                    let current = timing.current();

                    if current == ct {
                        false
                    } else {
                        ct = current;
                        true
                    }
                })
                .map(|(_, i)| i.clone())
        };
        let steps_cut = {
            let mut ct = sv_now.get();
            (&sa_timing, &steps_og.watch())
                .map(|t, i| (*t, i.clone()))
                .cutoff(move |(timing, _)| {
                    let current = timing.current();

                    if current == ct {
                        false
                    } else {
                        ct = current;
                        true
                    }
                })
                .map(|(_, i)| i.clone())
        };
        let props_cut = {
            let mut ct = sv_now.get();
            (&sa_timing, &props_og.watch())
                .map(|t, i| (*t, i.clone()))
                .cutoff(move |(timing, _)| {
                    let current = timing.current();
                    if current == ct {
                        false
                    } else {
                        ct = current;
                        true
                    }
                })
                .map(|(_, i)| i.clone())
        };

        // let mut current_time = Duration::default();

        let sa_processed_interruptions: StateAnchor<(
            StepTimeVector<Message>,
            StepTimeVector<Message>,
        )> =
        //  sa_timing.then(move |timing: &Timing| {
            // println!("in interruption: timing:{:?}", &timing);
            // let dt = timing.dt();

            (&sa_timing,& interruption_cut)
                // .watch()
                .map(move |timing,interruption: &StepTimeVector<Message>| {
                    println!("interruption_w.map:-> len:{}", interruption.len());
                    interruption
                        .clone()
                        .into_iter()
                        .map(|(wait, steps)| {
                            // println!("wait: {:?} , dt: {:?}", &wait, &dt);
                            (wait.saturating_sub(timing.dt()), steps)
                        })
                        .partition(|(wait, _)| wait.is_zero())
                });
        // .into()
        // });
        let sa_queued_interruptions: StateAnchor<StepTimeVector<Message>> =
            sa_processed_interruptions.map(|v| v.1.clone());

        let sa_steps_props: StateAnchor<(Vector<Step<Message>>, Vector<Property>)> =
            (&sa_processed_interruptions, &steps_cut, &props_cut).map(
                |processed_interruptions: &(StepTimeVector<Message>, StepTimeVector<Message>),
                 steps: &Vector<Step<Message>>,
                 props: &Vector<Property>| {
                    match processed_interruptions.0.head() {
                        Some((_ /* is zero */, interrupt_steps)) => {
                            println!("get ready step:---> \n{:?}", &interrupt_steps);

                            (
                                interrupt_steps.clone(),
                                props
                                    .clone()
                                    .into_iter()
                                    .map(|prop| {
                                        map_to_motion(
                                            Rc::new(|mut m: Motion| {
                                                *m.interpolation_override_mut() = None;
                                                m
                                            })
                                            .as_ref(),
                                            prop,
                                        )
                                    })
                                    .collect::<Vector<_>>(),
                            )
                        }
                        None => (steps.clone(), props.clone()),
                    }
                },
            );
        // let sa_steps_props_clone = sa_steps_props.clone();
        let revised: SAPropsMessageSteps<Message> = 
        // sa_timing.then(move |timing: &Timing| {
            // let dt = timing.dt();
            (&sa_timing,& sa_steps_props)
                // (&sa_timing,&sa_steps_props)
                .map(
                    move |timing,(steps, props): &(Vector<Step<Message>>, Vector<Property>)| {
                        let (revised_props, sent_messages, revised_steps) =
                            resolve_steps(props.clone(), steps.clone(), timing.dt());
                        (revised_props, sent_messages, revised_steps)
                    },
                )
                ;
                // .into()
        // });

        let sa_revised_props = revised.map(|x| x.0.clone());
        let sa_revised_steps = revised.map(|x| x.2.clone());
        let sa_message = revised.map(|x| x.1.clone());
        let sa_running = (&sa_queued_interruptions, &sa_revised_steps)
            .map(|q, r| !q.is_empty() || !r.is_empty());

            // ─────────────────────────────────────────────────────────────────

        // let sa_queued_interruptions_clone = sa_queued_interruptions.clone();
        // let sa_revised_steps_clone = sa_revised_steps.clone();
        // let sa_revised_props_clone = sa_revised_props.clone();
        // let mut updated_time = Duration::default();
        // ────────────────────────────────────────────────────────────────────────────────

        // let timing_ob =
        // // sa_timing.then(move |_| {
        //     (
        //         // &sa_queued_interruptions_clone,
        //         // &sa_revised_steps_clone,
        //         // &sa_revised_props_clone,
        //         &sa_queued_interruptions,
        //         &sa_revised_steps,
        //         &sa_revised_props,
        //     )
        //         .map(move |queued_interruptions, revised_steps, revised_props| {
        //             println!("timing change!");

        //             interruption_og.set(queued_interruptions.clone());
        //             steps_og.set(revised_steps.clone());
        //             props_og.set(revised_props.clone());
        //         })
        //         ;
        // // .into()
        // // });
        // // ─────────────────────────────────────────────────────────────────

        // state_store()
        //     .borrow()
        //     .engine_mut()
        //     .mark_observed(timing_ob.anchor());
        // ─────────────────────────────────────────────────────────────────
        Self {
            inside: sv_inside,
            timing: sa_timing,
            running: sa_running,
            store: state_store(),
            edge_nodes: edge.node.clone(), //TODO: 如果是 针对一个特别 Path的动画,那么需要 输入 特别路径Path
            layout: edge.layout,
            queued_interruptions: sa_queued_interruptions,
            revised_props: sa_revised_props,
            revised_steps: sa_revised_steps,
            send_messages: sa_message,
            // timing_ob,
            // processed_interruptions: sa_processed_interruptions,
            // revised,
        }
    }

    pub fn interrupt(&mut self, steps: Vector<Step<Message>>) -> &mut Self {
        //TODO use store
        self.inside.interruption.set_with_once(|interruption| {
            let mut new_interruption = interruption.clone();
            let xx = extract_initial_wait(steps);
            new_interruption.push_front(xx);
            new_interruption
        });

        self
    }

    pub fn update_animation(&self) {
        //
        // self.inside.props.get();
        // self.store.borrow().engine_mut().stabilize();
        let queued_interruptions = self.queued_interruptions.get();
        let revised_steps = self.revised_steps.get();
        let revised_props = self.revised_props.get();
        self.inside.interruption.set(queued_interruptions);
        self.inside.steps.set(revised_steps);
        self.inside.props.set(revised_props);
        //TODO: cmd send message
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use emg::edge_index_no_source;
    use emg_animation::{opacity, to};
    use emg_state::{use_state, CloneStateAnchor, CloneStateVar, Dict, StateAnchor};
    use im::vector;

    use crate::EmgEdgeItem;

    use super::AnimationEdge;
    use tracing::{info, span, trace, trace_span, Level};

    use tracing_flame::FlameLayer;
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    fn _init() {
        // let _el = env_logger::try_init();

        let _subscriber = tracing_subscriber::fmt()
            .with_test_writer()
            // .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_span_events(
                tracing_subscriber::fmt::format::FmtSpan::ACTIVE
                    | tracing_subscriber::fmt::format::FmtSpan::ENTER
                    | tracing_subscriber::fmt::format::FmtSpan::CLOSE,
            )
            .with_max_level(Level::TRACE)
            .try_init();

        // tracing::subscriber::set_global_default(subscriber)
        // .expect("setting default subscriber failed");
    }

    fn setup_global_subscriber() -> impl Drop {
        std::env::set_var("RUST_LOG", "trace");
        std::env::set_var("RUST_LOG", "warn");

        // let _el = env_logger::try_init();

        let filter_layer = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("trace"))
            .unwrap();

        let fmt_layer = fmt::Layer::default()
            .with_target(false)
            .with_test_writer()
            .with_span_events(
                tracing_subscriber::fmt::format::FmtSpan::ENTER
                    // |tracing_subscriber::fmt::format::FmtSpan::FULL
                    | tracing_subscriber::fmt::format::FmtSpan::CLOSE,
            );

        let (flame_layer, _guard) = FlameLayer::with_file("./tracing/tracing.folded").unwrap();

        let _s = tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .with(flame_layer)
            .try_init();
        _guard
    }

    #[derive(Debug, Clone, PartialEq)]
    enum Message {
        A,
    }

    #[test]
    fn test_animation_edge() {
        // let nn = _init();
        {
            let span = trace_span!("am-test");
            let _guard = span.enter();
            trace!("fff");

            let ei = edge_index_no_source("fff");
            let source = use_state(ei.source_nix().as_ref().cloned());
            let target = use_state(ei.target_nix().as_ref().cloned());
            let edge_item: EmgEdgeItem<String> = EmgEdgeItem::default_with_wh_in_topo(
                source.watch(),
                target.watch(),
                StateAnchor::constant(Dict::default()),
                1920,
                1080,
            );

            let sv_now = use_state(Duration::ZERO);
            let mut a: AnimationEdge<String, Message> =
                AnimationEdge::new_in_topo(vector![opacity(1.)], edge_item, sv_now);
            // println!("a:{:#?}", &a);
            insta::assert_debug_snapshot!("new", &a);
            insta::assert_debug_snapshot!("new2", &a);
            assert_eq!(a.running.get(), false);
            insta::assert_debug_snapshot!("get_running", &a);
            println!("now set interrupt");
            a.interrupt(vector![
                to(vector![emg_animation::opacity(0.)]),
                to(vector![emg_animation::opacity(1.)])
            ]);
            println!("over interrupt");

            insta::assert_debug_snapshot!("interrupt", &a);
            insta::assert_debug_snapshot!("interrupt2", &a);
            println!("over interrupt insta");

            assert_eq!(a.running.get(), true);
            println!("over interrupt running.get()");
            // a.update_animation();
            // ────────────────────────────────────────────────────────────────────────────────

            sv_now.set(Duration::from_millis(16));
            println!("set timing 16");
            insta::assert_debug_snapshot!("set16", &a);

            // insta::assert_debug_snapshot!("time_16_0", &a);
            // insta::assert_debug_snapshot!("time_16_1", &a);

            a.update_animation();
            println!("set timing 16-- update");

            // println!("1**{:?}", a.inside.props.get());

            insta::assert_debug_snapshot!("updated_16_0", &a);
            insta::assert_debug_snapshot!("updated_16_1", &a);
            println!("set timing 16-- insta");
            // ────────────────────────────────────────────────────────────────────────────────
            sv_now.set(Duration::from_millis(16));
            println!("set timing 16-2");

            a.update_animation();

            insta::assert_debug_snapshot!("updated_16_0-2", &a);
            insta::assert_debug_snapshot!("updated_16_1-2", &a);
            println!("set timing 16-- insta-2");
            // ─────────────────────────────────────────────────────────────────

            sv_now.set(Duration::from_millis(33));
            println!("set timing 33");

            // println!("....set 2 ");
            insta::assert_debug_snapshot!("set33", &a);

            a.update_animation();
            insta::assert_debug_snapshot!("updated_33_0", &a);

            println!("set timing 33 -- update 1");

            a.update_animation();
            insta::assert_debug_snapshot!("updated_33_1", &a);

            println!("set timing 33 -- update 2");

            // println!("2**{:?}", a.inside.props.get());

            insta::assert_debug_snapshot!("snap_updated_33_0", &a);
            insta::assert_debug_snapshot!("snap_updated_33_1", &a);
            println!("set timing 33 -- insta  ");

            // sv_now.set(Duration::from_millis(2));
            // a.update_animation();
            // insta::assert_debug_snapshot!("updated_back_0", &a);
            // insta::assert_debug_snapshot!("updated_back_1", &a);

            for i in 3..100 {
                sv_now.set(Duration::from_millis(i * 16));
                println!("in ------ i:{}", &i);
                // a.timing.get();
                a.update_animation();
                println!("3***{:?}", a.inside.props.get());
            }
            insta::assert_debug_snapshot!("updated_end_0", &a);
            insta::assert_debug_snapshot!("updated_end_1", &a);

            // // println!("{:?}", a.revised_props.get());
            // // state_store().borrow().engine_mut().stabilize();
            println!("end : {:?}", a.inside.props.get());
            println!("{:?}", sv_now);
        }
    }
}
