/*
 * @Author: Rais
 * @Date: 2021-03-15 17:10:47
 * @LastEditTime: 2021-06-21 19:04:36
 * @LastEditors: Rais
 * @Description:
 */

pub use anchors::singlethread::Anchor;
pub use anchors::singlethread::Var;
use anchors::{
    expert::{cutoff, map, map_mut, refmap, then, AnchorInner},
    singlethread::{Engine, MultiAnchor},
};
use anymap::any::Any;
use tracing::debug;

use std::hash::BuildHasherDefault;
// use im::HashMap;
use std::{cell::RefCell, clone::Clone, marker::PhantomData, rc::Rc};
use tracing::{trace, trace_span};
// use delegate::delegate;
use slotmap::{DefaultKey, Key, SlotMap, SparseSecondaryMap};

thread_local! {
    static G_STATE_STORE: Rc<RefCell<GStateStore>> =Rc::new( RefCell::new(
        GStateStore::default()
    ));
}

// ────────────────────────────────────────────────────────────────────────────────
//TODO build benchmark for AHash FxHash
// use ahash::AHashMap as HashMap;
// use ahash::AHasher as CustomHasher;
use indexmap::IndexMap as HashMap;
// use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHasher as CustomHasher;
// ────────────────────────────────────────────────────────────────────────────────

#[allow(clippy::module_name_repetitions)]
pub struct GStateStore {
    anymap: anymap::Map<dyn Any>,
    id_to_key_map: HashMap<StorageKey, DefaultKey>,
    primary_slotmap: SlotMap<DefaultKey, StorageKey>,
    engine: RefCell<Engine>,
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
            anymap: anymap::Map::new(),
            id_to_key_map: HashMap::default(),
            primary_slotmap: SlotMap::new(),
            engine: RefCell::new(Engine::new()),
        }
    }
}

type VarSimMap<T> = (Var<T>, SynCallFnsMap<T>, SynCallFnsMap<T>);

type VarSecMap<T> = SparseSecondaryMap<DefaultKey, VarSimMap<T>, BuildHasherDefault<CustomHasher>>;

