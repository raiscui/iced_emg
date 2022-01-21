/*
 * @Author: Rais
 * @Date: 2021-05-28 11:50:10
 * @LastEditTime: 2022-01-20 18:46:04
 * @LastEditors: Rais
 * @Description:
 */

mod define;
mod func;

use std::{cell::Cell, rc::Rc, time::Duration};

use emg_core::{vector, Vector};
use emg_state::{
    topo, use_state,
    use_state_impl::{TopoKey, Var},
    Anchor, CloneStateAnchor, CloneStateVar, StateAnchor, StateMultiAnchor, StateVar,
};

use emg_animation::{
    extract_initial_wait,
    models::{map_to_motion, resolve_steps, Motion, Property, Step, StepTimeVector},
    set_default_interpolation, Timing,
};
use tracing::{debug, trace};

use crate::{EPath, EmgEdgeItem};

use self::{define::StateVarProperty, func::props::warn_for_double_listed_properties};

// ────────────────────────────────────────────────────────────────────────────────
#[allow(dead_code)]
type SAPropsMessageSteps<Message> =
    StateAnchor<(Vector<Property>, Vector<Message>, Vector<Step<Message>>)>;
#[allow(dead_code)]
type SAPropsMessageSteps2<Message> = StateAnchor<(
    StepTimeVector<Message>,
    Vector<Step<Message>>,
    Vector<Property>,
    Vector<Message>,
)>;
// ────────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct AnimationInside<Message>
where
    Message: Clone + std::fmt::Debug + 'static,
{
    pub(crate) steps: StateVar<Vector<Step<Message>>>,
    pub(crate) interruption: StateVar<StepTimeVector<Message>>,
    pub(crate) props: Vector<StateVarProperty>,
}

impl<Message> AnimationInside<Message>
where
    Message: Clone + std::fmt::Debug + 'static,
{
    #[topo::nested]
    fn new_in_topo(props: Vector<StateVarProperty>) -> Self {
        props
            .iter()
            .for_each(|prop| prop.set_with_once(|p| set_default_interpolation(p.clone())));
        Self {
            steps: use_state(vector![]),
            interruption: use_state(vector![]),
            props,
        }
    }
}
thread_local! {
    static G_CLOCK: StateVar<Duration> = use_state(Duration::ZERO);
}

thread_local! {
    static G_ANIMA_RUNNING_STORE: StateVar<Vector<Anchor<bool>>> = use_state(vector![]);
}
thread_local! {
    static G_AM_RUNING: StateAnchor<bool> = global_anima_running_build();
}
pub fn global_anima_running_add(running: &StateAnchor<bool>) {
    G_ANIMA_RUNNING_STORE.with(|sv| sv.update(|v| v.push_back(running.get_anchor())));
}

#[must_use]
pub fn global_anima_running_sa() -> StateAnchor<bool> {
    G_AM_RUNING.with(std::clone::Clone::clone)
}
#[must_use]
pub fn global_anima_running() -> bool {
    G_AM_RUNING.with(emg_state::CloneStateAnchor::get)
}
#[must_use]
pub fn global_anima_running_build() -> StateAnchor<bool> {
    let watch: Anchor<Vector<bool>> = G_ANIMA_RUNNING_STORE.with(|am| {
        am.watch().anchor().then(|v: &Vector<Anchor<bool>>| {
            v.clone().into_iter().collect::<Anchor<Vector<bool>>>()
        })
    });
    watch.map(|list: &Vector<bool>| list.contains(&true)).into()
}
#[must_use]
pub fn global_clock() -> StateVar<Duration> {
    G_CLOCK.with(|c| *c)
}
pub fn global_clock_set(now: Duration) {
    G_CLOCK.with(|c| c.set(now));
}

// ────────────────────────────────────────────────────────────────────────────────

//TODO for the path macro: [path =>[xx],path=>[xxx]]
#[macro_export]
macro_rules! anima {
    ( $( $element:expr ) , * ) => {
        {
            let mut v = emg_core::Vector::new();

            $(
                v.push_back($element.into());
            )*

            $crate::AnimationE::new_in_topo(v)
            // $crate::topo::call(||$crate::AnimationE::new_in_topo(v))
        }
    };
}
// // @ use for refresh_for  表示 要关联到 单一路径节点 ────────────────────────────────────────────────────────────────────────────────

// pub struct AnimaForPath<Message>(pub AnimationE<Message>)
// where
//     Message: Clone + std::fmt::Debug + 'static + PartialEq;

// impl<Message> std::ops::Deref for AnimaForPath<Message>
// where
//     Message: Clone + std::fmt::Debug + 'static + PartialEq,
// {
//     type Target = AnimationE<Message>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
// ────────────────────────────────────────────────────────────────────────────────

#[allow(clippy::module_name_repetitions)]
pub struct AnimationE<Message>
where
    Message: Clone + std::fmt::Debug + 'static + PartialEq,
    // Ix: Clone + std::hash::Hash + Eq + Default + Ord + 'static,
{
    // sv_now: StateVar<Duration>,
    inside: AnimationInside<Message>,
    timing: StateAnchor<Timing>,
    pub(crate) running: StateAnchor<bool>,
    // store: Rc<RefCell<GStateStore>>,
    // edge: Option<EmgEdgeItem<Ix>>,
    // queued_interruptions: StateAnchor<StepTimeVector<Message>>,
    // revised_steps: StateAnchor<Vector<Step<Message>>>,
    // revised_props: StateAnchor<Vector<Property>>,
    // send_messages: StateAnchor<Vector<Message>>,
    // timing_ob: StateAnchor<()>,
    // processed_interruptions: StateAnchor<(StepTimeVector<Message>, StepTimeVector<Message>)>,
    // revised: SAPropsMessageSteps2<Message>,
    id: TopoKey,
    ref_count: Rc<Cell<usize>>,
}

impl<Message> Clone for AnimationE<Message>
where
    Message: Clone + std::fmt::Debug + 'static + PartialEq,
{
    fn clone(&self) -> Self {
        debug!("in animation clone, count:{}", self.ref_count.get());
        self.ref_count.set(self.ref_count.get() + 1);
        Self {
            inside: self.inside.clone(),
            timing: self.timing.clone(),
            running: self.running.clone(),
            id: self.id,
            ref_count: self.ref_count.clone(),
        }
    }
}

