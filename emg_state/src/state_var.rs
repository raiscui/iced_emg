/*
 * @Author: Rais
 * @Date: 2023-03-27 18:08:54
 * @LastEditTime: 2023-04-19 12:51:14
 * @LastEditors: Rais
 * @Description:
 */
// ─────────────────────────────────────────────────────────────────────────────

use std::{cell::RefCell, marker::PhantomData, panic::Location, rc::Rc};

use anchors::singlethread::Var;
use tracing::{debug, trace, warn};

use crate::{
    general_fns::{or_insert_var_with_topo_id, state_exists_for_topo_id, state_store_with},
    general_struct::TopoKey,
    CloneState, GStateStore, SkipKeyCollection, StateAnchor, StorageKey,
};
// ─────────────────────────────────────────────────────────────────────────────

/// 没有 #[[`topo::nested`]] 的函数,call结果就是 同一个[`StateVarVal`].
#[must_use]
#[track_caller]
pub fn use_state<F, T>(func: F) -> StateVar<T>
where
    T: 'static,
    F: FnOnce() -> T,
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
                // let old = StateVar::<T>::new(id);
                // let old_v = old.get_rc();
                // let v = func();

                warn!(target:"use_state","this is checker: use_state call again, StateVarVal already settled state ->{} ,\n Location: {}",std::any::type_name::<T>(),loc);
                // warn!(target:"use_state","this is checker: use_state call again, StateVarVal already settled state ->{} ,\n Location: {},\n old_v:{:?},\n new V:{:?}",std::any::type_name::<T>(),loc,old_v,v);
                // if format!("{old_v:?}") != format!("{v:?}") {
                //     warn!("val changed !!!!!!!!!!!!!!!!!!!!!!!!");
                // }
                return StateVar::new(id);
            }
        }
        or_insert_var_with_topo_id::<Var<T>, T, _>(func, id);
        StateVar::new(id)
    })
}
// ─────────────────────────────────────────────────────────────────────────────

pub struct StateVar<T> {
    pub(crate) id: TopoKey,
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
// impl<T: 'static + std::fmt::Display + Clone> std::fmt::Display for StateVarVal<StateAnchor<T>> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let v = self.get();
//         write!(f, "\u{2726} ({})", &v)
//     }
// }

impl<T: 'static + std::fmt::Debug + Clone> std::fmt::Debug for StateVar<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.get();
        f.debug_tuple("StateVarVal").field(&v).finish()
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
    T: 'static,
{
    #[must_use]
    pub const fn id(&self) -> &TopoKey {
        &self.id
    }
    /// # Panics
    ///
    /// Will panic if `store.id_to_key_map` not have Self `topo_key`
    pub fn manually_drop(&self) {
        debug!("StateVar<{}> drop .. ", std::any::type_name::<T>(),);

        // let store = state_store();
        state_store_with(|g_state_store_refcell| {
            debug!("in store");
            let mut store = g_state_store_refcell.borrow_mut();

            let topo_key = StorageKey::TopoKey(self.id);
            let key = store.id_to_key_map.remove(&topo_key).unwrap();

            store.primary_slotmap.remove(key);
            store.b_a_fn_drop_link_map.remove(key);
        });
        // .ok();
    }

    #[must_use]
    const fn new(id: TopoKey) -> Self {
        Self {
            id,
            _phantom_data: PhantomData,
        }
    }

    // #[must_use]
    // #[inline]
    // pub fn state_exists(&self) -> bool {
    //     state_exists_for_topo_id::<T>(self.id)
    // }

    #[inline]
    pub fn get_with<F: Fn(&T) -> R, R>(&self, func: F) -> R {
        self.get_var_with(|v| func(v.get().as_ref()))
    }

    #[must_use]
    #[inline]
    pub fn store_get_rc(&self, store: &GStateStore) -> Rc<T> {
        self.store_get_var_with(store, Var::get)
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
        state_store_with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
            trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

            let store = g_state_store_refcell.borrow();
            self.store_get_var_with(&store, func)
        })
    }
    pub fn store_get_var_with<F: Fn(&Var<T>) -> R, R>(&self, store: &GStateStore, func: F) -> R {
        let var = &store
            .opt_get_var_use_id::<Var<T>>(&StorageKey::TopoKey(self.id))
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
    pub fn seting_in_b_a_callback(&self, skip: &mut SkipKeyCollection, data_fn: impl FnOnce() -> T)
    where
        T: Clone + std::fmt::Debug,
    {
        state_store_with(|g_state_store_refcell| {
            trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

            g_state_store_refcell
                .borrow()
                .set_in_callback::<Var<T>, T>(skip, data_fn, self.id);
        });
    }
}
