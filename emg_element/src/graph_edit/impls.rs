/*
 * @Author: Rais
 * @Date: 2023-01-20 00:02:37
 * @LastEditTime: 2023-01-22 16:47:56
 * @LastEditors: Rais
 * @Description:
 */

use std::marker::PhantomData;

use emg::EdgeIndex;
use emg_state::CloneStateVar;

use crate::GraphType;

use super::{EdittingGraph, GraphEdit, GraphEditManyMethod, Mode};

impl<Message, Ix> GraphEditManyMethod<Ix> for GraphType<Message, Ix>
where
    Ix: std::hash::Hash
        + std::clone::Clone
        + std::cmp::Ord
        + std::default::Default
        + std::fmt::Debug,
{
    fn edge_change_source(&mut self, who: &EdgeIndex<Ix>, to: Ix) {
        let source = self.edge_source(who);
        source.set(Some(to.into()));
    }

    fn edge_path_node_change_edge(&mut self) {
        todo!("edge_path_node_change_edge")
    }
}

impl<Message, Ix> GraphEdit<Ix> for GraphType<Message, Ix>
where
    Self: GraphEditManyMethod<Ix>,
    Ix: std::hash::Hash + std::clone::Clone + std::cmp::Ord + std::default::Default,
{
    fn edit<M: Mode>(&mut self) -> EdittingGraph<Ix, M> {
        EdittingGraph {
            inner: self,
            phantom_data: PhantomData,
        }
    }
}

//test

#[cfg(test)]
#[allow(unused)]
mod test {
    use emg::{edge_index, node_index};
    use emg_common::IdStr;
    use emg_layout::epath;

    use crate::graph_edit::{EdgeMode, ModeInterface};

    use super::*;

    enum A {}

    #[test]
    fn test_edge_path_change_source() {
        let mut g = GraphType::<A, IdStr>::default();
        // let a_c = epath!("a"=>"c");
        let a_c = edge_index::<IdStr>("a", "c");
        // let b_c = epath!("b"=>"c");
        g.edit::<EdgeMode>().move_to(&a_c, "b");
    }
}
