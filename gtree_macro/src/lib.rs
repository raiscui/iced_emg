// #![feature(proc_macro_diagnostic)]

use proc_macro2::TokenStream;
// use quote::{quote, ToTokens};
use quote::{quote, quote_spanned, ToTokens};
// use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{bracketed, ext::IdentExt, punctuated::Punctuated, spanned::Spanned, token};
use syn::{Ident, Token};

// ────────────────────────────────────────────────────────────────────────────────
// use proc_macro::Diagnostic;
pub mod kw {
    // use std::fmt::Debug;

    syn::custom_keyword!(Layer);

    // impl Debug for layer {
    //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //         f.write_str(concat!("Keyword [", stringify!(layer), "]"))
    //     }
    // }
}

// ────────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum GTreeElement {
    GL(GTreeLayerStruct),
    GS(GTreeSurface),
    OtherExpr(syn::Expr),
}
impl Parse for GTreeElement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // use syn::ext::IdentExt;

        // if input.peek(kw::layer) {
        if input.peek(kw::Layer) {
            Ok(GTreeElement::GL(input.parse()?))
        } else if input.peek(Ident::peek_any) {
            Ok(GTreeElement::GS(input.parse()?))
        } else {
            Ok(GTreeElement::OtherExpr(input.parse()?))
        }
    }
}

impl ToTokens for GTreeElement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::GL(layer_struct) => layer_struct.to_tokens(tokens),
            Self::GS(surface) => surface.to_tokens(tokens),
            Self::OtherExpr(expr) => expr.to_tokens(tokens),
        }
    }
}

// @ GSurface ────────────────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GTreeSurface {
    expr: syn::Expr,
}
impl Parse for GTreeSurface {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        //println!("GSurface:{}", input);
        Ok(GTreeSurface {
            expr: input.parse()?,
        })
    }
}
impl ToTokens for GTreeSurface {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // self.expr.to_tokens(tokens)
        let GTreeSurface { expr } = self;
        // println!("expr===={:?}", self.expr);

        quote_spanned!(
            expr.span()=> GTreeBuilderElement::TreeEl(#expr.into())
        )
        .to_tokens(tokens)
        // quote_spanned!(expr.span()=>GTreeBuilderElement::El(#expr.into())).to_tokens(tokens)
        // quote!(GTreeBuilderElement::El(#expr.into())).to_tokens(tokens)
    }
}

// @ GTreeLayerStruct ────────────────────────────────────────────────────────────────────────────────
type ChildrenType = Option<Punctuated<GTreeElement, Token![,]>>;
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GTreeLayerStruct {
    layer: kw::Layer,
    id: syn::LitStr,
    children: ChildrenType,
}
// TODO make id Option,
/*
Uuid::new_v4()
.to_simple()
.encode_lower(&mut Uuid::encode_buffer())
.to_string()
*/

impl Parse for GTreeLayerStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let layer = input.parse::<kw::Layer>()?;

        let id = input.parse::<syn::LitStr>()?;

        if input.peek(Token![,]) {
            // if input.is_empty() {
            return Ok(GTreeLayerStruct {
                layer,
                id,
                children: None,
            });
        }

        //println!("input {:?}", &input);
        let content;
        let _bracket = bracketed!(content in input);
        //println!("brace_token=>{:?}", &bracket);
        let children: ChildrenType = Some(content.parse_terminated(GTreeElement::parse)?);
        //println!("children:=>{:?}", &children);
        // println!("children op :=>{}", quote!(  #children));

        Ok(GTreeLayerStruct {
            layer,
            id,
            children,
        })
    }
}

