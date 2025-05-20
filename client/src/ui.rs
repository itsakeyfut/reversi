// src/ui.rs
use bevy::prelude::*;
use crate::state::{GameState, ConnectionState, BoardState, DiskColor, ClientMessage};
use crate::network::{WebSocketClient, send_message};

// UIのルートエンティティを管理するリソース
#[derive(Resource, Default)]
pub struct UiRoot {
    pub connection_ui: Option<Entity>,
    pub lobby_ui: Option<Entity>,
    pub matchmaking_ui: Option<Entity>,
    pub game_ui: Option<Entity>,
    pub game_over_ui: Option<Entity>,
}

// UI関連のコンポーネント
#[derive(Component)]
pub struct ConnectionUi;

#[derive(Component)]
pub struct LobbyUi;

#[derive(Component)]
pub struct MatchmakingUi;

#[derive(Component)]
pub struct GameUi;

#[derive(Component)]
pub struct GameOverUi;

// ゲームボードとセルのコンポーネント
#[derive(Component)]
pub struct GameBoard;

#[derive(Component)]
pub struct BoardCell {
    pub x: usize,
    pub y: usize,
}

#[derive(Component)]
pub struct DiskDisplay {
    pub color: DiskColor,
}

// 接続UIのセットアップ
pub fn setup_connection_ui(mut commands: Commands, asset_server: Res<AssetServer>, mut ui_root: ResMut<UiRoot>) {
    // 既存のUIを削除
    if let Some(entity) = ui_root.connection_ui {
        commands.entity(entity).despawn_recursive();
    }
    
    // 接続UIを作成
    let connection_ui = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            ConnectionUi,
        ))
        .with_children(|parent| {
            // タイトル
            parent.spawn(
                TextBundle::from_section(
                    "Othello Online",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::WHITE,
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                }),
            );
            
            // ユーザー名入力
            parent.spawn(
                TextBundle::from_section(
                    "Enter your username:",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                }),
            );
            
            // 入力フィールド
            parent.spawn((
                TextInputBundle {
                    text_input: TextInput {
                        text: "Player".to_string(),
                        cursor_position: 6,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Px(30.0),
                        padding: UiRect::all(Val::Px(5.0)),
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    ..default()
                },
                UsernameInput,
            ));
            
            // ログインボタン
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(120.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.2, 0.2, 0.8)),
                    ..default()
                },
                LoginButton,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Login",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                ));
            });
        })
        .id();
    
    ui_root.connection_ui = Some(connection_ui);
}

// ロビーUIのセットアップ
pub fn setup_lobby_ui(mut commands: Commands, asset_server: Res<AssetServer>, mut ui_root: ResMut<UiRoot>) {
    // 既存のUIを削除
    if let Some(entity) = ui_root.lobby_ui {
        commands.entity(entity).despawn_recursive();
    }
    
    // ロビーUIを作成
    let lobby_ui = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            LobbyUi,
        ))
        .with_children(|parent| {
            // タイトル
            parent.spawn(
                TextBundle::from_section(
                    "Lobby",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::WHITE,
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                }),
            );
            
            // マッチング開始ボタン
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
                    ..default()
                },
                FindMatchButton,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Find Match",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 24.0,
                        color: Color::WHITE,
                    },
                ));
            });
        })
        .id();
    
    ui_root.lobby_ui = Some(lobby_ui);
}

// マッチング検索UI
pub fn setup_matchmaking_ui(mut commands: Commands, asset_server: Res<AssetServer>, mut ui_root: ResMut<UiRoot>) {
    // 既存のUIを削除
    if let Some(entity) = ui_root.matchmaking_ui {
        commands.entity(entity).despawn_recursive();
    }
    
    // マッチング検索UIを作成
    let matchmaking_ui = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            MatchmakingUi,
        ))
        .with_children(|parent| {
            // 検索中メッセージ
            parent.spawn(
                TextBundle::from_section(
                    "Searching for opponent...",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 30.0,
                        color: Color::WHITE,
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                }),
            );
            
            // キャンセルボタン
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(150.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
                    ..default()
                },
                CancelMatchButton,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Cancel",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                ));
            });
        })
        .id();
    
    ui_root.matchmaking_ui = Some(matchmaking_ui);
}

