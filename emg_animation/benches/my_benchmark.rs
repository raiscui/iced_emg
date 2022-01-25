/*
 * @Author: Rais
 * @Date: 2022-01-20 09:35:37
 * @LastEditTime: 2022-01-25 09:27:49
 * @LastEditors: Rais
 * @Description:
 */

mod need {

    use emg_animation::models::Property;
    use emg_core::{tiny_vec, SmallVec};

    use smallvec::smallvec;
    use tracing::warn;
    pub fn zip_properties_greedy_mut_8(
        initial_props: &mut SmallVec<[Property; 8]>,
        mut new_target_props: SmallVec<[Property; 8]>,
    ) -> SmallVec<[Option<Property>; 8]> {
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
        initial_props: &mut SmallVec<[Property; 3]>,
        mut new_target_props: SmallVec<[Property; 3]>,
    ) -> SmallVec<[Option<Property>; 3]> {
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
use std::time::Duration;
const PROP_SIZE: usize = 3;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use emg_animation::{
    init_motion,
    models::{
        step, step_sm_mut, zip_properties_greedy, zip_properties_greedy_mut, PropName, Property,
        PropertySM,
    },
};
use emg_core::{into_smvec, into_vector, tiny_vec, vector, IdStr, SmallVec, Vector};
use need::{zip_properties_greedy_mut_3, zip_properties_greedy_mut_8};
use seed_styles::{height, px, width, Unit};
use smallvec::smallvec;

pub fn clone_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone-vecs");

    group
        .significance_level(0.1)
        .warm_up_time(Duration::from_secs(6))
        .sample_size(1000);

    group.bench_function("SmallVec-init-clone-3", |b| {
        let initial_props: SmallVec<[Property; 3]> = into_smvec![
            width(px(black_box(2))),
            width(px(black_box(0))),
            width(px(black_box(1)))
        ];

        b.iter(|| {
            let x = initial_props.clone();
        })
    });
    group.bench_function("Vec-init-clone-3", |b| {
        let initial_props: Vec<Property> = vec![
            width(px(black_box(2))).into(),
            width(px(black_box(0))).into(),
            width(px(black_box(1))).into(),
        ];

        b.iter(|| {
            let x = initial_props.clone();
        })
    });
    group.bench_function("im_Vec-init-clone-3", |b| {
        let initial_props: Vector<Property> = vector![
            width(px(black_box(2))).into(),
            width(px(black_box(0))).into(),
            width(px(black_box(1))).into(),
        ];

        b.iter(|| {
            let x = initial_props.clone();
        })
    });
    group.bench_function("SmallVec-clone-3", |b| {
        b.iter(|| {
            let initial_props: SmallVec<[Property; 3]> = into_smvec![
                width(px(black_box(2))),
                width(px(black_box(0))),
                width(px(black_box(1)))
            ];
            let x = initial_props.clone();
        })
    });
    group.bench_function("Vec-clone-3", |b| {
        b.iter(|| {
            let initial_props: Vec<Property> = vec![
                width(px(black_box(2))).into(),
                width(px(black_box(0))).into(),
                width(px(black_box(1))).into(),
            ];
            let x = initial_props.clone();
        })
    });
    group.bench_function("im_Vec-clone-3", |b| {
        b.iter(|| {
            let initial_props: Vector<Property> = vector![
                width(px(black_box(2))).into(),
                width(px(black_box(0))).into(),
                width(px(black_box(1))).into(),
            ];
            let x = initial_props.clone();
        })
    });

    group.finish();
}
pub fn step_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("step");

    group
        .significance_level(0.1)
        .warm_up_time(Duration::from_secs(6))
        .sample_size(1000);

