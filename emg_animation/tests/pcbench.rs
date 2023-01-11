/*
 * @Author: Rais
 * @Date: 2022-05-17 11:30:44
 * @LastEditTime: 2023-01-11 17:24:23
 * @LastEditors: Rais
 * @Description:
 */
/*
 * @Author: Rais
 * @Date: 2022-05-16 17:12:23
 * @LastEditTime: 2022-05-16 19:19:38
 * @LastEditors: Rais
 * @Description:
 */
/*
 * @Author: Rais
 * @Date: 2022-01-20 09:35:37
 * @LastEditTime: 2022-05-16 15:50:07
 * @LastEditors: Rais
 * @Description:
 */
#![allow(unused_imports)]
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod pc_bench {
    use easybench::{bench, bench_env};

    #[derive(Debug, Clone, PartialEq)]
    enum Message {
        A,
    }
    mod need {

        use emg_animation::models::PropertyOG;
        use emg_common::{smallvec, SmallVec};
        use tracing::warn;
        pub fn zip_properties_greedy_mut_8(
            initial_props: &mut SmallVec<[PropertyOG; 8]>,
            mut new_target_props: SmallVec<[PropertyOG; 8]>,
        ) -> SmallVec<[Option<PropertyOG>; 8]> {
            // println!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@===================");
            // for x in initial_props.iter() {
            //     println!("{}", x);
            // }
            // println!(".................................");

            // for x in new_target_props.iter() {
            //     println!("{}", x);
            // }
            // println!("---------------------------------------------------");
            initial_props.sort_by(|left, right| left.name().cmp(&right.name()));
            new_target_props.sort_by(|left, right| left.name().cmp(&right.name()));
            // for x in initial_props.iter() {
            //     println!("{}", x);
            // }
            // println!(".................................");

            // for x in new_target_props.iter() {
            //     println!("{}", x);
            // }
            // println!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@===================");

            let mut res_props = smallvec![];
            let mut b_iter = new_target_props.into_iter().peekable();

            for current_a in initial_props.iter() {
                let a_name = current_a.name();

                loop {
                    if let Some(current_b) = b_iter.peek() {
                        if current_b.name() < a_name {
                            b_iter.next();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                if let Some(current_b) = b_iter.peek() {
                    if current_b.name() == a_name {
                        res_props.push(b_iter.next());
                        continue;
                    }
                }
                res_props.push(None);
            }

            for b in b_iter {
                warn!(
                    "{} has no initial value and therefore will not be animated.",
                    b.name()
                );
            }
            res_props
        }
        pub fn zip_properties_greedy_mut_3(
            initial_props: &mut SmallVec<[PropertyOG; 3]>,
            mut new_target_props: SmallVec<[PropertyOG; 3]>,
        ) -> SmallVec<[Option<PropertyOG>; 3]> {
            // println!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@===================");
            // for x in initial_props.iter() {
            //     println!("{}", x);
            // }
            // println!(".................................");

            // for x in new_target_props.iter() {
            //     println!("{}", x);
            // }
            // println!("---------------------------------------------------");
            initial_props.sort_by(|left, right| left.name().cmp(&right.name()));
            new_target_props.sort_by(|left, right| left.name().cmp(&right.name()));
            // for x in initial_props.iter() {
            //     println!("{}", x);
            // }
            // println!(".................................");

            // for x in new_target_props.iter() {
            //     println!("{}", x);
            // }
            // println!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@===================");

            let mut res_props = smallvec![];
            let mut b_iter = new_target_props.into_iter().peekable();

            for current_a in initial_props.iter() {
                let a_name = current_a.name();

                loop {
                    if let Some(current_b) = b_iter.peek() {
                        if current_b.name() < a_name {
                            b_iter.next();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                if let Some(current_b) = b_iter.peek() {
                    if current_b.name() == a_name {
                        res_props.push(b_iter.next());
                        continue;
                    }
                }
                res_props.push(None);
            }

            for b in b_iter {
                warn!(
                    "{} has no initial value and therefore will not be animated.",
                    b.name()
                );
            }
            res_props
        }
    }
    use std::{
        cell::RefCell,
        collections::VecDeque,
        rc::Rc,
        time::{Duration, Instant},
    };
    // const PROP_SIZE: usize = 1;
    use emg_animation::{
        fill, init_motion, loop_am, loop_am_og,
        models::{
            color::{fill_sm, Color},
            resolve_steps, resolve_steps_og, step, step_og, zip_properties_greedy_mut,
            zip_properties_greedy_og, MsgBackIsNew, PropName, Property, PropertyOG, Step, StepOG,
        },
        to, to_og, PROP_SIZE,
    };
    use emg_common::{into_smvec, into_vector, smallvec, vector, IdStr, SmallVec, Vector};
    use need::{zip_properties_greedy_mut_3, zip_properties_greedy_mut_8};
    use seed_styles::{height, px, width, Unit};

    #[test]
    fn benchmark() {
        let initial_props: Vector<PropertyOG> = into_vector![width(px(1))];
        let steps: Vector<StepOG<Message>> = vector![to_og(into_vector![width(px(0))])];

        let props2: SmallVec<[Property; PROP_SIZE]> = into_smvec![width(px(1))];
        let steps2: VecDeque<Step<Message>> = [loop_am([
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
        ])]
        .into();

        println!(
            "fib 500: {}",
            bench(|| {
                let (ps, _, ss) = resolve_steps_og(
                    initial_props.clone(),
                    steps.clone(),
                    &Duration::from_millis(16),
                );
            })
        );
        println!(
            "new fib 500: {}",
            bench_env(
                (
                    props2.clone(),
                    steps2.clone(),
                    MsgBackIsNew::<Message>::default(),
                ),
                |(p, i, m)| {
                    let f = resolve_steps(p, i, m, &Duration::from_millis(16));
                }
            )
        );
    }
}
