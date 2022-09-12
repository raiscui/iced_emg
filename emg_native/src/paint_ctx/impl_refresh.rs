/*
 * @Author: Rais
 * @Date: 2022-08-30 12:18:05
 * @LastEditTime: 2022-09-10 15:37:23
 * @LastEditors: Rais
 * @Description:
 */
use emg_refresh::{RefreshFor, RefreshWhoNoWarper};
use seed_styles::{CssBorderColor, CssBorderWidth, CssFill};

use crate::WidgetState;
impl RefreshFor<WidgetState> for CssFill
where
    WidgetState: RefreshWhoNoWarper,
{
    fn refresh_for(&self, who: &mut WidgetState) {
        who.fill = Some(self.clone())
    }
}

macro_rules! impl_css_refresh_widget_state {
    ($css:ident,$ws_v:ident) => {
        impl RefreshFor<WidgetState> for $css
        where
            WidgetState: RefreshWhoNoWarper,
        {
            fn refresh_for(&self, who: &mut WidgetState) {
                who.$ws_v = Some(self.clone());
            }
        }
    };
}

// impl_css_refresh_widget_state!(CssFill, fill);
//TODO finish full this
impl_css_refresh_widget_state!(CssBorderWidth, border_width);
impl_css_refresh_widget_state!(CssBorderColor, border_color);
