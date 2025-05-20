use serde::{Deserialize, Serialize};

use crate::{app_log, debug_log, info_log};

const BOARD_SIZE: usize = 8;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiskColor {
    Black,
    White,
}

impl DiskColor {
    pub fn to_string(&self) -> String {
        match self {
            DiskColor::Black => "black".to_string(),
            DiskColor::White => "white".to_string(),
        }
    }

    pub fn opposite(&self) -> DiskColor {
        match self {
            DiskColor::Black => DiskColor::White,
            DiskColor::White => DiskColor::Black,
        }
    }
}

pub struct Game {
    pub game_id: String,
    pub black_player_id: String,
    pub black_player_name: String,
    pub white_player_id: String,
    pub white_player_name: String,
    board: Vec<Vec<Option<DiskColor>>>,
    current_color: DiskColor,
    is_game_over: bool,
    winner: Option<String>,
    move_history: Vec<GameMove>,
}

#[derive(Clone, Debug)]
pub struct GameMove {
    pub player_id: String,
    pub x: usize,
    pub y: usize,
    pub color: DiskColor,
}

#[derive(Clone, Debug)]
pub struct GameState {
    pub board: Vec<Vec<Option<String>>>,
    pub current_player: String,
    pub is_game_over: bool,
    pub winner: Option<String>,
    pub black_count: usize,
    pub white_count: usize,
    pub black_can_move: bool,
    pub white_can_move: bool,
}

const DIRECTIONS: [(i32, i32); 8] = [
    (-1, -1), (-1, 0), (-1, 1),
    (0, -1),           (0, 1),
    (1, -1), (1, 0), (1, 1),
];

impl Game {
    pub fn new(
        game_id: String,
        black_player_id: String,
        black_player_name: String,
        white_player_id: String,
        white_player_name: String,
    ) -> Self {
        let mut board = vec![vec![None; BOARD_SIZE]; BOARD_SIZE];

        // 初期配置
        board[3][3] = Some(DiskColor::White);
        board[3][4] = Some(DiskColor::Black);
        board[4][3] = Some(DiskColor::Black);
        board[4][4] = Some(DiskColor::White);

        Self {
            game_id,
            black_player_id,
            black_player_name,
            white_player_id,
            white_player_name,
            board,
            current_color: DiskColor::Black,
            is_game_over: false,
            winner: None,
            move_history: Vec::new(),
        }
    }

    pub fn get_state(&self) -> GameState {
        let mut black_count = 0;
        let mut white_count = 0;

        let board_strings = self.board
            .iter()
            .map(|row| {
                row.iter().map(|cell| {
                    match cell {
                        Some(DiskColor::Black) => {
                            black_count += 1;
                            Some("black".to_string())
                        },
                        Some(DiskColor::White) => {
                            white_count += 1;
                            Some("white".to_string())
                        },
                        None => None,
                    }
                }).collect()
            })
            .collect();

        let black_can_move = self.can_player_move(&DiskColor::Black);
        let white_can_move = self.can_player_move(&DiskColor::White);

        let current_player = match self.current_color {
            DiskColor::Black => "black".to_string(),
            DiskColor::White => "white".to_string(),
        };

        GameState {
            board: board_strings,
            current_player,
            is_game_over: self.is_game_over,
            winner: self.winner.clone(),
            black_count,
            white_count,
            black_can_move,
            white_can_move,
        }
    }