impl GStateStore {
    /// # Panics
    ///
    /// Will panic if engine cannot `borrow_mut`
    fn engine_get<O: Clone + 'static>(&self, anchor: &Anchor<O>) -> O {
        trace!("engine_get: {}", &std::any::type_name::<O>());
        let _g = trace_span!("-> enging_get", "type: {}", &std::any::type_name::<O>()).entered();

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
                    "-> enging_get:engine borrow_muted , now getting.. ",
                    "type: {}",
                    &std::any::type_name::<O>()
                )
                .entered();

                e.get(anchor)
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
    fn get_mut_secondarymap<T: 'static>(&mut self) -> Option<&mut VarSecMap<T>> {
        self.anymap.get_mut::<VarSecMap<T>>()
    }

    fn register_secondarymap<T: 'static>(&mut self) {
        let sm: VarSecMap<T> = VarSecMap::<T>::default();
        self.anymap.insert(sm);
    }

    fn get_state_use_key_and_start_set_run_cb<T: std::clone::Clone + 'static>(
        &self,
        data: T,
        current_id: &StorageKey,
    ) {
        //unwrap or default to keep borrow checker happy
        let (var, before_fns, after_fns) = self
            .get_state_and_bf_af_use_id::<T>(current_id)
            .expect("set_state_with_key: can't set state that doesn't exist in this context!");

        start_set_var_and_before_after(self, current_id, var, data, before_fns, after_fns);
    }
    fn set_in_callback<T: Clone + 'static + std::fmt::Debug>(
        &self,
        skip: &SkipKeyCollection,
        data: &T,
        current_id: &StorageKey,
    ) {
        if skip.borrow().contains(current_id) {
            // println!(
            //     "===skip contains current_id at set_in_similar_fn start -> data:{:?}",
            //     data
            // );
            return;
        }
        skip.borrow_mut().push(*current_id);

        //unwrap or default to keep borrow checker happy
        let (var, before_fns, after_fns) = self
            .get_state_and_bf_af_use_id::<T>(current_id)
            .expect("set_state_with_key: can't set state that doesn't exist in this context!");

        //
        if !before_fns.is_empty() {
            before_fns
                .iter()
                //TODO check performance
                // .map(|(_, func)| func)
                .filter_map(|(key, func)| {
                    if skip.borrow().contains(key) {
                        // println!("similar_fns has key in skip, it's loop !!",);
                        None
                    } else {
                        Some(func)
                    }
                })
                .for_each(|func| {
                    // let skip_clone = skip2.clone();
                    func(self, skip, data);
                });
        }
        debug!("in callbacks ,bf_fns called, then --> var set :{:?}", data);
        // ─────────────────────────────────────────────────────────────────

        var.set(data.clone());
        // ─────────────────────────────────────────────────────────────────

        if !after_fns.is_empty() {
            after_fns
                .iter()
                .filter_map(|(key, func)| {
                    if skip.borrow().contains(key) {
                        None
                    } else {
                        Some(func)
                    }
                })
                .for_each(|func| {
                    func(self, skip, data);
                });
        }
    }
    fn insert_var_with_key<T: 'static>(&mut self, var: Var<T>, current_id: &StorageKey) {
        //unwrap or default to keep borrow checker happy
        let key = self
            .id_to_key_map
            .get(current_id)
            .copied()
            .unwrap_or_default();

        if key.is_null() {
            let key = self.primary_slotmap.insert(*current_id);
            self.id_to_key_map.insert(*current_id, key);
            if let Some(sec_map) = self.get_mut_secondarymap::<T>() {
                //TODO  use (var,secondarymap) replace (var, HashMap::default())
                sec_map.insert(
                    key,
                    (
                        var,
                        SynCallFnsMap::<T>::default(),
                        SynCallFnsMap::<T>::default(),
                    ),
                );
            } else {
                self.register_secondarymap::<T>();
                self.get_mut_secondarymap::<T>().unwrap().insert(
                    key,
                    (
                        var,
                        SynCallFnsMap::<T>::default(),
                        SynCallFnsMap::<T>::default(),
                    ),
                );
            }
        } else if let Some(existing_secondary_map) = self.get_mut_secondarymap::<T>() {
            existing_secondary_map.insert(
                key,
                (
                    var,
                    SynCallFnsMap::<T>::default(),
                    SynCallFnsMap::<T>::default(),
                ),
            );
        } else {
            // key ! null  && T not find
            // self.register_secondarymap::<T>();
            // self.get_mut_secondarymap::<T>().unwrap().insert(key, var);
            panic!("panic current using find why here 2");
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
    pub fn insert_before_fn<T: 'static>(
        &mut self,
        current_id: &StorageKey,
        before_fn_id: &StorageKey,
        func: BoxSynCallFn<T>, //TODO add store
    ) {
        assert_ne!(current_id, before_fn_id);

        let key = self.id_to_key_map.get(current_id).copied();
        let secondarymap = self.get_mut_secondarymap::<T>();
        let (_, fns, _) = match (key, secondarymap) {
            (Some(existing_key), Some(existing_secondary_map)) => existing_secondary_map
                .get_mut(existing_key)
                .expect("cannot get second map"),
            (key, map) => panic!(
                "something(key or map) is None: {} {}",
                key.is_none(),
                map.is_none()
            ),
        };
        //TODO is need check both after_fns_map before_fns_map ??
        if fns.contains_key(before_fn_id) {
            panic!("before_fns already has this key");
        }

        fns.insert(*before_fn_id, func);
    }

    /// # Panics
    ///
    /// Will panic if fns already has `func`
    pub fn insert_after_fn<T: 'static>(
        &mut self,
        current_id: &StorageKey,
        after_fn_id: &StorageKey,
        func: BoxSynCallFn<T>, //TODO add store
    ) {
        assert_ne!(current_id, after_fn_id);

        let key = self.id_to_key_map.get(current_id).copied();
        let secondarymap = self.get_mut_secondarymap::<T>();
        let (_, _, fns) = match (key, secondarymap) {
            (Some(existing_key), Some(existing_secondary_map)) => existing_secondary_map
                .get_mut(existing_key)
                .expect("cannot get second map"),
            (key, map) => panic!(
                "something(key or map) is None: {} {}",
                key.is_none(),
                map.is_none()
            ),
        };
        //TODO is need check both after_fns_map before_fns_map ??
        if fns.contains_key(after_fn_id) {
            panic!("after_fns already has this key");
        }

        fns.insert(*after_fn_id, func);
    }

    fn get_state_and_bf_af_use_id<T: 'static>(
        &self,
        current_id: &StorageKey,
    ) -> Option<&VarSimMap<T>> {
        match (
            self.id_to_key_map.get(current_id),
            self.get_secondarymap::<T>(),
        ) {
            (Some(existing_key), Some(existing_secondary_map)) => {
                existing_secondary_map.get(*existing_key)
            }
            (_, _) => None,
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
}

type SynCallFnsMap<T> = HashMap<StorageKey, BoxSynCallFn<T>>;

fn before_fns_run<T: 'static>(
    store: &GStateStore,
    current_id: &StorageKey,
    data: &T,
    fns: &SynCallFnsMap<T>,
) -> RefCell<Vec<StorageKey>> {
    // let mut new_set = HashSet::default();
    // new_set.insert(*current_id);
    let key_collection = vec![*current_id];
    let skip = RefCell::new(key_collection);
    fns.values().for_each(|bf_func| bf_func(store, &skip, data));
    skip
}
fn after_fns_run<T: 'static>(
    store: &GStateStore,
    skip: &RefCell<Vec<StorageKey>>,
    data: &T,
    fns: &SynCallFnsMap<T>,
) {
    // let mut new_set = HashSet::default();
    // new_set.insert(*current_id);
    fns.values().for_each(|af_func| af_func(store, skip, data));
}

