/*
 * @Author: Rais
 * @Date: 2023-01-22 14:02:47
 * @LastEditTime: 2023-04-08 00:13:56
 * @LastEditors: Rais
 * @Description:
 */

use emg::{edge_index, edge_index_no_source, EdgeIndex, NodeIndex};
use emg_common::Vector;
use nom::{error::Error, Finish};
use std::{fmt::Write, panic::Location};
use std::{hash::Hash, str::FromStr};
use tracing::error;

use crate::parser::parse_edge_ix_s;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Default)]
//TODO  loop check
pub struct EPath(pub(crate) Vector<EdgeIndex>);
// : Clone + Hash + Eq + PartialEq + Default

#[macro_export]
macro_rules! epath {




    (@end $($e:expr),+ ; @source $s:literal; $t:literal => $($y:expr)=>+) => {


        epath![@end $($e),+,$crate::EdgeIndex::new($crate::node_index($s), $crate::node_index($t)) ; @source $t; $($y)=>+]
    };

    (@end $($e:expr),+ ; @source $s:literal; $t:literal ) => {
        // println!("{}-{}|end",$s,$t);
        $crate::EPath::new($crate::emg_common::im::vector![
            $($e),+,$crate::EdgeIndex::new($crate::node_index($s), $crate::node_index($t))
        ])

    };

    ($x:literal => $($y:expr)=>+) => {
            // println!("start-{}",$x);

        epath![@end $crate::EdgeIndex::new(None, $crate::node_index($x)) ; @source $x; $($y)=>+]
    };



    ($root:literal ) => {
        $crate::EPath::new($crate::emg_common::im::vector![
            $crate::EdgeIndex::new(None, $crate::node_index($root))
        ])
    };
    // ─────────────────────────────────────────────────────────────────────
    // with ident/expr now
    // ─────────────────────────────────────────────────────────────────────────────


    (@end $($e:expr),+ ; @source $s:literal; $t:expr => $($y:expr)=>+) => {


        epath![@end $($e),+,$crate::EdgeIndex::new($crate::node_index($s), Some($t .clone())) ; @source $t; $($y)=>+]
    };
    (@end $($e:expr),+ ; @source $s:expr; $t:literal => $($y:expr)=>+) => {


        epath![@end $($e),+,$crate::EdgeIndex::new(Some($s .clone()), $crate::node_index($t)) ; @source $t; $($y)=>+]
    };
    (@end $($e:expr),+ ; @source $s:expr; $t:expr => $($y:expr)=>+) => {


        epath![@end $($e),+,$crate::EdgeIndex::new(Some($s .clone()), Some($t .clone())) ; @source $t; $($y)=>+]
    };
    // ─────────────────────────────────────────────────────────────────────

    (@end $($e:expr),+ ; @source $s:literal; $t:expr ) => {
        // println!("{}-{}|end",$s,$t);
        $crate::EPath::new($crate::emg_common::im::vector![
            $($e),+,$crate::EdgeIndex::new($crate::node_index($s), Some($t .clone()))
        ])

    };
    (@end $($e:expr),+ ; @source $s:expr; $t:literal ) => {
        // println!("{}-{}|end",$s,$t);
        $crate::EPath::new($crate::emg_common::im::vector![
            $($e),+,$crate::EdgeIndex::new(Some($s .clone()), $crate::node_index($t))
        ])

    };
    (@end $($e:expr),+ ; @source $s:expr; $t:expr ) => {
        // println!("{}-{}|end",$s,$t);
        $crate::EPath::new($crate::emg_common::im::vector![
            $($e),+,$crate::EdgeIndex::new(Some($s .clone()), Some($t .clone()))
        ])

    };
    // ─────────────────────────────────────────────────────────────────────




    ($x:expr => $($y:expr)=>+) => {
            // println!("start-{}",$x);

        epath![@end $crate::EdgeIndex::new(None, Some($x .clone())) ; @source $x; $($y)=>+]
    };

    ($root:expr ) => {
        $crate::EPath::new($crate::emg_common::im::vector![
            $crate::EdgeIndex::new(None, $root .clone())
        ])
    };

    // ─────────────────────────────────────────────────────────────────────
// ─────────────────────────────────────────────────────────────────────────────



    ($($rest:tt)*) =>{
        $crate::epath![<_>$($rest)*]
    };


}

impl std::ops::Deref for EPath {
    type Target = Vector<EdgeIndex>;
    // type Target = TinyVec<[EdgeIndex;2]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for EPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl EPath {
    /// # Panics
    ///
    /// Will panic if 'vec' is empty, or if the first element's `source_nix` is not None.
    #[track_caller]
    #[must_use]
    pub fn new(vec: Vector<EdgeIndex>) -> Self {
        // assert!(vec.front().unwrap().source_nix().is_none());
        #[cfg(debug_assertions)]
        {
            if vec
                .front()
                .expect("EPath must has one EdgeIndex")
                .source_nix()
                .is_some()
            {
                let loc = Location::caller();
                error!(
                    "vec.front().expect().source_nix().is_none() is not none :{:?}\n{}",
                    vec, loc
                );
            }
        }
        assert!(!vec.is_empty());
        Self(vec)
    }

