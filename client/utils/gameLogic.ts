import { DiskColor } from "@/store/gameStore";

const DIRECTIONS = [
  [-1, -1],
  [-1, 0],
  [-1, 1],
  [0, -1],
  [0, 1],
  [1, -1],
  [1, 0],
  [1, 1],
];

const BOARD_SIZE = 8;

export const calculateValidMoves = (
  board: Array<Array<DiskColor | null>>,
  currentPlayer: DiskColor
): Array<[number, number]> => {
  const validMoves: Array<[number, number]> = [];

  for (let y = 0; y < BOARD_SIZE; y++) {
    for (let x = 0; x < BOARD_SIZE; x++) {
      if (isValidMove(board, x, y, currentPlayer)) {
        validMoves.push([x, y]);
      }
    }
  }

  return validMoves;
};

export const isValidMove = (
  board: Array<Array<DiskColor | null>>,
  x: number,
  y: number,
  color: DiskColor
): boolean => {
  // セルが既に埋まっている場合は無効
  if (board[y][x] !== null) {
    return false;
  }

  const oppositeColor = color === "black" ? "white" : "black";

  // 8方向をチェック
  for (const [dx, dy] of DIRECTIONS) {
    let nx = x + dx;
    let ny = y + dy;

    // 盤外または相手の石がない場合はこの方向をスキップ
    if (
      nx < 0 ||
      nx >= BOARD_SIZE ||
      ny < 0 ||
      ny >= BOARD_SIZE ||
      board[ny][nx] !== oppositeColor
    ) {
      continue;
    }

    nx += dx;
    ny += dy;

    // この方向に進み、自分の石があるかチェック
    while (nx >= 0 && nx < BOARD_SIZE && ny >= 0 && ny < BOARD_SIZE) {
      if (board[ny][nx] === null) {
        // 空のセルがあれば無効
        break;
      }

      if (board[ny][nx] === color) {
        // 自分の石があれば有効な手
        return true;
      }

      // 相手の石の場合は進み続ける
      nx += dx;
      ny += dy;
    }
  }

  return false;
};
