use yew_interop::declare_resources;
use wasm_bindgen::prelude::*;
use js_sys::{Object, Reflect};
declare_resources!{
    toast
    "https://cdn.jsdelivr.net/npm/toastify-js@1.11.2/src/toastify.min.js"
    "https://cdn.jsdelivr.net/npm/toastify-js@1.11.2/src/toastify.min.css"
}

// The javascript API: https://github.com/apvarun/toastify-js/blob/572517040fae6a7f8be4a99778dacda9c933db45/README.md
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = Toastify)]
    pub type Toast;

    #[wasm_bindgen(constructor, js_class = "Toastify")]
    pub fn new(config: &JsValue) -> Toast;

    #[wasm_bindgen(method, structural, js_class = "Toastify", js_name = showToast)]
    pub fn show_toast(this: &Toast);
}
pub fn show_congrats_toast(text: &str) {
    let config = Object::new();
    Reflect::set(
        &config,
        &"text".into(),
        &text.to_string().into()
    )
        .ok();
    let toast = Toast::new(&config);
    toast.show_toast();
}
pub fn show_execution_toast(text: &str) {
    let config = Object::new();
    let style = Object::new();
    Reflect::set(
        &config,
        &"text".into(),
        &text.to_string().into()
    )
        .ok();
    Reflect::set(
        &style,
        &"background".into(),
        &"red".into()
    )
        .ok();
    Reflect::set(
        &config,
        &"style".into(),
        &style
    )
        .ok();
    let toast = Toast::new(&config);
    toast.show_toast();
}
