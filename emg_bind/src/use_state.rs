/*
 * @Author: Rais
 * @Date: 2021-03-15 17:10:47
 * @LastEditTime: 2021-03-17 13:17:56
 * @LastEditors: Rais
 * @Description:
 */

use std::{cell::RefCell, clone::Clone, collections::HashMap, marker::PhantomData};

use anchors::singlethread::{Anchor, Engine, Var};
use anymap::any::Any;
// use delegate::delegate;
use slotmap::{DefaultKey, DenseSlotMap, Key, SecondaryMap};

thread_local! {
    static G_STATE_STORE: RefCell<GStateStore> = RefCell::new(
        GStateStore::default()
    );
}
#[allow(clippy::module_name_repetitions)]
struct GStateStore {
    anymap: anymap::Map<dyn Any>,
    id_to_key_map: HashMap<StorageKey, DefaultKey>,
    primary_slotmap: DenseSlotMap<DefaultKey, StorageKey>,
    engine: Engine,
}
impl Default for GStateStore {
    fn default() -> Self {
        Self {
            anymap: anymap::Map::new(),
            id_to_key_map: HashMap::new(),
            primary_slotmap: DenseSlotMap::new(),
            engine: Engine::new(),
        }
    }
}

impl GStateStore {
    fn engine_get<O: Clone + 'static>(&mut self, anchor: &Anchor<O>) -> O {
        self.engine.get(anchor)
    }
    const fn engine(&self) -> &Engine {
        &self.engine
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
    fn get_secondarymap<T: 'static>(&self) -> Option<&SecondaryMap<DefaultKey, Var<T>>> {
        self.anymap.get::<SecondaryMap<DefaultKey, Var<T>>>()
    }
    fn get_mut_secondarymap<T: 'static>(
        &mut self,
    ) -> Option<&mut SecondaryMap<DefaultKey, Var<T>>> {
        self.anymap.get_mut::<SecondaryMap<DefaultKey, Var<T>>>()
    }
    fn register_secondarymap<T: 'static>(&mut self) {
        let sm: SecondaryMap<DefaultKey, Var<T>> = SecondaryMap::new();
        self.anymap.insert(sm);
    }

    fn set_state_with_key<T: 'static>(&mut self, data: T, current_id: &StorageKey) {
        //unwrap or default to keep borrow checker happy
        self.get_state_with_id::<T>(current_id)
            .expect("set_state_with_key: can't set state that doesn't exist in this context!")
            .set(data);
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
                sec_map.insert(key, var);
            } else {
                self.register_secondarymap::<T>();
                self.get_mut_secondarymap::<T>().unwrap().insert(key, var);
            }
        } else if let Some(existing_secondary_map) = self.get_mut_secondarymap::<T>() {
            existing_secondary_map.insert(key, var);
        } else {
            // key ! null  && not find
            // self.register_secondarymap::<T>();
            // self.get_mut_secondarymap::<T>().unwrap().insert(key, var);
            panic!("panic current using find why here 2");
        }
    }

    fn get_state_with_id<T: 'static>(&self, current_id: &StorageKey) -> Option<&Var<T>> {
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

    fn remove_state_with_id<T: 'static>(&mut self, current_id: &StorageKey) -> Option<Var<T>> {
        // /self.unseen_ids.remove(&current_id);
        //unwrap or default to keep borrow checker happy
        let key = self
            .id_to_key_map
            .get(current_id)
            .copied()
            .unwrap_or_default();

        if key.is_null() {
            None
        } else {
            self.get_mut_secondarymap::<T>()
                .and_then(|existing_secondary_map| existing_secondary_map.remove(key))
        }
    }
}
// ────────────────────────────────────────────────────────────────────────────────

