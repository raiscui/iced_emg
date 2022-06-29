use cassowary::Variable;
use emg_core::{im::HashMap, IdStr, NotNan};
use slotmap::SlotMap;

/*
 * @Author: Rais
 * @Date: 2022-06-23 22:52:57
 * @LastEditTime: 2022-06-29 16:32:05
 * @LastEditors: Rais
 * @Description:
 */

#[derive(Debug, Clone)]
enum NameChars {
    Id(IdStr),
    Class(IdStr),
    Element(IdStr),
    Virtual(IdStr),
    Number(NotNan<f64>),
}
enum PredEq {
    Eq,
    Lt,
    Le,
    Ge,
    Gt,
}
struct ViewSelector {
    view: NameChars,
    pred: Option<Predicate>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CassowaryMap {
    map: HashMap<IdStr, Variable>,
    v_k: HashMap<Variable, IdStr>,
    // SlotMap<Variable>,
}

impl CassowaryMap {
    fn var(&self, key: &IdStr) -> Option<Variable> {
        self.map.get(key).copied()
    }
}
