use bevy::{ecs::schedule::ShouldRun, prelude::*};
use bevy_renet::{run_if_client_connected, RenetClientPlugin};
use renet::{
    ClientAuthentication, RenetClient, RenetConnectionConfig, RenetError, NETCODE_USER_DATA_BYTES,
};
use std::{net::UdpSocket, time::SystemTime};
use store::{EndGameReason, GameEvent, GameState};

// This id needs to be the same that the server is using
const PROTOCOL_ID: u64 = 1208;

fn main() {
    // Get username from stdin args
    let args = std::env::args().collect::<Vec<String>>();
    let username = &args[1];

    App::new()
        .insert_resource(WindowDescriptor {
            title: "TicTacTussle".to_string(),
            width: 480.0,
            height: 540.0,
            ..default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::hex("282828").unwrap()))
        .add_plugins(DefaultPlugins)
        // Renet setup
        .add_plugin(RenetClientPlugin)
        .insert_resource(new_renet_client(&username).unwrap())
        .add_system(handle_renet_error)
        .add_system_to_stage(
            CoreStage::PostUpdate,
            receive_events_from_server.with_run_criteria(run_if_client_connected),
        )
        // Add our game state and register GameEvent as a bevy event
        .insert_resource(GameState::default())
        .add_event::<GameEvent>()
        // Add setup function to spawn UI and board graphics
        .add_startup_system(setup)
        .add_system(change_ui_by_stage)
        // Update Ui in pregame state
        .add_system(update_waiting_text.with_run_criteria(run_if_pregame))
        // Add input and update system, but only run them when we are ingame
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(run_if_ingame)
                .with_system(input)
                .with_system(update_board)
                .with_system(update_in_game_ui),
        )
        // Finally we run the thing!
        .run();
}

////////// COMPONENTS //////////
#[derive(Component)]
struct UIRoot;

type TileIndex = usize;
#[derive(Component)]
struct HoverDot(pub TileIndex);

#[derive(Component)]
struct WaitingText;

////////// SETUP //////////
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    // Spawn board background
    commands.spawn_bundle(SpriteBundle {
        transform: Transform::from_xyz(0.0, -30.0, 0.0),
        sprite: Sprite {
            custom_size: Some(Vec2::new(480.0, 480.0)),
            ..default()
        },
        texture: asset_server.load("background.png").into(),
        ..default()
    });

    // Spawn a dot in each tile for hover effect
    for x in 0..3 {
        for y in 0..3 {
            commands
                .spawn_bundle(SpriteBundle {
                    transform: Transform::from_xyz(
                        160.0 * (x as f32 - 1.0),
                        -30.0 + 160.0 * (y as f32 - 1.0),
                        0.0,
                    ),
                    sprite: Sprite {
                        color: Color::rgba(1.0, 1.0, 1.0, 0.0),
                        custom_size: Some(Vec2::new(160.0, 160.0)),
                        ..default()
                    },
                    texture: asset_server.load("dot.png").into(),
                    ..default()
                })
                .insert(HoverDot(x + y * 3));
        }
    }

    // Spawn pregame ui
    commands
        // A container that centers its children on the screen
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                size: Size::new(Val::Percent(100.0), Val::Px(60.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(UIRoot)
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle::from_section(
                    "Waiting for an opponent...",
                    TextStyle {
                        font: asset_server.load("Inconsolata.ttf"),
                        font_size: 24.0,
                        color: Color::hex("ebdbb2").unwrap(),
                    },
                ))
                .insert(WaitingText);
        });
}

////////// UPDATE SYSTEMS //////////
fn input(
    windows: Res<Windows>,
    mut hover_dots: Query<(&HoverDot, &mut Sprite)>,
    input: Res<Input<MouseButton>>,
    mut client: ResMut<RenetClient>,
) {
    let window = windows.get_primary().unwrap();
    if let Some(mouse_position) = window.cursor_position() {
        // Determine the index of the tile that the mouse is currently over
        let x_tile: usize = (mouse_position.x / 160.0).floor() as usize;
        let y_tile: usize = (mouse_position.y / 160.0).floor() as usize;
        let tile = x_tile + y_tile * 3;

        // If mouse is outside of board we do nothing
        if 8 < tile {
            return;
        }

        // Toggle hover dots on and off
        for (dot, mut dot_sprite) in hover_dots.iter_mut() {
            if dot.0 == tile {
                dot_sprite.color.set_a(1.0);
            } else {
                dot_sprite.color.set_a(0.0);
            }
        }

        // If left mouse button is pressed, send a place tile event to the server
        if input.just_pressed(MouseButton::Left) {
            let event = GameEvent::PlaceTile {
                player_id: client.client_id(),
                at: tile,
            };
            client.send_message(0, bincode::serialize(&event).unwrap());
        }
    }
}

