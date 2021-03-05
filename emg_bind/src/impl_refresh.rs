/*
 * @Author: Rais
 * @Date: 2021-02-19 16:16:22
 * @LastEditTime: 2021-03-04 19:30:12
 * @LastEditors: Rais
 * @Description:
 */
use anchors::expert::Anchor;
use anchors::expert::Var;
use anchors::singlethread::Engine;
use std::{
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
};

use crate::{GElement, GElement::*, RefreshFor, RefreshUseFor, Refresher, RefresherFor};
// ────────────────────────────────────────────────────────────────────────────────
// @ impl RefreshUseFor────────────────────────────────────────────────────────────────────────────────

impl<Who> RefreshUseFor<Who> for Who {
    #[inline]
    default fn refresh_use(&mut self, updater: &dyn RefreshFor<Who>) {
        updater.refresh_for(self);
    }
}

// impl<Who> RefreshUseFor<Who> for AnchorWithUpdater<Who>
// where
//     Who: std::clone::Clone + GeneralRefreshFor,
// {
//     fn refresh_use(&mut self, updater: &dyn RefreshFor<Who>) {
//         let mut v = self.get();
//         updater.refresh_for(&mut v);
//         self.get_setter().set(v);
//     }
// }

// @ impl RefreshFor────────────────────────────────────────────────────────────────────────────────
pub auto trait GeneralRefreshFor {}
impl<Who> !GeneralRefreshFor for Var<Who, Engine> {}
impl<Use, E> GeneralRefreshFor for Anchor<Use, E> where E: anchors::expert::Engine + ?Sized {}
// ────────────────────────────────────────────────────────────────────────────────

impl<Who: 'static, Use> RefreshFor<Var<Who, Engine>> for Use
where
    Use: GeneralRefreshFor + RefreshFor<Who> + std::clone::Clone,
    Who: std::clone::Clone,
{
    fn refresh_for(&self, who: &mut Var<Who, Engine>) {
        log::debug!("==========refresh_for Var");
        let mut w = who.get().deref().clone();
        // let mut w = (*who.get()).clone();
        self.refresh_for(&mut w);
        who.set(w);

        // who.refresh_use(self);
    }
}
// ────────────────────────────────────────────────────────────────────────────────

impl<Who, Use> RefreshFor<Who> for Var<Use, Engine>
where
    Who: GeneralRefreshFor,
    Use: RefreshFor<Who> + Clone + 'static,
{
    fn refresh_for(&self, who: &mut Who) {
        // self.get().refresh_for(who);
        who.refresh_use(self.get().deref());
    }
}
// ────────────────────────────────────────────────────────────────────────────────

impl<Who: 'static, Use: 'static> RefreshFor<Var<Who, Engine>> for Var<Use, Engine>
where
    Use: RefreshFor<Who> + std::clone::Clone,
    Who: std::clone::Clone,
{
    fn refresh_for(&self, who: &mut Var<Who, Engine>) {
        let v = self.get();
        let mut w = who.get().deref().clone();
        // let mut w = (*who.get()).clone();
        w.refresh_use(v.deref());

        who.set(w);
        // who.refresh_use(self);
    }
}
// ────────────────────────────────────────────────────────────────────────────────

// impl<Who> RefreshFor<Who> for RefresherForSelf<Who> {
//     fn refresh_for(&self, who: &mut Who) {
//         self.get()(who);
//     }
// }
impl<Who> RefreshFor<Who> for RefresherFor<Who> {
    fn refresh_for(&self, who: &mut Who) {
        self.get()(who);
    }
}

impl<'a, Who, Use> RefreshFor<Who> for Refresher<'a, Use>
where
    Use: RefreshFor<Who>,
{
    fn refresh_for(&self, who: &mut Who) {
        // self.get()().refresh_for(who);
        who.refresh_use(&self.get());
    }
}

// ────────────────────────────────────────────────────────────────────────────────

impl<Who, Use> RefreshFor<Who> for Anchor<Use, Engine>
where
    Who: GeneralRefreshFor,
    Use: RefreshFor<Who> + Clone + 'static,
{
    fn refresh_for(&self, who: &mut Who) {
        crate::ENGINE.with(|e| {
            let u = e.borrow_mut().get(self);
            who.refresh_use(&u);
        })
    }
}

// ────────────────────────────────────────────────────────────────────────────────

impl<'a, Message> RefreshFor<GElement<'a, Message>> for GElement<'a, Message>
where
    Message: 'static + Clone,
{
    fn refresh_for(&self, el: &mut GElement<'a, Message>) {
        match (el, self) {
            //任何 el 刷新, 包括 el=refresher
            //refreshing use any impl RefreshFor
            (el, Refresher_(refresher)) => {
                log::debug!("{} refresh use refresher", el);
                el.refresh_use(refresher.deref());
            }
            // layer 包裹 任何除了refresher的el
            (Layer_(l), any_not_refresher) => {
                log::debug!("layer refresh use {} (do push)", any_not_refresher);
                l.try_ref_push(any_not_refresher.clone());
            }
            // refresher 不与任何不是 refresher 的 el 产生刷新动作
            (Refresher_(_), _any_not_refresher) => {
                panic!(
                    "refresh for ( Refresher_ ) use ( {} ) is not supported",
                    _any_not_refresher
                )
            }
            (not_layer_or_refresher, b) => {
                panic!(
                    "refresh for ( {} ) use ( {} ) - that is not supported",
                    not_layer_or_refresher, b
                )
            }
        }
    }
}

/// for Refresher<Use> many type
impl<'a, Message> RefreshFor<GElement<'a, Message>> for i32 {
    fn refresh_for(&self, el: &mut GElement<'a, Message>) {
        match el {
            Layer_(_layer) => {
                log::debug!("layer update use i32");
            }

            Text_(text) => {
                log::info!("==========Text update use i32");
                text.content(format!("i32:{}", self));
            }
            Refresher_(_) => {
                log::debug!("Updater update use i32");
            }
            Element_(_) => {
                log::debug!("Element_ update use i32");
            }
        }
    }
}