impl ToTokens for GTreeLayerStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let GTreeLayerStruct {
            layer,
            id,
            children,
        } = self;
        let children_iter = children.iter();
        let g_tree_builder_element_layer_token =
            quote_spanned! {layer.span()=>GTreeBuilderElement::TreeLayer};
        let id_token = quote_spanned! {id.span()=> String::from(#id)};
        let children_token = quote_spanned! {children.span()=>vec![#(#children_iter),*]};
        // let brace_op_token = quote_spanned! {children.span()=>vec![#children_token]};

        quote!(#g_tree_builder_element_layer_token(#id_token,#children_token)).to_tokens(tokens)
        // quote!(GTreeBuilderElement::#layer(String::from(#id),vec![#(#children_iter),*])).to_tokens(tokens)
    }
}

// @ Gtree ────────────────────────────────────────────────────────────────────────────────
#[derive(Debug)]
pub struct Gtree {
    // g_type: Option<syn::Ident>,
    root: GTreeLayerStruct,
}

impl Parse for Gtree {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // let _: LayerSegments = input.parse()?;
        // let _: GLayer = input.parse()?;
        // let g_type = input.parse::<syn::Ident>().ok();
        // println!(
        //     "g_type------: {}",
        //     quote!(
        //        & #g_type
        //     )
        // );

        let root = input.parse::<GTreeLayerStruct>()?;

        Ok(Gtree { root })
    }
}

// #[derive(Debug)]
// enum NodeList<'a, A> {
//     // Nil,
//     LayerCons(A, &'a [NodeList<'a, A>]),
// }

impl ToTokens for Gtree {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Gtree { root } = self;

        // fn pp<A: std::fmt::Debug>(layer: &NodeList<A>) {
        //     match layer {
        //         NodeList::LayerCons(id, list) => {
        //             println!("{:?}==>{:?}", id, list);
        //             list.iter().for_each(|l| pp(l));
        //         }
        //     };
        // }
        let token = quote_spanned! {root.span()=> {
            {
                use std::{cell::RefCell, rc::Rc};
                use gtree::log::Level;
                use gtree::log;
                use gtree::illicit;
                use emg_bind::{Uuid,runtime::Text,runtime::Element,GElement, Graph, GraphStore, Layer,E,N,GraphType};
                use GElement::{GContainer, GSurface};


                // type N<'a, Message> = RefCell<GElement<'a, Message>>;
                // type E = String;
                // type GraphType<'a, Message> = Graph<N<'a, Message>, E>;


                // ─────────────────────────────────────────────────────────────────

                #[derive(Debug)]
                enum GTreeBuilderElement<'a, Message> {
                    TreeLayer(String, Vec<GTreeBuilderElement<'a, Message>>),
                    TreeEl(Element<'a, Message>),
                }

                // • • • • •


                fn handle_root<'a, Message>(
                    g: &mut GraphType<'a, Message>,
                    treelayer: &GTreeBuilderElement<'a, Message>,
                ) where
                    Message: Clone + std::fmt::Debug,
                {
                    match treelayer {
                        GTreeBuilderElement::TreeLayer(id, children_list) => {
                            log::info!("{:?}==>{:?}", id.to_string(), children_list);
                            let nix = g.insert_node(
                                id.to_string(),
                                RefCell::new(GContainer(
                                    Layer::new().push(Text::new(format!("in quote..{}", id.to_string()))),
                                )),
                            );
                            illicit::Layer::new().offer(nix.clone()).enter(|| {
                                assert_eq!(
                                    *illicit::expect::<emg_bind::NodeIndex<String>>(),
                                    nix.clone()
                                );
                                log::info!("{:?}", *illicit::expect::<emg_bind::NodeIndex<String>>());
                                children_list
                                    .iter()
                                    .for_each(|child_layer| handle_layer(g, child_layer));
                            });
                        }
                        GTreeBuilderElement::TreeEl(surface) => {
                            panic!("not allow this {:?}, first element must layer ", surface)
                        }
                    };
                }
                fn handle_layer<'a, Message>(
                    g: &mut GraphType<'a, Message>,
                    treelayer: &GTreeBuilderElement<'a, Message>,
                ) where
                    Message: Clone + std::fmt::Debug,
                {
                    let parent_nix = illicit::expect::<emg_bind::NodeIndex<String>>();
                    match treelayer {
                        GTreeBuilderElement::TreeLayer(id, children_list) => {
                            log::info!("{:?}==>{:?}", id.to_string(), children_list);
                            let nix = g.insert_node(
                                id.to_string(),
                                RefCell::new(GContainer(
                                    Layer::new().push(Text::new(format!("in quote..{}", id.to_string()))),
                                )),
                            );
                            let edge = format!("{} -> {}", parent_nix.index(), nix.index());
                            log::info!("{}", &edge);
                            g.insert_update_edge(&*parent_nix, &nix, edge);
                            illicit::Layer::new().offer(nix.clone()).enter(|| {
                                assert_eq!(
                                    *illicit::expect::<emg_bind::NodeIndex<String>>(),
                                    nix.clone()
                                );
                                children_list
                                    .iter()
                                    .for_each(|child_layer| handle_layer(g, child_layer));
                            });
                        }
                        GTreeBuilderElement::TreeEl(surface) => {
                            let parent_nix = illicit::expect::<emg_bind::NodeIndex<String>>();
                            let nix = g.insert_node(
                                Uuid::new_v4()
                                    .to_simple()
                                    .encode_lower(&mut Uuid::encode_buffer())
                                    .to_string(),
                                RefCell::new(GSurface(surface.clone())),
                            );
                            let edge = format!("{} -> {}", parent_nix.index(), nix.index());
                            log::info!("{}", &edge);
                            g.insert_update_edge(&*parent_nix, &nix, edge);
                        }
                    };
                }

                // • • • • •

                gtree::console_log::init_with_level(Level::Debug).ok();


                let children = #root;


                GraphType::<Message>::init();
                GraphType::<Message>::get_mut_graph_with(|g| {
                    // g.insert_node(
                    //     1,
                    //     Rc::new(GContainer(Layer::new().push(
                    //         Layer::new().push(Text::new("in quote..")),
                    //     ))),
                    // );
                    handle_root(g,&children);
                    log::info!("{:#?}",g);
                });






            }
        }};
        token.to_tokens(tokens)
    }
}

