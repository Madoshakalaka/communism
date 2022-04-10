use fov_calculator::Degree;
use serde::{Deserialize, Serialize};
use stylist::yew::styled_component;

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
/// production/development aware static url
#[cfg(debug_assertions)]
macro_rules! static_url {
    ($rest:tt) => {
        concat!("/minecraft/fov-calculator/image/", $rest)
    };
}
#[cfg(not(debug_assertions))]
macro_rules! static_url {
    ($rest:tt) => {
        concat!(
            "https://assets.siyuanyan.net/minecraft/fov-calculator/image/",
            $rest
        )
    };
}

#[function_component(App)]
pub fn app() -> Html {
    let tip_list = use_style!("padding-left: 0;");


    let portrait_image = use_style!("max-width: 100%;");
    fragment()
        .child(h1("Minecraft FOV Calculator - Kill Motion Sickness"))
        .child(html! {<FovCalculator/>})
        .child(hr())
        .child(h1("How it Works"))
        .child(p("By using trigonometry, it calculates the FOV such that the game's window looks like a portal to the game world, not too wide, not too narrow.")
            .child(br())
            .child("This convinces your brain that what you are seeing is real and stops dizziness.")
            )
        .child(h1("Other Tips"))
        .child(ul()
            .class(tip_list)
            .child(
                li()
                    .child("Disable 'View Bobbing'")
            )
            .child(
                li()
                    .child("Use a realistic looking shader, I recommend SEUS")
            )
            .child(
                li()
                    .child("Disable 'Dynamic FOV'")
            )
            .child(
                li()
                    .child("Lower mouse dpi")
            )
        )
        .child(
            p("It might be impossible to get a playable FOV from a 'small' 27 inch monitor in landscape mode. \
              You would have to either sit very very close or turn the FOV way down which affects gameplay a lot. \
              I actually recommend playing in portrait mode on these monitors, \
              you can also try to play on huge TVs too, like 50 inch ones.")
        )
        .child(
            figure()
                .child(
                    img()
                        .src(static_url!("portrait-27-inch.jpg").into())
                        .alt("use portrait mode now".into())
                        .class(portrait_image)
                ).
                child(
                    figcaption()
                        .child("portrait mode actually looks decent")
                )
        )
        .into()
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Copy)]
enum Unit {
    Cm,
    Inch,
}

impl Default for Unit {
    fn default() -> Self {
        Self::Cm
    }
}

impl Unit {
    fn into_option_value(self) -> &'static str {
        match self {
            Unit::Cm => "cm",
            Unit::Inch => "inch",
        }
    }
    fn from_option_value(v: &str) -> Self {
        match v {
            "cm" => Self::Cm,
            "inch" => Self::Inch,
            _ => {
                panic!()
            }
        }
    }

    fn toggle_to(&self, v: f64) -> f64 {
        match self {
            Unit::Cm => v * 2.54,
            Unit::Inch => v / 2.54,
        }
    }
}

// #[derive(Clone, Serialize, Deserialize)]
// struct GameWidth {
//     value: f64,
//     unit: Unit,
// }

// impl Default for GameWidth {
//     fn default() -> Self {
//         Self {
//             value: 59.8,
//             unit: Unit::Cm,
//         }
//     }
// }
//
// impl Persistent for GameWidth {}

#[derive(Clone, Serialize, Deserialize)]
struct GameHeight {
    value: f64,
    unit: Unit,
}

impl Default for GameHeight {
    fn default() -> Self {
        Self {
            value: 31.6,
            unit: Unit::Cm,
        }
    }
}

impl Persistent for GameHeight {}

#[derive(Clone, Serialize, Deserialize)]
struct EyeDistance {
    value: f64,
    unit: Unit,
}

