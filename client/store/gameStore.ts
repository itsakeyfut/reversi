import { create } from "zustand";

export type DiskColor = "black" | "white";

export type GameState = {
  board: Array<Array<DiskColor | null>>;
  currentPlayer: DiskColor;
  yourColor: DiskColor | null;
  validMoves: Array<[number, number]>;
  opponent: string | null;
  isInGame: boolean;
  isSearchingMatch: boolean;
  isGameOver: boolean;
  winner: string | null;
  gameOverReason: string | null;
  username: string | null;
  isConnected: boolean;
};

export type GameActions = {
  setUsername: (username: string) => void;
  setConnectionStatus: (status: boolean) => void;
  startMatchSearch: () => void;
  cancelMatchSearch: () => void;
  setMatchFound: (opponent: string) => void;
  updateBoard: (
    board: Array<Array<DiskColor | null>>,
    currentPlayer: DiskColor,
    yourColor: DiskColor
  ) => void;
  setValidMoves: (moves: Array<[number, number]>) => void;
  makeMove: (x: number, y: number) => void;
  resign: () => void;
  setGameOver: (winner: string | null, reason: string) => void;
  resetGame: () => void;
};

const initialState: GameState = {
  board: Array(8)
    .fill(null)
    .map(() => Array(8).fill(null)),
  currentPlayer: "black",
  yourColor: null,
  validMoves: [],
  opponent: null,
  isInGame: false,
  isSearchingMatch: false,
  isGameOver: false,
  winner: null,
  gameOverReason: null,
  username: null,
  isConnected: false,
};

export const useGameStore = create<GameState & GameActions>((set, get) => ({
  ...initialState,

  setUsername: (username) => set({ username }),

  setConnectionStatus: (status) => set({ isConnected: status }),

  startMatchSearch: () => set({ isSearchingMatch: true }),

  cancelMatchSearch: () => set({ isSearchingMatch: false }),

  setMatchFound: (opponent) =>
    set({
      opponent,
      isSearchingMatch: false,
      isInGame: true,
    }),

  updateBoard: (board, currentPlayer, yourColor) =>
    set({
      board,
      currentPlayer,
      yourColor,
    }),

  setValidMoves: (moves) => set({ validMoves: moves }),

  makeMove: (x, y) => {},
  resign: () => {},

  setGameOver: (winner, reason) =>
    set({
      isGameOver: true,
      isInGame: false,
      winner,
      gameOverReason: reason,
    }),

  resetGame: () =>
    set({
      ...initialState,
      username: get().username,
      isConnected: get().isConnected,
    }),
}));