/// @ gtree_macro ────────────────────────────────────────────────────────────────────────────────
pub fn gtree_macro(item: TokenStream) -> Result<TokenStream, syn::Error> {
    let output = syn::parse2::<Gtree>(item)?;
    Ok(quote_spanned! { output.span()=>#output})
}

// ────────────────────────────────────────────────────────────────────────────────
// @ Gview ────────────────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Gview {
    root_ix: syn::LitStr,
}
impl Parse for Gview {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Gview {
            root_ix: input.parse()?,
        })
    }
}
impl ToTokens for Gview {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // self.expr.to_tokens(tokens)
        let Gview { root_ix } = self;
        quote!({
            use emg_bind::{GraphType};
            GraphType::<Message>::view( #root_ix .to_string() )

        })
        .to_tokens(tokens)
    }
}
/// @ gview_macro ────────────────────────────────────────────────────────────────────────────────
pub fn gview_macro(item: TokenStream) -> Result<TokenStream, syn::Error> {
    let output = syn::parse2::<Gview>(item)?;
    Ok(quote_spanned! { output.span()=>#output})
}
// @ test ────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test1() {
        fn token_test(input: &str) {
            match syn::parse_str::<Gtree>(input) {
                Ok(ok) => println!("===>{}", ok.to_token_stream()),
                Err(error) => println!("...{:?}", error),
            }
        }
        println!();
        // type GraphType = Vec<i32>;
        let input = r#" 
        Layer "a" [
            Layer "b" [
                Layer "c" [],
                Layer "d" [],
                Text::new(format!("in quote..{}", "b"))
            ]
        ]

        "#;

        token_test(input);
        println!();
    }

    #[test]
    fn test2() {
        fn token_test(input: &str) {
            match syn::parse_str::<Gview>(input) {
                Ok(ok) => println!("Gview===>{}", ok.to_token_stream()),
                Err(error) => println!("...{:?}", error),
            }
        }
        println!();
        // type GraphType = Vec<i32>;
        let input = r#" "a" "#;

        token_test(input);
        println!();
    }
    // #[test]
    // fn test3() {
    //     fn token_test(input: &str) {
    //         match syn::parse_str::<layer>(input) {
    //             Ok(ok) => println!("Gview===>{}", ok.to_token_stream()),
    //             Err(error) => println!("...{:?}", error),
    //         }
    //     }
    //     println!();
    //     // type GraphType = Vec<i32>;
    //     let input = r#" layer("f") "#;

    //     token_test(input);
    //     println!();
    // }
}
