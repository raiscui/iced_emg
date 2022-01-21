/*
 * @Author: Rais
 * @Date: 2022-01-20 09:35:37
 * @LastEditTime: 2022-01-21 12:42:25
 * @LastEditors: Rais
 * @Description:
 */

use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use emg_animation::{
    init_motion,
    models::{
        step, step_mut, zip_properties_greedy, zip_properties_greedy_mut, PropName, Property,
    },
};
use emg_core::{into_tvec, into_vector, vector, IdStr, TinyVec, Vector};
use seed_styles::{height, px, width, Unit};

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
    group.bench_function("step2-mut", |b| {
        b.iter(|| {
            let p = Property::Prop(
                PropName::new(IdStr::new_inline(black_box("xx"))),
                init_motion(black_box(100f64), Unit::Px),
            );
            let mut vp = vector![p];

            step_mut(Duration::from_millis(black_box(16)), &mut vp);
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
            let mut initial_props: TinyVec<[Property; 2]> =
                into_tvec![width(px(2)), width(px(0)), width(px(1))];
            let new_target_props: TinyVec<[Property; 2]> =
                into_tvec![height(px(1)), width(px(0)), width(px(1))];
            zip_properties_greedy_mut(&mut initial_props, new_target_props);
        })
    });
    group.bench_function("def", |b| {
        b.iter(|| {
            let initial_props: Vector<Property> =
                into_vector![width(px(2)), width(px(0)), width(px(1))];
            let new_target_props: Vector<Property> =
                into_vector![height(px(1)), width(px(0)), width(px(1))];
            zip_properties_greedy(initial_props, new_target_props);
        })
    });

    group.finish();
}

criterion_group!(benches, zip_properties_benchmark, step_benchmark);
criterion_main!(benches);
