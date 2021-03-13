use std::collections::HashSet as Set;

use trace_var::trace_var;

use proc_macro2::{Span, TokenStream};
// use quote::{quote, ToTokens};
use proc_quote::{quote, quote_spanned, ToTokens};
// use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{
    bracketed, ext::IdentExt, punctuated::Punctuated, spanned::Spanned, token, FieldsNamed,
    ItemStruct, Lifetime, LifetimeDef,
};
use syn::{fold::Fold, parse_quote};

use syn::{Ident, Token};
// ────────────────────────────────────────────────────────────────────────────────
// use proc_macro::Diagnostic;
pub mod kw {
    // use std::fmt::Debug;

    syn::custom_keyword!(Layer);
    syn::custom_keyword!(Refresher);
    syn::custom_keyword!(On);
    syn::custom_keyword!(Event);

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
// @ G_On_Event ────────────────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GOnEvent {
    event_name: syn::LitStr,
    closure: syn::ExprClosure,
}
impl Parse for GOnEvent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        println!("0");
        input.parse::<kw::On>()?;
        println!("1");
        input.parse::<Token![:]>()?;
        println!("2");

        input.parse::<kw::Event>()?;
        println!("3");
        let event_name = input.parse()?;

        input.parse::<token::FatArrow>()?;

        Ok(GOnEvent {
            event_name,
            closure: input.parse()?,
        })
    }
}
impl ToTokens for GOnEvent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let GOnEvent {
            event_name,
            closure,
        } = self;

        let token = quote! (GTreeBuilderElement::EventCallBack((String::from(#event_name),Box::new(#closure))) );

        token.to_tokens(tokens)
        // quote_spanned!(expr.span()=>GTreeBuilderElement::El(#expr.into())).to_tokens(tokens)
        // quote!(GTreeBuilderElement::El(#expr.into())).to_tokens(tokens)
    }
}
// @ GRefresher ────────────────────────────────────────────────────────────────────────────────

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
        let kw_token = quote_spanned! (kws.span()=>GTreeBuilderElement::RefreshUse(Rc::new(#kws::new(#closure_token))) );

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

        // Tree GElementTree
        quote_spanned! (expr.span() => GTreeBuilderElement::GElementTree(#expr,#children_token))
            .to_tokens(tokens)
    }
}

// @ GTreeElement ────────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum GTreeMacroElement {
    GL(GTreeLayerStruct),
    GS(Box<GTreeSurface>),
    RT(GRefresher),
    GC(GTreeClosure),
    OnEvent(GOnEvent), // OtherExpr(syn::Expr),
}

impl Parse for GTreeMacroElement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // use syn::ext::IdentExt;

        if input.peek(kw::Layer) {
            //@layer
            Ok(GTreeMacroElement::GL(input.parse()?))
        } else if input.peek(kw::Refresher) {
            // @refresher
            Ok(GTreeMacroElement::RT(input.parse()?))
        } else if input.peek(token::Fn) && (input.peek2(Token![||]) || input.peek3(Token![||])) {
            // @closure
            Ok(GTreeMacroElement::GC(input.parse()?))
        } else if input.peek(kw::On) && (input.peek3(kw::Event)) {
            //@ On:Event
            Ok(GTreeMacroElement::OnEvent(input.parse()?))
        }
        //  must on bottom ─────────────────────────────────────────────────────────────────
        else if input.peek(Ident::peek_any) {
            // @surface  expr, GElement
            Ok(GTreeMacroElement::GS(input.parse()?))
        } else {
            panic!("can't know what is");
            // Ok(GTreeMacroElement::OtherExpr(input.parse()?))
        }
    }
}

impl ToTokens for GTreeMacroElement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use match_any::match_any;

        match_any!( self ,
            Self::GL(x)|Self::GS(x)|Self::RT(x)|Self::GC(x)|Self::OnEvent(x) => x.to_tokens(tokens)
        )
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
    // emg_graph: Ident,
    root: GTreeLayerStruct,
}

impl Parse for Gtree {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // let emg_graph: Ident = input.parse()?;
        // let _ = input.parse::<Token![=>]>()?;

        let root = input.parse::<GTreeLayerStruct>()?;

        // Ok(Gtree { emg_graph, root })
        Ok(Gtree { root })
    }
}

impl ToTokens for Gtree {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Gtree { root } = self;

        let token = quote_spanned! {root.span()=> {

            #[allow(unused)]
            use std::rc::Rc;
            #[allow(unused)]
            use emg_bind::CloneState;
            #[allow(unused)]
            use emg_bind::{
                 runtime::Element, runtime::Text, GElement, GTreeBuilderElement,
                 Refresher,
            };
            #[allow(unused)]
            use gtree::log;
            #[allow(unused)]
            use GElement::*;

            #[allow(unused)]
            use anchors::singlethread::*;
            emg_bind::ENGINE.with(|_e| {
                log::info!("============= engine initd");
            });


             #root



        }};
        token.to_tokens(tokens)
    }
}

