use std::{net::UdpSocket, time::SystemTime};

use crate::{AppState, UIRoot};
use bevy::prelude::*;
use renet::{ClientAuthentication, RenetClient, RenetConnectionConfig, NETCODE_USER_DATA_BYTES};

// In the 'Initial' state the player enters their name and tries to connect to the server.

const DELETE: char = '\u{7f}';
const ENTER: char = '\r';

#[derive(Component)]
struct NameInput;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        // A container that centers its children on the screen
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                // Bevy UI is placing children such that first child goes at the bottom
                // This is the opposite of how a browser does it, but we can get back to
                // familiarity by just using ColumnReverse 👍
                padding: UiRect::all(Val::Px(32.0)),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(UIRoot)
        .with_children(|parent| {
            // Some explanation text to get the player started
            parent.spawn_bundle(
                TextBundle::from_section(
                    "Enter your name",
                    TextStyle {
                        font: asset_server.load("Inconsolata.ttf"),
                        font_size: 24.0,
                        color: Color::hex("ebdbb2").unwrap(),
                    },
                )
                .with_style(Style {
                    margin: UiRect {
                        bottom: Val::Px(8.0),
                        ..default()
                    },
                    ..default()
                }),
            );

            // A text field that will serve as out text input
            parent
                .spawn_bundle(
                    TextBundle::from_section(
                        "",
                        TextStyle {
                            font: asset_server.load("Inconsolata.ttf"),
                            font_size: 38.0,
                            color: Color::hex("458488").unwrap(),
                        },
                    )
                    .with_style(Style {
                        size: Size::new(Val::Auto, Val::Px(42.0)),
                        margin: UiRect {
                            bottom: Val::Px(32.0),
                            ..default()
                        },
                        ..default()
                    }),
                )
                .insert(NameInput);

            // A button that will trigger a connection attempt
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect {
                            top: Val::Px(8.0),
                            bottom: Val::Px(8.0),
                            left: Val::Px(16.0),
                            right: Val::Px(16.0),
                        },
                        ..default()
                    },
                    color: Color::hex("458488").unwrap().into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle::from_section(
                        "Get Tusslin'!",
                        TextStyle {
                            font: asset_server.load("Inconsolata.ttf"),
                            font_size: 24.0,
                            color: Color::hex("ebdbb2").unwrap(),
                        },
                    ));
                });
        });
}

fn username_input(
    mut char_events: EventReader<ReceivedCharacter>,
    mut name_input: Query<&mut Text, With<NameInput>>,
) {
    let mut text = name_input.get_single_mut().unwrap();
    for event in char_events.iter() {
        if event.char == DELETE {
            text.sections[0].value.pop();
            continue;
        }

        // If theres still space left in renet user data push the char to username
        if text.sections[0].value.len() < NETCODE_USER_DATA_BYTES {
            text.sections[0].value.push(event.char);
        }
    }
}

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

fn connect_to_server(
    mut commands: Commands,
    mut char_events: EventReader<ReceivedCharacter>,
    interactions: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    name_input: Query<&Text, With<NameInput>>,
    mut app_state: ResMut<State<AppState>>,
) {
    let mut should_connect = false;
    let username = &name_input.get_single().unwrap().sections[0].value;

    // Connect to server when player presses enter
    for event in char_events.iter() {
        should_connect = event.char == ENTER;
    }

    // Connect to server when player pressed the button
    for interaction in interactions.iter() {
        match interaction {
            Interaction::Clicked => {
                should_connect = true;
            }
            _ => {}
        }
    }

    if should_connect {
        if let Ok(client) = new_renet_client(&username) {
            commands.insert_resource(client);
            app_state.set(AppState::Connected).unwrap();
        } else {
            app_state.set(AppState::Error).unwrap();
        }
    }
}

fn cleanup(mut commands: Commands, ui_root: Query<Entity, With<UIRoot>>) {
    commands
        .entity(ui_root.get_single().unwrap())
        .despawn_recursive();
}

pub struct StateInitial;
impl Plugin for StateInitial {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::Initial).with_system(setup));
        app.add_system_set(
            SystemSet::on_update(AppState::Initial)
                .with_system(username_input)
                .with_system(connect_to_server),
        );
        app.add_system_set(SystemSet::on_exit(AppState::Initial).with_system(cleanup));
    }
}
