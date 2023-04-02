/*
 * @Author: Rais
 * @Date: 2023-03-29 14:30:01
 * @LastEditTime: 2023-03-29 15:31:03
 * @LastEditors: Rais
 * @Description:
 */

use std::{cell::RefCell, panic::Location, rc::Rc};

use anchors::singlethread::Var;
use emg_common::{TypeCheck, TypeName};
use tracing::trace;

use crate::{
    error::Error,
    general_fns::{
        insert_after_fn_common_in_topo, insert_before_fn_common_in_topo, remove_after_fn,
        start_set_var_and_run_before_after, state_store_with,
    },
    general_struct::TopoKey,
    general_traits::{BiState, StateFn},
    state_store,
    state_var::{use_state, StateVar},
    CloneState, GStateStore, SkipKeyCollection, StateTypeCheck, StorageKey,
};

impl<T> StateTypeCheck for StateVar<T>
where
    T: TypeCheck,
{
    const INSIDE_TYPE_NAME: TypeName = T::TYPE_NAME;
}

impl<T> StateFn<T> for StateVar<T> {
    fn id(&self) -> &TopoKey {
        &self.id
    }
}

impl<T> CloneState<T> for StateVar<T>
where
    T: Clone + 'static,
{
    type GetOut = T;

    #[inline]
    fn get_out_val(&self) -> T {
        self.get()
    }
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

    fn set(&self, value: impl Into<T>)
    // where
    //     T: std::fmt::Debug,
    {
        state_store_with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
            trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

            let store = g_state_store_refcell.borrow();
            self.store_set(&store, value);
        });
    }

    //TODO use illicit @set replace set_in_callback

    fn store_set(&self, store: &GStateStore, value: impl Into<T>) {
        let (var, before_fns, after_fns) = store.get_var_b_a_fn_collect::<Var<T>, T>(self.id);
        let current = var.get();
        start_set_var_and_run_before_after(
            self.id,
            var,
            current,
            &value.into(),
            before_fns,
            after_fns,
        );
    }

    fn set_with<X: Into<T>, F: Fn(&T) -> X>(&self, func: F) {
        state_store_with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
            trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

            let store = g_state_store_refcell.borrow();
            let (var, before_fns, after_fns) = store
                .opt_get_var_and_bf_af_use_id::<Var<T>, T>(self.id)
                .expect("set_state_with_key: can't set state that doesn't exist in this context!");

            let current = var.get();
            let data = func(&current).into();
            start_set_var_and_run_before_after(
                // store,
                self.id, var, current, &data, before_fns, after_fns,
            );
        });
    }
    fn set_with_once<X: Into<T>, F: FnOnce(&T) -> X>(&self, func_once: F) -> Result<(), Error> {
        state_store_with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
            trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

            let store = g_state_store_refcell.borrow();
            self.store_set_with_once(&store, func_once)
        })
    }
    fn opt_set_with_once<X: Into<T>, F: FnOnce(&T) -> Option<X>>(&self, func_once: F) {
        state_store_with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
            trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

            let store = g_state_store_refcell.borrow();
            let (var, before_fns, after_fns) = store
                .opt_get_var_and_bf_af_use_id::<Var<T>, T>(self.id)
                .expect("set_state_with_key: can't set state that doesn't exist in this context!");

            let current = var.get();
            let opt_data = func_once(&current);
            if let Some(data) = opt_data {
                start_set_var_and_run_before_after(
                    // store,
                    self.id,
                    var,
                    current,
                    &data.into(),
                    before_fns,
                    after_fns,
                );
            }
        });
    }

    /// # Errors
    ///
    /// Will return `Err` if can't get Var use id.
    /// permission to read it.
    fn store_set_with_once<X: Into<T>, F: FnOnce(&T) -> X>(
        &self,
        store: &GStateStore,
        func_once: F,
    ) -> Result<(), Error> {
        let (var, before_fns, after_fns) =
            store.opt_get_var_and_bf_af_use_id::<Var<T>, T>(self.id)?;
        let current = var.get();
        let data = func_once(&current).into();

        start_set_var_and_run_before_after(
            // store,
            self.id, var, current, &data, before_fns, after_fns,
        );
        Ok(())
    }

    fn update<F: FnOnce(&mut T) -> R, R>(&self, func: F) -> R {
        {
            state_store_with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
                trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

                let store = g_state_store_refcell.borrow();
                self.store_update(&store, func)
            })
        }
    }
    fn update_bool_check<F: FnOnce(&mut T) -> bool>(&self, func: F) -> bool {
        state_store_with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
            //----------
            let store = g_state_store_refcell.borrow();
            let (var, before_fns, after_fns) = store
                .opt_get_var_and_bf_af_use_id::<Var<T>, T>(self.id)
                .unwrap();

            let current = var.get();
            let mut edited_v = (*current).clone();

            let is_changed = func(&mut edited_v);
            if is_changed {
                start_set_var_and_run_before_after(
                    // store,
                    self.id, var, current, &edited_v, before_fns, after_fns,
                );
            }

            is_changed
        })
    }

    fn update_opt_check<F: FnOnce(&mut T) -> Option<R>, R>(&self, func: F) -> Option<R> {
        state_store_with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
            //----------
            let store = g_state_store_refcell.borrow();
            let (var, before_fns, after_fns) = store
                .opt_get_var_and_bf_af_use_id::<Var<T>, T>(self.id)
                .unwrap();
            let current = var.get();

            let mut edited_v = (*current).clone();

            let r = func(&mut edited_v);

            start_set_var_and_run_before_after(
                // store,
                self.id, var, current, &edited_v, before_fns, after_fns,
            );
            r
        })
    }

    fn store_update<F: FnOnce(&mut T) -> R, R>(&self, store: &GStateStore, func: F) -> R {
        let (var, before_fns, after_fns) = store
            .opt_get_var_and_bf_af_use_id::<Var<T>, T>(self.id)
            .unwrap();
        let current = var.get();

        let mut edited_v = (*current).clone();

        let r = func(&mut edited_v);

        start_set_var_and_run_before_after(
            // store,
            self.id, var, current, &edited_v, before_fns, after_fns,
        );
        r
    }
    fn store_update_result_check<F: FnOnce(&mut T) -> Result<R, E>, R, E>(
        &self,
        store: &GStateStore,
        func: F,
    ) -> Result<R, E> {
        let (var, before_fns, after_fns) = store
            .opt_get_var_and_bf_af_use_id::<Var<T>, T>(self.id)
            .unwrap();
        let current = var.get();

        let mut edited_v = (*current).clone();

        let r = func(&mut edited_v)?;
        start_set_var_and_run_before_after(
            // store,
            self.id, var, current, &edited_v, before_fns, after_fns,
        );

        Ok(r)
    }

    //TODO 回环检测 , 当两个或者两个以上 有 di关系的 StateVar  set的时候 会再次互相调用set -回环
    /// 添加不添加 deps 都不会使 after before func 循环,
    /// 但是添加deps 可以 在 deps-TopoKey 的StateVar drop时候 将该func drop,
    //
    /// 如果 deps is some , 则返回 none , Rc储存在deps中
    #[must_use]
    #[inline]
    #[topo::nested]
    fn insert_before_fn_in_topo(
        &self,
        func: impl Fn(&mut SkipKeyCollection, &Option<Rc<T>>, &T) + 'static,
        init: bool,
        deps: &[TopoKey],
    ) -> Option<Rc<StorageKey>> {
        insert_before_fn_common_in_topo::<Self, T, Var<T>>(self, Box::new(func), init, deps)
    }

    /// 添加不添加 deps 都不会使 after before func 循环,
    /// 但是添加deps 可以 在 deps-TopoKey 的StateVar drop时候 将该func drop,
    /// 同时如果 运行func时刻, skip有deps的key,则不会运行该func
    /// 如果 deps is some , 则返回 none , Rc储存在deps中
    #[inline]
    #[topo::nested]
    fn insert_after_fn_in_topo(
        &self,
        func: impl Fn(&mut SkipKeyCollection, &T) + 'static,
        init: bool,
        deps: &[TopoKey],
    ) -> Option<Rc<StorageKey>> {
        insert_after_fn_common_in_topo::<T, Var<T>>(self.id(), Box::new(func), init, deps)
    }
    #[inline]
    fn remove_after_fn(&self, after_fn_key: TopoKey) {
        remove_after_fn::<T>(*self.id(), &StorageKey::TopoKey(after_fn_key));
    }

    //手动 连接 statevar 与 function key , when statevar drop,then fk drop
    fn link_callback_drop(&self, fk: Rc<StorageKey>) {
        let state_store = state_store();
        let mut store = state_store.borrow_mut();
        store.link_callback_drop(self.id, fk);
    }
}