    #[track_caller]
    #[must_use]
    pub const fn new_uncheck(vec: Vector<EdgeIndex>) -> Self {
        Self(vec)
    }

    #[must_use]
    pub fn last_target(&self) -> Option<&NodeIndex> {
        self.0.last().and_then(emg::EdgeIndex::target_nix)
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
    pub fn link_ref(&self, target_nix: NodeIndex) -> Self {
        let last = self.last().and_then(emg::EdgeIndex::target_nix).cloned();
        let mut new_e = self.clone();
        new_e.push_back(EdgeIndex::new(last, target_nix));
        new_e
    }
    #[must_use]
    pub fn link(mut self, target_nix: NodeIndex) -> Self {
        let last = self.last().and_then(emg::EdgeIndex::target_nix).cloned();
        self.push_back(EdgeIndex::new(last, target_nix));
        self
    }
}

impl std::fmt::Display for EPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let sv: String = self
        //     .0
        //     .iter()
        //     //TODO  textwrap
        //     .map(|v| format!("{v}"))
        //     .intersperse(String::from(","))
        //     .fold(String::default(), |acc, v| format!("{acc}{v}"));
        // write!(f, "ep-[{}]", &sv)

        let mut path = String::new();
        let front = self.0.front();
        if let Some(e) = front {
            let first_target = e.target_nix().unwrap();
            write!(path, "⚬-{first_target}")?;
        } else {
            write!(path, "EMPTY!!!!!!!!!!!")?;
        }

        for e in self.0.iter().skip(1) {
            if let Some(t) = e.target_nix() {
                write!(path, " => {t}")?;
            }
        }
        write!(f, "[{}]", &path)
    }
}

impl FromStr for EPath {
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
    use emg::{edge_index, edge_index_no_source, node_index, NodeIndex};
    use emg_common::im::vector;

    use crate::EPath;

    #[test]
    #[allow(clippy::many_single_char_names)]
    fn test_macro_var() {
        let a: NodeIndex = node_index("a");
        let b: NodeIndex = node_index("b");
        let c: NodeIndex = node_index("c");
        let d: NodeIndex = node_index("d");
        let e: NodeIndex = node_index("e");

        // let xx = EPath::new({
        //     let mut l = vector::Vector::new();
        //     l.push_back((EdgeIndex::new(None, Some(a))));
        //     l.push_back((EdgeIndex::new(Some(a), Some(b))));
        //     l.push_back((EdgeIndex::new(Some(b), Some(c))));
        //     l.push_back((EdgeIndex::new(Some(c), Some(d))));
        //     l.push_back((EdgeIndex::new(Some(d), Some(e))));
        //     l
        // });

        let ep: EPath = epath![a=>b=>c=>d=>e];
        println!("{ep}");
        assert_eq!(
            ep,
            EPath(vector![
                edge_index_no_source("a"),
                edge_index("a", "b"),
                edge_index("b", "c"),
                edge_index("c", "d"),
                edge_index("d", "e"),
            ])
        );
        let ep: EPath = epath![a =>"b"=>c=>"d"];
        println!("{ep}");
        assert_eq!(
            ep,
            EPath(vector![
                edge_index_no_source("a"),
                edge_index("a", "b"),
                edge_index("b", "c"),
                edge_index("c", "d"),
            ])
        );
        let ep: EPath = epath!["a" =>b=>"c"];
        println!("{ep}");
        assert_eq!(
            ep,
            EPath(vector![
                edge_index_no_source("a"),
                edge_index("a", "b"),
                edge_index("b", "c"),
                // edge_index("c", "d"),
                // edge_index("d", "e"),
            ])
        );
        let ep: EPath = epath![" a x "=>"b"];
        println!("{ep}");
        assert_eq!(
            ep,
            EPath(vector![
                edge_index_no_source(" a x "),
                edge_index(" a x ", "b"),
                // edge_index("b", "c"),
                // edge_index("c", "d"),
                // edge_index("d", "e"),
            ])
        );
        let ep: EPath = epath!["a"];
        println!("{ep}");
        assert_eq!(
            ep,
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
    fn test_macro_literal() {
        let a = epath![ "a"=>"b"=>"c"=>"d"=>"e"];
        println!("{a}");
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
        let a: EPath = epath!["a"=>"b"=>"c"=>"d"];
        println!("{a}");
        assert_eq!(
            a,
            EPath(vector![
                edge_index_no_source("a"),
                edge_index("a", "b"),
                edge_index("b", "c"),
                edge_index("c", "d"),
            ])
        );
        let a: EPath = epath!["a"=>"b"=>"c"];
        println!("{a}");
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
        let a: EPath = epath![" a x "=>"b"];
        println!("{a}");
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
        let a: EPath = epath!["a"];
        println!("{a}");
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
        let f: EPath = s.parse().unwrap();
        println!("{f}");
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
        let f: EPath = s.parse().unwrap();
        println!("{f}");
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
