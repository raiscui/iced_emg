/*
 * @Author: Rais
 * @Date: 2022-01-20 09:35:37
 * @LastEditTime: 2023-01-13 16:57:28
 * @LastEditors: Rais
 * @Description:
 */
//
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum Message {
    A,
}

#[cfg(not(target_arch = "wasm32"))]
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use emg_animation::{
    init_motion, loop_am,
    models::{
        color::{fill, Color},
        resolve_steps, step, zip_properties_greedy_mut, MsgBackIsNew, PropName, Property, Step,
    },
    to, PROP_SIZE,
};
use emg_common::{into_smvec, into_vector, smallvec, vector, IdStr, SmallVec, Vector};
use seed_styles::{height, px, width, Unit};
use std::{collections::VecDeque, time::Duration};

pub fn step_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("step");

    group
        .significance_level(0.1)
        .warm_up_time(Duration::from_secs(6))
        .sample_size(1000);

    group.bench_function("step2-mut-smallvec", |b| {
        let p = Property::Prop(
            PropName::new(IdStr::new_inline(black_box("xx"))),
            init_motion(black_box(100f64), Unit::Px),
        );
        let mut vp: SmallVec<[Property; PROP_SIZE]> = smallvec![p];
        b.iter(|| {
            step(&Duration::from_millis(black_box(16)), &mut vp);
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

    group.bench_function("mut_tinyvec-color-sm", |b| {
        b.iter(|| {
            let mut initial_props: SmallVec<[Property; PROP_SIZE]> =
                into_smvec![fill(Color::new(0, 0, 0, 0.))];
            let new_target_props: SmallVec<[Property; PROP_SIZE]> =
                into_smvec![fill(Color::new(1, 1, 1, 1.))];
            zip_properties_greedy_mut(&mut initial_props, new_target_props);
        })
    });
    group.bench_function("mut_tinyvec-color-sm-has-init", |b| {
        let initial_props: SmallVec<[Property; PROP_SIZE]> =
            into_smvec![fill(Color::new(0, 0, 0, 0.))];
        let new_target_props: SmallVec<[Property; PROP_SIZE]> =
            into_smvec![fill(Color::new(1, 1, 1, 1.))];
        b.iter(|| {
            zip_properties_greedy_mut(&mut initial_props.clone(), new_target_props.clone());
        })
    });

    group.bench_function("mut_tinyvec", |b| {
        b.iter(|| {
            let mut initial_props: SmallVec<[Property; PROP_SIZE]> =
                into_smvec![width(px(2)), width(px(0)), width(px(1))];
            let new_target_props: SmallVec<[Property; PROP_SIZE]> =
                into_smvec![height(px(1)), width(px(0)), width(px(1))];
            zip_properties_greedy_mut(&mut initial_props, new_target_props);
        })
    });

    group.bench_function("mut_tinyvec_has_1", |b| {
        b.iter(|| {
            let mut initial_props: SmallVec<[Property; PROP_SIZE]> = into_smvec![width(px(2))];
            let new_target_props: SmallVec<[Property; PROP_SIZE]> = into_smvec![width(px(0))];
            zip_properties_greedy_mut(&mut initial_props, new_target_props);
        })
    });

    group.finish();
}

pub fn resolve_steps_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolve_steps");

    group
        .significance_level(0.1)
        .warm_up_time(Duration::from_secs(6))
        .sample_size(500)
        .measurement_time(Duration::from_secs(3));

    // ─────────────────────────────────────────────────────────────────

    let props2: SmallVec<[Property; PROP_SIZE]> = into_smvec![width(px(black_box(1)))];
    let steps2: VecDeque<Step<Message>> = [loop_am([
        to(into_smvec![width(px(black_box(0)))]),
        to(into_smvec![width(px(black_box(1)))]),
    ])]
    .into();
    // ─────────────────────────────────────────────────────────────────

    group.bench_function("resolve_steps-mut-once", |b| {
        b.iter_batched(
            || {
                (
                    props2.clone(),
                    steps2.clone(),
                    MsgBackIsNew::<Message>::default(),
                )
            },
            |(mut p, mut i, mut m)| {
                resolve_steps(
                    &mut p,
                    &mut i,
                    &mut m,
                    &Duration::from_millis(black_box(16)),
                );
            },
            BatchSize::PerIteration,
        )
    });

    group.finish();
}

// criterion_group!(benches, resolve_steps_benchmark);
#[cfg(not(target_arch = "wasm32"))]
criterion_group!(
    benches,
    // clone_benchmark,
    // step_benchmark,
    // zip_properties_benchmark,
    resolve_steps_benchmark
);
#[cfg(not(target_arch = "wasm32"))]
criterion_main!(benches);