impl<T> BiState<T> for StateVar<T>
where
    T: Clone,
{
    type SV<X> = StateVar<X>;
    #[topo::nested]
    ///if self change , B will change too;
    fn build_similar_use_into_in_topo<B>(&self) -> StateVar<B>
    where
        T: std::fmt::Debug + 'static,
        B: Clone + PartialEq + From<T> + 'static + std::fmt::Debug,
    {
        let v = self.get();
        let b: StateVar<B> = use_state(|| v.clone().into());
        insert_before_fn_common_in_topo::<Self, T, Var<T>>(
            self,
            Box::new(move |skip, _current, value| {
                b.seting_in_b_a_callback(skip, || value.clone().into());
            }),
            false,
            &[b.id],
        );
        b
    }

    fn bi<B>(&self, b: StateVar<B>)
    where
        T: std::fmt::Debug + PartialEq + 'static,
        B: Clone + From<T> + Into<T> + 'static + std::fmt::Debug + PartialEq,
    {
        let v = self.get();
        b.set(v);
        let this = *self;

        insert_before_fn_common_in_topo::<Self, T, Var<T>>(
            self,
            Box::new(move |skip, _current, value| {
                b.seting_in_b_a_callback(skip, || value.clone().into());
            }),
            false,
            &[b.id],
        );

        insert_before_fn_common_in_topo::<StateVar<B>, B, Var<B>>(
            &b,
            Box::new(move |skip, _current, value| {
                this.seting_in_b_a_callback(skip, || value.clone().into());
            }),
            false,
            &[this.id],
        );
    }

    #[topo::nested]
    fn build_bi_similar_use_into_in_topo<B>(&self) -> StateVar<B>
    where
        T: std::fmt::Debug + PartialEq + 'static,
        B: Clone + From<T> + Into<T> + 'static + std::fmt::Debug + PartialEq,
    {
        let v = self.get();
        let b: StateVar<B> = use_state(|| v.into());

        let this = *self;

        insert_before_fn_common_in_topo::<Self, T, Var<T>>(
            self,
            Box::new(move |skip, _current, value| {
                b.seting_in_b_a_callback(skip, || value.clone().into());
            }),
            false,
            &[b.id],
        );

        insert_before_fn_common_in_topo::<StateVar<B>, B, Var<B>>(
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
}