pub struct StateVar<T> {
    id: TopoKey,
    _phantom_data: PhantomData<T>,
}
impl<T> std::fmt::Debug for StateVar<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:#?})", self.id)
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
impl<T> StateVar<T>
where
    T: 'static + std::fmt::Debug + Clone,
{
    pub fn debug(&self, s: &str) {
        log::debug!("{:?} StateVar({:?})", s, self.get());
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

    // stores a value of type T in a backing Store
    pub fn set(self, value: T) {
        set_state_with_topo_id::<T>(value, self.id);
    }

    /// `set_with` Fn(&T) -> T
    /// `G_STATE_STORE`  only use once to set, get.
    pub fn set_with<F: Fn(&T) -> T>(self, func: F) {
        read_var_with_topo_id::<_, T, ()>(self.id, |var| var.set(func(var.get().as_ref())))
    }

    #[must_use]
    pub fn state_exists(self) -> bool {
        state_exists_for_topo_id::<T>(self.id)
    }

    #[must_use]
    pub fn get_with<F: Fn(&T) -> R, R>(&self, func: F) -> R {
        read_state_val_with_topo_id(self.id, func)
    }

    pub fn get_var_with<F: Fn(&Var<T>) -> R, R>(&self, func: F) -> R {
        read_var_with_topo_id(self.id, func)
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

pub trait CloneState<T>
where
    T: Clone + 'static,
{
    fn get(&self) -> T;

    fn try_get(&self) -> Option<T>;
    fn watch(&self) -> StateAnchor<T>;
}

impl<T> CloneState<T> for StateVar<T>
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

    fn try_get(&self) -> Option<T> {
        clone_state_with_topo_id::<T>(self.id).map(|v| (*v.get()).clone())
    }
    fn watch(&self) -> StateAnchor<T> {
        self.get_var_with(|v| StateAnchor(v.watch()))
    }
}
#[derive(Clone)]
pub struct StateAnchor<T>(Anchor<T>);

impl<T> StateAnchor<T>
where
    T: 'static + Clone,
{
    pub fn get(&self) -> T {
        global_engine_get_anchor_val(&self.0)
    }

    pub fn anchor(&self) -> &Anchor<T> {
        &self.0
    }
    // ────────────────────────────────────────────────────────────────────────────────

    #[track_caller]
    pub fn map<
        Out: 'static + std::cmp::PartialEq + std::clone::Clone,
        F: 'static + for<'any> FnMut(&'any T) -> Out,
    >(
        &self,
        f: F,
    ) -> StateAnchor<Out> {
        self.0.map(f).into()
    }
    #[track_caller]
    pub fn map_mut<
        Out: 'static + std::cmp::PartialEq + std::clone::Clone,
        F: 'static + for<'any> FnMut(&'any mut Out, &'any T) -> bool,
    >(
        &self,
        initial: Out,
        f: F,
    ) -> StateAnchor<Out> {
        self.0.map_mut(initial, f).into()
    }

    #[track_caller]
    pub fn then<
        Out: 'static + std::cmp::PartialEq + std::clone::Clone,
        F: 'static + for<'any> FnMut(&'any T) -> Anchor<Out>,
    >(
        &self,
        f: F,
    ) -> StateAnchor<Out> {
        self.0.then(f).into()
    }

    // delegate! {
    //     to self.0 {
    //         #[into]
    //         pub fn map<
    //             Out: 'static + std::cmp::PartialEq + std::clone::Clone,
    //             F: 'static + for<'any> std::ops::FnMut<(&'any T,)> + FnOnce(&T) -> Out,
    //          >
    //          ( self, f: F, ) -> StateAnchor<Out>;
    //     }
    // }
}

impl<T> From<Anchor<T>> for StateAnchor<T>
where
    T: 'static + Clone,
{
    fn from(anchor: Anchor<T>) -> Self {
        Self(anchor)
    }
}
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
struct TopoKey {
    // pub ctx: Option<SlottedKey>,
    id: topo::CallId,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
enum StorageKey {
    // SlottedKey(SlottedKey),
    TopoKey(TopoKey),
}

fn global_engine_get_anchor_val<O: Clone + 'static>(anchor: &Anchor<O>) -> O {
    G_STATE_STORE
        .with(|g_state_store_refcell| g_state_store_refcell.borrow_mut().engine_get(anchor))
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
fn set_state_with_topo_id<T: 'static>(data: T, current_id: TopoKey) {
    G_STATE_STORE.with(|g_state_store_refcell| {
        g_state_store_refcell
            .borrow_mut()
            .set_state_with_key::<T>(data, &StorageKey::TopoKey(current_id))
    });

    // execute_reaction_nodes(&StorageKey::TopoKey(current_id));
}
fn insert_var_with_topo_id<T: 'static>(var: Var<T>, current_id: TopoKey) {
    G_STATE_STORE.with(|g_state_store_refcell| {
        g_state_store_refcell
            .borrow_mut()
            .insert_var_with_key::<T>(var, &StorageKey::TopoKey(current_id))
    });

    // execute_reaction_nodes(&StorageKey::TopoKey(current_id));
}

fn clone_state_with_topo_id<T: 'static + Clone>(id: TopoKey) -> Option<Var<T>> {
    G_STATE_STORE.with(|g_state_store_refcell| {
        g_state_store_refcell
            .borrow_mut()
            .get_state_with_id::<T>(&StorageKey::TopoKey(id))
            .cloned()
    })
}
fn read_state_val_with_topo_id<F: FnOnce(&T) -> R, T: 'static, R>(id: TopoKey, func: F) -> R {
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

    read_var_with_topo_id::<_, T, R>(id, |var| func(var.get().as_ref()))
}
// fn read_state_val_with_topo_id_old<F: FnOnce(&T) -> R, T: 'static, R>(id: TopoKey, func: F) -> R {
//     let item = remove_state_with_topo_id::<T>(id)
//         .expect("You are trying to read a type state that doesn't exist in this context!");
//     let read = func(&*(item.get()));

//     insert_var_with_topo_id(item, id);
//     read
// }
fn read_var_with_topo_id<F: FnOnce(&Var<T>) -> R, T: 'static, R>(id: TopoKey, func: F) -> R {
    G_STATE_STORE.with(|g_state_store_refcell| {
        func(
            g_state_store_refcell
                .borrow()
                .get_state_with_id::<T>(&StorageKey::TopoKey(id))
                .expect("You are trying to get a var state that doesn't exist in this context!"),
        )
    })
    // G_STATE_STORE.with(|g_state_store_refcell| {
    //     let var = g_state_store_refcell
    //         .borrow_mut()
    //         .get_state_with_id::<T>(&StorageKey::TopoKey(id))
    //         .expect("You are trying to get a var state that doesn't exist in this context!")
    //         .clone();
    //     func(&var)
    // })
}
// fn read_var_with_topo_id_old<F: FnOnce(&Var<T>) -> R, T: 'static, R>(id: TopoKey, func: F) -> R {
//     let var = remove_state_with_topo_id::<T>(id)
//         .expect("You are trying to read a type state that doesn't exist in this context!");
//     let read = func(&var);
//     insert_var_with_topo_id(var, id);
//     read
// }
fn remove_state_with_topo_id<T: 'static>(id: TopoKey) -> Option<Var<T>> {
    G_STATE_STORE.with(|g_state_store_refcell| {
        g_state_store_refcell
            .borrow_mut()
            .remove_state_with_id::<T>(&StorageKey::TopoKey(id))
    })
}

