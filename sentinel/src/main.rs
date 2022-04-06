use tokio::sync::watch::{Receiver, Sender};

use aws_sdk_ec2::model::InstanceStateName;
use aws_sdk_ec2::{Client, Error as Ec2Error};
use axum::handler::Handler;
use axum::{extract::{
    ws::{Message, WebSocket, WebSocketUpgrade},
    TypedHeader,
}, headers, http::StatusCode, response::IntoResponse, routing::{get, get_service, post}, Router, Extension};
use std::env;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::time::Duration;

use common::{ClientOpt, ContainerStatus, OnlinePeople, ServerStatus};
use tower_http::auth::RequireAuthorizationLayer;


macro_rules! send_to_instance {
    ($client:ident, $id:ident) => {
        $client
            .$id()
            .set_instance_ids(Some(vec!["i-04b01cb5264a5c895".to_string()]))
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
    Extension(rx): Extension<Receiver<Option<ServerStatus>>>
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        println!("`{}` connected", user_agent.as_str());
    }

    ws.on_upgrade(|socket: WebSocket| handle_socket(socket, rx))
}
async fn handle_socket(mut socket: WebSocket, rx: Receiver<Option<ServerStatus>>) {


    let config = bincode::config::standard();

    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(_) => {
                    // println!("client send str: {:?}", t);
                }
                Message::Binary(d) => {
                    if let Ok((decoded, _)) = bincode::decode_from_slice::<ClientOpt, _>(d.as_slice(), config){


                    }
                }
                Message::Ping(_) => {
                    println!("socket ping");
                }
                Message::Pong(_) => {
                    println!("socket pong");
                }
                Message::Close(_) => {
                    println!("client disconnected");
                    return;
                }
            }
        } else {
            println!("client disconnected");
            return;
        }
    }

    loop {
        if socket
            .send(Message::Text(String::from("Hi!")))
            .await
            .is_err()
        {
            println!("client disconnected");
            return;
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

async fn poll_server_status(client: &Client) -> ServerStatus {
    let host = show_state(&client)
        .await
        .map_or("failed to get instance state".to_string(), |n| n.as_str().to_string());

    // todo:
    let container = ContainerStatus::Unknown;

    // todo:
    let online = OnlinePeople::Unknown;

    ServerStatus {
        host,
        container,
        online,
    }
}

#[tokio::main]
async fn main() -> Result<(), Ec2Error> {
    dotenv::dotenv().ok();

    let auth = RequireAuthorizationLayer::bearer("elfiscute");

    let (tx, rx) = tokio::sync::watch::channel::<Option<ServerStatus>>(None);

    // todo: handle log streaming. Can't be done with watch because no log must be lost.

    let endless_poll = async {
        // ap-east-1 is Hong Kong
        let shared_config = aws_config::from_env().region("ap-east-1").load().await;

        let client = Client::new(&shared_config);

        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            let status = poll_server_status(&client).await;
            tx.send(Some(status));
        }
    };

    let ws_router = Router::new()
        .route("/", get(ws_handler))
        .layer(Extension(rx));


    let app = Router::new()
        .nest("/ws", ws_router)
        .route("/auth", post(|| async { StatusCode::OK }))
        .layer(auth);

    // run it with hyper on localhost:3000
    let server =
        axum::Server::bind(&"0.0.0.0:3000".parse().unwrap()).serve(app.into_make_service());

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
