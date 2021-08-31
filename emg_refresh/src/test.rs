use std::sync::Once;

/*
 * @Author: Rais
 * @Date: 2021-05-22 08:44:52
 * @LastEditTime: 2021-05-22 09:06:15
 * @LastEditors: Rais
 * @Description:
 */

#[cfg(test)]
static TEST_INIT: Once = Once::new();

#[cfg(test)]
pub fn setup_tracing() {
    TEST_INIT.call_once(|| {
        console_error_panic_hook::set_once();
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")]{
                tracing_wasm::set_as_global_default();

            }
        }
    });
}
