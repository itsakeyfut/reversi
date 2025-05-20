use bevy::prelude::*;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio::sync::mpsc::{channel, Sender, Receiver};
use anyhow::Result;

use crate::{game_logic::calculate_valid_moves, state::{BoardState, ClientMessage, ConnectionState, DiskColor, GameState, ServerMessage}};

#[derive(Resource)]
pub struct WebSocketClient {
    pub message_tx: Option<Sender<String>>,
    pub message_rx: Option<Receiver<ServerMessage>>
}

impl Default for WebSocketClient {
    fn default() -> Self {
        Self {
            message_tx: None,
            message_rx: None,
        }
    }
}

pub fn init_websocket(mut commands: Commands) {
    let (client_tx, mut client_rx) = channel::<String>(100);
    let (server_tx, server_rx) = channel::<ServerMessage>(100);

    commands.insert_resource(WebSocketClient {
        message_tx: Some(client_tx),
        message_rx: Some(server_rx),
    });

    tokio::spawn(async move {
        let url = "ws://localhost:8080/ws";
        match connect_websocket(url, &mut client_rx, server_tx).await {
            Ok(_) => println!("WebSocket connection closed"),
            Err(e) => eprintln!("WebSocket error: {}", e),
        }
    });
}


async fn connect_websocket(
    url: &str,
    client_rx: &mut Receiver<String>,
    server_tx: Sender<ServerMessage>,
) -> Result<()> {
    // WebSocketに接続
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    // 受信ハンドラー
    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            if let Ok(msg) = msg {
                if let Ok(text) = msg.into_text() {
                    match serde_json::from_str::<ServerMessage>(&text) {
                        Ok(server_msg) => {
                            if let Err(e) = server_tx.send(server_msg).await {
                                eprintln!("Failed to send server message: {}", e);
                                break;
                            }
                        }
                        Err(e) => eprintln!("Failed to parse server message: {}", e),
                    }
                }
            } else {
                break;
            }
        }
    });

    // 送信ハンドラー
    while let Some(msg) = client_rx.recv().await {
        if let Err(e) = write.send(Message::Text(msg.into())).await {
            eprintln!("Failed to send message: {}", e);
            break;
        }
    }

    Ok(())
}

pub fn send_message(client: &WebSocketClient, message: ClientMessage) -> Result<()> {
    if let Some(tx) = &client.message_tx {
        let json = serde_json::to_string(&message)?;
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            if let Err(e) = tx_clone.send(json).await {
                eprintln!("Failed to send message: {}", e);
            }
        });
    }

    Ok(())
}

pub fn handle_server_messages(
    mut connection: ResMut<ConnectionState>,
    mut client: ResMut<WebSocketClient>, // Resから ResMut<WebSocketClient> に変更
    mut next_state: ResMut<NextState<GameState>>,
    mut board_state: ResMut<BoardState>,
    mut ev_game_events: EventWriter<GameEvent>
) {
    // rxを取り出し、新しいものと交換する方法
    if let Some(mut rx) = client.message_rx.take() {
        // ブロッキングしないでメッセージを処理
        while let Ok(message) = rx.try_recv() {
            match message {
                ServerMessage::Success { message } => {
                    println!("Success: {}", message);

                    // マッチングキューに参加/離脱のレスポンスを処理
                    if message.contains("Joined matchmaking queue") {
                        connection.in_matchmaking = true;
                        next_state.set(GameState::MatchSearch);
                    } else if message.contains("Left matchmaking queue") {
                        connection.in_matchmaking = false;
                        next_state.set(GameState::Lobby);
                    }
                }
                ServerMessage::Error { message } => {
                    eprintln!("Error: {}", message);
                }
                ServerMessage::MatchFound { opponent } => {
                    println!("Match found with: {}", opponent);
                    connection.opponent = Some(opponent.clone());
                    connection.in_matchmaking = false;
                    next_state.set(GameState::InGame);
                }
                ServerMessage::GameState { board, current_player, your_color } => {
                    update_board_state(&mut board_state, board, current_player, your_color);
                }
                ServerMessage::GameOver { winner, reason } => {
                    println!("Game over. Reason: {}", reason);
                    if let Some(ref winner_name) = winner {
                        println!("Winner: {}", winner_name);
                    } else {
                        println!("Game ended in a draw.");
                    }

                    ev_game_events.send(GameEvent::GameOver {
                        winner,
                        reason,
                    });

                    next_state.set(GameState::GameOver);
                }
            }
        }
        
        // 処理が終わったらrxを戻す
        client.message_rx = Some(rx);
    }
}

fn update_board_state(
    board_state: &mut BoardState,
    server_board: Vec<Vec<Option<String>>>,
    current_player: String,
    your_color: String,
) {
    board_state.board = server_board
        .iter()
        .map(|row| {
            row.iter()
                .map(|cell| match cell {
                    Some(color) => match color.as_str() {
                        "black" => Some(DiskColor::Black),
                        "white" => Some(DiskColor::White),
                        _ => None,
                    },
                    None => None,
                })
                .collect()
        })
        .collect();

    board_state.current_player = match current_player.as_str() {
        "black" => DiskColor::Black,
        "white" => DiskColor::White,
        _ => DiskColor::Black, // Default value
    };

    board_state.your_color = match your_color.as_str() {
        "black" => Some(DiskColor::Black),
        "white" => Some(DiskColor::White),
        _ => None,
    };

    board_state.valid_moves = calculate_valid_moves(&board_state.board, &board_state.current_player);
}

#[derive(Event)]
pub enum GameEvent {
    GameOver {
        winner: Option<String>,
        reason: String,
    },
}