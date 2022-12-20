/*
 * @Author: Rais
 * @Date: 2022-08-18 18:01:09
 * @LastEditTime: 2022-09-19 10:13:47
 * @LastEditors: Rais
 * @Description:
 */

// mod index;

#[cfg(all(feature = "old_node"))]
mod node_item_rc_sv;
#[cfg(all(feature = "old_node"))]
pub use node_item_rc_sv::{GelType, GraphType, NItem, E, N};

#[cfg(all(feature = "new_node"))]
mod node_item_rc_sv_with_ctx;
#[cfg(all(feature = "new_node"))]
pub use node_item_rc_sv_with_ctx::GraphMethods;
#[cfg(all(feature = "new_node"))]
pub use node_item_rc_sv_with_ctx::{EventMatchsSa, GelType, GraphType, NItem, E, N};

use emg_common::IdStr;
use emg_layout::EPath;
use emg_state::{Dict, StateAnchor};

const POOL_SIZE: usize = 1;

// pub type GelType<Message> = GElement<Message>;
// pub type NItem<Message> = StateAnchor<GelType<Message>>;
// pub type N<Message, Ix> = EmgNodeItem<NItem<Message>, GelType<Message>, Ix>;
// pub type E<Ix> = EmgEdgeItem<Ix>;
// pub type GraphType<Message, Ix = IdStr> = Graph<N<Message, Ix>, E<Ix>, Ix>;

type PathDict<Ix> = Dict<EPath<Ix>, ()>;

// type CurrentPathChildrenEixGElSA<Message> =
// StateAnchor<(EdgeIndex<IdStr>, Either<GelType<Message>, GelType<Message>>)>;

// type GElEither<Message> = Either<GelType<Message>, GelType<Message>>;

pub struct EmgNodeItem<NItem, GelType, Ix = IdStr>
where
    // Message: 'static + Clone + std::cmp::PartialEq,
    Ix: std::clone::Clone + std::hash::Hash + std::cmp::Eq + std::default::Default,
    // Dict<EPath<Ix>, EmgNodeItem<Message, Ix>>: PartialEq,
{
    gel_sa: NItem,
    //TODO maybe indexSet
    // paths_sa: StateAnchor<Vector<EPath<Ix>>>, //NOTE: has self
    paths_sa: StateAnchor<PathDict<Ix>>, //NOTE: has self
    // incoming_eix_sa: StateAnchor<NodeEdgeCollect<Ix>>,
    // outgoing_eix_sa: StateAnchor<NodeEdgeCollect<Ix>>,
    paths_view_gel: StateAnchor<Dict<EPath<Ix>, GelType>>,
    paths_view_gel_sa: StateAnchor<Dict<EPath<Ix>, StateAnchor<GelType>>>,
}

impl<NItem, GelType, Ix> Clone for EmgNodeItem<NItem, GelType, Ix>
where
    // Message: 'static + Clone + std::cmp::PartialEq,
    Ix: std::clone::Clone + std::hash::Hash + std::cmp::Eq + std::default::Default,
    NItem: std::clone::Clone,
{
    fn clone(&self) -> Self {
        Self {
            gel_sa: self.gel_sa.clone(),
            paths_sa: self.paths_sa.clone(),
            paths_view_gel_sa: self.paths_view_gel_sa.clone(),
            paths_view_gel: self.paths_view_gel.clone(),
        }
    }
}
