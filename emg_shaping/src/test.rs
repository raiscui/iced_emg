use std::sync::Once;

/*
 * @Author: Rais
 * @Date: 2021-05-22 08:44:52
 * @LastEditTime: 2023-01-31 12:49:23
 * @LastEditors: Rais
 * @Description:
 */

#[cfg(test)]
static TEST_INIT: Once = Once::new();

#[cfg(test)]
pub fn setup_tracing() {
    TEST_INIT.call_once(|| {
        console_error_panic_hook::set_once();
        #[cfg(target_arch = "wasm32")]
        tracing_wasm::set_as_global_default();
    });
}