fn update_board(
    mut commands: Commands,
    game_state: Res<GameState>,
    mut game_events: EventReader<GameEvent>,
    asset_server: Res<AssetServer>,
) {
    for event in game_events.iter() {
        match event {
            GameEvent::PlaceTile { player_id, at } => {
                let x = at % 3;
                let y = at / 3;
                let texture =
                    asset_server.load(match game_state.get_player_tile(player_id).unwrap() {
                        store::Tile::Tac => "tac.png",
                        store::Tile::Tic => "tic.png",
                        store::Tile::Empty => "dot.png", // This should never happen
                    });

                commands.spawn_bundle(SpriteBundle {
                    transform: Transform::from_xyz(
                        160.0 * (x as f32 - 1.0),
                        -30.0 + 160.0 * (y as f32 - 1.0),
                        0.0,
                    ),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(160.0, 160.0)),
                        ..default()
                    },
                    texture: texture.into(),
                    ..default()
                });
            }
            _ => {}
        }
    }
}

fn update_waiting_text(mut text_query: Query<&mut Text, With<WaitingText>>, time: Res<Time>) {
    let mut text = text_query.get_single_mut().unwrap();
    let num_dots = (time.time_since_startup().as_secs() % 3) + 1;
    text.sections[0].value = format!(
        "Waiting for an opponent{}{}",
        ".".repeat(num_dots as usize),
        // Pad with spaces to avoid text changing width and dancing all around the screen 🕺
        " ".repeat(3 - num_dots as usize)
    );
}

fn change_ui_by_stage(
    mut commands: Commands,
    mut game_events: EventReader<GameEvent>,
    waiting_text: Query<Entity, With<WaitingText>>,
) {
    for event in game_events.iter() {
        match event {
            GameEvent::BeginGame { goes_first: _ } => {
                // Remove waiting text when game begins
                commands
                    .entity(waiting_text.get_single().unwrap())
                    .despawn_recursive();

                // TODO: Spawn in game ui
            }
            GameEvent::EndGame { reason } => {
                // TODO: Despawn in game ui
                match reason {
                    EndGameReason::PlayerLeft { player_id: _ } => {
                        // TODO: Spawn error text
                    }
                    EndGameReason::PlayerWon { winner: _ } => {
                        // TODO: Spawn celebration
                    }
                }
            }
            _ => {}
        }
    }
}

fn update_in_game_ui(_game_state: Res<GameState>, mut game_events: EventReader<GameEvent>) {
    for event in game_events.iter() {
        match event {
            GameEvent::PlaceTile {
                player_id: _,
                at: _,
            } => {
                // TODO: Change highlighted player
            }
            _ => {}
        }
    }
}

////////// RENET NETWORKING //////////
fn new_renet_client(username: &String) -> anyhow::Result<RenetClient> {
    let server_addr = format!("{}:{}", env!("HOST"), env!("PORT")).parse()?;
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let client_id = current_time.as_millis() as u64;

    // Place username in user data
    let mut user_data = [0u8; NETCODE_USER_DATA_BYTES];
    if username.len() > NETCODE_USER_DATA_BYTES - 8 {
        panic!("Username is too big");
    }
    user_data[0..8].copy_from_slice(&(username.len() as u64).to_le_bytes());
    user_data[8..username.len() + 8].copy_from_slice(username.as_bytes());

    let client = RenetClient::new(
        current_time,
        socket,
        client_id,
        RenetConnectionConfig::default(),
        ClientAuthentication::Unsecure {
            client_id,
            protocol_id: crate::PROTOCOL_ID,
            server_addr,
            user_data: Some(user_data),
        },
    )?;

    Ok(client)
}

fn receive_events_from_server(
    mut client: ResMut<RenetClient>,
    mut game_state: ResMut<GameState>,
    mut game_events: EventWriter<GameEvent>,
) {
    while let Some(message) = client.receive_message(0) {
        // Whenever the server sends a message we know that it must be a game event
        let event: GameEvent = bincode::deserialize(&message).unwrap();
        trace!("{:#?}", event);

        // We trust the server - It's always been good to us!
        // No need to validate the events it is sending us
        game_state.consume(&event);

        // Send the event into the bevy event system so systems can react to it
        game_events.send(event);
    }
}

// If there's any error network we just panic 🤷‍♂️
fn handle_renet_error(mut renet_error: EventReader<RenetError>) {
    for err in renet_error.iter() {
        panic!("{}", err);
    }
}

////////// RUN CRITERIA //////////
fn run_if_pregame(game_state: Res<GameState>) -> ShouldRun {
    match game_state.stage {
        store::Stage::PreGame => ShouldRun::Yes,
        _ => ShouldRun::No,
    }
}

fn run_if_ingame(game_state: Res<GameState>) -> ShouldRun {
    match game_state.stage {
        store::Stage::InGame => ShouldRun::Yes,
        _ => ShouldRun::No,
    }
}
