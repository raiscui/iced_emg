/*
 * @Author: Rais
 * @Date: 2021-03-15 17:10:47
 * @LastEditTime: 2023-03-04 12:45:55
 * @LastEditors: Rais
 * @Description:
 */

mod collections;
pub use anchors::collections::ord_map_methods::Dict;
pub use anchors::dict;
pub use anchors::singlethread::Anchor;
pub use anchors::singlethread::Engine;
pub use anchors::singlethread::Var;
use anchors::{
    expert::{cutoff, map, map_mut, refmap, then, AnchorInner},
    singlethread::MultiAnchor,
};
use emg_common::{smallvec, SVec::ToSmallVec, SmallVec, TypeCheck, TypeName, Vector};
use tracing::{debug, debug_span, instrument, warn};

use std::{
    hash::{BuildHasherDefault, Hash},
    ops::Deref,
    panic::Location,
    rc::Weak,
};
// use im::HashMap;
use std::{cell::RefCell, clone::Clone, marker::PhantomData, rc::Rc};
use tracing::{trace, trace_span};
// use delegate::delegate;
// use anchors::collections::
use slotmap::{DefaultKey, Key, SlotMap, SparseSecondaryMap};
// ────────────────────────────────────────────────────────────────────────────────

thread_local! {
    pub(crate) static G_STATE_STORE: Rc<RefCell<GStateStore>> =Rc::new( RefCell::new(
        GStateStore::default()
    ));
}

// ────────────────────────────────────────────────────────────────────────────────
//TODO build benchmark for AHash FxHash
// use ahash::AHashMap as HashMap;
// use ahash::AHasher as CustomHasher;
use indexmap::IndexMap as HashMap;
use weak_table::WeakKeyHashMap;
// use rustc_hash::FxHashMap as HashMap;
use emg_hasher::CustomHasher;
// use std::collections::HashMap;

use crate::error::Error;
// ────────────────────────────────────────────────────────────────────────────────
pub type SkipKeyCollection = SmallVec<[TopoKey; 3]>;

type BoxSynCallAfterFn<T> = Box<dyn Fn(&mut SkipKeyCollection, &T)>;
type BoxSynCallBeforeFn<T> = Box<dyn Fn(&mut SkipKeyCollection, &Option<Rc<T>>, &T)>;

pub type DepsVarTopoKey = SmallVec<[TopoKey; 1]>;

type FnKey = Weak<StorageKey>;

type SynCallAfterFnsMap<T> =
    WeakKeyHashMap<FnKey, (DepsVarTopoKey, BoxSynCallAfterFn<T>), BuildHasherDefault<CustomHasher>>;
type SynCallBeforeFnsMap<T> = WeakKeyHashMap<
    FnKey,
    (DepsVarTopoKey, BoxSynCallBeforeFn<T>),
    BuildHasherDefault<CustomHasher>,
>;
// NOTE: rc RefCell 为了 可以移出borrowmut store block后操作
type RCRSynCallAfterFnsMap<T> = Rc<RefCell<SynCallAfterFnsMap<T>>>;
type RCRSynCallBeforeFnsMap<T> = Rc<RefCell<SynCallBeforeFnsMap<T>>>;

type VarOptBAfnCollectRef<'a, T> = (
    &'a Var<T>,
    Option<&'a RCRSynCallBeforeFnsMap<T>>,
    Option<&'a RCRSynCallAfterFnsMap<T>>,
);

type VarSecMap<T> = SparseSecondaryMap<DefaultKey, Var<T>, BuildHasherDefault<CustomHasher>>;
type VarBeforeSecMap<T> =
    SparseSecondaryMap<DefaultKey, RCRSynCallBeforeFnsMap<T>, BuildHasherDefault<CustomHasher>>;
type VarAfterSecMap<T> =
    SparseSecondaryMap<DefaultKey, RCRSynCallAfterFnsMap<T>, BuildHasherDefault<CustomHasher>>;

type VarDepRequireSecMap =
    SparseSecondaryMap<DefaultKey, SmallVec<[Rc<StorageKey>; 2]>, BuildHasherDefault<CustomHasher>>;

// ─────────────────────────────────────────────────────────────────────────────

#[allow(clippy::module_name_repetitions)]
pub struct GStateStore {
    anymap: anymap::AnyMap,
    id_to_key_map: HashMap<StorageKey, DefaultKey, BuildHasherDefault<CustomHasher>>,
    // pub id_to_key_map: HashMap<StorageKey, DefaultKey>,
    primary_slotmap: SlotMap<DefaultKey, StorageKey>,
    b_a_fn_drop_link_map: VarDepRequireSecMap,
    engine: RefCell<Engine>,
}

impl Drop for GStateStore {
    fn drop(&mut self) {
        debug!("GStateStore drop");
    }
}

impl std::fmt::Debug for GStateStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the graph store")
    }
}

impl Default for GStateStore {
    fn default() -> Self {
        //TODO with capacity
        Self {
            anymap: anymap::AnyMap::new(),
            id_to_key_map: HashMap::default(),
            primary_slotmap: SlotMap::new(),
            b_a_fn_drop_link_map: VarDepRequireSecMap::default(),
            //256
            engine: RefCell::new(Engine::new_with_max_height(256)),
        }
    }
}

// type VarBAfnCollect<T> = (Var<T>, SynCallBeforeFnsMap<T>, SynCallAfterFnsMap<T>);

impl GStateStore {
    /// # Panics
    ///
    /// Will panic if engine cannot `borrow_mut`
    #[track_caller]
    fn engine_get<O: Clone + 'static>(&self, anchor: &Anchor<O>) -> O {
        trace!("engine_get: {}", &std::any::type_name::<O>());
        let _g = trace_span!("-> enging_get", "type: {}", &std::any::type_name::<O>()).entered();

