use std::ops::Deref;

use crate::{GElement, GElement::*, RefreshFor, RefreshUseFor};

/*
 * @Author: Rais
 * @Date: 2021-02-19 16:16:22
 * @LastEditTime: 2021-02-20 14:54:13
 * @LastEditors: Rais
 * @Description:
 */
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
