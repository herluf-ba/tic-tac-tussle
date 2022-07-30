use bevy::prelude::*;

mod states;
// use renet::RenetError;
use states::{
    connected::StateConnected, error::StateError, finished::StateFinished, ingame::StateInGame,
    initial::StateInitial,
};

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
        // .add_plugin(RenetClientPlugin)
        .add_startup_system(setup)
        // .add_system(handle_renet_error)
        .add_state(AppState::Initial)
        .add_plugin(StateInitial)
        .add_plugin(StateConnected)
        .add_plugin(StateInGame)
        .add_plugin(StateFinished)
        .add_plugin(StateError)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

// If there's any error we just head straight to the error screen
// fn handle_renet_error(
//     mut renet_error: EventReader<RenetError>,
//     mut app_state: ResMut<State<AppState>>,
// ) {
//     for err in renet_error.iter() {
//         error!("{}", err);
//         app_state.set(AppState::Error).unwrap();
//     }
// }
