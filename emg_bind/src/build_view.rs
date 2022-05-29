use std::{cell::RefCell, rc::Rc};

use emg_core::IdStr;
use emg_state::StateVar;

use crate::GraphType;

/*
 * @Author: Rais
 * @Date: 2022-05-26 18:22:22
 * @LastEditTime: 2022-05-28 23:20:57
 * @LastEditors: Rais
 * @Description:
 */
#[allow(clippy::module_name_repetitions)]
pub trait GraphBuildView<Message> {
    type Ix;

    fn build_view_state(&self, root_ix_var: &StateVar<Self::Ix>);
}

impl<Message> GraphBuildView<Message> for Rc<RefCell<GraphType<Message>>>
where
    Message: std::clone::Clone + std::fmt::Debug + 'static,
{
    type Ix = IdStr;

    fn build_view_state(&self, root_ix_var: &StateVar<Self::Ix>) {
        let this = self.clone();
        root_ix_var.watch().map(move |r_ix| {
            let g = this.borrow();
            let current_node = g.get_node_use_ix(r_ix).unwrap();
        });
    }
}