        self.engine.try_borrow_mut().map_or_else(
            |err| {
                let x = &*illicit::expect::<LocationEngineGet>();
                panic!(
                    "can't borrow_mut engine err: {} , for anchor type: {},\n at:{}",
                    &err,
                    &std::any::type_name::<O>(),
                    x.0
                )
            },
            |mut e| {
                let _gg = trace_span!(
                    "-> enging_get:engine borrow_muted , now getting.. ",
                    "type: {}",
                    &std::any::type_name::<O>()
                )
                .entered();

                e.get(anchor)
            },
        )
    }

    /// # Panics
    ///
    /// Will panic if engine cannot `borrow_mut`
    #[track_caller]
    fn try_engine_get<O: Clone + 'static>(&self, anchor: &Anchor<O>) -> Result<O, Error> {
        self.engine.try_borrow_mut().map_or_else(
            |err| {
                let x = *illicit::expect::<LocationEngineGet>().deref();

                #[cfg(feature = "engine-try-borrow-mut-no-panic")]
                {
                    let e = format!(
                        "can't borrow_mut engine err: {} , for anchor type: {}",
                        &err,
                        &std::any::type_name::<O>(),
                    );
                    Err(Error::EngineAlreadyMut(x, e))
                }
                #[cfg(not(feature = "engine-try-borrow-mut-no-panic"))]
                {
                    panic!(
                        "can't borrow_mut engine err: {} , for anchor type: {},\n at:{}",
                        &err,
                        &std::any::type_name::<O>(),
                        x.0
                    )
                }
            },
            |mut e| {
                let _gg = trace_span!(
                    "-> enging_get:engine borrow_muted , now getting.. ",
                    "type: {}",
                    &std::any::type_name::<O>()
                )
                .entered();

                Ok(e.get(anchor))
            },
        )
    }

    fn engine_get_with<O: Clone + 'static, F: FnOnce(&O) -> R, R>(
        &self,
        anchor: &Anchor<O>,
        func: F,
    ) -> R {
        trace!("engine_get_with: {}", &std::any::type_name::<O>());
        let _g = trace_span!(
            "-> engine_get_with",
            "type: {}",
            &std::any::type_name::<O>()
        )
        .entered();

        self.engine.try_borrow_mut().map_or_else(
            |err| {
                panic!(
                    "can't borrow_mut engine err: {} , for anchor type: {}",
                    &err,
                    &std::any::type_name::<O>()
                )
            },
            |mut e| {
                let _gg = trace_span!(
                    "-> engine_get_with:engine borrow_muted , now getting.. ",
                    "type: {}",
                    &std::any::type_name::<O>()
                )
                .entered();

                e.get_with(anchor, func)
            },
        )
    }

    fn state_exists_with_id<T: 'static>(&self, id: StorageKey) -> bool {
        match (self.id_to_key_map.get(&id), self.get_secondarymap::<T>()) {
            (Some(existing_key), Some(existing_secondary_map)) => {
                existing_secondary_map.contains_key(*existing_key)
            }
            (_, _) => false,
        }
    }

    #[must_use]
    fn get_secondarymap<T: 'static>(&self) -> Option<&VarSecMap<T>> {
        self.anymap.get::<VarSecMap<T>>()
    }
    fn remove_secondarymap<T: 'static>(&mut self) {
        self.anymap.remove::<VarSecMap<T>>();
    }

    fn get_before_secondarymap<T: 'static>(&self) -> Option<&VarBeforeSecMap<T>> {
        self.anymap.get::<VarBeforeSecMap<T>>()
    }
    fn remove_before_secondarymap<T: 'static>(&mut self) {
        self.anymap.remove::<VarBeforeSecMap<T>>();
    }
    fn get_after_secondarymap<T: 'static>(&self) -> Option<&VarAfterSecMap<T>> {
        self.anymap.get::<VarAfterSecMap<T>>()
    }
    fn remove_after_secondarymap<T: 'static>(&mut self) {
        self.anymap.remove::<VarAfterSecMap<T>>();
    }
    fn get_mut_secondarymap<T: 'static>(&mut self) -> Option<&mut VarSecMap<T>> {
        self.anymap.get_mut::<VarSecMap<T>>()
    }
    fn get_mut_before_secondarymap<T: 'static>(&mut self) -> Option<&mut VarBeforeSecMap<T>> {
        trace!(
            "-- get_mut_before_secondarymap: {}",
            &std::any::type_name::<T>()
        );
        self.anymap.get_mut::<VarBeforeSecMap<T>>()
    }
    fn get_mut_after_secondarymap<T: 'static>(&mut self) -> Option<&mut VarAfterSecMap<T>> {
        self.anymap.get_mut::<VarAfterSecMap<T>>()
    }

    fn register_secondarymap<T: 'static>(&mut self) {
        let sm: VarSecMap<T> = VarSecMap::<T>::default();
        self.anymap.insert(sm);
    }
    fn register_before_secondarymap<T: 'static>(&mut self) {
        let sm: VarBeforeSecMap<T> = VarBeforeSecMap::<T>::default();
        self.anymap.insert(sm);
    }
    fn register_after_secondarymap<T: 'static>(&mut self) {
        let sm: VarAfterSecMap<T> = VarAfterSecMap::<T>::default();
        self.anymap.insert(sm);
    }

    #[inline]
    fn get_var_b_a_fn_collect<T: std::clone::Clone + 'static>(
        &self,
        // data: T,
        current_id: &TopoKey,
    ) -> VarOptBAfnCollectRef<T> {
        trace!("get_state_use_key_and_start_set_run_cb");
        //unwrap or default to keep borrow checker happy
        self.opt_get_var_and_bf_af_use_id::<T>(current_id)
            .expect("set_state_with_key: can't set state that doesn't exist in this context!")

        // trace!("start_set_var_and_before_after");

        // start_set_var_and_before_after(current_id, var, data, before_fns, after_fns);
    }
    fn set_in_callback<T: Clone + 'static + std::fmt::Debug>(
        &self,
        skips: &mut SkipKeyCollection,
        data_fn: impl FnOnce() -> T,
        current_id: &TopoKey,
    ) {
        // if skips.borrow().contains(current_id) {
        //     // println!(
        //     //     "===skip contains current_id at set_in_similar_fn start -> data:{:?}",
        //     //     data
        //     // );
        //     return;
        // }

        //TODO  如果 deps statevars drop了, 但是还有 其他 deps , 怎么办?

        // {
        // let mut skips = skips.borrow_mut();
        if skips.contains(current_id) {
            return;
        }
        skips.push(*current_id);
        // }
        let data = data_fn();

        //unwrap or default to keep borrow checker happy
        let (var, opt_before_fns, opt_after_fns) = self
            .opt_get_var_and_bf_af_use_id::<T>(current_id)
            .expect("set_state_with_key: can't set state that doesn't exist in this context!");

        //
        if let Some(rcr_before_fns) = opt_before_fns {
            let current = Some(var.get());

            let before_fns = rcr_before_fns.borrow();
            if before_fns.is_empty() {
                return;
            }
            let skips_clone = skips.clone();

            //
            // let skips_borrowed = skips.get_mut();

            before_fns
                .iter()
                .filter_map(|(_key, (deps, func))| {
                    if deps.iter().any(|d| skips_clone.contains(d)) {
                        None
                    } else {
                        Some(func)
                    }
                })
                .for_each(|func| {
                    // let skip_clone = skip2.clone();
                    func(skips, &current, &data);
                });
        }

        // debug!("in callbacks ,bf_fns called, then --> var set :{:?}", data);
        // ─────────────────────────────────────────────────────────────────

        var.set(data.clone());
        // ─────────────────────────────────────────────────────────────────
        if let Some(rcr_after_fns) = opt_after_fns {
            let after_fns = rcr_after_fns.borrow();
            if after_fns.is_empty() {
                return;
            }
            let skips_clone = skips.clone();

            after_fns
                .iter()
                .filter_map(|(_key, (deps, func))| {
                    if deps.iter().any(|d| skips_clone.contains(d)) {
                        None
                    } else {
                        Some(func)
                    }
                })
                .for_each(|func| {
                    func(skips, &data);
                });
        }
    }
    fn insert_var_with_key<T: 'static>(&mut self, var: Var<T>, current_id: &StorageKey) {
        //unwrap or default to keep borrow checker happy
        trace!(
            "use_state::insert_var_with_key::({})",
            &std::any::type_name::<T>()
        );

        let key = self
            .id_to_key_map
            .get(current_id)
            .copied()
            .unwrap_or_default();

        if key.is_null() {
            let key = self.primary_slotmap.insert(*current_id);
            self.id_to_key_map.insert(*current_id, key);
            if let Some(sec_map) = self.get_mut_secondarymap::<T>() {
                sec_map.insert(key, var);
            } else {
                self.register_secondarymap::<T>();
                self.get_mut_secondarymap::<T>().unwrap().insert(key, var);
            }
        } else if let Some(existing_secondary_map) = self.get_mut_secondarymap::<T>() {
            existing_secondary_map.insert(key, var);
        } else {
            // key ! null  && T not find
            // self.register_secondarymap::<T>();
            // self.get_mut_secondarymap::<T>().unwrap().insert(key, var);
            panic!("panic current using find why here 2");
        }
    }

    #[track_caller]
    #[instrument(level = "debug", skip_all)]
    fn or_insert_var_with_key<F: FnOnce() -> T, T: 'static + std::fmt::Debug>(
        &mut self,
        func: F,
        current_id: &StorageKey,
    ) {
        //unwrap or default to keep borrow checker happy
        trace!(
            "use_state::or_insert_var_with_key::({})",
            &std::any::type_name::<T>()
        );

        if let Some(k) = self.id_to_key_map.get(current_id).copied() {
            self.get_mut_secondarymap::<T>().map_or_else(
                || {
                    panic!("panic current using find why here 2");
                },
                |existing_secondary_map| {
                    if !existing_secondary_map.contains_key(k) {
                        existing_secondary_map.insert(k, Var::new(func()));
                    }
                },
            );
        } else {
            let key = self.primary_slotmap.insert(*current_id);
            debug!(id=?current_id, "or_insert_var_with_key::insert new key");
            let data = func();
            debug!("data ty ======>{}", std::any::type_name::<T>());
            // debug!("data ======>{:?}", &data);

            self.id_to_key_map.insert(*current_id, key);

            if let Some(sec_map) = self.get_mut_secondarymap::<T>() {
                sec_map.insert(key, Var::new(data));
            } else {
                self.register_secondarymap::<T>();
                self.get_mut_secondarymap::<T>()
                    .unwrap()
                    .insert(key, Var::new(data));
            }
        }
    }
    // fn get_similar_map<T: Clone + 'static>(
    //     &mut self,
    //     id: &StorageKey,
    // ) -> Option<&HashMap<StorageKey, Box<dyn Fn(Vec<StorageKey>, T)>>> {
    //     self.get_state_and_similar_with_id::<T>(id).map(|km| &km.1)
    // }

    /// # Panics
    ///
    /// Will panic if HashMap<StorageKey, Box<dyn Fn(&GStateStore, &RefCell<Vec<StorageKey, Global>>, T)> has `callback_id`
    /// # Errors
    ///
    /// Will return `Err` if `fns.contains_key(after_fn_id)`
    /// permission to read it.
    #[tracing::instrument(skip(self, func), level = "debug")]
    #[topo::nested]
    pub fn insert_before_fn_in_topo<T: 'static>(
        &mut self,
        current_id: &StorageKey,
        func: BoxSynCallBeforeFn<T>,
        deps: &[TopoKey],
        //TODO add store
    ) -> Result<Rc<StorageKey>, String> {
        // assert_ne!(current_id, before_fn_id);

        let before_fn_id = Rc::new(StorageKey::TopoKey(TopoKey {
            id: topo::call(topo::CallId::current),
        }));

        let key = self.id_to_key_map.get(current_id).copied();
        let before_secondary_map = self.get_mut_before_secondarymap::<T>();
        match (key, before_secondary_map) {
            (Some(existing_key), Some(existing_before_secondary_map)) => {
                let rcr_fns = existing_before_secondary_map
                    .entry(existing_key)
                    .unwrap()
                    .or_insert_with(|| {
                        Rc::new(RefCell::new(
                            SynCallBeforeFnsMap::<T>::with_capacity_and_hasher(
                                1,
                                BuildHasherDefault::<CustomHasher>::default(),
                            ),
                        ))
                    });
                let mut fns = rcr_fns.borrow_mut();

                // let f = Rc::new(*before_fn_id);
                // let k = Rc::downgrade(&f);
                debug_assert!(!fns.contains_key(&before_fn_id));

                fns.insert(before_fn_id.clone(), (deps.to_smallvec(), func));

                Ok(before_fn_id)

                // if fns.contains_key(before_fn_id) {
                //     return Err("before_fns already has this fn".to_string());
                // }

                // fns.insert(*before_fn_id, func);
                // Ok(())
            }
            (Some(existing_key), None) => {
                self.register_before_secondarymap::<T>();

                let mut new_map = SynCallBeforeFnsMap::<T>::with_capacity_and_hasher(
                    1,
                    BuildHasherDefault::<CustomHasher>::default(),
                );
                new_map.insert(before_fn_id.clone(), (deps.to_smallvec(), func));

                self.get_mut_before_secondarymap::<T>()
                    .unwrap()
                    .insert(existing_key, Rc::new(RefCell::new(new_map)));

                Ok(before_fn_id)
            }
            (key, map) => {
                panic!(
                    "something(key or map) is None?: key.is_none?{} map.is_none?{}",
                    key.is_none(),
                    map.is_none()
                )
            }
        }
    }

    /// # Panics
    ///
    /// Will panic if HashMap<StorageKey, Box<dyn Fn(&GStateStore, &RefCell<Vec<StorageKey, Global>>, T)> has `callback_id`
    /// # Errors
    ///
    /// Will return `Err` if `fns.contains_key(after_fn_id)`
    /// permission to read it.
    #[topo::nested]
    pub fn insert_after_fn_in_topo<T: 'static>(
        &mut self,
        current_id: &StorageKey,
        func: BoxSynCallAfterFn<T>,
        deps: &[TopoKey],
    ) -> Result<Rc<StorageKey>, String> {
        // assert_ne!(current_id, after_fn_id);

        let after_fn_id = Rc::new(StorageKey::TopoKey(TopoKey {
            id: topo::call(topo::CallId::current),
        }));

        let key = self.id_to_key_map.get(current_id).copied();
        let after_secondary_map = self.get_mut_after_secondarymap::<T>();
        match (key, after_secondary_map) {
            (Some(existing_key), Some(existing_after_secondary_map)) => {
                let rcr_fns = existing_after_secondary_map
                    .entry(existing_key)
                    .unwrap()
                    .or_insert_with(|| {
                        Rc::new(RefCell::new(
                            SynCallAfterFnsMap::<T>::with_capacity_and_hasher(
                                1,
                                BuildHasherDefault::<CustomHasher>::default(),
                            ),
                        ))
                    });
                let mut fns = rcr_fns.borrow_mut();
                // if fns.contains_key(after_fn_id) {
                //     return Err("before_fns already has this fn".to_string());
                // }
                debug_assert!(!fns.contains_key(&after_fn_id));

                fns.insert(after_fn_id.clone(), (deps.to_smallvec(), func));

                Ok(after_fn_id)
            }
            (Some(existing_key), None) => {
                self.register_after_secondarymap::<T>();

                let mut new_map = SynCallAfterFnsMap::<T>::with_capacity_and_hasher(
                    1,
                    BuildHasherDefault::<CustomHasher>::default(),
                );
                new_map.insert(after_fn_id.clone(), (deps.to_smallvec(), func));

                self.get_mut_after_secondarymap::<T>()
                    .unwrap()
                    .insert(existing_key, Rc::new(RefCell::new(new_map)));

                Ok(after_fn_id)
            }
            (key, map) => {
                panic!(
                    "[insert_after_fn] something(key or map) is None?: key.is_none?{} map.is_none?{}",
                    key.is_none(),
                    map.is_none()
                )
            }
        }
    }

    /// # Panics
    ///
    /// Will panic if something(key or map) is None
    ///
    pub fn get_after_fns_map<T: 'static>(
        &self,
        current_id: &StorageKey,
    ) -> RCRSynCallAfterFnsMap<T> {
        let key = self.id_to_key_map.get(current_id).copied();
        let secondarymap = self.get_after_secondarymap::<T>();
        let fns = match (key, secondarymap) {
            (Some(existing_key), Some(existing_secondary_map)) => existing_secondary_map
                .get(existing_key)
                .expect("cannot get second map"),
            (key, map) => panic!(
                "something(key or map) is None: {} {}",
                key.is_none(),
                map.is_none()
            ),
        };
        fns.clone()

        // debug!("fns: {:?}", &fns.keys());
    }
    /// # Panics
    ///
    /// Will panic if something(key or map) is None
    ///
    // pub fn remove_after_fn<T: 'static>(
    //     &mut self,
    //     current_id: &StorageKey,
    //     after_fn_id: &StorageKey,
    // ) {
    //     let key = self.id_to_key_map.get(current_id).copied();
    //     let secondarymap = self.get_mut_after_secondarymap::<T>();
    //     let fns = match (key, secondarymap) {
    //         (Some(existing_key), Some(existing_secondary_map)) => existing_secondary_map
    //             .get_mut(existing_key)
    //             .expect("cannot get second map"),
    //         (key, map) => panic!(
    //             "something(key or map) is None: {} {}",
    //             key.is_none(),
    //             map.is_none()
    //         ),
    //     };

    //     debug!("fns: {:?}", &fns.keys());
    //     fns.remove(after_fn_id);
    // }

    fn opt_get_var_and_bf_af_use_id<T: 'static>(
        &self,
        current_id: impl Into<StorageKey>,
    ) -> Result<VarOptBAfnCollectRef<T>, Error> {
        let storage_key = current_id.into();
        match (
            self.id_to_key_map.get(&storage_key),
            self.get_secondarymap::<T>(),
        ) {
            (Some(existing_key), Some(existing_secondary_map)) => {
                let v = existing_secondary_map
                    .get(*existing_key)
                    .ok_or(Error::SecMapNoKey(*existing_key, storage_key))?;

                let b =
                    self.get_before_secondarymap::<T>()
                        .and_then(|existing_before_secondary_map| {
                            existing_before_secondary_map.get(*existing_key)
                        });

                let a =
                    self.get_after_secondarymap::<T>()
                        .and_then(|existing_after_secondary_map| {
                            existing_after_secondary_map.get(*existing_key)
                        });

                Ok((v, b, a))
            }
            (None, None) => Err(Error::StoreNoKeyNoVarMapForType(
                storage_key,
                std::any::type_name::<T>(),
            )),
            (None, Some(_)) => Err(Error::StoreNoKey(storage_key)),
            (Some(_), None) => Err(Error::StoreNoVarMapForType(std::any::type_name::<T>())),
        }
    }
    #[instrument(level = "debug", skip_all)]
    pub fn opt_get_var_use_id<T: 'static>(&self, current_id: &StorageKey) -> Option<&Var<T>> {
        match (
            self.id_to_key_map.get(current_id),
            self.get_secondarymap::<T>(),
        ) {
            (Some(existing_key), Some(existing_secondary_map)) => {
                existing_secondary_map.get(*existing_key)
            }
            _ => None,
        }
    }

    // fn remove_state_with_id<T: 'static>(&mut self, current_id: &StorageKey) -> Option<Var<T>> {
    //     // /self.unseen_ids.remove(&current_id);
    //     //unwrap or default to keep borrow checker happy
    //     let key = self
    //         .id_to_key_map
    //         .get(current_id)
    //         .copied()
    //         .unwrap_or_default();

    //     if key.is_null() {
    //         None
    //     } else {
    //         self.get_mut_secondarymap::<T>()
    //             .and_then(|existing_secondary_map| existing_secondary_map.remove(key))
    //     }
    // }

    /// Get a mutable reference to the g state store's engine.
    pub fn engine_mut(&self) -> std::cell::RefMut<Engine> {
        self.engine.borrow_mut()
    }

    fn link_callback_drop(&mut self, dep_topo_key: TopoKey, b_a_key: Rc<StorageKey>) {
        let key = self
            .id_to_key_map
            .get(&StorageKey::TopoKey(dep_topo_key))
            .copied()
            .unwrap();
        let entry = self.b_a_fn_drop_link_map.entry(key).unwrap();
        entry.or_default().push(b_a_key);
    }
}

