use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Connecting,
    Login,
    Lobby,
    MatchSearch,
    InGame,
    GameOver,
}

#[derive(Resource)]
pub struct BoardState {
    pub board: Vec<Vec<Option<DiskColor>>>,
    pub current_player: DiskColor,
    pub your_color: Option<DiskColor>,
    pub valid_moves: Vec<(usize, usize)>,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiskColor {
    Black,
    White,
}

#[derive(Resource, Default)]
pub struct ConnectionState {
    pub connected: bool,
    pub username: Option<String>,
    pub opponent: Option<String>,
    pub in_matchmaking: bool,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "success")]
    Success { message: String },
    
    #[serde(rename = "error")]
    Error { message: String },
    
    #[serde(rename = "match_found")]
    MatchFound { opponent: String },
    
    #[serde(rename = "game_state")]
    GameState {
        board: Vec<Vec<Option<String>>>,
        current_player: String,
        your_color: String,
    },
    
    #[serde(rename = "game_over")]
    GameOver {
        winner: Option<String>,
        reason: String,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    #[serde(rename = "authenticate")]
    Authenticate { username: String },
    
    #[serde(rename = "join_queue")]
    JoinQueue,
    
    #[serde(rename = "leave_queue")]
    LeaveQueue,
    
    #[serde(rename = "make_move")]
    MakeMove { x: usize, y: usize },
    
    #[serde(rename = "resign")]
    Resign,
    
    #[serde(rename = "heartbeat")]
    Heartbeat,
}