use aws_sdk_ec2::model::InstanceStateName;
use aws_sdk_ec2::{Client, Error as Ec2Error};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        TypedHeader,
    },
    headers,
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use common::{AuthResult, ClientOpt, ContainerStatus, Newspeak, OnlinePeople, ServerStatus};
use futures::{sink::SinkExt, stream::StreamExt};
use ssh2::Session;
use std::borrow::BorrowMut;

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::Read;
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::ops::{AddAssign, SubAssign};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use axum_server::tls_rustls::RustlsConfig;
use tokio::sync::watch::Receiver;
use tokio::sync::{Mutex, Notify};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

macro_rules! send_to_instance {
    ($client:ident, $id:ident) => {
        $client
            .$id()
            .set_instance_ids(Some(vec!["i-04f77bba0b522dfbe".to_string()]))
            .send()
            .await
            .unwrap()
    };
}

async fn show_state(client: &Client) -> Result<InstanceStateName, Ec2Error> {
    let resp = send_to_instance!(client, describe_instances);

    let instance = resp
        .reservations()
        .unwrap()
        .first()
        .unwrap()
        .instances()
        .unwrap()
        .first()
        .unwrap();
    Ok(instance.state().unwrap().name().unwrap().to_owned())
}

struct TimeoutError;

impl Debug for TimeoutError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Oh no, timed out")
    }
}

impl Display for TimeoutError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for TimeoutError {}

// async fn poll_until(desired_state: InstanceStateName, client: &Client) -> Result<(), TimeoutError> {
//     println!("I'm polling until the state of the machine is {desired_state:?}...");
//
//     let endless_poll = async {
//         let mut state = show_state(client).await.unwrap();
//         while state != desired_state {
//             println!("Server is {state:?}");
//             state = show_state(client).await.unwrap();
//             tokio::time::sleep(Duration::from_secs(3)).await;
//         }
//     };
//
//     tokio::select! {
//         _ = endless_poll =>{
//             println!("Yay! Server is now {desired_state:?}!");
//             Ok(())
//         }
//         _ = tokio::time::sleep(Duration::from_secs(60)) => {
//             println!("Uh oh, timed out");
//             Err(TimeoutError)
//         }
//     }
// }

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    Extension(rx): Extension<Receiver<Option<ServerStatus>>>,
    Extension(client): Extension<Client>,
    Extension(con_notify): Extension<Arc<Notify>>,
    Extension(con_count): Extension<Arc<Mutex<u16>>>,
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        println!("`{}` connected", user_agent.as_str());
    }

    ws.on_upgrade(|socket: WebSocket| handle_socket(socket, rx, client, con_count, con_notify))
}
async fn handle_socket(
    socket: WebSocket,
    mut rx: Receiver<Option<ServerStatus>>,
    client: Client,
    con_count: Arc<Mutex<u16>>,
    con_notify: Arc<Notify>,
) {
    let (sender, mut receiver) = socket.split();

    con_notify.notify_one();

    {
        let mut con = con_count.lock().await;
        let con = con.borrow_mut();
        con.add_assign(1);
    }
    let sender = Arc::new(Mutex::new(sender));

    let config = bincode::config::standard();

    let broadcast_status = {
        let sender = sender.clone();

        async move {
            loop {
                rx.changed().await.ok();
                let s = rx.borrow_and_update().as_ref().cloned();
                if let Some(s) = s {
                    if let Ok(s) = bincode::encode_to_vec(Newspeak::ServerStatus(s), config) {
                        let send_result = {
                            let mut sender = sender.lock().await;
                            let sender = sender.borrow_mut();
                            sender.send(Message::Binary(s)).await
                        };

                        if send_result.is_err() {
                            // client disconnected
                            return;
                        }
                    }
                }
            }
        }
    };

    let receive_commands = async {
        'outer: loop {
            if let Some(Ok(m)) = receiver.next().await {
                match m {
                    Message::Text(p) => {
                        if p == dotenv::var("PASSWORD").unwrap() {
                            break 'outer;
                        } else {
                            tracing::info!("wrong password received");
                            sender
                                .lock()
                                .await
                                .send(Message::Binary(
                                    bincode::encode_to_vec(
                                        Newspeak::AuthResult(AuthResult::Sus),
                                        config,
                                    )
                                    .unwrap(),
                                ))
                                .await
                                .ok();
                        }
                    }
                    Message::Close(_) => return,
                    _ => {}
                }
            } else {
                return;
            }
        }

        tracing::info!("a client authenticated");
        sender
            .lock()
            .await
            .send(Message::Binary(
                bincode::encode_to_vec(Newspeak::AuthResult(AuthResult::Goob), config).unwrap(),
            ))
            .await
            .ok();

        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(_) => {
                        // println!("client send str: {:?}", t);
                    }
                    Message::Binary(d) => {
                        if let Ok((decoded, _)) =
                            bincode::decode_from_slice::<ClientOpt, _>(d.as_slice(), config)
                        {
                            match decoded {
                                ClientOpt::On => {
                                    tracing::info!("received power on request");
                                    send_to_instance!(client, start_instances);
                                    sender.lock().await.send(Message::Binary(bincode::encode_to_vec(Newspeak::Feedback("sentinel acknowledged the request, will boot the host shortly".to_string()), config).unwrap())).await.ok();
                                }
                                ClientOpt::Off => {
                                    tracing::info!("received power off request");
                                    send_to_instance!(client, stop_instances);
                                    sender.lock().await.send(Message::Binary(bincode::encode_to_vec(Newspeak::Feedback("sentinel acknowledged the request, will shutdown the host shortly".to_string()), config).unwrap())).await.ok();
                                }
                                ClientOpt::Reboot => {
                                    tracing::info!("received reboot request");
                                    send_to_instance!(client, reboot_instances);
                                    sender.lock().await.send(Message::Binary(bincode::encode_to_vec(Newspeak::Feedback("sentinel acknowledged the request, will reboot the host shortly".to_string()), config).unwrap())).await.ok();
                                }
                            }
                        }
                    }
                    Message::Ping(_) => {
                        // println!("socket ping");
                    }
                    Message::Pong(_) => {
                        // println!("socket pong");
                    }
                    Message::Close(_) => {
                        // println!("client disconnected");
                        return;
                    }
                }
            } else {
                // println!("client disconnected");
                return;
            }
        }
    };

    tokio::select!(
        _ = broadcast_status => {

        }
        _ = receive_commands => {

        }
    );

    let send_result = {
        let mut sender = sender.lock().await;
        let sender = sender.borrow_mut();
        sender.send(Message::Close(None)).await
    };
    send_result.ok();
    {
        let mut con = con_count.lock().await;
        let con = con.borrow_mut();
        con.sub_assign(1);
    }
    tracing::info!("a client left");
}

