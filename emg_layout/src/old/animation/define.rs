use emg_animation::{
    models::{Property, PropertyOG},
    Debuggable,
};
use emg_state::{topo, use_state, CloneStateVar, StateVar};
use tracing::trace;

use crate::old::GenericSizeAnchor;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StateVarPropertyOG(StateVar<PropertyOG>);

impl std::ops::Deref for StateVarPropertyOG {
    type Target = StateVar<PropertyOG>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// pub auto trait NotGenericSizeAnchor {} //GenericSizeAnchor
// impl<T> NotGenericSizeAnchor for T {}
// impl !NotGenericSizeAnchor for GenericSizeAnchor {}
pub auto trait NotStateVar {}

impl<T> !NotStateVar for StateVar<T> {}

impl<T> NotStateVar for Debuggable<T> {}
// impl NotStateVar for Debuggable<dyn Fn(Precision) -> Precision> {}

impl<T> From<StateVar<T>> for StateVarPropertyOG
where
    T: Clone + 'static + From<PropertyOG> + std::fmt::Debug,
    PropertyOG: From<T>,
{
    #[topo::nested]
    fn from(sv: StateVar<T>) -> Self {
        trace!("StateVar to StateVarPropertyOG");
        Self(sv.build_bi_similar_use_into_in_topo::<PropertyOG>())
    }
}
impl<T> From<T> for StateVarPropertyOG
where
    T: NotStateVar + Clone + 'static + From<PropertyOG>,
    PropertyOG: From<T>,
{
    #[topo::nested]
    fn from(v: T) -> Self {
        trace!("{} to StateVarPropertyOG", &std::any::type_name::<T>());

        Self(use_state(v.into()))
    }
}

impl From<StateVarPropertyOG> for StateVar<GenericSizeAnchor> {
    #[topo::nested]
    fn from(sv: StateVarPropertyOG) -> Self {
        trace!("StateVarPropertyOG to StateVar<GenericSizeAnchor>");

        use_state(
            //
            GenericSizeAnchor(sv.get_var_with(|v| v.watch().map(|p| p.clone().into()).into())),
        )
    }
}
impl std::ops::ShlAssign<&StateVarPropertyOG> for StateVar<GenericSizeAnchor> {
    fn shl_assign(&mut self, rhs: &StateVarPropertyOG) {
        self.set(GenericSizeAnchor(
            rhs.get_var_with(|v| v.watch().map(|p| p.clone().into()).into()),
        ));
    }
}
impl std::ops::ShlAssign<StateVarPropertyOG> for StateVar<GenericSizeAnchor> {
    fn shl_assign(&mut self, rhs: StateVarPropertyOG) {
        self.set(GenericSizeAnchor(
            rhs.get_var_with(|v| v.watch().map(|p| p.clone().into()).into()),
        ));
    }
}
