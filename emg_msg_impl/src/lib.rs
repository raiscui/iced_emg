//! This crate implements the macro for `emg_msg` and should not be used directly.

use std::iter::FromIterator;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error, Meta, NestedMeta};

#[doc(hidden)]
pub fn emg_msg(_attr: TokenStream, item: TokenStream) -> Result<TokenStream, syn::Error> {
    // Implement your proc-macro logic here. :)
    // Ok(quote! {
    //     "Hello world!"
    // })
    let mut ast = syn::parse2::<DeriveInput>(item).unwrap();
    let mut has_derive = false;
    for a in &mut ast.attrs {
        if a.path.is_ident("derive") {
            has_derive = true;
            let mut m = match a.parse_meta()? {
                Meta::List(meta) => Ok(Vec::from_iter(meta.nested)),
                bad => Err(Error::new_spanned(bad, "unrecognized attribute")),
            }
            .unwrap();

            let tid = syn::parse2::<Meta>(quote! {better_any::Tid}).unwrap();
            let mut has_tid = false;
            for nm in m.iter() {
                has_tid = if let NestedMeta::Meta(mm) = nm {
                    has_tid || mm == &tid
                } else {
                    has_tid
                }
            }
            if !has_tid {
                m.push(NestedMeta::Meta(
                    syn::parse2::<Meta>(quote! {better_any::Tid}).unwrap(),
                ));
                a.tokens = quote! {
                    (#(#m),*)
                };
            }
        }
    }
    // println!("{:#?}", ast);

    let id = ast.ident.clone();
    let output = if has_derive {
        quote! {
            use better_any::TidAble;

            #ast
            impl<'a> any::MessageTid<'a> for #id {}
        }
    } else {
        quote! {
            use better_any::TidAble;

            #[derive(Debug, Copy, Clone, PartialEq,Tid)]
            #ast
            impl<'a> any::MessageTid<'a> for #id {}
        }
    };
    // println!("out::::\n{}", output);

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        let res = emg_msg(
            quote! {},
            quote! {
                #[derive(Debug, Copy, Clone, PartialEq)]
                enum Message {
                    IncrementPressed,
                    DecrementPressed,
                    // None,
                    Event(Event),
                    X,
                    Y,
                }
            },
        )
        .unwrap();
        println!("res===\n{}", res);
        insta::assert_display_snapshot!(res);
        // assert!(res);
    }
}
