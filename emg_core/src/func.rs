/*
 * @Author: Rais
 * @Date: 2021-06-16 12:55:09
 * @LastEditTime: 2021-06-28 15:13:26
 * @LastEditors: Rais
 * @Description:
 */
#[macro_export]
macro_rules! into_vector {
    ( $( $element:expr ) , * ) => {
        {
            let mut v = im::Vector::new();

            $(
                v.push_back($element.into());
            )*

            v
        }
    };
}