fn start_set_var_and_before_after<T: Clone + 'static>(
    store: &GStateStore,
    current_id: &StorageKey,
    var: &anchors::expert::Var<T, Engine>,
    data: T,
    before_fns: &SynCallFnsMap<T>,
    after_fns: &SynCallFnsMap<T>,
) {
    //NOTE staring first callback call
    let skip = before_fns_run(store, current_id, &data, before_fns);
    var.set(data.clone());
    after_fns_run(store, &skip, &data, after_fns);
}
// ────────────────────────────────────────────────────────────────────────────────

#[derive(PartialEq, Eq)]
pub struct StateVar<T> {
    id: TopoKey,
    _phantom_data: PhantomData<T>,
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
    #[must_use]
    pub fn get(&self) -> T {
        get_anchor_val_in_var_with_topo_id::<T>(self.id)
    }
    pub fn store_get(&self, store: &GStateStore) -> T {
        let var_with_sa = &store
            .get_state_and_bf_af_use_id::<StateAnchor<T>>(&StorageKey::TopoKey(self.id))
            .expect("You are trying to get a var state that doesn't exist in this context!")
            .0;
        store.engine_get((*var_with_sa.get()).clone().anchor())
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
    #[must_use]
    const fn new(id: TopoKey) -> Self {
        Self {
            id,
            _phantom_data: PhantomData,
        }
    }

    #[must_use]
    pub fn state_exists(&self) -> bool {
        state_exists_for_topo_id::<T>(self.id)
    }

    #[must_use]
    pub fn get_with<F: Fn(&T) -> R, R>(&self, func: F) -> R {
        read_state_val_with_topo_id(self.id, |t, _, _| func(t))
    }

    #[must_use]
    pub fn store_get_rc(&self, store: &GStateStore) -> Rc<T> {
        // store
        //     .get_state_with_id::<T>(&StorageKey::TopoKey(self.id))
        //     .expect("You are trying to get a var state that doesn't exist in this context!")
        //     .get()
        self.store_get_var_with(store, anchors::expert::Var::get)
    }

    pub fn get_var_with<F: Fn(&Var<T>) -> R, R>(&self, func: F) -> R {
        read_var_with_topo_id::<_, T, R>(self.id, |_, (v, _, _): &VarSimMap<T>| -> R { func(v) })
    }
    pub fn store_get_var_with<F: Fn(&Var<T>) -> R, R>(&self, store: &GStateStore, func: F) -> R {
        let var = &store
            .get_state_and_bf_af_use_id::<T>(&StorageKey::TopoKey(self.id))
            .expect("You are trying to get a var state that doesn't exist in this context!")
            .0;

        func(var)
    }

    #[must_use]
    pub fn watch(&self) -> StateAnchor<T> {
        self.get_var_with(|v| StateAnchor(v.watch()))
    }
    #[must_use]
    pub fn store_watch(&self, store: &GStateStore) -> StateAnchor<T> {
        // self.get_var_with(|v| StateAnchor(v.watch()))
        self.store_get_var_with(store, |v| StateAnchor(v.watch()))
    }
    pub fn set_in_callback(&self, store: &GStateStore, skip: &SkipKeyCollection, value: &T)
    where
        T: Clone + std::fmt::Debug,
    {
        set_in_callback(store, skip, value, self.id);
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

pub type SkipKeyCollection = RefCell<Vec<StorageKey>>;

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
    fn set_with_once<F: FnOnce(&T) -> T>(&self, func_once: F);
    fn store_set_with<F: Fn(&T) -> T>(&self, store: &GStateStore, func: F);
    fn store_set_with_once<F: FnOnce(&T) -> T>(&self, store: &GStateStore, func_once: F);
    fn set_with<F: Fn(&T) -> T>(&self, func: F);
    fn try_get(&self) -> Option<T>;

    fn update<F: FnOnce(&mut T)>(&self, func: F);
    fn store_update<F: FnOnce(&mut T)>(&self, store: &GStateStore, func: F);

    fn insert_before_fn(
        &self,
        before_fn_key: TopoKey,
        func: impl Fn(&GStateStore, &SkipKeyCollection, &T) + 'static,
        init: bool,
    );

    fn insert_after_fn(
        &self,
        callback_key: TopoKey,
        func: impl Fn(&GStateStore, &SkipKeyCollection, &T) + 'static,
        init: bool,
    );
    fn build_similar_use_into_in_topo<B: Clone + From<T> + 'static + std::fmt::Debug>(
        &self,
    ) -> StateVar<B>;
    fn build_bi_similar_use_into_in_topo<B: Clone + From<T> + Into<T> + 'static + std::fmt::Debug>(
        &self,
    ) -> StateVar<B>
    where
        T: std::fmt::Debug;

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
        // println!("set: {:?}", &value);
        set_state_and_run_cb_with_topo_id::<T>(value, self.id);
    }

    //TODO use illicit @set replace set_in_callback

    fn store_set(&self, store: &GStateStore, value: T) {
        store.get_state_use_key_and_start_set_run_cb::<T>(value, &StorageKey::TopoKey(self.id));
    }
    fn store_set_with<F: Fn(&T) -> T>(&self, store: &GStateStore, func: F) {
        let (var, before_fns, after_fns) = &store
            .get_state_and_bf_af_use_id::<T>(&StorageKey::TopoKey(self.id))
            .expect("You are trying to get a var state that doesn't exist in this context!");

        let data = func(var.get().as_ref());

        start_set_var_and_before_after(
            store,
            &StorageKey::TopoKey(self.id),
            var,
            data,
            before_fns,
            after_fns,
        );
    }

    fn set_with<F: Fn(&T) -> T>(&self, func: F) {
        read_var_with_topo_id::<_, T, ()>(
            self.id,
            |store, (var, before_fns, after_fns): &VarSimMap<T>| {
                let data = func(var.get().as_ref());
                start_set_var_and_before_after(
                    store,
                    &StorageKey::TopoKey(self.id),
                    var,
                    data,
                    before_fns,
                    after_fns,
                );
            },
        );
    }
    fn set_with_once<F: FnOnce(&T) -> T>(&self, func_once: F) {
        read_var_with_topo_id::<_, T, ()>(
            self.id,
            |store, (var, before_fns, after_fns): &VarSimMap<T>| {
                let data = func_once(var.get().as_ref());

                start_set_var_and_before_after(
                    store,
                    &StorageKey::TopoKey(self.id),
                    var,
                    data,
                    before_fns,
                    after_fns,
                );
            },
        );
    }

    fn store_set_with_once<F: FnOnce(&T) -> T>(&self, store: &GStateStore, func_once: F) {
        let (var,before_fns,after_fns) = store
            .get_state_and_bf_af_use_id::<T>(&StorageKey::TopoKey(self.id))
            .expect(
            "fn store_set_with: You are trying to get a var state that doesn't exist in this context!",
        );
        let data = func_once(var.get().as_ref());

        start_set_var_and_before_after(
            store,
            &StorageKey::TopoKey(self.id),
            var,
            data,
            before_fns,
            after_fns,
        );
    }
    fn try_get(&self) -> Option<T> {
        clone_state_with_topo_id::<T>(self.id).map(|v| (*v.get()).clone())
    }

    fn update<F: FnOnce(&mut T)>(&self, func: F) {
        // read_var_with_topo_id::<_, T, ()>(self.id, |var| {
        //     let mut old = (*var.get()).clone();
        //     func(&mut old);
        //     var.set(old);
        // })

        //NOTE 'set_with_once' has callback update inside
        self.set_with_once(|v| {
            let mut old = v.clone();
            func(&mut old);
            old
        });
    }
    fn store_update<F: FnOnce(&mut T)>(&self, store: &GStateStore, func: F) {
        // read_var_with_topo_id::<_, T, ()>(self.id, |var| {
        //     let mut old = (*var.get()).clone();
        //     func(&mut old);
        //     var.set(old);
        // })
        //NOTE 'store_set_with_once' has callback update inside
        self.store_set_with_once(store, |v| {
            let mut old = v.clone();
            func(&mut old);
            old
        });
    }

    // #[topo::nested]
    // fn to_di_in_topo<B>(&self) -> StateVarDi<T, B>
    // where
    //     B: From<T> + Clone + 'static,
    // {
    //     let b = use_state(self.get().into());
    //     StateVarDi::new_use_into(*self, b)
    // }
    // #[topo::nested]
    // fn to_bi_in_topo<B>(&self) -> (StateVarDi<T, B>, StateVarDi<B, T>)
    // where
    //     B: From<T> + Clone + 'static,
    //     T: From<B> + 'static,
    // {
    //     let b = use_state(self.get().into());
    //     (
    //         StateVarDi::new_use_into(*self, b),
    //         StateVarDi::new_use_into(b, *self),
    //     )
    // }

    //TODO 回环检测 , 当两个或者两个以上 有 di关系的 StateVar  set的时候 会再次互相调用set -回环
    fn insert_before_fn(
        &self,
        callback_key: TopoKey,
        func: impl Fn(&GStateStore, &SkipKeyCollection, &T) + 'static,
        init: bool,
    ) {
        insert_before_fn(
            self,
            &StorageKey::TopoKey(callback_key),
            Box::new(func),
            init,
        );
    }
    fn insert_after_fn(
        &self,
        callback_key: TopoKey,
        func: impl Fn(&GStateStore, &SkipKeyCollection, &T) + 'static,
        init: bool,
    ) {
        insert_after_fn(
            self,
            &StorageKey::TopoKey(callback_key),
            Box::new(func),
            init,
        );
    }
    #[topo::nested]
    fn build_similar_use_into_in_topo<B: Clone + From<T> + 'static + std::fmt::Debug>(
        &self,
    ) -> StateVar<B> {
        let v = self.get();
        let b: StateVar<B> = use_state(v.into());
        insert_before_fn(
            self,
            &StorageKey::TopoKey(b.id),
            Box::new(move |store, skip, value| {
                b.set_in_callback(store, skip, &(*value).clone().into());
            }),
            false,
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
        let b: StateVar<B> = use_state(v.into());
        insert_before_fn(
            self,
            &StorageKey::TopoKey(b.id),
            Box::new(move |store, skip, value| {
                b.set_in_callback(store, skip, &(*value).clone().into());
            }),
            false,
        );
        let this = *self;
        insert_before_fn(
            &b,
            &StorageKey::TopoKey(self.id),
            Box::new(move |store, skip, value| {
                this.set_in_callback(store, skip, &(*value).clone().into());
            }),
            false,
        );
        b
    }
}