fn before_fns_run<T: 'static>(
    // store: &GStateStore,
    current_topo_key: TopoKey,
    current_data: &Option<Rc<T>>, //TODO 是否需要 opt? 看上去好像不需要
    data: &T,
    opt_fns: Option<&RCRSynCallBeforeFnsMap<T>>,
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
fn after_fns_run<T: 'static>(
    // store: &GStateStore,
    mut skip: SkipKeyCollection,
    data: &T,
    opt_fns: Option<&RCRSynCallAfterFnsMap<T>>,
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

fn start_set_var_and_run_before_after<T: Clone + 'static>(
    // store: &GStateStore,
    current_topo_key: TopoKey,
    var: &anchors::expert::Var<T, Engine>,
    current: Rc<T>,
    data: T,
    before_fns: Option<&RCRSynCallBeforeFnsMap<T>>,
    after_fns: Option<&RCRSynCallAfterFnsMap<T>>,
) {
    debug!(
        ?current_topo_key,
        "[start_set_var_and_run_before_after] type: {}",
        std::any::type_name::<T>()
    );

    //NOTE staring first callback call
    let skip: SkipKeyCollection =
        before_fns_run(current_topo_key, &Some(current), &data, before_fns);
    trace!("start_set_var_and_before_after:: before_fns_runned,now set new v");

    var.set(data.clone());
    trace!("start_set_var_and_before_after:: after_fns_run");

    // after_fns_run(&skip.clone(), &data.clone(), &after_fns.clone());
    after_fns_run(skip, &data, after_fns);
}
// ────────────────────────────────────────────────────────────────────────────────
pub trait StateTypeCheck {
    const INSIDE_TYPE_NAME: TypeName;
}
impl<T> StateTypeCheck for StateVar<T>
where
    T: TypeCheck,
{
    const INSIDE_TYPE_NAME: TypeName = T::TYPE_NAME;
}
impl<T> StateTypeCheck for StateAnchor<T>
where
    T: TypeCheck,
{
    const INSIDE_TYPE_NAME: TypeName = T::TYPE_NAME;
}

// TODO read https://docs.rs/graph_safe_compare/latest/graph_safe_compare/
pub struct StateVar<T> {
    id: TopoKey,
    _phantom_data: PhantomData<T>,
}

impl<T> Eq for StateVar<T> {}

impl<T> PartialEq for StateVar<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T: 'static + std::fmt::Display + Clone> std::fmt::Display for StateVar<T> {
    default fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.get();
        write!(f, "\u{2726} ({})", &v)
    }
}
// NOTE: need #![feature(specialization)]
// impl<T: 'static + std::fmt::Display + Clone> std::fmt::Display for StateVar<StateAnchor<T>> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let v = self.get();
//         write!(f, "\u{2726} ({})", &v)
//     }
// }

impl<T: 'static + std::fmt::Debug + Clone> std::fmt::Debug for StateVar<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.get();
        f.debug_tuple("StateVar").field(&v).finish()
    }
}

impl<T> Copy for StateVar<T> {}
impl<T> Clone for StateVar<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _phantom_data: PhantomData::<T>,
        }
    }
}

impl<T> StateVar<StateAnchor<T>>
where
    T: 'static + std::clone::Clone,
{
    /// # Errors
    ///
    /// Will return `Err` if engine cannot `borrow_mut`
    /// permission to read it.
    pub fn try_get(&self) -> Result<T, Error> {
        self.get_with(CloneStateAnchor::try_get)
    }
    #[must_use]
    pub fn get(&self) -> T {
        get_anchor_val_in_var_with_topo_id::<T>(self.id)
    }
    pub fn store_get(&self, store: &GStateStore) -> T {
        let var_with_sa = store
            .opt_get_var_use_id::<StateAnchor<T>>(&StorageKey::TopoKey(self.id))
            .expect("You are trying to get a var state that doesn't exist in this context!");

        store.engine_get((var_with_sa.get()).anchor())
    }
    pub fn get_inner_anchor(&self) -> StateAnchor<T> {
        self.get_with(std::clone::Clone::clone)
    }
    pub fn store_get_inner_anchor(&self, store: &GStateStore) -> StateAnchor<T> {
        self.store_get_rc(store).as_ref().clone()
    }
}

impl<T> StateVar<T>
where
    T: 'static,
{
    /// # Panics
    ///
    /// Will panic if `store.id_to_key_map` not have Self `topo_key`
    pub fn manually_drop(&self) {
        debug!("StateVar<{}> drop .. ", std::any::type_name::<T>(),);

        // let store = state_store();
        debug!("a");
        state_store_with(|g_state_store_refcell| {
            debug!("b");
            let mut store = g_state_store_refcell.borrow_mut();

            let topo_key = StorageKey::TopoKey(self.id);
            let key = store.id_to_key_map.remove(&topo_key).unwrap();

            store.primary_slotmap.remove(key);
            store.b_a_fn_drop_link_map.remove(key);

            // let existing_secondary_map = store.get_mut_secondarymap::<T>().unwrap();
            // existing_secondary_map.remove(key);
            // if existing_secondary_map.is_empty() {
            //     store.remove_secondarymap::<T>();
            // }

            // if let Some(b_map) = store.get_mut_before_secondarymap::<T>() {
            //     b_map.remove(key);
            //     if b_map.is_empty() {
            //         store.remove_before_secondarymap::<T>();
            //     }
            // }
            // if let Some(a_map) = store.get_mut_after_secondarymap::<T>() {
            //     a_map.remove(key);
            //     if a_map.is_empty() {
            //         store.remove_after_secondarymap::<T>();
            //     }
            // }
        });
    }

    #[must_use]
    const fn new(id: TopoKey) -> Self {
        Self {
            id,
            _phantom_data: PhantomData,
        }
    }
    /// Get a reference to the state var's id.
    #[must_use]
    pub const fn id(&self) -> &TopoKey {
        &self.id
    }

    #[must_use]
    #[inline]
    pub fn state_exists(&self) -> bool {
        state_exists_for_topo_id::<T>(self.id)
    }

    // #[must_use]
    pub fn get_with<F: Fn(&T) -> R, R>(&self, func: F) -> R {
        read_val_with_topo_id(self.id, |t| func(t))
    }

    #[must_use]
    #[inline]
    pub fn store_get_rc(&self, store: &GStateStore) -> Rc<T> {
        self.store_get_var_with(store, anchors::expert::Var::get)
    }

    #[must_use]
    pub fn get_rc(&self) -> Rc<T> {
        state_store_with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
            trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

            let store = g_state_store_refcell.borrow();
            self.store_get_rc(&store)
        })
    }

    #[inline]
    pub fn get_var_with<F: Fn(&Var<T>) -> R, R>(&self, func: F) -> R {
        read_var_with_topo_id::<_, T, R>(self.id, |v: &Var<T>| -> R { func(v) })
    }
    pub fn store_get_var_with<F: Fn(&Var<T>) -> R, R>(&self, store: &GStateStore, func: F) -> R {
        let var = &store
            .opt_get_var_use_id::<T>(&StorageKey::TopoKey(self.id))
            .expect("You are trying to get a var state that doesn't exist in this context!");

        // .clone();

        func(var)
    }

    #[must_use]
    #[inline]
    pub fn watch(&self) -> StateAnchor<T> {
        self.get_var_with(|v| StateAnchor(v.watch()))
    }
    #[must_use]
    #[inline]
    pub fn store_watch(&self, store: &GStateStore) -> StateAnchor<T> {
        // self.get_var_with(|v| StateAnchor(v.watch()))
        self.store_get_var_with(store, |v| StateAnchor(v.watch()))
    }
    /// # set, but in the before / after callback fn scope
    #[inline]

    pub fn seting_in_b_a_callback(&self, skip: &mut SkipKeyCollection, data_fn: impl FnOnce() -> T)
    where
        T: Clone + std::fmt::Debug,
    {
        set_in_callback(skip, data_fn, self.id());
    }
}

// impl<T> From<anchors::expert::Anchor<T, anchors::singlethread::Engine>> for StateAnchor<T> {
//     fn from(anchor: anchors::expert::Anchor<T, anchors::singlethread::Engine>) -> Self {
//         StateAnchor(anchor)
//     }
// }

// impl<T> MultiAnchor<Engine> for StateAnchor<T>
// where
//     T: 'static + Clone,
// {
//     type Target = Anchor<T>;
//     fn map<F, Out>(self, f: F) -> Anchor<Out>
//     where
//         MultiAnchor
//     {
//         self.0.map::<F, Out>(f)
//     }
// }

