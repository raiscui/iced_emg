// #![feature(proc_macro_diagnostic)]

use std::{cell::RefCell, rc::Rc};

use proc_macro2::{TokenStream, TokenTree};
use proc_quote::{quote, ToTokens};
use std::fmt;
use syn::{braced, bracketed, punctuated::Punctuated, Attribute, Block};
use syn::{
    parse::{Parse, ParseStream, Peek},
    Field,
};
use syn::{Expr, Ident, Token};

// ────────────────────────────────────────────────────────────────────────────────
// use proc_macro::Diagnostic;
pub mod kw {
    use std::fmt::Debug;

    syn::custom_keyword!(layer);

    // impl Debug for layer {
    //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //         f.write_str(concat!("Keyword [", stringify!(layer), "]"))
    //     }
    // }
}
// ────────────────────────────────────────────────────────────────────────────────

// syn::custom_punctuation!(LayerSeparator, |);

// // expr |- expr, |- expr |- expr ,...
// #[allow(dead_code)]
// struct LayerSegments {
//     segments: Punctuated<GLayer, LayerSeparator>,
// }

// impl Parse for LayerSegments {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         let mut segments = Punctuated::new();
//         println!("l1");

//         let first = parse_until(input, LayerSeparator)?;
//         println!("l2:{}", &input);
//         println!("l2:{}", &first);
//         println!("l2:{:#?}", &first);
//         // let ff = syn::parse2(first)?;

//         segments.push_value(syn::parse2::<GLayer>(first)?);
//         println!("l2.1");

//         while !input.is_empty() && input.peek(LayerSeparator) {
//             println!("in l3.0");

//             segments.push_punct(input.parse()?);
//             println!("l3");

//             let next = parse_until(input, LayerSeparator)?;
//             segments.push_value(syn::parse2(next)?);
//             println!("l4");
//         }

//         Ok(LayerSegments { segments })
//     }
// }

// fn parse_until<E: Peek>(input: ParseStream, end: E) -> syn::Result<TokenStream> {
//     let mut tokens = TokenStream::new();
//     while !input.is_empty() && !input.peek(end) {
//         let next: TokenTree = input.parse()?;
//         tokens.extend(Some(next));
//     }
//     Ok(tokens)
// }

// @ GLayer ────────────────────────────────────────────────────────────────────────────────
type ChildrenType = Punctuated<GLayer, Token![,]>;
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GLayer {
    id: syn::LitStr,
    children: ChildrenType,
}

impl Parse for GLayer {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        println!("in GLayer");
        let _: kw::layer = input.parse()?;
        println!("1");

        let layer_id: syn::LitStr = input.parse()?;
        println!("2");
        // if input.is_empty() {
        //     return Ok(GLayer {
        //         id: layer_id,
        //         children: Punctuated::new(),
        //     });
        // }

        println!("input {:?}", &input);
        let content;
        let brace_token = bracketed!(content in input);
        println!("{:?}", &brace_token);
        let children: ChildrenType = content.parse_terminated(GLayer::parse)?;
        println!("{:?}", &children);

        Ok(GLayer {
            id: layer_id,
            children,
        })
    }
}

impl ToTokens for GLayer {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let GLayer { id, children } = self;
        let children_iter = children.iter();
        quote!(NodeList::LayerCons(#id,&[#(#children_iter),*])).to_tokens(tokens)
    }
}

// @ Gtree ────────────────────────────────────────────────────────────────────────────────
#[derive(Debug)]
pub struct Gtree {
    g_type: Option<syn::TypePath>,
    root: GLayer,
}

impl Parse for Gtree {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // let _: LayerSegments = input.parse()?;
        // let _: GLayer = input.parse()?;
        let g_type = input.parse::<syn::TypePath>().ok();
        println!(
            "g_type------: {}",
            quote!(
                #g_type
            )
        );

        let root = input.parse::<GLayer>()?;

        println!("alayer");

        println!("gtree end");
        Ok(Gtree { g_type, root })
    }
}

// #[derive(Debug)]
// enum NodeList<'a, A> {
//     // Nil,
//     LayerCons(A, &'a [NodeList<'a, A>]),
// }

impl ToTokens for Gtree {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        println!("gtree to tokens");
        let Gtree { g_type, root } = self;

