/*
 * @Author: Rais
 * @Date: 2022-06-24 18:11:24
 * @LastEditTime: 2022-07-04 23:54:43
 * @LastEditors: Rais
 * @Description:
 */
use parse_display::{Display, FromStr};
use std::{collections::HashMap, rc::Rc};

use im_rc::{vector, Vector};
use proc_macro2::{Span, TokenStream};

use quote::{quote_spanned, ToTokens};
use syn::{
    braced, bracketed, parenthesized,
    parse::{discouraged::Speculative, Parse, ParseStream},
    punctuated::Punctuated,
    token, BinOp, Ident, LitFloat, LitInt, LitStr, Token,
};
use tracing::{debug, debug_span, instrument};

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
    #![warn(clippy::expl_impl_clone_on_copy)]

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
enum Number {
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
enum NameChars {
    #[display("#{0}")]
    Id(Ident), // #xxx
    #[display(".{0}")]
    Class(Ident), // .xxx
    #[display("{0}")]
    Element(Ident), // xxxx
    #[display("\"{0:?}\"")]
    Virtual(LitStr), //"xxx"
    #[display("{0}")]
    Number(Number), // 12 | 12.1
    #[display("{0}:next")]
    Next(Box<Self>), // TODO do parse
    #[display("{0}:last")]
    Last(Box<Self>),
    #[display("{0}:first")]
    First(Box<Self>),
}

impl NameChars {
    fn into_next(self) -> Self {
        assert!(!self.is_id());
        Self::Next(Box::new(self))
    }
    fn make_next(&self) -> Self {
        assert!(!self.is_id());
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
    /// [`Id`]: NameChars::Id
    #[must_use]
    fn is_id(&self) -> bool {
        matches!(self, Self::Id(..))
    }

    /// Returns `true` if the name chars is [`Number`].
    ///
    /// [`Number`]: NameChars::Number
    #[must_use]
    fn is_number(&self) -> bool {
        matches!(self, Self::Number(..))
    }
}

impl Parse for NameChars {
    #[instrument(name = "NameChars")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // TODO [a-zA-Z0-9#.\-_$:""&]

        if input.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            let name = input.parse::<Ident>()?;
            debug!("got id: #{:?}", &name);
            return Ok(Self::Id(name));
        }
        debug!("not id : {:?}", &input);

        if input.peek(Token![.]) {
            input.parse::<Token![.]>()?;
            let name = input.parse::<Ident>()?;
            debug!("got class: .{:?}", &name);
            return Ok(Self::Class(name));
        }
        debug!("not class : {:?}", &input);

