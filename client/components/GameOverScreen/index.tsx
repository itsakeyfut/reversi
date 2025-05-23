import { useGameStore } from "@/store/gameStore";

export default function GameOverScreen() {
  const { winner, gameOverReason, resetGame } = useGameStore();

  return (
    <div className="flex flex-col items-center justify-center space-y-6 p-8 bg-gray-800 rounded-lg shadow-lg">
      <h1 className="text-3xl font-bold text-white">Game Over</h1>

      <p className="text-xl text-white">{gameOverReason}</p>

      {winner ? (
        <p className="text-2xl font-bold text-yellow-400">Winner: {winner}</p>
      ) : (
        <p className="text-2xl text-gray-400">Game ended in a draw</p>
      )}

      <button
        onClick={resetGame}
        className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-6 rounded mt-4"
      >
        Return to Lobby
      </button>
    </div>
  );
}