async fn poll_server_status(client: &Client) -> Option<ServerStatus> {
    let host = show_state(client)
        .await
        .map_or("failed to get instance state".to_string(), |n| {
            n.as_str().to_string()
        });

    let (container, online) = if host.contains("running") {
        let addr = SocketAddr::new(
            IpAddr::from_str(&dotenv::var("MC_HOST").unwrap()).unwrap(),
            22,
        );
        let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(3)).ok()?;
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();
        sess.userauth_pubkey_file(
            "root",
            None,
            dotenv::var("PRIVATE_KEY").unwrap().as_ref(),
            None,
        )
        .unwrap();
        sess.set_keepalive(false, 40);
        let mut channel = sess.channel_session().ok()?;
        channel.exec("docker container ls").ok()?;
        let mut s = String::new();
        let b = channel.read_to_string(&mut s).ok();
        // println!("{}", s);
        channel.wait_close().ok();
        // println!("{}", channel.exit_status() .unwrap());
        let container = b
            .map(|_| {
                let mut s = s.trim().split('\n');

                let cell_bounds = s.next().map(|x| (x.find("STATUS"), x.find("PORTS")));

                cell_bounds
                    .and_then(|lr| {
                        if let (Some(l), Some(r)) = lr {
                            Some((l, r))
                        } else {
                            None
                        }
                    })
                    .map(|(l, r)| {
                        s.next()
                            .map(|x| ContainerStatus::Up((&x[l..r]).to_string()))
                            .unwrap_or(ContainerStatus::NotUp)
                    })
                    .unwrap_or(ContainerStatus::Unknown)
            })
            .unwrap_or(ContainerStatus::Unknown);

        let mut channel = sess.channel_session().ok()?;
        channel.exec("docker exec root-mc-1 rcon-cli list").ok()?;
        let mut s = String::new();

        channel.read_to_string(&mut s).ok();
        // println!("{}", s);
        channel.wait_close().ok();
        // println!("{}", channel.exit_status() .unwrap());
        let online = strip_ansi_escapes::strip(s)
            .ok()
            .and_then(|x| String::from_utf8(x).ok())
            .map(OnlinePeople::Known)
            .unwrap_or(OnlinePeople::Unknown);

        (container, online)
    } else {
        (ContainerStatus::NotUp, OnlinePeople::Unknown)
    };

    Some(ServerStatus {
        host,
        container,
        online,
    })
}

