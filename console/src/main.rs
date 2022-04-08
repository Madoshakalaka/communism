use common::{AuthResult, ClientOpt, ContainerStatus, Newspeak, OnlinePeople, ServerStatus};
use console::interop::show_congrats_toast;
use console::interop::ResourceProvider;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use gloo_timers::callback::Timeout;
use gloo_timers::future::TimeoutFuture;
use instant::Instant;
use reqwasm::websocket::futures::WebSocket;
use reqwasm::websocket::Message;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::ops::Deref;

use std::rc::Rc;
use std::sync::Mutex;
use std::sync::RwLock;

use stylist::yew::styled_component;

use web_sys::HtmlButtonElement;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use yew_vdom_gen::prelude::*;
use yewdux::prelude::*;
use yewdux_functional::*;

// macro_rules! use_style {
//     ($a: tt) => {{
//         let style = stylist::yew::use_style!($a);
//         let style = style.get_class_name().to_owned();
//         let attr_val: AttrValue = style.into();
//         attr_val
//     }};
// }

#[derive(Clone, Serialize, Deserialize, Default)]
struct Password(String);
impl Persistent for Password {}

enum AutoSignStatus {
    NotTried,
    Trying,
    Tried,
}

#[styled_component(App)]
pub fn app() -> Html {
    let toast_ready = console::interop::use_toast();

    let server_status: UseStateHandle<Option<(ServerStatus, Instant)>> = use_state(|| None);

    let authenticated = use_state(|| false);
    let authenticating = use_state(|| false);

    let socket = if cfg!(debug_assertions) {
        "ws://localhost:3000/ws"
    } else {
        "wss://siyuanyan.net/ws"
    };

    // let open_soc = use_ref(||Arc::new(Mutex::new(WebSocket::open(socket).ok())));
    let open_soc = use_ref(|| {
        RwLock::new(WebSocket::open(socket).ok().map(|x| {
            let (write, read) = x.split();
            (Mutex::new(write), Mutex::new(read))
        }))
    });

    let re_render = use_state(|| ());

    let button_waiting = use_state(|| false);

    let input_ref = use_node_ref();
    let password = use_store::<PersistentStore<Password>>();
    let password_dispatch = use_dispatch::<PersistentStore<Password>>();
    use_ref(|| {
        // let ws = ws.clone();
        // let con_watcher = con_watcher.clone();

        let reporting_server_status = server_status.clone();
        let open_soc_report = open_soc.clone();
        let authenticated = authenticated.clone();
        let authenticating = authenticating.clone();

        let input_ref = input_ref.clone();

        // let button_waiting = button_waiting.clone();
        let reporter = async move {
            let config = bincode::config::standard();

            loop {
                {
                    let g = open_soc_report.deref().read().unwrap();
                    {
                        let mut t = g.as_ref().map(|(_, g)| g.lock().unwrap());
                        match t.as_deref_mut() {
                            Some(ws) => {
                                while let Some(Ok(Message::Bytes(b))) = ws.next().await {
                                    let n: Option<(Newspeak, _)> =
                                        bincode::decode_from_slice(b.as_slice(), config)
                                            .map_err(|e| {
                                                gloo_console::warn!(e.to_string());
                                            })
                                            .ok();

                                    match n {
                                        None => {}
                                        Some((n, _)) => {
                                            match n {
                                                Newspeak::AuthResult(r) => match r {
                                                    AuthResult::Goob => {
                                                        let input_ref = input_ref.clone();
                                                        password_dispatch.reduce(
                                                            move |x: &mut Password| {
                                                                let ele: HtmlInputElement =
                                                                    input_ref.cast().unwrap();
                                                                let val = ele.value();
                                                                if !val.is_empty() {
                                                                    x.0 = val;
                                                                };
                                                            },
                                                        );

                                                        authenticating.set(false);
                                                        authenticated.set(true);
                                                        console::interop::show_congrats_toast("welcome, fellow equal member of communism");
                                                    }
                                                    AuthResult::Sus => {
                                                        authenticating.set(false);
                                                        authenticated.set(false);
                                                        console::interop::show_execution_toast(
                                                            "sus, go ask Matt",
                                                        );
                                                    }
                                                },
                                                Newspeak::ServerStatus(s) => {
                                                    // #[cfg(debug_assertions)]
                                                    // log::debug!("received server status");
                                                    reporting_server_status
                                                        .set(Some((s, Instant::now())));
                                                }
                                                Newspeak::Feedback(f) => {
                                                    show_congrats_toast(&f);
                                                }
                                            }
                                        }
                                    };
                                }
                            }
                            None => {}
                        }
                    }
                }

                authenticated.set(false);
                button_waiting.set(false);

                {
                    let mut g = open_soc_report.deref().write().unwrap();
                    WebSocket::open(socket).ok().map(|s| {
                        let (write, read) = s.split();
                        g.replace((Mutex::new(write), Mutex::new(read)))
                    });
                }
                TimeoutFuture::new(1_000).await;
            }
        };

        let interval_refresh = {
            // let server_status = server_status.clone();
            // let seconds_elapsed_handle = seconds_elapsed.clone();
            async move {
                loop {
                    TimeoutFuture::new(300).await;

                    re_render.set(());
                    // #[cfg(debug_assertions)]
                    // log::debug!("refreshing");
                    // let new_time_elapsed = server_status.deref().as_ref().map(|(_, i)|{
                    //     Instant::now().duration_since(*i)
                    // });

                    // let new_time_elapsed = get_seconds_elapsed(server_status);
                    //
                    // if new_time_elapsed != *seconds_elapsed_handle{
                    //
                    //     seconds_elapsed_handle.set(new_time_elapsed);
                    // }
                }
            }
        };

        // let server_status = server_status.clone();
        //
        // let con_watch = async move {
        //
        //     // loop {
        //     //
        //     //
        //     //     let ws = {
        //     //         let open_soc_ref = open_soc.deref();
        //     //         let mut open_soc_ref = open_soc_ref.lock().unwrap();
        //     //         open_soc_ref.deref_mut().take()
        //     //     };
        //     //     match ws{
        //     //         Some(ws) => {
        //     //
        //     //             match ws.state() {
        //     //                 State::Closed => {
        //     //                     server_status.set(None);
        //     //                     // open_soc.deref().replace_with(|_|WebSocket::open(socket).ok());
        //     //                 },
        //     //                 State::Open => {
        //     //
        //     //                 }
        //     //                 _ => {
        //     //
        //     //                 }
        //     //             }
        //     //
        //     //
        //     //         }
        //     //         None =>{
        //     //             server_status.set(None);
        //     //             // open_soc.deref().replace_with(|_|WebSocket::open(socket).ok());
        //     //
        //     //         }
        //     //     };
        //     //
        //     //     TimeoutFuture::new(1_000).await;
        //     //
        //     // }
        // };

        wasm_bindgen_futures::spawn_local(reporter);
        wasm_bindgen_futures::spawn_local(interval_refresh);
    });

    let auto_sign = use_state(|| AutoSignStatus::NotTried);

    let stored_password = password.state().cloned().unwrap_or_default().0.clone();

    {
        let stored_password = stored_password;

        if matches!(*auto_sign, AutoSignStatus::NotTried)
            && !stored_password.is_empty()
            && toast_ready
        {
            let open_soc = open_soc.clone();
            let authenticating = authenticating.clone();
            let auto_sign = auto_sign.clone();
            auto_sign.set(AutoSignStatus::Trying);
            wasm_bindgen_futures::spawn_local(async move {
                let soc = open_soc.deref().read().unwrap();

                match soc.as_ref() {
                    None => {}
                    Some((s, _)) => {
                        authenticating.set(true);
                        s.lock()
                            .unwrap()
                            .send(Message::Text(stored_password))
                            .await
                            .ok();
                    }
                }
                auto_sign.set(AutoSignStatus::Tried);
            });
        }
    }

    let password_label = label("The password? (Ask Matt)").child({
        let input_ref = input_ref.clone();
        html! {<input type="password" ref={input_ref} autocomplete="true"/>}
    });

    let button_waiting: bool =
        { !toast_ready || matches!(*auto_sign, AutoSignStatus::Trying) || *authenticating };

    let status_display = (*server_status)
        .as_ref()
        .map(
            |(
                ServerStatus {
                    host,
                    container,
                    online,
                },
                i,
            )| {
                fragment()
                    .child(h1("Host Status"))
                    .child(p(host.clone()))
                    .child(h1("Container Status"))
                    .child(p(match container {
                        ContainerStatus::Unknown => Cow::from("unknown"),
                        ContainerStatus::Up(s) => Cow::from(s.clone()),
                        ContainerStatus::NotUp => Cow::from("not up"),
                    }))
                    .child(h1("Online People"))
                    .child(p(match online {
                        OnlinePeople::Unknown => Cow::from("unknown"),
                        OnlinePeople::Known(s) => Cow::from(s.clone()),
                    }))
                    .child(hr())
                    .child(p(Cow::from(format!(
                        "last update: {}s ago",
                        i.elapsed().as_secs()
                    ))))
            },
        )
        .unwrap_or_else(|| {
            // #[cfg(debug_assertions)]
            // log::debug!("render: status is None");
            fragment().child(p("receiving from sentinel..."))
        });

    let frag = fragment().child(h1("'Chung' Minecraft Server Dashboard"));

    let frag = if *authenticated {
        frag
    } else {
        let submit_button = {
            let button = button(if button_waiting {
                "please wait..."
            } else {
                "confirm"
            });

            if button_waiting {
                button.disabled("true".into())
            } else {
                button
            }
        };
        let password_form = {
            let authenticating = authenticating.clone();
            let open_soc = open_soc.clone();
            form()
                .child(password_label)
                .child(submit_button)
                .listener(on_submit(move |e| {
                    e.prevent_default();
                    #[cfg(debug_assertions)]
                    log::debug!("password submitted");
                    let input_ref = input_ref.clone();
                    let open_soc = open_soc.clone();
                    let authenticating = authenticating.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let soc = open_soc.deref().read().unwrap();

                        match soc.as_ref() {
                            None => {}
                            Some((s, _)) => {
                                authenticating.set(true);
                                s.lock()
                                    .unwrap()
                                    .send(Message::Text(
                                        input_ref.cast::<HtmlInputElement>().unwrap().value(),
                                    ))
                                    .await
                                    .ok();
                            }
                        }
                    });
                }))
        };

        frag.child(password_form)
    };

    let is_running_or_closed = server_status
        .deref()
        .as_ref()
        .map(|(ServerStatus { host, .. }, _)| (host.contains("running"), host.contains("stopped")));

    let send_opt = |opt: ClientOpt,
                    open_soc: Rc<
        RwLock<
            Option<(
                Mutex<SplitSink<WebSocket, Message>>,
                Mutex<SplitStream<WebSocket>>,
            )>,
        >,
    >,
                    authenticating: UseStateHandle<bool>| {
        wasm_bindgen_futures::spawn_local(async move {
            let config = bincode::config::standard();
            let soc = open_soc.deref().read().unwrap();

            match soc.as_ref() {
                None => {}
                Some((s, _)) => {
                    authenticating.set(true);
                    s.lock()
                        .unwrap()
                        .send(Message::Bytes(bincode::encode_to_vec(opt, config).unwrap()))
                        .await
                        .ok();
                }
            }
        });
    };

    let debounce = |opt: ClientOpt,
                    soc: Rc<
        RwLock<
            Option<(
                Mutex<SplitSink<WebSocket, Message>>,
                Mutex<SplitStream<WebSocket>>,
            )>,
        >,
    >,
                    authenticating: UseStateHandle<bool>| {
        on_click(move |e| {
            let soc = soc.clone();
            let authenticating = authenticating.clone();
            let b: HtmlButtonElement = e.target_unchecked_into();
            b.set_disabled(true);

            let t = Timeout::new(2_000, move || b.set_disabled(false));
            t.forget();
            send_opt(opt, soc, authenticating);
        })
    };

    let reboot_button = button("reboot");
    let reboot_button = if let Some((true, _)) = is_running_or_closed {
        if !*authenticated || !toast_ready {
            reboot_button.disabled("true".into())
        } else {
            reboot_button
        }
    } else {
        reboot_button.disabled("true".into())
    }
    .listener(debounce(
        ClientOpt::Reboot,
        open_soc.clone(),
        authenticating.clone(),
    ));

    let shutdown_button = button("shutdown");
    let shutdown_button = if let Some((true, _)) = is_running_or_closed {
        if !*authenticated || !toast_ready {
            shutdown_button.disabled("true".into())
        } else {
            shutdown_button
        }
    } else {
        shutdown_button.disabled("true".into())
    }
    .listener(debounce(
        ClientOpt::Off,
        open_soc.clone(),
        authenticating.clone(),
    ));

    let power_on_button = button("power on");
    let power_on_button = if let Some((_, true)) = is_running_or_closed {
        if !*authenticated || !toast_ready {
            power_on_button.disabled("true".into())
        } else {
            power_on_button
        }
    } else {
        power_on_button.disabled("true".into())
    }
    .listener(debounce(ClientOpt::On, open_soc, authenticating));

    frag.child(reboot_button)
        .child(power_on_button)
        .child(shutdown_button)
        .child(status_display)
        .into()
}

#[function_component(ContextedApp)]
pub fn contexted_app() -> Html {
    html! {
        <ResourceProvider>
            <App/>
        </ResourceProvider>
    }
}

fn main() {
    #[cfg(debug_assertions)]
    wasm_logger::init(wasm_logger::Config::default());
    yew::set_event_bubbling(false);
    yew::start_app::<ContextedApp>();
}
