/*
 * @Author: Rais
 * @Date: 2021-03-15 17:10:47
 * @LastEditTime: 2021-06-17 08:56:33
 * @LastEditors: Rais
 * @Description:
 */

use anchors::{
    expert::{cutoff, map, map_mut, refmap, then, AnchorInner},
    singlethread::{Anchor, Engine, MultiAnchor, Var},
};
use anymap::any::Any;

use std::{
    any::TypeId, cell::RefCell, clone::Clone, collections::HashMap, convert::TryFrom,
    marker::PhantomData, rc::Rc,
};
use tracing::{trace, trace_span};
// use delegate::delegate;
use slotmap::{DefaultKey, DenseSlotMap, Key, SecondaryMap};

thread_local! {
    static G_STATE_STORE: Rc<RefCell<GStateStore>> =Rc::new( RefCell::new(
        GStateStore::default()
    ));
}
#[allow(clippy::module_name_repetitions)]
pub struct GStateStore {
    anymap: anymap::Map<dyn Any>,
    id_to_key_map: HashMap<StorageKey, DefaultKey>,
    primary_slotmap: DenseSlotMap<DefaultKey, StorageKey>,
    engine: RefCell<Engine>,
}

impl std::fmt::Debug for GStateStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the graph store")
    }
}
impl Default for GStateStore {
    fn default() -> Self {
        Self {
            anymap: anymap::Map::new(),
            id_to_key_map: HashMap::new(),
            primary_slotmap: DenseSlotMap::new(),
            engine: RefCell::new(Engine::new()),
        }
    }
}

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

    fn set_state_with_key<T: 'static>(&self, data: T, current_id: &StorageKey) {
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
// ────────────────────────────────────────────────────────────────────────────────

