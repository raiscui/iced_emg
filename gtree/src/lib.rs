pub use console_log;
pub use gtree_macro::kw::layer;
pub use gtree_proc_macro::gtree;
pub use log;
pub use illicit;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
