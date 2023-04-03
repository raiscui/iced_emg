/*
 * @Author: Rais
 * @Date: 2021-05-28 11:50:10
 * @LastEditTime: 2023-04-03 23:10:43
 * @LastEditors: Rais
 * @Description:
 */

mod define;
mod func;

use emg_common::{im::vector, Precision, SmallVec, Vector};
use emg_state::{
    anchors::expert::CastIntoValOrAnchor, general_struct::TopoKey, state_lit::StateVarLit,
    state_store, topo, Anchor, CloneState, CloneStateAnchor, StateAnchor, StateMultiAnchor,
    StateVar,
};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
    time::Duration,
};

use emg_animation::{
    extract_initial_wait,
    models::{map_to_motion, resolve_steps, Motion, MsgBackIsNew, Property, Step, StepTimeVector},
    set_default_interpolation, Timing, PROP_SIZE,
};
use tracing::{debug, trace};

use crate::{global_anima_running_add, global_clock, EPath, EmgEdgeItem};

use self::{define::StateVarProperty, func::props::warn_for_double_listed_properties};

// ────────────────────────────────────────────────────────────────────────────────
// #[allow(dead_code)]
// type SAPropsMessageSteps<Message> =
//     StateAnchor<(Vector<Property>, Vector<Message>, Vector<StepOG<Message>>)>;

#[allow(dead_code)]
type SAPropsMessageSteps2<Message> = StateAnchor<(
    StepTimeVector<Message>,
    // VecDeque<Step<Message>>,
    Rc<RefCell<VecDeque<Step<Message>>>>,
    SmallVec<[Property; PROP_SIZE]>,
    MsgBackIsNew<Message>,
)>;

// ────────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
struct AnimationInside<Message>
where
    Message: Clone + std::fmt::Debug + 'static,
{
    pub(crate) steps: StateVarLit<Rc<RefCell<VecDeque<Step<Message>>>>>,
    pub(crate) interruption: StateVarLit<StepTimeVector<Message>>,
    pub(crate) props: SmallVec<[StateVarProperty; PROP_SIZE]>,
}

