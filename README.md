# Multiplayer Reversi

## Screenshots

### Lobby

![Lobby](./screenshots/player1_lobby.png)

### Playing

![player1_in_game](./screenshots/player1_in_game.png)

![player2_in_game](./screenshots/player2_in_game.png)

![player3_in_game](./screenshots/player3_in_game.png)

![player4_in_game](./screenshots/player4_in_game.png)

### Winner

![player1_win](./screenshots/player1_win.png)

![player4_win](./screenshots/player4_win.png)

## Features

- Sharing messages
  - ClientMessage and ServerMessage
  - Connect and Disconnect
  - SendMessage
- Finding matches
  - QueueEntry, which manages player info and ready flag
  - Supply matchmaking service with queues
- Managing sessions
  - Actor Based Session
  - Manage user status
  - operate WebSocket Streaming handler
