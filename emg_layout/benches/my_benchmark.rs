/*
 * @Author: Rais
 * @Date: 2022-01-20 09:35:37
 * @LastEditTime: 2022-07-27 15:01:05
 * @LastEditors: Rais
 * @Description:
 */

use std::{collections::VecDeque, time::Duration};
const PROP_SIZE: usize = 3;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use emg_animation::{
    fill, init_motion, interrupt, interrupt_og, loop_am, loop_am_og,
    models::{
        color::{fill_sm, Color},
        resolve_steps, resolve_steps_og, step, step_og, update_animation, update_animation_og,
        zip_properties_greedy_mut, zip_properties_greedy_og, Animation, MsgBackIsNew, PropName,
        Property, PropertyOG, Step, StepOG,
    },
    opacity, opacity_og, replace, replace_og, style, style_og, to, to_og, AmState, AmStateOG, Tick,
};
use emg_common::{into_smvec, into_vector, smallvec, vector, IdStr, SmallVec, Vector};
use emg_layout::{global_clock, AnimationE};
use emg_state::{topo, CloneStateVar};
use seed_styles::{height, px, width, Unit};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    A,
}

pub fn ame_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("animationE");

    group
        .significance_level(0.1)
        .warm_up_time(Duration::from_secs(6))
        .sample_size(500)
        .measurement_time(Duration::from_secs(10));

    // ────────────────────────────────────────────────────────────────────────────────

    group.bench_function("animation-get", |b| {
        b.iter(|| {
            let mut am_state: AmState<Message> = style(smallvec![opacity(1.)]);
            let mut now = Duration::from_millis(10000);

            interrupt(
                [loop_am([
                    to(smallvec![opacity(0.)]),
                    to(smallvec![opacity(1.)]),
                ])],
                &mut am_state,
            );

            now += Duration::from_millis(16);
            update_animation(Tick(now), &mut am_state);
        })
    });
    group.bench_function("animation-og-get", |b| {
        b.iter(|| {
            let mut am_state: AmStateOG<Message> = style_og(vector![opacity_og(1.)]);
            let mut now = Duration::from_millis(10000);

            interrupt_og(
                vector![loop_am_og(vector![
                    to_og(vector![emg_animation::opacity_og(0.)]),
                    to_og(vector![emg_animation::opacity_og(1.)])
                ]),],
                &mut am_state,
            );

            now += Duration::from_millis(16);
            update_animation_og(Tick(now), &mut am_state);
        })
    });

    // ────────────────────────────────────────────────────────────────────────────────

    group.bench_function("animationE-get", |b| {
        let sv_now = global_clock();

        topo::call(move || {
            b.iter(|| {
                let a: AnimationE<Message> = AnimationE::new_in_topo(into_smvec![opacity(1.)]);

                a.interrupt([loop_am([
                    to(smallvec![opacity(0.)]),
                    to(smallvec![opacity(1.)]),
                ])]);

                sv_now.set_with(|t| {
                    (*t).checked_add(Duration::from_millis(16))
                        .unwrap_or(Duration::ZERO)
                });
            })
        });
    });

    // ────────────────────────────────────────────────────────────────────────────────

    group.finish();
}
pub fn ame_initd_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("animationE-initd");

    group
        .significance_level(0.1)
        .warm_up_time(Duration::from_secs(6))
        .sample_size(500)
        .measurement_time(Duration::from_secs(10));

    // ────────────────────────────────────────────────────────────────────────────────

    // ────────────────────────────────────────────────────────────────────────────────

    group.bench_function("animation-initd-get", |b| {
        let mut am_state: AmState<Message> = style(smallvec![opacity(1.)]);
        let mut now = Duration::from_millis(10000);

        replace(
            [loop_am([
                to(smallvec![opacity(0.)]),
                to(smallvec![opacity(1.)]),
            ])],
            &mut am_state,
        );
        b.iter(|| {
            now += Duration::from_millis(16);
            update_animation(Tick(now), &mut am_state);
        })
    });
    group.bench_function("animation-og-initd-get", |b| {
        let mut am_state: AmStateOG<Message> = style_og(vector![opacity_og(1.)]);
        let mut now = Duration::from_millis(10000);

        replace_og(
            vector![loop_am_og(vector![
                to_og(vector![emg_animation::opacity_og(0.)]),
                to_og(vector![emg_animation::opacity_og(1.)])
            ])],
            &mut am_state,
        );
        b.iter(|| {
            now += Duration::from_millis(16);
            update_animation_og(Tick(now), &mut am_state);
        })
    });
    group.bench_function("animationE-initd-get", |b| {
        let sv_now = global_clock();

        let a: AnimationE<Message> = AnimationE::new_in_topo(into_smvec![opacity(1.)]);

        a.replace([loop_am([to![opacity(0.)], to![opacity(1.)]])]);

        b.iter(|| {
            sv_now.set_with(|t| {
                (*t).checked_add(Duration::from_millis(16))
                    .unwrap_or(Duration::ZERO)
            });
        })
    });

    group.finish();
}

pub fn ame_new_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("animationE-initd-new");

    group
        .significance_level(0.1)
        .warm_up_time(Duration::from_secs(6))
        .sample_size(500)
        .measurement_time(Duration::from_secs(10));

    // ────────────────────────────────────────────────────────────────────────────────

    // ────────────────────────────────────────────────────────────────────────────────

    group.bench_function("animationE-initd-get", |b| {
        let sv_now = global_clock();

        let a: AnimationE<Message> = AnimationE::new_in_topo(into_smvec![opacity(1.)]);

        a.replace([loop_am([to![opacity(0.)], to![opacity(1.)]])]);

        b.iter(|| {
            sv_now.set_with(|t| {
                (*t).checked_add(Duration::from_millis(16))
                    .unwrap_or(Duration::ZERO)
            });
            a.get_position(0);
        })
    });
}
pub fn ame_old_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("animationE-initd-old");

    group
        .significance_level(0.1)
        .warm_up_time(Duration::from_secs(6))
        .sample_size(500)
        .measurement_time(Duration::from_secs(10));

    // ────────────────────────────────────────────────────────────────────────────────

    group.finish();
}

criterion_group!(benches, ame_initd_benchmark);
// criterion_group!(benches, ame_old_benchmark);
// criterion_group!(benches, ame_new_benchmark);
criterion_main!(benches);
