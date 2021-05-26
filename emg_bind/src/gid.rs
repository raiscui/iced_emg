use emg_refresh::{RefreshFor, RefreshUseNoWarper, RefreshWhoNoWarper};

/*
 * @Author: Rais
 * @Date: 2021-05-26 16:32:44
 * @LastEditTime: 2021-05-26 16:54:13
 * @LastEditors: Rais
 * @Description:
 */
impl !RefreshUseNoWarper for Gid {}

pub struct Gid(String);

impl Gid {
    #[must_use]
    pub fn id(&self) -> String {
        self.0.clone()
    }
}

impl<Who> RefreshFor<Who> for Gid
where
    Who: RefreshWhoNoWarper,
{
    default fn refresh_for(&self, _el: &mut Who) {}
}
