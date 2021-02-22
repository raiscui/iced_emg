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
    syn::custom_keyword!(Refresher);

    // impl Debug for layer {
    //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //         f.write_str(concat!("Keyword [", stringify!(layer), "]"))
    //     }
    // }
}

// @ GClosure ────────────────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GTreeClosure {
    closure: syn::ExprClosure,
}
impl Parse for GTreeClosure {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        //println!("GSurface:{}", input);
        let ec = input.parse::<syn::ExprClosure>()?;
        if ec.inputs.is_empty() {
            Ok(GTreeClosure { closure: ec })
        } else {
            Err(input.error("closure argument must be empty"))
        }
    }
}
impl ToTokens for GTreeClosure {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let GTreeClosure { closure } = self;

        quote_spanned!(
            closure.span()=> GTreeBuilderElement::Cl(#closure)
        )
        .to_tokens(tokens)
    }
}
// @ GUpdater ────────────────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GRefresher {
    kws: kw::Refresher,
    closure: syn::ExprClosure,
}
impl Parse for GRefresher {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // println!("parsing GRefresher");
        // println!("{:?}", &input);

        // input.parse::<kw::Refresher>()?;
        Ok(GRefresher {
            kws: input.parse()?,
            closure: input.parse()?,
        })
    }
}
impl ToTokens for GRefresher {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let GRefresher { kws, closure } = self;

        let closure_token = quote_spanned!(
            closure.span()=> #closure
        );
        let kw_token = quote_spanned! (kws.span()=>GTreeBuilderElement::Updater(Rc::new(#kws::new(#closure_token))) );

        kw_token.to_tokens(tokens)
        // quote_spanned!(expr.span()=>GTreeBuilderElement::El(#expr.into())).to_tokens(tokens)
        // quote!(GTreeBuilderElement::El(#expr.into())).to_tokens(tokens)
    }
}

// @ GSurface ────────────────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GTreeSurface {
    expr: syn::Expr,
    children: ChildrenType,
}
impl Parse for GTreeSurface {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        //println!("GSurface:{}", input);
        let expr = input.parse::<syn::Expr>()?;
        if input.peek(token::FatArrow) {
            input.parse::<token::FatArrow>()?; //=>
                                               // []
            if input.peek(token::Bracket) {
                // println!("=>[] find");
                let content;
                let _bracket = bracketed!(content in input);
                let children: ChildrenType =
                    Some(content.parse_terminated(GTreeMacroElement::parse)?);
                Ok(GTreeSurface { expr, children })
            } else {
                Err(input.error("还没有完成 直接 单一 无[] 的后缀"))
            }
        } else {
            Ok(GTreeSurface {
                expr,
                children: None,
            })
        }
    }
}
impl ToTokens for GTreeSurface {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // self.expr.to_tokens(tokens)
        let GTreeSurface { expr, children } = self;
        // println!("expr===={:?}", self.expr);

        let children_iter = children.iter();
        let children_token = quote_spanned! {children.span()=>vec![#(#children_iter),*]};

        // TreeWhoWithUpdater
        quote_spanned! (expr.span() => GTreeBuilderElement::WhoWithUpdater(#expr,#children_token))
            .to_tokens(tokens)
    }
}

// @ GTreeElement ────────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum GTreeMacroElement {
    GL(GTreeLayerStruct),
    GS(GTreeSurface),
    RT(GRefresher),
    GC(GTreeClosure),
    // OtherExpr(syn::Expr),
}
impl Parse for GTreeMacroElement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // use syn::ext::IdentExt;

        if input.peek(kw::Layer) {
            Ok(GTreeMacroElement::GL(input.parse()?))
        } else if input.peek(kw::Refresher) {
            // println!("peek Refresher");
            Ok(GTreeMacroElement::RT(input.parse()?))
        } else if input.peek(token::Fn) && (input.peek2(Token![||]) || input.peek3(Token![||])) {
            Ok(GTreeMacroElement::GC(input.parse()?))
        } else if input.peek(Ident::peek_any) {
            Ok(GTreeMacroElement::GS(input.parse()?))
        } else {
            panic!("can't know what is");
            // Ok(GTreeMacroElement::OtherExpr(input.parse()?))
        }
    }
}

