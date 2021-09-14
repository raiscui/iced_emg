/*
 * @Author: Rais
 * @Date: 2021-08-31 11:58:58
 * @LastEditTime: 2021-09-14 16:23:09
 * @LastEditors: Rais
 * @Description:
 */
#[macro_export]
macro_rules! into_vector {
    ( $( $element:expr ) , * ) => {
        {
            let mut v = $crate::Vector::new();

            $(
                v.push_back($element.into());
            )*

            v
        }
    };
}
/// ## use parent type to generate GenericSize::Parent(T::static_type_name())
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
    use crate::{TypeCheck, TypeCheckObjectSafe, TypeName};

    struct EE {}
    impl TypeCheck for EE {
        fn static_type_name() -> crate::TypeName {
            TypeName("ff".to_string())
        }
    }
    impl TypeCheckObjectSafe for EE {
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
