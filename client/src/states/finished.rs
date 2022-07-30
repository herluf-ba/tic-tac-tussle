use bevy::prelude::*;

use crate::AppState;

fn setup() {}

fn update() {}

fn cleanup() {}

pub struct StateFinished;
impl Plugin for StateFinished {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::Finished).with_system(setup));
        app.add_system_set(SystemSet::on_update(AppState::Finished).with_system(update));
        app.add_system_set(SystemSet::on_exit(AppState::Finished).with_system(cleanup));
    }
}