// #[macro_export]
// macro_rules! to_vector_di {
//     ( $( $element:expr ) , * ) => {
//         {
//             let mut v:im::Vector<emg_state::StateVar<emg_animation::Property>> = im::Vector::new();

//             $(
//                 let di =emg_state::StateVarDi::From( $element);
//                 v.push_back(di.into());
//             )*

//             v
//         }
//     };
// }
// pub struct StateVarDi<A, B> {
//     pub this: StateVar<A>,
//     pub similar: StateVar<B>,
//     update_fn: Box<dyn Fn(A, StateVar<B>)>,
// }
// impl<A, B, T> TryFrom<StateVarDi<A, B>> for StateVar<T>
// where
//     A: 'static,
//     B: 'static,
//     T: 'static,
// {
//     type Error = ();
//     fn try_from(di: StateVarDi<A, B>) -> Result<Self, Self::Error> {
//         if TypeId::of::<A>() == TypeId::of::<T>() {
//             let any: Box<dyn std::any::Any> = Box::new(di.this);
//             any.downcast::<Self>().map(|v| *v).map_err(|_| ())
//         } else if TypeId::of::<B>() == TypeId::of::<T>() {
//             let any: Box<dyn std::any::Any> = Box::new(di.similar);
//             any.downcast::<Self>().map(|v| *v).map_err(|_| ())
//         } else {
//             panic!("not match any type")
//         }
//     }
// }