pub trait CloneStateVar<T>
where
    T: Clone + 'static,
{
    fn get(&self) -> T;
    fn store_get(&self, store: &GStateStore) -> T;
    fn set(&self, value: T);
    // fn set_in_callback(&self, store: &GStateStore, skip: &SkipKeyCollection, value: &T)
    // where
    //     T: std::fmt::Debug;
    fn store_set(&self, store: &GStateStore, value: T);
    fn opt_set_with_once<F: FnOnce(&T) -> Option<T>>(&self, func_once: F);
    fn set_with_once<F: FnOnce(&T) -> T>(&self, func_once: F);

    /// # Errors
    ///
    /// Will return `Err` if got [Error]
    /// permission to read it.
    fn store_set_with_once<F: FnOnce(&T) -> T>(
        &self,
        store: &GStateStore,
        func_once: F,
    ) -> Result<(), Error>;
    fn set_with<F: Fn(&T) -> T>(&self, func: F);
    // fn try_get(&self) -> Option<T>;

    // fn update<F: FnOnce(&mut T)>(&self, func: F);
    fn update<F: FnOnce(&mut T) -> R, R>(&self, func: F) -> R;
    fn update_bool_check<F: FnOnce(&mut T) -> bool>(&self, func: F) -> bool;

    fn update_opt_check<F: FnOnce(&mut T) -> Option<R>, R>(&self, func: F) -> Option<R>;

    fn store_update<F: FnOnce(&mut T) -> R, R>(&self, store: &GStateStore, func: F) -> R;
    /// # Errors
    ///
    /// Will return `Err` if got [Error]
    fn store_update_result_check<F: FnOnce(&mut T) -> Result<R, E>, R, E>(
        &self,
        store: &GStateStore,
        func: F,
    ) -> Result<R, E>;
    /// # Errors
    ///
    /// Will return `Err` if `fns.contains_key(callback_key)`
    /// permission to read it.
    fn insert_before_fn_in_topo(
        &self,
        func: impl Fn(&mut SkipKeyCollection, &Option<Rc<T>>, &T) + 'static,
        init: bool,
        deps: &[TopoKey],
    ) -> Option<Rc<StorageKey>>;

    /// # 插入 修改后执行function
    /// # Errors
    ///
    /// ### skip(err) or insert(ok)
    ///
    /// Will return `Err` if `fns.contains_key(callback_key)`
    /// permission to read it.
    fn insert_after_fn_in_topo(
        &self,
        func: impl Fn(&mut SkipKeyCollection, &T) + 'static,
        init: bool,
        deps: &[TopoKey],
    ) -> Option<Rc<StorageKey>>;

    fn remove_after_fn(&self, callback_key: TopoKey);

    fn build_similar_use_into_in_topo<B: Clone + From<T> + 'static + std::fmt::Debug>(
        &self,
    ) -> StateVar<B>;
    fn build_bi_similar_use_into_in_topo<B: Clone + From<T> + Into<T> + 'static + std::fmt::Debug>(
        &self,
    ) -> StateVar<B>
    where
        T: std::fmt::Debug;

    fn link_callback_drop(&self, fk: Rc<StorageKey>);

    // fn to_bi_in_topo<B>(&self) -> (StateVarDi<T, B>, StateVarDi<B, T>)
    // where
    //     B: From<T> + Clone + 'static,
    //     T: From<B> + 'static;
}

impl<T> CloneStateVar<T> for StateVar<T>
where
    T: Clone + 'static,
{
    /// returns a clone of the stored state panics if not stored.
    fn get(&self) -> T {
        // let var = clone_state_with_topo_id::<T>(self.id).expect("state should be present");
        // (*var.get()).clone()
        self.get_with(std::clone::Clone::clone)
        // log::debug!("=====StateVar get {:?}", &t);
    }

    fn store_get(&self, store: &GStateStore) -> T {
        self.store_get_rc(store).as_ref().clone()
    }

    fn set(&self, value: T)
    // where
    //     T: std::fmt::Debug,
    {
        read_var_b_a_with_topo_id::<_, T, ()>(
            self.id(),
            |(var, before_fns, after_fns): VarOptBAfnCollectRef<T>| {
                let current = var.get();
                start_set_var_and_run_before_after(
                    // store,
                    self.id, var, current, value, before_fns, after_fns,
                );
            },
        );
    }

    //TODO use illicit @set replace set_in_callback

    fn store_set(&self, store: &GStateStore, value: T) {
        let (var, before_fns, after_fns) = store.get_var_b_a_fn_collect::<T>(self.id());
        let current = var.get();
        start_set_var_and_run_before_after(self.id, var, current, value, before_fns, after_fns);
    }

    fn set_with<F: Fn(&T) -> T>(&self, func: F) {
        read_var_b_a_with_topo_id::<_, T, ()>(
            self.id(),
            |(var, before_fns, after_fns): VarOptBAfnCollectRef<T>| {
                let current = var.get();
                let data = func(&current);
                start_set_var_and_run_before_after(
                    // store,
                    self.id, var, current, data, before_fns, after_fns,
                );
            },
        );
    }
    fn set_with_once<F: FnOnce(&T) -> T>(&self, func_once: F) {
        read_var_b_a_with_topo_id::<_, T, ()>(
            self.id(),
            |(var, before_fns, after_fns): VarOptBAfnCollectRef<T>| {
                let current = var.get();
                let data = func_once(&current);

                start_set_var_and_run_before_after(
                    // store,
                    self.id, var, current, data, before_fns, after_fns,
                );
            },
        );
    }
    fn opt_set_with_once<F: FnOnce(&T) -> Option<T>>(&self, func_once: F) {
        read_var_b_a_with_topo_id::<_, T, ()>(
            self.id(),
            |(var, before_fns, after_fns): VarOptBAfnCollectRef<T>| {
                let current = var.get();
                let opt_data = func_once(&current);
                if let Some(data) = opt_data {
                    start_set_var_and_run_before_after(
                        // store,
                        self.id, var, current, data, before_fns, after_fns,
                    );
                }
            },
        );
    }

    fn store_set_with_once<F: FnOnce(&T) -> T>(
        &self,
        store: &GStateStore,
        func_once: F,
    ) -> Result<(), Error> {
        let (var, before_fns, after_fns) = store.opt_get_var_and_bf_af_use_id::<T>(self.id())?;
        let current = var.get();
        let data = func_once(&current);

        start_set_var_and_run_before_after(
            // store,
            self.id, var, current, data, before_fns, after_fns,
        );
        Ok(())
    }
    // fn try_get(&self) -> Option<T> {
    //     clone_state_with_topo_id::<T>(self.id).map(|v| (*v.get()).clone())
    // }

    // fn update<F: FnOnce(&mut T)>(&self, func: F) {
    //     // read_var_with_topo_id::<_, T, ()>(self.id, |var| {
    //     //     let mut old = (*var.get()).clone();
    //     //     func(&mut old);
    //     //     var.set(old);
    //     // })

    //     //NOTE 'set_with_once' has callback update inside
    //     self.set_with_once(|v| {
    //         let mut old = v.clone();
    //         func(&mut old);
    //         old
    //     });
    // }
    fn update<F: FnOnce(&mut T) -> R, R>(&self, func: F) -> R {
        read_var_b_a_with_topo_id::<_, T, R>(
            self.id(),
            |(var, before_fns, after_fns): VarOptBAfnCollectRef<T>| {
                let current = var.get();
                let mut edited_v = (*current).clone();

                let r = func(&mut edited_v);

                start_set_var_and_run_before_after(
                    // store,
                    self.id, var, current, edited_v, before_fns, after_fns,
                );
                r
            },
        )
    }
    fn update_bool_check<F: FnOnce(&mut T) -> bool>(&self, func: F) -> bool {
        read_var_b_a_with_topo_id::<_, T, bool>(
            self.id(),
            |(var, before_fns, after_fns): VarOptBAfnCollectRef<T>| {
                let current = var.get();
                let mut edited_v = (*current).clone();

                let is_changed = func(&mut edited_v);
                if is_changed {
                    start_set_var_and_run_before_after(
                        // store,
                        self.id, var, current, edited_v, before_fns, after_fns,
                    );
                }

                is_changed
            },
        )
    }

    fn update_opt_check<F: FnOnce(&mut T) -> Option<R>, R>(&self, func: F) -> Option<R> {
        read_var_b_a_with_topo_id::<_, T, Option<R>>(
            self.id(),
            |(var, before_fns, after_fns): VarOptBAfnCollectRef<T>| {
                let current = var.get();
                let mut edited_v = (*current).clone();

                let opt_r = func(&mut edited_v);
                if opt_r.is_some() {
                    start_set_var_and_run_before_after(
                        // store,
                        self.id, var, current, edited_v, before_fns, after_fns,
                    );
                }
                opt_r
            },
        )
    }

    fn store_update<F: FnOnce(&mut T) -> R, R>(&self, store: &GStateStore, func: F) -> R {
        let (var, before_fns, after_fns) =
            store.opt_get_var_and_bf_af_use_id::<T>(self.id()).unwrap();
        let current = var.get();

        let mut edited_v = (*current).clone();

        let r = func(&mut edited_v);

        start_set_var_and_run_before_after(
            // store,
            self.id, var, current, edited_v, before_fns, after_fns,
        );
        r
    }
    fn store_update_result_check<F: FnOnce(&mut T) -> Result<R, E>, R, E>(
        &self,
        store: &GStateStore,
        func: F,
    ) -> Result<R, E> {
        let (var, before_fns, after_fns) =
            store.opt_get_var_and_bf_af_use_id::<T>(self.id()).unwrap();
        let current = var.get();

        let mut edited_v = (*current).clone();

        let r = func(&mut edited_v)?;
        start_set_var_and_run_before_after(
            // store,
            self.id, var, current, edited_v, before_fns, after_fns,
        );

        Ok(r)
    }

    // #[topo::nested]
    // fn to_di_in_topo<B>(&self) -> StateVarDi<T, B>
    // where
    //     B: From<T> + Clone + 'static,
    // {
    //     let b = use_state(||self.get().into());
    //     StateVarDi::new_use_into(*self, b)
    // }
    // #[topo::nested]
    // fn to_bi_in_topo<B>(&self) -> (StateVarDi<T, B>, StateVarDi<B, T>)
    // where
    //     B: From<T> + Clone + 'static,
    //     T: From<B> + 'static,
    // {
    //     let b = use_state(||self.get().into());
    //     (
    //         StateVarDi::new_use_into(*self, b),
    //         StateVarDi::new_use_into(b, *self),
    //     )
    // }

    //TODO 回环检测 , 当两个或者两个以上 有 di关系的 StateVar  set的时候 会再次互相调用set -回环
    /// 添加不添加 deps 都不会使 after before func 循环,
    /// 但是添加deps 可以 在 deps-TopoKey 的StateVar drop时候 将该func drop,
    //
    /// 如果 deps is some , 则返回 none , Rc储存在deps中
    #[must_use]
    #[topo::nested]
    fn insert_before_fn_in_topo(
        &self,
        func: impl Fn(&mut SkipKeyCollection, &Option<Rc<T>>, &T) + 'static,
        init: bool,
        deps: &[TopoKey],
    ) -> Option<Rc<StorageKey>> {
        insert_before_fn_common_in_topo(self, Box::new(func), init, deps)
    }

    /// 添加不添加 deps 都不会使 after before func 循环,
    /// 但是添加deps 可以 在 deps-TopoKey 的StateVar drop时候 将该func drop,
    /// 同时如果 运行func时刻, skip有deps的key,则不会运行该func
    /// 如果 deps is some , 则返回 none , Rc储存在deps中
    #[topo::nested]
    fn insert_after_fn_in_topo(
        &self,
        func: impl Fn(&mut SkipKeyCollection, &T) + 'static,
        init: bool,
        deps: &[TopoKey],
    ) -> Option<Rc<StorageKey>> {
        insert_after_fn_common_in_topo(self, Box::new(func), init, deps)
    }
    fn remove_after_fn(&self, after_fn_key: TopoKey) {
        remove_after_fn(self, &StorageKey::TopoKey(after_fn_key));
    }

    #[topo::nested]
    fn build_similar_use_into_in_topo<B: Clone + From<T> + 'static + std::fmt::Debug>(
        &self,
    ) -> StateVar<B> {
        let v = self.get();
        let b: StateVar<B> = use_state(|| v.clone().into());
        insert_before_fn_common_in_topo(
            self,
            Box::new(move |skip, _current, value| {
                b.seting_in_b_a_callback(skip, || value.clone().into());
            }),
            false,
            &[b.id],
        );
        b
    }

    #[topo::nested]
    fn build_bi_similar_use_into_in_topo<B: Clone + From<T> + Into<T> + 'static + std::fmt::Debug>(
        &self,
    ) -> StateVar<B>
    where
        T: std::fmt::Debug,
    {
        let v = self.get();
        let b: StateVar<B> = use_state(|| v.clone().into());

        let this = *self;

        insert_before_fn_common_in_topo(
            self,
            Box::new(move |skip, _current, value| {
                b.seting_in_b_a_callback(skip, || value.clone().into());
            }),
            false,
            &[b.id],
        );

        insert_before_fn_common_in_topo(
            &b,
            Box::new(move |skip, _current, value| {
                this.seting_in_b_a_callback(skip, || value.clone().into());
            }),
            false,
            &[this.id],
        );

        // .expect("insert_before_fn error");
        b
    }
    fn link_callback_drop(&self, fk: Rc<StorageKey>) {
        let state_store = state_store();
        let mut store = state_store.borrow_mut();
        store.link_callback_drop(self.id, fk);
    }
}

