// #![feature(proc_macro_diagnostic)]

use std::{cell::RefCell, rc::Rc};

use proc_macro2::{TokenStream, TokenTree};
use proc_quote::{quote, ToTokens};
use std::fmt;
use syn::{
    braced, bracketed, ext::IdentExt, punctuated::Punctuated, token::CustomToken, Attribute, Block,
};
use syn::{
    parse::{Parse, ParseStream},
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
#[derive(Debug, Clone)]
enum GTreeElement {
    GL(GTreeLayerStruct),
    GS(GTreeSurface),
    OtherExpr(syn::Expr),
}
impl Parse for GTreeElement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // use syn::ext::IdentExt;

        if input.peek(kw::layer) {
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
            Self::GL(layer) => layer.to_tokens(tokens),
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
        println!("GSurface:{}", input);
        Ok(GTreeSurface {
            expr: input.parse()?,
        })
    }
}
impl ToTokens for GTreeSurface {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // self.expr.to_tokens(tokens)
        let GTreeSurface{expr } = self;
        quote!(GTreeBuilderElement::El(#expr.into())).to_tokens(tokens)
    }
}

// @ GLayer ────────────────────────────────────────────────────────────────────────────────
type ChildrenType = Punctuated<GTreeElement, Token![,]>;
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GTreeLayerStruct {
    id: syn::LitStr,
    children: ChildrenType,
}

impl Parse for GTreeLayerStruct {
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
        println!("brace_token=>{:?}", &brace_token);
        let children: ChildrenType = content.parse_terminated(GTreeElement::parse)?;
        println!("children:=>{:?}", &children);
        println!("children op :=>{}", quote!( let xx = #children));

        Ok(GTreeLayerStruct {
            id: layer_id,
            children,
        })
    }
}

impl ToTokens for GTreeLayerStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let GTreeLayerStruct { id, children } = self;
        let children_iter = children.iter();
        quote!(GTreeBuilderElement::GLayerTree(#id,vec![#(#children_iter),*])).to_tokens(tokens)
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

        println!("alayer");

        println!("gtree end");
        Ok(Gtree {  root })
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
        let Gtree {  root } = self;

        // fn pp<A: std::fmt::Debug>(layer: &NodeList<A>) {
        //     match layer {
        //         NodeList::LayerCons(id, list) => {
        //             println!("{:?}==>{:?}", id, list);
        //             list.iter().for_each(|l| pp(l));
        //         }
        //     };
        // }

            quote!({
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
                enum GTreeBuilderElement<'a, ToIx, Message>
                where
                    ToIx: ToString + std::fmt::Debug + Clone + std::fmt::Display,
                {
                    GLayerTree(ToIx, Vec<GTreeBuilderElement<'a, ToIx, Message>>),
                    El(Element<'a, Message>),
                }

                // • • • • •


                fn handle_root<'a, ToIx, Message>(
                    g: &mut GraphType<'a, Message>,
                    layer: &GTreeBuilderElement<'a, ToIx, Message>,
                ) where
                    Message: Clone + std::fmt::Debug,
                    ToIx: ToString + std::fmt::Debug + Clone + std::fmt::Display,
                {
                    match layer {
                        GTreeBuilderElement::GLayerTree(id, children_list) => {
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
                        GTreeBuilderElement::El(surface) => {
                            panic!("not allow this {:?}, first element must layer ", surface)
                        }
                    };
                }
                fn handle_layer<'a, ToIx, Message>(
                    g: &mut GraphType<'a, Message>,
                    layer: &GTreeBuilderElement<'a, ToIx, Message>,
                ) where
                    Message: Clone + std::fmt::Debug,
                    ToIx: ToString + std::fmt::Debug + Clone + std::fmt::Display,
                {
                    let parent_nix = illicit::expect::<emg_bind::NodeIndex<String>>();
                    match layer {
                        GTreeBuilderElement::GLayerTree(id, children_list) => {
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
                        GTreeBuilderElement::El(surface) => {
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




                println!(" {:#?}",&children);
                // pp(&children);
                log::info!("info==={:#?}",&children);


            }
            )
            .to_tokens(tokens)
       
    }
}

/// @ gtree_macro ────────────────────────────────────────────────────────────────────────────────
pub fn gtree_macro(item: TokenStream) -> Result<TokenStream, syn::Error> {
    let output = syn::parse2::<Gtree>(item)?;
    let token = output.to_token_stream();
    println!("token: {}", &token);
    Ok(token)
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
        let Gview{root_ix } = self;
        quote!({
            use emg_bind::{GraphType};
            GraphType::<Message>::view( #root_ix .to_string() )
    
        }).to_tokens(tokens)
    }
}
/// @ gview_macro ────────────────────────────────────────────────────────────────────────────────
pub fn gview_macro(item: TokenStream) -> Result<TokenStream, syn::Error> {
    let output = syn::parse2::<Gview>(item)?;
    let token = output.to_token_stream();
    println!("token: {}", &token);
    Ok(token)
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
        GraphType layer "a" [
            layer "b" [
                layer "c" [],
                layer "d" [],
                Text::new(format!("in quote..{}", id))
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

}
