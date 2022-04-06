use serde::{Deserialize, Serialize};
use stylist::yew::styled_component;
use stylist::{Style, StyleSource};
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use yew::virtual_dom::AttrValue;
use yew_vdom_gen::prelude::*;
use yewdux::prelude::*;
use yewdux_functional::*;

macro_rules! use_style {
    ($a: tt) => {{
        let style = stylist::yew::use_style!($a);
        let style = style.get_class_name().to_owned();
        let attr_val: AttrValue = style.into();
        attr_val
    }};
}

#[styled_component(App)]
pub fn app() -> Html {
    let password_input = input().r#type("password".into());

    let password_label = label("The password? (Ask Matt)")
        .child(password_input);

    let password_form = form()
        .child(password_label)
        .child(br())
        .child(button("confirm"))
        .listener(on_submit(|e| {
            e.prevent_default();
            #[cfg(debug_assertions)]
            log::debug!("password submitted");
        }));

    fragment()
        .child(h1("'Chung' Minecraft Server Dashboard"))
        .child(password_form)
        .into()
}

fn main() {
    #[cfg(debug_assertions)]
    wasm_logger::init(wasm_logger::Config::default());
    yew::set_event_bubbling(false);
    yew::start_app::<App>();
}
