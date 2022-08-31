/*
 * @Author: Rais
 * @Date: 2022-08-30 12:18:05
 * @LastEditTime: 2022-08-31 15:17:13
 * @LastEditors: Rais
 * @Description:
 */
use emg_refresh::{RefreshFor, RefreshWhoNoWarper};
use seed_styles::CssFill;

use crate::WidgetState;

impl RefreshFor<WidgetState> for CssFill
where
    WidgetState: RefreshWhoNoWarper,
{
    fn refresh_for(&self, who: &mut WidgetState) {
        who.fill = Some(self.clone())
    }
}
