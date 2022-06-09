/*
 * @Author: Rais
 * @Date: 2022-06-09 15:09:55
 * @LastEditTime: 2022-06-09 17:05:05
 * @LastEditors: Rais
 * @Description:
 */
static MIN_DICT_INDEX: usize = 100_000_000_usize;
#[derive(Debug, Eq)]
struct DictIndex<T>(usize, T);

impl<T> DictIndex<T> {
    pub fn new(i: usize, v: T) -> Self {
        Self(MIN_DICT_INDEX + i, v)
    }
}

impl<T: std::cmp::PartialEq> PartialEq for DictIndex<T> {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}
impl<T: std::cmp::PartialEq + Ord> PartialOrd for DictIndex<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.0.cmp(&other.0) {
            std::cmp::Ordering::Equal => {
                // Some(self.1.cmp(&other.1))
                panic!("DictIndex never meet Equal")
            }
            // std::cmp::Ordering::Less => todo!(),
            // std::cmp::Ordering::Greater => todo!(),
            ord => Some(ord),
        }
    }
}
#[cfg(test)]
mod dict_index_test {

    use wasm_bindgen_test::wasm_bindgen_test;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    use crate::g_node::index::DictIndex;
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
    fn dict_index_test_panic() {
        let a = DictIndex::new(0, 1);
        let b = DictIndex::new(0, 2);
        assert!(a < b);
    }
}
