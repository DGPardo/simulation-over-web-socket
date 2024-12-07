const ADDRESS: &str = "0.0.0.0:5000";

use futures_util::{SinkExt, StreamExt};
use std::{io::Error, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{unbounded_channel, UnboundedSender},
};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::{handler::handle_client_to_server_messages, state::ServerState};
use wasm_bindings::deserialize_client_msg;

pub async fn launch_ws_server(state: Arc<ServerState>) -> Result<(), Error> {
    println!("Starting WebSocket server at {}", ADDRESS);
    // await for a new connection over TCP
    // for each connection spawn a new task
    // the task will await for messages from the client

    let listener = TcpListener::bind(ADDRESS).await?;
    while let Ok((stream, socket)) = listener.accept().await {
        println!("Accepted connection from {:?}", socket);
        tokio::spawn(handle_connection(stream, Arc::clone(&state)));
    }
    Ok(())
}

async fn handle_connection(tcp_stream: TcpStream, state: Arc<ServerState>) -> Result<(), Error> {
    let connection = accept_async(tcp_stream)
        .await
        .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;

    let (mut to_client, mut from_client) = connection.split();
    let (tx, mut rx) = unbounded_channel();

    // This task listens for incoming messages from the client
    // and forwards them to the appropiate handler
    tokio::spawn(async move {
        loop {
            let msg = from_client.next().await;
            if let Some(Ok(msg)) = msg {
                handle_msg(msg, Arc::clone(&state), tx.clone()).await;
            }
        }
    });

    // This task replies to the client with the messages
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let _ = to_client.send(msg).await;
        }
    });

    Ok(())
}

async fn handle_msg(msg: Message, state: Arc<ServerState>, tx: UnboundedSender<Message>) {
    match msg {
        Message::Binary(data) => match deserialize_client_msg(&data) {
            Some(msg) => {
                handle_client_to_server_messages(msg, state, tx).await;
            }
            None => {
                match tx.send(Message::Text(
                    format!("Failed to parse message: {:?} ", data).into(),
                )) {
                    Ok(_) => {}
                    Err(_) => eprintln!("Failed to send invalid message response"),
                }
            }
        },
        _ => {
            eprintln!("Received invalid message: {:?}", msg);
            match tx.send(Message::Text(
                format!("Received invalid message: {:?}", msg).into(),
            )) {
                Ok(_) => {}
                Err(_) => println!("Failed to send invalid message response"),
            }
        }
    }
}
