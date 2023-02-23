#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
// ────────────────────────────────────────────────────────────────────────────────
#![feature(is_some_and)]
// ────────────────────────────────────────────────────────────────────────────────
// use std::collections::HashSet as Set;

// use trace_var::trace_var;

use cassowary::Cassowary;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
// use quote::quote;
use syn::{bracketed, ext::IdentExt, punctuated::Punctuated, spanned::Spanned, token};
use syn::{
    parse::{discouraged::Speculative, Parse, ParseStream},
    Expr,
};

use syn::{Ident, Token};
// use uuid::Uuid;
use nanoid::nanoid;
use tracing::{debug, instrument};

// ────────────────────────────────────────────────────────────────────────────────
pub mod cassowary;
mod quote_option;
// use proc_macro::Diagnostic;
pub mod kw {
    // use std::fmt::Debug;
    #![warn(clippy::expl_impl_clone_on_copy)]

    syn::custom_keyword!(Layer);
    syn::custom_keyword!(ShapingUse);
    syn::custom_keyword!(On);
    syn::custom_keyword!(Event);
    syn::custom_keyword!(E);
    syn::custom_keyword!(Mod);
    syn::custom_keyword!(SkipInit);
    // syn::custom_keyword!(Dyn);

    // impl Debug for layer {
    //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //         f.write_str(concat!("Keyword [", stringify!(layer), "]"))
    //     }
    // }
}
//@ ID ──────────────────────────────
#[derive(Debug, Default)]
struct ID(Option<Expr>);

impl ID {
    pub fn get(&self, def_name: &str) -> TokenStream {
        self.0.as_ref().map_or_else(
            || {
                let id = make_id(def_name);

                // if id.len() <= 12usize {
                // quote!(IdStr::new_inline(#id))
                // } else {
                quote!(IdStr::new(#id))
                // }
            },
            |id| {
                // let id_string = id.to_string();
                // if id_string.len() <= 12usize {
                //     quote_spanned!(id.span()=>IdStr::new_inline(#id_string))
                // } else {
                //     quote_spanned!(id.span()=>IdStr::new(#id_string))
                // }

                // quote_spanned!(id.span()=>#id)
                // ─────────────────────────────────────────────

                if let Expr::Lit(lit) = id {
                    if let syn::Lit::Str(lit_str) = &lit.lit {
                        // let id_string = lit_str.value();
                        // if id_string.len() <= 12usize {
                        // quote_spanned!(id.span()=>IdStr::new_inline(#id_string))
                        // } else {
                        // quote_spanned!(id.span()=>IdStr::new(#id_string))
                        quote_spanned!(id.span()=>IdStr::new(#lit_str))
                        // }
                    } else {
                        quote_spanned!(id.span()=>IdStr::from(#id))
                    }
                } else {
                    quote_spanned!(id.span()=>IdStr::from(#id))
                }
            },
        )
    }
}

fn make_id(name: &str) -> String {
    // let mut id = nanoid!(8);
    let mut id = name.to_string();
    // let mut id = (*Uuid::new_v4()
    //     .to_simple()
    //     .encode_lower(&mut Uuid::encode_buffer()))
    // .to_string();
    if let Some(n) = (12usize - 1usize).checked_sub(id.len()) {
        id.push('-');
        id.push_str(nanoid!(n).as_str());
    }

    id
}

//@ @Parse ──────────────────────────────

// type OptEdge = Option<Edge>;
// the "@"
#[derive(Debug)]
enum At {
    Id(ID),
    Edge(Edge),
    Mod,
    SkipInit,
}

impl From<Edge> for At {
    fn from(v: Edge) -> Self {
        Self::Edge(v)
    }
}

impl From<ID> for At {
    fn from(v: ID) -> Self {
        Self::Id(v)
    }
}

#[derive(Debug)]
struct AtList(Vec<At>);

// struct Edge(Option<syn::Expr>);
impl Parse for AtList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut at_list = vec![];
        while !input.is_empty() && input.peek(Token![@]) {
            input.parse::<Token![@]>()?;

            // println!("in at_list parse :{}", &input);

            if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                let id = input.parse::<Expr>()?;
                at_list.push(ID(Some(id)).into());
            } else if input.parse::<kw::E>().is_ok() {
                input.parse::<Token![=]>()?;
                at_list.push(input.parse::<Edge>()?.into());
            } else if input.parse::<kw::Mod>().is_ok() {
                at_list.push(At::Mod);
            } else if input.parse::<kw::SkipInit>().is_ok() {
                at_list.push(At::SkipInit);
            } else {
                return Err(
                    input.error(" '@' Keyword trailing content does not satisfy any matches")
                );
            }
        }
        Ok(Self(at_list))
    }
}

//@ Edge ──────────────────────────────

#[derive(Debug)]
enum EdgeObject {
    E(Box<Expr>),
    Cassowary(Box<Cassowary>),
}
impl ToTokens for EdgeObject {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::E(expr) => quote_spanned!(expr.span()=>#expr).to_tokens(tokens),
            Self::Cassowary(cassowary) => {
                quote_spanned!(cassowary.span()=>#cassowary).to_tokens(tokens);
            }
        };
    }
}
impl Parse for EdgeObject {
    #[instrument(name = "EdgeObject")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(token::Brace) {
            debug!("====== in EdgeObject peek-> {{}}, will parse cassowary... ");

            Ok(Self::Cassowary(input.parse()?))
        } else {
            Ok(Self::E(input.parse()?))
        }
    }
}

#[derive(Debug)]
struct Edge {
    bracket_token: token::Bracket,
    content: Punctuated<EdgeObject, Token![,]>,
    // content: Punctuated<syn::Expr, Token![,]>,
}
// struct Edge(Option<syn::Expr>);

impl Parse for Edge {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let bracket_token = bracketed!(content in input);
        debug!("======Edge-> will parse ()");

        let content: Punctuated<EdgeObject, Token![,]> =
            content.parse_terminated(EdgeObject::parse)?;
        // content.parse_terminated(syn::Expr::parse)?;
        debug!("content: {:?}", &content);
        debug!("");

        // Ok(Self {
        //         bracket_token,
        //         content
        //     })

        Ok(Self {
            bracket_token,
            content,
        })
    }
}
impl ToTokens for Edge {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            bracket_token,
            content,
        } = self;
        let content_iter = content.iter();
        //NOTE use Rc because dyn
        quote_spanned!(
            bracket_token.span=> vec![#(std::rc::Rc::new(#content_iter) as std::rc::Rc<dyn Shaping<EmgEdgeItem>>),*]
        )
        .to_tokens(tokens);
    }
}
// ────────────────────────────────────────────────────────────────────────────────

// @ GClosure ────────────────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug)]
pub struct GTreeClosure {
    id: ID,
    closure: syn::ExprClosure,
}
impl AtSetup for GTreeClosure {
    fn at_setup(&mut self, at_list: AtList) -> syn::Result<()> {
        for at in at_list.0 {
            match at {
                At::Id(id) => {
                    self.id = id;
                }
                At::Edge(_) => {
                    return syn::Result::Err(syn::Error::new(
                        self.span(),
                        "closure can't have any edge",
                    ));
                    // panic!("closure can't have any edge");
                }
                At::Mod => {
                    // panic!("closure can't be Mod");
                    return syn::Result::Err(syn::Error::new(self.span(), "closure can't be Mod"));
                }
                At::SkipInit => todo!(),
            }
        }
        syn::Result::Ok(())
    }
}
impl Parse for GTreeClosure {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let id = ID::default();