    group.bench_function("step1-clone", |b| {
        b.iter(|| {
            let p = Property::Prop(
                PropName::new(IdStr::new_inline(black_box("xx"))),
                init_motion(black_box(100f64), Unit::Px),
            );
            let vp = vector![p];

            step(Duration::from_millis(black_box(16)), vp);
        })
    });
    // group.bench_function("step2-mut", |b| {
    //     let p = Property::Prop(
    //         PropName::new(IdStr::new_inline(black_box("xx"))),
    //         init_motion(black_box(100f64), Unit::Px),
    //     );
    //     let mut vp = vector![p];
    //     b.iter(|| {
    //         step_mut(Duration::from_millis(black_box(16)), &mut vp);
    //     })
    // });
    group.bench_function("step2-mut-smallvec", |b| {
        let p = PropertySM::Prop(
            PropName::new(IdStr::new_inline(black_box("xx"))),
            init_motion(black_box(100f64), Unit::Px),
        );
        let mut vp: SmallVec<[PropertySM; PROP_SIZE]> = smallvec![p];
        b.iter(|| {
            step_sm_mut(Duration::from_millis(black_box(16)), &mut vp);
        })
    });

    group.finish();
}

pub fn zip_properties_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("zip_properties");

    group
        .significance_level(0.1)
        .warm_up_time(Duration::from_secs(6))
        .sample_size(1000);

    group.bench_function("mut_tinyvec", |b| {
        b.iter(|| {
            let mut initial_props: SmallVec<[PropertySM; PROP_SIZE]> =
                into_smvec![width(px(2)), width(px(0)), width(px(1))];
            let new_target_props: SmallVec<[PropertySM; PROP_SIZE]> =
                into_smvec![height(px(1)), width(px(0)), width(px(1))];
            zip_properties_greedy_mut(&mut initial_props, new_target_props);
        })
    });

    group.bench_function("mut_tinyvec_has_1", |b| {
        b.iter(|| {
            let mut initial_props: SmallVec<[PropertySM; PROP_SIZE]> = into_smvec![width(px(2))];
            let new_target_props: SmallVec<[PropertySM; PROP_SIZE]> = into_smvec![width(px(0))];
            zip_properties_greedy_mut(&mut initial_props, new_target_props);
        })
    });
    // group.bench_function("mut_tinyvec_1", |b| {
    //     b.iter(|| {
    //         let mut initial_props: SmallVec<[Property; 1]> =
    //             into_smvec![width(px(2)), width(px(0)), width(px(1))];
    //         let new_target_props: SmallVec<[Property; 1]> =
    //             into_smvec![height(px(1)), width(px(0)), width(px(1))];
    //         zip_properties_greedy_mut_1::<1>(&mut initial_props, new_target_props);
    //     })
    // });
    group.bench_function("mut_tinyvec_3-3", |b| {
        b.iter(|| {
            let mut initial_props: SmallVec<[Property; 3]> =
                into_smvec![width(px(2)), width(px(0)), width(px(1))];
            let new_target_props: SmallVec<[Property; 3]> =
                into_smvec![height(px(1)), width(px(0)), width(px(1))];
            zip_properties_greedy_mut_3(&mut initial_props, new_target_props);
        })
    });
    group.bench_function("mut_tinyvec_3-1", |b| {
        b.iter(|| {
            let mut initial_props: SmallVec<[Property; 3]> = into_smvec![width(px(2))];
            let new_target_props: SmallVec<[Property; 3]> = into_smvec![width(px(0))];
            zip_properties_greedy_mut_3(&mut initial_props, new_target_props);
        })
    });
    group.bench_function("mut_tinyvec_8-3", |b| {
        b.iter(|| {
            let mut initial_props: SmallVec<[Property; 8]> =
                into_smvec![width(px(2)), width(px(0)), width(px(1))];
            let new_target_props: SmallVec<[Property; 8]> =
                into_smvec![height(px(1)), width(px(0)), width(px(1))];
            zip_properties_greedy_mut_8(&mut initial_props, new_target_props);
        })
    });
    // group.bench_function("def", |b| {
    //     b.iter(|| {
    //         let initial_props: Vector<Property> =
    //             into_vector![width(px(2)), width(px(0)), width(px(1))];
    //         let new_target_props: Vector<Property> =
    //             into_vector![height(px(1)), width(px(0)), width(px(1))];
    //         zip_properties_greedy(initial_props, new_target_props);
    //     })
    // });

    group.finish();
}

criterion_group!(benches, zip_properties_benchmark);
// criterion_group!(
//     benches,
//     clone_benchmark,
//     step_benchmark,
//     zip_properties_benchmark
// );
criterion_main!(benches);
