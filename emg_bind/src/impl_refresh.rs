/*
 * @Author: Rais
 * @Date: 2021-02-19 16:16:22
 * @LastEditTime: 2022-06-22 10:06:24
 * @LastEditors: Rais
 * @Description:
 */
use crate::{GElement, NodeBuilderWidget};
use emg_core::IdStr;
use emg_refresh::{
    EqRefreshFor, RefreshFor, RefreshForUse, RefreshUseNoWarper, RefreshWhoNoWarper,
};
use tracing::{trace, warn};

// ────────────────────────────────────────────────────────────────────────────────

impl<Message> RefreshWhoNoWarper for GElement<Message> {}
impl<Message> RefreshUseNoWarper for GElement<Message> {}
// impl<Message: PartialEq + Clone + 'static> EqRefreshFor<Self> for GElement<Message> {}
impl<Message> RefreshFor<Self> for GElement<Message>
where
    Message: Clone,
{
    fn refresh_for(&self, el: &mut Self) {
        use GElement::{Builder_, Event_, Generic_, Layer_, Refresher_, Text_};
        //TODO for builder
        //TODO allways check when add GElement number;

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
            //TODO event 只和 builder 起作用
            (_gel, _g_event_callback @ Event_(_)) => {
                // gel.try_convert_into_gelement_node_builder_widget_().expect("can't convert to NodeBuilderWidget,Allowing this can cause performance problems")
                // .refresh_use(g_event_callback)
                panic!("should never directly use event_callback for GElement");
            }

            //其他任何 el 刷新, 包括 el=refresher
            //refreshing use any impl RefreshFor
            (gel, Refresher_(refresher)) => {
                trace!("{} refresh use refresher", gel);
                gel.refresh_for_use(refresher.as_ref() as &dyn RefreshFor<Self>);
            }
            // TODO: do not many clone event_callback

            // layer 包裹 任何除了refresher的el
            (Layer_(l), any_not_refresher_event) => {
                trace!("layer refresh use {} (do push)", any_not_refresher_event);
                l.push(any_not_refresher_event.clone());
            }
            // refresher 不与任何不是 refresher 的 el 产生刷新动作
            (Refresher_(_), any_not_refresher_event) => {
                panic!(
                    "refresh for ( Refresher_ ) use ( {} ) is not supported",
                    any_not_refresher_event
                );
            }
            (Text_(who), Text_(us_it)) => {
                who.set_content(us_it.get_content());
            }
            (who, Builder_(builder)) => {
                builder.widget().unwrap().refresh_for(who);
            }

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
// impl DynPartialEq for u32 {}
// TODO : check no need? because already impl<Who, Use> EqRefreshFor<Who> for Use
impl<Message> EqRefreshFor<GElement<Message>> for u32 {}
impl<Message> RefreshFor<GElement<Message>> for u32 {
    fn refresh_for(&self, el: &mut GElement<Message>) {
        use GElement::Text_;

        match el {
            Text_(text) => {
                trace!("==========Text update use u32");
                text.set_content(format!("u32:{}", self));
            }

            other => {
                trace!("====> {} refreshing use u32", other);
            }
        }
    }
}

impl<Message> EqRefreshFor<GElement<Message>> for i32 {}

impl<Message> RefreshFor<GElement<Message>> for i32 {
    fn refresh_for(&self, el: &mut GElement<Message>) {
        use GElement::Text_;

        match el {
            Text_(text) => {
                trace!("==========Text update use i32");
                text.set_content(format!("i32:{}", self));
            }
            GElement::Builder_(_) => todo!(),
            GElement::Layer_(_) => todo!(),
            GElement::Button_(_) => todo!(),
            GElement::Refresher_(_) => todo!(),
            GElement::Event_(_) => todo!(),
            GElement::Generic_(x) => {
                todo!()
            }
            GElement::NodeRef_(_) => todo!(),
            GElement::SaNode_(_) => todo!(),
            GElement::EvolutionaryFactor(_) => todo!(),
            GElement::EmptyNeverUse => todo!(), // other => {
                                                //     warn!("====> {} refreshing use i32,no effect", other);
                                                // }
        }
    }
}

impl<Message> RefreshFor<GElement<Message>> for f64 {
    fn refresh_for(&self, el: &mut GElement<Message>) {
        use GElement::Text_;

        match el {
            Text_(text) => {
                trace!("==========Text update use f64");
                text.set_content(format!("f64:{}", self));
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

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use emg_refresh::Refresher;

    use crate::GElement;

    enum Message {
        A,
    }

    #[test]
    fn it_works() {
        let _f = GElement::<Message>::Refresher_(Rc::new(Refresher::new(|| 1i32)));
        // let ff: Rc<dyn EqRefreshFor<GElement<Message>>> = f;
        // Rc<dyn EqRefreshFor<GElement<Message>>>, found Rc<Refresher<u32>>
    }
}
