use either::Either;
use Either::{Left, Right};
/*
 * @Author: Rais
 * @Date: 2022-06-24 18:11:24
 * @LastEditTime: 2023-02-19 01:06:08
 * @LastEditors: Rais
 * @Description:
 */
use im_rc::{vector, Vector};
use parse_display::Display;
use proc_macro2::{Span, TokenStream};
use std::collections::HashMap;

use quote::{quote, quote_spanned, ToTokens};
use syn::{
    braced, bracketed, parenthesized,
    parse::{discouraged::Speculative, Parse, ParseStream},
    punctuated::Punctuated,
    token, BinOp, Ident, LitFloat, LitInt, LitStr, Token,
};
use syn::{ext::IdentExt, spanned::Spanned};
use tracing::{debug, debug_span, error, instrument};

use crate::quote_option::QuoteOption;

fn size_var_names(d: &Dimension) -> Ident {
    match d {
        Dimension::H => Ident::new("width", Span::call_site()),
        Dimension::V => Ident::new("height", Span::call_site()),
    }
}
fn left_var_names(d: &Dimension) -> Ident {
    match d {
        Dimension::H => Ident::new("right", Span::call_site()),
        Dimension::V => Ident::new("bottom", Span::call_site()),
    }
}
fn right_var_names(d: &Dimension) -> Ident {
    match d {
        Dimension::H => Ident::new("left", Span::call_site()),
        Dimension::V => Ident::new("top", Span::call_site()),
    }
}
fn super_left_var_names(d: &Dimension) -> Ident {
    match d {
        Dimension::H => Ident::new("left", Span::call_site()),
        Dimension::V => Ident::new("top", Span::call_site()),
    }
}
fn super_right_var_names(d: &Dimension) -> Ident {
    match d {
        Dimension::H => Ident::new("right", Span::call_site()),
        Dimension::V => Ident::new("bottom", Span::call_site()),
    }
}
fn standard_gap_names(d: &Dimension) -> Ident {
    match d {
        Dimension::H => Ident::new("hgap", Span::call_site()),
        Dimension::V => Ident::new("vgap", Span::call_site()),
    }
}

mod kw_strength {
    // use std::fmt::Debug;
    #![warn(clippy::expl_impl_clone_on_copy)]

    syn::custom_keyword!(weak);
    syn::custom_keyword!(medium); //default
    syn::custom_keyword!(strong);
    syn::custom_keyword!(required);
    // syn::custom_keyword!(Dyn);

    // impl Debug for layer {
    //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //         f.write_str(concat!("Keyword [", stringify!(layer), "]"))
    //     }
    // }
}
mod kw_opt {
    // use std::fmt::Debug;
    // #![warn(clippy::expl_impl_clone_on_copy)]

    syn::custom_keyword!(chain);
    syn::custom_keyword!(gap);
    syn::custom_keyword!(outer);
    // syn::custom_keyword!(Dyn);

    // impl Debug for layer {
    //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //         f.write_str(concat!("Keyword [", stringify!(layer), "]"))
    //     }
    // }
}

#[derive(Debug)]
enum Dimension {
    H,
    V,
}
impl Dimension {
    const H_KW: [&'static str; 2] = ["horizontal", "h"];
    const V_KW: [&'static str; 2] = ["vertical", "v"];
}
impl Parse for Dimension {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug!("in Dimension");

        input.parse::<Token![@]>()?;
        let got_kw = input.parse::<Ident>()?;
        debug!("got_kw {:?} | {:?}", &got_kw, got_kw.to_string());

        let mut dim = Self::H;
        for kw in Self::H_KW {
            if kw == got_kw.to_string().as_str() {
                dim = Self::H;
            }
        }
        for kw in Self::V_KW {
            if kw == got_kw.to_string().as_str() {
                dim = Self::V;
            }
        }
        Ok(dim)
    }
}

#[derive(Debug, Clone, Display)]
pub enum Number {
    #[display("{0}")]
    Int(LitInt),
    #[display("{0}")]
    Float(LitFloat),
}
impl Parse for Number {
    #[instrument(name = "Number")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitInt) {
            let n = input.parse::<LitInt>()?;
            debug!("got int: {:?}", &n);
            return Ok(Self::Int(n));
        }

        debug!("not int {:?}", input);

        let n = input.parse::<LitFloat>()?;
        debug!("got float: {:?}", &n);
        Ok(Self::Float(n))
    }
}
#[derive(Debug, Clone, Display)]
pub enum NameCharsOrNumber {
    #[display("#{0}")]
    Id(Ident), // #xxx
    #[display(".{0}")]
    Class(Ident), // .xxx
    #[display("{0}")]
    Element(Ident), // xxxx
    #[display("\"{0}\"")]
    Virtual(String), //"xxx"
    #[display("{0}")]
    Number(Number), // 12 | 12.1
    #[display("{0}:next")]
    Next(Box<Self>), // TODO do parse
    #[display("{0}:last")]
    Last(Box<Self>),
    #[display("{0}:first")]
    First(Box<Self>),
    //TODO add before after 表示 层级 前后
}

impl NameCharsOrNumber {
    // fn into_next(self) -> Self {
    //     assert!(!self.is_id());
    //     Self::Next(Box::new(self))
    // }
    fn make_next(&self) -> Self {
        assert!(!self.is_id(), "只适用于selector,非 id");
        Self::Next(Box::new(self.clone()))
    }
    fn make_last(&self) -> Self {
        assert!(!self.is_id());
        Self::Last(Box::new(self.clone()))
    }
    fn make_first(&self) -> Self {
        assert!(!self.is_id());
        Self::First(Box::new(self.clone()))
    }

    /// Returns `true` if the name chars is [`Id`].
    ///
    /// [`Id`]: NameCharsOrNumber::Id
    #[must_use]
    const fn is_id(&self) -> bool {
        matches!(self, Self::Id(..))
    }

    /// Returns `true` if the name chars is [`Number`].
    ///
    /// [`Number`]: NameCharsOrNumber::Number
    #[must_use]
    const fn is_number(&self) -> bool {
        matches!(self, Self::Number(..))
    }
}

impl Parse for NameCharsOrNumber {
    #[instrument(name = "NameCharsOrNumber")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // TODO [a-zA-Z0-9#.\-_$:""&]

