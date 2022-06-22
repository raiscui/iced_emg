use tracing::warn;

use crate::RefreshFor;

/*
 * @Author: Rais
 * @Date: 2021-09-07 16:19:26
 * @LastEditTime: 2022-06-22 18:00:27
 * @LastEditors: Rais
 * @Description:
 */

pub trait RefreshUse<Use> {
    fn refresh_use(&mut self, use_something: &Use);
}

// impl<Use, Who> RefreshUse<Use> for Who {
//     default fn refresh_use(&mut self, use_something: &Use) {
//         warn!(
//             "this is un implemented yet {} refresh_use {}",
//             std::any::type_name::<Use>(),
//             std::any::type_name::<Who>()
//         );
//     }
// }

impl<Use, Who> RefreshUse<Use> for Who
where
    Use: RefreshFor<Who>,
{
    fn refresh_use(&mut self, use_something: &Use) {
        use_something.refresh_for(self);
    }
}
