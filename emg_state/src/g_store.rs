use std::{
    cell::RefCell,
    hash::BuildHasherDefault,
    rc::{Rc, Weak},
};

use crate::{
    general_struct::{LocationEngineGet, StorageKey, TopoKey},
    general_traits::VTFn,
    topo,
};
use std::ops::Deref;

use emg_common::{SVec::ToSmallVec, SmallVec};

use crate::error::Error;

use tracing::{instrument, trace_span};

use tracing::trace;

use anchors::singlethread::Anchor;

use tracing::debug;

use anchors::singlethread::Engine;

use slotmap::{Key, SlotMap, SparseSecondaryMap};

use emg_hasher::CustomHasher;
use indexmap::IndexMap as HashMap;
use slotmap::DefaultKey;
use weak_table::WeakKeyHashMap;

// ─────────────────────────────────────────────────────────────────────────────

thread_local! {
    pub static G_STATE_STORE: Rc<RefCell<GStateStore>> =Rc::new( RefCell::new(
        GStateStore::default()
    ));
}
// ─────────────────────────────────────────────────────────────────────────────
pub type DepsVarTopoKey = SmallVec<[TopoKey; 1]>;
type FnKey = Weak<StorageKey>;

type SynCallAfterFnsMap<VOA> = WeakKeyHashMap<
    FnKey,
    (DepsVarTopoKey, BoxSynCallAfterFn<VOA>),
    BuildHasherDefault<CustomHasher>,
>;
type SynCallBeforeFnsMap<VOA> = WeakKeyHashMap<
    FnKey,
    (DepsVarTopoKey, BoxSynCallBeforeFn<VOA>),
    BuildHasherDefault<CustomHasher>,
>;
// NOTE: rc RefCell 为了 可以移出borrowmut store block后操作
pub type RCRSynCallAfterFnsMap<VOA> = Rc<RefCell<SynCallAfterFnsMap<VOA>>>;
pub type RCRSynCallBeforeFnsMap<VOA> = Rc<RefCell<SynCallBeforeFnsMap<VOA>>>;

pub type BoxSynCallAfterFn<VOA> = Box<dyn Fn(&mut SkipKeyCollection, &VOA)>;
pub type BoxSynCallBeforeFn<VOA> = Box<dyn Fn(&mut SkipKeyCollection, &Option<Rc<VOA>>, &VOA)>;

pub type VarOptBAfnCollectRef<'a, VT, VOA> = (
    &'a VT,
    Option<&'a RCRSynCallBeforeFnsMap<VOA>>,
    Option<&'a RCRSynCallAfterFnsMap<VOA>>,
);

type VarBeforeSecMap<VOA> =
    SparseSecondaryMap<DefaultKey, RCRSynCallBeforeFnsMap<VOA>, BuildHasherDefault<CustomHasher>>;
type VarAfterSecMap<VOA> =
    SparseSecondaryMap<DefaultKey, RCRSynCallAfterFnsMap<VOA>, BuildHasherDefault<CustomHasher>>;

pub type SkipKeyCollection = SmallVec<[TopoKey; 3]>;

type VarSecMap<VT> = SparseSecondaryMap<DefaultKey, VT, BuildHasherDefault<CustomHasher>>;

type VarDepRequireSecMap =
    SparseSecondaryMap<DefaultKey, SmallVec<[Rc<StorageKey>; 2]>, BuildHasherDefault<CustomHasher>>;

// ─────────────────────────────────────────────────────────────────────────────