    fn can_player_move(&self, color: &DiskColor) -> bool {
        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                if self.is_valid_move(x, y, color) {
                    return true;
                }
            }
        }
        false
    }

    fn is_valid_move(&self, x: usize, y: usize, color: &DiskColor) -> bool {
        if self.board[y][x].is_some() {
            return false;
        }

        for (dx, dy) in &DIRECTIONS {
            let mut nx = x as i32 + dx;
            let mut ny = y as i32 + dy;
            
            // 盤外なら次の方向へ
            if nx < 0 || nx >= BOARD_SIZE as i32 || ny < 0 || ny >= BOARD_SIZE as i32 {
                continue;
            }
            
            // 隣が相手の色でなければ次の方向へ
            match &self.board[ny as usize][nx as usize] {
                Some(c) if c != color => {},
                _ => continue,
            }

            nx += dx;
            ny += dy;

            // 盤外に出るまで進む
            while nx >= 0 && nx < BOARD_SIZE as i32 && ny >= 0 && ny < BOARD_SIZE as i32 {
                match &self.board[ny as usize][nx as usize] {
                    // 自分の色を見つけたら有効
                    Some(c) if c == color => return true,
                    // 相手の色なら進み続ける
                    Some(_) => {
                        nx += dx;
                        ny += dy;
                    },
                    // 空白なら無効（連続していない）
                    None => break,
                }
            }
        }

        false
    }

        /// 石を反転させる
    fn flip_disks(&mut self, x: usize, y: usize, color: &DiskColor) {
        for (dx, dy) in &DIRECTIONS {
            let mut to_flip = Vec::new();
            let mut nx = x as i32 + dx;
            let mut ny = y as i32 + dy;
            
            // 盤外なら次の方向へ
            if nx < 0 || nx >= BOARD_SIZE as i32 || ny < 0 || ny >= BOARD_SIZE as i32 {
                continue;
            }
            
            // 隣が相手の色でなければ次の方向へ
            match &self.board[ny as usize][nx as usize] {
                Some(c) if c != color => {
                    to_flip.push((nx as usize, ny as usize));
                },
                _ => continue,
            }
            
            // さらに進む
            nx += dx;
            ny += dy;
            
            // 盤外に出るまで進む
            while nx >= 0 && nx < BOARD_SIZE as i32 && ny >= 0 && ny < BOARD_SIZE as i32 {
                match &self.board[ny as usize][nx as usize] {
                    // 自分の色を見つけたら挟まれた石を反転
                    Some(c) if c == color => {
                        for (fx, fy) in to_flip {
                            self.board[fy][fx] = Some(color.clone());
                        }
                        break;
                    },
                    // 相手の色なら反転候補に追加して進む
                    Some(_) => {
                        to_flip.push((nx as usize, ny as usize));
                        nx += dx;
                        ny += dy;
                    },
                    // 空白なら無効（連続していない）
                    None => break,
                }
            }
        }
    }
    
    /// 次のプレイヤーに変更
    fn next_player(&mut self) {
        self.current_color = match self.current_color {
            DiskColor::Black => DiskColor::White,
            DiskColor::White => DiskColor::Black,
        };
        
        // 次のプレイヤーが手を打てなければスキップ
        if !self.can_player_move(&self.current_color) {
            debug_log!(
                "Player {} has no valid moves. Skipping turn.", 
                self.current_color.to_string()
            );
            
            // 反対の色に戻す
            self.current_color = self.current_color.opposite();
            
            // もう一方も打てなければゲーム終了
            if !self.can_player_move(&self.current_color) {
                self.end_game();
            }
        }
    }
    
    /// ゲームを終了させる
    fn end_game(&mut self) {
        // 石の数をカウント
        let mut black_count = 0;
        let mut white_count = 0;
        
        for row in &self.board {
            for cell in row {
                match cell {
                    Some(DiskColor::Black) => black_count += 1,
                    Some(DiskColor::White) => white_count += 1,
                    None => {},
                }
            }
        }
        
        // 勝者を決定
        self.is_game_over = true;
        
        if black_count > white_count {
            self.winner = Some(self.black_player_id.clone());
            info_log!("Game over: {} won with {} disks vs {}", 
                   self.black_player_name, black_count, white_count);
        } else if white_count > black_count {
            self.winner = Some(self.white_player_id.clone());
            info_log!("Game over: {} won with {} disks vs {}", 
                   self.white_player_name, white_count, black_count);
        } else {
            // 引き分け
            self.winner = None;
            info_log!("Game over: Draw with {} disks each", black_count);
        }
    }
    
    /// プレイヤーIDから石の色を取得
    fn get_player_color(&self, player_id: &str) -> Option<DiskColor> {
        if player_id == self.black_player_id {
            Some(DiskColor::Black)
        } else if player_id == self.white_player_id {
            Some(DiskColor::White)
        } else {
            None
        }
    }
    
    /// 手を打つ
    pub fn make_move(&mut self, player_id: &str, x: usize, y: usize) -> Result<GameState, String> {
        // ゲームが終了していないか確認
        if self.is_game_over {
            return Err("Game is already over".to_string());
        }
        
        // プレイヤーの色を確認
        let player_color = match self.get_player_color(player_id) {
            Some(color) => color,
            None => return Err("You are not a player in this game".to_string()),
        };
        
        // プレイヤーのターンか確認
        if player_color != self.current_color {
            return Err("It's not your turn".to_string());
        }
        
        // 座標が有効範囲内か確認
        if x >= BOARD_SIZE || y >= BOARD_SIZE {
            return Err("Invalid coordinates".to_string());
        }
        
        // 手が有効か確認
        if !self.is_valid_move(x, y, &player_color) {
            return Err("Invalid move".to_string());
        }
        
        // 石を置く
        self.board[y][x] = Some(player_color.clone());
        
        // 反転処理
        self.flip_disks(x, y, &player_color);
        
        // 履歴に追加
        self.move_history.push(GameMove {
            player_id: player_id.to_string(),
            x,
            y,
            color: player_color,
        });
        
        // 次のプレイヤーに変更
        self.next_player();
        
        // 新しいゲーム状態を返す
        Ok(self.get_state())
    }
    
    /// ゲームを降参する
    pub fn resign(&mut self, player_id: &str) -> Result<GameState, String> {
        // ゲームが終了していないか確認
        if self.is_game_over {
            return Err("Game is already over".to_string());
        }
        
        // プレイヤーの色を確認
        if player_id != self.black_player_id && player_id != self.white_player_id {
            return Err("You are not a player in this game".to_string());
        }
        
        // 勝者を設定
        self.is_game_over = true;
        
        if player_id == self.black_player_id {
            self.winner = Some(self.white_player_id.clone());
            info_log!("Game over: {} resigned, {} wins", 
                   self.black_player_name, self.white_player_name);
        } else {
            self.winner = Some(self.black_player_id.clone());
            info_log!("Game over: {} resigned, {} wins", 
                   self.white_player_name, self.black_player_name);
        }
        
        // 新しいゲーム状態を返す
        Ok(self.get_state())
    }
}