// ゲームUI
pub fn setup_game_ui(mut commands: Commands, asset_server: Res<AssetServer>, mut ui_root: ResMut<UiRoot>) {
    // 既存のUIを削除
    if let Some(entity) = ui_root.game_ui {
        commands.entity(entity).despawn_recursive();
    }
    
    // ゲームUIを作成
    let game_ui = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            GameUi,
        ))
        .with_children(|parent| {
            // ゲーム情報
            parent.spawn(
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    ..default()
                },
            )
            .with_children(|parent| {
                // プレイヤー情報
                parent.spawn((
                    TextBundle::from_section(
                        "Your turn (Black)",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 24.0,
                            color: Color::WHITE,
                        },
                    ),
                    GameInfoText,
                ));
                
                // 降参ボタン
                parent.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(100.0),
                            height: Val::Px(30.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
                        ..default()
                    },
                    ResignButton,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Resign",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                            font_size: 16.0,
                            color: Color::WHITE,
                        },
                    ));
                });
            });
            
            // ゲームボード
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(400.0),
                        height: Val::Px(400.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.1, 0.5, 0.1)),
                    ..default()
                },
                GameBoard,
            ));
        })
        .id();
    
    ui_root.game_ui = Some(game_ui);
    
    // ゲームボードのセルを作成
    if let Some(board_entity) = commands.get_entity(game_ui) {
        if let Some(board_node) = board_entity.get::<Children>() {
            // ボードを探す
            for &child in board_node.iter() {
                if let Some(node) = commands.get_entity(child) {
                    if let Some(_) = node.get::<GameBoard>() {
                        // ボードにセルを追加
                        spawn_board_cells(commands, child, asset_server);
                        break;
                    }
                }
            }
        }
    }
}

// ボードのセルを生成する関数 (続き)
fn spawn_board_cells(mut commands: Commands, board_entity: Entity, asset_server: Res<AssetServer>) {
    let cell_size = 50.0;
    
    commands.entity(board_entity).with_children(|parent| {
        for y in 0..8 {
            for x in 0..8 {
                let position = UiRect {
                    left: Val::Px(x as f32 * cell_size),
                    top: Val::Px(y as f32 * cell_size),
                    right: Val::Auto,
                    bottom: Val::Auto,
                };
                
                // セルを生成
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(cell_size),
                            height: Val::Px(cell_size),
                            position_type: PositionType::Absolute,
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            position: position,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
                        ..default()
                    },
                    BoardCell { x, y },
                ));
            }
        }
    });
}

// ゲーム終了UI
pub fn setup_game_over_ui(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    mut ui_root: ResMut<UiRoot>,
    winner: Option<String>,
    reason: String,
) {
    // 既存のUIを削除
    if let Some(entity) = ui_root.game_over_ui {
        commands.entity(entity).despawn_recursive();
    }
    
    // ゲーム終了UIを作成
    let game_over_ui = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                ..default()
            },
            GameOverUi,
        ))
        .with_children(|parent| {
            // ゲーム終了メッセージ
            parent.spawn(
                TextBundle::from_section(
                    "Game Over",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::WHITE,
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                }),
            );
            
            // 理由
            parent.spawn(
                TextBundle::from_section(
                    reason,
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                        font_size: 24.0,
                        color: Color::WHITE,
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                }),
            );
            
            // 勝者
            if let Some(winner_name) = winner {
                parent.spawn(
                    TextBundle::from_section(
                        format!("Winner: {}", winner_name),
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 32.0,
                            color: Color::srgb(1.0, 0.8, 0.2),
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    }),
                );
            } else {
                parent.spawn(
                    TextBundle::from_section(
                        "Game ended in a draw",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 32.0,
                            color: Color::srgb(0.8, 0.8, 0.8),
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    }),
                );
            }
            
            // 戻るボタン
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(150.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.2, 0.2, 0.8)),
                    ..default()
                },
                ReturnToLobbyButton,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Return to Lobby",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                ));
            });
        })
        .id();
    
    ui_root.game_over_ui = Some(game_over_ui);
}

