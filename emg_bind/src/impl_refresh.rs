/*
 * @Author: Rais
 * @Date: 2021-02-19 16:16:22
 * @LastEditTime: 2021-03-13 16:31:24
 * @LastEditors: Rais
 * @Description:
 */
use anchors::expert::Anchor;
use anchors::expert::Var;
use anchors::singlethread::Engine;
use std::ops::Deref;

use crate::{
    GElement,
    GElement::{Event_, Layer_, Refresher_, Text_},
    NodeBuilderWidget, RefreshFor, RefreshUseFor, Refresher, RefresherFor,
};
// ────────────────────────────────────────────────────────────────────────────────
// @ impl RefreshUseFor────────────────────────────────────────────────────────────────────────────────

impl<Who> RefreshUseFor<Self> for Who {
    #[inline]
    default fn refresh_use(&mut self, updater: &dyn RefreshFor<Self>) {
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
impl<'a, Who> RefreshFor<Who> for RefresherFor<'a, Who> {
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
impl<'a, Message> GeneralRefreshFor for GElement<'a, Message> {}
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
                panic!("should never directly use event_callback for GElement")
            }

            //其他任何 el 刷新, 包括 el=refresher
            //refreshing use any impl RefreshFor
            (gel, Refresher_(refresher)) => {
                log::debug!("{} refresh use refresher", gel);
                gel.refresh_use(refresher.deref());
            }
            // TODO: do not many clone event_callback

            // layer 包裹 任何除了refresher的el
            (Layer_(l), any_not_refresher) => {
                log::debug!("layer refresh use {} (do push)", any_not_refresher);
                l.try_ref_push(any_not_refresher.clone());
            }
            // refresher 不与任何不是 refresher 的 el 产生刷新动作
            (Refresher_(_), any_not_refresher) => {
                panic!(
                    "refresh for ( Refresher_ ) use ( {} ) is not supported",
                    any_not_refresher
                )
            }

            // @ any not match ─────────────────────────────────────────────────────────────────

            // TODO : event_callbacks prosess
            // TODO : NodeBuilderWidget prosess
            (not_layer_or_refresher, b) => {
                panic!(
                    "refresh for ( {} ) use ( {} ) - that is not supported",
                    not_layer_or_refresher, b
                )
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
                log::info!("==========Text update use i32");
                text.content(format!("i32:{}", self));
            }

            other => {
                log::debug!("====> {} refreshing use i32", other);
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
        log::debug!("node_builder_widget refresh use GElement (event_callback)");

        match self {
            // @ Clear type match
            Event_(event_callback) => {
                log::debug!("node_builder_widget.add_event_callback(event_callback.clone()) ");
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
                )
            }
        }
    }
}
