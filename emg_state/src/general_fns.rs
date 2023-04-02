/*
 * @Author: Rais
 * @Date: 2023-03-28 17:09:19
 * @LastEditTime: 2023-03-29 14:34:32
 * @LastEditors: Rais
 * @Description:
 */

use std::{cell::RefCell, panic::Location, rc::Rc, thread::AccessError};

use anchors::singlethread::Anchor;
use emg_common::smallvec;
use tracing::{debug, debug_span, instrument, trace, warn};

use crate::{
    error::Error,
    g_store::{
        BoxSynCallAfterFn, BoxSynCallBeforeFn, GStateStore, RCRSynCallAfterFnsMap,
        RCRSynCallBeforeFnsMap, SkipKeyCollection, G_STATE_STORE,
    },
    general_struct::{StorageKey, TopoKey},
    general_traits::{StateFn, VTFn},
};

// ─────────────────────────────────────────────────────────────────────────────

#[track_caller]
pub(crate) fn or_insert_var_with_topo_id<VT: VTFn<VOA> + 'static, VOA, F: FnOnce() -> VOA>(
    func: F,
    current_id: TopoKey,
) {
    state_store_with(
        #[track_caller]
        |g_state_store_refcell| {
            trace!(target:"G_STATE_STORE","G_STATE_STORE::borrow_mut:\n{}", Location::caller());

            g_state_store_refcell
                .borrow_mut()
                .or_insert_var_with_key::<VT, VOA, _>(func, &StorageKey::TopoKey(current_id));
        },
    );
}

#[must_use]
pub fn state_store() -> Rc<RefCell<GStateStore>> {
    state_store_with(std::clone::Clone::clone)
}

#[inline]
#[instrument(target = "G_STATE_STORE", name = "state_store_with", skip_all)]
pub fn state_store_with<F, R>(f: F) -> R
where
    F: FnOnce(&Rc<RefCell<GStateStore>>) -> R,
{
    G_STATE_STORE.with(f)
}
#[inline]
#[instrument(target = "G_STATE_STORE", name = "state_store_try_with", skip_all)]
pub fn state_store_try_with<F, R>(f: F) -> Result<R, AccessError>
where
    F: FnOnce(&Rc<RefCell<GStateStore>>) -> R,
{
    G_STATE_STORE.try_with(f)
}
// ─────────────────────────────────────────────────────────────────────────────

///
///  Uses the current topological id to create a new state accessor
///
pub(crate) fn state_exists_for_topo_id<VT: 'static>(id: TopoKey) -> bool {
    state_store_with(|g_state_store_refcell| {
        trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

        g_state_store_refcell
            .borrow()
            .state_exists_with_id::<VT>(StorageKey::TopoKey(id))
    })
}

pub(crate) fn before_fns_run<VOA>(
    // store: &GStateStore,
    current_topo_key: TopoKey,
    current_data: &Option<Rc<VOA>>, //TODO 是否需要 opt? 看上去好像不需要
    data: &VOA,
    opt_fns: Option<&RCRSynCallBeforeFnsMap<VOA>>,
) -> SkipKeyCollection {
    let mut skip: SkipKeyCollection = smallvec![current_topo_key];
    if let Some(rcr_fns) = opt_fns {
        let fns = rcr_fns.borrow();

        for (_deps, f) in fns.values() {
            f(&mut skip, current_data, data);
        }
        // fns.values()
        // .for_each(|bf_func| bf_func(&skip, current_data, data));
    }

    skip
}
pub(crate) fn after_fns_run<VOA>(
    // store: &GStateStore,
    mut skip: SkipKeyCollection,
    data: &VOA,
    opt_fns: Option<&RCRSynCallAfterFnsMap<VOA>>,
) {
    // let mut new_set = HashSet::default();
    // new_set.insert(*current_id);
    if let Some(rcr_fns) = opt_fns {
        let fns = rcr_fns.borrow();

        debug!("after_fns len:{}", fns.len());

        for (_deps, f) in fns.values() {
            f(&mut skip, data);
        }
    }

    // fns.values().for_each(|af_func| af_func(skip, data));
}

pub(crate) fn remove_after_fn<VOA: 'static>(id: TopoKey, after_key: &StorageKey) {
    let fns = state_store_with(|g_state_store_refcell| {
        trace!("G_STATE_STORE::borrow_mut:\n{}", Location::caller());

        let _span = debug_span!("G_STATE_STORE::remove_after_fn->borrow_mut").entered();

        g_state_store_refcell
            .borrow()
            .get_after_fns_map::<VOA>(&StorageKey::TopoKey(id))
    });
    fns.borrow_mut().remove(after_key);
}
// ─────────────────────────────────────────────────────────────────────────────

#[track_caller]
//TODO replace use global_engine_get_anchor_val_with - Clone
pub(crate) fn global_engine_get_anchor_val<T: Clone + 'static>(anchor: &Anchor<T>) -> T {
    trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

    state_store_with(|g_state_store_refcell| g_state_store_refcell.borrow().engine_get(anchor))
}

