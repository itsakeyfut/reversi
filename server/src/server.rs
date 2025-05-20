use actix::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

use crate::game::{Game, GameState};
use crate::matchmaking::MatchmakingService;
use crate::message::{ClientMessage, Connect, Disconnect, SendMessage, ServerMessage};
use crate::session::UserStatus;
use crate::{app_log, debug_log, info_log, warning_log};

// マッチングのタイムアウト（秒）
const MATCH_TIMEOUT: Duration = Duration::from_secs(30);
// マッチングメイキングの実行間隔（ミリ秒）
const MATCHMAKING_INTERVAL: Duration = Duration::from_millis(1000);
// デフォルトレーティング
const DEFAULT_RATING: u32 = 1000;

/// ゲームサーバーアクター - 全セッションとゲーム状態を管理
pub struct GameServer {
    /// セッションID -> (ユーザー名, アクターアドレス)のマッピング
    sessions: HashMap<String, (String, actix::Addr<crate::session::WsGameSession>)>,
    /// ユーザー名 -> セッションIDのマッピング
    users: HashMap<String, String>,
    /// ユーザーID -> セッションIDのマッピング
    user_statuses: HashMap<String, UserStatus>,
    /// マッチングメイキングサービス
    matchmaking: MatchmakingService,
    /// アクティブなゲーム (ゲームID -> ゲーム)
    active_games: HashMap<String, Game>,
    /// ユーザーID -> ゲームIDのマッピング
    user_games: HashMap<String, String>,
}

