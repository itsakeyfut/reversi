use actix::prelude::*;
use serde::{Deserialize, Serialize};

use crate::session::WsGameSession;

/// クライアントからのメッセージを表す列挙型
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    /// 認証リクエスト
    #[serde(rename = "authenticate")]
    Authenticate { username: String },

    /// マッチングキューに参加
    #[serde(rename = "join_queue")]
    JoinQueue,

    /// マッチングキューから離脱
    #[serde(rename = "leave_queue")]
    LeaveQueue,

    /// ゲームの手を打つ
    #[serde(rename = "make_move")]
    MakeMove { x: usize, y: usize },

    /// ゲームを降参する
    #[serde(rename = "resign")]
    Resign,

    /// ハートビート
    #[serde(rename = "heartbeat")]
    Heartbeat,
}

/// サーバーからクライアントへのメッセージを表す列挙型
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// 成功メッセージ
    #[serde(rename = "success")]
    Success { message: String },

    /// エラーメッセージ
    #[serde(rename = "error")]
    Error { message: String },

    /// マッチが見つかった通知
    #[serde(rename = "match_found")]
    MatchFound { opponent: String },

    /// ゲーム状態の更新
    #[serde(rename = "game_state")]
    GameState {
        board: Vec<Vec<Option<String>>>,
        current_player: String,
        your_color: String,
    },

    /// ゲーム終了通知
    #[serde(rename = "game_over")]
    GameOver {
        winner: Option<String>,
        reason: String,
    },
}

/// WebSocketセッションの接続メッセージ
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub session_id: String,
    pub username: String,
    pub addr: Addr<WsGameSession>,
}

/// WebSocketセッションの切断メッセージ
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub session_id: String,
}

/// クライアントへのメッセージ送信リクエスト
#[derive(Message)]
#[rtype(result = "()")]
pub struct SendMessage {
    pub message: String,
}

/// ゲームサーバーからのセッションへのメッセージ
impl Handler<SendMessage> for WsGameSession {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, ctx: &mut Self::Context) {
        // テキストメッセージをWebSocketに送信
        ctx.text(msg.message);
    }
}