// ディスクを描画する
pub fn update_disk_display(
    mut commands: Commands,
    board_state: Res<BoardState>,
    cells: Query<(Entity, &BoardCell)>,
    disks: Query<(Entity, &DiskDisplay), With<DiskDisplay>>,
) {
    // 既存のディスクを削除
    for (entity, _) in &disks {
        commands.entity(entity).despawn();
    }
    
    // 新しいディスクを配置
    for (cell_entity, cell) in &cells {
        if let Some(cell_value) = board_state.board.get(cell.y).and_then(|row| row.get(cell.x)) {
            if let Some(color) = cell_value {
                commands.entity(cell_entity).with_children(|parent| {
                    parent.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Px(40.0),
                                height: Val::Px(40.0),
                                border_radius: BorderRadius::all(Val::Px(20.0)),
                                ..default()
                            },
                            background_color: BackgroundColor(match color {
                                DiskColor::Black => Color::BLACK,
                                DiskColor::White => Color::WHITE,
                            }),
                            ..default()
                        },
                        DiskDisplay { color: color.clone() },
                    ));
                });
            }
        }
    }
}

// 有効な手を表示する
pub fn highlight_valid_moves(
    mut cells: Query<(&BoardCell, &mut BackgroundColor)>,
    board_state: Res<BoardState>,
) {
    // セルの背景色をリセット
    for (cell, mut bg) in &mut cells {
        *bg = BackgroundColor(Color::srgb(0.2, 0.6, 0.2));
    }
    
    // 自分のターンの場合のみ有効な手をハイライト
    if let Some(your_color) = &board_state.your_color {
        if your_color == &board_state.current_player {
            for (cell, mut bg) in &mut cells {
                if board_state.valid_moves.contains(&(cell.x, cell.y)) {
                    *bg = BackgroundColor(Color::srgb(0.4, 0.7, 0.4));
                }
            }
        }
    }
}

// ゲーム情報テキストを更新
pub fn update_game_info_text(
    mut text_query: Query<&mut Text, With<GameInfoText>>,
    board_state: Res<BoardState>,
    connection: Res<ConnectionState>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        let current_player_text = match board_state.current_player {
            DiskColor::Black => "Black",
            DiskColor::White => "White",
        };
        
        let your_color_text = match &board_state.your_color {
            Some(DiskColor::Black) => "Black",
            Some(DiskColor::White) => "White",
            None => "Unknown",
        };
        
        let opponent_name = connection.opponent.as_deref().unwrap_or("Opponent");
        
        let turn_text = if Some(&board_state.current_player) == board_state.your_color.as_ref() {
            "Your turn"
        } else {
            &format!("{}'s turn", opponent_name)
        };
        
        text.sections[0].value = format!("{} ({}) vs {} - {}", 
            your_color_text,
            connection.username.as_deref().unwrap_or("You"),
            opponent_name,
            turn_text
        );
    }
}

// ボタンコンポーネント
#[derive(Component)]
pub struct LoginButton;

#[derive(Component)]
pub struct FindMatchButton;

#[derive(Component)]
pub struct CancelMatchButton;

#[derive(Component)]
pub struct ResignButton;

#[derive(Component)]
pub struct ReturnToLobbyButton;

#[derive(Component)]
pub struct UsernameInput;

#[derive(Component)]
pub struct GameInfoText;

