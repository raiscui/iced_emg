/*
 * @Author: Rais
 * @Date: 2023-03-28 16:36:33
 * @LastEditTime: 2023-03-31 16:05:37
 * @LastEditors: Rais
 * @Description:
 */

use std::{cell::RefCell, panic::Location, rc::Rc};

use anchors::{
    expert::CastIntoValOrAnchor,
    singlethread::{ValOrAnchor, VarVOA},
};
use emg_common::{TypeCheck, TypeName};
use tracing::trace;

use crate::{
    error::Error,
    general_fns::{
        insert_after_fn_common_in_topo, insert_before_fn_common_in_topo, remove_after_fn,
        start_set_var_and_run_before_after, state_store_with,
    },
    general_struct::TopoKey,
    general_traits::{BiState, CloneState, StateFn},
    state_store, use_state_voa, GStateStore, SkipKeyCollection, StateTypeCheck, StateVOA,
    StorageKey,
};

// ─────────────────────────────────────────────────────────────────────────────

impl<T> StateTypeCheck for StateVOA<T>
where
    T: TypeCheck,
{
    const INSIDE_TYPE_NAME: TypeName = T::TYPE_NAME;
}

impl<T> StateFn<ValOrAnchor<T>> for StateVOA<T> {
    fn id(&self) -> &TopoKey {
        &self.id
    }
}

