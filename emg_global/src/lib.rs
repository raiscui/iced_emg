use std::time::{Duration, Instant};

use emg_state::{anchors::im::Vector, use_state, Anchor, CloneState, StateAnchor, StateVar};
use static_init::dynamic;

#[dynamic]
pub static G_START: std::time::Instant = Instant::now();

thread_local! {
   static G_ELAPSED: StateVar<Duration> = use_state(||Duration::ZERO);
}

thread_local! {
    static G_ANIMA_RUNNING_STATE: StateVar<Vector<Anchor<bool>>> = use_state(Vector::new);
}
thread_local! {
    static G_AM_RUNING: StateAnchor<bool> = global_anima_running_build();
}

#[must_use]
pub fn global_elapsed() -> StateVar<Duration> {
    G_ELAPSED.with(|c| *c)
}
pub fn global_elapsed_set(now: Duration) {
    G_ELAPSED.with(|c| c.set(now));
}

pub fn global_anima_running_add(running: &StateAnchor<bool>) {
    G_ANIMA_RUNNING_STATE.with(|sv| sv.update(|v| v.push_back(running.get_anchor())));
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
fn global_anima_running_build() -> StateAnchor<bool> {
    let watch: Anchor<Vector<bool>> = G_ANIMA_RUNNING_STATE.with(|am| am.watch().anchor().into());
    watch.map(|list: &Vector<bool>| list.contains(&true)).into()
}

// ─────────────────────────────────────────────────────────────────────────────

thread_local! {
    static G_WIDTH: StateVar<f64> = use_state(||0.);
}
#[must_use]
pub fn global_width() -> StateVar<f64> {
    G_WIDTH.with(|sv| *sv)
}
thread_local! {
    static G_HEIGHT: StateVar<f64> = use_state(||0.);
}
#[must_use]
pub fn global_height() -> StateVar<f64> {
    G_HEIGHT.with(|sv| *sv)
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    // use super::*;
}
