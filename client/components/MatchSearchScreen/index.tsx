import { leaveQueue } from "@/services/ws";

export default function MatchSearchScreen() {
  return (
    <div className="flex flex-col items-center justify-center space-y-8 p-8 bg-gray-800 rounded-lg shadow-lg">
      <h1 className="text-3xl font-bold text-white">
        Searching for opponent...
      </h1>

      <div className="w-16 h-16 border-t-4 border-blue-500 border-solid rounded-full animate-spin"></div>

      <button
        onClick={leaveQueue}
        className="bg-red-600 hover:bg-red-700 text-white font-bold py-2 px-4 rounded"
      >
        Cancel
      </button>
    </div>
  );
}
