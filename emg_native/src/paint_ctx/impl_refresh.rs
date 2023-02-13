/*
 * @Author: Rais
 * @Date: 2022-08-30 12:18:05
 * @LastEditTime: 2023-02-03 17:55:52
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
    fn shaping(&self, who: &mut WidgetState) -> bool {
        let new_v = Some(self.clone());
        if who.fill != new_v {
            who.fill = new_v;
            return true;
        }
        false
    }
}

macro_rules! impl_css_refresh_widget_state {
    ($css:ident,$ws_v:ident) => {
        impl Shaping<WidgetState> for $css
        where
            WidgetState: ShapingWhoNoWarper,
        {
            fn shaping(&self, who: &mut WidgetState) -> bool {
                let new_v = Some(self.clone());
                if who.$ws_v != new_v {
                    who.$ws_v = new_v;
                    return true;
                }
                false
            }
        }
    };
}

// impl_css_refresh_widget_state!(CssFill, fill);
//TODO finish full this
impl_css_refresh_widget_state!(CssBorderWidth, border_width);
impl_css_refresh_widget_state!(CssBorderColor, border_color);
