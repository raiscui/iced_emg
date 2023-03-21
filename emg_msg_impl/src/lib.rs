//! This crate implements the macro for `emg_msg` and should not be used directly.

use std::iter::FromIterator;

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[doc(hidden)]
pub fn emg_msg(_attr: TokenStream, item: TokenStream) -> Result<TokenStream, syn::Error> {
    // Implement your proc-macro logic here. :)
    // Ok(quote! {
    //     "Hello world!"
    // })
    let ast = syn::parse2::<DeriveInput>(item).unwrap();
    let mut has_derive = false;
    let mut has_tid = false;

    let tid_token_stream = quote! {#[derive(better_any::Tid)]};
    for attr in &ast.attrs {
        if attr.path().is_ident("derive") {
            has_derive = true;

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("Tid") {
                    has_tid = true;
                }
                Ok(())
            })
            .ok();
        }
    }
    // println!("{:#?}", ast);

    let id = &ast.ident;
    let output = if has_derive {
        if has_tid {
            quote! {
                use better_any::TidAble;

                #ast
                impl<'a> any::MessageTid<'a> for #id {}
            }
        } else {
            quote! {
                use better_any::TidAble;

                #tid_token_stream
                #ast
                impl<'a> any::MessageTid<'a> for #id {}
            }
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
        #[cfg(feature = "insta")]
        insta::assert_display_snapshot!(res);
        // assert!(res);
    }
}
