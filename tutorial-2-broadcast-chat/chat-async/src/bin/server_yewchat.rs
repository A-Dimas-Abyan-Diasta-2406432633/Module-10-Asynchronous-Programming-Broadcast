use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    net::SocketAddr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

type SharedUsers = Arc<Mutex<HashMap<SocketAddr, ConnectedUser>>>;

#[derive(Clone)]
struct ConnectedUser {
    nick: String,
    tx: mpsc::UnboundedSender<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum MsgType {
    Users,
    Register,
    Message,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IncomingMessage {
    message_type: MsgType,
    data: Option<String>,
    data_array: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OutgoingMessage {
    message_type: MsgType,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data_array: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct ChatPayload {
    from: String,
    message: String,
    time: u128,
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

async fn broadcast_raw(users: &SharedUsers, payload: String) {
    let recipients: Vec<(SocketAddr, mpsc::UnboundedSender<String>)> = {
        let locked = users.lock().await;
        locked
            .iter()
            .map(|(addr, user)| (*addr, user.tx.clone()))
            .collect()
    };

    let mut disconnected = Vec::new();
    for (addr, tx) in recipients {
        if tx.send(payload.clone()).is_err() {
            disconnected.push(addr);
        }
    }

    if !disconnected.is_empty() {
        let mut locked = users.lock().await;
        for addr in disconnected {
            locked.remove(&addr);
        }
    }
}

async fn broadcast_user_list(users: &SharedUsers) -> Result<(), Box<dyn Error + Send + Sync>> {
    let names = {
        let locked = users.lock().await;
        locked.values().map(|u| u.nick.clone()).collect::<Vec<_>>()
    };

    let msg = OutgoingMessage {
        message_type: MsgType::Users,
        data: None,
        data_array: Some(names),
    };

    let payload = serde_json::to_string(&msg)?;
    broadcast_raw(users, payload).await;
    Ok(())
}

async fn handle_connection(
    addr: SocketAddr,
    ws_stream: WebSocketStream<TcpStream>,
    users: SharedUsers,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let (mut ws_write, mut ws_read) = ws_stream.split();

    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<String>();

    let writer = tokio::spawn(async move {
        while let Some(outbound) = out_rx.recv().await {
            if ws_write.send(Message::text(outbound)).await.is_err() {
                break;
            }
        }
    });

    while let Some(incoming) = ws_read.next().await {
        let msg = match incoming {
            Ok(m) => m,
            Err(err) => return Err(err.into()),
        };

        let Some(text) = msg.as_text() else {
            continue;
        };

        let parsed: IncomingMessage = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("invalid json from {addr}: {err}");
                continue;
            }
        };

        let _ = parsed.data_array.as_ref();

        match parsed.message_type {
            MsgType::Register => {
                if let Some(nick) = parsed.data {
                    let mut locked = users.lock().await;
                    locked.insert(
                        addr,
                        ConnectedUser {
                            nick,
                            tx: out_tx.clone(),
                        },
                    );
                    drop(locked);
                    broadcast_user_list(&users).await?;
                }
            }
            MsgType::Message => {
                let sender_name = {
                    let locked = users.lock().await;
                    locked.get(&addr).map(|u| u.nick.clone())
                };

                if let (Some(from), Some(message_text)) = (sender_name, parsed.data) {
                    let payload = ChatPayload {
                        from,
                        message: message_text,
                        time: now_millis(),
                    };
                    let wrapped = OutgoingMessage {
                        message_type: MsgType::Message,
                        data: Some(serde_json::to_string(&payload)?),
                        data_array: None,
                    };
                    broadcast_raw(&users, serde_json::to_string(&wrapped)?).await;
                }
            }
            MsgType::Users => {}
        }
    }

    {
        let mut locked = users.lock().await;
        locked.remove(&addr);
    }
    broadcast_user_list(&users).await?;

    writer.abort();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Rust YewChat websocket server listening on 127.0.0.1:8080");

    let users: SharedUsers = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("new websocket connection from {addr}");

        let users = Arc::clone(&users);
        tokio::spawn(async move {
            match ServerBuilder::new().accept(socket).await {
                Ok((_req, ws_stream)) => {
                    if let Err(err) = handle_connection(addr, ws_stream, users).await {
                        eprintln!("connection {addr} error: {err}");
                    }
                }
                Err(err) => eprintln!("handshake error from {addr}: {err}"),
            }
        });
    }
}
