use std::ops::Deref;

use crate::{GElement, GElement::*, RefreshFor, RefreshUseFor};
use anymap::any::CloneAny;

/*
 * @Author: Rais
 * @Date: 2021-02-19 16:16:22
 * @LastEditTime: 2021-02-19 18:59:14
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
            (el, Refresher_(refresher)) => {
                log::debug!("el refresh use refresher");
                el.refresh_use(refresher.deref());
            }
            // layer 包裹 任何除了refresher的el
            (Layer_(l), any_not_refresher) => {
                log::debug!("layer refresh use any_not_refresher (do push)");
                l.try_ref_push(any_not_refresher.clone());
            }
            // refresher 不与任何不是 refresher 的 el 产生刷新动作
            (Refresher_(_), _any_not_refresher) => {
                panic!("Refresher_ refresh_for Refresher_ is not supported")
            }
            _ => {
                panic!("refresh_for this - that not supported")
            } // (Text_(_), Layer_(_)) => {}
              // (Text_(_), Text_(_)) => {}
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
        }
    }
}
