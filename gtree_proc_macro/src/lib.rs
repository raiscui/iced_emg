// #![feature(proc_macro_diagnostic)]
extern crate proc_macro;
// use proc_macro_error::*;
// use proc_quote::quote;
use syn::parse_macro_input;

// #![feature(proc_macro_diagnostic)]
#[proc_macro]
pub fn gtree(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as proc_macro2::TokenStream);

    match gtree_macro::gtree_macro(item) {
        Ok(tokens) => tokens.into(),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
    }
}

// #[proc_macro_attribute]
// pub fn emg(
//     args: proc_macro::TokenStream,
//     input: proc_macro::TokenStream,
// ) -> proc_macro::TokenStream {
//     let input = parse_macro_input!(input as proc_macro2::TokenStream);

//     // Parse the list of variables the user wanted to print.
//     let args = parse_macro_input!(args as proc_macro2::TokenStream);

//     // Use a syntax tree traversal to transform the function body.
//     match gtree_macro::emg_macro(args, input) {
//         Ok(tokens) => tokens.into(),
//         Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
//     }
// }

// // #![feature(proc_macro_diagnostic)]
// #[proc_macro]
// pub fn gview(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     let item = parse_macro_input!(input as gtree_macro::Gview);

//     quote!(#item).into()
// }
// #[proc_macro]
// pub fn glayer(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     let item = parse_macro_input!(input as gtree_macro::GTreeLayerStruct);
//     quote!(#item).into()
// }
