use std::marker::PhantomData;

use crate::error::Error;
use emg::{Direction, EdgeIndex};

use super::{GraphEditManyMethod, Mode};

//@ EdgeMode ─────────────────────────────────────────────────────────────────────────────
pub struct EdgeMode<I = ()>(PhantomData<I>);

impl Mode for EdgeMode {
    type Interface<'a, Ix> = EdittingGraphEdge<'a, Ix, Self> where Ix:'a;

    fn interface<Ix, G>(g: &G) -> Self::Interface<'_, Ix>
    where
        G: GraphEditManyMethod<Ix = Ix>,
    {
        let inner = g;
        let phantom_data = PhantomData;
        EdittingGraphEdge {
            inner,
            phantom_data,
        }
    }
}
// ─────────────────────────────────────────────────────────────────────────────

pub struct EdittingGraphEdge<'a, Ix, M> {
    inner: &'a dyn GraphEditManyMethod<Ix = Ix>,
    pub(crate) phantom_data: PhantomData<M>,
}

impl<'a, Ix, M> EdittingGraphEdge<'a, Ix, M> {
    pub fn moving(
        &self,
        who: impl Into<EdgeIndex<Ix>>,
        dir: Direction,
        to: impl Into<Ix>,
    ) -> Result<(), Error> {
        self.inner.edge_plug_edit(&who.into(), dir, to.into())
    }

    //TODO fn edit to edit other eg. node
}
