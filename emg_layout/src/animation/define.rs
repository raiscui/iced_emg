use emg_animation::{models::Property, Debuggable};
use emg_core::GenericSize;
use emg_state::{topo, use_state, CloneStateVar, StateVar};
use tracing::trace;

use crate::GenericSizeAnchor;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StateVarProperty(StateVar<Property>);

impl std::ops::Deref for StateVarProperty {
    type Target = StateVar<Property>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
        Self(sv.build_bi_similar_use_into_in_topo::<Property>())
    }
}
impl<T> From<T> for StateVarProperty
where
    T: NotStateVar + Clone + 'static + Into<Property>,
{
    #[topo::nested]
    fn from(v: T) -> Self {
        trace!("{} to StateVarProperty", &std::any::type_name::<T>());

        Self(use_state(v.into()))
    }
}

impl From<StateVarProperty> for StateVar<GenericSizeAnchor> {
    #[topo::nested]
    fn from(sv: StateVarProperty) -> Self {
        trace!("StateVarProperty to StateVar<GenericSizeAnchor>");

        use_state(
            //
            GenericSizeAnchor(sv.get_var_with(|v| v.watch().map(|p| p.clone().into()).into())),
        )
    }
}
