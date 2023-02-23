use crate::{error::Error, GTreeBuilderElement};
use emg::{Direction, EdgeIndex};
use emg_common::{ResultWithRef, ResultWithSomething};

use super::{GraphEdit, GraphEditManyMethod, Mode};

// //@ EdgeMode ─────────────────────────────────────────────────────────────────────────────
// pub struct EdgeMode<I = ()>(PhantomData<I>);

// impl Mode for EdgeMode {
//     type Interface<'a, Ix> = EdittingGraphEdge<'a, Ix, Self> where Ix:'a;

//     fn interface<Ix, G>(g: &G) -> Self::Interface<'_, Ix>
//     where
//         G: GraphEditManyMethod<Ix = Ix>,
//     {
//         let inner = g;
//         let phantom_data = PhantomData;
//         EdittingGraphEdge {
//             inner,
//             phantom_data,
//         }
//     }
// }
//@ EdgeMode ─────────────────────────────────────────────────────────────────────────────
impl<Ix, Message> Mode<Message> for EdgeIndex<Ix> {
    type Ix = Ix;

    type Interface<'a> = EdittingGraphEdge<'a, Ix, Message> where Ix: 'a, Message: 'a;

    fn interface(
        self,
        g: &dyn GraphEditManyMethod<Message = Message, Ix = Ix>,
    ) -> Self::Interface<'_> {
        let inner = g;
        // let phantom_data = PhantomData;
        EdittingGraphEdge {
            edge: self,
            graph: inner,
            // phantom_data,
        }
    }
}
// ─────────────────────────────────────────────────────────────────────────────

pub struct EdittingGraphEdge<'a, Ix, Message> {
    edge: EdgeIndex<Ix>,
    graph: &'a dyn GraphEditManyMethod<Message = Message, Ix = Ix>,
    // pub(crate) phantom_data: PhantomData<Mod>,
}

impl<'a, Ix, Message> GraphEdit for EdittingGraphEdge<'a, Ix, Message> {
    type Ix = Ix;
    type Message = Message;

    /// &self  will drop , edge drop. return new editor interface
    fn edit<M: Mode<Message, Ix = Ix>>(&self, mode: M) -> M::Interface<'_> {
        mode.interface(self.graph)
    }
}

impl<'a, Ix, Message> EdittingGraphEdge<'a, Ix, Message> {
    pub fn moving(&self, dir: Direction, to: impl Into<Ix>) -> ResultWithRef<Self, (), Error> {
        self.graph
            .edge_plug_edit(&self.edge, dir, to.into())
            .with(self)
    }

    //TODO fn edit to edit other eg. node
}

//@ tree builder Mode ─────────────────────────────────────────────────────────────────────────────

impl<Ix, Message> Mode<Message> for GTreeBuilderElement<Message, Ix>
where
    Ix: Clone + std::hash::Hash + Ord + Default,
{
    type Ix = Ix;
    type Interface<'a> = EdittingGraphUseTreeBuilder<'a, Ix, Message>;

    fn interface(
        self,
        g: &dyn GraphEditManyMethod<Message = Message, Ix = Ix>,
    ) -> Self::Interface<'_> {
        let inner = g;
        // let phantom_data = PhantomData;
        EdittingGraphUseTreeBuilder {
            node: self,
            graph: inner,
        }
    }
}
// ─────────────────────────────────────────────────────────────────────────────

pub struct EdittingGraphUseTreeBuilder<'a, Ix, Message>
where
    Ix: Clone + std::hash::Hash + Ord + Default + 'static,
    Message: 'static,
{
    node: GTreeBuilderElement<Message, Ix>,
    graph: &'a dyn GraphEditManyMethod<Message = Message, Ix = Ix>,
    // pub(crate) phantom_data: PhantomData<M>,
}

impl<'a, Ix, Message> GraphEdit for EdittingGraphUseTreeBuilder<'a, Ix, Message>
where
    Ix: Clone + std::hash::Hash + Ord + Default + 'static,
    Message: 'static,
{
    type Ix = Ix;
    type Message = Message;

    /// &self  will drop , edge drop. return new editor interface
    fn edit<M: Mode<Message, Ix = Ix>>(&self, mode: M) -> M::Interface<'_> {
        mode.interface(self.graph)
    }
}

impl<'a, Ix, Message> EdittingGraphUseTreeBuilder<'a, Ix, Message>
where
    Ix: Clone + std::hash::Hash + Ord + Default,
{
    pub fn insert(&self, to: impl Into<Ix>) -> ResultWithRef<Self, (), Error> {
        todo!()
        // self.graph
        // self.graph
        //     .edge_plug_edit(&self.edge, dir, to.into())
        //     .with(self)
    }
}