impl<Message> AnimationInside<Message>
where
    Message: Clone + std::fmt::Debug + 'static,
{
    fn new(props: SmallVec<[StateVarProperty; PROP_SIZE]>) -> Self {
        props.iter().for_each(|prop| {
            prop.update(|p| {
                set_default_interpolation(p);
            });
        });
        Self {
            steps: StateVarLit::new(Rc::new(RefCell::new(VecDeque::new()))),
            interruption: StateVarLit::new(vector![]),
            props,
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────────

//TODO for the path macro: [path =>[xx],path=>[xxx]]
#[macro_export]
macro_rules! anima {
    ( $( $element:expr ) , * ) => {
        {
            let mut v = $crate::emg_common::SmallVec::new();

            $(
                v.push($element.into());
            )*

            $crate::AnimationE::new_in_topo(v)
            // $crate::topo::call(||$crate::AnimationE::new_in_topo(v))
        }
    };
}
// // @ use for shaping  表示 要关联到 单一路径节点 ────────────────────────────────────────────────────────────────────────────────

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
{
    // sv_now: StateVar<Duration>,
    inside: AnimationInside<Message>,
    timing: StateAnchor<Timing>,
    pub(crate) running: StateAnchor<bool>,
    // store: Rc<RefCell<GStateStore>>,
    // edge: Option<EmgEdgeItem>,
    // queued_interruptions: StateAnchor<StepTimeVector<Message>>,
    // revised_steps: StateAnchor<Vector<Step<Message>>>,
    // revised_props: StateAnchor<Vector<Property>>,
    // send_messages: StateAnchor<Vector<Message>>,
    // timing_ob: StateAnchor<()>,
    // processed_interruptions: StateAnchor<(StepTimeVector<Message>, StepTimeVector<Message>)>,
    // revised: SAPropsMessageSteps2<Message>,
    id: TopoKey,
    ref_count: Rc<Cell<usize>>, //NOTE for drop, current no used
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
{
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
                    if now == current {
                        // timing.set_dt(Duration::ZERO);
                        return false;
                    }
                    // • • • • •

                    let dt_tmp = now.saturating_sub(*current);
                    let dt = {
                        if current.is_zero() || dt_tmp.as_millis() > 34 {
                            Duration::from_micros(16666) //16.666 ms
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
    //TODO return bool changed or not
    pub fn effecting_edge_path(&self, edge: &EmgEdgeItem, for_path: EPath) {
        edge.build_path_layout(|l| {
            // • • • • •

            self.inside.props.iter().for_each(|svp| {
                // let name = svp.get().name().to_string();
                let name = svp.get_with(emg_animation::models::Property::name);
                // let name = svp.store_get_rc(&*store).name().to_string();
                match name.as_str() {
                    //TODO full this
                    "CssWidth" => {
                        l.w.set(svp.watch().cast_into());
                    }
                    _ => {
                        unimplemented!("not implemented....")
                    }
                }
            });
            // • • • • •

            (for_path, l) //TODO return bool changed or not
        })
        .unwrap();
    }

    // pub fn effecting_path(self, for_path: EPath) -> Result<Self, String> {
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
    //TODO check 是否需要 nested (有一个 CallId::current())
    #[topo::nested]
    #[must_use]
    pub fn new_in_topo(
        props: SmallVec<[StateVarProperty; PROP_SIZE]>,
        // sv_now: StateVar<Duration>,
        // edge_path: Option<(EmgEdgeItem, EPath)>,
    ) -> Self
    where
        Message: Clone + std::fmt::Debug + 'static + PartialEq,
    {
        let sv_now = global_clock();
        // let sv_now = use_state(||Duration::ZERO);
        let cb_fn_deps = props.iter().map(|p| *p.id()).collect::<Vec<_>>();
        debug!("cb_fn_deps:{:?}", &cb_fn_deps);

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

        let sv_inside: AnimationInside<Message> = AnimationInside::<Message>::new(props);

        let sa_timing = Self::set_timer(sv_now);

        let AnimationInside {
            interruption: interruption_init,
            steps: steps_init,
            props: props_init,
        } = sv_inside.clone();

        // trace!("||@@@@@@@@@@@@@@@@@@@@ step id:{:#?}", &steps_init.id());

        let mut opt_old_current: Option<Duration> = None;
        // let mut opt_old_interruption: Option<StepTimeVector<Message>> = None;
        // interruption_init.store_watch(&store)
        let pa: StateAnchor<SmallVec<[Property; PROP_SIZE]>> = props_init
            .iter()
            .map(
                |sv| sv.watch().get_anchor(), //  .get_var_with(Var::watch)
            )
            .collect::<Anchor<SmallVec<[Property; PROP_SIZE]>>>()
            .into();

        let revised: SAPropsMessageSteps2<Message> = (
            &sa_timing,
            &interruption_init.watch(),
            &pa,
            &steps_init.watch(),
        )
            // .map(|t, i, p| (*t, i.clone(), p.clone()))
            // .refmap(|t, i, p| (t, i, p))
            .map_mut(
                (
                    Vector::<(Duration, VecDeque<Step<Message>>)>::new(),
                    Rc::new(RefCell::new(VecDeque::<Step<Message>>::new())),
                    SmallVec::<[Property; PROP_SIZE]>::new(),
                    MsgBackIsNew::<Message>::default(),
                ),
                move |out, timing, interruption, props, steps| {
                    let new_t = timing.current();
                    if let Some(old_t) = &opt_old_current {
                        if old_t == new_t {
                            return false;
                        }
                    }

                    opt_old_current = Some(*new_t);
                    // • • • • •
                    // ────────────────────────────────────────────────────────────────────────────────
                    let (mut ready_interruption, queued_interruptions): (
                        StepTimeVector<Message>,
                        StepTimeVector<Message>,
                    ) = interruption
                        .clone()
                        .into_iter()
                        .map(|(wait, a_steps)| {
                            // println!("wait: {:?} , dt: {:?}", &wait, &dt);
                            (wait.saturating_sub(*timing.dt()), a_steps)
                        })
                        .partition(|(wait, _)| wait.is_zero());

                    let mut new_props = props.clone();

                    if let Some((_ /* is zero */, interrupt_steps)) = ready_interruption.pop_front()
                    {
                        new_props.iter_mut().for_each(|prop| {
                            map_to_motion(
                                &|m: &mut Motion| {
                                    *m.interpolation_override_mut() = None;
                                },
                                prop,
                            );
                        });
                        steps.replace(interrupt_steps);
                    }

                    // let mut new_steps = match ready_interruption.pop_front() {
                    //     Some((_ /* is zero */, interrupt_steps)) => {
                    //         new_props.iter_mut().for_each(|prop| {
                    //             map_to_motion(
                    //                 |m: &mut Motion| {
                    //                     *m.interpolation_override_mut() = None;
                    //                 },
                    //                 prop,
                    //             )
                    //         });

                    //         interrupt_steps
                    //     }

                    //     None => steps.clone(),
                    // };
                    //--------
                    //drop(ready_interruption);
                    let mut sent_messages = MsgBackIsNew::default();
                    resolve_steps(
                        &mut new_props,
                        &mut steps.borrow_mut(),
                        &mut sent_messages,
                        timing.dt(),
                    );
                    *out = (
                        queued_interruptions,
                        steps.clone(),
                        new_props,
                        sent_messages,
                    );
                    true

                    // out.0 = i.clone();
                    // out.1 = p.clone();
                    // true
                },
            );
        // .cutoff(move |(timing, _, _): &(Timing, _, _)| {
        //     let new_t = timing.current();
        //     if let Some(old_t) = &opt_old_current {
        //         if old_t == new_t {
        //             return false;
        //         }
        //     }

        //     // if let Some(old_interruption) = &opt_old_interruption {
        //     //     // if old_interruption.ptr_eq(new_interruption) {
        //     //     if old_interruption == new_interruption {
        //     //         return false;
        //     //     }
        //     // }

        //     opt_old_current = Some(*new_t);

        //     // opt_old_interruption = Some(new_interruption.clone());

        //     true
        // })
        // .map(|(_, i, p)| (i.clone(), p.clone()))

        // let revised: SAPropsMessageSteps2<Message> = (&sa_timing, &i_p_cut, &steps_init.watch())
        //     .map(
        //         move |//
        //               timing: &Timing,
        //               //
        //               (
        //             // timing,
        //             interruption,
        //             props,
        //         ): &(
        //             // Timing,
        //             StepTimeVector<Message>,
        //             SmallVec<[Property; PROP_SIZE]>,
        //         ),
        //               steps: &VecDeque<Step<Message>>| {
        //             //----------------------------------
        //             let (mut ready_interruption, queued_interruptions): (
        //                 StepTimeVector<Message>,
        //                 StepTimeVector<Message>,
        //             ) = interruption
        //                 .clone()
        //                 .into_iter()
        //                 .map(|(wait, a_steps)| {
        //                     // println!("wait: {:?} , dt: {:?}", &wait, &dt);
        //                     (wait.saturating_sub(*timing.dt()), a_steps)
        //                 })
        //                 .partition(|(wait, _)| wait.is_zero());

        //             let mut new_props = props.clone();

        //             let mut new_steps = match ready_interruption.pop_front() {
        //                 Some((_ /* is zero */, interrupt_steps)) => {
        //                     new_props.iter_mut().for_each(|prop| {
        //                         map_to_motion(
        //                             |m: &mut Motion| {
        //                                 *m.interpolation_override_mut() = None;
        //                             },
        //                             prop,
        //                         )
        //                     });

        //                     interrupt_steps
        //                 }

        //                 None => steps.clone(),
        //             };
        //             drop(ready_interruption);
        //             let mut sent_messages = MsgBackIsNew::default();

        //             resolve_steps(
        //                 &mut new_props,
        //                 &mut new_steps,
        //                 &mut sent_messages,
        //                 timing.dt(),
        //             );
        //             (queued_interruptions, new_steps, new_props, sent_messages)
        //         },
        //     );

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
            .map(|q, r| !q.is_empty() || !r.borrow().is_empty());

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

        sv_now.insert_after_fn_in_topo(
            move |skip, _| {
                // println!("call update after set timing {:?}", v);
                debug!("-> [insert_after_fn] calling --> topo id:{:?}", &id);
                // anima_clone.update_in_callback(skip);
                if !sa_running_clone.get() {
                    debug!("not running , return");
                    return;
                }
                debug!("after callback running ");

                //TODO remove clone, 每一次都克隆 比较重 , get_with? sized?
                // let revised_value = revised.get();
                revised.store_get_with(&state_store().borrow(), |(a, _b, c, _d)| {
                    props_init.iter().zip(c.iter()).for_each(|(sv, prop)| {
                        sv.seting_in_b_a_callback(skip, move || prop.clone());
                    });
                    interruption_init.set(a.clone());
                    #[cfg(test)]
                    {
                        let s = steps_init.get();
                        debug_assert_eq!(&s, _b);
                    }

                    //NOTE debug_assert_eq!(&s, b) is no panic, steps_init no need set ⇣
                    // steps_init.set(b.clone());
                });
                // props_init
                //     .iter()
                //     .zip(revised_value.2.iter())
                //     .for_each(|(sv, prop)| sv.seting_in_b_a_callback(skip, prop));
                // interruption_init.seting_in_b_a_callback(skip, &revised_value.0);
                // steps_init.seting_in_b_a_callback(skip, &revised_value.1);
            },
            false,
            &cb_fn_deps,
        );

        // .unwrap_or_else(|_| panic!("find same id already in after_fn map \n id:{:?}", &id));

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

    pub fn interrupt(&self, steps: impl Into<VecDeque<Step<Message>>>) {
        self.inside.interruption.update(|interruption| {
            let steps_vd = steps.into();
            // trace!("steps_vd: {steps_vd:#?}");

            interruption.push_front(extract_initial_wait(steps_vd));
            // trace!("Interrupt: {new_interruption:#?}");
        });
    }
    pub fn replace(&self, steps: impl Into<VecDeque<Step<Message>>>) {
        let steps_vd = steps.into();
        self.inside.interruption.set(
            // trace!("steps_vd: {steps_vd:#?}");
            vector!(extract_initial_wait(steps_vd)),
        );
    }

    /// # Panics
    /// temp fn ,
    /// Will panic if p not prop
    #[must_use]
    pub fn get_position(&self, prop_index: usize) -> Precision {
        self.inside.props[prop_index].get_with(|p| match p {
            Property::Prop(_name, m) => m.position().into_inner(),
            _ => todo!("not implemented"),
        })
    }
}

// impl<Message> Drop for AnimationE<Message>
// where
//     Message: Clone + std::fmt::Debug + 'static + PartialEq,
// {
//     fn drop(&mut self) {
//         let count = self.ref_count.get();
//         debug!("===============in Dropping  AnimationE count:{}", &count);

//         if count <= 1 {
//             G_CLOCK.with(|clock| {
//                 // self.running = StateAnchor::constant(false);
//                 let _span = debug_span!("clock.remove_after_fn").entered();
//                 clock.remove_after_fn(self.id);
//             });
//         } else {
//             self.ref_count.set(count - 1);
//             debug!("===============after count:{}", &self.ref_count.get());
//         }
//         // let clock = global_clock();
//     }
// }
#[cfg(test)]
mod tests {
    extern crate test;

    use std::path::Path;
    use std::time::Duration;

    use emg::{edge_index, edge_index_no_source, node_index, Edge, EdgeIndex};
    use emg_animation::{interrupt, models::Property, opacity, style, to};
    use emg_common::{animation::Tick, im::vector, into_smvec, smallvec, IdStr};
    use emg_state::{
        state_store, topo, use_state, CloneState, CloneStateAnchor, Dict, GStateStore, StateVar,
    };
    use seed_styles as styles;
    use styles::{pc, width};
    use styles::{px, CssWidth};

    use crate::{animation::global_clock, tests::tracing_init};
    use crate::{EPath, EdgeItemNode, EmgEdgeItem, GraphEdgesDict};

    use super::AnimationE;
    use tracing::{debug, debug_span, trace_span, warn, warn_span, Level};

    // use tracing_flame::FlameLayer;
    // use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    #[allow(dead_code)]
    #[derive(Debug, Clone, PartialEq)]
    enum Message {
        A,
    }

    use test::{black_box, Bencher};

    #[bench]
    fn bench_nom_am(b: &mut Bencher) {
        b.iter(|| {
            let mut am = style::<Message>(into_smvec![width(px(1))]);
            nom_am_run(&mut am);
            black_box(());
        });
    }
    #[test]
    fn nom_am() {
        let mut am = style::<Message>(into_smvec![width(px(1))]);
        nom_am_run(&mut am);
    }

    fn nom_am_run(am: &mut emg_animation::models::Animation<Message>) {
        interrupt(
            [
                to(into_smvec![width(px(0))]),
                to(into_smvec![width(px(1))]),
                to(into_smvec![width(px(0))]),
                to(into_smvec![width(px(1))]),
                to(into_smvec![width(px(0))]),
                to(into_smvec![width(px(1))]),
                to(into_smvec![width(px(0))]),
                to(into_smvec![width(px(1))]),
                to(into_smvec![width(px(0))]),
                to(into_smvec![width(px(1))]),
                to(into_smvec![width(px(0))]),
                to(into_smvec![width(px(1))]),
                to(into_smvec![width(px(0))]),
                to(into_smvec![width(px(1))]),
                to(into_smvec![width(px(0))]),
                to(into_smvec![width(px(1))]),
                to(into_smvec![width(px(0))]),
                to(into_smvec![width(px(1))]),
                to(into_smvec![width(px(0))]),
            ],
            am,
        );
        for i in 1002..2000 {
            emg_animation::update(Tick(Duration::from_millis(i * 16)), am);
            let _e = am.get_position(0);
            println!("pos: {_e}");
        }
        let _e = am.get_position(0);
        println!("pos: {_e}");
    }

    #[bench]
    #[topo::nested]

    fn bench_less_state_am(b: &mut Bencher) {
        // let ei = edge_index_no_source("fff");
        // let source = use_state(||ei.source_nix().as_ref().cloned());
        // let target = use_state(||ei.target_nix().as_ref().cloned());
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
            let a: AnimationE<Message> = AnimationE::new_in_topo(into_smvec![width(px(1))]);
            less_am_run(&state_store().borrow(), &a, &sv_now);
            black_box(());
        });
    }

    fn less_am_run(
        storeref: &GStateStore,
        a: &AnimationE<Message>,
        sv_now: &emg_state::StateVar<Duration>,
    ) {
        a.interrupt([
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
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
        // let source = use_state(||ei.source_nix().as_ref().cloned());
        // let target = use_state(||ei.target_nix().as_ref().cloned());
        // let edge_item: EmgEdgeItem<String> = EmgEdgeItem::default_with_wh_in_topo(
        //     source.watch(),
        //     target.watch(),
        //     StateAnchor::constant(Dict::default()),
        //     1920,
        //     1080,
        // );
        // let sv_now = use_state(||Duration::ZERO);
        let sv_now = global_clock();

        b.iter(move || {
            sv_now.set(Duration::from_millis(0));
            // let edge_item1 = edge_item.clone();
            let w = use_state(|| width(px(1)));
            let a: AnimationE<Message> = AnimationE::new_in_topo(into_smvec![w]);
            // AnimationE::new_in_topo(into_smvec![width(px(1))], sv_now);
            many_am_run(&a, &sv_now);
            black_box(());
        });
    }

    #[test]
    #[topo::nested]
    fn many() {
        let _g = tracing_init();

        // let sv_now = use_state(||Duration::ZERO);
        let sv_now = global_clock();
        sv_now.set(Duration::from_millis(0));
        let w1 = use_state(|| width(px(2)));
        let w2 = use_state(|| width(px(99)));
        debug!("===================================main loop ");
        let a: AnimationE<Message> = AnimationE::new_in_topo(into_smvec![w1]);
        a.interrupt([to(into_smvec![width(px(0))])]);

        debug!("===================================main loop--2 ");

        let b: AnimationE<Message> = AnimationE::new_in_topo(into_smvec![w2]);
        b.interrupt([to(into_smvec![width(px(888))])]);
        sv_now.set(Duration::from_millis(16));
        debug!("a====:\n {:#?}", a.inside.props[0].get());
        debug!("b====:\n {:#?}", b.inside.props[0].get());
        sv_now.set(Duration::from_millis(33));
        debug!("a 33====:\n {:#?}", a.inside.props[0].get());
        debug!("b 33====:\n {:#?}", b.inside.props[0].get());
        #[cfg(feature = "insta")]
        insta::assert_debug_snapshot!("many-33-a", &a);
        #[cfg(feature = "insta")]
        insta::assert_debug_snapshot!("many-33-b", &b);
    }

    #[test]
    #[topo::nested]
    fn many_for() {
        let _g = tracing_init();

        // let sv_now = use_state(||Duration::ZERO);
        let sv_now = global_clock();
        // let edge_item1 = edge_item.clone();
        sv_now.set(Duration::from_millis(0));

        debug!("===================================main loop ");
        let a: AnimationE<Message> = AnimationE::new_in_topo(into_smvec![width(px(0.5))]);
        for _i in 0..4 {
            many_am_run_for_test(&a, &sv_now);
            black_box(());
        }
    }

    fn many_am_run_for_test(
        // storeref: &GStateStore,
        a: &AnimationE<Message>,
        sv_now: &emg_state::StateVar<Duration>,
    ) {
        a.interrupt([
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(2))]),
            to(into_smvec![width(px(2.3))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
            // to(into_smvec![width(px(1))]),
            // to(into_smvec![width(px(0))]),
        ]);
        warn!("set time ---------------------------------------------------- 0");
        // sv_now.set(Duration::from_millis(0));
        // sv_now.store_set(storeref, Duration::from_millis(16));
        // a.update();
        // for i in 1002..1004 {
        for i in 1..60 {
            // warn!(
            // "in loop: set time ---------------------------------------------------- loop:{}",
            // &i
            // );
            // sv_now.set(Duration::from_millis(0));

            sv_now.set(Duration::from_millis(i * 16));
            // sv_now.store_set(storeref, Duration::from_millis(i * 16));
            // a.update();
            // a.inside.props[0].store_get(storeref);
            // if i % 10 == 0 {
            let _e = a.inside.props[0].get();
            warn!("i: {i}, pos: {_e}");
            // }
        }
        let _e = a.inside.props[0].get();
        warn!("end pos: {_e}");

        // a.inside.props[0].store_get(storeref);
    }

    fn many_am_run(
        // storeref: &GStateStore,
        a: &AnimationE<Message>,
        sv_now: &emg_state::StateVar<Duration>,
    ) {
        a.interrupt([
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            to(into_smvec![width(px(0))]),
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
            // if i % 40 == 0 {
            let _e = a.inside.props[0].get();

            // }
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
            // let source = use_state(||ei.source_nix().as_ref().cloned());
            // let target = use_state(||ei.target_nix().as_ref().cloned());
            // let edge_item: EmgEdgeItem<String> = EmgEdgeItem::default_with_wh_in_topo(
            //     source.watch(),
            //     target.watch(),
            //     StateAnchor::constant(Dict::default()),
            //     1920,
            //     1080,
            // );

            let sv_now = global_clock();
            sv_now.set(Duration::from_millis(0));

            let a: AnimationE<Message> = AnimationE::new_in_topo(into_smvec![opacity(1.)]);
            // println!("a:{:#?}", &a);
            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("new", &a);
            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("new2", &a);
            assert!(!a.running.get());
            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("get_running", &a);
            // println!("now set interrupt");
            a.interrupt([
                to(smallvec![emg_animation::opacity(0.)]),
                to(smallvec![emg_animation::opacity(1.)]),
            ]);
            // println!("over interrupt");

            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("interrupt", &a);
            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("interrupt2", &a);
            // println!("over interrupt insta");

            assert!(a.running.get());
            // println!("over interrupt running.get()");
            // a.update_animation();
            // ────────────────────────────────────────────────────────────────────────────────

            sv_now.set(Duration::from_millis(16));
            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("set16", &a);
            // a.update();
            // println!("set timing 16");

            // println!("set timing 16-- update");

            // println!("1**{:?}", a.inside.props.get());

            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("updated_16_a_0", &a);
            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("updated_16_a_1", &a);
            // println!("set timing 16-- insta");
            // ────────────────────────────────────────────────────────────────────────────────
            sv_now.set(Duration::from_millis(16));
            // a.update();
            // println!("set timing 16-2");

            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("updated_16_b_0", &a);
            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("updated_16_b_1", &a);
            // println!("set timing 16-- insta-2");
            // ─────────────────────────────────────────────────────────────────

            sv_now.set(Duration::from_millis(33));
            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("set33", &a);
            // a.update();
            // println!("set timing 33");

            // println!("....set 2 ");

            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("updated_33_0", &a);

            // println!("set timing 33 -- update 1");

            // a.update();
            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("updated_33_1", &a);

            // println!("set timing 33 -- update 2");

            // println!("2**{:?}", a.inside.props.get());

            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("snap_updated_33_0", &a);
            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("snap_updated_33_1", &a);
            // println!("set timing 33 -- insta  ");

            // sv_now.set(Duration::from_millis(2));
            // a.update_animation();

            for i in 3..200 {
                sv_now.set(Duration::from_millis(i * 16));
                // a.update();
                // println!("in ------ i:{}", &i);
                // a.timing.get();
                // println!("3***{:?}", a.inside.props.get());
                a.inside.props[0].get();
            }
            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("updated_end_0", &a);
            #[cfg(feature = "insta")]
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
        let _nn = tracing_init();

        insta::with_settings!({snapshot_path => Path::new("./layout_am")}, {

                    let css_w: StateVar<CssWidth> = use_state(||width(px(1)));

                    // let span = trace_span!("am-test");
                    // let _guard = span.enter();
                    // trace!("fff");

                    let e_dict_sv:StateVar<GraphEdgesDict> = use_state(Dict::new);

                    let root_e_source =use_state(|| None);
                    let root_e_target = use_state(||Some(node_index("root")));
                    let  root_e = EmgEdgeItem::default_with_wh_in_topo(&root_e_source.watch(),& root_e_target.watch(),e_dict_sv.watch(),1920, 1080);
                    // e_dict_sv.set_with(|d|{
                    //     let mut nd = d .clone();
                    //     nd.insert(EdgeIndex::new(None,node_index("root")), Edge::new(root_e_source, root_e_target, root_e.clone()));
                    //     nd
                    // });

                    // let e1_source =use_state(|| Some(node_index("root")));
                    // let e1_target = use_state(||Some(node_index("1")));
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
                    #[cfg(feature="insta")]
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
                        // AnimationEdge::new_in_topo(into_smvec![width(px(1))], e1, sv_now);
                        AnimationE::new_in_topo(into_smvec![css_w]);
                        a.effecting_edge_path( &root_e,EPath(vector![edge_index_no_source("root")]));
                    // println!("a:{:#?}", &a);
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("new", &a);
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("new2", &a);
                    let new1 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__new.snap")).unwrap();
                    let new2 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__new2.snap")).unwrap();
                    assert_eq!(new1.contents(),new2.contents());

                    assert!(!a.running.get());
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("get_running", &a);
                    // println!("now set interrupt");
                    a.interrupt([
                        to(into_smvec![width(px(0))]),
                        to(into_smvec![width(px(1))])
                    ]);
                    // println!("over interrupt");

                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("interrupt", &a);

                    // println!("over interrupt insta");

                    assert!(a.running.get());
                    // println!("over interrupt running.get()");
                    // a.update_animation();
                    // ────────────────────────────────────────────────────────────────────────────────

                    sv_now.set(Duration::from_millis(16));
                    // println!("set timing 16");
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("set16", &a);

                    // a.update();
                    // println!("set timing 16-- update");

                    // println!("1**{:?}", a.inside.props.get());

                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("updated_16_0", &a);
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("updated_16_0-edge", &root_e);

                    // println!("set timing 16-- insta");
                    // ────────────────────────────────────────────────────────────────────────────────
                    sv_now.set(Duration::from_millis(16));
                    // println!("set timing 16-2");

                    // a.update();

                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("updated_16_0-2", &a);

                    // println!("set timing 16-- insta-2");
                    let u16 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__updated_16_0.snap")).unwrap();
                    let u16_2 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__updated_16_0-2.snap")).unwrap();
                    assert_eq!(u16.contents(),u16_2.contents());
                    // ─────────────────────────────────────────────────────────────────

                    sv_now.set(Duration::from_millis(33));
                    // println!("set timing 33");

                    // println!("....set 2 ");
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("set33", &a);

                    // a.update();
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("updated_33_0", &a);
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("updated_33_0-edge", &root_e);

                    // println!("set timing 33 -- update 1");

                    // a.update();
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("updated_33_1", &a);

                    // println!("set timing 33 -- update 2");

                    // println!("2**{:?}", a.inside.props.get());

                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("snap_updated_33_0", &a);

                    // println!("set timing 33 -- insta  ");
                    let f33 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__updated_33_1.snap")).unwrap();
                    let f33_2 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__snap_updated_33_0.snap")).unwrap();
                    assert_eq!(f33.contents(),f33_2.contents());

                    // sv_now.set(Duration::from_millis(2));
                    // a.update_animation();


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
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("updated_end_0", &a);


                    // // println!("{:?}", a.revised_props.get());
                    // // state_store().borrow().engine_mut().stabilize();
                    // println!("end : {:?}", a.inside.props.get());
                    // println!("{:?}", a);
                    // a.inside.props[0].get();
                    // ─────────────────────────────────────────────────────────────────

                    css_w.set(width(px(20)));
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("end_set1", &a);
                    sv_now.set(sv_now.get() + Duration::from_millis(16));
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("end_set2-settime", &a);
                    // a.update();
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("end_set3-update", &a);

                });
    }

    #[bench]
    #[topo::nested]

    fn anima_macro_bench(b: &mut Bencher) {
        b.iter(move || {
            anima_macro_for_bench();
            black_box(());
        });
    }

    #[test]
    #[topo::nested]
    fn anima_macro_for_bench_2_test() {
        let _g = tracing_init();

        anima_macro_for_bench();
        global_clock().set(Duration::from_millis(0));

        anima_macro_for_bench();
    }
    #[test]
    #[topo::nested]
    fn anima_macro_for_2_test() {
        let _g = tracing_init();

        warn!("run anima_macro first time");
        anima_macro();
        warn!("set time 0====================================================");
        anima_macro();
    }
    // #[test]
    // #[topo::nested]
    // fn anchor_err_test() {
    //     let _g = tracing_init();
    //     let _span = debug_span!("anchors-dirty").entered();
    //     warn!("run anima_macro first time");

    //     anima_macro_for_anchor_error();
    //     warn!("set time 0");
    //     global_clock().set(Duration::from_millis(0));

    //     anima_macro_for_anchor_error();
    // }

    #[test]
    #[topo::nested]
    fn anima_macro_for_anchor_error() {
        let _g = tracing_init();
        let sv_now = global_clock();
        sv_now.set(Duration::from_millis(0));

        let css_w: StateVar<CssWidth> = use_state(|| width(px(1)));
        let a: AnimationE<Message> = anima![css_w];

        let e_dict_sv: StateVar<GraphEdgesDict> = use_state(Dict::new);
        let root_e_source = use_state(|| None);
        let root_e_target = use_state(|| Some(node_index("root")));
        let root_e = EmgEdgeItem::default_with_wh_in_topo(
            &root_e_source.watch(),
            &root_e_target.watch(),
            e_dict_sv.watch(),
            1920,
            1080,
        );
        a.effecting_edge_path(&root_e, EPath(vector![edge_index_no_source("root")]));
        a.interrupt([to(into_smvec![width(px(0))]), to(into_smvec![width(px(1))])]);

        for i in 1..100 {
            sv_now.set(Duration::from_millis(i * 16));
            if i == 1 {
                #[cfg(feature = "insta")]
                insta::assert_debug_snapshot!("anima_macro_16", &a);
                #[cfg(feature = "insta")]
                insta::assert_debug_snapshot!("anima_macro_16_edge", &root_e);
            }
            // a.update();
            // println!("in ------ i:{}", &i);
            // a.timing.get();
            debug!("prop current : {:?}", a.inside.props[0].get());
            a.inside.props[0].get();
        }
    }

    #[test]
    #[topo::nested]
    fn anima_macro_for_bench() {
        // let _g = _init();
        let sv_now = global_clock();
        sv_now.set(Duration::from_millis(0));
        let css_w: StateVar<CssWidth> = use_state(|| width(px(1)));
        let a: AnimationE<Message> = anima![css_w];

        let e_dict_sv: StateVar<GraphEdgesDict> = use_state(Dict::new);
        let root_e_source = use_state(|| None);
        let root_e_target = use_state(|| Some(node_index("root")));
        let root_e = EmgEdgeItem::default_with_wh_in_topo(
            &root_e_source.watch(),
            &root_e_target.watch(),
            e_dict_sv.watch(),
            1920,
            1080,
        );
        a.effecting_edge_path(&root_e, EPath(vector![edge_index_no_source("root")]));
        a.interrupt([
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
            to(into_smvec![width(px(0))]),
        ]);
        for i in 1..1000 {
            sv_now.set(Duration::from_millis(i * 16));
            a.inside.props[0].get();
        }
    }
    #[test]
    #[topo::nested]
    fn anima_macro() {
        // let _g = tracing_init();
        let sv_now = global_clock();
        sv_now.set(Duration::from_millis(0));

        let css_w: StateVar<CssWidth> = use_state(|| width(px(1)));
        let a: AnimationE<Message> = anima![css_w];
        debug!("will assert_debug_snapshot a");
        #[cfg(feature = "insta")]
        insta::assert_debug_snapshot!("anima_macro_init", &a);

        let e_dict_sv: StateVar<GraphEdgesDict> = use_state(Dict::new);
        let root_e_source = use_state(|| None);
        let root_e_target = use_state(|| Some(node_index("root")));
        let root_e = EmgEdgeItem::default_with_wh_in_topo(
            &root_e_source.watch(),
            &root_e_target.watch(),
            e_dict_sv.watch(),
            1920,
            1080,
        );
        a.effecting_edge_path(&root_e, EPath(vector![edge_index_no_source("root")]));
        a.interrupt([to(into_smvec![width(px(0))]), to(into_smvec![width(px(1))])]);
        #[cfg(feature = "insta")]
        insta::assert_debug_snapshot!("anima_macro_interrupt", &a);

        for i in 1..100 {
            sv_now.set(Duration::from_millis(i * 16));
            if i == 1 {
                #[cfg(feature = "insta")]
                insta::assert_debug_snapshot!("anima_macro_16", &a);
                #[cfg(feature = "insta")]
                insta::assert_debug_snapshot!("anima_macro_16_edge", &root_e);
            }
            // a.update();
            // println!("in ------ i:{}", &i);
            // a.timing.get();
            // debug!("prop current : {:?}", a.inside.props[0].get());
            a.inside.props[0].get();
        }
    }

    #[test]
    #[topo::nested]
    fn test_layout_children_anima() {
        // ! layout am
        let _nn = tracing_init();

        insta::with_settings!({snapshot_path => Path::new("./layout_children_am")}, {

                    let css_w: StateVar<CssWidth> = use_state(||width(px(1)));

                    // let span = trace_span!("am-test");
                    // let _guard = span.enter();
                    // trace!("fff");

                    let e_dict_sv:StateVar<GraphEdgesDict> = use_state(Dict::new);

                    let root_e_source =use_state(|| None);
                    let root_e_target = use_state(||Some(node_index("root")));
                    let root_e = EmgEdgeItem::default_with_wh_in_topo(&root_e_source.watch(), &root_e_target.watch(),e_dict_sv.watch(),1920, 1080);
                    e_dict_sv.set_with_once(|d|{
                        let mut nd = d .clone();
                        nd.insert(EdgeIndex::new(None,node_index("root")), Edge::new(root_e_source, root_e_target, root_e.clone()));
                        nd
                    });

                    let e1_source =use_state(|| Some(node_index("root")));
                    let e1_target = use_state(||Some(node_index("1")));
                    let e1 = EmgEdgeItem::new_in_topo(
                            &e1_source.watch(),
                            &e1_target.watch(),
                        e_dict_sv.watch(),
                        (px(50), px(50)),
                         (pc(0), pc(0), pc(0)),
                          (pc(50), pc(50), pc(50)),
                    );

                    e_dict_sv.set_with_once(|d|{
                        let mut nd = d .clone();
                        nd.insert(edge_index("root","1"), Edge::new(e1_source, e1_target, e1.clone()));
                        nd
                    });

                    // ─────────────────────────────────────────────────────────────────

                    let ew = root_e.layout.w;
                    // debug!("e->{}",&e1);
                    #[cfg(feature="insta")]
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
                        // AnimationEdge::new_in_topo(into_smvec![width(px(1))], e1, sv_now);
                        AnimationE::new_in_topo(into_smvec![css_w]);
                        a.effecting_edge_path(&e1,EPath(vector![edge_index_no_source("root"),edge_index("root","1")]));

                    // println!("a:{:#?}", &a);
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("new", &a);
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("new2", &a);
                    let new1 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__new.snap")).unwrap();
                    let new2 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__new2.snap")).unwrap();
                    assert_eq!(new1.contents(),new2.contents());

                    assert!(!a.running.get());
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("get_running", &a);
                    // println!("now set interrupt");
                    a.interrupt([
                        to(into_smvec![width(px(0))]),
                        to(into_smvec![width(px(1))])
                    ]);
                    // println!("over interrupt");

                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("interrupt", &a);

                    // println!("over interrupt insta");

                    assert!(a.running.get());
                    // println!("over interrupt running.get()");
                    // a.update_animation();
                    // ────────────────────────────────────────────────────────────────────────────────

                    sv_now.set(Duration::from_millis(16));
                    // println!("set timing 16");
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("set16", &a);

                    // a.update();
                    // println!("set timing 16-- update");

                    // println!("1**{:?}", a.inside.props.get());

                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("updated_16_0", &a);
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("updated_16_0-edge", &e1);

                    // println!("set timing 16-- insta");
                    // ────────────────────────────────────────────────────────────────────────────────
                    sv_now.set(Duration::from_millis(16));
                    // println!("set timing 16-2");

                    // a.update();

                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("updated_16_0-2", &a);

                    // println!("set timing 16-- insta-2");
                    let u16 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__updated_16_0.snap")).unwrap();
                    let u16_2 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__updated_16_0-2.snap")).unwrap();
                    assert_eq!(u16.contents(),u16_2.contents());
                    // ─────────────────────────────────────────────────────────────────

                    sv_now.set(Duration::from_millis(33));
                    // println!("set timing 33");

                    // println!("....set 2 ");
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("set33", &a);

                    // a.update();
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("updated_33_0", &a);
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("updated_33_0-edge", &e1);

                    // println!("set timing 33 -- update 1");

                    // a.update();
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("updated_33_1", &a);

                    // println!("set timing 33 -- update 2");

                    // println!("2**{:?}", a.inside.props.get());

                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("snap_updated_33_0", &a);

                    // println!("set timing 33 -- insta  ");
                    let f33 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__updated_33_1.snap")).unwrap();
                    let f33_2 = insta::Snapshot::from_file(Path::new("./src/layout_am/emg_layout__animation__tests__snap_updated_33_0.snap")).unwrap();
                    assert_eq!(f33.contents(),f33_2.contents());

                    // sv_now.set(Duration::from_millis(2));
                    // a.update_animation();


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
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("updated_end_0", &a);


                    // // println!("{:?}", a.revised_props.get());
                    // // state_store().borrow().engine_mut().stabilize();
                    // println!("end : {:?}", a.inside.props.get());
                    // println!("{:?}", a);
                    // a.inside.props[0].get();
                    // ─────────────────────────────────────────────────────────────────

                    css_w.set(width(px(20)));
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("end_set1", &a);
                    sv_now.set(sv_now.get() + Duration::from_millis(16));
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("end_set2-settime", &a);
                    // a.update();
                    #[cfg(feature="insta")]
        insta::assert_debug_snapshot!("end_set3-update", &a);

                });
    }
}
