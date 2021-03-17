#[cfg(test)]
mod test {
    use gtree_proc_macro::emg;

    use wasm_bindgen_test::*;

    #[derive(Debug, Copy, Clone)]
enum Message {
    IncrementPressed,
    DecrementPressed,
}

    #[wasm_bindgen_test]
    fn test1() {
        // gtree! { layer "a" [ layer "b"  ]};
        #[emg(init)]
        struct Aa<'a> {
            x: i32,
        }
    }
}
