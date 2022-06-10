use std::{borrow::Borrow, fmt::Display};

/*
 * @Author: Rais
 * @Date: 2022-06-09 15:09:55
 * @LastEditTime: 2022-06-10 10:24:19
 * @LastEditors: Rais
 * @Description:
 */
use std::fmt::Debug;

use emg_core::im::ordmap::DiffIter;
use emg_state::Dict;
use tracing::warn;
static MIN_DICT_INDEX: usize = 100_000_000_usize;

#[derive(Debug, Clone, Eq)]
struct DictIndex<T>(usize, T);

impl<T> DictIndex<T> {
    pub fn new(i: usize, key: T) -> Self {
        Self(MIN_DICT_INDEX + i, key)
    }
}
// impl<T> Borrow<Key<T>> for DictIndex<T> {
//     fn borrow(&self) -> &Key<T> {
//         &self.1
//     }
// }
impl<T: Debug> Display for DictIndex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("⇣(i:{},k:{:?})", self.0, self.1))
    }
}

impl<T: std::cmp::PartialEq> PartialEq for DictIndex<T> {
    fn eq(&self, other: &Self) -> bool {
        warn!("PartialEq");
        self.1 == other.1
    }
}
// impl<T: std::cmp::PartialEq> PartialEq<T> for DictIndex<T> {
//     fn eq(&self, other: &T) -> bool {
//         self.1 == *other
//     }
// }
// impl<T: std::cmp::PartialEq> PartialEq<&T> for DictIndex<T> {
//     fn eq(&self, other: &&T) -> bool {
//         self.1 == **other
//     }
// }
impl<T: std::cmp::PartialEq + PartialOrd> PartialOrd for DictIndex<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.0.cmp(&other.0) {
            std::cmp::Ordering::Equal => {
                warn!("PartialOrd eq");
                self.1.partial_cmp(&other.1)
                //Some(std::cmp::Ordering::Equal)
                // panic!("DictIndex never meet Equal")
            }
            // std::cmp::Ordering::Less => todo!(),
            // std::cmp::Ordering::Greater => todo!(),
            ord => Some(ord),
        }
    }
}
impl<T: PartialEq + Ord + Debug> Ord for DictIndex<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // TODO self.1 same ,return Eq, should override old one.
        match self.1.cmp(&other.1) {
            std::cmp::Ordering::Equal => std::cmp::Ordering::Equal,
            _ => {
                match self.0.cmp(&other.0) {
                    std::cmp::Ordering::Equal => {
                        warn!("Ord eq");

                        // std::cmp::Ordering::Equal
                        self.1.cmp(&other.1)
                        // panic!("DictIndex never meet index Equal {} {}", self, other)
                    }
                    // std::cmp::Ordering::Less => todo!(),
                    // std::cmp::Ordering::Greater => todo!(),
                    ord => ord,
                }
            }
        }
    }
}

#[derive(Clone)]
struct DDiff<K, V>(Dict<K, V>)
where
    K: Ord + Clone + PartialEq,
    V: Clone + PartialEq,
    (K, V): PartialEq;

impl<K, V> DDiff<K, V>
where
    K: Ord + Clone + PartialEq + Debug,
    V: Clone + PartialEq + Debug,
    (K, V): PartialEq,
{
    pub fn diff<'a>(&'a self, other: &'a Self) {
        // self.0.diff(&other.0)
        for item in self.0.diff(&other.0) {
            warn!("Diff=========================================");
            match item {
                emg_core::im::ordmap::DiffItem::Add(k, v) => warn!("Add:  {:?} {:?}", k, v),
                emg_core::im::ordmap::DiffItem::Update { old, new } => {
                    warn!("Update: {:?} {:?}", old, new);
                }
                emg_core::im::ordmap::DiffItem::Remove(k, v) => warn!("Remove: {:?} {:?}", k, v),
            }
        }
    }
}

#[cfg(test)]
mod dict_index_test {
    use emg_core::im::ordmap::DiffItem;
    use std::borrow::Borrow;

