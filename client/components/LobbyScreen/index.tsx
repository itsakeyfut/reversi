import { joinQueue } from "@/services/ws";
import { useGameStore } from "@/store/gameStore";

const LobbyScreen = () => {
  const username = useGameStore((state) => state.username);

  return (
    <div className="flex flex-col items-center justify-center space-y-6 p-8 bg-gray-800 rounded-lg shadow-log">
      <h1 className="text-3xl font-bold text-white">Lobby</h1>
      <p className="text-white">Welcome, {username}</p>

      <button
        onClick={joinQueue}
        className="bg-green-600 hover:bg-green-700 text-white font-bold py-3 px-6 rounded-lg text-xl"
      >
        Find Match
      </button>
    </div>
  );
};

export default LobbyScreen;