impl Default for EyeDistance {
    fn default() -> Self {
        Self {
            value: 65.0,
            unit: Unit::Cm,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Notes(Vec<(GameHeight, EyeDistance, String)>);

impl Default for Notes {
    fn default() -> Self {
        Self(vec![
            (
                GameHeight {
                    value: 31.6,
                    unit: Unit::Cm,
                },
                EyeDistance {
                    value: 52.0,
                    unit: Unit::Cm,
                },
                "sitting straight but relaxed, 27'' monitor, maximized window".to_string(),
            ),
            (
                GameHeight {
                    value: 31.6,
                    unit: Unit::Cm,
                },
                EyeDistance {
                    value: 22.0,
                    unit: Unit::Cm,
                },
                "sitting close, 27'' monitor, maximized window".to_string(),
            ),
            (
                GameHeight {
                    value: 33.6,
                    unit: Unit::Cm,
                },
                EyeDistance {
                    value: 69.0,
                    unit: Unit::Cm,
                },
                "slouched, 27'' monitor, fullscreen".to_string(),
            ),
        ])
    }
}

impl Persistent for Notes {}

impl Persistent for EyeDistance {}

#[function_component(FovCalculator)]
pub fn fov_calc() -> Html {
    // let width = use_store::<PersistentStore<GameWidth>>();
    let height = use_store::<PersistentStore<GameHeight>>();
    let distance = use_store::<PersistentStore<EyeDistance>>();
    let notes = use_store::<PersistentStore<Notes>>();

    // let width_input_ref = NodeRef::default();
    // let width_input = html! {<input oninput={
    //     let width = width.dispatch().clone();
    //
    //     move |e: InputEvent|{
    //             let input: HtmlInputElement = e.target_unchecked_into();
    //             if let Ok(input_width) = input.value().parse::<f64>(){
    //             width.reduce(move |w|{
    //             w.value = input_width
    //         });
    //         }
    //
    //         }
    // } ref={width_input_ref.clone()} type="number" value = {width.state().map(|w| w.value).unwrap_or_default().to_string()}/> };
    //
    // let width_label = label(
    //     ""
    // )
    //     .child(h2("The width of your Minecraft window"))
    //     .child(p("If you play in fullscreen or maximized windowed mode, it's the width of your monitor.")
    //         .child(br())
    //         .child("Note: a 27 inch monitor has a DIAGONAL length of 27 inch, not the width. look up monitor dimensions ")
    //         .child(a("here")
    //             .href("https://en.tab-tv.com/?page_id=7333#:~:text=Size%20screen%2019%2D105%20inch%20height%20and%20width".into()).rel("noopener".into()).target("_blank".into()))
    //     )
    // .child(br())
    // .child(
    //     width_input
    // )
    // .child(
    //     select()
    //         .child(option().selected("selected".into()).text(Unit::Cm.into_option_value()))
    //         .child(option().text(Unit::Inch.into_option_value()))
    //         .listener({
    //             let width = width.dispatch().clone();
    //
    //             on_change(move |e| {
    //             let element = e.target_unchecked_into::<HtmlSelectElement>();
    //             let u = Unit::from_option_value(&element.value());
    //                 width.reduce(move |w| w.unit = u);
    //         })}),
    // );

    let height_input_ref = NodeRef::default();
    let height_input = html! {<input oninput={
        let height = height.dispatch().clone();

        move |e: InputEvent|{
                let input: HtmlInputElement = e.target_unchecked_into();
                if let Ok(input_height) = input.value().parse::<f64>(){
                height.reduce(move |w|{
                w.value = input_height
            });
            }

            }
    } ref={height_input_ref.clone()} type="number" value = {height.state().map(|w| w.value).unwrap_or_default().to_string()}/> };

    let height_label = label(
        ""
    )
        .child(h2("The height of your Minecraft window"))
        .child(p("If you play in fullscreen mode, it's the height of your monitor. If you play in maximized window mode, it's a bit less than the height of your monitor (by roughly 2cm).")
            .child(br())
                       .child("Note: a 27 inch monitor has a DIAGONAL length of 27 inch, not the height. look up monitor dimensions ")
                       .child(a("here")
                           .href("https://en.tab-tv.com/?page_id=7333#:~:text=Size%20screen%2019%2D105%20inch%20height%20and%20width".into()).rel("noopener".into()).target("_blank".into()))


        )
        .child(br())
        .child(
            height_input
        )
        .child(
            select()
                .child(option().selected("selected".into()).text(Unit::Cm.into_option_value()))
                .child(option().text(Unit::Inch.into_option_value()))
                .listener({
                    let height = height.dispatch().clone();

                    on_change(move |e| {
                        let element = e.target_unchecked_into::<HtmlSelectElement>();
                        let u = Unit::from_option_value(&element.value());
                        height.reduce(move |v| v.unit = u);
                    })}),
        );

    let distance_input_ref = NodeRef::default();

    let distance_input = html! {<input oninput={
        let distance = distance.dispatch().clone();
        move |e: InputEvent|{
                let input: HtmlInputElement = e.target_unchecked_into();
                if let Ok(input_distance) = input.value().parse::<f64>(){
                distance.reduce(move |d| d.value = input_distance);
            }

            }
    } ref={distance_input_ref.clone()} type="number" value = {distance.state().map(|s| s.value).unwrap_or_default().to_string()}/> };

    let distance_label = label("")
        .child(h2(
            "The distance between your eyes and the center of the screen",
        ))
        .child(distance_input)
        .child(
            select()
                .child(
                    option()
                        .selected("selected".into())
                        .text(Unit::Cm.into_option_value()),
                )
                .child(option().text(Unit::Inch.into_option_value()))
                .listener(on_change({
                    let distance = distance.dispatch().clone();
                    move |e| {
                        let element = e.target_unchecked_into::<HtmlSelectElement>();
                        let u = Unit::from_option_value(&element.value());
                        distance.reduce(move |d| d.unit = u);
                    }
                })),
        );

    let mut result = p("BEST FOV: ");

    let right_margin = use_style!("margin-right: 2rem;");

    result = if let (Some(height), Some(distance)) = (height.state(), distance.state()) {
        #[cfg(debug_assertions)]
        log::debug!("computing result");

        let best = calculate_fov(height, distance);

        let add_note_dispatch = notes.dispatch().clone();

        // let width = (**width).clone();
        let height = (**height).clone();
        let distance = (**distance).clone();

        result
            .child(span().child(format!("{best:.0}")).class(right_margin))
            .child(
                button("Take Notes (chair position, posture, monitor etc.)").listener(on_click(
                    move |_| {
                        // let width = width.clone();
                        let height = height.clone();
                        let distance = distance.clone();

                        add_note_dispatch.reduce(move |n| {
                            let note_text =
                                gloo_dialogs::prompt("Note text:", None).unwrap_or_default();
                            n.0.push((height, distance, note_text))
                        })
                    },
                )),
            )
    } else {
        result
    };

    let note_table_class = use_style!(
        r"
    td, th{
        padding: 0.5rem;
    }
    "
    );

    let note_table = table().class(note_table_class).child(
        tr()
            // .child(th().child("window width"))
            .child(th().child("window height"))
            .child(th().child("distance"))
            .child(th().child("FOV"))
            .child(th().child("notes")),
    );

    let delete_button = use_style!("background: red; color: white;");

    let delete_note_dispatch = notes.dispatch();
    let notes = match notes.state() {
        Some(notes) => {
            let make_table =
                |ind: usize,
                 (
                    ref h @ GameHeight {
                        value: height,
                        unit: height_unit,
                    },
                    ref d @ EyeDistance {
                        value: distance,
                        unit: distance_unit,
                    },
                    s,
                ),
                 delete_button_class: AttrValue,
                 delete_note_dispatch: Dispatch<PersistentStore<Notes>>| {
                    let best = calculate_fov(h, d);
                    // let width_unit = width_unit.into_option_value();
                    let height_unit = height_unit.into_option_value();
                    let distance_unit = distance_unit.into_option_value();

                    tr()
                        // .child(td().child(format!("{width} {width_unit}")))
                        .child(td().child(format!("{height} {height_unit}")))
                        .child(td().child(format!("{distance} {distance_unit}")))
                        .child(td().child(format!("{best:.0}")))
                        .child(td().child(s))
                        .child(
                            td().child(button("delete").class(delete_button_class).listener(
                                on_click(move |_| {
                                    delete_note_dispatch.reduce(move |n| {
                                        n.0.remove(ind);
                                    })
                                }),
                            )),
                        )
                };

            notes
                .0
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, x)| make_table(i, x, delete_button.clone(), delete_note_dispatch.clone()))
                .fold(note_table, |acc, x| acc.child(x))
        }
        None => note_table,
    };

    fragment()
        .child(height_label)
        .child(distance_label)
        .child(result)
        .child(notes)
        .into()
}

fn calculate_fov(height: &GameHeight, distance: &EyeDistance) -> Degree {
    // let width = if matches!(width.unit, Unit::Inch) {
    //     #[cfg(debug_assertions)]
    //     log::debug!("width has unit inch, converting to cm");
    //     Unit::Cm.toggle_to(width.value)
    // } else {
    //     width.value
    // };
    let height = if matches!(height.unit, Unit::Inch) {
        #[cfg(debug_assertions)]
        log::debug!("height has unit inch, converting to cm");
        Unit::Cm.toggle_to(height.value)
    } else {
        height.value
    };

    let distance = if matches!(distance.unit, Unit::Inch) {
        #[cfg(debug_assertions)]
        log::debug!("distance has unit inch, converting to cm");
        Unit::Cm.toggle_to(distance.value)
    } else {
        distance.value
    };

    fov_calculator::calculate_fov(height, distance)
}

fn main() {
    #[cfg(debug_assertions)]
    wasm_logger::init(wasm_logger::Config::default());
    yew::set_event_bubbling(false);
    yew::start_app::<App>();
}
