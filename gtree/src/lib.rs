extern crate gtree_proc_macro;
pub use console_log;
// pub use gtree_proc_macro::glayer;
pub use gtree_proc_macro::gtree;
// pub use gtree_proc_macro::gview;
pub use log;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
