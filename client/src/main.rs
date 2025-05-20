use bevy::prelude::*;

mod state;
mod network;
mod ui;
mod game_logic;

use state::{GameState, ConnectionState, BoardState};
use network::{WebSocketClient, init_websocket, handle_server_messages, GameEvent};
use ui::{UiRoot, setup_connection_ui, setup_lobby_ui, setup_matchmaking_ui, setup_game_ui, 
        setup_game_over_ui, update_disk_display, highlight_valid_moves, 
        handle_button_interactions, handle_board_click, update_game_info_text, send_heartbeat};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Reversi Online".to_string(),
                resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        // 状態とリソースを追加
        .init_state::<GameState>()
        .init_resource::<ConnectionState>()
        .init_resource::<BoardState>()
        .init_resource::<UiRoot>()
        .init_resource::<WebSocketClient>()
        // イベントを追加
        .add_event::<GameEvent>()
        // 起動時に実行するシステム
        .add_systems(Startup, (init_websocket, setup_connection_ui))
        // 状態に応じたシステム
        .add_systems(OnEnter(GameState::Connecting), setup_connection_ui)
        .add_systems(OnEnter(GameState::Lobby), setup_lobby_ui)
        .add_systems(OnEnter(GameState::MatchSearch), setup_matchmaking_ui)
        .add_systems(OnEnter(GameState::InGame), setup_game_ui)
        // 常に実行するシステム
        .add_systems(Update, (
            handle_server_messages,
            handle_button_interactions,
            send_heartbeat,
            handle_game_over_event,
        ))
        // ゲーム中のみ実行するシステム
        .add_systems(Update, (
            update_disk_display,
            highlight_valid_moves,
            handle_board_click,
            update_game_info_text,
        ).run_if(in_state(GameState::InGame)))
        .run();
}

// ゲーム終了イベントを処理するシステム
fn handle_game_over_event(
    mut events: EventReader<GameEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ui_root: ResMut<UiRoot>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in events.iter() {
        if let GameEvent::GameOver { winner, reason } = event {
            setup_game_over_ui(commands, asset_server, ui_root, winner.clone(), reason.clone());
            next_state.set(GameState::GameOver);
        }
    }
}