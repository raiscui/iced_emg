/*
 * @Author: Rais
 * @Date: 2023-01-19 17:43:32
 * @LastEditTime: 2023-01-27 16:29:41
 * @LastEditors: Rais
 * @Description:
 */

mod impls;
use std::marker::PhantomData;

use emg::{Direction, EdgeIndex};

// ─────────────────────────────────────────────────────────────────────────────
// I :实例?
pub struct EdgeMode<I = ()>(PhantomData<I>);

trait Mode {
    type Interface<'a, Ix>
    where
        Ix: 'a;

    fn interface<'a, Ix, G>(g: &'a mut G) -> Self::Interface<'a, Ix>
    where
        G: GraphEditManyMethod<Ix = Ix>;
}

impl Mode for EdgeMode {
    type Interface<'a, Ix> = EdittingGraphEdge<'a, Ix, Self> where Ix:'a;

    fn interface<'a, Ix, G>(g: &'a mut G) -> Self::Interface<'a, Ix>
    where
        G: GraphEditManyMethod<Ix = Ix>,
    {
        EdittingGraphEdge {
            inner: g,
            phantom_data: PhantomData,
        }
    }
}

trait GraphEdit {
    type Ix;
    fn edit<M: Mode>(&mut self) -> M::Interface<'_, Self::Ix>;
}
impl<'a, T> GraphEdit for T
where
    T: GraphEditManyMethod,
{
    type Ix = T::Ix;
    fn edit<M: Mode>(&mut self) -> M::Interface<'_, Self::Ix> {
        M::interface(self)
    }
}

trait GraphEditManyMethod {
    type Ix;
    //实例连源枝移动( 某 path edge 原 edge 更改 source node) ,枝上其他node 不动(clone edge?)
    fn edge_plug_edit(&self, who: &EdgeIndex<Self::Ix>, dir: Direction, to: Self::Ix);

    //实例嫁接(实例不连源枝移动 , 某 path node 原 edge 断开, xin edge 接上)
    fn edge_path_node_change_edge(&mut self);
}

struct EdittingGraphEdge<'a, Ix, M> {
    inner: &'a mut dyn GraphEditManyMethod<Ix = Ix>,
    phantom_data: PhantomData<M>,
}

impl<'a, Ix, M> EdittingGraphEdge<'a, Ix, M> {
    fn move_to(&self, who: &EdgeIndex<Ix>, dir: Direction, to: impl Into<Ix>) {
        self.inner.edge_plug_edit(who, dir, to.into());
    }

    //TODO fn edit to edit other eg. node
}
