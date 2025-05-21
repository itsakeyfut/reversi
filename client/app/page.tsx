"use client";

import GameBoard from "@/components/GameBoard";
import GameOverScreen from "@/components/GameOverScreen";
import LobbyScreen from "@/components/LobbyScreen";
import LoginScreen from "@/components/LoginScreen";
import MatchSearchScreen from "@/components/MatchSearchScreen";
import { connectWebSocket } from "@/services/ws";
import { useGameStore } from "@/store/gameStore";
import { useEffect } from "react";

export default function Home() {
  const { isConnected, username, isSearchingMatch, isInGame, isGameOver } =
    useGameStore();

  useEffect(() => {
    connectWebSocket("ws://localhost:8080/ws").catch((err) =>
      console.error("Failed to connect WebSocket:", err)
    );

    return () => {
      // Disconnect
    };
  }, []);

  const renderScreen = () => {
    if (!isConnected) {
      return (
        <div className="text-white text-center">Connecting to server...</div>
      );
    }

    if (!username) return <LoginScreen />;
    if (isSearchingMatch) return <MatchSearchScreen />;
    if (isInGame) return <GameBoard />;
    if (isGameOver) return <GameOverScreen />;

    return <LobbyScreen />;
  };

  return (
    <main className="flex min-h-screen flex-col items-center justify-center p-4 bg-gray-900">
      {renderScreen()}
    </main>
  );
}
