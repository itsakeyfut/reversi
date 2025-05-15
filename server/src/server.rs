use actix::prelude::*;
use std::collections::HashMap;

use crate::{app_log, info_log, warning_log};
use crate::message::{SendMessage, ServerMessage};

/// ゲームサーバーアクター - 全セッションとゲーム状態を管理
pub struct GameServer {
    /// セッションID -> (ユーザー名, アクターアドレス)のマッピング
    sessions: HashMap<String, (String, actix::Addr<crate::session::WsGameSession>)>,

    /// ユーザー名 -> セッションIDのマッピング
    users: HashMap<String, String>,
}

impl GameServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            users: HashMap::new(),
        }
    }

    /// 全クライアントにメッセージをブロードキャスト
    fn broadcast_message(&self, message: &ServerMessage, skip_id: Option<&str>) {
        let msg = serde_json::to_string(message).unwrap();

        for (id, (_, addr)) in &self.sessions {
            if let Some(skip) = skip_id {
                if id == skip {
                    continue;
                }
            }
            addr.do_send(SendMessage {
                message: msg.clone(),
            });
        }
    }

    /// 特定のクライアントにメッセージを送信
    fn send_message_to_session(&self, session_id: &str, message: &ServerMessage) {
        if let Some((_, addr)) = self.sessions.get(session_id) {
            let msg = serde_json::to_string(message).unwrap();
            addr.do_send(SendMessage {
                message: msg,
            });
        }
    }
}