// impl<A, B> StateVarDi<A, B>
// where
//     B: Clone + From<A> + 'static,
//     A: Clone + 'static,
// {
//     #[must_use]
//     pub fn new_use_into(this: StateVar<A>, similar: StateVar<B>) -> Self {
//         Self {
//             this,
//             similar,
//             update_fn: Box::new(|a, sv_b| sv_b.set(a.into())),
//         }
//     }
// }
// impl<A, B> StateVarDi<A, B>
// where
//     B: 'static,
//     A: Clone + 'static,
// {
//     #[must_use]
//     pub fn new(
//         this: StateVar<A>,
//         similar: StateVar<B>,
//         update_fn: Box<dyn Fn(A, StateVar<B>)>,
//     ) -> Self {
//         Self {
//             this,
//             similar,
//             update_fn,
//         }
//     }

//     pub fn set(&self, value: A) {
//         self.this.set(value.clone());
//         (self.update_fn)(value, self.similar);
//     }
//     #[must_use]
//     pub fn get(&self) -> A {
//         self.this.get()
//     }
// }

#[derive(Clone, PartialEq, Eq)]
pub struct StateAnchor<T>(Anchor<T>);

impl<T: 'static + std::fmt::Display + Clone> std::fmt::Display for StateAnchor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.get();
        write!(f, "\u{2693} ({})", &v)
    }
}

impl<T: 'static + std::fmt::Debug + Clone> std::fmt::Debug for StateAnchor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.get();
        f.debug_tuple("StateAnchor").field(&v).finish()
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
    fn from(v: T) -> Self {
        Self::constant(v)
    }
}

pub trait CloneStateAnchor<T>
where
    T: Clone + 'static,
{
    fn get(&self) -> T;
    fn store_get(&self, store: &GStateStore) -> T;
}
impl<T> CloneStateAnchor<T> for StateAnchor<T>
where
    T: Clone + 'static,
{
    fn get(&self) -> T {
        global_engine_get_anchor_val(&self.0)
    }
    fn store_get(&self, store: &GStateStore) -> T {
        store.engine_get(&self.0)
    }
}

pub use anchors::collections::ord_map::Dict;

impl<K: Ord + Clone + PartialEq + 'static, V: Clone + PartialEq + 'static> StateAnchor<Dict<K, V>> {
    #[track_caller]
    pub fn filter<F: FnMut(&K, &V) -> bool + 'static>(&self, mut f: F) -> Self {
        self.0
            .filter_map(move |k, v| if f(k, v) { Some(v.clone()) } else { None })
            .into()
    }

    #[track_caller]
    pub fn map_<F: FnMut(&K, &V) -> T + 'static, T: Clone + PartialEq + 'static>(
        &self,
        mut f: F,
    ) -> StateAnchor<Dict<K, T>> {
        self.0.filter_map(move |k, v| Some(f(k, v))).into()
    }

    /// FOOBAR
    #[track_caller]
    pub fn filter_map<F: FnMut(&K, &V) -> Option<T> + 'static, T: Clone + PartialEq + 'static>(
        &self,
        f: F,
    ) -> StateAnchor<Dict<K, T>> {
        self.0.filter_map(f).into()
    }
}