impl ToTokens for GTreeMacroElement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::GL(layer_struct) => layer_struct.to_tokens(tokens),
            Self::GS(surface) => surface.to_tokens(tokens),
            Self::RT(realtime_update_in) => realtime_update_in.to_tokens(tokens),
            Self::GC(closure) => closure.to_tokens(tokens),
            // Self::OtherExpr(expr) => expr.to_tokens(tokens),
        }
    }
}

// @ GTreeLayerStruct ────────────────────────────────────────────────────────────────────────────────
type ChildrenType = Option<Punctuated<GTreeMacroElement, Token![,]>>;
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
        let children: ChildrenType = Some(content.parse_terminated(GTreeMacroElement::parse)?);
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
            quote_spanned! {layer.span()=>GTreeBuilderElement::Layer};
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


            use std::ops::Deref;
            use emg_bind::{
                runtime::Element, runtime::Text, GElement, GraphStore, GraphType, Layer, RefreshFor,
                Refresher, Uuid,
            };
            use gtree::illicit;
            use gtree::log;
            use gtree::log::Level;
            use std::{cell::RefCell, rc::Rc};
            use GElement::*;




            #[allow(dead_code)]
            enum GTreeBuilderElement<'a, Message> {
                Layer(String, Vec<GTreeBuilderElement<'a, Message>>),
                El(Element<'a, Message>),
                WhoWithUpdater(GElement<'a, Message>, Vec<GTreeBuilderElement<'a, Message>>),
                Updater(Rc<dyn RefreshFor<GElement<'a, Message>>>),
                Cl(Box<dyn Fn()>),
            }
            impl<'a, Message> std::fmt::Debug for GTreeBuilderElement<'a, Message> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        GTreeBuilderElement::Layer(s, children_list) => f
                            .debug_tuple("GTreeBuilderElement::Layer")
                            .field(s)
                            .field(children_list)
                            .finish(),
                        GTreeBuilderElement::El(el) => {
                            f.debug_tuple("GTreeBuilderElement::El").field(el).finish()
                        }
                        GTreeBuilderElement::WhoWithUpdater(_, updaters) => {
                            let who = "dyn RefreshUseFor<GElement<'a, Message>>";
                            f.debug_tuple("GTreeBuilderElement::WhoWithUpdater")
                                .field(&who)
                                .field(updaters)
                                .finish()
                        }
                        GTreeBuilderElement::Updater(_) => {
                            let updater = "Rc<dyn RefreshFor<GElement<'a, Message>>>";
                            f.debug_tuple("GTreeBuilderElement::Updater")
                                .field(&updater)
                                .finish()
                        }
                        GTreeBuilderElement::Cl(_) => f.debug_tuple("GTreeBuilderElement::Cl").finish(),
                    }
                }
            }
            fn handle_root<'a, Message>(
                g: &mut GraphType<'a, Message>,
                tree_layer: &GTreeBuilderElement<'a, Message>,
            ) where
                Message: Clone + std::fmt::Debug,
            {
                match tree_layer {
                    GTreeBuilderElement::Layer(id, children_list) => {
                        log::debug!("{:?}==>{:?}", id.to_string(), children_list);
                        let nix = g.insert_node(
                            id.to_string(),
                            RefCell::new(Layer_(
                                Layer::new(id),
                            )),
                        );
                        illicit::Layer::new().offer(nix.clone()).enter(|| {
                            assert_eq!(
                                *illicit::expect::<emg_bind::NodeIndex<String>>(),
                                nix.clone()
                            );
                            log::debug!("{:?}", *illicit::expect::<emg_bind::NodeIndex<String>>());
                            children_list
                                .iter()
                                .for_each(|child_layer| handle_layer(g, child_layer));
                        });
                    }
                    _ => {
                        panic!("not allow this , first element must layer ")
                    }
                };
            }

            fn handle_layer<'a, Message>(
                g: &mut GraphType<'a, Message>,
                tree_layer: &GTreeBuilderElement<'a, Message>,
            ) where
                Message: Clone + std::fmt::Debug,
            {
                let parent_nix = illicit::expect::<emg_bind::NodeIndex<String>>();
                match tree_layer {
                    GTreeBuilderElement::Layer(id, children_list) => {
                        log::debug!("{:?}==>{:?}", id.to_string(), children_list);
                        let nix = g.insert_node(
                            id.to_string(),
                            RefCell::new(Layer_(
                                Layer::new(id),
                            )),
                        );
                        let edge = format!("{} -> {}", parent_nix.index(), nix.index());
                        log::debug!("{}", &edge);
                        g.insert_update_edge(parent_nix.deref(), &nix, edge);
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
                    GTreeBuilderElement::El(element) => {
                        let mut id = Uuid::new_v4()
                            .to_simple()
                            .encode_lower(&mut Uuid::encode_buffer())
                            .to_string();
                        id.push_str("-Element");
                        let nix = g.insert_node(id, RefCell::new(Element_(element.clone())));
                        let edge = format!("{} -> {}", parent_nix.index(), nix.index());
                        log::debug!("{}", &edge);
                        g.insert_update_edge(parent_nix.deref(), &nix, edge);
                    }
                    GTreeBuilderElement::WhoWithUpdater(gel, updaters) => {
                        let mut id = Uuid::new_v4()
                            .to_simple()
                            .encode_lower(&mut Uuid::encode_buffer())
                            .to_string();
                        id.push_str(format!("-{}", gel).as_ref());
                        let nix = g.insert_node(id, RefCell::new(gel.clone()));
                        let edge = format!("{} -> {}", parent_nix.index(), nix.index());
                        log::debug!("{}", &edge);
                        g.insert_update_edge(parent_nix.deref(), &nix, edge);
                        illicit::Layer::new().offer(nix.clone()).enter(|| {
                            assert_eq!(
                                *illicit::expect::<emg_bind::NodeIndex<String>>(),
                                nix.clone()
                            );
                            updaters
                                .iter()
                                .for_each(|child_layer| handle_layer(g, child_layer));
                        });
                    }
                    GTreeBuilderElement::Updater(u) => {
                        let mut id = Uuid::new_v4()
                            .to_simple()
                            .encode_lower(&mut Uuid::encode_buffer())
                            .to_string();
                        id.push_str("-Refresher");
                        let nix = g.insert_node(id, RefCell::new(Refresher_(Rc::clone(u))));
                        let edge = format!("{} -> {}", parent_nix.index(), nix.index());
                        log::debug!("{}", &edge);
                        g.insert_update_edge(parent_nix.deref(), &nix, edge);
                    }
                    GTreeBuilderElement::Cl(dyn_fn) => {
                        dyn_fn();
                    }
                };
            }



            // gtree::console_log::init_with_level(Level::Debug).ok();



            let children = #root;



            GraphType::<Message>::init();
            GraphType::<Message>::get_mut_graph_with(|g| {

                handle_root(g,&children);
                log::info!("{:#?}",g);
            });



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
            use emg_bind::GraphType;
            use emg_bind::GraphStore;
            GraphType::<Message>::view( #root_ix.to_string() )

            // G_STORE.with(|g_store_refcell| {
            //     // g_store_refcell.borrow_mut().set_graph(g);
            //     g_store_refcell
            //         .borrow_mut()
            //         .get_mut_graph_with(|g: &mut GraphType| {
            //             log::info!("graph==> {:#?}", &g);

            //             // Rc::make_mut(&mut Rc::clone(rc_e)).clone()
            //             // rc_e.clone().into()
            //             // Rc::make_mut(rc_e).clone().into()
            //             g.g_element_to_el(&cix)
            //         })
            // })
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
                Layer "d" [Refresher ||{Text_(Text::new(format!("ee up")))}],
                Text_(Text::new(format!("in quote..{}", "b"))) => [
                    Refresher ||{99},
                    Refresher ||{33}
                ]
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
