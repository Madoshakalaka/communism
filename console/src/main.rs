use crate::WsStatus::Created;
use common::{ContainerStatus, OnlinePeople, ServerStatus};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use gloo_timers::callback::Timeout;
use gloo_timers::future::TimeoutFuture;
use instant::Instant;
use reqwasm::websocket::futures::WebSocket;
use reqwasm::websocket::{Message, State};
use serde::{Deserialize, Serialize};
use std::borrow::{Borrow, BorrowMut, Cow};
use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
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

#[derive(Clone, Serialize, Deserialize, Default)]
struct Password(String);
impl Persistent for Password {}

enum AutoSignStatus {
    NotTried,
    Trying,
    Tried,
}

enum WsStatus {
    NotCreated,
    Connecting,
    Created(WebSocket),
}

enum WsAction {
    Connect,
    Created(WebSocket),
    Closed,
}

impl WsStatus {
    async fn next(&mut self) -> Option<ServerStatus> {
        match self {
            WsStatus::NotCreated => None,
            WsStatus::Connecting => None,
            Created(s) => {
                let config = bincode::config::standard();
                let next = s.next().await;
                if let Message::Bytes(b) = next.transpose().ok()?? {
                    let (d, _) = bincode::decode_from_slice(b.as_slice(), config).ok()?;
                    d
                } else {
                    None
                }
            }
        }
    }
}

impl Reducible for WsStatus {
    type Action = WsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            WsAction::Connect => Rc::new(Self::Connecting),
            WsAction::Created(s) => Rc::new(Created(s)),
            WsAction::Closed => Rc::new(Self::NotCreated),
        }
    }
}

#[styled_component(App)]
pub fn app() -> Html {
    let server_status: UseStateHandle<Option<(ServerStatus, Instant)>> = use_state(|| None);

    let seconds_elapsed: UseStateHandle<Option<u64>> = use_state(|| None);

    // let ws = use_reducer(|| WsStatus::NotCreated);

    // let ws = use_mut_ref(|| WebSocket::open(socket));

    // if let WsStatus::NotConnected = *ws {
    //     ws.dispatch(WsAction::Connect);
    //
    //     let ws = ws.dispatcher();
    //     wasm_bindgen_futures::spawn_local(async move {
    //         loop{
    //             match WebSocket::open(socket){
    //                 Ok(s) => {
    //                     ws.dispatch(WsAction::Connected(s));
    //                     break
    //                 }
    //                 _ =>{
    //                     TimeoutFuture::new(1_000).await;
    //                 }
    //             }
    //         }
    //     });
    // }

    // let (con_watch_tx, con_watch_rx) =

    // let con_watcher = use_mut_ref(||{
    //     tokio::sync::watch::channel(false)
    // });

    let socket = if cfg!(debug_assertions) {
        "ws://localhost:3000/ws"
    } else {
        "wss://siyuanyan.net/ws"
    };

    // let open_soc = use_ref(||Arc::new(Mutex::new(WebSocket::open(socket).ok())));
    let open_soc = use_mut_ref(|| WebSocket::open(socket).ok());

    let re_render = use_state(|| ());

    use_ref(|| {
        // let ws = ws.clone();
        // let con_watcher = con_watcher.clone();

        let reporting_server_status = server_status.clone();
        let open_soc_report = open_soc.clone();
        let reporter = async move {
            let config = bincode::config::standard();

            loop {
                {
                    let g = open_soc_report.deref();
                    {
                        let mut t = g.deref().borrow_mut();
                        match t.deref_mut() {
                            Some(ws) => {
                                let (_, mut read) = ws.split();

                                while let Some(Ok(Message::Bytes(b))) = read.next().await {
                                    let s: Option<(ServerStatus, Instant)> =
                                        bincode::decode_from_slice(b.as_slice(), config)
                                            .ok()
                                            .map(|(s, _)| (s, Instant::now()));
                                    reporting_server_status.set(s);
                                }
                            }
                            None => {}
                        }
                    }
                }

                open_soc_report.replace_with(|_| WebSocket::open(socket).ok());
                TimeoutFuture::new(1_000).await;
            }
        };

        let interval_refresh = {
            let server_status = server_status.clone();
            let seconds_elapsed_handle = seconds_elapsed.clone();
            async move {
                loop {
                    TimeoutFuture::new(300).await;

                    re_render.set(());
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

    let password = use_store::<PersistentStore<Password>>();

    let auto_sign = use_state(|| AutoSignStatus::NotTried);

    let button_waiting = use_state(|| false);

    let stored_password = password.state().cloned().unwrap_or_default().0.clone();
    if matches!(*auto_sign, AutoSignStatus::NotTried) && !stored_password.is_empty() {
        auto_sign.set(AutoSignStatus::Trying);
        button_waiting.set(true);
    }

    let password_input = input().r#type("password".into());

    let password_label = label("The password? (Ask Matt)").child(password_input);

    let submit_button = {
        let button = button(if *button_waiting {
            "authenticating..."
        } else {
            "confirm"
        });

        if *button_waiting {
            button.disabled("true".into())
        } else {
            button
        }
    };

    let password_form = form()
        .child(password_label)
        .child(submit_button)
        .listener(on_submit(|e| {
            e.prevent_default();
            #[cfg(debug_assertions)]
            log::debug!("password submitted");
        }));

    let status_display = (*server_status)
        .as_ref()
        .map(|(ServerStatus{host, container, online}, i)|{
            fragment()
                .child(h1("Host Status"))
                .child(p(host.clone()))
                .child(h1("Container Status"))
                .child(
                    p(
                        match container {
                            ContainerStatus::Unknown =>{
                                Cow::from("unknown")
                            }
                            ContainerStatus::Up(s) => Cow::from(s.clone()),
                            ContainerStatus::NotUp => Cow::from("not up"),
                        }
                    )
                )
                .child(h1("Online People"))
                .child(
                    p(
                        match online  {
                            OnlinePeople::Unknown =>{
                                Cow::from("unknown")
                            }
                            OnlinePeople::Known(s) => Cow::from(s.clone()),
                        }
                    )
                )
                .child(hr())
                .child(
                    p(
                        Cow::from(format!(
                            "last update: {}s ago",
                            i.elapsed().as_secs()
                        ))
                    )
                )
        })
        .unwrap_or_else(|| fragment()
                .child(
                    p("receiving from sentinel...")
                ));


    fragment()
        .child(h1("'Chung' Minecraft Server Dashboard"))
        .child(password_form)
        .child(status_display)
        .into()
}

fn main() {
    #[cfg(debug_assertions)]
    wasm_logger::init(wasm_logger::Config::default());
    yew::set_event_bubbling(false);
    yew::start_app::<App>();
}
