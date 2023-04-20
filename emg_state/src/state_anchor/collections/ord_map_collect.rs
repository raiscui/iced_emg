/*
 * @Author: Rais
 * @Date: 2023-03-31 18:03:25
 * @LastEditTime: 2023-03-31 18:11:00
 * @LastEditors: Rais
 * @Description:
 */

use anchors::{
    expert::{AnchorHandle, AnchorInner, OutputContext, Poll, UpdateContext},
    im::{ordmap, OrdMap},
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
        StateOrdMapCollect::new_to_anchor(iter.into_iter().collect())
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
        StateOrdMapCollect::new_to_anchor(iter.into_iter().cloned().collect())
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
        StateOrdMapCollect::new_to_anchor(
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
    pub fn new_to_anchor(anchors: OrdMap<I, StateAnchor<V>>) -> StateAnchor<OrdMap<I, V>> {
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
        let mut changed = false;

        if self.dirty {
            let polls = self
                .state_anchors
                .iter()
                .try_fold(vec![], |mut acc, (i, anchor)| {
                    let s = ctx.request(anchor.anchor(), true);
                    if s == Poll::Pending {
                        None
                    } else {
                        acc.push((s, (i, anchor)));
                        Some(acc)
                    }
                });

            if polls.is_none() {
                return Poll::Pending;
            }

            self.dirty = false;

            if let Some(ref mut old_vals) = self.vals {
                for (poll, (i, anchor)) in &polls.unwrap() {
                    if &Poll::Updated == poll {
                        old_vals.insert((**i).clone(), ctx.get(anchor.anchor()).clone());
                        changed = true;
                    }
                }
            } else {
                // self.vals = Some(
                //     self.anchors
                //         .iter()
                //         .map(|(i, anchor)| (i.clone(), ctx.get(anchor).clone()))
                //         .collect(),
                // );
                // changed = true;

                let pool = ordmap::OrdMapPool::new(self.state_anchors.len());
                let mut dict = OrdMap::with_pool(&pool);

                self.state_anchors
                    .iter()
                    .map(|(i, anchor)| (i.clone(), ctx.get(anchor.anchor()).clone()))
                    .collect_into(&mut dict);

                self.vals = Some(dict);
                changed = true;
            }
        }

        if changed {
            Poll::Updated
        } else {
            Poll::Unchanged
        }
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