#[derive(PartialEq, Eq)]
pub struct StateVar<T> {
    id: TopoKey,
    has_similar: bool,
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
            has_similar: self.has_similar,
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
        let var_with_sa = store
            .get_state_with_id::<StateAnchor<T>>(&StorageKey::TopoKey(self.id))
            .expect("You are trying to get a var state that doesn't exist in this context!");
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
            has_similar: false,
            _phantom_data: PhantomData,
        }
    }

    // stores a value of type T in a backing Store
    pub fn set(&self, value: T)
    // where
    //     T: std::fmt::Debug,
    {
        // println!("set: {:?}", &value);
        set_state_with_topo_id::<T>(value, self.id);
    }
    pub fn store_set(&self, store: &GStateStore, value: T) {
        store.set_state_with_key::<T>(value, &StorageKey::TopoKey(self.id));
    }

    /// `set_with` Fn(&T) -> T
    /// `G_STATE_STORE`  only use once to set, get.
    pub fn set_with<F: Fn(&T) -> T>(&self, func: F) {
        read_var_with_topo_id::<_, T, ()>(self.id, |var| var.set(func(var.get().as_ref())));
    }
    pub fn store_set_with<F: Fn(&T) -> T>(&self, store: &GStateStore, func: F) {
        self.store_get_var_with(store, |var| {
            var.set(func(var.get().as_ref()));
        });
    }

    pub fn set_with_once<F: FnOnce(&T) -> T>(&self, func_once: F) {
        read_var_with_topo_id::<_, T, ()>(self.id, |var| var.set(func_once(var.get().as_ref())));
    }
    pub fn store_set_with_once<F: FnOnce(&T) -> T>(&self, store: &GStateStore, func_once: F) {
        let var = store
            .get_state_with_id::<T>(&StorageKey::TopoKey(self.id))
            .expect(
            "fn store_set_with: You are trying to get a var state that doesn't exist in this context!",
        );

        var.set(func_once(var.get().as_ref()));
    }

    #[must_use]
    pub fn state_exists(&self) -> bool {
        state_exists_for_topo_id::<T>(self.id)
    }

    #[must_use]
    pub fn get_with<F: Fn(&T) -> R, R>(&self, func: F) -> R {
        read_state_val_with_topo_id(self.id, func)
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
        read_var_with_topo_id(self.id, func)
    }
    pub fn store_get_var_with<F: Fn(&Var<T>) -> R, R>(&self, store: &GStateStore, func: F) -> R {
        let var = store
            .get_state_with_id::<T>(&StorageKey::TopoKey(self.id))
            .expect("You are trying to get a var state that doesn't exist in this context!");

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
    fn try_get(&self) -> Option<T>;

    fn update<F: FnOnce(&mut T)>(&self, func: F);
    fn store_update<F: FnOnce(&mut T)>(&self, store: &GStateStore, func: F);

    fn to_di_in_topo<B>(&self) -> StateVarDi<T, B>
    where
        B: From<T> + Clone + 'static;

    fn to_bi_in_topo<B>(&self) -> (StateVarDi<T, B>, StateVarDi<B, T>)
    where
        B: From<T> + Clone + 'static,
        T: From<B> + 'static;
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

    fn try_get(&self) -> Option<T> {
        clone_state_with_topo_id::<T>(self.id).map(|v| (*v.get()).clone())
    }

    fn update<F: FnOnce(&mut T)>(&self, func: F) {
        // read_var_with_topo_id::<_, T, ()>(self.id, |var| {
        //     let mut old = (*var.get()).clone();
        //     func(&mut old);
        //     var.set(old);
        // })
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
        self.store_set_with_once(store, |v| {
            let mut old = v.clone();
            func(&mut old);
            old
        });
    }

    #[topo::nested]
    fn to_di_in_topo<B>(&self) -> StateVarDi<T, B>
    where
        B: From<T> + Clone + 'static,
    {
        let b = use_state(self.get().into());
        StateVarDi::new_use_into(*self, b)
    }
    #[topo::nested]
    fn to_bi_in_topo<B>(&self) -> (StateVarDi<T, B>, StateVarDi<B, T>)
    where
        B: From<T> + Clone + 'static,
        T: From<B> + 'static,
    {
        let b = use_state(self.get().into());
        (
            StateVarDi::new_use_into(*self, b),
            StateVarDi::new_use_into(b, *self),
        )
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
pub struct StateVarDi<A, B> {
    pub this: StateVar<A>,
    pub similar: StateVar<B>,
    update_fn: Box<dyn Fn(A, StateVar<B>)>,
}
impl<A, B, T> TryFrom<StateVarDi<A, B>> for StateVar<T>
where
    A: 'static,
    B: 'static,
    T: 'static,
{
    type Error = ();
    fn try_from(di: StateVarDi<A, B>) -> Result<Self, Self::Error> {
        if TypeId::of::<A>() == TypeId::of::<T>() {
            let any: Box<dyn std::any::Any> = Box::new(di.this);
            any.downcast::<Self>().map(|v| *v).map_err(|_| ())
        } else if TypeId::of::<B>() == TypeId::of::<T>() {
            let any: Box<dyn std::any::Any> = Box::new(di.similar);
            any.downcast::<Self>().map(|v| *v).map_err(|_| ())
        } else {
            panic!("not match any type")
        }
    }
}

impl<A, B> StateVarDi<A, B>
where
    B: From<A> + 'static,
    A: Clone + 'static,
{
    #[must_use]
    pub fn new_use_into(this: StateVar<A>, similar: StateVar<B>) -> Self {
        Self {
            this,
            similar,
            update_fn: Box::new(|a, sv_b| sv_b.set(a.into())),
        }
    }
}
impl<A, B> StateVarDi<A, B>
where
    B: 'static,
    A: Clone + 'static,
{
    #[must_use]
    pub fn new(
        this: StateVar<A>,
        similar: StateVar<B>,
        update_fn: Box<dyn Fn(A, StateVar<B>)>,
    ) -> Self {
        Self {
            this,
            similar,
            update_fn,
        }
    }

    pub fn set(&self, value: A) {
        self.this.set(value.clone());
        (self.update_fn)(value, self.similar);
    }
    #[must_use]
    pub fn get(&self) -> A {
        self.this.get()
    }
}

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
fn set_state_with_topo_id<T: 'static>(data: T, current_id: TopoKey) {
    G_STATE_STORE.with(|g_state_store_refcell| {
        g_state_store_refcell
            .borrow()
            .set_state_with_key::<T>(data, &StorageKey::TopoKey(current_id));
    });

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

fn get_anchor_val_in_var_with_topo_id<T: 'static + std::clone::Clone>(id: TopoKey) -> T {
    G_STATE_STORE.with(|g_state_store_refcell| {
        let store = g_state_store_refcell.borrow();
        let var_with_sa = store
            .get_state_with_id::<StateAnchor<T>>(&StorageKey::TopoKey(id))
            .expect("You are trying to get a var state that doesn't exist in this context!");
        store.engine_get((*var_with_sa.get()).clone().anchor())
    })
}

fn read_var_with_topo_id<F: FnOnce(&Var<T>) -> R, T: 'static, R>(id: TopoKey, func: F) -> R {
    G_STATE_STORE.with(|g_state_store_refcell| {
        func(
            g_state_store_refcell
                .borrow()
                .get_state_with_id::<T>(&StorageKey::TopoKey(id))
                .expect("You are trying to get a var state that doesn't exist in this context!"),
        )
    })
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

    #[test]
    #[wasm_bindgen_test]
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
