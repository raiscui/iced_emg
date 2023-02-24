use crate::{error::Error, GTreeBuilderElement};
use emg::{Direction, EdgeIndex};
use emg_common::{IdStr, ResultWithRef, ResultWithSomething};
use emg_state::topo;
use tracing::{debug, debug_span};

use super::{GraphEdit, GraphEditManyMethod, Mode};

// //@ EdgeMode ─────────────────────────────────────────────────────────────────────────────
// pub struct EdgeMode<I = ()>(PhantomData<I>);

// impl Mode for EdgeMode {
//     type Interface<'a> = EdittingGraphEdge<'a,  Self> ;

//     fn interface< G>(g: &G) -> Self::Interface<'_>
//     where
//         G: GraphEditManyMethod<>,
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
impl<Message> Mode<Message> for EdgeIndex {
    type Interface<'a> = EdittingGraphEdge<'a, Message> where  Message: 'a;

    fn interface(self, g: &dyn GraphEditManyMethod<Message = Message>) -> Self::Interface<'_> {
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

pub struct EdittingGraphEdge<'a, Message> {
    edge: EdgeIndex,
    graph: &'a dyn GraphEditManyMethod<Message = Message>,
    // pub(crate) phantom_data: PhantomData<Mod>,
}

impl<'a, Message> GraphEdit for EdittingGraphEdge<'a, Message> {
    type Message = Message;

    /// &self  will drop , edge drop. return new editor interface
    fn edit<M: Mode<Message>>(&self, mode: M) -> M::Interface<'_> {
        mode.interface(self.graph)
    }
}

impl<'a, Message> EdittingGraphEdge<'a, Message> {
    pub fn moving(&self, dir: Direction, to: impl Into<IdStr>) -> ResultWithRef<Self, (), Error> {
        self.graph
            .edge_plug_edit(&self.edge, dir, to.into())
            .with(self)
    }

    //TODO fn edit to edit other eg. node
}

//@ tree builder Mode ─────────────────────────────────────────────────────────────────────────────

impl<Message> Mode<Message> for GTreeBuilderElement<Message> {
    type Interface<'a> = EdittingGraphUseTreeBuilder<'a, Message>;

    fn interface(self, g: &dyn GraphEditManyMethod<Message = Message>) -> Self::Interface<'_> {
        let inner = g;
        // let phantom_data = PhantomData;
        EdittingGraphUseTreeBuilder {
            node: self,
            graph: inner,
        }
    }
}
// ─────────────────────────────────────────────────────────────────────────────

pub struct EdittingGraphUseTreeBuilder<'a, Message>
where
    Message: 'static,
{
    node: GTreeBuilderElement<Message>,
    graph: &'a dyn GraphEditManyMethod<Message = Message>,
    // pub(crate) phantom_data: PhantomData<M>,
}

impl<'a, Message> GraphEdit for EdittingGraphUseTreeBuilder<'a, Message>
where
    Message: 'static,
{
    type Message = Message;

    /// &self  will drop , edge drop. return new editor interface
    fn edit<M: Mode<Message>>(&self, mode: M) -> M::Interface<'_> {
        mode.interface(self.graph)
    }
}

impl<'a, Message> EdittingGraphUseTreeBuilder<'a, Message> {
    #[topo::nested]
    pub fn insert(&self, to: impl Into<IdStr>) -> ResultWithRef<Self, (), Error> {
        let _span = debug_span!("editor", action = "insert").entered();

        self.graph.insert_node_in_topo(&self.node, to.into());
        Ok(()).with(self)
        // self.graph
        // self.graph
        //     .edge_plug_edit(&self.edge, dir, to.into())
        //     .with(self)
    }
}