#[allow(clippy::module_name_repetitions)]
pub struct GStateStore {
    pub(crate) anymap: anymap::AnyMap,
    pub(crate) id_to_key_map: HashMap<StorageKey, DefaultKey, BuildHasherDefault<CustomHasher>>,
    // pub id_to_key_map: HashMap<StorageKey, DefaultKey>,
    pub(crate) primary_slotmap: SlotMap<DefaultKey, StorageKey>,
    pub(crate) b_a_fn_drop_link_map: VarDepRequireSecMap,
    pub(crate) engine: RefCell<Engine>,
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

impl GStateStore {
    /// # Panics
    ///
    /// Will panic if engine cannot `borrow_mut`
    #[track_caller]
    pub(crate) fn engine_get<T: Clone + 'static>(&self, anchor: &Anchor<T>) -> T {
        trace!("engine_get: {}", &std::any::type_name::<T>());
        let _g = trace_span!("-> enging_get", "type: {}", &std::any::type_name::<T>()).entered();

        self.engine.try_borrow_mut().map_or_else(
            |err| {
                let x = &*illicit::expect::<LocationEngineGet>();
                panic!(
                    "can't borrow_mut engine err: {} , for anchor type: {},\n at:{}",
                    &err,
                    &std::any::type_name::<T>(),
                    x.0
                )
            },
            |mut e| {
                let _gg = trace_span!(
                    "-> enging_get:engine borrow_muted , now getting.. ",
                    "type: {}",
                    &std::any::type_name::<T>()
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
    pub(crate) fn try_engine_get<T: Clone + 'static>(
        &self,
        anchor: &Anchor<T>,
    ) -> Result<T, Error> {
        self.engine.try_borrow_mut().map_or_else(
            |err| {
                let x = *illicit::expect::<LocationEngineGet>().deref();

                #[cfg(feature = "engine-try-borrow-mut-no-panic")]
                {
                    let e = format!(
                        "can't borrow_mut engine err: {} , for anchor type: {}",
                        &err,
                        &std::any::type_name::<T>(),
                    );
                    Err(Error::EngineAlreadyMut(x, e))
                }
                #[cfg(not(feature = "engine-try-borrow-mut-no-panic"))]
                {
                    panic!(
                        "can't borrow_mut engine err: {} , for anchor type: {},\n at:{}",
                        &err,
                        &std::any::type_name::<T>(),
                        x.0
                    )
                }
            },
            |mut e| {
                let _gg = trace_span!(
                    "-> enging_get:engine borrow_muted , now getting.. ",
                    "type: {}",
                    &std::any::type_name::<T>()
                )
                .entered();

                Ok(e.get(anchor))
            },
        )
    }

    pub(crate) fn engine_get_with<T: Clone + 'static, F: FnOnce(&T) -> R, R>(
        &self,
        anchor: &Anchor<T>,
        func: F,
    ) -> R {
        trace!("engine_get_with: {}", &std::any::type_name::<T>());
        let _g = trace_span!(
            "-> engine_get_with",
            "type: {}",
            &std::any::type_name::<T>()
        )
        .entered();

        self.engine.try_borrow_mut().map_or_else(
            |err| {
                panic!(
                    "can't borrow_mut engine err: {} , for anchor type: {}",
                    &err,
                    &std::any::type_name::<T>()
                )
            },
            |mut e| {
                let _gg = trace_span!(
                    "-> engine_get_with:engine borrow_muted , now getting.. ",
                    "type: {}",
                    &std::any::type_name::<T>()
                )
                .entered();

                e.get_with(anchor, func)
            },
        )
    }

    pub(crate) fn state_exists_with_id<VT: 'static>(&self, id: StorageKey) -> bool {
        match (self.id_to_key_map.get(&id), self.get_secondarymap::<VT>()) {
            (Some(existing_key), Some(existing_secondary_map)) => {
                existing_secondary_map.contains_key(*existing_key)
            }
            (_, _) => false,
        }
    }

