import { useGameStore } from "@/store/gameStore";
import { calculateValidMoves } from "@/utils/gameLogic";

let socket: WebSocket | null = null;
let heartbeatInterval: NodeJS.Timeout | null = null;

export const connectWebSocket = (url: string): Promise<void> => {
  return new Promise((resolve, reject) => {
    if (socket) {
      socket.close();
    }

    socket = new WebSocket(url);

    socket.onopen = () => {
      console.log("WebSocket connection established");
      useGameStore.getState().setConnectionStatus(true);

      // ハートビートの設定
      if (heartbeatInterval) {
        clearInterval(heartbeatInterval);
      }

      heartbeatInterval = setInterval(() => {
        if (socket && socket.readyState === WebSocket.OPEN) {
          sendMessage({
            type: "heartbeat",
            payload: {},
          });
        }
      }, 5000);

      resolve();
    };

    socket.onclose = () => {
      console.log("WebSocket connection closed");
      useGameStore.getState().setConnectionStatus(false);

      if (heartbeatInterval) {
        clearInterval(heartbeatInterval);
        heartbeatInterval = null;
      }
    };

    socket.onerror = (error) => {
      console.error("WebSocket error:", error);
      reject(error);
    };

    socket.onmessage = (event) => {
      try {
        if (typeof event.data === "string") {
          const message = JSON.parse(event.data);
          handleServerMessage(message);
        } else {
          console.error("Received non-string data:", event.data);
        }
      } catch (e) {
        console.error(
          "Failed to parse server message:",
          e,
          "Raw data:",
          event.data
        );
      }
    };
  });
};

export const sendMessage = (message: any): void => {
  if (socket && socket.readyState === WebSocket.OPEN) {
    try {
      const jsonString = JSON.stringify(message);
      socket.send(jsonString);
    } catch (e) {
      console.error("Failed to convert message to JSON:", e);
    }
  } else {
    console.error("WebSocket is not connected");
  }
};

export const authenticate = (username: string): void => {
  sendMessage({
    type: "authenticate",
    payload: { username },
  });

  useGameStore.getState().setUsername(username);
};

export const joinQueue = (): void => {
  sendMessage({
    type: "join_queue",
  });

  useGameStore.getState().startMatchSearch();
};

export const leaveQueue = (): void => {
  sendMessage({
    type: "leave_queue",
  });

  useGameStore.getState().cancelMatchSearch();
};

export const makeMove = (x: number, y: number): void => {
  sendMessage({
    type: "make_move",
    payload: { x, y },
  });
};

export const resign = (): void => {
  sendMessage({
    type: "resign",
  });
};

export const disconnect = (): void => {
  if (socket) {
    socket.close();
    socket = null;
  }

  if (heartbeatInterval) {
    clearInterval(heartbeatInterval);
    heartbeatInterval = null;
  }
};

const handleServerMessage = (message: any): void => {
  const gameStore = useGameStore.getState();

  switch (message.type) {
    case "success":
      console.log("Success:", message.message);

      if (message.message.includes("Joined matchmaking queue")) {
        gameStore.startMatchSearch();
      } else if (message.message.includes("Left matchmaking queue")) {
        gameStore.cancelMatchSearch();
      }
      break;

    case "error":
      console.error("Error:", message.message);
      break;

    case "match_found":
      console.log("Match found with:", message.opponent);
      gameStore.setMatchFound(message.opponent);
      break;

    case "game_state":
      // ボード状態を更新
      const board = message.board;
      const currentPlayer = message.current_player;
      const yourColor = message.your_color;

      gameStore.updateBoard(board, currentPlayer, yourColor);

      // 有効な手を計算
      const validMoves = calculateValidMoves(board, currentPlayer);
      gameStore.setValidMoves(validMoves);
      break;

    case "game_over":
      console.log("Game over. Reason:", message.reason);
      if (message.winner) {
        console.log("Winner:", message.winner);
      } else {
        console.log("Game ended in a draw");
      }

      gameStore.setGameOver(message.winner, message.reason);
      break;

    default:
      console.warn("Unknown message type:", message.type);
  }
};
