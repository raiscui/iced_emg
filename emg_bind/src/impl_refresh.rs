/*
 * @Author: Rais
 * @Date: 2021-02-19 16:16:22
 * @LastEditTime: 2021-08-18 19:15:31
 * @LastEditors: Rais
 * @Description:
 */
use crate::{
    GElement,
    GElement::{Event_, Layer_, Refresher_, Text_},
    NodeBuilderWidget,
};
use emg_refresh::{RefreshFor, RefreshUseFor, RefreshWhoNoWarper};
use tracing::{trace, warn};

// ────────────────────────────────────────────────────────────────────────────────

impl<'a, Message> RefreshWhoNoWarper for GElement<'a, Message> {}
impl<'a, Message> RefreshFor<GElement<'a, Message>> for GElement<'a, Message>
where
    Message: 'static + Clone,
{
    fn refresh_for(&self, el: &mut GElement<'a, Message>) {
        match (el, self) {
            // @ Single explicit match
            (_gel, _g_event_callback @ Event_(_)) => {
                // gel.try_convert_into_gelement_node_builder_widget_().expect("can't convert to NodeBuilderWidget,Allowing this can cause performance problems")
                // .refresh_use(g_event_callback)
                panic!("should never directly use event_callback for GElement");
            }

            //其他任何 el 刷新, 包括 el=refresher
            //refreshing use any impl RefreshFor
            (gel, Refresher_(refresher)) => {
                trace!("{} refresh use refresher", gel);
                gel.refresh_use(&**refresher);
            }
            // TODO: do not many clone event_callback

            // layer 包裹 任何除了refresher的el
            (Layer_(l), any_not_refresher) => {
                trace!("layer refresh use {} (do push)", any_not_refresher);
                l.try_ref_push(any_not_refresher.clone());
            }
            // refresher 不与任何不是 refresher 的 el 产生刷新动作
            (Refresher_(_), any_not_refresher) => {
                panic!(
                    "refresh for ( Refresher_ ) use ( {} ) is not supported",
                    any_not_refresher
                );
            }

            // @ any not match ─────────────────────────────────────────────────────────────────

            // TODO : event_callbacks prosess
            // TODO : NodeBuilderWidget prosess
            (not_layer_or_refresher, b) => {
                panic!(
                    "refresh for ( {} ) use ( {} ) - that is not supported",
                    not_layer_or_refresher, b
                );
            }
        }
    }
}

/// `GElement` refresh use X
/// for Refresher<GElement> many type
// this is `GElement` refresh use `i32`
impl<'a, Message> RefreshFor<GElement<'a, Message>> for i32 {
    fn refresh_for(&self, el: &mut GElement<'a, Message>) {
        match el {
            Text_(text) => {
                trace!("==========Text update use i32");
                text.content(format!("i32:{}", self));
            }

            other => {
                trace!("====> {} refreshing use i32", other);
            }
        }
    }
}
/// `GElement` refresh use X
/// for Refresher<GElement> many type
// this is `GElement` refresh use `i32`
impl<'a, Message> RefreshFor<GElement<'a, Message>> for u32 {
    fn refresh_for(&self, el: &mut GElement<'a, Message>) {
        match el {
            Text_(text) => {
                trace!("==========Text update use u32");
                text.content(format!("u32:{}", self));
            }

            other => {
                warn!("not implemented ====> {} refreshing use u32", other);
            }
        }
    }
}
impl<'a, Message> RefreshFor<GElement<'a, Message>> for f64 {
    fn refresh_for(&self, el: &mut GElement<'a, Message>) {
        match el {
            Text_(text) => {
                trace!("==========Text update use f64");
                text.content(format!("f64:{}", self));
            }

            other => {
                warn!("not implemented ====> {} refreshing use f64", other);
            }
        }
    }
}
// ────────────────────────────────────────────────────────────────────────────────

impl<'a, Message> RefreshFor<NodeBuilderWidget<'a, Message>> for GElement<'a, Message>
where
    Message: 'static + Clone,
{
    fn refresh_for(&self, node_builder_widget: &mut NodeBuilderWidget<'a, Message>) {
        trace!("node_builder_widget refresh use GElement (event_callback)");

        match self {
            // @ Clear type match
            Event_(event_callback) => {
                trace!("node_builder_widget.add_event_callback(event_callback.clone()) ");
                node_builder_widget.add_event_callback(event_callback.clone());
            }
            // ─────────────────────────────────────────────────────────────────

            // @ Single explicit match

            //其他任何 el 刷新, 包括 el=refresher
            // TODO impl refresher for NodeBuilderWidget(most edit event_callbacks list )
            // (gel, Refresher_(refresher)) => {
            //     gel.refresh_use(refresher.deref());
            // }

            // @ any not match ─────────────────────────────────────────────────────────────────
            any => {
                panic!(
                    "refresh for ( {} ) use ( {} ) - that is not supported",
                    "not_node_builder_widget", any
                );
            }
        }
    }
}
