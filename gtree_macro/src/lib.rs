#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]

// use std::collections::HashSet as Set;

// use trace_var::trace_var;

use std::error::Error;

use cassowary::Cassowary;
use proc_macro2::{TokenStream, Span, Punct, Spacing};
use quote::{quote, quote_spanned, ToTokens};
// use quote::quote;
use syn::{parse::{Parse, ParseStream}, braced};
use syn::{bracketed, ext::IdentExt, punctuated::Punctuated, spanned::Spanned, token};

use syn::{Ident, Token};
// use uuid::Uuid;
use nanoid::nanoid;
use tracing::{debug, instrument};
// ────────────────────────────────────────────────────────────────────────────────
pub(crate) mod cassowary;
// use proc_macro::Diagnostic;
pub mod kw {
    // use std::fmt::Debug;
    #![warn(clippy::expl_impl_clone_on_copy)]

    syn::custom_keyword!(Layer);
    syn::custom_keyword!(RefreshUse);
    syn::custom_keyword!(On);
    syn::custom_keyword!(Event);
    syn::custom_keyword!(E);
    syn::custom_keyword!(Mod);
    // syn::custom_keyword!(Dyn);

    // impl Debug for layer {
    //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //         f.write_str(concat!("Keyword [", stringify!(layer), "]"))
    //     }
    // }
}
//@ ID ──────────────────────────────
#[derive(Debug, Default)]
struct ID(Option<Ident>);