        if input.peek(LitStr) {
            let r#virtual: LitStr = input.parse()?;
            debug!("got virtual: {:?}", &r#virtual);
            return Ok(Self::Virtual(r#virtual));
        }
        debug!("not Virtual : {:?}", &input);

        if input.peek(LitFloat) || input.peek(LitInt) {
            let n: Number = input.parse()?;
            debug!("got Number: {:?}", &n);

            return Ok(Self::Number(n));
        }
        debug!("not Number : {:?}", &input);

        let name = input.parse::<Ident>()?;
        debug!("got Element: {:?}", &name);
        Ok(Self::Element(name))
    }
}

impl ToTokens for NameChars {
    #[allow(clippy::match_same_arms)]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Id(x) => {
                let str = x.to_string();
                quote_spanned!(x.span()=> NameChars::Id(IdStr::new(#str))).to_tokens(tokens);
            }
            Self::Class(x) => {
                let str = x.to_string();
                quote_spanned!(x.span()=> NameChars::Class(IdStr::new(#str))).to_tokens(tokens);
            }
            Self::Element(x) => {
                let str = x.to_string();
                quote_spanned!(x.span()=> NameChars::Element(IdStr::new(#str))).to_tokens(tokens);
            }
            Self::Virtual(_) => todo!(),
            Self::Number(n) => match n {
                Number::Int(int) => {
                    quote_spanned!(int.span()=> NameChars::Number( NotNan::new(#int as f64).unwrap() ))
                        .to_tokens(tokens);
                }
                Number::Float(float) => {
                    quote_spanned!(float.span()=> NameChars::Number( NotNan::new(#float).unwrap() ))
                        .to_tokens(tokens);
                }
            },
            Self::Next(_) => todo!(),
            Self::Last(_) => todo!(),
            Self::First(_) => todo!(),
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

impl PredOp {
    fn new_add() -> Self {
        Self::Add(token::Add::default())
    }
    fn new_sub() -> Self {
        Self::Sub(token::Sub::default())
    }
    fn new_mul() -> Self {
        Self::Mul(token::Star::default())
    }
}
impl Parse for PredOp {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug!("in PredOp");

        let pred_op: BinOp = input.parse()?;
        match pred_op {
            BinOp::Add(x) => Ok(Self::Add(x)),
            BinOp::Sub(x) => Ok(Self::Sub(x)),
            BinOp::Mul(x) => Ok(Self::Mul(x)),
            _ => panic!("[PredOp] op not support :{:?}", pred_op),
        }
    }
}
// /// `123f64`
// #[derive(Debug, Clone)]
// struct PredLiteral(LitFloat);
// impl Parse for PredLiteral {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         debug!("in PredLiteral");

//         Ok(Self(input.parse()?))
//     }
// }

/// `[var]`
#[derive(Debug, Clone, Display)]
#[display("[{0}]")]
struct PredVariable(Ident);
impl Parse for PredVariable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug!("in PredVariable");

        let content;
        let _bracket_token = bracketed!(content in input);
        let var: Ident = content.parse()?;
        Ok(Self(var))
    }
}
fn disp_opt<T: std::fmt::Display>(o: Option<T>) -> String {
    o.map_or("".to_string(), |x| format!("{}", x))
}
/// `&name[var]`
#[derive(Debug, Clone)]
struct ScopeViewVariable {
    scope: Option<Scope>,
    view: Option<NameChars>,
    variable: Option<PredVariable>,
}

impl std::fmt::Display for ScopeViewVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let scope = self
            .scope
            .as_ref()
            .map_or("".to_string(), |x| format!("{}", x));
        let view = self
            .view
            .as_ref()
            .map_or("".to_string(), |x| format!("{}", x));
        let variable = self
            .variable
            .as_ref()
            .map_or("".to_string(), |x| format!("{}", x));

        write!(f, "{}{}{}", scope, view, variable)
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
        if !self.view.is_some_and(NameChars::is_number) {
            self.variable = Some(PredVariable(var));
        }
        self
    }
    #[must_use]
    fn or_with_variable(mut self, var: Ident) -> Self {
        if self.variable_is_none() && !self.view.is_some_and(NameChars::is_number) {
            self.variable = Some(PredVariable(var));
        }
        self
    }

    fn set_variable(&mut self, var: Ident) {
        if !self.view.is_some_and(NameChars::is_number) {
            self.variable = Some(PredVariable(var));
        }
    }
}
impl Parse for ScopeViewVariable {
    #[instrument(name = "ScopeViewVariable")]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let scope = input.parse();
        let view = input.parse();
        let variable = input.parse();
        // assert!(
        //     !(scope.is_none() && view.is_none() && variable.is_none()),
        //     "all none in ScopeViewVariable"
        // );

        if scope.is_err() && view.is_err() && variable.is_err() {
            let e = scope.err().unwrap();
            return Err(syn::Error::new(e.span(), "all none in ScopeViewVariable"));
        }

        Ok(Self {
            scope: scope.ok(),
            view: view.ok(),
            variable: variable.ok(),
        })
    }
}

/// `name[var]`
#[derive(Debug, Clone)]
struct PredViewVariable {
    view: NameChars,
    pred_variable: PredVariable,
}
impl Parse for PredViewVariable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug!("in PredViewVariable");

        Ok(Self {
            view: input.parse()?,
            pred_variable: input.parse()?,
        })
    }
}

/// `name`
#[derive(Debug, Clone)]
struct PredView {
    view: NameChars,
}
impl Parse for PredView {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug!("in PredView");

        Ok(Self {
            view: input.parse()?,
        })
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
enum PredExpressionItem {
    PredOp(PredOp),
    // PredLiteral(PredLiteral),
    ScopeViewVariable(ScopeViewVariable),
    //
    // PredVariable(PredVariable),
    // PredViewVariable(PredViewVariable),
    // PredView(PredView),
    // ViewPropInsideBuild(ViewProp),
}

impl PredExpressionItem {
    const fn as_scope_view_variable(&self) -> Option<&ScopeViewVariable> {
        if let Self::ScopeViewVariable(v) = self {
            Some(v)
        } else {
            None
        }
    }
}
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
        while !input.peek(Token![>]) && !input.is_empty() {
            if let Ok(x) = input.parse::<PredExpressionItem>() {
                match x {
                    PredExpressionItem::PredOp(x) if op.is_none() => {
                        op = Some(x);
                    }
                    PredExpressionItem::ScopeViewVariable(x) if op.is_some() => {
                        exps.push((op.take().unwrap(), x));
                    }
                    _ => panic!("[PredExpression] 运算顺序错误 ,必须 一个 op + 一个 var 一对 "),
                }
            } else if !input.is_empty() {
                panic!("[PredExpression] input not empty {:?}", input)
            } else {
                break;
            }
        }
        Ok(Self(first, exps))
    }
}
/// !weak10   !require
#[derive(Debug, Clone)]
enum StrengthAndWeight {
    Weak(Option<LitInt>),
    Medium(Option<LitInt>),
    Strong(Option<LitInt>),
    Require(Option<LitInt>),
}

impl std::fmt::Display for StrengthAndWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrengthAndWeight::Weak(x) => {
                if let Some(i) = x {
                    write!(f, "!weak{}", i)
                } else {
                    write!(f, "!weak")
                }
            }
            StrengthAndWeight::Medium(x) => {
                if let Some(i) = x {
                    write!(f, "!medium{}", i)
                } else {
                    write!(f, "!medium")
                }
            }
            StrengthAndWeight::Strong(x) => {
                if let Some(i) = x {
                    write!(f, "!strong{}", i)
                } else {
                    write!(f, "!strong")
                }
            }
            StrengthAndWeight::Require(x) => {
                if let Some(i) = x {
                    write!(f, "!require{}", i)
                } else {
                    write!(f, "!require")
                }
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
            if input.peek(LitInt) {
                debug!("got weak number");
                return Ok(Self::Weak(Some(input.parse()?)));
            }
            return Ok(Self::Weak(None));
        }
        debug!("not weak kw {:?}", &input);
        if input.parse::<kw_strength::medium>().is_ok() {
            if input.peek(LitInt) {
                return Ok(Self::Medium(Some(input.parse()?)));
            }
            return Ok(Self::Medium(None));
        }
        if input.parse::<kw_strength::strong>().is_ok() {
            if input.peek(LitInt) {
                return Ok(Self::Strong(Some(input.parse()?)));
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

        input.parse::<kw_strength::required>()?;
        debug!("find required keyword");
        if input.peek(LitInt) {
            return Ok(Self::Require(Some(input.parse()?)));
        }
        Ok(Self::Require(None))
    }
}

/// ` == < > >= <= `
#[derive(Debug, Copy, Clone, Display)]
enum PredEq {
    #[display("{0}")]
    Eq(#[display("==")] Token![==]),
    #[display("{0}")]
    Lt(#[display("<")] Token![<]),
    #[display("{0}")]
    Le(#[display("<=")] Token![<=]),
    #[display("{0}")]
    Ge(#[display(">=")] Token![>=]),
    #[display("{0}")]
    Gt(#[display(">")] Token![>]),
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
            _ => panic!("[PredEq] op not support :{:?}", pred_eq),
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
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug!("in PredicateItem");
        Ok(Self {
            pred_eq: input.parse()?,
            pred_expression: input.parse()?,
            strength_and_weight: input.parse().ok(),
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
        debug!("got ()");
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
        let _span = debug_span!("parse-> (`&name[var]`? (`Predicate`)?) ").entered();
        debug!("input: {:?}", &input);
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
    Local,
    #[display("^({0})")]
    Parent(u8),
    #[display("$")]
    Global,
}
impl Parse for Scope {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug!("in Scope");

        if input.parse::<Token![&]>().is_ok() {
            return Ok(Self::Local);
        }
        if input.peek(Token![^]) {
            let mut n = 0u8;
            while input.peek(Token![^]) {
                input.parse::<Token![^]>()?;
                n += 1;
            }
            return Ok(Self::Parent(n));
        }

        input.parse::<Token![$]>()?;
        Ok(Self::Global)
    }
}
// #[derive(Debug, Clone)]
// struct ExplicitGap(ScopeViewVariable);
// impl Parse for ExplicitGap {
//     #[instrument(name = "ExplicitGap")]
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         let svv = input.parse::<ScopeViewVariable>()?;
//         Ok(Self(svv))
//     }
// }
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

            if let Ok(explicit_gap) = input.parse::<ScopeViewVariable>() {
                debug!("got ScopeViewVariable: {:?}", &explicit_gap);

                input.parse::<Token![-]>()?;
                return Ok(Self::Eq(Gap::Var(explicit_gap)));
            }
            debug!("got just - ");
            return Ok(Self::Eq(Gap::Standard));
        }

        if input.peek(Token![~]) {
            debug!("peek ~ ");

            input.parse::<Token![~]>()?;

            //TODO check input.parse::<ScopeViewVariable>()  不会通吃
            if let Ok(explicit_gap) = input.parse::<ScopeViewVariable>() {
                input.parse::<Token![-]>()?;
                return Ok(Self::Le(Gap::Var(explicit_gap)));
            }

            if input.parse::<Token![-]>().is_ok() {
                input.parse::<Token![~]>()?;
                return Ok(Self::Le(Gap::Standard));
            }

            return Ok(Self::Le(Gap::None));
        }
        Ok(Self::Eq(Gap::None))
    }
}

/// ( `NameChars`[`Predicate`]? ) `[- ~]?` ...
#[derive(Debug, Clone)]
struct Splat {
    view_selector: ViewSelector,
    opt_connection: Option<Connection>,
}

impl Splat {
    fn opt_connection(&self) -> Option<&Connection> {
        self.opt_connection.as_ref()
    }
}

impl Parse for Splat {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _span =
            debug_span!("in Splat, parse-> ( `NameChars`[`Predicate`]? ) `[- ~]` ... ").entered();

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
    Or,
}

impl ViewProcessedScopeViewVariable {
    /// Returns `true` if the view processed name chars is [`Or`].
    ///
    /// [`Or`]: ViewProcessedNameChars::Or
    #[must_use]
    const fn is_or(&self) -> bool {
        matches!(self, Self::Or)
    }

    fn as_node(&self) -> Option<&ScopeViewVariable> {
        if let Self::Node(v) = self {
            Some(v)
        } else {
            None
        }
    }

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
#[display("{op} {var}")]
struct CCSSOpVar {
    op: PredOp,
    var: ScopeViewVariable,
}

impl CCSSOpVar {
    fn new(op: PredOp, var: ScopeViewVariable) -> Self {
        Self { op, var }
    }
}

#[derive(Debug, Clone)]
struct CCSSVarOpVarExpr {
    var: ScopeViewVariable,
    op_exprs: Vec<CCSSOpVar>,
}

impl std::fmt::Display for CCSSVarOpVarExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.var)?;
        for op in &self.op_exprs {
            write!(f, " {}", op)?;
        }
        Ok(())
    }
}

impl CCSSVarOpVarExpr {
    const fn new(var: ScopeViewVariable, op_exprs: Vec<CCSSOpVar>) -> Self {
        Self { var, op_exprs }
    }
    const fn new_var(var: ScopeViewVariable) -> Self {
        Self {
            var,
            op_exprs: vec![],
        }
    }
}

#[derive(Debug, Clone, Display)]
#[display("{eq} {expr}")]
struct CCSSEqExpression {
    eq: PredEq,
    expr: CCSSVarOpVarExpr,
}

impl CCSSEqExpression {
    const fn new(eq: PredEq, expr: CCSSVarOpVarExpr) -> Self {
        Self { eq, expr }
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
struct CCSS {
    var_op_vars: CCSSVarOpVarExpr,
    eq_exprs: Vec<CCSSEqExpression>,
    sw: Option<StrengthAndWeight>,
}

impl std::fmt::Display for CCSS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            var_op_vars,
            eq_exprs,
            sw,
        } = self;
        let sw = disp_opt(self.sw.as_ref());
        write!(f, "{} ", var_op_vars)?;
        for eqe in eq_exprs {
            write!(f, "{}", eqe)?;
        }
        write!(f, "{}", sw)
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

/// - ( `NameChars`[`Predicate`]? ) `[- ~]` ...
/// - ( `NameChars`[`Predicate`]? )
/// - `< Predicate >`
/// - |

#[derive(Clone)]
enum ViewObj {
    Splat(Splat),
    ViewSelector(ViewSelector),
    Point(Point),

    /// NOTE  "|"  
    Or,
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
                view: ViewProcessedScopeViewVariable::Or,
                is_splat: false,
                is_point: true,
                pos: Some(x.clone()),
                connection: None,
                pred: None,
            },
            Self::Or => ViewProcessed {
                view: ViewProcessedScopeViewVariable::Or,
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
            ViewObj::Splat(splat) => selectors.push(splat.view_selector.view.clone()),
            ViewObj::ViewSelector(vs) => selectors.push(vs.view.clone()),
            ViewObj::Point(_) | ViewObj::Or => (),
        }
    }
}

impl std::fmt::Debug for ViewObj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Splat(arg0) => f.debug_tuple("Splat").field(arg0).finish(),
            Self::ViewSelector(arg0) => f.debug_tuple("ViewSelector").field(arg0).finish(),
            Self::Point(arg0) => f.debug_tuple("Point").field(arg0).finish(),
            Self::Or => write!(f, "|"),
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
        Ok(Self::Or)
    }
}

/// `[- ~ ]? `
/// with........
/// - ( `NameChars` [`Predicate`]? ) `[- ~]` ...
/// - ( `NameChars` [`Predicate`]? )
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

/// `[== < > >= <=]? NameChars? [== < > >= <=]? StrengthAndWeight?`
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
            "PredEq/NameChars/StrengthAndWeight must has one"
        );
        Ok(Self {
            head_eq,
            value,
            tail_eq,
            s,
        })
    }
}

/// `([== < > >= <=]? NameChars? [== < > >= <=]? StrengthAndWeight? , ...)`
#[derive(Debug, Clone)]
struct ChainPredicate(Vec<ChainPredicateItem>);
impl Parse for ChainPredicate {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let paren_token = parenthesized!(content in input);
        let content: Punctuated<ChainPredicateItem, Token![,]> =
            content.parse_terminated(ChainPredicateItem::parse)?;

        Ok(Self(content.into_iter().collect()))
    }
}

/// `chain-xxx([== < > >= <=]? NameChars? [== < > >= <=]? StrengthAndWeight? ,  ...)`
#[derive(Debug, Clone)]
struct Chain {
    prop: Ident, //chain-[what]
    preds: ChainPredicate,
}
impl Parse for Chain {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw_opt::chain>()?;
        input.parse::<Token![-]>()?;
        let prop: Ident = input.parse()?;
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
            OptionItem::Chain(_) => "Chain".to_string(),
            OptionItem::In(_) => "In".to_string(),
            OptionItem::Gap(_) => "Gap".to_string(),
            OptionItem::OuterGap(_) => "OuterGap".to_string(),
            OptionItem::SW(_) => "SW".to_string(),
        }
    }

