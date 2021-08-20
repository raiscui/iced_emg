use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

/*
 * @Author: Rais
 * @Date: 2021-06-26 16:46:18
 * @LastEditTime: 2021-08-18 19:18:48
 * @LastEditors: Rais
 * @Description:
 */
#[wasm_bindgen(module = "/js/resize_observe.js")]
extern "C" {
    #[wasm_bindgen(js_name = observeSize)]
    pub fn observe_size(element: &web_sys::Element, callback: &JsValue);

}
