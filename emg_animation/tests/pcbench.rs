/*
 * @Author: Rais
 * @Date: 2022-05-17 11:30:44
 * @LastEditTime: 2023-01-30 09:40:18
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
#![allow(unused)]
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod pc_bench {
    use easybench::{bench, bench_env};

    #[derive(Debug, Clone, PartialEq)]
    #[allow(dead_code)]
    enum Message {
        A,
    }

    use std::{
        cell::RefCell,
        collections::VecDeque,
        rc::Rc,
        time::{Duration, Instant},
    };
    // const PROP_SIZE: usize = 1;
    use emg_animation::{
        init_motion, loop_am,
        models::{
            color::{fill, Color},
            resolve_steps, step, zip_properties_greedy_mut, MsgBackIsNew, PropName, Property, Step,
        },
        to, PROP_SIZE,
    };
    use emg_common::{im::vector, into_smvec, into_vector, smallvec, IdStr, SmallVec, Vector};
    use seed_styles::{height, px, width, Unit};

    #[test]
    fn benchmark() {
        let props2: SmallVec<[Property; PROP_SIZE]> = into_smvec![width(px(1))];
        let steps2: VecDeque<Step<Message>> = [loop_am([
            to(into_smvec![width(px(0))]),
            to(into_smvec![width(px(1))]),
        ])]
        .into();

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
