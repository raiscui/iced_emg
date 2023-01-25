/*
 * @Author: Rais
 * @Date: 2021-08-31 11:58:58
 * @LastEditTime: 2023-01-23 22:47:50
 * @LastEditors: Rais
 * @Description:
 */
#[macro_export]
macro_rules! into_vector {
    (  $element:expr  ) => {
        {
             $crate::Vector::unit($element.into())


        }
    };
    ( $( $element:expr ) , + ) => {
        {
            let mut v = $crate::Vector::new();

            $(
                v.push_back($element.into());
            )*

            v
        }
    };
}
#[macro_export]
macro_rules! into_tvec {

    ( $( $element:expr ) , * ) => {
        {
            $crate::tiny_vec!( $( $element.into() ),*)

        }
    };
}
#[macro_export]
macro_rules! into_smvec {

    ( $( $element:expr ) , * ) => {
        {
            $crate::smallvec![ $( $element.into() ),*]

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
    use crate::{IdStr, TypeCheck, TypeCheckObjectSafe, TypeName};

    struct EE {}
    impl TypeCheck for EE {
        const TYPE_NAME: TypeName = TypeName::new(IdStr::new_inline("ff"));
        // fn static_type_name() -> crate::TypeName {
        //     TypeName::new(IdStr::new_inline("ff"))
        // }
    }
    impl TypeCheckObjectSafe for EE {
        fn type_name(&self) -> crate::TypeName {
            // TypeName::new(IdStr::new_inline("ff"))
            "ff".into()
            // TypeName::new(IdStr::new_inline("ff"))
        }
    }

    #[test]
    fn test_macro() {
        let f = parent!(EE);
        println!("{}", f);
    }
}