    use emg_state::Dict;
    use tracing::warn;
    use wasm_bindgen_test::wasm_bindgen_test;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    use crate::g_node::index::{DDiff, DictIndex};
    #[wasm_bindgen_test]
    fn dict_index_test1() {
        let a = DictIndex::new(0, 1);
        let b = DictIndex::new(0, 2);
        assert_ne!(a, b);
    }
    #[wasm_bindgen_test]
    fn dict_index_test_eq() {
        let a = DictIndex::new(1, 1);
        let b = DictIndex::new(0, 1);
        assert_eq!(a, b);
    }
    #[wasm_bindgen_test]
    fn dict_index_test_eq3() {
        let a = DictIndex::new(0, 1);
        let b = DictIndex::new(0, 1);
        assert_eq!(a, b);
    }
    #[wasm_bindgen_test]
    fn dict_index_test_eq4() {
        let a = DictIndex::new(0, 1);
        let b = DictIndex::new(0, 1);
        assert!(a == b);
    }
    // #[wasm_bindgen_test]
    // fn dict_index_test_eq5() {
    //     let a = DictIndex::new(0, 11);
    //     let b = DictIndex::new(0, 12);
    //     assert!(a == Key(11));
    //     assert!(b == Key(12));
    // }
    #[wasm_bindgen_test]
    fn dict_index_test_eq2() {
        let a = DictIndex::new(1, 1);
        let b = DictIndex::new(0, 1);
        assert!(a == b);
    }
    #[wasm_bindgen_test]
    fn dict_index_test_2() {
        let a = DictIndex::new(0, 2);
        let b = DictIndex::new(1, 1);
        assert!(a < b);
        assert!(!(a > b));
        assert!(!(a == b));
        assert!(!(b == a));
        assert!(b > a);
    }
    #[wasm_bindgen_test]
    fn dict_index_test_dict() {
        // ─────────────────────────────────────────────────────────────────

        console_error_panic_hook::set_once();
        // ─────────────────────────────────────────────────────────────────
        let mut config = tracing_wasm::WASMLayerConfigBuilder::default();
        config.set_max_level(tracing::Level::DEBUG);
        config.set_max_level(tracing::Level::INFO);
        config.set_console_config(tracing_wasm::ConsoleConfig::ReportWithConsoleColor);
        // config.set_console_config(tracing_wasm::ConsoleConfig::NoReporting);

        tracing_wasm::set_as_global_default_with_config(config.build());

        // ────────────────────────────────────────────────────────────────────────────────

        let a = DictIndex::new(0, "ff".to_string());
        let b = DictIndex::new(3, "xx".to_string());
        let c = DictIndex::new(1, "xxx".to_string());
        let cc = DictIndex::new(1, "xxxx".to_string());

        let mut d = Dict::new();
        // d.insert(cc.clone(), "cc".to_string());

        d.insert(b.clone(), "b".to_string());
        d.insert(a.clone(), "a".to_string());
        d.insert(c.clone(), "c".to_string());

        d.iter().for_each(|(k, v)| {
            warn!("k:{:?},v:{:?}", k, v);
        });
        let b_got = d.get(&DictIndex::new(3, "xx".to_string()));
        warn!("got b: {:?}", &b_got);
        assert_eq!(b_got.unwrap(), &"b".to_string());
        // let c_got = d.get(&DictIndex::new(1, "xxx".to_string()));
        // warn!("got c: {:?}", &c_got);
        // assert_eq!(c_got.unwrap(), &"c".to_string());

        let a_got = d.get(&DictIndex::new(0, "ff".to_string()));
        warn!("got a: {:?}", &a_got);
        assert_eq!(a_got.unwrap(), &"a".to_string());

        let cc_got = d.get(&cc);
        warn!("got c: {:?}", &cc_got);
        assert_eq!(cc_got.unwrap(), &"cc".to_string());

        warn!("====================");

        let mut d2 = Dict::new();
        d2.insert(6, "b".to_string());
        d2.insert(9, "a".to_string());
        d2.insert(4, "c".to_string());

        d2.iter().for_each(|(k, v)| {
            warn!("k:{},v:{}", k, v);
        });

        assert!(a < b);
        assert!(!(a > b));
        assert!(!(a == b));
        assert!(!(b == a));
        assert!(b > a);

        // ─────────────────────────────────────────────────────────────────

        // let a = DictIndex::new(0, "ff".to_string());
        // let b = DictIndex::new(3, "xx".to_string());
        // let c = DictIndex::new(1, "xxx".to_string());

        let a2 = DictIndex::new(0, "fff".to_string());
        let b2 = DictIndex::new(4, "xx".to_string());
        let c2 = DictIndex::new(1, "xxx".to_string());

        let mut d3 = Dict::new();
        d3.insert(b2.clone(), "b".to_string());
        d3.insert(a2.clone(), "a".to_string());
        d3.insert(c2.clone(), "c".to_string());
        // let fff = &*DD(d);
        let dd = DDiff(d);
        let dd3 = DDiff(d3);
        dd.diff(&dd3);
    }

    #[wasm_bindgen_test]
    fn dict_index_test_panic() {
        let a = DictIndex::new(0, 1);
        let b = DictIndex::new(0, 2);
        assert!((a < b));
    }
}
