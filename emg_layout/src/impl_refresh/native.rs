/*
 * @Author: Rais
 * @Date: 2022-08-29 23:19:00
 * @LastEditTime: 2022-08-31 16:38:11
 * @LastEditors: Rais
 * @Description:
 */

use std::rc::Rc;

use crate::EmgEdgeItem;
use emg_common::TypeCheck;
use emg_refresh::{RefreshFor, RefreshWhoNoWarper};
use emg_state::CloneStateVar;
use seed_styles::{CssBorderColor, CssBorderWidth, CssFill};

// impl<Ix, RenderCtx> RefreshFor<EmgEdgeItem<Ix, RenderCtx>> for CssFill
// where
//     Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
//     EmgEdgeItem<Ix, RenderCtx>: RefreshWhoNoWarper,
//     RenderCtx: 'static,
// {
//     fn refresh_for(&self, who: &mut EmgEdgeItem<Ix, RenderCtx>) {
//         let type_name = Self::TYPE_NAME;
//         who.styles.update(|s| {
//             s.insert(type_name, Rc::new(self.clone()));
//         });
//     }
// }

macro_rules! impl_css_native_refresh {
    ($css:ident) => {
        impl<Ix, RenderCtx> RefreshFor<EmgEdgeItem<Ix, RenderCtx>> for $css
        where
            Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
            EmgEdgeItem<Ix, RenderCtx>: RefreshWhoNoWarper,
            RenderCtx: 'static,
        {
            fn refresh_for(&self, who: &mut EmgEdgeItem<Ix, RenderCtx>) {
                let type_name = Self::TYPE_NAME;
                who.styles.update(|s| {
                    s.insert(type_name, Rc::new(self.clone()));
                });
            }
        }
    };
}

impl_css_native_refresh!(CssFill);
impl_css_native_refresh!(CssBorderWidth);
impl_css_native_refresh!(CssBorderColor);