impl<T> CloneState<ValOrAnchor<T>> for StateVOA<T>
where
    T: Clone + 'static,
{
    type GetOut = T;

    fn get_out_val(&self) -> T {
        self.get_with(|x| match x {
            anchors::expert::ValOrAnchor::Val(v) => v.clone(),
            anchors::expert::ValOrAnchor::Anchor(an) => {
                state_store_with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
                    let store = g_state_store_refcell.borrow();
                    store.engine_get(an)
                })
            }
        })
    }
    /// returns a clone of the stored state panics if not stored.
    fn get(&self) -> ValOrAnchor<T> {
        // let var = clone_state_with_topo_id::<T>(self.id).expect("state should be present");
        // (*var.get()).clone()
        self.get_with(std::clone::Clone::clone)
        // log::debug!("=====StateVar get {:?}", &t);
    }

    fn store_get(&self, store: &GStateStore) -> ValOrAnchor<T> {
        self.store_get_rc(store).as_ref().clone()
    }

    fn set(&self, value: impl Into<ValOrAnchor<T>>)
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

    fn store_set(&self, store: &GStateStore, value: impl Into<ValOrAnchor<T>>) {
        let (var, before_fns, after_fns) =
            store.get_var_b_a_fn_collect::<VarVOA<T>, ValOrAnchor<T>>(self.id);
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

    fn set_with<X: Into<ValOrAnchor<T>>, F: Fn(&ValOrAnchor<T>) -> X>(&self, func: F) {
        state_store_with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
            trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

            let store = g_state_store_refcell.borrow();
            let (var, before_fns, after_fns) = store
                .opt_get_var_and_bf_af_use_id::<VarVOA<T>, ValOrAnchor<T>>(self.id)
                .expect("set_state_with_key: can't set state that doesn't exist in this context!");

            let current = var.get();
            let data = func(&current).into();
            start_set_var_and_run_before_after(
                // store,
                self.id, var, current, &data, before_fns, after_fns,
            );
        });
    }
    fn set_with_once<X: Into<ValOrAnchor<T>>, F: FnOnce(&ValOrAnchor<T>) -> X>(
        &self,
        func_once: F,
    ) -> Result<(), Error> {
        state_store_with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
            trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

            let store = g_state_store_refcell.borrow();
            self.store_set_with_once(&store, func_once)
        })
    }
    fn opt_set_with_once<X: Into<ValOrAnchor<T>>, F: FnOnce(&ValOrAnchor<T>) -> Option<X>>(
        &self,
        func_once: F,
    ) {
        state_store_with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
            trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

            let store = g_state_store_refcell.borrow();
            let (var, before_fns, after_fns) = store
                .opt_get_var_and_bf_af_use_id::<VarVOA<T>, ValOrAnchor<T>>(self.id)
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
    fn store_set_with_once<X: Into<ValOrAnchor<T>>, F: FnOnce(&ValOrAnchor<T>) -> X>(
        &self,
        store: &GStateStore,
        func_once: F,
    ) -> Result<(), Error> {
        let (var, before_fns, after_fns) =
            store.opt_get_var_and_bf_af_use_id::<VarVOA<T>, ValOrAnchor<T>>(self.id)?;
        let current = var.get();
        let data = func_once(&current).into();

        start_set_var_and_run_before_after(
            // store,
            self.id, var, current, &data, before_fns, after_fns,
        );
        Ok(())
    }

    fn update<F: FnOnce(&mut ValOrAnchor<T>) -> R, R>(&self, func: F) -> R {
        {
            state_store_with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
                trace!("G_STATE_STORE::borrow:\n{}", Location::caller());

                let store = g_state_store_refcell.borrow();
                self.store_update(&store, func)
            })
        }
    }
    fn update_bool_check<F: FnOnce(&mut ValOrAnchor<T>) -> bool>(&self, func: F) -> bool {
        state_store_with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
            //----------
            let store = g_state_store_refcell.borrow();
            let (var, before_fns, after_fns) = store
                .opt_get_var_and_bf_af_use_id::<VarVOA<T>, ValOrAnchor<T>>(self.id)
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

    fn update_opt_check<F: FnOnce(&mut ValOrAnchor<T>) -> Option<R>, R>(
        &self,
        func: F,
    ) -> Option<R> {
        state_store_with(|g_state_store_refcell: &Rc<RefCell<GStateStore>>| {
            //----------
            let store = g_state_store_refcell.borrow();
            let (var, before_fns, after_fns) = store
                .opt_get_var_and_bf_af_use_id::<VarVOA<T>, ValOrAnchor<T>>(self.id)
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

    fn store_update<F: FnOnce(&mut ValOrAnchor<T>) -> R, R>(
        &self,
        store: &GStateStore,
        func: F,
    ) -> R {
        let (var, before_fns, after_fns) = store
            .opt_get_var_and_bf_af_use_id::<VarVOA<T>, ValOrAnchor<T>>(self.id)
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
    fn store_update_result_check<F: FnOnce(&mut ValOrAnchor<T>) -> Result<R, E>, R, E>(
        &self,
        store: &GStateStore,
        func: F,
    ) -> Result<R, E> {
        let (var, before_fns, after_fns) = store
            .opt_get_var_and_bf_af_use_id::<VarVOA<T>, ValOrAnchor<T>>(self.id)
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
        func: impl Fn(&mut SkipKeyCollection, &Option<Rc<ValOrAnchor<T>>>, &ValOrAnchor<T>) + 'static,
        init: bool,
        deps: &[TopoKey],
    ) -> Option<Rc<StorageKey>> {
        insert_before_fn_common_in_topo::<Self, ValOrAnchor<T>, VarVOA<T>>(
            self,
            Box::new(func),
            init,
            deps,
        )
    }

    /// 添加不添加 deps 都不会使 after before func 循环,
    /// 但是添加deps 可以 在 deps-TopoKey 的StateVar drop时候 将该func drop,
    /// 同时如果 运行func时刻, skip有deps的key,则不会运行该func
    /// 如果 deps is some , 则返回 none , Rc储存在deps中
    #[inline]
    #[topo::nested]
    fn insert_after_fn_in_topo(
        &self,
        func: impl Fn(&mut SkipKeyCollection, &ValOrAnchor<T>) + 'static,
        init: bool,
        deps: &[TopoKey],
    ) -> Option<Rc<StorageKey>> {
        insert_after_fn_common_in_topo::<ValOrAnchor<T>, VarVOA<T>>(
            self.id(),
            Box::new(func),
            init,
            deps,
        )
    }
    #[inline]
    fn remove_after_fn(&self, after_fn_key: TopoKey) {
        remove_after_fn::<ValOrAnchor<T>>(*self.id(), &StorageKey::TopoKey(after_fn_key));
    }

    //手动 连接 statevar 与 function key , when statevar drop,then fk drop
    fn link_callback_drop(&self, fk: Rc<StorageKey>) {
        let state_store = state_store();
        let mut store = state_store.borrow_mut();
        store.link_callback_drop(self.id, fk);
    }
}

// impl<T> BiState<T> for StateVOA<T>
// where
//     T: Clone,
// {
//     type SV<X> = StateVOA<X>;
//     #[topo::nested]
//     ///if self change , B will change too;
//     fn build_similar_use_into_in_topo<B>(&self) -> StateVOA<B>
//     where
//         T: std::fmt::Debug + 'static,
//         B: Clone + PartialEq + From<T> + 'static + std::fmt::Debug,
//     {
//         let v = self.get();
//         let b: StateVOA<B> = use_state_voa(|| v.clone().cast_into());
//         insert_before_fn_common_in_topo::<Self, ValOrAnchor<T>, VarVOA<T>>(
//             self,
//             Box::new(move |skip, _current, value| {
//                 b.seting_in_b_a_callback(skip, || value.clone().cast_into());
//             }),
//             false,
//             &[b.id],
//         );
//         b
//     }

//     fn bi<B>(&self, b: StateVOA<B>)
//     where
//         T: std::fmt::Debug + PartialEq + 'static,
//         B: Clone + From<T> + Into<T> + 'static + std::fmt::Debug + PartialEq,
//     {
//         let v = self.get();
//         b.set(v.cast_into());
//         let this = *self;

//         insert_before_fn_common_in_topo::<Self, ValOrAnchor<T>, VarVOA<T>>(
//             self,
//             Box::new(move |skip, _current, value| {
//                 b.seting_in_b_a_callback(skip, || value.clone().cast_into());
//             }),
//             false,
//             &[b.id],
//         );

//         insert_before_fn_common_in_topo::<StateVOA<B>, ValOrAnchor<B>, VarVOA<B>>(
//             &b,
//             Box::new(move |skip, _current, value| {
//                 this.seting_in_b_a_callback(skip, || value.clone().cast_into());
//             }),
//             false,
//             &[this.id],
//         );
//     }

//     #[topo::nested]
//     fn build_bi_similar_use_into_in_topo<B>(&self) -> StateVOA<B>
//     where
//         T: std::fmt::Debug + PartialEq + 'static,
//         B: Clone + From<T> + Into<T> + 'static + std::fmt::Debug + PartialEq,
//     {
//         let v = self.get();
//         let b: StateVOA<B> = use_state_voa(|| v.cast_into());

//         let this = *self;

//         insert_before_fn_common_in_topo::<Self, ValOrAnchor<T>, VarVOA<T>>(
//             self,
//             Box::new(move |skip, _current, value| {
//                 b.seting_in_b_a_callback(skip, || value.clone().cast_into());
//             }),
//             false,
//             &[b.id],
//         );

//         insert_before_fn_common_in_topo::<StateVOA<B>, ValOrAnchor<B>, VarVOA<B>>(
//             &b,
//             Box::new(move |skip, _current, value| {
//                 this.seting_in_b_a_callback(skip, || value.clone().cast_into());
//             }),
//             false,
//             &[this.id],
//         );

//         // .expect("insert_before_fn error");
//         b
//     }
// }

impl<T> BiState<T> for StateVOA<T>
where
    T: Clone,
{
    type SV<X> = StateVOA<X>;
    #[topo::nested]
    ///if self change , B will change too;
    fn build_similar_use_into_in_topo<B>(&self) -> StateVOA<B>
    where
        T: std::fmt::Debug + 'static,
        B: Clone + PartialEq + From<T> + 'static + std::fmt::Debug,
    {
        let sw = self.watch();
        let b: StateVOA<B> = use_state_voa(|| sw.cast_into());
        b
    }

    fn bi<B>(&self, b: StateVOA<B>)
    where
        T: std::fmt::Debug + PartialEq + 'static,
        B: Clone + From<T> + Into<T> + 'static + std::fmt::Debug + PartialEq,
    {
        let sw = self.watch();

        b.set(self.watch().cast_into());
        let bw = b.watch();

        let this = *self;

        self.insert_before_fn_in_topo(
            move |skip, _current, _value| {
                b.seting_in_b_a_callback(skip, || sw.clone().cast_into());
            },
            false,
            &[b.id],
        );

        b.insert_before_fn_in_topo(
            move |skip, _current, _value| {
                this.seting_in_b_a_callback(skip, || bw.clone().cast_into());
            },
            false,
            &[this.id],
        );
    }

    #[topo::nested]
    fn build_bi_similar_use_into_in_topo<B>(&self) -> StateVOA<B>
    where
        T: std::fmt::Debug + PartialEq + 'static,
        B: Clone + From<T> + Into<T> + 'static + std::fmt::Debug + PartialEq,
    {
        let sw = self.watch();

        let b: StateVOA<B> = use_state_voa(|| sw.clone().cast_into());
        let bw = b.watch();

        let this = *self;

        self.insert_before_fn_in_topo(
            move |skip, _current, _value| {
                b.seting_in_b_a_callback(skip, || sw.clone().cast_into());
            },
            false,
            &[b.id],
        );

        b.insert_before_fn_in_topo(
            move |skip, _current, _value| {
                this.seting_in_b_a_callback(skip, || bw.clone().cast_into());
            },
            false,
            &[this.id],
        );

        b
    }
}

#[cfg(test)]
#[allow(clippy::many_single_char_names)]
#[allow(clippy::let_unit_value)]
#[allow(clippy::fallible_impl_from)]
#[allow(clippy::disallowed_types)]
#[allow(unused)]
mod svoa_test {
    use crate::{
        g_store::G_STATE_STORE,
        general_fns::state_store_with,
        general_struct::TopoKey,
        general_traits::{BiState, StateFn},
        topo, use_state_voa, CloneState, CloneStateAnchor, StateAnchor, StateVOA, StorageKey,
    };
    use std::{collections::HashMap, rc::Rc};

    use anchors::{
        collections::ord_map_methods::Dict, dict, expert::force_op, singlethread::VarVOA,
    };
    use tracing::{debug, trace};

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

    #[derive(Clone, Debug, PartialEq)]
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
        let a = use_state_voa(|| 1);
        let b = use_state_voa(|| 2);
        assert_ne!(a.id(), b.id());
    }

    #[test]
    // #[wasm_bindgen_test]
    fn callback() {
        use BiState;
        let _g = tracing_init();
        let a = use_state_voa(|| 1);
        let b = a.build_similar_use_into_in_topo::<TT>();
        debug!("init: a:{:?} b:{:?}", &a, &b);
        a.set(2);
        debug!("a set 2 : a:{:?} b:{:?}", &a, &b);
        assert_eq!(format!("{:?}", a.get_out_val()), b.get_out_val().0);
        let c = a.build_bi_similar_use_into_in_topo::<TT>();
        debug!("build c : a:{:?} b:{:?} c:{:?}", &a, &b, &c);
        c.set(TT("3".to_string()));
        debug!("c set '3' : a:{:?} b:{:?} c:{:?}", &a, &b, &c);
        a.set(9);
        debug!("a set 9: a:{:?} b:{:?} c:{:?}", &a, &b, &c);
        let d = c.build_similar_use_into_in_topo::<i32>();

        assert_eq!(a.get_out_val(), d.get_out_val());
        a.set(19);
        assert_eq!(a.get_out_val(), d.get_out_val());
    }

    #[test]
    #[topo::nested]
    fn callback2_clone() {
        let _g = tracing_init();

        let a = use_state_voa(|| 1);
        let a_2 = use_state_voa(|| 11);
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

        let a = use_state_voa(|| 1);
        let a_2 = use_state_voa(|| 1);
        let update_id_a_2 = TopoKey::new(topo::call(topo::CallId::current));

        a_2.insert_before_fn_in_topo(
            move |_skip, current, value| {
                println!("current:{:?}", &current);
                println!("value:{value}");
                assert_eq!(current, &Some(Rc::new(1.into())));
                assert_eq!(*value.as_val().unwrap(), 2);
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
                c.seting_in_b_a_callback(skip, || value.clone());
            },
            true,
            &[c.id],
        );

        let update_id = TopoKey::new(topo::call(topo::CallId::current));

        c.insert_before_fn_in_topo(
            move |skip, current, value| {
                println!("c -> before_fns 1 -> set a:{:?}", &value);

                a.seting_in_b_a_callback(skip, || value.clone());
            },
            true,
            &[a.id],
        );

        let update_id2 = TopoKey::new(topo::call(topo::CallId::current));

        //NOTE same a set_in_callback will ignored at second times
        c.insert_before_fn_in_topo(
            move |skip, current, value| {
                println!("c -> before_fns 2 -> set a:{value:?}");
                a.seting_in_b_a_callback(skip, || force_op(value.clone()) + 1);
            },
            true,
            &[],
        )
        .expect("");
        let e = use_state_voa(|| 11);
        c.insert_after_fn_in_topo(
            move |skip, v| {
                e.seting_in_b_a_callback(skip, || v.clone());
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
        assert_eq!(a.get_out_val(), c.get_out_val());
        assert_eq!(a.get_out_val(), d.get_out_val());
        assert_eq!(a.get_out_val(), e.get_out_val());
        c.set(3);
        assert_eq!(a.get_out_val(), c.get_out_val());
        assert_eq!(a.get_out_val(), d.get_out_val());
        d.set(4);
        assert_eq!(a.get_out_val(), c.get_out_val());
        assert_eq!(a.get_out_val(), d.get_out_val());
        assert_eq!(a.get_out_val(), e.get_out_val());
        c.remove_after_fn(e.id);
    }
    #[test]
    // #[wasm_bindgen_test]
    fn update() {
        let a = use_state_voa(|| 111);
        a.update(|aa| *aa += 1);
        println!("{}", &a);
        // ─────────────────────────────────────────────────────────────

        assert_eq!(a.get(), 112);
        // ─────────────────────────────────────────────────────────────

        a.update(|aa| *aa -= 2);
        println!("{}", &a);
        assert_eq!(a.get(), 110);
    }

    // #[wasm_bindgen_test]
    #[test]

    fn sa_in_sv() {
        let n = 1u8;
        let nn: u32 = n.into();
        let x = use_state_voa(|| 1u8);
        let xw = x.watch();
        let a: StateVOA<u32> = use_state_voa(|| xw.clone().cast_into());
        println!("{a}");
        println!("{}", a.get());
        assert_eq!(1u32, a.get_out_val());
    }
    #[test]
    fn macros() {
        let ffss = dict! {1=>1};
        println!("{ffss:?}");
    }
    #[allow(clippy::similar_names)]
    #[test]
    fn xx() {
        let a = use_state_voa(|| 99);

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

        let dd = VarVOA::new(99);
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
        let a = use_state_voa(|| dict.clone());
        let a_node1 = use_state_voa(|| 1);
        let a_node2 = use_state_voa(|| 2);
        let a_node0 = use_state_voa(|| 0);

        let b = a.watch().map_(1, |_, x: &StateVOA<i32>| {
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
        let a = use_state_voa(|| dict.clone());
        let a_node1 = use_state_voa(|| 1);
        let a_node2 = use_state_voa(|| 2);
        let a_node0 = use_state_voa(|| 0);

        let b = a
            .watch()
            .map_(1, |_, x: &StateAnchor<i32>| x.map(|xx| xx + 1));

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
        let a = use_state_voa(|| 1);

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
            let var_map = store.get_secondarymap::<VarVOA<i32>>().unwrap();
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
                let var_map = store.get_secondarymap::<VarVOA<i32>>().unwrap();
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
