/*
 * @Author: Rais
 * @Date: 2023-03-29 11:12:34
 * @LastEditTime: 2023-04-19 15:11:02
 * @LastEditors: Rais
 * @Description:
 */
// ────────────────────────────────────────────────────────────────────────────────
mod collections;
use crate::general_traits::CloneStateAnchor;
pub use anchors::dict;
use anchors::expert::{
    cutoff, either, map, map_mut, refmap, then, AnchorInner, CastFromValOrAnchor,
};
pub use anchors::singlethread::Var;
use anchors::{
    collections::ord_map_methods::Dict,
    singlethread::{Anchor, Engine, MultiAnchor, ValOrAnchor},
};
use emg_common::{TypeCheck, TypeName, Vector};
use tracing::debug_span;

use crate::{general_fns::state_store_with, general_traits::StateTypeCheck};

// ─────────────────────────────────────────────────────────────────────────────

impl<T> StateTypeCheck for StateAnchor<T>
where
    T: TypeCheck,
{
    const INSIDE_TYPE_NAME: TypeName = T::TYPE_NAME;
}
// ────────────────────────────────────────────────────────────────────────────────

pub struct StateAnchor<T>(pub(crate) Anchor<T>);
impl<T> From<StateAnchor<T>> for ValOrAnchor<T> {
    fn from(value: StateAnchor<T>) -> Self {
        Self::Anchor(value.0)
    }
}

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
// impl<T> From<T> for StateAnchor<T>
// where
//     T: 'static,
// {
//     #[track_caller]
//     fn from(v: T) -> Self {
//         Self::constant(v)
//     }
// }

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

impl<K: Ord + Clone + PartialEq + 'static, V: Clone + PartialEq + 'static> StateAnchor<Dict<K, V>> {
    #[track_caller]
    #[must_use]
    #[inline]
    pub fn filter<F: FnMut(&K, &V) -> bool + 'static>(&self, pool_size: usize, f: F) -> Self {
        self.0.filter(pool_size, f).into()
    }
    #[track_caller]
    #[must_use]
    #[inline]
    pub fn filter_with_anchor<A, F>(&self, pool_size: usize, anchor: &StateAnchor<A>, f: F) -> Self
    where
        A: 'static + std::cmp::PartialEq + std::clone::Clone,
        F: FnMut(&A, &K, &V) -> bool + 'static,
    {
        self.0
            .filter_with_anchor(pool_size, anchor.anchor(), f)
            .into()
    }

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn map_<F: FnMut(&K, &V) -> T + 'static, T: Clone + PartialEq + 'static>(
        &self,
        pool_size: usize,
        f: F,
    ) -> StateAnchor<Dict<K, T>> {
        self.0.map_(pool_size, f).into()
    }
    #[track_caller]
    #[must_use]
    #[inline]
    pub fn map_with_anchor<A, F, T>(
        &self,
        pool_size: usize,
        anchor: &StateAnchor<A>,
        f: F,
    ) -> StateAnchor<Dict<K, T>>
    where
        A: 'static + std::cmp::PartialEq + Clone,
        F: FnMut(&A, &K, &V) -> T + 'static,
        T: Clone + PartialEq + 'static,
    {
        self.0.map_with_anchor(pool_size, anchor.anchor(), f).into()
    }

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn filter_map<F: FnMut(&K, &V) -> Option<T> + 'static, T: Clone + PartialEq + 'static>(
        &self,
        pool_size: usize,
        f: F,
    ) -> StateAnchor<Dict<K, T>> {
        self.0.filter_map(pool_size, f).into()
    }
    #[track_caller]
    #[must_use]
    #[inline]
    pub fn filter_map_with_anchor<A, F, T>(
        &self,
        pool_size: usize,
        anchor: &StateAnchor<A>,
        f: F,
    ) -> StateAnchor<Dict<K, T>>
    where
        A: 'static + std::cmp::PartialEq + Clone,
        F: FnMut(&A, &K, &V) -> Option<T> + 'static,
        T: Clone + PartialEq + 'static,
    {
        self.0
            .filter_map_with_anchor(pool_size, anchor.anchor(), f)
            .into()
    }

    /// Dict 增加/更新 K V 会增量执行 function f , 用于更新 out,
    /// Dict 移除 K V 并不会触发 out 的更新,
    #[track_caller]
    #[must_use]
    #[inline]
    pub fn increment_reduction<
        F: FnMut(&mut T, &K, &V) -> bool + 'static,
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
    #[inline]
    #[must_use]
    pub fn into_either(self) -> ValOrAnchor<T> {
        self.0.into()
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
        StateAnchor(self.0.map(f))
    }
    #[track_caller]
    #[inline]
    pub fn map_mut<Out, F>(&self, initial: Out, f: F) -> StateAnchor<Out>
    where
        Out: 'static,
        F: 'static,
        map_mut::MapMut<(Anchor<T>,), F, Out>: AnchorInner<Engine, Output = Out>,
    {
        StateAnchor(self.0.map_mut(initial, f))
    }

    #[track_caller]
    #[inline]
    pub fn then<Out, F>(&self, f: F) -> StateAnchor<Out>
    where
        F: 'static,
        Out: 'static,
        then::Then<(Anchor<T>,), Out, F, Engine>: AnchorInner<Engine, Output = Out>,
    {
        StateAnchor(self.0.then(f))
    }
    #[track_caller]
    #[inline]
    pub fn either<Out, F>(&self, f: F) -> StateAnchor<Out>
    where
        F: 'static,
        Out: 'static,
        either::Either<(Anchor<T>,), Out, F, Engine>: AnchorInner<Engine, Output = Out>,
    {
        StateAnchor(self.0.either(f))
    }

    #[track_caller]
    #[inline]
    pub fn refmap<F, Out>(&self, f: F) -> StateAnchor<Out>
    where
        Out: 'static,
        F: 'static,
        refmap::RefMap<(Anchor<T>,), F>: AnchorInner<Engine, Output = Out>,
    {
        StateAnchor(self.0.refmap(f))
    }
    #[track_caller]
    #[inline]
    pub fn cutoff<F, Out>(&self, f: F) -> StateAnchor<Out>
    where
        Out: 'static,
        F: 'static,
        cutoff::Cutoff<(Anchor<T>,), F>: AnchorInner<Engine, Output = Out>,
    {
        StateAnchor(self.0.cutoff(f))
    }
    #[track_caller]
    #[inline]
    #[must_use]
    pub fn debounce(&self) -> Self
    where
        T: Copy + PartialEq,
    {
        Self(self.0.debounce())
    }
    #[track_caller]
    #[inline]
    #[must_use]
    pub fn debounce_clone(&self) -> Self
    where
        T: Clone + PartialEq,
    {
        Self(self.0.debounce_clone())
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

    fn either<F, Out>(self, f: F) -> StateAnchor<Out>
    where
        F: 'static,
        Out: 'static,
        either::Either<Self::Target, Out, F, Engine>: AnchorInner<Engine, Output = Out>;

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
            fn either<F, Out>(self, f: F) -> StateAnchor<Out>
            where
                F: 'static,
                Out: 'static,
                either::Either<Self::Target, Out, F,Engine>: AnchorInner<Engine, Output=Out>,
            {
                ($(&self.$num.0,)+).either(f).into()
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
