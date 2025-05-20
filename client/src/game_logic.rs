use crate::state::DiskColor;

// 方向を表す定数
const DIRECTIONS: [(i32, i32); 8] = [
    (-1, -1), (-1, 0), (-1, 1), 
    (0, -1),           (0, 1), 
    (1, -1),  (1, 0),  (1, 1)
];

// ボードの大きさ
const BOARD_SIZE: usize = 8;

// 有効な手かどうかをチェックする関数
pub fn is_valid_move(board: &Vec<Vec<Option<DiskColor>>>, x: usize, y: usize, color: &DiskColor) -> bool {
    // すでに石が置かれていたら無効
    if board[y][x].is_some() {
        return false;
    }
    
    // 8方向をチェック
    for (dx, dy) in &DIRECTIONS {
        let mut nx = x as i32 + dx;
        let mut ny = y as i32 + dy;
        
        // 盤外なら次の方向へ
        if nx < 0 || nx >= BOARD_SIZE as i32 || ny < 0 || ny >= BOARD_SIZE as i32 {
            continue;
        }
        
        // 隣が相手の色でなければ次の方向へ
        let opposite_color = match color {
            DiskColor::Black => DiskColor::White,
            DiskColor::White => DiskColor::Black,
        };
        
        match board[ny as usize][nx as usize] {
            Some(ref c) if c == &opposite_color => {},
            _ => continue,
        }
        
        // さらに進む
        nx += dx;
        ny += dy;
        
        // 盤外に出るまで進む
        while nx >= 0 && nx < BOARD_SIZE as i32 && ny >= 0 && ny < BOARD_SIZE as i32 {
            match board[ny as usize][nx as usize] {
                // 自分の色を見つけたら有効
                Some(ref c) if c == color => return true,
                // 相手の色なら進み続ける
                Some(ref c) if c == &opposite_color => {
                    nx += dx;
                    ny += dy;
                },
                // 空白なら無効（連続していない）
                _ => break,
            }
        }
    }
    
    false
}

// 有効な手を計算する関数
pub fn calculate_valid_moves(board: &Vec<Vec<Option<DiskColor>>>, current_player: &DiskColor) -> Vec<(usize, usize)> {
    let mut valid_moves = Vec::new();
    
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            if is_valid_move(board, x, y, current_player) {
                valid_moves.push((x, y));
            }
        }
    }
    
    valid_moves
}