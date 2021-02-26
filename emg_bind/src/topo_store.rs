/*
 * @Author: Rais
 * @Date: 2021-02-24 19:38:42
 * @LastEditTime: 2021-02-26 10:08:26
 * @LastEditors: Rais
 * @Description:
 */

use std::marker::PhantomData;

use crate::G_STATE_STORE;
pub struct StateAccess<T> {
    pub id: TopoKey,
    _phantom_data: PhantomData<T>,
}
impl<T> std::fmt::Debug for StateAccess<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:#?})", self.id)
    }
}

impl<T> Copy for StateAccess<T> {}
impl<T> Clone for StateAccess<T> {
    fn clone(&self) -> StateAccess<T> {
        StateAccess::<T> {
            id: self.id,
            _phantom_data: PhantomData::<T>,
        }
    }
}

impl<T> StateAccess<T>
where
    T: 'static + Clone,
{
    pub fn new(id: TopoKey) -> StateAccess<T> {
        StateAccess {
            id,
            _phantom_data: PhantomData,
        }
    }

    // stores a value of type T in a backing Store
    pub fn set(self, value: T) {
        set_state_with_topo_id(value, self.id);
    }

    pub fn state_exists(self) -> bool {
        state_exists_for_topo_id::<T>(self.id)
    }

    pub fn get_with<F: Fn(&T) -> R, R>(&self, func: F) -> R {
        read_state_with_topo_id(self.id, func)
    }
}

pub trait CloneState<T>
where
    T: Clone + 'static,
{
    fn get(&self) -> T;

    fn soft_get(&self) -> Option<T>;
}

impl<T> CloneState<T> for StateAccess<T>
where
    T: Clone + 'static,
{
    /// returns a clone of the stored state panics if not stored.
    fn get(&self) -> T {
        clone_state_with_topo_id::<T>(self.id).expect("state should be present")
    }

    fn soft_get(&self) -> Option<T> {
        clone_state_with_topo_id::<T>(self.id)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum StorageKey {
    // SlottedKey(SlottedKey),
    TopoKey(TopoKey),
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct TopoKey {
    // pub ctx: Option<SlottedKey>,
    pub id: topo::CallId,
}

///
///  Uses the current topological id to create a new state accessor
///
pub fn state_exists_for_topo_id<T: 'static + Clone>(id: TopoKey) -> bool {
    G_STATE_STORE.with(|g_state_store_refcell| {
        g_state_store_refcell
            .borrow()
            .state_exists_with_id::<T>(StorageKey::TopoKey(id))
    })
}

/// Sets the state of type T keyed to the given TopoId
pub fn set_state_with_topo_id<T: 'static + Clone>(data: T, current_id: TopoKey) {
    G_STATE_STORE.with(|g_state_store_refcell| {
        g_state_store_refcell
            .borrow_mut()
            .set_state_with_id::<T>(data, &StorageKey::TopoKey(current_id))
    });

    // execute_reaction_nodes(&StorageKey::TopoKey(current_id));
}

pub fn clone_state_with_topo_id<T: 'static + Clone>(id: TopoKey) -> Option<T> {
    G_STATE_STORE.with(|g_state_store_refcell| {
        g_state_store_refcell
            .borrow_mut()
            .get_state_with_id::<T>(&StorageKey::TopoKey(id))
            .cloned()
    })
}
pub fn read_state_with_topo_id<T: 'static + Clone, F: FnOnce(&T) -> R, R>(
    id: TopoKey,
    func: F,
) -> R {
    let item = remove_state_with_topo_id::<T>(id)
        .expect("You are trying to read a type state that doesnt exist in this context!");
    let read = func(&item);
    set_state_with_topo_id(item, id);
    read
}
pub fn remove_state_with_topo_id<T: 'static + Clone>(id: TopoKey) -> Option<T> {
    G_STATE_STORE.with(|g_state_store_refcell| {
        g_state_store_refcell
            .borrow_mut()
            .remove_state_with_id::<T>(&StorageKey::TopoKey(id))
    })
}

#[topo::nested]
pub fn use_state<T: 'static + Clone>(data: T) -> StateAccess<T> {
    let id = topo::CallId::current();
    let id = TopoKey { id };

    if !state_exists_for_topo_id::<T>(id) {
        set_state_with_topo_id::<T>(data, id);
    }
    StateAccess::new(id)
}