// ボタンのインタラクションを処理するシステム
pub fn handle_button_interactions(
    mut interaction_query: Query
        (&Interaction, &mut BackgroundColor, Option<&LoginButton>, Option<&FindMatchButton>, 
         Option<&CancelMatchButton>, Option<&ResignButton>, Option<&ReturnToLobbyButton>),
        (Changed::<Interaction>, With<Button>)
    >,
    mut username_input: Query<&mut TextInput, With<UsernameInput>>,
    mut connection: ResMut<ConnectionState>,
    client: Res<WebSocketClient>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut bg_color, login_btn, find_match_btn, cancel_match_btn, resign_btn, return_to_lobby_btn) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.35, 0.35));
                
                // ログインボタンが押された
                if login_btn.is_some() {
                    if let Ok(input) = username_input.get_single() {
                        let username = input.text.clone();
                        if !username.trim().is_empty() {
                            connection.username = Some(username.clone());
                            
                            if let Some(client) = &client.message_tx {
                                // 認証メッセージを送信
                                let _ = send_message(&client, ClientMessage::Authenticate { 
                                    username: username.clone() 
                                });
                                next_state.set(GameState::Lobby);
                            }
                        }
                    }
                }
                
                // マッチング検索ボタンが押された
                if find_match_btn.is_some() {
                    if let Some(client) = &client.message_tx {
                        // マッチングキューに参加
                        let _ = send_message(&client, ClientMessage::JoinQueue);
                    }
                }
                
                // キャンセルボタンが押された
                if cancel_match_btn.is_some() {
                    if let Some(client) = &client.message_tx {
                        // マッチングキューから離脱
                        let _ = send_message(&client, ClientMessage::LeaveQueue);
                    }
                }
                
                // 降参ボタンが押された
                if resign_btn.is_some() {
                    if let Some(client) = &client.message_tx {
                        // 降参メッセージを送信
                        let _ = send_message(&client, ClientMessage::Resign);
                    }
                }
                
                // ロビーに戻るボタンが押された
                if return_to_lobby_btn.is_some() {
                    next_state.set(GameState::Lobby);
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
            }
            Interaction::None => {
                *bg_color = if login_btn.is_some() {
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.8))
                } else if find_match_btn.is_some() {
                    BackgroundColor(Color::srgb(0.2, 0.6, 0.2))
                } else if cancel_match_btn.is_some() || resign_btn.is_some() {
                    BackgroundColor(Color::srgb(0.8, 0.2, 0.2))
                } else if return_to_lobby_btn.is_some() {
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.8))
                } else {
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15))
                };
            }
        }
    }
}

// ボードのクリックを処理するシステム
pub fn handle_board_click(
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    cells: Query<(&BoardCell, &Node, &GlobalTransform)>,
    board_state: Res<BoardState>,
    client: Res<WebSocketClient>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        // 自分のターンか確認
        if let Some(your_color) = &board_state.your_color {
            if your_color != &board_state.current_player {
                return; // 自分のターンでなければ何もしない
            }
        } else {
            return; // 色が設定されていなければ何もしない
        }
        
        // カーソル位置を取得
        let window = windows.get_single().unwrap();
        if let Some(cursor_position) = window.cursor_position() {
            let (camera, camera_transform) = camera_q.get_single().unwrap();
            
            // カーソル位置をワールド座標に変換
            let ndc = (cursor_position / Vec2::new(window.width(), window.height())) * 2.0 - Vec2::ONE;
            let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
            let world_position = ndc_to_world.project_point3(Vec3::new(ndc.x, ndc.y, 0.0));
            
            // クリックされたセルを特定
            for (cell, node, transform) in &cells {
                let cell_size = node.size();
                let cell_position = transform.translation();
                
                // クリックがセル内か確認
                let cell_min = Vec2::new(cell_position.x - cell_size.x / 2.0, cell_position.y - cell_size.y / 2.0);
                let cell_max = Vec2::new(cell_position.x + cell_size.x / 2.0, cell_position.y + cell_size.y / 2.0);
                
                if world_position.x >= cell_min.x && world_position.x <= cell_max.x &&
                   world_position.y >= cell_min.y && world_position.y <= cell_max.y {
                    // 有効な手か確認
                    if board_state.valid_moves.contains(&(cell.x, cell.y)) {
                        // 手を打つメッセージを送信
                        if let Some(client) = &client.message_tx {
                            let _ = send_message(&client, ClientMessage::MakeMove { 
                                x: cell.x, 
                                y: cell.y 
                            });
                        }
                    }
                    break;
                }
            }
        }
    }
}

// 定期的なハートビートを送信するシステム
pub fn send_heartbeat(
    time: Res<Time>,
    mut heartbeat_timer: Local<f32>,
    client: Res<WebSocketClient>,
) {
    // 5秒ごとにハートビートを送信
    *heartbeat_timer += time.delta_secs();
    if *heartbeat_timer >= 5.0 {
        *heartbeat_timer = 0.0;
        
        if let Some(client) = &client.message_tx {
            let _ = send_message(&client, ClientMessage::Heartbeat);
        }
    }
}