pub struct StateAnchor<T>(pub(crate) Anchor<T>);

impl<T> Clone for StateAnchor<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Eq for StateAnchor<T> {}

impl<T> PartialEq for StateAnchor<T> {
    fn eq(&self, other: &Self) -> bool {
        let _span = debug_span!("PartialEq for StateAnchor").entered();
        self.0 == other.0
    }
}

impl<T: 'static + std::fmt::Display + Clone> std::fmt::Display for StateAnchor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.get();
        write!(f, "\u{2693} ({})", &v)
    }
}

impl<T: 'static + std::fmt::Debug + Clone> std::fmt::Debug for StateAnchor<T> {
    #[track_caller]
    #[allow(clippy::print_in_format_impl)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.try_get() {
            Ok(v) => f.debug_tuple("StateAnchor").field(&v).finish(),
            Err(e) => {
                eprintln!("error:!!!!!!!!!!!! \n---- {e}");
                f.debug_tuple("StateAnchor")
                    .field(&"_err_can_not_get_mut_engine_")
                    .finish()
            }
        }
    }
}
impl<T> From<Anchor<T>> for StateAnchor<T>
where
    T: 'static,
{
    fn from(anchor: Anchor<T>) -> Self {
        Self(anchor)
    }
}
impl<T> From<StateAnchor<T>> for Anchor<T>
where
    T: 'static,
{
    fn from(sa: StateAnchor<T>) -> Self {
        sa.0
    }
}
impl<T> From<T> for StateAnchor<T>
where
    T: 'static,
{
    #[track_caller]
    fn from(v: T) -> Self {
        Self::constant(v)
    }
}

impl<I, V> From<StateAnchor<Dict<I, Anchor<V>>>> for StateAnchor<Dict<I, V>>
where
    V: std::clone::Clone + 'static,
    I: 'static + Clone + std::cmp::Ord,
    // Dict<I, V>: std::cmp::Eq,
{
    fn from(value: StateAnchor<Dict<I, Anchor<V>>>) -> Self {
        value.then(|v| {
            anchors::collections::ord_map_collect::OrdMapCollect::new_to_anchor(v.clone())
        })
    }
}
impl<I, V> From<StateAnchor<Dict<I, Anchor<V>>>> for StateAnchor<Vector<V>>
where
    V: std::clone::Clone + 'static,
    I: 'static + Clone + std::cmp::Ord,
    // Dict<I, V>: std::cmp::Eq,
{
    fn from(value: StateAnchor<Dict<I, Anchor<V>>>) -> Self {
        value.then(|v| {
            anchors::collections::vector::VectorCollect::new_to_anchor(
                v.values().cloned().collect(),
            )
        })
    }
}

impl<V> From<StateAnchor<Vector<Anchor<V>>>> for StateAnchor<Vector<V>>
where
    V: std::clone::Clone + 'static,
{
    fn from(value: StateAnchor<Vector<Anchor<V>>>) -> Self {
        value.then(|v| anchors::collections::vector::VectorCollect::new_to_anchor(v.clone()))
    }
}

pub trait CloneStateAnchor<T>
where
    T: Clone + 'static,
{
    fn get(&self) -> T;
    /// # Errors
    ///
    /// Will return `Err` if engine cannot `borrow_mut`
    /// permission to read it.
    fn try_get(&self) -> Result<T, Error>;
    fn get_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R;

    fn store_get(&self, store: &GStateStore) -> T;
    fn store_get_with<F: FnOnce(&T) -> R, R>(&self, store: &GStateStore, func: F) -> R;
}

#[derive(Copy, Clone, Debug)]
pub struct LocationEngineGet(&'static Location<'static>);

impl LocationEngineGet {
    #[allow(clippy::new_without_default)]
    #[track_caller]
    #[must_use]
    fn new() -> Self {
        illicit::get::<Self>()
            .as_deref()
            .map_or_else(|_| Self(Location::caller()), |x| *x)
    }
    #[track_caller]
    #[must_use]
    fn reset_new() -> Self {
        Self(Location::caller())
    }
}

impl Deref for LocationEngineGet {
    type Target = &'static Location<'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> CloneStateAnchor<T> for StateAnchor<T>
where
    T: Clone + 'static,
{
    #[track_caller]
    fn get(&self) -> T {
        // let loc = LocationEngineGet::reset_new();
        // illicit::Layer::new().offer(loc).enter(|| {
        let _span = debug_span!("StateAnchor.get",ty = %std::any::type_name::<T>()).entered();
        global_engine_get_anchor_val(&self.0)
        // })
    }

    #[track_caller]
    fn try_get(&self) -> Result<T, Error> {
        let loc = LocationEngineGet::new();

        illicit::Layer::new().offer(loc).enter(|| {
            let _span =
                debug_span!("StateAnchor::try_get",ty = %std::any::type_name::<T>()).entered();
            try_global_engine_get_anchor_val(&self.0)
        })
    }

    fn get_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R {
        global_engine_get_anchor_val_with(&self.0, func)
    }
    fn store_get(&self, store: &GStateStore) -> T {
        store.engine_get(&self.0)
    }
    fn store_get_with<F: FnOnce(&T) -> R, R>(&self, store: &GStateStore, func: F) -> R {
        store.engine_get_with(&self.0, func)
    }
}

impl<K: Ord + Clone + PartialEq + 'static, V: Clone + PartialEq + 'static> StateAnchor<Dict<K, V>> {
    #[track_caller]
    #[must_use]
    #[inline]
    pub fn filter<F: FnMut(&K, &V) -> bool + 'static>(&self, f: F) -> Self {
        self.0.filter(f).into()
    }
    #[track_caller]
    #[must_use]
    #[inline]
    pub fn filter_with_anchor<A, F>(&self, anchor: &StateAnchor<A>, f: F) -> Self
    where
        A: 'static + std::cmp::PartialEq + std::clone::Clone,
        F: FnMut(&A, &K, &V) -> bool + 'static,
    {
        self.0.filter_with_anchor(anchor.anchor(), f).into()
    }

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn map_<F: FnMut(&K, &V) -> T + 'static, T: Clone + PartialEq + 'static>(
        &self,
        f: F,
    ) -> StateAnchor<Dict<K, T>> {
        self.0.map_(f).into()
    }
    #[track_caller]
    #[must_use]
    #[inline]
    pub fn map_with_anchor<A, F, T>(&self, anchor: &StateAnchor<A>, f: F) -> StateAnchor<Dict<K, T>>
    where
        A: 'static + std::cmp::PartialEq + Clone,
        F: FnMut(&A, &K, &V) -> T + 'static,
        T: Clone + PartialEq + 'static,
    {
        self.0.map_with_anchor(anchor.anchor(), f).into()
    }

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn filter_map<F: FnMut(&K, &V) -> Option<T> + 'static, T: Clone + PartialEq + 'static>(
        &self,
        f: F,
    ) -> StateAnchor<Dict<K, T>> {
        self.0.filter_map(f).into()
    }
    #[track_caller]
    #[must_use]
    #[inline]
    pub fn filter_map_with_anchor<A, F, T>(
        &self,
        anchor: &StateAnchor<A>,
        f: F,
    ) -> StateAnchor<Dict<K, T>>
    where
        A: 'static + std::cmp::PartialEq + Clone,
        F: FnMut(&A, &K, &V) -> Option<T> + 'static,
        T: Clone + PartialEq + 'static,
    {
        self.0.filter_map_with_anchor(anchor.anchor(), f).into()
    }

    /// Dict 增加/更新 K V 会增量执行 function f , 用于更新 out,
    /// Dict 移除 K V 并不会触发 out 的更新,
    #[track_caller]
    #[must_use]
    #[inline]
    pub fn increment_reduction<
        F: FnMut(&mut T, &K, &V) + 'static,
        T: Clone + PartialEq + 'static,
    >(
        &self,
        init: T,
        f: F,
    ) -> StateAnchor<T> {
        self.0.increment_reduction(init, f).into()
    }

    /// Dict 增加/更新 K V 会增量执行 function f , 用于更新 out,
    /// Dict 移除 K V 也会执行 remove f,
    #[track_caller]
    #[must_use]
    #[inline]
    pub fn reduction<
        F: FnMut(&mut T, &K, &V) -> bool + 'static,
        Fu: FnMut(&mut T, (&K, &V), &K, &V) -> bool + 'static,
        T: Clone + PartialEq + 'static,
    >(
        &self,
        init: T,
        add: F,
        update: Fu,
        remove: F,
    ) -> StateAnchor<T> {
        self.0.reduction(init, add, update, remove).into()
    }
}