//TODO remove static
impl<T> StateAnchor<T>
where
    T: 'static,
{
    pub fn constant(val: T) -> Self {
        G_STATE_STORE.with(|_g_state_store_refcell| Self(Anchor::constant(val)))
    }
    #[must_use]
    pub const fn anchor(&self) -> &Anchor<T> {
        &self.0
    }
    #[must_use]
    pub fn get_anchor(&self) -> Anchor<T> {
        self.0.clone()
    }
    // ────────────────────────────────────────────────────────────────────────────────

    #[track_caller]
    pub fn map<Out, F>(&self, f: F) -> StateAnchor<Out>
    where
        Out: 'static,
        F: 'static,
        map::Map<(Anchor<T>,), F, Out>: AnchorInner<Engine, Output = Out>,
    {
        self.0.map(f).into()
    }
    #[track_caller]
    pub fn map_mut<Out, F>(&self, initial: Out, f: F) -> StateAnchor<Out>
    where
        Out: 'static,
        F: 'static,
        map_mut::MapMut<(Anchor<T>,), F, Out>: AnchorInner<Engine, Output = Out>,
    {
        self.0.map_mut(initial, f).into()
    }

    #[track_caller]
    pub fn then<Out, F>(&self, f: F) -> StateAnchor<Out>
    where
        F: 'static,
        Out: 'static,
        then::Then<(Anchor<T>,), Out, F, Engine>: AnchorInner<Engine, Output = Out>,
    {
        self.0.then(f).into()
    }

    #[track_caller]
    pub fn refmap<F, Out>(&self, f: F) -> StateAnchor<Out>
    where
        Out: 'static,
        F: 'static,
        refmap::RefMap<(Anchor<T>,), F>: AnchorInner<Engine, Output = Out>,
    {
        self.0.refmap(f).into()
    }
    #[track_caller]
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

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum StorageKey {
    // SlottedKey(SlottedKey),
    TopoKey(TopoKey),
}

impl From<TopoKey> for StorageKey {
    fn from(v: TopoKey) -> Self {
        Self::TopoKey(v)
    }
}

fn global_engine_get_anchor_val<O: Clone + 'static>(anchor: &Anchor<O>) -> O {
    G_STATE_STORE.with(|g_state_store_refcell| g_state_store_refcell.borrow().engine_get(anchor))
}

///
///  Uses the current topological id to create a new state accessor
///
fn state_exists_for_topo_id<T: 'static>(id: TopoKey) -> bool {
    G_STATE_STORE.with(|g_state_store_refcell| {
        g_state_store_refcell
            .borrow()
            .state_exists_with_id::<T>(StorageKey::TopoKey(id))
    })
}
/// Sets the state of type T keyed to the given `TopoId`
fn set_state_and_run_cb_with_topo_id<T: 'static + std::clone::Clone>(data: T, current_id: TopoKey) {
    G_STATE_STORE.with(|g_state_store_refcell| {
        g_state_store_refcell
            .borrow()
            .get_state_use_key_and_start_set_run_cb::<T>(data, &StorageKey::TopoKey(current_id));
    });

    // execute_reaction_nodes(&StorageKey::TopoKey(current_id));
}
fn set_in_callback<T: 'static + std::fmt::Debug + std::clone::Clone>(
    store: &GStateStore,
    skip: &SkipKeyCollection,
    data: &T,
    current_id: TopoKey,
) {
    // G_STATE_STORE.with(|g_state_store_refcell| {
    store.set_in_callback::<T>(skip, data, &StorageKey::TopoKey(current_id));
    // });

    // execute_reaction_nodes(&StorageKey::TopoKey(current_id));
}
fn insert_var_with_topo_id<T: 'static>(var: Var<T>, current_id: TopoKey) {
    G_STATE_STORE.with(|g_state_store_refcell| {
        g_state_store_refcell
            .borrow_mut()
            .insert_var_with_key::<T>(var, &StorageKey::TopoKey(current_id));
    });

    // execute_reaction_nodes(&StorageKey::TopoKey(current_id));
}

fn clone_state_with_topo_id<T: 'static>(id: TopoKey) -> Option<Var<T>> {
    G_STATE_STORE.with(|g_state_store_refcell| {
        g_state_store_refcell
            .borrow_mut()
            .get_state_and_bf_af_use_id::<T>(&StorageKey::TopoKey(id))
            .map(|v| &v.0)
            .cloned()
    })
}

fn read_state_val_with_topo_id<
    F: FnOnce(&T, &SynCallFnsMap<T>, &SynCallFnsMap<T>) -> R,
    T: 'static,
    R,