        if input.peek(Token![#]) {
            debug!("peek #");
            input.parse::<Token![#]>()?;
            debug!("parse #");
            debug!("will parse ident {:?}", &input);

            let name = input.call(Ident::parse_any)?;
            debug!("got id: #{:?}", &name);
            return Ok(Self::Id(name));
        }
        debug!("not id : {:?}", &input);

        if input.peek(Token![.]) {
            debug!("peek .");

            input.parse::<Token![.]>()?;
            let name = input.call(Ident::parse_any)?;
            debug!("got class: .{:?}", &name);
            return Ok(Self::Class(name));
        }
        debug!("not class : {:?}", &input);

        if input.peek(LitStr) {
            debug!("peek \" \" ");

            let r#virtual: LitStr = input.parse()?;
            debug!("got virtual: {:?}", &r#virtual);
            return Ok(Self::Virtual(r#virtual.value()));
        }
        debug!("not Virtual : {:?}", &input);

        if input.peek(LitFloat) || input.peek(LitInt) {
            debug!("peek number");

            let n: Number = input.parse()?;
            debug!("got Number: {:?}", &n);

            return Ok(Self::Number(n));
        }
        debug!("not Number : {:?}", &input);
        // ────────────────────────────────────────────────────────────────────────────────

        let name = input.call(Ident::parse_any)?;

        debug!("got Element: {:?}", &name);
        Ok(Self::Element(name))
    }
}

impl ToTokens for NameCharsOrNumber {
    #[allow(clippy::match_same_arms)]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Id(x) => {
                let str = x.to_string();
                quote_spanned!(x.span()=> emg_bind::layout::ccsa::NameCharsOrNumber::Id(emg_bind::common::IdStr::new(#str)))
                    .to_tokens(tokens);
            }
            Self::Class(x) => {
                let str = x.to_string();
                quote_spanned!(x.span()=> emg_bind::layout::ccsa::NameCharsOrNumber::Class(emg_bind::common::IdStr::new(#str)))
                    .to_tokens(tokens);
            }
            Self::Element(x) => {
                let str = x.to_string();
                quote_spanned!(x.span()=> emg_bind::layout::ccsa::NameCharsOrNumber::Element(emg_bind::common::IdStr::new(#str)))
                    .to_tokens(tokens);
            }
            Self::Virtual(x) => {
                quote_spanned!(x.span()=> emg_bind::layout::ccsa::NameCharsOrNumber::Virtual(emg_bind::common::IdStr::new(#x)))
                    .to_tokens(tokens);
            }
            Self::Number(n) => match n {
                Number::Int(int) => {
                    quote_spanned!(int.span()=> emg_bind::layout::ccsa::NameCharsOrNumber::Number( emg_bind::common::NotNan::new(#int.into()).unwrap() ))
                        .to_tokens(tokens);
                }
                Number::Float(float) => {
                    quote_spanned!(float.span()=> emg_bind::layout::ccsa::NameCharsOrNumber::Number( emg_bind::common::NotNan::new(#float).unwrap() ))
                        .to_tokens(tokens);
                }
            },
            Self::Next(_) => todo!("Next"),
            Self::Last(_) => todo!("Last"),
            Self::First(_) => todo!("First"),
        }
    }
}
/// `+ - *`
#[derive(Debug, Clone, Display)]
enum PredOp {
    #[display("{0}")]
    Add(#[display("+")] Token![+]),
    #[display("{0}")]
    Sub(#[display("-")] Token![-]),
    #[display("{0}")]
    Mul(#[display("*")] Token![*]),
}
impl ToTokens for PredOp {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Add(x) => {
                quote_spanned!(x.span()=> emg_bind::layout::ccsa::PredOp::Add).to_tokens(tokens);
            }
            Self::Sub(x) => {
                quote_spanned!(x.span()=> emg_bind::layout::ccsa::PredOp::Sub).to_tokens(tokens);
            }
            Self::Mul(x) => {
                quote_spanned!(x.span()=> emg_bind::layout::ccsa::PredOp::Mul).to_tokens(tokens);
            }
        }
    }
}

impl PredOp {
    fn new_add() -> Self {
        Self::Add(token::Add::default())
    }
    // fn new_sub() -> Self {
    //     Self::Sub(token::Sub::default())
    // }
    // fn new_mul() -> Self {
    //     Self::Mul(token::Star::default())
    // }
}
impl Parse for PredOp {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug!("in PredOp");

        let pred_op: BinOp = input.parse()?;
        match pred_op {
            BinOp::Add(x) => Ok(Self::Add(x)),
            BinOp::Sub(x) => Ok(Self::Sub(x)),
            BinOp::Mul(x) => Ok(Self::Mul(x)),
            // _ => panic!("[PredOp] op not support :{:?}", pred_op),
            _ => Err(syn::Error::new(
                pred_op.span(),
                format!("[PredOp] op not support :{pred_op:?}"),
            )),
        }
    }
}

/// `[var]`
#[derive(Debug, Clone, Display)]
#[display("[{0}]")]
struct PredVariable(Ident);
impl Parse for PredVariable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug!("in PredVariable");

        let content;
        let _bracket_token = bracketed!(content in input);
        let var: Ident = content.call(Ident::parse_any)?;
        Ok(Self(var))
    }
}

impl ToTokens for PredVariable {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let var = self.0.to_string();

        quote_spanned!(self.0.span()=> emg_bind::layout::ccsa::PredVariable(emg_bind::common::IdStr::new(#var)))
            .to_tokens(tokens);
    }
}

fn disp_opt<T: std::fmt::Display>(o: Option<T>) -> String {
    o.map_or(String::new(), |x| format!("{x}"))
}
/// `&name[var]`
#[derive(Debug, Clone)]
pub struct ScopeViewVariable {
    scope: Option<Scope>,
    view: Option<NameCharsOrNumber>,
    variable: Option<PredVariable>,
}

impl std::fmt::Display for ScopeViewVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let scope = self
            .scope
            .as_ref()
            .map_or(String::new(), |x| format!("{x}"));
        let view = self.view.as_ref().map_or(String::new(), |x| format!("{x}"));
        let variable = self
            .variable
            .as_ref()
            .map_or(String::new(), |x| format!("{x}"));

        write!(f, "{scope}{view}{variable}")
    }
}

impl ScopeViewVariable {
    #[must_use]
    fn into_next(self, var: Option<Ident>) -> Self {
        Self {
            scope: self.scope,
            view: self.view.map(|v| v.make_next()),
            variable: var.map(PredVariable).or(self.variable),
        }
    }

    // fn make_next(&self, var: Option<Ident>) -> Self {
    //     Self {
    //         scope: self.scope.clone(),
    //         view: self.view.map(|v| v.make_next()),
    //         variable: var
    //             .map(|v| PredVariable(v))
    //             .or_else(|| self.variable.clone()),
    //     }
    // }
    #[must_use]
    fn make_last(&self, var: Option<Ident>) -> Self {
        Self {
            scope: self.scope,
            view: self.view.clone().map(|v| v.make_last()),
            variable: var.map(PredVariable).or_else(|| self.variable.clone()),
        }
    }
    #[must_use]
    fn make_first(&self, var: Option<Ident>) -> Self {
        Self {
            scope: self.scope,
            view: self.view.clone().map(|v| v.make_first()),
            variable: var.map(PredVariable).or_else(|| self.variable.clone()),
        }
    }
    #[must_use]
    const fn new_scope(scope: Scope) -> Self {
        Self {
            scope: Some(scope),
            view: None,
            variable: None,
        }
    }
    #[must_use]
    const fn new_var(var: Ident) -> Self {
        Self {
            scope: None,
            view: None,
            variable: Some(PredVariable(var)),
        }
    }
    const fn variable_is_none(&self) -> bool {
        self.variable.is_none()
    }
    #[must_use]
    fn with_variable(mut self, var: Ident) -> Self {
        if !self.view.as_ref().is_some_and(NameCharsOrNumber::is_number) {
            self.variable = Some(PredVariable(var));
        }
        self
    }
    #[must_use]
    fn or_with_variable(mut self, var: Ident) -> Self {
        if self.variable_is_none() && !self.view.as_ref().is_some_and(NameCharsOrNumber::is_number)
        {
            self.variable = Some(PredVariable(var));
        }
        self
    }

    fn set_variable(&mut self, var: Ident) {
        if !self.view.as_ref().is_some_and(NameCharsOrNumber::is_number) {
            self.variable = Some(PredVariable(var));
        }
    }
}
impl Parse for ScopeViewVariable {
    #[instrument(name = "ScopeViewVariable")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let scope = input.parse();
        let fork = input.fork();

        if let Ok(scope2) = fork.parse::<Scope>() {
            return Err(syn::Error::new(scope2.span(), "scope duplicated "));
        }

        let view = input.parse();
        let variable = input.parse();

        if scope.is_err() && view.is_err() && variable.is_err() {
            let e = scope.err().unwrap();
            let e2 = view.err().unwrap();
            let e3 = variable.err().unwrap();
            error!(
                "all none in ScopeViewVariable \n {:?} \n {:?} \n {:?}\n",
                &e, &e2, &e3
            );
            return Err(syn::Error::new(e.span(), "all none in ScopeViewVariable"));
        }

        let res = Self {
            scope: scope.ok(),
            view: view.ok(),
            variable: variable.ok(),
        };
        debug!("got ScopeViewVariable : {:?}", &res);

        Ok(res)
    }
}

impl ToTokens for ScopeViewVariable {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let scope = QuoteOption(self.scope.as_ref());
        let view = QuoteOption(self.view.as_ref());
        let variable = QuoteOption(self.variable.as_ref());
        let span = self
            .scope
            .span()
            .join(self.view.span())
            .unwrap()
            .join(self.variable.span())
            .unwrap();
        quote_spanned!(span=> emg_bind::layout::ccsa::ScopeViewVariable::new(#scope, #view, #variable))
            .to_tokens(tokens);
    }
}

// /// `name[var]`
// #[derive(Debug, Clone)]
// struct PredViewVariable {
//     view: NameCharsOrNumber,
//     pred_variable: PredVariable,
// }
// impl Parse for PredViewVariable {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         debug!("in PredViewVariable");

//         Ok(Self {
//             view: input.parse()?,
//             pred_variable: input.parse()?,
//         })
//     }
// }

// /// `name`
// #[derive(Debug, Clone)]
// struct PredView {
//     view: NameCharsOrNumber,
// }
// impl Parse for PredView {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         debug!("in PredView");

//         Ok(Self {
//             view: input.parse()?,
//         })
//     }
// }

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
enum PredExpressionItem {
    PredOp(PredOp),
    ScopeViewVariable(ScopeViewVariable),
}

// impl PredExpressionItem {
//     const fn as_scope_view_variable(&self) -> Option<&ScopeViewVariable> {
//         if let Self::ScopeViewVariable(v) = self {
//             Some(v)
//         } else {
//             None
//         }
//     }
// }
impl Parse for PredExpressionItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug!("in PredExpressionItem");
        if let Ok(x) = input.parse::<PredOp>() {
            return Ok(Self::PredOp(x));
        }
        debug!("not PredOp : {:?}", &input);

        // if let Ok(x) = input.parse::<PredLiteral>() {
        //     return Ok(Self::PredLiteral(x));
        // }
        // debug!("not PredLiteral : {:?}", &input);

        Ok(Self::ScopeViewVariable(input.parse::<ScopeViewVariable>()?))
    }
}
/// Vec [ (`PredOp` , `ScopeViewVariable` ) ]
#[derive(Debug, Clone)]
struct PredExpression(ScopeViewVariable, Vec<(PredOp, ScopeViewVariable)>);
impl Parse for PredExpression {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug!("in PredExpression");
        let first: ScopeViewVariable = input.parse()?;
        // ─────────────────────────────────────────────────────────────────

        let mut exps = vec![];
        let mut op = None;

        // !input.peek(Token![>])
        while !input.peek(Token![,]) && !input.peek(Token![!]) && !input.peek(Token![>]) /*for point <xxx> */&& !input.is_empty()
        {
            if let Ok(x) = input.parse::<PredExpressionItem>() {
                match x {
                    PredExpressionItem::PredOp(x) if op.is_none() => {
                        op = Some(x);
                    }
                    PredExpressionItem::ScopeViewVariable(x) if op.is_some() => {
                        exps.push((op.take().unwrap(), x));
                    }
                    // _ => panic!("[PredExpression] 运算顺序错误 ,必须 一个 op + 一个 var 一对 "),
                    _ => {
                        return Err(syn::Error::new(
                            input.span(),
                            "[PredExpression] 运算顺序错误 ,必须 一个 op + 一个 var 一对 ",
                        ))
                    }
                }
            } else if !input.is_empty() {
                panic!("[PredExpression] input not empty {input:?}")
            } else {
                break;
            }
        }

        let pred_expression = Self(first, exps);
        debug!("got pred_expression {:?}", pred_expression);
        Ok(pred_expression)
    }
}
/// !weak10   !require
#[derive(Debug, Clone)]
enum StrengthAndWeight {
    //TODO use Number struct
    Weak(Option<Either<LitInt, LitFloat>>),
    Medium(Option<Either<LitInt, LitFloat>>),
    Strong(Option<Either<LitInt, LitFloat>>),
    Require,
}
impl ToTokens for StrengthAndWeight {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Weak(x) => {
                let xx = x.as_ref().map_or_else(
                    || quote! { ::std::option::Option::None },
                    |lint| {
                        match lint{
                            Left(xxx) => {
                                quote_spanned! (xxx.span()=> ::std::option::Option::Some( emg_bind::common::NotNan::new(#xxx.into()).unwrap()) )
                            },
                            Right(xxx) => {
                                quote_spanned! (xxx.span()=> ::std::option::Option::Some( emg_bind::common::NotNan::new(#xxx).unwrap()) )
                            },
                        }
                    },
                );
                quote_spanned!(xx.span()=>emg_bind::layout::ccsa::StrengthAndWeight::Weak(#xx))
                    .to_tokens(tokens);
            }
            Self::Medium(x) => {
                let xx = x.as_ref().map_or_else(
                    || quote! { ::std::option::Option::None },
                    |lint| {
                        match lint{
                            Left(xxx) => {
                                quote_spanned! (xxx.span()=> ::std::option::Option::Some( emg_bind::common::NotNan::new(#xxx.into()).unwrap()) )
                            },
                            Right(xxx) => {
                                quote_spanned! (xxx.span()=> ::std::option::Option::Some( emg_bind::common::NotNan::new(#xxx).unwrap()) )
                            },
                        }
                    },
                );
                quote_spanned!(xx.span()=>emg_bind::layout::ccsa::StrengthAndWeight::Medium(#xx))
                    .to_tokens(tokens);
            }
            Self::Strong(x) => {
                let xx = x.as_ref().map_or_else(
                    || quote! { ::std::option::Option::None },
                    |lint| {
                        match lint{
                            Left(xxx) => {
                                quote_spanned! (xxx.span()=> ::std::option::Option::Some( emg_bind::common::NotNan::new(#xxx.into()).unwrap()) )
                            },
                            Right(xxx) => {
                                quote_spanned! (xxx.span()=> ::std::option::Option::Some( emg_bind::common::NotNan::new(#xxx).unwrap()) )
                            },
                        }
                    },
                );
                quote_spanned!(xx.span()=>emg_bind::layout::ccsa::StrengthAndWeight::Strong(#xx))
                    .to_tokens(tokens);
            }
            Self::Require => {
                quote!(emg_bind::layout::ccsa::StrengthAndWeight::Require).to_tokens(tokens);
            }
        }
    }
}

impl std::fmt::Display for StrengthAndWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Weak(x) => {
                if let Some(i) = x {
                    write!(f, " !weak({i})")
                } else {
                    write!(f, " !weak")
                }
            }
            Self::Medium(x) => {
                if let Some(i) = x {
                    write!(f, " !medium({i})")
                } else {
                    write!(f, " !medium")
                }
            }
            Self::Strong(x) => {
                if let Some(i) = x {
                    write!(f, " !strong({i})")
                } else {
                    write!(f, " !strong")
                }
            }
            Self::Require => {
                write!(f, " !require")
            }
        }
    }
}
impl Parse for StrengthAndWeight {
    #[instrument(name = "StrengthAndWeight")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug!("in strength_and_weight");
        input.parse::<Token![!]>()?;
        if input.parse::<kw_strength::weak>().is_ok() {
            debug!("got weak");
            if input.peek(token::Paren) {
                let content;
                let _paren_token = parenthesized!(content in input);
                debug!("got weak number");
                if let Ok(x) = content.parse::<LitInt>() {
                    return Ok(Self::Weak(Some(Left(x))));
                }
                return Ok(Self::Weak(Some(Right(content.parse::<LitFloat>()?))));
            }
            return Ok(Self::Weak(None));
        }
        debug!("not weak kw {:?}", &input);
        if input.parse::<kw_strength::medium>().is_ok() {
            if input.peek(token::Paren) {
                let content;
                let _paren_token = parenthesized!(content in input);
                debug!("got medium number");
                if let Ok(x) = content.parse::<LitInt>() {
                    return Ok(Self::Medium(Some(Left(x))));
                }
                return Ok(Self::Medium(Some(Right(content.parse::<LitFloat>()?))));
            }
            return Ok(Self::Medium(None));
        }
        if input.parse::<kw_strength::strong>().is_ok() {
            if input.peek(token::Paren) {
                let content;
                let _paren_token = parenthesized!(content in input);
                debug!("got strong number");
                if let Ok(x) = content.parse::<LitInt>() {
                    return Ok(Self::Strong(Some(Left(x))));
                }
                return Ok(Self::Strong(Some(Right(content.parse::<LitFloat>()?))));
            }
            return Ok(Self::Strong(None));
        }
        debug!("not strong kw");
        // if input.parse::<kw_strength::require>().is_ok() {
        //     if input.peek(LitInt) {
        //         return Ok(Self::Require(Some(input.parse()?)));
        //     }
        //     return Ok(Self::Require(None));
        // }

        let req = input.parse::<kw_strength::required>()?;
        debug!("find required keyword");
        if input.peek(token::Paren) {
            return Err(syn::Error::new(
                req.span(),
                "can't use number behead required keyWord",
            ));
        }
        Ok(Self::Require)
    }
}

/// ` == < > >= <= `
#[derive(Debug, Copy, Clone, Display)]
enum PredEq {
    #[display(" {0} ")]
    Eq(#[display("==")] Token![==]),
    #[display(" {0} ")]
    Lt(#[display("<")] Token![<]),
    #[display(" {0} ")]
    Le(#[display("<=")] Token![<=]),
    #[display(" {0} ")]
    Ge(#[display(">=")] Token![>=]),
    #[display(" {0} ")]
    Gt(#[display(">")] Token![>]),
}
impl ToTokens for PredEq {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Eq(x) => {
                quote_spanned!(x.span()=> emg_bind::layout::ccsa::PredEq::Eq).to_tokens(tokens);
            }
            Self::Lt(x) => {
                quote_spanned!(x.span()=> emg_bind::layout::ccsa::PredEq::Lt).to_tokens(tokens);
            }
            Self::Le(x) => {
                quote_spanned!(x.span()=> emg_bind::layout::ccsa::PredEq::Le).to_tokens(tokens);
            }
            Self::Ge(x) => {
                quote_spanned!(x.span()=> emg_bind::layout::ccsa::PredEq::Ge).to_tokens(tokens);
            }
            Self::Gt(x) => {
                quote_spanned!(x.span()=> emg_bind::layout::ccsa::PredEq::Gt).to_tokens(tokens);
            }
        }
    }
}

impl PredEq {
    fn chain_tail_eq_map(self) -> Self {
        match self {
            Self::Eq(_) => self,
            Self::Lt(_) => Self::Gt(token::Gt::default()),
            Self::Le(_) => Self::Ge(token::Ge::default()),
            Self::Ge(_) => Self::Le(token::Le::default()),
            Self::Gt(_) => Self::Lt(token::Lt::default()),
        }
    }
}

impl Default for PredEq {
    fn default() -> Self {
        Self::Eq(token::EqEq(Span::call_site()))
    }
}
impl Parse for PredEq {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug!("in PredEq");
        let pred_eq: BinOp = input.parse()?;
        match pred_eq {
            BinOp::Eq(x) => Ok(Self::Eq(x)),
            BinOp::Lt(x) => Ok(Self::Lt(x)),
            BinOp::Le(x) => Ok(Self::Le(x)),
            BinOp::Ge(x) => Ok(Self::Ge(x)),
            BinOp::Gt(x) => Ok(Self::Gt(x)),
            _ => panic!("[PredEq] op not support :{pred_eq:?}"),
        }
        // Ok(Self(pred_eq))
    }
}

#[derive(Debug, Clone)]
struct PredicateItem {
    pred_eq: PredEq,
    pred_expression: PredExpression,
    strength_and_weight: Option<StrengthAndWeight>,
}
impl Parse for PredicateItem {
    #[instrument(name = "PredicateItem")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug!("in PredicateItem");
        let pred_eq = input.parse()?;
        debug!("got pred_eq : {:?}", &pred_eq);
        let pred_expression = input.parse()?;
        debug!("got pred_expression : {:?}", &pred_expression);
        let strength_and_weight = input.parse().ok();
        debug!("got strength_and_weight : {:?}", &strength_and_weight);

        Ok(Self {
            pred_eq,
            pred_expression,
            strength_and_weight,
        })
    }
}
/// ( [`PredEq`] [`PredExpression`] [`StrengthAndWeight`]? )
#[derive(Debug, Clone)]
struct Predicate(Vec<PredicateItem>);
impl Parse for Predicate {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug!("in Predicate");
        let content;
        // ()
        let _paren_token = parenthesized!(content in input);
        debug!("got () : {:?}", &content);
        let content: Punctuated<PredicateItem, Token![,]> =
            content.parse_terminated(PredicateItem::parse)?;
        if !content.is_empty() {
            debug!("got PredicateItems vec:{:?}", &content);
        }

        Ok(Self(content.into_iter().collect()))
    }
}

// #[derive(Debug)]
// struct ComplexViewSelector {}
// impl Parse for ComplexViewSelector {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         let content;
//         let paren_token = parenthesized!(content in input);
//         // "(" _ view:[^()]* _ pred:Predicate? _ ")"
//         //TODO any not () ?
//         todo!()
//     }
// }
/// (`&name[var]`(`Predicate`)?)
#[derive(Debug, Clone)]
struct ViewSelector {
    view: ScopeViewVariable,
    pred: Option<Predicate>,
}

impl Parse for ViewSelector {
    #[instrument(name = "ViewSelector")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _paren_token = parenthesized!(content in input);
        debug!(" in ViewSelector find () \n content:{:?}", &content);

        let view = content.parse()?;
        debug!("got `view:ScopeViewVariable`:{:?}", &view);

        let pred = content.parse().ok();
        if pred.is_some() {
            debug!("got `pred:Option<Predicate> `:{:?}", &pred);
        }
        Ok(Self { view, pred })
    }
}

/// ` & ^ $` [`Scope::Local`] , [`Scope::Parent(u8)`] , [`Scope::Global`]
#[derive(Debug, Copy, Clone, Display)]
enum Scope {
    #[display("&")]
    Local(Span),
    //
    #[display("^({0})")]
    Parent(u8, Span),
    //
    #[display("$")]
    Global(Span),
}
impl Parse for Scope {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug!("in Scope");
        let span = input.span();

        if input.parse::<Token![&]>().is_ok() {
            return Ok(Self::Local(span));
        }
        if input.parse::<Token![$]>().is_ok() {
            return Ok(Self::Global(span));
        }
        if input.peek(Token![^]) {
            let mut n = 0u8;
            while input.peek(Token![^]) {
                span.join(input.span()).unwrap();
                input.parse::<Token![^]>()?;
                n += 1;
            }
            return Ok(Self::Parent(n, span));
        }
        Err(syn::Error::new(input.span(), "expected `&` or `^` or `$`"))

        // input.parse::<Token![$]>()?;
        // Ok(Self::Global)
    }
}

impl ToTokens for Scope {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Local(span) => {
                quote_spanned!(*span=> emg_bind::layout::ccsa::Scope::Local).to_tokens(tokens);
            }

            Self::Parent(n, span) => {
                quote_spanned!(*span=> emg_bind::layout::ccsa::Scope::Parent(#n)).to_tokens(tokens);
            }
            Self::Global(span) => {
                quote_spanned!(*span=> emg_bind::layout::ccsa::Scope::Global).to_tokens(tokens);
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Gap {
    Var(ScopeViewVariable),
    Standard,
    None,
}

/// `[- ~ ]`
#[derive(Debug, Clone)]
enum Connection {
    Eq(Gap),
    Le(Gap),
}

impl Connection {
    const fn gap(&self) -> &Gap {
        match self {
            Self::Eq(x) | Self::Le(x) => x,
        }
    }
    fn op(&self) -> PredEq {
        match self {
            Self::Eq(_) => PredEq::Eq(token::EqEq::default()),
            Self::Le(_) => PredEq::Le(token::Le::default()),
        }
    }
}
impl Parse for Connection {
    #[instrument(name = "Connection")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![-]) {
            debug!("peek - ");
            input.parse::<Token![-]>()?;

            if input.peek(token::Dot3) {
                return Ok(Self::Eq(Gap::Standard));
            }

            if let Ok(explicit_gap) = input.parse::<ScopeViewVariable>() {
                debug!("got ScopeViewVariable: {:?}", &explicit_gap);

                input.parse::<Token![-]>()?;
                return Ok(Self::Eq(Gap::Var(explicit_gap)));
            }
            debug!("got just - ");
            return Ok(Self::Eq(Gap::Standard));
        }
        debug!("not - ");

        if input.peek(Token![~]) {
            debug!("peek ~ ");

            input.parse::<Token![~]>()?;

            //TODO check input.parse::<ScopeViewVariable>()  不会通吃
            if let Ok(explicit_gap) = input.parse::<ScopeViewVariable>() {
                debug!("got ScopeViewVariable: {:?}", &explicit_gap);

                input.parse::<Token![~]>()?;
                debug!(" ~view~ ");
                return Ok(Self::Le(Gap::Var(explicit_gap)));
            }
            debug!("not ~view_var ");

            if input.parse::<Token![-]>().is_ok() {
                input.parse::<Token![~]>()?;
                debug!(" ~-~ ");

                return Ok(Self::Le(Gap::Standard));
            }
            debug!("not ~-~ ");

            return Ok(Self::Le(Gap::None));
        }
        debug!("not ~ ");

        Ok(Self::Eq(Gap::None))
    }
}

/// ( `NameCharsOrNumber`[`Predicate`]? ) `[- ~]?` ...
#[derive(Debug, Clone)]
struct Splat {
    view_selector: ViewSelector,
    opt_connection: Option<Connection>,
}

impl Parse for Splat {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _span =
            debug_span!("in Splat, parse-> ( `NameCharsOrNumber`[`Predicate`]? ) `[- ~]` ... ")
                .entered();

        let view_selector = input.parse::<ViewSelector>()?;
        debug!("got view_selector:{:?}", &view_selector);
        let opt_connection = input.parse::<Connection>().ok();
        debug!("got opt_connection:{:?}", &opt_connection);

        let s = Self {
            view_selector,
            opt_connection,
        };
        debug!("splat parsing(finding)-> ...");

        input.parse::<Token![...]>()?;
        debug!("splat parse success");
        Ok(s)
    }
}

/// `< Predicate >`
///
/// TODO : support < (.box ! .foo:bar:next .black)[left] >
#[derive(Debug, Clone)]
struct Point(PredExpression);
impl Parse for Point {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _span = debug_span!("in Point").entered();
        // "<" _? position:[^>]+ _? ">"
        // < [line] >
        input.parse::<Token![<]>()?;
        let p = input.parse::<PredExpression>()?;
        input.parse::<Token![>]>()?;
        Ok(Self(p))
    }
}
#[derive(Debug, Clone)]
enum ViewProcessedScopeViewVariable {
    Node(ScopeViewVariable),
    VLine,
}

impl ViewProcessedScopeViewVariable {
    /// Returns `true` if the view processed name chars is [`Or`].
    ///
    /// [`Or`]: ViewProcessedNameChars::Or
    #[must_use]
    const fn is_v_line(&self) -> bool {
        matches!(self, Self::VLine)
    }

    const fn as_node(&self) -> Option<&ScopeViewVariable> {
        if let Self::Node(v) = self {
            Some(v)
        } else {
            None
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    fn try_into_node(self) -> Result<ScopeViewVariable, Self> {
        if let Self::Node(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }
}

// #[derive(Debug, Copy, Clone)]
// enum CCSSOp {
//     Add(Option<BinOp>),
//     // ────────────────────────────────────────────────────────────────────────────────
//     Eq(Option<BinOp>),
//     Lt(Option<BinOp>),
//     Le(Option<BinOp>),
//     Ge(Option<BinOp>),
//     Gt(Option<BinOp>),
// }

// impl Default for CCSSOp {
//     fn default() -> Self {
//         Self::Eq(None)
//     }
// }

#[derive(Debug, Clone, Display)]
#[display(" {op} {var}")]
struct CCSSOpSvv {
    op: PredOp,
    var: ScopeViewVariable,
}
impl ToTokens for CCSSOpSvv {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { op, var } = self;
        quote_spanned! (op.span().join(var.span()).unwrap() => emg_bind::layout::ccsa::CCSSOpSvv::new(#op,#var))
            .to_tokens(tokens);
    }
}
impl CCSSOpSvv {
    const fn new(op: PredOp, var: ScopeViewVariable) -> Self {
        Self { op, var }
    }
}

#[derive(Debug, Clone)]
pub struct CCSSSvvOpSvvExpr {
    svv: ScopeViewVariable,
    op_exprs: Vec<CCSSOpSvv>,
}
impl ToTokens for CCSSSvvOpSvvExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { svv, op_exprs } = self;
        quote_spanned! (svv.span() => emg_bind::layout::ccsa::CCSSSvvOpSvvExpr::new(#svv,vec![#(#op_exprs),*]))
            .to_tokens(tokens);
    }
}

// impl ToTokens for CCSSSvvOpSvvExpr {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         let Self { svv, op_exprs } = self;
//         let mut op_exprs_tokens = TokenStream::new();
//         for op_expr in op_exprs {
//             op_expr.to_tokens(&mut op_exprs_tokens);
//         }
//         quote_spanned! (svv.span().join(op_exprs_tokens.span()).unwrap() => #svv #op_exprs_tokens)
//             .to_tokens(tokens);
//     }
// }

impl std::fmt::Display for CCSSSvvOpSvvExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.svv)?;
        for op in &self.op_exprs {
            write!(f, "{op}")?;
        }
        Ok(())
    }
}

impl CCSSSvvOpSvvExpr {
    const fn new(svv: ScopeViewVariable, op_exprs: Vec<CCSSOpSvv>) -> Self {
        Self { svv, op_exprs }
    }
    const fn new_var(svv: ScopeViewVariable) -> Self {
        Self {
            svv,
            op_exprs: vec![],
        }
    }
}

#[derive(Debug, Clone, Display)]
#[display("{eq} {expr}")]
struct CCSSEqExpression {
    eq: PredEq,
    expr: CCSSSvvOpSvvExpr,
}
impl ToTokens for CCSSEqExpression {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { eq, expr } = self;
        quote_spanned!(eq.span().join(expr.span()).unwrap()=> emg_bind::layout::ccsa::CCSSEqExpression::new(#eq,#expr))
            .to_tokens(tokens);
    }
}
impl CCSSEqExpression {
    const fn new(eq: PredEq, expr: CCSSSvvOpSvvExpr) -> Self {
        Self { eq, expr }
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct CCSS {
    svv_op_svvs: CCSSSvvOpSvvExpr,
    eq_exprs: Vec<CCSSEqExpression>,
    opt_sw: Option<StrengthAndWeight>,
}
impl ToTokens for CCSS {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            svv_op_svvs,
            eq_exprs,
            opt_sw,
        } = self;
        let opt_sw_quote = QuoteOption(opt_sw.as_ref());

        quote_spanned!(svv_op_svvs.span()=>emg_bind::layout::ccsa::CCSS::new(#svv_op_svvs, vec![#(#eq_exprs),*],#opt_sw_quote)).to_tokens(tokens);
    }
}
impl std::fmt::Display for CCSS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            svv_op_svvs: var_op_vars,
            eq_exprs,
            opt_sw: sw,
        } = self;
        let sw_str = disp_opt(sw.as_ref());
        write!(f, "{var_op_vars} ")?;
        for eqe in eq_exprs {
            write!(f, "{eqe}")?;
        }
        write!(f, "{sw_str}")
    }
}
#[derive(Debug, Clone)]
struct ViewProcessed {
    view: ViewProcessedScopeViewVariable,
    is_splat: bool,
    is_point: bool,
    pos: Option<Point>,
    connection: Option<Connection>,
    pred: Option<Predicate>,
}

/// - ( `NameCharsOrNumber`[`Predicate`]? ) `[- ~]` ...
/// - ( `NameCharsOrNumber`[`Predicate`]? )
/// - `< Predicate >`
/// - |

#[derive(Clone)]
enum ViewObj {
    Splat(Splat),
    ViewSelector(ViewSelector),
    Point(Point),

    /// NOTE  "|"
    VLine,
}

impl ViewObj {
    fn processe(&self) -> ViewProcessed {
        match self {
            Self::Splat(x) => {
                if x.opt_connection.is_none() {}

                ViewProcessed {
                    view: ViewProcessedScopeViewVariable::Node(x.view_selector.view.clone()),
                    is_splat: true,
                    is_point: false,
                    pos: None,
                    connection: x.opt_connection.clone(),
                    pred: None,
                }
            }
            Self::ViewSelector(x) => ViewProcessed {
                view: ViewProcessedScopeViewVariable::Node(x.view.clone()),
                is_splat: false,
                is_point: false,
                pos: None,
                connection: None,
                pred: x.pred.clone(),
            },
            Self::Point(x) => ViewProcessed {
                view: ViewProcessedScopeViewVariable::VLine,
                is_splat: false,
                is_point: true,
                pos: Some(x.clone()),
                connection: None,
                pred: None,
            },
            Self::VLine => ViewProcessed {
                view: ViewProcessedScopeViewVariable::VLine,
                is_splat: false,
                is_point: false,
                pos: None,
                connection: None,
                pred: None,
            },
        }
    }
    fn clone_selector_in(&self, selectors: &mut Vec<ScopeViewVariable>) {
        match self {
            Self::Splat(splat) => selectors.push(splat.view_selector.view.clone()),
            Self::ViewSelector(vs) => selectors.push(vs.view.clone()),
            Self::Point(_) | Self::VLine => (),
        }
    }
}

impl std::fmt::Debug for ViewObj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Splat(arg0) => f.debug_tuple("Splat").field(arg0).finish(),
            Self::ViewSelector(arg0) => f.debug_tuple("ViewSelector").field(arg0).finish(),
            Self::Point(arg0) => f.debug_tuple("Point").field(arg0).finish(),
            Self::VLine => write!(f, "|"),
        }
    }
}
impl Parse for ViewObj {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _span = debug_span!("in view ---------").entered();
        if input.peek(token::Paren) {
            debug!("peek ()");
            let fork = input.fork();
            if let Ok(splat) = fork.parse::<Splat>() {
                input.advance_to(&fork);
                debug!("got Splat");
                return Ok(Self::Splat(splat));
            }
            debug!("not Splat");

            if let Ok(view_selector) = input.parse::<ViewSelector>() {
                debug!("got ViewSelector");
                return Ok(Self::ViewSelector(view_selector));
            }
            debug!("not ViewSelector");
        }

        if input.peek(Token![<]) {
            debug!("peek < ");
            let fork = input.fork();
            if let Ok(point) = fork.parse::<Point>() {
                debug!("got Point");
                input.advance_to(&fork);
                return Ok(Self::Point(point));
            }
        }
        debug!("not Point");

        input.parse::<Token![|]>()?;
        debug!("got |");
        Ok(Self::VLine)
    }
}

/// `[- ~ ]? `
/// with........
/// - ( `NameCharsOrNumber` [`Predicate`]? ) `[- ~]` ...
/// - ( `NameCharsOrNumber` [`Predicate`]? )
/// - `< Predicate >`
/// - |
#[derive(Debug, Clone)]
struct ConnectionView {
    opt_connection: Option<Connection>,
    view_obj: ViewObj,
}

impl ConnectionView {
    fn clone_selector_in(&self, selectors: &mut Vec<ScopeViewVariable>) {
        self.view_obj.clone_selector_in(selectors);
    }
}

impl Parse for ConnectionView {
    #[instrument(name = "ConnectionView")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug!("in ConnectionView,  parse-> [- ~] [`View`] ");
        Ok(Self {
            opt_connection: input.parse::<Connection>().ok(),
            view_obj: input.parse::<ViewObj>()?,
        })
    }
}

/// `[== < > >= <=]? NameCharsOrNumber? [== < > >= <=]? StrengthAndWeight?`
#[derive(Debug, Clone)]
struct ChainPredicateItem {
    head_eq: Option<PredEq>,
    value: Option<ScopeViewVariable>,
    tail_eq: Option<PredEq>,
    s: Option<StrengthAndWeight>,
}

impl Default for ChainPredicateItem {
    fn default() -> Self {
        Self {
            head_eq: Some(PredEq::default()),
            value: None,
            tail_eq: None,
            s: None,
        }
    }
}

// TODO: make more DRY... if all rules are optional sends parser into infinite loop
impl Parse for ChainPredicateItem {
    #[instrument(name = "ChainPredicateItem")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let head_eq = input.parse::<PredEq>().ok();
        let value = input.parse::<ScopeViewVariable>().ok();
        let tail_eq = input.parse::<PredEq>().ok();
        let s = input.parse::<StrengthAndWeight>().ok();
        assert!(
            !(head_eq.is_none() && value.is_none() && s.is_none()),
            "PredEq/NameCharsOrNumber/StrengthAndWeight must has one"
        );
        Ok(Self {
            head_eq,
            value,
            tail_eq,
            s,
        })
    }
}

/// `([== < > >= <=]? NameCharsOrNumber? [== < > >= <=]? StrengthAndWeight? , ...)`
#[derive(Debug, Clone)]
struct ChainPredicate(Vec<ChainPredicateItem>);
impl Parse for ChainPredicate {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _paren_token = parenthesized!(content in input);
        let content: Punctuated<ChainPredicateItem, Token![,]> =
            content.parse_terminated(ChainPredicateItem::parse)?;

        Ok(Self(content.into_iter().collect()))
    }
}

/// `chain-xxx([== < > >= <=]? NameCharsOrNumber? [== < > >= <=]? StrengthAndWeight? ,  ...)`
#[derive(Debug, Clone)]
struct Chain {
    prop: Ident, //chain-[what]
    preds: ChainPredicate,
}
impl Parse for Chain {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw_opt::chain>()?;
        input.parse::<Token![-]>()?;
        let prop: Ident = input.call(Ident::parse_any)?;
        if input.peek(token::Paren) {
            let preds: ChainPredicate = input.parse()?;
            return Ok(Self { prop, preds });
        }

        Ok(Self {
            prop,
            preds: ChainPredicate(vec![]),
        })
    }
}
#[derive(Debug, Clone)]
enum OptionItem {
    Chain(Chain),
    In(ScopeViewVariable),
    Gap(ScopeViewVariable),
    OuterGap(ScopeViewVariable),
    SW(StrengthAndWeight),
}

impl OptionItem {
    fn key(&self) -> String {
        match self {
            Self::Chain(_) => "Chain".to_string(),
            Self::In(_) => "In".to_string(),
            Self::Gap(_) => "Gap".to_string(),
            Self::OuterGap(_) => "OuterGap".to_string(),
            Self::SW(_) => "SW".to_string(),
        }
    }

    const fn as_outer_gap(&self) -> Option<&ScopeViewVariable> {
        if let Self::OuterGap(v) = self {
            Some(v)
        } else {
            None
        }
    }

    const fn as_gap(&self) -> Option<&ScopeViewVariable> {
        if let Self::Gap(v) = self {
            Some(v)
        } else {
            None
        }
    }

    const fn as_in(&self) -> Option<&ScopeViewVariable> {
        if let Self::In(v) = self {
            Some(v)
        } else {
            None
        }
    }

    const fn as_sw(&self) -> Option<&StrengthAndWeight> {
        if let Self::SW(v) = self {
            Some(v)
        } else {
            None
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    fn try_into_chain(self) -> Result<Chain, Self> {
        if let Self::Chain(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    /// Returns `true` if the option item is [`Chain`].
    ///
    /// [`Chain`]: OptionItem::Chain
    #[must_use]
    const fn is_chain(&self) -> bool {
        matches!(self, Self::Chain(..))
    }
}

impl Parse for OptionItem {
    #[instrument(name = "OptionItem")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(kw_opt::chain) && input.peek2(Token![-]) {
            debug!("peek chain-");
            return Ok(Self::Chain(input.parse()?));
        }
        if input.peek(Token![in]) && input.peek2(token::Paren) {
            debug!("peek in()");
            input.parse::<Token![in]>()?;

            let content;
            let _paren_token = parenthesized!(content in input);

            //TODO 原版使用空格, 这里使用 “,” ,测试是否有问题
            let content: Punctuated<ScopeViewVariable, Token![,]> =
                content.parse_terminated(ScopeViewVariable::parse)?;
            return Ok(Self::In(content.first().cloned().unwrap()));
        }
        if input.peek(kw_opt::gap) && input.peek2(token::Paren) {
            debug!("peek gap()");

            input.parse::<kw_opt::gap>()?;

            let content;
            let _paren_token = parenthesized!(content in input);

            //TODO 原版使用空格, 这里使用 “,” ,测试是否有问题
            let content: Punctuated<ScopeViewVariable, Token![,]> =
                content.parse_terminated(ScopeViewVariable::parse)?;

            return Ok(Self::Gap(content.first().cloned().unwrap()));
        }
        if input.peek(kw_opt::outer) && input.peek2(Token![-]) && input.peek3(kw_opt::gap) {
            debug!("peek outer-gap kw");
            input.parse::<kw_opt::outer>()?;
            input.parse::<Token![-]>()?;
            input.parse::<kw_opt::gap>()?;

            let content;
            let _paren_token = parenthesized!(content in input);

            //TODO 原版使用空格, 这里使用 “,” ,测试是否有问题
            let content: Punctuated<ScopeViewVariable, Token![,]> =
                content.parse_terminated(ScopeViewVariable::parse)?;

            return Ok(Self::OuterGap(content.first().cloned().unwrap()));
        }
        if input.peek(Token![!]) {
            let sw = input.parse::<StrengthAndWeight>()?;
            debug!("got sw");
            return Ok(Self::SW(sw));
        }

        panic!("OptionItem parse failed");
    }
}

#[derive(Debug)]
struct Options {
    map: HashMap<String, OptionItem>,
    chains: Vec<Chain>,
}
impl Parse for Options {
    #[instrument(name = "Options")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut os = HashMap::new();
        let mut chains = vec![];
        while let Some(option_item) = (!input.is_empty())
            .then(|| input.parse::<OptionItem>().ok())
            .flatten()
        {
            debug!("got optionItem: {:?}", &option_item);
            if option_item.is_chain() {
                chains.push(option_item.try_into_chain().unwrap());
            } else {
                os.insert(option_item.key(), option_item);
            }
        }
        Ok(Self { map: os, chains })
    }
}

fn get_left_var(
    view: &ViewProcessedScopeViewVariable,
    d: &Dimension,
    o: &Options,
    view_obj: &ViewProcessed,
) -> (ScopeViewVariable, Vec<CCSSOpSvv>) {
    if view_obj.is_point {
        let pos = view_obj.pos.clone().unwrap();
        // if (pos.0).0.len() > 1 {
        //     panic!("[get_left_var] can't support point expression is not single ScopeViewVariable current now");
        // }
        let PredExpression(v, op_s) = pos.0;
        (
            v,
            op_s.into_iter()
                .map(|(op, var)| CCSSOpSvv::new(op, var))
                .collect(),
        )
    } else if view.is_v_line() {
        (
            get_super_view_name(o).with_variable(super_left_var_names(d)),
            vec![],
        )
    } else if view_obj.is_splat {
        (
            view.as_node()
                .map(|v| v.make_last(Some(left_var_names(d))))
                .unwrap(),
            vec![],
        )
    } else {
        (
            view.as_node()
                .cloned()
                .map(|v| v.with_variable(left_var_names(d)))
                .unwrap(),
            vec![],
        )
    }
}
fn get_right_var(
    view: &ViewProcessedScopeViewVariable,
    d: &Dimension,
    o: &Options,
    view_obj: &ViewProcessed,
) -> (ScopeViewVariable, Vec<CCSSOpSvv>) {
    if view_obj.is_point {
        let pos = view_obj.pos.clone().unwrap();
        // if (pos.0).0.len() > 1 {
        //     panic!("[get_right_var] can't support point expression is not single ScopeViewVariable current now");
        // }

        let PredExpression(v, op_s) = pos.0;
        (
            v,
            op_s.into_iter()
                .map(|(op, var)| CCSSOpSvv::new(op, var))
                .collect(),
        )
    } else if view.is_v_line() {
        (
            get_super_view_name(o).with_variable(super_right_var_names(d)),
            vec![],
        )
    } else if view_obj.is_splat {
        (
            view.as_node()
                .map(|v| v.make_first(Some(right_var_names(d))))
                .unwrap(),
            vec![],
        )
    } else {
        (
            view.as_node()
                .cloned()
                .map(|v| v.with_variable(right_var_names(d)))
                .unwrap(),
            vec![],
        )
    }
}

fn get_super_view_name(o: &Options) -> ScopeViewVariable {
    o.map.get("In").map_or_else(
        || ScopeViewVariable::new_scope(Scope::Local(Span::call_site())),
        |in_name| in_name.as_in().cloned().unwrap(),
    )
}

#[derive(Debug)]
pub struct VFLStatement {
    d: Dimension,
    head: ViewObj,
    tails: Vec<ConnectionView>,
    o: Options,
    selectors: Vec<ScopeViewVariable>,
    pub ccsss: Vec<CCSS>,
}
impl ToTokens for VFLStatement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ccsss = &self.ccsss;
        let selectors = &self.selectors;
        quote! { (emg_bind::common::im::vector![ #( #ccsss ), * ],emg_bind::common::im::vector![ #( #selectors ), * ]) }
            .to_tokens(tokens);
    }
}

impl VFLStatement {
    fn get_op_gap(&self, opt_gap: Option<&Gap>, with_container: bool) -> Option<CCSSOpSvv> {
        let g: Option<CCSSOpSvv>;
        if let Some(gap) = opt_gap {
            match gap {
                Gap::None => {
                    g = None;
                }
                Gap::Standard => {
                    if with_container && self.o.map.contains_key("OuterGap") {
                        g = Some(CCSSOpSvv {
                            op: PredOp::new_add(),
                            var: self
                                .o
                                .map
                                .get("OuterGap")
                                .and_then(OptionItem::as_outer_gap)
                                .cloned()
                                .unwrap(),
                        });
                    } else if self.o.map.contains_key("Gap") {
                        g = Some(CCSSOpSvv {
                            op: PredOp::new_add(),
                            var: self
                                .o
                                .map
                                .get("Gap")
                                .and_then(OptionItem::as_gap)
                                .cloned()
                                .unwrap(),
                        });
                    } else {
                        g = Some(CCSSOpSvv {
                            op: PredOp::new_add(),
                            var: ScopeViewVariable::new_var(standard_gap_names(&self.d)),
                        });
                    }
                }
                Gap::Var(x) => {
                    g = Some(CCSSOpSvv {
                        op: PredOp::new_add(),
                        var: x.clone(),
                    });
                }
            }
        } else {
            g = None;
        }
        g
    }

    fn get_connection_string(
        &self,
        c: Option<&Connection>,
        with_container: bool,
    ) -> (Option<CCSSOpSvv>, PredEq) {
        let gap = c.map(Connection::gap);
        let op_gap = self.get_op_gap(gap, with_container);
        let connection_op = c.map(Connection::op).unwrap_or_default();
        (op_gap, connection_op)
    }

    fn add_splat_if_needed(&mut self, view_processed: &ViewProcessed) {
        if view_processed.is_splat {
            let mut view_var = view_processed.view.as_node().cloned().unwrap();
            view_var.set_variable(left_var_names(&self.d));
            // ────────────────────────────────────────────────────────────────────────────────
            let (op, eq) = self.get_connection_string(view_processed.connection.as_ref(), false);
            // @ left ─────────────────────────────────────────────────────────────────
            let var_op_vars = CCSSSvvOpSvvExpr {
                svv: view_var.clone(),
                op_exprs: op.map_or(vec![], |op_one| vec![op_one]),
            };

            // @right ─────────────────────────────────────────────────────────────────

            let right = view_var.into_next(Some(right_var_names(&self.d)));
            let eq_exprs = vec![CCSSEqExpression {
                eq,
                expr: CCSSSvvOpSvvExpr::new_var(right),
            }];
            let ccss = CCSS {
                svv_op_svvs: var_op_vars,
                eq_exprs,
                opt_sw: None, //TODO check no sw?
            };
            self.ccsss.push(ccss);
        }
    }

    fn add_preds(&mut self, view: &ViewProcessedScopeViewVariable, opt_preds: Option<&Predicate>) {
        if let Some(preds) = opt_preds {
            //NOTE has like  ( <=100!required,>=30!strong100 )
            for pred in &preds.0 {
                let node = view
                    .as_node()
                    .cloned()
                    .unwrap()
                    .or_with_variable(size_var_names(&self.d));

                let eq = pred.pred_eq;

                let right_var = pred
                    .pred_expression
                    .0
                    .clone()
                    .or_with_variable(size_var_names(&self.d));

                let mut op_exprs: Vec<CCSSOpSvv> = vec![];
                for (op, view) in pred.pred_expression.1.clone() {
                    let var = if view.variable_is_none() {
                        view.clone().or_with_variable(size_var_names(&self.d))
                    } else {
                        view
                    };

                    op_exprs.push(CCSSOpSvv::new(op, var));
                }

                let sw = pred.strength_and_weight.clone();
                let ccss = CCSS {
                    svv_op_svvs: CCSSSvvOpSvvExpr::new_var(node),
                    eq_exprs: vec![CCSSEqExpression::new(
                        eq,
                        CCSSSvvOpSvvExpr::new(right_var, op_exprs),
                    )],
                    opt_sw: sw,
                };
                self.ccsss.push(ccss);
            }
        }
    }
    fn add_chains(&mut self, views: &Vector<ViewProcessedScopeViewVariable>) {
        let chains = &self.o.chains;
        if !chains.is_empty() {
            for chain in chains {
                let chain_var = &chain.prop;
                let mut preds = chain.preds.clone();
                if preds.0.is_empty() {
                    // TODO  do it in parse for Chain?
                    preds = ChainPredicate(vec![ChainPredicateItem::default()]);
                }
                for pred in &preds.0 {
                    let mut views_clone = views.clone();

                    let first = views_clone
                        .pop_front()
                        .and_then(|v| v.try_into_node().ok())
                        .map(|v| v.with_variable(chain_var.clone()))
                        .expect("must have one view");
                    let var_op_vars = CCSSSvvOpSvvExpr::new_var(first);
                    let mut eq_exprs = vec![];
                    let eq = pred.head_eq.unwrap_or_default();

                    for view in views_clone {
                        let var = view
                            .try_into_node()
                            .ok()
                            .unwrap()
                            .with_variable(chain_var.clone());
                        let right_view_var_op_vars = CCSSSvvOpSvvExpr::new_var(var);

                        if pred.value.is_some() {
                            let right_chain_var_op_vars =
                                CCSSSvvOpSvvExpr::new_var(pred.value.clone().unwrap());
                            eq_exprs.push(CCSSEqExpression::new(eq, right_chain_var_op_vars));

                            let tail_eq = pred.tail_eq.unwrap_or_else(|| eq.chain_tail_eq_map());
                            eq_exprs.push(CCSSEqExpression::new(tail_eq, right_view_var_op_vars));
                        } else {
                            eq_exprs.push(CCSSEqExpression::new(eq, right_view_var_op_vars));
                        }
                    }
                    if !eq_exprs.is_empty() {
                        let ccss = CCSS {
                            svv_op_svvs: var_op_vars,
                            eq_exprs,
                            opt_sw: pred.s.clone(),
                        };
                        self.ccsss.push(ccss);
                    }
                }
            }
        }
    }
    #[instrument(skip(self))]
    fn build(&mut self) {
        debug!("in build");
        //   p.addSplatIfNeeded(head, d, o);
        let mut chained_views = vector![];
        let mut head_view_obj = self.head.processe();
        self.add_splat_if_needed(&head_view_obj);
        let mut head_view = head_view_obj.view.clone();
        if !head_view.is_v_line() {
            chained_views.push_back(head_view.clone());
        }
        self.add_preds(&head_view, head_view_obj.pred.as_ref());

        let sw = self.o.map.get("SW").and_then(OptionItem::as_sw).cloned();
        debug!("tail {:#?}", self.tails);

        // let mut hold_right = None;

        for tail in self.tails.clone() {
            debug!("in tail {:#?}", tail);
            let connection = tail.opt_connection.clone();
            let tail_view_obj = tail.view_obj.processe();
            self.add_splat_if_needed(&tail_view_obj);
            let tail_view = tail_view_obj.view.clone();
            if !tail_view.is_v_line() {
                chained_views.push_back(tail_view.clone());
            }
            self.add_preds(&tail_view, tail_view_obj.pred.as_ref());
            //result = [...]
            if !(head_view_obj.is_point && tail_view_obj.is_point) {
                debug!("不全部是 point",);
                //NOTE 不全部是 point
                //NOTE used for out-gap
                let with_container = (head_view.is_v_line() || tail_view.is_v_line())
                    && !(head_view_obj.is_point || tail_view_obj.is_point); //NOTE 都不是
                                                                            // • • • • •
                let (left_v, mut left_point_op_var) =
                    get_left_var(&head_view, &self.d, &self.o, &head_view_obj);
                let (opt_op_gap, eq) =
                    self.get_connection_string(connection.as_ref(), with_container);
                let (right_v, right_point_op_var) =
                    get_right_var(&tail_view, &self.d, &self.o, &tail_view_obj);
                // • • • • •
                if let Some(op_gap) = opt_op_gap {
                    left_point_op_var.push(op_gap);
                }
                let left_var_op_vars = CCSSSvvOpSvvExpr {
                    svv: left_v,
                    op_exprs: left_point_op_var,
                };

                let right_var_op_vars = CCSSSvvOpSvvExpr {
                    svv: right_v,
                    op_exprs: right_point_op_var,
                };
                let eq_exprs = vec![CCSSEqExpression::new(eq, right_var_op_vars)];
                debug!("======== sw: {:?}", &sw);

                let ccss = CCSS {
                    svv_op_svvs: left_var_op_vars,
                    eq_exprs,
                    opt_sw: sw.clone(),
                };

                self.ccsss.push(ccss);
            }
            head_view_obj = tail_view_obj;
            head_view = tail_view;
        }
        self.add_chains(&chained_views);
    }
}

impl Parse for VFLStatement {
    #[instrument(name = "VFLStatement")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut selectors = vec![];

        // @Dimension ─────────────────────────────────────────────────────────────────
        let d: Dimension = input.parse()?;
        debug!("dim -- {:?}", &d);

        // @ head View ─────────────────────────────────────────────────────────────────
        let head: ViewObj = input.parse()?;
        head.clone_selector_in(&mut selectors);
        debug!("got view -- {:?}", &head);

        // @ tail ConnectionView ─────────────────────────────────────────────────────────────────
        let mut tails = vec![];

        let tail_span = debug_span!("getting gail now -----------");

        tail_span.in_scope(|| {
            while let Ok(connection_view) = input.parse::<ConnectionView>() {
                connection_view.clone_selector_in(&mut selectors);
                tails.push(connection_view);
            }
            debug!("tail -- {:?}", &tails);
        });

        // @ options ────────────────────────────────────────────────────────────────────────────────
        let opt_span = debug_span!("getting options now -----------");
        let opt_g = opt_span.enter();
        debug!("input -- {:?}", &input);
        let o: Options = input.parse()?;
        debug!("got Options: {:?}", &o);
        drop(opt_g);

        // @ ─────────────────────────────────────────────────────────────────

        // d: Dimension,
        // head: ViewObj,
        // tails: Vec<ConnectionView>,
        // o: Options,
        // selectors: Vec<NameCharsOrNumber>,
        // ccsss: Vec<CCSS>,

        let mut vfl_statement = Self {
            d,
            head,
            tails,
            o,
            selectors,
            ccsss: vec![],
        };
        vfl_statement.build();

        Ok(vfl_statement)
    }
}

#[derive(Debug)]
pub struct Virtual(LitStr, Punctuated<GeneralVar, Token![,]>);
impl ToTokens for Virtual {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self(name, vars) = self;
        let vars_iter = vars.iter();
        quote_spanned!(name.span().join(vars.span()).unwrap()=>emg_bind::layout::ccsa::Virtual(emg_bind::common::IdStr::new(#name),vec![ #(#vars_iter),* ]))
            .to_tokens(tokens);
    }
}
impl Parse for Virtual {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lit_str: LitStr = input.parse()?;
        input.parse::<Token![:]>()?;

        let content;

        let _bracket_token = braced!(content in input);

        let content: Punctuated<GeneralVar, Token![,]> =
            content.parse_terminated(GeneralVar::parse)?;

        Ok(Self(lit_str, content))
    }
}

#[derive(Debug)]
pub struct GeneralVar(Ident, ScopeViewVariable);
impl ToTokens for GeneralVar {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self(name, svv) = self;

        let name_str = name.to_string();
        quote_spanned!(name.span().join(svv.span()).unwrap()=>emg_bind::layout::ccsa::GeneralVar(emg_bind::common::IdStr::new(#name_str),#svv))
            .to_tokens(tokens);
    }
}

impl Parse for GeneralVar {
    #[instrument(name = "GeneralVar")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // let name: LitStr = input.parse()?;
        let name = input.call(Ident::parse_any)?;
        input.parse::<Token![==]>()?;
        let scope_view_variable = input.parse()?;
        Ok(Self(name, scope_view_variable))
    }
}
#[derive(Debug)]
pub enum DefineVar {
    General(GeneralVar),
    Virtual(Virtual),
}
impl ToTokens for DefineVar {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::General(general_var) => {
                quote_spanned!(general_var.span()=> emg_bind::layout::ccsa::CassowaryVar::General( #general_var ))
                    .to_tokens(tokens);
            }
            Self::Virtual(virtual_) => {
                quote_spanned!(virtual_.span()=> emg_bind::layout::ccsa::CassowaryVar::Virtual( #virtual_ ))
                .to_tokens(tokens);
            }
        }
    }
}
impl Parse for DefineVar {
    #[instrument(name = "CassowaryVar")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) && input.peek2(Token![:]) {
            return Ok(Self::Virtual(input.parse()?));
        }
        // else if content.peek(Ident::peek_any) && content.peek2(Token![==]) {
        //     Ok(Self::General(input.parse()?))
        // }

        Ok(Self::General(input.parse()?))
    }
}

#[derive(Debug)]
pub enum Cassowary {
    Vfl(Box<VFLStatement>),
    CassowaryVars(Punctuated<DefineVar, Token![,]>),
    CCss,
}
impl ToTokens for Cassowary {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Vfl(vfl) => {
                quote_spanned!(vfl.span()=> #vfl).to_tokens(tokens);
            }
            Self::CassowaryVars(vars) => {
                let vars_iter = vars.iter();
                quote_spanned!(vars.span()=>vec![ #(#vars_iter),* ]).to_tokens(tokens);
            }
            Self::CCss => todo!(),
        }
    }
}

impl Parse for Cassowary {
    #[instrument(name = "Cassowary")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _bracket_token = braced!(content in input);
        debug!(" in Cassowary find braced \n content-> {:?}", &content);
        // let ident_h = Ident::new("@h", Span::call_site());

        if content.peek(Token![@]) {
            debug!(" find @ , is VFL");
            Ok(Self::Vfl(content.parse()?))
        } else {
            debug!(" not VFL");

            let content: Punctuated<DefineVar, Token![,]> =
                content.parse_terminated(DefineVar::parse)?;

            Ok(Self::CassowaryVars(content))
        }
    }
}

#[cfg(test)]
mod tests {

    use std::path::Path;

    use emg_common::VecDisp;
    use quote::ToTokens;
    use tracing::debug;

    use crate::{cassowary::VFLStatement, Gtree};
    use tracing_subscriber::{prelude::*, registry::Registry};

    fn token_expect_error(_name: &str, input: &str) {
        // ────────────────────────────────────────────────────────────────────────────────

        let subscriber = Registry::default().with(tracing_tree::HierarchicalLayer::new(2));
        // .with(subscriber1);
        tracing::subscriber::set_global_default(subscriber).ok();

        // ─────────────────────────────────────────────────────────────────

        insta::with_settings!({snapshot_path => Path::new("./vfl_snap")}, {

            debug!("=========== parse \n {:?}\n",&input);

            match syn::parse_str::<VFLStatement>(input) {
                Ok(ok) => {
                    panic!("should error =============\n{:#?}\n", &ok);
                     }
                Err(error) => println!("...{:?}", error),
            }
        });
    }

    fn token_test(name: &str, input: &str) {
        // ────────────────────────────────────────────────────────────────────────────────

        let subscriber = Registry::default().with(tracing_tree::HierarchicalLayer::new(2));
        // .with(subscriber1);
        tracing::subscriber::set_global_default(subscriber).ok();

        // ─────────────────────────────────────────────────────────────────

        insta::with_settings!({snapshot_path => Path::new("./vfl_snap")}, {

            debug!("=========== parse \n {:?}\n",&input);

            match syn::parse_str::<VFLStatement>(input) {
                Ok(ok) => {
                    println!("=============\n{:#?}\n", &ok);
                    // insta::assert_debug_snapshot!(name.to_string()+"_prase", &ok);

                    // ok.build();
                    println!("=================== build \n {:#?}\n", ok.ccsss);
                    // insta::assert_debug_snapshot!(name.to_string()+"_ccss", &ok.ccsss);
                    let disp = VecDisp(ok.ccsss);
                    println!("=================== build---display \n {}\n", &disp);

                    insta::assert_display_snapshot!(name.to_string()+"_ccss_display", disp);

                    // let x = format!("{}", ok.to_token_stream());
                    // println!("===================\n {}\n", x);

                    // assert_eq!(x.as_str(), r#"NameCharsOrNumber :: Id (IdStr :: new ("button"))"#)
                }
                Err(error) => panic!("...{:?}", error),
            }
        });
    }

    #[test]
    fn base() {
        println!();
        // ─────────────────────────────────────────────────────────────────
        let input = r#"
            @horizontal (#b1)(#b2)
        "#;

        token_test("base-simple", input);
        // ─────────────────────────────────────────────────────────────────
        // ─────────────────────────────────────────────────────────────────
        let input = r#"
                @h (#b1)(#b2)
            "#;

        token_test("base-shorthand", input);
        // ─────────────────────────────────────────────────────────────────
        // ─────────────────────────────────────────────────────────────────
        let input = r#"
                @vertical (#b1)(#b2)
            "#;

        token_test("base-vertical", input);
        // ─────────────────────────────────────────────────────────────────
        // ─────────────────────────────────────────────────────────────────
        let input = r#"
                @v (#b1)-(#b2)  -  (#b3)- (#b4) -(#b5)
            "#;

        token_test("base-standard-gaps", input);
        // ─────────────────────────────────────────────────────────────────
        // ─────────────────────────────────────────────────────────────────
        let input = r#"
                @v (#b1)-(#b2)  -  (#b3)- (#b4) -(#b5) !weak
            "#;

        token_test("base-sw-gaps", input);
        // ─────────────────────────────────────────────────────────────────
    }
    #[test]
    fn explicit_gap() {
        let input = r#"
                    @h (#b1)-100-(#b2)-8-(#b3)
            "#;

        token_test("explicit-gaps", input);
        let input = r#"
                    @h (#b1) - 100 - (#b2) - 8 - (#b3)
            "#;

        token_test("explicit-gaps2", input);
    }
    #[test]
    fn explicit_var_gap() {
        let input = r#"
                @h (#b1)-[my_gap]-(#b2)-[my_other_gap]-(#b3)
            "#;

        token_test("explicit-var-gaps", input);
    }
    #[test]
    fn mix_gap() {
        let input = r#"
        @h (#b1)(#b2)-(#b3)-100-(#b4) gap(20)
            "#;

        token_test("mix-gaps", input);
    }
    #[test]
    fn variable_standard_gap() {
        let input = r#"
        @h (#b1)-100-(#b2)-(#b3)-(#b4) gap([col_width])
            "#;

        token_test("variable-standard-gap", input);
    }
    #[test]
    fn view_variable_standard_gap() {
        let input = r#"
        @h (#b1)-100-(#b2)-(#b3)-(#b4) gap(#box1[width])
            "#;

        token_test("view-variable-standard-gap", input);
    }
    #[test]
    fn virtuals() {
        let input = r#"
        @v ("Zone")-("1")-("a")-("q-1")-("_fallout")
            "#;

        token_test("virtuals", input);
    }
    #[test]
    fn virtuals2() {
        //TODO make it auto
        let input = r#"
        @h (#b1)-("vv"(==#b2[left]-#b1[right]-[hgap]))-(#b2)
            "#;

        token_test("virtuals2", input);
    }
    #[test]
    fn virtuals3() {
        let input = r#"
        @v ("vv")-(#b1)
            "#;

        token_test("virtuals3", input);
    }
    #[test]
    fn err1() {
        let input = r#"
        @h (#b1(#b2)
            "#;

        token_expect_error("err1", input);
    }

    #[test]
    fn var_scope() {
        let input = r#"
        @h (#b1)-$[md]-(#b2)
            "#;

        token_test("var scope", input);
        let input = r#"
        @h (#b1)-$md-(#b2)
            "#;

        token_test("var scope2", input);
    }
    #[test]
    #[should_panic]
    fn var_scope_err() {
        let input = r#"
        @h (#b1)-$$md-(#b2)
            "#;

        syn::parse_str::<VFLStatement>(input).expect("err....");
    }

    #[test]
    fn parent() {
        let input = r#"
        @h (#b1)-^[md]-(#b2)
            "#;

        token_test("parent", input);
        let input = r#"
        @h (#b1)-^md-(#b2)
            "#;

        token_test("parent2", input);
        let input = r#"
        @h (#b1)-^^md-(#b2)
            "#;

        token_test("parent3", input);
    }

    #[test]
    fn local() {
        let input = r#"
        @h (#b1)-&[md]-(#b2)
            "#;

        token_test("local", input);
        let input = r#"
        @h (#b1)-&md-(#b2)
            "#;

        token_test("local2", input);
    }

    #[test]
    #[should_panic]
    fn local_err() {
        let input = r#"
        @h (#b1)-&&md-(#b2)
            "#;

        syn::parse_str::<VFLStatement>(input).ok();
    }

    #[test]
    fn element_containment_parent() {
        let input = r#"
        @v |(#sub)| in(#parent)
            "#;

        token_test("element_containment_parent", input);
    }
    #[test]
    fn element_containment_virtuals() {
        let input = r#"
        @v |(#sub)| in("parent")
            "#;

        token_test("element_containment_virtuals", input);
    }
    #[test]
    fn element_containment_default() {
        let input = r#"
        @v |(#sub)|
            "#;

        token_test("element_containment_default", input);
    }
    #[test]
    fn element_containment_view_gap() {
        let input = r#"
        @h |-(#sub1)-(#sub2)-| in(#parent)
            "#;

        token_test("element_containment_view_gap", input);
    }
    #[test]
    fn element_containment_view_explicit_gap() {
        let input = r#"
        @h |-1-(#sub)-2-| in(#parent)
            "#;

        token_test("element_containment_view_explicit_gap", input);
    }
    #[test]
    fn element_containment_outer_gap() {
        let input = r#"
        @h |-(#sub1)-(#sub2)-| in(#parent) outer-gap(10)
            "#;

        token_test("element_containment_outer_gap", input);
    }
    #[test]
    fn element_containment_outer_gap2() {
        let input = r#"
        @h |-(#sub1)-(#sub2)-| in(#parent) gap(8) outer-gap([baseline])
            "#;

        token_test("element_containment_outer_gap2", input);
    }
    #[test]
    #[should_panic]
    fn element_containment_err() {
        let input = r#"
        @h |-(#box]-
            "#;

        syn::parse_str::<VFLStatement>(input).expect("err....");
    }

    #[test]
    fn points() {
        let input = r#"
        @v <100>(#sub)<300>
            "#;

        token_test("points", input);
    }

    #[test]
    #[should_panic]
    fn point_containment() {
        //TODO support special node element, then, remove #[should_panic]
        let input = r#"
        @h < "col1"[center_x] + 20 > -(#box1)- < ::window[center_x] >
            "#;

        token_test("point_containment", input);
    }
    #[test]
    fn point_containment3() {
        let input = r#"
        @h < [line] >-(#box1)-(#box2)
            "#;

        token_test("point_containment3", input);
    }
    #[test]
    #[should_panic]
    fn point_containment_point_in_alignment() {
        let input = r#"
        @h (#btn1)-<::window[center_x]>-(#btn2) gap(8)
            "#;

        token_test("point_containment_point_in_alignment", input);
    }
    #[test]
    fn point_containment_point_in_alignment2() {
        let input = r#"
        @h (#btn1)-<&window[center_x]>-(#btn2) gap(8)
            "#;

        token_test("point_containment_point_in_alignment2", input);
    }
    #[test]
    #[should_panic]
    fn point_containment_chains() {
        let input = r#"
        @h (#btn1)-<::window[center_x]>-(#btn2) gap(8) chain-top chain-width(==)
            "#;

        token_test("point_containment_chains", input);
    }
    #[test]
    fn point_containment_chains2() {
        let input = r#"
        @h (#btn1)-<&window[center_x]>-(#btn2) gap(8) chain-top chain-width(==)
            "#;

        token_test("point_containment_chains2", input);
    }
    #[test]
    fn point_containment_consecutive_point() {
        let input = r#"
        @h (#btn1)- <"col3"[left]>
                    <"col4"[right]>-(#btn2)
                gap(8)
            "#;

        token_test("point_containment_consecutive_point", input);
    }
    #[test]
    fn point_containment_this_scope() {
        let input = r#"
        @h (#btn1)-<&[other_place]>
                       < &[center_x] >-(#btn2)
              gap(&[gap])
            "#;

        token_test("point_containment_this_scope", input);
    }
    #[test]
    #[should_panic]
    fn point_containment_complex_selectors() {
        //TODO support complex selectors
        let input = r#"
        @h (#btn1)-< (.box .foo:bar:next .black)[center_x] >
                       < (.box ! .foo:bar:next .black)[left] >-(#btn2)
              gap(&[gap])
            "#;

        token_test("point_containment_complex_selectors", input);
    }
    #[test]
    fn point_containment_this_scope2() {
        let input = r#"
                        @h | - (#btn1) - <&[right]>
                                        < &[right] > - (#btn2) - |
                        gap(&[gap])
                        outer-gap(&[outer_gap])
                        in(&)
            "#;

        token_test("point_containment_this_scope2", input);
    }
    #[test]
    fn cushion() {
        let input = r#"
        @h (#b1)~(#b2)
            "#;

        token_test("cushion", input);
    }
    #[test]
    fn cushion_gap() {
        let input = r#"
        @h (#b1)~-~(#b2)~100~(#b3)
            "#;

        token_test("cushion_gap", input);
    }
    #[test]
    fn cushion_super_view_with_cushions() {
        let input = r#"
        @h |~(#sub)~2~| in(#parent)
            "#;

        token_test("cushion_super_view_with_cushions", input);
    }
    #[test]
    fn predicates() {
        let input = r#"
        @v (#sub(==100))
            "#;

        token_test("predicates", input);
    }
    #[test]
    fn predicate_multiple_with_sw() {
        let input = r#"
        @v (#boox(<=100!required,>=30!strong(100)))
            "#;

        token_test("predicate_multiple_with_sw", input);
    }
    #[test]
    fn predicate_connected() {
        let input = r#"
        @h |(#b1(<=100))(#b2(==#b1))|
            "#;

        token_test("predicate_connected", input);
    }
    #[test]
    fn predicate_virtuals() {
        let input = r#"
        @h ("b1"(<=100)) ("b2"(=="b1"))
            "#;

        token_test("predicate_virtuals", input);
    }
    #[test]
    fn predicate_multiple_conneected_sw() {
        let input = r#"
        @h (#b1( <=100 , ==#b99 !weak(99) ))(#b2(>= #b1 *2  !weak(10), <=3!required))-100-(.b3(==200)) !medium(200)
            "#;

        token_test("predicate_multiple_conneected_sw", input);
    }
    #[test]
    fn predicate_constraint_variable() {
        let input = r#"
        @h (#b1(==[colwidth]))
            "#;

        token_test("predicate_constraint_variable", input);
    }
    #[test]
    fn predicate_explicit_view_var() {
        let input = r#"
        @h (#b1(==#b2[height]))
            "#;

        token_test("predicate_explicit_view_var", input);
    }
    #[test]
    fn chain() {
        let input = r#"
        @h (#b1)(#b2) chain-height chain-width(250)
            "#;

        token_test("chain", input);
    }
    #[test]
    fn chain_v() {
        let input = r#"
        @h (#b1)(#b2) chain-top chain-bottom(250)
            "#;

        token_test("chain_v", input);
    }
    #[test]
    fn chain_multiply() {
        let input = r#"
        @h (#b1)(#b2)(#b3) chain-width(==[colwidth]!strong,<=500!required)
            "#;

        token_test("chain_multiply", input);
    }
    #[test]
    fn chain_explicit_equality_inequality_chains() {
        let input = r#"
        @v (#b1)(#b2)(#b3)(#b4) chain-width(==!weak(10)) chain-height(<=150>= !required) !medium
            "#;

        token_test("chain_explicit_equality_inequality_chains", input);
    }
    #[test]
    fn chain_single_view_with_equality() {
        let input = r#"
        @v (#b1(==100!strong))(#b2) chain-centerX chain-width( 50 !weak(10))
            "#;

        token_test("chain_single_view_with_equality", input);
    }
    #[test]
    //TODO need support
    fn chain_single_view_with_equality2() {
        let input = r#"
        @v (#b1(==100!strong))(#b2) chain-centerX chain-width( [xx]-50 !weak(10))
            "#;

        token_test("chain_single_view_with_equality", input);
    }
    #[test]
    fn chain_adv_with_super_view_chains() {
        let input = r#"
        @v |-8-(#b1(==100!strong))(#b2)-8-| in(#panel) chain-centerX( #panel[centerX] !required) chain-width(>=50<= !weak(10))
            "#;

        token_test("chain_adv_with_super_view_chains", input);
    }
    #[test]
    fn chain_adv_with_virtuals() {
        let input = r#"
        @v |-(#b1)-(#b2)-| in("panel") gap("zone"[col_size]) outer-gap("outer-zone"[row_size]) chain-centerX( "panel"[centerX] !required)
            "#;

        token_test("chain_adv_with_virtuals", input);
    }
    #[test]
    fn splats() {
        let input = r#"
        @h (.box)...
            "#;

        token_test("splats", input);
    }
    #[test]
    fn splats2() {
        let input = r#"
        @h (.box)-10-...
            "#;

        token_test("splats2", input);
    }
    #[test]
    fn splats3() {
        let input = r#"
        @h (.box)-... gap(11)
            "#;

        token_test("splats3", input);
    }

    // ────────────────────────────────────────────────────────────────────────────────

    #[test]
    fn explicit_view_var_gap() {
        //TODO support?
        // let input = r#"
        //                         @v (#b1)
        //                             -#box1.class1[width]-
        //                         (#b2)
        //                             -"virtual"[-my-custom-prop]-
        //                         (#b3)
        //     "#;

        // token_test("explicit-view-var-gaps", input);
    }
    #[test]
    fn gap() {
        let input = r#"
                @v (#b1)-(#b2)-(#b3)-(#b4)-(#b5) gap(20)
            "#;

        token_test("explicit-standard-gaps", input);
        let input = r#"
                                @v (#b1)
                                    -
                                (#b2)
                                    -20-
                                (#b3)
                                    -20-
                                (#b4)
                                    -
                                (#b5)

                                gap(20)
            "#;

        token_test("explicit-standard-gaps2", input);
    }
    #[test]
    fn weak() {
        let input = r#"
                @v (#b1(>=100)) !required
            "#;

        token_test("sw-weak", input);
    }

    #[test]
    fn test_1() {
        // ────────────────────────────────────────────────────────────────────────────────
        // ────────────────────────────────────────────────────────────────────────────────

        let subscriber = Registry::default().with(tracing_tree::HierarchicalLayer::new(2));
        // .with(subscriber1);
        tracing::subscriber::set_global_default(subscriber).ok();

        // ─────────────────────────────────────────────────────────────────
        // ────────────────────────────────────────────────────────────────────────────────

        fn token_test(input: &str) {
            match syn::parse_str::<Gtree>(input) {
                Ok(ok) => println!("===>{}", ok.to_token_stream()),
                Err(error) => panic!("...{:?}", error),
            }
        }

        println!();
        let input = r#"
        @=root
                Layer [
                    @="x111x" @E=[{@h |(#button)...| }]
                    Layer [],
                    @=x111x @E=[{@h |(#button)...| in(#panel) gap(10) }]
                    Layer []

                ]

        "#;

        token_test(input);
    }
}
