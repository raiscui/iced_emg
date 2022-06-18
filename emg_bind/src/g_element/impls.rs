/*
 * @Author: Rais
 * @Date: 2022-06-16 14:46:28
 * @LastEditTime: 2022-06-17 15:40:36
 * @LastEditors: Rais
 * @Description:
 */
// impl<T> From<StateVar<T>> for GElement<Message> {}

// pub trait NotStateAnchorEqRefreshFor<Who>: EqRefreshFor<Who> + NotStateAnchor4Refresher {}

// impl<Who> core::cmp::PartialEq for dyn NotStateAnchorEqRefreshFor<Who> {
//     fn eq(&self, other: &Self) -> bool {
//         self.box_eq(other.as_any())
//     }
// }
// impl<Who: 'static> core::cmp::PartialEq<dyn NotStateAnchorEqRefreshFor<Who>>
//     for Box<dyn NotStateAnchorEqRefreshFor<Who>>
// {
//     fn eq(&self, other: &(dyn NotStateAnchorEqRefreshFor<Who>)) -> bool {
//         self.box_eq(other.as_any())
//     }
// }

// impl<Who, Use> NotStateAnchorEqRefreshFor<Who> for Use where
//     Use: EqRefreshFor<Who> + NotStateAnchor4Refresher
// {
// }

// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────

use std::rc::Rc;

use emg_refresh::NotStateAnchor4Refresher;

use crate::GElement;

#[derive(Clone)]
pub struct GelRefresher<Use>(Rc<dyn Fn() -> Use>)
where
    Use: NotStateAnchor4Refresher;

impl<Use> GelRefresher<Use>
where
    Use: NotStateAnchor4Refresher,
{
    pub fn new<F: Fn() -> Use + 'static>(f: F) -> Self {
        Self(Rc::new(f))
    }
    #[must_use]
    pub fn get(&self) -> Use {
        (self.0)()
        // Rc::clone(&self.0)()
    }
}
impl<Use> Eq for GelRefresher<Use> where Use: NotStateAnchor4Refresher {}
impl<Use> PartialEq for GelRefresher<Use>
where
    Use: NotStateAnchor4Refresher,
{
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(
            (std::ptr::addr_of!(*self.0)).cast::<u8>(),
            (std::ptr::addr_of!(*other.0)).cast::<u8>(),
        )
        // Rc::ptr_eq(&self.0, &other.0)
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────

// impl<Use, Message> NotStateAnchor4Refresher for GelRefresher<Use, Message> where
//     Use: NotStateAnchor4Refresher + RefreshFor<GElement<Message>>
// {
// }
// impl<Use, Message> !RefreshUseNoWarper for GelRefresher<Use, Message> {}

// impl<Use, Message> RefreshFor<GElement<Message>> for GelRefresher<Use, Message>
// where
//     Use: RefreshUseNoWarper + NotStateAnchor4Refresher + RefreshFor<GElement<Message>>,
// {
//     fn refresh_for(&self, who: &mut GElement<Message>) {
//         // self.get()().refresh_for(who);
//         who.refresh_for_use(&self.get());
//     }
// }
// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────

// impl<Use, Message> NotStateAnchorEqRefreshFor<GElement<Message>> for GelRefresher<Use, Message>
// where
//     Use: NotStateAnchor4Refresher + RefreshFor<GElement<Message>>,
//     Message: NotStateAnchor4Refresher,
// {
// }