        let ec = input.parse::<syn::ExprClosure>()?;
        if ec.inputs.is_empty() {
            Ok(Self { id, closure: ec })
        } else {
            Err(input.error("closure argument must be empty"))
        }
    }
}
impl ToTokens for GTreeClosure {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { id, closure } = self;
        let id_token = id.get("Cl");

        quote_spanned!(
            closure.span()=> GTreeBuilderElement::Cl(#id_token,#closure)
        )
        .to_tokens(tokens);
    }
}

// @ G_On_Event ────────────────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug)]
pub struct GOnEvent {
    id: ID,
    event_name: Ident,
    closure: syn::ExprClosure,
}
impl AtSetup for GOnEvent {
    fn at_setup(&mut self, at_list: AtList) -> syn::Result<()> {
        for at in at_list.0 {
            match at {
                At::Id(id) => {
                    self.id = id;
                }
                At::Edge(_) => {
                    return syn::Result::Err(syn::Error::new(
                        self.span(),
                        "On:Event can't have any edge",
                    ));
                    // panic!("On:Event can't have any edge");
                }
                At::Mod => {
                    return syn::Result::Err(syn::Error::new(self.span(), "On:Event can't be Mod"));
                    // panic!("On:Event can't be Mod");
                }
                At::SkipInit => todo!(),
            }
        }
        syn::Result::Ok(())
    }
}
impl Parse for GOnEvent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let id = ID::default();

        input.parse::<kw::On>()?;
        input.parse::<Token![:]>()?;

        let event_name = input.parse::<Ident>()?;

        Ok(Self {
            id,
            event_name,
            closure: input.parse()?,
        })
    }
}
impl ToTokens for GOnEvent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            id,
            event_name,
            closure,
        } = self;
        let id_token = id.get(format!("Ev-{event_name}").as_str()); //just emg graph node id

        let token = if closure.inputs.is_empty() {
            quote_spanned! (closure.span()=> GTreeBuilderElement::Event(#id_token,EventMessage::new((#event_name).into(), #closure ).into()) )
        } else if closure.inputs.len() == 3 {
            quote_spanned! (closure.span()=>GTreeBuilderElement::Event(#id_token,EventCallback::new((#event_name).into(),std::rc::Rc::new(#closure)).into()) )
        } else {
            panic!("event callback argument size is must empty or three")
        };
        token.to_tokens(tokens);
    }
}
// @ GRefresher ────────────────────────────────────────────────────────────────────────────────
#[derive(Debug)]
pub enum RefresherType {
    Callback(syn::ExprClosure),
    Expr(Expr),
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct GShapingUse {
    id: ID,
    kws: kw::ShapingUse,
    method: RefresherType,
}
impl AtSetup for GShapingUse {
    fn at_setup(&mut self, at_list: AtList) -> syn::Result<()> {
        for at in at_list.0 {
            match at {
                At::Id(id) => {
                    self.id = id;
                }
                At::Edge(_) => {
                    return syn::Result::Err(syn::Error::new(
                        self.span(),
                        "@ShapingUse can't have any edge",
                    ));
                    // panic!("@ShapingUse can't have any edge");
                }
                At::Mod => {
                    return syn::Result::Err(syn::Error::new(
                        self.span(),
                        "@ShapingUse can't be Mod",
                    ));
                    // panic!("@ShapingUse can't be Mod");
                }
                At::SkipInit => todo!(),
            }
        }
        syn::Result::Ok(())
    }
}

impl Parse for GShapingUse {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let id = ID::default();
        let kws = input.parse::<kw::ShapingUse>()?;

        let fork = input.fork();

        if fork.parse::<syn::ExprClosure>().is_ok() {
            Ok(Self {
                id,
                kws,
                method: RefresherType::Callback(input.parse()?),
            })
        } else {
            let expr = input.parse::<Expr>()?;
            Ok(Self {
                id,
                kws,
                method: RefresherType::Expr(expr),
            })
        }
    }
}

impl ToTokens for GShapingUse {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { id, kws, method } = self;

        let kw_token = match method {
            RefresherType::Callback(callback) => {
                let closure_token = callback.into_token_stream();
                let id_token = id.get("ShapingUseShaper");

                quote_spanned! (kws.span()=>GTreeBuilderElement::#kws(#id_token,std::rc::Rc::new(Shaper::new(#closure_token)) as std::rc::Rc<dyn EqShaping<GElement<Message>>>) )
            }
            RefresherType::Expr(expr) => {
                let expr_token = expr.into_token_stream();
                let id_token = id.get("ShapingUse");
                quote_spanned! (kws.span()=>GTreeBuilderElement::#kws(#id_token,std::rc::Rc::new(#expr_token) as std::rc::Rc<dyn EqShaping<GElement<Message>>>) )
            }
        };

        // let kw_token = quote_spanned! (kws.span()=>GTreeBuilderElement::ShapingUse(#id_token,Rc::new(#kws::new(#closure_token))) );

        kw_token.to_tokens(tokens);
    }
}

// @ GSurface ────────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct SaGel {
    pub left: Box<Expr>,
    pub _map_fn_token: token::FatArrow,
    pub right: Box<syn::ExprClosure>,
}

impl Parse for SaGel {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            left: input.parse()?,
            _map_fn_token: input.parse()?,
            right: input.parse()?,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct GTreeSurface {
    edge: Option<Edge>,
    id: ID,
    module: bool,
    opt_expr: Option<Expr>,
    opt_sa_gel: Option<SaGel>,
    children: ChildrenType,
    skip_init: bool,
}

impl AtSetup for GTreeSurface {
    /// setup the @ mark
    fn at_setup(&mut self, at_list: AtList) -> syn::Result<()> {
        for at in at_list.0 {
            match at {
                At::Id(id) => {
                    self.id = id;
                }
                At::Edge(edge) => {
                    self.edge = Some(edge);
                }
                At::Mod => {
                    self.module = true;
                }
                At::SkipInit => {
                    self.skip_init = true;
                }
            }
        }
        syn::Result::Ok(())
    }
}

impl Parse for GTreeSurface {
    #[allow(clippy::non_ascii_literal)]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let edge = None;
        let id = ID::default();
        let module = false;

        let fork2 = input.fork();

        let opt_sa_gel = fork2.parse::<SaGel>().ok().map_or_else(
            || None,
            |sa_gel| {
                input.advance_to(&fork2);
                Some(sa_gel)
            },
        );

        let opt_expr = if opt_sa_gel.is_none() {
            Some(input.parse::<Expr>()?)
        } else {
            None
        };

        if input.peek(token::FatArrow) {
            // println!("has fa");

            input.parse::<token::FatArrow>()?; //=>
                                               // []
                                               //                                    let fork3 = input.fork().to_string();

            // let f = quote!{
            //     #fork3
            // };

            if input.peek(token::Bracket) {
                // println!("=>[] find");
                let content;
                let _bracket = bracketed!(content in input);
                let children: ChildrenType =
                    Some(content.parse_terminated(GTreeMacroElement::parse)?);
                Ok(Self {
                    edge,
                    id,
                    module,
                    opt_expr,
                    opt_sa_gel,
                    children,
                    skip_init: false,
                })
            } else {
                // panic!("还没有完成 直接 单一 无[] 的后缀.. {}",&f)
                panic!("还没有完成 直接 单一 无[] 的后缀.")
            }
        } else {
            Ok(Self {
                edge,
                id,
                module,
                opt_expr,
                opt_sa_gel,
                children: None,
                skip_init: false,
            })
        }
    }
}
impl ToTokens for GTreeSurface {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // self.expr.to_tokens(tokens)
        let Self {
            edge,
            id,
            module,
            opt_expr,
            opt_sa_gel,
            children,
            skip_init,
        } = self;
        // println!("expr===={:?}", self.expr);
        let edge_token = edge2token(edge);

        let children_iter = children.iter();
        let children_token = quote_spanned! {children.span()=>vec![#(#children_iter),*]};

        // Tree GElementTree
        //TODO namespace ,slot

        if *module {
            let id_token = id.get("GTM");

            let expr = opt_expr.as_ref().unwrap();

            quote_spanned! (expr.span() =>
                // let exp_v:GTreeBuilderElement<_,_> = ;

                match #expr{
                    GTreeBuilderElement::Layer( expr_id,mut expr_edge,expr_children) =>{
                        //TODO maybe change Ord to   expr_id, #id_token,
                        let new_id =format!("{}|{}", #id_token, expr_id);
                        expr_edge.extend( #edge_token);
                        // let new_children = expr_children.
                        GTreeBuilderElement::Layer(IdStr::new(new_id),expr_edge,expr_children)
                    }
                    // GTreeBuilderElement::El(expr_id, el)=>{
                    //     let new_id =format!("{}|{}", #id_token, expr_id);
                    //     GTreeBuilderElement::El(new_id, el)
                    // },
                    GTreeBuilderElement::GElementTree(
                        expr_id,
                        mut expr_edge,
                        ge,
                        expr_children
                    )=>{
                        let new_id =format!("{}|{}", #id_token, expr_id);
                        expr_edge.extend( #edge_token);
                        GTreeBuilderElement::GElementTree(
                            IdStr::new(new_id),
                            expr_edge,
                            ge,
                            expr_children
                        )
                    }

                    GTreeBuilderElement::Dyn(
                        _expr_id, //NOTE if use from , allways "" (default)
                        mut expr_edge,
                        x
                    ) =>{
                        // let new_id =format!("{}|{}", #id_token, expr_id);

                        expr_edge.extend( #edge_token);
                        debug!("dyn:::: {}",&{#id_token});

                        GTreeBuilderElement::Dyn(#id_token,expr_edge,x)
                    }

                    _=>{
                    panic!("不能转换元件表达式到 Layer: {:?}",&#expr);

                    }
                }



            )
            .to_tokens(tokens);
        } else {
            let id_token = id.get("GEl");

            match (opt_sa_gel, opt_expr) {
                (None, None) | (Some(_), Some(_)) => unreachable!(),
                (None, Some(expr)) => {
                    //NOTE Sa 不带后缀 也会转换 为 gel, = InsideUseSa_(StateAnchor<Self>),需要预处理掉
                    if *skip_init {
                        quote_spanned! (expr.span() =>
                            GTreeBuilderElement::GElementTree(#id_token,#edge_token,{#expr}.into(),#children_token)
                        )
                        .to_tokens(tokens);
                    } else {
                        //NOTE is builder (default case)
                        //TODO 其他各个元素处理 skip_init

                        quote_spanned! (expr.span() =>
                        {
                            let id = #id_token;
                            let edges = #edge_token;
                            let children = #children_token;
                            #expr
                            .tree_init_calling(&id,&edges,&children)
                            .with_id_edge_children(id,Some(edges),Some(children))
                        }
                        )
                        .to_tokens(tokens);
                    }
                }
                (Some(sa_gel_func), None) => {
                    let sa_gel = &sa_gel_func.left;
                    let sa_fn = &sa_gel_func.right;

                    quote_spanned! (sa_gel.span() =>

                            GTreeBuilderElement::GElementTree(#id_token.into(),#edge_token,
                                emg_bind::SaWithMapFn::new(#sa_gel,std::rc::Rc::new(#sa_fn)).into()
                                // Rc::new(move |parent_sa|{
                                //     (parent_sa,&#sa_gel).map(#sa_fn)
                                // })
                            ,#children_token)
                    )
                    .to_tokens(tokens);
                }
            };
        }
    }
}

// // @ DynObjTree ────────────────────────────────────────────────────────────────────────────────

// #[allow(dead_code)]
// #[derive(Debug)]
// pub struct DynObjTree {
//     edge: Option<Edge>,
//     id: ID,
//     module: bool,
//     expr: syn::Expr,
//     children: ChildrenType,
// }

// impl AtSetup for DynObjTree {
//     /// setup the @ mark
//     fn at_setup(&mut self, at_list: AtList) {
//         for at in at_list.0 {
//             match at {
//                 At::Id(id) => {
//                     self.id = id;
//                 }
//                 At::Edge(edge) => {
//                     self.edge = Some(edge);
//                 }
//                 At::Mod => {
//                     self.module = true;
//                 }
//             }
//         }
//     }
// }

// impl Parse for DynObjTree {
//     #[allow(clippy::non_ascii_literal)]
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         let edge = None;
//         let id = ID::default();
//         let module = false;

//         //println!("GSurface:{}", input);
//         let expr = input.parse::<syn::Expr>()?;
//         if input.peek(token::FatArrow) {
//             input.parse::<token::FatArrow>()?; //=>
//                                                // []
//             if input.peek(token::Bracket) {
//                 // println!("=>[] find");
//                 let content;
//                 let _bracket = bracketed!(content in input);
//                 let children: ChildrenType =
//                     Some(content.parse_terminated(GTreeMacroElement::parse)?);
//                 Ok(Self {
//                     edge,
//                     id,
//                     module,
//                     expr,
//                     children,
//                 })
//             } else {
//                 panic!("还没有完成 直接 单一 无[] 的后缀")
//             }
//         } else {
//             Ok(Self {
//                 edge,
//                 id,
//                 module,
//                 expr,
//                 children: None,
//             })
//         }
//     }
// }
// impl ToTokens for DynObjTree {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         // self.expr.to_tokens(tokens)
//         let DynObjTree {
//             edge,
//             id,
//             module,
//             expr,
//             children,
//         } = self;
//         // println!("expr===={:?}", self.expr);
//         let edge_token = edge2token(edge);

//         let children_iter = children.iter();
//         let children_token = quote_spanned! {children.span()=>vec![#(#children_iter),*]};
//         let id_token = id.get("Generic");

//         // Tree GElementTree
//         //TODO namespace ,slot

//         if *module {
//             quote_spanned! (expr.span() =>

//                 match #expr{
//                     GTreeBuilderElement::Layer( expr_id,mut expr_edge,expr_children) =>{
//                         let new_id =format!("{}|{}", #id_token, expr_id);
//                         expr_edge.extend( #edge_token);
//                         // let new_children = expr_children.
//                         GTreeBuilderElement::Layer(new_id,expr_edge,expr_children)
//                     }
//                     GTreeBuilderElement::El(expr_id, el)=>{
//                         let new_id =format!("{}|{}", #id_token, expr_id);
//                         GTreeBuilderElement::El(new_id, el)
//                     },
//                     GTreeBuilderElement::GElementTree(
//                         expr_id,
//                         mut expr_edge,
//                         ge,
//                         expr_children
//                     )=>{
//                         let new_id =format!("{}|{}", #id_token, expr_id);
//                         expr_edge.extend( #edge_token);
//                         GTreeBuilderElement::GElementTree(
//                             new_id,
//                             expr_edge,
//                             ge,
//                             expr_children
//                         )
//                     }

//                     _=>{
//                     panic!("不能转换元件表达式到 Layer");

//                     }
//                 }

//             )

//             .to_tokens(tokens);
//         } else {
//             quote_spanned! (expr.span() => {
//                 let dyn_gel = #expr;
//                 let type_name = dyn_gel.type_name();
//                 let id_token_end = format!("{}-{}",#id_token,type_name);
//                 GTreeBuilderElement::GenericTree(id_token_end,#edge_token,Box::new(dyn_gel),#children_token)
//             } )

//              .to_tokens(tokens);
//         }
//     }
// }

// @ GTreeMacroElement ────────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
enum GTreeMacroElement {
    GL(GTreeLayerStruct),
    GS(Box<GTreeSurface>),
    RT(Box<GShapingUse>),
    GC(GTreeClosure),
    OnEvent(GOnEvent),
    // GT(Box<DynObjTree>) // OtherExpr(syn::Expr),
}

impl Parse for GTreeMacroElement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // println!("at list");
        let at_list = input.parse::<AtList>()?;
        // check id second

        if input.peek(kw::Layer) {
            //@layer
            let mut parsed: GTreeLayerStruct = input.parse()?;
            parsed.at_setup(at_list)?;
            Ok(Self::GL(parsed))
            // ─────────────────────────────────────────────────────────────────

            // }else if input.peek(kw::Dyn) {
            //     //@ Dyn
            //     let mut parsed: DynObjTree = input.parse()?;
            //     parsed.at_setup(at_list);
            //     Ok(Self::GT(Box::new(parsed)))
        } else if input.peek(kw::ShapingUse) {
            // @shaper
            let mut parsed: GShapingUse = input.parse()?;
            parsed.at_setup(at_list)?;
            Ok(Self::RT(Box::new(parsed)))
        } else if input.peek(token::Fn) && (input.peek2(Token![||]) || input.peek3(Token![||])) {
            // @closure
            let mut parsed: GTreeClosure = input.parse()?;
            parsed.at_setup(at_list)?;
            Ok(Self::GC(parsed))
        } else if input.peek(kw::On) && (input.peek2(Token![:])) {
            //@ On:Event
            let mut parsed: GOnEvent = input.parse()?;
            parsed.at_setup(at_list)?;
            Ok(Self::OnEvent(parsed))
        }
        //  must on bottom ─────────────────────────────────────────────────────────────────
        else if input.peek(Ident::peek_any) {
            // @surface  expr, GElement
            let mut parsed: GTreeSurface = input.parse()?;
            parsed.at_setup(at_list)?;
            Ok(Self::GS(Box::new(parsed)))
        } else {
            return syn::Result::Err(syn::Error::new(
                input.span(),
                format!("can't know what is , input current:{input}"),
            ));
            // panic!("can't know what is , input current:{}", input);
            // Ok(Self::OtherExpr(input.parse()?))
        }
    }
}

impl ToTokens for GTreeMacroElement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use match_any::match_any;

        match_any!( self ,
            Self::GL(x)|Self::GS(x)|Self::RT(x)|Self::GC(x)|Self::OnEvent(x)
            // |Self::GT(x)
            => x.to_tokens(tokens)
        );
    }
}
trait AtSetup: Sized {
    fn at_setup(&mut self, at_list: AtList) -> syn::Result<()>;
}

// @ GTreeLayerStruct ────────────────────────────────────────────────────────────────────────────────
type ChildrenType = Option<Punctuated<GTreeMacroElement, Token![,]>>;
#[allow(dead_code)]
#[derive(Debug)]
pub struct GTreeLayerStruct {
    edge: Option<Edge>,
    layer: kw::Layer,
    id: ID,
    children: ChildrenType,
    skip_init: bool,
}

impl AtSetup for GTreeLayerStruct {
    fn at_setup(&mut self, at_list: AtList) -> syn::Result<()> {
        for at in at_list.0 {
            match at {
                At::Id(id) => {
                    self.id = id;
                }
                At::Edge(edge) => {
                    self.edge = Some(edge);
                }
                At::Mod => {
                    return syn::Result::Err(syn::Error::new(self.span(), "layer can't be Mod"));
                    // panic!("layer can't be Mod");
                }
                At::SkipInit => {
                    self.skip_init = true;
                }
            }
        }
        syn::Result::Ok(())
    }
}

impl Parse for GTreeLayerStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // println!("at list");

        let edge = None;

        let id = ID::default();

        let layer = input.parse::<kw::Layer>()?;

        if input.peek(Token![,]) {
            //提前结束 , 没有[]
            // if input.is_empty() {
            return Ok(Self {
                edge,
                layer,
                id,
                children: None,
                skip_init: false,
            });
        }

        //println!("input {:?}", &input);
        let content;
        let _bracket = bracketed!(content in input);
        //println!("brace_token=>{:?}", &bracket);
        let children: ChildrenType = Some(content.parse_terminated(GTreeMacroElement::parse)?);
        //println!("children:=>{:?}", &children);
        // println!("children op :=>{}", quote!(  #children));

        Ok(Self {
            edge,
            layer,
            id,
            children,
            skip_init: false,
        })
    }
}

fn edge2token(edge: &Option<Edge>) -> TokenStream {
    edge.as_ref().map_or_else(|| quote!(vec![]), |e| quote!(#e))
}

impl ToTokens for GTreeLayerStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            edge,
            layer,
            id,
            children,
            skip_init,
        } = self;

        let id_token = id.get("L");
        let edge_token = edge2token(edge);
        let children_iter = children.iter();
        let children_token = quote_spanned! {children.span()=>vec![#(#children_iter),*]};
        let layer_token = quote_spanned! {layer.span()=>Layer};

        if *skip_init {
             quote! (
                GTreeBuilderElement::GElementTree(#id_token,#edge_token,#layer_token ::new(#id_token).into(),#children_token)
            )
        } else {
            quote! ({
                let id = #id_token;
                let edges = #edge_token;
                let children = #children_token;
                #layer_token ::new(id.clone())
                .tree_init_calling(&id,&edges,&children)
                .with_id_edge_children(id,Some(edges),Some(children))
            })
        } .to_tokens(tokens);
        // quote!(GTreeBuilderElement::#layer(String::from(#id),vec![#(#children_iter),*])).to_tokens(tokens)
    }
}

// @ Gtree ────────────────────────────────────────────────────────────────────────────────
#[derive(Debug)]
pub struct Gtree {
    root: GTreeMacroElement,
}

impl Parse for Gtree {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let root = input.parse::<GTreeMacroElement>()?;

        // for at in at_list.0 {
        //     match at {
        //         At::Id(id) => {
        //             root.id = id;
        //         }
        //         At::Edge(_) => {
        //             panic!("root layer can't have any Edge");
        //         }
        //         At::Mod => {
        //             panic!("root can't be Mod");
        //         }
        //     }
        // }
        // Ok(Gtree { emg_graph, root })

        if !input.is_empty() {
            let err = format!("input has unknown tokens not parsed: {input} ",);

            return Err(input.error(err));
        }

        Ok(Self { root })
    }
}

impl ToTokens for Gtree {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { root } = self;

        let token = quote_spanned! {root.span()=> {

            // use crate::gtree_macro_prelude::*;

             #root



        }};
        token.to_tokens(tokens);
    }
}

// @ gtree_macro ────────────────────────────────────────────────────────────────────────────────
/// # Errors
///
/// Will return `Err` if can parse to Gtree
pub fn gtree_macro(item: TokenStream) -> Result<TokenStream, syn::Error> {
    let output = syn::parse2::<Gtree>(item)?;
    Ok(quote! (#output))
}

// #[derive(Debug, Clone)]
// struct EmgArgs {
//     vars: Set<Ident>,
//     first_life_time: Option<Lifetime>,
// }
// impl EmgArgs {
//     fn has_init_var(&self) -> bool {
//         self.vars.contains(&Ident::new("init", Span::call_site()))
//     }
// }
// impl Parse for EmgArgs {
//
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         let vars = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
//         Ok(EmgArgs {
//             vars: vars.into_iter().collect(),
//             first_life_time: None,
//         })
//     }
// }
// impl Fold for EmgArgs {
//     // fn fold_field(&mut self, field: Field) -> Field {
//     //     // let o: Field = parse_quote! {#i};
//     //     println!("===Field: {:#?}", &field);
//     //     fold::fold_field(self, field)
//     // }
//     fn fold_fields_named(&mut self, i: FieldsNamed) -> FieldsNamed {
//         let FieldsNamed {
//             brace_token: _,
//             named,
//         } = &i;
//         let field = named.iter();
//         // println!("---->{}", quote! {#named});
//         let lifetime = self.first_life_time.as_ref().unwrap();

//         parse_quote!({#(#field),* ,emg_graph:emg_bind::GraphType<#lifetime,Message>})
//         // fold::fold_fields_named(self, i)
//     }
// }

// /// @ emg_macro ────────────────────────────────────────────────────────────────────────────────
// pub fn emg_macro(args: TokenStream, input: TokenStream) -> Result<TokenStream, syn::Error> {
//     let args = syn::parse2::<EmgArgs>(args)?;
//     println!("has_init_var? {:?}", args.has_init_var());

//     let input = syn::parse2::<ItemStruct>(input)?;
//     // ────────────────────────────────────────────────────────────────────────────────

//     let o = emg_handle(args, input);
//     // ────────────────────────────────────────────────────────────────────────────────

//     Ok(quote!(

//         #o

//     ))
// }

// fn emg_handle(mut args: EmgArgs, input: ItemStruct) -> ItemStruct {
//     let mut need_add_lifetime = false;
//     if args.first_life_time.is_none() {
//         let first_lifetime = input.generics.lifetimes().next();
//         need_add_lifetime = first_lifetime.is_none();

//         args.first_life_time = first_lifetime
//             .map(|l_def| &l_def.lifetime)
//             .cloned()
//             .or_else(|| Some(Lifetime::new("'a", Span::call_site())));
//     };
//     println!("=====first_life_time:{:?}", &args.first_life_time);
//     let mut o = args.fold_item_struct(input);
//     if need_add_lifetime {
//         o.generics
//             .params
//             .push(syn::GenericParam::Lifetime(LifetimeDef::new(
//                 args.first_life_time.unwrap(),
//             )))
//     }
//     o
// }

// ────────────────────────────────────────────────────────────────────────────────
// @ Gview ────────────────────────────────────────────────────────────────────────────────

// #[allow(dead_code)]
// #[derive(Debug, Clone)]
// pub struct Gview {
//     root_ix: syn::LitStr,
// }
// impl Parse for Gview {
//
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         Ok(Gview {
//             root_ix: input.parse()?,
//         })
//     }
// }
// impl ToTokens for Gview {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         // self.expr.to_tokens(tokens)
//         let Gview { root_ix } = self;
//         quote!({
//             use emg_bind::GraphType;
//             use emg_bind::GraphStore;
//             GraphType::<Message>::view( #root_ix.to_string() )

//             // G_STORE.with(|g_store_refcell| {
//             //     // g_store_refcell.borrow_mut().set_graph(g);
//             //     g_store_refcell
//             //         .borrow_mut()
//             //         .get_mut_graph_with(|g: &mut GraphType| {
//             //             log::info!("graph==> {:#?}", &g);

//             //             // Rc::make_mut(&mut Rc::clone(rc_e)).clone()
//             //             // rc_e.clone().into()
//             //             // Rc::make_mut(rc_e).clone().into()
//             //             g.g_element_to_el(&cix)
//             //         })
//             // })
//         })
//         .to_tokens(tokens)
//     }
// }

// @ test ────────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_new_skip_init() {
        fn token_test(input: &str) {
            let parse_str = syn::parse_str::<Gtree>(input);

            match parse_str {
                Ok(ok) => {
                    #[cfg(feature = "insta")]
                    insta::assert_display_snapshot!("test_new_skip_init", ok.to_token_stream());
                    println!("===>{}", ok.to_token_stream());
                }
                Err(error) => println!("...{error:?}"),
            }
        }

        println!();
        let input = r#"
        @="root"
                @SkipInit Layer []
        "#;

        token_test(input);
        println!();
    }
    #[test]
    fn test_new_not_builder() {
        fn token_test(input: &str) {
            let parse_str = syn::parse_str::<Gtree>(input);

            match parse_str {
                Ok(ok) => {
                    #[cfg(feature = "insta")]
                    insta::assert_display_snapshot!("test_new_not_builder", ok.to_token_stream());
                    println!("===>{}", ok.to_token_stream());
                }
                Err(error) => println!("...{error:?}"),
            }
        }

        println!();
        let input = r#"
        @="root"
            Checkbox::new(false,"abcd",|_|Message::IncrementPressed) => [
            ]
        "#;

        token_test(input);
        println!();
    }

    #[test]
    fn test_vfl_1() {
        fn token_test(input: &str) {
            match syn::parse_str::<Gtree>(input) {
                Ok(ok) => {
                    #[cfg(feature = "insta")]
                    insta::assert_display_snapshot!("test_vfl_1", ok.to_token_stream());
                    println!("===>{}", ok.to_token_stream());
                }
                Err(error) => println!("...{error:?}"),
            }
        }

        println!();
        let input = r#"
        @=root
                Layer [
                    @=x111x @E=[{@h (#b1)(#b2)},h(px(11))]
                    Layer []
                ]
        "#;

        token_test(input);
        println!();
    }

    #[test]
    fn test_vfl_2() {
        fn token_test(input: &str) {
            match syn::parse_str::<Gtree>(input) {
                Ok(ok) => println!("===>{}", ok.to_token_stream()),
                Err(error) => println!("...{error:?}"),
            }
        }

        println!();
        let input = r#"
        @=root
                Layer [
                    @=b @E=[{@h (#b1)-[my_gap]-(#b2)-[my_other_gap]-(#b3)},
                    {"my_gap"==20,"my_other_gap"==88},
                    ]
                    Layer []
                ]
        "#;

        token_test(input);
        println!();
    }

    #[test]
    fn test_vfl_3() {
        #[allow(unused)]
        enum A {
            B,
            C,
        }
        fn token_test(input: &str) {
            match syn::parse_str::<Gtree>(input) {
                Ok(ok) => println!("===>{}", ok.to_token_stream()),
                Err(error) => println!("...{error:?}"),
            }
        }

        println!();
        let input = r#"
        @=root
                Layer [
                    @=x111x @E=[
                        {md==120},
                        {"nn":{
                                width==100,
                                height==20,
                            }
                        }
                        ]
                    Layer []
                ]
        "#;

        token_test(input);
        println!();

        // GTreeBuilderElement :: Layer (IdStr :: new_inline ("root") , vec ! [] , vec ! [
        //     GTreeBuilderElement :: Layer (IdStr :: new_inline ("x111x") , vec ! [
        //         Rc :: new (vec ! [
        //             emg_layout :: ccsa :: CassowaryVar :: General (
        //                 emg_layout :: ccsa :: GeneralVar (
        //                     emg_common :: IdStr :: new ("md") ,
        //                     emg_layout :: ccsa :: ScopeViewVariable :: new (
        //                         :: std :: option :: Option :: None ,
        //                         :: std :: option :: Option :: Some (
        //                             emg_layout :: ccsa :: NameCharsOrNumber :: Number (
        //                                 NotNan :: new (120 as f64) . unwrap ()
        //                             )
        //                         ) ,
        //                         :: std :: option :: Option :: None)
        //                 )
        //             )
        //         ]) as Rc < (dyn Shaping < EmgEdgeItem < _ >>) > ,

        //         Rc :: new (vec ! [
        //             emg_layout :: ccsa :: CassowaryVar :: Virtual (
        //                 emg_layout :: ccsa :: Virtual (
        //                     emg_common :: IdStr :: new ("nn") ,
        //                     vec ! [
        //                         emg_layout :: ccsa :: GeneralVar (
        //                             emg_common :: IdStr :: new ("width") ,
        //                             emg_layout :: ccsa :: ScopeViewVariable :: new (
        //                                 :: std :: option :: Option :: None ,
        //                                 :: std :: option :: Option :: Some (emg_layout :: ccsa :: NameCharsOrNumber :: Number (NotNan :: new (100 as f64) . unwrap ())) ,
        //                                 :: std :: option :: Option :: None
        //                             )
        //                         ) ,
        //                         emg_layout :: ccsa :: GeneralVar (
        //                             emg_common :: IdStr :: new ("height") ,
        //                             emg_layout :: ccsa :: ScopeViewVariable :: new (
        //                                 :: std :: option :: Option :: None ,
        //                                 :: std :: option :: Option :: Some (emg_layout :: ccsa :: NameCharsOrNumber :: Number (NotNan :: new (20 as f64) . unwrap ())) ,
        //                                 :: std :: option :: Option :: None
        //                             )
        //                         )
        //                     ]
        //                 )
        //             )
        //         ]) as Rc < (dyn Shaping < EmgEdgeItem < _ >>) >] ,
        //         vec ! [])]) ;
    }

    #[test]
    fn test_2() {
        fn token_test(input: &str) {
            match syn::parse_str::<Gtree>(input) {
                Ok(ok) => println!("===>{}", ok.to_token_stream()),
                Err(error) => println!("...{error:?}"),
            }
        }

        println!();
        let input = r#"
        @="root"
                Layer [
                    @="x111x" @E=[{@h |(button)...| in(#panel) gap(10)},h(px(11))]
                    Layer []
                ]
        "#;

        //TODO support this
        // token_test(input);
        println!();
    }
    #[test]
    fn test_id() {
        fn token_test(input: &str) {
            match syn::parse_str::<Gtree>(input) {
                Ok(ok) => println!("===>{}", ok.to_token_stream()),
                Err(error) => println!("...{error:?}"),
            }
        }

        println!();
        let input = r#"
        @=aa1 @E=[@h |(button)...| in(#panel) gap(10),h(px(11))]
                Layer [
                    Text::new(format!("aa1***********8"))=>[
                        ShapingUse dyn_v
                    ],
                    StateAnchor::constant(1) => |p,gel|p.clone() =>[

                    ]

                ]

        "#;

        token_test(input);
        println!();
    }

    // #[test]
    // fn test2() {
    //     fn token_test(input: &str) {
    //         match syn::parse_str::<Gview>(input) {
    //             Ok(ok) => println!("Gview===>{}", ok.to_token_stream()),
    //             Err(error) => println!("...{:?}", error),
    //         }
    //     }
    //     println!();
    //     // type GraphType = Vec<i32>;
    //     let input = r#" "a" "#;

    //     token_test(input);
    //     println!();
    // }
    // #[test]
    // fn emg_life() {
    //     let input: ItemStruct = syn::parse_quote!(
    //         struct AA<'f: 'b + 'c, 'b, 'c> {
    //             bb: String,
    //             cc: String,
    //         }
    //     );
    //     println!("====input:{:#?}", &input);

    //     let args: EmgArgs = EmgArgs {
    //         vars: Set::new(),
    //         first_life_time: None,
    //     };
    //     println!("has_init_var? {:?}", args.has_init_var());
    //     // ─────────────────────────────────────────────────────────────────
    //     let o = emg_handle(args, input);

    //     // ─────────────────────────────────────────────────────────────────

    //     println!("=======================");
    //     // println!("o: {:#?}", &o);
    //     println!("=======================");
    //     println!("{}", quote! {#o});
    // }
    // #[test]
    // fn emg_def_life() {
    //     let input: ItemStruct = syn::parse_quote!(
    //         struct AA<'f: 'b + 'c, 'b, 'c> {
    //             bb: String,
    //             cc: String,
    //         }
    //     );
    //     println!("====input:{:#?}", &input);

    //     let args: EmgArgs = EmgArgs {
    //         vars: Set::new(),
    //         first_life_time: None,
    //     };
    //     println!("has_init_var? {:?}", args.has_init_var());
    //     // ─────────────────────────────────────────────────────────────────
    //     let o = emg_handle(args, input);

    //     // ─────────────────────────────────────────────────────────────────

    //     println!("=======================");
    //     // println!("o: {:#?}", &o);
    //     println!("=======================");
    //     println!("{}", quote! {#o});
    // }
    // #[test]
    // fn emg_no_lifetime() {
    //     let input: ItemStruct = syn::parse_quote!(
    //         struct AA {
    //             bb: String,
    //             cc: String,
    //         }
    //     );
    //     println!("====input:{:#?}", &input);

    //     let args: EmgArgs = EmgArgs {
    //         vars: Set::new(),
    //         first_life_time: None,
    //     };
    //     println!("has_init_var? {:?}", args.has_init_var());
    //     // ─────────────────────────────────────────────────────────────────
    //     let o = emg_handle(args, input);
    //     // ─────────────────────────────────────────────────────────────────

    //     println!("=======================");
    //     // println!("o: {:#?}", &o);
    //     println!("=======================");
    //     println!("{}", quote! {#o});
    // }
}