impl<Message> std::fmt::Debug for AnimationE<Message>
where
    Message: Clone + std::fmt::Debug + 'static + PartialEq,
    // Ix: Clone + std::hash::Hash + Eq + Default + Ord + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnimationEdge")
            .field("inside", &self.inside)
            .field("timing", &self.timing)
            .field("running", &self.running)
            // .field("revised", &self.revised)
            // .field("queued_interruptions", &self.queued_interruptions)
            // .field("revised_steps", &self.revised_steps)
            // .field("revised_props", &self.revised_props)
            // .field("send_messages", &self.send_messages)
            .finish()
    }
}

impl<Message> AnimationE<Message>
where
    Message: Clone + std::fmt::Debug + 'static + PartialEq,
    // Ix: Clone + std::hash::Hash + Eq + Default + Ord + std::fmt::Display + 'static,
{
    /// # Panics
    ///
    /// if not implemented
    // #[must_use]
    // pub fn get_position(&self, style_i: usize) -> Precision {
    //     self.inside.props.get_with(|props| {
    //         let p = props.get(style_i).unwrap();
    //         match p {
    //             Property::Prop(_name, m) => **m.position(),
    //             _ => todo!("not implemented"),
    //         }
    //     })
    // }

    fn set_timer(sv_now: StateVar<Duration>) -> StateAnchor<Timing> {
        // let mut timing = Timing::default();
        // let mut opt_old_current: Option<Timing> = None;
        sv_now
            .watch()
            // .map(move |now: &Duration| {
            //     if timing.current() != *now {
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
                    let current = timing.current();
                    if now == &current {
                        // timing.set_dt(Duration::ZERO);
                        return false;
                    }
                    // • • • • •

                    let dt_tmp = now.saturating_sub(current);
                    let dt = {
                        if current.is_zero() || dt_tmp.as_millis() > 34 {
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

    ////////
    /// using at tree building
    /// make animation property effecting what edge at what path
    /// # Panics
    ///
    /// Will panic when unimplemented
    /// # Errors
    ///
    /// Will return `Err` if `self.edge` is None
    /// permission to read it.
    pub fn effecting_edge_path<Ix>(&self, edge: &EmgEdgeItem<Ix>, for_path: EPath<Ix>)
    where
        Ix: Clone + std::hash::Hash + Eq + Default + Ord + std::fmt::Display + 'static,
    {
        edge.build_path_layout(|mut l| {
            // • • • • •

            self.inside.props.iter().for_each(|svp| {
                // let name = svp.get().name().to_string();
                let name = svp.get_with(emg_animation::models::Property::name);
                // let name = svp.store_get_rc(&*store).name().to_string();
                match name.as_str() {
                    //TODO full this
                    "CssWidth" => {
                        //TODO why directly l.w = ..., maybe change impl From<StateVarProperty> for StateVar<GenericSizeAnchor>
                        // l.w = (*svp).into();
                        l.w <<= svp;
                        // panic!("why directly l.w = ..., maybe change impl From<StateVarProperty> for StateVar<GenericSizeAnchor>");
                    }
                    _ => {
                        unimplemented!("not implemented....")
                    }
                }
            });
            // • • • • •

            (for_path, l)
        });
    }
    // pub fn effecting_path(self, for_path: EPath<Ix>) -> Result<Self, String> {
    //     self.edge
    //         .as_ref()
    //         .ok_or_else(|| "cannot effecting_path where self.edge is None .".to_string())?
    //         .build_path_layout(|mut l| {
    //             // • • • • •

    //             self.inside.props.iter().for_each(|svp| {
    //                 let name = svp.get().name().to_string();
    //                 // let name = svp.store_get_rc(&*store).name().to_string();
    //                 match name.as_str() {
    //                     "width" => l.w = (*svp).into(),
    //                     _ => {
    //                         unimplemented!("not implemented....")
    //                     }
    //                 }
    //             });
    //             // • • • • •

    //             (for_path, l)
    //         });
    //     Ok(self)
    // }
    #[allow(clippy::too_many_lines)]
    // #[track_caller]
    #[topo::nested]
    #[must_use]
    pub fn new_in_topo(
        props: Vector<StateVarProperty>,
        // sv_now: StateVar<Duration>,
        // edge_path: Option<(EmgEdgeItem<Ix>, EPath<Ix>)>,
    ) -> Self
    where
        Message: Clone + std::fmt::Debug + 'static + PartialEq,
        // Ix: Clone + std::hash::Hash + Eq + Default + Ord + std::fmt::Display + 'static,
    {
        let sv_now = global_clock();
        // let sv_now = use_state(Duration::ZERO);

        // let rc_store = state_store();
        // let rc_store2 = rc_store;

        // ─────────────────────────────────────────────────────────────────

        // let edge = if let Some((edge, p)) = edge_path {
        //     // let props_clone = props.clone();
        //     let store = rc_store2.clone();
        //     // ─────────────────────────────────────────────────────────────────
        //     // let layout_var: StateVar<GenericSizeAnchor> = props.iter().
        //     edge.build_path_layout(|mut l| {
        //         // • • • • •

        //         props.iter().for_each(|svp| {
        //             // let name = svp.get().name().to_string();
        //             let name = svp.store_get_rc(&store.borrow()).name().to_string();
        //             match name.as_str() {
        //                 "width" => l.w = (*svp).into(),
        //                 _ => {
        //                     unimplemented!("not implemented....")
        //                 }
        //             }
        //         });
        //         // • • • • •

        //         (p, l)
        //     });
        //     Some(edge)
        // } else {
        //     None
        // };

        {
            //TODO 这里对 props get操作了很多次 包括上面svp.get_with
            warn_for_double_listed_properties(&props);
        }

        // ─────────────────────────────────────────────────────────────────

        let sv_inside: AnimationInside<Message> = AnimationInside::<Message>::new_in_topo(props);

        // let store = rc_store2.borrow();

        let sa_timing = Self::set_timer(sv_now);

        let AnimationInside {
            interruption: interruption_init,
            steps: steps_init,
            props: props_init,
        } = sv_inside.clone();

        trace!("||@@@@@@@@@@@@@@@@@@@@ step id:{:#?}", &steps_init.id());

        let i_p_cut = {
            let mut opt_old_current: Option<Duration> = None;
            // let mut opt_old_interruption: Option<StepTimeVector<Message>> = None;
            // interruption_init.store_watch(&store)
            let pa: StateAnchor<Vector<Property>> = props_init
                .iter()
                .map(|sv| sv.get_var_with(Var::watch))
                // .map(|sv| sv.store_get_var_with(&store, Var::watch))
                .collect::<Anchor<Vector<_>>>()
                .into();

            (
                &sa_timing,
                &interruption_init.watch(),
                // &steps_init.store_watch(&store),
                &pa,
            )
                .map(|t, i, p| (*t, i.clone(), p.clone()))
                .cutoff(move |(timing, _, _)| {
                    let new_t = timing.current();
                    if let Some(old_t) = opt_old_current {
                        if old_t == new_t {
                            return false;
                        }
                    }

                    // if let Some(old_interruption) = &opt_old_interruption {
                    //     // if old_interruption.ptr_eq(new_interruption) {
                    //     if old_interruption == new_interruption {
                    //         return false;
                    //     }
                    // }

                    opt_old_current = Some(new_t);

                    // opt_old_interruption = Some(new_interruption.clone());

                    true
                })
                .map(|(_, i, p)| (i.clone(), p.clone()))
        };

        let revised: SAPropsMessageSteps2<Message> = (&sa_timing, &i_p_cut, &steps_init.watch())
            .map(
                move |timing: &Timing,
                      (interruption, props): &(StepTimeVector<Message>, Vector<Property>),
                      steps: &Vector<Step<Message>>| {
                    //----------------------------------
                    let (mut ready_interruption, queued_interruptions): (
                        StepTimeVector<Message>,
                        StepTimeVector<Message>,
                    ) = interruption
                        .clone()
                        .into_iter()
                        .map(|(wait, a_steps)| {
                            // println!("wait: {:?} , dt: {:?}", &wait, &dt);
                            (wait.saturating_sub(timing.dt()), a_steps)
                        })
                        .partition(|(wait, _)| wait.is_zero());

                    let (new_steps, new_props) = match ready_interruption.pop_front() {
                        Some((_ /* is zero */, interrupt_steps)) => (
                            interrupt_steps,
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
                        ),
                        None => (steps.clone(), props.clone()),
                    };
                    let (revised_props, sent_messages, revised_steps) =
                        resolve_steps(new_props, new_steps, timing.dt());
                    (
                        queued_interruptions,
                        revised_steps,
                        revised_props,
                        sent_messages,
                    )
                },
            );

        // ────────────────────────────────────────────────────────────────────────────────

        // // ─────────────────────────────────────────────────────────────────

        // state_store()
        //     .borrow()
        //     .engine_mut()
        //     .mark_observed(timing_ob.anchor());
        // ─────────────────────────────────────────────────────────────────
        // let line_id = Location::caller();
        // let id = TopoKey::new(topo::call_in_slot(line_id, topo::CallId::current));
        let id = TopoKey::new(topo::CallId::current());
        // let id = TopoKey::new(topo::call(topo::CallId::current));
        // let const_id = || topo::root(topo::CallId::current);
        // let id = TopoKey::new(const_id());
        trace!("=======|||||||||||||||||||||||--> topo id:{:?}", &id);

        let sa_running = (&interruption_init.watch(), &steps_init.watch())
            .map(|q, r| !q.is_empty() || !r.is_empty());

        // state_store()
        //     .borrow()
        //     .engine_mut()
        //     .mark_observed(sa_timing.anchor());
        // state_store()
        //     .borrow()
        //     .engine_mut()
        //     .mark_observed(revised.anchor());
        // state_store()
        //     .borrow()
        //     .engine_mut()

        //     .mark_unobserved(sa_running.anchor());

        // ─────────────────────────────────────────────────────────────────

        //TODO need remove when self drop
        global_anima_running_add(&sa_running);
        let sa_running_clone = sa_running.clone();

        sv_now
            .insert_after_fn(
                id,
                move |skip, _| {
                    // println!("call update after set timing {:?}", v);
                    debug!("====[insert_after_fn] calling --> topo id:{:?}", &id);
                    // anima_clone.update_in_callback(skip);
                    if !sa_running_clone.get() {
                        debug!("not running , return");
                        return;
                    }
                    debug!("after callback running ");

                    let revised_value = revised.get();
                    props_init
                        .iter()
                        .zip(revised_value.2.iter())
                        .for_each(|(sv, prop)| sv.seting_in_b_a_callback(skip, prop));
                    interruption_init.seting_in_b_a_callback(skip, &revised_value.0);
                    steps_init.seting_in_b_a_callback(skip, &revised_value.1);
                },
                false,
            )
            // .ok();
            .expect("find same id already in after_fn map");

        // let update_id = TopoKey::new(topo::call(topo::CallId::current));

        // let mut anima_clone = an.clone();
        // anima_clone.id = None;

        // drop(store);

        Self {
            // sv_now,
            inside: sv_inside,
            timing: sa_timing,
            running: sa_running,
            // store: rc_store,
            // edge,
            // queued_interruptions: sa_queued_interruptions,
            // revised_props: sa_revised_props,
            // revised_steps: sa_revised_steps,
            // send_messages: sa_message,
            // timing_ob,
            // processed_interruptions: sa_processed_interruptions,
            // revised,
            id,
            ref_count: Rc::new(Cell::new(1)), //start with 1
        }
    }

    pub fn interrupt(&self, steps: Vector<Step<Message>>) {
        self.inside.interruption.set_with_once(|interruption| {
            let mut new_interruption = interruption.clone();
            let xx = extract_initial_wait(steps);
            new_interruption.push_front(xx);
            new_interruption
        });
    }
}

impl<Message> Drop for AnimationE<Message>
where
    Message: Clone + std::fmt::Debug + 'static + PartialEq,
{
    fn drop(&mut self) {
        let count = self.ref_count.get();
        debug!("===============in Dropping  AnimationE count:{}", &count);

        if count <= 1 {
            G_CLOCK.with(|clock| {
                // self.running = StateAnchor::constant(false);
                clock.remove_after_fn(self.id);
            });
        } else {
            self.ref_count.set(count - 1);
            debug!("===============after count:{}", &self.ref_count.get());
        }
        // let clock = global_clock();
    }
}
#[cfg(test)]
mod tests {
    extern crate test;

    use std::path::Path;
    use std::time::Duration;

    use emg::{edge_index, edge_index_no_source, node_index, Edge, EdgeIndex};
    use emg_animation::models::Property;
    use emg_animation::{interrupt, opacity, style, to, Tick};
    use emg_core::{into_vector, IdStr};
    use emg_core::{vector, Vector};
    use emg_state::{
        state_store, topo, use_state, CloneStateAnchor, CloneStateVar, Dict, GStateStore, StateVar,
    };
    use seed_styles as styles;
    use styles::{pc, width};
    use styles::{px, CssWidth};

    use crate::animation::global_clock;
    use crate::{EPath, EdgeItemNode, EmgEdgeItem, GraphEdgesDict};

    use super::AnimationE;
    use tracing::{debug, warn, Level};

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
            .with_max_level(Level::DEBUG)
            .try_init();

        // tracing::subscriber::set_global_default(subscriber)
        // .expect("setting default subscriber failed");
    }

    #[allow(dead_code)]
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
    #[allow(dead_code)]
    #[derive(Debug, Clone, PartialEq)]
    enum Message {
        A,
    }

    use test::{black_box, Bencher};

    #[bench]
    fn bench_nom_am(b: &mut Bencher) {
        b.iter(|| {
            let mut am = style::<Message>(into_vector![width(px(1))]);
            black_box(nom_am_run(&mut am));
        });
    }
    #[test]
    fn nom_am() {
        let mut am = style::<Message>(into_vector![width(px(1))]);
        nom_am_run(&mut am);
    }

    fn nom_am_run(am: &mut emg_animation::models::Animation<Message>) {
        interrupt(
            vector![
                to(into_vector![width(px(0))]),
                to(into_vector![width(px(1))]),
                to(into_vector![width(px(0))]),
                to(into_vector![width(px(1))]),
                to(into_vector![width(px(0))]),
                to(into_vector![width(px(1))]),
                to(into_vector![width(px(0))]),
                to(into_vector![width(px(1))]),
                to(into_vector![width(px(0))]),
                to(into_vector![width(px(1))]),
                to(into_vector![width(px(0))]),
                to(into_vector![width(px(1))]),
                to(into_vector![width(px(0))]),
                to(into_vector![width(px(1))]),
                to(into_vector![width(px(0))]),
                to(into_vector![width(px(1))]),
                to(into_vector![width(px(0))]),
                to(into_vector![width(px(1))]),
                to(into_vector![width(px(0))]),
            ],
            am,
        );
        for i in 1002..2000 {
            emg_animation::update(Tick(Duration::from_millis(i * 16)), am);
            let _e = am.get_position(0);
        }
        let _e = am.get_position(0);
    }

    #[bench]
    #[topo::nested]

    fn bench_less_state_am(b: &mut Bencher) {
        // let ei = edge_index_no_source("fff");
        // let source = use_state(ei.source_nix().as_ref().cloned());
        // let target = use_state(ei.target_nix().as_ref().cloned());
        // let edge_item: EmgEdgeItem<String> = EmgEdgeItem::default_with_wh_in_topo(
        //     source.watch(),
        //     target.watch(),
        //     StateAnchor::constant(Dict::default()),
        //     1920,
        //     1080,
        // );
        let sv_now = global_clock();

        b.iter(move || {
            sv_now.set(Duration::from_millis(0));

            // let edge_item1 = edge_item.clone();
            let a: AnimationE<Message> = AnimationE::new_in_topo(into_vector![width(px(1))]);
            black_box(less_am_run(&state_store().borrow(), &a, &sv_now));
        });
    }

    fn less_am_run(
        storeref: &GStateStore,
        a: &AnimationE<Message>,
        sv_now: &emg_state::StateVar<Duration>,
    ) {
        a.interrupt(vector![
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
            // to(into_vector![width(px(0))]),
            // to(into_vector![width(px(1))]),
        ]);

        sv_now.store_set(storeref, Duration::from_millis(16));
        // a.update();
        for i in 1002..2000 {
            sv_now.store_set(storeref, Duration::from_millis(i * 16));
            // a.update();
            a.inside.props[0].store_get(storeref);
        }
        a.inside.props[0].store_get(storeref);
    }
    #[bench]
    #[topo::nested]

    fn bench_many_state_am(b: &mut Bencher) {
        // let ei = edge_index_no_source("fff");
        // let source = use_state(ei.source_nix().as_ref().cloned());
        // let target = use_state(ei.target_nix().as_ref().cloned());
        // let edge_item: EmgEdgeItem<String> = EmgEdgeItem::default_with_wh_in_topo(
        //     source.watch(),
        //     target.watch(),
        //     StateAnchor::constant(Dict::default()),
        //     1920,
        //     1080,
        // );
        // let sv_now = use_state(Duration::ZERO);
        let sv_now = global_clock();

        b.iter(move || {
            sv_now.set(Duration::from_millis(0));
            // let edge_item1 = edge_item.clone();

            let a: AnimationE<Message> = AnimationE::new_in_topo(into_vector![width(px(1))]);
            // AnimationE::new_in_topo(into_vector![width(px(1))], sv_now);
            black_box(many_am_run(&a, &sv_now));
        });
    }

    #[test]
    #[topo::nested]
    fn many() {
        let _g = _init();

        // let sv_now = use_state(Duration::ZERO);
        let sv_now = global_clock();
        sv_now.set(Duration::from_millis(0));

        debug!("===================================main loop ");
        let a: AnimationE<Message> = AnimationE::new_in_topo(into_vector![width(px(2))]);
        a.interrupt(vector![to(into_vector![width(px(0))]),]);

        debug!("===================================main loop--2 ");

        let b: AnimationE<Message> = AnimationE::new_in_topo(into_vector![width(px(99))]);
        b.interrupt(vector![to(into_vector![width(px(888))]),]);
        sv_now.set(Duration::from_millis(16));
        debug!("a====:\n {:#?}", a.inside.props[0].get());
        debug!("b====:\n {:#?}", b.inside.props[0].get());
        sv_now.set(Duration::from_millis(33));
        debug!("a 33====:\n {:#?}", a.inside.props[0].get());
        debug!("b 33====:\n {:#?}", b.inside.props[0].get());
        insta::assert_debug_snapshot!("many-33-a", &a);
        insta::assert_debug_snapshot!("many-33-b", &b);
    }

    #[test]
    #[topo::nested]
    fn many_for() {
        let _g = _init();

        // let sv_now = use_state(Duration::ZERO);
        let sv_now = global_clock();
        for i in 0..4 {
            // let edge_item1 = edge_item.clone();
            sv_now.set(Duration::from_millis(0));

            debug!("===================================main loop :{}", i);
            let a: AnimationE<Message> = AnimationE::new_in_topo(into_vector![width(px(1))]);
            black_box(many_am_run_for_test(&a, &sv_now));
        }
    }

    fn many_am_run_for_test(
        // storeref: &GStateStore,
        a: &AnimationE<Message>,
        sv_now: &emg_state::StateVar<Duration>,
    ) {
        a.interrupt(vector![
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
        ]);
        warn!("set time ---------------------------------------------------- 16");
        sv_now.set(Duration::from_millis(16));
        // sv_now.store_set(storeref, Duration::from_millis(16));
        // a.update();
        // for i in 1002..1004 {
        for i in 1..5 {
            warn!(
                "in loop: set time ---------------------------------------------------- loop:{}",
                &i
            );

            sv_now.set(Duration::from_millis(i * 16));
            // sv_now.store_set(storeref, Duration::from_millis(i * 16));
            // a.update();
            // a.inside.props[0].store_get(storeref);
            a.inside.props[0].get();
        }
        a.inside.props[0].get();
        // a.inside.props[0].store_get(storeref);
    }

    fn many_am_run(
        // storeref: &GStateStore,
        a: &AnimationE<Message>,
        sv_now: &emg_state::StateVar<Duration>,
    ) {
        a.interrupt(vector![
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
        ]);
        warn!("set time ---------------------------------------------------- 16");
        sv_now.set(Duration::from_millis(16));
        // sv_now.store_set(storeref, Duration::from_millis(16));
        // a.update();
        // for i in 1002..1004 {
        for i in 1002..2000 {
            warn!(
                "in loop: set time ---------------------------------------------------- loop:{}",
                &i
            );

            sv_now.set(Duration::from_millis(i * 16));
            // sv_now.store_set(storeref, Duration::from_millis(i * 16));
            // a.update();
            // a.inside.props[0].store_get(storeref);
            a.inside.props[0].get();
        }
        assert_eq!(a.inside.props[0].get(), Property::from(width(px(0))));
        // a.inside.props[0].store_get(storeref);
    }

    #[test]
    #[topo::nested]
    fn test_animation_in_topo() {
        // let nn = _init();
        {
            // let span = trace_span!("am-test");
            // let _guard = span.enter();
            // trace!("fff");

            // let ei = edge_index_no_source("fff");
            // let source = use_state(ei.source_nix().as_ref().cloned());
            // let target = use_state(ei.target_nix().as_ref().cloned());
            // let edge_item: EmgEdgeItem<String> = EmgEdgeItem::default_with_wh_in_topo(
            //     source.watch(),
            //     target.watch(),
            //     StateAnchor::constant(Dict::default()),
            //     1920,
            //     1080,
            // );

            let sv_now = global_clock();
            sv_now.set(Duration::from_millis(0));

            let a: AnimationE<Message> = AnimationE::new_in_topo(into_vector![opacity(1.)]);
            // println!("a:{:#?}", &a);
            insta::assert_debug_snapshot!("new", &a);
            insta::assert_debug_snapshot!("new2", &a);
            assert_eq!(a.running.get(), false);
            insta::assert_debug_snapshot!("get_running", &a);
            // println!("now set interrupt");
            a.interrupt(vector![
                to(vector![emg_animation::opacity(0.)]),
                to(vector![emg_animation::opacity(1.)])
            ]);
            // println!("over interrupt");

            insta::assert_debug_snapshot!("interrupt", &a);
            insta::assert_debug_snapshot!("interrupt2", &a);
            // println!("over interrupt insta");

            assert_eq!(a.running.get(), true);
            // println!("over interrupt running.get()");
            // a.update_animation();
            // ────────────────────────────────────────────────────────────────────────────────

            sv_now.set(Duration::from_millis(16));
            insta::assert_debug_snapshot!("set16", &a);
            // a.update();
            // println!("set timing 16");

            // println!("set timing 16-- update");

            // println!("1**{:?}", a.inside.props.get());

            insta::assert_debug_snapshot!("updated_16_a_0", &a);
            insta::assert_debug_snapshot!("updated_16_a_1", &a);
            // println!("set timing 16-- insta");
            // ────────────────────────────────────────────────────────────────────────────────
            sv_now.set(Duration::from_millis(16));
            // a.update();
            // println!("set timing 16-2");

            insta::assert_debug_snapshot!("updated_16_b_0", &a);
            insta::assert_debug_snapshot!("updated_16_b_1", &a);
            // println!("set timing 16-- insta-2");
            // ─────────────────────────────────────────────────────────────────

            sv_now.set(Duration::from_millis(33));
            insta::assert_debug_snapshot!("set33", &a);
            // a.update();
            // println!("set timing 33");

            // println!("....set 2 ");

            insta::assert_debug_snapshot!("updated_33_0", &a);

            // println!("set timing 33 -- update 1");

            // a.update();
            insta::assert_debug_snapshot!("updated_33_1", &a);

            // println!("set timing 33 -- update 2");

            // println!("2**{:?}", a.inside.props.get());

            insta::assert_debug_snapshot!("snap_updated_33_0", &a);
            insta::assert_debug_snapshot!("snap_updated_33_1", &a);
            // println!("set timing 33 -- insta  ");

            // sv_now.set(Duration::from_millis(2));
            // a.update_animation();
            // insta::assert_debug_snapshot!("updated_back_0", &a);
            // insta::assert_debug_snapshot!("updated_back_1", &a);

            for i in 3..200 {
                sv_now.set(Duration::from_millis(i * 16));
                // a.update();
                // println!("in ------ i:{}", &i);
                // a.timing.get();
                // println!("3***{:?}", a.inside.props.get());
                a.inside.props[0].get();
            }
            insta::assert_debug_snapshot!("updated_end_0", &a);
            insta::assert_debug_snapshot!("updated_end_1", &a);

            // // println!("{:?}", a.revised_props.get());
            // // state_store().borrow().engine_mut().stabilize();
            // println!("end : {:?}", a.inside.props.get());
            // println!("{:?}", a);

            a.inside.props[0].get();
        }
    }

    #[test]
    #[topo::nested]
    fn test_layout_anima() {
        // ! layout am
        let _nn = _init();

        insta::with_settings!({snapshot_path => Path::new("./layout_am")}, {

            let css_w: StateVar<CssWidth> = use_state(width(px(1)));

            // let span = trace_span!("am-test");
            // let _guard = span.enter();
            // trace!("fff");

            let e_dict_sv:StateVar<GraphEdgesDict<IdStr>> = use_state(Dict::new());

            let root_e_source =use_state( None);
            let root_e_target = use_state(Some(node_index("root")));
            let  root_e = EmgEdgeItem::default_with_wh_in_topo(root_e_source.watch(), root_e_target.watch(),e_dict_sv.watch(),1920, 1080);
            e_dict_sv.set_with(|d|{
                let mut nd = d .clone();
                nd.insert(EdgeIndex::new(None,node_index("root")), Edge::new(root_e_source, root_e_target, root_e.clone()));
                nd
            });

            // let e1_source =use_state( Some(node_index("root")));
            // let e1_target = use_state(Some(node_index("1")));
            // let e1 = EmgEdgeItem::new_in_topo(
            //         e1_source.watch(),
            //         e1_target.watch(),
            //     e_dict_sv.watch(),
            //     (px(50).into(), px(50).into()),
            //      (pc(0).into(), pc(0).into(), pc(0).into()),
            //       (pc(50).into(), pc(50).into(), pc(50).into()),
            // );

            // e_dict_sv.set_with(|d|{
            //     let mut nd = d .clone();
            //     nd.insert(edge_index("root","1"), Edge::new(e1_source, e1_target, e1.clone()));
            //     nd
            // });

            // ─────────────────────────────────────────────────────────────────

            let ew = root_e.layout.w;
            // debug!("e->{}",&e1);
            insta::assert_debug_snapshot!("layout-am-edge", &root_e);

            css_w.set(width(px(99)));

            let edge_style_string_sa = root_e
                    .edge_nodes
                    .get()
                    .get(&EPath(vector![edge_index_no_source("root")]))
                    .and_then(EdgeItemNode::as_edge_data)
                            .unwrap()
                            .styles_string.clone();
                            // .get();
            // let edge_style_string_sa = e1
            //         .node
            //         .get()
            //         .get(&EPath(vector![edge_index_no_source("root"),edge_index("root","1")]))
            //         .and_then(EdgeItemNode::as_edge_data)
            //                 .unwrap()
            //                 .styles_string.clone();
            //                 // .get();

            let sv_now = global_clock();
            sv_now.set(Duration::from_millis(0));

            let a: AnimationE< Message> =
                // AnimationEdge::new_in_topo(into_vector![width(px(1))], e1, sv_now);
                AnimationE::new_in_topo(into_vector![css_w]);
                a.effecting_edge_path( &root_e,EPath(vector![edge_index_no_source("root")]));
            // println!("a:{:#?}", &a);
            insta::assert_debug_snapshot!("new", &a);
            insta::assert_debug_snapshot!("new2", &a);
            let new1 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__new.snap")).unwrap();
            let new2 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__new2.snap")).unwrap();
            assert_eq!(new1.contents(),new2.contents());

            assert_eq!(a.running.get(), false);
            insta::assert_debug_snapshot!("get_running", &a);
            // println!("now set interrupt");
            a.interrupt(vector![
                to(into_vector![width(px(0))]),
                to(into_vector![width(px(1))])
            ]);
            // println!("over interrupt");

            insta::assert_debug_snapshot!("interrupt", &a);
            // insta::assert_debug_snapshot!("interrupt2", &a);
            // println!("over interrupt insta");

            assert_eq!(a.running.get(), true);
            // println!("over interrupt running.get()");
            // a.update_animation();
            // ────────────────────────────────────────────────────────────────────────────────

            sv_now.set(Duration::from_millis(16));
            // println!("set timing 16");
            insta::assert_debug_snapshot!("set16", &a);

            // a.update();
            // println!("set timing 16-- update");

            // println!("1**{:?}", a.inside.props.get());

            insta::assert_debug_snapshot!("updated_16_0", &a);
            insta::assert_debug_snapshot!("updated_16_0-edge", &root_e);
            // insta::assert_debug_snapshot!("updated_16_1", &a);
            // println!("set timing 16-- insta");
            // ────────────────────────────────────────────────────────────────────────────────
            sv_now.set(Duration::from_millis(16));
            // println!("set timing 16-2");

            // a.update();

            insta::assert_debug_snapshot!("updated_16_0-2", &a);
            // insta::assert_debug_snapshot!("updated_16_1-2", &a);
            // println!("set timing 16-- insta-2");
            let u16 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__updated_16_0.snap")).unwrap();
            let u16_2 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__updated_16_0-2.snap")).unwrap();
            assert_eq!(u16.contents(),u16_2.contents());
            // ─────────────────────────────────────────────────────────────────

            sv_now.set(Duration::from_millis(33));
            // println!("set timing 33");

            // println!("....set 2 ");
            insta::assert_debug_snapshot!("set33", &a);

            // a.update();
            insta::assert_debug_snapshot!("updated_33_0", &a);
            insta::assert_debug_snapshot!("updated_33_0-edge", &root_e);

            // println!("set timing 33 -- update 1");

            // a.update();
            insta::assert_debug_snapshot!("updated_33_1", &a);

            // println!("set timing 33 -- update 2");

            // println!("2**{:?}", a.inside.props.get());

            insta::assert_debug_snapshot!("snap_updated_33_0", &a);
            // insta::assert_debug_snapshot!("snap_updated_33_1", &a);
            // println!("set timing 33 -- insta  ");
            let f33 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__updated_33_1.snap")).unwrap();
            let f33_2 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__snap_updated_33_0.snap")).unwrap();
            assert_eq!(f33.contents(),f33_2.contents());

            // sv_now.set(Duration::from_millis(2));
            // a.update_animation();
            // insta::assert_debug_snapshot!("updated_back_0", &a);
            // insta::assert_debug_snapshot!("updated_back_1", &a);

            for i in 3..200 {
                println!("===========================================================");
                if i == 50{
                    println!(" 50-> : current width:{}",&css_w);
                    css_w.set(width(px(20)));

                }
                sv_now.set(Duration::from_millis(i * 16));

                // println!("in ------ i:{}", &i);
                // a.timing.get();´ß
                // a.update();

                println!("***-- {:?} | \new:-> {:?}, \n style:-> {:?}",CssWidth::from( a.inside.props[0].get()),ew,edge_style_string_sa);

                println!("===========================================================");

                //  a.inside.props[0].get();
            }
            insta::assert_debug_snapshot!("updated_end_0", &a);
            // insta::assert_debug_snapshot!("updated_end_1", &a);

            // // println!("{:?}", a.revised_props.get());
            // // state_store().borrow().engine_mut().stabilize();
            // println!("end : {:?}", a.inside.props.get());
            // println!("{:?}", a);
            // a.inside.props[0].get();
            // ─────────────────────────────────────────────────────────────────

            css_w.set(width(px(20)));
            insta::assert_debug_snapshot!("end_set1", &a);
            sv_now.set(sv_now.get() + Duration::from_millis(16));
            insta::assert_debug_snapshot!("end_set2-settime", &a);
            // a.update();
            insta::assert_debug_snapshot!("end_set3-update", &a);

        });
    }

    #[bench]
    #[topo::nested]

    fn anima_macro_bench(b: &mut Bencher) {
        b.iter(move || {
            black_box(anima_macro_for_bench());
        });
    }

    #[test]
    #[topo::nested]
    fn anima_macro_for_bench_2_test() {
        let _g = _init();

        anima_macro_for_bench();
        global_clock().set(Duration::from_millis(0));

        anima_macro_for_bench();
    }
    #[test]
    #[topo::nested]
    fn anima_macro_for_2_test() {
        let _g = _init();

        anima_macro();
        global_clock().set(Duration::from_millis(0));

        anima_macro();
    }
    #[test]
    #[topo::nested]
    fn anima_macro_for_bench() {
        // let _g = _init();
        let sv_now = global_clock();
        sv_now.set(Duration::from_millis(0));
        let css_w: StateVar<CssWidth> = use_state(width(px(1)));
        let a: AnimationE<Message> = anima![css_w];

        let e_dict_sv: StateVar<GraphEdgesDict<IdStr>> = use_state(Dict::new());
        let root_e_source = use_state(None);
        let root_e_target = use_state(Some(node_index("root")));
        let root_e = EmgEdgeItem::default_with_wh_in_topo(
            root_e_source.watch(),
            root_e_target.watch(),
            e_dict_sv.watch(),
            1920,
            1080,
        );
        a.effecting_edge_path(&root_e, EPath(vector![edge_index_no_source("root")]));
        a.interrupt(vector![
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
            to(into_vector![width(px(0))]),
        ]);
        for i in 1..1000 {
            sv_now.set(Duration::from_millis(i * 16));
            a.inside.props[0].get();
        }
    }
    #[test]
    #[topo::nested]
    fn anima_macro() {
        let _g = _init();
        let sv_now = global_clock();
        sv_now.set(Duration::from_millis(0));

        let css_w: StateVar<CssWidth> = use_state(width(px(1)));
        let a: AnimationE<Message> = anima![css_w];
        insta::assert_debug_snapshot!("anima_macro_init", &a);

        let e_dict_sv: StateVar<GraphEdgesDict<IdStr>> = use_state(Dict::new());
        let root_e_source = use_state(None);
        let root_e_target = use_state(Some(node_index("root")));
        let root_e = EmgEdgeItem::default_with_wh_in_topo(
            root_e_source.watch(),
            root_e_target.watch(),
            e_dict_sv.watch(),
            1920,
            1080,
        );
        a.effecting_edge_path(&root_e, EPath(vector![edge_index_no_source("root")]));
        a.interrupt(vector![
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))]),
        ]);
        insta::assert_debug_snapshot!("anima_macro_interrupt", &a);

        for i in 1..100 {
            sv_now.set(Duration::from_millis(i * 16));
            if i == 1 {
                insta::assert_debug_snapshot!("anima_macro_16", &a);
                insta::assert_debug_snapshot!("anima_macro_16_edge", &root_e);
            }
            // a.update();
            // println!("in ------ i:{}", &i);
            // a.timing.get();
            debug!("**{:?}", a.inside.props[0].get());
            a.inside.props[0].get();
        }
    }

    #[test]
    #[topo::nested]
    fn test_layout_children_anima() {
        // ! layout am
        let _nn = _init();

        insta::with_settings!({snapshot_path => Path::new("./layout_children_am")}, {

            let css_w: StateVar<CssWidth> = use_state(width(px(1)));

            // let span = trace_span!("am-test");
            // let _guard = span.enter();
            // trace!("fff");

            let e_dict_sv:StateVar<GraphEdgesDict<IdStr>> = use_state(Dict::new());

            let root_e_source =use_state( None);
            let root_e_target = use_state(Some(node_index("root")));
            let root_e = EmgEdgeItem::default_with_wh_in_topo(root_e_source.watch(), root_e_target.watch(),e_dict_sv.watch(),1920, 1080);
            e_dict_sv.set_with(|d|{
                let mut nd = d .clone();
                nd.insert(EdgeIndex::new(None,node_index("root")), Edge::new(root_e_source, root_e_target, root_e.clone()));
                nd
            });

            let e1_source =use_state( Some(node_index("root")));
            let e1_target = use_state(Some(node_index("1")));
            let e1 = EmgEdgeItem::new_in_topo(
                    e1_source.watch(),
                    e1_target.watch(),
                e_dict_sv.watch(),
                (px(50).into(), px(50).into()),
                 (pc(0).into(), pc(0).into(), pc(0).into()),
                  (pc(50).into(), pc(50).into(), pc(50).into()),
            );

            e_dict_sv.set_with(|d|{
                let mut nd = d .clone();
                nd.insert(edge_index("root","1"), Edge::new(e1_source, e1_target, e1.clone()));
                nd
            });

            // ─────────────────────────────────────────────────────────────────

            let ew = root_e.layout.w;
            // debug!("e->{}",&e1);
            insta::assert_debug_snapshot!("layout-am-edge", &e1);

            css_w.set(width(px(99)));

            let edge_style_string_sa = e1
                    .edge_nodes
                    .get()
                    .get(&EPath(vector![edge_index_no_source("root"),edge_index("root","1")]))
                    .and_then(EdgeItemNode::as_edge_data)
                            .unwrap()
                            .styles_string.clone();
            //                 // .get();

            let sv_now = global_clock();
            sv_now.set(Duration::from_millis(0));

            let a: AnimationE< Message> =
                // AnimationEdge::new_in_topo(into_vector![width(px(1))], e1, sv_now);
                AnimationE::new_in_topo(into_vector![css_w]);
                a.effecting_edge_path(&e1,EPath(vector![edge_index_no_source("root"),edge_index("root","1")]));

            // println!("a:{:#?}", &a);
            insta::assert_debug_snapshot!("new", &a);
            insta::assert_debug_snapshot!("new2", &a);
            let new1 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__new.snap")).unwrap();
            let new2 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__new2.snap")).unwrap();
            assert_eq!(new1.contents(),new2.contents());

            assert_eq!(a.running.get(), false);
            insta::assert_debug_snapshot!("get_running", &a);
            // println!("now set interrupt");
            a.interrupt(vector![
                to(into_vector![width(px(0))]),
                to(into_vector![width(px(1))])
            ]);
            // println!("over interrupt");

            insta::assert_debug_snapshot!("interrupt", &a);
            // insta::assert_debug_snapshot!("interrupt2", &a);
            // println!("over interrupt insta");

            assert_eq!(a.running.get(), true);
            // println!("over interrupt running.get()");
            // a.update_animation();
            // ────────────────────────────────────────────────────────────────────────────────

            sv_now.set(Duration::from_millis(16));
            // println!("set timing 16");
            insta::assert_debug_snapshot!("set16", &a);

            // a.update();
            // println!("set timing 16-- update");

            // println!("1**{:?}", a.inside.props.get());

            insta::assert_debug_snapshot!("updated_16_0", &a);
            insta::assert_debug_snapshot!("updated_16_0-edge", &e1);
            // insta::assert_debug_snapshot!("updated_16_1", &a);
            // println!("set timing 16-- insta");
            // ────────────────────────────────────────────────────────────────────────────────
            sv_now.set(Duration::from_millis(16));
            // println!("set timing 16-2");

            // a.update();

            insta::assert_debug_snapshot!("updated_16_0-2", &a);
            // insta::assert_debug_snapshot!("updated_16_1-2", &a);
            // println!("set timing 16-- insta-2");
            let u16 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__updated_16_0.snap")).unwrap();
            let u16_2 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__updated_16_0-2.snap")).unwrap();
            assert_eq!(u16.contents(),u16_2.contents());
            // ─────────────────────────────────────────────────────────────────

            sv_now.set(Duration::from_millis(33));
            // println!("set timing 33");

            // println!("....set 2 ");
            insta::assert_debug_snapshot!("set33", &a);

            // a.update();
            insta::assert_debug_snapshot!("updated_33_0", &a);
            insta::assert_debug_snapshot!("updated_33_0-edge", &e1);

            // println!("set timing 33 -- update 1");

            // a.update();
            insta::assert_debug_snapshot!("updated_33_1", &a);

            // println!("set timing 33 -- update 2");

            // println!("2**{:?}", a.inside.props.get());

            insta::assert_debug_snapshot!("snap_updated_33_0", &a);
            // insta::assert_debug_snapshot!("snap_updated_33_1", &a);
            // println!("set timing 33 -- insta  ");
            let f33 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__updated_33_1.snap")).unwrap();
            let f33_2 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__snap_updated_33_0.snap")).unwrap();
            assert_eq!(f33.contents(),f33_2.contents());

            // sv_now.set(Duration::from_millis(2));
            // a.update_animation();
            // insta::assert_debug_snapshot!("updated_back_0", &a);
            // insta::assert_debug_snapshot!("updated_back_1", &a);

            for i in 3..200 {
                println!("===========================================================");
                if i == 50{
                    println!(" 50-> : current width:{}",&css_w);
                    css_w.set(width(px(20)));

                }
                sv_now.set(Duration::from_millis(i * 16));

                // println!("in ------ i:{}", &i);
                // a.timing.get();´ß
                // a.update();

                println!("***-- {:?} | \new:-> {:?}, \n style:-> {:?}",CssWidth::from( a.inside.props[0].get()),ew,edge_style_string_sa);

                println!("===========================================================");

                //  a.inside.props[0].get();
            }
            insta::assert_debug_snapshot!("updated_end_0", &a);
            // insta::assert_debug_snapshot!("updated_end_1", &a);

            // // println!("{:?}", a.revised_props.get());
            // // state_store().borrow().engine_mut().stabilize();
            // println!("end : {:?}", a.inside.props.get());
            // println!("{:?}", a);
            // a.inside.props[0].get();
            // ─────────────────────────────────────────────────────────────────

            css_w.set(width(px(20)));
            insta::assert_debug_snapshot!("end_set1", &a);
            sv_now.set(sv_now.get() + Duration::from_millis(16));
            insta::assert_debug_snapshot!("end_set2-settime", &a);
            // a.update();
            insta::assert_debug_snapshot!("end_set3-update", &a);

        });
    }
}
