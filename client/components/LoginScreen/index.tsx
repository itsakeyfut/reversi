import { authenticate } from "@/services/ws";
import { useState } from "react";

const LoginScreen = () => {
  const [username, setUsername] = useState("Player");

  const handleLogin = () => {
    if (username.trim()) {
      authenticate(username);
    }
  };

  return (
    <div className="flex flex-col items-center justify-center space-y-6 p-8 bg-gray-800 rounded-lg shadow-lg">
      <h1 className="text-3xl font-bold text-white">Reversi Online</h1>

      <div className="flex flex-col space-y-4 w-full max-w-md">
        <label className="text-white">Username:</label>
        <input
          type="text"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          className="px-4 py-2 rounded"
        />

        <button
          onClick={handleLogin}
          className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
        >
          Login
        </button>
      </div>
    </div>
  );
};

export default LoginScreen;
