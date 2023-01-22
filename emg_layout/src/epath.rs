use std::{hash::Hash, str::FromStr};

use emg::{edge_index, edge_index_no_source, EdgeIndex, NodeIndex};
use emg_common::Vector;
use nom::{
    bytes::complete::{tag, take_while},
    error::Error,
    Finish, IResult,
};

use crate::parser::parse_edge_ix_s;
/*
 * @Author: Rais
 * @Date: 2023-01-22 14:02:47
 * @LastEditTime: 2023-01-22 16:44:29
 * @LastEditors: Rais
 * @Description:
 */
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Default)]
// pub struct EPath<Ix: Clone + Hash + Eq + PartialEq + Default>(TinyVec<[EdgeIndex<Ix>;2]>);
//TODO  loop check
pub struct EPath<Ix: Clone + Hash + Eq + PartialEq + Default>(pub Vector<EdgeIndex<Ix>>);

#[macro_export]
macro_rules! epath {



    (@end $($e:expr),+ ; @source $s:expr; $t:expr ) => {
        // println!("{}-{}|end",$s,$t);
        $crate::EPath::new($crate::emg_common::im::vector![
            $($e),+,$crate::EdgeIndex::new($crate::node_index($s), $crate::node_index($t))
        ])

    };

    (@end $($e:expr),+ ; @source $s:expr; $t:expr => $($y:expr)=>+) => {


        epath![@end $($e),+,$crate::EdgeIndex::new($crate::node_index($s), $crate::node_index($t)) ; @source $t; $($y)=>+]
    };

    ( $x:expr => $($y:expr)=>+) => {
            // println!("start-{}",$x);

        epath![@end $crate::EdgeIndex::new(None, $crate::node_index($x)) ; @source $x; $($y)=>+]
    };
    ( $root:expr ) => {
        $crate::EPath::new($crate::emg_common::im::vector![
            $crate::EdgeIndex::new(None, $crate::node_index($root))
        ])
    };

}

impl<Ix: Clone + Hash + Eq + PartialEq + Default> std::ops::Deref for EPath<Ix> {
    type Target = Vector<EdgeIndex<Ix>>;
    // type Target = TinyVec<[EdgeIndex<Ix>;2]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<Ix: Clone + Hash + Eq + PartialEq + Default> std::ops::DerefMut for EPath<Ix> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<Ix: Clone + Hash + Eq + PartialEq + Default> EPath<Ix> {
    #[must_use]
    pub const fn new(vec: Vector<EdgeIndex<Ix>>) -> Self {
        Self(vec)
    }

    #[must_use]
    pub fn last_target(&self) -> Option<&NodeIndex<Ix>> {
        self.0.last().and_then(|e| e.target_nix().as_ref())
    }
    #[must_use]
    ///除了 `other_added_tail` 的最后一个 nix, 其他全部匹配
    pub fn except_tail_match(&self, other_added_tail: &Self) -> bool {
        if self.0.len() - 1 != other_added_tail.0.len() {
            return false;
        }
        for i in 0..self.0.len() - 1 {
            if self.0[i] != other_added_tail.0[i] {
                return false;
            }
        }
        true
    }

    #[must_use]
    pub fn link_ref(&self, target_nix: NodeIndex<Ix>) -> Self {
        let last = self.last().and_then(|e| e.target_nix().as_ref()).cloned();
        let mut new_e = self.clone();
        new_e.push_back(EdgeIndex::new(last, target_nix));
        new_e
    }
    #[must_use]
    pub fn link(mut self, target_nix: NodeIndex<Ix>) -> Self {
        let last = self.last().and_then(|e| e.target_nix().as_ref()).cloned();
        self.push_back(EdgeIndex::new(last, target_nix));
        self
    }
}

impl<Ix> std::fmt::Display for EPath<Ix>
where
    Ix: Clone + Hash + Eq + PartialEq + Default + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sv: String = self
            .0
            .iter()
            //TODO  textwrap
            .map(|v| format!("{v}"))
            .intersperse(String::from(","))
            .fold(String::default(), |acc, v| format!("{acc}{v}"));

        write!(f, "path [{}]", &sv)
    }
}

