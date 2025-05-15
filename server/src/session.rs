use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::message::{ClientMessage, Connect, Disconnect};
use crate::{app_log, debug_log, error_log, info_log, warning_log};
use crate::{message::ServerMessage, server::GameServer};

/// ハートビートインターバル - クライアント接続を維持するための定期的な確認
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// クライアントがタイムアウトする前の時間
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// WebSocketゲームセッション - 各クライアント接続に対するアクター
pub struct WsGameSession {
    /// ユニークセッションID
    pub id: String,

    /// GamerServerアクターへの参照（接続時に設定）
    pub server_addr: Option<actix::Addr<GameServer>>,

    /// 最後にハードビートを受け取った時刻
    pub last_heartbeat: Instant,

    /// ユーザー名（認証後に設定）
    pub username: Option<String>,

    /// ユーザーの現在の状態
    pub status: UserStatus,
}

/// ユーザーの状態を表す列挙型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserStatus {
    Connecting,
    Online,
    Idle,
    SearchingMatch,
    InGame,
    Spectating,
    Offline,
}

impl WsGameSession {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            server_addr: None,
            last_heartbeat: Instant::now(),
            username: None,
            status: UserStatus::Connecting,
        }
    }

    /// ハートビートを送信し、クライアント接続を維持
    fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // 最後のハートビートから経過した時間をチェック
            if Instant::now().duration_since(act.last_heartbeat) > CLIENT_TIMEOUT {
                warning_log!("WebSocket client is heartbeat timeout: {}", act.id);

                // GameServerに切断を通知
                if let Some(addr) = &act.server_addr {
                    addr.do_send(Disconnect {
                        session_id: act.id.clone(),
                    });
                }

                // セッションを終了
                ctx.stop();
                return;
            }

            // ハートビートメッセージを送信
            ctx.ping(b"");
        });
    }

    /// WebSocketに成功レスポンスを送信
    fn send_success(&self, ctx: &mut ws::WebsocketContext<Self>, message: &str) {
        let response = ServerMessage::Success {
            message: message.to_string(),
        };
        let msg = serde_json::to_string(&response).map_err(|e| {
            error_log!("Couldn't create a success message: {}", e);
        });

        match msg {
            Ok(msg) => ctx.text(msg),
            Err(_) => return,
        }
    }

    /// WebSocketにエラーレスポンスを送信
    fn send_error(&self, ctx: &mut ws::WebsocketContext<Self>, message: &str) {
        let response = ServerMessage::Error {
            message: message.to_string(),
        };
        let msg = serde_json::to_string(&response).map_err(|e| {
            error_log!("Couldn't create a error message: {}", e);
        });

        match msg {
            Ok(msg) => ctx.text(msg),
            Err(_) => return,
        }
    }
}

/// WebSocketアクター
impl Actor for WsGameSession {
    type Context = ws::WebsocketContext<Self>;

    /// セッション開始時に呼び出される
    fn started(&mut self, ctx: &mut Self::Context) {
        // ハートビート処理を開始
        self.heartbeat(ctx);

        info_log!("Start new ws session: {}", self.id);

        // GameServerアクターのアドレスを取得
        let server_addr = ctx.address();
        self.server_addr = Some(server_addr);

        // ハンドシェイク完了メッセージをクライアントに送信
        self.send_success(ctx, "Connected Successfully. Authentication is required.");
    }

    /// セッション終了時に呼び出される
    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        info_log!("ws session is terminated: {}", self.id);

        // GameServerに切断を通知
        if let Some(addr) = &self.server_addr {
            addr.do_send(Disconnect {
                session_id: self.id.clone(),
            });
        }

        actix::Running::Stop
    }
}

/// WebSocketメッセージを処理するハンドラー
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsGameSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                // Pingに応答
                self.last_heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                // Pongメッセージを受信
                self.last_heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                // テキストメッセージを処理
                debug_log!("Received text message: {:?}", text);

                // JSONメッセージをパース
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        match client_msg {
                            ClientMessage::Authenticate { username } => {
                                info_log!("Authenticated successfully: {}", username);
                                // TODO: 認証処理を追加

                                self.username = Some(username.clone());
                                self.status = UserStatus::Idle;

                                // 認証メッセージを送信
                                self.send_success(
                                    ctx,
                                    &format!("Authenticated successfully. Hello {}!", username),
                                );

                                // GameServerに接続を通知
                                if let Some(addr) = &self.server_addr {
                                    addr.do_send(Connect {
                                        session_id: self.id.clone(),
                                        username,
                                        addr: ctx.address(),
                                    });
                                }
                            }
                            // TODO: 他のメッセージタイプをマッチガードに含める
                            _ => {
                                warning_log!("Received an unknown message type: {:?}", client_msg);
                                self.send_error(ctx, "This message type is not supported.");
                            }
                        }
                    }
                    Err(e) => {
                        // JSONパースエラー
                        warning_log!("Received invalid JSON message: {}", e);
                        self.send_error(ctx, "Invalid format received. JSON required.");
                    }
                }
            }
            Ok(ws::Message::Binary(_)) => {
                warning_log!("Binary message not supported");
                self.send_error(
                    ctx,
                    "Binary message is not supported. Use text message instead.",
                );
            }
            Ok(ws::Message::Close(reason)) => {
                // WebSocket切断
                info_log!("Received signal of ws disconnected: {:?}", reason);
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}
