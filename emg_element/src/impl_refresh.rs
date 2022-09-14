/*
 * @Author: Rais
 * @Date: 2022-08-22 16:28:40
 * @LastEditTime: 2022-09-14 22:08:08
 * @LastEditors: Rais
 * @Description:
 */
use std::{any::Any, rc::Rc};

/*
 * @Author: Rais
 * @Date: 2021-02-19 16:16:22
 * @LastEditTime: 2022-08-10 15:31:34
 * @LastEditors: Rais
 * @Description:
 */
use crate::{GElement, NodeBuilderWidget};
use emg_shaping::{
    EqShaping, ShapeOfUse, Shaping, ShapingUse, ShapingUseNoWarper, ShapingWhoNoWarper,
};
use tracing::{trace, warn};

// ────────────────────────────────────────────────────────────────────────────────

impl<Message, RenderCtx> ShapingWhoNoWarper for GElement<Message, RenderCtx> {}
impl<Message, RenderCtx> ShapingUseNoWarper for GElement<Message, RenderCtx> {}
// impl<Message, RenderContext: PartialEq + Clone + 'static> EqShaping<Self> for GElement<Message, RenderContext> {}
impl<Message, RenderCtx> Shaping<Self> for GElement<Message, RenderCtx>
// where
//     Message: Clone,
where
    Message: 'static,
    RenderCtx: 'static,
{
    fn shaping(&self, who_el: &mut Self) {
        use GElement::{Builder_, Generic_, Layer_, Shaper_};
        //TODO for builder
        //TODO allways check when add GElement number;

        match (who_el, self) {
            (who, Generic_(use_something)) => {
                use_something.shaping(who);
                // let something = use_something.as_refresh_for();
                // who.shaping_use(use_something);
            }
            (Generic_(who), use_something) => {
                let dyn_ref = who.as_mut();
                // use_something.shaping(who);
                dyn_ref.shaping_use(use_something);
            }

            // @ Single explicit match
            //TODO event 只和 builder 起作用
            // (_gel, _g_event_callback @ Event_(_)) => {
            //     // gel.try_convert_into_gelement_node_builder_widget_().expect("can't convert to NodeBuilderWidget,Allowing this can cause performance problems")
            //     // .shaping_use(g_event_callback)
            //     panic!("should never directly use event_callback for GElement");
            // }

            //其他任何 el 刷新, 包括 el=shaper
            //refreshing use any impl Shaping
            (gel, Shaper_(shaper)) => {
                trace!("{} refresh use shaper", gel);
                gel.shape_of_use(shaper.as_ref() as &dyn Shaping<Self>);
            }
            // TODO: do not many clone event_callback

            // layer 包裹 任何除了shaper的el
            (Layer_(l), any_not_shaper_event) => {
                trace!("layer refresh use {} (do push)", any_not_shaper_event);
                l.push(any_not_shaper_event.clone());
            }
            // shaper 不与任何不是 shaper 的 el 产生刷新动作
            (Shaper_(_who), any_not_shaper_event) => {
                panic!(
                    "refresh for ( Refresher_ ) use ( {} ) is not supported",
                    any_not_shaper_event
                );
            }
            // (Text_(who), Text_(us_it)) => {
            //     who.set_content(us_it.get_content());
            // }
            (who, Builder_(builder)) => {
                builder.widget().shaping(who);
            }

            // TODO : event_callbacks prosess
            // TODO : NodeBuilderWidget prosess
            (not_layer_or_shaper, b) => {
                panic!(
                    "refresh for ( {} ) use ( {} ) - that is not supported",
                    not_layer_or_shaper, b
                );
            }
        }
    }
}

/// `GElement` refresh use X
/// for Shaper<GElement> many type
// this is `GElement` refresh use `i32`
// impl DynPartialEq for u32 {}
// TODO : check no need? because already impl<Who, Use> EqShaping<Who> for Use
impl<Message, RenderContext> EqShaping<GElement<Message, RenderContext>> for u32 {}
impl<Message, RenderContext> Shaping<GElement<Message, RenderContext>> for u32 {
    fn shaping(&self, el: &mut GElement<Message, RenderContext>) {
        // use GElement::Text_;

        match el {
            // Text_(text) => {
            //     trace!("==========Text update use u32");
            //     text.set_content(format!("u32:{}", self));
            // }
            other => {
                warn!("====> {} refreshing use u32,no effect", other);
            }
        }
    }
}

impl<Message, RenderContext> Shaping<GElement<Message, RenderContext>> for i32
where
    Message: 'static,
{
    #[allow(clippy::match_same_arms)]
    fn shaping(&self, el: &mut GElement<Message, RenderContext>) {
        match el {
            // Text_(text) => {
            //     trace!("==========Text update use i32");
            //     text.set_content(format!("i32:{}", self));
            // }
            GElement::Builder_(_) => todo!(),
            GElement::Layer_(_) => todo!(),
            // GElement::Button_(_) => todo!(),
            GElement::Shaper_(_) => todo!(),
            GElement::Event_(_) => todo!(),
            GElement::Generic_(w) => {
                warn!("i32 try_refresh_for Generic_");

                // self.try_refresh_for(x);
                // w.try_shaping_use(Box::new(*self));
                (**w).shaping_use(self);

                // w.shape_of_use(self);
            }
            GElement::NodeRef_(_) => todo!(),
            GElement::SaNode_(_) => todo!(),
            GElement::EvolutionaryFactor(_) => todo!(),
            GElement::EmptyNeverUse => todo!(),
            // other => {
            //     warn!("====> {} refreshing use i32,no effect", other);
            // }
        }
    }
}

impl<Message, RenderContext> Shaping<GElement<Message, RenderContext>> for f64 {
    fn shaping(&self, el: &mut GElement<Message, RenderContext>) {
        match el {
            // Text_(text) => {
            //     trace!("==========Text update use f64");
            //     text.set_content(format!("f64:{}", self));
            // }
            other => {
                warn!("not implemented ====> {} refreshing use f64", other);
            }
        }
    }
}
// ────────────────────────────────────────────────────────────────────────────────

impl<Message, RenderContext> Shaping<NodeBuilderWidget<Message, RenderContext>>
    for GElement<Message, RenderContext>
where
    Message: 'static,
    RenderContext: 'static,
{
    fn shaping(&self, node_builder_widget: &mut NodeBuilderWidget<Message, RenderContext>) {
        trace!("node_builder_widget refresh use GElement (event_callback)");

        match self {
            // @ Clear type match
            Self::Event_(event_callback) => {
                trace!("node_builder_widget.add_event_callback(event_callback.clone()) ");
                node_builder_widget.add_event_callback(event_callback.clone());
            }
            // ─────────────────────────────────────────────────────────────────

            // @ Single explicit match

            //其他任何 el 刷新, 包括 el=shaper
            // TODO impl shaper for NodeBuilderWidget(most edit event_callbacks list )
            // (gel, Refresher_(shaper)) => {
            //     gel.shaping_use(shaper.deref());
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

    use emg_piet_gpu::RenderCtx;
    use emg_shaping::Shaper;

    use crate::GElement;

    enum Message {
        A,
    }

    #[test]
    fn it_works() {
        let _f = GElement::<Message, RenderCtx>::Shaper_(Rc::new(Shaper::new(|| 1i32)));
        // let ff: Rc<dyn EqShaping<GElement<Message>>> = f;
        // Rc<dyn EqShaping<GElement<Message>>>, found Rc<Shaper<u32>>
    }
}