    fn as_outer_gap(&self) -> Option<&ScopeViewVariable> {
        if let Self::OuterGap(v) = self {
            Some(v)
        } else {
            None
        }
    }

    fn as_gap(&self) -> Option<&ScopeViewVariable> {
        if let Self::Gap(v) = self {
            Some(v)
        } else {
            None
        }
    }

    fn as_in(&self) -> Option<&ScopeViewVariable> {
        if let Self::In(v) = self {
            Some(v)
        } else {
            None
        }
    }

    fn as_sw(&self) -> Option<&StrengthAndWeight> {
        if let Self::SW(v) = self {
            Some(v)
        } else {
            None
        }
    }

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
    fn is_chain(&self) -> bool {
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
            let paren_token = parenthesized!(content in input);

            //TODO 原版使用空格, 这里使用 “,” ,测试是否有问题
            let content: Punctuated<ScopeViewVariable, Token![,]> =
                content.parse_terminated(ScopeViewVariable::parse)?;
            return Ok(Self::In(content.first().cloned().unwrap()));
        }
        if input.peek(kw_opt::gap) && input.peek2(token::Paren) {
            debug!("peek gap()");

            input.parse::<kw_opt::gap>()?;

            let content;
            let paren_token = parenthesized!(content in input);

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
            let paren_token = parenthesized!(content in input);

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
) -> (ScopeViewVariable, Vec<CCSSOpVar>) {
    if view_obj.is_point {
        let pos = view_obj.pos.clone().unwrap();
        // if (pos.0).0.len() > 1 {
        //     panic!("[get_left_var] can't support point expression is not single ScopeViewVariable current now");
        // }
        let PredExpression(v, op_s) = pos.0;
        (
            v,
            op_s.into_iter()
                .map(|(op, var)| CCSSOpVar::new(op, var))
                .collect(),
        )
    } else if view.is_or() {
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
) -> (ScopeViewVariable, Vec<CCSSOpVar>) {
    if view_obj.is_point {
        let pos = view_obj.pos.clone().unwrap();
        // if (pos.0).0.len() > 1 {
        //     panic!("[get_right_var] can't support point expression is not single ScopeViewVariable current now");
        // }

        let PredExpression(v, op_s) = pos.0;
        (
            v,
            op_s.into_iter()
                .map(|(op, var)| CCSSOpVar::new(op, var))
                .collect(),
        )
    } else if view.is_or() {
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
    if let Some(in_name) = o.map.get("In") {
        in_name.as_in().cloned().unwrap()
    } else {
        ScopeViewVariable::new_scope(Scope::Local)
    }
}
fn get_trailing_options(o: &Options) {
    if !o.map.is_empty() {}
}

#[derive(Debug)]
pub struct VFLStatement {
    d: Dimension,
    head: ViewObj,
    tails: Vec<ConnectionView>,
    o: Options,
    selectors: Vec<ScopeViewVariable>,
    ccsss: Vec<CCSS>,
}

impl VFLStatement {
    fn get_op_gap(&self, opt_gap: Option<&Gap>, with_container: bool) -> Option<CCSSOpVar> {
        let mut g: Option<CCSSOpVar>;
        if let Some(gap) = opt_gap {
            match gap {
                Gap::None => {
                    g = None;
                }
                Gap::Standard => {
                    if with_container && self.o.map.contains_key("OuterGap") {
                        g = Some(CCSSOpVar {
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
                        g = Some(CCSSOpVar {
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
                        g = Some(CCSSOpVar {
                            op: PredOp::new_add(),
                            var: ScopeViewVariable::new_var(standard_gap_names(&self.d)),
                        });
                    }
                }
                Gap::Var(x) => {
                    g = Some(CCSSOpVar {
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
    ) -> (Option<CCSSOpVar>, PredEq) {
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
            let var_op_vars = CCSSVarOpVarExpr {
                var: view_var.clone(),
                op_exprs: op.map_or(vec![], |op_one| vec![op_one]),
            };

            // @right ─────────────────────────────────────────────────────────────────

            let right = view_var.into_next(Some(right_var_names(&self.d)));
            let eq_exprs = vec![CCSSEqExpression {
                eq,
                expr: CCSSVarOpVarExpr::new_var(right),
            }];
            let ccss = CCSS {
                var_op_vars,
                eq_exprs,
                sw: None, //TODO check no sw?
            };
            self.ccsss.push(ccss);
        }
    }

    fn addPreds(&mut self, view: &ViewProcessedScopeViewVariable, opt_preds: Option<&Predicate>) {
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

                let mut op_exprs: Vec<CCSSOpVar> = vec![];
                for (op, view) in pred.pred_expression.1.clone() {
                    let var = if view.variable_is_none() {
                        view.clone().or_with_variable(size_var_names(&self.d))
                    } else {
                        view
                    };

                    op_exprs.push(CCSSOpVar::new(op, var));
                }

                let sw = pred.strength_and_weight.clone();
                let ccss = CCSS {
                    var_op_vars: CCSSVarOpVarExpr::new_var(node),
                    eq_exprs: vec![CCSSEqExpression::new(
                        eq,
                        CCSSVarOpVarExpr::new(right_var, op_exprs),
                    )],
                    sw,
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
                    let var_op_vars = CCSSVarOpVarExpr::new_var(first);
                    let mut eq_exprs = vec![];
                    let eq = pred.head_eq.unwrap_or_default();

                    for view in views_clone {
                        let var = view
                            .try_into_node()
                            .ok()
                            .unwrap()
                            .with_variable(chain_var.clone());
                        let right_view_var_op_vars = CCSSVarOpVarExpr::new_var(var);

                        if pred.value.is_some() {
                            let right_chain_var_op_vars =
                                CCSSVarOpVarExpr::new_var(pred.value.clone().unwrap());
                            eq_exprs.push(CCSSEqExpression::new(eq, right_chain_var_op_vars));

                            let tail_eq = pred.tail_eq.unwrap_or_else(|| eq.chain_tail_eq_map());
                            eq_exprs.push(CCSSEqExpression::new(tail_eq, right_view_var_op_vars));
                        } else {
                            eq_exprs.push(CCSSEqExpression::new(eq, right_view_var_op_vars));
                        }
                    }
                    let ccss = CCSS {
                        var_op_vars,
                        eq_exprs,
                        sw: pred.s.clone(),
                    };
                    self.ccsss.push(ccss);
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
        if !head_view.is_or() {
            chained_views.push_back(head_view.clone());
        }
        self.addPreds(&head_view, head_view_obj.pred.as_ref());

        let sw = self.o.map.get("SW").and_then(OptionItem::as_sw).cloned();
        debug!("tail {:#?}", self.tails);

        for tail in self.tails.clone() {
            debug!("in tail {:#?}", tail);
            let connection = tail.opt_connection.clone();
            let tail_view_obj = tail.view_obj.processe();
            self.add_splat_if_needed(&tail_view_obj);
            let tail_view = tail_view_obj.view.clone();
            if !tail_view.is_or() {
                chained_views.push_back(tail_view.clone());
            }
            self.addPreds(&tail_view, tail_view_obj.pred.as_ref());
            //result = [...]
            if !(head_view_obj.is_point && tail_view_obj.is_point) {
                debug!("不全部是 point",);
                //NOTE 不全部是 point
                let with_container = (head_view.is_or() || tail_view.is_or())
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
                let left_var_op_vars = CCSSVarOpVarExpr {
                    var: left_v,
                    op_exprs: left_point_op_var,
                };
                let right_var_op_vars = CCSSVarOpVarExpr {
                    var: right_v,
                    op_exprs: right_point_op_var,
                };
                let eq_exprs = vec![CCSSEqExpression::new(eq, right_var_op_vars)];
                debug!("======== sw: {:?}", &sw);

                let ccss = CCSS {
                    var_op_vars: left_var_op_vars,
                    eq_exprs,
                    sw: sw.clone(),
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
        // selectors: Vec<NameChars>,
        // ccsss: Vec<CCSS>,

        Ok(Self {
            d,
            head,
            tails,
            o,
            selectors,
            ccsss: vec![],
        })
    }
}

#[derive(Debug)]
pub enum Cassowary {
    Vfl(Box<VFLStatement>),
    CCss,
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

            Ok(Self::CCss)
        }
    }
}
struct CCSSSDisp(Vec<CCSS>);
impl std::fmt::Display for CCSSSDisp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CCSSS [")?;
        for ccss in &self.0 {
            writeln!(f, "{},", ccss)?;
        }
        writeln!(f, "]")
    }
}

#[cfg(test)]
mod tests {

    use std::path::Path;

    use quote::ToTokens;
    use tracing::debug;

    use crate::{
        cassowary::{CCSSSDisp, NameChars, VFLStatement},
        Gtree,
    };
    use tracing_subscriber::{prelude::*, registry::Registry};

    fn token_test(name: &str, input: &str) {
        // ────────────────────────────────────────────────────────────────────────────────

        let subscriber = Registry::default().with(tracing_tree::HierarchicalLayer::new(2));
        // .with(subscriber1);
        tracing::subscriber::set_global_default(subscriber).ok();

        // ─────────────────────────────────────────────────────────────────

        insta::with_settings!({snapshot_path => Path::new("./vfl_snap")}, {

            debug!("=========== parse \n {:?}\n",&input);

            match syn::parse_str::<VFLStatement>(input) {
                Ok(mut ok) => {
                    println!("=============\n{:#?}\n", &ok);
                    // insta::assert_debug_snapshot!(name.to_string()+"_prase", &ok);

                    ok.build();
                    println!("=================== build \n {:#?}\n", ok.ccsss);
                    // insta::assert_debug_snapshot!(name.to_string()+"_ccss", &ok.ccsss);
                    let disp = CCSSSDisp(ok.ccsss);
                    println!("=================== build---display \n {}\n", &disp);

                    insta::assert_display_snapshot!(name.to_string()+"_ccss_display", disp);

                    // let x = format!("{}", ok.to_token_stream());
                    // println!("===================\n {}\n", x);

                    // assert_eq!(x.as_str(), r#"NameChars :: Id (IdStr :: new ("button"))"#)
                }
                Err(error) => println!("...{:?}", error),
            }
        });
    }

    #[test]
    fn name_chars() {
        // ────────────────────────────────────────────────────────────────────────────────
        // ────────────────────────────────────────────────────────────────────────────────

        let subscriber = Registry::default().with(tracing_tree::HierarchicalLayer::new(2));
        // .with(subscriber1);
        tracing::subscriber::set_global_default(subscriber);

        // ─────────────────────────────────────────────────────────────────
        // ────────────────────────────────────────────────────────────────────────────────

        fn token_test(input: &str) {
            match syn::parse_str::<NameChars>(input) {
                Ok(ok) => {
                    let x = format!("{}", ok.to_token_stream());
                    println!("{}", x);
                    assert_eq!(x.as_str(), r#"NameChars :: Id (IdStr :: new ("button"))"#)
                }
                Err(error) => println!("...{:?}", error),
            }
        }

        println!();
        let input = r#" 
            #button
        "#;

        token_test(input);
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
    fn gap() {
        let input = r#" 
                @v (#b1)-(#b2)-(#b3)-(#b4)-(#b5) gap(20)
            "#;

        token_test("explicit-standard-gaps", input);
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
                Err(error) => println!("...{:?}", error),
            }
        }

        println!();
        let input = r#" 
        @=root
                Layer [
                    @=x111x @E=[{@h |(#button)...| }]
                    Layer [],
                    @=x111x @E=[{@h |(#button)...| in(#panel) gap(10) }]
                    Layer []

                ]
                
        "#;

        token_test(input);
    }
}
