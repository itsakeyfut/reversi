use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::{app_log, info_log, warning_log, debug_log};

/// マッチングキューのエントリー
#[derive(Debug, Clone)]
pub struct QueueEntry {
    /// ユーザーID
    pub user_id: String,
    /// ユーザー名
    pub username: String,
    /// レーティング
    pub rating: u32,
    /// キューに入った時刻
    pub joined_at: Instant,
}

/// 保留中のマッチ情報
#[derive(Debug, Clone)]
pub struct PendingMatch {
    /// マッチID
    pub match_id: String,
    /// プレイヤー1のID
    pub player1_id: String,
    /// プレイヤー1の名前
    pub player1_name: String,
    /// プレイヤー2のID
    pub player2_id: String,
    /// プレイヤー2の名前
    pub player2_name: String,
    /// マッチが作成された時刻
    pub created_at: Instant,
    /// プレイヤー1の準備完了フラグ
    pub player1_ready: bool,
    /// プレイヤー2の準備完了フラグ
    pub player2_ready: bool,
}

/// マッチングメイキングサービス
pub struct MatchmakingService {
    /// マッチングキュー（プレイヤーID -> キューエントリー）
    queue: HashMap<String, QueueEntry>,
    /// マッチングリクエストの順番を保持するキュー
    queue_order: VecDeque<String>,
    /// 保留中のマッチ（マッチID -> マッチ情報）
    pending_matches: HashMap<String, PendingMatch>,
}

impl MatchmakingService {
    pub fn new() -> Self {
        Self {
            queue: HashMap::new(),
            queue_order: VecDeque::new(),
            pending_matches: HashMap::new(),
        }
    }

    /// マッチングキューにプレイヤーを追加
    pub fn add_to_queue(&mut self, user_id: String, username: String, rating: u32) -> bool {
        // 既にキューにいる場合は追加しない
        if self.queue.contains_key(&user_id) {
            return false;
        }

        // キューエントリを作成
        let entry = QueueEntry {
            user_id: user_id.clone(),
            username,
            rating,
            joined_at: Instant::now(),
        };

        // キューに追加
        self.queue.insert(user_id.clone(), entry);
        self.queue_order.push_back(user_id);

        info_log!("Player added to matchmaking queue. Players in queue: {}", self.queue.len());
        true
    }

    /// マッチングキューからプレイヤーを削除
    pub fn remove_from_queue(&mut self, user_id: &str) -> bool {
        if self.queue.remove(user_id).is_some() {
            self.queue_order.retain(|id| id != user_id);
            info_log!("Player removed from matchmaking queue. Players in queue: {}", self.queue.len());
            true
        } else {
            false
        }
    }

    /// マッチを探す
    /// FIFOマッチメイキングだが、使用感によっては変更したい
    pub fn find_matches(&mut self) -> Vec<PendingMatch> {
        let mut created_matches = Vec::new();

        // キュー内のプレイヤーが2人以上いる場合、マッチングを試みる
        while self.queue.len() >= 2 {
            // キューから最も古い2つのエントリを取得
            if let Some(player1_id) = self.queue_order.pop_front() {
                if let Some(player2_id) = self.queue_order.pop_front() {
                    // プレイヤー情報を取得
                    let player1 = self.queue.remove(&player1_id).unwrap();
                    let player2 = self.queue.remove(&player2_id).unwrap();

                    // マッチを作成
                    let match_id = Uuid::new_v4().to_string();
                    let new_match = PendingMatch {
                        match_id: match_id.clone(),
                        player1_id: player1.user_id,
                        player1_name: player1.username,
                        player2_id: player2.user_id,
                        player2_name: player2.username,
                        created_at: Instant::now(),
                        player1_ready: false,
                        player2_ready: false,
                    };

                    // 保留中のマッチに参加
                    self.pending_matches.insert(match_id.clone(), new_match.clone());
                    created_matches.push(new_match);

                    info_log!("New match created: {}", match_id);
                } else {
                    // Player2がいない場合はPlayer1をキューに戻す
                    self.queue_order.push_front(player1_id);
                    break;
                }
            } else {
                break;
            }
        }

        created_matches
    }

    /// プレイヤーの準備完了を設定
    pub fn set_player_ready(&mut self, match_id: &str, user_id: &str) -> Option<bool> {
        if let Some(pending_match) = self.pending_matches.get_mut(match_id) {
            if pending_match.player1_id == user_id {
                pending_match.player1_ready = true;
                debug_log!("Player 1 is ready: {}", user_id);
            } else if pending_match.player2_id == user_id {
                pending_match.player2_ready = true;
                debug_log!("Player 2 is ready: {}", user_id);
            } else {
                return None;
            }

            // 両方のプレイヤーが準備完了しているかチェック
            let both_ready = pending_match.player1_ready && pending_match.player2_ready;

            if both_ready {
                info_log!("Both players are ready. Starting match: {}", match_id);
            }

            Some(both_ready)
        } else {
            None
        }
    }

    /// マッチをキャンセル
    pub fn cancel_match(&mut self, match_id: &str) -> Option<PendingMatch> {
        self.pending_matches.remove(match_id)
    }

    /// 特定のユーザーの保留中のマッチを見つける
    pub fn find_pending_match_for_user(&self, user_id: &str) -> Option<&PendingMatch> {
        self.pending_matches.values().find(|m| m.player1_id == user_id || m.player2_id == user_id)
    }

    /// キューの統計情報を取得
    pub fn get_queue_stats(&self) -> (usize, Duration) {
        let queue_size = self.queue.len();
        let avg_wait_time = if !self.queue.is_empty() {
            let total_wait_time = self.queue.values()
                .map(|entry| entry.joined_at.elapsed())
                .sum::<Duration>();
            total_wait_time / queue_size as u32
        } else {
            Duration::from_secs(0)
        };

        (queue_size, avg_wait_time)
    }

    /// 古いペンディングマッチをクリーンアップ
    pub fn cleanup_pending_matches(&mut self, timeout: Duration) -> Vec<PendingMatch> {
        let now = Instant::now();
        let mut expired_matches = Vec::new();

        // タイムアウトしたマッチを特定
        let expired_ids: Vec<String> = self.pending_matches.iter()
            .filter(|(_, m)| now.duration_since(m.created_at) > timeout)
            .map(|(id, _)| id.clone())
            .collect();

        // 期限切れのマッチを削除して返す
        for id in expired_ids {
            if let Some(expired_match) = self.pending_matches.remove(&id) {
                warning_log!("Match timed out: {}", id);
                expired_matches.push(expired_match);
            }
        }

        expired_matches
    }
}