        // fn pp<A: std::fmt::Debug>(layer: &NodeList<A>) {
        //     match layer {
        //         NodeList::LayerCons(id, list) => {
        //             println!("{:?}==>{:?}", id, list);
        //             list.iter().for_each(|l| pp(l));
        //         }
        //     };
        // }

        if let Some(gt) = g_type {
            quote!(
                use std::{cell::RefCell, rc::Rc};
                use gtree::log::Level;
                use gtree::log;
                use gtree::illicit;
                use emg_bind::{runtime::Text,GElement, Graph, GraphStore, Layer};
                use GElement::{GContainer, GSurface};

                type N<'a> = Rc<GElement<'a, Message>>;
                type Ix = &'static str;

                #[derive(Debug)]
                enum NodeList<'a, A> {
                    // Nil,
                    LayerCons(A, &'a [NodeList<'a, A>]),
                }

                fn handle_root<'a>(g: &mut GraphType, layer: &NodeList<'a, Ix>) {
                    match layer {
                        NodeList::LayerCons(id, children_list) => {
                            log::info!("{:?}==>{:?}", &id, &children_list);
                            let nix = g.insert_node(
                                id.clone(),
                                Rc::new(GContainer(
                                    Layer::new().push(Text::new(format!("in quote..{}", id))),
                                )),
                            );
                            illicit::Layer::new().offer(nix).enter(|| {
                                assert_eq!(*illicit::expect::<emg_bind::NodeIndex<Ix>>(), nix);
                                log::info!("{:?}", *illicit::expect::<emg_bind::NodeIndex<Ix>>());
                                children_list.iter().for_each(|l| handle_layer(g, l));
                            });
                        }
                    };
                }
                fn handle_layer<'a>(g: &mut GraphType, layer: &NodeList<'a, Ix>) {
                    let parent_nix = *illicit::expect::<emg_bind::NodeIndex<Ix>>();
                    match layer {
                        NodeList::LayerCons(id, children_list) => {
                            log::info!("{:?}==>{:?}", &id, &children_list);
                            let nix = g.insert_node(
                                id.clone(),
                                Rc::new(GContainer(
                                    Layer::new().push(Text::new(format!("in quote..{}", &id))),
                                )),
                            );
                            let edge = format!("{} -> {}", parent_nix.index(), nix.index());
                            log::info!("{}", &edge);
                            g.insert_update_edge(&parent_nix, &nix, edge);
                            illicit::Layer::new().offer(nix).enter(|| {
                                assert_eq!(*illicit::expect::<emg_bind::NodeIndex<Ix>>(), nix);
                                children_list.iter().for_each(|l| handle_layer(g, l));
                            });
                        }
                    };
                }

                // • • • • •

                gtree::console_log::init_with_level(Level::Debug).ok();


                let children = #root;


                <#gt>::init();
                <#gt>::get_mut_graph_with(|g| {
                    // g.insert_node(
                    //     1,
                    //     Rc::new(GContainer(Layer::new().push(
                    //         Layer::new().push(Text::new("in quote..")),
                    //     ))),
                    // );
                    handle_root(g,&children);
                    log::info!("{:#?}",g);
                });







                println!(" {:#?}",&children);
                // pp(&children);
                log::info!("info==={:#?}",&children);



            )
            .to_tokens(tokens)
        } else {
            quote!({}).to_tokens(tokens)
        }
    }
}

/// @ gtree_macro ────────────────────────────────────────────────────────────────────────────────
pub fn gtree_macro(item: TokenStream) -> Result<TokenStream, syn::Error> {
    let output = syn::parse2::<Gtree>(item)?;
    //println!("||>{}", &output);
    let mut tokens = TokenStream::new();
    output.to_tokens(&mut tokens);

    Ok(tokens)
}
// @ test ────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test1() {
        fn token_test(input: &str) {
            match syn::parse_str::<Gtree>(input) {
                Ok(ok) => println!("===>{}", quote!(#ok)),
                Err(error) => println!("...{:?}", error),
            }
        }
        println!();
        type GraphType = Vec<i32>;
        let input = r#" 
        GraphType layer "a" [
            layer "b" [
                layer "c" [],
                layer "d" []
            ]
        ]

        "#;

        token_test(input);
        println!();
    }
}
