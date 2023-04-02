/*
 * @Author: Rais
 * @Date: 2023-03-28 16:25:08
 * @LastEditTime: 2023-03-29 15:42:45
 * @LastEditors: Rais
 * @Description:
 */

use std::rc::Rc;
pub mod impls;

use emg_common::TypeName;

use crate::{
    error::Error,
    g_store::{GStateStore, SkipKeyCollection},
    general_struct::{StorageKey, TopoKey},
};

pub trait StateTypeCheck {
    const INSIDE_TYPE_NAME: TypeName;
}
pub trait VTFn<T> {
    fn new(val: T) -> Self;
    /// Get a reference to the state var's id.
    #[must_use]
    fn get(&self) -> Rc<T>;
    fn set(&self, val: T);
}
pub trait StateFn<VOA> {
    /// Get a reference to the state var's id.
    #[must_use]
    fn id(&self) -> &TopoKey;
}

pub trait CloneState<VOA> {
    type GetOut;

    fn get_out_val(&self) -> Self::GetOut;
    fn get(&self) -> VOA;
    fn store_get(&self, store: &GStateStore) -> VOA;
    fn set(&self, value: impl Into<VOA>);
    // fn set_in_callback(&self, store: &GStateStore, skip: &SkipKeyCollection, value: &T)
    // where
    //     T: std::fmt::Debug;
    fn store_set(&self, store: &GStateStore, value: impl Into<VOA>);
    fn opt_set_with_once<X: Into<VOA>, F: FnOnce(&VOA) -> Option<X>>(&self, func_once: F);

    /// # Errors
    ///
    /// Will return `Err` if can't get Var use id.
    /// permission to read it.
    fn set_with_once<X: Into<VOA>, F: FnOnce(&VOA) -> X>(&self, func_once: F) -> Result<(), Error>;

    /// # Errors
    ///
    /// Will return `Err` if got [Error]
    /// permission to read it.
    fn store_set_with_once<X: Into<VOA>, F: FnOnce(&VOA) -> X>(
        &self,
        store: &GStateStore,
        func_once: F,
    ) -> Result<(), Error>;
    fn set_with<X: Into<VOA>, F: Fn(&VOA) -> X>(&self, func: F);
    // fn try_get(&self) -> Option<T>;

    // fn update<F: FnOnce(&mut T)>(&self, func: F);
    fn update<F: FnOnce(&mut VOA) -> R, R>(&self, func: F) -> R;
    fn update_bool_check<F: FnOnce(&mut VOA) -> bool>(&self, func: F) -> bool;

    fn update_opt_check<F: FnOnce(&mut VOA) -> Option<R>, R>(&self, func: F) -> Option<R>;

    fn store_update<F: FnOnce(&mut VOA) -> R, R>(&self, store: &GStateStore, func: F) -> R;
    /// # Errors
    ///
    /// Will return `Err` if can't get Var use id. or func return error
    /// permission to read it.
    fn store_update_result_check<F: FnOnce(&mut VOA) -> Result<R, E>, R, E>(
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
        func: impl Fn(&mut SkipKeyCollection, &Option<Rc<VOA>>, &VOA) + 'static,
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
        func: impl Fn(&mut SkipKeyCollection, &VOA) + 'static,
        init: bool,
        deps: &[TopoKey],
    ) -> Option<Rc<StorageKey>>;

    fn remove_after_fn(&self, callback_key: TopoKey);

    fn link_callback_drop(&self, fk: Rc<StorageKey>);

    // fn to_bi_in_topo<B>(&self) -> (StateVarDi<T, B>, StateVarDi<B, T>)
    // where
    //     B: From<T> + Clone + 'static,
    //     T: From<B> + 'static;
}

pub trait BiState<T> {
    type SV<X>;
    fn build_similar_use_into_in_topo<B>(&self) -> Self::SV<B>
    where
        T: std::fmt::Debug + 'static,
        B: PartialEq,
        B: Clone + From<T> + 'static + std::fmt::Debug;
    fn build_bi_similar_use_into_in_topo<B>(&self) -> Self::SV<B>
    where
        T: std::fmt::Debug + PartialEq + 'static,
        B: Clone + From<T> + Into<T> + 'static + std::fmt::Debug + PartialEq;

    fn bi<B>(&self, b: Self::SV<B>)
    where
        T: std::fmt::Debug + PartialEq + 'static,
        B: Clone + From<T> + Into<T> + 'static + std::fmt::Debug + PartialEq;
}

// ─────────────────────────────────────────────────────────────────────────────

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
