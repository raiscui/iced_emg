/*
 * @Author: Rais
 * @Date: 2022-08-30 12:18:05
 * @LastEditTime: 2023-01-13 12:15:42
 * @LastEditors: Rais
 * @Description:
 */
use emg_shaping::{Shaping, ShapingWhoNoWarper};
use seed_styles::{CssBorderColor, CssBorderWidth, CssFill};

use crate::WidgetState;
impl Shaping<WidgetState> for CssFill
where
    WidgetState: ShapingWhoNoWarper,
{
    fn shaping(&self, who: &mut WidgetState) {
        who.fill = Some(self.clone())
    }
}

macro_rules! impl_css_refresh_widget_state {
    ($css:ident,$ws_v:ident) => {
        impl Shaping<WidgetState> for $css
        where
            WidgetState: ShapingWhoNoWarper,
        {
            fn shaping(&self, who: &mut WidgetState) {
                who.$ws_v = Some(self.clone());
            }
        }
    };
}

// impl_css_refresh_widget_state!(CssFill, fill);
//TODO finish full this
impl_css_refresh_widget_state!(CssBorderWidth, border_width);
impl_css_refresh_widget_state!(CssBorderColor, border_color);
