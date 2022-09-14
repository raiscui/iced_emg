use tracing::warn;

use crate::Shaping;

/*
 * @Author: Rais
 * @Date: 2021-09-07 16:19:26
 * @LastEditTime: 2022-09-14 16:38:12
 * @LastEditors: Rais
 * @Description:
 */

pub trait ShapingUse<Use> {
    fn shaping_use(&mut self, use_something: &Use);
}

// impl<Use, Who> ShapingUse<Use> for Who {
//     default fn shaping_use(&mut self, use_something: &Use) {
//         warn!(
//             "this is un implemented yet {} shaping_use {}",
//             std::any::type_name::<Use>(),
//             std::any::type_name::<Who>()
//         );
//     }
// }

impl<Use, Who> ShapingUse<Use> for Who
where
    Use: Shaping<Who>,
{
    fn shaping_use(&mut self, use_something: &Use) {
        use_something.shaping(self);
    }
}
