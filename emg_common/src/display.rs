/*
 * @Author: Rais
 * @Date: 2023-01-24 22:46:22
 * @LastEditTime: 2023-01-28 20:58:50
 * @LastEditors: Rais
 * @Description:
 */
use im::{HashMap, OrdMap};
use indented::{indented, indented_with};
use std::fmt::{format, Write};
// pub struct MapDisplay<'a, K, V>(pub &'a str, pub OrdMap<K, V>);
// impl<'a, K, V> std::fmt::Display for MapDisplay<'a, K, V>
// where
//     K: std::fmt::Display + Ord,
//     V: std::fmt::Display,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         // let sv: String = self
//         //     .0
//         //     .iter()
//         //     .map(|(k, v)| format!("{} :\n{}\n,\n", k, indented(v)))
//         //     .fold(String::default(), |acc, v| format!("{acc}{v}"));

//         let mut members = String::new();
//         self.1.iter().for_each(|(k, v)| {
//             writeln!(members, "{} :\n{}\n,", k, indented(v)).unwrap();
//         });

//         write!(f, "{} {{\n{}\n}}", self.0, indented(&members))
//     }
// }
pub struct DictDisplay<'a, K, V>(pub &'a str, pub OrdMap<K, V>);

impl<'a, K, V> std::fmt::Display for DictDisplay<'a, K, V>
where
    K: std::fmt::Display + Ord,
    V: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let sv: String = self
        //     .0
        //     .iter()
        //     .map(|(k, v)| format!("{} :\n{}\n,\n", k, indented(v)))
        //     .fold(String::default(), |acc, v| format!("{acc}{v}"));

        let mut members = String::new();
        self.1.iter().for_each(|(k, v)| {
            writeln!(members, "{k} : {v}").unwrap();
        });

        write!(
            f,
            "{} {{\n{}}}",
            self.0,
            // indented_with(&members, " ".repeat(self.0.len() + 2).as_str())
            indented(&members)
        )
    }
}

pub struct HashMapDisplay<'a, K, V, S>(pub &'a str, pub HashMap<K, V, S>)
where
    K: std::clone::Clone + std::hash::Hash + std::cmp::Eq + std::default::Default;

impl<'a, K, V, S> std::fmt::Display for HashMapDisplay<'a, K, V, S>
where
    K: std::fmt::Display + Ord,
    V: std::fmt::Display,
    K: std::clone::Clone + std::hash::Hash + std::cmp::Eq + std::default::Default,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let sv: String = self
        //     .1
        //     .iter()
        //     .map(|(k, v)| format!("{} :\n{}\n,\n", k, indented(v)))
        //     .fold(String::default(), |acc, v| format!("{acc}{v}"));

        let mut members = String::new();
        self.1.iter().for_each(|(k, v)| {
            writeln!(members, "{k} : {v} ,").unwrap();
        });

        write!(
            f,
            "{} {{\n{}}}",
            self.0,
            indented_with(&members, " ".repeat(self.0.len() + 1).as_str())
        )
    }
}