impl<Ix> FromStr for EPath<Ix>
where
    Ix: std::clone::Clone
        + std::hash::Hash
        + std::cmp::Eq
        + std::default::Default
        + for<'a> std::convert::From<&'a str>,
{
    type Err = Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_edge_ix_s(s).finish() {
            Ok((_remaining, names)) => {
                //NOTE never empty because take_while1( take_while1 will error before)
                // if names.is_empty() {
                //     return Err(Error::new(
                //         ErrorKind::InvalidData,
                //         format!("invalid path: {}", s),
                //     ));
                // }
                let (_, eix_s) = names.into_iter().fold(
                    (None, Vector::default()),
                    |(mut opt_s, mut eix_s), ix| {
                        match opt_s {
                            None => {
                                eix_s.push_back(edge_index_no_source(ix));
                                opt_s = Some(ix);
                            }
                            Some(s) => {
                                eix_s.push_back(edge_index(s, ix));
                                opt_s = Some(ix);
                            }
                        };
                        (opt_s, eix_s)
                    },
                );
                Ok(Self::new(eix_s))
            }
            Err(Error { input, code }) => Err(Error {
                input: input.to_string(),
                code,
            }),
        }
    }
}

#[cfg(test)]
mod test_epath {
    use emg::{edge_index, edge_index_no_source};
    use emg_common::{im::vector, IdStr};

    use crate::EPath;

    #[test]
    fn test_macro() {
        let a: EPath<IdStr> = epath!["a"=>"b"=>"c"=>"d"=>"e"];
        println!("{}", a);
        assert_eq!(
            a,
            EPath(vector![
                edge_index_no_source("a"),
                edge_index("a", "b"),
                edge_index("b", "c"),
                edge_index("c", "d"),
                edge_index("d", "e"),
            ])
        );
        let a: EPath<IdStr> = epath!["a"=>"b"=>"c"=>"d"];
        println!("{}", a);
        assert_eq!(
            a,
            EPath(vector![
                edge_index_no_source("a"),
                edge_index("a", "b"),
                edge_index("b", "c"),
                edge_index("c", "d"),
            ])
        );
        let a: EPath<IdStr> = epath!["a"=>"b"=>"c"];
        println!("{}", a);
        assert_eq!(
            a,
            EPath(vector![
                edge_index_no_source("a"),
                edge_index("a", "b"),
                edge_index("b", "c"),
                // edge_index("c", "d"),
                // edge_index("d", "e"),
            ])
        );
        let a: EPath<IdStr> = epath![" a x "=>"b"];
        println!("{}", a);
        assert_eq!(
            a,
            EPath(vector![
                edge_index_no_source(" a x "),
                edge_index(" a x ", "b"),
                // edge_index("b", "c"),
                // edge_index("c", "d"),
                // edge_index("d", "e"),
            ])
        );
        let a: EPath<IdStr> = epath!["a"];
        println!("{}", a);
        assert_eq!(
            a,
            EPath(vector![
                edge_index_no_source("a"),
                // edge_index("a", "b"),
                // edge_index("b", "c"),
                // edge_index("c", "d"),
                // edge_index("d", "e"),
            ])
        );
    }

    #[test]
    fn test_parser() {
        let s = "a=>b=>c=>D=>e";
        let f: EPath<IdStr> = s.parse().unwrap();
        println!("{}", f);
        assert_eq!(f, epath!["a"=>"b"=>"c"=>"D"=>"e"]);
        assert_eq!(
            f,
            EPath(vector![
                edge_index_no_source("a"),
                edge_index("a", "b"),
                edge_index("b", "c"),
                edge_index("c", "D"),
                edge_index("D", "e"),
            ])
        );
    }
    #[test]
    fn test_parser_has_space() {
        let s = "中 文 =>b=>c=>D=>e";
        let f: EPath<IdStr> = s.parse().unwrap();
        println!("{}", f);
        assert_eq!(f, epath!["中 文" => "b"=>"c"=>"D"=>"e"]);
        assert_eq!(
            f,
            EPath(vector![
                edge_index_no_source("中 文"),
                edge_index("中 文", "b"),
                edge_index("b", "c"),
                edge_index("c", "D"),
                edge_index("D", "e"),
            ])
        );
    }
}
