use crate::Shaping;

/*
 * @Author: Rais
 * @Date: 2021-09-07 16:19:26
 * @LastEditTime: 2023-02-04 21:15:31
 * @LastEditors: Rais
 * @Description:
 */

pub trait ShapingUse<Use> {
    #[must_use]
    fn shaping_use(&mut self, use_something: &Use) -> bool;
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
    #[inline]
    fn shaping_use(&mut self, use_something: &Use) -> bool {
        use_something.shaping(self)
    }
}
