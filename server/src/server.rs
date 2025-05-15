use actix::prelude::*;
use std::collections::HashMap;

use crate::{app_log, info_log, warning_log};
use crate::message::{Connect, SendMessage, ServerMessage};

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

/// GameServerアクターの実装
impl Actor for GameServer {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        info_log!("GameServer Actor started");
    }
}

/// 接続メッセージのハンドラー
impl Handler<Connect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) {
        // 既に同じユーザー名で接続があれば切断
        if let Some(old_session_id) = self.users.get(&msg.username) {
            if let Some((_, addr)) = self.sessions.get(old_session_id) {
                // 古いセッションに切断メッセージを送信
                let disconnect_msg = ServerMessage::Error {
                    message: "Your account has been logged in from another device or location. If this wasn't you, please secure your account immediately.".to_string(),
                };
                let msg_str = serde_json::to_string(&disconnect_msg).unwrap();
                addr.do_send(SendMessage {
                    message: msg_str,
                });
            }
            // 古いセッションをマップから削除
            self.sessions.remove(old_session_id);
        }

        // 新しいセッションを登録
        self.sessions.insert(
            msg.session_id.clone(),
            (msg.username.clone(), msg.addr),
        );
        self.users.insert(msg.username.clone(), msg.session_id.clone());

        info_log!(
            "New youser connected: {} (Session ID: {}), total connections: {}",
            msg.username,
            msg.session_id,
            self.sessions.len()
        );

        // 全ユーザーに新しいユーザーのログインを通知
        let notification = ServerMessage::Success {
            message: format!("User {} has logged in", msg.username),
        };
        self.broadcast_message(&notification, Some(&msg.session_id));
    }
}