use bevy::prelude::*;

mod states;
use bevy_renet::{run_if_client_connected, RenetClientPlugin};
use renet::{RenetClient, RenetError};
use states::{
    connected::StateConnected, error::StateError, finished::StateFinished, ingame::StateInGame,
    initial::StateInitial,
};
use store::{EndGameReason, GameEvent, GameState};

// This id needs to be the same that the server is using
const PROTOCOL_ID: u64 = 1208;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum AppState {
    Initial,
    Connected,
    InGame,
    Error,
    Finished,
}

// A marker component used by multiple app states for despawning all state ui on exit
#[derive(Component)]
struct UIRoot;

fn main() {
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
        // Setup renet by adding the client plugin, registering our GameEvent
        // adding an error handler and a event receiver
        .add_plugin(RenetClientPlugin)
        .insert_resource(GameState::default())
        .add_event::<GameEvent>()
        .add_system(handle_renet_error)
        .add_system_to_stage(
            CoreStage::PreUpdate,
            receive_events_from_server.with_run_criteria(run_if_client_connected),
        )
        // Add setup function and register all our app states
        .add_startup_system(setup)
        .add_state(AppState::Initial)
        .add_plugin(StateInitial)
        .add_plugin(StateConnected)
        .add_plugin(StateInGame)
        .add_plugin(StateFinished)
        .add_plugin(StateError)
        // Finally we just run the thing!
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

// If there's any error we just head straight to the error screen
fn handle_renet_error(
    mut renet_error: EventReader<RenetError>,
    mut app_state: ResMut<State<AppState>>,
) {
    for err in renet_error.iter() {
        error!("{}", err);
        app_state.set(AppState::Error).unwrap();
    }
}

fn receive_events_from_server(
    mut client: ResMut<RenetClient>,
    mut game_state: ResMut<GameState>,
    mut game_events: EventWriter<GameEvent>,
    mut app_state: ResMut<State<AppState>>,
) {
    while let Some(message) = client.receive_message(0) {
        // Whenever the server sends a message we know that it must be a game event
        let event: GameEvent = bincode::deserialize(&message).unwrap();
        trace!("{:#?}", event);

        // We trust the server - It's always been good to us!
        // No need to validate the events it is sending us
        game_state.consume(&event);

        // When the game begins or ends move to the appropriate app state
        match event {
            GameEvent::BeginGame { goes_first: _ } => {
                app_state.set(AppState::InGame).unwrap();
            }
            GameEvent::EndGame { reason } => match reason {
                EndGameReason::PlayerWon { winner: _ } => {
                    app_state.set(AppState::Finished).unwrap();
                }
                EndGameReason::PlayerLeft { player_id: _ } => {
                    app_state.set(AppState::Error).unwrap();
                }
            },
            _ => {}
        }

        // Send the event into the bevy event system so systems can react to it
        game_events.send(event);
    }
}