#[must_use]
#[topo::nested]
pub fn use_state<T: 'static>(data: T) -> StateVar<T> {
    let id = topo::CallId::current();
    let id = TopoKey { id };

    if !state_exists_for_topo_id::<T>(id) {
        insert_var_with_topo_id::<T>(Var::new(data), id);
    }
    StateVar::new(id)
}
// use overloadf::overload;
// #[overload]
// pub fn xx<T>(d: Var<T>) {}
// #[overload]
// pub fn xx<T>(dd: Anchor<T>) {}
#[cfg(test)]
#[allow(unused_variables)]
mod state_test {
    use wasm_bindgen_test::*;

    use super::*;
    #[wasm_bindgen_test]
    fn xx() {
        // let engine = Engine::new();

        let a = use_state(99);
        let b = a.watch();
        let b2 = a.watch();
        let cadd = b.map(|x| *x + 1);
        let cadd2 = b.map(|x| *x + 2);
        let caddc = cadd.clone();
        let cadd2c = cadd2.clone();
        let c = b.map(|x| format!("{}", x));
        let d = b.then(move |x| {
            if *x > 1 {
                b2.anchor().clone()
            } else {
                cadd.anchor().clone()
            }
        });
        log::debug!("========================{:?}", caddc.get());
        log::debug!("========================{:?}", cadd2c.get());

        assert_eq!(caddc.get(), 100);
        assert_eq!(cadd2c.get(), 101);

        let dd = Var::new(99);
        let ddw = dd.watch();
        let ddw2 = dd.watch();
        let dcadd = ddw.map(|x| *x + 1);
        let dc = ddw.map(|x| format!("{}", x));

        let ddw3 = ddw.then(move |x| if *x > 1 { ddw2.clone() } else { dcadd.clone() });
    }
}