//TODO remove static
impl<T> StateAnchor<T>
where
    T: 'static,
{
    #[track_caller]
    #[inline]
    #[allow(clippy::missing_const_for_fn)]
    pub fn constant(val: T) -> Self {
        state_store_with(|_g_state_store_refcell| {});
        Self(Anchor::constant(val))
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    #[inline]
    pub fn into_anchor(self) -> Anchor<T> {
        self.0
    }

    #[must_use]
    #[inline]
    pub const fn anchor(&self) -> &Anchor<T> {
        &self.0
    }
    #[must_use]
    #[inline]
    pub fn get_anchor(&self) -> Anchor<T> {
        self.0.clone()
    }
    // ────────────────────────────────────────────────────────────────────────────────

    #[track_caller]
    #[inline]
    pub fn map<Out, F>(&self, f: F) -> StateAnchor<Out>
    where
        Out: 'static,
        F: 'static,
        map::Map<(Anchor<T>,), F, Out>: AnchorInner<Engine, Output = Out>,
    {
        self.0.map(f).into()
    }
    #[track_caller]
    #[inline]
    pub fn map_mut<Out, F>(&self, initial: Out, f: F) -> StateAnchor<Out>
    where
        Out: 'static,
        F: 'static,
        map_mut::MapMut<(Anchor<T>,), F, Out>: AnchorInner<Engine, Output = Out>,
    {
        self.0.map_mut(initial, f).into()
    }

    #[track_caller]
    #[inline]
    pub fn then<Out, F>(&self, f: F) -> StateAnchor<Out>
    where
        F: 'static,
        Out: 'static,
        then::Then<(Anchor<T>,), Out, F, Engine>: AnchorInner<Engine, Output = Out>,
    {
        self.0.then(f).into()
    }

    #[track_caller]
    #[inline]
    pub fn refmap<F, Out>(&self, f: F) -> StateAnchor<Out>
    where
        Out: 'static,
        F: 'static,
        refmap::RefMap<(Anchor<T>,), F>: AnchorInner<Engine, Output = Out>,
    {
        self.0.refmap(f).into()
    }
    #[track_caller]
    #[inline]
    pub fn cutoff<F, Out>(&self, f: F) -> StateAnchor<Out>
    where
        Out: 'static,
        F: 'static,
        cutoff::Cutoff<(Anchor<T>,), F>: AnchorInner<Engine, Output = Out>,
    {
        self.0.cutoff(f).into()
    }
}

pub trait StateMultiAnchor: Sized {
    type Target;

    fn map<F, Out>(self, f: F) -> StateAnchor<Out>
    where
        Out: 'static,
        F: 'static,
        map::Map<Self::Target, F, Out>: AnchorInner<Engine, Output = Out>;

    fn map_mut<F, Out>(self, initial: Out, f: F) -> StateAnchor<Out>
    where
        Out: 'static,
        F: 'static,
        map_mut::MapMut<Self::Target, F, Out>: AnchorInner<Engine, Output = Out>;

    fn then<F, Out>(self, f: F) -> StateAnchor<Out>
    where
        F: 'static,
        Out: 'static,
        then::Then<Self::Target, Out, F, Engine>: AnchorInner<Engine, Output = Out>;

    fn cutoff<F, Out>(self, _f: F) -> StateAnchor<Out>
    where
        Out: 'static,
        F: 'static,
        cutoff::Cutoff<Self::Target, F>: AnchorInner<Engine, Output = Out>;

    fn refmap<F, Out>(self, _f: F) -> StateAnchor<Out>
    where
        Out: 'static,
        F: 'static,
        refmap::RefMap<Self::Target, F>: AnchorInner<Engine, Output = Out>;
}
// ────────────────────────────────────────────────────────────────────────────────

macro_rules! impl_tuple_ext {
    ($([$output_type:ident, $num:tt])+) => {
        impl <$($output_type,)+ > StateAnchor<($($output_type,)+)>
        where
            $(
                $output_type: Clone + PartialEq + 'static,
            )+
            // E: Engine,
        {
            #[must_use]
            #[track_caller]
            pub fn split(&self) -> ($(StateAnchor<$output_type>,)+) {
                ($(
                    self.0.refmap(|v| &v.$num).into(),
                )+)
            }
        }

        impl<$($output_type,)+ > StateMultiAnchor for ($(&StateAnchor<$output_type>,)+)
        where
            $(
                $output_type: 'static,
            )+
            // E: Engine,
        {
            type Target = ($(Anchor<$output_type>,)+);

            #[track_caller]
            fn map<F, Out>(self, f: F) -> StateAnchor<Out>
            where
                Out: 'static,
                F: 'static,
                map::Map<Self::Target, F, Out>: AnchorInner<Engine, Output=Out>,
            {
                ($(&self.$num.0,)+).map(f).into()
            }

            #[track_caller]
            fn map_mut<F, Out>(self, initial: Out, f: F) -> StateAnchor<Out>
            where
                Out: 'static,
                F: 'static,
                map_mut::MapMut<Self::Target, F, Out>: AnchorInner<Engine, Output=Out>,
            {
                ($(&self.$num.0,)+).map_mut(initial,f).into()
            }

            #[track_caller]
            fn then<F, Out>(self, f: F) -> StateAnchor<Out>
            where
                F: 'static,
                Out: 'static,
                then::Then<Self::Target, Out, F,Engine>: AnchorInner<Engine, Output=Out>,
            {
                ($(&self.$num.0,)+).then(f).into()
            }

            #[track_caller]
            fn refmap<F, Out>(self, f: F) -> StateAnchor<Out>
            where
                Out: 'static,
                F: 'static,
                refmap::RefMap<Self::Target, F>: AnchorInner<Engine, Output = Out>,
            {
                ($(&self.$num.0,)+).refmap(f).into()
            }

            #[track_caller]
            fn cutoff<F, Out>(self, f: F) -> StateAnchor<Out>
            where
                Out: 'static,
                F: 'static,
                cutoff::Cutoff<Self::Target, F>: AnchorInner<Engine, Output = Out>,
            {
                ($(&self.$num.0,)+).cutoff(f).into()
            }
        }
    }
}

impl_tuple_ext! {
    [O0, 0]
}

impl_tuple_ext! {
    [O0, 0]
    [O1, 1]
}

impl_tuple_ext! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
}

impl_tuple_ext! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
    [O3, 3]
}

impl_tuple_ext! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
    [O3, 3]
    [O4, 4]
}

impl_tuple_ext! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
    [O3, 3]
    [O4, 4]
    [O5, 5]
}

impl_tuple_ext! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
    [O3, 3]
    [O4, 4]
    [O5, 5]
    [O6, 6]
}

impl_tuple_ext! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
    [O3, 3]
    [O4, 4]
    [O5, 5]
    [O6, 6]
    [O7, 7]
}

impl_tuple_ext! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
    [O3, 3]
    [O4, 4]
    [O5, 5]
    [O6, 6]
    [O7, 7]
    [O8, 8]
}

// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct TopoKey {
    // pub ctx: Option<SlottedKey>,
    id: topo::CallId,
}

impl TopoKey {
    #[must_use]
    pub const fn new(id: topo::CallId) -> Self {
        Self { id }
    }
}

//TODO use this replace StorageKey for b a callback fn
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct CallbackFnStorageKey(TopoKey);

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum StorageKey {
    // SlottedKey(SlottedKey),
    TopoKey(TopoKey),
}

impl StorageKey {
    #[must_use]
    pub const fn as_topo_key(&self) -> Option<&TopoKey> {
        match self {
            Self::TopoKey(v) => Some(v),
            _ => None,
        }
    }
}

impl From<TopoKey> for StorageKey {
    fn from(v: TopoKey) -> Self {
        Self::TopoKey(v)
    }
}
impl From<&TopoKey> for StorageKey {
    fn from(v: &TopoKey) -> Self {
        Self::TopoKey(*v)
    }
}

#[track_caller]
//TODO replace use global_engine_get_anchor_val_with - Clone
fn global_engine_get_anchor_val<O: Clone + 'static>(anchor: &Anchor<O>) -> O {
    trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

    state_store_with(|g_state_store_refcell| g_state_store_refcell.borrow().engine_get(anchor))
}

#[track_caller]
fn try_global_engine_get_anchor_val<O: Clone + 'static>(anchor: &Anchor<O>) -> Result<O, Error> {
    trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

    state_store_with(|g_state_store_refcell| g_state_store_refcell.borrow().try_engine_get(anchor))
}

#[track_caller]
fn global_engine_get_anchor_val_with<O: Clone + 'static, F: FnOnce(&O) -> R, R>(
    anchor: &Anchor<O>,
    func: F,
) -> R {
    trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

    state_store_with(|g_state_store_refcell| {
        g_state_store_refcell.borrow().engine_get_with(anchor, func)
    })
}

fn set_in_callback<T: 'static + std::fmt::Debug + std::clone::Clone>(
    // store: &GStateStore,
    skip: &mut SkipKeyCollection,
    data_fn: impl FnOnce() -> T,
    current_id: &TopoKey,
) {
    state_store_with(|g_state_store_refcell| {
        trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

        g_state_store_refcell
            .borrow()
            .set_in_callback::<T>(skip, data_fn, current_id);
    });

    // execute_reaction_nodes(&StorageKey::TopoKey(current_id));
}

///
///  Uses the current topological id to create a new state accessor
///
fn state_exists_for_topo_id<T: 'static>(id: TopoKey) -> bool {
    state_store_with(|g_state_store_refcell| {
        trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

        g_state_store_refcell
            .borrow()
            .state_exists_with_id::<T>(StorageKey::TopoKey(id))
    })
}

//NOTE 覆盖 insert
fn insert_var_with_topo_id<T: 'static>(var: Var<T>, current_id: TopoKey) {
    state_store_with(|g_state_store_refcell| {
        trace!("G_STATE_STORE::borrow_mut:\n{}", Location::caller());

        g_state_store_refcell
            .borrow_mut()
            .insert_var_with_key::<T>(var, &StorageKey::TopoKey(current_id));
    });
}

#[track_caller]
fn or_insert_var_with_topo_id<F: FnOnce() -> T, T: 'static + std::fmt::Debug>(
    func: F,
    current_id: TopoKey,
) {
    state_store_with(
        #[track_caller]
        |g_state_store_refcell| {
            trace!("G_STATE_STORE::borrow_mut:\n{}", Location::caller());

            g_state_store_refcell
                .borrow_mut()
                .or_insert_var_with_key::<F, T>(func, &StorageKey::TopoKey(current_id));
        },
    );
}

fn read_val_with_topo_id<F: FnOnce(&T) -> R, T: 'static, R>(id: TopoKey, func: F) -> R {
    read_var_with_topo_id::<_, T, R>(id, |var: &Var<T>| func(var.get().as_ref()))
}

fn get_anchor_val_in_var_with_topo_id<T: 'static + std::clone::Clone>(id: TopoKey) -> T {
    state_store_with(|g_state_store_refcell| {
        trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

        let store = g_state_store_refcell.borrow();
        let var_with_sa = &store
            .opt_get_var_use_id::<StateAnchor<T>>(&StorageKey::TopoKey(id))
            .expect("You are trying to get a var state that doesn't exist in this context!");

        store.engine_get(var_with_sa.get().anchor())
    })
}

//maybe won't use  read_var_with_topo_id just directly use opt_get_state_and_bf_af_use_id
fn read_var_with_topo_id<F: FnOnce(&Var<T>) -> R, T: 'static, R>(id: TopoKey, func: F) -> R {
    state_store_with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
        trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

        let s = g_state_store_refcell.borrow();
        let var = s
            .opt_get_var_use_id::<T>(&StorageKey::TopoKey(id))
            .expect("You are trying to get a var state that doesn't exist in this context!");

        func(var)
    })
}
fn read_var_b_a_with_topo_id<F: FnOnce(VarOptBAfnCollectRef<T>) -> R, T: 'static, R>(
    id: &TopoKey,
    func: F,
) -> R {
    state_store_with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
        trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

        let store = g_state_store_refcell.borrow();
        let var_b_a = store
            .opt_get_var_and_bf_af_use_id::<T>(id)
            .expect("set_state_with_key: can't set state that doesn't exist in this context!");

        func(var_b_a)
    })
}

// type BoxSynCallFn<T> = Box<dyn Fn(&GStateStore, &SkipKeyCollection, &T)>;

