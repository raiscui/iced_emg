use crate::RefreshFor;

/*
 * @Author: Rais
 * @Date: 2021-09-07 16:19:26
 * @LastEditTime: 2021-09-07 16:43:16
 * @LastEditors: Rais
 * @Description:
 */

pub trait RefreshUse<Use> {
    fn refresh_use(&mut self, use_something: &Use);
}
impl<Use, Who> RefreshUse<Use> for Who
where
    Use: RefreshFor<Who>,
{
    fn refresh_use(&mut self, use_something: &Use) {
        use_something.refresh_for(self);
    }
}