    #[must_use]
    pub(crate) fn get_secondarymap<VT: 'static>(&self) -> Option<&VarSecMap<VT>> {
        self.anymap.get::<VarSecMap<VT>>()
    }
    pub(crate) fn remove_secondarymap<VT: 'static>(&mut self) {
        self.anymap.remove::<VarSecMap<VT>>();
    }

    pub(crate) fn get_before_secondarymap<VOA: 'static>(&self) -> Option<&VarBeforeSecMap<VOA>> {
        self.anymap.get::<VarBeforeSecMap<VOA>>()
    }
    pub(crate) fn remove_before_secondarymap<VOA: 'static>(&mut self) {
        self.anymap.remove::<VarBeforeSecMap<VOA>>();
    }
    pub(crate) fn get_after_secondarymap<VOA: 'static>(&self) -> Option<&VarAfterSecMap<VOA>> {
        self.anymap.get::<VarAfterSecMap<VOA>>()
    }
    pub(crate) fn remove_after_secondarymap<VOA: 'static>(&mut self) {
        self.anymap.remove::<VarAfterSecMap<VOA>>();
    }
    pub(crate) fn get_mut_secondarymap<VT: 'static>(&mut self) -> Option<&mut VarSecMap<VT>> {
        self.anymap.get_mut::<VarSecMap<VT>>()
    }
    pub(crate) fn get_mut_before_secondarymap<VOA: 'static>(
        &mut self,
    ) -> Option<&mut VarBeforeSecMap<VOA>> {
        trace!(
            "-- get_mut_before_secondarymap: {}",
            &std::any::type_name::<VOA>()
        );
        self.anymap.get_mut::<VarBeforeSecMap<VOA>>()
    }
    pub(crate) fn get_mut_after_secondarymap<VOA: 'static>(
        &mut self,
    ) -> Option<&mut VarAfterSecMap<VOA>> {
        self.anymap.get_mut::<VarAfterSecMap<VOA>>()
    }

    pub(crate) fn register_secondarymap<VT: 'static>(&mut self) {
        let sm: VarSecMap<VT> = VarSecMap::<VT>::default();
        self.anymap.insert(sm);
    }
    pub(crate) fn register_before_secondarymap<VOA: 'static>(&mut self) {
        let sm: VarBeforeSecMap<VOA> = VarBeforeSecMap::<VOA>::default();
        self.anymap.insert(sm);
    }
    pub(crate) fn register_after_secondarymap<VOA: 'static>(&mut self) {
        let sm: VarAfterSecMap<VOA> = VarAfterSecMap::<VOA>::default();
        self.anymap.insert(sm);
    }

    #[inline]
    pub(crate) fn get_var_b_a_fn_collect<VT: 'static, VOA: 'static>(
        &self,
        // data: T,
        current_id: TopoKey,
    ) -> VarOptBAfnCollectRef<VT, VOA> {
        trace!("get_state_use_key_and_start_set_run_cb");
        //unwrap or default to keep borrow checker happy
        self.opt_get_var_and_bf_af_use_id::<VT, VOA>(current_id)
            .expect("set_state_with_key: can't set state that doesn't exist in this context!")

        // trace!("start_set_var_and_before_after");

        // start_set_var_and_before_after(current_id, var, data, before_fns, after_fns);
    }
    pub(crate) fn set_in_callback<VT: VTFn<VOA> + 'static, VOA: Clone + 'static>(
        &self,
        skips: &mut SkipKeyCollection,
        data_fn: impl FnOnce() -> VOA,
        current_id: TopoKey,
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
        if skips.contains(&current_id) {
            return;
        }
        skips.push(current_id);
        // }
        let data = data_fn();

        //unwrap or default to keep borrow checker happy
        let (var, opt_before_fns, opt_after_fns) = self
            .opt_get_var_and_bf_af_use_id::<VT, VOA>(current_id)
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
    // pub(crate) fn insert_var_with_key<VT: 'static>(&mut self, var: VT, current_id: &StorageKey) {
    //     //unwrap or default to keep borrow checker happy
    //     trace!(
    //         "use_state::insert_var_with_key::({})",
    //         &std::any::type_name::<VT>()
    //     );

    //     let key = self
    //         .id_to_key_map
    //         .get(current_id)
    //         .copied()
    //         .unwrap_or_default();

    //     if key.is_null() {
    //         let key = self.primary_slotmap.insert(*current_id);
    //         self.id_to_key_map.insert(*current_id, key);
    //         if let Some(sec_map) = self.get_mut_secondarymap::<VT>() {
    //             sec_map.insert(key, var);
    //         } else {
    //             self.register_secondarymap::<VT>();
    //             self.get_mut_secondarymap::<VT>().unwrap().insert(key, var);
    //         }
    //     } else if let Some(existing_secondary_map) = self.get_mut_secondarymap::<VT>() {
    //         existing_secondary_map.insert(key, var);
    //     } else {
    //         // key ! null  && T not find
    //         // self.register_secondarymap::<T>();
    //         // self.get_mut_secondarymap::<T>().unwrap().insert(key, var);
    //         panic!("panic current using find why here 2");
    //     }
    // }

    #[track_caller]
    #[instrument(level = "debug", skip_all)]
    pub(crate) fn or_insert_var_with_key<VT: VTFn<VOA> + 'static, VOA, F: FnOnce() -> VOA>(
        &mut self,
        func: F,
        current_id: &StorageKey,
    ) {
        //unwrap or default to keep borrow checker happy
        trace!(
            "use_state::or_insert_var_with_key::({})",
            &std::any::type_name::<VOA>()
        );

        if let Some(k) = self.id_to_key_map.get(current_id).copied() {
            self.get_mut_secondarymap::<VT>().map_or_else(
                || {
                    panic!("panic current using find why here 2");
                },
                |existing_secondary_map| {
                    if !existing_secondary_map.contains_key(k) {
                        existing_secondary_map.insert(k, VT::new(func()));
                    }
                },
            );
        } else {
            let key = self.primary_slotmap.insert(*current_id);
            debug!(id=?current_id, "or_insert_var_with_key::insert new key");
            let data = func();
            debug!("data ty ======>{}", std::any::type_name::<VOA>());
            // debug!("data ======>{:?}", &data);

            self.id_to_key_map.insert(*current_id, key);

            if let Some(sec_map) = self.get_mut_secondarymap::<VT>() {
                sec_map.insert(key, VT::new(data));
            } else {
                self.register_secondarymap::<VT>();
                self.get_mut_secondarymap::<VT>()
                    .unwrap()
                    .insert(key, VT::new(data));
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

    // ─────────────────────────────────────────────────────────────────────

    #[tracing::instrument(skip(self, func), level = "debug")]
    #[topo::nested]
    pub fn insert_before_fn_in_topo<VOA: 'static>(
        &mut self,
        current_id: &StorageKey,
        func: BoxSynCallBeforeFn<VOA>,
        deps: &[TopoKey],
        //TODO add store
    ) -> Result<Rc<StorageKey>, String> {
        // assert_ne!(current_id, before_fn_id);

        let before_fn_id = Rc::new(StorageKey::TopoKey(TopoKey {
            id: topo::call(topo::CallId::current),
        }));

        let key = self.id_to_key_map.get(current_id).copied();
        let before_secondary_map = self.get_mut_before_secondarymap::<VOA>();
        match (key, before_secondary_map) {
            (Some(existing_key), Some(existing_before_secondary_map)) => {
                let rcr_fns = existing_before_secondary_map
                    .entry(existing_key)
                    .unwrap()
                    .or_insert_with(|| {
                        Rc::new(RefCell::new(
                            SynCallBeforeFnsMap::<VOA>::with_capacity_and_hasher(
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
                self.register_before_secondarymap::<VOA>();

                let mut new_map = SynCallBeforeFnsMap::<VOA>::with_capacity_and_hasher(
                    1,
                    BuildHasherDefault::<CustomHasher>::default(),
                );
                new_map.insert(before_fn_id.clone(), (deps.to_smallvec(), func));

                self.get_mut_before_secondarymap::<VOA>()
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
    pub fn insert_after_fn_in_topo<VOA: 'static>(
        &mut self,
        current_id: &StorageKey,
        func: BoxSynCallAfterFn<VOA>,
        deps: &[TopoKey],
    ) -> Result<Rc<StorageKey>, String> {
        // assert_ne!(current_id, after_fn_id);

        let after_fn_id = Rc::new(StorageKey::TopoKey(TopoKey {
            id: topo::call(topo::CallId::current),
        }));

        let key = self.id_to_key_map.get(current_id).copied();
        let after_secondary_map = self.get_mut_after_secondarymap::<VOA>();
        match (key, after_secondary_map) {
            (Some(existing_key), Some(existing_after_secondary_map)) => {
                let rcr_fns = existing_after_secondary_map
                    .entry(existing_key)
                    .unwrap()
                    .or_insert_with(|| {
                        Rc::new(RefCell::new(
                            SynCallAfterFnsMap::<VOA>::with_capacity_and_hasher(
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
                self.register_after_secondarymap::<VOA>();

                let mut new_map = SynCallAfterFnsMap::<VOA>::with_capacity_and_hasher(
                    1,
                    BuildHasherDefault::<CustomHasher>::default(),
                );
                new_map.insert(after_fn_id.clone(), (deps.to_smallvec(), func));

                self.get_mut_after_secondarymap::<VOA>()
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
    pub fn get_after_fns_map<VOA: 'static>(
        &self,
        current_id: &StorageKey,
    ) -> RCRSynCallAfterFnsMap<VOA> {
        let key = self.id_to_key_map.get(current_id).copied();
        let secondarymap = self.get_after_secondarymap::<VOA>();
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

    pub(crate) fn opt_get_var_and_bf_af_use_id<VT: 'static, VOA: 'static>(
        &self,
        current_id: impl Into<StorageKey>,
    ) -> Result<VarOptBAfnCollectRef<VT, VOA>, Error> {
        let storage_key = current_id.into();
        match (
            self.id_to_key_map.get(&storage_key),
            self.get_secondarymap::<VT>(),
        ) {
            (Some(existing_key), Some(existing_secondary_map)) => {
                let v = existing_secondary_map
                    .get(*existing_key)
                    .ok_or(Error::SecMapNoKey(*existing_key, storage_key))?;

                let b = self.get_before_secondarymap::<VOA>().and_then(
                    |existing_before_secondary_map| {
                        existing_before_secondary_map.get(*existing_key)
                    },
                );

                let a =
                    self.get_after_secondarymap::<VOA>()
                        .and_then(|existing_after_secondary_map| {
                            existing_after_secondary_map.get(*existing_key)
                        });

                Ok((v, b, a))
            }
            (None, None) => Err(Error::StoreNoKeyNoVarMapForType(
                storage_key,
                std::any::type_name::<VOA>(),
            )),
            (None, Some(_)) => Err(Error::StoreNoKey(storage_key)),
            (Some(_), None) => Err(Error::StoreNoVarMapForType(std::any::type_name::<VOA>())),
        }
    }
    #[instrument(level = "debug", skip_all)]
    pub fn opt_get_var_use_id<VT: 'static>(&self, current_id: &StorageKey) -> Option<&VT> {
        match (
            self.id_to_key_map.get(current_id),
            self.get_secondarymap::<VT>(),
        ) {
            (Some(existing_key), Some(existing_secondary_map)) => {
                existing_secondary_map.get(*existing_key)
            }
            _ => None,
        }
    }

    // fn remove_state_with_id<T: 'static>(&mut self, current_id: &StorageKey) -> Option<VarEA<T>> {
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

    pub(crate) fn link_callback_drop(&mut self, dep_topo_key: TopoKey, b_a_key: Rc<StorageKey>) {
        let key = self
            .id_to_key_map
            .get(&StorageKey::TopoKey(dep_topo_key))
            .copied()
            .unwrap();
        let entry = self.b_a_fn_drop_link_map.entry(key).unwrap();
        entry.or_default().push(b_a_key);
    }
}