>(
    id: TopoKey,
    func: F,
) -> R {
    // G_STATE_STORE.with(|g_state_store_refcell| {
    //     func(
    //         g_state_store_refcell
    //             .borrow_mut()
    //             .get_state_with_id::<T>(&StorageKey::TopoKey(id))
    //             .expect("You are trying to get a var state that doesn't exist in this context!")
    //             .get()
    //             .as_ref(),
    //     )
    // })

    read_var_with_topo_id::<_, T, R>(
        id,
        |_: &GStateStore, (var, before_fns, after_fns): &VarSimMap<T>| {
            func(var.get().as_ref(), before_fns, after_fns)
        },
    )
}
// fn read_state_val_with_topo_id_old<F: FnOnce(&T) -> R, T: 'static, R>(id: TopoKey, func: F) -> R {
//     let item = remove_state_with_topo_id::<T>(id)
//         .expect("You are trying to read a type state that doesn't exist in this context!");
//     let read = func(&*(item.get()));

//     insert_var_with_topo_id(item, id);
//     read
// }

fn get_anchor_val_in_var_with_topo_id<T: 'static + std::clone::Clone>(id: TopoKey) -> T {
    G_STATE_STORE.with(|g_state_store_refcell| {
        let store = g_state_store_refcell.borrow();
        let var_with_sa = &store
            .get_state_and_bf_af_use_id::<StateAnchor<T>>(&StorageKey::TopoKey(id))
            .expect("You are trying to get a var state that doesn't exist in this context!")
            .0;
        store.engine_get((*var_with_sa.get()).clone().anchor())
    })
}

fn read_var_with_topo_id<F: FnOnce(&GStateStore, &VarSimMap<T>) -> R, T: 'static, R>(
    id: TopoKey,
    func: F,
) -> R {
    G_STATE_STORE.with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
        let store = g_state_store_refcell.borrow();
        func(
            &store,
            store
                .get_state_and_bf_af_use_id::<T>(&StorageKey::TopoKey(id))
                .expect("You are trying to get a var state that doesn't exist in this context!"),
        )
    })
}

type BoxSynCallFn<T> = Box<dyn Fn(&GStateStore, &SkipKeyCollection, &T)>;

fn insert_before_fn<T: 'static + std::clone::Clone>(
    sv: &StateVar<T>,
    before_id: &StorageKey,
    func: BoxSynCallFn<T>,
    init: bool,
) {
    G_STATE_STORE.with(|g_state_store_refcell| {
        if init {
            let store = &g_state_store_refcell.borrow();
            let (var, _before_fns, _) = store
                .get_state_and_bf_af_use_id::<T>(&StorageKey::TopoKey(sv.id))
                .expect("set_state_with_key: can't set state that doesn't exist in this context!");

            let key_collection = vec![StorageKey::TopoKey(sv.id)];
            let skip = RefCell::new(key_collection);

            func(store, &skip, &*var.get());
            // let v = &(*var.get());
            // before_fns_run(store, &StorageKey::TopoKey(sv.id), v, before_fns);
            // if v.clone() != (*var.get()).clone() {
            //     panic!("not same");
            // }
        }

        g_state_store_refcell.borrow_mut().insert_before_fn(
            &StorageKey::TopoKey(sv.id),
            before_id,
            func,
        );
    });
}
fn insert_after_fn<T: 'static + std::clone::Clone>(
    sv: &StateVar<T>,
    after_id: &StorageKey,
    func: BoxSynCallFn<T>,
    init: bool,
) {
    G_STATE_STORE.with(|g_state_store_refcell| {
        if init {
            let store = &g_state_store_refcell.borrow();
            let (var, _, _) = store
                .get_state_and_bf_af_use_id::<T>(&StorageKey::TopoKey(sv.id))
                .expect("set_state_with_key: can't set state that doesn't exist in this context!");

            let key_collection = vec![StorageKey::TopoKey(sv.id)];
            let skip = RefCell::new(key_collection);

            func(store, &skip, &*var.get());

            // after_fns_run(store, &skip, &(*var.get()), after_fns);
        }

        g_state_store_refcell.borrow_mut().insert_after_fn(
            &StorageKey::TopoKey(sv.id),
            after_id,
            func,
        );
    });
}

pub fn state_store_with<F, R>(func: F) -> R
where
    F: FnOnce(&GStateStore) -> R,
{
    G_STATE_STORE.with(|g_state_store_refcell| func(&*g_state_store_refcell.borrow()))
}
#[must_use]
pub fn state_store() -> Rc<RefCell<GStateStore>> {
    G_STATE_STORE.with(std::clone::Clone::clone)
}

