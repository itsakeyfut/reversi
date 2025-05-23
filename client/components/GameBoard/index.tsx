import { useGameStore, DiskColor } from "@/store/gameStore";
import { makeMove, resign } from "@/services/ws";

export default function GameBoard() {
  const { board, currentPlayer, yourColor, validMoves, opponent } =
    useGameStore();

  const isYourTurn = currentPlayer === yourColor;

  const handleCellClick = (x: number, y: number) => {
    if (isYourTurn && validMoves.some(([mx, my]) => mx === x && my === y)) {
      makeMove(x, y);
    }
  };

  const handleResign = () => {
    if (confirm("Are you sure you want to resign?")) {
      resign();
    }
  };

  return (
    <div className="flex flex-col items-center space-y-6">
      <div className="flex justify-between w-full max-w-lg p-4">
        <div className="text-white">
          <p>You: {yourColor === "black" ? "Black" : "White"}</p>
          <p>Opponent: {opponent}</p>
          <p>{isYourTurn ? "Your turn" : "Opponent's turn"}</p>
        </div>

        <button
          onClick={handleResign}
          className="bg-red-600 hover:bg-red-700 text-white px-4 py-2 rounded"
        >
          Resign
        </button>
      </div>

      <div className="grid grid-cols-8 gap-1 bg-green-800 p-4 rounded-lg">
        {board.map((row, y) =>
          row.map((cell, x) => (
            <div
              key={`${x}-${y}`}
              className={`w-14 h-14 flex items-center justify-center border border-green-900 ${
                validMoves.some(([mx, my]) => mx === x && my === y)
                  ? "bg-green-700 cursor-pointer"
                  : "bg-green-800"
              }`}
              onClick={() => handleCellClick(x, y)}
            >
              {cell && (
                <div
                  className={`w-10 h-10 rounded-full ${
                    cell === "black" ? "bg-black" : "bg-white"
                  }`}
                ></div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