#[tokio::main]
async fn main() -> Result<(), Ec2Error> {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "sentinel=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // let auth = RequireAuthorizationLayer::bearer("elfiscute");

    let (tx, rx) = tokio::sync::watch::channel::<Option<ServerStatus>>(None);

    // todo: handle log streaming. Can't be done with watch because no log must be lost.

    // ap-east-1 is Hong Kong
    let shared_config = aws_config::from_env().region("ap-east-1").load().await;

    let con_notify = Arc::new(tokio::sync::Notify::new());
    let con_count = Arc::new(Mutex::new(0u16));

    let client = Client::new(&shared_config);

    let poll_client = client.clone();
    let endless_poll = {
        let con_count = con_count.clone();
        let con_notify = con_notify.clone();
        async move {
            loop {
                tracing::info!("waiting for connection to start polling server");
                con_notify.notified().await;

                while con_count.lock().await.gt(&0) {
                    tracing::trace!("polling the server");
                    let status = poll_server_status(&poll_client)
                        .await
                        .unwrap_or(ServerStatus {
                            host: "unknown".to_string(),
                            container: ContainerStatus::Unknown,
                            online: OnlinePeople::Unknown,
                        });
                    tx.send(Some(status)).ok();
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
                tracing::info!("no websocket connection remains");
            }
        }
    };

    let ws_router = Router::new()
        .route("/", get(ws_handler))
        .layer(Extension(rx))
        .layer(Extension(client))
        .layer(Extension(con_count))
        .layer(Extension(con_notify));

    let app = Router::new().nest("/ws", ws_router);

    let config = RustlsConfig::from_pem_file(
        dotenv::var("CERT_FILE").unwrap(),
        dotenv::var("KEY_FILE").unwrap(),
    )
        .await
        .unwrap();

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("https listening on {}", addr);
    let server = axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service());


    // let server =
    //     axum::Server::bind(&"0.0.0.0:3000".parse().unwrap()).serve(app.into_make_service());

    tokio::select! {
        _ = endless_poll =>{

        }
        _ = server => {

        }
    }

    //
    // let args = Args::parse();
    // match args.opt {
    //     Opt::On => {
    //         let state = show_state(&client).await?;
    //         match state {
    //             InstanceStateName::Pending => {
    //                 println!("Instance is pending, it will start soon. Instead of sending the power on command again. I'm just gonna poll until it's running.");
    //                 poll_until(InstanceStateName::Running, &client)
    //                     .await
    //                     .unwrap();
    //             }
    //             InstanceStateName::Running => {
    //                 println!("Instance is already running. I'm not doing anything")
    //             }
    //             InstanceStateName::ShuttingDown => {
    //                 println!("Instance is shutting down. I'll poll until it's completely shut down and then I'll send the power on command for ya.");
    //                 poll_until(InstanceStateName::Stopped, &client)
    //                     .await
    //                     .unwrap();
    //                 send_on_and_poll(&client).await;
    //             }
    //             InstanceStateName::Stopped => {
    //                 send_on_and_poll(&client).await;
    //             }
    //             InstanceStateName::Stopping => {
    //                 println!("Instance is stopping. I'll poll until it's completely shut down and then I'll send the power on command for ya.");
    //                 poll_until(InstanceStateName::Stopped, &client)
    //                     .await
    //                     .unwrap();
    //                 send_on_and_poll(&client).await;
    //             }
    //             InstanceStateName::Terminated => {
    //                 panic!("there is no server anymore :ferrissweatballz:")
    //             }
    //             _ => {
    //                 panic!("Owo what's this")
    //             }
    //         }
    //     }
    //     Opt::Off => {
    //         let state = show_state(&client).await?;
    //         match state {
    //             InstanceStateName::Pending => {
    //                 println!("Instance is pending, it will start soon. Instead of sending the power off command right now. I'll first wait until it's fully running.");
    //                 poll_until(InstanceStateName::Running, &client)
    //                     .await
    //                     .unwrap();
    //                 send_off_and_poll(&client).await;
    //             }
    //             InstanceStateName::Running => {
    //                 send_off_and_poll(&client).await;
    //             }
    //             InstanceStateName::ShuttingDown => {
    //                 println!("The server is already shutting down. I'll just poll until it's completely shut down.");
    //                 poll_until(InstanceStateName::Stopped, &client)
    //                     .await
    //                     .unwrap();
    //             }
    //             InstanceStateName::Stopped => {
    //                 println!("The server is already shutdown. Don't overkill it");
    //             }
    //             InstanceStateName::Stopping => {
    //                 println!("Instance is already stopping. I'll just poll until it's completely shut down.");
    //                 poll_until(InstanceStateName::Stopped, &client)
    //                     .await
    //                     .unwrap();
    //             }
    //             InstanceStateName::Terminated => {
    //                 panic!("there is no server anymore :ferrissweatballz:")
    //             }
    //             _ => {
    //                 panic!("Owo what's this")
    //             }
    //         }
    //     }
    //     Opt::Check => {
    //         let state = show_state(&client).await?;
    //         println!("Server is now {state:?}");
    //     }
    //     Opt::Reboot => {
    //         println!("Why don't you, uhh, hmm, do a off and then do a on?")
    //         // polling behaviour is so tricking on reboot :sob:
    //         // can't be too fast or too slow. So I'll just leave it here.
    //     }
    // }
    //
    Ok(())
}

// async fn send_on_and_poll(client: &Client) {
//     send_to_instance!(client, start_instances);
//     println!("I just sent the power on command OwO");
//     println!("Will poll until the machine is fully running!");
//     poll_until(InstanceStateName::Running, client)
//         .await
//         .unwrap();
// }
//
// async fn send_off_and_poll(client: &Client) {
//     send_to_instance!(client, stop_instances);
//     println!("I just sent the power off command OwO");
//     println!("Will poll until the machine is fully stopped!");
//     poll_until(InstanceStateName::Stopped, client)
//         .await
//         .unwrap();
// }
