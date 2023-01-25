/*
 * @Author: Rais
 * @Date: 2023-01-19 17:43:32
 * @LastEditTime: 2023-01-23 01:31:43
 * @LastEditors: Rais
 * @Description:
 */

mod impls;
use std::marker::PhantomData;

use emg::EdgeIndex;

// ─────────────────────────────────────────────────────────────────────────────
// I :实例?
pub struct EdgeMode<I = ()>(PhantomData<I>);

trait Mode {
    type FromIndex<Ix>;
    type ToIndex<Ix>;
    fn move_to<Ix, G>(g: &G, who: &Self::FromIndex<Ix>, to: Self::ToIndex<Ix>)
    where
        G: GraphEditManyMethod<Ix> + ?Sized;
}

impl Mode for EdgeMode {
    type FromIndex<Ix> = EdgeIndex<Ix>;
    type ToIndex<Ix> = Ix;
    fn move_to<Ix, G>(g: &G, who: &Self::FromIndex<Ix>, to: Self::ToIndex<Ix>)
    where
        G: GraphEditManyMethod<Ix> + ?Sized,
    {
        g.edge_change_source(who, to);
    }
}

trait ModeInterface<Ix> {
    type FromIndex;
    type ToIndex;
    fn move_to<T: Into<Self::ToIndex>>(&self, who: &Self::FromIndex, to: T);
}

struct EdittingGraph<'a, Ix, M> {
    inner: &'a mut dyn GraphEditManyMethod<Ix>,
    phantom_data: PhantomData<M>,
}

impl<'a, Ix, M> ModeInterface<Ix> for EdittingGraph<'a, Ix, M>
where
    M: Mode,
{
    type FromIndex = M::FromIndex<Ix>;

    type ToIndex = M::ToIndex<Ix>;

    fn move_to<T: Into<M::ToIndex<Ix>>>(&self, who: &M::FromIndex<Ix>, to: T) {
        M::move_to(self.inner, who, to.into());
    }
}

trait GraphEdit<Ix> {
    fn edit<M: Mode>(&mut self) -> EdittingGraph<Ix, M>;
}

trait GraphEditManyMethod<Ix> {
    //实例连源枝移动( 某 path edge 原 edge 更改 source node) ,枝上其他node 不动(clone edge?)
    fn edge_change_source(&self, who: &EdgeIndex<Ix>, to: Ix);

    //实例嫁接(实例不连源枝移动 , 某 path node 原 edge 断开, xin edge 接上)
    fn edge_path_node_change_edge(&mut self);
}
