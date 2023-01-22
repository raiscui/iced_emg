/*
 * @Author: Rais
 * @Date: 2023-01-22 13:59:45
 * @LastEditTime: 2023-01-22 16:06:42
 * @LastEditors: Rais
 * @Description:
 */
/*
 * @Author: Rais
 * @Date: 2023-01-22 11:08:55
 * @LastEditTime: 2023-01-22 13:59:38
 * @LastEditors: Rais
 * @Description:
 */

use nom::{
    bytes::complete::{tag, take_while, take_while1},
    combinator::{map_opt, map_res},
    error::{Error, ErrorKind, ParseError},
    multi::separated_list1,
    sequence::delimited,
    IResult,
};

// pub type IResult<I, O, E = (I, ErrorKind)> = Result<(I, O), Err<E>>;

// #[derive(Debug, PartialEq, Eq, Clone, Copy)]
// pub enum Needed {
//     Unknown,
//     Size(u32),
// }

// #[derive(Debug, Clone, PartialEq)]
// pub enum Err<E> {
//     Incomplete(Needed),
//     Error(E),
//     Failure(E),
// }
const fn is_space(chr: char) -> bool {
    chr == ' '
}
///ix allow [a-z,A-Z,各国文字,_,-,空格]
fn is_ix_allow_char(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '-' || is_space(c)
}

//fn edge_link , parse result "=>" in the input:"axx=>xxxxx"
fn parse_edge_link(input: &str) -> IResult<&str, &str> {
    tag("=>")(input)
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
///
// fn drop_front_tail_space<'a, F, O, E: ParseError<&'a str>>(
//     inner: F,
// ) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
// where
//     F: FnMut(&'a str) -> IResult<&'a str, O, E>,
// {
//     // delimited(multispace0, inner, multispace0)
//     delimited(take_while(is_space), inner, take_while(is_space))
// }
fn drop_front_tail_space<'a, F, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, &'a str, E>,
{
    // delimited(multispace0, inner, multispace0)
    map_opt(inner, |o| Some(o.trim()))
}
fn parse_edge_ix_mayhas_space(input: &str) -> IResult<&str, &str> {
    // delimited(multispace0, inner, multispace0)
    take_while1(is_ix_allow_char)(input)
}

pub fn parse_edge_ix_s(input: &str) -> IResult<&str, Vec<&str>> {
    // parse result "axx" in the input:"ax中文x=>xxxxx"
    separated_list1(
        parse_edge_link,
        drop_front_tail_space(parse_edge_ix_mayhas_space),
    )(input)
}

#[cfg(test)]
mod parser_test {

    use nom::error::ErrorKind;

    use super::*;

    #[test]
    fn test() {
        assert_eq!(parse_edge_ix_s("a=>b=>c"), Ok(("", vec!["a", "b", "c"])));
        assert_eq!(
            parse_edge_ix_s("中文_a=>b=>c"),
            Ok(("", vec!["中文_a", "b", "c"]))
        );
        assert_eq!(
            parse_edge_ix_s("안녕 잘 지내=>b=>c"),
            Ok(("", vec!["안녕 잘 지내", "b", "c"]))
        );
        assert_eq!(
            parse_edge_ix_s("   안녕 잘 지내  =>b=>c"),
            Ok(("", vec!["안녕 잘 지내", "b", "c"]))
        );
    }
}
