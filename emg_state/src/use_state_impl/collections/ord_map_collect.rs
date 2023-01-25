/*
 * @Author: Rais
 * @Date: 2023-01-25 18:39:47
 * @LastEditTime: 2023-01-25 19:45:06
 * @LastEditors: Rais
 * @Description:
 */
/*
 * @Author: Rais
 * @Date: 2022-09-14 11:08:53
 * @LastEditTime: 2023-01-25 15:14:17
 * @LastEditors: Rais
 * @Description:
 */

use anchors::{
    expert::{AnchorHandle, AnchorInner, OutputContext, Poll, UpdateContext},
    im::OrdMap,
    singlethread::Engine,
};
use std::panic::Location;

use crate::StateAnchor;

impl<I, V> std::iter::FromIterator<(I, StateAnchor<V>)> for StateAnchor<OrdMap<I, V>>
where
    V: std::clone::Clone + 'static,
    I: 'static + Clone + std::cmp::Ord,

    OrdMap<I, V>: std::cmp::Eq,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (I, StateAnchor<V>)>,
    {
        StateOrdMapCollect::new(iter.into_iter().collect())
    }
}

impl<'a, I, V> std::iter::FromIterator<&'a (I, StateAnchor<V>)> for StateAnchor<OrdMap<I, V>>
where
    V: std::clone::Clone + 'static,
    I: 'static + Clone + std::cmp::Ord,

    OrdMap<I, V>: std::cmp::Eq,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a (I, StateAnchor<V>)>,
    {
        StateOrdMapCollect::new(iter.into_iter().cloned().collect())
    }
}

impl<'a, I, V> std::iter::FromIterator<(&'a I, &'a StateAnchor<V>)> for StateAnchor<OrdMap<I, V>>
where
    V: std::clone::Clone + 'static,
    I: 'static + Clone + std::cmp::Ord,

    OrdMap<I, V>: std::cmp::Eq,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (&'a I, &'a StateAnchor<V>)>,
    {
        StateOrdMapCollect::new(
            iter.into_iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        )
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct StateOrdMapCollect<I, V> {
    state_anchors: OrdMap<I, StateAnchor<V>>,
    vals: Option<OrdMap<I, V>>,
    dirty: bool,
    location: &'static Location<'static>,
}

impl<I: 'static + Clone + std::cmp::Ord, V> StateOrdMapCollect<I, V>
where
    V: std::clone::Clone + 'static,
    //TODO 通过特化 制作PartialEq版本, (im_rc 在 PartialEq 的情况下 对比 ,没有Eq 性能高)
    //TODO 制作 vector smallvec 的对比版本
    OrdMap<I, V>: std::cmp::Eq,
{
    #[allow(clippy::new_ret_no_self)]
    #[track_caller]
    pub fn new(anchors: OrdMap<I, StateAnchor<V>>) -> StateAnchor<OrdMap<I, V>> {
        StateAnchor(<Engine as anchors::expert::Engine>::mount(Self {
            state_anchors: anchors,
            vals: None,
            dirty: true,
            location: Location::caller(),
        }))
    }
}

impl<I, V> AnchorInner<anchors::singlethread::Engine> for StateOrdMapCollect<I, V>
where
    V: std::clone::Clone + 'static,
    I: 'static + Clone + std::cmp::Ord,

    OrdMap<I, V>: std::cmp::Eq,
{
    type Output = OrdMap<I, V>;
    fn dirty(
        &mut self,
        _edge: &<<anchors::singlethread::Engine as anchors::expert::Engine>::AnchorHandle as AnchorHandle>::Token,
    ) {
        // self.vals = None;
        self.dirty = true;
    }

    fn poll_updated<G: UpdateContext<Engine = anchors::singlethread::Engine>>(
        &mut self,
        ctx: &mut G,
    ) -> Poll {
        if self.dirty {
            let pending_exists = self.state_anchors.iter().any(|(_i, state_anchor)| {
                ctx.request(state_anchor.anchor(), true) == Poll::Pending
            });
            if pending_exists {
                return Poll::Pending;
            }
            let new_vals: Option<OrdMap<I, V>> = Some(
                self.state_anchors
                    .iter()
                    .map(|(i, state_anchor)| (i.clone(), ctx.get(state_anchor.anchor()).clone()))
                    .collect(),
            );

            if self.vals != new_vals {
                self.vals = new_vals;
                return Poll::Updated;
            }
        }
        self.dirty = false;
        Poll::Unchanged
    }

    fn output<'slf, 'out, G: OutputContext<'out, Engine = Engine>>(
        &'slf self,
        _ctx: &mut G,
    ) -> &'out Self::Output
    where
        'slf: 'out,
    {
        self.vals.as_ref().unwrap()
    }

    fn debug_location(&self) -> Option<(&'static str, &'static Location<'static>)> {
        Some(("DictCollect", self.location))
    }
}

#[cfg(test)]
mod test {

    use crate::{dict, use_state, CloneStateAnchor, CloneStateVar, StateAnchor};
    use anchors::{collections::ord_map_methods::Dict, singlethread::*};
    #[test]
    fn collect() {
        let a = use_state(1);
        let b = use_state(2);
        let c = use_state(5);
        let bcut = {
            let mut old_num_opt: Option<usize> = None;
            b.watch().cutoff(move |num| {
                if let Some(old_num) = old_num_opt {
                    if old_num == *num {
                        return false;
                    }
                }
                old_num_opt = Some(*num);
                true
            })
        };

        let bw = bcut.map(|v| {
            println!("b change");
            *v
        });
        let f = dict!(1usize=>a.watch(),2usize=>b.watch(),3usize=>c.watch());
        let nums: StateAnchor<Dict<_, _>> = (&f).into_iter().collect();
        let sum: StateAnchor<usize> = nums.map(|nums| nums.values().sum());
        let ns: StateAnchor<usize> = nums.map(|nums: &Dict<_, _>| nums.len());

        assert_eq!(sum.get(), 8);

        a.set(2);
        assert_eq!(sum.get(), 9);

        c.set(1);
        assert_eq!(sum.get(), 5);
        println!("ns {}", ns.get());
        b.set(9);
        println!("after b set: {}", sum.get()); // [2,1,9]
        assert_eq!(sum.get(), 12);

        b.set(9);
        println!("after b set2: {}", sum.get());
        assert_eq!(sum.get(), 12);
    }
}
