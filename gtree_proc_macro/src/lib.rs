// #![feature(proc_macro_diagnostic)]
use proc_macro;
// use proc_macro_error::*;
// use quote::quote;
use proc_quote::quote;
use syn::parse_macro_input;

// #![feature(proc_macro_diagnostic)]
#[proc_macro]
pub fn gtree(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as gtree_macro::Gtree);

    quote!(#item).into()
}
// pub fn gtree(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     let item = parse_macro_input!(input as proc_macro2::TokenStream);

//     match gtree_macro::gtree_macro(item) {
//         Ok(tokens) => tokens.into(),
//         Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
//     }
// }

// #![feature(proc_macro_diagnostic)]
#[proc_macro]
pub fn gview(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as gtree_macro::Gview);

    quote!(#item).into()
}
#[proc_macro]
pub fn glayer(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as gtree_macro::GTreeLayerStruct);
    quote!(#item).into()
}
