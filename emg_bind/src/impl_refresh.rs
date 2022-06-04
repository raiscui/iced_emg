use std::ops::Deref;

/*
 * @Author: Rais
 * @Date: 2021-02-19 16:16:22
 * @LastEditTime: 2022-06-01 19:03:54
 * @LastEditors: Rais
 * @Description:
 */
use crate::{GElement, NodeBuilderWidget};
use emg_refresh::{RefreshFor, RefreshForUse, RefreshUseNoWarper, RefreshWhoNoWarper};
use tracing::{trace, warn};

// ────────────────────────────────────────────────────────────────────────────────

impl<Message: PartialEq> RefreshWhoNoWarper for GElement<Message> {}
impl<Message: PartialEq> RefreshUseNoWarper for GElement<Message> {}
impl<Message: PartialEq> RefreshFor<Self> for GElement<Message>
where
    Message: 'static + Clone,
{
    fn refresh_for(&self, el: &mut Self) {
        use GElement::{Builder_, Event_, Generic_, Layer_, Refresher_, Text_};
        //TODO for builder
        match (el, self) {
            (who, Generic_(use_something)) => {
                use_something.refresh_for(who);
                // let something = use_something.as_refresh_for();
                // who.refresh_use(use_something);
            }
            (Generic_(who), use_something) => {
                // let dyn_ref = who.as_ref();
                // use_something.refresh_for(who);
                who.refresh_use(use_something);
            }

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
                gel.refresh_for_use(refresher.as_ref());
            }
            // TODO: do not many clone event_callback

            // layer 包裹 任何除了refresher的el
            (Layer_(l), any_not_refresher_event) => {
                trace!("layer refresh use {} (do push)", any_not_refresher_event);
                l.try_ref_push(any_not_refresher_event.clone());
            }
            // refresher 不与任何不是 refresher 的 el 产生刷新动作
            (Refresher_(_), any_not_refresher_event) => {
                panic!(
                    "refresh for ( Refresher_ ) use ( {} ) is not supported",
                    any_not_refresher_event
                );
            }
            (Text_(who), Text_(us_it)) => {
                who.content(us_it.get_content());
            }
            (who, Builder_(us_gel, _)) => {
                us_gel.deref().refresh_for(who);
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
impl<Message: std::cmp::PartialEq> RefreshFor<GElement<Message>> for i32 {
    fn refresh_for(&self, el: &mut GElement<Message>) {
        use GElement::Text_;

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

impl<Message: std::cmp::PartialEq> RefreshFor<GElement<Message>> for f64 {
    fn refresh_for(&self, el: &mut GElement<Message>) {
        use GElement::Text_;

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

impl<Message> RefreshFor<NodeBuilderWidget<Message>> for GElement<Message>
where
    Message: 'static + Clone + PartialEq,
{
    fn refresh_for(&self, node_builder_widget: &mut NodeBuilderWidget<Message>) {
        use GElement::Event_;
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