#[track_caller]
pub(crate) fn try_global_engine_get_anchor_val<T: Clone + 'static>(
    anchor: &Anchor<T>,
) -> Result<T, Error> {
    trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

    state_store_with(|g_state_store_refcell| g_state_store_refcell.borrow().try_engine_get(anchor))
}

#[track_caller]
pub(crate) fn global_engine_get_anchor_val_with<T: Clone + 'static, F: FnOnce(&T) -> R, R>(
    anchor: &Anchor<T>,
    func: F,
) -> R {
    trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

    state_store_with(|g_state_store_refcell| {
        g_state_store_refcell.borrow().engine_get_with(anchor, func)
    })
}
// ─────────────────────────────────────────────────────────────────────────────

/// 添加不添加 deps 都不会使 after before func 循环,
/// 但是添加deps 可以 在 deps-TopoKey 的StateVar drop时候 将该func drop,
/// 同时如果 运行func时刻, skip有deps的key,则不会运行该func
/// 如果 deps is some , 则返回 none , Rc储存在deps中
#[topo::nested]
pub(crate) fn insert_before_fn_common_in_topo<
    SV: StateFn<VOA>,
    VOA: 'static,
    VT: VTFn<VOA> + 'static,
>(
    sv: &SV,
    func: BoxSynCallBeforeFn<VOA>,
    init: bool,
    deps: &[TopoKey],
) -> Option<Rc<StorageKey>> {
    assert!(!deps.contains(sv.id()), "deps can't contain self");

    state_store_with(|g_state_store_refcell| {
        if init {
            trace!("G_STATE_STORE::borrow:\n{}", Location::caller());
            let store = &g_state_store_refcell.borrow();
            let var = store
                .opt_get_var_use_id::<VT>(&StorageKey::TopoKey(*sv.id()))
                .unwrap();
            // .expect("set_state_with_key: can't set state that doesn't exist in this context!");

            let mut skip = smallvec![*sv.id()];

            func(&mut skip, &None, &*var.get());
            // let v = &(*var.get());
            // before_fns_run(store, &StorageKey::TopoKey(sv.id), v, before_fns);
            // if v.clone() != (*var.get()).clone() {
            //     panic!("not same");
            // }
        }
        trace!("G_STATE_STORE::borrow_mut:\n{}", Location::caller());

        let mut store = g_state_store_refcell.borrow_mut();
        let fk = store
            .insert_before_fn_in_topo(&StorageKey::TopoKey(*sv.id()), func, deps)
            .unwrap();

        if deps.is_empty() {
            Some(fk)
        } else {
            for d in deps {
                store.link_callback_drop(*d, fk.clone());
            }
            drop(fk);
            None
        }
    })
}

#[topo::nested]
pub(crate) fn insert_after_fn_common_in_topo<VOA: 'static, VT: VTFn<VOA> + 'static>(
    key: &TopoKey,
    func: BoxSynCallAfterFn<VOA>,
    init: bool,
    deps: &[TopoKey],
) -> Option<Rc<StorageKey>> {
    assert!(!deps.contains(key), "deps can't contain self");

    state_store_with(|g_state_store_refcell| {
        if init {
            trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

            let store = &g_state_store_refcell.borrow();
            let var = store
                .opt_get_var_use_id::<VT>(&StorageKey::TopoKey(*key))
                .unwrap();

            let mut skip = smallvec![*key];

            func(&mut skip, &*var.get());

            // after_fns_run(store, &skip, &(*var.get()), after_fns);
        }
        trace!("G_STATE_STORE::borrow_mut:\n{}", Location::caller());

        let mut store = g_state_store_refcell.borrow_mut();
        let fk = store
            .insert_after_fn_in_topo::<VOA>(&StorageKey::TopoKey(*key), func, deps)
            .unwrap();

        if deps.is_empty() {
            Some(fk)
        } else {
            for d in deps {
                store.link_callback_drop(*d, fk.clone());
            }
            drop(fk);
            None
        }
    })
}

pub(crate) fn start_set_var_and_run_before_after<VT: VTFn<VOA>, VOA: Clone>(
    // store: &GStateStore,
    current_topo_key: TopoKey,
    var: &VT,
    current: Rc<VOA>,
    data: &VOA,
    before_fns: Option<&RCRSynCallBeforeFnsMap<VOA>>,
    after_fns: Option<&RCRSynCallAfterFnsMap<VOA>>,
) {
    debug!(
        ?current_topo_key,
        "[start_set_var_and_run_before_after] type: {}",
        std::any::type_name::<VOA>()
    );

    //NOTE staring first callback call
    let skip: SkipKeyCollection =
        before_fns_run(current_topo_key, &Some(current), data, before_fns);
    trace!("start_set_var_and_before_after:: before_fns_runned,now set new v");

    var.set(data.clone());
    trace!("start_set_var_and_before_after:: after_fns_run");

    // after_fns_run(&skip.clone(), &data.clone(), &after_fns.clone());
    after_fns_run(skip, data, after_fns);
}
