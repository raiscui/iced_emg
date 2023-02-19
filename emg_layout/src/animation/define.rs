use std::{cell::Cell, rc::Rc};

use emg_animation::{models::Property, Debuggable};
use emg_state::{state_store, topo, use_state, CloneStateVar, StateVar};
// use emg_state::{state_store, topo, use_state, CloneStateVar, StateVar, StorageKey};
use tracing::{debug, debug_span, trace};

use crate::GenericSizeAnchor;

/// 第一个 StateVarProperty Drop 将会 Drop 内部StateVar,以及相关依赖 before_fn after_fn,
/// clone的其他 StateVarProperty drop将没有任何额外操作
/// *建议 第一个用来 储存 和使用 ,clone的仅用来 使用
// TODO change to enum :DropEffect/ DropNoneEffect
#[derive(Debug, PartialEq, Eq)]
pub struct StateVarProperty {
    prop_sv: StateVar<Property>,
    // ref_count: Rc<Cell<usize>>,
    trace_id: usize,
}

impl Clone for StateVarProperty {
    fn clone(&self) -> Self {
        let _span = debug_span!("StateVarProperty clone").entered();

        // self.ref_count.set(self.ref_count.get() + 1);

        Self {
            prop_sv: self.prop_sv,
            // ref_count: self.ref_count.clone(),
            trace_id: self.trace_id + 1,
        }
    }
}

impl StateVarProperty {
    fn new(prop_sv: StateVar<Property>) -> Self {
        Self {
            prop_sv,
            // ref_count: Rc::new(Cell::new(1)),
            trace_id: 1,
        }
    }
}

impl Drop for StateVarProperty {
    fn drop(&mut self) {
        let _span = debug_span!("StateVarProperty drop",trace_id=%self.trace_id).entered();

        // let new_count = self.ref_count.get() - 1;

        // if new_count == 0 {
        //     debug!("will use sv var manually_drop");
        //     self.prop_sv.manually_drop();
        // } else {
        //     debug!("ddddd skip drop new_count:{}", new_count);

        //     self.ref_count.set(new_count);
        // }

        if self.trace_id == 1 {
            debug!("will use sv var manually_drop");
            self.prop_sv.manually_drop();
        } else {
            debug!("skip drop trace_id:{}", self.trace_id);
        }
    }
}

impl std::ops::Deref for StateVarProperty {
    type Target = StateVar<Property>;

    fn deref(&self) -> &Self::Target {
        &self.prop_sv
    }
}

// pub auto trait NotGenericSizeAnchor {} //GenericSizeAnchor
// impl<T> NotGenericSizeAnchor for T {}
// impl !NotGenericSizeAnchor for GenericSizeAnchor {}
pub auto trait NotStateVar {}

impl<T> !NotStateVar for StateVar<T> {}

impl<T> NotStateVar for Debuggable<T> {}
// impl NotStateVar for Debuggable<dyn Fn(Precision) -> Precision> {}

impl<T> From<StateVar<T>> for StateVarProperty
where
    T: Clone + 'static + From<Property> + std::fmt::Debug,
    Property: From<T>,
{
    #[topo::nested]
    fn from(sv: StateVar<T>) -> Self {
        trace!("StateVar to StateVarProperty");
        let _span = debug_span!("sv to svp").entered();
        let bi_self = sv.build_bi_similar_use_into_in_topo::<Property>();
        debug!("bi_self.id: {:?}", bi_self.id());
        Self::new(bi_self)
    }
}
impl<T> From<T> for StateVarProperty
where
    T: NotStateVar + Clone + 'static + From<Property>,
    Property: From<T>,
{
    #[topo::nested]
    fn from(v: T) -> Self {
        trace!("{} to StateVarProperty", &std::any::type_name::<T>());

        Self::new(use_state(|| v.into()))
    }
}

impl From<StateVarProperty> for StateVar<GenericSizeAnchor> {
    #[topo::nested]
    fn from(sv: StateVarProperty) -> Self {
        trace!("StateVarProperty to StateVar<GenericSizeAnchor>");

        use_state(||
            //
            //TODO impl new_from
            GenericSizeAnchor(sv.watch().map(|p| p.clone().into())))
    }
}
impl std::ops::ShlAssign<&StateVarProperty> for StateVar<GenericSizeAnchor> {
    fn shl_assign(&mut self, rhs: &StateVarProperty) {
        self.set(GenericSizeAnchor(
            // rhs.get_var_with(|v| v.watch().map(|p| p.clone().into()).into()),
            //TODO impl new_from
            rhs.watch().map(|p| p.clone().into()),
        ));
    }
}
impl std::ops::ShlAssign<StateVarProperty> for StateVar<GenericSizeAnchor> {
    fn shl_assign(&mut self, rhs: StateVarProperty) {
        self.set(GenericSizeAnchor(
            //TODO impl new_from
            //TODO check performance
            // rhs.get_var_with(|v| v.watch().map(|p| p.clone().into()).into()),
            rhs.watch().map(|p| p.clone().into()),
        ));
    }
}
