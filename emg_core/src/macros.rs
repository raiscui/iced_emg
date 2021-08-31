/*
 * @Author: Rais
 * @Date: 2021-08-31 11:58:58
 * @LastEditTime: 2021-08-31 12:40:59
 * @LastEditors: Rais
 * @Description:
 */
#[macro_export]
macro_rules! into_vector {
    ( $( $element:expr ) , * ) => {
        {
            let mut v = im_rc::Vector::new();

            $(
                v.push_back($element.into());
            )*

            v
        }
    };
}

#[macro_export]
macro_rules! parent {
    ( $type_name:ty  ) => {{
        $crate::parent_ty::<$type_name>()
    }};
    ( $type_name:literal  ) => {{
        $crate::parent_str($type_name)
    }};
}

#[cfg(test)]
mod tests {
    use crate::{TypeCheck, TypeName};

    struct EE {}
    impl TypeCheck for EE {
        fn static_type_name() -> crate::TypeName {
            TypeName("ff".to_string())
        }

        fn type_name(&self) -> crate::TypeName {
            TypeName("ff".to_string())
        }
    }

    #[test]
    fn test_macro() {
        let f = parent!(EE);
        println!("{}", f);
    }
}