/// @ gtree_macro ────────────────────────────────────────────────────────────────────────────────
pub fn gtree_macro(item: TokenStream) -> Result<TokenStream, syn::Error> {
    let output = syn::parse2::<Gtree>(item)?;
    Ok(quote! (#output))
}

#[derive(Debug, Clone)]
struct EmgArgs {
    vars: Set<Ident>,
    first_life_time: Option<Lifetime>,
}
impl EmgArgs {
    fn has_init_var(&self) -> bool {
        self.vars.contains(&Ident::new("init", Span::call_site()))
    }
}
impl Parse for EmgArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vars = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(EmgArgs {
            vars: vars.into_iter().collect(),
            first_life_time: None,
        })
    }
}
impl Fold for EmgArgs {
    // fn fold_field(&mut self, field: Field) -> Field {
    //     // let o: Field = parse_quote! {#i};
    //     println!("===Field: {:#?}", &field);
    //     fold::fold_field(self, field)
    // }
    fn fold_fields_named(&mut self, i: FieldsNamed) -> FieldsNamed {
        let FieldsNamed {
            brace_token: _,
            named,
        } = &i;
        let field = named.iter();
        // println!("---->{}", quote! {#named});
        let lifetime = self.first_life_time.as_ref().unwrap();

        parse_quote!({#(#field),* ,emg_graph:emg_bind::GraphType<#lifetime,Message>})
        // fold::fold_fields_named(self, i)
    }
}

/// @ emg_macro ────────────────────────────────────────────────────────────────────────────────
pub fn emg_macro(args: TokenStream, input: TokenStream) -> Result<TokenStream, syn::Error> {
    let args = syn::parse2::<EmgArgs>(args)?;
    println!("has_init_var? {:?}", args.has_init_var());

    let input = syn::parse2::<ItemStruct>(input)?;
    // ────────────────────────────────────────────────────────────────────────────────

    let o = emg_handle(args, input);
    // ────────────────────────────────────────────────────────────────────────────────

    Ok(quote!(

        #o

    ))
}

fn emg_handle(mut args: EmgArgs, input: ItemStruct) -> ItemStruct {
    let mut need_add_lifetime = false;
    if args.first_life_time.is_none() {
        let first_lifetime = input.generics.lifetimes().next();
        need_add_lifetime = first_lifetime.is_none();

        args.first_life_time = first_lifetime
            .map(|l_def| &l_def.lifetime)
            .cloned()
            .or_else(|| Some(Lifetime::new("'a", Span::call_site())));
    };
    println!("=====first_life_time:{:?}", &args.first_life_time);
    let mut o = args.fold_item_struct(input);
    if need_add_lifetime {
        o.generics
            .params
            .push(syn::GenericParam::Lifetime(LifetimeDef::new(
                args.first_life_time.unwrap(),
            )))
    }
    o
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
                    Refresher ||{33},
                    On:Event "click" => |_root,_vdom,_event|{let x=9987665;log::info!("in gtree {}",x);}
                ],
                Layer "e" [
                    Button_(Button::new(Text::new(format!("button in quote..{}", "e")))) => [
                        On:Event "click" => |_root,_vdom,_event|{let x=888888;log::info!("in gtree {}",x);}
                    ]
                ],
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
    #[test]
    fn emg_life() {
        let input: ItemStruct = syn::parse_quote!(
            struct AA<'f: 'b + 'c, 'b, 'c> {
                bb: String,
                cc: String,
            }
        );
        println!("====input:{:#?}", &input);

        let args: EmgArgs = EmgArgs {
            vars: Set::new(),
            first_life_time: None,
        };
        println!("has_init_var? {:?}", args.has_init_var());
        // ─────────────────────────────────────────────────────────────────
        let o = emg_handle(args, input);

        // ─────────────────────────────────────────────────────────────────

        println!("=======================");
        // println!("o: {:#?}", &o);
        println!("=======================");
        println!("{}", quote! {#o});
    }
    #[test]
    fn emg_def_life() {
        let input: ItemStruct = syn::parse_quote!(
            struct AA<'f: 'b + 'c, 'b, 'c> {
                bb: String,
                cc: String,
            }
        );
        println!("====input:{:#?}", &input);

        let args: EmgArgs = EmgArgs {
            vars: Set::new(),
            first_life_time: None,
        };
        println!("has_init_var? {:?}", args.has_init_var());
        // ─────────────────────────────────────────────────────────────────
        let o = emg_handle(args, input);

        // ─────────────────────────────────────────────────────────────────

        println!("=======================");
        // println!("o: {:#?}", &o);
        println!("=======================");
        println!("{}", quote! {#o});
    }
    #[test]
    fn emg_no_life() {
        let input: ItemStruct = syn::parse_quote!(
            struct AA {
                bb: String,
                cc: String,
            }
        );
        println!("====input:{:#?}", &input);

        let args: EmgArgs = EmgArgs {
            vars: Set::new(),
            first_life_time: None,
        };
        println!("has_init_var? {:?}", args.has_init_var());
        // ─────────────────────────────────────────────────────────────────
        let o = emg_handle(args, input);
        // ─────────────────────────────────────────────────────────────────

        println!("=======================");
        // println!("o: {:#?}", &o);
        println!("=======================");
        println!("{}", quote! {#o});
    }
}