impl ID {
    pub fn get(&self, def_name: &str) -> TokenStream {
        self.0.as_ref().map_or_else(
            || {
                let id = make_id(def_name);
                // println!("id:{}", &id);

                // quote!(String::from(#id))
                if id.len() <= 12usize{
                    quote!(IdStr::new_inline(#id))

                }else{
                    quote!(IdStr::new(#id))

                }
            },
            |id| {
                let id_string = id.to_string();
                // println!("id:{}", &id_string);

                // quote_spanned!(id.span()=>String::from(#id_string))
                if id_string.len() <= 12usize{
                    quote_spanned!(id.span()=>IdStr::new_inline(#id_string))
                }else{
                    quote_spanned!(id.span()=>IdStr::new(#id_string))

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
    if let Some(n) = (12usize-1usize).checked_sub(id.len()) {
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
                assert!(
                    input.peek2(Ident::peek_any),
                    "emg-gtree_macro: should not use Rust keyword "
                );
                // println!("is id");

                input.parse::<Token![=]>()?;
                let id = input.parse::<Ident>()?;
                at_list.push(ID(Some(id)).into());
            }
            if input.peek(kw::E) {
                // println!("is edge");

                input.parse::<kw::E>()?;
                input.parse::<Token![=]>()?;
                at_list.push(input.parse::<Edge>()?.into());
            }
            if input.peek(kw::Mod) {
                // println!("is edge");

                input.parse::<kw::Mod>()?;
                at_list.push(At::Mod);
            }
        }
        Ok(Self(at_list))
    }
}

//@ Edge ──────────────────────────────

#[derive(Debug)]
enum EdgeObject{
    E(Box<syn::Expr>),
    Cassowary(Box<Cassowary>)
}
impl Parse for EdgeObject{
    #[instrument(name = "EdgeObject")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(token::Brace){
        debug!("====== in EdgeObject peek-> {{}}, will parse cassowary... ");
            Ok(Self::Cassowary(input.parse()?))
        }else{
            Ok(Self::E(input.parse()?))
        }
      

        
        
    }
}



#[derive(Debug)]
struct Edge {
    bracket_token: token::Bracket,
    // content: Punctuated<EdgeObject, Token![,]>,
    content: Punctuated<syn::Expr, Token![,]>,
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
        debug!("content: {:?}",&content);
        debug!("");

        // Ok(Self {
        //         bracket_token,
        //         content
        //     })


        Ok(Self {
            bracket_token,
            content:Punctuated::<syn::Expr, Token![,]>::default(),
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
        quote_spanned!(
            bracket_token.span=> vec![#(Rc::new(#content_iter) as Rc<(dyn RefreshFor<EmgEdgeItem<_>>)>),*]
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
    fn at_setup(&mut self, at_list: AtList) {
        for at in at_list.0 {
            match at {
                At::Id(id) => {
                    self.id = id;
                }
                At::Edge(_) => {
                    panic!("closure can't have any edge");
                }
                At::Mod => {
                    panic!("closure can't be Mod");
                }
            }
        }
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
    event_name: String,
    closure: syn::ExprClosure,
}
impl AtSetup for GOnEvent {
    fn at_setup(&mut self, at_list: AtList) {
        for at in at_list.0 {
            match at {
                At::Id(id) => {
                    self.id = id;
                }
                At::Edge(_) => {
                    panic!("On:Event can't have any edge");
                }
                At::Mod => {
                    panic!("On:Event can't be Mod");
                }
            }
        }
    }
}
impl Parse for GOnEvent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let id = ID::default();

        input.parse::<kw::On>()?;
        input.parse::<Token![:]>()?;

        let event_name = input.parse::<Ident>()?.to_string();

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
        let id_token = id.get(format!("Ev-{}", event_name).as_str());//just emg graph node id

        let token = if closure.inputs.is_empty() {
            quote_spanned! (closure.span()=> GTreeBuilderElement::Event(#id_token,EventMessage::new(IdStr::new_inline(#event_name), #closure ).into()) )
        } else if closure.inputs.len() == 3 {
            quote_spanned! (closure.span()=>GTreeBuilderElement::Event(#id_token,EventCallback::new(IdStr::new_inline(#event_name),Rc::new(#closure)).into()) )
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
    Expr(syn::Expr),
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct GRefresher {
    id: ID,
    kws: kw::RefreshUse,
    method: RefresherType,
}
impl AtSetup for GRefresher {
    fn at_setup(&mut self, at_list: AtList) {
        for at in at_list.0 {
            match at {
                At::Id(id) => {
                    self.id = id;
                }
                At::Edge(_) => {
                    panic!("@RefreshUse can't have any edge");
                }
                At::Mod => {
                    panic!("@RefreshUse can't be Mod");
                }
            }
        }
    }
}

impl Parse for GRefresher {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let id = ID::default();
        let kws = input.parse::<kw::RefreshUse>()?;

        let fork = input.fork();

        if fork.parse::<syn::ExprClosure>().is_ok() {
            Ok(Self {
                id,
                kws,
                method: RefresherType::Callback(input.parse()?),
            })
        } else {
            let expr = input.parse::<syn::Expr>()?;
            Ok(Self {
                id,
                kws,
                method: RefresherType::Expr(expr),
            })
        }
    }
}

impl ToTokens for GRefresher {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { id, kws, method } = self;

        let kw_token = match method {
            RefresherType::Callback(callback) => {
                let closure_token = quote_spanned!(
                    callback.span()=> #callback
                );
                let id_token = id.get("Refresh");

                quote_spanned! (kws.span()=>GTreeBuilderElement::#kws(#id_token,Rc::new(Refresher::new(#closure_token)) as Rc<dyn EqRefreshFor<GElement<Message>>>) )
            }
            RefresherType::Expr(expr) => {
                let expr_token = quote_spanned!(
                    expr.span()=> #expr
                );
                let id_token = id.get("Refresh");
                quote_spanned! (kws.span()=>GTreeBuilderElement::#kws(#id_token,Rc::new(#expr_token) as Rc<dyn EqRefreshFor<GElement<Message>>>) )
            }
        };

        // let kw_token = quote_spanned! (kws.span()=>GTreeBuilderElement::RefreshUse(#id_token,Rc::new(#kws::new(#closure_token))) );

        kw_token.to_tokens(tokens);
    }
}

// @ GSurface ────────────────────────────────────────────────────────────────────────────────


#[derive(Debug, Clone)]
struct SaGel {
    pub left: Box<syn::Expr>,
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
    opt_expr: Option<syn::Expr>,
    opt_sa_gel:Option<SaGel>,
    children: ChildrenType,
}

impl AtSetup for GTreeSurface {
    /// setup the @ mark
    fn at_setup(&mut self, at_list: AtList) {
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
            }
        }
    }
}

impl Parse for GTreeSurface {
    #[allow(clippy::non_ascii_literal)]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let edge = None;
        let id = ID::default();
        let module = false;

        let fork2 = input.fork();
       

        let opt_sa_gel = if fork2.parse::<SaGel>().is_ok(){
            let sa_gel:SaGel = input.parse()?;
           Some (sa_gel)
        }else{
            None
        };

       
        let opt_expr = if opt_sa_gel.is_none(){
             Some(input.parse::<syn::Expr>()?)
            }else{
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

            match (opt_sa_gel,opt_expr){
                (None, None)| (Some(_), Some(_)) => unreachable!(),
                (None, Some(expr)) => {
                    //NOTE Sa 不带后缀 也会转换 为 gel, = InsideUseSa_(StateAnchor<Self>),需要预处理掉
                    quote_spanned! (expr.span() => 
                    GTreeBuilderElement::GElementTree(#id_token,#edge_token,{#expr}.into(),#children_token)
                    )
                    .to_tokens(tokens);
                },
                (Some(sa_gel_func), None) => {
                    let sa_gel = &sa_gel_func.left;
                    let sa_fn = &sa_gel_func.right;

                    quote_spanned! (sa_gel.span() => 

                            GTreeBuilderElement::GElementTree(#id_token,#edge_token,
                                emg_bind::SaWithMapFn::new(#sa_gel,Rc::new(#sa_fn)).into()
                                // Rc::new(move |parent_sa|{
                                //     (parent_sa,&#sa_gel).map(#sa_fn)
                                // })
                            ,#children_token)
                    )
                    .to_tokens(tokens);

                },
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
    RT(Box<GRefresher>),
    GC(GTreeClosure),
    OnEvent(GOnEvent),
    // GT(Box<DynObjTree>) // OtherExpr(syn::Expr),
}

impl Parse for GTreeMacroElement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // println!("at list");
        let at_list = input.parse::<AtList>()?;
        // check id second
        // println!("check kw");

        if input.peek(kw::Layer) {
            //@layer
            let mut parsed: GTreeLayerStruct = input.parse()?;
            parsed.at_setup(at_list);
            Ok(Self::GL(parsed))
            // ─────────────────────────────────────────────────────────────────

            // }else if input.peek(kw::Dyn) {
            //     //@ Dyn
            //     let mut parsed: DynObjTree = input.parse()?;
            //     parsed.at_setup(at_list);
            //     Ok(Self::GT(Box::new(parsed)))
        } else if input.peek(kw::RefreshUse) {
            // @refresher
            let mut parsed: GRefresher = input.parse()?;
            parsed.at_setup(at_list);
            Ok(Self::RT(Box::new(parsed)))
        } else if input.peek(token::Fn) && (input.peek2(Token![||]) || input.peek3(Token![||])) {
            // @closure
            let mut parsed: GTreeClosure = input.parse()?;
            parsed.at_setup(at_list);
            Ok(Self::GC(parsed))
        } else if input.peek(kw::On) && (input.peek2(Token![:])) {
            //@ On:Event
            let mut parsed: GOnEvent = input.parse()?;
            parsed.at_setup(at_list);
            Ok(Self::OnEvent(parsed))
        }
        //  must on bottom ─────────────────────────────────────────────────────────────────
        else if input.peek(Ident::peek_any) {
            // @surface  expr, GElement
            let mut parsed: GTreeSurface = input.parse()?;
            parsed.at_setup(at_list);
            Ok(Self::GS(Box::new(parsed)))
        } else {
            panic!("can't know what is , input current:{}", input);
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
trait AtSetup {
    fn at_setup(&mut self, at_list: AtList);
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
}

impl AtSetup for GTreeLayerStruct {
    fn at_setup(&mut self, at_list: AtList) {
        for at in at_list.0 {
            match at {
                At::Id(id) => {
                    self.id = id;
                }
                At::Edge(edge) => {
                    self.edge = Some(edge);
                }
                At::Mod => {
                    panic!("layer can't be Mod");
                }
            }
        }
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
        })
    }
}

fn edge2token(edge: &Option<Edge>) -> TokenStream {
    match edge {
        Some(e) => {
            quote!(#e)
        }
        None => {
            quote!(vec![])
        }
    }
}

impl ToTokens for GTreeLayerStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            edge,
            layer,
            id,
            children,
        } = self;
        let edge_token = edge2token(edge);
        let children_iter = children.iter();
        let g_tree_builder_element_layer_token =
            quote_spanned! {layer.span()=>GTreeBuilderElement::Layer};

        let id_token = id.get("L");
        let children_token = quote_spanned! {children.span()=>vec![#(#children_iter),*]};
        // let brace_op_token = quote_spanned! {children.span()=>vec![#children_token]};

        quote!(#g_tree_builder_element_layer_token(#id_token,#edge_token,#children_token))
            .to_tokens(tokens);
        // quote!(GTreeBuilderElement::#layer(String::from(#id),vec![#(#children_iter),*])).to_tokens(tokens)
    }
}

// @ Gtree ────────────────────────────────────────────────────────────────────────────────
#[derive(Debug)]
pub struct Gtree {
    // emg_graph: Ident,
    root: GTreeLayerStruct,
}

impl Parse for Gtree {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // let emg_graph: Ident = input.parse()?;
        // let _ = input.parse::<Token![=>]>()?;
        let at_list = input.parse::<AtList>()?;

        let mut root = input.parse::<GTreeLayerStruct>()?;
        for at in at_list.0 {
            match at {
                At::Id(id) => {
                    root.id = id;
                }
                At::Edge(_) => {
                    panic!("root layer can't have any Edge");
                }
                At::Mod => {
                    panic!("root can't be Mod");
                }
            }
        }
        // Ok(Gtree { emg_graph, root })
        Ok(Self { root })
    }
}

impl ToTokens for Gtree {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { root } = self;

        let token = quote_spanned! {root.span()=> {

            #[allow(unused)]
            use emg_bind::{ EventCallback, EventMessage, GElement,
                GTreeBuilderElement,node_ref
            };
            use emg_bind::better_any::{
                impl_tid, tid, type_id, Tid, TidAble, TidExt,
            };
            #[allow(unused)]
            use emg_layout::{css, styles::*,add_values::*,EmgEdgeItem};
            #[allow(unused)]
            use emg_refresh::{EqRefreshFor,RefreshFor,Refresher,RefreshUse};

            #[allow(unused)]
            use emg_state::{use_state, StateMultiAnchor,CloneStateVar,CloneStateAnchor};

            #[allow(unused)]
            use std::rc::Rc;
            #[allow(unused)]
            use GElement::*;
            #[allow(unused)]
            use emg_core::{TypeCheck,Vector,IdStr};
            // #[allow(unused)]
            // pub use emg_bind::serde_closure;

            // #[allow(unused)]
            // use anchors::singlethread::*;
            // ENGINE.with(|_e| {
            //     log::info!("============= engine initd");
            // });


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
    fn test_2() {
        fn token_test(input: &str) {
            match syn::parse_str::<Gtree>(input) {
                Ok(ok) => println!("===>{}", ok.to_token_stream()),
                Err(error) => println!("...{:?}", error),
            }
        }

        println!();
        let input = r#" 
        @=root
                Layer [
                    @=x111x @E=[{@h |(button)...| in(#panel) gap(10)},h(px(11))]
                    Layer []
                ]
        "#;

        token_test(input);
        println!();
    }
    #[test]
    fn test_id() {
        fn token_test(input: &str) {
            match syn::parse_str::<Gtree>(input) {
                Ok(ok) => println!("===>{}", ok.to_token_stream()),
                Err(error) => println!("...{:?}", error),
            }
        }

        println!();
        let input = r#" 
        @=aa1 @E=[@h |(button)...| in(#panel) gap(10),h(px(11))]
                Layer [
                    Text::new(format!("aa1***********8"))=>[
                        RefreshUse dyn_v
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
