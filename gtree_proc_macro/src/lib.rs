// #![feature(proc_macro_diagnostic)]
use proc_macro;
// use proc_macro_error::*;
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
