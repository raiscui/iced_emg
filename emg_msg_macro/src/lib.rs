//! This crate implements the macro for `emg_msg` and should not be used directly.
// extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn emg_msg(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as proc_macro2::TokenStream);
    let item = parse_macro_input!(item as proc_macro2::TokenStream);

    match emg_msg_impl::emg_msg(attr, item) {
        Ok(tokens) => tokens.into(),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}