// fn read_var_with_topo_id_old<F: FnOnce(&Var<T>) -> R, T: 'static, R>(id: TopoKey, func: F) -> R {
//     let var = remove_state_with_topo_id::<T>(id)
//         .expect("You are trying to read a type state that doesn't exist in this context!");
//     let read = func(&var);
//     insert_var_with_topo_id(var, id);
//     read
// }
// fn remove_state_with_topo_id<T: 'static>(id: TopoKey) -> Option<Var<T>> {
//     G_STATE_STORE.with(|g_state_store_refcell| {
//         g_state_store_refcell
//             .borrow_mut()
//             .remove_state_with_id::<T>(&StorageKey::TopoKey(id))
//     })
// }

#[must_use]
#[topo::nested]
#[allow(clippy::if_not_else)]
pub fn use_state<T>(data: T) -> StateVar<T>
where
    T: 'static,
{
    let id = topo::CallId::current();
    let id = TopoKey { id };

    if !state_exists_for_topo_id::<T>(id) {
        insert_var_with_topo_id::<T>(Var::new(data), id);
    } else {
        panic!("this is checker:  already settled state");
    }
    StateVar::new(id)
}

// pub fn add_similar<T>(func:F)
// use overloadf::overload;
// #[overload]
// pub fn xx<T>(d: Var<T>) {}
// #[overload]
// pub fn xx<T>(dd: Anchor<T>) {}
#[cfg(test)]
#[allow(unused_variables)]
mod state_test {
    use tracing::debug;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[derive(Clone, Debug)]
    struct TT(String);
    impl From<i32> for TT {
        fn from(v: i32) -> Self {
            Self(format!("{:?}", v))
        }
    }
    impl From<TT> for i32 {
        fn from(v: TT) -> Self {
            let s = v.0;
            let i = s.parse::<i32>().unwrap();
            i
        }
    }

    use tracing::Level;

    use tracing_flame::FlameLayer;
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    fn _init() {
        // let _el = env_logger::try_init();

        let _subscriber = tracing_subscriber::fmt()
            .with_test_writer()
            // .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_span_events(
                tracing_subscriber::fmt::format::FmtSpan::ACTIVE
                    | tracing_subscriber::fmt::format::FmtSpan::ENTER
                    | tracing_subscriber::fmt::format::FmtSpan::CLOSE,
            )
            .with_max_level(Level::DEBUG)
            .try_init();

        // tracing::subscriber::set_global_default(subscriber)
        // .expect("setting default subscriber failed");
    }

    #[test]
    // #[wasm_bindgen_test]
    fn callback() {
        let _g = _init();
        let a = use_state(1);
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
    // #[wasm_bindgen_test]
    fn callback2() {
        let a = use_state(1);
        let b = a.build_bi_similar_use_into_in_topo::<TT>();
        let c = b.build_similar_use_into_in_topo::<i32>();
        let d = b.build_similar_use_into_in_topo::<i32>();
        d.insert_before_fn(
            c.id.into(),
            move |store, skip, value| {
                c.set_in_callback(store, skip, &(*value).into());
            },
            true,
        );
        let update_id = TopoKey::new(topo::call(topo::CallId::current));

        c.insert_before_fn(
            a.id.into(),
            move |store, skip, value| {
                println!("c -> before_fns 1 -> set a:{:?}", &value);

                a.set_in_callback(store, skip, &value);
            },
            true,
        );
        let update_id2 = TopoKey::new(topo::call(topo::CallId::current));

        //NOTE same a set_in_callback will ignored at second times
        c.insert_before_fn(
            update_id2.into(),
            move |store, skip, value| {
                println!("c -> before_fns 2 -> set a:{:?}", value + 1);
                a.set_in_callback(store, skip, &(value + 1).into());
            },
            true,
        );
        let e = use_state(11);
        c.insert_after_fn(
            e.id.into(),
            move |store, skip, v| {
                e.set_in_callback(store, skip, v);
            },
            true,
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
    }
    #[test]
    // #[wasm_bindgen_test]
    fn update() {
        let a = use_state(111);
        a.update(|aa| *aa += 1);
        println!("{}", &a);
        assert_eq!(112, a.get());
        a.update(|aa| *aa -= 2);
        println!("{}", &a);
        assert_eq!(110, a.get());
    }

    // #[wasm_bindgen_test]
    #[test]
    #[wasm_bindgen_test]

    fn sa_in_sv() {
        let x = use_state(1);
        let xw = x.watch();
        let a = use_state(xw);
        println!("{}", a);
        println!("{}", a.get());
        assert_eq!(1, a.get());
    }
    #[allow(clippy::similar_names)]
    #[test]
    #[wasm_bindgen_test]
    fn xx() {
        let a = use_state(99);

        let b = a.watch();
        let b2 = a.watch();
        let cadd = b.map(|x| *x + 1);
        let cadd2 = b.map(|x| *x + 2);
        let cadd_c = cadd.clone();
        let cadd2_c = cadd2;
        let c = b.map(|x| format!("{}", x));
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
        let dc = ddw.map(|x| format!("{}", x));

        let ddw3 = ddw.then(move |x| if *x > 1 { ddw2.clone() } else { dcadd.clone() });
    }
}