/// 添加不添加 deps 都不会使 after before func 循环,
/// 但是添加deps 可以 在 deps-TopoKey 的StateVar drop时候 将该func drop,
/// 同时如果 运行func时刻, skip有deps的key,则不会运行该func
/// 如果 deps is some , 则返回 none , Rc储存在deps中
#[topo::nested]
fn insert_before_fn_common_in_topo<T: 'static + std::clone::Clone>(
    sv: &StateVar<T>,
    func: BoxSynCallBeforeFn<T>,
    init: bool,
    deps: &[TopoKey],
) -> Option<Rc<StorageKey>> {
    assert!(!deps.contains(sv.id()), "deps can't contain self");

    state_store_with(|g_state_store_refcell| {
        if init {
            trace!("G_STATE_STORE::borrow:\n{}", Location::caller());
            let store = &g_state_store_refcell.borrow();
            let var = store
                .opt_get_var_use_id::<T>(&StorageKey::TopoKey(sv.id))
                .unwrap();
            // .expect("set_state_with_key: can't set state that doesn't exist in this context!");

            let mut skip = smallvec![sv.id];

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
            .insert_before_fn_in_topo(&StorageKey::TopoKey(sv.id), func, deps)
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
fn insert_after_fn_common_in_topo<T: 'static + std::clone::Clone>(
    sv: &StateVar<T>,
    func: BoxSynCallAfterFn<T>,
    init: bool,
    deps: &[TopoKey],
) -> Option<Rc<StorageKey>> {
    assert!(!deps.contains(sv.id()), "deps can't contain self");

    state_store_with(|g_state_store_refcell| {
        if init {
            trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

            let store = &g_state_store_refcell.borrow();
            let var = store
                .opt_get_var_use_id::<T>(&StorageKey::TopoKey(sv.id))
                .unwrap();

            let mut skip = smallvec![sv.id];

            func(&mut skip, &*var.get());

            // after_fns_run(store, &skip, &(*var.get()), after_fns);
        }
        trace!("G_STATE_STORE::borrow_mut:\n{}", Location::caller());

        let mut store = g_state_store_refcell.borrow_mut();
        let fk = store
            .insert_after_fn_in_topo(&StorageKey::TopoKey(sv.id), func, deps)
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

// fn remove_after_fn<T: 'static + std::clone::Clone>(sv: &StateVar<T>, after_id: &StorageKey) {
//     state_store_with(|g_state_store_refcell| {
//         trace!("G_STATE_STORE::borrow_mut:\n{}", Location::caller());

//         let _span = debug_span!("G_STATE_STORE::remove_after_fn->borrow_mut").entered();

//         g_state_store_refcell
//             .borrow_mut()
//             .remove_after_fn::<T>(&StorageKey::TopoKey(sv.id), after_id);
//     });
// }
fn remove_after_fn<T: 'static + std::clone::Clone>(sv: &StateVar<T>, after_key: &StorageKey) {
    let fns = state_store_with(|g_state_store_refcell| {
        trace!("G_STATE_STORE::borrow_mut:\n{}", Location::caller());

        let _span = debug_span!("G_STATE_STORE::remove_after_fn->borrow_mut").entered();

        g_state_store_refcell
            .borrow()
            .get_after_fns_map::<T>(&StorageKey::TopoKey(sv.id))
    });
    fns.borrow_mut().remove(after_key);
}

// pub fn state_store_with<F, R>(func: F) -> R
// where
//     F: FnOnce(&GStateStore) -> R,
// {
//     trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

//     state_store_with(|g_state_store_refcell| func(&g_state_store_refcell.borrow()))
// }
#[must_use]
pub fn state_store() -> Rc<RefCell<GStateStore>> {
    state_store_with(std::clone::Clone::clone)
}

#[inline]
#[instrument(name = "G_STATE_STORE with", skip_all)]
pub fn state_store_with<F, R>(f: F) -> R
where
    F: FnOnce(&Rc<RefCell<GStateStore>>) -> R,
{
    G_STATE_STORE.with(f)
}

// fn read_var_with_topo_id_old<F: FnOnce(&Var<T>) -> R, T: 'static, R>(id: TopoKey, func: F) -> R {
//     let var = remove_state_with_topo_id::<T>(id)
//         .expect("You are trying to read a type state that doesn't exist in this context!");
//     let read = func(&var);
//     insert_var_with_topo_id(var, id);
//     read
// }
// fn remove_state_with_topo_id<T: 'static>(id: TopoKey) -> Option<Var<T>> {
//     state_store_with(|g_state_store_refcell| {
//         g_state_store_refcell
//             .borrow_mut()
//             .remove_state_with_id::<T>(&StorageKey::TopoKey(id))
//     })
// }

//-> &'static Location<'static>
#[topo::nested]
pub fn get_caller_location() {
    let loc = Location::caller();
    let id = topo::CallId::current();
    warn!("get_caller_location::at:\n{} \n id:{:?}", &loc, &id);
}
#[topo::nested]
pub fn get_caller_location2() {
    get_caller_location();
}

/// # Panics
/// new old not eq
#[must_use]
// #[topo::nested]
#[track_caller]
pub fn use_state<F: FnOnce() -> T, T>(func: F) -> StateVar<T>
where
    T: 'static + std::fmt::Debug,
{
    let loc = Location::caller();

    topo::call(move || {
        trace!("use_state::({}) \n", &std::any::type_name::<T>(),);
        let id = topo::CallId::current();
        let id = TopoKey { id };
        trace!("use_state::TopoKey:\n{:#?}", &id);
        #[cfg(debug_assertions)]
        {
            if state_exists_for_topo_id::<T>(id) {
                let old = StateVar::<T>::new(id);
                let old_v = old.get_rc();
                let v = func();

                warn!("this is checker: use_state call again, StateVar already settled state ->{} ,\n Location: {},\n old_v:{:?},\n new V:{:?}",std::any::type_name::<T>(),loc,old_v,v);
                if format!("{old_v:?}") != format!("{v:?}") {
                    warn!("val changed !!!!!!!!!!!!!!!!!!!!!!!!");
                }
                return StateVar::new(id);
            }
        }
        or_insert_var_with_topo_id(func, id);
        StateVar::new(id)
    })
}

#[must_use]
#[topo::nested]
pub fn reset_state<T>(data: T) -> StateVar<T>
where
    T: 'static + Clone,
{
    // info!(
    //     "use_state::({}) \n data: {:?}",
    //     &std::any::type_name::<T>(),
    //     &data
    // );

    let loc = Location::caller();
    trace!("use_state::at:\n{}", &loc);

    let id = topo::CallId::current();
    let id = TopoKey { id };

    if state_exists_for_topo_id::<T>(id) {
        let old = StateVar::<T>::new(id);
        old.set(data);
        old
    } else {
        insert_var_with_topo_id::<T>(Var::new(data), id);
        StateVar::new(id)
    }
}

// pub fn add_similar<T>(func:F)
// use overloadf::overload;
// #[overload]
// pub fn xx<T>(d: Var<T>) {}
// #[overload]
// pub fn xx<T>(dd: Anchor<T>) {}
// #[allow(unused)]
#[cfg(test)]
// #[allow(unused_variables)]
#[allow(clippy::many_single_char_names)]
#[allow(clippy::let_unit_value)]
#[allow(clippy::fallible_impl_from)]
#[allow(clippy::disallowed_types)]
#[allow(unused)]
mod state_test {

    use crate::topo;
    use std::collections::HashMap;

    use tracing::debug;

    use super::*;

    use color_eyre::eyre::Report;
    fn tracing_init() -> Result<(), Report> {
        use tracing_subscriber::prelude::*;
        fn theme() -> color_eyre::config::Theme {
            use color_eyre::{config::Theme, owo_colors::style};

            Theme::dark().active_line(style().bright_yellow().bold())
            // ^ use `new` to derive from a blank theme, or `light` to derive from a light theme.
            // Now configure your theme (see the docs for all options):
            // .line_number(style().blue())
            // .help_info_suggestion(style().red())
        }
        // let error_layer =
        // tracing_subscriber::fmt::layer().with_filter(tracing::metadata::LevelFilter::ERROR);

        let tree_layer = tracing_tree::HierarchicalLayer::new(2)
            .with_indent_lines(true)
            .with_indent_amount(4)
            .with_targets(true)
            .with_filter(tracing_subscriber::EnvFilter::new(
                // "emg_layout=debug,emg_layout[build inherited cassowary_generals_map],emg_layout[LayoutOverride]=error",
                // "[GElement-shaping]=debug",
                // "error,[sa gel in map clone]=debug",
                // "emg_state=off,[anchors-dirty]=debug,cassowary=off",
                // ,
                "[manually_drop]=debug,[sv to svp]=debug,[clock.remove_after_fn]=debug",
                // emg_layout::animation::tests=off
                // "error",
            ));

        tracing_subscriber::registry()
            // .with(layout_override_layer)
            // .with(event_matching_layer)
            // .with(touch_layer)
            .with(tracing_error::ErrorLayer::default())
            .with(tree_layer)
            // .with(out_layer)
            .try_init()?;

        // color_eyre::install()
        color_eyre::config::HookBuilder::new()
            .theme(theme())
            .install()
    }

    #[derive(Clone, Debug)]
    struct TT(String);
    impl From<i32> for TT {
        fn from(v: i32) -> Self {
            Self(format!("{v}"))
        }
    }
    impl From<TT> for i32 {
        fn from(v: TT) -> Self {
            let s = v.0;

            s.parse::<Self>().unwrap()
        }
    }

    impl From<u32> for TT {
        fn from(v: u32) -> Self {
            Self(format!("{v}"))
        }
    }
    impl From<TT> for u32 {
        fn from(v: TT) -> Self {
            let s = v.0;

            s.parse::<Self>().unwrap()
        }
    }

    #[test]
    fn id_test() {
        let a = use_state(|| 1);
        let b = use_state(|| 2);
        assert_ne!(a.id(), b.id());
    }

    #[test]
    // #[wasm_bindgen_test]
    fn callback() {
        let _g = tracing_init();
        let a = use_state(|| 1);
        let b = a.build_similar_use_into_in_topo::<TT>();
        debug!("init: a:{:?} b:{:?}", &a, &b);
        a.set(2);
        debug!("a set 2 : a:{:?} b:{:?}", &a, &b);
        assert_eq!(format!("{:?}", a.get()), b.get().0);
        let c = a.build_bi_similar_use_into_in_topo::<TT>();
        debug!("build c : a:{:?} b:{:?} c:{:?}", &a, &b, &c);
        c.set(TT("3".to_string()));
        debug!("c set '3' : a:{:?} b:{:?} c:{:?}", &a, &b, &c);
        a.set(9);
        debug!("a set 9: a:{:?} b:{:?} c:{:?}", &a, &b, &c);
        let d = c.build_similar_use_into_in_topo::<i32>();

        assert_eq!(a.get(), d.get());
        a.set(19);
        assert_eq!(a.get(), d.get());
    }

    #[test]
    #[topo::nested]
    fn callback2_clone() {
        let _g = tracing_init();

        let a = use_state(|| 1);
        let a_2 = use_state(|| 11);
        let update_id_a_2 = TopoKey::new(topo::call(topo::CallId::current));
        trace!("update_id_a_2:{:#?}", &update_id_a_2);
        a_2.insert_before_fn_in_topo(
            move |_skip, _current, _value| {
                // println!("current:{:?}", &current);
                // println!("value:{}", value);
                // assert_eq!(current, &Some(Rc::new(1)));
                // assert_eq!(*value, 2);
            },
            false,
            &[],
        )
        .unwrap();
        // a_2.set(2);
        trace!("==================build_bi_similar_use_into_in_topo========================");

        let _b = a.build_bi_similar_use_into_in_topo::<TT>();
    }
    #[test]
    // #[wasm_bindgen_test]
    fn callback2() {
        #[allow(clippy::let_unit_value)]
        let _g = tracing_init();

        let a = use_state(|| 1);
        let a_2 = use_state(|| 1);
        let update_id_a_2 = TopoKey::new(topo::call(topo::CallId::current));

        a_2.insert_before_fn_in_topo(
            move |_skip, current, value| {
                println!("current:{:?}", &current);
                println!("value:{value}");
                assert_eq!(current, &Some(Rc::new(1)));
                assert_eq!(*value, 2);
            },
            false,
            &[],
        )
        .unwrap();
        a_2.set(2);

        let b = a.build_bi_similar_use_into_in_topo::<TT>();
        let c = b.build_similar_use_into_in_topo::<i32>();
        let d = b.build_similar_use_into_in_topo::<i32>();
        d.insert_before_fn_in_topo(
            move |skip, _current, value| {
                c.seting_in_b_a_callback(skip, || *value);
            },
            true,
            &[c.id],
        );

        let update_id = TopoKey::new(topo::call(topo::CallId::current));

        c.insert_before_fn_in_topo(
            move |skip, current, value| {
                println!("c -> before_fns 1 -> set a:{:?}", &value);

                a.seting_in_b_a_callback(skip, || *value);
            },
            true,
            &[a.id],
        );

        let update_id2 = TopoKey::new(topo::call(topo::CallId::current));

        //NOTE same a set_in_callback will ignored at second times
        c.insert_before_fn_in_topo(
            move |skip, current, value| {
                println!("c -> before_fns 2 -> set a:{:?}", value + 1);
                a.seting_in_b_a_callback(skip, || (value + 1));
            },
            true,
            &[],
        )
        .expect("");
        let e = use_state(|| 11);
        c.insert_after_fn_in_topo(
            move |skip, v| {
                e.seting_in_b_a_callback(skip, || *v);
            },
            true,
            &[e.id],
        );

        println!("e:{:?}", &e);

        println!("init: a:{:?} b:{:?} c:{:?} d:{:?}", &a, &b, &c, &d);

        a.set(2);
        println!(
            "a set 2--------------: a:{:?} b:{:?} c:{:?} d:{:?} e:{:?}",
            &a, &b, &c, &d, &e
        );
        assert_eq!(a.get(), c.get());
        assert_eq!(a.get(), d.get());
        assert_eq!(a.get(), e.get());
        c.set(3);
        assert_eq!(a.get(), c.get());
        assert_eq!(a.get(), d.get());
        d.set(4);
        assert_eq!(a.get(), c.get());
        assert_eq!(a.get(), d.get());
        assert_eq!(a.get(), e.get());
        c.remove_after_fn(e.id);
    }
    #[test]
    // #[wasm_bindgen_test]
    fn update() {
        let a = use_state(|| 111);
        a.update(|aa| *aa += 1);
        println!("{}", &a);
        assert_eq!(112, a.get());
        a.update(|aa| *aa -= 2);
        println!("{}", &a);
        assert_eq!(110, a.get());
    }

    // #[wasm_bindgen_test]
    #[test]

    fn sa_in_sv() {
        let x = use_state(|| 1);
        let xw = x.watch();
        let a = use_state(|| xw.clone());
        println!("{a}");
        println!("{}", a.get());
        assert_eq!(1, a.get());
    }
    #[test]
    fn macros() {
        let ffss = dict! {1=>1};
        println!("{ffss:?}");
    }
    #[allow(clippy::similar_names)]
    #[test]
    fn xx() {
        let a = use_state(|| 99);

        let b = a.watch();
        let b2 = a.watch();
        let cadd = b.map(|x| *x + 1);
        let cadd2 = b.map(|x| *x + 2);
        let cadd_c = cadd.clone();
        let cadd2_c = cadd2;
        let c = b.map(|x| format!("{x}"));
        let d = b.then(move |x| {
            if *x > 1 {
                b2.anchor().clone()
            } else {
                cadd.anchor().clone()
            }
        });
        debug!("========================{:?}", cadd_c.get());
        debug!("========================{:?}", cadd2_c.get());

        assert_eq!(cadd_c.get(), 100);
        assert_eq!(cadd2_c.get(), 101);
        assert_eq!(99, d.get());
        a.set(1);
        assert_eq!(2, d.get());

        let dd = Var::new(99);
        let ddw = dd.watch();
        let ddw2 = dd.watch();
        let dcadd = ddw.map(|x| *x + 1);
        let dc = ddw.map(|x| format!("{x}"));

        let ddw3 = ddw.then(move |x| if *x > 1 { ddw2.clone() } else { dcadd.clone() });
    }

    #[test]
    fn map_test() {
        let mut a = HashMap::new();
        let v = vec![1];
        a.insert(v, 1);
        assert_eq!(a.get(&vec![1]), Some(&1));
    }

    #[test]
    fn test_map_eq() {
        let mut dict = Dict::new();
        let a = use_state(|| dict.clone());
        let a_node1 = use_state(|| 1);
        let a_node2 = use_state(|| 2);
        let a_node0 = use_state(|| 0);

        let b = a.watch().map_(|_, x: &StateVar<i32>| {
            x.set(x.get() + 1);
            *x
        });

        dict.insert("a".to_string(), a_node1);
        dict.insert("b".to_string(), a_node2);
        a.set(dict.clone());

        println!("a:{:#?}", &a);
        println!("b:{:#?}", &b);
        a_node1.set(33);
        println!("a-edit:{:#?}", &a);
        println!("b-edit:{:#?}", &b);

        a_node1.set(333);
        println!("=========2 a-edit:{:#?}", &a);
        println!("=========2 b-edit:{:#?}", &b);

        if let Some(av) = dict.get_mut("a") {
            println!("get a");
            *av = a_node0;
            a.set(dict.clone());
        }
        println!("=========3 a-edit:{:#?}", &a);
        println!("=========3 b-edit:{:#?}", &b);
        println!("===================");
        println!("=========3 a-edit:{:#?}", &a);
        println!("=========3 b-edit:{:#?}", &b);
    }

    #[test]
    fn test_map_anchor_eq() {
        let mut dict = Dict::new();
        let a = use_state(|| dict.clone());
        let a_node1 = use_state(|| 1);
        let a_node2 = use_state(|| 2);
        let a_node0 = use_state(|| 0);

        let b = a.watch().map_(|_, x: &StateAnchor<i32>| x.map(|xx| xx + 1));

        dict.insert("a".to_string(), a_node1.watch());
        dict.insert("b".to_string(), a_node2.watch());
        a.set(dict.clone());

        println!("a->:{:#?}", &a);
        println!("b->:{:#?}", &b);
        a_node1.set(33);
        println!("a-edit:{:#?}", &a);
        println!("b-edit:{:#?}", &b);

        a_node1.set(333);
        println!("=========2 a-edit:{:#?}", &a);
        println!("=========2 b-edit:{:#?}", &b);
        if let Some(av) = dict.get_mut("a") {
            println!("get a");
            *av = a_node0.watch();
            a.set(dict.clone());
        }

        println!("=========3 a-edit:{:#?}", &a);
        println!("=========3 b-edit:{:#?}", &b);
    }

    #[test]
    #[topo::nested]
    fn drop_test() {
        let _g = tracing_init();
        let a = use_state(|| 1);

        let fk = a
            .insert_before_fn_in_topo(|_, _, _| println!("xxx"), false, &[])
            .unwrap();
        let fk_c = {
            println!("fk: {:?}", &fk);

            *fk
        };

        a.link_callback_drop(fk);
        // .unwrap()
        // .unwrap();

        state_store_with(|g_state_store_refcell| {
            let store = g_state_store_refcell.borrow();
            println!("anymap len:{:#?}", store.anymap.len());
            println!("id_to_key_map len:{:#?}", store.id_to_key_map.len());
            println!("primary_slotmap len:{:#?}", store.primary_slotmap.len());
            println!(
                "dep_require_map len:{:#?}",
                store.b_a_fn_drop_link_map.len()
            );
        });
        let topo_key = StorageKey::TopoKey(*a.id());
        let key = state_store_with(|g_state_store_refcell| {
            let store = g_state_store_refcell.borrow();
            *store.id_to_key_map.get(&topo_key).unwrap()
        });
        println!("var topo_key: {:?}", a.id());
        println!("var key: {:?}", &key);

        state_store_with(|g_state_store_refcell| {
            let store = g_state_store_refcell.borrow();
            let var_map = store.get_secondarymap::<i32>().unwrap();
            println!("map len:{:#?}", var_map.len());

            for x in var_map.iter() {
                println!("x:{x:#?}");
            }

            if let Some(b_map) = store.get_before_secondarymap::<i32>() {
                let f = b_map.get(key).unwrap();
                let before_fn_weak_map = f.borrow();
                // let func = borrow.get(&fk);
                println!("before fn map len:{:?}", before_fn_weak_map.len());
                for (k, f) in before_fn_weak_map.iter() {
                    println!("before fn map:{k:#?}");
                }
                assert!(before_fn_weak_map.len() == 1);
                let (fk_got, f) = before_fn_weak_map.get(&fk_c).unwrap();
                println!("fk_got:{fk_got:?}");
            }

            if let Some(drop_cb_deps) = store.b_a_fn_drop_link_map.get(key) {
                for fk_linked in drop_cb_deps.iter() {
                    println!("fk_linked:{fk_linked:?}");
                }
                assert!(drop_cb_deps.len() == 1);
            }
        });

        a.manually_drop();

        state_store_with(|g_state_store_refcell| {
            let store = g_state_store_refcell.borrow();
            println!("anymap len:{:#?}", store.anymap.len());
            println!("id_to_key_map len:{:#?}", store.id_to_key_map.len());
            println!("primary_slotmap len:{:#?}", store.primary_slotmap.len());
            println!(
                "dep_require_map len:{:#?}",
                store.b_a_fn_drop_link_map.len()
            );

            // assert_eq!(0, store.anymap.len());
            assert_eq!(0, store.id_to_key_map.len());
            assert_eq!(0, store.primary_slotmap.len());
        });

        state_store_with(|g_state_store_refcell| {
            {
                let store = g_state_store_refcell.borrow();
                let var_map = store.get_secondarymap::<i32>().unwrap();
                println!("map len:{:#?}", var_map.len());

                for x in var_map.iter() {
                    println!("x:{x:#?}");
                }
            }
            let mut store = g_state_store_refcell.borrow();

            if let Some(b_map) = store.get_before_secondarymap::<i32>() {
                let f = b_map.get(key).unwrap();
                let mut before_fn_weak_map = f.borrow_mut();

                assert!(before_fn_weak_map.get(&fk_c).is_none());

                println!("before fn map len:{:?}", before_fn_weak_map.len());
                println!(
                    "before fn map  load_factor {}",
                    before_fn_weak_map.load_factor()
                );
                for (k, f) in before_fn_weak_map.iter() {
                    println!("before fn map:{k:#?}");
                }
                before_fn_weak_map.remove_expired();

                assert!(before_fn_weak_map.len() == 0);

                // let func = borrow.get(&fk);
            }
        });
    }
    struct DD;
    impl Drop for DD {
        fn drop(&mut self) {
            println!("drop");
            G_STATE_STORE.with(|x| {
                println!("drop... in G_STATE_STORE");
            });
            println!("drop ...");
        }
    }

    #[test]
    fn g_state_store_test() {
        G_STATE_STORE.with(|x| {
            println!("in G_STATE_STORE");
            let d = DD {};
            drop(d);
            G_STATE_STORE.with(|x| {
                println!("in G_STATE_STORE");
                let d = DD {};
                drop(d);
            });
        });
    }
}