impl GameServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            users: HashMap::new(),
            user_statuses: HashMap::new(),
            matchmaking: MatchmakingService::new(),
            active_games: HashMap::new(),
            user_games: HashMap::new(),
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
            addr.do_send(SendMessage { message: msg });
        }
    }

    /// ユーザーステータスを更新
    fn update_user_status(&mut self, user_id: &str, status: UserStatus) {
        self.user_statuses
            .insert(user_id.to_string(), status.clone());
        debug_log!("Updated user status: {} -> {:?}", user_id, status);
    }

    /// マッチングメイキングキューにユーザーを追加
    fn add_user_to_matchmaking(&mut self, session_id: &str) -> Result<(), &'static str> {
        if let Some((username, _)) = self.sessions.get(session_id) {
            // ユーザーがキューに追加された場合はステータスを更新
            if self.matchmaking.add_to_queue(
                session_id.to_string(),
                username.clone(),
                DEFAULT_RATING,
            ) {
                self.update_user_status(session_id, UserStatus::SearchingMatch);

                let msg = ServerMessage::Success {
                    message: "Joined matchmaking queue. Searching for opponent...".to_string(),
                };
                self.send_message_to_session(session_id, &msg);

                Ok(())
            } else {
                Err("Already in matchmaking queue")
            }
        } else {
            Err("User session not found")
        }
    }

    /// マッチングメイキングキューからユーザーを削除
    fn remove_user_from_matchmaking(&mut self, session_id: &str) -> Result<(), &'static str> {
        if self.matchmaking.remove_from_queue(session_id) {
            self.update_user_status(session_id, UserStatus::Idle);

            let msg = ServerMessage::Success {
                message: "Left matchmaking queue".to_string(),
            };
            self.send_message_to_session(session_id, &msg);

            Ok(())
        } else {
            Err("Not in matchmaking queue")
        }
    }

    /// マッチングメイキングの実行
    fn run_matchmaking(&mut self) {
        // マッチを探す
        let created_matches = self.matchmaking.find_matches();

        for created_match in created_matches {
            info_log!(
                "Match found: {} vs. {}",
                created_match.player1_name,
                created_match.player2_name
            );

            // 新しいゲームを作成
            let game_id = created_match.match_id.clone();
            let game = Game::new(
                game_id.clone(),
                created_match.player1_id.clone(),
                created_match.player1_name.clone(),
                created_match.player2_id.clone(),
                created_match.player2_name.clone(),
            );

            // ゲームの初期状態を取得
            let game_state = game.get_state();

            // ゲームをアクティブゲームマップに登録
            self.active_games.insert(game_id.clone(), game);

            // プレイヤーをゲームに関連付け
            self.user_games
                .insert(created_match.player1_id.clone(), game_id.clone());
            self.user_games
                .insert(created_match.player2_id.clone(), game_id.clone());

            // プレイヤー1にマッチが見つかったことを通知
            let msg1 = ServerMessage::MatchFound {
                opponent: created_match.player2_name.clone(),
            };
            self.send_message_to_session(&created_match.player1_id, &msg1);
            self.update_user_status(&created_match.player1_id, UserStatus::InGame);

            // プレイヤー1に初期ゲーム状態を送信
            let black_state = ServerMessage::GameState {
                board: game_state.board.clone(),
                current_player: game_state.current_player.clone(),
                your_color: "black".to_string(),
            };
            self.send_message_to_session(&created_match.player1_id, &black_state);

            // プレイヤー2にマッチが見つかったことを通知
            let msg2 = ServerMessage::MatchFound {
                opponent: created_match.player1_name.clone(),
            };
            self.send_message_to_session(&created_match.player2_id, &msg2);
            self.update_user_status(&created_match.player2_id, UserStatus::InGame);

            // プレイヤー2に初期ゲーム状態を送信
            let white_state = ServerMessage::GameState {
                board: game_state.board.clone(),
                current_player: game_state.current_player.clone(),
                your_color: "white".to_string(),
            };
            self.send_message_to_session(&created_match.player2_id, &white_state);
        }

        // タイムアウトしたマッチをクリーンアップ
        let expired_matches = self.matchmaking.cleanup_pending_matches(MATCH_TIMEOUT);

        // タイムアウトしたマッチを処理
        for expired_match in expired_matches {
            warning_log!(
                "Match timed out: {} vs. {}",
                expired_match.player1_name,
                expired_match.player2_name
            );

            // プレイヤー1に通知
            let msg1 = ServerMessage::Error {
                message: "Match timed out. Please join the queue again.".to_string(),
            };
            self.send_message_to_session(&expired_match.player1_id, &msg1);
            self.update_user_status(&expired_match.player1_id, UserStatus::Idle);

            // プレイヤー2に通知
            let msg2 = ServerMessage::Error {
                message: "Match timed out. Please join the queue again.".to_string(),
            };
            self.send_message_to_session(&expired_match.player2_id, &msg2);
            self.update_user_status(&expired_match.player2_id, UserStatus::Idle);
        }
    }

    /// ゲームの手を処理する
    fn handle_game_move(&mut self, session_id: &str, x: usize, y: usize) -> Result<(), String> {
        // プレイヤーがゲームに参加しているか確認
        let game_id = match self.user_games.get(session_id) {
            Some(id) => id.clone(),
            None => return Err("You are not in a game".to_string()),
        };

        // ゲームが存在するか確認
        let game = match self.active_games.get_mut(&game_id) {
            Some(game) => game,
            None => return Err("Game not found".to_string()),
        };

        // 手を打つ
        match game.make_move(session_id, x, y) {
            Ok(game_state) => {
                // 両プレイヤーにゲーム状態を送信
                self.send_game_state_to_players(&game_id, &game_state);

                // ゲームが終了したか確認
                if game_state.is_game_over {
                    self.handle_game_over(&game_id, &game_state);
                }

                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// ゲーム降参を処理する
    fn handle_resign(&mut self, session_id: &str) -> Result<(), String> {
        // プレイヤーがゲームに参加しているか確認
        let game_id = match self.user_games.get(session_id) {
            Some(id) => id.clone(),
            None => return Err("You are not in a game".to_string()),
        };

        // ゲームが存在するか確認
        let game = match self.active_games.get_mut(&game_id) {
            Some(game) => game,
            None => return Err("Game not found".to_string()),
        };

        // 降参処理
        match game.resign(session_id) {
            Ok(game_state) => {
                // 両プレイヤーにゲーム状態を送信
                self.send_game_state_to_players(&game_id, &game_state);

                // ゲーム終了処理
                self.handle_game_over(&game_id, &game_state);

                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// ゲーム状態を両プレイヤーに送信
    fn send_game_state_to_players(&self, game_id: &str, game_state: &GameState) {
        if let Some(game) = self.active_games.get(game_id) {
            // 黒プレイヤーにゲーム状態を送信
            let black_state = ServerMessage::GameState {
                board: game_state.board.clone(),
                current_player: game_state.current_player.clone(),
                your_color: "black".to_string(),
            };
            self.send_message_to_session(&game.black_player_id, &black_state);

            // 白プレイヤーにゲーム状態を送信
            let white_state = ServerMessage::GameState {
                board: game_state.board.clone(),
                current_player: game_state.current_player.clone(),
                your_color: "white".to_string(),
            };
            self.send_message_to_session(&game.white_player_id, &white_state);
        }
    }

    /// ゲーム終了処理
    fn handle_game_over(&mut self, game_id: &str, game_state: &GameState) {
        // 必要な情報を先に取り出す
        let (black_player_id, white_player_id, _, _, winner_name) =
            if let Some(game) = self.active_games.get(game_id) {
                // 勝者を特定
                let winner_name = match &game_state.winner {
                    Some(player_id) => {
                        if player_id == &game.black_player_id {
                            Some(game.black_player_name.clone())
                        } else {
                            Some(game.white_player_name.clone())
                        }
                    }
                    None => None, // 引き分け
                };

                (
                    game.black_player_id.clone(),
                    game.white_player_id.clone(),
                    game.black_player_name.clone(),
                    game.white_player_name.clone(),
                    winner_name,
                )
            } else {
                // ゲームが見つからない場合は何もしない
                return;
            };

        info_log!("Game over: {}", game_id);

        // 両プレイヤーにゲーム終了を通知
        let game_over = ServerMessage::GameOver {
            winner: winner_name,
            reason: "Game completed".to_string(),
        };

        self.send_message_to_session(&black_player_id, &game_over);
        self.send_message_to_session(&white_player_id, &game_over);

        // プレイヤーのステータスを更新
        self.update_user_status(&black_player_id, UserStatus::Idle);
        self.update_user_status(&white_player_id, UserStatus::Idle);

        // ゲームの関連付けを解除
        self.user_games.remove(&black_player_id);
        self.user_games.remove(&white_player_id);

        // ゲームをアクティブリストから削除
        self.active_games.remove(game_id);
    }
}

/// マッチメイキングの定期実行
struct RunMatchmaking;

impl Message for RunMatchmaking {
    type Result = ();
}

/// GameServerアクターの実装
impl Actor for GameServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info_log!("GameServer Actor started");

        // 定期的にマッチメイキングを実行
        ctx.run_interval(MATCHMAKING_INTERVAL, |_, ctx| {
            ctx.address().do_send(RunMatchmaking);
        });
    }
}

/// 定期マッチメイキング処理
impl Handler<RunMatchmaking> for GameServer {
    type Result = ();

    fn handle(&mut self, _: RunMatchmaking, _: &mut Self::Context) {
        self.run_matchmaking();
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessageWrapper {
    pub session_id: String,
    pub message: ClientMessage,
}

/// クライアントメッセージハンドラー
impl Handler<ClientMessageWrapper> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessageWrapper, _: &mut Self::Context) {
        match msg.message {
            ClientMessage::JoinQueue => {
                if let Err(e) = self.add_user_to_matchmaking(&msg.session_id) {
                    let error_msg = ServerMessage::Error {
                        message: e.to_string(),
                    };
                    self.send_message_to_session(&msg.session_id, &error_msg);
                }
            }
            ClientMessage::LeaveQueue => {
                if let Err(e) = self.remove_user_from_matchmaking(&msg.session_id) {
                    let error_msg = ServerMessage::Error {
                        message: e.to_string(),
                    };
                    self.send_message_to_session(&msg.session_id, &error_msg);
                }
            }
            ClientMessage::MakeMove { x, y } => {
                match self.handle_game_move(&msg.session_id, x, y) {
                    Ok(_) => {}
                    Err(e) => {
                        let error_msg = ServerMessage::Error { message: e };
                        self.send_message_to_session(&msg.session_id, &error_msg);
                    }
                }
            }
            ClientMessage::Resign => match self.handle_resign(&msg.session_id) {
                Ok(_) => {}
                Err(e) => {
                    let error_msg = ServerMessage::Error { message: e };
                    self.send_message_to_session(&msg.session_id, &error_msg);
                }
            },
            ClientMessage::Heartbeat => {
                // ハートビートは無視
            }
            ClientMessage::Authenticate { username: _ } => {
                // 認証はWsGameSessionで処理済み
            }
        }
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
                    message: "Your account has been logged in from another device or location.
                            If this wasn't you, please secure your account immediately."
                        .to_string(),
                };
                let msg_str = serde_json::to_string(&disconnect_msg).unwrap();
                addr.do_send(SendMessage { message: msg_str });
            }
            // 古いセッションをマップから削除
            self.sessions.remove(old_session_id);
        }

        // 新しいセッションを登録
        self.sessions
            .insert(msg.session_id.clone(), (msg.username.clone(), msg.addr));
        self.users
            .insert(msg.username.clone(), msg.session_id.clone());

        info_log!(
            "New user connected: {} (Session ID: {}), total connections: {}",
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

/// 切断メッセージのハンドラー
impl Handler<Disconnect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        if let Some((username, _)) = self.sessions.remove(&msg.session_id) {
            self.users.remove(&username);
            self.user_statuses.remove(&msg.session_id);

            // マッチングキューからも削除
            self.matchmaking.remove_from_queue(&msg.session_id);

            // プレイヤーがゲームに参加していた場合は処理
            if let Some(game_id) = self.user_games.get(&msg.session_id).cloned() {
                if let Some(game) = self.active_games.get(&game_id) {
                    // 相手プレイヤーを特定
                    let opponent_id = if msg.session_id == game.black_player_id {
                        game.white_player_id.clone()
                    } else {
                        game.black_player_id.clone()
                    };

                    let winner_name = if msg.session_id == game.black_player_id {
                        game.white_player_name.clone()
                    } else {
                        game.black_player_name.clone()
                    };

                    // 相手プレイヤーにゲーム終了を通知
                    let game_over = ServerMessage::GameOver {
                        winner: Some(winner_name),
                        reason: "Opponent disconnected".to_string(),
                    };

                    self.send_message_to_session(&opponent_id, &game_over);

                    // 相手プレイヤーのステータスを更新
                    self.update_user_status(&opponent_id, UserStatus::Idle);

                    // ゲームの関連付けを解除
                    self.user_games.remove(&opponent_id);
                    self.user_games.remove(&msg.session_id);

                    // ゲームをアクティブリストから削除
                    self.active_games.remove(&game_id);
                } else {
                    // ゲームが見つからなかった場合は単に関連付けを解除
                    self.user_games.remove(&msg.session_id);
                }
            }

            info_log!(
                "User disconnected: {} (Session ID: {}), total connections: {}",
                username,
                msg.session_id,
                self.sessions.len(),
            );

            // 全ユーザーにユーザーのログアウトを通知
            let notification = ServerMessage::Success {
                message: format!("User {} has logged out", username),
            };
            self.broadcast_message(&notification, None);
        }
    }
}
