use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

pub struct QuoteOption<T>(pub Option<T>);

impl<T: ToTokens> ToTokens for QuoteOption<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.0.as_ref().map_or_else(
            || quote! { ::std::option::Option::None },
            |t| quote! { ::std::option::Option::Some(#t) },
        ));
    }
}
